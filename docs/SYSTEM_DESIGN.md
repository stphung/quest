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

Quest is a terminal-based idle RPG built in Rust using Ratatui for UI rendering and Crossterm for terminal backend.

```
┌─────────────────────────────────────────────────────────────────┐
│                        QUEST ARCHITECTURE                        │
└─────────────────────────────────────────────────────────────────┘

┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   main.rs    │────▶│  Game Loop   │────▶│   Renderer   │
│  (Entry)     │     │  (10 ticks/s)│     │  (Ratatui)   │
└──────────────┘     └──────┬───────┘     └──────────────┘
                           │
         ┌─────────────────┼─────────────────┐
         ▼                 ▼                 ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│    Combat    │  │  Progression │  │   Systems    │
│    Engine    │  │   (XP/Level) │  │ (Fish/Dung)  │
└──────────────┘  └──────────────┘  └──────────────┘
         │                 │                 │
         └─────────────────┼─────────────────┘
                           ▼
                  ┌──────────────┐
                  │  GameState   │
                  │   (JSON)     │
                  └──────────────┘
```

### Key Dependencies

| Crate | Purpose |
|-------|---------|
| ratatui 0.26 | Terminal UI framework |
| crossterm 0.27 | Terminal backend |
| serde / serde_json | JSON serialization |
| rand | RNG for procedural systems |
| chrono | Offline progression timing |
| chess-engine 0.1 | Chess minigame AI |
| ureq | HTTP client for auto-update |

---

## Core Game Loop

The game runs at **10 ticks per second** (100ms intervals).

```
┌─────────────────────────────────────────────────────────────────┐
│                         GAME TICK FLOW                           │
└─────────────────────────────────────────────────────────────────┘

                    ┌─────────────────┐
                    │   Process Input │
                    └────────┬────────┘
                             │
              ┌──────────────┴──────────────┐
              ▼                             ▼
     ┌─────────────────┐          ┌─────────────────┐
     │  Menu/UI Input  │          │  Game State     │
     │  (Navigation)   │          │  Update         │
     └─────────────────┘          └────────┬────────┘
                                           │
         ┌─────────────┬───────────────────┼───────────────────┐
         ▼             ▼                   ▼                   ▼
    ┌─────────┐  ┌──────────┐      ┌──────────────┐    ┌──────────┐
    │ Combat  │  │  Passive │      │   Discovery  │    │  Regen   │
    │ Tick    │  │  XP Tick │      │   Rolls      │    │  Tick    │
    └─────────┘  └──────────┘      └──────────────┘    └──────────┘
         │             │                   │                   │
         └─────────────┴───────────────────┴───────────────────┘
                                   │
                                   ▼
                          ┌───────────────┐
                          │    Render     │
                          └───────────────┘
```

### Key Constants

| Constant | Value | Purpose |
|----------|-------|---------|
| Tick interval | 100ms | Core loop speed |
| Attack interval | 1.5s | Base combat speed |
| HP regen delay | 2.5s | Post-kill healing |
| Autosave | 30s | Periodic save |
| Update check | 30 min | Version polling |

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
cap = 20 + (prestige_rank × 5)
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
| Max HP | 50 + (CON_mod × 10) |
| Physical Damage | 5 + (STR_mod × 2) |
| Magic Damage | 5 + (INT_mod × 2) |
| Defense | DEX_mod (min 0) |
| Crit Chance | 5% + (DEX_mod × 1%) |
| XP Multiplier | 1.0 + (WIS_mod × 0.05) |

---

## Combat System

### Auto-Battle Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                        COMBAT STATE MACHINE                      │
└─────────────────────────────────────────────────────────────────┘

    ┌──────────────┐
    │   IDLE       │◀────────────────────────────────┐
    │ (No Enemy)   │                                 │
    └──────┬───────┘                                 │
           │ spawn                                   │
           ▼                                         │
    ┌──────────────┐                                 │
    │   FIGHTING   │◀───────┐                        │
    │              │        │                        │
    └──────┬───────┘        │                        │
           │                │                        │
     ┌─────┴─────┐          │                        │
     ▼           ▼          │                        │
