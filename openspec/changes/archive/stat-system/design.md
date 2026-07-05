> Backported design record. Sources: docs/archive/STAT_SYSTEM.md, docs/archive/plans/2026-01-31-stat-system-overhaul-design.md.

## STAT_SYSTEM.md

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

## 2026-01-31-stat-system-overhaul-design.md

# Classic Fantasy RPG Stat System Design

**Date:** 2026-01-31
**Status:** Approved
**Goal:** Transform the stat system to reflect classic fantasy RPG mechanics (D&D-inspired)

## Overview

This design overhauls the game's stat system from simple numeric values into a D&D-inspired attribute system with derived stats, while preserving the automated idle game progression that makes the game satisfying.

## Core Attribute System

### The Six Attributes

Characters have six core D&D attributes:
- **Strength (STR)** - Physical power, affects physical damage
- **Dexterity (DEX)** - Agility and reflexes, affects defense and critical hits
- **Constitution (CON)** - Health and endurance, affects max HP
- **Intelligence (INT)** - Magical power, affects magic damage
- **Wisdom (WIS)** - Learning and insight, affects XP gain rate
- **Charisma (CHA)** - Force of personality, enhances prestige bonuses

### Starting Values

All attributes start at 10 (representing an average human).

### Growth System

**Points per Level:** Gain 3 attribute points per level up

**Random Distribution:** Points automatically distribute randomly across all six attributes. This maintains idle automation while creating organic build diversity - no decision paralysis, no meta-gaming. Each playthrough naturally develops differently.

**Distribution Logic:**
- On level up, loop 3 times
- Pick random attribute (0-5)
- If attribute below cap, increment
- If at cap, pick another random attribute
- Ensures all 3 points distribute to non-capped attributes

### Attribute Caps

**Prestige-Scaling Caps:**
- Base game (prestige rank 0): cap = 20
- Each prestige rank: cap increases by 5
- Formula: `cap = 20 + (prestige_rank × 5)`
- Examples:
  - Prestige 0: cap 20
  - Prestige 1: cap 25
  - Prestige 2: cap 30
  - Prestige 5: cap 45

This gives prestige meaningful long-term value beyond XP multipliers.

## Modifier System

Attributes use D&D's modifier calculation:

```
modifier = (attribute - 10) / 2 (rounded down)
```

**Modifier Breakpoints:**
- 8-9 = -1
- 10-11 = +0
- 12-13 = +1
- 14-15 = +2
- 16-17 = +3
- 18-19 = +4
- 20-21 = +5

Power spikes occur every 2 attribute points, creating meaningful progression milestones.

## Derived Stats

All combat and progression stats are calculated from attribute modifiers:

### Combat Stats

**Max HP:**
```
Max HP = 50 + (CON_modifier × 10)
```
- 10 CON (+0): 50 HP
- 16 CON (+3): 80 HP
- 20 CON (+5): 100 HP

**Physical Damage:**
```
Physical Damage = 5 + (STR_modifier × 2)
```

**Magic Damage:**
```
Magic Damage = 5 + (INT_modifier × 2)
```

**Total Damage per Hit:**
```
Total Damage = Physical Damage + Magic Damage
```
Both STR and INT always contribute to damage output.

**Defense:**
```
Defense = 0 + (DEX_modifier × 1)
```
Reduces incoming damage by this flat amount per hit.

**Critical Hit Chance:**
```
Crit Chance = 5% + (DEX_modifier × 1%)
```
On crit, damage is doubled.

### Progression Stats

**XP Gain Multiplier:**
```
XP Multiplier = 1.0 + (WIS_modifier × 0.05)
```
Affects passive tick-based XP gain.
- 10 WIS (+0): 1.0× (no bonus)
- 20 WIS (+5): 1.25× XP gain

**Prestige Bonus:**
```
Prestige Multiplier = base_prestige_multiplier + (CHA_modifier × 0.1)
```
Stacks with prestige tier multipliers.
- Prestige rank 2 base: 2.25×
- With 16 CHA (+3): 2.25 + 0.3 = 2.55× total

