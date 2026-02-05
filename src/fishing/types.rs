//! Fishing system data structures and state management.
//!
//! The fishing system provides a separate progression track where players can
//! catch fish of varying rarities to earn XP and occasionally find items.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use crate::items::Item;

/// Rarity tiers for caught fish, determining XP rewards and catch difficulty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FishRarity {
    Common = 0,
    Uncommon = 1,
    Rare = 2,
    Epic = 3,
    Legendary = 4,
}

/// Represents a single fish that has been caught.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaughtFish {
    pub name: String,
    pub rarity: FishRarity,
    pub xp_reward: u32,
}

/// Current phase of the fishing process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FishingPhase {
    /// Casting the line (1s delay)
    Casting,
    /// Waiting for a bite (2-4s random)
    Waiting,
    /// Fish is biting, reeling in (1-2s)
    Reeling,
}

/// Represents an active fishing session at a particular spot.
#[derive(Debug, Clone)]
pub struct FishingSession {
    pub spot_name: String,
    pub total_fish: u32,
    pub fish_caught: Vec<CaughtFish>,
    pub items_found: Vec<Item>,
    /// Ticks remaining in current phase
    pub ticks_remaining: u32,
    /// Current fishing phase
    pub phase: FishingPhase,
}

/// Persistent fishing state that is saved with the character.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FishingState {
    pub rank: u32,
    pub total_fish_caught: u32,
    pub fish_toward_next_rank: u32,
    pub legendary_catches: u32,
}

impl Default for FishingState {
    fn default() -> Self {
        Self {
            rank: 1,
            total_fish_caught: 0,
            fish_toward_next_rank: 0,
            legendary_catches: 0,
        }
    }
}

/// All 30 fishing rank names organized by tier.
/// - Novice tier (1-5): Learning the basics
/// - Apprentice tier (6-10): Developing skills
/// - Journeyman tier (11-15): Competent angler
/// - Expert tier (16-20): Mastering the depths
/// - Master tier (21-25): Legendary pursuits
/// - Grandmaster tier (26-30): Ocean mastery
pub const RANK_NAMES: [&str; 30] = [
    // Novice tier (1-5)
    "Bait Handler",
    "Line Tangler",
    "Nibble Watcher",
    "Hook Setter",
    "Line Caster",
    // Apprentice tier (6-10)
    "Pond Fisher",
    "River Wader",
    "Lake Lounger",
    "Stream Reader",
    "Net Weaver",
    // Journeyman tier (11-15)
    "Tide Reader",
    "Reef Walker",
    "Shell Seeker",
    "Wave Rider",
    "Current Master",
    // Expert tier (16-20)
    "Deep Diver",
    "Trench Explorer",
    "Abyssal Angler",
    "Pressure Breaker",
    "Storm Fisher",
    // Master tier (21-25)
    "Legend Hunter",
    "Myth Seeker",
    "Leviathan Lurer",
    "Serpent Tamer",
    "Kraken Caller",
    // Grandmaster tier (26-30)
    "Ocean Sage",
    "Tidebinder",
    "Depthless One",
    "Sea Eternal",
    "Poseidon's Chosen",
];

