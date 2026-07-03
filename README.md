# quest

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS-blue.svg)](https://github.com/stphung/quest/releases/latest)

A terminal-based idle RPG game written in Rust. Watch your hero grow stronger automatically as they battle through 50 zones, explore procedural dungeons, fish for legendary catches, and dive into layered endgame systems — prestige, Ascension, the Deep, and the Loom of Worlds!

> **Why "quest"?** Because that's exactly what it is. Simple, memorable, and to the point.

## Quick Start

**Install and play with one command:**

**macOS / Linux:**
```bash
curl -sSf https://raw.githubusercontent.com/stphung/quest/main/install.sh | sh
```

Then run `quest` to start your adventure!

## Features

- **Automatic Combat** - Your character fights enemies automatically with turn-based combat
- **50 Zones** - 10 base zones from Meadow to Storm Citadel, Fracture zones (12-30) unlocked through the Deep, and Loom zones (31-50) gated by Woven Patterns, Ascension, and prestige
- **6 Attributes** - STR, DEX, CON, INT, WIS, CHA form the foundation of your character
- **Prestige System** - Reset for a permanent XP multiplier that grows with rank (diminishing returns) and unlock higher zones
- **Procedural Dungeons** - Explore grid-based dungeons with fog of war, treasure rooms, elite guardians, and bosses
- **Fishing** - Separate progression track with 40 ranks and 5 fish rarities
- **Diablo-style Items** - 7 equipment slots, 6 rarity tiers (including God items with unique passives), procedural names, and smart auto-equip
- **Multi-Character** - Create and manage multiple characters with JSON saves
- **Offline Progress** - Continue gaining XP even when closed (25% rate, max 7 days)
- **Challenge Minigames** - Discover and play 14 minigames: Chess, Go, Nine Men's Morris, Gomoku, Minesweeper, Rune Deciphering, Runic Lights, Runic Shift, Shard Fusion, Sudoku, Snake, Jezzball, Flappy Bird, and Vault Warden
- **Haven Base Building** - Account-level base with upgradeable rooms providing permanent bonuses
- **Soulforge Enhancement** - Account-wide per-slot equipment enhancement (+1 to +10) with escalating risk
- **Ascension** - Multiplicative combat power system (2× up to 324×) gated by Deep milestones and Woven Patterns
- **The Deep** - Recruit a mercenary company and send real-time expeditions into an endless underground structure
- **Loom of Worlds** - Resource production chains weaving Patterns that unlock the final 20 zones
- **Stormglass Exchange** - Currency earned through gameplay (item salvage, dungeon caches, enhancement consolation), spent on Storm Sigils and combat boosts
- **Power Cores** - Passive prestige rank generation unlocked by Deep layer breakthroughs
- **Time Vault** - Git-based save versioning: browse, restore, and fork your save history
- **Achievements** - Track milestones across combat, zones, fishing, challenges, and prestige
- **3D ASCII Combat** - First-person dungeon view with visual effects
- **Animated UI** - Throbber animations and progress bars for XP and fishing rank

## Installation

### Quick Install (Recommended)

**macOS / Linux:**
```bash
curl -sSf https://raw.githubusercontent.com/stphung/quest/main/install.sh | sh
```

The installer will:
- Download the latest release for your platform
- Install to `~/.local/bin/quest`
- Provide instructions to add to PATH if needed

### Updating

To update to the latest version, run:
```bash
quest update
```

The binary will self-update with the latest build.

### Manual Download

Download the latest release for your platform from the [releases page](https://github.com/stphung/quest/releases/latest).

**Supported platforms:**
- Linux (x86_64)
- macOS (Intel x86_64 and Apple Silicon ARM64)

### Building from Source

**Prerequisites:**
- A recent stable Rust toolchain (CI builds on latest stable)
- Cargo (comes with Rust)

```bash
git clone https://github.com/stphung/quest.git
cd quest
cargo run --release
```

## Controls

### Character Select
- **Arrow Keys**: Navigate character list
- **Enter**: Select character
- **N**: Create new character
- **D**: Delete character
- **R**: Rename character
- **A**: Achievements browser
- **T**: Time Vault
- **W**: Open the wiki
- **!**: Bug report
- **Esc**: Quit

### Gameplay
- **P**: Prestige (reset for XP multiplier, requires meeting level threshold)
- **Tab**: Challenge menu (when challenges are pending)
- **H**: Haven / **S**: Soulforge / **G**: Stormglass Exchange / **D**: The Deep / **L**: Loom of Worlds / **U**: Ascension (each once discovered)
- **A**: Achievements browser / **T**: Time Vault / **W**: Wiki / **!**: Bug report
- **Esc**: Quit to character select

## Game Systems

### Zones & Progression

Progress through 50 zones. The first 10 base zones each have 3-4 subzones and unique bosses:

| Tier | Zones | Prestige Required | Levels |
|------|-------|-------------------|--------|
| Nature's Edge | Meadow, Dark Forest | P0 | 1-25 |
| Civilization's Remnants | Mountain Pass, Ancient Ruins | P5 | 25-55 |
| Elemental Forces | Volcanic Wastes, Frozen Tundra | P10 | 55-85 |
| Hidden Depths | Crystal Caverns, Sunken Kingdom | P15 | 85-115 |
| Ascending | Floating Isles, Storm Citadel | P20 | 115-150 |

- Defeat 10 enemies in a subzone to spawn the boss
- Defeat subzone bosses to advance
- Zone 10's final boss requires forging **Stormbreaker**
- Beyond Zone 10: The Expanse (Zone 11, an infinite endgame zone) opens after completing Zone 10 at P25; Fracture zones (12-30) unlock by clearing layers of the Deep; Loom zones (31-50) unlock through Woven Patterns, Ascension tiers, and prestige milestones

### Attributes & Combat

**Six Core Attributes** (modifier = `(value - 10) / 2`):
- **Strength (STR)**: Physical damage (+2 per modifier)
- **Dexterity (DEX)**: Defense and crit chance (+1% crit per modifier)
- **Constitution (CON)**: Maximum HP (+10 per modifier)
- **Intelligence (INT)**: Magic damage (+2 per modifier)
- **Wisdom (WIS)**: XP gain (+5% per modifier)
- **Charisma (CHA)**: Prestige multiplier bonus (+10% per modifier)

**Combat Mechanics:**
- Turn-based rounds every 1.5 seconds
- Critical hits deal 2× damage
- HP regenerates over 2.5s after killing an enemy
- Dying to a boss resets the encounter (prestige is preserved)

### Prestige System

Prestige resets your level for permanent benefits:
- **XP Multiplier**: `1 + 0.5 × rank^0.7` — diminishing returns (P1 = 1.5×, P2 ≈ 1.8×, P3 ≈ 2.1×, ...)
- **Attribute Caps**: Base 20 + (5 × prestige rank)
- **Zone Unlocks**: Higher zones require prestige ranks
- **Better Item Drops**: +1% drop rate per prestige rank (capped at 25% total)

Rank tiers: Bronze (P1) → Silver (P2) → Gold (P3) → Platinum (P4) → Diamond (P5) → Emerald → Sapphire → Ruby → Obsidian → Celestial (P10)...

### Dungeons

Procedural grid-based exploration:
- **Discovery**: 1% chance per enemy kill to discover and automatically enter a dungeon
- **Sizes**: Small (5×5), Medium (7×7), Large (9×9), Epic (11×11), Legendary (13×13) based on level and prestige
- **Room Types**: Combat, Treasure (guaranteed item), Elite (key guardian), Boss
- **Key System**: Defeat Elite guardian to get key for Boss room
- **Fog of War**: Rooms revealed as you explore
- **Safe Death**: No prestige loss when dying in dungeons

### Fishing

Separate progression track with 40 ranks across 8 tiers:
- Novice → Apprentice → Journeyman → Expert → Master → Grandmaster → Mythic → Transcendent
- Fish rarities: Common, Uncommon, Rare, Epic, Legendary
- Higher ranks improve catch quality
- Base rank cap is 30; ranks 31-40 require the Haven Fishing Dock at Tier 4
- At rank 40, legendary catches can trigger the **Storm Leviathan** hunt — catching it unlocks forging Stormbreaker at the Haven Storm Forge

### Challenge Minigames

Discover challenge minigames while adventuring (requires Prestige 1+):

- **Chess** - Play against AI with 4 difficulty levels (Novice ~500 ELO to Master ~1350 ELO)
- **Go** - 9×9 territory control on a classic board, MCTS AI (4 difficulty levels)
- **Nine Men's Morris** - Classic strategy board game against AI opponents
- **Gomoku** - Five-in-a-row on a 15×15 board with minimax AI (4 difficulty levels)
- **Minesweeper (Trap Detection)** - Clear minefields across 4 difficulty levels (9×9 to 20×16)
- **Rune Deciphering** - Mastermind-style deduction game with symbol sequences
- **...and more** - Runic Lights, Runic Shift, Shard Fusion, Sudoku, Snake, Jezzball, Flappy Bird, and Vault Warden (14 total)
- Challenges appear randomly (~2 hour average discovery time)
- Accept or decline from the challenge menu
- Winning rewards Stormglass, plus prestige ranks (and sometimes fishing ranks) at higher difficulties

### Haven (Base Building)

An account-level base that persists across all prestige resets:
- Build and upgrade rooms that provide permanent bonuses
- Bonuses include: XP multiplier, item drop rate, item rarity, fishing gain, challenge discovery rate
- Rooms cost prestige ranks to build and upgrade
- Benefits apply to all characters on the account

### Achievements

Track your progress across all characters:
- Categories: Combat, Level, Prestige, Progression, Challenges, Exploration, Deep, Loom, and Stats
- Account-level persistence (never lost on prestige or character deletion)
- Stored in `~/.quest/achievements.json`

### Items & Equipment

**7 Equipment Slots**: Weapon, Armor, Helmet, Gloves, Boots, Amulet, Ring

**6 Rarity Tiers**:
| Rarity | Affixes |
|--------|---------|
| Common | 0 |
| Magic | 1 |
| Rare | 2-3 |
| Epic | 3-4 |
| Legendary | 4-5 |
| God | 4-5 + unique passive |

- Items roll 1-3 attributes; per-attribute values scale with rarity, item level (zone × 10), and quality tier (T0-T9)
- Procedural name generation with prefixes/suffixes
- Smart auto-equip based on weighted scoring (God items are never auto-replaced)
- Drop rate: 15% base + 1% per prestige rank (capped at 25%)

## Save System

- **Location**: `~/.quest/` directory (JSON format)
- **Multi-character**: Each character saved separately, plus account-level files (haven, achievements, the Deep)
- **Auto-save**: Every 30 seconds
- **Offline Progress**: Simulates kills at 25% rate (max 7 days)
- **Time Vault**: `~/.quest/` is a git repository — every meaningful event commits your full save state, and you can browse, restore, and fork history in-game (with optional GitHub cloud sync)

## Technical Details

- Built with [Ratatui](https://github.com/ratatui-org/ratatui) for terminal UI
- Uses [Crossterm](https://github.com/crossterm-rs/crossterm) for cross-platform terminal handling
- Save files use JSON format
- 100ms game tick (10 ticks/sec)

## Development

### Project Structure

```
src/
├── main.rs            # Entry point, game loop
├── input/             # Keyboard input routing
├── core/              # Game state, tick engine, constants
├── character/         # Attributes, prestige, save system
├── combat/            # Enemy generation, combat logic
├── zones/             # Zone data and progression (base, Fracture, Loom)
├── dungeon/           # Procedural dungeon system
├── fishing/           # Fishing minigame
├── items/             # Equipment and drop system
├── enhancement/       # Soulforge equipment enhancement
├── ascension/         # Combat power multiplier system
├── deep/              # Mercenary expedition system
├── loom/              # Resource production chains
├── stormglass/        # Currency and Storm Sigils
├── power_cores/       # Passive PR generation
├── god_items/         # Norse mythology endgame items
├── challenges/        # 14 challenge minigames
├── haven/             # Account-level base building
├── achievements/      # Achievement tracking system
├── history/           # Git-based save versioning (Time Vault)
├── vessel/            # Act 2 Vessel/Voyage (dark behind kill-switch)
├── main_helpers/      # Orchestration between main.rs and domain modules
├── utils/             # Build info, updater, debug menu
├── bin/               # Simulator and fixture-generator binaries
└── ui/                # Terminal UI components
```

See [CLAUDE.md](CLAUDE.md) for detailed architecture documentation.

### Build & Test

```bash
cargo build            # Build
cargo run              # Run the game
make check             # Run all CI checks (format, lint, test, progression check, audit, coverage)
make fmt               # Auto-fix formatting
```

## License

This project is open source and available under the MIT License.
