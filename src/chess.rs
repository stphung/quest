//! Chess minigame data structures and state management.

use serde::{Deserialize, Serialize};

/// AI difficulty levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChessDifficulty {
    Novice,     // 50% random moves, ~500 ELO
    Apprentice, // 1-ply search, ~800 ELO
    Journeyman, // 2-ply search, ~1100 ELO
    Master,     // 3-ply search, ~1350 ELO
}

impl ChessDifficulty {
    pub const ALL: [ChessDifficulty; 4] = [
        ChessDifficulty::Novice,
        ChessDifficulty::Apprentice,
        ChessDifficulty::Journeyman,
        ChessDifficulty::Master,
    ];

    pub fn from_index(index: usize) -> Self {
        Self::ALL.get(index).copied().unwrap_or(ChessDifficulty::Novice)
    }

    pub fn search_depth(&self) -> i32 {
        match self {
            Self::Novice => 1,
            Self::Apprentice => 1,
            Self::Journeyman => 2,
            Self::Master => 3,
        }
    }

    pub fn random_move_chance(&self) -> f64 {
        match self {
            Self::Novice => 0.5,
            _ => 0.0,
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

    pub fn estimated_elo(&self) -> u32 {
        match self {
            Self::Novice => 500,
            Self::Apprentice => 800,
            Self::Journeyman => 1100,
            Self::Master => 1350,
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

/// Persistent chess stats (saved to disk)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChessStats {
    pub games_played: u32,
    pub games_won: u32,
    pub games_lost: u32,
    pub games_drawn: u32,
    pub prestige_earned: u32,
}

/// Result of a completed chess game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChessResult {
    Win,
    Loss,
    Draw,
    Forfeit,
}

/// Active chess game session (transient, not saved)
#[derive(Debug, Clone)]
pub struct ChessGame {
    pub board: chess_engine::Board,
    pub difficulty: ChessDifficulty,
    pub cursor: (u8, u8),
    pub selected_square: Option<(u8, u8)>,
    pub game_result: Option<ChessResult>,
    pub forfeit_pending: bool,
    pub ai_thinking: bool,
    pub ai_think_ticks: u32,
    pub ai_think_target: u32,
    pub ai_pending_board: Option<chess_engine::Board>,
    pub player_is_white: bool,
}

impl ChessGame {
    pub fn new(difficulty: ChessDifficulty) -> Self {
        Self {
            board: chess_engine::Board::default(),
            difficulty,
            cursor: (4, 1), // e2
            selected_square: None,
            game_result: None,
            forfeit_pending: false,
            ai_thinking: false,
            ai_think_ticks: 0,
            ai_think_target: 0,
            ai_pending_board: None,
            player_is_white: true,
        }
    }

    pub fn move_cursor(&mut self, dx: i8, dy: i8) {
        let new_x = (self.cursor.0 as i8 + dx).clamp(0, 7) as u8;
        let new_y = (self.cursor.1 as i8 + dy).clamp(0, 7) as u8;
        self.cursor = (new_x, new_y);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_from_index() {
        assert_eq!(ChessDifficulty::from_index(0), ChessDifficulty::Novice);
        assert_eq!(ChessDifficulty::from_index(1), ChessDifficulty::Apprentice);
        assert_eq!(ChessDifficulty::from_index(2), ChessDifficulty::Journeyman);
        assert_eq!(ChessDifficulty::from_index(3), ChessDifficulty::Master);
        assert_eq!(ChessDifficulty::from_index(99), ChessDifficulty::Novice);
    }

    #[test]
    fn test_difficulty_properties() {
        assert_eq!(ChessDifficulty::Novice.random_move_chance(), 0.5);
        assert_eq!(ChessDifficulty::Apprentice.random_move_chance(), 0.0);
        assert_eq!(ChessDifficulty::Novice.reward_prestige(), 1);
        assert_eq!(ChessDifficulty::Master.reward_prestige(), 5);
        assert_eq!(ChessDifficulty::Novice.estimated_elo(), 500);
        assert_eq!(ChessDifficulty::Master.estimated_elo(), 1350);
    }

    #[test]
    fn test_chess_game_new() {
        let game = ChessGame::new(ChessDifficulty::Journeyman);
        assert_eq!(game.difficulty, ChessDifficulty::Journeyman);
        assert_eq!(game.cursor, (4, 1));
        assert!(game.selected_square.is_none());
        assert!(game.game_result.is_none());
        assert!(!game.ai_thinking);
    }

    #[test]
    fn test_cursor_movement() {
        let mut game = ChessGame::new(ChessDifficulty::Novice);
        game.cursor = (3, 3);
        game.move_cursor(1, 0);
        assert_eq!(game.cursor, (4, 3));
        game.move_cursor(0, 1);
        assert_eq!(game.cursor, (4, 4));
        game.move_cursor(-1, -1);
        assert_eq!(game.cursor, (3, 3));
    }

    #[test]
    fn test_cursor_bounds() {
        let mut game = ChessGame::new(ChessDifficulty::Novice);
        game.cursor = (0, 0);
        game.move_cursor(-1, -1);
        assert_eq!(game.cursor, (0, 0));
        game.cursor = (7, 7);
        game.move_cursor(1, 1);
        assert_eq!(game.cursor, (7, 7));
    }

    #[test]
    fn test_chess_stats_default() {
        let stats = ChessStats::default();
        assert_eq!(stats.games_played, 0);
        assert_eq!(stats.games_won, 0);
        assert_eq!(stats.prestige_earned, 0);
    }
}
