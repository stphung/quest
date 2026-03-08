//! Ascension logic — eligibility checks and execution.
//!
//! Full implementation in Task 5 (depends on GameState.ascension_level field).

/// Result of an Ascend action.
#[derive(Debug, Clone, PartialEq)]
pub enum AscendResult {
    /// Successfully ascended to the given level.
    Success { new_level: u32, multiplier: f64 },
    /// Not enough prestige ranks.
    InsufficientPR { needed: u32, have: u32 },
    /// Deep layer gate not met.
    DeepGateNotMet {
        needed_layer: u32,
        current_layer: u32,
    },
    /// Loom pattern gate not met.
    PatternGateNotMet {
        needed_patterns: usize,
        current_patterns: usize,
    },
    /// Already at the maximum Ascension level.
    MaxLevelReached,
}

/// Check if the character can Ascend to their next level.
pub fn can_ascend(
    ascension_level: u32,
    prestige_rank: u32,
    deepest_layer: u32,
    completed_patterns: usize,
) -> bool {
    if ascension_level >= super::types::MAX_ASCENSION_LEVEL {
        return false;
    }
    let next = ascension_level + 1;
    let cost = super::types::ascension_cost(next);
    if prestige_rank < cost {
        return false;
    }
    if let Some(gate) = super::types::ascension_deep_gate(next) {
        if deepest_layer < gate {
            return false;
        }
    }
    if let Some(pattern_gate) = super::types::ascension_pattern_gate(next) {
        if completed_patterns < pattern_gate {
            return false;
        }
    }
    true
}

/// Attempt to Ascend the character to the next level.
///
/// Checks PR cost and Deep layer gates. On success, deducts PR and increments ascension_level.
pub fn ascend(
    state: &mut crate::core::game_state::GameState,
    deepest_layer: u32,
    completed_patterns: usize,
) -> AscendResult {
    if state.ascension_level >= super::types::MAX_ASCENSION_LEVEL {
        return AscendResult::MaxLevelReached;
    }

    let next = state.ascension_level + 1;
    let cost = super::types::ascension_cost(next);

    if state.prestige_rank < cost {
        return AscendResult::InsufficientPR {
            needed: cost,
            have: state.prestige_rank,
        };
    }

    if let Some(gate) = super::types::ascension_deep_gate(next) {
        if deepest_layer < gate {
            return AscendResult::DeepGateNotMet {
                needed_layer: gate,
                current_layer: deepest_layer,
            };
        }
    }

    if let Some(pattern_gate) = super::types::ascension_pattern_gate(next) {
        if completed_patterns < pattern_gate {
            return AscendResult::PatternGateNotMet {
                needed_patterns: pattern_gate,
                current_patterns: completed_patterns,
            };
        }
    }

    state.prestige_rank -= cost;
    state.ascension_level = next;
    let multiplier = super::types::ascension_combat_multiplier(next);

    AscendResult::Success {
        new_level: next,
        multiplier,
    }
}
