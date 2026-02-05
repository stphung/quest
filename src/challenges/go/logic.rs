//! Go game logic: placement, capture, ko, scoring.

use super::types::{GoGame, GoMove, GoResult, Stone, BOARD_SIZE};
use std::collections::HashSet;

/// Get all stones in the same group as the stone at (row, col).
/// Returns empty set if position is empty.
pub fn get_group(
    board: &[[Option<Stone>; BOARD_SIZE]; BOARD_SIZE],
    row: usize,
    col: usize,
) -> HashSet<(usize, usize)> {
    let mut group = HashSet::new();
    let Some(stone) = board[row][col] else {
        return group;
    };

    let mut stack = vec![(row, col)];
    while let Some((r, c)) = stack.pop() {
        if group.contains(&(r, c)) {
            continue;
        }
        if board[r][c] == Some(stone) {
            group.insert((r, c));
            // Add adjacent positions
            for (dr, dc) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let nr = r as i32 + dr;
                let nc = c as i32 + dc;
                if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
                    stack.push((nr as usize, nc as usize));
                }
            }
        }
    }
    group
}

/// Count liberties (empty adjacent points) of a group.
pub fn count_liberties(
    board: &[[Option<Stone>; BOARD_SIZE]; BOARD_SIZE],
    group: &HashSet<(usize, usize)>,
) -> usize {
    let mut liberties = HashSet::new();
    for &(row, col) in group {
        for (dr, dc) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let nr = row as i32 + dr;
            let nc = col as i32 + dc;
            if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
                let nr = nr as usize;
                let nc = nc as usize;
                if board[nr][nc].is_none() {
                    liberties.insert((nr, nc));
                }
            }
        }
    }
    liberties.len()
}

/// Get liberties count for the group containing the stone at (row, col).
pub fn get_liberties_at(
    board: &[[Option<Stone>; BOARD_SIZE]; BOARD_SIZE],
    row: usize,
    col: usize,
) -> usize {
    let group = get_group(board, row, col);
    count_liberties(board, &group)
}

/// Remove a group from the board and return the number of stones captured.
fn remove_group(
    board: &mut [[Option<Stone>; BOARD_SIZE]; BOARD_SIZE],
    group: &HashSet<(usize, usize)>,
) -> u32 {
    let count = group.len() as u32;
    for &(row, col) in group {
        board[row][col] = None;
    }
    count
}

/// Check and remove any opponent groups with zero liberties adjacent to (row, col).
/// Returns the total number of stones captured.
pub fn capture_dead_groups(
    board: &mut [[Option<Stone>; BOARD_SIZE]; BOARD_SIZE],
    row: usize,
    col: usize,
    capturing_player: Stone,
) -> u32 {
    let opponent = capturing_player.opponent();
    let mut captured = 0;
    let mut checked = HashSet::new();

    // Check all adjacent positions for opponent groups
    for (dr, dc) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
        let nr = row as i32 + dr;
        let nc = col as i32 + dc;
        if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
            let nr = nr as usize;
            let nc = nc as usize;

            if checked.contains(&(nr, nc)) {
                continue;
            }

            if board[nr][nc] == Some(opponent) {
                let group = get_group(board, nr, nc);
                for &pos in &group {
                    checked.insert(pos);
                }
                if count_liberties(board, &group) == 0 {
                    captured += remove_group(board, &group);
                }
            }
        }
    }
    captured
}

/// Check if a move is legal (ignoring ko for now).
/// A move is illegal if:
/// 1. Position is occupied
/// 2. Position is the ko point
/// 3. Move would be suicide (no liberties after placement, and no captures)
pub fn is_legal_move(game: &GoGame, row: usize, col: usize) -> bool {
    // Position must be empty
    if !game.is_empty(row, col) {
        return false;
    }

    // Cannot play at ko point
    if game.ko_point == Some((row, col)) {
        return false;
    }

    // Check for suicide
    // Temporarily place the stone
    let mut test_board = game.board;
    test_board[row][col] = Some(game.current_player);

    // First check if this move captures anything
    let opponent = game.current_player.opponent();
    let mut would_capture = false;
    for (dr, dc) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
        let nr = row as i32 + dr;
        let nc = col as i32 + dc;
        if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
            let nr = nr as usize;
            let nc = nc as usize;
            if test_board[nr][nc] == Some(opponent) {
                let group = get_group(&test_board, nr, nc);
                if count_liberties(&test_board, &group) == 0 {
                    would_capture = true;
                    break;
                }
            }
        }
    }

    // If we capture, move is legal (not suicide)
    if would_capture {
        return true;
    }

    // Check if our group would have liberties
    let our_group = get_group(&test_board, row, col);
    count_liberties(&test_board, &our_group) > 0
}

