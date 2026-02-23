//! End-to-end integration tests: The Deep prestige reset and persistence model.
//!
//! Tests the full generational lifecycle across multiple prestige cycles, verifying
//! the "ratchet" — that each generation starts stronger than the last due to
//! accumulated persistent state (guild rank, cleared layers, infrastructure,
//! familiarity) while transient state (mercs, marks, missions) resets cleanly.
//!
//! Covers:
//!  1. Generation 1 (P15→P16): Discovery, roster init, layer clearing, infra build,
//!     guild rank upgrade, then prestige.
//!  2. Post-prestige verification: persistent state preserved, transient state reset.
//!  3. Generation 2 (P16→P17): Faster ramp from compounding infrastructure, deeper push.
//!  4. Generation 3 verification: Cumulative benefits across all prior generations.
//!  5. Edge cases: mid-mission prestige, injured mercs, full/empty roster.

use quest::deep::{
    DeepState, GuildRank, Infrastructure, LayerTier, MercArchetype, MercStatus, MissionOutcome,
    MissionType,
};
use quest::deep::{
    apply_duration_modifiers, apply_familiarity_gain, base_mission_duration_secs,
    build_infrastructure, generate_starter_roster, is_frontier_layer, is_safe_layer,
    mark_layer_cleared, try_upgrade_guild_rank, DurationModifiers, FamiliarityLevel,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

// ── Helpers ─────────────────────────────────────────────────────────────────────

fn seeded_rng() -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(42)
}

/// Build up a DeepState simulating Generation 1 at P15:
/// - Discover The Deep (manually set discovered = true)
/// - Generate starter roster (3 mercs: Vanguard, Scout, Medic)
/// - Clear layers 1-5
/// - Build infrastructure on L1-3 (Outpost on L1, SupplyCache on L2, Watchtower on L3)
/// - Accumulate familiarity on L1-5 via missions
/// - Upgrade guild rank to 2 (requires L3 cleared + 200 marks)
fn build_generation_1_state() -> DeepState {
    let mut rng = seeded_rng();
    let mut deep = DeepState::new();

    // Discovery
    deep.persistent.discovered = true;

    // Generate starter roster
    let starters = generate_starter_roster(
        deep.persistent.guild_rank,
        || deep.persistent.next_merc_id(),
        &mut rng,
    );
    deep.prestige.roster.extend(starters);

    // Clear layers 1-5
    for layer in 1..=5 {
        mark_layer_cleared(&mut deep.persistent, layer);
    }

    // Build infrastructure on L1-3
    build_infrastructure(deep.persistent.layer_record_mut(1), Infrastructure::Outpost).unwrap();
    build_infrastructure(deep.persistent.layer_record_mut(2), Infrastructure::SupplyCache).unwrap();
    build_infrastructure(deep.persistent.layer_record_mut(3), Infrastructure::Watchtower).unwrap();

    // Accumulate familiarity via missions on L1-5
    // Simulate several missions on each layer
    for layer in 1..=5 {
        let record = deep.persistent.layer_record_mut(layer);
        // Supply run (+5) x3 + Recon (+15) + Expedition (+10) = 40 familiarity
        apply_familiarity_gain(record, MissionType::SupplyRun);
        apply_familiarity_gain(record, MissionType::SupplyRun);
        apply_familiarity_gain(record, MissionType::SupplyRun);
        apply_familiarity_gain(record, MissionType::Recon);
        apply_familiarity_gain(record, MissionType::Expedition);
    }

    // Give enough marks and upgrade guild rank to 2
    deep.prestige.warband_marks = 500;
    let result = try_upgrade_guild_rank(&mut deep.persistent, &mut deep.prestige);
    assert_eq!(result, Ok(GuildRank(2)));

    deep
}

// ── Generation 1 Setup Tests ────────────────────────────────────────────────────

#[test]
fn test_gen1_initial_state_is_correct() {
    let deep = build_generation_1_state();

    // Persistent state
    assert!(deep.persistent.discovered);
    assert_eq!(deep.persistent.guild_rank, GuildRank(2));
    assert_eq!(deep.persistent.deepest_layer_reached, 5);
    assert_eq!(deep.persistent.merc_id_counter, 3); // 3 starter mercs

    // Layers 1-5 cleared
    for layer in 1..=5 {
        assert!(
            is_safe_layer(&deep.persistent, layer),
            "Layer {} should be cleared",
            layer,
        );
    }

    // Infrastructure
    assert!(deep.persistent.layer_record(1).unwrap().has_infrastructure(Infrastructure::Outpost));
    assert!(deep.persistent.layer_record(2).unwrap().has_infrastructure(Infrastructure::SupplyCache));
    assert!(deep.persistent.layer_record(3).unwrap().has_infrastructure(Infrastructure::Watchtower));

    // Familiarity on all 5 layers
    // L1-2, L4-5: 40 from missions only
    // L3: 40 from missions + 25 from Watchtower build = 65
    for layer in 1..=5 {
        let record = deep.persistent.layer_record(layer).unwrap();
        let expected = if layer == 3 { 65 } else { 40 };
        assert_eq!(
            record.familiarity, expected,
            "Layer {} familiarity should be {}",
            layer, expected,
        );
    }

    // Prestige state
    assert_eq!(deep.prestige.roster.len(), 3);
    // Marks: started with 500, spent 200 for guild upgrade = 300
    assert_eq!(deep.prestige.warband_marks, 300);
}

