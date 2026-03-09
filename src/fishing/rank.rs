//! Fishing rank progression logic.

#![allow(dead_code)]

use super::types::FishingState;
use crate::core::constants::{BASE_MAX_FISHING_RANK, MAX_FISHING_RANK};

/// Returns the effective maximum fishing rank based on Haven bonus.
///
/// Base max is 30, but FishingDock T4 adds +10 for a total of 40.
pub fn get_max_fishing_rank(fishing_rank_bonus: u32) -> u32 {
    (BASE_MAX_FISHING_RANK + fishing_rank_bonus).min(MAX_FISHING_RANK)
}

/// Checks if the player should rank up in fishing.
///
/// Returns a rank up message if the threshold is reached.
///
/// # Arguments
/// - `fishing_state`: The player's fishing state
/// - `max_rank`: The effective maximum rank (base 30 + Haven bonus)
///
/// # Rank Up Mechanics
/// - Each rank requires a certain number of fish to catch
/// - Fish requirement increases with rank tier
/// - Excess fish count carries over to next rank
/// - Rank is capped at the effective max rank
pub fn check_rank_up_with_max(fishing_state: &mut FishingState, max_rank: u32) -> Option<String> {
    // Already at max rank - no further progression
    if fishing_state.rank >= max_rank {
        return None;
    }

    let required = FishingState::fish_required_for_rank(fishing_state.rank);

    if fishing_state.fish_toward_next_rank >= required {
        // Rank up
        fishing_state.fish_toward_next_rank -= required;
        fishing_state.rank += 1;

        let new_rank_name = fishing_state.rank_name();
        Some(format!(
            "Fishing rank up! Now rank {}: {}",
            fishing_state.rank, new_rank_name
        ))
    } else {
        None
    }
}

/// Checks if the player should rank up in fishing (legacy, uses absolute max).
///
/// Returns a rank up message if the threshold is reached.
///
/// # Rank Up Mechanics
/// - Each rank requires a certain number of fish to catch
/// - Fish requirement increases with rank tier
/// - Excess fish count carries over to next rank
/// - Rank is capped at MAX_FISHING_RANK (40)
pub fn check_rank_up(fishing_state: &mut FishingState) -> Option<String> {
    check_rank_up_with_max(fishing_state, MAX_FISHING_RANK)
}
