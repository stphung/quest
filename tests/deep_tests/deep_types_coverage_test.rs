//! Coverage tests for `src/deep/types.rs`.
//!
//! Targets branches and methods not covered by the inline `#[cfg(test)]` tests
//! in types.rs or the integration test suite. Focuses on:
//!
//!  - Display/name methods for all enum variants
//!  - All `MercArchetype::base_stats()` exact values
//!  - `MissionType::display_name()` including `Construction` variants
//!  - `MissionOutcome::display_name()` — all three variants
//!  - `Infrastructure::display_name()`, `description()`, `duration_reduction()`
//!  - `LayerTier::display_name()` — all six tiers
//!  - `DeepPersistent` id counters and frontier edge cases
//!  - `DeepPrestige` find/count/event helpers with mixed-status rosters
//!  - `Mercenary` effective stat scaling and `missions_to_next_level` curve
//!  - `GuildRank` out-of-range, advancement helpers
//!  - `Mission::progress()`, `is_time_elapsed()`, `unresolved_event_count()`
//!  - `CheckInEvent::effective_choice()` and `is_resolved()` branches
//!  - `Layer` infrastructure helpers and `available_infrastructure_slots()`
//!  - `LayerRecord` helpers
//!  - `DeepState::on_prestige()` generation counter and record cap
//!  - `DeepState::is_active()`
//!  - `DeepView` tab navigation and labels
//!  - `DeepUiState` open/close state transitions
//!  - `RecruitPool::needs_refresh()`
//!  - Serde round-trips for all major structs

use chrono::{Duration, TimeZone, Utc};
use quest::deep::types::*;

// ─── helpers ──────────────────────────────────────────────────────────────────

/// Fixed wall-clock anchor so tests do not depend on the real system clock.
fn t0() -> chrono::DateTime<chrono::Utc> {
    Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap()
}

fn make_merc(id: u64, archetype: MercArchetype, status: MercStatus) -> Mercenary {
    let (power, resilience, expertise) = archetype.base_stats();
    Mercenary {
        id,
        name: format!("Merc-{id}"),
        archetype,
        power,
        resilience,
        expertise,
        level: 1,
        missions_completed: 0,
        status,
    }
}

fn make_active_mission(id: u64, mission_type: MissionType, layer: u32, squad: Vec<u64>) -> Mission {
    let start = t0();
    let end = start + Duration::hours(2);
    Mission {
        id,
        mission_type,
        layer,
        squad,
        started_at: start,
        ends_at: end,
        events: Vec::new(),
        pending_event_index: 0,
        status: MissionStatus::Active,
        result: None,
        is_first_orders: false,
    }
}

fn make_event(auto_resolve_choice: usize, resolved_choice: Option<usize>) -> CheckInEvent {
    CheckInEvent {
        title: "Test Event".to_string(),
        description: "Something happened.".to_string(),
        choices: vec![
            EventChoice {
                label: "Safe choice".to_string(),
                required_archetype: None,
                time_delta_secs: 0,
                is_risky: false,
                unlocks_bonus_event: false,
                risk_percent: None,
            },
            EventChoice {
                label: "Risky choice".to_string(),
                required_archetype: Some(MercArchetype::Scout),
                time_delta_secs: -3600,
                is_risky: true,
                unlocks_bonus_event: true,
                risk_percent: Some(40),
            },
        ],
        auto_resolve_choice,
        archetype_bonus: Some(MercArchetype::Scout),
        fired_at: t0(),
        resolved_choice,
    }
}

// =============================================================================
// LayerTier
// =============================================================================

#[test]
fn test_layer_tier_display_names_all_variants() {
    assert_eq!(LayerTier::Shallows.display_name(), "The Shallows");
    assert_eq!(LayerTier::Warrens.display_name(), "The Warrens");
    assert_eq!(LayerTier::Hollows.display_name(), "The Hollows");
    assert_eq!(LayerTier::SunkenReach.display_name(), "The Sunken Reach");
    assert_eq!(LayerTier::Abyss.display_name(), "The Abyss");
    assert_eq!(LayerTier::Void.display_name(), "The Void");
}

#[test]
fn test_layer_tier_from_layer_boundary_values() {
    // All tier-boundary values
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
    assert_eq!(LayerTier::from_layer(30), LayerTier::Void);
    assert_eq!(LayerTier::from_layer(100), LayerTier::Void);
}

// =============================================================================
// MercArchetype
// =============================================================================

#[test]
fn test_merc_archetype_display_names_all_variants() {
    assert_eq!(MercArchetype::Vanguard.display_name(), "Vanguard");
    assert_eq!(MercArchetype::Scout.display_name(), "Scout");
    assert_eq!(MercArchetype::Arcanist.display_name(), "Arcanist");
    assert_eq!(MercArchetype::Medic.display_name(), "Medic");
    assert_eq!(MercArchetype::Saboteur.display_name(), "Saboteur");
}

#[test]
fn test_merc_archetype_base_stats_exact_values() {
    assert_eq!(MercArchetype::Vanguard.base_stats(), (14, 12, 4));
    assert_eq!(MercArchetype::Scout.base_stats(), (8, 10, 12));
    assert_eq!(MercArchetype::Arcanist.base_stats(), (10, 6, 14));
    assert_eq!(MercArchetype::Medic.base_stats(), (6, 14, 10));
    assert_eq!(MercArchetype::Saboteur.base_stats(), (10, 8, 12));
}

#[test]
fn test_merc_archetype_all_slice_contains_exactly_five_variants() {
    assert_eq!(MercArchetype::ALL.len(), 5);
    assert!(MercArchetype::ALL.contains(&MercArchetype::Vanguard));
    assert!(MercArchetype::ALL.contains(&MercArchetype::Scout));
    assert!(MercArchetype::ALL.contains(&MercArchetype::Arcanist));
    assert!(MercArchetype::ALL.contains(&MercArchetype::Medic));
    assert!(MercArchetype::ALL.contains(&MercArchetype::Saboteur));
}

