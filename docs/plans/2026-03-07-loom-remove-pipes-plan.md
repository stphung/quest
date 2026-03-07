# Loom Simplification: Remove Pipes, Direct-Pull Refineries — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Remove pipes from the Loom and replace with direct-pull refineries that declare their input sources, plus convert patterns from rate×time to raw amounts.

**Architecture:** Delete `pipes.rs` and all pipe types/fields. Add `sources_a`/`sources_b` vectors to `Refinery`. Replace `tick_pipe_flow()` with `tick_refinery_pull()` that calculates contention and pulls from sources. Convert `PatternRequirement` from `rate_per_hour` to `amount` with accumulator. Update UI, input, tick stages, debug menu, and persistence.

**Tech Stack:** Rust, Serde (JSON persistence), Ratatui (terminal UI)

**Design doc:** `docs/plans/2026-03-07-loom-remove-pipes-design.md`

---

## Task 1: Convert Patterns from Rate×Time to Raw Amounts

**Files:**
- Modify: `src/loom/types.rs` — `PatternRequirement` and `WovenPattern` structs
- Modify: `src/loom/discovery.rs` — `create_pattern_sequence()` and `pattern()` helper
- Modify: `src/loom/patterns.rs` — all rate-checking and sustain logic
- Test: inline `#[cfg(test)]` modules in each file

**Context:** Currently patterns require sustaining a `rate_per_hour` for `sustain_seconds`. We're converting to: produce a total `amount` of each resource. The accumulator increments based on actual production rate each tick. This is a self-contained change with no pipe dependencies.

**Step 1: Update `PatternRequirement` in `types.rs`**

Change the struct from:

```rust
pub struct PatternRequirement {
    pub resource: Resource,
    pub rate_per_hour: f64,
}
```

to:

```rust
pub struct PatternRequirement {
    pub resource: Resource,
    /// Total amount of this resource needed to complete the pattern.
    pub amount: f64,
    /// Accumulated production so far.
    #[serde(default)]
    pub accumulated: f64,
}
```

Also in `WovenPattern`, remove `sustain_seconds`, `sustained_seconds`, and `sustained_seconds_frac` fields. The completion check moves to "all requirements accumulated >= amount".

New `WovenPattern`:

```rust
pub struct WovenPattern {
    pub index: u32,
    pub name: String,
    pub requirements: Vec<PatternRequirement>,
    #[serde(default)]
    pub completed: bool,
}
```

**Step 2: Update `discovery.rs` — convert all 18 patterns**

Change the `pattern()` helper from `(index, name, reqs_with_rates, sustain_seconds)` to `(index, name, reqs_with_amounts)`.

Converted amounts (computed as `rate_per_hour * sustain_seconds / 3600`):

| # | Name | Requirements (resource, amount) |
|---|------|-------------------------------|
| 0 | First Thread | Ember 1.0 |
| 1 | The Bridge | Ember 3.0, Reflection 1.0 |
| 2 | Long Road | Ember 2.0, Memory 1.0 |
| 3 | Balancing Act | Ember 3.0, Reflection 3.0, VoidEssence 3.0 |
| 4 | Full Circle | All 6 base × 2.0 |
| 5 | The Catalyst | CondensedEmber 2.0 |
| 6 | Crossed Streams | CondensedEmber 2.0, EmberEcho 2.0 |
| 7 | The Diversion | ForgedLight 2.5, Ember 7.5 |
| 8 | Three Confluences | ForgedLight 3.0, EchoGlass 3.0, StillbornSong 3.0 |
| 9 | Pressure Test | ForgedLight 6.0, EchoGlass 6.0 |
| 10 | The Bottleneck | StillbornSong 9.0 |
| 11 | Shifting Gears | ForgedLight 6.0 |
| 12 | Harmony | All 6 base × 20.0 |
| 13 | The Triad | All 6 base × 12.0, all 3 confluence × 12.0 |
| 14 | Razor's Edge | ForgedLight 16.0, EchoGlass 16.0 |
| 15 | Resonance Cascade | Resonance 40.0 |
| 16 | The Unraveling | WovenReality 6.0 |
| 17 | Mended Loom | WovenReality 24.0, base×40.0, confluence×24.0 |

