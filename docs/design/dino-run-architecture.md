# Dino Run Architecture: Real-Time Endless Runner Integration

This document describes how to integrate Dino Run (a Chrome dinosaur-style endless runner) into Quest's existing game loop. Like Flappy Bird, Dino Run is a real-time action minigame that runs at high FPS while the rest of the game continues at 100ms ticks.

The real-time infrastructure (adaptive polling, dual-tick timers, event draining) already exists from the Flappy Bird implementation. Dino Run reuses this exact infrastructure by adding a new arm to `is_realtime_minigame()`.

---

## 1. Module Structure

```
src/challenges/dino/
├── mod.rs      # Public exports
├── types.rs    # DinoRunGame, DinoRunDifficulty, DinoRunResult, Obstacle, ObstacleType
└── logic.rs    # Physics, collision, input processing, tick_dino_run(), apply_game_result()
```

---

## 2. Type Definitions (`types.rs`)

### DinoRunDifficulty

```rust
use serde::{Deserialize, Serialize};

/// Difficulty levels for Dino Run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DinoRunDifficulty {
    Novice,
    Apprentice,
    Journeyman,
    Master,
}

difficulty_enum_impl!(DinoRunDifficulty);
```

### DinoRunResult

```rust
/// Game outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DinoRunResult {
    Win,   // Reached target score
    Loss,  // Hit an obstacle (or forfeited)
}
```

### ObstacleType

```rust
/// Types of obstacles the runner must avoid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObstacleType {
    // Ground obstacles — jump over these
    SmallRock,     // 1 row tall, 2 cols wide
    LargeRock,     // 2 rows tall, 2 cols wide
    Cactus,        // 2 rows tall, 1 col wide
    DoubleCactus,  // 2 rows tall, 3 cols wide
    // Flying obstacles — duck under these
    Bat,           // 1 row tall, 2 cols wide, at head height
    Stalactite,    // 1 row tall, 3 cols wide, at head height
}
```

Each variant knows its hitbox:

```rust
impl ObstacleType {
    /// Width in columns.
    pub fn width(&self) -> u16 {
        match self {
            Self::SmallRock => 2,
            Self::LargeRock => 2,
            Self::Cactus => 1,
            Self::DoubleCactus => 3,
            Self::Bat => 2,
            Self::Stalactite => 3,
        }
    }

    /// Height in rows.
    pub fn height(&self) -> u16 {
        match self {
            Self::SmallRock => 1,
            Self::LargeRock => 2,
            Self::Cactus => 2,
            Self::DoubleCactus => 2,
            Self::Bat => 1,
            Self::Stalactite => 1,
        }
    }

    /// True if this obstacle is airborne (duck to avoid).
    pub fn is_flying(&self) -> bool {
        matches!(self, Self::Bat | Self::Stalactite)
    }
}
```

### Obstacle

```rust
/// A single obstacle in the game world.
#[derive(Debug, Clone)]
pub struct Obstacle {
    /// X position (float for smooth scrolling, cols from left edge).
    pub x: f64,
    /// The type of obstacle (determines hitbox and rendering).
    pub obstacle_type: ObstacleType,
    /// Whether the runner has cleared this obstacle (for scoring).
    pub passed: bool,
}
```

### Game Constants

```rust
/// Game area dimensions.
pub const GAME_WIDTH: u16 = 60;
pub const GAME_HEIGHT: u16 = 18;

/// Ground row (bottom of play area, 0-indexed). Runner stands here.
pub const GROUND_ROW: u16 = 15;

/// Runner fixed horizontal column position (left edge of runner).
pub const RUNNER_COL: u16 = 6;

/// Runner dimensions.
pub const RUNNER_WIDTH: u16 = 2;
pub const RUNNER_STANDING_HEIGHT: u16 = 2; // standing: 2 rows tall (rows 14-15)
pub const RUNNER_DUCKING_HEIGHT: u16 = 1;  // ducking: 1 row tall (row 15 only)

/// Flying obstacle row (head height for standing runner).
pub const FLYING_ROW: u16 = 13; // just above runner's head when standing

/// Run animation frame count (alternates between 2 frames).
pub const RUN_ANIM_FRAMES: u32 = 2;
```

### DinoRunInput

```rust
/// UI-agnostic input actions for Dino Run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DinoRunInput {
    Jump,    // Space or Up arrow
    Duck,    // Down arrow (hold)
    Release, // Down arrow released (stop ducking)
    Forfeit, // Esc
    Other,   // Any other key (cancels forfeit_pending)
}
```

### DinoRunGame

