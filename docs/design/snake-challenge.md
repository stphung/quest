# Snake Challenge Architecture

A real-time grid-based Snake minigame for the Quest challenge system.
The player controls a growing snake that must eat food items to reach a target score
without colliding with walls or its own body.

## File Structure

```
src/challenges/snake/
├── mod.rs      # Public exports: SnakeDifficulty, SnakeGame, SnakeResult
├── types.rs    # Data structures, difficulty parameters, grid constants
└── logic.rs    # Physics tick, input processing, collision detection, reward application

src/ui/
└── snake_scene.rs  # Rendering: grid, snake body, food, walls, info panel
```

### Responsibilities

| File | Responsibility |
|------|---------------|
| `types.rs` | `SnakeDifficulty` enum, `SnakeGame` struct, `SnakeResult` enum, `Direction` enum, grid constants, difficulty parameter methods, food spawning |
| `logic.rs` | `SnakeInput` enum, `process_input()`, `tick_snake()` (accumulator-based physics), `step_physics()` (single movement step), collision detection, `apply_game_result()`, `DifficultyInfo` impl |
| `mod.rs` | Re-exports: `pub use types::{SnakeDifficulty, SnakeGame, SnakeResult}` |
| `snake_scene.rs` | `render_snake_scene()`, play field rendering with cell buffer, status bar, info panel, game-over overlay, start prompt |

## Data Types

### `SnakeDifficulty` (types.rs)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SnakeDifficulty {
    Novice,
    Apprentice,
    Journeyman,
    Master,
}

difficulty_enum_impl!(SnakeDifficulty);
```

Methods on `SnakeDifficulty`:

| Method | Novice | Apprentice | Journeyman | Master | Description |
|--------|--------|------------|------------|--------|-------------|
| `grid_width()` | 20 | 18 | 15 | 12 | Grid columns |
| `grid_height()` | 15 | 13 | 11 | 9 | Grid rows |
| `move_interval_ms()` | 150 | 120 | 90 | 70 | Milliseconds between snake moves |
| `target_score()` | 10 | 15 | 20 | 30 | Food items to eat to win |
| `initial_length()` | 3 | 3 | 3 | 3 | Starting snake body length |

### `Direction` (types.rs)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// Returns true if self is the opposite of other.
    /// Used to prevent the snake from reversing into itself.
    pub fn is_opposite(&self, other: &Direction) -> bool {
        matches!(
            (self, other),
            (Direction::Up, Direction::Down)
                | (Direction::Down, Direction::Up)
                | (Direction::Left, Direction::Right)
                | (Direction::Right, Direction::Left)
        )
    }
}
```

### `SnakeResult` (types.rs)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnakeResult {
    Win,
    Loss,
}
```

No `Draw` variant -- snake either reaches the target score or dies.

### `SnakeGame` (types.rs)

```rust
#[derive(Debug, Clone)]
pub struct SnakeGame {
    pub difficulty: SnakeDifficulty,
    pub game_result: Option<SnakeResult>,
    pub forfeit_pending: bool,
    /// True until the player presses Space to begin. Physics paused while waiting.
    pub waiting_to_start: bool,

    // Grid dimensions (cached from difficulty)
    pub grid_width: u16,
    pub grid_height: u16,

    // Snake state
    /// Snake body segments, head at front (index 0). Uses VecDeque for O(1)
    /// push_front (grow head) and pop_back (remove tail).
    pub body: VecDeque<(u16, u16)>,
    /// Current movement direction.
    pub direction: Direction,
    /// Buffered direction change from input (applied on next physics step).
    /// Prevents multiple direction changes between physics ticks.
    pub pending_direction: Option<Direction>,

    // Food
    /// Current food position (col, row).
    pub food: (u16, u16),

    // Scoring
    /// Food items eaten so far.
    pub score: u32,
    /// Food items needed to win.
    pub target_score: u32,

