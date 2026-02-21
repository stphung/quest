//! Prestige eligibility checks and prestige execution logic.

use crate::core::constants::*;
use crate::core::game_state::GameState;

use super::tiers::get_next_prestige_tier;

/// Checks if the player can prestige
///
/// # Arguments
/// * `state` - The current game state
///
/// # Returns
/// true if character level meets the required level for next prestige tier
pub fn can_prestige(state: &GameState) -> bool {
    let next_tier = get_next_prestige_tier(state.prestige_rank);
    state.character_level >= next_tier.required_level
}

/// Performs a prestige, resetting character progress and incrementing prestige rank
///
/// # Arguments
/// * `state` - The game state to modify
pub fn perform_prestige(state: &mut GameState) {
    use super::attributes::Attributes;
    use crate::combat::CombatState;
    use crate::items::Equipment;

    // Only prestige if eligible
    if !can_prestige(state) {
        return;
    }

    // Reset character to level 1, XP 0
    state.character_level = 1;
    state.character_xp = 0;

    // Reset attributes to base 10
    state.attributes = Attributes::new();

    // Reset equipment (complete wipe)
    state.equipment = Equipment::new();

    // Reset active dungeon
    state.active_dungeon = None;

    // Clear active fishing session (transient state)
    // Note: Fishing rank and progression (total_fish_caught, legendary_catches, etc.)
    // are intentionally preserved across prestige as a separate progression track
    state.active_fishing = None;

    // Clear any active minigame session
    state.active_minigame = None;

    // Reset combat state with base HP for fresh attributes
    state.combat_state = CombatState::new(BASE_HP as u32);

    // Increment prestige rank and total prestige count
    state.prestige_rank += 1;
    state.total_prestige_count += 1;

    // Clear XP rate tracking so ETA recalculates from fresh post-prestige data
    state.xp_rate_samples.clear();
    state.xp_this_second = 0;
    state.combat_seconds_this_tick = false;

    // Reset zone progression but keep unlocks based on new prestige rank
    state
        .zone_progression
        .reset_for_prestige(state.prestige_rank);
}

/// Performs prestige with Vault item preservation.
/// `preserved_slots` contains the equipment slots to keep (limited by Vault tier externally).
pub fn perform_prestige_with_vault(
    state: &mut GameState,
    preserved_slots: &[crate::items::EquipmentSlot],
) {
    use crate::items::EquipmentSlot;

    if !can_prestige(state) {
        return;
    }

    // Save items from preserved slots before reset
    let mut saved_items: Vec<(EquipmentSlot, crate::items::Item)> = Vec::new();
    for slot in preserved_slots {
        if let Some(item) = state.equipment.get(*slot) {
            saved_items.push((*slot, item.clone()));
        }
    }

    // Normal prestige reset
    perform_prestige(state);

    // Restore preserved items
    for (slot, item) in saved_items {
        state.equipment.set(slot, Some(item));
    }
}
