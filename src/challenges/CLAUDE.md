# Challenge Minigames

This module contains player-controlled challenge minigames. Challenges are discovered randomly during gameplay (requires P1+) and appear in the challenge menu for the player to accept or decline.

## Shared Types (`mod.rs`)

All 6 minigames share these types defined in `challenges/mod.rs`:

- **`ChallengeDifficulty`** — Novice, Apprentice, Journeyman, Master (4-tier system with `ALL`, `from_index()`, `name()`, `to_str()`)
- **`ChallengeResult`** — Win, Loss, Draw, Forfeit
- **`MinigameInput`** — Up, Down, Left, Right, Primary, Secondary, Cancel, Other
- **`ActiveMinigame`** — Enum wrapping all 6 game structs
- **`MinigameWinInfo`** — Returned on win for achievement tracking

Two shared functions handle game lifecycle:
- **`start_minigame(state, challenge_type, difficulty)`** — Creates game and sets `active_minigame`
- **`apply_minigame_result(state)`** — Extracts result, grants rewards (XP, prestige, fishing ranks), logs entries, clears game

## Adding a New Challenge

### 1. Module Structure

Create a new subdirectory with three files:

```
src/challenges/newgame/
├── mod.rs      # Public exports
├── types.rs    # Game struct, game-specific helper functions
└── logic.rs    # Game logic, input processing, AI
```

Optional additional files for complex AI:
- `mcts.rs` - Monte Carlo Tree Search (see Go)
- `ai.rs` - Other AI implementations

### 2. Required Types (`types.rs`)

Games use the shared `ChallengeDifficulty` and `ChallengeResult` from `mod.rs`. No per-game difficulty or result enums needed.

```rust
use crate::challenges::{ChallengeDifficulty, ChallengeResult};

/// Main game state
#[derive(Debug, Clone)]
pub struct NewGameGame {
    pub difficulty: ChallengeDifficulty,
    pub game_result: Option<ChallengeResult>,
    pub ai_thinking: bool,
    pub ai_think_ticks: u32,
    pub forfeit_pending: bool,
    pub cursor: (usize, usize),  // If applicable
    // ... game-specific fields
}

impl NewGameGame {
    pub fn new(difficulty: ChallengeDifficulty) -> Self {
        // Map difficulty to game-specific params (board size, AI depth, etc.)
        let board_size = match difficulty {
            ChallengeDifficulty::Novice => 9,
            // ...
        };
        Self { difficulty, game_result: None, /* ... */ }
    }

    /// Game-specific difficulty params as methods on the game struct
    pub fn search_depth(&self) -> i32 { /* ... */ }
    pub fn random_move_chance(&self) -> f64 { /* ... */ }
}
```

### 3. Required Logic (`logic.rs`)

Games use the shared `MinigameInput` enum — no per-game input enum needed.

```rust
use crate::challenges::{ChallengeResult, MinigameInput};

/// Process player input (uses shared MinigameInput)
pub fn process_input(game: &mut NewGameGame, input: MinigameInput) -> bool {
    if game.ai_thinking { return false; }
    match input {
        MinigameInput::Up => { /* move cursor */ }
        MinigameInput::Primary => { /* select/place/submit */ }
        MinigameInput::Secondary => { /* game-specific: pass, flag, clear */ }
        MinigameInput::Cancel => { /* forfeit flow */ }
        MinigameInput::Other => { game.forfeit_pending = false; }
        // ...
    }
    true
}

/// Tick the game (for AI moves, timers)
pub fn process_ai_thinking(game: &mut NewGameGame, rng: &mut impl Rng) -> bool {
    // Handle AI thinking delay, then make AI move
}
```

**Note:** `start_newgame_game()` and `apply_game_result()` are NOT needed per-game. The shared `start_minigame()` and `apply_minigame_result()` in `mod.rs` handle all games.

### 4. Integrate with Menu System (`menu.rs`)

1. Add to `ChallengeType` enum:
```rust
pub enum ChallengeType {
    // ...
    NewGame,
}
```

2. Add reward data to `ChallengeType::reward()`:
```rust
(ChallengeType::NewGame, ChallengeDifficulty::Novice) => ChallengeReward {
    prestige_ranks: 1, xp_percent: 0, fishing_ranks: 0,
},
// ... other difficulties
```

3. Add flavor text to `ChallengeType::result_flavor()` for all 4 result variants.

4. Add `game_type_str()`, `log_icon()`, and optionally `difficulty_extra_info()` cases.

5. Add to `ActiveMinigame` enum in `mod.rs`:
```rust
pub enum ActiveMinigame {
    // ...
    NewGame(NewGameGame),
}
```

6. Add to `start_minigame()` and `apply_minigame_result()` match arms in `mod.rs`.

7. Wire up discovery in `menu.rs`:
   - `create_challenge()` - creates PendingChallenge
   - Discovery weights in `CHALLENGE_TABLE`

### 5. Add UI Scene (`src/ui/newgame_scene.rs`)