    // Timing (accumulator pattern, same as Flappy Bird)
    /// Sub-tick time accumulator (milliseconds).
    pub accumulated_time_ms: u64,
    /// Move interval in milliseconds (cached from difficulty).
    pub move_interval_ms: u64,
}
```

### Constructor: `SnakeGame::new(difficulty)`

```rust
impl SnakeGame {
    pub fn new(difficulty: SnakeDifficulty) -> Self {
        let grid_width = difficulty.grid_width();
        let grid_height = difficulty.grid_height();
        let initial_length = difficulty.initial_length();

        // Snake starts horizontally centered, vertically centered, facing Right
        let start_y = grid_height / 2;
        let start_x = grid_width / 2;

        let mut body = VecDeque::new();
        for i in 0..initial_length {
            body.push_back((start_x - i, start_y));
        }

        // Generate initial food position avoiding the snake body
        let mut rng = rand::thread_rng();
        let food = Self::random_food_position(grid_width, grid_height, &body, &mut rng);

        Self {
            difficulty,
            game_result: None,
            forfeit_pending: false,
            waiting_to_start: true,
            grid_width,
            grid_height,
            body,
            direction: Direction::Right,
            pending_direction: None,
            food,
            score: 0,
            target_score: difficulty.target_score(),
            accumulated_time_ms: 0,
            move_interval_ms: difficulty.move_interval_ms() as u64,
        }
    }

    /// Generate a random food position on an empty cell.
    /// Collects all empty cells and picks one uniformly at random.
    pub fn random_food_position<R: Rng>(
        width: u16,
        height: u16,
        body: &VecDeque<(u16, u16)>,
        rng: &mut R,
    ) -> (u16, u16) {
        let empty_cells: Vec<(u16, u16)> = (0..width)
            .flat_map(|x| (0..height).map(move |y| (x, y)))
            .filter(|pos| !body.contains(pos))
            .collect();

        if empty_cells.is_empty() {
            // Grid full -- should not happen since win triggers first
            (0, 0)
        } else {
            empty_cells[rng.gen_range(0..empty_cells.len())]
        }
    }
}
```

## Physics / Tick Model

Snake uses the same accumulator-based timing as Flappy Bird. The main game loop
calls `tick_snake(game, dt_ms)` every `REALTIME_FRAME_MS` (16ms). Internally,
Snake accumulates time and steps physics at its own interval (`move_interval_ms`).

### `tick_snake(game: &mut SnakeGame, dt_ms: u64) -> bool` (logic.rs)

```rust
/// Physics tick interval for Snake.
/// Unlike Flappy Bird (16ms continuous physics), Snake moves discretely
/// at the difficulty's move_interval_ms.
const MAX_DT_MS: u64 = 200; // Clamp dt to prevent physics explosion after pause

pub fn tick_snake(game: &mut SnakeGame, dt_ms: u64) -> bool {
    if game.game_result.is_some() {
        return false;
    }

    // Pause physics while waiting to start or during forfeit
    if game.waiting_to_start || game.forfeit_pending {
        return false;
    }

    let dt_ms = dt_ms.min(MAX_DT_MS);
    game.accumulated_time_ms += dt_ms;
    let mut changed = false;

    while game.accumulated_time_ms >= game.move_interval_ms {
        game.accumulated_time_ms -= game.move_interval_ms;
        step_physics(game);
        changed = true;

        if game.game_result.is_some() {
            break;
        }
    }

    changed
}
```

### `step_physics(game: &mut SnakeGame)` (logic.rs)

A single movement step:

```rust
fn step_physics(game: &mut SnakeGame) {
    // 1. Apply pending direction change (if any)
    if let Some(new_dir) = game.pending_direction.take() {
        if !new_dir.is_opposite(&game.direction) {
            game.direction = new_dir;
        }
    }

    // 2. Calculate new head position
    let (hx, hy) = game.body[0]; // Current head
    let (nx, ny) = match game.direction {
        Direction::Up    => (hx as i32,     hy as i32 - 1),
        Direction::Down  => (hx as i32,     hy as i32 + 1),
        Direction::Left  => (hx as i32 - 1, hy as i32),
        Direction::Right => (hx as i32 + 1, hy as i32),
    };

    // 3. Wall collision (out of bounds)
    if nx < 0 || nx >= game.grid_width as i32 || ny < 0 || ny >= game.grid_height as i32 {
        game.game_result = Some(SnakeResult::Loss);
        return;
    }

    let new_head = (nx as u16, ny as u16);

    // 4. Self-collision (check before moving — tail hasn't been removed yet,
    //    but if we're about to eat food the tail stays, so check full body)
    //    Exception: the last tail segment will be removed if NOT eating food,
    //    so only check against body[0..len-1] when not eating food.
    let eating = new_head == game.food;
    let collision_range = if eating {
        game.body.len() // Full body stays (tail won't be removed)
    } else {
        game.body.len() - 1 // Tail will be removed, so don't count it
    };
    if game.body.iter().take(collision_range).any(|&seg| seg == new_head) {
        game.game_result = Some(SnakeResult::Loss);
        return;
    }

    // 5. Move: push new head
    game.body.push_front(new_head);

    // 6. Food check
    if eating {
        game.score += 1;

        // Win condition
        if game.score >= game.target_score {
            game.game_result = Some(SnakeResult::Win);
            return;
        }

        // Spawn new food
        let mut rng = rand::thread_rng();
        game.food = SnakeGame::random_food_position(
            game.grid_width, game.grid_height, &game.body, &mut rng
        );
    } else {
        // Remove tail (snake doesn't grow)
        game.body.pop_back();
    }
}
```

## Input Handling

### `SnakeInput` (logic.rs)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnakeInput {
    Up,
    Down,
    Left,
    Right,
    Start,   // Space — starts the game from waiting state
    Forfeit, // Esc
    Other,   // Any other key (cancels forfeit_pending)
}
```

