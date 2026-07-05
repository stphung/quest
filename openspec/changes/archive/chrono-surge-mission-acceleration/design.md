> Backported design record. Sources: docs/plans/2026-03-06-chrono-surge-mission-acceleration-design.md, docs/plans/2026-03-06-chrono-surge-mission-acceleration.md.

## 2026-03-06-chrono-surge-mission-acceleration-design.md

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

## 2026-03-06-chrono-surge-mission-acceleration.md

# Chrono Surge Mission Acceleration — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make Chrono Surge accelerate Deep mission timers (1:1 with surge ticks), auto-resolving passed check-in events and showing missions completed in the surge summary.

**Architecture:** Add `accelerate_missions()` in `src/deep/missions.rs` that shifts `ends_at` on active missions. Call it from `run_chrono_surge_batch()` after each batch. Extend `ChronoSurgeState`/`ChronoSurgeSummary` with `missions_completed` counter. Events trigger naturally via existing `tick_all_missions` since shifting `ends_at` increases `progress()`.

**Tech Stack:** Rust, chrono::Duration, existing Deep mission infrastructure

---

### Task 1: Add `missions_completed` to ChronoSurgeState and ChronoSurgeSummary

**Files:**
- Modify: `src/stormglass/types.rs:191-203` (ChronoSurgeState)
- Modify: `src/stormglass/types.rs:251-259` (ChronoSurgeSummary)

**Step 1: Add field to ChronoSurgeState**

In `src/stormglass/types.rs`, add `pub missions_completed: u32` after `items_equipped` (line 198):

```rust
pub struct ChronoSurgeState {
    pub ticks_remaining: u64,
    pub ticks_total: u64,
    pub batch_size: u64,
    pub kills: u64,
    pub levels_gained: u32,
    pub items_equipped: u32,
    pub missions_completed: u32,  // NEW
    pub overcharged: bool,
    pub created_at_ms: u128,
}
```

**Step 2: Initialize in `ChronoSurgeState::new()`**

In the `new()` method (around line 206), add `missions_completed: 0` to the struct literal.

**Step 3: Add field to ChronoSurgeSummary**

```rust
pub struct ChronoSurgeSummary {
    pub kills: u64,
    pub levels_gained: u32,
    pub items_equipped: u32,
    pub missions_completed: u32,  // NEW
    pub ticks_completed: u64,
    pub ticks_total: u64,
    pub overcharged: bool,
}
```

**Step 4: Wire summary field in `run_chrono_surge_batch()`**

In `src/main_helpers/chrono_surge.rs:84-90`, add `missions_completed: surge.missions_completed` to the `ChronoSurgeSummary` struct literal.

**Step 5: Build to verify no compilation errors**

Run: `cargo build 2>&1 | head -20`
Expected: Compiles successfully

**Step 6: Commit**

```bash
git add src/stormglass/types.rs src/main_helpers/chrono_surge.rs
git commit -m "feat(stormglass): add missions_completed to ChronoSurgeState and summary"
```

---

### Task 2: Implement `accelerate_missions()` with TDD

**Files:**
- Modify: `src/deep/missions.rs` (add function + unit tests)
- Modify: `src/deep/mod.rs` (re-export)

**Step 1: Write failing tests**

Add at the bottom of the `#[cfg(test)] mod tests` block in `src/deep/missions.rs`:

```rust
#[test]
fn test_accelerate_missions_shifts_ends_at() {
    let now = Utc::now();
    let mut prestige = DeepPrestige::default();
    let acceleration = Duration::seconds(3600); // 1 hour

    // Create a mission that ends in 4 hours
    let mission = Mission {
        id: 1,
        mission_type: MissionType::SupplyRun,
        layer: 1,
        squad: vec![],
        started_at: now,
        ends_at: now + Duration::hours(4),
        events: vec![],
        pending_event_index: 0,
        status: MissionStatus::Active,
        result: None,
        is_first_orders: false,
    };
    prestige.active_missions.push(mission);

    let completed = accelerate_missions(&mut prestige, acceleration);

    assert_eq!(completed, 0);
    let m = &prestige.active_missions[0];
    // ends_at should be shifted 1 hour earlier
    let expected = now + Duration::hours(3);
    let diff = (m.ends_at - expected).num_seconds().abs();
    assert!(diff < 2, "ends_at not shifted correctly: diff={diff}s");
}

#[test]
fn test_accelerate_missions_completes_mission() {
    let now = Utc::now();
    let mut prestige = DeepPrestige::default();
    let acceleration = Duration::seconds(7200); // 2 hours

    // Create a mission that ends in 1 hour
    let mission = Mission {
        id: 1,
        mission_type: MissionType::SupplyRun,
        layer: 1,
        squad: vec![],
        started_at: now - Duration::hours(3),
        ends_at: now + Duration::hours(1),
        events: vec![],
        pending_event_index: 0,
        status: MissionStatus::Active,
        result: None,
        is_first_orders: false,
    };
    prestige.active_missions.push(mission);

    let completed = accelerate_missions(&mut prestige, acceleration);

    assert_eq!(completed, 1);
    // Mission still in active_missions (tick_all_missions handles moving to pending_results)
    let m = &prestige.active_missions[0];
    assert!(m.is_time_elapsed(now), "Mission should be time-elapsed after acceleration");
}

#[test]
fn test_accelerate_missions_skips_non_active() {
    let now = Utc::now();
    let mut prestige = DeepPrestige::default();
    let acceleration = Duration::seconds(3600);

    let mut mission = Mission {
        id: 1,
        mission_type: MissionType::SupplyRun,
        layer: 1,
        squad: vec![],
        started_at: now,
        ends_at: now + Duration::hours(4),
        events: vec![],
        pending_event_index: 0,
        status: MissionStatus::Completed,
        result: None,
        is_first_orders: false,
    };
    let original_ends_at = mission.ends_at;
    prestige.active_missions.push(mission);

    let completed = accelerate_missions(&mut prestige, acceleration);

    assert_eq!(completed, 0);
    assert_eq!(prestige.active_missions[0].ends_at, original_ends_at);
}

#[test]
fn test_accelerate_missions_multiple() {
    let now = Utc::now();
    let mut prestige = DeepPrestige::default();
    let acceleration = Duration::seconds(3600);

    for i in 1..=3 {
        prestige.active_missions.push(Mission {
            id: i,
            mission_type: MissionType::SupplyRun,
            layer: 1,
            squad: vec![],
            started_at: now,
            ends_at: now + Duration::hours(i as i64),
            events: vec![],
            pending_event_index: 0,
            status: MissionStatus::Active,
            result: None,
            is_first_orders: false,
        });
    }

    let completed = accelerate_missions(&mut prestige, acceleration);

    // Mission 1 (1h duration) should now be elapsed
    assert_eq!(completed, 1);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib deep::missions::tests::test_accelerate_missions -- 2>&1 | tail -10`
Expected: FAIL — `accelerate_missions` not found

**Step 3: Implement `accelerate_missions()`**

Add in `src/deep/missions.rs`, in the "Mission Ticking" section (after `tick_mission` around line 1122):

```rust
/// Accelerate all active missions by subtracting `acceleration` from their `ends_at`.
///
/// Called during Chrono Surge to fast-forward mission timers. Events trigger
/// naturally via the existing `tick_all_missions` flow since shifting `ends_at`
/// increases `progress()` for the same wall-clock `now`.
///
/// Returns the number of missions whose `ends_at` fell into the past (became
/// completable) during this acceleration. The caller tallies this for the surge
/// summary. Actual mission resolution happens in `tick_all_missions`.
pub fn accelerate_missions(
    prestige: &mut DeepPrestige,
    acceleration: Duration,
) -> u32 {
    let now = Utc::now();
    let mut newly_completed = 0u32;

    for mission in &mut prestige.active_missions {
        if !matches!(
            mission.status,
            MissionStatus::Active | MissionStatus::EventPending
        ) {
            continue;
        }

        let was_elapsed = mission.is_time_elapsed(now);
        mission.ends_at -= acceleration;

        if !was_elapsed && mission.is_time_elapsed(now) {
            newly_completed += 1;
        }
    }

    newly_completed
}
```

**Step 4: Add import for `Duration` if not already present**

Check top of `src/deep/missions.rs` — `chrono::Duration` is already imported at line 15.

**Step 5: Run tests to verify they pass**

Run: `cargo test --lib deep::missions::tests::test_accelerate_missions 2>&1 | tail -15`
Expected: 4 tests PASS

