# Minesweeper: Trap Detection - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a Minesweeper minigame challenge themed as "Trap Detection" with four difficulty levels.

**Architecture:** Single-player puzzle using a 2D grid of cells. Mines placed after first click to guarantee safe opening. Flood-fill reveal for empty cells. Integrates with existing challenge menu system.

**Tech Stack:** Rust, Ratatui for TUI, Serde for serialization, rand for mine placement.

---

## Task 1: Create Minesweeper Data Structures

**Files:**
- Create: `src/minesweeper.rs`
- Modify: `src/lib.rs`

**Step 1: Create the minesweeper module with basic types**

Create `src/minesweeper.rs`:

```rust
//! Minesweeper (Trap Detection) minigame data structures.
//!
//! Grid-based puzzle where player reveals cells while avoiding hidden mines.

use serde::{Deserialize, Serialize};

/// Cell state in the grid
#[derive(Debug, Clone, Copy, Default)]
pub struct Cell {
    pub has_mine: bool,
    pub revealed: bool,
    pub flagged: bool,
    pub adjacent_mines: u8,
}

/// AI difficulty levels (grid size + mine count)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MinesweeperDifficulty {
    Novice,     // 9x9, 10 mines
    Apprentice, // 12x12, 25 mines
    Journeyman, // 16x16, 40 mines
    Master,     // 20x16, 60 mines
}

impl MinesweeperDifficulty {
    pub const ALL: [MinesweeperDifficulty; 4] = [
        MinesweeperDifficulty::Novice,
        MinesweeperDifficulty::Apprentice,
        MinesweeperDifficulty::Journeyman,
        MinesweeperDifficulty::Master,
    ];

    pub fn from_index(index: usize) -> Self {
        Self::ALL.get(index).copied().unwrap_or(MinesweeperDifficulty::Novice)
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Novice => "Novice",
            Self::Apprentice => "Apprentice",
            Self::Journeyman => "Journeyman",
            Self::Master => "Master",
        }
    }

    pub fn grid_size(&self) -> (usize, usize) {
        match self {
            Self::Novice => (9, 9),
            Self::Apprentice => (12, 12),
            Self::Journeyman => (16, 16),
            Self::Master => (16, 20),
        }
    }

    pub fn mine_count(&self) -> u16 {
        match self {
            Self::Novice => 10,
            Self::Apprentice => 25,
            Self::Journeyman => 40,
            Self::Master => 60,
        }
    }
}

/// Game result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinesweeperResult {
    Win,
    Loss,
}

/// Main game state
#[derive(Debug, Clone)]
pub struct MinesweeperGame {
    /// 2D grid of cells (row-major: grid[row][col])
    pub grid: Vec<Vec<Cell>>,
    /// Grid dimensions (height, width)
    pub height: usize,
    pub width: usize,
    /// Current cursor position (row, col)
    pub cursor: (usize, usize),
    /// Difficulty level
    pub difficulty: MinesweeperDifficulty,
    /// Game result (None if game in progress)
    pub game_result: Option<MinesweeperResult>,
    /// Mines placed after first click
    pub first_click_done: bool,
    /// Total mines in grid
    pub total_mines: u16,
    /// Flags currently placed
    pub flags_placed: u16,
    /// Forfeit confirmation pending
    pub forfeit_pending: bool,
}

impl MinesweeperGame {
    pub fn new(difficulty: MinesweeperDifficulty) -> Self {
        let (height, width) = difficulty.grid_size();
        let grid = vec![vec![Cell::default(); width]; height];

        Self {
            grid,
            height,
            width,
            cursor: (height / 2, width / 2),
            difficulty,
            game_result: None,
            first_click_done: false,
            total_mines: difficulty.mine_count(),
            flags_placed: 0,
            forfeit_pending: false,
        }
    }

    pub fn move_cursor(&mut self, delta_row: i32, delta_col: i32) {
        let new_row = (self.cursor.0 as i32 + delta_row).clamp(0, self.height as i32 - 1) as usize;
        let new_col = (self.cursor.1 as i32 + delta_col).clamp(0, self.width as i32 - 1) as usize;
        self.cursor = (new_row, new_col);
    }

    pub fn mines_remaining(&self) -> i16 {
        self.total_mines as i16 - self.flags_placed as i16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game() {
        let game = MinesweeperGame::new(MinesweeperDifficulty::Novice);
        assert_eq!(game.height, 9);
        assert_eq!(game.width, 9);
        assert_eq!(game.total_mines, 10);
        assert!(!game.first_click_done);
        assert!(game.game_result.is_none());
    }

    #[test]
    fn test_difficulty_grid_sizes() {
        assert_eq!(MinesweeperDifficulty::Novice.grid_size(), (9, 9));
        assert_eq!(MinesweeperDifficulty::Apprentice.grid_size(), (12, 12));
        assert_eq!(MinesweeperDifficulty::Journeyman.grid_size(), (16, 16));
        assert_eq!(MinesweeperDifficulty::Master.grid_size(), (16, 20));
    }

    #[test]
    fn test_move_cursor() {
        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);
        game.cursor = (4, 4);

        game.move_cursor(-1, 0);
        assert_eq!(game.cursor, (3, 4));

        game.move_cursor(0, 1);
        assert_eq!(game.cursor, (3, 5));

        // Test clamping at edges
        game.cursor = (0, 0);
        game.move_cursor(-1, -1);
        assert_eq!(game.cursor, (0, 0));
    }

    #[test]
    fn test_mines_remaining() {
        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);
        assert_eq!(game.mines_remaining(), 10);

        game.flags_placed = 3;
        assert_eq!(game.mines_remaining(), 7);
    }
}
```

