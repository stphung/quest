# Ticker SLO-Aware Speed

**Date:** 2026-02-17
**Status:** Approved

## Problem

The scrolling ticker falls 20+ seconds behind during fast combat (high prestige + Sleipnir). The max scroll speed (4x base = 16 chars/sec) cannot keep up with event generation rate (~67 chars/sec with Sleipnir's 100% attack speed). Level-up messages, fishing results, and item drops appear long after they occur.

## Solution

Three changes in `src/core/game_state.rs`:

1. **`TICKER_MAX_SPEED_MULT`: 4.0 -> 8.0** — doubles max scroll speed to 32 chars/sec
2. **`TICKER_SPEED_LERP`: 0.08 -> 0.2** — ramps to target speed 2.5x faster
3. **SLO-aware target speed formula** — `target = max(base, debt / SLO_TICKS)` capped at max. `SLO_TICKS = 50` (5 seconds). Directly ties speed to "what's needed to display this entry within 5 seconds."

## Expected Behavior

- Ticker lag drops from 20+ seconds to ~5-8 seconds during sustained Sleipnir combat
- Catches up fully during natural pauses (HP regen, zone transitions)
- Normal (non-Sleipnir) gameplay stays real-time
- No events are dropped — completeness preserved

## Alternatives Considered

- **Dynamic gap compression**: Reduce ENTRY_GAP from 3 to 1 under load. Rejected: more complexity, cramped visuals.
- **Born_at capping**: Hard-cap queue depth. Rejected: causes entry overlap, fundamentally changes spacing model.
