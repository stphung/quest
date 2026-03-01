//! Integration test: Item Drop -> Auto-Equip Pipeline
//!
//! Tests the full end-to-end flow: drop chance → item generation → power scoring → equip decision.
//! Covers the complete lifecycle from enemy kill drop roll through to equipment slot management.

use quest::core::constants::{ITEM_DROP_BASE_CHANCE, ITEM_DROP_MAX_CHANCE};
use quest::items::drops::{drop_chance_for_prestige, roll_rarity_for_mob, try_drop_from_mob};
use quest::items::generation::generate_item;
use quest::items::scoring::auto_equip_if_better;
use quest::items::types::{Affix, AffixType, AttributeBonuses, EquipmentSlot, Item, Rarity};
use quest::GameState;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

// =========================================================================
// Drop chance: base rate and prestige scaling
// =========================================================================

#[test]
fn test_drop_chance_base_rate_is_15_percent() {
    let chance = drop_chance_for_prestige(0);
    assert!(
        (chance - ITEM_DROP_BASE_CHANCE).abs() < f64::EPSILON,
        "Prestige 0 should give exactly the base chance (15%), got {chance}"
    );
}

#[test]
fn test_drop_chance_scales_linearly_with_prestige() {
    // Each prestige rank adds 1% (0.01)
    for rank in 0..=10 {
        let expected = (ITEM_DROP_BASE_CHANCE + rank as f64 * 0.01).min(ITEM_DROP_MAX_CHANCE);
        let actual = drop_chance_for_prestige(rank);
        assert!(
            (actual - expected).abs() < f64::EPSILON,
            "Prestige {rank}: expected {expected}, got {actual}"
        );
    }
}

#[test]
fn test_drop_chance_caps_at_25_percent() {
    // Prestige 10 = 15% + 10% = 25% (the cap)
    assert!(
        (drop_chance_for_prestige(10) - ITEM_DROP_MAX_CHANCE).abs() < f64::EPSILON,
        "Prestige 10 should hit the cap"
    );

    // Prestige 20 should still be capped
    assert!(
        (drop_chance_for_prestige(20) - ITEM_DROP_MAX_CHANCE).abs() < f64::EPSILON,
        "Prestige 20 should still be capped at 25%"
    );

    // Extreme prestige should still be capped
    assert!(
        (drop_chance_for_prestige(100) - ITEM_DROP_MAX_CHANCE).abs() < f64::EPSILON,
        "Prestige 100 should still be capped at 25%"
    );
}

#[test]
fn test_drop_chance_never_exceeds_cap() {
    // Test a wide range of prestige ranks to ensure the cap holds
    for rank in 0..=200 {
        let chance = drop_chance_for_prestige(rank);
        assert!(
            chance <= ITEM_DROP_MAX_CHANCE,
            "Prestige {rank} gave chance {chance}, which exceeds max {ITEM_DROP_MAX_CHANCE}"
        );
    }
}

// =========================================================================
// try_drop_item: actual drop frequency matches expected rates
// =========================================================================

#[test]
fn test_try_drop_item_frequency_at_prestige_zero() {
    let game_state = GameState::new("Drop Test".to_string(), 0);
    let trials = 2000;
    let drops: usize = (0..trials)
        .filter(|_| try_drop_from_mob(&game_state, 1, 0.0, 0.0).is_some())
        .count();

    // Expected: 15% = 300 out of 2000, allow wide margin for randomness
    let low = (trials as f64 * 0.08) as usize;
    let high = (trials as f64 * 0.22) as usize;
    assert!(
        drops >= low && drops <= high,
        "Expected ~15% drop rate at P0, got {drops}/{trials} ({:.1}%)",
        drops as f64 / trials as f64 * 100.0
    );
}

#[test]
fn test_try_drop_item_frequency_increases_with_prestige() {
    let trials = 2000;

    let mut state_p0 = GameState::new("P0".to_string(), 0);
    state_p0.prestige_rank = 0;
    let drops_p0: usize = (0..trials)
        .filter(|_| try_drop_from_mob(&state_p0, 1, 0.0, 0.0).is_some())
        .count();

    let mut state_p5 = GameState::new("P5".to_string(), 0);
    state_p5.prestige_rank = 5;
    let drops_p5: usize = (0..trials)
        .filter(|_| try_drop_from_mob(&state_p5, 1, 0.0, 0.0).is_some())
        .count();

    assert!(
        drops_p5 > drops_p0,
        "P5 ({drops_p5} drops) should get more drops than P0 ({drops_p0} drops)"
    );
}

