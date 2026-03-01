//! Tests for item generation and scoring pipeline.
//!
//! Covers gaps not handled by item_pipeline_test.rs and item_combat_coverage_test.rs:
//! - Mythic rarity generation
//! - T0/T9 multiplier precision at specific ilvls
//! - God item protection edge cases (Mythic vs Mythic replacement)
//! - All affix power weights verified numerically
//! - Name generation for Mythic rarity items
//! - Unknown affix type power weight is zero
//! - Mob drop distribution verification at prestige boundaries
//! - Boss drop never Common for both normal and final zone

use quest::items::drops::{
    drop_chance_for_prestige, ilvl_for_zone, roll_rarity_for_boss, roll_rarity_for_mob,
    try_drop_from_boss,
};
use quest::items::generation::{generate_item, tier_multiplier};
use quest::items::scoring::{affix_power_weight, auto_equip_if_better};
use quest::items::types::{Affix, AffixType, AttributeBonuses, EquipmentSlot, Item, Rarity};
use quest::GameState;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

// =========================================================================
// Affix power weight — verify every variant numerically
// =========================================================================

#[test]
fn test_affix_power_weight_damage_percent() {
    assert_eq!(affix_power_weight(AffixType::DamagePercent), 2.0);
}

#[test]
fn test_affix_power_weight_crit_chance() {
    assert_eq!(affix_power_weight(AffixType::CritChance), 1.5);
}

#[test]
fn test_affix_power_weight_crit_multiplier() {
    assert_eq!(affix_power_weight(AffixType::CritMultiplier), 1.5);
}

#[test]
fn test_affix_power_weight_attack_speed() {
    assert_eq!(affix_power_weight(AffixType::AttackSpeed), 1.2);
}

#[test]
fn test_affix_power_weight_damage_reduction() {
    assert_eq!(affix_power_weight(AffixType::DamageReduction), 1.3);
}

#[test]
fn test_affix_power_weight_hp_regen() {
    assert_eq!(affix_power_weight(AffixType::HPRegen), 1.0);
}

#[test]
fn test_affix_power_weight_xp_gain() {
    assert_eq!(affix_power_weight(AffixType::XPGain), 1.0);
}

#[test]
fn test_affix_power_weight_damage_reflection() {
    assert_eq!(affix_power_weight(AffixType::DamageReflection), 0.8);
}

#[test]
fn test_affix_power_weight_hp_bonus_lowest() {
    assert_eq!(affix_power_weight(AffixType::HPBonus), 0.5);
}

#[test]
fn test_affix_power_weight_unknown_is_zero() {
    assert_eq!(affix_power_weight(AffixType::Unknown), 0.0);
}

#[test]
fn test_weight_ordering_damage_highest() {
    // DamagePercent > CritChance == CritMultiplier > DamageReduction > AttackSpeed
    // > HPRegen == XPGain > DamageReflection > HPBonus
    assert!(
        affix_power_weight(AffixType::DamagePercent) > affix_power_weight(AffixType::CritChance)
    );
    assert!(
        affix_power_weight(AffixType::CritChance) > affix_power_weight(AffixType::DamageReduction)
    );
    assert!(
        affix_power_weight(AffixType::DamageReduction) > affix_power_weight(AffixType::AttackSpeed)
    );
    assert!(affix_power_weight(AffixType::AttackSpeed) > affix_power_weight(AffixType::HPRegen));
    assert!(
        affix_power_weight(AffixType::HPRegen) > affix_power_weight(AffixType::DamageReflection)
    );
    assert!(
        affix_power_weight(AffixType::DamageReflection) > affix_power_weight(AffixType::HPBonus)
    );
    assert!(affix_power_weight(AffixType::HPBonus) > affix_power_weight(AffixType::Unknown));
}

// =========================================================================
// Power calculation — exact numeric checks
// =========================================================================

