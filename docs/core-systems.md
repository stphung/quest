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
modifier = (attribute - 10) / 2  (integer division, can be negative)
```

Power spikes occur every 2 attribute points (e.g., 12 = +1, 14 = +2, 16 = +3).

### Growth

On level up, 3 attribute points are randomly distributed among non-capped attributes. This maintains idle automation while creating organic build diversity across playthroughs.

### Attribute Caps

```
cap = BASE_ATTRIBUTE_CAP + (prestige_rank * ATTRIBUTE_CAP_PER_PRESTIGE)
    = 20 + (prestige_rank * 5)
```

| Prestige | Cap |
|----------|-----|
| P0 | 20 |
| P1 | 25 |
| P5 | 45 |
| P10 | 70 |
| P20 | 120 |

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

- Enemy stats: Static zone-based values from `ZONE_ENEMY_STATS` table in `core/constants.rs`. Each zone defines `(base_hp, hp_step, base_dmg, dmg_step, base_def, def_step)` tuples; subzone depth adds incremental stats
- Procedurally generated fantasy names from syllable combinations

### Boss Enrage

Bosses enrage after 60 seconds of combat, increasing their damage output. This prevents indefinite stalling against bosses the player cannot defeat.

### Death Consequences

- **Death to regular enemy**: Instant respawn, no penalty
- **Death to boss**: Resets player to subzone 1 of the current zone. Preserves prestige
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
- Attribute caps (20 + 5 * new_rank)

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

### Rarity Tiers (6)

| Rarity | Color | Attribute Range | Affix Count |
|--------|-------|-----------------|-------------|
| Common | White | +1-2 | 0 |
| Magic | Blue | +2-4 | 1 |
| Rare | Yellow | +3-6 | 2-3 |
| Epic | Purple | +5-10 | 3-4 |
| Legendary | Orange | +8-15 | 4-5 |
| God (Mythic) | — | Fixed per item | Fixed per item |

God (Mythic) rarity is exclusive to the three god items (Asprika, Sleipnir, Megingjord). These have fixed stats and unique passives, not procedurally generated. God items always receive tier T9.

Attribute values scale with both ilvl (zone-based) and tier (T0-T9 quality roll). See Item Tier section below.

### Item Tier (Quality) System

Every item is independently assigned a quality tier (T0-T9) on an exponential drop curve. Tier multiplies stat values alongside ilvl: `effective_multiplier = ilvl_multiplier × tier_multiplier`. T9 (0.1% drop rate, 1.00x multiplier) equals the pre-tier power ceiling. T0 (38.0% drop rate, 0.40x multiplier) is the most common. God items always T9. Legacy saves default to T1.

### Drop System

- Base drop rate: 15% per kill
- Prestige bonus: +1% per prestige rank
- Haven Trophy Hall bonus: multiplicative on base chance
- Maximum total: 25% (reached at P10 without Haven, earlier with Haven)

**Mob rarity distribution** (base at P0):
- Common: 60%, Magic: 28%, Rare: 10%, Epic: 2%, Legendary: never (mob drops cannot be Legendary)
- Prestige bonus for rarity: +0.5%/rank, capped at 10% (cap reached at P20). Haven Workshop bonus: multiplicative on non-Common rates (up to +25%)
- Common floor: never drops below 20%

### Affix Types (9 + Unknown)

| Category | Affixes |
|----------|---------|
| Damage | DamagePercent, CritChance, CritMultiplier, AttackSpeed |
| Survivability | HPBonus, DamageReduction, HPRegen, DamageReflection |
| Progression | XPGain |

### Intrinsic Power Score

Every item has a character-independent power score (⚡) computed from raw attribute totals plus weighted affix values. Formula: `sum(attributes) + sum(affix_value × affix_weight)`. Displayed in cyan in the equipment panel and loot ticker. Uses the same affix weights as auto-equip (DmgPct 2.0, Crit 1.5, etc.).

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

## Tick Architecture

The game runs a 100ms tick loop. Each tick calls `game_tick()` in `src/core/tick.rs`, which orchestrates all game systems and returns a `TickResult`.

The tick implementation is split across several files:
- `tick.rs` -- Orchestrator: calls each stage in order, returns `TickResult`
- `tick_types.rs` -- `TickEvent` enum (35 variants) and `TickResult` struct
- `tick_stages.rs` -- Processing stages 4-6 and helper functions (`process_item_drop`, `process_discoveries`, etc.)
- `xp.rs` -- XP calculation, leveling logic, combat kill XP
- `discoveries.rs` -- Discovery rolls for dungeons, fishing spots, Haven, Soulforge
- `enemy_spawning.rs` -- Enemy generation and spawning (spawn_enemy_if_needed, try_discover_dungeon)
- `offline.rs` -- Offline XP progression
- `recent_drops.rs` -- RecentDrop struct and deque management
- `ticker.rs` -- XP rate sampling and rolling window

### game_tick() Signature

```rust
pub fn game_tick<R: Rng>(
    state: &mut GameState,
    tick_counter: &mut u32,
    haven: &mut Haven,
    enhancement: &mut EnhancementProgress,
    achievements: &mut Achievements,
    debug_mode: bool,
    rng: &mut R,
) -> TickResult
```

Generic `<R: Rng>` allows seeded RNG in tests (`ChaCha8Rng`) and `thread_rng()` in production.

### TickEvent and TickResult

`TickEvent` is an enum with 35 variants describing everything that can happen in a single tick. The presentation layer (`main.rs` via `tick_events.rs`) maps these to combat log entries and visual effects. Game logic never touches UI types. Defined in `tick_types.rs`.

```rust
pub struct TickResult {
    pub events: Vec<TickEvent>,
    pub leviathan_encounter: Option<u8>,
    pub achievements_changed: bool,
    pub haven_changed: bool,
    pub enhancement_changed: bool,
    pub god_items_changed: bool,
    pub achievement_modal_ready: Vec<AchievementId>,
}
```

**TickEvent categories**:
- Combat: `PlayerAttack`, `PlayerAttackBlocked`, `EnemyAttack`, `DamageReflected`, `RegenComplete`, `EnemyDefeated`, `PlayerDied`, `PlayerDiedInDungeon`
- Items: `ItemDropped` (rarity, tier, ilvl, power, slot, stats, equipped flag, from_boss flag)
- Zones: `SubzoneBossDefeated` (with `BossDefeatResult`)
- Dungeon: room entry, treasure, keys, boss unlock, completion, failure
- Fishing: messages, catches, item drops, rank-ups, Storm Leviathan
- Discovery: challenges, dungeons, fishing spots, Haven, Soulforge, Stormglass
- Stormglass: `StormglassEarned`, `SigilActivated`, `SigilExpired`
- Progress: `LeveledUp`, `AchievementUnlocked`

### Processing Stages

| Stage | What it does | File |
|-------|-------------|------|
| 1. Challenge AI | Ticks AI thinking for active Chess, Morris, Gomoku, or Go games | tick.rs |
| 2. Challenge discovery | Rolls for new challenge discovery (P1+ required, Haven bonus applied) | tick.rs |
| 3. Sync player HP | Recalculates DerivedStats and updates player_max_hp | tick.rs |
| 4. Dungeon exploration | Processes room entry, treasure, keys, boss unlock, completion/failure | tick_stages.rs |
| 5. Fishing | If fishing active: ticks session, handles catches/items/rank-ups/Leviathan, **returns early** (skips combat) | tick_stages.rs |
| 6. Combat | Maps CombatEvent to TickEvent, applies XP, handles kills/deaths, processes item drops and discoveries | tick_stages.rs |
| 7. Enemy spawn | Spawns enemy if no enemy and not regenerating | tick.rs |
| 8. Play time | Increments tick counter; at 10 ticks, increments play_time_seconds | tick.rs |
| 9. Achievement collection | Drains newly unlocked achievements into TickResult.events | tick.rs |
| 10. Haven discovery | Rolls for Haven discovery (P10+, no active content) | tick.rs |
| 11. Soulforge discovery | Rolls for Soulforge discovery (P15+, no active content) | tick.rs |
| 12. Achievement modal | Checks if 500ms accumulation window has elapsed for modal display | tick.rs |

**Important**: Stage 5 (fishing) returns early, skipping stages 6-7. Fishing and combat are mutually exclusive.

### Event Mapping (tick_events.rs)

`src/tick_events.rs` is a binary-only module (not part of `lib.rs`) that bridges pure game-logic events to UI types. It maps `TickEvent` variants to `add_log_entry()` calls and `VisualEffect` spawns. This keeps `tick.rs` free of UI imports while keeping `main.rs` focused on the game loop.

## Offline Progression Module

Offline progression is implemented in `src/core/offline.rs` as a self-contained module:

```rust
pub fn calculate_offline_xp(
    elapsed_seconds: i64,
    prestige_rank: u32,
    wis_modifier: i32,
    cha_modifier: i32,
    haven_offline_xp_percent: f64,
) -> f64

