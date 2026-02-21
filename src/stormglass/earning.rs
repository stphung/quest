//! Pure functions for calculating Stormglass earnings.

use crate::dungeon::types::DungeonSize;
use crate::items::types::Rarity;

use super::types::*;

/// Returns the Stormglass value for salvaging an item of the given rarity.
pub fn salvage_value(rarity: Rarity) -> u64 {
    match rarity {
        Rarity::Common => SALVAGE_COMMON,
        Rarity::Magic => SALVAGE_MAGIC,
        Rarity::Rare => SALVAGE_RARE,
        Rarity::Epic => SALVAGE_EPIC,
        Rarity::Legendary => SALVAGE_LEGENDARY,
        Rarity::Mythic => SALVAGE_LEGENDARY, // God items are never salvaged in practice
    }
}

/// Returns the Stormglass cache found in a dungeon treasure room based on dungeon size.
pub fn dungeon_cache(size: DungeonSize) -> u64 {
    match size {
        DungeonSize::Small => DUNGEON_CACHE_SMALL,
        DungeonSize::Medium => DUNGEON_CACHE_MEDIUM,
        DungeonSize::Large => DUNGEON_CACHE_LARGE,
        DungeonSize::Epic => DUNGEON_CACHE_EPIC,
        DungeonSize::Legendary => DUNGEON_CACHE_LEGENDARY,
    }
}

/// Returns the Stormglass consolation for a failed Soulforge enhancement.
/// Only awards SG for target levels 5+ (where failure is possible).
pub fn soulforge_consolation(target_level: u8) -> u64 {
    if target_level == 0 || target_level as usize > SOULFORGE_CONSOLATION.len() {
        return 0;
    }
    SOULFORGE_CONSOLATION[(target_level - 1) as usize]
}
