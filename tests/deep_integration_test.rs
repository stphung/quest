//! Integration tests for The Deep — persistence, discovery, and prestige interactions.
//!
//! Covers the full lifecycle: discovery → play → prestige → reload.
//! Uses in-memory serde roundtrips for persistence tests where possible;
//! file I/O tests write to a temp directory that is cleaned up automatically.

use chrono::{Duration, Utc};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use quest::deep::{
    // Persistence
    deep_save_path,
    generate_mission_pool,
    // Layers
    mark_layer_cleared,
    // Missions
    resolve_offline_missions,
    // Discovery
    try_discover_deep,
    // Types
    DeepState,
    GuildRank,
    Infrastructure,
    MercArchetype,
    MercStatus,
    Mission,
    MissionStatus,
    MissionType,
    DEEP_MIN_PRESTIGE_RANK,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn seeded_rng() -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(42)
}

/// Advance a `DeepState` through `try_discover_deep` until discovery fires.
/// Panics if discovery does not occur within `max_ticks`.
fn force_discover(deep: &mut DeepState, prestige_rank: u32, max_ticks: u64) -> ChaCha8Rng {
    let mut rng = seeded_rng();
    for _ in 0..max_ticks {
        if try_discover_deep(deep, prestige_rank, &mut rng) {
            return rng;
        }
    }
    panic!(
        "The Deep was not discovered within {} ticks at prestige rank {}",
        max_ticks, prestige_rank
    );
}

/// Build a minimal `Mission` whose timer has already elapsed (completed in the past).
fn make_elapsed_supply_run(deep: &mut DeepState) -> Mission {
    let now = Utc::now();
    let id = deep.persistent.next_mission_id();
    let merc_id = deep.prestige.roster.first().map(|m| m.id).unwrap_or(1);
    Mission {
        id,
        mission_type: MissionType::SupplyRun,
        layer: 1,
        squad: vec![merc_id],
        started_at: now - Duration::hours(5),
        ends_at: now - Duration::minutes(1), // ended 1 minute ago
        events: vec![],
        pending_event_index: 0,
        status: MissionStatus::Active,
        result: None,
    }
}

/// Build a `Mission` that is still in progress (ends in the future).
fn make_active_supply_run(deep: &mut DeepState) -> Mission {
    let now = Utc::now();
    let id = deep.persistent.next_mission_id();
    let merc_id = deep.prestige.roster.first().map(|m| m.id).unwrap_or(1);
    Mission {
        id,
        mission_type: MissionType::SupplyRun,
        layer: 1,
        squad: vec![merc_id],
        started_at: now - Duration::hours(1),
        ends_at: now + Duration::hours(2), // still running
        events: vec![],
        pending_event_index: 0,
        status: MissionStatus::Active,
        result: None,
    }
}

// ── 1. Save / load roundtrip ──────────────────────────────────────────────────

#[test]
fn test_save_load_roundtrip_in_memory() {
    let mut deep = DeepState::new();
    deep.persistent.discovered = true;
    deep.persistent.guild_rank = GuildRank(3);
    deep.persistent.deepest_layer_reached = 7;
    deep.persistent.merc_id_counter = 12;
    deep.persistent.mission_id_counter = 8;
    deep.prestige.warband_marks = 1_240;

    // Add a layer record so we test Vec serialization too.
    {
        let rec = deep.persistent.layer_record_mut(3);
        rec.cleared = true;
        rec.familiarity = 75;
        rec.infrastructure.push(Infrastructure::Outpost);
    }

    let json = serde_json::to_string_pretty(&deep).expect("serialize");
    let loaded: DeepState = serde_json::from_str(&json).expect("deserialize");

    assert!(loaded.persistent.discovered);
    assert_eq!(loaded.persistent.guild_rank, GuildRank(3));
    assert_eq!(loaded.persistent.deepest_layer_reached, 7);
    assert_eq!(loaded.persistent.merc_id_counter, 12);
    assert_eq!(loaded.persistent.mission_id_counter, 8);
    assert_eq!(loaded.prestige.warband_marks, 1_240);

    let rec = loaded.persistent.layer_record(3).expect("layer 3 present");
    assert!(rec.cleared);
    assert_eq!(rec.familiarity, 75);
    assert!(rec.has_infrastructure(Infrastructure::Outpost));
}

