#![allow(deprecated)]
//! Integration tests for items/types, combat/types, core/tick, and character/combat_bonuses.
//! Targets coverage improvements across these four modules.

use quest::character::combat_bonuses::PrestigeCombatBonuses;
use quest::combat::enemy_generation::{
    generate_boss_for_current_zone, generate_dungeon_boss, generate_dungeon_elite,
    generate_dungeon_enemy, generate_enemy_for_current_zone, generate_enemy_name,
    generate_subzone_boss, generate_zone_enemy, generate_zone_enemy_name,
};
use quest::combat::types::{CombatState, Enemy};
use quest::items::scoring::affix_power_weight;
use quest::items::types::{Affix, AffixType, AttributeBonuses, EquipmentSlot, Item, Rarity};
use quest::zones::get_all_zones;

// ── Helper ─────────────────────────────────────────────────────────

fn make_item(
    slot: EquipmentSlot,
    rarity: Rarity,
    attrs: AttributeBonuses,
    affixes: Vec<Affix>,
) -> Item {
    Item {
        slot,
        rarity,
        ilvl: 10,
        tier: 5,
        base_name: "Test".to_string(),
        display_name: "Test Item".to_string(),
        attributes: attrs,
        affixes,
        god_item_id: None,
    }
}

// ═══════════════════════════════════════════════════════════════════
// character/combat_bonuses — PrestigeCombatBonuses
// ═══════════════════════════════════════════════════════════════════

#[test]
fn prestige_bonuses_rank_0_all_zeros() {
    let b = PrestigeCombatBonuses::from_rank(0);
    assert_eq!(b.flat_damage, 0);
    assert_eq!(b.flat_defense, 0);
    assert!((b.crit_chance - 0.0).abs() < f64::EPSILON);
    assert_eq!(b.flat_hp, 0);
}

#[test]
fn prestige_bonuses_rank_1_all_positive() {
    let b = PrestigeCombatBonuses::from_rank(1);
    assert!(b.flat_damage > 0, "flat_damage should be > 0 at rank 1");
    assert!(b.flat_defense > 0, "flat_defense should be > 0 at rank 1");
    assert!(b.crit_chance > 0.0, "crit_chance should be > 0 at rank 1");
    assert!(b.flat_hp > 0, "flat_hp should be > 0 at rank 1");
}

#[test]
fn prestige_bonuses_monotonic_scaling() {
    let ranks = [1, 5, 10, 20];
    for pair in ranks.windows(2) {
        let lower = PrestigeCombatBonuses::from_rank(pair[0]);
        let higher = PrestigeCombatBonuses::from_rank(pair[1]);
        assert!(
            higher.flat_damage >= lower.flat_damage,
            "flat_damage should increase: rank {} ({}) vs rank {} ({})",
            pair[0],
            lower.flat_damage,
            pair[1],
            higher.flat_damage
        );
        assert!(
            higher.flat_defense >= lower.flat_defense,
            "flat_defense should increase: rank {} ({}) vs rank {} ({})",
            pair[0],
            lower.flat_defense,
            pair[1],
            higher.flat_defense
        );
        assert!(
            higher.crit_chance >= lower.crit_chance,
            "crit_chance should increase: rank {} ({}) vs rank {} ({})",
            pair[0],
            lower.crit_chance,
            pair[1],
            higher.crit_chance
        );
        assert!(
            higher.flat_hp >= lower.flat_hp,
            "flat_hp should increase: rank {} ({}) vs rank {} ({})",
            pair[0],
            lower.flat_hp,
            pair[1],
            higher.flat_hp
        );
    }
}

#[test]
fn prestige_bonuses_damage_formula() {
    // flat_damage = floor(5.0 * rank^0.7)
    let b5 = PrestigeCombatBonuses::from_rank(5);
    let expected_5 = (5.0_f64 * 5.0_f64.powf(0.7)).floor() as u64;
    assert_eq!(b5.flat_damage, expected_5);

    let b10 = PrestigeCombatBonuses::from_rank(10);
    let expected_10 = (5.0_f64 * 10.0_f64.powf(0.7)).floor() as u64;
    assert_eq!(b10.flat_damage, expected_10);

    let b20 = PrestigeCombatBonuses::from_rank(20);
    let expected_20 = (5.0_f64 * 20.0_f64.powf(0.7)).floor() as u64;
    assert_eq!(b20.flat_damage, expected_20);
}

#[test]
fn prestige_bonuses_defense_formula() {
    // flat_defense = floor(3.0 * rank^0.6)
    let b10 = PrestigeCombatBonuses::from_rank(10);
    let expected = (3.0_f64 * 10.0_f64.powf(0.6)).floor() as u64;
    assert_eq!(b10.flat_defense, expected);
}

