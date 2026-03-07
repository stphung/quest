# Loom of Worlds

A mid-to-late game resource production and crafting system. Players manage 6 fixed Extractor nodes and buildable Refinery nodes, connecting them with Pipes to create Factorio-style production chains. Resources flow through the network, combining via recipes to produce higher-tier materials needed to sustain Woven Patterns.

## Module Structure

```
src/loom/
├── mod.rs          — Public API re-exports
├── types.rs        — All data structures: NodeId, LoomNodeRef, Resource, Pipe, Refinery, LoomState, etc.
├── logic.rs        — Node upgrades, base production, stall detection, reactions, refinery building/demolishing
├── pipes.rs        — Pipe building/demolishing/upgrading, flow simulation, split ratios
├── recipes.rs      — Recipe registry, lookup_recipe(a, b, nature), recipes_by_nature()
├── patterns.rs     — Woven Pattern sustain timer and completion tracking
├── discovery.rs    — 18 woven patterns defined in create_pattern_sequence()
└── persistence.rs  — Save/load from ~/.quest/loom.json
```

## Key Types

| Type | Purpose |
|------|---------|
| `NodeId` | Enum of 6 fixed extractor identities (EmberSpindle, VoidCondenser, etc.) |
| `LoomNodeRef` | Unified addressing: `Extractor(NodeId)` or `Refinery(usize)` — used in all pipe from/to fields |
| `LoomNode` | Extractor state: level, buffers per resource, nature, stall flag, unlocked status |
| `Refinery` | Recipe-locked processing node: input_a/b, nature, output, amount, tier, buffer, construction state |
| `Pipe` | Connection between two `LoomNodeRef` endpoints with tier, split ratio, construction state |
| `Resource` | Enum of all resources: 6 base + confluence + reaction products |
| `WovenPattern` | Pattern with resource requirements, sustain timer, completion state |
| `LoomState` | Top-level state: `persistent` (saved) + `ui_state` (transient) |
| `LoomPersistent` | Saved state: nodes, pipes, refineries, stockpile, codex, patterns |

## Node Addressing with LoomNodeRef

All pipe endpoints use `LoomNodeRef` instead of raw `NodeId`:

```rust
pub enum LoomNodeRef {
    Extractor(NodeId),  // Fixed nodes, identified by NodeId enum
    Refinery(usize),    // Dynamic nodes, identified by index in refineries vec
}
```

This means pipe building, flow simulation, reactions, and UI all handle both node types through match arms on `LoomNodeRef`.

## Refinery System

Refineries are recipe-locked processing nodes that create multi-step production chains.

**Building**: `build_refinery(loom, recipe_index)` validates tier unlock, resource cost, and refinery limit, then pushes a new `Refinery` to `loom.persistent.refineries`.

**Tiers and gating**:
| Tier | Pattern gate | Resource cost |
|------|-------------|---------------|
| T1 | 1 pattern complete | 25 of input_a resource |
| T2 | 6 patterns complete | 15 of input_a resource |
| T3 | 12 patterns complete | 10 of input_a resource |

**Refinery limit**: Max refineries = number of completed Woven Patterns (max 18).

**Construction**: Refineries have a 50-tick construction period. `tick_refinery_construction()` decrements timers and marks them ready.

**Processing**: `process_refinery_reactions()` converts buffered inputs into outputs using the locked recipe, depositing results into the refinery's output buffer.

**Demolishing**: `demolish_refinery(loom, idx)` removes the refinery and all connected pipes, then re-indexes remaining refinery pipe references.

**Stall detection**: `tick_refinery_stall_detection()` marks refineries as stalled when output buffers are full with no outgoing pipes to drain them.

## Pipe Flow System

Pipes transport resources between nodes. Key functions in `pipes.rs`:

- `build_pipe(loom, from, to, tier)` — creates a pipe with construction delay
- `tick_pipe_flow(loom)` — simulates resource flow each tick, handling both Extractor and Refinery endpoints
- `pipe_flow_rate(pipe, loom)` — calculates throughput based on source type and tier
- `upgrade_pipe(loom, idx)` / `demolish_pipe(loom, idx)` — modify existing pipes

Split ratios control how output distributes across multiple outgoing pipes from the same source.

## Production Chain Flow

```
Extractor (base production) → Pipe → Refinery (recipe processing) → Pipe → Refinery/Pattern
```

1. Extractors produce base resources automatically based on level
2. Pipes move resources from source buffers to destination buffers
3. Refineries consume input resources and produce output resources
4. Woven Patterns consume specific resources to sustain and complete

## UI (in `src/ui/loom_scene.rs`)

Two views controlled by `LoomView` enum:
- **FlowView**: 2D grid with extractors in top 3x2 grid, refineries in scrollable processing area below (2 columns). Shows pipes as connection labels with port indicators.
- **ListDetail**: Vertical list of all nodes with detail panel.

Refinery boxes show: recipe name, tier badge, construction progress, buffer levels, stall indicator.

## Input (in `src/input/loom_input.rs`)

FlowView navigation: arrow keys move a 2D cursor across extractors (indices 0-5) and refineries (indices 6+). Key bindings:
- `B` — build refinery (opens recipe selection)
- `D` — demolish selected refinery
- `P` — build/manage pipes
- `Enter` — select/interact with node

## Debug Menu Actions

Three debug actions in `utils/debug_menu.rs`:
- `LoomBuildTestRefineryT1` — instantly build a T1 test refinery
- `LoomBuildTestRefineryT2` — instantly build a T2 test refinery
- `LoomClearRefineries` — remove all refineries and their pipes

## Integration Points

- **Ticked by**: `src/core/tick_stages.rs` `tick_loom()` — calls base production, pipe flow, refinery construction, refinery reactions, stall detection, pattern sustain
- **Input from**: `src/input/loom_input.rs` dispatches keyboard events
- **Rendered by**: `src/ui/loom_scene.rs` renders both views
- **Persisted to**: `~/.quest/loom.json` via `persistence.rs`
- **Discovery**: Triggered by pattern completion milestones
