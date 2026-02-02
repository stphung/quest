# Auto-Update Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add self-updating capability via `quest update` command with startup notification.

**Architecture:** CLI argument parsing routes to update command or game. Update checks GitHub API for latest release, compares embedded build hash, downloads platform-specific binary, backs up saves, and replaces itself. Game startup shows non-blocking notification if update available.

**Tech Stack:** ureq (HTTP), flate2+tar (Unix archives), zip (Windows archives), build.rs (compile-time embedding)

---

## Task 1: Add Dependencies

**Files:**
- Modify: `Cargo.toml`

**Step 1: Add new dependencies to Cargo.toml**

Add these dependencies after the existing ones:

```toml
ureq = { version = "2.9", features = ["json"] }
flate2 = "1.0"
tar = "0.4"
zip = "0.6"
```

**Step 2: Verify dependencies resolve**

Run: `cargo check`
Expected: Compiles successfully (downloads new crates)

**Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: add dependencies for auto-update feature

- ureq: HTTP client for GitHub API
- flate2: gzip decompression
- tar: tar archive extraction
- zip: zip extraction (Windows)

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 2: Create Build Info Embedding

**Files:**
- Create: `build.rs`
- Create: `src/build_info.rs`
- Modify: `src/main.rs` (add module)

**Step 1: Create build.rs in project root**

```rust
//! Build script to embed commit hash and build date at compile time.

use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    // Get commit from env var (CI) or git command (local dev)
    let commit = env::var("BUILD_COMMIT").unwrap_or_else(|_| {
        Command::new("git")
            .args(["rev-parse", "--short=7", "HEAD"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    });

    // Get date from env var (CI) or current date (local dev)
    let date = env::var("BUILD_DATE").unwrap_or_else(|_| {
        chrono::Utc::now().format("%Y-%m-%d").to_string()
    });

    // Write to OUT_DIR for inclusion
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("build_info.rs");

    fs::write(
        &dest_path,
        format!(
            r#"pub const BUILD_COMMIT: &str = "{}";
pub const BUILD_DATE: &str = "{}";"#,
            commit, date
        ),
    )
    .unwrap();

    // Rerun if git HEAD changes
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-env-changed=BUILD_COMMIT");
    println!("cargo:rerun-if-env-changed=BUILD_DATE");
}
```

**Step 2: Add chrono as build dependency**

Add to `Cargo.toml` at the end:

```toml
[build-dependencies]
chrono = "0.4"
```

**Step 3: Create src/build_info.rs**

```rust
//! Compile-time build information.

include!(concat!(env!("OUT_DIR"), "/build_info.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_info_not_empty() {
        assert!(!BUILD_COMMIT.is_empty());
        assert!(!BUILD_DATE.is_empty());
    }

    #[test]
    fn test_build_commit_format() {
        // Should be 7 chars or "unknown"
        assert!(BUILD_COMMIT == "unknown" || BUILD_COMMIT.len() == 7);
    }

    #[test]
    fn test_build_date_format() {
        // Should be YYYY-MM-DD format
        assert!(BUILD_DATE.len() == 10 || BUILD_DATE == "unknown");
    }
}
```

**Step 4: Add module to main.rs**

Add after `mod ui;`:

```rust
mod build_info;
```

**Step 5: Verify build works**

Run: `cargo build`
Expected: Compiles successfully

**Step 6: Run tests**

Run: `cargo test build_info`
Expected: All tests pass

**Step 7: Commit**

```bash
git add build.rs Cargo.toml src/build_info.rs src/main.rs
git commit -m "feat: embed build commit and date at compile time

- build.rs generates build_info.rs with commit hash and date
- Falls back to git command for local development
- CI can override via BUILD_COMMIT and BUILD_DATE env vars

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 3: Update CI to Pass Build Info

**Files:**
- Modify: `.github/workflows/ci.yml`

**Step 1: Add environment variables to build step**

In the `build` job, modify the "Build" step to set env vars:

```yaml
      - name: Build
        env:
          BUILD_COMMIT: ${{ github.sha }}
          BUILD_DATE: ${{ github.event.head_commit.timestamp }}
        run: |
          # Extract short commit (7 chars)
          export BUILD_COMMIT="${BUILD_COMMIT:0:7}"
          # Extract date from timestamp (YYYY-MM-DD)
          export BUILD_DATE="${BUILD_DATE:0:10}"
          cargo build --release --target ${{ matrix.target }}
