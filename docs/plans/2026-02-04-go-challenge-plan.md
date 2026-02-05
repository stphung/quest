# Go Challenge Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a 9x9 Go challenge with MCTS AI to Quest, matching the existing challenge pattern (Chess, Gomoku, etc.)

**Architecture:** New `src/challenges/go/` module with types, logic, and MCTS AI. UI in `src/ui/go_scene.rs`. Integrates with existing challenge menu system. Pure Rust, no external dependencies.

**Tech Stack:** Rust, Ratatui for UI, rand for MCTS randomness

---

## Task 1: Create Go Module Structure

**Files:**
- Create: `src/challenges/go/mod.rs`
- Create: `src/challenges/go/types.rs`
- Modify: `src/challenges/mod.rs`

**Step 1: Create the module file**

Create `src/challenges/go/mod.rs`:

```rust
//! Go (Territory Control) minigame.

#![allow(unused_imports)]

pub mod logic;
pub mod mcts;
pub mod types;

pub use logic::*;
pub use types::*;
```

**Step 2: Create empty placeholder files**

Create `src/challenges/go/types.rs`:

```rust
//! Go minigame data structures.
```

Create `src/challenges/go/logic.rs`:

```rust
//! Go game logic.
```

Create `src/challenges/go/mcts.rs`:

```rust
//! Monte Carlo Tree Search AI for Go.
```

**Step 3: Add go module to challenges**

Modify `src/challenges/mod.rs` - add after line 10 (`pub mod rune;`):

```rust
pub mod go;
```

**Step 4: Verify it compiles**

Run: `cargo build 2>&1 | tail -5`

Expected: Compiles (warnings about empty modules are OK)

**Step 5: Commit**

```bash
git add src/challenges/go/ src/challenges/mod.rs
git commit -m "feat(go): create module structure"
```

---

## Task 2: Implement Core Types (Stone, GoMove, GoDifficulty)

**Files:**
- Modify: `src/challenges/go/types.rs`
- Test: `src/challenges/go/types.rs` (inline tests)

**Step 1: Write tests for Stone enum**

Add to `src/challenges/go/types.rs`:

```rust
//! Go (Territory Control) minigame data structures.
//!
//! 9x9 board, players place stones to surround territory.

use serde::{Deserialize, Serialize};

/// Board size (9x9)
pub const BOARD_SIZE: usize = 9;

/// Stone color
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stone {
    Black,
    White,
}

impl Stone {
    pub fn opponent(&self) -> Self {
        match self {
            Stone::Black => Stone::White,
            Stone::White => Stone::Black,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stone_opponent() {
        assert_eq!(Stone::Black.opponent(), Stone::White);
        assert_eq!(Stone::White.opponent(), Stone::Black);
    }
}
```

**Step 2: Run test**

Run: `cargo test -p quest --lib stone_opponent -- --nocapture`

Expected: PASS

**Step 3: Add GoMove enum**

Add after `Stone` impl:

```rust
/// A move in Go
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoMove {
    Place(usize, usize),
    Pass,
}
```

**Step 4: Add GoDifficulty enum**

Add after `GoMove`:

```rust
/// AI difficulty levels (based on MCTS simulation count)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoDifficulty {
    Novice,     // 500 simulations
    Apprentice, // 2,000 simulations
    Journeyman, // 8,000 simulations
    Master,     // 20,000 simulations
}

impl GoDifficulty {
    pub const ALL: [GoDifficulty; 4] = [
        GoDifficulty::Novice,
        GoDifficulty::Apprentice,
        GoDifficulty::Journeyman,
        GoDifficulty::Master,
    ];

    pub fn from_index(index: usize) -> Self {
        Self::ALL
            .get(index)
            .copied()
            .unwrap_or(GoDifficulty::Novice)
    }

    pub fn simulation_count(&self) -> u32 {
        match self {
            Self::Novice => 500,
            Self::Apprentice => 2_000,
            Self::Journeyman => 8_000,
            Self::Master => 20_000,
        }
    }

    pub fn reward_prestige(&self) -> u32 {
        match self {
            Self::Novice => 1,
            Self::Apprentice => 2,
            Self::Journeyman => 3,
            Self::Master => 5,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Novice => "Novice",
            Self::Apprentice => "Apprentice",
            Self::Journeyman => "Journeyman",
            Self::Master => "Master",
        }
    }
}
```

**Step 5: Add tests for GoDifficulty**

Add to tests module:

```rust
    #[test]
    fn test_difficulty_from_index() {
        assert_eq!(GoDifficulty::from_index(0), GoDifficulty::Novice);
        assert_eq!(GoDifficulty::from_index(3), GoDifficulty::Master);
        assert_eq!(GoDifficulty::from_index(99), GoDifficulty::Novice);
    }

    #[test]
    fn test_difficulty_simulation_count() {
        assert_eq!(GoDifficulty::Novice.simulation_count(), 500);
        assert_eq!(GoDifficulty::Apprentice.simulation_count(), 2_000);
        assert_eq!(GoDifficulty::Journeyman.simulation_count(), 8_000);
        assert_eq!(GoDifficulty::Master.simulation_count(), 20_000);
    }

    #[test]
    fn test_difficulty_reward_prestige() {
        assert_eq!(GoDifficulty::Novice.reward_prestige(), 1);
        assert_eq!(GoDifficulty::Apprentice.reward_prestige(), 2);
        assert_eq!(GoDifficulty::Journeyman.reward_prestige(), 3);
        assert_eq!(GoDifficulty::Master.reward_prestige(), 5);
    }
```

**Step 6: Run all type tests**

Run: `cargo test -p quest --lib go::types -- --nocapture`

Expected: All PASS

**Step 7: Commit**

```bash
git add src/challenges/go/types.rs
git commit -m "feat(go): add Stone, GoMove, GoDifficulty types"
```

---

## Task 3: Implement GoResult and GoGame Struct

**Files:**
- Modify: `src/challenges/go/types.rs`

**Step 1: Add GoResult enum**

Add after `GoDifficulty`:

```rust
/// Result of a completed Go game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoResult {
    Win,
    Loss,
    Draw,
}
```

**Step 2: Add GoGame struct**

Add after `GoResult`:

```rust
/// Main Go game state
#[derive(Debug, Clone)]
pub struct GoGame {
    /// 9x9 board, None = empty intersection
    pub board: [[Option<Stone>; BOARD_SIZE]; BOARD_SIZE],
    /// Current player's turn
    pub current_player: Stone,
    /// Ko point - illegal to play here this turn (prevents infinite capture loops)
    pub ko_point: Option<(usize, usize)>,
    /// Stones captured by Black (White's prisoners)
    pub captured_by_black: u32,
    /// Stones captured by White (Black's prisoners)
    pub captured_by_white: u32,
    /// Count of consecutive passes (2 = game over)
    pub consecutive_passes: u8,
    /// Cursor position (row, col) for UI
    pub cursor: (usize, usize),
    /// Difficulty level
    pub difficulty: GoDifficulty,
    /// Game result (None if in progress)
    pub game_result: Option<GoResult>,
    /// Is AI currently thinking?
    pub ai_thinking: bool,
    /// Ticks spent thinking (for delayed AI move)
    pub ai_think_ticks: u32,
    /// Last move for highlighting
    pub last_move: Option<GoMove>,
    /// Forfeit confirmation pending
    pub forfeit_pending: bool,
}

impl GoGame {
    pub fn new(difficulty: GoDifficulty) -> Self {
        Self {
            board: [[None; BOARD_SIZE]; BOARD_SIZE],
            current_player: Stone::Black, // Black plays first in Go
            ko_point: None,
            captured_by_black: 0,
            captured_by_white: 0,
            consecutive_passes: 0,
            cursor: (BOARD_SIZE / 2, BOARD_SIZE / 2), // Center (4, 4)
            difficulty,
            game_result: None,
            ai_thinking: false,
            ai_think_ticks: 0,
            last_move: None,
            forfeit_pending: false,
        }
    }

    /// Move cursor in a direction
    pub fn move_cursor(&mut self, d_row: i32, d_col: i32) {
        let new_row = (self.cursor.0 as i32 + d_row).clamp(0, BOARD_SIZE as i32 - 1) as usize;
        let new_col = (self.cursor.1 as i32 + d_col).clamp(0, BOARD_SIZE as i32 - 1) as usize;
        self.cursor = (new_row, new_col);
    }

    /// Check if a position is empty
    pub fn is_empty(&self, row: usize, col: usize) -> bool {
        row < BOARD_SIZE && col < BOARD_SIZE && self.board[row][col].is_none()
    }

    /// Switch to the other player's turn
    pub fn switch_player(&mut self) {
        self.current_player = self.current_player.opponent();
    }
}
```

