# Loom Graph View Design

**Date:** 2026-04-04
**Status:** Draft
**Branch:** fix/loom-stockpile-progression

## Overview

Replace the Loom's text-based FlowView with an interactive DAG (directed acyclic graph) visualization rendered on a ratatui Canvas widget. The graph shows extractors, shuttles, and pattern sinks as nodes with animated edges representing resource flow. The goal is spatial awareness — players see how their production network connects, where resources flow, and where bottlenecks exist.

## Motivation

The current FlowView shows extractors as a static diamond grid and shuttles as a scrollable list. Neither view shows the actual production network — which shuttles connect to which sources, how resources chain through tiers, or which paths feed active patterns. The Loom's depth comes from orchestrating multi-hop production chains, but the UI hides that orchestration entirely.

## Graph Data Layer

### petgraph Integration

Add `petgraph` as a dependency. The Loom's production network is represented as a `petgraph::stable_graph::StableDiGraph<LoomGraphNode, LoomEdge>`.

### Node Types

```rust
enum LoomGraphNode {
    Extractor(NodeId),    // 6 fixed source nodes
    Shuttle(usize),       // Player-built processors, index into loom.shuttles[]
    PatternSink(usize),   // One per visible Woven Pattern (see visibility rules below)
}
```

- **Extractor**: The 6 fixed resource producers (Ember Spindle, Reflection Lens, etc.)
- **Shuttle**: Recipe-locked processors that pull from extractors or lower-tier shuttles
- **PatternSink**: Virtual sink nodes representing Woven Patterns the player can see. Edges are **inferred**: any shuttle currently producing a resource required by the pattern gets an incoming edge to that pattern's sink. Completed patterns are omitted.

**Pattern sink visibility rules:**
- The current active pattern (the one being sustained toward) is always shown.
- The next 1-2 unlocked-but-incomplete patterns are shown as dimmer "upcoming" sinks, so players can plan ahead.
- Completed patterns are omitted.
- This keeps sink count at 2-3 max, preventing the graph from bloating with all 28 patterns.

### Edge Weight

```rust
struct LoomEdge {
    resource: Resource,
    current_rate: f64,    // units/hr from per-shuttle RateTracker
    max_rate: f64,        // theoretical max: shuttle_effective_intake_cap(tier, level)
}
```

### Per-Shuttle Rate Tracking

The existing `RateTracker` system tracks aggregate rates per `Resource` type. This is insufficient for per-edge flow visualization. **New addition**: each `Shuttle` gains a `output_rate_tracker: RateTracker` field that tracks that individual shuttle's production rate. Updated each tick alongside the existing aggregate trackers. This is the source of truth for `LoomEdge::current_rate`.

For edges between shuttles (T1→T2, T2→T3), the rate is the downstream shuttle's per-input consumption, derived from its output rate and recipe ratios.

### Graph Lifecycle

- **Rebuild** from `LoomState` on structural changes: shuttle build, demolish, pattern completion, **shuttle upgrade** (upgrades change `max_rate`). These are infrequent events.
- **Rate updates** every tick: copy `current_rate` from per-shuttle `RateTracker` into edge weights. Cheap — just number copies. `max_rate` is set during rebuild from `shuttle_effective_intake_cap(tier, level)`.
- **Derived state only**: The graph is never serialized. It is rebuilt from `LoomState` on load, keeping saves backward-compatible.
- **UI-only**: The petgraph lives in `LoomUiState`, not in the tick path. Existing manual T1→T2→T3 tick ordering in `logic.rs` is unchanged. This avoids coupling the game simulation to the UI data structure.

## Sugiyama Layout Engine

A `LoomLayout` struct takes the petgraph and produces `(x, y)` coordinates for every node.

### Four Phases

1. **Layer assignment** — Explicit from tier structure:
   - Layer 0: Extractors
   - Layer 1: T1 Shuttles
   - Layer 2: T2 Shuttles
   - Layer 3: T3 Shuttles
   - Layer 4: Pattern Sinks