### `process_input(game: &mut SnakeGame, input: SnakeInput)` (logic.rs)

```rust
pub fn process_input(game: &mut SnakeGame, input: SnakeInput) {
    if game.game_result.is_some() {
        return;
    }

    // Waiting screen: Space starts the game
    if game.waiting_to_start {
        if matches!(input, SnakeInput::Start) {
            game.waiting_to_start = false;
        }
        return;
    }

    match input {
        SnakeInput::Up => try_set_direction(game, Direction::Up),
        SnakeInput::Down => try_set_direction(game, Direction::Down),
        SnakeInput::Left => try_set_direction(game, Direction::Left),
        SnakeInput::Right => try_set_direction(game, Direction::Right),
        SnakeInput::Forfeit => {
            if game.forfeit_pending {
                game.game_result = Some(SnakeResult::Loss);
            } else {
                game.forfeit_pending = true;
            }
        }
        SnakeInput::Start | SnakeInput::Other => {
            if game.forfeit_pending {
                game.forfeit_pending = false;
            }
        }
    }
}

/// Buffer a direction change. Rejects opposite directions (no reversal).
/// Direction changes cancel forfeit_pending.
fn try_set_direction(game: &mut SnakeGame, dir: Direction) {
    if game.forfeit_pending {
        game.forfeit_pending = false;
        return; // Cancel forfeit only, don't change direction
    }
    // Check against current direction to prevent reversal.
    // Use the pending direction if set (in case of multiple inputs between ticks).
    let effective_dir = game.pending_direction.unwrap_or(game.direction);
    if !dir.is_opposite(&effective_dir) {
        game.pending_direction = Some(dir);
    }
}
```

**Key design decisions:**

1. **`pending_direction` buffer**: Only one direction change is buffered between physics steps. This prevents the player from inputting two quick turns (e.g., Up then Left) that would effectively reverse the snake within one frame.

2. **Reversal prevention**: Direction changes are checked against `pending_direction` if set, otherwise against `current direction`. This ensures that even rapid inputs cannot cause a reversal.

3. **Forfeit cancellation**: Direction arrow keys cancel `forfeit_pending` without actually changing direction. This follows the established forfeit pattern (any non-Esc key cancels).

## Collision Detection

Two types of collision, checked in `step_physics()`:

### Wall Collision
The grid has no wrapping. If the new head position is outside `[0, grid_width)` x `[0, grid_height)`, the snake dies.

```
nx < 0 || nx >= grid_width || ny < 0 || ny >= grid_height
```

### Self-Collision
The new head position is checked against existing body segments. The check range depends on whether the snake is eating food:

