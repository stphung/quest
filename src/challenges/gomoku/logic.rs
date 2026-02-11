//! Gomoku game logic and AI.

use super::{GomokuDifficulty, GomokuGame, GomokuResult, Player, BOARD_SIZE};
use crate::challenges::ActiveMinigame;
use crate::core::game_state::GameState;
use rand::seq::SliceRandom;
use rand::Rng;

/// Input actions for the Gomoku game (UI-agnostic).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GomokuInput {
    Up,
    Down,
    Left,
    Right,
    PlaceStone,
    Forfeit,
    Other,
}

/// Process a key input during active Gomoku game.
/// Returns true if the input was handled.
/// Does nothing if AI is thinking.
pub fn process_input(game: &mut GomokuGame, input: GomokuInput) -> bool {
    // Don't process input while AI is thinking
    if game.ai_thinking {
        return false;
    }

    // Handle forfeit confirmation (double-Esc pattern)
    if game.forfeit_pending {
        match input {
            GomokuInput::Forfeit => {
                game.game_result = Some(GomokuResult::Loss);
            }
            _ => {
                game.forfeit_pending = false;
            }
        }
        return true;
    }

    // Normal game input
    match input {
        GomokuInput::Up => game.move_cursor(-1, 0),
        GomokuInput::Down => game.move_cursor(1, 0),
        GomokuInput::Left => game.move_cursor(0, -1),
        GomokuInput::Right => game.move_cursor(0, 1),
        GomokuInput::PlaceStone => {
            process_human_move(game);
        }
        GomokuInput::Forfeit => {
            game.forfeit_pending = true;
        }
        GomokuInput::Other => {}
    }
    true
}

/// Directions to check for lines: (row_delta, col_delta)
const DIRECTIONS: [(i32, i32); 4] = [
    (0, 1),  // Horizontal
    (1, 0),  // Vertical
    (1, 1),  // Diagonal down-right
    (1, -1), // Diagonal down-left
];

/// Check if placing at (row, col) creates 5+ in a row for the given player.
/// Assumes the stone is already placed.
pub fn check_win(
    board: &[[Option<Player>; BOARD_SIZE]; BOARD_SIZE],
    row: usize,
    col: usize,
    player: Player,
) -> bool {
    for (dr, dc) in DIRECTIONS {
        let count = count_line(board, row, col, dr, dc, player);
        if count >= 5 {
            return true;
        }
    }
    false
}

/// Get the positions of the winning line (5+ in a row) if one exists.
/// Returns None if no winning line found.
pub fn get_winning_line(
    board: &[[Option<Player>; BOARD_SIZE]; BOARD_SIZE],
    row: usize,
    col: usize,
    player: Player,
) -> Option<Vec<(usize, usize)>> {
    for (dr, dc) in DIRECTIONS {
        let count = count_line(board, row, col, dr, dc, player);
        if count >= 5 {
            return Some(collect_line_positions(board, row, col, dr, dc, player));
        }
    }
    None
}

/// Collect all positions in a winning line.
fn collect_line_positions(
    board: &[[Option<Player>; BOARD_SIZE]; BOARD_SIZE],
    row: usize,
    col: usize,
    dr: i32,
    dc: i32,
    player: Player,
) -> Vec<(usize, usize)> {
    let mut positions = vec![(row, col)];

    // Collect in positive direction
    let mut r = row as i32 + dr;
    let mut c = col as i32 + dc;
    while r >= 0 && r < BOARD_SIZE as i32 && c >= 0 && c < BOARD_SIZE as i32 {
        if board[r as usize][c as usize] == Some(player) {
            positions.push((r as usize, c as usize));
            r += dr;
            c += dc;
        } else {
            break;
        }
    }

    // Collect in negative direction
    r = row as i32 - dr;
    c = col as i32 - dc;
    while r >= 0 && r < BOARD_SIZE as i32 && c >= 0 && c < BOARD_SIZE as i32 {
        if board[r as usize][c as usize] == Some(player) {
            positions.push((r as usize, c as usize));
            r -= dr;
            c -= dc;
        } else {
            break;
        }
    }

    positions
}

/// Count consecutive stones in both directions from (row, col).
fn count_line(
    board: &[[Option<Player>; BOARD_SIZE]; BOARD_SIZE],
    row: usize,
    col: usize,
    dr: i32,
    dc: i32,
    player: Player,
) -> u32 {
    let mut count = 1; // Count the center stone

    // Count in positive direction
    count += count_direction(board, row, col, dr, dc, player);
    // Count in negative direction
    count += count_direction(board, row, col, -dr, -dc, player);

    count
}