// =============================================================================
// MissionType
// =============================================================================

#[test]
fn test_mission_type_display_name_all_variants() {
    assert_eq!(MissionType::SupplyRun.display_name(), "Supply Run");
    assert_eq!(MissionType::Recon.display_name(), "Recon");
    assert_eq!(MissionType::Expedition.display_name(), "Expedition");
    assert_eq!(MissionType::Breakthrough.display_name(), "Breakthrough");
    assert_eq!(
        MissionType::GatewayExpedition.display_name(),
        "Gateway Expedition"
    );
    // Construction reports "Construction" regardless of infrastructure payload
    assert_eq!(
        MissionType::Construction(Infrastructure::Outpost).display_name(),
        "Construction"
    );
    assert_eq!(
        MissionType::Construction(Infrastructure::SupplyCache).display_name(),
        "Construction"
    );
    assert_eq!(
        MissionType::Construction(Infrastructure::Watchtower).display_name(),
        "Construction"
    );
    assert_eq!(
        MissionType::Construction(Infrastructure::Bridge).display_name(),
        "Construction"
    );
}

#[test]
fn test_mission_type_risk_tier_all_variants() {
    assert_eq!(MissionType::SupplyRun.risk_tier(), 0);
    assert_eq!(
        MissionType::Construction(Infrastructure::Outpost).risk_tier(),
        0
    );
    assert_eq!(
        MissionType::Construction(Infrastructure::Watchtower).risk_tier(),
        0
    );
    assert_eq!(MissionType::Recon.risk_tier(), 1);
    assert_eq!(MissionType::Expedition.risk_tier(), 2);
    assert_eq!(MissionType::Breakthrough.risk_tier(), 3);
    assert_eq!(MissionType::GatewayExpedition.risk_tier(), 3);
}

#[test]
fn test_mission_type_max_events_gateway_expedition() {
    assert_eq!(MissionType::GatewayExpedition.max_events(), 5);
}

#[test]
fn test_mission_type_duration_range_secs_gateway_expedition() {
    let (min, max) = MissionType::GatewayExpedition.duration_range_secs();
    assert_eq!(min, 172800);
    assert_eq!(max, 172800);
}

// =============================================================================
// MissionOutcome
// =============================================================================

// MissionOutcome does not have a Display impl in types.rs; it derives Debug.
// We verify it can be compared and used in serde round-trips.

#[test]
fn test_mission_outcome_serde_roundtrip() {
    for outcome in [
        MissionOutcome::Success,
        MissionOutcome::PartialSuccess,
        MissionOutcome::Failure,
    ] {
        let json = serde_json::to_string(&outcome).unwrap();
        let loaded: MissionOutcome = serde_json::from_str(&json).unwrap();
        assert_eq!(outcome, loaded);
    }
}

// =============================================================================
// Infrastructure
// =============================================================================

#[test]
fn test_infrastructure_display_name_all_variants() {
    assert_eq!(Infrastructure::Outpost.display_name(), "Outpost");
    assert_eq!(Infrastructure::SupplyCache.display_name(), "Supply Cache");
    assert_eq!(Infrastructure::Watchtower.display_name(), "Watchtower");
    assert_eq!(Infrastructure::Bridge.display_name(), "Bridge");
}

#[test]
fn test_infrastructure_all_slice_contains_exactly_four_variants() {
    assert_eq!(Infrastructure::ALL.len(), 4);
    assert!(Infrastructure::ALL.contains(&Infrastructure::Outpost));
    assert!(Infrastructure::ALL.contains(&Infrastructure::SupplyCache));
    assert!(Infrastructure::ALL.contains(&Infrastructure::Watchtower));
    assert!(Infrastructure::ALL.contains(&Infrastructure::Bridge));
}

#[test]
fn test_infrastructure_duration_reduction_exact_values() {
    assert_eq!(Infrastructure::Outpost.duration_reduction(), 0.25);
    assert_eq!(Infrastructure::SupplyCache.duration_reduction(), 0.0);
    assert_eq!(Infrastructure::Watchtower.duration_reduction(), 0.0);
    assert_eq!(Infrastructure::Bridge.duration_reduction(), 0.0);
}

#[test]
fn test_infrastructure_description_non_empty_for_all_variants() {
    for &infra in Infrastructure::ALL {
        let desc = infra.description();
        assert!(
            !desc.is_empty(),
            "{:?} description must not be empty",
            infra
        );
    }
}

// =============================================================================
// GuildRank
// =============================================================================

#[test]
fn test_guild_rank_display_name_out_of_range() {
    // Rank 0 and rank 6 are out of the 1-5 range
    assert_eq!(GuildRank(0).display_name(), "Unknown");
    assert_eq!(GuildRank(6).display_name(), "Unknown");
    assert_eq!(GuildRank(255).display_name(), "Unknown");
}

#[test]
fn test_guild_rank_display_name_all_valid_ranks() {
    assert_eq!(GuildRank(1).display_name(), "Freelancers");
    assert_eq!(GuildRank(2).display_name(), "Company");
    assert_eq!(GuildRank(3).display_name(), "Battalion");
    assert_eq!(GuildRank(4).display_name(), "Legion");
    assert_eq!(GuildRank(5).display_name(), "Vanguard");
}

#[test]
fn test_guild_rank_max_roster_all_ranks() {
    assert_eq!(GuildRank(1).max_roster(), 5);
    assert_eq!(GuildRank(2).max_roster(), 7);
    assert_eq!(GuildRank(3).max_roster(), 9);
    assert_eq!(GuildRank(4).max_roster(), 12);
    assert_eq!(GuildRank(5).max_roster(), 15);
}

