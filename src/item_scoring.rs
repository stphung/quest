#![allow(dead_code)]
use crate::game_state::GameState;
use crate::items::{AffixType, AttributeBonuses, Item};

pub fn score_item(item: &Item, game_state: &GameState) -> f64 {
    let mut score = 0.0;

    // Calculate attribute weights based on current character build
    let weights = calculate_attribute_weights(game_state);

    // Score attributes
    score += item.attributes.str as f64 * weights.str as f64;
    score += item.attributes.dex as f64 * weights.dex as f64;
    score += item.attributes.con as f64 * weights.con as f64;
    score += item.attributes.int as f64 * weights.int as f64;
    score += item.attributes.wis as f64 * weights.wis as f64;
    score += item.attributes.cha as f64 * weights.cha as f64;

    // Score affixes with different weights
    for affix in &item.affixes {
        let affix_score = match affix.affix_type {
            AffixType::DamagePercent => affix.value * 2.0,
            AffixType::CritChance => affix.value * 1.5,
            AffixType::CritMultiplier => affix.value * 1.5,
            AffixType::AttackSpeed => affix.value * 1.2,
            AffixType::HPBonus => affix.value * 0.5, // Flat HP less valuable
            AffixType::DamageReduction => affix.value * 1.3,
            AffixType::HPRegen => affix.value * 1.0,
            AffixType::DamageReflection => affix.value * 0.8,
            AffixType::XPGain => affix.value * 1.0,
            AffixType::DropRate => affix.value * 0.5,
            AffixType::PrestigeBonus => affix.value * 0.8,
            AffixType::OfflineRate => affix.value * 0.5,
        };
        score += affix_score;
    }

    score
}

fn calculate_attribute_weights(game_state: &GameState) -> AttributeBonuses {
    // Weight attributes based on current values (specialization bonus)
    // Higher existing attributes get higher weights
    use crate::attributes::AttributeType;

    let attrs = &game_state.attributes;
    let attr_values: Vec<u32> = AttributeType::all()
        .iter()
        .map(|&attr| attrs.get(attr))
        .collect();
    let total = attr_values.iter().sum::<u32>().max(1);

    AttributeBonuses {
        str: 1 + (attr_values[0] * 100 / total),
        dex: 1 + (attr_values[1] * 100 / total),
        con: 1 + (attr_values[2] * 100 / total),
        int: 1 + (attr_values[3] * 100 / total),
        wis: 1 + (attr_values[4] * 100 / total),
        cha: 1 + (attr_values[5] * 100 / total),
    }
}

pub fn auto_equip_if_better(item: Item, game_state: &mut GameState) -> bool {
    let new_score = score_item(&item, game_state);

    let should_equip = if let Some(current) = game_state.equipment.get(item.slot) {
        let current_score = score_item(current, game_state);
        new_score > current_score
    } else {
        // Empty slot, always equip
        true
    };

    if should_equip {
        game_state.equipment.set(item.slot, Some(item));
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::{Affix, EquipmentSlot, Rarity};
    use chrono::Utc;

    fn create_test_item(slot: EquipmentSlot, rarity: Rarity, str_bonus: u32) -> Item {
        Item {
            slot,
            rarity,
            base_name: "Test".to_string(),
            display_name: "Test Item".to_string(),
            attributes: AttributeBonuses {
                str: str_bonus,
                ..AttributeBonuses::new()
            },
            affixes: vec![],
        }
    }

    #[test]
    fn test_score_item_with_attributes() {
        let game_state = GameState::new("Test Hero".to_string(), Utc::now().timestamp());
        let item = create_test_item(EquipmentSlot::Weapon, Rarity::Common, 5);

        let score = score_item(&item, &game_state);
        assert!(score > 0.0);
    }

    #[test]
    fn test_score_item_with_affixes() {
        let game_state = GameState::new("Test Hero".to_string(), Utc::now().timestamp());
        let item = Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Magic,
            base_name: "Test".to_string(),
            display_name: "Test Item".to_string(),
            attributes: AttributeBonuses::new(),
            affixes: vec![Affix {
                affix_type: AffixType::DamagePercent,
                value: 15.0,
            }],
        };

        let score = score_item(&item, &game_state);
        // DamagePercent has 2.0 weight, so 15 * 2 = 30
        assert!(score >= 30.0);
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
    fn test_auto_equip_better_item() {
        let mut game_state = GameState::new("Test Hero".to_string(), Utc::now().timestamp());

        // Equip weak item
        let weak = create_test_item(EquipmentSlot::Weapon, Rarity::Common, 1);
        auto_equip_if_better(weak, &mut game_state);

        // Try stronger item
        let strong = create_test_item(EquipmentSlot::Weapon, Rarity::Rare, 10);
        let equipped = auto_equip_if_better(strong.clone(), &mut game_state);

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
    fn test_auto_equip_rejects_worse_item() {
        let mut game_state = GameState::new("Test Hero".to_string(), Utc::now().timestamp());

        // Equip strong item
        let strong = create_test_item(EquipmentSlot::Weapon, Rarity::Rare, 10);
        auto_equip_if_better(strong, &mut game_state);

        // Try weaker item
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
}