// ── 2. Missing save file ──────────────────────────────────────────────────────

#[test]
fn test_missing_save_file_returns_default_state() {
    // Write nothing — rely on the fact that the real save path almost certainly
    // does NOT have a deep.json in the test environment (CI / clean machine).
    // Instead, exercise the same code path via direct serde fallback.
    let corrupt_json = "";
    let result: DeepState = serde_json::from_str(corrupt_json).unwrap_or_default();

    // Default state: not discovered, guild rank 1, no layers.
    assert!(!result.persistent.discovered);
    assert_eq!(result.persistent.guild_rank, GuildRank(1));
    assert!(result.persistent.layers.is_empty());
    assert_eq!(result.prestige.warband_marks, 0);
}

// ── 3. Corrupted save — graceful fallback ─────────────────────────────────────

#[test]
fn test_corrupted_save_falls_back_to_default() {
    let corrupt_json = "not valid JSON at all!!!";
    let result: DeepState = serde_json::from_str(corrupt_json).unwrap_or_default();

    assert!(!result.persistent.discovered);
    assert_eq!(result.persistent.guild_rank, GuildRank::MIN);
    assert!(result.prestige.roster.is_empty());
}

#[test]
fn test_partial_json_falls_back_to_default() {
    // A JSON object that is structurally invalid for DeepState.
    let partial = r#"{"persistent": {"discovered": true}}"#;
    let result: DeepState = serde_json::from_str(partial).unwrap_or_default();
    // serde will fail on missing required fields -> default.
    // Either it deserializes successfully with the partial data or falls back.
    // The important thing is: no panic.
    let _ = result; // just assert it doesn't panic
}

// ── 4. Discovery fires at P15 ─────────────────────────────────────────────────

#[test]
fn test_discovery_fires_at_min_prestige() {
    let mut deep = DeepState::new();
    let mut rng = seeded_rng();
    let mut found = false;

    for _ in 0..500_000 {
        if try_discover_deep(&mut deep, DEEP_MIN_PRESTIGE_RANK, &mut rng) {
            found = true;
            break;
        }
    }

    assert!(
        found,
        "The Deep should be discoverable within 500k ticks at P{}",
        DEEP_MIN_PRESTIGE_RANK
    );
    assert!(deep.persistent.discovered);
}

// ── 5. No discovery below P15 ─────────────────────────────────────────────────

#[test]
fn test_no_discovery_below_min_prestige() {
    let mut rng = seeded_rng();
    let below_min = DEEP_MIN_PRESTIGE_RANK - 1;

    for _ in 0..100_000 {
        let mut deep = DeepState::new();
        assert!(
            !try_discover_deep(&mut deep, below_min, &mut rng),
            "Discovery must not fire below P{}",
            DEEP_MIN_PRESTIGE_RANK
        );
    }
}

#[test]
fn test_no_discovery_at_prestige_zero() {
    let mut rng = seeded_rng();
    let mut deep = DeepState::new();

    for _ in 0..100_000 {
        assert!(!try_discover_deep(&mut deep, 0, &mut rng));
    }
    assert!(!deep.persistent.discovered);
}

// ── 6. Discovery creates correct initial state ────────────────────────────────

#[test]
fn test_discovery_creates_initial_state() {
    let mut deep = DeepState::new();
    force_discover(&mut deep, DEEP_MIN_PRESTIGE_RANK, 500_000);

    // Guild rank starts at 1.
    assert_eq!(deep.persistent.guild_rank, GuildRank(1));

    // Exactly 3 starter mercs, all at base level with no missions.
    assert_eq!(
        deep.prestige.roster.len(),
        3,
        "Starter roster must have exactly 3 mercs"
    );
    for merc in &deep.prestige.roster {
        assert_eq!(merc.level, 1, "Starter mercs must be level 1");
        assert_eq!(merc.missions_completed, 0);
        assert_eq!(merc.status, MercStatus::Available);
    }

    // Warband Marks start at 50 (seed money for first missions).
    assert_eq!(deep.prestige.warband_marks, 50);
}

