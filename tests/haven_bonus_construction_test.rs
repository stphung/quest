//! Tests for Haven bonus calculation and construction.
//!
//! Covers gaps not handled by haven_dungeon_coverage_test.rs:
//! - compute_bonuses at T2/T3 upgrades (upgrading changes bonuses correctly)
//! - FishingDock T4 special max rank bonus
//! - haven_discovery_chance formula at boundary prestige ranks
//! - All bonus types from compute_bonuses() verified at max tier
//! - Skill tree depth/cost relationships
//! - StormForge bonus type (StormForgeAccess)
//! - Bonus accumulation (upgrading room increases bonuses correctly)
//! - Room name and tree integrity

use quest::haven::bonus::haven_discovery_chance;
use quest::haven::logic::{can_afford, try_build_room};
use quest::haven::types::Haven;
use quest::haven::{tier_cost, HavenBonusType, HavenRoomId};

// =========================================================================
// haven_discovery_chance formula boundary conditions
// =========================================================================

#[test]
fn test_haven_discovery_chance_exactly_zero_below_p10() {
    for rank in 0..10 {
        let chance = haven_discovery_chance(rank);
        assert_eq!(
            chance, 0.0,
            "P{rank} should have exactly 0.0 discovery chance, got {chance}"
        );
    }
}

#[test]
fn test_haven_discovery_chance_base_at_p10() {
    // Base: HAVEN_DISCOVERY_BASE_CHANCE = 0.000014
    let chance = haven_discovery_chance(10);
    assert!(
        (chance - 0.000014).abs() < 1e-10,
        "P10 should be exactly 0.000014, got {chance}"
    );
}

#[test]
fn test_haven_discovery_chance_at_p11() {
    // 0.000014 + 1 * 0.000007 = 0.000021
    let chance = haven_discovery_chance(11);
    assert!(
        (chance - 0.000021).abs() < 1e-10,
        "P11 should be 0.000021, got {chance}"
    );
}

#[test]
fn test_haven_discovery_chance_at_p15() {
    // 0.000014 + 5 * 0.000007 = 0.000049
    let chance = haven_discovery_chance(15);
    assert!(
        (chance - 0.000049).abs() < 1e-10,
        "P15 should be 0.000049, got {chance}"
    );
}

#[test]
fn test_haven_discovery_chance_at_p20() {
    // 0.000014 + 10 * 0.000007 = 0.000084
    let chance = haven_discovery_chance(20);
    assert!(
        (chance - 0.000084).abs() < 1e-10,
        "P20 should be 0.000084, got {chance}"
    );
}

#[test]
fn test_haven_discovery_chance_increases_monotonically() {
    let mut prev = haven_discovery_chance(10);
    for rank in 11..=50 {
        let curr = haven_discovery_chance(rank);
        assert!(
            curr > prev,
            "Discovery chance at P{rank} ({curr}) should exceed P{} ({prev})",
            rank - 1
        );
        prev = curr;
    }
}

#[test]
fn test_haven_discovery_chance_p10_transition() {
    // P9: 0.0, P10: > 0.0 — the gate boundary
    let p9 = haven_discovery_chance(9);
    let p10 = haven_discovery_chance(10);
    assert_eq!(p9, 0.0, "P9 must be zero");
    assert!(p10 > 0.0, "P10 must be positive");
}

// =========================================================================
// Bonus accumulation through tier upgrades
// =========================================================================

#[test]
fn test_armory_bonus_increases_through_tiers() {
    let mut haven = Haven::new();
    haven.build_room(HavenRoomId::Hearthstone);

    // T0 = no bonus
    assert_eq!(haven.get_bonus(HavenBonusType::DamagePercent), 0.0);

    // T1: +5%
    haven.build_room(HavenRoomId::Armory);
    assert_eq!(haven.get_bonus(HavenBonusType::DamagePercent), 5.0);

    // T2: +10%
    haven.build_room(HavenRoomId::Armory);
    assert_eq!(haven.get_bonus(HavenBonusType::DamagePercent), 10.0);

    // T3: +25%
    haven.build_room(HavenRoomId::Armory);
    assert_eq!(haven.get_bonus(HavenBonusType::DamagePercent), 25.0);

    // Cannot upgrade past T3
    let result = haven.build_room(HavenRoomId::Armory);
    assert!(result.is_none(), "Armory should not upgrade past T3");
    assert_eq!(haven.get_bonus(HavenBonusType::DamagePercent), 25.0);
}