// =========================================================================
// generate_drop: produced items have valid structure
// =========================================================================

#[test]
fn test_generated_items_have_correct_slot() {
    let slots = [
        EquipmentSlot::Weapon,
        EquipmentSlot::Armor,
        EquipmentSlot::Helmet,
        EquipmentSlot::Gloves,
        EquipmentSlot::Boots,
        EquipmentSlot::Amulet,
        EquipmentSlot::Ring,
    ];

    for slot in slots {
        let item = generate_item(slot, Rarity::Rare, 10);
        assert_eq!(
            item.slot, slot,
            "Generated item should have the requested slot"
        );
    }
}

#[test]
fn test_generated_items_have_correct_rarity() {
    let rarities = [
        Rarity::Common,
        Rarity::Magic,
        Rarity::Rare,
        Rarity::Epic,
        Rarity::Legendary,
    ];

    for rarity in rarities {
        let item = generate_item(EquipmentSlot::Weapon, rarity, 10);
        assert_eq!(
            item.rarity, rarity,
            "Generated item should have the requested rarity"
        );
    }
}

#[test]
fn test_generated_items_always_have_positive_attributes() {
    // Every generated item must contribute some attribute bonuses
    for _ in 0..100 {
        let item = generate_item(EquipmentSlot::Weapon, Rarity::Common, 1);
        assert!(
            item.attributes.total() > 0,
            "Every generated item should have at least some attributes"
        );
    }
}

#[test]
fn test_generated_items_have_nonempty_display_name() {
    let item = generate_item(EquipmentSlot::Boots, Rarity::Epic, 15);
    assert!(
        !item.display_name.is_empty(),
        "Generated items must have a display name"
    );
}

// =========================================================================
// Rarity → affix count contract
// =========================================================================

#[test]
fn test_affix_count_contract_across_all_rarities() {
    // Run multiple times to cover the random ranges
    for _ in 0..50 {
        let common = generate_item(EquipmentSlot::Weapon, Rarity::Common, 1);
        assert_eq!(common.affixes.len(), 0, "Common items: 0 affixes");

        let magic = generate_item(EquipmentSlot::Weapon, Rarity::Magic, 5);
        assert_eq!(magic.affixes.len(), 1, "Magic items: exactly 1 affix");

        let rare = generate_item(EquipmentSlot::Weapon, Rarity::Rare, 10);
        assert!(
            (2..=3).contains(&rare.affixes.len()),
            "Rare items: 2-3 affixes, got {}",
            rare.affixes.len()
        );

        let epic = generate_item(EquipmentSlot::Weapon, Rarity::Epic, 15);
        assert!(
            (3..=4).contains(&epic.affixes.len()),
            "Epic items: 3-4 affixes, got {}",
            epic.affixes.len()
        );

        let legendary = generate_item(EquipmentSlot::Weapon, Rarity::Legendary, 20);
        assert!(
            (4..=5).contains(&legendary.affixes.len()),
            "Legendary items: 4-5 affixes, got {}",
            legendary.affixes.len()
        );
    }
}

// =========================================================================
// Rarity → attribute strength (higher rarity = stronger attributes on avg)
// =========================================================================

#[test]
fn test_higher_rarity_produces_higher_average_attribute_total() {
    let sample_avg = |rarity: Rarity| -> f64 {
        let n = 2000;
        // Use ilvl 50 so tier multiplier differences are meaningful
        let sum: u32 = (0..n)
            .map(|_| {
                generate_item(EquipmentSlot::Weapon, rarity, 50)
                    .attributes
                    .total()
            })
            .sum();
        sum as f64 / n as f64
    };

    let common_avg = sample_avg(Rarity::Common);
    let magic_avg = sample_avg(Rarity::Magic);
    let rare_avg = sample_avg(Rarity::Rare);
    let epic_avg = sample_avg(Rarity::Epic);
    let legendary_avg = sample_avg(Rarity::Legendary);

    assert!(
        common_avg < magic_avg,
        "Common avg ({common_avg:.1}) should be < Magic avg ({magic_avg:.1})"
    );
    assert!(
        magic_avg < rare_avg,
        "Magic avg ({magic_avg:.1}) should be < Rare avg ({rare_avg:.1})"
    );
    assert!(
        rare_avg < epic_avg,
        "Rare avg ({rare_avg:.1}) should be < Epic avg ({epic_avg:.1})"
    );
    assert!(
        epic_avg < legendary_avg,
        "Epic avg ({epic_avg:.1}) should be < Legendary avg ({legendary_avg:.1})"
    );
}

