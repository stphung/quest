# Loom Power Integration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Connect the Loom of Worlds back to the main game: Ascension VII–X gated by pattern milestones, shuttle upgrades with progressive level caps, WR→PR generation at endgame, and 20 new Loom Zones (Z31–50) with 1.25x stat scaling.

**Architecture:** Extend the existing Ascension system to support levels 7–10 with a new pattern gate (alongside the existing Deep gate). Add a `completed_pattern_count()` helper to the Loom module. Add shuttle upgrade logic using the existing `Shuttle.level` field. Add WR→PR tick processing alongside Power Cores. Extend the zone data table from 30 to 50 entries with a new `LOOM_ZONE_STAT_MULTIPLIER` constant. Extend `sync_account_zone_unlocks` to handle Loom zone access.

**Tech Stack:** Rust, Ratatui, Serde (JSON persistence)

---

### Task 1: Add `completed_pattern_count()` helper to Loom

This helper is used by every downstream task (Ascension gating, shuttle caps, zone unlocks).

**Files:**
- Modify: `src/loom/types.rs`
- Modify: `src/loom/mod.rs`

**Step 1: Write the failing test**

Add to the bottom of the `#[cfg(test)] mod tests` block in `src/loom/types.rs`:

```rust
#[test]
fn test_completed_pattern_count_empty() {
    let state = LoomState::new();
    assert_eq!(state.persistent.completed_pattern_count(), 0);
}

#[test]
fn test_completed_pattern_count_some_completed() {
    let mut state = LoomState::new();
    state.persistent.patterns.push(WovenPattern {
        index: 0,
        name: "A".to_string(),
        requirements: vec![],
        completed: true,
    });
    state.persistent.patterns.push(WovenPattern {
        index: 1,
        name: "B".to_string(),
        requirements: vec![],
        completed: false,
    });
    assert_eq!(state.persistent.completed_pattern_count(), 1);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib loom::types::tests::test_completed_pattern_count -- --no-capture`
Expected: FAIL with "no method named `completed_pattern_count`"

**Step 3: Write minimal implementation**

In `src/loom/types.rs`, add a method to the `impl LoomPersistent` block (right after `max_shuttles()`):

```rust
/// Number of completed Woven Patterns.
pub fn completed_pattern_count(&self) -> usize {
    self.patterns.iter().filter(|p| p.completed).count()
}
```

Note: `max_shuttles()` already does the same computation. Refactor `max_shuttles()` to call this:

```rust
pub fn max_shuttles(&self) -> usize {
    self.completed_pattern_count()
}
```

**Step 4: Add re-export in `src/loom/mod.rs`**

No re-export needed — `completed_pattern_count()` is a method on `LoomPersistent` which is already public.

**Step 5: Run test to verify it passes**

Run: `cargo test --lib loom::types::tests::test_completed_pattern_count`
Expected: PASS (both tests)

**Step 6: Commit**

```bash
git add src/loom/types.rs
git commit -m "feat(loom): add completed_pattern_count() helper"
```

---

### Task 2: Extend Ascension to support levels VII–X with pattern gates

Change `MAX_ASCENSION_LEVEL` from 6 to 10, add new cost table entries for levels 7–10, and add a new `ascension_pattern_gate()` function.

**Files:**
- Modify: `src/ascension/types.rs`
- Modify: `src/ascension/logic.rs`

**Step 1: Write the failing tests**

Add to the `#[cfg(test)] mod tests` block in `src/ascension/types.rs`:

```rust
#[test]
fn test_ascension_cost_levels_7_through_10_loom() {
    assert_eq!(ascension_cost(7), 1500);
    assert_eq!(ascension_cost(8), 4000);
    assert_eq!(ascension_cost(9), 8000);
    assert_eq!(ascension_cost(10), 15000);
}

#[test]
fn test_ascension_pattern_gate() {
    assert_eq!(ascension_pattern_gate(1), None);
    assert_eq!(ascension_pattern_gate(6), None);
    assert_eq!(ascension_pattern_gate(7), Some(8));
    assert_eq!(ascension_pattern_gate(8), Some(16));
    assert_eq!(ascension_pattern_gate(9), Some(22));
    assert_eq!(ascension_pattern_gate(10), Some(28));
}

#[test]
fn test_ascension_combat_multiplier_levels_7_through_10() {
    assert!((ascension_combat_multiplier(7) - 96.0).abs() < 1e-10);
    assert!((ascension_combat_multiplier(8) - 144.0).abs() < 1e-10);
    assert!((ascension_combat_multiplier(9) - 216.0).abs() < 1e-10);
    assert!((ascension_combat_multiplier(10) - 324.0).abs() < 1e-10);
}

#[test]
fn test_max_shuttle_level_for_ascension() {
    assert_eq!(max_shuttle_level(0), 1);
    assert_eq!(max_shuttle_level(6), 1);
    assert_eq!(max_shuttle_level(7), 3);
    assert_eq!(max_shuttle_level(8), 5);
    assert_eq!(max_shuttle_level(9), 7);
    assert_eq!(max_shuttle_level(10), 10);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib ascension::types::tests -- --no-capture`
Expected: FAIL — `ascension_cost(7)` returns 575 (old formula), `ascension_pattern_gate` and `max_shuttle_level` don't exist.

**Step 3: Write the implementation**

In `src/ascension/types.rs`:

1. Change `MAX_ASCENSION_LEVEL` from 6 to 10.

2. Add the Loom Ascension cost table for levels 7–10. Replace the `ascension_cost()` function:

