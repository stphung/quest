# Flappy Bird Challenge: "Skyward Gauntlet"

## Overview

A real-time action minigame for the Quest challenge system. The player controls a bird navigating through a series of pipe gaps. This is Quest's first real-time (non-turn-based) challenge, requiring smooth frame-rate rendering and physics-based movement.

**Challenge Title:** Skyward Gauntlet
**Icon:** `>`  (arrow/bird symbol -- simple ASCII, unique among challenge icons)
**Prestige Requirement:** P1+ (same as other challenges)

---

## 1. Core Mechanics

### 1.1 Game Area

- **Width:** 50 characters
- **Height:** 18 characters (rows 0-17)
- Row 0: ceiling (solid boundary)
- Rows 1-16: playable airspace
- Row 17: ground (solid boundary, rendered as `=====...`)

The game area fits within the right panel of the standard `create_game_layout` (content area is roughly 50-60 chars wide, 15-20 chars tall after borders).

### 1.2 Bird

- **Position:** Fixed horizontal position at column 8 (left side of screen)
- **Vertical position:** Floating-point `y` coordinate, rendered to nearest row
- **Size:** 1 character tall, 2 characters wide
- **Render:** `=>` (normal), `=^` (flapping frame, shown for 3 ticks after flap)
- **Gravity:** Pulls bird downward each tick (value varies by difficulty)
- **Flap:** Pressing Space (or Up arrow) applies an upward impulse
- **Terminal velocity:** Capped at 1.5 rows/tick downward to prevent tunneling through pipes

### 1.3 Physics (per tick at 30 FPS)

```
velocity += gravity          // gravity pulls down each tick
velocity = min(velocity, terminal_velocity)  // cap fall speed
y += velocity                // update position
```

On flap:
```
velocity = flap_impulse      // negative value (upward), replaces current velocity
```

This "replace velocity" model (rather than additive) matches classic Flappy Bird feel and prevents exploits from rapid key mashing.

### 1.4 Pipes

- **Structure:** Each pipe is a vertical column with a gap
- **Width:** 3 characters wide (`|||` pattern using `|` characters)
- **Gap:** An opening in the pipe where the bird can pass safely
- **Movement:** Pipes scroll left at a constant speed (chars/tick, varies by difficulty)
- **Spacing:** Horizontal distance between consecutive pipes (varies by difficulty)
- **Generation:** New pipes spawn off the right edge with random gap positions
- **Gap vertical position:** Random, but constrained so the gap center is between rows 3 and 14 (ensuring the gap never clips the ceiling/floor boundaries)

Pipe rendering (example, gap_size=6):
```
  |||
  |||
  |||
  |||
  |||        <- top pipe section
            <- gap (6 rows of open space)
  |||        <- bottom pipe section
  |||
  |||
  |||
```

### 1.5 Collision Detection

The bird collides (and loses) if:
1. **Bird touches a pipe:** Bird's bounding box (2 chars wide, 1 char tall at rounded y) overlaps any pipe column character
2. **Bird touches the ground:** `round(y) >= 17`
3. **Bird touches the ceiling:** `round(y) <= 0`

Collision is checked after each position update. Pipe positions are stored as floating-point for smooth scrolling, but collision uses integer rounding (same grid the player sees).

### 1.6 Scoring

- **+1 point** each time a pipe's right edge scrolls past the bird's horizontal position
- Score is displayed in the top-right corner of the game area: `Score: 12/20`
- A progress bar or fraction shows how close the player is to winning

---

## 2. Difficulty Tiers

All four difficulties use the same core mechanics but tune the parameters:

| Parameter | Novice | Apprentice | Journeyman | Master |
|---|---|---|---|---|
| Gravity (rows/tick) | 0.08 | 0.10 | 0.12 | 0.15 |
| Flap impulse (rows/tick) | -0.55 | -0.55 | -0.58 | -0.60 |
| Terminal velocity | 1.2 | 1.3 | 1.4 | 1.5 |
| Pipe gap size (rows) | 7 | 6 | 5 | 4 |
| Pipe speed (cols/tick) | 0.15 | 0.20 | 0.25 | 0.30 |
| Pipe spacing (cols) | 20 | 17 | 15 | 13 |
| Pipes to win | 10 | 15 | 20 | 30 |

**Extra info display in challenge menu:**
- Novice: "10 pipes, wide gaps"
- Apprentice: "15 pipes, normal gaps"
- Journeyman: "20 pipes, narrow gaps"
- Master: "30 pipes, razor gaps"