// ── Prestige Reset (Gen 1 → Gen 2) ─────────────────────────────────────────────

#[test]
fn test_prestige_preserves_discovered_flag() {
    let mut deep = build_generation_1_state();
    deep.on_prestige();
    assert!(deep.persistent.discovered);
}

#[test]
fn test_prestige_preserves_guild_rank() {
    let mut deep = build_generation_1_state();
    deep.on_prestige();
    assert_eq!(deep.persistent.guild_rank, GuildRank(2));
    assert_eq!(deep.persistent.guild_rank.display_name(), "Sellswords");
}

#[test]
fn test_prestige_preserves_deepest_layer_reached() {
    let mut deep = build_generation_1_state();
    deep.on_prestige();
    assert_eq!(deep.persistent.deepest_layer_reached, 5);
}

#[test]
fn test_prestige_preserves_cleared_layers() {
    let mut deep = build_generation_1_state();
    deep.on_prestige();

    for layer in 1..=5 {
        assert!(
            is_safe_layer(&deep.persistent, layer),
            "Layer {} should remain cleared after prestige",
            layer,
        );
    }
    // Layer 6 should still be frontier
    assert!(is_frontier_layer(&deep.persistent, 6));
}

#[test]
fn test_prestige_preserves_infrastructure() {
    let mut deep = build_generation_1_state();
    deep.on_prestige();

    assert!(deep.persistent.layer_record(1).unwrap().has_infrastructure(Infrastructure::Outpost));
    assert!(deep.persistent.layer_record(2).unwrap().has_infrastructure(Infrastructure::SupplyCache));
    assert!(deep.persistent.layer_record(3).unwrap().has_infrastructure(Infrastructure::Watchtower));
}

#[test]
fn test_prestige_preserves_familiarity() {
    let mut deep = build_generation_1_state();
    deep.on_prestige();

    for layer in 1..=5 {
        let record = deep.persistent.layer_record(layer).unwrap();
        // L3 has Watchtower bonus (+25) in addition to mission familiarity (40)
        let expected = if layer == 3 { 65 } else { 40 };
        assert_eq!(
            record.familiarity, expected,
            "Layer {} familiarity should persist through prestige",
            layer,
        );
    }
}

#[test]
fn test_prestige_preserves_merc_id_counter() {
    let mut deep = build_generation_1_state();
    let counter_before = deep.persistent.merc_id_counter;
    deep.on_prestige();
    assert_eq!(
        deep.persistent.merc_id_counter, counter_before,
        "merc_id_counter should persist across prestige"
    );
}

#[test]
fn test_prestige_preserves_mission_id_counter() {
    let mut deep = build_generation_1_state();
    deep.persistent.mission_id_counter = 7;
    deep.on_prestige();
    assert_eq!(deep.persistent.mission_id_counter, 7);
}

#[test]
fn test_prestige_resets_roster() {
    let mut deep = build_generation_1_state();
    assert!(!deep.prestige.roster.is_empty());
    deep.on_prestige();
    assert!(
        deep.prestige.roster.is_empty(),
        "Roster must be empty after prestige"
    );
}

#[test]
fn test_prestige_resets_warband_marks() {
    let mut deep = build_generation_1_state();
    assert!(deep.prestige.warband_marks > 0);
    deep.on_prestige();
    assert_eq!(deep.prestige.warband_marks, 0);
}

#[test]
fn test_prestige_resets_active_missions() {
    let mut deep = build_generation_1_state();
    // Simulate having active missions
    deep.prestige.active_missions.push(quest::deep::Mission {
        id: 1,
        mission_type: MissionType::SupplyRun,
        layer: 1,
        squad: vec![1],
        started_at: chrono::Utc::now(),
        ends_at: chrono::Utc::now() + chrono::Duration::hours(2),
        events: vec![],
        pending_event_index: 0,
        status: quest::deep::MissionStatus::Active,
        result: None,
    });
    deep.on_prestige();
    assert!(deep.prestige.active_missions.is_empty());
}

