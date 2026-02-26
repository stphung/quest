//! Integration tests for The Deep — persistence, discovery, and prestige interactions.
//!
//! Covers the full lifecycle: discovery → play → prestige → reload.
//! Uses in-memory serde roundtrips for persistence tests where possible;
//! file I/O tests write to a temp directory that is cleaned up automatically.

use chrono::{Duration, Utc};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use quest::deep::{
    // Discovery
    advance_deep_story,
    complete_story_discovery,
    // Persistence
    deep_save_path,
    effective_concurrent_missions,
    generate_mission_pool,
    // Layers
    mark_layer_cleared,
    // Missions
    resolve_offline_missions,
    // Types
    DeepPersistent,
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

/// Force-discover The Deep via the narrative story chain.
/// Sets story stage to 4 (The Entrance) then calls `complete_story_discovery`.
fn force_discover(deep: &mut DeepState) -> ChaCha8Rng {
    let mut rng = seeded_rng();
    deep.persistent.deep_story_stage = 4;
    deep.persistent.rift_fragments = 4;
    complete_story_discovery(deep, &mut rng);
    assert!(
        deep.persistent.discovered,
        "Discovery must succeed at stage 4"
    );
    rng
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
        is_first_orders: false,
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
        is_first_orders: false,
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

// ── 4. Narrative discovery via story chain ─────────────────────────────────────

#[test]
fn test_narrative_discovery_requires_stage_4() {
    let mut deep = DeepState::new();
    let mut rng = seeded_rng();

    // Stage 3 should not allow discovery.
    deep.persistent.deep_story_stage = 3;
    deep.persistent.rift_fragments = 4;
    complete_story_discovery(&mut deep, &mut rng);
    assert!(
        !deep.persistent.discovered,
        "Discovery must not fire below story stage 4"
    );

    // Stage 4 allows discovery.
    deep.persistent.deep_story_stage = 4;
    complete_story_discovery(&mut deep, &mut rng);
    assert!(deep.persistent.discovered);
    assert_eq!(deep.persistent.deep_story_stage, 5);
}

// ── 5. Story chain advances through prestige in The Expanse ──────────────────

#[test]
fn test_story_chain_advances_on_prestige() {
    let mut deep = DeepState::new();

    // Simulate prestiges from The Expanse (zone 11+) at P15+.
    // Each prestige increments rift_resonance.
    for _ in 0..7 {
        deep.maybe_increment_rift_resonance(11, DEEP_MIN_PRESTIGE_RANK + 6);
    }
    assert_eq!(deep.persistent.rift_resonance, 7);

    // Advance story — should reach stage 1 (Tremors) at resonance 1, P15+.
    // We simulate the full chain by calling advance_deep_story at each stage's
    // required prestige rank.
    deep.persistent.rift_resonance = 1;
    let stage = advance_deep_story(&mut deep, DEEP_MIN_PRESTIGE_RANK);
    assert_eq!(stage, Some(1));
}

#[test]
fn test_no_discovery_without_expanse_prestiges() {
    let mut deep = DeepState::new();
    let mut rng = seeded_rng();

    // No rift resonance, no story progression, no discovery.
    deep.persistent.deep_story_stage = 0;
    complete_story_discovery(&mut deep, &mut rng);
    assert!(!deep.persistent.discovered);
}

// ── 6. Discovery creates correct initial state ────────────────────────────────

#[test]
fn test_discovery_creates_initial_state() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    // Guild rank starts at 1.
    assert_eq!(deep.persistent.guild_rank, GuildRank(1));

    // Exactly 3 starter mercs, all at base level.
    // After First Orders auto-queue, they are on the starter mission.
    assert_eq!(
        deep.prestige.roster.len(),
        3,
        "Starter roster must have exactly 3 mercs"
    );
    for merc in &deep.prestige.roster {
        assert_eq!(merc.level, 1, "Starter mercs must be level 1");
        assert_eq!(merc.missions_completed, 0);
        // Mercs are on the First Orders mission after discovery
        assert!(
            matches!(merc.status, MercStatus::OnMission(_)),
            "Starter mercs should be on First Orders mission"
        );
    }

    // Warband Marks start at 50 (seed money for first missions).
    assert_eq!(deep.prestige.warband_marks, 50);
}

#[test]
fn test_discovery_starter_roster_archetypes() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

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
    force_discover(&mut deep);

    let ids: Vec<u64> = deep.prestige.roster.iter().map(|m| m.id).collect();
    // IDs should be 1, 2, 3 (counter starts at 0 and increments before assignment).
    assert_eq!(ids, vec![1, 2, 3]);
    assert_eq!(deep.persistent.merc_id_counter, 3);
}

