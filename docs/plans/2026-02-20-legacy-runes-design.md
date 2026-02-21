# Storm Sigils — Stormglass Exchange Option #3

**Date:** 2026-02-20
**Status:** Approved

## Overview

Storm Sigils are character-level persistent sigil slots that survive prestige resets. Players spend Stormglass to unlock slots, inscribe sigils, and reroll for better values. Each inscribe/reroll presents 3 randomly rolled sigils and the player picks one. Rolls are graded S through F based on percentile within their range.

## Slot Unlock Costs (2x exponential from 25k)

| Slot | Cost | Cumulative |
|------|------|-----------|
| 1 | 25,000 SG | 25,000 |
| 2 | 50,000 SG | 75,000 |
| 3 | 100,000 SG | 175,000 |
| 4 | 200,000 SG | 375,000 |
| 5 | 400,000 SG | 775,000 |

Slot 1 is available immediately once Stormglass is discovered (P15+). Only the next unlockable slot's cost is shown; locked slots beyond that show only a lock icon.

## Inscribe & Reroll

- **Cost:** 25,000 SG (both inscribe and reroll)
- **Inscribe:** Fill an empty slot. Pay 25,000 SG, 3 random sigils are rolled, pick 1.
- **Reroll:** Replace an existing sigil. Pay 25,000 SG, current sigil is destroyed, 3 new sigils are rolled, pick 1.
- **Forfeit:** Esc on the pick-1-of-3 screen is allowed with confirmation. SG is lost. Matches Invoke Trial forfeit pattern.

## Sigil Effect Pool

Each sigil rolls one of these effects at a random value within the range. All 3 presented sigils roll independently (can be same or different effects). All effects are percentage-based to remain relevant at all prestige levels.

| Effect | Range | Sigil Name | Example |
|--------|-------|-----------|---------|
| +% XP | 5-25% | Sigil of Wisdom | +18.2% XP |
| +% Damage | 3-15% | Sigil of Fury | +9.7% Damage |
| +% Damage Reduction | 1-5% | Sigil of the Bulwark | +3.4% DR |
| +% Crit Chance | 2-8% | Sigil of Precision | +5.8% Crit |
| +% Drop Rate | 2-10% | Sigil of Fortune | +6.1% Drop Rate |
| +% Max HP | 3-15% | Sigil of Vitality | +11.3% HP |
| +% Fishing Speed | 5-25% | Sigil of the Tide | +17.4% Fishing Speed |
| +% Offline XP | 5-20% | Sigil of Echoes | +14.7% Offline XP |
| +% Attack Speed | 2-10% | Sigil of Swiftness | +7.1% ASPD |
| +% Double Strike | 1-5% | Sigil of the Twin Strike | +3.6% Double Strike |
| -% Regen Delay | 2-10% | Sigil of Renewal | +6.8% Regen Speed |

Duplicate effects across slots are allowed (e.g., two XP sigils).

### Stacking Safety Analysis

All ranges are tuned so that 5x S+ of the same effect doesn't break game balance:

| Effect | 5x S+ Total | Context | Safe? |
|--------|-------------|---------|-------|
| +% XP | ~125% | Training Yard T3 = +30%, WIS modifier adds +5%/pt | Yes — large but XP is non-competitive |
| +% Damage | ~75% | Haven Armory T3 = +25%, Megingjord = +150% | Yes — meaningful but doesn't approach god item |
| +% DR | ~25% | Asprika Divine Bulwark = 30% | Yes — doesn't surpass god item unique |
| +% Crit | ~40% | Prestige cap = 15%, Watchtower T3 = +20% | Yes — high but crit is 2x damage (not more) |
| +% Drop Rate | ~50% | Trophy Hall T3 = +15%, mob cap at 25% | Yes — generous but drops are already common |
| +% Max HP | ~75% | Prestige flat HP = ~82 at P15 | Yes — large but defensive, doesn't trivialize |
| +% Fishing Speed | ~125% | Garden T3 = -40% | Yes — QoL only, no combat impact |
| +% Offline XP | ~100% | Hearthstone T3 = +100%, base rate 25% | Yes — caps at 75% of online rate (25% * 3.0) |
| +% ASPD | ~50% | Equipment affixes add 10-50%, Sleipnir = +100% | Yes — doesn't approach god item |
| +% Double Strike | ~25% | War Room T3 = +35% | Yes — total 60% is high but earned |
| -% Regen Delay | ~41% effective | Bedroom T3 = -50%, Sleipnir = -50% | Yes — multiplicative, can't eliminate regen |