- **Eating food**: Check against all body segments (tail stays, so the full body is a collision surface).
- **Not eating food**: Check against `body[0..len-1]` (the tail segment will be removed after moving, so it's safe to move into that position).

This distinction is important for a common Snake scenario: the snake's head moving into the position its tail is vacating.

## Food Spawning

`SnakeGame::random_food_position()` collects all grid cells not occupied by the snake body and selects one uniformly at random. This is O(grid_size) but the grids are small (max 20x15 = 300 cells), so performance is not a concern.

Food is spawned:
1. During `SnakeGame::new()` for the initial position
2. In `step_physics()` after the snake eats food and the game hasn't been won yet

## Win / Loss Conditions

| Condition | Result |
|-----------|--------|
| Snake eats `target_score` food items | `SnakeResult::Win` |
| Snake head moves out of bounds (wall) | `SnakeResult::Loss` |
| Snake head moves into its own body | `SnakeResult::Loss` |
| Player confirms forfeit (Esc twice) | `SnakeResult::Loss` |

## Rewards

Following the established pattern for real-time action games (similar to Flappy Bird):

| Difficulty | Reward |
|------------|--------|
| Novice | +50% level XP |
| Apprentice | +100% level XP |
| Journeyman | +1 Prestige Rank, +75% level XP |
| Master | +2 Prestige Ranks, +150% level XP, +1 Fish Rank |

### `DifficultyInfo` impl (logic.rs)

```rust
impl DifficultyInfo for SnakeDifficulty {
    fn name(&self) -> &'static str {
        SnakeDifficulty::name(self)
    }

    fn reward(&self) -> ChallengeReward {
        match self {
            SnakeDifficulty::Novice => ChallengeReward {
                xp_percent: 50,
                ..Default::default()
            },
            SnakeDifficulty::Apprentice => ChallengeReward {
                xp_percent: 100,
                ..Default::default()
            },
            SnakeDifficulty::Journeyman => ChallengeReward {
                prestige_ranks: 1,
                xp_percent: 75,
                ..Default::default()
            },
            SnakeDifficulty::Master => ChallengeReward {
                prestige_ranks: 2,
                xp_percent: 150,
                fishing_ranks: 1,
            },
        }
    }

    fn extra_info(&self) -> Option<String> {
        Some(format!(
            "{}x{}, {} food, {}ms",
            self.grid_width(),
            self.grid_height(),
            self.target_score(),
            self.move_interval_ms()
        ))
    }
}
```

### `apply_game_result()` (logic.rs)

Uses the shared `apply_challenge_rewards()` helper from `challenges/mod.rs`:

```rust
pub fn apply_game_result(state: &mut GameState) -> Option<MinigameWinInfo> {
    let (result, difficulty, score, target) = {
        if let Some(ActiveMinigame::Snake(ref game)) = state.active_minigame {
            (game.game_result, game.difficulty, game.score, game.target_score)
        } else {
            return None;
        }
    };

    let result = result?;
    let won = matches!(result, SnakeResult::Win);
    let reward = difficulty.reward();

    if won {
        state.combat_state.add_log_entry(
            format!("~ You conquered the Serpent's Path! ({}/{} food)", score, target),
            false, true,
        );
    } else {
        state.combat_state.add_log_entry(
            format!("~ The serpent falls after {} food.", score),
            false, true,
        );
    }

    crate::challenges::apply_challenge_rewards(
        state,
        GameResultInfo {
            won,
            game_type: "snake",
            difficulty_str: difficulty.difficulty_str(),
            reward,
            icon: "~",
            win_message: "Serpent's Path conquered!",
            loss_message: "The serpent falls.",
        },
    )
}
```

## UI Layout and Visual Design

### Scene Structure (`snake_scene.rs`)

Uses the shared `create_game_layout()` from `game_common.rs`:

```
+-  Serpent's Path  ---------------+-- Info -------+
|                                  |               |
|   [grid: walls + snake + food]   | Difficulty:   |
|                                  | Score: 5/10   |
|                                  |               |
|                                  | Grid: 20x15   |
|                                  | Speed: 150ms  |
|                                  |               |
|                                  | Legend:        |
|                                  |  @ Head       |
|                                  |  # Body       |
|                                  |  * Food       |
|                                  |  . Empty      |
| [status bar - 2 lines]          |               |
+----------------------------------+---------------+
```

Layout parameters:
- Title: `" Serpent's Path "`
- Border color: `Color::LightGreen`
- Content min height: 15
- Info panel width: 18

### Visual Characters

| Element | Character | Color |
|---------|-----------|-------|
| Snake head (up) | `^` | `Color::LightGreen` |
| Snake head (down) | `v` | `Color::LightGreen` |
| Snake head (left) | `<` | `Color::LightGreen` |
| Snake head (right) | `>` | `Color::LightGreen` |
| Snake body | `#` | `Color::Green` |
| Snake tail (last segment) | `~` | `Color::Rgb(40, 120, 40)` (darker green) |
| Food | `*` | `Color::LightRed` |
| Empty cell | ` ` (space) | N/A |
| Grid border (top/bottom) | `-` | `Color::DarkGray` |
| Grid border (sides) | `|` | `Color::DarkGray` |
| Grid border (corners) | `+` | `Color::DarkGray` |

### Rendering Approach

Use a cell buffer approach (same as Flappy Bird). Build a 2D grid of characters and colors,
then render row by row. The grid is drawn within the `content` area from `create_game_layout()`.

Grid rendering is scaled to fit the available terminal space using `x_scale` and `y_scale`
factors similar to Flappy Bird, but since Snake is grid-based rather than continuous-physics-based,
the grid cells should be rendered at their natural 1:1 character-to-cell ratio when space permits.
If the terminal is too small, the grid can be clipped.

The grid border is drawn around the playable area. The snake and food are drawn within the border.

### Score Display

Score displayed in the info panel (right side) and optionally as a small overlay in the top-right
of the grid area (following the Flappy Bird pattern).

### Status Bar

| State | Line 1 | Line 2 |
|-------|--------|--------|
| Waiting | "Ready" (LightGreen) | `[Space] Start  [Esc] Forfeit` |
| Playing | "Slither!" (Green) | `[Arrows] Move  [Esc] Forfeit` |
| Forfeit pending | "Forfeit game?" (Red) | `[Esc] Confirm  [Any] Cancel` |

### Game Over Overlay

Uses the shared `render_game_over_overlay()`:

- **Win**: `GameResultType::Win`, title ":: SERPENT'S PATH CONQUERED! ::", message with score
- **Loss**: `GameResultType::Loss`, title "SERPENT FALLS", message with score (or forfeit message if `forfeit_pending`)

### Start Prompt

Centered "[ Press Space to Start ]" overlay on the play field (same pattern as Flappy Bird).

## Integration Checklist

Every file that needs changes to add the Snake challenge:

### New Files (3)

1. **`src/challenges/snake/mod.rs`** -- Module exports
2. **`src/challenges/snake/types.rs`** -- Data structures and difficulty parameters
3. **`src/challenges/snake/logic.rs`** -- Game logic, input, rewards, DifficultyInfo
4. **`src/ui/snake_scene.rs`** -- UI rendering

### Modified Files (8)

5. **`src/challenges/mod.rs`**
   - Add `pub mod snake;`
   - Add `pub use snake::{SnakeDifficulty, SnakeGame, SnakeResult};`
   - Add `Snake(SnakeGame)` variant to `ActiveMinigame` enum

6. **`src/challenges/menu.rs`**
   - Add `use super::snake::{SnakeDifficulty, SnakeGame};`
   - Add `Snake` variant to `ChallengeType` enum
   - Add `ChallengeType::Snake` icon, discovery_flavor to `ChallengeType` impl
   - Add `DifficultyInfo` import for `SnakeDifficulty` (already in logic.rs)
   - Add `Snake` case to `accept_selected_challenge()`
   - Add `ChallengeWeight` entry to `CHALLENGE_TABLE` (weight: 20, ~15%)
   - Add `ChallengeType::Snake` case to `create_challenge()`
   - Add `SnakeDifficulty::ALL[i].difficulty_str()` assertion to `test_difficulty_str_all_types`
   - Add `ChallengeType::Snake` to icon uniqueness and non-empty tests

7. **`src/input.rs`**
   - Add `use crate::challenges::snake::logic::{apply_game_result as apply_snake_result, process_input as process_snake_input, SnakeInput};`
   - Add `ActiveMinigame::Snake(game)` arm to `handle_minigame()`:
     ```rust
     ActiveMinigame::Snake(snake_game) => {
         if snake_game.game_result.is_some() {
             state.last_minigame_win = apply_snake_result(state);
             return InputResult::Continue;
         }
         let input = match key.code {
             KeyCode::Up => SnakeInput::Up,
             KeyCode::Down => SnakeInput::Down,
             KeyCode::Left => SnakeInput::Left,
             KeyCode::Right => SnakeInput::Right,
             KeyCode::Char(' ') => SnakeInput::Start,
             KeyCode::Esc => SnakeInput::Forfeit,
             _ => SnakeInput::Other,
         };
         process_snake_input(snake_game, input);
     }
     ```

8. **`src/main.rs`**
   - Add `ActiveMinigame::Snake` to `is_realtime_minigame()` match:
     ```rust
     fn is_realtime_minigame(state: &GameState) -> bool {
         matches!(
             state.active_minigame,
             Some(challenges::ActiveMinigame::FlappyBird(_))
                 | Some(challenges::ActiveMinigame::Snake(_))
         )
     }
     ```
   - Add Snake tick alongside Flappy Bird in the real-time tick block:
     ```rust
     if let Some(challenges::ActiveMinigame::Snake(ref mut game)) =
         state.active_minigame
     {
         challenges::snake::logic::tick_snake(game, dt.as_millis() as u64);
     }
     ```

9. **`src/ui/mod.rs`**
   - Add `mod snake_scene;`
   - Add `ActiveMinigame::Snake(game)` arm to `draw_right_content()`:
     ```rust
     Some(ActiveMinigame::Snake(game)) => {
         snake_scene::render_snake_scene(frame, area, game);
     }
     ```

10. **`src/utils/debug_menu.rs`**
    - Add "Trigger Snake Challenge" to `DEBUG_OPTIONS`
    - Add `trigger_snake_challenge()` function
    - Wire into `trigger_selected()` match

11. **`src/achievements/types.rs`**
    - Add 4 achievement IDs: `SnakeNovice`, `SnakeApprentice`, `SnakeJourneyman`, `SnakeMaster`
    - Add `("snake", difficulty)` arms to `on_minigame_won()` match

12. **`src/achievements/data.rs`**
    - Add 4 `AchievementDef` entries for Snake achievements (category: Challenges, icon: "~")

## Achievement Definitions

```rust
// In types.rs AchievementId enum:
SnakeNovice,
SnakeApprentice,
SnakeJourneyman,
SnakeMaster,

// In data.rs ALL_ACHIEVEMENTS:
AchievementDef {
    id: AchievementId::SnakeNovice,
    name: "Snake Novice",
    description: "Win Snake on Novice difficulty",
    category: AchievementCategory::Challenges,
    icon: "~",
},
AchievementDef {
    id: AchievementId::SnakeApprentice,
    name: "Snake Apprentice",
    description: "Win Snake on Apprentice difficulty",
    category: AchievementCategory::Challenges,
    icon: "~",
},
AchievementDef {
    id: AchievementId::SnakeJourneyman,
    name: "Snake Journeyman",
    description: "Win Snake on Journeyman difficulty",
    category: AchievementCategory::Challenges,
    icon: "~",
},
AchievementDef {
    id: AchievementId::SnakeMaster,
    name: "Snake Master",
    description: "Win Snake on Master difficulty",
    category: AchievementCategory::Challenges,
    icon: "~",
},

// In types.rs on_minigame_won() match:
("snake", "novice") => Some(AchievementId::SnakeNovice),
("snake", "apprentice") => Some(AchievementId::SnakeApprentice),
("snake", "journeyman") => Some(AchievementId::SnakeJourneyman),
("snake", "master") => Some(AchievementId::SnakeMaster),
```

## Challenge Menu Configuration

### Discovery Weight

```rust
ChallengeWeight {
    challenge_type: ChallengeType::Snake,
    weight: 20, // ~13% - moderate, action game alongside Flappy Bird
},
```

Weight 20 places Snake at the same tier as Flappy Bird and Gomoku (moderate discovery rate).

### ChallengeType Additions

```rust
// In ChallengeType enum:
Snake,

// Icon:
ChallengeType::Snake => "~",

// Discovery flavor:
ChallengeType::Snake => "A serpentine trail of glowing runes appears on the dungeon floor...",

// PendingChallenge:
ChallengeType::Snake => PendingChallenge {
    challenge_type: ChallengeType::Snake,
    title: "Serpent's Path".to_string(),
    icon: "~",
    description: "A serpentine trail of glowing runes slithers across the dungeon floor. \
        As you step closer, they coil into a grid of ancient symbols. A spectral voice \
        hisses: \"Guide the serpent through the maze. Feed it, grow it, but beware your \
        own trail. The path is narrow, and the serpent is hungry.\"".to_string(),
},
```

## Real-Time Tick Integration

Snake is a real-time game. It follows the exact same pattern as Flappy Bird:

1. **`is_realtime_minigame()`** returns `true` for `ActiveMinigame::Snake`
2. This activates non-blocking event polling (`Duration::ZERO`) and multi-event drain per frame
3. The main loop calls `tick_snake()` every `REALTIME_FRAME_MS` (16ms)
4. `tick_snake()` accumulates time and steps physics at `move_interval_ms` intervals
5. `last_flappy_frame` (should be renamed to `last_realtime_frame`) tracks the dt for all real-time games

### Frame Timing

The main loop's `last_flappy_frame` `Instant` is used for both Flappy Bird and Snake.
Only one real-time minigame can be active at a time (they share `active_minigame`),
so the same timer works for both. The existing code structure in `main.rs`:

```rust
if realtime_mode {
    let dt = last_flappy_frame.elapsed();
    if dt >= Duration::from_millis(REALTIME_FRAME_MS) {
        // Tick the active real-time game
        if let Some(challenges::ActiveMinigame::FlappyBird(ref mut game)) = state.active_minigame {
            challenges::flappy::logic::tick_flappy_bird(game, dt.as_millis() as u64);
        }
        if let Some(challenges::ActiveMinigame::Snake(ref mut game)) = state.active_minigame {
            challenges::snake::logic::tick_snake(game, dt.as_millis() as u64);
        }
        last_flappy_frame = Instant::now();
    }
}
```

Note: These are mutually exclusive (only one `active_minigame` at a time), so only one branch
will ever execute. Using `if let` for each is clean and avoids nested matching.

## Test Plan

Tests should be placed in `types.rs` and `logic.rs` following the existing Flappy Bird test patterns:

### types.rs Tests
- `test_new_game_defaults` -- Verify initial state (body length, direction, score, waiting_to_start)
- `test_difficulty_parameters` -- Verify all difficulty methods return correct values
- `test_difficulty_from_index` -- Verify from_index mapping
- `test_difficulty_names` -- Verify name() strings
- `test_all_difficulties` -- Verify ALL constant length
- `test_random_food_position` -- Verify food doesn't spawn on snake body
- `test_direction_is_opposite` -- Verify all opposite pairs

### logic.rs Tests
- `test_waiting_to_start_blocks_input` -- Non-Start input ignored, Start begins game
- `test_waiting_to_start_blocks_physics` -- tick_snake returns false, no movement
- `test_process_input_direction_change` -- Arrow keys set pending_direction
- `test_process_input_no_reversal` -- Opposite direction rejected
- `test_process_input_forfeit_flow` -- First Esc sets pending, second confirms
- `test_process_input_forfeit_cancelled` -- Any key cancels forfeit
- `test_process_input_ignored_when_game_over` -- No input after result set
- `test_physics_snake_moves` -- After tick, head advances in direction
- `test_physics_snake_grows_on_food` -- Body length increases when eating
- `test_physics_snake_doesnt_grow_without_food` -- Body length stays same
- `test_collision_wall` -- Death on out-of-bounds
- `test_collision_self` -- Death on self-intersection
- `test_win_condition` -- Score reaches target triggers Win
- `test_physics_paused_during_forfeit` -- No movement during forfeit_pending
- `test_dt_clamped` -- Large dt clamped to MAX_DT_MS
- `test_tick_returns_false_when_game_over` -- No ticks after result
- `test_reward_structure` -- Verify rewards for each difficulty
- `test_apply_game_result_win` -- Win grants rewards and clears minigame
- `test_apply_game_result_loss` -- Loss grants nothing and clears minigame
- `test_extra_info` -- Verify DifficultyInfo extra_info format
- `test_difficulty_str_values` -- Verify difficulty_str() returns lowercase
