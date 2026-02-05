//! Chess game logic: AI moves, and game resolution.

use super::{ChessDifficulty, ChessGame, ChessResult};
use crate::core::game_state::GameState;
use chess_engine::Evaluate;
use rand::Rng;

/// Input actions for the Chess game (UI-agnostic).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChessInput {
    Up,
    Down,
    Left,
    Right,
    Select, // Enter - select piece or move
    Cancel, // Esc - clear selection or forfeit
    Other,
}

/// Process a key input during active Chess game.
/// Returns true if the input was handled.
/// Does nothing if AI is thinking.
pub fn process_input(game: &mut ChessGame, input: ChessInput) -> bool {
    // Don't process input while AI is thinking
    if game.ai_thinking {
        return false;
    }

    match input {
        ChessInput::Up => game.move_cursor(0, 1),
        ChessInput::Down => game.move_cursor(0, -1),
        ChessInput::Left => game.move_cursor(-1, 0),
        ChessInput::Right => game.move_cursor(1, 0),
        ChessInput::Select => {
            process_select(game);
        }
        ChessInput::Cancel => {
            process_cancel(game);
        }
        ChessInput::Other => {
            // Any other key cancels forfeit pending
            game.forfeit_pending = false;
        }
    }
    true
}

/// Process Enter key: select piece or move to destination.
fn process_select(game: &mut ChessGame) {
    if game.selected_square.is_some() {
        // A piece is selected - try to move or reselect
        if game.legal_move_destinations.contains(&game.cursor) {
            game.try_move_to_cursor();
        } else if game.cursor_on_player_piece() {
            // Cursor on another player piece - select it instead
            game.select_piece_at_cursor();
        } else {
            // Invalid destination - clear selection
            game.clear_selection();
        }
    } else {
        // No piece selected - try to select piece at cursor
        game.select_piece_at_cursor();
    }
}

/// Process Esc key: clear selection or initiate/confirm forfeit.
fn process_cancel(game: &mut ChessGame) {
    if game.forfeit_pending {
        game.game_result = Some(ChessResult::Forfeit);
    } else if game.selected_square.is_some() {
        game.clear_selection();
        game.forfeit_pending = false;
    } else {
        game.forfeit_pending = true;
    }
}

/// Start a chess game with the selected difficulty
pub fn start_chess_game(state: &mut GameState, difficulty: ChessDifficulty) {
    state.active_chess = Some(ChessGame::new(difficulty));
    state.challenge_menu.close();
}

/// Calculate variable AI thinking time in ticks (1.5-6s range at 100ms/tick)
pub fn calculate_think_ticks<R: Rng>(board: &chess_engine::Board, rng: &mut R) -> u32 {
    let base_ticks = rng.gen_range(15..40); // 1.5-4s base
    let legal_moves = board.get_legal_moves();
    let complexity_bonus = (legal_moves.len() / 5) as u32;
    base_ticks + complexity_bonus
}

/// Get AI move with difficulty-based weakening, returns the chosen Move
pub fn get_ai_move<R: Rng>(
    board: &chess_engine::Board,
    difficulty: ChessDifficulty,
    rng: &mut R,
) -> chess_engine::Move {
    let legal_moves = board.get_legal_moves();
    if legal_moves.is_empty() {
        return chess_engine::Move::Resign;
    }

    // Random move for Novice difficulty
    if rng.gen::<f64>() < difficulty.random_move_chance() {
        let idx = rng.gen_range(0..legal_moves.len());
        return legal_moves[idx];
    }

    let (best_move, _, _) = board.get_best_next_move(difficulty.search_depth());
    best_move
}

/// Apply a move to the board and return the resulting board state (if game continues)
pub fn apply_move_to_board(
    board: &chess_engine::Board,
    m: chess_engine::Move,
) -> Option<chess_engine::Board> {
    match board.play_move(m) {
        chess_engine::GameResult::Continuing(new_board) => Some(new_board),
        _ => None,
    }
}