impl FishingState {
    /// Returns the display name for the current fishing rank.
    pub fn rank_name(&self) -> &'static str {
        let index = (self.rank.saturating_sub(1) as usize).min(RANK_NAMES.len() - 1);
        RANK_NAMES[index]
    }

    /// Returns the number of fish required to advance from the given rank.
    ///
    /// Fish requirements by tier:
    /// - Novice (1-5): 100 fish per rank = 500 total
    /// - Apprentice (6-10): 200 fish per rank = 1000 total
    /// - Journeyman (11-15): 400 fish per rank = 2000 total
    /// - Expert (16-20): 800 fish per rank = 4000 total
    /// - Master (21-25): 1500 fish per rank = 7500 total
    /// - Grandmaster (26-30): 2000 fish per rank = 10000 total
    pub fn fish_required_for_rank(rank: u32) -> u32 {
        match rank {
            1..=5 => 100,
            6..=10 => 200,
            11..=15 => 400,
            16..=20 => 800,
            21..=25 => 1500,
            26..=30 => 2000,
            _ => 2000, // Max tier requirement for ranks beyond 30
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fishing_state_default() {
        let state = FishingState::default();
        assert_eq!(state.rank, 1);
        assert_eq!(state.total_fish_caught, 0);
        assert_eq!(state.fish_toward_next_rank, 0);
        assert_eq!(state.legendary_catches, 0);
    }

    #[test]
    fn test_rank_name_returns_correct_names() {
        // Test first rank (Novice tier)
        let state = FishingState {
            rank: 1,
            ..Default::default()
        };
        assert_eq!(state.rank_name(), "Bait Handler");

        // Test middle of Novice tier
        let state = FishingState {
            rank: 3,
            ..Default::default()
        };
        assert_eq!(state.rank_name(), "Nibble Watcher");

        // Test last Novice rank
        let state = FishingState {
            rank: 5,
            ..Default::default()
        };
        assert_eq!(state.rank_name(), "Line Caster");

        // Test first Apprentice rank
        let state = FishingState {
            rank: 6,
            ..Default::default()
        };
        assert_eq!(state.rank_name(), "Pond Fisher");

        // Test Journeyman tier
        let state = FishingState {
            rank: 15,
            ..Default::default()
        };
        assert_eq!(state.rank_name(), "Current Master");

        // Test Expert tier
        let state = FishingState {
            rank: 20,
            ..Default::default()
        };
        assert_eq!(state.rank_name(), "Storm Fisher");

        // Test Master tier
        let state = FishingState {
            rank: 25,
            ..Default::default()
        };
        assert_eq!(state.rank_name(), "Kraken Caller");

        // Test max rank (Grandmaster tier)
        let state = FishingState {
            rank: 30,
            ..Default::default()
        };
        assert_eq!(state.rank_name(), "Poseidon's Chosen");
    }

    #[test]
    fn test_rank_name_clamps_to_valid_range() {
        // Test rank 0 (should clamp to first rank)
        let state = FishingState {
            rank: 0,
            ..Default::default()
        };
        assert_eq!(state.rank_name(), "Bait Handler");

        // Test rank beyond max (should clamp to last rank)
        let state = FishingState {
            rank: 100,
            ..Default::default()
        };
        assert_eq!(state.rank_name(), "Poseidon's Chosen");
    }

    #[test]
    fn test_fish_required_for_rank() {
        // Novice tier (1-5): 100 fish per rank
        assert_eq!(FishingState::fish_required_for_rank(1), 100);
        assert_eq!(FishingState::fish_required_for_rank(5), 100);

        // Apprentice tier (6-10): 200 fish per rank
        assert_eq!(FishingState::fish_required_for_rank(6), 200);
        assert_eq!(FishingState::fish_required_for_rank(10), 200);

        // Journeyman tier (11-15): 400 fish per rank
        assert_eq!(FishingState::fish_required_for_rank(11), 400);
        assert_eq!(FishingState::fish_required_for_rank(15), 400);

        // Expert tier (16-20): 800 fish per rank
        assert_eq!(FishingState::fish_required_for_rank(16), 800);
        assert_eq!(FishingState::fish_required_for_rank(20), 800);

        // Master tier (21-25): 1500 fish per rank
        assert_eq!(FishingState::fish_required_for_rank(21), 1500);
        assert_eq!(FishingState::fish_required_for_rank(25), 1500);

        // Grandmaster tier (26-30): 2000 fish per rank
        assert_eq!(FishingState::fish_required_for_rank(26), 2000);
        assert_eq!(FishingState::fish_required_for_rank(30), 2000);

        // Beyond max rank
        assert_eq!(FishingState::fish_required_for_rank(31), 2000);
        assert_eq!(FishingState::fish_required_for_rank(100), 2000);
    }

    #[test]
    fn test_fish_rarity_ordering() {
        assert!(FishRarity::Common < FishRarity::Uncommon);
        assert!(FishRarity::Uncommon < FishRarity::Rare);
        assert!(FishRarity::Rare < FishRarity::Epic);
        assert!(FishRarity::Epic < FishRarity::Legendary);
    }

    #[test]
    fn test_caught_fish_creation() {
        let fish = CaughtFish {
            name: "Golden Trout".to_string(),
            rarity: FishRarity::Rare,
            xp_reward: 150,
        };
        assert_eq!(fish.name, "Golden Trout");
        assert_eq!(fish.rarity, FishRarity::Rare);
        assert_eq!(fish.xp_reward, 150);
    }

    #[test]
    fn test_fishing_session_creation() {
        let session = FishingSession {
            spot_name: "Moonlit Lake".to_string(),
            total_fish: 5,
            fish_caught: vec![
                CaughtFish {
                    name: "Perch".to_string(),
                    rarity: FishRarity::Common,
                    xp_reward: 10,
                },
                CaughtFish {
                    name: "Bass".to_string(),
                    rarity: FishRarity::Uncommon,
                    xp_reward: 25,
                },
            ],
            items_found: vec![],
            ticks_remaining: 15,
            phase: FishingPhase::Waiting,
        };
        assert_eq!(session.spot_name, "Moonlit Lake");
        assert_eq!(session.total_fish, 5);
        assert_eq!(session.fish_caught.len(), 2);
        assert!(session.items_found.is_empty());
        assert_eq!(session.ticks_remaining, 15);
        assert_eq!(session.phase, FishingPhase::Waiting);
    }
}
