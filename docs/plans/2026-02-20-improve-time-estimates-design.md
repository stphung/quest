# Improve Time Estimates (Issue #313)

## Problem

The XP rate and ETA calculations become inaccurate due to:

1. **Fishing zeros out XP rate**: While fishing, `xp_this_second` stays 0 but still gets sampled every second, dragging the rolling average down and making level/prestige ETAs spike wildly.
2. **Dungeon XP bursts**: Boss kills and treasure rooms produce large XP chunks followed by zeros while exploring, making the rate volatile.
3. **No activity awareness**: The 5-minute rolling window samples every second regardless of whether the player is earning XP through combat.

## Solution: Combat-Only XP Rate Sampling

Only sample XP rate during seconds where combat XP was actually earned. Extend the window from 5 minutes (300 samples) to 15 minutes (900 combat-seconds) to smooth dungeon burst volatility.

### Core Change: `GameState` (`game_state.rs`)

Add a transient flag:
```rust
#[serde(skip)]
pub combat_seconds_this_tick: bool,
```

### Sampling Logic: `tick.rs` + `tick_stages.rs`

At second boundaries, only push samples when combat was active:
```rust
if state.combat_seconds_this_tick {
    state.xp_rate_samples.push_back(state.xp_this_second);
}
state.xp_this_second = 0;
state.combat_seconds_this_tick = false;
if state.xp_rate_samples.len() > XP_RATE_WINDOW_SECONDS {
    state.xp_rate_samples.pop_front();
}
```

### Setting the Flag: `xp.rs`

In `apply_tick_xp()`, set `state.combat_seconds_this_tick = true` when XP is accumulated.

### Constants: `constants.rs`

```rust
pub const XP_RATE_WINDOW_SECONDS: usize = 900; // 15 min of combat time
```

### No Changes Needed

- `xp_per_hour()` — already computes average from samples; now samples are combat-only
- `stats_panel.rs` / `stats_prestige.rs` — already use `xp_per_hour()` for ETA display

### What This Fixes

| Problem | Fix |
|---------|-----|
| Fishing zeros dilute rate | No samples pushed while fishing |
| Dungeon XP bursts | 15-min window smooths spikes over more data |
| Minigame/menu idle time | No samples pushed when not in combat |

### Testing

- Unit: `xp_rate_samples` only grows when `combat_seconds_this_tick` is true
- Unit: fishing ticks don't add samples
- Unit: 900-sample cap enforced
- Integration: mixed combat/fishing session produces stable rate
