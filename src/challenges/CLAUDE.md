# Challenge Minigames

This module contains player-controlled challenge minigames. Challenges are discovered randomly during gameplay (requires P1+) and appear in the challenge menu for the player to accept or decline.

## Adding a New Challenge

### 1. Module Structure

Create a new subdirectory with three files:

```
src/challenges/newgame/
├── mod.rs      # Public exports
├── types.rs    # Data structures (Game, Difficulty, Result enums)
└── logic.rs    # Game logic, input processing, AI
```

Optional additional files for complex AI:
- `mcts.rs` - Monte Carlo Tree Search (see Go)
- `ai.rs` - Minimax/alpha-beta AI (see Morris, Gomoku)

### 2. Required Types (`types.rs`)

```rust
/// Difficulty levels (typically 4)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NewGameDifficulty {
    Novice,
    Apprentice,
    Journeyman,
    Master,
}

/// Game result (forfeit maps to Loss, not a separate variant)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewGameResult {
    Win,
    Loss,     // Also used for forfeits (UI checks forfeit_pending flag)
    Draw,     // Optional, if applicable
}

/// Main game state
#[derive(Debug, Clone)]
pub struct NewGameGame {
    pub difficulty: NewGameDifficulty,
    pub game_result: Option<NewGameResult>,
    pub ai_thinking: bool,
    pub ai_think_ticks: u32,
    pub forfeit_pending: bool,
    pub cursor: (usize, usize),  // If applicable
    // ... game-specific fields
}
```

### 3. Required Logic (`logic.rs`)

```rust
/// UI-agnostic input enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewGameInput {
    Up, Down, Left, Right,
    Select,   // Enter
    Forfeit,  // Esc (triggers forfeit flow)
    Other,
}

/// Start a new game
pub fn start_newgame_game(difficulty: NewGameDifficulty) -> ActiveMinigame {
    ActiveMinigame::NewGame(NewGameGame::new(difficulty))
}

/// Process player input
pub fn process_input(game: &mut NewGameGame, input: NewGameInput) {
    // Handle forfeit_pending state first
    // Then handle normal input
}

/// Apply game result — use the impl_apply_game_result! macro (see below)

/// Tick the game (for AI moves, timers)
pub fn tick_game(game: &mut NewGameGame) {
    // Handle AI thinking delay, then make AI move
}
```

### `facade.rs`

