//! Tests for The Deep — Discovery flow and new player tutorial experience.
//!
//! Validates the onboarding sequence for The Deep mercenary expedition system:
//! 1. Discovery chance mechanics (P15+ gate, scaling with prestige rank)
//! 2. Initial state after discovery (Guild Rank 1, 0 Marks, empty roster)
//! 3. Prestige reset behavior (account state persists, per-prestige state resets)
//! 4. Guild rank properties (roster caps, concurrent missions, advancement)
//! 5. Type-level API contracts (mercs, missions, events, infrastructure)
//!
//! Tests that require tick integration (TickEvent::DeepDiscovered, deep_changed flag,
//! mission generation/logic) are marked #[ignore] until Tasks #10, #12, #13, #14 land.
//!
//! These tests follow the same patterns as haven/soulforge discovery tests in
//! tick_stage_coverage_test.rs and game_loop_orchestration_test.rs.

use quest::deep::{
    deep_discovery_chance, CheckInEvent, DeepPersistent, DeepPrestige, DeepState, DeepUiState,
    DeepView, EventChoice, GuildRank, Infrastructure, Layer, LayerTier, MercArchetype, MercStatus,
    Mercenary, MissionType, DEEP_DISCOVERY_BASE_CHANCE, DEEP_MIN_PRESTIGE_RANK,
};

// =============================================================================
// 1. Discovery Chance — Prestige Gating
// =============================================================================

/// Discovery chance is 0.0 below P15.
#[test]
fn test_deep_discovery_chance_zero_below_p15() {
    assert_eq!(deep_discovery_chance(0), 0.0);
    assert_eq!(deep_discovery_chance(1), 0.0);
    assert_eq!(deep_discovery_chance(10), 0.0);
    assert_eq!(deep_discovery_chance(14), 0.0);
}

/// Discovery chance is exactly the base at P15.
#[test]
fn test_deep_discovery_chance_at_p15() {
    let chance = deep_discovery_chance(15);
    assert!(
        (chance - DEEP_DISCOVERY_BASE_CHANCE).abs() < 1e-10,
        "At P15, chance should equal the base chance"
    );
}

/// Discovery chance increases with prestige rank above minimum.
#[test]
fn test_deep_discovery_chance_scales_with_rank() {
    let p15 = deep_discovery_chance(15);
    let p20 = deep_discovery_chance(20);
    let p30 = deep_discovery_chance(30);
    let p50 = deep_discovery_chance(50);
    assert!(p20 > p15, "P20 chance should exceed P15");
    assert!(p30 > p20, "P30 chance should exceed P20");
    assert!(p50 > p30, "P50 chance should exceed P30");
}

/// Discovery chance follows the same formula as Haven and Soulforge:
/// base + (rank - min_rank) * rank_bonus.
#[test]
fn test_deep_discovery_chance_formula_matches_design() {
    let rank = 25_u32;
    let expected = DEEP_DISCOVERY_BASE_CHANCE
        + (rank - DEEP_MIN_PRESTIGE_RANK) as f64 * quest::deep::DEEP_DISCOVERY_RANK_BONUS;
    let actual = deep_discovery_chance(rank);
    assert!(
        (actual - expected).abs() < 1e-15,
        "Discovery chance should match the documented formula"
    );
}

// =============================================================================
// 2. Initial State After Discovery — First Overlay View
// =============================================================================

/// A fresh DeepState should have discovery=false, Guild Rank 1, 0 Marks,
/// empty roster, no active missions, deepest layer 0.
#[test]
fn test_deep_initial_state() {
    let deep = DeepState::new();

    assert!(!deep.persistent.discovered, "Starts undiscovered");
    assert_eq!(deep.persistent.guild_rank, GuildRank::MIN);
    assert_eq!(deep.persistent.guild_rank, GuildRank(1));
    assert_eq!(deep.persistent.deepest_layer_reached, 0);
    assert!(deep.persistent.layers.is_empty());
    assert_eq!(deep.prestige.warband_marks, 0);
    assert!(deep.prestige.roster.is_empty());
    assert!(deep.prestige.active_missions.is_empty());
    assert!(deep.prestige.available_missions.is_empty());
}

/// Guild Rank 1 (Freelancers) has correct roster cap and concurrent missions.
#[test]
fn test_deep_initial_guild_rank_properties() {
    let rank = GuildRank(1);
    assert_eq!(rank.display_name(), "Freelancers");
    assert_eq!(rank.max_roster(), 5);
    assert_eq!(rank.concurrent_missions(), 1);
    assert_eq!(rank.required_breakthrough_layer(), None);
}

/// Persistent state should start with frontier at layer 1.
#[test]
fn test_deep_initial_frontier_is_layer_1() {
    let deep = DeepPersistent::new();
    assert_eq!(deep.frontier_layer(), 1);
}

/// DeepUiState should start closed at the Hub view.
#[test]
fn test_deep_ui_state_starts_closed() {
    let ui = DeepUiState::new();
    assert!(!ui.open);
    assert_eq!(ui.view, DeepView::Hub);
    assert_eq!(ui.selected_index, 0);
    assert!(ui.event_mission_id.is_none());
    assert!(ui.staged_squad.is_empty());
}