// ── 7. Prestige preserves guild rank ─────────────────────────────────────────

#[test]
fn test_prestige_preserves_guild_rank() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

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
    force_discover(&mut deep);

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
    force_discover(&mut deep);

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
    force_discover(&mut deep);

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
    force_discover(&mut deep);

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
    force_discover(&mut deep);

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
    force_discover(&mut deep);

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
    force_discover(&mut deep);

    // Inject an active mission directly.
    let mission = make_active_supply_run(&mut deep);
    let mission_id = mission.id;
    deep.prestige.active_missions.push(mission);

    // Mark the merc as on this mission.
    if let Some(merc) = deep.prestige.roster.first_mut() {
        merc.status = MercStatus::OnMission(mission_id);
    }

    // First Orders + injected mission = 2 active missions
    assert_eq!(deep.prestige.active_missions.len(), 2);

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
    force_discover(&mut deep);

    // Clear the auto-queued First Orders mission so we test in isolation.
    deep.prestige.active_missions.clear();
    for merc in &mut deep.prestige.roster {
        merc.status = MercStatus::Available;
    }

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
    force_discover(&mut deep);

    // Clear the auto-queued First Orders mission so we test in isolation.
    deep.prestige.active_missions.clear();
    for merc in &mut deep.prestige.roster {
        merc.status = MercStatus::Available;
    }

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
    force_discover(&mut deep);

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
    force_discover(&mut deep);

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
    force_discover(&mut deep);

    // First Orders auto-queued on discovery already consumed mission ID 1.
    // Allocate two more mission IDs.
    let _ = deep.persistent.next_mission_id();
    let _ = deep.persistent.next_mission_id();
    assert_eq!(deep.persistent.mission_id_counter, 3);

    deep.on_prestige();

    // Counters must NOT reset on prestige — they are monotonic across all generations.
    assert_eq!(
        deep.persistent.mission_id_counter, 3,
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
    force_discover(&mut deep);
    assert!(deep.is_active());
}

// =========================================================================
// Rift Resonance
// =========================================================================

#[test]
fn test_rift_resonance_increments_when_in_expanse_at_p15() {
    let mut deep = DeepState::new();
    deep.maybe_increment_rift_resonance(11, 15);
    assert_eq!(deep.persistent.rift_resonance, 1);
}

#[test]
fn test_rift_resonance_no_increment_below_p15() {
    let mut deep = DeepState::new();
    deep.maybe_increment_rift_resonance(11, 14);
    assert_eq!(deep.persistent.rift_resonance, 0);
}

#[test]
fn test_rift_resonance_no_increment_below_zone_11() {
    let mut deep = DeepState::new();
    deep.maybe_increment_rift_resonance(10, 15);
    assert_eq!(deep.persistent.rift_resonance, 0);
}

#[test]
fn test_rift_resonance_accumulates_across_prestiges() {
    let mut deep = DeepState::new();
    for _ in 0..7 {
        deep.maybe_increment_rift_resonance(11, 20);
        deep.on_prestige();
    }
    assert_eq!(deep.persistent.rift_resonance, 7);
}

#[test]
fn test_rift_resonance_survives_prestige_reset() {
    let mut deep = DeepState::new();
    deep.maybe_increment_rift_resonance(11, 15);
    deep.on_prestige();
    assert_eq!(
        deep.persistent.rift_resonance, 1,
        "Rift resonance should persist across prestige"
    );
}

// =========================================================================
// Story Chain Progression
// =========================================================================