```rust
/// Main game state.
#[derive(Debug, Clone)]
pub struct DinoRunGame {
    pub difficulty: DinoRunDifficulty,
    pub game_result: Option<DinoRunResult>,
    pub forfeit_pending: bool,
    /// True until the player presses Space/Up to begin. Physics paused while waiting.
    pub waiting_to_start: bool,

    // ── Runner state ──
    /// Vertical position of runner's feet in rows (float for smooth physics).
    /// GROUND_ROW = on ground, lower values = higher in the air.
    pub runner_y: f64,
    /// Current vertical velocity in rows/tick (negative = upward).
    pub velocity: f64,
    /// Whether the runner is currently ducking.
    pub is_ducking: bool,
    /// Duck input queued for next physics tick.
    pub duck_queued: bool,
    /// Jump input queued for next physics tick.
    pub jump_queued: bool,
    /// Animation frame for running (0 or 1, alternates every N ticks).
    pub run_anim_frame: u32,

    // ── Obstacle state ──
    /// Active obstacles on screen.
    pub obstacles: Vec<Obstacle>,
    /// Distance until next obstacle spawns (in cols).
    pub next_obstacle_distance: f64,

    // ── Scoring ──
    /// Obstacles successfully passed.
    pub score: u32,
    /// Obstacles needed to win.
    pub target_score: u32,
    /// Current game speed in cols/tick (increases over time).
    pub game_speed: f64,
    /// Total distance traveled (cols), used for speed ramping.
    pub distance: f64,

    // ── Timing ──
    /// Sub-tick time accumulator (milliseconds).
    pub accumulated_time_ms: u64,
    /// Total physics ticks elapsed.
    pub tick_count: u64,

    // ── Cached difficulty parameters ──
    pub gravity: f64,
    pub jump_impulse: f64,
    pub terminal_velocity: f64,
    pub initial_speed: f64,
    pub max_speed: f64,
    pub speed_increase_rate: f64,
    pub obstacle_frequency_min: f64,
    pub obstacle_frequency_max: f64,
}
```

### DinoRunGame::new()

```rust
impl DinoRunGame {
    pub fn new(difficulty: DinoRunDifficulty) -> Self {
        Self {
            difficulty,
            game_result: None,
            forfeit_pending: false,
            waiting_to_start: true,

            runner_y: GROUND_ROW as f64,
            velocity: 0.0,
            is_ducking: false,
            duck_queued: false,
            jump_queued: false,
            run_anim_frame: 0,

            obstacles: Vec::new(),
            next_obstacle_distance: GAME_WIDTH as f64 + 10.0,

            score: 0,
            target_score: difficulty.target_score(),
            game_speed: difficulty.initial_speed(),
            distance: 0.0,

            accumulated_time_ms: 0,
            tick_count: 0,

            gravity: difficulty.gravity(),
            jump_impulse: difficulty.jump_impulse(),
            terminal_velocity: difficulty.terminal_velocity(),
            initial_speed: difficulty.initial_speed(),
            max_speed: difficulty.max_speed(),
            speed_increase_rate: difficulty.speed_increase_rate(),
            obstacle_frequency_min: difficulty.obstacle_frequency_min(),
            obstacle_frequency_max: difficulty.obstacle_frequency_max(),
        }
    }

    /// Returns true if the runner is on the ground.
    pub fn is_on_ground(&self) -> bool {
        self.runner_y >= GROUND_ROW as f64
    }

    /// Spawn a new obstacle with a random type.
    pub fn spawn_obstacle<R: Rng>(&mut self, rng: &mut R) {
        let obstacle_type = if self.score > 5 && rng.gen::<f64>() < 0.25 {
            // 25% chance of flying obstacle after score > 5
            if rng.gen::<bool>() {
                ObstacleType::Bat
            } else {
                ObstacleType::Stalactite
            }
        } else {
            // Ground obstacles, weighted by difficulty
            match rng.gen_range(0..4) {
                0 => ObstacleType::SmallRock,
                1 => ObstacleType::LargeRock,
                2 => ObstacleType::Cactus,
                _ => ObstacleType::DoubleCactus,
            }
        };

        let x = GAME_WIDTH as f64 + obstacle_type.width() as f64;
        self.obstacles.push(Obstacle {
            x,
            obstacle_type,
            passed: false,
        });

        // Randomize next obstacle distance within frequency range
        self.next_obstacle_distance =
            rng.gen_range(self.obstacle_frequency_min..=self.obstacle_frequency_max);
    }
}
```

### Difficulty Parameters

