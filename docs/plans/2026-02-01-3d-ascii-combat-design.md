# First-Person 3D ASCII Combat System - Design

**Date:** 2026-02-01
**Status:** Approved
**Goal:** Transform combat visualization from simple emojis to immersive first-person 3D ASCII dungeon view

## Overview

This design creates a first-person dungeon crawler style combat view using ASCII art and perspective rendering. Players will see battles from their character's eyes, with enemies rendered as scaled ASCII sprites in a detailed stone dungeon environment with dynamic combat effects.

## Architecture & Rendering System

### Core Rendering Pipeline

The combat view renders in layers from back to front:

1. **Background layer** - Ceiling gradient (darkest at top)
2. **Wall layer** - Left and right stone walls with perspective (wider at bottom)
3. **Floor layer** - Cobblestone with perspective grid lines converging to center
4. **Detail layer** - Torches, chains, atmospheric elements
5. **Enemy layer** - Scaled ASCII sprite at center depth
6. **Effect layer** - Attack animations, particles, impacts
7. **UI overlay** - Damage numbers, status indicators

### Distance/Scale System

Enemy visual size represents combat state using a `combat_depth` value (0.0 to 1.0):

- **0.0** = Enemy far away (small, 3-5 lines tall)
- **0.5** = Medium distance (7-10 lines tall)
- **1.0** = Enemy very close (15-20 lines tall, intimidating)

**Depth Calculation:**
```rust
combat_depth = 0.5 + (player_hp_ratio - enemy_hp_ratio) * 0.3
// When player is losing: depth → 1.0 (enemy looms closer)
// When player is winning: depth → 0.0 (enemy retreats)
// Clamped to [0.2, 0.9] for visibility
```

### Character Density for Shading

ASCII density gradient for 3D depth perception:
```
Light to Dark: ` .·:;=+*#%@█`
```

- Characters farther from light sources: lighter chars (` .·:`)
- Characters closer to light/foreground: denser chars (`#%@█`)

## Enemy Sprite System & Scaling

### Enemy Sprite Templates

Each enemy type has a master ASCII sprite template (10 lines tall base):

**Example - Orc:**
```
      ╱╲
     ╱██╲
    │████│
   ╱██████╲
  │ ●    ● │
  │   ▼    │
  ╰────────╯
   ││    ││
   ╰╯    ╰╯
```

**Example - Drake:**
```
     ╱╲ ╱╲
    ╱  V  ╲
   ╱ ████ ╲
  ╱ ██████ ╲
  │ ◆    ◆ │
  │   ▼▼   │
  ╰─┬────┬─╯
    └────┘
```

### Procedural Variations

Base templates for each enemy suffix type:
- **Orc** - Brutish, wide stance, angular features
- **Troll** - Tall, hunched posture, heavy build
- **Drake** - Winged, serpentine, scaled
- **Crusher/Render** - Massive, intimidating, weapon-wielding
- **Beast/Fiend** - Quadruped or amorphous, feral
- **Horror/Terror** - Eldritch, unsettling, asymmetric
- **Maw** - Tooth/jaw focused, gaping mouth

### Dynamic Scaling Algorithm

```rust
fn scale_sprite(base_sprite: &str, depth: f64) -> Vec<String> {
    // Calculate target height
    let min_height = 3;
    let max_height = 20;
    let target_height = min_height + ((max_height - min_height) as f64 * depth) as usize;

    // Scale sprite
    let scaled = resize_ascii_art(base_sprite, target_height);

    // Apply depth shading
    apply_depth_shading(scaled, depth)
}

fn apply_depth_shading(sprite: Vec<String>, depth: f64) -> Vec<String> {
    let shading_map = if depth < 0.3 {
        // Far: light chars
        [('█', '#'), ('#', '+'), ('+', ':'), ('*', '.')]
    } else if depth < 0.7 {
        // Medium: medium chars
        [('█', '@'), ('@', '#'), ('#', '*')]
    } else {
        // Close: dark/bold chars (keep as-is or enhance)
        [('*', '#'), ('+', '@')]
    };

    // Replace characters according to map
    sprite.iter().map(|line| apply_char_map(line, &shading_map)).collect()
}
```