New helper signature:

```rust
fn pattern(index: u32, name: &str, reqs: Vec<(Resource, f64)>) -> WovenPattern {
    WovenPattern {
        index,
        name: name.to_string(),
        requirements: reqs
            .into_iter()
            .map(|(resource, amount)| PatternRequirement {
                resource,
                amount,
                accumulated: 0.0,
            })
            .collect(),
        completed: false,
    }
}
```

**Step 3: Rewrite `patterns.rs` — accumulator-based completion**

Replace `active_pattern_requirements_met()`:

```rust
pub fn active_pattern_requirements_met(persistent: &LoomPersistent) -> bool {
    let Some(pattern) = persistent.patterns.get(persistent.active_pattern) else {
        return false;
    };
    if pattern.completed {
        return false;
    }
    pattern.requirements.iter().all(|req| req.accumulated >= req.amount)
}
```

Replace `tick_pattern_sustain()` — now takes `rates: &HashMap<Resource, f64>` and accumulates:

```rust
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

    let delta_hours = delta_seconds / 3600.0;

    // Accumulate production for each requirement.
    for req in &mut pattern.requirements {
        let rate = rates.get(&req.resource).copied().unwrap_or(0.0);
        req.accumulated = (req.accumulated + rate * delta_hours).min(req.amount);
    }

    // Check if all requirements are met.
    if pattern.requirements.iter().all(|req| req.accumulated >= req.amount) {
        complete_active_pattern(persistent);
        return true;
    }

    false
}
```

Replace `active_pattern_requirement_status()`:

```rust
pub fn active_pattern_requirement_status(
    persistent: &LoomPersistent,
) -> Vec<(f64, f64)> {
    let Some(pattern) = persistent.patterns.get(persistent.active_pattern) else {
        return Vec::new();
    };
    pattern
        .requirements
        .iter()
        .map(|req| (req.accumulated, req.amount))
        .collect()
}
```

**Step 4: Update tests in all three files**

- `types.rs` tests: Remove references to `sustain_seconds`, `sustained_seconds`, `sustained_seconds_frac`.
- `discovery.rs` tests: Update `test_first_pattern_sustain_is_30_minutes` → test first pattern amount is 1.0. Update `test_all_requirement_rates_are_positive` → test all amounts are positive. Remove sustain-related tests.
- `patterns.rs` tests: Rewrite all tests to use accumulator model. Key tests:
  - `test_accumulates_production_each_tick` — rate 2.0/hr, 0.1s tick → accumulated increases
  - `test_completes_when_all_accumulated` — all reqs at 100% → completion
  - `test_partial_accumulation_no_completion` — some reqs at 100%, others not → no completion
  - `test_zero_rate_no_accumulation` — 0 rate → accumulated stays at 0
  - `test_accumulated_capped_at_amount` — can't exceed target

**Step 5: Update callers**

The `tick_stages.rs` call to `tick_pattern_sustain` already passes `rates` and `delta_seconds` — the signature is compatible. But `active_pattern_requirements_met` now takes no `rates` parameter (requirements are checked internally via `accumulated`). Search for all callers of `active_pattern_requirements_met` and `active_pattern_requirement_status` and update.

Also update `render_pattern_bar()` in `loom_scene.rs` to show `37/60` format instead of time remaining.

**Step 6: Run tests and commit**

Run: `cargo test -p quest --lib -- loom`
Expected: All loom tests pass.

```bash
git add src/loom/types.rs src/loom/discovery.rs src/loom/patterns.rs src/ui/loom_scene.rs src/core/tick_stages.rs
git commit -m "feat(loom): convert patterns from rate×time to raw amounts"
```

---

## Task 2: Add Source Fields to Refinery, Remove Pipe Types

**Files:**
- Modify: `src/loom/types.rs` — `Refinery` struct, remove `Pipe`/`PipeTier`, remove `pipes` from `LoomPersistent`
- Modify: `src/loom/mod.rs` — remove pipe re-exports
- Delete: `src/loom/pipes.rs`
- Test: inline `#[cfg(test)]` in `types.rs`

**Context:** This is the core data model change. Refineries get `sources_a`/`sources_b` vectors. All pipe types are removed. The `pipes.rs` file is deleted entirely.