```rust
impl DinoRunDifficulty {
    /// Gravity (velocity change per 16ms tick, positive = downward).
    pub fn gravity(&self) -> f64 {
        match self {
            Self::Novice     => 0.012,
            Self::Apprentice => 0.014,
            Self::Journeyman => 0.016,
            Self::Master     => 0.018,
        }
    }

    /// Jump impulse (negative = upward, sets velocity directly).
    pub fn jump_impulse(&self) -> f64 {
        match self {
            Self::Novice     => -0.28,
            Self::Apprentice => -0.27,
            Self::Journeyman => -0.26,
            Self::Master     => -0.25,
        }
    }

    /// Terminal velocity (max downward speed per 16ms tick).
    pub fn terminal_velocity(&self) -> f64 {
        match self {
            Self::Novice     => 0.40,
            Self::Apprentice => 0.40,
            Self::Journeyman => 0.40,
            Self::Master     => 0.40,
        }
    }

    /// Initial scroll speed in cols/tick.
    pub fn initial_speed(&self) -> f64 {
        match self {
            Self::Novice     => 0.10,
            Self::Apprentice => 0.13,
            Self::Journeyman => 0.16,
            Self::Master     => 0.19,
        }
    }

    /// Maximum scroll speed in cols/tick.
    pub fn max_speed(&self) -> f64 {
        match self {
            Self::Novice     => 0.18,
            Self::Apprentice => 0.22,
            Self::Journeyman => 0.28,
            Self::Master     => 0.35,
        }
    }

    /// Speed increase per 100 distance traveled (cols/tick increment).
    pub fn speed_increase_rate(&self) -> f64 {
        match self {
            Self::Novice     => 0.0003,
            Self::Apprentice => 0.0004,
            Self::Journeyman => 0.0005,
            Self::Master     => 0.0006,
        }
    }

    /// Minimum distance between obstacles (cols).
    pub fn obstacle_frequency_min(&self) -> f64 {
        match self {
            Self::Novice     => 25.0,
            Self::Apprentice => 22.0,
            Self::Journeyman => 18.0,
            Self::Master     => 15.0,
        }
    }

    /// Maximum distance between obstacles (cols).
    pub fn obstacle_frequency_max(&self) -> f64 {
        match self {
            Self::Novice     => 40.0,
            Self::Apprentice => 35.0,
            Self::Journeyman => 30.0,
            Self::Master     => 25.0,
        }
    }

    /// Number of obstacles to pass to win.
    pub fn target_score(&self) -> u32 {
        match self {
            Self::Novice     => 15,
            Self::Apprentice => 25,
            Self::Journeyman => 40,
            Self::Master     => 60,
        }
    }
}
```

### Difficulty Parameter Summary

| Parameter | Novice | Apprentice | Journeyman | Master |
|-----------|--------|------------|------------|--------|
| Gravity | 0.012 | 0.014 | 0.016 | 0.018 |
| Jump Impulse | -0.28 | -0.27 | -0.26 | -0.25 |
| Terminal Velocity | 0.40 | 0.40 | 0.40 | 0.40 |
| Initial Speed | 0.10 | 0.13 | 0.16 | 0.19 |
| Max Speed | 0.18 | 0.22 | 0.28 | 0.35 |
| Speed Increase Rate | 0.0003 | 0.0004 | 0.0005 | 0.0006 |
| Obstacle Min Gap | 25 | 22 | 18 | 15 |
| Obstacle Max Gap | 40 | 35 | 30 | 25 |
| Target Score | 15 | 25 | 40 | 60 |

All physics values are calibrated for 16ms tick intervals, matching Flappy Bird's `PHYSICS_TICK_MS`.

---

## 3. Function Signatures (`logic.rs`)

### Physics Tick Interval

```rust
/// Physics tick interval in milliseconds (~60 FPS), same as Flappy Bird.
const PHYSICS_TICK_MS: u64 = 16;
```

### Public Functions

```rust
/// Process player input (called from input.rs).
pub fn process_input(game: &mut DinoRunGame, input: DinoRunInput) {
    // Game over: ignore all input (dismissal handled by input.rs)
    // Waiting to start: Space/Jump starts the game
    // Normal play:
    //   Jump: if on ground and not ducking, queue jump; if forfeit_pending, cancel
    //   Duck: queue duck; if forfeit_pending, cancel
    //   Release: clear duck_queued, stop ducking
    //   Forfeit: first Esc sets pending, second confirms (Loss)
    //   Other: cancel forfeit_pending
}

/// Advance Dino Run physics. Called from the main game loop.
/// `dt_ms` is milliseconds since last call. Internally steps physics in
/// 16ms increments (~60 FPS). Returns true if the game state changed.
pub fn tick_dino_run(game: &mut DinoRunGame, dt_ms: u64) -> bool {
    // Identical accumulator pattern to tick_flappy_bird:
    // - Return false if game_result is Some, waiting_to_start, or forfeit_pending
    // - Clamp dt_ms to 100 max
    // - Accumulate, step in PHYSICS_TICK_MS increments
    // - Each step calls step_physics(game)
}

/// Apply game result using the shared challenge reward system.
/// Returns `Some(MinigameWinInfo)` if the player won, `None` otherwise.
pub fn apply_game_result(state: &mut GameState) -> Option<MinigameWinInfo> {
    // Extract result from ActiveMinigame::DinoRun
    // Log score-specific messages
    // Call apply_challenge_rewards() with GameResultInfo
}
```

### Private: step_physics()