### Animation Frames

Each sprite has multiple states:
- **Idle** (2 frames): Slight breathing animation (size pulse ±1 char)
- **Hit** (1 frame): Distorted, characters scattered/shifted
- **Attack** (1 frame): Lunging forward pose
- **Death** (3 frames): Progressive collapse/fade

## Combat Effects & Animations

### Attack Effect Sequence

**Player Attack (1.5s total):**

**Frame 1 (0.0-0.2s): Weapon Launch**
```
Player weapon appears from bottom:
Sword:  ║═╗ sweeps upward in arc
Magic:  ✦*˚ particles spiral from bottom
```

**Frame 2 (0.2-0.4s): Impact**
```
Explosion at enemy hit point:
    *!@#$%
   !@#$%^&*
  @#$%^&*()
   #$%^&*!
    $%^&*

Enemy sprite flickers (characters invert momentarily)
```

**Frame 3 (0.4-0.8s): Damage Number**
```
Large ASCII number rises from enemy:
  ╔═════╗
  ║  23 ║  <- Damage value
  ╚═════╝

Normal: White text
Critical: Yellow, 2x size, with "CRITICAL!" above
```

**Frame 4 (0.8-1.5s): Aftermath**
```
Particles drift down: .·˚*
Enemy returns to idle animation
```

### Enemy Attack Sequence

**Enemy Attack (1.5s total):**

**Frame 1 (0.0-0.3s): Wind-up**
```
Enemy sprite scales up 10% (lunging forward)
Red glow around enemy
```

**Frame 2 (0.3-0.5s): Strike**
```
Screen shakes
Red slash effect sweeps from enemy toward camera:
  ════════
   ══════
    ════
```

**Frame 3 (0.5-1.0s): Impact & Damage**
```
Screen edges flash red
Damage number appears at bottom:
  ╔═════╗
  ║ -15 ║  <- Damage taken
  ╚═════╝
```

**Frame 4 (1.0-1.5s): Recovery**
```
Enemy scales back to normal
Red flash fades
```

### Screen Effects

**Screen Shake:**
- Shift entire combat view 1-2 chars randomly on heavy hits
- Duration: 0.2s
- Triggered: Crits, killing blows, player near death

**Flash Effects:**
- White flash on crit: Entire screen brightens for 0.1s
- Red flash on player hit: Edges glow red
- Green flash on regeneration: Healing effect

**Vignette Pulse:**
- When player HP < 33%: Edges darken progressively
- Pulsing effect (darker/lighter) matching heartbeat rhythm

**Damage Vignette:**
```
Player HP > 66%: Normal brightness
Player HP 33-66%: Slight darkening at edges using ░
Player HP < 33%: Heavy darkening using ▒▓, pulsing red
```

### Critical Hit Special Effects

When a critical hit occurs:

1. **Enemy Recoil**: Enemy sprite scales UP 20% momentarily (reeling back)
2. **Enhanced Damage Number**:
   ```
   ╔═══════════╗
   ║ CRITICAL! ║
   ╠═══════════╣
   ║    46     ║
   ╚═══════════╝
   ```
3. **Particle Cascade**: Extended particle effect, 2x duration
4. **Screen Impact**: Stronger shake + white flash
5. **Sound Effect Text**: "BOOM!" or "SLASH!" appears briefly

### Enemy Death Sequence

**Death Animation (2.5s):**

**Phase 1 (0.0-0.5s): Recoil**
```
Enemy sprite rapidly scales down (falling away)
Explosion of particles: *@#%!◆◇○●
```

**Phase 2 (0.5-1.5s): Collapse**
```
Sprite breaks apart into constituent characters
Characters drift down like ash: ░▒▓
Screen flashes white briefly
```

**Phase 3 (1.5-2.0s): Victory**
```
"VICTORY" banner appears:
╔═══════════════╗
║   VICTORY!    ║
╚═══════════════╝

XP gained notification below
```

