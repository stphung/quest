//! Dungeon rewards: XP, item generation, and treasure room handling.

use crate::core::game_state::GameState;
use crate::items::{
    generate_item, ilvl_for_zone, roll_random_slot, roll_rarity_for_mob, Item, Rarity,
};
use rand::{Rng, RngExt};

use super::types::DungeonSize;

/// Calculates the XP reward for defeating a dungeon boss
pub fn calculate_boss_xp_reward<R: Rng>(rng: &mut R, size: DungeonSize) -> u64 {
    let (min_xp, max_xp) = size.boss_xp_range();
    rng.random_range(min_xp..=max_xp)
}

/// Generates a treasure room item with rarity boost based on dungeon size.
/// `zone_id` determines item level (ilvl = zone_id * 10).
/// Haven Workshop bonus applies to base rarity before dungeon size boost.
pub fn generate_treasure_item<R: Rng>(
    rng: &mut R,
    prestige_rank: u32,
    zone_id: usize,
    rarity_boost: u32,
    haven_rarity_percent: f64,
) -> Item {
    // Roll a random slot
    let slot = roll_random_slot(rng);

    // Roll rarity with Haven bonus, then boost based on dungeon tier
    let base_rarity = roll_rarity_for_mob(prestige_rank, haven_rarity_percent, rng);
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
pub fn on_treasure_room_entered<R: Rng>(
    rng: &mut R,
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
        rng,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::GameState;
    use crate::dungeon::types::{Dungeon, DungeonSize};
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn boost_rarity_caps_at_legendary_and_preserves_mythic() {
        assert_eq!(boost_rarity(Rarity::Common, 0), Rarity::Common);
        assert_eq!(boost_rarity(Rarity::Common, 2), Rarity::Rare);
        assert_eq!(boost_rarity(Rarity::Epic, 5), Rarity::Legendary);
        assert_eq!(boost_rarity(Rarity::Legendary, 3), Rarity::Legendary);
        assert_eq!(boost_rarity(Rarity::Mythic, 3), Rarity::Mythic);
    }

    #[test]
    fn treasure_room_collects_item_into_active_dungeon() {
        let mut rng = ChaCha8Rng::seed_from_u64(7);
        let mut state = GameState::new("Hero".to_string(), 0);
        state.zone_progression.current_zone_id = 7;
        state.active_dungeon = Some(Dungeon::new(DungeonSize::Large));

        let (item, _equipped) = on_treasure_room_entered(&mut rng, &mut state, 0.0).unwrap();
        let dungeon = state.active_dungeon.as_ref().unwrap();

        assert_eq!(item.ilvl, ilvl_for_zone(7));
        assert_eq!(dungeon.collected_items.len(), 1);
        assert_eq!(dungeon.collected_items[0].display_name, item.display_name);
        assert_eq!(dungeon.collected_items[0].ilvl, item.ilvl);
    }

    #[test]
    fn treasure_room_without_active_dungeon_still_returns_item() {
        let mut rng = ChaCha8Rng::seed_from_u64(11);
        let mut state = GameState::new("Hero".to_string(), 0);
        state.zone_progression.current_zone_id = 3;

        let (item, _equipped) = on_treasure_room_entered(&mut rng, &mut state, 0.0).unwrap();

        assert_eq!(item.ilvl, ilvl_for_zone(3));
        assert!(state.active_dungeon.is_none());
    }
}