/// Count consecutive stones in one direction from (row, col), excluding center.
fn count_direction(
    board: &[[Option<Player>; BOARD_SIZE]; BOARD_SIZE],
    row: usize,
    col: usize,
    dr: i32,
    dc: i32,
    player: Player,
) -> u32 {
    let mut count = 0;
    let mut r = row as i32 + dr;
    let mut c = col as i32 + dc;

    while r >= 0 && r < BOARD_SIZE as i32 && c >= 0 && c < BOARD_SIZE as i32 {
        if board[r as usize][c as usize] == Some(player) {
            count += 1;
            r += dr;
            c += dc;
        } else {
            break;
        }
    }
    count
}

/// Check if the board is full (draw condition).
pub fn is_board_full(board: &[[Option<Player>; BOARD_SIZE]; BOARD_SIZE]) -> bool {
    for row in board {
        for cell in row {
            if cell.is_none() {
                return false;
            }
        }
    }
    true
}

/// Process a human move at the cursor position.
pub fn process_human_move(game: &mut GomokuGame) -> bool {
    if game.game_result.is_some() || game.current_player != Player::Human {
        return false;
    }

    let (row, col) = game.cursor;
    if !game.place_stone(row, col) {
        return false;
    }

    // Check for win
    if check_win(&game.board, row, col, Player::Human) {
        game.winning_line = get_winning_line(&game.board, row, col, Player::Human);
        game.game_result = Some(GomokuResult::Win);
        return true;
    }

    // Check for draw
    if is_board_full(&game.board) {
        game.game_result = Some(GomokuResult::Draw);
        return true;
    }

    // Switch to AI turn
    game.switch_player();
    game.ai_thinking = true;
    game.ai_think_ticks = 0;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn place(game: &mut GomokuGame, row: usize, col: usize, player: Player) {
        game.board[row][col] = Some(player);
    }

    #[test]
    fn test_horizontal_win() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        for c in 0..5 {
            place(&mut game, 7, c, Player::Human);
        }
        assert!(check_win(&game.board, 7, 2, Player::Human));
    }

    #[test]
    fn test_vertical_win() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        for r in 0..5 {
            place(&mut game, r, 7, Player::Human);
        }
        assert!(check_win(&game.board, 2, 7, Player::Human));
    }

    #[test]
    fn test_diagonal_win() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        for i in 0..5 {
            place(&mut game, i, i, Player::Human);
        }
        assert!(check_win(&game.board, 2, 2, Player::Human));
    }

    #[test]
    fn test_no_win_with_four() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        for c in 0..4 {
            place(&mut game, 7, c, Player::Human);
        }
        assert!(!check_win(&game.board, 7, 2, Player::Human));
    }

    #[test]
    fn test_six_in_row_wins() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        for c in 0..6 {
            place(&mut game, 7, c, Player::Human);
        }
        assert!(check_win(&game.board, 7, 3, Player::Human));
    }

    #[test]
    fn test_board_not_full() {
        let game = GomokuGame::new(super::super::GomokuDifficulty::Novice);
        assert!(!is_board_full(&game.board));
    }
}

// === AI Evaluation ===

/// Score values for different patterns
const SCORE_FIVE: i32 = 100_000;
const SCORE_OPEN_FOUR: i32 = 10_000;
const SCORE_CLOSED_FOUR: i32 = 1_000;
const SCORE_OPEN_THREE: i32 = 500;
#[allow(dead_code)]
const SCORE_CLOSED_THREE: i32 = 100;
const SCORE_OPEN_TWO: i32 = 50;
const SCORE_CENTER_BONUS: i32 = 5;

/// Maximum candidates to evaluate at each depth (limits branching factor)
const MAX_CANDIDATES: usize = 15;

/// Evaluate the board from AI's perspective.
/// Positive = good for AI, negative = good for Human.
pub fn evaluate_board(board: &[[Option<Player>; BOARD_SIZE]; BOARD_SIZE]) -> i32 {
    let mut score = 0;

    // Evaluate all lines on the board
    score += evaluate_all_lines(board, Player::Ai);
    score -= evaluate_all_lines(board, Player::Human);

    // Small bonus for center control
    let center = BOARD_SIZE / 2;
    let start = center.saturating_sub(2);
    let end = (center + 2).min(BOARD_SIZE - 1);
    for row in board.iter().take(end + 1).skip(start) {
        for cell in row.iter().take(end + 1).skip(start) {
            if *cell == Some(Player::Ai) {
                score += SCORE_CENTER_BONUS;
            } else if *cell == Some(Player::Human) {
                score -= SCORE_CENTER_BONUS;
            }
        }
    }

    score
}

