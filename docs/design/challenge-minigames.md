# Challenge Minigames Design

This document describes the 6 challenge minigames as implemented. All challenges share a common framework: discovery via RNG, difficulty selection, prestige/XP rewards, and a forfeit pattern (double-Esc to quit).

## Common Framework

### Discovery

- **Chance**: 0.000014 per tick (~2 hour average at 10 ticks/sec)
- **Requirement**: P1+ (not in dungeon, fishing, or another minigame)
- **Haven Library bonus**: Up to +50% discovery rate at T3
- **Weighted distribution**: Not all challenges are equally likely

| Challenge | Weight | ~Probability |
|-----------|--------|--------------|
| Minesweeper | 30 | 27% |
| Rune | 25 | 23% |
| Gomoku | 20 | 18% |
| Morris | 15 | 14% |
| Chess | 10 | 9% |
| Go | 10 | 9% |

### Difficulty Tiers

All challenges use 4 difficulty levels: Novice, Apprentice, Journeyman, Master. Higher difficulties provide better rewards.

### Forfeit Pattern

All interactive minigames: first Esc sets `forfeit_pending`, second Esc confirms forfeit, any other key cancels.

### Reward Summary

| Challenge | Novice | Apprentice | Journeyman | Master |
|-----------|--------|------------|------------|--------|
| Chess | +1 PR | +2 PR | +3 PR | +5 PR |
| Go | +1 PR | +2 PR | +3 PR | +5 PR |
| Gomoku | +75% XP | +100% XP | +1 PR, +50% XP | +2 PR, +100% XP |
| Morris | +50% XP | +100% XP | +150% XP | +1 FR, +200% XP |
| Minesweeper | +50% XP | +75% XP | +100% XP | +1 PR, +200% XP |
| Rune | +25% XP | +50% XP | +1 FR, +75% XP | +1 PR, +2 FR |

PR = Prestige Rank, FR = Fishing Rank, XP% = percentage of current level's XP requirement.

---

## Chess

**Theme**: "A hooded challenger sits before you, pieces already arranged."

### Rules

Standard chess rules. Uses the `chess-engine` crate (v0.1) for move validation and AI.

### AI

Minimax search with difficulty-based depth:

| Difficulty | Depth | Random Move % | Est. ELO | Prestige Reward |
|------------|-------|---------------|----------|-----------------|
| Novice | 1-ply | 50% | ~500 | +1 |
| Apprentice | 1-ply | 0% | ~800 | +2 |
| Journeyman | 2-ply | 0% | ~1100 | +3 |
| Master | 3-ply | 0% | ~1350 | +5 |

### UI

Standard 8x8 board. Cursor navigation, piece selection/movement with Enter. Shows legal move highlights. AI "thinking" delay for natural feel.

### Stats

Chess tracks persistent stats across prestiges (wins, losses, draws per difficulty).

---

## Go (Territory Control)

**Theme**: "Territory Control" — a 9x9 Go board.

### Rules

- 9x9 board only (intersections, not squares)
- Chinese scoring (stones on board + surrounded empty points)
- Standard ko rule (cannot immediately recapture)
- Suicide illegal
- Game ends after two consecutive passes
- Player can also resign

### AI — MCTS (Monte Carlo Tree Search)

No external crate — pure Rust implementation.

| Difficulty | Simulations | Est. Strength | Prestige Reward |
|------------|-------------|---------------|-----------------|
| Novice | 500 | ~20 kyu | +1 |
| Apprentice | 2,000 | ~17 kyu | +2 |
| Journeyman | 8,000 | ~14 kyu | +3 |
| Master | 20,000 | ~12 kyu | +5 |

**MCTS phases**: Selection (UCT), Expansion, Simulation (random playout with heuristics), Backpropagation.

**Playout heuristics**: Respond to atari, avoid self-atari, avoid filling own eyes.

### UI

Grid with intersections. `●` = player (Black), `○` = AI (White). Cursor highlights intersection. P to pass, Esc to resign.

---

## Gomoku (Five in a Row)

**Theme**: A 15x15 board game — first to get 5 stones in a row wins.

### Rules

- 15x15 grid
- Human plays first (white), AI responds (red)
- Stones placed permanently — never moved or removed
- Win: 5+ in a row (horizontal, vertical, or diagonal)
- Draw: Board fills with no winner (rare)

### AI — Minimax with Alpha-Beta Pruning

| Difficulty | Search Depth | Prestige Reward |
|------------|-------------|-----------------|
| Novice | 2 | +75% XP |
| Apprentice | 3 | +100% XP |
| Journeyman | 4 | +1 PR, +50% XP |
| Master | 5 | +2 PR, +100% XP |

**Evaluation heuristics** (scan lines for patterns):
- Five in a row: ±100,000
- Open four: ±10,000
- Closed four: ±1,000
- Open three: ±500
- Closed three: ±100
- Open two: ±50
- Center proximity: small bonus