**Step 2: Add module to lib.rs**

Add to `src/lib.rs`:

```rust
pub mod minesweeper;
```

**Step 3: Run tests to verify**

Run: `cargo test minesweeper`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/minesweeper.rs src/lib.rs
git commit -m "feat(minesweeper): add core data structures

- MinesweeperDifficulty enum with grid sizes and mine counts
- Cell struct for grid state (mine, revealed, flagged, adjacent)
- MinesweeperGame struct with cursor movement
- Unit tests for basic functionality"
```

---

## Task 2: Implement Mine Placement and Adjacent Count

**Files:**
- Create: `src/minesweeper_logic.rs`
- Modify: `src/lib.rs`

**Step 1: Create minesweeper_logic module**

Create `src/minesweeper_logic.rs`:

```rust
//! Minesweeper game logic.

use crate::minesweeper::{Cell, MinesweeperGame, MinesweeperResult};
use rand::seq::SliceRandom;
use rand::Rng;

/// Place mines on the grid, avoiding the first-click cell and its neighbors.
pub fn place_mines<R: Rng>(game: &mut MinesweeperGame, first_row: usize, first_col: usize, rng: &mut R) {
    // Collect all valid positions (excluding first click and neighbors)
    let mut valid_positions: Vec<(usize, usize)> = Vec::new();

    for row in 0..game.height {
        for col in 0..game.width {
            // Skip first click cell and its 8 neighbors
            let row_diff = (row as i32 - first_row as i32).abs();
            let col_diff = (col as i32 - first_col as i32).abs();
            if row_diff <= 1 && col_diff <= 1 {
                continue;
            }
            valid_positions.push((row, col));
        }
    }

    // Shuffle and take mine_count positions
    valid_positions.shuffle(rng);
    let mine_count = game.total_mines as usize;

    for &(row, col) in valid_positions.iter().take(mine_count) {
        game.grid[row][col].has_mine = true;
    }

    // Calculate adjacent mine counts for all cells
    calculate_adjacent_counts(game);
}

/// Calculate adjacent mine counts for all cells.
fn calculate_adjacent_counts(game: &mut MinesweeperGame) {
    for row in 0..game.height {
        for col in 0..game.width {
            if game.grid[row][col].has_mine {
                continue;
            }

            let mut count = 0u8;
            for dr in -1i32..=1 {
                for dc in -1i32..=1 {
                    if dr == 0 && dc == 0 {
                        continue;
                    }
                    let nr = row as i32 + dr;
                    let nc = col as i32 + dc;
                    if nr >= 0 && nr < game.height as i32 && nc >= 0 && nc < game.width as i32 {
                        if game.grid[nr as usize][nc as usize].has_mine {
                            count += 1;
                        }
                    }
                }
            }
            game.grid[row][col].adjacent_mines = count;
        }
    }
}