#[test]
fn test_hearthstone_offline_xp_bonus_through_tiers() {
    let mut haven = Haven::new();

    // T1: +25%
    haven.build_room(HavenRoomId::Hearthstone);
    assert_eq!(haven.get_bonus(HavenBonusType::OfflineXpPercent), 25.0);

    // T2: +50%
    haven.build_room(HavenRoomId::Hearthstone);
    assert_eq!(haven.get_bonus(HavenBonusType::OfflineXpPercent), 50.0);

    // T3: +100%
    haven.build_room(HavenRoomId::Hearthstone);
    assert_eq!(haven.get_bonus(HavenBonusType::OfflineXpPercent), 100.0);
}

#[test]
fn test_training_yard_xp_bonus_through_tiers() {
    let mut haven = Haven::new();
    haven.build_room(HavenRoomId::Hearthstone);
    haven.build_room(HavenRoomId::Armory);

    haven.build_room(HavenRoomId::TrainingYard);
    assert_eq!(haven.get_bonus(HavenBonusType::XpGainPercent), 5.0);

    haven.build_room(HavenRoomId::TrainingYard);
    assert_eq!(haven.get_bonus(HavenBonusType::XpGainPercent), 10.0);

    haven.build_room(HavenRoomId::TrainingYard);
    assert_eq!(haven.get_bonus(HavenBonusType::XpGainPercent), 30.0);
}

#[test]
fn test_trophy_hall_drop_rate_through_tiers() {
    let mut haven = Haven::new();
    haven.build_room(HavenRoomId::Hearthstone);
    haven.build_room(HavenRoomId::Armory);

    haven.build_room(HavenRoomId::TrophyHall);
    assert_eq!(haven.get_bonus(HavenBonusType::DropRatePercent), 5.0);

    haven.build_room(HavenRoomId::TrophyHall);
    assert_eq!(haven.get_bonus(HavenBonusType::DropRatePercent), 10.0);

    haven.build_room(HavenRoomId::TrophyHall);
    assert_eq!(haven.get_bonus(HavenBonusType::DropRatePercent), 15.0);
}

#[test]
fn test_watchtower_crit_chance_through_tiers() {
    let mut haven = Haven::new();
    haven.build_room(HavenRoomId::Hearthstone);
    haven.build_room(HavenRoomId::Armory);
    haven.build_room(HavenRoomId::TrainingYard);

    haven.build_room(HavenRoomId::Watchtower);
    assert_eq!(haven.get_bonus(HavenBonusType::CritChancePercent), 5.0);

    haven.build_room(HavenRoomId::Watchtower);
    assert_eq!(haven.get_bonus(HavenBonusType::CritChancePercent), 10.0);

    haven.build_room(HavenRoomId::Watchtower);
    assert_eq!(haven.get_bonus(HavenBonusType::CritChancePercent), 20.0);
}

#[test]
fn test_alchemy_lab_hp_regen_through_tiers() {
    let mut haven = Haven::new();
    haven.build_room(HavenRoomId::Hearthstone);
    haven.build_room(HavenRoomId::Armory);
    haven.build_room(HavenRoomId::TrophyHall);

    haven.build_room(HavenRoomId::AlchemyLab);
    assert_eq!(haven.get_bonus(HavenBonusType::HpRegenPercent), 25.0);

    haven.build_room(HavenRoomId::AlchemyLab);
    assert_eq!(haven.get_bonus(HavenBonusType::HpRegenPercent), 50.0);

    haven.build_room(HavenRoomId::AlchemyLab);
    assert_eq!(haven.get_bonus(HavenBonusType::HpRegenPercent), 100.0);
}

#[test]
fn test_war_room_double_strike_through_tiers() {
    let mut haven = build_to_war_room();

    // Build War Room
    haven.build_room(HavenRoomId::WarRoom);
    assert_eq!(haven.get_bonus(HavenBonusType::DoubleStrikeChance), 10.0);

    haven.build_room(HavenRoomId::WarRoom);
    assert_eq!(haven.get_bonus(HavenBonusType::DoubleStrikeChance), 20.0);

    haven.build_room(HavenRoomId::WarRoom);
    assert_eq!(haven.get_bonus(HavenBonusType::DoubleStrikeChance), 35.0);
}