/// Evaluate all lines for a player, summing pattern scores.
fn evaluate_all_lines(board: &[[Option<Player>; BOARD_SIZE]; BOARD_SIZE], player: Player) -> i32 {
    let mut score = 0;

    // Check all rows
    for r in 0..BOARD_SIZE {
        score += evaluate_line_segment(board, r, 0, 0, 1, player);
    }

    // Check all columns
    for c in 0..BOARD_SIZE {
        score += evaluate_line_segment(board, 0, c, 1, 0, player);
    }

    // Check diagonals (down-right)
    for start in 0..BOARD_SIZE {
        score += evaluate_line_segment(board, start, 0, 1, 1, player);
        if start > 0 {
            score += evaluate_line_segment(board, 0, start, 1, 1, player);
        }
    }

    // Check diagonals (down-left)
    for start in 0..BOARD_SIZE {
        score += evaluate_line_segment(board, start, BOARD_SIZE - 1, 1, -1, player);
        if start > 0 {
            score += evaluate_line_segment(board, 0, BOARD_SIZE - 1 - start, 1, -1, player);
        }
    }

    score
}

/// Evaluate a line segment starting at (r, c) going in direction (dr, dc).
fn evaluate_line_segment(
    board: &[[Option<Player>; BOARD_SIZE]; BOARD_SIZE],
    start_r: usize,
    start_c: usize,
    dr: i32,
    dc: i32,
    player: Player,
) -> i32 {
    let mut score = 0;
    let mut r = start_r as i32;
    let mut c = start_c as i32;

    // Collect the line
    let mut line = Vec::new();
    while r >= 0 && r < BOARD_SIZE as i32 && c >= 0 && c < BOARD_SIZE as i32 {
        line.push(board[r as usize][c as usize]);
        r += dr;
        c += dc;
    }

    // Score windows of 5 in this line
    if line.len() >= 5 {
        for window in line.windows(5) {
            score += score_window(window, player);
        }
    }

    score
}

/// Score a window of 5 cells for patterns.
fn score_window(window: &[Option<Player>], player: Player) -> i32 {
    let own = window.iter().filter(|&&c| c == Some(player)).count();
    let empty = window.iter().filter(|&&c| c.is_none()).count();
    let opponent = 5 - own - empty;

    // If opponent has stones in this window, we can't complete it
    if opponent > 0 {
        return 0;
    }

    match own {
        5 => SCORE_FIVE,
        4 if empty == 1 => SCORE_CLOSED_FOUR, // One empty = closed four
        3 if empty == 2 => SCORE_OPEN_THREE,
        2 if empty == 3 => SCORE_OPEN_TWO,
        _ => 0,
    }
}

#[cfg(test)]
mod eval_tests {
    use super::super::GomokuDifficulty;
    use super::*;

    #[test]
    fn test_evaluate_empty_board() {
        let game = GomokuGame::new(GomokuDifficulty::Novice);
        let score = evaluate_board(&game.board);
        assert_eq!(score, 0);
    }

    #[test]
    fn test_evaluate_ai_advantage() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        // AI has 3 in a row with space
        game.board[7][7] = Some(Player::Ai);
        game.board[7][8] = Some(Player::Ai);
        game.board[7][9] = Some(Player::Ai);
        let score = evaluate_board(&game.board);
        assert!(score > 0, "AI should have positive score");
    }

    #[test]
    fn test_evaluate_human_advantage() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        // Human has 3 in a row with space
        game.board[7][7] = Some(Player::Human);
        game.board[7][8] = Some(Player::Human);
        game.board[7][9] = Some(Player::Human);
        let score = evaluate_board(&game.board);
        assert!(score < 0, "Human advantage should give negative score");
    }
}

// === Minimax AI ===

/// Get candidate moves (positions near existing stones).
fn get_candidate_moves(board: &[[Option<Player>; BOARD_SIZE]; BOARD_SIZE]) -> Vec<(usize, usize)> {
    let mut candidates = std::collections::HashSet::new();
    let mut has_stones = false;

    for r in 0..BOARD_SIZE {
        for c in 0..BOARD_SIZE {
            if board[r][c].is_some() {
                has_stones = true;
                // Add empty positions within 2 spaces
                for dr in -2i32..=2 {
                    for dc in -2i32..=2 {
                        let nr = r as i32 + dr;
                        let nc = c as i32 + dc;
                        if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
                            let nr = nr as usize;
                            let nc = nc as usize;
                            if board[nr][nc].is_none() {
                                candidates.insert((nr, nc));
                            }
                        }
                    }
                }
            }
        }
    }

    // If no stones on board, return center area
    if !has_stones {
        let center = BOARD_SIZE / 2;
        return vec![(center, center)];
    }

    candidates.into_iter().collect()
}

