> Backported implementation plan (completed — this work shipped).

## 2026-02-23-cloud-sync-plan.md

# Cloud Sync Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enable players to back up Time Vault saves to GitHub and restore them on another machine.

**Architecture:** Build on the existing `cloud.rs` from commit `d397416`. Add auto-fetch on launch, push-on-commit, divergence detection with resolution dialog, and first-launch restore prompt. Uses `git2` for all git operations and `ureq` for GitHub API calls.

**Tech Stack:** Rust, git2, ureq, serde_json, std::thread + mpsc for background ops.

**Design Doc:** `docs/plans/2026-02-23-cloud-sync-design.md`

---

### Task 1: Create cloud module with types and config persistence

**Files:**
- Create: `src/history/cloud.rs`
- Modify: `src/history/mod.rs`

**Step 1: Create `src/history/cloud.rs` with types and config persistence**

```rust
//! GitHub cloud sync for Time Vault saves.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Name of the git remote used for cloud sync.
pub const REMOTE_NAME: &str = "cloud";

/// Default repository name created on the user's GitHub account.
pub const DEFAULT_REPO_NAME: &str = "quest-saves";

/// User-Agent header for GitHub API requests.
pub const USER_AGENT: &str = "quest-cloud-sync";

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
    Offline,
    /// Linked and idle.
    Linked,
    /// Operation in progress (push, pull, link).
    Syncing,
    /// Local branch has diverged from cloud.
    OutOfSync,
    /// Last operation failed.
    Error(String),
}

/// Result of a completed cloud background operation.
#[derive(Debug)]
pub enum CloudOpResult {
    /// Link succeeded — returns the new config.
    Linked(CloudConfig),
    /// Push succeeded.
    Pushed,
    /// Pull succeeded — caller should reload state from disk.
    Pulled,
    /// Unlink succeeded.
    Unlinked,
    /// Push failed due to non-fast-forward (divergence).
    Diverged,
    /// Operation failed.
    Failed(String),
}

/// Divergence info for a single branch.
#[derive(Debug, Clone)]
pub struct BranchDivergence {
    pub branch_name: String,
    pub local_level: u32,
    pub local_prestige: u32,
    pub local_playtime: u64,
    pub remote_level: u32,
    pub remote_prestige: u32,
    pub remote_playtime: u64,
}

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
pub fn save_config(quest_dir: &Path, config: &CloudConfig) -> Result<(), String> {
    let path = config_path(quest_dir);
    let data = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(path, data).map_err(|e| format!("failed to save cloud config: {e}"))
}

/// Delete cloud config from disk.
pub fn delete_config(quest_dir: &Path) {
    let _ = std::fs::remove_file(config_path(quest_dir));
}
```

**Step 2: Add `cloud` module to `src/history/mod.rs`**

Add after existing exports:
```rust
pub mod cloud;
```

**Step 3: Verify it compiles**