#[test]
fn test_bedroom_regen_delay_reduction_through_tiers() {
    let mut haven = Haven::new();
    haven.build_room(HavenRoomId::Hearthstone);

    haven.build_room(HavenRoomId::Bedroom);
    assert_eq!(haven.get_bonus(HavenBonusType::HpRegenDelayReduction), 15.0);

    haven.build_room(HavenRoomId::Bedroom);
    assert_eq!(haven.get_bonus(HavenBonusType::HpRegenDelayReduction), 30.0);

    haven.build_room(HavenRoomId::Bedroom);
    assert_eq!(haven.get_bonus(HavenBonusType::HpRegenDelayReduction), 50.0);
}

#[test]
fn test_garden_fishing_timer_reduction_through_tiers() {
    let mut haven = Haven::new();
    haven.build_room(HavenRoomId::Hearthstone);
    haven.build_room(HavenRoomId::Bedroom);

    haven.build_room(HavenRoomId::Garden);
    assert_eq!(haven.get_bonus(HavenBonusType::FishingTimerReduction), 10.0);

    haven.build_room(HavenRoomId::Garden);
    assert_eq!(haven.get_bonus(HavenBonusType::FishingTimerReduction), 20.0);

    haven.build_room(HavenRoomId::Garden);
    assert_eq!(haven.get_bonus(HavenBonusType::FishingTimerReduction), 40.0);
}

#[test]
fn test_library_challenge_discovery_through_tiers() {
    let mut haven = Haven::new();
    haven.build_room(HavenRoomId::Hearthstone);
    haven.build_room(HavenRoomId::Bedroom);

    haven.build_room(HavenRoomId::Library);
    assert_eq!(
        haven.get_bonus(HavenBonusType::ChallengeDiscoveryPercent),
        20.0
    );

    haven.build_room(HavenRoomId::Library);
    assert_eq!(
        haven.get_bonus(HavenBonusType::ChallengeDiscoveryPercent),
        30.0
    );

    haven.build_room(HavenRoomId::Library);
    assert_eq!(
        haven.get_bonus(HavenBonusType::ChallengeDiscoveryPercent),
        50.0
    );
}

#[test]
fn test_workshop_item_rarity_through_tiers() {
    let mut haven = Haven::new();
    haven.build_room(HavenRoomId::Hearthstone);
    haven.build_room(HavenRoomId::Bedroom);
    haven.build_room(HavenRoomId::Library);

    haven.build_room(HavenRoomId::Workshop);
    assert_eq!(haven.get_bonus(HavenBonusType::ItemRarityPercent), 10.0);

    haven.build_room(HavenRoomId::Workshop);
    assert_eq!(haven.get_bonus(HavenBonusType::ItemRarityPercent), 15.0);

    haven.build_room(HavenRoomId::Workshop);
    assert_eq!(haven.get_bonus(HavenBonusType::ItemRarityPercent), 25.0);
}

// =========================================================================
// FishingDock — special T4 behavior
// =========================================================================

#[test]
fn test_fishing_dock_t1_gives_double_fish() {
    let mut haven = Haven::new();
    haven.build_room(HavenRoomId::Hearthstone);
    haven.build_room(HavenRoomId::Bedroom);
    haven.build_room(HavenRoomId::Garden);

    haven.build_room(HavenRoomId::FishingDock);
    assert_eq!(haven.get_bonus(HavenBonusType::DoubleFishChance), 25.0);
    assert_eq!(
        haven.fishing_rank_bonus(),
        0,
        "T1 should not give rank bonus"
    );
}

#[test]
fn test_fishing_dock_t2_gives_more_double_fish() {
    let haven = haven_with_fishing_dock(2);
    assert_eq!(haven.get_bonus(HavenBonusType::DoubleFishChance), 50.0);
    assert_eq!(haven.fishing_rank_bonus(), 0);
}

#[test]
fn test_fishing_dock_t3_gives_double_fish_100() {
    let haven = haven_with_fishing_dock(3);
    assert_eq!(haven.get_bonus(HavenBonusType::DoubleFishChance), 100.0);
    assert_eq!(haven.fishing_rank_bonus(), 0);
}

