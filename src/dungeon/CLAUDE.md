# Dungeon System

Procedural grid-based dungeon exploration with fog of war, room types, key mechanics, and safe death.

## Module Structure

```
src/dungeon/
├── mod.rs         # Public re-exports
├── types.rs       # Room, RoomType, RoomState, Dungeon, DungeonSize
├── generation.rs  # Procedural dungeon generation with connected rooms
└── logic.rs       # Navigation, room clearing, key system, boss encounters
```

## Key Types

### `RoomType` (`types.rs`)
- **Entrance**: Starting room, no enemies
- **Combat**: Standard combat encounter (60% spawn rate)
- **Treasure**: Guaranteed item drop, no combat (20%)
- **Elite**: Tough guardian that drops the boss key (15%)
- **Boss**: Final encounter, requires key to unlock (5%)

### `RoomState` (`types.rs`)
- **Hidden**: Not yet visible (fog of war)
- **Revealed**: Visible but not entered (adjacent to visited room)
- **Current**: Player is currently in this room
- **Cleared**: Completed

### `Dungeon` (`types.rs`)
```rust
pub struct Dungeon {
    pub size: DungeonSize,
    pub grid: Vec<Vec<Option<Room>>>,   // 2D grid, None = no room
    pub player_position: (usize, usize),
    pub entrance_position: (usize, usize),
    pub boss_position: (usize, usize),
    pub has_key: bool,
    pub boss_defeated: bool,
    #[serde(default)]
    pub zone_id: u32,                   // Zone where dungeon was discovered (for enemy scaling)
}
```

### `DungeonSize`
| Size   | Grid  | Prestige Requirement |
|--------|-------|---------------------|
| Small  | 5x5   | Any                 |
| Medium | 7x7   | P5+                 |
| Large  | 9x9   | P10+                |
| Epic   | 11x11 | P15+                |

## Generation Algorithm (`generation.rs`)

```rust
pub fn generate_dungeon(level: u32, prestige_rank: u32, zone_id: u32) -> Dungeon
```

1. Roll dungeon size from level and prestige rank
2. Place Entrance at center of grid
3. Use random walk / branching to carve out connected rooms
4. Assign room types based on probability distribution (Combat 60%, Treasure 20%, Elite 15%, Boss 5%)
5. Ensure exactly one Elite and one Boss room per dungeon
6. Boss room placed far from entrance
7. Set connections between adjacent rooms (up/right/down/left booleans)
8. Entrance and adjacent rooms start Revealed; all others Hidden
9. Store `zone_id` on the Dungeon for enemy scaling

## Navigation & Clearing (`logic.rs`)

### Movement
- Player can move to Revealed or Cleared adjacent rooms
- Moving to a new room reveals its adjacent Hidden rooms (fog of war)
- Moving to a Combat/Elite room triggers combat

### Room Clearing Flow
1. **Combat room**: Defeat enemy → room becomes Cleared
2. **Treasure room**: Auto-clear, generate item drop
3. **Elite room**: Defeat guardian → get key (`has_key = true`) → room Cleared
4. **Boss room**: Requires `has_key == true` to enter. Defeat boss → `boss_defeated = true` → dungeon complete

### Death Handling
- Death in dungeon exits the dungeon entirely
- No prestige loss (safe death)
- Dungeon progress is lost (no saving mid-dungeon)

## Integration Points

- **Combat**: Dungeon combat uses the same `combat/logic.rs` system with zone-scaled enemies via `generate_dungeon_enemy(zone_id)`, `generate_dungeon_elite(zone_id)`, `generate_dungeon_boss(zone_id)`
- **Items**: Treasure rooms use `items/drops.rs` for guaranteed drops
- **UI**: `ui/dungeon_map.rs` renders the top-down minimap; `ui/combat_3d.rs` renders first-person view
- **Game State**: Active dungeon stored in `GameState.active_dungeon: Option<Dungeon>`
- **Spawning**: Dungeon enemies scale based on `dungeon.zone_id` (the zone where the dungeon was discovered)

## Adding a New Room Type

1. Add variant to `RoomType` enum in `types.rs`
2. Add `icon()` and `cleared_icon()` display characters
3. Add spawn probability in `generation.rs`
4. Add clearing logic in `logic.rs`
5. Add rendering in `ui/dungeon_map.rs` (minimap icon + color)
6. Add combat/reward handling if applicable
