# Haven: Base Building System - Design Document

## Overview

The Haven is an account-level base building system presented as a skill tree. Players spend prestige ranks and fishing ranks to construct and upgrade rooms that provide permanent passive bonuses. The Haven persists across all prestige resets and benefits every character on the account.

## Unlock Conditions

- **Prestige gate:** Character must reach P10 (Celestial tier)
- **Discovery:** Once at P10, the Haven enters the weighted discovery table alongside challenges (~2hr average via RNG tick)
- **Persistence:** Once any character discovers the Haven, it remains accessible to all characters regardless of prestige rank — even if the discovering character drops below P10 through spending

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

## UI Design

### Skill Tree View

Accessed from the character select screen via a keypress (e.g., `H`). Displays the full tree with visual indicators:

- **Built rooms (T1-T3):** Bright, with tier indicator
- **Available rooms (prerequisites met):** Pulsing or highlighted border
- **Locked rooms:** Dim, shows prerequisite chain

Arrow keys navigate between rooms. Selected room shows a detail panel:
- Room name and description
- Current tier and bonus
- Next tier cost (prestige + fishing ranks from active character)
- Build/upgrade confirmation prompt

### Visual Progression

Room nodes in the tree change appearance with each tier:
- Unbuilt: `[ · ]`
- T1 (Basic): `[room]` dim
- T2 (Improved): `[ROOM]` normal
- T3 (Grand): `[ROOM]` bright/highlighted

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

Add Haven to the weighted discovery table in `challenge_menu.rs`. Requires P10+. Once discovered, set `haven.discovered = true` and save `haven.json`.

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
