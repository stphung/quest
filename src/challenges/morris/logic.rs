//! Nine Men's Morris game logic: AI moves, and game resolution.

use super::{
    CursorDirection, MorrisDifficulty, MorrisGame, MorrisMove, MorrisPhase, MorrisResult, Player,
    ADJACENCIES, MILLS,
};
use crate::challenges::ActiveMinigame;
use crate::core::game_state::GameState;
use rand::Rng;

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

/// Input actions for the Morris game (UI-agnostic).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MorrisInput {
    Up,
    Down,
    Left,
    Right,
    Select, // Enter - select piece, place, move, or capture
    Cancel, // Esc - clear selection or forfeit
    Other,
}

/// Process a key input during active Morris game.
/// Returns true if the input was handled.
/// Does nothing if AI is thinking.
pub fn process_input(game: &mut MorrisGame, input: MorrisInput) -> bool {
    // Don't process input while AI is thinking
    if game.ai_thinking {
        return false;
    }

    match input {
        MorrisInput::Up => game.move_cursor(CursorDirection::Up),
        MorrisInput::Down => game.move_cursor(CursorDirection::Down),
        MorrisInput::Left => game.move_cursor(CursorDirection::Left),
        MorrisInput::Right => game.move_cursor(CursorDirection::Right),
        MorrisInput::Select => {
            process_human_enter(game);
        }
        MorrisInput::Cancel => {
            process_cancel(game);
        }
        MorrisInput::Other => {
            // Any other key cancels forfeit pending
            game.forfeit_pending = false;
        }
    }
    true
}

/// Process Esc key: clear selection or initiate/confirm forfeit.
fn process_cancel(game: &mut MorrisGame) {
    if game.forfeit_pending {
        game.game_result = Some(MorrisResult::Forfeit);
    } else if game.selected_position.is_some() {
        game.clear_selection();
        game.forfeit_pending = false;
    } else {
        game.forfeit_pending = true;
    }
}

