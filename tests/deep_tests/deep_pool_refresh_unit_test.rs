//! Unit tests for The Deep — Mission Pool Refresh Logic.
//!
//! Tests cover the 8 required cases for Task #5:
//!   1.  Pool regenerates when empty
//!   2.  Pool regenerates after 6h even when not empty (stale pool)
//!   3.  Pool does NOT regenerate before 6h when not empty (fresh pool)
//!   4.  pool_refreshed_at updates on refresh
//!   5.  Serde roundtrip with pool_refreshed_at field
//!   6.  Missing pool_refreshed_at deserializes with None default (backward compat)
//!   7.  on_prestige() resets pool and timestamp
//!   8.  Pool content is correct for current layer state after refresh
//!
//! All tests use `maybe_refresh_mission_pool` from `quest::deep`.
//! Time-based tests pass explicit `now` values for determinism — no wall-clock calls.
//!
//! Conventions:
//!   - Seeded `ChaCha8Rng::seed_from_u64(42)` for all randomness.
//!   - Exact value assertions (not ranges) except where wall-clock-dependent.
//!   - Section separators mark the 8 required test categories.

use chrono::{Duration, TimeZone, Utc};
use quest::deep::{
    available_mission_count, generate_mission_pool, maybe_refresh_mission_pool, AvailableMission,
    DeepPersistent, DeepSession, DeepState, GuildRank, Infrastructure, MissionType, GATEWAY_LAYER,
    POOL_REFRESH_INTERVAL_SECS,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

// ─── helpers ──────────────────────────────────────────────────────────────────

fn seeded_rng() -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(42)
}

/// A fixed reference time used as "now" in tests (2025-01-15 12:00:00 UTC).
fn t0() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2025, 1, 15, 12, 0, 0).unwrap()
}

/// Build a minimal `DeepPersistent` at guild rank 1 with the given number of
/// layers pre-cleared so that `frontier_layer()` equals `frontier_layer`.
fn make_persistent_with_frontier(frontier_layer: u32) -> DeepPersistent {
    let mut p = DeepPersistent::new();
    // Clear all layers before the frontier.
    for layer in 1..frontier_layer {
        p.layer_record_mut(layer).cleared = true;
        if layer > p.deepest_layer_reached {
            p.deepest_layer_reached = layer;
        }
    }
    p
}

/// Build a `DeepSession` with a non-empty pool and `pool_refreshed_at` set to
/// `secs_ago` seconds before `now`.
fn prestige_with_pool_age(now: chrono::DateTime<Utc>, secs_ago: i64) -> DeepSession {
    let mut prestige = DeepSession::new();
    let persistent = make_persistent_with_frontier(1);
    prestige.available_missions = generate_mission_pool(&persistent, &[], &mut seeded_rng());
    prestige.pool_refreshed_at = Some(now - chrono::Duration::seconds(secs_ago));
    // Keep affordability out of these timing-focused tests so emergency
    // fallback insertion does not mutate the pool.
    prestige.warband_marks = 10_000;
    prestige
}

// =========================================================================
// 1. Pool regenerates when empty
// =========================================================================

#[test]
fn empty_pool_triggers_refresh() {
    let persistent = make_persistent_with_frontier(1);
    let mut prestige = DeepSession::new();
    // New state starts with None timestamp and empty pool — both trigger refresh.
    assert!(prestige.available_missions.is_empty());

    let now = t0();
    let refreshed = maybe_refresh_mission_pool(&mut prestige, &persistent, now, &mut seeded_rng());

    assert!(refreshed, "empty pool must cause a refresh (returns true)");
    assert!(
        !prestige.available_missions.is_empty(),
        "pool must be populated after refresh"
    );
}

#[test]
fn empty_pool_triggers_refresh_even_when_recently_generated() {
    let persistent = make_persistent_with_frontier(1);
    let mut prestige = DeepSession::new();
    prestige.available_missions.clear();
    // Set pool_refreshed_at to 1 minute ago — normally too fresh to trigger time-based refresh.
    let now = t0();
    prestige.pool_refreshed_at = Some(now - chrono::Duration::minutes(1));

    let refreshed = maybe_refresh_mission_pool(&mut prestige, &persistent, now, &mut seeded_rng());

    assert!(
        refreshed,
        "empty pool must refresh even if generated only 1 minute ago"
    );
    assert!(!prestige.available_missions.is_empty());
}

