//! Cloud sync for Time Vault — push/pull save branches to a private GitHub repo.
//!
//! Uses `ureq` for GitHub API calls and `git2` for remote operations.
//! All functions return `Result<T, String>` for simplicity.

use std::path::Path;

use git2::{BranchType, Repository};
use serde::{Deserialize, Serialize};

use super::git::parse_commit_suffix;

// ── Constants ────────────────────────────────────────────────────────────

/// Name of the git remote added for cloud sync.
pub const REMOTE_NAME: &str = "cloud";

/// Default GitHub repository name created for save sync.
pub const DEFAULT_REPO_NAME: &str = "quest-saves";

/// User-Agent header sent with GitHub API requests.
const USER_AGENT: &str = "quest-cloud-sync";

/// GitHub topic used to tag quest save repositories.
const REPO_TOPIC: &str = "quest-time-vaults";

/// GitHub API base URL.
const GITHUB_API: &str = "https://api.github.com";

// ── Types ────────────────────────────────────────────────────────────────

/// Persisted cloud configuration (stored in `~/.quest/.cloud.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudConfig {
    /// GitHub personal access token.
    pub token: String,
    /// GitHub username (validated via API).
    pub username: String,
    /// Full clone URL of the remote repository.
    pub repo_url: String,
}

/// Current cloud sync status for UI display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloudStatus {
    /// No cloud config present.
    Offline,
    /// Linked to a remote but not currently syncing.
    Linked,
    /// A sync operation is in progress.
    Syncing,
    /// Local and remote have diverged on at least one branch.
    OutOfSync,
    /// An error occurred during the last operation.
    Error(String),
}

/// Result of a cloud operation, returned to the caller for UI feedback.
#[derive(Debug, Clone)]
pub enum CloudOpResult {
    /// Successfully linked to a remote. Contains the saved config.
    Linked(CloudConfig),
    /// Token validated; here are the username and existing repos for selection.
    TokenValidated {
        username: String,
        token: String,
        repos: Vec<String>,
    },
    /// All branches pushed to remote.
    Pushed,
    /// All branches pulled (fast-forwarded) from remote.
    Pulled,
    /// Cloud remote and config removed.
    Unlinked,
    /// At least one branch has diverged (ahead AND behind).
    Diverged(BranchDivergence),
    /// Operation failed with an error message.
    Failed(String),
}

/// Describes a single branch that has diverged between local and remote.
#[derive(Debug, Clone)]
pub struct BranchDivergence {
    /// Name of the diverged branch.
    pub branch_name: String,
    /// Local head metadata.
    pub local_level: u32,
    pub local_prestige: u32,
    pub local_playtime: u64,
    /// Remote head metadata.
    pub remote_level: u32,
    pub remote_prestige: u32,
    pub remote_playtime: u64,
}

// ── Config persistence ───────────────────────────────────────────────────

/// Load cloud config from `~/.quest/.cloud.json`.
///
/// Returns `None` if the file does not exist or cannot be parsed.
pub fn load_config(quest_dir: &Path) -> Option<CloudConfig> {
    let path = quest_dir.join(".cloud.json");
    let data = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&data).ok()
}

/// Save cloud config to `~/.quest/.cloud.json`.
pub fn save_config(quest_dir: &Path, config: &CloudConfig) -> Result<(), String> {
    let path = quest_dir.join(".cloud.json");
    let data = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(path, data).map_err(|e| e.to_string())
}

/// Delete cloud config from `~/.quest/.cloud.json`.
pub fn delete_config(quest_dir: &Path) -> Result<(), String> {
    let path = quest_dir.join(".cloud.json");
    if path.exists() {
        std::fs::remove_file(path).map_err(|e| e.to_string())
    } else {
        Ok(())
    }
}

// ── GitHub API helpers ───────────────────────────────────────────────────