Defines `tick_challenge_ai_facade()`, a decoupled entry point for ticking active-game AI thinking (mirrors `fishing/facade.rs`'s `tick_fishing_facade()`). Not currently called in production — `tick_stages.rs` Stage 1 duplicates the chess/morris/gomoku/go dispatch logic inline instead of calling the facade.

### `impl_apply_game_result!` Macro (`mod.rs`)

All 14 challenge types use the `impl_apply_game_result!` macro in `mod.rs` to generate their `apply_game_result()` function. Instead of manually implementing reward logic, invoke the macro:

```rust
impl_apply_game_result! {
    fn apply_newgame_result;
    variant: NewGame;
    result_body: |result, state, reward| {
        use newgame::types::NewGameResult;
        let (won, loss_message) = match result {
            NewGameResult::Win => (true, ""),
            NewGameResult::Loss => (false, "Defeated!"),
        };
        (won, loss_message)
    }
    game_type: crate::achievements::MinigameType::NewGame;
    icon: "\u{265F}";           // Unicode icon for combat log
    win_message: "Victory!";
}
```

The macro generates `apply_newgame_result(state) -> Option<MinigameWinInfo>` which extracts the game result, applies rewards via `apply_challenge_rewards()`, and emits `MinigameWinInfo` for achievement tracking. Omitting the `fn apply_newgame_result;` line generates the default name `apply_game_result`.

### 4. Integrate with Menu System (`menu.rs`)

1. Add to `ChallengeType` enum:
```rust
pub enum ChallengeType {
    // ...
    NewGame,
}
```

2. Implement `DifficultyInfo` trait for your difficulty enum:
```rust
impl DifficultyInfo for NewGameDifficulty {
    fn name(&self) -> &'static str { ... }
    fn reward(&self) -> ChallengeReward { ... }
    fn extra_info(&self) -> Option<String> { None }
}
```

3. Add to `ActiveMinigame` enum in `mod.rs`:
```rust
pub enum ActiveMinigame {
    // ...
    NewGame(NewGameGame),
}
```

4. Wire up in `menu.rs`:
   - `create_challenge()` - creates Challenge with difficulties
   - `accept_selected_challenge()` - starts the game
   - Discovery weights in `CHALLENGE_TABLE`

### 5. Add UI Scene (`src/ui/newgame_scene.rs`)

Use shared components from `game_common.rs`:

```rust
use super::game_common::{
    create_game_layout,
    render_forfeit_status_bar,
    render_game_over_overlay,
    render_info_panel_frame,
    render_status_bar,
    render_thinking_status_bar,
    GameResultType,
};

pub fn render_newgame_scene(frame: &mut Frame, area: Rect, game: &NewGameGame) {
    // 1. Check for game over overlay first
    if game.game_result.is_some() {
        render_game_over(frame, area, game);
        return;
    }

    // 2. Create layout (title, border_color, content_height, info_panel_width)
    let layout = create_game_layout(frame, area, " Title ", Color::Cyan, 15, 22);

    // 3. Render components
    render_board(frame, layout.content, game);
    render_status_bar_content(frame, layout.status_bar, game);
    render_info_panel(frame, layout.info_panel, game);
}

fn render_status_bar_content(frame: &mut Frame, area: Rect, game: &NewGameGame) {
    // AI thinking state
    if game.ai_thinking {
        render_thinking_status_bar(frame, area, "Opponent is thinking...");
        return;
    }

    // Forfeit confirmation
    if render_forfeit_status_bar(frame, area, game.forfeit_pending) {
        return;
    }

    // Normal controls
    render_status_bar(frame, area, "Your turn", Color::White, &[
        ("[Arrows]", "Move"),
        ("[Enter]", "Select"),
        ("[Esc]", "Forfeit"),
    ]);
}
```

### 6. Wire Up Input Handling (`src/input/minigame_input.rs`)

Add case to `handle_minigame()`:

```rust
ActiveMinigame::NewGame(game) => {
    if game.game_result.is_some() {
        apply_newgame_result(state);
        return InputResult::Continue;
    }
    let input = match key.code {
        KeyCode::Up => NewGameInput::Up,
        // ... map all inputs
        _ => NewGameInput::Other,
    };
    process_newgame_input(game, input);
}
```

### 7. Add to Debug Menu (`src/utils/debug_menu.rs`)

Add a variant to the `DebugAction` enum and include it in the `CHALLENGE_ACTIONS: &[DebugAction]` const array, then add its label match arm and dispatch match arm:

```rust
enum DebugAction {
    // ...
    TriggerNewGameChallenge,
}

const CHALLENGE_ACTIONS: &[DebugAction] = &[
    // ...
    DebugAction::TriggerNewGameChallenge,
];

// label match arm:
DebugAction::TriggerNewGameChallenge => "Trigger NewGame Challenge",

// dispatch match arm:
DebugAction::TriggerNewGameChallenge => {
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
2. Second Esc: confirm forfeit, set result to `Loss` (not a separate Forfeit variant)
3. Any other key: cancel forfeit (`forfeit_pending = false`)
4. Use `render_forfeit_status_bar` for consistent UI
5. UI checks `forfeit_pending` flag on Loss result to display "Forfeit" vs "Defeat"

### Quit Confirmation
When the player quits the game while challenges are pending, a confirmation dialog appears warning that pending challenges will be lost. [Enter] leaves, [Esc] stays.

### AI Thinking
All games with AI use a standardized `process_ai_thinking()` function name (not game-specific names like `process_go_ai`).

### Rewards (`ChallengeReward`)
```rust
ChallengeReward {
    prestige_ranks: 1,     // Direct prestige gain
    stormglass: 4_000,     // Stormglass currency (XP fallback if not yet discovered)
    fishing_ranks: 0,      // Fishing rank gain
}
```

All challenges award Stormglass currency. Higher difficulties give more Stormglass (e.g., Novice 300-1000, Master 4000-12500). If the player has not yet discovered Stormglass, the reward falls back to XP.

## Discovery Weights

Challenges are discovered randomly (~2hr average). The `CHALLENGE_TABLE` in `menu.rs` controls relative probability:

| Challenge | Weight | ~Probability | Rationale |
|-----------|--------|--------------|-----------|
| Rune | 30 | ~12% | Fastest (~2 min) |
| Minesweeper | 28 | ~11% | Fast puzzle |
| Snake | 22 | ~9% | Quick action |
| Flappy Bird | 20 | ~8% | Moderate action |
| Sigil Surge | 20 | ~8% | Moderate action-puzzle |
| Shard Fusion | 20 | ~8% | Moderate puzzle (2048-style) |
| Runic Lights | 20 | ~8% | Moderate puzzle |
| JezzBall | 18 | ~7% | Moderate action |
| Sudoku | 18 | ~7% | Moderate puzzle |
| Vault Warden | 18 | ~7% | Moderate puzzle |
| Gomoku | 15 | ~6% | Medium-length strategy |
| Morris | 12 | ~5% | Longer strategy |
| Chess | 8 | ~3% | Long commitment |
| Go | 7 | ~3% | Longest game |

When adding a new challenge, add it to `CHALLENGE_TABLE` with an appropriate weight.

Haven's discovery boost room increases the base discovery chance.

## Achievement Integration

Winning a minigame emits a `MinigameWinInfo` (defined in `mod.rs`) with `game_type` and `difficulty` strings. The achievement system in `src/achievements/` tracks wins per game type and difficulty level. When adding a new challenge, ensure `MinigameWinInfo` values are emitted in `apply_game_result()`.

## Existing Challenges

| Challenge | Board | AI Type | Special Features |
|-----------|-------|---------|------------------|
| Chess | 8x8 | chess-engine crate | Move history, piece selection |
| Morris | 24 points | Minimax (`ai.rs`) | Mill detection, 3 phases |
| Gomoku | 15x15 | Minimax depth 2-5 (`ai.rs`) | Win line detection |
| Minesweeper | Variable | N/A (puzzle) | Flood fill reveal, flags |
| Rune | 3-5 slots | N/A (puzzle) | Mastermind-style feedback |
| Go | 9x9 | MCTS | Captures, ko rule, territory scoring |
| Snake (Serpent's Path) | 26×26 grid | N/A (action) | Real-time ~60 FPS, direction-based movement, 4 difficulties (Novice 10 food/200ms, Master 25 food/90ms), requires P1+ |
| Flappy Bird (Skyward Gauntlet) | 50×18 area | N/A (action) | Real-time ~60 FPS, gravity/flap physics, pipe obstacles with gap sizes (7→4 rows), 3 lives, 4 difficulties, requires P1+ |
| JezzBall (Containment Breach) | 34×22 grid | N/A (action) | Real-time ~60 FPS, ball physics, wall-building to capture area, 3 lives, 2-5 balls (Novice→Master), target 60-84%, 4 difficulties, requires P1+ |
| Sigil Surge (Runic Shift) | 6×12 grid | N/A (action-puzzle) | Real-time ~60 FPS, panel-matching with 5 rune colors, 3 lives, rising blocks (7000-3000ms interval), chain combos, 4 difficulties, requires P1+ |
| Sudoku (Sigil Matrix) | 9×9 grid | N/A (puzzle) | Classic Sudoku with 4 difficulties, pencil marks, cursor navigation, requires P1+ |
| Shard Fusion | 4×4 grid | N/A (puzzle) | 2048-style tile merging, target values (256-2048 by difficulty), slide animations, 4 difficulties, requires P1+ |
| Runic Lights | Variable | N/A (puzzle) | Light-pattern matching puzzle, 4 difficulties, requires P1+ |
| Vault Warden | Variable | N/A (puzzle) | Vault security puzzle, 4 difficulties, requires P1+ |