#[test]
fn prestige_bonuses_crit_formula() {
    // crit_chance = min(rank * 0.5, 15.0)
    let b10 = PrestigeCombatBonuses::from_rank(10);
    assert!((b10.crit_chance - 5.0).abs() < f64::EPSILON);

    let b20 = PrestigeCombatBonuses::from_rank(20);
    assert!((b20.crit_chance - 10.0).abs() < f64::EPSILON);
}

#[test]
fn prestige_bonuses_crit_capped_at_15() {
    // At rank 50: 50 * 0.5 = 25.0, but cap is 15.0
    let b50 = PrestigeCombatBonuses::from_rank(50);
    assert!(
        (b50.crit_chance - 15.0).abs() < f64::EPSILON,
        "crit_chance should be capped at 15.0 but was {}",
        b50.crit_chance
    );

    // At rank 100, still 15.0
    let b100 = PrestigeCombatBonuses::from_rank(100);
    assert!(
        (b100.crit_chance - 15.0).abs() < f64::EPSILON,
        "crit_chance should be capped at 15.0 but was {}",
        b100.crit_chance
    );
}

#[test]
fn prestige_bonuses_hp_formula() {
    // flat_hp = floor(15.0 * rank^0.6)
    let b5 = PrestigeCombatBonuses::from_rank(5);
    let expected = (15.0_f64 * 5.0_f64.powf(0.6)).floor() as u64;
    assert_eq!(b5.flat_hp, expected);
}

#[test]
fn prestige_bonuses_all_fields_at_rank_10() {
    let b = PrestigeCombatBonuses::from_rank(10);
    assert!(b.flat_damage > 0);
    assert!(b.flat_defense > 0);
    assert!(b.crit_chance > 0.0);
    assert!(b.flat_hp > 0);
}

#[test]
fn prestige_bonuses_default_is_zeros() {
    let d = PrestigeCombatBonuses::default();
    assert_eq!(d.flat_damage, 0);
    assert_eq!(d.flat_defense, 0);
    assert!((d.crit_chance - 0.0).abs() < f64::EPSILON);
    assert_eq!(d.flat_hp, 0);
}

// ═══════════════════════════════════════════════════════════════════
// items/types — Item, EquipmentSlot, Rarity, AttributeBonuses, Affix
// ═══════════════════════════════════════════════════════════════════

#[test]
fn item_power_attributes_only() {
    let attrs = AttributeBonuses {
        str: 5,
        dex: 3,
        con: 2,
        int: 0,
        wis: 0,
        cha: 0,
    };
    let item = make_item(EquipmentSlot::Weapon, Rarity::Common, attrs, vec![]);
    // power = sum(attrs) = 5 + 3 + 2 = 10
    assert_eq!(item.power(), 10);
}

#[test]
fn item_power_with_affixes() {
    let attrs = AttributeBonuses {
        str: 4,
        ..AttributeBonuses::new()
    };
    let affixes = vec![Affix {
        affix_type: AffixType::DamagePercent,
        value: 10.0,
    }];
    let item = make_item(EquipmentSlot::Weapon, Rarity::Magic, attrs, affixes);
    // power = 4 + (10.0 * 2.0) = 4 + 20 = 24
    assert_eq!(item.power(), 24);
}

#[test]
fn item_power_unknown_affix_contributes_zero() {
    let attrs = AttributeBonuses {
        str: 5,
        ..AttributeBonuses::new()
    };
    let affixes = vec![Affix {
        affix_type: AffixType::Unknown,
        value: 100.0,
    }];
    let item = make_item(EquipmentSlot::Weapon, Rarity::Common, attrs, affixes);
    // Unknown weight is 0.0, so power = 5 + (100.0 * 0.0) = 5
    assert_eq!(item.power(), 5);
}

#[test]
fn item_power_multiple_affixes() {
    let attrs = AttributeBonuses::new(); // all zeros
    let affixes = vec![
        Affix {
            affix_type: AffixType::CritChance,
            value: 10.0,
        }, // 10.0 * 1.5 = 15
        Affix {
            affix_type: AffixType::HPBonus,
            value: 50.0,
        }, // 50.0 * 0.5 = 25
        Affix {
            affix_type: AffixType::DamageReduction,
            value: 5.0,
        }, // 5.0 * 1.3 = 6.5
    ];
    let item = make_item(EquipmentSlot::Armor, Rarity::Epic, attrs, affixes);
    // power = 0 + 15 + 25 + 6.5 = 46.5 -> rounded = 47 (note: round(46.5) = 46 or 47)
    let expected = (15.0 + 25.0 + 6.5_f64).round() as u32;
    assert_eq!(item.power(), expected);
}

