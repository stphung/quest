# Loom Refineries Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add buildable, recipe-locked processing nodes (Refineries) to the Loom of Worlds, creating multi-step Factorio-style production chains below the existing 6 Extractor nodes.

**Architecture:** Introduce a unified `LoomNodeRef` addressing type that covers both fixed Extractors (`NodeId`) and dynamic Refineries (by index). Refineries are stored in a new `Vec<Refinery>` on `LoomPersistent`. Pipes, flow simulation, and reactions all operate on `LoomNodeRef` instead of `NodeId`. The UI renders Refineries in a scrollable processing area below the existing 3x2 Extractor grid. Pattern completion gates which Refinery tiers can be built; resource costs gate each instance.

**Tech Stack:** Rust, Serde (JSON persistence), Ratatui (terminal UI), existing `scene_fx` cell buffer rendering

---

## Context for the Implementer

### Current Architecture

The Loom has 6 fixed nodes identified by the `NodeId` enum (`EmberSpindle`, `VoidCondenser`, etc.). Each is a `LoomNode` struct stored in `loom.persistent.nodes: Vec<LoomNode>`. Pipes connect nodes using `NodeId` for `from`/`to` fields. The pipe flow system (`pipes.rs`) finds nodes by iterating `loom.persistent.nodes` and matching on `NodeId`. Recipes are looked up by `(input_a, input_b, node_nature)`.

### Key Files

| File | What it does |
|------|-------------|
| `src/loom/types.rs` | `NodeId` enum, `LoomNode`, `Pipe`, `LoomPersistent`, `LoomState`, `LoomUiState` |
| `src/loom/pipes.rs` | Pipe building, flow simulation, split ratios — all use `NodeId` |
| `src/loom/logic.rs` | Base production, stall detection, node upgrades, neighbor unlocking, reactions |
| `src/loom/recipes.rs` | Recipe registry, `lookup_recipe(a, b, nature)`, `recipes_by_nature()` |
| `src/loom/patterns.rs` | Woven Pattern sustain timer and completion |
| `src/loom/discovery.rs` | 18 patterns defined in `create_pattern_sequence()` |
| `src/ui/loom_scene.rs` | Flow View rendering with cell buffer, sidebar, node boxes |
| `src/input/loom_input.rs` | Keyboard navigation (2D grid for FlowView, list for ListDetail) |
| `src/core/tick_stages.rs:987` | `tick_loom()` — calls base production, pipe flow, reactions, patterns |

### Design Decisions

1. **Refineries are recipe-locked**: Each Refinery runs exactly one recipe, chosen at build time. It has no base production — it only processes piped-in resources.
2. **3-tier production chains**: T1 Refineries process base→derived recipes, T2 process derived→derived, T3 process high-tier recipes. Tier is determined by the recipe's tier field.
3. **Pattern-gated unlocks**: Completing Woven Patterns unlocks the ability to build Refinery tiers. T1 after pattern 1, T2 after pattern 6, T3 after pattern 12.
4. **Resource costs**: Building a Refinery costs stockpile resources. T1: 25 of a base resource. T2: 15 of a T1 product. T3: 10 of a T2 product.
5. **Refinery limit**: Max Refineries = number of completed patterns. Starts at 1 after first pattern, grows to 18 max.
6. **Layout**: Extractors stay in fixed top 3x2 grid. Refineries appear in a scrollable processing area below, 2 columns wide.

---

### Task 1: Add LoomNodeRef and Refinery types

**Files:**
- Modify: `src/loom/types.rs`

**Step 1: Write the failing test**

Add to the `#[cfg(test)] mod tests` block at the bottom of `src/loom/types.rs`:

```rust
#[test]
fn test_loom_node_ref_equality() {
    let ext_a = LoomNodeRef::Extractor(NodeId::EmberSpindle);
    let ext_b = LoomNodeRef::Extractor(NodeId::EmberSpindle);
    let ref_a = LoomNodeRef::Refinery(0);
    let ref_b = LoomNodeRef::Refinery(0);
    let ref_c = LoomNodeRef::Refinery(1);
    assert_eq!(ext_a, ext_b);
    assert_eq!(ref_a, ref_b);
    assert_ne!(ext_a, ref_a);
    assert_ne!(ref_a, ref_c);
}

#[test]
fn test_refinery_new() {
    use super::Resource;
    let r = Refinery::new(
        Resource::Ember,
        Resource::VoidEssence,
        NodeNature::Heat,
        Resource::ForgedLight,
        1.0,
        1,
    );
    assert_eq!(r.input_a, Resource::Ember);
    assert_eq!(r.input_b, Resource::VoidEssence);
    assert_eq!(r.nature, NodeNature::Heat);
    assert_eq!(r.output, Resource::ForgedLight);
    assert!((r.amount - 1.0).abs() < 0.001);
    assert_eq!(r.tier, 1);
    assert!(!r.stalled);
    assert!((r.buffer - 0.0).abs() < 0.001);
    assert!((r.buffer_capacity - 20.0).abs() < 0.001);
    assert_eq!(r.level, 1);
}

#[test]
fn test_loom_state_default_has_empty_refineries() {
    let state = LoomState::new();
    assert!(state.persistent.refineries.is_empty());
}

#[test]
fn test_refinery_limit_zero_with_no_patterns() {
    let state = LoomState::new();
    assert_eq!(state.persistent.max_refineries(), 0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib loom::types::tests -- --nocapture 2>&1 | tail -20`
