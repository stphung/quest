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