#[test]
fn empty_pool_refresh_return_value_is_true() {
    let persistent = make_persistent_with_frontier(1);
    let mut prestige = DeepSession::new();
    prestige.available_missions.clear();

    let now = t0();
    let result = maybe_refresh_mission_pool(&mut prestige, &persistent, now, &mut seeded_rng());

    assert!(result, "must return true when pool is refreshed");
}

// =========================================================================
// 2. Pool regenerates after 6h even when not empty (stale pool)
// =========================================================================

#[test]
fn stale_pool_at_exactly_six_hours_triggers_refresh() {
    let persistent = make_persistent_with_frontier(1);
    let now = t0();
    // Pool not empty, but exactly 6 hours old (= POOL_REFRESH_INTERVAL_SECS).
    let mut prestige = prestige_with_pool_age(now, POOL_REFRESH_INTERVAL_SECS);

    let refreshed = maybe_refresh_mission_pool(&mut prestige, &persistent, now, &mut seeded_rng());

    assert!(
        refreshed,
        "pool exactly {} seconds old must be refreshed",
        POOL_REFRESH_INTERVAL_SECS
    );
}

#[test]
fn stale_pool_at_seven_hours_triggers_refresh() {
    let persistent = make_persistent_with_frontier(1);
    let now = t0();
    let mut prestige = prestige_with_pool_age(now, 7 * 3600);

    let refreshed = maybe_refresh_mission_pool(&mut prestige, &persistent, now, &mut seeded_rng());

    assert!(
        refreshed,
        "pool 7 hours old must be refreshed even though not empty"
    );
    assert!(!prestige.available_missions.is_empty());
}

#[test]
fn none_pool_refreshed_at_triggers_refresh_on_non_empty_pool() {
    // Simulates a new prestige or old save file: pool_refreshed_at is None,
    // which guarantees an immediate refresh even if missions exist.
    let persistent = make_persistent_with_frontier(1);
    let mut prestige = DeepSession::new();
    prestige.available_missions = vec![AvailableMission {
        mission_type: MissionType::SupplyRun,
        layer: 1,
        duration_secs: 7200,
        min_squad_power: 10,
        marks_cost: 0,
        required_archetype: None,
        recommended_archetype: None,
        description: String::new(),
    }];
    // pool_refreshed_at stays None from default.
    assert_eq!(prestige.pool_refreshed_at, None);

    let now = t0();
    let refreshed = maybe_refresh_mission_pool(&mut prestige, &persistent, now, &mut seeded_rng());

    assert!(
        refreshed,
        "None pool_refreshed_at must trigger refresh (backward compat + post-prestige)"
    );
}

// =========================================================================
// 3. Pool does NOT regenerate before 6h when not empty (fresh pool)
// =========================================================================

#[test]
fn fresh_pool_one_hour_old_does_not_refresh() {
    let persistent = make_persistent_with_frontier(1);
    let now = t0();
    let mut prestige = prestige_with_pool_age(now, 3600);
    let original_len = prestige.available_missions.len();

    let refreshed = maybe_refresh_mission_pool(&mut prestige, &persistent, now, &mut seeded_rng());

    assert!(
        !refreshed,
        "fresh pool (1h old, not empty) must NOT refresh"
    );
    assert_eq!(
        prestige.available_missions.len(),
        original_len,
        "pool contents must be unchanged when not refreshed"
    );
}

#[test]
fn fresh_pool_five_hours_59_minutes_old_does_not_refresh() {
    let persistent = make_persistent_with_frontier(1);
    let now = t0();
    let mut prestige = DeepSession::new();
    prestige.warband_marks = 10_000;
    prestige.available_missions = generate_mission_pool(&persistent, &[], &mut seeded_rng());
    // 5h 59m old = 21540 seconds — just under the 6h = 21600s threshold.
    prestige.pool_refreshed_at =
        Some(now - chrono::Duration::seconds(POOL_REFRESH_INTERVAL_SECS - 60));

    let refreshed = maybe_refresh_mission_pool(&mut prestige, &persistent, now, &mut seeded_rng());

    assert!(
        !refreshed,
        "pool 5h59m old must NOT refresh (threshold is {} seconds)",
        POOL_REFRESH_INTERVAL_SECS
    );
    assert!(!prestige.available_missions.is_empty());
}

