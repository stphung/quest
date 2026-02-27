//! End-to-end integration tests: The Deep prestige persistence model.
//!
//! Tests the full generational lifecycle across multiple prestige cycles, verifying
//! the "ratchet" — that each generation starts stronger than the last due to
//! accumulated persistent state (guild rank, cleared layers, infrastructure,
//! familiarity) and that operational state (mercs, marks, missions) persists.
//!
//! Covers:
//!  1. Generation 1 (P15→P16): Discovery, roster init, layer clearing, infra build,
//!     guild rank upgrade, then prestige.
//!  2. Post-prestige verification: all state preserved (persistent + operational).
//!  3. Generation 2 (P16→P17): Continued progress with persisted roster and marks.
//!  4. Generation 3 verification: Cumulative benefits across all prior generations.
//!  5. Edge cases: mid-mission prestige, injured mercs, full/empty roster.

use quest::deep::{
    apply_duration_modifiers, apply_familiarity_gain, base_mission_duration_secs,
    build_infrastructure, generate_starter_roster, is_frontier_layer, is_safe_layer,
    mark_layer_cleared, try_upgrade_guild_rank, DurationModifiers, FamiliarityLevel,
};
use quest::deep::{
    DeepPersistent, DeepState, GuildRank, Infrastructure, LayerTier, MercArchetype, MercStatus,
    MissionOutcome, MissionType,
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
    build_infrastructure(
        deep.persistent.layer_record_mut(2),
        Infrastructure::SupplyCache,
    )
    .unwrap();
    build_infrastructure(
        deep.persistent.layer_record_mut(3),
        Infrastructure::Watchtower,
    )
    .unwrap();

    // Accumulate familiarity via missions on L1-5
    // Simulate several missions on each layer
    for layer in 1..=5 {
        let record = deep.persistent.layer_record_mut(layer);
        // Supply run (+8) x3 + Recon (+20) + Expedition (+10) = 54 familiarity
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
    assert!(deep
        .persistent
        .layer_record(1)
        .unwrap()
        .has_infrastructure(Infrastructure::Outpost));
    assert!(deep
        .persistent
        .layer_record(2)
        .unwrap()
        .has_infrastructure(Infrastructure::SupplyCache));
    assert!(deep
        .persistent
        .layer_record(3)
        .unwrap()
        .has_infrastructure(Infrastructure::Watchtower));

    // Familiarity on all 5 layers
    // L1-2, L4-5: 54 from missions only (3×8 + 20 + 10)
    // L3: 54 from missions + 40 from Watchtower build = 94
    for layer in 1..=5 {
        let record = deep.persistent.layer_record(layer).unwrap();
        let expected = if layer == 3 { 94 } else { 54 };
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

// ── Prestige Preservation (Gen 1 → Gen 2) ──────────────────────────────────────

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
    assert_eq!(deep.persistent.guild_rank.display_name(), "Company");
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

    assert!(deep
        .persistent
        .layer_record(1)
        .unwrap()
        .has_infrastructure(Infrastructure::Outpost));
    assert!(deep
        .persistent
        .layer_record(2)
        .unwrap()
        .has_infrastructure(Infrastructure::SupplyCache));
    assert!(deep
        .persistent
        .layer_record(3)
        .unwrap()
        .has_infrastructure(Infrastructure::Watchtower));
}

#[test]
fn test_prestige_preserves_familiarity() {
    let mut deep = build_generation_1_state();
    deep.on_prestige();

    for layer in 1..=5 {
        let record = deep.persistent.layer_record(layer).unwrap();
        // L3 has Watchtower bonus (+40) in addition to mission familiarity (54)
        let expected = if layer == 3 { 94 } else { 54 };
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
fn test_prestige_preserves_roster() {
    let mut deep = build_generation_1_state();
    let roster_len = deep.prestige.roster.len();
    assert!(roster_len > 0);
    deep.on_prestige();
    assert_eq!(
        deep.prestige.roster.len(),
        roster_len,
        "Roster must persist after prestige"
    );
}

#[test]
fn test_prestige_preserves_warband_marks() {
    let mut deep = build_generation_1_state();
    let marks = deep.prestige.warband_marks;
    assert!(marks > 0);
    deep.on_prestige();
    assert_eq!(deep.prestige.warband_marks, marks);
}

#[test]
fn test_prestige_preserves_active_missions() {
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
        is_first_orders: false,
    });
    deep.on_prestige();
    assert_eq!(deep.prestige.active_missions.len(), 1);
}

#[test]
fn test_prestige_preserves_available_missions() {
    let mut deep = build_generation_1_state();
    deep.prestige
        .available_missions
        .push(quest::deep::AvailableMission {
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
    assert_eq!(deep.prestige.available_missions.len(), 1);
}

#[test]
fn test_prestige_preserves_recruit_pool() {
    let mut deep = build_generation_1_state();
    deep.on_prestige();
    // Recruit pool persists (empty here since build_generation_1_state doesn't populate it)
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
fn test_gen2_recon_benefits_from_gen1_infrastructure() {
    let mut deep = build_generation_1_state();
    deep.on_prestige();

    // L1 has an Outpost — recon missions should be faster.
    // Use Recon (3600s base) instead of SupplyRun (1800s base = floor) so
    // duration modifiers can take effect above the 30-minute floor.
    let base_duration = base_mission_duration_secs(LayerTier::Shallows, MissionType::Recon);
    let with_outpost = apply_duration_modifiers(
        base_duration,
        &DurationModifiers {
            has_outpost: true,
            familiarity: 40, // from gen 1
            has_saboteur: false,
            saboteur_is_veteran: false,
            is_overpowered: false,
            bridge_layers: 0,
        },
    );
    let without_infra = apply_duration_modifiers(
        base_duration,
        &DurationModifiers {
            has_outpost: false,
            familiarity: 0,
            has_saboteur: false,
            saboteur_is_veteran: false,
            is_overpowered: false,
            bridge_layers: 0,
        },
    );

    assert!(
        with_outpost < without_infra,
        "Gen 2 recon on L1 should be faster due to gen 1 infrastructure: {} vs {}",
        with_outpost,
        without_infra,
    );
}

#[test]
fn test_gen2_familiarity_continues_accumulating() {
    let mut deep = build_generation_1_state();
    deep.on_prestige();

    // L1 had 54 familiarity from gen 1 — add more in gen 2
    let record = deep.persistent.layer_record_mut(1);
    let fam_before = record.familiarity;
    apply_familiarity_gain(record, MissionType::Recon); // +20
    assert_eq!(record.familiarity, fam_before + 20);
    assert_eq!(record.familiarity, 74);
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
    let result = build_infrastructure(deep.persistent.layer_record_mut(6), Infrastructure::Outpost);
    assert!(result.is_ok());
    assert!(deep
        .persistent
        .layer_record(6)
        .unwrap()
        .has_infrastructure(Infrastructure::Outpost));
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
    build_infrastructure(
        deep.persistent.layer_record_mut(2),
        Infrastructure::SupplyCache,
    )
    .unwrap();

    // Accumulate some familiarity on L1
    for _ in 0..4 {
        apply_familiarity_gain(deep.persistent.layer_record_mut(1), MissionType::SupplyRun);
    }
    // L1 familiarity: 4 * 8 = 32

    // Upgrade to rank 2 (requires L3 cleared + 200 marks)
    deep.prestige.warband_marks = 300;
    assert!(try_upgrade_guild_rank(&mut deep.persistent, &mut deep.prestige).is_ok());
    assert_eq!(deep.persistent.guild_rank, GuildRank(2));

    // Snapshot Gen 1 persistent state
    let gen1_deepest = deep.persistent.deepest_layer_reached;
    let gen1_merc_counter = deep.persistent.merc_id_counter;
    let gen1_l1_familiarity = deep.persistent.layer_record(1).unwrap().familiarity;

    // ═══ Prestige 1 (P15→P16) ═══
    let gen1_marks = deep.prestige.warband_marks;
    let gen1_roster_len = deep.prestige.roster.len();
    deep.on_prestige();

    // Verify persistent survived
    assert_eq!(deep.persistent.guild_rank, GuildRank(2));
    assert_eq!(deep.persistent.deepest_layer_reached, gen1_deepest);
    assert_eq!(deep.persistent.merc_id_counter, gen1_merc_counter);
    assert_eq!(
        deep.persistent.layer_record(1).unwrap().familiarity,
        gen1_l1_familiarity
    );
    assert!(deep
        .persistent
        .layer_record(1)
        .unwrap()
        .has_infrastructure(Infrastructure::Outpost));

    // Verify operational state preserved
    assert_eq!(deep.prestige.roster.len(), gen1_roster_len);
    assert_eq!(deep.prestige.warband_marks, gen1_marks);

    // ═══ Generation 2 (P16→P17) ═══
    // Roster persists from gen 1 — no need to generate new starters
    assert_eq!(deep.prestige.roster.len(), 3);

    // Push deeper: clear L4-7
    for layer in 4..=7 {
        mark_layer_cleared(&mut deep.persistent, layer);
    }
    assert_eq!(deep.persistent.deepest_layer_reached, 7);

    // Build more infrastructure
    build_infrastructure(deep.persistent.layer_record_mut(4), Infrastructure::Outpost).unwrap();
    build_infrastructure(
        deep.persistent.layer_record_mut(5),
        Infrastructure::Watchtower,
    )
    .unwrap();

    // Accumulate more familiarity on L1 (continues from gen 1)
    for _ in 0..4 {
        apply_familiarity_gain(deep.persistent.layer_record_mut(1), MissionType::SupplyRun);
    }
    // L1 familiarity: 32 + 32 = 64

    // Upgrade to rank 3 (requires L7 cleared + 500 marks)
    deep.prestige.warband_marks = 600;
    assert!(try_upgrade_guild_rank(&mut deep.persistent, &mut deep.prestige).is_ok());
    assert_eq!(deep.persistent.guild_rank, GuildRank(3));

    let gen2_marks = deep.prestige.warband_marks;

    // ═══ Prestige 2 (P16→P17) ═══
    deep.on_prestige();

    // ═══ Generation 3 Verification ═══
    // All cumulative infrastructure should be present
    assert!(deep
        .persistent
        .layer_record(1)
        .unwrap()
        .has_infrastructure(Infrastructure::Outpost));
    assert!(deep
        .persistent
        .layer_record(2)
        .unwrap()
        .has_infrastructure(Infrastructure::SupplyCache));
    assert!(deep
        .persistent
        .layer_record(4)
        .unwrap()
        .has_infrastructure(Infrastructure::Outpost));
    assert!(deep
        .persistent
        .layer_record(5)
        .unwrap()
        .has_infrastructure(Infrastructure::Watchtower));

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
    assert_eq!(deep.persistent.layer_record(1).unwrap().familiarity, 64);

    // L5 got Watchtower bonus (+40) in gen 2
    assert_eq!(deep.persistent.layer_record(5).unwrap().familiarity, 40);

    // Operational state persists through prestige
    assert_eq!(deep.prestige.roster.len(), 3);
    assert_eq!(deep.prestige.warband_marks, gen2_marks);
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
    assert_eq!(
        loaded.persistent.deepest_layer_reached,
        deep.persistent.deepest_layer_reached
    );
    assert_eq!(
        loaded.persistent.merc_id_counter,
        deep.persistent.merc_id_counter
    );
    assert_eq!(
        loaded.persistent.mission_id_counter,
        deep.persistent.mission_id_counter
    );
    assert_eq!(loaded.persistent.layers.len(), deep.persistent.layers.len());

    for (i, (loaded_layer, orig_layer)) in loaded
        .persistent
        .layers
        .iter()
        .zip(deep.persistent.layers.iter())
        .enumerate()
    {
        assert_eq!(
            loaded_layer.index, orig_layer.index,
            "Layer {} index mismatch",
            i
        );
        assert_eq!(
            loaded_layer.familiarity, orig_layer.familiarity,
            "Layer {} familiarity mismatch",
            i
        );
        assert_eq!(
            loaded_layer.infrastructure, orig_layer.infrastructure,
            "Layer {} infra mismatch",
            i
        );
        assert_eq!(
            loaded_layer.cleared, orig_layer.cleared,
            "Layer {} cleared mismatch",
            i
        );
    }

    // Verify transient state roundtrips too
    assert_eq!(loaded.prestige.warband_marks, deep.prestige.warband_marks);
    assert_eq!(loaded.prestige.roster.len(), deep.prestige.roster.len());
}

#[test]
fn test_serde_roundtrip_after_prestige() {
    let mut deep = build_generation_1_state();
    let marks_before = deep.prestige.warband_marks;
    let roster_len = deep.prestige.roster.len();
    deep.on_prestige();

    let json = serde_json::to_string(&deep).unwrap();
    let loaded: DeepState = serde_json::from_str(&json).unwrap();

    // Persistent survived serialization after prestige
    assert!(loaded.persistent.discovered);
    assert_eq!(loaded.persistent.guild_rank, GuildRank(2));
    assert_eq!(loaded.persistent.deepest_layer_reached, 5);

    // Operational state preserved in serialized form
    assert_eq!(loaded.prestige.roster.len(), roster_len);
    assert_eq!(loaded.prestige.warband_marks, marks_before);
}

// ── Edge Cases ──────────────────────────────────────────────────────────────────

#[test]
fn test_prestige_with_injured_mercs_preserves_roster() {
    let mut deep = build_generation_1_state();

    // Injure a merc
    deep.prestige.roster[0].status = MercStatus::Injured {
        missions_remaining: 5,
    };
    let roster_len = deep.prestige.roster.len();
    deep.on_prestige();

    assert_eq!(
        deep.prestige.roster.len(),
        roster_len,
        "Injured mercs should persist after prestige"
    );
}

#[test]
fn test_prestige_with_lost_mercs_preserves_roster() {
    let mut deep = build_generation_1_state();

    // Mark a merc as lost
    deep.prestige.roster[0].status = MercStatus::Lost;
    let roster_len = deep.prestige.roster.len();
    deep.on_prestige();

    assert_eq!(
        deep.prestige.roster.len(),
        roster_len,
        "Roster (including lost mercs) should persist after prestige"
    );
}

#[test]
fn test_prestige_with_mercs_on_mission_preserves_state() {
    let mut deep = build_generation_1_state();

    // Put mercs on mission
    deep.prestige.roster[0].status = MercStatus::OnMission(1);
    deep.prestige.roster[1].status = MercStatus::OnMission(2);
    let roster_len = deep.prestige.roster.len();
    deep.on_prestige();

    assert_eq!(deep.prestige.roster.len(), roster_len);
}

#[test]
fn test_prestige_with_full_roster_preserves() {
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
    assert_eq!(deep.prestige.roster.len(), 7);
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
fn test_prestige_with_pending_results_preserves() {
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
        is_first_orders: false,
        result: Some(quest::deep::MissionResult {
            outcome: MissionOutcome::Success,
            marks_earned: 100,
            xp_earned: 500,
            stormglass_earned: 5,
            item_ilvl: None,
            injured_mercs: vec![],
            lost_mercs: vec![],
            merc_level_ups: vec![],
            danger_bonus_xp: false,
        }),
    });

    deep.on_prestige();
    assert_eq!(deep.prestige.pending_results.len(), 1);
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
    let mut deep = DeepState::new();
    deep.persistent.discovered = true;

    // Gen 1: Build outpost on L1
    mark_layer_cleared(&mut deep.persistent, 1);
    build_infrastructure(deep.persistent.layer_record_mut(1), Infrastructure::Outpost).unwrap();

    deep.on_prestige();
    assert!(deep
        .persistent
        .layer_record(1)
        .unwrap()
        .has_infrastructure(Infrastructure::Outpost));

    // Gen 2: Build supply cache on L1
    build_infrastructure(
        deep.persistent.layer_record_mut(1),
        Infrastructure::SupplyCache,
    )
    .unwrap();

    deep.on_prestige();
    assert!(deep
        .persistent
        .layer_record(1)
        .unwrap()
        .has_infrastructure(Infrastructure::Outpost));
    assert!(deep
        .persistent
        .layer_record(1)
        .unwrap()
        .has_infrastructure(Infrastructure::SupplyCache));

    // Gen 3: Build watchtower on L1
    build_infrastructure(
        deep.persistent.layer_record_mut(1),
        Infrastructure::Watchtower,
    )
    .unwrap();

    deep.on_prestige();
    assert!(deep
        .persistent
        .layer_record(1)
        .unwrap()
        .has_infrastructure(Infrastructure::Outpost));
    assert!(deep
        .persistent
        .layer_record(1)
        .unwrap()
        .has_infrastructure(Infrastructure::SupplyCache));
    assert!(deep
        .persistent
        .layer_record(1)
        .unwrap()
        .has_infrastructure(Infrastructure::Watchtower));

    // All 3 infrastructure types survive 3 prestiges
    assert_eq!(
        deep.persistent
            .layer_record(1)
            .unwrap()
            .infrastructure
            .len(),
        3
    );
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
fn test_gen2_recon_is_faster_than_gen1_baseline() {
    // Use Recon (3600s base in Shallows) instead of SupplyRun (1800s = floor) so
    // duration modifiers can take effect above the 30-minute floor.
    let base_shallows_recon = base_mission_duration_secs(LayerTier::Shallows, MissionType::Recon);

    // Gen 1 (no modifiers)
    let gen1_duration = apply_duration_modifiers(
        base_shallows_recon,
        &DurationModifiers {
            has_outpost: false,
            familiarity: 0,
            has_saboteur: false,
            saboteur_is_veteran: false,
            is_overpowered: false,
            bridge_layers: 0,
        },
    );

    // Gen 2 (Outpost on L1 + 40% familiarity from gen 1)
    let gen2_duration = apply_duration_modifiers(
        base_shallows_recon,
        &DurationModifiers {
            has_outpost: true,
            familiarity: 40, // Mapped level (25-49%)
            has_saboteur: false,
            saboteur_is_veteran: false,
            is_overpowered: false,
            bridge_layers: 0,
        },
    );

    assert!(
        gen2_duration < gen1_duration,
        "Gen 2 with Outpost + familiarity should be faster: {} < {}",
        gen2_duration,
        gen1_duration,
    );

    // Verify the exact reduction: Outpost -25% * Mapped familiarity -15% = 0.75 * 0.85 = 0.6375x
    let expected = (base_shallows_recon as f64 * 0.75 * 0.85) as u64;
    assert_eq!(gen2_duration, expected);
}

#[test]
fn test_gen3_recon_is_faster_than_gen2() {
    // Use Recon (3600s base in Shallows) instead of SupplyRun (1800s = floor) so
    // duration modifiers can take effect above the 30-minute floor.
    let base_shallows_recon = base_mission_duration_secs(LayerTier::Shallows, MissionType::Recon);

    // Gen 2 (Outpost + Mapped familiarity 40%)
    let gen2_duration = apply_duration_modifiers(
        base_shallows_recon,
        &DurationModifiers {
            has_outpost: true,
            familiarity: 40, // Mapped
            has_saboteur: false,
            saboteur_is_veteran: false,
            is_overpowered: false,
            bridge_layers: 0,
        },
    );

    // Gen 3 (Outpost + Mastered familiarity 80% from accumulated missions)
    let gen3_duration = apply_duration_modifiers(
        base_shallows_recon,
        &DurationModifiers {
            has_outpost: true,
            familiarity: 80, // Mastered
            has_saboteur: false,
            saboteur_is_veteran: false,
            is_overpowered: false,
            bridge_layers: 0,
        },
    );

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
    assert_eq!(
        unique.len(),
        9,
        "All merc IDs must be unique across generations"
    );

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
fn test_prestige_preserves_large_mark_balance() {
    let mut deep = build_generation_1_state();
    deep.prestige.warband_marks = 99_999;
    deep.on_prestige();
    assert_eq!(deep.prestige.warband_marks, 99_999);
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
    assert_eq!(
        deep.persistent.deepest_layer_reached,
        persistent_before.deepest_layer_reached
    );
    assert_eq!(
        deep.persistent.merc_id_counter,
        persistent_before.merc_id_counter
    );
    assert_eq!(deep.persistent.layers.len(), persistent_before.layers.len());

    for (i, layer) in deep.persistent.layers.iter().enumerate() {
        assert_eq!(layer.cleared, persistent_before.layers[i].cleared);
        assert_eq!(layer.familiarity, persistent_before.layers[i].familiarity);
        assert_eq!(
            layer.infrastructure,
            persistent_before.layers[i].infrastructure
        );
    }
}

// ── Generation Counter ──────────────────────────────────────────────────────────

#[test]
fn test_generation_counter_increments_on_prestige() {
    let mut deep = DeepState::new();
    deep.persistent.discovered = true;
    assert_eq!(deep.persistent.generation_counter, 0);
    assert_eq!(deep.prestige.generation_number, 0);

    deep.on_prestige();
    assert_eq!(deep.persistent.generation_counter, 1);
    assert_eq!(deep.prestige.generation_number, 1);

    deep.on_prestige();
    assert_eq!(deep.persistent.generation_counter, 2);
    assert_eq!(deep.prestige.generation_number, 2);
}

#[test]
fn test_generation_counter_persists_across_saves() {
    let mut deep = DeepState::new();
    deep.persistent.discovered = true;
    deep.persistent.generation_counter = 5;

    let json = serde_json::to_string(&deep).unwrap();
    let loaded: DeepState = serde_json::from_str(&json).unwrap();

    assert_eq!(loaded.persistent.generation_counter, 5);
}

#[test]
fn test_generation_counter_backward_compat_defaults_to_zero() {
    // JSON missing the generation_counter field — serde(default) should give 0.
    let json = r#"{
        "discovered": true,
        "guild_rank": 1,
        "guild_upgrade_cost": 500,
        "layers": [],
        "deepest_layer_reached": 0,
        "merc_id_counter": 0,
        "mission_id_counter": 0
    }"#;
    let loaded: quest::deep::DeepPersistent = serde_json::from_str(json).unwrap();
    assert_eq!(loaded.generation_counter, 0);
}

// ── Warband Log Serde ───────────────────────────────────────────────────────────

#[test]
fn test_warband_log_serde_roundtrip() {
    let mut prestige = quest::deep::DeepPrestige::new();
    prestige.warband_log.push(quest::deep::WarbandLogEntry {
        mission_name: "Supply Run L2".to_string(),
        layer: 2,
        outcome: MissionOutcome::Success,
        marks_earned: 120,
        timestamp: chrono::Utc::now(),
    });

    let json = serde_json::to_string(&prestige).unwrap();
    let loaded: quest::deep::DeepPrestige = serde_json::from_str(&json).unwrap();

    assert_eq!(loaded.warband_log.len(), 1);
    assert_eq!(loaded.warband_log[0].mission_name, "Supply Run L2");
    assert_eq!(loaded.warband_log[0].layer, 2);
    assert_eq!(loaded.warband_log[0].outcome, MissionOutcome::Success);
    assert_eq!(loaded.warband_log[0].marks_earned, 120);
}

#[test]
fn test_warband_log_backward_compat_defaults_to_empty() {
    // DeepPrestige JSON missing the warband_log field — serde(default) should give empty Vec.
    let json = r#"{
        "warband_marks": 0,
        "roster": [],
        "active_missions": [],
        "available_missions": [],
        "recruit_pool": {"candidates": [], "refreshed_at": "2026-01-01T00:00:00Z", "recruit_costs": []},
        "pending_results": []
    }"#;
    let loaded: quest::deep::DeepPrestige = serde_json::from_str(json).unwrap();
    assert!(loaded.warband_log.is_empty());
}

// ── EventChoice / MissionResult Backward Compat ─────────────────────────────────

#[test]
fn test_risk_percent_backward_compat_defaults_to_none() {
    // EventChoice JSON missing the risk_percent field — serde(default) should give None.
    let json = r#"{
        "label": "Dig through",
        "required_archetype": null,
        "time_delta_secs": 3600,
        "is_risky": false,
        "unlocks_bonus_event": false
    }"#;
    let loaded: quest::deep::EventChoice = serde_json::from_str(json).unwrap();
    assert_eq!(loaded.risk_percent, None);
}

#[test]
fn test_mission_result_danger_bonus_xp_backward_compat() {
    // MissionResult JSON missing danger_bonus_xp — serde(default) should give false.
    let json = r#"{
        "outcome": "Success",
        "marks_earned": 100,
        "xp_earned": 500,
        "stormglass_earned": 5,
        "item_ilvl": null,
        "injured_mercs": [],
        "lost_mercs": [],
        "merc_level_ups": []
    }"#;
    let loaded: quest::deep::MissionResult = serde_json::from_str(json).unwrap();
    assert!(!loaded.danger_bonus_xp);
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

// ── Rift Resonance and Story Chain Serde ──────────────────────────────────────

#[test]
fn test_deep_persistent_rift_resonance_defaults_on_missing() {
    let json = r#"{"discovered":false,"guild_rank":1,"guild_upgrade_cost":0,"layers":[],"deepest_layer_reached":0,"merc_id_counter":0,"mission_id_counter":0}"#;
    let persistent: DeepPersistent = serde_json::from_str(json).unwrap();
    assert_eq!(persistent.rift_resonance, 0);
    assert_eq!(persistent.deep_story_stage, 0);
    assert!(!persistent.gateway_opened);
}

#[test]
fn test_deep_persistent_rift_resonance_roundtrip() {
    let mut persistent = DeepPersistent::new();
    persistent.rift_resonance = 7;
    persistent.deep_story_stage = 4;
    persistent.gateway_opened = true;

    let json = serde_json::to_string(&persistent).unwrap();
    let loaded: DeepPersistent = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.rift_resonance, 7);
    assert_eq!(loaded.deep_story_stage, 4);
    assert!(loaded.gateway_opened);
}

#[test]
fn test_rift_resonance_survives_prestige_serde_roundtrip() {
    let mut deep = DeepState::new();
    deep.persistent.rift_resonance = 5;
    deep.persistent.deep_story_stage = 3;
    deep.on_prestige();

    let json = serde_json::to_string(&deep.persistent).unwrap();
    let loaded: DeepPersistent = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.rift_resonance, 5);
    assert_eq!(loaded.deep_story_stage, 3);
}
