# Vault Warden Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking. Use the `add-challenge` skill for integration checklist reference.

**Goal:** Add a Sokoban puzzle challenge called "Vault Warden" with curated Microban levels, emoji rendering, limited undo, and deadlock detection.

**Architecture:** Turn-based puzzle with curated levels stored as const data. Game state tracks grid terrain, crate positions, and move history for undo. Rendering uses emoji characters (2-column wide) in the standard `create_game_layout()` framework. 16 files touched total (5 new, 11 modified).

**Tech Stack:** Rust, Ratatui, rand (for level selection)

**Spec:** `docs/superpowers/specs/2026-03-15-vault-warden-design.md`

---

## Chunk 1: Core Game Module

### Task 1: Types and Level Data

**Files:**
- Create: `src/challenges/vault_warden/types.rs`
- Create: `src/challenges/vault_warden/levels.rs`

- [ ] **Step 1: Create types.rs with all data structures**

```rust
//! Vault Warden data types.

use crate::challenges::difficulty_enum_impl;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultWardenDifficulty {
    Novice,
    Apprentice,
    Journeyman,
    Master,
}

difficulty_enum_impl!(VaultWardenDifficulty);

impl VaultWardenDifficulty {
    /// Number of undos available at this difficulty.
    pub fn max_undos(&self) -> u8 {
        match self {
            Self::Novice => 5,
            Self::Apprentice => 3,
            Self::Journeyman => 2,
            Self::Master => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultWardenResult {
    Win,
    Loss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultWardenInput {
    Up,
    Down,
    Left,
    Right,
    Undo,
    Restart,
    Forfeit,
    Other,
}

/// Static terrain cell (walls and floors). Goals are tracked separately.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Wall,
    Floor,
}

/// Record of a single move for undo support.
#[derive(Debug, Clone)]
pub struct MoveRecord {
    pub player_from: (usize, usize),
    pub pushed_crate: Option<CratePush>,
}

#[derive(Debug, Clone)]
pub struct CratePush {
    pub from: (usize, usize),
    pub to: (usize, usize),
}

#[derive(Debug, Clone)]
pub struct VaultWardenGame {
    pub difficulty: VaultWardenDifficulty,
    pub game_result: Option<VaultWardenResult>,
    pub forfeit_pending: bool,
    pub grid: Vec<Vec<Cell>>,
    pub width: usize,
    pub height: usize,
    pub player_pos: (usize, usize),
    pub crate_positions: Vec<(usize, usize)>,
    pub goal_positions: Vec<(usize, usize)>,
    pub moves: u16,
    pub move_limit: u16,
    pub par: u16,
    pub undos_remaining: u8,
    pub undos_max: u8,
    pub move_history: Vec<MoveRecord>,
    // Snapshot of initial state for restart
    pub initial_player_pos: (usize, usize),
    pub initial_crate_positions: Vec<(usize, usize)>,
}

impl VaultWardenGame {
    /// Count of crates currently on goal squares.
    pub fn crates_on_goals(&self) -> usize {
        self.crate_positions
            .iter()
            .filter(|c| self.goal_positions.contains(c))
            .count()
    }

    /// Total number of crates (= total goals).
    pub fn total_crates(&self) -> usize {
        self.crate_positions.len()
    }

    /// Check if a position contains a crate.
    pub fn has_crate_at(&self, pos: (usize, usize)) -> bool {
        self.crate_positions.contains(&pos)
    }

    /// Check if a position is a goal.
    pub fn is_goal(&self, pos: (usize, usize)) -> bool {
        self.goal_positions.contains(&pos)
    }

    /// Check if a position is walkable (floor/goal, not wall, not crate).
    pub fn is_walkable(&self, pos: (usize, usize)) -> bool {
        if pos.0 >= self.height || pos.1 >= self.width {
            return false;
        }
        self.grid[pos.0][pos.1] != Cell::Wall && !self.has_crate_at(pos)
    }
}
```

- [ ] **Step 2: Create levels.rs with Microban level data**

Create `src/challenges/vault_warden/levels.rs` with curated levels from Microban. Each level uses standard Sokoban notation (`#` wall, ` ` floor, `$` crate, `.` goal, `*` crate on goal, `@` player, `+` player on goal).

```rust
//! Curated Sokoban levels from Microban by David W. Skinner.
//! Free for personal use and redistribution with attribution.
//! Original: http://www.abelmartin.com/rj/sokobanJS/Skinner/David%20W.%20Skinner%20-%20Microban.htm

pub struct VaultWardenLevel {
    pub data: &'static str,
    pub optimal_moves: u16,
}

// Novice: 5x5-7x7 grids, 1-2 crates
pub const NOVICE_LEVELS: &[VaultWardenLevel] = &[
    // Microban #1 (6x6, 1 crate, optimal 4 moves)
    VaultWardenLevel {
        data: "\
####
# .#
#  ###
#*@  #
#  $ #
#  ###
####",
        optimal_moves: 9,
    },
    // ... populate with 20-40 Novice-appropriate levels
];

pub const APPRENTICE_LEVELS: &[VaultWardenLevel] = &[
    // ... 20-40 Apprentice-appropriate levels (6x6-8x8, 2-3 crates)
];

pub const JOURNEYMAN_LEVELS: &[VaultWardenLevel] = &[
    // ... 15-30 Journeyman-appropriate levels (7x7-9x9, 3-4 crates)
];

pub const MASTER_LEVELS: &[VaultWardenLevel] = &[
    // ... 10-20 Master-appropriate levels (9x9-11x11, 4-5 crates)
];
```

**Important:** The implementer must source actual Microban levels with correct optimal move counts from published solutions. Each level's `data` field is a multi-line string in standard Sokoban notation. Categorize by grid size and crate count per the spec's tier table. Aim for at least 10 levels per tier, more for lower tiers.

- [ ] **Step 3: Write tests for types**

