//! Gomoku (Five in a Row) minigame data structures.
//!
//! 15x15 board, first to get 5+ in a row wins.

use serde::{Deserialize, Serialize};

/// Board size (15x15 standard)
pub const BOARD_SIZE: usize = 15;

/// Player in Gomoku
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Player {
    Human,
    Ai,
}

impl Player {
    pub fn opponent(&self) -> Self {
        match self {
            Player::Human => Player::Ai,
            Player::Ai => Player::Human,
        }
    }
}

/// AI difficulty levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GomokuDifficulty {
    Novice,     // depth 2
    Apprentice, // depth 3
    Journeyman, // depth 4
    Master,     // depth 5
}

difficulty_enum_impl!(GomokuDifficulty);

impl GomokuDifficulty {
    pub fn search_depth(&self) -> i32 {
        match self {
            Self::Novice => 2,
            Self::Apprentice => 3,
            Self::Journeyman => 4,
            Self::Master => 5,
        }
    }
}

/// Game result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GomokuResult {
    Win,
    Loss,
    Draw,
}

/// Main game state
#[derive(Debug, Clone)]
pub struct GomokuGame {
    /// 15x15 board, None = empty
    pub board: [[Option<Player>; BOARD_SIZE]; BOARD_SIZE],
    /// Current cursor position (row, col)
    pub cursor: (usize, usize),
    /// Whose turn it is
    pub current_player: Player,
    /// Difficulty level
    pub difficulty: GomokuDifficulty,
    /// Game result (None if game in progress)
    pub game_result: Option<GomokuResult>,
    /// Is AI currently thinking?
    pub ai_thinking: bool,
    /// Ticks spent thinking (for delayed AI move)
    pub ai_think_ticks: u32,
    /// Move history for display
    pub move_history: Vec<(usize, usize, Player)>,
    /// Last move position for highlighting
    pub last_move: Option<(usize, usize)>,
    /// Forfeit confirmation pending
    pub forfeit_pending: bool,
    /// Winning line positions (for highlighting on game over)
    pub winning_line: Option<Vec<(usize, usize)>>,
}

impl GomokuGame {
    pub fn new(difficulty: GomokuDifficulty) -> Self {
        Self {
            board: [[None; BOARD_SIZE]; BOARD_SIZE],
            cursor: (BOARD_SIZE / 2, BOARD_SIZE / 2), // Center
            current_player: Player::Human,            // Human plays first
            difficulty,
            game_result: None,
            ai_thinking: false,
            ai_think_ticks: 0,
            move_history: Vec::new(),
            last_move: None,
            forfeit_pending: false,
            winning_line: None,
        }
    }

    /// Check if a position is valid and empty
    pub fn is_valid_move(&self, row: usize, col: usize) -> bool {
        row < BOARD_SIZE && col < BOARD_SIZE && self.board[row][col].is_none()
    }

    /// Place a stone at the given position
    pub fn place_stone(&mut self, row: usize, col: usize) -> bool {
        if !self.is_valid_move(row, col) || self.game_result.is_some() {
            return false;
        }
        self.board[row][col] = Some(self.current_player);
        self.move_history.push((row, col, self.current_player));
        self.last_move = Some((row, col));
        true
    }

    /// Switch to the other player's turn
    pub fn switch_player(&mut self) {
        self.current_player = self.current_player.opponent();
    }

    /// Move cursor in a direction
    pub fn move_cursor(&mut self, d_row: i32, d_col: i32) {
        let new_row = (self.cursor.0 as i32 + d_row).clamp(0, BOARD_SIZE as i32 - 1) as usize;
        let new_col = (self.cursor.1 as i32 + d_col).clamp(0, BOARD_SIZE as i32 - 1) as usize;
        self.cursor = (new_row, new_col);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game() {
        let game = GomokuGame::new(GomokuDifficulty::Novice);
        assert_eq!(game.cursor, (7, 7));
        assert_eq!(game.current_player, Player::Human);
        assert!(game.game_result.is_none());
    }

    #[test]
    fn test_place_stone() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        assert!(game.place_stone(7, 7));
        assert_eq!(game.board[7][7], Some(Player::Human));
        assert!(!game.place_stone(7, 7)); // Can't place on occupied
    }

    #[test]
    fn test_difficulty_depths() {
        assert_eq!(GomokuDifficulty::Novice.search_depth(), 2);
        assert_eq!(GomokuDifficulty::Apprentice.search_depth(), 3);
        assert_eq!(GomokuDifficulty::Journeyman.search_depth(), 4);
        assert_eq!(GomokuDifficulty::Master.search_depth(), 5);
    }

    #[test]
    fn test_difficulty_rewards() {
        use crate::challenges::menu::DifficultyInfo;

        // Novice/Apprentice: XP only
        let novice = GomokuDifficulty::Novice.reward();
        assert_eq!(novice.xp_percent, 75);
        assert_eq!(novice.prestige_ranks, 0);

        let apprentice = GomokuDifficulty::Apprentice.reward();
        assert_eq!(apprentice.xp_percent, 100);
        assert_eq!(apprentice.prestige_ranks, 0);

        // Journeyman/Master: XP + Prestige
        let journeyman = GomokuDifficulty::Journeyman.reward();
        assert_eq!(journeyman.xp_percent, 50);
        assert_eq!(journeyman.prestige_ranks, 1);

        let master = GomokuDifficulty::Master.reward();
        assert_eq!(master.xp_percent, 100);
        assert_eq!(master.prestige_ranks, 2);
    }

    #[test]
    fn test_move_cursor() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        game.move_cursor(-1, 0); // Up
        assert_eq!(game.cursor, (6, 7));
        game.cursor = (0, 0);
        game.move_cursor(-1, -1); // Should clamp
        assert_eq!(game.cursor, (0, 0));
    }

    #[test]
    fn test_player_opponent() {
        assert_eq!(Player::Human.opponent(), Player::Ai);
        assert_eq!(Player::Ai.opponent(), Player::Human);
    }
}