### Design Rationale

- **Novice:** Very forgiving. Gap of 7 rows means nearly half the screen is open. Slow pipe speed gives plenty of reaction time. Only 10 pipes keeps it short. A player who can flap rhythmically should win.
- **Apprentice:** Standard Flappy Bird feel. Gap of 6 is comfortable but requires attention. 15 pipes adds endurance pressure.
- **Journeyman:** Requires precise control. Gap of 5 leaves little margin. Faster pipes demand quicker reactions. 20 pipes is a real endurance test.
- **Master:** Punishing. Gap of 4 rows with high gravity and fast pipes means nearly pixel-perfect timing. 30 pipes at this difficulty is a significant achievement.

---

## 3. Win / Loss Conditions

### Win
- Successfully pass the required number of pipes (varies by difficulty)
- On winning, the game freezes, displays a victory overlay, and any key exits

### Loss
- Collide with a pipe, the ground, or the ceiling
- On losing, the game freezes for a brief moment (10 ticks / 0.33s), displays a defeat overlay with final score, and any key exits

### Forfeit
- First Esc press: sets `forfeit_pending = true`, status bar shows "Press Esc again to forfeit..."
- Second Esc: confirms forfeit, treated as a loss (no reward)
- Any other key: cancels forfeit, resumes gameplay
- Note: game physics PAUSE while `forfeit_pending` is true (unlike other challenges where the game is turn-based, pausing is necessary here to be fair)

---

## 4. Reward Structure

Flappy Bird is an action/reflex challenge, positioned between the quick puzzles (Minesweeper, Rune) and the deep strategy games (Chess, Go). The rewards reflect moderate difficulty with emphasis on XP and some prestige at higher tiers.

```rust
impl DifficultyInfo for FlappyBirdDifficulty {
    fn reward(&self) -> ChallengeReward {
        match self {
            FlappyBirdDifficulty::Novice => ChallengeReward {
                xp_percent: 50,
                ..Default::default()
            },
            FlappyBirdDifficulty::Apprentice => ChallengeReward {
                xp_percent: 100,
                ..Default::default()
            },
            FlappyBirdDifficulty::Journeyman => ChallengeReward {
                prestige_ranks: 1,
                xp_percent: 75,
                ..Default::default()
            },
            FlappyBirdDifficulty::Master => ChallengeReward {
                prestige_ranks: 2,
                xp_percent: 150,
                fishing_ranks: 1,
                ..Default::default()
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
- Novice/Apprentice rewards match Minesweeper (50%/75-100% XP) -- appropriate for an accessible action game
- Journeyman gives 1 prestige rank (like Gomoku Journeyman), recognizing real skill
- Master gives 2 prestige + fish rank + XP, slightly above Gomoku Master but below Chess/Go Master (5 prestige) since it tests reflexes rather than deep strategy

---

## 5. ASCII Visual Design

### 5.1 Bird Sprites

```
Normal:  =>
Flap:    =^
Dead:    =x
```

The bird is rendered in `Color::Yellow` (bright, stands out against the dark background).

### 5.2 Pipe Rendering

Pipes use box-drawing-like characters for a solid look:

```
Top pipe:     |#|
              |#|
              |#|
              |_|    <- pipe cap (bottom of top section)
                     <- gap (empty space)
              |~|    <- pipe cap (top of bottom section)
              |#|
              |#|
              |#|
```

- Pipe body: `|#|` (3 chars wide) in `Color::Green`
- Pipe caps: `|_|` and `|~|` in `Color::DarkGray` to visually delineate the gap edges
- Gap: empty space (no characters)

### 5.3 Ground

```
==================    <- ground line at row 17
```

Rendered in `Color::DarkGray`.

### 5.4 Full Scene Example (Novice, gap=7)

```
 Skyward Gauntlet                Score: 3/10
--------------------------------------------------
                    |#|
                    |#|
                    |#|
                    |_|
                                         |#|
    =>                                   |#|
                                         |#|
                                         |_|
                    |~|
                    |#|
                    |#|                  |~|
                    |#|                  |#|
                    |#|                  |#|
                    |#|                  |#|
                    |#|                  |#|
==================================================
--------------------------------------------------
 [Space/Up] Flap  [Esc] Forfeit
```

### 5.5 Score Display

Top-right corner of game area:
- During play: `Score: 3/10` in `Color::White`
- Pipe counter acts as a progress indicator so the player knows how close they are to winning

