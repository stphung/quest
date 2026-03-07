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
