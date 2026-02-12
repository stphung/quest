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
  → Check GitHub API for latest release (~1 sec)
  → Compare commit hash with compiled-in build hash
  → Same version → continue to character select
  → Newer exists → show banner: "Update available. Run 'quest update'"
  → Network failure → silently continue
```

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

### Tracked Metrics

- Combat: kills, deaths, boss kills, crits, total XP
- Items: drops by rarity, equipped count, boss drops
- Progression: level milestones (tick at which each level was reached), zone entry times
- Fishing: fish caught, rank-ups, final rank
- Dungeons: discovered, completed, failed
- Achievements: total unlocked, Haven discovery
- Deaths per zone (for balance tuning)

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

1. **Trigger Dungeon** — Spawns a dungeon immediately
2. **Trigger Fishing** — Starts a fishing session immediately
3. **Trigger Chess Challenge** — Adds chess to challenge menu
4. **Trigger Morris Challenge** — Adds morris to challenge menu
5. **Trigger Gomoku Challenge** — Adds gomoku to challenge menu
6. **Trigger Minesweeper Challenge** — Adds minesweeper to challenge menu
7. **Trigger Rune Challenge** — Adds rune challenge to challenge menu
8. **Trigger Go Challenge** — Adds go to challenge menu
9. **Trigger Haven Discovery** — Discovers Haven immediately

Each option calls existing generation functions to bypass the normal RNG discovery system. Useful for testing features without waiting for random events.

### UI Style

Yellow border popup overlay, centered on screen. Matches challenge menu styling.

### Debug Mode Behavior

When `--debug` is active:
- **Saves disabled**: File I/O (`save_character()`, `save_haven()`, `save_achievements()`) is skipped
- **`last_save_time` always synced**: The in-memory `state.last_save_time = Utc::now().timestamp()` is updated every autosave cycle regardless of debug mode, preventing the suspension detection system from false-triggering
- **Save signals suppressed**: `TickResult.achievements_changed` and `haven_changed` flags are suppressed in `tick.rs` when `debug_mode` is true

### Suspension Detection

The game detects OS-level process suspension (e.g., laptop lid close/open):
- Autosave syncs `last_save_time` every 30 seconds
- Each frame checks if `Utc::now() - last_save_time > 60s`
- If gap detected: shows offline XP welcome screen (via `process_offline_progression()`), resets tick/autosave timers, and immediately saves

## Storage Layout

```
~/.quest/
├── <character_name>.json     # Character saves (max 3)
├── haven.json                # Haven state (account-level)
├── achievements.json         # Achievement state (account-level)
└── backups/
    └── YYYY-MM-DD_HHMMSS/   # Timestamped backup before update
        └── *.json
```

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| ratatui | 0.26 | Terminal UI framework |
| crossterm | 0.27 | Terminal backend |
| serde / serde_json | - | JSON serialization |
| rand | - | RNG for all procedural systems |
| chrono | - | Date/time for offline progression |
| directories | - | Platform-appropriate save paths |
| chess-engine | 0.1 | Chess minigame AI |
| ureq | - | HTTP client for auto-update |
| flate2 / tar | - | Archive extraction for updates |
| rand_chacha | - | Seedable RNG for simulator and tests |
