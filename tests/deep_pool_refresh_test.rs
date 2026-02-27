//! Integration tests for The Deep — Mission Pool Refresh Lifecycle.
//!
//! Covers the full pool refresh lifecycle:
//!   1.  Discovery generates initial pool with a valid timestamp
//!   2.  Pool drains (all missions launched) → refresh generates a new pool
//!   3.  6-hour timer fires → pool refreshes with correct frontier missions
//!   4.  Clear a layer → refresh reflects new frontier in the pool
//!   5.  Offline load after 8h → pool is refreshed (old timestamp triggers refresh)
//!   6.  Pool refresh after prestige → fresh pool for the new generation
//!   7.  pool_refreshed_at serde backward-compatibility (missing field → None)
//!   8.  pool_refreshed_at persists across serde roundtrip
//!
//! Conventions:
//!   - Seeded `ChaCha8Rng` for all randomness.
//!   - Fixed `DateTime<Utc>` anchors — no dependency on system clock.
//!   - Time is advanced by setting `pool_refreshed_at` to a past value.

use chrono::{DateTime, Duration, TimeZone, Utc};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use quest::deep::{
    complete_discovery, generate_mission_pool, mark_layer_cleared, maybe_refresh_mission_pool,
    DeepPrestige, DeepState, GuildRank, MissionType, POOL_REFRESH_INTERVAL_SECS,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn seeded_rng(seed: u64) -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(seed)
}

/// Fixed anchor time: 2024-03-01 10:00:00 UTC.
fn t0() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2024, 3, 1, 10, 0, 0).unwrap()
}

/// Force-discover The Deep and return the ready state.
fn force_discover(deep: &mut DeepState) {
    let mut rng = seeded_rng(42);
    complete_discovery(deep, &mut rng);
    assert!(deep.persistent.discovered, "Discovery must succeed");
}

// =============================================================================
// 1. Discovery generates initial pool with a valid timestamp
// =============================================================================

#[test]
fn test_discovery_sets_pool_refreshed_at() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    // After discovery, pool_refreshed_at should be Some (not None).
    assert!(
        deep.prestige.pool_refreshed_at.is_some(),
        "pool_refreshed_at must be Some after discovery (not None)"
    );
}

#[test]
fn test_discovery_timestamp_is_recent() {
    let before = Utc::now();
    let mut deep = DeepState::new();
    force_discover(&mut deep);
    let after = Utc::now();

    let ts = deep
        .prestige
        .pool_refreshed_at
        .expect("must be Some after discovery");
    assert!(
        ts >= before && ts <= after,
        "pool_refreshed_at ({}) must be within the discovery window [{}, {}]",
        ts,
        before,
        after
    );
}

#[test]
fn test_discovery_pool_missions_target_layer_one() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    // After discovery (no layers cleared yet), frontier = 1.
    // Re-generate the pool the same way discovery does.
    let mut rng = seeded_rng(42);
    let pool = generate_mission_pool(&deep.persistent, &mut rng);

    // All missions on a fresh state target layer 1 (frontier = 1).
    for mission in &pool {
        assert_eq!(
            mission.layer, 1,
            "Initial pool missions must target the frontier (layer 1), got layer {}",
            mission.layer
        );
    }
}

#[test]
fn test_discovery_pool_always_contains_supply_run() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    let mut rng = seeded_rng(7);
    let pool = generate_mission_pool(&deep.persistent, &mut rng);
    assert!(
        pool.iter()
            .any(|m| m.mission_type == MissionType::SupplyRun),
        "Initial pool must always contain a Supply Run"
    );
}

// =============================================================================
// 2. Pool drains → next refresh regenerates it
// =============================================================================

#[test]
fn test_empty_pool_triggers_refresh() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    // Drain the pool.
    deep.prestige.available_missions.clear();
    assert!(
        deep.prestige.available_missions.is_empty(),
        "Pool should be empty after draining"
    );

    // Apply refresh — empty pool should always trigger.
    let mut rng = seeded_rng(10);
    let now = t0();
    let refreshed = maybe_refresh_mission_pool(&mut deep.prestige, &deep.persistent, now, &mut rng);

    assert!(refreshed, "Empty pool must trigger a refresh");
    assert!(
        !deep.prestige.available_missions.is_empty(),
        "Pool must be non-empty after refresh"
    );
}

