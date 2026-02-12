# Flappy Bird Architecture: Real-Time Game Loop Integration

This document describes how to integrate a real-time action minigame (Flappy Bird) into Quest's existing 100ms tick-based game loop. The key challenge is running Flappy Bird at 30+ FPS while keeping all other game systems (combat, fishing, dungeons) at their normal 100ms tick rate.

## Design Principle: Single-Threaded, Adaptive Tick Rate

The approach is **single-threaded with adaptive tick interval**. When Flappy Bird is active, the main loop runs at ~33ms (30 FPS) instead of the normal 100ms. The normal game tick (`game_tick()`) still runs at its usual 100ms cadence via an accumulator, while Flappy Bird gets its own higher-frequency update. No threads, no async, no complexity.

---

## 1. Game Loop Changes (`src/main.rs`)

### Current Loop Structure (simplified)

```rust
// Current: fixed 50ms poll, 100ms tick
loop {
    terminal.draw(|frame| { /* ... */ })?;

    if event::poll(Duration::from_millis(50))? {
        // handle one key event
    }

    if last_tick.elapsed() >= Duration::from_millis(TICK_INTERVAL_MS) {
        game_tick(/* ... */);
        last_tick = Instant::now();
    }

    // autosave, update check, etc.
}
```

### Modified Loop: Adaptive Timing

Add a helper to detect real-time minigame mode:

```rust
/// Returns true if the active minigame requires real-time (high FPS) updates.
fn is_realtime_minigame(state: &GameState) -> bool {
    matches!(
        state.active_minigame,
        Some(ActiveMinigame::FlappyBird(_))
    )
}
```

The inner game loop in `Screen::Game` changes to:

```rust
// New constants (add to core/constants.rs)
pub const REALTIME_FRAME_MS: u64 = 33; // ~30 FPS for action games

// Modified game loop
let mut last_flappy_frame = Instant::now();

loop {
    let realtime_mode = is_realtime_minigame(&state);

    // ── 1. Render ──────────────────────────────────────────
    terminal.draw(|frame| {
        draw_ui_with_update(frame, &state, /* ... */);
        // overlays...
    })?;

    // ── 2. Input: drain ALL events in realtime mode ────────
    let poll_duration = if realtime_mode {
        Duration::ZERO  // Non-blocking: drain all pending events
    } else {
        Duration::from_millis(50)  // Normal: block up to 50ms
    };

    // Drain all available events (critical for responsive input at 30+ FPS)
    while event::poll(poll_duration)? {
        if let Event::Key(key_event) = event::read()? {
            if key_event.kind != KeyEventKind::Press {
                continue;
            }
            // ... existing input handling (handle_game_input) ...
        }
        // After first event in realtime mode, switch to non-blocking
        // to drain remaining events without waiting
        if realtime_mode {
            // Continue draining with ZERO timeout
        } else {
            break; // Normal mode: process one event per frame
        }
    }

    // ── 3. Suspension detection ────────────────────────────
    // (unchanged - runs regardless of mode)

    // ── 4. Normal game tick at 100ms (always) ──────────────
    if last_tick.elapsed() >= Duration::from_millis(TICK_INTERVAL_MS) {
        if !matches!(overlay, GameOverlay::LeviathanEncounter { .. }) {
            let mut rng = rand::thread_rng();
            let tick_result = core::tick::game_tick(
                &mut state,
                &mut tick_counter,
                &mut haven,
                &mut global_achievements,
                debug_mode,
                &mut rng,
            );
            // ... existing tick result handling ...
        }
        last_tick = Instant::now();
    }

    // ── 5. Flappy Bird real-time tick (33ms) ───────────────
    if realtime_mode {
        let dt = last_flappy_frame.elapsed();
        if dt >= Duration::from_millis(REALTIME_FRAME_MS) {
            if let Some(ActiveMinigame::FlappyBird(ref mut game)) = state.active_minigame {
                let dt_secs = dt.as_secs_f64();
                crate::challenges::flappy::logic::tick(game, dt_secs);
            }
            last_flappy_frame = Instant::now();
        }
    }

    // ── 6. Autosave, update checks ─────────────────────────
    // (unchanged)
}
```

### Key Design Decisions