Use shared components from `game_common.rs`. Game-over rendering uses `ChallengeResult` and `ChallengeType::reward()` for reward display:

```rust
use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_game_over_overlay,
    render_info_panel_frame, render_status_bar, render_thinking_status_bar,
    GameResultType,
};
use crate::challenges::ChallengeResult;

pub fn render_newgame_scene(frame: &mut Frame, area: Rect, game: &NewGameGame) {
    if game.game_result.is_some() {
        render_game_over(frame, area, game);
        return;
    }
    let layout = create_game_layout(frame, area, " Title ", Color::Cyan, 15, 22);
    render_board(frame, layout.content, game);
    render_status_bar_content(frame, layout.status_bar, game);
    render_info_panel(frame, layout.info_panel, game);
}

fn render_game_over(frame: &mut Frame, area: Rect, game: &NewGameGame) {
    use crate::challenges::menu::ChallengeType;
    let result = game.game_result.unwrap();
    let reward = match result {
        ChallengeResult::Win => ChallengeType::NewGame.reward(game.difficulty).description(),
        _ => "No penalty incurred.".to_string(),
    };
    // ... render with render_game_over_overlay()
}
```

### 6. Wire Up Input Handling (`src/input.rs`)

The unified `handle_minigame()` handles key→`MinigameInput` mapping for all games. Add your game's dispatch case:

```rust
ActiveMinigame::NewGame(game) => {
    newgame::logic::process_input(game, input);
}
```

The game-over check and `apply_minigame_result()` call are handled uniformly for all games — no per-game code needed.

### 7. Add to Debug Menu (`src/utils/debug_menu.rs`)

```rust
pub const DEBUG_OPTIONS: &[&str] = &[
    // ...
    "Trigger NewGame Challenge",
];

fn trigger_newgame_challenge(state: &mut GameState) -> &'static str {
    if state.challenge_menu.has_challenge(&ChallengeType::NewGame) {
        return "NewGame challenge already pending!";
    }
    state.challenge_menu.add_challenge(create_challenge(&ChallengeType::NewGame));
    "NewGame challenge added!"
}
```

## Conventions

### Colors
- Player pieces: `Color::White`
- AI pieces: `Color::LightRed`
- Cursor highlight: `Color::Yellow`
- Last move highlight: `Color::Green`
- Grid/board: `Color::DarkGray`
- Border: Unique per game (Cyan, Green, Yellow, Magenta, etc.)

### AI Thinking
- Set `ai_thinking = true` when AI turn begins
- Increment `ai_think_ticks` each tick
- Execute AI move after delay (typically 5-10 ticks = 0.5-1s)
- Always provide visual feedback via `render_thinking_status_bar`

### Forfeit Flow
1. First Esc press: set `forfeit_pending = true`
2. Second Esc: confirm forfeit, set result to `ChallengeResult::Forfeit`
3. Any other key: cancel forfeit (`forfeit_pending = false`)
4. Use `render_forfeit_status_bar` for consistent UI

### Rewards (`ChallengeReward`)
Defined per game+difficulty in `ChallengeType::reward()`:
```rust
ChallengeReward {
    prestige_ranks: 1,  // Direct prestige gain
    xp_percent: 0,      // % of XP needed for next level
    fishing_ranks: 0,   // Fishing rank gain
}
```

## Discovery Weights

Challenges are discovered randomly (~2hr average). The `CHALLENGE_TABLE` in `menu.rs` controls relative probability:

| Challenge | Weight | ~Probability | Rationale |
|-----------|--------|--------------|-----------|
| Minesweeper | 30 | 27% | Common - quick puzzle |
| Rune | 25 | 23% | Common - quick puzzle |
| Gomoku | 20 | 18% | Moderate |
| Morris | 15 | 14% | Less common |
| Chess | 10 | 9% | Rare - complex strategy |
| Go | 10 | 9% | Rare - complex strategy |

When adding a new challenge, add it to `CHALLENGE_TABLE` with an appropriate weight.

Haven's discovery boost room increases the base discovery chance.

## Achievement Integration

Winning a minigame returns `Some(MinigameWinInfo)` from `apply_minigame_result()` with `game_type` and `difficulty` strings. The achievement system in `src/achievements/` tracks wins per game type and difficulty level. When adding a new challenge, ensure the `game_type_str()` method on `ChallengeType` returns the correct string.

## Existing Challenges

| Challenge | Board | AI Type | Special Features |
|-----------|-------|---------|------------------|
| Chess | 8x8 | chess-engine crate | Move history, piece selection |
| Morris | 24 points | Minimax | Mill detection, 3 phases |
| Gomoku | 15x15 | Minimax (depth 2-5) | Win line detection |
| Minesweeper | Variable | N/A (puzzle) | Flood fill reveal, flags |
| Rune | 4-6 slots | N/A (puzzle) | Mastermind-style feedback |
| Go | 9x9 | MCTS | Captures, ko rule, territory scoring |
