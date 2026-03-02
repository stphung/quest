//! Cloud sync state and operations extracted from main.rs.
//!
//! Bundles all cloud-related variables and provides helper functions for polling
//! cloud operation results and dispatching cloud input actions.

use crate::character::manager::CharacterManager;
use crate::enhancement;
use crate::history;
use crate::history::cloud::{CloudConfig, CloudOpResult, CloudStatus};
use crate::input::{GameOverlay, InputResult};
use chrono::Utc;

/// All cloud sync runtime state, previously scattered as locals in main.rs.
pub struct CloudSyncState {
    pub config: Option<CloudConfig>,
    pub status: CloudStatus,
    pub username: Option<String>,
    pub op_in_flight: bool,
    pub tx: std::sync::mpsc::Sender<CloudOpResult>,
    pub rx: std::sync::mpsc::Receiver<CloudOpResult>,
}

impl CloudSyncState {
    /// Initialize cloud sync state from disk.
    pub fn new(quest_dir: &std::path::Path) -> Self {
        let config = history::cloud::load_config(quest_dir);
        let status = if config.is_some() {
            CloudStatus::Linked
        } else {
            CloudStatus::Offline
        };
        let username = config.as_ref().map(|c| c.username.clone());
        let (tx, rx) = std::sync::mpsc::channel::<CloudOpResult>();
        Self {
            config,
            status,
            username,
            op_in_flight: false,
            tx,
            rx,
        }
    }
}

/// Return value from `poll_cloud_result` when a cloud operation completes.
pub struct CloudActionResult {
    /// Reloaded game state when character data was refreshed from disk.
    pub reloaded_state: Option<crate::core::game_state::GameState>,
    /// True when account-level state (haven/enhancement/achievements) was reloaded.
    pub account_state_reloaded: bool,
    /// True when `last_save_time` should be updated to now.
    pub needs_save_timestamp: bool,
}

/// Poll the cloud result channel and apply state transitions.
///
/// Call once per frame when `cloud.op_in_flight` is true. Returns `Some` when an
/// operation completed (whether successfully or not). The caller is responsible for
/// applying `CloudActionResult` fields to the surrounding game state.
pub fn poll_cloud_result(
    cloud: &mut CloudSyncState,
    overlay: &mut GameOverlay,
    quest_dir: &std::path::Path,
    character_manager: &CharacterManager,
    character_name: &str,
    enhancement: &enhancement::EnhancementProgress,
) -> Option<CloudActionResult> {
    if !cloud.op_in_flight {
        return None;
    }
    let result = cloud.rx.try_recv().ok()?;
    cloud.op_in_flight = false;

    let mut action_result = CloudActionResult {
        reloaded_state: None,
        account_state_reloaded: false,
        needs_save_timestamp: false,
    };

    match result {
        CloudOpResult::TokenValidated {
            username,
            token,
            repos,
        } => {
            cloud.status = if cloud.config.is_some() {
                CloudStatus::Linked
            } else {
                CloudStatus::Offline
            };
            if let GameOverlay::TimeVault { ref mut browser } = overlay {
                browser.cloud_validated_token = Some(token);
                browser.cloud_repos = repos;
                browser.cloud_repo_selected = 0;
                browser.cloud_repo_input.clear();
                browser.cloud_username = Some(username);
                browser.cloud_current_repo = cloud
                    .config
                    .as_ref()
                    .map(|c| history::cloud::repo_name_from_url(&c.repo_url));
                browser.mode = crate::ui::time_vault_scene::BrowserMode::SelectingRepo;
            }
        }
        CloudOpResult::Linked(config) => {
            cloud.username = Some(config.username.clone());
            cloud.config = Some(config);
            match history::cloud::check_divergence(quest_dir) {
                Ok(Some(_div)) => {
                    cloud.status = CloudStatus::OutOfSync;
                    if let GameOverlay::TimeVault { ref mut browser } = overlay {
                        browser.cloud_divergence = Some(_div);
                    }
                }
                _ => {
                    cloud.status = CloudStatus::Linked;
                    // Auto-push local saves on first link
                    if let Some(ref config) = cloud.config {
                        super::cloud_ops::spawn_cloud_push(
                            &mut cloud.op_in_flight,
                            &cloud.tx,
                            quest_dir,
                            &config.token,
                        );
                        cloud.status = CloudStatus::Syncing;
                    }
                }
            }
        }
        CloudOpResult::Pushed => {
            cloud.status = CloudStatus::Linked;
        }
        CloudOpResult::Pulled => {
            cloud.status = CloudStatus::Linked;
            action_result.account_state_reloaded = true;
            action_result.reloaded_state =
                reload_character(character_manager, character_name, enhancement);
            action_result.needs_save_timestamp = true;
        }
        CloudOpResult::Unlinked => {
            cloud.config = None;
            cloud.username = None;
            cloud.status = CloudStatus::Offline;
        }
        CloudOpResult::Diverged(div) => {
            cloud.status = CloudStatus::OutOfSync;
            if let GameOverlay::TimeVault { ref mut browser } = overlay {
                browser.cloud_divergence = Some(div);
                browser.mode = crate::ui::time_vault_scene::BrowserMode::DivergenceResolution;
            }
        }
        CloudOpResult::TokenUpdated(new_config) => {
            cloud.status = CloudStatus::Linked;
            cloud.username = Some(new_config.username.clone());
            cloud.config = Some(new_config);
        }
        CloudOpResult::Failed(msg) => {
            if history::cloud::is_auth_error(&msg) {
                cloud.status = CloudStatus::TokenExpired;
            } else {
                cloud.status = CloudStatus::Error(msg);
            }
        }
    }

    // Update Time Vault overlay if open
    if let GameOverlay::TimeVault { ref mut browser } = overlay {
        browser.cloud_status = cloud.status.clone();
        browser.cloud_username = cloud.username.clone();
        browser.cloud_current_repo = cloud
            .config
            .as_ref()
            .map(|c| history::cloud::repo_name_from_url(&c.repo_url));
    }

    Some(action_result)
}