#[test]
fn test_guild_rank_concurrent_missions_all_ranks() {
    assert_eq!(GuildRank(1).concurrent_missions(), 1);
    assert_eq!(GuildRank(2).concurrent_missions(), 2);
    assert_eq!(GuildRank(3).concurrent_missions(), 2);
    assert_eq!(GuildRank(4).concurrent_missions(), 3);
    assert_eq!(GuildRank(5).concurrent_missions(), 4);
}

#[test]
fn test_guild_rank_next_returns_none_at_max() {
    assert_eq!(GuildRank(5).next(), None);
    assert_eq!(GuildRank(4).next(), Some(GuildRank(5)));
    assert_eq!(GuildRank(1).next(), Some(GuildRank(2)));
}

#[test]
fn test_guild_rank_can_advance() {
    assert!(GuildRank(1).can_advance());
    assert!(GuildRank(4).can_advance());
    assert!(!GuildRank(5).can_advance());
}

#[test]
fn test_guild_rank_default_is_min() {
    assert_eq!(GuildRank::default(), GuildRank::MIN);
    assert_eq!(GuildRank::MIN.0, 1);
    assert_eq!(GuildRank::MAX.0, 5);
}

// =============================================================================
// effective_concurrent_missions
// =============================================================================

#[test]
fn test_effective_concurrent_missions_rank1_layer3_bonus() {
    // Rank 1 + deepest layer < 3: no bonus
    assert_eq!(effective_concurrent_missions(GuildRank(1), 0), 1);
    assert_eq!(effective_concurrent_missions(GuildRank(1), 2), 1);
    // Rank 1 + deepest layer >= 3: +1 bonus
    assert_eq!(effective_concurrent_missions(GuildRank(1), 3), 2);
    assert_eq!(effective_concurrent_missions(GuildRank(1), 10), 2);
}

#[test]
fn test_effective_concurrent_missions_higher_ranks_unaffected_by_bonus() {
    // Rank 2+ should not get the Rank 1 bonus
    assert_eq!(effective_concurrent_missions(GuildRank(2), 3), 2);
    assert_eq!(effective_concurrent_missions(GuildRank(3), 10), 2);
    assert_eq!(effective_concurrent_missions(GuildRank(4), 20), 3);
    assert_eq!(effective_concurrent_missions(GuildRank(5), 25), 4);
}

// =============================================================================
// Mercenary helpers
// =============================================================================

#[test]
fn test_mercenary_is_available_for_each_status() {
    let avail = make_merc(1, MercArchetype::Vanguard, MercStatus::Available);
    assert!(avail.is_available());

    let on_mission = make_merc(2, MercArchetype::Scout, MercStatus::OnMission(99));
    assert!(!on_mission.is_available());

    let injured = make_merc(
        3,
        MercArchetype::Medic,
        MercStatus::Injured {
            missions_remaining: 3,
        },
    );
    assert!(!injured.is_available());

    let lost = make_merc(4, MercArchetype::Arcanist, MercStatus::Lost);
    assert!(!lost.is_available());
}

#[test]
fn test_mercenary_effective_power_at_level_1() {
    // Level 1: no bonus applied (level.saturating_sub(1) == 0)
    let m = make_merc(1, MercArchetype::Vanguard, MercStatus::Available);
    assert_eq!(m.effective_power(), m.power); // 14
    assert_eq!(m.effective_resilience(), m.resilience); // 12
}

#[test]
fn test_mercenary_effective_power_scaling() {
    // Level 5: effective_power = base + (5-1) * 3 = base + 12
    //          effective_resilience = base + (5-1) * 2 = base + 8
    let mut m = make_merc(1, MercArchetype::Vanguard, MercStatus::Available);
    m.level = 5;
    assert_eq!(m.effective_power(), m.power + 12);
    assert_eq!(m.effective_resilience(), m.resilience + 8);
}

#[test]
fn test_mercenary_effective_power_scaling_high_level() {
    // Level 10: effective_power = base + 9 * 3 = base + 27
    let mut m = make_merc(1, MercArchetype::Scout, MercStatus::Available);
    m.level = 10;
    assert_eq!(m.effective_power(), m.power + 27);
    assert_eq!(m.effective_resilience(), m.resilience + 18);
}

#[test]
fn test_mercenary_missions_to_next_level_curve() {
    // Formula: 3 + level * 2
    assert_eq!(Mercenary::missions_to_next_level(1), 5);
    assert_eq!(Mercenary::missions_to_next_level(2), 7);
    assert_eq!(Mercenary::missions_to_next_level(5), 13);
    assert_eq!(Mercenary::missions_to_next_level(10), 23);
}

// =============================================================================
// Layer helpers
// =============================================================================

#[test]
fn test_layer_available_infrastructure_slots_empty() {
    let layer = Layer {
        index: 1,
        tier: LayerTier::Shallows,
        theme_name: "Test".to_string(),
        difficulty: 1.0,
        familiarity: 0,
        infrastructure: Vec::new(),
        cleared: false,
    };
    assert_eq!(layer.available_infrastructure_slots(), 4);
}

#[test]
fn test_layer_available_infrastructure_slots_partial() {
    let layer = Layer {
        index: 1,
        tier: LayerTier::Shallows,
        theme_name: "Test".to_string(),
        difficulty: 1.0,
        familiarity: 0,
        infrastructure: vec![Infrastructure::Outpost, Infrastructure::Watchtower],
        cleared: false,
    };
    assert_eq!(layer.available_infrastructure_slots(), 2);
}

#[test]
fn test_layer_available_infrastructure_slots_full() {
    let layer = Layer {
        index: 1,
        tier: LayerTier::Shallows,
        theme_name: "Test".to_string(),
        difficulty: 1.0,
        familiarity: 0,
        infrastructure: vec![
            Infrastructure::Outpost,
            Infrastructure::SupplyCache,
            Infrastructure::Watchtower,
            Infrastructure::Bridge,
        ],
        cleared: false,
    };
    assert_eq!(layer.available_infrastructure_slots(), 0);
}