┌─────────┐ ┌─────────┐     │                        │
│ Player  │ │ Enemy   │     │                        │
│ Attacks │ │ Attacks │     │                        │
└────┬────┘ └────┬────┘     │                        │
     │           │          │                        │
     ▼           ▼          │                        │
  [Damage]    [Damage]      │                        │
     │           │          │                        │
     │      ┌────┴────┐     │                        │
     │      ▼         ▼     │                        │
     │  [Player   [Enemy    │                        │
     │   Dies]    Dies]     │                        │
     │      │         │     │                        │
     │      ▼         ▼     │                        │
     │  [Respawn] [+XP/Gold]│                        │
     │      │         │     │                        │
     │      │         ▼     │                        │
     │      │  ┌──────────────┐                      │
     │      │  │ REGENERATING │                      │
     │      │  │  (2.5 sec)   │                      │
     │      │  └──────┬───────┘                      │
     │      │         │ healed                       │
     └──────┴─────────┴──────────────────────────────┘
```

### Combat Mechanics

- **Attack Interval**: 1.5 seconds (base), reduced by AttackSpeed affixes
- **Critical Hits**: 2× base damage (increased by CritMultiplier affixes)
- **Defense**: Flat damage reduction
- **Double Strike**: Haven War Room bonus (10-35% chance for extra attack)

### Enemy Generation

- HP: 80-120% of player max HP
- Damage: Calibrated for 5-10 second fights
- Names: Procedurally generated from syllable combinations

---

## Progression Systems

### XP Curve

```
xp_needed = 100 × level^1.5
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
xp_per_tick = 1.0 × prestige_mult × wis_mult
```

**Combat Kill XP:**
```
kill_xp = xp_per_tick × random(200..400)
```
Each kill awards 20-40 seconds worth of passive XP.

### Prestige System

#### Multiplier Formula (Diminishing Returns)

```
multiplier = 1.0 + 0.5 × rank^0.7
```

| Rank | Multiplier | Gain |
|------|------------|------|
| P1 | 1.50× | +50% |
| P5 | 2.54× | +10%/rank |
| P10 | 3.51× | +6%/rank |
| P20 | 5.07× | +3%/rank |

#### Prestige Tiers

| Tier | Ranks | Names | Required Levels |
|------|-------|-------|-----------------|
| Metals | 1-4 | Bronze, Silver, Gold, Platinum | 10, 25, 50, 65 |
| Gems | 5-9 | Diamond, Emerald, Sapphire, Ruby, Obsidian | 80, 90, 100, 110, 120 |
| Cosmic | 10-14 | Celestial, Astral, Cosmic, Stellar, Galactic | 130, 140, 150, 160, 170 |
| Divine | 15-19 | Transcendent, Divine, Exalted, Mythic, Legendary | 180, 190, 200, 210, 220 |
| Eternal | 20+ | Eternal | 220 + (rank-19)×15 |

#### Prestige Reset

**Wiped:** Level, XP, attributes, equipment, zone progress, active activities

**Preserved:** Prestige rank, fishing state, Haven, achievements

---

## Zone & World Structure

### Zone Layout

```
┌─────────────────────────────────────────────────────────────────┐
│                        WORLD STRUCTURE                           │
└─────────────────────────────────────────────────────────────────┘

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

Tier 4 (P15)         Tier 5 (P20)         Post-Game
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│  Crystal    │      │  Floating   │      │    The      │
│  Caverns    │      │   Isles     │      │  Expanse    │
│  (4 subs)   │      │  (4 subs)   │      │ (∞ cycles)  │
└─────────────┘      └─────────────┘      └─────────────┘
┌─────────────┐      ┌─────────────┐
│   Sunken    │      │   Storm     │
│  Kingdom    │      │  Citadel    │
│  (4 subs)   │      │  (4 subs)   │
└─────────────┘      └─────────────┘
                           ▲
                    [Stormbreaker
                      Required]
