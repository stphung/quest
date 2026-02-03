# Chess Minigame System Design

## Overview

A chess minigame where the player plays a full game of chess against an AI opponent for massive rewards (1–3 prestige ranks). This is the first player-controlled interactive element in the game — everything else is idle/automated.

## Motivation

The game currently has no direct player agency during gameplay. Combat, dungeon exploration, and fishing all run automatically. Chess introduces a high-stakes skill-based activity that breaks the idle loop with genuine player interaction, creating a memorable moment in an otherwise ambient experience.

## Core Design

### Discovery

A chess challenge is discovered randomly while the player is in normal combat (not in a dungeon or fishing session). Discovery works like fishing/dungeons but is much rarer.

- **Discovery chance**: ~0.5% per tick cycle (roughly once every 30–60 minutes of play)
- **Prestige gate**: Requires prestige rank >= 1 (player must understand the game before encountering chess)
- **State requirement**: Not in dungeon, not fishing, not already in a chess challenge
- **Discovery message**: A mysterious figure appears in the combat log offering a chess challenge

### Challenge Acceptance

Unlike fishing (auto-enters) and dungeons (auto-enters), chess presents a **pending challenge** that the player must accept or decline. This is critical because:

1. Chess requires active attention — entering automatically would be hostile
2. The reward is so large that accidental entry/exit must be prevented
3. It establishes the pattern for future player-controlled minigames

**Flow:**
```
Discovery → Pending challenge state → Player presses 'C' to accept / 'Esc' to decline
```

- Pending challenges persist until accepted or declined (no timeout)
- While a challenge is pending, combat continues normally
- A visual indicator appears in the stats panel showing the pending challenge
- Declining a challenge removes it; a new one must be discovered naturally

### Chess Gameplay

**Board**: Standard 8×8 chess board with Unicode piece symbols rendered in the right panel (replacing the combat scene).

**Input model** (arrow keys + enter):
```
Arrow keys  → Move cursor on the board
Enter       → Select piece / confirm move destination
Esc         → Deselect piece (if selected) / offer forfeit (if no piece selected)
```

**Move flow:**
1. Cursor appears on the board (highlighted square)
2. Player navigates to a piece and presses Enter to select it
3. Legal destination squares are highlighted
4. Player navigates to a destination and presses Enter to confirm
5. AI responds after a brief delay (0.5–1s, animated "thinking")
6. Repeat until checkmate, stalemate, or forfeit

**Visual feedback:**
- Selected piece: highlighted background
- Legal moves: distinct color/marker on destination squares
- Last move: both source and destination highlighted
- Check: king square highlighted in warning color
- Captured pieces shown alongside the board

### AI Opponent

