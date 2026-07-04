> Backported design record. Sources: docs/plans/2026-03-06-loom-of-worlds-design.md, docs/plans/2026-03-06-loom-pipeline-recipe-system-design.md, docs/plans/2026-03-06-loom-starting-archetypes-design.md, docs/plans/2026-03-07-loom-flow-view-redesign.md, docs/plans/2026-03-07-loom-power-integration-design.md, docs/plans/2026-03-07-loom-remove-pipes-design.md, docs/plans/2026-03-07-loom-sustained-patterns-design.md, docs/superpowers/plans/2026-04-04-loom-graph-view.md, docs/superpowers/specs/2026-04-04-loom-graph-view-design.md.

## 2026-03-06-loom-of-worlds-design.md

# The Loom of Worlds — Post-Zone 30 Factory Endgame

**Date:** 2026-03-06
**Status:** Approved

## Premise

Beyond the Deep's Gateway lies the Loom — the broken mechanism that weaves reality. The six fracture chapters (Red Fault through Origin Wound) were symptoms of the Loom failing. The prestige cycle was unknowingly generating the energy the Loom needs. Now the player operates it deliberately.

The game transitions from combat RPG to incremental factory/engine-builder. The combat game and Loom coexist initially, with the factory gradually becoming the primary game as it outscales combat.

## Entry

Requires completing the **Gateway Expedition** at Deep Layer 30. This mission is extended to **168 hours (7 days)** with a fixed duration that **bypasses all duration modifiers** (infrastructure, familiarity, saboteur, overpower). Only wall-clock time and Chrono Surge acceleration apply.

On completion, the Loom overlay unlocks permanently.

## First Interaction

The Loom presents six broken nodes corresponding to the six fracture chapters. The player chooses which to repair first, determining their starting branch and early strategy. Adjacent nodes unlock as the first resource accumulates.

## Resource Cycle

### Tier 1: Base Loop

Six resources in a closed cycle, each tied to a fracture chapter:

```
Ember (Red Fault) --> Reflection (Mirror Scar) --> Void Essence (Black Mouth)
  ^                                                         |
  |                                                         v
Resonance (Origin Wound) <-- Silence (Wailing Reach) <-- Memory (Hollow Throne)
```

1. **Ember** — Raw energy, heat of creation. Produced from prestige fuel.
2. **Reflection** — Structured energy, given form. Refined from Ember.
3. **Void Essence** — Consuming force that strips impurities. Distilled from Reflection.
4. **Memory** — Blueprint of what was. Crystallized from Void Essence.
5. **Silence** — Space between things. Condensed from Memory.
6. **Resonance** — Vibration of stable reality. Synthesized from Silence. Feeds back into Ember production as a multiplier, closing the loop.

Each node has a building (Spindle, Lens, Condenser, Archive, Well, Forge) with a level, conversion ratio, and throughput. Upgrading improves ratio or throughput. **Prestige/hour** is derived from Resonance output.

### Tier 2: Confluence Nodes

Three buildings that combine two non-adjacent base resources:

- **Forged Light** = Ember + Void Essence (fire tempered by void)
- **Echo Glass** = Reflection + Memory (form given memory)
- **Stillborn Song** = Silence + Resonance (space that vibrates)

Confluences are **required to upgrade base nodes past Level 10**. This forces the player to divert throughput from the base cycle into upgrade materials — consuming output to unlock higher output.

### Tier 3: The Tapestry

**The Tapestry Locus** requires all three confluence resources as input. Produces **Woven Reality** — a global prestige/hour multiplier. Infinite tiers with diminishing-but-never-zero returns.

```
Forged Light ---\
Echo Glass ------+--> Tapestry Locus --> Woven Reality (prestige/hr multiplier)
Stillborn Song -/
```

## Existing System Integration

Deep, Haven, Stormglass, and Ascension provide small ongoing production bonuses that are meaningful when the Loom is new (e.g., +1 Ember/hr when your Spindle produces 2/hr) but become negligible as the Loom scales (irrelevant at 200/hr). This acknowledges the player's journey without distorting the Loom's own progression curve.

The Loom does not feed back into existing systems — yet. The architecture leaves this door open for a future narrative beat.

## Transition Model

The combat game and Loom coexist. Early Loom stages need prestige fuel from the combat loop. As the Loom grows, it generates its own fuel autonomously. The crossover point — when the Loom produces more prestige/hour than combat — is a key milestone (approximately week 1-2).

Late-game, the combat loop becomes a small passive income stream while the Loom is the primary game. The player naturally migrates without a hard cutoff.

## Progression Timeline

- **Week 1:** Base loop. First node online, adjacent nodes unlocking, full cycle closing by end of week. Resonance feedback loop first activates.
- **Weeks 2-3:** Confluences. Base nodes hit Level 10 cap. Player begins producing Forged Light, Echo Glass, Stillborn Song to break through.
- **Week 4:** Tapestry. All three confluences feeding the Tapestry Locus. First Woven Reality produced.
- **Month 2+:** Infinite scaling tail. All structures continue to level with no cap. Diminishing returns but the number never stops growing. Prestige/hour becomes a high-score chase.

## TUI Design

Two views toggled by Tab, following existing Quest overlay patterns:

### Flow View

ASCII diagram showing the full resource cycle, confluence connections, and flow rates. Selected node highlights with colored input/output connections. For understanding the big picture.

```
         +-------- 12/hr --------+
         v                       |
   +==========+           +==========+
   | Ember  8 |--5/hr--> |Reflect 6 |
   +==========+           +==========+
         |                      |
         |                      |
   +==========+           +==========+
   |Resonan 1 |          | Void   4 |
   +==========+           +==========+
         ^                      |
         |               1.8/hr |
   +==========+                 v
   |Silence 2 |<-- 0.9/hr ----+
   +==========+           +==========+
         ^                |Memory  3 |
         |                +==========+
         +--- 0.4/hr ----------+
```

Confluence nodes appear visually between their two inputs when unlocked.

### List + Detail View

Left panel: scrollable node list with level, rate, stock. Right panel: selected node's input/output connections as a mini-diagram, bottleneck indicator, upgrade cost. For taking action.

