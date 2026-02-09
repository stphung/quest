# Core Systems Design

This document describes the foundational game systems as implemented. It consolidates the original design documents for the stat system, combat, zones, prestige, items, and characters.

## Attribute System

### Six Core Attributes (D&D-Inspired)

| Attribute | Abbrev | Effect |
|-----------|--------|--------|
| Strength | STR | Physical damage |
| Dexterity | DEX | Defense, critical hit chance |
| Constitution | CON | Maximum HP |
| Intelligence | INT | Magic damage |
| Wisdom | WIS | Passive XP gain rate |
| Charisma | CHA | Prestige XP multiplier bonus |

All attributes start at 10 (average human baseline).

### Modifier System

```
modifier = (attribute - 10) / 2  (integer division, min 0)
```

Power spikes occur every 2 attribute points (e.g., 12 = +1, 14 = +2, 16 = +3).

### Growth

On level up, 3 attribute points are randomly distributed among non-capped attributes. This maintains idle automation while creating organic build diversity across playthroughs.

### Attribute Caps

```
cap = 10 + (prestige_rank * 5)
```

| Prestige | Cap |
|----------|-----|
| P0 | 10 |
| P1 | 15 |
| P5 | 35 |
| P10 | 60 |

## Derived Stats

All combat and progression stats are calculated from attribute modifiers:

| Stat | Formula | Example (mod +3) |
|------|---------|-------------------|
| Max HP | `50 + (CON_mod * 10)` | 80 HP |
| Physical Damage | `5 + (STR_mod * 2)` | 11 |
| Magic Damage | `5 + (INT_mod * 2)` | 11 |
| Total Damage | Physical + Magic | 22 |
| Defense | `DEX_mod` (min 0) | 3 |
| Crit Chance | `5% + (DEX_mod * 1%)` | 8% |
| XP Multiplier | `1.0 + (WIS_mod * 0.05)` | 1.15x |

Critical hits deal 2x base damage (crit multiplier can be increased by equipment affixes). Defense reduces incoming damage as a flat subtraction.

### Equipment Affix Effects on Stats

| Affix | Effect |
|-------|--------|
| DamagePercent | `damage_mult *= 1.0 + (value / 100)` |
| CritChance | Adds flat crit chance |
| CritMultiplier | Adds to base 2.0x multiplier |
| AttackSpeed | `1.0 + (value / 100)` attack speed multiplier |
| HPBonus | Flat max HP increase |
| DamageReduction | `defense_mult *= 1.0 + (value / 100)` |
| HPRegen | `1.0 + (value / 100)` regen speed multiplier |
| DamageReflection | Reflects % of damage taken |
| XPGain | `xp_mult *= 1.0 + (value / 100)` |

## Experience and Leveling

### XP Curve

```
xp_needed = 100 * (level ^ 1.5)
```

| Level | XP Required |
|-------|-------------|
| 1 | 100 |
| 10 | 3,162 |
| 50 | 35,355 |
| 100 | 100,000 |

### XP Sources

**Passive tick XP:**
```
xp_per_tick = BASE_XP_PER_TICK * prestige_mult * wis_mult
            = 1.0 * (1.0 + 0.5 * rank^0.7 + CHA_mod * 0.1) * (1.0 + WIS_mod * 0.05)
```
Ticks run at 10/sec.

**Combat kill XP:**
```
ticks = random(200..=400)
base_xp = xp_per_tick * ticks
kill_xp = base_xp * (1.0 + haven_xp_gain_percent / 100)
```
Each kill awards 200-400 ticks worth of passive XP (20-40 seconds), modified by Haven Training Yard bonus.

### Offline Progression

Offline progression **simulates kills**, not just passive XP:

```
estimated_kills = (elapsed_seconds / 5.0) * 0.25
avg_xp_per_kill = xp_per_tick * 300   (average of 200-400 ticks)
base_xp = estimated_kills * avg_xp_per_kill
final_xp = base_xp * (1.0 + haven_offline_xp_percent / 100)
```

- Assumes 1 kill every 5 seconds (combat + regen time)
- Offline multiplier: 25% of online kill rate
- Cap: 7 days maximum
- Haven Hearthstone bonus applied multiplicatively

## Combat System

### Auto-Battle Flow

1. Enemy spawns when no enemy exists and player is not regenerating
2. Both sides attack every 1.5 seconds (base interval, reduced by AttackSpeed affixes)
3. Player deals Total Damage (with crit chance roll); enemy damage reduced by player Defense
4. On enemy death: award kill XP, begin HP regen (2.5 seconds base), then spawn next enemy
5. On player death: instant respawn at full HP, enemy resets

### Enemy Generation

- Enemy HP: 80-120% of player max HP
- Enemy damage: calibrated for 5-10 second fights
- Procedurally generated fantasy names from syllable combinations

### Death Consequences

