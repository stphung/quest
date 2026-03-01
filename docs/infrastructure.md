# Infrastructure Design

This document describes the CI/CD pipeline, auto-update system, and debug menu as implemented.

## CI/CD Pipeline

### Local Development

```bash
make setup     # First time: configure git hooks
make check     # Run all CI checks locally (same script as CI)
make fmt       # Auto-fix formatting
cargo run      # Run the game
```

### Quality Checks (`scripts/ci-checks.sh`)

Both local `make check` and CI run the exact same script:

1. `cargo fmt --check` — Format checking
2. `cargo clippy --all-targets -- -D warnings` — Lint checking
3. `cargo test` — All tests
4. `cargo build --all-targets` — Build verification
5. `cargo audit --deny yanked` — Security audit

### CI Workflows

**On every PR:**
- Runs `scripts/ci-checks.sh`
- Must pass to merge

**On push to main:**
- Runs all checks
- Builds release binaries for 3 platforms:
  - Linux x86_64
  - macOS x86_64
  - macOS ARM64 (aarch64)
- Signs macOS binaries with ad-hoc signature (prevents Gatekeeper blocking)
- Creates GitHub release with downloadable binaries

## Auto-Update System

### Overview

Quest supports self-updating via a CLI command. On game startup, checks for updates and displays a notification. The user runs `quest update` to download and install.

### Commands

```
quest           → Run game (shows update notification if available)
quest update    → Check for updates and install
```

### Build Identity

Build info embedded at compile time via `build.rs`:
- `BUILD_COMMIT` — Short commit hash (7 chars)
- `BUILD_DATE` — ISO date string

### Startup Check Flow

```
Launch game
  → Display splash screen (QUEST ASCII art, achievement badges, journey badges)
  → Check GitHub API for latest release (~1 sec, braille spinner while checking)
  → Compare commit hash with compiled-in build hash
  → Same version → show "Latest" with commit hash
  → Newer exists → show animated "Update" indicator with new commit hash
  → Network failure → silently continue to character select
```

The splash screen shows achievement score badge (if > 0), character journey badges for discovered systems (zones, challenges, fishing), and the character select list below.

Update checks run every 30 minutes (`UPDATE_CHECK_INTERVAL_SECONDS = 1800`).

### Update Command Flow

```
quest update
  → Check GitHub API for latest release
  → Already latest → "You're up to date" → exit
  → Update available → show changelog (commit messages)
  → Backup saves to ~/.quest/backups/YYYY-MM-DD_HHMMSS/
  → Download new binary (platform-appropriate)
  → Replace current binary on disk
  → macOS: ad-hoc code sign new binary
  → "Updated successfully! Run 'quest' to play." → exit
```

### Platform Asset Selection

| Platform | Asset Name |
|----------|------------|
| Linux x86_64 | `quest-x86_64-unknown-linux-gnu.tar.gz` |
| macOS Intel | `quest-x86_64-apple-darwin.tar.gz` |
| macOS ARM | `quest-aarch64-apple-darwin.tar.gz` |
| Windows | `quest-x86_64-pc-windows-msvc.zip` |

### Binary Replacement

- **Unix**: Overwrite file on disk (OS keeps old binary in memory until process exits)
- **Windows**: Rename current to `.old`, move new into place, delete `.old`
- **macOS**: After replacement, run ad-hoc `codesign` to prevent Gatekeeper blocking

### Backup Mechanism

Before downloading:
1. Create `~/.quest/backups/YYYY-MM-DD_HHMMSS/`
2. Copy all `*.json` files from `~/.quest/`
3. Proceed to download

All backups kept permanently. Manual cleanup by user.

### GitHub API

- Latest release: `GET https://api.github.com/repos/stphung/quest/releases/latest`
- Changelog: `GET https://api.github.com/repos/stphung/quest/compare/{old}...{new}`
- Release tag format: `build-{full_commit_hash}`

### Dependencies

- `ureq` — Blocking HTTP client
- `flate2` — Gzip decompression
- `tar` — Tar archive extraction

## Headless Game Simulator

### Overview

A separate binary (`src/bin/simulator.rs`) that runs the game tick loop without any UI, collecting metrics for game balance analysis. Uses the exact same `game_tick()` function as the real game, ensuring perfect fidelity.

