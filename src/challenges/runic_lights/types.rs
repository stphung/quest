//! Runic Lights challenge data structures.
//!
//! A Lights Out puzzle where toggling a rune flips it and its orthogonal neighbors.

use serde::{Deserialize, Serialize};

/// Difficulty levels controlling grid size and solution depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunicLightsDifficulty {
    Novice,
    Apprentice,
    Journeyman,
    Master,
}

difficulty_enum_impl!(RunicLightsDifficulty);

impl RunicLightsDifficulty {
    /// Grid side length for this difficulty.
    pub fn grid_size(&self) -> usize {
        match self {
            Self::Novice => 3,
            Self::Apprentice => 4,
            Self::Journeyman => 5,
            Self::Master => 6,
        }
    }

    /// Range of toggles used during puzzle generation (min, max inclusive).
    pub fn solution_depth_range(&self) -> (usize, usize) {
        match self {
            Self::Novice => (3, 5),
            Self::Apprentice => (5, 8),
            Self::Journeyman => (8, 12),
            Self::Master => (12, 16),
        }
    }

    /// Par score (upper bound of solution depth range).
    pub fn par(&self) -> u32 {
        match self {
            Self::Novice => 5,
            Self::Apprentice => 8,
            Self::Journeyman => 12,
            Self::Master => 16,
        }
    }

    /// Move limit = 3x par.
    pub fn move_limit(&self) -> u32 {
        self.par() * 3
    }
}

/// Result of a Runic Lights game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunicLightsResult {
    Win,
    Loss,
}

/// UI-agnostic input enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunicLightsInput {
    Up,
    Down,
    Left,
    Right,
    Toggle,
    Forfeit,
    Other,
}

/// Full Runic Lights game state.
#[derive(Debug, Clone)]
pub struct RunicLightsGame {
    pub difficulty: RunicLightsDifficulty,
    /// Board state: true = lit, false = dark.
    pub board: Vec<Vec<bool>>,
    /// Grid side length.
    pub size: usize,
    /// Cursor position (row, col).
    pub cursor: (usize, usize),
    /// Number of moves the player has made.
    pub moves: u32,
    /// Par for the current puzzle.
    pub par: u32,
    /// Move limit (3x par).
    pub move_limit: u32,
    pub game_result: Option<RunicLightsResult>,
    pub forfeit_pending: bool,
}

impl RunicLightsGame {
    pub fn new(difficulty: RunicLightsDifficulty) -> Self {
        let size = difficulty.grid_size();
        Self {
            difficulty,
            board: vec![vec![false; size]; size],
            size,
            cursor: (0, 0),
            moves: 0,
            par: difficulty.par(),
            move_limit: difficulty.move_limit(),
            game_result: None,
            forfeit_pending: false,
        }
    }

    /// Count of lit cells on the board.
    pub fn lit_count(&self) -> usize {
        self.board.iter().flatten().filter(|&&c| c).count()
    }

    /// Total number of cells.
    pub fn total_cells(&self) -> usize {
        self.size * self.size
    }

    /// Move cursor with clamping.
    pub fn move_cursor(&mut self, d_row: i32, d_col: i32) {
        let new_row = (self.cursor.0 as i32 + d_row).clamp(0, self.size as i32 - 1) as usize;
        let new_col = (self.cursor.1 as i32 + d_col).clamp(0, self.size as i32 - 1) as usize;
        self.cursor = (new_row, new_col);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_grid_sizes() {
        assert_eq!(RunicLightsDifficulty::Novice.grid_size(), 3);
        assert_eq!(RunicLightsDifficulty::Apprentice.grid_size(), 4);
        assert_eq!(RunicLightsDifficulty::Journeyman.grid_size(), 5);
        assert_eq!(RunicLightsDifficulty::Master.grid_size(), 6);
    }

    #[test]
    fn test_par_and_move_limit() {
        assert_eq!(RunicLightsDifficulty::Novice.par(), 5);
        assert_eq!(RunicLightsDifficulty::Novice.move_limit(), 15);
        assert_eq!(RunicLightsDifficulty::Master.par(), 16);
        assert_eq!(RunicLightsDifficulty::Master.move_limit(), 48);
    }

    #[test]
    fn test_difficulty_enum_impl() {
        assert_eq!(
            RunicLightsDifficulty::from_index(0),
            RunicLightsDifficulty::Novice
        );
        assert_eq!(
            RunicLightsDifficulty::from_index(3),
            RunicLightsDifficulty::Master
        );
        assert_eq!(
            RunicLightsDifficulty::from_index(99),
            RunicLightsDifficulty::Novice
        );
        assert_eq!(RunicLightsDifficulty::ALL.len(), 4);
    }

    #[test]
    fn test_game_new() {
        let game = RunicLightsGame::new(RunicLightsDifficulty::Novice);
        assert_eq!(game.size, 3);
        assert_eq!(game.board.len(), 3);
        assert_eq!(game.board[0].len(), 3);
        assert!(game.board.iter().flatten().all(|&c| !c));
        assert_eq!(game.cursor, (0, 0));
        assert_eq!(game.moves, 0);
        assert_eq!(game.par, 5);
        assert_eq!(game.move_limit, 15);
        assert!(game.game_result.is_none());
        assert!(!game.forfeit_pending);
    }

    #[test]
    fn test_lit_count() {
        let mut game = RunicLightsGame::new(RunicLightsDifficulty::Novice);
        assert_eq!(game.lit_count(), 0);
        game.board[0][0] = true;
        game.board[1][1] = true;
        assert_eq!(game.lit_count(), 2);
    }

    #[test]
    fn test_move_cursor_clamping() {
        let mut game = RunicLightsGame::new(RunicLightsDifficulty::Novice);
        game.move_cursor(-1, -1);
        assert_eq!(game.cursor, (0, 0));
        game.move_cursor(10, 10);
        assert_eq!(game.cursor, (2, 2));
        game.move_cursor(1, 1);
        assert_eq!(game.cursor, (2, 2));
    }
}