/// Validate a GitHub personal access token and return the username.
///
/// Calls `GET /user` to verify the token is valid.
pub fn github_get_username(token: &str) -> Result<String, String> {
    #[derive(Deserialize)]
    struct GithubUser {
        login: String,
    }

    let url = format!("{GITHUB_API}/user");
    let resp: GithubUser = ureq::get(&url)
        .header("User-Agent", USER_AGENT)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| format!("GitHub API error: {e}"))?
        .into_body()
        .read_json()
        .map_err(|e| format!("Failed to parse GitHub response: {e}"))?;

    Ok(resp.login)
}

/// List the authenticated user's repositories that may contain quest saves.
///
/// First tries the GitHub search API filtered by the `quest-time-vaults` topic.
/// If no tagged repos are found, falls back to listing all user repos so that
/// repos created before topic tagging was added still appear.
pub fn github_list_repos(token: &str) -> Result<Vec<String>, String> {
    let username = github_get_username(token)?;

    // Try topic-filtered search first.
    let tagged = github_list_repos_by_topic(token, &username)?;
    if !tagged.is_empty() {
        return Ok(tagged);
    }

    // Fallback: list all user repos (handles pre-topic repos).
    github_list_all_user_repos(token)
}

/// Search for repos tagged with the quest topic.
fn github_list_repos_by_topic(token: &str, username: &str) -> Result<Vec<String>, String> {
    #[derive(Deserialize)]
    struct SearchResult {
        items: Vec<SearchItem>,
    }
    #[derive(Deserialize)]
    struct SearchItem {
        name: String,
    }

    let query = format!("topic:{REPO_TOPIC} user:{username}");
    let url = format!(
        "{GITHUB_API}/search/repositories?q={}&per_page=100",
        urlencoded(&query)
    );
    let result: SearchResult = ureq::get(&url)
        .header("User-Agent", USER_AGENT)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| format!("GitHub API error: {e}"))?
        .into_body()
        .read_json()
        .map_err(|e| format!("Failed to parse search response: {e}"))?;

    Ok(result.items.into_iter().map(|r| r.name).collect())
}

/// List all repositories owned by the authenticated user.
fn github_list_all_user_repos(token: &str) -> Result<Vec<String>, String> {
    #[derive(Deserialize)]
    struct RepoItem {
        name: String,
    }

    let url = format!("{GITHUB_API}/user/repos?per_page=100&sort=updated&affiliation=owner");
    let repos: Vec<RepoItem> = ureq::get(&url)
        .header("User-Agent", USER_AGENT)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| format!("GitHub API error: {e}"))?
        .into_body()
        .read_json()
        .map_err(|e| format!("Failed to parse repos response: {e}"))?;

    Ok(repos.into_iter().map(|r| r.name).collect())
}

/// Extract the repository name from a GitHub clone URL.
///
/// `https://github.com/user/repo-name.git` → `repo-name`
pub fn repo_name_from_url(url: &str) -> String {
    url.rsplit('/')
        .next()
        .unwrap_or(url)
        .trim_end_matches(".git")
        .to_string()
}

/// Minimal URL-encoding for query parameters.
fn urlencoded(s: &str) -> String {
    s.replace(' ', "+").replace(':', "%3A")
}

/// Ensure a private repository exists on GitHub, creating it if needed.
///
/// Returns the HTTPS clone URL.
pub fn github_ensure_repo(token: &str, repo_name: &str) -> Result<String, String> {
    #[derive(Deserialize)]
    struct GithubRepo {
        clone_url: String,
    }

    // First, try to get the repo directly (works for both owner and org repos).
    let username = github_get_username(token)?;
    let get_url = format!("{GITHUB_API}/repos/{username}/{repo_name}");

    let get_result = ureq::get(&get_url)
        .header("User-Agent", USER_AGENT)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .call();

    match get_result {
        Ok(resp) => {
            let repo: GithubRepo = resp
                .into_body()
                .read_json()
                .map_err(|e| format!("Failed to parse repo response: {e}"))?;
            return Ok(repo.clone_url);
        }
        Err(ureq::Error::StatusCode(404)) => {
            // Repo doesn't exist, create it below.
        }
        Err(e) => {
            return Err(format!("GitHub API error checking repo: {e}"));
        }
    }

    // Create the repo.
    #[derive(Serialize)]
    struct CreateRepo<'a> {
        name: &'a str,
        private: bool,
        description: &'a str,
        auto_init: bool,
    }

    let create_url = format!("{GITHUB_API}/user/repos");
    let body = CreateRepo {
        name: repo_name,
        private: true,
        description: "Quest save data (managed by Quest cloud sync)",
        auto_init: false,
    };

    let resp: GithubRepo = ureq::post(&create_url)
        .header("User-Agent", USER_AGENT)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .send_json(&body)
        .map_err(|e| format!("GitHub API error creating repo: {e}"))?
        .into_body()
        .read_json()
        .map_err(|e| format!("Failed to parse create-repo response: {e}"))?;

    // Tag the new repo with our topic so it shows up in the repo picker.
    github_set_topic(token, &username, repo_name);

    Ok(resp.clone_url)
}