// =========================================================================
// power(): higher rarity items have higher power on average
// =========================================================================

#[test]
fn test_power_increases_with_rarity_on_average() {
    let sample_avg_power = |rarity: Rarity| -> f64 {
        let n = 2000;
        let sum: u32 = (0..n)
            .map(|_| generate_item(EquipmentSlot::Weapon, rarity, 10).power())
            .sum();
        sum as f64 / n as f64
    };

    let common_avg = sample_avg_power(Rarity::Common);
    let magic_avg = sample_avg_power(Rarity::Magic);
    let rare_avg = sample_avg_power(Rarity::Rare);
    let epic_avg = sample_avg_power(Rarity::Epic);
    let legendary_avg = sample_avg_power(Rarity::Legendary);

    assert!(
        common_avg < magic_avg,
        "Common power ({common_avg:.1}) < Magic ({magic_avg:.1})"
    );
    assert!(
        magic_avg < rare_avg,
        "Magic power ({magic_avg:.1}) < Rare ({rare_avg:.1})"
    );
    assert!(
        rare_avg < epic_avg,
        "Rare power ({rare_avg:.1}) < Epic ({epic_avg:.1})"
    );
    assert!(
        epic_avg < legendary_avg,
        "Epic power ({epic_avg:.1}) < Legendary ({legendary_avg:.1})"
    );
}

#[test]
fn test_power_is_deterministic_for_same_item() {
    let item = generate_item(EquipmentSlot::Weapon, Rarity::Rare, 10);

    let power1 = item.power();
    let power2 = item.power();

    assert_eq!(
        power1, power2,
        "Scoring the same item twice should produce identical results: {power1} vs {power2}"
    );
}

// =========================================================================
// Auto-equip decision logic
// =========================================================================

#[test]
fn test_auto_equip_into_empty_slot_always_succeeds() {
    let slots = [
        EquipmentSlot::Weapon,
        EquipmentSlot::Armor,
        EquipmentSlot::Helmet,
        EquipmentSlot::Gloves,
        EquipmentSlot::Boots,
        EquipmentSlot::Amulet,
        EquipmentSlot::Ring,
    ];

    for slot in slots {
        let mut game_state = GameState::new("Empty Slot Test".to_string(), 0);

        // Verify slot is empty
        assert!(game_state.equipment.get(slot).is_none());

        // Any item with positive attributes should equip into an empty slot
        let item = generate_item(slot, Rarity::Common, 1);
        let equipped = auto_equip_if_better(item, &mut game_state);

        assert!(
            equipped,
            "Item should always equip into empty slot {:?}",
            slot
        );
        assert!(
            game_state.equipment.get(slot).is_some(),
            "Slot {:?} should now be occupied",
            slot
        );
    }
}

#[test]
fn test_auto_equip_higher_power_item_replaces_lower() {
    let mut game_state = GameState::new("Replace Test".to_string(), 0);

    // Equip a weak common item
    let weak = Item {
        slot: EquipmentSlot::Weapon,
        rarity: Rarity::Common,
        ilvl: 10,
        tier: 5,
        base_name: "Rusty Sword".to_string(),
        display_name: "Rusty Sword".to_string(),
        attributes: AttributeBonuses {
            str: 1,
            ..AttributeBonuses::new()
        },
        affixes: vec![],
        god_item_id: None,
    };
    let weak_power = weak.power();
    auto_equip_if_better(weak, &mut game_state);

    // Try a much stronger legendary item
    let strong = Item {
        slot: EquipmentSlot::Weapon,
        rarity: Rarity::Legendary,
        ilvl: 10,
        tier: 5,
        base_name: "Legendary Blade".to_string(),
        display_name: "Legendary Blade".to_string(),
        attributes: AttributeBonuses {
            str: 15,
            dex: 10,
            ..AttributeBonuses::new()
        },
        affixes: vec![
            Affix {
                affix_type: AffixType::DamagePercent,
                value: 40.0,
            },
            Affix {
                affix_type: AffixType::CritChance,
                value: 30.0,
            },
        ],
        god_item_id: None,
    };
    let strong_power = strong.power();
    let replaced = auto_equip_if_better(strong, &mut game_state);

    assert!(
        replaced,
        "Stronger item (power {strong_power}) should replace weaker (power {weak_power})"
    );
    assert_eq!(
        game_state
            .equipment
            .get(EquipmentSlot::Weapon)
            .as_ref()
            .unwrap()
            .display_name,
        "Legendary Blade"
    );
}

