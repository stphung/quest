//! God Items integration tests.
//!
//! End-to-end tests for the god item system: definitions, auto-equip protection,
//! serialization, and equipped bonus queries.

use quest::god_items::{
    asprika_definition, equipped_god_item_attack_speed_percent, equipped_god_item_damage_percent,
    equipped_god_item_dr, equipped_god_item_dungeon_speed_percent,
    equipped_god_item_fishing_reduction_percent, equipped_god_item_regen_reduction_percent,
    megingjord_definition, sleipnir_definition, GodItemId,
};
use quest::items::{
    auto_equip_if_better, Affix, AffixType, AttributeBonuses, Equipment, EquipmentSlot, Item,
    Rarity,
};
use quest::GameState;

// =========================================================================
// Asprika definition and item creation
// =========================================================================

#[test]
fn test_asprika_item_is_mythic_with_correct_fields() {
    let asprika = asprika_definition().to_item();
    assert_eq!(asprika.rarity, Rarity::Mythic);
    assert_eq!(asprika.slot, EquipmentSlot::Armor);
    assert_eq!(asprika.display_name, "Asprika");
    assert_eq!(asprika.god_item_id, Some(GodItemId::Asprika));
    assert_eq!(asprika.ilvl, 100);
    assert!(asprika.attributes.con > 0, "Asprika should have CON");
    assert!(asprika.attributes.wis > 0, "Asprika should have WIS");
}

// =========================================================================
// Auto-equip protection: Mythic items cannot be replaced by non-Mythic
// =========================================================================

#[test]
fn test_asprika_item_is_always_best_in_slot() {
    // Asprika should never be replaced by auto-equip, even by a strong Legendary.
    let asprika = asprika_definition().to_item();
    let legendary = Item {
        slot: EquipmentSlot::Armor,
        rarity: Rarity::Legendary,
        ilvl: 100,
        tier: 5,
        base_name: "Test".to_string(),
        display_name: "Test".to_string(),
        attributes: AttributeBonuses {
            con: 20,
            dex: 10,
            ..AttributeBonuses::new()
        },
        affixes: vec![
            Affix {
                affix_type: AffixType::DamageReduction,
                value: 15.0,
            },
            Affix {
                affix_type: AffixType::HPBonus,
                value: 100.0,
            },
        ],
        god_item_id: None,
    };

    let mut state = GameState::new("Test".to_string(), 0);
    state.equipment.set(EquipmentSlot::Armor, Some(asprika));

    let equipped = auto_equip_if_better(legendary, &mut state);
    assert!(
        !equipped,
        "Legendary should not replace Mythic Asprika via auto-equip"
    );
    assert_eq!(
        state
            .equipment
            .get(EquipmentSlot::Armor)
            .as_ref()
            .unwrap()
            .rarity,
        Rarity::Mythic,
        "Asprika should still be in the Armor slot"
    );
}

// =========================================================================
// Serialization roundtrips
// =========================================================================

#[test]
fn test_mythic_item_serialization_roundtrip() {
    let asprika = asprika_definition().to_item();
    let json = serde_json::to_string(&asprika).unwrap();
    let loaded: Item = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.rarity, Rarity::Mythic);
    assert_eq!(loaded.god_item_id, Some(GodItemId::Asprika));
    assert_eq!(loaded.display_name, "Asprika");
}

// =========================================================================
// Equipped god item bonus queries
// =========================================================================

#[test]
fn test_equipped_god_item_dr_integration() {
    let mut equipment = Equipment::new();

    // No god item — 0% DR
    assert!((equipped_god_item_dr(&equipment) - 0.0).abs() < f64::EPSILON);

    // Equip Asprika — 30% DR
    let asprika = asprika_definition().to_item();
    equipment.set(EquipmentSlot::Armor, Some(asprika));
    assert!((equipped_god_item_dr(&equipment) - 30.0).abs() < f64::EPSILON);
}

#[test]
fn test_equipped_god_item_dr_zero_with_non_god_items() {
    let mut equipment = Equipment::new();
    let normal_item = Item {
        slot: EquipmentSlot::Armor,
        rarity: Rarity::Legendary,
        ilvl: 100,
        tier: 5,
        base_name: "Plate".to_string(),
        display_name: "Plate".to_string(),
        attributes: AttributeBonuses {
            con: 15,
            ..AttributeBonuses::new()
        },
        affixes: vec![Affix {
            affix_type: AffixType::DamageReduction,
            value: 10.0,
        }],
        god_item_id: None,
    };
    equipment.set(EquipmentSlot::Armor, Some(normal_item));

    assert!(
        (equipped_god_item_dr(&equipment) - 0.0).abs() < f64::EPSILON,
        "Non-god items should contribute 0% god item DR"
    );
}

// =========================================================================
// Sleipnir and Megingjord definitions
// =========================================================================

