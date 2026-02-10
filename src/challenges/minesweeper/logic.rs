//! Minesweeper game logic for mine placement and cell counting.

use rand::seq::SliceRandom;
use rand::Rng;

use super::MinesweeperGame;
use crate::challenges::{ChallengeResult, MinigameInput};

/// Process a key input during active Minesweeper game.
/// Returns true if the input was handled.
pub fn process_input<R: Rng>(
    game: &mut MinesweeperGame,
    input: MinigameInput,
    rng: &mut R,
) -> bool {
    // Handle forfeit confirmation (double-Esc pattern)
    if game.forfeit_pending {
        match input {
            MinigameInput::Cancel => {
                game.game_result = Some(ChallengeResult::Forfeit);
            }
            _ => {
                game.forfeit_pending = false;
            }
        }
        return true;
    }

    // Normal game input
    match input {
        MinigameInput::Up => game.move_cursor(-1, 0),
        MinigameInput::Down => game.move_cursor(1, 0),
        MinigameInput::Left => game.move_cursor(0, -1),
        MinigameInput::Right => game.move_cursor(0, 1),
        MinigameInput::Primary => {
            let (row, col) = game.cursor;
            if !game.first_click_done {
                handle_first_click(game, row, col, rng);
            } else {
                reveal_cell(game, row, col);
            }
        }
        MinigameInput::Secondary => {
            let (row, col) = game.cursor;
            toggle_flag(game, row, col);
        }
        MinigameInput::Cancel => {
            game.forfeit_pending = true;
        }
        MinigameInput::Other => {}
    }
    true
}

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
            if new_row >= 0 && new_row < height as i32 && new_col >= 0 && new_col < width as i32 {
                neighbors.push((new_row as usize, new_col as usize));
            }
        }
    }

    neighbors
}