/// Set the quest topic on a repository (best-effort, failure is non-fatal).
fn github_set_topic(token: &str, owner: &str, repo_name: &str) {
    #[derive(Serialize)]
    struct TopicBody {
        names: Vec<&'static str>,
    }

    let url = format!("{GITHUB_API}/repos/{owner}/{repo_name}/topics");
    let _ = ureq::put(&url)
        .header("User-Agent", USER_AGENT)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .send_json(&TopicBody {
            names: vec![REPO_TOPIC],
        });
}

/// Transform an HTTPS clone URL into one with embedded token credentials.
///
/// `https://github.com/user/repo.git` becomes
/// `https://x-access-token:TOKEN@github.com/user/repo.git`
pub fn authenticated_url(clone_url: &str, token: &str) -> String {
    clone_url.replacen("https://", &format!("https://x-access-token:{token}@"), 1)
}

// ── git2 remote operations ───────────────────────────────────────────────

/// Build git2 `RemoteCallbacks` that authenticate via a PAT.
fn make_callbacks(token: &str) -> git2::RemoteCallbacks<'_> {
    let mut callbacks = git2::RemoteCallbacks::new();
    let token = token.to_string();
    callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
        git2::Cred::userpass_plaintext("x-access-token", &token)
    });
    callbacks
}

/// Push a single branch to the cloud remote.
fn push_branch(
    repo: &Repository,
    branch_name: &str,
    token: &str,
    force: bool,
) -> Result<(), String> {
    let mut remote = repo
        .find_remote(REMOTE_NAME)
        .map_err(|e| format!("Remote '{REMOTE_NAME}' not found: {e}"))?;

    let prefix = if force { "+" } else { "" };
    let refspec = format!("{prefix}refs/heads/{branch_name}:refs/heads/{branch_name}");

    let callbacks = make_callbacks(token);
    let mut push_opts = git2::PushOptions::new();
    push_opts.remote_callbacks(callbacks);

    remote
        .push(&[&refspec], Some(&mut push_opts))
        .map_err(|e| format!("Failed to push branch '{branch_name}': {e}"))
}

/// Push all local branches to the cloud remote.
pub fn push_all_branches(quest_dir: &Path, token: &str) -> Result<(), String> {
    let repo = Repository::open(quest_dir).map_err(|e| format!("Failed to open repo: {e}"))?;

    let branches: Vec<String> = repo
        .branches(Some(BranchType::Local))
        .map_err(|e| format!("Failed to list branches: {e}"))?
        .filter_map(|b| b.ok())
        .filter_map(|(branch, _)| branch.name().ok().flatten().map(|n| n.to_string()))
        .collect();

    for branch_name in &branches {
        push_branch(&repo, branch_name, token, false)?;
    }

    Ok(())
}

/// Fetch all branches from the cloud remote.
pub fn fetch_all(quest_dir: &Path, token: &str) -> Result<(), String> {
    let repo = Repository::open(quest_dir).map_err(|e| format!("Failed to open repo: {e}"))?;

    let mut remote = repo
        .find_remote(REMOTE_NAME)
        .map_err(|e| format!("Remote '{REMOTE_NAME}' not found: {e}"))?;

    let callbacks = make_callbacks(token);
    let mut fetch_opts = git2::FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);

    remote
        .fetch(
            &[] as &[&str],
            Some(&mut fetch_opts),
            Some("cloud sync fetch"),
        )
        .map_err(|e| format!("Failed to fetch from cloud: {e}"))?;

    Ok(())
}