1. **Event draining**: In realtime mode, we use `Duration::ZERO` and drain ALL pending key events each frame. This prevents input lag — if the player presses Space rapidly, all presses are processed before the next render.

2. **Two tick timers**: `last_tick` controls the 100ms game tick (combat, fishing, etc.), while `last_flappy_frame` controls the 33ms Flappy Bird physics tick. Both run independently.

3. **game_tick() still runs at 100ms**: The existing `game_tick()` function continues to run on its normal schedule, even during Flappy Bird. This means challenge discovery, play time tracking, and achievement accumulation all work normally. The only change is that `game_tick()` should **skip AI thinking for FlappyBird** (it has no AI thinking phase).

4. **Poll duration**: Normal mode polls at 50ms (responsive but CPU-friendly). Realtime mode polls at 0ms (non-blocking) to maximize frame rate.

---

## 2. Tick Integration (`src/core/tick.rs`)

### Changes to `game_tick()`

The `game_tick()` function's Section 1 (challenge AI thinking) already uses a `match` over `ActiveMinigame` variants. FlappyBird simply has no AI thinking, so it falls through to the default `_ => {}` arm. No changes needed there.

However, game_tick() should be aware that FlappyBird is active for discovery suppression:

```rust
// In game_tick(), the existing Section 1 already handles this correctly:
match &mut state.active_minigame {
    Some(ActiveMinigame::Chess(game)) => { /* ... */ }
    Some(ActiveMinigame::Morris(game)) => { /* ... */ }
    Some(ActiveMinigame::Gomoku(game)) => { /* ... */ }
    Some(ActiveMinigame::Go(game)) => { /* ... */ }
    // FlappyBird has no AI thinking — falls through to:
    _ => {}
}
```

**FlappyBird ticks are NOT called from game_tick().** They are called directly from main.rs at the higher frame rate (see Section 1 above). This is the critical distinction — `game_tick()` runs at 100ms for all the normal game systems, while FlappyBird's physics run at 33ms from the main loop.

The 100ms game_tick() continues to:
- Track play time (tick_counter)
- Run challenge discovery rolls
- Process fishing/combat (mutually exclusive with FlappyBird)
- Check Haven discovery
- Accumulate achievement modals

None of these need modification — FlappyBird is just another `ActiveMinigame` variant, and the existing `active_minigame.is_some()` guards already prevent conflicting activities.

---

## 3. Module Structure

```
src/challenges/flappy/
├── mod.rs      # Public exports
├── types.rs    # FlappyBirdGame, FlappyBirdDifficulty, FlappyBirdResult, Pipe
└── logic.rs    # Physics, collision, input processing, tick(), apply_game_result()
```

### Key Structs (`types.rs`)

```rust
use serde::{Deserialize, Serialize};

/// Difficulty levels following the standard 4-tier pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlappyBirdDifficulty {
    Novice,
    Apprentice,
    Journeyman,
    Master,
}
// Use difficulty_enum_impl!(FlappyBirdDifficulty); macro from challenges/mod.rs

/// Game outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlappyBirdResult {
    Win,   // Survived the required duration / passed enough pipes
    Loss,  // Hit a pipe or floor/ceiling
}

/// A single pipe obstacle (top and bottom pair)
#[derive(Debug, Clone)]
pub struct Pipe {
    /// X position in game-world coordinates (scrolls left)
    pub x: f64,
    /// Y position of the gap center (0.0 = top, 1.0 = bottom)
    pub gap_center_y: f64,
    /// Whether the player has passed this pipe (for scoring)
    pub passed: bool,
}

/// Main game state
#[derive(Debug, Clone)]
pub struct FlappyBirdGame {
    // ── Difficulty ──
    pub difficulty: FlappyBirdDifficulty,

    // ── Bird state ──
    /// Bird Y position (0.0 = top, 1.0 = bottom of play area)
    pub bird_y: f64,
    /// Bird Y velocity (positive = downward)
    pub bird_vel: f64,

    // ── World state ──
    /// Active pipes in the world
    pub pipes: Vec<Pipe>,
    /// Distance until next pipe spawns
    pub next_pipe_distance: f64,
    /// Total elapsed game time in seconds
    pub elapsed: f64,
    /// Scroll speed (game-world units per second)
    pub scroll_speed: f64,

    // ── Scoring ──
    /// Number of pipes successfully passed
    pub score: u32,
    /// Target score to win (depends on difficulty)
    pub target_score: u32,

    // ── Game flow ──
    pub game_result: Option<FlappyBirdResult>,
    pub forfeit_pending: bool,

    // ── Rendering hints ──
    /// Frames since last flap (for wing animation)
    pub flap_animation_timer: f64,
}
```