```rust
/// Loom-gated Ascension costs for levels 7-10.
const LOOM_ASCENSION_COSTS: [u32; 4] = [1500, 4000, 8000, 15000];

/// Loom pattern gates for Ascension levels 7-10.
const LOOM_ASCENSION_PATTERN_GATES: [usize; 4] = [8, 16, 22, 28];

/// Max shuttle level per Ascension tier (7-10).
const LOOM_SHUTTLE_LEVEL_CAPS: [u32; 4] = [3, 5, 7, 10];

/// Prestige rank cost to Ascend to the given level.
pub fn ascension_cost(level: u32) -> u32 {
    if (1..=6).contains(&level) {
        ASCENSION_COSTS[(level - 1) as usize]
    } else if (7..=10).contains(&level) {
        LOOM_ASCENSION_COSTS[(level - 7) as usize]
    } else {
        0
    }
}
```

3. Add the pattern gate function:

```rust
/// Woven Pattern gate for the given Ascension level.
/// Returns None for levels 1-6 (gated by Deep layers instead).
/// Returns Some(required_patterns) for levels 7-10.
pub fn ascension_pattern_gate(level: u32) -> Option<usize> {
    if (7..=10).contains(&level) {
        Some(LOOM_ASCENSION_PATTERN_GATES[(level - 7) as usize])
    } else {
        None
    }
}
```

4. Add the shuttle level cap function:

```rust
/// Maximum shuttle upgrade level allowed at the given Ascension level.
/// Returns 1 (no upgrades) for levels 0-6, progressive caps for 7-10.
pub fn max_shuttle_level(ascension_level: u32) -> u32 {
    if (7..=10).contains(&ascension_level) {
        LOOM_SHUTTLE_LEVEL_CAPS[(ascension_level - 7) as usize]
    } else {
        1
    }
}
```

5. Fix the existing test `test_ascension_cost_level_7_plus` — the old formula `500 + 75*(level-6)` no longer applies. Remove or update it:

```rust
#[test]
fn test_ascension_cost_level_0_returns_zero() {
    assert_eq!(ascension_cost(0), 0);
}

#[test]
fn test_ascension_cost_level_11_plus_returns_zero() {
    assert_eq!(ascension_cost(11), 0);
    assert_eq!(ascension_cost(100), 0);
}
```

6. Fix the existing test `test_total_pr_for_levels_1_through_6` — it should still pass (values 1-6 unchanged).

**Step 4: Update `src/ascension/logic.rs`**

The `can_ascend()` and `ascend()` functions need to check the pattern gate for levels 7+. They currently take `deepest_layer: u32`. Add a `completed_patterns: usize` parameter.

Update `can_ascend()`:

```rust
pub fn can_ascend(
    ascension_level: u32,
    prestige_rank: u32,
    deepest_layer: u32,
    completed_patterns: usize,
) -> bool {
    if ascension_level >= super::types::MAX_ASCENSION_LEVEL {
        return false;
    }
    let next = ascension_level + 1;
    let cost = super::types::ascension_cost(next);
    if prestige_rank < cost {
        return false;
    }
    if let Some(gate) = super::types::ascension_deep_gate(next) {
        if deepest_layer < gate {
            return false;
        }
    }
    if let Some(pattern_gate) = super::types::ascension_pattern_gate(next) {
        if completed_patterns < pattern_gate {
            return false;
        }
    }
    true
}
```

Add a new `AscendResult` variant:

```rust
/// Woven Pattern requirement not met.
PatternGateNotMet {
    needed_patterns: usize,
    current_patterns: usize,
},
```

Update `ascend()` to take `completed_patterns: usize` and check the pattern gate:

```rust
pub fn ascend(
    state: &mut crate::core::game_state::GameState,
    deepest_layer: u32,
    completed_patterns: usize,
) -> AscendResult {
    if state.ascension_level >= super::types::MAX_ASCENSION_LEVEL {
        return AscendResult::MaxLevelReached;
    }

    let next = state.ascension_level + 1;
    let cost = super::types::ascension_cost(next);

    if state.prestige_rank < cost {
        return AscendResult::InsufficientPR {
            needed: cost,
            have: state.prestige_rank,
        };
    }

    if let Some(gate) = super::types::ascension_deep_gate(next) {
        if deepest_layer < gate {
            return AscendResult::DeepGateNotMet {
                needed_layer: gate,
                current_layer: deepest_layer,
            };
        }
    }

    if let Some(pattern_gate) = super::types::ascension_pattern_gate(next) {
        if completed_patterns < pattern_gate {
            return AscendResult::PatternGateNotMet {
                needed_patterns: pattern_gate,
                current_patterns: completed_patterns,
            };
        }
    }

    state.prestige_rank -= cost;
    state.ascension_level = next;
    let multiplier = super::types::ascension_combat_multiplier(next);

    AscendResult::Success {
        new_level: next,
        multiplier,
    }
}
```

**Step 5: Fix all callers of `can_ascend()` and `ascend()`**

Search for all call sites. They need the extra `completed_patterns` parameter. For now, pass `0` where Loom state isn't available, or thread Loom state through. The call sites are:

- `src/ui/ascension_scene.rs` — `can_ascend(current_level, state.prestige_rank, deepest)` → add `completed_patterns` param. The render function needs to accept `&LoomState` (or just `completed_patterns: usize`).
- `src/input/` — wherever ascension input is handled. Search for `ascend(` calls.
- Any tests in `ascension/logic.rs`.

Run: `grep -rn "can_ascend\|ascend(" src/ --include="*.rs" | grep -v test | grep -v "//"`

Update each call site to pass `completed_patterns`. For UI rendering, add `loom: &LoomState` parameter to `render_ascension_confirm()` and use `loom.persistent.completed_pattern_count()`.

**Step 6: Run all tests**

Run: `cargo test`
Expected: PASS

**Step 7: Commit**

```bash
git add src/ascension/types.rs src/ascension/logic.rs src/ui/ascension_scene.rs
git commit -m "feat(ascension): extend to levels VII-X with Loom pattern gates"
```

---

### Task 3: Add shuttle upgrade logic

Implement shuttle level upgrades using the existing `Shuttle.level` field. Apply the level multiplier to intake caps. Gate upgrades behind Ascension VII+ with progressive level caps.

