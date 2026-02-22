//! Nine Men's Morris AI: minimax with alpha-beta pruning and make/unmake optimization.

use super::logic::{apply_move, get_legal_moves};
use super::{MorrisDifficulty, MorrisGame, MorrisMove, MorrisPhase, MorrisResult, Player, MILLS};
use rand::{Rng, RngExt};

/// Undo information for reversing a move during search.
/// This avoids expensive game.clone() in minimax.
#[derive(Debug, Clone)]
struct MoveUndo {
    mv: MorrisMove,
    prev_must_capture: bool,
    prev_phase: MorrisPhase,
    prev_player: Player,
    prev_game_result: Option<MorrisResult>,
    /// For captures: the player whose piece was captured
    captured_player: Option<Player>,
    /// Whether a mill was formed (triggering must_capture)
    formed_mill: bool,
}

/// Apply a move for AI search and return undo information.
/// This is optimized to avoid cloning the game state.
fn make_move_for_search(game: &mut MorrisGame, mv: MorrisMove) -> MoveUndo {
    let prev_must_capture = game.must_capture;
    let prev_phase = game.phase;
    let prev_player = game.current_player;
    let prev_game_result = game.game_result;

    let mut undo = MoveUndo {
        mv,
        prev_must_capture,
        prev_phase,
        prev_player,
        prev_game_result,
        captured_player: None,
        formed_mill: false,
    };

    match mv {
        MorrisMove::Place(pos) => {
            game.board[pos] = Some(game.current_player);

            match game.current_player {
                Player::Human => {
                    game.pieces_to_place.0 -= 1;
                    game.pieces_on_board.0 += 1;
                }
                Player::Ai => {
                    game.pieces_to_place.1 -= 1;
                    game.pieces_on_board.1 += 1;
                }
            }

            if game.forms_mill(pos, game.current_player) {
                game.must_capture = true;
                undo.formed_mill = true;
            } else {
                end_turn_for_search(game);
            }
        }
        MorrisMove::Move { from, to } => {
            game.board[from] = None;
            game.board[to] = Some(game.current_player);

            if game.forms_mill(to, game.current_player) {
                game.must_capture = true;
                undo.formed_mill = true;
            } else {
                end_turn_for_search(game);
            }
        }
        MorrisMove::Capture(pos) => {
            let opponent = match game.current_player {
                Player::Human => Player::Ai,
                Player::Ai => Player::Human,
            };

            undo.captured_player = Some(opponent);
            game.board[pos] = None;

            match opponent {
                Player::Human => game.pieces_on_board.0 -= 1,
                Player::Ai => game.pieces_on_board.1 -= 1,
            }

            game.must_capture = false;
            end_turn_for_search(game);
        }
    }

    undo
}

/// Reverse a move using undo information.
fn unmake_move(game: &mut MorrisGame, undo: MoveUndo) {
    // Restore previous state
    game.must_capture = undo.prev_must_capture;
    game.phase = undo.prev_phase;
    game.current_player = undo.prev_player;
    game.game_result = undo.prev_game_result;

    match undo.mv {
        MorrisMove::Place(pos) => {
            game.board[pos] = None;

            match undo.prev_player {
                Player::Human => {
                    game.pieces_to_place.0 += 1;
                    game.pieces_on_board.0 -= 1;
                }
                Player::Ai => {
                    game.pieces_to_place.1 += 1;
                    game.pieces_on_board.1 -= 1;
                }
            }
        }
        MorrisMove::Move { from, to } => {
            game.board[to] = None;
            game.board[from] = Some(undo.prev_player);
        }
        MorrisMove::Capture(pos) => {
            if let Some(captured) = undo.captured_player {
                game.board[pos] = Some(captured);

                match captured {
                    Player::Human => game.pieces_on_board.0 += 1,
                    Player::Ai => game.pieces_on_board.1 += 1,
                }
            }
        }
    }
}