### Difficulty Parameters

| Parameter | Novice | Apprentice | Journeyman | Master |
|-----------|--------|------------|------------|--------|
| Gravity | 1.2 | 1.5 | 1.8 | 2.2 |
| Flap Impulse | -0.45 | -0.42 | -0.40 | -0.38 |
| Scroll Speed | 0.3 | 0.4 | 0.5 | 0.6 |
| Gap Size | 0.35 | 0.30 | 0.25 | 0.20 |
| Pipe Spacing | 0.5 | 0.45 | 0.40 | 0.35 |
| Target Score | 10 | 15 | 20 | 30 |

(All values are in normalized coordinates where 1.0 = play area height/width. These will be tuned during implementation.)

### Public API (`mod.rs`)

```rust
pub mod logic;
pub mod types;

pub use types::{FlappyBirdDifficulty, FlappyBirdGame, FlappyBirdResult};
```

### Logic Functions (`logic.rs`)

```rust
use super::types::*;
use crate::challenges::{ActiveMinigame, GameResultInfo, MinigameWinInfo};
use crate::core::game_state::GameState;

/// Input actions for Flappy Bird (UI-agnostic)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlappyBirdInput {
    Flap,    // Space or Up
    Forfeit, // Esc
    Other,
}

/// Create a new Flappy Bird game as an ActiveMinigame
pub fn start_flappy_game(difficulty: FlappyBirdDifficulty) -> ActiveMinigame {
    ActiveMinigame::FlappyBird(FlappyBirdGame::new(difficulty))
}

/// Process player input (called from input.rs)
pub fn process_input(game: &mut FlappyBirdGame, input: FlappyBirdInput) {
    if game.game_result.is_some() {
        return; // Game over — any key dismisses (handled by input.rs)
    }

    match input {
        FlappyBirdInput::Flap => {
            if game.forfeit_pending {
                game.forfeit_pending = false; // Cancel forfeit
            } else {
                game.bird_vel = game.flap_impulse(); // Apply upward velocity
                game.flap_animation_timer = 0.0;
            }
        }
        FlappyBirdInput::Forfeit => {
            if game.forfeit_pending {
                game.game_result = Some(FlappyBirdResult::Loss); // Confirm forfeit
            } else {
                game.forfeit_pending = true;
            }
        }
        FlappyBirdInput::Other => {
            if game.forfeit_pending {
                game.forfeit_pending = false; // Cancel forfeit
            }
        }
    }
}

/// Physics tick — called at 30+ FPS from main.rs
/// `dt` is the actual elapsed time in seconds since last tick
pub fn tick(game: &mut FlappyBirdGame, dt: f64) {
    if game.game_result.is_some() {
        return;
    }

    // Clamp dt to prevent physics explosion after pause/lag
    let dt = dt.min(0.1);

    game.elapsed += dt;
    game.flap_animation_timer += dt;

    // ── Bird physics ──
    let gravity = game.gravity();
    game.bird_vel += gravity * dt;
    game.bird_y += game.bird_vel * dt;

    // ── Pipe scrolling ──
    let speed = game.scroll_speed;
    for pipe in &mut game.pipes {
        pipe.x -= speed * dt;
    }

    // ── Score: check pipes the bird has passed ──
    let bird_x = 0.15; // Bird is at fixed X position (15% from left)
    for pipe in &mut game.pipes {
        if !pipe.passed && pipe.x + 0.05 < bird_x {
            pipe.passed = true;
            game.score += 1;
        }
    }

    // ── Spawn new pipes ──
    game.next_pipe_distance -= speed * dt;
    if game.next_pipe_distance <= 0.0 {
        game.spawn_pipe();
        game.next_pipe_distance = game.pipe_spacing();
    }

    // ── Remove off-screen pipes ──
    game.pipes.retain(|p| p.x > -0.2);

    // ── Collision detection ──
    // Floor/ceiling
    if game.bird_y <= 0.0 || game.bird_y >= 1.0 {
        game.game_result = Some(FlappyBirdResult::Loss);
        return;
    }

    // Pipe collision
    let bird_half_size = 0.03;
    let pipe_half_width = 0.025;
    for pipe in &game.pipes {
        let pipe_left = pipe.x - pipe_half_width;
        let pipe_right = pipe.x + pipe_half_width;

        // Bird overlaps pipe horizontally?
        if bird_x + bird_half_size > pipe_left && bird_x - bird_half_size < pipe_right {
            let gap_half = game.gap_size() / 2.0;
            let gap_top = pipe.gap_center_y - gap_half;
            let gap_bottom = pipe.gap_center_y + gap_half;

            // Bird outside the gap?
            if game.bird_y - bird_half_size < gap_top
                || game.bird_y + bird_half_size > gap_bottom
            {
                game.game_result = Some(FlappyBirdResult::Loss);
                return;
            }
        }
    }

    // ── Win condition ──
    if game.score >= game.target_score {
        game.game_result = Some(FlappyBirdResult::Win);
    }
}

/// Apply game result using the shared challenge reward system
pub fn apply_game_result(state: &mut GameState) -> Option<MinigameWinInfo> {
    let (result, difficulty) = {
        if let Some(ActiveMinigame::FlappyBird(ref game)) = state.active_minigame {
            (game.game_result, game.difficulty)
        } else {
            return None;
        }
    };

    let won = matches!(result, Some(FlappyBirdResult::Win));
    let reward = difficulty.reward();

    crate::challenges::apply_challenge_rewards(
        state,
        GameResultInfo {
            won,
            game_type: "flappy",
            difficulty_str: difficulty.difficulty_str(),
            reward,
            icon: "\u{1F426}",  // bird emoji
            win_message: "You navigated the gauntlet!",
            loss_message: "The bird crashes into an obstacle...",
        },
    )
}
```

