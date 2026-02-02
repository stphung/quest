#![allow(dead_code)]
use crate::game_state::GameState;
use crate::item_generation::generate_item;
use crate::items::{EquipmentSlot, Item, Rarity};
use rand::Rng;

pub fn try_drop_item(game_state: &GameState) -> Option<Item> {
    let mut rng = rand::thread_rng();

    // Calculate drop chance: 30% base + 5% per prestige rank
    let drop_chance = 0.30 + (game_state.prestige_rank as f64 * 0.05);

    if rng.gen::<f64>() > drop_chance {
        return None;
    }

    // Roll rarity based on prestige rank
    let rarity = roll_rarity(game_state.prestige_rank, &mut rng);

    // Roll random equipment slot
    let slot = roll_random_slot(&mut rng);

    // Generate item
    Some(generate_item(slot, rarity, game_state.character_level))
}

fn roll_rarity(prestige_rank: u32, rng: &mut impl Rng) -> Rarity {
    let roll = rng.gen::<f64>();

    match prestige_rank {
        0..=1 => {
            // Bronze: 60% Common, 30% Magic, 10% Rare
            if roll < 0.60 {
                Rarity::Common
            } else if roll < 0.90 {
                Rarity::Magic
            } else {
                Rarity::Rare
            }
        }
        2..=3 => {
            // Silver: 30% Common, 40% Magic, 25% Rare, 5% Epic
            if roll < 0.30 {
                Rarity::Common
            } else if roll < 0.70 {
                Rarity::Magic
            } else if roll < 0.95 {
                Rarity::Rare
            } else {
                Rarity::Epic
            }
        }
        4..=5 => {
            // Gold: 15% Common, 30% Magic, 40% Rare, 13% Epic, 2% Legendary
            if roll < 0.15 {
                Rarity::Common
            } else if roll < 0.45 {
                Rarity::Magic
            } else if roll < 0.85 {
                Rarity::Rare
            } else if roll < 0.98 {
                Rarity::Epic
            } else {
                Rarity::Legendary
            }
        }
        _ => {
            // Platinum+: 10% Common, 20% Magic, 35% Rare, 25% Epic, 10% Legendary
            if roll < 0.10 {
                Rarity::Common
            } else if roll < 0.30 {
                Rarity::Magic
            } else if roll < 0.65 {
                Rarity::Rare
            } else if roll < 0.90 {
                Rarity::Epic
            } else {
                Rarity::Legendary
            }
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_roll_rarity_bronze_prestige() {
        // Test Bronze prestige (0-1) distribution
        let mut common = 0;
        let mut magic = 0;
        let mut rare = 0;

        for _ in 0..1000 {
            let mut rng = rand::thread_rng();
            match roll_rarity(0, &mut rng) {
                Rarity::Common => common += 1,
                Rarity::Magic => magic += 1,
                Rarity::Rare => rare += 1,
                _ => {}
            }
        }

        // Rough distribution check (should be ~60%, 30%, 10%)
        assert!(common > 500); // At least 50%
        assert!(magic > 200); // At least 20%
        assert!(rare > 0); // Some rares
    }

    #[test]
    fn test_roll_rarity_platinum_can_drop_legendary() {
        // Test Platinum+ can drop legendary
        let mut found_legendary = false;
        for _ in 0..1000 {
            let mut rng = rand::thread_rng();
            if roll_rarity(6, &mut rng) == Rarity::Legendary {
                found_legendary = true;
                break;
            }
        }
        assert!(found_legendary);
    }

    #[test]
    fn test_roll_random_slot_coverage() {
        let mut slots_seen = std::collections::HashSet::new();
        let mut rng = rand::thread_rng();

        for _ in 0..100 {
            slots_seen.insert(format!("{:?}", roll_random_slot(&mut rng)));
        }

        // Should see most/all slots in 100 rolls
        assert!(slots_seen.len() >= 5);
    }

    #[test]
    fn test_try_drop_item_respects_prestige() {
        let mut game_state = GameState::new(Utc::now().timestamp());

        // With prestige 0, should get some drops
        let mut drops = 0;
        for _ in 0..100 {
            if try_drop_item(&game_state).is_some() {
                drops += 1;
            }
        }
        // ~30% drop rate, so expect 20-40 drops
        assert!(drops > 15 && drops < 50);

        // Higher prestige should increase drops
        game_state.prestige_rank = 4; // 30% + 20% = 50% drop rate
        let mut high_prestige_drops = 0;
        for _ in 0..100 {
            if try_drop_item(&game_state).is_some() {
                high_prestige_drops += 1;
            }
        }
        assert!(high_prestige_drops > drops); // Should be noticeably higher
    }
}