## Tier Grading

Each roll is graded based on its percentile within the effect's range. Each tier has +/base/- sub-tiers, with "+" being harder to hit (aspirational).

| Sub-tier | Percentile | Width | Color |
|----------|-----------|-------|-------|
| S+ | 98.5-100% | 1.5% | Gold |
| S | 96-98.5% | 2.5% | Gold |
| S- | 95-96% | 1% | Gold |
| A+ | 92-95% | 3% | Green |
| A | 85-92% | 7% | Green |
| A- | 80-85% | 5% | Green |
| B+ | 75-80% | 5% | Cyan |
| B | 67-75% | 8% | Cyan |
| B- | 60-67% | 7% | Cyan |
| C+ | 55-60% | 5% | White |
| C | 47-55% | 8% | White |
| C- | 40-47% | 7% | White |
| D+ | 35-40% | 5% | Gray |
| D | 27-35% | 8% | Gray |
| D- | 20-27% | 7% | Gray |
| E+ | 17-20% | 3% | Dark Gray |
| E | 13-17% | 4% | Dark Gray |
| E- | 10-13% | 3% | Dark Gray |
| F+ | 7-10% | 3% | Red |
| F | 3-7% | 4% | Red |
| F- | 0-3% | 3% | Red |

S+ is 1.5% of all rolls — a true trophy.

**Sub-tier styling:** Terminal modifiers differentiate +/base/- within each tier color:

| Variant | Modifier |
|---------|----------|
| + | BOLD |
| base | (none) |
| - | DIM |

For example, S+ is Bold Gold, A is Green, B- is Dim Cyan. Stacks with the tier color for immediate readability.

Tier is shown on the pick-1-of-3 screen, result screen, and sigils list.

## Persistence

- Character-level (not account-level)
- Persists through prestige resets
- Stored alongside character save data
- Sigil bonuses applied via explicit parameter injection (like Haven bonuses)

## UI Flow

### Exchange Menu (3rd item)

```
  Invoke Trial                    3,000 SG
  Chrono Surge                        >>>
> Storm Sigils                        >>>

Inscribe permanent sigils that persist
through prestige.
```

### Sigils Screen

```
╭─── Storm Sigils  [12,500 SG] ───────────────╮
│                                              │
│  Sigils of power etched into your soul.      │
│                                              │
│  > Slot 1:  (empty)                          │
│    Slot 2:  locked                           │
│    Slot 3:  locked                           │
│    Slot 4:  locked                           │
│    Slot 5:  locked                           │
│                                              │
│  Next unlock: 50,000 SG                      │
│                                              │
│  Select an empty slot to inscribe a sigil.   │
│                                              │
│  [up/down] Select  [Enter] Action  [Esc] Back│
╰──────────────────────────────────────────────╯
```

With inscribed sigils:

```
│  > Slot 1:  Sigil of Wisdom  +24.1% XP   S  │
│    Slot 2:  Sigil of Fury    +5.8% Damage D  │
│    Slot 3:  (empty)                          │
│    Slot 4:  locked                           │
│    Slot 5:  locked                           │
│                                              │
│  Next unlock: 200,000 SG                     │
│  Reroll: 25,000 SG                           │
```

### Action Flows

**Locked slot (next unlockable only):**
- Enter → Unlock confirmation (shows balance/cost/after) → Y to unlock → slot becomes empty

**Empty slot:**
- Enter → Inscribe confirmation (25,000 SG, shows balance/cost/after) → Y → pick 1 of 3 sigils → result screen

**Inscribed slot:**
- Enter → Reroll confirmation (25,000 SG, shows current sigil being destroyed, balance/cost/after) → Y → pick 1 of 3 sigils → result screen

**Pick 1 of 3 screen:**

```
╭─── Choose a Sigil ──────────────────────────╮
│                                              │
│  The storm fractures. Three sigils emerge.   │
│                                              │
│  > +24.1% XP          S    (range: 5-25%)    │
│    +5.8% Damage        D    (range: 3-15%)   │
│    +5.4% Crit Chance   B    (range: 2-8%)    │
│                                              │
│  [up/down] Select [Enter] Inscribe [Esc] Forfeit│
╰──────────────────────────────────────────────╯
```