Add to the bottom of `types.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_enum() {
        assert_eq!(VaultWardenDifficulty::ALL.len(), 4);
        assert_eq!(
            VaultWardenDifficulty::from_index(0),
            VaultWardenDifficulty::Novice
        );
        assert_eq!(
            VaultWardenDifficulty::from_index(3),
            VaultWardenDifficulty::Master
        );
        assert_eq!(
            VaultWardenDifficulty::from_index(99),
            VaultWardenDifficulty::Novice
        );
    }

    #[test]
    fn test_max_undos() {
        assert_eq!(VaultWardenDifficulty::Novice.max_undos(), 5);
        assert_eq!(VaultWardenDifficulty::Apprentice.max_undos(), 3);
        assert_eq!(VaultWardenDifficulty::Journeyman.max_undos(), 2);
        assert_eq!(VaultWardenDifficulty::Master.max_undos(), 1);
    }

    #[test]
    fn test_crates_on_goals() {
        let game = VaultWardenGame {
            difficulty: VaultWardenDifficulty::Novice,
            game_result: None,
            forfeit_pending: false,
            grid: vec![vec![Cell::Floor; 5]; 5],
            width: 5,
            height: 5,
            player_pos: (0, 0),
            crate_positions: vec![(1, 1), (2, 2)],
            goal_positions: vec![(1, 1), (3, 3)],
            moves: 0,
            move_limit: 20,
            par: 8,
            undos_remaining: 5,
            undos_max: 5,
            move_history: vec![],
            initial_player_pos: (0, 0),
            initial_crate_positions: vec![(1, 1), (2, 2)],
        };
        assert_eq!(game.crates_on_goals(), 1); // (1,1) is on goal
        assert_eq!(game.total_crates(), 2);
        assert!(game.has_crate_at((1, 1)));
        assert!(!game.has_crate_at((0, 0)));
        assert!(game.is_goal((1, 1)));
        assert!(game.is_goal((3, 3)));
        assert!(!game.is_goal((0, 0)));
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test --lib vault_warden::types`
Expected: All tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/challenges/vault_warden/types.rs src/challenges/vault_warden/levels.rs
git commit -m "feat(vault-warden): add types, level data, and difficulty enums"
```

---

### Task 2: Game Logic — Level Parsing and Movement

**Files:**
- Create: `src/challenges/vault_warden/logic.rs`
- Create: `src/challenges/vault_warden/mod.rs`
- Modify: `src/challenges/mod.rs` (add module declaration only, enough to compile)

- [ ] **Step 1: Create mod.rs with module declarations**

```rust
//! Vault Warden challenge — Sokoban puzzle.

pub mod levels;
pub mod logic;
pub mod types;

pub use logic::{process_input, start_vault_warden_game};
pub use types::{VaultWardenDifficulty, VaultWardenGame, VaultWardenInput, VaultWardenResult};

use crate::challenges::impl_apply_game_result;

impl_apply_game_result! {
    fn apply_vault_warden_result;
    variant: VaultWarden;
    result_body: |result, _state, _reward| {
        use VaultWardenResult::*;
        match result {
            Win => (true, ""),
            Loss => (false, "The relics remain scattered."),
        }
    }
    game_type: crate::achievements::MinigameType::VaultWarden;
    icon: "\u{1F512}";
    win_message: "All relics placed!";
}
```

- [ ] **Step 2: Create logic.rs with level parsing**

```rust
//! Vault Warden game logic.

use super::levels::{
    VaultWardenLevel, APPRENTICE_LEVELS, JOURNEYMAN_LEVELS, MASTER_LEVELS, NOVICE_LEVELS,
};
use super::types::*;
use crate::challenges::ActiveMinigame;
use rand::Rng;

/// Parse a Sokoban level string into game state.
pub fn parse_level(level: &VaultWardenLevel, difficulty: VaultWardenDifficulty) -> VaultWardenGame {
    let mut grid = Vec::new();
    let mut crate_positions = Vec::new();
    let mut goal_positions = Vec::new();
    let mut player_pos = (0, 0);

    let lines: Vec<&str> = level.data.lines().collect();
    let height = lines.len();
    let width = lines.iter().map(|l| l.len()).max().unwrap_or(0);

    for (row, line) in lines.iter().enumerate() {
        let mut grid_row = vec![Cell::Wall; width]; // Default to wall for padding
        for (col, ch) in line.chars().enumerate() {
            match ch {
                '#' => grid_row[col] = Cell::Wall,
                ' ' | '-' => grid_row[col] = Cell::Floor,
                '$' => {
                    grid_row[col] = Cell::Floor;
                    crate_positions.push((row, col));
                }
                '.' => {
                    grid_row[col] = Cell::Floor;
                    goal_positions.push((row, col));
                }
                '*' => {
                    grid_row[col] = Cell::Floor;
                    crate_positions.push((row, col));
                    goal_positions.push((row, col));
                }
                '@' => {
                    grid_row[col] = Cell::Floor;
                    player_pos = (row, col);
                }
                '+' => {
                    grid_row[col] = Cell::Floor;
                    player_pos = (row, col);
                    goal_positions.push((row, col));
                }
                _ => grid_row[col] = Cell::Floor,
            }
        }
        grid.push(grid_row);
    }

    let undos_max = difficulty.max_undos();
    let move_limit = ((level.optimal_moves as f64) * 2.5).ceil() as u16;

    VaultWardenGame {
        difficulty,
        game_result: None,
        forfeit_pending: false,
        grid,
        width,
        height,
        player_pos,
        crate_positions: crate_positions.clone(),
        goal_positions,
        moves: 0,
        move_limit,
        par: level.optimal_moves,
        undos_remaining: undos_max,
        undos_max,
        move_history: Vec::new(),
        initial_player_pos: player_pos,
        initial_crate_positions: crate_positions,
    }
}

/// Start a new Vault Warden game with a random level for the given difficulty.
pub fn start_vault_warden_game<R: Rng>(
    difficulty: VaultWardenDifficulty,
    rng: &mut R,
) -> ActiveMinigame {
    let levels = match difficulty {
        VaultWardenDifficulty::Novice => NOVICE_LEVELS,
        VaultWardenDifficulty::Apprentice => APPRENTICE_LEVELS,
        VaultWardenDifficulty::Journeyman => JOURNEYMAN_LEVELS,
        VaultWardenDifficulty::Master => MASTER_LEVELS,
    };
    let idx = rng.random_range(0..levels.len());
    let game = parse_level(&levels[idx], difficulty);
    ActiveMinigame::VaultWarden(game)
}

