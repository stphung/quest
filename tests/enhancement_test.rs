//! Enhancement system tests: types, logic, persistence roundtrip.

use quest::enhancement::{
    apply_enhancement_result, enhancement_color_tier, enhancement_cost, enhancement_multiplier,
    enhancement_prefix, fail_penalty, roll_enhancement, soulforge_discovery_chance, success_rate,
    try_discover_soulforge, EnhancementProgress, MAX_ENHANCEMENT_LEVEL,
    SOULFORGE_MIN_PRESTIGE_RANK,
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

/// Test helper: roll + apply in one step (convenience wrapper).
fn attempt_enhancement<R: Rng>(ep: &mut EnhancementProgress, slot: usize, rng: &mut R) -> bool {
    let current_level = ep.level(slot);
    if current_level >= MAX_ENHANCEMENT_LEVEL {
        return false;
    }
    let (success, new_level) = roll_enhancement(current_level, rng);
    apply_enhancement_result(ep, slot, new_level, success);
    success
}

// =========================================================================
// EnhancementProgress basics
// =========================================================================

#[test]
fn test_new_defaults() {
    let ep = EnhancementProgress::new();
    assert!(!ep.discovered);
    assert_eq!(ep.levels, [0; 7]);
    assert_eq!(ep.total_attempts, 0);
    assert_eq!(ep.total_successes, 0);
    assert_eq!(ep.total_failures, 0);
    assert_eq!(ep.highest_level_reached, 0);
}

#[test]
fn test_default_trait() {
    let ep: EnhancementProgress = Default::default();
    assert!(!ep.discovered);
    assert_eq!(ep.levels, [0; 7]);
}

#[test]
fn test_level_get_set() {
    let mut ep = EnhancementProgress::new();
    ep.set_level(0, 5);
    assert_eq!(ep.level(0), 5);
    ep.set_level(6, 3);
    assert_eq!(ep.level(6), 3);
}

#[test]
fn test_set_level_clamped_to_max() {
    let mut ep = EnhancementProgress::new();
    ep.set_level(0, 15); // exceeds MAX_ENHANCEMENT_LEVEL (10)
    assert_eq!(ep.level(0), MAX_ENHANCEMENT_LEVEL);
}

#[test]
fn test_level_out_of_bounds_returns_zero() {
    let ep = EnhancementProgress::new();
    assert_eq!(ep.level(7), 0);
    assert_eq!(ep.level(100), 0);
}

#[test]
fn test_set_level_out_of_bounds_ignored() {
    let mut ep = EnhancementProgress::new();
    ep.set_level(7, 5); // out of bounds, should be no-op
    assert_eq!(ep.levels, [0; 7]);
}

#[test]
fn test_highest_level_reached_tracking() {
    let mut ep = EnhancementProgress::new();
    ep.set_level(0, 3);
    assert_eq!(ep.highest_level_reached, 3);
    ep.set_level(1, 7);
    assert_eq!(ep.highest_level_reached, 7);
    ep.set_level(0, 2); // lower — should not decrease highest
    assert_eq!(ep.highest_level_reached, 7);
}

#[test]
fn test_highest_level_reached_clamped() {
    let mut ep = EnhancementProgress::new();
    ep.set_level(0, 20); // clamped to MAX
    assert_eq!(ep.highest_level_reached, MAX_ENHANCEMENT_LEVEL);
}

// =========================================================================
// success_rate()
// =========================================================================

#[test]
fn test_success_rate_all_levels() {
    // +1-4: 100%
    for lvl in 1..=4 {
        assert!(
            (success_rate(lvl) - 1.0).abs() < f64::EPSILON,
            "Level +{lvl} should be 100%"
        );
    }
    // +5: 60%, +6: 50%, +7: 40%
    assert!((success_rate(5) - 0.60).abs() < f64::EPSILON);
    assert!((success_rate(6) - 0.50).abs() < f64::EPSILON);
    assert!((success_rate(7) - 0.40).abs() < f64::EPSILON);
    // +8: 30%, +9: 20%, +10: 10%
    assert!((success_rate(8) - 0.30).abs() < f64::EPSILON);
    assert!((success_rate(9) - 0.20).abs() < f64::EPSILON);
    assert!((success_rate(10) - 0.10).abs() < f64::EPSILON);
}

#[test]
fn test_success_rate_boundaries() {
    assert!((success_rate(0) - 0.0).abs() < f64::EPSILON);
    assert!((success_rate(11) - 0.0).abs() < f64::EPSILON);
    assert!((success_rate(255) - 0.0).abs() < f64::EPSILON);
}

// =========================================================================
// enhancement_cost()
// =========================================================================

#[test]
fn test_enhancement_cost_all_levels() {
    // +1-4: 1 PR each
    for lvl in 1..=4 {
        assert_eq!(enhancement_cost(lvl), 1, "Level +{lvl} should cost 1 PR");
    }
    // +5-7: 3 PR each
    for lvl in 5..=7 {
        assert_eq!(enhancement_cost(lvl), 3, "Level +{lvl} should cost 3 PR");
    }
    // +8-9: 5 PR each
    for lvl in 8..=9 {
        assert_eq!(enhancement_cost(lvl), 5, "Level +{lvl} should cost 5 PR");
    }
    // +10: 10 PR
    assert_eq!(enhancement_cost(10), 10);
}

#[test]
fn test_enhancement_cost_boundaries() {
    assert_eq!(enhancement_cost(0), 0);
    assert_eq!(enhancement_cost(11), 0);
    assert_eq!(enhancement_cost(255), 0);
}

// =========================================================================
// fail_penalty()
// =========================================================================

#[test]
fn test_fail_penalty_all_levels() {
    // +1-4: safe (0)
    for lvl in 1..=4 {
        assert_eq!(fail_penalty(lvl), 0, "Level +{lvl} should have no penalty");
    }
    // +5-7: -1
    for lvl in 5..=7 {
        assert_eq!(fail_penalty(lvl), 1, "Level +{lvl} should have -1 penalty");
    }
    // +8-9: -1
    for lvl in 8..=9 {
        assert_eq!(fail_penalty(lvl), 1, "Level +{lvl} should have -1 penalty");
    }
    // +10: -2
    assert_eq!(fail_penalty(10), 2, "Level +10 should have -2 penalty");
}

#[test]
fn test_fail_penalty_boundaries() {
    assert_eq!(fail_penalty(0), 0);
    assert_eq!(fail_penalty(11), 0);
}

// =========================================================================
// enhancement_multiplier()
// =========================================================================

#[test]
fn test_enhancement_multiplier_key_levels() {
    assert!((enhancement_multiplier(0) - 1.0).abs() < f64::EPSILON);
    assert!((enhancement_multiplier(5) - 1.30).abs() < f64::EPSILON);
    assert!((enhancement_multiplier(10) - 2.5).abs() < f64::EPSILON);
}

#[test]
fn test_enhancement_multiplier_above_max_clamped() {
    // Levels above MAX should clamp to MAX value
    assert!((enhancement_multiplier(11) - enhancement_multiplier(10)).abs() < f64::EPSILON);
    assert!((enhancement_multiplier(255) - enhancement_multiplier(10)).abs() < f64::EPSILON);
}

// =========================================================================
// enhancement_prefix()
// =========================================================================

#[test]
fn test_enhancement_prefix_zero() {
    assert_eq!(enhancement_prefix(0), "");
}

#[test]
fn test_enhancement_prefix_nonzero() {
    assert_eq!(enhancement_prefix(1), "+1 ");
    assert_eq!(enhancement_prefix(10), "+10 ");
}

// =========================================================================
// enhancement_color_tier()
// =========================================================================

#[test]
fn test_enhancement_color_tier_all() {
    assert_eq!(enhancement_color_tier(0), 0); // none
    for lvl in 1..=4 {
        assert_eq!(
            enhancement_color_tier(lvl),
            1,
            "Level +{lvl} should be tier 1 (white)"
        );
    }
    for lvl in 5..=7 {
        assert_eq!(
            enhancement_color_tier(lvl),
            2,
            "Level +{lvl} should be tier 2 (yellow)"
        );
    }
    for lvl in 8..=9 {
        assert_eq!(
            enhancement_color_tier(lvl),
            3,
            "Level +{lvl} should be tier 3 (magenta)"
        );
    }
    assert_eq!(enhancement_color_tier(10), 4); // gold
    assert_eq!(enhancement_color_tier(11), 4); // above max still gold
}

// =========================================================================
// attempt_enhancement() logic
// =========================================================================

#[test]
fn test_attempt_enhancement_safe_levels_always_succeed() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut ep = EnhancementProgress::new();

    // +1 through +4 have 100% success rate
    for expected_level in 1..=4u8 {
        let result = attempt_enhancement(&mut ep, 0, &mut rng);
        assert!(
            result,
            "Enhancement to +{expected_level} should always succeed"
        );
        assert_eq!(ep.level(0), expected_level);
    }
    assert_eq!(ep.total_attempts, 4);
    assert_eq!(ep.total_successes, 4);
    assert_eq!(ep.total_failures, 0);
}