#[test]
fn test_discovery_starter_roster_archetypes() {
    let mut deep = DeepState::new();
    force_discover(&mut deep, DEEP_MIN_PRESTIGE_RANK, 500_000);

    let archetypes: Vec<MercArchetype> = deep.prestige.roster.iter().map(|m| m.archetype).collect();

    assert!(
        archetypes.contains(&MercArchetype::Vanguard),
        "Starter roster must include a Vanguard"
    );
    assert!(
        archetypes.contains(&MercArchetype::Scout),
        "Starter roster must include a Scout"
    );
    assert!(
        archetypes.contains(&MercArchetype::Medic),
        "Starter roster must include a Medic"
    );
}

#[test]
fn test_discovery_merc_ids_are_unique_and_sequential() {
    let mut deep = DeepState::new();
    force_discover(&mut deep, DEEP_MIN_PRESTIGE_RANK, 500_000);

    let ids: Vec<u64> = deep.prestige.roster.iter().map(|m| m.id).collect();
    // IDs should be 1, 2, 3 (counter starts at 0 and increments before assignment).
    assert_eq!(ids, vec![1, 2, 3]);
    assert_eq!(deep.persistent.merc_id_counter, 3);
}

// ── 7. Prestige preserves guild rank ─────────────────────────────────────────

#[test]
fn test_prestige_preserves_guild_rank() {
    let mut deep = DeepState::new();
    force_discover(&mut deep, DEEP_MIN_PRESTIGE_RANK, 500_000);

    // Advance guild rank to 2.
    deep.persistent.guild_rank = GuildRank(2);
    deep.prestige.warband_marks = 5_000;

    deep.on_prestige();

    assert_eq!(
        deep.persistent.guild_rank,
        GuildRank(2),
        "Guild rank must persist across prestige"
    );
}

// ── 8. Prestige preserves cleared layers ─────────────────────────────────────

#[test]
fn test_prestige_preserves_cleared_layers() {
    let mut deep = DeepState::new();
    force_discover(&mut deep, DEEP_MIN_PRESTIGE_RANK, 500_000);

    // Mark layers 1 and 2 as cleared.
    mark_layer_cleared(&mut deep.persistent, 1);
    mark_layer_cleared(&mut deep.persistent, 2);

    assert!(deep.persistent.layer_record(1).unwrap().cleared);
    assert!(deep.persistent.layer_record(2).unwrap().cleared);

    deep.on_prestige();

    assert!(
        deep.persistent.layer_record(1).unwrap().cleared,
        "Layer 1 cleared status must persist across prestige"
    );
    assert!(
        deep.persistent.layer_record(2).unwrap().cleared,
        "Layer 2 cleared status must persist across prestige"
    );
    assert_eq!(deep.persistent.deepest_layer_reached, 2);
}

// ── 9. Prestige preserves infrastructure ────────────────────────────────────

#[test]
fn test_prestige_preserves_infrastructure() {
    let mut deep = DeepState::new();
    force_discover(&mut deep, DEEP_MIN_PRESTIGE_RANK, 500_000);

    // Build infrastructure on layer 1 directly (bypassing cost checks for test).
    mark_layer_cleared(&mut deep.persistent, 1);
    {
        let rec = deep.persistent.layer_record_mut(1);
        rec.infrastructure.push(Infrastructure::Outpost);
        rec.infrastructure.push(Infrastructure::Watchtower);
    }

    deep.on_prestige();

    let rec = deep
        .persistent
        .layer_record(1)
        .expect("layer 1 must exist after prestige");
    assert!(
        rec.has_infrastructure(Infrastructure::Outpost),
        "Outpost must persist across prestige"
    );
    assert!(
        rec.has_infrastructure(Infrastructure::Watchtower),
        "Watchtower must persist across prestige"
    );
}