#[test]
fn test_power_pure_attributes_equal_total() {
    let item = Item {
        slot: EquipmentSlot::Weapon,
        rarity: Rarity::Common,
        ilvl: 10,
        tier: 5,
        base_name: "Sword".to_string(),
        display_name: "Sword".to_string(),
        attributes: AttributeBonuses {
            str: 3,
            dex: 5,
            con: 2,
            int: 0,
            wis: 0,
            cha: 0,
        },
        affixes: vec![],
        god_item_id: None,
    };
    // power = 3+5+2 = 10, no affixes
    assert_eq!(item.power(), 10);
}

#[test]
fn test_power_mixed_attributes_and_affixes() {
    let item = Item {
        slot: EquipmentSlot::Ring,
        rarity: Rarity::Rare,
        ilvl: 10,
        tier: 5,
        base_name: "Ring".to_string(),
        display_name: "Ring".to_string(),
        attributes: AttributeBonuses {
            str: 10,
            ..AttributeBonuses::new()
        },
        affixes: vec![
            Affix {
                affix_type: AffixType::DamagePercent,
                value: 5.0,
            },
            Affix {
                affix_type: AffixType::HPBonus,
                value: 20.0,
            },
        ],
        god_item_id: None,
    };
    // power = 10 (attr) + 5*2.0 (dmg%) + 20*0.5 (hp) = 10 + 10 + 10 = 30
    assert_eq!(item.power(), 30);
}

#[test]
fn test_power_unknown_affixes_contribute_zero() {
    let item = Item {
        slot: EquipmentSlot::Amulet,
        rarity: Rarity::Magic,
        ilvl: 10,
        tier: 5,
        base_name: "Amulet".to_string(),
        display_name: "Amulet".to_string(),
        attributes: AttributeBonuses {
            int: 4,
            ..AttributeBonuses::new()
        },
        affixes: vec![Affix {
            affix_type: AffixType::Unknown,
            value: 100.0, // Should contribute 0
        }],
        god_item_id: None,
    };
    // power = 4 (attr) + 100 * 0.0 (unknown) = 4
    assert_eq!(item.power(), 4);
}

#[test]
fn test_power_all_affix_types_contribute_to_power() {
    // A single item with every affix type — ensures all branches are exercised
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
    let item = Item {
        slot: EquipmentSlot::Weapon,
        rarity: Rarity::Legendary,
        ilvl: 10,
        tier: 5,
        base_name: "Test".to_string(),
        display_name: "Test".to_string(),
        attributes: AttributeBonuses::new(),
        affixes,
        god_item_id: None,
    };
    // power = 0 (attrs) + 2.0+1.5+1.5+1.2+0.5+1.3+1.0+0.8+1.0 = 10.8 -> rounds to 11
    assert_eq!(item.power(), 11);
}

// =========================================================================
// Tier multiplier precision
// =========================================================================

#[test]
fn test_tier_multiplier_t0_is_040() {
    let t0 = tier_multiplier(0);
    assert!(
        (t0 - 0.40).abs() < f64::EPSILON,
        "T0 should be exactly 0.40, got {t0}"
    );
}

#[test]
fn test_tier_multiplier_t9_is_100() {
    let t9 = tier_multiplier(9);
    assert!(
        (t9 - 1.00).abs() < f64::EPSILON,
        "T9 should be exactly 1.00, got {t9}"
    );
}

#[test]
fn test_tier_multiplier_monotonically_increasing() {
    let mut prev = tier_multiplier(0);
    for t in 1u8..=9 {
        let curr = tier_multiplier(t);
        assert!(
            curr > prev,
            "tier_multiplier({t}) = {curr} should be > tier_multiplier({}) = {prev}",
            t - 1
        );
        prev = curr;
    }
}

#[test]
fn test_tier_multiplier_out_of_range_uses_fallback() {
    // Tier 10 and beyond fall back to T5 (0.74)
    let fallback = tier_multiplier(5);
    assert_eq!(
        tier_multiplier(10),
        fallback,
        "Tier 10 should fall back to T5"
    );
    assert_eq!(
        tier_multiplier(100),
        fallback,
        "Tier 100 should fall back to T5"
    );
}

