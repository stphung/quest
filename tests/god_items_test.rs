//! God Items integration tests.
//!
//! End-to-end tests for the god item system: definitions, auto-equip protection,
//! serialization, and equipped bonus queries.

use quest::god_items::{
    asprika_definition, equipped_god_item_dr, megingjord_definition, sleipnir_definition, GodItemId,
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
    assert_eq!(equipped_god_item_dr(&equipment), 0.0);

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

    assert_eq!(
        equipped_god_item_dr(&equipment),
        0.0,
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