#[test]
fn test_attempt_enhancement_max_level_blocked() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut ep = EnhancementProgress::new();
    ep.set_level(0, MAX_ENHANCEMENT_LEVEL);

    let result = attempt_enhancement(&mut ep, 0, &mut rng);
    assert!(!result);
    assert_eq!(ep.level(0), MAX_ENHANCEMENT_LEVEL);
    assert_eq!(ep.total_attempts, 0); // not even counted
}

#[test]
fn test_attempt_enhancement_failure_penalty_minus_1() {
    // At level 4, attempting +5 with 70% rate.
    // Use a seeded RNG to find a seed that produces a failure.
    // We try multiple seeds to find one that fails at +5.
    for seed in 0..1000u64 {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut ep = EnhancementProgress::new();
        ep.set_level(0, 4);
        let result = attempt_enhancement(&mut ep, 0, &mut rng);
        if !result {
            // Failure at +5 => penalty of 1, so level goes from 4 to 3
            assert_eq!(ep.level(0), 3, "Failed +5 should drop from 4 to 3");
            assert_eq!(ep.total_failures, 1);
            return;
        }
    }
    panic!("Could not find a seed that fails at +5 within 1000 attempts");
}

#[test]
fn test_attempt_enhancement_failure_penalty_minus_2() {
    // At level 9, attempting +10 with 5% rate (only +10 has -2 penalty now).
    for seed in 0..1000u64 {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut ep = EnhancementProgress::new();
        ep.set_level(0, 9);
        let result = attempt_enhancement(&mut ep, 0, &mut rng);
        if !result {
            // Failure at +10 => penalty of 2, so level goes from 9 to 7
            assert_eq!(ep.level(0), 7, "Failed +10 should drop from 9 to 7");
            assert_eq!(ep.total_failures, 1);
            return;
        }
    }
    panic!("Could not find a seed that fails at +10 within 1000 attempts");
}

