//! Chess puzzle game logic: input processing, move validation, puzzle advancement.

use super::puzzles::get_puzzles;
use super::types::{
    ChessPuzzleDifficulty, ChessPuzzleGame, ChessPuzzleResult, ChessPuzzleStats, PuzzleSolution,
    PuzzleState,
};
use crate::challenges::ActiveMinigame;
use crate::core::game_state::GameState;
use chess_engine::Evaluate;

/// Input actions for chess puzzles (UI-agnostic).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChessPuzzleInput {
    Up,
    Down,
    Left,
    Right,
    Select,  // Enter - select piece or confirm move
    Forfeit, // Esc - clear selection or forfeit
    Other,
}

/// Set up the board for the current puzzle by replaying setup_moves.
pub fn setup_current_puzzle(game: &mut ChessPuzzleGame) {
    let puzzle_idx = game.puzzle_order[game.current_puzzle_index];
    let puzzles = get_puzzles(game.difficulty);
    let puzzle = &puzzles[puzzle_idx];

    let mut board = chess_engine::Board::default();
    for &(from_rank, from_file, to_rank, to_file) in puzzle.setup_moves {
        let from = chess_engine::Position::new(from_rank, from_file);
        let to = chess_engine::Position::new(to_rank, to_file);
        let m = chess_engine::Move::Piece(from, to);
        match board.play_move(m) {
            chess_engine::GameResult::Continuing(new_board) => {
                board = new_board;
            }
            _ => {
                debug_assert!(false, "Puzzle setup move ended game: {:?}", puzzle.title);
                return;
            }
        }
    }

    game.board = board;
    game.player_is_white = puzzle.player_is_white;
    game.puzzle_state = PuzzleState::Solving;
    game.move_number_in_puzzle = 0;
    game.selected_square = None;
    game.legal_move_destinations.clear();
    game.last_move = None;
    game.forfeit_pending = false;
    game.cursor = if puzzle.player_is_white {
        (4, 3)
    } else {
        (4, 4)
    };
}

/// Process player input during the puzzle.
pub fn process_input(game: &mut ChessPuzzleGame, input: ChessPuzzleInput) -> bool {
    // Block input during AI thinking or feedback display
    if game.ai_thinking
        || game.puzzle_state == PuzzleState::Correct
        || game.puzzle_state == PuzzleState::Wrong
    {
        return false;
    }

    match input {
        ChessPuzzleInput::Up => game.move_cursor(0, 1),
        ChessPuzzleInput::Down => game.move_cursor(0, -1),
        ChessPuzzleInput::Left => game.move_cursor(-1, 0),
        ChessPuzzleInput::Right => game.move_cursor(1, 0),
        ChessPuzzleInput::Select => process_select(game),
        ChessPuzzleInput::Forfeit => process_cancel(game),
        ChessPuzzleInput::Other => {
            game.forfeit_pending = false;
        }
    }
    true
}

fn process_select(game: &mut ChessPuzzleGame) {
    if let Some(from) = game.selected_square {
        if game.legal_move_destinations.contains(&game.cursor) {
            let to = game.cursor;
            game.last_move = Some((from, to));
            validate_player_move(game, from, to);
        } else if cursor_on_player_piece(game) {
            select_piece_at_cursor(game);
        } else {
            game.selected_square = None;
            game.legal_move_destinations.clear();
        }
    } else {
        select_piece_at_cursor(game);
    }
}

fn process_cancel(game: &mut ChessPuzzleGame) {
    if game.forfeit_pending {
        game.game_result = Some(ChessPuzzleResult::Loss);
    } else if game.selected_square.is_some() {
        game.selected_square = None;
        game.legal_move_destinations.clear();
        game.forfeit_pending = false;
    } else {
        game.forfeit_pending = true;
    }
}

/// Check if the cursor is on a piece belonging to the player.
fn cursor_on_player_piece(game: &ChessPuzzleGame) -> bool {
    let pos = chess_engine::Position::new(game.cursor.1 as i32, game.cursor.0 as i32);
    if let Some(piece) = game.board.get_piece(pos) {
        piece.get_color() == game.player_color()
    } else {
        false
    }
}