**Step 6: Add re-export in `src/deep/mod.rs`**

Add `accelerate_missions` to the missions re-export block (line 116-121):

```rust
pub use missions::{
    accelerate_missions, available_mission_count, daily_supply_run_resets_at,
    effective_duration_secs, generate_mission_pool, is_daily_supply_run_available,
    maybe_refresh_mission_pool, maybe_refresh_recruit_pool, resolve_mission,
    resolve_offline_missions, run_softlock_safeguards, start_mission, tick_all_missions,
    tick_mission, validate_squad_assignment, MissionTickSummary, OfflineResolutionSummary,
    SquadAssignmentError, POOL_REFRESH_INTERVAL_SECS,
};
```

**Step 7: Build to verify**

Run: `cargo build 2>&1 | head -20`
Expected: Compiles

**Step 8: Commit**

```bash
git add src/deep/missions.rs src/deep/mod.rs
git commit -m "feat(deep): add accelerate_missions() with unit tests"
```

---

### Task 3: Wire `accelerate_missions()` into the surge batch loop

**Files:**
- Modify: `src/main_helpers/chrono_surge.rs:44-106`

**Step 1: Add import**

At top of `src/main_helpers/chrono_surge.rs`, add:

```rust
use chrono::Duration;
```

**Step 2: Call accelerate_missions after the batch loop**

In `run_chrono_surge_batch()`, after line 77 (`surge.ticks_remaining -= 1;`) and the closing brace of the `for` loop (line 77), add:

```rust
    // Accelerate Deep mission timers by the wall-clock equivalent of this batch.
    if deep_state.persistent.discovered {
        let acceleration = Duration::milliseconds((batch as i64) * 100);
        let missions_done =
            crate::deep::missions::accelerate_missions(&mut deep_state.prestige, acceleration);
        if missions_done > 0 {
            surge.missions_completed += missions_done;
            needs_save = true;
        }
    }
```

Place this between the end of the `for` loop and the `state.stormglass = sg_before_batch;` line.

**Step 3: Build to verify**

Run: `cargo build 2>&1 | head -20`
Expected: Compiles

**Step 4: Run full test suite**

Run: `cargo test 2>&1 | tail -20`
Expected: All tests pass

**Step 5: Commit**

```bash
git add src/main_helpers/chrono_surge.rs
git commit -m "feat(stormglass): wire mission acceleration into Chrono Surge batch loop"
```

---

### Task 4: Render `missions_completed` in surge summary UI

**Files:**
- Modify: `src/ui/stormglass_scene.rs:1711-1715`

**Step 1: Add missions_completed to the stats array**

In `render_chrono_surge_summary()` at line 1711, change the `stats` array to:

```rust
    let mut stats: Vec<String> = vec![
        format!("\u{2694}  Kills: {}", summary.kills),
        format!("\u{2B06}  Levels gained: +{}", summary.levels_gained),
        format!("\u{1F528} Items equipped: {}", summary.items_equipped),
    ];
    if summary.missions_completed > 0 {
        stats.push(format!("\u{1F4DC} Missions completed: {}", summary.missions_completed));
    }
```

Note: `\u{1F4DC}` is the scroll emoji (📜), thematic for missions.

**Step 2: Adjust overlay height if missions line is shown**

At line 1669, the overlay height is `14u16`. This should accommodate the extra line. Change to:

```rust
    let base_height = if summary.missions_completed > 0 { 15u16 } else { 14u16 };
    let overlay_height = base_height.min(area.height.saturating_sub(2));
```

**Step 3: Build to verify**

Run: `cargo build 2>&1 | head -20`
Expected: Compiles

**Step 4: Commit**

```bash
git add src/ui/stormglass_scene.rs
git commit -m "feat(ui): show missions completed in Chrono Surge summary"
```

---

### Task 5: Final verification

**Step 1: Run clippy**

Run: `cargo clippy --all-targets -- -D warnings 2>&1 | tail -20`
Expected: No warnings

**Step 2: Run full test suite**

Run: `cargo test 2>&1 | tail -20`
Expected: All tests pass

**Step 3: Run format check**

Run: `cargo fmt --check 2>&1`
Expected: No formatting issues

**Step 4: Run full CI check**

Run: `make check`
Expected: All checks pass

**Step 5: Commit any fixups if needed**
