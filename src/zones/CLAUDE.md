# Zone System

Zone and subzone progression with prestige-gated tiers, boss encounters, the Stormbreaker weapon gate, and Deep-unlocked fracture zones 12-30.

## Module Structure

```
src/zones/
├── mod.rs          # Public re-exports (Zone, Subzone, ZoneProgression, BossDefeatResult, FractureRegion)
├── data.rs         # Zone/subzone definitions (30 zones), boss data, lookup functions
├── progression.rs  # Progression state, kill tracking, prestige reset
├── advancement.rs  # Zone/subzone advancement logic, travel_to(), advance_to_next_subzone()
├── boss_defeat.rs  # BossDefeatResult enum, on_boss_defeated(), on_boss_defeated_with_cap()
├── gates.rs        # boss_weapon_blocked(), zone unlock queries
├── fracture.rs     # FractureRegion enum (RedFault, MirrorScar, BlackMouth, HollowThrone, WailingReach, OriginWound) with chapter metadata
└── access.rs       # sync_account_zone_unlocks() — account-level zone access synchronization
```

## Key Types

### `Zone` (`data.rs`)
```rust
pub struct Zone {
    pub id: u32,                        // 1-30
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

### `BossDefeatResult` (`boss_defeat.rs`)
Enum returned by `on_boss_defeated()` / `on_boss_defeated_with_cap()`:
- **SubzoneComplete** -- advanced to next subzone
- **ZoneComplete** -- completed zone, advanced to next
- **ZoneCompleteButGated** -- zone done but next requires higher prestige
- **WeaponRequired** -- Zone 10 boss needs Stormbreaker
- **StormsEnd** -- completed Zone 10, unlocks Zone 11
- **ExpanseCycle** -- completed Zone 11 cycle, loops back to subzone 1
- **FractureCycle { zone_id }** -- completed a fracture cap zone cycle, loops back to subzone 1

### `FractureRegion` (`fracture.rs`)
Named fracture chapters, each containing 3-4 zones:
- `RedFault` (Zones 12-14, unlocked by P50 + Deep Layer 3)
- `MirrorScar` (Zones 15-17, unlocked by P75 + Deep Layer 7)
- `BlackMouth` (Zones 18-20, unlocked by P100 + Deep Layer 12)
- `HollowThrone` (Zones 21-23, unlocked by P150 + Deep Layer 18)
- `WailingReach` (Zones 24-26, unlocked by P200 + Deep Layer 25)
- `OriginWound` (Zones 27-30, unlocked by P300 + Deep Layer 30)

Methods: `start_zone_id()`, `end_zone_id()`, `unlock_layer()`, `from_layer()`, `unlock_headline()`, `unlock_atmospheric()`, `unlock_mechanical()`, `unlock_log_line()`, `unlock_ticker_text()`

## Zone Tiers and Prestige Requirements

| Tier | Prestige | Zones | Subzones | Level Range |
|------|----------|-------|----------|-------------|
| 1: Nature's Edge | P0 | Meadow, Dark Forest | 3 each | 1-25 |
| 2: Civilization's Remnants | P5 | Mountain Pass, Ancient Ruins | 3 each | 25-55 |
| 3: Elemental Forces | P10 | Volcanic Wastes, Frozen Tundra | 4 each | 55-85 |
| 4: Hidden Depths | P15 | Crystal Caverns, Sunken Kingdom | 4 each | 85-115 |
| 5: Ascending | P20 | Floating Isles, Storm Citadel | 4 each | 115-150 |
| Endgame | P25 + StormsEnd achievement | The Expanse (Zone 11) | 4 | 150+ |
| Ch.1: The Red Fault | P50 + Deep Layer 3 | Splintered Rim, Ember Ravine, Heart of the Fault (Z12-14) | 5 each | 165-210 |
| Ch.2: The Mirror Scar | P75 + Deep Layer 7 | Shard Fields, Refraction Steps, Hall of Second Suns (Z15-17) | 5 each | 210-255 |
| Ch.3: The Black Mouth | P100 + Deep Layer 12 | Ashen Verge, Throat of the World, The Black Mouth (Z18-20) | 5 each | 255-300 |
| Ch.4: The Hollow Throne | P150 + Deep Layer 18 | Sunken Processional, The Pale Archive, The Hollow Throne (Z21-23) | 5 each | 300-345 |
| Ch.5: The Wailing Reach | P200 + Deep Layer 25 | The Stillborn Sea, Resonance Fault, The Wailing Reach (Z24-26) | 5 each | 345-390 |
| Ch.6: The Origin Wound | P300 + Deep Layer 30 | The Scar Root, Echoing Abyss, Threshold of Silence, The Origin Wound (Z27-30) | 5 each | 390+ |

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

Infinite endgame zone unlocked by completing Zone 10:
- `on_boss_defeated()` unlocks `AchievementId::StormsEnd` and zone 11
- Has 4 subzones that cycle infinitely (`ExpanseCycle` result loops to subzone 1)
- `prestige_requirement: 25` -- dual-gated by StormsEnd achievement AND P25 prestige
- `max_level: u32::MAX` for unbounded scaling
- When fracture zones are unlocked (`fracture_zone_cap > 11`), Expanse stops cycling and advances to Zone 12

## Fracture Zones 12-30

Nineteen zones across six chapters, unlocked by Deep layer breakthroughs. Enemy stats scale at 1.6x per zone from Zone 11.

**Unlock cadence:**
1. Deep Layer 3 breakthrough -> Zones 12-14 (The Red Fault), cap = 14
2. Deep Layer 7 breakthrough -> Zones 15-17 (The Mirror Scar), cap = 17
3. Deep Layer 12 breakthrough -> Zones 18-20 (The Black Mouth), cap = 20
4. Deep Layer 18 breakthrough -> Zones 21-23 (The Hollow Throne), cap = 23
5. Deep Layer 25 breakthrough -> Zones 24-26 (The Wailing Reach), cap = 26
6. Deep Layer 30 breakthrough -> Zones 27-30 (The Origin Wound), cap = 30

**Progression semantics:**
- Only the current cap zone cycles (returns `FractureCycle`)
- All other fracture zones advance forward when their boss is defeated
- Zone 30 becomes the permanent fracture loop cap
- Fracture zones have prestige requirements: P50 (Z12-14), P75 (Z15-17), P100 (Z18-20), P150 (Z21-23), P200 (Z24-26), P300 (Z27-30) — enforced alongside Deep layer gates by `sync_account_zone_unlocks()`

**Boss defeat with cap awareness:** `on_boss_defeated_with_cap(prestige_rank, achievements, fracture_zone_cap)` extends the original `on_boss_defeated()` to handle fracture cycling logic.

## Zone Access Sync (`access.rs`)

`sync_account_zone_unlocks(prog, storms_end_unlocked, fracture_zone_cap)`:
- Called at: character load, prestige reset, StormsEnd, fracture region unlock
- If `storms_end_unlocked`, unlocks Zone 11
- Unlocks every zone in `12..=fracture_zone_cap`
- Never unlocks above cap, never removes earlier unlocks

## Prestige Reset

`reset_for_prestige(new_prestige_rank)`:
- Resets position to Zone 1, Subzone 1
- Clears all defeated bosses and kill tracking
- Recalculates `unlocked_zones` based on new prestige rank (zones whose `prestige_requirement <= rank`)
- Player can immediately `travel_to()` any unlocked zone's first subzone

## Lookup Functions (`data.rs`)

- `get_all_zones()` -- returns all 30 zones (static slice, no allocation)
- `get_zone(zone_id)` -- find by ID
- `get_subzone(zone_id, subzone_id)` -- returns `(Zone, Subzone)` pair

## Integration Points

- **Core** (`core/tick.rs`): Calls `record_kill()` and `on_boss_defeated()` during game tick processing
- **Core** (`core/game_logic.rs`): Enemy spawning uses zone/subzone data for stat scaling
- **Core** (`core/game_state.rs`): `GameState` owns a `ZoneProgression` instance
- **Combat** (`combat/enemy_generation.rs`, `combat/orchestration.rs`): Enemy generation reads current zone, boss flag drives boss spawning
- **Items** (`items/drops.rs`): Item level = `zone_id * 10` (Zone 1 = ilvl 10, Zone 10 = ilvl 100)
- **Character** (`character/prestige.rs`): Prestige reset triggers `reset_for_prestige()`
- **Achievements** (`achievements/types.rs`): `TheStormbreaker` gates Zone 10 boss, `StormsEnd` unlocked on Zone 10 completion
- **Fishing** (`fishing/logic.rs`): Storm Leviathan path feeds into Stormbreaker forging
- **Haven** (`haven/types.rs`): Storm Forge room enables Stormbreaker creation
- **Ascension** (`ascension/`): Ascension multiplier provides the combat power to progress through fracture zones
- **Deep** (`deep/types.rs`): `DeepPersistent.fracture_zone_cap` and `pending_fracture_region_unlock` control zone access
- **UI** (`ui/stats_panel.rs`): Displays current zone/subzone names, kill progress, and POST row for zones 12-30
