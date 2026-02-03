#![allow(dead_code)]
use crate::constants::{ITEM_DROP_BASE_CHANCE, ITEM_DROP_MAX_CHANCE, ITEM_DROP_PRESTIGE_BONUS};
use crate::game_state::GameState;
use crate::item_generation::generate_item;
use crate::items::{EquipmentSlot, Item, Rarity};
use rand::Rng;

pub fn drop_chance_for_prestige(prestige_rank: u32) -> f64 {
    let chance = ITEM_DROP_BASE_CHANCE + (prestige_rank as f64 * ITEM_DROP_PRESTIGE_BONUS);
    chance.min(ITEM_DROP_MAX_CHANCE)
}

pub fn try_drop_item(game_state: &GameState) -> Option<Item> {
    let mut rng = rand::thread_rng();

    let drop_chance = drop_chance_for_prestige(game_state.prestige_rank);

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

    // Prestige gives a small bonus (1% per rank, max 10%) that shifts
    // weight from Common toward higher rarities.
    let prestige_bonus = (prestige_rank as f64 * 0.01).min(0.10);

    // Base distribution: 55% Common, 30% Magic, 12% Rare, 2.5% Epic, 0.5% Legendary
    // Prestige shifts Common down and spreads the bonus across higher tiers.
    let common_threshold = 0.55 - prestige_bonus;
    let magic_threshold = common_threshold + 0.30;
    let rare_threshold = magic_threshold + 0.12 + prestige_bonus * 0.4;
    let epic_threshold = rare_threshold + 0.025 + prestige_bonus * 0.4;
    // Legendary is the remainder: 0.5% base + 20% of prestige bonus

    if roll < common_threshold {
        Rarity::Common
    } else if roll < magic_threshold {
        Rarity::Magic
    } else if roll < rare_threshold {
        Rarity::Rare
    } else if roll < epic_threshold {
        Rarity::Epic
    } else {
        Rarity::Legendary
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
    fn test_roll_rarity_base_distribution() {
        // At prestige 0: ~55% Common, 30% Magic, 12% Rare, 2.5% Epic, 0.5% Legendary
        let mut common = 0;
        let mut magic = 0;
        let mut rare = 0;
        let mut epic = 0;
        let mut legendary = 0;

        for _ in 0..10000 {
            let mut rng = rand::thread_rng();
            match roll_rarity(0, &mut rng) {
                Rarity::Common => common += 1,
                Rarity::Magic => magic += 1,
                Rarity::Rare => rare += 1,
                Rarity::Epic => epic += 1,
                Rarity::Legendary => legendary += 1,
            }
        }

        // Common should be the majority
        assert!(common > 4500, "Common should be ~55%, got {common}");
        assert!(magic > 2500, "Magic should be ~30%, got {magic}");
        assert!(rare > 800, "Rare should be ~12%, got {rare}");
        assert!(epic > 50, "Epic should be ~2.5%, got {epic}");
        // Legendary is rare but should appear in 10k rolls
        assert!(legendary > 0, "Legendary should appear, got {legendary}");
    }

    #[test]
    fn test_roll_rarity_all_prestiges_can_drop_legendary() {
        // Even at prestige 0, legendary is possible (0.5% chance)
        let mut found_legendary = false;
        for _ in 0..5000 {
            let mut rng = rand::thread_rng();
            if roll_rarity(0, &mut rng) == Rarity::Legendary {
                found_legendary = true;
                break;
            }
        }
        assert!(
            found_legendary,
            "Legendary should be possible at prestige 0"
        );
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

        // With prestige 0, should get some drops (~15% rate)
        let mut drops = 0;
        for _ in 0..200 {
            if try_drop_item(&game_state).is_some() {
                drops += 1;
            }
        }
        assert!(drops > 10 && drops < 55, "Expected ~15% drops, got {drops}");

        // Higher prestige gives a modest increase (+1% per rank)
        game_state.prestige_rank = 10; // 15% + 10% = 25% (cap)
        let mut high_prestige_drops = 0;
        for _ in 0..200 {
            if try_drop_item(&game_state).is_some() {
                high_prestige_drops += 1;
            }
        }
        assert!(
            high_prestige_drops > drops,
            "Higher prestige should increase drops slightly"
        );
    }

    #[test]
    fn test_roll_rarity_prestige_shifts_distribution() {
        let mut rng = rand::thread_rng();

        // At high prestige, Common % should decrease compared to prestige 0
        let mut common_p0 = 0;
        let mut common_p10 = 0;
        let trials = 10000;

        for _ in 0..trials {
            if roll_rarity(0, &mut rng) == Rarity::Common {
                common_p0 += 1;
            }
            if roll_rarity(10, &mut rng) == Rarity::Common {
                common_p10 += 1;
            }
        }

        // Prestige 10 should have noticeably fewer commons (45% vs 55%)
        assert!(
            common_p10 < common_p0,
            "High prestige should reduce common rate: p0={common_p0}, p10={common_p10}"
        );
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
    fn test_drop_chance_capped_at_max() {
        // Very high prestige should be capped at ITEM_DROP_MAX_CHANCE (50%)
        let chance = drop_chance_for_prestige(100);
        assert!(
            (chance - ITEM_DROP_MAX_CHANCE).abs() < f64::EPSILON,
            "Drop chance should cap at {}%, got {}%",
            ITEM_DROP_MAX_CHANCE * 100.0,
            chance * 100.0,
        );
    }

    #[test]
    fn test_drop_chance_for_prestige_values() {
        // Prestige 0: 15% base
        assert!((drop_chance_for_prestige(0) - 0.15).abs() < f64::EPSILON);
        // Prestige 5: 15% + 5% = 20%
        assert!((drop_chance_for_prestige(5) - 0.20).abs() < f64::EPSILON);
        // Prestige 10: 15% + 10% = 25% (cap)
        assert!((drop_chance_for_prestige(10) - 0.25).abs() < f64::EPSILON);
        // Prestige 20: still capped at 25%
        assert!((drop_chance_for_prestige(20) - 0.25).abs() < f64::EPSILON);
    }
}
