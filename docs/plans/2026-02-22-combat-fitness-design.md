# Combat Fitness System — Death Loop Prevention

**Issue:** #320 — Expanse death or monster death should not infinite loop
**Date:** 2026-02-22

## Problem

Players can get stuck in an infinite death loop when fighting regular mobs they can't beat. Unlike bosses (which have an enrage timer and reset mechanics), regular mob deaths just reset the enemy to full HP and restart combat immediately. An underpowered player will die to the same mob forever with no escape.

A secondary problem: a player who barely survives but takes 30+ seconds to kill a single regular mob is also effectively stuck — not dying, but not progressing meaningfully.

## Solution: Combat Fitness System

Two detection mechanisms that trigger an automatic retreat to the last safe zone:

### Detection Triggers

1. **Death loop** (`consecutive_deaths >= 3`): Three consecutive deaths to regular mobs without killing anything triggers retreat.
2. **Fight timeout** (`MOB_FIGHT_TIMEOUT = 30s`): Any single regular mob fight exceeding 30 seconds triggers immediate retreat. Analogous to the existing 60-second boss enrage timer but for regular mobs.

Normal mob fights last 3-8 seconds, so 30 seconds is a generous threshold that only fires when the player is clearly outmatched.

### Retreat Behavior

When either trigger fires:

1. Find the **last safe zone**: the highest `zone_id` in `defeated_bosses` on `ZoneProgression`. If no bosses defeated, fall back to Zone 1, Subzone 1.
2. Call `travel_to(safe_zone_id, 1)` to move the player to subzone 1 of that zone.
3. Clear `current_enemy`, reset `kills_in_subzone` to 0, reset combat timers.
4. Reset `consecutive_deaths` to 0.
5. Emit `TickEvent::CombatRetreat { zone_name }` for the combat log: *"You were overwhelmed and retreated to [zone name]..."*

### State Changes

| Location | Field | Type | Notes |
|----------|-------|------|-------|
| `GameState` | `consecutive_deaths` | `u32` | Transient (`#[serde(skip)]`), reset on kill |
| `CombatState` | `current_fight_elapsed` | `f64` | Transient, reset when new enemy spawns |

### Constants

| Constant | Value | Purpose |
|----------|-------|---------|
| `DEATH_LOOP_THRESHOLD` | 3 | Consecutive mob deaths before retreat |
| `MOB_FIGHT_TIMEOUT` | 30.0 | Seconds before forced retreat vs regular mob |

### What This Does NOT Change

- **Boss deaths**: Existing mechanics stay (zone boss resets to subzone 1, subzone boss requires 5 kills to retry, 60s enrage timer).
- **Dungeon deaths**: Existing mechanic stays (exit dungeon, no penalty).
- **Prestige**: No prestige loss on any death (unchanged).

## Code Changes

| File | Change |
|------|--------|
| `core/constants.rs` | Add `DEATH_LOOP_THRESHOLD` and `MOB_FIGHT_TIMEOUT` constants |
| `core/game_state.rs` | Add `consecutive_deaths: u32` field (`#[serde(skip)]`) |
| `combat/types.rs` | Add `current_fight_elapsed: f64` to `CombatState` (`#[serde(skip)]`) |
| `combat/orchestration.rs` | Increment `current_fight_elapsed` by `delta_time`; check timeout for non-boss mobs (similar to boss enrage check) |
| `combat/enemy_attack.rs` | On regular mob death: increment `consecutive_deaths`, check `>= DEATH_LOOP_THRESHOLD`, trigger retreat |
| `combat/damage.rs` | On enemy kill: reset `consecutive_deaths` to 0 |
| `core/tick_types.rs` | Add `TickEvent::CombatRetreat { zone_name: String }` variant |
| `tick_events.rs` | Map `CombatRetreat` to combat log message |

## Edge Cases

- **Zone 1, Subzone 1**: If the player somehow dies here with no defeated bosses, they stay put (already at the safest location). The retreat still clears the enemy and resets timers, breaking the loop.
- **The Expanse (Zone 11)**: Retreat sends the player back to their highest cleared zone (likely Zone 10). This is intentional — The Expanse is an endgame wall.
- **Offline progression**: Not affected. Offline XP doesn't involve combat.
- **Dungeon/fishing/challenges**: Not affected. These have their own death/exit mechanics.
