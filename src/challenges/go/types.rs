//! Go (Territory Control) minigame data structures.
//!
//! 9x9 board, players place stones to surround territory.

use crate::challenges::{ChallengeDifficulty, ChallengeResult};
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
    pub difficulty: ChallengeDifficulty,
    /// Game result (None if in progress)
    pub game_result: Option<ChallengeResult>,
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
    pub fn new(difficulty: ChallengeDifficulty) -> Self {
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

    /// MCTS simulation count based on difficulty
    pub fn simulation_count(&self) -> u32 {
        match self.difficulty {
            ChallengeDifficulty::Novice => 500,
            ChallengeDifficulty::Apprentice => 2_000,
            ChallengeDifficulty::Journeyman => 8_000,
            ChallengeDifficulty::Master => 20_000,
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
    fn test_simulation_count() {
        let game_novice = GoGame::new(ChallengeDifficulty::Novice);
        assert_eq!(game_novice.simulation_count(), 500);
        let game_apprentice = GoGame::new(ChallengeDifficulty::Apprentice);
        assert_eq!(game_apprentice.simulation_count(), 2_000);
        let game_journeyman = GoGame::new(ChallengeDifficulty::Journeyman);
        assert_eq!(game_journeyman.simulation_count(), 8_000);
        let game_master = GoGame::new(ChallengeDifficulty::Master);
        assert_eq!(game_master.simulation_count(), 20_000);
    }

    #[test]
    fn test_new_game() {
        let game = GoGame::new(ChallengeDifficulty::Novice);
        assert_eq!(game.cursor, (4, 4)); // Center of 9x9
        assert_eq!(game.current_player, Stone::Black);
        assert!(game.game_result.is_none());
        assert_eq!(game.consecutive_passes, 0);
        assert!(game.ko_point.is_none());
    }

    #[test]
    fn test_move_cursor() {
        let mut game = GoGame::new(ChallengeDifficulty::Novice);
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
        let mut game = GoGame::new(ChallengeDifficulty::Novice);
        assert!(game.is_empty(4, 4));
        game.board[4][4] = Some(Stone::Black);
        assert!(!game.is_empty(4, 4));
    }

    #[test]
    fn test_switch_player() {
        let mut game = GoGame::new(ChallengeDifficulty::Novice);
        assert_eq!(game.current_player, Stone::Black);
        game.switch_player();
        assert_eq!(game.current_player, Stone::White);
        game.switch_player();
        assert_eq!(game.current_player, Stone::Black);
    }
}