---

## 4. Input Architecture (`src/input.rs`)

### Changes to `handle_minigame()`

Add a new arm in the `match minigame` block:

```rust
ActiveMinigame::FlappyBird(flappy_game) => {
    if flappy_game.game_result.is_some() {
        // Game over: any key dismisses
        state.last_minigame_win = apply_flappy_result(state);
        return InputResult::Continue;
    }
    let input = match key.code {
        KeyCode::Char(' ') | KeyCode::Up => FlappyBirdInput::Flap,
        KeyCode::Esc => FlappyBirdInput::Forfeit,
        _ => FlappyBirdInput::Other,
    };
    process_flappy_input(flappy_game, input);
}
```

### Input Responsiveness

At 30+ FPS with event draining, input is processed every ~33ms. The player presses Space to flap, and ALL pending Space presses are processed in the same frame. This prevents the "missed flap" problem that would occur at 100ms ticks.

Import additions at the top of `input.rs`:

```rust
use crate::challenges::flappy::logic::{
    apply_game_result as apply_flappy_result,
    process_input as process_flappy_input,
    FlappyBirdInput,
};
```

---

## 5. Rendering Pipeline (`src/ui/flappy_scene.rs`)

### Integration with `ui/mod.rs`

Add to `draw_right_content()`:

```rust
fn draw_right_content(frame: &mut Frame, area: Rect, game_state: &GameState) {
    match &game_state.active_minigame {
        // ... existing minigame arms ...
        Some(ActiveMinigame::FlappyBird(game)) => {
            flappy_scene::render_flappy_scene(frame, area, game);
        }
        None => { /* ... existing fallback ... */ }
    }
}
```

Add module declaration in `ui/mod.rs`:

```rust
pub mod flappy_scene;
```

### Scene Structure