**Files:**
- Modify: `src/loom/logic.rs`
- Modify: `src/loom/mod.rs`

**Step 1: Write the failing tests**

Add to the test module in `src/loom/logic.rs`:

```rust
#[test]
fn test_shuttle_level_intake_multiplier() {
    // T1 shuttle at level 1: intake cap = 20.0
    assert!((shuttle_effective_intake_cap(1, 1) - 20.0).abs() < 0.001);
    // T1 shuttle at level 3: 20.0 * (1.0 + (3-1)*0.5) = 20.0 * 2.0 = 40.0
    assert!((shuttle_effective_intake_cap(1, 3) - 40.0).abs() < 0.001);
    // T3 shuttle at level 5: 40.0 * (1.0 + (5-1)*0.5) = 40.0 * 3.0 = 120.0
    assert!((shuttle_effective_intake_cap(3, 5) - 120.0).abs() < 0.001);
}

#[test]
fn test_upgrade_shuttle_success() {
    let mut loom = LoomState::new();
    initialize_loom(&mut loom);
    setup_patterns(&mut loom, 8); // Need Asc VII for shuttle upgrades
    // Unlock all nodes for shuttle building
    for node in loom.persistent.nodes.iter_mut() {
        node.unlocked = true;
    }
    // Build a T1 shuttle
    let recipes = crate::loom::recipes::all_recipes();
    let t1_idx = recipes.iter().position(|r| r.tier == 1).unwrap();
    let _ = build_shuttle(
        &mut loom,
        t1_idx,
        vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
        vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
    );
    // Give the shuttle enough buffer to afford the upgrade
    loom.persistent.shuttles[0].buffer = 500.0;
    loom.persistent.shuttles[0].under_construction = false;

    let result = upgrade_shuttle(&mut loom, 0, 7); // ascension_level = 7
    assert!(result.is_ok());
    assert_eq!(loom.persistent.shuttles[0].level, 2);
}

#[test]
fn test_upgrade_shuttle_blocked_by_ascension_cap() {
    let mut loom = LoomState::new();
    initialize_loom(&mut loom);
    setup_patterns(&mut loom, 8);
    for node in loom.persistent.nodes.iter_mut() {
        node.unlocked = true;
    }
    let recipes = crate::loom::recipes::all_recipes();
    let t1_idx = recipes.iter().position(|r| r.tier == 1).unwrap();
    let _ = build_shuttle(
        &mut loom,
        t1_idx,
        vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
        vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
    );
    loom.persistent.shuttles[0].buffer = 5000.0;
    loom.persistent.shuttles[0].under_construction = false;
    // Upgrade to level 3 (max for Asc VII)
    loom.persistent.shuttles[0].level = 3;

    let result = upgrade_shuttle(&mut loom, 0, 7); // at cap for Asc VII
    assert!(result.is_err());
}

#[test]
fn test_upgrade_shuttle_blocked_without_ascension_vii() {
    let mut loom = LoomState::new();
    initialize_loom(&mut loom);
    setup_patterns(&mut loom, 1);
    for node in loom.persistent.nodes.iter_mut() {
        node.unlocked = true;
    }
    let recipes = crate::loom::recipes::all_recipes();
    let t1_idx = recipes.iter().position(|r| r.tier == 1).unwrap();
    let _ = build_shuttle(
        &mut loom,
        t1_idx,
        vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
        vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
    );
    loom.persistent.shuttles[0].buffer = 5000.0;
    loom.persistent.shuttles[0].under_construction = false;

    let result = upgrade_shuttle(&mut loom, 0, 6); // no Asc VII
    assert!(result.is_err());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib loom::logic::tests::test_shuttle_level_intake -- --no-capture`
Expected: FAIL — functions don't exist yet.

**Step 3: Write the implementation**

In `src/loom/logic.rs`:

1. Add `shuttle_effective_intake_cap()`:

```rust
/// Effective intake cap for a shuttle, applying the level multiplier.
/// Formula: tier_intake_cap(tier) × node_level_multiplier(level)
pub fn shuttle_effective_intake_cap(tier: u8, level: u32) -> f64 {
    tier_intake_cap(tier) * node_level_multiplier(level)
}
```

2. Modify `tick_shuttle_pull()` to use `shuttle_effective_intake_cap()` instead of `tier_intake_cap()`. In the shuttle processing loop, change:

```rust
let cap = tier_intake_cap(r.tier);
```

to:

```rust
let cap = shuttle_effective_intake_cap(r.tier, r.level);
```

3. Add the shuttle upgrade function:

```rust
/// Error type for shuttle upgrade failures.
#[derive(Debug, Clone, PartialEq)]
pub enum ShuttleUpgradeError {
    /// Invalid shuttle index.
    InvalidIndex,
    /// Shuttle is under construction.
    UnderConstruction,
    /// Ascension level too low for shuttle upgrades (need VII+).
    AscensionTooLow,
    /// Shuttle already at max level for current Ascension tier.
    AtMaxLevel,
    /// Not enough output resource in shuttle buffer.
    InsufficientBuffer { needed: f64, have: f64 },
}

/// Attempt to upgrade a shuttle's level.
/// Cost is the same formula as node upgrades: 100 × level^1.5, paid from shuttle buffer.
/// Max level is capped by the player's Ascension level via max_shuttle_level().
pub fn upgrade_shuttle(
    loom: &mut LoomState,
    shuttle_idx: usize,
    ascension_level: u32,
) -> Result<(), ShuttleUpgradeError> {
    let max_level = crate::ascension::types::max_shuttle_level(ascension_level);
    if max_level <= 1 {
        return Err(ShuttleUpgradeError::AscensionTooLow);
    }

    let shuttle = loom
        .persistent
        .shuttles
        .get(shuttle_idx)
        .ok_or(ShuttleUpgradeError::InvalidIndex)?;

    if shuttle.under_construction {
        return Err(ShuttleUpgradeError::UnderConstruction);
    }

    if shuttle.level >= max_level {
        return Err(ShuttleUpgradeError::AtMaxLevel);
    }

    let cost = 100.0 * (shuttle.level as f64).powf(1.5);
    if shuttle.buffer < cost {
        return Err(ShuttleUpgradeError::InsufficientBuffer {
            needed: cost,
            have: shuttle.buffer,
        });
    }

    let shuttle = loom.persistent.shuttles.get_mut(shuttle_idx).unwrap();
    shuttle.buffer -= cost;
    shuttle.level += 1;

    Ok(())
}
```