/// Place mines on the grid, avoiding the first click cell and its neighbors.
///
/// This ensures the first click is always safe (opens a clearing).
pub fn place_mines<R: Rng>(
    game: &mut MinesweeperGame,
    first_row: usize,
    first_col: usize,
    rng: &mut R,
) {
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

/// Reveal a cell at the given position.
///
/// Returns true if the game continues, false if a mine was hit.
/// - If cell is flagged or already revealed, no action is taken.
/// - If cell has a mine, the game ends in a loss.
/// - If cell has adjacent_mines > 0, only that cell is revealed.
/// - If cell has adjacent_mines == 0, flood-fill reveals neighboring cells.
pub fn reveal_cell(game: &mut MinesweeperGame, row: usize, col: usize) -> bool {
    let cell = &game.grid[row][col];

    // If cell is flagged or already revealed, no action
    if cell.flagged || cell.revealed {
        return true;
    }

    // Reveal the cell
    game.grid[row][col].revealed = true;

    // Check if it's a mine
    if game.grid[row][col].has_mine {
        game.game_result = Some(ChallengeResult::Loss);
        reveal_all_mines(game);
        return false;
    }

    // If cell has adjacent mines > 0, just reveal it
    // If cell has 0 adjacent mines, flood-fill reveal neighbors
    if game.grid[row][col].adjacent_mines == 0 {
        flood_fill_reveal(game, row, col);
    }

    check_win_condition(game);
    true
}

/// Flood-fill reveal cells starting from a cell with 0 adjacent mines.
///
/// Uses a stack-based approach to avoid recursion.
/// For each cell with 0 adjacent mines, reveals all neighbors.
/// Stops at cells with adjacent_mines > 0 (but still reveals them).
/// Skips revealed, flagged, and mine cells.
pub fn flood_fill_reveal(game: &mut MinesweeperGame, start_row: usize, start_col: usize) {
    let mut stack: Vec<(usize, usize)> = vec![(start_row, start_col)];

    while let Some((row, col)) = stack.pop() {
        // Get all neighbors of this cell
        for (n_row, n_col) in get_neighbors(row, col, game.height, game.width) {
            let neighbor = &game.grid[n_row][n_col];

            // Skip revealed, flagged, and mine cells
            if neighbor.revealed || neighbor.flagged || neighbor.has_mine {
                continue;
            }

            // Reveal the neighbor
            game.grid[n_row][n_col].revealed = true;

            // If this neighbor also has 0 adjacent mines, add it to the stack
            if game.grid[n_row][n_col].adjacent_mines == 0 {
                stack.push((n_row, n_col));
            }
        }
    }
}

/// Reveal all mines on the grid (called on game loss).
pub fn reveal_all_mines(game: &mut MinesweeperGame) {
    for row in 0..game.height {
        for col in 0..game.width {
            if game.grid[row][col].has_mine {
                game.grid[row][col].revealed = true;
            }
        }
    }
}

/// Toggle flag on a cell.
///
/// - If the cell is already revealed, do nothing.
/// - If the cell is flagged, remove the flag and decrement flags_placed.
/// - If the cell is not flagged, add a flag and increment flags_placed.
pub fn toggle_flag(game: &mut MinesweeperGame, row: usize, col: usize) {
    let cell = &game.grid[row][col];

    // Cannot flag a revealed cell
    if cell.revealed {
        return;
    }

    if cell.flagged {
        game.grid[row][col].flagged = false;
        game.flags_placed -= 1;
    } else {
        game.grid[row][col].flagged = true;
        game.flags_placed += 1;
    }
}

/// Check if the player has won the game.
///
/// Win condition: all non-mine cells are revealed.
/// Equivalent to: unrevealed cells == total mines.
pub fn check_win_condition(game: &mut MinesweeperGame) {
    let mut unrevealed_count = 0u16;

    for row in 0..game.height {
        for col in 0..game.width {
            if !game.grid[row][col].revealed {
                unrevealed_count += 1;
            }
        }
    }

    if unrevealed_count == game.total_mines {
        game.game_result = Some(ChallengeResult::Win);
    }
}

/// Handle the first click on the grid.
///
/// Places mines (avoiding the clicked cell and its neighbors),
/// calculates adjacent counts, sets first_click_done, and reveals the cell.
pub fn handle_first_click<R: Rng>(game: &mut MinesweeperGame, row: usize, col: usize, rng: &mut R) {
    place_mines(game, row, col, rng);
    calculate_adjacent_counts(game);
    game.first_click_done = true;
    reveal_cell(game, row, col);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::challenges::ChallengeDifficulty;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_get_neighbors_center() {
        // Center cell should have 8 neighbors
        let neighbors = get_neighbors(4, 4, 9, 9);
        assert_eq!(neighbors.len(), 8);

        // Verify all 8 neighbors are present
        let expected = vec![
            (3, 3),
            (3, 4),
            (3, 5),
            (4, 3),
            (4, 5),
            (5, 3),
            (5, 4),
            (5, 5),
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
        let mut game = MinesweeperGame::new(ChallengeDifficulty::Novice);
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
        let mut game = MinesweeperGame::new(ChallengeDifficulty::Novice);
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
                n_row, n_col
            );
        }
    }

    #[test]
    fn test_place_mines_avoids_first_click_corner() {
        // Test corner click where there are fewer neighbors
        let mut game = MinesweeperGame::new(ChallengeDifficulty::Novice);
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
                n_row, n_col
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
        let mut game = MinesweeperGame::new(ChallengeDifficulty::Novice);

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
        for difficulty in ChallengeDifficulty::ALL {
            let mut game = MinesweeperGame::new(difficulty);
            let mut rng = StdRng::seed_from_u64(42);

            // Place mines with first click at center
            let (height, width) = (game.height, game.width);
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
        let mut game1 = MinesweeperGame::new(ChallengeDifficulty::Novice);
        let mut game2 = MinesweeperGame::new(ChallengeDifficulty::Novice);

        let mut rng1 = StdRng::seed_from_u64(42);
        let mut rng2 = StdRng::seed_from_u64(42);

        place_mines(&mut game1, 4, 4, &mut rng1);
        place_mines(&mut game2, 4, 4, &mut rng2);

        // Mine placement should be identical
        for row in 0..game1.height {
            for col in 0..game1.width {
                assert_eq!(
                    game1.grid[row][col].has_mine, game2.grid[row][col].has_mine,
                    "Mine placement differs at ({}, {})",
                    row, col
                );
            }
        }
    }

    #[test]
    fn test_reveal_safe_cell() {
        let mut game = MinesweeperGame::new(ChallengeDifficulty::Novice);

        // Manually set up a simple grid with a known mine pattern
        // Place a mine at (0, 0)
        game.grid[0][0].has_mine = true;
        calculate_adjacent_counts(&mut game);
        game.first_click_done = true;

        // Cell (1, 1) should have adjacent_mines = 1
        assert_eq!(game.grid[1][1].adjacent_mines, 1);

        // Reveal a safe cell with adjacent mines
        let result = reveal_cell(&mut game, 1, 1);
        assert!(result, "Should return true when revealing safe cell");
        assert!(game.grid[1][1].revealed, "Cell should be revealed");
        assert!(
            game.game_result.is_none(),
            "Game should still be in progress"
        );
    }

    #[test]
    fn test_reveal_mine() {
        let mut game = MinesweeperGame::new(ChallengeDifficulty::Novice);

        // Place a mine at (2, 2)
        game.grid[2][2].has_mine = true;
        game.first_click_done = true;

        // Reveal the mine
        let result = reveal_cell(&mut game, 2, 2);
        assert!(!result, "Should return false when hitting a mine");
        assert!(game.grid[2][2].revealed, "Mine cell should be revealed");
        assert_eq!(
            game.game_result,
            Some(ChallengeResult::Loss),
            "Game should be lost"
        );
    }

    #[test]
    fn test_toggle_flag() {
        let mut game = MinesweeperGame::new(ChallengeDifficulty::Novice);

        // Initially: no flags
        assert_eq!(game.flags_placed, 0);
        assert!(!game.grid[3][3].flagged);

        // Toggle flag on
        toggle_flag(&mut game, 3, 3);
        assert!(game.grid[3][3].flagged);
        assert_eq!(game.flags_placed, 1);

        // Toggle flag off
        toggle_flag(&mut game, 3, 3);
        assert!(!game.grid[3][3].flagged);
        assert_eq!(game.flags_placed, 0);
    }

    #[test]
    fn test_cannot_reveal_flagged() {
        let mut game = MinesweeperGame::new(ChallengeDifficulty::Novice);
        game.first_click_done = true;

        // Flag a cell
        toggle_flag(&mut game, 3, 3);
        assert!(game.grid[3][3].flagged);

        // Try to reveal the flagged cell
        let result = reveal_cell(&mut game, 3, 3);
        assert!(result, "Should return true (no action taken)");
        assert!(
            !game.grid[3][3].revealed,
            "Flagged cell should not be revealed"
        );
    }

    #[test]
    fn test_win_condition() {
        let mut game = MinesweeperGame::new(ChallengeDifficulty::Novice);

        // Place mines at known positions
        let mine_positions = vec![
            (0, 0),
            (0, 1),
            (0, 2),
            (0, 3),
            (0, 4),
            (0, 5),
            (0, 6),
            (0, 7),
            (0, 8),
            (1, 0),
        ];

        for (row, col) in &mine_positions {
            game.grid[*row][*col].has_mine = true;
        }
        calculate_adjacent_counts(&mut game);
        game.first_click_done = true;

        // Reveal all non-mine cells
        for row in 0..game.height {
            for col in 0..game.width {
                if !game.grid[row][col].has_mine {
                    game.grid[row][col].revealed = true;
                }
            }
        }

        // Check win condition
        check_win_condition(&mut game);
        assert_eq!(
            game.game_result,
            Some(ChallengeResult::Win),
            "Game should be won when all non-mine cells are revealed"
        );
    }

    #[test]
    fn test_flood_fill_reveal() {
        let mut game = MinesweeperGame::new(ChallengeDifficulty::Novice);

        // Place a mine in the corner - this leaves a large area with 0 adjacent mines
        game.grid[0][0].has_mine = true;
        calculate_adjacent_counts(&mut game);
        game.first_click_done = true;

        // Reveal a cell far from the mine (should trigger flood fill)
        let result = reveal_cell(&mut game, 8, 8);
        assert!(result, "Should return true for safe cell");

        // The bottom-right area should have multiple cells revealed via flood fill
        // Cell (8, 8) should be revealed
        assert!(game.grid[8][8].revealed, "Clicked cell should be revealed");

        // Check that neighbors were also revealed (flood fill behavior)
        // The exact pattern depends on the mine placement, but cells far from the mine
        // should be revealed
        let revealed_count: usize = game
            .grid
            .iter()
            .flat_map(|row| row.iter())
            .filter(|cell| cell.revealed)
            .count();

        // Should have revealed more than just the clicked cell
        assert!(
            revealed_count > 1,
            "Flood fill should reveal multiple cells, got {}",
            revealed_count
        );
    }

    #[test]
    fn test_handle_first_click() {
        let mut game = MinesweeperGame::new(ChallengeDifficulty::Novice);
        let mut rng = StdRng::seed_from_u64(42);

        assert!(!game.first_click_done);

        // Handle first click at center
        handle_first_click(&mut game, 4, 4, &mut rng);

        assert!(game.first_click_done, "first_click_done should be set");
        assert!(
            game.grid[4][4].revealed,
            "First click cell should be revealed"
        );
        assert!(
            !game.grid[4][4].has_mine,
            "First click cell should not have a mine"
        );

        // Count mines to verify they were placed
        let mine_count: usize = game
            .grid
            .iter()
            .flat_map(|row| row.iter())
            .filter(|cell| cell.has_mine)
            .count();
        assert_eq!(mine_count, 10, "Should have 10 mines placed");
    }

    #[test]
    fn test_reveal_all_mines() {
        let mut game = MinesweeperGame::new(ChallengeDifficulty::Novice);

        // Place mines at known positions
        game.grid[0][0].has_mine = true;
        game.grid[1][1].has_mine = true;
        game.grid[2][2].has_mine = true;

        // Mines should not be revealed initially
        assert!(!game.grid[0][0].revealed);
        assert!(!game.grid[1][1].revealed);
        assert!(!game.grid[2][2].revealed);

        reveal_all_mines(&mut game);

        // All mines should now be revealed
        assert!(game.grid[0][0].revealed, "Mine at (0,0) should be revealed");
        assert!(game.grid[1][1].revealed, "Mine at (1,1) should be revealed");
        assert!(game.grid[2][2].revealed, "Mine at (2,2) should be revealed");

        // Non-mine cells should not be revealed
        assert!(
            !game.grid[3][3].revealed,
            "Non-mine cell should not be revealed"
        );
    }

    #[test]
    fn test_cannot_flag_revealed_cell() {
        let mut game = MinesweeperGame::new(ChallengeDifficulty::Novice);

        // Reveal a cell
        game.grid[3][3].revealed = true;

        // Try to flag the revealed cell
        toggle_flag(&mut game, 3, 3);

        // Should not be flagged
        assert!(
            !game.grid[3][3].flagged,
            "Revealed cell should not be flagged"
        );
        assert_eq!(game.flags_placed, 0, "No flags should be placed");
    }

    // ============ process_input Tests ============

    #[test]
    fn test_process_input_cursor_movement() {
        let mut game = MinesweeperGame::new(ChallengeDifficulty::Novice);
        let mut rng = StdRng::seed_from_u64(42);

        // Start at center (4, 4) for 9x9 Novice grid
        assert_eq!(game.cursor, (4, 4));

        process_input(&mut game, MinigameInput::Down, &mut rng);
        assert_eq!(game.cursor, (5, 4));

        process_input(&mut game, MinigameInput::Right, &mut rng);
        assert_eq!(game.cursor, (5, 5));

        process_input(&mut game, MinigameInput::Up, &mut rng);
        assert_eq!(game.cursor, (4, 5));

        process_input(&mut game, MinigameInput::Left, &mut rng);
        assert_eq!(game.cursor, (4, 4));
    }

    #[test]
    fn test_process_input_reveal_first_click() {
        let mut game = MinesweeperGame::new(ChallengeDifficulty::Novice);
        let mut rng = StdRng::seed_from_u64(42);

        assert!(!game.first_click_done);

        // Cursor starts at center (4, 4), reveal there
        assert_eq!(game.cursor, (4, 4));
        process_input(&mut game, MinigameInput::Primary, &mut rng);

        assert!(game.first_click_done);
        assert!(game.grid[4][4].revealed);
    }

    #[test]
    fn test_process_input_toggle_flag() {
        let mut game = MinesweeperGame::new(ChallengeDifficulty::Novice);
        let mut rng = StdRng::seed_from_u64(42);

        // Cursor starts at center (4, 4)
        let (row, col) = game.cursor;
        assert!(!game.grid[row][col].flagged);
        assert_eq!(game.flags_placed, 0);

        process_input(&mut game, MinigameInput::Secondary, &mut rng);

        assert!(game.grid[row][col].flagged);
        assert_eq!(game.flags_placed, 1);

        // Toggle off
        process_input(&mut game, MinigameInput::Secondary, &mut rng);

        assert!(!game.grid[row][col].flagged);
        assert_eq!(game.flags_placed, 0);
    }

    #[test]
    fn test_process_input_forfeit_single_esc() {
        let mut game = MinesweeperGame::new(ChallengeDifficulty::Novice);
        let mut rng = StdRng::seed_from_u64(42);

        assert!(!game.forfeit_pending);

        process_input(&mut game, MinigameInput::Cancel, &mut rng);

        assert!(game.forfeit_pending);
        assert!(game.game_result.is_none());
    }

    #[test]
    fn test_process_input_forfeit_double_esc() {
        let mut game = MinesweeperGame::new(ChallengeDifficulty::Novice);
        let mut rng = StdRng::seed_from_u64(42);

        // First Esc sets pending
        process_input(&mut game, MinigameInput::Cancel, &mut rng);
        assert!(game.forfeit_pending);

        // Second Esc confirms forfeit
        process_input(&mut game, MinigameInput::Cancel, &mut rng);

        assert_eq!(game.game_result, Some(ChallengeResult::Forfeit));
    }

    #[test]
    fn test_process_input_forfeit_cancelled() {
        let mut game = MinesweeperGame::new(ChallengeDifficulty::Novice);
        let mut rng = StdRng::seed_from_u64(42);

        // First Esc sets pending
        process_input(&mut game, MinigameInput::Cancel, &mut rng);
        assert!(game.forfeit_pending);

        // Any other key cancels forfeit
        process_input(&mut game, MinigameInput::Other, &mut rng);

        assert!(!game.forfeit_pending);
        assert!(game.game_result.is_none());
    }

    // ============ Flood Fill / Reveal Logic Tests ============

    /// Helper: build a MinesweeperGame from a string grid.
    ///
    /// Characters:
    ///   'M' = mine
    ///   '.' = empty (no mine)
    ///
    /// The grid is set up with `first_click_done = true`, `total_mines` counted
    /// from the layout, and `adjacent_mines` calculated automatically.
    fn make_game(layout: &[&str]) -> MinesweeperGame {
        let height = layout.len();
        let width = layout[0].len();
        let mut grid = vec![vec![super::super::Cell::default(); width]; height];
        let mut mine_count = 0u16;

        for (r, row_str) in layout.iter().enumerate() {
            for (c, ch) in row_str.chars().enumerate() {
                if ch == 'M' {
                    grid[r][c].has_mine = true;
                    mine_count += 1;
                }
            }
        }

        let mut game = MinesweeperGame {
            grid,
            height,
            width,
            cursor: (0, 0),
            difficulty: ChallengeDifficulty::Novice,
            game_result: None,
            first_click_done: true,
            total_mines: mine_count,
            flags_placed: 0,
            forfeit_pending: false,
        };

        calculate_adjacent_counts(&mut game);
        game
    }

    /// Count how many cells are revealed in the game.
    fn count_revealed(game: &MinesweeperGame) -> usize {
        game.grid
            .iter()
            .flat_map(|row| row.iter())
            .filter(|cell| cell.revealed)
            .count()
    }

    // ---- Flood fill on empty cell reveals neighbors ----

    #[test]
    fn test_flood_fill_reveals_entire_empty_region() {
        // A 5x5 grid with mines forming a wall across row 2.
        // The bottom region (rows 3-4) is completely mine-free and
        // disconnected from the top region by the numbered border.
        //
        // Layout:
        //   . . . . .    row 0: all empty, adjacent_mines varies
        //   . . . . .    row 1: border row (numbered cells next to mines)
        //   M M M M M    row 2: mine wall
        //   . . . . .    row 3: empty
        //   . . . . .    row 4: empty
        //
        // Rows 3-4 have adjacent_mines = 0 for interior cells of row 4,
        // and row 3 cells adjacent to the mine wall will have counts > 0.
        let game = make_game(&[".....", ".....", "MMMMM", ".....", "....."]);

        // Verify adjacent counts for key cells.
        // Row 3 cells are adjacent to mine wall, so they have counts.
        assert!(game.grid[3][0].adjacent_mines > 0); // next to mines
        assert!(game.grid[3][2].adjacent_mines > 0); // next to mines
                                                     // Row 4 cells are far from mines.
        assert_eq!(game.grid[4][0].adjacent_mines, 0);
        assert_eq!(game.grid[4][2].adjacent_mines, 0);
        assert_eq!(game.grid[4][4].adjacent_mines, 0);

        // Reveal cell (4, 2) -- has 0 adjacent mines, should flood fill.
        let mut game = game;
        let result = reveal_cell(&mut game, 4, 2);
        assert!(result, "Revealing empty cell should succeed");
        assert!(game.game_result.is_none());

        // All row-4 cells should be revealed (they are all 0-adjacent).
        for c in 0..5 {
            assert!(
                game.grid[4][c].revealed,
                "Row 4, col {} should be revealed by flood fill",
                c
            );
        }

        // Row 3 cells should also be revealed (flood fill reveals numbered
        // neighbors but stops propagating from them).
        for c in 0..5 {
            assert!(
                game.grid[3][c].revealed,
                "Row 3, col {} should be revealed (numbered border)",
                c
            );
        }

        // Mine cells in row 2 should NOT be revealed.
        for c in 0..5 {
            assert!(
                !game.grid[2][c].revealed,
                "Mine at row 2, col {} should NOT be revealed",
                c
            );
        }
    }

    // ---- Flood fill stops at numbered cells ----

    #[test]
    fn test_flood_fill_stops_at_numbered_cells() {
        // 5x5 grid with a single mine at (0, 0).
        // Cells adjacent to the mine have counts > 0; the rest are 0.
        //
        //   M . . . .
        //   . . . . .
        //   . . . . .
        //   . . . . .
        //   . . . . .
        //
        // (0,1) has adjacent_mines=1, (1,0) has 1, (1,1) has 1.
        // All other non-mine cells have adjacent_mines=0.
        let mut game = make_game(&["M....", ".....", ".....", ".....", "....."]);

        // Verify the numbered border around the mine.
        assert_eq!(game.grid[0][1].adjacent_mines, 1);
        assert_eq!(game.grid[1][0].adjacent_mines, 1);
        assert_eq!(game.grid[1][1].adjacent_mines, 1);
        // Cells beyond the border should be 0.
        assert_eq!(game.grid[0][2].adjacent_mines, 0);
        assert_eq!(game.grid[2][0].adjacent_mines, 0);
        assert_eq!(game.grid[2][2].adjacent_mines, 0);

        // Reveal (4, 4) -- far from the mine, 0 adjacent.
        reveal_cell(&mut game, 4, 4);

        // The flood fill should spread across the large empty area and stop
        // at the numbered border around the mine.
        // The numbered cells themselves should be revealed.
        assert!(game.grid[0][1].revealed, "(0,1) numbered cell revealed");
        assert!(game.grid[1][0].revealed, "(1,0) numbered cell revealed");
        assert!(game.grid[1][1].revealed, "(1,1) numbered cell revealed");

        // The mine itself must NOT be revealed.
        assert!(
            !game.grid[0][0].revealed,
            "Mine at (0,0) must not be revealed"
        );

        // Cells far from mine should all be revealed.
        assert!(game.grid[4][4].revealed);
        assert!(game.grid[3][3].revealed);
        assert!(game.grid[2][2].revealed);
        assert!(game.grid[0][4].revealed);
        assert!(game.grid[4][0].revealed);

        // Total revealed should be all 24 non-mine cells.
        assert_eq!(count_revealed(&game), 24);
    }

    // ---- Flood fill does not cross into mine cells ----

    #[test]
    fn test_flood_fill_never_reveals_mines() {
        // 4x4 grid with mines forming a diagonal.
        //   M . . .
        //   . M . .
        //   . . M .
        //   . . . M
        let mut game = make_game(&["M...", ".M..", "..M.", "...M"]);

        // Reveal (0, 3) -- top-right corner, no mine.
        reveal_cell(&mut game, 0, 3);

        // No mine cell should ever be revealed by flood fill.
        assert!(!game.grid[0][0].revealed, "Mine (0,0) not revealed");
        assert!(!game.grid[1][1].revealed, "Mine (1,1) not revealed");
        assert!(!game.grid[2][2].revealed, "Mine (2,2) not revealed");
        assert!(!game.grid[3][3].revealed, "Mine (3,3) not revealed");
    }

    // ---- Revealing a numbered cell only reveals that single cell ----

    #[test]
    fn test_reveal_numbered_cell_no_flood_fill() {
        // 3x3 grid with mine at center.
        //   . . .
        //   . M .
        //   . . .
        //
        // All 8 surrounding cells have adjacent_mines = 1.
        let mut game = make_game(&["...", ".M.", "..."]);

        // Verify all border cells are numbered.
        for r in 0..3 {
            for c in 0..3 {
                if !(r == 1 && c == 1) {
                    assert_eq!(
                        game.grid[r][c].adjacent_mines, 1,
                        "Cell ({},{}) should have adjacent_mines=1",
                        r, c
                    );
                }
            }
        }

        // Reveal (0, 0) -- adjacent_mines=1, should only reveal that cell.
        let result = reveal_cell(&mut game, 0, 0);
        assert!(result);
        assert!(game.grid[0][0].revealed);

        // No other cell should be revealed.
        assert_eq!(
            count_revealed(&game),
            1,
            "Only the clicked numbered cell should be revealed"
        );
    }

    // ---- Revealing a mine triggers game over ----

    #[test]
    fn test_reveal_mine_triggers_loss_and_reveals_all_mines() {
        // 3x3 grid with mines at two positions.
        //   M . .
        //   . . .
        //   . . M
        let mut game = make_game(&["M..", "...", "..M"]);

        // Initially no mines are revealed.
        assert!(!game.grid[0][0].revealed);
        assert!(!game.grid[2][2].revealed);

        // Reveal the mine at (0, 0).
        let result = reveal_cell(&mut game, 0, 0);
        assert!(!result, "Hitting a mine should return false");
        assert_eq!(game.game_result, Some(ChallengeResult::Loss));

        // All mines should be revealed after loss.
        assert!(game.grid[0][0].revealed, "Hit mine revealed");
        assert!(game.grid[2][2].revealed, "Other mine also revealed on loss");
    }

    // ---- Flagged cells are not revealed by flood fill ----

    #[test]
    fn test_flood_fill_skips_flagged_cells() {
        // 4x4 grid with a single mine at (0, 0).
        //   M . . .
        //   . . . .
        //   . . . .
        //   . . . .
        let mut game = make_game(&["M...", "....", "....", "...."]);

        // Flag cell (3, 3) before revealing.
        toggle_flag(&mut game, 3, 3);
        assert!(game.grid[3][3].flagged);

        // Reveal (3, 0) -- has 0 adjacent mines, triggers flood fill.
        reveal_cell(&mut game, 3, 0);

        // The flagged cell should NOT be revealed even though flood fill
        // reaches its neighbors.
        assert!(
            !game.grid[3][3].revealed,
            "Flagged cell must not be revealed by flood fill"
        );
        assert!(game.grid[3][3].flagged, "Flag should remain intact");

        // Cells near the flagged cell should still be revealed.
        assert!(
            game.grid[3][2].revealed,
            "Neighbor of flagged cell revealed"
        );
        assert!(
            game.grid[2][3].revealed,
            "Neighbor of flagged cell revealed"
        );
    }

    // ---- Already-revealed cells are skipped by flood fill ----

    #[test]
    fn test_flood_fill_skips_already_revealed_cells() {
        // 4x4 grid with a single mine at (0, 0).
        //   M . . .
        //   . . . .
        //   . . . .
        //   . . . .
        let mut game = make_game(&["M...", "....", "....", "...."]);

        // Pre-reveal some cells manually.
        game.grid[3][3].revealed = true;
        game.grid[2][2].revealed = true;

        // Reveal (3, 0) -- triggers flood fill.
        reveal_cell(&mut game, 3, 0);

        // Previously revealed cells should still be revealed (not toggled off).
        assert!(game.grid[3][3].revealed, "Pre-revealed cell stays revealed");
        assert!(game.grid[2][2].revealed, "Pre-revealed cell stays revealed");

        // The flood fill should still reveal the other cells.
        assert!(game.grid[3][0].revealed, "Clicked cell revealed");
        assert!(game.grid[3][1].revealed, "Neighbor revealed by flood fill");
    }

    // ---- Edge/corner cells handle boundaries correctly ----

    #[test]
    fn test_flood_fill_from_corner_cell() {
        // 4x4 grid with a mine at (0, 3) -- top-right corner.
        //   . . . M
        //   . . . .
        //   . . . .
        //   . . . .
        let mut game = make_game(&["...M", "....", "....", "...."]);

        // Reveal corner (0, 0) -- should trigger flood fill.
        reveal_cell(&mut game, 0, 0);

        // The flood fill should propagate from (0,0) which has 0 adjacent mines.
        assert!(game.grid[0][0].revealed);

        // All non-mine cells should be revealed since they are all reachable.
        let total_non_mine = (4 * 4) - 1; // 15 non-mine cells
        assert_eq!(
            count_revealed(&game),
            total_non_mine,
            "All non-mine cells should be revealed from corner flood fill"
        );

        // Mine must not be revealed.
        assert!(!game.grid[0][3].revealed, "Mine should not be revealed");
    }

    #[test]
    fn test_flood_fill_from_edge_cell() {
        // 5x5 grid with mines forming a vertical wall at column 2.
        //   . . M . .
        //   . . M . .
        //   . . M . .
        //   . . M . .
        //   . . M . .
        let mut game = make_game(&["..M..", "..M..", "..M..", "..M..", "..M.."]);

        // Reveal (0, 0) -- top-left corner, left of the mine wall.
        reveal_cell(&mut game, 0, 0);

        // Flood fill should reveal the left region and stop at the mine wall.
        // Left side: columns 0-1 (10 cells), but col 1 cells are numbered
        // (adjacent to mines in col 2), so flood fill reveals them but
        // doesn't propagate from them.
        for r in 0..5 {
            assert!(
                game.grid[r][0].revealed,
                "Cell ({}, 0) should be revealed",
                r
            );
            assert!(
                game.grid[r][1].revealed,
                "Cell ({}, 1) should be revealed (numbered border)",
                r
            );
        }

        // Mine wall should not be revealed.
        for r in 0..5 {
            assert!(
                !game.grid[r][2].revealed,
                "Mine ({}, 2) should NOT be revealed",
                r
            );
        }

        // Right side of wall should NOT be revealed (flood fill can't cross mines).
        for r in 0..5 {
            assert!(
                !game.grid[r][3].revealed,
                "Cell ({}, 3) should NOT be revealed (other side of wall)",
                r
            );
            assert!(
                !game.grid[r][4].revealed,
                "Cell ({}, 4) should NOT be revealed (other side of wall)",
                r
            );
        }
    }

    // ---- Win condition: all non-mine cells revealed ----

    #[test]
    fn test_win_condition_triggered_by_reveal_cell() {
        // 3x3 grid with 1 mine at (0, 0).
        //   M . .
        //   . . .
        //   . . .
        //
        // 8 non-mine cells. Reveal all non-mine cells via reveal_cell
        // and verify the win condition triggers automatically.
        let mut game = make_game(&["M..", "...", "..."]);

        // Manually reveal all non-mine cells except one, then reveal the last
        // one through reveal_cell to test that check_win_condition fires.
        // The numbered cells around the mine: (0,1), (1,0), (1,1).
        // The zero-adjacent cells: (0,2), (1,2), (2,0), (2,1), (2,2).

        // Reveal all numbered cells first (they don't trigger flood fill).
        reveal_cell(&mut game, 0, 1);
        assert!(game.game_result.is_none());
        reveal_cell(&mut game, 1, 0);
        assert!(game.game_result.is_none());
        reveal_cell(&mut game, 1, 1);
        assert!(game.game_result.is_none());

        // Now reveal one zero-adjacent cell -- flood fill will reveal the rest.
        reveal_cell(&mut game, 2, 2);

        // All 8 non-mine cells should now be revealed.
        assert_eq!(count_revealed(&game), 8);

        // Win condition should have been triggered.
        assert_eq!(
            game.game_result,
            Some(ChallengeResult::Win),
            "Game should be won when all non-mine cells are revealed"
        );
    }

    #[test]
    fn test_no_win_when_cells_remain_unrevealed() {
        // 3x3 grid with mine at (1, 1).
        //   . . .
        //   . M .
        //   . . .
        let mut game = make_game(&["...", ".M.", "..."]);

        // Reveal only one cell.
        reveal_cell(&mut game, 0, 0);
        assert_eq!(count_revealed(&game), 1);
        assert!(
            game.game_result.is_none(),
            "Game should not be won with unrevealed cells remaining"
        );
    }

    // ---- Flood fill on a 1x1 grid (extreme boundary) ----

    #[test]
    fn test_flood_fill_single_cell_no_mine() {
        // 1x1 grid with no mine.
        let mut game = make_game(&["."]);
        assert_eq!(game.grid[0][0].adjacent_mines, 0);

        reveal_cell(&mut game, 0, 0);
        assert!(game.grid[0][0].revealed);
        // With 0 mines and 1 cell, revealing it should win.
        assert_eq!(game.game_result, Some(ChallengeResult::Win));
    }

    // ---- Flood fill with multiple disjoint empty regions ----

    #[test]
    fn test_flood_fill_does_not_cross_numbered_boundary_into_second_region() {
        // 5x1 grid (single column): mine in the middle separates two regions.
        //   .
        //   .
        //   M
        //   .
        //   .
        let mut game = make_game(&[".", ".", "M", ".", "."]);

        // Cells adjacent to the mine: (1,0) and (3,0) have adjacent_mines=1.
        assert_eq!(game.grid[1][0].adjacent_mines, 1);
        assert_eq!(game.grid[3][0].adjacent_mines, 1);
        // Cells far from mine: (0,0) and (4,0) have adjacent_mines=0.
        assert_eq!(game.grid[0][0].adjacent_mines, 0);
        assert_eq!(game.grid[4][0].adjacent_mines, 0);

        // Reveal (0, 0) -- triggers flood fill in the top region.
        reveal_cell(&mut game, 0, 0);

        // Top region: (0,0) revealed via click, (1,0) revealed as numbered border.
        assert!(game.grid[0][0].revealed, "(0,0) revealed");
        assert!(game.grid[1][0].revealed, "(1,0) numbered border revealed");

        // Mine not revealed.
        assert!(!game.grid[2][0].revealed, "Mine not revealed");

        // Bottom region NOT revealed -- flood fill stops at numbered (1,0)
        // and cannot reach past the mine.
        assert!(!game.grid[3][0].revealed, "(3,0) not revealed");
        assert!(!game.grid[4][0].revealed, "(4,0) not revealed");
    }

    // ---- Reveal already-revealed cell is a no-op ----

    #[test]
    fn test_reveal_already_revealed_cell_is_noop() {
        let mut game = make_game(&["M..", "...", "..."]);

        // Reveal (2, 2).
        reveal_cell(&mut game, 2, 2);
        let revealed_after_first = count_revealed(&game);

        // Reveal (2, 2) again -- should be a no-op.
        let result = reveal_cell(&mut game, 2, 2);
        assert!(result, "Re-revealing should return true (no action)");
        assert_eq!(
            count_revealed(&game),
            revealed_after_first,
            "Revealed count should not change on re-reveal"
        );
    }

    // ---- Flood fill with L-shaped mine barrier ----

    #[test]
    fn test_flood_fill_with_l_shaped_mine_barrier() {
        // 5x5 grid with L-shaped mine barrier separating top-left from bottom-right.
        //   . . M . .
        //   . . M . .
        //   M M M . .
        //   . . . . .
        //   . . . . .
        let mut game = make_game(&["..M..", "..M..", "MMM..", ".....", "....."]);

        // Reveal (0, 0) in the top-left pocket.
        reveal_cell(&mut game, 0, 0);

        // Top-left pocket: (0,0), (0,1), (1,0), (1,1) should be revealed.
        // (0,0) has adjacent_mines=0 (no mine neighbors), (0,1) has 1 (adj to (0,2)),
        // (1,0) has 1 (adj to (2,0)), (1,1) has 3 (adj to (0,2),(1,2),(2,0),(2,1),(2,2)).
        // Actually let me check: (0,0) neighbors are (0,1),(1,0),(1,1).
        // None of those are mines. (0,0) adjacent_mines=0.
        assert_eq!(game.grid[0][0].adjacent_mines, 0);

        assert!(game.grid[0][0].revealed, "(0,0) revealed");
        assert!(game.grid[0][1].revealed, "(0,1) revealed via flood fill");
        assert!(game.grid[1][0].revealed, "(1,0) revealed via flood fill");
        assert!(game.grid[1][1].revealed, "(1,1) revealed via flood fill");

        // Mines should NOT be revealed.
        assert!(!game.grid[0][2].revealed);
        assert!(!game.grid[1][2].revealed);
        assert!(!game.grid[2][0].revealed);
        assert!(!game.grid[2][1].revealed);
        assert!(!game.grid[2][2].revealed);

        // Bottom-right region should NOT be revealed.
        assert!(!game.grid[3][0].revealed);
        assert!(!game.grid[4][4].revealed);
    }

    // ---- Flood fill on a large empty board (no mines) ----

    #[test]
    fn test_flood_fill_entire_board_no_mines() {
        // 4x4 grid with zero mines.
        let mut game = make_game(&["....", "....", "....", "...."]);
        assert_eq!(game.total_mines, 0);

        // Reveal (0, 0) -- should flood fill the entire board.
        reveal_cell(&mut game, 0, 0);

        // Every cell should be revealed.
        assert_eq!(count_revealed(&game), 16);

        // Win condition: unrevealed == total_mines == 0 -> Win.
        assert_eq!(game.game_result, Some(ChallengeResult::Win));
    }

    // ---- Flagged cell adjacent to empty region blocks propagation locally ----

    #[test]
    fn test_flood_fill_propagates_around_flagged_cell() {
        // 3x3 grid with no mines. All cells have adjacent_mines=0.
        //   . . .
        //   . . .
        //   . . .
        let mut game = make_game(&["...", "...", "..."]);

        // Flag the center cell.
        toggle_flag(&mut game, 1, 1);

        // Reveal (0, 0) -- flood fill should spread everywhere except (1,1).
        reveal_cell(&mut game, 0, 0);

        // 8 cells revealed (all except the flagged center).
        assert_eq!(count_revealed(&game), 8);
        assert!(
            !game.grid[1][1].revealed,
            "Flagged center should not be revealed"
        );

        // Cells on the far side of the flag should still be revealed
        // because flood fill goes around via other paths.
        assert!(game.grid[2][2].revealed, "(2,2) revealed around flag");
        assert!(game.grid[2][0].revealed, "(2,0) revealed around flag");
        assert!(game.grid[0][2].revealed, "(0,2) revealed around flag");
    }

    // ---- Reveal via process_input triggers flood fill correctly ----

    #[test]
    fn test_process_input_reveal_triggers_flood_fill() {
        let mut game = make_game(&["M...", "....", "....", "...."]);
        let mut rng = StdRng::seed_from_u64(42);

        // Move cursor to (3, 3) and reveal.
        game.cursor = (3, 3);
        process_input(&mut game, MinigameInput::Primary, &mut rng);

        // Cell (3,3) has 0 adjacent mines, flood fill should occur.
        assert!(game.grid[3][3].revealed);
        assert!(
            count_revealed(&game) > 1,
            "Flood fill via process_input should reveal multiple cells"
        );

        // Mine should not be revealed.
        assert!(!game.grid[0][0].revealed);
    }
}
