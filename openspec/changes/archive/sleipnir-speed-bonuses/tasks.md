> Backported implementation plan (completed — this work shipped).

## 2026-02-16-sleipnir-speed-bonuses-plan.md

# Sleipnir Speed Bonuses Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add Swiftfoot (50% dungeon speed) and Nimble Hands (50% fishing speed) bonuses to Sleipnir.

**Architecture:** `GodItemDefinition.bonus` becomes `bonuses: Vec<GodItemBonus>` so Sleipnir can have 3 bonuses. New parameters are injected into `update_dungeon()` and `tick_fishing_with_haven_result()` following the existing Haven bonus pattern. Fishing reduction is multiplicative with Haven's Garden reduction.

**Tech Stack:** Rust, serde (backward-compatible with `#[serde(default)]`)

---

## Task 1: Update god_items/types.rs — new variants, bonus → bonuses

**Files:**
- Modify: `src/god_items/types.rs`

### What to do

1. Add two new variants to `GodItemBonus`:

```rust
pub enum GodItemBonus {
    OfflineXpMultiplier { multiplier: f64 },
    Swiftstrider { regen_reduction_percent: f64 },
    PrestigeMastery { prestige_xp_multiplier: f64 },
    /// Reduces dungeon room movement timers (multiplicative).
    Swiftfoot { dungeon_speed_percent: f64 },
    /// Reduces fishing phase timers (multiplicative, stacks with Haven).
    NimbleHands { fishing_reduction_percent: f64 },
}
```

2. Change `GodItemDefinition.bonus: GodItemBonus` to `bonuses: Vec<GodItemBonus>`:

```rust
pub struct GodItemDefinition {
    // ... existing fields ...
    pub bonuses: Vec<GodItemBonus>,  // was: pub bonus: GodItemBonus,
}
```

3. Update all 3 definitions. Asprika and Megingjord get `vec![single_bonus]`. Sleipnir gets 3:

```rust
// sleipnir_definition():
bonuses: vec![
    GodItemBonus::Swiftstrider { regen_reduction_percent: 50.0 },
    GodItemBonus::Swiftfoot { dungeon_speed_percent: 50.0 },
    GodItemBonus::NimbleHands { fishing_reduction_percent: 50.0 },
],
```

4. Update ALL existing helper functions to iterate `def.bonuses` instead of matching `def.bonus`:

```rust
// Example pattern — apply to ALL 4 existing helpers:
pub fn equipped_god_item_regen_reduction_percent(equipment: &crate::items::Equipment) -> f64 {
    for item in equipment.iter_equipped() {
        if let Some(id) = item.god_item_id {
            let def = get_god_item_definition(id);
            for bonus in &def.bonuses {
                if let GodItemBonus::Swiftstrider { regen_reduction_percent } = bonus {
                    return *regen_reduction_percent;
                }
            }
        }
    }
    0.0
}
```

Apply the same `for bonus in &def.bonuses` pattern to:
- `equipped_god_item_offline_xp_percent()` (match `OfflineXpMultiplier`)
- `equipped_god_item_regen_reduction_percent()` (match `Swiftstrider`)
- `equipped_god_item_prestige_xp_multiplier()` (match `PrestigeMastery`, default 1.0)

5. Add two new helper functions:

```rust
/// Returns the god item dungeon movement speed bonus percent (0.0 if none).
pub fn equipped_god_item_dungeon_speed_percent(equipment: &crate::items::Equipment) -> f64 {
    for item in equipment.iter_equipped() {
        if let Some(id) = item.god_item_id {
            let def = get_god_item_definition(id);
            for bonus in &def.bonuses {
                if let GodItemBonus::Swiftfoot { dungeon_speed_percent } = bonus {
                    return *dungeon_speed_percent;
                }
            }
        }
    }
    0.0
}

/// Returns the god item fishing timer reduction percent (0.0 if none).
pub fn equipped_god_item_fishing_reduction_percent(equipment: &crate::items::Equipment) -> f64 {
    for item in equipment.iter_equipped() {
        if let Some(id) = item.god_item_id {
            let def = get_god_item_definition(id);
            for bonus in &def.bonuses {
                if let GodItemBonus::NimbleHands { fishing_reduction_percent } = bonus {
                    return *fishing_reduction_percent;
                }
            }
        }
    }
    0.0
}
```

6. Update existing tests that reference `def.bonus` (singular) to use `def.bonuses`:

- `test_asprika_has_offline_xp_bonus`: match against `def.bonuses` instead of `def.bonus`
- `test_sleipnir_has_swiftstrider_bonus`: match against `def.bonuses` instead of `def.bonus`
- `test_megingjord_has_prestige_mastery_bonus`: match against `def.bonuses` instead of `def.bonus`

Pattern for updating these tests:
```rust
// Before:
match def.bonus {
    GodItemBonus::Swiftstrider { regen_reduction_percent } => { ... }
    _ => panic!("Expected Swiftstrider bonus"),
}

// After:
let has_swiftstrider = def.bonuses.iter().any(|b| {
    matches!(b, GodItemBonus::Swiftstrider { regen_reduction_percent } if (*regen_reduction_percent - 50.0).abs() < f64::EPSILON)
});
assert!(has_swiftstrider, "Expected Swiftstrider bonus with 50%");
```