4. Add re-exports in `src/loom/mod.rs`:

```rust
pub use logic::{shuttle_effective_intake_cap, upgrade_shuttle, ShuttleUpgradeError};
```

**Step 4: Run tests**

Run: `cargo test --lib loom::logic::tests`
Expected: PASS

**Step 5: Commit**

```bash
git add src/loom/logic.rs src/loom/mod.rs
git commit -m "feat(loom): add shuttle upgrade logic with Ascension-gated level caps"
```

---

### Task 4: Add WR→PR tick processing

Add a per-tick WR→PR conversion that runs when all 28 patterns are complete (Ascension X unlocked). Uses a tiered bracket system. Follows the same architecture as `tick_power_cores()`.

**Files:**
- Modify: `src/loom/logic.rs`
- Modify: `src/loom/mod.rs`
- Modify: `src/core/tick_types.rs` (new TickEvent variant)
- Modify: `src/core/tick_stages.rs` (call the new function from `tick_loom`)

**Step 1: Write the failing tests**

Add to the test module in `src/loom/logic.rs`:

```rust
#[test]
fn test_wr_to_pr_per_day_zero_rate() {
    assert_eq!(wr_to_pr_per_day(0.0), 0);
}

#[test]
fn test_wr_to_pr_per_day_low_bracket() {
    // 5 WR/hr → 5 * 5 = 25 PR/day
    assert_eq!(wr_to_pr_per_day(5.0), 25);
}

#[test]
fn test_wr_to_pr_per_day_mid_bracket() {
    // 20 WR/hr → (10 * 5) + (10 * 10) = 50 + 100 = 150 PR/day
    assert_eq!(wr_to_pr_per_day(20.0), 150);
}

#[test]
fn test_wr_to_pr_per_day_high_bracket() {
    // 60 WR/hr → (10 * 5) + (15 * 10) + (35 * 15) = 50 + 150 + 525 = 725 PR/day
    assert_eq!(wr_to_pr_per_day(60.0), 725);
}

#[test]
fn test_wr_to_pr_per_day_exact_bracket_boundary() {
    // 10 WR/hr → 10 * 5 = 50 PR/day
    assert_eq!(wr_to_pr_per_day(10.0), 50);
    // 25 WR/hr → (10 * 5) + (15 * 10) = 50 + 150 = 200 PR/day
    assert_eq!(wr_to_pr_per_day(25.0), 200);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib loom::logic::tests::test_wr_to_pr_per_day -- --no-capture`
Expected: FAIL — function doesn't exist.

**Step 3: Write the implementation**

In `src/loom/logic.rs`, add the bracket calculator:

```rust
/// Calculate PR generated per day from a given WR production rate (units/hr).
///
/// Tiered brackets:
/// - 0–10 WR/hr: 5 PR per WR/hr per day
/// - 10–25 WR/hr: 10 PR per WR/hr per day
/// - 25+ WR/hr: 15 PR per WR/hr per day
pub fn wr_to_pr_per_day(wr_per_hour: f64) -> u32 {
    if wr_per_hour <= 0.0 {
        return 0;
    }

    let mut pr = 0.0;
    let mut remaining = wr_per_hour;

    // Bracket 1: 0–10 at 5 PR per WR/hr
    let b1 = remaining.min(10.0);
    pr += b1 * 5.0;
    remaining -= b1;

    // Bracket 2: 10–25 at 10 PR per WR/hr
    if remaining > 0.0 {
        let b2 = remaining.min(15.0);
        pr += b2 * 10.0;
        remaining -= b2;
    }

    // Bracket 3: 25+ at 15 PR per WR/hr
    if remaining > 0.0 {
        pr += remaining * 15.0;
    }

    pr.round() as u32
}
```

**Step 4: Run tests**

Run: `cargo test --lib loom::logic::tests::test_wr_to_pr_per_day`
Expected: PASS

**Step 5: Add the tick function**

In `src/loom/logic.rs`, add the per-tick WR→PR grant function. This follows the `tick_power_cores()` pattern using wall-clock time:

```rust
/// Tick WR→PR conversion. Called from tick_loom() each game tick.
///
/// Only active when all 28 patterns are complete. Reads the WR production
/// rate from the rate tracker, calculates PR/day, and grants PR at the
/// appropriate wall-clock interval.
///
/// Returns the number of PR granted this tick (0 in most ticks).
pub fn tick_wr_to_pr(
    loom: &LoomState,
    state: &mut crate::core::game_state::GameState,
) -> u32 {
    if !crate::loom::all_patterns_complete(&loom.persistent) {
        return 0;
    }

    // Get current WR production rate from rate tracker.
    let wr_rate = loom
        .rate_trackers
        .get(&Resource::WovenReality)
        .map(|t| t.rate_per_hour())
        .unwrap_or(0.0);

    let pr_per_day = wr_to_pr_per_day(wr_rate);
    if pr_per_day == 0 {
        return 0;
    }

    // Use the same wall-clock interval pattern as Power Cores.
    // PR is granted once per fill cycle: 86400 / pr_per_day seconds.
    let fill_secs = 86400i64 / pr_per_day as i64;
    let now = chrono::Utc::now().timestamp();
    let last = loom.persistent.wr_pr_last_granted_at;

    if last == 0 {
        // First tick with WR→PR active — don't grant, just initialise.
        return 0; // Caller sets the timestamp.
    }

    let elapsed = now - last;
    if elapsed < fill_secs {
        return 0;
    }

    let completed_cycles = (elapsed / fill_secs) as u32;
    state.prestige_rank = state.prestige_rank.saturating_add(completed_cycles);
    state.recalculate_prestige_bonuses();
    state.derived_stats_dirty = true;

    completed_cycles
}
```