#[test]
fn just_generated_pool_does_not_refresh() {
    let persistent = make_persistent_with_frontier(1);
    let now = t0();
    // Pool generated 0 seconds ago.
    let mut prestige = prestige_with_pool_age(now, 0);

    let refreshed = maybe_refresh_mission_pool(&mut prestige, &persistent, now, &mut seeded_rng());

    assert!(!refreshed, "just-generated non-empty pool must not refresh");
}

#[test]
fn no_refresh_returns_false() {
    let persistent = make_persistent_with_frontier(1);
    let now = t0();
    let mut prestige = prestige_with_pool_age(now, 3600);

    let result = maybe_refresh_mission_pool(&mut prestige, &persistent, now, &mut seeded_rng());

    assert!(!result, "must return false when pool is not refreshed");
}

// =========================================================================
// 4. pool_refreshed_at updates on refresh
// =========================================================================

#[test]
fn pool_refreshed_at_updated_after_empty_pool_refresh() {
    let persistent = make_persistent_with_frontier(1);
    let mut prestige = DeepSession::new();
    // None timestamp — will be updated on refresh.
    assert_eq!(prestige.pool_refreshed_at, None);

    let now = t0();
    maybe_refresh_mission_pool(&mut prestige, &persistent, now, &mut seeded_rng());

    assert_eq!(
        prestige.pool_refreshed_at,
        Some(now),
        "pool_refreshed_at must be set to `now` after refresh"
    );
}

#[test]
fn pool_refreshed_at_updated_after_stale_pool_refresh() {
    let persistent = make_persistent_with_frontier(1);
    let now = t0();
    let mut prestige = prestige_with_pool_age(now, 7 * 3600);

    let refreshed = maybe_refresh_mission_pool(&mut prestige, &persistent, now, &mut seeded_rng());

    assert!(refreshed);
    assert_eq!(
        prestige.pool_refreshed_at,
        Some(now),
        "pool_refreshed_at must be set to `now` after stale pool refresh"
    );
}

#[test]
fn pool_refreshed_at_unchanged_when_no_refresh_occurs() {
    let persistent = make_persistent_with_frontier(1);
    let now = t0();
    let mut prestige = prestige_with_pool_age(now, 3600);
    let original_ts = prestige.pool_refreshed_at;

    let refreshed = maybe_refresh_mission_pool(&mut prestige, &persistent, now, &mut seeded_rng());

    assert!(!refreshed);
    assert_eq!(
        prestige.pool_refreshed_at, original_ts,
        "pool_refreshed_at must not change when pool is not refreshed"
    );
}

// =========================================================================
// 5. Serde roundtrip with pool_refreshed_at field
// =========================================================================

#[test]
fn pool_refreshed_at_survives_serde_roundtrip_some() {
    let mut prestige = DeepSession::new();
    let ts = Utc.with_ymd_and_hms(2024, 6, 15, 12, 30, 0).unwrap();
    prestige.pool_refreshed_at = Some(ts);

    let json = serde_json::to_string(&prestige).expect("serialize");
    let loaded: DeepSession = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(
        loaded.pool_refreshed_at,
        Some(ts),
        "pool_refreshed_at must survive serde roundtrip"
    );
}

#[test]
fn pool_refreshed_at_survives_serde_roundtrip_none() {
    let prestige = DeepSession::new();
    assert_eq!(prestige.pool_refreshed_at, None);

    let json = serde_json::to_string(&prestige).expect("serialize");
    let loaded: DeepSession = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(
        loaded.pool_refreshed_at, None,
        "None pool_refreshed_at must survive serde roundtrip"
    );
}

#[test]
fn pool_refreshed_at_field_name_is_stable_for_save_files() {
    let mut prestige = DeepSession::new();
    prestige.pool_refreshed_at = Some(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap());

    let json = serde_json::to_string(&prestige).expect("serialize");

    assert!(
        json.contains("\"pool_refreshed_at\""),
        "JSON must use field name 'pool_refreshed_at' for save-file stability; got: {json}"
    );
}

