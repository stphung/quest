# Combat System

Turn-based auto-combat with zone-based enemy generation, prestige combat bonuses, damage calculation, and event-driven state transitions.

## Module Structure

```
src/combat/
├── mod.rs      # Public re-exports
├── types.rs    # Enemy struct, CombatState enum, zone-based enemy generators
└── logic.rs    # Turn processing, damage calculation, HP regen, boss encounters
```

## Key Types

### `Enemy` (`types.rs`)
```rust
pub struct Enemy {
    pub name: String,
    pub max_hp: u32,
    pub current_hp: u32,
    pub damage: u32,
    #[serde(default)]
    pub defense: u32,
}
```

Constructors:
- `Enemy::new(name, max_hp, damage)` -- Legacy constructor (defense = 0)
- `Enemy::new_with_defense(name, max_hp, damage, defense)` -- Full constructor

### `CombatState` (`types.rs`)
State machine for combat flow:
- **Idle**: No enemy, waiting for spawn
- **Fighting**: Active combat (turns every 1.5s for player, variable for enemies by tier)
- **Regen**: HP regenerating after kill (2.5s)
- **Dead**: Player died (triggers reset or dungeon exit)

## Combat Flow

1. **Enemy spawn**: Triggered by zone progression or dungeon room entry
2. **Turn loop**: Player attacks every 1.5s (15 ticks); enemy attack intervals vary by tier (2.0s normal, 1.8s boss, 1.5s zone boss, 1.6s dungeon elite, 1.4s dungeon boss)
3. **Player damage pipeline**: base damage (from DerivedStats) -> Haven % bonus (Armory) -> prestige flat damage -> subtract enemy defense -> min 1 -> crit roll (2x)
4. **Enemy damage pipeline**: enemy.damage -> subtract (derived.defense + prestige flat_defense) -> min 1
5. **Critical hits**: Chance from DEX modifier + prestige crit bonus (capped at 15%), deals 2x damage
6. **Enemy death**: Awards XP, triggers item drop roll, enters Regen state
7. **Player death**:
   - In zone: Sets `kills_in_subzone = KILLS_FOR_BOSS - KILLS_FOR_BOSS_RETRY` (only 5 more kills needed), preserves prestige
   - In dungeon: Exits dungeon, no prestige loss

## Enemy Generation (Zone-Based Static Scaling)

Enemies scale from a static `ZONE_ENEMY_STATS` table in `core/constants.rs`, **not** from player HP. Each zone has `(base_hp, hp_step, base_dmg, dmg_step, base_def, def_step)` tuples. Subzone depth adds incremental stats via `hp_step`/`dmg_step`/`def_step`.

### Zone Enemy Generators (`types.rs`)

| Function | Purpose |
|----------|---------|
| `generate_zone_enemy(zone, subzone)` | Normal mob from zone/subzone stats |
| `generate_subzone_boss(zone, subzone)` | Boss with multiplied stats (subzone or zone boss multipliers) |
| `generate_enemy_for_current_zone(zone_id, subzone_id)` | Convenience wrapper for zone mob |
| `generate_boss_for_current_zone(zone_id, subzone_id)` | Convenience wrapper for zone boss |
| `generate_dungeon_enemy(zone_id)` | Dungeon combat room enemy (base zone stats, depth 1) |
| `generate_dungeon_elite(zone_id)` | Dungeon elite with `DUNGEON_ELITE_MULTIPLIERS` |
| `generate_dungeon_boss(zone_id)` | Dungeon boss with `DUNGEON_BOSS_MULTIPLIERS` |

### Boss Stat Multipliers (from `core/constants.rs`)
- **Subzone boss**: 3.0x HP, 1.5x DMG, 1.8x DEF
- **Zone boss**: 5.0x HP, 1.8x DMG, 2.5x DEF
- **Dungeon elite**: 2.2x HP, 1.5x DMG, 1.6x DEF
- **Dungeon boss**: 3.5x HP, 1.8x DMG, 2.0x DEF

### Zone 11: The Expanse (Endgame Wall)
Zone 11 has dramatically higher stats than Zone 10 (~6.2x HP, ~4.6x DMG, ~4.8x DEF). Designed as an endgame wall requiring very high prestige ranks (P50+) to farm comfortably.

## Prestige Combat Bonuses

`update_combat()` receives a `&PrestigeCombatBonuses` (from `character/prestige.rs`) that provides flat bonuses scaling with prestige rank:
- **flat_damage**: Added after Haven % multiplier, before enemy defense subtraction
- **flat_defense**: Added to DEX-based defense when calculating damage taken
- **crit_chance**: Added to DEX-based crit chance (capped at PRESTIGE_CRIT_CAP = 15%)
- **flat_hp**: Applied to `combat_state.player_max_hp` in `core/tick.rs` (not in DerivedStats)

## Boss Encounters

- After 10 kills in a subzone, the next enemy is the subzone boss
- Boss defined in `zones/data.rs` with specific stats
- Defeating boss advances to next subzone
- Death to boss sets `kills_in_subzone = KILLS_FOR_BOSS - KILLS_FOR_BOSS_RETRY` (only 5 more kills to retry, not 10)
- Zone 10 final boss requires Stormbreaker weapon (checked via `TheStormbreaker` achievement in `zones/progression.rs`)

## Key Function: `update_combat()`

```rust
pub fn update_combat(
    state: &mut GameState,
    delta_time: f64,
    haven: &HavenCombatBonuses,
    prestige_bonuses: &PrestigeCombatBonuses,
    achievements: &mut Achievements,
) -> Vec<CombatEvent>
```

Called from `core/tick.rs` each tick. Returns `Vec<CombatEvent>` that tick.rs maps to `TickEvent` variants.

## Integration Points

- **Core** (`core/tick.rs`): Drives the per-tick game loop, computes `PrestigeCombatBonuses::from_rank()`, applies `flat_hp` to combat HP
- **Core** (`core/game_logic.rs`): Enemy spawning via zone-based generators, XP calculation, level-up logic
- **Character** (`character/derived_stats.rs`): Player base damage, defense, HP, crit stats
- **Character** (`character/prestige.rs`): `PrestigeCombatBonuses` struct with `from_rank()` constructor
- **Items** (`items/drops.rs`): Mob drops via `try_drop_from_mob()`, boss drops via `try_drop_from_boss()`
- **Zones** (`zones/progression.rs`): Zone-based stat lookup, boss definitions
- **Dungeon** (`dungeon/logic.rs`): Dungeon room combat with zone-scaled enemies
- **UI** (`ui/combat_scene.rs`): HP bars, enemy sprites, visual effects

## Constants (from `core/constants.rs`)

- Player attack interval: 1.5s (15 ticks)
- Enemy attack intervals: 2.0s (normal), 1.8s (boss), 1.5s (zone boss), 1.6s (dungeon elite), 1.4s (dungeon boss)
- HP regen duration: 2.5s after kill
- XP per kill: 200-400 ticks of passive XP
- Boss kill tracking: 10 kills per subzone to trigger boss, 5 kills to retry after death
- Boss stat multipliers: Subzone (3.0/1.5/1.8), Zone (5.0/1.8/2.5), Dungeon Elite (2.2/1.5/1.6), Dungeon Boss (3.5/1.8/2.0)