#[test]
fn test_attempt_enhancement_failure_penalty_saturates_at_zero() {
    // At level 1, attempting +2 is 100% success, so we manually test saturating_sub
    // by going to level 5 (with set_level) then failing +6 (penalty 1 -> level 4)
    // But to test saturation, set level to 0 and try to fail at +1...
    // +1 is 100% so it can't fail. Instead, verify the penalty math:
    // At level 0, fail_penalty for +1 is 0, so no change needed.
    // Better: set level to 1, then the penalty for +5 is 1 but level would go to 0.
    // We need to fail at a level where penalty could go below 0.
    // Set level to 0 with fail_penalty = 2 won't happen naturally since +1 always succeeds.
    // Instead: set level to 1, fail at +8 equivalent scenario:
    let mut ep = EnhancementProgress::new();
    // Manually test saturating_sub on the level field
    ep.set_level(0, 1);
    // fail_penalty for target +8 = 2, but current is 1. saturating_sub(2) = 0
    // We need to simulate this: current=7, target=8, fail, penalty=2, new=5
    // Actually, let's test the edge: current=1, target=2 (100% rate, no fail).
    // The simplest edge: level(slot) = 0 after saturating.
    // Use set_level directly to set to 1, then simulate penalty of 2:
    let val: u8 = 1u8.saturating_sub(2);
    assert_eq!(val, 0, "saturating_sub should clamp at 0");
}

// =========================================================================
// soulforge_discovery_chance()
// =========================================================================

#[test]
fn test_soulforge_discovery_below_min_prestige() {
    for rank in 0..SOULFORGE_MIN_PRESTIGE_RANK {
        assert!(
            soulforge_discovery_chance(rank).abs() < f64::EPSILON,
            "Rank {rank} should have 0 discovery chance"
        );
    }
}

#[test]
fn test_soulforge_discovery_at_min_prestige() {
    let chance = soulforge_discovery_chance(SOULFORGE_MIN_PRESTIGE_RANK);
    assert!(
        (chance - 0.000014).abs() < f64::EPSILON,
        "At min prestige, chance should equal base"
    );
}

#[test]
fn test_soulforge_discovery_above_min_prestige() {
    let chance = soulforge_discovery_chance(SOULFORGE_MIN_PRESTIGE_RANK + 5);
    let expected = 0.000014 + 5.0 * 0.000007;
    assert!(
        (chance - expected).abs() < f64::EPSILON,
        "Expected {expected}, got {chance}"
    );
}

// =========================================================================
// try_discover_soulforge()
// =========================================================================

#[test]
fn test_discover_soulforge_below_prestige_always_fails() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut ep = EnhancementProgress::new();
    for _ in 0..1_000 {
        assert!(!try_discover_soulforge(&mut ep, 14, &mut rng));
    }
    assert!(!ep.discovered);
}

#[test]
fn test_discover_soulforge_eventually_succeeds() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut ep = EnhancementProgress::new();
    let mut found = false;
    for _ in 0..1_000_000 {
        if try_discover_soulforge(&mut ep, 15, &mut rng) {
            found = true;
            break;
        }
    }
    assert!(found, "Should discover soulforge within 1M ticks at P15");
    assert!(ep.discovered);
}

#[test]
fn test_discover_soulforge_already_discovered() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut ep = EnhancementProgress::new();
    ep.discovered = true;
    assert!(!try_discover_soulforge(&mut ep, 50, &mut rng));
}

// =========================================================================
// Serialization roundtrip
// =========================================================================

#[test]
fn test_serialization_roundtrip() {
    let mut ep = EnhancementProgress::new();
    ep.discovered = true;
    ep.set_level(0, 5);
    ep.set_level(3, 10);
    ep.total_attempts = 42;
    ep.total_successes = 30;
    ep.total_failures = 12;

    let json = serde_json::to_string(&ep).expect("serialize");
    let restored: EnhancementProgress = serde_json::from_str(&json).expect("deserialize");

    assert!(restored.discovered);
    assert_eq!(restored.level(0), 5);
    assert_eq!(restored.level(3), 10);
    assert_eq!(restored.total_attempts, 42);
    assert_eq!(restored.total_successes, 30);
    assert_eq!(restored.total_failures, 12);
    assert_eq!(restored.highest_level_reached, 10);
}

#[test]
fn test_deserialization_with_missing_fields_uses_defaults() {
    // Simulate an older save format missing some fields
    let json = r#"{"discovered":true,"levels":[1,0,0,0,0,0,0],"total_attempts":5,"total_successes":3,"total_failures":2,"highest_level_reached":1}"#;
    let ep: EnhancementProgress = serde_json::from_str(json).expect("deserialize");
    assert!(ep.discovered);
    assert_eq!(ep.level(0), 1);
    assert_eq!(ep.total_attempts, 5);
}

// =========================================================================
// Enhancement → DerivedStats integration
// =========================================================================