**Step 1: Add source fields to `Refinery` in `types.rs`**

```rust
pub struct Refinery {
    pub input_a: Resource,
    pub input_b: Resource,
    pub nature: NodeNature,
    pub output: Resource,
    pub amount: f64,
    pub tier: u8,
    /// Sources for input A — extractors or lower-tier refineries.
    #[serde(default)]
    pub sources_a: Vec<LoomNodeRef>,
    /// Sources for input B — extractors or lower-tier refineries.
    #[serde(default)]
    pub sources_b: Vec<LoomNodeRef>,
    #[serde(default)]
    pub buffer: f64,
    #[serde(default = "default_buffer_capacity")]
    pub buffer_capacity: f64,
    #[serde(default = "default_node_level")]
    pub level: u32,
    #[serde(default)]
    pub stalled: bool,
    #[serde(default)]
    pub under_construction: bool,
    #[serde(default)]
    pub construction_ticks_remaining: u32,
}
```

Update `Refinery::new()` to accept `sources_a` and `sources_b`:

```rust
pub fn new(
    input_a: Resource, input_b: Resource, nature: NodeNature,
    output: Resource, amount: f64, tier: u8,
    sources_a: Vec<LoomNodeRef>, sources_b: Vec<LoomNodeRef>,
) -> Self {
    Self {
        input_a, input_b, nature, output, amount, tier,
        sources_a, sources_b,
        buffer: 0.0, buffer_capacity: 20.0, level: 1,
        stalled: false, under_construction: false,
        construction_ticks_remaining: 0,
    }
}
```

**Step 2: Remove `Pipe`, `PipeTier`, and `pipes` field**

Delete these from `types.rs`:
- `PipeTier` enum (lines 208-226)
- `Pipe` struct (lines 228-240)
- `pipes: Vec<Pipe>` field from `LoomPersistent` (line 286)
- `pipes: Vec::new()` from `LoomPersistent::default()` (line 320)

**Step 3: Delete `src/loom/pipes.rs`**

Remove the entire file (~600+ LOC including tests).

**Step 4: Update `src/loom/mod.rs`**

- Remove `pub mod pipes;`
- Remove entire `pub use pipes::{...}` block
- Remove `Pipe`, `PipeTier` from the `pub use types::{...}` block

**Step 5: Fix all compilation errors**

Every file that references `Pipe`, `PipeTier`, `pipes`, pipe functions, or `Refinery::new()` (which now takes sources) will fail to compile. The main files to fix:
- `src/loom/logic.rs` — `demolish_refinery` pipe cleanup, `tick_stall_detection` pipe references, `process_reactions` (pipe delivery based), `process_refinery_reactions` (pipe delivery based), `build_refinery` uses `PIPE_CONSTRUCTION_TICKS`
- `src/core/tick_stages.rs` — `tick_pipe_construction`, `tick_pipe_flow`, pipe delivery processing
- `src/input/loom_input.rs` — pipe selection, split ratio adjustment, `P` hotkey
- `src/ui/loom_scene.rs` — pipe rendering, port labels
- `src/utils/debug_menu.rs` — `LoomClearRefineries` pipe cleanup

For now, **stub out** the removed functions so the code compiles. The next tasks will implement the replacements.

**Step 6: Update all `Refinery::new()` call sites**

Search for all `Refinery::new(` calls and add empty source vectors: `vec![], vec![]`. Call sites:
- `src/loom/logic.rs` `build_refinery()`
- `src/utils/debug_menu.rs` `LoomBuildTestRefineryT1`, `LoomBuildTestRefineryT2`
- All test code creating refineries

**Step 7: Run tests and commit**

Run: `cargo test -p quest --lib -- loom`
Expected: Pipe tests gone, remaining loom tests pass (some logic tests may need updating).

```bash
git add -A
git commit -m "feat(loom): remove pipes, add source fields to Refinery"
```

---

## Task 3: Implement Direct-Pull Tick (`tick_refinery_pull`)

