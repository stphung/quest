# Quest Balancing Guide

How to tune Quest's game economy without breaking progression. This document covers balance philosophy, key levers, danger zones, and testing methodology.

## Table of Contents

1. [Balance Philosophy](#balance-philosophy)
2. [The Core Loop](#the-core-loop)
3. [Progression Pacing](#progression-pacing)
4. [Key Balance Levers](#key-balance-levers)
5. [System Interactions](#system-interactions)
6. [Danger Zones](#danger-zones)
7. [Testing & Validation](#testing--validation)
8. [Common Tuning Scenarios](#common-tuning-scenarios)

---

## Balance Philosophy

### Idle RPG Principles

Quest is an **idle RPG** — balance should support:

1. **Meaningful AFK progress** — Players should feel rewarded for leaving the game running
2. **Active play advantage** — But active decisions (prestige timing, minigames, Haven) should outpace pure idling
3. **Long-term goals** — Endgame (Stormbreaker) should take weeks/months, not hours
4. **No hard walls** — Progress should slow but never stop completely
5. **Prestige feel-good** — Each reset should feel like a meaningful power boost

### The Golden Ratio

```
Active play should be ~2-3× more efficient than pure idle.
```

This means:
- Winning minigames for prestige ranks beats grinding levels
- Strategic prestige timing beats waiting for max level
- Haven investment pays off over multiple prestiges

### Player Psychology Targets

| Milestone | Target Time | Feel |
|-----------|-------------|------|
| First prestige (P1) | 30-60 min | "I get it now" |
| Haven unlock (P10) | 8-12 hours | "New system!" |
| Stormbreaker | 2-4 weeks | "Finally!" |
| The Expanse cycles | Infinite | "One more run" |

---

## The Core Loop

```
┌─────────────────────────────────────────────────────────────────┐
│                    CORE PROGRESSION LOOP                         │
└─────────────────────────────────────────────────────────────────┘

    ┌──────────┐
    │  Combat  │◀─────────────────────────────────┐
    │  (Idle)  │                                  │
    └────┬─────┘                                  │
         │ XP + Items                             │
         ▼                                        │
    ┌──────────┐                                  │
    │  Level   │                                  │
    │   Up     │                                  │
    └────┬─────┘                                  │
         │ +3 Attributes                          │
         ▼                                        │
    ┌──────────┐     ┌──────────┐                │
    │  Power   │────▶│  Zone    │                │
    │ Increase │     │ Progress │                │
    └──────────┘     └────┬─────┘                │
                          │ Wall                  │
                          ▼                       │
                    ┌──────────┐                  │
                    │ Prestige │──────────────────┘
                    │  Reset   │ (Multiplier boost)
                    └──────────┘
```

### What Makes This Work

1. **XP scales with prestige** — Higher prestige = faster XP = faster levels
2. **Attribute caps scale** — Higher prestige = higher potential power
3. **Zones gate progress** — Can't rush ahead without prestige investment
4. **Multiplier diminishes** — Each prestige matters less, preventing runaway

---

## Progression Pacing

### XP Curve Analysis

Current formula: `xp_needed = 100 × level^1.5`

| Level | XP Needed | Time at P0* | Time at P10* |
|-------|-----------|-------------|--------------|
| 10 | 3,162 | ~5 min | ~1.5 min |
| 50 | 35,355 | ~1 hour | ~20 min |
| 100 | 100,000 | ~3 hours | ~50 min |
| 200 | 283,000 | ~8 hours | ~2.5 hours |

*Approximate, assuming constant combat with average kill XP.

### Prestige Level Requirements

The level gates create natural prestige timing:

| Prestige | Required Level | Typical Play Time |
|----------|----------------|-------------------|
| P1 | 10 | 30-60 min |
| P5 | 80 | 4-6 hours cumulative |
| P10 | 130 | 10-15 hours cumulative |
| P20 | 235 | 40-60 hours cumulative |

**Key insight**: Later prestiges require exponentially more time, but the multiplier gains shrink. This is intentional — it creates diminishing returns that prevent infinite scaling.

---

## Key Balance Levers

### Lever 1: XP Curve Exponent

```rust
// Current: 1.5
xp_needed = 100 × level^EXPONENT
```

| Exponent | Effect |
|----------|--------|
| 1.3 | Faster leveling, shorter prestige cycles |
| 1.5 | **Current** — balanced idle pacing |
| 1.7 | Slower leveling, more grind per prestige |
| 2.0 | Very slow — only for hardcore modes |

**When to adjust**: If prestiges feel too fast/slow.

### Lever 2: Prestige Multiplier Formula

```rust
// Current: diminishing returns
multiplier = 1.0 + 0.5 × rank^0.7
```

| Formula | P1 | P10 | P20 | Character |
|---------|-----|------|------|-----------|
| `1 + 0.3 × rank^0.7` | 1.3× | 2.5× | 3.5× | Slower power curve |
| `1 + 0.5 × rank^0.7` | 1.5× | 3.5× | 5.1× | **Current** |
| `1 + 0.7 × rank^0.7` | 1.7× | 4.5× | 6.7× | Faster power curve |

**When to adjust**: If prestige feels unrewarding (increase) or trivializes content (decrease).

### Lever 3: Kill XP Range

```rust
// Current: 200-400 ticks worth of XP per kill
kill_xp = xp_per_tick × random(MIN..MAX)
```

| Range | Effect |
|-------|--------|
| 100-200 | More passive-like, longer fights matter less |
| 200-400 | **Current** — kills are ~30× passive value |
| 300-600 | Kills dominate, pure idle is weak |

**When to adjust**: If combat feels unrewarding vs pure idling.

### Lever 4: Attribute Scaling

```rust
// Damage formula
physical_damage = 5 + (STR_mod × DAMAGE_PER_MOD)
// Current: DAMAGE_PER_MOD = 2

// HP formula  
max_hp = 50 + (CON_mod × HP_PER_MOD)
// Current: HP_PER_MOD = 10
```

**When to adjust**: If characters feel too squishy/tanky or damage feels low/high.

### Lever 5: Drop Rates

```rust
BASE_DROP_RATE = 0.15        // 15% per kill
PRESTIGE_BONUS = 0.01        // +1% per rank
MAX_DROP_RATE = 0.25         // 25% cap
```

**When to adjust**: If players are drowning in loot (decrease) or items feel too rare (increase).

### Lever 6: Haven Bonuses

Each Haven room has T1/T2/T3 values. These are percentage-based and stack multiplicatively with other systems.

| Room | Tuning Consideration |
|------|----------------------|
| Hearthstone | Offline XP — affects AFK players most |
| Armory | Raw damage — directly speeds combat |
| Training Yard | XP gain — speeds all progression |
| War Room | Double Strike — multiplicative damage |

**Danger**: Haven bonuses are permanent and cumulative. Small changes compound across all future play.

---

## System Interactions

### Interaction Matrix

```
           │ Prestige │ Haven │ Items │ Fishing │ Challenges
───────────┼──────────┼───────┼───────┼─────────┼───────────
Prestige   │    -     │ Gates │ Reset │ Persist │ Rewards PR
Haven      │ Currency │   -   │ Rarity│ Rank Cap│ Discovery
Items      │ Lost     │ Vault │   -   │ Drops   │    -
Fishing    │ Persist  │ Dock  │ Drops │    -    │ Ranks
Challenges │ +Ranks   │Library│   -   │ +Ranks  │    -
```

### Critical Chains

**1. Prestige → Haven → Everything**
```
More prestige → More Haven rooms → Permanent bonuses → Faster prestige
```
This is a **virtuous cycle** — players who engage with Haven accelerate faster.

**2. Fishing → Stormbreaker Gate**
```
Fishing Rank 40 → Storm Leviathan → StormForge → Zone 10 boss
```
This chain gates endgame. If fishing is too fast/slow, endgame timing shifts.

**3. Challenges → Prestige Shortcuts**
```
Minigame wins → +Prestige ranks → Skip level grinding
```
Skilled players can prestige faster via minigames.

---

## Danger Zones

### 🚨 Do NOT Touch Without Testing

| Constant | Risk |
|----------|------|
| `TICK_INTERVAL` (100ms) | Breaks all timing, UI responsiveness |
| `BASE_XP_PER_TICK` (1.0) | Ripples through entire XP economy |
| Zone prestige requirements | Blocks/trivializes content |
| Prestige level requirements | Core progression pacing |
| `MAX_FISHING_RANK` (40) | Breaks Stormbreaker chain |

### ⚠️ High-Impact Changes

| Change | Ripple Effects |
|--------|----------------|
| XP curve exponent | All level timings, prestige pacing |
| Prestige multiplier | Power curve, Haven value |
| Haven T3 bonuses | Endgame power ceiling |
| Challenge prestige rewards | Speedrun strategies |

### ✅ Safe to Tune

| Change | Isolated To |
|--------|-------------|
| Fish rarity weights | Fishing feel |
| Enemy name syllables | Flavor only |
| Item affix ranges | Item power variance |
| Dungeon room types | Dungeon variety |
| UI colors/layout | Presentation |

---

## Testing & Validation

### Quick Smoke Test

```bash
cargo run -- --debug
```

Use debug menu (backtick) to:
1. Trigger fishing → verify rank-up timing
2. Trigger challenges → verify rewards apply
3. Trigger Haven → verify discovery and building

### Progression Simulation

To test XP/prestige pacing without playing:

```rust
// Add to tests or a scratch file
fn simulate_progression(prestiges: u32) {
    let mut total_time = 0.0;
    for p in 0..=prestiges {
        let mult = 1.0 + 0.5 * (p as f64).powf(0.7);
        let req_level = get_required_level(p + 1);
        let xp_needed = total_xp_to_level(req_level);
        let time_hours = xp_needed / (mult * 3600.0 * XP_PER_SECOND);
        total_time += time_hours;
        println!("P{}: {}h (cumulative: {}h)", p, time_hours, total_time);
    }
}
```

### Balance Checkpoints

Before shipping balance changes, verify:

- [ ] P1 achievable in 30-60 min
- [ ] P10 achievable in 10-15 hours
- [ ] Stormbreaker requires meaningful fishing investment
- [ ] Haven bonuses feel impactful but not mandatory
- [ ] Minigame rewards are attractive but not required

---

## Common Tuning Scenarios

### "Prestige feels pointless"

**Symptom**: Players don't feel stronger after prestiging.

**Fixes**:
1. Increase prestige multiplier coefficient (0.5 → 0.6)
2. Increase attribute cap scaling (5 → 6 per rank)
3. Add more visible power indicators in UI

### "Game is too slow"

**Symptom**: Players quit before P5.

**Fixes**:
1. Lower XP curve exponent (1.5 → 1.4)
2. Increase kill XP range (200-400 → 250-500)
3. Lower early prestige level requirements

### "Game is too fast"

**Symptom**: Players hit endgame in days, not weeks.

**Fixes**:
1. Raise XP curve exponent (1.5 → 1.6)
2. Raise prestige level requirements
3. Lower prestige multiplier coefficient

### "Items don't matter"

**Symptom**: Players ignore equipment.

**Fixes**:
1. Increase affix value ranges
2. Lower base stats, raise item contribution
3. Add more impactful affix types

### "Fishing takes forever"

**Symptom**: Storm Leviathan feels impossibly far.

**Fixes**:
1. Lower fish-per-rank requirements in upper tiers
2. Increase FishingDock bonuses
3. Add more fishing rank rewards from challenges

### "Haven is too expensive"

**Symptom**: Players hoard prestige ranks, never build.

**Fixes**:
1. Lower tier costs (especially T1)
2. Increase bonus values to make investment obvious
3. Add "preview" of bonuses before purchase

---

## Appendix: Current Constants

For reference, key balance constants as of this writing:

```rust
// XP
const BASE_XP_PER_TICK: f64 = 1.0;
const COMBAT_XP_MIN_TICKS: u32 = 200;
const COMBAT_XP_MAX_TICKS: u32 = 400;

// Combat
const ATTACK_INTERVAL_SECONDS: f64 = 1.5;
const HP_REGEN_DURATION_SECONDS: f64 = 2.5;
const BASE_CRIT_CHANCE_PERCENT: u32 = 5;  // 5% base crit
const BASE_CRIT_MULTIPLIER: f64 = 2.0;

// Items
const ITEM_DROP_BASE_CHANCE: f64 = 0.15;
const ITEM_DROP_MAX_CHANCE: f64 = 0.25;
const DROP_PRESTIGE_BONUS: f64 = 0.01;

// Offline
const OFFLINE_MULTIPLIER: f64 = 0.25;
const MAX_OFFLINE_SECONDS: i64 = 604800; // 7 days

// Discovery
const DUNGEON_DISCOVERY_CHANCE: f64 = 0.02;
const FISHING_DISCOVERY_CHANCE: f64 = 0.05;
const CHALLENGE_DISCOVERY_CHANCE: f64 = 0.000014;

// Fishing
const BASE_MAX_FISHING_RANK: u32 = 30;
const MAX_FISHING_RANK: u32 = 40;
```

---

*Balance is never "done" — it's an ongoing conversation between designer intent and player experience. When in doubt, playtest.*