```
+- Nodes ---------------+- Void Condenser [Lvl 4] ----------+
|                       |                                    |
| Ember Spindle    8    |  INPUTS                            |
| Reflection Lens  6    |    Reflection -- 8 per cycle --+   |
| >Void Condenser  4    |                                v   |
| Memory Archive   3    |              +=============+       |
| Silence Well     2    |              | Void        |       |
| Resonance Forge  1    |              | 1.8/hr      |       |
|                       |              | Stock: 43   |       |
| CONFLUENCES           |              +=============+       |
| Forged Light     0    |                    |               |
|                       |  OUTPUTS           v               |
|                       |    Memory -- 2 per cycle           |
|                       |    Forged Light -- needs 5V + 10E  |
|                       |                                    |
|                       |  BOTTLENECK: Reflection supply     |
|                       |  Upgrade: 50 Reflection            |
|                       |  [Enter] Upgrade  [Space] Details  |
+-----------------------+------------------------------------+
```

Navigation: arrow keys to select, Enter to upgrade, Tab to toggle views, Esc to close.

## Design Decisions

- **168-hour Gateway mission** with fixed duration (no infrastructure/familiarity/saboteur/overpower modifiers, only wall-clock and Chrono Surge)
- **Cyclical engine** with feedback loop (Resonance amplifies Ember)
- **6 base resources** tied to fracture chapters
- **3 confluence resources** requiring multi-input combinations
- **1 tapestry output** as the ultimate prestige/hr multiplier
- **Starting choice** of first fracture node creates early strategic variation
- **Existing systems provide diminishing ongoing bonuses** (meaningful early, negligible late)
- **One-directional integration** (existing systems feed Loom, not reverse) — door left open for future bidirectional
- **~1 month structured content** with infinite scaling tail
- **Hybrid transition** from combat to factory (no hard cutoff)

## Narrative Arc

The hero's journey recontextualized:
1. Zones 1-10: Survived the world
2. Zone 11: Crossed beyond the world
3. Zones 12-30: Traversed the wounds in reality
4. The Deep/Gateway: Found why reality is wounded
5. The Loom: Became the one who mends it

Each prestige cycle was always fuel for the Loom. Each generation stood on the shoulders of the last. Now the player understands why — they were always building toward this. The tone shifts from ontological horror (discovering the breaks) to ontological craft (repairing the mechanism). The hero doesn't conquer the final boss; they become the architect.

## 2026-03-06-loom-pipeline-recipe-system-design.md

# Loom Pipeline & Recipe System — Factory Depth Design

**Date:** 2026-03-06
**Status:** Approved
**Extends:** [Loom of Worlds Design](2026-03-06-loom-of-worlds-design.md), [Starting Archetypes](2026-03-06-loom-starting-archetypes-design.md)

## Overview

Replaces the Loom's implicit node connections and linear upgrade path with explicit pipelines, combinatorial recipes driven by node natures, and a structured 18-pattern progression sequence. The result is a puzzle-factory that rewards routing optimization and recipe discovery.

## 1. Pipelines

Implicit connections between nodes are replaced with player-built directional pipes.

### Rules
- Pipes are **one-directional** (Ember → Reflection, not bidirectional)
- Each pipe has a **bandwidth tier**: T1 (5/hr), T2 (12/hr), T3 (25/hr), T4 (50/hr)
- Max **3 outgoing, 3 incoming** pipes per node
- **Split ratios** on outgoing pipes are free to adjust anytime (the tinkering knob)

### Costs
- **Building** a new pipe: resources + 2 hour construction time
- **Upgrading** bandwidth tier: resources, instant
- **Demolishing**: refunds 50% of build materials, instant

### Backpressure
- Each node has a **buffer** storing up to 4 hours of production at current rate
- Buffer size scales automatically with node level
- When all outgoing pipes are at capacity or destinations are full, buffer fills
- **Buffer full = node stalls** — production stops, resources wasted

### Split Ratio UI
```
[EMBER] Lv 6  Output: 10/hr
  │
  ├══ 70% ═══► [REFLECT]  pipe T2 (12/hr cap)  using 7/hr  ✓
  ├══ 20% ═══► [VOID]     pipe T1 (5/hr cap)   using 2/hr  ✓
  └══ 10% ═══► buffer     storing excess

[↑↓] Select pipe  [←→] Adjust ratio  [Enter] Confirm
```

## 2. Node Natures

Each node has a **nature** that acts as a hidden ingredient in every reaction. Same inputs piped into different nodes produce different outputs.

| Node | Nature | Thematic Role |
|------|--------|---------------|
| Ember Spindle | Heat | Intensifies, accelerates, burns away impurity |
| Reflection Lens | Form | Gives structure, duplicates, refracts |
| Void Condenser | Void | Strips, purifies, reduces to essence |
| Memory Archive | Pattern | Records, preserves, creates blueprints |
| Silence Well | Stillness | Dampens, concentrates, creates potential |
| Resonance Forge | Vibration | Amplifies, harmonizes, creates feedback |

### Base Production
Each node still produces its native resource from prestige fuel with no pipe inputs. Reactions begin when a second resource is piped in.

### Example — Same Input, Different Nodes
```
Ember → Reflection Lens (Form):    produces Reflection
Ember → Silence Well (Stillness):  produces Condensed Ember
Ember → Memory Archive (Pattern):  produces Ember Echo
```

The recipe is: **Input A + Input B + Node Nature = Output**

## 3. Combinatorial Recipes

### Core Rules
- No recipe selection menu — output determined by what you pipe in and which node processes it
- Each node still produces its native resource with no inputs (base production)
- Two-input combinations produce Tier 1 and Tier 2 resources
- Three-input combinations (Tier 3) produce the highest-tier resources

### Layered Recipe Space
- **Tier 1** (~15 recipes): Available from start. Two-input combinations using base resources
- **Tier 2** (~10-12 recipes): Unlocked via pattern progression. Confluence resources become valid inputs
- **Tier 3** (~10-12 recipes): Late progression. Three-input combinations, Tapestry-tier resources
- **Total: ~35-40 hand-designed recipes**

### Recipe Codex
- First discovery of a recipe records it permanently in the codex
- Recipes **adjacent** to discovered ones show their input requirements but not the output ("???")
- Undiscovered recipes with no adjacent discovered recipes are invisible
- Codex accessible from the List View

