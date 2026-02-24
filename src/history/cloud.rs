//! GitHub cloud sync for Time Vault saves.
//!
//! Provides push/pull operations to back up save history to a private GitHub
//! repository. Uses Personal Access Tokens for authentication and the GitHub
//! REST API (via `ureq`) for repo management.

use git2::{BranchType, FetchOptions, PushOptions, RemoteCallbacks, Repository};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Name of the git remote used for cloud sync.
const REMOTE_NAME: &str = "cloud";

/// Default repository name created on the user's GitHub account.
const DEFAULT_REPO_NAME: &str = "quest-saves";

/// User-Agent header for GitHub API requests.
const USER_AGENT: &str = "quest-cloud-sync";

// ── CloudConfig ──────────────────────────────────────────────────────────

/// Persisted cloud configuration stored in `~/.quest/.cloud.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudConfig {
    /// GitHub Personal Access Token.
    pub token: String,
    /// GitHub username (fetched from API).
    pub username: String,
    /// Full HTTPS clone URL of the remote repo.
    pub repo_url: String,
}

/// Current state of cloud sync (not persisted).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloudStatus {
    /// No GitHub account linked.
    Unlinked,
    /// Linked and idle.
    Linked,
    /// Operation in progress (push, pull, link).
    Syncing,
    /// Last operation failed.
    Error(String),
}

/// Result of a completed cloud operation.
#[derive(Debug)]
#[allow(dead_code)]
pub enum CloudOpResult {
    /// Link succeeded — returns the new config.
    Linked(CloudConfig),
    /// Push succeeded.
    Pushed,
    /// Pull succeeded — caller should reload state from disk.
    Pulled,
    /// Unlink succeeded.
    Unlinked,
    /// Operation failed.
    Failed(String),
}

// ── Config persistence ───────────────────────────────────────────────────

fn config_path(quest_dir: &Path) -> PathBuf {
    quest_dir.join(".cloud.json")
}

/// Load cloud config from disk. Returns `None` if not linked.
pub fn load_config(quest_dir: &Path) -> Option<CloudConfig> {
    let path = config_path(quest_dir);
    let data = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

/// Save cloud config to disk.
fn save_config(quest_dir: &Path, config: &CloudConfig) -> Result<(), String> {
    let path = config_path(quest_dir);
    let data = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(path, data).map_err(|e| format!("failed to save cloud config: {e}"))
}

/// Delete cloud config from disk.
fn delete_config(quest_dir: &Path) {
    let _ = std::fs::remove_file(config_path(quest_dir));
}

// ── GitHub API helpers ───────────────────────────────────────────────────

#[derive(Deserialize)]
struct GitHubUser {
    login: String,
}

#[derive(Deserialize)]
struct GitHubRepo {
    clone_url: String,
}

/// Validate a PAT and return the authenticated username.
fn github_get_username(token: &str) -> Result<String, String> {
    let user: GitHubUser = ureq::get("https://api.github.com/user")
        .header("Authorization", &format!("Bearer {token}"))
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| format!("GitHub API error: {e}"))?
        .into_body()
        .read_json()
        .map_err(|e| format!("failed to parse GitHub response: {e}"))?;
    Ok(user.login)
}

/// Create a private repo (or return the existing one).
fn github_ensure_repo(token: &str, repo_name: &str) -> Result<String, String> {
    // Try to get existing repo first.
    let url = "https://api.github.com/user/repos?per_page=100&type=owner";
    let repos: Vec<GitHubRepo> = ureq::get(url)
        .header("Authorization", &format!("Bearer {token}"))
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| format!("GitHub API error: {e}"))?
        .into_body()
        .read_json()
        .map_err(|e| format!("failed to parse repos list: {e}"))?;

    // Check if repo already exists by clone_url containing the repo name.
    for repo in &repos {
        if repo.clone_url.ends_with(&format!("/{repo_name}.git"))
            || repo.clone_url.ends_with(&format!("/{repo_name}"))
        {
            return Ok(repo.clone_url.clone());
        }
    }

    // Create new private repo.
    let body = serde_json::json!({
        "name": repo_name,
        "private": true,
        "description": "Quest save data (auto-managed by Quest game)",
        "auto_init": false,
    });

    let body_str = body.to_string();
    let created: GitHubRepo = ureq::post("https://api.github.com/user/repos")
        .header("Authorization", &format!("Bearer {token}"))
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/vnd.github+json")
        .content_type("application/json")
        .send(body_str.as_bytes())
        .map_err(|e| format!("failed to create repo: {e}"))?
        .into_body()
        .read_json()
        .map_err(|e| format!("failed to parse create-repo response: {e}"))?;

    Ok(created.clone_url)
}

/// Build an authenticated HTTPS URL from a clone URL and token.
///
/// Transforms `https://github.com/user/repo.git` into
/// `https://x-access-token:TOKEN@github.com/user/repo.git`.
fn authenticated_url(clone_url: &str, token: &str) -> String {
    if let Some(rest) = clone_url.strip_prefix("https://") {
        format!("https://x-access-token:{token}@{rest}")
    } else {
        clone_url.to_string()
    }
}

// ── git2 credential callback ─────────────────────────────────────────────

fn make_callbacks(token: String) -> RemoteCallbacks<'static> {
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
        git2::Cred::userpass_plaintext("x-access-token", &token)
    });
    callbacks
}

// ── Public operations ────────────────────────────────────────────────────