```rust
use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_game_over_overlay,
    render_status_bar, GameResultType,
};
use crate::challenges::flappy::types::{FlappyBirdGame, FlappyBirdResult};
use ratatui::{/* ... */};

pub fn render_flappy_scene(frame: &mut Frame, area: Rect, game: &FlappyBirdGame) {
    // 1. Game over overlay
    if let Some(result) = game.game_result {
        render_game_over(frame, area, game, result);
        return;
    }

    // 2. Create layout using shared game layout
    //    Smaller info panel (16 wide) since Flappy Bird info is minimal
    let layout = create_game_layout(frame, area, " Flappy Bird ", Color::LightCyan, 15, 16);

    // 3. Render the play field (bird, pipes, ground)
    render_play_field(frame, layout.content, game);

    // 4. Status bar
    render_status_bar_content(frame, layout.status_bar, game);

    // 5. Info panel (score, target, difficulty)
    render_info_panel(frame, layout.info_panel, game);
}
```

### Play Field Rendering

The play field is rendered as ASCII art within the content area:

```
Bird position mapped to terminal rows:
- Content area height = N rows
- bird_y (0.0 to 1.0) maps to row 0..N

Pipes rendered as vertical columns:
- Each pipe is 1-2 columns wide
- Gap is rendered as empty space
- Pipe body rendered with block characters (e.g., '|', '#', or box-drawing)

Bird rendered as a small ASCII sprite:
- Normal: >o  or  >O  (2 chars)
- Flapping: >'  or  >"  (wing animation)
```

**Rendering coordinates**: The game uses normalized coordinates (0.0 to 1.0) internally. The renderer maps these to terminal cells based on the actual content area size:

```rust
fn world_to_screen_y(world_y: f64, area_height: u16) -> u16 {
    (world_y * area_height as f64).round() as u16
}

fn world_to_screen_x(world_x: f64, area_width: u16) -> u16 {
    (world_x * area_width as f64).round() as u16
}
```

### Performance Notes

- **Zero allocation per frame**: Pre-compute pipe screen positions using iterators, not Vec allocations
- **Minimal widget creation**: Render directly to buffer cells where possible using `frame.buffer_mut()`, or use a single `canvas::Canvas` widget
- **Simple sprites**: Bird is 2-3 chars, pipes are single-column line draws. This is far simpler than the existing dungeon 3D renderer

---

## 6. Integration Touchpoints

Every file that needs modification, with specific changes:

### `src/challenges/flappy/mod.rs` (NEW)
- Create module with `pub mod logic; pub mod types;`
- Re-export public types

### `src/challenges/flappy/types.rs` (NEW)
- `FlappyBirdDifficulty` enum with `difficulty_enum_impl!` macro
- `FlappyBirdResult` enum
- `FlappyBirdGame` struct
- `Pipe` struct
- `impl FlappyBirdGame` with `new()`, difficulty parameter accessors

### `src/challenges/flappy/logic.rs` (NEW)
- `FlappyBirdInput` enum
- `start_flappy_game()` function
- `process_input()` function
- `tick()` function (physics update)
- `apply_game_result()` function using shared `apply_challenge_rewards()`

### `src/challenges/mod.rs` (MODIFY)
- Add `pub mod flappy;` to module declarations
- Add `pub use flappy::{FlappyBirdDifficulty, FlappyBirdGame, FlappyBirdResult};` to re-exports
- Add `FlappyBird(FlappyBirdGame)` variant to `ActiveMinigame` enum

### `src/challenges/menu.rs` (MODIFY)
- Add `use super::flappy::{FlappyBirdDifficulty, FlappyBirdGame};` import
- Add `FlappyBird` variant to `ChallengeType` enum
- Implement `DifficultyInfo` for `FlappyBirdDifficulty`
- Add `FlappyBird` arm to `accept_selected_challenge()`
- Add `FlappyBird` entry to `CHALLENGE_TABLE` (weight: ~15, moderate)
- Add `FlappyBird` arm to `ChallengeType::icon()` (use bird emoji)
- Add `FlappyBird` arm to `ChallengeType::discovery_flavor()`
- Add `FlappyBird` arm to `create_challenge()`

### `src/core/constants.rs` (MODIFY)
- Add `pub const REALTIME_FRAME_MS: u64 = 33;` for 30 FPS target

### `src/core/tick.rs` (NO CHANGES NEEDED)
- FlappyBird falls through the existing `_ => {}` arm in challenge AI thinking
- All other systems continue at 100ms as normal
- FlappyBird physics are called from main.rs, not from game_tick()