/// Dispatch async (non-blocking) cloud `InputResult` variants.
///
/// Handles: `ValidateToken`, `ChangeRepo`, `LinkCloud`, `PushCloud`, `PullCloud`,
/// `UnlinkCloud`, `UpdateToken`.
///
/// Returns `true` if the result was handled (caller should `continue` the event loop),
/// `false` if the result is not a cloud action.
pub fn dispatch_cloud_action(
    result: &InputResult,
    cloud: &mut CloudSyncState,
    overlay: &mut GameOverlay,
    quest_dir: &std::path::Path,
) -> bool {
    match result {
        InputResult::ValidateToken { token } => {
            if !cloud.op_in_flight {
                cloud.op_in_flight = true;
                let tx = cloud.tx.clone();
                let tok = token.clone();
                std::thread::spawn(move || {
                    let tx2 = tx.clone();
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        match history::cloud::github_get_username(&tok) {
                            Ok(username) => {
                                let repos =
                                    history::cloud::github_list_repos(&tok).unwrap_or_default();
                                let _ = tx.send(CloudOpResult::TokenValidated {
                                    username,
                                    token: tok,
                                    repos,
                                });
                            }
                            Err(e) => {
                                let _ = tx.send(CloudOpResult::Failed(e));
                            }
                        }
                    }));
                    if result.is_err() {
                        let _ = tx2.send(CloudOpResult::Failed(
                            "Cloud operation failed unexpectedly".to_string(),
                        ));
                    }
                });
                if let GameOverlay::TimeVault { ref mut browser } = overlay {
                    browser.cloud_status = CloudStatus::Syncing;
                }
            }
            true
        }
        InputResult::ChangeRepo => {
            if !cloud.op_in_flight {
                if let Some(ref config) = cloud.config {
                    cloud.op_in_flight = true;
                    let tx = cloud.tx.clone();
                    let tok = config.token.clone();
                    std::thread::spawn(move || {
                        let tx2 = tx.clone();
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            match history::cloud::github_get_username(&tok) {
                                Ok(username) => {
                                    let repos =
                                        history::cloud::github_list_repos(&tok).unwrap_or_default();
                                    let _ = tx.send(CloudOpResult::TokenValidated {
                                        username,
                                        token: tok,
                                        repos,
                                    });
                                }
                                Err(e) => {
                                    let _ = tx.send(CloudOpResult::Failed(e));
                                }
                            }
                        }));
                        if result.is_err() {
                            let _ = tx2.send(CloudOpResult::Failed(
                                "Cloud operation failed unexpectedly".to_string(),
                            ));
                        }
                    });
                    if let GameOverlay::TimeVault { ref mut browser } = overlay {
                        browser.cloud_status = CloudStatus::Syncing;
                    }
                }
            }
            true
        }
        InputResult::LinkCloud {
            token,
            repo_name,
            private,
        } => {
            if !cloud.op_in_flight {
                cloud.status = CloudStatus::Syncing;
                cloud.op_in_flight = true;
                let tx = cloud.tx.clone();
                let dir = quest_dir.to_path_buf();
                let tok = token.clone();
                let rname = repo_name.clone();
                let priv_ = *private;
                std::thread::spawn(move || {
                    let tx2 = tx.clone();
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        let res = match history::cloud::link_github(&dir, &tok, &rname, priv_) {
                            Ok(config) => CloudOpResult::Linked(config),
                            Err(e) => CloudOpResult::Failed(e),
                        };
                        let _ = tx.send(res);
                    }));
                    if result.is_err() {
                        let _ = tx2.send(CloudOpResult::Failed(
                            "Cloud operation failed unexpectedly".to_string(),
                        ));
                    }
                });
                if let GameOverlay::TimeVault { ref mut browser } = overlay {
                    browser.cloud_status = cloud.status.clone();
                }
            }
            true
        }
        InputResult::PushCloud => {
            if !cloud.op_in_flight {
                if let Some(ref config) = cloud.config {
                    cloud.status = CloudStatus::Syncing;
                    super::cloud_ops::spawn_cloud_push(
                        &mut cloud.op_in_flight,
                        &cloud.tx,
                        quest_dir,
                        &config.token,
                    );
                    if let GameOverlay::TimeVault { ref mut browser } = overlay {
                        browser.cloud_status = cloud.status.clone();
                    }
                }
            }
            true
        }
        InputResult::PullCloud => {
            if !cloud.op_in_flight {
                if let Some(ref config) = cloud.config {
                    cloud.status = CloudStatus::Syncing;
                    cloud.op_in_flight = true;
                    let tx = cloud.tx.clone();
                    let dir = quest_dir.to_path_buf();
                    let tok = config.token.clone();
                    std::thread::spawn(move || {
                        let tx2 = tx.clone();
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            let res = (|| -> CloudOpResult {
                                if let Err(e) = history::cloud::fetch_all(&dir, &tok) {
                                    return CloudOpResult::Failed(e);
                                }
                                match history::cloud::check_divergence(&dir) {
                                    Ok(Some(div)) => CloudOpResult::Diverged(div),
                                    Ok(None) => match history::cloud::fast_forward_all(&dir) {
                                        Ok(_) => CloudOpResult::Pulled,
                                        Err(e) => CloudOpResult::Failed(e),
                                    },
                                    Err(e) => CloudOpResult::Failed(e),
                                }
                            })();
                            let _ = tx.send(res);
                        }));
                        if result.is_err() {
                            let _ = tx2.send(CloudOpResult::Failed(
                                "Cloud operation failed unexpectedly".to_string(),
                            ));
                        }
                    });
                    if let GameOverlay::TimeVault { ref mut browser } = overlay {
                        browser.cloud_status = cloud.status.clone();
                    }
                }
            }
            true
        }
        InputResult::UnlinkCloud => {
            let _ = history::cloud::unlink(quest_dir);
            cloud.config = None;
            cloud.username = None;
            cloud.status = CloudStatus::Offline;
            if let GameOverlay::TimeVault { ref mut browser } = overlay {
                browser.cloud_status = cloud.status.clone();
                browser.cloud_username = None;
            }
            true
        }
        InputResult::UpdateToken { token } => {
            if !cloud.op_in_flight {
                if let Some(ref config) = cloud.config {
                    cloud.op_in_flight = true;
                    cloud.status = CloudStatus::Syncing;
                    if let GameOverlay::TimeVault { ref mut browser } = overlay {
                        browser.cloud_status = cloud.status.clone();
                    }
                    let quest = quest_dir.to_path_buf();
                    let cfg = config.clone();
                    let tx = cloud.tx.clone();
                    let tok = token.clone();
                    std::thread::spawn(move || {
                        let tx2 = tx.clone();
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            let op_result = match history::cloud::update_token(&quest, &tok, &cfg) {
                                Ok(new_config) => CloudOpResult::TokenUpdated(new_config),
                                Err(e) => {
                                    if history::cloud::is_auth_error(&e) {
                                        CloudOpResult::Failed("invalid token".to_string())
                                    } else {
                                        CloudOpResult::Failed(e)
                                    }
                                }
                            };
                            let _ = tx.send(op_result);
                        }));
                        if result.is_err() {
                            let _ = tx2.send(CloudOpResult::Failed(
                                "Cloud operation failed unexpectedly".to_string(),
                            ));
                        }
                    });
                }
            }
            true
        }
        // ResolveKeepLocal is async (spawns a thread), dispatch it here
        InputResult::ResolveKeepLocal => {
            if !cloud.op_in_flight {
                if let Some(ref config) = cloud.config {
                    cloud.status = CloudStatus::Syncing;
                    cloud.op_in_flight = true;
                    let tx = cloud.tx.clone();
                    let dir = quest_dir.to_path_buf();
                    let tok = config.token.clone();
                    std::thread::spawn(move || {
                        let tx2 = tx.clone();
                        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            let res = match history::cloud::force_push_branch(&dir, "main", &tok) {
                                Ok(()) => CloudOpResult::Pushed,
                                Err(e) => CloudOpResult::Failed(e),
                            };
                            let _ = tx.send(res);
                        }));
                        if result.is_err() {
                            let _ = tx2.send(CloudOpResult::Failed(
                                "Cloud operation failed unexpectedly".to_string(),
                            ));
                        }
                    });
                    if let GameOverlay::TimeVault { ref mut browser } = overlay {
                        browser.cloud_status = cloud.status.clone();
                    }
                }
            }
            true
        }
        _ => false,
    }
}

