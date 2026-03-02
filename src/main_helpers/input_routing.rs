//! Game input result routing extracted from main.rs game loop.

use chrono::{Local, Utc};
use std::path::Path;
use std::time::Instant;

use crate::achievements::Achievements;
use crate::character::manager::CharacterManager;
use crate::core::game_state::GameState;
use crate::deep::DeepState;
use crate::enhancement::EnhancementProgress;
use crate::haven::Haven;
use crate::history::{self, HistoryRepo, SaveEvent};
use crate::input::{GameOverlay, InputResult};
use crate::main_helpers::cloud_sync::CloudSyncState;
use crate::main_helpers::game_context::GameContext;
use crate::stormglass::types::ChronoSurgeState;

use super::persistence::{commit_save, save_all, save_files};

/// What the game loop should do after processing an input result.
pub enum InputAction {
    /// Continue the game loop normally.
    Continue,
    /// Save completed; commit deferred to next tick for sequential indicator.
    DeferCommit(SaveEvent),
    /// Player quit to character select — break the game loop.
    QuitToSelect,
}

/// Process the result of `handle_game_input()` and perform side effects
/// (saving, toggling update details).
///
/// Returns an [`InputAction`] telling the caller whether to continue or
/// exit the game loop.
pub fn route_game_input(
    result: InputResult,
    ctx: &GameContext<'_>,
    character_manager: &CharacterManager,
    last_save_instant: &mut Option<Instant>,
    last_save_time: &mut Option<chrono::DateTime<Local>>,
) -> InputAction {
    let state = &*ctx.state;
    let global_achievements = &*ctx.achievements;
    let haven = &*ctx.haven;
    let enhancement = &*ctx.enhancement;
    let deep = &*ctx.deep_state;
    let debug_mode = ctx.debug_mode;

    match result {
        InputResult::Continue => InputAction::Continue,
        InputResult::QuitToSelect => {
            if !debug_mode {
                save_all(
                    character_manager,
                    state,
                    global_achievements,
                    haven,
                    enhancement,
                    deep,
                    None,
                    None,
                );
            }
            InputAction::QuitToSelect
        }
        InputResult::NeedsSave | InputResult::NeedsSaveAll => {
            if !debug_mode {
                save_files(
                    character_manager,
                    state,
                    global_achievements,
                    haven,
                    enhancement,
                    deep,
                );
                *last_save_instant = Some(Instant::now());
                *last_save_time = Some(Local::now());
            }
            InputAction::Continue
        }
        InputResult::NeedsSaveWithEvent(event) | InputResult::NeedsSaveAllWithEvent(event) => {
            if !debug_mode {
                save_files(
                    character_manager,
                    state,
                    global_achievements,
                    haven,
                    enhancement,
                    deep,
                );
                *last_save_instant = Some(Instant::now());
                *last_save_time = Some(Local::now());
                InputAction::DeferCommit(event)
            } else {
                InputAction::Continue
            }
        }
        // StartChronoSurge is handled by dispatch_chrono_surge() before reaching
        // route_game_input, but must be matched for exhaustiveness.
        InputResult::StartChronoSurge { .. } => InputAction::Continue,
        // Time Vault actions are handled by dispatch_time_vault_action() before
        // reaching route_game_input, but must be matched for exhaustiveness.
        InputResult::OpenTimeVault
        | InputResult::RestoreSave { .. }
        | InputResult::RefreshSaveHistoryCommits { .. }
        | InputResult::ForkSave { .. }
        | InputResult::SwitchSaveBranch { .. }
        | InputResult::DeleteSaveBranch { .. } => InputAction::Continue,
        // Cloud actions are handled by dispatch_cloud_action() / apply_cloud_resolve()
        // before reaching route_game_input, but must be matched for exhaustiveness.
        InputResult::ValidateToken { .. }
        | InputResult::ChangeRepo
        | InputResult::LinkCloud { .. }
        | InputResult::PushCloud
        | InputResult::PullCloud
        | InputResult::UnlinkCloud
        | InputResult::ResolveKeepLocal
        | InputResult::ResolveUseCloud
        | InputResult::ResolveKeepBoth
        | InputResult::UpdateToken { .. } => InputAction::Continue,
    }
}