```rust
/// Single physics step (16ms tick).
fn step_physics(game: &mut DinoRunGame) {
    game.tick_count += 1;

    // 1. Consume buffered jump input (only if on ground and not ducking)
    if game.jump_queued && game.is_on_ground() && !game.is_ducking {
        game.velocity = game.jump_impulse;
        game.jump_queued = false;
    }

    // 2. Consume buffered duck input
    if game.duck_queued {
        game.is_ducking = true;
        game.duck_queued = false;
        // If in the air, fast-fall: apply extra gravity
        if !game.is_on_ground() {
            game.velocity += game.gravity * 2.0; // Fast-fall multiplier
        }
    }

    // 3. Apply gravity (only when airborne)
    if !game.is_on_ground() {
        game.velocity += game.gravity;
        if game.velocity > game.terminal_velocity {
            game.velocity = game.terminal_velocity;
        }
    }

    // 4. Update runner position
    game.runner_y += game.velocity;

    // 5. Clamp to ground
    if game.runner_y >= GROUND_ROW as f64 {
        game.runner_y = GROUND_ROW as f64;
        game.velocity = 0.0;
        // Stop ducking when landing if duck input is not held
        // (duck_queued is set each frame while held)
    }

    // 6. Move obstacles left
    for obstacle in &mut game.obstacles {
        obstacle.x -= game.game_speed;
    }

    // 7. Score: check if runner has passed obstacles
    let runner_right = (RUNNER_COL + RUNNER_WIDTH) as f64;
    for obstacle in &mut game.obstacles {
        if !obstacle.passed && (obstacle.x + obstacle.obstacle_type.width() as f64) < runner_right as f64 {
            obstacle.passed = true;
            game.score += 1;
        }
    }

    // 8. Spawn new obstacles
    game.next_obstacle_distance -= game.game_speed;
    if game.next_obstacle_distance <= 0.0 {
        let mut rng = rand::thread_rng();
        game.spawn_obstacle(&mut rng);
    }

    // 9. Remove off-screen obstacles
    game.obstacles.retain(|o| o.x > -10.0);

    // 10. Update game speed (gradual acceleration)
    game.distance += game.game_speed;
    game.game_speed = (game.initial_speed + game.distance * game.speed_increase_rate)
        .min(game.max_speed);

    // 11. Update run animation
    if game.is_on_ground() && game.tick_count % 8 == 0 {
        game.run_anim_frame = (game.run_anim_frame + 1) % RUN_ANIM_FRAMES;
    }

    // 12. Collision detection
    if check_collision(game) {
        game.game_result = Some(DinoRunResult::Loss);
        return;
    }

    // 13. Win condition
    if game.score >= game.target_score {
        game.game_result = Some(DinoRunResult::Win);
    }
}
```

### Private: check_collision()

```rust
/// Check collision between runner and all obstacles.
fn check_collision(game: &DinoRunGame) -> bool {
    let runner_left = RUNNER_COL as f64;
    let runner_right = (RUNNER_COL + RUNNER_WIDTH) as f64;

    let runner_height = if game.is_ducking {
        RUNNER_DUCKING_HEIGHT
    } else {
        RUNNER_STANDING_HEIGHT
    };

    // Runner's top row (remember: lower y = higher on screen)
    let runner_top = game.runner_y - (runner_height as f64 - 1.0);
    let runner_bottom = game.runner_y;

    for obstacle in &game.obstacles {
        let obs_left = obstacle.x;
        let obs_right = obstacle.x + obstacle.obstacle_type.width() as f64;

        // Horizontal overlap check
        if runner_right <= obs_left || runner_left >= obs_right {
            continue;
        }

        // Vertical position of obstacle
        let (obs_top, obs_bottom) = if obstacle.obstacle_type.is_flying() {
            // Flying obstacles at head height
            let top = FLYING_ROW as f64;
            let bottom = top + obstacle.obstacle_type.height() as f64 - 1.0;
            (top, bottom)
        } else {
            // Ground obstacles sit on the ground
            let bottom = GROUND_ROW as f64;
            let top = bottom - (obstacle.obstacle_type.height() as f64 - 1.0);
            (top, bottom)
        };

        // Vertical overlap check
        if runner_bottom >= obs_top && runner_top <= obs_bottom {
            return true;
        }
    }

    false
}
```

### DifficultyInfo Implementation

```rust
impl DifficultyInfo for DinoRunDifficulty {
    fn name(&self) -> &'static str {
        DinoRunDifficulty::name(self)
    }

    fn reward(&self) -> ChallengeReward {
        match self {
            DinoRunDifficulty::Novice => ChallengeReward {
                xp_percent: 50,
                ..Default::default()
            },
            DinoRunDifficulty::Apprentice => ChallengeReward {
                xp_percent: 100,
                ..Default::default()
            },
            DinoRunDifficulty::Journeyman => ChallengeReward {
                prestige_ranks: 1,
                xp_percent: 75,
                ..Default::default()
            },
            DinoRunDifficulty::Master => ChallengeReward {
                prestige_ranks: 2,
                xp_percent: 150,
                fishing_ranks: 1,
            },
        }
    }

    fn extra_info(&self) -> Option<String> {
        match self {
            DinoRunDifficulty::Novice => Some("15 obstacles, slow start".to_string()),
            DinoRunDifficulty::Apprentice => Some("25 obstacles, moderate pace".to_string()),
            DinoRunDifficulty::Journeyman => Some("40 obstacles, fast pace".to_string()),
            DinoRunDifficulty::Master => Some("60 obstacles, relentless".to_string()),
        }
    }
}
```

