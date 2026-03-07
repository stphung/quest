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