**Step 3: Add tests for GoGame**

Add to tests module:

```rust
    #[test]
    fn test_new_game() {
        let game = GoGame::new(GoDifficulty::Novice);
        assert_eq!(game.cursor, (4, 4)); // Center of 9x9
        assert_eq!(game.current_player, Stone::Black);
        assert!(game.game_result.is_none());
        assert_eq!(game.consecutive_passes, 0);
        assert!(game.ko_point.is_none());
    }

    #[test]
    fn test_move_cursor() {
        let mut game = GoGame::new(GoDifficulty::Novice);
        game.move_cursor(-1, 0); // Up
        assert_eq!(game.cursor, (3, 4));
        game.cursor = (0, 0);
        game.move_cursor(-1, -1); // Should clamp
        assert_eq!(game.cursor, (0, 0));
        game.cursor = (8, 8);
        game.move_cursor(1, 1); // Should clamp
        assert_eq!(game.cursor, (8, 8));
    }

    #[test]
    fn test_is_empty() {
        let mut game = GoGame::new(GoDifficulty::Novice);
        assert!(game.is_empty(4, 4));
        game.board[4][4] = Some(Stone::Black);
        assert!(!game.is_empty(4, 4));
    }

    #[test]
    fn test_switch_player() {
        let mut game = GoGame::new(GoDifficulty::Novice);
        assert_eq!(game.current_player, Stone::Black);
        game.switch_player();
        assert_eq!(game.current_player, Stone::White);
        game.switch_player();
        assert_eq!(game.current_player, Stone::Black);
    }
```

**Step 4: Run tests**

Run: `cargo test -p quest --lib go::types -- --nocapture`

Expected: All PASS

**Step 5: Commit**

```bash
git add src/challenges/go/types.rs
git commit -m "feat(go): add GoResult and GoGame struct"
```

---

## Task 4: Implement Liberty Counting and Group Detection

**Files:**
- Modify: `src/challenges/go/logic.rs`

**Step 1: Write test for liberty counting**

Replace `src/challenges/go/logic.rs`:

```rust
//! Go game logic: placement, capture, ko, scoring.

use super::types::{GoGame, GoMove, GoResult, Stone, BOARD_SIZE};
use std::collections::HashSet;

/// Get all stones in the same group as the stone at (row, col).
/// Returns empty set if position is empty.
pub fn get_group(
    board: &[[Option<Stone>; BOARD_SIZE]; BOARD_SIZE],
    row: usize,
    col: usize,
) -> HashSet<(usize, usize)> {
    let mut group = HashSet::new();
    let Some(stone) = board[row][col] else {
        return group;
    };

    let mut stack = vec![(row, col)];
    while let Some((r, c)) = stack.pop() {
        if group.contains(&(r, c)) {
            continue;
        }
        if board[r][c] == Some(stone) {
            group.insert((r, c));
            // Add adjacent positions
            for (dr, dc) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let nr = r as i32 + dr;
                let nc = c as i32 + dc;
                if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
                    stack.push((nr as usize, nc as usize));
                }
            }
        }
    }
    group
}

/// Count liberties (empty adjacent points) of a group.
pub fn count_liberties(
    board: &[[Option<Stone>; BOARD_SIZE]; BOARD_SIZE],
    group: &HashSet<(usize, usize)>,
) -> usize {
    let mut liberties = HashSet::new();
    for &(row, col) in group {
        for (dr, dc) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let nr = row as i32 + dr;
            let nc = col as i32 + dc;
            if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
                let nr = nr as usize;
                let nc = nc as usize;
                if board[nr][nc].is_none() {
                    liberties.insert((nr, nc));
                }
            }
        }
    }
    liberties.len()
}

/// Get liberties count for the group containing the stone at (row, col).
pub fn get_liberties_at(
    board: &[[Option<Stone>; BOARD_SIZE]; BOARD_SIZE],
    row: usize,
    col: usize,
) -> usize {
    let group = get_group(board, row, col);
    count_liberties(board, &group)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn place(board: &mut [[Option<Stone>; BOARD_SIZE]; BOARD_SIZE], row: usize, col: usize, stone: Stone) {
        board[row][col] = Some(stone);
    }

    #[test]
    fn test_single_stone_liberties() {
        let mut board = [[None; BOARD_SIZE]; BOARD_SIZE];
        place(&mut board, 4, 4, Stone::Black);
        // Center stone has 4 liberties
        assert_eq!(get_liberties_at(&board, 4, 4), 4);
    }

    #[test]
    fn test_corner_stone_liberties() {
        let mut board = [[None; BOARD_SIZE]; BOARD_SIZE];
        place(&mut board, 0, 0, Stone::Black);
        // Corner stone has 2 liberties
        assert_eq!(get_liberties_at(&board, 0, 0), 2);
    }

    #[test]
    fn test_edge_stone_liberties() {
        let mut board = [[None; BOARD_SIZE]; BOARD_SIZE];
        place(&mut board, 0, 4, Stone::Black);
        // Edge stone has 3 liberties
        assert_eq!(get_liberties_at(&board, 0, 4), 3);
    }

    #[test]
    fn test_group_liberties() {
        let mut board = [[None; BOARD_SIZE]; BOARD_SIZE];
        // Two connected stones share liberties
        place(&mut board, 4, 4, Stone::Black);
        place(&mut board, 4, 5, Stone::Black);
        // Group has 6 liberties (shared liberty counted once)
        assert_eq!(get_liberties_at(&board, 4, 4), 6);
        assert_eq!(get_liberties_at(&board, 4, 5), 6);
    }

    #[test]
    fn test_surrounded_stone() {
        let mut board = [[None; BOARD_SIZE]; BOARD_SIZE];
        place(&mut board, 4, 4, Stone::Black);
        place(&mut board, 3, 4, Stone::White);
        place(&mut board, 5, 4, Stone::White);
        place(&mut board, 4, 3, Stone::White);
        place(&mut board, 4, 5, Stone::White);
        // Surrounded stone has 0 liberties
        assert_eq!(get_liberties_at(&board, 4, 4), 0);
    }

    #[test]
    fn test_get_group() {
        let mut board = [[None; BOARD_SIZE]; BOARD_SIZE];
        place(&mut board, 4, 4, Stone::Black);
        place(&mut board, 4, 5, Stone::Black);
        place(&mut board, 4, 6, Stone::Black);

        let group = get_group(&board, 4, 4);
        assert_eq!(group.len(), 3);
        assert!(group.contains(&(4, 4)));
        assert!(group.contains(&(4, 5)));
        assert!(group.contains(&(4, 6)));
    }

    #[test]
    fn test_empty_position_group() {
        let board = [[None; BOARD_SIZE]; BOARD_SIZE];
        let group = get_group(&board, 4, 4);
        assert!(group.is_empty());
    }
}
```

**Step 2: Run tests**

Run: `cargo test -p quest --lib go::logic -- --nocapture`

Expected: All PASS

**Step 3: Commit**

```bash
git add src/challenges/go/logic.rs
git commit -m "feat(go): implement liberty counting and group detection"
```

---

## Task 5: Implement Capture Logic

**Files:**
- Modify: `src/challenges/go/logic.rs`

**Step 1: Add capture function**

Add after `get_liberties_at` function:

```rust
/// Remove a group from the board and return the number of stones captured.
fn remove_group(
    board: &mut [[Option<Stone>; BOARD_SIZE]; BOARD_SIZE],
    group: &HashSet<(usize, usize)>,
) -> u32 {
    let count = group.len() as u32;
    for &(row, col) in group {
        board[row][col] = None;
    }
    count
}

/// Check and remove any opponent groups with zero liberties adjacent to (row, col).
/// Returns the total number of stones captured.
pub fn capture_dead_groups(
    board: &mut [[Option<Stone>; BOARD_SIZE]; BOARD_SIZE],
    row: usize,
    col: usize,
    capturing_player: Stone,
) -> u32 {
    let opponent = capturing_player.opponent();
    let mut captured = 0;
    let mut checked = HashSet::new();

    // Check all adjacent positions for opponent groups
    for (dr, dc) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
        let nr = row as i32 + dr;
        let nc = col as i32 + dc;
        if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
            let nr = nr as usize;
            let nc = nc as usize;

            if checked.contains(&(nr, nc)) {
                continue;
            }

            if board[nr][nc] == Some(opponent) {
                let group = get_group(board, nr, nc);
                for &pos in &group {
                    checked.insert(pos);
                }
                if count_liberties(board, &group) == 0 {
                    captured += remove_group(board, &group);
                }
            }
        }
    }
    captured
}
```

**Step 2: Add capture tests**

Add to tests module:

```rust
    #[test]
    fn test_capture_single_stone() {
        let mut board = [[None; BOARD_SIZE]; BOARD_SIZE];
        // Set up a capture scenario
        place(&mut board, 4, 4, Stone::White);
        place(&mut board, 3, 4, Stone::Black);
        place(&mut board, 5, 4, Stone::Black);
        place(&mut board, 4, 3, Stone::Black);
        // White has 1 liberty at (4, 5)
        assert_eq!(get_liberties_at(&board, 4, 4), 1);

        // Black plays at (4, 5) to capture
        place(&mut board, 4, 5, Stone::Black);
        let captured = capture_dead_groups(&mut board, 4, 5, Stone::Black);

        assert_eq!(captured, 1);
        assert!(board[4][4].is_none()); // White stone removed
    }

    #[test]
    fn test_capture_group() {
        let mut board = [[None; BOARD_SIZE]; BOARD_SIZE];
        // Two white stones
        place(&mut board, 4, 4, Stone::White);
        place(&mut board, 4, 5, Stone::White);
        // Surround with black
        place(&mut board, 3, 4, Stone::Black);
        place(&mut board, 3, 5, Stone::Black);
        place(&mut board, 5, 4, Stone::Black);
        place(&mut board, 5, 5, Stone::Black);
        place(&mut board, 4, 3, Stone::Black);
        // One liberty left at (4, 6)

        place(&mut board, 4, 6, Stone::Black);
        let captured = capture_dead_groups(&mut board, 4, 6, Stone::Black);

        assert_eq!(captured, 2);
        assert!(board[4][4].is_none());
        assert!(board[4][5].is_none());
    }

    #[test]
    fn test_no_capture_with_liberties() {
        let mut board = [[None; BOARD_SIZE]; BOARD_SIZE];
        place(&mut board, 4, 4, Stone::White);
        place(&mut board, 3, 4, Stone::Black);
        place(&mut board, 5, 4, Stone::Black);
        place(&mut board, 4, 3, Stone::Black);
        // White still has liberty at (4, 5) - not surrounded

        // Black plays elsewhere
        place(&mut board, 0, 0, Stone::Black);
        let captured = capture_dead_groups(&mut board, 0, 0, Stone::Black);

        assert_eq!(captured, 0);
        assert!(board[4][4].is_some()); // White stone still there
    }
```

**Step 3: Run tests**

Run: `cargo test -p quest --lib go::logic -- --nocapture`

Expected: All PASS

**Step 4: Commit**

```bash
git add src/challenges/go/logic.rs
git commit -m "feat(go): implement capture logic"
```

---

## Task 6: Implement Move Validation (Ko Rule, Suicide Rule)

**Files:**
- Modify: `src/challenges/go/logic.rs`

**Step 1: Add move validation function**

Add after `capture_dead_groups`:

```rust
/// Check if a move is legal (ignoring ko for now).
/// A move is illegal if:
/// 1. Position is occupied
/// 2. Position is the ko point
/// 3. Move would be suicide (no liberties after placement, and no captures)
pub fn is_legal_move(game: &GoGame, row: usize, col: usize) -> bool {
    // Position must be empty
    if !game.is_empty(row, col) {
        return false;
    }

    // Cannot play at ko point
    if game.ko_point == Some((row, col)) {
        return false;
    }

    // Check for suicide
    // Temporarily place the stone
    let mut test_board = game.board;
    test_board[row][col] = Some(game.current_player);

    // First check if this move captures anything
    let opponent = game.current_player.opponent();
    let mut would_capture = false;
    for (dr, dc) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
        let nr = row as i32 + dr;
        let nc = col as i32 + dc;
        if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
            let nr = nr as usize;
            let nc = nc as usize;
            if test_board[nr][nc] == Some(opponent) {
                let group = get_group(&test_board, nr, nc);
                if count_liberties(&test_board, &group) == 0 {
                    would_capture = true;
                    break;
                }
            }
        }
    }

    // If we capture, move is legal (not suicide)
    if would_capture {
        return true;
    }

    // Check if our group would have liberties
    let our_group = get_group(&test_board, row, col);
    count_liberties(&test_board, &our_group) > 0
}

/// Get all legal moves for the current player.
pub fn get_legal_moves(game: &GoGame) -> Vec<GoMove> {
    let mut moves = Vec::new();
    for row in 0..BOARD_SIZE {
        for col in 0..BOARD_SIZE {
            if is_legal_move(game, row, col) {
                moves.push(GoMove::Place(row, col));
            }
        }
    }
    moves.push(GoMove::Pass); // Pass is always legal
    moves
}
```

**Step 2: Add validation tests**

Add to tests module:

```rust
    #[test]
    fn test_legal_move_empty_board() {
        let game = GoGame::new(GoDifficulty::Novice);
        assert!(is_legal_move(&game, 4, 4));
        assert!(is_legal_move(&game, 0, 0));
    }

    #[test]
    fn test_illegal_move_occupied() {
        let mut game = GoGame::new(GoDifficulty::Novice);
        game.board[4][4] = Some(Stone::Black);
        assert!(!is_legal_move(&game, 4, 4));
    }

    #[test]
    fn test_illegal_move_ko() {
        let mut game = GoGame::new(GoDifficulty::Novice);
        game.ko_point = Some((4, 4));
        assert!(!is_legal_move(&game, 4, 4));
    }

    #[test]
    fn test_suicide_illegal() {
        let mut game = GoGame::new(GoDifficulty::Novice);
        // Create a surrounded empty point
        game.board[3][4] = Some(Stone::White);
        game.board[5][4] = Some(Stone::White);
        game.board[4][3] = Some(Stone::White);
        game.board[4][5] = Some(Stone::White);
        game.current_player = Stone::Black;
        // Black playing at (4,4) would be suicide
        assert!(!is_legal_move(&game, 4, 4));
    }

    #[test]
    fn test_capture_not_suicide() {
        let mut game = GoGame::new(GoDifficulty::Novice);
        // White stone at center
        game.board[4][4] = Some(Stone::White);
        // Black almost surrounds
        game.board[3][4] = Some(Stone::Black);
        game.board[5][4] = Some(Stone::Black);
        game.board[4][3] = Some(Stone::Black);
        game.current_player = Stone::Black;
        // Black playing at (4,5) captures white, so it's legal even though
        // black would have no liberties without the capture
        assert!(is_legal_move(&game, 4, 5));
    }

    #[test]
    fn test_get_legal_moves_includes_pass() {
        let game = GoGame::new(GoDifficulty::Novice);
        let moves = get_legal_moves(&game);
        assert!(moves.contains(&GoMove::Pass));
    }

    #[test]
    fn test_get_legal_moves_empty_board() {
        let game = GoGame::new(GoDifficulty::Novice);
        let moves = get_legal_moves(&game);
        // 81 board positions + 1 pass
        assert_eq!(moves.len(), 82);
    }
```