#[test]
fn test_fishing_dock_t4_grants_max_rank_bonus() {
    let haven = haven_with_fishing_dock(4);
    assert_eq!(
        haven.fishing_rank_bonus(),
        10,
        "T4 should give +10 max fishing rank"
    );
    // T4 double fish bonus is same as T3 (100%)
    assert_eq!(haven.get_bonus(HavenBonusType::DoubleFishChance), 100.0);
}

#[test]
fn test_fishing_dock_t4_cost_is_10() {
    assert_eq!(tier_cost(HavenRoomId::FishingDock, 4), 10);
}

#[test]
fn test_fishing_dock_max_tier_is_4() {
    assert_eq!(HavenRoomId::FishingDock.max_tier(), 4);
}

#[test]
fn test_fishing_dock_t4_format_bonus() {
    let formatted = HavenRoomId::FishingDock.format_bonus(4);
    assert!(
        formatted.contains("Max Fishing Rank") || formatted.contains("10"),
        "T4 format should mention max fishing rank, got '{formatted}'"
    );
}

#[test]
fn test_compute_bonuses_reflects_fishing_rank_t4() {
    let haven = haven_with_fishing_dock(4);
    let bonuses = haven.compute_bonuses();
    assert_eq!(bonuses.max_fishing_rank_bonus, 10);
}

#[test]
fn test_compute_bonuses_fishing_rank_zero_without_t4() {
    let haven = haven_with_fishing_dock(3);
    let bonuses = haven.compute_bonuses();
    assert_eq!(
        bonuses.max_fishing_rank_bonus, 0,
        "T3 should not give rank bonus"
    );
}

// =========================================================================
// StormForge bonus type
// =========================================================================

#[test]
fn test_storm_forge_bonus_type_is_access() {
    let bonus = HavenRoomId::StormForge.bonus();
    assert_eq!(bonus.bonus_type, HavenBonusType::StormForgeAccess);
}

#[test]
fn test_storm_forge_bonus_value_t1_is_one() {
    assert_eq!(HavenRoomId::StormForge.bonus_value(1), 1.0);
    assert_eq!(HavenRoomId::StormForge.bonus_value(0), 0.0);
    assert_eq!(HavenRoomId::StormForge.bonus_value(2), 0.0);
}

#[test]
fn test_compute_bonuses_has_storm_forge_false_when_unbuilt() {
    let haven = Haven::new();
    let bonuses = haven.compute_bonuses();
    assert!(!bonuses.has_storm_forge);
}

#[test]
fn test_compute_bonuses_has_storm_forge_true_when_built() {
    let haven = build_complete_haven();
    let bonuses = haven.compute_bonuses();
    assert!(bonuses.has_storm_forge);
}

// =========================================================================
// compute_bonuses — max tier comprehensive check
// =========================================================================

#[test]
fn test_compute_bonuses_all_rooms_maxed() {
    let mut haven = build_complete_haven();

    // Max all rooms to T3 (or their max tier)
    for _ in 0..3 {
        haven.build_room(HavenRoomId::Hearthstone);
        haven.build_room(HavenRoomId::Armory);
        haven.build_room(HavenRoomId::TrainingYard);
        haven.build_room(HavenRoomId::TrophyHall);
        haven.build_room(HavenRoomId::Watchtower);
        haven.build_room(HavenRoomId::AlchemyLab);
        haven.build_room(HavenRoomId::WarRoom);
        haven.build_room(HavenRoomId::Bedroom);
        haven.build_room(HavenRoomId::Garden);
        haven.build_room(HavenRoomId::Library);
        haven.build_room(HavenRoomId::Workshop);
        haven.build_room(HavenRoomId::Vault);
    }
    for _ in 0..4 {
        haven.build_room(HavenRoomId::FishingDock);
    }

    let bonuses = haven.compute_bonuses();

    // Verify each bonus type at max tier
    assert_eq!(
        bonuses.offline_xp_percent, 100.0,
        "Hearthstone T3 = 100% offline XP"
    );
    assert_eq!(bonuses.damage_percent, 25.0, "Armory T3 = 25% damage");
    assert_eq!(bonuses.xp_gain_percent, 30.0, "Training Yard T3 = 30% XP");
    assert_eq!(
        bonuses.drop_rate_percent, 15.0,
        "Trophy Hall T3 = 15% drops"
    );
    assert_eq!(
        bonuses.crit_chance_percent, 20.0,
        "Watchtower T3 = 20% crit"
    );
    assert_eq!(
        bonuses.hp_regen_percent, 100.0,
        "Alchemy Lab T3 = 100% regen"
    );
    assert_eq!(
        bonuses.double_strike_chance, 35.0,
        "War Room T3 = 35% double strike"
    );
    assert_eq!(
        bonuses.hp_regen_delay_reduction, 50.0,
        "Bedroom T3 = 50% regen delay"
    );
    assert_eq!(
        bonuses.fishing_timer_reduction, 40.0,
        "Garden T3 = 40% fishing timer"
    );
    assert_eq!(
        bonuses.challenge_discovery_percent, 50.0,
        "Library T3 = 50% discovery"
    );
    assert_eq!(
        bonuses.double_fish_chance, 100.0,
        "Fishing Dock T3-T4 = 100% double fish"
    );
    assert_eq!(
        bonuses.item_rarity_percent, 25.0,
        "Workshop T3 = 25% item rarity"
    );
    assert_eq!(bonuses.vault_slots, 5, "Vault T3 = 5 slots");
    assert_eq!(
        bonuses.max_fishing_rank_bonus, 10,
        "Fishing Dock T4 = +10 rank"
    );
    assert!(bonuses.has_storm_forge, "Storm Forge should be built");
}

