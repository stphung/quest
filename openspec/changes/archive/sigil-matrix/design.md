> Backported design record. Sources: docs/superpowers/plans/2026-03-14-sigil-matrix.md, docs/superpowers/specs/2026-03-14-sigil-matrix-design.md.

## 2026-03-14-sigil-matrix.md

# Sigil Matrix Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Sigil Matrix (Sudoku) as the 11th challenge minigame with achievements and stats integration.

**Architecture:** New `src/challenges/sudoku/` module with types, generation, and logic. Integrates with existing challenge discovery, achievement tracking, input routing, and UI overlay systems following established patterns.

**Tech Stack:** Rust, Ratatui, rand (for puzzle generation)

---

## Chunk 1: Core Types and Puzzle Generation

### Task 1: Sudoku Types

**Files:**
- Create: `src/challenges/sudoku/types.rs`

- [ ] **Step 1: Create types.rs with difficulty, result, and game structs**

```rust
use crate::challenges::difficulty_enum_impl;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SudokuDifficulty {
    Novice,
    Apprentice,
    Journeyman,
    Master,
}

difficulty_enum_impl!(SudokuDifficulty);

impl SudokuDifficulty {
    /// Number of cells to remove from the solved board
    pub fn cells_to_remove_range(&self) -> (usize, usize) {
        match self {
            SudokuDifficulty::Novice => (39, 43),
            SudokuDifficulty::Apprentice => (47, 51),
            SudokuDifficulty::Journeyman => (53, 55),
            SudokuDifficulty::Master => (57, 59),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SudokuResult {
    Win,
    Loss,
}

#[derive(Debug, Clone)]
pub struct SudokuGame {
    pub difficulty: SudokuDifficulty,
    pub board: [[u8; 9]; 9],
    pub solution: [[u8; 9]; 9],
    pub given: [[bool; 9]; 9],
    pub conflicts: [[bool; 9]; 9],
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub game_result: Option<SudokuResult>,
    pub forfeit_pending: bool,
}

impl SudokuGame {
    pub fn new(
        difficulty: SudokuDifficulty,
        board: [[u8; 9]; 9],
        solution: [[u8; 9]; 9],
        given: [[bool; 9]; 9],
    ) -> Self {
        Self {
            difficulty,
            board,
            solution,
            given,
            conflicts: [[false; 9]; 9],
            cursor_row: 0,
            cursor_col: 0,
            game_result: None,
            forfeit_pending: false,
        }
    }
}
```

- [ ] **Step 2: Verify it compiles**

Create a minimal `src/challenges/sudoku/mod.rs`:

```rust
pub mod types;

pub use types::*;
```

Do NOT add `pub mod sudoku;` to `src/challenges/mod.rs` yet — we'll do that in the integration task. For now, just verify the files parse correctly:

Run: `cargo check 2>&1 | head -5`

This will show warnings about dead code but should not error since the module isn't wired in yet. Instead, verify syntax by checking the file compiles as a standalone unit conceptually. We'll wire it in during integration.

- [ ] **Step 3: Commit**

```bash
git add src/challenges/sudoku/types.rs src/challenges/sudoku/mod.rs
git commit -m "feat(sudoku): add core types - SudokuDifficulty, SudokuGame, SudokuResult"
```

---

### Task 2: Puzzle Generation

**Files:**
- Create: `src/challenges/sudoku/generation.rs`
- Modify: `src/challenges/sudoku/mod.rs`

- [ ] **Step 1: Write generation.rs with solved board generation**

```rust
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
```

- [ ] **Step 2: Add `pub mod generation;` to mod.rs**

Update `src/challenges/sudoku/mod.rs`:

```rust
pub mod generation;
pub mod types;

pub use generation::*;
pub use types::*;
```

- [ ] **Step 3: Commit**

```bash
git add src/challenges/sudoku/generation.rs src/challenges/sudoku/mod.rs
git commit -m "feat(sudoku): add puzzle generation with backtracking and uniqueness verification"
```

---

### Task 3: Game Logic (Input, Conflicts, Win Check)

**Files:**
- Create: `src/challenges/sudoku/logic.rs`
- Modify: `src/challenges/sudoku/mod.rs`

- [ ] **Step 1: Write logic.rs with input processing, conflict detection, and DifficultyInfo**

