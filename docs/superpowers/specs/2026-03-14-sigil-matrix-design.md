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

| Tier | Given Cells | Cells to Remove | Rewards |
|------|-------------|-----------------|---------|
| Novice | 38-42 | 39-43 | 400 Stormglass |
| Apprentice | 30-34 | 47-51 | 1,200 Stormglass |
| Journeyman | 26-28 | 53-55 | 3,000 Stormglass + 1 PR |
| Master | 22-24 | 57-59 | 6,000 Stormglass + 2 PR |

Stormglass fallback (if not yet discovered): Stormglass / 10 = XP % toward next level.

## Game State

```rust
pub enum SudokuDifficulty {
    Novice,
    Apprentice,
    Journeyman,
    Master,
}

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

pub enum SudokuResult {
    Win,
    Loss, // forfeit only
}
```

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

After every `Place` or `Clear`, recalculate `conflicts` for the entire board (check row, column, and 3x3 box for duplicates). When all 81 cells are filled with no conflicts, set `game_result = Some(Win)`.

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

- **`mod.rs`:** Add `Sudoku` to `ChallengeType` enum. Add to `CHALLENGE_TABLE` with weight 18. Register `difficulty_enum_impl!` and `impl_apply_game_result!` macros.
- **`menu.rs`:** Add `ChallengeType::Sudoku` arm in `create_challenge()` with flavor text and ⬡ icon.

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

- Add `SigilMatrixStats` struct: `games_played: u32`, `games_won: u32`, `games_lost: u32`
- Add field to `GameState` (persisted per character)
- Display in achievement browser Stats category tab

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
