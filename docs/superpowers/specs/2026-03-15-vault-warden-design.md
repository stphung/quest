# Vault Warden — Design Spec

## Overview

Vault Warden is a Sokoban puzzle challenge where the player pushes crates onto goal squares inside a vault. Levels are curated from the public-domain Microban set (~155 levels) embedded as const data, categorized by difficulty tier.

## Name & Theme

- **Challenge name:** Vault Warden
- **Flavor:** You guard an ancient vault — arrange relics (crates) onto their pedestals (goals) to seal the chamber.
- **Border color:** Amber/gold `Rgb(180, 140, 40)` — vault theme.

## Core Mechanics

### Movement & Pushing
- Arrow keys move the player one cell at a time.
- Walking into a crate pushes it one cell in the movement direction.
- A crate cannot be pushed if blocked by a wall or another crate.
- Crates can only be pushed, never pulled.

### Undo
- Press `Z` to undo the last move (reverses both player and any pushed crate).
- Limited undo budget per difficulty tier (see Difficulty section).
- Undo does NOT decrement the move counter — moves stay counted.

### Restart
- Press `R` to restart the current level from scratch.
- Resets moves to 0 and undos to full budget.

### Win Condition
- All crates are on goal squares simultaneously.
- Par star awarded if total moves <= optimal solution length.

### Loss Conditions
- Exceeding the move limit (optimal × 2.5, rounded up).
- Forfeit (standard double-Esc pattern).

### Deadlock Detection
- **Corner deadlock:** Crate in a corner (two adjacent walls) where neither wall-adjacent cell is a goal.
- **Wall-edge deadlock:** Crate against a continuous straight wall segment with no goal anywhere along that segment (the run of consecutive wall cells in one direction from the crate's position).
- Deadlocked crates are visually highlighted (red) but the game does not auto-end — player can undo or restart.

## Difficulty Tiers

| Tier | Grid Range | Crates | Undos | Move Limit |
|------|-----------|--------|-------|------------|
| Novice | 5×5 – 7×7 | 1–2 | 5 | optimal × 2.5 |
| Apprentice | 6×6 – 8×8 | 2–3 | 3 | optimal × 2.5 |
| Journeyman | 7×7 – 9×9 | 3–4 | 2 | optimal × 2.5 |
| Master | 9×9 – 11×11 | 4–5 | 1 | optimal × 2.5 |

Levels are pre-categorized into tiers based on grid size and crate count. Each game randomly selects one level from the chosen tier's pool.

## Level Data

### Source
Microban by David W. Skinner — 155 levels, free for personal use and redistribution with attribution. Attribution included as a comment in the levels file. Levels are compact (5×5 to 11×11), well-designed, and span a clean difficulty curve from trivial to challenging.

### Optimal Move Counts
Each level's `optimal_moves` value comes from published Microban solutions. These are pre-computed and embedded alongside the level data. The move limit is `ceil(optimal_moves * 2.5)`.

### Tier Categorization
Levels are pre-assigned to tiers based on grid size and crate count. Ambiguous cases (e.g., 7×7 with 2 crates) are assigned to the lower tier. The categorization is done once when embedding the levels, not at runtime.

### Storage Format
Levels are stored as const string slices in a dedicated module (`src/challenges/vault_warden/levels.rs`), using standard Sokoban notation:
- `#` wall
- ` ` (space) floor
- `$` crate
- `.` goal
- `*` crate on goal
- `@` player
- `+` player on goal

Each level includes metadata: tier classification, optimal solution length (for par), and move limit.

```rust
pub struct VaultWardenLevel {
    pub data: &'static str,
    pub optimal_moves: u16,
}

pub const NOVICE_LEVELS: &[VaultWardenLevel] = &[
    VaultWardenLevel { data: "...", optimal_moves: 8 },
    // ...
];
```

## Rendering

### Emoji Grid
Full emoji rendering — each cell is one emoji character (2 terminal columns):

| Element | Emoji | Description |
|---------|-------|-------------|
| Wall | `⬜` | White square |
| Floor | `⬛` | Black square |
| Crate | `📦` | Box |
| Goal | `⭐` | Star |
| Crate on goal | `✅` | Green check |
| Player | `🧙` | Wizard |
| Player on goal | `🧙` | Wizard (same) |
| Deadlocked crate | `🟥` | Red square |

### Layout
Uses `create_game_layout()` from `game_common.rs` with:
- Content area: the emoji grid, centered
- Info panel: difficulty, grid size, moves/par, undos remaining, crates placed
- Status bar: controls hint

### Info Panel Content
```
Difficulty  [tier name]

Grid        [W×H]

Moves       [current]/[limit]
Par         [optimal]

Placed      [on_goal]/[total_crates]

Undos       [remaining]/[max]
```

### Status Bar
```
[Arrows] Move  [Z] Undo  [R] Restart  [Esc] Forfeit
```

### Game Over Overlay
Uses `render_game_over_overlay()`:
- **Win:** "VAULT SEALED!" / "Solved in N moves (par P)" or with star if at/under par
- **Loss:** "VAULT BREACHED!" / "Exceeded move limit (N/M)"

## Controls

| Key | Action |
|-----|--------|
| Arrow keys | Move player / push crate |
| Z | Undo last move |
| R | Restart level |
| Esc (first) | Set forfeit pending |
| Esc (second) | Confirm forfeit (loss) |
| Any key (during forfeit pending) | Cancel forfeit |
| Any key (on game over) | Dismiss result |

## Rewards

Standard challenge reward curve via `DifficultyInfo` trait:

| Tier | Stormglass | Prestige Ranks |
|------|-----------|----------------|
| Novice | 400 | 0 |
| Apprentice | 1,200 | 0 |
| Journeyman | 3,000 | 1 |
| Master | 6,000 | 2 |

Matches Runic Lights reward curve (similar puzzle complexity).

## Data Structures

### Game State
```rust
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
}

pub enum Cell {
    Wall,
    Floor,
    Goal,
}

pub struct MoveRecord {
    pub player_from: (usize, usize),
    pub pushed_crate: Option<CratePush>,
}

pub struct CratePush {
    pub from: (usize, usize),
    pub to: (usize, usize),
}

pub enum VaultWardenResult {
    Win,
    Loss,
}
```

Crate and goal positions are tracked separately from the grid so the grid stores only static terrain (Wall vs Floor). Goals are in the grid as `Cell::Goal` for rendering convenience but also tracked in `goal_positions` for quick win-condition checks.

### Input Enum
```rust
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
```

## Integration Points

Per the `add-challenge` skill, the following files are touched:

### New Files (5)
1. `src/challenges/vault_warden/mod.rs`
2. `src/challenges/vault_warden/types.rs`
3. `src/challenges/vault_warden/logic.rs`
4. `src/challenges/vault_warden/levels.rs`
5. `src/ui/vault_warden_scene.rs`

### Modified Files (11+)
Per the add-challenge skill's 15-point checklist — covers challenges/mod.rs, challenges/menu.rs, input/minigame_input.rs, ui/mod.rs, achievements/milestones.rs, stormglass/spending.rs, ui/achievement_details.rs, utils/debug_menu.rs, and VARIANT_COUNT updates.

### Discovery Weight
Weight: 18 in `CHALLENGE_TABLE` (similar to other puzzle challenges like Minesweeper at 18, Runic Lights at 20).

## Non-Goals
- No procedural level generation (curated only).
- No complex deadlock detection (frozen groups, 2×2 blocks) — basic corner/wall-edge only.
- No level editor.
- No level progression (each game is a single random level from the tier).
- Not a real-time game (turn-based, no tick function needed).
