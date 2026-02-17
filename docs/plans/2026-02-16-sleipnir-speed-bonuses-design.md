# Sleipnir Speed Bonuses Design

## Goal

Add two new bonuses to Sleipnir (Boots god item) that reinforce its "speed" fantasy: faster dungeon movement and faster fishing. Both use 50% multiplicative reduction, matching Swiftstrider's existing theme.

## New Bonuses

### Swiftfoot — 50% Dungeon Movement Speed

- Reduces both room exploration (2.5s → 1.25s) and backtracking (0.8s → 0.4s)
- Applied as: `interval * (1.0 - dungeon_speed_percent / 100.0)`
- Passed as explicit parameter into dungeon logic (same pattern as Haven bonuses)
- No existing dungeon speed bonuses — this is the first

### Nimble Hands — 50% Fishing Timer Reduction

- Multiplicative with Haven Garden reduction, applied after Haven
- Formula: `base_ticks * (1.0 - haven_reduction) * (1.0 - sleipnir_reduction)`
- Affects all 3 phases (casting, waiting, reeling)
- With max Garden (40%): 70% total reduction. Without Garden: 50% reduction
- Uses existing `apply_timer_reduction` pattern in fishing/logic.rs

## Updated Sleipnir Definition

Sleipnir's bonuses become:

| Bonus | Effect | Value |
|-------|--------|-------|
| Swiftstrider | HP regen delay reduction | 50% (existing) |
| Swiftfoot | Dungeon movement speed | 50% (new) |
| Nimble Hands | Fishing timer reduction | 50% (new) |

## Integration Points

| File | Change |
|------|--------|
| `god_items/types.rs` | Add `Swiftfoot` and `NimbleHands` to `GodItemBonus`, add helper functions |
| `dungeon/logic.rs` | Add `god_item_dungeon_speed_percent: f64` parameter to move interval calc |
| `fishing/logic.rs` | Add `god_item_fishing_reduction_percent: f64` parameter after Haven reduction |
| `core/tick.rs` | Wire new parameters from equipped item helpers into dungeon/fishing calls |
