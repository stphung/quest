# Nine Men's Morris - Design Document

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add Nine Men's Morris as a challenge type, following the same patterns as chess.

**Architecture:** New game module with data structures, logic/AI, and UI scene. Integrates with existing challenge menu system. Discovery rate matches chess (~2 hour average).

**Tech Stack:** Pure Rust implementation (no external crate), Ratatui for UI, minimax AI with alpha-beta pruning.

---

## Overview

Nine Men's Morris is a strategy board game where two players take turns placing and moving pieces, trying to form "mills" (three pieces in a row) to capture opponent pieces. The game ends when one player is reduced to 2 pieces or cannot move.

### Game Phases

1. **Placing** - Players alternate placing 9 pieces each on empty points
2. **Moving** - Once all pieces placed, slide pieces along lines to adjacent points
3. **Flying** - When down to 3 pieces, that player can move to any empty point

### Win Conditions

- Reduce opponent to 2 pieces
- Block all opponent moves

---

## Architecture

### New Files

| File | Purpose |
|------|---------|
| `src/morris.rs` | Data structures: MorrisGame, MorrisDifficulty, MorrisResult, MorrisPhase |
| `src/morris_logic.rs` | Game logic: discovery, legal moves, mill detection, AI, tick processing |
| `src/ui/morris_scene.rs` | UI rendering: board, pieces, cursor, help panel, game-over overlay |

### Modified Files

| File | Change |
|------|--------|
| `src/challenge_menu.rs` | Add `ChallengeType::Morris` variant |
| `src/game_state.rs` | Add `active_morris: Option<MorrisGame>` field |
| `src/main.rs` | Handle Morris input, tick processing, rendering |
| `src/ui/mod.rs` | Export morris_scene module |

---

## Data Structures

### MorrisDifficulty

Same as chess - 4 tiers with identical names and prestige rewards:

| Difficulty | Search Depth | Random Move % | Prestige Reward |
|------------|--------------|---------------|-----------------|
| Novice | 1-ply | 50% | +1 |
| Apprentice | 1-ply | 0% | +2 |
| Journeyman | 2-ply | 0% | +3 |
| Master | 3-ply | 0% | +5 |

### MorrisResult

- `Win` - Player reduced AI to 2 pieces or blocked all moves
- `Loss` - AI reduced player to 2 pieces or blocked all moves
- `Forfeit` - Player quit the game

No draws - Nine Men's Morris always ends with a winner.

### MorrisPhase

- `Placing` - First 18 moves (9 per player)
- `Moving` - Normal movement along lines
- `Flying` - Player with 3 pieces can move anywhere

### MorrisGame

```rust
pub struct MorrisGame {
    pub board: [Option<Player>; 24],      // 24 positions
    pub phase: MorrisPhase,
    pub pieces_to_place: (u8, u8),        // (player, AI) remaining
    pub difficulty: MorrisDifficulty,
    pub cursor: usize,                     // 0-23 position index
    pub selected_position: Option<usize>,  // For moving phase
    pub must_capture: bool,                // After forming a mill
    pub game_result: Option<MorrisResult>,
    pub ai_thinking: bool,
    pub ai_think_ticks: u32,
    pub ai_think_target: u32,
    pub ai_pending_move: Option<MorrisMove>,
    pub player_is_first: bool,             // Player moves first
}
```

---

## Board Representation

The 24 positions are indexed 0-23:

```
0-----------1-----------2
|           |           |
|   3-------4-------5   |
|   |       |       |   |
|   |   6---7---8   |   |
|   |   |       |   |   |
9---10--11      12--13--14
|   |   |       |   |   |
|   |   15--16--17  |   |
|   |       |       |   |
|   18------19------20  |
|           |           |
21----------22----------23
```

### Mill Lines (16 total)

```rust
const MILLS: [[usize; 3]; 16] = [
    // Outer square
    [0, 1, 2], [2, 14, 23], [21, 22, 23], [0, 9, 21],
    // Middle square
    [3, 4, 5], [5, 13, 20], [18, 19, 20], [3, 10, 18],
    // Inner square
    [6, 7, 8], [8, 12, 17], [15, 16, 17], [6, 11, 15],
    // Connecting lines (spokes)
    [1, 4, 7], [12, 13, 14], [16, 19, 22], [9, 10, 11],
];
```

### Adjacency Lists

Each position has 2-4 adjacent positions (stored as constant arrays).

---

## Game Logic

