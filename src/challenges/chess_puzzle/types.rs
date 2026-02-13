//! Chess puzzle data structures and state management.

use serde::{Deserialize, Serialize};

/// Difficulty levels for chess puzzles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChessPuzzleDifficulty {
    Novice,     // Mate-in-1 puzzles
    Apprentice, // Simple tactics (forks, pins, skewers)
    Journeyman, // Mate-in-2 puzzles
    Master,     // Complex tactics (sacrifices, back rank mates)
}

difficulty_enum_impl!(ChessPuzzleDifficulty);

impl ChessPuzzleDifficulty {
    /// How many puzzles the player must solve to win at this difficulty.
    pub fn target_score(&self) -> u32 {
        match self {
            Self::Novice => 3,
            Self::Apprentice => 4,
            Self::Journeyman => 3,
            Self::Master => 3,
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

/// A single chess puzzle definition (static data).
pub struct PuzzleDef {
    /// Short display title (e.g., "Back Rank Mate", "Knight Fork")
    pub title: &'static str,
    /// Hint text shown in the info panel
    pub hint: &'static str,
    /// Moves from standard starting position to reach the puzzle position.
    /// Each tuple is (from_rank, from_file, to_rank, to_file) for Move::Piece.
    /// Uses chess-engine Position::new(rank, file) coordinates:
    ///   rank 0-7 = ranks 1-8, file 0-7 = files a-h
    pub setup_moves: &'static [(i32, i32, i32, i32)],
    /// Which color the player plays as
    pub player_is_white: bool,
    /// The expected solution
    pub solution: PuzzleSolution,
}

/// How the puzzle is validated.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum PuzzleSolution {
    /// Player must deliver checkmate in one move.
    /// Any move that results in checkmate is correct.
    MateInOne,

    /// Player must find the specific best move (for tactics: forks, pins, skewers).
    /// Tuple is (from_rank, from_file, to_rank, to_file).
    BestMove(i32, i32, i32, i32),

    /// Player must deliver checkmate in two moves.
    /// move1: player's first move (from_rank, from_file, to_rank, to_file)
    /// After move1, the engine plays the best response automatically.
    /// move2: player's second move that must result in checkmate.
    MateInTwo {
        move1: (i32, i32, i32, i32),
        move2: (i32, i32, i32, i32),
    },
}

/// Current state within a single puzzle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum PuzzleState {
    /// Player is choosing their move
    Solving,
    /// Player found the correct move/sequence
    Correct,
    /// Player made the wrong move
    Wrong,
    /// Waiting for AI response in mate-in-2 (after player's first move)
    WaitingForAI,
}

/// Persistent chess puzzle stats (saved to disk).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChessPuzzleStats {
    pub sessions_played: u32,
    pub sessions_won: u32,
    pub sessions_lost: u32,
    pub puzzles_solved: u32,
    pub puzzles_attempted: u32,
}

/// Result of a completed chess puzzle session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChessPuzzleResult {
    Win,
    Loss,
}

/// Active chess puzzle session (transient, not saved).
#[derive(Debug, Clone)]
pub struct ChessPuzzleGame {
    // Puzzle set
    pub difficulty: ChessPuzzleDifficulty,
    /// Ordered list of puzzle indices into the difficulty's puzzle array.
    pub puzzle_order: Vec<usize>,
    /// Index into puzzle_order (which puzzle we're on)
    pub current_puzzle_index: usize,

    // Scoring
    pub puzzles_solved: u32,
    pub puzzles_attempted: u32,
    /// How many puzzles the player must solve to win
    pub target_score: u32,
    /// Total puzzles in this session
    pub total_puzzles: u32,

    // Board state
    pub board: chess_engine::Board,
    pub player_is_white: bool,

    // Cursor/selection
    pub cursor: (u8, u8),
    pub selected_square: Option<(u8, u8)>,
    pub legal_move_destinations: Vec<(u8, u8)>,

    // Puzzle flow
    pub puzzle_state: PuzzleState,
    /// Ticks remaining for Correct/Wrong feedback display (10 ticks = 1s)
    pub feedback_ticks: u32,
    /// For mate-in-2: tracks which move the player is on (0 = first, 1 = second)
    pub move_number_in_puzzle: u8,
    /// AI thinking state for mate-in-2 intermediate response
    pub ai_thinking: bool,
    pub ai_think_ticks: u32,
    pub ai_think_target: u32,
    pub ai_pending_board: Option<chess_engine::Board>,

    // Game result
    pub game_result: Option<ChessPuzzleResult>,
    pub forfeit_pending: bool,

    // Display
    /// Last move highlight (from, to)
    pub last_move: Option<((u8, u8), (u8, u8))>,
}

