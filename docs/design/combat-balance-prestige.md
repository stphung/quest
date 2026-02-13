# Prestige Combat Benefits Design

Issue: #123 — Prestige provides no direct combat benefit; 65% of P0 players stuck in Zone 1.

## 1. Analysis of Current Prestige Benefits Gap

### What Prestige Currently Provides

| Benefit | Formula | Combat Impact |
|---------|---------|---------------|
| XP multiplier | `1.0 + 0.5 * rank^0.7` | Indirect — faster leveling, not combat power |
| Attribute cap | `20 + 5 * rank` | **Negated** — enemies scale with `player_max_hp` |
| Item drop rate | `+1% per rank (cap 25%)` | Indirect — more gear, not combat power |
| Mob rarity bonus | `+1% per rank (cap 10%)` | Indirect — better gear quality |

### The Core Problem: Enemy HP Scaling Cancels Attribute Caps

Enemy generation in `combat/types.rs:63-77` uses `player_max_hp` as the base:

```
Enemy HP   = player_max_hp * random(0.8..1.2) * zone_multiplier
Enemy DMG  = player_max_hp / 7.0 * random(0.8..1.2) * zone_multiplier
```

When a P5 player has CON 35 (cap 45), their max HP is 175. But enemies are generated at 80-120% of 175, keeping fights at the same relative difficulty as a P0 player with CON 20 and max HP 100. **Higher attribute caps give bigger numbers but identical fight outcomes.**

This means prestige rank 1-9 provides:
- Faster XP gain (useful)
- Higher numbers that are cosmetically larger (not useful)
- No ability to kill enemies faster or survive longer in relative terms

### Haven Cannot Fill the Gap

Haven provides real combat bonuses (Armory: +25% damage at T3, Watchtower: +20% crit, War Room: +35% double strike). However:
- Haven requires P10+ just to discover
- Building Haven rooms costs prestige ranks (competing with the ranks needed to advance)
- P0-P9 players have zero access to these combat bonuses
- This creates a dead zone where prestige 1-9 feels unrewarding in combat

### Player Impact

- **P0 players (65% stuck in Zone 1)**: No combat tools beyond raw attribute points, which enemies match
- **P1-P4 players**: XP multiplier helps them level faster, but fights against Zone 2 bosses are the same difficulty
- **P5-P9 players**: Unlocked Zones 3-4 by rank requirement, but combat scaling still neutralizes their stats
- **P10+ players**: First real combat boost when Haven is discovered, but it takes significant prestige rank investment

## 2. Proposed Prestige Combat Bonuses

### Design Principles

1. **Prestige should break the HP-scaling treadmill**: Bonuses that are not derived from attributes and thus not reflected in `player_max_hp`
2. **Additive with Haven, not multiplicative**: Prestige bonuses and Haven bonuses stack additively to prevent runaway scaling
3. **Diminishing returns**: Same formula shape as XP multiplier to prevent late-game trivialization
4. **Immediately tangible**: Even P1 should feel a combat difference

### Proposed Bonuses Per Prestige Rank

We introduce **four** prestige combat bonuses, stored in a new `PrestigeCombatBonuses` struct. These are computed from prestige rank alone, not from attributes, so they bypass enemy HP scaling.

#### A. Prestige Damage Bonus (flat)

- **Formula**: `floor(2.0 * rank^0.6)` flat damage added after all multipliers
- **Rationale**: Flat damage is not part of `DerivedStats` and not reflected in `player_max_hp`, so enemies cannot scale against it. It represents "veteran battle instinct."
- **Values**:

| Rank | Flat Damage | Notes |
|------|-------------|-------|
| P0 | +0 | Baseline |
| P1 | +2 | Noticeable on 10-damage base hits |
| P2 | +3 | |
| P3 | +3 | |
| P5 | +4 | |
| P10 | +7 | Significant vs. Zone 5-6 enemies |
| P15 | +9 | |
| P20 | +11 | Meaningful but not dominant |

This bonus is applied as a final additive step in damage calculation, after Haven's `damage_percent` multiplier.

#### B. Prestige Defense Bonus (flat)

- **Formula**: `floor(1.0 * rank^0.55)` flat defense added after derived defense
- **Rationale**: Reduces incoming damage by a fixed amount that is independent of `player_max_hp`. Represents "hardened veteran resilience."
- **Values**:

| Rank | Flat Defense | Notes |
|------|-------------|-------|
| P0 | +0 | Baseline |
| P1 | +1 | Reduces mob damage by 1 |
| P2 | +1 | |
| P3 | +1 | |
| P5 | +2 | |
| P10 | +3 | Blocks ~20% of early zone mob damage |
| P15 | +4 | |
| P20 | +5 | |

