# Loom of Worlds

A mid-to-late game resource production and crafting system. Players manage 6 fixed Extractor nodes and buildable Shuttle nodes, connecting them via direct-pull sources to create Factorio-style production chains. Resources flow through the network, combining via recipes to produce higher-tier materials needed to sustain Woven Patterns.

## Module Structure

```
src/loom/
├── mod.rs          — Public API re-exports
├── types.rs        — All data structures: NodeId, LoomNodeRef, Resource, Shuttle, LoomState, etc.
├── logic.rs        — Node upgrades, base production, stall detection, reactions, shuttle building/demolishing, direct-pull tick
├── recipes.rs      — Recipe registry, lookup_recipe(a, b, nature), recipes_by_nature()
├── patterns.rs     — Woven Pattern sustain timer and completion tracking
├── discovery.rs    — 28 woven patterns defined in create_pattern_sequence()
├── milestones.rs   — Pattern milestone types and helpers for key pattern completion modals
├── graph.rs        — petgraph DAG construction, rebuild logic, rate updates
├── layout.rs       — Sugiyama layout engine: layer assignment, crossing minimization, coordinate assignment
└── persistence.rs  — Save/load from ~/.quest/loom.json
```

## Key Types

| Type | Purpose |
|------|---------|
| `NodeId` | Enum of 6 fixed extractor identities (EmberSpindle, VoidCondenser, etc.) |
| `LoomNodeRef` | Unified addressing: `Extractor(NodeId)` or `Shuttle(usize)` — used in shuttle source fields |
| `LoomNode` | Extractor state: level, buffers per resource, nature, stall flag, unlocked status |
| `Shuttle` | Recipe-locked processing node: input_a/b, nature, output, amount, tier, buffer, sources_a/sources_b, construction state; `output_rate_tracker: RateTracker` (transient, not serialized) |
| `Resource` | Enum of all resources: 6 base + confluence + reaction products |
| `WovenPattern` | Pattern with resource requirements (sustained rate thresholds and durations), completion state |
| `RateTracker` | 20-second rolling window rate measurement (200 ticks at 100ms/tick, transient, not serialized) |
| `LoomState` | Top-level state: `persistent` (saved) + `ui_state` (transient) |
| `LoomPersistent` | Saved state: nodes, shuttles, stockpile, codex, patterns |
| `LoomGraphNode` | Graph node enum: `Extractor(NodeId)`, `Shuttle(usize)`, `PatternSink(usize)` |
| `LoomEdge` | Edge weight: resource type, current_rate, max_rate |
| `LoomGraph` | `StableDiGraph` wrapper with reverse lookup maps |
| `LoomLayout` | Computed node positions (x, y) and dummy node paths for long edges |

## Node Addressing with LoomNodeRef

All shuttle source references use `LoomNodeRef` instead of raw `NodeId`:

```rust
pub enum LoomNodeRef {
    Extractor(NodeId),  // Fixed nodes, identified by NodeId enum
    Shuttle(usize),    // Dynamic nodes, identified by index in shuttles vec
}
```

Shuttle structs carry `sources_a: Vec<LoomNodeRef>` and `sources_b: Vec<LoomNodeRef>` — the set of nodes from which each input slot pulls each tick.

## Direct-Pull System

Instead of discrete pipe objects, shuttles pull directly from their declared sources each tick. Key properties:

**Contention model**: If multiple shuttles share a source, the source's available buffer is split equally among consumers. Each consumer gets `share = source_buffer / num_consumers`.

**No intake cap**: There is no per-tier intake cap on shuttle throughput. Shuttle throughput is limited only by the source node's output rate and contention splitting among consumers. `tier_intake_cap()` exists in `logic.rs` but is used for display purposes only and does not gate actual production.

**Source restrictions** (`valid_source_for_tier()` in `logic.rs`):
- Extractors are always valid sources for any tier.
- Shuttles are valid sources only if their tier is strictly less than the consuming shuttle's tier.
  - T1 shuttles: pull only from Extractors
  - T2 shuttles: pull from Extractors or T1 shuttles
  - T3 shuttles: pull from Extractors, T1, or T2 shuttles

**Processing order**: Shuttles are processed T1 first, then T2, then T3, so lower-tier output is available for higher-tier consumption within the same tick.

**Output rate**: `output_rate = min(total_pull_a, total_pull_b) * recipe_amount`

## Shuttle System

Shuttles are recipe-locked processing nodes that create multi-step production chains.

**Building**: `build_shuttle(loom, recipe_index, sources_a, sources_b)` validates tier unlock, source restrictions, resource cost, and shuttle limit, then pushes a new `Shuttle` to `loom.persistent.shuttles`.

