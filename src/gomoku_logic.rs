//! Gomoku game logic and AI.

use crate::gomoku::{GomokuGame, GomokuResult, Player, BOARD_SIZE};

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