/// Get list of neighbor coordinates for a cell.
fn get_neighbors(row: usize, col: usize, height: usize, width: usize) -> Vec<(usize, usize)> {
    let mut neighbors = Vec::new();
    for dr in -1i32..=1 {
        for dc in -1i32..=1 {
            if dr == 0 && dc == 0 {
                continue;
            }
            let nr = row as i32 + dr;
            let nc = col as i32 + dc;
            if nr >= 0 && nr < height as i32 && nc >= 0 && nc < width as i32 {
                neighbors.push((nr as usize, nc as usize));
            }
        }
    }
    neighbors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::minesweeper::MinesweeperDifficulty;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_place_mines_count() {
        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);
        let mut rng = StdRng::seed_from_u64(42);

        place_mines(&mut game, 4, 4, &mut rng);

        let mine_count: usize = game.grid.iter()
            .flat_map(|row| row.iter())
            .filter(|cell| cell.has_mine)
            .count();

        assert_eq!(mine_count, 10);
    }

    #[test]
    fn test_place_mines_avoids_first_click() {
        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);
        let mut rng = StdRng::seed_from_u64(42);
        let first_row = 4;
        let first_col = 4;

        place_mines(&mut game, first_row, first_col, &mut rng);

        // Check that first click and neighbors have no mines
        for dr in -1i32..=1 {
            for dc in -1i32..=1 {
                let r = (first_row as i32 + dr) as usize;
                let c = (first_col as i32 + dc) as usize;
                assert!(!game.grid[r][c].has_mine, "Mine at ({}, {}) near first click", r, c);
            }
        }
    }

    #[test]
    fn test_adjacent_counts() {
        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);

        // Manually place mines in known positions
        game.grid[0][0].has_mine = true;
        game.grid[0][1].has_mine = true;

        calculate_adjacent_counts(&mut game);

        // Cell at (1, 0) should have 2 adjacent mines
        assert_eq!(game.grid[1][0].adjacent_mines, 2);
        // Cell at (1, 1) should have 2 adjacent mines
        assert_eq!(game.grid[1][1].adjacent_mines, 2);
        // Cell at (0, 2) should have 1 adjacent mine
        assert_eq!(game.grid[0][2].adjacent_mines, 1);
    }
}
```

**Step 2: Add module to lib.rs**

Add to `src/lib.rs`:

```rust
pub mod minesweeper_logic;
```

**Step 3: Run tests**

Run: `cargo test minesweeper`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/minesweeper_logic.rs src/lib.rs
git commit -m "feat(minesweeper): add mine placement logic

- place_mines() avoids first click cell and neighbors
- calculate_adjacent_counts() computes neighbor mine counts
- Seeded RNG tests for deterministic verification"
```

---

## Task 3: Implement Reveal and Flag Logic

**Files:**
- Modify: `src/minesweeper_logic.rs`

**Step 1: Add reveal and flag functions**

Add to `src/minesweeper_logic.rs`:

```rust
/// Reveal a cell. Returns true if game should continue, false if mine hit.
pub fn reveal_cell(game: &mut MinesweeperGame, row: usize, col: usize) -> bool {
    let cell = &game.grid[row][col];

    // Can't reveal flagged or already revealed cells
    if cell.flagged || cell.revealed {
        return true;
    }

    // Hit a mine - game over
    if cell.has_mine {
        game.grid[row][col].revealed = true;
        game.game_result = Some(MinesweeperResult::Loss);
        reveal_all_mines(game);
        return false;
    }

    // Reveal this cell
    game.grid[row][col].revealed = true;

    // If zero adjacent mines, flood-fill reveal neighbors
    if game.grid[row][col].adjacent_mines == 0 {
        flood_fill_reveal(game, row, col);
    }

    // Check win condition
    check_win_condition(game);

    true
}

/// Flood-fill reveal all connected zero cells and their borders.
fn flood_fill_reveal(game: &mut MinesweeperGame, start_row: usize, start_col: usize) {
    let mut stack = vec![(start_row, start_col)];

    while let Some((row, col)) = stack.pop() {
        for (nr, nc) in get_neighbors(row, col, game.height, game.width) {
            let cell = &game.grid[nr][nc];
            if cell.revealed || cell.flagged || cell.has_mine {
                continue;
            }

            game.grid[nr][nc].revealed = true;

            // If this neighbor is also zero, add to stack for further exploration
            if game.grid[nr][nc].adjacent_mines == 0 {
                stack.push((nr, nc));
            }
        }
    }
}

/// Reveal all mines (called on game loss).
fn reveal_all_mines(game: &mut MinesweeperGame) {
    for row in 0..game.height {
        for col in 0..game.width {
            if game.grid[row][col].has_mine {
                game.grid[row][col].revealed = true;
            }
        }
    }
}

/// Toggle flag on a cell.
pub fn toggle_flag(game: &mut MinesweeperGame, row: usize, col: usize) {
    let cell = &game.grid[row][col];

    // Can't flag revealed cells
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

/// Check if player has won (all non-mine cells revealed).
fn check_win_condition(game: &mut MinesweeperGame) {
    let unrevealed_count: usize = game.grid.iter()
        .flat_map(|row| row.iter())
        .filter(|cell| !cell.revealed)
        .count();

    if unrevealed_count == game.total_mines as usize {
        game.game_result = Some(MinesweeperResult::Win);
    }
}

/// Handle first click: place mines then reveal.
pub fn handle_first_click<R: Rng>(game: &mut MinesweeperGame, row: usize, col: usize, rng: &mut R) {
    place_mines(game, row, col, rng);
    game.first_click_done = true;
    reveal_cell(game, row, col);
}
```