### Usage

```bash
cargo run --bin simulator -- [OPTIONS]
```

### Options

| Option | Default | Description |
|--------|---------|-------------|
| `--ticks N` | 36000 | Ticks to simulate (36000 = 1 hour game time) |
| `--seed N` | 42 | RNG seed for reproducibility |
| `--prestige N` | 0 | Starting prestige rank |
| `--runs N` | 1 | Number of runs with incrementing seeds |
| `--verbose` | off | Per-tick event logging |
| `--csv FILE` | none | Write time-series CSV (snapshot every 100 ticks) |
| `--quiet` | off | Only final summary line |
| `--stormbreaker` | off | Force-unlock TheStormbreaker achievement for Zone 10+ testing |
| `--haven STR` | none | Haven auto-build strategy: `combat`, `qol`, `balanced`, `full` |

### Haven Auto-Building

`--haven <strategy>` enables automatic Haven room construction during simulation. When enabled, Haven is force-discovered at start and prestige ranks are spent on rooms each tick following the strategy's priority order:

- **combat**: Armory/damage path (Hearthstone → Armory → Training Yard → Trophy Hall → Watchtower → Alchemy Lab → War Room)
- **qol**: Bedroom/fishing path (Hearthstone → Bedroom → Garden → Library → Fishing Dock → Workshop → Vault)
- **balanced**: Both branches interleaved
- **full**: Everything including StormForge

This models the real gameplay trade-off between investing prestige in Haven vs keeping it for combat bonuses.

### Tracked Metrics

- Combat: kills, deaths, boss kills, crits, total XP
- Items: drops by rarity, equipped count, boss drops
- Progression: level milestones (tick at which each level was reached), zone entry times
- Fishing: fish caught, rank-ups, final rank
- Dungeons: discovered, completed, failed
- Achievements: total unlocked, Haven discovery
- Deaths per zone (for balance tuning)

### Deep Economy Simulator

A separate binary (`src/bin/deep_simulator.rs`) for testing The Deep's mercenary economy in isolation. Simulates mission cycles, recruitment, infrastructure building, and guild rank progression without a UI.

```bash
cargo run --bin deep_simulator -- [OPTIONS]
```

| Option | Default | Description |
|--------|---------|-------------|
| `--hours N` | 168 | Hours to simulate |
| `--seed N` | 42 | RNG seed for reproducibility |
| `--strategy STR` | balanced | Strategy: `rush`, `balanced`, `infrastructure` |
| `--guild-rank N` | 1 | Starting guild rank |
| `--verbose` | off | Detailed event logging |
| `--quiet` | off | Only final summary line |

### Multi-Run Aggregation

With `--runs N`, the simulator runs N simulations with incrementing seeds and produces an aggregate report with min/avg/max for all metrics, plus a final zone distribution across runs.

### CSV Output

The `--csv` option writes a time-series with columns: tick, game_time_s, level, xp, zone_id, subzone_id, prestige_rank, total_kills, total_deaths, fishing_rank, items_found. Useful for graphing progression curves.

## Debug Menu

### Activation

```bash
cargo run -- --debug
```

When active, a `[DEBUG]` indicator shows in the UI corner.

### Menu Access