```rust
use crate::challenges::menu::{ChallengeReward, DifficultyInfo};

use super::types::{SudokuDifficulty, SudokuGame, SudokuResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SudokuInput {
    Up,
    Down,
    Left,
    Right,
    Place(u8),
    Clear,
    Forfeit,
    Other,
}

pub fn process_sudoku_input(game: &mut SudokuGame, input: SudokuInput) -> bool {
    // Handle forfeit confirmation first
    if game.forfeit_pending {
        match input {
            SudokuInput::Forfeit => {
                crate::challenges::handle_forfeit(
                    &mut game.game_result,
                    &mut game.forfeit_pending,
                    SudokuResult::Loss,
                );
            }
            _ => {
                crate::challenges::cancel_forfeit_if_pending(&mut game.forfeit_pending);
            }
        }
        return true;
    }

    match input {
        SudokuInput::Up => {
            game.cursor_row = if game.cursor_row == 0 { 8 } else { game.cursor_row - 1 };
        }
        SudokuInput::Down => {
            game.cursor_row = (game.cursor_row + 1) % 9;
        }
        SudokuInput::Left => {
            game.cursor_col = if game.cursor_col == 0 { 8 } else { game.cursor_col - 1 };
        }
        SudokuInput::Right => {
            game.cursor_col = (game.cursor_col + 1) % 9;
        }
        SudokuInput::Place(digit) => {
            if !game.given[game.cursor_row][game.cursor_col] {
                game.board[game.cursor_row][game.cursor_col] = digit;
                update_conflicts(game);
                check_win(game);
            }
        }
        SudokuInput::Clear => {
            if !game.given[game.cursor_row][game.cursor_col] {
                game.board[game.cursor_row][game.cursor_col] = 0;
                update_conflicts(game);
            }
        }
        SudokuInput::Forfeit => {
            crate::challenges::handle_forfeit(
                &mut game.game_result,
                &mut game.forfeit_pending,
                SudokuResult::Loss,
            );
        }
        SudokuInput::Other => {}
    }
    true
}

/// Recalculate all conflict flags for the entire board.
fn update_conflicts(game: &mut SudokuGame) {
    game.conflicts = [[false; 9]; 9];

    for r in 0..9 {
        for c in 0..9 {
            let val = game.board[r][c];
            if val == 0 {
                continue;
            }

            // Check row for duplicates
            for c2 in 0..9 {
                if c2 != c && game.board[r][c2] == val {
                    game.conflicts[r][c] = true;
                    game.conflicts[r][c2] = true;
                }
            }

            // Check column for duplicates
            for r2 in 0..9 {
                if r2 != r && game.board[r2][c] == val {
                    game.conflicts[r][c] = true;
                    game.conflicts[r2][c] = true;
                }
            }

            // Check 3x3 box for duplicates
            let box_row = (r / 3) * 3;
            let box_col = (c / 3) * 3;
            for r2 in box_row..box_row + 3 {
                for c2 in box_col..box_col + 3 {
                    if (r2, c2) != (r, c) && game.board[r2][c2] == val {
                        game.conflicts[r][c] = true;
                        game.conflicts[r2][c2] = true;
                    }
                }
            }
        }
    }
}

/// Check if the board is complete and matches the solution.
fn check_win(game: &mut SudokuGame) {
    if game.board == game.solution {
        game.game_result = Some(SudokuResult::Win);
    }
}

/// Count how many cells are currently filled (non-zero).
pub fn filled_count(game: &SudokuGame) -> usize {
    game.board.iter().flatten().filter(|&&v| v != 0).count()
}

/// Count how many cells were given (pre-filled).
pub fn given_count(game: &SudokuGame) -> usize {
    game.given.iter().flatten().filter(|&&v| v).count()
}

impl DifficultyInfo for SudokuDifficulty {
    fn name(&self) -> &'static str {
        SudokuDifficulty::name(self)
    }

    fn reward(&self) -> ChallengeReward {
        match self {
            SudokuDifficulty::Novice => ChallengeReward {
                stormglass: 400,
                ..Default::default()
            },
            SudokuDifficulty::Apprentice => ChallengeReward {
                stormglass: 1_200,
                ..Default::default()
            },
            SudokuDifficulty::Journeyman => ChallengeReward {
                prestige_ranks: 1,
                stormglass: 3_000,
                ..Default::default()
            },
            SudokuDifficulty::Master => ChallengeReward {
                prestige_ranks: 2,
                stormglass: 6_000,
                ..Default::default()
            },
        }
    }

    fn extra_info(&self) -> Option<String> {
        let (min, max) = self.cells_to_remove_range();
        let given_max = 81 - min;
        let given_min = 81 - max;
        Some(format!("{}-{} sigils given", given_min, given_max))
    }
}

impl_apply_game_result! {
    variant: Sudoku;
    result_body: |result, _state, _reward| {
        match result {
            SudokuResult::Win => (true, ""),
            SudokuResult::Loss => (false, "The sigil matrix fractures. The pattern is lost."),
        }
    }
    game_type: crate::achievements::MinigameType::SigilMatrix;
    icon: "\u{2B21}";
    win_message: "The sigil matrix hums with power! Pattern complete.";
}
```

- [ ] **Step 2: Add `pub mod logic;` and `pub use logic::*;` to mod.rs**

Update `src/challenges/sudoku/mod.rs`:

```rust
pub mod generation;
pub mod logic;
pub mod types;

pub use generation::*;
pub use logic::*;
pub use types::*;
```

- [ ] **Step 3: Commit**

```bash
git add src/challenges/sudoku/logic.rs src/challenges/sudoku/mod.rs
git commit -m "feat(sudoku): add input processing, conflict detection, win check, and DifficultyInfo"
```

---

## Chunk 2: System Integration

### Task 4: Wire Into Challenge System

**Files:**
- Modify: `src/challenges/mod.rs` — add module, enum variants, re-exports
- Modify: `src/challenges/menu.rs` — add to discovery table, create_challenge, accept_selected_challenge

- [ ] **Step 1: Add sudoku module and types to src/challenges/mod.rs**

Add `pub mod sudoku;` alongside the other challenge modules (near the top of the file, where `pub mod chess;`, `pub mod rune;`, etc. are listed).

Add to the `pub use` re-exports block:

```rust
pub use sudoku::{SudokuDifficulty, SudokuGame, SudokuInput, SudokuResult};
```

Add `Sudoku(SudokuGame)` variant to the `ActiveMinigame` enum (alongside the other variants like `Rune(RuneGame)`, `Chess(Box<ChessGame>)`, etc.).

Add a match arm to `ActiveMinigame::has_game_result()` (exhaustive match in mod.rs):
```rust
ActiveMinigame::Sudoku(g) => g.game_result.is_some(),
```

- [ ] **Step 2: Add to menu.rs — ChallengeType, CHALLENGE_TABLE, create_challenge, accept_selected_challenge**

Add `Sudoku` variant to the `ChallengeType` enum.

Add to `CHALLENGE_TABLE`:
```rust
ChallengeWeight { challenge_type: ChallengeType::Sudoku, weight: 18 },
```

