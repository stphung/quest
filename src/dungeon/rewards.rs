//! Dungeon rewards: XP, item generation, and treasure room handling.

use crate::core::game_state::GameState;
use crate::items::{
    generate_item, ilvl_for_zone, roll_random_slot, roll_rarity_for_mob, Item, Rarity,
};
use rand::RngExt;

use super::types::DungeonSize;

/// Calculates the XP reward for defeating a dungeon boss
pub fn calculate_boss_xp_reward(size: DungeonSize) -> u64 {
    let mut rng = rand::rng();
    let (min_xp, max_xp) = size.boss_xp_range();
    rng.random_range(min_xp..=max_xp)
}

/// Generates a treasure room item with rarity boost based on dungeon size.
/// `zone_id` determines item level (ilvl = zone_id * 10).
/// Haven Workshop bonus applies to base rarity before dungeon size boost.
pub fn generate_treasure_item(
    prestige_rank: u32,
    zone_id: usize,
    rarity_boost: u32,
    haven_rarity_percent: f64,
) -> Item {
    let mut rng = rand::rng();

    // Roll a random slot
    let slot = roll_random_slot(&mut rng);

    // Roll rarity with Haven bonus, then boost based on dungeon tier
    let base_rarity = roll_rarity_for_mob(prestige_rank, haven_rarity_percent, &mut rng);
    let boosted_rarity = boost_rarity(base_rarity, rarity_boost);

    // Item level based on zone
    let ilvl = ilvl_for_zone(zone_id);

    generate_item(slot, boosted_rarity, ilvl)
}

/// Boosts a rarity by N tiers (capped at Legendary)
fn boost_rarity(rarity: Rarity, boost: u32) -> Rarity {
    if rarity == Rarity::Mythic {
        return Rarity::Mythic; // God items are never rarity-shifted
    }

    let rarity_level = match rarity {
        Rarity::Common => 0,
        Rarity::Magic => 1,
        Rarity::Rare => 2,
        Rarity::Epic => 3,
        Rarity::Legendary => 4,
        Rarity::Mythic => unreachable!(),
    };

    match (rarity_level + boost).min(4) {
        0 => Rarity::Common,
        1 => Rarity::Magic,
        2 => Rarity::Rare,
        3 => Rarity::Epic,
        _ => Rarity::Legendary,
    }
}

/// Adds XP earned to the dungeon tally
pub fn add_dungeon_xp(state: &mut GameState, xp: u64) {
    if let Some(dungeon) = &mut state.active_dungeon {
        dungeon.xp_earned += xp;
    }
}

/// Adds an item to the dungeon collected items
#[allow(dead_code)]
pub fn collect_dungeon_item(state: &mut GameState, item: Item) {
    if let Some(dungeon) = &mut state.active_dungeon {
        dungeon.collected_items.push(item);
    }
}

/// Called when player enters a treasure room - generates and collects an item
/// Returns (item, was_equipped)
pub fn on_treasure_room_entered(
    state: &mut GameState,
    haven_rarity_percent: f64,
) -> Option<(Item, bool)> {
    // Get rarity boost from dungeon size (defaults to 1 if no dungeon somehow)
    let rarity_boost = state
        .active_dungeon
        .as_ref()
        .map(|d| d.size.treasure_rarity_boost())
        .unwrap_or(1);

    // Use current zone for item level
    let zone_id = state.zone_progression.current_zone_id as usize;

    let item = generate_treasure_item(
        state.prestige_rank,
        zone_id,
        rarity_boost,
        haven_rarity_percent,
    );

    // Auto-equip if better
    let item_clone = item.clone();
    let equipped = crate::items::auto_equip_if_better(item, state);

    // Collect in dungeon tally (whether equipped or not, for completion summary)
    if let Some(dungeon) = &mut state.active_dungeon {
        dungeon.collected_items.push(item_clone.clone());
    }

    Some((item_clone, equipped))
}