#[test]
fn test_prestige_resets_available_missions() {
    let mut deep = build_generation_1_state();
    deep.prestige.available_missions.push(quest::deep::AvailableMission {
        mission_type: MissionType::SupplyRun,
        layer: 1,
        duration_secs: 7200,
        min_squad_power: 10,
        required_archetype: None,
        recommended_archetype: None,
        marks_cost: 20,
        description: "Test".to_string(),
    });
    deep.on_prestige();
    assert!(deep.prestige.available_missions.is_empty());
}

#[test]
fn test_prestige_resets_recruit_pool() {
    let mut deep = build_generation_1_state();
    deep.on_prestige();
    assert!(deep.prestige.recruit_pool.candidates.is_empty());
}

// ── Generation 2: New Mercs Inherit Infrastructure ──────────────────────────────

#[test]
fn test_gen2_new_starters_use_rank2_id_counter() {
    let mut deep = build_generation_1_state();
    let ids_before = deep.persistent.merc_id_counter;
    deep.on_prestige();

    // Generate new starter roster for gen 2
    let mut rng = seeded_rng();
    let starters = generate_starter_roster(
        deep.persistent.guild_rank,
        || deep.persistent.next_merc_id(),
        &mut rng,
    );

    // IDs should continue from where gen 1 left off
    assert_eq!(starters.len(), 3);
    for (i, merc) in starters.iter().enumerate() {
        assert_eq!(
            merc.id,
            ids_before + 1 + i as u64,
            "Gen 2 merc {} should have id {}",
            i,
            ids_before + 1 + i as u64,
        );
    }
}

#[test]
fn test_gen2_roster_size_matches_rank2() {
    let mut deep = build_generation_1_state();
    deep.on_prestige();

    // Rank 2 allows 7 mercs
    assert_eq!(deep.persistent.guild_rank.max_roster(), 7);
}

#[test]
fn test_gen2_supply_runs_benefit_from_gen1_infrastructure() {
    let mut deep = build_generation_1_state();
    deep.on_prestige();

    // L1 has an Outpost — supply runs should be faster
    let base_duration = base_mission_duration_secs(LayerTier::Shallows, MissionType::SupplyRun);
    let with_outpost = apply_duration_modifiers(base_duration, &DurationModifiers {
        has_outpost: true,
        familiarity: 40, // from gen 1
        has_saboteur: false,
        saboteur_is_veteran: false,
        is_overpowered: false,
    });
    let without_infra = apply_duration_modifiers(base_duration, &DurationModifiers {
        has_outpost: false,
        familiarity: 0,
        has_saboteur: false,
        saboteur_is_veteran: false,
        is_overpowered: false,
    });

    assert!(
        with_outpost < without_infra,
        "Gen 2 supply runs on L1 should be faster due to gen 1 infrastructure: {} vs {}",
        with_outpost,
        without_infra,
    );
}

#[test]
fn test_gen2_familiarity_continues_accumulating() {
    let mut deep = build_generation_1_state();
    deep.on_prestige();

    // L1 had 40 familiarity from gen 1 — add more in gen 2
    let record = deep.persistent.layer_record_mut(1);
    let fam_before = record.familiarity;
    apply_familiarity_gain(record, MissionType::Recon); // +15
    assert_eq!(record.familiarity, fam_before + 15);
    assert_eq!(record.familiarity, 55);
    assert_eq!(
        FamiliarityLevel::from_familiarity(record.familiarity),
        FamiliarityLevel::Familiar,
    );
}

#[test]
fn test_gen2_can_push_beyond_gen1_frontier() {
    let mut deep = build_generation_1_state();
    deep.on_prestige();

    // Gen 1 cleared L1-5. Gen 2 should be able to clear L6+
    assert!(is_frontier_layer(&deep.persistent, 6));
    mark_layer_cleared(&mut deep.persistent, 6);
    assert_eq!(deep.persistent.deepest_layer_reached, 6);
    assert!(is_safe_layer(&deep.persistent, 6));
    assert!(is_frontier_layer(&deep.persistent, 7));
}

#[test]
fn test_gen2_build_more_infrastructure_on_new_layers() {
    let mut deep = build_generation_1_state();
    deep.on_prestige();

    // Clear L6 and build infrastructure
    mark_layer_cleared(&mut deep.persistent, 6);
    let result = build_infrastructure(
        deep.persistent.layer_record_mut(6),
        Infrastructure::Outpost,
    );
    assert!(result.is_ok());
    assert!(deep.persistent.layer_record(6).unwrap().has_infrastructure(Infrastructure::Outpost));
}

