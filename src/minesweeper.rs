//! Minesweeper minigame data structures.
//!
//! Classic minesweeper with variable grid sizes and mine counts.

use serde::{Deserialize, Serialize};

/// Represents a single cell in the minesweeper grid.
#[derive(Debug, Clone, Copy, Default)]
pub struct Cell {
    /// Whether this cell contains a mine.
    pub has_mine: bool,
    /// Whether this cell has been revealed.
    pub revealed: bool,
    /// Whether this cell has been flagged by the player.
    pub flagged: bool,
    /// Number of adjacent mines (0-8).
    pub adjacent_mines: u8,
}

/// Difficulty levels for Minesweeper with varying grid sizes and mine counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MinesweeperDifficulty {
    Novice,     // 9x9, 10 mines
    Apprentice, // 12x12, 25 mines
    Journeyman, // 16x16, 40 mines
    Master,     // 16x20, 60 mines
}

impl MinesweeperDifficulty {
    pub const ALL: [MinesweeperDifficulty; 4] = [
        MinesweeperDifficulty::Novice,
        MinesweeperDifficulty::Apprentice,
        MinesweeperDifficulty::Journeyman,
        MinesweeperDifficulty::Master,
    ];

    pub fn from_index(index: usize) -> Self {
        Self::ALL
            .get(index)
            .copied()
            .unwrap_or(MinesweeperDifficulty::Novice)
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Novice => "Novice",
            Self::Apprentice => "Apprentice",
            Self::Journeyman => "Journeyman",
            Self::Master => "Master",
        }
    }

    /// Returns (height, width) for the grid.
    pub fn grid_size(&self) -> (usize, usize) {
        match self {
            Self::Novice => (9, 9),
            Self::Apprentice => (12, 12),
            Self::Journeyman => (16, 16),
            Self::Master => (16, 20),
        }
    }

    /// Returns the number of mines for this difficulty.
    pub fn mine_count(&self) -> u16 {
        match self {
            Self::Novice => 10,
            Self::Apprentice => 25,
            Self::Journeyman => 40,
            Self::Master => 60,
        }
    }
}

/// Result of a completed minesweeper game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinesweeperResult {
    Win,
    Loss,
}

/// Active minesweeper game session.
#[derive(Debug, Clone)]
pub struct MinesweeperGame {
    /// The game grid, indexed as grid[row][col].
    pub grid: Vec<Vec<Cell>>,
    /// Grid height (number of rows).
    pub height: usize,
    /// Grid width (number of columns).
    pub width: usize,
    /// Current cursor position (row, col).
    pub cursor: (usize, usize),
    /// Difficulty level.
    pub difficulty: MinesweeperDifficulty,
    /// Game result (None if game in progress).
    pub game_result: Option<MinesweeperResult>,
    /// Whether the first click has been made (mines placed after first click).
    pub first_click_done: bool,
    /// Total number of mines in the grid.
    pub total_mines: u16,
    /// Number of flags currently placed.
    pub flags_placed: u16,
    /// Forfeit confirmation pending.
    pub forfeit_pending: bool,
}

impl MinesweeperGame {
    /// Create a new minesweeper game with the given difficulty.
    /// Note: Mines are not placed until the first reveal to ensure first click is safe.
    pub fn new(difficulty: MinesweeperDifficulty) -> Self {
        let (height, width) = difficulty.grid_size();
        let grid = vec![vec![Cell::default(); width]; height];

        Self {
            grid,
            height,
            width,
            cursor: (height / 2, width / 2), // Center of grid
            difficulty,
            game_result: None,
            first_click_done: false,
            total_mines: difficulty.mine_count(),
            flags_placed: 0,
            forfeit_pending: false,
        }
    }

    /// Move the cursor in a direction, clamping to grid bounds.
    pub fn move_cursor(&mut self, d_row: i32, d_col: i32) {
        let new_row = (self.cursor.0 as i32 + d_row).clamp(0, self.height as i32 - 1) as usize;
        let new_col = (self.cursor.1 as i32 + d_col).clamp(0, self.width as i32 - 1) as usize;
        self.cursor = (new_row, new_col);
    }