/// Simplified end_turn for AI search (no UI state, no AI thinking trigger).
fn end_turn_for_search(game: &mut MorrisGame) {
    // Check phase transition
    if game.phase == MorrisPhase::Placing
        && game.pieces_to_place.0 == 0
        && game.pieces_to_place.1 == 0
    {
        game.phase = MorrisPhase::Moving;
    }

    // Switch players
    game.current_player = match game.current_player {
        Player::Human => Player::Ai,
        Player::Ai => Player::Human,
    };

    // Check win conditions (simplified - no UI side effects)
    if game.phase != MorrisPhase::Placing {
        if game.pieces_on_board.0 < 3 && game.pieces_to_place.0 == 0 {
            game.game_result = Some(MorrisResult::Loss);
        } else if game.pieces_on_board.1 < 3 && game.pieces_to_place.1 == 0 {
            game.game_result = Some(MorrisResult::Win);
        } else {
            // Check for no legal moves
            let legal_moves = get_legal_moves(game);
            if legal_moves.is_empty() && !game.must_capture {
                game.game_result = Some(match game.current_player {
                    Player::Human => MorrisResult::Loss,
                    Player::Ai => MorrisResult::Win,
                });
            }
        }
    }
}

/// Process AI thinking tick.
pub fn process_ai_thinking<R: Rng>(game: &mut MorrisGame, rng: &mut R) {
    if !game.ai_thinking {
        return;
    }

    game.ai_think_ticks += 1;

    // Compute AI move on first tick
    if game.ai_pending_move.is_none() {
        game.ai_pending_move = get_ai_move(game, rng);
        game.ai_think_target = calculate_think_ticks(rng);
    }

    // Apply move after delay
    if game.ai_think_ticks >= game.ai_think_target {
        if let Some(mv) = game.ai_pending_move.take() {
            apply_move(game, mv);
        }

        // If AI formed a mill and must capture, keep thinking for the capture move
        if game.must_capture && game.current_player == Player::Ai {
            game.ai_think_ticks = 0;
            game.ai_pending_move = None; // Will compute capture on next tick
            return;
        }

        game.ai_thinking = false;
        game.ai_think_ticks = 0;
    }
}

/// Calculate variable AI thinking time in ticks (1-3 seconds at 100ms/tick)
pub fn calculate_think_ticks<R: Rng>(rng: &mut R) -> u32 {
    rng.random_range(10..=30)
}

/// Get the best AI move based on difficulty
pub fn get_ai_move<R: Rng>(game: &MorrisGame, rng: &mut R) -> Option<MorrisMove> {
    let legal_moves = get_legal_moves(game);
    if legal_moves.is_empty() {
        return None;
    }

    // Random move chance for Novice
    if rng.random::<f64>() < game.difficulty.random_move_chance() {
        let idx = rng.random_range(0..legal_moves.len());
        return Some(legal_moves[idx]);
    }

    // Use minimax to find best move (with make/unmake optimization)
    let depth = game.difficulty.search_depth();
    let mut game_mut = game.clone(); // Single clone at the root
    let mut best_move = None;
    let mut best_score = i32::MIN;

    for mv in legal_moves.iter() {
        let undo = make_move_for_search(&mut game_mut, *mv);
        // After AI makes a move, it's Human's turn - Human minimizes (maximizing=false)
        // No negation needed: standard minimax with evaluation always from AI's perspective
        let score = minimax_optimized(&mut game_mut, depth - 1, i32::MIN, i32::MAX, false);
        unmake_move(&mut game_mut, undo);

        if score > best_score {
            best_score = score;
            best_move = Some(*mv);
        }
    }

    best_move
}

/// Optimized minimax with alpha-beta pruning using make/unmake pattern.
/// This avoids cloning the game state at each node.
fn minimax_optimized(
    game: &mut MorrisGame,
    depth: i32,
    mut alpha: i32,
    mut beta: i32,
    maximizing: bool,
) -> i32 {
    // Terminal conditions
    if depth == 0 || game.game_result.is_some() {
        return evaluate_board(game);
    }

    let legal_moves = get_legal_moves(game);
    if legal_moves.is_empty() {
        return evaluate_board(game);
    }

    if maximizing {
        let mut max_eval = i32::MIN;
        for mv in legal_moves {
            let undo = make_move_for_search(game, mv);
            let eval = minimax_optimized(game, depth - 1, alpha, beta, false);
            unmake_move(game, undo);

            max_eval = max_eval.max(eval);
            alpha = alpha.max(eval);
            if beta <= alpha {
                break;
            }
        }
        max_eval
    } else {
        let mut min_eval = i32::MAX;
        for mv in legal_moves {
            let undo = make_move_for_search(game, mv);
            let eval = minimax_optimized(game, depth - 1, alpha, beta, true);
            unmake_move(game, undo);

            min_eval = min_eval.min(eval);
            beta = beta.min(eval);
            if beta <= alpha {
                break;
            }
        }
        min_eval
    }
}