**Step 2: Add tests for reveal and flag**

Add to the tests module in `src/minesweeper_logic.rs`:

```rust
    #[test]
    fn test_reveal_safe_cell() {
        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);
        let mut rng = StdRng::seed_from_u64(42);
        place_mines(&mut game, 4, 4, &mut rng);

        // First click area should be safe
        let result = reveal_cell(&mut game, 4, 4);
        assert!(result);
        assert!(game.grid[4][4].revealed);
        assert!(game.game_result.is_none() || game.game_result == Some(MinesweeperResult::Win));
    }

    #[test]
    fn test_reveal_mine() {
        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);
        game.grid[0][0].has_mine = true;
        game.first_click_done = true;

        let result = reveal_cell(&mut game, 0, 0);
        assert!(!result);
        assert_eq!(game.game_result, Some(MinesweeperResult::Loss));
    }

    #[test]
    fn test_toggle_flag() {
        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);

        toggle_flag(&mut game, 0, 0);
        assert!(game.grid[0][0].flagged);
        assert_eq!(game.flags_placed, 1);

        toggle_flag(&mut game, 0, 0);
        assert!(!game.grid[0][0].flagged);
        assert_eq!(game.flags_placed, 0);
    }

    #[test]
    fn test_cannot_reveal_flagged() {
        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);
        game.first_click_done = true;

        toggle_flag(&mut game, 0, 0);
        reveal_cell(&mut game, 0, 0);

        assert!(!game.grid[0][0].revealed);
    }

    #[test]
    fn test_win_condition() {
        let mut game = MinesweeperGame::new(MinesweeperDifficulty::Novice);
        game.first_click_done = true;
        game.total_mines = 1;
        game.grid[0][0].has_mine = true;
        calculate_adjacent_counts(&mut game);

        // Reveal all cells except the mine
        for row in 0..game.height {
            for col in 0..game.width {
                if !game.grid[row][col].has_mine {
                    game.grid[row][col].revealed = true;
                }
            }
        }

        check_win_condition(&mut game);
        assert_eq!(game.game_result, Some(MinesweeperResult::Win));
    }
```

**Step 3: Run tests**

Run: `cargo test minesweeper`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/minesweeper_logic.rs
git commit -m "feat(minesweeper): add reveal and flag logic

- reveal_cell() with flood-fill for zero cells
- toggle_flag() for marking suspected mines
- Win condition: all non-mine cells revealed
- handle_first_click() places mines then reveals"
```

---

## Task 4: Integrate with Challenge Menu

**Files:**
- Modify: `src/challenge_menu.rs`

**Step 1: Add ChallengeType variant**

In `src/challenge_menu.rs`, add to the `ChallengeType` enum:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ChallengeType {
    Chess,
    Morris,
    Gomoku,
    Minesweeper,
}
```

**Step 2: Add to challenge table**

Update `CHALLENGE_TABLE`:

```rust
const CHALLENGE_TABLE: &[ChallengeWeight] = &[
    ChallengeWeight {
        challenge_type: ChallengeType::Chess,
        weight: 25,
    },
    ChallengeWeight {
        challenge_type: ChallengeType::Morris,
        weight: 25,
    },
    ChallengeWeight {
        challenge_type: ChallengeType::Gomoku,
        weight: 25,
    },
    ChallengeWeight {
        challenge_type: ChallengeType::Minesweeper,
        weight: 25,
    },
];
```