/// Select the piece at the cursor and compute legal destinations.
fn select_piece_at_cursor(game: &mut ChessPuzzleGame) {
    if !cursor_on_player_piece(game) {
        return;
    }

    game.selected_square = Some(game.cursor);
    game.legal_move_destinations.clear();
    game.forfeit_pending = false;

    let legal_moves = game.board.get_legal_moves();
    let (cursor_file, cursor_rank) = game.cursor;

    for m in &legal_moves {
        if let chess_engine::Move::Piece(from, to) = m {
            if from.get_row() as u8 == cursor_rank && from.get_col() as u8 == cursor_file {
                game.legal_move_destinations
                    .push((to.get_col() as u8, to.get_row() as u8));
            }
        }
    }
}

/// Validate the player's move against the puzzle solution.
fn validate_player_move(game: &mut ChessPuzzleGame, from: (u8, u8), to: (u8, u8)) {
    let puzzle_idx = game.puzzle_order[game.current_puzzle_index];
    let puzzles = get_puzzles(game.difficulty);
    let puzzle = &puzzles[puzzle_idx];

    let from_pos = chess_engine::Position::new(from.1 as i32, from.0 as i32);
    let to_pos = chess_engine::Position::new(to.1 as i32, to.0 as i32);
    let player_move = chess_engine::Move::Piece(from_pos, to_pos);

    match &puzzle.solution {
        PuzzleSolution::MateInOne => {
            match game.board.play_move(player_move) {
                chess_engine::GameResult::Victory(_) => {
                    // Checkmate! Correct.
                    mark_correct(game);
                }
                chess_engine::GameResult::Continuing(new_board) => {
                    if new_board.is_checkmate() {
                        mark_correct(game);
                    } else {
                        mark_wrong(game);
                    }
                    game.board = new_board;
                }
                _ => {
                    mark_wrong(game);
                }
            }
        }

        PuzzleSolution::BestMove(exp_fr, exp_ff, exp_tr, exp_tf) => {
            if from.1 as i32 == *exp_fr
                && from.0 as i32 == *exp_ff
                && to.1 as i32 == *exp_tr
                && to.0 as i32 == *exp_tf
            {
                if let chess_engine::GameResult::Continuing(new_board) =
                    game.board.play_move(player_move)
                {
                    game.board = new_board;
                }
                mark_correct(game);
            } else {
                if let chess_engine::GameResult::Continuing(new_board) =
                    game.board.play_move(player_move)
                {
                    game.board = new_board;
                }
                mark_wrong(game);
            }
        }

        PuzzleSolution::MateInTwo { move1, move2: _ } => {
            if game.move_number_in_puzzle == 0 {
                let (exp_fr, exp_ff, exp_tr, exp_tf) = move1;
                if from.1 as i32 == *exp_fr
                    && from.0 as i32 == *exp_ff
                    && to.1 as i32 == *exp_tr
                    && to.0 as i32 == *exp_tf
                {
                    if let chess_engine::GameResult::Continuing(new_board) =
                        game.board.play_move(player_move)
                    {
                        game.board = new_board;
                        game.move_number_in_puzzle = 1;
                        game.last_move = Some((from, to));
                        game.selected_square = None;
                        game.legal_move_destinations.clear();
                        game.ai_thinking = true;
                        game.ai_think_ticks = 0;
                        game.ai_think_target = 8; // ~0.8 seconds
                    }
                } else {
                    if let chess_engine::GameResult::Continuing(new_board) =
                        game.board.play_move(player_move)
                    {
                        game.board = new_board;
                    }
                    mark_wrong(game);
                }
            } else {
                // Second move: check if it results in checkmate
                match game.board.play_move(player_move) {
                    chess_engine::GameResult::Victory(_) => {
                        mark_correct(game);
                    }
                    chess_engine::GameResult::Continuing(new_board) => {
                        if new_board.is_checkmate() {
                            mark_correct(game);
                        } else {
                            game.board = new_board;
                            mark_wrong(game);
                        }
                    }
                    _ => {
                        mark_wrong(game);
                    }
                }
            }
        }
    }
}