pub fn process_offline_progression(
    state: &mut GameState,
    haven_offline_xp_percent: f64,
) -> OfflineReport
```

`OfflineReport` contains elapsed_seconds, total_level_ups, xp_gained, level_before/after, and effective rates. Re-exported from `game_logic.rs` for backwards compatibility.

## Enhancement / Soulforge System

### Overview

Account-level equipment enhancement system that persists across all characters. Players enhance equipment slots (not individual items) by spending prestige ranks at the Soulforge. Enhancement levels provide cumulative damage/stat multipliers. Stored in `~/.quest/enhancement.json`.

### Discovery

- **Prestige gate**: Character must be P15+ (Transcendent tier)
- **Discovery**: Independent RNG roll per tick, chance scales with prestige rank:
  - `chance = 0.000014 + (prestige_rank - 15) * 0.000007`
  - Uses the same formula shape as Haven discovery but gated at P15 instead of P10
- **One-time**: Once discovered, accessible account-wide permanently via `[S]` key

### Mechanics

Enhancement operates on the 7 equipment slots (Weapon, Armor, Helmet, Gloves, Boots, Amulet, Ring). Each slot can be enhanced from +0 to +10. Enhancement levels are slot-based and persist across prestige resets and item changes.

### Success Rates

| Target Level | Success Rate | Cost (PR) | Fail Penalty |
|-------------|-------------|-----------|-------------|
| +1 to +4 | 100% | 1 PR each | None (safe) |
| +5 | 70% | 2 PR | -1 level |
| +6 | 55% | 3 PR | -1 level |
| +7 | 40% | 3 PR | -1 level |
| +8 | 30% | 4 PR | -1 level |
| +9 | 20% | 4 PR | -1 level |
| +10 | 10% | 5 PR | -2 levels |

**Soul Tithe**: Levels +5/+6/+7 offer an alternative guaranteed-success option at higher PR cost (4/6/8 PR respectively) for 100% success rate.

### Cumulative Bonus Multiplier

| Level | Bonus | Multiplier |
|-------|-------|-----------|
| +0 | 0% | 1.00x |
| +1 | +5% | 1.05x |
| +2 | +10% | 1.10x |
| +3 | +15% | 1.15x |
| +4 | +20% | 1.20x |
| +5 | +30% | 1.30x |
| +6 | +40% | 1.40x |
| +7 | +55% | 1.55x |
| +8 | +75% | 1.75x |
| +9 | +100% | 2.00x |
| +10 | +150% | 2.50x |

### Color Tiers

| Level Range | Color | Display |
|-------------|-------|---------|
| +0 | Gray | No prefix |
| +1 to +4 | White | "+N " prefix |
| +5 to +7 | Yellow | "+N " prefix |
| +8 to +9 | Magenta | "+N " prefix |
| +10 | Gold | "+N " prefix |

### Persistence

Enhancement state (`EnhancementProgress`) is saved to `~/.quest/enhancement.json`. The `enhancement_changed` flag in `TickResult` signals when the file needs to be saved.

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
| Dungeon discovery | 1% per kill |
| Fishing discovery | 5% per kill |
| Challenge discovery | 0.000014/tick (~2hr avg) |
| Haven discovery base | 0.000014/tick (P10+) |
| Haven discovery rank bonus | +0.000007/tick per rank above 10 |
| Soulforge discovery base | 0.000014/tick (P15+) |
| Soulforge discovery rank bonus | +0.000007/tick per rank above 15 |
| Prestige mult formula | `1.0 + 0.5 * rank^0.7` |
| Base max fishing rank | 30 (40 with Fishing Dock T4) |