/// Evaluate the board position from AI's perspective
fn evaluate_board(game: &MorrisGame) -> i32 {
    // Check for terminal states
    if let Some(result) = &game.game_result {
        return match result {
            MorrisResult::Win => -10000, // Human wins = bad for AI
            MorrisResult::Loss => 10000, // Human loses = good for AI
        };
    }

    let mut score: i32 = 0;

    // Piece count difference (each piece worth 100 points)
    let human_pieces = game.pieces_on_board.0 as i32 + game.pieces_to_place.0 as i32;
    let ai_pieces = game.pieces_on_board.1 as i32 + game.pieces_to_place.1 as i32;
    score += (ai_pieces - human_pieces) * 100;

    // Mill count (each mill worth 50 points)
    let human_mills = count_mills(game, Player::Human);
    let ai_mills = count_mills(game, Player::Ai);
    score += (ai_mills - human_mills) * 50;

    // Potential mills (two pieces with empty third position) worth 25 points
    let human_potential = count_potential_mills(game, Player::Human);
    let ai_potential = count_potential_mills(game, Player::Ai);
    score += (ai_potential - human_potential) * 25;

    // Mobility (number of legal moves) worth 5 points each
    // Only count during moving phase
    if game.phase != MorrisPhase::Placing {
        let human_mobility = count_mobility(game, Player::Human);
        let ai_mobility = count_mobility(game, Player::Ai);
        score += (ai_mobility - human_mobility) * 5;
    }

    // Bonus for having pieces in strategic positions (center positions)
    // Positions 4, 10, 13, 19 are more connected
    let strategic_positions = [4, 10, 13, 19];
    for &pos in &strategic_positions {
        match game.board[pos] {
            Some(Player::Human) => score -= 10,
            Some(Player::Ai) => score += 10,
            None => {}
        }
    }

    score
}

/// Count the number of complete mills for a player
fn count_mills(game: &MorrisGame, player: Player) -> i32 {
    let mut count = 0;
    for mill in MILLS.iter() {
        if mill.iter().all(|&pos| game.board[pos] == Some(player)) {
            count += 1;
        }
    }
    count
}

/// Count the number of potential mills (two pieces and one empty) for a player
fn count_potential_mills(game: &MorrisGame, player: Player) -> i32 {
    let mut count = 0;
    for mill in MILLS.iter() {
        let player_count = mill
            .iter()
            .filter(|&&pos| game.board[pos] == Some(player))
            .count();
        let empty_count = mill
            .iter()
            .filter(|&&pos| game.board[pos].is_none())
            .count();
        if player_count == 2 && empty_count == 1 {
            count += 1;
        }
    }
    count
}