Add arm to `create_challenge()`:
```rust
ChallengeType::Sudoku => PendingChallenge {
    challenge_type: ChallengeType::Sudoku,
    title: "Sigil Matrix: Arcane Grid".to_string(),
    icon: "\u{2B21}",
    description: "A grid of ancient sigils pulses with arcane energy. Each row, column, and section demands a unique symbol \u{2014} one wrong placement and the matrix destabilizes.".to_string(),
},
```

Add arm to `accept_selected_challenge()`:
```rust
ChallengeType::Sudoku => {
    let d = SudokuDifficulty::from_index(difficulty_index);
    let mut rng = rand::rng();
    ActiveMinigame::Sudoku(crate::challenges::sudoku::generate_puzzle(d, &mut rng))
}
```

Add `ChallengeType::Sudoku` arms to any other exhaustive matches on `ChallengeType` in `menu.rs` (e.g., `icon()`, `flavor_text()`, or similar methods). Use the icon `"\u{2B21}"` and flavor text from the `create_challenge` arm.

- [ ] **Step 3: Verify it compiles**

Run: `cargo check 2>&1 | head -20`
Expected: May have warnings about unused imports but no errors. If there are errors about missing `MinigameType::SigilMatrix`, that's expected — we'll add it in the next task.

- [ ] **Step 4: Commit**

```bash
git add src/challenges/mod.rs src/challenges/menu.rs
git commit -m "feat(sudoku): wire Sigil Matrix into challenge discovery and menu system"
```

---

### Task 5: Achievement Integration

**Files:**
- Modify: `src/achievements/types.rs` — add 4 AchievementId variants, update VARIANT_COUNT
- Modify: `src/achievements/data.rs` — add 4 AchievementDef entries
- Modify: `src/achievements/milestones.rs` — add SigilMatrix to MinigameType
- Modify: `src/achievements/handlers.rs` — add 4 match arms in on_minigame_won

- [ ] **Step 1: Add achievement IDs to types.rs**

Add these 4 variants to the `AchievementId` enum (in the challenges section, after the existing challenge achievements):

```rust
SigilMatrixNovice,
SigilMatrixApprentice,
SigilMatrixJourneyman,
SigilMatrixMaster,
```

Update `VARIANT_COUNT` from `224` to `228`.

- [ ] **Step 2: Add achievement definitions to data.rs**

Add to the `ALL_ACHIEVEMENTS` array (in the challenges section):

```rust
AchievementDef {
    id: AchievementId::SigilMatrixNovice,
    name: "Matrix Initiate",
    description: "Win a Novice Sigil Matrix",
    category: AchievementCategory::Challenges,
    icon: "\u{2B21}",
    points: 10,
},
AchievementDef {
    id: AchievementId::SigilMatrixApprentice,
    name: "Matrix Adept",
    description: "Win an Apprentice Sigil Matrix",
    category: AchievementCategory::Challenges,
    icon: "\u{2B21}",
    points: 25,
},
AchievementDef {
    id: AchievementId::SigilMatrixJourneyman,
    name: "Matrix Weaver",
    description: "Win a Journeyman Sigil Matrix",
    category: AchievementCategory::Challenges,
    icon: "\u{2B21}",
    points: 50,
},
AchievementDef {
    id: AchievementId::SigilMatrixMaster,
    name: "Matrix Sovereign",
    description: "Win a Master Sigil Matrix",
    category: AchievementCategory::Challenges,
    icon: "\u{2B21}",
    points: 100,
},
```

- [ ] **Step 3: Add SigilMatrix to MinigameType in milestones.rs**

Add `SigilMatrix` variant to the `MinigameType` enum:

```rust
pub enum MinigameType {
    Chess,
    Morris,
    Gomoku,
    Minesweeper,
    Rune,
    Go,
    FlappyBird,
    Snake,
    Jezzball,
    RunicShift,
    SigilMatrix,
}
```

- [ ] **Step 4: Add match arms to on_minigame_won in handlers.rs**

Add these 4 match arms to the `on_minigame_won` function's match block:

```rust
(MinigameType::SigilMatrix, MinigameDifficulty::Novice) => Some(AchievementId::SigilMatrixNovice),
(MinigameType::SigilMatrix, MinigameDifficulty::Apprentice) => Some(AchievementId::SigilMatrixApprentice),
(MinigameType::SigilMatrix, MinigameDifficulty::Journeyman) => Some(AchievementId::SigilMatrixJourneyman),
(MinigameType::SigilMatrix, MinigameDifficulty::Master) => Some(AchievementId::SigilMatrixMaster),
```

- [ ] **Step 5: Verify it compiles**

Run: `cargo check 2>&1 | head -20`
Expected: No errors.

- [ ] **Step 6: Commit**

```bash
git add src/achievements/types.rs src/achievements/data.rs src/achievements/milestones.rs src/achievements/handlers.rs
git commit -m "feat(sudoku): add Sigil Matrix achievements (4 difficulty tiers) and MinigameType"
```

---

### Task 6: Input Routing

**Files:**
- Modify: `src/input/minigame_input.rs` — add Sudoku match arm
- Modify: `src/lib.rs` — add re-exports

- [ ] **Step 1: Add Sudoku input routing to minigame_input.rs**

Add a match arm for `ActiveMinigame::Sudoku` in the main input dispatch match. Follow the pattern of the Rune arm:

```rust
ActiveMinigame::Sudoku(sudoku_game) => {
    if sudoku_game.game_result.is_some() {
        state.last_minigame_win = apply_sudoku_result(state);
        return result_for_challenge(&state.last_minigame_win);
    }
    let input = match key.code {
        KeyCode::Up => SudokuInput::Up,
        KeyCode::Down => SudokuInput::Down,
        KeyCode::Left => SudokuInput::Left,
        KeyCode::Right => SudokuInput::Right,
        KeyCode::Char(c @ '1'..='9') => SudokuInput::Place(c as u8 - b'0'),
        KeyCode::Backspace | KeyCode::Delete => SudokuInput::Clear,
        KeyCode::Esc => SudokuInput::Forfeit,
        _ => SudokuInput::Other,
    };
    process_sudoku_input(sudoku_game, input);
}
```