- **Death to regular enemy**: Instant respawn, no penalty
- **Death to boss**: Resets boss encounter (fighting_boss=false, kills_in_subzone=0), preserves prestige
- **Death in dungeon**: Exits dungeon, no prestige loss

## Prestige System

### Multiplier Formula (Diminishing Returns)

```
multiplier = 1.0 + 0.5 * (rank as f64).powf(0.7)
```

| Rank | Multiplier | Per-Prestige Gain |
|------|------------|-------------------|
| P1 | 1.50x | +50% |
| P5 | 2.54x | +10% |
| P10 | 3.51x | +6% |
| P20 | 5.07x | +3% |
| P30 | 6.41x | +2% |

This formula provides strong early boosts that taper off, preventing late-game trivialization. The multiplier asymptotes around 6-7x.

### Charisma Bonus

```
final_multiplier = base_multiplier + (CHA_mod * 0.1)
```

### Prestige Tier Names and Level Requirements

| Rank | Name | Required Level |
|------|------|----------------|
| 1 | Bronze | 10 |
| 2 | Silver | 25 |
| 3 | Gold | 50 |
| 4 | Platinum | 65 |
| 5 | Diamond | 80 |
| 6 | Emerald | 90 |
| 7 | Sapphire | 100 |
| 8 | Ruby | 110 |
| 9 | Obsidian | 120 |
| 10 | Celestial | 130 |
| 11 | Astral | 140 |
| 12 | Cosmic | 150 |
| 13 | Stellar | 160 |
| 14 | Galactic | 170 |
| 15 | Transcendent | 180 |
| 16 | Divine | 190 |
| 17 | Exalted | 200 |
| 18 | Mythic | 210 |
| 19 | Legendary | 220 |
| 20+ | Eternal | 220 + (rank-19)*15 |

### Prestige Reset — What Changes

**Reset (complete wipe):**
- Character level → 1
- Character XP → 0
- All attributes → 10
- All equipment → empty (all 7 slots cleared)
- Zone progression → Zone 1, Subzone 1, 0 kills, no defeated bosses
- Active dungeon/fishing/minigame → cleared
- Combat state → fresh (HP reset to base 50)

**Preserved:**
- Prestige rank (incremented by 1)
- Total prestige count (incremented by 1)
- Character name and ID
- Fishing state (rank, total fish caught, legendary catches)
- Chess stats
- Haven (account-level, persists across all characters)
- Achievements (account-level)

**Recalculated:**
- Zone unlocks (based on new prestige rank — higher prestige unlocks more zones immediately)
- Attribute caps (10 + 5 * new_rank)

### Vault (Item Preservation)

The Haven Vault room allows preserving equipped items through prestige:
- T1: 1 item survives prestige
- T2: 3 items survive prestige
- T3: 5 items survive prestige

When prestiging with a Vault, the player selects which equipped items to keep. Those items are saved before reset and restored to their slots afterward.

## Zone System

### Structure

10 zones organized into 5 tiers, gated by prestige rank. Each zone has 3-4 subzones with a boss per subzone. An 11th post-game zone (The Expanse) is unlocked via the "StormsEnd" achievement after clearing Zone 10.

| Tier | Prestige | Zones | Subzones/Zone |
|------|----------|-------|---------------|
| 1 | P0 | Meadow, Dark Forest | 3 |
| 2 | P5 | Mountain Pass, Ancient Ruins | 3 |
| 3 | P10 | Volcanic Wastes, Frozen Tundra | 4 |
| 4 | P15 | Crystal Caverns, Sunken Kingdom | 4 |
| 5 | P20 | Floating Isles, Storm Citadel | 4 |
| Post | Achievement | The Expanse | 4 (cycles) |

### Complete Zone List

**Zone 1: Meadow** (P0) — Sunny Fields, Overgrown Thicket, Mushroom Caves → Sporeling Queen
**Zone 2: Dark Forest** (P0) — Forest Edge, Twisted Woods, Spider's Hollow → Broodmother Arachne
**Zone 3: Mountain Pass** (P5) — Rocky Foothills, Frozen Peaks, Dragon's Perch → Frost Wyrm
**Zone 4: Ancient Ruins** (P5) — Outer Sanctum, Sunken Temple, Sealed Catacombs → Lich King's Shade
**Zone 5: Volcanic Wastes** (P10) — Scorched Badlands, Lava Rivers, Obsidian Fortress, Magma Core → Infernal Titan
**Zone 6: Frozen Tundra** (P10) — Snowbound Plains, Glacier Maze, Frozen Lake, Permafrost Tomb → The Frozen One
**Zone 7: Crystal Caverns** (P15) — Glittering Tunnels, Prismatic Halls, Resonance Depths, Heart Crystal → Crystal Colossus
**Zone 8: Sunken Kingdom** (P15) — Coral Gardens, Drowned Streets, Abyssal Palace, Throne of Tides → The Drowned King
**Zone 9: Floating Isles** (P20) — Cloud Docks, Sky Bridges, Stormfront, Eye of the Storm → Tempest Lord
**Zone 10: Storm Citadel** (P20, requires Stormbreaker) — Lightning Fields, Thunder Halls, Generator Core, Apex Spire → The Undying Storm
**Zone 11: The Expanse** (StormsEnd achievement) — Void's Edge, Eternal Storm, Abyssal Rift, The Endless → Avatar of Infinity (cycles back to subzone 1)

