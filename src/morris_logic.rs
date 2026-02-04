//! Nine Men's Morris game logic: discovery, AI moves, and game resolution.

use crate::challenge_menu::{ChallengeType, PendingChallenge};
use crate::game_state::GameState;
use crate::morris::{
    MorrisDifficulty, MorrisGame, MorrisMove, MorrisPhase, MorrisResult, Player, ADJACENCIES, MILLS,
};
use rand::Rng;

/// Chance per tick to discover a morris challenge (~2 hour average)
/// At 10 ticks/sec, 0.000014 chance/tick = 71,429 ticks = 2 hours average
pub const MORRIS_DISCOVERY_CHANCE: f64 = 0.000014;

/// Create a morris challenge for the challenge menu
pub fn create_morris_challenge() -> PendingChallenge {
    PendingChallenge {
        challenge_type: ChallengeType::Morris,
        title: "Nine Men's Morris".to_string(),
        icon: "\u{25CB}", // White circle
        description: "An elderly sage arranges nine white stones on a weathered board. \
            \"The game of mills,\" they say. \"Three in a row captures. Shall we play?\""
            .to_string(),
    }
}

/// Check if morris discovery conditions are met and roll for discovery
pub fn try_discover_morris<R: Rng>(state: &mut GameState, rng: &mut R) -> bool {
    // Requirements: P1+, not in dungeon, not fishing, not in chess, not in morris, no pending morris
    if state.prestige_rank < 1
        || state.active_dungeon.is_some()
        || state.active_fishing.is_some()
        || state.active_chess.is_some()
        || state.active_morris.is_some()
        || state.challenge_menu.has_morris_challenge()
    {
        return false;
    }

    if rng.gen::<f64>() < MORRIS_DISCOVERY_CHANCE {
        state
            .challenge_menu
            .add_challenge(create_morris_challenge());
        true
    } else {
        false
    }
}

/// Start a morris game with the selected difficulty
pub fn start_morris_game(state: &mut GameState, difficulty: MorrisDifficulty) {
    state.active_morris = Some(MorrisGame::new(difficulty));
    state.challenge_menu.close();
}

/// Get all legal moves for the current player
pub fn get_legal_moves(game: &MorrisGame) -> Vec<MorrisMove> {
    if game.game_result.is_some() {
        return Vec::new();
    }

    if game.must_capture {
        return get_capture_moves(game, game.current_player);
    }

    match game.phase {
        MorrisPhase::Placing => get_placing_moves(game),
        MorrisPhase::Moving | MorrisPhase::Flying => get_movement_moves(game, game.current_player),
    }
}

/// Get legal placement moves (during Placing phase)
fn get_placing_moves(game: &MorrisGame) -> Vec<MorrisMove> {
    let mut moves = Vec::new();

    // Can only place if the current player has pieces left
    let pieces_left = game.pieces_to_place_for(game.current_player);
    if pieces_left == 0 {
        return moves;
    }

    // Can place on any empty position
    for (pos, &cell) in game.board.iter().enumerate() {
        if cell.is_none() {
            moves.push(MorrisMove::Place(pos));
        }
    }

    moves
}

/// Get legal movement moves (during Moving or Flying phase)
fn get_movement_moves(game: &MorrisGame, player: Player) -> Vec<MorrisMove> {
    let mut moves = Vec::new();
    let can_fly = game.can_fly(player);

    // Find all pieces belonging to the player
    for (from, &cell) in game.board.iter().enumerate() {
        if cell != Some(player) {
            continue;
        }

        if can_fly {
            // Flying: can move to any empty position
            for (to, &target) in game.board.iter().enumerate() {
                if target.is_none() {
                    moves.push(MorrisMove::Move { from, to });
                }
            }
        } else {
            // Normal movement: can only move to adjacent empty positions
            for &to in ADJACENCIES[from].iter() {
                if game.board[to].is_none() {
                    moves.push(MorrisMove::Move { from, to });
                }
            }
        }
    }

    moves
}

