# Combat System

Turn-based auto-combat with enemy generation, damage calculation, and event-driven state transitions.

## Module Structure

```
src/combat/
├── mod.rs      # Public re-exports
├── types.rs    # Enemy struct, CombatState enum, combat result types
└── logic.rs    # Turn processing, damage calculation, HP regen, boss encounters
```

## Key Types

### `Enemy` (`types.rs`)
```rust
pub struct Enemy {
    pub name: String,
    pub level: u32,
    pub max_hp: u32,
    pub current_hp: u32,
    pub damage: u32,
    pub defense: u32,
    pub xp_reward: u32,
    pub is_boss: bool,
}
```

### `CombatState` (`types.rs`)
State machine for combat flow:
- **Idle**: No enemy, waiting for spawn
- **Fighting**: Active combat (turns every 1.5s)
- **Regen**: HP regenerating after kill (2.5s)
- **Dead**: Player died (triggers reset or dungeon exit)

## Combat Flow

1. **Enemy spawn**: Triggered by zone progression or dungeon room entry
2. **Turn loop**: Every 1.5s (15 ticks), both player and enemy take a turn
3. **Damage calculation**: `max(1, attacker_damage - defender_defense)` with crit rolls
4. **Critical hits**: Chance from DEX modifier, deals 2x damage
5. **Enemy death**: Awards XP, triggers item drop roll, enters Regen state
6. **Player death**:
   - In zone: Resets boss encounter (`fighting_boss=false`, `kills_in_subzone=0`), preserves prestige
   - In dungeon: Exits dungeon, no prestige loss

## Enemy Generation

Enemies are generated based on the current zone/subzone:
- Stats scale with zone difficulty and subzone position
- Boss enemies have multiplied HP/damage/defense
- Dungeon enemies scale based on dungeon level and room type (Elite > Combat)

## Boss Encounters

- After 10 kills in a subzone, the next enemy is the subzone boss
- Boss defined in `zones/data.rs` with specific stats
- Defeating boss advances to next subzone
- Death to boss resets kill counter (must kill 10 more)
- Zone 10 final boss requires Stormbreaker weapon (checked via `TheStormbreaker` achievement in `zones/progression.rs`)

## Integration Points

- **Core** (`core/game_logic.rs`): Drives the combat tick, manages state transitions
- **Character** (`character/derived_stats.rs`): Player damage, defense, HP, crit stats
- **Items** (`items/drops.rs`): Mob drops via `try_drop_from_mob()`, boss drops via `try_drop_from_boss()`
- **Zones** (`zones/progression.rs`): Enemy generation parameters, boss definitions
- **Dungeon** (`dungeon/logic.rs`): Dungeon room combat with room-specific enemies
- **UI** (`ui/combat_scene.rs`): HP bars, enemy sprites, visual effects

## Constants (from `core/constants.rs`)

- Attack interval: 1.5s (15 ticks)
- HP regen duration: 2.5s after kill
- XP per kill: 200-400 (varies by enemy level)
- Boss kill tracking: 10 kills per subzone to trigger boss