#[test]
fn test_layer_has_infrastructure() {
    let layer = Layer {
        index: 2,
        tier: LayerTier::Shallows,
        theme_name: "Test".to_string(),
        difficulty: 1.0,
        familiarity: 50,
        infrastructure: vec![Infrastructure::SupplyCache],
        cleared: true,
    };
    assert!(layer.has_infrastructure(Infrastructure::SupplyCache));
    assert!(!layer.has_infrastructure(Infrastructure::Outpost));
    assert!(!layer.has_infrastructure(Infrastructure::Watchtower));
    assert!(!layer.has_infrastructure(Infrastructure::Bridge));
}

#[test]
fn test_layer_total_duration_reduction_no_infra() {
    let layer = Layer {
        index: 1,
        tier: LayerTier::Shallows,
        theme_name: "Test".to_string(),
        difficulty: 1.0,
        familiarity: 0,
        infrastructure: Vec::new(),
        cleared: false,
    };
    assert_eq!(layer.total_duration_reduction(), 0.0);
}

#[test]
fn test_layer_total_duration_reduction_outpost_only() {
    let layer = Layer {
        index: 1,
        tier: LayerTier::Shallows,
        theme_name: "Test".to_string(),
        difficulty: 1.0,
        familiarity: 0,
        infrastructure: vec![Infrastructure::Outpost],
        cleared: false,
    };
    assert_eq!(layer.total_duration_reduction(), 0.25);
}