## 4. Woven Patterns (Progression Milestones)

Patterns are the **sole progression gate**. The player always has one active pattern visible at the bottom of the Flow View.

### Rules
- Auto-detection — no manual start button
- Sustain timer fills while all required rates are met simultaneously
- If any rate dips, timer **pauses** (does not reset)
- Timer resumes when rates recover
- Completing a pattern unlocks the next pattern and its associated rewards

### UI — Pattern Bar (always visible)
```
├─ PATTERN: "The Burning Mirror" ──────────────────────────┤
│   Ember 3/hr ✓    Reflect 3/hr ✓    Forged Light 1/hr ✗ │
│   BLOCKED — need Forged Light production                 │
└──────────────────────────────────────────────────────────┘
```

When all rates are met:
```
├─ PATTERN: "The Burning Mirror" ──────────────────────────┤
│   Ember 3/hr ✓    Reflect 3/hr ✓    Forged Light 1/hr ✓ │
│   Weaving: ██████████████░░░░░░░░  1:14 / 2:00          │
└──────────────────────────────────────────────────────────┘
```

## 5. Pattern Sequence (18 Patterns)

### Teaching Arc (Patterns 1-6)
Each pattern introduces one new concept.

| # | Name | Requirements | Sustain | Teaches |
|---|------|-------------|---------|---------|
| 1 | First Thread | Ember 2/hr | 30 min | Base production exists |
| 2 | The Bridge | Ember 3/hr + Reflection 1/hr | 1 hr | Building your first pipe |
| 3 | Long Road | Ember 2/hr + Memory 1/hr | 1 hr | Routing to non-adjacent nodes |
| 4 | Balancing Act | Ember 2/hr + Reflection 2/hr + Void Essence 2/hr | 1.5 hr | Sustaining equal rates across a chain |
| 5 | Full Circle | All 6 base resources at 1/hr | 2 hr | Closing the cycle, feedback loop activates |
| 6 | The Catalyst | Condensed Ember 1/hr (Ember → Silence Well) | 2 hr | Node natures — same input, different node, different output |

### Mastery Arc (Patterns 7-12)
Combinations get harder, confluences enter.

| # | Name | Requirements | Sustain | Tests |
|---|------|-------------|---------|-------|
| 7 | Crossed Streams | 2 different reaction outputs simultaneously at 1/hr | 2 hr | Running two non-default recipes at once |
| 8 | The Diversion | Forged Light 1/hr while maintaining Ember 3/hr | 2.5 hr | Diverting without collapsing the base cycle |
| 9 | Three Confluences | All 3 confluence resources at 1/hr | 3 hr | Full confluence infrastructure |
| 10 | Pressure Test | Forged Light 2/hr + Echo Glass 2/hr + zero stalled nodes | 3 hr | Buffer management under load |
| 11 | The Bottleneck | Stillborn Song 3/hr | 3 hr | Maxing a single output — find and break the bottleneck |
| 12 | Shifting Gears | Forged Light 3/hr for 1 hr, then Echo Glass 3/hr for 1 hr (sequential) | 2 hr total | Reconfiguring mid-pattern — factory flexibility |

### Endgame Arc (Patterns 13-18)
Multiple tight constraints simultaneously.

| # | Name | Requirements | Sustain | Challenge |
|---|------|-------------|---------|-----------|
| 13 | Harmony | All 6 base resources at 5/hr simultaneously | 4 hr | Raw throughput across the whole cycle |
| 14 | The Triad | All 3 confluences at 3/hr + all 6 bases at 3/hr | 4 hr | Everything running at once |
| 15 | Razor's Edge | Forged Light 4/hr + Echo Glass 4/hr + max 2 pipes to each confluence node | 4 hr | Bandwidth optimization under pipe constraints |
| 16 | Resonance Cascade | Resonance 10/hr | 4 hr | Optimize entire cycle to maximize one output |
| 17 | The Unraveling | Woven Reality production + no node buffer exceeds 50% | 6 hr | Tapestry output with tight buffer discipline |
| 18 | Mended Loom | Woven Reality 3/hr + all 6 bases at 5/hr + all 3 confluences at 3/hr | 8 hr | Everything. The final test. Loom is mended. |

After Pattern 18: Woven Reality continues scaling infinitely as a high-score chase. No more pattern milestones. The narrative closes.

## 6. Reconfiguration Costs

| Action | Cost | Time |
|--------|------|------|
| Adjust pipe split ratios | Free | Instant |
| Upgrade pipe bandwidth | Resources | Instant |
| Build new pipe | Resources | 2 hours |
| Demolish pipe | -50% refund | Instant |

Encourages planning over thrashing. Split ratio adjustment is the free tinkering knob.

## 7. Session Rhythm

- **Every session (5-10 min):** Tweak split ratios, check buffer health, monitor pattern progress
- **Every few days:** Build/demolish pipes, try new input combinations, discover recipes
- **Each pattern completion:** Major redesign — new recipes change what's optimal, new requirements force rethinking the network
- **Idle between sessions:** Factory runs, sustain timer ticks, buffers fill and flow

## 8. Flow View UI

```
┌─ Flow View ──────────────────────────────────────────────┐
│                                                          │
│   [EMBER]══8/hr═══►[REFLECT]══5/hr═══►[VOID]           │
│     Lv6              Lv4               Lv4              │
│     Heat             Form              Void             │
│     Buf: ████░░      Buf: ██████░░     Buf: ██░░        │
│          ╚══2/hr══════════════════════════╝              │
│                                                          │
│   [RESON]◄══1/hr══[SILENCE]◄══2/hr══[MEMORY]           │
│     Lv2              Lv3              Lv3               │
│     Vibration        Stillness        Pattern           │
│     Buf: █░░░        Buf: ███░░       Buf: ████░░       │
│                                                          │
│  Pipes: 7/12    Bandwidth used: 68%    Codex: 8/37      │
│                                                          │
├─ PATTERN: "The Burning Mirror" ──────────────────────────┤
│   Ember 3/hr ✓    Reflect 3/hr ✓    Forged Light 1/hr ✗ │
│   BLOCKED — need Forged Light production                 │
└──────────────────────────────────────────────────────────┘
```

