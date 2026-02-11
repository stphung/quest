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

difficulty_enum_impl!(GoDifficulty);

impl GoDifficulty {
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
}

/// Result of a completed Go game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoResult {
    Win,
    Loss,
    Draw,
}

/// Main Go game state
#[derive(Debug, Clone)]
pub struct GoGame {
    /// 9x9 board, None = empty intersection
    pub board: [[Option<Stone>; BOARD_SIZE]; BOARD_SIZE],
    /// Current player's turn
    pub current_player: Stone,
    /// Ko point - illegal to play here this turn (prevents infinite capture loops)
    pub ko_point: Option<(usize, usize)>,
    /// Stones captured by Black (White's prisoners)
    pub captured_by_black: u32,
    /// Stones captured by White (Black's prisoners)
    pub captured_by_white: u32,
    /// Count of consecutive passes (2 = game over)
    pub consecutive_passes: u8,
    /// Cursor position (row, col) for UI
    pub cursor: (usize, usize),
    /// Difficulty level
    pub difficulty: GoDifficulty,
    /// Game result (None if in progress)
    pub game_result: Option<GoResult>,
    /// Is AI currently thinking?
    pub ai_thinking: bool,
    /// Ticks spent thinking (for delayed AI move)
    pub ai_think_ticks: u32,
    /// Last move for highlighting
    pub last_move: Option<GoMove>,
    /// Forfeit confirmation pending
    pub forfeit_pending: bool,
}

impl GoGame {
    pub fn new(difficulty: GoDifficulty) -> Self {
        Self {
            board: [[None; BOARD_SIZE]; BOARD_SIZE],
            current_player: Stone::Black, // Black plays first in Go
            ko_point: None,
            captured_by_black: 0,
            captured_by_white: 0,
            consecutive_passes: 0,
            cursor: (BOARD_SIZE / 2, BOARD_SIZE / 2), // Center (4, 4)
            difficulty,
            game_result: None,
            ai_thinking: false,
            ai_think_ticks: 0,
            last_move: None,
            forfeit_pending: false,
        }
    }

    /// Move cursor in a direction
    pub fn move_cursor(&mut self, d_row: i32, d_col: i32) {
        let new_row = (self.cursor.0 as i32 + d_row).clamp(0, BOARD_SIZE as i32 - 1) as usize;
        let new_col = (self.cursor.1 as i32 + d_col).clamp(0, BOARD_SIZE as i32 - 1) as usize;
        self.cursor = (new_row, new_col);
    }

    /// Check if a position is empty
    pub fn is_empty(&self, row: usize, col: usize) -> bool {
        row < BOARD_SIZE && col < BOARD_SIZE && self.board[row][col].is_none()
    }

    /// Switch to the other player's turn
    pub fn switch_player(&mut self) {
        self.current_player = self.current_player.opponent();
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

    #[test]
    fn test_new_game() {
        let game = GoGame::new(GoDifficulty::Novice);
        assert_eq!(game.cursor, (4, 4)); // Center of 9x9
        assert_eq!(game.current_player, Stone::Black);
        assert!(game.game_result.is_none());
        assert_eq!(game.consecutive_passes, 0);
        assert!(game.ko_point.is_none());
    }

    #[test]
    fn test_move_cursor() {
        let mut game = GoGame::new(GoDifficulty::Novice);
        game.move_cursor(-1, 0); // Up
        assert_eq!(game.cursor, (3, 4));
        game.cursor = (0, 0);
        game.move_cursor(-1, -1); // Should clamp
        assert_eq!(game.cursor, (0, 0));
        game.cursor = (8, 8);
        game.move_cursor(1, 1); // Should clamp
        assert_eq!(game.cursor, (8, 8));
    }

    #[test]
    fn test_is_empty() {
        let mut game = GoGame::new(GoDifficulty::Novice);
        assert!(game.is_empty(4, 4));
        game.board[4][4] = Some(Stone::Black);
        assert!(!game.is_empty(4, 4));
    }

    #[test]
    fn test_switch_player() {
        let mut game = GoGame::new(GoDifficulty::Novice);
        assert_eq!(game.current_player, Stone::Black);
        game.switch_player();
        assert_eq!(game.current_player, Stone::White);
        game.switch_player();
        assert_eq!(game.current_player, Stone::Black);
    }
}