#[test]
fn test_layer_total_duration_reduction_caps_at_75_percent() {
    // Four outposts would give 1.0 without cap
    let layer = Layer {
        index: 1,
        tier: LayerTier::Shallows,
        theme_name: "Test".to_string(),
        difficulty: 1.0,
        familiarity: 0,
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

// =============================================================================
// LayerRecord helpers
// =============================================================================

#[test]
fn test_layer_record_new() {
    let record = LayerRecord::new(5);
    assert_eq!(record.index, 5);
    assert_eq!(record.familiarity, 0);
    assert!(record.infrastructure.is_empty());
    assert!(!record.cleared);
}

#[test]
fn test_layer_record_has_infrastructure() {
    let mut record = LayerRecord::new(1);
    assert!(!record.has_infrastructure(Infrastructure::Outpost));
    record.infrastructure.push(Infrastructure::Outpost);
    assert!(record.has_infrastructure(Infrastructure::Outpost));
    assert!(!record.has_infrastructure(Infrastructure::Bridge));
}

#[test]
fn test_layer_record_total_duration_reduction_no_infra() {
    let record = LayerRecord::new(1);
    assert_eq!(record.total_duration_reduction(), 0.0);
}

#[test]
fn test_layer_record_total_duration_reduction_with_outpost() {
    let mut record = LayerRecord::new(1);
    record.infrastructure.push(Infrastructure::Outpost);
    assert_eq!(record.total_duration_reduction(), 0.25);
}

#[test]
fn test_layer_record_total_duration_reduction_capped() {
    let mut record = LayerRecord::new(1);
    // Multiple outposts still cap at 0.75
    for _ in 0..4 {
        record.infrastructure.push(Infrastructure::Outpost);
    }
    assert_eq!(record.total_duration_reduction(), 0.75);
}

// =============================================================================
// DeepPersistent — id counters and frontier
// =============================================================================

#[test]
fn test_deep_persistent_next_merc_id_increments() {
    let mut dp = DeepPersistent::new();
    assert_eq!(dp.next_merc_id(), 1);
    assert_eq!(dp.next_merc_id(), 2);
    assert_eq!(dp.next_merc_id(), 3);
    assert_eq!(dp.merc_id_counter, 3);
}

#[test]
fn test_deep_persistent_next_mission_id_increments() {
    let mut dp = DeepPersistent::new();
    assert_eq!(dp.next_mission_id(), 1);
    assert_eq!(dp.next_mission_id(), 2);
    assert_eq!(dp.mission_id_counter, 2);
}

#[test]
fn test_deep_persistent_merc_and_mission_counters_are_independent() {
    let mut dp = DeepPersistent::new();
    let _m1 = dp.next_merc_id();
    let _m2 = dp.next_merc_id();
    // Mission counter starts fresh
    assert_eq!(dp.next_mission_id(), 1);
    assert_eq!(dp.next_merc_id(), 3);
}

#[test]
fn test_deep_persistent_layer_record_zero_returns_none() {
    let dp = DeepPersistent::new();
    assert!(dp.layer_record(0).is_none());
}

#[test]
fn test_deep_persistent_layer_record_unreached_returns_none() {
    let dp = DeepPersistent::new();
    assert!(dp.layer_record(1).is_none());
    assert!(dp.layer_record(99).is_none());
}

#[test]
fn test_deep_persistent_layer_record_after_creation() {
    let mut dp = DeepPersistent::new();
    let _ = dp.layer_record_mut(3);
    // Index 1 and 2 are also created lazily
    assert!(dp.layer_record(1).is_some());
    assert!(dp.layer_record(2).is_some());
    assert!(dp.layer_record(3).is_some());
    assert!(dp.layer_record(4).is_none());
}

#[test]
fn test_deep_persistent_layer_record_mut_creates_lazily() {
    let mut dp = DeepPersistent::new();
    let record = dp.layer_record_mut(5);
    assert_eq!(record.index, 5);
    assert_eq!(dp.layers.len(), 5);
    // Intermediate layers are created with correct indices
    for (i, layer) in dp.layers.iter().enumerate() {
        assert_eq!(layer.index, i as u32 + 1);
    }
}

#[test]
fn test_deep_persistent_frontier_layer_no_layers_cleared() {
    let dp = DeepPersistent::new();
    // No layers yet → frontier is 1
    assert_eq!(dp.frontier_layer(), 1);
}

#[test]
fn test_deep_persistent_frontier_layer_uncleared_first_layer() {
    let mut dp = DeepPersistent::new();
    // Create layer 1 but do not clear it
    let _ = dp.layer_record_mut(1);
    assert_eq!(dp.frontier_layer(), 1);
}

#[test]
fn test_deep_persistent_frontier_layer_skips_cleared() {
    let mut dp = DeepPersistent::new();
    dp.layer_record_mut(1).cleared = true;
    dp.layer_record_mut(2).cleared = true;
    let _ = dp.layer_record_mut(3); // not cleared
    assert_eq!(dp.frontier_layer(), 3);
}

#[test]
fn test_deep_persistent_frontier_layer_all_cleared_returns_next() {
    let mut dp = DeepPersistent::new();
    dp.layer_record_mut(1).cleared = true;
    dp.layer_record_mut(2).cleared = true;
    dp.layer_record_mut(3).cleared = true;
    dp.deepest_layer_reached = 3;
    assert_eq!(dp.frontier_layer(), 4);
}

#[test]
fn test_deep_persistent_frontier_layer_deepest_zero_fallback() {
    let dp = DeepPersistent::new();
    // Edge case: all empty, deepest_layer_reached = 0 → frontier = max(0+1, 1) = 1
    assert_eq!(dp.frontier_layer(), 1);
}

// =============================================================================
// DeepPrestige — roster/mission helpers
// =============================================================================

#[test]
fn test_deep_prestige_find_merc_success_and_failure() {
    let mut dp = DeepPrestige::new();
    {
        let m = make_merc(10, MercArchetype::Vanguard, MercStatus::Available);
        dp.roster.insert(m.id, m);
    }
    {
        let m = make_merc(20, MercArchetype::Scout, MercStatus::OnMission(1));
        dp.roster.insert(m.id, m);
    }

    let found = dp.find_merc(10);
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, 10);

    assert!(dp.find_merc(99).is_none());
}

#[test]
fn test_deep_prestige_find_merc_mut_success_and_failure() {
    let mut dp = DeepPrestige::new();
    {
        let m = make_merc(5, MercArchetype::Medic, MercStatus::Available);
        dp.roster.insert(m.id, m);
    }

    {
        let merc = dp.find_merc_mut(5);
        assert!(merc.is_some());
        merc.unwrap().level = 3;
    }
    assert_eq!(dp.roster[&5].level, 3);
    assert!(dp.find_merc_mut(999).is_none());
}

#[test]
fn test_deep_prestige_available_merc_count_mixed_statuses() {
    let mut dp = DeepPrestige::new();
    for (id, arch, status) in [
        (1, MercArchetype::Vanguard, MercStatus::Available),
        (2, MercArchetype::Scout, MercStatus::OnMission(1)),
        (3, MercArchetype::Medic, MercStatus::Available),
        (
            4,
            MercArchetype::Arcanist,
            MercStatus::Injured {
                missions_remaining: 2,
            },
        ),
        (5, MercArchetype::Saboteur, MercStatus::Lost),
    ] {
        let m = make_merc(id, arch, status);
        dp.roster.insert(m.id, m);
    }

    assert_eq!(dp.available_merc_count(), 2);
}

#[test]
fn test_deep_prestige_available_merc_count_empty_roster() {
    let dp = DeepPrestige::new();
    assert_eq!(dp.available_merc_count(), 0);
}

#[test]
fn test_deep_prestige_available_merc_count_all_available() {
    let mut dp = DeepPrestige::new();
    for i in 0..3 {
        let m = make_merc(i, MercArchetype::Vanguard, MercStatus::Available);
        dp.roster.insert(m.id, m);
    }
    assert_eq!(dp.available_merc_count(), 3);
}

#[test]
fn test_deep_prestige_active_mission_count() {
    let mut dp = DeepPrestige::new();

    let mut m1 = make_active_mission(1, MissionType::SupplyRun, 1, vec![1]);
    m1.status = MissionStatus::Active;
    let mut m2 = make_active_mission(2, MissionType::Recon, 1, vec![2]);
    m2.status = MissionStatus::EventPending;
    let mut m3 = make_active_mission(3, MissionType::Expedition, 2, vec![3]);
    m3.status = MissionStatus::Completed;
    let mut m4 = make_active_mission(4, MissionType::Breakthrough, 2, vec![4]);
    m4.status = MissionStatus::Failed;
    let mut m5 = make_active_mission(5, MissionType::SupplyRun, 1, vec![5]);
    m5.status = MissionStatus::Preparing;

    dp.active_missions.extend([m1, m2, m3, m4, m5]);

    // Only Active and EventPending count
    assert_eq!(dp.active_mission_count(), 2);
}

#[test]
fn test_deep_prestige_active_mission_count_empty() {
    let dp = DeepPrestige::new();
    assert_eq!(dp.active_mission_count(), 0);
}

#[test]
fn test_deep_prestige_has_any_pending_event_false_when_none() {
    let mut dp = DeepPrestige::new();
    let mut m = make_active_mission(1, MissionType::Recon, 1, vec![1]);
    m.status = MissionStatus::Active;
    dp.active_missions.push(m);
    assert!(!dp.has_any_pending_event());
}

#[test]
fn test_deep_prestige_has_any_pending_event_true_when_some() {
    let mut dp = DeepPrestige::new();
    let mut m = make_active_mission(1, MissionType::Recon, 1, vec![1]);
    m.status = MissionStatus::EventPending;
    dp.active_missions.push(m);
    assert!(dp.has_any_pending_event());
}

#[test]
fn test_deep_prestige_spend_marks_success() {
    let mut dp = DeepPrestige::new();
    dp.warband_marks = 100;
    assert!(dp.spend_marks(60));
    assert_eq!(dp.warband_marks, 40);
}

#[test]
fn test_deep_prestige_spend_marks_exact_balance() {
    let mut dp = DeepPrestige::new();
    dp.warband_marks = 50;
    assert!(dp.spend_marks(50));
    assert_eq!(dp.warband_marks, 0);
}

#[test]
fn test_deep_prestige_spend_marks_insufficient_leaves_balance_unchanged() {
    let mut dp = DeepPrestige::new();
    dp.warband_marks = 30;
    assert!(!dp.spend_marks(31));
    assert_eq!(dp.warband_marks, 30);
}

#[test]
fn test_deep_prestige_spend_marks_zero_always_succeeds() {
    let mut dp = DeepPrestige::new();
    dp.warband_marks = 0;
    assert!(dp.spend_marks(0));
    assert_eq!(dp.warband_marks, 0);
}

// =============================================================================
// Mission helpers
// =============================================================================

#[test]
fn test_mission_progress_at_start() {
    let start = t0();
    let end = start + Duration::hours(4);
    let m = Mission {
        id: 1,
        mission_type: MissionType::Recon,
        layer: 1,
        squad: vec![1],
        started_at: start,
        ends_at: end,
        events: Vec::new(),
        pending_event_index: 0,
        status: MissionStatus::Active,
        result: None,
        is_first_orders: false,
    };
    assert_eq!(m.progress(start), 0.0);
}

#[test]
fn test_mission_progress_at_midpoint() {
    let start = t0();
    let end = start + Duration::hours(4);
    let m = Mission {
        id: 1,
        mission_type: MissionType::Recon,
        layer: 1,
        squad: vec![1],
        started_at: start,
        ends_at: end,
        events: Vec::new(),
        pending_event_index: 0,
        status: MissionStatus::Active,
        result: None,
        is_first_orders: false,
    };
    let midpoint = start + Duration::hours(2);
    let progress = m.progress(midpoint);
    assert!((progress - 0.5).abs() < 1e-9);
}

#[test]
fn test_mission_progress_after_end_clamps_to_1() {
    let start = t0();
    let end = start + Duration::hours(4);
    let m = Mission {
        id: 1,
        mission_type: MissionType::Expedition,
        layer: 2,
        squad: vec![2],
        started_at: start,
        ends_at: end,
        events: Vec::new(),
        pending_event_index: 0,
        status: MissionStatus::Active,
        result: None,
        is_first_orders: false,
    };
    let after = end + Duration::hours(2);
    assert_eq!(m.progress(after), 1.0);
}

#[test]
fn test_mission_is_time_elapsed_before_end() {
    let start = t0();
    let end = start + Duration::hours(2);
    let m = Mission {
        id: 1,
        mission_type: MissionType::SupplyRun,
        layer: 1,
        squad: vec![1],
        started_at: start,
        ends_at: end,
        events: Vec::new(),
        pending_event_index: 0,
        status: MissionStatus::Active,
        result: None,
        is_first_orders: false,
    };
    let now = start + Duration::hours(1);
    assert!(!m.is_time_elapsed(now));
}

#[test]
fn test_mission_is_time_elapsed_at_end() {
    let start = t0();
    let end = start + Duration::hours(2);
    let m = Mission {
        id: 1,
        mission_type: MissionType::SupplyRun,
        layer: 1,
        squad: vec![1],
        started_at: start,
        ends_at: end,
        events: Vec::new(),
        pending_event_index: 0,
        status: MissionStatus::Active,
        result: None,
        is_first_orders: false,
    };
    assert!(m.is_time_elapsed(end));
}

#[test]
fn test_mission_is_time_elapsed_after_end() {
    let start = t0();
    let end = start + Duration::hours(2);
    let m = Mission {
        id: 1,
        mission_type: MissionType::SupplyRun,
        layer: 1,
        squad: vec![1],
        started_at: start,
        ends_at: end,
        events: Vec::new(),
        pending_event_index: 0,
        status: MissionStatus::Active,
        result: None,
        is_first_orders: false,
    };
    assert!(m.is_time_elapsed(end + Duration::seconds(1)));
}

#[test]
fn test_mission_unresolved_event_count_none() {
    let mut m = make_active_mission(1, MissionType::Expedition, 2, vec![1, 2]);
    // No events
    assert_eq!(m.unresolved_event_count(), 0);

    // All resolved
    m.events.push(make_event(0, Some(0)));
    m.events.push(make_event(0, Some(1)));
    assert_eq!(m.unresolved_event_count(), 0);
}

#[test]
fn test_mission_unresolved_event_count_mixed() {
    let mut m = make_active_mission(1, MissionType::Expedition, 2, vec![1, 2]);
    m.events.push(make_event(0, Some(0))); // resolved
    m.events.push(make_event(0, None)); // unresolved
    m.events.push(make_event(0, None)); // unresolved
    assert_eq!(m.unresolved_event_count(), 2);
}

#[test]
fn test_mission_has_pending_event_false() {
    let m = make_active_mission(1, MissionType::Recon, 1, vec![1]);
    assert!(!m.has_pending_event()); // status is Active
}

#[test]
fn test_mission_has_pending_event_true() {
    let mut m = make_active_mission(1, MissionType::Recon, 1, vec![1]);
    m.status = MissionStatus::EventPending;
    assert!(m.has_pending_event());
}

// =============================================================================
// CheckInEvent helpers
// =============================================================================

#[test]
fn test_check_in_event_effective_choice_uses_auto_resolve_when_unresolved() {
    let event = make_event(0, None);
    assert_eq!(event.effective_choice(), 0);
    assert!(!event.is_resolved());
}

#[test]
fn test_check_in_event_effective_choice_uses_player_choice_when_resolved() {
    let event = make_event(0, Some(1));
    assert_eq!(event.effective_choice(), 1);
    assert!(event.is_resolved());
}

#[test]
fn test_check_in_event_effective_choice_auto_resolve_index_1() {
    // auto_resolve_choice can be index 1
    let event = make_event(1, None);
    assert_eq!(event.effective_choice(), 1);
}

#[test]
fn test_check_in_event_is_resolved_none_vs_some() {
    let unresolved = make_event(0, None);
    assert!(!unresolved.is_resolved());

    let resolved = make_event(0, Some(0));
    assert!(resolved.is_resolved());
}

// =============================================================================
// DeepState
// =============================================================================

#[test]
fn test_deep_state_is_active_reflects_discovered_flag() {
    let mut ds = DeepState::new();
    assert!(!ds.is_active());
    ds.persistent.discovered = true;
    assert!(ds.is_active());
}

#[test]
fn test_deep_state_on_prestige_preserves_all_state() {
    let mut ds = DeepState::new();
    ds.persistent.discovered = true;
    ds.persistent.guild_rank = GuildRank(3);
    ds.persistent.deepest_layer_reached = 7;
    ds.prestige.warband_marks = 5000;
    {
        let m = make_merc(1, MercArchetype::Vanguard, MercStatus::Available);
        ds.prestige.roster.insert(m.id, m);
    }

    ds.on_prestige();

    // Operational state persists across prestiges
    assert_eq!(ds.prestige.warband_marks, 5000);
    assert_eq!(ds.prestige.roster.len(), 1);

    // Generation counter advances
    assert_eq!(ds.persistent.generation_counter, 1);
    assert_eq!(ds.prestige.generation_number, 1);

    // Persistent state preserved
    assert!(ds.persistent.discovered);
    assert_eq!(ds.persistent.guild_rank, GuildRank(3));
    assert_eq!(ds.persistent.deepest_layer_reached, 7);
}

#[test]
fn test_deep_state_on_prestige_increments_generation_counter() {
    let mut ds = DeepState::new();
    assert_eq!(ds.persistent.generation_counter, 0);
    ds.on_prestige();
    assert_eq!(ds.persistent.generation_counter, 1);
    ds.on_prestige();
    assert_eq!(ds.persistent.generation_counter, 2);
    assert_eq!(ds.prestige.generation_number, 2);
}

#[test]
fn test_deep_state_on_prestige_generation_record_captures_stats() {
    let mut ds = DeepState::new();
    ds.prestige.total_marks_earned = 1500;
    ds.prestige.total_missions_completed = 12;
    ds.prestige.total_mercs_lost = 2;
    ds.persistent.deepest_layer_reached = 5;

    ds.on_prestige();

    assert_eq!(ds.persistent.generation_records.len(), 1);
    let record = &ds.persistent.generation_records[0];
    assert_eq!(record.generation, 1);
    assert_eq!(record.marks_earned, 1500);
    assert_eq!(record.missions_completed, 12);
    assert_eq!(record.mercs_lost, 2);
    assert_eq!(record.deepest_layer_reached, 5);
}

#[test]
fn test_deep_state_on_prestige_generation_records_capped_at_10() {
    let mut ds = DeepState::new();

    // Perform 12 prestige cycles
    for _ in 0..12 {
        ds.on_prestige();
    }

    // Records should be capped at 10
    assert_eq!(ds.persistent.generation_records.len(), 10);
    // The oldest records should have been dropped; newest should be generation 12
    let latest = ds.persistent.generation_records.last().unwrap();
    assert_eq!(latest.generation, 12);
    let oldest = ds.persistent.generation_records.first().unwrap();
    assert_eq!(oldest.generation, 3); // generations 1 and 2 were dropped
}

#[test]
fn test_deep_state_serde_roundtrip() {
    let mut ds = DeepState::new();
    ds.persistent.discovered = true;
    ds.persistent.guild_rank = GuildRank(4);
    ds.persistent.deepest_layer_reached = 12;
    ds.prestige.warband_marks = 999;

    let json = serde_json::to_string(&ds).unwrap();
    let loaded: DeepState = serde_json::from_str(&json).unwrap();

    assert!(loaded.persistent.discovered);
    assert_eq!(loaded.persistent.guild_rank, GuildRank(4));
    assert_eq!(loaded.persistent.deepest_layer_reached, 12);
    assert_eq!(loaded.prestige.warband_marks, 999);
}

#[test]
fn test_deep_state_serde_missing_fields_use_defaults() {
    // Simulate old save files that are missing optional fields (all use `#[serde(default)]`)
    let minimal_json = r#"{
        "persistent": {
            "discovered": true,
            "guild_rank": 1,
            "guild_upgrade_cost": 500,
            "layers": [],
            "deepest_layer_reached": 0,
            "merc_id_counter": 0,
            "mission_id_counter": 0
        },
        "prestige": {
            "warband_marks": 0,
            "roster": [],
            "active_missions": [],
            "available_missions": [],
            "recruit_pool": {
                "candidates": [],
                "refreshed_at": "2024-01-01T00:00:00Z",
                "recruit_costs": []
            },
            "pending_results": []
        }
    }"#;

    let loaded: DeepState = serde_json::from_str(minimal_json).unwrap();
    assert!(loaded.persistent.discovered);
    assert_eq!(loaded.persistent.generation_counter, 0);
    assert!(!loaded.persistent.gateway_opened);
    assert!(!loaded.persistent.first_orders_queued);
    assert!(loaded.persistent.generation_records.is_empty());
    assert_eq!(loaded.prestige.generation_number, 0);
    assert!(loaded.prestige.warband_log.is_empty());
}