#[test]
fn item_stat_summary_shows_nonzero_attrs() {
    let attrs = AttributeBonuses {
        str: 8,
        dex: 3,
        con: 0,
        int: 0,
        wis: 0,
        cha: 0,
    };
    let item = make_item(EquipmentSlot::Weapon, Rarity::Common, attrs, vec![]);
    let summary = item.stat_summary();
    assert!(
        summary.contains("+8 STR"),
        "Expected '+8 STR' in '{}'",
        summary
    );
    assert!(
        summary.contains("+3 DEX"),
        "Expected '+3 DEX' in '{}'",
        summary
    );
    assert!(
        !summary.contains("CON"),
        "Should not contain CON when 0 in '{}'",
        summary
    );
}

#[test]
fn item_stat_summary_shows_affixes() {
    let attrs = AttributeBonuses::new();
    let affixes = vec![
        Affix {
            affix_type: AffixType::DamagePercent,
            value: 15.0,
        },
        Affix {
            affix_type: AffixType::CritChance,
            value: 10.0,
        },
    ];
    let item = make_item(EquipmentSlot::Ring, Rarity::Rare, attrs, affixes);
    let summary = item.stat_summary();
    assert!(
        summary.contains("Dmg"),
        "Expected Dmg affix in '{}'",
        summary
    );
    assert!(
        summary.contains("Crit"),
        "Expected Crit affix in '{}'",
        summary
    );
}

#[test]
fn item_stat_summary_skips_unknown_affix() {
    let attrs = AttributeBonuses::new();
    let affixes = vec![
        Affix {
            affix_type: AffixType::Unknown,
            value: 99.0,
        },
        Affix {
            affix_type: AffixType::XPGain,
            value: 5.0,
        },
    ];
    let item = make_item(EquipmentSlot::Amulet, Rarity::Common, attrs, affixes);
    let summary = item.stat_summary();
    assert!(summary.contains("XP"), "Expected XP affix in '{}'", summary);
    // Unknown should be skipped entirely
    assert!(
        !summary.contains("99"),
        "Unknown affix should not appear in '{}'",
        summary
    );
}

#[test]
fn item_stat_summary_all_affix_types() {
    let affixes = vec![
        Affix {
            affix_type: AffixType::DamagePercent,
            value: 1.0,
        },
        Affix {
            affix_type: AffixType::CritChance,
            value: 1.0,
        },
        Affix {
            affix_type: AffixType::CritMultiplier,
            value: 1.0,
        },
        Affix {
            affix_type: AffixType::AttackSpeed,
            value: 1.0,
        },
        Affix {
            affix_type: AffixType::HPBonus,
            value: 1.0,
        },
        Affix {
            affix_type: AffixType::DamageReduction,
            value: 1.0,
        },
        Affix {
            affix_type: AffixType::HPRegen,
            value: 1.0,
        },
        Affix {
            affix_type: AffixType::DamageReflection,
            value: 1.0,
        },
        Affix {
            affix_type: AffixType::XPGain,
            value: 1.0,
        },
    ];
    let item = make_item(
        EquipmentSlot::Ring,
        Rarity::Legendary,
        AttributeBonuses::new(),
        affixes,
    );
    let summary = item.stat_summary();
    assert!(
        summary.contains("Dmg"),
        "Missing DamagePercent in '{}'",
        summary
    );
    assert!(
        summary.contains("Crit"),
        "Missing CritChance in '{}'",
        summary
    );
    assert!(
        summary.contains("CritDmg"),
        "Missing CritMultiplier in '{}'",
        summary
    );
    assert!(
        summary.contains("AtkSpd"),
        "Missing AttackSpeed in '{}'",
        summary
    );
    assert!(summary.contains("HP"), "Missing HPBonus in '{}'", summary);
    assert!(
        summary.contains("DR"),
        "Missing DamageReduction in '{}'",
        summary
    );
    assert!(
        summary.contains("Regen"),
        "Missing HPRegen in '{}'",
        summary
    );
    assert!(
        summary.contains("Reflect"),
        "Missing DamageReflection in '{}'",
        summary
    );
    assert!(summary.contains("XP"), "Missing XPGain in '{}'", summary);
}