```

**Step 2: Verify YAML syntax**

Run: `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"`
Expected: No errors

**Step 3: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: pass build commit and date to release builds

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 4: Create Updater Module - Types and GitHub API

**Files:**
- Create: `src/updater.rs`
- Modify: `src/main.rs` (add module)

**Step 1: Create src/updater.rs with types and API functions**

```rust
//! Auto-update functionality for Quest.
//!
//! Checks GitHub releases for updates and handles download/installation.

use serde::Deserialize;
use std::error::Error;

/// Repository owner and name for GitHub API
const GITHUB_OWNER: &str = "stphung";
const GITHUB_REPO: &str = "quest";

/// Information about a GitHub release
#[derive(Debug, Clone)]
pub struct ReleaseInfo {
    pub commit: String,
    pub date: String,
    pub download_url: Option<String>,
}

/// A commit in the changelog
#[derive(Debug, Clone)]
pub struct ChangelogEntry {
    pub message: String,
}

/// Result of checking for updates
#[derive(Debug)]
pub enum UpdateCheck {
    /// No update available, already on latest
    UpToDate,
    /// Update available with release info and changelog
    UpdateAvailable {
        current_commit: String,
        current_date: String,
        latest: ReleaseInfo,
        changelog: Vec<ChangelogEntry>,
    },
    /// Failed to check (network error, etc.)
    CheckFailed(String),
}

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    published_at: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Deserialize)]
struct GitHubCompare {
    commits: Vec<GitHubCommit>,
}

#[derive(Deserialize)]
struct GitHubCommit {
    commit: GitHubCommitDetail,
}

#[derive(Deserialize)]
struct GitHubCommitDetail {
    message: String,
}

/// Parse release tag to extract commit hash.
/// Format: "build-4993005923a4924e5c338655f872ea9ebc9efe10" -> "4993005"
fn parse_release_tag(tag: &str) -> Option<String> {
    tag.strip_prefix("build-").map(|s| s.chars().take(7).collect())
}

/// Parse published_at timestamp to extract date.
/// Format: "2026-02-02T18:30:13Z" -> "2026-02-02"
fn parse_release_date(timestamp: &str) -> String {
    timestamp.chars().take(10).collect()
}

/// Get the asset name for the current platform.
fn get_platform_asset_name() -> &'static str {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    { "quest-x86_64-unknown-linux-gnu.tar.gz" }

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    { "quest-x86_64-apple-darwin.tar.gz" }

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    { "quest-aarch64-apple-darwin.tar.gz" }

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    { "quest-x86_64-pc-windows-msvc.zip" }

    #[cfg(not(any(
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64"),
    )))]
    { "unsupported-platform" }
}

/// Fetch the latest release from GitHub.
fn fetch_latest_release() -> Result<GitHubRelease, Box<dyn Error>> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        GITHUB_OWNER, GITHUB_REPO
    );

    let response: GitHubRelease = ureq::get(&url)
        .set("User-Agent", "quest-updater")
        .call()?
        .into_json()?;

    Ok(response)
}

/// Fetch changelog between two commits.
fn fetch_changelog(from: &str, to: &str) -> Result<Vec<ChangelogEntry>, Box<dyn Error>> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/compare/{}...{}",
        GITHUB_OWNER, GITHUB_REPO, from, to
    );

    let response: GitHubCompare = ureq::get(&url)
        .set("User-Agent", "quest-updater")
        .call()?
        .into_json()?;

    let entries: Vec<ChangelogEntry> = response
        .commits
        .into_iter()
        .map(|c| {
            // Take first line of commit message
            let message = c.commit.message.lines().next().unwrap_or("").to_string();
            ChangelogEntry { message }
        })
        .collect();

    Ok(entries)
}

/// Check for updates against the current build.
pub fn check_for_updates(current_commit: &str, current_date: &str) -> UpdateCheck {
    // Fetch latest release
    let release = match fetch_latest_release() {
        Ok(r) => r,
        Err(e) => return UpdateCheck::CheckFailed(e.to_string()),
    };

    // Parse release info
    let latest_commit = match parse_release_tag(&release.tag_name) {
        Some(c) => c,
        None => return UpdateCheck::CheckFailed("Invalid release tag format".to_string()),
    };

    // Check if we're already up to date
    if latest_commit == current_commit {
        return UpdateCheck::UpToDate;
    }

    let latest_date = parse_release_date(&release.published_at);

    // Find download URL for current platform
    let asset_name = get_platform_asset_name();
    let download_url = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .map(|a| a.browser_download_url.clone());

    // Fetch changelog
    let changelog = fetch_changelog(current_commit, &latest_commit).unwrap_or_default();

    UpdateCheck::UpdateAvailable {
        current_commit: current_commit.to_string(),
        current_date: current_date.to_string(),
        latest: ReleaseInfo {
            commit: latest_commit,
            date: latest_date,
            download_url,
        },
        changelog,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_release_tag() {
        assert_eq!(
            parse_release_tag("build-4993005923a4924e5c338655f872ea9ebc9efe10"),
            Some("4993005".to_string())
        );
        assert_eq!(parse_release_tag("v1.0.0"), None);
        assert_eq!(parse_release_tag("build-abc"), Some("abc".to_string()));
    }

    #[test]
    fn test_parse_release_date() {
        assert_eq!(parse_release_date("2026-02-02T18:30:13Z"), "2026-02-02");
        assert_eq!(parse_release_date("2025-12-31T00:00:00Z"), "2025-12-31");
    }

    #[test]
    fn test_get_platform_asset_name() {
        let name = get_platform_asset_name();
        assert!(name.starts_with("quest-"));
        assert!(name.ends_with(".tar.gz") || name.ends_with(".zip"));
    }
}
```

