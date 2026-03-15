use rand::Rng;

use super::types::{SudokuDifficulty, SudokuGame};

/// Generate a complete Sudoku puzzle for the given difficulty.
pub fn generate_puzzle<R: Rng>(difficulty: SudokuDifficulty, rng: &mut R) -> SudokuGame {
    let solution = generate_solved_board(rng);
    let (board, given) = remove_cells(&solution, difficulty, rng);
    SudokuGame::new(difficulty, board, solution, given)
}

/// Generate a fully solved 9x9 Sudoku board.
fn generate_solved_board<R: Rng>(rng: &mut R) -> [[u8; 9]; 9] {
    let mut board = [[0u8; 9]; 9];

    // Fill the three diagonal 3x3 boxes (they share no constraints)
    for box_idx in 0..3 {
        let start = box_idx * 3;
        let mut digits: Vec<u8> = (1..=9).collect();
        shuffle(&mut digits, rng);
        let mut idx = 0;
        for r in start..start + 3 {
            for c in start..start + 3 {
                board[r][c] = digits[idx];
                idx += 1;
            }
        }
    }

    // Solve the rest with backtracking
    solve_board(&mut board, rng);
    board
}

/// Randomized backtracking solver. Returns true if a solution was found.
fn solve_board<R: Rng>(board: &mut [[u8; 9]; 9], rng: &mut R) -> bool {
    // Find next empty cell
    let mut empty = None;
    'outer: for r in 0..9 {
        for c in 0..9 {
            if board[r][c] == 0 {
                empty = Some((r, c));
                break 'outer;
            }
        }
    }

    let (row, col) = match empty {
        Some(pos) => pos,
        None => return true, // All cells filled
    };

    // Try digits in random order
    let mut digits: Vec<u8> = (1..=9).collect();
    shuffle(&mut digits, rng);

    for &digit in &digits {
        if is_valid_placement(board, row, col, digit) {
            board[row][col] = digit;
            if solve_board(board, rng) {
                return true;
            }
            board[row][col] = 0;
        }
    }

    false
}

/// Check if placing `digit` at (row, col) is valid.
fn is_valid_placement(board: &[[u8; 9]; 9], row: usize, col: usize, digit: u8) -> bool {
    // Check row
    for c in 0..9 {
        if board[row][c] == digit {
            return false;
        }
    }

    // Check column
    for r in 0..9 {
        if board[r][col] == digit {
            return false;
        }
    }

    // Check 3x3 box
    let box_row = (row / 3) * 3;
    let box_col = (col / 3) * 3;
    for r in box_row..box_row + 3 {
        for c in box_col..box_col + 3 {
            if board[r][c] == digit {
                return false;
            }
        }
    }

    true
}

/// Remove cells from a solved board to create the puzzle.
/// Ensures the puzzle has a unique solution after each removal.
fn remove_cells<R: Rng>(
    solution: &[[u8; 9]; 9],
    difficulty: SudokuDifficulty,
    rng: &mut R,
) -> ([[u8; 9]; 9], [[bool; 9]; 9]) {
    let mut board = *solution;
    let mut given = [[true; 9]; 9];

    let (min_remove, max_remove) = difficulty.cells_to_remove_range();
    let target = rng.gen_range(min_remove..=max_remove);

    // Create a shuffled list of all cell positions
    let mut positions: Vec<(usize, usize)> = Vec::with_capacity(81);
    for r in 0..9 {
        for c in 0..9 {
            positions.push((r, c));
        }
    }
    shuffle(&mut positions, rng);

    let mut removed = 0;
    for (r, c) in positions {
        if removed >= target {
            break;
        }

        let saved = board[r][c];
        board[r][c] = 0;

        if has_unique_solution(&board) {
            given[r][c] = false;
            removed += 1;
        } else {
            board[r][c] = saved; // Restore — removing this cell creates ambiguity
        }
    }

    (board, given)
}

/// Check if the board has exactly one solution.
/// Uses backtracking, stops as soon as a second solution is found.
fn has_unique_solution(board: &[[u8; 9]; 9]) -> bool {
    let mut board_copy = *board;
    let mut count = 0;
    count_solutions(&mut board_copy, &mut count, 2);
    count == 1
}

/// Count solutions up to `limit`. Stops early once limit is reached.
fn count_solutions(board: &mut [[u8; 9]; 9], count: &mut u32, limit: u32) {
    if *count >= limit {
        return;
    }

    // Find next empty cell
    let mut empty = None;
    'outer: for r in 0..9 {
        for c in 0..9 {
            if board[r][c] == 0 {
                empty = Some((r, c));
                break 'outer;
            }
        }
    }

    let (row, col) = match empty {
        Some(pos) => pos,
        None => {
            *count += 1;
            return;
        }
    };

    for digit in 1..=9 {
        if is_valid_placement(board, row, col, digit) {
            board[row][col] = digit;
            count_solutions(board, count, limit);
            board[row][col] = 0;
            if *count >= limit {
                return;
            }
        }
    }
}

/// Fisher-Yates shuffle
fn shuffle<T, R: Rng>(slice: &mut Vec<T>, rng: &mut R) {
    for i in (1..slice.len()).rev() {
        let j = rng.gen_range(0..=i);
        slice.swap(i, j);
    }
}