### Subzone Progression

- 10 kills in a subzone triggers its boss
- Defeating the boss advances to the next subzone (or next zone)
- Zone 11 (The Expanse) cycles: after defeating the final subzone boss, returns to subzone 1 for infinite replay

### Stormbreaker Weapon Gate

Zone 10's final boss (The Undying Storm) requires the Stormbreaker weapon. Without it, the boss fight resets. Stormbreaker is obtained through the **StormForge** system (see [Secondary Systems — Haven](secondary-systems.md#stormforge-endgame-capstone)).

### Thematic Arc

```
Tier 1 — Nature's Edge:       Meadow → Dark Forest
Tier 2 — Civilization's Ruins: Mountain Pass → Ancient Ruins
Tier 3 — Elemental Forces:    Volcanic Wastes → Frozen Tundra
Tier 4 — Hidden Depths:       Crystal Caverns → Sunken Kingdom
Tier 5 — Ascending:           Floating Isles → Storm Citadel
Post-game:                     The Expanse (infinite cycling)
```

## Item System

### Equipment Slots (7)

Weapon, Armor, Helmet, Gloves, Boots, Amulet, Ring.

### Rarity Tiers (5)

| Rarity | Color | Attribute Range | Affix Count |
|--------|-------|-----------------|-------------|
| Common | White | +1-2 | 0 |
| Magic | Blue | +2-4 | 1 |
| Rare | Yellow | +3-6 | 2-3 |
| Epic | Purple | +5-10 | 3-4 |
| Legendary | Orange | +8-15 | 4-5 |

### Drop System

- Base drop rate: 15% per kill
- Prestige bonus: +1% per prestige rank (max +10% at P10)
- Haven Trophy Hall bonus: multiplicative on base chance
- Maximum total: 25%

**Rarity distribution** (base at P0):
- Common: 55%, Magic: 30%, Rare: 12%, Epic: 2.5%, Legendary: 0.5%
- Prestige bonus (+1%/rank, max 10%) and Haven Workshop bonus (max 25%) shift weight away from Common toward higher rarities
- Common floor: never drops below 10%

### Affix Types (9)

| Category | Affixes |
|----------|---------|
| Damage | DamagePercent, CritChance, CritMultiplier, AttackSpeed |
| Survivability | HPBonus, DamageReduction, HPRegen, DamageReflection |
| Progression | XPGain |

### Auto-Equip

Items are automatically equipped if they score higher than the current item using a weighted scoring system:
- Attributes weighted by character's current build (specialization bonus)
- Affix types weighted by category (damage > survivability > progression)
- Empty slots always equip the first item found

### Procedural Names

Items get procedurally generated names with prefixes and suffixes tied to their affixes and rarity. Common/Magic items get simple names; Rare+ items get fantasy names (e.g., "Cruel Greatsword of Flame").

## Character System

### Save Format

Individual JSON files per character stored in `~/.quest/`. Maximum 3 characters. Plain JSON with no checksum — relies on serde for structural validation on load.

```
~/.quest/
├── hero.json
├── warrior.json
└── mage_the_great.json
```

### Character Management

- **Create**: Name validation (1-16 chars, alphanumeric + spaces/hyphens/underscores), UUID generation
- **Delete**: Requires typing exact name to confirm
- **Rename**: Updates filename and character_name field
- **Select**: Startup screen shows character list with detailed stats preview

### Naming Rules

- Case-insensitive uniqueness check
- Names sanitized to lowercase with underscores for filenames
- Leading/trailing whitespace trimmed

## Key Constants

| Constant | Value |
|----------|-------|
| Tick interval | 100ms (10/sec) |
| Attack interval | 1.5s (base) |
| HP regen after kill | 2.5s (base) |
| Autosave | 30s |
| Update check interval | 30 min |
| Offline XP multiplier | 0.25 (25%) |
| Max offline time | 7 days (604,800s) |
| Base drop rate | 15% |
| Drop prestige bonus | +1%/rank (max +10%) |
| Drop cap | 25% |
| Boss spawn threshold | 10 kills in subzone |
| Base XP per tick | 1.0 |
| Combat XP per kill | 200-400 ticks |
| Dungeon discovery | 2% per kill |
| Fishing discovery | 5% per tick |
| Challenge discovery | 0.000014/tick (~2hr avg) |
