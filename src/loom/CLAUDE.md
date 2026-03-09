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
└── persistence.rs  — Save/load from ~/.quest/loom.json
```

## Key Types

| Type | Purpose |
|------|---------|
| `NodeId` | Enum of 6 fixed extractor identities (EmberSpindle, VoidCondenser, etc.) |
| `LoomNodeRef` | Unified addressing: `Extractor(NodeId)` or `Shuttle(usize)` — used in shuttle source fields |
| `LoomNode` | Extractor state: level, buffers per resource, nature, stall flag, unlocked status |
| `Shuttle` | Recipe-locked processing node: input_a/b, nature, output, amount, tier, buffer, sources_a/sources_b, construction state |
| `Resource` | Enum of all resources: 6 base + confluence + reaction products |
| `WovenPattern` | Pattern with resource requirements (sustained rate thresholds and durations), completion state |
| `RateTracker` | 60-second rolling window rate measurement (transient, not serialized) |
| `LoomState` | Top-level state: `persistent` (saved) + `ui_state` (transient) |
| `LoomPersistent` | Saved state: nodes, shuttles, stockpile, codex, patterns |

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

**Intake caps** (`tier_intake_cap(tier)` in `logic.rs`):
| Tier | Max intake per input slot (units/hour) |
|------|---------------------------------------|
| T1   | 20.0 |
| T2   | 30.0 |
| T3   | 40.0 |

The actual pull per source is `min(intake_cap, share)`, summed across all sources for that input slot.

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

**Shuttle limit**: Max shuttles = number of completed Woven Patterns (max 28).

**Construction**: Shuttles have a 50-tick construction period. `tick_shuttle_construction()` decrements timers and marks them ready.

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
- Requirements complete independently — the player doesn't need to sustain all resources simultaneously
- Rate measurement uses a 60-second rolling window (`RateTracker` struct, transient, not serialized)
- Simple pause model: progress never decays, only pauses when rate drops below threshold

## Production Chain Flow

```
Extractor (base production) → [direct pull] → Shuttle (recipe processing) → [direct pull] → Higher-tier Shuttle / Pattern
```

1. Extractors produce base resources automatically based on level
2. Shuttles pull from declared sources each tick (direct-pull, no pipe objects)
3. Shuttles consume input resources and produce output resources into their buffer
4. `tick_pattern_sustain()` draws from stockpiles and shuttle output to advance pattern requirements

## UI (in `src/ui/loom_scene.rs`)

Two views controlled by `LoomView` enum:
- **FlowView**: 2D grid with extractors in top 3x2 grid, shuttles in scrollable processing area below (2 columns). Shows source connections in detail panel.
- **Codex**: Recipe codex showing discovered and undiscovered recipes.

Shuttle boxes show: recipe name, tier badge, construction progress, buffer levels, stall indicator.

## Input (in `src/input/loom_input.rs`)

FlowView navigation: arrow keys move a 2D cursor across extractors (indices 0-5) and shuttles (indices 6+). Key bindings:
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

## WR→PR Generation

When all 28 Woven Patterns are complete, the Loom converts Weave Rate (WR) into Prestige Ranks (PR) per day using tiered brackets:

| WR/hr bracket | PR per WR/hr per day |
|---------------|---------------------|
| 0–10 | 5 |
| 10–25 | 10 |
| 25+ | 15 |

- `wr_to_pr_per_day(wr_per_hour) -> f64` — calculates daily PR output from current weave rate
- Activation condition: `completed_pattern_count() >= 28`

## Key Functions (Power Integration)

- `completed_pattern_count() -> usize` — count of fully completed Woven Patterns
- `loom_zone_cap_for_ascension(patterns) -> u32` — returns max zone ID unlocked by pattern count (Z31-50)
- `wr_to_pr_per_day(wr_per_hour) -> f64` — tiered WR to PR/day conversion
- `upgrade_shuttle(loom, idx)` — upgrade shuttle level, increasing effective intake cap
- `shuttle_effective_intake_cap(tier, level) -> f64` — intake cap adjusted for shuttle level

## Integration Points

- **Ticked by**: `src/core/tick_stages.rs` `tick_loom()` — calls base production, `tick_shuttle_pull`, shuttle construction, stall detection, pattern sustain
- **Input from**: `src/input/loom_input.rs` dispatches keyboard events
- **Rendered by**: `src/ui/loom_scene.rs` renders FlowView and Codex views
- **Persisted to**: `~/.quest/loom.json` via `persistence.rs`
- **Discovery**: Triggered by pattern completion milestones
- **Ascension** (`ascension/types.rs`): `ascension_pattern_gate()` checks pattern count for VII-X eligibility; `max_shuttle_level()` gates shuttle upgrades by Ascension tier
- **Zones** (`zones/`): `loom_zone_cap_for_ascension()` unlocks Loom Zones 31-50 based on completed pattern count
