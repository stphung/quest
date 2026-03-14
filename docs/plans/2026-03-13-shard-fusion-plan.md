# Shard Fusion Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add "Shard Fusion", a 2048-style challenge minigame where players slide and merge tiles to reach a target value (512/1024/2048/4096 by difficulty).

**Architecture:** New `src/challenges/shard_fusion/` module following the established challenge pattern (types/logic/mod). Animation (slide + merge flash) is driven by the regular 100ms game tick via `tick_stages.rs`, not the realtime frame loop. No AI needed.

**Tech Stack:** Rust, Ratatui (terminal UI), existing challenge framework (`difficulty_enum_impl!`, `impl_apply_game_result!`).

---

### Task 1: Core Types

**Files:**
- Create: `src/challenges/shard_fusion/types.rs`

**Step 1: Create the types file**

```rust
//! Shard Fusion challenge data structures.
//!
//! A 2048-style puzzle minigame where the player merges tiles to reach a target value.

/// Difficulty levels controlling the target tile value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShardFusionDifficulty {
    Novice,
    Apprentice,
    Journeyman,
    Master,
}

difficulty_enum_impl!(ShardFusionDifficulty);

impl ShardFusionDifficulty {
    /// The tile value the player must reach to win.
    pub fn target_value(&self) -> u32 {
        match self {
            Self::Novice => 512,
            Self::Apprentice => 1024,
            Self::Journeyman => 2048,
            Self::Master => 4096,
        }
    }
}

/// Result of a Shard Fusion game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShardFusionResult {
    Win,
    Loss,
}

/// Animation state for tile sliding and merge flash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShardFusionAnimState {
    /// No animation running; input accepted.
    Idle,
    /// Tiles are sliding. Inner value counts down ticks (starts at SLIDE_TICKS).
    Sliding(u32),
    /// Merged tiles are flashing. Inner value counts down ticks (starts at FLASH_TICKS).
    Flashing(u32),
}

/// Duration of the slide animation in game ticks (100ms each).
pub const SLIDE_TICKS: u32 = 4;
/// Duration of the merge flash animation in game ticks.
pub const FLASH_TICKS: u32 = 3;

/// Records one tile's movement for slide rendering.
#[derive(Debug, Clone, Copy)]
pub struct TileMove {
    /// Source cell (row, col).
    pub from: (usize, usize),
    /// Destination cell (row, col).
    pub to: (usize, usize),
    /// The tile value being moved.
    pub value: u32,
}

/// Full Shard Fusion game state.
#[derive(Debug, Clone)]
pub struct ShardFusionGame {
    pub difficulty: ShardFusionDifficulty,
    /// Current board state (0 = empty).
    pub board: [[u32; 4]; 4],
    pub anim_state: ShardFusionAnimState,
    /// Tile movements for slide animation rendering.
    pub slide_moves: Vec<TileMove>,
    /// Cells that just merged, for flash rendering.
    pub merged_cells: Vec<(usize, usize)>,
    pub score: u32,
    pub game_result: Option<ShardFusionResult>,
    pub forfeit_pending: bool,
}

impl ShardFusionGame {
    pub fn new(difficulty: ShardFusionDifficulty) -> Self {
        Self {
            difficulty,
            board: [[0; 4]; 4],
            anim_state: ShardFusionAnimState::Idle,
            slide_moves: Vec::new(),
            merged_cells: Vec::new(),
            score: 0,
            game_result: None,
            forfeit_pending: false,
        }
    }

    /// Returns the highest tile value currently on the board.
    pub fn highest_tile(&self) -> u32 {
        self.board.iter().flatten().copied().max().unwrap_or(0)
    }

    /// Returns the number of empty cells on the board.
    pub fn empty_count(&self) -> usize {
        self.board.iter().flatten().filter(|&&v| v == 0).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_targets() {
        assert_eq!(ShardFusionDifficulty::Novice.target_value(), 512);
        assert_eq!(ShardFusionDifficulty::Apprentice.target_value(), 1024);
        assert_eq!(ShardFusionDifficulty::Journeyman.target_value(), 2048);
        assert_eq!(ShardFusionDifficulty::Master.target_value(), 4096);
    }

    #[test]
    fn test_difficulty_enum_impl() {
        assert_eq!(ShardFusionDifficulty::from_index(0), ShardFusionDifficulty::Novice);
        assert_eq!(ShardFusionDifficulty::from_index(3), ShardFusionDifficulty::Master);
        assert_eq!(ShardFusionDifficulty::from_index(99), ShardFusionDifficulty::Novice);
        assert_eq!(ShardFusionDifficulty::ALL.len(), 4);
    }

    #[test]
    fn test_game_new() {
        let game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        assert!(game.board.iter().flatten().all(|&v| v == 0));
        assert_eq!(game.anim_state, ShardFusionAnimState::Idle);
        assert!(game.slide_moves.is_empty());
        assert!(game.merged_cells.is_empty());
        assert_eq!(game.score, 0);
        assert!(game.game_result.is_none());
        assert!(!game.forfeit_pending);
    }

    #[test]
    fn test_highest_tile_empty_board() {
        let game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        assert_eq!(game.highest_tile(), 0);
    }

    #[test]
    fn test_highest_tile_with_values() {
        let mut game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        game.board[0][0] = 2;
        game.board[1][1] = 128;
        game.board[3][3] = 64;
        assert_eq!(game.highest_tile(), 128);
    }

    #[test]
    fn test_empty_count() {
        let mut game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        assert_eq!(game.empty_count(), 16);
        game.board[0][0] = 2;
        game.board[0][1] = 4;
        assert_eq!(game.empty_count(), 14);
    }
}
```