Expected: FAIL — `LoomNodeRef`, `Refinery`, `max_refineries` not defined

**Step 3: Write minimal implementation**

Add the following types to `src/loom/types.rs` (before the `LoomNode` struct):

```rust
/// Unified address for any node in the Loom — either a fixed Extractor or a built Refinery.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LoomNodeRef {
    /// One of the 6 fixed extractor nodes.
    Extractor(NodeId),
    /// A player-built refinery, identified by index in `LoomPersistent::refineries`.
    Refinery(usize),
}

/// A player-built processing node that runs a single locked recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Refinery {
    /// First input resource for this refinery's locked recipe.
    pub input_a: Resource,
    /// Second input resource for this refinery's locked recipe.
    pub input_b: Resource,
    /// The nature catalyst for this refinery's recipe.
    pub nature: NodeNature,
    /// Output resource produced.
    pub output: Resource,
    /// Output amount multiplier from the recipe.
    pub amount: f64,
    /// Recipe tier (1, 2, or 3).
    pub tier: u8,
    /// Current buffer level (holds output resource).
    #[serde(default)]
    pub buffer: f64,
    /// Buffer capacity.
    #[serde(default = "default_buffer_capacity")]
    pub buffer_capacity: f64,
    /// Refinery level (for future upgrades).
    #[serde(default = "default_node_level")]
    pub level: u32,
    /// Whether this refinery is stalled (missing inputs).
    #[serde(default)]
    pub stalled: bool,
    /// Whether currently under construction.
    #[serde(default)]
    pub under_construction: bool,
    /// Ticks remaining for construction.
    #[serde(default)]
    pub construction_ticks_remaining: u32,
}

impl Refinery {
    pub fn new(
        input_a: Resource,
        input_b: Resource,
        nature: NodeNature,
        output: Resource,
        amount: f64,
        tier: u8,
    ) -> Self {
        Self {
            input_a,
            input_b,
            nature,
            output,
            amount,
            tier,
            buffer: 0.0,
            buffer_capacity: 20.0,
            level: 1,
            stalled: false,
            under_construction: false,
            construction_ticks_remaining: 0,
        }
    }
}
```

Add `refineries` field to `LoomPersistent`:

```rust
#[serde(default)]
pub refineries: Vec<Refinery>,
```

And add the `max_refineries` method to `LoomPersistent`:

```rust
impl LoomPersistent {
    /// Maximum number of Refineries the player can build.
    /// Equal to the number of completed Woven Patterns.
    pub fn max_refineries(&self) -> usize {
        self.patterns.iter().filter(|p| p.completed).count()
    }
}
```

Update `Default for LoomPersistent` to include `refineries: Vec::new()`.

Update the `pub use types::` line in `src/loom/mod.rs` to include `LoomNodeRef` and `Refinery`.

**Step 4: Run test to verify it passes**

Run: `cargo test --lib loom::types -- --nocapture 2>&1 | tail -10`
Expected: PASS — all 5 type tests pass

**Step 5: Commit**

```bash
git add src/loom/types.rs src/loom/mod.rs
git commit -m "feat(loom): add LoomNodeRef and Refinery types"
```

---

### Task 2: Migrate Pipe from/to from NodeId to LoomNodeRef

**Files:**
- Modify: `src/loom/types.rs` (Pipe struct)
- Modify: `src/loom/pipes.rs` (all pipe functions)
- Modify: `src/loom/logic.rs` (process_reactions, node_native_resource usage)
- Modify: `src/ui/loom_scene.rs` (port labels, sidebar pipe display)
- Modify: `src/input/loom_input.rs` (pipe selection)

This is the largest task — it touches every file that references `pipe.from` or `pipe.to`. The migration is mechanical: change `Pipe::from` and `Pipe::to` from `NodeId` to `LoomNodeRef`, then update every call site to construct `LoomNodeRef::Extractor(node_id)` where it previously used bare `NodeId`.

**Step 1: Change the Pipe struct**

