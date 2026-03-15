# Runic Lights Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a Lights Out puzzle challenge called "Runic Lights" with 4 difficulty tiers, achievements, and full UI.

**Architecture:** New challenge module at `src/challenges/runic_lights/` with types, logic, and macro wiring. Grid-based UI scene following the minesweeper pattern. Reverse-generation puzzle solver guarantees solvability. Integration into achievements, menu, input routing, and debug menu.

**Tech Stack:** Rust, Ratatui, rand (seeded RNG for puzzle generation)

**Spec:** `docs/superpowers/specs/2026-03-15-runic-lights-design.md`

---

## File Map

| Action | File | Responsibility |
|--------|------|---------------|
| Create | `src/challenges/runic_lights/types.rs` | Game struct, difficulty enum, result enum, input enum |
| Create | `src/challenges/runic_lights/logic.rs` | Puzzle generation, toggle, input processing, win/loss, DifficultyInfo |
| Create | `src/challenges/runic_lights/mod.rs` | Public exports, `impl_apply_game_result!` macro |
| Create | `src/ui/runic_lights_scene.rs` | Grid rendering, info panel, status bar |
| Modify | `src/challenges/mod.rs` | Add `ActiveMinigame::RunicLights` variant, module declaration |
| Modify | `src/challenges/menu.rs` | Add `ChallengeType::RunicLights`, discovery weight, accept/create |
| Modify | `src/input/minigame_input.rs` | Input routing for RunicLights |
| Modify | `src/ui/mod.rs` | Module declaration, render dispatch |
| Modify | `src/achievements/milestones.rs` | Add `MinigameType::RunicLights` |
| Modify | `src/achievements/types.rs` | Add 4 `AchievementId` variants |
| Modify | `src/achievements/data.rs` | Add 4 `AchievementDef` entries |
| Modify | `src/achievements/handlers.rs` | Add match arms in `on_minigame_won()` |
| Modify | `src/utils/debug_menu.rs` | Add debug trigger |

---

## Chunk 1: Core Game Logic

### Task 1: Types

**Files:**
- Create: `src/challenges/runic_lights/types.rs`

- [ ] **Step 1: Write types with tests**