**Step 2: Add module to main.rs**

Add after `mod build_info;`:

```rust
mod updater;
```

**Step 3: Verify compilation**

Run: `cargo build`
Expected: Compiles successfully

**Step 4: Run tests**

Run: `cargo test updater`
Expected: All tests pass

**Step 5: Commit**

```bash
git add src/updater.rs src/main.rs
git commit -m "feat: add updater module with GitHub API integration

- Types for release info and changelog
- Functions to fetch latest release and compare commits
- Platform detection for correct asset download
- Tests for tag/date parsing

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 5: Add Backup and Download Functions

**Files:**
- Modify: `src/updater.rs`

**Step 1: Add backup and download functions to updater.rs**

Add these imports at the top:

```rust
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
```

Add these functions after `check_for_updates`:

```rust
/// Get the Quest data directory (~/.quest)
fn get_quest_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".quest"))
}

/// Backup all character saves to a timestamped directory.
/// Returns the backup path on success, or None if no saves exist.
pub fn backup_saves() -> Result<Option<PathBuf>, Box<dyn Error>> {
    let quest_dir = get_quest_dir().ok_or("Could not find home directory")?;

    if !quest_dir.exists() {
        return Ok(None); // No saves to backup
    }

    // Find all JSON files (character saves)
    let saves: Vec<_> = fs::read_dir(&quest_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
        .collect();

    if saves.is_empty() {
        return Ok(None); // No saves to backup
    }

    // Create backup directory with timestamp
    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H%M%S");
    let backup_dir = quest_dir.join("backups").join(timestamp.to_string());
    fs::create_dir_all(&backup_dir)?;

    // Copy each save file
    for entry in saves {
        let src = entry.path();
        let dst = backup_dir.join(entry.file_name());
        fs::copy(&src, &dst)?;
    }

    Ok(Some(backup_dir))
}

/// Download a file from URL to the specified path, showing progress.
pub fn download_file(url: &str, dest: &Path, progress_callback: impl Fn(u64, u64)) -> Result<(), Box<dyn Error>> {
    let response = ureq::get(url)
        .set("User-Agent", "quest-updater")
        .call()?;

    let total_size: u64 = response
        .header("Content-Length")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let mut reader = response.into_reader();
    let mut file = File::create(dest)?;
    let mut downloaded: u64 = 0;
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        file.write_all(&buffer[..bytes_read])?;
        downloaded += bytes_read as u64;
        progress_callback(downloaded, total_size);
    }

    Ok(())
}