#[test]
fn test_compute_bonuses_empty_haven() {
    let haven = Haven::new();
    let bonuses = haven.compute_bonuses();

    assert_eq!(bonuses.damage_percent, 0.0);
    assert_eq!(bonuses.xp_gain_percent, 0.0);
    assert_eq!(bonuses.drop_rate_percent, 0.0);
    assert_eq!(bonuses.crit_chance_percent, 0.0);
    assert_eq!(bonuses.hp_regen_percent, 0.0);
    assert_eq!(bonuses.double_strike_chance, 0.0);
    assert_eq!(bonuses.offline_xp_percent, 0.0);
    assert_eq!(bonuses.challenge_discovery_percent, 0.0);
    assert_eq!(bonuses.fishing_timer_reduction, 0.0);
    assert_eq!(bonuses.double_fish_chance, 0.0);
    assert_eq!(bonuses.item_rarity_percent, 0.0);
    assert_eq!(bonuses.hp_regen_delay_reduction, 0.0);
    assert_eq!(bonuses.vault_slots, 0);
    assert_eq!(bonuses.max_fishing_rank_bonus, 0);
    assert!(!bonuses.has_storm_forge);
}

// =========================================================================
// Room depth and tier cost relationships
// =========================================================================

#[test]
fn test_tier_cost_all_rooms_at_tier_1() {
    // Root (depth 0): 1
    assert_eq!(tier_cost(HavenRoomId::Hearthstone, 1), 1);
    // Depth 1: 1
    assert_eq!(tier_cost(HavenRoomId::Armory, 1), 1);
    assert_eq!(tier_cost(HavenRoomId::Bedroom, 1), 1);
    // Depth 2: 2
    assert_eq!(tier_cost(HavenRoomId::TrainingYard, 1), 2);
    assert_eq!(tier_cost(HavenRoomId::TrophyHall, 1), 2);
    assert_eq!(tier_cost(HavenRoomId::Garden, 1), 2);
    assert_eq!(tier_cost(HavenRoomId::Library, 1), 2);
    // Depth 3: 2
    assert_eq!(tier_cost(HavenRoomId::Watchtower, 1), 2);
    assert_eq!(tier_cost(HavenRoomId::AlchemyLab, 1), 2);
    assert_eq!(tier_cost(HavenRoomId::FishingDock, 1), 2);
    assert_eq!(tier_cost(HavenRoomId::Workshop, 1), 2);
    // Depth 4: 3
    assert_eq!(tier_cost(HavenRoomId::WarRoom, 1), 3);
    assert_eq!(tier_cost(HavenRoomId::Vault, 1), 3);
    // Depth 5 (StormForge): 25
    assert_eq!(tier_cost(HavenRoomId::StormForge, 1), 25);
}