**Step 3: Add DifficultyInfo implementation**

Add the import and implementation:

```rust
use crate::minesweeper::MinesweeperDifficulty;

impl DifficultyInfo for MinesweeperDifficulty {
    fn name(&self) -> &'static str {
        MinesweeperDifficulty::name(self)
    }

    fn reward(&self) -> ChallengeReward {
        match self {
            MinesweeperDifficulty::Novice => ChallengeReward {
                xp_percent: 50,
                ..Default::default()
            },
            MinesweeperDifficulty::Apprentice => ChallengeReward {
                xp_percent: 75,
                ..Default::default()
            },
            MinesweeperDifficulty::Journeyman => ChallengeReward {
                xp_percent: 100,
                ..Default::default()
            },
            MinesweeperDifficulty::Master => ChallengeReward {
                prestige_ranks: 1,
                xp_percent: 200,
                ..Default::default()
            },
        }
    }

    fn extra_info(&self) -> Option<String> {
        let (h, w) = self.grid_size();
        Some(format!("{}x{}, {} traps", w, h, self.mine_count()))
    }
}
```

**Step 4: Add create_challenge case**

In the `create_challenge` function, add:

```rust
ChallengeType::Minesweeper => PendingChallenge {
    challenge_type: ChallengeType::Minesweeper,
    title: "Minesweeper: Trap Detection".to_string(),
    icon: "\u{26A0}",
    description: "A weathered scout beckons you toward a ruined corridor. \
        'The floor's rigged with pressure plates,' she warns, pulling out a \
        worn map. 'One wrong step and...' She makes an explosive gesture. \
        'Help me chart the safe path. Probe carefully—the numbers tell you \
        how many traps lurk nearby.'".to_string(),
},
```

**Step 5: Run tests and build**

Run: `cargo build`
Expected: Compiles successfully

**Step 6: Commit**

```bash
git add src/challenge_menu.rs
git commit -m "feat(minesweeper): integrate with challenge menu

- Add ChallengeType::Minesweeper variant
- Implement DifficultyInfo for MinesweeperDifficulty
- Add to challenge discovery table with 25% weight
- Add thematic 'Trap Detection' description"
```

---

## Task 5: Add to Game State

**Files:**
- Modify: `src/game_state.rs`
- Modify: `src/character_manager.rs`
- Modify: `src/save_manager.rs`

**Step 1: Add active_minesweeper field to GameState**

In `src/game_state.rs`, add the import and field:

```rust
use crate::minesweeper::MinesweeperGame;

// In GameState struct:
pub active_minesweeper: Option<MinesweeperGame>,
```

**Step 2: Initialize in GameState::new()**

In the `GameState::new()` function, add:

```rust
active_minesweeper: None,
```

**Step 3: Add to character_manager.rs**

In `src/character_manager.rs`, in the `From<SaveData>` impl for GameState, add:

```rust
active_minesweeper: None,
```

**Step 4: Add to save_manager.rs**

In `src/save_manager.rs`, in the legacy load function (if it creates GameState), ensure:

```rust
active_minesweeper: None,
```

**Step 5: Build to verify**

Run: `cargo build`
Expected: Compiles successfully

**Step 6: Commit**

```bash
git add src/game_state.rs src/character_manager.rs src/save_manager.rs
git commit -m "feat(minesweeper): add active_minesweeper to GameState

- New optional field for active minesweeper game
- Initialize to None in new() and load functions"
```

---

## Task 6: Create Minesweeper UI Scene

**Files:**
- Create: `src/ui/minesweeper_scene.rs`
- Modify: `src/ui/mod.rs`

**Step 1: Create the minesweeper scene**

Create `src/ui/minesweeper_scene.rs`:

```rust
//! Minesweeper (Trap Detection) game UI rendering.

use crate::minesweeper::{MinesweeperGame, MinesweeperResult};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Render the minesweeper game.
pub fn render_minesweeper(frame: &mut Frame, area: Rect, game: &MinesweeperGame) {
    frame.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(24)])
        .split(area);

    render_grid(frame, chunks[0], game);
    render_info_panel(frame, chunks[1], game);

    if game.game_result.is_some() {
        render_game_over_overlay(frame, area, game);
    }
}

fn render_grid(frame: &mut Frame, area: Rect, game: &MinesweeperGame) {
    let block = Block::default()
        .title(" Trap Detection ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Calculate cell size (2 chars wide for better visibility)
    let cell_width = 2u16;
    let cell_height = 1u16;

    // Center the grid
    let grid_width = game.width as u16 * cell_width;
    let grid_height = game.height as u16 * cell_height;
    let start_x = inner.x + (inner.width.saturating_sub(grid_width)) / 2;
    let start_y = inner.y + (inner.height.saturating_sub(grid_height)) / 2;

    for row in 0..game.height {
        for col in 0..game.width {
            let cell = &game.grid[row][col];
            let is_cursor = game.cursor == (row, col);

            let (ch, fg_color) = get_cell_display(cell, game.game_result.is_some());

            let style = if is_cursor {
                Style::default().fg(fg_color).bg(Color::DarkGray)
            } else {
                Style::default().fg(fg_color)
            };

            let x = start_x + col as u16 * cell_width;
            let y = start_y + row as u16 * cell_height;

            if x < inner.x + inner.width && y < inner.y + inner.height {
                let cell_area = Rect {
                    x,
                    y,
                    width: cell_width.min(inner.x + inner.width - x),
                    height: 1,
                };
                let text = Paragraph::new(ch).style(style);
                frame.render_widget(text, cell_area);
            }
        }
    }
}

fn get_cell_display(cell: &crate::minesweeper::Cell, game_over: bool) -> (&'static str, Color) {
    if cell.flagged && !cell.revealed {
        return ("\u{2691} ", Color::Red); // Flag
    }

    if !cell.revealed {
        return ("\u{2591}\u{2591}", Color::Gray); // Unrevealed
    }

    if cell.has_mine {
        return ("* ", Color::Red); // Mine
    }

    match cell.adjacent_mines {
        0 => ("\u{00B7} ", Color::DarkGray), // Middle dot
        1 => ("1 ", Color::Blue),
        2 => ("2 ", Color::Green),
        3 => ("3 ", Color::Red),
        4 => ("4 ", Color::Magenta),
        5 => ("5 ", Color::Yellow),
        6 => ("6 ", Color::Cyan),
        7 => ("7 ", Color::Gray),
        8 => ("8 ", Color::White),
        _ => ("? ", Color::White),
    }
}

fn render_info_panel(frame: &mut Frame, area: Rect, game: &MinesweeperGame) {
    let block = Block::default()
        .title(" Info ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = Vec::new();

    // Title
    lines.push(Line::from(Span::styled(
        "Trap Detection",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    // Difficulty
    lines.push(Line::from(vec![
        Span::styled("Difficulty: ", Style::default().fg(Color::DarkGray)),
        Span::styled(game.difficulty.name(), Style::default().fg(Color::Cyan)),
    ]));

    // Grid size
    let (h, w) = game.difficulty.grid_size();
    lines.push(Line::from(vec![
        Span::styled("Grid: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{}x{}", w, h), Style::default().fg(Color::White)),
    ]));

    // Mines
    lines.push(Line::from(vec![
        Span::styled("Traps: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", game.total_mines),
            Style::default().fg(Color::White),
        ),
    ]));
    lines.push(Line::from(""));

    // Remaining (mines - flags)
    let remaining = game.mines_remaining();
    let remaining_color = if remaining < 0 {
        Color::Red
    } else {
        Color::Yellow
    };
    lines.push(Line::from(vec![
        Span::styled("Remaining: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{}", remaining), Style::default().fg(remaining_color)),
        Span::styled(" \u{2691}", Style::default().fg(Color::Red)),
    ]));
    lines.push(Line::from(""));

    // Status
    let status = if game.forfeit_pending {
        Span::styled("Forfeit game?", Style::default().fg(Color::LightRed))
    } else if !game.first_click_done {
        Span::styled("Click to begin", Style::default().fg(Color::Green))
    } else {
        Span::styled("Detecting...", Style::default().fg(Color::Green))
    };
    lines.push(Line::from(status));
    lines.push(Line::from(""));

    // Controls
    if game.forfeit_pending {
        lines.push(Line::from(Span::styled(
            "[Esc] Confirm forfeit",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            "[Any] Cancel",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "[Arrows] Move",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            "[Enter] Reveal",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            "[F] Flag",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            "[Esc] Forfeit",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let text = Paragraph::new(lines);
    frame.render_widget(text, inner);
}

fn render_game_over_overlay(frame: &mut Frame, area: Rect, game: &MinesweeperGame) {
    use crate::challenge_menu::DifficultyInfo;

    let result = game.game_result.as_ref().unwrap();
    let (title, color) = match result {
        MinesweeperResult::Win => ("Area Secured!", Color::Green),
        MinesweeperResult::Loss => ("Trap Triggered!", Color::Red),
    };

    let reward_text = match result {
        MinesweeperResult::Win => game.difficulty.reward().description().replace("Win: ", ""),
        MinesweeperResult::Loss => "No reward".to_string(),
    };

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            title,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(reward_text, Style::default().fg(Color::Yellow))),
        Line::from(""),
        Line::from(Span::styled(
            "[Any key to continue]",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let width = 30u16;
    let height = 8u16;
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;

    let overlay_area = Rect {
        x,
        y,
        width,
        height,
    };

    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color));

    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    let text = Paragraph::new(lines).alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(text, inner);
}
```

