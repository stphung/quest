//! Integration test: Item Drop -> Auto-Equip Pipeline
//!
//! Tests the full end-to-end flow: drop chance → item generation → scoring → equip decision.
//! Covers the complete lifecycle from enemy kill drop roll through to equipment slot management.

use quest::character::attributes::AttributeType;
use quest::core::constants::{ITEM_DROP_BASE_CHANCE, ITEM_DROP_MAX_CHANCE};
use quest::items::drops::{drop_chance_for_prestige, roll_rarity, try_drop_item};
use quest::items::generation::generate_item;
use quest::items::scoring::{auto_equip_if_better, score_item};
use quest::items::types::{Affix, AffixType, AttributeBonuses, EquipmentSlot, Item, Rarity};
use quest::GameState;

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
    let trials = 5000;
    let drops: usize = (0..trials)
        .filter(|_| try_drop_item(&game_state).is_some())
        .count();

    // Expected: 15% = 750 out of 5000, allow wide margin for randomness
    let low = (trials as f64 * 0.10) as usize;
    let high = (trials as f64 * 0.20) as usize;
    assert!(
        drops >= low && drops <= high,
        "Expected ~15% drop rate at P0, got {drops}/{trials} ({:.1}%)",
        drops as f64 / trials as f64 * 100.0
    );
}