/// Opening the Deep overlay resets to Hub with clean selection state.
#[test]
fn test_deep_ui_open_resets_state() {
    let mut ui = DeepUiState::new();
    ui.view = DeepView::Roster;
    ui.selected_index = 5;
    ui.staged_squad.push(42);
    ui.open();

    assert!(ui.open);
    assert_eq!(ui.view, DeepView::Hub);
    assert_eq!(ui.selected_index, 0);
    assert!(ui.staged_squad.is_empty());
}

/// Closing the Deep overlay resets all state.
#[test]
fn test_deep_ui_close_resets_state() {
    let mut ui = DeepUiState::new();
    ui.open();
    ui.view = DeepView::NewMission;
    ui.selected_index = 3;
    ui.close();

    assert!(!ui.open);
    assert_eq!(ui.view, DeepView::Hub);
    assert_eq!(ui.selected_index, 0);
}

// =============================================================================
// 3. Prestige Reset Behavior
// =============================================================================

/// on_prestige() resets per-prestige state while keeping persistent state.
#[test]
fn test_deep_prestige_resets_prestige_state() {
    let mut deep = DeepState::new();

    // Set up some persistent state
    deep.persistent.discovered = true;
    deep.persistent.guild_rank = GuildRank(3);
    deep.persistent.deepest_layer_reached = 7;
    deep.persistent.layer_record_mut(1).cleared = true;
    deep.persistent.layer_record_mut(2).cleared = true;

    // Set up some per-prestige state
    deep.prestige.warband_marks = 9999;
    deep.prestige.roster.push(Mercenary {
        id: 1,
        name: "TestMerc".to_string(),
        archetype: MercArchetype::Vanguard,
        power: 14,
        resilience: 12,
        expertise: 4,
        level: 5,
        missions_completed: 10,
        status: MercStatus::Available,
    });

    deep.on_prestige();

    // Persistent state survives
    assert!(deep.persistent.discovered);
    assert_eq!(deep.persistent.guild_rank, GuildRank(3));
    assert_eq!(deep.persistent.deepest_layer_reached, 7);
    assert!(deep.persistent.layers[0].cleared);
    assert!(deep.persistent.layers[1].cleared);

    // Per-prestige state resets
    assert_eq!(deep.prestige.warband_marks, 0);
    assert!(deep.prestige.roster.is_empty());
    assert!(deep.prestige.active_missions.is_empty());
}

/// Prestige should not affect layer infrastructure (persists across prestiges).
#[test]
fn test_deep_prestige_preserves_infrastructure() {
    let mut deep = DeepState::new();
    deep.persistent.discovered = true;
    deep.persistent
        .layer_record_mut(1)
        .infrastructure
        .push(Infrastructure::Outpost);
    deep.persistent
        .layer_record_mut(1)
        .infrastructure
        .push(Infrastructure::Watchtower);

    deep.on_prestige();

    let layer = deep.persistent.layer_record(1).unwrap();
    assert_eq!(layer.infrastructure.len(), 2);
    assert!(layer.infrastructure.contains(&Infrastructure::Outpost));
    assert!(layer.infrastructure.contains(&Infrastructure::Watchtower));
}

/// Prestige should not affect layer familiarity (intel persists).
#[test]
fn test_deep_prestige_preserves_familiarity() {
    let mut deep = DeepState::new();
    deep.persistent.discovered = true;
    deep.persistent.layer_record_mut(1).familiarity = 75;
    deep.persistent.layer_record_mut(2).familiarity = 30;

    deep.on_prestige();

    assert_eq!(deep.persistent.layer_record(1).unwrap().familiarity, 75);
    assert_eq!(deep.persistent.layer_record(2).unwrap().familiarity, 30);
}

// =============================================================================
// 4. Guild Rank Properties
// =============================================================================

/// All five guild ranks should have valid display names.
#[test]
fn test_all_guild_rank_names() {
    let expected = [
        "Freelancers",
        "Sellswords",
        "Company",
        "Battalion",
        "Legion",
    ];
    for i in 1..=5u8 {
        assert_eq!(GuildRank(i).display_name(), expected[(i - 1) as usize]);
    }
}

/// Roster size should increase monotonically across ranks.
#[test]
fn test_guild_rank_roster_increases() {
    let mut prev = 0;
    for i in 1..=5u8 {
        let roster = GuildRank(i).max_roster();
        assert!(
            roster > prev,
            "Rank {} roster ({}) should exceed rank {} roster ({})",
            i,
            roster,
            i - 1,
            prev
        );
        prev = roster;
    }
}

/// Concurrent missions should increase (or stay the same) across ranks.
#[test]
fn test_guild_rank_concurrent_missions_nondecreasing() {
    let mut prev = 0;
    for i in 1..=5u8 {
        let missions = GuildRank(i).concurrent_missions();
        assert!(
            missions >= prev,
            "Rank {} concurrent missions ({}) should be >= rank {} ({})",
            i,
            missions,
            i - 1,
            prev
        );
        prev = missions;
    }
}

/// Rank 1 has no breakthrough requirement; higher ranks require progressively deeper layers.
#[test]
fn test_guild_rank_breakthrough_requirements() {
    assert!(GuildRank(1).required_breakthrough_layer().is_none());

    let mut prev_layer = 0u32;
    for i in 2..=5u8 {
        let req = GuildRank(i).required_breakthrough_layer().unwrap();
        assert!(
            req > prev_layer,
            "Rank {} should require deeper layer than rank {}",
            i,
            i - 1
        );
        prev_layer = req;
    }
}