/// Check whether any branch has diverged (both ahead AND behind remote).
///
/// Returns the first diverged branch found, or `None` if all are
/// fast-forwardable or up-to-date.
pub fn check_divergence(quest_dir: &Path) -> Result<Option<BranchDivergence>, String> {
    let repo = Repository::open(quest_dir).map_err(|e| format!("Failed to open repo: {e}"))?;

    let branches: Vec<String> = repo
        .branches(Some(BranchType::Local))
        .map_err(|e| format!("Failed to list branches: {e}"))?
        .filter_map(|b| b.ok())
        .filter_map(|(branch, _)| branch.name().ok().flatten().map(|n| n.to_string()))
        .collect();

    for branch_name in &branches {
        let remote_ref = format!("refs/remotes/{REMOTE_NAME}/{branch_name}");
        let remote_oid = match repo.refname_to_id(&remote_ref) {
            Ok(oid) => oid,
            Err(_) => continue, // No remote tracking branch — skip.
        };

        let local_ref = format!("refs/heads/{branch_name}");
        let local_oid = match repo.refname_to_id(&local_ref) {
            Ok(oid) => oid,
            Err(_) => continue,
        };

        if local_oid == remote_oid {
            continue;
        }

        let (ahead, behind) = repo
            .graph_ahead_behind(local_oid, remote_oid)
            .map_err(|e| format!("Failed to compare '{branch_name}': {e}"))?;

        if ahead > 0 && behind > 0 {
            // Diverged — extract metadata from both heads.
            let local_commit = repo
                .find_commit(local_oid)
                .map_err(|e| format!("Failed to find local commit: {e}"))?;
            let remote_commit = repo
                .find_commit(remote_oid)
                .map_err(|e| format!("Failed to find remote commit: {e}"))?;

            let local_msg = local_commit.message().unwrap_or("");
            let remote_msg = remote_commit.message().unwrap_or("");

            let (local_level, local_prestige, _, local_playtime) = parse_commit_suffix(local_msg);
            let (remote_level, remote_prestige, _, remote_playtime) =
                parse_commit_suffix(remote_msg);

            return Ok(Some(BranchDivergence {
                branch_name: branch_name.clone(),
                local_level,
                local_prestige,
                local_playtime,
                remote_level,
                remote_prestige,
                remote_playtime,
            }));
        }
    }

    Ok(None)
}