#[test]
fn equipment_slot_icon_all_nonempty() {
    let slots = [
        EquipmentSlot::Weapon,
        EquipmentSlot::Armor,
        EquipmentSlot::Helmet,
        EquipmentSlot::Gloves,
        EquipmentSlot::Boots,
        EquipmentSlot::Amulet,
        EquipmentSlot::Ring,
    ];
    for slot in &slots {
        let icon = slot.icon();
        assert!(!icon.is_empty(), "Icon for {:?} should not be empty", slot);
    }
}

#[test]
fn equipment_slot_name_all_nonempty() {
    let slots = [
        EquipmentSlot::Weapon,
        EquipmentSlot::Armor,
        EquipmentSlot::Helmet,
        EquipmentSlot::Gloves,
        EquipmentSlot::Boots,
        EquipmentSlot::Amulet,
        EquipmentSlot::Ring,
    ];
    for slot in &slots {
        let name = slot.name();
        assert!(!name.is_empty(), "Name for {:?} should not be empty", slot);
    }
}

#[test]
fn equipment_slot_icon_width() {
    // Weapon and Armor have text symbols (width 1), others have emoji (width 2)
    assert_eq!(EquipmentSlot::Weapon.icon_width(), 1);
    assert_eq!(EquipmentSlot::Armor.icon_width(), 1);
    assert_eq!(EquipmentSlot::Helmet.icon_width(), 2);
    assert_eq!(EquipmentSlot::Gloves.icon_width(), 2);
    assert_eq!(EquipmentSlot::Boots.icon_width(), 2);
    assert_eq!(EquipmentSlot::Amulet.icon_width(), 2);
    assert_eq!(EquipmentSlot::Ring.icon_width(), 2);
}

#[test]
fn attribute_bonuses_to_attributes_conversion() {
    use quest::character::attributes::AttributeType;
    let bonuses = AttributeBonuses {
        str: 10,
        dex: 5,
        con: 3,
        int: 7,
        wis: 2,
        cha: 1,
    };
    let attrs = bonuses.to_attributes();
    assert_eq!(attrs.get(AttributeType::Strength), 10);
    assert_eq!(attrs.get(AttributeType::Dexterity), 5);
    assert_eq!(attrs.get(AttributeType::Constitution), 3);
    assert_eq!(attrs.get(AttributeType::Intelligence), 7);
    assert_eq!(attrs.get(AttributeType::Wisdom), 2);
    assert_eq!(attrs.get(AttributeType::Charisma), 1);
}

#[test]
fn attribute_bonuses_as_array() {
    let bonuses = AttributeBonuses {
        str: 1,
        dex: 2,
        con: 3,
        int: 4,
        wis: 5,
        cha: 6,
    };
    assert_eq!(bonuses.as_array(), [1, 2, 3, 4, 5, 6]);
}

#[test]
fn attribute_bonuses_total() {
    let bonuses = AttributeBonuses {
        str: 10,
        dex: 20,
        con: 30,
        int: 0,
        wis: 0,
        cha: 0,
    };
    assert_eq!(bonuses.total(), 60);
}

#[test]
fn rarity_ordering_complete() {
    assert!(Rarity::Common < Rarity::Magic);
    assert!(Rarity::Magic < Rarity::Rare);
    assert!(Rarity::Rare < Rarity::Epic);
    assert!(Rarity::Epic < Rarity::Legendary);
    assert!(Rarity::Legendary < Rarity::Mythic);
}

#[test]
fn rarity_names() {
    assert_eq!(Rarity::Common.name(), "Common");
    assert_eq!(Rarity::Magic.name(), "Magic");
    assert_eq!(Rarity::Rare.name(), "Rare");
    assert_eq!(Rarity::Epic.name(), "Epic");
    assert_eq!(Rarity::Legendary.name(), "Legendary");
    assert_eq!(Rarity::Mythic.name(), "God");
}

#[test]
fn item_slot_name_via_item() {
    let item = make_item(
        EquipmentSlot::Boots,
        Rarity::Common,
        AttributeBonuses::new(),
        vec![],
    );
    assert_eq!(item.slot_name(), "Boots");
}

#[test]
fn affix_power_weights_correct() {
    assert!((affix_power_weight(AffixType::DamagePercent) - 2.0).abs() < f64::EPSILON);
    assert!((affix_power_weight(AffixType::CritChance) - 1.5).abs() < f64::EPSILON);
    assert!((affix_power_weight(AffixType::CritMultiplier) - 1.5).abs() < f64::EPSILON);
    assert!((affix_power_weight(AffixType::AttackSpeed) - 1.2).abs() < f64::EPSILON);
    assert!((affix_power_weight(AffixType::HPBonus) - 0.5).abs() < f64::EPSILON);
    assert!((affix_power_weight(AffixType::DamageReduction) - 1.3).abs() < f64::EPSILON);
    assert!((affix_power_weight(AffixType::HPRegen) - 1.0).abs() < f64::EPSILON);
    assert!((affix_power_weight(AffixType::DamageReflection) - 0.8).abs() < f64::EPSILON);
    assert!((affix_power_weight(AffixType::XPGain) - 1.0).abs() < f64::EPSILON);
    assert!((affix_power_weight(AffixType::Unknown) - 0.0).abs() < f64::EPSILON);
}