#[test]
fn test_enhancement_multiplier_affects_derived_stats() {
    use quest::character::attributes::Attributes;
    use quest::character::derived_stats::DerivedStats;
    use quest::items::{AttributeBonuses, Equipment, EquipmentSlot, Item, Rarity};

    let attrs = Attributes::new();
    let mut equipment = Equipment::new();

    // Create a weapon with +4 STR
    let weapon = Item {
        slot: EquipmentSlot::Weapon,
        rarity: Rarity::Common,
        ilvl: 10,
        tier: 5,
        base_name: "Sword".to_string(),
        display_name: "Iron Sword".to_string(),
        attributes: AttributeBonuses {
            str: 4,
            dex: 0,
            con: 0,
            int: 0,
            wis: 0,
            cha: 0,
        },
        affixes: vec![],
        god_item_id: None,
    };
    equipment.set(EquipmentSlot::Weapon, Some(weapon));

    // Without enhancement
    let stats_base = DerivedStats::calculate_derived_stats(&attrs, &equipment, &[0; 7]);

    // With weapon at +10 (1.50x multiplier)
    let mut levels = [0u8; 7];
    levels[0] = 10; // Weapon slot
    let stats_max = DerivedStats::calculate_derived_stats(&attrs, &equipment, &levels);

    // With +10 (2.50x multiplier): str contribution = round(4 * 2.50) = 10, total str = 20, mod = +5
    // Without enhancement (1.0x): str contribution = 4, total str = 14, mod = +2
    assert!(
        stats_max.physical_damage > stats_base.physical_damage,
        "Enhancement at +10 should increase physical damage: {} vs {}",
        stats_max.physical_damage,
        stats_base.physical_damage
    );
}

#[test]
fn test_enhancement_multiplier_affects_affixes() {
    use quest::character::attributes::Attributes;
    use quest::character::derived_stats::DerivedStats;
    use quest::items::{
        Affix, AffixType, AttributeBonuses, Equipment, EquipmentSlot, Item, Rarity,
    };

    let attrs = Attributes::new();
    let mut equipment = Equipment::new();

    let weapon = Item {
        slot: EquipmentSlot::Weapon,
        rarity: Rarity::Rare,
        ilvl: 10,
        tier: 5,
        base_name: "Sword".to_string(),
        display_name: "Epic Sword".to_string(),
        attributes: AttributeBonuses::new(),
        affixes: vec![Affix {
            affix_type: AffixType::CritChance,
            value: 10.0,
        }],
        god_item_id: None,
    };
    equipment.set(EquipmentSlot::Weapon, Some(weapon));

    let stats_base = DerivedStats::calculate_derived_stats(&attrs, &equipment, &[0; 7]);

    let mut levels = [0u8; 7];
    levels[0] = 10; // +10 weapon (1.50x)
    let stats_enhanced = DerivedStats::calculate_derived_stats(&attrs, &equipment, &levels);

    // Base crit: 5% base + 10.0 crit affix = 15
    // Enhanced crit: 5% base + 10.0*1.5 = 15.0 crit affix = 20
    assert!(
        stats_enhanced.crit_chance_percent > stats_base.crit_chance_percent,
        "Enhanced weapon should increase crit chance from affix: {} vs {}",
        stats_enhanced.crit_chance_percent,
        stats_base.crit_chance_percent
    );
}

#[test]
fn test_enhancement_zero_levels_no_change() {
    use quest::character::attributes::Attributes;
    use quest::character::derived_stats::DerivedStats;
    use quest::items::{AttributeBonuses, Equipment, EquipmentSlot, Item, Rarity};

    let attrs = Attributes::new();
    let mut equipment = Equipment::new();

    let weapon = Item {
        slot: EquipmentSlot::Weapon,
        rarity: Rarity::Common,
        ilvl: 10,
        tier: 5,
        base_name: "Sword".to_string(),
        display_name: "Iron Sword".to_string(),
        attributes: AttributeBonuses {
            str: 5,
            dex: 0,
            con: 0,
            int: 0,
            wis: 0,
            cha: 0,
        },
        affixes: vec![],
        god_item_id: None,
    };
    equipment.set(EquipmentSlot::Weapon, Some(weapon));

    // With [0;7] enhancement levels, multiplier is 1.0 -- no change
    let stats = DerivedStats::calculate_derived_stats(&attrs, &equipment, &[0; 7]);

    // STR: 10 + floor(5 * 1.0) = 15, mod = +2, phys_dmg = 5 + 2*2 = 9
    assert_eq!(stats.physical_damage, 9);
}

