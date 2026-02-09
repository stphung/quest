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