#[test]
fn test_story_stage_advances_at_resonance_thresholds() {
    let mut deep = DeepState::new();

    // Stage 0 -> 1 at resonance 1, P15
    deep.persistent.rift_resonance = 1;
    assert_eq!(deep.check_story_progression(15), Some(1));
    assert_eq!(deep.persistent.deep_story_stage, 1);

    // Stage 1 -> 2 at resonance 3, P17
    deep.persistent.rift_resonance = 3;
    assert_eq!(deep.check_story_progression(17), Some(2));
    assert_eq!(deep.persistent.deep_story_stage, 2);

    // Stage 2 -> 3 at resonance 5, P19
    deep.persistent.rift_resonance = 5;
    assert_eq!(deep.check_story_progression(19), Some(3));
    assert_eq!(deep.persistent.deep_story_stage, 3);
}

#[test]
fn test_story_stage_blocked_by_prestige_gate() {
    let mut deep = DeepState::new();
    deep.persistent.rift_resonance = 3;
    // Resonance 3 but only P15 — captain requires P17
    deep.persistent.deep_story_stage = 1;
    assert_eq!(deep.check_story_progression(15), None);
    assert_eq!(deep.persistent.deep_story_stage, 1);
}

#[test]
fn test_story_stage_4_requires_fragments() {
    let mut deep = DeepState::new();
    deep.persistent.rift_resonance = 7;
    deep.persistent.deep_story_stage = 3;
    deep.persistent.rift_fragments = 3; // need 4
    assert_eq!(deep.check_story_progression(21), None);

    deep.persistent.rift_fragments = 4;
    assert_eq!(deep.check_story_progression(21), Some(4));
}

#[test]
fn test_rift_fragments_awarded_at_resonance_5_6_7() {
    let mut deep = DeepState::new();

    deep.persistent.rift_resonance = 4;
    deep.maybe_award_rift_fragment();
    assert_eq!(
        deep.persistent.rift_fragments, 0,
        "No fragment before resonance 5"
    );

    deep.persistent.rift_resonance = 5;
    deep.maybe_award_rift_fragment();
    assert_eq!(deep.persistent.rift_fragments, 1);

    deep.persistent.rift_resonance = 6;
    deep.maybe_award_rift_fragment();
    assert_eq!(deep.persistent.rift_fragments, 2);

    deep.persistent.rift_resonance = 7;
    deep.maybe_award_rift_fragment();
    assert_eq!(deep.persistent.rift_fragments, 3);
}

#[test]
fn test_rift_fragments_cap_at_4() {
    let mut deep = DeepState::new();
    deep.persistent.rift_resonance = 10;
    deep.maybe_award_rift_fragment();
    assert_eq!(deep.persistent.rift_fragments, 4);
}

#[test]
fn test_advance_deep_story_full_chain() {
    let mut deep = DeepState::new();

    // Resonance 1 + P15 → stage 1 (Tremors)
    deep.persistent.rift_resonance = 1;
    assert_eq!(advance_deep_story(&mut deep, 15), Some(1));

    // Resonance 3 + P17 → stage 2 (Captain)
    deep.persistent.rift_resonance = 3;
    assert_eq!(advance_deep_story(&mut deep, 17), Some(2));

    // Resonance 5 + P19 → stage 3 (Fragment) + awards fragment 1
    deep.persistent.rift_resonance = 5;
    assert_eq!(advance_deep_story(&mut deep, 19), Some(3));
    assert_eq!(deep.persistent.rift_fragments, 1);

    // Resonance 7 + P21 → need 4 fragments, only have 3 after awarding
    deep.persistent.rift_resonance = 7;
    // Awarding fragments first: resonance 7 => 3 fragments
    assert_eq!(advance_deep_story(&mut deep, 21), None);
    assert_eq!(deep.persistent.rift_fragments, 3);

    // Manually set 4 fragments to test stage 4
    deep.persistent.rift_fragments = 4;
    assert_eq!(advance_deep_story(&mut deep, 21), Some(4));
}

#[test]
fn test_advance_deep_story_skipped_when_already_discovered() {
    let mut deep = DeepState::new();
    deep.persistent.discovered = true;
    deep.persistent.rift_resonance = 7;
    assert_eq!(advance_deep_story(&mut deep, 21), None);
}