#[test]
fn test_auto_equip_lower_power_item_does_not_replace() {
    let mut game_state = GameState::new("No Replace Test".to_string(), 0);

    // Equip a strong legendary item first
    let strong = Item {
        slot: EquipmentSlot::Armor,
        rarity: Rarity::Legendary,
        ilvl: 10,
        tier: 5,
        base_name: "Dragon Plate".to_string(),
        display_name: "Dragon Plate".to_string(),
        attributes: AttributeBonuses {
            con: 15,
            str: 10,
            ..AttributeBonuses::new()
        },
        affixes: vec![Affix {
            affix_type: AffixType::DamageReduction,
            value: 40.0,
        }],
        god_item_id: None,
    };
    auto_equip_if_better(strong, &mut game_state);

    // Try a weak common item
    let weak = Item {
        slot: EquipmentSlot::Armor,
        rarity: Rarity::Common,
        ilvl: 10,
        tier: 5,
        base_name: "Cloth Shirt".to_string(),
        display_name: "Cloth Shirt".to_string(),
        attributes: AttributeBonuses {
            con: 1,
            ..AttributeBonuses::new()
        },
        affixes: vec![],
        god_item_id: None,
    };
    let replaced = auto_equip_if_better(weak, &mut game_state);

    assert!(
        !replaced,
        "Weaker item should NOT replace stronger equipped item"
    );
    assert_eq!(
        game_state
            .equipment
            .get(EquipmentSlot::Armor)
            .as_ref()
            .unwrap()
            .display_name,
        "Dragon Plate"
    );
}

#[test]
fn test_auto_equip_across_different_slots_is_independent() {
    let mut game_state = GameState::new("Multi Slot".to_string(), 0);

    // Equip items in different slots
    let weapon = Item {
        slot: EquipmentSlot::Weapon,
        rarity: Rarity::Rare,
        ilvl: 10,
        tier: 5,
        base_name: "Sword".to_string(),
        display_name: "Fine Sword".to_string(),
        attributes: AttributeBonuses {
            str: 6,
            ..AttributeBonuses::new()
        },
        affixes: vec![],
        god_item_id: None,
    };
    let helmet = Item {
        slot: EquipmentSlot::Helmet,
        rarity: Rarity::Magic,
        ilvl: 10,
        tier: 5,
        base_name: "Helm".to_string(),
        display_name: "Iron Helm".to_string(),
        attributes: AttributeBonuses {
            con: 3,
            ..AttributeBonuses::new()
        },
        affixes: vec![],
        god_item_id: None,
    };
    let boots = Item {
        slot: EquipmentSlot::Boots,
        rarity: Rarity::Epic,
        ilvl: 10,
        tier: 5,
        base_name: "Boots".to_string(),
        display_name: "Swift Boots".to_string(),
        attributes: AttributeBonuses {
            dex: 8,
            ..AttributeBonuses::new()
        },
        affixes: vec![],
        god_item_id: None,
    };

    assert!(auto_equip_if_better(weapon, &mut game_state));
    assert!(auto_equip_if_better(helmet, &mut game_state));
    assert!(auto_equip_if_better(boots, &mut game_state));

    // All three slots should be independently filled
    assert!(game_state.equipment.get(EquipmentSlot::Weapon).is_some());
    assert!(game_state.equipment.get(EquipmentSlot::Helmet).is_some());
    assert!(game_state.equipment.get(EquipmentSlot::Boots).is_some());

    // Other slots should still be empty
    assert!(game_state.equipment.get(EquipmentSlot::Armor).is_none());
    assert!(game_state.equipment.get(EquipmentSlot::Gloves).is_none());
    assert!(game_state.equipment.get(EquipmentSlot::Amulet).is_none());
    assert!(game_state.equipment.get(EquipmentSlot::Ring).is_none());
}

// =========================================================================
// Full pipeline: generate → power → equip → upgrade chain
// =========================================================================

