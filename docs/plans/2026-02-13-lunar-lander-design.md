# Lunar Lander Challenge Design

## Overview

A classic 1979 Atari-style Lunar Lander challenge. Real-time action (~60 FPS) where the player controls a lander descending toward terrain with a single landing pad. Fits alongside Flappy Bird and Snake as the third real-time action challenge.

## Core Mechanics

- **Controls**: Left/Right arrow to rotate lander, Up arrow (or Space) to fire main thruster
- **Physics**: Gravity pulls lander down each tick. Thrust applies force in the direction the lander faces. Velocity and position update per tick (2D vectors: vx, vy, angle)
- **Fuel**: Finite supply depletes when thrusting. When empty, gravity takes over
- **Win condition**: Touch the landing pad with vertical velocity below threshold AND near-level angle
- **Loss conditions**: Landing too fast, landing too tilted, hitting terrain outside the pad, running out of fuel and crashing

## Difficulty Tiers

| Tier | Fuel | Gravity | Pad Width | Terrain |
|------|------|---------|-----------|---------|
| Novice | 100% | 0.5x | Wide | Gentle rolling |
| Apprentice | 80% | 0.75x | Medium | Moderate bumps |
| Journeyman | 60% | 1.0x | Small | Jagged peaks |
| Master | 40% | 1.25x | Tiny | Very jagged, deep valleys |

## ASCII Rendering & UI

- **Lander sprite**: 3-5 chars wide, ~5 rotation angles (hard left, left, straight, right, hard right). Thrust flame below when firing
- **Terrain**: Procedural jagged polyline across the bottom. One flat segment = landing pad (green). Drawn with block characters
- **HUD**: Fuel gauge bar, altitude, velocity (vx/vy), angle indicator in info panel
- **Layout**: Uses `create_game_layout` from `game_common.rs`. Main area = sky + lander + terrain. Info panel = instruments. Status bar = controls
- **Colors**: Border `Color::LightBlue` (space theme). Pad = `Color::Green`. Lander = `Color::White`. Flame = `Color::Yellow`/`Color::Red`. Terrain = `Color::DarkGray`
- **Game over**: Standard `render_game_over_overlay` -- "Landed!" for win, "Crashed!" for loss. Standard double-Esc forfeit

## File Structure

```
src/challenges/lander/
  mod.rs      -- public exports
  types.rs    -- LanderDifficulty, LanderResult, LanderGame, terrain types
  logic.rs    -- physics tick, input processing, terrain generation, collision detection
```

```
src/ui/lander_scene.rs  -- terrain rendering, lander sprite, HUD instruments
```

## Integration Points

1. `src/challenges/mod.rs` -- add `ActiveMinigame::Lander(LanderGame)` variant
2. `src/challenges/menu.rs` -- add `ChallengeType::Lander`, discovery weight ~20, `DifficultyInfo` impl, `create_challenge` / `accept_selected_challenge` wiring
3. `src/input.rs` -- add `ActiveMinigame::Lander` match arm in `handle_minigame()`
4. `src/ui/mod.rs` -- add rendering dispatch for Lander scene
5. `src/utils/debug_menu.rs` -- add "Trigger Lander Challenge" option
6. `src/core/tick.rs` -- add tick dispatch for Lander game (like Flappy/Snake)
7. Achievements -- emit `MinigameWinInfo { game_type: "lander", difficulty }` on win

## Rewards

Same structure as other action challenges. Prestige ranks per difficulty tier following `ChallengeReward` convention.