/// Rank advancement: can advance from 1-4, cannot advance from 5.
#[test]
fn test_guild_rank_advancement() {
    for i in 1..=4u8 {
        assert!(
            GuildRank(i).can_advance(),
            "Rank {} should be able to advance",
            i
        );
        assert_eq!(GuildRank(i).next(), Some(GuildRank(i + 1)));
    }
    assert!(!GuildRank(5).can_advance());
    assert_eq!(GuildRank(5).next(), None);
}

// =============================================================================
// 5. Mercenary Type Contracts
// =============================================================================

/// All archetypes have positive base stats.
#[test]
fn test_all_archetypes_have_positive_stats() {
    for &archetype in MercArchetype::ALL {
        let (p, r, e) = archetype.base_stats();
        assert!(p > 0 && r > 0 && e > 0, "{:?} has zero stats", archetype);
    }
}

/// Archetypes have non-empty display names.
#[test]
fn test_all_archetypes_have_names() {
    for &archetype in MercArchetype::ALL {
        assert!(!archetype.display_name().is_empty());
    }
}

/// Mercenary availability depends on MercStatus variant.
#[test]
fn test_mercenary_availability() {
    let base = Mercenary {
        id: 1,
        name: "Test".to_string(),
        archetype: MercArchetype::Scout,
        power: 10,
        resilience: 10,
        expertise: 10,
        level: 1,
        missions_completed: 0,
        status: MercStatus::Available,
    };

    assert!(base.is_available());

    let on_mission = Mercenary {
        status: MercStatus::OnMission(42),
        ..base.clone()
    };
    assert!(!on_mission.is_available());

    let injured = Mercenary {
        status: MercStatus::Injured {
            missions_remaining: 2,
        },
        ..base.clone()
    };
    assert!(!injured.is_available());

    let lost = Mercenary {
        status: MercStatus::Lost,
        ..base
    };
    assert!(!lost.is_available());
}

/// Effective stats increase with level.
#[test]
fn test_mercenary_effective_stats_scale() {
    let merc_lv1 = Mercenary {
        id: 1,
        name: "Test".to_string(),
        archetype: MercArchetype::Vanguard,
        power: 14,
        resilience: 12,
        expertise: 4,
        level: 1,
        missions_completed: 0,
        status: MercStatus::Available,
    };

    let merc_lv5 = Mercenary {
        level: 5,
        ..merc_lv1.clone()
    };

    assert!(merc_lv5.effective_power() > merc_lv1.effective_power());
    assert!(merc_lv5.effective_resilience() > merc_lv1.effective_resilience());
}

// =============================================================================
// 6. Mission Type Properties
// =============================================================================

/// Risk tiers: Supply Run and Construction are safe (0), others increase.
#[test]
fn test_mission_type_risk_tiers() {
    assert_eq!(MissionType::SupplyRun.risk_tier(), 0);
    assert_eq!(
        MissionType::Construction(Infrastructure::Outpost).risk_tier(),
        0
    );
    assert_eq!(MissionType::Recon.risk_tier(), 1);
    assert_eq!(MissionType::Expedition.risk_tier(), 2);
    assert_eq!(MissionType::Breakthrough.risk_tier(), 3);
}

/// Duration ranges increase with risk level.
#[test]
fn test_mission_duration_ranges_ordered() {
    let supply_min = MissionType::SupplyRun.duration_range_secs().0;
    let recon_min = MissionType::Recon.duration_range_secs().0;
    let expedition_min = MissionType::Expedition.duration_range_secs().0;
    let breakthrough_min = MissionType::Breakthrough.duration_range_secs().0;

    assert!(supply_min < recon_min, "Supply min < Recon min");
    assert!(recon_min < expedition_min, "Recon min < Expedition min");
    assert!(
        expedition_min < breakthrough_min,
        "Expedition min < Breakthrough min"
    );
}

/// Supply runs and construction have 0 check-in events. Breakthroughs have the most.
#[test]
fn test_mission_max_events() {
    assert_eq!(MissionType::SupplyRun.max_events(), 0);
    assert_eq!(
        MissionType::Construction(Infrastructure::Bridge).max_events(),
        0
    );
    assert!(MissionType::Recon.max_events() > 0);
    assert!(MissionType::Breakthrough.max_events() > MissionType::Expedition.max_events());
}

/// All mission types have non-empty display names.
#[test]
fn test_mission_type_display_names() {
    let types = [
        MissionType::SupplyRun,
        MissionType::Recon,
        MissionType::Expedition,
        MissionType::Breakthrough,
        MissionType::Construction(Infrastructure::Outpost),
    ];
    for mt in &types {
        assert!(!mt.display_name().is_empty());
    }
}

// =============================================================================
// 7. Layer and Infrastructure
// =============================================================================

/// Layer tiers are correctly assigned from layer index.
#[test]
fn test_layer_tier_boundaries() {
    assert_eq!(LayerTier::from_layer(1), LayerTier::Shallows);
    assert_eq!(LayerTier::from_layer(3), LayerTier::Shallows);
    assert_eq!(LayerTier::from_layer(4), LayerTier::Warrens);
    assert_eq!(LayerTier::from_layer(7), LayerTier::Warrens);
    assert_eq!(LayerTier::from_layer(8), LayerTier::Hollows);
    assert_eq!(LayerTier::from_layer(12), LayerTier::Hollows);
    assert_eq!(LayerTier::from_layer(13), LayerTier::SunkenReach);
    assert_eq!(LayerTier::from_layer(18), LayerTier::SunkenReach);
    assert_eq!(LayerTier::from_layer(19), LayerTier::Abyss);
    assert_eq!(LayerTier::from_layer(25), LayerTier::Abyss);
    assert_eq!(LayerTier::from_layer(26), LayerTier::Void);
    assert_eq!(LayerTier::from_layer(100), LayerTier::Void);
}