#[test]
fn test_full_pipeline_generate_power_equip() {
    let mut game_state = GameState::new("Pipeline Hero".to_string(), 0);

    // Generate a common item and equip it
    let common_item = generate_item(EquipmentSlot::Weapon, Rarity::Common, 5);
    assert!(common_item.attributes.total() > 0);
    let common_power = common_item.power();
    assert!(common_power > 0);

    let equipped = auto_equip_if_better(common_item, &mut game_state);
    assert!(equipped, "Common item should equip into empty weapon slot");

    // Generate a legendary item and verify it replaces the common
    let legendary_item = generate_item(EquipmentSlot::Weapon, Rarity::Legendary, 20);
    let legendary_power = legendary_item.power();

    // Legendary should outpower common (on average, overwhelmingly so)
    assert!(
        legendary_power > common_power,
        "Legendary (power {legendary_power}) should outpower Common (power {common_power})"
    );

    let replaced = auto_equip_if_better(legendary_item, &mut game_state);
    assert!(
        replaced,
        "Legendary should replace Common in the weapon slot"
    );

    // Verify the equipped item is indeed the legendary
    let equipped_item = game_state
        .equipment
        .get(EquipmentSlot::Weapon)
        .as_ref()
        .unwrap();
    assert_eq!(equipped_item.rarity, Rarity::Legendary);
}

#[test]
fn test_full_pipeline_progressive_upgrade_chain() {
    let mut game_state = GameState::new("Upgrade Chain".to_string(), 0);

    // Simulate a player finding progressively better items and equipping them.
    // We use hand-crafted items to guarantee the upgrade ordering.
    let items = vec![
        Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Common,
            ilvl: 10,
            tier: 5,
            base_name: "Tier1".to_string(),
            display_name: "Wooden Sword".to_string(),
            attributes: AttributeBonuses {
                str: 1,
                ..AttributeBonuses::new()
            },
            affixes: vec![],
            god_item_id: None,
        },
        Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Magic,
            ilvl: 10,
            tier: 5,
            base_name: "Tier2".to_string(),
            display_name: "Iron Sword".to_string(),
            attributes: AttributeBonuses {
                str: 4,
                ..AttributeBonuses::new()
            },
            affixes: vec![Affix {
                affix_type: AffixType::DamagePercent,
                value: 8.0,
            }],
            god_item_id: None,
        },
        Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Rare,
            ilvl: 10,
            tier: 5,
            base_name: "Tier3".to_string(),
            display_name: "Steel Longsword".to_string(),
            attributes: AttributeBonuses {
                str: 6,
                dex: 3,
                ..AttributeBonuses::new()
            },
            affixes: vec![
                Affix {
                    affix_type: AffixType::DamagePercent,
                    value: 15.0,
                },
                Affix {
                    affix_type: AffixType::CritChance,
                    value: 12.0,
                },
            ],
            god_item_id: None,
        },
        Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Legendary,
            ilvl: 10,
            tier: 5,
            base_name: "Tier4".to_string(),
            display_name: "Excalibur".to_string(),
            attributes: AttributeBonuses {
                str: 15,
                dex: 10,
                con: 8,
                ..AttributeBonuses::new()
            },
            affixes: vec![
                Affix {
                    affix_type: AffixType::DamagePercent,
                    value: 45.0,
                },
                Affix {
                    affix_type: AffixType::CritChance,
                    value: 35.0,
                },
                Affix {
                    affix_type: AffixType::CritMultiplier,
                    value: 40.0,
                },
                Affix {
                    affix_type: AffixType::AttackSpeed,
                    value: 30.0,
                },
            ],
            god_item_id: None,
        },
    ];

    let mut prev_power = 0u32;
    for (i, item) in items.into_iter().enumerate() {
        let item_name = item.display_name.clone();
        let item_power = item.power();

        assert!(
            item_power > prev_power,
            "Tier {} ({item_name}, power {item_power}) should outpower previous ({prev_power})",
            i + 1
        );

        let equipped = auto_equip_if_better(item, &mut game_state);
        assert!(
            equipped,
            "Tier {} ({item_name}) should equip as upgrade",
            i + 1
        );

        prev_power = item_power;
    }

    // Final state: Excalibur should be equipped
    assert_eq!(
        game_state
            .equipment
            .get(EquipmentSlot::Weapon)
            .as_ref()
            .unwrap()
            .display_name,
        "Excalibur"
    );
}

