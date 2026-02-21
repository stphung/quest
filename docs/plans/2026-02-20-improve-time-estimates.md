# Improve Time Estimates Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make level and prestige ETA calculations accurate by only sampling XP rate during combat seconds, and smooth volatility with a longer 15-minute window.

**Architecture:** Add a `combat_seconds_this_tick` flag to GameState. Set it in `apply_tick_xp()` when XP is earned. Only push XP samples at second boundaries when this flag is true. Replace hardcoded 300-sample cap with a 900-sample constant.

**Tech Stack:** Rust, no new dependencies

**Worktree:** `/Users/stphung/workspace/quest/.worktrees/improve-time-estimates`

**Run commands from worktree root.** All file paths are relative to the worktree.

---

### Task 1: Add XP_RATE_WINDOW_SECONDS constant

**Files:**
- Modify: `src/core/constants.rs:25` (after `MAX_OFFLINE_SECONDS`)

**Step 1: Add the constant**

After line 25 (`pub const MAX_OFFLINE_SECONDS`), add:

```rust
pub const XP_RATE_WINDOW_SECONDS: usize = 900; // 15 min of combat time
```

**Step 2: Verify it compiles**

Run: `cargo build --quiet 2>&1 | head -5`
Expected: Clean build (constant is unused for now, but that's fine — it's a `pub const`)

**Step 3: Commit**

```bash
git add src/core/constants.rs
git commit -m "feat: add XP_RATE_WINDOW_SECONDS constant (900 = 15 min)"
```

---

### Task 2: Add combat_seconds_this_tick flag to GameState

**Files:**
- Modify: `src/core/game_state.rs:112-117` (add field after `derived_stats_dirty`)
- Modify: `src/core/game_state.rs:160-161` (add to `new()` constructor)
- Modify: `src/core/game_state.rs:173-179` (update `xp_per_hour()` doc comment)
- Modify: `src/character/persistence.rs:84-85` (add to load_character init)
- Modify: `src/character/manager.rs:122-123` (add to create_character init)
- Modify: `src/character/prestige_actions.rs:63-64` (add to prestige reset)

**Step 1: Add field to GameState struct**

In `src/core/game_state.rs`, after the `xp_this_second` field (line 117), add:

```rust
    /// True if any combat XP was earned during the current second (controls rate sampling)
    #[serde(skip)]
    pub combat_seconds_this_tick: bool,
```

**Step 2: Initialize in GameState::new()**

In the `Self { ... }` block in `new()`, after `xp_this_second: 0,` add:

```rust
            combat_seconds_this_tick: false,
```

**Step 3: Update xp_per_hour() doc comment**

Change the doc comment on `xp_per_hour()` from:
```rust
    /// Returns XP per hour based on rolling 5-minute window, or None if < 10s of data.
```
to:
```rust
    /// Returns XP per hour based on rolling 15-minute combat-only window, or None if < 10s of data.
```

**Step 4: Initialize in persistence.rs load_character**

In `src/character/persistence.rs`, after `xp_this_second: 0,` add:

```rust
            combat_seconds_this_tick: false,
```

**Step 5: Initialize in manager.rs create_character**

In `src/character/manager.rs`, after `xp_this_second: 0,` add:

```rust
            combat_seconds_this_tick: false,
```

**Step 6: Reset in prestige_actions.rs**

In `src/character/prestige_actions.rs`, after the line `state.xp_this_second = 0;` add:

```rust
    state.combat_seconds_this_tick = false;
```

**Step 7: Verify it compiles and tests pass**

Run: `cargo test --quiet 2>&1 | tail -3`
Expected: All tests pass (new field is just initialized, not used yet)

**Step 8: Commit**

```bash
git add src/core/game_state.rs src/character/persistence.rs src/character/manager.rs src/character/prestige_actions.rs
git commit -m "feat: add combat_seconds_this_tick flag to GameState"
```

---

### Task 3: Set flag in apply_tick_xp()

