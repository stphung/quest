# Loom of Worlds — Sustained Rate Pattern Redesign Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the accumulated-totals pattern system with sustained flow rate tracking, expand from 18 to 28 patterns, fix extractor buffer overflow, and shift tier gates.

**Architecture:** Add a `RateTracker` struct for 60-second rolling window rate measurement. Redesign `PatternRequirement` to use rate thresholds and sustain durations instead of accumulated amounts. Each requirement completes independently when its sustain timer reaches the required duration. Fix extractors to auto-drain excess production instead of stalling.

**Tech Stack:** Rust, serde (JSON persistence), Ratatui (UI)

---

## Summary of Changes

| Area | What changes |
|------|-------------|
| `types.rs` | New `RateTracker` struct, redesigned `PatternRequirement` with rate/duration fields |
| `patterns.rs` | Rewrite `tick_pattern_sustain()` for rate-based sustain logic with pause model |
| `discovery.rs` | Replace 18 patterns with 28, using rate thresholds and durations from design doc |
| `logic.rs` | Fix extractor buffer overflow (auto-drain), shift tier gate thresholds |
| `tick_stages.rs` | Pass per-resource production amounts to rate tracker each tick |
| `loom_scene.rs` | Update pattern bar to show rate/duration/state instead of accumulated/amount |
| `mod.rs` | Re-export new types |
| `persistence.rs` | No structural changes (serde handles new fields via defaults) |
| `CLAUDE.md` | Update Loom module documentation |

---

### Task 1: Add RateTracker struct to types.rs

**Files:**
- Modify: `src/loom/types.rs`

**Context:** The `RateTracker` measures a 60-second rolling window average of production for a single resource. It uses a circular buffer of 600 ticks (at 100ms/tick = 60 seconds). The running sum gives O(1) per-tick updates. This struct is NOT serialized — on load, it starts empty and ramps up over 60 seconds (negligible).

**Step 1: Write the failing test**

Add to the `#[cfg(test)] mod tests` block at the bottom of `src/loom/types.rs`:

```rust
#[test]
fn test_rate_tracker_new_is_empty() {
    let tracker = RateTracker::new();
    assert!((tracker.rate_per_hour()).abs() < 1e-9);
}

#[test]
fn test_rate_tracker_push_single_value() {
    let mut tracker = RateTracker::new();
    // Push a single tick's production: 1.0 units produced in 0.1s
    tracker.push(1.0);
    // Window has 1 sample out of 600, so average per tick = 1.0/600
    // Rate per hour = (sum / window_size) * ticks_per_hour
    // = (1.0 / 600) * 36000 = 60.0/hr
    let rate = tracker.rate_per_hour();
    assert!((rate - 60.0).abs() < 1e-6, "rate was {}", rate);
}

#[test]
fn test_rate_tracker_full_window_steady() {
    let mut tracker = RateTracker::new();
    // Simulate 600 ticks at 50/hr = 50/36000 per tick ≈ 0.001389 per tick
    let per_tick = 50.0 / 36000.0;
    for _ in 0..600 {
        tracker.push(per_tick);
    }
    let rate = tracker.rate_per_hour();
    assert!((rate - 50.0).abs() < 0.1, "rate was {}", rate);
}

#[test]
fn test_rate_tracker_evicts_old_values() {
    let mut tracker = RateTracker::new();
    // Fill 600 ticks with high production
    for _ in 0..600 {
        tracker.push(1.0);
    }
    // Now push 600 ticks of zero — should evict all old values
    for _ in 0..600 {
        tracker.push(0.0);
    }
    assert!((tracker.rate_per_hour()).abs() < 1e-9);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib loom::types::tests::test_rate_tracker -- --nocapture`
Expected: FAIL — `RateTracker` type does not exist yet.

**Step 3: Write minimal implementation**

Add above the `#[cfg(test)]` block in `src/loom/types.rs`:

```rust
/// 60-second rolling window rate tracker.
///
/// Measures production rate using a circular buffer of the last 600 ticks
/// (at 100ms/tick = 60 seconds). Maintains a running sum for O(1) updates.
///
/// Not serialized — reconstructed from scratch on load (60s ramp-up is negligible).
#[derive(Debug, Clone)]
pub struct RateTracker {
    buffer: std::collections::VecDeque<f64>,
    sum: f64,
}

const RATE_WINDOW_SIZE: usize = 600; // 600 ticks × 0.1s = 60 seconds
const TICKS_PER_HOUR: f64 = 36_000.0; // 3600s / 0.1s

impl RateTracker {
    pub fn new() -> Self {
        Self {
            buffer: std::collections::VecDeque::with_capacity(RATE_WINDOW_SIZE),
            sum: 0.0,
        }
    }

    /// Push one tick's production amount into the window.
    pub fn push(&mut self, amount: f64) {
        if self.buffer.len() >= RATE_WINDOW_SIZE {
            self.sum -= self.buffer.pop_front().unwrap_or(0.0);
        }
        self.buffer.push_back(amount);
        self.sum += amount;
    }

    /// Current production rate in units/hour, averaged over the 60-second window.
    pub fn rate_per_hour(&self) -> f64 {
        // Average production per tick × ticks per hour
        (self.sum / RATE_WINDOW_SIZE as f64) * TICKS_PER_HOUR
    }
}

impl Default for RateTracker {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib loom::types::tests::test_rate_tracker -- --nocapture`
Expected: PASS — all 4 tests pass.

**Step 5: Commit**

```bash
git add src/loom/types.rs
git commit -m "feat(loom): add RateTracker for 60-second rolling window rate measurement"
```

---

### Task 2: Redesign PatternRequirement for sustained rates

**Files:**
- Modify: `src/loom/types.rs`

