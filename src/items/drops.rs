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

/// Calculate item level from zone ID.
/// Zone 1 = ilvl 10, Zone 10 = ilvl 100.
pub fn ilvl_for_zone(zone_id: usize) -> u32 {
    (zone_id as u32) * 10
}

/// Try to drop an item from a normal mob (non-boss).
/// Legendaries CANNOT drop from normal mobs.
/// Haven bonuses apply to mob drops.
pub fn try_drop_from_mob(
    game_state: &GameState,
    zone_id: usize,
    haven_drop_rate_percent: f64,
    haven_rarity_percent: f64,
) -> Option<Item> {
    let mut rng = rand::thread_rng();

    // Apply Trophy Hall bonus to drop chance
    let base_chance = drop_chance_for_prestige(game_state.prestige_rank);
    let drop_chance =
        (base_chance * (1.0 + haven_drop_rate_percent / 100.0)).min(ITEM_DROP_MAX_CHANCE);

    if rng.gen::<f64>() > drop_chance {
        return None;
    }

    // Roll rarity - capped at Epic for mobs
    let rarity = roll_rarity_for_mob(game_state.prestige_rank, haven_rarity_percent, &mut rng);

    // Roll random equipment slot
    let slot = roll_random_slot(&mut rng);

    // Generate item with zone-based ilvl
    let ilvl = ilvl_for_zone(zone_id);
    Some(generate_item(slot, rarity, ilvl))
}

/// Try to drop an item from a boss.
/// Bosses always drop an item and can drop Legendaries.
/// Haven bonuses do NOT apply to boss drops (fixed rates).
pub fn try_drop_from_boss(zone_id: usize, is_final_zone: bool) -> Item {
    let mut rng = rand::thread_rng();

    // Roll rarity with boss drop table
    let rarity = roll_rarity_for_boss(is_final_zone, &mut rng);

    // Roll random equipment slot
    let slot = roll_random_slot(&mut rng);

    // Generate item with zone-based ilvl
    let ilvl = ilvl_for_zone(zone_id);
    generate_item(slot, rarity, ilvl)
}

/// Roll rarity for mob drops - caps at Epic (no legendaries).
/// Haven Workshop bonus shifts distribution toward higher rarities.
pub fn roll_rarity_for_mob(
    prestige_rank: u32,
    haven_rarity_percent: f64,
    rng: &mut impl Rng,
) -> Rarity {
    let roll = rng.gen::<f64>();

    // Prestige gives a small bonus (1% per rank, max 10%)
    let prestige_bonus = (prestige_rank as f64 * 0.01).min(0.10);

    // Workshop bonus: shifts distribution toward higher rarities (max 25%)
    let haven_bonus = (haven_rarity_percent / 100.0).min(0.25);
    let total_bonus = prestige_bonus + haven_bonus;

    // Mob distribution: 60% Common, 28% Magic, 10% Rare, 2% Epic, 0% Legendary
    // Bonuses shift Common down and spread across higher tiers.
    let common_threshold = (0.60 - total_bonus).max(0.20); // Never go below 20% common
    let magic_threshold = common_threshold + 0.28;
    let rare_threshold = magic_threshold + 0.10 + total_bonus * 0.6;
    // Epic is the remainder (capped, no legendary)

    if roll < common_threshold {
        Rarity::Common
    } else if roll < magic_threshold {
        Rarity::Magic
    } else if roll < rare_threshold {
        Rarity::Rare
    } else {
        Rarity::Epic
    }
}