/// Link a GitHub account: validate token, create/find repo, add remote, push.
///
/// This is a blocking operation (network I/O). Call from a background thread.
pub fn link_github(quest_dir: &Path, token: &str) -> Result<CloudConfig, String> {
    // 1. Validate token and get username.
    let username = github_get_username(token)?;

    // 2. Create or find the saves repo.
    let clone_url = github_ensure_repo(token, DEFAULT_REPO_NAME)?;

    // 3. Open local repo and add remote.
    let repo = Repository::open(quest_dir).map_err(|e| format!("failed to open repo: {e}"))?;

    // Remove existing cloud remote if present (re-linking).
    if repo.find_remote(REMOTE_NAME).is_ok() {
        repo.remote_delete(REMOTE_NAME)
            .map_err(|e| format!("failed to remove old remote: {e}"))?;
    }

    let auth_url = authenticated_url(&clone_url, token);
    repo.remote(REMOTE_NAME, &auth_url)
        .map_err(|e| format!("failed to add remote: {e}"))?;

    // 4. Push all local branches.
    push_all_branches(&repo, token)?;

    // 5. Save config.
    let config = CloudConfig {
        token: token.to_string(),
        username,
        repo_url: clone_url,
    };
    save_config(quest_dir, &config)?;

    Ok(config)
}

/// Push the active branch to the cloud remote.
///
/// Blocking operation — call from a background thread.
#[allow(dead_code)]
pub fn push_to_cloud(quest_dir: &Path, config: &CloudConfig) -> Result<(), String> {
    let repo = Repository::open(quest_dir).map_err(|e| format!("failed to open repo: {e}"))?;

    let head = repo
        .head()
        .map_err(|e| format!("failed to get HEAD: {e}"))?;
    let branch_name = head
        .shorthand()
        .ok_or_else(|| "HEAD is not on a branch".to_string())?
        .to_string();

    push_branch(&repo, &branch_name, &config.token)
}

/// Push all local branches to the cloud remote.
pub fn push_all(quest_dir: &Path, config: &CloudConfig) -> Result<(), String> {
    let repo = Repository::open(quest_dir).map_err(|e| format!("failed to open repo: {e}"))?;
    push_all_branches(&repo, &config.token)
}

/// Fetch from cloud and reset current branch to match remote.
///
/// Blocking operation — call from a background thread.
pub fn pull_from_cloud(quest_dir: &Path, config: &CloudConfig) -> Result<(), String> {
    let repo = Repository::open(quest_dir).map_err(|e| format!("failed to open repo: {e}"))?;

    // Fetch all branches from remote.
    let mut remote = repo
        .find_remote(REMOTE_NAME)
        .map_err(|e| format!("cloud remote not found: {e}"))?;

    let callbacks = make_callbacks(config.token.clone());
    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);

    remote
        .fetch(
            &["refs/heads/*:refs/remotes/cloud/*"],
            Some(&mut fetch_opts),
            None,
        )
        .map_err(|e| format!("fetch failed: {e}"))?;

    // Reset current branch to match remote tracking branch.
    let head = repo
        .head()
        .map_err(|e| format!("failed to get HEAD: {e}"))?;
    let branch_name = head
        .shorthand()
        .ok_or_else(|| "HEAD is not on a branch".to_string())?
        .to_string();

    let remote_ref = format!("refs/remotes/cloud/{branch_name}");
    if let Ok(reference) = repo.find_reference(&remote_ref) {
        let commit = reference
            .peel_to_commit()
            .map_err(|e| format!("failed to resolve remote ref: {e}"))?;
        repo.reset(commit.as_object(), git2::ResetType::Hard, None)
            .map_err(|e| format!("reset failed: {e}"))?;
    } else {
        return Err(format!(
            "branch '{}' not found on cloud remote",
            branch_name
        ));
    }

    Ok(())
}

/// Remove the cloud remote and delete the config file.
pub fn unlink(quest_dir: &Path) -> Result<(), String> {
    if let Ok(repo) = Repository::open(quest_dir) {
        let _ = repo.remote_delete(REMOTE_NAME);
    }
    delete_config(quest_dir);
    Ok(())
}

/// Check whether cloud is configured.
#[allow(dead_code)]
pub fn is_linked(quest_dir: &Path) -> bool {
    load_config(quest_dir).is_some()
}

// ── Internal helpers ─────────────────────────────────────────────────────

fn push_branch(repo: &Repository, branch_name: &str, token: &str) -> Result<(), String> {
    let mut remote = repo
        .find_remote(REMOTE_NAME)
        .map_err(|e| format!("cloud remote not found: {e}"))?;

    let callbacks = make_callbacks(token.to_string());
    let mut push_opts = PushOptions::new();
    push_opts.remote_callbacks(callbacks);

    let refspec = format!("refs/heads/{branch_name}:refs/heads/{branch_name}");
    remote
        .push(&[&refspec], Some(&mut push_opts))
        .map_err(|e| format!("push failed: {e}"))?;

    Ok(())
}

fn push_all_branches(repo: &Repository, token: &str) -> Result<(), String> {
    let branches: Vec<String> = repo
        .branches(Some(BranchType::Local))
        .map_err(|e| format!("failed to list branches: {e}"))?
        .filter_map(|b| {
            b.ok()
                .and_then(|(branch, _)| branch.name().ok().flatten().map(|n| n.to_string()))
        })
        .collect();

    for branch_name in &branches {
        push_branch(repo, branch_name, token)?;
    }
    Ok(())
}
