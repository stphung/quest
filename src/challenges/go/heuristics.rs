//! Go heuristics and pattern recognition for improved MCTS.
//!
//! These heuristics guide the AI toward sensible moves without
//! requiring neural network training.

use super::logic::{count_liberties, get_group, would_be_captured};
use super::types::{GoGame, GoMove, Stone, BOARD_SIZE};

/// Score a move based on various heuristics.
/// Higher score = more promising move for MCTS prioritization.
pub fn score_move(game: &GoGame, row: usize, col: usize) -> f64 {
    let mut score = 0.0;
    let player = game.current_player;
    let opponent = player.opponent();

    // Base position value (corners > sides > center for opening)
    score += position_value(row, col, game);

    // Proximity to last move (local fighting is important)
    if let Some(GoMove::Place(lr, lc)) = game.last_move {
        let dist = ((row as i32 - lr as i32).abs() + (col as i32 - lc as i32).abs()) as f64;
        if dist <= 3.0 {
            score += (4.0 - dist) * 5.0; // Bonus for playing near last move
        }
    }

    // Capture threat: if we can capture opponent stones
    score += capture_score(game, row, col, player, opponent);

    // Defensive: save our groups in atari
    score += defense_score(game, row, col, player);

    // Extension from existing stones
    score += extension_score(game, row, col, player);

    // Eye-making potential
    score += eye_potential_score(game, row, col, player);

    // Avoid self-atari (putting own group in atari)
    score += self_atari_penalty(game, row, col, player);

    // Pattern bonuses
    score += pattern_score(game, row, col, player);

    score
}

/// Position value based on board location.
/// In the opening, corners and sides are more valuable.
fn position_value(row: usize, col: usize, game: &GoGame) -> f64 {
    // Count stones on board to determine game phase
    let stone_count: usize = game.board.iter().flatten().filter(|s| s.is_some()).count();

    if stone_count < 20 {
        // Opening phase: corners and sides are valuable
        let from_edge = |x: usize| (x).min(BOARD_SIZE - 1 - x);
        let edge_dist_row = from_edge(row);
        let edge_dist_col = from_edge(col);

        // Star points and 3-3 points are excellent
        if is_star_point(row, col) {
            return 15.0;
        }

        // 3rd and 4th line are good
        if (edge_dist_row == 2 || edge_dist_row == 3) && (edge_dist_col == 2 || edge_dist_col == 3)
        {
            return 10.0;
        }

        // Edges are okay
        if edge_dist_row <= 3 || edge_dist_col <= 3 {
            return 5.0;
        }

        // Center is less valuable in opening
        return 2.0;
    }

    // Middle/end game: all positions roughly equal
    3.0
}

/// Check if position is a star point (9x9 board).
fn is_star_point(row: usize, col: usize) -> bool {
    // 9x9 star points: corners at (2,2), (2,6), (6,2), (6,6) and center (4,4)
    matches!(
        (row, col),
        (2, 2) | (2, 6) | (6, 2) | (6, 6) | (4, 4) | (2, 4) | (4, 2) | (4, 6) | (6, 4)
    )
}

/// Score for capturing opponent stones.
fn capture_score(
    game: &GoGame,
    row: usize,
    col: usize,
    player: Stone,
    opponent: Stone,
) -> f64 {
    let mut score = 0.0;

    // Check adjacent opponent groups
    for (dr, dc) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
        let nr = row as i32 + dr;
        let nc = col as i32 + dc;
        if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
            let nr = nr as usize;
            let nc = nc as usize;
            if game.board[nr][nc] == Some(opponent) {
                let group = get_group(&game.board, nr, nc);
                let libs = count_liberties(&game.board, &group);
                if libs == 1 {
                    // Can capture! Huge bonus scaled by group size
                    score += 50.0 + (group.len() as f64 * 10.0);
                } else if libs == 2 {
                    // Threatening atari
                    score += 15.0;
                }
            }
        }
    }

    score
}

/// Score for defending our own groups.
fn defense_score(game: &GoGame, row: usize, col: usize, player: Stone) -> f64 {
    let mut score = 0.0;

    // Check if any adjacent friendly group is in atari
    for (dr, dc) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
        let nr = row as i32 + dr;
        let nc = col as i32 + dc;
        if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
            let nr = nr as usize;
            let nc = nc as usize;
            if game.board[nr][nc] == Some(player) {
                let group = get_group(&game.board, nr, nc);
                let libs = count_liberties(&game.board, &group);
                if libs == 1 {
                    // Our group is in atari! Save it!
                    score += 40.0 + (group.len() as f64 * 8.0);
                } else if libs == 2 {
                    // Running or adding liberties to weak group
                    score += 10.0;
                }
            }
        }
    }

    score
}