### apply_game_result()

```rust
pub fn apply_game_result(state: &mut GameState) -> Option<MinigameWinInfo> {
    let (result, difficulty, score, target) = {
        if let Some(ActiveMinigame::DinoRun(ref game)) = state.active_minigame {
            (game.game_result, game.difficulty, game.score, game.target_score)
        } else {
            return None;
        }
    };

    let result = result?;
    let won = matches!(result, DinoRunResult::Win);
    let reward = difficulty.reward();

    // Log score-specific message
    if won {
        state.combat_state.add_log_entry(
            format!("> You survived the Dungeon Sprint! ({}/{} obstacles)", score, target),
            false, true,
        );
    } else {
        state.combat_state.add_log_entry(
            format!("> Stumbled after {} obstacles.", score),
            false, true,
        );
    }

    crate::challenges::apply_challenge_rewards(
        state,
        GameResultInfo {
            won,
            game_type: "dino_run",
            difficulty_str: difficulty.difficulty_str(),
            reward,
            icon: "\u{1F3C3}",  // runner emoji (🏃)
            win_message: "Dungeon Sprint conquered!",
            loss_message: "The sprint claims another.",
        },
    )
}
```

### Module Exports (`mod.rs`)

```rust
pub mod logic;
pub mod types;

pub use types::{DinoRunDifficulty, DinoRunGame, DinoRunResult, Obstacle, ObstacleType};
```

---

## 4. Rendering (`src/ui/dino_scene.rs`)

### Scene Structure

```rust
use super::game_common::{
    create_game_layout, render_forfeit_status_bar, render_game_over_overlay,
    render_info_panel_frame, render_status_bar, GameResultType,
};
use crate::challenges::dino::types::*;
use crate::challenges::menu::DifficultyInfo;

pub fn render_dino_scene(frame: &mut Frame, area: Rect, game: &DinoRunGame) {
    // 1. Game over overlay
    if game.game_result.is_some() {
        render_dino_game_over(frame, area, game);
        return;
    }

    // 2. Create layout using shared game layout
    let layout = create_game_layout(frame, area, " Dungeon Sprint ", Color::LightYellow, 15, 18);

    // 3. Render play field (runner, obstacles, ground)
    render_play_field(frame, layout.content, game);

    // 4. Start prompt overlay
    if game.waiting_to_start {
        render_start_prompt(frame, layout.content);
    }

    // 5. Status bar
    render_status_bar_content(frame, layout.status_bar, game);

    // 6. Info panel
    render_info_panel(frame, layout.info_panel, game);
}
```

### Play Field ASCII Art

```
Runner (standing, frame 0):     Runner (standing, frame 1):
   O                               O
  /|>                             <|\

Runner (ducking):                Runner (jumping):
  _O>                              O
                                  /|>

Ground obstacles:
  Small rock: ..    Large rock: ##    Cactus: |    Double cactus: |+|
                               ##            +                   |+|

Flying obstacles:
  Bat: ^^    Stalactite: vvv

Ground line:
  ════════════════════════════════════════════════════════════
```

The renderer uses a cell buffer approach identical to `flappy_scene.rs`: build a 2D buffer, stamp objects, then emit as `Paragraph` widgets per row.

### Status Bar

```rust
fn render_status_bar_content(frame: &mut Frame, area: Rect, game: &DinoRunGame) {
    if game.waiting_to_start {
        render_status_bar(frame, area, "Ready", Color::LightYellow,
            &[("[Space/Up]", "Start"), ("[Esc]", "Forfeit")]);
        return;
    }
    if render_forfeit_status_bar(frame, area, game.forfeit_pending) {
        return;
    }
    render_status_bar(frame, area, "Run!", Color::Yellow,
        &[("[Space/Up]", "Jump"), ("[Down]", "Duck"), ("[Esc]", "Forfeit")]);
}
```

### Info Panel

Displays: Difficulty, Score (current/target), Speed (percentage of max), progress bar.

### Game Over

Uses `render_game_over_overlay()` with:
- Win: `GameResultType::Win`, title ":: DUNGEON SPRINT CONQUERED! ::"
- Loss: `GameResultType::Loss`, title "SPRINT FAILED"

### Border Color

`Color::LightYellow` (unique among minigames: Flappy=LightCyan, Chess=Cyan, Go=Green, etc.)

---

## 5. Rewards Structure

Following the existing challenge reward pattern (matches Flappy Bird exactly):