Add the necessary imports at the top of the file (following the aliasing pattern used by other minigames):
```rust
use crate::challenges::sudoku::{
    apply_game_result as apply_sudoku_result, process_sudoku_input, SudokuInput,
};
```

- [ ] **Step 2: Add re-exports to src/lib.rs**

Add to the `pub use challenges::{ ... }` block:

```rust
SudokuDifficulty, SudokuGame, SudokuResult,
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check 2>&1 | head -20`
Expected: No errors (maybe warnings about unused UI code we haven't written yet).

- [ ] **Step 4: Commit**

```bash
git add src/input/minigame_input.rs src/lib.rs
git commit -m "feat(sudoku): add input routing and library re-exports"
```

---

### Task 7: Debug Menu

**Files:**
- Modify: `src/utils/debug_menu.rs`

- [ ] **Step 1: Add Sigil Matrix trigger to debug menu**

Add a `TriggerSudokuChallenge` variant to the `DebugAction` enum.

Add it to the `DEBUG_ACTIONS` array.

Add the display name in the match that maps actions to labels (something like `"Trigger Sigil Matrix Challenge"`).

Add the handler that creates and adds the challenge:
```rust
DebugAction::TriggerSudokuChallenge => {
    if state.challenge_menu.has_challenge(&ChallengeType::Sudoku) {
        "Sigil Matrix challenge already pending!"
    } else {
        state.challenge_menu.add_challenge(create_challenge(&ChallengeType::Sudoku));
        "Sigil Matrix challenge added!"
    }
}
```

Add the necessary import for `ChallengeType::Sudoku` if not already covered by existing imports.

- [ ] **Step 2: Verify it compiles**

Run: `cargo check 2>&1 | head -20`
Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add src/utils/debug_menu.rs
git commit -m "feat(sudoku): add Sigil Matrix trigger to debug menu"
```

---

## Chunk 3: UI and Testing

### Task 8: UI Scene

**Files:**
- Create: `src/ui/sudoku_scene.rs`
- Modify: `src/ui/mod.rs` — add module declaration
- Modify: the file that dispatches `render_*` calls based on `ActiveMinigame` variant (likely `src/ui/minigame_overlay.rs` or similar)

- [ ] **Step 1: Identify the minigame render dispatch location**

Search for where `ActiveMinigame::Rune` triggers `render_rune` to find the dispatch location. This is the file you'll add `ActiveMinigame::Sudoku` rendering to.

Run: `grep -rn "ActiveMinigame::Rune" src/ui/`

- [ ] **Step 2: Create sudoku_scene.rs**

```rust
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    Frame,
};

use crate::challenges::sudoku::{SudokuGame, SudokuResult};

use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_minigame_too_small, render_status_bar,
};

const SUDOKU_CONTROLS: &[(&str, &str)] = &[
    ("Arrows", "Move"),
    ("1-9", "Place"),
    ("BS", "Clear"),
    ("Esc", "Forfeit"),
];
const MIN_WIDTH: u16 = 30;
const MIN_HEIGHT: u16 = 18;

pub fn render_sudoku(
    frame: &mut Frame,
    area: Rect,
    game: &SudokuGame,
    ctx: &super::responsive::LayoutContext,
    show_dismiss_hint: bool,
    stormglass_discovered: bool,
) {
    if game.game_result.is_some() {
        render_sudoku_game_over(frame, area, game, show_dismiss_hint, stormglass_discovered);
        return;
    }

    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_minigame_too_small(frame, area, "Sigil Matrix", MIN_WIDTH, MIN_HEIGHT);
        return;
    }

    let layout = create_game_layout(
        frame,
        area,
        " Sigil Matrix ",
        Color::Magenta,
        15,  // content min height (13 grid rows + 2 padding)
        22,
        ctx,
    );

    render_grid(frame, layout.content, game);
    render_sudoku_status_bar(frame, layout.status_bar, game);
    render_info_panel(frame, layout.info_panel, game);
}