/// Process player input.
pub fn process_input(game: &mut VaultWardenGame, input: VaultWardenInput) {
    if game.game_result.is_some() {
        return;
    }

    // Handle forfeit double-Esc pattern
    if game.forfeit_pending {
        match input {
            VaultWardenInput::Forfeit => {
                crate::challenges::handle_forfeit(
                    &mut game.game_result,
                    &mut game.forfeit_pending,
                    VaultWardenResult::Loss,
                );
            }
            _ => {
                crate::challenges::cancel_forfeit_if_pending(&mut game.forfeit_pending);
            }
        }
        return;
    }

    match input {
        VaultWardenInput::Up => try_move(game, -1, 0),
        VaultWardenInput::Down => try_move(game, 1, 0),
        VaultWardenInput::Left => try_move(game, 0, -1),
        VaultWardenInput::Right => try_move(game, 0, 1),
        VaultWardenInput::Undo => undo_move(game),
        VaultWardenInput::Restart => restart_level(game),
        VaultWardenInput::Forfeit => {
            crate::challenges::handle_forfeit(
                &mut game.game_result,
                &mut game.forfeit_pending,
                VaultWardenResult::Loss,
            );
        }
        VaultWardenInput::Other => {}
    }
}

/// Attempt to move the player in direction (dr, dc).
fn try_move(game: &mut VaultWardenGame, dr: i32, dc: i32) {
    let (pr, pc) = game.player_pos;
    let new_r = pr as i32 + dr;
    let new_c = pc as i32 + dc;

    if new_r < 0 || new_c < 0 {
        return;
    }
    let new_pos = (new_r as usize, new_c as usize);

    if new_pos.0 >= game.height || new_pos.1 >= game.width {
        return;
    }
    if game.grid[new_pos.0][new_pos.1] == Cell::Wall {
        return;
    }

    // Check if pushing a crate
    let pushed_crate = if game.has_crate_at(new_pos) {
        let crate_new_r = new_pos.0 as i32 + dr;
        let crate_new_c = new_pos.1 as i32 + dc;

        if crate_new_r < 0 || crate_new_c < 0 {
            return; // Can't push crate out of bounds
        }
        let crate_new_pos = (crate_new_r as usize, crate_new_c as usize);

        if crate_new_pos.0 >= game.height || crate_new_pos.1 >= game.width {
            return;
        }
        if game.grid[crate_new_pos.0][crate_new_pos.1] == Cell::Wall {
            return;
        }
        if game.has_crate_at(crate_new_pos) {
            return; // Can't push into another crate
        }

        // Move the crate
        if let Some(idx) = game.crate_positions.iter().position(|&c| c == new_pos) {
            game.crate_positions[idx] = crate_new_pos;
        }

        Some(CratePush {
            from: new_pos,
            to: crate_new_pos,
        })
    } else {
        None
    };

    // Record move for undo
    game.move_history.push(MoveRecord {
        player_from: game.player_pos,
        pushed_crate,
    });

    // Move player
    game.player_pos = new_pos;
    game.moves += 1;

    // Check win/loss
    if game.crates_on_goals() == game.total_crates() {
        game.game_result = Some(VaultWardenResult::Win);
    } else if game.moves >= game.move_limit {
        game.game_result = Some(VaultWardenResult::Loss);
    }
}

/// Undo the last move.
fn undo_move(game: &mut VaultWardenGame) {
    if game.undos_remaining == 0 || game.move_history.is_empty() {
        return;
    }

    let record = game.move_history.pop().unwrap();
    game.player_pos = record.player_from;

    // Reverse crate push if one occurred
    if let Some(push) = record.pushed_crate {
        if let Some(idx) = game.crate_positions.iter().position(|&c| c == push.to) {
            game.crate_positions[idx] = push.from;
        }
    }

    game.undos_remaining -= 1;
    // Note: moves counter is NOT decremented (per spec)
}

/// Restart the level from scratch.
fn restart_level(game: &mut VaultWardenGame) {
    game.player_pos = game.initial_player_pos;
    game.crate_positions = game.initial_crate_positions.clone();
    game.moves = 0;
    game.undos_remaining = game.undos_max;
    game.move_history.clear();
}

/// Check if a crate is deadlocked (in a non-goal corner).
pub fn is_crate_deadlocked(game: &VaultWardenGame, crate_pos: (usize, usize)) -> bool {
    // A crate on a goal is never considered deadlocked
    if game.is_goal(crate_pos) {
        return false;
    }

    let (r, c) = crate_pos;

    // Check all four corners: if wall on two adjacent sides, it's a corner deadlock
    let up_wall = r == 0 || game.grid[r - 1][c] == Cell::Wall;
    let down_wall = r + 1 >= game.height || game.grid[r + 1][c] == Cell::Wall;
    let left_wall = c == 0 || game.grid[r][c - 1] == Cell::Wall;
    let right_wall = c + 1 >= game.width || game.grid[r][c + 1] == Cell::Wall;

    // Corner deadlock: wall on two perpendicular sides
    if (up_wall && left_wall)
        || (up_wall && right_wall)
        || (down_wall && left_wall)
        || (down_wall && right_wall)
    {
        return true;
    }

    // Wall-edge deadlock: crate against a straight wall with no goal along it
    if up_wall || down_wall {
        // Check if there's any goal along this horizontal wall segment
        if !has_goal_along_wall_segment(game, crate_pos, 0, -1, up_wall, down_wall)
            && !has_goal_along_wall_segment(game, crate_pos, 0, 1, up_wall, down_wall)
        {
            return true;
        }
    }

    if left_wall || right_wall {
        // Check if there's any goal along this vertical wall segment
        if !has_goal_along_wall_segment_vertical(game, crate_pos, -1, left_wall, right_wall)
            && !has_goal_along_wall_segment_vertical(game, crate_pos, 1, left_wall, right_wall)
        {
            return true;
        }
    }

    false
}