**Step 3: Run tests**

Run: `cargo test -p quest --lib go::logic -- --nocapture`

Expected: All PASS

**Step 4: Commit**

```bash
git add src/challenges/go/logic.rs
git commit -m "feat(go): implement move validation with ko and suicide rules"
```

---

## Task 7: Implement Move Execution

**Files:**
- Modify: `src/challenges/go/logic.rs`

**Step 1: Add make_move function**

Add after `get_legal_moves`:

```rust
/// Execute a move on the game state.
/// Returns true if the move was successful.
pub fn make_move(game: &mut GoGame, mv: GoMove) -> bool {
    match mv {
        GoMove::Pass => {
            game.consecutive_passes += 1;
            game.ko_point = None;
            game.last_move = Some(GoMove::Pass);
            game.switch_player();

            // Check for game end (two consecutive passes)
            if game.consecutive_passes >= 2 {
                end_game_by_scoring(game);
            }
            true
        }
        GoMove::Place(row, col) => {
            if !is_legal_move(game, row, col) {
                return false;
            }

            // Place the stone
            game.board[row][col] = Some(game.current_player);
            game.consecutive_passes = 0;

            // Capture any dead opponent groups
            let captured = capture_dead_groups(&mut game.board, row, col, game.current_player);

            // Update capture counts
            match game.current_player {
                Stone::Black => game.captured_by_black += captured,
                Stone::White => game.captured_by_white += captured,
            }

            // Set ko point if exactly one stone was captured
            game.ko_point = if captured == 1 {
                // Find where the captured stone was (it's now empty and adjacent)
                let mut ko = None;
                for (dr, dc) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let nr = row as i32 + dr;
                    let nc = col as i32 + dc;
                    if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
                        let nr = nr as usize;
                        let nc = nc as usize;
                        if game.board[nr][nc].is_none() {
                            // Verify this was where the capture happened by checking
                            // if replaying there would recapture our stone
                            let mut test_board = game.board;
                            test_board[nr][nc] = Some(game.current_player.opponent());
                            let test_group = get_group(&test_board, row, col);
                            if count_liberties(&test_board, &test_group) == 0 {
                                ko = Some((nr, nc));
                                break;
                            }
                        }
                    }
                }
                ko
            } else {
                None
            };

            game.last_move = Some(GoMove::Place(row, col));
            game.switch_player();
            true
        }
    }
}

/// End the game and calculate scores using Chinese rules.
fn end_game_by_scoring(game: &mut GoGame) {
    let (black_score, white_score) = calculate_score(&game.board);

    // Determine winner (Black plays as human)
    game.game_result = Some(if black_score > white_score {
        GoResult::Win
    } else if white_score > black_score {
        GoResult::Loss
    } else {
        GoResult::Draw
    });
}

/// Calculate scores using Chinese rules (stones + territory).
pub fn calculate_score(board: &[[Option<Stone>; BOARD_SIZE]; BOARD_SIZE]) -> (i32, i32) {
    let mut black_score = 0i32;
    let mut white_score = 0i32;
    let mut counted = [[false; BOARD_SIZE]; BOARD_SIZE];

    // Count stones
    for row in 0..BOARD_SIZE {
        for col in 0..BOARD_SIZE {
            match board[row][col] {
                Some(Stone::Black) => black_score += 1,
                Some(Stone::White) => white_score += 1,
                None => {}
            }
        }
    }

    // Count territory (empty regions completely surrounded by one color)
    for row in 0..BOARD_SIZE {
        for col in 0..BOARD_SIZE {
            if board[row][col].is_none() && !counted[row][col] {
                let (region, owner) = get_empty_region(board, row, col);
                for &(r, c) in &region {
                    counted[r][c] = true;
                }
                match owner {
                    Some(Stone::Black) => black_score += region.len() as i32,
                    Some(Stone::White) => white_score += region.len() as i32,
                    None => {} // Contested - no points
                }
            }
        }
    }

    // Apply komi (6.5 points to White for going second)
    // We use integer math, so 6 points (simplified)
    white_score += 6;

    (black_score, white_score)
}

/// Get an empty region and determine its owner (if surrounded by one color).
fn get_empty_region(
    board: &[[Option<Stone>; BOARD_SIZE]; BOARD_SIZE],
    start_row: usize,
    start_col: usize,
) -> (HashSet<(usize, usize)>, Option<Stone>) {
    let mut region = HashSet::new();
    let mut stack = vec![(start_row, start_col)];
    let mut borders_black = false;
    let mut borders_white = false;

    while let Some((row, col)) = stack.pop() {
        if region.contains(&(row, col)) {
            continue;
        }

        match board[row][col] {
            None => {
                region.insert((row, col));
                for (dr, dc) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let nr = row as i32 + dr;
                    let nc = col as i32 + dc;
                    if nr >= 0 && nr < BOARD_SIZE as i32 && nc >= 0 && nc < BOARD_SIZE as i32 {
                        stack.push((nr as usize, nc as usize));
                    }
                }
            }
            Some(Stone::Black) => borders_black = true,
            Some(Stone::White) => borders_white = true,
        }
    }

    let owner = match (borders_black, borders_white) {
        (true, false) => Some(Stone::Black),
        (false, true) => Some(Stone::White),
        _ => None, // Contested or touches both
    };

    (region, owner)
}
```

**Step 2: Add move execution tests**

Add to tests module:

```rust
    #[test]
    fn test_make_move_place() {
        let mut game = GoGame::new(GoDifficulty::Novice);
        assert!(make_move(&mut game, GoMove::Place(4, 4)));
        assert_eq!(game.board[4][4], Some(Stone::Black));
        assert_eq!(game.current_player, Stone::White);
        assert_eq!(game.consecutive_passes, 0);
    }

    #[test]
    fn test_make_move_pass() {
        let mut game = GoGame::new(GoDifficulty::Novice);
        assert!(make_move(&mut game, GoMove::Pass));
        assert_eq!(game.current_player, Stone::White);
        assert_eq!(game.consecutive_passes, 1);
    }

    #[test]
    fn test_two_passes_end_game() {
        let mut game = GoGame::new(GoDifficulty::Novice);
        make_move(&mut game, GoMove::Pass);
        make_move(&mut game, GoMove::Pass);
        assert!(game.game_result.is_some());
    }

    #[test]
    fn test_capture_updates_count() {
        let mut game = GoGame::new(GoDifficulty::Novice);
        // Set up capture
        game.board[4][4] = Some(Stone::White);
        game.board[3][4] = Some(Stone::Black);
        game.board[5][4] = Some(Stone::Black);
        game.board[4][3] = Some(Stone::Black);

        make_move(&mut game, GoMove::Place(4, 5)); // Black captures
        assert_eq!(game.captured_by_black, 1);
        assert!(game.board[4][4].is_none());
    }

    #[test]
    fn test_ko_point_set() {
        let mut game = GoGame::new(GoDifficulty::Novice);
        // Classic ko shape
        // . B W .
        // B . B W
        // . B W .
        game.board[0][1] = Some(Stone::Black);
        game.board[0][2] = Some(Stone::White);
        game.board[1][0] = Some(Stone::Black);
        game.board[1][2] = Some(Stone::Black);
        game.board[1][3] = Some(Stone::White);
        game.board[2][1] = Some(Stone::Black);
        game.board[2][2] = Some(Stone::White);
        game.current_player = Stone::White;

        // White captures at (1,1)
        make_move(&mut game, GoMove::Place(1, 1));

        // Ko point should be set
        assert!(game.ko_point.is_some());
    }

    #[test]
    fn test_calculate_score_empty_board() {
        let board = [[None; BOARD_SIZE]; BOARD_SIZE];
        let (black, white) = calculate_score(&board);
        // Empty board = 0 + 0 stones, all territory contested, white gets 6 komi
        assert_eq!(black, 0);
        assert_eq!(white, 6);
    }

    #[test]
    fn test_calculate_score_with_territory() {
        let mut board = [[None; BOARD_SIZE]; BOARD_SIZE];
        // Black owns top-left corner (3x3 = 9 points)
        for i in 0..3 {
            board[3][i] = Some(Stone::Black);
            board[i][3] = Some(Stone::Black);
        }
        board[3][3] = Some(Stone::Black); // Corner of the wall

        let (black, white) = calculate_score(&board);
        // Black: 7 stones + 9 territory = 16
        // White: 0 stones + 0 territory + 6 komi = 6
        assert_eq!(black, 16);
        assert_eq!(white, 6);
    }
```