fn mark_correct(game: &mut ChessPuzzleGame) {
    game.puzzle_state = PuzzleState::Correct;
    game.puzzles_solved += 1;
    game.puzzles_attempted += 1;
    game.feedback_ticks = 10; // 1 second
    game.selected_square = None;
    game.legal_move_destinations.clear();
}

fn mark_wrong(game: &mut ChessPuzzleGame) {
    game.puzzle_state = PuzzleState::Wrong;
    game.puzzles_attempted += 1;
    game.feedback_ticks = 10; // 1 second
    game.selected_square = None;
    game.legal_move_destinations.clear();
}

/// Process AI thinking tick for mate-in-2 intermediate response.
pub fn process_ai_thinking(game: &mut ChessPuzzleGame) {
    if !game.ai_thinking {
        return;
    }

    game.ai_think_ticks += 1;

    // Compute AI response on first tick
    if game.ai_pending_board.is_none() {
        let (best_move, _, _) = game.board.get_best_next_move(3);
        if let chess_engine::GameResult::Continuing(new_board) = game.board.play_move(best_move) {
            game.ai_pending_board = Some(new_board);
        }
    }

    // Apply after delay
    if game.ai_think_ticks >= game.ai_think_target {
        if let Some(new_board) = game.ai_pending_board.take() {
            game.board = new_board;
        }
        game.ai_thinking = false;
        game.ai_think_ticks = 0;
        game.puzzle_state = PuzzleState::Solving;
    }
}

/// Process feedback countdown and advance to next puzzle.
pub fn tick_feedback(game: &mut ChessPuzzleGame) {
    if game.puzzle_state != PuzzleState::Correct && game.puzzle_state != PuzzleState::Wrong {
        return;
    }

    if game.feedback_ticks > 0 {
        game.feedback_ticks -= 1;
        return;
    }

    // Feedback period over -- advance
    game.current_puzzle_index += 1;

    // Check win
    if game.puzzles_solved >= game.target_score {
        game.game_result = Some(ChessPuzzleResult::Win);
        return;
    }

    // Check if winning is still possible
    let remaining = game
        .total_puzzles
        .saturating_sub(game.current_puzzle_index as u32);
    if game.puzzles_solved + remaining < game.target_score {
        game.game_result = Some(ChessPuzzleResult::Loss);
        return;
    }

    // Check if all puzzles exhausted
    if game.current_puzzle_index >= game.puzzle_order.len() {
        game.game_result = Some(ChessPuzzleResult::Loss);
        return;
    }

    // Set up next puzzle
    setup_current_puzzle(game);
}

/// Start a chess puzzle game (used in integration tests).
#[allow(dead_code)]
pub fn start_chess_puzzle_game(state: &mut GameState, difficulty: ChessPuzzleDifficulty) {
    let mut game = ChessPuzzleGame::new(difficulty);
    setup_current_puzzle(&mut game);
    state.active_minigame = Some(ActiveMinigame::ChessPuzzle(Box::new(game)));
    state.challenge_menu.close();
}