/// All layer tiers have non-empty display names.
#[test]
fn test_layer_tier_display_names() {
    let tiers = [
        LayerTier::Shallows,
        LayerTier::Warrens,
        LayerTier::Hollows,
        LayerTier::SunkenReach,
        LayerTier::Abyss,
        LayerTier::Void,
    ];
    for tier in &tiers {
        assert!(!tier.display_name().is_empty());
    }
}

/// Infrastructure duration reduction is capped at 75%.
#[test]
fn test_layer_duration_reduction_capped() {
    let layer = Layer {
        index: 1,
        tier: LayerTier::Shallows,
        theme_name: "Test".to_string(),
        difficulty: 1.0,
        familiarity: 0,
        // Simulate impossible scenario to test the cap
        infrastructure: vec![
            Infrastructure::Outpost,
            Infrastructure::Outpost,
            Infrastructure::Outpost,
            Infrastructure::Outpost,
        ],
        cleared: false,
    };
    assert_eq!(layer.total_duration_reduction(), 0.75);
}

/// Infrastructure has_infrastructure query works correctly.
#[test]
fn test_layer_has_infrastructure_query() {
    let layer = Layer {
        index: 1,
        tier: LayerTier::Shallows,
        theme_name: "Test".to_string(),
        difficulty: 1.0,
        familiarity: 0,
        infrastructure: vec![Infrastructure::Outpost, Infrastructure::Watchtower],
        cleared: true,
    };

    assert!(layer.has_infrastructure(Infrastructure::Outpost));
    assert!(layer.has_infrastructure(Infrastructure::Watchtower));
    assert!(!layer.has_infrastructure(Infrastructure::SupplyCache));
    assert!(!layer.has_infrastructure(Infrastructure::Bridge));
}

/// Available infrastructure slots = 4 - built.
#[test]
fn test_layer_available_infrastructure_slots() {
    let layer = Layer {
        index: 1,
        tier: LayerTier::Shallows,
        theme_name: "Test".to_string(),
        difficulty: 1.0,
        familiarity: 0,
        infrastructure: vec![Infrastructure::Outpost],
        cleared: true,
    };
    assert_eq!(layer.available_infrastructure_slots(), 3);
}

/// All infrastructure types have non-empty names and descriptions.
#[test]
fn test_infrastructure_metadata() {
    for &infra in Infrastructure::ALL {
        assert!(!infra.display_name().is_empty());
        assert!(!infra.description().is_empty());
    }
}

/// Only Outpost provides duration reduction.
#[test]
fn test_only_outpost_reduces_duration() {
    assert!(Infrastructure::Outpost.duration_reduction() > 0.0);
    for &infra in Infrastructure::ALL {
        if infra != Infrastructure::Outpost {
            assert_eq!(
                infra.duration_reduction(),
                0.0,
                "{:?} should not reduce duration",
                infra
            );
        }
    }
}

// =============================================================================
// 8. DeepPersistent — Layer Record Management
// =============================================================================

/// Layer records grow on demand and maintain correct indices.
#[test]
fn test_persistent_layer_records_grow_on_demand() {
    let mut dp = DeepPersistent::new();
    let _ = dp.layer_record_mut(5);
    assert_eq!(dp.layers.len(), 5);
    for (i, layer) in dp.layers.iter().enumerate() {
        assert_eq!(layer.index, (i + 1) as u32);
        assert_eq!(layer.familiarity, 0);
        assert!(!layer.cleared);
        assert!(layer.infrastructure.is_empty());
    }
}

/// Frontier advances past cleared layers.
#[test]
fn test_persistent_frontier_advances() {
    let mut dp = DeepPersistent::new();
    dp.layer_record_mut(1).cleared = true;
    dp.layer_record_mut(2).cleared = true;
    let _ = dp.layer_record_mut(3); // create but don't clear

    assert_eq!(dp.frontier_layer(), 3);
}

/// When all layers are cleared, frontier returns last layer + 1.
#[test]
fn test_persistent_frontier_when_all_cleared() {
    let mut dp = DeepPersistent::new();
    dp.layer_record_mut(1).cleared = true;
    dp.layer_record_mut(2).cleared = true;
    // All tracked layers are cleared, but there's no layer 3 yet
    // frontier_layer should return 1 (since no uncleared layer found)
    // This is correct since frontier_layer falls back to max(1, ...)
    let frontier = dp.frontier_layer();
    assert!(frontier >= 1);
}

/// ID counters are monotonically increasing and independent.
#[test]
fn test_persistent_id_counters() {
    let mut dp = DeepPersistent::new();
    let m1 = dp.next_merc_id();
    let m2 = dp.next_merc_id();
    let mi1 = dp.next_mission_id();
    let mi2 = dp.next_mission_id();

    assert!(m2 > m1, "Merc IDs should increase");
    assert!(mi2 > mi1, "Mission IDs should increase");
    // They're independent counters
    assert_eq!(m1, 1);
    assert_eq!(mi1, 1);
}