/// Score for extending from existing stones.
fn extension_score(game: &GoGame, row: usize, col: usize, player: Stone) -> f64 {
    let mut adjacent_friendly = 0;
    let mut diagonal_friendly = 0;

    // Count adjacent friendly stones
    for (dr, dc) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
        let nr = row as i32 + dr;
        let nc = col as i32 + dc;
        if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
            if game.board[nr as usize][nc as usize] == Some(player) {
                adjacent_friendly += 1;
            }
        }
    }

    // Count diagonal friendly stones (for knight's move, etc.)
    for (dr, dc) in &[(-1, -1), (-1, 1), (1, -1), (1, 1)] {
        let nr = row as i32 + dr;
        let nc = col as i32 + dc;
        if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
            if game.board[nr as usize][nc as usize] == Some(player) {
                diagonal_friendly += 1;
            }
        }
    }

    // One adjacent is good (extension), too many is slow (clumping)
    match adjacent_friendly {
        0 => diagonal_friendly as f64 * 3.0, // Knight's move or jump
        1 => 8.0,                             // Good extension
        2 => 4.0,                             // Okay
        _ => -5.0,                            // Clumping, usually bad
    }
}

/// Score for eye-making potential.
fn eye_potential_score(game: &GoGame, row: usize, col: usize, player: Stone) -> f64 {
    // Count how many corners and edges around this point are our stones
    let mut friendly_around = 0;
    let mut total_around = 0;

    for dr in -1..=1 {
        for dc in -1..=1 {
            if dr == 0 && dc == 0 {
                continue;
            }
            let nr = row as i32 + dr;
            let nc = col as i32 + dc;
            if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
                total_around += 1;
                if game.board[nr as usize][nc as usize] == Some(player) {
                    friendly_around += 1;
                }
            }
        }
    }

    // If surrounded mostly by our stones, this could be an eye
    if total_around > 0 && friendly_around >= total_around - 1 && friendly_around >= 3 {
        // Don't fill our own eye!
        return -30.0;
    }

    0.0
}

/// Penalty for self-atari.
fn self_atari_penalty(game: &GoGame, row: usize, col: usize, player: Stone) -> f64 {
    // Simulate placing the stone and check if it puts us in atari
    if would_be_captured(game, row, col, player) {
        return -100.0; // Suicide (handled elsewhere, but penalize anyway)
    }

    // Check if this move would put our group in atari
    let mut test_board = game.board;
    test_board[row][col] = Some(player);
    let group = get_group(&test_board, row, col);
    let libs = count_liberties(&test_board, &group);

    if libs == 1 && group.len() > 1 {
        // Self-atari of a multi-stone group is usually bad
        return -25.0;
    }

    0.0
}

/// Pattern-based scoring for common shapes.
fn pattern_score(game: &GoGame, row: usize, col: usize, player: Stone) -> f64 {
    let mut score = 0.0;
    let opponent = player.opponent();

    // Check for cutting points
    score += cut_score(game, row, col, opponent);

    // Check for connection moves
    score += connection_score(game, row, col, player);

    score
}

/// Score for cutting opponent's stones.
fn cut_score(game: &GoGame, row: usize, col: usize, opponent: Stone) -> f64 {
    // A cut separates opponent groups that were diagonally connected
    let mut diagonal_opponent = 0;

    for (dr, dc) in &[(-1, -1), (-1, 1), (1, -1), (1, 1)] {
        let nr = row as i32 + dr;
        let nc = col as i32 + dc;
        if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
            if game.board[nr as usize][nc as usize] == Some(opponent) {
                diagonal_opponent += 1;
            }
        }
    }

    // Check if adjacent points are empty (potential cut)
    if diagonal_opponent >= 2 {
        let mut adj_empty = 0;
        for (dr, dc) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let nr = row as i32 + dr;
            let nc = col as i32 + dc;
            if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
                if game.board[nr as usize][nc as usize].is_none() {
                    adj_empty += 1;
                }
            }
        }
        if adj_empty >= 2 {
            return 12.0; // Potential cut
        }
    }

    0.0
}

