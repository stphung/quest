# Loom of Worlds

A mid-to-late game resource production and crafting system. Players manage 6 fixed Extractor nodes and buildable Refinery nodes, connecting them via direct-pull sources to create Factorio-style production chains. Resources flow through the network, combining via recipes to produce higher-tier materials needed to sustain Woven Patterns.

## Module Structure

```
src/loom/
├── mod.rs          — Public API re-exports
├── types.rs        — All data structures: NodeId, LoomNodeRef, Resource, Refinery, LoomState, etc.
├── logic.rs        — Node upgrades, base production, stall detection, reactions, refinery building/demolishing, direct-pull tick
├── recipes.rs      — Recipe registry, lookup_recipe(a, b, nature), recipes_by_nature()
├── patterns.rs     — Woven Pattern sustain timer and completion tracking
├── discovery.rs    — 28 woven patterns defined in create_pattern_sequence()
└── persistence.rs  — Save/load from ~/.quest/loom.json
```

## Key Types

| Type | Purpose |
|------|---------|
| `NodeId` | Enum of 6 fixed extractor identities (EmberSpindle, VoidCondenser, etc.) |
| `LoomNodeRef` | Unified addressing: `Extractor(NodeId)` or `Refinery(usize)` — used in refinery source fields |
| `LoomNode` | Extractor state: level, buffers per resource, nature, stall flag, unlocked status |
| `Refinery` | Recipe-locked processing node: input_a/b, nature, output, amount, tier, buffer, sources_a/sources_b, construction state |
| `Resource` | Enum of all resources: 6 base + confluence + reaction products |
| `WovenPattern` | Pattern with resource requirements (sustained rate thresholds and durations), completion state |
| `RateTracker` | 60-second rolling window rate measurement (transient, not serialized) |
| `LoomState` | Top-level state: `persistent` (saved) + `ui_state` (transient) |
| `LoomPersistent` | Saved state: nodes, refineries, stockpile, codex, patterns |

## Node Addressing with LoomNodeRef

All refinery source references use `LoomNodeRef` instead of raw `NodeId`:

```rust
pub enum LoomNodeRef {
    Extractor(NodeId),  // Fixed nodes, identified by NodeId enum
    Refinery(usize),    // Dynamic nodes, identified by index in refineries vec
}
```

Refinery structs carry `sources_a: Vec<LoomNodeRef>` and `sources_b: Vec<LoomNodeRef>` — the set of nodes from which each input slot pulls each tick.

## Direct-Pull System

Instead of discrete pipe objects, refineries pull directly from their declared sources each tick. Key properties:

**Contention model**: If multiple refineries share a source, the source's available buffer is split equally among consumers. Each consumer gets `share = source_buffer / num_consumers`.

**Intake caps** (`tier_intake_cap(tier)` in `logic.rs`):
| Tier | Max intake per input slot (units/hour) |
|------|---------------------------------------|
| T1   | 2.0 |
| T2   | 3.0 |
| T3   | 4.0 |

The actual pull per source is `min(intake_cap, share)`, summed across all sources for that input slot.

**Source restrictions** (`valid_source_for_tier()` in `logic.rs`):
- Extractors are always valid sources for any tier.
- Refineries are valid sources only if their tier is strictly less than the consuming refinery's tier.
  - T1 refineries: pull only from Extractors
  - T2 refineries: pull from Extractors or T1 refineries
  - T3 refineries: pull from Extractors, T1, or T2 refineries

**Processing order**: Refineries are processed T1 first, then T2, then T3, so lower-tier output is available for higher-tier consumption within the same tick.

**Output rate**: `output_rate = min(total_pull_a, total_pull_b) * recipe_amount`

## Refinery System

Refineries are recipe-locked processing nodes that create multi-step production chains.

**Building**: `build_refinery(loom, recipe_index, sources_a, sources_b)` validates tier unlock, source restrictions, resource cost, and refinery limit, then pushes a new `Refinery` to `loom.persistent.refineries`.

**Tiers and gating**:
| Tier | Pattern gate | Resource cost |
|------|-------------|---------------|
| T1 | 1 pattern complete | 25 of input_a resource |
| T2 | 8 patterns complete | 15 of input_a resource |
| T3 | 15 patterns complete | 10 of input_a resource |

**Refinery limit**: Max refineries = number of completed Woven Patterns (max 28).

**Construction**: Refineries have a 50-tick construction period. `tick_refinery_construction()` decrements timers and marks them ready.

**Processing**: `tick_refinery_pull(loom, delta_seconds)` runs the direct-pull simulation each tick, returning a map of `Resource → amount produced` for pattern tracking.

**Demolishing**: `demolish_refinery(loom, idx)` removes the refinery and re-indexes any `Refinery(usize)` references in remaining refineries' source lists.

**Stall detection**: `tick_refinery_stall_detection()` marks refineries as stalled when output buffers are full or inputs cannot be sourced.

## Pattern System

Woven Patterns use sustained production rates rather than accumulated totals:

- `PatternRequirement::required_rate` — minimum production rate (units/hr) that must be sustained
- `PatternRequirement::sustain_duration_secs` — total seconds the rate must be sustained
- `PatternRequirement::sustained_secs` — seconds sustained so far (advances when rate >= threshold, pauses when below)
- `PatternRequirement::completed` — whether this individual requirement is complete (locks independently)
- Pattern completes when all requirements have `completed = true`
- Requirements complete independently — the player doesn't need to sustain all resources simultaneously
- Rate measurement uses a 60-second rolling window (`RateTracker` struct, transient, not serialized)
- Simple pause model: progress never decays, only pauses when rate drops below threshold

## Production Chain Flow

```
Extractor (base production) → [direct pull] → Refinery (recipe processing) → [direct pull] → Higher-tier Refinery / Pattern
```

1. Extractors produce base resources automatically based on level
2. Refineries pull from declared sources each tick (direct-pull, no pipe objects)
3. Refineries consume input resources and produce output resources into their buffer
4. `tick_pattern_sustain()` draws from stockpiles and refinery output to advance pattern requirements

## UI (in `src/ui/loom_scene.rs`)

Two views controlled by `LoomView` enum:
- **FlowView**: 2D grid with extractors in top 3x2 grid, refineries in scrollable processing area below (2 columns). Shows source connections in detail panel.
- **ListDetail**: Vertical list of all nodes with detail panel.

Refinery boxes show: recipe name, tier badge, construction progress, buffer levels, stall indicator.

## Input (in `src/input/loom_input.rs`)

FlowView navigation: arrow keys move a 2D cursor across extractors (indices 0-5) and refineries (indices 6+). Key bindings:
- `B` — build refinery (opens recipe selection)
- `D` — demolish selected refinery
- `Enter` — select/interact with node

## Debug Menu Actions

Three debug actions in `utils/debug_menu.rs`:
- `LoomBuildTestRefineryT1` — instantly build a T1 test refinery
- `LoomBuildTestRefineryT2` — instantly build a T2 test refinery
- `LoomClearRefineries` — remove all refineries

## Integration Points

- **Ticked by**: `src/core/tick_stages.rs` `tick_loom()` — calls base production, `tick_refinery_pull`, refinery construction, stall detection, pattern sustain
- **Input from**: `src/input/loom_input.rs` dispatches keyboard events
- **Rendered by**: `src/ui/loom_scene.rs` renders both views
- **Persisted to**: `~/.quest/loom.json` via `persistence.rs`
- **Discovery**: Triggered by pattern completion milestones