#[test]
fn test_full_pipeline_equip_all_seven_slots_from_drops() {
    // Simulate a player who keeps getting drops until all 7 slots are filled
    let mut game_state = GameState::new("Full Gear".to_string(), 0);
    game_state.prestige_rank = 5; // Moderate prestige for decent drop rate

    let slots = [
        EquipmentSlot::Weapon,
        EquipmentSlot::Armor,
        EquipmentSlot::Helmet,
        EquipmentSlot::Gloves,
        EquipmentSlot::Boots,
        EquipmentSlot::Amulet,
        EquipmentSlot::Ring,
    ];

    // Generate and equip one item per slot
    for slot in slots {
        let item = generate_item(slot, Rarity::Rare, 10);
        let equipped = auto_equip_if_better(item, &mut game_state);
        assert!(equipped, "Should equip into empty slot {:?}", slot);
    }

    // Verify all slots are filled
    for slot in slots {
        assert!(
            game_state.equipment.get(slot).is_some(),
            "Slot {:?} should be equipped",
            slot
        );
    }

    // Verify iterator returns all 7
    assert_eq!(game_state.equipment.iter_equipped().count(), 7);
}

// =========================================================================
// Rarity distribution: roll_rarity
// =========================================================================

#[test]
fn test_roll_rarity_covers_all_mob_tiers() {
    // Over enough rolls, all 4 mob rarity tiers should appear (no Legendary from mobs)
    // Legendary only drops from bosses now
    let mut rng = ChaCha8Rng::seed_from_u64(800);
    let mut seen = std::collections::HashSet::new();

    for _ in 0..5_000 {
        let rarity = roll_rarity_for_mob(0, 0.0, &mut rng);
        seen.insert(format!("{:?}", rarity));
        if seen.len() == 4 {
            break;
        }
    }

    assert_eq!(
        seen.len(),
        4,
        "All 4 mob rarity tiers should be reachable (Common, Magic, Rare, Epic). Got: {:?}",
        seen
    );
    assert!(
        !seen.contains("Legendary"),
        "Legendary should not drop from mobs"
    );
}

#[test]
fn test_roll_rarity_prestige_bonus_shifts_toward_higher_tiers() {
    let mut rng = ChaCha8Rng::seed_from_u64(900);
    let trials = 5_000;

    let mut common_p0 = 0usize;
    let mut common_p10 = 0usize;

    for _ in 0..trials {
        if roll_rarity_for_mob(0, 0.0, &mut rng) == Rarity::Common {
            common_p0 += 1;
        }
        if roll_rarity_for_mob(10, 0.0, &mut rng) == Rarity::Common {
            common_p10 += 1;
        }
    }

    // Prestige 10 gives 10% bonus, reducing common from ~55% to ~45%
    assert!(
        common_p10 < common_p0,
        "P10 should have fewer commons ({common_p10}) than P0 ({common_p0})"
    );

    // Verify the magnitude is meaningful (at least ~2.5% difference = 125 in 5k)
    let diff = common_p0 as i64 - common_p10 as i64;
    assert!(
        diff > 100,
        "Prestige bonus should meaningfully reduce common rate. Diff = {diff}"
    );
}

// =========================================================================
// Affix quality scales with rarity through the pipeline
// =========================================================================

#[test]
fn test_affix_values_scale_with_rarity() {
    // Higher rarity items should have higher affix values on average
    let avg_affix_value = |rarity: Rarity| -> f64 {
        let mut total_value = 0.0;
        let mut total_affixes = 0;
        for _ in 0..200 {
            let item = generate_item(EquipmentSlot::Weapon, rarity, 10);
            for affix in &item.affixes {
                total_value += affix.value;
                total_affixes += 1;
            }
        }
        if total_affixes == 0 {
            return 0.0;
        }
        total_value / total_affixes as f64
    };

    // Common has no affixes, so skip it
    let magic_avg = avg_affix_value(Rarity::Magic);
    let rare_avg = avg_affix_value(Rarity::Rare);
    let epic_avg = avg_affix_value(Rarity::Epic);
    let legendary_avg = avg_affix_value(Rarity::Legendary);

    assert!(
        magic_avg < rare_avg,
        "Magic affix avg ({magic_avg:.1}) < Rare ({rare_avg:.1})"
    );
    assert!(
        rare_avg < epic_avg,
        "Rare affix avg ({rare_avg:.1}) < Epic ({epic_avg:.1})"
    );
    assert!(
        epic_avg < legendary_avg,
        "Epic affix avg ({epic_avg:.1}) < Legendary ({legendary_avg:.1})"
    );
}

// =========================================================================
// Power with affix type weight ordering
// =========================================================================