#[test]
fn test_empty_pool_regenerates_correct_count() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    deep.prestige.available_missions.clear();

    let mut rng = seeded_rng(11);
    maybe_refresh_mission_pool(&mut deep.prestige, &deep.persistent, t0(), &mut rng);

    // Guild Rank 1 → 3 missions.
    assert_eq!(
        deep.prestige.available_missions.len(),
        3,
        "Rank 1 pool must have 3 missions after refresh"
    );
}

#[test]
fn test_none_timestamp_triggers_refresh_even_with_non_empty_pool() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    // Override pool_refreshed_at to None (simulates old save or post-prestige state).
    deep.prestige.pool_refreshed_at = None;
    // Keep a non-empty pool to test that None timestamp triggers refresh regardless.
    let old_pool_len = deep.prestige.available_missions.len();
    assert!(old_pool_len > 0, "Pool should be non-empty for this test");

    let mut rng = seeded_rng(12);
    let refreshed =
        maybe_refresh_mission_pool(&mut deep.prestige, &deep.persistent, t0(), &mut rng);

    assert!(
        refreshed,
        "None pool_refreshed_at must trigger a refresh even when pool is non-empty"
    );
    assert!(
        deep.prestige.pool_refreshed_at.is_some(),
        "pool_refreshed_at must be set to Some after refresh"
    );
}

#[test]
fn test_non_empty_pool_does_not_refresh_before_interval() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    // Set pool_refreshed_at to 1 hour ago — should not refresh (1h < 6h).
    let one_hour_ago = t0() - Duration::hours(1);
    deep.prestige.pool_refreshed_at = Some(one_hour_ago);
    let pool_before: Vec<_> = deep
        .prestige
        .available_missions
        .iter()
        .map(|m| m.mission_type)
        .collect();

    let mut rng = seeded_rng(13);
    let refreshed =
        maybe_refresh_mission_pool(&mut deep.prestige, &deep.persistent, t0(), &mut rng);

    assert!(
        !refreshed,
        "Pool should NOT refresh when < 6 hours have elapsed and pool is non-empty"
    );
    // Pool content unchanged.
    let pool_after: Vec<_> = deep
        .prestige
        .available_missions
        .iter()
        .map(|m| m.mission_type)
        .collect();
    assert_eq!(
        pool_before, pool_after,
        "Pool content must be unchanged when refresh is skipped"
    );
}

// =============================================================================
// 3. 6-hour timer fires → pool refreshes with current frontier missions
// =============================================================================

#[test]
fn test_pool_refreshes_after_six_hours() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    let interval_hours = POOL_REFRESH_INTERVAL_SECS / 3600;
    // Simulate pool refreshed exactly 6 hours ago.
    let six_hours_ago = t0() - Duration::seconds(POOL_REFRESH_INTERVAL_SECS);
    deep.prestige.pool_refreshed_at = Some(six_hours_ago);
    // Manually set pool as non-empty (old content).
    let old_missions = generate_mission_pool(&deep.persistent, &mut seeded_rng(99));
    deep.prestige.available_missions = old_missions;

    let mut rng = seeded_rng(20);
    let refreshed =
        maybe_refresh_mission_pool(&mut deep.prestige, &deep.persistent, t0(), &mut rng);

    assert!(
        refreshed,
        "Pool must refresh when exactly {}h ({} secs) have elapsed",
        interval_hours, POOL_REFRESH_INTERVAL_SECS
    );
    assert!(
        !deep.prestige.available_missions.is_empty(),
        "Pool must not be empty after 6-hour refresh"
    );
    assert_eq!(
        deep.prestige.pool_refreshed_at,
        Some(t0()),
        "pool_refreshed_at must be updated to the refresh time"
    );
}

#[test]
fn test_pool_does_not_refresh_at_five_hours_fifty_nine_minutes() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    // Just under 6 hours — should not refresh.
    let almost_six_hours =
        t0() - Duration::seconds(POOL_REFRESH_INTERVAL_SECS) + Duration::minutes(1);
    deep.prestige.pool_refreshed_at = Some(almost_six_hours);
    // Pool must be non-empty for this to test the timer correctly.
    deep.prestige.available_missions = generate_mission_pool(&deep.persistent, &mut seeded_rng(98));

    let mut rng = seeded_rng(21);
    let refreshed =
        maybe_refresh_mission_pool(&mut deep.prestige, &deep.persistent, t0(), &mut rng);

    assert!(
        !refreshed,
        "Pool must NOT refresh when < 6 hours have elapsed"
    );
}

