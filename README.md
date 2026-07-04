# quest

[![CI](https://github.com/stphung/quest/actions/workflows/ci.yml/badge.svg)](https://github.com/stphung/quest/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS-blue.svg)](https://github.com/stphung/quest/releases/latest)

A terminal-based idle RPG written in Rust. Watch your hero grow stronger automatically as they battle through 50 zones, explore procedural dungeons, fish for legendary catches, and dive into layered endgame systems — prestige, Ascension, the Deep, and the Loom of Worlds.

> **Why "quest"?** Because that's exactly what it is. Simple, memorable, and to the point.

## Quick Start

**Install and play with one command (macOS / Linux):**

```bash
curl -sSf https://raw.githubusercontent.com/stphung/quest/main/install.sh | sh
```

The installer downloads the latest release for your platform, installs to `~/.local/bin/quest`, and tells you how to add it to your `PATH` if needed. Then run `quest` to start your adventure.

### Updating

```bash
quest update
```

The binary self-updates to the latest release.

### Manual Download

Grab a binary for your platform from the [releases page](https://github.com/stphung/quest/releases/latest).

**Supported platforms:**
- Linux (x86_64)
- macOS (Intel x86_64 and Apple Silicon ARM64)

### Building from Source

**Prerequisites:** a recent stable Rust toolchain and Cargo (CI builds on latest stable).

```bash
git clone https://github.com/stphung/quest.git
cd quest
cargo run --release
```

## Playing the Game

Quest plays itself — your hero fights, levels, and loots automatically. You steer the meta: when to prestige, which endgame systems to invest in, and which challenges to accept.

Controls are shown in-game, and pressing **W** opens the [wiki](https://github.com/stphung/quest/wiki) in your browser.

**For the full picture — zones, attributes, prestige, dungeons, fishing, items, Haven, Ascension, the Deep, the Loom, and every other system — see the [Quest Wiki](https://github.com/stphung/quest/wiki).**

## Development

Contributions welcome. The wiki covers gameplay; this section and [CLAUDE.md](CLAUDE.md) cover the code.

### Setup

```bash
make setup             # First time: configure git hooks
cargo build            # Build
cargo run              # Run the game
```

### Build & Test

```bash
make check             # Run all CI checks locally (format, lint, test, progression check, audit, coverage)
make fmt               # Auto-fix formatting
```

Run `make check` before pushing — it mirrors the PR checks. See [CLAUDE.md](CLAUDE.md) for the targeted verification steps that match what you touched (snapshot tests, simulators, save-compat tests, and more).

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
├── challenges/        # Challenge minigames
├── haven/             # Account-level base building
├── achievements/      # Achievement tracking system
├── history/           # Git-based save versioning (Time Vault)
├── vessel/            # Act 2 Vessel/Voyage (dark behind kill-switch)
├── main_helpers/      # Orchestration between main.rs and domain modules
├── utils/             # Build info, updater, debug menu
├── bin/               # Simulator and fixture-generator binaries
└── ui/                # Terminal UI components
```

Larger modules have their own `CLAUDE.md` with implementation patterns and extension guides. Start from the root [CLAUDE.md](CLAUDE.md) for the full architecture map.

### Tech Stack

Built with [Ratatui](https://github.com/ratatui-org/ratatui) and [Crossterm](https://github.com/crossterm-rs/crossterm) for the terminal UI, a 100ms game tick (10 ticks/sec), and JSON save files stored in `~/.quest/`.

## License

Open source under the MIT License.