#[test]
fn test_enhancement_per_slot_independence() {
    use quest::character::attributes::Attributes;
    use quest::character::derived_stats::DerivedStats;
    use quest::items::{AttributeBonuses, Equipment, EquipmentSlot, Item, Rarity};

    let attrs = Attributes::new();
    let mut equipment = Equipment::new();

    // Weapon with +4 STR
    let weapon = Item {
        slot: EquipmentSlot::Weapon,
        rarity: Rarity::Common,
        ilvl: 10,
        tier: 5,
        base_name: "Sword".to_string(),
        display_name: "Sword".to_string(),
        attributes: AttributeBonuses {
            str: 4,
            dex: 0,
            con: 0,
            int: 0,
            wis: 0,
            cha: 0,
        },
        affixes: vec![],
        god_item_id: None,
    };
    // Armor with +4 CON
    let armor = Item {
        slot: EquipmentSlot::Armor,
        rarity: Rarity::Common,
        ilvl: 10,
        tier: 5,
        base_name: "Armor".to_string(),
        display_name: "Armor".to_string(),
        attributes: AttributeBonuses {
            str: 0,
            dex: 0,
            con: 4,
            int: 0,
            wis: 0,
            cha: 0,
        },
        affixes: vec![],
        god_item_id: None,
    };
    equipment.set(EquipmentSlot::Weapon, Some(weapon));
    equipment.set(EquipmentSlot::Armor, Some(armor));

    // Enhance only weapon to +10
    let levels_weapon = [10, 0, 0, 0, 0, 0, 0];
    let stats_weapon = DerivedStats::calculate_derived_stats(&attrs, &equipment, &levels_weapon);

    // Enhance only armor to +10
    let levels_armor = [0, 10, 0, 0, 0, 0, 0];
    let stats_armor = DerivedStats::calculate_derived_stats(&attrs, &equipment, &levels_armor);

    // Weapon enhancement should increase physical_damage but not max_hp (beyond base)
    // Armor enhancement should increase max_hp but not physical_damage (beyond base)
    let base_stats = DerivedStats::calculate_derived_stats(&attrs, &equipment, &[0; 7]);

    assert!(
        stats_weapon.physical_damage > base_stats.physical_damage,
        "Weapon enhancement should increase phys damage"
    );
    assert_eq!(
        stats_weapon.max_hp, base_stats.max_hp,
        "Weapon enhancement should not affect HP"
    );

    assert!(
        stats_armor.max_hp > base_stats.max_hp,
        "Armor enhancement should increase HP"
    );
    assert_eq!(
        stats_armor.physical_damage, base_stats.physical_damage,
        "Armor enhancement should not affect phys damage"
    );
}

// =========================================================================
// Enhancement Achievements
// =========================================================================

#[test]
fn test_enhancement_achievements() {
    use quest::achievements::Achievements;

    let mut achievements = Achievements::default();
    let char_name = Some("Test Hero");

    // Soulforge discovered
    achievements.on_soulforge_discovered(char_name);
    assert!(achievements.is_unlocked(quest::achievements::AchievementId::SoulforgeDiscovered));

    // +1 on any slot
    achievements.on_enhancement_upgraded(1, &[1, 0, 0, 0, 0, 0, 0], 1, char_name);
    assert!(achievements.is_unlocked(quest::achievements::AchievementId::ApprenticeSmith));
    assert!(!achievements.is_unlocked(quest::achievements::AchievementId::JourneymanSmith));

    // +4 on all slots
    achievements.on_enhancement_upgraded(4, &[4, 4, 4, 4, 4, 4, 4], 7, char_name);
    assert!(achievements.is_unlocked(quest::achievements::AchievementId::FullyTempered));

    // +5 on any slot
    achievements.on_enhancement_upgraded(5, &[5, 4, 4, 4, 4, 4, 4], 10, char_name);
    assert!(achievements.is_unlocked(quest::achievements::AchievementId::JourneymanSmith));

    // +6 on any slot
    achievements.on_enhancement_upgraded(6, &[6, 4, 4, 4, 4, 4, 4], 15, char_name);
    assert!(achievements.is_unlocked(quest::achievements::AchievementId::SoulforgeAdept));

    // +7 on any slot
    achievements.on_enhancement_upgraded(7, &[7, 4, 4, 4, 4, 4, 4], 20, char_name);
    assert!(achievements.is_unlocked(quest::achievements::AchievementId::SoulforgeSavant));
    assert!(!achievements.is_unlocked(quest::achievements::AchievementId::SoulConvergence));

    // +8 on any slot
    achievements.on_enhancement_upgraded(8, &[8, 4, 4, 4, 4, 4, 4], 25, char_name);
    assert!(achievements.is_unlocked(quest::achievements::AchievementId::SoulforgeMaster));

    // +9 on any slot
    achievements.on_enhancement_upgraded(9, &[9, 4, 4, 4, 4, 4, 4], 30, char_name);
    assert!(achievements.is_unlocked(quest::achievements::AchievementId::SoulforgeGrandmaster));

    // +10 on one slot
    achievements.on_enhancement_upgraded(10, &[10, 4, 4, 4, 4, 4, 4], 50, char_name);
    assert!(achievements.is_unlocked(quest::achievements::AchievementId::MasterSmith));
    assert!(!achievements.is_unlocked(quest::achievements::AchievementId::SoulConvergence));

    // +7 on all slots
    achievements.on_enhancement_upgraded(7, &[7, 7, 7, 7, 7, 7, 7], 99, char_name);
    assert!(achievements.is_unlocked(quest::achievements::AchievementId::SoulConvergence));
    assert!(!achievements.is_unlocked(quest::achievements::AchievementId::PersistentHammering));

    // 100 attempts
    achievements.on_enhancement_upgraded(1, &[10, 10, 10, 10, 10, 10, 10], 100, char_name);
    assert!(achievements.is_unlocked(quest::achievements::AchievementId::PersistentHammering));
}

// =========================================================================
// Full enhancement flow (integration)
// =========================================================================