**Files:**
- Modify: `src/loom/logic.rs` — add `tick_refinery_pull()`, add contention calculation, add source validation
- Modify: `src/core/tick_stages.rs` — replace pipe tick calls with `tick_refinery_pull()`
- Test: inline `#[cfg(test)]` in `logic.rs`

**Context:** This is the new core simulation. Each tick, refineries pull resources directly from their sources. Contention splits source output evenly among consumers.

**Step 1: Add intake cap constant**

In `logic.rs`:

```rust
/// Max intake rate per input, by refinery tier (units/hour).
pub fn tier_intake_cap(tier: u8) -> f64 {
    match tier {
        1 => 2.0,
        2 => 3.0,
        3 => 4.0,
        _ => 2.0,
    }
}
```

**Step 2: Add source validation**

```rust
/// Check if a source is valid for a refinery's tier.
/// T1: extractors only. T2: extractors + T1. T3: extractors + T1 + T2.
pub fn valid_source_for_tier(
    source: LoomNodeRef,
    refinery_tier: u8,
    refineries: &[Refinery],
) -> bool {
    match source {
        LoomNodeRef::Extractor(_) => true, // all tiers can pull from extractors
        LoomNodeRef::Refinery(idx) => {
            if let Some(source_ref) = refineries.get(idx) {
                source_ref.tier < refinery_tier
            } else {
                false
            }
        }
    }
}
```

**Step 3: Implement `tick_refinery_pull()`**

```rust
/// Direct-pull tick: each refinery pulls from its sources, respecting contention and intake caps.
///
/// Returns a map of resource → total produced this tick (for pattern tracking).
pub fn tick_refinery_pull(
    loom: &mut LoomState,
    delta_seconds: f64,
) -> std::collections::HashMap<Resource, f64> {
    use std::collections::HashMap;
    let delta_hours = delta_seconds / 3600.0;
    let mut produced: HashMap<Resource, f64> = HashMap::new();

    // Step 1: Count consumers per source (for contention).
    let mut consumer_count: HashMap<LoomNodeRef, usize> = HashMap::new();
    for r in &loom.persistent.refineries {
        if r.under_construction {
            continue;
        }
        for src in r.sources_a.iter().chain(r.sources_b.iter()) {
            *consumer_count.entry(*src).or_insert(0) += 1;
        }
    }

    // Step 2: Calculate available output per source.
    let mut source_output: HashMap<LoomNodeRef, f64> = HashMap::new();
    for node in &loom.persistent.nodes {
        if !node.unlocked {
            continue;
        }
        let rate = node_effective_rate(loom, node);
        source_output.insert(LoomNodeRef::Extractor(node.id), rate);
    }
    // Need to avoid borrow conflict — collect node data first, then use loom reference.
    // (Node rates are already in source_output.)
    // Refinery outputs as sources: use their buffer drain rate (output rate from previous tick).
    // For simplicity, use the refinery's current buffer as the available pool.
    // Actually, the design says refineries pull from source's *output rate*, not buffer.
    // For extractors: use effective_rate. For refineries as sources: use their last-tick output rate.
    // We'll compute in two passes: first T1 (from extractors), then T2 (from extractors+T1), then T3.

    // Simplified: process by tier order to ensure lower tiers produce before higher tiers pull.
    let refinery_indices_by_tier: Vec<Vec<usize>> = {
        let mut by_tier: Vec<Vec<usize>> = vec![vec![], vec![], vec![]];
        for (i, r) in loom.persistent.refineries.iter().enumerate() {
            if !r.under_construction {
                let tier_idx = (r.tier as usize).saturating_sub(1).min(2);
                by_tier[tier_idx].push(i);
            }
        }
        by_tier
    };

    let mut refinery_output_rates: HashMap<usize, f64> = HashMap::new();

    for tier_group in &refinery_indices_by_tier {
        for &idx in tier_group {
            let r = &loom.persistent.refineries[idx];
            let cap = tier_intake_cap(r.tier);

            // Pull input A
            let mut total_pull_a = 0.0;
            for src in &r.sources_a {
                let available = match src {
                    LoomNodeRef::Extractor(nid) => {
                        source_output.get(&LoomNodeRef::Extractor(*nid)).copied().unwrap_or(0.0)
                    }
                    LoomNodeRef::Refinery(ri) => {
                        refinery_output_rates.get(ri).copied().unwrap_or(0.0)
                    }
                };
                let consumers = consumer_count.get(src).copied().unwrap_or(1).max(1);
                let share = available / consumers as f64;
                total_pull_a += share.min(cap);
            }

            // Pull input B
            let mut total_pull_b = 0.0;
            for src in &r.sources_b {
                let available = match src {
                    LoomNodeRef::Extractor(nid) => {
                        source_output.get(&LoomNodeRef::Extractor(*nid)).copied().unwrap_or(0.0)
                    }
                    LoomNodeRef::Refinery(ri) => {
                        refinery_output_rates.get(ri).copied().unwrap_or(0.0)
                    }
                };
                let consumers = consumer_count.get(src).copied().unwrap_or(1).max(1);
                let share = available / consumers as f64;
                total_pull_b += share.min(cap);
            }

            // Output = min(pull_a, pull_b) * recipe_amount
            let output_rate = total_pull_a.min(total_pull_b) * r.amount;
            refinery_output_rates.insert(idx, output_rate);

            // Add to buffer
            let actual = output_rate * delta_hours;
            let r_mut = &mut loom.persistent.refineries[idx];
            let space = (r_mut.buffer_capacity - r_mut.buffer).max(0.0);
            let deposited = actual.min(space);
            r_mut.buffer += deposited;
            if deposited > 0.0 {
                r_mut.stalled = false;
            }

            *produced.entry(r_mut.output).or_insert(0.0) += deposited;
        }
    }

    produced
}
```

