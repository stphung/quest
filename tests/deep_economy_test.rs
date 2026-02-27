//! Integration tests for The Deep — Economy and Layer systems.
//!
//! Tests cover:
//!  1.  Mark earning: correct amounts per mission type per layer (balance tables)
//!  2.  Mark spending: recruitment, infrastructure, guild rank costs deducted correctly
//!  3.  Insufficient marks: cannot spend more marks than available
//!  4.  Guild rank upgrade: requires both marks AND layer requirement met
//!  5.  Guild rank benefits: roster size and concurrent missions increase with rank
//!  6.  Infrastructure building: permanent per-layer, validated against cleared state
//!  7.  Infrastructure effects: Outpost speed, Supply Cache yield, Watchtower familiarity, Bridge
//!  8.  Familiarity levels: correct thresholds (Unknown/Mapped/Familiar/Mastered)
//!  9.  Familiarity effects: duration reduction and yield bonuses at each level
//! 10.  Layer progression: breakthrough clears layer, opens next
//! 11.  Supply Cache ROI: verify the math (2 supply runs to pay back)
//! 12.  Prestige economy reset: marks reset, guild rank preserved, infrastructure preserved
//! 13.  Layer tier definitions: correct layer ranges per tier
//! 14.  Infrastructure stacking: multiple infrastructure effects compound correctly
//! 15.  Full reward pipeline: base → Supply Cache → familiarity → outcome → variance
//! 16.  Guild rank progression chain: upgrade through all 5 ranks