**Step 3: Run tests**

Run: `cargo test -p quest --lib go::logic -- --nocapture`

Expected: All PASS

**Step 4: Commit**

```bash
git add src/challenges/go/logic.rs
git commit -m "feat(go): implement move execution with scoring"
```

---

## Task 8: Implement MCTS Core Algorithm

**Files:**
- Modify: `src/challenges/go/mcts.rs`

**Step 1: Implement MCTS structure and algorithm**

Replace `src/challenges/go/mcts.rs`:

```rust
//! Monte Carlo Tree Search AI for Go.

use super::logic::{get_legal_moves, is_legal_move, make_move};
use super::types::{GoDifficulty, GoGame, GoMove, GoResult, Stone, BOARD_SIZE};
use rand::seq::SliceRandom;
use rand::Rng;

/// UCT exploration constant
const UCT_C: f64 = 1.4;

/// MCTS tree node
struct MctsNode {
    /// Move that led to this node
    move_taken: Option<GoMove>,
    /// Parent node index
    parent: Option<usize>,
    /// Child node indices
    children: Vec<usize>,
    /// Number of visits
    visits: u32,
    /// Number of wins (from perspective of player who made the move)
    wins: f32,
    /// Moves not yet expanded
    untried_moves: Vec<GoMove>,
    /// Player who just moved (to reach this state)
    player_just_moved: Stone,
}

impl MctsNode {
    fn new(parent: Option<usize>, move_taken: Option<GoMove>, player_just_moved: Stone, legal_moves: Vec<GoMove>) -> Self {
        Self {
            move_taken,
            parent,
            children: Vec::new(),
            visits: 0,
            wins: 0.0,
            untried_moves: legal_moves,
            player_just_moved,
        }
    }

    fn uct_value(&self, parent_visits: u32) -> f64 {
        if self.visits == 0 {
            return f64::INFINITY;
        }
        let exploitation = self.wins as f64 / self.visits as f64;
        let exploration = UCT_C * ((parent_visits as f64).ln() / self.visits as f64).sqrt();
        exploitation + exploration
    }
}

/// Run MCTS and return the best move.
pub fn mcts_best_move<R: Rng>(game: &GoGame, rng: &mut R) -> GoMove {
    let simulations = game.difficulty.simulation_count();
    let mut nodes: Vec<MctsNode> = Vec::with_capacity(simulations as usize);

    // Create root node
    let legal_moves = get_legal_moves(game);
    let root = MctsNode::new(
        None,
        None,
        game.current_player.opponent(), // Opponent "just moved" to create current state
        legal_moves,
    );
    nodes.push(root);

    for _ in 0..simulations {
        let mut game_clone = game.clone();

        // Selection: traverse tree using UCT
        let mut node_idx = 0;
        while nodes[node_idx].untried_moves.is_empty() && !nodes[node_idx].children.is_empty() {
            node_idx = select_child(&nodes, node_idx);
            if let Some(mv) = nodes[node_idx].move_taken {
                make_move(&mut game_clone, mv);
            }
        }

        // Expansion: add a new child if possible
        if !nodes[node_idx].untried_moves.is_empty() && game_clone.game_result.is_none() {
            let mv_idx = rng.gen_range(0..nodes[node_idx].untried_moves.len());
            let mv = nodes[node_idx].untried_moves.swap_remove(mv_idx);

            let current_player = game_clone.current_player;
            make_move(&mut game_clone, mv);

            let child_legal_moves = get_legal_moves(&game_clone);
            let child = MctsNode::new(
                Some(node_idx),
                Some(mv),
                current_player,
                child_legal_moves,
            );
            let child_idx = nodes.len();
            nodes.push(child);
            nodes[node_idx].children.push(child_idx);
            node_idx = child_idx;
        }

        // Simulation: random playout
        let winner = simulate_random_game(&mut game_clone, rng);

        // Backpropagation: update statistics
        backpropagate(&mut nodes, node_idx, winner);
    }

    // Select best move (most visits)
    select_best_move(&nodes)
}

/// Select child with highest UCT value.
fn select_child(nodes: &[MctsNode], parent_idx: usize) -> usize {
    let parent_visits = nodes[parent_idx].visits;
    nodes[parent_idx]
        .children
        .iter()
        .max_by(|&&a, &&b| {
            nodes[a]
                .uct_value(parent_visits)
                .partial_cmp(&nodes[b].uct_value(parent_visits))
                .unwrap()
        })
        .copied()
        .unwrap_or(parent_idx)
}

/// Simulate a random game to completion.
fn simulate_random_game<R: Rng>(game: &mut GoGame, rng: &mut R) -> Option<Stone> {
    let mut moves_made = 0;
    const MAX_MOVES: u32 = 200; // Prevent infinite games

    while game.game_result.is_none() && moves_made < MAX_MOVES {
        let legal_moves = get_legal_moves(game);
        if legal_moves.is_empty() {
            break;
        }

        // Prefer non-pass moves during simulation
        let non_pass: Vec<_> = legal_moves.iter().filter(|m| **m != GoMove::Pass).copied().collect();
        let mv = if !non_pass.is_empty() && rng.gen::<f32>() > 0.1 {
            *non_pass.choose(rng).unwrap()
        } else {
            *legal_moves.choose(rng).unwrap()
        };

        make_move(game, mv);
        moves_made += 1;
    }

    // Determine winner
    match game.game_result {
        Some(GoResult::Win) => Some(Stone::Black),   // Human is Black
        Some(GoResult::Loss) => Some(Stone::White),  // AI is White
        Some(GoResult::Draw) => None,
        None => {
            // Game didn't end naturally, use current score
            let (black, white) = super::logic::calculate_score(&game.board);
            if black > white {
                Some(Stone::Black)
            } else if white > black {
                Some(Stone::White)
            } else {
                None
            }
        }
    }
}

/// Backpropagate result through the tree.
fn backpropagate(nodes: &mut [MctsNode], start_idx: usize, winner: Option<Stone>) {
    let mut node_idx = Some(start_idx);

    while let Some(idx) = node_idx {
        nodes[idx].visits += 1;

        // Add win if this node's player matches the winner
        if let Some(w) = winner {
            if nodes[idx].player_just_moved == w {
                nodes[idx].wins += 1.0;
            }
        } else {
            // Draw - half point
            nodes[idx].wins += 0.5;
        }

        node_idx = nodes[idx].parent;
    }
}

/// Select the best move (most visited child of root).
fn select_best_move(nodes: &[MctsNode]) -> GoMove {
    nodes[0]
        .children
        .iter()
        .max_by_key(|&&idx| nodes[idx].visits)
        .and_then(|&idx| nodes[idx].move_taken)
        .unwrap_or(GoMove::Pass)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcts_returns_move() {
        let game = GoGame::new(GoDifficulty::Novice);
        let mut rng = rand::thread_rng();
        let mv = mcts_best_move(&game, &mut rng);
        // Should return some move (likely a placement, not pass on empty board)
        match mv {
            GoMove::Place(r, c) => {
                assert!(r < BOARD_SIZE);
                assert!(c < BOARD_SIZE);
            }
            GoMove::Pass => {} // Also valid
        }
    }

    #[test]
    fn test_mcts_avoids_obvious_suicide() {
        let mut game = GoGame::new(GoDifficulty::Novice);
        // Create a situation where (4,4) would be suicide for White
        game.board[3][4] = Some(Stone::Black);
        game.board[5][4] = Some(Stone::Black);
        game.board[4][3] = Some(Stone::Black);
        game.board[4][5] = Some(Stone::Black);
        game.current_player = Stone::White;

        let mut rng = rand::thread_rng();
        let mv = mcts_best_move(&game, &mut rng);

        // Should not play at (4,4) - it's suicide
        assert_ne!(mv, GoMove::Place(4, 4));
    }

    #[test]
    fn test_uct_value() {
        let node = MctsNode {
            move_taken: Some(GoMove::Place(4, 4)),
            parent: Some(0),
            children: vec![],
            visits: 10,
            wins: 5.0,
            untried_moves: vec![],
            player_just_moved: Stone::Black,
        };

        let uct = node.uct_value(100);
        // Should be roughly 0.5 (exploitation) + exploration bonus
        assert!(uct > 0.5);
        assert!(uct < 2.0);
    }

    #[test]
    fn test_unvisited_node_has_infinite_uct() {
        let node = MctsNode {
            move_taken: Some(GoMove::Place(4, 4)),
            parent: Some(0),
            children: vec![],
            visits: 0,
            wins: 0.0,
            untried_moves: vec![],
            player_just_moved: Stone::Black,
        };

        assert!(node.uct_value(100).is_infinite());
    }
}
```

