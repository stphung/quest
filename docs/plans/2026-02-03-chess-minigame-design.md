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

### Challenge Menu

Unlike fishing (auto-enters) and dungeons (auto-enters), chess presents a **pending challenge** that the player reviews through a navigable menu. This is critical because:

1. Chess requires active attention — entering automatically would be hostile
2. The reward is so large that accidental entry/exit must be prevented
3. It establishes a generic menu pattern for future player-controlled minigames

**Menu architecture**: The challenge menu is a generic system — not chess-specific. It holds a list of `PendingChallenge` items that any minigame system can push into. Chess is the first producer, but future minigames (gambling, crafting, etc.) will add their own challenge types to the same menu.

**Opening the menu**: When one or more challenges are pending, a notification appears in the stats panel: `"1 challenge available — [Tab] to view"`. Pressing `Tab` opens the challenge menu as a full overlay on the right panel (replacing the combat scene). Combat continues in the background while browsing.

**Menu states**: The menu has two views — list and detail:

```
LIST VIEW                              DETAIL VIEW
┌──────────────────────────┐           ┌──────────────────────────┐
│   Pending Challenges     │           │   Chess Challenge        │
│                          │           │                          │
│ > ♟ Chess Challenge      │  Enter →  │   A mysterious figure    │
│                          │           │   challenges you to a    │
│                          │           │   game of chess.         │
│                          │           │                          │
│                          │           │   Difficulty: Journeyman │
│                          │           │   AI Depth:   3 ply      │
│                          │           │   Reward:     +2 Prestige│
│                          │           │                          │
│                          │           │   Win: +2 prestige ranks │
│                          │           │   Lose: No penalty       │
│                          │           │   Draw: Bonus XP         │
│                          │           │                          │
│                          │  ← Esc    │   [Enter] Accept         │
│                          │           │   [D]     Decline        │
│   [Tab/Esc] Close        │           │   [Esc]   Back           │
└──────────────────────────┘           └──────────────────────────┘
```

**Input in list view:**
- `Up/Down` — Navigate between challenges
- `Enter` — Open detail view for selected challenge
- `Tab` or `Esc` — Close menu, return to combat

**Input in detail view:**
- `Enter` — Accept challenge (starts the minigame)
- `D` — Decline challenge (removes it from the list)
- `Esc` — Back to list view

**Behavior:**
- Pending challenges persist until accepted or declined (no timeout)
- While browsing the menu, combat continues in the background (ticks still fire)
- Declining a challenge removes it; a new one must be discovered naturally
- Accepting a challenge closes the menu and enters the minigame immediately
- Multiple challenges can queue up (e.g., a chess challenge arrives while another is already pending). Each is a separate list entry

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
/// Generic challenge menu (transient, not saved)
pub challenge_menu: ChallengeMenu,              // #[serde(skip)]

/// Persistent chess stats (survives prestige, saved to disk)
pub chess: ChessState,                          // #[serde(default)]

/// Active chess game (transient, not saved)
pub active_chess: Option<ChessGame>,            // #[serde(skip)]
```

### Challenge Menu (Generic)

The challenge menu is minigame-agnostic. Each minigame defines its own `ChallengeType` variant and detail text. The menu handles navigation, display, and accept/decline flow uniformly.

```rust
/// A single pending challenge in the menu
pub struct PendingChallenge {
    pub challenge_type: ChallengeType,
    pub title: String,              // e.g. "Chess Challenge"
    pub icon: &'static str,        // e.g. "♟"
    pub description: String,       // Multi-line flavor text for detail view
    pub details: Vec<(String, String)>,  // Key-value pairs: ("Difficulty", "Journeyman")
    pub reward_summary: String,    // e.g. "+2 Prestige Ranks"
}

/// Extensible enum — future minigames add variants here
pub enum ChallengeType {
    Chess(ChessChallenge),
    // Future: Gambling(GamblingChallenge), Crafting(CraftingChallenge), etc.
}

/// Chess-specific challenge data (carried inside ChallengeType::Chess)
pub struct ChessChallenge {
    pub difficulty: ChessDifficulty,
    pub reward_prestige: u32,       // 1, 2, or 3
}