Applied after DEX-based defense in `update_combat()`, subtracted from enemy damage alongside existing defense.

#### C. Prestige Crit Chance Bonus (percentage)

- **Formula**: `min(rank * 0.5, 10.0)` percentage points of crit chance
- **Rationale**: Percentage-based but capped at 10% to not overshadow Haven Watchtower (+20% at T3). Represents "experienced combatant precision."
- **Values**:

| Rank | Crit Bonus | Total with Haven T3 |
|------|------------|---------------------|
| P0 | +0% | 0% bonus |
| P1 | +0.5% | +0.5% |
| P5 | +2.5% | +2.5% |
| P10 | +5% | +25% (with Watchtower T3) |
| P15 | +7.5% | +27.5% |
| P20 | +10% (cap) | +30% |

Stacks additively with base crit (from DEX), equipment affixes, and Haven Watchtower.

#### D. Prestige HP Bonus (flat)

- **Formula**: `floor(5.0 * rank^0.5)` flat HP added to `max_hp`
- **Rationale**: Extra HP that is added AFTER enemies are generated (applied during combat, not during `DerivedStats` calculation). This is a survivability cushion that enemies do not scale against.
- **Important implementation note**: This bonus must NOT be included in `DerivedStats.max_hp` because that is what enemy generation reads. Instead, it is added as a separate bonus in the combat system, similar to how Haven bonuses are injected.
- **Values**:

| Rank | Flat HP | Notes |
|------|---------|-------|
| P0 | +0 | Baseline |
| P1 | +5 | One extra hit vs. early enemies |
| P2 | +7 | |
| P5 | +11 | |
| P10 | +15 | ~15% extra for P0-cap characters |
| P15 | +19 | |
| P20 | +22 | Meaningful buffer, not dominant |

**Critical constraint**: This HP bonus must be applied in combat only, not in `DerivedStats::calculate_derived_stats()`, to prevent enemies from scaling against it. The UI should still display the combined HP total.

## 3. Interaction with Haven Bonuses

### Stacking Rules

All prestige combat bonuses stack **additively** with Haven bonuses — never multiplicatively. This prevents exponential power curves.

| Stat | Prestige Source | Haven Source | Stacking |
|------|----------------|-------------|----------|
| Damage | Flat bonus (post-multiplier) | Armory % multiplier | Prestige flat added after Haven % applied to base |
| Defense | Flat bonus | None (Haven has no defense) | No overlap |
| Crit Chance | Flat % | Watchtower flat % | Sum of both added to base crit |
| HP | Flat bonus (combat-only) | None (Haven has no HP bonus) | No overlap |
| Double Strike | None | War Room % | No overlap |
| HP Regen | None | Alchemy Lab %, Bedroom delay | No overlap |

### Damage Calculation Order (Updated)

```
1. Calculate base damage from DerivedStats (STR/INT + equipment)
2. Apply Haven Armory multiplier: damage *= (1 + armory_percent / 100)
3. Apply prestige flat damage: damage += prestige_damage_bonus
4. Roll crit: crit_chance = base_crit + haven_crit + prestige_crit
5. If crit: damage *= crit_multiplier
6. Apply to enemy
```

Note: Prestige flat damage is added BEFORE crit multiplier so crits amplify it. This is intentional — it makes crits feel more impactful for prestiged players.

### Defense Calculation Order (Updated)

```
1. Calculate base defense from DerivedStats (DEX + equipment)
2. Add prestige flat defense: total_defense = base_defense + prestige_defense
3. Enemy damage after defense: max(1, enemy.damage - total_defense)
```

### No Double-Dipping Guarantee

- Haven bonuses are passed via `HavenCombatBonuses` struct (percentage-based)
- Prestige bonuses are passed via a new `PrestigeCombatBonuses` struct (flat values)
- They operate on different stages of the damage pipeline
- A player cannot get the same bonus from both sources

## 4. Expected Combat Effectiveness by Prestige Tier

### Baseline Scenario: P0, Level 10, Base Attributes (all 10), No Equipment

- Player max HP: 50
- Player damage: 10 (5 phys + 5 magic)
- Player defense: 0
- Player crit: 5%
- Enemy HP: ~40-60 (80-120% of 50)
- Enemy damage: ~6-8 (50/7 * variance)
- **Fight duration**: ~6-8 attacks each, ~9-12 seconds
- **Win rate**: ~60-70% (player often dies to Zone 1 subzone 3 boss)