fn render_grid(frame: &mut Frame, area: Rect, game: &SudokuGame) {
    let mut lines: Vec<Line> = Vec::new();

    // Top border
    lines.push(Line::from("┌───────┬───────┬───────┐"));

    for row in 0..9 {
        let mut spans: Vec<Span> = Vec::new();
        spans.push(Span::raw("│"));

        for col in 0..9 {
            if col > 0 && col % 3 == 0 {
                spans.push(Span::raw("│"));
            } else if col > 0 {
                spans.push(Span::raw(" "));
            } else {
                spans.push(Span::raw(" "));
            }

            let val = game.board[row][col];
            let is_cursor = row == game.cursor_row && col == game.cursor_col;

            let style = if is_cursor {
                Style::default().bg(Color::DarkGray).fg(
                    if game.conflicts[row][col] {
                        Color::Red
                    } else if game.given[row][col] {
                        Color::White
                    } else if val != 0 {
                        Color::Cyan
                    } else {
                        Color::DarkGray
                    },
                )
            } else if game.conflicts[row][col] {
                Style::default().fg(Color::Red)
            } else if game.given[row][col] {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else if val != 0 {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let ch = if val == 0 {
                "\u{00b7}".to_string() // middle dot ·
            } else {
                val.to_string()
            };

            spans.push(Span::styled(ch, style));
        }

        spans.push(Span::raw(" │"));
        lines.push(Line::from(spans));

        // Box separator or bottom border
        if row == 2 || row == 5 {
            lines.push(Line::from("├───────┼───────┼───────┤"));
        }
    }

    lines.push(Line::from("└───────┴───────┴───────┘"));

    // Render centered in the content area
    let grid_height = lines.len() as u16;
    let y_offset = if area.height > grid_height {
        (area.height - grid_height) / 2
    } else {
        0
    };

    for (i, line) in lines.iter().enumerate() {
        let y = area.y + y_offset + i as u16;
        if y >= area.y + area.height {
            break;
        }
        let grid_width = 25u16; // "┌───────┬───────┬───────┐" is 25 chars
        let x_offset = if area.width > grid_width {
            (area.width - grid_width) / 2
        } else {
            0
        };
        let render_area = Rect::new(area.x + x_offset, y, grid_width.min(area.width), 1);
        frame.render_widget(ratatui::widgets::Paragraph::new(line.clone()), render_area);
    }
}

fn render_sudoku_status_bar(frame: &mut Frame, area: Rect, game: &SudokuGame) {
    if render_forfeit_status_bar(frame, area, game.forfeit_pending) {
        return;
    }

    let filled = crate::challenges::sudoku::filled_count(game);
    let status_text = format!("{}/81 sigils placed", filled);
    let status_color = if filled == 81 { Color::Yellow } else { Color::Green };

    render_status_bar(frame, area, &status_text, status_color, SUDOKU_CONTROLS);
}

fn render_info_panel(frame: &mut Frame, area: Rect, game: &SudokuGame) {
    let given = crate::challenges::sudoku::given_count(game);
    let filled = crate::challenges::sudoku::filled_count(game);
    let remaining = 81 - filled;

    let mut lines = vec![
        Line::from(Span::styled(
            format!(" {} ", game.difficulty.name()),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!(" Given: {}", given),
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            format!(" Filled: {}/81", filled),
            Style::default().fg(Color::Cyan),
        )),
        Line::from(Span::styled(
            format!(" Remaining: {}", remaining),
            Style::default().fg(if remaining == 0 { Color::Green } else { Color::Gray }),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!(" Row {}, Col {}", game.cursor_row + 1, game.cursor_col + 1),
            Style::default().fg(Color::DarkGray),
        )),
    ];

    // Show conflict count if any
    let conflict_count: usize = game.conflicts.iter().flatten().filter(|&&c| c).count();
    if conflict_count > 0 {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!(" {} conflicts", conflict_count),
            Style::default().fg(Color::Red),
        )));
    }

    let paragraph = ratatui::widgets::Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