/// Get legal capture moves (after forming a mill)
fn get_capture_moves(game: &MorrisGame, player: Player) -> Vec<MorrisMove> {
    let mut moves = Vec::new();
    let opponent = match player {
        Player::Human => Player::Ai,
        Player::Ai => Player::Human,
    };

    // Check if all opponent pieces are in mills
    let all_in_mills = game
        .board
        .iter()
        .enumerate()
        .filter(|(_, &cell)| cell == Some(opponent))
        .all(|(pos, _)| game.is_in_mill(pos, opponent));

    for (pos, &cell) in game.board.iter().enumerate() {
        if cell != Some(opponent) {
            continue;
        }

        // Can only capture pieces not in mills, unless ALL are in mills
        if all_in_mills || !game.is_in_mill(pos, opponent) {
            moves.push(MorrisMove::Capture(pos));
        }
    }

    moves
}

/// Apply a move to the game state
pub fn apply_move(game: &mut MorrisGame, mv: MorrisMove) {
    match mv {
        MorrisMove::Place(pos) => {
            // Place piece
            game.board[pos] = Some(game.current_player);

            // Update piece counts
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

            // Check for mill formation
            if game.forms_mill(pos, game.current_player) {
                game.must_capture = true;
            } else {
                end_turn(game);
            }
        }
        MorrisMove::Move { from, to } => {
            // Move piece
            game.board[from] = None;
            game.board[to] = Some(game.current_player);

            // Check for mill formation at destination
            if game.forms_mill(to, game.current_player) {
                game.must_capture = true;
            } else {
                end_turn(game);
            }
        }
        MorrisMove::Capture(pos) => {
            // Remove captured piece
            let opponent = match game.current_player {
                Player::Human => Player::Ai,
                Player::Ai => Player::Human,
            };

            game.board[pos] = None;
            match opponent {
                Player::Human => game.pieces_on_board.0 -= 1,
                Player::Ai => game.pieces_on_board.1 -= 1,
            }

            game.must_capture = false;
            end_turn(game);
        }
    }
}

/// End the current turn and switch players
fn end_turn(game: &mut MorrisGame) {
    // Check phase transition: from Placing to Moving
    if game.phase == MorrisPhase::Placing
        && game.pieces_to_place.0 == 0
        && game.pieces_to_place.1 == 0
    {
        game.phase = MorrisPhase::Moving;
    }

    // Check for flying phase for each player (handled dynamically via can_fly)

    // Switch players
    game.current_player = match game.current_player {
        Player::Human => Player::Ai,
        Player::Ai => Player::Human,
    };

    // Clear selection
    game.selected_position = None;

    // Check win condition
    check_win_condition(game);

    // Start AI thinking if it's AI's turn and game is not over
    if game.current_player == Player::Ai && game.game_result.is_none() {
        game.ai_thinking = true;
        game.ai_think_ticks = 0;
        game.ai_pending_move = None;
    }
}

/// Check if the game has ended
fn check_win_condition(game: &mut MorrisGame) {
    if game.game_result.is_some() {
        return;
    }

    // Only check win conditions after placing phase
    if game.phase == MorrisPhase::Placing {
        return;
    }

    // Check for loss by piece count (less than 3 pieces)
    if game.pieces_on_board.0 < 3 && game.pieces_to_place.0 == 0 {
        game.game_result = Some(MorrisResult::Loss);
        return;
    }
    if game.pieces_on_board.1 < 3 && game.pieces_to_place.1 == 0 {
        game.game_result = Some(MorrisResult::Win);
        return;
    }

    // Check for loss by no legal moves (current player cannot move)
    let legal_moves = get_legal_moves(game);
    if legal_moves.is_empty() && !game.must_capture {
        game.game_result = Some(match game.current_player {
            Player::Human => MorrisResult::Loss,
            Player::Ai => MorrisResult::Win,
        });
    }
}

/// Process AI thinking tick, returns true if AI made a move
pub fn process_ai_thinking<R: Rng>(game: &mut MorrisGame, rng: &mut R) -> bool {
    if !game.ai_thinking {
        return false;
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
        game.ai_thinking = false;
        game.ai_think_ticks = 0;
        return true;
    }

    false
}

/// Calculate variable AI thinking time in ticks (1-3 seconds at 100ms/tick)
pub fn calculate_think_ticks<R: Rng>(rng: &mut R) -> u32 {
    rng.gen_range(10..=30)
}