## Design Decisions Summary

- **Explicit pipelines** over implicit connections — routing IS the puzzle
- **Node natures as catalyst** — same inputs, different nodes, different outputs
- **Combinatorial recipes** over menu selection — discovery through experimentation
- **Layered recipe space** (~35-40 total) expanding with progression
- **Recipe codex** with adjacent discovery hints
- **18 curated woven patterns** as sole progression gate
- **Auto-detection with pausing timer** — forgiving but honest
- **Reconfigurable but costly** — free ratio tweaks, expensive pipe changes
- **Idle-with-tinkering rhythm** — 5-10 min sessions, major redesigns every few days

## 2026-03-06-loom-starting-archetypes-design.md

# Loom Starting Archetypes — Node Selection Design

**Date:** 2026-03-06
**Status:** Approved
**Extends:** [Loom of Worlds Design](2026-03-06-loom-of-worlds-design.md)

## Overview

When the Loom unlocks (after Gateway Expedition), the player chooses from 3 archetypes instead of 6 individual fracture nodes. Each archetype pairs two nodes with complementary passives. The first node unlocks immediately; the second activates ~4 hours later as an early milestone.

All passives create meaningful early-game divergence (weeks 1-2) but converge to negligible impact by the time confluences unlock.

## Design Decisions

- **3 archetypes** instead of 6 individual nodes — reduces decision paralysis for a system the player doesn't understand yet
- **Flavor-forward selection** — no mechanical jargon on the selection screen; thematic quotes plus plain-English hints
- **Staggered unlock** — first node immediate, second node ~4 hours later; avoids overwhelming the player and creates an early milestone
- **Early divergence, late convergence** — passives shape weeks 1-2 but become negligible by confluence phase

## Selection Screen

```
╔══════════════════════════════════════════════════════════════════╗
║                                                                  ║
║     The Loom lies before you — six threads, all frayed.          ║
║     Each wound in reality you crossed left its mark here.        ║
║     Choose the first threads to mend.                            ║
║                                                                  ║
╠══════════════════════════════════════════════════════════════════╣
║                                                                  ║
║  ┌─ Burn Bright ─────────────────────────────────────────┐      ║
║  │                                                       │      ║
║  │  Red Fault + Black Mouth                              │      ║
║  │  "It always burns first. Nothing wasted."             │      ║
║  │                                                       │      ║
║  │  Fast and efficient early, normalizes over time       │      ║
║  └───────────────────────────────────────────────────────┘      ║
║                                                                  ║
║  ┌─ Reach Wide ──────────────────────────────────────────┐      ║
║  │                                                       │      ║
║  │  Mirror Scar + Hollow Throne                          │      ║
║  │  "One reflection reveals three. The throne remembers."│      ║
║  │                                                       │      ║
║  │  More coverage and a head start from what came before │      ║
║  └───────────────────────────────────────────────────────┘      ║
║                                                                  ║
║  ┌─ Run Deep ────────────────────────────────────────────┐      ║
║  │                                                       │      ║
║  │  Wailing Reach + Origin Wound                         │      ║
║  │  "Silence is cheap. The echo came before the sound."  │      ║
║  │                                                       │      ║
║  │  Cheaper to grow, the cycle rewards you early         │      ║
║  └───────────────────────────────────────────────────────┘      ║
║                                                                  ║
║              [↑↓] Select    [Enter] Mend                         ║
╚══════════════════════════════════════════════════════════════════╝
```

## Archetypes

### Burn Bright (Red Fault + Black Mouth)

| Node | Unlock | Passive | Convergence |
|------|--------|---------|-------------|
| Ember Spindle | Immediate | +50% throughput, neighbors unlock 30% slower | Irrelevant when base throughput is 200+/hr |
| Void Condenser | ~4 hours | 2x conversion ratio at levels 1-3 | Gone by day 2 |

**Feel:** Produce a lot, convert efficiently. Narrow but powerful early. The player who wants to see big numbers fast.

### Reach Wide (Mirror Scar + Hollow Throne)

| Node | Unlock | Passive | Convergence |
|------|--------|---------|-------------|
| Reflection Lens | Immediate | Unlocks 3 neighbors instead of 2 | One-time unlock, done by day 3 |
| Memory Archive | ~4 hours | Starts with stockpile of 3 adjacent resources | Consumed immediately |

**Feel:** Spread fast, skip cold-starts. Wide but shallow early. The player who wants to see the whole system sooner.

### Run Deep (Wailing Reach + Origin Wound)

| Node | Unlock | Passive | Convergence |
|------|--------|---------|-------------|
| Silence Well | Immediate | -25% upgrade costs for first 5 levels | Done by day 3-4 |
| Resonance Forge | ~4 hours | Feedback loop at 50% strength before cycle closes | Replaced by real 100% loop |

**Feel:** Cheap scaling, early feedback loop. Patient builder, big payoff. The player who wants to invest for later.

## Unlock Flow

```
Player completes Gateway Expedition (7 days)
         │
         ▼
   Selection screen — pick 1 of 3 archetypes
         │
         ▼
   Node 1 activates immediately
   Player learns: what a node is, production, upgrading
         │
         │  ~4 hours of play
         ▼
   Node 2 activates — "A second thread responds"
   Player learns: connections between nodes, conversion
         │
         │  Continued play
         ▼
   Adjacent nodes unlock normally from here
   (standard Loom progression per original design)
```

## Convergence Timeline

By week 2, all three archetypes are mechanically identical. The choice shaped the journey, not the destination.

- Ember's +50% throughput: negligible vs scaled base rates
- Void's 2x ratio: only levels 1-3 (gone day 2)
- Reflection's extra neighbor: one-time effect (done day 3)
- Memory's stockpile: consumed immediately
- Silence's -25% cost: only first 5 levels (done day 3-4)
- Resonance's 50% feedback: replaced by real 100% loop once cycle closes

## 2026-03-07-loom-flow-view-redesign.md

# Loom Flow View Redesign: Living Machines

## Summary

Replace the current text-table Flow View with an animated factory floor rendered using `scene_fx` cell buffers. Six machine nodes in a 3x2 grid, each with unique animated textures, buffer bars, and recipe input slots. Port labels below each box show connections without drawn wires. A sidebar shows full detail for the selected node. Animations freeze on stall. Responsive fallbacks at smaller terminal sizes.