**Phase 4 (2.0-2.5s): Transition**
```
Fade to white
Begin HP regeneration sequence
```

### HP Regeneration Animation

**Regeneration Display (2.5s):**

```
╔═══════════════════════════╗
║    REGENERATING HP...     ║
╠═══════════════════════════╣
║   ♥ ♥ ♥ ♥ ♥ ░ ░ ░ ░ ░   ║ <- Progress bar
║        68% (1.2s)          ║
╚═══════════════════════════╝

Green particles rising: ˚·.✦
Heartbeat pulse effect
```

## Dungeon Environment Rendering

### Stone Wall Perspective

Walls use perspective projection to create depth:

```
Left Wall Pattern:           Right Wall Pattern:
█▓▓▒▒░                            ░▒▒▓▓█
█▓#▓▒░                            ░▒▓#▓█
█▓▓▒░                              ░▒▓▓█
█▓▒░                                ░▒▓█
█▓░                                  ░▓█
█░                                    ░█
█                                      █

(Wider at bottom, narrower at top)
```

**Stone Texture Patterns:**
```
Variations rotated through for visual interest:
Pattern 1: █▓▒░
Pattern 2: ▓▓▒▒
Pattern 3: ##▒▒
Pattern 4: ▓#▓▒
```

### Floor Grid

Perspective cobblestone floor:

```
             ═
          ═══════
       ═══════════
    ═══════════════
  ═══════════════════
═══════════════════════

Horizontal lines: ═
Vertical lines (perspective): ║ (converging to center)
Stones: ▓░▒ scattered between grid
```

### Ceiling Gradient

Arched ceiling with atmospheric darkness:

```
Top (darkest):     ░░░░░░░░░░░
                   ▒▒▒▒▒▒▒▒▒▒▒
Middle:            ▓▓▓▓▓▓▓▓▓▓▓
                   ██████████
Bottom (lighter):  ▓▓▓▓▓▓▓▓▓▓▓
```

### Torch Details

Animated torches on walls:

**Torch Frames (cycles every 0.3s):**
```
Frame 1:    Frame 2:    Frame 3:
   ^           ~           *
  ^^^         ~~~         ***
   |           |           |
   |           |           |

Colors: Orange/Yellow glow
Glow radius: 3 chars around torch
```

**Torch Position:**
- Left wall: 1/4 down from top
- Right wall: 1/4 down from top
- Provides dynamic lighting effect on walls

### Atmospheric Details

**Chains:**
```
Hanging from ceiling (left side):
  ╔═╗
  ║ ║
  ╠═╣
  ║ ║
  ╚═╝
```

**Skulls:**
```
On floor corners (random):
  ▄▀▀▀▀▀▄
 █ ● ● █
 █  ▼   █
  ▀▄▄▄▄▀
```

**Cracks in walls:**
```
Random placement:
  ╱
 ╱
╱
```

These details add atmosphere without cluttering the view.

## Integration & Performance

### File Structure

```
src/ui/
  combat_scene_3d.rs    <- New 3D renderer
  combat_effects.rs     <- Attack animations & effects
  enemy_sprites.rs      <- Enemy ASCII art templates
  ascii_scaler.rs       <- Sprite scaling algorithms
  perspective.rs        <- 3D projection math
```

### Rendering Performance

**Target**: 10 FPS minimum (100ms per frame) for smooth animation

**Optimizations:**
1. **Pre-render static elements** - Walls, floor, ceiling cached per frame
2. **Sprite caching** - Store scaled sprites at common sizes
3. **Effect pooling** - Reuse particle character vectors
4. **Layer composition** - Only redraw changed layers
5. **String building** - Use `Vec<String>` not `String` concat

**Frame Budget (100ms):**
- Environment render: 30ms
- Enemy sprite scale/shade: 20ms
- Effects rendering: 30ms
- Terminal draw: 20ms

### State Management