**Step 6: Add persistence field**

In `src/loom/types.rs`, add to `LoomPersistent`:

```rust
/// Unix timestamp of last WR→PR grant (wall-clock, like Power Cores).
#[serde(default)]
pub wr_pr_last_granted_at: i64,
```

And in the `Default` impl:

```rust
wr_pr_last_granted_at: 0,
```

**Step 7: Add TickEvent variant**

In `src/core/tick_types.rs`, add a new variant to the `TickEvent` enum:

```rust
/// Woven Reality production granted prestige ranks.
WovenRealityPRGranted { pr_amount: u32, wr_rate: f64 },
```

**Step 8: Wire into tick_loom()**

In `src/core/tick_stages.rs`, at the end of `tick_loom()` (after the pattern sustain block), add:

```rust
// Tick WR→PR conversion (active after all 28 patterns complete).
if crate::loom::all_patterns_complete(&loom.persistent) {
    let now = chrono::Utc::now().timestamp();
    // Initialise timestamp on first tick.
    if loom.persistent.wr_pr_last_granted_at == 0 {
        loom.persistent.wr_pr_last_granted_at = now;
        result.loom_changed = true;
    }

    let wr_rate = loom
        .rate_trackers
        .get(&crate::loom::Resource::WovenReality)
        .map(|t| t.rate_per_hour())
        .unwrap_or(0.0);

    let pr_per_day = crate::loom::wr_to_pr_per_day(wr_rate);
    if pr_per_day > 0 {
        let fill_secs = 86400i64 / pr_per_day as i64;
        let last = loom.persistent.wr_pr_last_granted_at;
        let elapsed = now - last;
        if elapsed >= fill_secs {
            let completed_cycles = (elapsed / fill_secs) as u32;
            // Need GameState access — this function currently doesn't have it.
            // We'll need to add state parameter to tick_loom() — see integration note below.
        }
    }
}
```

**Integration note:** `tick_loom()` currently takes `(deep, loom, result)`. It needs `state: &mut GameState` to grant PR. Update the signature and the call in `src/core/tick.rs`:

In `src/core/tick_stages.rs`, change:
```rust
pub(super) fn tick_loom(
    deep: &crate::deep::DeepState,
    loom: &mut crate::loom::LoomState,
    state: &mut crate::core::game_state::GameState,
    result: &mut TickResult,
)
```

In `src/core/tick.rs`, update the call:
```rust
tick_stages::tick_loom(ctx.deep, ctx.loom, ctx.state, &mut result);
```

The full WR→PR grant logic in `tick_loom()`:

```rust
// Tick WR→PR conversion (active after all 28 patterns complete).
if crate::loom::all_patterns_complete(&loom.persistent) {
    let now = chrono::Utc::now().timestamp();
    if loom.persistent.wr_pr_last_granted_at == 0 {
        loom.persistent.wr_pr_last_granted_at = now;
        result.loom_changed = true;
    } else {
        let wr_rate = loom
            .rate_trackers
            .get(&crate::loom::Resource::WovenReality)
            .map(|t| t.rate_per_hour())
            .unwrap_or(0.0);
        let pr_per_day = crate::loom::wr_to_pr_per_day(wr_rate);
        if pr_per_day > 0 {
            let fill_secs = 86400i64 / pr_per_day as i64;
            let last = loom.persistent.wr_pr_last_granted_at;
            let elapsed = now - last;
            if elapsed >= fill_secs {
                let completed_cycles = (elapsed / fill_secs) as u32;
                state.prestige_rank = state.prestige_rank.saturating_add(completed_cycles);
                state.recalculate_prestige_bonuses();
                state.derived_stats_dirty = true;
                loom.persistent.wr_pr_last_granted_at = last + fill_secs * completed_cycles as i64;
                for _ in 0..completed_cycles {
                    result.events.push(TickEvent::WovenRealityPRGranted {
                        pr_amount: 1,
                        wr_rate,
                    });
                }
                result.loom_changed = true;
            }
        }
    }
}
```

**Step 9: Add re-export**

In `src/loom/mod.rs`:
```rust
pub use logic::wr_to_pr_per_day;
```

**Step 10: Run all tests**

Run: `cargo test`
Expected: PASS

**Step 11: Commit**

```bash
git add src/loom/logic.rs src/loom/types.rs src/loom/mod.rs src/core/tick_types.rs src/core/tick_stages.rs src/core/tick.rs
git commit -m "feat(loom): add WR→PR tiered bracket generation system"
```

---

### Task 5: Add Loom Zones (Z31–50) data

Extend the zone data table with 20 new Loom-themed zones. Add zone definitions, enemy stat scaling at 1.25x per zone, and the `LOOM_ZONE_STAT_MULTIPLIER` constant.

**Files:**
- Modify: `src/core/constants.rs`
- Modify: `src/zones/data.rs`

**Step 1: Write the failing tests**

Add to `src/core/constants.rs` tests:

```rust
#[test]
fn test_zone_enemy_stats_has_50_entries() {
    assert_eq!(ZONE_ENEMY_STATS.len(), 50);
}

#[test]
fn test_loom_zone_stat_multiplier() {
    assert!((LOOM_ZONE_STAT_MULTIPLIER - 1.25).abs() < 1e-10);
}

#[test]
fn test_loom_zone_constants() {
    assert_eq!(FIRST_LOOM_ZONE_ID, 31);
    assert_eq!(LAST_LOOM_ZONE_ID, 50);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib core::constants::tests`
