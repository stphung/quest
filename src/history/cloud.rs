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

/// Timeout for HTTP requests to GitHub API.
const HTTP_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);

/// Create a ureq agent with standard timeout configuration.
fn github_agent() -> ureq::Agent {
    ureq::Agent::new_with_config(
        ureq::Agent::config_builder()
            .timeout_global(Some(HTTP_TIMEOUT))
            .build(),
    )
}

/// Repository metadata returned from the GitHub API.
#[derive(Debug, Clone)]
pub struct RepoInfo {
    pub name: String,
    pub private: bool,
}

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
    /// The stored PAT is expired or revoked (HTTP 401/403).
    TokenExpired,
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
        repos: Vec<RepoInfo>,
    },
    /// All branches pushed to remote.
    Pushed,
    /// All branches pulled (fast-forwarded) from remote.
    Pulled,
    /// Cloud remote and config removed.
    Unlinked,
    /// At least one branch has diverged (ahead AND behind).
    Diverged(BranchDivergence),
    /// Token was successfully updated. Contains the new config.
    TokenUpdated(CloudConfig),
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
    let resp: GithubUser = github_agent()
        .get(&url)
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
pub fn github_list_repos(token: &str) -> Result<Vec<RepoInfo>, String> {
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
fn github_list_repos_by_topic(token: &str, username: &str) -> Result<Vec<RepoInfo>, String> {
    #[derive(Deserialize)]
    struct SearchResult {
        items: Vec<SearchItem>,
    }
    #[derive(Deserialize)]
    struct SearchItem {
        name: String,
        private: bool,
    }

    let query = format!("topic:{REPO_TOPIC} user:{username}");
    let url = format!(
        "{GITHUB_API}/search/repositories?q={}&per_page=100",
        urlencoded(&query)
    );
    let result: SearchResult = github_agent()
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| format!("GitHub API error: {e}"))?
        .into_body()
        .read_json()
        .map_err(|e| format!("Failed to parse search response: {e}"))?;

    Ok(result
        .items
        .into_iter()
        .map(|r| RepoInfo {
            name: r.name,
            private: r.private,
        })
        .collect())
}

/// List all repositories owned by the authenticated user.
fn github_list_all_user_repos(token: &str) -> Result<Vec<RepoInfo>, String> {
    #[derive(Deserialize)]
    struct RepoItem {
        name: String,
        private: bool,
    }

    let url = format!("{GITHUB_API}/user/repos?per_page=100&sort=updated&affiliation=owner");
    let repos: Vec<RepoItem> = github_agent()
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| format!("GitHub API error: {e}"))?
        .into_body()
        .read_json()
        .map_err(|e| format!("Failed to parse repos response: {e}"))?;

    Ok(repos
        .into_iter()
        .map(|r| RepoInfo {
            name: r.name,
            private: r.private,
        })
        .collect())
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

/// Ensure a repository exists on GitHub, creating it if needed.
///
/// Returns the HTTPS clone URL. The `private` flag is only used when creating
/// a new repo; existing repos keep their current visibility.
pub fn github_ensure_repo(token: &str, repo_name: &str, private: bool) -> Result<String, String> {
    #[derive(Deserialize)]
    struct GithubRepo {
        clone_url: String,
    }

    // First, try to get the repo directly (works for both owner and org repos).
    let username = github_get_username(token)?;
    let get_url = format!("{GITHUB_API}/repos/{username}/{repo_name}");

    let get_result = github_agent()
        .get(&get_url)
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
        private,
        description: "Quest save data (managed by Quest cloud sync)",
        auto_init: false,
    };

    let resp: GithubRepo = github_agent()
        .post(&create_url)
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

    // Delete any auto-applied rulesets that would block pushes.
    // GitHub can apply default branch rulesets to new repos which reject
    // pushes with "push declined due to repository rule violations".
    github_delete_rulesets(token, &username, repo_name);

    // Disable secret scanning push protection which is enabled by default
    // on all GitHub repos (since 2024). It can reject pushes if it detects
    // anything resembling a secret in commit content.
    github_disable_push_protection(token, &username, repo_name);

    Ok(resp.clone_url)
}

/// Delete all rulesets on a repository (best-effort, failure is non-fatal).
///
/// GitHub may auto-apply default rulesets to newly created repos that block
/// pushes. We remove them so cloud sync can push freely.
fn github_delete_rulesets(token: &str, owner: &str, repo_name: &str) {
    #[derive(Deserialize)]
    struct Ruleset {
        id: u64,
    }

    let url = format!("{GITHUB_API}/repos/{owner}/{repo_name}/rulesets");
    let Ok(resp) = github_agent()
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .call()
    else {
        return;
    };

    let Ok(rulesets): Result<Vec<Ruleset>, _> = resp.into_body().read_json() else {
        return;
    };

    for ruleset in rulesets {
        let delete_url = format!(
            "{GITHUB_API}/repos/{owner}/{repo_name}/rulesets/{}",
            ruleset.id
        );
        let _ = github_agent()
            .delete(&delete_url)
            .header("User-Agent", USER_AGENT)
            .header("Authorization", &format!("Bearer {token}"))
            .header("Accept", "application/vnd.github+json")
            .call();
    }
}

