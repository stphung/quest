//! Item drop logic for fishing catches.

use super::types::FishRarity;
use crate::core::constants::{
    FISHING_DROP_CHANCE_COMMON, FISHING_DROP_CHANCE_EPIC, FISHING_DROP_CHANCE_LEGENDARY,
    FISHING_DROP_CHANCE_RARE, FISHING_DROP_CHANCE_UNCOMMON,
};
use crate::items::generation as item_generation;
use crate::items::{ilvl_for_zone, roll_random_slot, Rarity};
use rand::{Rng, RngExt};

/// Attempts to drop an item based on fish rarity.
///
/// Drop chances:
/// - Common: 5%
/// - Uncommon: 5%
/// - Rare: 15%
/// - Epic: 35%
/// - Legendary: 75%
///
/// Item level is based on zone_id (ilvl = zone_id * 10).
pub(crate) fn try_fishing_item_drop(
    rarity: FishRarity,
    zone_id: usize,
    rng: &mut impl Rng,
) -> Option<crate::items::Item> {
    let drop_chance = match rarity {
        FishRarity::Common => FISHING_DROP_CHANCE_COMMON,
        FishRarity::Uncommon => FISHING_DROP_CHANCE_UNCOMMON,
        FishRarity::Rare => FISHING_DROP_CHANCE_RARE,
        FishRarity::Epic => FISHING_DROP_CHANCE_EPIC,
        FishRarity::Legendary => FISHING_DROP_CHANCE_LEGENDARY,
    };

    if rng.random::<f64>() < drop_chance {
        // Generate item with rarity matching fish rarity
        let item_rarity = match rarity {
            FishRarity::Common => Rarity::Common,
            FishRarity::Uncommon => Rarity::Magic,
            FishRarity::Rare => Rarity::Rare,
            FishRarity::Epic => Rarity::Epic,
            FishRarity::Legendary => Rarity::Legendary,
        };

        // Random equipment slot
        let slot = roll_random_slot(rng);

        // Item level based on zone
        let ilvl = ilvl_for_zone(zone_id);

        Some(item_generation::generate_item_with_rng(
            slot,
            item_rarity,
            ilvl,
            rng,
        ))
    } else {
        None
    }
}
