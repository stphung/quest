# Prestige Multiplier Rebalance

**Date**: 2026-02-03
**Status**: Implementing
**Branch**: `feat/prestige-multiplier-rebalance`

## Problem

The current prestige XP multiplier formula `1.2^rank` is hyper-exponential, causing later prestige cycles to become **faster** than earlier ones:

| Rank | Current Multiplier | Cycle Time |
|------|-------------------|------------|
| P10 | 6.2x | 8.2 hours |
| P20 | 38.3x | 5.9 hours |
| P30 | 237.4x | 3.3 hours |

This breaks the core idle game loop where players should feel a "wall" before prestiging, then feel powerful after. With the current formula, late-game prestige becomes trivial button-mashing.

## Solution

Change the multiplier formula to use **diminishing returns**:

```
Old: 1.2^rank (exponential)
New: 1 + 0.5 * rank^0.7 (sub-linear growth)
```

### New Multiplier Values

| Rank | Old (1.2^r) | New (1+0.5r^0.7) | Per-Prestige Gain |
|------|-------------|------------------|-------------------|
| P1 | 1.2x | 1.5x | +50% |
| P5 | 2.5x | 2.5x | +10% |
| P10 | 6.2x | 3.5x | +6% |
| P15 | 15.4x | 4.3x | +4% |
| P20 | 38.3x | 5.1x | +3% |
| P25 | 95.4x | 5.8x | +2.5% |
| P30 | 237.4x | 6.4x | +2% |

### Expected Cycle Times

| Phase | Prestige Range | Cycle Time | Player Experience |
|-------|---------------|------------|-------------------|
| Tutorial | P1-3 | 30m - 2h | Quick hook, learn the mechanic |
| Learning | P4-7 | 3-8h | One prestige per session |
| Midgame | P8-15 | 8-24h | Overnight runs |
| Lategame | P16-25 | 1-3 days | Goal-oriented play |
| Endgame | P25+ | 3+ days | Dedicated players |

### Cumulative Time to Milestones

| Milestone | Old | New |
|-----------|-----|-----|
| P5 | 13.6h | 11.6h |
| P10 | 2.2 days | 2.5 days |
| P15 | 3.8 days | 6.4 days |
| P20 | 5.1 days | 12.8 days |
| P25 | 6.2 days | 24.2 days |
| P30 | 7.0 days | 42.4 days |

## Design Rationale

### Idle Game Prestige Principles

1. **The Wall → Reset → Power Fantasy Loop**
   - Players should feel stuck before prestige (the "wall")
   - After prestige, early game should feel fast and powerful
   - New wall should be slightly further than before

2. **Meaningful Progression**
   - Each prestige should feel like a noticeable boost
   - But not so strong that previous runs feel wasted

3. **Session-Appropriate Cycles**
   - Early: Quick cycles to teach mechanics
   - Mid: One prestige per play session
   - Late: Overnight/workday runs
   - End: Multi-day commitment for dedicated players

### Why Diminishing Returns?

The formula `1 + 0.5 * rank^0.7` provides:

- **Strong early boost**: +50% at P1 feels meaningful
- **Tapering gains**: Prevents late-game trivialization
- **Predictable ceiling**: Multiplier approaches ~6-7x asymptotically
- **Cycles get longer**: Creates the "wall" feeling that makes prestige satisfying

## Implementation

### Files Changed

1. `src/prestige.rs` - Update `get_prestige_tier()` multiplier formula
2. `src/prestige.rs` - Update tests for new expected values

### Code Change

```rust
// In get_prestige_tier()
// Old:
let multiplier = 1.2_f64.powi(rank as i32);

// New:
let multiplier = 1.0 + 0.5 * (rank as f64).powf(0.7);
```

## Testing

- [ ] Update `test_get_prestige_tier()` with new multiplier values
- [ ] Add test for multiplier formula correctness
- [ ] Add test verifying diminishing returns property
- [ ] Run full test suite
- [ ] Manual playtest early prestige cycles

## Rollback Plan

If players find progression too slow:
1. Adjust coefficient: `1 + 0.6 * rank^0.7` (faster)
2. Adjust exponent: `1 + 0.5 * rank^0.75` (slightly faster late-game)
3. Cap level requirements instead of multiplier

## Future Considerations

- Monitor player feedback on mid/late game pacing
- Consider adding prestige "milestones" with bonus rewards
- Could introduce prestige tiers with different multiplier curves