## Combat System

### Auto-Battle Mechanics

**Attack Timing:**
- Both player and enemy attack every 1.5 seconds
- Attacks are simultaneous
- Combat happens automatically in the background

**Damage Calculation:**
- Player deals Total Damage (Physical + Magic combined)
- Roll for critical hit (DEX-based crit chance)
- If crit: damage × 2
- Enemy deals base damage
- Player reduces incoming damage by Defense value

**Combat Flow:**
1. Enemy spawns with full HP
2. Combat begins, attacks every 1.5s
3. Player or enemy HP reaches 0
4. If player dies: instant respawn at full HP, enemy resets
5. If enemy dies: player gains bonus XP, regenerates HP over 2-3s
6. New enemy spawns

### Procedural Enemy Generation

**Random Enemy Creation:**
- Procedurally generated fantasy names (syllable combinations)
- Stats scaled to player power level
- Variation: 80-120% of player stats

**Enemy Stat Formulas:**
```
Enemy HP = Player Max HP × random(0.8, 1.2)
Enemy Damage = calculated to make fights last 5-10 seconds on average
```

**Name Generation:**
Combine random syllables from lists:
- Prefixes: "Grizz", "Sav", "Dark", "Blood", "Bone", "Shadow"
- Roots: "led", "age", "en", "tooth", "claw", "fang"
- Suffixes: "Orc", "Troll", "Drake", "Crusher", "Render", "Maw"
- Examples: "Grizzled Orc", "Savage Bonecrusher", "Darken Fangmaw"

### HP and Death

**Player Death:**
- Instant respawn at full HP
- Enemy resets to full HP
- No penalty, no frustration
- Eventually win through stat growth

**Enemy Death:**
- Player gains bonus XP
- Player regenerates to full HP over 2-3 seconds
- New procedural enemy spawns

**HP Regeneration:**
- Only between fights
- 2-3 second rapid regeneration after kill
- Ensures each fight starts fresh

## Progression System

### XP Sources

**1. Passive Tick XP (Existing System Enhanced):**
```
XP per tick = base_xp_per_tick × prestige_multiplier × (1.0 + WIS_modifier × 0.05)
```
- Continues ticking every 100ms as before
- Modified by WIS for learning speed
- Modified by prestige (with CHA bonus)
- Ensures pure idle play still progresses

**2. Combat Bonus XP:**
```
Kill XP = passive_xp_per_tick × random(50, 100)
```
- Each enemy kill grants 5-10 seconds worth of passive XP
- Randomized per kill
- Active play roughly doubles XP rate
- Rewards engagement without punishing idle players

### Level Up Process

1. Character level increases
2. 3 attribute points randomly distribute to non-capped attributes
3. All derived stats recalculate automatically
4. New power level affects future enemy scaling

### Offline Progression

- Calculates passive XP gained while away (50% multiplier as before)
- No combat happens offline
- Return to passive gains accumulated
- Combat resumes immediately when back

## Prestige Integration

### Multiple Prestige Benefits

**1. Attribute Caps Increase:**
```
cap = 20 + (prestige_rank × 5)
```
Gives prestige long-term value for attribute growth.

**2. CHA Enhances Prestige Multiplier:**
```
Total Prestige Multiplier = base_tier_multiplier + (CHA_modifier × 0.1)
```
Makes CHA valuable throughout the game.

**3. Prestige Reset Behavior:**
- Reset to level 1
- All attributes reset to 10
- Keep prestige rank and total count
- Caps immediately reflect new prestige tier
- Faster progression due to multipliers

### Strategic Implications

Different random attribute distributions create different advantages:
- High WIS: Level faster through passive XP
- High CHA: Better prestige multipliers
- High STR/INT: More damage, faster kills
- High DEX: Better survivability and crits
- High CON: Tank through tough enemies

## UI and Display

### Attribute Display
```
STR: 14 (+2)    INT: 12 (+1)
DEX: 16 (+3)    WIS: 11 (+0)
CON: 13 (+1)    CHA: 10 (+0)
```