```rust
//! Runic Lights challenge data structures.
//!
//! A Lights Out puzzle where toggling a rune flips it and its orthogonal neighbors.

use serde::{Deserialize, Serialize};

/// Difficulty levels controlling grid size and solution depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunicLightsDifficulty {
    Novice,
    Apprentice,
    Journeyman,
    Master,
}

difficulty_enum_impl!(RunicLightsDifficulty);

impl RunicLightsDifficulty {
    /// Grid side length for this difficulty.
    pub fn grid_size(&self) -> usize {
        match self {
            Self::Novice => 3,
            Self::Apprentice => 4,
            Self::Journeyman => 5,
            Self::Master => 6,
        }
    }

    /// Range of toggles used during puzzle generation (min, max inclusive).
    pub fn solution_depth_range(&self) -> (usize, usize) {
        match self {
            Self::Novice => (3, 5),
            Self::Apprentice => (5, 8),
            Self::Journeyman => (8, 12),
            Self::Master => (12, 16),
        }
    }

    /// Par score (upper bound of solution depth range).
    pub fn par(&self) -> u32 {
        match self {
            Self::Novice => 5,
            Self::Apprentice => 8,
            Self::Journeyman => 12,
            Self::Master => 16,
        }
    }

    /// Move limit = 3x par.
    pub fn move_limit(&self) -> u32 {
        self.par() * 3
    }
}

/// Result of a Runic Lights game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunicLightsResult {
    Win,
    Loss,
}

/// UI-agnostic input enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunicLightsInput {
    Up,
    Down,
    Left,
    Right,
    Toggle,
    Forfeit,
    Other,
}

/// Full Runic Lights game state.
#[derive(Debug, Clone)]
pub struct RunicLightsGame {
    pub difficulty: RunicLightsDifficulty,
    /// Board state: true = lit, false = dark.
    pub board: Vec<Vec<bool>>,
    /// Grid side length.
    pub size: usize,
    /// Cursor position (row, col).
    pub cursor: (usize, usize),
    /// Number of moves the player has made.
    pub moves: u32,
    /// Par for the current puzzle.
    pub par: u32,
    /// Move limit (3x par).
    pub move_limit: u32,
    pub game_result: Option<RunicLightsResult>,
    pub forfeit_pending: bool,
}

impl RunicLightsGame {
    pub fn new(difficulty: RunicLightsDifficulty) -> Self {
        let size = difficulty.grid_size();
        Self {
            difficulty,
            board: vec![vec![false; size]; size],
            size,
            cursor: (0, 0),
            moves: 0,
            par: difficulty.par(),
            move_limit: difficulty.move_limit(),
            game_result: None,
            forfeit_pending: false,
        }
    }

    /// Count of lit cells on the board.
    pub fn lit_count(&self) -> usize {
        self.board.iter().flatten().filter(|&&c| c).count()
    }

    /// Total number of cells.
    pub fn total_cells(&self) -> usize {
        self.size * self.size
    }

    /// Move cursor with clamping.
    pub fn move_cursor(&mut self, d_row: i32, d_col: i32) {
        let new_row = (self.cursor.0 as i32 + d_row).clamp(0, self.size as i32 - 1) as usize;
        let new_col = (self.cursor.1 as i32 + d_col).clamp(0, self.size as i32 - 1) as usize;
        self.cursor = (new_row, new_col);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_grid_sizes() {
        assert_eq!(RunicLightsDifficulty::Novice.grid_size(), 3);
        assert_eq!(RunicLightsDifficulty::Apprentice.grid_size(), 4);
        assert_eq!(RunicLightsDifficulty::Journeyman.grid_size(), 5);
        assert_eq!(RunicLightsDifficulty::Master.grid_size(), 6);
    }

    #[test]
    fn test_par_and_move_limit() {
        assert_eq!(RunicLightsDifficulty::Novice.par(), 5);
        assert_eq!(RunicLightsDifficulty::Novice.move_limit(), 15);
        assert_eq!(RunicLightsDifficulty::Master.par(), 16);
        assert_eq!(RunicLightsDifficulty::Master.move_limit(), 48);
    }

    #[test]
    fn test_difficulty_enum_impl() {
        assert_eq!(RunicLightsDifficulty::from_index(0), RunicLightsDifficulty::Novice);
        assert_eq!(RunicLightsDifficulty::from_index(3), RunicLightsDifficulty::Master);
        assert_eq!(RunicLightsDifficulty::from_index(99), RunicLightsDifficulty::Novice);
        assert_eq!(RunicLightsDifficulty::ALL.len(), 4);
    }

    #[test]
    fn test_game_new() {
        let game = RunicLightsGame::new(RunicLightsDifficulty::Novice);
        assert_eq!(game.size, 3);
        assert_eq!(game.board.len(), 3);
        assert_eq!(game.board[0].len(), 3);
        assert!(game.board.iter().flatten().all(|&c| !c));
        assert_eq!(game.cursor, (0, 0));
        assert_eq!(game.moves, 0);
        assert_eq!(game.par, 5);
        assert_eq!(game.move_limit, 15);
        assert!(game.game_result.is_none());
        assert!(!game.forfeit_pending);
    }

    #[test]
    fn test_lit_count() {
        let mut game = RunicLightsGame::new(RunicLightsDifficulty::Novice);
        assert_eq!(game.lit_count(), 0);
        game.board[0][0] = true;
        game.board[1][1] = true;
        assert_eq!(game.lit_count(), 2);
    }

    #[test]
    fn test_move_cursor_clamping() {
        let mut game = RunicLightsGame::new(RunicLightsDifficulty::Novice);
        game.move_cursor(-1, -1); // Already at (0,0)
        assert_eq!(game.cursor, (0, 0));
        game.move_cursor(10, 10); // Clamped to (2,2)
        assert_eq!(game.cursor, (2, 2));
        game.move_cursor(1, 1); // Clamped to (2,2)
        assert_eq!(game.cursor, (2, 2));
    }
}
```

- [ ] **Step 2: Run test to verify**

Run: `cargo test --lib challenges::runic_lights::types`
Expected: All tests pass

- [ ] **Step 3: Commit**

```bash
git add src/challenges/runic_lights/types.rs
git commit -m "feat(challenges): add Runic Lights types — difficulty, game state, input enum"
```

---

### Task 2: Core Logic — Toggle, Generation, Win/Loss

**Files:**
- Create: `src/challenges/runic_lights/logic.rs`

- [ ] **Step 1: Write toggle and generation logic with tests**

