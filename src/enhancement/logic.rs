use super::types::*;
use rand::{Rng, RngExt};

/// Attempt to enhance a slot. Returns true on success, false on failure.
/// Caller must verify prestige_rank >= cost and level < MAX before calling.
pub fn attempt_enhancement<R: Rng>(
    enhancement: &mut EnhancementProgress,
    slot_index: usize,
    rng: &mut R,
) -> bool {
    let current_level = enhancement.level(slot_index);
    if current_level >= MAX_ENHANCEMENT_LEVEL {
        return false;
    }

    let target_level = current_level + 1;
    let rate = success_rate(target_level);
    enhancement.total_attempts += 1;

    if rng.random::<f64>() < rate {
        enhancement.set_level(slot_index, target_level);
        enhancement.total_successes += 1;
        true
    } else {
        let penalty = fail_penalty(target_level);
        let new_level = current_level.saturating_sub(penalty);
        enhancement.set_level(slot_index, new_level);
        enhancement.total_failures += 1;
        false
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
