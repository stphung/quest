# Haven System

Account-level base building that persists across all prestige resets and benefits every character.

## Module Structure

```
src/haven/
├── mod.rs      # Public re-exports
├── types.rs    # Haven struct, 14 room definitions, skill tree, upgrade tiers, 15 bonus types
└── logic.rs    # Room construction, upgrades, bonus calculation, prestige rank cost system
```

## Key Concepts

### Account-Level Persistence
Haven is **not** tied to a single character. It persists across:
- Prestige resets (never lost)
- Character switches (shared across all characters)
- Stored in `~/.quest/haven.json`

### Discovery
Haven is discovered randomly at P10+. Base discovery chance: `0.000014 + (prestige_rank - 10) × 0.000007` per tick.

### Room Skill Tree
The Haven consists of 14 rooms organized in a two-branch skill tree. Each room has upgrade tiers (most have 3 tiers, exceptions noted). Rooms require parent rooms at T1+ to unlock. Capstones require both parents.

```
                    Hearthstone (root)
                   /              \
            Armory                 Bedroom
           /      \               /      \
   TrainingYard  TrophyHall   Garden    Library
        |            |          |          |
    Watchtower  AlchemyLab  FishingDock  Workshop
         \         /           \          /
          War Room              Vault
               \               /
              Storm Forge (ultimate)
```

### Room Bonuses

| Room | Bonus Type | T1 | T2 | T3 | T4 | Max Tier |
|------|-----------|-----|-----|-----|-----|----------|
| Hearthstone | Offline XP | +25% | +50% | +100% | — | 3 |
| Armory | Damage | +5% | +10% | +25% | — | 3 |
| Training Yard | XP Gain | +5% | +10% | +30% | — | 3 |
| Trophy Hall | Drop Rate | +5% | +10% | +15% | — | 3 |
| Watchtower | Crit Chance | +5% | +10% | +20% | — | 3 |
| Alchemy Lab | HP Regen | +25% | +50% | +100% | — | 3 |
| War Room | Double Strike | +10% | +20% | +35% | — | 3 |
| Bedroom | Regen Delay Reduction | -15% | -30% | -50% | — | 3 |
| Garden | Fishing Timer Reduction | -10% | -20% | -40% | — | 3 |
| Library | Challenge Discovery | +20% | +30% | +50% | — | 3 |
| Fishing Dock | Double Fish | +25% | +50% | +100% | +10 Max Rank | 4 |
| Workshop | Item Rarity | +10% | +15% | +25% | — | 3 |
| Vault | Items Preserved | 1 | 3 | 5 | — | 3 |
| Storm Forge | Stormbreaker forging | enabled | — | — | — | 1 |

### Prestige Rank Costs

Costs scale with tree depth:

| Depth | Rooms | T1 | T2 | T3 |
|-------|-------|-----|-----|-----|
| 0 | Hearthstone | 1 | 2 | 3 |
| 1 | Armory, Bedroom | 1 | 3 | 5 |
| 2-3 | Mid-tree rooms | 2 | 4 | 6 |
| 4 | War Room, Vault | 3 | 5 | 7 |

Special: Fishing Dock T4 costs 10 PR. Storm Forge (single tier) costs 25 PR.

## Storm Forge

The ultimate Haven room, requiring both capstones (War Room + Vault) to unlock:
- Single tier, costs 25 prestige ranks
- Enables forging of Stormbreaker weapon
- Requires catching the Storm Leviathan first (tracked via achievements)
- Stormbreaker is required to defeat Zone 10's final boss

## Integration Points

Haven bonuses are consumed by other systems via parameter passing:

- **Items** (`items/drops.rs`): `try_drop_from_mob(state, zone_id, drop_bonus, rarity_bonus)` — Trophy Hall and Workshop bonuses (mob drops only, not boss drops)
- **Challenges** (`challenges/menu.rs`): Library discovery rate boost
- **Combat/XP** (`core/game_logic.rs`): Training Yard XP multiplier, Armory damage, Watchtower crit, War Room double strike
- **Fishing** (`fishing/logic.rs`): `HavenFishingBonuses` struct with Garden timer reduction, Fishing Dock double fish chance, max rank bonus
- **Offline** (`core/game_logic.rs`): Hearthstone offline XP bonus
- **UI** (`ui/haven_scene.rs`): Haven overlay for building/upgrading
- **Input** (`input.rs`): `HavenUiState` manages the overlay

## Haven UI

Haven is displayed as an overlay (not a separate screen):
- Toggled from the game screen
- Shows room list with current tier, cost, and bonus description
- Build/Upgrade confirmation dialog
- Accessible from character select screen too

## Adding a New Haven Room

1. Add `HavenRoomId` variant in `types.rs`
2. Add to `HavenRoomId::ALL` array
3. Implement `name()`, `description()`, `parents()`, `children()`, `depth()`, `max_tier()` matches
4. Add tier costs in `tier_cost()` function
5. Add bonus definition in `bonus()` match
6. Add `HavenBonusType` variant if new bonus type
7. Add bonus display formatting in `format_bonus()`
8. Add bonus application logic in the consuming module
9. Pass the bonus value through the appropriate function parameters
10. Update `ui/haven_scene.rs` to display the new room
11. Consider achievement integration for building/upgrading

## Design Pattern

Haven follows a **parameter injection** pattern rather than global state:
- Haven bonuses are calculated once per tick (via `compute_bonuses()`)
- Passed as explicit parameters to functions that need them
- This keeps modules decoupled — `items/drops.rs` doesn't import `haven/`