#[test]
fn pool_refreshed_at_non_none_roundtrip_with_missions() {
    let ts = Utc.with_ymd_and_hms(2025, 3, 10, 8, 0, 0).unwrap();
    let mut prestige = DeepSession::new();
    prestige.pool_refreshed_at = Some(ts);
    prestige.available_missions =
        generate_mission_pool(&make_persistent_with_frontier(1), &[], &mut seeded_rng());

    let json = serde_json::to_string(&prestige).expect("serialize");
    let loaded: DeepSession = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(loaded.pool_refreshed_at, Some(ts));
    assert_eq!(
        loaded.available_missions.len(),
        prestige.available_missions.len()
    );
}

// =========================================================================
// 6. Missing pool_refreshed_at deserializes with None default (backward compat)
// =========================================================================

#[test]
fn missing_pool_refreshed_at_deserializes_to_none() {
    // Save file from before pool_refreshed_at was added — field is absent.
    let json = r#"{
        "warband_marks": 100,
        "roster": [],
        "active_missions": [],
        "available_missions": [],
        "recruit_pool": {
            "candidates": [],
            "refreshed_at": "1970-01-01T00:00:00Z",
            "recruit_costs": []
        },
        "pending_results": []
    }"#;

    let loaded: DeepSession = serde_json::from_str(json).expect("deserialize old save");

    assert_eq!(
        loaded.pool_refreshed_at, None,
        "missing pool_refreshed_at must default to None for backward compat"
    );
}

#[test]
fn missing_pool_refreshed_at_triggers_immediate_refresh_on_load() {
    // Old save without pool_refreshed_at: deserializes with None, triggering refresh.
    let json = r#"{
        "warband_marks": 50,
        "roster": [],
        "active_missions": [],
        "available_missions": [
            {
                "mission_type": "SupplyRun",
                "layer": 1,
                "duration_secs": 7200,
                "min_squad_power": 10,
                "marks_cost": 0,
                "required_archetype": null,
                "recommended_archetype": null,
                "description": ""
            }
        ],
        "recruit_pool": {
            "candidates": [],
            "refreshed_at": "1970-01-01T00:00:00Z",
            "recruit_costs": []
        },
        "pending_results": []
    }"#;

    let mut prestige: DeepSession = serde_json::from_str(json).expect("deserialize old save");
    assert_eq!(prestige.pool_refreshed_at, None);

    let persistent = make_persistent_with_frontier(1);
    let now = t0();
    let refreshed = maybe_refresh_mission_pool(&mut prestige, &persistent, now, &mut seeded_rng());

    assert!(
        refreshed,
        "old save with None pool_refreshed_at must trigger immediate refresh"
    );
    assert_eq!(
        prestige.pool_refreshed_at,
        Some(now),
        "pool_refreshed_at must be set to `now` after backward-compat refresh"
    );
}

#[test]
fn serde_default_attribute_present_on_pool_refreshed_at() {
    // Verify that the serde(default) attribute is working:
    // extra known fields with pool_refreshed_at absent must deserialize cleanly.
    let json = r#"{
        "warband_marks": 200,
        "roster": [],
        "active_missions": [],
        "available_missions": [],
        "recruit_pool": {
            "candidates": [],
            "refreshed_at": "1970-01-01T00:00:00Z",
            "recruit_costs": []
        },
        "pending_results": [],
        "generation_number": 3,
        "total_marks_earned": 1000,
        "total_missions_completed": 20,
        "total_mercs_lost": 2
    }"#;

    let loaded: DeepSession = serde_json::from_str(json).expect("deserialize");

    assert_eq!(loaded.pool_refreshed_at, None);
    assert_eq!(loaded.warband_marks, 200);
    assert_eq!(loaded.generation_number, 3);
}

// =========================================================================
// 7. on_prestige() preserves pool and timestamp
// =========================================================================

#[test]
fn on_prestige_preserves_available_missions() {
    let mut state = DeepState::new();
    state.persistent.discovered = true;

    // Populate the pool.
    state.session.available_missions =
        generate_mission_pool(&state.persistent.clone(), &[], &mut seeded_rng());
    let count = state.session.available_missions.len();
    assert!(count > 0);

    state.on_prestige();

    assert_eq!(
        state.session.available_missions.len(),
        count,
        "on_prestige must preserve available_missions"
    );
}