**Step 2: Add to ui/mod.rs**

In `src/ui/mod.rs`, add:

```rust
pub mod minesweeper_scene;
```

**Step 3: Build to verify**

Run: `cargo build`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add src/ui/minesweeper_scene.rs src/ui/mod.rs
git commit -m "feat(minesweeper): add game UI scene

- Grid rendering with colored numbers
- Info panel with difficulty, mine count, remaining
- Cursor highlighting and forfeit state
- Game over overlay with win/loss display"
```

---

## Task 7: Add to Challenge Menu Scene

**Files:**
- Modify: `src/ui/challenge_menu_scene.rs`

**Step 1: Add MinesweeperDifficulty to imports and render**

In `src/ui/challenge_menu_scene.rs`, add import:

```rust
use crate::minesweeper::MinesweeperDifficulty;
```

**Step 2: Add case in render_detail_view**

In the `render_detail_view` function, add the Minesweeper case:

```rust
ChallengeType::Minesweeper => {
    render_difficulty_selector(
        frame,
        chunks[2],
        &MinesweeperDifficulty::ALL,
        menu.selected_difficulty,
    );
}
```

**Step 3: Build to verify**

Run: `cargo build`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add src/ui/challenge_menu_scene.rs
git commit -m "feat(minesweeper): add difficulty selector to challenge menu"
```

---

## Task 8: Wire Up Input Handling in Main

**Files:**
- Modify: `src/main.rs`

**Step 1: Add imports**

Add to imports in `src/main.rs`:

```rust
use quest::minesweeper::{MinesweeperDifficulty, MinesweeperGame, MinesweeperResult};
use quest::minesweeper_logic::{handle_first_click, reveal_cell, toggle_flag};
```

**Step 2: Add minesweeper input handling block**

Find the section with other active game handlers (gomoku, chess, morris) and add before them:

```rust
// Handle active minesweeper game input
if let Some(ref mut minesweeper_game) = state.active_minesweeper {
    if minesweeper_game.game_result.is_some() {
        // Any key dismisses result
        let result = minesweeper_game.game_result.unwrap();
        if result == MinesweeperResult::Win {
            use crate::challenge_menu::DifficultyInfo;
            let reward = minesweeper_game.difficulty.reward();
            if reward.xp_percent > 0 {
                let xp_gain = (state.xp_for_next_level() * reward.xp_percent / 100) as u64;
                state.gain_xp(xp_gain);
            }
            if reward.prestige_ranks > 0 {
                state.prestige_rank += reward.prestige_ranks;
            }
        }
        state.active_minesweeper = None;
        continue;
    }

    // Handle forfeit confirmation
    if minesweeper_game.forfeit_pending {
        match key_event.code {
            KeyCode::Esc => {
                minesweeper_game.game_result = Some(MinesweeperResult::Loss);
            }
            _ => {
                minesweeper_game.forfeit_pending = false;
            }
        }
        continue;
    }

    // Normal game input
    match key_event.code {
        KeyCode::Up => minesweeper_game.move_cursor(-1, 0),
        KeyCode::Down => minesweeper_game.move_cursor(1, 0),
        KeyCode::Left => minesweeper_game.move_cursor(0, -1),
        KeyCode::Right => minesweeper_game.move_cursor(0, 1),
        KeyCode::Enter => {
            let (row, col) = minesweeper_game.cursor;
            if !minesweeper_game.first_click_done {
                let mut rng = rand::thread_rng();
                handle_first_click(minesweeper_game, row, col, &mut rng);
            } else {
                reveal_cell(minesweeper_game, row, col);
            }
        }
        KeyCode::Char('f') | KeyCode::Char('F') => {
            let (row, col) = minesweeper_game.cursor;
            toggle_flag(minesweeper_game, row, col);
        }
        KeyCode::Esc => {
            minesweeper_game.forfeit_pending = true;
        }
        _ => {}
    }
    continue;
}
```