/// Score for connecting our own stones.
fn connection_score(game: &GoGame, row: usize, col: usize, player: Stone) -> f64 {
    // Connecting diagonally separated groups is often good
    let mut score = 0.0;
    let directions = [(-1, -1), (-1, 1), (1, -1), (1, 1)];

    for (dr, dc) in &directions {
        let nr = row as i32 + dr;
        let nc = col as i32 + dc;
        if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
            if game.board[nr as usize][nc as usize] == Some(player) {
                // Check the two adjacent squares
                let adj1_r = row as i32 + dr;
                let adj1_c = col as i32;
                let adj2_r = row as i32;
                let adj2_c = col as i32 + dc;

                let empty1 = if adj1_r >= 0 && adj1_r < BOARD_SIZE as i32 {
                    game.board[adj1_r as usize][adj1_c as usize].is_none()
                } else {
                    false
                };
                let empty2 = if adj2_c >= 0 && adj2_c < BOARD_SIZE as i32 {
                    game.board[adj2_r as usize][adj2_c as usize].is_none()
                } else {
                    false
                };

                if empty1 && empty2 {
                    score += 6.0; // Connecting diagonal stones
                }
            }
        }
    }

    score
}

/// Order moves by heuristic score for MCTS.
pub fn order_moves(game: &GoGame, moves: &mut Vec<GoMove>) {
    moves.sort_by(|a, b| {
        let score_a = match a {
            GoMove::Place(r, c) => score_move(game, *r, *c),
            GoMove::Pass => -50.0, // Pass is usually undesirable
        };
        let score_b = match b {
            GoMove::Place(r, c) => score_move(game, *r, *c),
            GoMove::Pass => -50.0,
        };
        score_b.partial_cmp(&score_a).unwrap()
    });
}

/// Get the top N moves by heuristic score.
pub fn get_top_moves(game: &GoGame, moves: &[GoMove], n: usize) -> Vec<GoMove> {
    let mut scored: Vec<_> = moves
        .iter()
        .map(|m| {
            let score = match m {
                GoMove::Place(r, c) => score_move(game, *r, *c),
                GoMove::Pass => -50.0,
            };
            (*m, score)
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    scored.into_iter().take(n).map(|(m, _)| m).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::GoDifficulty;

    #[test]
    fn test_star_points() {
        assert!(is_star_point(2, 2));
        assert!(is_star_point(4, 4));
        assert!(is_star_point(6, 6));
        assert!(!is_star_point(0, 0));
        assert!(!is_star_point(5, 5));
    }

    #[test]
    fn test_capture_priority() {
        let mut game = GoGame::new(GoDifficulty::Novice);
        // Set up a capture scenario: Black stone at (4,4) with 1 liberty at (4,5)
        game.board[4][4] = Some(Stone::Black);
        game.board[3][4] = Some(Stone::White);
        game.board[5][4] = Some(Stone::White);
        game.board[4][3] = Some(Stone::White);
        game.current_player = Stone::White;

        // (4,5) should have a high score - it captures!
        let capture_score = score_move(&game, 4, 5);
        let random_score = score_move(&game, 0, 0);

        assert!(
            capture_score > random_score + 30.0,
            "Capture should be prioritized: {} vs {}",
            capture_score,
            random_score
        );
    }

    #[test]
    fn test_defense_priority() {
        let mut game = GoGame::new(GoDifficulty::Novice);
        // Set up: White stone at (4,4) in atari, liberty at (4,5)
        game.board[4][4] = Some(Stone::White);
        game.board[3][4] = Some(Stone::Black);
        game.board[5][4] = Some(Stone::Black);
        game.board[4][3] = Some(Stone::Black);
        game.current_player = Stone::White;

        // (4,5) should have high score - saves our stone!
        let defense_score = score_move(&game, 4, 5);
        let random_score = score_move(&game, 0, 0);

        assert!(
            defense_score > random_score + 20.0,
            "Defense should be prioritized: {} vs {}",
            defense_score,
            random_score
        );
    }

    #[test]
    fn test_opening_prefers_star_points() {
        let game = GoGame::new(GoDifficulty::Novice);

        let star_score = score_move(&game, 2, 2);
        let edge_score = score_move(&game, 0, 4);
        let center_score = score_move(&game, 4, 4);

        assert!(star_score > edge_score, "Star point should beat edge");
        assert!(center_score >= star_score - 5.0, "Center star is good too");
    }

    #[test]
    fn test_eye_filling_penalty() {
        let mut game = GoGame::new(GoDifficulty::Novice);
        // Create a near-eye: surrounded by our stones
        game.board[3][4] = Some(Stone::Black);
        game.board[5][4] = Some(Stone::Black);
        game.board[4][3] = Some(Stone::Black);
        game.board[4][5] = Some(Stone::Black);
        game.board[3][3] = Some(Stone::Black);
        game.board[3][5] = Some(Stone::Black);
        game.board[5][3] = Some(Stone::Black);
        game.board[5][5] = Some(Stone::Black);
        game.current_player = Stone::Black;

        let eye_score = score_move(&game, 4, 4);

        assert!(eye_score < 0.0, "Filling own eye should be penalized");
    }
}