/// Handle blocking resolve variants that need character reload.
///
/// Handles: `ResolveUseCloud`, `ResolveKeepBoth`.
///
/// Returns `Some(CloudActionResult)` if the result was handled, `None` otherwise.
#[allow(clippy::too_many_arguments)]
pub fn apply_cloud_resolve(
    result: &InputResult,
    cloud: &mut CloudSyncState,
    overlay: &mut GameOverlay,
    quest_dir: &std::path::Path,
    character_manager: &CharacterManager,
    character_name: &str,
    enhancement: &enhancement::EnhancementProgress,
    history_repo: Option<&history::HistoryRepo>,
) -> Option<CloudActionResult> {
    match result {
        InputResult::ResolveUseCloud => {
            if let Some(ref config) = cloud.config {
                let _ = history::cloud::fetch_all(quest_dir, &config.token);
                let _ = history::cloud::reset_to_remote(quest_dir, "main");

                let action_result = CloudActionResult {
                    reloaded_state: reload_character(
                        character_manager,
                        character_name,
                        enhancement,
                    ),
                    account_state_reloaded: true,
                    needs_save_timestamp: true,
                };

                cloud.status = CloudStatus::Linked;
                if let GameOverlay::TimeVault { ref mut browser } = overlay {
                    browser.cloud_status = cloud.status.clone();
                    refresh_browser_branches(browser, history_repo);
                }
                Some(action_result)
            } else {
                None
            }
        }
        InputResult::ResolveKeepBoth => {
            let _ = history::cloud::backup_and_reset(quest_dir, "main");

            let action_result = CloudActionResult {
                reloaded_state: reload_character(character_manager, character_name, enhancement),
                account_state_reloaded: true,
                needs_save_timestamp: true,
            };

            cloud.status = CloudStatus::Linked;
            if let GameOverlay::TimeVault { ref mut browser } = overlay {
                browser.cloud_status = cloud.status.clone();
                browser.cloud_username = cloud.username.clone();
                refresh_browser_branches(browser, history_repo);
            }
            Some(action_result)
        }
        _ => None,
    }
}