Expected: FAIL — array has 30 entries, constants don't exist.

**Step 3: Write the implementation**

In `src/core/constants.rs`:

1. Add new constants:

```rust
pub const FIRST_LOOM_ZONE_ID: u32 = 31;
pub const LAST_LOOM_ZONE_ID: u32 = 50;
pub const LOOM_ZONE_STAT_MULTIPLIER: f64 = 1.25;
```

2. Extend `ZONE_ENEMY_STATS` from 30 to 50 entries. Calculate each zone's stats as `Zone30_base × 1.25^(zone_id - 30)`. Zone 30 base stats are `(37778883, 3022365, 3777930, 604515, 1888978, 226688)`.

Zone 31 = Zone 30 × 1.25:
```
(47223604, 3777956, 4722413, 755644, 2361223, 283360)
```

Continue the pattern for all 20 zones. Use a comment block:

```rust
// Loom Zones — 1.25x exponential scaling from Zone 30
(47_223_604, 3_777_956, 4_722_413, 755_644, 2_361_223, 283_360),  // Zone 31
(59_029_505, 4_722_445, 5_903_016, 944_555, 2_951_529, 354_200),  // Zone 32
// ... (calculate all 20 entries)
```

**Important**: Use `u32` max (4,294,967,295) as a ceiling. Some stats for very high zones may exceed u32 range — use u64 if needed, OR cap at u32::MAX. Check whether the existing enemy generation code uses u32 — it does (see `calc_zone_enemy_stats` returning `(u32, u32, u32)`). Zone 50 at 1.25^20 ≈ 86.7x Zone 30 base:
- HP: 37778883 × 86.7 ≈ 3.27 billion (fits u32 max of 4.29 billion)
- DMG: 3777930 × 86.7 ≈ 327 million (fits u32)

All values fit within u32. Calculate all 20 entries precisely:

For each zone z (31–50): `stat = (Zone30_stat as f64 × 1.25^(z-30)).round() as u32`

3. Fix the existing test `test_zone_enemy_stats_has_30_entries` — change to 50.

**Step 4: Add zone definitions in `src/zones/data.rs`**

Add 20 new zone entries to the `ALL_ZONES` LazyLock vec. Each zone needs:
- `id: 31..=50`
- `name` and `description` — Loom-themed names (woven realms)
- `subzones: Vec<Subzone>` — 5 subzones each (like fracture zones)
- `prestige_requirement` — progressive: P400 (Z31-34), P500 (Z35-38), P600 (Z39-42), P700 (Z43-46), P800 (Z47-50)
- `min_level` and `max_level` — continuing from Z30
- `requires_weapon: false`
- `weapon_name: None`

Loom-themed zone names (20 zones across 5 chapters):

**Ch.7: The Thread Wilds (Z31-34)**
- Z31: Threadbare Wastes
- Z32: Spindle Hollow
- Z33: The Weft Expanse
- Z34: Heart of the Thread Wilds

**Ch.8: The Woven Frontier (Z35-38)**
- Z35: Loom's Edge
- Z36: Shuttle Run
- Z37: The Pattern Gate
- Z38: Heart of the Woven Frontier

**Ch.9: The Unraveling (Z39-42)**
- Z39: Frayed Reaches
- Z40: The Loose Ends
- Z41: Tangle of Fates
- Z42: Heart of the Unraveling

**Ch.10: The Grand Design (Z43-46)**
- Z43: The Blueprint Halls
- Z44: Architect's Loom
- Z45: Tapestry of Stars
- Z46: Heart of the Grand Design

**Ch.11: The Final Weave (Z47-50)**
- Z47: The Last Shuttle
- Z48: Reality's Seam
- Z49: The World Loom
- Z50: The Origin Thread

**Step 5: Run tests**

Run: `cargo test`
Expected: PASS

**Step 6: Commit**

```bash
git add src/core/constants.rs src/zones/data.rs
git commit -m "feat(zones): add 20 Loom Zones (Z31-50) with 1.25x stat scaling"
```

---

### Task 6: Add Loom Zone unlock gating

Extend `sync_account_zone_unlocks` to handle Loom zone access based on completed pattern count. Add a `loom_zone_cap` concept similar to `fracture_zone_cap`.

**Files:**
- Modify: `src/zones/access.rs`
- Modify: `src/loom/logic.rs` (add `loom_zone_cap_for_patterns()`)
- Modify: `src/loom/mod.rs`

**Step 1: Write the failing tests**

Add to `src/loom/logic.rs` tests:

```rust
#[test]
fn test_loom_zone_cap_for_patterns() {
    assert_eq!(loom_zone_cap_for_patterns(0), 30);  // No Loom zones
    assert_eq!(loom_zone_cap_for_patterns(3), 30);  // Not enough
    assert_eq!(loom_zone_cap_for_patterns(4), 34);  // First tier
    assert_eq!(loom_zone_cap_for_patterns(7), 34);  // Still first tier
    assert_eq!(loom_zone_cap_for_patterns(8), 38);  // Second tier
    assert_eq!(loom_zone_cap_for_patterns(15), 38); // Still second tier
    assert_eq!(loom_zone_cap_for_patterns(16), 42); // Third tier
    assert_eq!(loom_zone_cap_for_patterns(21), 42); // Still third tier
    assert_eq!(loom_zone_cap_for_patterns(22), 46); // Fourth tier
    assert_eq!(loom_zone_cap_for_patterns(27), 46); // Still fourth tier
    assert_eq!(loom_zone_cap_for_patterns(28), 50); // All patterns = all zones
}
```

Add to `src/zones/access.rs` tests:

```rust
#[test]
fn test_sync_unlocks_loom_zones_31_to_34() {
    let mut prog = ZoneProgression::new();
    sync_account_zone_unlocks(&mut prog, true, 30, 400, 34);
    assert!(prog.is_zone_unlocked(31));
    assert!(prog.is_zone_unlocked(34));
    assert!(!prog.is_zone_unlocked(35));
}

#[test]
fn test_sync_does_not_unlock_loom_zones_beyond_cap() {
    let mut prog = ZoneProgression::new();
    sync_account_zone_unlocks(&mut prog, true, 30, 800, 34);
    assert!(prog.is_zone_unlocked(34));
    assert!(!prog.is_zone_unlocked(35));
}
```

**Step 2: Run tests to verify they fail**

Expected: FAIL — functions don't exist.

**Step 3: Write the implementation**

In `src/loom/logic.rs`:

```rust
/// Returns the highest zone ID unlocked by the given completed pattern count.
///
/// | Patterns | Zones Unlocked |
/// |----------|----------------|
/// | 4        | Z31–34         |
/// | 8        | Z35–38         |
/// | 16       | Z39–42         |
/// | 22       | Z43–46         |
/// | 28       | Z47–50         |
pub fn loom_zone_cap_for_patterns(completed_patterns: usize) -> u32 {
    if completed_patterns >= 28 {
        50
    } else if completed_patterns >= 22 {
        46
    } else if completed_patterns >= 16 {
        42
    } else if completed_patterns >= 8 {
        38
    } else if completed_patterns >= 4 {
        34
    } else {
        30 // No Loom zones
    }
}
```

In `src/loom/mod.rs`:
```rust
pub use logic::loom_zone_cap_for_patterns;
```

In `src/zones/access.rs`, add a `loom_zone_cap` parameter to `sync_account_zone_unlocks`:

```rust
pub fn sync_account_zone_unlocks(
    prog: &mut ZoneProgression,
    storms_end_unlocked: bool,
    fracture_zone_cap: u32,
    prestige_rank: u32,
    loom_zone_cap: u32,
) {
    // ... existing logic for zones 11 and 12..=fracture_zone_cap ...

    // Loom zones 31..=loom_zone_cap
    let zones = crate::zones::data::get_all_zones();
    for zone_id in 31..=loom_zone_cap {
        if let Some(zone) = zones.iter().find(|z| z.id == zone_id) {
            if prestige_rank >= zone.prestige_requirement {
                prog.unlock_zone(zone_id);
            }
        }
    }
}
```

**Step 4: Fix all callers of `sync_account_zone_unlocks`**

Search for call sites and add the `loom_zone_cap` parameter. The call sites need access to Loom state to compute `loom_zone_cap_for_patterns(loom.persistent.completed_pattern_count())`. Initially, callers without Loom access can pass `30` (no Loom zones).

The main call site in `tick_stages.rs` (fracture region unlock) already has access to `loom` — pass the computed cap.

**Step 5: Wire zone cap computation into tick_loom**

In `tick_stages.rs`, when a pattern completes, recompute the zone cap and call `sync_account_zone_unlocks` if it changed:

```rust
if pattern_completed {
    result.loom_changed = true;
    let new_cap = crate::loom::loom_zone_cap_for_patterns(
        loom.persistent.completed_pattern_count()
    );
    // Store and compare — if changed, sync zone unlocks
    // This requires access to state.zone_progression — add state param
}
```

**Step 6: Run all tests**

Run: `cargo test`
Expected: PASS

**Step 7: Commit**

```bash
git add src/loom/logic.rs src/loom/mod.rs src/zones/access.rs src/core/tick_stages.rs
git commit -m "feat(zones): add Loom zone unlock gating (Z31-50 via pattern milestones)"
```

---

### Task 7: Add Loom zone cycling (boss defeat behavior)

Extend boss defeat logic so Loom cap zones cycle like fracture cap zones. Add a `LoomZoneCycle` variant to `BossDefeatResult`.

**Files:**
- Modify: `src/zones/boss_defeat.rs`
- Modify: `src/zones/advancement.rs` (if boss defeat flow goes through here)

**Step 1: Write the failing tests**

Add to `src/zones/boss_defeat.rs` tests:

```rust
#[test]
fn test_loom_zone_cap_cycles() {
    let mut prog = ZoneProgression::new();
    // Unlock zones up to 34
    for z in 1..=34 {
        prog.unlock_zone(z);
    }
    prog.current_zone_id = 34;
    prog.current_subzone_id = 5; // Last subzone
    prog.fighting_boss = true;

    let result = on_boss_defeated_with_cap(
        &mut prog,
        400,
        &mut Achievements::default(),
        30,  // fracture_zone_cap
        34,  // loom_zone_cap
    );

    // Zone 34 is the Loom cap — should cycle back to subzone 1
    assert!(matches!(result, BossDefeatResult::LoomZoneCycle { zone_id: 34 }));
    assert_eq!(prog.current_subzone_id, 1);
}
```

**Step 2: Run to verify failure**

**Step 3: Implement**

Add `LoomZoneCycle { zone_id: u32 }` variant to `BossDefeatResult` enum.

In `on_boss_defeated_with_cap()`, add a `loom_zone_cap: u32` parameter. After the fracture cycling check, add:

```rust
// If this is the Loom zone cap, cycle back to subzone 1
if zone_id >= 31 && zone_id == loom_zone_cap && zone_id <= 50 {
    prog.current_subzone_id = 1;
    prog.kills_in_subzone = 0;
    prog.fighting_boss = false;
    return BossDefeatResult::LoomZoneCycle { zone_id };
}
```

**Step 4: Fix all callers** of `on_boss_defeated_with_cap()` to pass `loom_zone_cap`.

**Step 5: Run tests, commit**

```bash
git add src/zones/boss_defeat.rs src/zones/mod.rs
git commit -m "feat(zones): add Loom zone cycling for cap zones"
```

---

### Task 8: Update Ascension UI for Loom gates

Show pattern requirement for Ascension VII–X in the confirmation dialog. Show "Requires: N Woven Patterns" instead of "No Deep requirement" for levels 7+.

