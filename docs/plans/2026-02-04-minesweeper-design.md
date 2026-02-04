# Minesweeper: Trap Detection - Design Document

## Overview

A single-player puzzle challenge where the player reveals safe tiles on a grid while avoiding hidden traps (mines). Themed as "Trap Detection" - disarming traps in a dungeon corridor.

## Difficulty Levels

| Difficulty | Grid | Mines | Density | Reward |
|------------|------|-------|---------|--------|
| Novice | 9×9 | 10 | 12% | +50% XP |
| Apprentice | 12×12 | 25 | 17% | +75% XP |
| Journeyman | 16×16 | 40 | 16% | +100% XP |
| Master | 20×16 | 60 | 19% | +1 Prestige, +200% XP |

## Core Mechanics

### First Click Safety
Mines are placed *after* the first reveal, avoiding the clicked cell and its 8 neighbors. This guarantees:
- First click never hits a mine
- First click opens a clearing (if the area is sparse)

### Cell States
Each cell has:
- `has_mine: bool` - contains a trap
- `revealed: bool` - player has uncovered this cell
- `flagged: bool` - player has marked as suspected trap
- `adjacent_mines: u8` - count of neighboring mines (0-8)

### Reveal Logic
1. If flagged → do nothing (prevents accidental reveal)
2. If mine → game over (Loss), reveal all mines
3. If adjacent_mines > 0 → reveal that cell only
4. If adjacent_mines == 0 → flood-fill reveal all connected zeros and their borders

### Win Condition
All non-mine cells revealed (unrevealed count == mine count).

### Controls
- `[Arrows]` Move cursor
- `[Enter]` Reveal cell
- `[F]` Toggle flag
- `[Esc] [Esc]` Forfeit (consistent with other challenges)

## Data Structures

### MinesweeperDifficulty
```rust
pub enum MinesweeperDifficulty {
    Novice,     // 9×9, 10 mines
    Apprentice, // 12×12, 25 mines
    Journeyman, // 16×16, 40 mines
    Master,     // 20×16, 60 mines
}
```

### Cell
```rust
pub struct Cell {
    pub has_mine: bool,
    pub revealed: bool,
    pub flagged: bool,
    pub adjacent_mines: u8,
}
```

### MinesweeperResult
```rust
pub enum MinesweeperResult {
    Win,  // All safe cells revealed
    Loss, // Revealed a mine
}
```

### MinesweeperGame
```rust
pub struct MinesweeperGame {
    pub grid: Vec<Vec<Cell>>,
    pub width: usize,
    pub height: usize,
    pub cursor: (usize, usize),
    pub difficulty: MinesweeperDifficulty,
    pub game_result: Option<MinesweeperResult>,
    pub first_click_done: bool,
    pub total_mines: u16,
    pub flags_placed: u16,
    pub forfeit_pending: bool,
}
```

## UI Design

### Layout
- Left panel: Grid display
- Right panel: Info (difficulty, grid size, mines remaining, controls)

### Cell Rendering (ASCII)
```
Unrevealed:  ░
Flagged:     ⚑
Revealed 0:  ·
Revealed 1-8: 1-8 (colored)
Mine (loss): *
```

### Number Colors
- 1: Blue
- 2: Green
- 3: Red
- 4: Dark Blue
- 5: Dark Red
- 6: Cyan
- 7: Gray
- 8: White

### Cursor
Current cell highlighted with brackets: `[░]`

### Right Panel
```
Trap Detection

Difficulty: Journeyman
Grid: 16×16
Traps: 40

Remaining: 37 ⚑

[Arrows] Move
[Enter] Reveal
[F] Flag
[Esc] Forfeit
```

### Game Over
- Win: "Area Secured!" (green), show reward
- Loss: "Trap Triggered!" (red), reveal all mines

## Challenge Integration

### Challenge Menu Entry
```rust
ChallengeType::Minesweeper => PendingChallenge {
    challenge_type: ChallengeType::Minesweeper,
    title: "Minesweeper: Trap Detection".to_string(),
    icon: "⚠",
    description: "A weathered scout beckons you toward a ruined corridor. \
        'The floor's rigged with pressure plates,' she warns, pulling out a \
        worn map. 'One wrong step and...' She makes an explosive gesture. \
        'Help me chart the safe path. Probe carefully—the numbers tell you \
        how many traps lurk nearby.'".to_string(),
}
```

### Rewards (DifficultyInfo impl)
```rust
impl DifficultyInfo for MinesweeperDifficulty {
    fn reward(&self) -> ChallengeReward {
        match self {
            Self::Novice => ChallengeReward { xp_percent: 50, ..Default::default() },
            Self::Apprentice => ChallengeReward { xp_percent: 75, ..Default::default() },
            Self::Journeyman => ChallengeReward { xp_percent: 100, ..Default::default() },
            Self::Master => ChallengeReward { prestige_ranks: 1, xp_percent: 200, ..Default::default() },
        }
    }
}
```

### Discovery Weight
Equal chance with other challenges: Chess 25, Morris 25, Gomoku 25, Minesweeper 25.

## Files to Create
- `src/minesweeper.rs` - data structures
- `src/minesweeper_logic.rs` - game logic
- `src/ui/minesweeper_scene.rs` - rendering

## Files to Modify
- `src/challenge_menu.rs` - add ChallengeType::Minesweeper, DifficultyInfo impl
- `src/game_state.rs` - add active_minesweeper field
- `src/main.rs` - input handling, game loop
- `src/debug_menu.rs` - add debug trigger
- `src/lib.rs` - export modules

## Out of Scope (YAGNI)
- Timer/time bonuses
- Partial credit for losses
- Chord clicking (reveal neighbors when flag count matches)
- Custom grid sizes
- Difficulty beyond Master