/// Walk along a horizontal wall segment checking for goals.
fn has_goal_along_wall_segment(
    game: &VaultWardenGame,
    start: (usize, usize),
    _dr: i32,
    dc: i32,
    wall_above: bool,
    wall_below: bool,
) -> bool {
    if game.is_goal(start) {
        return true;
    }
    let mut c = start.1 as i32 + dc;
    while c >= 0 && (c as usize) < game.width {
        let pos = (start.0, c as usize);
        if game.grid[pos.0][pos.1] == Cell::Wall {
            break; // Hit a wall, end of segment
        }
        // Check wall continuity above/below
        let still_walled = if wall_above {
            pos.0 == 0 || game.grid[pos.0 - 1][pos.1] == Cell::Wall
        } else if wall_below {
            pos.0 + 1 >= game.height || game.grid[pos.0 + 1][pos.1] == Cell::Wall
        } else {
            false
        };
        if !still_walled {
            return true; // Wall ended, crate could escape this direction
        }
        if game.is_goal(pos) {
            return true;
        }
        c += dc;
    }
    false
}

/// Walk along a vertical wall segment checking for goals.
fn has_goal_along_wall_segment_vertical(
    game: &VaultWardenGame,
    start: (usize, usize),
    dr: i32,
    wall_left: bool,
    wall_right: bool,
) -> bool {
    if game.is_goal(start) {
        return true;
    }
    let mut r = start.0 as i32 + dr;
    while r >= 0 && (r as usize) < game.height {
        let pos = (r as usize, start.1);
        if game.grid[pos.0][pos.1] == Cell::Wall {
            break;
        }
        let still_walled = if wall_left {
            pos.1 == 0 || game.grid[pos.0][pos.1 - 1] == Cell::Wall
        } else if wall_right {
            pos.1 + 1 >= game.width || game.grid[pos.0][pos.1 + 1] == Cell::Wall
        } else {
            false
        };
        if !still_walled {
            return true;
        }
        if game.is_goal(pos) {
            return true;
        }
        r += dr;
    }
    false
}

// --- DifficultyInfo impl ---

