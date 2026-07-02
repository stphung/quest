//! Gomoku AI: board evaluation and minimax search.

use super::logic::{check_win, DIRECTIONS};
use super::{Player, BOARD_SIZE};
use rand::seq::IndexedRandom;
use rand::Rng;

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
        scored.sort_by_key(|b| std::cmp::Reverse(b.1)); // Descending by score
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
    scored.sort_by_key(|b| std::cmp::Reverse(b.1)); // Descending by score
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
pub fn find_best_move<R: Rng>(game: &super::GomokuGame, rng: &mut R) -> Option<(usize, usize)> {
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

#[cfg(test)]
mod eval_tests {
    use super::super::GomokuDifficulty;
    use super::*;
    use crate::challenges::gomoku::GomokuGame;

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

#[cfg(test)]
mod ai_tests {
    use super::super::GomokuDifficulty;
    use super::*;
    use crate::challenges::gomoku::GomokuGame;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_ai_takes_winning_move() {
        let mut game = GomokuGame::new(GomokuDifficulty::Novice);
        // AI has 4 in a row, should complete it
        game.board[7][3] = Some(Player::Ai);
        game.board[7][4] = Some(Player::Ai);
        game.board[7][5] = Some(Player::Ai);
        game.board[7][6] = Some(Player::Ai);

        let mut rng = ChaCha8Rng::seed_from_u64(42);
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

        let mut rng = ChaCha8Rng::seed_from_u64(42);
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
