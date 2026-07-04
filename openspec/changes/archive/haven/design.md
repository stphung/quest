> Backported design record. Sources: docs/archive/plans/2026-02-04-haven-design.md.

## 2026-02-04-haven-design.md

# Haven: Base Building System - Design Document

## Overview

The Haven is an account-level base building system presented as a skill tree. Players spend prestige ranks and fishing ranks to construct and upgrade rooms that provide permanent passive bonuses. The Haven persists across all prestige resets and benefits every character on the account.

## Unlock Conditions

- **Prestige gate:** Character must be P10+ (Celestial tier)
- **Discovery:** Independent RNG roll per tick (separate from challenge discovery). Chance scales with prestige rank so higher-prestige players find it faster:
  - `chance = 0.000014 + (prestige_rank - 10) * 0.000007`
  - P10: ~2 hours average, P12: ~1 hour, P15: ~34 min, P20: ~20 min
- **One-time:** Once any character discovers the Haven, it remains accessible account-wide to all characters regardless of prestige rank — even if the discovering character drops below P10 through spending

## Currency: Prestige Ranks & Fishing Ranks

Players spend actual prestige ranks and fishing ranks from the contributing character. Ranks decrease when spent. Characters can drop below P10 after discovery without losing Haven access.

Any character who has unlocked the Haven can contribute.

### Tier Costs (per room)

| Tier | Prestige Cost | Fishing Cost |
|------|--------------|-------------|
| Basic (T1) | 1 rank | 2 ranks |
| Improved (T2) | 3 ranks | 4 ranks |
| Grand (T3) | 5 ranks | 6 ranks |

**Total to max one room:** 9 prestige, 12 fishing.
**Total to max all 13 rooms:** 117 prestige, 156 fishing.

## Skill Tree Structure

The Haven is a skill tree with a single root (Hearthstone) that branches into two paths: Combat and QoL. Each path forks midway, and both forks must reach T1 before the capstone unlocks.

**Progression rule:** Building a room at T1 unlocks its children. Upgrading to T2/T3 can happen at any time in any order. Capstones require T1 of both parent rooms.

```
                  [Hearthstone]
                 /              \
         Combat Branch        QoL Branch
              |                    |
           Armory              Bedroom
           /    \              /    \
   Training    Trophy     Garden   Library
     Yard       Hall        |        |
       |          |     Fish Dock  Workshop
   Watchtower  Alchemy     \       /
       \       Lab          Vault
        \     /           (capstone)
        War Room
       (capstone)
```

## Room Definitions

All bonuses are percentages of base values (before other multipliers like prestige).

### Root

**Hearthstone** — The warm center of your Haven.
| Tier | Bonus |
|------|-------|
| T1 | +10% offline XP rate |
| T2 | +25% offline XP rate |
| T3 | +40% offline XP rate |

### Combat Branch

**Armory** — Your weapon collection strengthens all who visit.
| Tier | Bonus |
|------|-------|
| T1 | +5% damage |
| T2 | +10% damage |
| T3 | +18% damage |

**Training Yard** — Practice dummies and sparring targets.
| Tier | Bonus |
|------|-------|
| T1 | +5% XP gain |
| T2 | +12% XP gain |
| T3 | +20% XP gain |

**Trophy Hall** — Trophies from past victories attract fortune.
| Tier | Bonus |
|------|-------|
| T1 | +2% drop rate |
| T2 | +4% drop rate |
| T3 | +7% drop rate |

**Watchtower** — Sharpens your eye for weak points.
| Tier | Bonus |
|------|-------|
| T1 | +3% crit chance |
| T2 | +6% crit chance |
| T3 | +10% crit chance |

**Alchemy Lab** — Brews and tonics always simmering.
| Tier | Bonus |
|------|-------|
| T1 | +15% HP regen rate |
| T2 | +30% HP regen rate |
| T3 | +50% HP regen rate |

**War Room (Capstone)** — Tactical planning speeds your strikes. Requires Watchtower T1 and Alchemy Lab T1.
| Tier | Bonus |
|------|-------|
| T1 | -5% attack interval |
| T2 | -10% attack interval |
| T3 | -15% attack interval |

### QoL Branch