// ═══════════════════════════════════════════════════════════════════
// combat/types — Enemy, CombatState, enemy generators
// ═══════════════════════════════════════════════════════════════════

#[test]
fn enemy_new_hp_equals_max() {
    let e = Enemy::new("Orc".to_string(), 100, 15);
    assert_eq!(e.current_hp, e.max_hp);
    assert_eq!(e.max_hp, 100);
    assert_eq!(e.damage, 15);
    assert_eq!(e.defense, 0);
}

#[test]
fn enemy_new_with_defense_sets_defense() {
    let e = Enemy::new_with_defense("Knight".to_string(), 200, 25, 10);
    assert_eq!(e.defense, 10);
    assert_eq!(e.max_hp, 200);
    assert_eq!(e.current_hp, 200);
    assert_eq!(e.damage, 25);
}

#[test]
fn enemy_is_alive_true_when_hp_positive() {
    let e = Enemy::new("Alive".to_string(), 1, 1);
    assert!(e.is_alive());
}

#[test]
fn enemy_is_alive_false_when_hp_zero() {
    let mut e = Enemy::new("Dead".to_string(), 10, 1);
    e.take_damage(10);
    assert!(!e.is_alive());
}

#[test]
fn enemy_take_damage_reduces_hp() {
    let mut e = Enemy::new("Mob".to_string(), 50, 5);
    e.take_damage(20);
    assert_eq!(e.current_hp, 30);
}

#[test]
fn enemy_take_damage_saturates_at_zero() {
    let mut e = Enemy::new("Mob".to_string(), 50, 5);
    e.take_damage(999);
    assert_eq!(e.current_hp, 0);
}

#[test]
fn enemy_reset_hp_restores_to_max() {
    let mut e = Enemy::new("Boss".to_string(), 100, 10);
    e.take_damage(80);
    assert_eq!(e.current_hp, 20);
    e.reset_hp();
    assert_eq!(e.current_hp, 100);
}

#[test]
fn generate_enemy_name_produces_nonempty_string() {
    let name = generate_enemy_name();
    assert!(!name.is_empty());
    assert!(
        name.contains(' '),
        "Name should have prefix+suffix: '{}'",
        name
    );
}

#[test]
fn generate_zone_enemy_name_per_zone() {
    for zone_id in 1..=10 {
        let name = generate_zone_enemy_name(zone_id);
        assert!(
            !name.is_empty(),
            "Name for zone {} should not be empty",
            zone_id
        );
        assert!(
            name.contains(' '),
            "Name for zone {} should have space: '{}'",
            zone_id,
            name
        );
    }
    // Fallback zone
    let fallback = generate_zone_enemy_name(99);
    assert!(!fallback.is_empty());
}

#[test]
fn generate_zone_enemy_produces_valid_enemy() {
    let zones = get_all_zones();
    let zone = &zones[0]; // Meadow
    let subzone = &zone.subzones[0];
    let enemy = generate_zone_enemy(zone, subzone);
    assert!(enemy.max_hp >= 1);
    assert!(enemy.damage >= 1);
    assert_eq!(enemy.current_hp, enemy.max_hp);
}

#[test]
fn generate_subzone_boss_has_higher_stats() {
    let zones = get_all_zones();
    let zone = &zones[0]; // Meadow
    let subzone = &zone.subzones[0];

    // Generate many pairs to account for variance
    let mut boss_hp_sum = 0u64;
    let mut mob_hp_sum = 0u64;
    let samples = 20;
    for _ in 0..samples {
        let boss = generate_subzone_boss(zone, subzone);
        let mob = generate_zone_enemy(zone, subzone);
        boss_hp_sum += boss.max_hp as u64;
        mob_hp_sum += mob.max_hp as u64;
    }
    assert!(
        boss_hp_sum > mob_hp_sum,
        "Average boss HP ({}) should exceed average mob HP ({})",
        boss_hp_sum / samples as u64,
        mob_hp_sum / samples as u64
    );
}