2. **Dummy node insertion** — When an edge spans multiple layers (e.g., Extractor → T2 shuttle), insert invisible dummy nodes at intermediate layers so edges route cleanly through columns as polylines.

3. **Crossing minimization** — Barycenter heuristic: for each layer left-to-right, reorder nodes by the average position of their neighbors in the previous layer. Run 2-3 sweeps (forward + backward). Extractors stay in fixed order (Ember Spindle at top, Memory Archive at bottom).

4. **Coordinate assignment** — Convert layer/position into canvas pixel coordinates. Equal spacing within layers, equal spacing between layers. Center each layer vertically. **Note**: Layout engine outputs coordinates in screen convention (x increases rightward, y increases downward). The Canvas renderer inverts y when mapping to ratatui's Canvas y-bounds (which increase upward).

### Output

```rust
struct LoomLayout {
    node_positions: HashMap<NodeIndex, (f64, f64)>,
    dummy_paths: Vec<Vec<(f64, f64)>>,  // polyline waypoints for long edges
    bounds: (f64, f64),                  // total width, height
}
```

### Zoom-to-Fit

Scale all coordinates so `bounds` fits the available Canvas area (top 70% of screen). As the network grows, nodes get proportionally smaller but the full graph is always visible.

### Minimum Terminal Size

The graph view requires a minimum terminal size of **100 columns x 30 rows**. Below this, render a "Terminal too small" message consistent with the existing `TooSmall` responsive tier pattern.

### Recalculation

Layout only recomputes on structural changes (build/demolish/upgrade shuttle, pattern completion). Not every tick.

### Ghost Node Positioning

During build mode, a temporary ghost node is inserted into the petgraph at the appropriate tier layer, and layout is recomputed. Since layout is sub-millisecond for 25 nodes, this is acceptable during the interactive build flow. The ghost node and its dashed edges are removed if the build is cancelled.

## Canvas Renderer

The graph renders on a ratatui `Canvas` widget occupying the top ~70% of the screen.

### Node Rendering

- **Extractors**: Rounded rectangles, colored by resource type (Ember = orange, Void = purple, etc.). Show abbreviated name + level + buffer gauge as a small fill bar inside the box.
- **Shuttles**: Rectangles with border colored by output resource. Show recipe shorthand (e.g., "Em+Rf→FL") + level. Under-construction shuttles render with dashed borders. Navigable but not upgradable until construction completes.
- **Pattern Sinks**: Diamond shapes, visually distinct from production nodes. Show pattern name + sustain progress as an arc/ring indicator. Active pattern rendered brightly; upcoming patterns rendered dimmer.
- **Selected node**: Brighter border or highlight color.

### Edge Rendering

- Lines follow polyline waypoints from the layout engine (straight segments through dummy nodes for multi-layer edges).
- **Thickness**: 1 char-width for low throughput, 2 for medium, 3 for high (relative to max_rate). Implemented by drawing parallel lines.
- **Animation**: Each edge maintains a `particle_phase: f64` (0.0 to 1.0). Each tick, phase advances proportional to `current_rate / max_rate`. Renderer places 2-3 particle markers (`●`) along the edge at evenly spaced offsets from the phase, moving in the flow direction against a dimmer edge line (`─`). Stalled edges (rate = 0) show no particles, rendered dimmed/gray.
- **Pattern glow**: Edges feeding an actively-sustaining pattern render in gold/amber. Glow propagates upstream via BFS from pattern sinks — if a T2 feeds a glowing pattern, and a T1 feeds that T2, both edges glow. If an edge feeds multiple patterns (one glowing, one not), it renders as glowing (glow wins). Non-glowing edges render in dim gray/blue.
- **Under construction**: Incoming edges to building shuttles pulse with a dashed pattern.

### Canvas Configuration