/// Get the best AI move based on difficulty
pub fn get_ai_move<R: Rng>(game: &MorrisGame, rng: &mut R) -> Option<MorrisMove> {
    let legal_moves = get_legal_moves(game);
    if legal_moves.is_empty() {
        return None;
    }

    // Random move chance for Novice
    if rng.gen::<f64>() < game.difficulty.random_move_chance() {
        let idx = rng.gen_range(0..legal_moves.len());
        return Some(legal_moves[idx]);
    }

    // Use minimax to find best move
    let depth = game.difficulty.search_depth();
    let mut best_move = None;
    let mut best_score = i32::MIN;

    for mv in legal_moves.iter() {
        let mut game_copy = game.clone();
        apply_move(&mut game_copy, *mv);

        // If AI was in must_capture state, this might trigger another move need
        // We evaluate from the opponent's perspective after our move
        let score = -minimax(&game_copy, depth - 1, i32::MIN, i32::MAX, false);

        if score > best_score {
            best_score = score;
            best_move = Some(*mv);
        }
    }

    best_move
}

/// Minimax algorithm with alpha-beta pruning
fn minimax(game: &MorrisGame, depth: i32, mut alpha: i32, mut beta: i32, maximizing: bool) -> i32 {
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
            let mut game_copy = game.clone();
            apply_move(&mut game_copy, mv);
            let eval = minimax(&game_copy, depth - 1, alpha, beta, false);
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
            let mut game_copy = game.clone();
            apply_move(&mut game_copy, mv);
            let eval = minimax(&game_copy, depth - 1, alpha, beta, true);
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
            MorrisResult::Forfeit => 10000,
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
            for &to in ADJACENCIES[from].iter() {
                if game.board[to].is_none() {
                    moves += 1;
                }
            }
        }
    }

    moves
}