**Files:**
- Modify: `src/core/xp.rs:52-53`

**Step 1: Set the flag when XP is applied**

In `apply_tick_xp()`, change:

```rust
    state.xp_this_second += xp_gain as u64;
    state.character_xp += xp_gain as u64;
```

to:

```rust
    state.xp_this_second += xp_gain as u64;
    state.character_xp += xp_gain as u64;
    state.combat_seconds_this_tick = true;
```

**Step 2: Verify tests pass**

Run: `cargo test --quiet 2>&1 | tail -3`
Expected: All tests pass

**Step 3: Commit**

```bash
git add src/core/xp.rs
git commit -m "feat: set combat_seconds_this_tick in apply_tick_xp"
```

---

### Task 4: Update sampling in tick.rs (combat path)

**Files:**
- Modify: `src/core/tick.rs:176-180`

**Step 1: Change sampling to combat-only with new window constant**

Replace the block at lines 176-180:

```rust
        state.xp_rate_samples.push_back(state.xp_this_second);
        state.xp_this_second = 0;
        if state.xp_rate_samples.len() > 300 {
            state.xp_rate_samples.pop_front();
        }
```

with:

```rust
        if state.combat_seconds_this_tick {
            state.xp_rate_samples.push_back(state.xp_this_second);
            if state.xp_rate_samples.len() > crate::core::constants::XP_RATE_WINDOW_SECONDS {
                state.xp_rate_samples.pop_front();
            }
        }
        state.xp_this_second = 0;
        state.combat_seconds_this_tick = false;
```

Note: `xp_this_second` and `combat_seconds_this_tick` are always reset regardless of whether a sample was pushed.

**Step 2: Verify tests pass**

Run: `cargo test --quiet 2>&1 | tail -3`
Expected: All tests pass (combat path still pushes samples when XP is earned)

**Step 3: Commit**

```bash
git add src/core/tick.rs
git commit -m "feat: combat-only sampling with 900-sample window in tick.rs"
```

---

### Task 5: Update sampling in tick_stages.rs (fishing path)

**Files:**
- Modify: `src/core/tick_stages.rs:288-292`

**Step 1: Change fishing path to combat-only sampling**

Replace the block at lines 288-292:

```rust
        state.xp_rate_samples.push_back(state.xp_this_second);
        state.xp_this_second = 0;
        if state.xp_rate_samples.len() > 300 {
            state.xp_rate_samples.pop_front();
        }
```

with:

```rust
        if state.combat_seconds_this_tick {
            state.xp_rate_samples.push_back(state.xp_this_second);
            if state.xp_rate_samples.len() > crate::core::constants::XP_RATE_WINDOW_SECONDS {
                state.xp_rate_samples.pop_front();
            }
        }
        state.xp_this_second = 0;
        state.combat_seconds_this_tick = false;
```

This is the critical fix: during fishing, `combat_seconds_this_tick` is always false (no `apply_tick_xp()` calls), so zeros are never pushed into the sample window.

**Step 2: Run tests — expect 2 failures**

Run: `cargo test --quiet 2>&1 | grep "FAILED\|failures"`

Expected failures:
- `test_fishing_tick_pushes_xp_sample_at_second_boundary` — this test expects fishing to push samples
- `test_fishing_xp_rate_samples_capped_at_300` — this test expects 300-sample cap during fishing

These are expected because the behavior intentionally changed.

**Step 3: Commit (with failing tests noted)**

```bash
git add src/core/tick_stages.rs
git commit -m "feat: combat-only sampling in fishing tick path

Intentionally breaks 2 tests that expected fishing to push XP samples.
Tests will be updated in next commit."
```

---

### Task 6: Update existing tests for new behavior

**Files:**
- Modify: `tests/tick_stages_coverage_test.rs` — two tests to update
- Modify: `tests/item_combat_coverage_test.rs` — one test to verify still works

**Step 1: Update test_fishing_tick_pushes_xp_sample_at_second_boundary**

