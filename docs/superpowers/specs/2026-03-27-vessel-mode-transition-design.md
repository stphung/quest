# Vessel Mode Transition & Basic Voyage Shell

**Parent spec:** `docs/superpowers/specs/2026-03-27-the-vessel-design.md`
**Sub-project:** 2 of 7
**Depends on:** Sub-project 1 (Launch Gate)

## Overview

After the player confirms launch in the Vessel overlay, a 5-beat narrative transition plays where the old UI visually transforms into the Act 2 UI. Then the basic voyage shell appears: two-column layout with ship stats on the left and void view on the right. The ship moves, fuel drains, hull exists, and the ticker shows voyage events. No combat, rooms, crew, or events yet — just the skeleton.

## Launch Transition: 5-Beat Narrative Sequence

Each beat advances with Enter. The UI visually transforms as the story progresses.

### Beat 1: Farewell

The normal game UI is still visible but dimming (darken all colors by ~50%). Text overlay centered:

```
The Origin Thread has spoken.
This branch of Yggdrasil is dying.
Everything you built was preparation for this moment.

                         [Enter]
```

### Beat 2: Unweaving

The stats panel and combat scene dissolve — characters scatter/fall (random positions each frame, like static noise). The left panel disintegrates first.

```
The Loom unweaves itself.
Twenty-eight patterns unspool into something new.
Woven fate becomes woven hull.

                         [Enter]
```

### Beat 3: Construction

The scattered characters reform into a ship shape in the center. ASCII art builds itself (reveal line by line or character by character).

```
Reality folds. The Deep's roots become a keel.
Haven's stones become ballast. The forge becomes an engine.
A vessel takes shape from everything you were.

                         [Enter]
```

### Beat 4: Launch

Ship art complete and centered. Stars begin appearing. A bright line extends rightward toward the destination beacon.

```
A signal from 10,000 light-years away.
A living branch. The last hope of a dying tree.

The Vessel launches into the void.

                         [Enter]
```

### Beat 5: Void

Stars streak horizontally (speed lines). Ship is small, centered. Screen is mostly dark. This is the final beat before the Act 2 UI appears.

```
         ·  ·
   ·          ·    ·
·  ·    ╱═══╲
       ╱ ◆◆◆ ╲    ·
·     ╱═══════╲        ·
     ═══════════    ·
·        ║║║║          ·
     ·        ·  ·

                         [Enter]
```

After beat 5, transition to the Act 2 UI.

## Implementation Notes for Transition

The transition is a sequence of full-screen renders, not an overlay. While the transition is playing:
- Game tick is paused (or runs but input is blocked)
- Each beat renders a full frame
- Enter advances to next beat
- No Esc/cancel — launch was already confirmed

The visual effects (dimming, dissolving, reforming) can be simple:
- Beat 1: render normal UI with darkened colors
- Beat 2: fill screen with random characters from a sparse set, fading
- Beat 3: reveal ship ASCII art progressively
- Beat 4: ship art + star field + beacon line
- Beat 5: star streaks + small ship

These can be refined later. For initial implementation, static text screens per beat are sufficient — animation can be layered on.

## Act 2 UI: Two-Column Voyage Shell

Same structure as Act 1 (stats left, activity right, ticker bottom) but with new content.

### Left Panel: Ship Stats

```
┌─ The Vessel ──────────────┐
│ Distance: 0.3 / 10,000 ly │
│ Speed:    0.1 ly/day       │
│                            │
│ ── STATS ──                │
│ Firepower:   12            │
│ Hull:        80/80         │
│ Engines:     1             │
│ Sensors:     5             │
│                            │
│ ── RESOURCES ──            │
│ Fuel:    ████████░░ 80%    │
│ Hull:    ██████████ 100%   │
│ Supplies:███████░░░ 70%    │
│                            │
│ ── CREW ──                 │
│ 0/8 aboard                 │
│ (none yet)                 │
│                            │
│ ── ROOMS ──                │
│ 2/20 slots                 │
│ Reactor     Lv 1           │
│ Engines     Lv 1           │
│                            │
│ Transmissions: 1,100 PR/d  │
└────────────────────────────┘
```

Shows at a glance: how far you've gone, how fast, your stats, resource bars, crew roster summary, built rooms, and supply line income.

### Right Panel: Void View

