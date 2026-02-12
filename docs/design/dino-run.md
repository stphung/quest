# Dino Run Challenge: "Gauntlet Run"

## Overview

A real-time action minigame for the Quest challenge system. The player controls a runner dodging dungeon traps in a Chrome dinosaur-style endless runner. This is Quest's second real-time (non-turn-based) challenge, reusing the sub-tick rendering architecture established by Skyward Gauntlet (Flappy Bird).

**Challenge Title:** Gauntlet Run
**Icon:** `!`  (exclamation -- danger/traps, unique among challenge icons)
**Prestige Requirement:** P1+ (same as other challenges)

---

## 1. Core Mechanics

### 1.1 Game Area

- **Width:** 50 characters
- **Height:** 18 characters (rows 0-17)
- Row 0: ceiling (decorative dungeon ceiling `~~~~~...`)
- Rows 1-13: upper airspace (flying obstacles appear here)
- Row 14: runner standing head position
- Row 15: runner standing body/feet position (ground level for runner)
- Row 16: ground surface (`######...`)
- Row 17: below ground (decorative `......`)

The game area fits within the right panel of the standard `create_game_layout` (content area is roughly 50-60 chars wide, 15-20 chars tall after borders).

### 1.2 Runner

- **Position:** Fixed horizontal position at columns 5-6 (left side of screen)
- **Vertical position:** Integer row, with smooth transition during jumps via float `y` coordinate
- **Size:** 2 characters wide, 2 rows tall (standing), 2 characters wide, 1 row tall (ducking)
- **States:**
  - **Running (standing):** Occupies rows 14-15, alternates between two animation frames
  - **Jumping:** Follows a parabolic arc upward, occupies rows (y) to (y+1)
  - **Ducking:** Occupies row 15 only (1 row tall), hitbox is half height
  - **Dead:** Shows crash sprite, game over

**Render sprites:**
```
Running frame 1:  o>     Running frame 2:  o>
                  />                       \>

Ducking:          o>
                  (on ground row only, 1 row tall)

Jumping:          o^
                  />

Dead:             oX
                  />
```

The runner is rendered in `Color::Yellow` (matches Skyward Gauntlet bird for consistency).

### 1.3 Physics (per tick at ~60 FPS / 16ms)

**Jumping:**
```
if on_ground and jump_pressed:
    velocity_y = jump_impulse       // negative = upward
    on_ground = false

velocity_y += gravity               // gravity pulls down each tick
y += velocity_y                      // update position

if y >= ground_y:                    // landed
    y = ground_y
    velocity_y = 0.0
    on_ground = true
```

**Ducking:**
- While Down is held: runner hitbox shrinks to 1 row (row 15 only)
- Cannot jump while ducking
- Cannot duck while in the air
- Releasing Down returns to standing hitbox (2 rows: 14-15)

**Key design: No terminal velocity needed.** Unlike Flappy Bird, the runner only jumps from ground level and the arc is short enough that velocity capping is unnecessary.

### 1.4 Obstacles

Three types of obstacles scroll from right to left:

**Ground obstacles (jump over):**
- **Spikes:** `^` or `^^` or `^^^` (1-3 chars wide, 1 row tall on row 16... no, on row 15)
- Actually: ground obstacles sit on the ground surface. They occupy row 15 (same row as runner's feet).
- Small: 1 char wide `^`, Medium: 2 chars `^^`, Large: 3 chars `^^^`
- Rendered in `Color::Red`

**Flying obstacles (duck under):**
- **Bats/Blades:** `~v~` or `\|/` (3 chars wide, 1 row tall)
- Occupy row 14 (runner's head level when standing)
- The runner must duck to avoid these (ducking removes the head hitbox at row 14)
- Rendered in `Color::Magenta`

**Tall obstacles (must jump with good timing):**
- **Pillars:** 2 rows tall (rows 14-15), 1-2 chars wide `#` or `##`
- These block both standing and ducking -- must jump over
- Rendered in `Color::DarkGray`

Obstacle generation:
- Obstacles spawn off the right edge of the screen
- Minimum spacing between obstacles varies by difficulty
- Obstacle type is randomly chosen from the pool available at the current difficulty
- At higher difficulties, mixed sequences (e.g., ground obstacle immediately followed by flying obstacle) create challenging patterns

### 1.5 Collision Detection

The runner collides (and loses) if:
1. **Runner overlaps a ground obstacle:** Runner's foot row (row 15) overlaps an obstacle on the same row, and horizontal positions overlap
2. **Runner overlaps a flying obstacle:** Runner's head row (row 14, only when standing) overlaps a flying obstacle at the same columns
3. **Runner overlaps a tall obstacle:** Any occupied runner row overlaps the obstacle columns and rows

Collision uses integer rounding for both runner position and obstacle positions (same grid the player sees). Horizontal overlap checks: runner occupies columns 5-6 (2 chars wide), obstacle occupies its column range.

### 1.6 Scoring

- **+1 point** each time an obstacle's right edge scrolls past the runner's horizontal position
- Score is displayed in the top-right corner: `Score: 12/25`
- A fraction shows progress toward winning

### 1.7 Speed Progression

The game speed increases over time within each run:

```
current_speed = base_speed + (speed_increment * obstacles_passed)
```

This creates a natural difficulty curve: early obstacles are manageable, later ones demand quicker reactions. The speed increment is tuned per difficulty so the final obstacles approach but don't exceed the max speed.

---

## 2. Difficulty Tiers

All four difficulties use the same core mechanics but tune the parameters:

| Parameter | Novice | Apprentice | Journeyman | Master |
|---|---|---|---|---|
| Base speed (cols/tick) | 0.10 | 0.13 | 0.16 | 0.20 |
| Max speed (cols/tick) | 0.14 | 0.19 | 0.25 | 0.32 |
| Speed increment per obstacle | 0.003 | 0.003 | 0.003 | 0.003 |
| Gravity (rows/tick) | 0.008 | 0.009 | 0.010 | 0.012 |
| Jump impulse (rows/tick) | -0.22 | -0.23 | -0.24 | -0.26 |
| Min obstacle spacing (cols) | 18 | 15 | 12 | 10 |
| Obstacles to win | 15 | 25 | 35 | 50 |
| Obstacle types | Ground only | Ground + Flying | Ground + Flying + Tall | All + mixed combos |

**Extra info display in challenge menu:**
- Novice: "15 traps, ground only"
- Apprentice: "25 traps, mixed"
- Journeyman: "35 traps, fast"
- Master: "50 traps, gauntlet"

### Design Rationale

- **Novice:** Very forgiving. Only ground obstacles (jump only). Slow speed, wide spacing. A player who can time jumps should win. Target ~15 obstacles to keep it short.
- **Apprentice:** Introduces flying obstacles, requiring the player to learn ducking. Medium speed. 25 obstacles adds endurance.
- **Journeyman:** All obstacle types including tall pillars. Fast speed with tighter spacing. 35 obstacles is a real endurance test. Mixed sequences (ground then flying) require quick stance changes.
- **Master:** Very fast, dense obstacles, mixed combos. 50 obstacles at accelerating speed is a significant achievement. Speed ramps up to 0.32 cols/tick, demanding precise timing.

---

## 3. Win / Loss Conditions

### Win
- Successfully dodge the required number of obstacles (varies by difficulty)
- On winning, the game freezes, displays a victory overlay, and any key exits

### Loss
- Collide with any obstacle
- On losing, the game freezes for a brief moment (10 ticks / ~160ms), displays a defeat overlay with final score, and any key exits

### Forfeit
- First Esc press: sets `forfeit_pending = true`, status bar shows "Press Esc again to forfeit..."
- Second Esc: confirms forfeit, treated as a loss (no reward)
- Any other key: cancels forfeit, resumes gameplay
- Note: game physics PAUSE while `forfeit_pending` is true (same as Skyward Gauntlet -- pausing is necessary for fairness in real-time games)

---

## 4. Reward Structure

Gauntlet Run is an action/reflex challenge, matching Skyward Gauntlet (Flappy Bird) in reward tier. Both test reflexes rather than deep strategy.

```rust
impl DifficultyInfo for DinoRunDifficulty {
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
}
```

**Reward summary:**

| Difficulty | Prestige | XP% | Fishing | Display Text |
|---|---|---|---|---|
| Novice | 0 | 50% | 0 | Win: +50% level XP |
| Apprentice | 0 | 100% | 0 | Win: +100% level XP |
| Journeyman | 1 | 75% | 0 | Win: +1 Prestige Rank, +75% level XP |
| Master | 2 | 150% | 1 | Win: +2 Prestige Ranks, +1 Fish Rank, +150% level XP |

**Comparison with existing challenges:**
- Identical to Skyward Gauntlet (Flappy Bird) rewards -- both are reflex-based action games of similar difficulty
- Below Chess/Go Master (5 prestige) since it tests reflexes rather than deep strategy

---

## 5. ASCII Visual Design

### 5.1 Runner Sprites

```
Standing frame 1:  o>     Standing frame 2:  o>
                   />                        \>

Ducking:           o=

Jumping (rising):  o^     Jumping (falling):  o>
                   />                         />

Dead:              oX
                   />
```

- Standing animation alternates legs (`/>` and `\>`) every 4 ticks for a running effect
- Ducking sprite is 1 row: `o=` (crouched, on row 15)
- Runner rendered in `Color::Yellow`

### 5.2 Obstacle Rendering

**Ground obstacles (spikes):**
```
Small:   ^     (1 char, row 15)
Medium:  ^^    (2 chars, row 15)
Large:   ^^^   (3 chars, row 15)
```
Rendered in `Color::Red`.

**Flying obstacles:**
```
Bat:     ~v~   (3 chars, row 14)
Blade:   \|/   (3 chars, row 14)
```
Rendered in `Color::Magenta`.

**Tall obstacles (pillars):**
```
Small:   #     (1 char, rows 14-15)
Large:   ##    (2 chars, rows 14-15)
```
Rendered in `Color::DarkGray`.

### 5.3 Ground

```
################...###    <- ground surface at row 16
..................        <- below ground at row 17
```

Ground rendered in `Color::DarkGray`. Uses `#` characters for a dungeon floor feel.

### 5.4 Ceiling

```
~~~~~~~~~~~~~~~~~~~~~~    <- ceiling at row 0
```

Rendered in `Color::DarkGray`. Decorative only, no collision.

### 5.5 Full Scene Example (Apprentice, mid-run)

```
 Gauntlet Run                       Score: 8/25
--------------------------------------------------
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~









                                         ~v~

    o>                 ^^         #
    />                            #
##################################################
..................................................
--------------------------------------------------
 [Space/Up] Jump  [Down] Duck  [Esc] Forfeit
```

### 5.6 Score Display

Top-right corner of game area:
- During play: `Score: 8/25` in `Color::White`
- Obstacle counter acts as a progress indicator

### 5.7 Game Over Overlays

Use the shared `render_game_over_overlay` from `game_common.rs`:

**Win overlay:**
- Title: "Victory!"
- Result type: `GameResultType::Win`
- Message: "You survived the gauntlet! {score}/{target} traps dodged."

**Loss overlay:**
- Title: "Defeated"
- Result type: `GameResultType::Loss`
- Message: "Hit a trap after dodging {score}. The gauntlet claims another."

---

## 6. RPG-Themed Flavor

### 6.1 Challenge Title
**"Gauntlet Run"**

### 6.2 Discovery Flavor Text
`ChallengeType::DinoRun => "The ground trembles as a hidden corridor reveals a deadly obstacle course..."`

### 6.3 Full Discovery Description (PendingChallenge)
> The dungeon wall splits apart, revealing a narrow corridor lined with deadly traps. Rusted spikes jut from the floor, pendulum blades swing overhead, and stone pillars block the path. A faded inscription carved above the entrance reads: "Only the swift survive the Gauntlet. Run, dodge, and leap -- or perish." A faint breeze carries the scent of old stone and danger.

### 6.4 Combat Log Messages
- Discovery: `> A hidden gauntlet corridor opens before you!`
- Win: `> You conquered the Gauntlet Run! (+{reward})`
- Loss: `> You stumble in the gauntlet. The traps reset behind you.`
- Forfeit: `> You retreat from the Gauntlet Run.`

---

## 7. Discovery Weight

```rust
ChallengeWeight {
    challenge_type: ChallengeType::DinoRun,
    weight: 20, // ~13% - moderate frequency, action game
},
```

**Updated probability table** (total weight: 150):

| Challenge | Weight | Probability |
|---|---|---|
| Minesweeper | 30 | ~20% |
| Rune | 25 | ~17% |
| FlappyBird | 20 | ~13% |
| **DinoRun** | **20** | **~13%** |
| Gomoku | 20 | ~13% |
| Morris | 15 | ~10% |
| Chess | 10 | ~7% |
| Go | 10 | ~7% |

**Rationale:** Weight of 20 (same as Flappy Bird and Gomoku) because:
- It's a novel action game type so players should see it reasonably often
- It's quick to play (30-90 seconds), similar to Skyward Gauntlet
- It's not as deep as Chess/Go, so higher frequency is appropriate

---

## 8. Frame Rate and Tick Architecture

### 8.1 Reuse Existing Real-Time Infrastructure

Gauntlet Run reuses the same sub-tick rendering architecture as Skyward Gauntlet:

- **Game tick rate:** 16ms (~60 FPS) using the `REALTIME_FRAME_MS` constant
- **Implementation:** The main game loop's 100ms tick calls `tick_dino_run()`, which internally tracks elapsed time using `Instant` and advances the physics simulation in 16ms steps. Each 100ms main tick produces ~6 physics updates.
- **Input handling:** Jump and duck inputs are buffered and consumed on the next physics tick.
- **Rendering:** Main loop poll timeout reduced to ~16ms while a Dino Run game is active (same as Flappy Bird).

### 8.2 Tick Function Signature

```rust
/// Advance dino run physics. Called from the main game tick.
/// `dt_ms` is milliseconds since last call.
/// Returns true if the game state changed (needs redraw).
pub fn tick_dino_run(game: &mut DinoRunGame, dt_ms: u64) -> bool {
    // Accumulate time, step physics in 16ms increments
    // Move obstacles, check collisions, update score
    // Return true if any state changed
}
```

### 8.3 Main Loop Integration

Same pattern as Flappy Bird: when a Dino Run game is active, `event::poll()` timeout is reduced from 100ms to 16ms. When no real-time game is active, it returns to 100ms.

---

## 9. State Structure

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DinoRunDifficulty {
    Novice,
    Apprentice,
    Journeyman,
    Master,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DinoRunResult {
    Win,
    Loss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObstacleType {
    /// Ground spikes -- jump over (1 row tall, on runner ground row)
    GroundSmall,   // 1 char wide
    GroundMedium,  // 2 chars wide
    GroundLarge,   // 3 chars wide
    /// Flying obstacle -- duck under (1 row tall, at runner head row)
    FlyingBat,     // 3 chars wide
    FlyingBlade,   // 3 chars wide
    /// Tall pillar -- must jump over (2 rows tall)
    TallSmall,     // 1 char wide
    TallLarge,     // 2 chars wide
}

#[derive(Debug, Clone)]
pub struct Obstacle {
    pub x: f64,              // horizontal position (float for smooth scrolling)
    pub obstacle_type: ObstacleType,
    pub passed: bool,        // whether runner has passed this obstacle (for scoring)
}

#[derive(Debug, Clone)]
pub struct DinoRunGame {
    pub difficulty: DinoRunDifficulty,
    pub game_result: Option<DinoRunResult>,
    pub forfeit_pending: bool,
    /// True until the player presses Space/Up to begin. Physics paused.
    pub waiting_to_start: bool,

    // Runner state
    pub runner_y: f64,         // vertical position (float for smooth jumping)
    pub runner_velocity: f64,  // vertical velocity (positive = downward)
    pub on_ground: bool,       // true when runner is on the ground
    pub is_ducking: bool,      // true while Down is held
    pub anim_frame: u32,       // running animation frame counter

    // Obstacle state
    pub obstacles: Vec<Obstacle>,
    pub next_obstacle_x: f64,  // x position where next obstacle spawns

    // Scoring
    pub score: u32,            // obstacles successfully dodged
    pub target_score: u32,     // obstacles needed to win

    // Speed
    pub base_speed: f64,       // initial scroll speed
    pub speed_increment: f64,  // speed increase per obstacle passed
    pub max_speed: f64,        // speed cap

    // Timing
    pub accumulated_time_ms: u64, // sub-tick time accumulator
    pub tick_count: u64,          // total physics ticks elapsed

    // Input buffer
    pub jump_queued: bool,     // jump input waiting to be consumed
    pub duck_held: bool,       // duck key currently held

    // Cached difficulty parameters
    pub gravity: f64,
    pub jump_impulse: f64,
    pub min_obstacle_spacing: f64,
}
```

---

## 10. Input Mapping

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DinoRunInput {
    Jump,      // Space or Up arrow
    DuckStart, // Down arrow pressed
    DuckEnd,   // Down arrow released (handled via key release detection or state toggle)
    Cancel,    // Esc (forfeit flow)
    Other,     // Any other key (cancels forfeit_pending)
}
```

Key mapping in `input.rs`:
- `KeyCode::Char(' ')` => `DinoRunInput::Jump`
- `KeyCode::Up` => `DinoRunInput::Jump`
- `KeyCode::Down` => `DinoRunInput::DuckStart` (toggle: if already ducking, treat as DuckEnd)
- `KeyCode::Esc` => `DinoRunInput::Cancel`
- Everything else => `DinoRunInput::Other`

**Duck implementation note:** Since terminal input doesn't reliably detect key-up events, ducking uses a toggle model: pressing Down starts ducking, pressing Down again (or Space/Up to jump) stops ducking. Alternatively, ducking could last for a fixed duration (e.g., 0.5 seconds) and auto-release. **Recommended: toggle model** -- press Down to duck, any other action (jump, another Down press) releases the duck. This is simpler and more reliable in terminal environments.

---

## 11. Testing Strategy

### Unit Tests
- Physics: verify jump arc (up, apex, down, land)
- Physics: verify gravity accumulates correctly
- Physics: verify runner cannot jump while airborne (no double-jump)
- Physics: verify runner cannot duck while airborne
- Ducking: hitbox shrinks to 1 row when ducking
- Ducking: standing hitbox is 2 rows
- Collision: runner vs ground obstacle (standing, hits; jumping, clears)
- Collision: runner vs flying obstacle (standing, hits; ducking, clears)
- Collision: runner vs tall obstacle (ducking, hits; jumping, clears)
- Scoring: score increments when obstacle passes runner x position
- Win condition: game result set to Win when score reaches target
- Loss condition: game result set to Loss on collision
- Speed progression: current speed increases with obstacles passed
- Speed progression: speed does not exceed max_speed
- Obstacle generation: obstacles spawn at correct intervals
- Obstacle generation: obstacle types match difficulty pool
- Difficulty parameters: each tier has correct values
- Forfeit: double-Esc triggers forfeit, other key cancels

### Integration Tests
- Full game simulation: programmatically jump/duck at correct intervals to pass N obstacles and verify win
- Full game simulation: let runner stand still, verify loss on first ground obstacle
- Reward application: verify ChallengeReward values are correct per difficulty
- Achievement integration: verify MinigameWinInfo is emitted on win

---

## 12. Module Structure

```
src/challenges/dino/
    mod.rs      # Public exports (DinoRunGame, DinoRunDifficulty, DinoRunResult, etc.)
    types.rs    # Data structures (DinoRunGame, Obstacle, ObstacleType, Difficulty, params)
    logic.rs    # Physics simulation, input processing, collision detection, scoring

src/ui/
    dino_run_scene.rs  # Rendering (runner, obstacles, ground, ceiling, score, overlays)
```

---

## 13. Achievement Integration

Following the existing pattern, winning a Dino Run game emits:

```rust
MinigameWinInfo {
    game_type: "dino_run".to_string(),
    difficulty: difficulty.difficulty_str().to_string(),
}
```

This enables future achievements like:
- "Gauntlet Novice" -- Win a Gauntlet Run on Novice
- "Gauntlet Master" -- Win a Gauntlet Run on Master
- "Trap Dodger" -- Win 10 Gauntlet Runs at any difficulty

---

## 14. Obstacle Generation Algorithm

```rust
fn spawn_obstacle<R: Rng>(game: &mut DinoRunGame, rng: &mut R) {
    let obstacle_type = match game.difficulty {
        // Novice: ground only
        DinoRunDifficulty::Novice => {
            match rng.gen_range(0..3) {
                0 => ObstacleType::GroundSmall,
                1 => ObstacleType::GroundMedium,
                _ => ObstacleType::GroundLarge,
            }
        }
        // Apprentice: ground + flying
        DinoRunDifficulty::Apprentice => {
            match rng.gen_range(0..5) {
                0 => ObstacleType::GroundSmall,
                1 => ObstacleType::GroundMedium,
                2 => ObstacleType::GroundLarge,
                3 => ObstacleType::FlyingBat,
                _ => ObstacleType::FlyingBlade,
            }
        }
        // Journeyman: ground + flying + tall
        DinoRunDifficulty::Journeyman => {
            match rng.gen_range(0..7) {
                0 => ObstacleType::GroundSmall,
                1 => ObstacleType::GroundMedium,
                2 => ObstacleType::GroundLarge,
                3 => ObstacleType::FlyingBat,
                4 => ObstacleType::FlyingBlade,
                5 => ObstacleType::TallSmall,
                _ => ObstacleType::TallLarge,
            }
        }
        // Master: all types, weighted toward harder ones
        DinoRunDifficulty::Master => {
            match rng.gen_range(0..10) {
                0 => ObstacleType::GroundSmall,
                1 | 2 => ObstacleType::GroundMedium,
                3 => ObstacleType::GroundLarge,
                4 | 5 => ObstacleType::FlyingBat,
                6 => ObstacleType::FlyingBlade,
                7 | 8 => ObstacleType::TallSmall,
                _ => ObstacleType::TallLarge,
            }
        }
    };

    // Add spacing jitter to prevent predictable patterns
    let jitter = rng.gen_range(0.0..5.0);
    let spacing = game.min_obstacle_spacing + jitter;

    game.obstacles.push(Obstacle {
        x: game.next_obstacle_x,
        obstacle_type,
        passed: false,
    });
    game.next_obstacle_x += spacing;
}
```

### Mixed Combo Generation (Master difficulty)

At Master difficulty, after scoring 20+ obstacles, there is a 30% chance to spawn a "combo" -- two obstacles with reduced spacing (60% of normal). This forces rapid stance switching (e.g., jump then immediately duck).

---

## 15. Physics Tuning Rationale

### Jump Arc

At Novice settings (gravity 0.008, impulse -0.22):
- Time to apex: `0.22 / 0.008 = 27.5 ticks` (~440ms)
- Peak height: `0.22^2 / (2 * 0.008) = 3.025 rows` above ground
- Total jump duration: ~55 ticks (~880ms)
- At base speed 0.10 cols/tick, runner traverses ~5.5 cols during a jump

This gives comfortable clearance over 3-char-wide ground obstacles and requires reasonable timing.

At Master settings (gravity 0.012, impulse -0.26):
- Time to apex: `0.26 / 0.012 = 21.7 ticks` (~347ms)
- Peak height: `0.26^2 / (2 * 0.012) = 2.82 rows`
- Total jump duration: ~43 ticks (~694ms)
- At max speed 0.32 cols/tick, runner traverses ~13.8 cols during a jump

The tighter timing at Master means jumps must be more precisely timed relative to obstacles.

### Speed Progression

With speed increment of 0.003 per obstacle:
- **Novice:** Start 0.10, after 15 obstacles: 0.10 + 15*0.003 = 0.145 (capped at 0.14 by max_speed)
- **Apprentice:** Start 0.13, after 25 obstacles: 0.13 + 25*0.003 = 0.205 (approaching max 0.19, capped)
- **Journeyman:** Start 0.16, after 35 obstacles: 0.16 + 35*0.003 = 0.265 (approaching max 0.25, capped)
- **Master:** Start 0.20, after 50 obstacles: 0.20 + 50*0.003 = 0.35 (capped at 0.32)

The speed ramp ensures the early game is approachable while the endgame creates tension.