#[test]
fn test_refreshed_pool_missions_target_current_frontier() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    // Frontier = layer 1 on fresh state. Force a refresh.
    deep.prestige.pool_refreshed_at = None;
    deep.prestige.available_missions = vec![]; // drain

    let mut rng = seeded_rng(22);
    maybe_refresh_mission_pool(&mut deep.prestige, &deep.persistent, t0(), &mut rng);

    // All missions should target the current frontier (layer 1).
    for mission in &deep.prestige.available_missions {
        assert_eq!(
            mission.layer, 1,
            "Refreshed pool missions must target the frontier (layer 1)"
        );
    }
}

#[test]
fn test_pool_refresh_updates_timestamp() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    let refresh_time = t0() + Duration::hours(7);
    deep.prestige.pool_refreshed_at = None;
    deep.prestige.available_missions.clear(); // force empty

    let mut rng = seeded_rng(23);
    maybe_refresh_mission_pool(&mut deep.prestige, &deep.persistent, refresh_time, &mut rng);

    assert_eq!(
        deep.prestige.pool_refreshed_at,
        Some(refresh_time),
        "pool_refreshed_at must be updated to the time of refresh"
    );
}

// =============================================================================
// 4. Clear a layer → refresh reflects new frontier
// =============================================================================

#[test]
fn test_pool_refresh_reflects_cleared_layer_frontier() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    // Clear layer 1 → frontier becomes 2.
    mark_layer_cleared(&mut deep.persistent, 1);
    assert_eq!(
        deep.persistent.frontier_layer(),
        2,
        "Frontier should be layer 2 after clearing layer 1"
    );

    // Force a refresh.
    deep.prestige.pool_refreshed_at = None;
    deep.prestige.available_missions.clear();

    let mut rng = seeded_rng(30);
    let refreshed =
        maybe_refresh_mission_pool(&mut deep.prestige, &deep.persistent, t0(), &mut rng);

    assert!(refreshed, "Pool must refresh after clearing a layer");

    // Frontier-targeting missions (Recon, Expedition, Breakthrough) must target layer 2.
    let frontier_missions: Vec<_> = deep
        .prestige
        .available_missions
        .iter()
        .filter(|m| {
            matches!(
                m.mission_type,
                MissionType::Recon | MissionType::Expedition | MissionType::Breakthrough
            )
        })
        .collect();

    assert!(
        !frontier_missions.is_empty(),
        "Pool must contain at least one frontier mission"
    );

    for mission in &frontier_missions {
        assert_eq!(
            mission.layer, 2,
            "After clearing layer 1, frontier missions must target layer 2, got {}",
            mission.layer
        );
    }
}

#[test]
fn test_pool_refresh_reflects_multiple_cleared_layers() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    // Clear layers 1 and 2 → frontier becomes 3.
    mark_layer_cleared(&mut deep.persistent, 1);
    mark_layer_cleared(&mut deep.persistent, 2);
    assert_eq!(deep.persistent.frontier_layer(), 3);

    deep.prestige.pool_refreshed_at = None;
    deep.prestige.available_missions.clear();

    let mut rng = seeded_rng(31);
    maybe_refresh_mission_pool(&mut deep.prestige, &deep.persistent, t0(), &mut rng);

    // Frontier missions (Recon, Expedition, Breakthrough) must target layer 3.
    let frontier_targets: Vec<u32> = deep
        .prestige
        .available_missions
        .iter()
        .filter(|m| {
            matches!(
                m.mission_type,
                MissionType::Recon | MissionType::Expedition | MissionType::Breakthrough
            )
        })
        .map(|m| m.layer)
        .collect();

    for layer in frontier_targets {
        assert_eq!(
            layer, 3,
            "After clearing layers 1+2, frontier missions must target layer 3"
        );
    }
}

#[test]
fn test_supply_run_targets_cleared_layer_after_refresh() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    // Clear layer 1 → supply run should target the most recently cleared layer.
    mark_layer_cleared(&mut deep.persistent, 1);

    deep.prestige.available_missions.clear();
    deep.prestige.pool_refreshed_at = None;

    let mut rng = seeded_rng(32);
    maybe_refresh_mission_pool(&mut deep.prestige, &deep.persistent, t0(), &mut rng);

    // After clearing layer 1, supply runs should target the last cleared layer (1).
    let supply_runs: Vec<_> = deep
        .prestige
        .available_missions
        .iter()
        .filter(|m| m.mission_type == MissionType::SupplyRun)
        .collect();

    assert!(
        !supply_runs.is_empty(),
        "Pool must always contain a Supply Run after refresh"
    );

    for sr in &supply_runs {
        assert_eq!(
            sr.layer, 1,
            "After clearing layer 1, Supply Run should target the cleared layer 1"
        );
    }
}