**Step 2: Run tests**

```bash
cargo test shard_fusion::types
```

Expected: all tests pass (the file is self-contained at this stage).

**Step 3: Commit**

```bash
git add src/challenges/shard_fusion/types.rs
git commit -m "feat(shard-fusion): add core types"
```

---

### Task 2: Game Logic

**Files:**
- Create: `src/challenges/shard_fusion/logic.rs`

**Step 1: Write failing tests first**

At the top of `logic.rs`, add the test module before writing any implementation:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Helper: build a board from a flat 16-element array (row-major).
    fn board(vals: [u32; 16]) -> [[u32; 4]; 4] {
        let mut b = [[0u32; 4]; 4];
        for r in 0..4 {
            for c in 0..4 {
                b[r][c] = vals[r * 4 + c];
            }
        }
        b
    }

    #[test]
    fn test_slide_left_basic_merge() {
        // [2, 2, 0, 0] -> [4, 0, 0, 0]
        let result = slide_row_left([2, 2, 0, 0]);
        assert_eq!(result.row, [4, 0, 0, 0]);
        assert_eq!(result.score_gain, 4);
        assert_eq!(result.merged_cols, vec![0]);
    }

    #[test]
    fn test_slide_left_no_merge() {
        // [2, 4, 8, 16] -> unchanged
        let result = slide_row_left([2, 4, 8, 16]);
        assert_eq!(result.row, [2, 4, 8, 16]);
        assert_eq!(result.score_gain, 0);
        assert!(result.merged_cols.is_empty());
    }

    #[test]
    fn test_slide_left_compacts() {
        // [0, 0, 2, 4] -> [2, 4, 0, 0]
        let result = slide_row_left([0, 0, 2, 4]);
        assert_eq!(result.row, [2, 4, 0, 0]);
        assert_eq!(result.score_gain, 0);
    }

    #[test]
    fn test_slide_left_no_double_merge() {
        // [2, 2, 2, 2] -> [4, 4, 0, 0] (not [8, 0, 0, 0])
        let result = slide_row_left([2, 2, 2, 2]);
        assert_eq!(result.row, [4, 4, 0, 0]);
        assert_eq!(result.score_gain, 8);
    }

    #[test]
    fn test_has_valid_moves_full_no_adjacent() {
        // A full board with no adjacent matches has no valid moves.
        let b = board([
            2, 4, 2, 4,
            4, 2, 4, 2,
            2, 4, 2, 4,
            4, 2, 4, 2,
        ]);
        assert!(!has_valid_moves(&b));
    }

    #[test]
    fn test_has_valid_moves_empty_cell() {
        let mut b = [[0u32; 4]; 4];
        b[0][0] = 2;
        assert!(has_valid_moves(&b));
    }

    #[test]
    fn test_has_valid_moves_adjacent_match() {
        let b = board([
            2, 2, 4, 8,
            4, 8, 16, 32,
            8, 16, 32, 64,
            16, 32, 64, 128,
        ]);
        assert!(has_valid_moves(&b));
    }

    #[test]
    fn test_apply_slide_left_updates_board_and_score() {
        let mut game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        game.board = board([
            2, 2, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        // Pre-place the new tile spawn position to avoid RNG dependency:
        // We test score and board directly; spawn is tested separately.
        let changed = apply_slide(&mut game, Direction::Left);
        assert!(changed);
        assert_eq!(game.board[0][0], 4);
        assert_eq!(game.score, 4);
        assert_eq!(game.anim_state, ShardFusionAnimState::Sliding(SLIDE_TICKS));
    }

    #[test]
    fn test_apply_slide_no_change_returns_false() {
        let mut game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        game.board = board([
            2, 4, 8, 16,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        // Left slide has nowhere to go (already compacted left, no merges possible).
        let changed = apply_slide(&mut game, Direction::Left);
        assert!(!changed);
        assert_eq!(game.anim_state, ShardFusionAnimState::Idle);
    }

    #[test]
    fn test_tick_advances_sliding_to_flashing() {
        let mut game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        game.merged_cells = vec![(0, 0)]; // simulate a merge happened
        game.anim_state = ShardFusionAnimState::Sliding(1);
        tick_shard_fusion(&mut game);
        // Sliding(1) -> tick -> Flashing(FLASH_TICKS) because there are merged cells
        assert_eq!(game.anim_state, ShardFusionAnimState::Flashing(FLASH_TICKS));
    }

    #[test]
    fn test_tick_advances_sliding_to_idle_no_merges() {
        let mut game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        game.anim_state = ShardFusionAnimState::Sliding(1);
        // No merged cells -> skip flash phase
        tick_shard_fusion(&mut game);
        assert_eq!(game.anim_state, ShardFusionAnimState::Idle);
    }

    #[test]
    fn test_tick_advances_flashing_to_idle() {
        let mut game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        game.anim_state = ShardFusionAnimState::Flashing(1);
        tick_shard_fusion(&mut game);
        assert_eq!(game.anim_state, ShardFusionAnimState::Idle);
    }

    #[test]
    fn test_tick_counts_down() {
        let mut game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        game.anim_state = ShardFusionAnimState::Sliding(3);
        tick_shard_fusion(&mut game);
        assert_eq!(game.anim_state, ShardFusionAnimState::Sliding(2));
    }

    #[test]
    fn test_win_detected_after_slide() {
        let mut game = ShardFusionGame::new(ShardFusionDifficulty::Novice); // target = 512
        game.board = board([
            256, 256, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
            0, 0, 0, 0,
        ]);
        apply_slide(&mut game, Direction::Left);
        // After slide, 256+256=512 = target. Win should be set.
        assert_eq!(game.game_result, Some(ShardFusionResult::Win));
    }

    #[test]
    fn test_loss_detected_when_no_moves() {
        let mut game = ShardFusionGame::new(ShardFusionDifficulty::Novice);
        // Fill board with no valid moves (alternating pattern).
        game.board = board([
            2, 4, 2, 4,
            4, 2, 4, 2,
            2, 4, 2, 4,
            4, 2, 4, 2,
        ]);
        // apply_slide in any direction won't change the board; loss should be set
        // We test via apply_slide which checks after each move.
        // Since no change is possible, we directly call check_game_over.
        check_game_over(&mut game);
        assert_eq!(game.game_result, Some(ShardFusionResult::Loss));
    }
}
```

**Step 2: Run tests to confirm they fail**

```bash
cargo test shard_fusion::logic 2>&1 | head -20
```

Expected: compile error (functions not defined yet).

**Step 3: Implement the logic**

Write the full implementation above the test module:

```rust
//! Shard Fusion game logic.
//!
//! Handles tile sliding, merging, animation, spawn, and win/loss detection.

use super::{
    ShardFusionAnimState, ShardFusionDifficulty, ShardFusionGame, ShardFusionResult, TileMove,
    FLASH_TICKS, SLIDE_TICKS,
};
use crate::challenges::ActiveMinigame;
use rand::{Rng, RngExt};

/// Direction for a slide move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

/// Input actions for Shard Fusion (UI-agnostic).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShardFusionInput {
    Left,
    Right,
    Up,
    Down,
    Forfeit,
    Other,
}

/// Result of sliding one row left: new row contents, score gained, and which
/// destination columns received a merge.
pub struct SlideRowResult {
    pub row: [u32; 4],
    pub score_gain: u32,
    /// Destination column indices where a merge occurred.
    pub merged_cols: Vec<usize>,
}

/// Slide a single row to the left, compacting and merging.
/// Each pair of identical values merges once per move.
pub fn slide_row_left(row: [u32; 4]) -> SlideRowResult {
    // Compact: remove zeros
    let tiles: Vec<u32> = row.iter().copied().filter(|&v| v != 0).collect();

    let mut result = [0u32; 4];
    let mut merged_cols = Vec::new();
    let mut out_idx = 0;
    let mut i = 0;

    while i < tiles.len() {
        if i + 1 < tiles.len() && tiles[i] == tiles[i + 1] {
            result[out_idx] = tiles[i] * 2;
            merged_cols.push(out_idx);
            out_idx += 1;
            i += 2;
        } else {
            result[out_idx] = tiles[i];
            out_idx += 1;
            i += 1;
        }
    }

    let score_gain: u32 = merged_cols.iter().map(|&c| result[c]).sum();

    SlideRowResult {
        row: result,
        score_gain,
        merged_cols,
    }
}

/// Returns true if the board has at least one valid move (empty cell or adjacent match).
pub fn has_valid_moves(board: &[[u32; 4]; 4]) -> bool {
    for r in 0..4 {
        for c in 0..4 {
            if board[r][c] == 0 {
                return true;
            }
            if c + 1 < 4 && board[r][c] == board[r][c + 1] {
                return true;
            }
            if r + 1 < 4 && board[r][c] == board[r + 1][c] {
                return true;
            }
        }
    }
    false
}

/// Rotate the board 90° clockwise (used to reuse slide_row_left for all directions).
fn rotate_cw(board: [[u32; 4]; 4]) -> [[u32; 4]; 4] {
    let mut out = [[0u32; 4]; 4];
    for r in 0..4 {
        for c in 0..4 {
            out[c][3 - r] = board[r][c];
        }
    }
    out
}

/// Rotate the board 90° counter-clockwise.
fn rotate_ccw(board: [[u32; 4]; 4]) -> [[u32; 4]; 4] {
    let mut out = [[0u32; 4]; 4];
    for r in 0..4 {
        for c in 0..4 {
            out[3 - c][r] = board[r][c];
        }
    }
    out
}

/// Slide the entire board in the given direction.
/// Returns (new_board, score_gain, slide_moves, merged_cells).
/// Does NOT spawn a new tile.
fn slide_board(
    board: [[u32; 4]; 4],
    direction: Direction,
) -> ([[u32; 4]; 4], u32, Vec<TileMove>, Vec<(usize, usize)>) {
    // Normalize: rotate so we always slide left.
    let rotations = match direction {
        Direction::Left => 0,
        Direction::Down => 1,
        Direction::Right => 2,
        Direction::Up => 3,
    };

    let mut working = board;
    for _ in 0..rotations {
        working = rotate_cw(working);
    }

    let mut new_board = [[0u32; 4]; 4];
    let mut total_score = 0u32;
    let mut merged_cols_by_row: Vec<(usize, Vec<usize>)> = Vec::new();

    for r in 0..4 {
        let result = slide_row_left(working[r]);
        new_board[r] = result.row;
        total_score += result.score_gain;
        if !result.merged_cols.is_empty() {
            merged_cols_by_row.push((r, result.merged_cols));
        }
    }

    // Rotate back
    let reverse_rotations = (4 - rotations) % 4;
    for _ in 0..reverse_rotations {
        new_board = rotate_cw(new_board);
    }

    // Build TileMoves by comparing original board to new board.
    // This is an approximation: we record (from, to, value) for each non-zero tile
    // in the final board, tracing where it came from in the pre-slide board.
    // For simplicity, we record moves in the rotated coordinate space, then un-rotate.
    let mut slide_moves = Vec::new();
    let mut merged_cells = Vec::new();

    // Build merged cells (in original coordinates)
    for (row, cols) in &merged_cols_by_row {
        for &col in cols {
            // Un-rotate the (row, col) coordinate back to original space
            let (orig_r, orig_c) = unrotate_coord(*row, col, rotations);
            merged_cells.push((orig_r, orig_c));
        }
    }

    // Build slide moves: for each cell in working board that is non-zero,
    // record an approximate from→to. We do a simple column-trace.
    for r in 0..4 {
        let original_row = working[r];
        let result_row = {
            let res = slide_row_left(original_row);
            res.row
        };

        // Map source tiles to destination positions
        let sources: Vec<(usize, u32)> = original_row
            .iter()
            .enumerate()
            .filter(|(_, &v)| v != 0)
            .map(|(c, &v)| (c, v))
            .collect();

        let mut dest_idx = 0;
        let mut s_idx = 0;
        while s_idx < sources.len() {
            let (from_c, val) = sources[s_idx];
            if s_idx + 1 < sources.len() && sources[s_idx].1 == sources[s_idx + 1].1 {
                // Two tiles merge into dest_idx
                let (from_c2, _) = sources[s_idx + 1];
                let (to_r, to_c) = unrotate_coord(r, dest_idx, rotations);
                let (from_r, from_c_orig) = unrotate_coord(r, from_c, rotations);
                let (from_r2, from_c2_orig) = unrotate_coord(r, from_c2, rotations);
                slide_moves.push(TileMove { from: (from_r, from_c_orig), to: (to_r, to_c), value: val });
                slide_moves.push(TileMove { from: (from_r2, from_c2_orig), to: (to_r, to_c), value: val });
                dest_idx += 1;
                s_idx += 2;
            } else {
                let to_dest = dest_idx;
                let (to_r, to_c) = unrotate_coord(r, to_dest, rotations);
                let (from_r, from_c_orig) = unrotate_coord(r, from_c, rotations);
                if (from_r, from_c_orig) != (to_r, to_c) {
                    slide_moves.push(TileMove { from: (from_r, from_c_orig), to: (to_r, to_c), value: val });
                }
                dest_idx += 1;
                s_idx += 1;
            }
        }
        let _ = result_row; // used via slide_row_left above
    }

    (new_board, total_score, slide_moves, merged_cells)
}

/// Un-rotate a (row, col) coordinate from the rotated space back to original space.
fn unrotate_coord(r: usize, c: usize, rotations: usize) -> (usize, usize) {
    let mut pos = (r, c);
    for _ in 0..rotations {
        // Un-rotate one step CCW: (r,c) in CW-rotated = (3-c, r) in original
        pos = (3 - pos.1, pos.0);
    }
    pos
}

/// Spawn a new tile (2 or 4) in a random empty cell. 90% chance of 2, 10% of 4.
pub fn spawn_tile<R: Rng>(board: &mut [[u32; 4]; 4], rng: &mut R) {
    let empty_cells: Vec<(usize, usize)> = (0..4)
        .flat_map(|r| (0..4).map(move |c| (r, c)))
        .filter(|&(r, c)| board[r][c] == 0)
        .collect();

    if empty_cells.is_empty() {
        return;
    }

    let idx = rng.random_range(0..empty_cells.len());
    let (r, c) = empty_cells[idx];
    board[r][c] = if rng.random::<f32>() < 0.9 { 2 } else { 4 };
}

/// Check if the game is over (win or loss) and update game_result.
pub fn check_game_over(game: &mut ShardFusionGame) {
    if game.game_result.is_some() {
        return;
    }
    if game.highest_tile() >= game.difficulty.target_value() {
        game.game_result = Some(ShardFusionResult::Win);
        return;
    }
    if !has_valid_moves(&game.board) {
        game.game_result = Some(ShardFusionResult::Loss);
    }
}

/// Apply a slide in the given direction. Returns true if the board changed.
/// If the board changed, begins the slide animation and schedules tile spawn after animation.
pub fn apply_slide(game: &mut ShardFusionGame, direction: Direction) -> bool {
    let (new_board, score_gain, slide_moves, merged_cells) =
        slide_board(game.board, direction);

    if new_board == game.board {
        return false;
    }

    game.board = new_board;
    game.score += score_gain;
    game.slide_moves = slide_moves;
    game.merged_cells = merged_cells;
    game.anim_state = ShardFusionAnimState::Sliding(SLIDE_TICKS);

    // Check win immediately (tile spawn happens in tick, but win check can be early).
    check_game_over(game);

    true
}

/// Start a new Shard Fusion game, spawning 2 initial tiles.
pub fn start_shard_fusion_game<R: Rng>(
    difficulty: ShardFusionDifficulty,
    rng: &mut R,
) -> ActiveMinigame {
    let mut game = ShardFusionGame::new(difficulty);
    spawn_tile(&mut game.board, rng);
    spawn_tile(&mut game.board, rng);
    ActiveMinigame::ShardFusion(game)
}

/// Advance the animation state by one game tick (called every 100ms from tick_stages).
/// Spawns a new tile when the slide animation ends.
pub fn tick_shard_fusion<R: Rng>(game: &mut ShardFusionGame, rng: &mut R) {
    match game.anim_state {
        ShardFusionAnimState::Idle => {}
        ShardFusionAnimState::Sliding(t) => {
            if t <= 1 {
                // Slide done: spawn tile, then start flash if merges occurred.
                if game.game_result.is_none() {
                    spawn_tile(&mut game.board, rng);
                    check_game_over(game);
                }
                game.slide_moves.clear();
                if !game.merged_cells.is_empty() {
                    game.anim_state = ShardFusionAnimState::Flashing(FLASH_TICKS);
                } else {
                    game.anim_state = ShardFusionAnimState::Idle;
                }
            } else {
                game.anim_state = ShardFusionAnimState::Sliding(t - 1);
            }
        }
        ShardFusionAnimState::Flashing(t) => {
            if t <= 1 {
                game.merged_cells.clear();
                game.anim_state = ShardFusionAnimState::Idle;
            } else {
                game.anim_state = ShardFusionAnimState::Flashing(t - 1);
            }
        }
    }
}

/// Process player input. Input is ignored during animation.
pub fn process_input<R: Rng>(game: &mut ShardFusionGame, input: ShardFusionInput, rng: &mut R) {
    if game.game_result.is_some() {
        return;
    }

    // Forfeit flow (double-Esc)
    if game.forfeit_pending {
        match input {
            ShardFusionInput::Forfeit => {
                crate::challenges::handle_forfeit(
                    &mut game.game_result,
                    &mut game.forfeit_pending,
                    ShardFusionResult::Loss,
                );
            }
            _ => {
                crate::challenges::cancel_forfeit_if_pending(&mut game.forfeit_pending);
            }
        }
        return;
    }

    // Block input during animation
    if game.anim_state != ShardFusionAnimState::Idle {
        return;
    }

    match input {
        ShardFusionInput::Left => { apply_slide(game, Direction::Left); }
        ShardFusionInput::Right => { apply_slide(game, Direction::Right); }
        ShardFusionInput::Up => { apply_slide(game, Direction::Up); }
        ShardFusionInput::Down => { apply_slide(game, Direction::Down); }
        ShardFusionInput::Forfeit => {
            crate::challenges::handle_forfeit(
                &mut game.game_result,
                &mut game.forfeit_pending,
                ShardFusionResult::Loss,
            );
        }
        ShardFusionInput::Other => {}
    }
}
```

Note: the test module uses `tick_shard_fusion` without `rng` — update the test helper or adjust the signature. The simplest fix: in tests use a `rand::rngs::SmallRng` seeded with `rand::SeedableRng::seed_from_u64(0)`.

**Step 4: Run tests**

```bash
cargo test shard_fusion::logic
```

Expected: all tests pass.

**Step 5: Commit**

```bash
git add src/challenges/shard_fusion/logic.rs
git commit -m "feat(shard-fusion): add slide/merge logic and animation ticking"
```

---

### Task 3: Module Wiring

**Files:**
- Create: `src/challenges/shard_fusion/mod.rs`
- Modify: `src/challenges/mod.rs`

**Step 1: Create `shard_fusion/mod.rs`**

```rust
//! Shard Fusion challenge — 2048-style tile-merging puzzle.

mod logic;
mod types;

pub use logic::{
    process_input, start_shard_fusion_game, tick_shard_fusion, Direction, ShardFusionInput,
};
pub use types::{
    ShardFusionAnimState, ShardFusionDifficulty, ShardFusionGame, ShardFusionResult, TileMove,
    FLASH_TICKS, SLIDE_TICKS,
};

use crate::core::game_state::GameState;

impl_apply_game_result! {
    variant: ShardFusion;
    result_body: |result, _state, _reward| {
        use ShardFusionResult::*;
        match result {
            Win => (true, ""),
            Loss => (false, "The shards refuse to fuse further."),
        }
    }
    game_type: crate::achievements::MinigameType::ShardFusion;
    icon: "\u{25C6}";
    win_message: "Fusion achieved!";
}
```

**Step 2: Add to `src/challenges/mod.rs`**

Add the module declaration (alphabetical with other `pub mod` lines):
```rust
pub mod shard_fusion;
```

Add re-exports (after `pub use runic_shift::`):
```rust
pub use shard_fusion::{
    ShardFusionAnimState, ShardFusionDifficulty, ShardFusionGame, ShardFusionResult, TileMove,
};
```

Add to `ActiveMinigame` enum (after `RunicShift`):
```rust
ShardFusion(ShardFusionGame),
```

Add to `has_game_result` match:
```rust
ActiveMinigame::ShardFusion(g) => g.game_result.is_some(),
```

**Step 3: Build**

```bash
cargo build 2>&1 | head -30
```

Expected: compile errors only for missing `MinigameType::ShardFusion` (fixed in Task 4).

**Step 4: Commit (after Task 4 compiles)**

Hold commit until Task 4 is done.

---

### Task 4: Menu + Achievement Integration

**Files:**
- Modify: `src/challenges/menu.rs`
- Modify: `src/achievements/milestones.rs`

**Step 1: Add `MinigameType::ShardFusion` to achievements**

In `src/achievements/milestones.rs`, find the `MinigameType` enum and add:
```rust
ShardFusion,
```

Check how `MinigameType` is matched (grep for `MinigameType::Jezzball` or `MinigameType::RunicShift`) and add the `ShardFusion` arm to any exhaustive matches. Look for pattern like:
```rust
MinigameType::Jezzball => "Jezzball",
```
and add:
```rust
MinigameType::ShardFusion => "Shard Fusion",
```

**Step 2: Add `ChallengeType::ShardFusion` to menu.rs**

In `src/challenges/menu.rs`:

1. Add import at top: `use super::shard_fusion::ShardFusionDifficulty;`

2. Add to `ChallengeType` enum:
```rust
ShardFusion,
```

3. Add icon to `ChallengeType::icon()`:
```rust
ChallengeType::ShardFusion => "\u{25C6}",  // ◆
```

4. Add name to `ChallengeType::name()` (if it exists, otherwise check `Display`):
```rust
ChallengeType::ShardFusion => "Shard Fusion",
```

5. Implement `DifficultyInfo` for `ShardFusionDifficulty`. Find where `impl DifficultyInfo for RunicShiftDifficulty` is and add after it:
```rust
impl DifficultyInfo for ShardFusionDifficulty {
    fn name(&self) -> &'static str {
        self.name()
    }

    fn reward(&self) -> ChallengeReward {
        match self {
            Self::Novice => ChallengeReward { prestige_ranks: 1, stormglass: 1_000, fishing_ranks: 0 },
            Self::Apprentice => ChallengeReward { prestige_ranks: 1, stormglass: 2_500, fishing_ranks: 0 },
            Self::Journeyman => ChallengeReward { prestige_ranks: 1, stormglass: 5_000, fishing_ranks: 0 },
            Self::Master => ChallengeReward { prestige_ranks: 2, stormglass: 10_000, fishing_ranks: 0 },
        }
    }

    fn extra_info(&self) -> Option<String> {
        Some(format!("Target: {}", self.target_value()))
    }
}
```

6. Add to `create_challenge()` match:
```rust
ChallengeType::ShardFusion => {
    let d = ShardFusionDifficulty::from_index(difficulty_index);
    Challenge::new(challenge_type, d.name(), d.reward())
}
```

7. Add to `accept_selected_challenge()` match:
```rust
ChallengeType::ShardFusion => {
    let d = ShardFusionDifficulty::from_index(difficulty_index);
    crate::challenges::shard_fusion::start_shard_fusion_game(d, rng)
}
```

8. Add to `CHALLENGE_TABLE`:
```rust
ChallengeWeight {
    challenge_type: ChallengeType::ShardFusion,
    weight: 20,
},
```

**Step 3: Build**

```bash
cargo build 2>&1 | head -30
```

Expected: no errors (or only missing input/UI wiring from later tasks).

**Step 4: Commit**

```bash
git add src/challenges/shard_fusion/mod.rs src/challenges/mod.rs src/challenges/menu.rs src/achievements/milestones.rs
git commit -m "feat(shard-fusion): wire module, ActiveMinigame, ChallengeType, achievements"
```

---

### Task 5: Tick Integration

**Files:**
- Modify: `src/core/tick_stages.rs`

**Step 1: Add ShardFusion arm to the AI thinking dispatch**

Find the block in `tick_stages.rs` around line 671 that matches `&mut state.active_minigame` for AI thinking. It looks like:

```rust
match &mut state.active_minigame {
    Some(ActiveMinigame::Chess(game)) => { ... }
    Some(ActiveMinigame::Morris(game)) => { ... }
    ...
    _ => {}
}
```

Add a ShardFusion arm to this match (it doesn't do AI thinking, but does do animation ticking):
```rust
Some(ActiveMinigame::ShardFusion(game)) => {
    crate::challenges::shard_fusion::tick_shard_fusion(game, rng);
}
```

**Step 2: Build**

```bash
cargo build 2>&1 | head -20
```

**Step 3: Commit**

```bash
git add src/core/tick_stages.rs
git commit -m "feat(shard-fusion): drive animation from game tick"
```

---

### Task 6: UI Scene

**Files:**
- Create: `src/ui/shard_fusion_scene.rs`
- Modify: `src/ui/mod.rs`

**Step 1: Create the scene file**

Key rendering approach:
- Each of the 4×4 cells is rendered as a bordered block with the tile value centered
- Tile color is determined by value (see color map below)
- During `Sliding(t)`: interpolate tile position using `t / SLIDE_TICKS` fraction — render tiles at an offset between `from` and `to` positions. Because Ratatui uses integer cell coordinates, use `(SLIDE_TICKS - t) / SLIDE_TICKS` to compute progress (0.0 = start, 1.0 = end) and round to nearest cell column/row.
- During `Flashing(t)`: merged cells render with `Color::White` background regardless of value
- Info panel shows: Score, Target, Highest Tile

```rust
//! Shard Fusion scene renderer.

use crate::challenges::shard_fusion::{
    ShardFusionAnimState, ShardFusionGame, TileMove, SLIDE_TICKS,
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_game_over_overlay,
    render_status_bar, GameResultType,
};

/// Returns the foreground color for a tile value.
fn tile_color(value: u32) -> Color {
    match value {
        0 => Color::DarkGray,
        2 | 4 => Color::White,
        8 | 16 => Color::Yellow,
        32 | 64 => Color::LightGreen,
        128 | 256 => Color::LightCyan,
        512 | 1024 => Color::LightMagenta,
        _ => Color::LightRed, // 2048+
    }
}

/// Render the Shard Fusion scene.
pub fn render_shard_fusion_scene(
    frame: &mut Frame,
    area: Rect,
    game: &ShardFusionGame,
    show_dismiss_hint: bool,
) {
    use crate::challenges::ShardFusionResult;

    if let Some(result) = game.game_result {
        let result_type = match result {
            ShardFusionResult::Win => GameResultType::Win,
            ShardFusionResult::Loss => {
                if game.forfeit_pending {
                    GameResultType::Forfeit
                } else {
                    GameResultType::Loss
                }
            }
        };
        render_game_over_overlay(
            frame,
            area,
            result_type,
            &format!("Score: {}", game.score),
            show_dismiss_hint,
        );
        return;
    }

    let layout = create_game_layout(frame, area, " Shard Fusion ", Color::Yellow, 20, 24);

    render_board(frame, layout.content, game);
    render_status_bar_content(frame, layout.status_bar, game);
    render_info_panel(frame, layout.info_panel, game);
}

fn render_board(frame: &mut Frame, area: Rect, game: &ShardFusionGame) {
    // Divide area into a 4×4 grid.
    let row_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
        ])
        .split(area);

    // Build a map: (row, col) -> display value, color, is_merging
    let mut display = [[(0u32, Color::DarkGray, false); 4]; 4];
    for r in 0..4 {
        for c in 0..4 {
            let v = game.board[r][c];
            let is_merging = game.merged_cells.contains(&(r, c));
            display[r][c] = (v, tile_color(v), is_merging);
        }
    }

    // During slide: overlay animated tiles at interpolated positions.
    // We render slide_moves at their interpolated position (snap to nearest cell).
    let slide_overlay: Vec<(usize, usize, u32)> = if let ShardFusionAnimState::Sliding(t) = game.anim_state {
        let progress = (SLIDE_TICKS - t) as f32 / SLIDE_TICKS as f32;
        game.slide_moves.iter().map(|tm| {
            let r = (tm.from.0 as f32 + (tm.to.0 as f32 - tm.from.0 as f32) * progress).round() as usize;
            let c = (tm.from.1 as f32 + (tm.to.1 as f32 - tm.from.1 as f32) * progress).round() as usize;
            let r = r.min(3);
            let c = c.min(3);
            (r, c, tm.value)
        }).collect()
    } else {
        Vec::new()
    };

    for r in 0..4 {
        let col_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Ratio(1, 4),
                Constraint::Ratio(1, 4),
                Constraint::Ratio(1, 4),
                Constraint::Ratio(1, 4),
            ])
            .split(row_chunks[r]);

        for c in 0..4 {
            let cell_area = col_chunks[c];
            let (base_val, base_color, is_merging) = display[r][c];

            // Check if a sliding tile is at this position.
            let slide_val = slide_overlay.iter()
                .filter(|&&(sr, sc, _)| sr == r && sc == c)
                .map(|&(_, _, v)| v)
                .max();

            let (val, color) = if let Some(sv) = slide_val {
                (sv, tile_color(sv))
            } else {
                (base_val, base_color)
            };

            // Flash: override color for merged cells during flash phase.
            let color = if is_merging && matches!(game.anim_state, ShardFusionAnimState::Flashing(_)) {
                Color::White
            } else {
                color
            };

            let text = if val == 0 {
                String::new()
            } else {
                val.to_string()
            };

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray));

            let para = Paragraph::new(Line::from(Span::styled(
                text,
                Style::default().fg(color),
            )))
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);

            frame.render_widget(para, cell_area);
        }
    }
}

