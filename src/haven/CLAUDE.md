# Haven System

Account-level base building that persists across all prestige resets and benefits every character.

## Module Structure

```
src/haven/
├── mod.rs      # Public re-exports
├── types.rs    # Haven struct, room definitions, upgrade trees, bonus system
└── logic.rs    # Room construction, upgrades, bonus calculation
```

## Key Concepts

### Account-Level Persistence
Haven is **not** tied to a single character. It persists across:
- Prestige resets (never lost)
- Character switches (shared across all characters)
- Stored separately from character saves

### Room System
The Haven consists of buildable rooms, each with upgrade tiers:
- Each room provides a specific bonus type
- Rooms cost prestige ranks and/or fishing ranks to build
- Upgrades increase the bonus magnitude

### Bonus Types (`HavenBonusType`)
Bonuses that Haven rooms can provide:
- **XP multiplier**: Increases XP gain rate
- **Item drop rate**: Increases chance of item drops (Trophy Hall)
- **Item rarity**: Shifts rarity distribution upward (Workshop)
- **Fishing gain**: Increases fishing rank progression
- **Challenge discovery**: Increases minigame discovery rate
- And others tied to specific room types

## Integration Points

Haven bonuses are consumed by other systems via parameter passing:

- **Items** (`items/drops.rs`): `try_drop_item_with_haven(state, drop_bonus, rarity_bonus)` — Trophy Hall and Workshop bonuses
- **Challenges** (`challenges/menu.rs`): Discovery rate boost from Haven
- **Combat/XP** (`core/game_logic.rs`): XP multiplier bonus
- **Fishing** (`fishing/logic.rs`): Fishing rank gain bonus
- **UI** (`ui/haven_scene.rs`): Haven overlay for building/upgrading
- **Input** (`input.rs`): `HavenUiState` manages the overlay

## Haven UI

Haven is displayed as an overlay (not a separate screen):
- Toggled from the game screen
- Shows room list with current tier, cost, and bonus description
- Build/Upgrade confirmation dialog
- Accessible from character select screen too

## Adding a New Haven Room

1. Add room definition in `types.rs` (ID, name, description, tier costs, bonus values)
2. Add bonus application logic in the consuming module
3. Pass the bonus value through the appropriate function parameters
4. Update `ui/haven_scene.rs` to display the new room
5. Consider achievement integration for building/upgrading

## Design Pattern

Haven follows a **parameter injection** pattern rather than global state:
- Haven bonuses are calculated once per tick
- Passed as explicit parameters to functions that need them
- This keeps modules decoupled — `items/drops.rs` doesn't import `haven/`