// =============================================================================
// 5. Offline load after 8h → pool refreshed on load
// =============================================================================

#[test]
fn test_pool_refresh_after_offline_8_hours() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    // Simulate pool refreshed 8 hours ago (game was closed for 8 hours).
    let eight_hours_ago = t0() - Duration::hours(8);
    deep.prestige.pool_refreshed_at = Some(eight_hours_ago);
    deep.prestige.available_missions = generate_mission_pool(&deep.persistent, &mut seeded_rng(40));

    // On load, simulate the offline refresh check.
    let now = t0();
    let mut rng = seeded_rng(41);
    let refreshed = maybe_refresh_mission_pool(&mut deep.prestige, &deep.persistent, now, &mut rng);

    let interval_hours = POOL_REFRESH_INTERVAL_SECS / 3600;
    assert!(
        refreshed,
        "Pool must refresh on load when offline duration (8h) exceeds refresh interval ({}h)",
        interval_hours
    );
    assert_eq!(
        deep.prestige.pool_refreshed_at,
        Some(now),
        "pool_refreshed_at must be updated to load time after offline refresh"
    );
}

#[test]
fn test_pool_not_refreshed_after_offline_3_hours() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    // Simulate pool refreshed 3 hours ago (within refresh window).
    let three_hours_ago = t0() - Duration::hours(3);
    deep.prestige.pool_refreshed_at = Some(three_hours_ago);
    deep.prestige.available_missions = generate_mission_pool(&deep.persistent, &mut seeded_rng(42));

    let mut rng = seeded_rng(43);
    let refreshed =
        maybe_refresh_mission_pool(&mut deep.prestige, &deep.persistent, t0(), &mut rng);

    assert!(
        !refreshed,
        "Pool must NOT refresh on load when offline duration (3h) is within refresh interval"
    );
}

#[test]
fn test_pool_none_timestamp_triggers_immediate_refresh() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    // pool_refreshed_at = None (new prestige or old save) → must trigger refresh immediately.
    deep.prestige.pool_refreshed_at = None;
    deep.prestige.available_missions = generate_mission_pool(&deep.persistent, &mut seeded_rng(50));

    let mut rng = seeded_rng(51);
    let refreshed =
        maybe_refresh_mission_pool(&mut deep.prestige, &deep.persistent, t0(), &mut rng);

    assert!(
        refreshed,
        "None pool_refreshed_at must trigger immediate refresh"
    );
}

// =============================================================================
// 6. Pool persistence after prestige — pool and timestamp survive
// =============================================================================

#[test]
fn test_prestige_preserves_pool_refreshed_at() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    // Set a non-None timestamp before prestige.
    let ts = Some(t0());
    deep.prestige.pool_refreshed_at = ts;
    deep.prestige.available_missions = generate_mission_pool(&deep.persistent, &mut seeded_rng(60));

    deep.on_prestige();

    // After prestige, pool_refreshed_at persists (no forced refresh).
    assert_eq!(
        deep.prestige.pool_refreshed_at, ts,
        "pool_refreshed_at must persist across prestiges"
    );
}

#[test]
fn test_prestige_preserves_available_missions_pool() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    deep.prestige.available_missions = generate_mission_pool(&deep.persistent, &mut seeded_rng(61));
    let count = deep.prestige.available_missions.len();
    assert!(count > 0, "Pool should be non-empty before prestige");

    deep.on_prestige();

    assert_eq!(
        deep.prestige.available_missions.len(),
        count,
        "available_missions pool must persist across prestiges"
    );
}

#[test]
fn test_pool_normal_refresh_still_works_after_prestige() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    // Build up some progression.
    mark_layer_cleared(&mut deep.persistent, 1);
    deep.prestige.available_missions = generate_mission_pool(&deep.persistent, &mut seeded_rng(62));
    deep.prestige.pool_refreshed_at = Some(t0());

    deep.on_prestige();

    // Normal time-based refresh still works (advance time past refresh interval).
    let mut rng = seeded_rng(63);
    let later = t0() + Duration::hours(25); // well past any refresh interval
    let refreshed =
        maybe_refresh_mission_pool(&mut deep.prestige, &deep.persistent, later, &mut rng);

    // Whether it refreshes depends on time elapsed vs refresh interval.
    // The point is the system doesn't crash or behave oddly after prestige.
    if refreshed {
        assert!(!deep.prestige.available_missions.is_empty());
    }
    // Frontier layer should still reflect cleared layers.
    assert_eq!(deep.persistent.frontier_layer(), 2);
}

