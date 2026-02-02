# Auto-Update Feature Design

## Overview

On every startup, Quest checks GitHub for a newer release. If found, it shows a blocking modal with version comparison and changelog. The user can update (downloads binary, swaps on restart) or skip (proceeds to game).

## Startup Flow

```
Launch game
    ↓
[Pending update exists?] → Apply update (swap binary + re-exec) → New binary takes over
    ↓
Check GitHub API for latest release (~1 sec)
    ↓
Compare commit hash with compiled-in build hash
    ↓
[Same version] → Continue to character select
[Newer exists] → Fetch commit changelog → Show update modal
    ↓
[User chooses Update] → Backup saves → Download binary → Store pending → Exit with message
[User chooses Skip] → Continue to character select
```

Network failures are handled gracefully — if the API call fails, silently continue to the game.

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
├── backups/
│   └── 2026-02-02_183045/    # Timestamped backup
│       └── *.json
└── pending_update/
    ├── quest                  # Downloaded binary
    └── update.json            # {commit, date, downloaded_at}
```

On next launch, if `pending_update/` exists:
1. Replace current binary with the downloaded one
2. Delete `pending_update/` folder
3. Re-exec: replace current process with new binary (user sees seamless transition)

The re-exec ensures the user immediately runs the new version without needing a third launch.

## GitHub API Integration

### Get Latest Release

```
GET https://api.github.com/repos/stphung/quest/releases/latest
```

Returns: `tag_name`, `published_at`, `assets[{name, browser_download_url}]`

### Get Changelog (if update available)

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

## Update Modal UI

```
┌───────────────────────────────────────────────────────┐
│                  Update Available!                    │
│                                                       │
│  Your build:  Feb 01 (f54c32e)                        │
│  Latest:      Feb 02 (4993005)                        │
│                                                       │
│  ─────────────────────────────────────────────────    │
│  What's new:                                          │
│  • feat: add procedural dungeon exploration           │
│  • fix: character input text not visible              │
│  • feat: reset equipment on prestige                  │
│                                                       │
│         [U] Update now    [S] Skip for now            │
└───────────────────────────────────────────────────────┘
```

### Behavior

- Modal renders centered, sized to content (max ~60 chars wide)
- Changelog shows up to 10 most recent commits (truncate with "...and N more")
- Long commit messages truncated at ~50 chars with "..."
- `U` → Start update process
- `S` or `Esc` → Dismiss and continue to game

### Download Progress

```
Downloading update... 45%
```

### Completion

```
Update downloaded! Restart the game to apply.
Press any key to exit.
```

## Backup Mechanism

### Trigger

Immediately after user presses `U`, before download starts.

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
- Backup folder creation fails → Abort update, show error, continue to game
- Copy fails mid-backup → Delete partial backup folder, abort update

## Binary Swap Mechanism

A running binary cannot overwrite itself. The solution uses two launches:

**Launch 1 (download):**
1. User presses `U` to update
2. Download new binary to `~/.quest/pending_update/quest`
3. Game exits with "Restart to apply update"

**Launch 2 (swap + re-exec):**
1. Old binary starts, detects `pending_update/` exists
2. Old binary replaces itself on disk with new binary
3. Old binary re-execs into new binary (process replacement)
4. New binary starts fresh, deletes `pending_update/`, continues normally

### Platform-Specific Re-exec

**Unix (Linux/macOS):**
```rust
use std::os::unix::process::CommandExt;
std::process::Command::new(&new_binary_path)
    .args(std::env::args().skip(1))
    .exec(); // Replaces current process, never returns
```

**Windows:**
Windows doesn't have `exec()`. Instead:
1. Rename current binary to `quest.old`
2. Move new binary to `quest.exe`
3. Spawn new binary as child process
4. Exit current process
5. New binary deletes `quest.old` on startup

## Implementation

### New Files

- `src/updater.rs` — Update check, download, swap logic
- `src/ui/update_modal.rs` — Modal widget
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
| Check timing | On startup only |
| Version display | Build date + short commit hash |
| Update prompt | Blocking modal with changelog |
| Skip behavior | Always ask again next startup |
| Backup | Timestamped folder copy before download |
| API | GitHub Releases + Compare API (no auth needed) |
| Binary swap | Download to pending folder, swap + re-exec on next launch |
| HTTP client | `ureq` (simple, blocking) |
| Network failure | Silent continue to game |
