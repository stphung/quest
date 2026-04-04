# Loom Graph View Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the Loom's text-based FlowView with an interactive DAG visualization using petgraph and ratatui Canvas, with animated edges showing resource flow.

**Architecture:** petgraph `StableDiGraph` as UI-only derived state in `LoomUiState`, Sugiyama layout engine for node positioning, Canvas renderer with particle animation. Existing tick logic unchanged — graph is rebuilt from `LoomState` on structural changes, rates updated per tick.

**Tech Stack:** Rust, petgraph (new dep), ratatui 0.30 Canvas widget (HalfBlock mode)

**Spec:** `docs/superpowers/specs/2026-04-04-loom-graph-view-design.md`

---

## File Map

### New Files
| File | Responsibility |
|------|---------------|
| `src/loom/graph.rs` | Build petgraph from LoomState, node/edge types, rebuild logic, rate updates, ghost nodes |
| `src/loom/layout.rs` | Sugiyama layout: layer assignment, dummy nodes, crossing minimization, coordinate assignment, zoom-to-fit |
| `src/ui/loom_graph.rs` | Canvas renderer: node shapes, edges, particle animation, glow propagation |

### Modified Files
| File | Changes |
|------|---------|
| `Cargo.toml` | Add `petgraph` dependency |
| `src/loom/types.rs` | Add `output_rate_tracker` to Shuttle, new fields on LoomUiState, rename FlowView→GraphView, MAX_SHUTTLES constant |
| `src/loom/mod.rs` | Re-export graph and layout modules |
| `src/loom/logic.rs` | Update per-shuttle rate tracker in tick, enforce MAX_SHUTTLES in build_shuttle() |
| `src/ui/loom_scene.rs` | Route GraphView to new renderer, adjust bottom panel layout |
| `src/input/loom_input.rs` | Graph-topology navigation, update FlowView→GraphView match arms |

---

## Task 1: Add petgraph dependency and type foundations

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/loom/types.rs`
- Modify: `src/loom/mod.rs`

- [ ] **Step 1: Add petgraph to Cargo.toml**

Add `petgraph = "0.7"` to `[dependencies]` in `Cargo.toml`.

- [ ] **Step 2: Add `output_rate_tracker` to Shuttle struct**

In `src/loom/types.rs`, add to the `Shuttle` struct (after line 149, before the closing `}`):

```rust
/// Per-shuttle output rate tracker (transient, not serialized).
#[serde(skip)]
pub output_rate_tracker: RateTracker,
```

And in `Shuttle::new()` (line 164-179), add `output_rate_tracker: RateTracker::new()` to the initializer.

- [ ] **Step 3: Add MAX_SHUTTLES constant**

In `src/loom/types.rs`, add near the top constants area:

```rust
/// Maximum number of shuttles a player can build (balance cap).
pub const MAX_SHUTTLES: usize = 12;
```

- [ ] **Step 4: Update `max_shuttles()` to use MAX_SHUTTLES**

In `src/loom/types.rs`, change `LoomPersistent::max_shuttles()` (line 321-323) from:

```rust
pub fn max_shuttles(&self) -> usize {
    self.completed_pattern_count()
}
```

To:

```rust
pub fn max_shuttles(&self) -> usize {
    // Shuttle slots unlock at pattern milestones, capped at MAX_SHUTTLES
    let patterns = self.completed_pattern_count();
    let slots = match patterns {
        0 => 0,
        1..=2 => 1,
        3..=5 => 2,
        6..=9 => 4,
        10..=14 => 6,
        15..=20 => 8,
        21..=27 => 10,
        _ => MAX_SHUTTLES,
    };
    slots.min(MAX_SHUTTLES)
}
```

- [ ] **Step 5: Rename LoomView::FlowView to LoomView::GraphView**

In `src/loom/types.rs` line 378-381, rename the variant:

```rust
pub enum LoomView {
    GraphView,
    Codex,
}
```

Update `LoomUiState::new()` (line 437) to use `LoomView::GraphView`.

- [ ] **Step 6: Add graph-related fields to LoomUiState**

In `src/loom/types.rs`, replace the `LoomUiState` struct (line 419-431) with:

```rust
use petgraph::stable_graph::{EdgeIndex, NodeIndex};

pub struct LoomUiState {
    pub open: bool,
    pub view: LoomView,
    /// Selected node in the graph (replaces old selected_node: usize).
    pub selected_graph_node: Option<NodeIndex>,
    pub codex_column: usize,
    pub codex_row: usize,
    pub throbber_frame: u32,
    pub build: Option<BuildState>,
    /// Particle animation phase per edge (0.0..1.0), transient.
    pub particle_phases: HashMap<EdgeIndex, f64>,
    /// The built production graph (derived, not persisted).
    pub loom_graph: Option<LoomGraph>,
    /// Layout positions for the graph (derived, not persisted).
    pub loom_layout: Option<LoomLayout>,
    /// Whether the graph needs rebuilding (set by structural changes).
    pub graph_dirty: bool,
}
```

Remove the old `selected_node: usize` field.

Update `LoomUiState::new()` to initialize the new fields (`None` for graph/layout/selected, empty HashMap, `true` for dirty).

Also add `graph_dirty: bool` to `LoomState` (NOT `LoomPersistent` — skip serialization with `#[serde(skip)]`). This flag is set by tick-path logic (build/demolish/upgrade) where `LoomUiState` is not available, then copied to `LoomUiState.graph_dirty` in the render path.

- [ ] **Step 6b: Note — existing tests may break**

The `max_shuttles()` change (Step 4) will break any tests that assert the old 1:1 pattern-to-shuttle behavior. Find these with `cargo test 2>&1 | grep FAILED` and update expected values to match the new milestone curve.

- [ ] **Step 7: Fix all FlowView → GraphView references across codebase**

Run: `cargo build 2>&1 | head -50`

Fix every compile error from the rename. Key files:
- `src/input/loom_input.rs`: all `LoomView::FlowView` match arms
- `src/ui/loom_scene.rs`: all `LoomView::FlowView` match arms
- Any other files referencing `FlowView`

- [ ] **Step 8: Fix all `selected_node` → `selected_graph_node` references**

This will cause compile errors in `src/input/loom_input.rs` and `src/ui/loom_scene.rs`. For now, stub the navigation:
- In input handling, temporarily comment out diamond grid navigation logic (it will be replaced in Task 6)
- Replace `selected_node` reads with `selected_graph_node` returning a default/None where needed

The goal is to get the project compiling. Navigation will be fully rewritten in Task 6.

- [ ] **Step 9: Update mod.rs re-exports**

