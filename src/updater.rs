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