| Difficulty | Prestige | XP% | Fishing | Target Score |
|-----------|----------|-----|---------|-------------|
| Novice | 0 | 50 | 0 | 15 obstacles |
| Apprentice | 0 | 100 | 0 | 25 obstacles |
| Journeyman | 1 | 75 | 0 | 40 obstacles |
| Master | 2 | 150 | 1 | 60 obstacles |

---

## 6. Achievement IDs

Add to `AchievementId` enum in `src/achievements/types.rs`:

```rust
// Challenge achievements - Dino Run
DinoRunNovice,
DinoRunApprentice,
DinoRunJourneyman,
DinoRunMaster,
```

Add to `on_minigame_won()` match in `src/achievements/types.rs`:

```rust
("dino_run", "novice") => Some(AchievementId::DinoRunNovice),
("dino_run", "apprentice") => Some(AchievementId::DinoRunApprentice),
("dino_run", "journeyman") => Some(AchievementId::DinoRunJourneyman),
("dino_run", "master") => Some(AchievementId::DinoRunMaster),
```

Add to `ALL_ACHIEVEMENTS` in `src/achievements/data.rs`:

```rust
// CHALLENGE ACHIEVEMENTS - DINO RUN
AchievementDef {
    id: AchievementId::DinoRunNovice,
    name: "Sprint Novice",
    description: "Win Dungeon Sprint on Novice difficulty",
    category: AchievementCategory::Challenges,
    icon: "\u{1F3C3}",
},
AchievementDef {
    id: AchievementId::DinoRunApprentice,
    name: "Sprint Apprentice",
    description: "Win Dungeon Sprint on Apprentice difficulty",
    category: AchievementCategory::Challenges,
    icon: "\u{1F3C3}",
},
AchievementDef {
    id: AchievementId::DinoRunJourneyman,
    name: "Sprint Journeyman",
    description: "Win Dungeon Sprint on Journeyman difficulty",
    category: AchievementCategory::Challenges,
    icon: "\u{1F3C3}",
},
AchievementDef {
    id: AchievementId::DinoRunMaster,
    name: "Sprint Master",
    description: "Win Dungeon Sprint on Master difficulty",
    category: AchievementCategory::Challenges,
    icon: "\u{1F3C3}",
},
```

---

## 7. Integration Checklist

Every file that needs modification, with specific changes:

### `src/challenges/dino/mod.rs` (NEW)
- Create module with `pub mod logic; pub mod types;`
- Re-export: `pub use types::{DinoRunDifficulty, DinoRunGame, DinoRunResult, Obstacle, ObstacleType};`

### `src/challenges/dino/types.rs` (NEW)
- `DinoRunDifficulty` enum with `difficulty_enum_impl!` macro
- `DinoRunResult` enum (Win, Loss)
- `ObstacleType` enum with `width()`, `height()`, `is_flying()` methods
- `Obstacle` struct (x, obstacle_type, passed)
- `DinoRunGame` struct with all fields
- `DinoRunGame::new()`, `is_on_ground()`, `spawn_obstacle<R: Rng>()`
- Game constants: `GAME_WIDTH`, `GAME_HEIGHT`, `GROUND_ROW`, `RUNNER_COL`, etc.
- Difficulty parameter accessors on `DinoRunDifficulty`

### `src/challenges/dino/logic.rs` (NEW)
- `DinoRunInput` enum (Jump, Duck, Release, Forfeit, Other)
- `process_input()` function
- `tick_dino_run()` function (accumulator + fixed timestep physics)
- `step_physics()` private function (gravity, collision, scoring, speed ramp)
- `check_collision()` private function (AABB intersection)
- `DifficultyInfo` impl for `DinoRunDifficulty`
- `apply_game_result()` function using shared `apply_challenge_rewards()`
- Unit tests following Flappy Bird test patterns

### `src/challenges/mod.rs` (MODIFY)
- Add `pub mod dino;` to module declarations (after `pub mod chess;`)
- Add `pub use dino::{DinoRunDifficulty, DinoRunGame, DinoRunResult};` to re-exports
- Add `DinoRun(DinoRunGame)` variant to `ActiveMinigame` enum

### `src/challenges/menu.rs` (MODIFY)
- Add `use super::dino::{DinoRunDifficulty, DinoRunGame};` import
- Add `DinoRun` variant to `ChallengeType` enum
- Add `DinoRun` entry to `CHALLENGE_TABLE` (weight: 20, moderate — same as FlappyBird)
- Add `DinoRun` arm to `ChallengeType::icon()` — return `"\u{1F3C3}"` (runner emoji)
- Add `DinoRun` arm to `ChallengeType::discovery_flavor()` — "A hidden passage rumbles open, revealing an endless corridor..."
- Add `DinoRun` arm to `create_challenge()` — title: "Dungeon Sprint", description: flavor text about an endless corridor
- Add `DinoRun` arm to `accept_selected_challenge()` — creates `ActiveMinigame::DinoRun(DinoRunGame::new(d))`