impl ChessPuzzleGame {
    pub fn new(difficulty: ChessPuzzleDifficulty) -> Self {
        let puzzles = super::puzzles::get_puzzles(difficulty);
        let total = puzzles.len();
        let target = difficulty.target_score();
        let puzzle_order: Vec<usize> = (0..total).collect();

        Self {
            difficulty,
            puzzle_order,
            current_puzzle_index: 0,
            puzzles_solved: 0,
            puzzles_attempted: 0,
            target_score: target,
            total_puzzles: total as u32,
            board: chess_engine::Board::default(),
            player_is_white: true,
            cursor: (4, 3),
            selected_square: None,
            legal_move_destinations: Vec::new(),
            puzzle_state: PuzzleState::Solving,
            feedback_ticks: 0,
            move_number_in_puzzle: 0,
            ai_thinking: false,
            ai_think_ticks: 0,
            ai_think_target: 0,
            ai_pending_board: None,
            game_result: None,
            forfeit_pending: false,
            last_move: None,
        }
    }

    pub fn move_cursor(&mut self, dx: i8, dy: i8) {
        let new_x = (self.cursor.0 as i8 + dx).clamp(0, 7) as u8;
        let new_y = (self.cursor.1 as i8 + dy).clamp(0, 7) as u8;
        self.cursor = (new_x, new_y);
    }

    /// Get the player's color
    pub fn player_color(&self) -> chess_engine::Color {
        if self.player_is_white {
            chess_engine::Color::White
        } else {
            chess_engine::Color::Black
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_from_index() {
        assert_eq!(
            ChessPuzzleDifficulty::from_index(0),
            ChessPuzzleDifficulty::Novice
        );
        assert_eq!(
            ChessPuzzleDifficulty::from_index(1),
            ChessPuzzleDifficulty::Apprentice
        );
        assert_eq!(
            ChessPuzzleDifficulty::from_index(2),
            ChessPuzzleDifficulty::Journeyman
        );
        assert_eq!(
            ChessPuzzleDifficulty::from_index(3),
            ChessPuzzleDifficulty::Master
        );
        assert_eq!(
            ChessPuzzleDifficulty::from_index(99),
            ChessPuzzleDifficulty::Novice
        );
    }

    #[test]
    fn test_target_scores() {
        assert_eq!(ChessPuzzleDifficulty::Novice.target_score(), 3);
        assert_eq!(ChessPuzzleDifficulty::Apprentice.target_score(), 4);
        assert_eq!(ChessPuzzleDifficulty::Journeyman.target_score(), 3);
        assert_eq!(ChessPuzzleDifficulty::Master.target_score(), 3);
    }

    #[test]
    fn test_reward_prestige() {
        assert_eq!(ChessPuzzleDifficulty::Novice.reward_prestige(), 1);
        assert_eq!(ChessPuzzleDifficulty::Apprentice.reward_prestige(), 2);
        assert_eq!(ChessPuzzleDifficulty::Journeyman.reward_prestige(), 3);
        assert_eq!(ChessPuzzleDifficulty::Master.reward_prestige(), 5);
    }

    #[test]
    fn test_new_game() {
        let game = ChessPuzzleGame::new(ChessPuzzleDifficulty::Novice);
        assert_eq!(game.difficulty, ChessPuzzleDifficulty::Novice);
        assert_eq!(game.current_puzzle_index, 0);
        assert_eq!(game.puzzles_solved, 0);
        assert_eq!(game.puzzles_attempted, 0);
        assert_eq!(game.target_score, 3);
        assert!(game.game_result.is_none());
        assert!(!game.forfeit_pending);
        assert!(!game.ai_thinking);
        assert_eq!(game.puzzle_state, PuzzleState::Solving);
    }

    #[test]
    fn test_cursor_movement() {
        let mut game = ChessPuzzleGame::new(ChessPuzzleDifficulty::Novice);
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
        let mut game = ChessPuzzleGame::new(ChessPuzzleDifficulty::Novice);
        game.cursor = (0, 0);
        game.move_cursor(-1, -1);
        assert_eq!(game.cursor, (0, 0));
        game.cursor = (7, 7);
        game.move_cursor(1, 1);
        assert_eq!(game.cursor, (7, 7));
    }

    #[test]
    fn test_difficulty_rewards_via_trait() {
        use crate::challenges::menu::DifficultyInfo;

        let novice = ChessPuzzleDifficulty::Novice.reward();
        assert_eq!(novice.prestige_ranks, 1);
        assert_eq!(novice.xp_percent, 0);
        assert_eq!(novice.fishing_ranks, 0);

        let master = ChessPuzzleDifficulty::Master.reward();
        assert_eq!(master.prestige_ranks, 5);
    }

    #[test]
    fn test_puzzle_stats_default() {
        let stats = ChessPuzzleStats::default();
        assert_eq!(stats.sessions_played, 0);
        assert_eq!(stats.sessions_won, 0);
        assert_eq!(stats.puzzles_solved, 0);
    }
}
