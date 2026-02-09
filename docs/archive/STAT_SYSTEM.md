# Stat System Documentation

## Overview

The stat system is a comprehensive character progression framework that replaces the old individual stat system with a unified attribute-based system. It features six core attributes, derived combat stats, prestige ranks, and dynamic combat.

## Architecture

### Core Modules

1. **Attributes** (`src/attributes.rs`)
   - Six core attributes: STR, DEX, CON, INT, WIS, CHA
   - Each attribute has a value (base 10) and a modifier (calculated as `(value - 10) / 2`)
   - Attributes are capped based on prestige rank

2. **Derived Stats** (`src/derived_stats.rs`)
   - Stats calculated from attributes:
     - Max HP: 50 + (CON_mod × 10)
     - Physical Damage: 5 + (STR_mod × 2)
     - Magic Damage: 5 + (INT_mod × 2)
     - Defense: DEX_mod (min 0)
     - Crit Chance: 5% + (DEX_mod × 1%)
     - XP Multiplier: 1.0 + (WIS_mod × 0.05)

3. **Combat** (`src/combat.rs`, `src/combat_logic.rs`)
   - Turn-based combat with enemies
   - Attack interval: 1.5 seconds
   - Critical hits deal 2x damage
   - Enemy stats scale with player power
   - HP regeneration after combat (2.5 seconds)

4. **Prestige** (`src/prestige.rs`)
   - Rank-based progression system
   - Resets character level but provides permanent bonuses
   - XP multiplier increases with prestige rank
   - Attribute caps increase with prestige rank

5. **Game Logic** (`src/game_logic.rs`)
   - XP progression with exponential curve
   - Random attribute distribution on level-up
   - Offline progression support
   - Combat XP bonuses

## Attributes

### The Six Attributes

| Attribute | Abbrev | Primary Effects |
|-----------|--------|-----------------|
| Strength | STR | Physical damage in combat |
| Dexterity | DEX | Defense, critical hit chance |
| Constitution | CON | Maximum HP |
| Intelligence | INT | Magic damage in combat |
| Wisdom | WIS | Passive XP gain rate |
| Charisma | CHA | Prestige XP multiplier bonus |

### Modifier Calculation

```rust
modifier = (attribute_value - 10) / 2
```

Examples:
- Attribute 10 → Modifier +0
- Attribute 16 → Modifier +3
- Attribute 20 → Modifier +5
- Attribute 8 → Modifier -1

### Attribute Caps

Attributes are capped based on prestige rank:

```rust
cap = 10 + prestige_rank * 5
```

| Prestige Rank | Attribute Cap |
|---------------|---------------|
| 0 | 10 |
| 1 | 15 |
| 2 | 20 |
| 3 | 25 |
| 5 | 35 |
| 10 | 60 |

## Derived Stats

All combat and progression stats are derived from attributes:

### Max HP
- Formula: `50 + (CON_mod × 10)`
- Example: CON 16 (+3) → 80 HP
- Minimum: 1 HP

### Physical Damage
- Formula: `5 + (STR_mod × 2)`
- Example: STR 16 (+3) → 11 damage
- Minimum: 1 damage

### Magic Damage
- Formula: `5 + (INT_mod × 2)`
- Example: INT 16 (+3) → 11 damage
- Minimum: 1 damage

### Total Damage
- Formula: `Physical Damage + Magic Damage`
- Used for combat calculations

### Defense
- Formula: `DEX_mod` (minimum 0)
- Reduces incoming damage: `damage_taken = enemy_damage - defense`
- Example: DEX 16 (+3) → 3 defense

### Crit Chance
- Formula: `5% + (DEX_mod × 1%)`
- Example: DEX 16 (+3) → 8% crit chance
- Critical hits deal 2x damage

### XP Multiplier
- Formula: `1.0 + (WIS_mod × 0.05)`
- Example: WIS 16 (+3) → 1.15x multiplier (15% bonus)
- Affects passive XP gain

## Combat System

### Combat Flow

1. **Enemy Spawning**: When no enemy exists and player is not regenerating
   - Enemy stats scale with player power
   - Enemy HP: 80-120% of player max HP
   - Enemy damage calculated for 5-10 second fights

2. **Combat Loop** (every 1.5 seconds):
   - Player attacks enemy
   - Roll for critical hit based on crit chance
   - Enemy takes damage (reduced by 0)
   - If enemy survives, enemy attacks player
   - Player takes damage (reduced by defense)

