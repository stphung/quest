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