/// Start a morris game with the selected difficulty
pub fn start_morris_game(state: &mut GameState, difficulty: MorrisDifficulty) {
    state.active_minigame = Some(ActiveMinigame::Morris(MorrisGame::new(difficulty)));
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

/// Process human player pressing Enter at current cursor position.
/// Handles placing, moving, and capturing based on game phase.
pub fn process_human_enter(game: &mut MorrisGame) {
    let cursor = game.cursor;

    // If must capture, try to capture at cursor
    if game.must_capture {
        let capture_moves = get_legal_moves(game);
        if capture_moves
            .iter()
            .any(|m| matches!(m, MorrisMove::Capture(pos) if *pos == cursor))
        {
            apply_move(game, MorrisMove::Capture(cursor));
        }
        return;
    }

    // During placing phase, place at cursor if empty
    if game.phase == MorrisPhase::Placing {
        if game.board[cursor].is_none() {
            apply_move(game, MorrisMove::Place(cursor));
        }
        return;
    }

    // During moving/flying phase
    if let Some(selected) = game.selected_position {
        // Already selected a piece - try to move to cursor
        let legal_moves = get_legal_moves(game);
        if legal_moves.iter().any(
            |m| matches!(m, MorrisMove::Move { from, to } if *from == selected && *to == cursor),
        ) {
            apply_move(
                game,
                MorrisMove::Move {
                    from: selected,
                    to: cursor,
                },
            );
        } else if game.board[cursor] == Some(Player::Human) {
            // Clicked on another human piece - select it instead
            game.selected_position = Some(cursor);
        } else {
            // Invalid move - clear selection
            game.clear_selection();
        }
    } else {
        // No piece selected - try to select piece at cursor
        if game.board[cursor] == Some(Player::Human) {
            game.selected_position = Some(cursor);
        }
    }
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

        // If AI formed a mill and must capture, keep thinking for the capture move
        if game.must_capture && game.current_player == Player::Ai {
            game.ai_think_ticks = 0;
            game.ai_pending_move = None; // Will compute capture on next tick
            return true;
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

/// Apply game result: grant rewards and add combat log entries.
/// Returns true if a result was processed.
pub fn apply_game_result(state: &mut GameState) -> bool {
    use crate::challenges::menu::DifficultyInfo;

    let game = match state.active_minigame.as_ref() {
        Some(ActiveMinigame::Morris(g)) => g,
        _ => return false,
    };
    let result = match game.game_result {
        Some(r) => r,
        None => return false,
    };
    let reward = game.difficulty.reward();

    match result {
        MorrisResult::Win => {
            // XP reward
            let xp_for_level =
                crate::core::game_logic::xp_for_next_level(state.character_level.max(1));
            let xp_gained = (xp_for_level as f64 * reward.xp_percent as f64 / 100.0) as u64;
            let xp_gained = xp_gained.max(100); // Floor of 100 XP
            state.character_xp += xp_gained;

            // Fishing rank reward (capped at 30, preserves fish progress)
            let fishing_rank_up = if reward.fishing_ranks > 0 && state.fishing.rank < 30 {
                state.fishing.rank = (state.fishing.rank + reward.fishing_ranks).min(30);
                true
            } else {
                false
            };

            // Prestige reward (if any)
            state.prestige_rank += reward.prestige_ranks;

            // Combat log entries
            state.combat_state.add_log_entry(
                "○ Victory! The sage bows with respect.".to_string(),
                false,
                true,
            );
            state
                .combat_state
                .add_log_entry(format!("○ +{} XP", xp_gained), false, true);
            if fishing_rank_up {
                state.combat_state.add_log_entry(
                    format!(
                        "○ Fishing rank up! Now rank {}: {}",
                        state.fishing.rank,
                        state.fishing.rank_name()
                    ),
                    false,
                    true,
                );
            }
        }
        MorrisResult::Loss => {
            state.combat_state.add_log_entry(
                "○ The sage nods knowingly and departs.".to_string(),
                false,
                true,
            );
        }
        MorrisResult::Forfeit => {
            state.combat_state.add_log_entry(
                "○ You concede. The sage gathers their stones quietly.".to_string(),
                false,
                true,
            );
        }
    }

    state.active_minigame = None;
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

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

        assert!(matches!(
            state.active_minigame,
            Some(ActiveMinigame::Morris(_))
        ));
        assert!(!state.challenge_menu.is_open);
        if let Some(ActiveMinigame::Morris(game)) = &state.active_minigame {
            assert_eq!(game.difficulty, MorrisDifficulty::Journeyman);
        } else {
            panic!("expected morris");
        }
    }

    // ============ Result Application Tests ============

    #[test]
    fn test_apply_win_result() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.character_level = 10;

        let mut game = MorrisGame::new(MorrisDifficulty::Master);
        game.game_result = Some(MorrisResult::Win);
        state.active_minigame = Some(ActiveMinigame::Morris(game));

        let old_xp = state.character_xp;
        let old_fishing_rank = state.fishing.rank;
        let processed = apply_game_result(&mut state);

        assert!(processed);
        // Master = 200% of xp_for_next_level(10) = 6324
        assert_eq!(state.character_xp, old_xp + 6324);
        // Master grants +1 fishing rank
        assert_eq!(state.fishing.rank, old_fishing_rank + 1);
        assert!(state.active_minigame.is_none());
    }

    #[test]
    fn test_apply_win_novice_no_fishing_rank() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.character_level = 5;

        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.game_result = Some(MorrisResult::Win);
        state.active_minigame = Some(ActiveMinigame::Morris(game));

        let old_fishing_rank = state.fishing.rank;
        let processed = apply_game_result(&mut state);

        assert!(processed);
        assert_eq!(state.fishing.rank, old_fishing_rank); // Novice grants no fishing rank
    }

    #[test]
    fn test_apply_win_master_fishing_rank_capped_at_30() {
        let mut state = GameState::new("Test".to_string(), 0);
        state.character_level = 10;
        state.fishing.rank = 30; // Already max

        let mut game = MorrisGame::new(MorrisDifficulty::Master);
        game.game_result = Some(MorrisResult::Win);
        state.active_minigame = Some(ActiveMinigame::Morris(game));

        let processed = apply_game_result(&mut state);

        assert!(processed);
        assert_eq!(state.fishing.rank, 30); // Capped at max
    }

    #[test]
    fn test_apply_loss_result() {
        let mut state = GameState::new("Test".to_string(), 0);

        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.game_result = Some(MorrisResult::Loss);
        state.active_minigame = Some(ActiveMinigame::Morris(game));

        let old_xp = state.character_xp;
        let processed = apply_game_result(&mut state);

        assert!(processed);
        assert_eq!(state.character_xp, old_xp); // Unchanged
        assert!(state.active_minigame.is_none());
    }

    #[test]
    fn test_apply_forfeit_result() {
        let mut state = GameState::new("Test".to_string(), 0);

        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.game_result = Some(MorrisResult::Forfeit);
        state.active_minigame = Some(ActiveMinigame::Morris(game));

        let old_xp = state.character_xp;
        let processed = apply_game_result(&mut state);

        assert!(processed);
        assert_eq!(state.character_xp, old_xp); // Unchanged
        assert!(state.active_minigame.is_none());
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

    // ============ Human Input Tests ============

    #[test]
    fn test_process_human_enter_places_piece_in_placing_phase() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        assert_eq!(game.phase, MorrisPhase::Placing);
        assert!(game.board[0].is_none());

        game.cursor = 0;
        process_human_enter(&mut game);

        assert_eq!(game.board[0], Some(Player::Human));
        // Turn should switch to AI
        assert_eq!(game.current_player, Player::Ai);
    }

    #[test]
    fn test_process_human_enter_ignores_occupied_position_in_placing() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.board[0] = Some(Player::Ai);

        game.cursor = 0;
        let pieces_before = game.pieces_to_place.0;
        process_human_enter(&mut game);

        // Nothing should change - position was occupied
        assert_eq!(game.board[0], Some(Player::Ai));
        assert_eq!(game.pieces_to_place.0, pieces_before);
    }

    #[test]
    fn test_process_human_enter_selects_piece_in_moving_phase() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        // Set up moving phase with human piece
        game.phase = MorrisPhase::Moving;
        game.pieces_to_place = (0, 0);
        game.board[0] = Some(Player::Human);
        game.pieces_on_board.0 = 3;

        game.cursor = 0;
        assert!(game.selected_position.is_none());

        process_human_enter(&mut game);

        assert_eq!(game.selected_position, Some(0));
    }

    #[test]
    fn test_process_human_enter_moves_selected_piece() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        // Set up moving phase with human piece selected
        game.phase = MorrisPhase::Moving;
        game.pieces_to_place = (0, 0);
        game.board[0] = Some(Player::Human);
        game.pieces_on_board.0 = 3;
        game.selected_position = Some(0);

        // Position 1 is adjacent to position 0 in the board layout
        game.cursor = 1;

        process_human_enter(&mut game);

        // Piece should have moved from 0 to 1
        assert!(game.board[0].is_none());
        assert_eq!(game.board[1], Some(Player::Human));
        assert!(game.selected_position.is_none());
    }

    #[test]
    fn test_process_human_enter_captures_opponent_piece() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.must_capture = true;
        game.board[5] = Some(Player::Ai);
        game.pieces_on_board.1 = 3;

        game.cursor = 5;
        process_human_enter(&mut game);

        // Opponent piece should be captured
        assert!(game.board[5].is_none());
        assert!(!game.must_capture);
    }

    // ============ Process Input Tests ============

    #[test]
    fn test_process_input_blocked_during_ai_thinking() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.ai_thinking = true;
        let old_cursor = game.cursor;

        let handled = process_input(&mut game, MorrisInput::Up);

        assert!(!handled);
        assert_eq!(game.cursor, old_cursor);
    }

    #[test]
    fn test_process_input_cursor_movement() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.cursor = 4; // Central position

        // Test all directions
        process_input(&mut game, MorrisInput::Up);
        // Cursor should change (exact position depends on board layout)
        let after_up = game.cursor;

        game.cursor = 4;
        process_input(&mut game, MorrisInput::Down);
        let after_down = game.cursor;

        game.cursor = 4;
        process_input(&mut game, MorrisInput::Left);
        let after_left = game.cursor;

        game.cursor = 4;
        process_input(&mut game, MorrisInput::Right);
        let after_right = game.cursor;

        // At least some movements should change the cursor
        assert!(
            after_up != 4 || after_down != 4 || after_left != 4 || after_right != 4,
            "At least one direction should move the cursor"
        );
    }

    #[test]
    fn test_process_input_select_places_piece() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        assert_eq!(game.phase, MorrisPhase::Placing);
        game.cursor = 0;

        process_input(&mut game, MorrisInput::Select);

        assert_eq!(game.board[0], Some(Player::Human));
    }

    #[test]
    fn test_process_input_cancel_initiates_forfeit() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        assert!(!game.forfeit_pending);

        process_input(&mut game, MorrisInput::Cancel);

        assert!(game.forfeit_pending);
    }

    #[test]
    fn test_process_input_cancel_confirms_forfeit() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.forfeit_pending = true;

        process_input(&mut game, MorrisInput::Cancel);

        assert_eq!(game.game_result, Some(MorrisResult::Forfeit));
    }

    #[test]
    fn test_process_input_cancel_clears_selection_first() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.phase = MorrisPhase::Moving;
        game.selected_position = Some(5);

        process_input(&mut game, MorrisInput::Cancel);

        // Should clear selection, not initiate forfeit
        assert!(game.selected_position.is_none());
        assert!(!game.forfeit_pending);
    }

    #[test]
    fn test_process_input_other_cancels_forfeit_pending() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.forfeit_pending = true;

        process_input(&mut game, MorrisInput::Other);

        assert!(!game.forfeit_pending);
    }

    #[test]
    fn test_process_input_returns_true_when_handled() {
        // Test each input returns true independently
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        assert!(process_input(&mut game, MorrisInput::Up));

        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        assert!(process_input(&mut game, MorrisInput::Down));

        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        assert!(process_input(&mut game, MorrisInput::Left));

        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        assert!(process_input(&mut game, MorrisInput::Right));

        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        assert!(process_input(&mut game, MorrisInput::Select));

        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        assert!(process_input(&mut game, MorrisInput::Cancel));

        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        assert!(process_input(&mut game, MorrisInput::Other));
    }
}