#[test]
fn test_sleipnir_item_is_mythic_with_correct_fields() {
    let sleipnir = sleipnir_definition().to_item();
    assert_eq!(sleipnir.rarity, Rarity::Mythic);
    assert_eq!(sleipnir.slot, EquipmentSlot::Boots);
    assert_eq!(sleipnir.display_name, "Sleipnir");
    assert_eq!(sleipnir.god_item_id, Some(GodItemId::Sleipnir));
    assert_eq!(sleipnir.ilvl, 100);
    assert!(sleipnir.attributes.dex > 0, "Sleipnir should have DEX");
}

#[test]
fn test_megingjord_item_is_mythic_with_correct_fields() {
    let megingjord = megingjord_definition().to_item();
    assert_eq!(megingjord.rarity, Rarity::Mythic);
    assert_eq!(megingjord.slot, EquipmentSlot::Ring);
    assert_eq!(megingjord.display_name, "Megingjord");
    assert_eq!(megingjord.god_item_id, Some(GodItemId::Megingjord));
    assert_eq!(megingjord.ilvl, 100);
    assert!(megingjord.attributes.str > 0, "Megingjord should have STR");
}

// =========================================================================
// Cross-god-item passive queries (wrong item equipped returns 0.0)
// =========================================================================

/// Equip a god item defined by the given slot, returning the populated Equipment.
fn equipment_with_god_item(slot: EquipmentSlot, item: Item) -> Equipment {
    let mut eq = Equipment::new();
    eq.set(slot, Some(item));
    eq
}

// --- equipped_god_item_dr: wrong items return 0.0 ---

#[test]
fn god_item_dr_with_megingjord_equipped_is_zero() {
    let item = megingjord_definition().to_item();
    let eq = equipment_with_god_item(EquipmentSlot::Ring, item);
    assert!(
        equipped_god_item_dr(&eq).abs() < f64::EPSILON,
        "Megingjord should not provide DR"
    );
}

#[test]
fn god_item_dr_with_sleipnir_equipped_is_zero() {
    let item = sleipnir_definition().to_item();
    let eq = equipment_with_god_item(EquipmentSlot::Boots, item);
    assert!(
        equipped_god_item_dr(&eq).abs() < f64::EPSILON,
        "Sleipnir should not provide DR"
    );
}

// --- equipped_god_item_damage_percent: only Megingjord provides this ---

#[test]
fn god_item_damage_percent_with_asprika_equipped_is_zero() {
    let item = asprika_definition().to_item();
    let eq = equipment_with_god_item(EquipmentSlot::Armor, item);
    assert!(
        equipped_god_item_damage_percent(&eq).abs() < f64::EPSILON,
        "Asprika should not provide damage %"
    );
}

#[test]
fn god_item_damage_percent_with_sleipnir_equipped_is_zero() {
    let item = sleipnir_definition().to_item();
    let eq = equipment_with_god_item(EquipmentSlot::Boots, item);
    assert!(
        equipped_god_item_damage_percent(&eq).abs() < f64::EPSILON,
        "Sleipnir should not provide damage %"
    );
}

// --- equipped_god_item_attack_speed_percent: only Sleipnir provides this ---

#[test]
fn god_item_attack_speed_with_asprika_equipped_is_zero() {
    let item = asprika_definition().to_item();
    let eq = equipment_with_god_item(EquipmentSlot::Armor, item);
    assert!(
        equipped_god_item_attack_speed_percent(&eq).abs() < f64::EPSILON,
        "Asprika should not provide attack speed"
    );
}

#[test]
fn god_item_attack_speed_with_megingjord_equipped_is_zero() {
    let item = megingjord_definition().to_item();
    let eq = equipment_with_god_item(EquipmentSlot::Ring, item);
    assert!(
        equipped_god_item_attack_speed_percent(&eq).abs() < f64::EPSILON,
        "Megingjord should not provide attack speed"
    );
}

// --- equipped_god_item_regen_reduction_percent: only Sleipnir (Swiftstrider) ---

#[test]
fn god_item_regen_reduction_with_asprika_equipped_is_zero() {
    let item = asprika_definition().to_item();
    let eq = equipment_with_god_item(EquipmentSlot::Armor, item);
    assert!(
        equipped_god_item_regen_reduction_percent(&eq).abs() < f64::EPSILON,
        "Asprika has no bonuses, regen reduction must be 0"
    );
}

#[test]
fn god_item_regen_reduction_with_megingjord_equipped_is_zero() {
    let item = megingjord_definition().to_item();
    let eq = equipment_with_god_item(EquipmentSlot::Ring, item);
    assert!(
        equipped_god_item_regen_reduction_percent(&eq).abs() < f64::EPSILON,
        "Megingjord has no bonuses, regen reduction must be 0"
    );
}

// --- equipped_god_item_dungeon_speed_percent: only Sleipnir (Swiftfoot) ---