3. **Enemy Death**:
   - Award XP bonus (50-100 ticks worth, 5-10 seconds of passive XP)
   - Start HP regeneration timer (2.5 seconds)
   - Spawn new enemy after regeneration

4. **Player Death**:
   - Reset to full HP immediately
   - Lose all prestige ranks
   - Keep character level and attributes

### Combat XP

Combat kills award bonus XP:
```rust
xp_bonus = passive_xp_rate × random(50..100) ticks
```

At base rates (1 XP/tick), this means 50-100 bonus XP per kill.

## Experience and Leveling

### XP Curve

Experience required for next level follows an exponential curve:

```rust
xp_needed = 100.0 × (level ^ 1.5)
```

| Level | XP Required | Time at 10 XP/sec |
|-------|-------------|-------------------|
| 1 | 100 | 10 seconds |
| 2 | 282 | 28 seconds |
| 5 | 1,118 | 1.9 minutes |
| 10 | 3,162 | 5.3 minutes |
| 20 | 8,944 | 14.9 minutes |
| 50 | 35,355 | 58.9 minutes |
| 100 | 100,000 | 2.8 hours |

### Passive XP Gain

Base XP per tick calculation:

```rust
xp_per_tick = BASE_XP_PER_TICK × prestige_mult × wis_mult
```

Where:
- `BASE_XP_PER_TICK = 1.0`
- `prestige_mult = prestige_tier.multiplier + (CHA_mod × 0.1)`
- `wis_mult = 1.0 + (WIS_mod × 0.05)`

Example at Prestige Rank 1, WIS 16 (+3), CHA 14 (+2):
- Base: 1.0
- Prestige: 1.5 + 0.2 = 1.7
- WIS: 1.0 + 0.15 = 1.15
- Total: 1.0 × 1.7 × 1.15 = 1.955 XP/tick (19.55 XP/sec)

### Level-Up Distribution

On level-up, 3 attribute points are randomly distributed among non-capped attributes:
- Each point goes to a random attribute
- Respects attribute caps
- Never wastes points (max 100 attempts to place each point)

## Prestige System

### Prestige Tiers

| Rank | Name | Required Level | XP Multiplier |
|------|------|----------------|---------------|
| 0 | None | 0 | 1.0x |
| 1 | Bronze | 10 | 1.5x |
| 2 | Silver | 25 | 2.25x |
| 3 | Gold | 50 | 3.375x |
| 5 | Platinum | 75 | 7.59x |
| 10 | Diamond | 100 | 57.67x |
| 15 | Celestial | 150 | 437.89x |

Formula: `multiplier = 1.5 ^ rank`

### Prestige Mechanics

When you prestige:
1. Character level resets to 1
2. All attributes reset to 10
3. Character XP resets to 0
4. Prestige rank increases by 1
5. Attribute cap increases by 5

Effects:
- Higher attribute caps enable more powerful builds
- XP multiplier accelerates progression
- HP resets to new maximum
- Combat difficulty rebalances

### Charisma Bonus

Charisma provides an additional bonus to the prestige multiplier:

```rust
final_multiplier = base_multiplier + (CHA_mod × 0.1)
```

Example: Bronze rank (1.5x) with CHA 16 (+3):
- Base: 1.5
- CHA bonus: 0.3
- Final: 1.8x multiplier

## Save System

### Save Format

Saves use a binary format with magic number verification:
- Magic number: `0x49444C4552504700` ("IDLE RPG" in hex)
- Serialization: MessagePack format
- Save location: Platform-specific data directory

### Backward Compatibility

The save system includes migration from old stat-based saves:
1. Detects old save format (missing attributes field)
2. Converts old stats to attributes:
   - Averages old stat levels
   - Maps to new attribute system
   - Calculates approximate XP
3. Preserves prestige rank and timestamp

### Autosave

Game automatically saves every 30 seconds during gameplay.

## Offline Progression

When you return after being offline:

1. **Calculate Elapsed Time**:
   - Max: 7 days
   - Time beyond 7 days is capped

2. **Calculate Offline XP**:
   ```rust
   offline_xp = passive_xp_rate × elapsed_seconds × 0.5
   ```
   - 50% rate compared to active play
   - Based on your prestige rank and WIS/CHA at time of return

3. **Apply Level-Ups**:
   - All earned XP is applied
   - Level-ups occur automatically
   - Attributes distributed randomly

