# Go Challenge Design

## Overview

Add a 9x9 Go challenge with MCTS AI to the Quest idle RPG. Players compete against an AI opponent using Monte Carlo Tree Search, with difficulty levels based on simulation count.

## Decisions

| Aspect | Decision |
|--------|----------|
| Board size | 9x9 only |
| Rules | Chinese scoring, no suicide, standard ko |
| AI | MCTS (no neural net) |
| Difficulty | Simulation count: 500 / 2,000 / 8,000 / 20,000 |
| Rewards | Prestige ranks: 1 / 2 / 3 / 5 (matches Chess) |
| Discovery | ~2hr average, requires P1+ |
| Challenge name | "Territory Control" |

## Game Rules

### Core Rules
- Stones placed on intersections (not squares)
- Capture groups with zero liberties after placing
- Ko rule: cannot immediately recapture single stone
- Suicide illegal: cannot place if your group would have zero liberties after captures
- Game ends after two consecutive passes

### Chinese Scoring
- Count stones on board + surrounded empty points
- Simpler to implement, MCTS-friendly (can play to completion)

## Data Structures

```rust
pub struct GoGame {
    board: [[Option<Stone>; 9]; 9],  // None = empty
    current_player: Stone,
    ko_point: Option<(usize, usize)>,  // Illegal to play here this turn
    captured_black: u32,  // White's prisoners
    captured_white: u32,  // Black's prisoners
    consecutive_passes: u8,  // 2 = game over
    move_history: Vec<GoMove>,
    difficulty: GoDifficulty,
}

pub enum Stone { Black, White }
pub enum GoMove { Place(usize, usize), Pass }
pub enum GoDifficulty { Novice, Apprentice, Journeyman, Master }
pub enum GoResult { PlayerWin, AiWin, Draw }
```

## MCTS Algorithm

### Tree Structure

```rust
struct MctsNode {
    game_state: GoGame,
    move_taken: Option<GoMove>,
    parent: Option<usize>,       // Index in node pool
    children: Vec<usize>,
    visits: u32,
    wins: f32,
    untried_moves: Vec<GoMove>,
}
```

### Four Phases (per simulation)

1. **Selection** - From root, pick child with best UCT score until reaching unexpanded node
   ```
   UCT = (wins / visits) + C * sqrt(ln(parent_visits) / visits)
   ```
   C ≈ 1.4 balances exploration vs exploitation

2. **Expansion** - Add one new child node for an untried move

3. **Simulation (Playout)** - Play random legal moves until game ends, return winner

4. **Backpropagation** - Walk back to root, updating visits and wins

### Playout Heuristics

Light heuristics to strengthen random playouts:
- Respond to atari (save groups under attack)
- Avoid self-atari
- Avoid filling own eyes

### Difficulty Levels

| Level | Simulations | Est. Strength | Prestige Reward |
|-------|-------------|---------------|-----------------|
| Novice | 500 | ~20 kyu | 1 |
| Apprentice | 2,000 | ~17 kyu | 2 |
| Journeyman | 8,000 | ~14 kyu | 3 |
| Master | 20,000 | ~12 kyu | 5 |

## UI Design

### Board Display

```
 ┌──┬──┬──┬──┬──┬──┬──┬──┐
 ├──┼──┼──○──┼──┼──┼──┼──┤
 ├──┼──●──○──┼──┼──●──┼──┤
 ├──┼──●──○──○──○──┼──┼──┤
 ├──┼──┼──●──●──○──┼──┼──┤
 ├──┼──┼──┼──┼──●──┼──┼──┤
 ├──┼──●──┼──┼──┼──┼──┼──┤
 ├──┼──┼──┼──┼──┼──┼──┼──┤
 └──┴──┴──┴──┴──┴──┴──┴──┘

 Black (You): ●  Captures: 2
 White (AI):  ○  Captures: 0

 [Arrows: move]  [Enter: place]  [P: pass]  [Q: resign]
```

- Stones on intersections
- No coordinates displayed
- Cursor highlights intersection point
- Shows captures for both sides

### Controls

- Arrow keys: move cursor
- Enter: place stone
- P: pass
- Q: resign (with confirmation)

## File Structure

```
src/challenges/go/
├── mod.rs       # Module exports
├── types.rs     # GoGame, Stone, GoMove, GoDifficulty, GoResult, GoStats
├── logic.rs     # Rules: placement, capture, ko, scoring, liberty counting
├── mcts.rs      # MCTS: selection, expansion, simulation, backpropagation

src/ui/
└── go_scene.rs  # Board rendering, cursor, controls
```

## Implementation Tasks

### Phase 1: Core Game Logic
1. Create `src/challenges/go/` module structure
2. Implement `types.rs` - game state, enums, stats
3. Implement `logic.rs` - stone placement, capture detection, ko rule
4. Implement liberty counting and group detection
5. Implement Chinese scoring
6. Add unit tests for rules

### Phase 2: MCTS AI
1. Implement MCTS node structure
2. Implement UCT selection
3. Implement node expansion
4. Implement random playout
5. Implement backpropagation
6. Add playout heuristics (atari response, eye protection)
7. Add difficulty-based simulation limits
8. Add unit tests for AI

### Phase 3: UI Integration
1. Create `go_scene.rs` with board rendering
2. Implement cursor navigation
3. Implement stone placement visualization
4. Implement game-over overlay
5. Add to challenge menu system
6. Wire up discovery mechanic

### Phase 4: Testing & Polish
1. Integration tests
2. Play testing for AI difficulty tuning
3. Adjust simulation counts if needed
4. Performance optimization if needed

## Technical Notes

### Why MCTS over Minimax

- Go's branching factor (~80 for 9x9) makes minimax impractical
- No reliable evaluation function exists for Go positions
- MCTS uses random playouts to estimate position value statistically
- Works well without domain knowledge, improves with heuristics

### Performance Considerations

- 20,000 simulations at Master difficulty should complete in <2 seconds on modern hardware
- Node pool allocation to avoid frequent heap allocations
- Playout can reuse a scratch board to avoid cloning

### Comparison to Chess Implementation

| Aspect | Chess | Go |
|--------|-------|-----|
| AI Algorithm | Minimax (3-ply) | MCTS (500-20k sims) |
| External Crate | chess-engine | None (pure Rust) |
| Board Size | 8x8 squares | 9x9 intersections |
| Est. Difficulty | ~500-1350 ELO | ~20-12 kyu (similar range) |
| Rewards | Prestige 1/2/3/5 | Prestige 1/2/3/5 |