/// Apply game result: grant prestige on win
pub fn apply_game_result(state: &mut GameState) -> Option<(MorrisResult, u32)> {
    let game = state.active_morris.as_ref()?;
    let result = game.game_result?;
    let difficulty = game.difficulty;

    let prestige_gained = match result {
        MorrisResult::Win => {
            let reward = difficulty.reward_prestige();
            state.prestige_rank += reward;
            reward
        }
        MorrisResult::Loss | MorrisResult::Forfeit => 0,
    };

    state.active_morris = None;
    Some((result, prestige_gained))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============ Challenge Creation Tests ============

    #[test]
    fn test_create_challenge() {
        let challenge = create_morris_challenge();
        assert_eq!(challenge.title, "Nine Men's Morris");
        assert_eq!(challenge.icon, "\u{25CB}");
        assert!(matches!(challenge.challenge_type, ChallengeType::Morris));
    }

    // ============ Placing Moves Tests ============

    #[test]
    fn test_placing_moves() {
        let game = MorrisGame::new(MorrisDifficulty::Novice);

        let moves = get_placing_moves(&game);

        // All 24 positions should be available for placement
        assert_eq!(moves.len(), 24);

        // All moves should be Place moves
        for mv in moves {
            assert!(matches!(mv, MorrisMove::Place(_)));
        }
    }

    #[test]
    fn test_placing_moves_with_occupied() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.board[0] = Some(Player::Human);
        game.board[5] = Some(Player::Ai);
        game.board[10] = Some(Player::Human);

        let moves = get_placing_moves(&game);

        // 24 - 3 occupied = 21 available
        assert_eq!(moves.len(), 21);

        // Should not include occupied positions
        for mv in moves {
            if let MorrisMove::Place(pos) = mv {
                assert!(pos != 0 && pos != 5 && pos != 10);
            }
        }
    }

    // ============ Apply Move Tests ============

    #[test]
    fn test_apply_place_move() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        assert_eq!(game.pieces_to_place.0, 9);
        assert_eq!(game.pieces_on_board.0, 0);

        apply_move(&mut game, MorrisMove::Place(0));

        assert_eq!(game.board[0], Some(Player::Human));
        assert_eq!(game.pieces_to_place.0, 8);
        assert_eq!(game.pieces_on_board.0, 1);
        // Turn should switch since no mill was formed
        assert_eq!(game.current_player, Player::Ai);
    }

    // ============ Mill and Capture Tests ============

    #[test]
    fn test_mill_triggers_capture() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);

        // Set up two pieces of a mill
        game.board[0] = Some(Player::Human);
        game.board[1] = Some(Player::Human);
        game.pieces_on_board.0 = 2;
        game.pieces_to_place.0 = 7;

        // Place an AI piece to be captured
        game.board[5] = Some(Player::Ai);
        game.pieces_on_board.1 = 1;
        game.pieces_to_place.1 = 8;

        // Complete the mill at position 2
        apply_move(&mut game, MorrisMove::Place(2));

        // Should be in must_capture state
        assert!(game.must_capture);
        // Should still be human's turn
        assert_eq!(game.current_player, Player::Human);
        // Mill should be formed
        assert!(game.is_in_mill(0, Player::Human));
        assert!(game.is_in_mill(1, Player::Human));
        assert!(game.is_in_mill(2, Player::Human));
    }

    #[test]
    fn test_capture_move() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);

        // Set up a completed mill
        game.board[0] = Some(Player::Human);
        game.board[1] = Some(Player::Human);
        game.board[2] = Some(Player::Human);
        game.pieces_on_board.0 = 3;

        // AI piece to capture
        game.board[5] = Some(Player::Ai);
        game.pieces_on_board.1 = 1;
        game.pieces_to_place.1 = 8;

        game.must_capture = true;

        // Capture the AI piece
        apply_move(&mut game, MorrisMove::Capture(5));

        assert!(game.board[5].is_none());
        assert_eq!(game.pieces_on_board.1, 0);
        assert!(!game.must_capture);
        // Turn should switch after capture
        assert_eq!(game.current_player, Player::Ai);
    }

    #[test]
    fn test_capture_moves_respect_mill_protection() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.current_player = Player::Human;
        game.must_capture = true;

        // AI has a mill (protected)
        game.board[3] = Some(Player::Ai);
        game.board[4] = Some(Player::Ai);
        game.board[5] = Some(Player::Ai);

        // AI has an unprotected piece
        game.board[10] = Some(Player::Ai);

        let capture_moves = get_capture_moves(&game, Player::Human);

        // Should only be able to capture the unprotected piece
        assert_eq!(capture_moves.len(), 1);
        assert!(matches!(capture_moves[0], MorrisMove::Capture(10)));
    }

    #[test]
    fn test_capture_all_in_mills() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.current_player = Player::Human;
        game.must_capture = true;

        // AI only has pieces in mills
        game.board[3] = Some(Player::Ai);
        game.board[4] = Some(Player::Ai);
        game.board[5] = Some(Player::Ai);

        let capture_moves = get_capture_moves(&game, Player::Human);

        // When all pieces are in mills, can capture any
        assert_eq!(capture_moves.len(), 3);
    }

    // ============ Movement Move Tests ============

    #[test]
    fn test_movement_moves() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.phase = MorrisPhase::Moving;
        game.pieces_to_place = (0, 0);

        // Place a piece at position 0 (adjacent to 1 and 9)
        game.board[0] = Some(Player::Human);
        game.pieces_on_board.0 = 5; // Not flying

        let moves = get_movement_moves(&game, Player::Human);

        // Position 0 is adjacent to 1 and 9, both empty
        assert_eq!(moves.len(), 2);

        // Both moves should be from position 0
        for mv in moves {
            if let MorrisMove::Move { from, to } = mv {
                assert_eq!(from, 0);
                assert!(to == 1 || to == 9);
            } else {
                panic!("Expected Move variant");
            }
        }
    }

    #[test]
    fn test_movement_blocked() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.phase = MorrisPhase::Moving;
        game.pieces_to_place = (0, 0);

        // Place a piece at position 0
        game.board[0] = Some(Player::Human);
        // Block both adjacent positions
        game.board[1] = Some(Player::Ai);
        game.board[9] = Some(Player::Ai);
        game.pieces_on_board.0 = 5;

        let moves = get_movement_moves(&game, Player::Human);

        // No valid moves from position 0
        assert_eq!(moves.len(), 0);
    }

    // ============ Flying Move Tests ============

    #[test]
    fn test_flying_moves() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.phase = MorrisPhase::Moving;
        game.pieces_to_place = (0, 0);
        game.pieces_on_board = (3, 5); // Human has exactly 3 pieces

        // Place human pieces
        game.board[0] = Some(Player::Human);
        game.board[5] = Some(Player::Human);
        game.board[10] = Some(Player::Human);

        // Place AI pieces
        game.board[1] = Some(Player::Ai);
        game.board[9] = Some(Player::Ai);
        game.board[6] = Some(Player::Ai);
        game.board[11] = Some(Player::Ai);
        game.board[13] = Some(Player::Ai);

        assert!(game.can_fly(Player::Human));

        let moves = get_movement_moves(&game, Player::Human);

        // Human should be able to fly from each of 3 pieces to any of the 16 empty positions
        // 3 pieces * 16 empty = 48 total moves
        assert_eq!(moves.len(), 48);
    }

    // ============ AI Tests ============

    #[test]
    fn test_ai_returns_legal_move() {
        let game = MorrisGame::new(MorrisDifficulty::Novice);
        let mut rng = rand::thread_rng();

        let legal_moves = get_legal_moves(&game);
        let ai_move = get_ai_move(&game, &mut rng);

        assert!(ai_move.is_some());
        let mv = ai_move.unwrap();
        assert!(legal_moves.contains(&mv));
    }

    #[test]
    fn test_ai_different_difficulties() {
        let mut rng = rand::thread_rng();

        for difficulty in MorrisDifficulty::ALL {
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

    // ============ Win Condition Tests ============

    #[test]
    fn test_win_by_piece_count() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.phase = MorrisPhase::Moving;
        game.pieces_to_place = (0, 0);
        game.pieces_on_board = (5, 2); // AI has only 2 pieces

        // Place pieces
        game.board[0] = Some(Player::Human);
        game.board[1] = Some(Player::Human);
        game.board[2] = Some(Player::Human);
        game.board[3] = Some(Player::Human);
        game.board[4] = Some(Player::Human);
        game.board[5] = Some(Player::Ai);
        game.board[6] = Some(Player::Ai);

        check_win_condition(&mut game);

        // Human should win because AI has < 3 pieces
        assert_eq!(game.game_result, Some(MorrisResult::Win));
    }

    #[test]
    fn test_loss_by_piece_count() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.phase = MorrisPhase::Moving;
        game.pieces_to_place = (0, 0);
        game.pieces_on_board = (2, 5); // Human has only 2 pieces

        // Place pieces
        game.board[0] = Some(Player::Ai);
        game.board[1] = Some(Player::Ai);
        game.board[2] = Some(Player::Ai);
        game.board[3] = Some(Player::Ai);
        game.board[4] = Some(Player::Ai);
        game.board[5] = Some(Player::Human);
        game.board[6] = Some(Player::Human);

        check_win_condition(&mut game);

        // Human should lose
        assert_eq!(game.game_result, Some(MorrisResult::Loss));
    }

    #[test]
    fn test_no_win_during_placing() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.phase = MorrisPhase::Placing;
        game.pieces_to_place = (7, 7);
        game.pieces_on_board = (2, 2);

        check_win_condition(&mut game);

        // Should not trigger win during placing phase
        assert!(game.game_result.is_none());
    }

    // ============ Phase Transition Tests ============

    #[test]
    fn test_phase_transition_to_moving() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);

        // Set up end of placing phase
        game.phase = MorrisPhase::Placing;
        game.pieces_to_place = (1, 0);
        game.pieces_on_board = (8, 9);

        // Place pieces in a pattern that doesn't form a mill when placing at position 23
        // Position 23 is only in mills [21, 22, 23] and [2, 14, 23]
        // So we avoid placing human pieces at (21, 22) and (2, 14)
        game.board[0] = Some(Player::Human);
        game.board[1] = Some(Player::Ai);
        game.board[3] = Some(Player::Human);
        game.board[4] = Some(Player::Ai);
        game.board[5] = Some(Player::Human);
        game.board[6] = Some(Player::Ai);
        game.board[7] = Some(Player::Human);
        game.board[8] = Some(Player::Ai);
        game.board[9] = Some(Player::Human);
        game.board[10] = Some(Player::Ai);
        game.board[11] = Some(Player::Human);
        game.board[12] = Some(Player::Ai);
        game.board[13] = Some(Player::Human);
        game.board[15] = Some(Player::Ai);
        game.board[16] = Some(Player::Ai);
        game.board[17] = Some(Player::Ai);
        game.board[18] = Some(Player::Ai);
        // Position 23 will be placed, it connects to 14 and 22 in mills
        // We leave 14, 21, 22, 2 empty or non-Human to avoid mill

        // Last placement at position 23 (doesn't form a mill since 21, 22 aren't Human)
        apply_move(&mut game, MorrisMove::Place(23));

        // Should transition to Moving phase
        assert_eq!(game.phase, MorrisPhase::Moving);
    }

    // ============ Start Game Tests ============

    #[test]
    fn test_start_morris_game() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.challenge_menu.open();

        start_morris_game(&mut state, MorrisDifficulty::Journeyman);

        assert!(state.active_morris.is_some());
        assert!(!state.challenge_menu.is_open);
        assert_eq!(
            state.active_morris.as_ref().unwrap().difficulty,
            MorrisDifficulty::Journeyman
        );
    }

    // ============ Result Application Tests ============

    #[test]
    fn test_apply_win_result() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.prestige_rank = 5;

        let mut game = MorrisGame::new(MorrisDifficulty::Master);
        game.game_result = Some(MorrisResult::Win);
        state.active_morris = Some(game);

        let result = apply_game_result(&mut state);

        assert!(result.is_some());
        let (morris_result, prestige) = result.unwrap();
        assert_eq!(morris_result, MorrisResult::Win);
        assert_eq!(prestige, 5); // Master reward
        assert_eq!(state.prestige_rank, 10);
        assert!(state.active_morris.is_none());
    }

    #[test]
    fn test_apply_loss_result() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.prestige_rank = 5;

        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.game_result = Some(MorrisResult::Loss);
        state.active_morris = Some(game);

        let result = apply_game_result(&mut state);

        let (morris_result, prestige) = result.unwrap();
        assert_eq!(morris_result, MorrisResult::Loss);
        assert_eq!(prestige, 0);
        assert_eq!(state.prestige_rank, 5); // Unchanged
        assert!(state.active_morris.is_none());
    }

    #[test]
    fn test_apply_forfeit_result() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.prestige_rank = 5;

        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.game_result = Some(MorrisResult::Forfeit);
        state.active_morris = Some(game);

        let result = apply_game_result(&mut state);

        let (morris_result, prestige) = result.unwrap();
        assert_eq!(morris_result, MorrisResult::Forfeit);
        assert_eq!(prestige, 0);
        assert!(state.active_morris.is_none());
    }

    // ============ Evaluation Tests ============

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

    // ============ Think Ticks Tests ============

    #[test]
    fn test_calculate_think_ticks_range() {
        let mut rng = rand::thread_rng();

        for _ in 0..100 {
            let ticks = calculate_think_ticks(&mut rng);
            assert!(
                (10..=30).contains(&ticks),
                "Think ticks {} should be in range 10-30",
                ticks
            );
        }
    }

    // ============ Legal Moves Comprehensive Tests ============

    #[test]
    fn test_get_legal_moves_placing() {
        let game = MorrisGame::new(MorrisDifficulty::Novice);
        let moves = get_legal_moves(&game);

        assert_eq!(moves.len(), 24);
        for mv in moves {
            assert!(matches!(mv, MorrisMove::Place(_)));
        }
    }

    #[test]
    fn test_get_legal_moves_capture() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.must_capture = true;
        game.board[10] = Some(Player::Ai);

        let moves = get_legal_moves(&game);

        assert_eq!(moves.len(), 1);
        assert!(matches!(moves[0], MorrisMove::Capture(10)));
    }

    #[test]
    fn test_get_legal_moves_game_over() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.game_result = Some(MorrisResult::Win);

        let moves = get_legal_moves(&game);

        assert!(moves.is_empty());
    }

    // ============ AI Thinking Process Tests ============

    #[test]
    fn test_process_ai_thinking_not_thinking() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.ai_thinking = false;
        let mut rng = rand::thread_rng();

        let moved = process_ai_thinking(&mut game, &mut rng);

        assert!(!moved);
    }

    #[test]
    fn test_process_ai_thinking_computes_move() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.current_player = Player::Ai;
        game.ai_thinking = true;
        game.ai_think_ticks = 0;
        game.ai_pending_move = None;
        let mut rng = rand::thread_rng();

        // First tick should compute the move
        let moved = process_ai_thinking(&mut game, &mut rng);

        assert!(!moved); // Should not have applied yet
        assert!(game.ai_pending_move.is_some());
        assert!(game.ai_think_target >= 10);
    }
}