**Context:** Currently `PatternRequirement` has `amount` (total needed) and `accumulated` (progress so far). We need to replace this with rate-based fields: `required_rate` (units/hr threshold), `sustain_duration_secs` (total seconds the rate must be sustained), and `sustained_secs` (seconds sustained so far). The `accumulated` and `amount` fields are removed. The `RateTracker` is transient (not serialized). Each requirement also gets a `completed` flag so requirements can complete independently.

**Important serde migration note:** Old saves have `amount` and `accumulated` fields. Use `#[serde(default)]` on all new fields so old saves load without crashing. The old `amount` field should be kept with `#[serde(default)]` so old JSON doesn't fail deserialization — it just won't be used.

**Step 1: Write the failing test**

Add to `src/loom/types.rs` tests:

```rust
#[test]
fn test_pattern_requirement_rate_fields() {
    let req = PatternRequirement {
        resource: Resource::Ember,
        required_rate: 25.0,
        sustain_duration_secs: 7200.0,
        sustained_secs: 0.0,
        completed: false,
        // Legacy fields for serde compat
        amount: 0.0,
        accumulated: 0.0,
    };
    assert_eq!(req.resource, Resource::Ember);
    assert!((req.required_rate - 25.0).abs() < 1e-9);
    assert!((req.sustain_duration_secs - 7200.0).abs() < 1e-9);
    assert!((req.sustained_secs).abs() < 1e-9);
    assert!(!req.completed);
}

#[test]
fn test_pattern_requirement_completes_independently() {
    let mut req = PatternRequirement {
        resource: Resource::Ember,
        required_rate: 25.0,
        sustain_duration_secs: 100.0,
        sustained_secs: 100.0,
        completed: true,
        amount: 0.0,
        accumulated: 0.0,
    };
    assert!(req.completed);
    assert!(req.sustained_secs >= req.sustain_duration_secs);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib loom::types::tests::test_pattern_requirement_rate -- --nocapture`
Expected: FAIL — fields `required_rate`, `sustain_duration_secs`, `sustained_secs`, `completed` don't exist.

**Step 3: Write minimal implementation**

Replace the `PatternRequirement` struct in `src/loom/types.rs`:

```rust
/// A single requirement within a woven pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternRequirement {
    pub resource: Resource,
    /// Minimum production rate (units/hr) that must be sustained.
    #[serde(default)]
    pub required_rate: f64,
    /// Total seconds the rate must be sustained to complete this requirement.
    #[serde(default)]
    pub sustain_duration_secs: f64,
    /// Seconds sustained so far (timer advances when rate >= threshold, pauses otherwise).
    #[serde(default)]
    pub sustained_secs: f64,
    /// Whether this individual requirement is complete (locks when sustain timer finishes).
    #[serde(default)]
    pub completed: bool,
    /// Legacy field — total amount needed (accumulated totals system). Kept for serde compat.
    #[serde(default, alias = "rate_per_hour")]
    pub amount: f64,
    /// Legacy field — accumulated production so far. Kept for serde compat.
    #[serde(default)]
    pub accumulated: f64,
}
```

**Step 4: Fix all compilation errors**

After changing `PatternRequirement`, code in `patterns.rs`, `discovery.rs`, `loom_scene.rs`, and tests will fail to compile. Fix each by adding the new fields with defaults. This is expected — subsequent tasks will properly update the logic. For now, just make it compile:

- In `discovery.rs` `pattern()` helper: add `required_rate: 0.0, sustain_duration_secs: 0.0, sustained_secs: 0.0, completed: false` to `PatternRequirement` construction (the old 18 patterns won't use these yet — Task 5 replaces them entirely).
- In test files: add the new fields where `PatternRequirement` is constructed directly.

**Step 5: Run all loom tests to verify they pass**

Run: `cargo test --lib loom:: -- --nocapture`
Expected: PASS — all existing tests should still pass with the added fields.

**Step 6: Commit**

```bash
git add src/loom/types.rs src/loom/discovery.rs src/loom/patterns.rs
git commit -m "feat(loom): add rate-based fields to PatternRequirement"
```

---

### Task 3: Rewrite tick_pattern_sustain() for sustained rate logic

**Files:**
- Modify: `src/loom/patterns.rs`

**Context:** The current `tick_pattern_sustain()` accumulates `rate * delta_hours` per tick. The new version should:
1. For each non-completed requirement in the active pattern:
   - Look up the current measured rate for that resource (from a `HashMap<Resource, f64>` of per-resource rates).
   - If rate >= `required_rate`: advance `sustained_secs` by `delta_seconds`.
   - If rate < `required_rate`: do nothing (simple pause — no decay).
   - If `sustained_secs >= sustain_duration_secs`: mark requirement `completed = true`.
2. If ALL requirements are `completed`: mark the pattern as completed and advance to the next one.

The function signature changes: the `rates` parameter now contains **measured rates** (from RateTracker), not raw per-tick production amounts.

**Step 1: Write the failing tests**

Replace the accumulator tests in `src/loom/patterns.rs` with rate-based tests. Keep the existing test helper functions. Add new tests:

```rust
#[test]
fn test_sustain_advances_when_rate_meets_threshold() {
    let mut state = state_with_patterns();
    // Pattern 0 requires Ember 25/hr for some duration.
    // Provide rate of 30/hr (above threshold).
    let r = rates(&[(Resource::Ember, 30.0)]);
    tick_pattern_sustain(&mut state.persistent, &r, 1.0);
    let req = &state.persistent.patterns[0].requirements[0];
    assert!(req.sustained_secs > 0.0, "should advance when rate >= threshold");
    assert!((req.sustained_secs - 1.0).abs() < 1e-9, "should advance by delta_seconds");
}

#[test]
fn test_sustain_pauses_when_rate_below_threshold() {
    let mut state = state_with_patterns();
    // Pre-set some sustained progress.
    state.persistent.patterns[0].requirements[0].sustained_secs = 100.0;
    // Provide rate below threshold.
    let r = rates(&[(Resource::Ember, 10.0)]); // below 25/hr
    tick_pattern_sustain(&mut state.persistent, &r, 1.0);
    let req = &state.persistent.patterns[0].requirements[0];
    assert!((req.sustained_secs - 100.0).abs() < 1e-9, "should not advance when rate < threshold");
}

#[test]
fn test_sustain_never_decays() {
    let mut state = state_with_patterns();
    state.persistent.patterns[0].requirements[0].sustained_secs = 50.0;
    let r = rates(&[]); // zero rate
    tick_pattern_sustain(&mut state.persistent, &r, 10.0);
    let req = &state.persistent.patterns[0].requirements[0];
    assert!(req.sustained_secs >= 50.0, "sustained_secs must never decrease");
}

#[test]
fn test_requirement_completes_when_duration_reached() {
    let mut state = state_with_patterns();
    let req = &mut state.persistent.patterns[0].requirements[0];
    req.sustained_secs = req.sustain_duration_secs - 0.5;
    let r = rates(&[(Resource::Ember, 100.0)]); // well above threshold
    tick_pattern_sustain(&mut state.persistent, &r, 1.0);
    assert!(state.persistent.patterns[0].requirements[0].completed);
}

#[test]
fn test_pattern_completes_when_all_requirements_complete() {
    let mut state = state_with_patterns();
    // Set all requirements to just below completion.
    for req in &mut state.persistent.patterns[0].requirements {
        req.sustained_secs = req.sustain_duration_secs - 0.1;
    }
    // Provide high rates for all required resources.
    let r = rates(&[(Resource::Ember, 100.0)]);
    let completed = tick_pattern_sustain(&mut state.persistent, &r, 1.0);
    assert!(completed);
    assert!(state.persistent.patterns[0].completed);
}

#[test]
fn test_requirement_independent_completion() {
    let mut state = state_with_patterns();
    // Skip to a multi-requirement pattern (pattern index 4 = "Mirror and Void" in new set,
    // but for now use whatever multi-req pattern exists).
    // We'll test this properly after Task 5 replaces patterns.
    // For now, just verify the single-req pattern 0 works.
    let req = &mut state.persistent.patterns[0].requirements[0];
    req.sustained_secs = req.sustain_duration_secs;
    req.completed = true;
    assert!(active_pattern_requirements_met(&state.persistent));
}

#[test]
fn test_already_completed_requirement_not_advanced() {
    let mut state = state_with_patterns();
    state.persistent.patterns[0].requirements[0].completed = true;
    state.persistent.patterns[0].requirements[0].sustained_secs = 100.0;
    let r = rates(&[(Resource::Ember, 100.0)]);
    tick_pattern_sustain(&mut state.persistent, &r, 1.0);
    // Should not advance past where it was.
    assert!((state.persistent.patterns[0].requirements[0].sustained_secs - 100.0).abs() < 1e-9);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib loom::patterns::tests -- --nocapture`
Expected: FAIL — the current implementation accumulates `rate * delta_hours` instead of advancing a sustain timer.

**Step 3: Rewrite tick_pattern_sustain()**

Replace the function body in `src/loom/patterns.rs`:

```rust
/// Tick the sustain timer for the active pattern.
///
/// Called once per game tick. `delta_seconds` is wall-clock time elapsed
/// since the last tick (typically 0.1s for a 100ms tick interval).
/// `rates` maps each resource to its current measured production rate in units/hour.
///
/// For each non-completed requirement:
/// - If the measured rate >= required_rate: advance sustained_secs by delta_seconds.
/// - Otherwise: do nothing (simple pause — no decay).
/// - If sustained_secs >= sustain_duration_secs: mark requirement completed.
///
/// When ALL requirements are completed, the pattern completes.
///
/// Returns `true` if a pattern was completed during this tick.
pub fn tick_pattern_sustain(
    persistent: &mut LoomPersistent,
    rates: &HashMap<Resource, f64>,
    delta_seconds: f64,
) -> bool {
    let Some(pattern) = persistent.patterns.get_mut(persistent.active_pattern) else {
        return false;
    };
    if pattern.completed {
        return false;
    }

    for req in &mut pattern.requirements {
        if req.completed {
            continue;
        }
        let rate = rates.get(&req.resource).copied().unwrap_or(0.0);
        if rate >= req.required_rate {
            req.sustained_secs += delta_seconds;
            if req.sustained_secs >= req.sustain_duration_secs {
                req.sustained_secs = req.sustain_duration_secs;
                req.completed = true;
            }
        }
        // Simple pause: do nothing when rate < threshold. No decay.
    }

    if pattern.requirements.iter().all(|req| req.completed) {
        complete_active_pattern(persistent);
        return true;
    }
    false
}
```

Also update `active_pattern_requirements_met()`:

```rust
pub fn active_pattern_requirements_met(persistent: &LoomPersistent) -> bool {
    let Some(pattern) = persistent.patterns.get(persistent.active_pattern) else {
        return false;
    };
    if pattern.completed {
        return false;
    }
    pattern.requirements.iter().all(|req| req.completed)
}
```

And update `active_pattern_requirement_status()` to return the new fields:

```rust
/// Returns `(sustained_secs, sustain_duration_secs, completed)` for each requirement.
pub fn active_pattern_requirement_status(
    persistent: &LoomPersistent,
) -> Vec<(f64, f64, bool)> {
    let Some(pattern) = persistent.patterns.get(persistent.active_pattern) else {
        return Vec::new();
    };
    pattern
        .requirements
        .iter()
        .map(|req| (req.sustained_secs, req.sustain_duration_secs, req.completed))
        .collect()
}
```

**Step 4: Update all existing tests that reference the old behavior**

Remove or rewrite tests that check `accumulated` or `amount` directly. The tests from Step 1 replace them. Keep structural tests like `test_advance_skips_already_completed_patterns` and `test_all_complete_when_every_pattern_marked`.

Update `active_pattern_requirement_status` tests to match the new return type `Vec<(f64, f64, bool)>`.

**Step 5: Run all loom tests**

Run: `cargo test --lib loom:: -- --nocapture`
Expected: PASS

**Step 6: Commit**

```bash
git add src/loom/patterns.rs
git commit -m "feat(loom): rewrite pattern sustain logic for rate-based tracking"
```

---

### Task 4: Fix extractor buffer overflow (auto-drain)

**Files:**
- Modify: `src/loom/logic.rs`

**Context:** Currently in `tick_base_production()`, when a node's buffer reaches capacity, the node stalls (`node.stalled = true; continue;`). This silently breaks sustained rate patterns because the extractor stops producing. The fix: always produce at full rate. If the buffer is full, discard the excess (auto-drain). The extractor never stalls due to a full buffer — the buffer is a reservoir, not a gate.

**Step 1: Write the failing test**

Add to `src/loom/logic.rs` tests (find the existing `mod tests` block):

```rust
#[test]
fn test_extractor_produces_at_full_rate_when_buffer_full() {
    let mut loom = LoomState::new();
    // Unlock the Ember Spindle.
    loom.persistent.nodes[0].unlocked = true;
    loom.persistent.nodes[0].buffer = loom.persistent.nodes[0].buffer_capacity; // full buffer

    let produced = tick_base_production(&mut loom, 0.1);

    // Should still report production (for rate tracking) even when buffer is full.
    let ember_produced = produced.get(&Resource::Ember).copied().unwrap_or(0.0);
    assert!(ember_produced > 0.0, "extractor should report production even with full buffer");
    // Node should NOT be marked stalled.
    assert!(!loom.persistent.nodes[0].stalled, "extractor should not stall from full buffer");
}

#[test]
fn test_extractor_buffer_does_not_exceed_capacity() {
    let mut loom = LoomState::new();
    loom.persistent.nodes[0].unlocked = true;
    loom.persistent.nodes[0].buffer = loom.persistent.nodes[0].buffer_capacity - 0.001;

    tick_base_production(&mut loom, 0.1);

    assert!(
        loom.persistent.nodes[0].buffer <= loom.persistent.nodes[0].buffer_capacity + 1e-9,
        "buffer should not exceed capacity"
    );
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib loom::logic::tests::test_extractor_produces_at_full_rate -- --nocapture`
Expected: FAIL — currently the node stalls and produces 0 when buffer is full.

**Step 3: Fix tick_base_production()**

In `src/loom/logic.rs`, change the `tick_base_production()` function. Replace the stall check block:

**Old code (around line 260-265):**
```rust
// If buffer is at capacity, node stalls — no production.
if node.buffer >= capacity {
    node.stalled = true;
    continue;
}
```

**New code:**
```rust
// Always produce at full rate for rate tracking.
// Buffer caps at capacity — excess is auto-drained (discarded).
// The extractor never stalls from a full buffer.
```

And change the production calculation to always report the full amount but only add to buffer what fits:

```rust
let amount = rate * delta_hours;
let new_buffer = (node.buffer + amount).min(capacity);
node.buffer = new_buffer;
node.stalled = false;

// Report full production amount for rate tracking, not just what fit in buffer.
if amount > 0.0 {
    let resource = node_native_resource(node_id);
    *produced.entry(resource).or_insert(0.0) += amount;
}
```

**Step 4: Update existing tests that check stall behavior**

Any test that expects `stalled = true` when buffer is full needs to be updated. The extractor no longer stalls from buffer fullness.

**Step 5: Run all loom tests**

Run: `cargo test --lib loom:: -- --nocapture`
Expected: PASS

**Step 6: Commit**

```bash
git add src/loom/logic.rs
git commit -m "fix(loom): auto-drain extractor buffers instead of stalling on full"
```

---

### Task 5: Replace 18 patterns with 28 in discovery.rs

**Files:**
- Modify: `src/loom/discovery.rs`

**Context:** Replace the entire `create_pattern_sequence()` function with the 28 patterns from the design doc. Each pattern now uses `required_rate` (units/hr) and `sustain_duration_secs` instead of `amount`. The `pattern()` helper needs updating.

**Step 1: Update the pattern() helper function**

```rust
fn pattern(index: u32, name: &str, reqs: Vec<(Resource, f64, f64)>) -> WovenPattern {
    WovenPattern {
        index,
        name: name.to_string(),
        requirements: reqs
            .into_iter()
            .map(|(resource, rate, duration_hours)| PatternRequirement {
                resource,
                required_rate: rate,
                sustain_duration_secs: duration_hours * 3600.0,
                sustained_secs: 0.0,
                completed: false,
                amount: 0.0,
                accumulated: 0.0,
            })
            .collect(),
        completed: false,
    }
}
```

Note: The third element in each tuple is duration **in hours** (converted to seconds internally).

**Step 2: Replace create_pattern_sequence() with all 28 patterns**

```rust
fn create_pattern_sequence() -> Vec<WovenPattern> {
    vec![
        // ── Teaching Arc (1-8) ── ~3 days (72 hours) ──
        pattern(0, "First Thread", vec![
            (Resource::Ember, 25.0, 2.0),
        ]),
        pattern(1, "Still Waters", vec![
            (Resource::Silence, 25.0, 2.0),
        ]),
        pattern(2, "Echoing Halls", vec![
            (Resource::Memory, 25.0, 4.0),
        ]),
        pattern(3, "Harmonic Pulse", vec![
            (Resource::Resonance, 25.0, 4.0),
        ]),
        pattern(4, "Mirror and Void", vec![
            (Resource::Reflection, 30.0, 6.0),
            (Resource::VoidEssence, 30.0, 6.0),
        ]),
        pattern(5, "Full Circle", vec![
            (Resource::Ember, 20.0, 10.0),
            (Resource::Reflection, 20.0, 10.0),
            (Resource::VoidEssence, 20.0, 10.0),
            (Resource::Memory, 20.0, 10.0),
            (Resource::Silence, 20.0, 10.0),
            (Resource::Resonance, 20.0, 10.0),
        ]),
        pattern(6, "The Catalyst", vec![
            (Resource::CondensedEmber, 8.0, 16.0),
        ]),
        pattern(7, "Echo of Flame", vec![
            (Resource::EmberEcho, 8.0, 28.0),
        ]),
        // ── Mastery Arc (9-16) ── ~10 days (236 hours) ──
        pattern(8, "Forged in Fire", vec![
            (Resource::ForgedLight, 15.0, 16.0),
        ]),
        pattern(9, "Glass Resonance", vec![
            (Resource::EchoGlass, 15.0, 16.0),
        ]),
        pattern(10, "The Unsung", vec![
            (Resource::StillbornSong, 15.0, 24.0),
        ]),
        pattern(11, "Void Distillation", vec![
            (Resource::PurifiedVoid, 10.0, 24.0),
        ]),
        pattern(12, "Crossed Streams", vec![
            (Resource::ForgedLight, 12.0, 24.0),
            (Resource::EchoGlass, 12.0, 24.0),
        ]),
        pattern(13, "The Asymmetry", vec![
            (Resource::ForgedLight, 25.0, 36.0),
            (Resource::StillbornSong, 8.0, 36.0),
        ]),
        pattern(14, "Pressure Test", vec![
            (Resource::CondensedEmber, 15.0, 36.0),
            (Resource::EmberEcho, 10.0, 36.0),
            (Resource::PurifiedVoid, 10.0, 36.0),
        ]),
        pattern(15, "Three Confluences", vec![
            (Resource::ForgedLight, 18.0, 60.0),
            (Resource::EchoGlass, 18.0, 60.0),
            (Resource::StillbornSong, 18.0, 60.0),
        ]),
        // ── Endgame Arc (17-28) ── ~22 days (534 hours) ──
        pattern(16, "The Amplifier", vec![
            (Resource::ForgedLight, 35.0, 18.0),
        ]),
        pattern(17, "Purified Cascade", vec![
            (Resource::PurifiedVoid, 20.0, 24.0),
            (Resource::ForgedLight, 20.0, 24.0),
        ]),
        pattern(18, "Resonance Cascade", vec![
            (Resource::Resonance, 150.0, 24.0),
            (Resource::StillbornSong, 25.0, 24.0),
        ]),
        pattern(19, "First Weave", vec![
            (Resource::WovenReality, 5.0, 30.0),
        ]),
        pattern(20, "The Unraveling", vec![
            (Resource::WovenReality, 15.0, 36.0),
            (Resource::PurifiedVoid, 15.0, 36.0),
        ]),
        pattern(21, "Grand Harmony", vec![
            (Resource::Ember, 100.0, 36.0),
            (Resource::Reflection, 100.0, 36.0),
            (Resource::VoidEssence, 100.0, 36.0),
            (Resource::Memory, 100.0, 36.0),
            (Resource::Silence, 100.0, 36.0),
            (Resource::Resonance, 100.0, 36.0),
            (Resource::ForgedLight, 30.0, 36.0),
            (Resource::EchoGlass, 30.0, 36.0),
            (Resource::StillbornSong, 30.0, 36.0),
        ]),
        pattern(22, "The Knot", vec![
            (Resource::ForgedLight, 25.0, 36.0),
            (Resource::PurifiedVoid, 15.0, 36.0),
            (Resource::CondensedEmber, 12.0, 36.0),
        ]),
        pattern(23, "Strange Alchemy", vec![
            (Resource::ForgedLight, 30.0, 42.0),
            (Resource::EchoGlass, 30.0, 42.0),
            (Resource::StillbornSong, 30.0, 42.0),
            (Resource::Ember, 80.0, 42.0),
            (Resource::VoidEssence, 80.0, 42.0),
        ]),
        pattern(24, "Refined Purpose", vec![
            (Resource::PurifiedVoid, 30.0, 48.0),
            (Resource::ForgedLight, 25.0, 48.0),
        ]),
        pattern(25, "The Flood", vec![
            (Resource::WovenReality, 35.0, 48.0),
        ]),
        pattern(26, "Everything Flows", vec![
            (Resource::Ember, 50.0, 72.0),
            (Resource::Reflection, 50.0, 72.0),
            (Resource::VoidEssence, 50.0, 72.0),
            (Resource::Memory, 50.0, 72.0),
            (Resource::Silence, 50.0, 72.0),
            (Resource::Resonance, 50.0, 72.0),
            (Resource::ForgedLight, 20.0, 72.0),
            (Resource::EchoGlass, 20.0, 72.0),
            (Resource::StillbornSong, 20.0, 72.0),
            (Resource::CondensedEmber, 10.0, 72.0),
            (Resource::EmberEcho, 10.0, 72.0),
            (Resource::PurifiedVoid, 10.0, 72.0),
            (Resource::WovenReality, 5.0, 72.0),
        ]),
        pattern(27, "Mended Loom", vec![
            (Resource::WovenReality, 20.0, 120.0),
            (Resource::ForgedLight, 40.0, 120.0),
            (Resource::EchoGlass, 40.0, 120.0),
            (Resource::StillbornSong, 40.0, 120.0),
            (Resource::Ember, 80.0, 120.0),
            (Resource::Silence, 80.0, 120.0),
            (Resource::Resonance, 80.0, 120.0),
        ]),
    ]
}
```

**Step 3: Update all discovery tests**

Update tests that reference `18` patterns to use `28`. Update tests that check specific pattern amounts to check `required_rate` and `sustain_duration_secs` instead:

```rust
#[test]
fn test_loom_discovery() {
    let mut loom = LoomState::new();
    complete_discovery(&mut loom);
    assert!(loom.persistent.discovered);
    assert_eq!(loom.persistent.patterns.len(), 28);
}

#[test]
fn test_first_pattern_requires_ember_at_25_per_hour() {
    let mut loom = LoomState::new();
    complete_discovery(&mut loom);
    let first = &loom.persistent.patterns[0];
    assert_eq!(first.requirements.len(), 1);
    assert_eq!(first.requirements[0].resource, Resource::Ember);
    assert!((first.requirements[0].required_rate - 25.0).abs() < 1e-9);
    assert!((first.requirements[0].sustain_duration_secs - 7200.0).abs() < 1e-9); // 2 hours
}

// Update the count check
#[test]
fn test_discovery_does_not_re_discover() {
    let mut loom = LoomState::new();
    complete_discovery(&mut loom);
    assert_eq!(loom.persistent.patterns.len(), 28);
    loom.persistent.patterns[0].requirements[0].sustained_secs = 100.0;
    complete_discovery(&mut loom);
    assert_eq!(loom.persistent.patterns.len(), 28);
    assert!((loom.persistent.patterns[0].requirements[0].sustained_secs - 100.0).abs() < 1e-9);
}
```

Also update `test_final_pattern_has_largest_total_amount` to check `sustain_duration_secs` instead of `amount`:

```rust
#[test]
fn test_final_pattern_has_longest_duration() {
    let mut loom = LoomState::new();
    complete_discovery(&mut loom);
    let last = loom.persistent.patterns.last().unwrap();
    let last_duration = last.requirements[0].sustain_duration_secs;
    // Mended Loom = 120 hours = 432000 seconds — longest in the set.
    assert!((last_duration - 432_000.0).abs() < 1e-9);
}
```

**Step 4: Run all loom tests**

Run: `cargo test --lib loom:: -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/loom/discovery.rs
git commit -m "feat(loom): replace 18 accumulated patterns with 28 sustained rate patterns"
```

---

### Task 6: Shift tier gate thresholds

**Files:**
- Modify: `src/loom/logic.rs`
- Modify: `src/ui/loom_scene.rs`

**Context:** The design shifts tier gates from T1@1, T2@6, T3@12 to T1@1, T2@8, T3@15. Two places need updating:
1. `refinery_tier_unlock_threshold()` in `logic.rs` (gates building refineries)
2. `visible_recipe_tier()` in `loom_scene.rs` (gates recipe visibility in UI)

**Step 1: Write the failing test**

Add to `logic.rs` tests:

```rust
#[test]
fn test_tier_gates_shifted() {
    let mut loom = LoomState::new();
    crate::loom::complete_discovery(&mut loom);
    // 0 complete → no tiers
    assert!(unlocked_tiers(&loom).is_empty());
    // 1 complete → T1 only
    loom.persistent.patterns[0].completed = true;
    assert_eq!(unlocked_tiers(&loom), vec![1]);
    // 7 complete → still T1 only (threshold is 8 for T2)
    for i in 1..7 {
        loom.persistent.patterns[i].completed = true;
    }
    assert_eq!(unlocked_tiers(&loom), vec![1]);
    // 8 complete → T1 + T2
    loom.persistent.patterns[7].completed = true;
    assert_eq!(unlocked_tiers(&loom), vec![1, 2]);
    // 14 complete → still T1 + T2 (threshold is 15 for T3)
    for i in 8..14 {
        loom.persistent.patterns[i].completed = true;
    }
    assert_eq!(unlocked_tiers(&loom), vec![1, 2]);
    // 15 complete → T1 + T2 + T3
    loom.persistent.patterns[14].completed = true;
    assert_eq!(unlocked_tiers(&loom), vec![1, 2, 3]);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib loom::logic::tests::test_tier_gates_shifted -- --nocapture`
Expected: FAIL — current thresholds are 1/6/12, not 1/8/15.

**Step 3: Update the threshold function**

In `src/loom/logic.rs`, change `refinery_tier_unlock_threshold()`:

```rust
fn refinery_tier_unlock_threshold(tier: u8) -> usize {
    match tier {
        1 => 1,
        2 => 8,
        _ => 15,
    }
}
```

In `src/ui/loom_scene.rs`, change `visible_recipe_tier()`:

```rust
fn visible_recipe_tier(completed_patterns: usize) -> u8 {
    if completed_patterns >= 15 {
        3
    } else if completed_patterns >= 8 {
        2
    } else {
        1
    }
}
```

**Step 4: Update any existing tests that relied on old thresholds**

Search for tests that use `6` or `12` as tier gate values and update them to `8` and `15`.

**Step 5: Run all loom tests**

Run: `cargo test --lib loom:: -- --nocapture`
Expected: PASS

**Step 6: Commit**

```bash
git add src/loom/logic.rs src/ui/loom_scene.rs
git commit -m "feat(loom): shift tier gates to T2@8 and T3@15 patterns"
```

---

### Task 7: Wire RateTracker into tick_stages.rs

**Files:**
- Modify: `src/core/tick_stages.rs`
- Modify: `src/loom/types.rs` (add rate trackers to LoomState)

**Context:** Currently `tick_loom()` passes per-node effective rates to `tick_pattern_sustain()`. The new system needs measured rates from a `RateTracker` per resource. The trackers live on `LoomState` (transient, not serialized). Each tick, we push the per-tick production amount for each resource into its tracker, then read the measured rate from each tracker to pass to `tick_pattern_sustain()`.

**Step 1: Add rate trackers to LoomState**

In `src/loom/types.rs`, add a field to `LoomState`:

```rust
pub struct LoomState {
    pub persistent: LoomPersistent,
    /// Per-resource rolling rate trackers (transient, not serialized).
    #[serde(skip)]
    pub rate_trackers: HashMap<Resource, RateTracker>,
}
```

Update `LoomState::new()` and `Default`:

```rust
impl LoomState {
    pub fn new() -> Self {
        Self {
            persistent: LoomPersistent::default(),
            rate_trackers: HashMap::new(),
        }
    }
}
```

**Step 2: Update tick_loom() to use rate trackers**

In `src/core/tick_stages.rs`, modify the `tick_loom()` function. Replace the `rates` computation (lines ~1042-1058) with:

```rust
// Push per-tick production amounts into rate trackers.
for (resource, amount) in &produced {
    loom.rate_trackers
        .entry(*resource)
        .or_insert_with(RateTracker::new)
        .push(*amount);
}
// Also push 0.0 for resources that weren't produced this tick
// (so their rate decays naturally in the rolling window).
for resource in &[
    Resource::Ember, Resource::Reflection, Resource::VoidEssence,
    Resource::Memory, Resource::Silence, Resource::Resonance,
    Resource::ForgedLight, Resource::EchoGlass, Resource::StillbornSong,
    Resource::CondensedEmber, Resource::EmberEcho, Resource::PurifiedVoid,
    Resource::WovenReality,
] {
    if !produced.contains_key(resource) {
        loom.rate_trackers
            .entry(*resource)
            .or_insert_with(RateTracker::new)
            .push(0.0);
    }
}

// Read measured rates from trackers for pattern sustain.
let rates: std::collections::HashMap<crate::loom::Resource, f64> = loom
    .rate_trackers
    .iter()
    .map(|(resource, tracker)| (*resource, tracker.rate_per_hour()))
    .collect();

let pattern_completed =
    crate::loom::tick_pattern_sustain(&mut loom.persistent, &rates, TICK_SECONDS);
if pattern_completed {
    result.loom_changed = true;
}
```

**Step 3: Add necessary imports**

Add `use crate::loom::types::RateTracker;` (or import via `crate::loom::RateTracker`) at the top of `tick_stages.rs` if needed.

**Step 4: Update mod.rs re-exports**

In `src/loom/mod.rs`, add `RateTracker` to the re-exports from `types`:

```rust
pub use types::{
    ..., RateTracker, ...
};
```

**Step 5: Run all tests**

Run: `cargo test --lib -- --nocapture`
Expected: PASS

**Step 6: Commit**

```bash
git add src/loom/types.rs src/core/tick_stages.rs src/loom/mod.rs
git commit -m "feat(loom): wire RateTracker into tick loop for measured production rates"
```

---

### Task 8: Update pattern bar UI for rate-based display

**Files:**
- Modify: `src/ui/loom_scene.rs`

**Context:** The pattern bar currently shows `accumulated/amount` per requirement. Update it to show:
- Rate display: `52/hr (need 25/hr)` with green/yellow coloring
- Time display: `15:00/30:00` sustain progress bar
- State indicator: `✓` (advancing) or `⏸` (paused)
- Completed requirements show a checkmark and full bar
- Update "All 18 Patterns Complete" to "All 28 Patterns Complete"

**Step 1: Read the current render_pattern_bar function**

Already read above (lines 1994-2167). The function renders per-requirement Gauge widgets showing `accumulated/amount`.

**Step 2: Update render_pattern_bar()**

Replace the per-requirement rendering in `render_pattern_bar()`. The key changes:

1. Change completion message from "18" to "28":
```rust
" \u{2728} Loom Mended \u{2014} All 28 Patterns Complete ",
```

2. Change per-requirement display. The gauge ratio becomes `sustained_secs / sustain_duration_secs`. The count label becomes time format `HH:MM/HH:MM`. Add rate display from `loom_state.rate_trackers`.

```rust
for (i, req) in pattern.requirements.iter().enumerate() {
    let row_area = rows[i];
    if row_area.height == 0 {
        continue;
    }

    let ratio = if req.sustain_duration_secs > 0.0 {
        (req.sustained_secs / req.sustain_duration_secs).min(1.0)
    } else {
        1.0
    };
    let met = req.completed;

    // Get current measured rate from rate trackers.
    let current_rate = loom_state
        .rate_trackers
        .get(&req.resource)
        .map(|t| t.rate_per_hour())
        .unwrap_or(0.0);
    let advancing = !met && current_rate >= req.required_rate;

    // Format time: sustained / duration in HH:MM
    let sustained_mins = (req.sustained_secs / 60.0) as u32;
    let duration_mins = (req.sustain_duration_secs / 60.0) as u32;
    let time_label = format!(
        "{}:{:02}/{}:{:02}",
        sustained_mins / 60, sustained_mins % 60,
        duration_mins / 60, duration_mins % 60,
    );

    // Rate label: "52/hr (25/hr)"
    let rate_label = format!("{:.0}/hr ({:.0}/hr)", current_rate, req.required_rate);

    // State indicator
    let state_icon = if met {
        " \u{2713}" // ✓
    } else if advancing {
        " \u{25B6}" // ▶
    } else {
        " \u{23F8}" // ⏸
    };

    // ... render emoji, gauge, labels using existing layout pattern ...
}
```

3. Update the count label column to show `time_label + rate_label + state_icon`:
```rust
let count_label = format!("{} {}{}", time_label, rate_label, state_icon);
```

4. Color the rate green when advancing, yellow/amber when paused, and bright green when completed.

5. Update the overall progress bar to average `sustained_secs / sustain_duration_secs` across all requirements.

**Step 3: Run the project to verify visually**

Run: `cargo build`
Expected: compiles successfully.

**Step 4: Commit**

```bash
git add src/ui/loom_scene.rs
git commit -m "feat(loom): update pattern bar UI for rate-based sustained display"
```

---

### Task 9: Update max_refineries cap and hardcoded "18" references

**Files:**
- Modify: `src/loom/types.rs` — `max_refineries()` is fine (counts completed patterns dynamically), but cap should be 28 now
- Modify: `src/loom/patterns.rs` — update comment "all 18 patterns"
- Modify: `src/loom/CLAUDE.md` — update documentation
- Search for any other hardcoded "18" references

**Step 1: Find all "18" references in loom module**

Search for `18` in the loom module and UI:

```bash
grep -rn "18" src/loom/ src/ui/loom_scene.rs | grep -i "pattern\|18"
```

**Step 2: Update each reference**

- `src/loom/patterns.rs:105` — comment "all 18 patterns" → "all 28 patterns"
- `src/loom/CLAUDE.md:14` — "18 woven patterns" → "28 woven patterns"
- `src/loom/CLAUDE.md:83` — "max 18" → "max 28"
- `src/loom/CLAUDE.md` — update pattern system description from accumulated amounts to sustained rates
- `src/ui/loom_scene.rs:2004` — already updated in Task 8

**Step 3: Update CLAUDE.md documentation**

Update `src/loom/CLAUDE.md` to reflect:
- 28 patterns instead of 18
- Sustained rate mechanic instead of accumulated totals
- New `RateTracker` type
- Updated `PatternRequirement` fields
- Tier gates at 1/8/15 instead of 1/6/12
- Auto-drain buffer behavior

**Step 4: Run all tests**

Run: `cargo test --lib loom:: -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/loom/patterns.rs src/loom/CLAUDE.md
git commit -m "docs(loom): update references from 18 to 28 patterns and document sustained rate system"
```

---

### Task 10: Update pattern tests in patterns.rs for 28-pattern coverage

**Files:**
- Modify: `src/loom/patterns.rs`

**Context:** After Tasks 3 and 5, the test suite needs updating to exercise the 28-pattern set with sustained rate mechanics. Key scenarios to test:
- Multi-requirement pattern with independent completion (e.g., pattern 4 "Mirror and Void")
- Pattern completion advances to next pattern correctly through all 28
- `all_patterns_complete()` works with 28 patterns

**Step 1: Add/update multi-requirement tests**

```rust
#[test]
fn test_multi_requirement_independent_completion() {
    let mut state = state_with_patterns();
    // Pattern 4 "Mirror and Void": Reflection 30/hr for 6hr, VoidEssence 30/hr for 6hr
    for i in 0..4 {
        state.persistent.patterns[i].completed = true;
    }
    state.persistent.active_pattern = 4;

    // Complete only the first requirement.
    state.persistent.patterns[4].requirements[0].sustained_secs =
        state.persistent.patterns[4].requirements[0].sustain_duration_secs;
    state.persistent.patterns[4].requirements[0].completed = true;

    // Pattern should NOT be complete yet (second requirement pending).
    assert!(!active_pattern_requirements_met(&state.persistent));

    // Complete second requirement.
    state.persistent.patterns[4].requirements[1].sustained_secs =
        state.persistent.patterns[4].requirements[1].sustain_duration_secs;
    state.persistent.patterns[4].requirements[1].completed = true;

    // Now pattern should be complete.
    assert!(active_pattern_requirements_met(&state.persistent));
}

#[test]
fn test_all_28_patterns_complete() {
    let mut state = state_with_patterns();
    for p in &mut state.persistent.patterns {
        p.completed = true;
    }
    assert!(all_patterns_complete(&state.persistent));
    assert_eq!(state.persistent.patterns.len(), 28);
}

#[test]
fn test_sustained_rate_exact_threshold_advances() {
    let mut state = state_with_patterns();
    // Rate exactly equal to required_rate should advance.
    let threshold = state.persistent.patterns[0].requirements[0].required_rate;
    let r = rates(&[(Resource::Ember, threshold)]);
    tick_pattern_sustain(&mut state.persistent, &r, 1.0);
    assert!(state.persistent.patterns[0].requirements[0].sustained_secs > 0.0);
}
```

**Step 2: Run all tests**

Run: `cargo test --lib loom:: -- --nocapture`
Expected: PASS

**Step 3: Commit**

```bash
git add src/loom/patterns.rs
git commit -m "test(loom): add sustained rate tests for 28-pattern coverage"
```

---

### Task 11: Full integration verification

**Files:** None (read-only verification)

**Step 1: Run full CI check suite**

```bash
make check
```

This runs: formatting, clippy, all tests, build, and audit.

Expected: All checks pass.

**Step 2: Verify loom tests specifically**

```bash
cargo test --lib loom:: -- --nocapture 2>&1 | tail -20
```

Expected: All loom tests pass.

**Step 3: Build and verify no warnings**

```bash
cargo build 2>&1 | grep -i warning
```

Expected: No new warnings (existing `dead_code` allows are fine).

**Step 4: Commit any remaining fixes**

If any issues surfaced during verification, fix and commit them.