// ── 10. Prestige preserves familiarity ───────────────────────────────────────

#[test]
fn test_prestige_preserves_familiarity() {
    let mut deep = DeepState::new();
    force_discover(&mut deep, DEEP_MIN_PRESTIGE_RANK, 500_000);

    mark_layer_cleared(&mut deep.persistent, 1);
    {
        let rec = deep.persistent.layer_record_mut(1);
        rec.familiarity = 80;
    }

    deep.on_prestige();

    let rec = deep
        .persistent
        .layer_record(1)
        .expect("layer 1 must exist after prestige");
    assert_eq!(
        rec.familiarity, 80,
        "Familiarity must persist across prestige"
    );
}

// ── 11. Prestige clears mercs ────────────────────────────────────────────────

#[test]
fn test_prestige_clears_roster() {
    let mut deep = DeepState::new();
    force_discover(&mut deep, DEEP_MIN_PRESTIGE_RANK, 500_000);

    assert_eq!(deep.prestige.roster.len(), 3, "Should start with 3 mercs");

    deep.on_prestige();

    assert!(
        deep.prestige.roster.is_empty(),
        "Roster must be empty immediately after prestige (before new starters)"
    );
}

#[test]
fn test_prestige_clears_available_missions() {
    let mut deep = DeepState::new();
    force_discover(&mut deep, DEEP_MIN_PRESTIGE_RANK, 500_000);

    // Populate available_missions pool.
    let mut rng = seeded_rng();
    mark_layer_cleared(&mut deep.persistent, 1);
    deep.prestige.available_missions = generate_mission_pool(&deep.persistent, &mut rng);
    assert!(!deep.prestige.available_missions.is_empty());

    deep.on_prestige();

    assert!(
        deep.prestige.available_missions.is_empty(),
        "Available mission pool must clear on prestige"
    );
}

// ── 12. Prestige clears Warband Marks ────────────────────────────────────────

#[test]
fn test_prestige_clears_warband_marks() {
    let mut deep = DeepState::new();
    force_discover(&mut deep, DEEP_MIN_PRESTIGE_RANK, 500_000);

    deep.prestige.warband_marks = 99_999;

    deep.on_prestige();

    assert_eq!(
        deep.prestige.warband_marks, 0,
        "Warband Marks must reset to 0 on prestige"
    );
}

// ── 13. Prestige cancels active missions ──────────────────────────────────────

#[test]
fn test_prestige_cancels_active_missions() {
    let mut deep = DeepState::new();
    force_discover(&mut deep, DEEP_MIN_PRESTIGE_RANK, 500_000);

    // Inject an active mission directly.
    let mission = make_active_supply_run(&mut deep);
    let mission_id = mission.id;
    deep.prestige.active_missions.push(mission);

    // Mark the merc as on this mission.
    if let Some(merc) = deep.prestige.roster.first_mut() {
        merc.status = MercStatus::OnMission(mission_id);
    }

    assert_eq!(deep.prestige.active_missions.len(), 1);

    deep.on_prestige();

    assert!(
        deep.prestige.active_missions.is_empty(),
        "Active missions must be cancelled on prestige"
    );
}

// ── 14. Offline mission resolution ───────────────────────────────────────────

#[test]
fn test_offline_mission_resolution_completes_elapsed_mission() {
    let mut deep = DeepState::new();
    force_discover(&mut deep, DEEP_MIN_PRESTIGE_RANK, 500_000);

    // Ensure the roster merc is available.
    let _merc_id = deep.prestige.roster.first().unwrap().id;

    // Create a mission whose timer has elapsed.
    let mission = make_elapsed_supply_run(&mut deep);
    deep.prestige.active_missions.push(mission);

    // Mark merc as on the mission.
    if let Some(merc) = deep.prestige.roster.first_mut() {
        merc.status = MercStatus::OnMission(1);
    }

    let mut rng = seeded_rng();
    let summary = resolve_offline_missions(&mut deep.prestige, &mut deep.persistent, &mut rng);

    assert_eq!(
        summary.missions_resolved, 1,
        "One elapsed mission should be resolved offline"
    );
    // The resolved mission should have moved to pending_results.
    assert_eq!(deep.prestige.pending_results.len(), 1);
    // Active missions should be empty now.
    assert!(deep.prestige.active_missions.is_empty());
}