- Press backtick (`` ` ``) to toggle debug menu overlay
- Arrow keys to navigate, Enter to trigger, backtick to close

### Menu Options

The debug menu uses a tabbed category structure with 8 tabs. Left/Right arrows switch tabs, Up/Down navigate within a tab, Enter triggers.

**Challenges tab:**
- Trigger Chess, Morris, Gomoku, Minesweeper, Rune, Go, Flappy Bird, JezzBall, Snake, Sigil Surge challenges

**World tab:**
- Trigger Dungeon, Trigger Fishing, Trigger Haven Discovery, Trigger Soulforge Discovery

**Resources tab:**
- Grant 1000 Stormglass, Discover Stormglass, Grant 100k Stormglass, Etch Random Sigils (All Slots), Etch S+ Sigil (Slot 1), Force Next Surge Overcharged

**Items tab:**
- Forge Asprika, Forge Sleipnir, Forge Megingjord (God Items)

**Deep tab:**
- Discover The Deep, Grant 10000 Warband Marks, Refresh Mission Pool, Refresh Recruit Pool, Clear Current Frontier Layer, Complete Active Missions

**Zones tab:**
- Zone/fracture zone travel options

**Character tab:**
- Character-level debug options

**Borders tab:**
- Border style options for visual customization

Each option calls existing generation functions to bypass the normal RNG discovery system. Useful for testing features without waiting for random events.

### UI Style

Yellow border popup overlay with tabbed header, centered on screen with fixed panel size.

### Debug Mode Behavior

When `--debug` is active:
- **Saves disabled**: File I/O (`save_character()`, `save_haven()`, `save_achievements()`) is skipped
- **`last_save_time` always synced**: The in-memory `state.last_save_time = Utc::now().timestamp()` is updated every autosave cycle regardless of debug mode, preventing the suspension detection system from false-triggering
- **Save signals suppressed**: `TickResult.achievements_changed`, `haven_changed`, `enhancement_changed`, `god_items_changed`, and `deep_changed` flags are suppressed in `tick.rs` when `debug_mode` is true

### Suspension Detection

The game detects OS-level process suspension (e.g., laptop lid close/open):
- Autosave syncs `last_save_time` every 30 seconds
- Each frame checks if `Utc::now() - last_save_time > 60s`
- If gap detected: shows offline XP welcome screen (via `process_offline_progression()`), resets tick/autosave timers, and immediately saves

## Time Vault / History System

Git-based save versioning system (`src/history/`). Every meaningful game event (prestige, boss defeat, etc.) creates a git commit containing the full save state. Players can browse, restore, and fork save branches through the Time Vault overlay.

- **Git repository**: `~/.quest/.history/` — initialized on first use
- **Commit triggers**: Prestige, boss defeat, zone completion, and other milestone events
- **Branch management**: Players can create branches (forks) from any commit point
- **Cloud sync**: Optional GitHub integration via personal access token. Config stored in `~/.quest/.cloud.json`. Supports push/pull, divergence detection, and resolution (cloud wins / local wins / keep both).
- **Cloud status states**: Offline, Linked, Syncing, OutOfSync (diverged), TokenExpired, Error

## Bug Report System

In-game bug report overlay (`src/utils/bug_report.rs`, `src/ui/bug_report_scene.rs`) that captures game state for troubleshooting. Generates a text summary of current state that can be copied to clipboard.

## Storage Layout

```
~/.quest/
├── <character_name>.json     # Character saves (max 3)
├── haven.json                # Haven state (account-level)
├── achievements.json         # Achievement state (account-level)
├── enhancement.json          # Soulforge enhancement state (account-level)
├── deep.json                 # The Deep state (account-level)
├── .cloud.json               # GitHub cloud sync config (token, username, repo URL)
├── .history/                 # Git repository for Time Vault save versioning
│   └── (git repo contents)
└── backups/
    └── YYYY-MM-DD_HHMMSS/   # Timestamped backup before update
        └── *.json
```

### Key Character Save Fields

Character JSON files contain the full `GameState` struct. Notable fields that persist across prestige:

- `prestige_rank`, `total_prestiges` -- Prestige progression
- `fishing_rank`, `total_fish_caught`, `legendary_fish_caught` -- Fishing state
- `ascension_level` -- Ascension combat multiplier level (survives prestige)
- `stormglass` -- Stormglass currency balance

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| ratatui | 0.30 | Terminal UI framework |
| crossterm | (transitive) | Terminal backend (pulled in via ratatui, not a direct dependency) |
| serde / serde_json | - | JSON serialization |
| rand | - | RNG for all procedural systems |
| chrono | - | Date/time for offline progression |
| dirs | 6.0 | Platform-appropriate save paths |
| git2 | 0.20 | Git operations for Time Vault history |
| tar | 0.4 | Archive extraction for updates |
| uuid | 1.21 | Character IDs |
| serde_json | 1.0 | JSON serialization |
| chess-engine | 0.1 | Chess minigame AI |
| ureq | - | HTTP client for auto-update |
| flate2 / tar | - | Archive extraction for updates |
| rand_chacha | - | Seedable RNG for simulator and tests |