// ---------------------------------------------------------------------------
// Chrono Surge dispatch
// ---------------------------------------------------------------------------

/// Handle a `StartChronoSurge` input result by computing the overcharge proc
/// and creating a [`ChronoSurgeState`].
///
/// Returns `Some(ChronoSurgeState)` if `result` is `StartChronoSurge`,
/// `None` otherwise.  The caller must set `state.chrono_surge_active = true`
/// and store the returned surge state.
pub fn dispatch_chrono_surge(
    result: &InputResult,
    state: &mut GameState,
) -> Option<ChronoSurgeState> {
    let InputResult::StartChronoSurge { ticks } = *result else {
        return None;
    };

    // Roll for overcharge proc from sigil bonus
    let overcharge_chance = crate::stormglass::sigils::SigilBonuses::compute(&state.storm_sigils)
        .chrono_overcharge_percent;
    let mut rng = rand::rng();
    let overcharged = if state.debug_force_overcharge {
        state.debug_force_overcharge = false;
        true
    } else {
        use rand::RngExt;
        overcharge_chance > 0.0 && rng.random::<f64>() * 100.0 < overcharge_chance
    };
    let actual_ticks = if overcharged {
        (ticks as f64 * 1.5) as u64
    } else {
        ticks
    };

    Some(ChronoSurgeState::new(actual_ticks, overcharged))
}

// ---------------------------------------------------------------------------
// Time Vault dispatch
// ---------------------------------------------------------------------------

/// Outcome of [`dispatch_time_vault_action`].
pub enum TimeVaultAction {
    /// The input result was not a Time Vault action.
    NotHandled,
    /// The action was handled; no further processing needed.
    Handled,
    /// Game state was reloaded from disk (restore / fork / switch).
    /// The caller must replace its `state` with the contained value.
    StateReloaded(Box<GameState>),
}