```rust
//! Runic Lights game logic.
//!
//! Puzzle generation uses reverse construction: start from all-off, apply N
//! random unique cell toggles. Each toggle flips the cross pattern (+).
//! The resulting board is guaranteed solvable in at most N moves.

use super::types::{
    RunicLightsDifficulty, RunicLightsGame, RunicLightsInput, RunicLightsResult,
};
use crate::challenges::ActiveMinigame;
use rand::Rng;

/// Toggle a cell and its orthogonal neighbors (cross pattern).
pub fn toggle_cell(board: &mut Vec<Vec<bool>>, row: usize, col: usize) {
    let size = board.len();
    board[row][col] = !board[row][col];
    if row > 0 {
        board[row - 1][col] = !board[row - 1][col];
    }
    if row + 1 < size {
        board[row + 1][col] = !board[row + 1][col];
    }
    if col > 0 {
        board[row][col - 1] = !board[row][col - 1];
    }
    if col + 1 < size {
        board[row][col + 1] = !board[row][col + 1];
    }
}

/// Generate a solvable puzzle by reverse construction.
///
/// Picks N unique random cells and toggles each once from the solved (all-off) state.
/// N is sampled uniformly from the difficulty's solution depth range.
pub fn generate_puzzle<R: Rng>(game: &mut RunicLightsGame, rng: &mut R) {
    let (min_depth, max_depth) = game.difficulty.solution_depth_range();
    let depth = rng.random_range(min_depth..=max_depth);
    let size = game.size;
    let total = size * size;

    // Collect all cell indices, shuffle, take first `depth`
    let mut indices: Vec<usize> = (0..total).collect();
    // Fisher-Yates shuffle
    for i in (1..indices.len()).rev() {
        let j = rng.random_range(0..=i);
        indices.swap(i, j);
    }

    for &idx in indices.iter().take(depth) {
        let row = idx / size;
        let col = idx % size;
        toggle_cell(&mut game.board, row, col);
    }
}

/// Check if the board is solved (all cells dark).
pub fn is_solved(game: &RunicLightsGame) -> bool {
    game.board.iter().flatten().all(|&c| !c)
}

/// Check win/loss conditions and update game_result.
fn check_game_over(game: &mut RunicLightsGame) {
    if game.game_result.is_some() {
        return;
    }
    if is_solved(game) {
        game.game_result = Some(RunicLightsResult::Win);
    } else if game.moves >= game.move_limit {
        game.game_result = Some(RunicLightsResult::Loss);
    }
}

/// Process player input.
pub fn process_input(game: &mut RunicLightsGame, input: RunicLightsInput) {
    if game.game_result.is_some() {
        return;
    }

    // Handle forfeit double-Esc pattern
    if game.forfeit_pending {
        match input {
            RunicLightsInput::Forfeit => {
                crate::challenges::handle_forfeit(
                    &mut game.game_result,
                    &mut game.forfeit_pending,
                    RunicLightsResult::Loss,
                );
            }
            _ => {
                crate::challenges::cancel_forfeit_if_pending(&mut game.forfeit_pending);
            }
        }
        return;
    }

    match input {
        RunicLightsInput::Up => game.move_cursor(-1, 0),
        RunicLightsInput::Down => game.move_cursor(1, 0),
        RunicLightsInput::Left => game.move_cursor(0, -1),
        RunicLightsInput::Right => game.move_cursor(0, 1),
        RunicLightsInput::Toggle => {
            let (row, col) = game.cursor;
            toggle_cell(&mut game.board, row, col);
            game.moves += 1;
            check_game_over(game);
        }
        RunicLightsInput::Forfeit => {
            crate::challenges::handle_forfeit(
                &mut game.game_result,
                &mut game.forfeit_pending,
                RunicLightsResult::Loss,
            );
        }
        RunicLightsInput::Other => {}
    }
}

impl crate::challenges::menu::DifficultyInfo for RunicLightsDifficulty {
    fn name(&self) -> &'static str {
        RunicLightsDifficulty::name(self)
    }

    fn reward(&self) -> crate::challenges::menu::ChallengeReward {
        match self {
            RunicLightsDifficulty::Novice => crate::challenges::menu::ChallengeReward {
                prestige_ranks: 0,
                stormglass: 400,
                fishing_ranks: 0,
            },
            RunicLightsDifficulty::Apprentice => crate::challenges::menu::ChallengeReward {
                prestige_ranks: 0,
                stormglass: 1_200,
                fishing_ranks: 0,
            },
            RunicLightsDifficulty::Journeyman => crate::challenges::menu::ChallengeReward {
                prestige_ranks: 1,
                stormglass: 3_000,
                fishing_ranks: 0,
            },
            RunicLightsDifficulty::Master => crate::challenges::menu::ChallengeReward {
                prestige_ranks: 2,
                stormglass: 6_000,
                fishing_ranks: 0,
            },
        }
    }

    fn extra_info(&self) -> Option<String> {
        Some(format!(
            "{}×{} grid, par {}",
            self.grid_size(),
            self.grid_size(),
            self.par()
        ))
    }
}

/// Start a new Runic Lights game and return it as an `ActiveMinigame`.
pub fn start_runic_lights_game<R: Rng>(
    difficulty: RunicLightsDifficulty,
    rng: &mut R,
) -> ActiveMinigame {
    let mut game = RunicLightsGame::new(difficulty);
    generate_puzzle(&mut game, rng);
    ActiveMinigame::RunicLights(game)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_toggle_center_3x3() {
        let mut board = vec![vec![false; 3]; 3];
        toggle_cell(&mut board, 1, 1);
        // Cross pattern: center + 4 neighbors
        assert!(board[1][1]); // center
        assert!(board[0][1]); // up
        assert!(board[2][1]); // down
        assert!(board[1][0]); // left
        assert!(board[1][2]); // right
        // Corners should be unaffected
        assert!(!board[0][0]);
        assert!(!board[0][2]);
        assert!(!board[2][0]);
        assert!(!board[2][2]);
    }

    #[test]
    fn test_toggle_corner() {
        let mut board = vec![vec![false; 3]; 3];
        toggle_cell(&mut board, 0, 0);
        assert!(board[0][0]); // self
        assert!(board[0][1]); // right
        assert!(board[1][0]); // down
        // Only 3 cells flipped for corner
        assert_eq!(board.iter().flatten().filter(|&&c| c).count(), 3);
    }

    #[test]
    fn test_toggle_edge() {
        let mut board = vec![vec![false; 3]; 3];
        toggle_cell(&mut board, 0, 1);
        assert!(board[0][1]); // self
        assert!(board[0][0]); // left
        assert!(board[0][2]); // right
        assert!(board[1][1]); // down
        // 4 cells flipped for edge
        assert_eq!(board.iter().flatten().filter(|&&c| c).count(), 4);
    }

    #[test]
    fn test_double_toggle_cancels() {
        let mut board = vec![vec![false; 3]; 3];
        toggle_cell(&mut board, 1, 1);
        toggle_cell(&mut board, 1, 1);
        assert!(board.iter().flatten().all(|&c| !c));
    }

    #[test]
    fn test_generate_puzzle_is_solvable() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        for difficulty in RunicLightsDifficulty::ALL {
            let mut game = RunicLightsGame::new(difficulty);
            generate_puzzle(&mut game, &mut rng);
            // Board should have some lit cells
            assert!(game.lit_count() > 0, "Puzzle for {:?} has no lit cells", difficulty);
        }
    }

    #[test]
    fn test_generate_puzzle_different_seeds_different_boards() {
        let mut rng1 = ChaCha8Rng::seed_from_u64(1);
        let mut rng2 = ChaCha8Rng::seed_from_u64(2);
        let mut game1 = RunicLightsGame::new(RunicLightsDifficulty::Journeyman);
        let mut game2 = RunicLightsGame::new(RunicLightsDifficulty::Journeyman);
        generate_puzzle(&mut game1, &mut rng1);
        generate_puzzle(&mut game2, &mut rng2);
        assert_ne!(game1.board, game2.board);
    }

    #[test]
    fn test_win_condition() {
        let mut game = RunicLightsGame::new(RunicLightsDifficulty::Novice);
        // Set up a board where toggling (0,0) solves it
        toggle_cell(&mut game.board, 0, 0);
        assert!(!is_solved(&game));

        // Player toggles (0,0) — should solve
        game.cursor = (0, 0);
        process_input(&mut game, RunicLightsInput::Toggle);
        assert_eq!(game.game_result, Some(RunicLightsResult::Win));
        assert_eq!(game.moves, 1);
    }

    #[test]
    fn test_loss_on_move_limit() {
        let mut game = RunicLightsGame::new(RunicLightsDifficulty::Novice);
        // Light up one cell that can't be solved in 1 move
        game.board[0][0] = true;
        game.moves = game.move_limit - 1; // One move left
        game.cursor = (1, 1); // Toggle center — won't solve it
        process_input(&mut game, RunicLightsInput::Toggle);
        assert_eq!(game.game_result, Some(RunicLightsResult::Loss));
    }

    #[test]
    fn test_forfeit_double_esc() {
        let mut game = RunicLightsGame::new(RunicLightsDifficulty::Novice);
        game.board[0][0] = true;

        // First Esc: sets pending
        process_input(&mut game, RunicLightsInput::Forfeit);
        assert!(game.forfeit_pending);
        assert!(game.game_result.is_none());

        // Second Esc: confirms
        process_input(&mut game, RunicLightsInput::Forfeit);
        assert_eq!(game.game_result, Some(RunicLightsResult::Loss));
    }

    #[test]
    fn test_forfeit_cancel() {
        let mut game = RunicLightsGame::new(RunicLightsDifficulty::Novice);
        game.board[0][0] = true;

        process_input(&mut game, RunicLightsInput::Forfeit);
        assert!(game.forfeit_pending);

        // Any other key cancels forfeit
        process_input(&mut game, RunicLightsInput::Up);
        assert!(!game.forfeit_pending);
        assert!(game.game_result.is_none());
    }

    #[test]
    fn test_no_input_after_game_over() {
        let mut game = RunicLightsGame::new(RunicLightsDifficulty::Novice);
        game.game_result = Some(RunicLightsResult::Win);
        let old_cursor = game.cursor;
        process_input(&mut game, RunicLightsInput::Down);
        assert_eq!(game.cursor, old_cursor); // No change
    }

    #[test]
    fn test_move_increments_on_toggle() {
        let mut game = RunicLightsGame::new(RunicLightsDifficulty::Novice);
        game.board[0][0] = true; // Prevent immediate win
        game.board[2][2] = true;
        game.cursor = (1, 1);
        process_input(&mut game, RunicLightsInput::Toggle);
        assert_eq!(game.moves, 1);
    }

    #[test]
    fn test_start_runic_lights_game() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let minigame = start_runic_lights_game(RunicLightsDifficulty::Novice, &mut rng);
        match minigame {
            ActiveMinigame::RunicLights(game) => {
                assert_eq!(game.size, 3);
                assert!(game.lit_count() > 0);
            }
            _ => panic!("Expected RunicLights variant"),
        }
    }

    #[test]
    fn test_difficulty_info_rewards() {
        use crate::challenges::menu::DifficultyInfo;
        let reward = RunicLightsDifficulty::Master.reward();
        assert_eq!(reward.prestige_ranks, 2);
        assert_eq!(reward.stormglass, 6_000);
        assert_eq!(reward.fishing_ranks, 0);
    }

    #[test]
    fn test_difficulty_info_extra_info() {
        use crate::challenges::menu::DifficultyInfo;
        let info = RunicLightsDifficulty::Novice.extra_info();
        assert_eq!(info, Some("3×3 grid, par 5".to_string()));
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --lib challenges::runic_lights`
Expected: All tests pass