/// Reload the active character from disk and recalculate stats.
fn reload_character(
    character_manager: &CharacterManager,
    character_name: &str,
    enhancement: &enhancement::EnhancementProgress,
) -> Option<crate::core::game_state::GameState> {
    let filename = format!("{}.json", character_name);
    if let Ok(mut reloaded) = character_manager.load_character(&filename) {
        reloaded.recalculate_derived_stats(&enhancement.levels);
        reloaded.recalculate_prestige_bonuses();
        reloaded.last_save_time = Utc::now().timestamp();
        Some(reloaded)
    } else {
        None
    }
}

/// Refresh branch list in a TimeVault browser after a cloud resolve.
fn refresh_browser_branches(
    browser: &mut crate::ui::time_vault_scene::TimeVaultState,
    history_repo: Option<&history::HistoryRepo>,
) {
    if let Some(repo) = history_repo {
        if let Ok(branches) = repo.list_branches() {
            browser.branches = branches;
            if browser.selected_branch >= browser.branches.len() {
                browser.selected_branch = browser.branches.len().saturating_sub(1);
            }
            if let Some(b) = browser.branches.get(browser.selected_branch) {
                browser.commits = repo.list_commits(&b.name).unwrap_or_default();
                browser.selected_commit = 0;
            }
        }
    }
}