**Engine**: The `chess-engine` crate (https://crates.io/crates/chess-engine) — a dependency-free Rust library with built-in minimax + alpha-beta pruning. This eliminates ~400–600 lines of hand-rolled move generation and search code, and gives us correct handling of castling, en passant, promotion, and draw rules for free.

**Key API surface:**
```rust
use chess_engine::*;

let board = Board::default();                    // Standard starting position
let legal_moves = board.get_legal_moves();       // All valid board states
let best_move = board.get_best_next_move(4);     // AI move at depth 4
let result = board.play_move(best_move);         // Returns GameResult enum

// GameResult variants:
// - Continuing(Board) — game continues
// - Victory(Color)    — checkmate
// - IllegalMove(Board) — invalid move attempted
// - Stalemate         — draw
```

The `get_best_next_move(depth)` parameter maps directly to difficulty tiers. The crate's `Board` type is the canonical board representation — we wrap it rather than reimplement it.

**Difficulty tiers** (based on prestige rank at time of discovery):

| Prestige Range | Difficulty | Search Depth | Reward    |
|---------------|------------|--------------|-----------|
| 1–4           | Apprentice | 2 ply       | 1 prestige |
| 5–9           | Journeyman | 3 ply       | 2 prestige |
| 10+           | Master     | 4 ply       | 3 prestige |

Higher difficulty = deeper search = stronger play = bigger reward. The AI should be beatable but require thought — this isn't meant to be a grandmaster-level engine.

**Thinking budget**: AI move computation must complete within ~200ms to avoid blocking the game tick. At 4-ply with alpha-beta pruning on an 8×8 board, this is comfortably achievable.

### Reward Structure

**On win (checkmate the AI)**:
- Prestige rank increases by 1–3 (based on difficulty tier)
- Prestige reset is performed (same as normal prestige: level, XP, attributes, equipment, zones all reset)
- Fishing rank is preserved (same as normal prestige)
- Victory message displayed in combat log
- The prestige confirmation dialog is NOT shown — the reward is applied directly since the player already committed by playing the full game

**On loss (AI checkmates the player)**:
- No penalty — just return to normal combat
- A message in the combat log: "The mysterious figure nods respectfully and vanishes"
- No prestige loss, no death, nothing negative

**On stalemate/draw**:
- Small consolation reward: bonus XP equivalent to ~50 kills
- "The figure smiles knowingly and fades away"

**On forfeit (player presses Esc twice to confirm)**:
- Same as loss — no penalty
- "You concede the game. The figure disappears without a word"

### Why Prestige as Reward?

Prestige is the highest-value currency in the game. Granting 1–3 ranks for a chess win:

- Creates a genuine incentive for the player to engage with the interactive element
- Provides a meaningful shortcut in the prestige grind for skilled players
- Makes discovery feel exciting rather than ignorable
- The reset that comes with prestige prevents it from being pure power inflation — you still restart

The reward should feel like "I just saved hours of grinding" not "I broke the game."

## State Model

### New fields on GameState

```rust
/// Persistent chess stats (survives prestige, saved to disk)
pub chess: ChessState,                          // #[serde(default)]

/// Active chess game (transient, not saved)
pub active_chess: Option<ChessGame>,            // #[serde(skip)]

/// Pending chess challenge (not yet accepted)
pub pending_chess_challenge: Option<ChessChallenge>,  // #[serde(skip)]
```

### Data Structures

```rust
/// Persistent stats across all chess games
pub struct ChessState {
    pub games_played: u32,
    pub games_won: u32,
    pub games_lost: u32,
    pub games_drawn: u32,
    pub prestige_earned: u32,       // Total prestige ranks earned from chess
}

/// A pending challenge waiting for player acceptance
pub struct ChessChallenge {
    pub difficulty: ChessDifficulty,
    pub reward_prestige: u32,       // 1, 2, or 3
}

/// Active chess game session
pub struct ChessGame {
    pub board: chess_engine::Board,    // Board state from chess-engine crate
    pub difficulty: ChessDifficulty,
    pub reward_prestige: u32,
    pub cursor: (u8, u8),              // Board cursor position (file, rank)
    pub selected_square: Option<(u8, u8)>,  // Currently selected square
    pub legal_targets: Vec<chess_engine::Board>,  // Legal board states from selected piece
    pub game_over: bool,               // True when game has ended
    pub forfeit_pending: bool,         // True after first Esc press (confirm with second)
    pub ai_thinking: bool,             // True while AI cosmetic delay is active
    pub ai_think_ticks: u32,           // Cosmetic delay counter
}

pub enum ChessDifficulty { Apprentice, Journeyman, Master }
```

**Note on the `chess-engine` crate's design**: The crate uses a copy-on-make model where `get_legal_moves()` returns `Vec<Board>` (all legal resulting board states) rather than a list of move coordinates. To map cursor-based input to legal moves, we filter `get_legal_moves()` to boards where the selected square's piece has moved. This is a different mental model from coordinate-based move selection but works well — see the Input Handling section for details.

## File Structure

```
src/
├── chess.rs                # ChessState, ChessChallenge, ChessGame, ChessDifficulty types
├── chess_logic.rs          # Discovery, session lifecycle, move application, AI turn, reward logic
├── ui/
│   └── chess_scene.rs      # Board rendering, cursor, highlights, captured pieces
```

With the `chess-engine` crate handling board representation, move generation, and AI search, a flat 2-file layout (matching the fishing pattern) is sufficient. No subdirectory needed — the complexity that would have required `board.rs`, `moves.rs`, and `engine.rs` is now in the crate.

## Integration Points

### Game Tick (main.rs)

```rust
// Priority: chess > fishing > dungeon > combat

// 1. Process active chess game (no tick processing needed — chess is turn-based)
//    AI thinking delay is the only tick-driven element
if let Some(ref mut chess) = game_state.active_chess {
    if chess.ai_thinking {
        chess.ai_think_ticks += 1;
        if chess.ai_think_ticks >= AI_THINK_DELAY_TICKS {
            let depth = chess.difficulty.search_depth();
            let ai_board = chess.board.get_best_next_move(depth);
            chess.board = ai_board;
            chess.ai_thinking = false;
            // Check game result via get_legal_moves() — empty = checkmate or stalemate
        }
    }
    // Update timers, skip combat
    return;
}

// 2. Discovery check (during normal combat ticks)
if game_state.pending_chess_challenge.is_none()
    && game_state.active_chess.is_none()
    && game_state.prestige_rank >= 1
{
    if rng.gen::<f64>() < CHESS_DISCOVERY_CHANCE {
        let difficulty = determine_difficulty(game_state.prestige_rank);
        game_state.pending_chess_challenge = Some(ChessChallenge { ... });
        game_state.combat_state.add_log_entry("A mysterious figure appears...", false, true);
    }
}
```

### Input Handling (main.rs)

```rust
// In Game screen input handling, before existing key checks:

// Pending challenge: C to accept, Esc to decline
if let Some(ref challenge) = game_state.pending_chess_challenge {
    match key.code {
        KeyCode::Char('c') | KeyCode::Char('C') => {
            let challenge = game_state.pending_chess_challenge.take().unwrap();
            game_state.active_chess = Some(ChessGame::new(challenge));
        }
        KeyCode::Esc => {
            game_state.pending_chess_challenge = None;
        }
        _ => {}
    }
    // Don't fall through to other input handlers while challenge is pending
}

// Active chess game input
if let Some(ref mut chess) = game_state.active_chess {
    if !chess.ai_thinking && !chess.game_over {
        match key.code {
            KeyCode::Up => chess.move_cursor(0, 1),
            KeyCode::Down => chess.move_cursor(0, -1),
            KeyCode::Left => chess.move_cursor(-1, 0),
            KeyCode::Right => chess.move_cursor(1, 0),
            KeyCode::Enter => {
                // Two-phase selection using the crate's copy-on-make model:
                // Phase 1 (no piece selected): Select the piece at cursor.
                //   Filter board.get_legal_moves() to only boards where the
                //   piece at cursor has moved. Store filtered list in legal_targets.
                // Phase 2 (piece selected): Player picks a destination square.
                //   Find the board in legal_targets where the piece now occupies
                //   the cursor square. Apply that board via play_move().
                //   Then trigger AI thinking delay.
                chess.select_or_move();
            }
            KeyCode::Esc => chess.deselect_or_forfeit(),  // First Esc = deselect/warn, second = forfeit
            _ => {}
        }
    } else if chess.game_over {
        // Any key dismisses the result and applies rewards/returns to combat
        chess_logic::end_game(game_state);
    }
    // Don't fall through to combat/prestige input
}
```

### UI Dispatch (ui/mod.rs)

Chess takes highest rendering priority (above fishing):

```rust
if let Some(ref chess) = game_state.active_chess {
    chess_scene::render_chess_scene(frame, chunks[1], chess);
} else if game_state.pending_chess_challenge.is_some() {
    // Show challenge banner overlaid on normal combat scene
    combat_scene::draw_combat_scene(frame, chunks[1], game_state);
    // Render challenge prompt overlay
} else if let Some(ref session) = game_state.active_fishing {
    // ... existing
}
```

## Terminal Rendering

The chess board fits comfortably in the right 50% panel. Each square needs ~3 characters wide × 1.5 lines tall for readability.

**Board layout** (approximate):
```
  a  b  c  d  e  f  g  h
8 ♜  ♞  ♝  ♛  ♚  ♝  ♞  ♜  8
7 ♟  ♟  ♟  ♟  ♟  ♟  ♟  ♟  7
6 ·  ·  ·  ·  ·  ·  ·  ·  6
5 ·  ·  ·  ·  ·  ·  ·  ·  5
4 ·  ·  ·  ·  ·  ·  ·  ·  4
3 ·  ·  ·  ·  ·  ·  ·  ·  3
2 ♙  ♙  ♙  ♙  ♙  ♙  ♙  ♙  2
1 ♖  ♘  ♗  ♕  ♔  ♗  ♘  ♖  1
  a  b  c  d  e  f  g  h

Captured: ♟♟♞        [You]
Captured: ♙           [Opponent]

Status: Your move (select a piece)
```

- Alternating background colors for light/dark squares (Ratatui styled spans)
- Cursor shown as highlighted background on current square
- Selected piece's legal moves shown with a marker (e.g., `×` or colored background)
- Unicode chess symbols: ♔♕♖♗♘♙ (white) / ♚♛♜♝♞♟ (black)

## Edge Cases

- **Player quits game during chess**: Session is `#[serde(skip)]`, so it's lost. This is intentional — chess requires commitment. The pending challenge is also lost.
- **Prestige during pending challenge**: If player manually prestiges while a challenge is pending, the challenge is cleared (transient state).
- **AI computation time**: With 4-ply minimax + alpha-beta on a standard board, worst case is ~50ms. No risk of blocking the game loop.
- **Draw by repetition / 50-move rule**: The `chess-engine` crate handles stalemate detection via `GameResult::Stalemate`. Threefold repetition is not tracked by the crate — acceptable for v1 (rare in short games against simple AI).
- **Pawn promotion**: Handled by the crate — `get_legal_moves()` returns separate board states for each promotion piece. For v1, auto-select queen promotion by filtering legal targets. Player choice can be added later.
- **Mapping cursor moves to crate API**: The crate returns `Vec<Board>` from `get_legal_moves()`, not move coordinates. To determine which square a piece moved to, diff the current board against each legal board to find the piece that changed position. This is O(64 × num_legal_moves) per selection — negligible cost.

## Dependencies

Add to `Cargo.toml`:
```toml
chess-engine = "0.1"
```

The crate has zero transitive dependencies, so it adds no dependency tree bloat.

## Implementation Phases

**Phase 1: Types and state integration**
- Add `chess-engine` to Cargo.toml
- Define ChessState, ChessChallenge, ChessGame, ChessDifficulty in `chess.rs`
- Add fields to GameState with serde attributes
- Board-to-grid helper: extract piece positions from `chess_engine::Board` for rendering and input mapping

**Phase 2: Game session logic**
- Discovery logic and difficulty determination in `chess_logic.rs`
- Session lifecycle (create game from challenge, AI turn via `get_best_next_move`, game end detection)
- Move selection logic: filtering `get_legal_moves()` by source/destination square
- Prestige reward application on win
- Forfeit flow

**Phase 3: Input handling**
- Challenge acceptance/decline key routing in main.rs
- Cursor movement and two-phase piece selection
- Forfeit confirmation (double-Esc)
- Game-over dismissal

**Phase 4: UI rendering**
- Chess board with Unicode pieces in `chess_scene.rs`
- Square coloring (light/dark) via Ratatui styled spans
- Cursor and selection highlighting
- Legal move indicators (highlight destination squares from filtered legal_targets)
- Captured pieces display (diff starting material vs current board)
- Game status messages (your move, AI thinking, check, checkmate, stalemate)
- Pending challenge banner overlay on combat scene

**Phase 5: Polish**
- AI "thinking" animation (dots or spinner)
- Stats display (games played/won in stats panel)
- Move history sidebar (algebraic notation, derived from board diffs)
