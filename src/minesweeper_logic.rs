//! Minesweeper game logic for mine placement and cell counting.

use rand::seq::SliceRandom;
use rand::Rng;

use crate::minesweeper::MinesweeperGame;

/// Get valid neighbor coordinates for a cell.
///
/// Returns a vector of (row, col) tuples for all valid neighbors (up to 8 directions).
pub fn get_neighbors(row: usize, col: usize, height: usize, width: usize) -> Vec<(usize, usize)> {
    let mut neighbors = Vec::with_capacity(8);

    for d_row in -1i32..=1 {
        for d_col in -1i32..=1 {
            // Skip the cell itself
            if d_row == 0 && d_col == 0 {
                continue;
            }

            let new_row = row as i32 + d_row;
            let new_col = col as i32 + d_col;

            // Check bounds
            if new_row >= 0
                && new_row < height as i32
                && new_col >= 0
                && new_col < width as i32
            {
                neighbors.push((new_row as usize, new_col as usize));
            }
        }
    }

    neighbors
}

/// Place mines on the grid, avoiding the first click cell and its neighbors.
///
/// This ensures the first click is always safe (opens a clearing).
pub fn place_mines<R: Rng>(game: &mut MinesweeperGame, first_row: usize, first_col: usize, rng: &mut R) {
    // Build the exclusion set: first click cell + its neighbors
    let mut excluded: Vec<(usize, usize)> = vec![(first_row, first_col)];
    excluded.extend(get_neighbors(first_row, first_col, game.height, game.width));

    // Build list of valid positions (all cells not in exclusion set)
    let mut valid_positions: Vec<(usize, usize)> = Vec::new();
    for row in 0..game.height {
        for col in 0..game.width {
            if !excluded.contains(&(row, col)) {
                valid_positions.push((row, col));
            }
        }
    }

    // Shuffle and take the required number of mines
    valid_positions.shuffle(rng);
    let mine_count = game.total_mines as usize;

    for &(row, col) in valid_positions.iter().take(mine_count) {
        game.grid[row][col].has_mine = true;
    }
}