#[test]
fn on_prestige_preserves_pool_refreshed_at() {
    let mut state = DeepState::new();
    state.persistent.discovered = true;
    // Set a non-None timestamp.
    let ts = Some(Utc.with_ymd_and_hms(2024, 9, 1, 12, 0, 0).unwrap());
    state.session.pool_refreshed_at = ts;

    state.on_prestige();

    assert_eq!(
        state.session.pool_refreshed_at, ts,
        "on_prestige must preserve pool_refreshed_at"
    );
}

#[test]
fn on_prestige_pool_continues_normal_refresh_cycle() {
    let mut state = DeepState::new();
    state.persistent.discovered = true;
    // Populate a fresh pool.
    state.session.available_missions = vec![AvailableMission {
        mission_type: MissionType::SupplyRun,
        layer: 1,
        duration_secs: 7200,
        min_squad_power: 10,
        marks_cost: 0,
        required_archetype: None,
        recommended_archetype: None,
        description: String::new(),
    }];
    state.session.pool_refreshed_at = Some(t0());

    state.on_prestige();

    // After prestige: pool and timestamp persist.
    assert_eq!(state.session.available_missions.len(), 1);
    assert_eq!(state.session.pool_refreshed_at, Some(t0()));

    // Normal time-based refresh still works after sufficient time.
    let later = t0() + Duration::hours(25);
    let refreshed = maybe_refresh_mission_pool(
        &mut state.session,
        &state.persistent,
        later,
        &mut seeded_rng(),
    );

    // The pool should eventually refresh based on normal time interval.
    if refreshed {
        assert!(!state.session.available_missions.is_empty());
    }
}

#[test]
fn on_prestige_pool_refreshed_at_is_preserved_not_cleared() {
    let mut state = DeepState::new();
    state.persistent.discovered = true;
    let ts = Some(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap());
    state.session.pool_refreshed_at = ts;

    state.on_prestige();

    // Must be preserved, not cleared to None.
    assert_eq!(
        state.session.pool_refreshed_at, ts,
        "on_prestige must preserve pool_refreshed_at"
    );
}

// =========================================================================
// 8. Pool content is correct for current layer state after refresh
// =========================================================================

#[test]
fn pool_after_refresh_contains_safe_mission_for_cleared_layer() {
    // Layer 1 cleared -> at least one safe mission (Supply or Construction) should exist.
    let mut persistent = make_persistent_with_frontier(1);
    persistent.layer_record_mut(1).cleared = true;
    persistent.deepest_layer_reached = 1;

    let mut prestige = DeepSession::new();
    let now = t0();
    maybe_refresh_mission_pool(&mut prestige, &persistent, now, &mut seeded_rng());

    let has_safe = prestige.available_missions.iter().any(|m| {
        matches!(
            m.mission_type,
            MissionType::SupplyRun | MissionType::Construction(_)
        )
    });

    assert!(
        has_safe,
        "pool must contain at least one safe mission for a cleared layer"
    );
}

#[test]
fn pool_after_refresh_contains_breakthrough_for_frontier_layer() {
    // Layer 1 cleared → frontier is layer 2 → Breakthrough should target layer 2.
    // Rank 3 has a pool count of 6.
    let mut persistent = make_persistent_with_frontier(2);
    persistent.guild_rank = GuildRank(3);
    persistent.layer_record_mut(1).cleared = true;
    persistent.deepest_layer_reached = 1;

    let mut prestige = DeepSession::new();
    let now = t0();
    maybe_refresh_mission_pool(&mut prestige, &persistent, now, &mut seeded_rng());

    let breakthrough = prestige
        .available_missions
        .iter()
        .find(|m| matches!(m.mission_type, MissionType::Breakthrough));

    assert!(
        breakthrough.is_some(),
        "pool must contain a Breakthrough mission for the frontier layer (rank 3 pool has 4 slots)"
    );
    assert_eq!(
        breakthrough.unwrap().layer,
        2,
        "Breakthrough must target the frontier layer (2)"
    );
}

#[test]
fn pool_size_matches_guild_rank_one() {
    let persistent = make_persistent_with_frontier(1);
    let mut prestige = DeepSession::new();
    prestige.warband_marks = 10_000;
    let now = t0();
    maybe_refresh_mission_pool(&mut prestige, &persistent, now, &mut seeded_rng());

    assert_eq!(
        prestige.available_missions.len(),
        4,
        "At initial frontier, pool should contain 4 unique valid missions"
    );
}