fn render_sudoku_game_over(
    frame: &mut Frame,
    area: Rect,
    game: &SudokuGame,
    show_dismiss_hint: bool,
    stormglass_discovered: bool,
) {
    use crate::challenges::menu::DifficultyInfo;

    let (title, color) = match game.game_result {
        Some(SudokuResult::Win) => ("Pattern Complete!", Color::Green),
        Some(SudokuResult::Loss) => ("Matrix Fractured", Color::Red),
        None => return,
    };

    let reward = game.difficulty.reward();
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            title,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    if game.game_result == Some(SudokuResult::Win) {
        if stormglass_discovered {
            lines.push(Line::from(Span::styled(
                format!(" +{} Stormglass", reward.stormglass),
                Style::default().fg(Color::Cyan),
            )));
        }
        if reward.prestige_ranks > 0 {
            lines.push(Line::from(Span::styled(
                format!(" +{} Prestige Rank(s)", reward.prestige_ranks),
                Style::default().fg(Color::Yellow),
            )));
        }
    }

    if show_dismiss_hint {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " Press any key to continue ",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let block = ratatui::widgets::Block::default()
        .borders(ratatui::widgets::Borders::ALL)
        .border_style(Style::default().fg(color))
        .title(" Sigil Matrix ");

    let paragraph = ratatui::widgets::Paragraph::new(lines)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);

    // Center the game-over popup
    let popup_width = 30u16.min(area.width);
    let popup_height = 10u16.min(area.height);
    let popup_area = Rect::new(
        area.x + (area.width.saturating_sub(popup_width)) / 2,
        area.y + (area.height.saturating_sub(popup_height)) / 2,
        popup_width,
        popup_height,
    );

    frame.render_widget(ratatui::widgets::Clear, popup_area);
    frame.render_widget(paragraph, popup_area);
}
```

- [ ] **Step 3: Add module to src/ui/mod.rs**

Add `pub mod sudoku_scene;` to `src/ui/mod.rs`.

- [ ] **Step 4: Add render dispatch for Sudoku**

In the file identified in Step 1 (the minigame render dispatch), add a match arm for `ActiveMinigame::Sudoku`:

```rust
ActiveMinigame::Sudoku(ref game) => {
    sudoku_scene::render_sudoku(frame, area, game, ctx, show_dismiss_hint, sg_discovered);
}
```

Add the import: `use super::sudoku_scene;` (or adjust path as needed based on the file location).

- [ ] **Step 5: Verify it compiles**

Run: `cargo check 2>&1 | head -20`
Expected: No errors.

- [ ] **Step 6: Commit**

```bash
git add src/ui/sudoku_scene.rs src/ui/mod.rs
# Also add the render dispatch file identified in Step 1
git commit -m "feat(sudoku): add Sigil Matrix terminal UI rendering"
```

---

### Task 9: Tests

**Files:**
- Add tests to: `src/challenges/sudoku/generation.rs` (generation tests)
- Add tests to: `src/challenges/sudoku/logic.rs` (logic tests)

- [ ] **Step 1: Add generation tests to generation.rs**

Add at the bottom of `generation.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_generate_solved_board_is_valid() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let board = generate_solved_board(&mut rng);

        // Every cell should be filled (1-9)
        for r in 0..9 {
            for c in 0..9 {
                assert!(board[r][c] >= 1 && board[r][c] <= 9, "Cell ({},{}) is {}", r, c, board[r][c]);
            }
        }

        // Each row should contain 1-9 exactly once
        for r in 0..9 {
            let mut seen = [false; 10];
            for c in 0..9 {
                let v = board[r][c] as usize;
                assert!(!seen[v], "Row {} has duplicate {}", r, v);
                seen[v] = true;
            }
        }

        // Each column should contain 1-9 exactly once
        for c in 0..9 {
            let mut seen = [false; 10];
            for r in 0..9 {
                let v = board[r][c] as usize;
                assert!(!seen[v], "Col {} has duplicate {}", c, v);
                seen[v] = true;
            }
        }

        // Each 3x3 box should contain 1-9 exactly once
        for box_r in 0..3 {
            for box_c in 0..3 {
                let mut seen = [false; 10];
                for r in 0..3 {
                    for c in 0..3 {
                        let v = board[box_r * 3 + r][box_c * 3 + c] as usize;
                        assert!(!seen[v], "Box ({},{}) has duplicate {}", box_r, box_c, v);
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

        // Board should differ from solution (some cells removed)
        assert_ne!(game.board, game.solution);

        // Solution should be valid
        assert!(has_unique_solution(&game.board));

        // Given cells should match solution
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
                given,
            );
        }
    }

    #[test]
    fn test_is_valid_placement() {
        let mut board = [[0u8; 9]; 9];
        board[0][0] = 5;

        // Same row conflict
        assert!(!is_valid_placement(&board, 0, 4, 5));
        // Same column conflict
        assert!(!is_valid_placement(&board, 4, 0, 5));
        // Same box conflict
        assert!(!is_valid_placement(&board, 1, 1, 5));
        // No conflict
        assert!(is_valid_placement(&board, 4, 4, 5));
        // Different digit is fine
        assert!(is_valid_placement(&board, 0, 1, 3));
    }
}
```

- [ ] **Step 2: Add logic tests to logic.rs**

Add at the bottom of `logic.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::challenges::sudoku::types::SudokuGame;

    fn make_test_game() -> SudokuGame {
        let solution = [
            [5,3,4,6,7,8,9,1,2],
            [6,7,2,1,9,5,3,4,8],
            [1,9,8,3,4,2,5,6,7],
            [8,5,9,7,6,1,4,2,3],
            [4,2,6,8,5,3,7,9,1],
            [7,1,3,9,2,4,8,5,6],
            [9,6,1,5,3,7,2,8,4],
            [2,8,7,4,1,9,6,3,5],
            [3,4,5,2,8,6,1,7,9],
        ];
        let mut board = solution;
        let mut given = [[true; 9]; 9];

        // Remove a few cells
        board[0][2] = 0; given[0][2] = false; // was 4
        board[1][1] = 0; given[1][1] = false; // was 7
        board[4][4] = 0; given[4][4] = false; // was 5

        SudokuGame::new(SudokuDifficulty::Novice, board, solution, given)
    }

    #[test]
    fn test_cursor_wrapping() {
        let mut game = make_test_game();
        game.cursor_row = 0;
        process_sudoku_input(&mut game, SudokuInput::Up);
        assert_eq!(game.cursor_row, 8);

        game.cursor_col = 8;
        process_sudoku_input(&mut game, SudokuInput::Right);
        assert_eq!(game.cursor_col, 0);
    }

    #[test]
    fn test_cannot_modify_given_cell() {
        let mut game = make_test_game();
        game.cursor_row = 0;
        game.cursor_col = 0; // given cell (value 5)

        process_sudoku_input(&mut game, SudokuInput::Place(9));
        assert_eq!(game.board[0][0], 5); // unchanged
    }

    #[test]
    fn test_place_and_clear() {
        let mut game = make_test_game();
        game.cursor_row = 0;
        game.cursor_col = 2; // empty cell

        process_sudoku_input(&mut game, SudokuInput::Place(4));
        assert_eq!(game.board[0][2], 4);

        process_sudoku_input(&mut game, SudokuInput::Clear);
        assert_eq!(game.board[0][2], 0);
    }

    #[test]
    fn test_conflict_detection() {
        let mut game = make_test_game();
        game.cursor_row = 0;
        game.cursor_col = 2; // empty cell

        // Place 5 — conflicts with (0,0) which is 5 (same row)
        process_sudoku_input(&mut game, SudokuInput::Place(5));
        assert!(game.conflicts[0][2]);
        assert!(game.conflicts[0][0]); // the other conflicting cell
    }

    #[test]
    fn test_win_condition() {
        let mut game = make_test_game();

        // Fill in the missing cells correctly
        game.cursor_row = 0;
        game.cursor_col = 2;
        process_sudoku_input(&mut game, SudokuInput::Place(4));
        assert!(game.game_result.is_none()); // not done yet

        game.cursor_row = 1;
        game.cursor_col = 1;
        process_sudoku_input(&mut game, SudokuInput::Place(7));
        assert!(game.game_result.is_none()); // not done yet

        game.cursor_row = 4;
        game.cursor_col = 4;
        process_sudoku_input(&mut game, SudokuInput::Place(5));
        assert_eq!(game.game_result, Some(SudokuResult::Win));
    }

    #[test]
    fn test_forfeit_double_esc() {
        let mut game = make_test_game();

        process_sudoku_input(&mut game, SudokuInput::Forfeit);
        assert!(game.forfeit_pending);
        assert!(game.game_result.is_none());

        // Cancel with other input
        process_sudoku_input(&mut game, SudokuInput::Up);
        assert!(!game.forfeit_pending);

        // Double esc to confirm
        process_sudoku_input(&mut game, SudokuInput::Forfeit);
        assert!(game.forfeit_pending);
        process_sudoku_input(&mut game, SudokuInput::Forfeit);
        assert_eq!(game.game_result, Some(SudokuResult::Loss));
    }

    #[test]
    fn test_difficulty_rewards() {
        assert_eq!(SudokuDifficulty::Novice.reward().stormglass, 400);
        assert_eq!(SudokuDifficulty::Novice.reward().prestige_ranks, 0);
        assert_eq!(SudokuDifficulty::Apprentice.reward().stormglass, 1_200);
        assert_eq!(SudokuDifficulty::Journeyman.reward().stormglass, 3_000);
        assert_eq!(SudokuDifficulty::Journeyman.reward().prestige_ranks, 1);
        assert_eq!(SudokuDifficulty::Master.reward().stormglass, 6_000);
        assert_eq!(SudokuDifficulty::Master.reward().prestige_ranks, 2);
    }
}
```

- [ ] **Step 3: Run the tests**

Run: `cargo test sudoku -- --nocapture 2>&1 | tail -30`
Expected: All tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/challenges/sudoku/generation.rs src/challenges/sudoku/logic.rs
git commit -m "test(sudoku): add generation, logic, conflict, forfeit, and reward tests"
```

