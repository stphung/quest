//! Auto-update functionality for Quest.
//!
//! Checks GitHub releases for updates and handles download/installation.

use serde::Deserialize;
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

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

/// Parse release tag to extract full commit hash.
/// Format: "build-4993005923a4924e5c338655f872ea9ebc9efe10" -> full hash
fn parse_release_tag(tag: &str) -> Option<String> {
    tag.strip_prefix("build-").map(|s| s.to_string())
}

/// Shorten a commit hash for display (7 chars).
fn short_commit(hash: &str) -> String {
    hash.chars().take(7).collect()
}

/// Parse published_at timestamp to extract date.
/// Format: "2026-02-02T18:30:13Z" -> "2026-02-02"
fn parse_release_date(timestamp: &str) -> String {
    timestamp.chars().take(10).collect()
}

/// Get the asset name for the current platform.
fn get_platform_asset_name() -> &'static str {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        "quest-x86_64-unknown-linux-gnu.tar.gz"
    }

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        "quest-x86_64-apple-darwin.tar.gz"
    }

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "quest-aarch64-apple-darwin.tar.gz"
    }

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        "quest-x86_64-pc-windows-msvc.zip"
    }

    #[cfg(not(any(
        all(target_os = "linux", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "x86_64"),
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "windows", target_arch = "x86_64"),
    )))]
    {
        "unsupported-platform"
    }
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
        .rev() // Reverse to show newest commits first
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
    let latest_commit_full = match parse_release_tag(&release.tag_name) {
        Some(c) => c,
        None => return UpdateCheck::CheckFailed("Invalid release tag format".to_string()),
    };

    // Compare short commits (7 chars) to handle both local builds (7 char)
    // and release builds (full hash in tag, 7 char in binary)
    let latest_commit_short = short_commit(&latest_commit_full);
    let current_commit_short = short_commit(current_commit);

    // Check if we're already up to date (same commit)
    if latest_commit_short == current_commit_short {
        return UpdateCheck::UpToDate;
    }

    let latest_date = parse_release_date(&release.published_at);

    // Only skip update if we're on a NEWER commit (different commit + newer date)
    // This prevents "updating" to an older release when on a dev build
    // But allows updating when commits differ, even if dates are the same
    if current_commit_short != latest_commit_short && current_date > latest_date.as_str() {
        return UpdateCheck::UpToDate;
    }

    // Find download URL for current platform
    let asset_name = get_platform_asset_name();
    let download_url = release
        .assets
        .iter()
        .find(|a| a.name == asset_name)
        .map(|a| a.browser_download_url.clone());

    // Fetch changelog using full commit hashes for GitHub API
    let changelog = fetch_changelog(current_commit, &latest_commit_full).unwrap_or_default();

    UpdateCheck::UpdateAvailable {
        current_commit: current_commit.to_string(),
        current_date: current_date.to_string(),
        latest: ReleaseInfo {
            commit: latest_commit_full,
            date: latest_date,
            download_url,
        },
        changelog,
    }
}

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
pub fn download_file(
    url: &str,
    dest: &Path,
    progress_callback: impl Fn(u64, u64),
) -> Result<(), Box<dyn Error>> {
    let response = ureq::get(url).set("User-Agent", "quest-updater").call()?;

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

/// Full update information for in-game display
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub new_version: String,
    pub new_commit: String,
    pub changelog: Vec<String>,
    pub changelog_total: usize,
}

/// Check for updates and return full info including changelog.
/// Returns Some(UpdateInfo) if update available, None otherwise.
pub fn check_update_info() -> Option<UpdateInfo> {
    use crate::utils::build_info::{BUILD_COMMIT, BUILD_DATE};

    match check_for_updates(BUILD_COMMIT, BUILD_DATE) {
        UpdateCheck::UpdateAvailable {
            latest, changelog, ..
        } => {
            let total = changelog.len();
            Some(UpdateInfo {
                new_version: latest.date,
                new_commit: short_commit(&latest.commit),
                changelog: changelog
                    .into_iter()
                    .take(5) // Limit to 5 entries for in-game display
                    .map(|e| {
                        // Truncate long messages and take first line only
                        let msg = e.message.lines().next().unwrap_or(&e.message);
                        if msg.len() > 45 {
                            format!("{}...", &msg[..42])
                        } else {
                            msg.to_string()
                        }
                    })
                    .collect(),
                changelog_total: total,
            })
        }
        _ => None,
    }
}