// =============================================================================
// DeepView tab navigation
// =============================================================================

#[test]
fn test_deep_view_tab_labels_all_variants() {
    assert_eq!(DeepView::Active.tab_label(), "Active");
    assert_eq!(DeepView::NewMission.tab_label(), "Deploy");
    assert_eq!(DeepView::Roster.tab_label(), "Roster");
    assert_eq!(DeepView::Infrastructure.tab_label(), "Layers");
    assert_eq!(DeepView::EventResponse.tab_label(), "Events");
    assert_eq!(DeepView::Recruit.tab_label(), "Recruit");
}

#[test]
fn test_deep_view_tabs_has_five_entries() {
    // 5 top-level tabs; EventResponse is a modal
    assert!(!DeepView::TABS.contains(&DeepView::EventResponse));
    assert!(DeepView::TABS.contains(&DeepView::Infrastructure));
    assert!(DeepView::TABS.contains(&DeepView::Active));
    assert!(DeepView::TABS.contains(&DeepView::NewMission));
    assert!(DeepView::TABS.contains(&DeepView::Roster));
    assert!(DeepView::TABS.contains(&DeepView::Recruit));
    assert_eq!(DeepView::TABS.len(), 5);
}

#[test]
fn test_deep_view_next_tab_wraps_around() {
    // Infrastructure → Active → NewMission → Roster → Recruit → Infrastructure
    assert_eq!(DeepView::Infrastructure.next_tab(), DeepView::Active);
    assert_eq!(DeepView::Active.next_tab(), DeepView::NewMission);
    assert_eq!(DeepView::NewMission.next_tab(), DeepView::Roster);
    assert_eq!(DeepView::Roster.next_tab(), DeepView::Recruit);
    assert_eq!(DeepView::Recruit.next_tab(), DeepView::Infrastructure);
}