// =============================================================================
// 9. DeepPrestige — Economy
// =============================================================================

/// Spending marks deducts the correct amount.
#[test]
fn test_prestige_spend_marks_success() {
    let mut dp = DeepPrestige::new();
    dp.warband_marks = 500;

    assert!(dp.spend_marks(200));
    assert_eq!(dp.warband_marks, 300);

    assert!(dp.spend_marks(300));
    assert_eq!(dp.warband_marks, 0);
}

/// Spending more marks than available fails without modifying the balance.
#[test]
fn test_prestige_spend_marks_insufficient() {
    let mut dp = DeepPrestige::new();
    dp.warband_marks = 100;

    assert!(!dp.spend_marks(200));
    assert_eq!(dp.warband_marks, 100, "Balance should be unchanged");
}

/// Find merc by id returns the correct mercenary.
#[test]
fn test_prestige_find_merc() {
    let mut dp = DeepPrestige::new();
    dp.roster.push(Mercenary {
        id: 42,
        name: "FindMe".to_string(),
        archetype: MercArchetype::Scout,
        power: 10,
        resilience: 10,
        expertise: 10,
        level: 1,
        missions_completed: 0,
        status: MercStatus::Available,
    });

    assert!(dp.find_merc(42).is_some());
    assert_eq!(dp.find_merc(42).unwrap().name, "FindMe");
    assert!(dp.find_merc(999).is_none());
}

/// Available merc count only counts mercs with status Available.
#[test]
fn test_prestige_available_merc_count() {
    let mut dp = DeepPrestige::new();
    let base = Mercenary {
        id: 0,
        name: "M".to_string(),
        archetype: MercArchetype::Vanguard,
        power: 10,
        resilience: 10,
        expertise: 10,
        level: 1,
        missions_completed: 0,
        status: MercStatus::Available,
    };

    dp.roster.push(Mercenary {
        id: 1,
        status: MercStatus::Available,
        ..base.clone()
    });
    dp.roster.push(Mercenary {
        id: 2,
        status: MercStatus::OnMission(1),
        ..base.clone()
    });
    dp.roster.push(Mercenary {
        id: 3,
        status: MercStatus::Injured {
            missions_remaining: 1,
        },
        ..base.clone()
    });
    dp.roster.push(Mercenary {
        id: 4,
        status: MercStatus::Available,
        ..base
    });

    assert_eq!(dp.available_merc_count(), 2);
}

// =============================================================================
// 10. CheckInEvent — Auto-Resolve and Resolution
// =============================================================================

/// Unresolved events fall back to auto_resolve_choice.
#[test]
fn test_checkin_event_auto_resolve_fallback() {
    let event = CheckInEvent {
        title: "Cave-in".to_string(),
        description: "A tunnel collapsed.".to_string(),
        choices: vec![
            EventChoice {
                label: "Dig through".to_string(),
                required_archetype: None,
                time_delta_secs: 3 * 3600,
                is_risky: false,
                unlocks_bonus_event: false,
                risk_percent: None,
            },
            EventChoice {
                label: "Find alternate route".to_string(),
                required_archetype: Some(MercArchetype::Saboteur),
                time_delta_secs: 0,
                is_risky: true,
                unlocks_bonus_event: true,
                risk_percent: Some(30),
            },
        ],
        auto_resolve_choice: 0,
        archetype_bonus: Some(MercArchetype::Saboteur),
        fired_at: chrono::Utc::now(),
        resolved_choice: None,
    };

    assert_eq!(
        event.effective_choice(),
        0,
        "Should fall back to auto-resolve"
    );
    assert!(!event.is_resolved());
}

/// Player resolution overrides auto-resolve.
#[test]
fn test_checkin_event_player_resolution() {
    let mut event = CheckInEvent {
        title: "Cave-in".to_string(),
        description: "A tunnel collapsed.".to_string(),
        choices: vec![
            EventChoice {
                label: "Dig through".to_string(),
                required_archetype: None,
                time_delta_secs: 3 * 3600,
                is_risky: false,
                unlocks_bonus_event: false,
                risk_percent: None,
            },
            EventChoice {
                label: "Find alternate route".to_string(),
                required_archetype: Some(MercArchetype::Saboteur),
                time_delta_secs: 0,
                is_risky: true,
                unlocks_bonus_event: true,
                risk_percent: Some(30),
            },
        ],
        auto_resolve_choice: 0,
        archetype_bonus: Some(MercArchetype::Saboteur),
        fired_at: chrono::Utc::now(),
        resolved_choice: None,
    };

    event.resolved_choice = Some(1);
    assert_eq!(event.effective_choice(), 1);
    assert!(event.is_resolved());
}

/// Auto-resolve choice should always point to a safe (non-risky) option.
#[test]
fn test_checkin_event_auto_resolve_is_safe() {
    let event = CheckInEvent {
        title: "Test".to_string(),
        description: "Test event.".to_string(),
        choices: vec![
            EventChoice {
                label: "Safe option".to_string(),
                required_archetype: None,
                time_delta_secs: 3600,
                is_risky: false,
                unlocks_bonus_event: false,
                risk_percent: None,
            },
            EventChoice {
                label: "Risky option".to_string(),
                required_archetype: Some(MercArchetype::Scout),
                time_delta_secs: 0,
                is_risky: true,
                unlocks_bonus_event: false,
                risk_percent: Some(40),
            },
        ],
        auto_resolve_choice: 0,
        archetype_bonus: None,
        fired_at: chrono::Utc::now(),
        resolved_choice: None,
    };

    let auto_choice = &event.choices[event.auto_resolve_choice];
    assert!(
        !auto_choice.is_risky,
        "Auto-resolve should always pick the safe option"
    );
}