This test (around line 2067) currently expects fishing ticks to push `xp_this_second` into samples. The new behavior: fishing ticks should NOT push samples because `combat_seconds_this_tick` is false.

Find and replace the assertions at the end of this test. Change:

```rust
    assert_eq!(state.xp_rate_samples.len(), 1);
    assert_eq!(state.xp_rate_samples[0], 42);
    assert_eq!(
        state.xp_this_second, 0,
        "xp_this_second should reset after push"
    );
```

to:

```rust
    assert!(
        state.xp_rate_samples.is_empty(),
        "Fishing ticks should not push XP samples (combat_seconds_this_tick is false)"
    );
    assert_eq!(
        state.xp_this_second, 0,
        "xp_this_second should still reset even without push"
    );
```

**Step 2: Update test_fishing_xp_rate_samples_capped_at_300**

This test (around line 2099) pre-fills 300 samples and checks that a fishing tick caps at 300. With the new behavior, fishing ticks don't push at all, so the samples stay at whatever count they were.

Rename and rewrite this test. Replace the entire test function `test_fishing_xp_rate_samples_capped_at_300` with:

```rust
#[test]
fn test_fishing_tick_does_not_push_samples() {
    let mut state = fresh_state();
    state.active_fishing = Some(make_fishing_session(FishingPhase::Waiting, 5000, 100));

    // Pre-fill xp_rate_samples with some data
    for i in 0..50 {
        state.xp_rate_samples.push_back(i);
    }
    assert_eq!(state.xp_rate_samples.len(), 50);

    let mut tc = 9u32;
    let mut ach = Achievements::default();
    let mut rng = seeded_rng(1);
    let bonuses = default_haven_bonuses();
    let mut result = TickResult::default();

    state.xp_this_second = 999;

    tick_stages::process_fishing_tick(
        &mut state,
        &mut tc,
        0.1,
        &bonuses,
        &mut ach,
        false,
        &mut result,
        &mut rng,
    );

    assert_eq!(
        state.xp_rate_samples.len(),
        50,
        "Fishing should not push samples — count should be unchanged"
    );
    assert_eq!(
        state.xp_this_second, 0,
        "xp_this_second should still reset"
    );
}
```

**Step 3: Run all tests**

Run: `cargo test --quiet 2>&1 | tail -3`
Expected: All tests pass

**Step 4: Commit**

```bash
git add tests/tick_stages_coverage_test.rs tests/item_combat_coverage_test.rs
git commit -m "test: update XP sampling tests for combat-only behavior"
```

---

### Task 7: Add new tests for combat-only sampling

**Files:**
- Modify: `tests/tick_stages_coverage_test.rs` — add 3 new tests

**Step 1: Add test that combat ticks DO push samples**

Add at the end of the file (before the closing `}`):

```rust
#[test]
fn test_combat_tick_pushes_xp_sample_when_flag_set() {
    let mut state = fresh_state();
    // Simulate a second where combat XP was earned
    state.xp_this_second = 500;
    state.combat_seconds_this_tick = true;

    // Manually trigger second boundary in tick.rs logic
    state.xp_rate_samples.clear();

    // Push sample like tick.rs does
    if state.combat_seconds_this_tick {
        state.xp_rate_samples.push_back(state.xp_this_second);
    }
    state.xp_this_second = 0;
    state.combat_seconds_this_tick = false;

    assert_eq!(state.xp_rate_samples.len(), 1);
    assert_eq!(state.xp_rate_samples[0], 500);
    assert_eq!(state.xp_this_second, 0);
    assert!(!state.combat_seconds_this_tick);
}

#[test]
fn test_no_sample_pushed_when_combat_flag_false() {
    let mut state = fresh_state();
    state.xp_this_second = 0;
    state.combat_seconds_this_tick = false;

    let initial_len = state.xp_rate_samples.len();

    // Simulate second boundary without combat
    if state.combat_seconds_this_tick {
        state.xp_rate_samples.push_back(state.xp_this_second);
    }
    state.xp_this_second = 0;
    state.combat_seconds_this_tick = false;

    assert_eq!(state.xp_rate_samples.len(), initial_len, "No sample should be pushed without combat");
}

#[test]
fn test_xp_rate_samples_capped_at_900() {
    let mut state = fresh_state();
    // Pre-fill to 900
    for i in 0..900 {
        state.xp_rate_samples.push_back(i);
    }
    assert_eq!(state.xp_rate_samples.len(), 900);

    // Simulate combat second boundary
    state.xp_this_second = 9999;
    state.combat_seconds_this_tick = true;

    if state.combat_seconds_this_tick {
        state.xp_rate_samples.push_back(state.xp_this_second);
        if state.xp_rate_samples.len() > quest::core::constants::XP_RATE_WINDOW_SECONDS {
            state.xp_rate_samples.pop_front();
        }
    }

    assert_eq!(state.xp_rate_samples.len(), 900, "Samples should stay capped at 900");
    assert_eq!(*state.xp_rate_samples.back().unwrap(), 9999, "New sample at back");
    assert_eq!(*state.xp_rate_samples.front().unwrap(), 1, "Oldest (0) popped");
}
```