```
┌─ The Void ──────────────────────────────┐
│                                         │
│  · ·    ·        ·    ·                 │
│     ·        ╱═══╲        ·    ·        │
│  ·          ╱ ◆◆◆ ╲                    │
│            ═══════════    ·             │
│  ·    ·       ║║║║      ·          ·   │
│         ·          ·         ·          │
│  ·           ·          ·               │
│                                         │
│  (The void stretches ahead)             │
│                                         │
│                                         │
├─────────────────────────────────────────┤
│  ✦ Launched from the dying branch       │
│  ✦ Fuel harvested: +2 from void matter  │
└─────────────────────────────────────────┘
```

The void view has:
- Star field background (animated, parallax with speed)
- Ship ASCII art centered
- Combat area (below ship, empty in sub-project 2)
- Lower section: recent events (like the existing ticker/combat log area)

### Bottom: Ticker

Same scrolling ticker component, now showing voyage events instead of loot drops.

### Footer

```
[R] Rooms  [C] Crew  [E] Events  [Esc] Menu
```

These hotkeys are stubs in sub-project 2 — they'll be wired in later sub-projects.

## Basic Voyage Tick Model

The voyage runs on the same 100ms tick as Act 1. Per tick:

1. **Distance increment:** `distance += speed_ly_per_day / (10 * 86400)` (10 ticks/sec, 86400 sec/day)
2. **Fuel drain:** `fuel -= fuel_drain_per_tick` (proportional to speed)
3. **Supply drain:** `supplies -= supply_drain_per_tick` (proportional to crew count; 0 crew = 0 drain)
4. **Drift check:** if fuel <= 0 or hull <= 0, enter drift state

Starting values:
- Distance: 0.0 ly
- Speed: 0.1 ly/day
- Fuel: 100% (1,000 units; drain rate ~1/day at base speed = ~1,000 day supply)
- Hull: 100% (100 HP; no drain without combat)
- Supplies: 100% (500 units; no drain without crew)

These numbers are placeholders — later sub-projects will tune them with combat and resource harvesting in the loop.

## Drift State

When fuel hits 0:
- Speed drops to 0
- Ship stops moving
- "DRIFT" indicator appears on the void view
- Transmissions from old world slowly restore fuel (1 unit per transmission PR, converted via base rate)
- Player can't do much except wait for recovery

When hull hits 0:
- Same drift state
- Transmissions slowly restore hull

Recovery from drift takes time but is automatic. No player death.

## Vessel State Model

New struct for Act 2 state:

```rust
pub struct VesselState {
    pub distance_ly: f64,        // Current distance traveled
    pub speed_ly_per_day: f64,   // Current speed
    pub fuel: f64,               // 0.0 - 1000.0
    pub fuel_capacity: f64,      // Max fuel
    pub hull: f64,               // 0.0 - 100.0
    pub hull_max: f64,           // Max hull
    pub supplies: f64,           // 0.0 - 500.0
    pub supplies_capacity: f64,  // Max supplies
    pub drifting: bool,          // In drift state
    pub ship_level: u32,         // XP-based level
    pub ship_xp: u64,            // Accumulated XP
    // Room and crew fields added in later sub-projects
}
```

Persisted to `~/.quest/vessel.json` (separate file, like Deep and Loom).

## Offline Progression

When the game is closed and reopened, calculate elapsed real time and simulate:
- Distance gained (speed × elapsed days, capped by fuel)
- Fuel consumed
- Supply consumed
- Drift if resources ran out mid-offline

Same pattern as existing offline XP and Deep mission resolution.

## Files

| File | Change |
|------|--------|
| `src/vessel/mod.rs` | New module: public API re-exports |
| `src/vessel/types.rs` | New: VesselState, VesselUiState |
| `src/vessel/tick.rs` | New: voyage tick (distance, fuel, drift) |
| `src/vessel/persistence.rs` | New: save/load vessel.json |
| `src/vessel/transition.rs` | New: launch transition beat state machine |
| `src/ui/vessel_scene.rs` | New: Act 2 main render (two-column layout) |
| `src/ui/vessel_transition.rs` | New: transition beat rendering |
| `src/input/vessel_input.rs` | New: Act 2 input handling |
| `src/main.rs` | Wire vessel tick, render, save/load, mode switching |
| `src/core/game_state.rs` | Add `vessel_launched` check for mode routing |
| `src/lib.rs` | Register vessel module |

## Testing

- Unit test: distance increments correctly per tick at given speed
- Unit test: fuel drains proportional to speed
- Unit test: drift triggers when fuel hits 0
- Unit test: drift triggers when hull hits 0
- Unit test: offline progression calculates correct distance/fuel
- Unit test: transition beat state machine advances correctly
- Unit test: serde round-trip for VesselState
- Snapshot tests: voyage shell layout and each transition beat (full-frame TUI snapshot infra from #623/#624)