#[test]
fn test_tier_cost_increases_with_tier_for_each_room() {
    let rooms_with_tiers_3 = [
        HavenRoomId::Hearthstone,
        HavenRoomId::Armory,
        HavenRoomId::Bedroom,
        HavenRoomId::TrainingYard,
        HavenRoomId::TrophyHall,
        HavenRoomId::Garden,
        HavenRoomId::Library,
        HavenRoomId::Watchtower,
        HavenRoomId::AlchemyLab,
        HavenRoomId::Workshop,
        HavenRoomId::WarRoom,
        HavenRoomId::Vault,
    ];

    for room in rooms_with_tiers_3 {
        let t1 = tier_cost(room, 1);
        let t2 = tier_cost(room, 2);
        let t3 = tier_cost(room, 3);
        assert!(t1 < t2, "{:?}: T1({t1}) should be < T2({t2})", room);
        assert!(t2 < t3, "{:?}: T2({t2}) should be < T3({t3})", room);
    }
}

#[test]
fn test_tier_cost_invalid_tiers_return_zero() {
    for room in HavenRoomId::ALL {
        assert_eq!(tier_cost(room, 0), 0, "{:?}: tier 0 cost should be 0", room);
        // Above max tier returns 0
        let above_max = room.max_tier() + 1;
        if room != HavenRoomId::StormForge {
            // StormForge T2 isn't handled in the match so returns 0
            assert_eq!(
                tier_cost(room, above_max),
                0,
                "{:?}: tier {} (above max) cost should be 0",
                room,
                above_max
            );
        }
    }
}

#[test]
fn test_tier_cost_deeper_rooms_cost_more_at_t1() {
    let root_cost = tier_cost(HavenRoomId::Hearthstone, 1); // depth 0
    let d1_cost = tier_cost(HavenRoomId::Armory, 1); // depth 1 (same as root in this case)
    let d2_cost = tier_cost(HavenRoomId::TrainingYard, 1); // depth 2
    let d4_cost = tier_cost(HavenRoomId::WarRoom, 1); // depth 4

    // root <= d1 <= d2 <= d4 (all T1 costs at increasing depths)
    assert!(root_cost <= d1_cost, "Root should be <= depth 1");
    assert!(d1_cost <= d2_cost, "Depth 1 should be <= depth 2");
    assert!(d2_cost < d4_cost, "Depth 2 should be < depth 4");
}

// =========================================================================
// Skill tree structure: every non-root room requires parents
// =========================================================================

#[test]
fn test_all_rooms_except_hearthstone_have_parents() {
    for room in HavenRoomId::ALL {
        if room == HavenRoomId::Hearthstone {
            assert!(
                room.parents().is_empty(),
                "Hearthstone should have no parents"
            );
        } else {
            assert!(
                !room.parents().is_empty(),
                "{:?} should have at least one parent",
                room
            );
        }
    }
}

#[test]
fn test_capstone_rooms_have_exactly_two_parents() {
    let capstones = [
        HavenRoomId::WarRoom,
        HavenRoomId::Vault,
        HavenRoomId::StormForge,
    ];
    for room in capstones {
        assert_eq!(
            room.parents().len(),
            2,
            "{:?} should have exactly 2 parents",
            room
        );
    }
}

#[test]
fn test_non_capstone_rooms_have_exactly_one_parent() {
    let single_parent_rooms = [
        HavenRoomId::Armory,
        HavenRoomId::TrainingYard,
        HavenRoomId::TrophyHall,
        HavenRoomId::Watchtower,
        HavenRoomId::AlchemyLab,
        HavenRoomId::Bedroom,
        HavenRoomId::Garden,
        HavenRoomId::Library,
        HavenRoomId::FishingDock,
        HavenRoomId::Workshop,
    ];
    for room in single_parent_rooms {
        assert_eq!(
            room.parents().len(),
            1,
            "{:?} should have exactly 1 parent, got {:?}",
            room,
            room.parents()
        );
    }
}

#[test]
fn test_child_parent_relationship_consistency() {
    // If A is a parent of B, then B should appear in A's children
    for room in HavenRoomId::ALL {
        for &parent in room.parents() {
            let parent_children = parent.children();
            assert!(
                parent_children.contains(&room),
                "{:?} is a parent of {:?} but {:?} is not in {:?}.children()",
                parent,
                room,
                room,
                parent
            );
        }
    }
}

// =========================================================================
// Construction prerequisites: can_build enforcement
// =========================================================================

