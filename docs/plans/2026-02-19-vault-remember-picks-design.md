# Vault Remember Last Picks — Design

**Date:** 2026-02-19
**Status:** Implemented

## Problem

The vault selection UI during prestige requires manually toggling each slot every time. With up to 5 vault slots, this is tedious when you typically want to keep the same items across prestiges.

## Solution

Remember the player's last vault selections and pre-populate them next time.

### Behavior

- **First vault use:** Nothing pre-selected (current behavior)
- **Subsequent uses:** Pre-selects the same slots chosen last prestige
- **Empty slot handling:** If a remembered slot has no item, it's skipped (no backfill)
- **Vault tier changes:** If vault capacity changed (e.g. upgraded T1→T2), selections are truncated to current capacity
- **Manual toggle:** Still works — player can adjust pre-selections before confirming
- **Backward compatibility:** Old saves without the field default to empty (no pre-selection)

## Changes

### 1. `src/haven/types.rs` — Haven struct

Added `last_vault_selections: Vec<EquipmentSlot>` with `#[serde(default)]` for backward compatibility.

### 2. `src/input/prestige_input.rs` — Vault input handling

- `handle_vault_selection()`: Changed `haven` from `&Haven` to `&mut Haven`. On prestige confirmation, saves `selected_slots` to `haven.last_vault_selections`.
- `handle_prestige_confirm()`: When opening vault, pre-populates `selected_slots` from `haven.last_vault_selections`, filtered to slots with items and truncated to current vault capacity.

### Tests

- `test_last_vault_selections_default_empty` — New Haven starts with empty selections
- `test_last_vault_selections_persists_through_serde` — Roundtrip serialization works
- `test_last_vault_selections_missing_in_old_save` — Old saves gracefully default to empty
