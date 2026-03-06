# Chrono Surge Mission Acceleration

**Date:** 2026-03-06
**Status:** Approved

## Summary

Chrono Surge should accelerate Deep mission timers. Each surge tick (100ms) subtracts 100ms from active mission timers, matching the existing "fast-forward game time" semantics. Missions that complete during a surge are moved to pending_results for player review. The surge summary includes a missions_completed counter.

## Design Decisions

- **1:1 acceleration**: each surge tick = 100ms off mission timers
- **Auto-resolve passed events**: check-in events whose timestamps fall into the past use their `auto_resolve_choice`, matching offline resolution behavior
- **Summary includes mission completions**: "Missions completed: N" shown alongside kills/levels/items
- **Completed missions go to pending_results**: player reviews results in the Deep overlay, preserving the narrative/loot reveal

## Implementation

### 1. Core Function: `accelerate_missions()`

New function in `src/deep/missions.rs`:

```rust
pub fn accelerate_missions(
    prestige: &mut DeepPrestige,
    persistent: &mut DeepPersistent,
    acceleration: Duration,
    rng: &mut impl Rng,
) -> u32  // returns count of missions that became time-elapsed
```

Behavior:
1. For each active mission in `prestige.active_missions`:
   - Subtract `acceleration` from `ends_at`
   - Subtract `acceleration` from all unresolved check-in event timestamps
   - Auto-resolve any events whose timestamps now fall in the past
2. Return count of missions whose `ends_at` is now in the past
3. Does NOT resolve completed missions — existing tick stage 13 handles that

### 2. Integration: Surge Batch Loop

In `src/main_helpers/chrono_surge.rs`, after the batch's game ticks run (after `surge.ticks_remaining -= 1`), call:

```rust
let missions_completed = accelerate_missions(
    &mut deep_state.prestige,
    &mut deep_state.persistent,
    Duration::milliseconds((batch as i64) * 100),
    &mut rng,
);
surge.missions_completed += missions_completed;
```

### 3. Surge Summary Extension

Add `missions_completed: u32` to:
- `ChronoSurgeState` in `src/stormglass/types.rs` (accumulator)
- `ChronoSurgeSummary` in `src/stormglass/types.rs` (display value)

Render in surge completion UI in `src/ui/stormglass_scene.rs`.

### 4. Mission Completion Flow

When `accelerate_missions()` shifts `ends_at` into the past, the existing tick stage 13 (running inside `game_tick_with_context` during the surge) detects `is_time_elapsed(Utc::now())` and moves the mission to `pending_results`. No new completion logic needed.

## Testing

### Unit tests (src/deep/missions.rs)
- `ends_at` shifts by correct duration
- Event timestamps shift in lockstep
- Past-due events get auto-resolved
- Missions with no events accelerate cleanly
- Multiple active missions all accelerated
- Already-completed missions unaffected

### Integration test (tests/)
- Surge batch with active Deep mission shifts `ends_at` by `batch * 100ms`
- `missions_completed` counter reflects missions finished during surge

## Files Changed

| File | Change |
|------|--------|
| `src/deep/missions.rs` | Add `accelerate_missions()` function |
| `src/main_helpers/chrono_surge.rs` | Call `accelerate_missions()` after each batch |
| `src/stormglass/types.rs` | Add `missions_completed` to `ChronoSurgeState` and `ChronoSurgeSummary` |
| `src/ui/stormglass_scene.rs` | Render missions_completed in surge summary |
| `src/deep/missions.rs` (tests) | Unit tests for acceleration |
| `tests/` | Integration test for surge + missions |