**Step 2: Run tests**

Run: `cargo test -p quest --lib go::mcts -- --nocapture`

Expected: All PASS

**Step 3: Commit**

```bash
git add src/challenges/go/mcts.rs
git commit -m "feat(go): implement MCTS AI algorithm"
```

---

## Task 9: Add Game Control Functions and Exports

**Files:**
- Modify: `src/challenges/go/logic.rs`
- Modify: `src/challenges/go/mod.rs`
- Modify: `src/challenges/mod.rs`

**Step 1: Add game control functions to logic.rs**

Add at the end of `src/challenges/go/logic.rs` (before tests module):

```rust
use crate::core::game_state::GameState;
use super::mcts::mcts_best_move;

/// Start a new Go game with the selected difficulty.
pub fn start_go_game(state: &mut GameState, difficulty: GoDifficulty) {
    state.active_go = Some(GoGame::new(difficulty));
    state.challenge_menu.close();
}

/// Process human move at cursor position.
pub fn process_human_move(game: &mut GoGame) -> bool {
    if game.game_result.is_some() || game.current_player != Stone::Black {
        return false;
    }

    let (row, col) = game.cursor;
    if !is_legal_move(game, row, col) {
        return false;
    }

    make_move(game, GoMove::Place(row, col));

    // Check for game end
    if game.game_result.is_some() {
        return true;
    }

    // Start AI thinking
    game.ai_thinking = true;
    game.ai_think_ticks = 0;
    true
}

/// Process human pass.
pub fn process_human_pass(game: &mut GoGame) -> bool {
    if game.game_result.is_some() || game.current_player != Stone::Black {
        return false;
    }

    make_move(game, GoMove::Pass);

    // Check for game end
    if game.game_result.is_some() {
        return true;
    }

    // Start AI thinking
    game.ai_thinking = true;
    game.ai_think_ticks = 0;
    true
}

/// Process AI turn (called each tick while ai_thinking is true).
pub fn process_go_ai<R: rand::Rng>(game: &mut GoGame, rng: &mut R) {
    if !game.ai_thinking || game.game_result.is_some() {
        return;
    }

    // Simulate thinking delay (5-15 ticks = 0.5-1.5 seconds)
    game.ai_think_ticks += 1;
    let min_ticks = match game.difficulty {
        GoDifficulty::Novice => 5,
        GoDifficulty::Apprentice => 8,
        GoDifficulty::Journeyman => 10,
        GoDifficulty::Master => 15,
    };

    if game.ai_think_ticks < min_ticks {
        return;
    }

    // Get AI move using MCTS
    let ai_move = mcts_best_move(game, rng);
    make_move(game, ai_move);
    game.ai_thinking = false;
}
```

**Step 2: Update mod.rs exports**

Replace `src/challenges/go/mod.rs`:

```rust
//! Go (Territory Control) minigame.

#![allow(unused_imports)]

pub mod logic;
pub mod mcts;
pub mod types;

pub use logic::{
    calculate_score, get_legal_moves, is_legal_move, make_move,
    process_go_ai, process_human_move, process_human_pass, start_go_game,
};
pub use types::*;
```

**Step 3: Update challenges/mod.rs exports**

Modify `src/challenges/mod.rs` - add to exports after line 19:

```rust
pub use go::{GoDifficulty, GoGame, GoMove, GoResult, Stone, BOARD_SIZE as GO_BOARD_SIZE};
```

**Step 4: Verify it compiles**

Run: `cargo build 2>&1 | tail -10`

Expected: Will fail - GameState doesn't have active_go field yet (we'll add it in Task 11)

**Step 5: Comment out GameState references temporarily**

In `src/challenges/go/logic.rs`, comment out the `start_go_game` function body temporarily:

```rust
/// Start a new Go game with the selected difficulty.
pub fn start_go_game(_state: &mut GameState, difficulty: GoDifficulty) {
    // TODO: Uncomment when active_go is added to GameState
    // state.active_go = Some(GoGame::new(difficulty));
    // state.challenge_menu.close();
    let _ = difficulty; // Suppress unused warning
}
```

Also add `#[allow(unused_imports)]` at the top and comment out the GameState import:

```rust
#[allow(unused_imports)]
use crate::core::game_state::GameState;
```

**Step 6: Verify it compiles now**

Run: `cargo build 2>&1 | tail -5`

Expected: Compiles

**Step 7: Run all Go tests**

Run: `cargo test -p quest --lib go:: -- --nocapture`

Expected: All PASS

**Step 8: Commit**

```bash
git add src/challenges/go/
git commit -m "feat(go): add game control functions and exports"
```

---

## Task 10: Implement DifficultyInfo Trait for Go

**Files:**
- Modify: `src/challenges/menu.rs`

**Step 1: Add Go import**

At top of `src/challenges/menu.rs`, add import:

```rust
use super::go::GoDifficulty;
```

**Step 2: Implement DifficultyInfo for GoDifficulty**

Add after `impl DifficultyInfo for RuneDifficulty` block (around line 205):

```rust
impl DifficultyInfo for GoDifficulty {
    fn name(&self) -> &'static str {
        GoDifficulty::name(self)
    }

    fn reward(&self) -> ChallengeReward {
        ChallengeReward {
            prestige_ranks: self.reward_prestige(),
            ..Default::default()
        }
    }

    fn extra_info(&self) -> Option<String> {
        Some(format!("{} sims", self.simulation_count()))
    }
}
```

**Step 3: Add Go to ChallengeType enum**

Modify the `ChallengeType` enum to include Go:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ChallengeType {
    Chess,
    Morris,
    Gomoku,
    Minesweeper,
    Rune,
    Go,
}
```

**Step 4: Add Go to challenge distribution table**

Add to `CHALLENGE_TABLE`:

```rust
    ChallengeWeight {
        challenge_type: ChallengeType::Go,
        weight: 25,
    },
```

**Step 5: Add Go to create_challenge function**

Add to `create_challenge` match:

```rust
        ChallengeType::Go => PendingChallenge {
            challenge_type: ChallengeType::Go,
            title: "Go: Territory Control".to_string(),
            icon: "◉",
            description: "An ancient master beckons from beneath a gnarled tree, a wooden \
                board resting on a flat stone before them. Nine lines cross nine lines, \
                forming a grid of intersections. 'Black and white stones,' they say, \
                'placed one by one. Surround territory, capture enemies. The simplest \
                rules hide the deepest strategy. Shall we play?'"
                .to_string(),
        },
