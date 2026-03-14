//! Shard Fusion challenge data structures.
//!
//! A 2048-style puzzle minigame where the player merges tiles to reach a target value.

use serde::{Deserialize, Serialize};

/// Difficulty levels controlling the target tile value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShardFusionDifficulty {
    Novice,
    Apprentice,
    Journeyman,
    Master,
}

difficulty_enum_impl!(ShardFusionDifficulty);

impl ShardFusionDifficulty {
    /// The tile value the player must reach to win.
    pub fn target_value(&self) -> u32 {
        match self {
            Self::Novice => 512,
            Self::Apprentice => 1024,
            Self::Journeyman => 2048,
            Self::Master => 4096,
        }
    }
}

/// Result of a Shard Fusion game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShardFusionResult {
    Win,
    Loss,
}

/// Animation state for tile sliding and merge flash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShardFusionAnimState {
    /// No animation running; input accepted.
    Idle,
    /// Tiles are sliding. Inner value counts down ticks (starts at SLIDE_TICKS).
    Sliding(u32),
    /// Merged tiles are flashing. Inner value counts down ticks (starts at FLASH_TICKS).
    Flashing(u32),
}

/// Duration of the slide animation in game ticks (100ms each).
pub const SLIDE_TICKS: u32 = 1;
/// Duration of the merge flash animation in game ticks.
pub const FLASH_TICKS: u32 = 1;

/// Records one tile's movement for slide rendering.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct TileMove {
    /// Source cell (row, col).
    pub from: (usize, usize),
    /// Destination cell (row, col).
    pub to: (usize, usize),
    /// The tile value being moved.
    pub value: u32,
}

/// Full Shard Fusion game state.
#[derive(Debug, Clone)]
pub struct ShardFusionGame {
    pub difficulty: ShardFusionDifficulty,
    /// Current board state (0 = empty).
    pub board: [[u32; 4]; 4],
    pub anim_state: ShardFusionAnimState,
    /// Tile movements for slide animation rendering.
    pub slide_moves: Vec<TileMove>,
    /// Cells that just merged, for flash rendering.
    pub merged_cells: Vec<(usize, usize)>,
    pub score: u32,
    pub game_result: Option<ShardFusionResult>,
    pub forfeit_pending: bool,
}

impl ShardFusionGame {
    pub fn new(difficulty: ShardFusionDifficulty) -> Self {
        Self {
            difficulty,
            board: [[0; 4]; 4],
            anim_state: ShardFusionAnimState::Idle,
            slide_moves: Vec::new(),
            merged_cells: Vec::new(),
            score: 0,
            game_result: None,
            forfeit_pending: false,
        }
    }

    /// Returns the highest tile value currently on the board.
    pub fn highest_tile(&self) -> u32 {
        self.board.iter().flatten().copied().max().unwrap_or(0)
    }

    /// Returns the number of empty cells on the board.
    #[allow(dead_code)]
    pub fn empty_count(&self) -> usize {
        self.board.iter().flatten().filter(|&&v| v == 0).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_targets() {
        assert_eq!(ShardFusionDifficulty::Novice.target_value(), 512);
        assert_eq!(ShardFusionDifficulty::Apprentice.target_value(), 1024);
        assert_eq!(ShardFusionDifficulty::Journeyman.target_value(), 2048);
        assert_eq!(ShardFusionDifficulty::Master.target_value(), 4096);
    }

    #[test]
    fn test_difficulty_enum_impl() {
        assert_eq!(
            ShardFusionDifficulty::from_index(0),
            ShardFusionDifficulty::Novice
        );
        assert_eq!(
            ShardFusionDifficulty::from_index(1),
            ShardFusionDifficulty::Apprentice
        );
        assert_eq!(
            ShardFusionDifficulty::from_index(2),
            ShardFusionDifficulty::Journeyman
        );
        assert_eq!(
            ShardFusionDifficulty::from_index(3),
            ShardFusionDifficulty::Master
        );
        assert_eq!(
            ShardFusionDifficulty::from_index(99),
            ShardFusionDifficulty::Novice
        );
        assert_eq!(ShardFusionDifficulty::ALL.len(), 4);
    }

    #[test]
    fn test_game_new() {
        let game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        assert!(game.board.iter().flatten().all(|&v| v == 0));
        assert_eq!(game.anim_state, ShardFusionAnimState::Idle);
        assert!(game.slide_moves.is_empty());
        assert!(game.merged_cells.is_empty());
        assert_eq!(game.score, 0);
        assert!(game.game_result.is_none());
        assert!(!game.forfeit_pending);
    }

    #[test]
    fn test_highest_tile_empty_board() {
        let game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        assert_eq!(game.highest_tile(), 0);
    }

    #[test]
    fn test_highest_tile_with_values() {
        let mut game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        game.board[0][0] = 2;
        game.board[1][1] = 128;
        game.board[3][3] = 64;
        assert_eq!(game.highest_tile(), 128);
    }

    #[test]
    fn test_empty_count() {
        let mut game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        assert_eq!(game.empty_count(), 16);
        game.board[0][0] = 2;
        game.board[0][1] = 4;
        assert_eq!(game.empty_count(), 14);
    }
}