**Tiers and gating**:
| Tier | Pattern gate | Resource cost |
|------|-------------|---------------|
| T1 | 1 pattern complete | 250 of input_a resource |
| T2 | 8 patterns complete | 150 of input_a resource |
| T3 | 15 patterns complete | 100 of input_a resource |

**Shuttle limit**: Governed by a milestone curve (`MAX_SHUTTLES = 5`). Capacity unlocks at specific pattern completion milestones:
| Patterns completed | Shuttle slots |
|--------------------|---------------|
| 1 | 1 |
| 4 | 2 |
| 8 | 3 |
| 12 | 4 |
| 15+ | 5 |

**Construction**: Shuttles have a tier-based construction period. `tick_shuttle_construction()` decrements timers and marks them ready:
| Tier | Construction time |
|------|-------------------|
| T1 | 2h (7,200 seconds) |
| T2 | 4h (14,400 seconds) |
| T3 | 6h (21,600 seconds) |

**Processing**: `tick_shuttle_pull(loom, delta_seconds)` runs the direct-pull simulation each tick, returning a map of `Resource → amount produced` for pattern tracking.

**Demolishing**: `demolish_shuttle(loom, idx)` removes the shuttle and re-indexes any `Shuttle(usize)` references in remaining shuttles' source lists.

**Stall detection**: `tick_shuttle_stall_detection()` marks shuttles as stalled when output buffers are full or inputs cannot be sourced.

## Pattern System

Woven Patterns use sustained production rates rather than accumulated totals:

- `PatternRequirement::required_rate` — minimum production rate (units/hr) that must be sustained
- `PatternRequirement::sustain_duration_secs` — total seconds the rate must be sustained
- `PatternRequirement::sustained_secs` — seconds sustained so far (advances when rate >= threshold, pauses when below)
- `PatternRequirement::completed` — whether this individual requirement is complete (locks independently)
- Pattern completes when all requirements have `completed = true`
- Requirements must be met **simultaneously**: ALL resource rates must be at or above their thresholds at the same time for any timers to advance. If even one resource drops below its threshold, no requirement timers advance — the entire pattern is paused until all rates recover.
- Rate measurement uses a 20-second rolling window (`RateTracker` struct, 200 ticks at 100ms/tick, transient, not serialized)
- Simple pause model: progress never decays, only pauses when any rate drops below threshold

## Production Chain Flow

```
Extractor (base production) → [direct pull] → Shuttle (recipe processing) → [direct pull] → Higher-tier Shuttle / Pattern
```

1. Extractors produce base resources automatically based on level
2. Shuttles pull from declared sources each tick (direct-pull, no pipe objects)
3. Shuttles consume input resources and produce output resources into their buffer
4. `tick_pattern_sustain()` draws from stockpiles and shuttle output to advance pattern requirements

## Base Production Rate

Extractors produce at a base rate of **25/hr** (at level 1). Higher levels multiply this base rate via `node_level_multiplier(level)`.

## Recipes

The Loom has **7 exclusive recipes** split across tiers with no overlap — each output resource belongs to exactly one tier:

| Tier | Count | Notes |
|------|-------|-------|
| T1 | 3 recipes | Combine base extractor resources into T1 outputs |
| T2 | 3 recipes | Combine T1 (and/or base) resources into T2 outputs |
| T3 | 1 recipe | Produces the single T3 output resource |

Each recipe is identified by `(input_a, input_b, nature)` and looked up via `lookup_recipe(a, b, nature)` in `recipes.rs`. `recipes_by_nature()` returns all recipes filtered by nature type.

## UI (in `src/ui/loom_scene.rs` and `src/ui/loom_graph.rs`)

Single graph view (the Codex was removed; FlowView was replaced by GraphView):
- **GraphView**: Canvas-based DAG visualization using petgraph. Nodes arranged in layers — extractors (layer 0), T1/T2/T3 shuttles (layers 1–3), pattern sinks (layer 4). Animated edges with particles and glow propagation reflecting live flow rates. Bottom panel shows node detail, build flow, or pattern info depending on selection. Layout computed by `layout.rs` (Sugiyama algorithm).

Shuttle nodes show: recipe name, tier badge, construction progress, buffer levels, stall indicator.

## Input (in `src/input/loom_input.rs`)

GraphView navigation: arrow keys follow graph topology — moving to adjacent nodes along edges rather than a fixed 2D grid. Key bindings:
- `B` — build shuttle (opens recipe selection)
- `D` — demolish selected shuttle
- `Enter` — select/interact with node