**Step 4: Update `tick_stages.rs`**

In `tick_loom()`, replace the pipe tick section:

```rust
// OLD (remove):
// let completed_pipes = crate::loom::tick_pipe_construction(loom);
// ...
// let deliveries = crate::loom::tick_pipe_flow(loom, TICK_SECONDS);
// let refinery_deliveries = ...
// let _reactions = crate::loom::process_reactions(loom, deliveries);
// let _refinery_reactions = crate::loom::process_refinery_reactions(loom, refinery_deliveries);

// NEW:
let refinery_produced = crate::loom::tick_refinery_pull(loom, TICK_SECONDS);
```

Merge `refinery_produced` into the `base_produced` rates map for pattern sustain.

**Step 5: Remove `process_reactions()` and `process_refinery_reactions()`**

These functions process pipe deliveries — no longer needed. Remove from `logic.rs` and `mod.rs`.

**Step 6: Update stall detection**

`tick_stall_detection()` currently checks pipes. Simplify to: an extractor is stalled when its buffer is full (no refineries pulling from it, or all pulling refineries have full buffers). The refinery stall detection (`tick_refinery_stall_detection`) already works correctly (buffer >= capacity).

For extractors, simplify `tick_stall_detection`:

```rust
pub fn tick_stall_detection(loom: &mut LoomState) -> Vec<NodeId> {
    let mut changed = Vec::new();
    for node in &mut loom.persistent.nodes {
        if !node.unlocked {
            continue;
        }
        let should_stall = node.buffer >= node.buffer_capacity;
        if node.stalled != should_stall {
            node.stalled = should_stall;
            changed.push(node.id);
        }
    }
    changed
}
```

**Step 7: Write tests**

Key tests for `tick_refinery_pull`:
- `test_single_refinery_pulls_from_extractors` — T1 with two extractor sources, verify output rate
- `test_contention_splits_evenly` — two T1s pulling from same extractor, each gets half
- `test_tier_intake_cap_limits_pull` — extractor produces 10/hr, T1 caps at 2.0
- `test_multi_source_merge` — refinery with two sources for input A, sums their shares
- `test_t2_pulls_from_t1_output` — T2 sources include a T1 refinery
- `test_stalled_refinery_no_buffer_overflow` — buffer full = no more deposits
- `test_source_validation_t1_only_extractors` — T1 can't pull from other refineries
- `test_source_validation_t2_from_t1` — T2 can pull from T1 but not T2

**Step 8: Run tests and commit**

Run: `cargo test -p quest --lib -- loom`

```bash
git add src/loom/logic.rs src/core/tick_stages.rs
git commit -m "feat(loom): implement direct-pull tick with contention"
```

