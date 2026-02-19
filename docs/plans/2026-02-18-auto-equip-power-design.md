# Auto-Equip: Use Intrinsic Power Score

**Date:** 2026-02-18
**Status:** Approved

## Problem

The auto-equip system uses `score_item()`, a character-dependent weighted score that favors attributes matching the character's current distribution. This causes lower-power items to replace higher-power ones when their attributes happen to align with the character's build.

Players see the intrinsic power score (displayed as lightning bolt in the UI) and expect the system to equip the strongest item. When a weaker item gets equipped because of hidden specialization weighting, it feels like a bug.

## Solution

Replace `score_item()` with `item.power()` in `auto_equip_if_better()`. The `power()` method is character-independent, uses equal attribute weights plus weighted affix values, and is already displayed in the UI.

Remove `score_item()` and `calculate_attribute_weights()` as dead code. Keep `affix_power_weight()` (still used by `power()`).

## Changes

- `src/items/scoring.rs` — Remove `score_item()`, `calculate_attribute_weights()`, update `auto_equip_if_better()` to compare `power()`, update tests
- `src/items/CLAUDE.md` — Update auto-equip scoring docs
- `CLAUDE.md` — Update auto-equip description

## Trade-offs

- **Lost**: Attribute specialization (DEX character no longer prefers DEX items). Since attributes are assigned randomly on level-up, this was solving a problem that doesn't meaningfully exist.
- **Gained**: Predictable equip behavior matching what the player sees in the UI. Simpler code.