// ── Full 3-Generation Lifecycle ─────────────────────────────────────────────────

#[test]
fn test_three_generation_lifecycle_ratchet() {
    let mut rng = seeded_rng();
    let mut deep = DeepState::new();
    deep.persistent.discovered = true;

    // ═══ Generation 1 (P15→P16) ═══
    // Starter roster
    let gen1_starters = generate_starter_roster(
        deep.persistent.guild_rank,
        || deep.persistent.next_merc_id(),
        &mut rng,
    );
    deep.prestige.roster.extend(gen1_starters);
    assert_eq!(deep.prestige.roster.len(), 3);

    // Clear L1-3 and build infrastructure
    for layer in 1..=3 {
        mark_layer_cleared(&mut deep.persistent, layer);
    }
    build_infrastructure(deep.persistent.layer_record_mut(1), Infrastructure::Outpost).unwrap();
    build_infrastructure(deep.persistent.layer_record_mut(2), Infrastructure::SupplyCache).unwrap();

    // Accumulate some familiarity on L1
    for _ in 0..4 {
        apply_familiarity_gain(deep.persistent.layer_record_mut(1), MissionType::SupplyRun);
    }
    // L1 familiarity: 4 * 5 = 20

    // Upgrade to rank 2 (requires L3 cleared + 200 marks)
    deep.prestige.warband_marks = 300;
    assert!(try_upgrade_guild_rank(&mut deep.persistent, &mut deep.prestige).is_ok());
    assert_eq!(deep.persistent.guild_rank, GuildRank(2));

    // Snapshot Gen 1 persistent state
    let gen1_deepest = deep.persistent.deepest_layer_reached;
    let gen1_merc_counter = deep.persistent.merc_id_counter;
    let gen1_l1_familiarity = deep.persistent.layer_record(1).unwrap().familiarity;

    // ═══ Prestige 1 (P15→P16) ═══
    deep.on_prestige();

    // Verify persistent survived
    assert_eq!(deep.persistent.guild_rank, GuildRank(2));
    assert_eq!(deep.persistent.deepest_layer_reached, gen1_deepest);
    assert_eq!(deep.persistent.merc_id_counter, gen1_merc_counter);
    assert_eq!(deep.persistent.layer_record(1).unwrap().familiarity, gen1_l1_familiarity);
    assert!(deep.persistent.layer_record(1).unwrap().has_infrastructure(Infrastructure::Outpost));

    // Verify transient reset
    assert!(deep.prestige.roster.is_empty());
    assert_eq!(deep.prestige.warband_marks, 0);
    assert!(deep.prestige.active_missions.is_empty());

    // ═══ Generation 2 (P16→P17) ═══
    let gen2_starters = generate_starter_roster(
        deep.persistent.guild_rank,
        || deep.persistent.next_merc_id(),
        &mut rng,
    );
    deep.prestige.roster.extend(gen2_starters);
    assert_eq!(deep.prestige.roster.len(), 3);
    // IDs should continue from gen 1
    assert!(deep.prestige.roster[0].id > gen1_merc_counter);

    // Push deeper: clear L4-7
    for layer in 4..=7 {
        mark_layer_cleared(&mut deep.persistent, layer);
    }
    assert_eq!(deep.persistent.deepest_layer_reached, 7);

    // Build more infrastructure
    build_infrastructure(deep.persistent.layer_record_mut(4), Infrastructure::Outpost).unwrap();
    build_infrastructure(deep.persistent.layer_record_mut(5), Infrastructure::Watchtower).unwrap();

    // Accumulate more familiarity on L1 (continues from gen 1)
    for _ in 0..4 {
        apply_familiarity_gain(deep.persistent.layer_record_mut(1), MissionType::SupplyRun);
    }
    // L1 familiarity: 20 + 20 = 40

    // Upgrade to rank 3 (requires L7 cleared + 500 marks)
    deep.prestige.warband_marks = 600;
    assert!(try_upgrade_guild_rank(&mut deep.persistent, &mut deep.prestige).is_ok());
    assert_eq!(deep.persistent.guild_rank, GuildRank(3));

    let gen2_merc_counter = deep.persistent.merc_id_counter;

    // ═══ Prestige 2 (P16→P17) ═══
    deep.on_prestige();

    // ═══ Generation 3 Verification ═══
    // All cumulative infrastructure should be present
    assert!(deep.persistent.layer_record(1).unwrap().has_infrastructure(Infrastructure::Outpost));
    assert!(deep.persistent.layer_record(2).unwrap().has_infrastructure(Infrastructure::SupplyCache));
    assert!(deep.persistent.layer_record(4).unwrap().has_infrastructure(Infrastructure::Outpost));
    assert!(deep.persistent.layer_record(5).unwrap().has_infrastructure(Infrastructure::Watchtower));

    // Guild rank 3 preserved (9 roster slots, 2 concurrent missions)
    assert_eq!(deep.persistent.guild_rank, GuildRank(3));
    assert_eq!(deep.persistent.guild_rank.max_roster(), 9);
    assert_eq!(deep.persistent.guild_rank.concurrent_missions(), 2);

    // Deepest layer reached includes gen 2 push
    assert_eq!(deep.persistent.deepest_layer_reached, 7);

    // L1-7 all cleared
    for layer in 1..=7 {
        assert!(is_safe_layer(&deep.persistent, layer));
    }

    // Frontier is L8
    assert!(is_frontier_layer(&deep.persistent, 8));

    // Familiarity accumulated across both generations
    assert_eq!(deep.persistent.layer_record(1).unwrap().familiarity, 40);

    // L5 got Watchtower bonus (+25) in gen 2
    assert_eq!(deep.persistent.layer_record(5).unwrap().familiarity, 25);

    // Transient state is clean
    assert!(deep.prestige.roster.is_empty());
    assert_eq!(deep.prestige.warband_marks, 0);
    assert!(deep.prestige.active_missions.is_empty());

    // New gen 3 starters should continue counter
    let gen3_starters = generate_starter_roster(
        deep.persistent.guild_rank,
        || deep.persistent.next_merc_id(),
        &mut rng,
    );
    deep.prestige.roster.extend(gen3_starters);
    assert!(deep.prestige.roster[0].id > gen2_merc_counter);
}