---

## Task 4: Update `build_refinery()` and `demolish_refinery()`

**Files:**
- Modify: `src/loom/logic.rs` — update build/demolish functions

**Step 1: Update `build_refinery()` to accept sources**

New signature:

```rust
pub fn build_refinery(
    loom: &mut LoomState,
    input_a: Resource,
    input_b: Resource,
    nature: NodeNature,
    sources_a: Vec<LoomNodeRef>,
    sources_b: Vec<LoomNodeRef>,
) -> Result<usize, RefineryError>
```

Add source validation: each source must be valid for the recipe tier (using `valid_source_for_tier`). Each source must actually produce the required resource. Add `RefineryError::InvalidSource` variant.

Replace `PIPE_CONSTRUCTION_TICKS` reference with a local constant:

```rust
pub const REFINERY_CONSTRUCTION_TICKS: u32 = 72_000; // 2 hours at 100ms/tick
```

**Step 2: Simplify `demolish_refinery()`**

Remove all pipe cleanup code. Also re-index source references in remaining refineries:

```rust
pub fn demolish_refinery(loom: &mut LoomState, idx: usize) {
    if idx >= loom.persistent.refineries.len() {
        return;
    }
    loom.persistent.refineries.remove(idx);

    // Re-index source references in remaining refineries.
    for r in &mut loom.persistent.refineries {
        reindex_sources(&mut r.sources_a, idx);
        reindex_sources(&mut r.sources_b, idx);
    }
}

fn reindex_sources(sources: &mut Vec<LoomNodeRef>, removed_idx: usize) {
    sources.retain(|s| !matches!(s, LoomNodeRef::Refinery(i) if *i == removed_idx));
    for s in sources.iter_mut() {
        if let LoomNodeRef::Refinery(ref mut i) = s {
            if *i > removed_idx {
                *i -= 1;
            }
        }
    }
}
```

**Step 3: Write tests and commit**

```bash
git commit -m "feat(loom): update build/demolish for direct-pull model"
```

---

## Task 5: Update Input Handling

**Files:**
- Modify: `src/input/loom_input.rs` — remove pipe input, add source editing

**Step 1: Remove pipe-related input**

- Remove `P` hotkey handler (pipe cycling)
- Remove `adjust_selected_pipe()` function
- Remove Left/Right split ratio adjustment in ListDetail view
- Remove `selected_pipe` resets (keep the field for now, clean up later)

**Step 2: Update tests**

- Remove `test_left_right_adjusts_split_ratio`
- Remove `test_left_right_no_op_when_no_pipes`
- Remove `test_p_cycles_pipe_selection`
- Remove `test_up_down_resets_selected_pipe` pipe assertions (keep navigation test)
- Remove all `Pipe` / `PipeTier` imports from test code

**Step 3: Run tests and commit**

```bash
git commit -m "feat(loom): remove pipe input handling"
```

---

## Task 6: Update Debug Menu

**Files:**
- Modify: `src/utils/debug_menu.rs` — update refinery debug actions

**Step 1: Update `LoomClearRefineries`**

Remove pipe cleanup from `LoomClearRefineries`:

```rust
// OLD:
loom.persistent.pipes.retain(|p| { ... });
loom.persistent.refineries.clear();

// NEW:
loom.persistent.refineries.clear();
```

**Step 2: Update `LoomBuildTestRefineryT1` and `LoomBuildTestRefineryT2`**

Add source vectors to `Refinery::new()` calls. Use sensible defaults — pull from the first two extractors:

```rust
// T1 test refinery: Ember+Void → ForgedLight, sources from EmberSpindle and VoidCondenser
let r = Refinery::new(
    recipe.input_a, recipe.input_b, recipe.node_nature,
    recipe.output, recipe.amount, recipe.tier,
    vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
    vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
);
```

**Step 3: Run tests and commit**

```bash
git commit -m "feat(loom): update debug menu for direct-pull model"
```

---

## Task 7: Update UI Rendering

**Files:**
- Modify: `src/ui/loom_scene.rs` — update Flow View, sidebar, pattern bar

**Step 1: Remove pipe rendering**