4. **Combat State**:
   - No combat during offline time
   - No combat XP bonuses
   - Player starts at full HP

## Game Constants

Located in `src/constants.rs`:

| Constant | Value | Description |
|----------|-------|-------------|
| `TICK_INTERVAL_MS` | 100 | Game tick interval (10 ticks/sec) |
| `BASE_XP_PER_TICK` | 1.0 | Base XP per tick before multipliers |
| `XP_CURVE_BASE` | 100.0 | Base XP for level calculation |
| `XP_CURVE_EXPONENT` | 1.5 | Exponential curve factor |
| `OFFLINE_MULTIPLIER` | 0.5 | 50% of normal XP while offline |
| `MAX_OFFLINE_SECONDS` | 604,800 | Max offline time (7 days) |
| `AUTOSAVE_INTERVAL_SECONDS` | 30 | Autosave every 30 seconds |
| `ATTACK_INTERVAL_SECONDS` | 1.5 | Time between combat rounds |
| `HP_REGEN_DURATION_SECONDS` | 2.5 | Time to fully regenerate HP |
| `COMBAT_XP_MIN_TICKS` | 50 | Min combat XP bonus |
| `COMBAT_XP_MAX_TICKS` | 100 | Max combat XP bonus |

## Testing

The stat system includes comprehensive unit tests:

### Attributes Tests
- Attribute creation and defaults
- Modifier calculations
- Get/set operations
- Increment operations

### Derived Stats Tests
- Base stat calculations
- High attribute values
- Low attribute values
- Prestige multiplier with CHA

### Combat Tests
- Combat state creation
- Enemy generation
- Enemy HP updates
- Damage calculations

### Game Logic Tests
- XP requirements per level
- XP gain calculations
- Level-up point distribution
- Attribute cap enforcement
- Spawn logic
- Combat XP bonuses

### Prestige Tests
- Prestige tier lookup
- Can prestige checks
- Prestige execution
- Rank calculations

### Save Manager Tests
- Save and load operations
- Migration from old format
- Non-existent save handling

Run tests with:
```bash
cargo test
```

## UI Components

### Stats Panel
- Displays character level and XP progress
- Shows all six attributes with values and modifiers
- Lists derived combat stats
- Shows prestige rank and multiplier
- Prestige button appears when eligible

### Combat Scene
- Visual enemy representation
- HP bars for player and enemy
- Real-time combat updates
- Enemy names dynamically generated
- Combat log events

## Future Enhancements

Potential additions to the stat system:

1. **Equipment System**
   - Weapons, armor, accessories
   - Attribute bonuses from gear
   - Rarity tiers

2. **Skills/Abilities**
   - Active combat skills
   - Passive bonuses
   - Skill trees

3. **Zone System**
   - Level-based zones (implemented but not active)
   - Zone-specific enemies
   - Environment effects

4. **Achievements**
   - Milestone tracking
   - Permanent bonuses
   - Prestige rank achievements

5. **Enhanced Combat**
   - Status effects
   - Multiple enemy types
   - Boss encounters

6. **Attribute Specialization**
   - Manual point allocation option
   - Build presets
   - Respec system

## Performance Considerations

The stat system is designed for efficiency:
- All calculations use simple arithmetic (no complex algorithms)
- Combat updates only every 1.5 seconds (not every tick)
- Derived stats cached in structs (not recalculated constantly)
- Save operations batched (autosave every 30 seconds)
- Offline calculation is O(1), not O(elapsed_time)

Expected performance:
- Memory: ~1KB per game state
- CPU: <1% on modern hardware
- Save file: <1KB per save

## Troubleshooting

### Common Issues

**Issue**: XP not increasing
- Check if player is dead (HP = 0)
- Verify prestige rank is set correctly
- Check WIS and CHA modifiers

**Issue**: Can't level up
- Verify XP exceeds threshold for current level
- Check if attributes are hitting caps
- Run `cargo test` to verify XP calculation

**Issue**: Combat too easy/hard
- Review attribute balance (STR, INT for damage)
- Check DEX for defense and crit
- Adjust CON for survivability

**Issue**: Save not loading
- Check save file exists in data directory
- Verify save file is not corrupted
- Try deleting save to start fresh

### Debug Mode

Run with debug logging:
```bash
RUST_LOG=debug cargo run
```

### Verification

Verify game state integrity:
```bash
cargo test --all
```

All 43 tests should pass with zero warnings.