### P1 (Bronze): +2 DMG, +1 DEF, +0.5% Crit, +5 HP

- Player effective damage: 12 (+20% vs baseline, significant)
- Player effective HP: 55 (enemies still see 50, so generated at P0 levels)
- Player defense: 1 (blocks ~15% of mob damage)
- **Fight duration**: ~5-7 attacks, ~7-10 seconds
- **Win rate**: ~80-85% vs Zone 1 (comfortable progression)
- **Impact**: P1 players can reliably clear Zone 1 bosses

### P5 (Diamond): +4 DMG, +2 DEF, +2.5% Crit, +11 HP

At P5, attributes can reach cap 45, but enemies scale to match. The flat bonuses provide the actual edge:

- Player effective damage: base ~39 + 4 flat + Haven Armory if available = ~43
- Player effective defense: base ~17 + 2 flat = 19
- Additional 11 HP hidden from enemy generation
- **Fight efficiency**: ~15-20% faster kills vs. equal-level content
- **Impact**: Zones 1-4 are comfortable; Zone 5+ (P10-gated) is still challenging

### P10 (Celestial): +7 DMG, +3 DEF, +5% Crit, +15 HP + Haven Access

P10 is the breakpoint where Haven becomes available, layering percentage bonuses on top:

| Stat | Prestige Alone | + Haven T1 | + Haven T3 |
|------|---------------|------------|------------|
| Damage bonus | +7 flat | +7 flat, +5% base | +7 flat, +25% base |
| Crit chance | +5% | +10% | +25% |
| Double strike | 0% | +10% | +35% |
| Defense | +3 flat | +3 flat | +3 flat |
| HP bonus | +15 flat | +15 flat | +15 flat |

- **Without Haven**: 15-20% combat advantage from prestige alone
- **With Haven T1**: 25-30% combat advantage
- **With Haven T3**: 50-70% combat advantage (the intended "fully invested" power level)

### P15 (Transcendent): +9 DMG, +4 DEF, +7.5% Crit, +19 HP

- Prestige combat bonuses provide a steady ~20% advantage at any gear/attribute level
- Haven bonuses (if invested) add another 50-70%
- Zones 7-8 are the target content here
- Total combat effectiveness: ~80-100% above what attribute scaling alone provides

### P20 (Eternal): +11 DMG, +5 DEF, +10% Crit (cap), +22 HP

- Prestige bonuses are approaching their diminishing returns ceiling
- The flat bonuses become proportionally less impactful as base stats grow
- Haven bonuses are the dominant factor at this level
- Focus shifts to Zone 9-10 content and Stormbreaker path
- Total combat effectiveness: ~90-120% above base (mostly from Haven)

## 5. How This Changes the "Stuck in Zone 1" Problem

### Current State (P0, No Combat Benefits)

```
Zone 1 Subzone 1: Win ~80% of fights (manageable)
Zone 1 Subzone 2: Win ~65% of fights (struggling)
Zone 1 Subzone 3 Boss: Win ~40% of fights (frequently stuck)
-> 65% of P0 players fail to progress past Zone 1
```

### With Prestige Combat Benefits (P1)

```
Zone 1 Subzone 1: Win ~95% of fights (+2 flat DMG, +1 DEF, +5 HP)
Zone 1 Subzone 2: Win ~85% of fights
Zone 1 Subzone 3 Boss: Win ~65% of fights (still challenging, not trivial)
Zone 2 Subzone 1: Win ~75% of fights (healthy progression)
-> Estimated 80%+ of P1 players clear Zone 1
```

### Key Insight: P0 Remains the "Tutorial"

P0 is intentionally harder. The game communicates through difficulty that prestige is the intended progression mechanic. Players who are stuck should prestige (requires level 10, which is achievable even with losses). The first prestige gives:

1. +50% XP multiplier (reach level 10 much faster on the next cycle)
2. **+2 flat damage** (10-20% more effective immediately)
3. **+1 flat defense** (blocks ~15% of mob damage)
4. **+5 flat HP** (one extra hit of survivability enemies cannot see)
5. Attribute cap 25 (minor benefit but reaches higher stats faster)

This creates a clear "aha moment" — after first prestige, Zone 1 feels noticeably easier.

### Projected Progression After Fix

| Prestige | Expected Zone Progress | Bottleneck |
|----------|----------------------|------------|
| P0 | Zone 1 (tutorial grind) | Boss difficulty, intentional |
| P1-2 | Zone 1-2 comfortably | XP curve for Zone 2 bosses |
| P3-4 | Zone 1-2 cleared, waiting for P5 | Prestige gate to Zone 3 |
| P5-9 | Zones 3-4 | Haven discovery wait (P10) |
| P10-14 | Zones 5-6, Haven building | PR investment vs. zone gates |
| P15-19 | Zones 7-8, Haven near-max | Level requirements get steep |
| P20+ | Zones 9-10, Stormbreaker path | End-game content gating |