**Step 3: Add game start from challenge menu**

Find where other challenges are started (ChallengeType::Chess, etc.) and add:

```rust
ChallengeType::Minesweeper => {
    let difficulty = MinesweeperDifficulty::from_index(
        state.challenge_menu.selected_difficulty,
    );
    state.active_minesweeper = Some(MinesweeperGame::new(difficulty));
}
```

**Step 4: Add rendering call**

Find the rendering section and add before other game checks:

```rust
if let Some(ref game) = state.active_minesweeper {
    ui::minesweeper_scene::render_minesweeper(frame, area, game);
    return;
}
```

**Step 5: Build to verify**

Run: `cargo build`
Expected: Compiles successfully

**Step 6: Commit**

```bash
git add src/main.rs
git commit -m "feat(minesweeper): wire up input handling and rendering

- Arrow keys move cursor, Enter reveals, F flags
- First click triggers mine placement
- Forfeit with double-Esc
- Rewards granted on win"
```

---

## Task 9: Add Debug Menu Trigger

**Files:**
- Modify: `src/debug_menu.rs`

**Step 1: Add to DEBUG_OPTIONS**

In `src/debug_menu.rs`, add to the `DEBUG_OPTIONS` array:

```rust
pub const DEBUG_OPTIONS: &[&str] = &[
    "Trigger Dungeon",
    "Trigger Fishing",
    "Trigger Chess Challenge",
    "Trigger Morris Challenge",
    "Trigger Gomoku Challenge",
    "Trigger Minesweeper Challenge",
];
```

**Step 2: Add match arm in trigger_selected**

In the `trigger_selected` method, add:

```rust
5 => trigger_minesweeper_challenge(state),
```

**Step 3: Add trigger function**

Add the trigger function:

```rust
fn trigger_minesweeper_challenge(state: &mut GameState) -> &'static str {
    if state.challenge_menu.has_challenge(&ChallengeType::Minesweeper) {
        return "Minesweeper challenge already pending!";
    }
    state
        .challenge_menu
        .add_challenge(create_challenge(&ChallengeType::Minesweeper));
    "Minesweeper challenge added!"
}
```

**Step 4: Build and test**

Run: `cargo build`
Expected: Compiles successfully

**Step 5: Commit**

```bash
git add src/debug_menu.rs
git commit -m "feat(minesweeper): add debug menu trigger"
```

---

## Task 10: Final Testing and Cleanup

**Step 1: Run all tests**

Run: `cargo test`
Expected: All tests pass

**Step 2: Run clippy**

Run: `cargo clippy --all-targets -- -D warnings`
Expected: No warnings

**Step 3: Run fmt**

Run: `cargo fmt`

**Step 4: Build release**

Run: `cargo build --release`
Expected: Compiles successfully

**Step 5: Manual testing**

Run: `cargo run -- --debug`
- Press backtick to open debug menu
- Select "Trigger Minesweeper Challenge"
- Press Tab to open challenge menu
- Select the minesweeper challenge
- Test all difficulty levels work
- Test reveal, flag, win, and loss scenarios

**Step 6: Final commit if needed**

```bash
git add -A
git commit -m "chore(minesweeper): final cleanup and formatting"
```

---

## Summary

| Task | Description |
|------|-------------|
| 1 | Core data structures (Cell, Difficulty, Game) |
| 2 | Mine placement and adjacent count logic |
| 3 | Reveal and flag logic with win/loss conditions |
| 4 | Challenge menu integration |
| 5 | Game state integration |
| 6 | UI scene rendering |
| 7 | Challenge menu difficulty selector |
| 8 | Main input handling and rendering |
| 9 | Debug menu trigger |
| 10 | Final testing and cleanup |