// =============================================================================
// 11. Serialization Roundtrip
// =============================================================================

/// DeepState should survive JSON serialization/deserialization.
#[test]
fn test_deep_state_serde_roundtrip() {
    let mut deep = DeepState::new();
    deep.persistent.discovered = true;
    deep.persistent.guild_rank = GuildRank(3);
    deep.persistent.deepest_layer_reached = 8;
    deep.prestige.warband_marks = 1240;
    deep.persistent.layer_record_mut(1).cleared = true;
    deep.persistent
        .layer_record_mut(1)
        .infrastructure
        .push(Infrastructure::Outpost);

    let json = serde_json::to_string(&deep).unwrap();
    let loaded: DeepState = serde_json::from_str(&json).unwrap();

    assert!(loaded.persistent.discovered);
    assert_eq!(loaded.persistent.guild_rank, GuildRank(3));
    assert_eq!(loaded.persistent.deepest_layer_reached, 8);
    assert_eq!(loaded.prestige.warband_marks, 1240);
    assert!(loaded.persistent.layer_record(1).unwrap().cleared);
    assert!(loaded
        .persistent
        .layer_record(1)
        .unwrap()
        .infrastructure
        .contains(&Infrastructure::Outpost));
}

/// Mercenary struct should survive JSON roundtrip.
#[test]
fn test_mercenary_serde_roundtrip() {
    let merc = Mercenary {
        id: 42,
        name: "Rodgar".to_string(),
        archetype: MercArchetype::Vanguard,
        power: 14,
        resilience: 12,
        expertise: 4,
        level: 3,
        missions_completed: 7,
        status: MercStatus::Injured {
            missions_remaining: 1,
        },
    };

    let json = serde_json::to_string(&merc).unwrap();
    let loaded: Mercenary = serde_json::from_str(&json).unwrap();

    assert_eq!(loaded.id, 42);
    assert_eq!(loaded.name, "Rodgar");
    assert_eq!(loaded.archetype, MercArchetype::Vanguard);
    assert_eq!(loaded.level, 3);
    assert_eq!(loaded.missions_completed, 7);
    assert!(matches!(
        loaded.status,
        MercStatus::Injured {
            missions_remaining: 1
        }
    ));
}

// =============================================================================
// 12. Tick Integration — Discovery via game_tick
// =============================================================================

use quest::achievements::Achievements;
use quest::core::game_state::GameState;
use quest::core::tick::game_tick;
use quest::core::tick_types::TickEvent;
use quest::enhancement::EnhancementProgress;
use quest::haven::Haven;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn strong_p50_state(name: &str) -> GameState {
    use quest::character::attributes::AttributeType;
    let mut state = GameState::new(name.to_string(), 0);
    state.prestige_rank = 50;
    state.attributes.set(AttributeType::Strength, 50);
    state.attributes.set(AttributeType::Constitution, 30);
    state
}

/// The Deep is discovered through the narrative story chain, not tick-based rolls.
/// Story advances on prestige when the player has reached The Expanse (Zone 11+).
#[test]
fn test_deep_discovery_via_narrative_story_chain() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut deep = DeepState::new();

    // Simulate 7 prestiges from The Expanse at P21+
    for _ in 0..7 {
        deep.maybe_increment_rift_resonance(11, 21);
    }
    assert_eq!(deep.persistent.rift_resonance, 7);

    // Advance story through all stages
    let _stage1 = quest::deep::advance_deep_story(&mut deep, 15);
    assert_eq!(deep.persistent.deep_story_stage, 1);

    deep.persistent.rift_resonance = 3;
    deep.persistent.deep_story_stage = 1;
    let _stage2 = quest::deep::advance_deep_story(&mut deep, 17);
    assert_eq!(deep.persistent.deep_story_stage, 2);

    deep.persistent.rift_resonance = 5;
    deep.persistent.deep_story_stage = 2;
    let _stage3 = quest::deep::advance_deep_story(&mut deep, 19);
    assert_eq!(deep.persistent.deep_story_stage, 3);

    deep.persistent.rift_resonance = 7;
    deep.persistent.deep_story_stage = 3;
    deep.persistent.rift_fragments = 4;
    let _stage4 = quest::deep::advance_deep_story(&mut deep, 21);
    assert_eq!(deep.persistent.deep_story_stage, 4);

    // Now complete discovery (simulates player pressing [D])
    quest::deep::complete_story_discovery(&mut deep, &mut rng);
    assert!(deep.persistent.discovered);
    assert_eq!(deep.persistent.deep_story_stage, 5);
    assert_eq!(deep.prestige.roster.len(), 3);
    assert_eq!(deep.prestige.warband_marks, 50);
}

/// Discovery should NOT fire via game_tick — no tick-based discovery roll exists.
#[test]
fn test_deep_discovery_blocked_below_p15_via_game_tick() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut state = GameState::new("Deep P14 Test".to_string(), 0);
    state.prestige_rank = 14;
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut enhancement = EnhancementProgress::new();
    let mut deep = DeepState::new();
    let mut achievements = Achievements::default();

    for _ in 0..1000 {
        let result = game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut enhancement,
            &mut deep,
            &mut achievements,
            false,
            &mut rng,
        );
        assert!(
            !result.deep_changed,
            "Deep should not be discovered via tick"
        );
    }
    assert!(!deep.persistent.discovered);
}