/// Calculate adjacent mine counts for all cells.
///
/// For each non-mine cell, counts how many neighboring cells contain mines.
pub fn calculate_adjacent_counts(game: &mut MinesweeperGame) {
    for row in 0..game.height {
        for col in 0..game.width {
            // Skip mine cells (they don't need a count displayed)
            if game.grid[row][col].has_mine {
                continue;
            }

            // Count adjacent mines
            let mut count = 0u8;
            for (n_row, n_col) in get_neighbors(row, col, game.height, game.width) {
                if game.grid[n_row][n_col].has_mine {
                    count += 1;
                }
            }

            game.grid[row][col].adjacent_mines = count;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::minesweeper::MinesweeperDifficulty;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_get_neighbors_center() {
        // Center cell should have 8 neighbors
        let neighbors = get_neighbors(4, 4, 9, 9);
        assert_eq!(neighbors.len(), 8);

        // Verify all 8 neighbors are present
        let expected = vec![
            (3, 3), (3, 4), (3, 5),
            (4, 3),         (4, 5),
            (5, 3), (5, 4), (5, 5),
        ];
        for pos in expected {
            assert!(neighbors.contains(&pos), "Missing neighbor {:?}", pos);
        }
    }

    #[test]
    fn test_get_neighbors_corner() {
        // Top-left corner should have 3 neighbors
        let neighbors = get_neighbors(0, 0, 9, 9);
        assert_eq!(neighbors.len(), 3);
        assert!(neighbors.contains(&(0, 1)));
        assert!(neighbors.contains(&(1, 0)));
        assert!(neighbors.contains(&(1, 1)));

        // Bottom-right corner
        let neighbors = get_neighbors(8, 8, 9, 9);
        assert_eq!(neighbors.len(), 3);
        assert!(neighbors.contains(&(7, 7)));
        assert!(neighbors.contains(&(7, 8)));
        assert!(neighbors.contains(&(8, 7)));
    }

    #[test]
    fn test_get_neighbors_edge() {
        // Top edge (not corner) should have 5 neighbors
        let neighbors = get_neighbors(0, 4, 9, 9);
        assert_eq!(neighbors.len(), 5);

        // Left edge (not corner) should have 5 neighbors
        let neighbors = get_neighbors(4, 0, 9, 9);
        assert_eq!(neighbors.len(), 5);
    }

    #[test]
    fn test_place_mines_count() {
        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);
        let mut rng = StdRng::seed_from_u64(42);

        // First click at center
        place_mines(&mut game, 4, 4, &mut rng);

        // Count total mines
        let mine_count: usize = game
            .grid
            .iter()
            .flat_map(|row| row.iter())
            .filter(|cell| cell.has_mine)
            .count();

        assert_eq!(mine_count, 10, "Novice should have 10 mines");
    }

    #[test]
    fn test_place_mines_avoids_first_click() {
        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);
        let mut rng = StdRng::seed_from_u64(42);

        let first_row = 4;
        let first_col = 4;
        place_mines(&mut game, first_row, first_col, &mut rng);

        // First click cell should not have a mine
        assert!(
            !game.grid[first_row][first_col].has_mine,
            "First click cell should not have a mine"
        );

        // All neighbors of first click should not have mines
        let neighbors = get_neighbors(first_row, first_col, game.height, game.width);
        for (n_row, n_col) in neighbors {
            assert!(
                !game.grid[n_row][n_col].has_mine,
                "Neighbor ({}, {}) should not have a mine",
                n_row,
                n_col
            );
        }
    }

    #[test]
    fn test_place_mines_avoids_first_click_corner() {
        // Test corner click where there are fewer neighbors
        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);
        let mut rng = StdRng::seed_from_u64(42);

        let first_row = 0;
        let first_col = 0;
        place_mines(&mut game, first_row, first_col, &mut rng);

        // First click cell should not have a mine
        assert!(
            !game.grid[first_row][first_col].has_mine,
            "First click cell should not have a mine"
        );

        // All neighbors of first click should not have mines
        let neighbors = get_neighbors(first_row, first_col, game.height, game.width);
        for (n_row, n_col) in neighbors {
            assert!(
                !game.grid[n_row][n_col].has_mine,
                "Neighbor ({}, {}) should not have a mine",
                n_row,
                n_col
            );
        }

        // Should still have 10 mines
        let mine_count: usize = game
            .grid
            .iter()
            .flat_map(|row| row.iter())
            .filter(|cell| cell.has_mine)
            .count();
        assert_eq!(mine_count, 10);
    }

    #[test]
    fn test_adjacent_counts() {
        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);

        // Manually place mines in a known pattern for testing
        // Place mines at (0,0), (0,1), (1,0) - forming an L in top-left
        game.grid[0][0].has_mine = true;
        game.grid[0][1].has_mine = true;
        game.grid[1][0].has_mine = true;

        calculate_adjacent_counts(&mut game);

        // Cell (1,1) should have 3 adjacent mines
        assert_eq!(
            game.grid[1][1].adjacent_mines, 3,
            "Cell (1,1) should have 3 adjacent mines"
        );

        // Cell (0,2) should have 1 adjacent mine (only (0,1))
        assert_eq!(
            game.grid[0][2].adjacent_mines, 1,
            "Cell (0,2) should have 1 adjacent mine"
        );

        // Cell (2,0) should have 1 adjacent mine (only (1,0))
        assert_eq!(
            game.grid[2][0].adjacent_mines, 1,
            "Cell (2,0) should have 1 adjacent mine"
        );

        // Cell (2,2) should have 0 adjacent mines
        assert_eq!(
            game.grid[2][2].adjacent_mines, 0,
            "Cell (2,2) should have 0 adjacent mines"
        );

        // Mine cells should have count 0 (we skip them)
        assert_eq!(game.grid[0][0].adjacent_mines, 0);
        assert_eq!(game.grid[0][1].adjacent_mines, 0);
        assert_eq!(game.grid[1][0].adjacent_mines, 0);
    }

    #[test]
    fn test_adjacent_counts_all_difficulties() {
        for difficulty in MinesweeperDifficulty::ALL {
            let mut game = MinesweeperGame::new(difficulty);
            let mut rng = StdRng::seed_from_u64(42);

            // Place mines with first click at center
            let (height, width) = difficulty.grid_size();
            place_mines(&mut game, height / 2, width / 2, &mut rng);
            calculate_adjacent_counts(&mut game);

            // Verify all non-mine cells have valid counts (0-8)
            for row in 0..game.height {
                for col in 0..game.width {
                    let cell = &game.grid[row][col];
                    if !cell.has_mine {
                        assert!(
                            cell.adjacent_mines <= 8,
                            "Adjacent count should be 0-8, got {} at ({}, {})",
                            cell.adjacent_mines,
                            row,
                            col
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_deterministic_with_seed() {
        // Verify that using the same seed produces the same mine placement
        let mut game1 = MinesweeperGame::new(MinesweeperDifficulty::Novice);
        let mut game2 = MinesweeperGame::new(MinesweeperDifficulty::Novice);

        let mut rng1 = StdRng::seed_from_u64(42);
        let mut rng2 = StdRng::seed_from_u64(42);

        place_mines(&mut game1, 4, 4, &mut rng1);
        place_mines(&mut game2, 4, 4, &mut rng2);

        // Mine placement should be identical
        for row in 0..game1.height {
            for col in 0..game1.width {
                assert_eq!(
                    game1.grid[row][col].has_mine,
                    game2.grid[row][col].has_mine,
                    "Mine placement differs at ({}, {})",
                    row,
                    col
                );
            }
        }
    }
}
