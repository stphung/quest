#![allow(dead_code)]
use super::generation::generate_item;
use super::types::{EquipmentSlot, Item, Rarity};
use crate::core::constants::{
    ITEM_DROP_BASE_CHANCE, ITEM_DROP_MAX_CHANCE, ITEM_DROP_PRESTIGE_BONUS,
};
use crate::core::game_state::GameState;
use rand::Rng;

pub fn drop_chance_for_prestige(prestige_rank: u32) -> f64 {
    let chance = ITEM_DROP_BASE_CHANCE + (prestige_rank as f64 * ITEM_DROP_PRESTIGE_BONUS);
    chance.min(ITEM_DROP_MAX_CHANCE)
}

/// Try to drop an item after killing an enemy
/// `haven_drop_rate_percent` is the Trophy Hall bonus (0.0 if not built)
/// `haven_rarity_percent` is the Workshop bonus (0.0 if not built)
pub fn try_drop_item_with_haven(
    game_state: &GameState,
    haven_drop_rate_percent: f64,
    haven_rarity_percent: f64,
) -> Option<Item> {
    let mut rng = rand::thread_rng();

    // Apply Trophy Hall bonus to drop chance
    let base_chance = drop_chance_for_prestige(game_state.prestige_rank);
    let drop_chance = (base_chance * (1.0 + haven_drop_rate_percent / 100.0)).min(ITEM_DROP_MAX_CHANCE);

    if rng.gen::<f64>() > drop_chance {
        return None;
    }

    // Roll rarity with Workshop bonus
    let rarity = roll_rarity_with_haven(game_state.prestige_rank, haven_rarity_percent, &mut rng);

    // Roll random equipment slot
    let slot = roll_random_slot(&mut rng);

    // Generate item
    Some(generate_item(slot, rarity, game_state.character_level))
}

/// Legacy function without Haven bonuses (for backwards compatibility)
pub fn try_drop_item(game_state: &GameState) -> Option<Item> {
    try_drop_item_with_haven(game_state, 0.0, 0.0)
}

pub fn roll_rarity(prestige_rank: u32, rng: &mut impl Rng) -> Rarity {
    roll_rarity_with_haven(prestige_rank, 0.0, rng)
}

/// Roll item rarity with Haven Workshop bonus
/// `haven_rarity_percent` shifts distribution toward higher rarities
pub fn roll_rarity_with_haven(prestige_rank: u32, haven_rarity_percent: f64, rng: &mut impl Rng) -> Rarity {
    let roll = rng.gen::<f64>();

    // Prestige gives a small bonus (1% per rank, max 10%) that shifts
    // weight from Common toward higher rarities.
    let prestige_bonus = (prestige_rank as f64 * 0.01).min(0.10);

    // Workshop bonus: shifts distribution further toward higher rarities
    // Max 25% at T3 which significantly reduces common rate
    let haven_bonus = (haven_rarity_percent / 100.0).min(0.25);
    let total_bonus = prestige_bonus + haven_bonus;

    // Base distribution: 55% Common, 30% Magic, 12% Rare, 2.5% Epic, 0.5% Legendary
    // Bonuses shift Common down and spread across higher tiers.
    let common_threshold = (0.55 - total_bonus).max(0.10); // Never go below 10% common
    let magic_threshold = common_threshold + 0.30;
    let rare_threshold = magic_threshold + 0.12 + total_bonus * 0.4;
    let epic_threshold = rare_threshold + 0.025 + total_bonus * 0.4;
    // Legendary is the remainder

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
        let trials = 2000;
        let mut drops = 0;
        for _ in 0..trials {
            if try_drop_item(&game_state).is_some() {
                drops += 1;
            }
        }
        assert!(
            drops > 200 && drops < 450,
            "Expected ~15% drops, got {drops}/{trials}"
        );

        // Higher prestige gives a modest increase (+1% per rank)
        game_state.prestige_rank = 10; // 15% + 10% = 25% (cap)
        let mut high_prestige_drops = 0;
        for _ in 0..trials {
            if try_drop_item(&game_state).is_some() {
                high_prestige_drops += 1;
            }
        }
        assert!(
            high_prestige_drops > drops,
            "Higher prestige should increase drops: p0={drops}, p10={high_prestige_drops}"
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

    // =========================================================================
    // Haven Bonus Tests
    // =========================================================================

    #[test]
    fn test_haven_drop_rate_bonus() {
        let game_state = GameState::new("Test Hero".to_string(), Utc::now().timestamp());

        // Compare drop rates with and without Haven bonus
        let trials = 5000;
        let mut drops_no_bonus = 0;
        let mut drops_with_bonus = 0;

        for _ in 0..trials {
            if try_drop_item_with_haven(&game_state, 0.0, 0.0).is_some() {
                drops_no_bonus += 1;
            }
            if try_drop_item_with_haven(&game_state, 15.0, 0.0).is_some() { // +15% drop rate from Trophy Hall
                drops_with_bonus += 1;
            }
        }

        // With +15% bonus, should see roughly 15% more drops
        assert!(
            drops_with_bonus > drops_no_bonus,
            "Haven +15% drop rate should increase drops: no_bonus={}, with_bonus={}",
            drops_no_bonus,
            drops_with_bonus
        );
    }

    #[test]
    fn test_haven_rarity_bonus() {
        let mut rng = rand::thread_rng();

        // Compare rarity distributions with and without Haven bonus
        let trials = 10000;
        let mut common_no_bonus = 0;
        let mut common_with_bonus = 0;

        for _ in 0..trials {
            if roll_rarity_with_haven(0, 0.0, &mut rng) == Rarity::Common {
                common_no_bonus += 1;
            }
            if roll_rarity_with_haven(0, 25.0, &mut rng) == Rarity::Common { // +25% from Workshop T3
                common_with_bonus += 1;
            }
        }

        // Workshop bonus should significantly reduce common rate
        assert!(
            common_with_bonus < common_no_bonus - 500,
            "Haven +25% rarity should reduce common rate: no_bonus={}, with_bonus={}",
            common_no_bonus,
            common_with_bonus
        );
    }
}
