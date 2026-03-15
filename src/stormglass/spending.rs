//! Pure functions for Stormglass spending calculations.

use crate::challenges::menu::ChallengeType;
use rand::seq::SliceRandom;
use rand::Rng;

use super::types::*;

/// Returns the (ticks, cost, label) for a Chrono Surge option, or None if out of range.
pub fn chrono_surge_cost(option_index: usize) -> Option<(u64, u64, &'static str)> {
    CHRONO_SURGE_OPTIONS.get(option_index).copied()
}

/// All challenge types available for Invoke Challenge.
const TRIAL_CHALLENGE_TYPES: [ChallengeType; 11] = [
    ChallengeType::Chess,
    ChallengeType::Morris,
    ChallengeType::Gomoku,
    ChallengeType::Minesweeper,
    ChallengeType::Rune,
    ChallengeType::Go,
    ChallengeType::FlappyBird,
    ChallengeType::Snake,
    ChallengeType::Jezzball,
    ChallengeType::RunicShift,
    ChallengeType::ShardFusion,
];

/// Number of trial options presented when invoking a trial.
const TRIAL_OPTION_COUNT: usize = 3;

/// Generate 3 unique trial options with different challenge types.
/// Excludes any challenge types already pending in the player's challenge menu.
pub fn generate_trial_options<R: Rng>(rng: &mut R, exclude: &[ChallengeType]) -> Vec<TrialOption> {
    let mut types: Vec<ChallengeType> = TRIAL_CHALLENGE_TYPES
        .iter()
        .filter(|ct| !exclude.contains(ct))
        .cloned()
        .collect();
    types.shuffle(rng);

    types
        .into_iter()
        .take(TRIAL_OPTION_COUNT)
        .map(|ct| {
            let display_name = challenge_type_name(&ct).to_string();
            TrialOption {
                challenge_type: ct,
                display_name,
            }
        })
        .collect()
}

/// Check if the player can purchase a Storm Lure.
/// Requirements: has enough SG, not already active, fishing rank >= 40.
#[allow(dead_code)]
pub fn can_purchase_storm_lure(stormglass: u64, lure_active: bool, fishing_rank: u32) -> bool {
    stormglass >= STORM_LURE_COST && !lure_active && fishing_rank >= 40
}

/// Full display name for a challenge type (matches challenge menu titles).
fn challenge_type_name(ct: &ChallengeType) -> &'static str {
    match ct {
        ChallengeType::Chess => "Chess: The Hooded Challenger",
        ChallengeType::Morris => "Morris: The Millkeeper's Game",
        ChallengeType::Gomoku => "Gomoku: Five Stones",
        ChallengeType::Minesweeper => "Minesweeper: Trap Detection",
        ChallengeType::Rune => "Rune Deciphering: Ancient Tablet",
        ChallengeType::Go => "Go: Territory Control",
        ChallengeType::FlappyBird => "Skyward Gauntlet",
        ChallengeType::Snake => "Serpent's Path",
        ChallengeType::Jezzball => "Containment Breach",
        ChallengeType::RunicShift => "Sigil Surge",
        ChallengeType::Sudoku => "Sigil Matrix: Arcane Grid",
        ChallengeType::ShardFusion => "Shard Fusion",
    }
}