### `src/core/tick.rs` (NO CHANGES NEEDED)
- DinoRun falls through the existing `_ => {}` arm in challenge AI thinking
- All other systems continue at 100ms as normal
- DinoRun physics are called from main.rs, not from game_tick()

### `src/core/constants.rs` (NO CHANGES NEEDED)
- `REALTIME_FRAME_MS` already exists (16ms, set by Flappy Bird)

### `src/main.rs` (MODIFY)
- Update `is_realtime_minigame()` to add `Some(ActiveMinigame::DinoRun(_))` arm:
  ```rust
  fn is_realtime_minigame(state: &GameState) -> bool {
      matches!(
          state.active_minigame,
          Some(challenges::ActiveMinigame::FlappyBird(_))
          | Some(challenges::ActiveMinigame::DinoRun(_))
      )
  }
  ```
- Add Dino Run tick call alongside Flappy Bird in the realtime tick block:
  ```rust
  if let Some(challenges::ActiveMinigame::DinoRun(ref mut game)) = state.active_minigame {
      challenges::dino::logic::tick_dino_run(game, dt.as_millis() as u64);
  }
  ```

### `src/input.rs` (MODIFY)
- Add imports:
  ```rust
  use crate::challenges::dino::logic::{
      apply_game_result as apply_dino_result,
      process_input as process_dino_input,
      DinoRunInput,
  };
  ```
- Add `ActiveMinigame::DinoRun(game)` arm to `handle_minigame()`:
  ```rust
  ActiveMinigame::DinoRun(game) => {
      if game.game_result.is_some() {
          state.last_minigame_win = apply_dino_result(state);
          return InputResult::Continue;
      }
      let input = match key.code {
          KeyCode::Char(' ') | KeyCode::Up => DinoRunInput::Jump,
          KeyCode::Down => DinoRunInput::Duck,
          KeyCode::Esc => DinoRunInput::Forfeit,
          _ => DinoRunInput::Other,
      };
      process_dino_input(game, input);
  }
  ```
- **Note on Down key release**: Crossterm `KeyEventKind::Release` events are needed for `DinoRunInput::Release`. The main loop already filters to `KeyEventKind::Press`. Two options:
  1. **Simple**: Treat duck as a toggle (press Down = start duck, press any other key = stop). This is simpler and works well in terminal.
  2. **With release detection**: Add `KeyEventKind::Release` handling for Down key only. This requires modifying the key event filter in main.rs.

  **Recommended: Option 1 (toggle duck)**. The input handler should set `duck_queued = true` on Down press. The physics step consumes it and sets `is_ducking = true`. When the runner lands or jumps, `is_ducking` is cleared. This matches Chrome dino behavior where duck is meaningful only while on the ground.

### `src/ui/mod.rs` (MODIFY)
- Add `pub mod dino_scene;` to module declarations
- Add `Some(ActiveMinigame::DinoRun(game))` arm to `draw_right_content()`:
  ```rust
  Some(ActiveMinigame::DinoRun(game)) => {
      dino_scene::render_dino_scene(frame, area, game);
  }
  ```

### `src/ui/dino_scene.rs` (NEW)
- `render_dino_scene()` using `create_game_layout()`
- `render_play_field()` for runner + obstacles + ground ASCII rendering
- `render_status_bar_content()` with forfeit handling
- `render_info_panel()` with score/target/speed display
- `render_start_prompt()` for "Press Space to Start"
- `render_dino_game_over()` using `render_game_over_overlay()`

### `src/lib.rs` (MODIFY)
- Add to the re-export block:
  ```rust
  pub use challenges::{
      // ... existing ...
      DinoRunDifficulty, DinoRunGame, DinoRunResult,
  };
  ```

### `src/utils/debug_menu.rs` (MODIFY)
- Add `"Trigger Dino Run Challenge"` to `DEBUG_OPTIONS` array (before "Trigger Haven Discovery")
- Add `trigger_dino_challenge()` function following existing pattern:
  ```rust
  fn trigger_dino_challenge(state: &mut GameState) -> &'static str {
      if state.challenge_menu.has_challenge(&ChallengeType::DinoRun) {
          return "Dino Run challenge already pending!";
      }
      state.challenge_menu.add_challenge(create_challenge(&ChallengeType::DinoRun));
      "Dino Run challenge added!"
  }
  ```
- Update `trigger_selected()` match indices (shift Haven discovery from index 9 to 10)

### `src/achievements/types.rs` (MODIFY)
- Add 4 variants to `AchievementId` enum: `DinoRunNovice`, `DinoRunApprentice`, `DinoRunJourneyman`, `DinoRunMaster`
- Add 4 arms to `on_minigame_won()` match for `("dino_run", ...)` mapping

### `src/achievements/data.rs` (MODIFY)
- Add 4 `AchievementDef` entries to `ALL_ACHIEVEMENTS` (after FlappyBird achievements, before GrandChampion)

---

## 8. Game Loop Integration (reuses Flappy Bird infrastructure)