/// Fast-forward all local branches that are behind their remote counterpart.
///
/// Also creates local branches for remote-only branches. If the current
/// branch was updated, resets the working tree to match.
///
/// Returns `Ok(true)` if any branch was updated.
pub fn fast_forward_all(quest_dir: &Path) -> Result<bool, String> {
    let repo = Repository::open(quest_dir).map_err(|e| format!("Failed to open repo: {e}"))?;
    let mut updated = false;

    // Determine which branch is currently checked out.
    let head_branch = repo
        .head()
        .ok()
        .and_then(|r| r.shorthand().map(|s| s.to_string()));

    // Collect remote branch names.
    let remote_branches: Vec<String> = repo
        .branches(Some(BranchType::Remote))
        .map_err(|e| format!("Failed to list remote branches: {e}"))?
        .filter_map(|b| b.ok())
        .filter_map(|(branch, _)| branch.name().ok().flatten().map(|n| n.to_string()))
        .filter(|n| n.starts_with(&format!("{REMOTE_NAME}/")))
        .collect();

    for remote_branch_name in &remote_branches {
        let local_name = match remote_branch_name.strip_prefix(&format!("{REMOTE_NAME}/")) {
            Some(n) => n,
            None => continue,
        };

        let remote_ref = format!("refs/remotes/{remote_branch_name}");
        let remote_oid = repo
            .refname_to_id(&remote_ref)
            .map_err(|e| format!("Failed to resolve remote ref: {e}"))?;

        // Check if local branch exists.
        let local_ref = format!("refs/heads/{local_name}");
        match repo.refname_to_id(&local_ref) {
            Ok(local_oid) => {
                if local_oid == remote_oid {
                    continue; // Already up to date.
                }

                let (ahead, behind) = repo
                    .graph_ahead_behind(local_oid, remote_oid)
                    .map_err(|e| format!("Failed to compare '{local_name}': {e}"))?;

                if ahead > 0 {
                    continue; // Local is ahead or diverged — skip (don't lose local work).
                }

                if behind > 0 {
                    // Fast-forward: move the local ref to the remote commit.
                    repo.reference(
                        &local_ref,
                        remote_oid,
                        true,
                        &format!("cloud sync: fast-forward {local_name}"),
                    )
                    .map_err(|e| format!("Failed to fast-forward '{local_name}': {e}"))?;
                    updated = true;
                }
            }
            Err(_) => {
                // Local branch doesn't exist — create it from remote.
                let remote_commit = repo
                    .find_commit(remote_oid)
                    .map_err(|e| format!("Failed to find remote commit: {e}"))?;

                repo.branch(local_name, &remote_commit, false)
                    .map_err(|e| format!("Failed to create branch '{local_name}': {e}"))?;
                updated = true;
            }
        }
    }

    // If the current branch was updated, reset the working tree.
    if updated {
        if let Some(ref branch_name) = head_branch {
            let local_ref = format!("refs/heads/{branch_name}");
            if let Ok(oid) = repo.refname_to_id(&local_ref) {
                if let Ok(commit) = repo.find_commit(oid) {
                    let _ = repo.reset(commit.as_object(), git2::ResetType::Hard, None);
                }
            }
        }
    }

    Ok(updated)
}

// ── High-level operations ────────────────────────────────────────────────

/// Link to GitHub: validate PAT, ensure repo, add remote, push all, save config.
///
/// Returns the saved `CloudConfig` on success.
pub fn link_github(quest_dir: &Path, token: &str, repo_name: &str) -> Result<CloudConfig, String> {
    // 1. Validate the token and get the username.
    let username = github_get_username(token)?;

    // 2. Ensure the remote repo exists.
    let clone_url = github_ensure_repo(token, repo_name)?;

    // 3. Add or update the git remote.
    let repo = Repository::open(quest_dir).map_err(|e| format!("Failed to open repo: {e}"))?;
    let auth_url = authenticated_url(&clone_url, token);

    // Remove existing remote if present, then add fresh.
    if repo.find_remote(REMOTE_NAME).is_ok() {
        repo.remote_delete(REMOTE_NAME)
            .map_err(|e| format!("Failed to remove old remote: {e}"))?;
    }
    repo.remote(REMOTE_NAME, &auth_url)
        .map_err(|e| format!("Failed to add remote: {e}"))?;

    // 4. Configure fetch refspec so `fetch_all` works.
    configure_fetch_refspec(&repo)?;

    // 5. Fetch remote refs (best-effort; empty repos will error, which is fine).
    let _ = fetch_all(quest_dir, token);

    // 6. Save config.
    let config = CloudConfig {
        token: token.to_string(),
        username,
        repo_url: clone_url,
    };
    save_config(quest_dir, &config)?;

    Ok(config)
}

/// Link and pull: validate PAT, ensure repo, add remote, fetch, fast-forward, save config.
///
/// Designed for new machines restoring saves from the cloud.
pub fn link_and_pull(
    quest_dir: &Path,
    token: &str,
    repo_name: &str,
) -> Result<CloudConfig, String> {
    // 1. Validate the token and get the username.
    let username = github_get_username(token)?;

    // 2. Ensure the remote repo exists.
    let clone_url = github_ensure_repo(token, repo_name)?;

    // 3. Add or update the git remote.
    let repo = Repository::open(quest_dir).map_err(|e| format!("Failed to open repo: {e}"))?;
    let auth_url = authenticated_url(&clone_url, token);

    if repo.find_remote(REMOTE_NAME).is_ok() {
        repo.remote_delete(REMOTE_NAME)
            .map_err(|e| format!("Failed to remove old remote: {e}"))?;
    }
    repo.remote(REMOTE_NAME, &auth_url)
        .map_err(|e| format!("Failed to add remote: {e}"))?;

    // 4. Configure fetch refspec so `fetch_all` works.
    configure_fetch_refspec(&repo)?;

    // 5. Fetch all branches from remote.
    fetch_all(quest_dir, token)?;

    // 6. Fast-forward local branches.
    fast_forward_all(quest_dir)?;

    // 7. Save config.
    let config = CloudConfig {
        token: token.to_string(),
        username,
        repo_url: clone_url,
    };
    save_config(quest_dir, &config)?;

    Ok(config)
}