In `src/loom/mod.rs`, add (the modules don't exist yet, so gate them):

```rust
pub mod graph;
pub mod layout;
```

Create empty stub files `src/loom/graph.rs` and `src/loom/layout.rs` with just `// TODO: implement` so the module declarations compile.

- [ ] **Step 10: Verify everything compiles**

Run: `cargo build 2>&1`
Expected: Compiles with no errors (warnings OK).

- [ ] **Step 11: Run existing tests**

Run: `cargo test 2>&1 | tail -20`
Expected: All existing tests pass. Some loom tests may need updating if they reference `selected_node` or `max_shuttles()` behavior.

- [ ] **Step 12: Commit**

```bash
git add -A && git commit -m "feat(loom): add petgraph dep, type foundations for graph view

- Add petgraph 0.7 dependency
- Add output_rate_tracker to Shuttle (transient, serde skip)
- Add MAX_SHUTTLES=12 constant, update max_shuttles() with milestone curve
- Rename LoomView::FlowView → GraphView
- Replace selected_node: usize with selected_graph_node: Option<NodeIndex>
- Add particle_phases and graph_dirty to LoomUiState
- Stub graph.rs and layout.rs modules"
```

---

## Task 2: Graph data layer (`src/loom/graph.rs`)

**Files:**
- Create: `src/loom/graph.rs`
- Test: `src/loom/graph.rs` (inline `#[cfg(test)]`)

- [ ] **Step 1: Write tests for graph construction**

In `src/loom/graph.rs`, write the module with test-first approach:

```rust
use petgraph::stable_graph::{NodeIndex, EdgeIndex, StableDiGraph};
use std::collections::HashMap;
use super::types::*;

/// Node types in the Loom production graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LoomGraphNode {
    Extractor(NodeId),
    Shuttle(usize),
    PatternSink(usize),
}

/// Edge weight carrying resource and flow rate info.
#[derive(Debug, Clone)]
pub struct LoomEdge {
    pub resource: Resource,
    pub current_rate: f64,
    pub max_rate: f64,
}

/// The built graph plus lookup tables.
pub struct LoomGraph {
    pub graph: StableDiGraph<LoomGraphNode, LoomEdge>,
    pub node_indices: HashMap<LoomGraphNode, NodeIndex>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_loom_has_only_unlocked_extractors() {
        let mut loom = LoomState::new();
        // After initialize, only EmberSpindle is unlocked
        loom.persistent.nodes[0].unlocked = true;
        let graph = build_graph(&loom);
        // Should have 1 extractor node (only unlocked ones)
        let extractor_count = graph.graph.node_weights()
            .filter(|n| matches!(n, LoomGraphNode::Extractor(_)))
            .count();
        assert_eq!(extractor_count, 1);
        assert_eq!(graph.graph.edge_count(), 0);
    }

    #[test]
    fn test_shuttle_creates_edges_from_sources() {
        let mut loom = LoomState::new();
        for node in &mut loom.persistent.nodes {
            node.unlocked = true;
        }
        // Build a T1 shuttle: Ember + Reflection → ForgedLight
        loom.persistent.shuttles.push(Shuttle::new(
            Resource::Ember, Resource::Reflection,
            NodeNature::Heat, Resource::ForgedLight, 1.0, 1,
            vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
            vec![LoomNodeRef::Extractor(NodeId::ReflectionLens)],
        ));
        let graph = build_graph(&loom);
        // Should have 6 extractors + 1 shuttle = 7 nodes
        assert_eq!(graph.graph.node_count(), 7);
        // Should have 2 edges: EmberSpindle→Shuttle, ReflectionLens→Shuttle
        assert_eq!(graph.graph.edge_count(), 2);
    }

    #[test]
    fn test_pattern_sink_gets_inferred_edges() {
        let mut loom = LoomState::new();
        for node in &mut loom.persistent.nodes {
            node.unlocked = true;
        }
        // Add shuttle producing ForgedLight
        loom.persistent.shuttles.push(Shuttle::new(
            Resource::Ember, Resource::Reflection,
            NodeNature::Heat, Resource::ForgedLight, 1.0, 1,
            vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
            vec![LoomNodeRef::Extractor(NodeId::ReflectionLens)],
        ));
        // Set active pattern requiring ForgedLight
        // Note: active_pattern is usize (not Option), 0 = first pattern
        loom.persistent.active_pattern = 0;
        loom.persistent.patterns[0].requirements = vec![
            PatternRequirement {
                resource: Resource::ForgedLight,
                required_rate: 30.0,
                sustain_duration_secs: 3600.0,
                sustained_secs: 0.0,
                completed: false,
                amount: 0.0,
                accumulated: 0.0,
            },
        ];
        let graph = build_graph(&loom);
        // Should have 6 extractors + 1 shuttle + 1 pattern sink = 8 nodes
        assert_eq!(graph.graph.node_count(), 8);
        // 2 extractor→shuttle edges + 1 shuttle→pattern edge = 3
        assert_eq!(graph.graph.edge_count(), 3);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib loom::graph 2>&1`
Expected: FAIL — `build_graph` function not defined yet.

- [ ] **Step 3: Implement `build_graph()`**

In `src/loom/graph.rs`, implement:

```rust
use super::logic::shuttle_effective_intake_cap;

/// Build the production graph from current LoomState.
///
/// Nodes: unlocked extractors + all shuttles + visible pattern sinks.
/// Edges: source→shuttle (from shuttle.sources_a/b) + shuttle→pattern (inferred).
pub fn build_graph(loom: &LoomState) -> LoomGraph {
    let mut graph = StableDiGraph::new();
    let mut node_indices = HashMap::new();

    // Add unlocked extractors
    for node in &loom.persistent.nodes {
        if node.unlocked {
            let gn = LoomGraphNode::Extractor(node.id);
            let idx = graph.add_node(gn.clone());
            node_indices.insert(gn, idx);
        }
    }

    // Add all shuttles (including under construction)
    for (i, _shuttle) in loom.persistent.shuttles.iter().enumerate() {
        let gn = LoomGraphNode::Shuttle(i);
        let idx = graph.add_node(gn.clone());
        node_indices.insert(gn, idx);
    }

    // Add visible pattern sinks
    let visible_patterns = visible_pattern_indices(loom);
    for &pat_idx in &visible_patterns {
        let gn = LoomGraphNode::PatternSink(pat_idx);
        let idx = graph.add_node(gn.clone());
        node_indices.insert(gn, idx);
    }

    // Add edges: sources → shuttles
    for (i, shuttle) in loom.persistent.shuttles.iter().enumerate() {
        let shuttle_idx = node_indices[&LoomGraphNode::Shuttle(i)];
        let full_cap = shuttle_effective_intake_cap(shuttle.tier, shuttle.level);
        // Per-edge max_rate is the intake cap divided by number of sources for that slot
        let cap_a = if shuttle.sources_a.is_empty() { full_cap } else { full_cap / shuttle.sources_a.len() as f64 };
        let cap_b = if shuttle.sources_b.is_empty() { full_cap } else { full_cap / shuttle.sources_b.len() as f64 };

        for (source_ref, max_rate) in shuttle.sources_a.iter().map(|s| (s, cap_a))
            .chain(shuttle.sources_b.iter().map(|s| (s, cap_b))) {
            let source_gn = match source_ref {
                LoomNodeRef::Extractor(id) => LoomGraphNode::Extractor(*id),
                LoomNodeRef::Shuttle(idx) => LoomGraphNode::Shuttle(*idx),
            };
            if let Some(&source_idx) = node_indices.get(&source_gn) {
                let resource = match source_ref {
                    LoomNodeRef::Extractor(id) => {
                        super::logic::node_native_resource(*id)
                    }
                    LoomNodeRef::Shuttle(idx) => {
                        loom.persistent.shuttles[*idx].output
                    }
                };
                graph.add_edge(source_idx, shuttle_idx, LoomEdge {
                    resource,
                    current_rate: 0.0,  // Updated per tick
                    max_rate,
                });
            }
        }
    }

    // Add edges: shuttles → pattern sinks (inferred)
    for &pat_idx in &visible_patterns {
        let pattern = &loom.persistent.patterns[pat_idx];
        let sink_idx = node_indices[&LoomGraphNode::PatternSink(pat_idx)];

        for req in &pattern.requirements {
            for (i, shuttle) in loom.persistent.shuttles.iter().enumerate() {
                if shuttle.output == req.resource {
                    let shuttle_idx = node_indices[&LoomGraphNode::Shuttle(i)];
                    graph.add_edge(shuttle_idx, sink_idx, LoomEdge {
                        resource: req.resource,
                        current_rate: 0.0,
                        max_rate: req.required_rate,
                    });
                }
            }
        }
    }

    LoomGraph { graph, node_indices }
}

/// Returns indices of patterns visible on the graph.
/// Active pattern + next 1-2 incomplete patterns. Max 3.
fn visible_pattern_indices(loom: &LoomState) -> Vec<usize> {
    let mut indices = Vec::new();

    // Active pattern always shown (active_pattern is usize, not Option)
    let active = loom.persistent.active_pattern;
    if active < loom.persistent.patterns.len()
        && !loom.persistent.patterns[active].completed
    {
        indices.push(active);
    }

    // Next 1-2 incomplete patterns after active
    for (i, pattern) in loom.persistent.patterns.iter().enumerate() {
        if indices.len() >= 3 {
            break;
        }
        if !pattern.completed && !indices.contains(&i) {
            indices.push(i);
        }
    }

    indices
}

/// Update edge rates from per-shuttle RateTrackers.
pub fn update_edge_rates(graph: &mut LoomGraph, loom: &LoomState) {
    for edge_idx in graph.graph.edge_indices().collect::<Vec<_>>() {
        let (source, _target) = graph.graph.edge_endpoints(edge_idx).unwrap();
        let source_node = &graph.graph[source];
        let rate = match source_node {
            LoomGraphNode::Extractor(id) => {
                loom.rate_trackers
                    .get(&super::logic::node_native_resource(*id))
                    .map(|t| t.rate_per_hour())
                    .unwrap_or(0.0)
            }
            LoomGraphNode::Shuttle(idx) => {
                loom.persistent.shuttles[*idx]
                    .output_rate_tracker
                    .rate_per_hour()
            }
            LoomGraphNode::PatternSink(_) => 0.0,
        };
        graph.graph[edge_idx].current_rate = rate;
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib loom::graph 2>&1`
Expected: All 3 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/loom/graph.rs && git commit -m "feat(loom): graph data layer with petgraph construction and rate updates"
```

---

## Task 3: Update tick logic for per-shuttle rate tracking

**Files:**
- Modify: `src/loom/logic.rs`
- Test: inline tests in `src/loom/logic.rs`

- [ ] **Step 1: Write test for per-shuttle rate tracking**

In the test module of `src/loom/logic.rs`, add:

```rust
#[test]
fn test_shuttle_output_rate_tracker_updates_per_tick() {
    let mut loom = LoomState::new();
    initialize_loom(&mut loom);
    for node in &mut loom.persistent.nodes {
        node.unlocked = true;
        node.buffer = 100.0; // Fill buffers so shuttles can pull
    }
    // Complete 1 pattern to unlock T1
    loom.persistent.patterns[0].completed = true;
    // Build a shuttle
    loom.persistent.shuttles.push(Shuttle::new(
        Resource::Ember, Resource::Reflection,
        NodeNature::Heat, Resource::ForgedLight, 1.0, 1,
        vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
        vec![LoomNodeRef::Extractor(NodeId::ReflectionLens)],
    ));
    // Run a few ticks
    for _ in 0..10 {
        tick_shuttle_pull(&mut loom, 0.1);
    }
    // Shuttle's output_rate_tracker should have recorded production
    let tracker = &loom.persistent.shuttles[0].output_rate_tracker;
    assert!(tracker.rate_per_hour() > 0.0, "Shuttle rate tracker should record production");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_shuttle_output_rate_tracker 2>&1`
Expected: FAIL — tracker stays at 0 because `tick_shuttle_pull` doesn't update it yet.

- [ ] **Step 3: Update `tick_shuttle_pull()` to push to per-shuttle tracker**

In `src/loom/logic.rs`, inside `tick_shuttle_pull()`, after the shuttle produces output (where `produced` amount is calculated and added to `shuttle.buffer`), add:

```rust
shuttle.output_rate_tracker.push(produced);
```

This goes right after the line that does `shuttle.buffer += produced;` (or equivalent). Also push 0.0 for shuttles that produce nothing this tick (stalled or under construction) so the window stays aligned.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test test_shuttle_output_rate_tracker 2>&1`
Expected: PASS.

- [ ] **Step 5: Run all tests**

Run: `cargo test 2>&1 | tail -20`
Expected: All tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/loom/logic.rs && git commit -m "feat(loom): per-shuttle output rate tracking in tick_shuttle_pull"
```

---

## Task 4: Sugiyama layout engine (`src/loom/layout.rs`)

**Files:**
- Create: `src/loom/layout.rs`
- Test: `src/loom/layout.rs` (inline `#[cfg(test)]`)

- [ ] **Step 1: Write tests for layout**

```rust
use petgraph::stable_graph::{NodeIndex, StableDiGraph};
use petgraph::Direction;
use std::collections::HashMap;
use super::graph::*;
use super::types::*;

/// Computed layout positions for graph nodes.
pub struct LoomLayout {
    /// Screen-convention coordinates (x right, y down).
    pub node_positions: HashMap<NodeIndex, (f64, f64)>,
    /// Polyline waypoints for edges that span multiple layers (through dummy nodes).
    pub dummy_paths: HashMap<(NodeIndex, NodeIndex), Vec<(f64, f64)>>,
    /// Total bounds (width, height) before zoom-to-fit.
    pub bounds: (f64, f64),
}

/// Helper: determine the layer for a graph node.
/// Shuttle tiers must be looked up from LoomState since LoomGraphNode::Shuttle only stores index.
fn node_layer(node: &LoomGraphNode, loom: &LoomState) -> usize {
    match node {
        LoomGraphNode::Extractor(_) => 0,
        LoomGraphNode::Shuttle(idx) => {
            loom.persistent.shuttles.get(*idx)
                .map(|s| s.tier as usize)
                .unwrap_or(1)
        }
        LoomGraphNode::PatternSink(_) => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loom::types::*;

    fn make_test_graph() -> LoomGraph {
        // 2 extractors + 1 T1 shuttle
        let mut graph = StableDiGraph::new();
        let mut node_indices = HashMap::new();

        let e0 = graph.add_node(LoomGraphNode::Extractor(NodeId::EmberSpindle));
        let e1 = graph.add_node(LoomGraphNode::Extractor(NodeId::ReflectionLens));
        let s0 = graph.add_node(LoomGraphNode::Shuttle(0));
        node_indices.insert(LoomGraphNode::Extractor(NodeId::EmberSpindle), e0);
        node_indices.insert(LoomGraphNode::Extractor(NodeId::ReflectionLens), e1);
        node_indices.insert(LoomGraphNode::Shuttle(0), s0);

        graph.add_edge(e0, s0, LoomEdge {
            resource: Resource::Ember, current_rate: 10.0, max_rate: 20.0,
        });
        graph.add_edge(e1, s0, LoomEdge {
            resource: Resource::Reflection, current_rate: 10.0, max_rate: 20.0,
        });

        LoomGraph { graph, node_indices }
    }

    #[test]
    fn test_layout_assigns_layers_by_node_type() {
        let lg = make_test_graph();
        let loom = LoomState::new();
        let layout = compute_layout(&lg, &loom, 200.0, 100.0);
        // Extractors should be at x=0 layer, shuttle at x=1 layer
        let e0_pos = layout.node_positions[&lg.node_indices[&LoomGraphNode::Extractor(NodeId::EmberSpindle)]];
        let s0_pos = layout.node_positions[&lg.node_indices[&LoomGraphNode::Shuttle(0)]];
        assert!(s0_pos.0 > e0_pos.0, "Shuttle should be to the right of extractors");
    }

    #[test]
    fn test_layout_positions_within_bounds() {
        let lg = make_test_graph();
        let loom = LoomState::new();
        let layout = compute_layout(&lg, &loom, 200.0, 100.0);
        for (_idx, &(x, y)) in &layout.node_positions {
            assert!(x >= 0.0 && x <= 200.0, "x={x} out of bounds");
            assert!(y >= 0.0 && y <= 100.0, "y={y} out of bounds");
        }
    }

    #[test]
    fn test_layout_single_node() {
        // Single extractor, no edges
        let mut graph = StableDiGraph::new();
        let mut node_indices = HashMap::new();
        let e0 = graph.add_node(LoomGraphNode::Extractor(NodeId::EmberSpindle));
        node_indices.insert(LoomGraphNode::Extractor(NodeId::EmberSpindle), e0);
        let lg = LoomGraph { graph, node_indices };

        let loom = LoomState::new();
        let layout = compute_layout(&lg, &loom, 200.0, 100.0);
        assert_eq!(layout.node_positions.len(), 1);
        // Single node should be centered
        let pos = layout.node_positions[&e0];
        assert!((pos.0 - 100.0).abs() < 1.0, "Single node should be centered horizontally");
        assert!((pos.1 - 50.0).abs() < 1.0, "Single node should be centered vertically");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib loom::layout 2>&1`
Expected: FAIL — `compute_layout` not defined.

- [ ] **Step 3: Implement `compute_layout()`**

```rust
/// Assign each node to a layer based on its type.
fn assign_layers(lg: &LoomGraph) -> HashMap<NodeIndex, usize> {
    let mut layers = HashMap::new();
    for idx in lg.graph.node_indices() {
        let layer = match &lg.graph[idx] {
            LoomGraphNode::Extractor(_) => 0,
            LoomGraphNode::Shuttle(i) => {
                // Look up tier from the shuttle index in node_indices
                // We need to derive tier from the graph structure
                // For now, use edge depth: nodes with no incoming edges from shuttles = T1
                // This is computed below in a second pass
                1 // placeholder
            }
            LoomGraphNode::PatternSink(_) => 4,
        };
        layers.insert(idx, layer);
    }
    // Fix shuttle layers based on tier stored in node
    // We need to look up shuttle tier - pass shuttles info or encode in node
    layers
}
```

Actually, `LoomGraphNode::Shuttle(usize)` only stores the index — tier info isn't in the graph node. The layout needs access to shuttle tiers. Two approaches: (a) store tier in the graph node, or (b) pass `&LoomState` to the layout function.

Better approach: enrich the graph node to include tier:

Update `LoomGraphNode` in `graph.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LoomGraphNode {
    Extractor(NodeId),
    Shuttle { index: usize, tier: u8 },
    PatternSink(usize),
}
```

Then the layout function uses `tier` directly for layer assignment: layer = tier (1, 2, or 3).

Implement `compute_layout()`:

```rust
/// Compute layout positions for all nodes using Sugiyama-style layered layout.
///
/// `width` and `height` are the available canvas area in pixels.
/// Returns positions in screen convention (x right, y down), fitted to bounds.
pub fn compute_layout(lg: &LoomGraph, loom: &LoomState, width: f64, height: f64) -> LoomLayout {
    if lg.graph.node_count() == 0 {
        return LoomLayout {
            node_positions: HashMap::new(),
            dummy_paths: HashMap::new(),
            bounds: (width, height),
        };
    }

    // Phase 1: Layer assignment
    let mut layers: HashMap<NodeIndex, usize> = HashMap::new();
    let mut layer_nodes: Vec<Vec<NodeIndex>> = vec![vec![]; 5]; // layers 0-4

    for idx in lg.graph.node_indices() {
        let layer = node_layer(&lg.graph[idx], loom);
        layers.insert(idx, layer);
        layer_nodes[layer].push(idx);
    }

    // Phase 2: Dummy node insertion (track but don't add to graph)
    // For edges spanning >1 layer, record waypoints
    let mut dummy_paths: HashMap<(NodeIndex, NodeIndex), Vec<(f64, f64)>> = HashMap::new();
    // Dummy paths computed after coordinate assignment

    // Phase 3: Crossing minimization (barycenter heuristic)
    // Fix extractor order (NodeId::ALL order)
    // For other layers, sort by barycenter of connected nodes in previous layer
    for sweep in 0..3 {
        for l in 1..5 {
            if layer_nodes[l].is_empty() { continue; }
            let mut barycenters: Vec<(NodeIndex, f64)> = layer_nodes[l].iter().map(|&idx| {
                let neighbors: Vec<f64> = lg.graph.neighbors_directed(idx, Direction::Incoming)
                    .filter_map(|n| {
                        if layers[&n] == l - 1 {
                            let pos = layer_nodes[l - 1].iter().position(|&x| x == n)?;
                            Some(pos as f64)
                        } else {
                            None
                        }
                    })
                    .collect();
                let bc = if neighbors.is_empty() {
                    layer_nodes[l].iter().position(|&x| x == idx).unwrap() as f64
                } else {
                    neighbors.iter().sum::<f64>() / neighbors.len() as f64
                };
                (idx, bc)
            }).collect();
            barycenters.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            layer_nodes[l] = barycenters.into_iter().map(|(idx, _)| idx).collect();
        }
        // Backward sweep (right to left) on odd iterations
        if sweep % 2 == 1 {
            for l in (0..4).rev() {
                if layer_nodes[l].is_empty() || layer_nodes[l + 1].is_empty() { continue; }
                let mut barycenters: Vec<(NodeIndex, f64)> = layer_nodes[l].iter().map(|&idx| {
                    let neighbors: Vec<f64> = lg.graph.neighbors_directed(idx, Direction::Outgoing)
                        .filter_map(|n| {
                            if layers[&n] == l + 1 {
                                let pos = layer_nodes[l + 1].iter().position(|&x| x == n)?;
                                Some(pos as f64)
                            } else {
                                None
                            }
                        })
                        .collect();
                    let bc = if neighbors.is_empty() {
                        layer_nodes[l].iter().position(|&x| x == idx).unwrap() as f64
                    } else {
                        neighbors.iter().sum::<f64>() / neighbors.len() as f64
                    };
                    (idx, bc)
                }).collect();
                // Don't reorder layer 0 (extractors have fixed order)
                if l > 0 {
                    barycenters.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                    layer_nodes[l] = barycenters.into_iter().map(|(idx, _)| idx).collect();
                }
            }
        }
    }

    // Phase 4: Coordinate assignment
    let active_layers: Vec<usize> = (0..5).filter(|l| !layer_nodes[*l].is_empty()).collect();
    let num_active = active_layers.len();

    let mut node_positions = HashMap::new();

    if num_active == 1 {
        // Single layer: center everything
        let l = active_layers[0];
        let count = layer_nodes[l].len();
        for (i, &idx) in layer_nodes[l].iter().enumerate() {
            let x = width / 2.0;
            let y = if count == 1 {
                height / 2.0
            } else {
                let margin = height * 0.1;
                margin + (i as f64 / (count - 1) as f64) * (height - 2.0 * margin)
            };
            node_positions.insert(idx, (x, y));
        }
    } else {
        for (layer_rank, &l) in active_layers.iter().enumerate() {
            let x_margin = width * 0.05;
            let x = x_margin + (layer_rank as f64 / (num_active - 1) as f64) * (width - 2.0 * x_margin);
            let count = layer_nodes[l].len();
            for (i, &idx) in layer_nodes[l].iter().enumerate() {
                let y = if count == 1 {
                    height / 2.0
                } else {
                    let margin = height * 0.1;
                    margin + (i as f64 / (count - 1) as f64) * (height - 2.0 * margin)
                };
                node_positions.insert(idx, (x, y));
            }
        }
    }

    // Compute dummy paths for long edges
    for edge_idx in lg.graph.edge_indices() {
        let (src, tgt) = lg.graph.edge_endpoints(edge_idx).unwrap();
        let src_layer = layers[&src];
        let tgt_layer = layers[&tgt];
        if tgt_layer > src_layer + 1 {
            // Edge spans multiple layers — create waypoints
            let src_pos = node_positions[&src];
            let tgt_pos = node_positions[&tgt];
            let mut waypoints = vec![src_pos];
            for intermediate_layer_rank in 1..(tgt_layer - src_layer) {
                let l = src_layer + intermediate_layer_rank;
                let frac = intermediate_layer_rank as f64 / (tgt_layer - src_layer) as f64;
                let x = src_pos.0 + frac * (tgt_pos.0 - src_pos.0);
                let y = src_pos.1 + frac * (tgt_pos.1 - src_pos.1);
                waypoints.push((x, y));
            }
            waypoints.push(tgt_pos);
            dummy_paths.insert((src, tgt), waypoints);
        }
    }

    LoomLayout {
        node_positions,
        dummy_paths,
        bounds: (width, height),
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib loom::layout 2>&1`
Expected: All 3 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/loom/layout.rs && git commit -m "feat(loom): Sugiyama layout engine with crossing minimization"
```

---

## Task 5: Canvas renderer (`src/ui/loom_graph.rs`)

**Files:**
- Create: `src/ui/loom_graph.rs`
- Modify: `src/ui/loom_scene.rs`
- Modify: `src/ui/mod.rs` (if needed to declare module)

- [ ] **Step 1: Create renderer module with node drawing**

Create `src/ui/loom_graph.rs`:

```rust
use ratatui::prelude::*;
use ratatui::widgets::canvas::{Canvas, Context, Line as CanvasLine, Shape};
use ratatui::widgets::Block;
use std::collections::{HashMap, HashSet};
use petgraph::stable_graph::{EdgeIndex, NodeIndex};
use petgraph::Direction;

use crate::loom::graph::*;
use crate::loom::layout::*;
use crate::loom::types::*;

/// Colors for each resource type.
fn resource_color(resource: Resource) -> Color {
    match resource {
        Resource::Ember => Color::Rgb(255, 140, 0),
        Resource::Reflection => Color::Rgb(100, 180, 255),
        Resource::VoidEssence => Color::Rgb(160, 80, 220),
        Resource::Memory => Color::Rgb(200, 180, 120),
        Resource::Silence => Color::Rgb(120, 120, 160),
        Resource::Resonance => Color::Rgb(200, 100, 100),
        Resource::ForgedLight => Color::Rgb(255, 220, 100),
        Resource::EchoGlass => Color::Rgb(140, 220, 200),
        Resource::StillbornSong => Color::Rgb(180, 140, 200),
        _ => Color::Rgb(180, 180, 180),
    }
}

/// Glow color for edges feeding active patterns.
const GLOW_COLOR: Color = Color::Rgb(255, 200, 60);
/// Dim color for non-glowing edges.
const DIM_EDGE_COLOR: Color = Color::Rgb(60, 60, 90);
/// Selected node highlight.
const SELECTED_COLOR: Color = Color::Rgb(255, 255, 200);

/// Render the full graph view onto the given area.
pub fn render_graph_canvas(
    frame: &mut Frame,
    area: Rect,
    loom_graph: &LoomGraph,
    layout: &LoomLayout,
    ui: &LoomUiState,
    loom: &LoomState,
) {
    let canvas_width = area.width as f64;
    let canvas_height = (area.height * 2) as f64; // HalfBlock doubles vertical resolution

    // Compute glow set: edges feeding active patterns
    let glowing_edges = compute_glowing_edges(loom_graph, loom);

    let selected = ui.selected_graph_node;
    let particle_phases = &ui.particle_phases;

    let canvas = Canvas::default()
        .block(Block::new())
        .x_bounds([0.0, canvas_width])
        .y_bounds([0.0, canvas_height])
        .marker(ratatui::symbols::Marker::HalfBlock)
        .paint(move |ctx| {
            // Draw edges first (behind nodes)
            for edge_idx in loom_graph.graph.edge_indices() {
                let (src, tgt) = loom_graph.graph.edge_endpoints(edge_idx).unwrap();
                let edge = &loom_graph.graph[edge_idx];

                let src_pos = layout.node_positions[&src];
                let tgt_pos = layout.node_positions[&tgt];

                // Determine edge color
                let is_glowing = glowing_edges.contains(&edge_idx);
                let edge_color = if is_glowing { GLOW_COLOR } else { DIM_EDGE_COLOR };

                // Draw edge line (y inverted for canvas)
                let sy = canvas_height - src_pos.1;
                let ty = canvas_height - tgt_pos.1;

                // Check for dummy path (long edges)
                if let Some(waypoints) = layout.dummy_paths.get(&(src, tgt)) {
                    for i in 0..waypoints.len() - 1 {
                        let (x1, y1) = waypoints[i];
                        let (x2, y2) = waypoints[i + 1];
                        ctx.draw(&CanvasLine::new(
                            x1, canvas_height - y1,
                            x2, canvas_height - y2,
                            edge_color,
                        ));
                    }
                } else {
                    ctx.draw(&CanvasLine::new(
                        src_pos.0, sy, tgt_pos.0, ty, edge_color,
                    ));
                }

                // Draw particles along edge
                if edge.current_rate > 0.0 {
                    let phase = particle_phases.get(&edge_idx).copied().unwrap_or(0.0);
                    let particle_color = if is_glowing {
                        Color::Rgb(255, 255, 150)
                    } else {
                        resource_color(edge.resource)
                    };
                    for p in 0..3 {
                        let t = (phase + p as f64 / 3.0) % 1.0;
                        let px = src_pos.0 + t * (tgt_pos.0 - src_pos.0);
                        let py = src_pos.1 + t * (tgt_pos.1 - src_pos.1);
                        ctx.draw(&CanvasLine::new(
                            px, canvas_height - py,
                            px + 0.5, canvas_height - py,
                            particle_color,
                        ));
                    }
                }
            }

            // Draw nodes
            for idx in loom_graph.graph.node_indices() {
                let node = &loom_graph.graph[idx];
                let pos = layout.node_positions[&idx];
                let is_selected = selected == Some(idx);

                let (color, label) = match node {
                    LoomGraphNode::Extractor(id) => {
                        let c = resource_color(crate::loom::logic::node_native_resource(*id));
                        let abbrev = match id {
                            NodeId::EmberSpindle => "ES",
                            NodeId::ReflectionLens => "RL",
                            NodeId::VoidCondenser => "VC",
                            NodeId::MemoryArchive => "MA",
                            NodeId::SilenceWell => "SW",
                            NodeId::ResonanceForge => "RF",
                        };
                        (c, abbrev.to_string())
                    }
                    LoomGraphNode::Shuttle(index) => {
                        let shuttle = &loom.persistent.shuttles[*index];
                        let c = resource_color(shuttle.output);
                        let label = format!("S{}", index);
                        (c, label)
                    }
                    LoomGraphNode::PatternSink(pat_idx) => {
                        let active = loom.persistent.active_pattern == *pat_idx;
                        let c = if active { GLOW_COLOR } else { Color::Rgb(100, 100, 120) };
                        let label = format!("P{}", pat_idx + 1);
                        (c, label)
                    }
                };

                let border_color = if is_selected { SELECTED_COLOR } else { color };
                let y = canvas_height - pos.1;

                // Draw node box (small rectangle)
                let hw = 4.0; // half-width
                let hh = 3.0; // half-height
                // Top edge
                ctx.draw(&CanvasLine::new(pos.0 - hw, y - hh, pos.0 + hw, y - hh, border_color));
                // Bottom edge
                ctx.draw(&CanvasLine::new(pos.0 - hw, y + hh, pos.0 + hw, y + hh, border_color));
                // Left edge
                ctx.draw(&CanvasLine::new(pos.0 - hw, y - hh, pos.0 - hw, y + hh, border_color));
                // Right edge
                ctx.draw(&CanvasLine::new(pos.0 + hw, y - hh, pos.0 + hw, y + hh, border_color));

                // Draw label centered in box
                ctx.print(pos.0 - (label.len() as f64 / 2.0), y, Line::from(label));
            }
        });

    frame.render_widget(canvas, area);
}

/// Compute which edges are "glowing" (feeding an active pattern that's sustaining).
fn compute_glowing_edges(lg: &LoomGraph, loom: &LoomState) -> HashSet<EdgeIndex> {
    let mut glowing = HashSet::new();
    let mut glow_nodes: Vec<NodeIndex> = Vec::new();

    // Start from pattern sinks that are actively sustaining
    for idx in lg.graph.node_indices() {
        if let LoomGraphNode::PatternSink(pat_idx) = &lg.graph[idx] {
            let pattern = &loom.persistent.patterns[*pat_idx];
            let is_sustaining = pattern.requirements.iter().any(|r| {
                !r.completed && r.sustained_secs > 0.0
            });
            if is_sustaining {
                glow_nodes.push(idx);
            }
        }
    }

    // BFS upstream
    while let Some(node) = glow_nodes.pop() {
        for edge_idx in lg.graph.edges_directed(node, Direction::Incoming) {
            let eidx = edge_idx.id();
            if glowing.insert(eidx) {
                glow_nodes.push(edge_idx.source());
            }
        }
    }

    glowing
}
```

- [ ] **Step 2: Wire renderer into loom_scene.rs**

In `src/ui/loom_scene.rs`, find the `render_loom_overlay()` function's match on `LoomView::FlowView` (now `GraphView`) and replace the FlowView rendering call with:

```rust
LoomView::GraphView => {
    // Split: top 70% = graph canvas, bottom 30% = detail panel
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(content_area);

    // Render graph canvas
    if let (Some(graph), Some(layout)) = (&ui.loom_graph, &ui.loom_layout) {
        crate::ui::loom_graph::render_graph_canvas(
            frame, chunks[0], graph, layout, ui, loom_state,
        );
    }

    // Render bottom panel (detail/build/pattern)
    render_bottom_panel(frame, chunks[1], loom_state, ui);
}
```

You'll need to add `loom_graph` and `loom_layout` fields to `LoomUiState` (if not done in Task 1, add them now):

```rust
pub loom_graph: Option<LoomGraph>,
pub loom_layout: Option<LoomLayout>,
```

Create a stub `render_bottom_panel()` function that renders the selected node's basic info. This will be fleshed out later.

- [ ] **Step 3: Verify compilation**

Run: `cargo build 2>&1`
Expected: Compiles. May need to adjust imports, add module declaration in `src/ui/mod.rs`.

- [ ] **Step 4: Commit**

```bash
git add src/ui/loom_graph.rs src/ui/loom_scene.rs src/ui/mod.rs src/loom/types.rs && \
git commit -m "feat(loom): Canvas graph renderer with nodes, edges, particles, and glow"
```

---

## Task 6: Graph-topology navigation (`src/input/loom_input.rs`)

**Files:**
- Modify: `src/input/loom_input.rs`

- [ ] **Step 1: Implement graph navigation helpers**

Add helper functions for navigating the graph topology:

```rust
use petgraph::stable_graph::NodeIndex;
use petgraph::Direction;
use crate::loom::graph::*;
use crate::loom::layout::*;

/// Find siblings: nodes in the same layer as the current node.
/// Note: `loom` is needed to look up shuttle tiers for layer determination.
fn siblings_in_layer(
    graph: &LoomGraph,
    layout: &LoomLayout,
    loom: &LoomState,
    current: NodeIndex,
) -> Vec<NodeIndex> {
    use crate::loom::layout::node_layer;

    let current_layer = node_layer(&graph.graph[current], loom);

    let mut siblings: Vec<(NodeIndex, f64)> = graph.graph.node_indices()
        .filter(|&idx| node_layer(&graph.graph[idx], loom) == current_layer)
        .map(|idx| (idx, layout.node_positions[&idx].1))
        .collect();

    siblings.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    siblings.into_iter().map(|(idx, _)| idx).collect()
}

/// Navigate right: find the nearest connected node in the next tier.
fn navigate_right(
    graph: &LoomGraph,
    layout: &LoomLayout,
    current: NodeIndex,
) -> Option<NodeIndex> {
    let current_y = layout.node_positions[&current].1;

    // Look at outgoing neighbors
    let mut candidates: Vec<(NodeIndex, f64)> = graph.graph
        .neighbors_directed(current, Direction::Outgoing)
        .map(|n| (n, (layout.node_positions[&n].1 - current_y).abs()))
        .collect();

    candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    candidates.first().map(|(idx, _)| *idx)
}

/// Navigate left: find the nearest connected node in the previous tier.
fn navigate_left(
    graph: &LoomGraph,
    layout: &LoomLayout,
    current: NodeIndex,
) -> Option<NodeIndex> {
    let current_y = layout.node_positions[&current].1;

    let mut candidates: Vec<(NodeIndex, f64)> = graph.graph
        .neighbors_directed(current, Direction::Incoming)
        .map(|n| (n, (layout.node_positions[&n].1 - current_y).abs()))
        .collect();

    candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    candidates.first().map(|(idx, _)| *idx)
}
```

- [ ] **Step 2: Replace diamond navigation with graph navigation**

In `handle_loom()`, replace the arrow key handling for `GraphView` with:

```rust
LoomView::GraphView if ui.build.is_none() => {
    match key.code {
        KeyCode::Up => {
            if let (Some(graph), Some(layout), Some(current)) =
                (&ui.loom_graph, &ui.loom_layout, ui.selected_graph_node)
            {
                let sibs = siblings_in_layer(graph, layout, loom_state, current);
                if let Some(pos) = sibs.iter().position(|&n| n == current) {
                    let next = if pos == 0 { sibs.len() - 1 } else { pos - 1 };
                    ui.selected_graph_node = Some(sibs[next]);
                }
            }
        }
        KeyCode::Down => {
            if let (Some(graph), Some(layout), Some(current)) =
                (&ui.loom_graph, &ui.loom_layout, ui.selected_graph_node)
            {
                let sibs = siblings_in_layer(graph, layout, loom_state, current);
                if let Some(pos) = sibs.iter().position(|&n| n == current) {
                    let next = (pos + 1) % sibs.len();
                    ui.selected_graph_node = Some(sibs[next]);
                }
            }
        }
        KeyCode::Right => {
            if let (Some(graph), Some(layout), Some(current)) =
                (&ui.loom_graph, &ui.loom_layout, ui.selected_graph_node)
            {
                if let Some(next) = navigate_right(graph, layout, current) {
                    ui.selected_graph_node = Some(next);
                }
            }
        }
        KeyCode::Left => {
            if let (Some(graph), Some(layout), Some(current)) =
                (&ui.loom_graph, &ui.loom_layout, ui.selected_graph_node)
            {
                if let Some(next) = navigate_left(graph, layout, current) {
                    ui.selected_graph_node = Some(next);
                }
            }
        }
        // ... U, B, D hotkeys stay similar but derive LoomNodeRef from selected_graph_node
    }
}
```

- [ ] **Step 3: Update U/B/D hotkeys to work with NodeIndex cursor**

The hotkeys need to derive `LoomNodeRef` from the selected `NodeIndex`:

```rust
KeyCode::Char('u') | KeyCode::Char('U') => {
    if let Some(current) = ui.selected_graph_node {
        if let Some(graph) = &ui.loom_graph {
            match &graph.graph[current] {
                LoomGraphNode::Extractor(id) => {
                    // upgrade extractor
                    try_upgrade_node(loom_state, *id);
                }
                LoomGraphNode::Shuttle(index) => {
                    let shuttle = &loom_state.persistent.shuttles[*index];
                    if !shuttle.under_construction {
                        upgrade_shuttle(loom_state, *index, ascension_level);
                    }
                }
                _ => {} // Can't upgrade pattern sinks
            }
        }
    }
}
```

- [ ] **Step 4: Verify compilation and manual test**

Run: `cargo build 2>&1`
Expected: Compiles.

- [ ] **Step 5: Commit**

```bash
git add src/input/loom_input.rs && git commit -m "feat(loom): graph-topology navigation with arrow keys"
```

---

## Task 7: Graph rebuild and tick integration

**Files:**
- Modify: `src/loom/types.rs` (if needed)
- Modify: `src/core/tick_stages.rs` or wherever `tick_loom()` is called
- Modify: `src/ui/loom_scene.rs`

This task wires the graph lifecycle: rebuild on structural changes, update rates per tick, advance particle phases.

- [ ] **Step 1: Add graph rebuild trigger function**

In `src/loom/graph.rs`, add:

```rust
use super::layout::compute_layout;

/// Rebuild the graph and layout if dirty, then update edge rates.
/// Called each tick from the render path.
pub fn refresh_graph(
    ui: &mut LoomUiState,
    loom: &LoomState,
    canvas_width: f64,
    canvas_height: f64,
) {
    if ui.graph_dirty || ui.loom_graph.is_none() {
        let graph = build_graph(loom);
        let layout = compute_layout(&graph, loom, canvas_width, canvas_height);

        // Preserve selected node if it still exists
        if let Some(selected) = ui.selected_graph_node {
            if graph.graph.node_weight(selected).is_none() {
                ui.selected_graph_node = None;
            }
        }
        // Default selection if none
        if ui.selected_graph_node.is_none() {
            ui.selected_graph_node = graph.graph.node_indices().next();
        }

        // Reset particle phases for new edges
        ui.particle_phases.clear();
        for edge_idx in graph.graph.edge_indices() {
            ui.particle_phases.insert(edge_idx, 0.0);
        }

        ui.loom_graph = Some(graph);
        ui.loom_layout = Some(layout);
        ui.graph_dirty = false;
    }

    // Update rates every tick
    if let Some(graph) = &mut ui.loom_graph {
        update_edge_rates(graph, loom);
    }

    // Advance particle phases
    if let Some(graph) = &ui.loom_graph {
        for edge_idx in graph.graph.edge_indices() {
            let edge = &graph.graph[edge_idx];
            let speed = if edge.max_rate > 0.0 {
                edge.current_rate / edge.max_rate
            } else {
                0.0
            };
            if let Some(phase) = ui.particle_phases.get_mut(&edge_idx) {
                *phase = (*phase + 0.05 * speed) % 1.0; // 0.05 per tick = ~2 cycles/sec at full rate
            }
        }
    }
}
```

- [ ] **Step 2: Set `graph_dirty = true` on structural changes**

The tick-path functions (`build_shuttle()`, `demolish_shuttle()`, `upgrade_shuttle()`, `tick_pattern_sustain()`) only have access to `&mut LoomState`, not `LoomUiState`. Use the `graph_dirty: bool` field on `LoomState` (added in Task 1 Step 6).

In `src/loom/logic.rs`, add `loom.graph_dirty = true` after:
- `build_shuttle()` succeeds (after pushing the new shuttle)
- `demolish_shuttle()` is called (after removing the shuttle)
- `upgrade_shuttle()` succeeds (after incrementing level)
- Pattern completion in `tick_pattern_sustain()` (when a pattern completes)

In `refresh_graph()`, check both flags:
```rust
let dirty = ui.graph_dirty || loom.graph_dirty;
if dirty || ui.loom_graph.is_none() {
    // ... rebuild graph ...
    ui.graph_dirty = false;
    // Note: loom.graph_dirty is reset by the caller after refresh_graph returns
}
```

- [ ] **Step 3: Call `refresh_graph()` in render path**

In `src/ui/loom_scene.rs`, at the start of `render_loom_overlay()`, before rendering:

```rust
if ui.view == LoomView::GraphView {
    let canvas_width = area.width as f64;
    let canvas_height = (area.height as f64 * 0.7) * 2.0; // 70% of area, HalfBlock doubles
    crate::loom::graph::refresh_graph(ui, loom_state, canvas_width, canvas_height);
}
```

- [ ] **Step 4: Verify compilation**

Run: `cargo build 2>&1`
Expected: Compiles.

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "feat(loom): wire graph rebuild and tick integration"
```

---

## Task 8: Bottom panel rendering

**Files:**
- Modify: `src/ui/loom_scene.rs` or `src/ui/loom_graph.rs`

- [ ] **Step 1: Implement `render_bottom_panel()`**

The bottom panel shows different content based on what's selected:

```rust
fn render_bottom_panel(
    frame: &mut Frame,
    area: Rect,
    loom: &LoomState,
    ui: &LoomUiState,
) {
    let block = Block::bordered()
        .title(" Detail ")
        .border_style(Style::default().fg(Color::Rgb(180, 120, 220)));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if ui.build.is_some() {
        render_build_panel(frame, inner, loom, ui);
        return;
    }

    let Some(graph) = &ui.loom_graph else { return };
    let Some(selected) = ui.selected_graph_node else {
        // Empty state guidance
        let text = Paragraph::new("Press B to build your first shuttle.")
            .alignment(Alignment::Center);
        frame.render_widget(text, inner);
        return;
    };

    let Some(node) = graph.graph.node_weight(selected) else { return };

    match node {
        LoomGraphNode::Extractor(id) => {
            render_extractor_detail(frame, inner, loom, *id);
        }
        LoomGraphNode::Shuttle(index) => {
            render_shuttle_detail(frame, inner, loom, *index);
        }
        LoomGraphNode::PatternSink(pat_idx) => {
            render_pattern_detail(frame, inner, loom, *pat_idx);
        }
    }
}
```

Implement each `render_*_detail()` function showing:
- **Extractor**: name, level, buffer/capacity gauge, production rate, upgrade cost, unlock status of neighbors
- **Shuttle**: recipe, tier, level, buffer/capacity, input sources, output rate, upgrade cost (or "Under Construction: X ticks remaining")
- **Pattern**: pattern name, each requirement (resource, required rate, sustained time / total time), completion status

These can reuse rendering logic from the existing FlowView detail panel in `loom_scene.rs`.

- [ ] **Step 2: Implement `render_build_panel()`**

Adapt the existing build flow rendering to work in the bottom panel. The build steps (SelectRecipe, SelectSourcesA/B, Confirm) render in this horizontal panel area instead of as a modal.

- [ ] **Step 3: Verify compilation and test visually**

Run: `cargo build && cargo run`
Open the Loom in-game and verify the bottom panel shows node details.

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "feat(loom): bottom panel with node detail, pattern, and build views"
```

---

## Task 9: Ghost node preview during build

**Files:**
- Modify: `src/loom/graph.rs`
- Modify: `src/input/loom_input.rs`

- [ ] **Step 1: Add ghost node insertion to graph**

In `src/loom/graph.rs`, add:

```rust
/// Insert a temporary ghost node for build preview.
/// Returns the ghost node's NodeIndex.
pub fn insert_ghost_node(
    graph: &mut LoomGraph,
    tier: u8,
    sources: &[LoomNodeRef],
    output_resource: Resource,
) -> NodeIndex {
    // Use usize::MAX as sentinel index — no real shuttle will have this index
    let ghost = LoomGraphNode::Shuttle(usize::MAX);
    let idx = graph.graph.add_node(ghost.clone());
    graph.node_indices.insert(ghost, idx);

    // Add dashed-style edges from sources
    for source_ref in sources {
        let source_gn = match source_ref {
            LoomNodeRef::Extractor(id) => LoomGraphNode::Extractor(*id),
            LoomNodeRef::Shuttle(i) => LoomGraphNode::Shuttle(*i),
        };
        if let Some(&source_idx) = graph.node_indices.get(&source_gn) {
            graph.graph.add_edge(source_idx, idx, LoomEdge {
                resource: output_resource,
                current_rate: 0.0,
                max_rate: 0.0,
            });
        }
    }

    idx
}

/// Remove the ghost node and its edges.
pub fn remove_ghost_node(graph: &mut LoomGraph, ghost_idx: NodeIndex) {
    graph.graph.remove_node(ghost_idx);
    graph.node_indices.retain(|_, &mut v| v != ghost_idx);
}
```

- [ ] **Step 2: Wire ghost node into build flow**

In the input handler, when the build flow reaches SelectSourcesA/B, insert/update the ghost node in the graph and trigger layout recompute. When build is cancelled or confirmed, remove the ghost.

- [ ] **Step 3: Render ghost node with dashed borders**

In `src/ui/loom_graph.rs`, check if a shuttle node has `index == usize::MAX` (ghost sentinel) and render with a dashed border style.

- [ ] **Step 4: Verify compilation**

Run: `cargo build 2>&1`

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "feat(loom): ghost node preview during shuttle build flow"
```

---

## Task 10: Clean up old FlowView code and final polish

**Files:**
- Modify: `src/ui/loom_scene.rs`
- Modify: `src/input/loom_input.rs`

- [ ] **Step 1: Remove dead FlowView rendering code**

Delete the old diamond grid rendering functions, shuttle list rendering, and old right-sidebar detail panel code from `loom_scene.rs`. Keep only Codex rendering and the new graph/bottom panel code.

- [ ] **Step 2: Remove dead diamond navigation code**

Delete the old diamond layout constants and helper functions from `loom_input.rs` that are no longer used by graph navigation.

- [ ] **Step 3: Add minimum terminal size check**

In the graph rendering path, check if the area is at least 100x30:

```rust
if area.width < 100 || area.height < 30 {
    let msg = Paragraph::new("Terminal too small for graph view (need 100x30)")
        .alignment(Alignment::Center);
    frame.render_widget(msg, area);
    return;
}
```

- [ ] **Step 4: Run full test suite**

Run: `cargo test 2>&1 | tail -30`
Expected: All tests pass. Fix any broken tests from the FlowView → GraphView migration.

- [ ] **Step 5: Run `make check`**

Run: `make check 2>&1`
Expected: Format, clippy, test, build, and audit all pass.

- [ ] **Step 6: Commit**

```bash
git add -A && git commit -m "refactor(loom): remove old FlowView code, add terminal size check"
```

---

## Task 11: Update CLAUDE.md documentation

**Files:**
- Modify: `src/loom/CLAUDE.md`

- [ ] **Step 1: Update module structure**

Add `graph.rs` and `layout.rs` to the module structure table. Update the UI section to describe the graph view instead of FlowView.

- [ ] **Step 2: Document new constants and types**

Add `MAX_SHUTTLES`, `LoomGraphNode`, `LoomEdge`, `LoomLayout` to the key types table. Document the shuttle milestone unlock curve.

- [ ] **Step 3: Update shuttle limit documentation**

Change "Max shuttles = number of completed Woven Patterns (max 28)" to the new milestone-based curve.

- [ ] **Step 4: Commit**

```bash
git add src/loom/CLAUDE.md && git commit -m "docs(loom): update CLAUDE.md for graph view architecture"
```