### Legal Move Generation

**Placing phase:** Any empty position (indices where `board[i].is_none()`)

**Moving phase:**
- Select own piece
- Move to adjacent empty position

**Flying phase:** (when player has exactly 3 pieces)
- Select own piece
- Move to any empty position

**Capture phase:** (after forming a mill)
- Select any opponent piece NOT in a mill
- Exception: if all opponent pieces are in mills, any can be captured

### Mill Detection

After each placement or move, check if the position completes any mill by checking all mill lines containing that position.

### Win Condition Check

After each turn:
1. Count opponent pieces - if ≤ 2, current player wins
2. Check if opponent has any legal moves - if not, current player wins

---

## AI Implementation

### Evaluation Function

Score based on:
- Piece count difference (heavily weighted)
- Mill count
- Potential mills (2 pieces + empty third)
- Mobility (number of legal moves)

### Minimax with Alpha-Beta Pruning

- Depth varies by difficulty (1/1/2/3 ply)
- Novice: 50% chance of random legal move instead
- Same "thinking delay" pattern as chess for natural feel

---

## UI Design

### Board Layout

```
┌─ Nine Men's Morris ─────────────────────────────────┐
│                                                     │
│   ●───────────●───────────○    ┌─ How to Play ────┐ │
│   │           │           │    │ 1. PLACE: Put 9  │ │
│   │   ·───────·───────·   │    │ 2. MOVE: Slide   │ │
│   │   │       │       │   │    │ 3. Three adjacent│ │
│   │   │   ·───·───·   │   │    │    → capture one │ │
│   │   │   │       │   │   │    │    of theirs     │ │
│   ●───·───·       ·───·───○    │ 4. Win: 2 pieces │ │
│   │   │   │       │   │   │    │    or blocked    │ │
│   │   │   ·───·───·   │   │    │ (3 left = fly)   │ │
│   │   │       │       │   │    └──────────────────┘ │
│   │   ·───────·───────·   │                         │
│   │           │           │    You: ● × 5          │
│   ○───────────·───────────·    Foe: ○ × 4          │
│                                                     │
│              Select a piece to capture              │
│        [Arrows] Move  [Enter] Select  [Esc] Forfeit │
└─────────────────────────────────────────────────────┘
```

### Visual Elements

| Symbol | Meaning |
|--------|---------|
| `●` | Player piece (bright white) |
| `○` | AI piece (dim gray) |
| `·` | Empty position |
| `[●]` or `[·]` | Cursor position |
| `<●>` | Selected piece (for moving) |

### Status Messages

- Placing: "Place a piece"
- Moving: "Select piece to move" / "Select destination"
- Capturing: "Select a piece to capture"
- AI turn: "⠋ Opponent is thinking..."
- Forfeit: "Forfeit game?" with confirmation controls

### Help Panel

Always visible to the right of the board:
```
┌─ How to Play ────┐
│ 1. PLACE: Put 9  │
│ 2. MOVE: Slide   │
│ 3. Three adjacent│
│    → capture one │
│    of theirs     │
│ 4. Win: 2 pieces │
│    or blocked    │
│ (3 left = fly)   │
└──────────────────┘
```

---

## Challenge Integration

### Discovery

- Same rate as chess: `MORRIS_DISCOVERY_CHANCE = 0.000014` (~2 hour average)
- Requirements: P1+, not in dungeon/fishing/chess/morris, no pending morris challenge
- Function: `try_discover_morris()`

### Challenge Description

```
A weathered board sits between you and a cloaked stranger.
"Do you know the Miller's Game?" they ask, gesturing at
the carved lines. Get three adjacent to capture pieces.
Reduce your opponent to two to win.
```

### ChallengeType Enum

```rust
pub enum ChallengeType {
    Chess,
    Morris,  // New variant
}
```

---

## Testing Plan

### Unit Tests

- Mill detection: All 16 mills correctly identified
- Adjacency: Each position's neighbors correct
- Legal moves: Placing, moving, flying phases
- Capture rules: Can't capture pieces in mills (unless all are)
- Win conditions: 2 pieces, no moves

### AI Tests

- Returns only legal moves
- Depth limits respected
- Novice randomness works

### Integration Tests

- Discovery requirements (P1+, not in other activities)
- Challenge menu integration
- Game start/end flow
- Prestige rewards applied correctly

---

## No Stats Tracking

Unlike chess, Nine Men's Morris will not track historical stats. Games are played for prestige rewards only.