---

### Task 10: Final Verification

- [ ] **Step 1: Run make check**

Run: `make check`
Expected: All CI checks pass (format, clippy, tests, build).

- [ ] **Step 2: Fix any clippy or formatting issues**

Run: `make fmt` if there are formatting issues.
Fix any clippy warnings.

- [ ] **Step 3: Run all tests one more time**

Run: `cargo test 2>&1 | tail -10`
Expected: All tests pass, including existing tests (no regressions).

- [ ] **Step 4: Final commit (if any fixes were needed)**

Stage only the files that were modified and commit:
```bash
git commit -am "fix(sudoku): address clippy and formatting issues"
```

## 2026-03-14-sigil-matrix-design.md

# Sigil Matrix — Challenge Design Spec

## Overview

Sigil Matrix is a classic Sudoku puzzle challenge for Quest. The player fills a 9x9 grid so that each row, column, and 3x3 box contains the digits 1-9 exactly once. Pre-filled cells are immutable. Conflicts are highlighted in red. Win by completing the board correctly; lose only by forfeiting.

## Theme

- **Name:** Sigil Matrix
- **Icon:** ⬡
- **Flavor text:** "A grid of ancient sigils pulses with arcane energy. Each row, column, and section demands a unique symbol — one wrong placement and the matrix destabilizes."
- **Discovery weight:** 18 (moderate frequency, similar to Containment Breach)
- **Discovery requirements:** P1+ (same as all challenges)

## Difficulty Tiers

| Tier | Given Cells | Cells to Remove | Stormglass | Prestige Ranks | Fishing Ranks |
|------|-------------|-----------------|------------|----------------|---------------|
| Novice | 38-42 | 39-43 | 400 | 0 | 0 |
| Apprentice | 30-34 | 47-51 | 1,200 | 0 | 0 |
| Journeyman | 26-28 | 53-55 | 3,000 | 1 | 0 |
| Master | 22-24 | 57-59 | 6,000 | 2 | 0 |

Stormglass fallback (if not yet discovered): Stormglass / 10 = XP % toward next level.

## Game State

```rust
// Derives: Debug, Clone, Copy, PartialEq, Eq
pub enum SudokuDifficulty {
    Novice,
    Apprentice,
    Journeyman,
    Master,
}
// Generated via: difficulty_enum_impl!(SudokuDifficulty);  (in types.rs)

// DifficultyInfo trait impl (in logic.rs) provides:
//   name() -> "Novice" / "Apprentice" / "Journeyman" / "Master"
//   reward() -> ChallengeReward with values from Difficulty Tiers table
//   extra_info() -> Some("38-42 sigils given") etc.

// Derives: Debug, Clone, Copy, PartialEq, Eq
pub enum SudokuResult {
    Win,
    Loss, // forfeit only
}

// Derives: Debug, Clone
pub struct SudokuGame {
    pub difficulty: SudokuDifficulty,
    pub board: [[u8; 9]; 9],          // Current state (0 = empty)
    pub solution: [[u8; 9]; 9],       // Solved board
    pub given: [[bool; 9]; 9],        // true = pre-filled, immutable
    pub conflicts: [[bool; 9]; 9],    // true = conflicts with another cell
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub game_result: Option<SudokuResult>,
    pub forfeit_pending: bool,
}
```

No `tick_game()` function is needed — Sigil Matrix is a pure logic puzzle with no AI or real-time elements (same as Rune and Minesweeper).

## Puzzle Generation

Runtime generation using backtracking (no pre-built puzzle bank).

### Algorithm

1. **Generate solved board:**
   - Fill the three diagonal 3x3 boxes first (they share no constraints)
   - Solve remaining cells with randomized backtracking
2. **Remove cells:**
   - Randomly select cells to remove (count based on difficulty)
   - After each removal, verify the puzzle still has a unique solution
   - If removing a cell creates multiple solutions, skip it and try another
3. **Uniqueness check:**
   - Backtracking solver that counts solutions, stops at 2

### Why This Works

The 9x9 grid is small enough that backtracking generation with uniqueness verification runs in milliseconds. Minesweeper already does runtime generation in this codebase.

## Input

| Key | Action |
|-----|--------|
| Arrow keys | Move cursor (wraps at edges) |
| 1-9 | Place digit in current cell (ignored if given cell) |
| Backspace/Delete | Clear current cell (ignored if given cell) |
| Esc | Forfeit (double-tap pattern) |

```rust
pub enum SudokuInput {
    Up, Down, Left, Right,
    Place(u8),   // 1-9
    Clear,       // Backspace/Delete
    Forfeit,     // Esc
    Other,
}
```