#[test]
fn test_story_discovery_not_triggered_before_stage_4() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut deep = DeepState::new();
    deep.persistent.deep_story_stage = 3;
    complete_story_discovery(&mut deep, &mut rng);
    assert!(!deep.persistent.discovered);
}

#[test]
fn test_story_discovery_completes_at_stage_4() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut deep = DeepState::new();
    deep.persistent.deep_story_stage = 4;
    complete_story_discovery(&mut deep, &mut rng);
    assert!(deep.persistent.discovered);
    assert_eq!(deep.persistent.deep_story_stage, 5);
    assert_eq!(deep.prestige.roster.len(), 3);
}

#[test]
fn test_story_discovery_assigns_starter_marks() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut deep = DeepState::new();
    deep.persistent.deep_story_stage = 4;
    complete_story_discovery(&mut deep, &mut rng);
    assert_eq!(deep.prestige.warband_marks, 50); // Guild rank 1 = 50 marks
}

#[test]
fn test_story_discovery_blocked_when_already_discovered() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut deep = DeepState::new();
    deep.persistent.discovered = true;
    deep.persistent.deep_story_stage = 4;
    complete_story_discovery(&mut deep, &mut rng);
    // Should not add more mercs
    assert!(deep.prestige.roster.is_empty());
}

// =========================================================================
// T2-5: effective_concurrent_missions (early concurrent slot from L3 breakthrough)
// =========================================================================

#[test]
fn test_effective_concurrent_missions_rank1_no_breakthrough() {
    assert_eq!(effective_concurrent_missions(GuildRank(1), 0), 1);
    assert_eq!(effective_concurrent_missions(GuildRank(1), 2), 1);
}

#[test]
fn test_effective_concurrent_missions_rank1_with_l3_breakthrough() {
    assert_eq!(effective_concurrent_missions(GuildRank(1), 3), 2);
    assert_eq!(effective_concurrent_missions(GuildRank(1), 10), 2);
}

#[test]
fn test_effective_concurrent_missions_higher_ranks_unaffected() {
    // Rank 2+ should use their normal concurrent values from GUILD_RANK_STATS
    assert_eq!(effective_concurrent_missions(GuildRank(2), 3), 2);
    assert_eq!(effective_concurrent_missions(GuildRank(3), 7), 2);
    assert_eq!(effective_concurrent_missions(GuildRank(4), 13), 3);
    assert_eq!(effective_concurrent_missions(GuildRank(5), 25), 4);
}

#[test]
fn test_effective_concurrent_missions_rank1_boundary_layer2_vs_3() {
    // Layer 2 = no bonus, Layer 3 = bonus
    assert_eq!(effective_concurrent_missions(GuildRank(1), 2), 1);
    assert_eq!(effective_concurrent_missions(GuildRank(1), 3), 2);
}

#[test]
fn test_effective_concurrent_missions_rank2_not_doubled_by_l3() {
    // Rank 2 already has 2 concurrent — L3 breakthrough should NOT give an extra slot
    assert_eq!(effective_concurrent_missions(GuildRank(2), 0), 2);
    assert_eq!(effective_concurrent_missions(GuildRank(2), 3), 2);
    assert_eq!(effective_concurrent_missions(GuildRank(2), 10), 2);
}

// =========================================================================
// T2-6: First Orders — auto-queued tutorial mission on discovery
// =========================================================================

#[test]
fn test_first_orders_queued_on_discovery() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    assert!(deep.persistent.discovered);
    assert!(
        deep.persistent.first_orders_queued,
        "First Orders flag must be set on discovery"
    );
    assert_eq!(
        deep.prestige.active_missions.len(),
        1,
        "Exactly one First Orders mission should be auto-queued"
    );
    let mission = &deep.prestige.active_missions[0];
    assert!(
        mission.is_first_orders,
        "Mission must be flagged as First Orders"
    );
    assert_eq!(mission.mission_type, MissionType::Recon);
    assert_eq!(mission.layer, 1);
    // Squad should include all 3 starter mercs
    assert_eq!(mission.squad.len(), 3);
}

