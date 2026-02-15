use super::types::*;
use rand::{Rng, RngExt};

/// Roll the enhancement outcome without modifying any state.
/// Returns (success, new_level).
pub fn roll_enhancement<R: Rng>(current_level: u8, rng: &mut R) -> (bool, u8) {
    if current_level >= MAX_ENHANCEMENT_LEVEL {
        return (false, current_level);
    }

    let target_level = current_level + 1;
    let rate = success_rate(target_level);

    if rng.random::<f64>() < rate {
        (true, target_level)
    } else {
        let penalty = fail_penalty(target_level);
        (false, current_level.saturating_sub(penalty))
    }
}

/// Apply a pre-rolled enhancement result to the state.
pub fn apply_enhancement_result(
    enhancement: &mut EnhancementProgress,
    slot_index: usize,
    new_level: u8,
    success: bool,
) {
    enhancement.set_level(slot_index, new_level);
    enhancement.total_attempts += 1;
    if success {
        enhancement.total_successes += 1;
    } else {
        enhancement.total_failures += 1;
    }
}

pub fn blacksmith_discovery_chance(prestige_rank: u32) -> f64 {
    if prestige_rank < BLACKSMITH_MIN_PRESTIGE_RANK {
        return 0.0;
    }
    BLACKSMITH_DISCOVERY_BASE_CHANCE
        + (prestige_rank - BLACKSMITH_MIN_PRESTIGE_RANK) as f64 * BLACKSMITH_DISCOVERY_RANK_BONUS
}

pub fn try_discover_blacksmith<R: Rng>(
    enhancement: &mut EnhancementProgress,
    prestige_rank: u32,
    rng: &mut R,
) -> bool {
    if enhancement.discovered {
        return false;
    }
    let chance = blacksmith_discovery_chance(prestige_rank);
    if chance <= 0.0 {
        return false;
    }
    if rng.random::<f64>() < chance {
        enhancement.discovered = true;
        return true;
    }
    false
}
