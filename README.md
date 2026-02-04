# quest

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS-blue.svg)](https://github.com/stphung/quest/releases/latest)

A terminal-based idle RPG game written in Rust. Watch your hero grow stronger automatically as they battle through 10 zones, explore procedural dungeons, and fish for legendary catches!

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
- **10 Zones** - Progress through 10 unique zones from Meadow to Storm Citadel, each with 3-4 subzones and bosses
- **6 Attributes** - STR, DEX, CON, INT, WIS, CHA form the foundation of your character
- **Prestige System** - Reset for permanent XP multipliers (1.5× per rank) and unlock higher zones
- **Procedural Dungeons** - Explore grid-based dungeons with fog of war, treasure rooms, elite guardians, and bosses
- **Fishing** - Separate progression track with 30 ranks and 5 fish rarities
- **Diablo-style Items** - 7 equipment slots, 5 rarity tiers, procedural names, and smart auto-equip
- **Multi-Character** - Create and manage multiple characters with JSON saves
- **Offline Progress** - Continue gaining XP even when closed (50% rate, max 7 days)
- **Challenge Minigames** - Discover and play Chess, Nine Men's Morris, Gomoku, and Minesweeper (requires P1+)
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
- Rust 1.70 or higher
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
- **Q**: Quit

### Gameplay
- **Q**: Quit the game
- **P**: Prestige (reset for XP multiplier, requires meeting level threshold)

## Game Systems

### Zones & Progression

Progress through 10 zones, each with 3-4 subzones and unique bosses:

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
- **XP Multiplier**: 1.5× per prestige rank (compounding)
- **Attribute Caps**: Base 10 + (5 × prestige rank)
- **Zone Unlocks**: Higher zones require prestige ranks
- **Better Item Drops**: +5% drop rate per prestige rank

Example progression: Bronze (1.5×) → Silver (2.25×) → Gold (3.375×) → Platinum → Diamond → Celestial...

### Dungeons

Procedural grid-based exploration:
- **Sizes**: Small (5×5), Medium (7×7), Large (9×9), Epic (11×11) based on prestige
- **Room Types**: Combat, Treasure (guaranteed item), Elite (key guardian), Boss
- **Key System**: Defeat Elite guardian to get key for Boss room
- **Fog of War**: Rooms revealed as you explore
- **Safe Death**: No prestige loss when dying in dungeons

### Fishing

Separate progression track with 30 ranks across 6 tiers:
- Novice → Apprentice → Journeyman → Expert → Master → Grandmaster
- Fish rarities: Common, Uncommon, Rare, Epic, Legendary
- Higher ranks improve catch quality

### Challenge Minigames

Discover challenge minigames while adventuring (requires Prestige 1+):

- **Chess** - Play against AI with 4 difficulty levels (Novice ~500 ELO to Master ~1350 ELO)
- **Nine Men's Morris** - Classic strategy board game against AI opponents
- **Gomoku** - Five-in-a-row on a 15×15 board with minimax AI (4 difficulty levels)
- **Minesweeper (Trap Detection)** - Clear minefields across 4 difficulty levels (9×9 to 20×16)
- Challenges appear randomly (~2 hour average discovery time)
- Accept or decline from the challenge menu
- Winning rewards prestige points based on difficulty

### Items & Equipment

**7 Equipment Slots**: Weapon, Armor, Helmet, Gloves, Boots, Amulet, Ring

**5 Rarity Tiers**:
| Rarity | Attributes | Affixes |
|--------|-----------|---------|
| Common | +1-2 | 0 |
| Magic | +2-4 | 1 |
| Rare | +4-7 | 2 |
| Epic | +6-10 | 3 |
| Legendary | +8-15 | 4-5 |

- Procedural name generation with prefixes/suffixes
- Smart auto-equip based on weighted scoring
- Drop rate: 30% base + 5% per prestige rank

## Save System

- **Location**: `~/.quest/` directory (JSON format)
- **Multi-character**: Each character saved separately
- **Auto-save**: Every 30 seconds
- **Offline Progress**: Simulates kills at 50% rate (max 7 days)

## Technical Details

- Built with [Ratatui](https://github.com/ratatui-org/ratatui) for terminal UI
- Uses [Crossterm](https://github.com/crossterm-rs/crossterm) for cross-platform terminal handling
- Save files use JSON format
- 100ms game tick (10 ticks/sec)

## License

This project is open source and available under the MIT License.