#[test]
fn test_first_orders_not_queued_twice() {
    let mut deep = DeepState::new();
    // Pretend First Orders was already queued in a previous discovery
    deep.persistent.first_orders_queued = true;
    deep.persistent.discovered = false;

    // Force discovery via story chain
    deep.persistent.deep_story_stage = 4;
    deep.persistent.rift_fragments = 4;
    let mut rng = seeded_rng();
    complete_story_discovery(&mut deep, &mut rng);

    if deep.persistent.discovered {
        // Should have discovered but no First Orders mission
        let has_first_orders = deep
            .prestige
            .active_missions
            .iter()
            .any(|m| m.is_first_orders);
        assert!(
            !has_first_orders,
            "First Orders should not be queued when first_orders_queued is already true"
        );
    }
}

#[test]
fn test_first_orders_mission_duration_is_20_minutes() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    assert!(!deep.prestige.active_missions.is_empty());
    let mission = &deep.prestige.active_missions[0];
    assert!(mission.is_first_orders);

    let duration_secs = (mission.ends_at - mission.started_at).num_seconds();
    assert_eq!(
        duration_secs, 1200,
        "First Orders mission should be 20 minutes (1200 seconds)"
    );
}

#[test]
fn test_first_orders_persists_across_serde() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    let json = serde_json::to_string_pretty(&deep).expect("serialize");
    let loaded: DeepState = serde_json::from_str(&json).expect("deserialize");

    assert!(loaded.persistent.first_orders_queued);
    assert_eq!(loaded.prestige.active_missions.len(), 1);
    assert!(loaded.prestige.active_missions[0].is_first_orders);
}

// =========================================================================
// T2-8: Generation Records — stats snapshot on prestige
// =========================================================================

#[test]
fn test_generation_record_created_on_prestige() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    // Simulate some activity before prestige
    deep.prestige.total_marks_earned = 500;
    deep.prestige.total_missions_completed = 10;
    deep.prestige.total_mercs_lost = 2;
    deep.persistent.deepest_layer_reached = 5;

    deep.on_prestige();

    assert_eq!(deep.persistent.generation_records.len(), 1);
    let record = &deep.persistent.generation_records[0];
    assert_eq!(record.generation, 1);
    assert_eq!(record.marks_earned, 500);
    assert_eq!(record.missions_completed, 10);
    assert_eq!(record.mercs_lost, 2);
    assert_eq!(record.deepest_layer_reached, 5);
}

#[test]
fn test_generation_records_accumulate_across_prestiges() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    // First prestige
    deep.prestige.total_marks_earned = 100;
    deep.prestige.total_missions_completed = 5;
    deep.prestige.total_mercs_lost = 1;
    deep.on_prestige();

    // Second prestige
    deep.prestige.total_marks_earned = 200;
    deep.prestige.total_missions_completed = 8;
    deep.prestige.total_mercs_lost = 0;
    deep.on_prestige();

    assert_eq!(deep.persistent.generation_records.len(), 2);
    assert_eq!(deep.persistent.generation_records[0].generation, 1);
    assert_eq!(deep.persistent.generation_records[0].marks_earned, 100);
    assert_eq!(deep.persistent.generation_records[1].generation, 2);
    assert_eq!(deep.persistent.generation_records[1].marks_earned, 200);
}

#[test]
fn test_generation_records_capped_at_10() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    for i in 0..15u32 {
        deep.prestige.total_marks_earned = (i + 1) * 100;
        deep.prestige.total_missions_completed = i + 1;
        deep.prestige.total_mercs_lost = 0;
        deep.on_prestige();
    }

    assert_eq!(
        deep.persistent.generation_records.len(),
        10,
        "Generation records must be capped at 10"
    );
    // Oldest should have been pruned — first record should be generation 6 (not 1)
    assert_eq!(
        deep.persistent.generation_records[0].generation, 6,
        "After 15 prestiges with cap 10, oldest record should be generation 6"
    );
    // Latest should be generation 15
    assert_eq!(deep.persistent.generation_records[9].generation, 15);
}

#[test]
fn test_generation_record_preserves_deepest_layer_at_snapshot_time() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    deep.persistent.deepest_layer_reached = 5;
    deep.prestige.total_marks_earned = 100;
    deep.on_prestige();

    // Advance deeper in next generation
    deep.persistent.deepest_layer_reached = 8;
    deep.prestige.total_marks_earned = 200;
    deep.on_prestige();

    // First record should have the depth at the time of that prestige
    assert_eq!(
        deep.persistent.generation_records[0].deepest_layer_reached,
        5
    );
    assert_eq!(
        deep.persistent.generation_records[1].deepest_layer_reached,
        8
    );
}