#[test]
fn test_damage_percent_affix_outpowers_hp_bonus_affix_at_same_value() {
    let dmg_item = Item {
        slot: EquipmentSlot::Ring,
        rarity: Rarity::Magic,
        ilvl: 10,
        tier: 5,
        base_name: "DmgRing".to_string(),
        display_name: "Ring of Damage".to_string(),
        attributes: AttributeBonuses::new(),
        affixes: vec![Affix {
            affix_type: AffixType::DamagePercent,
            value: 10.0,
        }],
        god_item_id: None,
    };

    let hp_item = Item {
        slot: EquipmentSlot::Ring,
        rarity: Rarity::Magic,
        ilvl: 10,
        tier: 5,
        base_name: "HPRing".to_string(),
        display_name: "Ring of Health".to_string(),
        attributes: AttributeBonuses::new(),
        affixes: vec![Affix {
            affix_type: AffixType::HPBonus,
            value: 10.0,
        }],
        god_item_id: None,
    };

    let dmg_power = dmg_item.power();
    let hp_power = hp_item.power();

    // DamagePercent weight = 2.0, HPBonus weight = 0.5
    // So 10*2.0 = 20 vs 10*0.5 = 5
    assert!(
        dmg_power > hp_power,
        "DamagePercent ({dmg_power}) should outpower HPBonus ({hp_power}) at same value"
    );
    assert_eq!(
        dmg_power, 20,
        "DamagePercent power should be 20, got {dmg_power}"
    );
    assert_eq!(hp_power, 5, "HPBonus power should be 5, got {hp_power}");
}

// =========================================================================
// Edge case: zero-attribute item with only affixes
// =========================================================================

#[test]
fn test_power_affix_only_item_is_positive() {
    let item = Item {
        slot: EquipmentSlot::Amulet,
        rarity: Rarity::Rare,
        ilvl: 10,
        tier: 5,
        base_name: "Pure Affix".to_string(),
        display_name: "Amulet of Power".to_string(),
        attributes: AttributeBonuses::new(), // zero attributes
        affixes: vec![
            Affix {
                affix_type: AffixType::CritChance,
                value: 15.0,
            },
            Affix {
                affix_type: AffixType::CritMultiplier,
                value: 20.0,
            },
        ],
        god_item_id: None,
    };

    let power = item.power();
    // CritChance: 15 * 1.5 = 22.5, CritMultiplier: 20 * 1.5 = 30.0, total = 52.5 → rounds to 53
    let expected = 53u32;
    assert_eq!(power, expected, "Power should be {expected}, got {power}");
    assert!(power > 0);
}

// =========================================================================
// Pipeline with prestige: higher prestige -> better items on average
// =========================================================================

#[test]
fn test_pipeline_prestige_produces_better_average_power() {
    // Higher prestige shifts rarity distribution, so average item power should be better
    let avg_power = |prestige_rank: u32, seed: u64| -> f64 {
        use rand::SeedableRng;
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        let n = 3000;
        let sum: u32 = (0..n)
            .map(|_| {
                let rarity = roll_rarity_for_mob(prestige_rank, 0.0, &mut rng);
                generate_item(EquipmentSlot::Weapon, rarity, 10).power()
            })
            .sum();
        sum as f64 / n as f64
    };

    let avg_p0 = avg_power(0, 42);
    let avg_p10 = avg_power(10, 42);

    assert!(
        avg_p10 > avg_p0,
        "P10 average power ({avg_p10:.1}) should exceed P0 average ({avg_p0:.1})"
    );
}

// =========================================================================
// Full pipeline from try_drop_item: items from actual drops are valid
// =========================================================================

#[test]
fn test_try_drop_item_produces_valid_equippable_items() {
    let mut game_state = GameState::new("Drop Equip".to_string(), 0);
    game_state.prestige_rank = 5;

    let mut items_equipped = 0;
    // Run many trials to collect some actual drops
    for _ in 0..1000 {
        if let Some(item) = try_drop_from_mob(&game_state, 1, 0.0, 0.0) {
            // Verify basic item validity
            assert!(!item.display_name.is_empty());
            assert!(item.attributes.total() > 0);

            // Power should be positive for any valid item
            let power = item.power();
            assert!(
                power > 0,
                "Dropped item should have positive power, got {power}"
            );

            // Try to equip it
            let slot = item.slot;
            if auto_equip_if_better(item, &mut game_state) {
                items_equipped += 1;
                assert!(game_state.equipment.get(slot).is_some());
            }
        }
    }

    // We should have equipped at least a few items from 1000 trials at 20% drop rate
    assert!(
        items_equipped > 0,
        "Should have equipped at least one item from drops"
    );
}

