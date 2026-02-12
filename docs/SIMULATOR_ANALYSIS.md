# Simulator vs Game Analysis

## Current State (Updated 2026-02-11)

The simulator module (`src/simulator/`) has been **fully removed**. All simulation logic was either folded into shared modules or deprecated during the game_tick refactor (PRs #135-#139).

The original analysis below is preserved for historical context, with status updates.

## ✅ Shared Code (Still Active)

These modules are used by the game directly. The simulator previously consumed them too, but the simulator no longer exists.

| Component | File | Notes |
|-----------|------|-------|
| Derived Stats | `character/derived_stats.rs` | `DerivedStats::calculate_derived_stats()` — formulas inline |
| Enemy Generation | `combat/types.rs` | `generate_enemy_for_current_zone()`, `generate_subzone_boss()` |
| Zone Data | `zones/data.rs` | `get_zone()`, `get_all_zones()` |
| Balance Constants | `core/constants.rs` | HP, damage, XP formulas (formerly `core/balance.rs`) |
| Item Generation | `items/generation.rs` | `generate_item()` |
| Item Scoring | `items/scoring.rs` | `auto_equip_if_better()` |
| Drop Rates | `items/drops.rs` | `drop_chance_for_prestige()`, `ilvl_for_zone()` |

## Completed Refactoring

### Phase 1-3: All Done ✅

The original plan called for shared traits and combat math extraction. This was completed and then further consolidated:

- `core/combat_math.rs` — Created, then removed when combat logic was folded into `combat/logic.rs` and `core/tick.rs`
- `core/progression.rs` — Created with `Progression` trait, then removed during tick refactor
- `core/balance.rs` — Constants moved to `core/constants.rs`; formulas inlined in `character/derived_stats.rs`
- `simulator/` directory — Entirely removed

### What Replaced the Simulator

The game_tick refactor (PRs #135-#139) consolidated game logic into `core/tick.rs`, making a separate simulator unnecessary. Balance testing now relies on the 1,263+ integration tests in `tests/`.

## Remaining Work

- [ ] `ZoneProgression` in `zones/progression.rs` could still benefit from a shared progression trait, but this is low priority since the simulator no longer exists as a consumer.

## Summary

| Original Item | Status |
|---------------|--------|
| Shared code (8 components) | ✅ Still shared, paths updated |
| Duplicated combat logic | ✅ Eliminated (simulator removed) |
| Duplicated XP/prestige logic | ✅ Eliminated (simulator removed) |
| Progression trait | ✅ Created then removed (no longer needed) |
| Combat math extraction | ✅ Created then consolidated into tick.rs |
| Test count | 1,263 `#[test]` annotations (up from original 962) |
