//! Loot drop simulation using real game mechanics.

use crate::core::game_state::GameState;
use crate::items::drops::{ilvl_for_zone, roll_rarity_for_boss, roll_rarity_for_mob};
use crate::items::generation::generate_item;
use crate::items::scoring::score_item;
use crate::items::types::{EquipmentSlot, Item, Rarity};
use rand::Rng;

/// Statistics about loot drops.
#[derive(Debug, Clone, Default)]
pub struct LootStats {
    pub total_drops: u32,
    pub upgrades_equipped: u32,
    pub common_drops: u32,
    pub magic_drops: u32,
    pub rare_drops: u32,
    pub epic_drops: u32,
    pub legendary_drops: u32,
    pub total_drop_attempts: u32,
}

impl LootStats {
    pub fn record_drop(&mut self, item: &Item, was_upgrade: bool) {
        self.total_drops += 1;
        if was_upgrade {
            self.upgrades_equipped += 1;
        }

        match item.rarity {
            Rarity::Common => self.common_drops += 1,
            Rarity::Magic => self.magic_drops += 1,
            Rarity::Rare => self.rare_drops += 1,
            Rarity::Epic => self.epic_drops += 1,
            Rarity::Legendary => self.legendary_drops += 1,
        }
    }

    pub fn record_attempt(&mut self) {
        self.total_drop_attempts += 1;
    }

    pub fn drop_rate(&self) -> f64 {
        if self.total_drop_attempts == 0 {
            0.0
        } else {
            self.total_drops as f64 / self.total_drop_attempts as f64
        }
    }
}

/// Roll for item drop from a mob using real game logic.
/// Returns None if no drop, Some(item) if dropped.
pub fn roll_mob_drop_real(
    zone_id: usize,
    prestige_rank: u32,
    haven_drop_rate: f64,
    haven_rarity: f64,
    rng: &mut impl Rng,
) -> Option<Item> {
    // Use real drop chance calculation
    let base_chance = crate::items::drops::drop_chance_for_prestige(prestige_rank);
    let drop_chance = (base_chance * (1.0 + haven_drop_rate / 100.0)).min(0.25);

    if rng.gen::<f64>() > drop_chance {
        return None;
    }

    // Roll rarity using real mob rarity table (caps at Epic)
    let rarity = roll_rarity_for_mob(prestige_rank, haven_rarity, rng);

    // Generate item using real generation
    let ilvl = ilvl_for_zone(zone_id);
    let slot = roll_random_slot(rng);

    Some(generate_item(slot, rarity, ilvl))
}

/// Roll for item drop from a boss using real game logic.
/// Bosses always drop an item and can drop legendaries.
pub fn roll_boss_drop_real(zone_id: usize, is_final_zone: bool, rng: &mut impl Rng) -> Item {
    let rarity = roll_rarity_for_boss(is_final_zone, rng);
    let ilvl = ilvl_for_zone(zone_id);
    let slot = roll_random_slot(rng);

    generate_item(slot, rarity, ilvl)
}

/// Roll a random equipment slot.
fn roll_random_slot(rng: &mut impl Rng) -> EquipmentSlot {
    match rng.gen_range(0..7) {
        0 => EquipmentSlot::Weapon,
        1 => EquipmentSlot::Armor,
        2 => EquipmentSlot::Helmet,
        3 => EquipmentSlot::Gloves,
        4 => EquipmentSlot::Boots,
        5 => EquipmentSlot::Amulet,
        6 => EquipmentSlot::Ring,
        _ => unreachable!(),
    }
}

/// Check if an item is an upgrade and should be equipped.
/// Uses real scoring system.
pub fn is_upgrade(new_item: &Item, current: Option<&Item>, game_state: &GameState) -> bool {
    let new_score = score_item(new_item, game_state);

    match current {
        None => true, // Empty slot, always equip
        Some(old) => {
            let old_score = score_item(old, game_state);
            new_score > old_score
        }
    }
}

/// Calculate average item level of equipped items.
pub fn average_equipped_ilvl(equipment: &crate::items::Equipment) -> f64 {
    let items: Vec<_> = equipment.iter_equipped().collect();

    if items.is_empty() {
        0.0
    } else {
        items.iter().map(|i| i.ilvl as f64).sum::<f64>() / items.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mob_drop_can_drop() {
        let mut rng = rand::thread_rng();
        let mut dropped = false;

        for _ in 0..100 {
            if roll_mob_drop_real(5, 0, 0.0, 0.0, &mut rng).is_some() {
                dropped = true;
                break;
            }
        }

        assert!(dropped, "Should drop something in 100 attempts");
    }

    #[test]
    fn test_mob_drop_no_legendary() {
        let mut rng = rand::thread_rng();

        for _ in 0..1000 {
            if let Some(item) = roll_mob_drop_real(10, 10, 25.0, 25.0, &mut rng) {
                assert_ne!(
                    item.rarity,
                    Rarity::Legendary,
                    "Mobs should never drop legendary"
                );
            }
        }
    }

    #[test]
    fn test_boss_drop_always_drops() {
        let mut rng = rand::thread_rng();

        // Boss always drops
        let item = roll_boss_drop_real(5, false, &mut rng);
        assert!(item.ilvl > 0);
    }

    #[test]
    fn test_boss_can_drop_legendary() {
        let mut rng = rand::thread_rng();
        let mut found_legendary = false;

        for _ in 0..500 {
            let item = roll_boss_drop_real(10, true, &mut rng);
            if item.rarity == Rarity::Legendary {
                found_legendary = true;
                break;
            }
        }

        assert!(found_legendary, "Boss should be able to drop legendary");
    }
}