#[test]
fn test_try_drop_item_frequency_increases_with_prestige() {
    let trials = 5000;

    let mut state_p0 = GameState::new("P0".to_string(), 0);
    state_p0.prestige_rank = 0;
    let drops_p0: usize = (0..trials)
        .filter(|_| try_drop_item(&state_p0).is_some())
        .count();

    let mut state_p5 = GameState::new("P5".to_string(), 0);
    state_p5.prestige_rank = 5;
    let drops_p5: usize = (0..trials)
        .filter(|_| try_drop_item(&state_p5).is_some())
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
        let n = 200;
        let sum: u32 = (0..n)
            .map(|_| {
                generate_item(EquipmentSlot::Weapon, rarity, 10)
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
// score_item: higher rarity items score higher on average
// =========================================================================

#[test]
fn test_score_increases_with_rarity_on_average() {
    let game_state = GameState::new("Score Test".to_string(), 0);

    let sample_avg_score = |rarity: Rarity| -> f64 {
        let n = 200;
        let sum: f64 = (0..n)
            .map(|_| {
                let item = generate_item(EquipmentSlot::Weapon, rarity, 10);
                score_item(&item, &game_state)
            })
            .sum();
        sum / n as f64
    };

    let common_avg = sample_avg_score(Rarity::Common);
    let magic_avg = sample_avg_score(Rarity::Magic);
    let rare_avg = sample_avg_score(Rarity::Rare);
    let epic_avg = sample_avg_score(Rarity::Epic);
    let legendary_avg = sample_avg_score(Rarity::Legendary);

    assert!(
        common_avg < magic_avg,
        "Common score ({common_avg:.1}) < Magic ({magic_avg:.1})"
    );
    assert!(
        magic_avg < rare_avg,
        "Magic score ({magic_avg:.1}) < Rare ({rare_avg:.1})"
    );
    assert!(
        rare_avg < epic_avg,
        "Rare score ({rare_avg:.1}) < Epic ({epic_avg:.1})"
    );
    assert!(
        epic_avg < legendary_avg,
        "Epic score ({epic_avg:.1}) < Legendary ({legendary_avg:.1})"
    );
}

#[test]
fn test_score_is_deterministic_for_same_item_and_state() {
    let game_state = GameState::new("Deterministic".to_string(), 0);
    let item = generate_item(EquipmentSlot::Weapon, Rarity::Rare, 10);

    let score1 = score_item(&item, &game_state);
    let score2 = score_item(&item, &game_state);

    assert!(
        (score1 - score2).abs() < f64::EPSILON,
        "Scoring the same item twice should produce identical results: {score1} vs {score2}"
    );
}

#[test]
fn test_score_reflects_attribute_specialization() {
    // A STR-focused character should score STR items higher than DEX items
    let mut game_state = GameState::new("STR Build".to_string(), 0);
    game_state.attributes.set(AttributeType::Strength, 30);
    game_state.attributes.set(AttributeType::Dexterity, 10);
    game_state.attributes.set(AttributeType::Constitution, 10);
    game_state.attributes.set(AttributeType::Intelligence, 10);
    game_state.attributes.set(AttributeType::Wisdom, 10);
    game_state.attributes.set(AttributeType::Charisma, 10);

    let str_item = Item {
        slot: EquipmentSlot::Weapon,
        rarity: Rarity::Common,
        ilvl: 10,
        base_name: "STR Sword".to_string(),
        display_name: "STR Sword".to_string(),
        attributes: AttributeBonuses {
            str: 5,
            ..AttributeBonuses::new()
        },
        affixes: vec![],
    };

    let dex_item = Item {
        slot: EquipmentSlot::Weapon,
        rarity: Rarity::Common,
        ilvl: 10,
        base_name: "DEX Dagger".to_string(),
        display_name: "DEX Dagger".to_string(),
        attributes: AttributeBonuses {
            dex: 5,
            ..AttributeBonuses::new()
        },
        affixes: vec![],
    };

    let str_score = score_item(&str_item, &game_state);
    let dex_score = score_item(&dex_item, &game_state);

    assert!(
        str_score > dex_score,
        "STR item ({str_score}) should score higher than DEX item ({dex_score}) for STR-focused build"
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
fn test_auto_equip_higher_scored_item_replaces_lower() {
    let mut game_state = GameState::new("Replace Test".to_string(), 0);

    // Equip a weak common item
    let weak = Item {
        slot: EquipmentSlot::Weapon,
        rarity: Rarity::Common,
        ilvl: 10,
        base_name: "Rusty Sword".to_string(),
        display_name: "Rusty Sword".to_string(),
        attributes: AttributeBonuses {
            str: 1,
            ..AttributeBonuses::new()
        },
        affixes: vec![],
    };
    auto_equip_if_better(weak, &mut game_state);

    let weak_score = score_item(
        game_state
            .equipment
            .get(EquipmentSlot::Weapon)
            .as_ref()
            .unwrap(),
        &game_state,
    );

    // Try a much stronger legendary item
    let strong = Item {
        slot: EquipmentSlot::Weapon,
        rarity: Rarity::Legendary,
        ilvl: 10,
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
    };
    let strong_score = score_item(&strong, &game_state);
    let replaced = auto_equip_if_better(strong, &mut game_state);

    assert!(
        replaced,
        "Stronger item (score {strong_score}) should replace weaker (score {weak_score})"
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
fn test_auto_equip_lower_scored_item_does_not_replace() {
    let mut game_state = GameState::new("No Replace Test".to_string(), 0);

    // Equip a strong legendary item first
    let strong = Item {
        slot: EquipmentSlot::Armor,
        rarity: Rarity::Legendary,
        ilvl: 10,
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
    };
    auto_equip_if_better(strong, &mut game_state);

    // Try a weak common item
    let weak = Item {
        slot: EquipmentSlot::Armor,
        rarity: Rarity::Common,
        ilvl: 10,
        base_name: "Cloth Shirt".to_string(),
        display_name: "Cloth Shirt".to_string(),
        attributes: AttributeBonuses {
            con: 1,
            ..AttributeBonuses::new()
        },
        affixes: vec![],
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
        base_name: "Sword".to_string(),
        display_name: "Fine Sword".to_string(),
        attributes: AttributeBonuses {
            str: 6,
            ..AttributeBonuses::new()
        },
        affixes: vec![],
    };
    let helmet = Item {
        slot: EquipmentSlot::Helmet,
        rarity: Rarity::Magic,
        ilvl: 10,
        base_name: "Helm".to_string(),
        display_name: "Iron Helm".to_string(),
        attributes: AttributeBonuses {
            con: 3,
            ..AttributeBonuses::new()
        },
        affixes: vec![],
    };
    let boots = Item {
        slot: EquipmentSlot::Boots,
        rarity: Rarity::Epic,
        ilvl: 10,
        base_name: "Boots".to_string(),
        display_name: "Swift Boots".to_string(),
        attributes: AttributeBonuses {
            dex: 8,
            ..AttributeBonuses::new()
        },
        affixes: vec![],
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
// Full pipeline: generate → score → equip → upgrade chain
// =========================================================================

#[test]
fn test_full_pipeline_generate_score_equip() {
    let mut game_state = GameState::new("Pipeline Hero".to_string(), 0);

    // Generate a common item and equip it
    let common_item = generate_item(EquipmentSlot::Weapon, Rarity::Common, 5);
    assert!(common_item.attributes.total() > 0);
    let common_score = score_item(&common_item, &game_state);
    assert!(common_score > 0.0);

    let equipped = auto_equip_if_better(common_item, &mut game_state);
    assert!(equipped, "Common item should equip into empty weapon slot");

    // Generate a legendary item and verify it replaces the common
    let legendary_item = generate_item(EquipmentSlot::Weapon, Rarity::Legendary, 20);
    let legendary_score = score_item(&legendary_item, &game_state);

    // Legendary should outscore common (on average, overwhelmingly so)
    // This could theoretically fail with astronomically bad RNG, but the ranges
    // make it impossible: legendary min attrs = 8, common max = 6
    assert!(
        legendary_score > common_score,
        "Legendary ({legendary_score:.1}) should outscore Common ({common_score:.1})"
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
            base_name: "Tier1".to_string(),
            display_name: "Wooden Sword".to_string(),
            attributes: AttributeBonuses {
                str: 1,
                ..AttributeBonuses::new()
            },
            affixes: vec![],
        },
        Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Magic,
            ilvl: 10,
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
        },
        Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Rare,
            ilvl: 10,
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
        },
        Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Legendary,
            ilvl: 10,
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
        },
    ];

    let mut prev_score = 0.0_f64;
    for (i, item) in items.into_iter().enumerate() {
        let item_name = item.display_name.clone();
        let item_score = score_item(&item, &game_state);

        assert!(
            item_score > prev_score,
            "Tier {} ({item_name}, score {item_score:.1}) should outscore previous ({prev_score:.1})",
            i + 1
        );

        let equipped = auto_equip_if_better(item, &mut game_state);
        assert!(
            equipped,
            "Tier {} ({item_name}) should equip as upgrade",
            i + 1
        );

        prev_score = item_score;
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
    let mut rng = rand::thread_rng();
    let mut seen = std::collections::HashSet::new();

    for _ in 0..10_000 {
        let rarity = roll_rarity(0, &mut rng);
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
    let mut rng = rand::thread_rng();
    let trials = 20_000;

    let mut common_p0 = 0usize;
    let mut common_p10 = 0usize;

    for _ in 0..trials {
        if roll_rarity(0, &mut rng) == Rarity::Common {
            common_p0 += 1;
        }
        if roll_rarity(10, &mut rng) == Rarity::Common {
            common_p10 += 1;
        }
    }

    // Prestige 10 gives 10% bonus, reducing common from ~55% to ~45%
    assert!(
        common_p10 < common_p0,
        "P10 should have fewer commons ({common_p10}) than P0 ({common_p0})"
    );

    // Verify the magnitude is meaningful (at least 5% difference = 1000 in 20k)
    let diff = common_p0 as i64 - common_p10 as i64;
    assert!(
        diff > 500,
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
// Score item with affix type weight ordering
// =========================================================================

#[test]
fn test_damage_percent_affix_outscores_hp_bonus_affix_at_same_value() {
    let game_state = GameState::new("Affix Weight Test".to_string(), 0);

    let dmg_item = Item {
        slot: EquipmentSlot::Ring,
        rarity: Rarity::Magic,
        ilvl: 10,
        base_name: "DmgRing".to_string(),
        display_name: "Ring of Damage".to_string(),
        attributes: AttributeBonuses::new(),
        affixes: vec![Affix {
            affix_type: AffixType::DamagePercent,
            value: 10.0,
        }],
    };

    let hp_item = Item {
        slot: EquipmentSlot::Ring,
        rarity: Rarity::Magic,
        ilvl: 10,
        base_name: "HPRing".to_string(),
        display_name: "Ring of Health".to_string(),
        attributes: AttributeBonuses::new(),
        affixes: vec![Affix {
            affix_type: AffixType::HPBonus,
            value: 10.0,
        }],
    };

    let dmg_score = score_item(&dmg_item, &game_state);
    let hp_score = score_item(&hp_item, &game_state);

    // DamagePercent weight = 2.0, HPBonus weight = 0.5
    // So 10*2.0 = 20.0 vs 10*0.5 = 5.0
    assert!(
        dmg_score > hp_score,
        "DamagePercent ({dmg_score}) should outscore HPBonus ({hp_score}) at same value"
    );
    assert!(
        (dmg_score - 20.0).abs() < f64::EPSILON,
        "DamagePercent score should be exactly 20.0, got {dmg_score}"
    );
    assert!(
        (hp_score - 5.0).abs() < f64::EPSILON,
        "HPBonus score should be exactly 5.0, got {hp_score}"
    );
}

// =========================================================================
// Edge case: zero-attribute item with only affixes
// =========================================================================

#[test]
fn test_score_affix_only_item_is_positive() {
    let game_state = GameState::new("Affix Only".to_string(), 0);

    let item = Item {
        slot: EquipmentSlot::Amulet,
        rarity: Rarity::Rare,
        ilvl: 10,
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
    };

    let score = score_item(&item, &game_state);
    // CritChance: 15 * 1.5 = 22.5, CritMultiplier: 20 * 1.5 = 30.0
    let expected = 15.0 * 1.5 + 20.0 * 1.5;
    assert!(
        (score - expected).abs() < f64::EPSILON,
        "Score should be {expected}, got {score}"
    );
    assert!(score > 0.0);
}

// =========================================================================
// Pipeline with prestige: higher prestige -> better items on average
// =========================================================================

#[test]
fn test_pipeline_prestige_produces_better_average_scores() {
    // Higher prestige shifts rarity distribution, so average scored items should be better
    let game_state_p0 = GameState::new("P0 Scorer".to_string(), 0);

    let mut game_state_p10 = GameState::new("P10 Scorer".to_string(), 0);
    game_state_p10.prestige_rank = 10;

    let avg_score = |gs: &GameState| -> f64 {
        let mut rng = rand::thread_rng();
        let n = 500;
        let sum: f64 = (0..n)
            .map(|_| {
                let rarity = roll_rarity(gs.prestige_rank, &mut rng);
                let item = generate_item(EquipmentSlot::Weapon, rarity, 10);
                score_item(&item, gs)
            })
            .sum();
        sum / n as f64
    };

    let avg_p0 = avg_score(&game_state_p0);
    let avg_p10 = avg_score(&game_state_p10);

    assert!(
        avg_p10 > avg_p0,
        "P10 average score ({avg_p10:.1}) should exceed P0 average ({avg_p0:.1})"
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
        if let Some(item) = try_drop_item(&game_state) {
            // Verify basic item validity
            assert!(!item.display_name.is_empty());
            assert!(item.attributes.total() > 0);

            // Score should be positive for any valid item
            let score = score_item(&item, &game_state);
            assert!(
                score > 0.0,
                "Dropped item should have positive score, got {score}"
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
// Auto-equip with equal-score items: tie goes to existing (no replacement)
// =========================================================================

#[test]
fn test_auto_equip_equal_score_does_not_replace() {
    let mut game_state = GameState::new("Tie Test".to_string(), 0);

    let item1 = Item {
        slot: EquipmentSlot::Ring,
        rarity: Rarity::Common,
        ilvl: 10,
        base_name: "Ring1".to_string(),
        display_name: "Ring Alpha".to_string(),
        attributes: AttributeBonuses {
            str: 3,
            ..AttributeBonuses::new()
        },
        affixes: vec![],
    };

    // Create an identical item (same stats, different name)
    let item2 = Item {
        slot: EquipmentSlot::Ring,
        rarity: Rarity::Common,
        ilvl: 10,
        base_name: "Ring2".to_string(),
        display_name: "Ring Beta".to_string(),
        attributes: AttributeBonuses {
            str: 3,
            ..AttributeBonuses::new()
        },
        affixes: vec![],
    };

    // Verify both items have the same score
    let score1 = score_item(&item1, &game_state);
    let score2 = score_item(&item2, &game_state);
    assert!(
        (score1 - score2).abs() < f64::EPSILON,
        "Items should have equal scores: {score1} vs {score2}"
    );

    // Equip first
    assert!(auto_equip_if_better(item1, &mut game_state));

    // Second item with equal score should NOT replace (strictly greater required)
    let replaced = auto_equip_if_better(item2, &mut game_state);
    assert!(
        !replaced,
        "Equal-score item should NOT replace existing (tie goes to incumbent)"
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