```

### Subzone Progression

- 10 kills per subzone triggers boss
- Boss defeat advances to next subzone/zone
- Zone 11 (The Expanse) cycles infinitely

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

### Drop Rates

- Base: 15% per kill
- Prestige bonus: +1% per rank (max +10%)
- Haven Trophy Hall: up to +15%
- Maximum: 25%

### Affix Types

| Category | Affixes |
|----------|---------|
| Damage | DamagePercent, CritChance, CritMultiplier, AttackSpeed |
| Survival | HPBonus, DamageReduction, HPRegen, DamageReflection |
| Utility | XPGain |

---

## Secondary Systems

### Fishing

```
┌─────────────────────────────────────────────────────────────────┐
│                      FISHING STATE MACHINE                       │
└─────────────────────────────────────────────────────────────────┘

   [Combat Tick]
        │
        ▼
   5% Discovery ──No──▶ [Continue Combat]
        │
       Yes
        ▼
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│   CASTING    │────▶│   WAITING    │────▶│   REELING    │
│  (0.5-1.5s)  │     │  (1.0-8.0s)  │     │  (0.5-3.0s)  │
└──────────────┘     └──────────────┘     └──────┬───────┘
                                                 │
                                                 ▼
                                          [Catch Fish]
                                                 │
                                    ┌────────────┴────────────┐
                                    ▼                         ▼
                             [Session End?]            [Next Fish]
                                    │                         │
                                   Yes                        │
                                    │                         │
                                    ▼                         │
                             [Return to                       │
                               Combat]◀───────────────────────┘
```

**Ranks:** 40 total (base cap 30, +10 with FishingDock T4)

**Fish Rarities:** Common (60%) → Legendary (1%)

### Dungeons

- 2% discovery chance per kill
- Procedurally generated rooms (5×5 to 13×13)
- Room types: Combat, Treasure, Elite, Boss
- Safe death (no prestige loss)

---

## Challenge Minigames

### Discovery

- 0.000014 per tick (~2 hour average)
- Requires P1+
- Haven Library bonus: up to +50%

### Games & AI

| Game | Algorithm | Difficulties |
|------|-----------|--------------|
| Chess | Minimax (1-3 ply) | 500-1350 ELO |
| Go | MCTS (500-20k sims) | 20-12 kyu |
| Gomoku | Minimax + α-β (2-5 ply) | - |
| Morris | Minimax + α-β (2-5 ply) | - |
| Minesweeper | N/A (puzzle) | 9×9 to 20×16 |
| Rune | N/A (deduction) | 60-32,768 combos |

### Rewards

| Game | Master Reward |
|------|---------------|
| Chess | +5 Prestige Ranks |
| Go | +5 Prestige Ranks |
| Gomoku | +2 PR, +100% XP |
| Morris | +1 Fishing Rank, +200% XP |
| Minesweeper | +1 PR, +200% XP |
| Rune | +1 PR, +2 Fishing Ranks |

---

## Haven (Base Building)

### Overview

Account-level skill tree unlocked at P10+. Spend prestige ranks to build rooms.

### Skill Tree

```
                         [Hearthstone]
                        /              \
                Combat Branch        QoL Branch
                     │                    │
                  Armory              Bedroom
                  /    \              /    \
          Training    Trophy     Garden   Library
            Yard       Hall        │        │
              │          │     Fish Dock  Workshop
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