In `src/loom/types.rs`, change:
```rust
pub struct Pipe {
    pub from: LoomNodeRef,
    pub to: LoomNodeRef,
    // ... rest unchanged
}
```

**Step 2: Update pipes.rs function signatures**

Change `build_pipe` to accept `LoomNodeRef` for `from`/`to`. Update all helper functions (`outgoing_pipe_count`, `incoming_pipe_count`, `pipe_exists`, `total_split_ratio`, `normalize_split_ratios`) to accept `LoomNodeRef`.

In the flow simulation (`tick_pipe_flow`), the node lookup must now handle both `LoomNodeRef::Extractor(id)` (finds in `loom.persistent.nodes`) and `LoomNodeRef::Refinery(idx)` (indexes into `loom.persistent.refineries`). Add a helper:

```rust
/// Resolve a LoomNodeRef to buffer/capacity/rate info.
fn resolve_node_ref(loom: &LoomState, node_ref: LoomNodeRef) -> Option<(f64, f64, f64, bool)> {
    match node_ref {
        LoomNodeRef::Extractor(id) => {
            let node = loom.persistent.nodes.iter().find(|n| n.id == id)?;
            if !node.unlocked { return None; }
            let rate = crate::loom::logic::node_effective_rate(loom, node);
            Some((node.buffer, node.buffer_capacity, rate, node.unlocked))
        }
        LoomNodeRef::Refinery(idx) => {
            let r = loom.persistent.refineries.get(idx)?;
            if r.under_construction { return None; }
            Some((r.buffer, r.buffer_capacity, 0.0, true))
        }
    }
}
```

Similarly add `resolve_node_ref_mut` for applying transfers.

**Step 3: Update logic.rs**

In `process_reactions`, when looking up pipe destinations, match on `LoomNodeRef::Extractor(id)` to get the node's nature for recipe lookup. For `LoomNodeRef::Refinery(idx)`, use the refinery's `nature` field.

In `node_native_resource`, this remains `NodeId`-based (only Extractors have native resources). Pipe flow for Refineries uses the refinery's `output` as the resource it sends downstream.

**Step 4: Update UI files**

In `src/ui/loom_scene.rs`, port labels use `pipe.from`/`pipe.to` to show colored letters. For `LoomNodeRef::Extractor(id)`, use `node_letter(id)`. For `LoomNodeRef::Refinery(idx)`, use `R` with a numeric suffix (e.g., `R1`, `R2`).

In `src/input/loom_input.rs`, the pipe selection logic uses `NodeId::ALL[loom_ui.selected_node]`. Wrap this in `LoomNodeRef::Extractor(...)` for Extractor selection. Refinery selection will be handled in Task 6.

**Step 5: Fix all compilation errors**

Run `cargo build` and fix each error. Most are mechanical `NodeId` → `LoomNodeRef::Extractor(NodeId)` wrapping.

**Step 6: Run all tests**

Run: `cargo test 2>&1 | tail -5`
Expected: All existing tests pass (no behavior change, just type widening)

**Step 7: Commit**

```bash
git add src/loom/
git commit -m "refactor(loom): migrate Pipe from/to from NodeId to LoomNodeRef"
```

---

### Task 3: Add Refinery building logic

**Files:**
- Modify: `src/loom/logic.rs`
- Modify: `src/loom/mod.rs` (re-exports)

**Step 1: Write the failing tests**

Add to `src/loom/logic.rs` test module:

```rust
#[test]
fn test_build_refinery_success() {
    let mut loom = LoomState::new();
    select_archetype(&mut loom, LoomArchetype::BurnBright);
    // Give pattern completion for capacity.
    loom.persistent.patterns = crate::loom::discovery::create_pattern_sequence();
    loom.persistent.patterns[0].completed = true;
    // Stock resources.
    *loom.persistent.stockpiles.entry(Resource::Ember).or_insert(0.0) += 50.0;

    let result = build_refinery(
        &mut loom,
        Resource::Ember,
        Resource::VoidEssence,
        NodeNature::Heat,
    );
    assert!(result.is_ok());
    assert_eq!(loom.persistent.refineries.len(), 1);
    let r = &loom.persistent.refineries[0];
    assert_eq!(r.output, Resource::ForgedLight);
    assert!(r.under_construction);
}

#[test]
fn test_build_refinery_fails_at_capacity() {
    let mut loom = LoomState::new();
    select_archetype(&mut loom, LoomArchetype::BurnBright);
    // No patterns completed → max_refineries = 0.
    let result = build_refinery(
        &mut loom,
        Resource::Ember,
        Resource::VoidEssence,
        NodeNature::Heat,
    );
    assert!(result.is_err());
}

#[test]
fn test_build_refinery_fails_insufficient_resources() {
    let mut loom = LoomState::new();
    select_archetype(&mut loom, LoomArchetype::BurnBright);
    loom.persistent.patterns = crate::loom::discovery::create_pattern_sequence();
    loom.persistent.patterns[0].completed = true;
    // No stockpile resources.

    let result = build_refinery(
        &mut loom,
        Resource::Ember,
        Resource::VoidEssence,
        NodeNature::Heat,
    );
    assert!(result.is_err());
}

#[test]
fn test_build_refinery_fails_invalid_recipe() {
    let mut loom = LoomState::new();
    select_archetype(&mut loom, LoomArchetype::BurnBright);
    loom.persistent.patterns = crate::loom::discovery::create_pattern_sequence();
    loom.persistent.patterns[0].completed = true;
    *loom.persistent.stockpiles.entry(Resource::Ember).or_insert(0.0) += 50.0;

    // WovenReality + WovenReality has no recipe.
    let result = build_refinery(
        &mut loom,
        Resource::WovenReality,
        Resource::WovenReality,
        NodeNature::Heat,
    );
    assert!(result.is_err());
}

#[test]
fn test_build_refinery_tier_gating() {
    let mut loom = LoomState::new();
    select_archetype(&mut loom, LoomArchetype::BurnBright);
    loom.persistent.patterns = crate::loom::discovery::create_pattern_sequence();
    loom.persistent.patterns[0].completed = true; // Only 1 pattern done.
    *loom.persistent.stockpiles.entry(Resource::ForgedLight).or_insert(0.0) += 50.0;
    *loom.persistent.stockpiles.entry(Resource::EchoGlass).or_insert(0.0) += 50.0;

    // T3 recipe requires 12 patterns. Should fail with only 1.
    let result = build_refinery(
        &mut loom,
        Resource::ForgedLight,
        Resource::EchoGlass,
        NodeNature::Heat,
    );
    assert!(result.is_err());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib loom::logic -- test_build_refinery 2>&1 | tail -10`
Expected: FAIL — `build_refinery` not defined

**Step 3: Write minimal implementation**

Add to `src/loom/logic.rs`:

```rust
/// Error conditions for refinery building.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefineryError {
    /// No matching recipe exists for the given inputs + nature.
    InvalidRecipe,
    /// The recipe's tier is not yet unlocked (need more pattern completions).
    TierLocked,
    /// Player has reached the max refinery count (= completed patterns).
    AtCapacity,
    /// Not enough stockpile resources to pay the build cost.
    InsufficientResources,
}

/// Build cost for a refinery based on its recipe tier.
/// T1: 25 of input_a. T2: 15 of input_a. T3: 10 of input_a.
fn refinery_build_cost(tier: u8) -> f64 {
    match tier {
        1 => 25.0,
        2 => 15.0,
        _ => 10.0,
    }
}

/// Pattern completion count required to unlock a recipe tier.
/// T1: 1 pattern. T2: 6 patterns. T3: 12 patterns.
fn refinery_tier_unlock_threshold(tier: u8) -> usize {
    match tier {
        1 => 1,
        2 => 6,
        _ => 12,
    }
}

/// Attempt to build a new Refinery locked to the recipe matching (input_a, input_b, nature).
///
/// Validates:
/// 1. A recipe exists for the inputs + nature
/// 2. The recipe's tier is unlocked (enough completed patterns)
/// 3. Player hasn't reached max refinery count
/// 4. Stockpile has enough of input_a to pay the build cost
///
/// On success, creates a Refinery under construction and deducts cost.
/// Returns Ok(refinery_index) or Err(RefineryError).
pub fn build_refinery(
    loom: &mut LoomState,
    input_a: Resource,
    input_b: Resource,
    nature: NodeNature,
) -> Result<usize, RefineryError> {
    // Look up recipe.
    let recipe = crate::loom::recipes::find_recipe(input_a, input_b, nature)
        .ok_or(RefineryError::InvalidRecipe)?;

    // Check tier gating.
    let completed_patterns = loom.persistent.patterns.iter().filter(|p| p.completed).count();
    if completed_patterns < refinery_tier_unlock_threshold(recipe.tier) {
        return Err(RefineryError::TierLocked);
    }

    // Check capacity.
    if loom.persistent.refineries.len() >= loom.persistent.max_refineries() {
        return Err(RefineryError::AtCapacity);
    }

    // Check and deduct cost from stockpile.
    let cost = refinery_build_cost(recipe.tier);
    let stockpile = loom.persistent.stockpiles.entry(input_a).or_insert(0.0);
    if *stockpile < cost {
        return Err(RefineryError::InsufficientResources);
    }
    *stockpile -= cost;

    // Create refinery.
    let refinery = Refinery::new(
        recipe.input_a,
        recipe.input_b,
        recipe.node_nature,
        recipe.output,
        recipe.amount,
        recipe.tier,
    );
    let mut r = refinery;
    r.under_construction = true;
    r.construction_ticks_remaining = crate::loom::pipes::PIPE_CONSTRUCTION_TICKS; // Same 2hr timer.
    loom.persistent.refineries.push(r);
    Ok(loom.persistent.refineries.len() - 1)
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib loom::logic -- test_build_refinery 2>&1 | tail -10`
Expected: PASS — all 5 tests pass