// =========================================================================
// Auto-equip with equal-power items: tie goes to existing (no replacement)
// =========================================================================

#[test]
fn test_auto_equip_equal_power_does_not_replace() {
    let mut game_state = GameState::new("Tie Test".to_string(), 0);

    let item1 = Item {
        slot: EquipmentSlot::Ring,
        rarity: Rarity::Common,
        ilvl: 10,
        tier: 5,
        base_name: "Ring1".to_string(),
        display_name: "Ring Alpha".to_string(),
        attributes: AttributeBonuses {
            str: 3,
            ..AttributeBonuses::new()
        },
        affixes: vec![],
        god_item_id: None,
    };

    // Create an identical item (same stats, different name)
    let item2 = Item {
        slot: EquipmentSlot::Ring,
        rarity: Rarity::Common,
        ilvl: 10,
        tier: 5,
        base_name: "Ring2".to_string(),
        display_name: "Ring Beta".to_string(),
        attributes: AttributeBonuses {
            str: 3,
            ..AttributeBonuses::new()
        },
        affixes: vec![],
        god_item_id: None,
    };

    // Verify both items have the same power
    let power1 = item1.power();
    let power2 = item2.power();
    assert_eq!(
        power1, power2,
        "Items should have equal power: {power1} vs {power2}"
    );

    // Equip first
    assert!(auto_equip_if_better(item1, &mut game_state));

    // Second item with equal power should NOT replace (strictly greater required)
    let replaced = auto_equip_if_better(item2, &mut game_state);
    assert!(
        !replaced,
        "Equal-power item should NOT replace existing (tie goes to incumbent)"
    );
    assert_eq!(
        game_state
            .equipment
            .get(EquipmentSlot::Ring)
            .as_ref()
            .unwrap()
            .display_name,
        "Ring Alpha"
    );
}

// =========================================================================
// Tier system integration tests
// =========================================================================

#[test]
fn test_tier_stored_on_generated_items() {
    // Items from generate_item() should have a tier in the 0-9 range
    for _ in 0..50 {
        let item = generate_item(EquipmentSlot::Weapon, Rarity::Rare, 50);
        assert!(item.tier <= 9, "Tier should be 0-9, got {}", item.tier);
    }
}

#[test]
fn test_tier_serialization_roundtrip() {
    // Tier persists through JSON save/load
    let item = Item {
        slot: EquipmentSlot::Weapon,
        rarity: Rarity::Epic,
        ilvl: 70,
        tier: 7,
        base_name: "Sword".to_string(),
        display_name: "Crystal Blade".to_string(),
        attributes: AttributeBonuses {
            str: 10,
            ..AttributeBonuses::new()
        },
        affixes: vec![],
        god_item_id: None,
    };

    let json = serde_json::to_string(&item).unwrap();
    let loaded: Item = serde_json::from_str(&json).unwrap();
    assert_eq!(
        loaded.tier, 7,
        "Tier should survive serialization roundtrip"
    );
}

#[test]
fn test_tier_default_for_legacy_items() {
    // Items without a tier field in JSON should deserialize as T1
    let json = r#"{
        "slot": "Weapon",
        "rarity": "Legendary",
        "ilvl": 100,
        "base_name": "Old Sword",
        "display_name": "Old Sword",
        "attributes": {"str": 5, "dex": 0, "con": 0, "int": 0, "wis": 0, "cha": 0},
        "affixes": [],
        "god_item_id": null
    }"#;
    let item: Item = serde_json::from_str(json).unwrap();
    assert_eq!(
        item.tier, 1,
        "Legacy items without tier should default to T1"
    );
}

#[test]
fn test_tier_distribution_across_many_items() {
    // Over many generated items, T0 should be most common and T9 very rare
    let mut counts = [0u32; 10];
    for _ in 0..1000 {
        let item = generate_item(EquipmentSlot::Armor, Rarity::Common, 10);
        counts[item.tier as usize] += 1;
    }
    // T0 should be most frequent
    assert!(
        counts[0] > counts[9],
        "T0 ({}) should be more common than T9 ({})",
        counts[0],
        counts[9]
    );
    // T0-T3 should be the vast majority (87% expected)
    let low_tiers: u32 = counts[0..=3].iter().sum();
    assert!(
        low_tiers > 750,
        "T0-T3 should be >75% of drops, got {}/1000",
        low_tiers
    );
}