## 6. Prestige Tier Unlock Flow and Power Curve

### Power Curve Visualization

```
Combat
Power
  ^
  |                                              P20 + Haven T3
  |                                         ****
  |                                     ****
  |                                 ****     <- Haven bonuses dominate
  |                            ****
  |                       ****               P10 + Haven discovered
  |                  *****
  |             *****                        <- Prestige flat bonuses
  |        *****                               provide the edge
  |   *****
  |***                                       P1 breakpoint
  |*                                         P0 baseline
  +----------------------------------------> Prestige Rank
  0    5    10    15    20
```

The curve has three phases:
1. **P0-P1**: Steep jump — first prestige breaks the "stuck" state
2. **P1-P10**: Gradual improvement — each prestige feels incrementally better
3. **P10-P20**: Accelerating returns — Haven bonuses layer on top of prestige bonuses

### Implementation Struct

```rust
/// Combat bonuses from prestige rank (independent of attributes/Haven)
pub struct PrestigeCombatBonuses {
    pub flat_damage: u32,       // Added after Haven % multiplier, before crit
    pub flat_defense: u32,      // Added to DEX-based defense
    pub crit_chance: f64,       // Percentage points added to crit chance
    pub flat_hp: u32,           // Added to combat HP, NOT to DerivedStats.max_hp
}

impl PrestigeCombatBonuses {
    pub fn from_rank(rank: u32) -> Self {
        Self {
            flat_damage: (2.0 * (rank as f64).powf(0.6)).floor() as u32,
            flat_defense: (1.0 * (rank as f64).powf(0.55)).floor() as u32,
            crit_chance: (rank as f64 * 0.5).min(10.0),
            flat_hp: (5.0 * (rank as f64).powf(0.5)).floor() as u32,
        }
    }
}
```

### Integration Points

1. **`combat/logic.rs`**: `update_combat()` accepts `PrestigeCombatBonuses` alongside existing `HavenCombatBonuses`
2. **`character/prestige.rs`**: New `prestige_combat_bonuses(rank) -> PrestigeCombatBonuses` function
3. **`core/tick.rs`**: Compute prestige bonuses at start of tick, pass to combat
4. **`combat/types.rs`**: Enemy generation remains unchanged (uses `DerivedStats.max_hp` only)
5. **`ui/stats_panel.rs`**: Display prestige bonuses separately (e.g., "+7 DMG from prestige")
6. **`ui/prestige_confirm.rs`**: Show the combat bonuses gained from next prestige

### Constants to Add (`core/constants.rs`)

```rust
// Prestige combat bonus formulas
pub const PRESTIGE_FLAT_DAMAGE_FACTOR: f64 = 2.0;
pub const PRESTIGE_FLAT_DAMAGE_EXPONENT: f64 = 0.6;
pub const PRESTIGE_FLAT_DEFENSE_FACTOR: f64 = 1.0;
pub const PRESTIGE_FLAT_DEFENSE_EXPONENT: f64 = 0.55;
pub const PRESTIGE_CRIT_PER_RANK: f64 = 0.5;
pub const PRESTIGE_CRIT_CAP: f64 = 10.0;
pub const PRESTIGE_FLAT_HP_FACTOR: f64 = 5.0;
pub const PRESTIGE_FLAT_HP_EXPONENT: f64 = 0.5;
```

## Summary of Changes

| Change | Files Affected | Risk |
|--------|---------------|------|
| New `PrestigeCombatBonuses` struct | `character/prestige.rs` | Low — new code |
| New constants | `core/constants.rs` | Low — additive |
| Pass bonuses to `update_combat()` | `combat/logic.rs`, `core/tick.rs` | Medium — modifies combat pipeline |
| Apply flat damage in attack phase | `combat/logic.rs` | Medium — changes damage numbers |
| Apply flat defense in defense phase | `combat/logic.rs` | Medium — changes incoming damage |
| Apply flat HP in combat state | `combat/logic.rs` or `core/tick.rs` | High — must NOT leak into enemy gen |
| Apply crit bonus | `combat/logic.rs` | Low — already sums crit sources |
| UI display | `ui/stats_panel.rs`, `ui/prestige_confirm.rs` | Low — display only |
| No change to enemy generation | `combat/types.rs` | None — intentionally unchanged |
