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

pub fn roll_rarity(prestige_rank: u32, rng: &mut impl Rng) -> Rarity {
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

pub fn roll_random_slot(rng: &mut impl Rng) -> EquipmentSlot {
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
        let mut game_state = GameState::new("Test Hero".to_string(), Utc::now().timestamp());

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

    #[test]
    fn test_roll_rarity_silver_prestige() {
        let mut rng = rand::thread_rng();
        let mut counts = std::collections::HashMap::new();

        for _ in 0..1000 {
            let rarity = roll_rarity(2, &mut rng);
            *counts.entry(format!("{:?}", rarity)).or_insert(0) += 1;
        }

        // Silver (prestige 2-3): 30% Common, 40% Magic, 25% Rare, 5% Epic
        assert!(counts.get("Common").copied().unwrap_or(0) > 200);
        assert!(counts.get("Magic").copied().unwrap_or(0) > 300);
        assert!(counts.get("Rare").copied().unwrap_or(0) > 150);
        // Epic should appear but be uncommon
        assert!(counts.get("Epic").copied().unwrap_or(0) > 0);
        // No legendary at silver tier
        assert_eq!(counts.get("Legendary").copied().unwrap_or(0), 0);
    }

    #[test]
    fn test_roll_rarity_gold_prestige() {
        let mut rng = rand::thread_rng();
        let mut found_legendary = false;
        let mut found_epic = false;

        for _ in 0..2000 {
            let rarity = roll_rarity(4, &mut rng);
            if rarity == Rarity::Legendary {
                found_legendary = true;
            }
            if rarity == Rarity::Epic {
                found_epic = true;
            }
        }

        // Gold (prestige 4-5): 2% legendary, 13% epic
        assert!(found_epic, "Gold prestige should produce epic items");
        assert!(
            found_legendary,
            "Gold prestige should produce legendary items"
        );
    }

    #[test]
    fn test_roll_rarity_bronze_no_epic_or_legendary() {
        let mut rng = rand::thread_rng();

        for _ in 0..1000 {
            let rarity = roll_rarity(0, &mut rng);
            assert!(
                rarity == Rarity::Common || rarity == Rarity::Magic || rarity == Rarity::Rare,
                "Bronze prestige should not produce {:?}",
                rarity
            );
        }
    }

    #[test]
    fn test_roll_random_slot_all_slots_reachable() {
        let mut rng = rand::thread_rng();
        let mut slots_seen = std::collections::HashSet::new();

        for _ in 0..500 {
            slots_seen.insert(format!("{:?}", roll_random_slot(&mut rng)));
        }

        assert_eq!(
            slots_seen.len(),
            7,
            "All 7 equipment slots should be reachable"
        );
    }

    #[test]
    fn test_try_drop_item_returns_valid_items() {
        let game_state = GameState::new("Test Hero".to_string(), Utc::now().timestamp());

        for _ in 0..50 {
            if let Some(item) = try_drop_item(&game_state) {
                assert!(!item.display_name.is_empty());
                assert!(item.attributes.total() > 0);
                // Verify affix count matches rarity contract
                match item.rarity {
                    Rarity::Common => assert_eq!(item.affixes.len(), 0),
                    Rarity::Magic => assert_eq!(item.affixes.len(), 1),
                    Rarity::Rare => assert!(item.affixes.len() >= 2 && item.affixes.len() <= 3),
                    Rarity::Epic => assert!(item.affixes.len() >= 3 && item.affixes.len() <= 4),
                    Rarity::Legendary => {
                        assert!(item.affixes.len() >= 4 && item.affixes.len() <= 5)
                    }
                }
            }
        }
    }

    #[test]
    fn test_very_high_prestige_drop_rate_capped_behavior() {
        // At prestige 14+: drop_chance = 0.30 + 14*0.05 = 1.0 (100%)
        let mut game_state = GameState::new("Test Hero".to_string(), Utc::now().timestamp());
        game_state.prestige_rank = 14;

        let mut drops = 0;
        for _ in 0..50 {
            if try_drop_item(&game_state).is_some() {
                drops += 1;
            }
        }
        // At 100% drop rate, all should drop
        assert_eq!(drops, 50, "Prestige 14 should give 100% drop rate");
    }
}