// =========================================================================
// Mythic rarity generation
// =========================================================================

#[test]
fn test_generate_mythic_item_correct_rarity() {
    let item = generate_item(EquipmentSlot::Armor, Rarity::Mythic, 100);
    assert_eq!(item.rarity, Rarity::Mythic);
}

#[test]
fn test_generate_mythic_item_has_affixes() {
    // Mythic follows same rules as Legendary: 4-5 affixes
    let item = generate_item(EquipmentSlot::Armor, Rarity::Mythic, 100);
    assert!(
        (4..=5).contains(&item.affixes.len()),
        "Mythic items should have 4-5 affixes, got {}",
        item.affixes.len()
    );
}

#[test]
fn test_generate_mythic_item_has_display_name() {
    let item = generate_item(EquipmentSlot::Boots, Rarity::Mythic, 100);
    assert!(
        !item.display_name.is_empty(),
        "Mythic item should have display name"
    );
}

#[test]
fn test_generate_mythic_item_attributes_are_positive() {
    for _ in 0..20 {
        let item = generate_item(EquipmentSlot::Ring, Rarity::Mythic, 100);
        assert!(
            item.attributes.total() > 0,
            "Mythic item at ilvl 100 should have positive attributes"
        );
    }
}

// =========================================================================
// ilvl for zone
// =========================================================================

#[test]
fn test_ilvl_for_zone_all_zones() {
    for zone in 1..=10 {
        assert_eq!(
            ilvl_for_zone(zone),
            zone as u32 * 10,
            "Zone {} should give ilvl {}",
            zone,
            zone * 10
        );
    }
}

#[test]
fn test_ilvl_for_zone_11_gives_110() {
    // Zone 11 (The Expanse) should be ilvl 110
    assert_eq!(ilvl_for_zone(11), 110);
}

// =========================================================================
// Mob rarity distribution at prestige boundaries
// =========================================================================

#[test]
fn test_mob_rarity_p0_no_prestige_bonus() {
    let mut rng = ChaCha8Rng::seed_from_u64(1234);
    let trials = 5000;
    let mut common = 0;
    let mut non_legendary = 0;

    for _ in 0..trials {
        match roll_rarity_for_mob(0, 0.0, &mut rng) {
            Rarity::Common => common += 1,
            Rarity::Legendary | Rarity::Mythic => {
                panic!("Legendary/Mythic should never drop from mobs")
            }
            _ => {}
        }
        non_legendary += 1;
    }

    // At P0 base, ~60% common
    let common_pct = common as f64 / non_legendary as f64;
    assert!(
        common_pct > 0.45 && common_pct < 0.75,
        "P0 common rate should be ~60%, got {:.1}%",
        common_pct * 100.0
    );
}

#[test]
fn test_mob_rarity_p20_higher_than_p10() {
    // P10 gives 5% prestige bonus (10 * 0.005), P20 gives 10% = cap
    // So P20 should have fewer commons than P10
    let mut rng = ChaCha8Rng::seed_from_u64(5555);
    let trials = 10_000;

    let mut commons_p10 = 0;
    let mut commons_p20 = 0;

    for _ in 0..trials {
        if roll_rarity_for_mob(10, 0.0, &mut rng) == Rarity::Common {
            commons_p10 += 1;
        }
        if roll_rarity_for_mob(20, 0.0, &mut rng) == Rarity::Common {
            commons_p20 += 1;
        }
    }

    // P20 (10% prestige bonus cap) should have meaningfully fewer commons than P10 (5% bonus)
    assert!(
        commons_p20 < commons_p10,
        "P20 should have fewer commons ({commons_p20}) than P10 ({commons_p10})"
    );
}