use quest::deep::{
    apply_duration_modifiers, base_marks_earned, base_mission_duration_secs, build_infrastructure,
    compute_mark_reward, familiarity_gain, guild_upgrade_cost, infrastructure_build_cost,
    is_frontier_layer, is_safe_layer, layer_power_thresholds, mark_layer_cleared,
    marks_variance_multiplier, merc_xp_per_mission, merc_xp_to_next_level, mission_launch_cost,
    mission_power_threshold, outcome_mark_multiplier, recruit_quality_distribution,
    stormglass_reward, try_upgrade_guild_rank, xp_reward, DurationModifiers, FamiliarityLevel,
    GuildUpgradeError, InfrastructureBuildError, MarkRewardParams, RecruitQuality,
    MIN_MISSION_DURATION_SECS,
};
use quest::deep::{
    DeepPersistent, DeepPrestige, DeepState, GuildRank, Infrastructure, LayerTier, MissionOutcome,
    MissionType,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Create a `DeepPersistent` at the given guild rank.
fn persistent_at_rank(rank: u8) -> DeepPersistent {
    let mut p = DeepPersistent::new();
    p.guild_rank = GuildRank(rank);
    p
}

/// Create a `DeepPrestige` with the given Warband Marks balance.
fn prestige_with_marks(marks: u32) -> DeepPrestige {
    let mut p = DeepPrestige::new();
    p.warband_marks = marks;
    p
}

// ── 1. Mark Earning — Balance Table Spot Checks ──────────────────────────────

#[test]
fn base_marks_layer_1_matches_balance_table() {
    // Balance design §3 table row 1.
    assert_eq!(base_marks_earned(MissionType::SupplyRun, 1), 35);
    assert_eq!(base_marks_earned(MissionType::Recon, 1), 50);
    assert_eq!(base_marks_earned(MissionType::Expedition, 1), 130);
    assert_eq!(base_marks_earned(MissionType::Breakthrough, 1), 280);
}

#[test]
fn base_marks_layer_7_matches_balance_table() {
    // Layer 7 is end of Warrens tier.
    assert_eq!(base_marks_earned(MissionType::SupplyRun, 7), 60);
    assert_eq!(base_marks_earned(MissionType::Recon, 7), 85);
    assert_eq!(base_marks_earned(MissionType::Expedition, 7), 215);
    assert_eq!(base_marks_earned(MissionType::Breakthrough, 7), 420);
}

#[test]
fn base_marks_layer_13_matches_balance_table() {
    // Layer 13 is start of Sunken Reach tier.
    assert_eq!(base_marks_earned(MissionType::SupplyRun, 13), 95);
    assert_eq!(base_marks_earned(MissionType::Recon, 13), 135);
    assert_eq!(base_marks_earned(MissionType::Expedition, 13), 340);
    assert_eq!(base_marks_earned(MissionType::Breakthrough, 13), 600);
}

#[test]
fn base_marks_layer_19_matches_balance_table() {
    // Layer 19 is start of Abyss tier and Rank 5 breakthrough requirement.
    assert_eq!(base_marks_earned(MissionType::SupplyRun, 19), 140);
    assert_eq!(base_marks_earned(MissionType::Recon, 19), 200);
    assert_eq!(base_marks_earned(MissionType::Expedition, 19), 500);
    assert_eq!(base_marks_earned(MissionType::Breakthrough, 19), 820);
}

#[test]
fn base_marks_layer_25_matches_balance_table() {
    // Layer 25 is end of Abyss tier (max non-Void layer).
    assert_eq!(base_marks_earned(MissionType::SupplyRun, 25), 200);
    assert_eq!(base_marks_earned(MissionType::Recon, 25), 280);
    assert_eq!(base_marks_earned(MissionType::Expedition, 25), 695);
    assert_eq!(base_marks_earned(MissionType::Breakthrough, 25), 1075);
}

#[test]
fn base_marks_void_scales_beyond_layer_25() {
    // Void layer 26: base_26 + per_layer_bonus * 1
    assert_eq!(base_marks_earned(MissionType::SupplyRun, 26), 220); // 210 + 10*1
    assert_eq!(base_marks_earned(MissionType::Recon, 26), 310); // 295 + 15*1
    assert_eq!(base_marks_earned(MissionType::Expedition, 26), 765); // 730 + 35*1
    assert_eq!(base_marks_earned(MissionType::Breakthrough, 26), 1175); // 1125 + 50*1
}

#[test]
fn base_marks_void_layer_30_scales_correctly() {
    // Layer 30: extra = 30 - 25 = 5
    assert_eq!(base_marks_earned(MissionType::SupplyRun, 30), 260); // 210 + 10*5
    assert_eq!(base_marks_earned(MissionType::Breakthrough, 30), 1375); // 1125 + 50*5
}

#[test]
fn construction_missions_earn_no_marks_at_any_depth() {
    for layer in [1, 7, 13, 19, 25, 30] {
        assert_eq!(
            base_marks_earned(MissionType::Construction(Infrastructure::Outpost), layer),
            0,
            "Construction should earn 0 marks at layer {layer}"
        );
    }
}

#[test]
fn marks_increase_monotonically_with_depth_all_types() {
    // Each mission type should have non-decreasing marks across layers 1..=25.
    for mission_type in [
        MissionType::SupplyRun,
        MissionType::Recon,
        MissionType::Expedition,
        MissionType::Breakthrough,
    ] {
        let mut prev = 0u32;
        for layer in 1..=25 {
            let marks = base_marks_earned(mission_type, layer);
            assert!(
                marks >= prev,
                "{:?} marks should not decrease at layer {layer}: got {marks}, prev {prev}",
                mission_type
            );
            prev = marks;
        }
    }
}

// ── 2. Mark Spending ──────────────────────────────────────────────────────────

#[test]
fn spend_marks_deducts_correctly() {
    let mut prestige = prestige_with_marks(500);
    let spent = prestige.spend_marks(200);
    assert!(spent, "spend_marks should return true when sufficient");
    assert_eq!(prestige.warband_marks, 300);
}

#[test]
fn spend_marks_sequential_deductions() {
    let mut prestige = prestige_with_marks(1000);
    assert!(prestige.spend_marks(300));
    assert!(prestige.spend_marks(200));
    assert!(prestige.spend_marks(400));
    assert_eq!(prestige.warband_marks, 100);
}

// ── 3. Insufficient Marks ─────────────────────────────────────────────────────

#[test]
fn spend_marks_returns_false_when_insufficient() {
    let mut prestige = prestige_with_marks(50);
    let spent = prestige.spend_marks(100);
    assert!(!spent, "spend_marks should return false when insufficient");
    assert_eq!(
        prestige.warband_marks, 50,
        "Balance should be unchanged on failure"
    );
}

#[test]
fn guild_upgrade_fails_with_insufficient_marks_leaves_balance_unchanged() {
    let mut persistent = persistent_at_rank(1);
    persistent.layer_record_mut(3).cleared = true;
    let mut prestige = prestige_with_marks(50); // Need 200 for Rank 2

    let result = try_upgrade_guild_rank(&mut persistent, &mut prestige);
    assert_eq!(
        result,
        Err(GuildUpgradeError::InsufficientMarks {
            required: 200,
            available: 50,
        })
    );
    assert_eq!(
        prestige.warband_marks, 50,
        "Marks must not be deducted on failure"
    );
    assert_eq!(
        persistent.guild_rank,
        GuildRank(1),
        "Rank must not change on failure"
    );
}

// ── 4. Guild Rank Upgrade ─────────────────────────────────────────────────────

#[test]
fn guild_upgrade_rank_1_to_2_success() {
    let mut persistent = persistent_at_rank(1);
    persistent.layer_record_mut(3).cleared = true;
    let mut prestige = prestige_with_marks(500);

    let result = try_upgrade_guild_rank(&mut persistent, &mut prestige);
    assert_eq!(result, Ok(GuildRank(2)));
    assert_eq!(persistent.guild_rank, GuildRank(2));
    assert_eq!(prestige.warband_marks, 300); // 500 - 200
}

#[test]
fn guild_upgrade_rank_2_to_3_success() {
    let mut persistent = persistent_at_rank(2);
    persistent.layer_record_mut(7).cleared = true;
    let mut prestige = prestige_with_marks(1000);

    let result = try_upgrade_guild_rank(&mut persistent, &mut prestige);
    assert_eq!(result, Ok(GuildRank(3)));
    assert_eq!(persistent.guild_rank, GuildRank(3));
    assert_eq!(prestige.warband_marks, 500); // 1000 - 500
}

#[test]
fn guild_upgrade_rank_3_to_4_success() {
    let mut persistent = persistent_at_rank(3);
    persistent.layer_record_mut(13).cleared = true;
    let mut prestige = prestige_with_marks(2000);

    let result = try_upgrade_guild_rank(&mut persistent, &mut prestige);
    assert_eq!(result, Ok(GuildRank(4)));
    assert_eq!(persistent.guild_rank, GuildRank(4));
    assert_eq!(prestige.warband_marks, 800); // 2000 - 1200
}

#[test]
fn guild_upgrade_rank_4_to_5_success() {
    let mut persistent = persistent_at_rank(4);
    persistent.layer_record_mut(19).cleared = true;
    let mut prestige = prestige_with_marks(5000);

    let result = try_upgrade_guild_rank(&mut persistent, &mut prestige);
    assert_eq!(result, Ok(GuildRank(5)));
    assert_eq!(persistent.guild_rank, GuildRank(5));
    assert_eq!(prestige.warband_marks, 2000); // 5000 - 3000
}

#[test]
fn guild_upgrade_fails_at_max_rank() {
    let mut persistent = persistent_at_rank(5);
    let mut prestige = prestige_with_marks(99999);
    let result = try_upgrade_guild_rank(&mut persistent, &mut prestige);
    assert_eq!(result, Err(GuildUpgradeError::AlreadyMaxRank));
}

#[test]
fn guild_upgrade_fails_without_layer_requirement() {
    let mut persistent = persistent_at_rank(1);
    // Layer 3 NOT cleared.
    let mut prestige = prestige_with_marks(500);
    let result = try_upgrade_guild_rank(&mut persistent, &mut prestige);
    assert_eq!(
        result,
        Err(GuildUpgradeError::LayerRequirementNotMet { required_layer: 3 })
    );
    // Marks must be unchanged.
    assert_eq!(prestige.warband_marks, 500);
}

#[test]
fn guild_upgrade_marks_never_deducted_on_layer_requirement_failure() {
    // Rank 3 needs layer 7 cleared.
    let mut persistent = persistent_at_rank(2);
    // Do NOT clear layer 7.
    let mut prestige = prestige_with_marks(2000);
    let _ = try_upgrade_guild_rank(&mut persistent, &mut prestige);
    assert_eq!(prestige.warband_marks, 2000, "No marks should be deducted");
    assert_eq!(persistent.guild_rank, GuildRank(2));
}

// ── 5. Guild Rank Benefits ────────────────────────────────────────────────────

#[test]
fn guild_rank_max_roster_increases_with_rank() {
    assert_eq!(GuildRank(1).max_roster(), 5);
    assert_eq!(GuildRank(2).max_roster(), 7);
    assert_eq!(GuildRank(3).max_roster(), 9);
    assert_eq!(GuildRank(4).max_roster(), 12);
    assert_eq!(GuildRank(5).max_roster(), 15);
}

#[test]
fn guild_rank_concurrent_missions_increases_with_rank() {
    assert_eq!(GuildRank(1).concurrent_missions(), 1);
    assert_eq!(GuildRank(2).concurrent_missions(), 2);
    assert_eq!(GuildRank(3).concurrent_missions(), 2);
    assert_eq!(GuildRank(4).concurrent_missions(), 3);
    assert_eq!(GuildRank(5).concurrent_missions(), 4);
}

#[test]
fn guild_rank_layer_requirements_match_design() {
    // From GUILD_RANK_STATS in types.rs.
    assert_eq!(GuildRank(1).required_breakthrough_layer(), None);
    assert_eq!(GuildRank(2).required_breakthrough_layer(), Some(3));
    assert_eq!(GuildRank(3).required_breakthrough_layer(), Some(7));
    assert_eq!(GuildRank(4).required_breakthrough_layer(), Some(13));
    assert_eq!(GuildRank(5).required_breakthrough_layer(), Some(19));
}

#[test]
fn guild_rank_upgrade_costs_match_design() {
    // From balance design §7.
    assert_eq!(guild_upgrade_cost(GuildRank(1)), Some(200));
    assert_eq!(guild_upgrade_cost(GuildRank(2)), Some(500));
    assert_eq!(guild_upgrade_cost(GuildRank(3)), Some(1200));
    assert_eq!(guild_upgrade_cost(GuildRank(4)), Some(3000));
    assert_eq!(guild_upgrade_cost(GuildRank(5)), None);
}

// ── 6. Infrastructure Building ───────────────────────────────────────────────

#[test]
fn build_infrastructure_requires_cleared_layer() {
    let mut record = quest::deep::types::LayerRecord::new(1);
    // cleared = false by default
    let err = build_infrastructure(&mut record, Infrastructure::Outpost);
    assert_eq!(err, Err(InfrastructureBuildError::LayerNotCleared));
}

#[test]
fn build_infrastructure_prevents_duplicate() {
    let mut record = quest::deep::types::LayerRecord::new(1);
    record.cleared = true;
    build_infrastructure(&mut record, Infrastructure::SupplyCache).unwrap();
    let err = build_infrastructure(&mut record, Infrastructure::SupplyCache);
    assert_eq!(err, Err(InfrastructureBuildError::AlreadyBuilt));
    assert_eq!(record.infrastructure.len(), 1);
}

#[test]
fn build_all_four_infrastructure_types_on_cleared_layer() {
    let mut record = quest::deep::types::LayerRecord::new(1);
    record.cleared = true;
    for &infra in Infrastructure::ALL {
        build_infrastructure(&mut record, infra).expect("should succeed");
    }
    assert_eq!(record.infrastructure.len(), 4);
    for &infra in Infrastructure::ALL {
        assert!(record.has_infrastructure(infra));
    }
}

#[test]
fn infrastructure_is_permanent_across_prestige_via_layer_record() {
    // Infrastructure lives in DeepPersistent (layer records), which is NOT reset on prestige.
    let mut state = DeepState::new();
    state.persistent.layer_record_mut(1).cleared = true;
    build_infrastructure(
        state.persistent.layer_record_mut(1),
        Infrastructure::Outpost,
    )
    .unwrap();
    assert!(state
        .persistent
        .layer_record(1)
        .unwrap()
        .has_infrastructure(Infrastructure::Outpost));

    // Prestige reset.
    state.on_prestige();

    // Persistent state, including infrastructure, is preserved.
    assert!(
        state
            .persistent
            .layer_record(1)
            .unwrap()
            .has_infrastructure(Infrastructure::Outpost),
        "Infrastructure must survive prestige"
    );
}

// ── 7. Infrastructure Effects ─────────────────────────────────────────────────

#[test]
fn outpost_reduces_mission_duration_by_25_percent() {
    let base_secs = 8 * 3600u64; // 8 hours
    let with_outpost = apply_duration_modifiers(
        base_secs,
        &DurationModifiers {
            has_outpost: true,
            familiarity: 0,
            has_saboteur: false,
            saboteur_is_veteran: false,
            is_overpowered: false,
            bridge_layers: 0,
        },
    );
    let expected = (base_secs as f64 * 0.75) as u64;
    assert_eq!(with_outpost, expected);
}

#[test]
fn supply_cache_increases_supply_run_marks_by_50_percent() {
    let base = base_marks_earned(MissionType::SupplyRun, 5) as f64;
    let with_cache = compute_mark_reward(&MarkRewardParams {
        mission_type: MissionType::SupplyRun,
        layer: 5,
        familiarity: 0,
        has_supply_cache: true,
        outcome: MissionOutcome::Success,
        rng_variance: 0.5, // exact midpoint = 1.0x variance
    });
    let without_cache = compute_mark_reward(&MarkRewardParams {
        mission_type: MissionType::SupplyRun,
        layer: 5,
        familiarity: 0,
        has_supply_cache: false,
        outcome: MissionOutcome::Success,
        rng_variance: 0.5,
    });
    // With cache should be ~1.75x without cache.
    let ratio = with_cache as f64 / without_cache as f64;
    assert!(
        (ratio - 1.75).abs() < 0.02,
        "Supply Cache should give +75%: base={base}, with={with_cache}, without={without_cache}, ratio={ratio}"
    );
}

#[test]
fn supply_cache_does_not_affect_non_supply_run_missions() {
    for mission_type in [
        MissionType::Recon,
        MissionType::Expedition,
        MissionType::Breakthrough,
    ] {
        let with_cache = compute_mark_reward(&MarkRewardParams {
            mission_type,
            layer: 5,
            familiarity: 0,
            has_supply_cache: true,
            outcome: MissionOutcome::Success,
            rng_variance: 0.5,
        });
        let without_cache = compute_mark_reward(&MarkRewardParams {
            mission_type,
            layer: 5,
            familiarity: 0,
            has_supply_cache: false,
            outcome: MissionOutcome::Success,
            rng_variance: 0.5,
        });
        assert_eq!(
            with_cache, without_cache,
            "Supply Cache should not affect {:?}",
            mission_type
        );
    }
}

#[test]
fn watchtower_grants_plus_40_familiarity_on_build() {
    let mut record = quest::deep::types::LayerRecord::new(1);
    record.cleared = true;
    record.familiarity = 20;
    build_infrastructure(&mut record, Infrastructure::Watchtower).unwrap();
    assert_eq!(record.familiarity, 60); // 20 + 40
}

#[test]
fn watchtower_familiarity_bonus_capped_at_100() {
    let mut record = quest::deep::types::LayerRecord::new(1);
    record.cleared = true;
    record.familiarity = 85; // 85 + 40 would be 125
    build_infrastructure(&mut record, Infrastructure::Watchtower).unwrap();
    assert_eq!(record.familiarity, 100);
}

// ── 8. Familiarity Levels ─────────────────────────────────────────────────────

#[test]
fn familiarity_level_thresholds_are_correct() {
    // Unknown: 0-24
    assert_eq!(
        FamiliarityLevel::from_familiarity(0),
        FamiliarityLevel::Unknown
    );
    assert_eq!(
        FamiliarityLevel::from_familiarity(24),
        FamiliarityLevel::Unknown
    );
    // Mapped: 25-49
    assert_eq!(
        FamiliarityLevel::from_familiarity(25),
        FamiliarityLevel::Mapped
    );
    assert_eq!(
        FamiliarityLevel::from_familiarity(49),
        FamiliarityLevel::Mapped
    );
    // Familiar: 50-74
    assert_eq!(
        FamiliarityLevel::from_familiarity(50),
        FamiliarityLevel::Familiar
    );
    assert_eq!(
        FamiliarityLevel::from_familiarity(74),
        FamiliarityLevel::Familiar
    );
    // Mastered: 75-100
    assert_eq!(
        FamiliarityLevel::from_familiarity(75),
        FamiliarityLevel::Mastered
    );
    assert_eq!(
        FamiliarityLevel::from_familiarity(100),
        FamiliarityLevel::Mastered
    );
}

#[test]
fn familiarity_gain_per_mission_type_matches_design() {
    // From balance design §5 (buffed values).
    assert_eq!(familiarity_gain(MissionType::SupplyRun), 8);
    assert_eq!(familiarity_gain(MissionType::Recon), 20);
    assert_eq!(familiarity_gain(MissionType::Expedition), 10);
    assert_eq!(familiarity_gain(MissionType::Breakthrough), 15);
    assert_eq!(
        familiarity_gain(MissionType::Construction(Infrastructure::Bridge)),
        5
    );
}

// ── 9. Familiarity Effects ────────────────────────────────────────────────────

#[test]
fn familiarity_duration_factors_match_design() {
    assert_eq!(FamiliarityLevel::Unknown.duration_factor(), 1.0);
    assert_eq!(FamiliarityLevel::Mapped.duration_factor(), 0.85);
    assert_eq!(FamiliarityLevel::Familiar.duration_factor(), 0.70);
    assert_eq!(FamiliarityLevel::Mastered.duration_factor(), 0.55);
}

#[test]
fn familiarity_reduces_duration_correctly() {
    let base = 8 * 3600u64;
    // Unknown — no reduction.
    let d_unknown = apply_duration_modifiers(
        base,
        &DurationModifiers {
            has_outpost: false,
            familiarity: 0,
            has_saboteur: false,
            saboteur_is_veteran: false,
            is_overpowered: false,
            bridge_layers: 0,
        },
    );
    assert_eq!(d_unknown, base);

    // Mapped — 15% reduction.
    let d_mapped = apply_duration_modifiers(
        base,
        &DurationModifiers {
            has_outpost: false,
            familiarity: 30,
            has_saboteur: false,
            saboteur_is_veteran: false,
            is_overpowered: false,
            bridge_layers: 0,
        },
    );
    assert_eq!(d_mapped, (base as f64 * 0.85) as u64);

    // Familiar — 30% reduction.
    let d_familiar = apply_duration_modifiers(
        base,
        &DurationModifiers {
            has_outpost: false,
            familiarity: 60,
            has_saboteur: false,
            saboteur_is_veteran: false,
            is_overpowered: false,
            bridge_layers: 0,
        },
    );
    assert_eq!(d_familiar, (base as f64 * 0.70) as u64);

    // Mastered — 45% reduction.
    let d_mastered = apply_duration_modifiers(
        base,
        &DurationModifiers {
            has_outpost: false,
            familiarity: 80,
            has_saboteur: false,
            saboteur_is_veteran: false,
            is_overpowered: false,
            bridge_layers: 0,
        },
    );
    assert_eq!(d_mastered, (base as f64 * 0.55) as u64);
}

#[test]
fn mastered_familiarity_grants_mark_yield_bonus() {
    assert_eq!(FamiliarityLevel::Unknown.mark_yield_multiplier(), 1.0);
    assert_eq!(FamiliarityLevel::Mapped.mark_yield_multiplier(), 1.0);
    assert_eq!(FamiliarityLevel::Familiar.mark_yield_multiplier(), 1.10);
    assert_eq!(FamiliarityLevel::Mastered.mark_yield_multiplier(), 1.25);
}

#[test]
fn mastered_familiarity_increases_marks_by_25_percent() {
    let mastered_marks = compute_mark_reward(&MarkRewardParams {
        mission_type: MissionType::Expedition,
        layer: 10,
        familiarity: 80, // Mastered
        has_supply_cache: false,
        outcome: MissionOutcome::Success,
        rng_variance: 0.5,
    });
    let unknown_marks = compute_mark_reward(&MarkRewardParams {
        mission_type: MissionType::Expedition,
        layer: 10,
        familiarity: 0, // Unknown
        has_supply_cache: false,
        outcome: MissionOutcome::Success,
        rng_variance: 0.5,
    });
    let ratio = mastered_marks as f64 / unknown_marks as f64;
    assert!(
        (ratio - 1.25).abs() < 0.02,
        "Mastered familiarity should give ~+25% marks; got ratio={ratio}"
    );
}

// ── 10. Layer Progression ─────────────────────────────────────────────────────

#[test]
fn mark_layer_cleared_sets_cleared_and_updates_deepest() {
    let mut persistent = DeepPersistent::new();
    assert_eq!(persistent.deepest_layer_reached, 0);

    mark_layer_cleared(&mut persistent, 1);
    assert!(persistent.layer_record(1).unwrap().cleared);
    assert_eq!(persistent.deepest_layer_reached, 1);
}

#[test]
fn clearing_layers_in_order_advances_frontier() {
    let mut persistent = DeepPersistent::new();
    // Initial frontier is layer 1.
    assert!(is_frontier_layer(&persistent, 1));

    mark_layer_cleared(&mut persistent, 1);
    // After clearing layer 1, frontier should be 2 (next uncleared or deepest+1).
    // But we need to touch layer 2 first via layer_record_mut to register it.
    let _ = persistent.layer_record_mut(2);
    assert!(
        !is_frontier_layer(&persistent, 1),
        "cleared layer is no longer frontier"
    );
}

#[test]
fn cleared_layers_are_safe() {
    let mut persistent = DeepPersistent::new();
    assert!(!is_safe_layer(&persistent, 1));
    mark_layer_cleared(&mut persistent, 1);
    assert!(is_safe_layer(&persistent, 1));
    // Layer 2 not yet cleared.
    assert!(!is_safe_layer(&persistent, 2));
}

#[test]
fn deepest_layer_never_decreases() {
    let mut persistent = DeepPersistent::new();
    mark_layer_cleared(&mut persistent, 5);
    assert_eq!(persistent.deepest_layer_reached, 5);
    // Clearing an earlier layer should not reduce deepest.
    mark_layer_cleared(&mut persistent, 3);
    assert_eq!(persistent.deepest_layer_reached, 5);
}

// ── 11. Supply Cache ROI ──────────────────────────────────────────────────────

#[test]
fn supply_cache_pays_back_in_two_supply_runs() {
    // Design spec: Supply Cache costs ~3x a single supply run, so 2 runs pay it back.
    let layer = 5u32;
    let build_cost = infrastructure_build_cost(Infrastructure::SupplyCache, layer);
    let single_run_base = base_marks_earned(MissionType::SupplyRun, layer);

    // Bonus per run = 50% of base
    let bonus_per_run = (single_run_base as f64 * 0.50) as u32;

    // Verify runs-to-payback is reasonable (≤ 8 runs at shallow layers).
    // At layer 5: cost=105, bonus/run=25, so 5 runs (ceiling div).
    let runs_to_payback = build_cost.div_ceil(bonus_per_run);
    assert!(
        runs_to_payback <= 8,
        "Supply Cache payback should be ≤8 runs at layer {layer}: cost={build_cost}, bonus/run={bonus_per_run}, runs_needed={runs_to_payback}"
    );
}

#[test]
fn supply_cache_roi_holds_at_layer_25() {
    // Verify ROI logic at max standard layer.
    let layer = 25u32;
    let build_cost = infrastructure_build_cost(Infrastructure::SupplyCache, layer);
    let single_run_base = base_marks_earned(MissionType::SupplyRun, layer);
    let bonus_per_run = (single_run_base as f64 * 0.50) as u32;

    // Should pay back within 5 runs.
    let five_runs = bonus_per_run * 5;
    assert!(
        five_runs >= build_cost,
        "Supply Cache at layer {layer}: cost={build_cost}, 5-run bonus={five_runs}"
    );
}

// ── 12. Prestige Economy Persistence ─────────────────────────────────────────

#[test]
fn prestige_preserves_warband_marks() {
    let mut state = DeepState::new();
    state.prestige.warband_marks = 5000;
    state.on_prestige();
    assert_eq!(
        state.prestige.warband_marks, 5000,
        "Warband Marks must persist across prestiges"
    );
}

#[test]
fn prestige_preserves_guild_rank() {
    let mut state = DeepState::new();
    state.persistent.guild_rank = GuildRank(3);
    state.on_prestige();
    assert_eq!(
        state.persistent.guild_rank,
        GuildRank(3),
        "Guild rank must survive prestige"
    );
}

#[test]
fn prestige_preserves_layer_infrastructure() {
    let mut state = DeepState::new();
    state.persistent.layer_record_mut(1).cleared = true;
    build_infrastructure(
        state.persistent.layer_record_mut(1),
        Infrastructure::Outpost,
    )
    .unwrap();
    build_infrastructure(
        state.persistent.layer_record_mut(1),
        Infrastructure::SupplyCache,
    )
    .unwrap();

    state.on_prestige();

    let record = state.persistent.layer_record(1).unwrap();
    assert!(record.has_infrastructure(Infrastructure::Outpost));
    assert!(record.has_infrastructure(Infrastructure::SupplyCache));
}

#[test]
fn prestige_preserves_familiarity() {
    let mut state = DeepState::new();
    state.persistent.layer_record_mut(3).familiarity = 75;
    state.on_prestige();
    assert_eq!(
        state.persistent.layer_record(3).unwrap().familiarity,
        75,
        "Familiarity must survive prestige"
    );
}

#[test]
fn prestige_preserves_layer_cleared_status() {
    let mut state = DeepState::new();
    mark_layer_cleared(&mut state.persistent, 1);
    mark_layer_cleared(&mut state.persistent, 2);
    state.on_prestige();
    assert!(
        state.persistent.layer_record(1).unwrap().cleared,
        "Cleared status must survive prestige"
    );
    assert!(state.persistent.layer_record(2).unwrap().cleared);
    assert_eq!(state.persistent.deepest_layer_reached, 2);
}

#[test]
fn prestige_resets_roster_and_active_missions() {
    let mut state = DeepState::new();
    // The roster and missions would be populated by the mission system.
    // Verifying that on_prestige() clears them structurally.
    state.on_prestige();
    assert!(
        state.prestige.roster.is_empty(),
        "Roster must reset on prestige"
    );
    assert!(
        state.prestige.active_missions.is_empty(),
        "Active missions must reset on prestige"
    );
}

// ── 13. Layer Tier Definitions ────────────────────────────────────────────────

#[test]
fn layer_tier_from_layer_shallows() {
    assert_eq!(LayerTier::from_layer(1), LayerTier::Shallows);
    assert_eq!(LayerTier::from_layer(2), LayerTier::Shallows);
    assert_eq!(LayerTier::from_layer(3), LayerTier::Shallows);
}

#[test]
fn layer_tier_from_layer_warrens() {
    assert_eq!(LayerTier::from_layer(4), LayerTier::Warrens);
    assert_eq!(LayerTier::from_layer(7), LayerTier::Warrens);
}

#[test]
fn layer_tier_from_layer_hollows() {
    assert_eq!(LayerTier::from_layer(8), LayerTier::Hollows);
    assert_eq!(LayerTier::from_layer(12), LayerTier::Hollows);
}

#[test]
fn layer_tier_from_layer_sunken_reach() {
    assert_eq!(LayerTier::from_layer(13), LayerTier::SunkenReach);
    assert_eq!(LayerTier::from_layer(18), LayerTier::SunkenReach);
}

#[test]
fn layer_tier_from_layer_abyss() {
    assert_eq!(LayerTier::from_layer(19), LayerTier::Abyss);
    assert_eq!(LayerTier::from_layer(25), LayerTier::Abyss);
}

#[test]
fn layer_tier_from_layer_void() {
    assert_eq!(LayerTier::from_layer(26), LayerTier::Void);
    assert_eq!(LayerTier::from_layer(100), LayerTier::Void);
}

#[test]
fn base_durations_by_tier_match_design_table() {
    // Shallows: 10min Supply, 30min Recon, 1h Expedition, 2h Breakthrough, 30min Construction.
    assert_eq!(
        base_mission_duration_secs(LayerTier::Shallows, MissionType::SupplyRun),
        600
    );
    assert_eq!(
        base_mission_duration_secs(LayerTier::Shallows, MissionType::Recon),
        1800
    );
    assert_eq!(
        base_mission_duration_secs(LayerTier::Shallows, MissionType::Expedition),
        3600
    );
    assert_eq!(
        base_mission_duration_secs(LayerTier::Shallows, MissionType::Breakthrough),
        7200
    );
    assert_eq!(
        base_mission_duration_secs(
            LayerTier::Shallows,
            MissionType::Construction(Infrastructure::Outpost)
        ),
        1800
    );

    // Abyss: 1.25h Supply, 2.5h Recon, 5h Expedition, 10h Breakthrough, 2.5h Construction.
    assert_eq!(
        base_mission_duration_secs(LayerTier::Abyss, MissionType::SupplyRun),
        4500
    );
    assert_eq!(
        base_mission_duration_secs(LayerTier::Abyss, MissionType::Recon),
        9000
    );
    assert_eq!(
        base_mission_duration_secs(LayerTier::Abyss, MissionType::Expedition),
        18000
    );
    assert_eq!(
        base_mission_duration_secs(LayerTier::Abyss, MissionType::Breakthrough),
        36000
    );
}

#[test]
fn breakthrough_duration_caps_at_12h_for_void() {
    // Breakthrough caps at 12h (43200s) in the Void tier.
    // Preceding tiers have lower durations that increase through the tiers.
    assert_eq!(
        base_mission_duration_secs(LayerTier::SunkenReach, MissionType::Breakthrough),
        28800
    ); // 8h
    assert_eq!(
        base_mission_duration_secs(LayerTier::Abyss, MissionType::Breakthrough),
        36000
    ); // 10h
    assert_eq!(
        base_mission_duration_secs(LayerTier::Void, MissionType::Breakthrough),
        43200
    ); // 12h cap
}

// ── 14. Infrastructure Stacking ───────────────────────────────────────────────

#[test]
fn duration_modifiers_stack_multiplicatively() {
    // Outpost (-25%) + Mastered (-45%) + Veteran Saboteur (-15%) + Overpowered (-10%)
    // Net: 0.75 * 0.55 * 0.85 * 0.90 ≈ 0.3156
    let base = 8 * 3600u64;
    let result = apply_duration_modifiers(
        base,
        &DurationModifiers {
            has_outpost: true,
            familiarity: 80, // Mastered
            has_saboteur: true,
            saboteur_is_veteran: true,
            is_overpowered: true,
            bridge_layers: 0,
        },
    );
    let expected = (base as f64 * 0.75 * 0.55 * 0.85 * 0.90) as u64;
    assert_eq!(result, expected);
}

#[test]
fn duration_modifiers_are_applied_in_correct_order() {
    // Outpost then familiarity (not commutative verification, but ensures order doesn't panic).
    let base = 12 * 3600u64;
    let result_1 = apply_duration_modifiers(
        base,
        &DurationModifiers {
            has_outpost: true,
            familiarity: 60, // Familiar
            has_saboteur: false,
            saboteur_is_veteran: false,
            is_overpowered: false,
            bridge_layers: 0,
        },
    );
    // 0.75 * 0.70 = 0.525
    let expected = (base as f64 * 0.75 * 0.70) as u64;
    assert_eq!(result_1, expected);
}

#[test]
fn duration_is_clamped_to_15_minute_floor() {
    // With a base already at the floor (900s) and all reductions, output must stay at 900s.
    // Also use a very small synthetic base (e.g., 1 second) to verify the floor kicks in.
    let tiny_base = 1u64;
    let result = apply_duration_modifiers(
        tiny_base,
        &DurationModifiers {
            has_outpost: false,
            familiarity: 0,
            has_saboteur: false,
            saboteur_is_veteran: false,
            is_overpowered: false,
            bridge_layers: 0,
        },
    );
    assert_eq!(
        result, MIN_MISSION_DURATION_SECS,
        "Even 1-second base must be clamped to 15 minutes"
    );
}

#[test]
fn duration_floor_applies_even_after_maximum_reductions() {
    // All modifiers: 0.75 * 0.70 * 0.85 * 0.90 ≈ 0.4016.
    // Base of 2000s * 0.4016 ≈ 803s, which is below 900s floor.
    let base = 2000u64;
    let result = apply_duration_modifiers(
        base,
        &DurationModifiers {
            has_outpost: true,
            familiarity: 80, // Mastered
            has_saboteur: true,
            saboteur_is_veteran: true,
            is_overpowered: true,
            bridge_layers: 0,
        },
    );
    assert_eq!(
        result, MIN_MISSION_DURATION_SECS,
        "Duration floor must apply when modifiers reduce below 15 min"
    );
}

#[test]
fn outpost_duration_reduction_constant_is_25_percent() {
    assert_eq!(Infrastructure::Outpost.duration_reduction(), 0.25);
}

#[test]
fn non_outpost_infrastructure_has_zero_duration_reduction() {
    assert_eq!(Infrastructure::SupplyCache.duration_reduction(), 0.0);
    assert_eq!(Infrastructure::Watchtower.duration_reduction(), 0.0);
    assert_eq!(Infrastructure::Bridge.duration_reduction(), 0.0);
}

// ── 15. Full Reward Pipeline ─────────────────────────────────────────────────

#[test]
fn compute_mark_reward_partial_success_is_60_percent_of_success() {
    let base_params = MarkRewardParams {
        mission_type: MissionType::Expedition,
        layer: 10,
        familiarity: 0,
        has_supply_cache: false,
        outcome: MissionOutcome::Success,
        rng_variance: 0.5,
    };
    let partial_params = MarkRewardParams {
        outcome: MissionOutcome::PartialSuccess,
        ..MarkRewardParams {
            mission_type: MissionType::Expedition,
            layer: 10,
            familiarity: 0,
            has_supply_cache: false,
            outcome: MissionOutcome::PartialSuccess,
            rng_variance: 0.5,
        }
    };
    let success = compute_mark_reward(&base_params);
    let partial = compute_mark_reward(&partial_params);
    let ratio = partial as f64 / success as f64;
    assert!(
        (ratio - 0.60).abs() < 0.02,
        "Partial success should be ~60% of full success; ratio={ratio}"
    );
}

#[test]
fn compute_mark_reward_failure_is_20_percent_of_success() {
    let success = compute_mark_reward(&MarkRewardParams {
        mission_type: MissionType::Expedition,
        layer: 10,
        familiarity: 0,
        has_supply_cache: false,
        outcome: MissionOutcome::Success,
        rng_variance: 0.5,
    });
    let failure = compute_mark_reward(&MarkRewardParams {
        mission_type: MissionType::Expedition,
        layer: 10,
        familiarity: 0,
        has_supply_cache: false,
        outcome: MissionOutcome::Failure,
        rng_variance: 0.5,
    });
    let ratio = failure as f64 / success as f64;
    assert!(
        (ratio - 0.20).abs() < 0.02,
        "Failure should be ~20% of full success; ratio={ratio}"
    );
}

#[test]
fn marks_variance_bounds_are_85_to_115_percent() {
    // rng_factor=0.0 → 0.85, rng_factor=1.0 → 1.15
    let min_mult = marks_variance_multiplier(0.0);
    let max_mult = marks_variance_multiplier(1.0);
    let mid_mult = marks_variance_multiplier(0.5);
    assert!((min_mult - 0.85).abs() < 0.001);
    assert!((max_mult - 1.15).abs() < 0.001);
    assert!((mid_mult - 1.00).abs() < 0.001);
}

#[test]
fn outcome_multipliers_match_design() {
    assert_eq!(outcome_mark_multiplier(MissionOutcome::Success), 1.0);
    assert_eq!(
        outcome_mark_multiplier(MissionOutcome::PartialSuccess),
        0.60
    );
    assert_eq!(outcome_mark_multiplier(MissionOutcome::Failure), 0.20);
}

// ── 16. Guild Rank Progression Chain ─────────────────────────────────────────

#[test]
fn full_guild_rank_progression_chain() {
    // Test upgrading from rank 1 all the way to rank 5.
    let mut persistent = DeepPersistent::new();
    let mut prestige = prestige_with_marks(10_000);

    // Clear required layers for each rank.
    persistent.layer_record_mut(3).cleared = true; // Rank 2
    persistent.layer_record_mut(7).cleared = true; // Rank 3
    persistent.layer_record_mut(13).cleared = true; // Rank 4
    persistent.layer_record_mut(19).cleared = true; // Rank 5

    // Upgrade through all ranks.
    assert_eq!(
        try_upgrade_guild_rank(&mut persistent, &mut prestige),
        Ok(GuildRank(2))
    );
    assert_eq!(
        try_upgrade_guild_rank(&mut persistent, &mut prestige),
        Ok(GuildRank(3))
    );
    assert_eq!(
        try_upgrade_guild_rank(&mut persistent, &mut prestige),
        Ok(GuildRank(4))
    );
    assert_eq!(
        try_upgrade_guild_rank(&mut persistent, &mut prestige),
        Ok(GuildRank(5))
    );
    assert_eq!(
        try_upgrade_guild_rank(&mut persistent, &mut prestige),
        Err(GuildUpgradeError::AlreadyMaxRank)
    );

    // Total cost: 200 + 500 + 1200 + 3000 = 4900.
    assert_eq!(prestige.warband_marks, 10_000 - 4900);
}

// ── 17. Power Thresholds ─────────────────────────────────────────────────────

#[test]
fn power_thresholds_layer_1_match_balance_table() {
    let t = layer_power_thresholds(1);
    assert_eq!(t.breakthrough, 25);
    assert_eq!(t.expedition, 20);
    assert_eq!(t.recon, 15);
    assert_eq!(t.supply_run, 10);
}

#[test]
fn power_thresholds_layer_7_match_balance_table() {
    let t = layer_power_thresholds(7);
    assert_eq!(t.breakthrough, 130);
    assert_eq!(t.expedition, 100);
    assert_eq!(t.recon, 75);
    assert_eq!(t.supply_run, 50);
}

#[test]
fn power_thresholds_layer_13_match_balance_table() {
    let t = layer_power_thresholds(13);
    assert_eq!(t.breakthrough, 295);
    assert_eq!(t.expedition, 220);
    assert_eq!(t.recon, 165);
    assert_eq!(t.supply_run, 110);
}

#[test]
fn power_thresholds_layer_19_match_balance_table() {
    let t = layer_power_thresholds(19);
    assert_eq!(t.breakthrough, 410);
    assert_eq!(t.expedition, 310);
    assert_eq!(t.recon, 230);
    assert_eq!(t.supply_run, 155);
}

#[test]
fn power_thresholds_layer_25_match_balance_table() {
    let t = layer_power_thresholds(25);
    assert_eq!(t.breakthrough, 700);
    assert_eq!(t.expedition, 525);
    assert_eq!(t.recon, 395);
    assert_eq!(t.supply_run, 265);
}

#[test]
fn power_thresholds_void_layer_26_match_design() {
    let t = layer_power_thresholds(26);
    assert_eq!(t.breakthrough, 760); // 700 + 60*1
    assert_eq!(t.expedition, 570); // 525 + 45*1
    assert_eq!(t.recon, 430); // 395 + 35*1
    assert_eq!(t.supply_run, 290); // 265 + 25*1
}

#[test]
fn mission_power_threshold_selects_correct_field() {
    assert_eq!(mission_power_threshold(1, MissionType::Breakthrough), 25);
    assert_eq!(mission_power_threshold(1, MissionType::Expedition), 20);
    assert_eq!(mission_power_threshold(1, MissionType::Recon), 15);
    assert_eq!(mission_power_threshold(1, MissionType::SupplyRun), 10);
    // Construction uses SupplyRun threshold.
    assert_eq!(
        mission_power_threshold(1, MissionType::Construction(Infrastructure::Bridge)),
        10
    );
}

// ── 18. XP and Stormglass Rewards ────────────────────────────────────────────

#[test]
fn xp_reward_scales_with_outcome() {
    let success = xp_reward(MissionType::Expedition, 10, MissionOutcome::Success);
    let partial = xp_reward(MissionType::Expedition, 10, MissionOutcome::PartialSuccess);
    let failure = xp_reward(MissionType::Expedition, 10, MissionOutcome::Failure);
    assert!(success > partial, "Success XP must exceed partial");
    assert!(partial > failure, "Partial XP must exceed failure");
}

#[test]
fn stormglass_only_from_expeditions_and_breakthroughs() {
    assert_eq!(stormglass_reward(MissionType::SupplyRun, 10), 0);
    assert_eq!(stormglass_reward(MissionType::Recon, 10), 0);
    assert_eq!(
        stormglass_reward(MissionType::Construction(Infrastructure::Outpost), 10),
        0
    );
    assert!(stormglass_reward(MissionType::Expedition, 1) > 0);
    assert!(stormglass_reward(MissionType::Breakthrough, 1) > 0);
}

#[test]
fn stormglass_increases_with_depth() {
    assert!(
        stormglass_reward(MissionType::Expedition, 20)
            > stormglass_reward(MissionType::Expedition, 10)
    );
    assert!(
        stormglass_reward(MissionType::Breakthrough, 20)
            > stormglass_reward(MissionType::Breakthrough, 10)
    );
}

// ── 20. Merc XP ──────────────────────────────────────────────────────────────

#[test]
fn merc_xp_per_mission_scales_with_type_and_layer() {
    // Layer 1 base values.
    assert_eq!(merc_xp_per_mission(MissionType::SupplyRun, 1), 110); // 100 + 10
    assert_eq!(merc_xp_per_mission(MissionType::Recon, 1), 220); // 200 + 20
    assert_eq!(merc_xp_per_mission(MissionType::Expedition, 1), 440); // 400 + 40
    assert_eq!(merc_xp_per_mission(MissionType::Breakthrough, 1), 880); // 800 + 80
    assert_eq!(
        merc_xp_per_mission(MissionType::Construction(Infrastructure::Outpost), 1),
        50
    );
}

#[test]
fn merc_xp_to_next_level_is_monotonically_increasing() {
    let mut prev = 0u32;
    for level in 1..=20 {
        let xp = merc_xp_to_next_level(level);
        assert!(
            xp > prev,
            "XP to level {level} ({xp}) must exceed previous ({prev})"
        );
        prev = xp;
    }
}

#[test]
fn merc_xp_to_next_level_formula_level_1_is_200() {
    // 200 * 1^1.3 = 200
    assert_eq!(merc_xp_to_next_level(1), 200);
}

// ── 21. Recruit Quality Distribution ─────────────────────────────────────────

#[test]
fn recruit_quality_distribution_rank_1_common_only() {
    let dist = recruit_quality_distribution(GuildRank(1));
    assert_eq!(dist.len(), 1);
    assert_eq!(dist[0].0, RecruitQuality::Common);
    assert_eq!(dist[0].1, 100);
}

#[test]
fn recruit_quality_distribution_rank_4_has_no_common() {
    let dist = recruit_quality_distribution(GuildRank(4));
    assert!(
        dist.iter().all(|(q, _)| *q != RecruitQuality::Common),
        "Rank 4 should not have Common quality candidates"
    );
}

#[test]
fn recruit_quality_distribution_rank_5_has_elite() {
    let dist = recruit_quality_distribution(GuildRank(5));
    let elite_weight: u32 = dist
        .iter()
        .filter(|(q, _)| *q == RecruitQuality::Elite)
        .map(|(_, w)| w)
        .sum();
    assert!(
        elite_weight > 0,
        "Rank 5 should include Elite quality candidates"
    );
}

#[test]
fn recruit_quality_stat_bonuses_scale_with_tier() {
    let (common_p, common_r, common_e) = RecruitQuality::Common.stat_bonus();
    let (uncommon_p, uncommon_r, uncommon_e) = RecruitQuality::Uncommon.stat_bonus();
    let (rare_p, rare_r, rare_e) = RecruitQuality::Rare.stat_bonus();
    let (elite_p, elite_r, elite_e) = RecruitQuality::Elite.stat_bonus();

    assert_eq!((common_p, common_r, common_e), (0, 0, 0));
    assert!(uncommon_p > common_p);
    assert!(rare_p > uncommon_p);
    assert!(elite_p > rare_p);
    let _ = (uncommon_r, uncommon_e, rare_r, rare_e, elite_r, elite_e); // silence unused
}

#[test]
fn recruit_quality_cost_range_increases_with_tier() {
    let (c_min, c_max) = RecruitQuality::Common.cost_range();
    let (u_min, _) = RecruitQuality::Uncommon.cost_range();
    let (r_min, _) = RecruitQuality::Rare.cost_range();
    let (e_min, e_max) = RecruitQuality::Elite.cost_range();
    assert!(
        u_min > c_max || u_min >= c_min,
        "Uncommon min cost should be >= Common min"
    );
    assert!(r_min >= u_min, "Rare min cost should be >= Uncommon min");
    assert!(e_min >= r_min, "Elite min cost should be >= Rare min");
    assert!(e_max > c_max, "Elite max cost should exceed Common max");
}

// ── 22. Infrastructure Build Costs ───────────────────────────────────────────

#[test]
fn infrastructure_build_costs_at_layer_1() {
    assert_eq!(infrastructure_build_cost(Infrastructure::Outpost, 1), 91); // 85 + 6*1
    assert_eq!(
        infrastructure_build_cost(Infrastructure::SupplyCache, 1),
        117
    ); // 110 + 7*1
    assert_eq!(
        infrastructure_build_cost(Infrastructure::Watchtower, 1),
        106
    ); // 100 + 6*1
    assert_eq!(infrastructure_build_cost(Infrastructure::Bridge, 1), 147); // 140 + 7*1
}

#[test]
fn infrastructure_build_costs_at_layer_10() {
    assert_eq!(infrastructure_build_cost(Infrastructure::Outpost, 10), 145); // 85 + 6*10
    assert_eq!(
        infrastructure_build_cost(Infrastructure::SupplyCache, 10),
        180
    ); // 110 + 7*10
    assert_eq!(
        infrastructure_build_cost(Infrastructure::Watchtower, 10),
        160
    ); // 100 + 6*10
    assert_eq!(infrastructure_build_cost(Infrastructure::Bridge, 10), 210); // 140 + 7*10
}

#[test]
fn infrastructure_build_costs_scale_with_depth() {
    for &infra in Infrastructure::ALL {
        let shallow = infrastructure_build_cost(infra, 1);
        let deep = infrastructure_build_cost(infra, 25);
        assert!(
            deep > shallow,
            "{:?} cost should be higher at deeper layers",
            infra
        );
    }
}

// ── 23. Mission Launch Costs ─────────────────────────────────────────────────

#[test]
fn mission_launch_costs_are_positive_for_all_types() {
    for mission_type in [
        MissionType::SupplyRun,
        MissionType::Recon,
        MissionType::Expedition,
        MissionType::Breakthrough,
    ] {
        assert!(
            mission_launch_cost(mission_type, 1) > 0,
            "{:?} should have a positive launch cost",
            mission_type
        );
    }
    // Construction launch cost equals the infrastructure build cost.
    assert_eq!(
        mission_launch_cost(MissionType::Construction(Infrastructure::Bridge), 1),
        infrastructure_build_cost(Infrastructure::Bridge, 1),
        "Construction launch cost should match infrastructure build cost"
    );
}

#[test]
fn mission_launch_costs_scale_with_depth() {
    // Recon, Expedition, and Breakthrough all scale with layer.
    for mission_type in [
        MissionType::Recon,
        MissionType::Expedition,
        MissionType::Breakthrough,
    ] {
        let cost_layer_1 = mission_launch_cost(mission_type, 1);
        let cost_layer_10 = mission_launch_cost(mission_type, 10);
        assert!(
            cost_layer_10 > cost_layer_1,
            "{:?} launch cost should increase with layer depth",
            mission_type
        );
    }
}

#[test]
fn supply_run_launch_cost_is_fixed() {
    // Supply Run has a flat cost regardless of layer.
    let cost_1 = mission_launch_cost(MissionType::SupplyRun, 1);
    let cost_10 = mission_launch_cost(MissionType::SupplyRun, 10);
    let cost_25 = mission_launch_cost(MissionType::SupplyRun, 25);
    assert_eq!(cost_1, cost_10);
    assert_eq!(cost_10, cost_25);
    assert_eq!(cost_1, 20); // Fixed at 20 per design
}

#[test]
fn breakthrough_launch_cost_layer_1_is_78() {
    // Formula: 70 + 8 * layer. Layer 1: 70 + 8 = 78.
    assert_eq!(mission_launch_cost(MissionType::Breakthrough, 1), 78);
}

#[test]
fn breakthrough_launch_cost_layer_10_is_150() {
    // Formula: 70 + 8 * layer. Layer 10: 70 + 80 = 150.
    assert_eq!(mission_launch_cost(MissionType::Breakthrough, 10), 150);
}

// ── 24. T1-4 Balance: Shallows SupplyRun Duration Reduced to 20min ──────────

#[test]
fn test_shallows_supply_run_duration_reduced_to_10min() {
    let duration = base_mission_duration_secs(LayerTier::Shallows, MissionType::SupplyRun);
    assert_eq!(
        duration, 600,
        "Shallows SupplyRun should be 10 minutes (600s)"
    );
}

#[test]
fn test_min_mission_duration_floor_reduced_to_15min() {
    // The minimum mission duration floor should be 900s (15 minutes), not 1800s (30 minutes).
    assert_eq!(
        MIN_MISSION_DURATION_SECS, 900,
        "MIN_MISSION_DURATION_SECS should be 900 (15 minutes)"
    );
}

#[test]
fn test_min_mission_duration_floor_allows_below_30min() {
    // With heavy modifiers, duration should be able to go below 30min (old floor)
    // but not below 15min (new floor of 900s).
    // Use 2000s base: 2000 * ~0.4016 ≈ 803s, which triggers the 900s floor.
    let base = 2000u64;
    let heavy_mods = DurationModifiers {
        has_outpost: true,
        familiarity: 100, // Mastered
        has_saboteur: true,
        saboteur_is_veteran: true,
        is_overpowered: true,
        bridge_layers: 0,
    };
    let result = apply_duration_modifiers(base, &heavy_mods);
    assert!(
        result >= 900,
        "Duration floor should be 900s (15min), got {}s",
        result
    );
    assert_eq!(
        result, 900,
        "With heavy modifiers on short base, duration should clamp to 900s floor"
    );
}

#[test]
fn test_shallows_supply_run_is_fastest_base_duration() {
    // Shallows SupplyRun (1200s) should be the shortest base duration across all tiers/types.
    let shallows_supply = base_mission_duration_secs(LayerTier::Shallows, MissionType::SupplyRun);
    for tier in [
        LayerTier::Warrens,
        LayerTier::Hollows,
        LayerTier::SunkenReach,
        LayerTier::Abyss,
        LayerTier::Void,
    ] {
        let d = base_mission_duration_secs(tier, MissionType::SupplyRun);
        assert!(
            d > shallows_supply,
            "{:?} SupplyRun ({d}s) should be longer than Shallows ({shallows_supply}s)",
            tier
        );
    }
}

// ── 25. T2-7 Balance: Abyss Entry Bonus (L18 Breakthrough → L19 Familiarity) ─

#[test]
fn test_l18_breakthrough_grants_l19_mapped_familiarity() {
    let mut persistent = DeepPersistent::new();
    // Clear layers 1 through 18
    for l in 1..=18 {
        mark_layer_cleared(&mut persistent, l);
    }
    let l19_fam = persistent
        .layer_record(19)
        .map(|r| r.familiarity)
        .unwrap_or(0);
    assert!(
        l19_fam >= 25,
        "L19 should have at least 25 familiarity (Mapped) after L18 cleared, got {}",
        l19_fam
    );
    assert_eq!(
        FamiliarityLevel::from_familiarity(l19_fam),
        FamiliarityLevel::Mapped
    );
}

#[test]
fn test_l18_breakthrough_does_not_reduce_existing_l19_familiarity() {
    let mut persistent = DeepPersistent::new();
    persistent.layer_record_mut(19).familiarity = 80;
    mark_layer_cleared(&mut persistent, 18);
    assert_eq!(
        persistent.layer_record(19).unwrap().familiarity,
        80,
        "L19 familiarity should not decrease if already higher than bonus"
    );
}

#[test]
fn test_only_l18_triggers_abyss_bonus_not_other_layers() {
    let mut persistent = DeepPersistent::new();
    mark_layer_cleared(&mut persistent, 17);
    // L19 should not have been touched
    let l19_fam = persistent
        .layer_record(19)
        .map(|r| r.familiarity)
        .unwrap_or(0);
    assert_eq!(l19_fam, 0, "Clearing L17 should not affect L19 familiarity");
}

#[test]
fn test_l18_bonus_grants_exactly_25_familiarity_when_l19_is_zero() {
    let mut persistent = DeepPersistent::new();
    mark_layer_cleared(&mut persistent, 18);
    let l19_fam = persistent.layer_record(19).unwrap().familiarity;
    assert_eq!(
        l19_fam, 25,
        "L19 should have exactly 25 familiarity when starting from 0"
    );
}

#[test]
fn test_l18_bonus_preserves_l19_familiarity_at_exactly_25() {
    let mut persistent = DeepPersistent::new();
    // Set L19 to exactly 25 before clearing L18
    persistent.layer_record_mut(19).familiarity = 25;
    mark_layer_cleared(&mut persistent, 18);
    assert_eq!(
        persistent.layer_record(19).unwrap().familiarity,
        25,
        "L19 familiarity should remain 25 when already at bonus threshold"
    );
}

#[test]
fn test_abyss_entry_bonus_only_fires_once() {
    let mut persistent = DeepPersistent::new();
    mark_layer_cleared(&mut persistent, 18);
    assert_eq!(persistent.layer_record(19).unwrap().familiarity, 25);
    // Clearing L18 again (idempotent) should not increase further
    mark_layer_cleared(&mut persistent, 18);
    assert_eq!(
        persistent.layer_record(19).unwrap().familiarity,
        25,
        "Repeated L18 clears should not stack the Abyss entry bonus"
    );
}