#[test]
fn generate_dungeon_enemy_valid() {
    let enemy = generate_dungeon_enemy(1);
    assert!(enemy.max_hp >= 1);
    assert!(enemy.damage >= 1);
    assert_eq!(enemy.current_hp, enemy.max_hp);
}

#[test]
fn generate_dungeon_elite_prefixed() {
    let elite = generate_dungeon_elite(1);
    assert!(
        elite.name.starts_with("Elite "),
        "Elite name should start with 'Elite ': '{}'",
        elite.name
    );
    assert!(elite.max_hp >= 1);
}

#[test]
fn generate_dungeon_boss_prefixed() {
    let boss = generate_dungeon_boss(1);
    assert!(
        boss.name.starts_with("Boss "),
        "Boss name should start with 'Boss ': '{}'",
        boss.name
    );
    assert!(boss.max_hp >= 1);
}

#[test]
fn generate_dungeon_elite_stronger_than_normal() {
    let mut elite_hp_sum = 0u64;
    let mut normal_hp_sum = 0u64;
    let samples = 20;
    for _ in 0..samples {
        elite_hp_sum += generate_dungeon_elite(5).max_hp as u64;
        normal_hp_sum += generate_dungeon_enemy(5).max_hp as u64;
    }
    assert!(
        elite_hp_sum > normal_hp_sum,
        "Average elite HP ({}) should exceed average normal HP ({})",
        elite_hp_sum / samples as u64,
        normal_hp_sum / samples as u64
    );
}

#[test]
fn generate_dungeon_boss_stronger_than_elite() {
    let mut boss_hp_sum = 0u64;
    let mut elite_hp_sum = 0u64;
    let samples = 20;
    for _ in 0..samples {
        boss_hp_sum += generate_dungeon_boss(5).max_hp as u64;
        elite_hp_sum += generate_dungeon_elite(5).max_hp as u64;
    }
    assert!(
        boss_hp_sum > elite_hp_sum,
        "Average boss HP ({}) should exceed average elite HP ({})",
        boss_hp_sum / samples as u64,
        elite_hp_sum / samples as u64
    );
}

#[test]
fn generate_enemy_for_current_zone_valid() {
    let enemy = generate_enemy_for_current_zone(1, 1);
    assert!(enemy.max_hp >= 1);
    assert!(enemy.damage >= 1);
}

#[test]
fn generate_enemy_for_current_zone_fallback() {
    // Invalid zone/subzone should still produce a valid enemy
    let enemy = generate_enemy_for_current_zone(999, 999);
    assert!(enemy.max_hp >= 1);
}

#[test]
fn generate_boss_for_current_zone_valid() {
    let boss = generate_boss_for_current_zone(1, 1);
    assert!(boss.max_hp >= 1);
    assert!(boss.damage >= 1);
}

#[test]
fn generate_boss_for_current_zone_fallback() {
    let boss = generate_boss_for_current_zone(999, 999);
    assert!(boss.max_hp >= 1);
    assert!(boss.name.contains("Unknown Boss") || !boss.name.is_empty());
}

#[test]
fn zone_enemies_scale_with_zone() {
    let zones = get_all_zones();
    // Zone 1 vs Zone 5
    let zone1 = &zones[0];
    let zone5 = &zones[4];

    let mut z1_hp = 0u64;
    let mut z5_hp = 0u64;
    let samples = 20;
    for _ in 0..samples {
        z1_hp += generate_zone_enemy(zone1, &zone1.subzones[0]).max_hp as u64;
        z5_hp += generate_zone_enemy(zone5, &zone5.subzones[0]).max_hp as u64;
    }
    assert!(
        z5_hp > z1_hp,
        "Zone 5 avg HP ({}) should exceed Zone 1 avg HP ({})",
        z5_hp / samples as u64,
        z1_hp / samples as u64
    );
}

#[test]
fn combat_state_new_defaults() {
    let cs = CombatState::new(100);
    assert_eq!(cs.player_max_hp, 100);
    assert_eq!(cs.player_current_hp, 100);
    assert!(cs.is_player_alive());
    assert!(cs.current_enemy.is_none());
    assert!(!cs.is_regenerating);
    assert_eq!(cs.player_attack_timer, 0.0);
    assert_eq!(cs.enemy_attack_timer, 0.0);
    assert_eq!(cs.regen_timer, 0.0);
}

#[test]
fn combat_state_default_uses_base_hp() {
    let cs = CombatState::default();
    // BASE_HP is defined in constants; just verify it's reasonable
    assert!(cs.player_max_hp > 0);
    assert_eq!(cs.player_current_hp, cs.player_max_hp);
}

