//! Gomoku game logic and AI.

use crate::game_state::GameState;
use crate::gomoku::{GomokuDifficulty, GomokuGame, GomokuResult, Player, BOARD_SIZE};
use rand::seq::SliceRandom;
use rand::Rng;

/// Start a gomoku game with the selected difficulty
pub fn start_gomoku_game(state: &mut GameState, difficulty: GomokuDifficulty) {
    state.active_gomoku = Some(GomokuGame::new(difficulty));
    state.challenge_menu.close();
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
        let mut game = GomokuGame::new(crate::gomoku::GomokuDifficulty::Novice);
        for c in 0..5 {
            place(&mut game, 7, c, Player::Human);
        }
        assert!(check_win(&game.board, 7, 2, Player::Human));
    }

    #[test]
    fn test_vertical_win() {
        let mut game = GomokuGame::new(crate::gomoku::GomokuDifficulty::Novice);
        for r in 0..5 {
            place(&mut game, r, 7, Player::Human);
        }
        assert!(check_win(&game.board, 2, 7, Player::Human));
    }

    #[test]
    fn test_diagonal_win() {
        let mut game = GomokuGame::new(crate::gomoku::GomokuDifficulty::Novice);
        for i in 0..5 {
            place(&mut game, i, i, Player::Human);
        }
        assert!(check_win(&game.board, 2, 2, Player::Human));
    }

    #[test]
    fn test_no_win_with_four() {
        let mut game = GomokuGame::new(crate::gomoku::GomokuDifficulty::Novice);
        for c in 0..4 {
            place(&mut game, 7, c, Player::Human);
        }
        assert!(!check_win(&game.board, 7, 2, Player::Human));
    }

    #[test]
    fn test_six_in_row_wins() {
        let mut game = GomokuGame::new(crate::gomoku::GomokuDifficulty::Novice);
        for c in 0..6 {
            place(&mut game, 7, c, Player::Human);
        }
        assert!(check_win(&game.board, 7, 3, Player::Human));
    }

    #[test]
    fn test_board_not_full() {
        let game = GomokuGame::new(crate::gomoku::GomokuDifficulty::Novice);
        assert!(!is_board_full(&game.board));
    }
}

// === AI Evaluation ===

/// Score values for different patterns
const SCORE_FIVE: i32 = 100_000;
#[allow(dead_code)]
const SCORE_OPEN_FOUR: i32 = 10_000;
const SCORE_CLOSED_FOUR: i32 = 1_000;
const SCORE_OPEN_THREE: i32 = 500;
#[allow(dead_code)]
const SCORE_CLOSED_THREE: i32 = 100;
const SCORE_OPEN_TWO: i32 = 50;
const SCORE_CENTER_BONUS: i32 = 5;

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
    use super::*;
    use crate::gomoku::GomokuDifficulty;

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

    let candidates = get_candidate_moves(board);
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

    // Use minimax for other moves
    let mut best_moves = Vec::new();
    let mut best_score = i32::MIN;

    for (r, c) in candidates {
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
    use super::*;
    use crate::gomoku::GomokuDifficulty;

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
}