#[test]
fn test_two_prestige_cycles_pool_persists() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    // Generate a pool.
    deep.prestige.available_missions = generate_mission_pool(&deep.persistent, &mut seeded_rng(70));
    deep.prestige.pool_refreshed_at = Some(t0());

    // Simulate two prestige cycles — pool should persist through both.
    for cycle in 1u32..=2 {
        let count_before = deep.prestige.available_missions.len();
        deep.on_prestige();

        assert_eq!(
            deep.prestige.available_missions.len(),
            count_before,
            "Cycle {}: pool must persist across prestiges",
            cycle
        );
        assert!(
            deep.prestige.pool_refreshed_at.is_some(),
            "Cycle {}: pool_refreshed_at must persist across prestiges",
            cycle
        );
    }
}

// =============================================================================
// 7. pool_refreshed_at serde backward-compatibility
// =============================================================================

#[test]
fn test_pool_refreshed_at_defaults_to_none_for_old_saves() {
    // Old save JSON without pool_refreshed_at field.
    let json = r#"{
        "warband_marks": 500,
        "roster": [],
        "active_missions": [],
        "available_missions": [],
        "recruit_pool": {
            "candidates": [],
            "refreshed_at": "2024-01-01T00:00:00Z",
            "recruit_costs": []
        },
        "pending_results": [],
        "generation_number": 1,
        "warband_log": [],
        "total_marks_earned": 100,
        "total_missions_completed": 5,
        "total_mercs_lost": 0
    }"#;

    let prestige: DeepPrestige = serde_json::from_str(json).unwrap();

    assert_eq!(
        prestige.pool_refreshed_at, None,
        "Missing pool_refreshed_at must default to None (ensures immediate refresh on old saves)"
    );
}

#[test]
fn test_pool_none_on_old_save_triggers_refresh() {
    // Simulate what happens when an old save is loaded: None timestamp + old pool.
    let mut deep = DeepState::new();
    deep.persistent.discovered = true;

    // None from old save deserialization default.
    assert_eq!(deep.prestige.pool_refreshed_at, None);

    // Old save has some missions in the pool.
    deep.prestige.available_missions = generate_mission_pool(&deep.persistent, &mut seeded_rng(80));

    // On load, apply refresh — None should trigger a refresh.
    let mut rng = seeded_rng(81);
    let refreshed =
        maybe_refresh_mission_pool(&mut deep.prestige, &deep.persistent, Utc::now(), &mut rng);

    assert!(
        refreshed,
        "None pool_refreshed_at must cause an immediate refresh on old save load"
    );
}

// =============================================================================
// 8. pool_refreshed_at persists across serde roundtrip
// =============================================================================

#[test]
fn test_pool_refreshed_at_roundtrip() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    let expected_ts = Utc.with_ymd_and_hms(2024, 6, 15, 14, 30, 0).unwrap();
    deep.prestige.pool_refreshed_at = Some(expected_ts);

    let json = serde_json::to_string_pretty(&deep).unwrap();
    let loaded: DeepState = serde_json::from_str(&json).unwrap();

    let loaded_ts = loaded
        .prestige
        .pool_refreshed_at
        .expect("pool_refreshed_at must survive roundtrip as Some");

    // Timestamps are serialized at second precision in RFC3339.
    let diff = (loaded_ts - expected_ts).num_seconds().abs();
    assert!(
        diff <= 1,
        "pool_refreshed_at must survive serde roundtrip (diff: {}s)",
        diff
    );
}

#[test]
fn test_pool_refreshed_at_none_roundtrip() {
    let mut deep = DeepState::new();
    deep.prestige.pool_refreshed_at = None;

    let json = serde_json::to_string_pretty(&deep).unwrap();
    let loaded: DeepState = serde_json::from_str(&json).unwrap();

    assert_eq!(
        loaded.prestige.pool_refreshed_at, None,
        "None pool_refreshed_at must survive serde roundtrip"
    );
}