#[test]
fn test_mob_rarity_p20_same_as_p30_cap_reached() {
    // Once cap is hit at P20, P30 should behave identically to P20
    // Use same-seed test to verify the distributions are equivalent
    let trials = 5000;
    let mut commons_p20 = 0;
    let mut commons_p30 = 0;

    // Use independent RNG instances with same seed
    let mut rng1 = ChaCha8Rng::seed_from_u64(8888);
    let mut rng2 = ChaCha8Rng::seed_from_u64(8888);

    for _ in 0..trials {
        if roll_rarity_for_mob(20, 0.0, &mut rng1) == Rarity::Common {
            commons_p20 += 1;
        }
        if roll_rarity_for_mob(30, 0.0, &mut rng2) == Rarity::Common {
            commons_p30 += 1;
        }
    }

    // Same seed + same capped parameters = identical results
    assert_eq!(
        commons_p20, commons_p30,
        "P20 and P30 should be identical (both at cap): p20={commons_p20}, p30={commons_p30}"
    );
}

#[test]
fn test_mob_rarity_no_common_floor_at_max_haven() {
    // Even with max Haven bonus (25%), common rate is floored at 20%
    let mut rng = ChaCha8Rng::seed_from_u64(7777);
    let trials = 5000;
    let mut common = 0;

    for _ in 0..trials {
        if roll_rarity_for_mob(20, 25.0, &mut rng) == Rarity::Common {
            common += 1;
        }
    }

    // Common should never be 0, floor is at 20% (MOB_RARITY_COMMON_FLOOR)
    let common_pct = common as f64 / trials as f64;
    assert!(
        common_pct > 0.10,
        "Common should still be present with max prestige + Haven (floor ~20%), got {:.1}%",
        common_pct * 100.0
    );
}

// =========================================================================
// Boss drop rarity distribution — precise thresholds
// =========================================================================

#[test]
fn test_boss_normal_rarity_distribution() {
    // Normal boss: 40% Magic, 35% Rare, 23% Epic, 2% Legendary
    let mut rng = ChaCha8Rng::seed_from_u64(9999);
    let trials = 10_000;
    let mut magic = 0;
    let mut rare = 0;
    let mut epic = 0;
    let mut legendary = 0;

    for _ in 0..trials {
        match roll_rarity_for_boss(false, &mut rng) {
            Rarity::Magic => magic += 1,
            Rarity::Rare => rare += 1,
            Rarity::Epic => epic += 1,
            Rarity::Legendary => legendary += 1,
            r => panic!("Unexpected rarity from boss: {:?}", r),
        }
    }

    // Verify approximate distribution (±5% tolerance)
    let magic_pct = magic as f64 / trials as f64;
    let rare_pct = rare as f64 / trials as f64;
    let epic_pct = epic as f64 / trials as f64;
    let legendary_pct = legendary as f64 / trials as f64;

    assert!(
        magic_pct > 0.35 && magic_pct < 0.45,
        "Normal boss: Magic should be ~40%, got {:.1}%",
        magic_pct * 100.0
    );
    assert!(
        rare_pct > 0.30 && rare_pct < 0.40,
        "Normal boss: Rare should be ~35%, got {:.1}%",
        rare_pct * 100.0
    );
    assert!(
        epic_pct > 0.18 && epic_pct < 0.28,
        "Normal boss: Epic should be ~23%, got {:.1}%",
        epic_pct * 100.0
    );
    assert!(
        legendary_pct > 0.005 && legendary_pct < 0.035,
        "Normal boss: Legendary should be ~2%, got {:.1}%",
        legendary_pct * 100.0
    );
}

