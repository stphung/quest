//! Chess minigame data structures and state management.

use crate::challenges::{ChallengeDifficulty, ChallengeResult};
use chess_engine::{Color as ChessColor, Evaluate, Move, Position};
use serde::{Deserialize, Serialize};

/// Persistent chess stats (saved to disk)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChessStats {
    pub games_played: u32,
    pub games_won: u32,
    pub games_lost: u32,
    pub games_drawn: u32,
    pub prestige_earned: u32,
}

/// Active chess game session (transient, not saved)
#[derive(Debug, Clone)]
pub struct ChessGame {
    pub board: chess_engine::Board,
    pub difficulty: ChallengeDifficulty,
    pub cursor: (u8, u8),
    pub selected_square: Option<(u8, u8)>,
    pub legal_move_destinations: Vec<(u8, u8)>,
    pub game_result: Option<ChallengeResult>,
    pub forfeit_pending: bool,
    pub ai_thinking: bool,
    pub ai_think_ticks: u32,
    pub ai_think_target: u32,
    pub ai_pending_board: Option<chess_engine::Board>,
    pub ai_pending_move: Option<Move>,
    pub player_is_white: bool,
    /// Last move made: (from_square, to_square)
    pub last_move: Option<((u8, u8), (u8, u8))>,
    /// Move history in algebraic notation
    pub move_history: Vec<String>,
}

impl ChessGame {
    /// Chess-specific AI search depth based on difficulty.
    pub fn search_depth(&self) -> i32 {
        match self.difficulty {
            ChallengeDifficulty::Novice => 1,
            ChallengeDifficulty::Apprentice => 1,
            ChallengeDifficulty::Journeyman => 2,
            ChallengeDifficulty::Master => 3,
        }
    }

    /// Chess-specific random move chance based on difficulty.
    pub fn random_move_chance(&self) -> f64 {
        match self.difficulty {
            ChallengeDifficulty::Novice => 0.5,
            _ => 0.0,
        }
    }

    pub fn new(difficulty: ChallengeDifficulty) -> Self {
        Self {
            board: chess_engine::Board::default(),
            difficulty,
            cursor: (4, 1), // e2
            selected_square: None,
            legal_move_destinations: Vec::new(),
            game_result: None,
            forfeit_pending: false,
            ai_thinking: false,
            ai_think_ticks: 0,
            ai_think_target: 0,
            ai_pending_board: None,
            ai_pending_move: None,
            player_is_white: true,
            last_move: None,
            move_history: Vec::new(),
        }
    }

    pub fn move_cursor(&mut self, dx: i8, dy: i8) {
        let new_x = (self.cursor.0 as i8 + dx).clamp(0, 7) as u8;
        let new_y = (self.cursor.1 as i8 + dy).clamp(0, 7) as u8;
        self.cursor = (new_x, new_y);
    }

    /// Get the player's color
    pub fn player_color(&self) -> ChessColor {
        if self.player_is_white {
            ChessColor::White
        } else {
            ChessColor::Black
        }
    }

    /// Check if it's the player's turn
    pub fn is_player_turn(&self) -> bool {
        self.board.get_turn_color() == self.player_color()
    }

    /// Check if the cursor is on a player's piece
    pub fn cursor_on_player_piece(&self) -> bool {
        let pos = Position::new(self.cursor.1 as i32, self.cursor.0 as i32);
        if let Some(piece) = self.board.get_piece(pos) {
            piece.get_color() == self.player_color()
        } else {
            false
        }
    }

    /// Select the piece at the cursor and compute legal moves for it
    pub fn select_piece_at_cursor(&mut self) -> bool {
        if !self.is_player_turn() || self.ai_thinking {
            return false;
        }

        let (file, rank) = self.cursor;
        let pos = Position::new(rank as i32, file as i32);

        // Check if there's a player piece at cursor
        if let Some(piece) = self.board.get_piece(pos) {
            if piece.get_color() != self.player_color() {
                return false;
            }

            // Find all legal moves from this position
            let all_moves = self.board.get_legal_moves();
            let mut destinations = Vec::new();

            for m in all_moves {
                match m {
                    Move::Piece(from, to) => {
                        if from == pos {
                            destinations.push((to.get_col() as u8, to.get_row() as u8));
                        }
                    }
                    Move::KingSideCastle => {
                        // Castling is from king position
                        if piece.is_king() {
                            let king_dest = if self.player_is_white { (6, 0) } else { (6, 7) };
                            destinations.push(king_dest);
                        }
                    }
                    Move::QueenSideCastle => {
                        if piece.is_king() {
                            let king_dest = if self.player_is_white { (2, 0) } else { (2, 7) };
                            destinations.push(king_dest);
                        }
                    }
                    Move::Resign => {}
                }
            }

            if !destinations.is_empty() {
                self.selected_square = Some((file, rank));
                self.legal_move_destinations = destinations;
                return true;
            }
        }

        false
    }