/// Menu state for navigation
pub struct ChallengeMenu {
    pub challenges: Vec<PendingChallenge>,
    pub is_open: bool,              // Whether the menu overlay is visible
    pub selected_index: usize,      // Cursor position in list view
    pub viewing_detail: bool,       // true = detail view, false = list view
}
```

### Chess-Specific Types

```rust
/// Persistent stats across all chess games
pub struct ChessState {
    pub games_played: u32,
    pub games_won: u32,
    pub games_lost: u32,
    pub games_drawn: u32,
    pub prestige_earned: u32,       // Total prestige ranks earned from chess
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
├── challenge_menu.rs       # PendingChallenge, ChallengeType, ChallengeMenu (generic)
├── chess.rs                # ChessState, ChessChallenge, ChessGame, ChessDifficulty types
├── chess_logic.rs          # Discovery, session lifecycle, move application, AI turn, reward logic
├── ui/
│   ├── challenge_menu_scene.rs  # Challenge menu list + detail rendering (generic)
│   └── chess_scene.rs           # Chess board rendering, cursor, highlights
```

`challenge_menu.rs` and `challenge_menu_scene.rs` are minigame-agnostic — they render challenges from any source. When a new minigame is added, it only needs to push a `PendingChallenge` with its own `ChallengeType` variant; the menu UI works unchanged.

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
//    Chess pushes into the generic challenge menu
if game_state.active_chess.is_none()
    && game_state.prestige_rank >= 1
    && !game_state.challenge_menu.has_challenge_of_type(ChallengeType::Chess)
{
    if rng.gen::<f64>() < CHESS_DISCOVERY_CHANCE {
        let challenge = chess_logic::create_challenge(game_state.prestige_rank);
        game_state.challenge_menu.challenges.push(challenge);
        game_state.combat_state.add_log_entry(
            "♟ A mysterious figure steps from the shadows...", false, true
        );
        game_state.combat_state.add_log_entry(
            "♟ Press [Tab] to view pending challenges", false, true
        );
    }
}
```

### Input Handling (main.rs)

Input priority: active chess > challenge menu open > Tab to open menu > normal game keys.

```rust
// PRIORITY 1: Active chess game (highest — full input capture)
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
            KeyCode::Esc => chess.deselect_or_forfeit(),
            _ => {}
        }
    } else if chess.game_over {
        // Any key dismisses the result and applies rewards/returns to combat
        chess_logic::end_game(game_state);
    }
    // Don't fall through — chess captures all input
    return;
}

// PRIORITY 2: Challenge menu is open (overlay on combat scene)
if game_state.challenge_menu.is_open {
    let menu = &mut game_state.challenge_menu;

    if menu.viewing_detail {
        // Detail view
        match key.code {
            KeyCode::Enter => {
                // Accept: remove challenge from list, start the minigame
                let challenge = menu.challenges.remove(menu.selected_index);
                menu.is_open = false;
                menu.viewing_detail = false;
                challenge_menu::accept_challenge(game_state, challenge);
                // accept_challenge() dispatches by ChallengeType:
                //   Chess → game_state.active_chess = Some(ChessGame::new(...))
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                // Decline: remove from list, stay in menu
                menu.challenges.remove(menu.selected_index);
                menu.selected_index = menu.selected_index.min(
                    menu.challenges.len().saturating_sub(1)
                );
                menu.viewing_detail = false;
                if menu.challenges.is_empty() {
                    menu.is_open = false;
                }
            }
            KeyCode::Esc => {
                // Back to list view
                menu.viewing_detail = false;
            }
            _ => {}
        }
    } else {
        // List view
        match key.code {
            KeyCode::Up => {
                if menu.selected_index > 0 {
                    menu.selected_index -= 1;
                }
            }
            KeyCode::Down => {
                if menu.selected_index + 1 < menu.challenges.len() {
                    menu.selected_index += 1;
                }
            }
            KeyCode::Enter => {
                if !menu.challenges.is_empty() {
                    menu.viewing_detail = true;
                }
            }
            KeyCode::Tab | KeyCode::Esc => {
                menu.is_open = false;
            }
            _ => {}
        }
    }
    // Don't fall through — menu captures input while open
    return;
}

// PRIORITY 3: Tab to open challenge menu (only when challenges exist)
if key.code == KeyCode::Tab && !game_state.challenge_menu.challenges.is_empty() {
    game_state.challenge_menu.is_open = true;
    game_state.challenge_menu.selected_index = 0;
    game_state.challenge_menu.viewing_detail = false;
    return;
}

// PRIORITY 4: Normal game keys (P for prestige, Q for quit, etc.)
// ... existing input handling ...
```

### UI Dispatch (ui/mod.rs)

Rendering priority: active chess > challenge menu overlay > fishing > dungeon > combat.

```rust
if let Some(ref chess) = game_state.active_chess {
    // Full chess board (replaces combat scene entirely)
    chess_scene::render_chess_scene(frame, chunks[1], chess);
} else if game_state.challenge_menu.is_open {
    // Challenge menu overlay (replaces combat scene while open)
    challenge_menu_scene::render_challenge_menu(frame, chunks[1], &game_state.challenge_menu);
} else if let Some(ref session) = game_state.active_fishing {
    // ... existing fishing
} else if let Some(dungeon) = &game_state.active_dungeon {
    // ... existing dungeon
} else {
    // Default combat
    combat_scene::draw_combat_scene(frame, chunks[1], game_state);
}
```

**Stats panel notification**: When `challenge_menu.challenges` is non-empty and the menu is not open, the stats panel shows a persistent notification:
```
┌─────────────────────────┐
│ 1 challenge pending     │
│ Press [Tab] to view     │
└─────────────────────────┘
```
This is rendered in `stats_panel.rs` by checking `game_state.challenge_menu.challenges.len()`. It provides passive awareness without interrupting the idle flow.

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
- **Prestige during pending challenge**: If player manually prestiges while challenges are in the menu, they are cleared (challenge_menu is transient/`#[serde(skip)]`).
- **Menu open during fishing/dungeon**: Tab only opens the menu when challenges exist. If a fishing session or dungeon starts while the menu is open, the menu closes automatically (active minigame takes rendering priority).
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

**Phase 1: Challenge menu system (generic, no chess yet)**
- Define PendingChallenge, ChallengeType, ChallengeMenu in `challenge_menu.rs`
- Add `challenge_menu: ChallengeMenu` to GameState with `#[serde(skip)]`
- Implement `challenge_menu_scene.rs`: list view and detail view rendering
- Input routing in main.rs: Tab to open, arrow keys to navigate, Enter/D/Esc for actions
- Stats panel notification when challenges are pending
- `accept_challenge()` dispatch function (match on ChallengeType)

**Phase 2: Chess types and state integration**
- Add `chess-engine` to Cargo.toml
- Define ChessState, ChessChallenge, ChessGame, ChessDifficulty in `chess.rs`
- Add `chess: ChessState` and `active_chess: Option<ChessGame>` to GameState
- Board-to-grid helper: extract piece positions from `chess_engine::Board` for rendering and input mapping
- Wire ChallengeType::Chess into `accept_challenge()` dispatch

**Phase 3: Chess game session logic**
- Discovery logic and difficulty determination in `chess_logic.rs`
- `create_challenge()` — builds a PendingChallenge with chess-specific details
- Session lifecycle (create game from challenge, AI turn via `get_best_next_move`, game end detection)
- Move selection logic: filtering `get_legal_moves()` by source/destination square
- Prestige reward application on win
- Forfeit flow

**Phase 4: Chess input handling**
- Chess game input routing in main.rs (cursor movement, two-phase piece selection)
- Forfeit confirmation (double-Esc)
- Game-over dismissal

**Phase 5: Chess UI rendering**
- Chess board with Unicode pieces in `chess_scene.rs`
- Square coloring (light/dark) via Ratatui styled spans
- Cursor and selection highlighting
- Legal move indicators (highlight destination squares from filtered legal_targets)
- Captured pieces display (diff starting material vs current board)
- Game status messages (your move, AI thinking, check, checkmate, stalemate)

**Phase 6: Polish**
- AI "thinking" animation (dots or spinner)
- Stats display (games played/won in stats panel)
- Move history sidebar (algebraic notation, derived from board diffs)