**Step 5: Commit**

```bash
git add src/loom/logic.rs src/loom/mod.rs
git commit -m "feat(loom): add Refinery building logic with tier gating and resource costs"
```

---

### Task 4: Add Refinery ticking (construction, processing, stall detection)

**Files:**
- Modify: `src/loom/logic.rs` (new tick functions)
- Modify: `src/core/tick_stages.rs` (wire into tick loop)

**Step 1: Write the failing tests**

```rust
#[test]
fn test_refinery_construction_completes() {
    let mut loom = LoomState::new();
    select_archetype(&mut loom, LoomArchetype::BurnBright);
    loom.persistent.refineries.push({
        let mut r = Refinery::new(
            Resource::Ember, Resource::VoidEssence, NodeNature::Heat,
            Resource::ForgedLight, 1.0, 1,
        );
        r.under_construction = true;
        r.construction_ticks_remaining = 1;
        r
    });

    let completed = tick_refinery_construction(&mut loom);
    assert_eq!(completed.len(), 1);
    assert!(!loom.persistent.refineries[0].under_construction);
}

#[test]
fn test_refinery_processing_produces_output() {
    let mut loom = LoomState::new();
    select_archetype(&mut loom, LoomArchetype::BurnBright);
    // Add a completed refinery that turns Ember+VoidEssence → ForgedLight.
    loom.persistent.refineries.push(Refinery::new(
        Resource::Ember, Resource::VoidEssence, NodeNature::Heat,
        Resource::ForgedLight, 1.0, 1,
    ));
    // Simulate deliveries: both inputs arrived this tick.
    let deliveries = vec![
        (LoomNodeRef::Refinery(0), Resource::Ember, 5.0),
        (LoomNodeRef::Refinery(0), Resource::VoidEssence, 3.0),
    ];

    let reactions = process_refinery_reactions(&mut loom, deliveries);
    assert!(!reactions.is_empty());
    // Output should be min(5.0, 3.0) * 1.0 = 3.0 ForgedLight in buffer.
    assert!((loom.persistent.refineries[0].buffer - 3.0).abs() < 0.01);
}

#[test]
fn test_refinery_stall_when_buffer_full() {
    let mut loom = LoomState::new();
    select_archetype(&mut loom, LoomArchetype::BurnBright);
    let mut r = Refinery::new(
        Resource::Ember, Resource::VoidEssence, NodeNature::Heat,
        Resource::ForgedLight, 1.0, 1,
    );
    r.buffer = r.buffer_capacity; // Full.
    loom.persistent.refineries.push(r);

    tick_refinery_stall_detection(&mut loom);
    assert!(loom.persistent.refineries[0].stalled);
}
```

**Step 2: Run tests to verify failure, then implement**

Add three functions to `src/loom/logic.rs`:

```rust
/// Tick construction for all refineries under construction.
/// Returns indices of refineries that completed this tick.
pub fn tick_refinery_construction(loom: &mut LoomState) -> Vec<usize> {
    let mut completed = Vec::new();
    for (i, r) in loom.persistent.refineries.iter_mut().enumerate() {
        if !r.under_construction { continue; }
        r.construction_ticks_remaining = r.construction_ticks_remaining.saturating_sub(1);
        if r.construction_ticks_remaining == 0 {
            r.under_construction = false;
            completed.push(i);
        }
    }
    completed
}

/// Process reactions at refineries from pipe deliveries.
/// Unlike Extractor reactions (which use node nature from NodeId),
/// Refineries have their recipe baked in — just check both inputs arrived.
pub fn process_refinery_reactions(
    loom: &mut LoomState,
    deliveries: Vec<(LoomNodeRef, Resource, f64)>,
) -> Vec<(usize, Resource, f64)> {
    let mut results = Vec::new();

    // Group deliveries by refinery index.
    let mut refinery_inputs: std::collections::HashMap<usize, Vec<(Resource, f64)>> =
        std::collections::HashMap::new();
    for (node_ref, resource, amount) in deliveries {
        if let LoomNodeRef::Refinery(idx) = node_ref {
            refinery_inputs.entry(idx).or_default().push((resource, amount));
        }
    }

    for (idx, inputs) in refinery_inputs {
        let Some(r) = loom.persistent.refineries.get(idx) else { continue };
        if r.under_construction { continue; }

        // Find amounts for each required input.
        let amt_a: f64 = inputs.iter().filter(|(res, _)| *res == r.input_a).map(|(_, a)| a).sum();
        let amt_b: f64 = inputs.iter().filter(|(res, _)| *res == r.input_b).map(|(_, a)| a).sum();

        if amt_a > 0.0 && amt_b > 0.0 {
            let output_amount = amt_a.min(amt_b) * r.amount;
            let cap = r.buffer_capacity;
            let r = &mut loom.persistent.refineries[idx];
            let space = (cap - r.buffer).max(0.0);
            let actual = output_amount.min(space);
            r.buffer += actual;
            r.stalled = false;
            results.push((idx, r.output, actual));
        }
    }

    results
}

/// Update stall flags for all refineries.
pub fn tick_refinery_stall_detection(loom: &mut LoomState) {
    for r in &mut loom.persistent.refineries {
        if r.under_construction { continue; }
        if r.buffer >= r.buffer_capacity {
            r.stalled = true;
        }
    }
}
```