impl crate::challenges::menu::DifficultyInfo for VaultWardenDifficulty {
    fn name(&self) -> &'static str {
        VaultWardenDifficulty::name(self)
    }

    fn reward(&self) -> crate::challenges::menu::ChallengeReward {
        match self {
            VaultWardenDifficulty::Novice => crate::challenges::menu::ChallengeReward {
                prestige_ranks: 0,
                stormglass: 400,
                fishing_ranks: 0,
            },
            VaultWardenDifficulty::Apprentice => crate::challenges::menu::ChallengeReward {
                prestige_ranks: 0,
                stormglass: 1_200,
                fishing_ranks: 0,
            },
            VaultWardenDifficulty::Journeyman => crate::challenges::menu::ChallengeReward {
                prestige_ranks: 1,
                stormglass: 3_000,
                fishing_ranks: 0,
            },
            VaultWardenDifficulty::Master => crate::challenges::menu::ChallengeReward {
                prestige_ranks: 2,
                stormglass: 6_000,
                fishing_ranks: 0,
            },
        }
    }

    fn extra_info(&self) -> Option<String> {
        let undos = self.max_undos();
        Some(format!("{} undos", undos))
    }
}
```

- [ ] **Step 3: Add `pub mod vault_warden;` to `src/challenges/mod.rs`**

Add after the last module declaration (after `pub mod runic_lights;`):

```rust
pub mod vault_warden;
```

Also add to the re-exports section:

```rust
pub use vault_warden::{VaultWardenDifficulty, VaultWardenGame, VaultWardenResult};
```

Also add `VaultWarden(VaultWardenGame)` to the `ActiveMinigame` enum, and add the match arm in `has_game_result()`:

```rust
ActiveMinigame::VaultWarden(g) => g.game_result.is_some(),
```

- [ ] **Step 4: Write tests for logic**

Add to the bottom of `logic.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_level() -> VaultWardenLevel {
        // Simple 5x5 level: player at (2,1), one crate at (2,2), goal at (2,3)
        VaultWardenLevel {
            data: "\
#####
#   #
# $.#
#   #
#####",
            optimal_moves: 1,
        }
    }

    #[test]
    fn test_parse_level() {
        let level = make_test_level();
        let game = parse_level(&level, VaultWardenDifficulty::Novice);
        assert_eq!(game.width, 5);
        assert_eq!(game.height, 5);
        assert_eq!(game.player_pos, (2, 1));
        assert_eq!(game.crate_positions, vec![(2, 2)]);
        assert_eq!(game.goal_positions, vec![(2, 3)]);
        assert_eq!(game.par, 1);
        assert_eq!(game.move_limit, 3); // ceil(1 * 2.5) = 3
        assert_eq!(game.undos_remaining, 5); // Novice
    }

    #[test]
    fn test_push_crate_to_win() {
        let level = make_test_level();
        let mut game = parse_level(&level, VaultWardenDifficulty::Novice);

        // Push crate right onto goal
        process_input(&mut game, VaultWardenInput::Right);
        assert_eq!(game.player_pos, (2, 2));
        assert_eq!(game.crate_positions, vec![(2, 3)]);
        assert_eq!(game.moves, 1);
        assert_eq!(game.game_result, Some(VaultWardenResult::Win));
    }

    #[test]
    fn test_cant_push_into_wall() {
        let level = VaultWardenLevel {
            data: "\
#####
#   #
#@$##
#   #
#####",
            optimal_moves: 10,
        };
        let mut game = parse_level(&level, VaultWardenDifficulty::Novice);
        let orig_crate = game.crate_positions[0];

        // Try to push crate right into wall
        process_input(&mut game, VaultWardenInput::Right);
        assert_eq!(game.crate_positions[0], orig_crate); // Crate didn't move
        assert_eq!(game.moves, 0); // Move didn't count
    }

    #[test]
    fn test_undo() {
        let level = make_test_level();
        let mut game = parse_level(&level, VaultWardenDifficulty::Novice);
        let orig_player = game.player_pos;
        let orig_crate = game.crate_positions[0];

        // Move right (pushing crate)
        process_input(&mut game, VaultWardenInput::Right);
        assert_ne!(game.player_pos, orig_player);

        // Undo — but game is already won, so undo won't work
        // Let's test with a move that doesn't win
        let level2 = VaultWardenLevel {
            data: "\
######
#    #
# @$.#
#    #
######",
            optimal_moves: 2,
        };
        let mut game2 = parse_level(&level2, VaultWardenDifficulty::Novice);
        let orig_player2 = game2.player_pos;
        let orig_crate2 = game2.crate_positions[0];

        process_input(&mut game2, VaultWardenInput::Right);
        assert_eq!(game2.moves, 1);
        assert_eq!(game2.undos_remaining, 5);

        process_input(&mut game2, VaultWardenInput::Undo);
        assert_eq!(game2.player_pos, orig_player2);
        assert_eq!(game2.crate_positions[0], orig_crate2);
        assert_eq!(game2.moves, 1); // Moves NOT decremented
        assert_eq!(game2.undos_remaining, 4);
    }

    #[test]
    fn test_undo_budget_exhaustion() {
        let level = VaultWardenLevel {
            data: "\
######
#    #
# @$.#
#    #
######",
            optimal_moves: 10,
        };
        let mut game = parse_level(&level, VaultWardenDifficulty::Master); // 1 undo

        // Move up (no crate push)
        process_input(&mut game, VaultWardenInput::Up);
        assert_eq!(game.undos_remaining, 1);

        // Undo
        process_input(&mut game, VaultWardenInput::Undo);
        assert_eq!(game.undos_remaining, 0);

        // Move again
        process_input(&mut game, VaultWardenInput::Up);

        // Try to undo again — should fail silently
        let pos_before = game.player_pos;
        process_input(&mut game, VaultWardenInput::Undo);
        assert_eq!(game.player_pos, pos_before); // Didn't undo
        assert_eq!(game.undos_remaining, 0);
    }

    #[test]
    fn test_restart() {
        let level = make_test_level();
        let mut game = parse_level(&level, VaultWardenDifficulty::Novice);
        let orig_player = game.player_pos;
        let orig_crates = game.crate_positions.clone();

        // Move a few times (up, to avoid winning)
        process_input(&mut game, VaultWardenInput::Up);
        assert_ne!(game.player_pos, orig_player);
        assert_eq!(game.moves, 1);

        // Restart
        process_input(&mut game, VaultWardenInput::Restart);
        assert_eq!(game.player_pos, orig_player);
        assert_eq!(game.crate_positions, orig_crates);
        assert_eq!(game.moves, 0);
        assert_eq!(game.undos_remaining, game.undos_max);
        assert!(game.move_history.is_empty());
    }

    #[test]
    fn test_forfeit() {
        let level = make_test_level();
        let mut game = parse_level(&level, VaultWardenDifficulty::Novice);

        // First Esc sets pending
        process_input(&mut game, VaultWardenInput::Forfeit);
        assert!(game.forfeit_pending);
        assert!(game.game_result.is_none());

        // Any other key cancels
        process_input(&mut game, VaultWardenInput::Up);
        assert!(!game.forfeit_pending);

        // First Esc again
        process_input(&mut game, VaultWardenInput::Forfeit);
        assert!(game.forfeit_pending);

        // Second Esc confirms
        process_input(&mut game, VaultWardenInput::Forfeit);
        assert_eq!(game.game_result, Some(VaultWardenResult::Loss));
    }

    #[test]
    fn test_move_limit_loss() {
        let level = VaultWardenLevel {
            data: "\
#####
#   #
# @.#
# $ #
#####",
            optimal_moves: 1, // move_limit = 3
        };
        let mut game = parse_level(&level, VaultWardenDifficulty::Novice);
        assert_eq!(game.move_limit, 3);

        // Make 3 moves without winning
        process_input(&mut game, VaultWardenInput::Up);
        process_input(&mut game, VaultWardenInput::Down);
        process_input(&mut game, VaultWardenInput::Up);
        assert_eq!(game.moves, 3);
        assert_eq!(game.game_result, Some(VaultWardenResult::Loss));
    }

    #[test]
    fn test_corner_deadlock() {
        let level = VaultWardenLevel {
            data: "\
#####
#  .#
# $ #
#@  #
#####",
            optimal_moves: 10,
        };
        let game = parse_level(&level, VaultWardenDifficulty::Novice);

        // Crate at (2,2) is not in a corner — not deadlocked
        assert!(!is_crate_deadlocked(&game, (2, 2)));

        // Simulate crate pushed to top-left corner (1,1) — no goal there
        let mut game2 = game.clone();
        game2.crate_positions = vec![(1, 1)];
        assert!(is_crate_deadlocked(&game2, (1, 1)));

        // Crate on a goal is never deadlocked
        let mut game3 = game.clone();
        game3.crate_positions = vec![(1, 3)];
        assert!(!is_crate_deadlocked(&game3, (1, 3))); // (1,3) is a goal
    }

    #[test]
    fn test_parse_crate_on_goal() {
        let level = VaultWardenLevel {
            data: "\
####
#@ #
# *#
####",
            optimal_moves: 5,
        };
        let game = parse_level(&level, VaultWardenDifficulty::Novice);
        // '*' means crate AND goal at same position
        assert!(game.crate_positions.contains(&(2, 2)));
        assert!(game.goal_positions.contains(&(2, 2)));
    }

    #[test]
    fn test_parse_player_on_goal() {
        let level = VaultWardenLevel {
            data: "\
####
# $#
#+.#
####",
            optimal_moves: 3,
        };
        let game = parse_level(&level, VaultWardenDifficulty::Novice);
        // '+' means player AND goal at same position
        assert_eq!(game.player_pos, (2, 1));
        assert!(game.goal_positions.contains(&(2, 1)));
    }
}
```

- [ ] **Step 5: Run tests**

Run: `cargo test --lib vault_warden`
Expected: All tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/challenges/vault_warden/ src/challenges/mod.rs
git commit -m "feat(vault-warden): add game logic with movement, undo, deadlock detection"
```