#[test]
fn test_cannot_build_mid_tree_room_without_prerequisites() {
    let haven = Haven::new();
    // None of the mid-tree rooms should be buildable from a fresh Haven
    let locked_rooms = [
        HavenRoomId::Armory,
        HavenRoomId::Bedroom,
        HavenRoomId::TrainingYard,
        HavenRoomId::TrophyHall,
        HavenRoomId::Watchtower,
        HavenRoomId::AlchemyLab,
        HavenRoomId::WarRoom,
        HavenRoomId::Garden,
        HavenRoomId::Library,
        HavenRoomId::FishingDock,
        HavenRoomId::Workshop,
        HavenRoomId::Vault,
        HavenRoomId::StormForge,
    ];
    for room in locked_rooms {
        assert!(
            !haven.can_build(room),
            "{:?} should be locked in a fresh Haven",
            room
        );
    }
}

#[test]
fn test_try_build_room_deducts_correct_prestige() {
    let mut haven = Haven::new();

    // Build the entire T1 tree, verify total prestige spent
    // Expected: Hearthstone(1) + Armory(1) + Bedroom(1) + TrainingYard(2) + TrophyHall(2)
    //         + Garden(2) + Library(2) + Watchtower(2) + AlchemyLab(2)
    //         + FishingDock(2) + Workshop(2) + WarRoom(3) + Vault(3) = 27 total
    let mut prestige = 100u32;
    let initial = prestige;

    let rooms_t1_order = [
        HavenRoomId::Hearthstone,
        HavenRoomId::Armory,
        HavenRoomId::Bedroom,
        HavenRoomId::TrainingYard,
        HavenRoomId::TrophyHall,
        HavenRoomId::Garden,
        HavenRoomId::Library,
        HavenRoomId::Watchtower,
        HavenRoomId::AlchemyLab,
        HavenRoomId::FishingDock,
        HavenRoomId::Workshop,
        HavenRoomId::WarRoom,
        HavenRoomId::Vault,
    ];

    for room in rooms_t1_order {
        let result = try_build_room(room, &mut haven, &mut prestige);
        assert!(result.is_some(), "Should be able to build {:?}", room);
        let (tier, cost) = result.unwrap();
        assert_eq!(tier, 1, "{:?} should be at T1 after first build", room);
        assert_eq!(cost, tier_cost(room, 1), "{:?} cost mismatch", room);
    }

    let expected_total: u32 = rooms_t1_order.iter().map(|r| tier_cost(*r, 1)).sum();
    assert_eq!(
        initial - prestige,
        expected_total,
        "Total prestige spent should be {expected_total}"
    );
}

#[test]
fn test_can_afford_checks_prestige_rank_not_room_state() {
    let mut haven = Haven::new();
    // With exactly 1 PR, can afford Hearthstone T1 (cost 1)
    assert!(can_afford(HavenRoomId::Hearthstone, &haven, 1));
    // With 0 PR, cannot afford anything
    assert!(!can_afford(HavenRoomId::Hearthstone, &haven, 0));

    // After building Hearthstone, T2 costs 2 — need at least 2
    haven.build_room(HavenRoomId::Hearthstone);
    assert!(!can_afford(HavenRoomId::Hearthstone, &haven, 1));
    assert!(can_afford(HavenRoomId::Hearthstone, &haven, 2));
}

// =========================================================================
// Upgrade idempotency: multiple upgrades don't stack beyond max
// =========================================================================

#[test]
fn test_cannot_build_already_maxed_room() {
    let mut haven = Haven::new();
    // Max out Hearthstone (3 upgrades)
    haven.build_room(HavenRoomId::Hearthstone);
    haven.build_room(HavenRoomId::Hearthstone);
    haven.build_room(HavenRoomId::Hearthstone);
    assert_eq!(haven.room_tier(HavenRoomId::Hearthstone), 3);

    // Further builds should fail
    let result = haven.build_room(HavenRoomId::Hearthstone);
    assert!(result.is_none(), "Cannot build past max tier");
    assert_eq!(haven.room_tier(HavenRoomId::Hearthstone), 3);
    assert_eq!(haven.get_bonus(HavenBonusType::OfflineXpPercent), 100.0);
}