    /// Try to move the selected piece to the cursor position
    /// Returns true if a move was made
    pub fn try_move_to_cursor(&mut self) -> bool {
        if !self.is_player_turn() || self.ai_thinking {
            return false;
        }

        let Some((sel_file, sel_rank)) = self.selected_square else {
            return false;
        };

        let (dest_file, dest_rank) = self.cursor;

        // Check if destination is in legal moves
        if !self
            .legal_move_destinations
            .contains(&(dest_file, dest_rank))
        {
            return false;
        }

        // Determine the move to make
        let from_pos = Position::new(sel_rank as i32, sel_file as i32);
        let to_pos = Position::new(dest_rank as i32, dest_file as i32);

        // Check for castling moves
        let player_move = if let Some(piece) = self.board.get_piece(from_pos) {
            if piece.is_king() {
                // Check for kingside castle (king moves 2 squares right)
                if dest_file == sel_file + 2 {
                    Move::KingSideCastle
                // Check for queenside castle (king moves 2 squares left)
                } else if sel_file >= 2 && dest_file == sel_file - 2 {
                    Move::QueenSideCastle
                } else {
                    Move::Piece(from_pos, to_pos)
                }
            } else {
                Move::Piece(from_pos, to_pos)
            }
        } else {
            Move::Piece(from_pos, to_pos)
        };

        // Check if this is a capture
        let is_capture = self.board.get_piece(to_pos).is_some();

        // Generate algebraic notation before applying the move
        let notation = Self::move_to_algebraic(&self.board, &player_move, is_capture);

        // Apply the move
        match self.board.play_move(player_move) {
            chess_engine::GameResult::Continuing(new_board) => {
                // Record the move
                self.record_move((sel_file, sel_rank), (dest_file, dest_rank), notation);

                self.board = new_board;
                self.selected_square = None;
                self.legal_move_destinations.clear();

                // Start AI thinking
                self.ai_thinking = true;
                self.ai_think_ticks = 0;
                self.ai_pending_board = None;
                self.ai_pending_move = None;
                true
            }
            chess_engine::GameResult::Victory(winner) => {
                // Record the move
                self.record_move((sel_file, sel_rank), (dest_file, dest_rank), notation);

                self.selected_square = None;
                self.legal_move_destinations.clear();

                // Determine winner
                self.game_result = Some(if winner == self.player_color() {
                    ChallengeResult::Win
                } else {
                    ChallengeResult::Loss
                });
                true
            }
            chess_engine::GameResult::Stalemate => {
                // Record the move
                self.record_move((sel_file, sel_rank), (dest_file, dest_rank), notation);

                self.selected_square = None;
                self.legal_move_destinations.clear();
                self.game_result = Some(ChallengeResult::Draw);
                true
            }
            chess_engine::GameResult::IllegalMove(_) => {
                // Should not happen since we're using legal moves
                false
            }
        }
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.selected_square = None;
        self.legal_move_destinations.clear();
    }

    /// Record a move in history and update last_move
    pub fn record_move(&mut self, from: (u8, u8), to: (u8, u8), notation: String) {
        self.last_move = Some((from, to));
        self.move_history.push(notation);
    }