/// Apply game result: update stats, grant rewards, and add combat log entries.
pub fn apply_game_result(state: &mut GameState) -> Option<crate::challenges::MinigameWinInfo> {
    use crate::challenges::menu::DifficultyInfo;
    use crate::challenges::{apply_challenge_rewards, GameResultInfo};

    let game = match state.active_minigame.as_ref() {
        Some(ActiveMinigame::ChessPuzzle(g)) => g,
        _ => return None,
    };
    let result = game.game_result?;
    let difficulty = game.difficulty;
    let reward = difficulty.reward();
    let puzzles_solved = game.puzzles_solved;
    let puzzles_attempted = game.puzzles_attempted;
    let total_puzzles = game.total_puzzles;

    // Stats tracking
    state.chess_puzzle_stats.sessions_played += 1;
    state.chess_puzzle_stats.puzzles_solved += puzzles_solved;
    state.chess_puzzle_stats.puzzles_attempted += puzzles_attempted;

    let (won, loss_message) = match result {
        ChessPuzzleResult::Win => {
            state.chess_puzzle_stats.sessions_won += 1;
            (true, "")
        }
        ChessPuzzleResult::Loss => {
            state.chess_puzzle_stats.sessions_lost += 1;
            (
                false,
                "The puzzle master shakes their head slowly and fades away.",
            )
        }
    };

    // Add puzzle-specific log line before the generic reward helper
    if won {
        state.combat_state.add_log_entry(
            format!(
                "\u{265E} Puzzle mastery! Solved {}/{} puzzles.",
                puzzles_solved, total_puzzles
            ),
            false,
            true,
        );
    }

    apply_challenge_rewards(
        state,
        GameResultInfo {
            won,
            game_type: "chess_puzzle",
            difficulty_str: difficulty.difficulty_str(),
            reward,
            icon: "\u{265E}",
            win_message: "Chess Puzzle challenge complete!",
            loss_message,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::challenges::menu::{ChallengeType, PendingChallenge};

    fn make_chess_puzzle_challenge() -> PendingChallenge {
        PendingChallenge {
            challenge_type: ChallengeType::ChessPuzzle,
            title: "Chess Puzzle Challenge".to_string(),
            icon: "\u{265E}",
            description: "Test".to_string(),
        }
    }

    #[test]
    fn test_start_chess_puzzle_game() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.challenge_menu.open();
        start_chess_puzzle_game(&mut state, ChessPuzzleDifficulty::Novice);
        assert!(matches!(
            state.active_minigame,
            Some(ActiveMinigame::ChessPuzzle(_))
        ));
        assert!(!state.challenge_menu.is_open);
    }

    #[test]
    fn test_setup_current_puzzle_sets_board() {
        let mut game = ChessPuzzleGame::new(ChessPuzzleDifficulty::Novice);
        setup_current_puzzle(&mut game);
        assert_eq!(game.puzzle_state, PuzzleState::Solving);
        assert!(game.player_is_white);
        assert!(game.selected_square.is_none());
        assert!(game.legal_move_destinations.is_empty());
    }

    #[test]
    fn test_process_input_cursor_movement() {
        let mut game = ChessPuzzleGame::new(ChessPuzzleDifficulty::Novice);
        setup_current_puzzle(&mut game);
        let initial = game.cursor;

        process_input(&mut game, ChessPuzzleInput::Up);
        assert_eq!(game.cursor, (initial.0, initial.1 + 1));

        process_input(&mut game, ChessPuzzleInput::Down);
        assert_eq!(game.cursor, initial);

        process_input(&mut game, ChessPuzzleInput::Right);
        assert_eq!(game.cursor, (initial.0 + 1, initial.1));

        process_input(&mut game, ChessPuzzleInput::Left);
        assert_eq!(game.cursor, initial);
    }

    #[test]
    fn test_process_input_blocked_during_ai_thinking() {
        let mut game = ChessPuzzleGame::new(ChessPuzzleDifficulty::Novice);
        setup_current_puzzle(&mut game);
        game.ai_thinking = true;
        let initial_cursor = game.cursor;

        let handled = process_input(&mut game, ChessPuzzleInput::Up);
        assert!(!handled);
        assert_eq!(game.cursor, initial_cursor);
    }

    #[test]
    fn test_process_input_blocked_during_feedback() {
        let mut game = ChessPuzzleGame::new(ChessPuzzleDifficulty::Novice);
        setup_current_puzzle(&mut game);
        game.puzzle_state = PuzzleState::Correct;
        let initial_cursor = game.cursor;

        let handled = process_input(&mut game, ChessPuzzleInput::Up);
        assert!(!handled);
        assert_eq!(game.cursor, initial_cursor);
    }

    #[test]
    fn test_forfeit_single_esc() {
        let mut game = ChessPuzzleGame::new(ChessPuzzleDifficulty::Novice);
        setup_current_puzzle(&mut game);

        process_input(&mut game, ChessPuzzleInput::Forfeit);
        assert!(game.forfeit_pending);
        assert!(game.game_result.is_none());
    }

    #[test]
    fn test_forfeit_double_esc() {
        let mut game = ChessPuzzleGame::new(ChessPuzzleDifficulty::Novice);
        setup_current_puzzle(&mut game);

        process_input(&mut game, ChessPuzzleInput::Forfeit);
        process_input(&mut game, ChessPuzzleInput::Forfeit);
        assert_eq!(game.game_result, Some(ChessPuzzleResult::Loss));
    }

    #[test]
    fn test_forfeit_cancelled_by_other_key() {
        let mut game = ChessPuzzleGame::new(ChessPuzzleDifficulty::Novice);
        setup_current_puzzle(&mut game);

        process_input(&mut game, ChessPuzzleInput::Forfeit);
        assert!(game.forfeit_pending);

        process_input(&mut game, ChessPuzzleInput::Other);
        assert!(!game.forfeit_pending);
        assert!(game.game_result.is_none());
    }

    #[test]
    fn test_select_piece() {
        let mut game = ChessPuzzleGame::new(ChessPuzzleDifficulty::Novice);
        setup_current_puzzle(&mut game);

        // Find a piece belonging to the player and select it
        // After Scholar's Mate setup, white has pieces. Let's find the queen on h5.
        game.cursor = (7, 4); // h5 = file 7, rank 4
        process_input(&mut game, ChessPuzzleInput::Select);

        // The queen should be selected (if it's there)
        if game.selected_square.is_some() {
            assert!(!game.legal_move_destinations.is_empty());
        }
    }

    #[test]
    fn test_clear_selection_with_esc() {
        let mut game = ChessPuzzleGame::new(ChessPuzzleDifficulty::Novice);
        setup_current_puzzle(&mut game);

        // Select a piece
        game.cursor = (7, 4);
        process_input(&mut game, ChessPuzzleInput::Select);

        if game.selected_square.is_some() {
            process_input(&mut game, ChessPuzzleInput::Forfeit);
            assert!(game.selected_square.is_none());
            assert!(!game.forfeit_pending);
        }
    }

    #[test]
    fn test_tick_feedback_decrements() {
        let mut game = ChessPuzzleGame::new(ChessPuzzleDifficulty::Novice);
        setup_current_puzzle(&mut game);
        game.puzzle_state = PuzzleState::Correct;
        game.feedback_ticks = 5;
        game.puzzles_solved = 1;

        tick_feedback(&mut game);
        assert_eq!(game.feedback_ticks, 4);
        assert_eq!(game.puzzle_state, PuzzleState::Correct);
    }

    #[test]
    fn test_tick_feedback_advances_puzzle() {
        let mut game = ChessPuzzleGame::new(ChessPuzzleDifficulty::Novice);
        setup_current_puzzle(&mut game);
        game.puzzle_state = PuzzleState::Wrong;
        game.feedback_ticks = 0;
        game.puzzles_attempted = 1;

        let initial_index = game.current_puzzle_index;
        tick_feedback(&mut game);

        assert_eq!(game.current_puzzle_index, initial_index + 1);
        assert_eq!(game.puzzle_state, PuzzleState::Solving);
    }

    #[test]
    fn test_tick_feedback_win_condition() {
        let mut game = ChessPuzzleGame::new(ChessPuzzleDifficulty::Novice);
        setup_current_puzzle(&mut game);
        game.puzzle_state = PuzzleState::Correct;
        game.feedback_ticks = 0;
        game.puzzles_solved = game.target_score;

        tick_feedback(&mut game);
        assert_eq!(game.game_result, Some(ChessPuzzleResult::Win));
    }

    #[test]
    fn test_tick_feedback_loss_impossible_to_win() {
        let mut game = ChessPuzzleGame::new(ChessPuzzleDifficulty::Novice);
        setup_current_puzzle(&mut game);
        game.puzzle_state = PuzzleState::Wrong;
        game.feedback_ticks = 0;
        game.puzzles_solved = 0;
        game.puzzles_attempted = game.total_puzzles;
        // Set current_puzzle_index such that remaining puzzles can't reach target
        game.current_puzzle_index = game.total_puzzles as usize - 1;

        tick_feedback(&mut game);
        assert_eq!(game.game_result, Some(ChessPuzzleResult::Loss));
    }

    #[test]
    fn test_apply_win_result() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.prestige_rank = 5;
        let mut game = ChessPuzzleGame::new(ChessPuzzleDifficulty::Master);
        setup_current_puzzle(&mut game);
        game.game_result = Some(ChessPuzzleResult::Win);
        game.puzzles_solved = 3;
        game.puzzles_attempted = 4;
        state.active_minigame = Some(ActiveMinigame::ChessPuzzle(Box::new(game)));

        let processed = apply_game_result(&mut state);
        assert!(processed.is_some());
        assert_eq!(state.prestige_rank, 10); // 5 + 5 (Master reward)
        assert_eq!(state.chess_puzzle_stats.sessions_won, 1);
        assert_eq!(state.chess_puzzle_stats.puzzles_solved, 3);
        assert!(state.active_minigame.is_none());
    }

    #[test]
    fn test_apply_loss_result() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.prestige_rank = 5;
        let mut game = ChessPuzzleGame::new(ChessPuzzleDifficulty::Novice);
        setup_current_puzzle(&mut game);
        game.game_result = Some(ChessPuzzleResult::Loss);
        state.active_minigame = Some(ActiveMinigame::ChessPuzzle(Box::new(game)));

        let processed = apply_game_result(&mut state);
        assert!(processed.is_none());
        assert_eq!(state.prestige_rank, 5); // Unchanged
        assert_eq!(state.chess_puzzle_stats.sessions_lost, 1);
    }

    #[test]
    fn test_process_ai_thinking() {
        let mut game = ChessPuzzleGame::new(ChessPuzzleDifficulty::Novice);
        // Use a default board (opening position) so AI can make a continuing move.
        // The puzzle setup positions may be near-checkmate, which causes Victory
        // instead of Continuing when the AI plays.
        game.board = chess_engine::Board::default();

        // Simulate being in AI thinking state
        game.ai_thinking = true;
        game.ai_think_target = 3;
        game.ai_think_ticks = 0;

        // Tick 1: computes AI move
        process_ai_thinking(&mut game);
        assert!(game.ai_thinking);
        assert!(game.ai_pending_board.is_some());

        // Tick 2
        process_ai_thinking(&mut game);
        assert!(game.ai_thinking);

        // Tick 3: delay reached, applies move
        process_ai_thinking(&mut game);
        assert!(!game.ai_thinking);
        assert!(game.ai_pending_board.is_none());
        assert_eq!(game.puzzle_state, PuzzleState::Solving);
    }

    #[test]
    fn test_process_ai_thinking_does_nothing_when_not_thinking() {
        let mut game = ChessPuzzleGame::new(ChessPuzzleDifficulty::Novice);
        setup_current_puzzle(&mut game);
        game.ai_thinking = false;

        process_ai_thinking(&mut game);
        // Should be a no-op
        assert!(!game.ai_thinking);
    }

    #[test]
    fn test_accepting_challenge_starts_game() {
        let mut state = GameState::new("Test".to_string(), 0);
        state
            .challenge_menu
            .add_challenge(make_chess_puzzle_challenge());
        state.challenge_menu.open();

        assert!(state
            .challenge_menu
            .has_challenge(&ChallengeType::ChessPuzzle));
        assert!(state.active_minigame.is_none());

        start_chess_puzzle_game(&mut state, ChessPuzzleDifficulty::Novice);

        assert!(matches!(
            state.active_minigame,
            Some(ActiveMinigame::ChessPuzzle(_))
        ));
        assert!(!state.challenge_menu.is_open);
    }

    #[test]
    fn test_forfeit_counts_as_loss_in_stats() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.prestige_rank = 5;

        let mut game = ChessPuzzleGame::new(ChessPuzzleDifficulty::Novice);
        setup_current_puzzle(&mut game);
        game.game_result = Some(ChessPuzzleResult::Loss);
        state.active_minigame = Some(ActiveMinigame::ChessPuzzle(Box::new(game)));

        let processed = apply_game_result(&mut state);
        assert!(processed.is_none());
        assert_eq!(state.chess_puzzle_stats.sessions_lost, 1);
        assert_eq!(state.chess_puzzle_stats.sessions_won, 0);
        assert_eq!(state.prestige_rank, 5);
    }
}