## Motivation

The current Flow View is a data table with paired rows showing node headers, buffer bars, and pipe connectors. It works but doesn't scale visually when pipes cross between non-paired nodes, and lacks the spatial "factory" feel of games like Factorio and Shapez. Players should be able to look at the Loom and see machines running, resources flowing, and bottlenecks at a glance.

## Overall Layout

```
+======================================+====================+
|                                      |                    |
|          Factory Floor               |      Sidebar       |
|        (3x2 machine grid)           |   (selected node   |
|                                      |    detail)         |
|                                      |                    |
+======================================|                    |
|        Pattern Bar (3 rows)          |                    |
+======================================+====================+
```

- **Factory floor** (~60% width): 3 rows x 2 columns of animated machine nodes. Arrow keys move selection. Primary visual area.
- **Sidebar** (~20 cols fixed right): Detail for selected node -- level, buffer, production rate, all pipes, all possible recipes with input slot indicators.
- **Pattern bar** (3 rows, bottom-left): Active pattern progress (unchanged from current).

Node grid positions are fixed by archetype pair:

| Row | Left | Right |
|-----|------|-------|
| 0 | Ember Spindle | Void Condenser |
| 1 | Reflection Lens | Memory Archive |
| 2 | Silence Well | Resonance Forge |

## Node Box Anatomy

Each node is roughly 22 cols x 6 rows:

```
+----- Ember Spindle --- Lv.5 ----+
| ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ |    <- animated texture (2 rows)
| ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ |
| ########.............. 18/40    |    <- buffer bar
| [*Emb] [*Void] > ForgedLt      |    <- recipe slots
+---------------------------------+
 ->V ->R ->M    <-F                    <- port labels (outside box)
```

### Title Bar

Node name and level. Selected node gets a bright border, unselected gets a dim border.

### Animated Texture (2 rows)

Unique per node type, cycles every ~300ms using `scene_fx::current_millis()`. Freezes and dims when stalled. Locked nodes show a lock icon instead.

| Node | Texture | Feel |
|------|---------|------|
| Ember Spindle | `~ ~ ~` shifting wave | Flickering heat |
| Reflection Lens | `. * . *` twinkling | Light refracting |
| Void Condenser | `: : : :` dripping | Dark matter condensing |
| Memory Archive | `x x x` crosshatch | Woven patterns |
| Silence Well | `_ _ _ _` still ripple | Calm surface |
| Resonance Forge | `~ ~ ~` vibrating | Sound waves |

Animation is a column offset shift computed at render time: `offset = (current_millis() / 300) % pattern_length`. Each node type uses a different shift direction (horizontal for Ember, vertical for Resonance, brightness pulse for Void).

### Buffer Bar

`#` filled / `.` empty. Colored green (< 75%), yellow (75-90%), red (> 90% or stalled). Shows `current/capacity` numerically after the bar.

### Recipe Slots

Shows the best active or candidate recipe for this node's nature. Two input indicators followed by an arrow and output name:

- `[*Emb]` -- filled, this resource is arriving via pipe (bright)
- `[oSlnc]` -- empty, not connected (dim red)
- `> ForgedLt` -- output name (bright when producing, dim when missing input)

When producing, the `>` arrow pulses on a 500ms cycle.

When no recipes are discovered, shows `? + ? > ???` dimmed.

### Port Labels

Sit outside the node box, below it. Use single-letter color-coded abbreviations:

- **E** = Ember Spindle (orange)
- **V** = Void Condenser (purple)
- **R** = Reflection Lens (cyan)
- **M** = Memory Archive (yellow)
- **S** = Silence Well (gray)
- **F** = Resonance Forge (blue)

Format: outgoing on left (`->V ->R ->M`), incoming on right (`<-F`), separated by gap.

Under-construction pipes blink on a 500ms cycle.

Max port labels per node: `->X ->X ->X  <-X <-X <-X` (~20 chars), fits under the box.

## Selection and Connection Highlighting

When a node is selected:
1. Its border brightens (dim `+--+` to bright `+==+`)
2. Its port labels brighten
3. Matching port labels on connected nodes also brighten (e.g., selecting Ember makes `<-E` on Void brighten)

This lets players visually trace connections without drawn wires.

## Sidebar Detail Panel

~20 columns wide. Updates when selection changes. Content top to bottom:

1. **Node identity**: Name, level, nature type (Heat/Void/Form/etc)
2. **Buffer + rate**: Bar and exact numbers with more room than the node box
3. **Recipe list**: All recipes matching this node's nature from the recipe registry. Each shows two input slots as filled/empty with output name. Active recipes bright, inactive dim. This is the key planning tool.
4. **Pipe list**: All outgoing and incoming pipes with flow rate and tier
5. **Controls**: Context-sensitive key hints ([B]uild, [U]pgrade, [D]emolish, [S]plit)

## Animation System

All animations are computed at render time from `scene_fx::current_millis()`. No new state or tick counters needed.

- **Texture cycling**: `frame = (current_millis() / 300) % frame_count`
- **Production pulse**: `bright = (current_millis() / 500) % 2 == 0`
- **Construction blink**: `visible = (current_millis() / 500) % 2 == 0`
- **Stall**: Freeze at frame 0, dim to dark gray
- **Locked**: No texture, lock icon, no animation

Performance: purely cosmetic render-time computation. The existing 100ms tick loop already redraws every frame, giving ~10fps animation.

## Rendering Approach

Switch from `Paragraph` with `Line`/`Span` (row-based text) to `scene_fx` cell buffer rendering (`SceneCell`, `put_text`, `put_cell`, `render_buffer`). This gives per-character control needed for:
- Placing node boxes at exact grid coordinates
- Animating texture patterns with color per cell
- Drawing buffer bars with per-cell coloring
- Highlighting ports across distant nodes

The sidebar can remain `Paragraph`-based since it's standard text.

## Responsive Sizing

| Tier | Min Size | Behavior |
|------|----------|----------|
| L/XL | 80x30+ | Full layout: factory floor + sidebar + pattern bar |
| M | 60x24+ | Drop sidebar, show 2-line detail strip at bottom instead of pattern bar |
| S | 40x16+ | Fall back to existing text-based List+Detail view |