/// Extract a tar.gz archive to a directory.
#[cfg(not(target_os = "windows"))]
pub fn extract_archive(archive_path: &Path, dest_dir: &Path) -> Result<PathBuf, Box<dyn Error>> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    let file = File::open(archive_path)?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);

    archive.unpack(dest_dir)?;

    // Return path to extracted binary
    Ok(dest_dir.join("quest"))
}

/// Extract a zip archive to a directory.
#[cfg(target_os = "windows")]
pub fn extract_archive(archive_path: &Path, dest_dir: &Path) -> Result<PathBuf, Box<dyn Error>> {
    use std::io::BufReader;
    use zip::ZipArchive;

    let file = File::open(archive_path)?;
    let reader = BufReader::new(file);
    let mut archive = ZipArchive::new(reader)?;

    archive.extract(dest_dir)?;

    // Return path to extracted binary
    Ok(dest_dir.join("quest.exe"))
}

/// Replace the current binary with a new one.
pub fn replace_binary(new_binary: &Path) -> Result<(), Box<dyn Error>> {
    let current_exe = std::env::current_exe()?;

    #[cfg(not(target_os = "windows"))]
    {
        // On Unix, we can just overwrite the file
        fs::copy(new_binary, &current_exe)?;

        // Make executable
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&current_exe)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&current_exe, perms)?;
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, rename current to .old, then move new into place
        let old_exe = current_exe.with_extension("exe.old");
        fs::rename(&current_exe, &old_exe)?;
        fs::rename(new_binary, &current_exe)?;
        // Try to delete old, ignore errors (may be locked)
        let _ = fs::remove_file(&old_exe);
    }

    Ok(())
}
```

**Step 2: Add chrono to regular dependencies for Local time**

Verify chrono is already in dependencies (it is).

**Step 3: Verify compilation**

Run: `cargo build`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add src/updater.rs
git commit -m "feat: add backup, download, and binary replacement functions

- backup_saves() copies character JSON files to timestamped folder
- download_file() fetches URL with progress callback
- extract_archive() handles tar.gz (Unix) and zip (Windows)
- replace_binary() swaps current executable with new one

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 6: Add Update Command Entry Point

**Files:**
- Modify: `src/updater.rs`

**Step 1: Add run_update_command function**

Add at the end of updater.rs, before the tests module:

```rust
/// Run the update command (quest update).
/// Returns Ok(true) if updated, Ok(false) if already up to date.
pub fn run_update_command() -> Result<bool, Box<dyn Error>> {
    use crate::build_info::{BUILD_COMMIT, BUILD_DATE};

    println!("Checking for updates...\n");

    let check = check_for_updates(BUILD_COMMIT, BUILD_DATE);

    match check {
        UpdateCheck::UpToDate => {
            println!("You're up to date.");
            println!("  Current: {} ({})", BUILD_DATE, BUILD_COMMIT);
            Ok(false)
        }
        UpdateCheck::CheckFailed(err) => {
            eprintln!("Failed to check for updates: {}", err);
            Err(err.into())
        }
        UpdateCheck::UpdateAvailable {
            current_commit,
            current_date,
            latest,
            changelog,
        } => {
            println!("Update available!");
            println!("  Your build:  {} ({})", current_date, current_commit);
            println!("  Latest:      {} ({})", latest.date, latest.commit);
            println!();

            // Show changelog (max 10 entries)
            if !changelog.is_empty() {
                println!("What's new:");
                for entry in changelog.iter().take(10) {
                    // Truncate long messages
                    let msg = if entry.message.len() > 50 {
                        format!("{}...", &entry.message[..47])
                    } else {
                        entry.message.clone()
                    };
                    println!("  • {}", msg);
                }
                if changelog.len() > 10 {
                    println!("  ...and {} more", changelog.len() - 10);
                }
                println!();
            }

            // Check for download URL
            let download_url = match &latest.download_url {
                Some(url) => url,
                None => {
                    eprintln!("No download available for your platform.");
                    return Err("Unsupported platform".into());
                }
            };

            // Backup saves
            print!("Backing up saves... ");
            io::stdout().flush()?;
            match backup_saves() {
                Ok(Some(path)) => println!("done ({})", path.display()),
                Ok(None) => println!("skipped (no saves)"),
                Err(e) => {
                    println!("failed");
                    eprintln!("Backup failed: {}", e);
                    return Err(e);
                }
            }

            // Download update
            let temp_dir = std::env::temp_dir().join("quest-update");
            fs::create_dir_all(&temp_dir)?;
            let archive_path = temp_dir.join("update.archive");

            print!("Downloading update... ");
            io::stdout().flush()?;
            download_file(download_url, &archive_path, |downloaded, total| {
                if total > 0 {
                    let percent = (downloaded * 100) / total;
                    print!("\rDownloading update... {}%", percent);
                    let _ = io::stdout().flush();
                }
            })?;
            println!("\rDownloading update... done");

            // Extract archive
            print!("Installing... ");
            io::stdout().flush()?;
            let new_binary = extract_archive(&archive_path, &temp_dir)?;

            // Replace binary
            replace_binary(&new_binary)?;
            println!("done");

            // Cleanup
            let _ = fs::remove_dir_all(&temp_dir);

            println!();
            println!("Updated successfully! Run 'quest' to play.");

            Ok(true)
        }
    }
}
```

**Step 2: Verify compilation**

Run: `cargo build`
Expected: Compiles successfully

**Step 3: Commit**

```bash
git add src/updater.rs
git commit -m "feat: add run_update_command entry point