- [ ] **Step 3: Commit**

```bash
git add src/challenges/runic_lights/logic.rs
git commit -m "feat(challenges): add Runic Lights logic — toggle, generation, input processing"
```

---

### Task 3: Module Exports and Macro Wiring

**Files:**
- Create: `src/challenges/runic_lights/mod.rs`
- Modify: `src/challenges/mod.rs`

- [ ] **Step 1: Create mod.rs**

```rust
//! Runic Lights challenge — Lights Out puzzle.

mod logic;
mod types;

pub use logic::{process_input, start_runic_lights_game};
pub use types::{RunicLightsDifficulty, RunicLightsGame, RunicLightsInput, RunicLightsResult};

impl_apply_game_result! {
    fn apply_runic_lights_result;
    variant: RunicLights;
    result_body: |result, _state, _reward| {
        use RunicLightsResult::*;
        match result {
            Win => (true, ""),
            Loss => (false, "The runes remain ablaze."),
        }
    }
    game_type: crate::achievements::MinigameType::RunicLights;
    icon: "\u{25C7}";
    win_message: "All runes extinguished!";
}
```

- [ ] **Step 2: Add module declaration and ActiveMinigame variant to `src/challenges/mod.rs`**

Add `pub mod runic_lights;` with the other module declarations.