/// Get all legal moves for the current player.
pub fn get_legal_moves(game: &GoGame) -> Vec<GoMove> {
    let mut moves = Vec::new();
    for row in 0..BOARD_SIZE {
        for col in 0..BOARD_SIZE {
            if is_legal_move(game, row, col) {
                moves.push(GoMove::Place(row, col));
            }
        }
    }
    moves.push(GoMove::Pass); // Pass is always legal
    moves
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::GoDifficulty;

    fn place(board: &mut [[Option<Stone>; BOARD_SIZE]; BOARD_SIZE], row: usize, col: usize, stone: Stone) {
        board[row][col] = Some(stone);
    }

    #[test]
    fn test_single_stone_liberties() {
        let mut board = [[None; BOARD_SIZE]; BOARD_SIZE];
        place(&mut board, 4, 4, Stone::Black);
        // Center stone has 4 liberties
        assert_eq!(get_liberties_at(&board, 4, 4), 4);
    }

    #[test]
    fn test_corner_stone_liberties() {
        let mut board = [[None; BOARD_SIZE]; BOARD_SIZE];
        place(&mut board, 0, 0, Stone::Black);
        // Corner stone has 2 liberties
        assert_eq!(get_liberties_at(&board, 0, 0), 2);
    }

    #[test]
    fn test_edge_stone_liberties() {
        let mut board = [[None; BOARD_SIZE]; BOARD_SIZE];
        place(&mut board, 0, 4, Stone::Black);
        // Edge stone has 3 liberties
        assert_eq!(get_liberties_at(&board, 0, 4), 3);
    }

    #[test]
    fn test_group_liberties() {
        let mut board = [[None; BOARD_SIZE]; BOARD_SIZE];
        // Two connected stones share liberties
        place(&mut board, 4, 4, Stone::Black);
        place(&mut board, 4, 5, Stone::Black);
        // Group has 6 liberties (shared liberty counted once)
        assert_eq!(get_liberties_at(&board, 4, 4), 6);
        assert_eq!(get_liberties_at(&board, 4, 5), 6);
    }

    #[test]
    fn test_surrounded_stone() {
        let mut board = [[None; BOARD_SIZE]; BOARD_SIZE];
        place(&mut board, 4, 4, Stone::Black);
        place(&mut board, 3, 4, Stone::White);
        place(&mut board, 5, 4, Stone::White);
        place(&mut board, 4, 3, Stone::White);
        place(&mut board, 4, 5, Stone::White);
        // Surrounded stone has 0 liberties
        assert_eq!(get_liberties_at(&board, 4, 4), 0);
    }

    #[test]
    fn test_get_group() {
        let mut board = [[None; BOARD_SIZE]; BOARD_SIZE];
        place(&mut board, 4, 4, Stone::Black);
        place(&mut board, 4, 5, Stone::Black);
        place(&mut board, 4, 6, Stone::Black);

        let group = get_group(&board, 4, 4);
        assert_eq!(group.len(), 3);
        assert!(group.contains(&(4, 4)));
        assert!(group.contains(&(4, 5)));
        assert!(group.contains(&(4, 6)));
    }

    #[test]
    fn test_empty_position_group() {
        let board = [[None; BOARD_SIZE]; BOARD_SIZE];
        let group = get_group(&board, 4, 4);
        assert!(group.is_empty());
    }

    #[test]
    fn test_capture_single_stone() {
        let mut board = [[None; BOARD_SIZE]; BOARD_SIZE];
        // Set up a capture scenario
        place(&mut board, 4, 4, Stone::White);
        place(&mut board, 3, 4, Stone::Black);
        place(&mut board, 5, 4, Stone::Black);
        place(&mut board, 4, 3, Stone::Black);
        // White has 1 liberty at (4, 5)
        assert_eq!(get_liberties_at(&board, 4, 4), 1);

        // Black plays at (4, 5) to capture
        place(&mut board, 4, 5, Stone::Black);
        let captured = capture_dead_groups(&mut board, 4, 5, Stone::Black);

        assert_eq!(captured, 1);
        assert!(board[4][4].is_none()); // White stone removed
    }

    #[test]
    fn test_capture_group() {
        let mut board = [[None; BOARD_SIZE]; BOARD_SIZE];
        // Two white stones
        place(&mut board, 4, 4, Stone::White);
        place(&mut board, 4, 5, Stone::White);
        // Surround with black
        place(&mut board, 3, 4, Stone::Black);
        place(&mut board, 3, 5, Stone::Black);
        place(&mut board, 5, 4, Stone::Black);
        place(&mut board, 5, 5, Stone::Black);
        place(&mut board, 4, 3, Stone::Black);
        // One liberty left at (4, 6)

        place(&mut board, 4, 6, Stone::Black);
        let captured = capture_dead_groups(&mut board, 4, 6, Stone::Black);

        assert_eq!(captured, 2);
        assert!(board[4][4].is_none());
        assert!(board[4][5].is_none());
    }

    #[test]
    fn test_no_capture_with_liberties() {
        let mut board = [[None; BOARD_SIZE]; BOARD_SIZE];
        place(&mut board, 4, 4, Stone::White);
        place(&mut board, 3, 4, Stone::Black);
        place(&mut board, 5, 4, Stone::Black);
        place(&mut board, 4, 3, Stone::Black);
        // White still has liberty at (4, 5) - not surrounded

        // Black plays elsewhere
        place(&mut board, 0, 0, Stone::Black);
        let captured = capture_dead_groups(&mut board, 0, 0, Stone::Black);

        assert_eq!(captured, 0);
        assert!(board[4][4].is_some()); // White stone still there
    }

    #[test]
    fn test_legal_move_empty_board() {
        let game = GoGame::new(GoDifficulty::Novice);
        assert!(is_legal_move(&game, 4, 4));
        assert!(is_legal_move(&game, 0, 0));
    }

    #[test]
    fn test_illegal_move_occupied() {
        let mut game = GoGame::new(GoDifficulty::Novice);
        game.board[4][4] = Some(Stone::Black);
        assert!(!is_legal_move(&game, 4, 4));
    }

    #[test]
    fn test_illegal_move_ko() {
        let mut game = GoGame::new(GoDifficulty::Novice);
        game.ko_point = Some((4, 4));
        assert!(!is_legal_move(&game, 4, 4));
    }

    #[test]
    fn test_suicide_illegal() {
        let mut game = GoGame::new(GoDifficulty::Novice);
        // Create a surrounded empty point
        game.board[3][4] = Some(Stone::White);
        game.board[5][4] = Some(Stone::White);
        game.board[4][3] = Some(Stone::White);
        game.board[4][5] = Some(Stone::White);
        game.current_player = Stone::Black;
        // Black playing at (4,4) would be suicide
        assert!(!is_legal_move(&game, 4, 4));
    }

    #[test]
    fn test_capture_not_suicide() {
        let mut game = GoGame::new(GoDifficulty::Novice);
        // White stone at center
        game.board[4][4] = Some(Stone::White);
        // Black almost surrounds
        game.board[3][4] = Some(Stone::Black);
        game.board[5][4] = Some(Stone::Black);
        game.board[4][3] = Some(Stone::Black);
        game.current_player = Stone::Black;
        // Black playing at (4,5) captures white, so it's legal even though
        // black would have no liberties without the capture
        assert!(is_legal_move(&game, 4, 5));
    }

    #[test]
    fn test_get_legal_moves_includes_pass() {
        let game = GoGame::new(GoDifficulty::Novice);
        let moves = get_legal_moves(&game);
        assert!(moves.contains(&GoMove::Pass));
    }

    #[test]
    fn test_get_legal_moves_empty_board() {
        let game = GoGame::new(GoDifficulty::Novice);
        let moves = get_legal_moves(&game);
        // 81 board positions + 1 pass
        assert_eq!(moves.len(), 82);
    }
}