/// Dispatch Time Vault `InputResult` variants.
///
/// Handles: `OpenTimeVault`, `RefreshSaveHistoryCommits`, `RestoreSave`,
/// `ForkSave`, `SwitchSaveBranch`, `DeleteSaveBranch`.
///
/// Returns a [`TimeVaultAction`] telling the caller what happened.
#[allow(clippy::too_many_arguments)]
pub fn dispatch_time_vault_action(
    result: &InputResult,
    state: &GameState,
    overlay: &mut GameOverlay,
    history_repo: Option<&HistoryRepo>,
    cloud: &mut CloudSyncState,
    quest_dir: &Path,
    character_manager: &CharacterManager,
    global_achievements: &Achievements,
    haven: &Haven,
    enhancement: &EnhancementProgress,
    deep: &DeepState,
    debug_mode: bool,
    last_save_instant: &mut Option<Instant>,
    last_save_time: &mut Option<chrono::DateTime<Local>>,
    last_commit_instant: &mut Option<Instant>,
    last_commit_time: &mut Option<chrono::DateTime<Local>>,
) -> TimeVaultAction {
    match result {
        InputResult::OpenTimeVault => {
            if let Some(repo) = history_repo {
                if let Ok(branches) = repo.list_branches() {
                    let commits = branches
                        .first()
                        .and_then(|b| repo.list_commits(&b.name).ok())
                        .unwrap_or_default();
                    let mut vault_state =
                        crate::ui::time_vault_scene::TimeVaultState::new(branches, commits);
                    vault_state.cloud_status = cloud.status.clone();
                    vault_state.cloud_username = cloud.username.clone();
                    vault_state.cloud_current_repo = cloud
                        .config
                        .as_ref()
                        .map(|c| history::cloud::repo_name_from_url(&c.repo_url));
                    // If already out-of-sync, re-check divergence and show resolution dialog
                    if matches!(cloud.status, history::cloud::CloudStatus::OutOfSync) {
                        if let Ok(Some(div)) = history::cloud::check_divergence(quest_dir) {
                            vault_state.cloud_divergence = Some(div);
                            vault_state.mode =
                                crate::ui::time_vault_scene::BrowserMode::DivergenceResolution;
                        }
                    }
                    *overlay = GameOverlay::TimeVault {
                        browser: Box::new(vault_state),
                    };
                }
            }
            TimeVaultAction::Handled
        }

        InputResult::RefreshSaveHistoryCommits { ref branch_name } => {
            if let Some(repo) = history_repo {
                if let Ok(commits) = repo.list_commits(branch_name) {
                    if let GameOverlay::TimeVault { ref mut browser } = overlay {
                        browser.commits = commits;
                    }
                }
            }
            TimeVaultAction::Handled
        }

        InputResult::RestoreSave { ref commit_id } => {
            if let Some(repo) = history_repo {
                save_and_commit(
                    state,
                    character_manager,
                    global_achievements,
                    haven,
                    enhancement,
                    deep,
                    repo,
                    cloud,
                    quest_dir,
                    last_save_time,
                    last_commit_instant,
                    last_commit_time,
                );
                if repo.restore_to(commit_id).is_ok() {
                    let reloaded = reload_game_state(
                        state,
                        character_manager,
                        enhancement,
                        "\u{23F3} Save restored",
                    );
                    refresh_vault_browser(overlay, repo, None);
                    if !debug_mode {
                        *last_save_instant = Some(Instant::now());
                        *last_save_time = Some(Local::now());
                    }
                    if let Some(reloaded) = reloaded {
                        return TimeVaultAction::StateReloaded(Box::new(reloaded));
                    }
                }
            }
            TimeVaultAction::Handled
        }

        InputResult::ForkSave {
            ref commit_id,
            ref branch_name,
        } => {
            if let Some(repo) = history_repo {
                save_and_commit(
                    state,
                    character_manager,
                    global_achievements,
                    haven,
                    enhancement,
                    deep,
                    repo,
                    cloud,
                    quest_dir,
                    last_save_time,
                    last_commit_instant,
                    last_commit_time,
                );
                if repo.fork_timeline(branch_name, commit_id).is_ok() {
                    let reloaded = reload_game_state(
                        state,
                        character_manager,
                        enhancement,
                        &format!("\u{1F500} Timeline branched: {branch_name}"),
                    );
                    refresh_vault_browser(overlay, repo, None);
                    if !debug_mode {
                        *last_save_instant = Some(Instant::now());
                        *last_save_time = Some(Local::now());
                    }
                    if let Some(reloaded) = reloaded {
                        return TimeVaultAction::StateReloaded(Box::new(reloaded));
                    }
                }
            }
            TimeVaultAction::Handled
        }

        InputResult::SwitchSaveBranch { ref branch_name } => {
            if let Some(repo) = history_repo {
                save_and_commit(
                    state,
                    character_manager,
                    global_achievements,
                    haven,
                    enhancement,
                    deep,
                    repo,
                    cloud,
                    quest_dir,
                    last_save_time,
                    last_commit_instant,
                    last_commit_time,
                );
                if repo.switch_timeline(branch_name).is_ok() {
                    let reloaded = reload_game_state(
                        state,
                        character_manager,
                        enhancement,
                        &format!("\u{1F500} Timeline switched: {branch_name}"),
                    );
                    refresh_vault_browser(overlay, repo, Some(branch_name));
                    if !debug_mode {
                        *last_save_instant = Some(Instant::now());
                        *last_save_time = Some(Local::now());
                    }
                    if let Some(reloaded) = reloaded {
                        return TimeVaultAction::StateReloaded(Box::new(reloaded));
                    }
                }
            }
            TimeVaultAction::Handled
        }

        InputResult::DeleteSaveBranch { ref branch_name } => {
            if let Some(repo) = history_repo {
                if repo.delete_timeline(branch_name).is_ok() {
                    refresh_vault_browser(overlay, repo, None);
                }
            }
            TimeVaultAction::Handled
        }

        _ => TimeVaultAction::NotHandled,
    }
}