// ── Serde Roundtrip Across Prestiges ────────────────────────────────────────────

#[test]
fn test_serde_roundtrip_preserves_all_persistent_state() {
    let deep = build_generation_1_state();

    // Serialize
    let json = serde_json::to_string_pretty(&deep).unwrap();
    // Deserialize
    let loaded: DeepState = serde_json::from_str(&json).unwrap();

    // Verify all persistent fields match
    assert_eq!(loaded.persistent.discovered, deep.persistent.discovered);
    assert_eq!(loaded.persistent.guild_rank, deep.persistent.guild_rank);
    assert_eq!(loaded.persistent.deepest_layer_reached, deep.persistent.deepest_layer_reached);
    assert_eq!(loaded.persistent.merc_id_counter, deep.persistent.merc_id_counter);
    assert_eq!(loaded.persistent.mission_id_counter, deep.persistent.mission_id_counter);
    assert_eq!(loaded.persistent.layers.len(), deep.persistent.layers.len());

    for (i, (loaded_layer, orig_layer)) in loaded.persistent.layers.iter()
        .zip(deep.persistent.layers.iter())
        .enumerate()
    {
        assert_eq!(loaded_layer.index, orig_layer.index, "Layer {} index mismatch", i);
        assert_eq!(loaded_layer.familiarity, orig_layer.familiarity, "Layer {} familiarity mismatch", i);
        assert_eq!(loaded_layer.infrastructure, orig_layer.infrastructure, "Layer {} infra mismatch", i);
        assert_eq!(loaded_layer.cleared, orig_layer.cleared, "Layer {} cleared mismatch", i);
    }

    // Verify transient state roundtrips too
    assert_eq!(loaded.prestige.warband_marks, deep.prestige.warband_marks);
    assert_eq!(loaded.prestige.roster.len(), deep.prestige.roster.len());
}

#[test]
fn test_serde_roundtrip_after_prestige() {
    let mut deep = build_generation_1_state();
    deep.on_prestige();

    let json = serde_json::to_string(&deep).unwrap();
    let loaded: DeepState = serde_json::from_str(&json).unwrap();

    // Persistent survived serialization after prestige
    assert!(loaded.persistent.discovered);
    assert_eq!(loaded.persistent.guild_rank, GuildRank(2));
    assert_eq!(loaded.persistent.deepest_layer_reached, 5);

    // Transient is correctly reset in serialized form
    assert!(loaded.prestige.roster.is_empty());
    assert_eq!(loaded.prestige.warband_marks, 0);
}

// ── Edge Cases ──────────────────────────────────────────────────────────────────

#[test]
fn test_prestige_with_injured_mercs_resets_completely() {
    let mut deep = build_generation_1_state();

    // Injure a merc
    deep.prestige.roster[0].status = MercStatus::Injured { missions_remaining: 5 };
    deep.on_prestige();

    assert!(
        deep.prestige.roster.is_empty(),
        "Injured mercs should be gone after prestige"
    );
}