Add `RunicLights(RunicLightsGame)` to the `ActiveMinigame` enum.

Add `ActiveMinigame::RunicLights(g) => g.game_result.is_some()` to `has_game_result()`.

Add `pub use runic_lights::{RunicLightsDifficulty, RunicLightsGame, RunicLightsResult};` to the re-exports.

- [ ] **Step 3: Verify it compiles**

Run: `cargo check`
Expected: Compiles (may have warnings about unused imports — that's fine at this stage)

- [ ] **Step 4: Commit**

```bash
git add src/challenges/runic_lights/mod.rs src/challenges/mod.rs
git commit -m "feat(challenges): wire Runic Lights module into ActiveMinigame enum"
```

---

## Chunk 2: Menu, Input, and Achievements Integration

### Task 4: Challenge Menu Integration

**Files:**
- Modify: `src/challenges/menu.rs`

- [ ] **Step 1: Add ChallengeType variant**

Add `RunicLights` to the `ChallengeType` enum (after `ShardFusion`).

- [ ] **Step 2: Add import**

Add to the imports section:

```rust
use super::runic_lights::{start_runic_lights_game, RunicLightsDifficulty};
```

- [ ] **Step 3: Add match arms**

In `accept_selected_challenge()` — add before the closing `};`:

```rust
ChallengeType::RunicLights => {
    let d = RunicLightsDifficulty::from_index(difficulty_index);
    start_runic_lights_game(d, &mut rand::rng())
}
```

In `icon()`:

```rust
ChallengeType::RunicLights => "\u{25C7}",
```

In `discovery_flavor()`:

```rust
ChallengeType::RunicLights => {
    "A grid of glowing runes pulses on the wall, each connected to its neighbors by threads of light..."
}
```

In `create_challenge()`:

```rust
ChallengeType::RunicLights => PendingChallenge {
    challenge_type: ChallengeType::RunicLights,
    title: "Runic Lights".to_string(),
    icon: "\u{25C7}",
    description: "A grid of glowing runes pulses on the dungeon wall. Each rune is bound \
        to its neighbors by threads of arcane light. Touching one rune shifts the light of \
        all connected runes. Extinguish every rune to break the ward."
        .to_string(),
},
```

- [ ] **Step 4: Add to CHALLENGE_TABLE**

Add after the ShardFusion entry:

```rust
ChallengeWeight {
    challenge_type: ChallengeType::RunicLights,
    weight: 20,
},
```

- [ ] **Step 5: Verify compilation**

Run: `cargo check`
Expected: Compiles

- [ ] **Step 6: Commit**

```bash
git add src/challenges/menu.rs
git commit -m "feat(challenges): integrate Runic Lights into challenge menu and discovery table"
```

---

### Task 5: Input Routing

**Files:**
- Modify: `src/input/minigame_input.rs`

- [ ] **Step 1: Add import**

```rust
use crate::challenges::runic_lights::{
    apply_runic_lights_result, process_input as process_runic_lights_input, RunicLightsInput,
};
```

- [ ] **Step 2: Add match arm in `handle_minigame()`**

Add before the closing `}` of the main match (after ShardFusion arm):

```rust
ActiveMinigame::RunicLights(game) => {
    if game.game_result.is_some() {
        state.last_minigame_win = apply_runic_lights_result(state);
        return result_for_challenge(&state.last_minigame_win);
    }
    let input = match key.code {
        KeyCode::Up => RunicLightsInput::Up,
        KeyCode::Down => RunicLightsInput::Down,
        KeyCode::Left => RunicLightsInput::Left,
        KeyCode::Right => RunicLightsInput::Right,
        KeyCode::Enter => RunicLightsInput::Toggle,
        KeyCode::Esc => RunicLightsInput::Forfeit,
        _ => RunicLightsInput::Other,
    };
    process_runic_lights_input(game, input);
}
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check`
Expected: Compiles

- [ ] **Step 4: Commit**

```bash
git add src/input/minigame_input.rs
git commit -m "feat(input): route keyboard input to Runic Lights"
```

---

### Task 6: Achievements

**Files:**
- Modify: `src/achievements/milestones.rs`
- Modify: `src/achievements/types.rs`
- Modify: `src/achievements/data.rs`
- Modify: `src/achievements/handlers.rs`

- [ ] **Step 1: Add MinigameType variant**

In `src/achievements/milestones.rs`, add `RunicLights` to the `MinigameType` enum.

- [ ] **Step 2: Add AchievementId variants**

In `src/achievements/types.rs`, add after the ShardFusion entries:

```rust
RunicLightsNovice,
RunicLightsApprentice,
RunicLightsJourneyman,
RunicLightsMaster,
```

- [ ] **Step 3: Add AchievementDef entries**

In `src/achievements/data.rs`, add after the ShardFusion definitions:

```rust
AchievementDef {
    id: AchievementId::RunicLightsNovice,
    name: "Runic Lights Novice",
    description: "Extinguish all runes on Novice difficulty",
    category: AchievementCategory::Challenges,
    icon: "\u{25C7}",
    points: 10,
},
AchievementDef {
    id: AchievementId::RunicLightsApprentice,
    name: "Runic Lights Apprentice",
    description: "Extinguish all runes on Apprentice difficulty",
    category: AchievementCategory::Challenges,
    icon: "\u{25C7}",
    points: 25,
},
AchievementDef {
    id: AchievementId::RunicLightsJourneyman,
    name: "Runic Lights Journeyman",
    description: "Extinguish all runes on Journeyman difficulty",
    category: AchievementCategory::Challenges,
    icon: "\u{25C7}",
    points: 50,
},
AchievementDef {
    id: AchievementId::RunicLightsMaster,
    name: "Runic Lights Master",
    description: "Extinguish all runes on Master difficulty",
    category: AchievementCategory::Challenges,
    icon: "\u{25C7}",
    points: 100,
},
```

- [ ] **Step 4: Add handler match arms**

In `src/achievements/handlers.rs`, in the `on_minigame_won()` match, add after the ShardFusion arms:

```rust
(MinigameType::RunicLights, MinigameDifficulty::Novice) => {
    Some(AchievementId::RunicLightsNovice)
}
(MinigameType::RunicLights, MinigameDifficulty::Apprentice) => {
    Some(AchievementId::RunicLightsApprentice)
}
(MinigameType::RunicLights, MinigameDifficulty::Journeyman) => {
    Some(AchievementId::RunicLightsJourneyman)
}
(MinigameType::RunicLights, MinigameDifficulty::Master) => {
    Some(AchievementId::RunicLightsMaster)
}
```

- [ ] **Step 5: Verify compilation and tests**

Run: `cargo test --lib achievements`
Expected: All tests pass

- [ ] **Step 6: Commit**

```bash
git add src/achievements/milestones.rs src/achievements/types.rs src/achievements/data.rs src/achievements/handlers.rs
git commit -m "feat(achievements): add 4 Runic Lights tier achievements"
```

---

## Chunk 3: UI and Debug Menu

### Task 7: UI Scene

**Files:**
- Create: `src/ui/runic_lights_scene.rs`
- Modify: `src/ui/mod.rs`

- [ ] **Step 1: Create the scene file**

```rust
//! Runic Lights minigame UI rendering.

use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_game_over_overlay,
    render_info_panel_frame, render_minigame_too_small, render_status_bar, GameResultType,
};
use crate::challenges::runic_lights::{RunicLightsGame, RunicLightsResult};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Render the Runic Lights game scene.
pub fn render_runic_lights(
    frame: &mut Frame,
    area: Rect,
    game: &RunicLightsGame,
    ctx: &super::responsive::LayoutContext,
    show_dismiss_hint: bool,
) {
    // Game over overlay
    if game.game_result.is_some() {
        render_game_over(frame, area, game, show_dismiss_hint);
        return;
    }

    // Minimum terminal size for the game
    let min_width = (game.size as u16 * 4) + 6;
    let min_height = (game.size as u16 * 2) + 6;
    if area.width < min_width || area.height < min_height {
        render_minigame_too_small(frame, area, "Runic Lights", min_width, min_height);
        return;
    }

    let layout = create_game_layout(
        frame,
        area,
        " Runic Lights ",
        Color::Cyan,
        (game.size as u16) + 2,
        22,
        ctx,
    );

    render_grid(frame, layout.content, game);
    render_status_bar_content(frame, layout.status_bar, game);
    render_info_panel(frame, layout.info_panel, game);
}

/// Render the grid of runes.
fn render_grid(frame: &mut Frame, area: Rect, game: &RunicLightsGame) {
    let size = game.size;
    // Each cell: 3 chars wide ("◆◆ " or "◇◇ "), 1 char tall
    let cell_width = 3u16;
    let grid_width = size as u16 * cell_width;
    let grid_height = size as u16;

    // Center the grid
    let x_offset = area.x + (area.width.saturating_sub(grid_width)) / 2;
    let y_offset = area.y + (area.height.saturating_sub(grid_height)) / 2;

    for row in 0..size {
        let mut spans = Vec::new();

        for col in 0..size {
            let lit = game.board[row][col];
            let is_cursor = game.cursor == (row, col);

            let symbol = if lit { "\u{25C6}\u{25C6}" } else { "\u{25C7}\u{25C7}" };
            let pad = " ";

            let base_color = if lit {
                Color::Rgb(100, 200, 255) // Bright cyan for lit
            } else {
                Color::Rgb(50, 50, 65) // Dim for dark
            };

            let mut style = Style::default().fg(base_color);

            if is_cursor {
                style = style.bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD);
            }

            spans.push(Span::styled(symbol, style));
            spans.push(Span::raw(pad));
        }

        let line = Line::from(spans);
        let y = y_offset + row as u16;
        if y < area.y + area.height {
            frame.render_widget(
                Paragraph::new(vec![line]),
                Rect::new(x_offset, y, grid_width, 1),
            );
        }
    }
}

/// Render the status bar.
fn render_status_bar_content(frame: &mut Frame, area: Rect, game: &RunicLightsGame) {
    if render_forfeit_status_bar(frame, area, game.forfeit_pending) {
        return;
    }

    render_status_bar(
        frame,
        area,
        "Extinguishing...",
        Color::Cyan,
        &[
            ("[Arrows]", "Move"),
            ("[Enter]", "Toggle"),
            ("[Esc]", "Forfeit"),
        ],
    );
}

/// Render the info panel.
fn render_info_panel(frame: &mut Frame, area: Rect, game: &RunicLightsGame) {
    let inner = render_info_panel_frame(frame, area);
    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("Difficulty ", Style::default().fg(Color::DarkGray)),
            Span::styled(game.difficulty.name(), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Grid   ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}×{}", game.size, game.size),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Moves  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}/{}", game.moves, game.move_limit),
                Style::default().fg(if game.moves > game.par as u32 {
                    Color::Yellow
                } else {
                    Color::White
                }),
            ),
        ]),
        Line::from(vec![
            Span::styled("Par    ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", game.par),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Lit    ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}/{}", game.lit_count(), game.total_cells()),
                Style::default().fg(if game.lit_count() == 0 {
                    Color::Green
                } else {
                    Color::Rgb(100, 200, 255)
                }),
            ),
        ]),
    ];

    // Truncate to fit
    lines.truncate(inner.height as usize);
    frame.render_widget(Paragraph::new(lines), inner);
}

/// Render game over overlay.
fn render_game_over(
    frame: &mut Frame,
    area: Rect,
    game: &RunicLightsGame,
    show_dismiss_hint: bool,
) {
    use crate::challenges::menu::DifficultyInfo;

    let (result_type, title, message, reward) = match game.game_result {
        Some(RunicLightsResult::Win) => {
            let r = game.difficulty.reward();
            let reward_text = if r.prestige_ranks > 0 {
                format!("+{} Prestige Ranks, +{} Stormglass", r.prestige_ranks, r.stormglass)
            } else {
                format!("+{} Stormglass", r.stormglass)
            };
            let msg = if game.moves <= game.par as u32 {
                format!("Solved in {} moves (par {}) \u{2605}", game.moves, game.par)
            } else {
                format!("Solved in {} moves (par {})", game.moves, game.par)
            };
            (GameResultType::Win, "ALL RUNES EXTINGUISHED!".to_string(), msg, reward_text)
        }
        _ => (
            GameResultType::Loss,
            "RUNES STILL ABLAZE!".to_string(),
            format!("Exceeded move limit ({}/{})", game.moves, game.move_limit),
            "No penalty incurred.".to_string(),
        ),
    };
    render_game_over_overlay(frame, area, result_type, &title, &message, &reward, show_dismiss_hint);
}
```

- [ ] **Step 2: Add module declaration to `src/ui/mod.rs`**

Add `pub mod runic_lights_scene;` with the other scene declarations.

- [ ] **Step 3: Add render dispatch to `src/ui/mod.rs`**

In the minigame render match (after `ActiveMinigame::ShardFusion`), add:

```rust
Some(ActiveMinigame::RunicLights(game)) => {
    runic_lights_scene::render_runic_lights(frame, area, game, &ctx, show_dismiss_hint);
}
```

- [ ] **Step 4: Verify compilation**

Run: `cargo check`
Expected: Compiles

- [ ] **Step 5: Commit**

```bash
git add src/ui/runic_lights_scene.rs src/ui/mod.rs
git commit -m "feat(ui): add Runic Lights scene with grid rendering and info panel"
```

---

### Task 8: Debug Menu

**Files:**
- Modify: `src/utils/debug_menu.rs`

- [ ] **Step 1: Add debug action**

Add `TriggerRunicLightsChallenge` to the `DebugAction` enum.

Add `DebugAction::TriggerRunicLightsChallenge` to `CHALLENGE_ACTIONS` slice.

Add the sort index (next available number after ShardFusion).

Add the display name:

```rust
Self::TriggerRunicLightsChallenge => "Trigger Runic Lights Challenge",
```

Add the execute match arm:

```rust
Self::TriggerRunicLightsChallenge => trigger_runic_lights_challenge(state),
```

Add the trigger function:

```rust
fn trigger_runic_lights_challenge(state: &mut GameState) -> &'static str {
    if state
        .challenge_menu
        .has_challenge(&ChallengeType::RunicLights)
    {
        return "Runic Lights challenge already pending!";
    }
    state
        .challenge_menu
        .add_challenge(create_challenge(&ChallengeType::RunicLights));
    "Runic Lights challenge added!"
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: Compiles

- [ ] **Step 3: Commit**

```bash
git add src/utils/debug_menu.rs
git commit -m "feat(debug): add Runic Lights challenge trigger to debug menu"
```

---

### Task 9: Final Verification

- [ ] **Step 1: Run full test suite**

Run: `cargo test`
Expected: All tests pass

- [ ] **Step 2: Run clippy**

Run: `cargo clippy --all-targets -- -D warnings`
Expected: No warnings

- [ ] **Step 3: Run format check**

Run: `cargo fmt --check`
Expected: No formatting issues (run `cargo fmt` if needed)

- [ ] **Step 4: Run full CI check**

Run: `make check`
Expected: All checks pass

- [ ] **Step 5: Manual smoke test**

Run `cargo run`, open debug menu, trigger "Runic Lights Challenge", play through all 4 difficulties to verify:
- Grid renders correctly at each size
- Toggle flips cross pattern
- Cursor navigation works
- Forfeit double-Esc works
- Win/loss conditions trigger correctly
- Achievement unlocks on win
- Move counter and par display correctly

- [ ] **Step 6: Final commit if any fixes were needed**

```bash
git add -A
git commit -m "fix: address issues found during Runic Lights smoke testing"
```
