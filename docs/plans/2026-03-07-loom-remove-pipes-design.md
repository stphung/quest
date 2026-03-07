# Loom Simplification: Remove Pipes, Direct-Pull Refineries

## Problem

The Loom has too many concepts for players to learn: Extractors, Pipes, Pipe Tiers, Split Ratios, Refineries, Refinery Tiers, Recipes, Resources, Natures, Buffers, Stalling, Patterns. Pipes are the biggest confusion source — they overlap conceptually with refineries (both route resources) and require manual wiring, tier management, and split ratio tuning.

## Solution

Remove pipes entirely. Refineries declare their input sources at build time and pull resources directly. The "factory builder" feel is preserved — players still design network topology by choosing which sources feed which refineries — but without an intermediate pipe object.

## Core Model

### Extractors (unchanged)

Six fixed nodes that produce base resources at rates scaling with level. Extractors no longer process reactions — they only produce their native resource.

### Refineries (revised)

Recipe-locked processing nodes. Each refinery declares its input sources and pulls resources directly from them.

**Key change:** Each input can have **multiple sources** (merge belt pattern). A refinery producing ForgedLight from Ember+Void can pull Ember from two different extractors, or pull ForgedLight from multiple T1 refineries.

```rust
pub struct Refinery {
    pub recipe_index: usize,
    pub input_a: Resource,
    pub input_b: Resource,
    pub output: Resource,
    pub amount: f64,           // recipe conversion multiplier
    pub tier: u8,
    pub sources_a: Vec<LoomNodeRef>,  // multiple sources for input A
    pub sources_b: Vec<LoomNodeRef>,  // multiple sources for input B
    pub buffer: f64,
    pub buffer_capacity: f64,
    pub stalled: bool,
    pub under_construction: bool,
    pub construction_ticks: u32,
}
```

### Source Restrictions

- **T1 refinery**: can pull from **extractors only**
- **T2 refinery**: can pull from **extractors + T1 refineries**
- **T3 refinery**: can pull from **extractors + T1 + T2 refineries**

No same-tier or upward pulling (no T1->T1, T2->T2, T3->T2, etc.).

### Intake Rate by Tier

Each refinery has a max intake rate per input, determined by tier:

- **T1**: 2.0/hr per input
- **T2**: 3.0/hr per input
- **T3**: 4.0/hr per input

Higher-tier refineries can pull harder from their sources, requiring fewer refinery slots for the same throughput.

### Contention

When multiple refineries pull from the same source, they **split** the available output evenly. This is the core optimization puzzle.

Example — Ember Spindle produces 4.0/hr, three T1 refineries pull from it:
- Each gets 4.0 / 3 = 1.33/hr
- Each T1 has a 2.0/hr cap, so 1.33 is the binding constraint
- Player must upgrade the extractor or restructure to fix the bottleneck

### Throughput Calculation

```
actual_pull = min(tier_intake_cap, source_available / num_consumers_of_source)
refinery_output = min(total_pull_a, total_pull_b) * recipe_amount
```

Where `total_pull_a` sums across all sources in `sources_a`, each contributing their share after contention.

### No Tier Throughput Multiplier

Tiers do NOT multiply output. A T3 running a T1 recipe produces the same output-per-input as a T1. Tiers matter because of:
1. **Source restrictions** — only T2+ can consume refined outputs
2. **Recipe exclusivity** — T2/T3 recipes only run on their tier
3. **Higher intake cap** — fewer slots needed for same throughput
4. **Slot scarcity** — max refineries = completed patterns

## Example Flows

### Simple T1

```
Ember Spindle (+4.0/hr) --> T1-A (Ember+Void -> ForgedLight, 1.0x)
Void Condenser (+3.0/hr) -->     pulls min(2.0 cap, 4.0 avail) = 2.0 Ember
                                  pulls min(2.0 cap, 3.0 avail) = 2.0 Void
                                  output: min(2.0, 2.0) * 1.0 = 2.0/hr ForgedLight
```

### Contention — Three T1s Sharing Ember

```
Ember Spindle (+4.0/hr) split 3 ways = 1.33 each

T1-A: gets 1.33 Ember -> output 1.33/hr ForgedLight
T1-B: gets 1.33 Ember -> output 1.33/hr CondensedEmber
T1-C: gets 1.33 Ember -> output 1.33/hr EmberEcho

Fix: upgrade Ember Spindle to +6.0/hr (2.0 each) or remove a T1.
```

### Scaling — Need 4.0/hr ForgedLight for a T3

```
Ember Spindle (+4.0/hr) split 2 = 2.0 each
Void Condenser (+3.0/hr) split 2 = 1.5 each  <-- bottleneck!

T1-A: min(2.0 cap, 2.0 Emb, 1.5 Void) = 1.5/hr ForgedLight
T1-B: min(2.0 cap, 2.0 Emb, 1.5 Void) = 1.5/hr ForgedLight

T3-A: sources_a = [T1-A, T1-B] (ForgedLight)
      pull cap = 4.0/hr
      available = 1.5 + 1.5 = 3.0/hr
      gets: min(4.0 cap, 3.0 avail) = 3.0/hr

To saturate T3 at 4.0/hr:
  Option A: Upgrade Void Condenser so T1s each get 2.0 (2*2.0 = 4.0)
  Option B: Add third T1 (3*1.5 = 4.5, capped at 4.0)
```