/// Process AI thinking tick, returns true if AI made a move
pub fn process_ai_thinking<R: Rng>(game: &mut ChessGame, rng: &mut R) -> bool {
    if !game.ai_thinking {
        return false;
    }

    game.ai_think_ticks += 1;

    // Compute AI move on first tick
    if game.ai_pending_board.is_none() {
        let ai_move = get_ai_move(&game.board, game.difficulty, rng);
        // Apply the move to get the resulting board
        if let Some(new_board) = apply_move_to_board(&game.board, ai_move) {
            game.ai_pending_board = Some(new_board);
            game.ai_pending_move = Some(ai_move);
        }
        game.ai_think_target = calculate_think_ticks(&game.board, rng);
    }

    // Apply move after delay
    if game.ai_think_ticks >= game.ai_think_target {
        // Record the AI's move before applying
        if let Some(ref ai_move) = game.ai_pending_move {
            let (from, to) = extract_move_squares(ai_move, game.player_is_white);
            let is_capture = game
                .board
                .get_piece(chess_engine::Position::new(to.1 as i32, to.0 as i32))
                .is_some();
            let notation = ChessGame::move_to_algebraic(&game.board, ai_move, is_capture);
            game.record_move(from, to, notation);
        }

        if let Some(new_board) = game.ai_pending_board.take() {
            game.board = new_board;
        }
        game.ai_pending_move = None;
        game.ai_thinking = false;
        game.ai_think_ticks = 0;
        check_game_over(game);
        return true;
    }

    false
}

/// Extract from/to squares from a Move
fn extract_move_squares(m: &chess_engine::Move, player_is_white: bool) -> ((u8, u8), (u8, u8)) {
    match m {
        chess_engine::Move::Piece(from, to) => (
            (from.get_col() as u8, from.get_row() as u8),
            (to.get_col() as u8, to.get_row() as u8),
        ),
        chess_engine::Move::KingSideCastle => {
            // AI is opposite color of player
            if player_is_white {
                // AI is black, king moves e8 to g8
                ((4, 7), (6, 7))
            } else {
                // AI is white, king moves e1 to g1
                ((4, 0), (6, 0))
            }
        }
        chess_engine::Move::QueenSideCastle => {
            if player_is_white {
                // AI is black, king moves e8 to c8
                ((4, 7), (2, 7))
            } else {
                // AI is white, king moves e1 to c1
                ((4, 0), (2, 0))
            }
        }
        chess_engine::Move::Resign => ((0, 0), (0, 0)),
    }
}

/// Check if the game is over (checkmate or stalemate)
pub fn check_game_over(game: &mut ChessGame) {
    if game.game_result.is_some() {
        return;
    }

    // Use the chess-engine's built-in checkmate/stalemate detection
    if game.board.is_checkmate() {
        // Current player (whose turn it is) is in checkmate
        let loser = game.board.get_turn_color();
        game.game_result = Some(if loser == game.player_color() {
            ChessResult::Loss
        } else {
            ChessResult::Win
        });
    } else if game.board.is_stalemate() {
        game.game_result = Some(ChessResult::Draw);
    }
}