**Bedroom** — Rest well, fight well.
| Tier | Bonus |
|------|-------|
| T1 | -10% HP regen delay |
| T2 | -20% HP regen delay |
| T3 | -35% HP regen delay |

**Garden** — Patience cultivated here carries over.
| Tier | Bonus |
|------|-------|
| T1 | -10% fishing phase timers |
| T2 | -20% fishing phase timers |
| T3 | -30% fishing phase timers |

**Library** — Ancient tomes reveal hidden challenges.
| Tier | Bonus |
|------|-------|
| T1 | +20% challenge discovery rate |
| T2 | +40% challenge discovery rate |
| T3 | +65% challenge discovery rate |

**Fishing Dock** — A private spot to cast.
| Tier | Bonus |
|------|-------|
| T1 | +15% fishing rank XP |
| T2 | +30% fishing rank XP |
| T3 | +50% fishing rank XP |

**Workshop** — Better tools yield better finds.
| Tier | Bonus |
|------|-------|
| T1 | +5% item rarity |
| T2 | +10% item rarity |
| T3 | +18% item rarity |

**Vault (Capstone)** — Preserves treasured equipment through prestige resets. Requires Fishing Dock T1 and Workshop T1.
| Tier | Bonus |
|------|-------|
| T1 | 1 equipped item survives prestige |
| T2 | 2 equipped items survive prestige |
| T3 | 3 equipped items survive prestige |

## User Experience

### Discovery

During gameplay, a P10+ character triggers discovery via the existing RNG tick system (~2hr average). The game does **not** pause — ticks continue behind the modal.

A centered modal overlay appears:

```
╔══════════════════════════════════════════╗
║                                          ║
║           🏠  Haven Discovered           ║
║                                          ║
║   Through the treeline, you glimpse      ║
║   crumbling stone walls wrapped in       ║
║   ivy. A hearthstone chimney still       ║
║   stands, waiting for a fire.            ║
║                                          ║
║   Press [H] anytime to visit.            ║
║                                          ║
║           [Enter] to dismiss             ║
║                                          ║
╚══════════════════════════════════════════╝
```

Player presses `[Enter]` to dismiss. The Haven is now accessible account-wide via `[H]` from both gameplay and character select. No tutorial — the tree UI is self-explanatory on first visit.

### Character Select Screen

After discovery, the character select screen gains `[H] Haven` in its controls:

```
[Enter] Play  [R] Rename  [D] Delete  [N] New  [Q] Quit
[H] Haven
```

### Haven Skill Tree View

Pressing `[H]` opens the skill tree. Left side shows the tree, right side shows details for the selected room.

```
┌──────────────────────── Haven ──────────────────────────┐
│                                                          │
│         ┌────────────┐  ┌─ Hearthstone ★★· ──────────┐  │
│         │ Hearthstone│  │                              │  │
│         │    ★★·     │  │  "A warm fire keeps progress │  │
│         └─────┬──────┘  │   burning while away."       │  │
│          ╱         ╲    │                              │  │
│   ┌──────────┐  ┌─────  │  Current: +25% offline XP    │  │
│   │  Armory  │  │ Bedr  │                              │  │
│   │    ★··   │  │  ★··  │  Upgrade to Grand (T3)       │  │
│   └────┬─────┘  └────┬  │  Bonus: +40% offline XP      │  │
│     ╱     ╲       ╱   │ │                              │  │
│ ┌──────┐┌──────┐┌──── │ │  Cost:                       │  │
│ │Train ││Trophy││Gard  │ │    5 Prestige Ranks          │  │
│ │ · · ·││ · · ·││ · ·  │ │    6 Fishing Ranks           │  │
│ └──┬───┘└──┬───┘└──┬─ │ │                              │  │
│    :       :       :   │ │  Aldric has:                 │  │
│ ┌──────┐┌──────┐┌──── │ │    P12 (5 available)   ✓     │  │
│ │Watch ││Alchmy││F.Do  │ │    Fish Rank 18 (6 avl) ✓   │  │
│ │  🔒  ││  🔒  ││  🔒 │ │                              │  │
│ └──┬───┘└──┬───┘└──┬─ │ └──────────────────────────────┘  │
│     ╲     ╱       ╲   │                                   │
│   ┌──────────┐  ┌──────────┐                              │
│   │ War Room │  │  Vault   │                              │
│   │   🔒     │  │   🔒     │                              │
│   └──────────┘  └──────────┘                              │
│                                                           │
│ [↑/↓/←/→] Navigate  [Enter] Build/Upgrade  [Esc] Back    │
└───────────────────────────────────────────────────────────┘
```