#[test]
fn test_prestige_with_lost_mercs_resets_completely() {
    let mut deep = build_generation_1_state();

    // Mark a merc as lost
    deep.prestige.roster[0].status = MercStatus::Lost;
    deep.on_prestige();

    assert!(
        deep.prestige.roster.is_empty(),
        "Lost mercs should be gone after prestige"
    );
}

#[test]
fn test_prestige_with_mercs_on_mission_resets_completely() {
    let mut deep = build_generation_1_state();

    // Put mercs on mission
    deep.prestige.roster[0].status = MercStatus::OnMission(1);
    deep.prestige.roster[1].status = MercStatus::OnMission(2);
    deep.on_prestige();

    assert!(deep.prestige.roster.is_empty());
    assert!(deep.prestige.active_missions.is_empty());
}

#[test]
fn test_prestige_with_full_roster_resets() {
    let mut rng = seeded_rng();
    let mut deep = build_generation_1_state();

    // Fill roster to capacity (Rank 2 = 7 mercs; already have 3)
    for _ in 0..4 {
        let id = deep.persistent.next_merc_id();
        let merc = quest::deep::Mercenary {
            id,
            name: "Extra".to_string(),
            archetype: MercArchetype::Vanguard,
            power: 14,
            resilience: 12,
            expertise: 4,
            level: 1,
            missions_completed: 0,
            status: MercStatus::Available,
        };
        deep.prestige.roster.push(merc);
    }
    assert_eq!(deep.prestige.roster.len(), 7); // Full at rank 2

    deep.on_prestige();
    assert!(deep.prestige.roster.is_empty());
}

#[test]
fn test_prestige_with_empty_roster_works() {
    let mut deep = DeepState::new();
    deep.persistent.discovered = true;
    // No mercs, no marks, nothing
    deep.on_prestige();
    assert!(deep.prestige.roster.is_empty());
    assert_eq!(deep.prestige.warband_marks, 0);
}

#[test]
fn test_prestige_with_pending_results_resets() {
    let mut deep = build_generation_1_state();

    // Add a pending result mission
    deep.prestige.pending_results.push(quest::deep::Mission {
        id: 99,
        mission_type: MissionType::Expedition,
        layer: 3,
        squad: vec![1, 2],
        started_at: chrono::Utc::now() - chrono::Duration::hours(10),
        ends_at: chrono::Utc::now() - chrono::Duration::hours(1),
        events: vec![],
        pending_event_index: 0,
        status: quest::deep::MissionStatus::Completed,
        result: Some(quest::deep::MissionResult {
            outcome: MissionOutcome::Success,
            marks_earned: 100,
            xp_earned: 500,
            stormglass_earned: 5,
            item_ilvl: None,
            prestige_fragment: false,
            injured_mercs: vec![],
            lost_mercs: vec![],
            merc_level_ups: vec![],
        }),
    });

    deep.on_prestige();
    assert!(deep.prestige.pending_results.is_empty());
}

#[test]
fn test_prestige_does_not_reset_undiscovered_state() {
    let mut deep = DeepState::new();
    // The Deep not yet discovered
    assert!(!deep.persistent.discovered);
    deep.on_prestige();
    assert!(!deep.persistent.discovered);
    assert_eq!(deep.persistent.guild_rank, GuildRank(1));
}

// ── Infrastructure Cannot Be Lost ───────────────────────────────────────────────

#[test]
fn test_infrastructure_survives_multiple_prestiges() {
    let mut rng = seeded_rng();
    let mut deep = DeepState::new();
    deep.persistent.discovered = true;

    // Gen 1: Build outpost on L1
    mark_layer_cleared(&mut deep.persistent, 1);
    build_infrastructure(deep.persistent.layer_record_mut(1), Infrastructure::Outpost).unwrap();

    deep.on_prestige();
    assert!(deep.persistent.layer_record(1).unwrap().has_infrastructure(Infrastructure::Outpost));

    // Gen 2: Build supply cache on L1
    build_infrastructure(deep.persistent.layer_record_mut(1), Infrastructure::SupplyCache).unwrap();

    deep.on_prestige();
    assert!(deep.persistent.layer_record(1).unwrap().has_infrastructure(Infrastructure::Outpost));
    assert!(deep.persistent.layer_record(1).unwrap().has_infrastructure(Infrastructure::SupplyCache));

    // Gen 3: Build watchtower on L1
    build_infrastructure(deep.persistent.layer_record_mut(1), Infrastructure::Watchtower).unwrap();

    deep.on_prestige();
    assert!(deep.persistent.layer_record(1).unwrap().has_infrastructure(Infrastructure::Outpost));
    assert!(deep.persistent.layer_record(1).unwrap().has_infrastructure(Infrastructure::SupplyCache));
    assert!(deep.persistent.layer_record(1).unwrap().has_infrastructure(Infrastructure::Watchtower));

    // All 3 infrastructure types survive 3 prestiges
    assert_eq!(deep.persistent.layer_record(1).unwrap().infrastructure.len(), 3);
}

