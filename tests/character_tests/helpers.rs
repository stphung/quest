//! Shared helpers for character/prestige/dungeon coverage tests.

use quest::core::game_state::GameState;
use quest::items::types::{Affix, AffixType, EquipmentSlot, Item, Rarity};
use quest::items::AttributeBonuses;

/// Creates a bare test item with no attributes or affixes.
pub fn base_item(slot: EquipmentSlot) -> Item {
    Item {
        slot,
        rarity: Rarity::Common,
        ilvl: 10,
        tier: 5,
        base_name: "Test".to_string(),
        display_name: "Test Item".to_string(),
        attributes: AttributeBonuses::new(),
        affixes: vec![],
        god_item_id: None,
    }
}

/// Creates an item with a single affix.
pub fn item_with_affix(slot: EquipmentSlot, affix_type: AffixType, value: f64) -> Item {
    Item {
        slot,
        affixes: vec![Affix { affix_type, value }],
        ..base_item(slot)
    }
}

/// Creates a `GameState` at the given character level and prestige rank.
pub fn state_at(level: u32, prestige_rank: u32) -> GameState {
    let mut s = GameState::new("Test Hero".to_string(), 0);
    s.character_level = level;
    s.prestige_rank = prestige_rank;
    s
}