## Edge Cases

- **No archetype**: Archetype selection screen (unchanged)
- **One node unlocked**: Only that node renders as a machine. Others show as dim locked boxes.
- **No pipes**: No port labels. Clean factory floor with isolated machines.
- **No recipes discovered**: Recipe slots show `? + ? > ???` dimmed. Sidebar says "No recipes discovered."
- **Stalled node**: Texture freezes, dims. Buffer bar turns red. Immediately obvious.

## Relationship to Other Views

The Flow View becomes the default/primary Loom view. List+Detail and Codex remain as separate Tab-accessible views, unchanged. They serve as supplementary data views for detailed pipe management and recipe browsing.

## 2026-03-07-loom-power-integration-design.md

# Loom Power Integration Design

## Overview

The Loom of Worlds feeds power back into the main game through three mechanisms: **Ascension level unlocks** (VII–X) gated by pattern milestones, **WR→PR generation** that activates once all patterns are complete, and **Loom Zones** (Z31–50) that give players combat content scaled to their new power. Shuttle upgrades provide post-pattern endgame scaling.

---

## 1. Ascension Unlocks

| Ascension | Multiplier | PR Cost | Patterns Required | Shuttle Max Level |
|-----------|-----------|---------|-------------------|-------------------|
| VII | 96x | 1,500 | 8 | 3 |
| VIII | 144x | 4,000 | 16 | 5 |
| IX | 216x | 8,000 | 22 | 7 |
| X | 324x | 15,000 | 28 | 10 |

- Uses existing formula: `64 × 1.5^(level-6)`
- Pattern milestones are arc-aligned (early/mid/late/endgame patterns)
- Ascension VII–X appear in the Ascension UI only after meeting pattern requirements
- PR costs are deliberately high to create demand for the WR→PR pipeline

## 2. WR→PR Generation

Unlocks at Ascension X (all 28 patterns complete). Sustained WR production converts to PR via tiered brackets:

| WR/hr Bracket | PR per WR/hr per day |
|---------------|---------------------|
| 0–10 | 5 |
| 10–25 | 10 |
| 25+ | 15 |

Example: 60 WR/hr = (10×5) + (15×10) + (35×15) = 50 + 150 + 525 = **725 PR/day**

Compare to Power Cores max of 48 PR/day — this is the next tier of power generation.

## 3. Shuttle Upgrades

- Shuttle level multiplier applies to intake cap: `tier_intake_cap(tier) × node_level_multiplier(level)`
- Same level multiplier formula as nodes: `1.0 + (level-1) × 0.5`
- Upgrade cost: same formula as nodes, paid from shuttle's output buffer
- Max level capped by current Ascension tier (see table above)
- Upgrade UI appears only after Ascension VII

## 4. Loom Zones (Z31–50)

**Unlock gating** — triple-gated by pattern milestones, ascension level, and prestige rank:

| Patterns | Ascension | Prestige | Zones Unlocked |
|----------|-----------|----------|----------------|
| 4 | — | P2,000 | Z31–34 |
| 8 | VII | P5,000 | Z35–38 |
| 16 | VIII | P15,000 | Z39–42 |
| 22 | IX | P30,000 | Z43–46 |
| 28 | X | P50,000 | Z47–50 |

**Stat scaling** — 1.25x per zone from Z31 base (gentler than the 1.6x used for Z12–30, tuned to the ~5x power growth across Ascension VII–X):

| Player State | Comfortable Zone Range |
|-------------|----------------------|
| Asc VI (64x), no Loom power | Z30 (current cap) |
| Asc VII (96x) | Z31–36 |
| Asc VIII (144x) | Z37–41 |
| Asc IX (216x) | Z42–46 |
| Asc X (324x) | Z47–49 |
| Asc X + optimized shuttles | Z50 |

Z50 is the "you need everything" zone — completable but punishing without Ascension X and shuttle optimization.

**Zone naming** — These are Loom-themed zones (woven realms, thread dimensions, etc.) rather than additional Fracture zones. Thematic names TBD during implementation.

## 5. What This Doesn't Change

- Early/mid Loom gameplay is untouched — no shuttle upgrades, no new Ascension levels visible until 8 patterns, first zone unlock at 4 patterns
- Existing Ascension I–VI unchanged
- Existing Power Cores unchanged
- Existing Fracture zones Z12–30 unchanged

## 6. Player Journey

1. Discover Loom → unlock nodes → build shuttles → produce WR → complete patterns
2. At 4 patterns → Z31–34 unlock, giving immediate combat engagement with Loom progression
3. At 8 patterns → Ascension VII (1,500 PR), shuttle upgrades unlock (max 3), Z35–38 unlock
4. Continue patterns + optimize shuttles → 16 patterns → Ascension VIII (4,000 PR), shuttle cap 5, Z39–42
5. Push further → 22 patterns → Ascension IX (8,000 PR), shuttle cap 7, Z43–46
6. All 28 patterns → Ascension X (15,000 PR), shuttle cap 10, Z47–50, WR→PR generation activates
7. Post-completion: optimize shuttle levels to push WR/hr higher → more PR/day → fund Ascension X cost → conquer Z50

## 2026-03-07-loom-remove-pipes-design.md

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

## Patterns as Raw Amounts

Patterns require producing a **total amount** of each resource, not sustaining a rate for a duration. The accumulator increments each tick based on actual production rate. Players see a number climbing toward a goal.

Example display:
```
─── Pattern: Thread of Dawn ────────── ██████░░░░░░░░░░ 37/60 ──────
  ForgedLight: 37/60  (+2.0/hr)
```

`PatternRequirement` has `amount: f64` (total needed) instead of `rate_per_hour`. The tick adds `actual_rate * delta_hours` to `accumulated`. Pattern completes when all requirements reach their amount.

Conversion from old rate×time model: `amount = rate_per_hour * sustain_seconds / 3600`.

## Optimization Loop

The player's cycle:
1. Check what the current pattern demands (total resource amounts)
2. Work backwards — how many refineries at each tier to produce fast enough?
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

## 2026-03-07-loom-sustained-patterns-design.md

# Loom of Worlds — Sustained Rate Pattern Redesign

## Overview

