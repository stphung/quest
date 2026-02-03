//! Chess game logic: discovery, AI moves, and game resolution.

use crate::challenge_menu::{ChallengeType, PendingChallenge};
use crate::chess::{ChessDifficulty, ChessGame, ChessResult};
use crate::game_state::GameState;
use chess_engine::Evaluate;
use rand::Rng;

/// Chance per tick to discover a chess challenge (0.5% = ~30-60 min)
pub const CHESS_DISCOVERY_CHANCE: f64 = 0.005;

/// Create a chess challenge for the challenge menu
pub fn create_chess_challenge() -> PendingChallenge {
    PendingChallenge {
        challenge_type: ChallengeType::Chess,
        title: "Chess Challenge".to_string(),
        icon: "♟",
        description: "A mysterious figure challenges you to a game of chess.".to_string(),
    }
}

/// Check if chess discovery conditions are met and roll for discovery
pub fn try_discover_chess<R: Rng>(state: &mut GameState, rng: &mut R) -> bool {
    // Requirements: P1+, not in dungeon, not fishing, not in chess, no pending chess
    if state.prestige_rank < 1
        || state.active_dungeon.is_some()
        || state.active_fishing.is_some()
        || state.active_chess.is_some()
        || state.challenge_menu.has_chess_challenge()
    {
        return false;
    }

    if rng.gen::<f64>() < CHESS_DISCOVERY_CHANCE {
        state.challenge_menu.add_challenge(create_chess_challenge());
        true
    } else {
        false
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
pub fn apply_move_to_board(board: &chess_engine::Board, m: chess_engine::Move) -> Option<chess_engine::Board> {
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
        }
        game.ai_think_target = calculate_think_ticks(&game.board, rng);
    }

    // Apply move after delay
    if game.ai_think_ticks >= game.ai_think_target {
        if let Some(new_board) = game.ai_pending_board.take() {
            game.board = new_board;
        }
        game.ai_thinking = false;
        game.ai_think_ticks = 0;
        check_game_over(game);
        return true;
    }

    false
}

/// Check if the game is over (no legal moves = checkmate or stalemate)
pub fn check_game_over(game: &mut ChessGame) {
    let legal_moves = game.board.get_legal_moves();
    if legal_moves.is_empty() && game.game_result.is_none() {
        // Simplified: no legal moves means loss for current side
        game.game_result = Some(if game.player_is_white {
            ChessResult::Loss
        } else {
            ChessResult::Win
        });
    }
}

/// Apply game result: update stats and grant prestige on win
pub fn apply_game_result(state: &mut GameState) -> Option<(ChessResult, u32)> {
    let game = state.active_chess.as_ref()?;
    let result = game.game_result?;
    let difficulty = game.difficulty;

    state.chess_stats.games_played += 1;

    let prestige_gained = match result {
        ChessResult::Win => {
            state.chess_stats.games_won += 1;
            let reward = difficulty.reward_prestige();
            state.prestige_rank += reward;
            state.chess_stats.prestige_earned += reward;
            reward
        }
        ChessResult::Loss | ChessResult::Forfeit => {
            state.chess_stats.games_lost += 1;
            0
        }
        ChessResult::Draw => {
            state.chess_stats.games_drawn += 1;
            0
        }
    };

    state.active_chess = None;
    Some((result, prestige_gained))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_challenge() {
        let challenge = create_chess_challenge();
        assert_eq!(challenge.title, "Chess Challenge");
        assert_eq!(challenge.icon, "♟");
        assert!(matches!(challenge.challenge_type, ChallengeType::Chess));
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

        let result = apply_game_result(&mut state);
        assert!(result.is_some());
        let (chess_result, prestige) = result.unwrap();
        assert_eq!(chess_result, ChessResult::Win);
        assert_eq!(prestige, 5);
        assert_eq!(state.prestige_rank, 10);
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

        let result = apply_game_result(&mut state);
        let (chess_result, prestige) = result.unwrap();
        assert_eq!(chess_result, ChessResult::Loss);
        assert_eq!(prestige, 0);
        assert_eq!(state.prestige_rank, 5);
        assert_eq!(state.chess_stats.games_lost, 1);
    }
}