**Optimization**: Only considers moves within 2 spaces of existing stones.

### UI

Compact grid (~30 chars wide). `●` = human (white), `○` = AI (red). Cursor with brackets `[·]`.

---

## Nine Men's Morris

**Theme**: "A weathered board sits between you and a cloaked stranger."

### Rules

Three-phase game on a 24-position board (3 concentric squares with connecting spokes):

1. **Placing**: Players alternate placing 9 pieces each on empty points
2. **Moving**: Once all placed, slide pieces along lines to adjacent points
3. **Flying**: When down to 3 pieces, can move to any empty point

**Mills**: Three pieces in a row along a line. Forming a mill allows capturing one opponent piece (not in a mill, unless all are).

**Win**: Reduce opponent to 2 pieces, or block all opponent moves. No draws.

### AI — Minimax with Alpha-Beta Pruning

| Difficulty | Search Depth | Random Move % | Reward |
|------------|-------------|---------------|--------|
| Novice | 2-ply | 50% | +50% XP |
| Apprentice | 3-ply | 0% | +100% XP |
| Journeyman | 4-ply | 0% | +150% XP |
| Master | 5-ply | 0% | +1 FR, +200% XP |

**Evaluation**: Piece count difference (heavy), mill count, potential mills (2 of 3), mobility.

### Board Layout

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

16 mill lines, 24 positions with 2-4 adjacencies each.

### UI

`●` = player (bright white), `○` = AI (dim gray), `·` = empty. `[●]` = cursor. Help panel on right explains phases.

---

## Minesweeper (Trap Detection)

**Theme**: "The floor's rigged with pressure plates... help me chart the safe path."

### Rules

Standard minesweeper mechanics themed as dungeon trap detection:

- **First click safety**: Mines placed after first reveal, avoiding clicked cell and 8 neighbors
- **Reveal**: Shows adjacent mine count (1-8) or flood-fills if 0
- **Flag**: Mark suspected traps with `[F]`
- **Win**: All non-mine cells revealed
- **Loss**: Reveal a mine

### Difficulty

| Difficulty | Grid | Mines | Density | Reward |
|------------|------|-------|---------|--------|
| Novice | 9x9 | 10 | 12% | +50% XP |
| Apprentice | 12x12 | 25 | 17% | +75% XP |
| Journeyman | 16x16 | 40 | 16% | +100% XP |
| Master | 20x16 | 60 | 19% | +1 PR, +200% XP |

### Cell Rendering

| Symbol | Meaning |
|--------|---------|
| `░` | Unrevealed |
| `⚑` | Flagged |
| `·` | Revealed, 0 adjacent |
| `1`-`8` | Revealed, colored by number |
| `*` | Mine (on loss) |
| `[░]` | Cursor |

**Number colors**: 1=Blue, 2=Green, 3=Red, 4=DarkBlue, 5=DarkRed, 6=Cyan, 7=Gray, 8=White.

### Controls

Arrow keys to move, Enter to reveal, F to toggle flag, double-Esc to forfeit.

---

## Rune Deciphering (Mastermind)

**Theme**: "Ancient rune tablets... decode the hidden sequence through logical deduction."

### Rules

Mastermind-style deduction puzzle:
- A hidden sequence of runes must be guessed
- After each guess, feedback reveals: `●` exact (right rune, right position), `○` misplaced (right rune, wrong position), `·` wrong (rune not in code)
- Feedback sorted (exact first, then misplaced, then wrong) to avoid leaking positional info

### Difficulty

| Difficulty | Runes | Slots | Guesses | Duplicates | Combinations |
|------------|-------|-------|---------|------------|-------------|
| Novice | 5 | 3 | 10 | No | 60 |
| Apprentice | 6 | 4 | 10 | No | 360 |
| Journeyman | 6 | 4 | 8 | Yes | 1,296 |
| Master | 8 | 5 | 8 | Yes | 32,768 |

### Rewards

| Difficulty | Reward |
|------------|--------|
| Novice | +25% XP |
| Apprentice | +50% XP |
| Journeyman | +1 Fishing Rank, +75% XP |
| Master | +1 Prestige Rank, +2 Fishing Ranks |

### Rune Symbols

8 terminal-friendly rune characters: `᛭ ᚦ ᛟ ᚱ ᛊ ᚹ ᛏ ᚲ`

### Controls

Left/Right to move between slots, Up/Down to cycle runes, Enter to submit, F to clear guess, double-Esc to forfeit.

---

## Shared UI Patterns

All challenges use the same layout convention:
- **Left panel**: Game board/grid
- **Right panel**: Info (difficulty, rules summary, controls)
- **Game-over overlay**: Centered on board area, shows win/loss + reward
- **AI thinking**: Throbber animation during AI computation
- **Status bar**: Context-sensitive instructions at bottom