/// Quick score for a single move - evaluates only the lines through this position.
/// Used for move ordering (not full board evaluation).
fn score_move_quick(
    board: &[[Option<Player>; BOARD_SIZE]; BOARD_SIZE],
    row: usize,
    col: usize,
    player: Player,
) -> i32 {
    let mut score = 0;
    let opponent = if player == Player::Ai {
        Player::Human
    } else {
        Player::Ai
    };

    // Check all 4 directions through this position
    for (dr, dc) in DIRECTIONS {
        // Count our stones and empty spaces in this line (window of 5 centered on position)
        let (own, opp, _empty) = count_line_window(board, row, col, dr, dc, player);

        // Score based on what placing here would create
        if opp == 0 {
            // No opponent stones blocking this line
            match own {
                4 => score += SCORE_FIVE,      // Would make 5
                3 => score += SCORE_OPEN_FOUR, // Would make open 4
                2 => score += SCORE_OPEN_THREE,
                1 => score += SCORE_OPEN_TWO,
                _ => {}
            }
        } else if own == 0 {
            // Check if this blocks opponent's threat
            let (opp_own, _, _) = count_line_window(board, row, col, dr, dc, opponent);
            match opp_own {
                4 => score += SCORE_FIVE / 2,      // Block their winning move
                3 => score += SCORE_OPEN_FOUR / 2, // Block their open 4
                2 => score += SCORE_OPEN_THREE / 2,
                _ => {}
            }
        }
    }

    // Small bonus for center proximity
    let center = BOARD_SIZE / 2;
    let dist = (row as i32 - center as i32).abs() + (col as i32 - center as i32).abs();
    score += (BOARD_SIZE as i32 - dist) * 2;

    score
}

/// Count stones in a line window of 5 centered on (row, col).
/// Returns (own_count, opponent_count, empty_count).
fn count_line_window(
    board: &[[Option<Player>; BOARD_SIZE]; BOARD_SIZE],
    row: usize,
    col: usize,
    dr: i32,
    dc: i32,
    player: Player,
) -> (i32, i32, i32) {
    let mut own = 0;
    let mut opp = 0;
    let mut empty = 0;

    // Check 4 positions in each direction (plus center = 9 total, but we want patterns of 5)
    for offset in -4i32..=4 {
        let r = row as i32 + dr * offset;
        let c = col as i32 + dc * offset;

        if r >= 0 && r < BOARD_SIZE as i32 && c >= 0 && c < BOARD_SIZE as i32 {
            match board[r as usize][c as usize] {
                Some(p) if p == player => own += 1,
                Some(_) => opp += 1,
                None => empty += 1,
            }
        }
    }

    (own, opp, empty)
}

/// Get candidate moves sorted by quick heuristic score (best first).
/// Limits to MAX_CANDIDATES to reduce branching factor.
fn get_ordered_candidates(
    board: &[[Option<Player>; BOARD_SIZE]; BOARD_SIZE],
    maximizing: bool,
) -> Vec<(usize, usize)> {
    let candidates = get_candidate_moves(board);

    if candidates.len() <= MAX_CANDIDATES {
        // Few candidates - just sort them
        let player = if maximizing {
            Player::Ai
        } else {
            Player::Human
        };
        let mut scored: Vec<_> = candidates
            .into_iter()
            .map(|(r, c)| ((r, c), score_move_quick(board, r, c, player)))
            .collect();
        scored.sort_by(|a, b| b.1.cmp(&a.1)); // Descending by score
        return scored.into_iter().map(|(pos, _)| pos).collect();
    }

    // Many candidates - score, sort, and limit
    let player = if maximizing {
        Player::Ai
    } else {
        Player::Human
    };
    let mut scored: Vec<_> = candidates
        .into_iter()
        .map(|(r, c)| ((r, c), score_move_quick(board, r, c, player)))
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1)); // Descending by score
    scored
        .into_iter()
        .take(MAX_CANDIDATES)
        .map(|(pos, _)| pos)
        .collect()
}

/// Minimax with alpha-beta pruning.
fn minimax(
    board: &mut [[Option<Player>; BOARD_SIZE]; BOARD_SIZE],
    depth: i32,
    mut alpha: i32,
    mut beta: i32,
    maximizing: bool,
    last_move: Option<(usize, usize)>,
) -> i32 {
    // Check for terminal state
    if let Some((r, c)) = last_move {
        let last_player = if maximizing {
            Player::Human
        } else {
            Player::Ai
        };
        if check_win(board, r, c, last_player) {
            return if maximizing { -SCORE_FIVE } else { SCORE_FIVE };
        }
    }

    if depth == 0 {
        return evaluate_board(board);
    }

    // Get candidates sorted by heuristic score (best first) and limited in count
    let candidates = get_ordered_candidates(board, maximizing);
    if candidates.is_empty() {
        return 0; // Draw
    }

    if maximizing {
        let mut max_eval = i32::MIN;
        for (r, c) in candidates {
            board[r][c] = Some(Player::Ai);
            let eval = minimax(board, depth - 1, alpha, beta, false, Some((r, c)));
            board[r][c] = None;
            max_eval = max_eval.max(eval);
            alpha = alpha.max(eval);
            if beta <= alpha {
                break;
            }
        }
        max_eval
    } else {
        let mut min_eval = i32::MAX;
        for (r, c) in candidates {
            board[r][c] = Some(Player::Human);
            let eval = minimax(board, depth - 1, alpha, beta, true, Some((r, c)));
            board[r][c] = None;
            min_eval = min_eval.min(eval);
            beta = beta.min(eval);
            if beta <= alpha {
                break;
            }
        }
        min_eval
    }
}

