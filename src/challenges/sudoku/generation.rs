use rand::{Rng, RngExt};

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
        for row in board.iter_mut().skip(start).take(3) {
            for cell in row.iter_mut().skip(start).take(3) {
                *cell = digits[idx];
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
    'outer: for (r, row) in board.iter().enumerate() {
        for (c, &cell) in row.iter().enumerate() {
            if cell == 0 {
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
    if board[row].contains(&digit) {
        return false;
    }

    // Check column
    if board.iter().any(|r| r[col] == digit) {
        return false;
    }

    // Check 3x3 box
    let box_row = (row / 3) * 3;
    let box_col = (col / 3) * 3;
    for r in &board[box_row..box_row + 3] {
        for &cell in &r[box_col..box_col + 3] {
            if cell == digit {
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
    let target = rng.random_range(min_remove..=max_remove);

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
    'outer: for (r, row) in board.iter().enumerate() {
        for (c, &cell) in row.iter().enumerate() {
            if cell == 0 {
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
fn shuffle<T, R: Rng>(slice: &mut [T], rng: &mut R) {
    for i in (1..slice.len()).rev() {
        let j = rng.random_range(0..=i);
        slice.swap(i, j);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_generate_solved_board_is_valid() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let board = generate_solved_board(&mut rng);

        for row in &board {
            for &cell in row {
                assert!((1..=9).contains(&cell));
            }
        }

        // Each row has 1-9
        for row in &board {
            let mut seen = [false; 10];
            for &cell in row {
                let v = cell as usize;
                assert!(!seen[v]);
                seen[v] = true;
            }
        }

        // Each col has 1-9
        for c in 0..9 {
            let mut seen = [false; 10];
            for row in &board {
                let v = row[c] as usize;
                assert!(!seen[v]);
                seen[v] = true;
            }
        }

        // Each 3x3 box has 1-9
        for box_r in 0..3 {
            for box_c in 0..3 {
                let mut seen = [false; 10];
                for r in 0..3 {
                    for c in 0..3 {
                        let v = board[box_r * 3 + r][box_c * 3 + c] as usize;
                        assert!(!seen[v]);
                        seen[v] = true;
                    }
                }
            }
        }
    }

    #[test]
    fn test_generate_puzzle_has_unique_solution() {
        let mut rng = ChaCha8Rng::seed_from_u64(123);
        let game = generate_puzzle(SudokuDifficulty::Novice, &mut rng);
        assert_ne!(game.board, game.solution);
        assert!(has_unique_solution(&game.board));

        for r in 0..9 {
            for c in 0..9 {
                if game.given[r][c] {
                    assert_eq!(game.board[r][c], game.solution[r][c]);
                } else {
                    assert_eq!(game.board[r][c], 0);
                }
            }
        }
    }

    #[test]
    fn test_difficulty_given_counts() {
        let mut rng = ChaCha8Rng::seed_from_u64(456);
        for difficulty in SudokuDifficulty::ALL {
            let game = generate_puzzle(difficulty, &mut rng);
            let given: usize = game.given.iter().flatten().filter(|&&g| g).count();
            let (min_remove, max_remove) = difficulty.cells_to_remove_range();
            let min_given = 81 - max_remove;
            let max_given = 81 - min_remove;
            assert!(
                given >= min_given && given <= max_given,
                "{:?}: expected {}-{} givens, got {}",
                difficulty,
                min_given,
                max_given,
                given
            );
        }
    }

    #[test]
    fn test_is_valid_placement() {
        let mut board = [[0u8; 9]; 9];
        board[0][0] = 5;
        assert!(!is_valid_placement(&board, 0, 4, 5)); // same row
        assert!(!is_valid_placement(&board, 4, 0, 5)); // same col
        assert!(!is_valid_placement(&board, 1, 1, 5)); // same box
        assert!(is_valid_placement(&board, 4, 4, 5)); // different
        assert!(is_valid_placement(&board, 0, 1, 3)); // different digit
    }
}