#[test]
fn test_deep_view_prev_tab_wraps_around() {
    assert_eq!(DeepView::Infrastructure.prev_tab(), DeepView::Recruit);
    assert_eq!(DeepView::Recruit.prev_tab(), DeepView::Roster);
    assert_eq!(DeepView::Roster.prev_tab(), DeepView::NewMission);
    assert_eq!(DeepView::NewMission.prev_tab(), DeepView::Active);
    assert_eq!(DeepView::Active.prev_tab(), DeepView::Infrastructure);
}

#[test]
fn test_deep_view_non_tabbed_views_return_self() {
    // EventResponse is not in TABS, so next/prev returns self
    assert_eq!(DeepView::EventResponse.next_tab(), DeepView::EventResponse);
    assert_eq!(DeepView::EventResponse.prev_tab(), DeepView::EventResponse);
}

// =============================================================================
// DeepUiState open/close transitions
// =============================================================================

#[test]
fn test_deep_ui_state_default_is_closed() {
    let ui = DeepUiState::new();
    assert!(!ui.open);
    assert_eq!(ui.view, DeepView::Infrastructure);
    assert_eq!(ui.selected_index, 0);
    assert!(ui.staged_squad.is_empty());
    assert!(ui.flash_message.is_none());
    assert!(!ui.show_help);
}