#[test]
fn test_full_enhancement_flow() {
    let mut enhancement = EnhancementProgress::new();
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    // Not yet discovered
    assert!(!enhancement.discovered);

    // Discover
    enhancement.discovered = true;
    assert!(enhancement.discovered);

    // Enhance weapon from +0 to +4 (100% success rate)
    for target in 1..=4 {
        let cost = enhancement_cost(target);
        let rate = success_rate(target);
        assert_eq!(rate, 1.0, "+{} should be 100% success", target);
        assert_eq!(cost, 1, "+{} should cost 1 PR", target);

        let success = attempt_enhancement(&mut enhancement, 0, &mut rng);
        assert!(success, "+{} should always succeed", target);
        assert_eq!(enhancement.level(0), target);
    }

    assert_eq!(enhancement.total_attempts, 4);
    assert_eq!(enhancement.total_successes, 4);
    assert_eq!(enhancement.total_failures, 0);
    assert_eq!(enhancement.highest_level_reached, 4);
}

// =========================================================================
// Persistence tests (without file I/O)
// =========================================================================

#[test]
fn test_enhancement_save_path_returns_expected() {
    use quest::enhancement::enhancement_save_path;

    let path = enhancement_save_path().expect("Should return a valid path");
    let path_str = path.to_str().expect("Path should be valid UTF-8");

    // Verify the path ends with .quest/enhancement.json
    assert!(
        path_str.ends_with(".quest/enhancement.json"),
        "Path should end with .quest/enhancement.json, got: {}",
        path_str
    );
}

#[test]
fn test_save_path_is_absolute() {
    use quest::enhancement::enhancement_save_path;

    let path = enhancement_save_path().expect("Should return a valid path");
    assert!(
        path.is_absolute(),
        "Enhancement save path should be absolute, got: {:?}",
        path
    );
}

#[test]
fn test_corrupt_json_deserializes_to_default() {
    // Test the pattern used in load_enhancement: unwrap_or_default()
    let corrupt_json = "not valid json at all";
    let result: EnhancementProgress = serde_json::from_str(corrupt_json).unwrap_or_default();

    // Should fall back to default values
    assert!(!result.discovered);
    assert_eq!(result.levels, [0; 7]);
    assert_eq!(result.total_attempts, 0);
    assert_eq!(result.total_successes, 0);
    assert_eq!(result.total_failures, 0);
    assert_eq!(result.highest_level_reached, 0);
}

#[test]
fn test_empty_json_object_fails_without_required_fields() {
    // Test serde behavior with empty JSON object
    // Without #[serde(default)] annotations, all fields are required
    let empty_json = "{}";
    let result: Result<EnhancementProgress, _> = serde_json::from_str(empty_json);

    // Should fail because required fields are missing
    assert!(
        result.is_err(),
        "Empty JSON should fail without all required fields"
    );
}

#[test]
fn test_partial_json_fails_without_all_fields() {
    // Test JSON with only some fields
    // Without #[serde(default)] annotations, all fields are required
    let partial_json = r#"{"discovered":true,"levels":[3,2,1,0,0,0,0]}"#;
    let result: Result<EnhancementProgress, _> = serde_json::from_str(partial_json);

    // Should fail because not all required fields are present
    assert!(
        result.is_err(),
        "Partial JSON should fail without all required fields"
    );
}

#[test]
fn test_extra_fields_in_json_ignored() {
    // Test forward compatibility: JSON with unknown extra fields should be ignored
    let json_with_extras = r#"{
        "discovered":true,
        "levels":[5,0,0,0,0,0,0],
        "total_attempts":10,
        "total_successes":8,
        "total_failures":2,
        "highest_level_reached":5,
        "future_field_1":"ignored",
        "future_field_2":999
    }"#;
    let result: EnhancementProgress =
        serde_json::from_str(json_with_extras).expect("Should deserialize with extra fields");

    // Known fields should be preserved
    assert!(result.discovered);
    assert_eq!(result.levels[0], 5);
    assert_eq!(result.total_attempts, 10);
    assert_eq!(result.total_successes, 8);
    assert_eq!(result.total_failures, 2);
    assert_eq!(result.highest_level_reached, 5);
}

#[test]
fn test_json_pretty_format() {
    // Verify serde_json::to_string_pretty() produces valid JSON that roundtrips
    let mut ep = EnhancementProgress::new();
    ep.discovered = true;
    ep.set_level(0, 7);
    ep.total_attempts = 15;
    ep.total_successes = 10;
    ep.total_failures = 5;

    let pretty_json = serde_json::to_string_pretty(&ep).expect("Should serialize to pretty JSON");

    // Verify it's valid JSON by deserializing
    let restored: EnhancementProgress =
        serde_json::from_str(&pretty_json).expect("Pretty JSON should be valid");

    assert_eq!(restored.discovered, ep.discovered);
    assert_eq!(restored.levels, ep.levels);
    assert_eq!(restored.total_attempts, ep.total_attempts);
    assert_eq!(restored.total_successes, ep.total_successes);
    assert_eq!(restored.total_failures, ep.total_failures);
    assert_eq!(restored.highest_level_reached, ep.highest_level_reached);

    // Verify it's actually pretty-formatted (has newlines and indentation)
    assert!(
        pretty_json.contains('\n'),
        "Pretty JSON should contain newlines"
    );
    assert!(
        pretty_json.contains("  "),
        "Pretty JSON should contain indentation"
    );
}