Redesign the Woven Pattern system from accumulated totals to sustained production rates. Patterns require the player to maintain a minimum flow rate for a duration, proving their production network works reliably. Expand from 18 to 28 patterns with full resource coverage.

## Core Mechanic: Sustained Flow Rates

### How It Works

Each pattern requirement specifies:
- **Rate threshold** (units/hr) — minimum production rate the player must sustain
- **Sustain duration** (hours) — how long the rate must be maintained

The player builds their production network to meet the threshold, verifies rates are green, then walks away. The Loom drinks continuously in the background.

### Measurement

**60-second rolling window average** (600 ticks at 100ms/tick). Smooths production spikes without being sluggish. Running sum maintained for O(1) per tick.

```rust
struct RateTracker {
    buffer: VecDeque<f64>,   // last 600 ticks of production
    window_size: usize,      // 600
    sum: f64,                // running sum
}
```

### Failure Model: Simple Pause

When the rate drops below the threshold:
- **Progress timer freezes** — does not advance, does not decay
- When rate recovers, timer resumes exactly where it left off
- **Progress is never lost**, only paused

This is idle-friendly: if the player walks away and something breaks, they fix it when they come back and the timer picks up.

### Requirement Completion

Requirements complete **independently**. The player doesn't need to sustain all resources simultaneously. Once Ember's sustain timer finishes, it locks complete even if other resources are still running.

### UI States

| State | Condition | Visual |
|-------|-----------|--------|
| **Advancing** | Rate >= threshold | Green bar filling, rate in green |
| **Paused** | Rate < threshold | Bar frozen, pulses amber, rate in yellow |

Example line:
```
Ember:  ████████░░░░░░░░  15:00/30:00   52/hr (need 25/hr) ✓
Echo:   ████████████░░░░  22:00/30:00   11/hr (need 15/hr) ⏸
```

### Persistence

Save only `sustained_secs` per requirement. On load, restart the rolling window from empty (60-second ramp-up is negligible). Offline: use configured production rates and simulate normally.

## Required Fix: Buffer Overflow

Extractors produce into a 200-unit buffer. When full, production halts — silently breaking sustained rate patterns.

**Fix:** Auto-drain excess. When buffer hits capacity, excess production is discarded. The extractor keeps producing at full rate. The buffer exists as a reservoir for refineries, not as a production gate.

## Tier Gates (shifted for 28 patterns)

| Tier | Gate | What it unlocks |
|------|------|----------------|
| T1 | 1 pattern complete | Base x Base recipes |
| T2 | 8 patterns complete | Confluence x Base recipes |
| T3 | 15 patterns complete | Confluence x Confluence recipes (Woven Reality) |

## The 28-Pattern Progression

### Teaching Arc — 3 days (72 hours)

| # | Name | Requirements | Duration |
|---|------|-------------|----------|
| 1 | First Thread | Ember 25/hr | 2 hr |
| 2 | Still Waters | Silence 25/hr | 2 hr |
| 3 | Echoing Halls | Memory 25/hr | 4 hr |
| 4 | Harmonic Pulse | Resonance 25/hr | 4 hr |
| 5 | Mirror and Void | Reflection 30/hr, VoidEssence 30/hr | 6 hr |
| 6 | Full Circle | All 6 base @ 20/hr | 10 hr |
| 7 | The Catalyst | CondensedEmber 8/hr | 16 hr |
| 8 | Echo of Flame | EmberEcho 8/hr | 28 hr |

### Mastery Arc — 10 days (236 hours)

| # | Name | Requirements | Duration |
|---|------|-------------|----------|
| 9 | Forged in Fire | ForgedLight 15/hr | 16 hr |
| 10 | Glass Resonance | EchoGlass 15/hr | 16 hr |
| 11 | The Unsung | StillbornSong 15/hr | 24 hr |
| 12 | Void Distillation | PurifiedVoid 10/hr | 24 hr |
| 13 | Crossed Streams | ForgedLight 12/hr, EchoGlass 12/hr | 24 hr |
| 14 | The Asymmetry | ForgedLight 25/hr, StillbornSong 8/hr | 36 hr |
| 15 | Pressure Test | CondensedEmber 15/hr, EmberEcho 10/hr, PurifiedVoid 10/hr | 36 hr |
| 16 | Three Confluences | ForgedLight 18/hr, EchoGlass 18/hr, StillbornSong 18/hr | 60 hr |

### Endgame Arc — 22 days (534 hours)

| # | Name | Requirements | Duration |
|---|------|-------------|----------|
| 17 | The Amplifier | ForgedLight 35/hr | 18 hr |
| 18 | Purified Cascade | PurifiedVoid 20/hr, ForgedLight 20/hr | 24 hr |
| 19 | Resonance Cascade | Resonance 150/hr, StillbornSong 25/hr | 24 hr |
| 20 | First Weave | WovenReality 5/hr | 30 hr |
| 21 | The Unraveling | WovenReality 15/hr, PurifiedVoid 15/hr | 36 hr |
| 22 | Grand Harmony | All 6 base @100/hr, all 3 confluence @30/hr | 36 hr |
| 23 | The Knot | ForgedLight 25/hr, PurifiedVoid 15/hr, CondensedEmber 12/hr | 36 hr |
| 24 | Strange Alchemy | ForgedLight 30/hr, EchoGlass 30/hr, StillbornSong 30/hr, Ember 80/hr, VoidEssence 80/hr | 42 hr |
| 25 | Refined Purpose | PurifiedVoid 30/hr, ForgedLight 25/hr | 48 hr |
| 26 | The Flood | WovenReality 35/hr | 48 hr |
| 27 | Everything Flows | All 13 resources at moderate rates | 72 hr |
| 28 | Mended Loom | WovenReality 20/hr, confluences @40/hr, Ember/Silence/Resonance @80/hr | 120 hr |

### Duration Summary

| Arc | Days | Range | Longest |
|-----|------|-------|---------|
| Teaching | 3 | 2 hr → 28 hr | Echo of Flame |
| Mastery | 10 | 16 hr → 60 hr | Three Confluences |
| Endgame | 22 | 18 hr → 120 hr | Mended Loom |
| **Total** | **35 days** | | |

### Resource Coverage