/// Apply game result: update stats, grant rewards, and add combat log entries.
/// Returns true if a result was processed.
pub fn apply_game_result(state: &mut GameState) -> bool {
    use crate::challenges::menu::DifficultyInfo;

    let game = match state.active_chess.as_ref() {
        Some(g) => g,
        None => return false,
    };
    let result = match game.game_result {
        Some(r) => r,
        None => return false,
    };
    let reward = game.difficulty.reward();
    let old_prestige = state.prestige_rank;

    state.chess_stats.games_played += 1;

    match result {
        ChessResult::Win => {
            state.chess_stats.games_won += 1;
            state.prestige_rank += reward.prestige_ranks;
            state.chess_stats.prestige_earned += reward.prestige_ranks;

            // Combat log entries
            state.combat_state.add_log_entry(
                "♟ Checkmate! You defeated the mysterious figure.".to_string(),
                false,
                true,
            );
            if reward.prestige_ranks > 0 {
                state.combat_state.add_log_entry(
                    format!(
                        "♟ +{} Prestige Ranks (P{} → P{})",
                        reward.prestige_ranks, old_prestige, state.prestige_rank
                    ),
                    false,
                    true,
                );
            }
        }
        ChessResult::Loss => {
            state.chess_stats.games_lost += 1;
            state.combat_state.add_log_entry(
                "♟ The mysterious figure nods respectfully and vanishes.".to_string(),
                false,
                true,
            );
        }
        ChessResult::Forfeit => {
            state.chess_stats.games_lost += 1;
            state.combat_state.add_log_entry(
                "♟ You concede the game. The figure disappears without a word.".to_string(),
                false,
                true,
            );
        }
        ChessResult::Draw => {
            state.chess_stats.games_drawn += 1;
            state.combat_state.add_log_entry(
                "♟ The figure smiles knowingly and fades away.".to_string(),
                false,
                true,
            );
        }
    }

    state.active_chess = None;
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::challenges::menu::{ChallengeType, PendingChallenge};

    fn make_chess_challenge() -> PendingChallenge {
        PendingChallenge {
            challenge_type: ChallengeType::Chess,
            title: "Chess Challenge".to_string(),
            icon: "♟",
            description: "Test".to_string(),
        }
    }

    #[test]
    fn test_start_chess_game() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.challenge_menu.open();
        start_chess_game(&mut state, ChessDifficulty::Journeyman);
        assert!(state.active_chess.is_some());
        assert!(!state.challenge_menu.is_open);
    }

    #[test]
    fn test_apply_win_result() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.prestige_rank = 5;
        let mut game = ChessGame::new(ChessDifficulty::Master);
        game.game_result = Some(ChessResult::Win);
        state.active_chess = Some(game);

        let processed = apply_game_result(&mut state);
        assert!(processed);
        assert_eq!(state.prestige_rank, 10); // 5 + 5 (Master reward)
        assert_eq!(state.chess_stats.games_won, 1);
        assert!(state.active_chess.is_none());
    }

    #[test]
    fn test_apply_loss_result() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.prestige_rank = 5;
        let mut game = ChessGame::new(ChessDifficulty::Novice);
        game.game_result = Some(ChessResult::Loss);
        state.active_chess = Some(game);

        let processed = apply_game_result(&mut state);
        assert!(processed);
        assert_eq!(state.prestige_rank, 5); // Unchanged
        assert_eq!(state.chess_stats.games_lost, 1);
    }

    // ============ AI Move Tests ============

    #[test]
    fn test_ai_makes_legal_move() {
        let board = chess_engine::Board::default();
        let mut rng = rand::thread_rng();

        let ai_move = get_ai_move(&board, ChessDifficulty::Novice, &mut rng);

        // Move should be in the legal moves list
        let legal_moves = board.get_legal_moves();
        assert!(
            legal_moves.contains(&ai_move),
            "AI should only make legal moves"
        );
    }

    #[test]
    fn test_ai_move_different_difficulties() {
        let board = chess_engine::Board::default();
        let mut rng = rand::thread_rng();

        // All difficulties should return legal moves
        for difficulty in ChessDifficulty::ALL {
            let ai_move = get_ai_move(&board, difficulty, &mut rng);
            let legal_moves = board.get_legal_moves();
            assert!(
                legal_moves.contains(&ai_move),
                "AI at {:?} should make legal moves",
                difficulty
            );
        }
    }

    #[test]
    fn test_extract_move_squares_piece_move() {
        use chess_engine::Position;
        let m = chess_engine::Move::Piece(Position::new(1, 4), Position::new(3, 4)); // e2-e4
        let (from, to) = extract_move_squares(&m, true);
        assert_eq!(from, (4, 1)); // e2
        assert_eq!(to, (4, 3)); // e4
    }

    #[test]
    fn test_extract_move_squares_kingside_castle() {
        // AI is black (player is white)
        let (from, to) = extract_move_squares(&chess_engine::Move::KingSideCastle, true);
        assert_eq!(from, (4, 7)); // e8
        assert_eq!(to, (6, 7)); // g8
    }

    #[test]
    fn test_extract_move_squares_queenside_castle() {
        // AI is black (player is white)
        let (from, to) = extract_move_squares(&chess_engine::Move::QueenSideCastle, true);
        assert_eq!(from, (4, 7)); // e8
        assert_eq!(to, (2, 7)); // c8
    }

    // ============ Challenge Menu Integration Tests ============

    #[test]
    fn test_accepting_challenge_starts_game() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.challenge_menu.add_challenge(make_chess_challenge());
        state.challenge_menu.open();

        assert!(state.challenge_menu.has_challenge(&ChallengeType::Chess));
        assert!(state.active_chess.is_none());

        // Start game (simulating accept)
        start_chess_game(&mut state, ChessDifficulty::Novice);

        assert!(state.active_chess.is_some());
        assert!(!state.challenge_menu.is_open);
    }

    #[test]
    fn test_accepting_challenge_removes_from_menu() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.challenge_menu.add_challenge(make_chess_challenge());

        assert_eq!(state.challenge_menu.challenges.len(), 1);

        // Take the challenge (simulating accept flow)
        let taken = state.challenge_menu.take_selected();
        assert!(taken.is_some());
        assert_eq!(state.challenge_menu.challenges.len(), 0);
        assert!(!state.challenge_menu.has_challenge(&ChallengeType::Chess));
    }

    #[test]
    fn test_declining_challenge_removes_it() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.challenge_menu.add_challenge(make_chess_challenge());

        assert_eq!(state.challenge_menu.challenges.len(), 1);

        // Decline = take and discard without starting game
        let _declined = state.challenge_menu.take_selected();

        assert_eq!(state.challenge_menu.challenges.len(), 0);
        assert!(state.active_chess.is_none()); // Game not started
    }

    #[test]
    fn test_multiple_challenges_in_menu() {
        let mut state = GameState::new("Test".to_string(), 0);

        // Add multiple challenges
        state.challenge_menu.add_challenge(make_chess_challenge());
        state.challenge_menu.add_challenge(PendingChallenge {
            challenge_type: ChallengeType::Chess,
            title: "Chess Challenge 2".to_string(),
            icon: "♟",
            description: "Another challenger appears!".to_string(),
        });

        assert_eq!(state.challenge_menu.challenges.len(), 2);

        // Take first challenge
        state.challenge_menu.take_selected();
        assert_eq!(state.challenge_menu.challenges.len(), 1);

        // Take second challenge
        state.challenge_menu.take_selected();
        assert_eq!(state.challenge_menu.challenges.len(), 0);
    }

    #[test]
    fn test_challenge_menu_navigation_with_multiple() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.challenge_menu.add_challenge(make_chess_challenge());
        state.challenge_menu.add_challenge(PendingChallenge {
            challenge_type: ChallengeType::Chess,
            title: "Second Challenge".to_string(),
            icon: "♟",
            description: "Another one".to_string(),
        });
        state.challenge_menu.open();

        assert_eq!(state.challenge_menu.selected_index, 0);

        state.challenge_menu.navigate_down(4);
        assert_eq!(state.challenge_menu.selected_index, 1);

        // Take selected (second challenge)
        let taken = state.challenge_menu.take_selected();
        assert_eq!(taken.unwrap().title, "Second Challenge");

        // Index should adjust
        assert_eq!(state.challenge_menu.selected_index, 0);
    }

    // ============ Forfeit Stats Tests ============

    #[test]
    fn test_forfeit_counts_as_loss_in_stats() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.prestige_rank = 5;

        let mut game = ChessGame::new(ChessDifficulty::Novice);
        game.game_result = Some(ChessResult::Forfeit);
        state.active_chess = Some(game);

        let processed = apply_game_result(&mut state);
        assert!(processed);
        assert_eq!(state.chess_stats.games_lost, 1); // Counts as loss
        assert_eq!(state.chess_stats.games_won, 0);
        assert_eq!(state.prestige_rank, 5); // No penalty
    }

    #[test]
    fn test_draw_gives_no_prestige() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.prestige_rank = 5;

        let mut game = ChessGame::new(ChessDifficulty::Master);
        game.game_result = Some(ChessResult::Draw);
        state.active_chess = Some(game);

        let processed = apply_game_result(&mut state);
        assert!(processed);
        assert_eq!(state.chess_stats.games_drawn, 1);
        assert_eq!(state.prestige_rank, 5); // Unchanged
    }

    // ============ process_input Tests ============

    #[test]
    fn test_process_input_cursor_movement() {
        let mut game = ChessGame::new(ChessDifficulty::Novice);

        // Start at initial cursor position (e2 for white)
        let initial = game.cursor;

        process_input(&mut game, ChessInput::Up);
        assert_eq!(game.cursor, (initial.0, initial.1 + 1));

        process_input(&mut game, ChessInput::Down);
        assert_eq!(game.cursor, initial);

        process_input(&mut game, ChessInput::Right);
        assert_eq!(game.cursor, (initial.0 + 1, initial.1));

        process_input(&mut game, ChessInput::Left);
        assert_eq!(game.cursor, initial);
    }

    #[test]
    fn test_process_input_select_piece() {
        let mut game = ChessGame::new(ChessDifficulty::Novice);

        // Move to e2 (pawn position for white)
        game.cursor = (4, 1);

        // Select the pawn
        process_input(&mut game, ChessInput::Select);

        assert_eq!(game.selected_square, Some((4, 1)));
        assert!(!game.legal_move_destinations.is_empty());
    }

    #[test]
    fn test_process_input_clear_selection_with_cancel() {
        let mut game = ChessGame::new(ChessDifficulty::Novice);

        // Select a piece first
        game.cursor = (4, 1);
        process_input(&mut game, ChessInput::Select);
        assert!(game.selected_square.is_some());

        // Cancel should clear selection
        process_input(&mut game, ChessInput::Cancel);

        assert!(game.selected_square.is_none());
        assert!(!game.forfeit_pending);
    }

    #[test]
    fn test_process_input_forfeit_single_esc() {
        let mut game = ChessGame::new(ChessDifficulty::Novice);

        // No piece selected, first Esc sets pending
        process_input(&mut game, ChessInput::Cancel);

        assert!(game.forfeit_pending);
        assert!(game.game_result.is_none());
    }

    #[test]
    fn test_process_input_forfeit_double_esc() {
        let mut game = ChessGame::new(ChessDifficulty::Novice);

        // First Esc sets pending
        process_input(&mut game, ChessInput::Cancel);
        assert!(game.forfeit_pending);

        // Second Esc confirms forfeit
        process_input(&mut game, ChessInput::Cancel);

        assert_eq!(game.game_result, Some(ChessResult::Forfeit));
    }

    #[test]
    fn test_process_input_forfeit_cancelled_by_other_key() {
        let mut game = ChessGame::new(ChessDifficulty::Novice);

        // First Esc sets pending
        process_input(&mut game, ChessInput::Cancel);
        assert!(game.forfeit_pending);

        // Any other key cancels forfeit
        process_input(&mut game, ChessInput::Other);

        assert!(!game.forfeit_pending);
        assert!(game.game_result.is_none());
    }

    #[test]
    fn test_process_input_blocked_during_ai_thinking() {
        let mut game = ChessGame::new(ChessDifficulty::Novice);
        game.ai_thinking = true;
        let initial_cursor = game.cursor;

        // Input should be blocked
        let handled = process_input(&mut game, ChessInput::Up);

        assert!(!handled);
        assert_eq!(game.cursor, initial_cursor);
    }

    #[test]
    fn test_process_input_reselect_different_piece() {
        let mut game = ChessGame::new(ChessDifficulty::Novice);

        // Select e2 pawn
        game.cursor = (4, 1);
        process_input(&mut game, ChessInput::Select);
        assert_eq!(game.selected_square, Some((4, 1)));

        // Move cursor to d2 pawn and select
        game.cursor = (3, 1);
        process_input(&mut game, ChessInput::Select);

        // Should now have d2 selected
        assert_eq!(game.selected_square, Some((3, 1)));
    }
}