#[test]
fn test_boss_final_zone_rarity_distribution() {
    // Final zone boss: 20% Magic, 40% Rare, 35% Epic, 5% Legendary
    let mut rng = ChaCha8Rng::seed_from_u64(11111);
    let trials = 10_000;
    let mut magic = 0;
    let mut rare = 0;
    let mut epic = 0;
    let mut legendary = 0;

    for _ in 0..trials {
        match roll_rarity_for_boss(true, &mut rng) {
            Rarity::Magic => magic += 1,
            Rarity::Rare => rare += 1,
            Rarity::Epic => epic += 1,
            Rarity::Legendary => legendary += 1,
            r => panic!("Unexpected rarity from final boss: {:?}", r),
        }
    }

    let magic_pct = magic as f64 / trials as f64;
    let rare_pct = rare as f64 / trials as f64;
    let epic_pct = epic as f64 / trials as f64;
    let legendary_pct = legendary as f64 / trials as f64;

    assert!(
        magic_pct > 0.15 && magic_pct < 0.25,
        "Final boss: Magic should be ~20%, got {:.1}%",
        magic_pct * 100.0
    );
    assert!(
        rare_pct > 0.35 && rare_pct < 0.45,
        "Final boss: Rare should be ~40%, got {:.1}%",
        rare_pct * 100.0
    );
    assert!(
        epic_pct > 0.30 && epic_pct < 0.40,
        "Final boss: Epic should be ~35%, got {:.1}%",
        epic_pct * 100.0
    );
    assert!(
        legendary_pct > 0.02 && legendary_pct < 0.08,
        "Final boss: Legendary should be ~5%, got {:.1}%",
        legendary_pct * 100.0
    );
}

// =========================================================================
// Boss drop always has correct zone ilvl
// =========================================================================

#[test]
fn test_boss_drop_ilvl_matches_zone() {
    for zone in 1..=10 {
        let item = try_drop_from_boss(zone, zone == 10);
        assert_eq!(
            item.ilvl,
            (zone as u32) * 10,
            "Zone {} boss drop should have ilvl {}",
            zone,
            zone * 10
        );
    }
}

#[test]
fn test_boss_drop_not_common_nor_mythic() {
    let mut rng = ChaCha8Rng::seed_from_u64(22222);
    for _ in 0..500 {
        match roll_rarity_for_boss(false, &mut rng) {
            Rarity::Common => panic!("Boss should never drop Common"),
            Rarity::Mythic => panic!("Boss should never drop Mythic (god items only via debug)"),
            _ => {}
        }
        match roll_rarity_for_boss(true, &mut rng) {
            Rarity::Common => panic!("Final boss should never drop Common"),
            Rarity::Mythic => panic!("Final boss should never drop Mythic"),
            _ => {}
        }
    }
}

// =========================================================================
// Drop chance formula verification
// =========================================================================

#[test]
fn test_drop_chance_formula_at_prestige_5() {
    // 15% + 5*1% = 20%
    let expected = 0.15 + 5.0 * 0.01;
    let actual = drop_chance_for_prestige(5);
    assert!(
        (actual - expected).abs() < f64::EPSILON,
        "P5 drop chance should be {expected}, got {actual}"
    );
}

#[test]
fn test_drop_chance_formula_at_prestige_9() {
    // 15% + 9*1% = 24% (below cap)
    let expected = 0.15 + 9.0 * 0.01;
    let actual = drop_chance_for_prestige(9);
    assert!(
        (actual - expected).abs() < f64::EPSILON,
        "P9 drop chance should be {expected}, got {actual}"
    );
}

#[test]
fn test_drop_chance_formula_at_prestige_10_hits_cap() {
    // 15% + 10*1% = 25% = cap
    let actual = drop_chance_for_prestige(10);
    assert!(
        (actual - 0.25).abs() < f64::EPSILON,
        "P10 drop chance should be 0.25 (cap), got {actual}"
    );
}

// =========================================================================
// God item protection — Mythic vs Mythic replacement
// =========================================================================