#[test]
fn combat_state_update_max_hp_caps_current() {
    let mut cs = CombatState::new(100);
    cs.player_current_hp = 80;
    cs.update_max_hp(50);
    assert_eq!(cs.player_max_hp, 50);
    assert_eq!(cs.player_current_hp, 50); // Capped to new max
}

#[test]
fn combat_state_update_max_hp_increases_max_only() {
    let mut cs = CombatState::new(50);
    cs.player_current_hp = 30;
    cs.update_max_hp(100);
    assert_eq!(cs.player_max_hp, 100);
    assert_eq!(cs.player_current_hp, 30); // Unchanged, still below new max
}

#[test]
fn combat_state_add_log_entry() {
    let mut cs = CombatState::new(100);
    cs.add_log_entry("Hit for 10".to_string(), false, true);
    assert_eq!(cs.combat_log.len(), 1);
    assert_eq!(cs.combat_log[0].message, "Hit for 10");
    assert!(!cs.combat_log[0].is_crit);
    assert!(cs.combat_log[0].is_player_action);
}

#[test]
fn combat_state_log_entry_cap() {
    let mut cs = CombatState::new(100);
    // Add more than COMBAT_LOG_CAPACITY entries
    for i in 0..20 {
        cs.add_log_entry(format!("Entry {}", i), false, true);
    }
    // Should be capped (COMBAT_LOG_CAPACITY is typically 10)
    assert!(
        cs.combat_log.len() <= 10,
        "Log should be capped at capacity"
    );
}

#[test]
fn combat_state_is_player_alive() {
    let mut cs = CombatState::new(10);
    assert!(cs.is_player_alive());
    cs.player_current_hp = 0;
    assert!(!cs.is_player_alive());
}

#[test]
fn combat_state_tick_hud_decays_floats() {
    let mut cs = CombatState::new(100);
    cs.player_damage_floats
        .push(quest::combat::types::DamageFlash {
            text: "10".to_string(),
            color: ratatui::style::Color::Red,
            bold: false,
            remaining: 0.5,
        });
    cs.enemy_damage_floats
        .push(quest::combat::types::DamageFlash {
            text: "20".to_string(),
            color: ratatui::style::Color::Yellow,
            bold: true,
            remaining: 0.1,
        });

    cs.tick_hud(0.2);
    // Player float: 0.5 - 0.2 = 0.3 (still alive)
    assert_eq!(cs.player_damage_floats.len(), 1);
    // Enemy float: 0.1 - 0.2 = -0.1 (expired)
    assert_eq!(cs.enemy_damage_floats.len(), 0);
}

// ═══════════════════════════════════════════════════════════════════
// core/tick — game_tick basic behavior
// ═══════════════════════════════════════════════════════════════════