```

**Step 6: Verify it compiles**

Run: `cargo build 2>&1 | tail -5`

Expected: Compiles

**Step 7: Run menu tests**

Run: `cargo test -p quest --lib challenges::menu -- --nocapture`

Expected: All PASS

**Step 8: Commit**

```bash
git add src/challenges/menu.rs
git commit -m "feat(go): integrate with challenge menu system"
```

---

## Task 11: Add Go to GameState

**Files:**
- Modify: `src/core/game_state.rs`
- Modify: `src/challenges/go/logic.rs`

**Step 1: Add import to game_state.rs**

Add to imports at top of `src/core/game_state.rs`:

```rust
use crate::challenges::go::GoGame;
```

**Step 2: Add active_go field**

Add after `active_rune` field (around line 62):

```rust
    /// Active Go game (transient, not saved)
    #[serde(skip)]
    pub active_go: Option<GoGame>,
```

**Step 3: Initialize in GameState::new**

In the `GameState::new` function, add initialization:

```rust
            active_go: None,
```

**Step 4: Uncomment start_go_game in logic.rs**

Now uncomment the `start_go_game` function body in `src/challenges/go/logic.rs`:

```rust
/// Start a new Go game with the selected difficulty.
pub fn start_go_game(state: &mut GameState, difficulty: GoDifficulty) {
    state.active_go = Some(GoGame::new(difficulty));
    state.challenge_menu.close();
}
```

And uncomment the import:

```rust
use crate::core::game_state::GameState;
```

**Step 5: Add active_go check to try_discover_challenge**

In `src/challenges/menu.rs`, find the `try_discover_challenge` function and add to the early return conditions:

```rust
        || state.active_go.is_some()
```

**Step 6: Verify it compiles**

Run: `cargo build 2>&1 | tail -5`

Expected: Compiles

**Step 7: Run all tests**

Run: `cargo test 2>&1 | grep -E "^test result"`

Expected: All test suites PASS

**Step 8: Commit**

```bash
git add src/core/game_state.rs src/challenges/go/logic.rs src/challenges/menu.rs
git commit -m "feat(go): add active_go to GameState"
```

---

## Task 12: Create Go Scene UI

**Files:**
- Create: `src/ui/go_scene.rs`
- Modify: `src/ui/mod.rs`

**Step 1: Create go_scene.rs**

Create `src/ui/go_scene.rs`:

```rust
//! Go game UI rendering.

use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_game_over_overlay,
    render_info_panel_frame, render_status_bar, render_thinking_status_bar, GameResultType,
};
use crate::challenges::go::{GoGame, GoMove, GoResult, Stone, BOARD_SIZE};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Render the Go game scene.
pub fn render_go_scene(frame: &mut Frame, area: Rect, game: &GoGame) {
    // Game over overlay
    if game.game_result.is_some() {
        render_go_game_over(frame, area, game);
        return;
    }

    // Use shared layout - Go board needs width for box drawing chars
    let layout = create_game_layout(frame, area, " Go ", Color::Green, 11, 22);

    render_board(frame, layout.content, game);
    render_status_bar_content(frame, layout.status_bar, game);
    render_info_panel(frame, layout.info_panel, game);
}

fn render_board(frame: &mut Frame, area: Rect, game: &GoGame) {
    let board_height = BOARD_SIZE as u16;
    let board_width = (BOARD_SIZE * 3 - 2) as u16; // "●──" format
    let y_offset = area.y + (area.height.saturating_sub(board_height)) / 2;
    let x_offset = area.x + (area.width.saturating_sub(board_width)) / 2;

    let human_color = Color::White;
    let ai_color = Color::LightRed;
    let cursor_color = Color::Yellow;
    let last_move_color = Color::Green;
    let grid_color = Color::DarkGray;

    for row in 0..BOARD_SIZE {
        let mut spans = Vec::new();
        for col in 0..BOARD_SIZE {
            let is_cursor = game.cursor == (row, col);
            let is_last_move = game.last_move == Some(GoMove::Place(row, col));
            let is_ko = game.ko_point == Some((row, col));

            // Determine the intersection character
            let (symbol, style) = match game.board[row][col] {
                Some(Stone::Black) => {
                    let base_style = Style::default()
                        .fg(human_color)
                        .add_modifier(Modifier::BOLD);
                    if is_cursor {
                        ("●", base_style.bg(Color::DarkGray))
                    } else if is_last_move {
                        ("●", base_style.fg(last_move_color))
                    } else {
                        ("●", base_style)
                    }
                }
                Some(Stone::White) => {
                    let base_style = Style::default().fg(ai_color).add_modifier(Modifier::BOLD);
                    if is_cursor {
                        ("○", base_style.bg(Color::DarkGray))
                    } else if is_last_move {
                        ("○", base_style.fg(last_move_color))
                    } else {
                        ("○", base_style)
                    }
                }
                None => {
                    if is_cursor {
                        (
                            "□",
                            Style::default()
                                .fg(cursor_color)
                                .add_modifier(Modifier::BOLD),
                        )
                    } else if is_ko {
                        ("×", Style::default().fg(Color::Red))
                    } else {
                        // Grid intersection
                        let ch = get_intersection_char(row, col);
                        (ch, Style::default().fg(grid_color))
                    }
                }
            };

            spans.push(Span::styled(symbol, style));

            // Add horizontal line between intersections
            if col < BOARD_SIZE - 1 {
                let line_char = if game.board[row][col].is_some() || game.board[row][col + 1].is_some() {
                    "──"
                } else {
                    "──"
                };
                spans.push(Span::styled(line_char, Style::default().fg(grid_color)));
            }
        }

        let line = Paragraph::new(Line::from(spans));
        frame.render_widget(
            line,
            Rect::new(x_offset, y_offset + row as u16, board_width, 1),
        );
    }
}

/// Get the appropriate intersection character based on position.
fn get_intersection_char(row: usize, col: usize) -> &'static str {
    let is_top = row == 0;
    let is_bottom = row == BOARD_SIZE - 1;
    let is_left = col == 0;
    let is_right = col == BOARD_SIZE - 1;

    match (is_top, is_bottom, is_left, is_right) {
        (true, _, true, _) => "┌",
        (true, _, _, true) => "┐",
        (_, true, true, _) => "└",
        (_, true, _, true) => "┘",
        (true, _, _, _) => "┬",
        (_, true, _, _) => "┴",
        (_, _, true, _) => "├",
        (_, _, _, true) => "┤",
        _ => "┼",
    }
}

fn render_status_bar_content(frame: &mut Frame, area: Rect, game: &GoGame) {
    if game.ai_thinking {
        render_thinking_status_bar(frame, area, "Opponent is thinking...");
        return;
    }

    if render_forfeit_status_bar(frame, area, game.forfeit_pending) {
        return;
    }

    render_status_bar(
        frame,
        area,
        "Your turn",
        Color::White,
        &[
            ("[Arrows]", "Move"),
            ("[Enter]", "Place"),
            ("[P]", "Pass"),
            ("[Esc]", "Forfeit"),
        ],
    );
}