/// Remove the cloud remote and delete the saved config.
pub fn unlink(quest_dir: &Path) -> Result<(), String> {
    let repo = Repository::open(quest_dir).map_err(|e| format!("Failed to open repo: {e}"))?;

    if repo.find_remote(REMOTE_NAME).is_ok() {
        repo.remote_delete(REMOTE_NAME)
            .map_err(|e| format!("Failed to remove remote: {e}"))?;
    }

    delete_config(quest_dir)
}

/// Force-push a single branch to the cloud remote (overwrites remote history).
///
/// Used to resolve divergence by choosing the local version.
pub fn force_push_branch(quest_dir: &Path, branch_name: &str, token: &str) -> Result<(), String> {
    let repo = Repository::open(quest_dir).map_err(|e| format!("Failed to open repo: {e}"))?;
    push_branch(&repo, branch_name, token, true)
}

/// Create a backup branch at the current local head, then reset the local
/// branch to match the remote.
///
/// Returns the backup branch name (e.g. `backup-main-20260223-1430`).
/// Reset a local branch to match the remote, discarding local commits.
/// Unlike `fast_forward_all`, this works even when the branch has diverged.
pub fn reset_to_remote(quest_dir: &Path, branch_name: &str) -> Result<(), String> {
    let repo = Repository::open(quest_dir).map_err(|e| format!("Failed to open repo: {e}"))?;

    // Resolve remote head.
    let remote_ref = format!("refs/remotes/{REMOTE_NAME}/{branch_name}");
    let remote_oid = repo
        .refname_to_id(&remote_ref)
        .map_err(|e| format!("Remote branch '{branch_name}' not found: {e}"))?;
    let remote_commit = repo
        .find_commit(remote_oid)
        .map_err(|e| format!("Failed to find remote commit: {e}"))?;

    // If this is the current branch, use git reset --hard which atomically
    // moves the branch ref, updates the index, and resets the working tree.
    let head_branch = repo
        .head()
        .ok()
        .and_then(|r| r.shorthand().map(|s| s.to_string()));
    if head_branch.as_deref() == Some(branch_name) {
        repo.reset(remote_commit.as_object(), git2::ResetType::Hard, None)
            .map_err(|e| format!("Failed to reset to remote: {e}"))?;
    } else {
        // Not on this branch — just move the ref directly.
        let local_ref = format!("refs/heads/{branch_name}");
        repo.reference(
            &local_ref,
            remote_oid,
            true,
            &format!("cloud sync: reset {branch_name} to remote"),
        )
        .map_err(|e| format!("Failed to reset branch: {e}"))?;
    }

    Ok(())
}