- `Canvas::default().x_bounds([0.0, width]).y_bounds([0.0, height])` with coordinate inversion: layout y (screen-down) is mapped to `height - y` for Canvas y (math-up).
- HalfBlock marker mode for double vertical resolution (already used in fishing/shard fusion scenes).

## Navigation & Interaction

### Cursor Type

Replace the existing `selected_node: usize` flat index in `LoomUiState` with a new `selected_graph_node: Option<NodeIndex>` field. The `NodeIndex` comes from petgraph and directly addresses a node in the graph. The existing `selected_node` field is removed. `BuildState` references to `LoomNodeRef` remain unchanged — `LoomNodeRef` is derived from the selected `NodeIndex` when entering build/upgrade flows.

### View Enum

Rename `LoomView::FlowView` to `LoomView::GraphView`. All existing match arms in `loom_input.rs` and `loom_scene.rs` are updated accordingly.

### Graph Navigation

Arrow keys traverse the graph topology, not pixel space:

- **Left/Right**: Move between tiers. Right from an extractor selects the first T1 shuttle it feeds. Left from a T1 goes back to its source extractor. If multiple connections exist, picks the one closest to current vertical position.
- **Up/Down**: Move between siblings in the same tier. Wraps around.
- **Tab**: Toggle between Graph view and Codex.

Selected node is visually highlighted on the graph and populates the bottom panel.

### Bottom Panel (30% of Screen)

Three modes depending on context:

1. **Node detail** (default): Selected node's full info — level, buffer capacity/fill, production rate, recipe, sources, upgrade cost. Horizontal layout utilizing the full width.

2. **Build mode** (press B): Multi-step flow rendered in the panel:
   - Step 1: Pick recipe (filtered by tier unlock gates)
   - Step 2: Pick source A (shows available nodes)
   - Step 3: Pick source B
   - Step 4: Confirm with expected throughput
   - While building, the graph shows a **ghost node** (see Ghost Node Positioning above) with dashed edges to selected sources. Updates live as sources are picked.

3. **Pattern detail** (when pattern sink selected): All requirements, sustained time, completion progress.

### Hotkeys

- `U` — Upgrade selected node (disabled for shuttles under construction)
- `B` — Enter build mode
- `D` — Demolish selected shuttle (immediate, matching current behavior)
- `Esc` — Exit build mode, or close Loom overlay

### Empty State

When the Loom is first discovered (1 extractor, 0 shuttles, 0 patterns), the graph shows a single extractor node centered in the canvas area. The bottom panel shows introductory guidance: "Unlock more extractors by sustaining production. Press B to build your first shuttle."

### Under-Construction Nodes

Shuttles under construction appear on the graph with dashed borders and are navigable (selectable via arrow keys). The detail panel shows construction progress. Upgrade is disabled until construction completes. Demolish is allowed (cancels construction).

## Shuttle Cap

Reduce maximum shuttles from 28 (one per pattern) to **10-12** (configurable via `MAX_SHUTTLES` constant).

### Rationale

This is a **balance change**, not just a visual constraint. The shuttle cap is enforced in `build_shuttle()` game logic.

- **Challenge**: Each shuttle slot is a meaningful decision. Players must demolish and rebuild as pattern requirements shift.
- **Visual clarity**: ~20-25 total nodes (6 extractors + 10-12 shuttles + 2-3 pattern sinks) keeps the graph readable without shrinking nodes to unreadable sizes.
- **Graph aesthetics**: Sugiyama crossing minimization works well at this scale. Larger graphs produce more visual noise.

### Unlock Progression

Gate shuttle slots at milestone patterns rather than 1:1. Exact curve TBD during implementation (e.g., patterns 1, 3, 6, 10, 15, 21, 28 each grant +1 slot, capping at 10-12).

### Backward Compatibility for Shuttle Cap

Existing saves with more than `MAX_SHUTTLES` shuttles continue to function — all shuttles remain operational. Players cannot build new shuttles until count drops below the cap via demolition. No forced demolition.