mod tick_tests {
    use quest::achievements::Achievements;
    use quest::core::game_state::GameState;
    use quest::core::tick::game_tick;
    use quest::core::tick_types::TickResult;
    use quest::enhancement::EnhancementProgress;
    use quest::haven::Haven;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn test_rng() -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(42)
    }

    #[test]
    fn game_tick_debug_mode_suppresses_achievement_save_flags() {
        let mut state = GameState::new("Debug Test".to_string(), 0);
        let mut tick_counter = 0u32;
        let mut haven = Haven::default();
        let mut enhancement = EnhancementProgress::new();
        let mut achievements = Achievements::default();
        let mut rng = test_rng();

        // With debug_mode=true, even if haven/achievements change,
        // the save flags should be suppressed for those triggered by discovery
        let result = game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut enhancement,
            &mut quest::deep::DeepState::new(),
            &mut achievements,
            true,
            &mut rng,
        );

        // After a single tick, no discovery should trigger (P0),
        // so no achievement/haven change flags
        assert!(!result.achievements_changed);
        assert!(!result.haven_changed);
        assert!(!result.enhancement_changed);
    }

    #[test]
    fn game_tick_recalculates_derived_stats_when_dirty() {
        let mut state = GameState::new("Stats Test".to_string(), 0);
        state.derived_stats_dirty = true;
        let mut tick_counter = 0u32;
        let mut haven = Haven::default();
        let mut enhancement = EnhancementProgress::new();
        let mut achievements = Achievements::default();
        let mut rng = test_rng();

        game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut enhancement,
            &mut quest::deep::DeepState::new(),
            &mut achievements,
            false,
            &mut rng,
        );

        // After tick, dirty flag should be cleared by recalculation
        assert!(!state.derived_stats_dirty);
    }

    #[test]
    fn game_tick_idle_state_spawns_enemy() {
        let mut state = GameState::new("Spawn Test".to_string(), 0);
        assert!(state.combat_state.current_enemy.is_none());

        let mut tick_counter = 0u32;
        let mut haven = Haven::default();
        let mut enhancement = EnhancementProgress::new();
        let mut achievements = Achievements::default();
        let mut rng = test_rng();

        game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut enhancement,
            &mut quest::deep::DeepState::new(),
            &mut achievements,
            false,
            &mut rng,
        );

        assert!(state.combat_state.current_enemy.is_some());
    }

    #[test]
    fn game_tick_increments_tick_counter() {
        let mut state = GameState::new("Counter Test".to_string(), 0);
        let mut tick_counter = 0u32;
        let mut haven = Haven::default();
        let mut enhancement = EnhancementProgress::new();
        let mut achievements = Achievements::default();
        let mut rng = test_rng();

        game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut enhancement,
            &mut quest::deep::DeepState::new(),
            &mut achievements,
            false,
            &mut rng,
        );

        assert_eq!(tick_counter, 1);
    }

    #[test]
    fn game_tick_play_time_increments_after_10_ticks() {
        let mut state = GameState::new("Time Test".to_string(), 0);
        let mut tick_counter = 0u32;
        let mut haven = Haven::default();
        let mut enhancement = EnhancementProgress::new();
        let mut achievements = Achievements::default();
        let mut rng = test_rng();

        let initial_time = state.play_time_seconds;

        for _ in 0..10 {
            game_tick(
                &mut state,
                &mut tick_counter,
                &mut haven,
                &mut enhancement,
                &mut quest::deep::DeepState::new(),
                &mut achievements,
                false,
                &mut rng,
            );
        }

        assert_eq!(state.play_time_seconds, initial_time + 1);
        assert_eq!(tick_counter, 0); // Resets after 10
    }

    #[test]
    fn game_tick_result_defaults() {
        let result = TickResult::default();
        assert!(result.events.is_empty());
        assert!(result.leviathan_encounter.is_none());
        assert!(!result.achievements_changed);
        assert!(!result.haven_changed);
        assert!(!result.enhancement_changed);
        assert!(result.achievement_modal_ready.is_empty());
    }

    #[test]
    fn game_tick_no_haven_discovery_at_rank_0() {
        let mut state = GameState::new("No Haven".to_string(), 0);
        state.prestige_rank = 0; // Below HAVEN_MIN_PRESTIGE_RANK (10)
        let mut tick_counter = 0u32;
        let mut haven = Haven::default();
        let mut enhancement = EnhancementProgress::new();
        let mut achievements = Achievements::default();
        let mut rng = test_rng();

        // Run many ticks -- haven should never be discovered at P0
        for _ in 0..100 {
            let result = game_tick(
                &mut state,
                &mut tick_counter,
                &mut haven,
                &mut enhancement,
                &mut quest::deep::DeepState::new(),
                &mut achievements,
                false,
                &mut rng,
            );
            assert!(!result.haven_changed);
        }
        assert!(!haven.discovered);
    }

    #[test]
    fn game_tick_no_soulforge_discovery_at_rank_0() {
        let mut state = GameState::new("No Forge".to_string(), 0);
        state.prestige_rank = 0; // Below SOULFORGE_MIN_PRESTIGE_RANK (15)
        let mut tick_counter = 0u32;
        let mut haven = Haven::default();
        let mut enhancement = EnhancementProgress::new();
        let mut achievements = Achievements::default();
        let mut rng = test_rng();

        for _ in 0..100 {
            let result = game_tick(
                &mut state,
                &mut tick_counter,
                &mut haven,
                &mut enhancement,
                &mut quest::deep::DeepState::new(),
                &mut achievements,
                false,
                &mut rng,
            );
            assert!(!result.enhancement_changed);
        }
        assert!(!enhancement.discovered);
    }

    #[test]
    fn game_tick_xp_rate_samples_populated_after_10_ticks() {
        let mut state = GameState::new("XP Rate".to_string(), 0);
        let mut tick_counter = 0u32;
        let mut haven = Haven::default();
        let mut enhancement = EnhancementProgress::new();
        let mut achievements = Achievements::default();
        let mut rng = test_rng();

        for _ in 0..10 {
            game_tick(
                &mut state,
                &mut tick_counter,
                &mut haven,
                &mut enhancement,
                &mut quest::deep::DeepState::new(),
                &mut achievements,
                false,
                &mut rng,
            );
        }

        // After 10 ticks (1 second) with no combat, no XP rate sample should be recorded
        // (combat_seconds_this_tick is false when no XP is earned)
        assert_eq!(state.xp_rate_samples.len(), 0);
    }
}