// ── Guild Rank Progression Across Prestiges ─────────────────────────────────────

#[test]
fn test_guild_rank_cannot_regress_on_prestige() {
    let mut deep = DeepState::new();
    deep.persistent.discovered = true;
    deep.persistent.guild_rank = GuildRank(3);

    deep.on_prestige();
    assert_eq!(deep.persistent.guild_rank, GuildRank(3));

    deep.on_prestige();
    assert_eq!(deep.persistent.guild_rank, GuildRank(3));
}

#[test]
fn test_guild_rank_upgrade_across_generations() {
    let mut deep = DeepState::new();
    deep.persistent.discovered = true;

    // Gen 1: Upgrade to rank 2 (requires L3 cleared + 200 marks)
    for layer in 1..=3 {
        mark_layer_cleared(&mut deep.persistent, layer);
    }
    deep.prestige.warband_marks = 200;
    assert!(try_upgrade_guild_rank(&mut deep.persistent, &mut deep.prestige).is_ok());
    assert_eq!(deep.persistent.guild_rank, GuildRank(2));

    // Prestige
    deep.on_prestige();
    assert_eq!(deep.persistent.guild_rank, GuildRank(2));

    // Gen 2: Upgrade to rank 3 (requires L7 cleared + 500 marks)
    for layer in 4..=7 {
        mark_layer_cleared(&mut deep.persistent, layer);
    }
    deep.prestige.warband_marks = 500;
    assert!(try_upgrade_guild_rank(&mut deep.persistent, &mut deep.prestige).is_ok());
    assert_eq!(deep.persistent.guild_rank, GuildRank(3));

    // Prestige
    deep.on_prestige();
    assert_eq!(deep.persistent.guild_rank, GuildRank(3));
    assert_eq!(deep.persistent.guild_rank.max_roster(), 9);
    assert_eq!(deep.persistent.guild_rank.concurrent_missions(), 2);
}

// ── Frontier Layer Tracking ─────────────────────────────────────────────────────

#[test]
fn test_frontier_layer_tracks_correctly_across_prestiges() {
    let mut deep = DeepState::new();
    deep.persistent.discovered = true;

    // Start: frontier is L1
    assert_eq!(deep.persistent.frontier_layer(), 1);

    // Gen 1: clear L1-3
    for layer in 1..=3 {
        mark_layer_cleared(&mut deep.persistent, layer);
    }
    assert_eq!(deep.persistent.frontier_layer(), 4);

    // Prestige: frontier should still be L4
    deep.on_prestige();
    assert_eq!(deep.persistent.frontier_layer(), 4);

    // Gen 2: clear L4-5
    for layer in 4..=5 {
        mark_layer_cleared(&mut deep.persistent, layer);
    }
    assert_eq!(deep.persistent.frontier_layer(), 6);

    // Prestige: frontier remains L6
    deep.on_prestige();
    assert_eq!(deep.persistent.frontier_layer(), 6);
}

// ── Duration Ratchet: Gen 2 Is Faster Than Gen 1 ────────────────────────────────

#[test]
fn test_gen2_supply_run_is_faster_than_gen1_baseline() {
    let base_shallows_supply = base_mission_duration_secs(LayerTier::Shallows, MissionType::SupplyRun);

    // Gen 1 (no modifiers)
    let gen1_duration = apply_duration_modifiers(base_shallows_supply, &DurationModifiers {
        has_outpost: false,
        familiarity: 0,
        has_saboteur: false,
        saboteur_is_veteran: false,
        is_overpowered: false,
    });

    // Gen 2 (Outpost on L1 + 40% familiarity from gen 1)
    let gen2_duration = apply_duration_modifiers(base_shallows_supply, &DurationModifiers {
        has_outpost: true,
        familiarity: 40, // Mapped level (25-49%)
        has_saboteur: false,
        saboteur_is_veteran: false,
        is_overpowered: false,
    });

    assert!(
        gen2_duration < gen1_duration,
        "Gen 2 with Outpost + familiarity should be faster: {} < {}",
        gen2_duration,
        gen1_duration,
    );

    // Verify the exact reduction: Outpost -25% * Mapped familiarity -10% = 0.75 * 0.90 = 0.675x
    let expected = (base_shallows_supply as f64 * 0.75 * 0.90) as u64;
    assert_eq!(gen2_duration, expected);
}