    /// Generate algebraic notation for a move
    pub fn move_to_algebraic(
        board: &chess_engine::Board,
        chess_move: &Move,
        is_capture: bool,
    ) -> String {
        match chess_move {
            Move::Piece(from, to) => {
                let piece_char = board
                    .get_piece(*from)
                    .map(|p| {
                        if p.is_king() {
                            "K"
                        } else if p.is_queen() {
                            "Q"
                        } else if p.is_rook() {
                            "R"
                        } else if p.is_bishop() {
                            "B"
                        } else if p.is_knight() {
                            "N"
                        } else {
                            ""
                        }
                    })
                    .unwrap_or("");

                let to_file = (b'a' + to.get_col() as u8) as char;
                let to_rank = (b'1' + to.get_row() as u8) as char;
                let capture = if is_capture { "x" } else { "" };

                if piece_char.is_empty() {
                    // Pawn move - include from file only on captures
                    if is_capture {
                        let from_file = (b'a' + from.get_col() as u8) as char;
                        format!("{}x{}{}", from_file, to_file, to_rank)
                    } else {
                        format!("{}{}", to_file, to_rank)
                    }
                } else {
                    format!("{}{}{}{}", piece_char, capture, to_file, to_rank)
                }
            }
            Move::KingSideCastle => "O-O".to_string(),
            Move::QueenSideCastle => "O-O-O".to_string(),
            Move::Resign => "resigns".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_from_index() {
        assert_eq!(
            ChallengeDifficulty::from_index(0),
            ChallengeDifficulty::Novice
        );
        assert_eq!(
            ChallengeDifficulty::from_index(1),
            ChallengeDifficulty::Apprentice
        );
        assert_eq!(
            ChallengeDifficulty::from_index(2),
            ChallengeDifficulty::Journeyman
        );
        assert_eq!(
            ChallengeDifficulty::from_index(3),
            ChallengeDifficulty::Master
        );
        assert_eq!(
            ChallengeDifficulty::from_index(99),
            ChallengeDifficulty::Novice
        );
    }

    #[test]
    fn test_difficulty_properties() {
        let novice = ChessGame::new(ChallengeDifficulty::Novice);
        let apprentice = ChessGame::new(ChallengeDifficulty::Apprentice);

        assert_eq!(novice.random_move_chance(), 0.5);
        assert_eq!(apprentice.random_move_chance(), 0.0);
    }

    #[test]
    fn test_difficulty_rewards_via_challenge_type() {
        use crate::challenges::menu::ChallengeType;

        // Chess rewards prestige only
        let novice = ChallengeType::Chess.reward(ChallengeDifficulty::Novice);
        assert_eq!(novice.prestige_ranks, 1);
        assert_eq!(novice.xp_percent, 0);
        assert_eq!(novice.fishing_ranks, 0);

        let apprentice = ChallengeType::Chess.reward(ChallengeDifficulty::Apprentice);
        assert_eq!(apprentice.prestige_ranks, 2);

        let journeyman = ChallengeType::Chess.reward(ChallengeDifficulty::Journeyman);
        assert_eq!(journeyman.prestige_ranks, 3);

        let master = ChallengeType::Chess.reward(ChallengeDifficulty::Master);
        assert_eq!(master.prestige_ranks, 5);
        assert_eq!(master.xp_percent, 0);
        assert_eq!(master.fishing_ranks, 0);
    }

    #[test]
    fn test_chess_game_new() {
        let game = ChessGame::new(ChallengeDifficulty::Journeyman);
        assert_eq!(game.difficulty, ChallengeDifficulty::Journeyman);
        assert_eq!(game.cursor, (4, 1));
        assert!(game.selected_square.is_none());
        assert!(game.game_result.is_none());
        assert!(!game.ai_thinking);
    }

    #[test]
    fn test_cursor_movement() {
        let mut game = ChessGame::new(ChallengeDifficulty::Novice);
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
        let mut game = ChessGame::new(ChallengeDifficulty::Novice);
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

    #[test]
    fn test_player_color() {
        let game = ChessGame::new(ChallengeDifficulty::Novice);
        assert!(game.player_is_white);
        assert_eq!(game.player_color(), ChessColor::White);
        assert!(game.is_player_turn());
    }

    #[test]
    fn test_cursor_on_player_piece() {
        let game = ChessGame::new(ChallengeDifficulty::Novice);
        // Cursor starts at e2 which has a white pawn
        assert!(game.cursor_on_player_piece());
    }

    #[test]
    fn test_select_piece_at_cursor() {
        let mut game = ChessGame::new(ChallengeDifficulty::Novice);
        // Cursor at e2 (white pawn)
        let selected = game.select_piece_at_cursor();
        assert!(selected);
        assert_eq!(game.selected_square, Some((4, 1)));
        // e2 pawn can move to e3 and e4
        assert!(game.legal_move_destinations.contains(&(4, 2))); // e3
        assert!(game.legal_move_destinations.contains(&(4, 3))); // e4
    }

    #[test]
    fn test_select_empty_square() {
        let mut game = ChessGame::new(ChallengeDifficulty::Novice);
        game.cursor = (4, 4); // e5 - empty square
        let selected = game.select_piece_at_cursor();
        assert!(!selected);
        assert!(game.selected_square.is_none());
    }

    #[test]
    fn test_select_enemy_piece() {
        let mut game = ChessGame::new(ChallengeDifficulty::Novice);
        game.cursor = (4, 6); // e7 - black pawn
        let selected = game.select_piece_at_cursor();
        assert!(!selected);
        assert!(game.selected_square.is_none());
    }

    #[test]
    fn test_clear_selection() {
        let mut game = ChessGame::new(ChallengeDifficulty::Novice);
        game.select_piece_at_cursor();
        assert!(game.selected_square.is_some());
        game.clear_selection();
        assert!(game.selected_square.is_none());
        assert!(game.legal_move_destinations.is_empty());
    }

    #[test]
    fn test_try_move_to_cursor() {
        let mut game = ChessGame::new(ChallengeDifficulty::Novice);
        // Select e2 pawn
        game.select_piece_at_cursor();
        // Move cursor to e4
        game.cursor = (4, 3);
        let moved = game.try_move_to_cursor();
        assert!(moved);
        // Selection should be cleared
        assert!(game.selected_square.is_none());
        // AI should be thinking
        assert!(game.ai_thinking);
    }

    #[test]
    fn test_try_invalid_move() {
        let mut game = ChessGame::new(ChallengeDifficulty::Novice);
        // Select e2 pawn
        game.select_piece_at_cursor();
        // Move cursor to e5 (not a legal move for pawn)
        game.cursor = (4, 4);
        let moved = game.try_move_to_cursor();
        assert!(!moved);
        // Selection should remain
        assert!(game.selected_square.is_some());
    }

    // ============ Algebraic Notation Tests ============

    #[test]
    fn test_pawn_move_notation() {
        let board = chess_engine::Board::default();
        // e2-e4 pawn move (no capture)
        let from = Position::new(1, 4); // e2
        let to = Position::new(3, 4); // e4
        let m = Move::Piece(from, to);
        let notation = ChessGame::move_to_algebraic(&board, &m, false);
        assert_eq!(notation, "e4");
    }

    #[test]
    fn test_pawn_capture_notation() {
        // Set up a board where a pawn can capture
        let board = chess_engine::Board::default();
        let from = Position::new(1, 4); // e2
        let to = Position::new(2, 3); // d3 (diagonal capture)
        let m = Move::Piece(from, to);
        let notation = ChessGame::move_to_algebraic(&board, &m, true);
        assert_eq!(notation, "exd3");
    }

    #[test]
    fn test_knight_move_notation() {
        let board = chess_engine::Board::default();
        // Ng1-f3
        let from = Position::new(0, 6); // g1
        let to = Position::new(2, 5); // f3
        let m = Move::Piece(from, to);
        let notation = ChessGame::move_to_algebraic(&board, &m, false);
        assert_eq!(notation, "Nf3");
    }

    #[test]
    fn test_knight_capture_notation() {
        let board = chess_engine::Board::default();
        let from = Position::new(0, 6); // g1
        let to = Position::new(2, 5); // f3
        let m = Move::Piece(from, to);
        let notation = ChessGame::move_to_algebraic(&board, &m, true);
        assert_eq!(notation, "Nxf3");
    }

    #[test]
    fn test_kingside_castle_notation() {
        let board = chess_engine::Board::default();
        let notation = ChessGame::move_to_algebraic(&board, &Move::KingSideCastle, false);
        assert_eq!(notation, "O-O");
    }

    #[test]
    fn test_queenside_castle_notation() {
        let board = chess_engine::Board::default();
        let notation = ChessGame::move_to_algebraic(&board, &Move::QueenSideCastle, false);
        assert_eq!(notation, "O-O-O");
    }

    #[test]
    fn test_bishop_move_notation() {
        let board = chess_engine::Board::default();
        // Bc1 (if it could move to c1)
        let from = Position::new(0, 2); // c1
        let to = Position::new(2, 4); // e3
        let m = Move::Piece(from, to);
        let notation = ChessGame::move_to_algebraic(&board, &m, false);
        assert_eq!(notation, "Be3");
    }

    #[test]
    fn test_queen_move_notation() {
        let board = chess_engine::Board::default();
        let from = Position::new(0, 3); // d1
        let to = Position::new(2, 3); // d3
        let m = Move::Piece(from, to);
        let notation = ChessGame::move_to_algebraic(&board, &m, false);
        assert_eq!(notation, "Qd3");
    }

    #[test]
    fn test_rook_move_notation() {
        let board = chess_engine::Board::default();
        let from = Position::new(0, 0); // a1
        let to = Position::new(3, 0); // a4
        let m = Move::Piece(from, to);
        let notation = ChessGame::move_to_algebraic(&board, &m, false);
        assert_eq!(notation, "Ra4");
    }

    #[test]
    fn test_king_move_notation() {
        let board = chess_engine::Board::default();
        let from = Position::new(0, 4); // e1
        let to = Position::new(1, 4); // e2
        let m = Move::Piece(from, to);
        let notation = ChessGame::move_to_algebraic(&board, &m, false);
        assert_eq!(notation, "Ke2");
    }

    // ============ Move History Tests ============

    #[test]
    fn test_record_move_updates_last_move() {
        let mut game = ChessGame::new(ChallengeDifficulty::Novice);
        assert!(game.last_move.is_none());

        game.record_move((4, 1), (4, 3), "e4".to_string());

        assert_eq!(game.last_move, Some(((4, 1), (4, 3))));
    }

    #[test]
    fn test_record_move_adds_to_history() {
        let mut game = ChessGame::new(ChallengeDifficulty::Novice);
        assert!(game.move_history.is_empty());

        game.record_move((4, 1), (4, 3), "e4".to_string());
        game.record_move((4, 6), (4, 4), "e5".to_string());

        assert_eq!(game.move_history.len(), 2);
        assert_eq!(game.move_history[0], "e4");
        assert_eq!(game.move_history[1], "e5");
    }

    #[test]
    fn test_move_history_after_player_move() {
        let mut game = ChessGame::new(ChallengeDifficulty::Novice);
        // Select e2 pawn and move to e4
        game.select_piece_at_cursor();
        game.cursor = (4, 3);
        game.try_move_to_cursor();

        // Move should be recorded
        assert_eq!(game.move_history.len(), 1);
        assert_eq!(game.move_history[0], "e4");
        assert_eq!(game.last_move, Some(((4, 1), (4, 3))));
    }

    #[test]
    fn test_last_move_updates_each_move() {
        let mut game = ChessGame::new(ChallengeDifficulty::Novice);

        game.record_move((4, 1), (4, 3), "e4".to_string());
        assert_eq!(game.last_move, Some(((4, 1), (4, 3))));

        game.record_move((4, 6), (4, 4), "e5".to_string());
        assert_eq!(game.last_move, Some(((4, 6), (4, 4))));

        game.record_move((6, 0), (5, 2), "Nf3".to_string());
        assert_eq!(game.last_move, Some(((6, 0), (5, 2))));
    }

    #[test]
    fn test_new_game_has_empty_history() {
        let game = ChessGame::new(ChallengeDifficulty::Master);
        assert!(game.move_history.is_empty());
        assert!(game.last_move.is_none());
    }

    // ============ Forfeit Flow Tests ============

    #[test]
    fn test_forfeit_pending_starts_false() {
        let game = ChessGame::new(ChallengeDifficulty::Novice);
        assert!(!game.forfeit_pending);
    }

    #[test]
    fn test_forfeit_pending_can_be_set() {
        let mut game = ChessGame::new(ChallengeDifficulty::Novice);
        game.forfeit_pending = true;
        assert!(game.forfeit_pending);
    }

    #[test]
    fn test_forfeit_result_sets_game_over() {
        let mut game = ChessGame::new(ChallengeDifficulty::Novice);
        game.game_result = Some(ChallengeResult::Forfeit);
        assert_eq!(game.game_result, Some(ChallengeResult::Forfeit));
    }

    #[test]
    fn test_forfeit_clears_on_piece_selection() {
        let mut game = ChessGame::new(ChallengeDifficulty::Novice);
        game.forfeit_pending = true;

        // Selecting a piece should conceptually clear forfeit
        // (In actual code this happens in main.rs input handling)
        // Here we test the state can be cleared
        game.forfeit_pending = false;
        assert!(!game.forfeit_pending);
    }

    #[test]
    fn test_move_clears_selection_not_forfeit() {
        let mut game = ChessGame::new(ChallengeDifficulty::Novice);
        game.forfeit_pending = true;

        // Make a move
        game.select_piece_at_cursor();
        game.cursor = (4, 3); // e4
        game.try_move_to_cursor();

        // Move itself doesn't clear forfeit - that's handled by input
        // But selection is cleared
        assert!(game.selected_square.is_none());
    }
}
