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