    /// Returns the number of mines remaining (total mines - flags placed).
    /// Can be negative if player has placed more flags than mines.
    pub fn mines_remaining(&self) -> i32 {
        self.total_mines as i32 - self.flags_placed as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game() {
        let game = MinesweeperGame::new(MinesweeperDifficulty::Novice);

        // Check grid dimensions
        assert_eq!(game.height, 9);
        assert_eq!(game.width, 9);
        assert_eq!(game.grid.len(), 9);
        assert_eq!(game.grid[0].len(), 9);

        // Check initial state
        assert_eq!(game.cursor, (4, 4)); // Center of 9x9
        assert_eq!(game.difficulty, MinesweeperDifficulty::Novice);
        assert!(game.game_result.is_none());
        assert!(!game.first_click_done);
        assert_eq!(game.total_mines, 10);
        assert_eq!(game.flags_placed, 0);
        assert!(!game.forfeit_pending);

        // All cells should be default (unrevealed, unflagged, no mine)
        for row in &game.grid {
            for cell in row {
                assert!(!cell.has_mine);
                assert!(!cell.revealed);
                assert!(!cell.flagged);
                assert_eq!(cell.adjacent_mines, 0);
            }
        }
    }

    #[test]
    fn test_difficulty_grid_sizes() {
        // Novice: 9x9, 10 mines
        assert_eq!(MinesweeperDifficulty::Novice.grid_size(), (9, 9));
        assert_eq!(MinesweeperDifficulty::Novice.mine_count(), 10);

        // Apprentice: 12x12, 25 mines
        assert_eq!(MinesweeperDifficulty::Apprentice.grid_size(), (12, 12));
        assert_eq!(MinesweeperDifficulty::Apprentice.mine_count(), 25);

        // Journeyman: 16x16, 40 mines
        assert_eq!(MinesweeperDifficulty::Journeyman.grid_size(), (16, 16));
        assert_eq!(MinesweeperDifficulty::Journeyman.mine_count(), 40);

        // Master: 16x20, 60 mines
        assert_eq!(MinesweeperDifficulty::Master.grid_size(), (16, 20));
        assert_eq!(MinesweeperDifficulty::Master.mine_count(), 60);
    }

    #[test]
    fn test_move_cursor() {
        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);

        // Start at center (4, 4)
        assert_eq!(game.cursor, (4, 4));

        // Move right
        game.move_cursor(0, 1);
        assert_eq!(game.cursor, (4, 5));

        // Move down
        game.move_cursor(1, 0);
        assert_eq!(game.cursor, (5, 5));

        // Move up-left
        game.move_cursor(-1, -1);
        assert_eq!(game.cursor, (4, 4));

        // Move to corner and try to go out of bounds
        game.cursor = (0, 0);
        game.move_cursor(-1, -1);
        assert_eq!(game.cursor, (0, 0)); // Clamped

        // Move to opposite corner
        game.cursor = (8, 8);
        game.move_cursor(1, 1);
        assert_eq!(game.cursor, (8, 8)); // Clamped
    }

    #[test]
    fn test_mines_remaining() {
        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);

        // Initially: 10 mines, 0 flags
        assert_eq!(game.mines_remaining(), 10);

        // Place some flags
        game.flags_placed = 3;
        assert_eq!(game.mines_remaining(), 7);

        // Place all flags
        game.flags_placed = 10;
        assert_eq!(game.mines_remaining(), 0);

        // Over-flag (more flags than mines)
        game.flags_placed = 15;
        assert_eq!(game.mines_remaining(), -5);
    }

    #[test]
    fn test_difficulty_from_index() {
        assert_eq!(
            MinesweeperDifficulty::from_index(0),
            MinesweeperDifficulty::Novice
        );
        assert_eq!(
            MinesweeperDifficulty::from_index(1),
            MinesweeperDifficulty::Apprentice
        );
        assert_eq!(
            MinesweeperDifficulty::from_index(2),
            MinesweeperDifficulty::Journeyman
        );
        assert_eq!(
            MinesweeperDifficulty::from_index(3),
            MinesweeperDifficulty::Master
        );
        // Out of bounds defaults to Novice
        assert_eq!(
            MinesweeperDifficulty::from_index(99),
            MinesweeperDifficulty::Novice
        );
    }

    #[test]
    fn test_difficulty_names() {
        assert_eq!(MinesweeperDifficulty::Novice.name(), "Novice");
        assert_eq!(MinesweeperDifficulty::Apprentice.name(), "Apprentice");
        assert_eq!(MinesweeperDifficulty::Journeyman.name(), "Journeyman");
        assert_eq!(MinesweeperDifficulty::Master.name(), "Master");
    }

    #[test]
    fn test_all_difficulties() {
        assert_eq!(MinesweeperDifficulty::ALL.len(), 4);
        assert_eq!(MinesweeperDifficulty::ALL[0], MinesweeperDifficulty::Novice);
        assert_eq!(
            MinesweeperDifficulty::ALL[1],
            MinesweeperDifficulty::Apprentice
        );
        assert_eq!(
            MinesweeperDifficulty::ALL[2],
            MinesweeperDifficulty::Journeyman
        );
        assert_eq!(MinesweeperDifficulty::ALL[3], MinesweeperDifficulty::Master);
    }

    #[test]
    fn test_game_with_each_difficulty() {
        for difficulty in MinesweeperDifficulty::ALL {
            let game = MinesweeperGame::new(difficulty);
            let (expected_height, expected_width) = difficulty.grid_size();

            assert_eq!(game.height, expected_height);
            assert_eq!(game.width, expected_width);
            assert_eq!(game.grid.len(), expected_height);
            assert_eq!(game.grid[0].len(), expected_width);
            assert_eq!(game.total_mines, difficulty.mine_count());

            // Cursor should be at center
            assert_eq!(game.cursor, (expected_height / 2, expected_width / 2));
        }
    }
}