/// Roll rarity for boss drops - can include Legendary.
/// Fixed rates, no Haven/prestige bonuses.
pub fn roll_rarity_for_boss(is_final_zone: bool, rng: &mut impl Rng) -> Rarity {
    let roll = rng.gen::<f64>();

    if is_final_zone {
        // Zone 10 final boss: 20% Magic, 40% Rare, 30% Epic, 10% Legendary
        if roll < 0.20 {
            Rarity::Magic
        } else if roll < 0.60 {
            Rarity::Rare
        } else if roll < 0.90 {
            Rarity::Epic
        } else {
            Rarity::Legendary
        }
    } else {
        // Normal zone boss: 40% Magic, 35% Rare, 20% Epic, 5% Legendary
        if roll < 0.40 {
            Rarity::Magic
        } else if roll < 0.75 {
            Rarity::Rare
        } else if roll < 0.95 {
            Rarity::Epic
        } else {
            Rarity::Legendary
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

// ============================================================================
// Legacy compatibility functions
// ============================================================================

/// Legacy function - redirects to try_drop_from_mob with zone 1.
/// Kept for backwards compatibility with existing callers.
pub fn try_drop_item(game_state: &GameState) -> Option<Item> {
    try_drop_from_mob(game_state, 1, 0.0, 0.0)
}

/// Legacy function - redirects to try_drop_from_mob.
/// Kept for backwards compatibility with existing callers.
pub fn try_drop_item_with_haven(
    game_state: &GameState,
    haven_drop_rate_percent: f64,
    haven_rarity_percent: f64,
) -> Option<Item> {
    // Use zone 1 as default - callers should migrate to try_drop_from_mob
    try_drop_from_mob(game_state, 1, haven_drop_rate_percent, haven_rarity_percent)
}

/// Legacy rarity roll - redirects to mob rarity (no legendaries).
pub fn roll_rarity(prestige_rank: u32, rng: &mut impl Rng) -> Rarity {
    roll_rarity_for_mob(prestige_rank, 0.0, rng)
}

/// Legacy rarity roll with haven - redirects to mob rarity (no legendaries).
pub fn roll_rarity_with_haven(
    prestige_rank: u32,
    haven_rarity_percent: f64,
    rng: &mut impl Rng,
) -> Rarity {
    roll_rarity_for_mob(prestige_rank, haven_rarity_percent, rng)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_ilvl_for_zone() {
        assert_eq!(ilvl_for_zone(1), 10);
        assert_eq!(ilvl_for_zone(5), 50);
        assert_eq!(ilvl_for_zone(10), 100);
    }

    #[test]
    fn test_mob_drops_never_legendary() {
        let mut rng = rand::thread_rng();

        // Roll 10000 times - should never get legendary
        for _ in 0..10000 {
            let rarity = roll_rarity_for_mob(10, 25.0, &mut rng); // Max bonuses
            assert_ne!(
                rarity,
                Rarity::Legendary,
                "Mobs should never drop legendary"
            );
        }
    }

    #[test]
    fn test_boss_can_drop_legendary() {
        let mut rng = rand::thread_rng();
        let mut found_legendary = false;

        // Normal boss has 5% legendary rate
        for _ in 0..500 {
            if roll_rarity_for_boss(false, &mut rng) == Rarity::Legendary {
                found_legendary = true;
                break;
            }
        }
        assert!(found_legendary, "Boss should be able to drop legendary");
    }

    #[test]
    fn test_final_boss_higher_legendary_rate() {
        let mut rng = rand::thread_rng();
        let trials = 10000;

        let mut normal_legendaries = 0;
        let mut final_legendaries = 0;

        for _ in 0..trials {
            if roll_rarity_for_boss(false, &mut rng) == Rarity::Legendary {
                normal_legendaries += 1;
            }
            if roll_rarity_for_boss(true, &mut rng) == Rarity::Legendary {
                final_legendaries += 1;
            }
        }

        // Final boss should have roughly 2x the legendary rate
        assert!(
            final_legendaries > normal_legendaries,
            "Final boss should have higher legendary rate: normal={}, final={}",
            normal_legendaries,
            final_legendaries
        );
    }

    #[test]
    fn test_boss_never_drops_common() {
        let mut rng = rand::thread_rng();

        for _ in 0..1000 {
            let rarity = roll_rarity_for_boss(false, &mut rng);
            assert_ne!(rarity, Rarity::Common, "Bosses should never drop common");

            let rarity = roll_rarity_for_boss(true, &mut rng);
            assert_ne!(
                rarity,
                Rarity::Common,
                "Final boss should never drop common"
            );
        }
    }

    #[test]
    fn test_mob_distribution_base() {
        let mut rng = rand::thread_rng();
        let trials = 10000;

        let mut common = 0;
        let mut magic = 0;
        let mut rare = 0;
        let mut epic = 0;

        for _ in 0..trials {
            match roll_rarity_for_mob(0, 0.0, &mut rng) {
                Rarity::Common => common += 1,
                Rarity::Magic => magic += 1,
                Rarity::Rare => rare += 1,
                Rarity::Epic => epic += 1,
                Rarity::Legendary => panic!("Should never happen"),
            }
        }

        // Expected: 60% common, 28% magic, 10% rare, 2% epic
        assert!(common > 5000, "Common should be ~60%, got {}", common);
        assert!(magic > 2000, "Magic should be ~28%, got {}", magic);
        assert!(rare > 500, "Rare should be ~10%, got {}", rare);
        assert!(epic > 50, "Epic should be ~2%, got {}", epic);
    }

    #[test]
    fn test_try_drop_from_mob_respects_zone_ilvl() {
        let game_state = GameState::new("Test Hero".to_string(), Utc::now().timestamp());

        // Try to get drops from different zones
        let mut zone1_drops = Vec::new();
        let mut zone10_drops = Vec::new();

        for _ in 0..1000 {
            if let Some(item) = try_drop_from_mob(&game_state, 1, 0.0, 0.0) {
                zone1_drops.push(item);
            }
            if let Some(item) = try_drop_from_mob(&game_state, 10, 0.0, 0.0) {
                zone10_drops.push(item);
            }
        }

        // Verify ilvls
        for item in &zone1_drops {
            assert_eq!(item.ilvl, 10, "Zone 1 items should have ilvl 10");
        }
        for item in &zone10_drops {
            assert_eq!(item.ilvl, 100, "Zone 10 items should have ilvl 100");
        }
    }

    #[test]
    fn test_try_drop_from_boss_always_drops() {
        // Boss drops are guaranteed
        for zone_id in 1..=10 {
            let item = try_drop_from_boss(zone_id, zone_id == 10);
            assert_eq!(item.ilvl, (zone_id as u32) * 10);
            assert!(!item.display_name.is_empty());
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
    fn test_drop_chance_capped_at_max() {
        let chance = drop_chance_for_prestige(100);
        assert!(
            (chance - ITEM_DROP_MAX_CHANCE).abs() < f64::EPSILON,
            "Drop chance should cap at {}%, got {}%",
            ITEM_DROP_MAX_CHANCE * 100.0,
            chance * 100.0,
        );
    }

    #[test]
    fn test_haven_bonuses_affect_mob_drops() {
        let mut rng = rand::thread_rng();
        let trials = 10000;

        let mut common_no_bonus = 0;
        let mut common_with_bonus = 0;

        for _ in 0..trials {
            if roll_rarity_for_mob(0, 0.0, &mut rng) == Rarity::Common {
                common_no_bonus += 1;
            }
            if roll_rarity_for_mob(0, 25.0, &mut rng) == Rarity::Common {
                common_with_bonus += 1;
            }
        }

        // With +25% haven bonus, common rate should drop significantly
        assert!(
            common_with_bonus < common_no_bonus - 500,
            "Haven bonus should reduce common rate: no_bonus={}, with_bonus={}",
            common_no_bonus,
            common_with_bonus
        );
    }

    #[test]
    fn test_legacy_functions_work() {
        let game_state = GameState::new("Test Hero".to_string(), Utc::now().timestamp());

        // Legacy functions should not panic
        let _ = try_drop_item(&game_state);
        let _ = try_drop_item_with_haven(&game_state, 10.0, 10.0);

        let mut rng = rand::thread_rng();
        let _ = roll_rarity(5, &mut rng);
        let _ = roll_rarity_with_haven(5, 10.0, &mut rng);
    }
}