#[test]
fn test_mythic_item_can_be_replaced_by_another_mythic() {
    // The protection only blocks NON-Mythic items from replacing Mythic.
    // A new Mythic with higher power SHOULD replace the existing Mythic.
    let mut game_state = GameState::new("Test".to_string(), 0);

    let weak_mythic = Item {
        slot: EquipmentSlot::Armor,
        rarity: Rarity::Mythic,
        ilvl: 100,
        tier: 9,
        base_name: "Weak God Armor".to_string(),
        display_name: "Weak God Armor".to_string(),
        attributes: AttributeBonuses {
            con: 1,
            ..AttributeBonuses::new()
        },
        affixes: vec![],
        god_item_id: None,
    };
    game_state
        .equipment
        .set(EquipmentSlot::Armor, Some(weak_mythic));

    let strong_mythic = Item {
        slot: EquipmentSlot::Armor,
        rarity: Rarity::Mythic,
        ilvl: 100,
        tier: 9,
        base_name: "Strong God Armor".to_string(),
        display_name: "Strong God Armor".to_string(),
        attributes: AttributeBonuses {
            con: 100,
            str: 100,
            ..AttributeBonuses::new()
        },
        affixes: vec![Affix {
            affix_type: AffixType::DamagePercent,
            value: 100.0,
        }],
        god_item_id: None,
    };

    // Mythic can replace Mythic if power is higher
    let replaced = auto_equip_if_better(strong_mythic, &mut game_state);
    assert!(
        replaced,
        "A higher-power Mythic should be able to replace a lower-power Mythic"
    );
    assert_eq!(
        game_state
            .equipment
            .get(EquipmentSlot::Armor)
            .as_ref()
            .unwrap()
            .display_name,
        "Strong God Armor"
    );
}

#[test]
fn test_mythic_item_not_replaced_by_equal_power_mythic() {
    let mut game_state = GameState::new("Test".to_string(), 0);

    let mythic1 = Item {
        slot: EquipmentSlot::Ring,
        rarity: Rarity::Mythic,
        ilvl: 100,
        tier: 9,
        base_name: "Megingjord".to_string(),
        display_name: "Megingjord".to_string(),
        attributes: AttributeBonuses {
            str: 40,
            ..AttributeBonuses::new()
        },
        affixes: vec![],
        god_item_id: None,
    };
    let power1 = mythic1.power();
    game_state.equipment.set(EquipmentSlot::Ring, Some(mythic1));

    let mythic2 = Item {
        slot: EquipmentSlot::Ring,
        rarity: Rarity::Mythic,
        ilvl: 100,
        tier: 9,
        base_name: "Megingjord2".to_string(),
        display_name: "Megingjord2".to_string(),
        attributes: AttributeBonuses {
            str: 40,
            ..AttributeBonuses::new()
        },
        affixes: vec![],
        god_item_id: None,
    };

    assert_eq!(
        power1,
        mythic2.power(),
        "Both mythic items should have equal power"
    );
    let replaced = auto_equip_if_better(mythic2, &mut game_state);
    assert!(
        !replaced,
        "Equal-power Mythic should not replace existing Mythic"
    );
    assert_eq!(
        game_state
            .equipment
            .get(EquipmentSlot::Ring)
            .as_ref()
            .unwrap()
            .display_name,
        "Megingjord"
    );
}

#[test]
fn test_mythic_not_replaced_by_high_power_legendary() {
    let mut game_state = GameState::new("Test".to_string(), 0);

    // Minimal-power Mythic
    let mythic = Item {
        slot: EquipmentSlot::Weapon,
        rarity: Rarity::Mythic,
        ilvl: 100,
        tier: 9,
        base_name: "God Sword".to_string(),
        display_name: "God Sword".to_string(),
        attributes: AttributeBonuses {
            str: 1,
            ..AttributeBonuses::new()
        },
        affixes: vec![],
        god_item_id: None,
    };
    game_state
        .equipment
        .set(EquipmentSlot::Weapon, Some(mythic));

    // Very high power Legendary (but not Mythic)
    let legendary = Item {
        slot: EquipmentSlot::Weapon,
        rarity: Rarity::Legendary,
        ilvl: 100,
        tier: 9,
        base_name: "Uber Blade".to_string(),
        display_name: "Uber Blade".to_string(),
        attributes: AttributeBonuses {
            str: 100,
            dex: 100,
            con: 100,
            ..AttributeBonuses::new()
        },
        affixes: vec![
            Affix {
                affix_type: AffixType::DamagePercent,
                value: 100.0,
            },
            Affix {
                affix_type: AffixType::CritChance,
                value: 50.0,
            },
        ],
        god_item_id: None,
    };

    let replaced = auto_equip_if_better(legendary, &mut game_state);
    assert!(
        !replaced,
        "Mythic should never be auto-replaced by a non-Mythic item"
    );
    assert_eq!(
        game_state
            .equipment
            .get(EquipmentSlot::Weapon)
            .as_ref()
            .unwrap()
            .display_name,
        "God Sword"
    );
}