## Debug Menu Actions

Three debug actions in `utils/debug_menu.rs`:
- `LoomBuildTestShuttleT1` — instantly build a T1 test shuttle
- `LoomBuildTestShuttleT2` — instantly build a T2 test shuttle
- `LoomClearShuttles` — remove all shuttles

## Shuttle Upgrades

Shuttles can be upgraded to increase their intake cap. Each level adds 0.5x to the base multiplier:

- `node_level_multiplier(level)` = `1.0 + (level - 1) * 0.5` (level 1 = 1.0x, level 2 = 1.5x, level 3 = 2.0x, etc.)
- `shuttle_effective_intake_cap(tier, level)` = `tier_intake_cap(tier) * node_level_multiplier(level)`
- `upgrade_shuttle(loom, idx)` — upgrade a shuttle's level (costs resources)

**Max shuttle level** is gated by Ascension tier via `max_shuttle_level(ascension_level)`:

| Ascension Level | Max Shuttle Level |
|-----------------|-------------------|
| 0–VI | 1 |
| VII | 3 |
| VIII | 5 |
| IX | 7 |
| X | 10 |

## Extractor Upgrade Lockout

When an Extractor is upgraded, it enters a lockout period:

- **Buffer drain**: 50% of the extractor's buffer *capacity* is consumed on upgrade start; the upgrade is blocked unless the current buffer holds at least that amount.
- **Lockout duration**: `level * 2h` (e.g., upgrading to level 2 = 2h lockout, level 3 = 4h, etc.)
- **Zero production**: The extractor produces nothing during the lockout period.
- **Rate tracker cleared**: The `RateTracker` for that extractor is reset when the upgrade begins, so rolling-window rates reflect only post-upgrade production.

## Wall-Clock Time Model

The Loom uses **wall-clock time** (`chrono::Utc::now()`) rather than tick-based time for timers (construction, upgrade lockout, pattern sustain durations). Key implications:

- Chrono Surge (the game's tick-acceleration mechanic) does **not** accelerate Loom timers — the Loom is skipped during surge processing.
- The debug `time_warp` tool does advance Loom wall-clock timers and can be used to fast-forward construction and lockout periods during development.

## WR→PR Generation

When all 28 Woven Patterns are complete, the Loom converts Weave Rate (WR) into Prestige Ranks (PR) per hour using a self-multiplying formula:

`PR/hr = WR × (1 + WR/100)`

Starts ~1:1 at low rates, scales superlinearly as WR increases:
- 10 WR/hr → 11 PR/hr (×1.1)
- 50 WR/hr → 75 PR/hr (×1.5) — typical at pattern 28 completion
- 131 WR/hr → 303 PR/hr (×2.3) — max extractors at L20

- `wr_to_pr_per_hour(wr_per_hour) -> u32` — calculates PR/hr from current weave rate
- `wr_pr_multiplier(wr_per_hour) -> f64` — returns the multiplier (1 + WR/100) for UI display
- Activation condition: `completed_pattern_count() >= 28`

## Key Functions (Power Integration)

- `completed_pattern_count() -> usize` — count of fully completed Woven Patterns
- `loom_zone_cap_for_patterns(completed_patterns) -> u32` — returns max zone ID unlocked by pattern count (Z31-50)
- `wr_to_pr_per_hour(wr_per_hour) -> u32` — self-multiplying WR to PR/hr conversion
- `upgrade_shuttle(loom, idx)` — upgrade shuttle level, increasing effective intake cap
- `shuttle_effective_intake_cap(tier, level) -> f64` — intake cap adjusted for shuttle level

## Integration Points

- **Ticked by**: `src/core/tick_stages.rs` `tick_loom()` — calls base production, `tick_shuttle_pull`, shuttle construction, stall detection, pattern sustain
- **Input from**: `src/input/loom_input.rs` dispatches keyboard events
- **Rendered by**: `src/ui/loom_scene.rs` + `src/ui/loom_graph.rs` render GraphView
- **Persisted to**: `~/.quest/loom.json` via `persistence.rs`
- **Discovery**: Triggered by pattern completion milestones
- **Ascension** (`ascension/types.rs`): `ascension_pattern_gate()` checks pattern count for VII-X eligibility; `max_shuttle_level()` gates shuttle upgrades by Ascension tier
- **Zones** (`zones/`): `loom_zone_cap_for_patterns()` unlocks Loom Zones 31-50 based on completed pattern count
- **petgraph 0.7**: DAG construction and traversal in `graph.rs`; layout computed in `layout.rs`