```rust
pub struct CombatRenderer3D {
    // Cached elements
    ceiling: Vec<String>,
    walls_cache: HashMap<usize, Vec<String>>,
    floor_cache: Vec<String>,

    // Sprite data
    enemy_sprites: HashMap<String, EnemySprite>,

    // Animation state
    current_effects: Vec<ActiveEffect>,
    frame_counter: u32,

    // Torch animation
    torch_frame: usize,
    torch_timer: f64,
}

pub struct ActiveEffect {
    effect_type: EffectType,
    position: (u16, u16),
    lifetime: f64,
    max_lifetime: f64,
    frames: Vec<Vec<String>>,
}

pub enum EffectType {
    DamageNumber { value: u32, is_crit: bool },
    Impact,
    Particles { direction: (i8, i8) },
    Flash { color: Color },
}
```

### Integration with Game Loop

```rust
// In main.rs game_tick()
fn game_tick(game_state: &mut GameState, tick_counter: &mut u32) {
    // ... existing XP logic ...

    // Update combat (creates CombatEvent list)
    let combat_events = update_combat(game_state, delta_time);

    // Convert combat events to visual effects
    // (stored in game_state.combat_state.visual_effects)
    for event in combat_events {
        match event {
            CombatEvent::PlayerAttack { damage, was_crit } => {
                spawn_attack_effect(game_state, damage, was_crit);
            }
            CombatEvent::EnemyAttack { damage } => {
                spawn_enemy_attack_effect(game_state, damage);
            }
            CombatEvent::EnemyDied { xp_gained } => {
                spawn_death_effect(game_state, xp_gained);
            }
            // ...
        }
    }

    // Update visual effect timers
    update_visual_effects(game_state, delta_time);
}
```

### Configuration Options

User-facing settings for accessibility:

```rust
pub struct Combat3DConfig {
    /// Enable/disable 3D combat view (fallback to simple view)
    pub enabled: bool,

    /// Reduce particle effects (performance)
    pub reduced_effects: bool,

    /// Disable screen shake (accessibility)
    pub no_screen_shake: bool,

    /// Disable flashes (epilepsy safety)
    pub no_flashes: bool,

    /// Enemy sprite detail level (low/medium/high)
    pub sprite_detail: DetailLevel,
}
```

## Testing Strategy

### Unit Tests

1. **Sprite Scaling**
   - Test scaling from 3 to 20 lines
   - Verify aspect ratio preservation
   - Test edge cases (1 line sprite, 50 line sprite)

2. **Depth Shading**
   - Verify correct character substitution at each depth
   - Test boundary conditions (depth = 0.0, 1.0)

3. **Effect Timing**
   - Verify effect lifetimes expire correctly
   - Test multiple simultaneous effects

4. **Perspective Projection**
   - Test floor grid convergence
   - Verify wall scaling

### Visual Tests

1. **Manual Combat Playthrough**
   - Start combat, verify environment renders
   - Attack enemy, verify effects trigger
   - Take damage, verify screen effects
   - Kill enemy, verify death sequence
   - Test critical hits, verify enhanced effects

2. **Performance Test**
   - Spawn max effects, measure FPS
   - Verify 10 FPS minimum maintained

3. **Edge Cases**
   - Very low HP (< 5%), check vignette
   - Maximum size enemy, verify no overflow
   - Rapid succession attacks, check effect stacking

## Success Metrics

- ✅ Environment renders at 10+ FPS
- ✅ Enemy appears 3D with proper depth perception
- ✅ Attacks have satisfying visual feedback
- ✅ Combat feels more engaging than emoji version
- ✅ No performance regression during normal gameplay
- ✅ Accessible (can disable effects if needed)

## Future Enhancements

Potential additions after initial implementation:

1. **Environmental Variety** - Different dungeon themes per zone
2. **Boss Battles** - Special larger sprites, unique effects
3. **Weather Effects** - Rain, fog, snow in outdoor zones
4. **Dynamic Lighting** - Spells illuminate the dungeon
5. **Player Weapon Display** - Show player's weapon at bottom
6. **Combo Attacks** - Chain multiple hits with escalating effects

---

**Implementation Priority:** High
**Complexity:** Medium-High
**Estimated Tasks:** 8-10 implementation tasks
**Visual Impact:** Very High