#[test]
fn test_deep_ui_state_open_resets_navigation() {
    let mut ui = DeepUiState::new();
    ui.view = DeepView::Roster;
    ui.selected_index = 5;
    ui.staged_squad = vec![1, 2, 3];
    ui.flash_message = Some("some error".to_string());
    ui.show_help = true;

    ui.open();

    assert!(ui.open);
    assert_eq!(ui.view, DeepView::Infrastructure);
    assert_eq!(ui.selected_index, 0);
    assert!(ui.staged_squad.is_empty());
    assert!(ui.flash_message.is_none());
    assert!(!ui.show_help);
}

#[test]
fn test_deep_ui_state_open_increments_hub_visit_count() {
    let mut ui = DeepUiState::new();
    assert_eq!(ui.hub_visit_count, 0);
    ui.open();
    assert_eq!(ui.hub_visit_count, 1);
    ui.open();
    assert_eq!(ui.hub_visit_count, 2);
}

#[test]
fn test_deep_ui_state_close_resets_state() {
    let mut ui = DeepUiState::new();
    ui.open();
    ui.view = DeepView::Roster;
    ui.selected_index = 3;
    ui.staged_squad = vec![10, 20];
    ui.flash_message = Some("msg".to_string());

    ui.close();

    assert!(!ui.open);
    assert_eq!(ui.view, DeepView::Infrastructure);
    assert_eq!(ui.selected_index, 0);
    assert!(ui.staged_squad.is_empty());
    assert!(ui.flash_message.is_none());
    assert!(!ui.show_help);
}

// =============================================================================
// RecruitPool
// =============================================================================

#[test]
fn test_recruit_pool_needs_refresh_false_when_fresh() {
    let pool = RecruitPool {
        candidates: Vec::new(),
        refreshed_at: t0(),
        recruit_costs: Vec::new(),
    };
    let now = t0() + Duration::hours(23);
    assert!(!pool.needs_refresh(now));
}

#[test]
fn test_recruit_pool_needs_refresh_true_at_24_hours() {
    let pool = RecruitPool {
        candidates: Vec::new(),
        refreshed_at: t0(),
        recruit_costs: Vec::new(),
    };
    let now = t0() + Duration::hours(24);
    assert!(pool.needs_refresh(now));
}

#[test]
fn test_recruit_pool_needs_refresh_true_after_24_hours() {
    let pool = RecruitPool {
        candidates: Vec::new(),
        refreshed_at: t0(),
        recruit_costs: Vec::new(),
    };
    let now = t0() + Duration::hours(48);
    assert!(pool.needs_refresh(now));
}

// =============================================================================
// Remaining constants
// =============================================================================

#[test]
fn test_deep_constants() {
    assert_eq!(GATEWAY_LAYER, 30);
    assert_eq!(DEEP_MIN_PRESTIGE_RANK, 15);
}