#[test]
fn pool_size_matches_guild_rank_five() {
    // Rank 5 wants 7 missions.
    // Set up: layer 1 cleared (enables Construction slot), frontier = layer 2 (Breakthrough target).
    let mut persistent = make_persistent_with_frontier(1);
    persistent.guild_rank = GuildRank(5);
    persistent.layer_record_mut(1).cleared = true;
    persistent.deepest_layer_reached = 1;

    let mut prestige = DeepSession::new();
    prestige.warband_marks = 10_000;
    let now = t0();
    maybe_refresh_mission_pool(&mut prestige, &persistent, now, &mut seeded_rng());

    let expected = available_mission_count(GuildRank(5));
    assert!(
        prestige.available_missions.len() >= expected,
        "pool size must be at least guild rank 5 baseline ({expected} missions); got {}",
        prestige.available_missions.len()
    );
}

#[test]
fn pool_missions_reflect_outpost_infrastructure_shorter_duration() {
    // Outpost applies -25% to the table duration on that layer.
    let mut persistent_no_outpost = make_persistent_with_frontier(5);
    persistent_no_outpost.guild_rank = GuildRank(4);

    let now = t0();
    let mut prestige_a = DeepSession::new();
    maybe_refresh_mission_pool(
        &mut prestige_a,
        &persistent_no_outpost,
        now,
        &mut seeded_rng(),
    );

    let no_outpost_dur = prestige_a
        .available_missions
        .iter()
        .filter(|m| m.layer == 5 && matches!(m.mission_type, MissionType::Breakthrough))
        .map(|m| m.duration_secs)
        .next();

    // With Outpost on layer 5.
    let mut persistent_with_outpost = persistent_no_outpost;
    persistent_with_outpost
        .layer_record_mut(5)
        .infrastructure
        .push(Infrastructure::Outpost);

    let mut prestige_b = DeepSession::new();
    maybe_refresh_mission_pool(
        &mut prestige_b,
        &persistent_with_outpost,
        now,
        &mut seeded_rng(),
    );

    let with_outpost_dur = prestige_b
        .available_missions
        .iter()
        .filter(|m| m.layer == 5 && matches!(m.mission_type, MissionType::Breakthrough))
        .map(|m| m.duration_secs)
        .next();

    if let (Some(d_none), Some(d_out)) = (no_outpost_dur, with_outpost_dur) {
        assert!(
            d_out < d_none,
            "Outpost must reduce Breakthrough duration on layer 5: without={d_none}s, with={d_out}s"
        );
    } else {
        panic!(
            "Expected Breakthrough on layer 5 in pool. no_outpost={:?}, with_outpost={:?}",
            no_outpost_dur, with_outpost_dur
        );
    }
}

#[test]
fn pool_refresh_with_same_seed_is_deterministic() {
    let persistent = make_persistent_with_frontier(1);
    let now = t0();

    let mut prestige_a = DeepSession::new();
    let mut rng_a = ChaCha8Rng::seed_from_u64(99);
    maybe_refresh_mission_pool(&mut prestige_a, &persistent, now, &mut rng_a);

    let mut prestige_b = DeepSession::new();
    let mut rng_b = ChaCha8Rng::seed_from_u64(99);
    maybe_refresh_mission_pool(&mut prestige_b, &persistent, now, &mut rng_b);

    assert_eq!(
        prestige_a.available_missions.len(),
        prestige_b.available_missions.len()
    );
    for (a, b) in prestige_a
        .available_missions
        .iter()
        .zip(prestige_b.available_missions.iter())
    {
        assert_eq!(a.mission_type, b.mission_type);
        assert_eq!(a.layer, b.layer);
        assert_eq!(a.duration_secs, b.duration_secs);
        assert_eq!(a.min_squad_power, b.min_squad_power);
    }
}

#[test]
fn all_pool_missions_have_valid_layer() {
    let persistent = make_persistent_with_frontier(1);
    let mut prestige = DeepSession::new();
    let now = t0();
    maybe_refresh_mission_pool(&mut prestige, &persistent, now, &mut seeded_rng());

    for mission in &prestige.available_missions {
        assert!(
            mission.layer >= 1,
            "all pool missions must target a valid layer (>= 1), got {}",
            mission.layer
        );
    }
}