/// Disable secret scanning push protection on a repository (best-effort).
///
/// GitHub enables push protection by default on all repos since 2024. This can
/// reject pushes if it detects patterns resembling secrets in commit content.
/// Since quest saves are game data (not code), we disable this to avoid false
/// positives blocking sync.
fn github_disable_push_protection(token: &str, owner: &str, repo_name: &str) {
    let url = format!("{GITHUB_API}/repos/{owner}/{repo_name}");
    let body = serde_json::json!({
        "security_and_analysis": {
            "secret_scanning_push_protection": {
                "status": "disabled"
            }
        }
    });

    let _ = github_agent()
        .patch(&url)
        .header("User-Agent", USER_AGENT)
        .header("Authorization", &format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .send_json(&body);
}

/// Set the quest topic on a repository (best-effort, failure is non-fatal).
fn github_set_topic(token: &str, owner: &str, repo_name: &str) {
    #[derive(Serialize)]
    struct TopicBody {
        names: Vec<&'static str>,
    }

    let url = format!("{GITHUB_API}/repos/{owner}/{repo_name}/topics");
    let _ = github_agent()
        .put(&url)
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

    // Capture per-ref push errors via callback. git2's remote.push() can
    // return Ok(()) even when the remote rejects refs — the only way to
    // detect rejection is through push_update_reference.
    let push_error: std::sync::Arc<std::sync::Mutex<Option<String>>> =
        std::sync::Arc::new(std::sync::Mutex::new(None));
    let push_error_cb = push_error.clone();

    let mut callbacks = make_callbacks(token);
    callbacks.push_update_reference(move |refname, status| {
        if let Some(msg) = status {
            *push_error_cb.lock().unwrap() = Some(format!("Remote rejected {refname}: {msg}"));
        }
        Ok(())
    });

    let mut push_opts = git2::PushOptions::new();
    push_opts.remote_callbacks(callbacks);

    remote
        .push(&[&refspec], Some(&mut push_opts))
        .map_err(|e| format!("Failed to push branch '{branch_name}': {e}"))?;

    // Check if the remote rejected the ref.
    if let Some(err) = push_error.lock().unwrap().take() {
        return Err(err);
    }

    Ok(())
}

/// Push all local branches to the cloud remote.
pub fn push_all_branches(quest_dir: &Path, token: &str) -> Result<(), String> {
    let repo = Repository::open(quest_dir).map_err(|e| format!("Failed to open repo: {e}"))?;

    // Safety: ensure .cloud.json is not in the index before pushing.
    // If it was accidentally tracked in an older version, remove it now
    // and create a cleanup commit.
    scrub_cloud_config_from_index(&repo);

    let branches: Vec<String> = repo
        .branches(Some(BranchType::Local))
        .map_err(|e| format!("Failed to list branches: {e}"))?
        .filter_map(|b| b.ok())
        .filter_map(|(branch, _)| branch.name().ok().flatten().map(|n| n.to_string()))
        .collect();

    for branch_name in &branches {
        match push_branch(&repo, branch_name, token, false) {
            Ok(()) => {}
            Err(e) if e.contains("push declined") || e.contains("repository rule") => {
                // Push protection rejected the push — likely .cloud.json is
                // in historical commits. Squash the branch to a single clean
                // orphan commit and force-push.
                squash_branch_clean(&repo, branch_name)?;
                push_branch(&repo, branch_name, token, true)?;
            }
            Err(e) if e.contains("non-fast-forward") => {
                // Remote has commits local doesn't (e.g. GitHub auto-init
                // README). Local saves are authoritative — force-push.
                push_branch(&repo, branch_name, token, true)?;
            }
            Err(e) => return Err(e),
        }
    }

    Ok(())
}

/// Squash a branch into a single orphan commit with `.cloud.json` removed.
///
/// This is a last-resort fix when push protection rejects the push because
/// `.cloud.json` (containing the PAT) was committed in historical commits.
/// The branch is replaced with a single parentless commit containing the
/// latest tree, minus `.cloud.json`.
fn squash_branch_clean(repo: &Repository, branch_name: &str) -> Result<(), String> {
    let branch = repo
        .find_branch(branch_name, BranchType::Local)
        .map_err(|e| format!("Branch '{branch_name}' not found: {e}"))?;
    let tip = branch
        .get()
        .peel_to_commit()
        .map_err(|e| format!("Failed to get branch tip: {e}"))?;

    // Build a clean tree (original tree minus .cloud.json).
    let orig_tree = tip.tree().map_err(|e| format!("Failed to get tree: {e}"))?;
    let clean_tree_oid = build_tree_without(repo, &orig_tree, ".cloud.json")?;
    let clean_tree = repo
        .find_tree(clean_tree_oid)
        .map_err(|e| format!("Failed to find clean tree: {e}"))?;

    // Reuse the tip's commit message and signature.
    let sig = tip.author();
    let message = tip.message().unwrap_or("Save state");

    let new_oid = repo
        .commit(None, &sig, &sig, message, &clean_tree, &[])
        .map_err(|e| format!("Failed to create squashed commit: {e}"))?;

    // Point the branch at the new orphan commit.
    // Use reference_set_target instead of repo.branch() because the latter
    // cannot force-update the branch HEAD currently points to.
    let refname = format!("refs/heads/{branch_name}");
    repo.reference(&refname, new_oid, true, "squash for clean cloud push")
        .map_err(|e| format!("Failed to update branch '{branch_name}': {e}"))?;

    Ok(())
}

/// Rebuild a tree object with one entry removed (by name).
fn build_tree_without(
    repo: &Repository,
    tree: &git2::Tree<'_>,
    remove_name: &str,
) -> Result<git2::Oid, String> {
    let mut builder = repo
        .treebuilder(Some(tree))
        .map_err(|e| format!("Failed to create tree builder: {e}"))?;
    let _ = builder.remove(remove_name); // ignore if not present
    builder
        .write()
        .map_err(|e| format!("Failed to write clean tree: {e}"))
}

/// Remove `.cloud.json` from the git index and rewrite HEAD if it was tracked.
///
/// If `.cloud.json` is in the current index (meaning it would appear in commits),
/// remove it and create a new commit so the pushed history doesn't contain the
/// cloud config (which includes the PAT token).
fn scrub_cloud_config_from_index(repo: &Repository) {
    let Ok(mut index) = repo.index() else {
        return;
    };

    let cloud_path = std::path::Path::new(".cloud.json");
    if index.get_path(cloud_path, 0).is_none() {
        return; // Not tracked, nothing to do.
    }

    // Remove from index.
    let _ = index.remove_path(cloud_path);
    let _ = index.write();

    // Create a new commit with the cleaned tree.
    let Ok(tree_oid) = index.write_tree() else {
        return;
    };
    let Ok(tree) = repo.find_tree(tree_oid) else {
        return;
    };
    let Ok(head) = repo.head() else {
        return;
    };
    let Ok(parent) = head.peel_to_commit() else {
        return;
    };
    let Ok(sig) = git2::Signature::now("Quest", "quest@game") else {
        return;
    };

    let _ = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        "Remove cloud config from tracking",
        &tree,
        &[&parent],
    );
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
        .and_then(|r| r.shorthand().ok().map(|s| s.to_string()));

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
                    repo.reset(commit.as_object(), git2::ResetType::Hard, None)
                        .map_err(|e| format!("Failed to reset working tree: {e}"))?;
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
pub fn link_github(
    quest_dir: &Path,
    token: &str,
    repo_name: &str,
    private: bool,
) -> Result<CloudConfig, String> {
    // 1. Validate the token and get the username.
    let username = github_get_username(token)?;

    // 2. Ensure the remote repo exists.
    let clone_url = github_ensure_repo(token, repo_name, private)?;

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

    // 2. Ensure the remote repo exists (repo already exists when updating token).
    let clone_url = github_ensure_repo(token, repo_name, true)?;

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

/// Update the PAT for an existing cloud link.
///
/// Validates the new token, updates the saved config, and re-creates the
/// git remote with the new auth URL. The existing repo link is preserved.
pub fn update_token(
    quest_dir: &Path,
    new_token: &str,
    config: &CloudConfig,
) -> Result<CloudConfig, String> {
    // 1. Validate the new token.
    let username = github_get_username(new_token)?;

    // 2. Re-create the git remote with updated credentials.
    let repo = Repository::open(quest_dir).map_err(|e| format!("Failed to open repo: {e}"))?;
    let auth_url = authenticated_url(&config.repo_url, new_token);

    if repo.find_remote(REMOTE_NAME).is_ok() {
        repo.remote_delete(REMOTE_NAME)
            .map_err(|e| format!("Failed to remove old remote: {e}"))?;
    }
    repo.remote(REMOTE_NAME, &auth_url)
        .map_err(|e| format!("Failed to add remote: {e}"))?;

    configure_fetch_refspec(&repo)?;

    // 3. Save updated config.
    let new_config = CloudConfig {
        token: new_token.to_string(),
        username,
        repo_url: config.repo_url.clone(),
    };
    save_config(quest_dir, &new_config)?;

    Ok(new_config)
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
        .and_then(|r| r.shorthand().ok().map(|s| s.to_string()));
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
        .and_then(|r| r.shorthand().ok().map(|s| s.to_string()));
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
        .map(|specs| {
            specs
                .iter()
                .any(|s| s.ok().flatten().is_some_and(|s| s == expected))
        })
        .unwrap_or(false);

    if !has_refspec {
        repo.remote_add_fetch(REMOTE_NAME, &expected)
            .map_err(|e| format!("Failed to configure fetch refspec: {e}"))?;
    }

    Ok(())
}

/// Check whether an error message indicates an authentication failure (expired/revoked PAT).
///
/// Matches HTTP 401/403 status codes and common GitHub auth error strings from ureq.
/// Excludes rate-limiting (HTTP 403 with "rate limit") which is transient, not an auth issue.
pub fn is_auth_error(error_msg: &str) -> bool {
    let msg = error_msg.to_lowercase();
    // Rate-limiting is NOT an auth error — user doesn't need a new token.
    if msg.contains("rate limit") {
        return false;
    }
    msg.contains("status code 401")
        || msg.contains("status code 403")
        || msg.contains("bad credentials")
        || (msg.contains("401") && (msg.contains("unauthorized") || msg.contains("auth")))
}

/// Convert a raw cloud error message into a user-friendly string.
pub fn sanitize_cloud_error(error_msg: &str) -> String {
    let msg = error_msg.to_lowercase();
    if msg.contains("rate limit") {
        return "GitHub rate limit reached — try again in a minute".to_string();
    }
    if msg.contains("status code 500")
        || msg.contains("status code 502")
        || msg.contains("status code 503")
    {
        return "GitHub is temporarily unavailable — try again later".to_string();
    }
    if msg.contains("connection") && (msg.contains("refused") || msg.contains("reset")) {
        return "Could not connect to GitHub — check your internet".to_string();
    }
    if msg.contains("dns") || msg.contains("resolve") {
        return "Could not reach GitHub — check your internet".to_string();
    }
    if msg.contains("timed out") || msg.contains("timeout") {
        return "Connection to GitHub timed out — try again".to_string();
    }
    if msg.contains("push declined") || msg.contains("repository rule") {
        return "Push blocked by GitHub push protection — see wiki".to_string();
    }
    // Truncate long raw errors for display.
    if error_msg.len() > 60 {
        format!("{}...", &error_msg[..57])
    } else {
        error_msg.to_string()
    }
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

    #[test]
    fn is_auth_error_detects_401() {
        assert!(is_auth_error(
            "https://api.github.com/user: status code 401"
        ));
    }

    #[test]
    fn is_auth_error_detects_403() {
        assert!(is_auth_error("GitHub API error: status code 403"));
    }

    #[test]
    fn is_auth_error_detects_bad_credentials() {
        assert!(is_auth_error("Bad credentials"));
    }

    #[test]
    fn is_auth_error_rejects_404() {
        assert!(!is_auth_error("status code 404"));
    }

    #[test]
    fn is_auth_error_rejects_network_error() {
        assert!(!is_auth_error("connection refused"));
    }

    #[test]
    fn is_auth_error_rejects_rate_limit() {
        assert!(!is_auth_error("API rate limit exceeded for user"));
        assert!(!is_auth_error("status code 403 - rate limit"));
    }

    #[test]
    fn sanitize_cloud_error_rate_limit() {
        let msg = sanitize_cloud_error("API rate limit exceeded for user");
        assert!(msg.contains("rate limit"));
        assert!(msg.contains("try again"));
    }

    #[test]
    fn sanitize_cloud_error_server_down() {
        let msg = sanitize_cloud_error("GitHub API error: status code 502");
        assert!(msg.contains("temporarily unavailable"));
    }

    #[test]
    fn sanitize_cloud_error_timeout() {
        let msg = sanitize_cloud_error("operation timed out");
        assert!(msg.contains("timed out"));
    }

    #[test]
    fn sanitize_cloud_error_passes_short_unknown() {
        assert_eq!(sanitize_cloud_error("some error"), "some error");
    }

    #[test]
    fn sanitize_cloud_error_truncates_long_messages() {
        let long = "a".repeat(100);
        let result = sanitize_cloud_error(&long);
        assert!(result.len() < 65);
        assert!(result.ends_with("..."));
    }
}