/// Run the update command (quest update).
/// Returns Ok(true) if updated, Ok(false) if already up to date.
pub fn run_update_command() -> Result<bool, Box<dyn Error>> {
    use crate::utils::build_info::{BUILD_COMMIT, BUILD_DATE};

    println!("Checking for updates...\n");

    let check = check_for_updates(BUILD_COMMIT, BUILD_DATE);

    match check {
        UpdateCheck::UpToDate => {
            println!("No updates available.\n");
            println!("  Current version: {} ({})", BUILD_DATE, BUILD_COMMIT);
            println!("\nYou're running the latest version!");
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
            println!(
                "  Latest:      {} ({})",
                latest.date,
                short_commit(&latest.commit)
            );
            println!();

            // Show changelog (max 15 entries)
            if !changelog.is_empty() {
                println!("What's new:");
                for entry in changelog.iter().take(15) {
                    // Truncate long messages
                    let msg = if entry.message.len() > 60 {
                        format!("{}...", &entry.message[..57])
                    } else {
                        entry.message.clone()
                    };
                    println!("  • {}", msg);
                }
                if changelog.len() > 15 {
                    println!("  ...and {} more", changelog.len() - 15);
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

            // Ask user to confirm
            print!("Install update? [Y/n] ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim().to_lowercase();
            if input == "n" || input == "no" {
                println!("Update cancelled.");
                return Ok(false);
            }

            // Get current executable path for display
            let current_exe = std::env::current_exe()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "quest".to_string());

            // Set up paths
            let temp_dir = std::env::temp_dir().join("quest-update");
            let archive_path = temp_dir.join("update.archive");

            // Show the plan
            println!("Update plan:");
            println!("  1. Backup saves to ~/.quest/backups/");
            println!(
                "  2. Download {} to {}",
                download_url,
                archive_path.display()
            );
            println!("  3. Extract archive to {}", temp_dir.display());
            println!("  4. Replace {}", current_exe);
            println!();

            // Step 1: Backup saves
            println!("[1/4] Backing up saves...");
            match backup_saves() {
                Ok(Some(path)) => {
                    println!("       ✓ Saved to: {}", path.display());
                }
                Ok(None) => {
                    println!("       ✓ Skipped (no saves found)");
                }
                Err(e) => {
                    println!("       ✗ Failed: {}", e);
                    return Err(e);
                }
            }

            // Step 2: Download update
            println!("[2/4] Downloading update...");
            fs::create_dir_all(&temp_dir)?;
            print!("       ");
            io::stdout().flush()?;
            download_file(download_url, &archive_path, |downloaded, total| {
                if total > 0 {
                    let percent = (downloaded * 100) / total;
                    print!("\r       {}%", percent);
                    let _ = io::stdout().flush();
                }
            })?;
            println!("\r       ✓ Downloaded");

            // Step 3: Extract archive
            println!("[3/4] Extracting archive...");
            let new_binary = extract_archive(&archive_path, &temp_dir)?;
            println!("       ✓ Extracted to: {}", new_binary.display());

            // Step 4: Replace binary
            println!("[4/4] Replacing binary...");
            replace_binary(&new_binary)?;
            println!("       ✓ Replaced: {}", current_exe);

            // Cleanup
            println!();
            println!("Cleaning up temporary files...");
            let _ = fs::remove_dir_all(&temp_dir);
            println!("       ✓ Done");

            println!();
            println!("✓ Update complete! Run 'quest' to play.");

            Ok(true)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_release_tag() {
        assert_eq!(
            parse_release_tag("build-4993005923a4924e5c338655f872ea9ebc9efe10"),
            Some("4993005923a4924e5c338655f872ea9ebc9efe10".to_string())
        );
        assert_eq!(parse_release_tag("v1.0.0"), None);
        assert_eq!(parse_release_tag("build-abc"), Some("abc".to_string()));
    }

    #[test]
    fn test_short_commit() {
        assert_eq!(
            short_commit("4993005923a4924e5c338655f872ea9ebc9efe10"),
            "4993005"
        );
        assert_eq!(short_commit("abc"), "abc");
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