Orchestrates the full update flow:
- Check for updates via GitHub API
- Display changelog
- Backup saves
- Download with progress
- Extract and replace binary

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 7: Add CLI Argument Handling

**Files:**
- Modify: `src/main.rs`

**Step 1: Add CLI handling at start of main()**

Replace the beginning of `fn main()` (before `let character_manager = ...`) with:

```rust
fn main() -> io::Result<()> {
    // Handle CLI arguments
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "update" => {
                match updater::run_update_command() {
                    Ok(_) => std::process::exit(0),
                    Err(_) => std::process::exit(1),
                }
            }
            "--version" | "-v" => {
                println!(
                    "quest {} ({})",
                    build_info::BUILD_DATE,
                    build_info::BUILD_COMMIT
                );
                std::process::exit(0);
            }
            "--help" | "-h" => {
                println!("Quest - Terminal-Based Idle RPG\n");
                println!("Usage: quest [command]\n");
                println!("Commands:");
                println!("  update     Check for and install updates");
                println!("  --version  Show version information");
                println!("  --help     Show this help message");
                std::process::exit(0);
            }
            other => {
                eprintln!("Unknown command: {}", other);
                eprintln!("Run 'quest --help' for usage.");
                std::process::exit(1);
            }
        }
    }

    // Initialize CharacterManager
    let character_manager = CharacterManager::new()?;
```

**Step 2: Verify compilation**

Run: `cargo build`
Expected: Compiles successfully

**Step 3: Test CLI commands**

Run: `cargo run -- --version`
Expected: Shows date and commit hash