fn render_info_panel(frame: &mut Frame, area: Rect, game: &GoGame) {
    let inner = render_info_panel_frame(frame, area);

    let lines: Vec<Line> = vec![
        Line::from(Span::styled(
            "RULES",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "Surround territory",
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            "and capture enemies.",
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            "Two passes end game.",
            Style::default().fg(Color::Gray),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Black (You): ", Style::default().fg(Color::DarkGray)),
            Span::styled("●", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("White (AI):  ", Style::default().fg(Color::DarkGray)),
            Span::styled("○", Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Captures: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("You {} | AI {}", game.captured_by_black, game.captured_by_white),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Difficulty: ", Style::default().fg(Color::DarkGray)),
            Span::styled(game.difficulty.name(), Style::default().fg(Color::Cyan)),
        ]),
    ];

    let para = Paragraph::new(lines);
    frame.render_widget(para, inner);
}

fn render_go_game_over(frame: &mut Frame, area: Rect, game: &GoGame) {
    let result_type = match game.game_result {
        Some(GoResult::Win) => GameResultType::Win,
        Some(GoResult::Loss) => GameResultType::Loss,
        Some(GoResult::Draw) => GameResultType::Draw,
        None => return,
    };

    render_game_over_overlay(frame, area, result_type, "Go");
}
```

**Step 2: Add to ui/mod.rs**

Add to `src/ui/mod.rs`:

```rust
pub mod go_scene;
```

And add to the pub use section:

```rust
pub use go_scene::render_go_scene;
```

**Step 3: Verify it compiles**

Run: `cargo build 2>&1 | tail -5`

Expected: Compiles

**Step 4: Commit**

```bash
git add src/ui/go_scene.rs src/ui/mod.rs
git commit -m "feat(go): create Go scene UI"
```

---

## Task 13: Wire Up Go in main.rs

**Files:**
- Modify: `src/main.rs`

**Step 1: Add Go imports**

Find the challenges imports section and add:

```rust
use crate::challenges::go::{process_go_ai, process_human_move, process_human_pass, start_go_game, GoDifficulty, GoResult};
```

**Step 2: Add Go to ChallengeType match in discovery**

Find where `try_discover_challenge` results are handled (around line 1252) and add Go:

```rust
                ChallengeType::Go => (
                    "◉",
                    "An ancient master beckons from beneath a gnarled tree...",
                ),
```

**Step 3: Add Go input handling**

Find the input handling section (after Gomoku handling, around line 889) and add Go handling:

```rust
                            // Handle active Go game input
                            if let Some(ref mut go_game) = state.active_go {
                                if go_game.game_result.is_some() {
                                    // Any key dismisses result and applies rewards
                                    let old_prestige = state.prestige_rank;
                                    if let Some((result, prestige_gained)) = apply_go_result(&mut state) {
                                        match result {
                                            GoResult::Win => {
                                                state.combat_state.add_log_entry(
                                                    "◉ Victory! The master nods approvingly."
                                                        .to_string(),
                                                    false,
                                                    true,
                                                );
                                                if prestige_gained > 0 {
                                                    state.combat_state.add_log_entry(
                                                        format!(
                                                            "◉ +{} Prestige Ranks (P{} → P{})",
                                                            prestige_gained,
                                                            old_prestige,
                                                            state.prestige_rank
                                                        ),
                                                        false,
                                                        false,
                                                    );
                                                }
                                            }
                                            GoResult::Loss => {
                                                state.combat_state.add_log_entry(
                                                    "◉ The master smiles. 'You learn by losing.'"
                                                        .to_string(),
                                                    false,
                                                    true,
                                                );
                                            }
                                            GoResult::Draw => {
                                                state.combat_state.add_log_entry(
                                                    "◉ A rare tie. The master seems impressed."
                                                        .to_string(),
                                                    false,
                                                    true,
                                                );
                                            }
                                        }
                                    }
                                    continue;
                                }

                                // Handle forfeit confirmation
                                if go_game.forfeit_pending {
                                    match key_event.code {
                                        KeyCode::Esc => {
                                            go_game.game_result = Some(GoResult::Loss);
                                        }
                                        _ => {
                                            go_game.forfeit_pending = false;
                                        }
                                    }
                                    continue;
                                }

                                // Normal game input
                                if !go_game.ai_thinking {
                                    match key_event.code {
                                        KeyCode::Up => go_game.move_cursor(-1, 0),
                                        KeyCode::Down => go_game.move_cursor(1, 0),
                                        KeyCode::Left => go_game.move_cursor(0, -1),
                                        KeyCode::Right => go_game.move_cursor(0, 1),
                                        KeyCode::Enter => {
                                            process_human_move(go_game);
                                        }
                                        KeyCode::Char('p') | KeyCode::Char('P') => {
                                            process_human_pass(go_game);
                                        }
                                        KeyCode::Esc => {
                                            go_game.forfeit_pending = true;
                                        }
                                        _ => {}
                                    }
                                }
                                continue;
                            }
```

**Step 4: Add Go AI tick processing**

Find the tick processing section (around line 1247) and add:

```rust
    // Process Go AI thinking
    if let Some(ref mut go_game) = game_state.active_go {
        let mut rng = rand::thread_rng();
        process_go_ai(go_game, &mut rng);
    }
```

**Step 5: Add Go rendering**

Find where other game scenes are rendered and add:

```rust
        } else if let Some(ref go_game) = state.active_go {
            ui::go_scene::render_go_scene(frame, main_area, go_game);
```

**Step 6: Add Go to challenge menu start**

Find where challenges are started from the menu and add Go case:

```rust
                            ChallengeType::Go => {
                                let difficulty = GoDifficulty::from_index(
                                    state.challenge_menu.selected_difficulty,
                                );
                                start_go_game(&mut state, difficulty);
                            }
```

**Step 7: Add apply_go_result helper function**

Add near other apply_*_result functions:

```rust
/// Apply the result of a completed Go game and return (result, prestige_gained).
fn apply_go_result(state: &mut GameState) -> Option<(GoResult, u32)> {
    let go_game = state.active_go.take()?;
    let result = go_game.game_result?;

    let prestige_gained = if result == GoResult::Win {
        let reward = go_game.difficulty.reward_prestige();
        state.prestige_rank += reward;
        reward
    } else {
        0
    };

    Some((result, prestige_gained))
}
```

**Step 8: Verify it compiles**

Run: `cargo build 2>&1 | tail -10`

Expected: Compiles (may have warnings)

**Step 9: Run full test suite**

Run: `cargo test 2>&1 | grep -E "^test result"`

Expected: All PASS

**Step 10: Commit**

```bash
git add src/main.rs
git commit -m "feat(go): wire up Go challenge in main game loop"
```

---

## Task 14: Update lib.rs Exports

**Files:**
- Modify: `src/lib.rs`

**Step 1: Add Go exports**

Add to the re-exports in `src/lib.rs`:

```rust
pub use challenges::{
    ChessDifficulty, ChessGame, ChessResult, GoDifficulty, GoGame, GoResult,
    GomokuDifficulty, GomokuGame, GomokuResult, MinesweeperDifficulty, MinesweeperGame,
    MinesweeperResult, MorrisDifficulty, MorrisGame, MorrisPhase, MorrisResult,
    RuneDifficulty, RuneGame, RuneResult,
};
```

**Step 2: Verify it compiles**

Run: `cargo build 2>&1 | tail -5`

Expected: Compiles

**Step 3: Commit**

```bash
git add src/lib.rs
git commit -m "feat(go): export Go types from lib.rs"
```

---

## Task 15: Final Testing and Cleanup

**Step 1: Run full CI checks**

Run: `make check`

Expected: All checks pass

**Step 2: Fix any clippy warnings**

Run: `cargo clippy --all-targets -- -D warnings 2>&1 | head -50`

Fix any warnings that appear.

**Step 3: Format code**

Run: `make fmt`

**Step 4: Run tests one more time**

Run: `cargo test`

Expected: All PASS

**Step 5: Test the game manually**

Run: `cargo run`

- Press Tab to open challenge menu
- Wait for Go challenge to appear (or use --debug mode)
- Accept the Go challenge
- Play a few moves to verify:
  - Cursor movement works
  - Stone placement works
  - AI responds
  - Pass works (P key)
  - Forfeit works (Esc twice)

**Step 6: Final commit**

```bash
git add -A
git commit -m "feat(go): complete Go challenge implementation"
```

---

## Summary

This plan implements a complete Go challenge for Quest with:

1. **Core game logic** - Stone placement, capture detection, ko rule, suicide prevention, Chinese scoring
2. **MCTS AI** - Monte Carlo Tree Search with UCT selection, configurable simulation counts
3. **UI** - Board rendering with box-drawing characters, cursor navigation, game-over overlay
4. **Integration** - Challenge menu, discovery system, GameState, main game loop

**Files created:**
- `src/challenges/go/mod.rs`
- `src/challenges/go/types.rs`
- `src/challenges/go/logic.rs`
- `src/challenges/go/mcts.rs`
- `src/ui/go_scene.rs`

**Files modified:**
- `src/challenges/mod.rs`
- `src/challenges/menu.rs`
- `src/core/game_state.rs`
- `src/ui/mod.rs`
- `src/main.rs`
- `src/lib.rs`