### `src/main.rs` (MODIFY)
- Add `is_realtime_minigame()` helper function
- Add `last_flappy_frame` Instant tracker in `Screen::Game` block
- Change event polling to use `Duration::ZERO` + drain loop in realtime mode
- Add Flappy Bird tick call after the normal game tick
- Import `REALTIME_FRAME_MS` from constants

### `src/input.rs` (MODIFY)
- Add imports for `FlappyBirdInput`, `process_flappy_input`, `apply_flappy_result`
- Add `ActiveMinigame::FlappyBird(game) => { ... }` arm to `handle_minigame()`
- Map `Space`/`Up` to `FlappyBirdInput::Flap`, `Esc` to `Forfeit`

### `src/ui/mod.rs` (MODIFY)
- Add `pub mod flappy_scene;` to module declarations
- Add `Some(ActiveMinigame::FlappyBird(game))` arm to `draw_right_content()`

### `src/ui/flappy_scene.rs` (NEW)
- `render_flappy_scene()` using `create_game_layout()`
- `render_play_field()` for bird + pipes ASCII rendering
- `render_status_bar_content()` with forfeit handling
- `render_info_panel()` with score/target display
- `render_game_over()` using `render_game_over_overlay()`

### `src/utils/debug_menu.rs` (MODIFY)
- Add `"Trigger Flappy Bird Challenge"` to `DEBUG_OPTIONS` array
- Add `trigger_flappy_challenge()` function (follow existing pattern)
- Add arm to `trigger_selected()` match (index shifts existing entries or insert at appropriate position)

---

## 7. Performance Requirements

### Target: 30 FPS (33ms frame budget)

The 33ms budget breaks down as:
- **Render**: ~5-15ms (Ratatui terminal draw)
- **Input**: ~0-1ms (event drain)
- **Physics**: ~0-1ms (simple arithmetic)
- **game_tick()**: ~1-5ms (only runs every 3rd frame)
- **Headroom**: ~10-25ms

This is well within budget. For comparison, the existing Go minigame's MCTS AI can consume 20k+ simulations per tick, which is far more expensive than Flappy Bird's physics.

### Zero Allocation in Hot Path

- `Pipe` structs are stored in a pre-allocated `Vec<Pipe>` that grows to ~10 elements and stays there
- Pipes are removed from the front with `retain()` (no allocation if capacity is sufficient)
- New pipes are pushed to the back (amortized O(1))
- Bird state is updated in-place (no allocation)
- Screen coordinates are computed on the stack

### Minimize Ratatui Widget Creation

- Use `create_game_layout()` once per frame (standard pattern)
- Render pipes and bird directly into the buffer using `Paragraph` with pre-built `Line`/`Span` arrays
- Avoid creating new `String` objects per frame — use `write!` to a reusable buffer or format into stack-allocated arrays
- The info panel updates only when score changes (but since Ratatui redraws fully each frame, just keep it simple)

### Frame Rate Stability

- `dt` is clamped to 0.1s to prevent physics explosion after lag spikes or process suspension
- If a frame takes longer than 33ms, the next frame runs immediately (no accumulation of missed frames)
- The `last_flappy_frame` timer resets each tick, providing natural frame rate limiting

---

## 8. Rewards Structure

Following the existing challenge reward pattern:

| Difficulty | Prestige | XP% | Fishing | Target Score |
|-----------|----------|-----|---------|-------------|
| Novice | 0 | 50 | 0 | 10 pipes |
| Apprentice | 0 | 100 | 0 | 15 pipes |
| Journeyman | 1 | 50 | 0 | 20 pipes |
| Master | 2 | 100 | 1 | 30 pipes |

These follow the same progression curve as Gomoku rewards (scaled by difficulty).

---

## 9. Summary: What Stays the Same

- `game_tick()` is unchanged — runs at 100ms, handles all normal game systems
- Autosave runs at its normal 30s interval
- Achievement accumulation and modal system work normally
- Suspension detection works normally
- Update checks work normally
- Haven, fishing, dungeon discovery continue to roll per tick
- The only difference during Flappy Bird is: faster poll + faster render + Flappy physics tick