### 5.6 Game Over Overlays

Use the shared `render_game_over_overlay` from `game_common.rs`:

**Win overlay:**
- Title: "Victory!"
- Result type: `GameResultType::Win`
- Message: "You navigated the gauntlet! {score}/{target} pipes cleared."

**Loss overlay:**
- Title: "Defeated"
- Result type: `GameResultType::Loss`
- Message: "Crashed after {score} pipes. The gauntlet claims another."

---

## 6. RPG-Themed Flavor

### 6.1 Challenge Title
**"Skyward Gauntlet"**

### 6.2 Discovery Flavor Text
`ChallengeType::FlappyBird => "A tiny clockwork bird whirs to life on a nearby ledge..."`

### 6.3 Full Discovery Description (PendingChallenge)
> A tiny clockwork bird sits on a moss-covered ledge, its brass wings twitching. As you approach, it springs to life, darting between a series of crumbling stone pillars. A runic inscription glows beneath it: "Guide the Skyward Vessel through the gauntlet. Prove your reflexes worthy of a true adventurer." The bird hovers expectantly, waiting for your command.

### 6.4 Combat Log Messages
- Discovery: `> A clockwork bird challenges you to a Skyward Gauntlet!`
- Win: `> You conquered the Skyward Gauntlet! (+{reward})`
- Loss: `> The clockwork bird sputters and falls. The gauntlet stands unbroken.`
- Forfeit: `> You walk away from the Skyward Gauntlet.`

---

## 7. Discovery Weight

```rust
ChallengeWeight {
    challenge_type: ChallengeType::FlappyBird,
    weight: 20, // ~15% - moderate frequency, action game novelty
},
```

**Updated probability table** (total weight: 130):

| Challenge | Weight | Probability |
|---|---|---|
| Minesweeper | 30 | ~23% |
| Rune | 25 | ~19% |
| **FlappyBird** | **20** | **~15%** |
| Gomoku | 20 | ~15% |
| Morris | 15 | ~12% |
| Chess | 10 | ~8% |
| Go | 10 | ~8% |

**Rationale:** Weight of 20 (same as Gomoku) because:
- It's a novel game type (action) so players should see it reasonably often
- It's quick to play (30-60 seconds), similar to puzzles
- It's not as deep as Chess/Go, so higher frequency is appropriate

---

## 8. Frame Rate and Tick Architecture

### 8.1 The Problem

Quest's main game loop runs at 100ms ticks (10 FPS). This is fine for turn-based challenges, but Flappy Bird needs 30+ FPS for smooth physics and responsive controls.

### 8.2 Solution: Sub-tick Rendering

The Flappy Bird game operates on its own timing within Quest's architecture:

- **Game tick rate:** 33ms (~30 FPS) driven by a separate timer within the game's tick function
- **Implementation:** The main game loop's 100ms tick calls `tick_flappy_bird()`, which internally tracks elapsed time using `Instant` and advances the physics simulation in 33ms steps. This means each 100ms main tick produces ~3 physics updates.
- **Input handling:** Flap input is buffered and consumed on the next physics step, ensuring responsive feel even if input arrives between physics updates.
- **Rendering:** Every main loop render pass draws the current Flappy Bird state. Since Ratatui redraws on every frame and the main loop polls at ~100ms, the visual update rate is tied to the main loop. To achieve smoother visuals, the main loop's poll timeout should be reduced to ~33ms while a Flappy Bird game is active.

### 8.3 Tick Function Signature

```rust
/// Advance flappy bird physics. Called from the main game tick.
/// `dt_ms` is milliseconds since last call.
/// Returns true if the game state changed (needs redraw).
pub fn tick_flappy_bird(game: &mut FlappyBirdGame, dt_ms: u64) -> bool {
    // Accumulate time, step physics in 33ms increments
    // Check collisions after each step
    // Return true if any state changed
}
```

### 8.4 Main Loop Integration

When a Flappy Bird game is active:
1. The `event::poll()` timeout in `main.rs` is reduced from 100ms to 33ms
2. Each poll cycle calls `tick_flappy_bird()` with actual elapsed time
3. This gives ~30 FPS rendering and physics, achieving smooth movement

When no Flappy Bird game is active, the poll timeout returns to 100ms (no performance impact on the rest of the game).

### 8.5 Why 30 FPS is Sufficient

