use super::types::{AffixType, Item, Rarity};
use crate::core::game_state::GameState;

/// Returns the power weight for a given affix type.
/// Used by intrinsic item power calculation (`Item::power()`).
pub fn affix_power_weight(affix_type: AffixType) -> f64 {
    match affix_type {
        AffixType::DamagePercent => 2.0,
        AffixType::CritChance => 1.5,
        AffixType::CritMultiplier => 1.5,
        AffixType::AttackSpeed => 1.2,
        AffixType::HPBonus => 0.5, // Flat HP less valuable
        AffixType::DamageReduction => 1.3,
        AffixType::HPRegen => 1.0,
        AffixType::DamageReflection => 0.8,
        AffixType::XPGain => 1.0,
        AffixType::Unknown => 0.0,
    }
}

pub fn auto_equip_if_better(item: Item, game_state: &mut GameState) -> bool {
    // Never auto-replace a Mythic (god) item
    if let Some(current) = game_state.equipment.get(item.slot).as_ref() {
        if current.rarity == Rarity::Mythic && item.rarity != Rarity::Mythic {
            return false;
        }
    }

    let new_power = item.power();
    let current_power = game_state
        .equipment
        .get(item.slot)
        .as_ref()
        .map(|current| current.power())
        .unwrap_or(0);

    if new_power > current_power {
        game_state.equipment.set(item.slot, Some(item));
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::{Affix, AttributeBonuses, EquipmentSlot, Rarity};
    use super::*;
    use chrono::Utc;

    fn create_test_item(slot: EquipmentSlot, rarity: Rarity, str_bonus: u32) -> Item {
        Item {
            slot,
            rarity,
            ilvl: 10,
            tier: 5,
            base_name: "Test".to_string(),
            display_name: "Test Item".to_string(),
            attributes: AttributeBonuses {
                str: str_bonus,
                ..AttributeBonuses::new()
            },
            affixes: vec![],
            god_item_id: None,
        }
    }

    #[test]
    fn test_auto_equip_empty_slot() {
        let mut game_state = GameState::new("Test Hero".to_string(), Utc::now().timestamp());
        let item = create_test_item(EquipmentSlot::Weapon, Rarity::Common, 2);

        let equipped = auto_equip_if_better(item, &mut game_state);
        assert!(equipped);
        assert!(game_state.equipment.get(EquipmentSlot::Weapon).is_some());
    }

    #[test]
    fn test_auto_equip_higher_power_replaces() {
        let mut game_state = GameState::new("Test Hero".to_string(), Utc::now().timestamp());

        // Equip weak item (power = 1)
        let weak = create_test_item(EquipmentSlot::Weapon, Rarity::Common, 1);
        auto_equip_if_better(weak, &mut game_state);

        // Try stronger item (power = 10)
        let strong = create_test_item(EquipmentSlot::Weapon, Rarity::Rare, 10);
        let equipped = auto_equip_if_better(strong, &mut game_state);

        assert!(equipped);
        assert_eq!(
            game_state
                .equipment
                .get(EquipmentSlot::Weapon)
                .as_ref()
                .unwrap()
                .attributes
                .str,
            10
        );
    }

    #[test]
    fn test_auto_equip_rejects_lower_power() {
        let mut game_state = GameState::new("Test Hero".to_string(), Utc::now().timestamp());

        // Equip strong item (power = 10)
        let strong = create_test_item(EquipmentSlot::Weapon, Rarity::Rare, 10);
        auto_equip_if_better(strong, &mut game_state);

        // Try weaker item (power = 1)
        let weak = create_test_item(EquipmentSlot::Weapon, Rarity::Common, 1);
        let equipped = auto_equip_if_better(weak, &mut game_state);

        assert!(!equipped);
        assert_eq!(
            game_state
                .equipment
                .get(EquipmentSlot::Weapon)
                .as_ref()
                .unwrap()
                .attributes
                .str,
            10
        );
    }

    #[test]
    fn test_auto_equip_uses_power_not_build() {
        // A high-STR character should still equip a higher-power DEX item
        let mut game_state = GameState::new("Test Hero".to_string(), Utc::now().timestamp());
        use crate::character::attributes::AttributeType;
        game_state.attributes.set(AttributeType::Strength, 30);
        game_state.attributes.set(AttributeType::Dexterity, 10);

        // Equip a STR item (power = 5)
        let str_item = Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Common,
            ilvl: 10,
            tier: 5,
            base_name: "Test".to_string(),
            display_name: "Test".to_string(),
            attributes: AttributeBonuses {
                str: 5,
                ..AttributeBonuses::new()
            },
            affixes: vec![],
            god_item_id: None,
        };
        auto_equip_if_better(str_item, &mut game_state);

        // DEX item with higher power (power = 8) should replace it
        let dex_item = Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Rare,
            ilvl: 10,
            tier: 5,
            base_name: "Test".to_string(),
            display_name: "Test".to_string(),
            attributes: AttributeBonuses {
                dex: 8,
                ..AttributeBonuses::new()
            },
            affixes: vec![],
            god_item_id: None,
        };
        let equipped = auto_equip_if_better(dex_item, &mut game_state);
        assert!(
            equipped,
            "Higher-power DEX item should replace lower-power STR item regardless of character build"
        );
    }

    #[test]
    fn test_auto_equip_different_slots_independent() {
        let mut game_state = GameState::new("Test Hero".to_string(), Utc::now().timestamp());

        let weapon = create_test_item(EquipmentSlot::Weapon, Rarity::Common, 3);
        let armor = Item {
            slot: EquipmentSlot::Armor,
            rarity: Rarity::Common,
            ilvl: 10,
            tier: 5,
            base_name: "Test".to_string(),
            display_name: "Test".to_string(),
            attributes: AttributeBonuses {
                con: 4,
                ..AttributeBonuses::new()
            },
            affixes: vec![],
            god_item_id: None,
        };

        assert!(auto_equip_if_better(weapon, &mut game_state));
        assert!(auto_equip_if_better(armor, &mut game_state));

        assert!(game_state.equipment.get(EquipmentSlot::Weapon).is_some());
        assert!(game_state.equipment.get(EquipmentSlot::Armor).is_some());
    }

    #[test]
    fn test_mythic_item_never_auto_replaced() {
        let mut game_state = GameState::new("Test Hero".to_string(), Utc::now().timestamp());

        // Equip a Mythic item (weak stats)
        let mythic = Item {
            slot: EquipmentSlot::Armor,
            rarity: Rarity::Mythic,
            ilvl: 100,
            tier: 5,
            base_name: "Asprika".to_string(),
            display_name: "Asprika".to_string(),
            attributes: AttributeBonuses {
                con: 1,
                ..AttributeBonuses::new()
            },
            affixes: vec![],
            god_item_id: None,
        };
        game_state.equipment.set(EquipmentSlot::Armor, Some(mythic));

        // Try to equip a Legendary with higher power
        let legendary = Item {
            slot: EquipmentSlot::Armor,
            rarity: Rarity::Legendary,
            ilvl: 100,
            tier: 5,
            base_name: "Test".to_string(),
            display_name: "Test".to_string(),
            attributes: AttributeBonuses {
                con: 50,
                str: 50,
                ..AttributeBonuses::new()
            },
            affixes: vec![Affix {
                affix_type: AffixType::DamagePercent,
                value: 100.0,
            }],
            god_item_id: None,
        };
        let equipped = auto_equip_if_better(legendary, &mut game_state);
        assert!(
            !equipped,
            "Mythic item should never be auto-replaced by a Legendary"
        );
        assert_eq!(
            game_state
                .equipment
                .get(EquipmentSlot::Armor)
                .as_ref()
                .unwrap()
                .rarity,
            Rarity::Mythic,
        );
    }

    #[test]
    fn test_auto_equip_affix_item_beats_attribute_item() {
        let mut game_state = GameState::new("Test Hero".to_string(), Utc::now().timestamp());

        // Equip a small attribute-only item (power = 1)
        let weak = create_test_item(EquipmentSlot::Weapon, Rarity::Common, 1);
        auto_equip_if_better(weak, &mut game_state);

        // An item with good affixes should replace it (power = 1 + 20*2.0 = 41)
        let affix_item = Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Magic,
            ilvl: 10,
            tier: 5,
            base_name: "Test".to_string(),
            display_name: "Test".to_string(),
            attributes: AttributeBonuses {
                str: 1,
                ..AttributeBonuses::new()
            },
            affixes: vec![Affix {
                affix_type: AffixType::DamagePercent,
                value: 20.0,
            }],
            god_item_id: None,
        };

        let equipped = auto_equip_if_better(affix_item, &mut game_state);
        assert!(
            equipped,
            "Item with strong affix should replace weak attribute-only item"
        );
    }
}