fn render_status_bar_content(frame: &mut Frame, area: Rect, game: &ShardFusionGame) {
    if render_forfeit_status_bar(frame, area, game.forfeit_pending) {
        return;
    }

    let animating = game.anim_state != ShardFusionAnimState::Idle;
    let label = if animating { "Animating..." } else { "Your move" };

    render_status_bar(frame, area, label, Color::White, &[
        ("[Arrows]", "Slide"),
        ("[Esc]", "Forfeit"),
    ]);
}

fn render_info_panel(frame: &mut Frame, area: Rect, game: &ShardFusionGame) {
    let target = game.difficulty.target_value();
    let highest = game.highest_tile();

    let lines = vec![
        Line::from(vec![
            Span::raw("Score:   "),
            Span::styled(game.score.to_string(), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Target:  "),
            Span::styled(target.to_string(), Style::default().fg(Color::LightCyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Highest: "),
            Span::styled(highest.to_string(), Style::default().fg(tile_color(highest))),
        ]),
    ];

    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" Info "));
    frame.render_widget(para, area);
}
```

**Step 2: Register in `src/ui/mod.rs`**

Add the module declaration (alphabetically):
```rust
pub mod shard_fusion_scene;
```

Add the render dispatch in the `ActiveMinigame` match (after the `RunicShift` arm):
```rust
Some(ActiveMinigame::ShardFusion(game)) => {
    shard_fusion_scene::render_shard_fusion_scene(
        frame, area, game, show_dismiss_hint,
    );
}
```

**Step 3: Build**

```bash
cargo build 2>&1 | head -30
```

Fix any type/import errors.

**Step 4: Commit**

```bash
git add src/ui/shard_fusion_scene.rs src/ui/mod.rs
git commit -m "feat(shard-fusion): add UI scene with slide animation and merge flash"
```

---

### Task 7: Input Handler

**Files:**
- Modify: `src/input/minigame_input.rs`

**Step 1: Add the handler arm**

Find the `handle_minigame` function. Add imports at top of file:
```rust
use crate::challenges::shard_fusion::{process_input as process_shard_fusion_input, ShardFusionInput};
```

Add the arm to the `match` in `handle_minigame` (after the `RunicShift` arm):
```rust
ActiveMinigame::ShardFusion(game) => {
    if game.game_result.is_some() {
        crate::challenges::shard_fusion::apply_shard_fusion_result(state);
        return InputResult::NeedsSave;
    }
    let input = match key.code {
        KeyCode::Up => ShardFusionInput::Up,
        KeyCode::Down => ShardFusionInput::Down,
        KeyCode::Left => ShardFusionInput::Left,
        KeyCode::Right => ShardFusionInput::Right,
        KeyCode::Esc => ShardFusionInput::Forfeit,
        _ => ShardFusionInput::Other,
    };
    process_shard_fusion_input(game, input, rng);
    InputResult::Continue
}
```

Note: `apply_shard_fusion_result` is the function generated by `impl_apply_game_result!` in `shard_fusion/mod.rs`. Make sure it's exported from `shard_fusion/mod.rs` as a pub function.

**Step 2: Build and test**

```bash
cargo build && cargo test 2>&1 | tail -20
```

**Step 3: Commit**

```bash
git add src/input/minigame_input.rs
git commit -m "feat(shard-fusion): wire input handler"
```

---

### Task 8: Debug Menu

**Files:**
- Modify: `src/utils/debug_menu.rs`

**Step 1: Add debug trigger**

1. Add to `DebugAction` enum:
```rust
TriggerShardFusionChallenge,
```

2. Add to the debug options list (after `TriggerRunicShiftChallenge`):
```rust
DebugAction::TriggerShardFusionChallenge,
```

3. Add to the action handler match:
```rust
DebugAction::TriggerShardFusionChallenge => {
    use crate::challenges::{menu::ChallengeType, shard_fusion::start_shard_fusion_game};
    if state.challenge_menu.has_challenge(&ChallengeType::ShardFusion) {
        "Shard Fusion challenge already pending!"
    } else {
        state.challenge_menu.add_challenge(
            crate::challenges::menu::create_challenge(&ChallengeType::ShardFusion, 0),
        );
        "Shard Fusion challenge added!"
    }
}
```

4. Add the display label. Find where labels are defined (likely a `label()` method or a `DEBUG_LABELS` array):
```rust
DebugAction::TriggerShardFusionChallenge => "Trigger Shard Fusion Challenge",
```

**Step 2: Build**

```bash
cargo build 2>&1 | head -20
```

**Step 3: Commit**

```bash
git add src/utils/debug_menu.rs
git commit -m "feat(shard-fusion): add debug menu trigger"
```

---

### Task 9: Final Verification

**Step 1: Run full CI checks**

```bash
make check
```

Expected: format, clippy, tests, build, audit all pass.

**Step 2: Manual smoke test via debug menu**

```bash
cargo run
```

1. Start a character, wait for game screen
2. Open debug menu (check keybinding in `src/input/` — typically `d` or `F12`)
3. Trigger "Shard Fusion Challenge"
4. Accept the challenge at Novice difficulty
5. Verify:
   - 4×4 board renders with 2 initial tiles
   - Arrow keys slide tiles with animation
   - Merging tiles flash white briefly
   - Info panel shows score, target (512), highest tile
   - Reaching 512 shows win overlay
   - Esc → Esc forfeits correctly

**Step 3: Commit any final fixes**

```bash
git add -p  # stage only what changed
git commit -m "fix(shard-fusion): address clippy warnings and final polish"
```