Run: `cargo run -- --help`
Expected: Shows help message

Run: `cargo run -- update`
Expected: Either shows "up to date" or "update available" (depends on releases)

**Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: add CLI argument handling for update command

- quest update: check and install updates
- quest --version: show build info
- quest --help: show usage

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 8: Add Startup Update Notification

**Files:**
- Modify: `src/main.rs`
- Modify: `src/updater.rs`

**Step 1: Add quick check function to updater.rs**

Add this function before `run_update_command`:

```rust
/// Quick check for update availability (for startup notification).
/// Returns Some((date, commit)) if update available, None otherwise.
pub fn quick_update_check() -> Option<(String, String)> {
    use crate::build_info::{BUILD_COMMIT, BUILD_DATE};

    match check_for_updates(BUILD_COMMIT, BUILD_DATE) {
        UpdateCheck::UpdateAvailable { latest, .. } => {
            Some((latest.date, latest.commit))
        }
        _ => None,
    }
}
```

**Step 2: Add notification display in main.rs**

After the CLI argument handling block (after the closing brace of the `if args.len() > 1` block), add:

```rust
    // Check for updates in background (non-blocking notification)
    let update_available = std::thread::spawn(|| updater::quick_update_check());
```

Then, after terminal setup (after `let mut terminal = Terminal::new(backend)?;`), add:

```rust
    // Show update notification if available
    if let Ok(Some((date, commit))) = update_available.join() {
        // Draw notification
        terminal.draw(|frame| {
            let area = frame.area();
            let block = ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .border_style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow))
                .title(" Update Available ");

            let inner = block.inner(area);
            frame.render_widget(block, area);

            let text = vec![
                ratatui::text::Line::from(""),
                ratatui::text::Line::from(format!("  New version: {} ({})", date, commit)),
                ratatui::text::Line::from(""),
                ratatui::text::Line::from("  Run 'quest update' to install."),
                ratatui::text::Line::from(""),
                ratatui::text::Line::from("  Press any key to continue..."),
            ];

            let paragraph = ratatui::widgets::Paragraph::new(text)
                .alignment(ratatui::layout::Alignment::Left);

            frame.render_widget(paragraph, inner);
        })?;

        // Wait for keypress (max 5 seconds)
        let _ = event::poll(Duration::from_secs(5));
        if event::poll(Duration::from_millis(0))? {
            let _ = event::read()?;
        }
    }
```

**Step 3: Verify compilation**

Run: `cargo build`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add src/main.rs src/updater.rs
git commit -m "feat: show update notification on game startup

- Background thread checks for updates while game loads
- Shows notification box if update available
- Auto-dismisses after 5 seconds or on keypress

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Task 9: Run Full Test Suite and Cleanup

**Files:**
- All modified files

**Step 1: Run formatter**

Run: `cargo fmt`
Expected: Formats all code

**Step 2: Run clippy**

Run: `cargo clippy --all-targets -- -D warnings`
Expected: No warnings

**Step 3: Run all tests**

Run: `cargo test`
Expected: All tests pass

**Step 4: Test the full flow manually**

Run: `cargo run -- --version`
Expected: Shows version

Run: `cargo run -- update`
Expected: Shows update status

Run: `cargo run`
Expected: Game starts (may show update notification)

**Step 5: Final commit if any fixes needed**

```bash
git add -A
git commit -m "chore: fix formatting and clippy warnings

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>"
```

---

## Summary

**New files created:**
- `build.rs` — Compile-time build info generation
- `src/build_info.rs` — Build info constants module
- `src/updater.rs` — Update check, download, and installation

**Files modified:**
- `Cargo.toml` — Added ureq, flate2, tar, zip dependencies
- `.github/workflows/ci.yml` — Pass BUILD_COMMIT and BUILD_DATE env vars
- `src/main.rs` — CLI argument handling and startup notification

**Commands added:**
- `quest update` — Check and install updates
- `quest --version` — Show build info
- `quest --help` — Show usage
