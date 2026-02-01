# shell.rpg

A terminal-based idle RPG game written in Rust. Watch your hero grow stronger automatically as they battle enemies across different zones!

> **Why "shell.rpg"?** Because it runs in your shell (terminal) and it's an RPG. Simple, clean, and perfectly nerdy.

## Features

- **Automatic Progression**: Your character gains XP and levels up automatically
- **4 Core Stats**: Strength, Magic, Wisdom, and Vitality all level up independently
- **Prestige System**: Reset your progress for permanent XP multipliers
- **5 Unique Zones**: Travel from Meadow to Volcanic Wastes as you level up
- **Combat System**: Watch your hero battle enemies with visual animations
- **Offline Progress**: Continue gaining XP even when the game is closed (at 50% rate)
- **Auto-Save**: Your progress is automatically saved every 30 seconds

## Installation

### Prerequisites

- Rust 1.70 or higher
- Cargo (comes with Rust)

### Building from Source

```bash
# Clone the repository
git clone <repository-url>
cd shell-rpg

# Build and run
cargo run --release
```

## Controls

- **Q**: Quit the game
- **P**: Prestige (resets stats for an XP multiplier, requires all stats at minimum level)

## Game Mechanics

### Stats

Each stat levels up independently based on XP gain:
- **Strength**: Physical power
- **Magic**: Magical ability
- **Wisdom**: Knowledge and insight
- **Vitality**: Health and endurance

XP required for next level follows the formula: `100 × level^1.5`

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

- Enemies spawn every 2.5 seconds
- Each spawn triggers an attack animation
- Enemy type is randomly selected from the current zone
- Combat is purely visual - progression is automatic

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