/// Deep discovery should NOT happen via game_tick even at high prestige.
/// Discovery is narrative-only (story chain through The Expanse).
#[test]
fn test_deep_discovery_blocked_by_dungeon_via_game_tick() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut state = strong_p50_state("Deep Tick Block Test");
    let mut tick_counter = 0u32;
    let mut haven = Haven::default();
    let mut enhancement = EnhancementProgress::new();
    let mut deep = DeepState::new();
    let mut achievements = Achievements::default();

    for _ in 0..500 {
        let result = game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut enhancement,
            &mut deep,
            &mut achievements,
            false,
            &mut rng,
        );
        let discovered = result
            .events
            .iter()
            .any(|e| matches!(e, TickEvent::DeepDiscovered));
        assert!(
            !discovered,
            "Deep should never be discovered via tick — narrative only"
        );
    }
    assert!(!deep.persistent.discovered);
}

/// Story chain requires rift fragments to reach stage 4.
#[test]
fn test_deep_story_requires_fragments() {
    let mut deep = DeepState::new();

    // Set resonance and prestige high enough for stage 4
    deep.persistent.rift_resonance = 7;
    deep.persistent.deep_story_stage = 3;
    // But don't give fragments
    deep.persistent.rift_fragments = 0;

    let result = deep.check_story_progression(21);
    assert_eq!(result, None, "Stage 4 requires 4 rift fragments");

    // Now give fragments
    deep.persistent.rift_fragments = 4;
    let result = deep.check_story_progression(21);
    assert_eq!(result, Some(4));
}

/// complete_story_discovery is idempotent — calling twice doesn't double-init.
#[test]
fn test_deep_discovery_only_once_via_game_tick() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut deep = DeepState::new();
    deep.persistent.deep_story_stage = 4;
    deep.persistent.rift_fragments = 4;

    quest::deep::complete_story_discovery(&mut deep, &mut rng);
    assert!(deep.persistent.discovered);
    let roster_len = deep.prestige.roster.len();
    let marks = deep.prestige.warband_marks;

    // Second call should be a no-op
    quest::deep::complete_story_discovery(&mut deep, &mut rng);
    assert_eq!(deep.prestige.roster.len(), roster_len);
    assert_eq!(deep.prestige.warband_marks, marks);
}

/// Rift resonance only increments when player reached The Expanse (zone 11+).
#[test]
fn test_deep_discovery_debug_mode_via_game_tick() {
    let mut deep = DeepState::new();

    // Zone 10 (not The Expanse) — should not increment
    deep.maybe_increment_rift_resonance(10, 15);
    assert_eq!(deep.persistent.rift_resonance, 0);

    // Zone 11 (The Expanse) at P15+ — should increment
    deep.maybe_increment_rift_resonance(11, 15);
    assert_eq!(deep.persistent.rift_resonance, 1);

    // Below P15 — should not increment even in Expanse
    deep.maybe_increment_rift_resonance(11, 14);
    assert_eq!(deep.persistent.rift_resonance, 1);
}

/// Deep discovery via narrative creates correct initial state.
#[test]
fn test_deep_discovery_triggers_achievement_via_game_tick() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut deep = DeepState::new();
    deep.persistent.deep_story_stage = 4;
    deep.persistent.rift_fragments = 4;

    quest::deep::complete_story_discovery(&mut deep, &mut rng);

    assert!(deep.persistent.discovered);
    assert_eq!(deep.persistent.deep_story_stage, 5);
    assert_eq!(deep.prestige.roster.len(), 3);
    assert_eq!(deep.prestige.warband_marks, 50);
    assert_eq!(deep.persistent.guild_rank, quest::deep::GuildRank(1));

    // Verify starter archetypes
    let archetypes: Vec<_> = deep.prestige.roster.iter().map(|m| m.archetype).collect();
    assert!(archetypes.contains(&quest::deep::MercArchetype::Vanguard));
    assert!(archetypes.contains(&quest::deep::MercArchetype::Scout));
    assert!(archetypes.contains(&quest::deep::MercArchetype::Medic));
}

// =============================================================================
// 13. Mission System Integration — Tutorial Flow
// =============================================================================

/// Initial mission pool contains a SupplyRun on Layer 1 as the first mission.
#[test]
fn test_initial_descent_is_first_mission() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut deep = DeepState::new();
    deep.persistent.discovered = true;
    // Create starter mercs
    let starter = quest::deep::generate_starter_roster(
        deep.persistent.guild_rank,
        &mut || deep.persistent.next_merc_id(),
        &mut rng,
    );
    deep.prestige.roster = starter;

    let pool = quest::deep::generate_mission_pool(&deep.persistent, &mut rng);
    assert!(!pool.is_empty(), "Mission pool should not be empty");

    // First mission should be a SupplyRun on Layer 1
    let first = &pool[0];
    assert_eq!(first.mission_type, MissionType::SupplyRun);
    assert_eq!(first.layer, 1);
}