**Files:**
- Modify: `src/ui/ascension_scene.rs`

**Step 1: Update render function signature**

Add `loom: &crate::loom::LoomState` to `render_ascension_confirm()`.

**Step 2: Add pattern gate display**

After the Deep gate display block, add:

```rust
if let Some(required_patterns) = ascension_pattern_gate(next_level) {
    let current_patterns = loom.persistent.completed_pattern_count();
    let met = current_patterns >= required_patterns;
    let color = if met { Color::Green } else { Color::Red };
    lines.push(Line::from(vec![
        Span::styled("Requires: ", Style::default().fg(Color::White)),
        Span::styled(
            format!("{} Woven Patterns", required_patterns),
            Style::default().fg(color),
        ),
        Span::styled(
            format!("  (completed: {})", current_patterns),
            Style::default().fg(Color::DarkGray),
        ),
    ]));
}
```

**Step 3: Fix all callers** that call `render_ascension_confirm()` — pass the Loom state.

**Step 4: Run `cargo test`, verify compilation**

**Step 5: Commit**

```bash
git add src/ui/ascension_scene.rs
git commit -m "feat(ui): show Woven Pattern gate in Ascension dialog for VII-X"
```

---

### Task 9: Update input handling for ascension with pattern gates

Update the ascension input handler to pass `completed_patterns` to `can_ascend()` and `ascend()`.

**Files:**
- Search for: `grep -rn "can_ascend\|ascend(" src/input/ --include="*.rs"`
- Modify: whichever input file handles the ascension 'Y' key press

**Step 1: Find the call site**

Run: `grep -rn "ascend(" src/input/ --include="*.rs"`

**Step 2: Update the call**

Pass `loom.persistent.completed_pattern_count()` as the `completed_patterns` parameter to both `can_ascend()` and `ascend()`.

**Step 3: Run tests, commit**

```bash
git commit -m "feat(input): pass completed patterns to ascension gating"
```

---

### Task 10: Add Loom zone enemy name prefixes/suffixes

Add Loom-themed enemy name generation for zones 31–50 so combat feels distinct.

**Files:**
- Modify: `src/combat/enemy_generation.rs`

**Step 1: Add name data**

In `get_zone_enemy_prefixes()`, add match arms for zones 31–50:

```rust
31..=34 => &["Threadbare", "Woven", "Spindle", "Weft", "Loom"],
35..=38 => &["Shuttle", "Pattern", "Weave", "Fabric", "Tapestry"],
39..=42 => &["Frayed", "Unraveled", "Tangled", "Knotted", "Snarled"],
43..=46 => &["Grand", "Architect", "Blueprint", "Design", "Schema"],
47..=50 => &["Final", "Origin", "Reality", "World", "Infinite"],
```

Similarly for `get_zone_enemy_suffixes()`:

```rust
31..=34 => &["Weaver", "Spinner", "Threader", "Bobbin", "Shuttle"],
35..=38 => &["Loomguard", "Weftwalker", "Patternborn", "Fabricant", "Threadseeker"],
39..=42 => &["Unmaker", "Raveler", "Tanglefoe", "Knotter", "Splicer"],
43..=46 => &["Architect", "Designer", "Schemer", "Artificer", "Crafter"],
47..=50 => &["Worldweaver", "Realityborn", "Originkeeper", "Threadmaster", "Loombinder"],
```

**Step 2: Run tests, commit**

```bash
git add src/combat/enemy_generation.rs
git commit -m "feat(combat): add Loom zone enemy name prefixes/suffixes for Z31-50"
```

---

### Task 11: Update CLAUDE.md documentation

Update the module documentation files to reflect all new systems.

**Files:**
- Modify: `src/ascension/CLAUDE.md`
- Modify: `src/loom/CLAUDE.md`
- Modify: `src/zones/CLAUDE.md`
- Modify: `CLAUDE.md` (root)

**Step 1: Update Ascension CLAUDE.md**

Add Ascension VII–X to the table:

```markdown
| VII | 8 Patterns | 1,500 PR | 96x |
| VIII | 16 Patterns | 4,000 PR | 144x |
| IX | 22 Patterns | 8,000 PR | 216x |
| X | 28 Patterns | 15,000 PR | 324x |
```

Add `ascension_pattern_gate(level)` and `max_shuttle_level(ascension_level)` to the Key Functions section. Update MAX_ASCENSION_LEVEL to 10.

**Step 2: Update Loom CLAUDE.md**

Add sections for:
- Shuttle upgrades (level multiplier on intake cap, Ascension-gated level caps)
- WR→PR generation (tiered brackets, activation condition)
- `completed_pattern_count()`, `loom_zone_cap_for_patterns()`, `wr_to_pr_per_day()`, `upgrade_shuttle()`, `shuttle_effective_intake_cap()`

**Step 3: Update Zones CLAUDE.md**

Add Loom Zones section:
- Ch.7–11 zone names and prestige requirements
- 1.25x stat scaling multiplier
- Pattern-gated unlock table
- `LoomZoneCycle` boss defeat result

**Step 4: Update root CLAUDE.md**

Add key constants:
- Loom Zone stat multiplier: 1.25x
- Ascension VII–X costs and multipliers
- WR→PR brackets
- Shuttle level caps per Ascension tier

**Step 5: Commit**

```bash
git add src/ascension/CLAUDE.md src/loom/CLAUDE.md src/zones/CLAUDE.md CLAUDE.md
git commit -m "docs: update module docs for Loom power integration"
```

---

### Task 12: Run full CI checks

**Step 1: Run `make check`**

```bash
make check
```

Expected: All checks pass (format, clippy, tests, build, audit).

**Step 2: Fix any issues found**

Address any compilation errors, clippy warnings, or test failures.

**Step 3: Commit fixes if needed**

```bash
git commit -m "fix: address CI check issues from Loom power integration"
```