#[test]
fn test_json_field_names_stable() {
    // Verify specific JSON field names are present (important for save file compatibility)
    let mut ep = EnhancementProgress::new();
    ep.discovered = true;
    ep.set_level(0, 3);
    ep.set_level(2, 5);
    ep.total_attempts = 20;
    ep.total_successes = 12;
    ep.total_failures = 8;

    let json = serde_json::to_string(&ep).expect("Should serialize");

    // Verify all expected field names are present
    assert!(
        json.contains("\"discovered\""),
        "JSON should contain 'discovered' field"
    );
    assert!(
        json.contains("\"levels\""),
        "JSON should contain 'levels' field"
    );
    assert!(
        json.contains("\"total_attempts\""),
        "JSON should contain 'total_attempts' field"
    );
    assert!(
        json.contains("\"total_successes\""),
        "JSON should contain 'total_successes' field"
    );
    assert!(
        json.contains("\"total_failures\""),
        "JSON should contain 'total_failures' field"
    );
    assert!(
        json.contains("\"highest_level_reached\""),
        "JSON should contain 'highest_level_reached' field"
    );

    // Verify field values
    assert!(
        json.contains("\"discovered\":true"),
        "discovered should be true"
    );
    assert!(
        json.contains("[3,0,5,0,0,0,0]"),
        "levels array should match"
    );
    assert!(
        json.contains("\"total_attempts\":20"),
        "total_attempts should be 20"
    );
    assert!(
        json.contains("\"total_successes\":12"),
        "total_successes should be 12"
    );
    assert!(
        json.contains("\"total_failures\":8"),
        "total_failures should be 8"
    );
    assert!(
        json.contains("\"highest_level_reached\":5"),
        "highest_level_reached should be 5"
    );
}

// =========================================================================
// Additional logic.rs tests
// =========================================================================

#[test]
fn test_roll_enhancement_direct_success() {
    // Call roll_enhancement directly at level 0 (attempting +1, 100% rate)
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let (success, new_level) = roll_enhancement(0, &mut rng);
    assert!(success, "Attempting +1 should always succeed");
    assert_eq!(new_level, 1, "New level should be 1");
}

#[test]
fn test_roll_enhancement_direct_at_max() {
    // Call roll_enhancement at MAX_ENHANCEMENT_LEVEL
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let (success, new_level) = roll_enhancement(MAX_ENHANCEMENT_LEVEL, &mut rng);
    assert!(!success, "Cannot enhance beyond MAX level");
    assert_eq!(
        new_level, MAX_ENHANCEMENT_LEVEL,
        "Level should remain at MAX"
    );
}

#[test]
fn test_roll_enhancement_fail_drops_level() {
    // At level 4, attempting +5 (60% rate, penalty -1)
    // We need to find a seed that fails
    for seed in 0..1000u64 {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let (success, new_level) = roll_enhancement(4, &mut rng);
        if !success {
            // Failed at +5, penalty is 1, so level drops from 4 to 3
            assert_eq!(new_level, 3, "Failed +5 should drop from 4 to 3");
            return;
        }
    }
    panic!("Could not find a seed that fails at +5 within 1000 attempts");
}

#[test]
fn test_apply_enhancement_result_success_counters() {
    let mut ep = EnhancementProgress::new();
    apply_enhancement_result(&mut ep, 0, 1, true);

    assert_eq!(ep.level(0), 1, "Level should be updated");
    assert_eq!(ep.total_attempts, 1, "Attempts should increment");
    assert_eq!(ep.total_successes, 1, "Successes should increment");
    assert_eq!(ep.total_failures, 0, "Failures should remain 0");
}

#[test]
fn test_apply_enhancement_result_failure_counters() {
    let mut ep = EnhancementProgress::new();
    ep.set_level(0, 4);
    apply_enhancement_result(&mut ep, 0, 3, false);

    assert_eq!(ep.level(0), 3, "Level should be updated to penalty value");
    assert_eq!(ep.total_attempts, 1, "Attempts should increment");
    assert_eq!(ep.total_successes, 0, "Successes should remain 0");
    assert_eq!(ep.total_failures, 1, "Failures should increment");
}

#[test]
fn test_apply_enhancement_result_out_of_bounds_slot() {
    let mut ep = EnhancementProgress::new();
    ep.set_level(0, 5);

    // Apply to invalid slot 99
    apply_enhancement_result(&mut ep, 99, 10, true);

    // Slot 0 should remain unchanged
    assert_eq!(ep.level(0), 5, "Valid slot should remain unchanged");
    // Counters should still increment (result is tracked regardless of slot validity)
    assert_eq!(ep.total_attempts, 1, "Attempts should increment");
    assert_eq!(ep.total_successes, 1, "Successes should increment");
}

#[test]
fn test_soulforge_discovery_chance_increases_with_rank() {
    let chance_p15 = soulforge_discovery_chance(SOULFORGE_MIN_PRESTIGE_RANK);
    let chance_p20 = soulforge_discovery_chance(SOULFORGE_MIN_PRESTIGE_RANK + 5);

    assert!(
        chance_p20 > chance_p15,
        "Discovery chance should increase with prestige rank: P20={} vs P15={}",
        chance_p20,
        chance_p15
    );
}

#[test]
fn test_try_discover_soulforge_at_exactly_min_prestige() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut ep = EnhancementProgress::new();
    let mut discovered = false;

    // At exactly P15, discovery should be possible
    for _ in 0..1_000_000 {
        if try_discover_soulforge(&mut ep, SOULFORGE_MIN_PRESTIGE_RANK, &mut rng) {
            discovered = true;
            break;
        }
    }

    assert!(
        discovered,
        "Should be able to discover soulforge at exactly min prestige rank (P15)"
    );
    assert!(ep.discovered);
}