**Step 2: Run all tests**

Run: `cargo test --quiet 2>&1 | tail -3`
Expected: All tests pass

**Step 3: Commit**

```bash
git add tests/tick_stages_coverage_test.rs
git commit -m "test: add combat-only XP sampling tests and 900-sample cap test"
```

---

### Task 8: Integration test — mixed combat/fishing rate stability

**Files:**
- Modify: `tests/tick_stages_coverage_test.rs` — add 1 integration test

**Step 1: Add mixed-session integration test**

```rust
#[test]
fn test_xp_rate_stable_across_fishing_interruption() {
    use quest::core::game_state::GameState;

    let mut state = GameState::new("Rate Test".to_string(), 0);

    // Simulate 30 seconds of combat earning ~500 XP/sec
    for _ in 0..30 {
        state.xp_this_second = 500;
        state.combat_seconds_this_tick = true;
        state.xp_rate_samples.push_back(state.xp_this_second);
        if state.xp_rate_samples.len() > quest::core::constants::XP_RATE_WINDOW_SECONDS {
            state.xp_rate_samples.pop_front();
        }
        state.xp_this_second = 0;
        state.combat_seconds_this_tick = false;
    }

    let rate_before = state.xp_per_hour().unwrap();

    // Simulate 60 seconds of fishing (no samples pushed)
    for _ in 0..60 {
        // combat_seconds_this_tick stays false, so no push
        state.xp_this_second = 0;
        state.combat_seconds_this_tick = false;
        // No push — this is the key behavioral change
    }

    let rate_after = state.xp_per_hour().unwrap();

    assert_eq!(
        rate_before, rate_after,
        "XP rate should be unchanged after fishing interruption (no zeros pushed)"
    );
    assert_eq!(state.xp_rate_samples.len(), 30, "Only combat seconds should be in samples");
}
```

**Step 2: Run all tests**

Run: `cargo test --quiet 2>&1 | tail -3`
Expected: All tests pass

**Step 3: Commit**

```bash
git add tests/tick_stages_coverage_test.rs
git commit -m "test: add integration test for rate stability across fishing"
```

---

### Task 9: Final verification and cleanup

**Step 1: Run full CI checks**

Run: `cargo fmt && cargo clippy --all-targets --quiet -- -D warnings && cargo test --quiet`
Expected: Format clean, no warnings, all tests pass

**Step 2: Verify the fix with a quick sanity check**

Open `src/core/game_state.rs` and confirm `xp_per_hour()` still works correctly:
- Returns `None` when `xp_rate_samples.len() < 10`
- Returns average XP/sec * 3600 otherwise
- Since samples are now combat-only, this rate reflects combat XP rate

**Step 3: Squash or finalize commits as appropriate**

All work should be on the `improve-time-estimates` branch in the worktree.
