# Zone System

Zone and subzone progression with prestige-gated tiers, boss encounters, and the Stormbreaker weapon gate.

## Module Structure

```
src/zones/
├── mod.rs          # Public re-exports (Zone, Subzone, ZoneProgression, BossDefeatResult)
├── data.rs         # Zone/subzone definitions, boss data, lookup functions
└── progression.rs  # Progression state, kill tracking, boss defeat logic, prestige reset
```

## Key Types

### `Zone` (`data.rs`)
```rust
pub struct Zone {
    pub id: u32,                        // 1-11
    pub name: &'static str,
    pub subzones: Vec<Subzone>,
    pub prestige_requirement: u32,      // Minimum prestige rank to unlock
    pub min_level: u32,
    pub max_level: u32,
    pub requires_weapon: bool,          // Zone 10 only
    pub weapon_name: Option<&'static str>,
}
```

### `Subzone` (`data.rs`)
```rust
pub struct Subzone {
    pub id: u32,        // 1-based within zone
    pub name: &'static str,
    pub depth: u32,     // Same as id, used for scaling
    pub boss: SubzoneBoss,
}
```

### `SubzoneBoss` (`data.rs`)
Each subzone has a named boss. The final subzone's boss has `is_zone_boss: true`.

### `ZoneProgression` (`progression.rs`)
Serializable state tracking the player's position and progress:
- `current_zone_id` / `current_subzone_id` -- current location
- `defeated_bosses: Vec<(u32, u32)>` -- (zone_id, subzone_id) pairs
- `unlocked_zones: Vec<u32>` -- zones the player can enter
- `kills_in_subzone: u32` -- kill counter toward boss spawn (resets on boss defeat or death)
- `fighting_boss: bool` -- whether a boss fight is active
- `has_stormbreaker: bool` -- legacy flag (achievement-based check preferred)

### `BossDefeatResult` (`progression.rs`)
Enum returned by `on_boss_defeated()`:
- **SubzoneComplete** -- advanced to next subzone
- **ZoneComplete** -- completed zone, advanced to next
- **ZoneCompleteButGated** -- zone done but next requires higher prestige
- **WeaponRequired** -- Zone 10 boss needs Stormbreaker
- **StormsEnd** -- completed Zone 10, unlocks Zone 11
- **ExpanseCycle** -- completed Zone 11 cycle, loops back to subzone 1

## Zone Tiers and Prestige Requirements

| Tier | Prestige | Zones | Subzones | Level Range |
|------|----------|-------|----------|-------------|
| 1: Nature's Edge | P0 | Meadow, Dark Forest | 3 each | 1-25 |
| 2: Civilization's Remnants | P5 | Mountain Pass, Ancient Ruins | 3 each | 25-55 |
| 3: Elemental Forces | P10 | Volcanic Wastes, Frozen Tundra | 4 each | 55-85 |
| 4: Hidden Depths | P15 | Crystal Caverns, Sunken Kingdom | 4 each | 85-115 |
| 5: Ascending | P20 | Floating Isles, Storm Citadel | 4 each | 115-150 |
| Post-game | StormsEnd achievement | The Expanse (Zone 11) | 4 | 150+ |

## Kill Tracking and Boss Spawn

1. Each mob kill calls `record_kill()`, incrementing `kills_in_subzone`
2. At `KILLS_FOR_BOSS` (10) kills, `fighting_boss` is set to `true`
3. The combat system spawns the subzone's named boss
4. On boss defeat, `on_boss_defeated()` handles advancement:
   - Subzone boss: advance to next subzone
   - Zone boss (final subzone): advance to next zone (if prestige allows)
5. On player death to boss: `kills_in_subzone` set to `KILLS_FOR_BOSS - KILLS_FOR_BOSS_RETRY` (5), so only 5 more kills needed to retry (not full 10)

Helper methods:
- `should_spawn_boss()` -- check without mutating state
- `kills_until_boss()` -- remaining kills needed

## Zone Advancement Flow

```
Kill 10 mobs -> Boss spawns -> Defeat boss
  |                               |
  |                     Is final subzone?
  |                     /              \
  |                   No               Yes
  |                   |                 |
  |            Next subzone     Has prestige for next zone?
  |                             /                    \
  |                           Yes                    No
  |                            |                      |
  |                      Next zone          ZoneCompleteButGated
  |                                        (stay in current zone)
  v
Player death -> Reset kills_in_subzone and fighting_boss
```

## Weapon Gate (Stormbreaker)

Zone 10 (Storm Citadel) final boss requires Stormbreaker:
- `boss_weapon_blocked(achievements)` checks `AchievementId::TheStormbreaker`
- Without the achievement, `on_boss_defeated()` returns `WeaponRequired` and resets the encounter
- The Stormbreaker path: max fishing rank -> catch Storm Leviathan (10 encounters) -> build Storm Forge in Haven -> forge Stormbreaker

## Zone 11: The Expanse

Infinite post-game zone unlocked by completing Zone 10:
- `on_boss_defeated()` unlocks `AchievementId::StormsEnd` and zone 11
- Has 4 subzones that cycle infinitely (`ExpanseCycle` result loops to subzone 1)
- `prestige_requirement: 0` because access is achievement-gated, not prestige-gated
- `max_level: u32::MAX` for unbounded scaling

## Prestige Reset

`reset_for_prestige(new_prestige_rank)`:
- Resets position to Zone 1, Subzone 1
- Clears all defeated bosses and kill tracking
- Recalculates `unlocked_zones` based on new prestige rank (zones whose `prestige_requirement <= rank`)
- Player can immediately `travel_to()` any unlocked zone's first subzone

## Lookup Functions (`data.rs`)

- `get_all_zones()` -- returns all 11 zones (allocates a new Vec each call)
- `get_zone(zone_id)` -- find by ID
- `get_subzone(zone_id, subzone_id)` -- returns `(Zone, Subzone)` pair

## Integration Points

- **Core** (`core/tick.rs`): Calls `record_kill()` and `on_boss_defeated()` during game tick processing
- **Core** (`core/game_logic.rs`): Enemy spawning uses zone/subzone data for stat scaling
- **Core** (`core/game_state.rs`): `GameState` owns a `ZoneProgression` instance
- **Combat** (`combat/types.rs`, `combat/logic.rs`): Enemy generation reads current zone, boss flag drives boss spawning
- **Items** (`items/drops.rs`): Item level = `zone_id * 10` (Zone 1 = ilvl 10, Zone 10 = ilvl 100)
- **Character** (`character/prestige.rs`): Prestige reset triggers `reset_for_prestige()`
- **Achievements** (`achievements/types.rs`): `TheStormbreaker` gates Zone 10 boss, `StormsEnd` unlocked on Zone 10 completion
- **Fishing** (`fishing/logic.rs`): Storm Leviathan path feeds into Stormbreaker forging
- **Haven** (`haven/types.rs`): Storm Forge room enables Stormbreaker creation
- **UI** (`ui/stats_panel.rs`): Displays current zone/subzone names and kill progress