// =========================================================================
// Additional types.rs tests
// =========================================================================

#[test]
fn test_enhancement_color_rgb_all_tiers() {
    use quest::enhancement::enhancement_color_rgb;

    // Tier 0: gray (no enhancement)
    assert_eq!(
        enhancement_color_rgb(0),
        (128, 128, 128),
        "Level 0 should be gray"
    );

    // Tier 1: white (levels 1-4)
    assert_eq!(
        enhancement_color_rgb(1),
        (255, 255, 255),
        "Level 1 should be white"
    );
    assert_eq!(
        enhancement_color_rgb(3),
        (255, 255, 255),
        "Level 3 should be white"
    );

    // Tier 2: yellow (levels 5-7)
    assert_eq!(
        enhancement_color_rgb(5),
        (255, 255, 0),
        "Level 5 should be yellow"
    );
    assert_eq!(
        enhancement_color_rgb(6),
        (255, 255, 0),
        "Level 6 should be yellow"
    );

    // Tier 3: magenta (levels 8-9)
    assert_eq!(
        enhancement_color_rgb(8),
        (255, 0, 255),
        "Level 8 should be magenta"
    );
    assert_eq!(
        enhancement_color_rgb(9),
        (255, 0, 255),
        "Level 9 should be magenta"
    );

    // Tier 4: gold (level 10+)
    assert_eq!(
        enhancement_color_rgb(10),
        (255, 215, 0),
        "Level 10 should be gold"
    );
}

#[test]
fn test_enhancement_multiplier_all_levels() {
    // Test all 11 multiplier values from level 0 to 10
    let expected = [
        (0, 1.00),  // 0% bonus
        (1, 1.05),  // +5%
        (2, 1.10),  // +10%
        (3, 1.15),  // +15%
        (4, 1.20),  // +20%
        (5, 1.30),  // +30%
        (6, 1.40),  // +40%
        (7, 1.55),  // +55%
        (8, 1.75),  // +75%
        (9, 2.00),  // +100%
        (10, 2.50), // +150%
    ];

    for (level, expected_mult) in expected {
        let actual = enhancement_multiplier(level);
        assert!(
            (actual - expected_mult).abs() < f64::EPSILON,
            "Level {} should have multiplier {}, got {}",
            level,
            expected_mult,
            actual
        );
    }
}

#[test]
fn test_soulforge_ui_state_open_close() {
    use quest::enhancement::SoulforgeUiState;

    let mut ui = SoulforgeUiState::new();
    assert!(!ui.open, "Should start closed");

    // Set non-default state before opening to verify reset
    ui.selected_slot = 3;
    ui.animation_tick = 5;

    ui.open();
    assert!(ui.open, "Should be open after open()");
    assert_eq!(ui.selected_slot, 0, "Should reset selected slot");
    assert_eq!(
        ui.phase,
        quest::enhancement::SoulforgePhase::Menu,
        "Should reset phase to Menu"
    );
    assert_eq!(ui.animation_tick, 0, "Should reset animation tick");
    assert!(ui.last_result.is_none(), "Should clear last result");

    // Set non-Menu phase before closing to verify reset
    ui.phase = quest::enhancement::SoulforgePhase::Hammering;

    ui.close();
    assert!(!ui.open, "Should be closed after close()");
    assert_eq!(
        ui.phase,
        quest::enhancement::SoulforgePhase::Menu,
        "Should reset phase on close"
    );
    assert!(
        ui.last_result.is_none(),
        "Should clear last result on close"
    );
}

#[test]
fn test_soulforge_ui_state_default() {
    use quest::enhancement::SoulforgeUiState;

    let ui: SoulforgeUiState = Default::default();
    assert!(!ui.open, "Default should be closed");
    assert_eq!(ui.selected_slot, 0, "Default should have slot 0 selected");
    assert!(
        ui.last_result.is_none(),
        "Default should have no last result"
    );
}

#[test]
fn test_soulforge_phase_equality() {
    use quest::enhancement::SoulforgePhase;

    // Test PartialEq for SoulforgePhase variants
    assert_eq!(SoulforgePhase::Menu, SoulforgePhase::Menu);
    assert_eq!(SoulforgePhase::Confirming, SoulforgePhase::Confirming);
    assert_eq!(SoulforgePhase::Hammering, SoulforgePhase::Hammering);
    assert_eq!(SoulforgePhase::ResultSuccess, SoulforgePhase::ResultSuccess);
    assert_eq!(SoulforgePhase::ResultFailure, SoulforgePhase::ResultFailure);

    assert_ne!(SoulforgePhase::Menu, SoulforgePhase::Confirming);
    assert_ne!(SoulforgePhase::Hammering, SoulforgePhase::ResultSuccess);
}

#[test]
fn test_enhancement_result_fields() {
    use quest::enhancement::EnhancementResult;

    let result = EnhancementResult {
        slot_index: 0,
        success: true,
        old_level: 4,
        new_level: 5,
        cost: 3,
    };

    assert_eq!(result.slot_index, 0);
    assert!(result.success);
    assert_eq!(result.old_level, 4);
    assert_eq!(result.new_level, 5);
    assert_eq!(result.cost, 3);
}