/// Save current state to disk, create an AutoSave commit, and optionally
/// push to cloud. Shared by RestoreSave, ForkSave, and SwitchSaveBranch.
#[allow(clippy::too_many_arguments)]
fn save_and_commit(
    state: &GameState,
    character_manager: &CharacterManager,
    global_achievements: &Achievements,
    haven: &Haven,
    enhancement: &EnhancementProgress,
    deep: &DeepState,
    repo: &HistoryRepo,
    cloud: &mut CloudSyncState,
    quest_dir: &Path,
    last_save_time: &mut Option<chrono::DateTime<Local>>,
    last_commit_instant: &mut Option<Instant>,
    last_commit_time: &mut Option<chrono::DateTime<Local>>,
) {
    save_files(
        character_manager,
        state,
        global_achievements,
        haven,
        enhancement,
        deep,
    );
    *last_save_time = Some(Local::now());
    if commit_save(state, &SaveEvent::AutoSave, repo) {
        *last_commit_instant = Some(Instant::now());
        *last_commit_time = Some(Local::now());
        if !cloud.op_in_flight {
            if let Some(ref config) = cloud.config {
                super::cloud_ops::spawn_cloud_push(
                    &mut cloud.op_in_flight,
                    &cloud.tx,
                    quest_dir,
                    &config.token,
                );
            }
        }
    }
}

/// Reload account state and character from disk after a Time Vault operation
/// (restore, fork, or switch). Returns the reloaded `GameState` if successful.
fn reload_game_state(
    state: &GameState,
    character_manager: &CharacterManager,
    enhancement: &EnhancementProgress,
    log_message: &str,
) -> Option<GameState> {
    // Reload account-level state (haven, enhancement, achievements)
    // Note: the caller is responsible for writing these back to their
    // mutable bindings since we return the reloaded character state only.
    // Account state reload happens in main.rs after receiving StateReloaded.

    let filename = format!("{}.json", state.character_name);
    if let Ok(mut reloaded) = character_manager.load_character(&filename) {
        reloaded.recalculate_derived_stats(&enhancement.levels);
        reloaded.recalculate_prestige_bonuses();
        reloaded
            .combat_state
            .add_log_entry(log_message.to_string(), false, true);
        // Suppress offline XP on the reloaded save
        reloaded.last_save_time = Utc::now().timestamp();
        Some(reloaded)
    } else {
        None
    }
}

/// Refresh the Time Vault browser's branch and commit lists after a repo
/// operation. If `select_branch_name` is `Some`, the browser selects that
/// branch; otherwise it clamps the existing selection.
fn refresh_vault_browser(
    overlay: &mut GameOverlay,
    repo: &HistoryRepo,
    select_branch_name: Option<&str>,
) {
    if let GameOverlay::TimeVault { ref mut browser } = overlay {
        if let Ok(branches) = repo.list_branches() {
            if let Some(name) = select_branch_name {
                // Select the specific branch (sort order may have changed)
                browser.selected_branch =
                    branches.iter().position(|b| b.name == *name).unwrap_or(0);
            } else {
                // Clamp selection to valid range
                if browser.selected_branch >= branches.len() {
                    browser.selected_branch = branches.len().saturating_sub(1);
                }
            }
            browser.branches = branches;
            if let Some(b) = browser.branches.get(browser.selected_branch) {
                browser.commits = repo.list_commits(&b.name).unwrap_or_default();
                browser.selected_commit = 0;
            }
        }
    }
}