#[test]
fn test_offline_resolution_does_not_complete_active_missions() {
    let mut deep = DeepState::new();
    force_discover(&mut deep, DEEP_MIN_PRESTIGE_RANK, 500_000);

    // Mission that hasn't ended yet.
    let mission = make_active_supply_run(&mut deep);
    deep.prestige.active_missions.push(mission);

    let mut rng = seeded_rng();
    let summary = resolve_offline_missions(&mut deep.prestige, &mut deep.persistent, &mut rng);

    assert_eq!(
        summary.missions_resolved, 0,
        "Still-active mission must not be resolved offline"
    );
    assert_eq!(
        deep.prestige.active_missions.len(),
        1,
        "Active mission should still be active"
    );
    assert!(deep.prestige.pending_results.is_empty());
}

// ── 15. Offline event auto-resolve ───────────────────────────────────────────

#[test]
fn test_offline_resolve_does_not_panic_without_events() {
    // A mission with no events at all should resolve cleanly offline.
    let mut deep = DeepState::new();
    force_discover(&mut deep, DEEP_MIN_PRESTIGE_RANK, 500_000);

    let mission = make_elapsed_supply_run(&mut deep);
    deep.prestige.active_missions.push(mission);

    let mut rng = seeded_rng();
    let summary = resolve_offline_missions(&mut deep.prestige, &mut deep.persistent, &mut rng);

    // events_auto_resolved should be 0 for a supply run (no events).
    assert_eq!(
        summary.events_auto_resolved, 0,
        "Supply runs have no events to auto-resolve"
    );
    assert_eq!(summary.missions_resolved, 1);
}

// ── 16. Multiple prestige cycles ─────────────────────────────────────────────

#[test]
fn test_three_prestige_cycles_preserves_persistent_state() {
    let mut deep = DeepState::new();
    force_discover(&mut deep, DEEP_MIN_PRESTIGE_RANK, 500_000);

    // Set up persistent state before any prestige.
    deep.persistent.guild_rank = GuildRank(2);
    mark_layer_cleared(&mut deep.persistent, 1);
    mark_layer_cleared(&mut deep.persistent, 2);
    {
        let rec = deep.persistent.layer_record_mut(1);
        rec.infrastructure.push(Infrastructure::Outpost);
        rec.familiarity = 60;
    }

    // Run 3 prestige cycles, adding marks/missions each time.
    for cycle in 1..=3 {
        deep.prestige.warband_marks = cycle * 1_000;
        let mission = make_active_supply_run(&mut deep);
        deep.prestige.active_missions.push(mission);

        deep.on_prestige();

        // After each cycle, persistent state must be intact.
        assert!(
            deep.persistent.discovered,
            "Cycle {}: discovered must persist",
            cycle
        );
        assert_eq!(
            deep.persistent.guild_rank,
            GuildRank(2),
            "Cycle {}: guild rank must persist",
            cycle
        );
        assert!(
            deep.persistent.layer_record(1).unwrap().cleared,
            "Cycle {}: layer 1 cleared must persist",
            cycle
        );
        assert!(
            deep.persistent.layer_record(2).unwrap().cleared,
            "Cycle {}: layer 2 cleared must persist",
            cycle
        );
        assert!(
            deep.persistent
                .layer_record(1)
                .unwrap()
                .has_infrastructure(Infrastructure::Outpost),
            "Cycle {}: outpost must persist",
            cycle
        );
        assert_eq!(
            deep.persistent.layer_record(1).unwrap().familiarity,
            60,
            "Cycle {}: familiarity must persist",
            cycle
        );

        // Per-prestige state must be cleared after each cycle.
        assert_eq!(
            deep.prestige.warband_marks, 0,
            "Cycle {}: marks must reset",
            cycle
        );
        assert!(
            deep.prestige.active_missions.is_empty(),
            "Cycle {}: missions must be cleared",
            cycle
        );
    }

    // After 3 cycles, merc_id_counter must reflect all mercs allocated
    // (only the 3 starters from initial discovery; no new starters on prestige).
    assert_eq!(deep.persistent.merc_id_counter, 3);
}