#[test]
fn test_storm_forge_cannot_upgrade_after_tier_1() {
    let mut haven = build_complete_haven();
    assert_eq!(haven.room_tier(HavenRoomId::StormForge), 1);

    // StormForge max tier is 1
    let result = haven.build_room(HavenRoomId::StormForge);
    assert!(result.is_none(), "StormForge cannot be upgraded past T1");
    assert_eq!(haven.room_tier(HavenRoomId::StormForge), 1);
}

// =========================================================================
// Haven serialization stability
// =========================================================================

#[test]
fn test_haven_serialize_deserialize_preserves_all_rooms() {
    let mut haven = Haven::new();
    haven.build_room(HavenRoomId::Hearthstone);
    haven.build_room(HavenRoomId::Hearthstone); // T2
    haven.build_room(HavenRoomId::Armory);
    haven.build_room(HavenRoomId::Bedroom);
    haven.discovered = true;

    let json = serde_json::to_string(&haven).unwrap();
    let loaded: Haven = serde_json::from_str(&json).unwrap();

    assert!(loaded.discovered);
    assert_eq!(loaded.room_tier(HavenRoomId::Hearthstone), 2);
    assert_eq!(loaded.room_tier(HavenRoomId::Armory), 1);
    assert_eq!(loaded.room_tier(HavenRoomId::Bedroom), 1);
    assert_eq!(loaded.room_tier(HavenRoomId::TrainingYard), 0);
    // Bonuses should be correct after reload
    let bonuses = loaded.compute_bonuses();
    assert_eq!(bonuses.offline_xp_percent, 50.0); // Hearthstone T2
    assert_eq!(bonuses.damage_percent, 5.0); // Armory T1
    assert_eq!(bonuses.hp_regen_delay_reduction, 15.0); // Bedroom T1
}

// =========================================================================
// rooms_built tracking
// =========================================================================

#[test]
fn test_rooms_built_counts_unique_rooms_not_upgrades() {
    let mut haven = Haven::new();
    assert_eq!(haven.rooms_built(), 0);

    haven.build_room(HavenRoomId::Hearthstone);
    assert_eq!(haven.rooms_built(), 1);

    // Upgrading Hearthstone doesn't add to rooms_built
    haven.build_room(HavenRoomId::Hearthstone);
    assert_eq!(haven.rooms_built(), 1);

    // Building a new room increases count
    haven.build_room(HavenRoomId::Armory);
    assert_eq!(haven.rooms_built(), 2);
}

#[test]
fn test_total_rooms_is_14() {
    let haven = Haven::new();
    assert_eq!(haven.total_rooms(), 14, "Haven should have 14 rooms total");
}

// =========================================================================
// Helper functions
// =========================================================================

fn haven_with_fishing_dock(target_tier: u8) -> Haven {
    let mut haven = Haven::new();
    haven.build_room(HavenRoomId::Hearthstone);
    haven.build_room(HavenRoomId::Bedroom);
    haven.build_room(HavenRoomId::Garden);
    for _ in 0..target_tier {
        haven.build_room(HavenRoomId::FishingDock);
    }
    haven
}

fn build_to_war_room() -> Haven {
    let mut haven = Haven::new();
    haven.build_room(HavenRoomId::Hearthstone);
    haven.build_room(HavenRoomId::Armory);
    haven.build_room(HavenRoomId::TrainingYard);
    haven.build_room(HavenRoomId::TrophyHall);
    haven.build_room(HavenRoomId::Watchtower);
    haven.build_room(HavenRoomId::AlchemyLab);
    haven
}

fn build_complete_haven() -> Haven {
    let mut haven = Haven::new();
    // Root
    haven.build_room(HavenRoomId::Hearthstone);
    // Combat branch
    haven.build_room(HavenRoomId::Armory);
    haven.build_room(HavenRoomId::TrainingYard);
    haven.build_room(HavenRoomId::TrophyHall);
    haven.build_room(HavenRoomId::Watchtower);
    haven.build_room(HavenRoomId::AlchemyLab);
    haven.build_room(HavenRoomId::WarRoom);
    // QoL branch
    haven.build_room(HavenRoomId::Bedroom);
    haven.build_room(HavenRoomId::Garden);
    haven.build_room(HavenRoomId::Library);
    haven.build_room(HavenRoomId::FishingDock);
    haven.build_room(HavenRoomId::Workshop);
    haven.build_room(HavenRoomId::Vault);
    // StormForge
    haven.build_room(HavenRoomId::StormForge);
    haven
}
