//! Pure functions for Stormglass spending calculations.

use crate::challenges::menu::ChallengeType;
use rand::seq::SliceRandom;
use rand::Rng;

use super::types::*;

/// Calculate the Stormglass cost to purchase a prestige rank.
/// Formula: floor(500 * (1 + 0.5 * rank))
/// Linear scaling — each rank costs 250 SG more than the last.
pub fn prestige_rank_cost(current_rank: u32) -> u64 {
    let rank = current_rank as f64;
    (PRESTIGE_RANK_BASE_COST * (1.0 + 0.5 * rank)).floor() as u64
}

/// Returns the (ticks, cost, label) for a Chrono Surge option, or None if out of range.
pub fn chrono_surge_cost(option_index: usize) -> Option<(u64, u64, &'static str)> {
    CHRONO_SURGE_OPTIONS.get(option_index).copied()
}

/// All challenge types available for Invoke Trial.
const TRIAL_CHALLENGE_TYPES: [ChallengeType; 10] = [
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
    }
}