## Animation & Performance

### Particle System

- Each edge stores a `particle_phase: f64` in a `HashMap<EdgeIndex, f64>` within `LoomUiState` (transient, not saved). Rebuilt alongside the graph on structural changes.
- Phase advances each tick: `particle_phase += tick_delta * speed_factor` where `speed_factor = current_rate / max_rate`. Wraps at 1.0.
- 2-3 particle markers per edge, evenly spaced.
- Stalled edges: no particles, dimmed rendering.

### Glow Propagation

- Pattern sinks with `sustained_secs > 0` and not yet completed trigger glow.
- BFS upstream through petgraph marks edges as glowing. Multiple simultaneous glowing sinks are supported — glow from any active sink propagates upstream through shared edges (glow wins over non-glow).
- With 20-25 nodes, this is sub-microsecond per tick.

### Performance Budget

- **Layout recompute**: Sub-millisecond for 25 nodes. Only on structural changes (including ghost node during build).
- **Per-tick**: Update particle phases (~25 edges) + copy rates from per-shuttle trackers (~25 edges) + glow BFS (~25 nodes). Well under 1ms total.
- **Canvas rendering**: ~25 nodes + ~25 edges with particles. Similar complexity to fishing scene which runs smoothly.
- **100ms tick budget**: No performance concerns at this scale.

## Module Architecture

### New Files

```
src/loom/graph.rs       # LoomGraph: petgraph construction from LoomState,
                        #   node/edge types, rebuild on structural change,
                        #   tick-rate updates, ghost node management

src/loom/layout.rs      # Sugiyama layout engine: layer assignment, dummy
                        #   nodes, crossing minimization, coordinate
                        #   assignment, zoom-to-fit

src/ui/loom_graph.rs    # Canvas renderer: node shapes, edge drawing,
                        #   particle animation, glow propagation,
                        #   ghost node preview during build
```

### Modified Files

- `Cargo.toml` — add `petgraph` dependency
- `src/loom/types.rs` — add graph/layout types, `MAX_SHUTTLES` constant, `output_rate_tracker: RateTracker` on `Shuttle`, `selected_graph_node: Option<NodeIndex>` + `particle_phases: HashMap<EdgeIndex, f64>` + graph/layout fields on `LoomUiState`, rename `LoomView::FlowView` → `LoomView::GraphView`
- `src/loom/mod.rs` — re-export `graph` and `layout` modules
- `src/loom/logic.rs` — enforce new shuttle cap in `build_shuttle()`, update per-shuttle `output_rate_tracker` in tick
- `src/ui/loom_scene.rs` — replace FlowView rendering with `loom_graph.rs` call, keep Codex, adjust bottom panel layout. Renderer receives both `&LoomState` and `&mut LoomUiState` (existing parameter threading pattern).
- `src/input/loom_input.rs` — graph-topology navigation replacing diamond grid logic, update all `FlowView` match arms to `GraphView`

### Unchanged Files

- `src/loom/persistence.rs` — graph is derived state, not persisted
- `src/loom/recipes.rs` — recipe definitions untouched
- Codex view rendering in `loom_scene.rs`
- Core tick ordering logic (manual T1→T2→T3 in `tick_shuttle_pull()`)
- Pattern sustain logic

## Codex View

The Codex stays as a separate Tab view, unchanged. It serves as the recipe discovery reference — showing what resources and recipes exist, which are discovered, and how they relate. The Graph view shows the live production network; the Codex shows the possibility space.

## Backward Compatibility

- The petgraph, layout, and animation state are **derived/transient** — rebuilt from `LoomState` on load.
- No changes to the save format (`loom.json`).
- New `output_rate_tracker` field on `Shuttle` is transient (not serialized), initialized empty on load.
- Shuttle cap reduction handled gracefully (see Shuttle Cap section).
- `LoomView::FlowView` → `LoomView::GraphView` rename: `FlowView` was not persisted, so no save migration needed.