### What already exists (from Flappy Bird implementation):

1. **`is_realtime_minigame()`** in main.rs -- just needs a new arm
2. **`REALTIME_FRAME_MS`** constant in constants.rs (16ms)
3. **`last_flappy_frame`** timer in main.rs -- rename to `last_realtime_frame` for clarity, or reuse as-is
4. **Event draining loop** with `Duration::ZERO` in realtime mode
5. **Dual-tick architecture**: 100ms game_tick + 16ms realtime tick

### Changes to main.rs realtime block:

The existing realtime tick block handles Flappy Bird:

```rust
if realtime_mode {
    let dt = last_flappy_frame.elapsed();
    if dt >= Duration::from_millis(REALTIME_FRAME_MS) {
        if let Some(challenges::ActiveMinigame::FlappyBird(ref mut game)) = state.active_minigame {
            challenges::flappy::logic::tick_flappy_bird(game, dt.as_millis() as u64);
        }
        last_flappy_frame = Instant::now();
    }
}
```

This becomes:

```rust
if realtime_mode {
    let dt = last_flappy_frame.elapsed();
    if dt >= Duration::from_millis(REALTIME_FRAME_MS) {
        match state.active_minigame {
            Some(challenges::ActiveMinigame::FlappyBird(ref mut game)) => {
                challenges::flappy::logic::tick_flappy_bird(game, dt.as_millis() as u64);
            }
            Some(challenges::ActiveMinigame::DinoRun(ref mut game)) => {
                challenges::dino::logic::tick_dino_run(game, dt.as_millis() as u64);
            }
            _ => {}
        }
        last_flappy_frame = Instant::now();
    }
}
```

---

## 9. Key Design Decisions

### Jump vs Duck Mechanics

Unlike Flappy Bird (single input: flap), Dino Run has two inputs:
- **Jump** (Space/Up): Only works when on the ground and not ducking. Sets velocity to jump_impulse.
- **Duck** (Down): Only meaningful on the ground. Reduces hitbox from 2 rows to 1 row. While ducking, flying obstacles pass overhead. Pressing Down while airborne triggers fast-fall (extra gravity).

### Speed Ramping

The game starts slow and progressively speeds up. `game_speed` increases linearly with `distance` traveled, capped at `max_speed`. This creates natural difficulty progression within a single run, making early game accessible while late game tests reflexes.

### Obstacle Variety

Six obstacle types provide variety:
- Ground obstacles require jumping (varying heights/widths test jump timing)
- Flying obstacles require ducking (test duck timing)
- After score > 5, flying obstacles begin appearing (25% chance), creating jump-or-duck decisions

### Physics Model

Fixed 16ms timestep with accumulator (identical to Flappy Bird). Gravity pulls the runner down, jump impulse sets an upward velocity. Terminal velocity prevents excessive falling speed. All values are pre-calibrated per-tick (not per-second) for deterministic behavior.

### No AI Thinking

Like Flappy Bird, Dino Run has no AI component. It falls through the `_ => {}` arm in game_tick()'s challenge AI thinking match. Physics run entirely from main.rs at 60 FPS.

---

## 10. Performance Requirements

Same as Flappy Bird: 33ms frame budget (30 FPS rendering), with physics at 60 FPS internally.

- **Render**: ~5-15ms (cell buffer approach, same as Flappy Bird)
- **Input**: ~0-1ms (event drain)
- **Physics**: ~0-1ms (simple arithmetic + AABB collision)
- **Obstacles**: Pre-allocated Vec, ~10 elements max, retained with `retain()`
- **Zero allocation in hot path**: All positions computed on stack

---

## 11. Summary: What This Architecture Reuses

| Component | Source | Changes Needed |
|-----------|--------|---------------|
| Real-time game loop | Flappy Bird (main.rs) | Add DinoRun arm to match |
| Adaptive polling | Flappy Bird (main.rs) | Add DinoRun to is_realtime_minigame() |
| REALTIME_FRAME_MS | constants.rs | None (already 16ms) |
| Physics accumulator | Flappy Bird (logic.rs) | Copy pattern, new physics |
| difficulty_enum_impl! | challenges/mod.rs | Apply to DinoRunDifficulty |
| DifficultyInfo trait | challenges/menu.rs | Implement for DinoRunDifficulty |
| apply_challenge_rewards() | challenges/mod.rs | Call with DinoRun GameResultInfo |
| Forfeit pattern | All minigames | Same Esc/Esc/cancel flow |
| create_game_layout() | ui/game_common.rs | Use for scene layout |
| render_game_over_overlay() | ui/game_common.rs | Use for win/loss screen |
| render_forfeit_status_bar() | ui/game_common.rs | Use in status bar |
| Cell buffer rendering | ui/flappy_scene.rs | Same approach for play field |
| Achievement tracking | achievements/types.rs | Add 4 new IDs + match arms |
| Debug menu trigger | utils/debug_menu.rs | Add new entry + function |
