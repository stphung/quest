//! Go (Territory Control) minigame data structures.
//!
//! 9x9 board, players place stones to surround territory.

use serde::{Deserialize, Serialize};

/// Board size (9x9)
pub const BOARD_SIZE: usize = 9;

/// Stone color
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stone {
    Black,
    White,
}

impl Stone {
    pub fn opponent(&self) -> Self {
        match self {
            Stone::Black => Stone::White,
            Stone::White => Stone::Black,
        }
    }
}

/// A move in Go
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoMove {
    Place(usize, usize),
    Pass,
}

/// AI difficulty levels (based on MCTS simulation count)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoDifficulty {
    Novice,     // 500 simulations
    Apprentice, // 2,000 simulations
    Journeyman, // 8,000 simulations
    Master,     // 20,000 simulations
}

impl GoDifficulty {
    pub const ALL: [GoDifficulty; 4] = [
        GoDifficulty::Novice,
        GoDifficulty::Apprentice,
        GoDifficulty::Journeyman,
        GoDifficulty::Master,
    ];

    pub fn from_index(index: usize) -> Self {
        Self::ALL
            .get(index)
            .copied()
            .unwrap_or(GoDifficulty::Novice)
    }

    pub fn simulation_count(&self) -> u32 {
        match self {
            Self::Novice => 500,
            Self::Apprentice => 2_000,
            Self::Journeyman => 8_000,
            Self::Master => 20_000,
        }
    }

    pub fn reward_prestige(&self) -> u32 {
        match self {
            Self::Novice => 1,
            Self::Apprentice => 2,
            Self::Journeyman => 3,
            Self::Master => 5,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Novice => "Novice",
            Self::Apprentice => "Apprentice",
            Self::Journeyman => "Journeyman",
            Self::Master => "Master",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stone_opponent() {
        assert_eq!(Stone::Black.opponent(), Stone::White);
        assert_eq!(Stone::White.opponent(), Stone::Black);
    }

    #[test]
    fn test_difficulty_from_index() {
        assert_eq!(GoDifficulty::from_index(0), GoDifficulty::Novice);
        assert_eq!(GoDifficulty::from_index(3), GoDifficulty::Master);
        assert_eq!(GoDifficulty::from_index(99), GoDifficulty::Novice);
    }

    #[test]
    fn test_difficulty_simulation_count() {
        assert_eq!(GoDifficulty::Novice.simulation_count(), 500);
        assert_eq!(GoDifficulty::Apprentice.simulation_count(), 2_000);
        assert_eq!(GoDifficulty::Journeyman.simulation_count(), 8_000);
        assert_eq!(GoDifficulty::Master.simulation_count(), 20_000);
    }

    #[test]
    fn test_difficulty_reward_prestige() {
        assert_eq!(GoDifficulty::Novice.reward_prestige(), 1);
        assert_eq!(GoDifficulty::Apprentice.reward_prestige(), 2);
        assert_eq!(GoDifficulty::Journeyman.reward_prestige(), 3);
        assert_eq!(GoDifficulty::Master.reward_prestige(), 5);
    }
}
