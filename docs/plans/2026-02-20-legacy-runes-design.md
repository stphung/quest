# Legacy Runes — Stormglass Exchange Option #3

**Date:** 2026-02-20
**Status:** Approved

## Overview

Legacy Runes are character-level persistent rune slots that survive prestige resets. Players spend Stormglass to unlock slots, inscribe runes, and reroll for better values. Each inscribe/reroll presents 3 randomly rolled runes and the player picks one. Rolls are graded S through F based on percentile within their range.

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
- **Inscribe:** Fill an empty slot. Pay 25,000 SG, 3 random runes are rolled, pick 1.
- **Reroll:** Replace an existing rune. Pay 25,000 SG, current rune is destroyed, 3 new runes are rolled, pick 1.
- **Forfeit:** Esc on the pick-1-of-3 screen is allowed with confirmation. SG is lost. Matches Invoke Trial forfeit pattern.

## Rune Effect Pool

Each rune rolls one of these effects at a random value within the range. All 3 presented runes roll independently (can be same or different effects).

| Effect | Range | Example |
|--------|-------|---------|
| +% XP | 1-5% | +3.2% XP |
| +Flat Damage | 2-10 | +7 Damage |
| +Flat Defense | 1-8 | +4 Defense |
| +% Crit Chance | 0.5-3% | +1.8% Crit |
| +% Drop Rate | 0.5-3% | +2.1% Drop Rate |
| +Flat HP | 5-25 | +18 HP |
| +% Fishing Speed | 2-8% | +5.4% Fishing Speed |
| +% Offline XP | 2-10% | +6.7% Offline XP |

Duplicate effects across slots are allowed (e.g., two XP runes).

## Tier Grading

Each roll is graded based on its percentile within the effect's range:

| Tier | Percentile | Color |
|------|-----------|-------|
| S | 95-100% | Gold |
| A | 80-95% | Green |
| B | 60-80% | Cyan |
| C | 40-60% | White |
| D | 20-40% | Gray |
| E | 10-20% | Dark Gray |
| F | 0-10% | Red |

Tier is shown on the pick-1-of-3 screen, result screen, and runes list.

## Persistence

- Character-level (not account-level)
- Persists through prestige resets
- Stored alongside character save data
- Rune bonuses applied via explicit parameter injection (like Haven bonuses)

## UI Flow

### Exchange Menu (3rd item)

```
  Invoke Trial                    3,000 SG
  Chrono Surge                        >>>
> Legacy Runes                        >>>

Inscribe permanent runes that persist
through prestige.
```

### Runes Screen

```
╭─── Legacy Runes  [12,500 SG] ───────────────╮
│                                              │
│  Runes of power etched into your soul.       │
│                                              │
│  > Slot 1:  (empty)                          │
│    Slot 2:  locked                           │
│    Slot 3:  locked                           │
│    Slot 4:  locked                           │
│    Slot 5:  locked                           │
│                                              │
│  Next unlock: 50,000 SG                      │
│                                              │
│  Select an empty slot to inscribe a rune.    │
│                                              │
│  [up/down] Select  [Enter] Action  [Esc] Back│
╰──────────────────────────────────────────────╯
```

With inscribed runes:

```
│  > Slot 1:  +4.8% XP              S         │
│    Slot 2:  +4 Damage             D         │
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
- Enter → Inscribe confirmation (25,000 SG, shows balance/cost/after) → Y → pick 1 of 3 runes → result screen

**Inscribed slot:**
- Enter → Reroll confirmation (25,000 SG, shows current rune being destroyed, balance/cost/after) → Y → pick 1 of 3 runes → result screen

**Pick 1 of 3 screen:**

```
╭─── Choose a Rune ───────────────────────────╮
│                                              │
│  The storm fractures. Three runes emerge.    │
│                                              │
│  > +4.8% XP           S    (range: 1-5%)    │
│    +4 Damage           D    (range: 2-10)    │
│    +2.1% Crit Chance   B    (range: 0.5-3%)  │
│                                              │
│  [up/down] Select [Enter] Inscribe [Esc] Forfeit│
╰──────────────────────────────────────────────╯
```

**Forfeit confirmation (Esc on pick screen):**

```
╭─── Abandon Runes? ──────────────────────────╮
│                                              │
│  25,000 SG already spent.                    │
│  The Stormglass cannot be reclaimed.         │
│                                              │
│  [Enter] Leave  [Esc] Stay                   │
╰──────────────────────────────────────────────╯
```

## Integration Points

- **Stormglass Exchange:** 3rd menu item, `EXCHANGE_MENU_ITEMS` bumped to 3
- **ExchangePhase:** New variants for runes screen, unlock confirm, inscribe confirm, reroll confirm, pick-1-of-3, forfeit confirm, result
- **GameState:** New `legacy_runes` field (Vec of Option<Rune>, max 5, plus `slots_unlocked: u8`)
- **Character persistence:** Serialized/deserialized with character save
- **Prestige:** `legacy_runes` field preserved during prestige reset
- **Derived stats / combat / fishing:** Rune bonuses injected as parameters (like Haven bonuses)
- **UI:** New render functions for each phase in `stormglass_scene.rs`
- **Input:** New handlers for each phase in `stormglass_input.rs`