---

## Chunk 2: UI and Integration Wiring

### Task 3: UI Scene Rendering

**Files:**
- Create: `src/ui/vault_warden_scene.rs`
- Modify: `src/ui/mod.rs`

- [ ] **Step 1: Create vault_warden_scene.rs**

```rust
//! Vault Warden minigame UI rendering.

use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_game_over_overlay,
    render_info_panel_frame, render_minigame_too_small, render_status_bar, GameResultType,
};
use crate::challenges::vault_warden::{
    VaultWardenGame, VaultWardenResult,
};
use crate::challenges::vault_warden::logic::is_crate_deadlocked;
use crate::challenges::menu::DifficultyInfo;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Border color: amber/gold vault theme.
const BORDER_COLOR: Color = Color::Rgb(180, 140, 40);

/// Render the Vault Warden game scene.
pub fn render_vault_warden(
    frame: &mut Frame,
    area: Rect,
    game: &VaultWardenGame,
    ctx: &super::responsive::LayoutContext,
    show_dismiss_hint: bool,
) {
    // Game over overlay
    if game.game_result.is_some() {
        render_game_over(frame, area, game, show_dismiss_hint);
        return;
    }

    // Each emoji is 2 terminal columns; grid is width emojis wide
    let grid_display_width = game.width as u16 * 2;
    let grid_display_height = game.height as u16;
    let min_width = grid_display_width + 6;
    let min_height = grid_display_height + 6;

    if area.width < min_width || area.height < min_height {
        render_minigame_too_small(frame, area, "Vault Warden", min_width, min_height);
        return;
    }

    let layout = create_game_layout(
        frame,
        area,
        " Vault Warden ",
        BORDER_COLOR,
        grid_display_height,
        22,
        ctx,
    );

    render_grid(frame, layout.content, game);
    render_status_bar_content(frame, layout.status_bar, game);
    render_info_panel(frame, layout.info_panel, game);
}

/// Render the emoji grid.
fn render_grid(frame: &mut Frame, area: Rect, game: &VaultWardenGame) {
    let grid_display_width = game.width as u16 * 2;
    let grid_display_height = game.height as u16;

    // Center the grid
    let x_offset = area.x + (area.width.saturating_sub(grid_display_width)) / 2;
    let y_offset = area.y + (area.height.saturating_sub(grid_display_height)) / 2;

    for row in 0..game.height {
        let mut spans = Vec::new();

        for col in 0..game.width {
            let pos = (row, col);
            let emoji = if pos == game.player_pos {
                "\u{1F9D9}" // 🧙
            } else if game.has_crate_at(pos) {
                if game.is_goal(pos) {
                    "\u{2705}" // ✅ crate on goal
                } else if is_crate_deadlocked(game, pos) {
                    "\u{1F7E5}" // 🟥 deadlocked
                } else {
                    "\u{1F4E6}" // 📦 crate
                }
            } else if game.is_goal(pos) {
                "\u{2B50}" // ⭐ goal
            } else if game.grid[row][col] == crate::challenges::vault_warden::types::Cell::Wall {
                "\u{2B1C}" // ⬜ wall
            } else {
                "\u{2B1B}" // ⬛ floor
            };

            spans.push(Span::raw(emoji));
        }

        let line = Line::from(spans);
        let y = y_offset + row as u16;
        if y < area.y + area.height {
            frame.render_widget(
                Paragraph::new(vec![line]),
                Rect::new(x_offset, y, grid_display_width, 1),
            );
        }
    }
}

/// Render the status bar.
fn render_status_bar_content(frame: &mut Frame, area: Rect, game: &VaultWardenGame) {
    if render_forfeit_status_bar(frame, area, game.forfeit_pending) {
        return;
    }

    render_status_bar(
        frame,
        area,
        "Arranging relics...",
        BORDER_COLOR,
        &[
            ("[Arrows]", "Move"),
            ("[Z]", "Undo"),
            ("[R]", "Restart"),
            ("[Esc]", "Forfeit"),
        ],
    );
}

/// Render the info panel.
fn render_info_panel(frame: &mut Frame, area: Rect, game: &VaultWardenGame) {
    let inner = render_info_panel_frame(frame, area);
    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let mut lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("Difficulty ", Style::default().fg(Color::DarkGray)),
            Span::styled(game.difficulty.name(), Style::default().fg(BORDER_COLOR)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Grid   ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}×{}", game.width, game.height),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Moves  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}/{}", game.moves, game.move_limit),
                Style::default().fg(if game.moves > game.par {
                    Color::Yellow
                } else {
                    Color::White
                }),
            ),
        ]),
        Line::from(vec![
            Span::styled("Par    ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", game.par), Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Placed ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}/{}", game.crates_on_goals(), game.total_crates()),
                Style::default().fg(if game.crates_on_goals() == game.total_crates() {
                    Color::Green
                } else {
                    Color::White
                }),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Undos  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}/{}", game.undos_remaining, game.undos_max),
                Style::default().fg(if game.undos_remaining == 0 {
                    Color::Red
                } else {
                    Color::White
                }),
            ),
        ]),
    ];

    lines.truncate(inner.height as usize);
    frame.render_widget(Paragraph::new(lines), inner);
}

/// Render game over overlay.
fn render_game_over(
    frame: &mut Frame,
    area: Rect,
    game: &VaultWardenGame,
    show_dismiss_hint: bool,
) {
    use crate::challenges::menu::DifficultyInfo;

    let (result_type, title, message, reward) = match game.game_result {
        Some(VaultWardenResult::Win) => {
            let r = game.difficulty.reward();
            let reward_text = if r.prestige_ranks > 0 {
                format!(
                    "+{} Prestige Ranks, +{} Stormglass",
                    r.prestige_ranks, r.stormglass
                )
            } else {
                format!("+{} Stormglass", r.stormglass)
            };
            let msg = if game.moves <= game.par {
                format!("Solved in {} moves (par {}) \u{2605}", game.moves, game.par)
            } else {
                format!("Solved in {} moves (par {})", game.moves, game.par)
            };
            (
                GameResultType::Win,
                "VAULT SEALED!".to_string(),
                msg,
                reward_text,
            )
        }
        _ => (
            GameResultType::Loss,
            "VAULT BREACHED!".to_string(),
            format!("Exceeded move limit ({}/{})", game.moves, game.move_limit),
            "No penalty incurred.".to_string(),
        ),
    };
    render_game_over_overlay(
        frame,
        area,
        result_type,
        &title,
        &message,
        &reward,
        show_dismiss_hint,
    );
}
```