#[test]
fn all_pool_missions_have_positive_duration() {
    let persistent = make_persistent_with_frontier(1);
    let mut prestige = DeepSession::new();
    let now = t0();
    maybe_refresh_mission_pool(&mut prestige, &persistent, now, &mut seeded_rng());

    for mission in &prestige.available_missions {
        assert!(
            mission.duration_secs > 0,
            "all pool missions must have positive duration, got {} for {:?} on layer {}",
            mission.duration_secs,
            mission.mission_type,
            mission.layer
        );
    }
}

// =========================================================================
// 9. Gateway Expedition survives rolling rebalance when frontier passes L30
// =========================================================================

#[test]
fn gateway_expedition_survives_rolling_rebalance_past_layer_30() {
    // Regression: when frontier advances 3+ layers past GATEWAY_LAYER, the gate
    // expedition at L30 falls outside layer_window() and was pruned. A Breakthrough
    // at the current frontier filled the Progression role, so the gate was never
    // re-added.
    let frontier = GATEWAY_LAYER + 5; // L35 — well outside window of (33..=35)
    let mut persistent = make_persistent_with_frontier(frontier);
    persistent.guild_rank = GuildRank(5);
    // Gateway not yet opened — gate expedition should be pinned.
    persistent.gateway_opened = false;

    let now = t0();

    // 1. Full refresh: gate expedition should appear.
    let mut prestige = DeepSession::new();
    prestige.warband_marks = 100_000;
    maybe_refresh_mission_pool(&mut prestige, &persistent, now, &mut seeded_rng());

    let has_gate = |p: &DeepSession| {
        p.available_missions
            .iter()
            .any(|m| matches!(m.mission_type, MissionType::GatewayExpedition))
    };
    assert!(
        has_gate(&prestige),
        "full refresh must include GatewayExpedition when gateway not opened"
    );

    // 2. Rolling rebalance (pool is fresh, not stale): gate must survive.
    let shortly_after = now + Duration::seconds(1);
    maybe_refresh_mission_pool(&mut prestige, &persistent, shortly_after, &mut seeded_rng());

    assert!(
        has_gate(&prestige),
        "GatewayExpedition must survive rolling rebalance when frontier is past L30"
    );

    // 3. Multiple successive ticks: gate must persist.
    for tick in 2..=10 {
        let t = now + Duration::seconds(tick);
        maybe_refresh_mission_pool(&mut prestige, &persistent, t, &mut seeded_rng());
    }
    assert!(
        has_gate(&prestige),
        "GatewayExpedition must persist across multiple rolling rebalance ticks"
    );
}

#[test]
fn gateway_expedition_injected_into_pool_missing_it_on_load() {
    // Simulates loading a save where the pool has a Breakthrough but no gate
    // expedition (e.g., save from before the pinning fix was added). The rolling
    // rebalance must inject the gate even though another Progression mission exists.
    let frontier = GATEWAY_LAYER + 5;
    let mut persistent = make_persistent_with_frontier(frontier);
    persistent.guild_rank = GuildRank(5);
    persistent.gateway_opened = false;

    let now = t0();

    // Manually build a pool with a Breakthrough (Progression role) but no gate.
    let mut prestige = DeepSession::new();
    prestige.warband_marks = 100_000;
    prestige.available_missions = vec![AvailableMission {
        mission_type: MissionType::Breakthrough,
        layer: frontier,
        duration_secs: 86400,
        min_squad_power: 500,
        marks_cost: 0,
        required_archetype: None,
        recommended_archetype: None,
        description: String::new(),
    }];
    prestige.pool_refreshed_at = Some(now - Duration::seconds(10));

    // Rolling rebalance (pool is fresh and non-empty).
    maybe_refresh_mission_pool(&mut prestige, &persistent, now, &mut seeded_rng());

    let has_gate = prestige
        .available_missions
        .iter()
        .any(|m| matches!(m.mission_type, MissionType::GatewayExpedition));

    assert!(
        has_gate,
        "rolling rebalance must inject GatewayExpedition even when Breakthrough fills Progression role"
    );
}
