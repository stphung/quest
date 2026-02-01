# quest

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-blue.svg)](https://github.com/stphung/quest/releases/latest)

A terminal-based idle RPG game written in Rust. Watch your hero grow stronger automatically as they battle enemies across different zones!

> **Why "quest"?** Because that's exactly what it is. Simple, memorable, and to the point.

## 🚀 Quick Start

**Install and play with one command:**

**macOS / Linux:**
```bash
curl -sSf https://raw.githubusercontent.com/stphung/quest/main/install.sh | sh
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/stphung/quest/main/install.ps1 | iex
```

Then run `quest` to start your adventure!

## ✨ Features

- ⚡ **Automatic Progression** - Your character gains XP and levels up automatically
- 💪 **6 Attributes** - STR, DEX, CON, INT, WIS, CHA form the foundation of your character
- 📊 **Derived Combat Stats** - HP, damage, defense, and crit chance calculated from attributes
- ⚔️ **Dynamic Combat** - Real-time battles with enemies that scale to your power
- 🔄 **Prestige System** - Reset your progress for permanent XP multipliers and higher attribute caps
- 🗺️ **5 Unique Zones** - Travel from Meadow to Volcanic Wastes as you level up
- 💤 **Offline Progress** - Continue gaining XP even when the game is closed (at 50% rate)
- 💾 **Auto-Save** - Your progress is automatically saved every 30 seconds

## Installation

### Quick Install (Recommended)

**macOS / Linux:**
```bash
curl -sSf https://raw.githubusercontent.com/stphung/quest/main/install.sh | sh
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/stphung/quest/main/install.ps1 | iex
```

The installer will:
- Download the latest release for your platform
- Install to `~/.local/bin/quest` (or `%USERPROFILE%\.local\bin\quest.exe` on Windows)
- Provide instructions to add to PATH if needed

### Manual Download

Download the latest release for your platform from the [releases page](https://github.com/stphung/quest/releases/latest).

**Supported platforms:**
- Linux (x86_64)
- macOS (Intel x86_64 and Apple Silicon ARM64)
- Windows (x86_64)

### Building from Source

**Prerequisites:**
- Rust 1.70 or higher
- Cargo (comes with Rust)

```bash
# Clone the repository
git clone https://github.com/stphung/quest.git
cd quest

# Build and run
cargo run --release
```

## Controls

- **Q**: Quit the game
- **P**: Prestige (resets stats for an XP multiplier, requires all stats at minimum level)

## Game Mechanics

### Stat System

Your character has **one unified level** and **six core attributes** that increase randomly on level-up:

**Attributes**:
- **Strength (STR)**: Increases physical damage in combat
- **Dexterity (DEX)**: Increases defense and critical hit chance
- **Constitution (CON)**: Increases maximum HP
- **Intelligence (INT)**: Increases magic damage in combat
- **Wisdom (WIS)**: Increases passive XP gain rate
- **Charisma (CHA)**: Boosts prestige XP multiplier

Each attribute has a **modifier** calculated as `(value - 10) / 2`. At base (10), modifier is +0. At 16, modifier is +3.

**Derived Stats**:
- **Max HP**: 50 + (CON modifier × 10)
- **Physical Damage**: 5 + (STR modifier × 2)
- **Magic Damage**: 5 + (INT modifier × 2)
- **Defense**: DEX modifier (reduces incoming damage)
- **Crit Chance**: 5% + (DEX modifier × 1%) - crits deal 2× damage
- **XP Multiplier**: 1.0 + (WIS modifier × 0.05)

**Attribute Caps**: Attributes are capped at `10 + (prestige_rank × 5)`. Prestiging increases your caps!

**Level-Up**: Gain 3 random attribute points (distributed among non-capped attributes)

XP required for next level: `100 × level^1.5`

For complete stat system documentation, see [docs/STAT_SYSTEM.md](docs/STAT_SYSTEM.md)

### XP Gain

- Base XP: 1.0 XP per tick (10 ticks per second)
- Prestige multiplier: 1.5^(prestige rank)
- Offline progression: 50% of online rate (capped at 7 days)

### Prestige Tiers

- **Bronze** (Rank 1): Level 10 required, 1.5× XP multiplier
- **Silver** (Rank 2): Level 25 required, 2.25× XP multiplier
- **Gold** (Rank 3): Level 50 required, 3.375× XP multiplier
- **Platinum** (Rank 5): Level 75 required, 7.59× XP multiplier
- **Diamond** (Rank 10): Level 100 required, 57.67× XP multiplier
- **Celestial** (Rank 15): Level 150 required, 437.89× XP multiplier

### Zones

Your current zone is determined by your average level:

1. **Meadow** (Levels 0-10): Fight Slimes, Rabbits, Ladybugs, and Butterflies
2. **Dark Forest** (Levels 10-25): Battle Wolves, Spiders, Dark Elves, and Bats
3. **Mountain Pass** (Levels 25-50): Face Golems, Yetis, Mountain Lions, and Eagles
4. **Ancient Ruins** (Levels 50-75): Confront Skeletons, Ghosts, Ancient Guardians, and Wraiths
5. **Volcanic Wastes** (Levels 75-100): Challenge Fire Elementals, Lava Beasts, Phoenixes, and Dragons

### Combat

- **Turn-Based**: Combat rounds occur every 1.5 seconds
- **Dynamic Scaling**: Enemy stats scale with your power (80-120% of your HP)
- **Critical Hits**: Based on DEX, critical hits deal 2× damage
- **Defense**: Your DEX-based defense reduces incoming damage
- **XP Rewards**: Killing enemies grants bonus XP (50-100 ticks worth)
- **HP Regeneration**: After killing an enemy, your HP regenerates over 2.5 seconds
- **Death Penalty**: Dying resets you to full HP but you lose all prestige ranks
- **Enemy Names**: Dynamically generated with procedural combinations

## Save System

- Save file location: `~/.local/share/idle-rpg/save.dat` (Linux/macOS) or `%APPDATA%\idle-rpg\save.dat` (Windows)
- Saves are checksummed to prevent corruption
- Auto-saves every 30 seconds
- Manual save on exit

## Technical Details

- Built with [Ratatui](https://github.com/ratatui-org/ratatui) for terminal UI
- Uses [Crossterm](https://github.com/crossterm-rs/crossterm) for cross-platform terminal handling
- Save files use [Bincode](https://github.com/bincode-org/bincode) serialization
- Checksums via [SHA-256](https://github.com/RustCrypto/hashes)

## License

This project is open source and available under the MIT License.
