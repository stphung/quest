# Auto-Update Feature Design

## Overview

Quest supports self-updating via a CLI command. On game startup, it checks for updates and displays a notification if one is available. The user runs `quest update` to download and install.

## Commands

```
quest           → Run the game (shows update notification if available)
quest update    → Check for updates and install if available
```

## Game Startup Flow

```
Launch game
    ↓
Check GitHub API for latest release (~1 sec)
    ↓
Compare commit hash with compiled-in build hash
    ↓
[Same version] → Continue to character select
[Newer exists] → Show banner: "Update available (Feb 02). Run 'quest update' to install."
    ↓
Continue to character select
```

Network failures are handled gracefully — if the API call fails, silently continue to the game.

## Update Command Flow

```
quest update
    ↓
Check GitHub API for latest release
    ↓
[Already latest] → "You're up to date (Feb 02, 4993005)" → Exit
    ↓
[Update available] → Show changelog
    ↓
Backup saves to ~/.quest/backups/YYYY-MM-DD_HHMMSS/
    ↓
Download new binary (show progress)
    ↓
Replace current binary on disk
    ↓
"Updated to Feb 02 (4993005). Run 'quest' to play." → Exit
```

## Build Identity

Embed build info at compile time via `build.rs`:

```rust
pub const BUILD_COMMIT: &str = "f54c32e";
pub const BUILD_DATE: &str = "2026-02-02";
```

CI sets these via environment variables during release builds.

## Storage Layout

```
~/.quest/
├── characters/
│   └── *.json
└── backups/
    └── 2026-02-02_183045/    # Timestamped backup before update
        └── *.json
```

No pending update folder needed — updates apply immediately.

## GitHub API Integration

### Get Latest Release

```
GET https://api.github.com/repos/stphung/quest/releases/latest
```

Returns: `tag_name`, `published_at`, `assets[{name, browser_download_url}]`

### Get Changelog

```
GET https://api.github.com/repos/stphung/quest/compare/{old}...{new}
```

Returns: `commits[{sha, commit.message}]`

### Release Tag Parsing

Current format: `build-4993005923a4924e5c338655f872ea9ebc9efe10`

Extract short commit: first 7 chars after `build-` → `4993005`

### Platform Asset Selection

| Platform | Asset name |
|----------|------------|
| Linux | `quest-x86_64-unknown-linux-gnu.tar.gz` |
| macOS Intel | `quest-x86_64-apple-darwin.tar.gz` |
| macOS ARM | `quest-aarch64-apple-darwin.tar.gz` |
| Windows | `quest-x86_64-pc-windows-msvc.zip` |

## CLI Output

### Update Available (game startup)

```
╭─────────────────────────────────────────────────────╮
│  Update available! (Feb 02, 4993005)                │
│  Run 'quest update' to install.                     │
╰─────────────────────────────────────────────────────╯
```

Shown briefly (2 sec) then continues to character select.

### quest update (already current)

```
Checking for updates...
You're up to date.
  Current: Feb 02 (4993005)
```

### quest update (update available)

```
Checking for updates...

Update available!
  Your build:  Feb 01 (f54c32e)
  Latest:      Feb 02 (4993005)

What's new:
  • feat: add procedural dungeon exploration
  • fix: character input text not visible
  • feat: reset equipment on prestige

Backing up saves... done
Downloading update... 100%
Installing... done

Updated successfully! Run 'quest' to play.
```

## Binary Replacement

The `quest update` command replaces its own binary on disk:

1. Download new binary to temp file
2. Replace current executable with new binary
3. Exit

This works because:
- **Unix:** The OS keeps the old binary in memory until the process exits. Overwriting the file on disk is allowed.
- **Windows:** Rename current binary to `.old`, move new binary into place, delete `.old`.

No re-exec needed — user simply runs `quest` again to use the new version.

## Backup Mechanism

### Trigger

Before downloading the update in `quest update`.

### Process

1. Create folder: `~/.quest/backups/YYYY-MM-DD_HHMMSS/`
2. Copy all `*.json` files from `~/.quest/` to the backup folder
3. Proceed to download

### Retention

Keep all backups. User can manually clean up.

### Restore

Manual — user copies files back from backup folder if needed.

### Edge Cases

- No characters yet → Skip backup, proceed to download
- Backup folder creation fails → Abort update, show error
- Copy fails mid-backup → Delete partial backup folder, abort update

## Implementation

### New Files

- `src/updater.rs` — Update check, download, binary replacement logic
- `src/cli.rs` — Argument parsing for `quest` vs `quest update`
- `build.rs` — Embed commit/date at compile time

### New Dependencies

- `ureq` — Blocking HTTP client (simple, minimal)
- `flate2` — Gzip decompression for .tar.gz assets
- `tar` — Tar archive extraction
- `zip` — Zip extraction (Windows)

### CI Changes

Pass environment variables during release builds:
- `BUILD_COMMIT` — Short commit hash (7 chars)
- `BUILD_DATE` — ISO date string

## Decisions

| Aspect | Decision |
|--------|----------|
| Update trigger | CLI command (`quest update`) |
| Check timing | On startup (notification only) |
| Version display | Build date + short commit hash |
| Changelog | Shown during `quest update` |
| Backup | Timestamped folder copy before download |
| API | GitHub Releases + Compare API (no auth needed) |
| Binary replacement | Direct overwrite (Unix) / rename swap (Windows) |
| HTTP client | `ureq` (simple, blocking) |
| Network failure | Silent continue (game) / Show error (update command) |