### Full Pipeline — T1 -> T2 -> T3

```
Ember (+6.0) split 2 = 3.0 each (> 2.0 cap, so T1s get 2.0 each)
Void (+4.0) split 2 = 2.0 each

T1-A: 2.0/hr ForgedLight
T1-B: 2.0/hr ForgedLight

T2-A: sources_a=[T1-A, T1-B] (4.0/hr FrgLt), sources_b=[Memory Archive]
      pull cap = 3.0/hr per input
      FrgLt available: 4.0, capped at 3.0
      Memory available: 5.0, capped at 3.0
      output: min(3.0, 3.0) * 0.3 = 0.9/hr WovenReality

T3-A: sources_a=[T2-A], sources_b=[T1-C (StillbornSong)]
      pull cap = 4.0/hr per input
      WovRl available: 0.9 from T2-A
      StSng available: 2.0 from T1-C
      output: min(0.9, 2.0) * 0.5 = 0.45/hr WovenReality (higher tier recipe)
```

## Optimization Loop

The player's cycle:
1. Check what the current pattern demands (resource rates)
2. Work backwards — how many refineries at each tier?
3. Check if extractors can supply them all (contention math)
4. Choose: upgrade extractors, build more refineries, or restructure the chain

## Build Interaction

When the player presses `B`:

1. **Pick a tier** — T1, T2, T3 (only unlocked tiers shown)
2. **Pick a recipe** — filtered to that tier's recipes
3. **Pick sources** — for each input, select one or more eligible nodes
   - Eligible list filtered by tier source rules
   - Shows each source's current output rate and consumer count
4. **Confirm** — shows expected throughput, costs resources, starts construction

Sources can be edited after building without demolishing.

## What Gets Removed

- `Pipe` struct and all fields
- `PipeTier` enum
- `pipes.rs` entirely (~600 LOC + tests)
- Split ratio system
- Pipe construction, upgrading, demolishing
- Pipe flow simulation (replaced by direct-pull tick)
- Port label rendering in Flow View
- `[P]ipe` hotkey and all pipe input handling
- `LoomNodeRef` stays (used for refinery source addressing)

## What Changes

- `Refinery` struct: `source_a`/`source_b` become `sources_a: Vec<LoomNodeRef>` / `sources_b: Vec<LoomNodeRef>`, add `tier_intake_cap` derived from tier
- Extractor `LoomNode`: remove reaction processing, recipe slots — extractors only produce base resources
- Tick: new `tick_refinery_pull()` replaces `tick_pipe_flow()` — iterates refineries, calculates contention, pulls from sources
- Flow View: connection arrows derived from refinery sources instead of pipe list
- Sidebar: shows contention info per extractor ("3 consumers, 1.33/hr each")
- Build UI: new multi-step builder (tier -> recipe -> sources -> confirm)

## What Stays

- `LoomNodeRef` enum (Extractor/Refinery addressing)
- Refinery tiers, pattern gating, slot limits
- Recipe system (unchanged)
- Extractor levels and production rates
- Buffer system and stall detection
- Woven Pattern sustain and completion
- Construction delays

## Visual Design

### Throbber System

Braille spinner characters (⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏) animate at tier-proportional speeds:
- **T1**: 500ms per frame (slow, steady)
- **T2**: 300ms per frame (moderate)
- **T3**: 150ms per frame (fast, intense)
- **Stalled**: Frozen spinner + `[■]` badge
- **Starved**: Stuttering animation (skips frames)

### Node Rendering

Compact row format for refineries in the processing area below extractors:

```
⠹ T1 ForgedLight    Emb←[ES] Voi←[VC]  2.0/hr  ████░░
```

Format: `[throbber] [tier] [output] [source badges] [rate] [buffer bar]`

### Bottleneck Indicators

- `[!!]` — Root bottleneck (source can't keep up with demand)
- `[↓]` — Downstream symptom (starved because upstream is bottlenecked)
- `[■]` — Stalled (output buffer full, no consumers)

### Extractors

Top 3×2 grid with animated node boxes (existing style). Each shows:
- Name, level, native resource
- Production rate
- Consumer count and contention status (e.g., "3 consumers, 1.33/hr each")

### Sidebar Detail Panel

Selected node shows full detail:
- All sources with individual pull rates
- Contention breakdown per source
- Buffer levels and capacity
- Expected vs actual throughput
- Bottleneck diagnosis

### Pattern Info

Three-layer hierarchy:
1. **Compact bar** (always visible) — pattern name, progress bar, time remaining
2. **Detail panel** (on select) — resource requirements with met/unmet status
3. **Progression overview** (dedicated view) — all 18 patterns with completion state