### Derived Stats Panel
```
HP: 60/60       Damage: 9-18 (5% crit)
Defense: 3      XP Rate: 1.0x
Prestige: 2.25x
```

### Combat Feed
```
Fighting: Grizzled Orc (HP: 45/68)
[You hit for 14 damage!]
[Grizzled Orc hits you for 5 damage!]
Your HP: 55/60
```

### Level Up Notification
```
LEVEL UP! Now level 15
+1 STR, +1 DEX, +1 WIS
```

### Prestige Cap Indicator
```
STR: 18/20    DEX: 20/20 [CAPPED]
(Prestige to increase caps to 25!)
```

### Enemy Info
```
Current Enemy: Savage Bonecrusher
Level ~14 | HP: 52/72
```

## Implementation Notes

### Code Structure Changes

**1. Stat Type Expansion:**
- Replace `pub type Stat = u32` with struct containing 6 attributes
- Add `fn modifier(attribute: u32) -> i32` for calculation
- Derive all combat stats on-demand from attributes

**2. Combat State Enhancement:**
- Add enemy struct with procedural name and stats
- Track player current HP and enemy current HP
- Add attack timers for player and enemy (1.5s)
- Add HP regeneration timer after kills

**3. Random Distribution:**
```rust
fn distribute_level_up_points(state: &mut GameState) {
    let mut points = 3;
    while points > 0 {
        let attr_index = rand::thread_rng().gen_range(0..6);
        let cap = 20 + (state.prestige_rank * 5);
        if state.attributes[attr_index] < cap {
            state.attributes[attr_index] += 1;
            points -= 1;
        }
    }
}
```

**4. Procedural Enemy Generation:**
```rust
fn generate_enemy(player_level: u32, player_stats: &PlayerStats) -> Enemy {
    let name = generate_fantasy_name();
    let variance = rand::thread_rng().gen_range(0.8..1.2);
    Enemy {
        name,
        max_hp: (player_stats.max_hp as f64 * variance) as u32,
        current_hp: /* same as max_hp */,
        damage: /* calculated for 5-10s fight */,
    }
}
```

**5. XP Calculations:**
```rust
fn xp_gain_per_tick(prestige_rank: u32, wis_modifier: i32) -> f64 {
    let base = BASE_XP_PER_TICK;
    let prestige_mult = prestige_multiplier(prestige_rank);
    let wis_mult = 1.0 + (wis_modifier as f64 * 0.05);
    base * prestige_mult * wis_mult
}

fn combat_kill_xp(passive_rate: f64) -> u64 {
    let ticks = rand::thread_rng().gen_range(50..100);
    (passive_rate * ticks as f64) as u64
}
```

### Backward Compatibility

Existing saves need migration:
- Old system: `stats: [u32; 4]` (STR, MAG, WIS, VIT)
- New system: `attributes: [u32; 6]` (STR, DEX, CON, INT, WIS, CHA)

**Migration strategy:**
```rust
// Map old stats to new attributes
attributes[0] = old_stats[0]; // STR -> STR
attributes[1] = 10;           // DEX (new, default)
attributes[2] = old_stats[3]; // VIT -> CON
attributes[3] = old_stats[1]; // MAG -> INT
attributes[4] = old_stats[2]; // WIS -> WIS
attributes[5] = 10;           // CHA (new, default)
```

## Testing Considerations

**Key scenarios to test:**
1. Modifier calculations at various attribute levels
2. Random distribution respects caps
3. Enemy scaling creates balanced fights
4. XP rates feel rewarding for both idle and active play
5. Prestige cap increases apply correctly
6. HP regeneration timing feels smooth
7. Combat timing at 1.5s intervals
8. Critical hits calculate correctly
9. Defense reduces damage as expected
10. Save migration from old format

## Success Metrics

- Attributes feel impactful at each breakpoint
- Combat is engaging but doesn't require constant attention
- Active play feels ~2× as rewarding as pure idle
- Prestige provides clear power progression
- Random builds create variety between playthroughs
- System is understandable without being overwhelming
