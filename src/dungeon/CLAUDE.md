# Dungeon System

Procedural grid-based dungeon exploration with fog of war, room types, key mechanics, and safe death.

## Module Structure

```
src/dungeon/
├── mod.rs         # Public re-exports
├── types.rs       # Room, RoomType, RoomState, Dungeon, DungeonSize
├── generation.rs  # Procedural dungeon generation with connected rooms
├── facade.rs      # tick_dungeon_facade() for decoupled dungeon ticking; DungeonInput struct is currently unused (aspirational decomposed interface, not wired in)
├── logic.rs       # Room clearing, key system, boss encounters
├── pathfinding.rs # BFS-based dungeon navigation, auto-exploration (ROOM_MOVE_INTERVAL 2.5s, ROOM_TRAVEL_INTERVAL 0.8s)
└── rewards.rs     # Dungeon boss XP rewards, item generation, treasure room handling
```

## Key Types

### `RoomType` (`types.rs`)
- **Entrance**: Starting room, no enemies (exactly 1)
- **Combat**: Standard combat encounter (all remaining rooms)
- **Treasure**: Guaranteed item drop, no combat (1-8 rooms by dungeon size)
- **Elite**: Tough guardian that drops the boss key (exactly 1)
- **Boss**: Final encounter, requires key to unlock (exactly 1)

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
    pub move_timer: f64,                // Timer for auto-exploration movement (seconds)
    pub collected_items: Vec<Item>,     // Items collected during this dungeon run
    pub xp_earned: u64,                 // Total XP earned in this dungeon
    pub rooms_cleared: u32,             // Number of rooms cleared
    #[serde(default)]
    pub current_room_cleared: bool,     // Whether current room's combat is done
    #[serde(skip)]
    pub is_traveling: bool,             // Currently traveling through cleared rooms (UI display)
    #[serde(default = "default_dungeon_zone")]
    pub zone_id: u32,                   // Zone where dungeon was discovered (for enemy scaling)
}
```

### `DungeonSize`
| Size      | Grid  |
|-----------|-------|
| Small     | 5x5   |
| Medium    | 7x7   |
| Large     | 9x9   |
| Epic      | 11x11 |
| Legendary | 13x13 |

Size is determined by `base_tier(level, prestige_rank)` which combines a level component (`level_tier`: 0 for level <25, 1 for 25-74, 2 for 75+) with `prestige_rank / 2`. A 20% variance roll may shift the result up or down by one tier. Higher level and prestige yield larger dungeons.

## Generation Algorithm (`generation.rs`)

```rust
pub fn generate_dungeon(level: u32, prestige_rank: u32, zone_id: u32) -> Dungeon
```

1. Roll dungeon size from level and prestige rank
2. Place Entrance at center of grid
3. Use random walk / branching to carve out connected rooms
4. Place special rooms deterministically (`place_special_rooms()`): exactly one Boss at the dead end furthest from the entrance, exactly one Elite at a dead end far from the entrance, then `treasure_room_count()` Treasure rooms by size (Small 1, Medium 2, Large 3, Epic 5, Legendary 8) at random positions
5. Remaining rooms stay Combat (default)
6. Set connections between adjacent rooms (up/right/down/left booleans)
7. Entrance and adjacent rooms start Revealed; all others Hidden
8. Store `zone_id` on the Dungeon for enemy scaling

## Navigation & Clearing (`logic.rs`, `pathfinding.rs`)

### Movement
- Auto-exploration uses BFS pathfinding (`pathfinding.rs`) to find the next room
- Player moves between rooms on a timer (2.5s for new rooms, 0.8s for cleared rooms)
- Moving to a new room reveals its adjacent Hidden rooms (fog of war)
- Moving to a Combat/Elite room triggers combat

### Room Clearing Flow
1. **Combat room**: Defeat enemy → room becomes Cleared
2. **Treasure room**: Auto-clear, generate item drop
3. **Elite room**: Defeat guardian → get key (`has_key = true`) → room Cleared
4. **Boss room**: Requires `has_key == true` to enter. Defeat boss → dungeon cleared (`state.active_dungeon = None`), emits `DungeonEvent::DungeonComplete` → dungeon complete

### Death Handling
- Death in dungeon exits the dungeon entirely
- No prestige loss (safe death)
- Dungeon progress is lost (no saving mid-dungeon)
- Stalemate protection: a dungeon fight that lasts 60s (`DUNGEON_FIGHT_TIMEOUT_SECONDS`) triggers a combat retreat that abandons the dungeon the same way (safe exit, no prestige loss)

## Integration Points

- **Combat**: Dungeon combat uses the same combat system with zone-scaled enemies via `generate_dungeon_enemy(zone_id)`, `generate_dungeon_elite(zone_id)`, `generate_dungeon_boss(zone_id)` (`combat/enemy_generation.rs`)
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
