# Quest System Design

A comprehensive overview of Quest's game systems, architecture, and technical implementation.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Core Game Loop](#core-game-loop)
3. [Character & Attributes](#character--attributes)
4. [Combat System](#combat-system)
5. [Progression Systems](#progression-systems)
6. [Zone & World Structure](#zone--world-structure)
7. [Item System](#item-system)
8. [Secondary Systems](#secondary-systems)
9. [Challenge Minigames](#challenge-minigames)
10. [Haven (Base Building)](#haven-base-building)
11. [Endgame Content](#endgame-content)
12. [Infrastructure](#infrastructure)
13. [Data Storage](#data-storage)

---

## Architecture Overview

Quest is a terminal-based idle RPG built in Rust using Ratatui for UI rendering and Crossterm for terminal backend. The architecture follows an **event-driven pattern** where the core game logic produces typed events (`TickEvent`) and the presentation layer maps them to UI effects.

```
                         QUEST ARCHITECTURE

                    ┌──────────────────────────┐
                    │        main.rs            │
                    │  (Entry, Input, Render)   │
                    └─────────┬────────────────┘
                              │ calls
                              ▼
                    ┌──────────────────────────┐
                    │  core/tick.rs             │
                    │  game_tick<R: Rng>()      │
                    │  (Central Orchestrator)   │
                    └─────────┬────────────────┘
                              │ returns TickResult { events, flags }
                              ▼
                    ┌──────────────────────────┐
                    │  tick_events.rs           │
                    │  apply_tick_events()      │
                    │  (Event → UI Bridge)      │
                    └──────────────────────────┘

    game_tick() coordinates all game systems per tick:

    ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐
    │ Combat   │ │ Fishing  │ │ Dungeon  │ │Challenges│
    │  Engine  │ │  System  │ │  System  │ │ Minigames│
    └──────────┘ └──────────┘ └──────────┘ └──────────┘
    ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐
    │   Zones  │ │  Items   │ │  Haven   │ │Achievemts│
    └──────────┘ └──────────┘ └──────────┘ └──────────┘
    ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐
    │Soulforge │ │God Items │ │Stormglass│ │ The Deep │
    │(Enhance) │ │          │ │ (Sigils) │ │  (Mercs) │
    └──────────┘ └──────────┘ └──────────┘ └──────────┘
         │              │            │             │
         └──────────────┴────────────┴─────────────┘
                              │
                              ▼
                     ┌──────────────┐
                     │  GameState   │
                     │   (JSON)     │
                     └──────────────┘
```

### Key Architectural Patterns

- **Event-driven tick processing**: `game_tick()` returns a `TickResult` containing `Vec<TickEvent>` (45 event variants). The presentation layer maps events to combat log entries, visual effects, and overlays. Game logic has zero UI imports.
- **Generic RNG**: `game_tick<R: Rng>()` uses a generic type parameter because `rand::Rng` is not dyn-compatible. Production uses `thread_rng()`, tests use seeded `ChaCha8Rng` for determinism.
- **Haven bonus injection**: Haven bonuses are passed as explicit parameters to game systems rather than accessed globally, keeping modules decoupled.

### Key Dependencies

| Crate | Purpose |
|-------|---------|
| ratatui 0.30 | Terminal UI framework |
| crossterm 0.27 | Terminal backend |
| serde / serde_json | JSON serialization |
| rand | RNG for procedural systems |
| rand_chacha | Deterministic RNG for simulator and tests |
| chrono | Offline progression timing |
| chess-engine 0.1 | Chess minigame AI |
| ureq | HTTP client for auto-update |
| flate2 / tar | Archive extraction for updates |

---

## Core Game Loop

The game runs at **10 ticks per second** (100ms intervals). Each tick is processed by `game_tick()` in `src/core/tick.rs`, which orchestrates all game systems through a 14-stage pipeline.

```
                    GAME TICK PIPELINE (core/tick.rs)

    ┌─────────────────────────────────────────────────────────┐
    │  main.rs: Process Input → call game_tick() → Render     │
    └──────────────────────────┬──────────────────────────────┘
                               │
               game_tick() 14-stage pipeline:
                               │
    ┌──────────────────────────┴──────────────────────────────┐
    │  1. Challenge AI        Tick AI thinking for active game │
    │  2. Challenge Discovery Roll for new challenge (P1+)    │
    │  3. Sync Player HP      Recalculate DerivedStats        │
    │  4. Dungeon Exploration Process rooms, keys, boss       │
    │  5. Fishing             Tick session (EARLY RETURN)     │
    │  6. Combat              Attack cycle, kills, deaths     │
    │  7. Enemy Spawn         Spawn if idle + not regen       │
    │  8. Play Time           Increment tick/second counters  │
    │  9. Achievement Collect Drain newly unlocked into events│
    │ 10. Haven Discovery     Roll for Haven (P10+)           │
    │ 11. Soulforge Discovery Roll for Soulforge (P15+)      │
    │ 12. Deep Discovery      Trigger hook (no per-tick roll) │
    │ 13. Deep Missions       Tick active missions, events    │
    │ 14. Achievement Modal   Check 500ms accumulation window │
    └─────────────────────────────────────────────────────────┘
                               │
                               ▼
                    ┌──────────────────────┐
                    │  TickResult {        │
                    │    events: Vec,      │
                    │    achievements_     │
                    │      changed: bool,  │
                    │    haven_changed,    │
                    │    deep_changed,     │
                    │    leviathan_        │
                    │      encounter,      │
                    │    achievement_      │
                    │      modal_ready     │
                    │  }                   │
                    └──────────────────────┘
                               │
              main.rs maps TickResult to:
              - Combat log entries (via tick_events.rs)
              - Visual effects (damage numbers, flashes)
              - Achievement/Leviathan modal overlays
              - File IO (save achievements/haven)
```

**Stage 5 (Fishing) returns early**, skipping stages 6-7. Fishing and combat are mutually exclusive within a tick.

### Key Types

**`**`TickEvent`** (48 variants):
- Combat: `PlayerAttack`, `PlayerAttackBlocked`, `EnemyAttack`, `DamageReflected`, `RegenComplete`, `EnemyDefeated`, `BossEnrage`, `PlayerDied`, `PlayerDiedInDungeon`, `CombatRetreat`
- Items: `ItemDropped`
- Zones: `SubzoneBossDefeated`
- Dungeon: `DungeonRoomEntered`, `DungeonTreasureFound`, `DungeonKeyFound`, `DungeonBossUnlocked`, `DungeonBossDefeated`, `DungeonEliteDefeated`, `DungeonFailed`, `DungeonCompleted`
- Fishing: `FishingMessage`, `FishCaught`, `FishingItemFound`, `FishingRankUp`, `StormLeviathanCaught`
- Discovery: `ChallengeDiscovered`, `DungeonDiscovered`, `FishingSpotDiscovered`, `HavenDiscovered`, `SoulforgeDiscovered`, `DeepDiscovered`, `StormglassDiscovered`
- Stormglass: `StormglassSalvaged`, `StormglassDungeonCache`
- Deep: `DeepMissionComplete`, `DeepEventPending`, `DeepMercInjured`, `DeepMercLost`, `DeepBreakthrough`, `DeepGuildRankUp`
- Fracture Zones: `FractureRegionUnlocked`
- Ascension: `Ascended`
- Power Cores: `PowerCoreGranted`
- Achievements: `AchievementUnlocked`
- Level: `LeveledUp`

Each variant carries pre-formatted message strings with unicode escapes. The presentation layer uses them directly for log entries.

**`TickResult`**:
```rust
pub struct TickResult {
    pub events: Vec<TickEvent>,
    pub leviathan_encounter: Option<u8>,
    pub leviathan_lure_consumed: bool,
    pub leviathan_catch_miss: bool,
    pub achievements_changed: bool,
    pub haven_changed: bool,
    pub enhancement_changed: bool,
    pub god_items_changed: bool,
    pub deep_changed: bool,
    pub power_cores_changed: bool,
    pub achievement_modal_ready: Vec<AchievementId>,
}
```

### Key Constants

| Constant | Value | Purpose |
|----------|-------|---------|
| `TICK_INTERVAL_MS` | 100ms | Core loop speed |
| `ATTACK_INTERVAL_SECONDS` | 1.5s | Base combat speed |
| `HP_REGEN_DURATION_SECONDS` | 2.5s | Post-kill healing |
| `AUTOSAVE_INTERVAL_SECONDS` | 30s | Periodic save |
| `UPDATE_CHECK_INTERVAL_SECONDS` | 900s | Version polling |
| `UPDATE_CHECK_JITTER_SECONDS` | 300s | Spread API requests |

---

## Character & Attributes

### Six Core Attributes (D&D-Inspired)

| Attribute | Abbrev | Primary Effect |
|-----------|--------|----------------|
| Strength | STR | Physical damage |
| Dexterity | DEX | Defense, crit chance |
| Constitution | CON | Maximum HP |
| Intelligence | INT | Magic damage |
| Wisdom | WIS | Passive XP rate |
| Charisma | CHA | Prestige multiplier |

### Modifier System

```
modifier = (attribute - 10) / 2  (integer division)
```

All attributes start at 10 (average human baseline). Power spikes occur at 12, 14, 16, etc.

### Attribute Caps

```
cap = 20 + (prestige_rank x 5)
```

| Prestige | Cap | Example |
|----------|-----|---------|
| P0 | 20 | Starting character |
| P5 | 45 | Mid-game |
| P10 | 70 | Haven unlock |
| P20 | 120 | Late-game |

### Derived Stats

| Stat | Formula |
|------|---------|
| Max HP | 50 + (CON_mod x 10) |
| Physical Damage | 5 + (STR_mod x 2) |
| Magic Damage | 5 + (INT_mod x 2) |
| Defense | DEX_mod (min 0) |
| Crit Chance | 5% + (DEX_mod x 1%) |
| XP Multiplier | 1.0 + (WIS_mod x 0.05) |

---

## Combat System

### Auto-Battle Flow

```
                    COMBAT STATE MACHINE

    ┌──────────────┐
    │   IDLE       │<────────────────────────────────┐
    │ (No Enemy)   │                                 │
    └──────┬───────┘                                 │
           │ spawn                                   │
           v                                         │
    ┌──────────────┐                                 │
    │   FIGHTING   │<───────┐                        │
    │              │        │                        │
    └──────┬───────┘        │                        │
           │                │                        │
     ┌─────┴─────┐          │                        │
     v           v          │                        │
┌─────────┐ ┌─────────┐     │                        │
│ Player  │ │ Enemy   │     │                        │
│ Attacks │ │ Attacks │     │                        │
└────┬────┘ └────┬────┘     │                        │
     │           │          │                        │
     v           v          │                        │
  [Damage]    [Damage]      │                        │
     │           │          │                        │
     │      ┌────┴────┐     │                        │
     │      v         v     │                        │
     │  [Player   [Enemy    │                        │
     │   Dies]    Dies]     │                        │
     │      │         │     │                        │
     │      v         v     │                        │
     │  [Respawn] [+XP/Item]│                        │
     │      │         │     │                        │
     │      │         v     │                        │
     │      │  ┌──────────────┐                      │
     │      │  │ REGENERATING │                      │
     │      │  │  (2.5 sec)   │                      │
     │      │  └──────┬───────┘                      │
     │      │         │ healed                       │
     └──────┴─────────┴──────────────────────────────┘
```

### Combat Mechanics

- **Attack Interval**: 1.5 seconds (base), reduced by AttackSpeed affixes
- **Critical Hits**: 2x base damage (increased by CritMultiplier affixes)
- **Defense**: Flat damage reduction
- **Double Strike**: Haven War Room bonus (10-35% chance for extra attack)

### Enemy Generation

- Stats: Static zone-based values from `ZONE_ENEMY_STATS` table in `core/constants.rs`. Each zone defines `(base_hp, hp_step, base_dmg, dmg_step, base_def, def_step)` tuples; subzone depth adds incremental stats
- Names: Procedurally generated from syllable combinations

---

## Progression Systems

### XP Curve

```
xp_needed = 100 x level^1.5
```

| Level | XP Required |
|-------|-------------|
| 1 | 100 |
| 10 | 3,162 |
| 50 | 35,355 |
| 100 | 100,000 |

### XP Sources

**Passive Tick XP:**
```
xp_per_tick = BASE_XP_PER_TICK x prestige_mult x wis_mult
            = 1.0 x (1.0 + 0.5 * rank^0.7 + CHA_mod * 0.1) x (1.0 + WIS_mod * 0.05)
```

**Combat Kill XP:**
```
ticks = random(200..=400)
base_xp = xp_per_tick x ticks
kill_xp = base_xp x (1.0 + haven_xp_gain_percent / 100)
```
Each kill awards 200-400 ticks worth of passive XP (20-40 seconds), modified by Haven Training Yard bonus.

### Offline Progression

Offline XP simulates kills at a reduced rate:
```
estimated_kills = (elapsed_seconds / 5.0) x 0.25
xp_per_kill = xp_per_tick x 300  (average of 200-400)
base_xp = estimated_kills x xp_per_kill
final_xp = base_xp x (1.0 + haven_offline_xp_percent / 100)
```
- Assumes 1 kill every 5 seconds (combat + regen time)
- Offline multiplier: 25% of online kill rate
- Cap: 7 days maximum
- Haven Hearthstone bonus applied multiplicatively

### Prestige System

#### Multiplier Formula (Diminishing Returns)

```
base_multiplier = 1.0 + 0.5 x rank^0.7
final_multiplier = base_multiplier + (CHA_mod x 0.1)
```

| Rank | Multiplier | Gain |
|------|------------|------|
| P1 | 1.50x | +50% |
| P5 | 2.54x | +10%/rank |
| P10 | 3.51x | +6%/rank |
| P20 | 5.07x | +3%/rank |

#### Prestige Tiers

| Tier | Ranks | Names | Required Levels |
|------|-------|-------|-----------------|
| Metals | 1-4 | Bronze, Silver, Gold, Platinum | 10, 25, 50, 65 |
| Gems | 5-9 | Diamond, Emerald, Sapphire, Ruby, Obsidian | 80, 90, 100, 110, 120 |
| Cosmic | 10-14 | Celestial, Astral, Cosmic, Stellar, Galactic | 130, 140, 150, 160, 170 |
| Divine | 15-19 | Transcendent, Divine, Exalted, Mythic, Legendary | 180, 190, 200, 210, 220 |
| Eternal | 20+ | Eternal | 220 + (rank-19)x15 |

#### Prestige Reset

**Reset (complete wipe):**
- Character level to 1, XP to 0
- All attributes to 10
- All equipment cleared (all 7 slots)
- Zone progression to Zone 1, Subzone 1, 0 kills
- Active dungeon/fishing/minigame cleared
- Combat state reset (HP to base 50)

**Preserved:**
- Prestige rank (incremented by 1)
- Total prestige count
- Character name and ID
- Fishing state (rank, total fish caught, legendary catches)
- Chess stats
- Haven (account-level)
- Achievements (account-level)

**Recalculated:**
- Zone unlocks (based on new prestige rank)
- Attribute caps (20 + 5 x new_rank)

---

## Zone & World Structure

### Zone Layout

```
Tier 1 (P0)          Tier 2 (P5)          Tier 3 (P10)
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│   Meadow    │      │ Mountain    │      │  Volcanic   │
│  (3 subs)   │      │   Pass      │      │   Wastes    │
└─────────────┘      │  (3 subs)   │      │  (4 subs)   │
┌─────────────┐      └─────────────┘      └─────────────┘
│ Dark Forest │      ┌─────────────┐      ┌─────────────┐
│  (3 subs)   │      │  Ancient    │      │   Frozen    │
└─────────────┘      │   Ruins     │      │   Tundra    │
                     │  (3 subs)   │      │  (4 subs)   │
                     └─────────────┘      └─────────────┘

Tier 4 (P15)         Tier 5 (P20)         Post-Game (Z11)
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│  Crystal    │      │  Floating   │      │    The      │
│  Caverns    │      │   Isles     │      │  Expanse    │
│  (4 subs)   │      │  (4 subs)   │      │ (inf cycle) │
└─────────────┘      └─────────────┘      └─────────────┘
┌─────────────┐      ┌─────────────┐
│   Sunken    │      │   Storm     │
│  Kingdom    │      │  Citadel    │
│  (4 subs)   │      │  (4 subs)   │
└─────────────┘      └─────────────┘
                           ^
                    [Stormbreaker
                      Required]

Fracture Zones (Z12-30, unlocked by Deep layer breakthroughs, 5 subzones each):
Ch.1 The Red Fault (Deep L3):   Z12 Splintered Rim, Z13 Ember Ravine, Z14 Heart of the Fault
Ch.2 The Mirror Scar (Deep L7): Z15 Shard Fields, Z16 Refraction Steps, Z17 Hall of Second Suns
Ch.3 The Black Mouth (Deep L12):Z18 Ashen Verge, Z19 Throat of the World, Z20 The Black Mouth
Ch.4 The Hollow Throne (Deep L18): Z21 Sunken Processional, Z22 The Pale Archive, Z23 The Hollow Throne
Ch.5 The Wailing Reach (Deep L25): Z24 The Stillborn Sea, Z25 Resonance Fault, Z26 The Wailing Reach
Ch.6 The Origin Wound (Deep L30): Z27 The Scar Root, Z28 Echoing Abyss, Z29 Threshold of Silence, Z30 The Origin Wound
```

### Subzone Progression

- 10 kills per subzone triggers boss
- Boss defeat advances to next subzone/zone
- Zone 11 (The Expanse) cycles infinitely until fracture zones unlock
- Fracture zones: only the current cap zone cycles; all others advance forward
- Enemy stats scale 1.6x per zone from Zone 11 base (FRACTURE_ZONE_STAT_MULTIPLIER)

---

## Item System

### Equipment Slots (7)

Weapon, Armor, Helmet, Gloves, Boots, Amulet, Ring

### Rarity Tiers

| Rarity | Color | Attribute Bonus | Affixes |
|--------|-------|-----------------|---------|
| Common | White | +1-2 | 0 |
| Magic | Blue | +2-4 | 1 |
| Rare | Yellow | +3-6 | 2-3 |
| Epic | Purple | +5-10 | 3-4 |
| Legendary | Orange | +8-15 | 4-5 |
| God (Mythic) | — | Fixed per item | Fixed per item |

God (Mythic) rarity is reserved for the three god items (Asprika, Sleipnir, Megingjord). These are not generated procedurally — they have fixed stats and unique passives. God items always receive tier T9. See [God Items](#god-items) below.

Attribute bonuses scale with both ilvl (zone-based) and tier (T0-T9 quality roll). The values above are approximate base ranges at ilvl 10. See [Item Tier System](#item-tier-quality-system) below for tier details.

### Drop Rates

**Mob drops:**
- Base chance: 15% per kill
- Prestige bonus: +1% per rank (capped at 25% total)
- Haven Trophy Hall: multiplicative bonus on base chance
- Maximum rarity: Epic

**Boss drops:**
- Guaranteed drop on every boss kill
- Can include Legendary rarity (2% normal boss, 5% Zone 10 final boss)

**Rarity bonuses:**
- Prestige: +0.5% per rank toward higher rarities (capped at +10%, reaches cap at P20)
- Haven Workshop: multiplicative on non-Common rates (up to +25%)
- Common floor: never drops below 20%

### Item Level Scaling

```
item_level = zone_id x 10
```
Zone 1 = ilvl 10, Zone 10 = ilvl 100. Higher ilvl items have proportionally stronger attribute bonuses and affix values (1.0x at ilvl 10 to 4.0x at ilvl 100).

### Item Tier (Quality) System

Every item is independently assigned a quality tier from T0 (worst) to T9 (best). Tier is rolled on an exponential drop curve and multiplies stat values alongside the ilvl multiplier. T9 equals the pre-tier power ceiling, so most drops are weaker than before, making high-tier finds exciting.

**Tier Distribution:**

| Tier | Drop Rate | Stat Multiplier |
|------|-----------|----------------|
| T0 | 38.0% | 0.40x |
| T1 | 24.0% | 0.47x |
| T2 | 15.0% | 0.54x |
| T3 | 10.0% | 0.61x |
| T4 | 6.0% | 0.68x |
| T5 | 3.5% | 0.74x |
| T6 | 2.0% | 0.80x |
| T7 | 1.0% | 0.86x |
| T8 | 0.4% | 0.93x |
| T9 | 0.1% | 1.00x |

**Effective stat multiplier**: `ilvl_multiplier × tier_multiplier`. For example, an ilvl 100 T9 item has 4.0x stats (same as pre-tier ceiling), while an ilvl 100 T0 item has only 1.6x stats.

God items always receive T9. Legacy saves default to T1 for backward compatibility.

### Affix Types (9 + Unknown)

| Category | Affixes |
|----------|---------|
| Damage | DamagePercent, CritChance, CritMultiplier, AttackSpeed |
| Survival | HPBonus, DamageReduction, HPRegen, DamageReflection |
| Utility | XPGain |

### Intrinsic Power Score

Every item exposes a character-independent power score (⚡) via `Item::power()`: `sum(attributes) + sum(affix_value × affix_weight)`. Displayed in cyan in the equipment panel and loot ticker. Shared affix weights with auto-equip (DmgPct 2.0, Crit 1.5, DR 1.3, AtkSpd 1.2, Regen/XP 1.0, Reflect 0.8, HP 0.5).

### Auto-Equip

Items are automatically equipped if they score higher than the current item using a weighted scoring system. Attributes are weighted by the character's current build (specialization bonus), affix types by category (damage > survivability > progression). Empty slots always equip the first item found.

---

## Secondary Systems

### Fishing

```
                  FISHING STATE MACHINE

   [Combat Tick]
        │
        v
   5% Discovery ──No──> [Continue Combat]
        │
       Yes
        v
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   CASTING    │────>│   WAITING    │────>│   REELING    │
│  (0.5-1.5s)  │     │  (1.0-8.0s)  │     │  (0.5-3.0s)  │
└──────────────┘     └──────────────┘     └──────┬───────┘
                                                 │
                                                 v
                                          [Catch Fish]
                                                 │
                                    ┌────────────┴────────────┐
                                    v                         v
                             [Session End?]            [Next Fish]
                                    │                         │
                                   Yes                        │
                                    │                         │
                                    v                         │
                             [Return to                       │
                               Combat]<───────────────────────┘
```

**Ranks:** 40 total across 8 tiers (base cap 30, +10 with FishingDock T4)

**Fish Rarities:** Common (60%), Uncommon (25%), Rare (10%), Epic (4%), Legendary (1%)

**Item Drops from Fishing:** Common/Uncommon: 5%, Rare: 15%, Epic: 35%, Legendary: 75%

**Haven Bonuses:** Garden reduces fishing timers, Fishing Dock provides double fish chance and extends max rank to 40.

### Dungeons

- 1% discovery chance per kill (overworld only)
- Procedurally generated connected rooms on a grid
- 5 sizes: Small 5x5, Medium 7x7, Large 9x9, Epic 11x11, Legendary 13x13
- Size based on character level and prestige rank (with 20% random variation)
- Room types: Entrance, Combat (60%), Treasure (20%), Elite (15%), Boss (5%)
- Key system: Elite guardian drops the key to unlock Boss room
- Safe death (no prestige loss, exits dungeon)
- Fog of war: rooms start Hidden, become Revealed when adjacent to visited rooms

### God Items

Three Norse mythology-themed endgame items with unique combat passives and non-combat bonuses. Rarity: Mythic (displayed as "God"), above Legendary.

| Item | Slot | Passive | Combat Effect |
|------|------|---------|--------------|
| Asprika | Armor | Divine Bulwark | 30% damage reduction (after defense) |
| Sleipnir | Boots | Windborne | 100% attack speed bonus |
| Megingjord | Ring | Giant's Might | 150% damage bonus |

Sleipnir also provides non-combat bonuses: 50% regen delay reduction, 50% dungeon movement speed, 50% fishing timer reduction.

All god items have ilvl 100, +40% XP affix, and high fixed attribute bonuses (+40 primary, +20 secondary). Discovery/forging system not yet implemented (tracked in issue #235); currently available via debug menu.

### The Deep (Mercenary Expeditions)

Endgame system gated at P15+. Players recruit and manage a mercenary company, sending squads on wall-clock time missions (2-24h real time) into a vast underground structure. See [Secondary Systems — The Deep](secondary-systems.md#the-deep-mercenary-expedition-system) for full details. Detailed implementation docs in `src/deep/CLAUDE.md`.

---

## Challenge Minigames

### Discovery

- 0.000014 per tick (~2 hour average)
- Requires P1+ (not in dungeon, fishing, or another minigame)
- Haven Library bonus: up to +50%
- Weighted distribution (total weight 180): 10 challenge types with weights favoring quick games (Rune 30, Minesweeper 28, Snake 22, Flappy Bird 20, Sigil Surge 20, JezzBall 18, Gomoku 15, Morris 12, Chess 8, Go 7)

### Games & AI

| Game | Algorithm | Difficulties |
|------|-----------|--------------|
| Chess | Minimax (1-3 ply) | 500-1350 ELO |
| Go | MCTS (500-20k sims) | 20-12 kyu |
| Gomoku | Minimax + alpha-beta (2-5 ply) | - |
| Morris | Minimax + alpha-beta (2-5 ply) | - |
| Minesweeper | N/A (puzzle) | 9x9 to 20x16 |
| Rune | N/A (deduction) | 60-32,768 combos |
| Snake | N/A (action) | 10-25 food, 200-90ms |
| Flappy Bird | N/A (action) | Gap 7-4 rows, 3 lives |
| JezzBall | N/A (action) | 2-5 balls, 3 lives |
| Sigil Surge | N/A (action-puzzle) | 6×12 grid, 5 colors, 3 lives |

All challenges use 4 difficulty levels: Novice, Apprentice, Journeyman, Master.

### Rewards

| Game | Novice | Apprentice | Journeyman | Master |
|------|--------|------------|------------|--------|
| Chess | +1 PR | +2 PR | +3 PR | +5 PR |
| Go | +1 PR | +2 PR | +3 PR | +5 PR |
| Gomoku | +75% XP | +100% XP | +1 PR, +50% XP | +2 PR, +100% XP |
| Morris | +50% XP | +100% XP | +150% XP | +1 FR, +200% XP |
| Minesweeper | +50% XP | +75% XP | +100% XP | +1 PR, +200% XP |
| Rune | +25% XP | +50% XP | +1 FR, +75% XP | +1 PR, +2 FR |
| Snake | +25% XP | +75% XP | +100% XP | +1 PR, +100% XP |
| Flappy Bird | +50% XP | +100% XP | +1 PR, +75% XP | +2 PR, +150% XP, +1 FR |
| JezzBall | +25% XP | +75% XP | +1 PR, +100% XP | +2 PR, +100% XP |
| Sigil Surge | +50% XP | +100% XP | +1 PR, +75% XP | +2 PR, +150% XP, +1 FR |

PR = Prestige Rank, FR = Fishing Rank, XP% = percentage of current level's XP requirement. Challenge wins also award Stormglass currency (see [Secondary Systems](secondary-systems.md#stormglass-system)).

### Forfeit Pattern

All interactive minigames: first Esc sets `forfeit_pending`, second Esc confirms, any other key cancels.

---

## Haven (Base Building)

### Overview

Account-level skill tree unlocked at P10+. Spend prestige ranks to build rooms. Discovery uses an independent RNG roll per tick:
```
chance = 0.000014 + (prestige_rank - 10) x 0.000007
```

### Skill Tree

```
                         [Hearthstone]
                        /              \
                Combat Branch        QoL Branch
                     |                    |
                  Armory              Bedroom
                  /    \              /    \
          Training    Trophy     Garden   Library
            Yard       Hall        |        |
              |          |     Fish Dock  Workshop
          Watchtower  Alchemy     \       /
              \       Lab          Vault
               \     /           (capstone)
               War Room
              (capstone)
                   \               /
                    \             /
                     [StormForge]
                  (ultimate capstone)
```

### Room Bonuses

| Room | Bonus Type | T1 | T2 | T3 | T4 |
|------|-----------|-----|-----|-----|-----|
| Hearthstone | Offline XP % | +25% | +50% | +100% | -- |
| Armory | Damage % | +5% | +10% | +25% | -- |
| Training Yard | XP Gain % | +5% | +10% | +30% | -- |
| Trophy Hall | Drop Rate % | +5% | +10% | +15% | -- |
| Watchtower | Crit Chance % | +5% | +10% | +20% | -- |
| Alchemy Lab | HP Regen % | +25% | +50% | +100% | -- |
| War Room | Double Strike % | +10% | +20% | +35% | -- |
| Bedroom | Regen Delay Reduction | -15% | -30% | -50% | -- |
| Garden | Fishing Timer Reduction | -10% | -20% | -40% | -- |
| Library | Challenge Discovery % | +20% | +30% | +50% | -- |
| Fishing Dock | Double Fish Chance % | +25% | +50% | +100% | +10 Max Rank |
| Workshop | Item Rarity % | +10% | +15% | +25% | -- |
| Vault | Items Survive Prestige | 1 | 3 | 5 | -- |
| StormForge | Stormbreaker Access | Yes | -- | -- | -- |

### Costs

Players spend actual prestige ranks from the contributing character. Costs scale by depth in the skill tree:

| Depth | T1 | T2 | T3 |
|-------|-----|-----|-----|
| 0 (Hearthstone) | 1 | 2 | 3 |
| 1 (Armory, Bedroom) | 1 | 3 | 5 |
| 2-3 (mid-tree) | 2 | 4 | 6 |
| 4 (capstones) | 3 | 5 | 7 |
| Fishing Dock T4 | -- | -- | -- | 10 |
| StormForge | 25 (T1 only) | -- | -- |

---

## Endgame Content

### Stormbreaker Quest Chain

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Reach      │     │  Catch      │     │  Build      │
│  Fishing    │────>│  Storm      │────>│  Full       │
│  Rank 40    │     │  Leviathan  │     │  Haven      │
└─────────────┘     └─────────────┘     └──────┬──────┘
      │                    │                   │
      │                    │                   v
      │                    │           ┌─────────────┐
      v                    v           │  Build      │
 [Requires           [10 encounters    │  StormForge │
  FishingDock         then 25%         │  (25 PR)    │
  T4]                 catch rate]      └──────┬──────┘
                                              │
                                              v
                                       ┌─────────────┐
                                       │   Forge     │
                                       │ Stormbreaker│
                                       │  (25 PR)    │
                                       └──────┬──────┘
                                              │
                                              v
                                       ┌─────────────┐
                                       │  Defeat     │
                                       │ The Undying │
                                       │   Storm     │
                                       └──────┬──────┘
                                              │
                                              v
                                       ┌─────────────┐
                                       │  Unlock     │
                                       │    The      │
                                       │  Expanse    │
                                       └─────────────┘
```

### Storm Leviathan Hunt

Progressive 10-encounter hunt at Fishing Rank 40:
1. Each legendary fish catch at rank 40+ rolls against encounter chances (non-monotonic narrative arc): 5%, 3%, 4%, 5%, 4%, 3%, 2%, 1.5%, 1%, 0.8%
2. After 10 encounters: 25% catch chance per legendary fish
3. Catching awards 10,000-15,000 XP and unlocks StormLeviathan achievement

### The Expanse (Zone 11)

- Infinite cycling post-game zone
- 4 subzones: Void's Edge, Eternal Storm, Abyssal Rift, The Endless
- Final boss: Avatar of Infinity (cycles back to subzone 1)
- Unlocked via StormsEnd achievement (after defeating The Undying Storm)

### The Deep (Mercenary Expeditions)

Endgame mercenary expedition system discovered on the first Expanse cycle boss kill (`BossDefeatResult::ExpanseCycle`, i.e. The Endless) at P15+. Players recruit mercenaries across 5 archetypes and 4 quality tiers, then send them on missions into a 25-layer underground structure (plus infinite Void scaling). Missions run on wall-clock time (2-24h). Two-tier persistence: guild rank, cleared layers, and infrastructure survive prestige; mercenaries, active missions, and Warband Marks reset. Five guild ranks with increasing roster and concurrent mission limits. Four infrastructure types per layer. Deep layer breakthroughs unlock fracture zones 12-30. See [Secondary Systems](secondary-systems.md#the-deep-mercenary-expedition-system) for details.

### Ascension

Per-character combat power multiplier purchased with prestige ranks, gated by Deep layer milestones. Survives prestige.

| Level | PR Cost | Deep Gate | Multiplier |
|-------|---------|-----------|------------|
| I | 35 | Layer 3 | 2x |
| II | 65 | Layer 7 | 4x |
| III | 120 | Layer 12 | 8x |
| IV | 200 | Layer 18 | 16x |
| V | 325 | Layer 25 | 32x |
| VI | 500 | Layer 30 | 64x |
| VII+ | 500 + 75*(level-6) | None | 64 × 1.5^(level-6) |

Total PR for I-VI: 1,245. Multiplier applies to damage, defense, and HP. Ascension level stored as `ascension_level: u32` on `GameState`.

### Power Rating

A single numeric score for comparing character strength across prestige levels and ascension. Defined as the geometric mean of combat output and survivability:

```
power_rating = sqrt(effective_DPS × effective_HP)
```

Accounts for the full damage pipeline (crit, double strike, attack speed), defense reduction multiplier, and all bonus sources (prestige, Haven, god items, sigils, ascension). Cached as `cached_power_rating: f64` on `GameState`. Displayed in the stats header.

---

## Infrastructure

### CI/CD Pipeline

```bash
make check    # Run all CI checks locally (same script as CI)
```

1. `cargo fmt --check` -- Format
2. `cargo clippy --all-targets -- -D warnings` -- Lint
3. `cargo test` -- Tests
4. `cargo build --all-targets` -- Build
5. `cargo audit --deny yanked` -- Security

### Balance Simulator

```bash
cargo run --bin simulator -- [OPTIONS]
```

Headless game balance simulator that runs `game_tick()` without UI:
- `--ticks N` -- Ticks to simulate (default: 36000 = 1 hour)
- `--seed N` -- RNG seed for deterministic runs (default: 42)
- `--prestige N` -- Starting prestige rank (default: 0)
- `--runs N` -- Multiple runs with incrementing seeds
- `--csv FILE` -- Time-series CSV output
- `--verbose` -- Per-tick event logging
- `--quiet` -- Only final summary line

### Auto-Update

- Startup: Check GitHub API for latest release
- Update: `quest update` downloads and replaces binary
- Backup: Saves created before update in `~/.quest/backups/`
- Check interval: every 30 min with +/-5 min jitter

### Debug Menu

Activated with `--debug` flag, toggle with backtick.

Options organized in 8 tabs: Challenges (all 10 types), World (Dungeon, Fishing, Haven, Soulforge), Resources (Stormglass), Items (God Items), Deep (discovery, missions, marks), Zones, Character, Borders.

### Integration Tests

79 integration test files in `tests/`:
- `game_loop_orchestration_test.rs` -- 36 behavior-locking tests for game tick pipeline
- `game_tick_behavior_test.rs` / `game_tick_supplemental_test.rs` -- Tick processing behavior
- `tick_integration_test.rs` -- Cross-system tick integration
- `tick_stage_coverage_test.rs` / `tick_stages_coverage_test.rs` -- Tick stage coverage tests
- `behavior_lock_fishing_dungeon_test.rs` -- Fishing/dungeon mutual exclusion
- `prestige_cycle_test.rs` -- Prestige reset and progression
- `zone_progression_test.rs` -- Zone advancement and gating
- `fishing_integration_test.rs` / `fishing_core_coverage_test.rs` -- Fishing session lifecycle and coverage
- `dungeon_completion_test.rs` -- Dungeon room clearing and completion
- `storm_forge_test.rs` -- Stormbreaker forging chain
- `item_pipeline_test.rs` / `item_combat_coverage_test.rs` -- Item generation, equipping, and combat coverage
- `chess_integration_test.rs` -- Chess minigame
- `enhancement_test.rs` / `enhancement_coverage_test.rs` -- Soulforge enhancement system
- `god_items_test.rs` -- God items system
- `game_loop_test.rs` -- Core game loop behavior
- `achievement_coverage_test.rs` / `achievement_handlers_test.rs` / `achievements_expanded_test.rs` -- Achievement system coverage
- `character_coverage_test.rs` -- Character system coverage
- `combat_submodules_test.rs` -- Combat submodule coverage
- `haven_dungeon_coverage_test.rs` -- Haven and dungeon coverage

---

## Data Storage

### File Layout

```
~/.quest/
├── <character>.json      # Character saves (max 3)
├── haven.json            # Haven state (account-level)
├── achievements.json     # Achievements (account-level)
├── enhancement.json      # Soulforge enhancement state (account-level)
├── deep.json             # The Deep state (account-level)
└── backups/
    └── YYYY-MM-DD_HHMMSS/
        └── *.json        # Pre-update backups
```

### Save Format

Plain JSON with serde. No checksum -- relies on structural validation on load.

### Persistence Rules

| Data | Scope | Persists Through Prestige |
|------|-------|---------------------------|
| Character stats | Per-character | Reset |
| Equipment | Per-character | Reset (except Vault) |
| Zone progress | Per-character | Reset |
| Fishing state | Per-character | Preserved |
| Chess stats | Per-character | Preserved |
| Haven | Account | Preserved |
| Enhancement | Account | Preserved |
| The Deep (persistent) | Account | Preserved (guild rank, layers, infra) |
| The Deep (prestige) | Per-character | Reset (mercs, missions, Warband Marks) |
| Achievements | Account | Preserved |
| Ascension level | Per-character | Preserved |

---

## Appendix: Key Formulas

```rust
// Prestige multiplier (base, before CHA bonus)
fn prestige_multiplier(rank: u32) -> f64 {
    1.0 + 0.5 * (rank as f64).powf(0.7)
}

// Attribute modifier
fn modifier(attribute: u32) -> i32 {
    (attribute as i32 - 10) / 2
}

// XP required for level
fn xp_for_level(level: u32) -> u64 {
    (100.0 * (level as f64).powf(1.5)) as u64
}

// Attribute cap
fn attribute_cap(prestige_rank: u32) -> u32 {
    20 + prestige_rank * 5
}

// Haven discovery chance (P10+)
fn haven_discovery_chance(prestige_rank: u32) -> f64 {
    0.000014 + (prestige_rank - 10) as f64 * 0.000007
}

// Soulforge discovery chance (P15+)
fn soulforge_discovery_chance(prestige_rank: u32) -> f64 {
    0.000014 + (prestige_rank - 15) as f64 * 0.000007
}

// The Deep discovery trigger (no per-tick RNG)
// Fires once on first BossDefeatResult::ExpanseCycle at P15+.

// Ascension multiplier
fn ascension_multiplier(level: u32) -> f64 {
    if level <= 6 { 2f64.powi(level as i32) }
    else { 64.0 * 1.5f64.powi((level - 6) as i32) }
}

// Power rating
fn power_rating(effective_dps: f64, effective_hp: f64) -> f64 {
    (effective_dps * effective_hp).sqrt().round()
}

// Offline XP
fn offline_xp(elapsed_s: i64, xp_per_tick: f64, haven_bonus_pct: f64) -> f64 {
    let kills = (elapsed_s.min(604800) as f64 / 5.0) * 0.25;
    let xp_per_kill = xp_per_tick * 300.0;
    kills * xp_per_kill * (1.0 + haven_bonus_pct / 100.0)
}
```

---

## Appendix: Project Structure

```
quest/
├── src/
│   ├── main.rs              # Entry point, game loop, input routing, UI rendering
│   ├── lib.rs               # Library crate for testing (exposes all game logic)
│   ├── input/               # Keyboard input routing (split into submodules)
│   │   ├── mod.rs           # Top-level input dispatcher
│   │   ├── types.rs         # Input-related types
│   │   ├── deep_input.rs    # The Deep overlay input
│   │   ├── haven_input.rs   # Haven overlay input
│   │   ├── minigame_input.rs # Minigame input dispatch
│   │   ├── prestige_input.rs # Prestige confirmation input
│   │   ├── soulforge_input.rs # Soulforge overlay input
│   │   ├── stormglass_input.rs # Stormglass overlay input
│   │   └── time_vault_input.rs # Time Vault overlay input
│   ├── main_helpers/        # Extracted helpers from main.rs
│   │   ├── mod.rs           # Re-exports
│   │   ├── achievements.rs  # Achievement modal/save handling
│   │   ├── character_screens.rs # Character management screen logic
│   │   ├── cloud_ops.rs     # Cloud sync state reload helpers
│   │   ├── input_routing.rs # Input dispatch from main loop
│   │   ├── offline.rs       # Offline progression handling
│   │   ├── overlay.rs       # Overlay state management
│   │   ├── persistence.rs   # Save/load orchestration
│   │   ├── scene.rs         # Scene rendering dispatch
│   │   └── update.rs        # Auto-update check handling, startup splash screen
│   ├── tick_events.rs       # TickEvent → combat log + visual effects bridge
│   ├── bin/
│   │   ├── simulator.rs     # Headless balance simulator binary
│   │   └── deep_simulator.rs # Headless Deep economy simulator binary
│   ├── core/                # Core game systems
│   │   ├── constants.rs     # All game balance constants
│   │   ├── game_logic.rs    # Thin re-export wrapper for submodules
│   │   ├── game_state.rs    # Main GameState struct
│   │   ├── tick.rs          # game_tick() orchestrator
│   │   ├── tick_types.rs    # TickEvent enum (48 variants), TickResult struct
│   │   ├── tick_stages.rs   # Tick processing stages 4-6 + helpers
│   │   ├── xp.rs            # XP calculation, leveling, combat kill XP
│   │   ├── discoveries.rs   # Discovery rolls (dungeon, fishing, Haven, Soulforge)
│   │   ├── enemy_spawning.rs # Enemy generation and spawning
│   │   ├── offline.rs       # Offline XP progression
│   │   ├── recent_drops.rs  # RecentDrop struct and deque management
│   │   ├── ticker.rs        # Scrolling loot ticker (TickerEntry, Ticker, adaptive scroll speed)
│   │   └── power_rating.rs  # Power rating formula: sqrt(effective_DPS × effective_HP)
│   ├── character/           # Character system
│   │   ├── attributes.rs    # 6 RPG attributes
│   │   ├── derived_stats.rs # Stats from attributes
│   │   ├── calculation.rs   # Derived stats calculation engine
│   │   ├── prestige.rs      # Prestige system
│   │   ├── combat_bonuses.rs # Prestige combat bonuses
│   │   ├── multipliers.rs   # Prestige multiplier calculations
│   │   ├── prestige_actions.rs # Prestige eligibility and execution
│   │   ├── tiers.rs         # Prestige tier definitions
│   │   ├── manager.rs       # Character CRUD operations
│   │   ├── persistence.rs   # JSON save/load (extracted from manager.rs)
│   │   ├── name_validation.rs # Character name validation rules
│   │   ├── input.rs         # Character input routing
│   │   ├── creation.rs      # Character creation input
│   │   ├── delete.rs        # Character delete input
│   │   ├── rename.rs        # Character rename input
│   │   └── select.rs        # Character select input
│   ├── combat/              # Combat system
│   │   ├── types.rs         # Enemy, combat state
│   │   ├── logic.rs         # Combat helper functions
│   │   ├── orchestration.rs # update_combat() orchestrator
│   │   ├── attacks.rs       # Attack interval calculations
│   │   ├── enemy_generation.rs # Zone/dungeon enemy generators
│   │   ├── player_attack.rs # Player damage pipeline
│   │   ├── enemy_attack.rs  # Enemy attack resolution
│   │   ├── damage.rs        # Shared damage calculation, enemy death handling
│   │   ├── events.rs        # CombatEvent, CombatBonuses (unified struct)
│   │   └── regen.rs         # HP regeneration after combat
│   ├── zones/               # Zone system
│   │   ├── data.rs          # Zone definitions (30 zones)
│   │   ├── progression.rs   # Zone progression
│   │   ├── advancement.rs   # Zone/subzone advancement and travel
│   │   ├── boss_defeat.rs   # Boss defeat handling
│   │   ├── gates.rs         # Weapon gate queries, access checks
│   │   ├── fracture.rs      # FractureRegion enum (RedFault through OriginWound)
│   │   └── access.rs        # sync_account_zone_unlocks()
│   ├── dungeon/             # Dungeon system
│   │   ├── types.rs         # Room types, dungeon sizes (5)
│   │   ├── generation.rs    # Procedural generation
│   │   ├── logic.rs         # Room clearing, key system
│   │   ├── pathfinding.rs   # BFS-based dungeon navigation
│   │   └── rewards.rs       # Dungeon XP, item generation, treasure rooms
│   ├── fishing/             # Fishing system
│   │   ├── types.rs         # Fish, phases, ranks
│   │   ├── generation.rs    # Fish generation, Leviathan
│   │   ├── logic.rs         # Session tick processing
│   │   ├── discovery.rs     # Fishing spot discovery logic
│   │   ├── drops.rs         # Item drops from fishing
│   │   └── rank.rs          # Rank progression and tier definitions
│   ├── items/               # Item system
│   │   ├── types.rs         # Items, slots, affixes
│   │   ├── equipment.rs     # Equipment container
│   │   ├── generation.rs    # Item generation
│   │   ├── drops.rs         # Drop system
│   │   ├── names.rs         # Name generation
│   │   └── scoring.rs       # Power scoring and auto-equip scoring
│   ├── challenges/          # Challenge minigames
│   │   ├── menu.rs          # Challenge menu
│   │   ├── chess/           # Chess minigame
│   │   ├── go/              # Go (Territory Control)
│   │   ├── morris/          # Nine Men's Morris (includes ai.rs)
│   │   ├── gomoku/          # Gomoku (Five in a Row, includes ai.rs)
│   │   ├── minesweeper/     # Trap Detection
│   │   ├── rune/            # Rune Deciphering
│   │   ├── snake/           # Serpent's Path (Snake)
│   │   ├── flappy/          # Skyward Gauntlet (Flappy Bird)
│   │   ├── jezzball/        # Containment Breach (JezzBall)
│   │   └── runic_shift/     # Sigil Surge (panel-matching)
│   ├── haven/               # Haven base building
│   │   ├── types.rs         # Haven struct, upgrade tiers, bonus types
│   │   ├── logic.rs         # Construction, upgrade orchestration
│   │   ├── room_defs.rs     # 14 room definitions and skill tree
│   │   └── bonus.rs         # Bonus calculation from room tiers
│   ├── enhancement/         # Soulforge enhancement system
│   │   ├── types.rs         # Enhancement progress, constants, success rates
│   │   ├── logic.rs         # Enhancement rolls, discovery
│   │   └── persistence.rs   # Save/load from ~/.quest/enhancement.json
│   ├── ascension/           # Ascension combat power multiplier system
│   │   ├── mod.rs           # Public re-exports
│   │   ├── types.rs         # Cost table, gate table, multiplier formula
│   │   └── logic.rs         # can_ascend(), ascend(), AscendResult
│   ├── deep/                # The Deep — Mercenary Expedition System
│   │   ├── mod.rs           # Public API re-exports
│   │   ├── types.rs         # All data structures (DeepState, Mercenary, Mission, etc.)
│   │   ├── mercenaries.rs   # Merc generation, recruitment, leveling, injuries
│   │   ├── layers.rs        # Layer difficulty, familiarity, infrastructure, durations
│   │   ├── missions.rs      # Mission creation, assignment, resolution
│   │   ├── events.rs        # Check-in events and event choices
│   │   ├── economy.rs       # Warband Marks economy, costs, rewards
│   │   ├── discovery.rs     # Discovery roll logic, starter roster
│   │   ├── narratives.rs    # Mission narrative text for results modal
│   │   └── persistence.rs   # Save/load from ~/.quest/deep.json
│   ├── stormglass/          # Stormglass currency and Storm Sigils
│   │   ├── types.rs         # Stormglass state, daily rotation
│   │   ├── sigils.rs        # Storm Sigil definitions and bonuses
│   │   ├── earning.rs       # Stormglass earning from challenges
│   │   └── spending.rs      # Stormglass spending on sigils
│   ├── power_cores/         # Power Cores — passive PR generation from Deep milestones
│   │   ├── types.rs         # Core definitions, state, helpers
│   │   ├── tick.rs          # Per-tick processing, offline catchup
│   │   ├── persistence.rs   # Save/load power_cores.json
│   │   └── mod.rs           # Public re-exports
│   ├── god_items/           # God Items system (Asprika, Sleipnir, Megingjord)
│   │   └── types.rs         # God item definitions, passives, bonuses, query helpers
│   ├── achievements/        # Achievement system
│   │   ├── types.rs         # AchievementId (213 variants), Achievements state
│   │   ├── data.rs          # Achievement database
│   │   ├── handlers.rs      # Event handlers (on_kill, on_boss_kill, etc.)
│   │   ├── milestones.rs    # Milestone definitions and thresholds
│   │   ├── modal.rs         # Modal notification queue (500ms accumulation window)
│   │   ├── notifications.rs # Notification state management
│   │   ├── stats.rs         # Achievement statistics and progress tracking
│   │   ├── titles.rs        # Title definitions (63 titles), selection, validation
│   │   ├── unlock.rs        # Core unlock machinery (is_unlocked, unlock, check_milestones)
│   │   └── persistence.rs   # Save/load
│   ├── utils/               # Utilities
│   │   ├── build_info.rs    # Build metadata
│   │   ├── updater.rs       # Self-update
│   │   └── debug_menu.rs    # Debug menu
│   └── ui/                  # UI components (terminal-coupled, not in lib.rs)
│       ├── mod.rs           # Layout coordinator
│       ├── game_common.rs   # Shared minigame layout
│       ├── stats_panel.rs   # Character stats
│       ├── stats_attributes.rs # Stats panel attribute rendering
│       ├── stats_equipment.rs # Stats panel equipment rendering
│       ├── stats_prestige.rs # Stats panel prestige info rendering
│       ├── stats_sigils.rs  # Stats panel Storm Sigils rendering
│       ├── ticker.rs        # Scrolling loot ticker with independent per-entry scrolling
│       ├── combat_scene.rs  # Combat view
│       ├── combat_3d.rs     # 3D dungeon renderer
│       ├── combat_effects.rs # Visual effects
│       ├── enemy_sprites.rs # ASCII enemy sprites
│       ├── enemy_sprite_data.rs # Sprite constant data
│       ├── dungeon_map.rs   # Dungeon minimap
│       ├── fishing_scene.rs # Fishing UI
│       ├── haven_scene.rs   # Haven overlay
│       ├── haven_details.rs # Haven room detail panel
│       ├── haven_tree.rs    # Haven skill tree panel
│       ├── prestige_confirm.rs # Prestige dialog
│       ├── achievement_browser_scene.rs # Achievement browser
│       ├── achievement_details.rs # Achievement detail panel
│       ├── achievement_list.rs # Achievement list rendering
│       ├── achievement_tabs.rs # Achievement tab navigation
│       ├── title_browser_scene.rs # Title browser overlay
│       ├── challenge_menu_scene.rs # Challenge menu
│       ├── chess_scene.rs, go_scene.rs, morris_scene.rs,
│       │   gomoku_scene.rs, minesweeper_scene.rs, rune_scene.rs,
│       │   snake_scene.rs, flappy_scene.rs, jezzball_scene.rs,
│       │   runic_shift_scene.rs
│       ├── deep_scene.rs     # The Deep overlay (top-level coordinator)
│       ├── deep_missions.rs  # Deep active missions panel
│       ├── deep_roster.rs    # Deep roster sub-view
│       ├── deep_layers.rs    # Deep layer map sub-view
│       ├── deep_events.rs    # Deep event response sub-view
│       ├── deep_results.rs   # Deep mission complete modal
│       ├── soulforge_scene.rs # Soulforge enhancement overlay
│       ├── soulforge_effects.rs # Soulforge animation effects
│       ├── soulforge_slots.rs # Soulforge slot selection rendering
│       ├── bug_report_scene.rs # Bug report overlay
│       ├── stormglass_scene.rs # Stormglass Exchange overlay with animations
│       ├── scene_fx.rs       # Shared utilities for layered ASCII scene rendering
│       ├── zone_bg.rs        # Stylized zone background scenes (6-layer compositing, all 30 zones)
│       ├── debug_menu_scene.rs # Debug menu with tabbed categories
│       ├── throbber.rs      # Spinner animations
│       └── character_select.rs, character_creation.rs,
│           character_delete.rs, character_rename.rs
├── tests/                   # 79 integration test files
├── .github/workflows/       # CI/CD pipeline
├── scripts/                 # Quality checks (ci-checks.sh)
├── docs/                    # Design documents
│   ├── system-design.md     # This file
│   ├── core-systems.md      # Core systems design
│   ├── secondary-systems.md # Secondary systems design
│   ├── balancing.md         # Game balance guide
│   ├── challenge-minigames.md # Challenge minigames design
│   ├── decisions.md         # Design decision log
│   ├── infrastructure.md   # Infrastructure design
│   └── archive/             # Original dated design documents
├── Makefile                 # Dev helpers
└── CLAUDE.md                # Project-level AI guide
```

### Module CLAUDE.md Files

Each major module has its own `CLAUDE.md` with implementation patterns, integration points, and extension guides:

- `src/core/CLAUDE.md` -- GameState, TickEvent/TickResult, game_tick() stages, constants
- `src/character/CLAUDE.md` -- Attributes, prestige, character persistence
- `src/combat/CLAUDE.md` -- Combat state machine, enemy generation
- `src/zones/CLAUDE.md` -- Zone tiers, progression, weapon gates
- `src/dungeon/CLAUDE.md` -- Procedural generation, room system
- `src/fishing/CLAUDE.md` -- Ranking, rarity, Storm Leviathan hunt
- `src/items/CLAUDE.md` -- Item generation pipeline, scoring, drop rates
- `src/challenges/CLAUDE.md` -- Adding new minigames (step-by-step checklist)
- `src/haven/CLAUDE.md` -- Account-level base building, bonus system
- `src/achievements/CLAUDE.md` -- Achievement tracking, modal notifications
- `src/enhancement/CLAUDE.md` -- Soulforge enhancement system
- `src/ascension/CLAUDE.md` -- Ascension combat power multiplier system
- `src/deep/CLAUDE.md` -- The Deep mercenary expedition system
- `src/ui/CLAUDE.md` -- Shared layout components, color conventions

---

*For detailed per-system design docs, see the companion files in `docs/`. For historical design decisions and rationale, see `docs/decisions.md`.*