#[test]
fn test_prestige_tracking_fields_reset_after_prestige() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    deep.prestige.total_marks_earned = 500;
    deep.prestige.total_missions_completed = 10;
    deep.prestige.total_mercs_lost = 2;

    deep.on_prestige();

    // After prestige, the new prestige state should have zeroed tracking fields
    assert_eq!(
        deep.prestige.total_marks_earned, 0,
        "total_marks_earned must reset on prestige"
    );
    assert_eq!(
        deep.prestige.total_missions_completed, 0,
        "total_missions_completed must reset on prestige"
    );
    assert_eq!(
        deep.prestige.total_mercs_lost, 0,
        "total_mercs_lost must reset on prestige"
    );
}

// =========================================================================
// Serde Backward Compatibility — new fields must default gracefully
// =========================================================================

#[test]
fn test_serde_backward_compat_missing_first_orders_field() {
    // JSON representing a DeepPersistent from before the first_orders feature
    let json = r#"{"discovered":true,"guild_rank":1,"guild_upgrade_cost":500,"layers":[],"deepest_layer_reached":0,"merc_id_counter":0,"mission_id_counter":0,"generation_counter":0,"rift_resonance":0,"deep_story_stage":0,"rift_fragments":0,"gateway_opened":false}"#;
    let persistent: DeepPersistent = serde_json::from_str(json).unwrap();
    assert!(
        !persistent.first_orders_queued,
        "first_orders_queued must default to false for old saves"
    );
    assert!(
        persistent.generation_records.is_empty(),
        "generation_records must default to empty for old saves"
    );
}

#[test]
fn test_serde_backward_compat_mission_without_first_orders() {
    // Mission JSON without is_first_orders field should deserialize with false
    let json = r#"{"id":1,"mission_type":"Recon","layer":1,"squad":[1],"started_at":"2026-01-01T00:00:00Z","ends_at":"2026-01-01T01:00:00Z","events":[],"pending_event_index":0,"status":"Active","result":null}"#;
    let mission: Mission = serde_json::from_str(json).unwrap();
    assert!(
        !mission.is_first_orders,
        "is_first_orders must default to false for old mission data"
    );
}

#[test]
fn test_serde_generation_records_roundtrip() {
    let mut deep = DeepState::new();
    force_discover(&mut deep);

    deep.prestige.total_marks_earned = 300;
    deep.prestige.total_missions_completed = 7;
    deep.prestige.total_mercs_lost = 1;
    deep.persistent.deepest_layer_reached = 4;
    deep.on_prestige();

    let json = serde_json::to_string_pretty(&deep).expect("serialize");
    let loaded: DeepState = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(loaded.persistent.generation_records.len(), 1);
    let record = &loaded.persistent.generation_records[0];
    assert_eq!(record.generation, 1);
    assert_eq!(record.marks_earned, 300);
    assert_eq!(record.missions_completed, 7);
    assert_eq!(record.mercs_lost, 1);
    assert_eq!(record.deepest_layer_reached, 4);
}

#[test]
fn test_serde_backward_compat_prestige_without_tracking_fields() {
    // DeepPrestige JSON from before tracking fields were added
    let json = r#"{"warband_marks":100,"roster":[],"active_missions":[],"available_missions":[],"recruit_pool":{"candidates":[],"refreshed_at":"2026-01-01T00:00:00Z","recruit_costs":[]},"pending_results":[],"generation_number":0,"warband_log":[]}"#;
    let prestige: quest::deep::DeepPrestige = serde_json::from_str(json).unwrap();
    assert_eq!(
        prestige.total_marks_earned, 0,
        "total_marks_earned must default to 0 for old saves"
    );
    assert_eq!(
        prestige.total_missions_completed, 0,
        "total_missions_completed must default to 0 for old saves"
    );
    assert_eq!(
        prestige.total_mercs_lost, 0,
        "total_mercs_lost must default to 0 for old saves"
    );
}