- [ ] **Step 2: Register the scene module in `src/ui/mod.rs`**

Add module declaration (after `pub mod runic_lights_scene;`):

```rust
pub mod vault_warden_scene;
```

Add rendering dispatch arm in the minigame match (after the RunicLights arm, around line 1374):

```rust
Some(ActiveMinigame::VaultWarden(game)) => {
    vault_warden_scene::render_vault_warden(frame, area, game, ctx, show_dismiss_hint);
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build`
Expected: Compiles (though the game can't be triggered yet).

- [ ] **Step 4: Commit**

```bash
git add src/ui/vault_warden_scene.rs src/ui/mod.rs
git commit -m "feat(vault-warden): add UI scene with emoji grid rendering"
```

---

### Task 4: Challenge Menu Integration

**Files:**
- Modify: `src/challenges/menu.rs`
- Modify: `src/ui/challenge_menu_scene.rs`

- [ ] **Step 1: Update menu.rs**

1. Add import at top (near line 16):
```rust
use super::vault_warden::{start_vault_warden_game, VaultWardenDifficulty};
```

2. Add `VaultWarden` to `ChallengeType` enum (after `RunicLights`):
```rust
VaultWarden,
```

3. Add icon in `ChallengeType::icon()` match:
```rust
ChallengeType::VaultWarden => "\u{1F512}", // 🔒
```

4. Add flavor text in `ChallengeType::discovery_flavor()` match:
```rust
ChallengeType::VaultWarden => "You discover a sealed vault deep in the dungeon. Stone pedestals await their relics — can you arrange them all?",
```

5. Add entry to `CHALLENGE_TABLE` (after RunicLights):
```rust
ChallengeWeight {
    challenge_type: ChallengeType::VaultWarden,
    weight: 18,
},
```

6. Add case to `accept_selected_challenge()` match:
```rust
ChallengeType::VaultWarden => {
    let d = VaultWardenDifficulty::from_index(difficulty_index);
    start_vault_warden_game(d, &mut rand::rng())
}
```

7. Add case to `create_challenge()` match:
```rust
ChallengeType::VaultWarden => PendingChallenge {
    challenge_type: ChallengeType::VaultWarden,
    title: "Vault Warden".to_string(),
    icon: "\u{1F512}",
    description: "A sealed chamber with stone pedestals — push each relic onto its pedestal to complete the lock.".to_string(),
},
```

- [ ] **Step 2: Update challenge_menu_scene.rs**

1. Add import (near line 12):
```rust
use crate::challenges::vault_warden::VaultWardenDifficulty;
```

2. Add difficulty selector arm (after `ChallengeType::RunicLights`):
```rust
ChallengeType::VaultWarden => {
    render_difficulty_selector(
        frame,
        chunks[2],
        &VaultWardenDifficulty::ALL,
        menu.selected_difficulty,
        stormglass_discovered,
    );
}
```

3. Add to the `difficulty_count` match (after RunicLights):
```rust
ChallengeType::VaultWarden => VaultWardenDifficulty::ALL.len(),
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build`

- [ ] **Step 4: Commit**

```bash
git add src/challenges/menu.rs src/ui/challenge_menu_scene.rs
git commit -m "feat(vault-warden): wire up challenge menu with discovery and difficulty selection"
```

---

### Task 5: Input Handling

**Files:**
- Modify: `src/input/minigame_input.rs`

- [ ] **Step 1: Add imports and input dispatch**

Add imports (near the other challenge imports):
```rust
use crate::challenges::vault_warden::{
    apply_vault_warden_result, process_input as process_vault_warden_input, VaultWardenInput,
};
```

Add match arm in `handle_minigame()` (after the RunicLights arm):
```rust
Some(ActiveMinigame::VaultWarden(game)) => {
    if game.game_result.is_some() {
        state.last_minigame_win = apply_vault_warden_result(state);
        return result_for_challenge(&state.last_minigame_win);
    }
    let input = match key.code {
        KeyCode::Up => VaultWardenInput::Up,
        KeyCode::Down => VaultWardenInput::Down,
        KeyCode::Left => VaultWardenInput::Left,
        KeyCode::Right => VaultWardenInput::Right,
        KeyCode::Char('z') | KeyCode::Char('Z') => VaultWardenInput::Undo,
        KeyCode::Char('r') | KeyCode::Char('R') => VaultWardenInput::Restart,
        KeyCode::Esc => VaultWardenInput::Forfeit,
        _ => VaultWardenInput::Other,
    };
    process_vault_warden_input(game, input);
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo build`

- [ ] **Step 3: Commit**

```bash
git add src/input/minigame_input.rs
git commit -m "feat(vault-warden): wire up keyboard input handling"
```

---

## Chunk 3: Achievements, Stormglass, and Debug Integration

### Task 6: Achievement Integration

**Files:**
- Modify: `src/achievements/milestones.rs`
- Modify: `src/achievements/types.rs`
- Modify: `src/achievements/data.rs`
- Modify: `src/achievements/handlers.rs`
- Modify: `src/ui/achievement_details.rs`

- [ ] **Step 1: Add MinigameType variant**

In `src/achievements/milestones.rs`, add to the `MinigameType` enum (after `RunicLights`):
```rust
VaultWarden,
```

- [ ] **Step 2: Add AchievementId variants**

In `src/achievements/types.rs`, add after the `RunicLightsMaster` variant:
```rust
// Challenge achievements - Vault Warden
VaultWardenNovice,
VaultWardenApprentice,
VaultWardenJourneyman,
VaultWardenMaster,
```

- [ ] **Step 3: Add achievement definitions**

In `src/achievements/data.rs`, add after the Runic Lights achievement definitions:
```rust
// ═══════════════════════════════════════════════════════════════
// Vault Warden
// ═══════════════════════════════════════════════════════════════
AchievementDef {
    id: AchievementId::VaultWardenNovice,
    name: "Vault Warden Novice",
    description: "Seal the vault on Novice difficulty",
    category: Category::Challenge,
    score: 5,
    hidden: false,
},
AchievementDef {
    id: AchievementId::VaultWardenApprentice,
    name: "Vault Warden Apprentice",
    description: "Seal the vault on Apprentice difficulty",
    category: Category::Challenge,
    score: 10,
    hidden: false,
},
AchievementDef {
    id: AchievementId::VaultWardenJourneyman,
    name: "Vault Warden Journeyman",
    description: "Seal the vault on Journeyman difficulty",
    category: Category::Challenge,
    score: 15,
    hidden: false,
},
AchievementDef {
    id: AchievementId::VaultWardenMaster,
    name: "Vault Warden Master",
    description: "Seal the vault on Master difficulty",
    category: Category::Challenge,
    score: 25,
    hidden: false,
},
```

- [ ] **Step 4: Add handler mapping**

In `src/achievements/handlers.rs`, add to the minigame win handler match (after RunicLights entries):
```rust
(MinigameType::VaultWarden, MinigameDifficulty::Novice) => {
    Some(AchievementId::VaultWardenNovice)
}
(MinigameType::VaultWarden, MinigameDifficulty::Apprentice) => {
    Some(AchievementId::VaultWardenApprentice)
}
(MinigameType::VaultWarden, MinigameDifficulty::Journeyman) => {
    Some(AchievementId::VaultWardenJourneyman)
}
(MinigameType::VaultWarden, MinigameDifficulty::Master) => {
    Some(AchievementId::VaultWardenMaster)
}
```

- [ ] **Step 5: Update achievement_details.rs**

In `src/ui/achievement_details.rs`, add to the challenge wins display list (after Runic Lights entry):
```rust
(
    "Vault Warden",
    [
        AchievementId::VaultWardenNovice,
        AchievementId::VaultWardenApprentice,
        AchievementId::VaultWardenJourneyman,
        AchievementId::VaultWardenMaster,
    ],
),
```

- [ ] **Step 6: Verify it compiles**

Run: `cargo build`

- [ ] **Step 7: Commit**

```bash
git add src/achievements/ src/ui/achievement_details.rs
git commit -m "feat(vault-warden): add achievement tracking for all difficulty tiers"
```

---

### Task 7: Stormglass and Debug Menu Integration

**Files:**
- Modify: `src/stormglass/spending.rs`
- Modify: `src/utils/debug_menu.rs`

- [ ] **Step 1: Update stormglass spending.rs**

1. Add `VaultWarden` to the `TRIAL_CHALLENGE_TYPES` array. Update the array length from `12` to `13`:
```rust
const TRIAL_CHALLENGE_TYPES: [ChallengeType; 13] = [
    // ... existing 12 entries ...
    ChallengeType::VaultWarden,
];
```

2. Add to `challenge_type_name()` match:
```rust
ChallengeType::VaultWarden => "Vault Warden",
```

- [ ] **Step 2: Update debug_menu.rs**

1. Add `TriggerVaultWardenChallenge` variant to the `DebugAction` enum.

2. Add to `CHALLENGE_ACTIONS` array:
```rust
DebugAction::TriggerVaultWardenChallenge,
```

3. Add to `DEBUG_ACTIONS` array (the full list).

4. Add sort key in `sort_key()` match:
```rust
DebugAction::TriggerVaultWardenChallenge => 614, // After RunicLights
```

5. Add label in `label()` match:
```rust
DebugAction::TriggerVaultWardenChallenge => "Trigger Vault Warden Challenge",
```

6. Add execution in `execute()` match:
```rust
DebugAction::TriggerVaultWardenChallenge => trigger_vault_warden_challenge(state),
```

7. Add the trigger function:
```rust
fn trigger_vault_warden_challenge(state: &mut GameState) -> &'static str {
    if state
        .challenge_menu
        .has_challenge(&ChallengeType::VaultWarden)
    {
        return "Vault Warden challenge already pending!";
    }
    state
        .challenge_menu
        .add_challenge(create_challenge(&ChallengeType::VaultWarden));
    "Vault Warden challenge added!"
}
```

- [ ] **Step 3: Run full test suite**

Run: `cargo test`
Expected: All tests pass.

- [ ] **Step 4: Run clippy**

Run: `cargo clippy --all-targets -- -D warnings`
Expected: No warnings.

- [ ] **Step 5: Run formatter**

Run: `cargo fmt`

- [ ] **Step 6: Commit**

```bash
git add src/stormglass/spending.rs src/utils/debug_menu.rs
git commit -m "feat(vault-warden): add stormglass trial support and debug menu trigger"
```

---

### Task 8: Final Verification

- [ ] **Step 1: Run make check**

Run: `make check`
Expected: All CI checks pass (fmt, clippy, tests, build, audit).

- [ ] **Step 2: Manual play test**

Run: `cargo run`
1. Open debug menu, trigger Vault Warden challenge
2. Accept the challenge on each difficulty
3. Verify: emoji rendering looks correct, movement works, crate pushing works
4. Verify: undo works and decrements budget, restart resets everything
5. Verify: deadlocked crates show red
6. Verify: winning shows "VAULT SEALED!" overlay
7. Verify: forfeit double-Esc works
8. Verify: move limit triggers loss

- [ ] **Step 3: Final commit if any fixes needed**

```bash
git add -A
git commit -m "fix(vault-warden): polish from play testing"
```