After every `Place` or `Clear`, recalculate `conflicts` for the entire board (check row, column, and 3x3 box for duplicates). When all 81 cells are filled and `board == solution`, set `game_result = Some(Win)`.

## Error Handling

- **Conflict highlighting:** Placing a digit that conflicts with an existing number in the same row, column, or 3x3 box highlights the conflicting cells in red
- **No mistake limit:** Player can freely place and correct digits
- **No undo:** Player manually clears and re-enters (Backspace + new digit)
- **Win condition:** All 81 cells filled AND board matches solution
- **Loss condition:** Forfeit only (double-Esc)

## Terminal UI Layout

Compact layout (~21 cols × 13 rows) using box-drawing characters:

```
┌───────┬───────┬───────┐
│ 5 3 · │ · 7 · │ · · · │
│ 6 · · │ 1 9 5 │ · · · │
│ · 9 8 │ · · · │ · 6 · │
├───────┼───────┼───────┤
│ 8 · · │ · 6 · │ · · 3 │
│ 4 · · │ 8 · 3 │ · · 1 │
│ 7 · · │ · 2 · │ · · 6 │
├───────┼───────┼───────┤
│ · 6 · │ · · · │ 2 8 · │
│ · · · │ 4 1 9 │ · · 5 │
│ · · · │ · 8 · │ · 7 9 │
└───────┴───────┴───────┘
      Row 1, Col 3  ·  Novice
```

### Color Coding

- **Given cells:** Bold white (immutable)
- **Player-placed cells:** Cyan (editable)
- **Conflicting cells:** Red text
- **Cursor cell:** Highlighted background (reverse video or yellow bg)
- **Empty cells:** Dim `·` character

## Module Structure

```
src/challenges/sudoku/
├── mod.rs         # Public API re-exports
├── types.rs       # SudokuDifficulty, SudokuGame, SudokuResult
├── logic.rs       # Input processing, conflict detection, win check
└── generation.rs  # Puzzle generation (fill + remove + uniqueness)
```

## Integration Points

### Challenge System (`src/challenges/`)

- **`mod.rs`:** Add `pub mod sudoku;` and re-export types. Add `Sudoku` to `ChallengeType` enum. Add to `CHALLENGE_TABLE` with weight 18. Add `ActiveMinigame::Sudoku(SudokuGame)` variant. Invoke `difficulty_enum_impl!(SudokuDifficulty)` in `types.rs`. Invoke `impl_apply_game_result!` in `logic.rs`. Implement `DifficultyInfo` trait for `SudokuDifficulty` in `logic.rs`.
- **`menu.rs`:** Add `ChallengeType::Sudoku` arm in `create_challenge()` with flavor text and ⬡ icon.

### Debug Menu (`src/utils/debug_menu.rs`)

Add "Trigger Sigil Matrix Challenge" entry to the debug menu for testing.

### Input (`src/input/minigame_input.rs`)

Add `ActiveMinigame::Sudoku(SudokuGame)` match arm. Map key codes to `SudokuInput` variants. Call `process_sudoku_input()` and `apply_sudoku_result()`.

### Achievements (`src/achievements/`)

**4 new achievement IDs:**

| AchievementId | Name | Description | Category | Icon | Points |
|---------------|------|-------------|----------|------|--------|
| SigilMatrixNovice | Matrix Initiate | Win a Novice Sigil Matrix | Challenges | ⬡ | 10 |
| SigilMatrixApprentice | Matrix Adept | Win an Apprentice Sigil Matrix | Challenges | ⬡ | 25 |
| SigilMatrixJourneyman | Matrix Weaver | Win a Journeyman Sigil Matrix | Challenges | ⬡ | 50 |
| SigilMatrixMaster | Matrix Sovereign | Win a Master Sigil Matrix | Challenges | ⬡ | 100 |

- Add `SigilMatrix` variant to `MinigameType` enum in `milestones.rs`
- Add 4 match arms in `on_minigame_won` in `handlers.rs`
- Increment `VARIANT_COUNT` from 224 → 228
- Wins count toward `total_minigame_wins` and `GrandChampion` (100 wins)

### Stats

Sigil Matrix wins are tracked via the existing `total_minigame_wins` counter in the `Achievements` struct (account-wide). No per-game stats struct is needed — the other puzzle challenges (Rune, Minesweeper) don't have dedicated stats either. The achievement browser's Stats category tab already displays `total_minigame_wins`.

### Library Crate (`src/lib.rs`)

Re-export `SudokuDifficulty`, `SudokuGame`, and `SudokuResult` alongside the other challenge types.

### UI (`src/ui/`)

- New `sudoku_scene.rs` for rendering the grid, cursor, status line
- Render as overlay (same pattern as chess_scene, rune_scene, etc.)

## Reward Application

Uses the `impl_apply_game_result!` macro:

```rust
impl_apply_game_result! {
    variant: Sudoku;
    result_body: |result, state, reward| {
        match result {
            SudokuResult::Win => (true, ""),
            SudokuResult::Loss => (false, "The sigil matrix fractures. The pattern is lost."),
        }
    }
    game_type: crate::achievements::MinigameType::SigilMatrix;
    icon: "\u{2B21}";  // ⬡
    win_message: "The sigil matrix hums with power! Pattern complete.";
}
```

## Testing

- **Generation tests:** Verify generated puzzles have unique solutions across all difficulties
- **Conflict detection tests:** Verify row, column, and box conflicts are correctly identified
- **Win condition tests:** Verify win triggers only when board is complete and correct
- **Input tests:** Verify given cells can't be modified, cursor wrapping works
- **Forfeit tests:** Verify double-Esc forfeit pattern
- **Difficulty tests:** Verify correct number of given cells per tier
- **Reward tests:** Verify correct Stormglass/PR amounts per difficulty