**Forfeit confirmation (Esc on pick screen):**

```
╭─── Abandon Sigil? ──────────────────────────╮
│                                              │
│  25,000 SG already spent.                    │
│  The Stormglass cannot be reclaimed.         │
│                                              │
│  [Enter] Leave  [Esc] Stay                   │
╰──────────────────────────────────────────────╯
```

## Stats Panel Display

Inscribed sigils appear as a bordered sub-panel inside the Equipment section of the main character stats panel (L/XL tiers). Only renders when at least one sigil is inscribed. Each sigil shows its name, effect value, and tier grade on its own line.

```
╭─ Equipment ─────────────────────────────────────────────╮
│ Weapon  Stormcleaver              Epic  T7  Z8  ⚡342   │
│  Armor  Iron Plate                Rare  T5  Z6  ⚡218   │
│ Helmet  Wolf Helm                Magic  T3  Z4  ⚡124   │
│ Gloves  Gauntlets of Might        Rare  T4  Z5  ⚡186   │
│  Boots  Sleipnir                   God  T9  Z10 ⚡999   │
│ Amulet  Jade Pendant             Magic  T2  Z3  ⚡ 89   │
│   Ring  Storm Band                Rare  T6  Z7  ⚡267   │
│                                                         │
│ ╭─ Storm Sigils (3/5) ──────────────────────────────╮   │
│ │  Sigil of Wisdom         +24.1% XP          S      │   │
│ │  Sigil of Fury            +5.8% Damage      D      │   │
│ │  Sigil of Vitality        +11.3% HP         B      │   │
│ ╰────────────────────────────────────────────────────╯   │
╰─────────────────────────────────────────────────────────╯
```

- Sigil name left-aligned, effect value and tier grade right-aligned
- Tier grade letter colored per tier grading table (S=Gold, A=Green, etc.)
- Sub-panel title shows inscribed/unlocked count
- Hidden when no sigils inscribed; equipment panel unchanged for players without sigils
- Equipment section uses `Min(0)` flex space so sigils grow naturally within it

## Integration Points

- **Stormglass Exchange:** 3rd menu item, `EXCHANGE_MENU_ITEMS` bumped to 3
- **ExchangePhase:** New variants for sigils screen, unlock confirm, inscribe confirm, reroll confirm, pick-1-of-3, forfeit confirm, result
- **GameState:** New `storm_sigils` field (Vec of Option<Sigil>, max 5, plus `slots_unlocked: u8`)
- **Character persistence:** Serialized/deserialized with character save
- **Prestige:** `storm_sigils` field preserved during prestige reset
- **Derived stats / combat / fishing:** Sigil bonuses injected as parameters (like Haven bonuses)
- **UI:** New render functions for each phase in `stormglass_scene.rs`
- **Input:** New handlers for each phase in `stormglass_input.rs`

### Bonus Application Points

| Sigil Effect | Injection Point | Mechanic |
|-------------|----------------|----------|
| +% XP | `core/xp.rs` | Multiply XP gain |
| +% Damage | `combat/player_attack.rs` | Multiply base damage (after Haven, before prestige flat) |
| +% DR | `combat/enemy_attack.rs` | Multiply damage taken (after defense subtraction, alongside Divine Bulwark) |
| +% Crit Chance | `combat/player_attack.rs` | Add to crit chance roll |
| +% Drop Rate | `items/drops.rs` | Add to mob drop chance |
| +% Max HP | `core/tick.rs` | Multiply max HP (alongside prestige flat HP) |
| +% Fishing Speed | `fishing/logic.rs` | Reduce fishing timers (alongside Garden) |
| +% Offline XP | `core/offline.rs` | Multiply offline XP rate (alongside Hearthstone) |
| +% ASPD | `combat/orchestration.rs` | Add to attack speed multiplier (alongside equipment + Sleipnir) |
| +% Double Strike | `combat/player_attack.rs` | Add to double strike chance (alongside War Room) |
| -% Regen Delay | `combat/regen.rs` | Multiplicative reduction (alongside Bedroom + Sleipnir) |