Run: `cargo build 2>&1 | head -5`
Expected: compiles with no errors (may have unused warnings, that's fine)

**Step 4: Commit**

```bash
git add src/history/cloud.rs src/history/mod.rs
git commit -m "feat(cloud): add cloud sync types and config persistence"
```

---

### Task 2: Add GitHub API helpers

**Files:**
- Modify: `src/history/cloud.rs`

**Step 1: Add GitHub API functions to `cloud.rs`**

Append after the config persistence section:

```rust
// ── GitHub API helpers ───────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct GitHubUser {
    login: String,
}

#[derive(serde::Deserialize)]
struct GitHubRepo {
    clone_url: String,
}

/// Validate a PAT and return the authenticated username.
pub fn github_get_username(token: &str) -> Result<String, String> {
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
pub fn github_ensure_repo(token: &str, repo_name: &str) -> Result<String, String> {
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
fn authenticated_url(clone_url: &str, token: &str) -> String {
    if let Some(rest) = clone_url.strip_prefix("https://") {
        format!("https://x-access-token:{token}@{rest}")
    } else {
        clone_url.to_string()
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo build 2>&1 | head -5`

**Step 3: Commit**

```bash
git add src/history/cloud.rs
git commit -m "feat(cloud): add GitHub API helpers (validate PAT, ensure repo)"
```

---

### Task 3: Add git2 remote operations (push, fetch, divergence check)

**Files:**
- Modify: `src/history/cloud.rs`

**Step 1: Add git2 remote operations**

Append to `cloud.rs`:

```rust
use git2::{BranchType, FetchOptions, PushOptions, RemoteCallbacks, Repository};

// ── git2 remote operations ──────────────────────────────────────────────

fn make_callbacks(token: String) -> RemoteCallbacks<'static> {
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
        git2::Cred::userpass_plaintext("x-access-token", &token)
    });
    callbacks
}

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

/// Push all local branches to the cloud remote.
pub fn push_all_branches(quest_dir: &Path, token: &str) -> Result<(), String> {
    let repo = Repository::open(quest_dir).map_err(|e| format!("failed to open repo: {e}"))?;
    let branches: Vec<String> = repo
        .branches(Some(BranchType::Local))
        .map_err(|e| format!("failed to list branches: {e}"))?
        .filter_map(|b| {
            b.ok()
                .and_then(|(branch, _)| branch.name().ok().flatten().map(|n| n.to_string()))
        })
        .collect();

    for branch_name in &branches {
        push_branch(&repo, branch_name, token)?;
    }
    Ok(())
}

/// Fetch all branches from the cloud remote.
pub fn fetch_all(quest_dir: &Path, token: &str) -> Result<(), String> {
    let repo = Repository::open(quest_dir).map_err(|e| format!("failed to open repo: {e}"))?;
    let mut remote = repo
        .find_remote(REMOTE_NAME)
        .map_err(|e| format!("cloud remote not found: {e}"))?;

    let callbacks = make_callbacks(token.to_string());
    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);

    remote
        .fetch(
            &["refs/heads/*:refs/remotes/cloud/*"],
            Some(&mut fetch_opts),
            None,
        )
        .map_err(|e| format!("fetch failed: {e}"))?;
    Ok(())
}

/// Check divergence status for all branches.
///
/// Returns `Ok(None)` if all branches are in sync or local-only.
/// Returns `Ok(Some(divergence))` if any branch has diverged.
/// "Behind" branches are safe to fast-forward.
/// "Ahead" branches are safe (push will handle them).
/// "Diverged" (ahead AND behind) blocks the pull.
pub fn check_divergence(quest_dir: &Path) -> Result<Option<BranchDivergence>, String> {
    let repo = Repository::open(quest_dir).map_err(|e| format!("failed to open repo: {e}"))?;

    let branches: Vec<String> = repo
        .branches(Some(BranchType::Local))
        .map_err(|e| format!("failed to list branches: {e}"))?
        .filter_map(|b| {
            b.ok()
                .and_then(|(branch, _)| branch.name().ok().flatten().map(|n| n.to_string()))
        })
        .collect();

    for branch_name in &branches {
        let remote_ref = format!("refs/remotes/cloud/{branch_name}");
        let remote_ref_obj = match repo.find_reference(&remote_ref) {
            Ok(r) => r,
            Err(_) => continue, // local-only branch, skip
        };

        let local_ref = format!("refs/heads/{branch_name}");
        let local_ref_obj = repo
            .find_reference(&local_ref)
            .map_err(|e| format!("failed to find local ref: {e}"))?;

        let local_oid = local_ref_obj
            .target()
            .ok_or("local ref has no target")?;
        let remote_oid = remote_ref_obj
            .target()
            .ok_or("remote ref has no target")?;

        if local_oid == remote_oid {
            continue; // in sync
        }

        let (ahead, behind) = repo
            .graph_ahead_behind(local_oid, remote_oid)
            .map_err(|e| format!("graph_ahead_behind failed: {e}"))?;

        if ahead > 0 && behind > 0 {
            // Diverged — extract metadata from both heads for the UI
            let local_commit = repo.find_commit(local_oid)
                .map_err(|e| format!("find local commit: {e}"))?;
            let remote_commit = repo.find_commit(remote_oid)
                .map_err(|e| format!("find remote commit: {e}"))?;

            let local_msg = local_commit.message().unwrap_or("").to_string();
            let remote_msg = remote_commit.message().unwrap_or("").to_string();

            let (ll, lp, _, lt) = super::git::parse_commit_suffix(&local_msg);
            let (rl, rp, _, rt) = super::git::parse_commit_suffix(&remote_msg);

            return Ok(Some(BranchDivergence {
                branch_name: branch_name.clone(),
                local_level: ll,
                local_prestige: lp,
                local_playtime: lt,
                remote_level: rl,
                remote_prestige: rp,
                remote_playtime: rt,
            }));
        }
        // ahead>0, behind==0: local is ahead, push will handle it
        // ahead==0, behind>0: behind, fast_forward_all will handle it
    }

    Ok(None)
}

/// Fast-forward all local branches that are behind their remote counterparts.
/// Creates local branches for remote-only branches.
pub fn fast_forward_all(quest_dir: &Path) -> Result<bool, String> {
    let repo = Repository::open(quest_dir).map_err(|e| format!("failed to open repo: {e}"))?;
    let mut updated = false;

    // Collect remote branch names
    let remote_branches: Vec<String> = repo
        .branches(Some(BranchType::Remote))
        .map_err(|e| format!("failed to list remote branches: {e}"))?
        .filter_map(|b| {
            b.ok().and_then(|(branch, _)| {
                branch
                    .name()
                    .ok()
                    .flatten()
                    .and_then(|n| n.strip_prefix("cloud/"))
                    .map(|n| n.to_string())
            })
        })
        .collect();

    for branch_name in &remote_branches {
        let remote_ref = format!("refs/remotes/cloud/{branch_name}");
        let remote_ref_obj = repo
            .find_reference(&remote_ref)
            .map_err(|e| format!("find remote ref: {e}"))?;
        let remote_oid = remote_ref_obj
            .target()
            .ok_or("remote ref has no target")?;
        let remote_commit = repo
            .find_commit(remote_oid)
            .map_err(|e| format!("find remote commit: {e}"))?;

        let local_ref = format!("refs/heads/{branch_name}");
        match repo.find_reference(&local_ref) {
            Ok(local_ref_obj) => {
                let local_oid = local_ref_obj.target().ok_or("local ref has no target")?;
                if local_oid == remote_oid {
                    continue; // in sync
                }
                let (ahead, _behind) = repo
                    .graph_ahead_behind(local_oid, remote_oid)
                    .map_err(|e| format!("graph_ahead_behind: {e}"))?;
                if ahead > 0 {
                    continue; // local is ahead, don't touch
                }
                // Behind — fast-forward
                let mut local_ref_mut = repo
                    .find_reference(&local_ref)
                    .map_err(|e| format!("find ref: {e}"))?;
                local_ref_mut
                    .set_target(remote_oid, "cloud: fast-forward")
                    .map_err(|e| format!("set target: {e}"))?;
                updated = true;
            }
            Err(_) => {
                // Remote-only branch — create local
                repo.branch(branch_name, &remote_commit, false)
                    .map_err(|e| format!("create branch: {e}"))?;
                updated = true;
            }
        }
    }

    // If the current branch was fast-forwarded, reset working tree
    if updated {
        let head = repo.head().map_err(|e| format!("get HEAD: {e}"))?;
        let commit = head
            .peel_to_commit()
            .map_err(|e| format!("peel HEAD: {e}"))?;
        repo.reset(commit.as_object(), git2::ResetType::Hard, None)
            .map_err(|e| format!("reset: {e}"))?;
    }

    Ok(updated)
}
```

**Step 2: Make `parse_commit_suffix` public in `src/history/git.rs`**

Find `fn parse_commit_suffix` and change to `pub fn parse_commit_suffix`. This function is used by cloud.rs to extract metadata from commit messages.

**Step 3: Verify it compiles**

Run: `cargo build 2>&1 | head -5`

**Step 4: Commit**

```bash
git add src/history/cloud.rs src/history/git.rs
git commit -m "feat(cloud): add git2 push, fetch, divergence detection, fast-forward"
```

---

### Task 4: Add link, unlink, and divergence resolution operations

**Files:**
- Modify: `src/history/cloud.rs`

**Step 1: Add high-level cloud operations**

Append to `cloud.rs`:

```rust
// ── High-level operations ────────────────────────────────────────────────

/// Link a GitHub account: validate token, create/find repo, add remote, push.
/// Blocking — call from a background thread.
pub fn link_github(quest_dir: &Path, token: &str) -> Result<CloudConfig, String> {
    let username = github_get_username(token)?;
    let clone_url = github_ensure_repo(token, DEFAULT_REPO_NAME)?;

    let repo = Repository::open(quest_dir).map_err(|e| format!("failed to open repo: {e}"))?;

    // Remove existing cloud remote if present (re-linking).
    if repo.find_remote(REMOTE_NAME).is_ok() {
        repo.remote_delete(REMOTE_NAME)
            .map_err(|e| format!("failed to remove old remote: {e}"))?;
    }

    let auth_url = authenticated_url(&clone_url, token);
    repo.remote(REMOTE_NAME, &auth_url)
        .map_err(|e| format!("failed to add remote: {e}"))?;

    // Push all local branches to the new remote.
    push_all_branches(quest_dir, token)?;

    let config = CloudConfig {
        token: token.to_string(),
        username,
        repo_url: clone_url,
    };
    save_config(quest_dir, &config)?;
    Ok(config)
}

/// Link and pull: validate token, find existing repo, add remote, fetch, fast-forward.
/// For new machines restoring from cloud. Blocking — call from a background thread.
pub fn link_and_pull(quest_dir: &Path, token: &str) -> Result<CloudConfig, String> {
    let username = github_get_username(token)?;
    let clone_url = github_ensure_repo(token, DEFAULT_REPO_NAME)?;

    let repo = Repository::open(quest_dir).map_err(|e| format!("failed to open repo: {e}"))?;

    if repo.find_remote(REMOTE_NAME).is_ok() {
        repo.remote_delete(REMOTE_NAME)
            .map_err(|e| format!("failed to remove old remote: {e}"))?;
    }

    let auth_url = authenticated_url(&clone_url, token);
    repo.remote(REMOTE_NAME, &auth_url)
        .map_err(|e| format!("failed to add remote: {e}"))?;

    fetch_all(quest_dir, token)?;
    fast_forward_all(quest_dir)?;

    let config = CloudConfig {
        token: token.to_string(),
        username,
        repo_url: clone_url,
    };
    save_config(quest_dir, &config)?;
    Ok(config)
}

/// Remove the cloud remote and delete the config file.
pub fn unlink(quest_dir: &Path) -> Result<(), String> {
    if let Ok(repo) = Repository::open(quest_dir) {
        let _ = repo.remote_delete(REMOTE_NAME);
    }
    delete_config(quest_dir);
    Ok(())
}

/// Force-push local branch to cloud (divergence resolution: keep local).
pub fn force_push_branch(quest_dir: &Path, branch_name: &str, token: &str) -> Result<(), String> {
    let repo = Repository::open(quest_dir).map_err(|e| format!("failed to open repo: {e}"))?;
    let mut remote = repo
        .find_remote(REMOTE_NAME)
        .map_err(|e| format!("cloud remote not found: {e}"))?;

    let callbacks = make_callbacks(token.to_string());
    let mut push_opts = PushOptions::new();
    push_opts.remote_callbacks(callbacks);

    let refspec = format!("+refs/heads/{branch_name}:refs/heads/{branch_name}");
    remote
        .push(&[&refspec], Some(&mut push_opts))
        .map_err(|e| format!("force push failed: {e}"))?;
    Ok(())
}

/// Rename local branch to backup and reset to cloud version (divergence resolution: keep both).
pub fn backup_and_reset(quest_dir: &Path, branch_name: &str) -> Result<String, String> {
    let repo = Repository::open(quest_dir).map_err(|e| format!("failed to open repo: {e}"))?;

    let backup_name = format!(
        "{}-backup-{}",
        branch_name,
        chrono::Local::now().format("%Y%m%d")
    );

    // Create backup branch at current local head
    let local_ref = format!("refs/heads/{branch_name}");
    let local_ref_obj = repo
        .find_reference(&local_ref)
        .map_err(|e| format!("find local ref: {e}"))?;
    let local_commit = local_ref_obj
        .peel_to_commit()
        .map_err(|e| format!("peel to commit: {e}"))?;
    repo.branch(&backup_name, &local_commit, false)
        .map_err(|e| format!("create backup branch: {e}"))?;

    // Reset local branch to remote
    let remote_ref = format!("refs/remotes/cloud/{branch_name}");
    let remote_ref_obj = repo
        .find_reference(&remote_ref)
        .map_err(|e| format!("find remote ref: {e}"))?;
    let remote_oid = remote_ref_obj.target().ok_or("remote ref has no target")?;

    let mut local_ref_mut = repo
        .find_reference(&local_ref)
        .map_err(|e| format!("find ref for reset: {e}"))?;
    local_ref_mut
        .set_target(remote_oid, "cloud: reset to remote after backup")
        .map_err(|e| format!("set target: {e}"))?;

    // Reset working tree
    let remote_commit = repo
        .find_commit(remote_oid)
        .map_err(|e| format!("find remote commit: {e}"))?;
    repo.reset(remote_commit.as_object(), git2::ResetType::Hard, None)
        .map_err(|e| format!("reset working tree: {e}"))?;

    Ok(backup_name)
}
```

**Step 2: Verify it compiles**

Run: `cargo build 2>&1 | head -5`

**Step 3: Commit**

```bash
git add src/history/cloud.rs
git commit -m "feat(cloud): add link, unlink, force-push, backup-and-reset operations"
```

---

### Task 5: Add cloud state to TimeVaultState and BrowserMode

**Files:**
- Modify: `src/ui/time_vault_scene.rs`
- Modify: `src/input/time_vault_input.rs`

**Step 1: Add cloud-related modes to `BrowserMode` in `time_vault_scene.rs`**

Add new variants to the `BrowserMode` enum (after `NamingFork`):

```rust
    /// Typing a GitHub PAT to link the account.
    LinkingCloud,
    /// Waiting for confirmation to push to cloud.
    ConfirmPush,
    /// Waiting for confirmation to pull from cloud.
    ConfirmPull,
    /// Waiting for confirmation to unlink cloud.
    ConfirmUnlink,
    /// Divergence detected — player must choose resolution.
    DivergenceResolution,
```

**Step 2: Add cloud fields to `TimeVaultState`**

Add after `delete_confirm_input`:

```rust
    pub cloud_status: crate::history::cloud::CloudStatus,
    pub cloud_username: Option<String>,
    pub cloud_token_input: String,
    pub cloud_token_error: Option<String>,
    pub cloud_divergence: Option<crate::history::cloud::BranchDivergence>,
```

**Step 3: Initialize cloud fields in `TimeVaultState::new()`**

Add to the `Self { ... }` block:

```rust
            cloud_status: crate::history::cloud::CloudStatus::Offline,
            cloud_username: None,
            cloud_token_input: String::new(),
            cloud_token_error: None,
            cloud_divergence: None,
```

**Step 4: Add new `TimeVaultAction` variants in `time_vault_input.rs`**

Add to the `TimeVaultAction` enum:

```rust
    /// Link a GitHub account with the given PAT.
    LinkCloud { token: String },
    /// Push all branches to cloud.
    PushCloud,
    /// Pull from cloud.
    PullCloud,
    /// Unlink the GitHub account.
    UnlinkCloud,
    /// Divergence resolution: keep local saves, force-push to cloud.
    ResolveKeepLocal,
    /// Divergence resolution: use cloud saves, discard local.
    ResolveUseCloud,
    /// Divergence resolution: keep both (backup local, reset to cloud).
    ResolveKeepBoth,
```

**Step 5: Add match arms in `draw_controls` and `paint_confirm_dialog`**

In `draw_controls`, add the new modes to the early-return match:

```rust
        | BrowserMode::LinkingCloud
        | BrowserMode::ConfirmPush
        | BrowserMode::ConfirmPull
        | BrowserMode::ConfirmUnlink
        | BrowserMode::DivergenceResolution
```

In `paint_confirm_dialog`, add a catch-all at the end of the match for the new modes (placeholder — actual UI in next task):

```rust
        BrowserMode::LinkingCloud
        | BrowserMode::ConfirmPush
        | BrowserMode::ConfirmPull
        | BrowserMode::ConfirmUnlink
        | BrowserMode::DivergenceResolution => {
            // Cloud dialogs — implemented in Task 7
        }
```

**Step 6: Add match arms in `handle_time_vault_input` dispatch**

In the main match in `handle_time_vault_input`, add:

```rust
        BrowserMode::LinkingCloud => handle_link_cloud(key, state),
        BrowserMode::ConfirmPush => handle_confirm_push(key, state),
        BrowserMode::ConfirmPull => handle_confirm_pull(key, state),
        BrowserMode::ConfirmUnlink => handle_confirm_unlink(key, state),
        BrowserMode::DivergenceResolution => handle_divergence_resolution(key, state),
```

Add stub handlers that just return `TimeVaultAction::Continue` for now (implemented in Task 6):

```rust
fn handle_link_cloud(_key: KeyEvent, _state: &mut TimeVaultState) -> TimeVaultAction {
    TimeVaultAction::Continue
}
fn handle_confirm_push(_key: KeyEvent, _state: &mut TimeVaultState) -> TimeVaultAction {
    TimeVaultAction::Continue
}
fn handle_confirm_pull(_key: KeyEvent, _state: &mut TimeVaultState) -> TimeVaultAction {
    TimeVaultAction::Continue
}
fn handle_confirm_unlink(_key: KeyEvent, _state: &mut TimeVaultState) -> TimeVaultAction {
    TimeVaultAction::Continue
}
fn handle_divergence_resolution(_key: KeyEvent, _state: &mut TimeVaultState) -> TimeVaultAction {
    TimeVaultAction::Continue
}
```

**Step 7: Update `dialog_h` match for new modes**

In `paint_confirm_dialog`, add height for new modes:

```rust
        BrowserMode::LinkingCloud => 10,
        BrowserMode::ConfirmPush | BrowserMode::ConfirmPull | BrowserMode::ConfirmUnlink => 7,
        BrowserMode::DivergenceResolution => 12,
```

**Step 8: Verify it compiles**

Run: `cargo clippy --all-targets -- -D warnings 2>&1 | head -10`

**Step 9: Commit**

```bash
git add src/ui/time_vault_scene.rs src/input/time_vault_input.rs
git commit -m "feat(cloud): add cloud modes, state fields, and action variants to Time Vault"
```

---

### Task 6: Implement cloud input handlers

**Files:**
- Modify: `src/input/time_vault_input.rs`

**Step 1: Implement `handle_link_cloud`**

Replace the stub:

```rust
fn handle_link_cloud(key: KeyEvent, state: &mut TimeVaultState) -> TimeVaultAction {
    match key.code {
        KeyCode::Esc => {
            state.mode = BrowserMode::Browse;
            state.cloud_token_input.clear();
            state.cloud_token_error = None;
            TimeVaultAction::Continue
        }
        KeyCode::Backspace => {
            state.cloud_token_input.pop();
            state.cloud_token_error = None;
            TimeVaultAction::Continue
        }
        KeyCode::Enter => {
            let token = state.cloud_token_input.clone();
            if token.is_empty() {
                state.cloud_token_error = Some("token cannot be empty".to_string());
                return TimeVaultAction::Continue;
            }
            state.mode = BrowserMode::Browse;
            state.cloud_token_input.clear();
            state.cloud_token_error = None;
            TimeVaultAction::LinkCloud { token }
        }
        KeyCode::Char(c) => {
            if state.cloud_token_input.len() < 100 {
                state.cloud_token_input.push(c);
                state.cloud_token_error = None;
            }
            TimeVaultAction::Continue
        }
        _ => TimeVaultAction::Continue,
    }
}
```

**Step 2: Implement `handle_confirm_push`**

```rust
fn handle_confirm_push(key: KeyEvent, state: &mut TimeVaultState) -> TimeVaultAction {
    match key.code {
        KeyCode::Enter => {
            state.mode = BrowserMode::Browse;
            TimeVaultAction::PushCloud
        }
        KeyCode::Esc => {
            state.mode = BrowserMode::Browse;
            TimeVaultAction::Continue
        }
        _ => TimeVaultAction::Continue,
    }
}
```

**Step 3: Implement `handle_confirm_pull`**

```rust
fn handle_confirm_pull(key: KeyEvent, state: &mut TimeVaultState) -> TimeVaultAction {
    match key.code {
        KeyCode::Enter => {
            state.mode = BrowserMode::Browse;
            TimeVaultAction::PullCloud
        }
        KeyCode::Esc => {
            state.mode = BrowserMode::Browse;
            TimeVaultAction::Continue
        }
        _ => TimeVaultAction::Continue,
    }
}
```

**Step 4: Implement `handle_confirm_unlink`**

```rust
fn handle_confirm_unlink(key: KeyEvent, state: &mut TimeVaultState) -> TimeVaultAction {
    match key.code {
        KeyCode::Enter => {
            state.mode = BrowserMode::Browse;
            TimeVaultAction::UnlinkCloud
        }
        KeyCode::Esc => {
            state.mode = BrowserMode::Browse;
            TimeVaultAction::Continue
        }
        _ => TimeVaultAction::Continue,
    }
}
```

**Step 5: Implement `handle_divergence_resolution`**

```rust
fn handle_divergence_resolution(key: KeyEvent, state: &mut TimeVaultState) -> TimeVaultAction {
    match key.code {
        KeyCode::Char('k') | KeyCode::Char('K') => {
            state.mode = BrowserMode::Browse;
            state.cloud_divergence = None;
            TimeVaultAction::ResolveKeepLocal
        }
        KeyCode::Char('c') | KeyCode::Char('C') => {
            state.mode = BrowserMode::Browse;
            state.cloud_divergence = None;
            TimeVaultAction::ResolveUseCloud
        }
        KeyCode::Char('b') | KeyCode::Char('B') => {
            state.mode = BrowserMode::Browse;
            state.cloud_divergence = None;
            TimeVaultAction::ResolveKeepBoth
        }
        KeyCode::Esc => {
            state.mode = BrowserMode::Browse;
            state.cloud_divergence = None;
            TimeVaultAction::Continue
        }
        _ => TimeVaultAction::Continue,
    }
}
```

**Step 6: Add cloud keybinds to `handle_browse`**

In `handle_browse`, add cases for C, V, X keys. Add before the `_ => TimeVaultAction::Continue` catch-all:

```rust
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if state.focus == PanelFocus::Left {
                match &state.cloud_status {
                    crate::history::cloud::CloudStatus::Offline => {
                        state.mode = BrowserMode::LinkingCloud;
                        state.cloud_token_input.clear();
                        state.cloud_token_error = None;
                    }
                    crate::history::cloud::CloudStatus::Linked
                    | crate::history::cloud::CloudStatus::OutOfSync
                    | crate::history::cloud::CloudStatus::Error(_) => {
                        state.mode = BrowserMode::ConfirmPush;
                    }
                    _ => {}
                }
            }
            TimeVaultAction::Continue
        }
        KeyCode::Char('v') | KeyCode::Char('V') => {
            if state.focus == PanelFocus::Left
                && !matches!(state.cloud_status, crate::history::cloud::CloudStatus::Offline)
            {
                state.mode = BrowserMode::ConfirmPull;
            }
            TimeVaultAction::Continue
        }
        KeyCode::Char('x') | KeyCode::Char('X') => {
            if state.focus == PanelFocus::Left
                && !matches!(state.cloud_status, crate::history::cloud::CloudStatus::Offline)
            {
                state.mode = BrowserMode::ConfirmUnlink;
            }
            TimeVaultAction::Continue
        }
```

Note: The `C` key is only used when focus is on the left panel (branches). On the right panel, there's no conflict.

**Step 7: Verify it compiles**

Run: `cargo clippy --all-targets -- -D warnings 2>&1 | head -10`

**Step 8: Commit**

```bash
git add src/input/time_vault_input.rs
git commit -m "feat(cloud): implement cloud input handlers (link, push, pull, unlink, divergence)"
```

---

### Task 7: Implement cloud UI dialogs and status indicator

**Files:**
- Modify: `src/ui/time_vault_scene.rs`

**Step 1: Add cloud status indicator to `draw_time_vault`**

In `draw_time_vault`, after `paint_vault_backdrop` and before `paint_branch_panel`, add a call to paint the cloud status in the top-right corner of the buffer:

```rust
    paint_cloud_status(&mut buffer, state, buf_w);
```

Add the function:

```rust
/// Paint cloud status indicator in the top-right corner of the buffer.
fn paint_cloud_status(buffer: &mut [Vec<SceneCell>], state: &TimeVaultState, buf_w: usize) {
    use crate::history::cloud::CloudStatus;

    let (text, color) = match &state.cloud_status {
        CloudStatus::Offline => ("\u{2298} offline", Color::DarkGray),           // ⊘
        CloudStatus::Linked => {
            if let Some(username) = &state.cloud_username {
                // Can't return a reference to a local, so handle below
                let text = format!("\u{2601} {}", username);                      // ☁
                let x = buf_w.saturating_sub(text.len() + 1) as i32;
                put_text(buffer, 0, x, &text, Color::Cyan);
                return;
            }
            ("\u{2601} linked", Color::Cyan)                                     // ☁
        }
        CloudStatus::Syncing => ("\u{2601} pushing...", Color::Cyan),            // ☁
        CloudStatus::OutOfSync => ("\u{2601} \u{26a0} out of sync", Color::Yellow), // ☁ ⚠
        CloudStatus::Error(_) => ("\u{2601} \u{2717} push failed", Color::Red),  // ☁ ✗
    };
    let x = buf_w.saturating_sub(text.len() + 1) as i32;
    put_text(buffer, 0, x, text, color);
}
```

**Step 2: Add cloud controls to `draw_controls`**

In the `PanelFocus::Left` branch of the Browse mode controls, add cloud keybinds after the Delete controls and before Tab:

```rust
                    // Cloud controls
                    match &state.cloud_status {
                        crate::history::cloud::CloudStatus::Offline => {
                            spans.push(dot.clone());
                            spans.push(Span::styled("[C] ", Style::default().fg(Color::Cyan)));
                            spans.push(Span::styled("Link", Style::default().fg(Color::DarkGray)));
                        }
                        crate::history::cloud::CloudStatus::Linked
                        | crate::history::cloud::CloudStatus::OutOfSync
                        | crate::history::cloud::CloudStatus::Error(_) => {
                            spans.push(dot.clone());
                            spans.push(Span::styled("[C] ", Style::default().fg(Color::Cyan)));
                            spans.push(Span::styled("Push", Style::default().fg(Color::DarkGray)));
                            spans.push(dot.clone());
                            spans.push(Span::styled("[V] ", Style::default().fg(Color::Cyan)));
                            spans.push(Span::styled("Pull", Style::default().fg(Color::DarkGray)));
                        }
                        _ => {}
                    }
```

**Step 3: Implement cloud dialog painting in `paint_confirm_dialog`**

Replace the placeholder match arm with actual dialog rendering:

```rust
        BrowserMode::LinkingCloud => {
            put_text(buffer, cy, cx, "Link GitHub Account", Color::White);
            put_text(
                buffer,
                cy + 1,
                cx,
                "Enter a Personal Access Token:",
                Color::DarkGray,
            );

            // Mask the token (show last 4 chars)
            let masked = if state.cloud_token_input.len() > 4 {
                let stars = "\u{2022}".repeat(state.cloud_token_input.len() - 4);
                format!("{}{}_{}", stars, &state.cloud_token_input[state.cloud_token_input.len()-4..], "")
            } else {
                format!("{}_", state.cloud_token_input)
            };
            put_text(buffer, cy + 3, cx, &masked, Color::Yellow);

            let mut ctrl_row = cy + 5;
            if let Some(err) = &state.cloud_token_error {
                put_text(buffer, cy + 4, cx, err, Color::Red);
                ctrl_row = cy + 6;
            }

            put_text(buffer, ctrl_row, cx, "[Enter]", Color::Cyan);
            put_text(buffer, ctrl_row, cx + 8, "Link", Color::DarkGray);
            put_text(buffer, ctrl_row, cx + 15, "[Esc]", Color::Cyan);
            put_text(buffer, ctrl_row, cx + 21, "Cancel", Color::DarkGray);
        }
        BrowserMode::ConfirmPush => {
            put_text(buffer, cy, cx, "Push to cloud?", Color::White);
            put_text(
                buffer,
                cy + 1,
                cx,
                "All branches will be uploaded.",
                Color::DarkGray,
            );

            put_text(buffer, cy + 3, cx, "[Enter]", Color::Cyan);
            put_text(buffer, cy + 3, cx + 8, "Push", Color::DarkGray);
            put_text(buffer, cy + 3, cx + 15, "[Esc]", Color::Cyan);
            put_text(buffer, cy + 3, cx + 21, "Cancel", Color::DarkGray);
        }
        BrowserMode::ConfirmPull => {
            put_text(buffer, cy, cx, "Pull from cloud?", Color::White);
            put_text(
                buffer,
                cy + 1,
                cx,
                "Local saves will be updated.",
                Color::DarkGray,
            );

            put_text(buffer, cy + 3, cx, "[Enter]", Color::Cyan);
            put_text(buffer, cy + 3, cx + 8, "Pull", Color::DarkGray);
            put_text(buffer, cy + 3, cx + 15, "[Esc]", Color::Cyan);
            put_text(buffer, cy + 3, cx + 21, "Cancel", Color::DarkGray);
        }
        BrowserMode::ConfirmUnlink => {
            put_text(buffer, cy, cx, "Unlink GitHub?", Color::Red);
            put_text(
                buffer,
                cy + 1,
                cx,
                "Cloud saves will not be deleted.",
                Color::DarkGray,
            );

            put_text(buffer, cy + 3, cx, "[Enter]", Color::Red);
            put_text(buffer, cy + 3, cx + 8, "Unlink", Color::DarkGray);
            put_text(buffer, cy + 3, cx + 17, "[Esc]", Color::Green);
            put_text(buffer, cy + 3, cx + 23, "Cancel", Color::DarkGray);
        }
        BrowserMode::DivergenceResolution => {
            put_text(buffer, cy, cx, "Saves have diverged from cloud", Color::Yellow);

            if let Some(div) = &state.cloud_divergence {
                let local_h = div.local_playtime / 3600;
                let local_m = (div.local_playtime % 3600) / 60;
                let local_stats = format!(
                    "Local:  Lv{} \u{00b7} P{} \u{00b7} {}h {:02}m",
                    div.local_level, div.local_prestige, local_h, local_m
                );
                put_text(buffer, cy + 2, cx, &local_stats, Color::White);

                let remote_h = div.remote_playtime / 3600;
                let remote_m = (div.remote_playtime % 3600) / 60;
                let remote_stats = format!(
                    "Cloud:  Lv{} \u{00b7} P{} \u{00b7} {}h {:02}m",
                    div.remote_level, div.remote_prestige, remote_h, remote_m
                );
                put_text(buffer, cy + 3, cx, &remote_stats, Color::Cyan);
            }

            put_text(buffer, cy + 5, cx, "[K]", Color::Yellow);
            put_text(buffer, cy + 5, cx + 4, "Keep local, push to cloud", Color::DarkGray);
            put_text(buffer, cy + 6, cx, "[C]", Color::Cyan);
            put_text(buffer, cy + 6, cx + 4, "Use cloud, discard local", Color::DarkGray);
            put_text(buffer, cy + 7, cx, "[B]", Color::Green);
            put_text(buffer, cy + 7, cx + 4, "Keep both (backup local)", Color::DarkGray);
            put_text(buffer, cy + 8, cx, "[Esc]", Color::DarkGray);
            put_text(buffer, cy + 8, cx + 6, "Decide later", Color::DarkGray);
        }
```

**Step 4: Update dialog width for cloud modes**

In `paint_confirm_dialog`, update the `base_w` calculation to use wider dialog for link/divergence modes:

```rust
    let is_fork = matches!(state.mode, BrowserMode::NamingFork { .. });
    let is_cloud_wide = matches!(
        state.mode,
        BrowserMode::LinkingCloud | BrowserMode::DivergenceResolution
    );
    let base_w = if is_fork || is_cloud_wide {
        56usize
    } else {
        44usize
    };
```

**Step 5: Verify it compiles**

Run: `cargo clippy --all-targets -- -D warnings 2>&1 | head -10`

**Step 6: Commit**

```bash
git add src/ui/time_vault_scene.rs
git commit -m "feat(cloud): add cloud status indicator, controls, and dialog UI"
```

---

### Task 8: Integrate cloud operations into main loop

**Files:**
- Modify: `src/main.rs`
- Modify: `src/input/types.rs`

This is the largest integration task. It connects the cloud module to the game loop.

**Step 1: Add cloud InputResult variants to `src/input/types.rs`**

Add to the `InputResult` enum:

```rust
    LinkCloud { token: String },
    PushCloud,
    PullCloud,
    UnlinkCloud,
    ResolveKeepLocal,
    ResolveUseCloud,
    ResolveKeepBoth,
```

**Step 2: Map TimeVaultAction to InputResult in `src/input/mod.rs`**

In the Time Vault input handling section, add match arms for the new `TimeVaultAction` variants, mapping them to the corresponding `InputResult` variants. Follow the same pattern as `Restore`, `Fork`, `Switch`, `Delete`.

**Step 3: Add cloud state variables to `main.rs`**

After `let history_repo = ...` (around line 223), add:

```rust
    // Cloud sync state
    let quest_dir = dirs::home_dir()
        .map(|d| d.join(".quest"))
        .unwrap_or_default();
    let mut cloud_config = history::cloud::load_config(&quest_dir);
    let mut cloud_status = if cloud_config.is_some() {
        history::cloud::CloudStatus::Linked
    } else {
        history::cloud::CloudStatus::Offline
    };
    let mut cloud_username = cloud_config.as_ref().map(|c| c.username.clone());
    let (cloud_tx, cloud_rx) = std::sync::mpsc::channel::<history::cloud::CloudOpResult>();
    let mut cloud_op_in_flight = false;
```

**Step 4: Add auto-fetch on launch**

After loading cloud config, before the character select loop, if cloud is linked:

```rust
    // Auto-fetch from cloud on launch (blocking, before game starts)
    if let Some(ref config) = cloud_config {
        if let Err(e) = history::cloud::fetch_all(&quest_dir, &config.token) {
            eprintln!("Cloud fetch failed: {e}");
            // Non-fatal — continue offline
        } else {
            match history::cloud::check_divergence(&quest_dir) {
                Ok(Some(divergence)) => {
                    // Store divergence for resolution dialog when Time Vault opens
                    // (handled below in the game loop)
                    cloud_status = history::cloud::CloudStatus::OutOfSync;
                    // TODO: Show divergence dialog on character select or game start
                }
                Ok(None) => {
                    // No divergence — fast-forward behind branches
                    if let Ok(updated) = history::cloud::fast_forward_all(&quest_dir) {
                        if updated {
                            // Reload state from disk since files changed
                            haven = haven::load_haven();
                            enhancement = enhancement::load_enhancement();
                            global_achievements = achievements::load_achievements();
                            crate::achievements::titles::validate_selected_title(
                                &mut global_achievements,
                            );
                            global_achievements.refresh_progress();
                        }
                    }
                }
                Err(_) => {} // Non-fatal
            }
        }
    }
```

**Step 5: Add cloud result polling to the game loop**

In the main game loop, after the tick processing and before input handling, add:

```rust
    // Poll cloud sync results
    if cloud_op_in_flight {
        if let Ok(result) = cloud_rx.try_recv() {
            cloud_op_in_flight = false;
            match result {
                history::cloud::CloudOpResult::Linked(config) => {
                    cloud_username = Some(config.username.clone());
                    cloud_config = Some(config);
                    cloud_status = history::cloud::CloudStatus::Linked;
                }
                history::cloud::CloudOpResult::Pushed => {
                    cloud_status = history::cloud::CloudStatus::Linked;
                }
                history::cloud::CloudOpResult::Pulled => {
                    cloud_status = history::cloud::CloudStatus::Linked;
                    // Reload all state from disk
                    haven = haven::load_haven();
                    enhancement = enhancement::load_enhancement();
                    global_achievements = achievements::load_achievements();
                    // ... reload character if in game ...
                }
                history::cloud::CloudOpResult::Unlinked => {
                    cloud_config = None;
                    cloud_username = None;
                    cloud_status = history::cloud::CloudStatus::Offline;
                }
                history::cloud::CloudOpResult::Diverged => {
                    cloud_status = history::cloud::CloudStatus::OutOfSync;
                }
                history::cloud::CloudOpResult::Failed(msg) => {
                    cloud_status = history::cloud::CloudStatus::Error(msg);
                }
            }
        }
    }
```

**Step 6: Handle cloud InputResult actions**

In the InputResult match block (where Restore, Fork, Switch, Delete are handled), add handlers for the new cloud actions:

```rust
    InputResult::LinkCloud { token } => {
        cloud_status = history::cloud::CloudStatus::Syncing;
        cloud_op_in_flight = true;
        let tx = cloud_tx.clone();
        let dir = quest_dir.clone();
        std::thread::spawn(move || {
            let result = match history::cloud::link_github(&dir, &token) {
                Ok(config) => history::cloud::CloudOpResult::Linked(config),
                Err(e) => history::cloud::CloudOpResult::Failed(e),
            };
            let _ = tx.send(result);
        });
    }
    InputResult::PushCloud => {
        if let Some(ref config) = cloud_config {
            cloud_status = history::cloud::CloudStatus::Syncing;
            cloud_op_in_flight = true;
            let tx = cloud_tx.clone();
            let dir = quest_dir.clone();
            let token = config.token.clone();
            std::thread::spawn(move || {
                let result = match history::cloud::push_all_branches(&dir, &token) {
                    Ok(()) => history::cloud::CloudOpResult::Pushed,
                    Err(e) if e.contains("non-fast-forward") => {
                        history::cloud::CloudOpResult::Diverged
                    }
                    Err(e) => history::cloud::CloudOpResult::Failed(e),
                };
                let _ = tx.send(result);
            });
        }
    }
    InputResult::PullCloud => {
        if let Some(ref config) = cloud_config {
            cloud_status = history::cloud::CloudStatus::Syncing;
            cloud_op_in_flight = true;
            let tx = cloud_tx.clone();
            let dir = quest_dir.clone();
            let token = config.token.clone();
            std::thread::spawn(move || {
                let result = match history::cloud::fetch_all(&dir, &token) {
                    Ok(()) => {
                        match history::cloud::fast_forward_all(&dir) {
                            Ok(_) => history::cloud::CloudOpResult::Pulled,
                            Err(e) => history::cloud::CloudOpResult::Failed(e),
                        }
                    }
                    Err(e) => history::cloud::CloudOpResult::Failed(e),
                };
                let _ = tx.send(result);
            });
        }
    }
    InputResult::UnlinkCloud => {
        let _ = history::cloud::unlink(&quest_dir);
        cloud_config = None;
        cloud_username = None;
        cloud_status = history::cloud::CloudStatus::Offline;
    }
    InputResult::ResolveKeepLocal => {
        if let Some(ref config) = cloud_config {
            cloud_status = history::cloud::CloudStatus::Syncing;
            cloud_op_in_flight = true;
            let tx = cloud_tx.clone();
            let dir = quest_dir.clone();
            let token = config.token.clone();
            std::thread::spawn(move || {
                let result = match history::cloud::force_push_branch(&dir, "main", &token) {
                    Ok(()) => history::cloud::CloudOpResult::Pushed,
                    Err(e) => history::cloud::CloudOpResult::Failed(e),
                };
                let _ = tx.send(result);
            });
        }
    }
    InputResult::ResolveUseCloud => {
        if let Some(ref config) = cloud_config {
            let _ = history::cloud::fetch_all(&quest_dir, &config.token);
            let _ = history::cloud::fast_forward_all(&quest_dir);
            cloud_status = history::cloud::CloudStatus::Linked;
            // Reload all state from disk
            haven = haven::load_haven();
            enhancement = enhancement::load_enhancement();
            global_achievements = achievements::load_achievements();
            // ... reload character ...
        }
    }
    InputResult::ResolveKeepBoth => {
        let _ = history::cloud::backup_and_reset(&quest_dir, "main");
        if let Some(ref config) = cloud_config {
            cloud_status = history::cloud::CloudStatus::Linked;
            // Reload state
            haven = haven::load_haven();
            enhancement = enhancement::load_enhancement();
            global_achievements = achievements::load_achievements();
        }
    }
```

**Step 7: Add push-on-commit**

In the section where `save_all` is called with a `save_event`, after the commit succeeds, trigger a background push:

```rust
    // After save_all with a save_event, push to cloud if linked
    if save_event.is_some() && cloud_config.is_some() && !cloud_op_in_flight {
        if let Some(ref config) = cloud_config {
            cloud_op_in_flight = true;
            let tx = cloud_tx.clone();
            let dir = quest_dir.clone();
            let token = config.token.clone();
            std::thread::spawn(move || {
                let result = match history::cloud::push_all_branches(&dir, &token) {
                    Ok(()) => history::cloud::CloudOpResult::Pushed,
                    Err(e) if e.contains("non-fast-forward") => {
                        history::cloud::CloudOpResult::Diverged
                    }
                    Err(e) => history::cloud::CloudOpResult::Failed(e),
                };
                let _ = tx.send(result);
            });
        }
    }
```

**Step 8: Pass cloud status to TimeVaultState**

When creating TimeVaultState (both in main.rs and character_screens.rs), set the cloud fields:

```rust
    let mut vault = TimeVaultState::new(branches, commits);
    vault.cloud_status = cloud_status.clone();
    vault.cloud_username = cloud_username.clone();
```

**Step 9: Verify it compiles**

Run: `cargo clippy --all-targets -- -D warnings 2>&1 | head -20`

**Step 10: Commit**

```bash
git add src/main.rs src/input/types.rs src/input/mod.rs
git commit -m "feat(cloud): integrate cloud operations into main game loop"
```

---

### Task 9: Add first-launch cloud restore prompt

**Files:**
- Modify: `src/main_helpers/character_screens.rs`

**Step 1: Add first-launch detection and prompt**

In `handle_select_frame`, when there are no characters and cloud is not linked, show the restore prompt. This needs a new overlay or special handling on the character select screen.

Add a check early in the character select flow: if no characters exist and cloud is offline, show a prompt asking "Restore saves from GitHub?" with PAT input. Follow the same text input pattern as the fork naming dialog.

The exact integration depends on how the character select screen renders — adapt the existing `CharacterSelectScreen` to optionally show a cloud restore prompt when `characters.is_empty() && cloud_status == Offline`.

**Step 2: Handle the link-and-pull action**

When the user enters a PAT from the first-launch prompt, spawn a background thread calling `link_and_pull()` instead of `link_github()`, since we want to fetch rather than push.

**Step 3: Verify it compiles**

Run: `cargo clippy --all-targets -- -D warnings 2>&1 | head -10`

**Step 4: Commit**

```bash
git add src/main_helpers/character_screens.rs
git commit -m "feat(cloud): add first-launch cloud restore prompt on character select"
```

---

### Task 10: Final verification and cleanup

**Files:**
- All modified files

**Step 1: Run formatter**

Run: `cargo fmt`

**Step 2: Run clippy**

Run: `cargo clippy --all-targets -- -D warnings`
Expected: no warnings

**Step 3: Run all tests**

Run: `cargo test`
Expected: all tests pass

**Step 4: Manual testing checklist**

- [x] Launch game, open Time Vault — verify `⊘ offline` indicator
- [x] Press `[C]` on left panel — verify PAT input dialog appears
- [x] Enter invalid token — verify error message
- [x] Enter valid token — verify linking, push, and `☁ username` indicator
- [x] Make game progress (kill a boss) — verify background push happens
- [x] Press `[V]` — verify pull confirm dialog
- [x] Press `[X]` — verify unlink confirm dialog
- [x] Close and reopen game — verify auto-fetch on launch
- [x] Test on second machine — verify saves sync across

**Step 5: Final commit if any cleanup needed**

```bash
git add -A
git commit -m "chore(cloud): final cleanup and formatting"
```