- Remove `render_port_labels()` function calls
- Remove pipe connection drawing in `render_flow_view()`

**Step 2: Update refinery rows in Flow View**

In the processing area below extractors, render each refinery as a compact row:

```
⠹ T1 ForgedLight    Emb←[ES] Voi←[VC]  2.0/hr  ████░░░░░░
```

Components:
- Throbber character (braille spinner, speed based on tier)
- Tier badge
- Output resource name (short form)
- Source badges: `ResourceShort←[SourceShort]` for each source
- Current output rate
- Buffer bar

**Step 3: Add throbber animation**

Add a throbber state to `LoomUiState`:

```rust
pub throbber_frame: u32, // incremented each render, used for spinner animation
```

Throbber frame rate per tier:
- T1: advance every 5 frames (500ms at 100ms render)
- T2: advance every 3 frames (300ms)
- T3: advance every 1-2 frames (150ms)

Braille chars: `['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏']`

**Step 4: Update sidebar detail panel**

When a refinery is selected, show:
- Recipe info (inputs → output × amount)
- Per-source pull rates with contention info
- Buffer level
- Bottleneck diagnosis

When an extractor is selected, show:
- Consumer count and contention split

**Step 5: Update pattern bar**

Change from `██████░░░░ 12:30` (time remaining) to `██████░░░░ 37/60` (accumulated/total).

Show per-requirement progress:
```
ForgedLight: 37/60  (+2.0/hr) ✓
EchoGlass: 12/60  (+1.5/hr)
```

**Step 6: Run full test suite and commit**

Run: `cargo test -p quest --lib`
Run: `cargo clippy --all-targets -- -D warnings`

```bash
git commit -m "feat(loom): update UI for direct-pull model and amount patterns"
```

---

## Task 8: Update Persistence and CLAUDE.md

**Files:**
- Modify: `src/loom/persistence.rs` — verify backward compatibility
- Modify: `src/loom/CLAUDE.md` — update documentation
- Modify: `CLAUDE.md` — update module description if needed

**Step 1: Verify backward compatibility**

All new fields use `#[serde(default)]`, so old save files without `sources_a`/`sources_b`/`accumulated` will deserialize correctly (empty vecs, 0.0). The removed `pipes` field also has `#[serde(default)]`, so old saves with `pipes` data will simply ignore it (serde's `deny_unknown_fields` is not enabled).

Old `PatternRequirement` saves with `rate_per_hour` field: since we're renaming to `amount`, we need `#[serde(alias = "rate_per_hour")]` on the `amount` field for backward compatibility:

```rust
pub struct PatternRequirement {
    pub resource: Resource,
    #[serde(alias = "rate_per_hour")]
    pub amount: f64,
    #[serde(default)]
    pub accumulated: f64,
}
```

Similarly, old `WovenPattern` saves have `sustain_seconds`, `sustained_seconds`, `sustained_seconds_frac` — these will be ignored since serde skips unknown fields by default.

**Step 2: Update `src/loom/CLAUDE.md`**

Remove all pipe documentation. Update:
- Module Structure (remove `pipes.rs`)
- Key Types (remove `Pipe`, `PipeTier`, update `Refinery` description)
- Node Addressing section (update to mention refinery sources instead of pipe endpoints)
- Production Chain Flow (remove pipe step)
- Refinery System (add source fields, contention, intake caps)
- Add "Direct-Pull System" section explaining contention model
- Update Input section (remove `P` hotkey)
- Update Debug Menu section
- Update Integration Points (remove pipe tick references)

**Step 3: Run `make check` and commit**

```bash
make check
git commit -m "docs(loom): update CLAUDE.md for pipe removal"
```

---

## Task 9: Full CI Check and Cleanup

**Files:**
- All modified files

**Step 1: Run full CI checks**

```bash
make check
```

This runs: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo build --all-targets`, `cargo audit --deny yanked`.

**Step 2: Fix any issues**

- Remove any remaining dead code warnings from pipe-related imports
- Clean up any `#[allow(dead_code)]` that are no longer needed
- Remove `selected_pipe` from `LoomUiState` if no longer used anywhere

**Step 3: Final commit**

```bash
git commit -m "chore(loom): cleanup dead code after pipe removal"
```