**Visual states for tree nodes:**
- `★★·` — T2 of 3 (filled stars for completed tiers, dot for remaining)
- `· · ·` — Unbuilt but available (parent is T1+), pulsing highlight
- `🔒` — Locked (parent not yet built), dim/gray text
- Bright text for built rooms, dim for locked
- Solid `│` connector for unlocked paths, dotted `:` for locked

### Room Detail Panel States

**Built room (can upgrade):**
```
┌─ Hearthstone ★★· ────────────────────┐
│                                        │
│  "A warm fire keeps progress burning   │
│   while you're away."                  │
│                                        │
│  Current: +25% offline XP (T2)        │
│                                        │
│  Upgrade to Grand (T3)                │
│  Bonus: +40% offline XP               │
│                                        │
│  Cost:                                 │
│    5 Prestige Ranks                    │
│    6 Fishing Ranks                     │
│                                        │
│  Aldric has:                           │
│    P12 (5 available)   ✓              │
│    Fish Rank 18 (6 available) ✓       │
│                                        │
│  [Enter] Upgrade    [Esc] Back         │
└────────────────────────────────────────┘
```

**Unbuilt but available:**
```
┌─ Training Yard · · · ─────────────────┐
│                                        │
│  "Practice dummies and sparring        │
│   targets."                            │
│                                        │
│  Build Basic (T1)                     │
│  Bonus: +5% XP gain                   │
│                                        │
│  Cost:                                 │
│    1 Prestige Rank                     │
│    2 Fishing Ranks                     │
│                                        │
│  Aldric has:                           │
│    P12 (1 available)   ✓              │
│    Fish Rank 18 (2 available) ✓       │
│                                        │
│  [Enter] Build    [Esc] Back           │
└────────────────────────────────────────┘
```

**Insufficient ranks:**
```
┌─ Training Yard · · · ─────────────────┐
│                                        │
│  "Practice dummies and sparring        │
│   targets."                            │
│                                        │
│  Build Basic (T1)                     │
│  Bonus: +5% XP gain                   │
│                                        │
│  Cost:                                 │
│    1 Prestige Rank                     │
│    2 Fishing Ranks                     │
│                                        │
│  Brynn has:                            │
│    P1 (1 available)    ✓              │
│    Fish Rank 0 (0 available) ✗        │
│                                        │
│  Cannot build — not enough ranks       │
└────────────────────────────────────────┘
```

**Locked room:**
```
┌─ Watchtower 🔒 ───────────────────────┐
│                                        │
│  "Sharpens your eye for weak points." │
│                                        │
│  Locked                                │
│  Requires: Training Yard (T1)          │
│                                        │
│  T1: +3% crit chance                   │
│  T2: +6% crit chance                   │
│  T3: +10% crit chance                  │
└────────────────────────────────────────┘
```

**Maxed room (T3):**
```
┌─ Hearthstone ★★★ ────────────────────┐
│                                        │
│  "A warm fire keeps progress burning   │
│   while you're away."                  │
│                                        │
│  Grand (T3) — MAX                     │
│  +40% offline XP                       │
└────────────────────────────────────────┘
```

### Build/Upgrade Confirmation

Pressing `[Enter]` on a buildable room shows a confirmation overlay:

```
╔═ Confirm Build ═══════════════════════╗
║                                        ║
║  Build Training Yard (Basic)?          ║
║                                        ║
║  Aldric will spend:                    ║
║    Prestige: P12 → P11                 ║
║    Fishing:  Rank 18 → Rank 16        ║
║                                        ║
║  Gain: +5% XP (base)                  ║
║  Unlocks: Watchtower                   ║
║                                        ║
║  [Enter] Confirm    [Esc] Cancel       ║
╚════════════════════════════════════════╝
```

### Active Bonuses Summary

The Haven screen shows a summary of all active bonuses at the top, visible when you open it via `[H]`:

```
┌──────────────────────── Haven ──────────────────────────┐
│ Active bonuses (5/13 rooms):                             │
│ +10% DMG  +5% XP  +2% Drops  +3% Crit  +15% Regen     │
│ +10% Offline XP                                          │
├──────────────────────────────────────────────────────────┤
│         ┌────────────┐  ┌─ ...                           │
```

Bonuses are not shown on the stats panel — the Haven is "set and forget." Players check `[H]` when they want the full picture.

### Vault Item Selection (on Prestige)

When a player prestiges with the Vault built, the prestige confirmation mentions preserved items. After confirming, a selection screen appears:

```
╔═ Vault — Choose 2 Items to Keep ═════╗
║                                        ║
║  Select items to preserve:             ║
║                                        ║
║  > ⚔️ Stormbreaker [Legendary] ★★★★★  ║
║    🛡 Iron Plate [Rare] ★★★           ║
║    🪖 Silk Hood [Uncommon] ★★          ║
║    🧤 Chain Gloves [Common] ★          ║
║    👢 Steel Boots [Rare] ★★★          ║
║    📿 Jade Amulet [Epic] ★★★★         ║
║    💍 Copper Ring [Uncommon] ★★        ║
║                                        ║
║  Selected: 1/2                         ║
║  ⚔️ Stormbreaker                       ║
║                                        ║
║  [Enter] Toggle  [Space] Confirm       ║
╚════════════════════════════════════════╝
```

Number of items equals the Vault tier (T1: 1, T2: 2, T3: 3). Player selects which equipped items to keep. Prestige proceeds with those items preserved.

### Access from Gameplay

During gameplay, `[H]` opens the Haven as an overlay (like the challenge menu). The active character's ranks are shown for spending. `[Esc]` returns to combat. Ticks continue in the background.

## Data Model

### Storage

Account-level file: `~/.quest/haven.json`

Loaded at startup, separate from character saves. All characters read from and write to this shared file.

### Structures

```rust
pub struct Haven {
    pub discovered: bool,
    pub rooms: HashMap<HavenRoomId, u8>,  // room -> tier (0=unbuilt, 1-3)
}

pub enum HavenRoomId {
    Hearthstone,
    // Combat branch
    Armory,
    TrainingYard,
    TrophyHall,
    Watchtower,
    AlchemyLab,
    WarRoom,
    // QoL branch
    Bedroom,
    Garden,
    Library,
    FishingDock,
    Workshop,
    Vault,
}
```

### Bonus Application

Bonuses are computed once when a character is loaded, not every tick. Each system reads the relevant Haven bonus as a base-percentage modifier:

- Combat systems read damage, crit, attack interval, HP regen, drop rate bonuses
- Fishing reads phase timer and rank XP bonuses
- Offline progression reads offline XP bonus
- Challenge discovery reads discovery rate bonus
- Prestige logic reads Vault tier for item preservation

## Integration Points

### Discovery

Separate RNG roll per tick in the game loop, independent from challenge discovery. Requires P10+, not in dungeon/fishing/minigame. Chance scales with prestige: `0.000014 + (prestige_rank - 10) * 0.000007`. On success, set `haven.discovered = true`, save `haven.json`, and show the discovery modal overlay.

### Spending

From the Haven UI, the active character's `prestige_rank` and `fishing.rank` are decremented. Character save is updated immediately after spending.

### Prestige (Vault)

During prestige reset, check Vault tier. Preserve that many equipped items (player chooses which, or highest-scored by auto-equip).

### New Files

```
src/haven.rs              — Data structures, room definitions, tree structure, bonus values
src/haven_logic.rs        — Build/upgrade validation, cost checks, bonus calculation
src/ui/haven_scene.rs     — Skill tree rendering, room detail panel, build confirmation
```

## Player Strategies

- **Rush strategy:** Build every room at T1 (13 prestige, 26 fishing) to unlock all bonuses quickly at low values
- **Deep strategy:** Max out high-value rooms (Training Yard, Armory) before branching further
- **Vault rush:** Push straight down the QoL branch to get Vault T1, then prestige with a preserved weapon
- **Balanced:** Alternate between branches, upgrading as resources allow
