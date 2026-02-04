# Gomoku Challenge Design

## Overview

Add Gomoku (Five in a Row) as a third challenge minigame alongside Chess and Morris.

## Game Rules

- **Board**: 15×15 grid
- **Objective**: First to get 5+ stones in a row (horizontal, vertical, or diagonal) wins
- **Turn order**: Human plays first (white), AI responds (red)
- **Stones**: Once placed, never moved or removed
- **Draw**: Board fills with no winner (rare)

## Visual Design

### Colors
- Human: White filled circle (●)
- AI: Light red filled circle (●)
- Empty: Dot (·)
- Cursor: Bracketed [●] or [·]

### Layout
Compact grid without coordinates, ~30 chars wide:

```
┌─ Gomoku ──────────────────┐┌─ Info ─────────────┐
│  · · · · · · · · · · · · ·││ RULES              │
│  · · · · · · · · · · · · ·││ Place stones. First│
│  · · · · · · · · · · · · ·││ to get 5 in a row  │
│  · · · · · · ● · · · · · ·││ wins.              │
│  · · · · · ● ○ ● · · · · ·││                    │
│  · · · · · · ● · · · · · ·││ Difficulty: Novice │
│  · · · · · · · · · · · · ·││                    │
│  · · · · · · · · · · · · ·││ [Arrows] Move      │
│  · · · · · · · · · · · · ·││ [Enter] Place      │
│  · · · · · · · · · · · · ·││ [Esc] Forfeit      │
└───────────────────────────┘└────────────────────┘
```

## Difficulty Levels

| Level | Search Depth | Prestige Reward |
|-------|--------------|-----------------|
| Novice | 2 | +1 |
| Apprentice | 3 | +2 |
| Journeyman | 4 | +3 |
| Master | 5 | +5 |

## AI Implementation

### Algorithm
Minimax with alpha-beta pruning.

### Evaluation Heuristics
Scan all lines (rows, columns, diagonals) for patterns:

| Pattern | Score |
|---------|-------|
| Five in a row | ±100,000 (win/loss) |
| Open four (both ends open) | ±10,000 |
| Closed four (one end blocked) | ±1,000 |
| Open three | ±500 |
| Closed three | ±100 |
| Open two | ±50 |
| Center proximity | Small bonus |

Positive scores favor AI, negative favor human.

### Optimization
- Alpha-beta pruning
- Consider only moves within 2 spaces of existing stones
- Early termination on win detection

## Data Structures

```rust
pub struct GomokuGame {
    pub board: [[Option<Player>; 15]; 15],
    pub cursor: (usize, usize),
    pub current_player: Player,
    pub difficulty: GomokuDifficulty,
    pub game_result: Option<GomokuResult>,
    pub ai_thinking: bool,
    pub move_history: Vec<(usize, usize)>,
}

pub enum GomokuDifficulty {
    Novice,      // depth 2
    Apprentice,  // depth 3
    Journeyman,  // depth 4
    Master,      // depth 5
}

pub enum GomokuResult {
    Win,
    Loss,
    Draw,
}
```

## Integration

### Files to Create
- `src/gomoku.rs` - Game state, types, constants
- `src/gomoku_logic.rs` - AI, move validation, win detection
- `src/ui/gomoku_scene.rs` - Board rendering, help panel

### Files to Modify
- `src/challenge_menu.rs` - Add `ChallengeType::Gomoku`, update weights
- `src/game_state.rs` - Add `active_gomoku: Option<GomokuGame>`
- `src/main.rs` - Input handling, AI tick processing, result handling
- `src/debug_menu.rs` - Add Gomoku trigger option
- `src/lib.rs` - Export new modules
- `src/ui/mod.rs` - Export gomoku_scene

### Challenge Discovery
- Same ~2hr average as Chess/Morris
- Requires P1+
- Equal weight in challenge table (33/33/33 split)

## Controls

- Arrow keys: Move cursor
- Enter: Place stone
- Esc: Forfeit game