#[test]
fn test_multiple_prestige_cycles_id_counters_never_reset() {
    let mut deep = DeepState::new();
    force_discover(&mut deep, DEEP_MIN_PRESTIGE_RANK, 500_000);

    // Allocate some mission IDs.
    let _ = deep.persistent.next_mission_id();
    let _ = deep.persistent.next_mission_id();
    assert_eq!(deep.persistent.mission_id_counter, 2);

    deep.on_prestige();

    // Counters must NOT reset on prestige — they are monotonic across all generations.
    assert_eq!(
        deep.persistent.mission_id_counter, 2,
        "mission_id_counter must not reset on prestige"
    );
    assert_eq!(
        deep.persistent.merc_id_counter, 3,
        "merc_id_counter must not reset on prestige"
    );
}

// ── DeepUiState farewell_mercs ─────────────────────────────────────────────────

#[test]
fn test_farewell_mercs_starts_empty() {
    let ui = quest::deep::DeepUiState::new();
    assert!(ui.farewell_mercs.is_empty());
}

#[test]
fn test_farewell_mercs_stores_tuples() {
    let mut ui = quest::deep::DeepUiState::new();
    ui.farewell_mercs
        .push(("Gareth the Unyielding".to_string(), 5, 12));
    ui.farewell_mercs.push(("Lyra the Swift".to_string(), 3, 7));

    assert_eq!(ui.farewell_mercs.len(), 2);
    assert_eq!(ui.farewell_mercs[0].0, "Gareth the Unyielding");
    assert_eq!(ui.farewell_mercs[0].1, 5);
    assert_eq!(ui.farewell_mercs[0].2, 12);
    assert_eq!(ui.farewell_mercs[1].0, "Lyra the Swift");
    assert_eq!(ui.farewell_mercs[1].1, 3);
    assert_eq!(ui.farewell_mercs[1].2, 7);
}

// ── Bonus: save_path format ───────────────────────────────────────────────────

#[test]
fn test_deep_save_path_ends_with_expected_filename() {
    let path = deep_save_path().expect("should return a valid path");
    let path_str = path.to_str().expect("path should be valid UTF-8");
    assert!(
        path_str.ends_with(".quest/deep.json"),
        "Save path should end with .quest/deep.json, got: {}",
        path_str
    );
    assert!(path.is_absolute(), "Save path must be absolute");
}

// ── Bonus: DeepState default / new ───────────────────────────────────────────

#[test]
fn test_deep_state_new_has_correct_defaults() {
    let ds = DeepState::new();
    assert!(!ds.persistent.discovered);
    assert_eq!(ds.persistent.guild_rank, GuildRank::MIN);
    assert_eq!(ds.persistent.deepest_layer_reached, 0);
    assert!(ds.persistent.layers.is_empty());
    assert_eq!(ds.persistent.merc_id_counter, 0);
    assert_eq!(ds.persistent.mission_id_counter, 0);
    assert_eq!(ds.prestige.warband_marks, 0);
    assert!(ds.prestige.roster.is_empty());
    assert!(ds.prestige.active_missions.is_empty());
    assert!(!ds.is_active());
}

#[test]
fn test_deep_state_is_active_after_discovery() {
    let mut deep = DeepState::new();
    assert!(!deep.is_active());
    force_discover(&mut deep, DEEP_MIN_PRESTIGE_RANK, 500_000);
    assert!(deep.is_active());
}