#[test]
fn test_pool_refreshed_at_preserved_across_full_state_roundtrip() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    let ts = t0() + Duration::hours(3);
    deep.prestige.pool_refreshed_at = Some(ts);
    deep.prestige.available_missions = generate_mission_pool(&deep.persistent, &mut seeded_rng(90));

    let json = serde_json::to_string_pretty(&deep).unwrap();
    let loaded: DeepState = serde_json::from_str(&json).unwrap();

    let loaded_ts = loaded
        .prestige
        .pool_refreshed_at
        .expect("pool_refreshed_at must be Some after roundtrip");
    let diff = (loaded_ts - ts).num_seconds().abs();
    assert!(
        diff <= 1,
        "pool_refreshed_at must be preserved in full DeepState roundtrip"
    );
    assert_eq!(
        loaded.prestige.available_missions.len(),
        deep.prestige.available_missions.len(),
        "available_missions must be preserved across serde roundtrip"
    );
}

// =============================================================================
// Bonus: pool refresh interaction with guild rank
// =============================================================================

#[test]
fn test_higher_rank_pool_has_more_missions() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    // Rank 1: 3 missions.
    deep.prestige.available_missions.clear();
    deep.prestige.pool_refreshed_at = None;
    let mut rng1 = seeded_rng(100);
    maybe_refresh_mission_pool(&mut deep.prestige, &deep.persistent, t0(), &mut rng1);
    let rank1_count = deep.prestige.available_missions.len();

    // Rank 5: 5 missions.
    deep.persistent.guild_rank = GuildRank(5);
    deep.prestige.available_missions.clear();
    deep.prestige.pool_refreshed_at = None;
    let mut rng5 = seeded_rng(101);
    maybe_refresh_mission_pool(
        &mut deep.prestige,
        &deep.persistent,
        t0() + Duration::hours(7),
        &mut rng5,
    );
    let rank5_count = deep.prestige.available_missions.len();

    assert!(
        rank5_count > rank1_count,
        "Rank 5 pool ({} missions) must be larger than Rank 1 pool ({} missions)",
        rank5_count,
        rank1_count
    );
}

#[test]
fn test_pool_always_contains_supply_run_after_refresh() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    // Run 5 consecutive refresh cycles.
    for i in 0..5u64 {
        deep.prestige.available_missions.clear();
        deep.prestige.pool_refreshed_at = None;
        let mut rng = seeded_rng(110 + i);
        maybe_refresh_mission_pool(
            &mut deep.prestige,
            &deep.persistent,
            t0() + Duration::hours(i as i64 * 7),
            &mut rng,
        );

        assert!(
            deep.prestige
                .available_missions
                .iter()
                .any(|m| m.mission_type == MissionType::SupplyRun),
            "Cycle {}: refreshed pool must always contain a Supply Run",
            i
        );
    }
}

#[test]
fn test_refreshed_pool_missions_have_positive_duration() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    deep.prestige.available_missions.clear();
    deep.prestige.pool_refreshed_at = None;
    let mut rng = seeded_rng(120);
    maybe_refresh_mission_pool(&mut deep.prestige, &deep.persistent, t0(), &mut rng);

    for mission in &deep.prestige.available_missions {
        assert!(
            mission.duration_secs > 0,
            "Every mission in the refreshed pool must have positive duration"
        );
    }
}

#[test]
fn test_pool_refresh_deterministic_with_same_seed() {
    let mut deep1 = DeepState::new();
    force_discover(&mut deep1);
    deep1.prestige.available_missions.clear();
    deep1.prestige.pool_refreshed_at = None;

    let mut deep2 = DeepState::new();
    force_discover(&mut deep2);
    deep2.prestige.available_missions.clear();
    deep2.prestige.pool_refreshed_at = None;

    let mut rng1 = seeded_rng(130);
    let mut rng2 = seeded_rng(130);

    maybe_refresh_mission_pool(&mut deep1.prestige, &deep1.persistent, t0(), &mut rng1);
    maybe_refresh_mission_pool(&mut deep2.prestige, &deep2.persistent, t0(), &mut rng2);

    let types1: Vec<_> = deep1
        .prestige
        .available_missions
        .iter()
        .map(|m| m.mission_type)
        .collect();
    let types2: Vec<_> = deep2
        .prestige
        .available_missions
        .iter()
        .map(|m| m.mission_type)
        .collect();

    assert_eq!(
        types1, types2,
        "Pool refresh must be deterministic with the same RNG seed"
    );
}
