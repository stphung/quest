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

**Engine**: Built-in minimax with alpha-beta pruning. No external chess engine dependency.

**Implementation scope** (~400–600 lines):
- Legal move generation for all piece types including castling, en passant, promotion
- Board representation (array-based, not bitboard — simplicity over performance)
- Minimax search with alpha-beta pruning
- Simple evaluation function: material count + piece-square tables + basic positional heuristics
- Search depth scales with difficulty

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
    pub board: Board,
    pub difficulty: ChessDifficulty,
    pub reward_prestige: u32,
    pub player_color: Color,        // Always White (plays first)
    pub cursor: (u8, u8),           // Board cursor position
    pub selected_piece: Option<(u8, u8)>,  // Currently selected piece
    pub legal_moves: Vec<(u8, u8)>,        // Legal destinations for selected piece
    pub game_status: ChessGameStatus,      // Playing, Checkmate, Stalemate, Forfeit
    pub move_history: Vec<ChessMove>,
    pub ai_thinking: bool,          // True while AI is computing
    pub ai_think_ticks: u32,        // Cosmetic delay counter
    pub captured_by_player: Vec<Piece>,
    pub captured_by_ai: Vec<Piece>,
}

pub enum ChessDifficulty { Apprentice, Journeyman, Master }
pub enum ChessGameStatus { Playing, Checkmate(Color), Stalemate, Forfeit }
```

### Board Representation

```rust
pub struct Board {
    pub squares: [[Option<Piece>; 8]; 8],
    pub turn: Color,
    pub castling_rights: CastlingRights,
    pub en_passant_target: Option<(u8, u8)>,
    pub halfmove_clock: u32,        // For 50-move draw rule
}

pub struct Piece {
    pub kind: PieceKind,
    pub color: Color,
}

pub enum PieceKind { Pawn, Knight, Bishop, Rook, Queen, King }
pub enum Color { White, Black }
```

## File Structure

```
src/
├── chess/
│   ├── mod.rs              # Module exports, ChessState, ChessChallenge types
│   ├── board.rs            # Board struct, piece types, move application
│   ├── moves.rs            # Legal move generation (all piece types)
│   ├── engine.rs           # Minimax + alpha-beta, evaluation function
│   └── logic.rs            # Game session management, discovery, tick processing
├── ui/
│   └── chess_scene.rs      # Board rendering, cursor, highlights, captured pieces
```

This uses a subdirectory rather than flat files because the chess implementation is substantially more complex than fishing (which is 3 files). Move generation alone warrants its own module.

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
            let ai_move = engine::find_best_move(&chess.board, chess.difficulty);
            chess.board.apply_move(ai_move);
            chess.ai_thinking = false;
            // Check for checkmate/stalemate after AI move
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
    if !chess.ai_thinking {
        match key.code {
            KeyCode::Up => chess.move_cursor(0, -1),
            KeyCode::Down => chess.move_cursor(0, 1),
            KeyCode::Left => chess.move_cursor(-1, 0),
            KeyCode::Right => chess.move_cursor(1, 0),
            KeyCode::Enter => chess.select_or_move(),  // Select piece or confirm move
            KeyCode::Esc => chess.deselect_or_forfeit(),
            _ => {}
        }
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
- **Draw by repetition / 50-move rule**: Track with halfmove clock. Threefold repetition can be skipped for v1 (rare in short games against simple AI).
- **Pawn promotion**: Auto-promote to queen for v1. Player choice can be added later.

## Implementation Phases

**Phase 1: Chess engine core**
- Board representation, piece types
- Legal move generation (all pieces including castling, en passant)
- Minimax + alpha-beta search
- Evaluation function
- Comprehensive tests for move generation and search

**Phase 2: Game session and state integration**
- ChessState, ChessChallenge, ChessGame structs
- Integration into GameState with serde attributes
- Discovery logic and difficulty determination
- Session lifecycle (create, play, end with reward/no-reward)
- Prestige reward application

**Phase 3: Input handling**
- Challenge acceptance/decline
- Cursor movement and piece selection
- Move confirmation
- Forfeit flow (double-Esc confirmation)

**Phase 4: UI rendering**
- Chess board with Unicode pieces
- Square coloring (light/dark)
- Cursor and selection highlighting
- Legal move indicators
- Captured pieces display
- Game status messages
- Pending challenge banner

**Phase 5: Polish**
- AI "thinking" animation
- Move history display (algebraic notation)
- Sound/visual feedback on check
- Stats display (games played/won in stats panel)