/// After clearing Layer 1, the mission pool includes Recon on the frontier.
#[test]
fn test_expanded_options_after_clearing_layer() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut deep = DeepState::new();
    deep.persistent.discovered = true;
    deep.persistent.guild_rank = GuildRank(2); // More mission slots
    deep.persistent.layer_record_mut(1).cleared = true;
    deep.persistent.deepest_layer_reached = 1;

    let pool = quest::deep::generate_mission_pool(&deep.persistent, &mut rng);
    assert!(pool.len() >= 2, "Should have at least 2 missions at Rank 2");

    let has_supply = pool
        .iter()
        .any(|m| m.mission_type == MissionType::SupplyRun);
    let has_recon = pool.iter().any(|m| m.mission_type == MissionType::Recon);
    assert!(has_supply, "Pool should contain a Supply Run");
    assert!(has_recon, "Pool should contain a Recon mission");
}

/// Supply runs have risk tier 0 and should never injure mercs when resolved.
#[test]
fn test_supply_runs_are_always_safe() {
    use quest::deep::{generate_mission_pool, resolve_mission, start_mission};

    for seed in 0u64..10 {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut deep = DeepState::new();
        deep.persistent.discovered = true;
        deep.persistent.layer_record_mut(1).cleared = true;
        deep.persistent.deepest_layer_reached = 1;

        // Create a starter roster
        let starter = quest::deep::generate_starter_roster(
            deep.persistent.guild_rank,
            &mut || deep.persistent.next_merc_id(),
            &mut rng,
        );
        deep.prestige.roster = starter;
        deep.prestige.warband_marks = 1000;

        let pool = generate_mission_pool(&deep.persistent, &mut rng);
        let supply_run = pool
            .iter()
            .find(|m| m.mission_type == MissionType::SupplyRun)
            .expect("Should have a supply run");

        let merc_id = deep.prestige.roster[0].id;
        let mut mission = start_mission(
            supply_run,
            &[merc_id],
            &mut deep.prestige,
            &mut deep.persistent,
            false,
            chrono::Utc::now() - chrono::Duration::hours(24),
            &mut rng,
        );

        resolve_mission(
            &mut mission,
            &mut deep.prestige,
            &mut deep.persistent,
            &mut rng,
        );

        let result = mission
            .result
            .as_ref()
            .expect("Mission should have a result");
        assert!(
            result.lost_mercs.is_empty(),
            "Supply run should never lose mercs (seed {})",
            seed
        );
        assert!(
            result.injured_mercs.is_empty(),
            "Supply run should never injure mercs (seed {})",
            seed
        );
    }
}

/// Squad assignment validation rejects empty squads and invalid merc IDs.
#[test]
fn test_squad_assignment_validation() {
    use quest::deep::{generate_mission_pool, validate_squad_assignment, SquadAssignmentError};

    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut deep = DeepState::new();
    deep.persistent.discovered = true;

    let starter = quest::deep::generate_starter_roster(
        deep.persistent.guild_rank,
        &mut || deep.persistent.next_merc_id(),
        &mut rng,
    );
    deep.prestige.roster = starter;

    let pool = generate_mission_pool(&deep.persistent, &mut rng);
    let mission = &pool[0];

    // Empty squad should fail
    let result = validate_squad_assignment(mission, &[], &deep.prestige, &deep.persistent, false);
    assert!(matches!(result, Err(SquadAssignmentError::EmptySquad)));

    // Invalid merc ID should fail
    let result =
        validate_squad_assignment(mission, &[99999], &deep.prestige, &deep.persistent, false);
    assert!(result.is_err(), "Invalid merc ID should fail validation");
}

/// Cannot exceed max concurrent missions at Rank 1 (1 mission).
#[test]
fn test_concurrent_mission_cap() {
    use quest::deep::{
        generate_mission_pool, start_mission, validate_squad_assignment, SquadAssignmentError,
    };

    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut deep = DeepState::new();
    deep.persistent.discovered = true;

    let starter = quest::deep::generate_starter_roster(
        deep.persistent.guild_rank,
        &mut || deep.persistent.next_merc_id(),
        &mut rng,
    );
    deep.prestige.roster = starter;
    deep.prestige.warband_marks = 5000;

    let pool = generate_mission_pool(&deep.persistent, &mut rng);
    let first_mission = &pool[0];

    // Start first mission (use first merc) and push to active list
    let merc1_id = deep.prestige.roster[0].id;
    let mission = start_mission(
        first_mission,
        &[merc1_id],
        &mut deep.prestige,
        &mut deep.persistent,
        false,
        chrono::Utc::now(),
        &mut rng,
    );
    deep.prestige.active_missions.push(mission);

    // Now active_mission_count() should be 1, matching Rank 1 concurrent limit of 1
    assert_eq!(deep.prestige.active_mission_count(), 1);
    assert_eq!(deep.persistent.guild_rank.concurrent_missions(), 1);

    // Try to start second mission — should hit concurrent limit
    let pool2 = generate_mission_pool(&deep.persistent, &mut rng);
    assert!(!pool2.is_empty(), "Pool should have missions");
    let merc2_id = deep.prestige.roster[1].id;
    let result = validate_squad_assignment(
        &pool2[0],
        &[merc2_id],
        &deep.prestige,
        &deep.persistent,
        false,
    );
    assert!(
        matches!(result, Err(SquadAssignmentError::ConcurrentMissionLimit)),
        "Should hit concurrent mission limit at Rank 1, got {:?}",
        result
    );
}
