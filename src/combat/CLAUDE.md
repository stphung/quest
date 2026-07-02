# Combat System

Turn-based auto-combat with zone-based enemy generation, prestige combat bonuses, damage calculation, and event-driven state transitions.

## Module Structure

```
src/combat/
├── mod.rs              # Public re-exports
├── types.rs            # Enemy struct, CombatState
├── logic.rs            # Re-exports (update_combat, effective_enemy_attack_interval) + tests
├── orchestration.rs    # update_combat() orchestrator — coordinates attack phases
├── attacks.rs          # Attack interval calculations (effective_enemy_attack_interval)
├── enemy_generation.rs # Zone/dungeon enemy generators (generate_zone_enemy, generate_dungeon_boss, etc.)
├── player_attack.rs    # Player damage pipeline (weapon gate → damage → crit → double strike)
├── enemy_attack.rs     # Enemy attack resolution (defense, Bulwark DR, reflection, death)
├── damage.rs           # Shared damage helpers, handle_enemy_death()
├── events.rs           # CombatEvent enum, CombatBonuses (unified struct)
├── facade.rs           # Combat facade for decoupled combat updates
└── regen.rs            # HP regeneration after combat
```

## Key Types

### `Enemy` (`types.rs`)
```rust
pub struct Enemy {
    pub name: String,
    pub max_hp: u64,
    pub current_hp: u64,
    pub damage: u64,
    #[serde(default)]
    pub defense: u64,
}
```

Constructors:
- `Enemy::new(name, max_hp, damage)` -- Legacy constructor (defense = 0)
- `Enemy::new_with_defense(name, max_hp, damage, defense)` -- Full constructor

### `CombatState` (`types.rs`)
Struct whose fields encode the combat flow:
- **Idle**: `current_enemy` is `None`, waiting for spawn
- **Fighting**: `current_enemy` is `Some` (turns every 1.5s for player, variable for enemies by tier)
- **Regen**: `is_regenerating` is `true`, HP regenerating after kill (2.5s)
- **Dead**: Player death is event-driven (triggers reset or dungeon exit), not a stored state

## Combat Flow

1. **Enemy spawn**: Triggered by zone progression or dungeon room entry
2. **Turn loop**: Player attacks every 1.5s (15 ticks); enemy attack intervals vary by tier (2.0s normal, 1.8s boss, 1.5s zone boss, 1.6s dungeon elite, 1.4s dungeon boss)
3. **Player damage pipeline**: base damage (from DerivedStats) -> Giant's Might % (god item) -> Haven % bonus (Armory) -> prestige flat damage -> subtract enemy defense -> min 1 -> crit roll (2x)
4. **Enemy damage pipeline**: enemy.damage -> subtract (derived.defense + prestige flat_defense) -> min 1 -> Divine Bulwark DR % (god item) -> min 1
5. **Critical hits**: Chance from DEX modifier + prestige crit bonus (capped at 15%), deals 2x damage
6. **Enemy death**: Awards XP, triggers item drop roll, enters Regen state
7. **Player death**:
   - In zone: Resets `kills_in_subzone = 0`, preserves prestige
   - In dungeon: Exits dungeon, no prestige loss

## Enemy Generation (Zone-Based Static Scaling)

Enemies scale from a static `ZONE_ENEMY_STATS` table in `core/constants.rs`, **not** from player HP. Each zone has `(base_hp, hp_step, base_dmg, dmg_step, base_def, def_step)` tuples. Subzone depth adds incremental stats via `hp_step`/`dmg_step`/`def_step`.

### Zone Enemy Generators (`enemy_generation.rs`)

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

## Unified Combat Bonuses

`update_combat()` receives a single `&CombatBonuses` struct (defined in `events.rs`) that merges all bonus sources — Haven, god items, prestige, and sigils — into one parameter. Key fields:

**Damage pipeline** (`player_attack.rs`):
- **early_damage_percent**: Applied to base damage first (e.g. Giant's Might 150%)
- **damage_percent**: Applied after early_damage_percent (e.g. Haven Armory, sigils)
- **flat_damage**: Added after % multipliers, before enemy defense (prestige)
- **ascension_multiplier**: Applied after flat_damage, before enemy defense (from Ascension system, default 1.0)
- **crit_chance_percent**: Haven Watchtower + prestige crit + sigils
- **double_strike_chance**: Haven War Room + sigils
- **xp_gain_percent**: Haven Training Yard + sigils

**Defense pipeline** (`enemy_attack.rs`):
- **flat_defense**: Added to DEX-based defense (prestige)
- **ascension_multiplier**: Applied to total defense (from Ascension system, default 1.0)
- **damage_reduction_percent**: After defense subtraction (e.g. Divine Bulwark 30%, sigils)

**Other**:
- **ascension_multiplier**: Also applied to player max HP in `core/tick.rs` (default 1.0x, up to 64x+ at Ascension VI)
- **flat_hp** (from `PrestigeCombatBonuses`): Applied to `combat_state.player_max_hp` in `core/tick.rs`
- **attack_speed_percent**: Windborne + sigils
- **hp_regen_percent**, **hp_regen_delay_reduction**, **regen_reduction_percent**: Regen modifiers from Haven, sigils, Sleipnir

## Combat Pipelines (Quick Reference)

- **Damage pipeline**: base damage --> Giant's Might % --> Haven Armory % --> prestige flat damage --> ascension multiplier --> enemy defense --> min 1 --> crit (2x)
- **Enemy damage pipeline**: enemy.damage --> subtract (defense + prestige flat_defense) --> min 1 --> Divine Bulwark DR % --> min 1
- **Defense pipeline**: base defense --> prestige flat defense --> ascension multiplier --> damage reduction %
- **Ascension multiplier**: Also applied to player max HP in `core/tick.rs` (default 1.0x, up to 64x+ at Ascension VI)
- **Boss enrage timer**: Bosses enrage after 60 seconds of combat, increasing damage output (instant kill)
- **Stalemate timeouts**: Non-boss fights auto-retreat to the last safe zone after `MOB_FIGHT_TIMEOUT_SECONDS` (30s); dungeon fights use `DUNGEON_FIGHT_TIMEOUT_SECONDS` (60s, elites/bosses have up to 3.5x HP). Retreating while inside a dungeon abandons it (emits `DungeonRetreat` → `TickEvent::DungeonFailed`, no prestige loss) so the uncleared room cannot respawn its enemy in an endless loop

See "Unified Combat Bonuses" below for the full field-level breakdown of `CombatBonuses`.

## Boss Encounters

- After 10 kills in a subzone, the next enemy is the subzone boss
- Boss defined in `zones/data.rs` with specific stats
- Defeating boss advances to next subzone
- Death to boss resets `kills_in_subzone = 0` (full 10 kills needed to retry)
- Zone 10 final boss requires Stormbreaker weapon (checked via `TheStormbreaker` achievement in `zones/progression.rs`)
- **Boss enrage timer**: After 60 seconds of fighting a boss, it enrages and instantly kills the player. Emits a `BossEnrage` combat event (mapped to `TickEvent::BossEnrage`)

## Key Function: `update_combat()`

```rust
pub fn update_combat<R: Rng>(
    rng: &mut R,
    state: &mut GameState,
    delta_time: f64,
    bonuses: &CombatBonuses,
    achievements: &mut Achievements,
    derived: &DerivedStats,
    fracture_zone_cap: u32,
    loom_zone_cap: u32,
) -> Vec<CombatEvent>
```

Called from `core/tick.rs` each tick. Takes a generic `R: Rng` and a single unified `&CombatBonuses` that aggregates Haven, prestige, god item, sigil, and ascension bonuses. The `fracture_zone_cap` and `loom_zone_cap` parameters control boss defeat cycling for their respective zone ranges (passed through to `on_boss_defeated_with_cap()`). Returns `Vec<CombatEvent>` that tick.rs maps to `TickEvent` variants.

## Integration Points

- **Core** (`core/tick.rs`): Drives the per-tick game loop, builds unified `CombatBonuses` from all sources, applies `flat_hp` (from `PrestigeCombatBonuses`) to combat HP
- **Core** (`core/game_logic.rs`): Enemy spawning via zone-based generators, XP calculation, level-up logic
- **Character** (`character/derived_stats.rs`): Player base damage, defense, HP, crit stats
- **Character** (`character/combat_bonuses.rs`): `PrestigeCombatBonuses` struct with `from_rank()` constructor
- **Items** (`items/drops.rs`): Mob drops via `try_drop_from_mob()`, boss drops via `try_drop_from_boss()`
- **Zones** (`zones/progression.rs`): Zone-based stat lookup, boss definitions
- **Dungeon** (`dungeon/logic.rs`): Dungeon room combat with zone-scaled enemies
- **God Items** (`god_items/types.rs`): `equipped_god_item_*()` helper functions supply god item bonus values
- **Stormglass** (`stormglass/sigils.rs`): `SigilBonuses` computed from etched sigils, injected into `CombatBonuses`
- **UI** (`ui/combat_scene.rs`): HP bars, enemy sprites, visual effects

## Constants (from `core/constants.rs`)

- Player attack interval: 1.5s (15 ticks)
- Enemy attack intervals: 2.0s (normal), 1.8s (boss), 1.5s (zone boss), 1.6s (dungeon elite), 1.4s (dungeon boss)
- HP regen duration: 2.5s after kill
- XP per kill: 200-400 ticks of passive XP
- Boss kill tracking: 10 kills per subzone to trigger boss, 10 kills to retry after death
- Boss stat multipliers: Subzone (3.0/1.5/1.8), Zone (5.0/1.8/2.5), Dungeon Elite (2.2/1.5/1.6), Dungeon Boss (3.5/1.8/2.0)