| Room | T1 | T2 | T3 |
|------|-----|-----|-----|
| Hearthstone | +25% Offline XP | +50% | +100% |
| Armory | +5% Damage | +10% | +25% |
| Training Yard | +5% XP | +10% | +30% |
| Trophy Hall | +5% Drops | +10% | +15% |
| Watchtower | +5% Crit | +10% | +20% |
| Alchemy Lab | +25% HP Regen | +50% | +100% |
| War Room | +10% Double Strike | +20% | +35% |
| Bedroom | -15% Regen Delay | -30% | -50% |
| Garden | -10% Fishing Time | -20% | -40% |
| Library | +20% Discovery | +30% | +50% |
| Fishing Dock | +25% Double Fish | +50% | +100% (T4: +10 Max Rank) |
| Workshop | +10% Item Rarity | +15% | +25% |
| Vault | 1 item preserved | 3 | 5 |
| StormForge | Stormbreaker access | - | - |

---

## Endgame Content

### Stormbreaker Quest Chain

```
┌─────────────────────────────────────────────────────────────────┐
│                    STORMBREAKER PROGRESSION                      │
└─────────────────────────────────────────────────────────────────┘

┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Reach      │     │  Catch      │     │  Build      │
│  Fishing    │────▶│  Storm      │────▶│  Full       │
│  Rank 40    │     │  Leviathan  │     │  Haven      │
└─────────────┘     └─────────────┘     └──────┬──────┘
      │                    │                   │
      │                    │                   ▼
      │                    │           ┌─────────────┐
      ▼                    ▼           │  Build      │
 [Requires           [10 encounters    │  StormForge │
  FishingDock         then 25%         │  (25 PR)    │
  T4]                 catch rate]      └──────┬──────┘
                                              │
                                              ▼
                                       ┌─────────────┐
                                       │   Forge     │
                                       │ Stormbreaker│
                                       │  (25 PR)    │
                                       └──────┬──────┘
                                              │
                                              ▼
                                       ┌─────────────┐
                                       │  Defeat     │
                                       │ The Undying │
                                       │   Storm     │
                                       └──────┬──────┘
                                              │
                                              ▼
                                       ┌─────────────┐
                                       │  Unlock     │
                                       │    The      │
                                       │  Expanse    │
                                       └─────────────┘
```

### The Expanse (Zone 11)

- Infinite cycling post-game zone
- 4 subzones: Void's Edge, Eternal Storm, Abyssal Rift, The Endless
- Final boss: Avatar of Infinity (cycles back to subzone 1)
- Unlocked via StormsEnd achievement

---

## Infrastructure

### CI/CD Pipeline

```bash
make check    # Run all CI checks locally
```

1. `cargo fmt --check` — Format
2. `cargo clippy -- -D warnings` — Lint
3. `cargo test` — Tests
4. `cargo build --all-targets` — Build
5. `cargo audit` — Security

### Auto-Update

- Startup: Check GitHub API for latest release
- Update: `quest update` downloads and replaces binary
- Backup: Saves created before update in `~/.quest/backups/`

### Debug Menu

Activated with `--debug` flag, toggle with backtick (\`).

Options: Trigger Dungeon, Fishing, all 6 Challenges, Haven Discovery

---

## Data Storage

### File Layout

```
~/.quest/
├── <character>.json      # Character saves (max 3)
├── haven.json            # Haven state (account-level)
├── achievements.json     # Achievements (account-level)
└── backups/
    └── YYYY-MM-DD_HHMMSS/
        └── *.json        # Pre-update backups
```

### Save Format

Plain JSON with serde. No checksum — relies on structural validation on load.

### Persistence Rules

| Data | Scope | Persists Through Prestige |
|------|-------|---------------------------|
| Character stats | Per-character | ❌ Reset |
| Equipment | Per-character | ❌ Reset (except Vault) |
| Zone progress | Per-character | ❌ Reset |
| Fishing state | Per-character | ✅ Preserved |
| Haven | Account | ✅ Preserved |
| Achievements | Account | ✅ Preserved |

---

## Appendix: Key Formulas

```rust
// Prestige multiplier
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
```

---

*This document consolidates the design specifications from `docs/design/`. For historical design decisions and rationale, see `docs/DECISIONS.md`.*