/// Find the best move for AI using minimax.
pub fn find_best_move<R: Rng>(game: &GomokuGame, rng: &mut R) -> Option<(usize, usize)> {
    let mut board = game.board;
    let depth = game.difficulty.search_depth();
    let candidates = get_candidate_moves(&board);

    if candidates.is_empty() {
        return None;
    }

    // First check for immediate winning move
    for &(r, c) in &candidates {
        board[r][c] = Some(Player::Ai);
        if check_win(&board, r, c, Player::Ai) {
            return Some((r, c));
        }
        board[r][c] = None;
    }

    // Then check for blocking opponent's winning move
    for &(r, c) in &candidates {
        board[r][c] = Some(Player::Human);
        if check_win(&board, r, c, Player::Human) {
            return Some((r, c));
        }
        board[r][c] = None;
    }

    // Use minimax for other moves (with ordered and limited candidates for speed)
    let ordered_candidates = get_ordered_candidates(&board, true);
    let mut best_moves = Vec::new();
    let mut best_score = i32::MIN;

    for (r, c) in ordered_candidates {
        board[r][c] = Some(Player::Ai);
        let score = minimax(
            &mut board,
            depth - 1,
            i32::MIN,
            i32::MAX,
            false,
            Some((r, c)),
        );
        board[r][c] = None;

        if score > best_score {
            best_score = score;
            best_moves.clear();
            best_moves.push((r, c));
        } else if score == best_score {
            best_moves.push((r, c));
        }
    }

    // Randomly pick among equally good moves
    best_moves.choose(rng).copied()
}

/// Apply game result: update stats, grant rewards, and add combat log entries.
/// Returns Some(MinigameWinInfo) if the player won, None otherwise.
pub fn apply_game_result(state: &mut GameState) -> Option<crate::challenges::MinigameWinInfo> {
    use crate::challenges::menu::DifficultyInfo;
    use crate::challenges::{apply_challenge_rewards, GameResultInfo};

    let game = match state.active_minigame.as_ref() {
        Some(ActiveMinigame::Gomoku(g)) => g,
        _ => return None,
    };
    let result = game.game_result?;
    let difficulty = game.difficulty;
    let reward = difficulty.reward();

    let (won, loss_message) = match result {
        GomokuResult::Win => (true, ""),
        GomokuResult::Loss => (false, "The strategist nods respectfully and departs."),
        GomokuResult::Draw => (false, "A rare draw. The strategist seems impressed."),
    };

    apply_challenge_rewards(
        state,
        GameResultInfo {
            won,
            game_type: "gomoku",
            difficulty_str: difficulty.difficulty_str(),
            reward,
            icon: "◎",
            win_message: "Victory! The strategist bows in defeat.",
            loss_message,
        },
    )
}

/// Process AI thinking (called each tick).
pub fn process_ai_thinking<R: Rng>(game: &mut GomokuGame, rng: &mut R) {
    if !game.ai_thinking || game.game_result.is_some() {
        return;
    }

    // Add small delay for visual feedback (5-15 ticks = 0.5-1.5 seconds)
    game.ai_think_ticks += 1;
    let min_ticks = 5 + game.difficulty.search_depth() as u32 * 2;
    if game.ai_think_ticks < min_ticks {
        return;
    }

    // Find and make move
    if let Some((r, c)) = find_best_move(game, rng) {
        game.board[r][c] = Some(Player::Ai);
        game.move_history.push((r, c, Player::Ai));
        game.last_move = Some((r, c));

        // Check for AI win
        if check_win(&game.board, r, c, Player::Ai) {
            game.winning_line = get_winning_line(&game.board, r, c, Player::Ai);
            game.game_result = Some(GomokuResult::Loss);
        } else if is_board_full(&game.board) {
            game.game_result = Some(GomokuResult::Draw);
        } else {
            game.switch_player();
        }
    }

    game.ai_thinking = false;
    game.ai_think_ticks = 0;
}

#[cfg(test)]
mod ai_tests {
    use super::GomokuDifficulty;
    use super::*;