// =========================================================================
// Item stat_summary — basic sanity
// =========================================================================

#[test]
fn test_stat_summary_empty_item() {
    let item = Item {
        slot: EquipmentSlot::Weapon,
        rarity: Rarity::Common,
        ilvl: 10,
        tier: 5,
        base_name: "Sword".to_string(),
        display_name: "Sword".to_string(),
        attributes: AttributeBonuses::new(),
        affixes: vec![],
        god_item_id: None,
    };
    // No attributes or affixes — should return empty string
    assert_eq!(item.stat_summary(), "");
}

#[test]
fn test_stat_summary_single_attribute() {
    let item = Item {
        slot: EquipmentSlot::Weapon,
        rarity: Rarity::Common,
        ilvl: 10,
        tier: 5,
        base_name: "Sword".to_string(),
        display_name: "Sword".to_string(),
        attributes: AttributeBonuses {
            str: 7,
            ..AttributeBonuses::new()
        },
        affixes: vec![],
        god_item_id: None,
    };
    let summary = item.stat_summary();
    assert!(summary.contains("STR"), "Should contain STR");
    assert!(summary.contains("7"), "Should contain value 7");
}

#[test]
fn test_stat_summary_includes_affix_type_labels() {
    let item = Item {
        slot: EquipmentSlot::Ring,
        rarity: Rarity::Rare,
        ilvl: 10,
        tier: 5,
        base_name: "Ring".to_string(),
        display_name: "Ring".to_string(),
        attributes: AttributeBonuses::new(),
        affixes: vec![
            Affix {
                affix_type: AffixType::DamagePercent,
                value: 15.0,
            },
            Affix {
                affix_type: AffixType::HPBonus,
                value: 50.0,
            },
        ],
        god_item_id: None,
    };
    let summary = item.stat_summary();
    assert!(summary.contains("Dmg"), "Should contain damage %");
    assert!(summary.contains("HP"), "Should contain HP bonus");
}

#[test]
fn test_stat_summary_unknown_affix_skipped() {
    let item = Item {
        slot: EquipmentSlot::Amulet,
        rarity: Rarity::Magic,
        ilvl: 10,
        tier: 5,
        base_name: "Amulet".to_string(),
        display_name: "Amulet".to_string(),
        attributes: AttributeBonuses::new(),
        affixes: vec![Affix {
            affix_type: AffixType::Unknown,
            value: 99.0,
        }],
        god_item_id: None,
    };
    // Unknown affixes should be silently skipped, so we get no contribution
    // from it; the summary should be empty
    assert_eq!(
        item.stat_summary(),
        "",
        "Unknown affixes should not appear in stat_summary"
    );
}

// =========================================================================
// Generation stress test — many items, no panics
// =========================================================================

#[test]
fn test_generate_items_all_combinations_no_panic() {
    let slots = [
        EquipmentSlot::Weapon,
        EquipmentSlot::Armor,
        EquipmentSlot::Helmet,
        EquipmentSlot::Gloves,
        EquipmentSlot::Boots,
        EquipmentSlot::Amulet,
        EquipmentSlot::Ring,
    ];
    let rarities = [
        Rarity::Common,
        Rarity::Magic,
        Rarity::Rare,
        Rarity::Epic,
        Rarity::Legendary,
        Rarity::Mythic,
    ];
    let ilvls = [10u32, 50, 100];

    for &slot in &slots {
        for &rarity in &rarities {
            for &ilvl in &ilvls {
                let item = generate_item(slot, rarity, ilvl);
                assert_eq!(item.slot, slot);
                assert_eq!(item.rarity, rarity);
                assert_eq!(item.ilvl, ilvl);
                assert!(!item.display_name.is_empty());
                assert!(item.tier <= 9);
            }
        }
    }
}