#[test]
fn god_item_dungeon_speed_with_asprika_equipped_is_zero() {
    let item = asprika_definition().to_item();
    let eq = equipment_with_god_item(EquipmentSlot::Armor, item);
    assert!(
        equipped_god_item_dungeon_speed_percent(&eq).abs() < f64::EPSILON,
        "Asprika should not provide dungeon speed"
    );
}

#[test]
fn god_item_dungeon_speed_with_megingjord_equipped_is_zero() {
    let item = megingjord_definition().to_item();
    let eq = equipment_with_god_item(EquipmentSlot::Ring, item);
    assert!(
        equipped_god_item_dungeon_speed_percent(&eq).abs() < f64::EPSILON,
        "Megingjord should not provide dungeon speed"
    );
}

// --- equipped_god_item_fishing_reduction_percent: only Sleipnir (NimbleHands) ---

#[test]
fn god_item_fishing_reduction_with_asprika_equipped_is_zero() {
    let item = asprika_definition().to_item();
    let eq = equipment_with_god_item(EquipmentSlot::Armor, item);
    assert!(
        equipped_god_item_fishing_reduction_percent(&eq).abs() < f64::EPSILON,
        "Asprika should not provide fishing reduction"
    );
}

#[test]
fn god_item_fishing_reduction_with_megingjord_equipped_is_zero() {
    let item = megingjord_definition().to_item();
    let eq = equipment_with_god_item(EquipmentSlot::Ring, item);
    assert!(
        equipped_god_item_fishing_reduction_percent(&eq).abs() < f64::EPSILON,
        "Megingjord should not provide fishing reduction"
    );
}

// --- Multiple god items equipped simultaneously: each query resolves to its owner ---

#[test]
fn god_item_multiple_equipped_each_query_resolves_correctly() {
    // Equip Asprika (Armor) and Sleipnir (Boots) and Megingjord (Ring)
    let mut eq = Equipment::new();
    eq.set(EquipmentSlot::Armor, Some(asprika_definition().to_item()));
    eq.set(EquipmentSlot::Boots, Some(sleipnir_definition().to_item()));
    eq.set(EquipmentSlot::Ring, Some(megingjord_definition().to_item()));

    // DR comes from Asprika
    assert!(
        (equipped_god_item_dr(&eq) - 30.0).abs() < f64::EPSILON,
        "DR should be 30 from Asprika"
    );

    // Attack speed comes from Sleipnir
    assert!(
        (equipped_god_item_attack_speed_percent(&eq) - 100.0).abs() < f64::EPSILON,
        "Attack speed should be 100 from Sleipnir"
    );

    // Damage % comes from Megingjord
    assert!(
        (equipped_god_item_damage_percent(&eq) - 150.0).abs() < f64::EPSILON,
        "Damage % should be 150 from Megingjord"
    );

    // Regen reduction, dungeon speed, fishing reduction all come from Sleipnir
    assert!(
        (equipped_god_item_regen_reduction_percent(&eq) - 50.0).abs() < f64::EPSILON,
        "Regen reduction should be 50 from Sleipnir"
    );
    assert!(
        (equipped_god_item_dungeon_speed_percent(&eq) - 50.0).abs() < f64::EPSILON,
        "Dungeon speed should be 50 from Sleipnir"
    );
    assert!(
        (equipped_god_item_fishing_reduction_percent(&eq) - 50.0).abs() < f64::EPSILON,
        "Fishing reduction should be 50 from Sleipnir"
    );
}

#[test]
fn god_item_asprika_only_does_not_provide_any_speed_or_fishing_bonus() {
    let eq = equipment_with_god_item(EquipmentSlot::Armor, asprika_definition().to_item());

    assert!(equipped_god_item_attack_speed_percent(&eq).abs() < f64::EPSILON);
    assert!(equipped_god_item_damage_percent(&eq).abs() < f64::EPSILON);
    assert!(equipped_god_item_regen_reduction_percent(&eq).abs() < f64::EPSILON);
    assert!(equipped_god_item_dungeon_speed_percent(&eq).abs() < f64::EPSILON);
    assert!(equipped_god_item_fishing_reduction_percent(&eq).abs() < f64::EPSILON);
}

#[test]
fn god_item_megingjord_only_does_not_provide_any_passive_except_damage() {
    let eq = equipment_with_god_item(EquipmentSlot::Ring, megingjord_definition().to_item());

    assert!(equipped_god_item_dr(&eq).abs() < f64::EPSILON);
    assert!(equipped_god_item_attack_speed_percent(&eq).abs() < f64::EPSILON);
    assert!(equipped_god_item_regen_reduction_percent(&eq).abs() < f64::EPSILON);
    assert!(equipped_god_item_dungeon_speed_percent(&eq).abs() < f64::EPSILON);
    assert!(equipped_god_item_fishing_reduction_percent(&eq).abs() < f64::EPSILON);
    // Damage % should be 150
    assert!(
        (equipped_god_item_damage_percent(&eq) - 150.0).abs() < f64::EPSILON,
        "Megingjord damage % should be 150"
    );
}