- Terminal rendering is character-cell based (discrete positions), so sub-pixel smoothness is impossible
- 30 FPS gives ~33ms between frames, well within human reaction time (~200ms)
- Pipe movement at 0.30 cols/tick (Master) at 30 FPS = ~9 cols/second, smooth and readable
- Higher FPS (60) would provide marginal benefit in a terminal while doubling CPU usage

---

## 9. State Structure

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlappyBirdDifficulty {
    Novice,
    Apprentice,
    Journeyman,
    Master,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlappyBirdResult {
    Win,
    Loss,
}

#[derive(Debug, Clone)]
pub struct Pipe {
    pub x: f64,           // horizontal position (float for smooth scrolling)
    pub gap_center: u16,  // row index of gap center
}

#[derive(Debug, Clone)]
pub struct FlappyBirdGame {
    pub difficulty: FlappyBirdDifficulty,
    pub game_result: Option<FlappyBirdResult>,
    pub forfeit_pending: bool,

    // Bird state
    pub bird_y: f64,        // vertical position (float for smooth physics)
    pub bird_velocity: f64, // current vertical velocity
    pub flap_timer: u32,    // ticks remaining to show flap animation

    // Pipe state
    pub pipes: Vec<Pipe>,   // active pipes on screen
    pub next_pipe_x: f64,   // x position where next pipe will spawn

    // Scoring
    pub score: u32,         // pipes successfully passed
    pub target_score: u32,  // pipes needed to win

    // Timing
    pub accumulated_time_ms: u64, // sub-tick time accumulator
    pub tick_count: u64,          // total physics ticks elapsed

    // Input buffer
    pub flap_queued: bool,  // flap input waiting to be consumed

    // Difficulty parameters (cached from difficulty)
    pub gravity: f64,
    pub flap_impulse: f64,
    pub terminal_velocity: f64,
    pub pipe_gap: u16,
    pub pipe_speed: f64,
    pub pipe_spacing: f64,
}
```

---

## 10. Input Mapping

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlappyBirdInput {
    Flap,    // Space or Up arrow
    Cancel,  // Esc (forfeit flow)
    Other,   // Any other key (cancels forfeit_pending)
}
```

Key mapping in `input.rs`:
- `KeyCode::Char(' ')` => `FlappyBirdInput::Flap`
- `KeyCode::Up` => `FlappyBirdInput::Flap`
- `KeyCode::Esc` => `FlappyBirdInput::Cancel`
- Everything else => `FlappyBirdInput::Other`

Note: Unlike turn-based games, input processing does NOT pause the game (except during forfeit confirmation). The `flap_queued` flag ensures the flap is consumed on the next physics tick rather than being lost between frames.

---

## 11. Testing Strategy

### Unit Tests
- Physics: verify gravity accumulates correctly over N ticks
- Physics: verify flap impulse replaces velocity
- Physics: verify terminal velocity cap
- Collision: bird vs pipe (overlap detection)
- Collision: bird vs ground/ceiling
- Scoring: score increments when pipe passes bird x position
- Win condition: game result set to Win when score reaches target
- Loss condition: game result set to Loss on collision
- Pipe generation: gaps are within valid range (rows 3-14)
- Pipe spacing: new pipes spawn at correct intervals
- Difficulty parameters: each tier has correct values
- Forfeit: double-Esc triggers forfeit, other key cancels

### Integration Tests
- Full game simulation: programmatically flap at correct intervals to pass N pipes and verify win
- Full game simulation: let bird fall without flapping, verify loss on ground collision
- Reward application: verify ChallengeReward values are correct per difficulty
- Achievement integration: verify MinigameWinInfo is emitted on win

---

## 12. Module Structure

```
src/challenges/flappy_bird/
    mod.rs      # Public exports (FlappyBirdGame, FlappyBirdDifficulty, FlappyBirdResult, etc.)
    types.rs    # Data structures (FlappyBirdGame, Pipe, Difficulty, Result, params)
    logic.rs    # Physics simulation, input processing, collision detection, scoring

src/ui/
    flappy_bird_scene.rs  # Rendering (bird, pipes, ground, score, overlays)
```

---

## 13. Achievement Integration

Following the existing pattern, winning a Flappy Bird game emits:

```rust
MinigameWinInfo {
    game_type: "flappy_bird".to_string(),
    difficulty: difficulty.difficulty_str().to_string(),
}
```

This enables future achievements like:
- "Skyward Novice" - Win a Skyward Gauntlet on Novice
- "Skyward Master" - Win a Skyward Gauntlet on Master
- "Bird Brain" - Win 10 Skyward Gauntlets at any difficulty
