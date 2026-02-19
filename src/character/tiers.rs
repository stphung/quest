//! Prestige tier definitions, names, and level requirements.

use crate::core::constants::*;

/// Represents a prestige tier with its properties
#[derive(Debug, Clone)]
pub struct PrestigeTier {
    #[allow(dead_code)]
    pub rank: u32,
    pub name: &'static str,
    pub required_level: u32,
    pub multiplier: f64,
}

/// Gets the name for a prestige rank
fn get_prestige_name(rank: u32) -> &'static str {
    match rank {
        0 => "None",
        // Metals (1-4)
        1 => "Bronze",
        2 => "Silver",
        3 => "Gold",
        4 => "Platinum",
        // Gems (5-9)
        5 => "Diamond",
        6 => "Emerald",
        7 => "Sapphire",
        8 => "Ruby",
        9 => "Obsidian",
        // Cosmic (10-14)
        10 => "Celestial",
        11 => "Astral",
        12 => "Cosmic",
        13 => "Stellar",
        14 => "Galactic",
        // Divine (15-19)
        15 => "Transcendent",
        16 => "Divine",
        17 => "Exalted",
        18 => "Mythic",
        19 => "Legendary",
        // Eternal (20+)
        _ => "Eternal",
    }
}

/// Gets the prestige tier for a given rank
///
/// # Arguments
/// * `rank` - The prestige rank
///
/// # Returns
/// The PrestigeTier with name, required level, and multiplier
///
/// # Multiplier Formula
/// Uses diminishing returns: `1 + 0.5 * rank^0.7`
///
/// This provides:
/// - Strong early boost (+50% at P1)
/// - Tapering gains to prevent late-game trivialization
/// - Cycles get progressively longer, creating the "wall" feeling
///
/// See docs/plans/2026-02-03-prestige-multiplier-rebalance.md for details.
pub fn get_prestige_tier(rank: u32) -> PrestigeTier {
    // Diminishing returns formula: 1 + BASE_FACTOR * rank^EXPONENT
    // P1: 1.5x, P5: 2.5x, P10: 3.5x, P20: 5.1x, P30: 6.4x
    let multiplier = 1.0 + PRESTIGE_MULT_BASE_FACTOR * (rank as f64).powf(PRESTIGE_MULT_EXPONENT);

    let required_level = match rank {
        0 => 0,
        1 => 10,
        2 => 25,
        3 => 50,
        4 => 65,
        5 => 80,
        6 => 90,
        7 => 100,
        8 => 110,
        9 => 120,
        10 => 130,
        11 => 140,
        12 => 150,
        13 => 160,
        14 => 170,
        15 => 180,
        16 => 190,
        17 => 200,
        18 => 210,
        19 => PRESTIGE_HIGH_RANK_BASE_LEVEL,
        // 20+: continues at +PRESTIGE_HIGH_RANK_LEVEL_STEP per rank
        _ => {
            PRESTIGE_HIGH_RANK_BASE_LEVEL
                + (rank - PRESTIGE_HIGH_RANK_THRESHOLD) * PRESTIGE_HIGH_RANK_LEVEL_STEP
        }
    };

    PrestigeTier {
        rank,
        name: get_prestige_name(rank),
        required_level,
        multiplier,
    }
}

/// Gets the next prestige tier based on current rank
///
/// # Arguments
/// * `current_rank` - The player's current prestige rank
///
/// # Returns
/// The PrestigeTier for the next rank
pub fn get_next_prestige_tier(current_rank: u32) -> PrestigeTier {
    get_prestige_tier(current_rank + 1)
}

/// Gets the adventurer rank based on average level
///
/// # Arguments
/// * `avg_level` - The average level across all stats
///
/// # Returns
/// A string describing the adventurer's rank
pub fn get_adventurer_rank(avg_level: u32) -> &'static str {
    match avg_level {
        0..=9 => "Novice",
        10..=24 => "Adept",
        25..=49 => "Master",
        50..=74 => "Grand Master",
        75..=99 => "Legend",
        _ => "Mythic",
    }
}
