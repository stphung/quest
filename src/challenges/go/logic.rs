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

#[cfg(test)]
mod tests {
    use super::*;

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
}