/// Count mobility (number of possible moves) for a player
fn count_mobility(game: &MorrisGame, player: Player) -> i32 {
    let can_fly = game.can_fly(player);
    let mut moves = 0;

    for (from, &cell) in game.board.iter().enumerate() {
        if cell != Some(player) {
            continue;
        }

        if can_fly {
            // Count all empty positions
            moves += game.board.iter().filter(|&&c| c.is_none()).count() as i32;
        } else {
            // Count adjacent empty positions
            for &to in super::ADJACENCIES[from].iter() {
                if game.board[to].is_none() {
                    moves += 1;
                }
            }
        }
    }

    moves
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_ai_returns_legal_move() {
        let game = MorrisGame::new(MorrisDifficulty::Novice);
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        let legal_moves = get_legal_moves(&game);
        let ai_move = get_ai_move(&game, &mut rng);

        assert!(ai_move.is_some());
        let mv = ai_move.unwrap();
        assert!(legal_moves.contains(&mv));
    }

    #[test]
    fn test_ai_different_difficulties() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        // Only test Novice and Apprentice (depth 2-3) on empty board.
        // Master/Journeyman (depth 4-5) are very slow with 24 legal moves.
        // Higher difficulties are covered by test_ai_blocks_obvious_mill
        // and test_ai_completes_own_mill_over_blocking (constrained boards).
        for difficulty in [MorrisDifficulty::Novice, MorrisDifficulty::Apprentice] {
            let mut game = MorrisGame::new(difficulty);
            game.current_player = Player::Ai;

            let legal_moves = get_legal_moves(&game);
            let ai_move = get_ai_move(&game, &mut rng);

            assert!(
                ai_move.is_some(),
                "AI at {:?} should return a move",
                difficulty
            );
            assert!(
                legal_moves.contains(&ai_move.unwrap()),
                "AI at {:?} should return a legal move",
                difficulty
            );
        }
    }

    #[test]
    fn test_ai_blocks_obvious_mill() {
        // Set up a position where Human has 2 in a row (positions 0, 1)
        // AI should block at position 2 to prevent mill [0, 1, 2]
        let mut game = MorrisGame::new(MorrisDifficulty::Master);
        game.board[0] = Some(Player::Human);
        game.board[1] = Some(Player::Human);
        game.pieces_on_board = (2, 0);
        game.pieces_to_place = (7, 9);
        game.current_player = Player::Ai;

        // Use a seeded RNG to ensure deterministic behavior (no random moves)
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        let ai_move = get_ai_move(&game, &mut rng);

        // AI should block by placing at position 2
        assert_eq!(
            ai_move,
            Some(MorrisMove::Place(2)),
            "AI should block the obvious mill at position 2"
        );
    }

    #[test]
    fn test_ai_completes_own_mill_over_blocking() {
        // Set up a position where AI can complete its own mill (3, 4, 5)
        // AI should prefer completing its mill over blocking Human's
        let mut game = MorrisGame::new(MorrisDifficulty::Master);
        // Human has 2 in row at 0, 1 (threatens 2)
        game.board[0] = Some(Player::Human);
        game.board[1] = Some(Player::Human);
        // AI has 2 in row at 3, 4 (can complete at 5)
        game.board[3] = Some(Player::Ai);
        game.board[4] = Some(Player::Ai);
        game.pieces_on_board = (2, 2);
        game.pieces_to_place = (7, 7);
        game.current_player = Player::Ai;

        let mut rng = rand::rngs::StdRng::seed_from_u64(42);

        let ai_move = get_ai_move(&game, &mut rng);

        // AI should complete its own mill at position 5 (attacking > defending)
        assert_eq!(
            ai_move,
            Some(MorrisMove::Place(5)),
            "AI should complete its own mill rather than just blocking"
        );
    }

    #[test]
    fn test_evaluate_board_piece_advantage() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.phase = MorrisPhase::Moving;
        game.pieces_to_place = (0, 0);
        game.pieces_on_board = (3, 5);

        // AI has more pieces, should be positive
        let score = evaluate_board(&game);
        assert!(score > 0, "AI with more pieces should have positive score");
    }

    #[test]
    fn test_evaluate_board_human_advantage() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.phase = MorrisPhase::Moving;
        game.pieces_to_place = (0, 0);
        game.pieces_on_board = (5, 3);

        // Human has more pieces, should be negative for AI
        let score = evaluate_board(&game);
        assert!(
            score < 0,
            "Human with more pieces should have negative score for AI"
        );
    }

    #[test]
    fn test_calculate_think_ticks_range() {
        let mut rng = ChaCha8Rng::seed_from_u64(1000);

        for _ in 0..100 {
            let ticks = calculate_think_ticks(&mut rng);
            assert!(
                (10..=30).contains(&ticks),
                "Think ticks {} should be in range 10-30",
                ticks
            );
        }
    }

    #[test]
    fn test_process_ai_thinking_not_thinking() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.ai_thinking = false;
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        process_ai_thinking(&mut game, &mut rng);

        // Should remain not thinking
        assert!(!game.ai_thinking);
    }

    #[test]
    fn test_process_ai_thinking_computes_move() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.current_player = Player::Ai;
        game.ai_thinking = true;
        game.ai_think_ticks = 0;
        game.ai_pending_move = None;
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        // First tick should compute the move
        process_ai_thinking(&mut game, &mut rng);

        assert!(game.ai_thinking); // Should still be thinking (waiting for delay)
        assert!(game.ai_pending_move.is_some());
        assert!(game.ai_think_target >= 10);
    }
}