    #[test]
    fn test_ai_takes_winning_move() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        // AI has 4 in a row, should complete it
        game.board[7][3] = Some(Player::Ai);
        game.board[7][4] = Some(Player::Ai);
        game.board[7][5] = Some(Player::Ai);
        game.board[7][6] = Some(Player::Ai);

        let mut rng = rand::thread_rng();
        let best = find_best_move(&game, &mut rng);
        assert!(
            best == Some((7, 2)) || best == Some((7, 7)),
            "AI should complete 5 in a row"
        );
    }

    #[test]
    fn test_ai_blocks_human_win() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        // Human has 4 in a row
        game.board[7][3] = Some(Player::Human);
        game.board[7][4] = Some(Player::Human);
        game.board[7][5] = Some(Player::Human);
        game.board[7][6] = Some(Player::Human);

        let mut rng = rand::thread_rng();
        let best = find_best_move(&game, &mut rng);
        assert!(
            best == Some((7, 2)) || best == Some((7, 7)),
            "AI should block human win"
        );
    }

    #[test]
    fn test_get_candidates_empty_board() {
        let game = GomokuGame::new(GomokuDifficulty::Novice);
        let candidates = get_candidate_moves(&game.board);
        assert_eq!(
            candidates,
            vec![(7, 7)],
            "Empty board should suggest center"
        );
    }

    #[test]
    fn test_get_candidates_near_stones() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        game.board[7][7] = Some(Player::Human);
        let candidates = get_candidate_moves(&game.board);
        assert!(!candidates.is_empty());
        assert!(
            !candidates.contains(&(7, 7)),
            "Occupied position should not be candidate"
        );
    }

    // ============ process_input Tests ============

    #[test]
    fn test_process_input_cursor_movement() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);

        // Start at center (7, 7)
        assert_eq!(game.cursor, (7, 7));

        process_input(&mut game, GomokuInput::Up);
        assert_eq!(game.cursor, (6, 7));

        process_input(&mut game, GomokuInput::Down);
        assert_eq!(game.cursor, (7, 7));

        process_input(&mut game, GomokuInput::Left);
        assert_eq!(game.cursor, (7, 6));

        process_input(&mut game, GomokuInput::Right);
        assert_eq!(game.cursor, (7, 7));
    }

    #[test]
    fn test_process_input_place_stone() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        game.cursor = (5, 5);

        assert!(game.board[5][5].is_none());

        process_input(&mut game, GomokuInput::PlaceStone);

        assert_eq!(game.board[5][5], Some(Player::Human));
    }

    #[test]
    fn test_process_input_forfeit_single_esc() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);

        assert!(!game.forfeit_pending);

        process_input(&mut game, GomokuInput::Forfeit);

        assert!(game.forfeit_pending);
        assert!(game.game_result.is_none());
    }

    #[test]
    fn test_process_input_forfeit_double_esc() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);

        // First Esc sets pending
        process_input(&mut game, GomokuInput::Forfeit);
        assert!(game.forfeit_pending);

        // Second Esc confirms forfeit
        process_input(&mut game, GomokuInput::Forfeit);

        assert_eq!(game.game_result, Some(GomokuResult::Loss));
    }

    #[test]
    fn test_process_input_forfeit_cancelled() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);

        // First Esc sets pending
        process_input(&mut game, GomokuInput::Forfeit);
        assert!(game.forfeit_pending);

        // Any other key cancels forfeit
        process_input(&mut game, GomokuInput::Other);

        assert!(!game.forfeit_pending);
        assert!(game.game_result.is_none());
    }

    #[test]
    fn test_process_input_blocked_during_ai_thinking() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        game.ai_thinking = true;
        game.cursor = (7, 7);

        // Input should be blocked
        let handled = process_input(&mut game, GomokuInput::Up);

        assert!(!handled);
        assert_eq!(game.cursor, (7, 7)); // Cursor unchanged
    }

    // ============ Human Move + Win Detection ============

    #[test]
    fn test_human_move_triggers_win() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        // Place 4 human stones, then place 5th via process_human_move
        for c in 0..4 {
            game.board[7][c] = Some(Player::Human);
        }
        game.cursor = (7, 4);
        process_human_move(&mut game);

        assert_eq!(game.game_result, Some(GomokuResult::Win));
        assert!(game.winning_line.is_some());
        let line = game.winning_line.as_ref().unwrap();
        assert!(line.len() >= 5);
    }

    #[test]
    fn test_human_move_switches_to_ai() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        game.cursor = (5, 5);

        let result = process_human_move(&mut game);

        assert!(result);
        assert_eq!(game.current_player, Player::Ai);
        assert!(game.ai_thinking);
        assert_eq!(game.ai_think_ticks, 0);
        assert_eq!(game.move_history.len(), 1);
        assert_eq!(game.last_move, Some((5, 5)));
    }

    #[test]
    fn test_human_move_rejected_on_occupied() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        game.board[5][5] = Some(Player::Ai);
        game.cursor = (5, 5);

        let result = process_human_move(&mut game);

        assert!(!result);
        assert_eq!(game.current_player, Player::Human); // Still human's turn
    }

    #[test]
    fn test_human_move_rejected_when_game_over() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        game.game_result = Some(GomokuResult::Win);
        game.cursor = (5, 5);

        let result = process_human_move(&mut game);

        assert!(!result);
        assert!(game.board[5][5].is_none()); // No stone placed
    }

    #[test]
    fn test_human_move_rejected_during_ai_turn() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        game.current_player = Player::Ai;
        game.cursor = (5, 5);

        let result = process_human_move(&mut game);

        assert!(!result);
    }

    // ============ AI Thinking + Move Execution ============

    #[test]
    fn test_ai_thinking_delays_move() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        // Place a human stone so AI has context
        game.board[7][7] = Some(Player::Human);
        game.move_history.push((7, 7, Player::Human));
        game.current_player = Player::Ai;
        game.ai_thinking = true;
        game.ai_think_ticks = 0;

        let mut rng = rand::thread_rng();
        // First tick should NOT make a move (delay not met)
        process_ai_thinking(&mut game, &mut rng);

        assert!(game.ai_thinking, "AI should still be thinking after 1 tick");
        // Board should still have only the human stone
        let ai_stones: usize = game
            .board
            .iter()
            .flatten()
            .filter(|c| **c == Some(Player::Ai))
            .count();
        assert_eq!(ai_stones, 0, "AI should not have placed a stone yet");
    }

    #[test]
    fn test_ai_thinking_makes_move_after_delay() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        game.board[7][7] = Some(Player::Human);
        game.move_history.push((7, 7, Player::Human));
        game.current_player = Player::Ai;
        game.ai_thinking = true;
        game.ai_think_ticks = 0;

        let mut rng = rand::thread_rng();
        // Tick enough times for AI to make its move (Novice: 5 + 2*2 = 9 ticks min)
        for _ in 0..20 {
            process_ai_thinking(&mut game, &mut rng);
            if !game.ai_thinking {
                break;
            }
        }

        assert!(!game.ai_thinking, "AI should have finished thinking");
        let ai_stones: usize = game
            .board
            .iter()
            .flatten()
            .filter(|c| **c == Some(Player::Ai))
            .count();
        assert_eq!(ai_stones, 1, "AI should have placed exactly 1 stone");
        assert!(game.last_move.is_some());
        assert_eq!(game.move_history.len(), 2); // 1 human + 1 AI
        assert_eq!(game.current_player, Player::Human); // Back to human
    }

    #[test]
    fn test_ai_thinking_skipped_when_not_thinking() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        game.ai_thinking = false;

        let mut rng = rand::thread_rng();
        process_ai_thinking(&mut game, &mut rng);

        // Nothing should change
        let total_stones: usize = game.board.iter().flatten().filter(|c| c.is_some()).count();
        assert_eq!(total_stones, 0);
    }

    #[test]
    fn test_ai_thinking_skipped_when_game_over() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        game.ai_thinking = true;
        game.game_result = Some(GomokuResult::Win);

        let mut rng = rand::thread_rng();
        for _ in 0..20 {
            process_ai_thinking(&mut game, &mut rng);
        }

        // No AI move should be placed
        let total_stones: usize = game.board.iter().flatten().filter(|c| c.is_some()).count();
        assert_eq!(total_stones, 0);
    }

    #[test]
    fn test_ai_wins_sets_loss() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        // Set up AI with 4 in a row, human stone elsewhere
        game.board[7][3] = Some(Player::Ai);
        game.board[7][4] = Some(Player::Ai);
        game.board[7][5] = Some(Player::Ai);
        game.board[7][6] = Some(Player::Ai);
        game.board[0][0] = Some(Player::Human);
        game.current_player = Player::Ai;
        game.ai_thinking = true;
        game.ai_think_ticks = 0;

        let mut rng = rand::thread_rng();
        for _ in 0..20 {
            process_ai_thinking(&mut game, &mut rng);
            if !game.ai_thinking {
                break;
            }
        }

        assert_eq!(game.game_result, Some(GomokuResult::Loss));
        assert!(game.winning_line.is_some());
    }

    // ============ Full Game Turn Cycle ============

    #[test]
    fn test_full_turn_cycle_human_then_ai() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);

        // Human places stone
        game.cursor = (7, 7);
        process_input(&mut game, GomokuInput::PlaceStone);

        assert_eq!(game.board[7][7], Some(Player::Human));
        assert!(game.ai_thinking);
        assert_eq!(game.current_player, Player::Ai);

        // AI thinks and places
        let mut rng = rand::thread_rng();
        for _ in 0..20 {
            process_ai_thinking(&mut game, &mut rng);
            if !game.ai_thinking {
                break;
            }
        }

        assert!(!game.ai_thinking);
        assert_eq!(game.current_player, Player::Human);
        let total_stones: usize = game.board.iter().flatten().filter(|c| c.is_some()).count();
        assert_eq!(total_stones, 2, "Should have human + AI stones");
    }

    // ============ apply_game_result ============

    #[test]
    fn test_apply_game_result_win_grants_xp() {
        use crate::challenges::menu::DifficultyInfo;
        use crate::core::game_state::GameState;

        let mut state = GameState::new("Test".to_string(), 0);
        let initial_xp = state.character_xp;
        let xp_for_level = crate::core::game_logic::xp_for_next_level(state.character_level);

        // Set up won game
        let mut game = GomokuGame::new(GomokuDifficulty::Apprentice);
        game.game_result = Some(GomokuResult::Win);
        state.active_minigame = Some(ActiveMinigame::Gomoku(game));

        let result = apply_game_result(&mut state);

        assert!(result.is_some()); // Win returns Some(MinigameWinInfo)
        assert!(
            state.active_minigame.is_none(),
            "Minigame should be cleared"
        );
        // Apprentice: xp_percent=100, so full level XP
        let expected_xp = (xp_for_level * 100) / 100;
        assert_eq!(state.character_xp, initial_xp + expected_xp);
    }

    #[test]
    fn test_apply_game_result_win_grants_prestige() {
        use crate::core::game_state::GameState;

        let mut state = GameState::new("Test".to_string(), 0);
        let initial_prestige = state.prestige_rank;

        // Master gives 2 prestige ranks
        let mut game = GomokuGame::new(GomokuDifficulty::Master);
        game.game_result = Some(GomokuResult::Win);
        state.active_minigame = Some(ActiveMinigame::Gomoku(game));

        apply_game_result(&mut state);

        assert_eq!(state.prestige_rank, initial_prestige + 2);
    }

    #[test]
    fn test_apply_game_result_loss_no_rewards() {
        use crate::core::game_state::GameState;

        let mut state = GameState::new("Test".to_string(), 0);
        let initial_xp = state.character_xp;
        let initial_prestige = state.prestige_rank;

        let mut game = GomokuGame::new(GomokuDifficulty::Master);
        game.game_result = Some(GomokuResult::Loss);
        state.active_minigame = Some(ActiveMinigame::Gomoku(game));

        let result = apply_game_result(&mut state);

        assert!(result.is_none()); // Loss returns None
        assert!(state.active_minigame.is_none()); // But game is still cleared
        assert_eq!(state.character_xp, initial_xp, "No XP on loss");
        assert_eq!(state.prestige_rank, initial_prestige, "No prestige on loss");
    }

    #[test]
    fn test_apply_game_result_draw_no_rewards() {
        use crate::core::game_state::GameState;

        let mut state = GameState::new("Test".to_string(), 0);
        let initial_xp = state.character_xp;

        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        game.game_result = Some(GomokuResult::Draw);
        state.active_minigame = Some(ActiveMinigame::Gomoku(game));

        let result = apply_game_result(&mut state);

        assert!(result.is_none()); // Draw returns None
        assert!(state.active_minigame.is_none()); // But game is still cleared
        assert_eq!(state.character_xp, initial_xp, "No XP on draw");
    }

    #[test]
    fn test_apply_game_result_returns_none_no_minigame() {
        use crate::core::game_state::GameState;

        let mut state = GameState::new("Test".to_string(), 0);
        assert!(apply_game_result(&mut state).is_none());
    }

    #[test]
    fn test_apply_game_result_returns_none_no_result() {
        use crate::core::game_state::GameState;

        let mut state = GameState::new("Test".to_string(), 0);
        let game = GomokuGame::new(GomokuDifficulty::Novice);
        state.active_minigame = Some(ActiveMinigame::Gomoku(game));

        assert!(apply_game_result(&mut state).is_none());
    }

    #[test]
    fn test_apply_game_result_adds_combat_log() {
        use crate::core::game_state::GameState;

        let mut state = GameState::new("Test".to_string(), 0);
        let initial_log_len = state.combat_state.combat_log.len();

        let mut game = GomokuGame::new(GomokuDifficulty::Journeyman);
        game.game_result = Some(GomokuResult::Win);
        state.active_minigame = Some(ActiveMinigame::Gomoku(game));

        apply_game_result(&mut state);

        assert!(
            state.combat_state.combat_log.len() > initial_log_len,
            "Should add combat log entries"
        );
    }
}