- All 13 resources featured at least once
- PurifiedVoid: 4 appearances (#12, #15, #18, #21)
- EchoGlass: solo spotlight (#10)
- Every base resource introduced individually before Full Circle (#6)
- WovenReality: 4 appearances (#20, #21, #26, #28)

### Mechanical Challenge Types

| Challenge | Patterns |
|-----------|---------|
| Raw throughput | #17 The Amplifier, #19 Resonance Cascade |
| Multi-tier chains | #20 First Weave, #21 The Unraveling |
| Full network | #22 Grand Harmony, #27 Everything Flows |
| Source contention | #23 The Knot, #15 Pressure Test |
| Recipe exploration | #24 Strange Alchemy |
| T2 depth | #25 Refined Purpose |
| Vertical scaling | #26 The Flood |
| Ultimate endurance | #28 Mended Loom |

## Narrative

### Discovery Text
> Beyond the Gateway, in a chamber older than memory, you find it — a vast mechanism of thread and light, broken and silent. The Loom of Worlds. Its spindles are dark, its weave unraveled. But as you draw near, something stirs. It has been waiting.

### Mastery Arc Opening
> The Loom no longer resists your touch. Its threads respond, its shuttle awaits. But comprehension is not mastery — now it demands not drops, but rivers. Sustain what you have learned. Feed it without faltering, and the weave will deepen beyond what teaching alone could reach.

### Endgame Arc Opening
> The Mastery Arc is complete. The Loom stirs — not with memory now, but with hunger. It remembers what it was, and demands you prove you can sustain what it will become. The final patterns require not moments of brilliance, but days of unwavering flow.

### Completion Text
> The Loom is whole. Across five days of unbroken flow, you wove what was shattered back into coherence. The hum beneath the world is steady now — not because it was fixed, but because you learned to sustain it. The Gateway dims. The work is done.

### Pattern Flavor Text

#### Teaching Arc

| # | Name | Flavor Text |
|---|------|------------|
| 1 | First Thread | A single ember, held steady, and the Loom stirs from its long silence. It drinks the warmth like parched earth drinks rain. |
| 2 | Still Waters | The Loom remembers stillness before fire. Feed it silence now — a sustained hush, patient and unbroken. |
| 3 | Echoing Halls | Hour after hour, memory flows into the weave. The Loom recalls what it was, one slow thread at a time. |
| 4 | Harmonic Pulse | A steady hum, sustained without faltering. The Loom tests whether its ancient frame can still hold a resonant frequency. |
| 5 | Mirror and Void | Form requires emptiness to fill. Feed the Loom both shape and absence together, and watch structure take root in nothing. |
| 6 | Full Circle | Six forces flow as one — ember, silence, memory, resonance, reflection, void. The Loom tastes the full spectrum for the first time in eons. |
| 7 | The Catalyst | Raw heat alone no longer suffices. The Loom demands ember compressed into dense purpose — a refinery's slow, steady yield. |
| 8 | Echo of Flame | Not fire itself but its afterimage — the memory of heat, distilled drop by drop. The Loom teaches you that recipe and rhythm both matter. |

#### Mastery Arc

| # | Name | Flavor Text |
|---|------|------------|
| 9 | Forged in Fire | Where ember meets void, light is born from contradiction. The Loom drinks this paradox through the night, steady as a forge that must not cool. |
| 10 | Glass Resonance | Memory poured through silence becomes glass that remembers. The Loom turns each reflection inward, weaving sixteen hours of frozen echoes into its frame. |
| 11 | The Unsung | A song caught between silence and resonance, never born yet always present. The silence that shapes it also feeds the glass — the first thread you must learn to share. |
| 12 | Void Distillation | To distill absence is to hold nothing so carefully it becomes substance. The Loom asks for pure potential, drawn slow and steady from the emptiness between things. |
| 13 | Crossed Streams | Light forged from fire, glass shaped from memory — both streams flowing at once. The Loom weaves with both hands now, and neither may falter. |
| 14 | The Asymmetry | The Loom demands a river of forged light but only a trickle of unsung sound. Unequal hungers require unequal commitment — one furnace will not suffice. |
| 15 | Pressure Test | Three streams drawn from shared roots, each pulling at the same deep wells of ember and void. The network strains. The Loom does not care — it drinks regardless. |
| 16 | Three Confluences | Light, glass, and silence-song sustained together for days without interruption. Every source is contested, every stream must hold. This is the full symphony the Loom was waiting to hear. |

#### Endgame Arc

| # | Name | Flavor Text |
|---|------|------------|
| 17 | The Amplifier | No single forge burns bright enough. The Loom demands a light that only parallel flames can cast. |
| 18 | Purified Cascade | Light must feed the void and still shine. Split the stream without dimming either side. |
| 19 | Resonance Cascade | The Forge screams for upgrades while Song drinks from the same well. Widen the source or drown in contention. |
| 20 | First Weave | A trickle of Woven Reality emerges from a chain three tiers deep. You are weaving existence itself now. |
| 21 | The Unraveling | Reality and void flow side by side, drawing from the same tiers. What feeds one starves the other without perfect coordination. |
| 22 | Grand Harmony | Nine rivers at once. Every extractor upgraded, every confluence sustained, every flow unbroken. The Loom hums on all frequencies. |
| 23 | The Knot | Three refineries. One Ember source. The Spindle cannot serve all masters — unless you untangle what seemed inseparable. |
| 24 | Strange Alchemy | Canonical recipes would starve the base flows the Loom also demands. Find stranger paths, or watch everything collapse. |
| 25 | Refined Purpose | Raw light is no longer enough. The Loom wants what only layered transmutation can provide. Build the chain deep. |
| 26 | The Flood | Parallel chains, eight refineries wide, all converging on a single thread. Reality pours forth like a river finding the sea. |
| 27 | Everything Flows | Thirteen resources. Seventy-two hours. Every tier, every recipe, every extractor singing in unison. This is the network you were always building toward. |
| 28 | Mended Loom | Five days. Every confluence roaring, every base resource at full draw, Woven Reality streaming into the ancient framework without pause. The Loom does not flicker. It does not falter. Thread by thread, the world holds. |

## 2026-04-04-loom-graph-view.md

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

## 2026-04-04-loom-graph-view-design.md

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