#[test]
fn test_gen3_supply_run_is_faster_than_gen2() {
    let base_shallows_supply = base_mission_duration_secs(LayerTier::Shallows, MissionType::SupplyRun);

    // Gen 2 (Outpost + Mapped familiarity 40%)
    let gen2_duration = apply_duration_modifiers(base_shallows_supply, &DurationModifiers {
        has_outpost: true,
        familiarity: 40, // Mapped
        has_saboteur: false,
        saboteur_is_veteran: false,
        is_overpowered: false,
    });

    // Gen 3 (Outpost + Mastered familiarity 80% from accumulated missions)
    let gen3_duration = apply_duration_modifiers(base_shallows_supply, &DurationModifiers {
        has_outpost: true,
        familiarity: 80, // Mastered
        has_saboteur: false,
        saboteur_is_veteran: false,
        is_overpowered: false,
    });

    assert!(
        gen3_duration < gen2_duration,
        "Gen 3 with Mastered familiarity should be faster than Gen 2: {} < {}",
        gen3_duration,
        gen2_duration,
    );
}

// ── ID Monotonicity Across Generations ──────────────────────────────────────────

#[test]
fn test_merc_ids_are_globally_unique_across_generations() {
    let mut rng = seeded_rng();
    let mut deep = DeepState::new();
    deep.persistent.discovered = true;

    let mut all_ids: Vec<u64> = Vec::new();

    // Gen 1
    let gen1 = generate_starter_roster(
        deep.persistent.guild_rank,
        || deep.persistent.next_merc_id(),
        &mut rng,
    );
    all_ids.extend(gen1.iter().map(|m| m.id));
    deep.on_prestige();

    // Gen 2
    let gen2 = generate_starter_roster(
        deep.persistent.guild_rank,
        || deep.persistent.next_merc_id(),
        &mut rng,
    );
    all_ids.extend(gen2.iter().map(|m| m.id));
    deep.on_prestige();

    // Gen 3
    let gen3 = generate_starter_roster(
        deep.persistent.guild_rank,
        || deep.persistent.next_merc_id(),
        &mut rng,
    );
    all_ids.extend(gen3.iter().map(|m| m.id));

    // All 9 IDs should be unique
    assert_eq!(all_ids.len(), 9);
    let unique: std::collections::HashSet<u64> = all_ids.iter().cloned().collect();
    assert_eq!(unique.len(), 9, "All merc IDs must be unique across generations");

    // IDs should be strictly increasing
    for window in all_ids.windows(2) {
        assert!(
            window[1] > window[0],
            "Merc IDs should be monotonically increasing: {} should be > {}",
            window[1],
            window[0],
        );
    }
}

// ── Prestige With High Marks ────────────────────────────────────────────────────

#[test]
fn test_prestige_resets_large_mark_balance() {
    let mut deep = build_generation_1_state();
    deep.prestige.warband_marks = 99_999;
    deep.on_prestige();
    assert_eq!(deep.prestige.warband_marks, 0);
}

// ── Multiple Rapid Prestiges ────────────────────────────────────────────────────

#[test]
fn test_rapid_successive_prestiges_are_idempotent_on_persistent() {
    let mut deep = build_generation_1_state();

    // Snapshot persistent state
    let persistent_before = deep.persistent.clone();

    // Prestige 5 times rapidly
    for _ in 0..5 {
        deep.on_prestige();
    }

    // Persistent state should be unchanged
    assert_eq!(deep.persistent.discovered, persistent_before.discovered);
    assert_eq!(deep.persistent.guild_rank, persistent_before.guild_rank);
    assert_eq!(deep.persistent.deepest_layer_reached, persistent_before.deepest_layer_reached);
    assert_eq!(deep.persistent.merc_id_counter, persistent_before.merc_id_counter);
    assert_eq!(deep.persistent.layers.len(), persistent_before.layers.len());

    for (i, layer) in deep.persistent.layers.iter().enumerate() {
        assert_eq!(layer.cleared, persistent_before.layers[i].cleared);
        assert_eq!(layer.familiarity, persistent_before.layers[i].familiarity);
        assert_eq!(layer.infrastructure, persistent_before.layers[i].infrastructure);
    }
}

// ── is_active() Behavior ────────────────────────────────────────────────────────

#[test]
fn test_is_active_survives_prestige() {
    let mut deep = DeepState::new();
    assert!(!deep.is_active());

    deep.persistent.discovered = true;
    assert!(deep.is_active());

    deep.on_prestige();
    assert!(deep.is_active(), "is_active should survive prestige");
}