pub fn backup_and_reset(quest_dir: &Path, branch_name: &str) -> Result<String, String> {
    let repo = Repository::open(quest_dir).map_err(|e| format!("Failed to open repo: {e}"))?;

    // Resolve local head.
    let local_ref = format!("refs/heads/{branch_name}");
    let local_oid = repo
        .refname_to_id(&local_ref)
        .map_err(|e| format!("Branch '{branch_name}' not found: {e}"))?;
    let local_commit = repo
        .find_commit(local_oid)
        .map_err(|e| format!("Failed to find commit: {e}"))?;

    // Generate a timestamped backup name.
    let now = chrono::Local::now();
    let backup_name = format!("backup-{}-{}", branch_name, now.format("%Y%m%d-%H%M"));

    // Create backup branch at local head.
    repo.branch(&backup_name, &local_commit, false)
        .map_err(|e| format!("Failed to create backup branch: {e}"))?;

    // Resolve remote head.
    let remote_ref = format!("refs/remotes/{REMOTE_NAME}/{branch_name}");
    let remote_oid = repo
        .refname_to_id(&remote_ref)
        .map_err(|e| format!("Remote branch '{branch_name}' not found: {e}"))?;
    let remote_commit = repo
        .find_commit(remote_oid)
        .map_err(|e| format!("Failed to find remote commit: {e}"))?;

    // If this is the current branch, use git reset --hard which atomically
    // moves the branch ref, updates the index, and resets the working tree.
    let head_branch = repo
        .head()
        .ok()
        .and_then(|r| r.shorthand().map(|s| s.to_string()));
    if head_branch.as_deref() == Some(branch_name) {
        repo.reset(remote_commit.as_object(), git2::ResetType::Hard, None)
            .map_err(|e| format!("Failed to reset to remote: {e}"))?;
    } else {
        // Not on this branch — just move the ref directly.
        repo.reference(
            &local_ref,
            remote_oid,
            true,
            &format!("cloud sync: reset {branch_name} to remote"),
        )
        .map_err(|e| format!("Failed to reset branch: {e}"))?;
    }

    Ok(backup_name)
}

// ── Internal helpers ─────────────────────────────────────────────────────

/// Configure the fetch refspec for the cloud remote so that `git fetch`
/// populates `refs/remotes/cloud/*`.
fn configure_fetch_refspec(repo: &Repository) -> Result<(), String> {
    // Check if the refspec is already configured.
    let remote = repo
        .find_remote(REMOTE_NAME)
        .map_err(|e| format!("Remote not found: {e}"))?;

    let expected = format!("+refs/heads/*:refs/remotes/{REMOTE_NAME}/*");
    let has_refspec = remote
        .fetch_refspecs()
        .map(|specs| specs.iter().any(|s| s.is_some_and(|s| s == expected)))
        .unwrap_or(false);

    if !has_refspec {
        repo.remote_add_fetch(REMOTE_NAME, &expected)
            .map_err(|e| format!("Failed to configure fetch refspec: {e}"))?;
    }

    Ok(())
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authenticated_url_inserts_token() {
        let url = "https://github.com/user/repo.git";
        let result = authenticated_url(url, "ghp_abc123");
        assert_eq!(
            result,
            "https://x-access-token:ghp_abc123@github.com/user/repo.git"
        );
    }

    #[test]
    fn authenticated_url_handles_no_https() {
        // If the URL doesn't start with https://, replacen does nothing.
        let url = "http://github.com/user/repo.git";
        let result = authenticated_url(url, "token");
        assert_eq!(result, "http://github.com/user/repo.git");
    }

    #[test]
    fn config_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let config = CloudConfig {
            token: "ghp_test".to_string(),
            username: "testuser".to_string(),
            repo_url: "https://github.com/testuser/quest-saves.git".to_string(),
        };

        save_config(dir.path(), &config).unwrap();
        let loaded = load_config(dir.path()).unwrap();
        assert_eq!(loaded.token, config.token);
        assert_eq!(loaded.username, config.username);
        assert_eq!(loaded.repo_url, config.repo_url);
    }

    #[test]
    fn config_load_missing_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        assert!(load_config(dir.path()).is_none());
    }

    #[test]
    fn config_delete_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        // Deleting non-existent config should succeed.
        assert!(delete_config(dir.path()).is_ok());
    }

    #[test]
    fn config_delete_removes_file() {
        let dir = tempfile::tempdir().unwrap();
        let config = CloudConfig {
            token: "t".to_string(),
            username: "u".to_string(),
            repo_url: "r".to_string(),
        };
        save_config(dir.path(), &config).unwrap();
        assert!(load_config(dir.path()).is_some());

        delete_config(dir.path()).unwrap();
        assert!(load_config(dir.path()).is_none());
    }
}