7. Add new tests for the new bonuses and helpers:

```rust
#[test]
fn test_sleipnir_has_swiftfoot_bonus() {
    let def = sleipnir_definition();
    let has_swiftfoot = def.bonuses.iter().any(|b| {
        matches!(b, GodItemBonus::Swiftfoot { dungeon_speed_percent } if (*dungeon_speed_percent - 50.0).abs() < f64::EPSILON)
    });
    assert!(has_swiftfoot, "Expected Swiftfoot bonus with 50%");
}

#[test]
fn test_sleipnir_has_nimble_hands_bonus() {
    let def = sleipnir_definition();
    let has_nimble = def.bonuses.iter().any(|b| {
        matches!(b, GodItemBonus::NimbleHands { fishing_reduction_percent } if (*fishing_reduction_percent - 50.0).abs() < f64::EPSILON)
    });
    assert!(has_nimble, "Expected NimbleHands bonus with 50%");
}

#[test]
fn test_sleipnir_has_three_bonuses() {
    let def = sleipnir_definition();
    assert_eq!(def.bonuses.len(), 3);
}

#[test]
fn test_equipped_god_item_dungeon_speed_with_sleipnir() {
    let mut equipment = crate::items::Equipment::new();
    let sleipnir = sleipnir_definition().to_item();
    equipment.set(crate::items::EquipmentSlot::Boots, Some(sleipnir));
    assert!((equipped_god_item_dungeon_speed_percent(&equipment) - 50.0).abs() < f64::EPSILON);
}

#[test]
fn test_equipped_god_item_dungeon_speed_without_god_item() {
    let equipment = crate::items::Equipment::new();
    assert!((equipped_god_item_dungeon_speed_percent(&equipment)).abs() < f64::EPSILON);
}

#[test]
fn test_equipped_god_item_fishing_reduction_with_sleipnir() {
    let mut equipment = crate::items::Equipment::new();
    let sleipnir = sleipnir_definition().to_item();
    equipment.set(crate::items::EquipmentSlot::Boots, Some(sleipnir));
    assert!((equipped_god_item_fishing_reduction_percent(&equipment) - 50.0).abs() < f64::EPSILON);
}

#[test]
fn test_equipped_god_item_fishing_reduction_without_god_item() {
    let equipment = crate::items::Equipment::new();
    assert!((equipped_god_item_fishing_reduction_percent(&equipment)).abs() < f64::EPSILON);
}
```

**Verify:** `cargo build --all-targets && cargo test`

**Commit:** `feat: add Swiftfoot and NimbleHands bonuses to GodItemBonus, change bonus to bonuses vec`

---

## Task 2: Add dungeon speed parameter to update_dungeon()

**Files:**
- Modify: `src/dungeon/logic.rs`
- Modify: `src/core/tick.rs` (1 call site)
- Modify: `tests/game_tick_behavior_test.rs` (1 call site)
- Modify: `tests/dungeon_completion_test.rs` (1 call site)
- Modify: `tests/behavior_lock_fishing_dungeon_test.rs` (4 call sites)

### What to do

1. Add `god_item_dungeon_speed_percent: f64` parameter to `update_dungeon()`:

```rust
pub fn update_dungeon(
    state: &mut GameState,
    delta_time: f64,
    god_item_dungeon_speed_percent: f64,
) -> Vec<DungeonEvent> {
```

2. Apply the speed reduction to the move interval calculation (around line 71-75):

```rust
// Use faster interval when traveling through cleared rooms
let base_interval = if is_traveling {
    ROOM_TRAVEL_INTERVAL
} else {
    ROOM_MOVE_INTERVAL
};
let move_interval = base_interval * (1.0 - god_item_dungeon_speed_percent / 100.0);
```

3. Update the tick.rs call site (line ~312). Wire the helper:

```rust
let god_item_dungeon_speed = god_items::equipped_god_item_dungeon_speed_percent(&state.equipment);
let dungeon_events = update_dungeon(state, delta_time, god_item_dungeon_speed);
```

Add the import if needed:
```rust
use crate::god_items;
```

4. Update all 6 test call sites to pass `0.0` (no god item bonus):

- `tests/game_tick_behavior_test.rs`: `update_dungeon(&mut state, delta_time, 0.0)`
- `tests/dungeon_completion_test.rs`: `update_dungeon(&mut state, ROOM_MOVE_INTERVAL + 0.1, 0.0)`
- `tests/behavior_lock_fishing_dungeon_test.rs` (4 sites): add `0.0` third argument

5. Add a unit test in `dungeon/logic.rs` `#[cfg(test)]` section that verifies the speed bonus reduces move timing:

```rust
#[test]
fn test_god_item_dungeon_speed_reduces_move_interval() {
    // Setup a dungeon with a revealed adjacent room
    let mut state = GameState::new_for_testing();
    let dungeon = Dungeon::generate(DungeonSize::Small, 1);
    state.active_dungeon = Some(dungeon);

    // Mark current room as cleared so movement can happen
    let pos = state.active_dungeon.as_ref().unwrap().player_position;
    state.active_dungeon.as_mut().unwrap().current_room_cleared = true;

    // Without speed bonus: need 2.5s to move
    // Accumulate 1.3s — not enough without bonus
    let events = update_dungeon(&mut state, 1.3, 0.0);
    // With 50% speed bonus: interval is 1.25s, so 1.3s should be enough
    state.active_dungeon.as_mut().unwrap().move_timer = 0.0;
    let events_fast = update_dungeon(&mut state, 1.3, 50.0);
    // The fast version should have moved (produced events) while normal didn't
    // Note: this depends on having a revealed adjacent room; if the dungeon
    // generation doesn't guarantee this, the test may need adjustment.
}
```

**Verify:** `cargo build --all-targets && cargo test`

**Commit:** `feat: add god item dungeon speed parameter to update_dungeon`

---

## Task 3: Add fishing speed parameter to tick_fishing_with_haven_result()

**Files:**
- Modify: `src/fishing/logic.rs`
- Modify: `src/core/tick.rs` (1 call site)
- Modify: `tests/game_tick_behavior_test.rs` (4 call sites)
- Modify: `tests/fishing_integration_test.rs` (1 call site)
- Modify: `tests/behavior_lock_fishing_dungeon_test.rs` (13 call sites)

### What to do

1. Add `god_item_fishing_reduction_percent: f64` parameter to `tick_fishing_with_haven_result()`:

```rust
pub fn tick_fishing_with_haven_result(
    state: &mut GameState,
    rng: &mut impl Rng,
    haven: &HavenFishingBonuses,
    god_item_fishing_reduction_percent: f64,
) -> FishingTickResult {
```

2. Apply the god item reduction multiplicatively AFTER Haven's reduction. Everywhere `apply_timer_reduction(base_ticks, haven.timer_reduction_percent)` is called, change to apply both reductions:

```rust
// Pattern: apply Haven first, then god item
let after_haven = apply_timer_reduction(base_ticks, haven.timer_reduction_percent);
session.ticks_remaining = apply_timer_reduction(after_haven, god_item_fishing_reduction_percent);
```

There are 3 sites in the function where `apply_timer_reduction` is called (one per phase: Casting→Waiting, Waiting→Reeling, and the initial casting setup). Find them all. The two inside the `match session.phase` block are at lines ~89-90 and ~100-101. Also check for the initial casting ticks setup (may be in the session creation code elsewhere — check `start_fishing` or similar).

Actually, looking at the code more carefully: the initial casting ticks are set when the fishing session is created (in `start_fishing()` or in `game_state.rs`). The `tick_fishing_with_haven_result` function only sees phase transitions. So only 2 `apply_timer_reduction` calls need updating (Casting→Waiting and Waiting→Reeling transitions).

Wait — there should also be a Reeling→Casting transition (after catching a fish, start casting again). Let me check:

After the Reeling phase catches a fish, it transitions back to Casting. Find that code and apply the same pattern there. Search for the casting ticks setup after a catch.

3. Update the tick.rs call site (line ~384):

```rust
let god_item_fishing_reduction = god_items::equipped_god_item_fishing_reduction_percent(&state.equipment);
let fishing_result = tick_fishing_with_haven_result(state, rng, &haven_fishing, god_item_fishing_reduction);
```

4. Update ALL 18 test call sites to pass `0.0` as the fourth argument:

- `tests/game_tick_behavior_test.rs`: 4 call sites
- `tests/fishing_integration_test.rs`: 1 call site
- `tests/behavior_lock_fishing_dungeon_test.rs`: 13 call sites

Use grep to find them all: `grep -n "tick_fishing_with_haven_result(" tests/`

5. Also check for internal calls within `src/fishing/logic.rs` itself — the grep showed 7 occurrences in that file. Some may be the function definition + doc comments, but verify and update any internal calls.

6. Add a unit test in `fishing/logic.rs` tests:

```rust
#[test]
fn test_god_item_fishing_reduction_stacks_with_haven() {
    // apply_timer_reduction is multiplicative
    let base_ticks = 100;
    let after_haven = apply_timer_reduction(base_ticks, 40.0); // 60 ticks
    assert_eq!(after_haven, 60);
    let after_god_item = apply_timer_reduction(after_haven, 50.0); // 30 ticks
    assert_eq!(after_god_item, 30);
    // Total reduction: 70% (not 90% — multiplicative, not additive)
}
```

**Verify:** `cargo build --all-targets && cargo test`

**Commit:** `feat: add god item fishing speed parameter to tick_fishing_with_haven_result`

---

## Verification

After all tasks:

```bash
make check   # format, clippy, test, build, audit
```

All ~1,298+ existing tests must continue passing. Coverage should remain above 90%.
