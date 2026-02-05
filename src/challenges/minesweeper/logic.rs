//! Minesweeper game logic for mine placement and cell counting.

use rand::seq::SliceRandom;
use rand::Rng;

use super::{MinesweeperGame, MinesweeperResult};

/// Input actions for the Minesweeper game (UI-agnostic).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinesweeperInput {
    Up,
    Down,
    Left,
    Right,
    Reveal,
    ToggleFlag,
    Forfeit,
    Other,
}

/// Process a key input during active Minesweeper game.
/// Returns true if the input was handled.
pub fn process_input<R: Rng>(game: &mut MinesweeperGame, input: MinesweeperInput, rng: &mut R) -> bool {
    // Handle forfeit confirmation (double-Esc pattern)
    if game.forfeit_pending {
        match input {
            MinesweeperInput::Forfeit => {
                game.game_result = Some(MinesweeperResult::Loss);
            }
            _ => {
                game.forfeit_pending = false;
            }
        }
        return true;
    }

    // Normal game input
    match input {
        MinesweeperInput::Up => game.move_cursor(-1, 0),
        MinesweeperInput::Down => game.move_cursor(1, 0),
        MinesweeperInput::Left => game.move_cursor(0, -1),
        MinesweeperInput::Right => game.move_cursor(0, 1),
        MinesweeperInput::Reveal => {
            let (row, col) = game.cursor;
            if !game.first_click_done {
                handle_first_click(game, row, col, rng);
            } else {
                reveal_cell(game, row, col);
            }
        }
        MinesweeperInput::ToggleFlag => {
            let (row, col) = game.cursor;
            toggle_flag(game, row, col);
        }
        MinesweeperInput::Forfeit => {
            game.forfeit_pending = true;
        }
        MinesweeperInput::Other => {}
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
        game.game_result = Some(MinesweeperResult::Loss);
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
        game.game_result = Some(MinesweeperResult::Win);
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

/// Apply game result: grant rewards on win (XP, prestige).
/// Returns (result, xp_gained).
pub fn apply_game_result(
    state: &mut crate::core::game_state::GameState,
) -> Option<(super::super::MinesweeperResult, u64)> {
    use super::super::MinesweeperResult;
    use crate::challenges::menu::DifficultyInfo;

    let game = state.active_minesweeper.as_ref()?;
    let result = game.game_result?;
    let reward = game.difficulty.reward();

    let xp_gained = match result {
        MinesweeperResult::Win => {
            // XP reward
            let xp = if reward.xp_percent > 0 {
                let xp_for_level =
                    crate::core::game_logic::xp_for_next_level(state.character_level.max(1));
                let xp = (xp_for_level * reward.xp_percent as u64) / 100;
                state.character_xp += xp;
                xp
            } else {
                0
            };

            // Prestige reward
            if reward.prestige_ranks > 0 {
                state.prestige_rank += reward.prestige_ranks;
            }

            xp
        }
        MinesweeperResult::Loss => 0,
    };

    state.active_minesweeper = None;
    Some((result, xp_gained))
}

#[cfg(test)]
mod tests {
    use super::super::MinesweeperDifficulty;
    use super::*;
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
                n_row, n_col
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
                    game1.grid[row][col].has_mine, game2.grid[row][col].has_mine,
                    "Mine placement differs at ({}, {})",
                    row, col
                );
            }
        }
    }

    #[test]
    fn test_reveal_safe_cell() {
        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);

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
        use super::super::MinesweeperResult;

        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);

        // Place a mine at (2, 2)
        game.grid[2][2].has_mine = true;
        game.first_click_done = true;

        // Reveal the mine
        let result = reveal_cell(&mut game, 2, 2);
        assert!(!result, "Should return false when hitting a mine");
        assert!(game.grid[2][2].revealed, "Mine cell should be revealed");
        assert_eq!(
            game.game_result,
            Some(MinesweeperResult::Loss),
            "Game should be lost"
        );
    }

    #[test]
    fn test_toggle_flag() {
        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);

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
        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);
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
        use super::super::MinesweeperResult;

        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);

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
            Some(MinesweeperResult::Win),
            "Game should be won when all non-mine cells are revealed"
        );
    }

    #[test]
    fn test_flood_fill_reveal() {
        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);

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
        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);
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
        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);

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
        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);

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

    // ============ apply_game_result Tests ============

    #[test]
    fn test_apply_win_result() {
        use crate::core::game_state::GameState;

        let mut state = GameState::new("Test".to_string(), 0);
        state.prestige_rank = 5;
        let initial_xp = state.character_xp;

        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Master);
        game.game_result = Some(MinesweeperResult::Win);
        state.active_minesweeper = Some(game);

        let result = apply_game_result(&mut state);
        assert!(result.is_some());
        let (ms_result, xp_gained) = result.unwrap();
        assert_eq!(ms_result, MinesweeperResult::Win);
        assert!(xp_gained > 0); // Master gives XP
        assert_eq!(state.character_xp, initial_xp + xp_gained);
        assert!(state.prestige_rank > 5); // Master gives prestige
        assert!(state.active_minesweeper.is_none());
    }

    #[test]
    fn test_apply_loss_result() {
        use crate::core::game_state::GameState;

        let mut state = GameState::new("Test".to_string(), 0);
        state.prestige_rank = 5;
        let initial_xp = state.character_xp;

        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);
        game.game_result = Some(MinesweeperResult::Loss);
        state.active_minesweeper = Some(game);

        let result = apply_game_result(&mut state);
        assert!(result.is_some());
        let (ms_result, xp_gained) = result.unwrap();
        assert_eq!(ms_result, MinesweeperResult::Loss);
        assert_eq!(xp_gained, 0); // No reward for loss
        assert_eq!(state.character_xp, initial_xp); // XP unchanged
        assert_eq!(state.prestige_rank, 5); // Prestige unchanged
        assert!(state.active_minesweeper.is_none());
    }

    #[test]
    fn test_apply_result_no_game() {
        use crate::core::game_state::GameState;

        let mut state = GameState::new("Test".to_string(), 0);
        state.active_minesweeper = None;

        let result = apply_game_result(&mut state);
        assert!(result.is_none());
    }

    #[test]
    fn test_apply_result_no_result() {
        use crate::core::game_state::GameState;

        let mut state = GameState::new("Test".to_string(), 0);
        let game = MinesweeperGame::new(MinesweeperDifficulty::Novice);
        // game.game_result is None
        state.active_minesweeper = Some(game);

        let result = apply_game_result(&mut state);
        assert!(result.is_none());
        // Game should still be active
        assert!(state.active_minesweeper.is_some());
    }

    // ============ process_input Tests ============

    #[test]
    fn test_process_input_cursor_movement() {
        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);
        let mut rng = StdRng::seed_from_u64(42);

        // Start at center (4, 4) for 9x9 Novice grid
        assert_eq!(game.cursor, (4, 4));

        process_input(&mut game, MinesweeperInput::Down, &mut rng);
        assert_eq!(game.cursor, (5, 4));

        process_input(&mut game, MinesweeperInput::Right, &mut rng);
        assert_eq!(game.cursor, (5, 5));

        process_input(&mut game, MinesweeperInput::Up, &mut rng);
        assert_eq!(game.cursor, (4, 5));

        process_input(&mut game, MinesweeperInput::Left, &mut rng);
        assert_eq!(game.cursor, (4, 4));
    }

    #[test]
    fn test_process_input_reveal_first_click() {
        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);
        let mut rng = StdRng::seed_from_u64(42);

        assert!(!game.first_click_done);

        // Cursor starts at center (4, 4), reveal there
        assert_eq!(game.cursor, (4, 4));
        process_input(&mut game, MinesweeperInput::Reveal, &mut rng);

        assert!(game.first_click_done);
        assert!(game.grid[4][4].revealed);
    }

    #[test]
    fn test_process_input_toggle_flag() {
        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);
        let mut rng = StdRng::seed_from_u64(42);

        // Cursor starts at center (4, 4)
        let (row, col) = game.cursor;
        assert!(!game.grid[row][col].flagged);
        assert_eq!(game.flags_placed, 0);

        process_input(&mut game, MinesweeperInput::ToggleFlag, &mut rng);

        assert!(game.grid[row][col].flagged);
        assert_eq!(game.flags_placed, 1);

        // Toggle off
        process_input(&mut game, MinesweeperInput::ToggleFlag, &mut rng);

        assert!(!game.grid[row][col].flagged);
        assert_eq!(game.flags_placed, 0);
    }

    #[test]
    fn test_process_input_forfeit_single_esc() {
        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);
        let mut rng = StdRng::seed_from_u64(42);

        assert!(!game.forfeit_pending);

        process_input(&mut game, MinesweeperInput::Forfeit, &mut rng);

        assert!(game.forfeit_pending);
        assert!(game.game_result.is_none());
    }

    #[test]
    fn test_process_input_forfeit_double_esc() {
        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);
        let mut rng = StdRng::seed_from_u64(42);

        // First Esc sets pending
        process_input(&mut game, MinesweeperInput::Forfeit, &mut rng);
        assert!(game.forfeit_pending);

        // Second Esc confirms forfeit
        process_input(&mut game, MinesweeperInput::Forfeit, &mut rng);

        assert_eq!(game.game_result, Some(MinesweeperResult::Loss));
    }

    #[test]
    fn test_process_input_forfeit_cancelled() {
        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);
        let mut rng = StdRng::seed_from_u64(42);

        // First Esc sets pending
        process_input(&mut game, MinesweeperInput::Forfeit, &mut rng);
        assert!(game.forfeit_pending);

        // Any other key cancels forfeit
        process_input(&mut game, MinesweeperInput::Other, &mut rng);

        assert!(!game.forfeit_pending);
        assert!(game.game_result.is_none());
    }
}