Wire into `tick_loom()` in `src/core/tick_stages.rs`, after the existing `tick_pipe_construction` and before `tick_stall_detection`:

```rust
// Tick refinery construction.
let completed_refineries = crate::loom::tick_refinery_construction(loom);
if !completed_refineries.is_empty() {
    result.loom_changed = true;
}

// After tick_pipe_flow: process refinery reactions from deliveries.
let refinery_deliveries: Vec<_> = deliveries.iter()
    .filter(|(nr, _, _)| matches!(nr, crate::loom::LoomNodeRef::Refinery(_)))
    .cloned()
    .collect();
let _refinery_reactions = crate::loom::process_refinery_reactions(loom, refinery_deliveries);

// After tick_stall_detection:
crate::loom::tick_refinery_stall_detection(loom);
```

**Step 3: Run tests, verify pass**

Run: `cargo test --lib loom -- test_refinery 2>&1 | tail -10`
Expected: PASS

**Step 4: Commit**

```bash
git add src/loom/logic.rs src/core/tick_stages.rs src/loom/mod.rs
git commit -m "feat(loom): add Refinery ticking — construction, processing, stall detection"
```

---

### Task 5: Add Refinery demolishing

**Files:**
- Modify: `src/loom/logic.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_demolish_refinery() {
    let mut loom = LoomState::new();
    select_archetype(&mut loom, LoomArchetype::BurnBright);
    loom.persistent.refineries.push(Refinery::new(
        Resource::Ember, Resource::VoidEssence, NodeNature::Heat,
        Resource::ForgedLight, 1.0, 1,
    ));
    // Add a pipe pointing to this refinery.
    loom.persistent.pipes.push(Pipe {
        from: LoomNodeRef::Extractor(NodeId::EmberSpindle),
        to: LoomNodeRef::Refinery(0),
        tier: PipeTier::T1,
        split_ratio: 1.0,
        under_construction: false,
        construction_ticks_remaining: 0,
    });

    demolish_refinery(&mut loom, 0);
    assert!(loom.persistent.refineries.is_empty());
    // Pipe should also be removed.
    assert!(loom.persistent.pipes.is_empty());
}
```

**Step 2: Implement**

```rust
/// Demolish a refinery by index.
/// Removes the refinery and all pipes connected to/from it.
/// Also re-indexes any LoomNodeRef::Refinery references in remaining pipes
/// that pointed to higher-indexed refineries.
pub fn demolish_refinery(loom: &mut LoomState, idx: usize) {
    if idx >= loom.persistent.refineries.len() {
        return;
    }

    // Remove pipes connected to this refinery.
    let ref_node = LoomNodeRef::Refinery(idx);
    loom.persistent.pipes.retain(|p| p.from != ref_node && p.to != ref_node);

    // Remove the refinery.
    loom.persistent.refineries.remove(idx);

    // Re-index pipe references for refineries above the removed index.
    for pipe in &mut loom.persistent.pipes {
        if let LoomNodeRef::Refinery(ref mut i) = pipe.from {
            if *i > idx { *i -= 1; }
        }
        if let LoomNodeRef::Refinery(ref mut i) = pipe.to {
            if *i > idx { *i -= 1; }
        }
    }
}
```

**Step 3: Run tests, verify pass, commit**

```bash
git add src/loom/logic.rs src/loom/mod.rs
git commit -m "feat(loom): add Refinery demolishing with pipe cleanup and re-indexing"
```

---

### Task 6: Render Refineries in the Flow View processing area

**Files:**
- Modify: `src/ui/loom_scene.rs`

**Step 1: Update `render_flow_view` layout**

After the Extractor grid rendering, add a processing area section. The processing area uses the same cell buffer approach, rendered below the Extractors:

```rust
// ── Processing area: Refineries ─────────────────────────────────────
let refineries = &loom_state.persistent.refineries;
if !refineries.is_empty() {
    // Calculate scroll: if refineries exceed visible area, scroll based on selection.
    let refinery_row_start = 3 * row_stride; // Below 3 rows of extractors.
    let refinery_cols = 2; // 2 columns, same as extractors.

    for (i, refinery) in refineries.iter().enumerate() {
        let grid_row = i / refinery_cols;
        let grid_col = i % refinery_cols;
        let top = (refinery_row_start + grid_row * row_stride) as i32;
        let left_col = if grid_col == 0 {
            col_spacing as i32
        } else {
            (col_spacing + NODE_BOX_WIDTH + col_spacing) as i32
        };

        // Render refinery box (similar to extractor but with recipe info).
        let is_sel = loom_ui_selected_is_refinery(ui, i);
        render_refinery_box(&mut buffer, top, left_col, refinery, is_sel);
    }
}
```

**Step 2: Add `render_refinery_box` function**

Similar to `render_node_box` but shows:
- Title: recipe output name + tier badge
- Texture: gear/cog animation (distinct from extractors)
- Buffer bar: same as extractors
- Recipe slots: both input indicators always shown (filled/empty based on active pipes)

```rust
fn render_refinery_box(
    buffer: &mut [Vec<SceneCell>],
    top: i32,
    left: i32,
    refinery: &crate::loom::types::Refinery,
    selected: bool,
) -> i32 {
    // Same box structure as render_node_box but with refinery-specific content.
    // Title: "T1 → ForgedLight" or recipe output name.
    // Texture rows: gear animation (⚙ pattern cycling).
    // Buffer bar: same green/yellow/red coloring.
    // Recipe line: [●Emb] [●Void] > FrgLt (always shows locked recipe).
    // ... (implementation mirrors render_node_box)
    top + NODE_BOX_HEIGHT as i32
}
```

**Step 3: Update navigation to include Refineries**

In `LoomUiState`, the `selected_node` index needs to cover both Extractors (0-5) and Refineries (6+). When `selected_node >= 6`, it refers to `refineries[selected_node - 6]`.

**Step 4: Update sidebar**

When a Refinery is selected, the sidebar shows:
- Refinery identity (recipe, tier)
- Buffer + rate
- Input status (which inputs are connected via pipes)
- Controls: [D]emolish

**Step 5: Run `cargo build`, verify no errors, commit**

```bash
git add src/ui/loom_scene.rs
git commit -m "feat(loom): render Refineries in Flow View processing area"
```

---

### Task 7: Add Refinery input handling (build, demolish, navigation)

**Files:**
- Modify: `src/input/loom_input.rs`

**Step 1: Extend navigation**

Down arrow past the last Extractor row enters the Refinery area. Up arrow from the first Refinery row returns to Extractors. Left/Right works within each row (2 columns).

```rust
// In FlowView, total selectable items = 6 extractors + refineries.len()
let total_nodes = 6 + loom_state.persistent.refineries.len();
```

**Step 2: Add build keybinding**

Add `B` key handling in FlowView:
- Opens a recipe selection sub-menu (or builds at current selection)
- For now: `B` when an Extractor is selected opens a list of unlocked recipes that use that Extractor's native resource
- Player picks a recipe → `build_refinery()` is called

**Step 3: Add demolish keybinding**

`D` key when a Refinery is selected calls `demolish_refinery()`.

**Step 4: Write tests for navigation bounds**

```rust
#[test]
fn test_navigation_extends_to_refineries() {
    let mut state = LoomState::new();
    select_archetype(&mut state, LoomArchetype::BurnBright);
    state.persistent.refineries.push(Refinery::new(
        Resource::Ember, Resource::VoidEssence, NodeNature::Heat,
        Resource::ForgedLight, 1.0, 1,
    ));
    let mut ui = make_ui(LoomView::FlowView);
    ui.selected_node = 4; // Bottom-left extractor.

    handle_loom(key(KeyCode::Down), &mut state, &mut ui);
    assert_eq!(ui.selected_node, 6, "should enter refinery area");

    handle_loom(key(KeyCode::Up), &mut state, &mut ui);
    assert_eq!(ui.selected_node, 4, "should return to extractors");
}
```

**Step 5: Run tests, verify pass, commit**

```bash
git add src/input/loom_input.rs
git commit -m "feat(loom): add Refinery input handling — build, demolish, navigation"
```

---

### Task 8: Add debug menu actions for Refineries

**Files:**
- Modify: `src/utils/debug_menu.rs`

**Step 1: Add debug actions**

Add to the Loom section of the debug menu:
- "Build T1 Refinery (Ember+Void→ForgedLight)" — instant build, skips cost/construction
- "Build T2 Refinery (ForgedLight+Reflection→EchoGlass)" — instant build
- "Clear All Refineries" — removes all refineries and their pipes

**Step 2: Implement**

```rust
Self::LoomBuildTestRefinery(tier) => {
    let (a, b, nature) = match tier {
        1 => (Resource::Ember, Resource::VoidEssence, NodeNature::Heat),
        2 => (Resource::ForgedLight, Resource::Reflection, NodeNature::Form),
        _ => (Resource::ForgedLight, Resource::EchoGlass, NodeNature::Heat),
    };
    if let Some(recipe) = crate::loom::recipes::find_recipe(a, b, nature) {
        let r = crate::loom::types::Refinery::new(
            recipe.input_a, recipe.input_b, recipe.node_nature,
            recipe.output, recipe.amount, recipe.tier,
        );
        loom.persistent.refineries.push(r);
        "Refinery built (debug)."
    } else {
        "No recipe found."
    }
}
```

**Step 3: Run `cargo build`, verify, commit**

```bash
git add src/utils/debug_menu.rs
git commit -m "feat(loom): add debug menu actions for Refineries"
```

---

### Task 9: Update Refinery pipe flow integration

**Files:**
- Modify: `src/loom/pipes.rs`

**Step 1: Update `tick_pipe_flow` to handle Refinery buffers**

The flow simulation currently only reads/writes `loom.persistent.nodes` buffers. After Task 2's `LoomNodeRef` migration, the resolve helper handles both. This task ensures:

1. Pipes from Extractors to Refineries drain Extractor buffer, add to Refinery input tracking
2. Pipes from Refineries to other nodes drain Refinery buffer (output resource)
3. Refinery output resource is determined by `refinery.output`, not `node_native_resource()`

**Step 2: Write test**

```rust
#[test]
fn test_pipe_flow_extractor_to_refinery() {
    let mut loom = LoomState::new();
    select_archetype(&mut loom, LoomArchetype::BurnBright);
    // Fill Ember Spindle buffer.
    loom.persistent.nodes.iter_mut()
        .find(|n| n.id == NodeId::EmberSpindle).unwrap()
        .buffer = 10.0;
    // Add a completed refinery.
    loom.persistent.refineries.push(Refinery::new(
        Resource::Ember, Resource::VoidEssence, NodeNature::Heat,
        Resource::ForgedLight, 1.0, 1,
    ));
    // Add pipe from Ember to Refinery 0.
    loom.persistent.pipes.push(Pipe {
        from: LoomNodeRef::Extractor(NodeId::EmberSpindle),
        to: LoomNodeRef::Refinery(0),
        tier: PipeTier::T1,
        split_ratio: 1.0,
        under_construction: false,
        construction_ticks_remaining: 0,
    });

    let deliveries = tick_pipe_flow(&mut loom, 3600.0); // 1 hour.
    assert!(!deliveries.is_empty(), "should have transferred resources");
}
```

**Step 3: Implement, test, commit**

```bash
git add src/loom/pipes.rs
git commit -m "feat(loom): integrate Refinery buffers into pipe flow simulation"
```

---

### Task 10: Visual polish and full integration test

**Files:**
- Modify: `src/ui/loom_scene.rs` (refinery texture animation)
- Run: `make check`

**Step 1: Add refinery-specific texture**

Refineries use a gear/cog animation pattern distinct from Extractor textures:
```
⚙ ∙ ⚙ ∙ ⚙ ∙ ⚙ ∙ ⚙ ∙ ⚙ ∙   (frame 0)
∙ ⚙ ∙ ⚙ ∙ ⚙ ∙ ⚙ ∙ ⚙ ∙ ⚙   (frame 1)
```

**Step 2: Run full CI checks**

Run: `make check`
Expected: All checks pass (format, clippy, tests, build)

**Step 3: Commit**

```bash
git add src/ui/loom_scene.rs
git commit -m "feat(loom): add Refinery texture animation and visual polish"
```

---

### Task 11: Update CLAUDE.md documentation

**Files:**
- Modify: `src/loom/CLAUDE.md` (if exists, otherwise skip)

Document:
- New types: `LoomNodeRef`, `Refinery`, `RefineryError`
- New functions: `build_refinery`, `demolish_refinery`, `tick_refinery_construction`, `process_refinery_reactions`, `tick_refinery_stall_detection`
- New UI: Processing area layout, Refinery node boxes, extended navigation
- Design decisions: recipe-locked, 3-tier, pattern-gated, resource-cost

**Commit:**

```bash
git add src/loom/CLAUDE.md
git commit -m "docs(loom): document Refinery system in CLAUDE.md"
```
