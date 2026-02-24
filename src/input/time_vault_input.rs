//! Input handling for the Time Vault overlay.

use crate::history::validate_branch_name;
use crate::ui::time_vault_scene::{BrowserMode, ForkSource, PanelFocus, TimeVaultState};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

/// Actions that the Time Vault can request from the main loop.
pub enum TimeVaultAction {
    /// Close the overlay, no action taken.
    Close,
    /// Continue processing (state was mutated in place).
    Continue,
    /// Restore to a specific commit by its short SHA.
    Restore { commit_id: String },
    /// Switch to a different branch and reload commits.
    RefreshCommits { branch_name: String },
    /// Fork a new branch at the given commit.
    Fork {
        commit_id: String,
        branch_name: String,
    },
    /// Switch the active branch.
    SwitchBranch { branch_name: String },
    /// Delete a branch by name.
    DeleteBranch { branch_name: String },
    /// Validate a PAT and fetch the user's repos for selection.
    #[allow(dead_code)]
    ValidateToken { token: String },
    /// Link a GitHub account with the given PAT and repo name.
    #[allow(dead_code)]
    LinkCloud { token: String, repo_name: String },
    /// Push all branches to cloud.
    PushCloud,
    /// Pull from cloud.
    PullCloud,
    /// Change the linked cloud repo (re-use existing PAT).
    ChangeRepo,
    /// Unlink the GitHub account.
    UnlinkCloud,
    /// Divergence resolution: keep local saves, force-push to cloud.
    ResolveKeepLocal,
    /// Divergence resolution: use cloud saves, discard local.
    ResolveUseCloud,
    /// Divergence resolution: keep both (backup local, reset to cloud).
    ResolveKeepBoth,
}

/// Handle keyboard input for the Time Vault overlay.
///
/// Dispatches by the current `BrowserMode` (state machine):
/// - **Browse**: Tab toggles focus, Up/Down navigates, Enter context-sensitive,
///   F starts fork (right panel), D starts delete (left panel).
/// - **ConfirmRestore**: Enter confirms, Esc cancels.
/// - **ConfirmDelete**: Enter confirms, Esc cancels.
/// - **NamingFork**: Character input, Backspace, Enter validates, Esc cancels.
pub fn handle_time_vault_input(key: KeyEvent, state: &mut TimeVaultState) -> TimeVaultAction {
    match &state.mode {
        BrowserMode::Browse => handle_browse(key, state),
        BrowserMode::ConfirmRestore => handle_confirm_restore(key, state),
        BrowserMode::ConfirmSwitch => handle_confirm_switch(key, state),
        BrowserMode::ConfirmDelete { .. } => handle_confirm_delete(key, state),
        BrowserMode::NamingFork { .. } => handle_naming_fork(key, state),
        BrowserMode::LinkingCloud => handle_link_cloud(key, state),
        BrowserMode::SelectingRepo => handle_selecting_repo(key, state),
        BrowserMode::ConfirmPush => handle_confirm_push(key, state),
        BrowserMode::ConfirmPull => handle_confirm_pull(key, state),
        BrowserMode::ConfirmUnlink => handle_confirm_unlink(key, state),
        BrowserMode::DivergenceResolution => handle_divergence_resolution(key, state),
    }
}

fn handle_browse(key: KeyEvent, state: &mut TimeVaultState) -> TimeVaultAction {
    match key.code {
        KeyCode::Esc => TimeVaultAction::Close,
        KeyCode::Tab | KeyCode::BackTab => {
            state.focus = match state.focus {
                PanelFocus::Left => PanelFocus::Right,
                PanelFocus::Right => PanelFocus::Left,
            };
            TimeVaultAction::Continue
        }
        KeyCode::Up => {
            match state.focus {
                PanelFocus::Left => {
                    if state.selected_branch > 0 {
                        state.selected_branch -= 1;
                        // Auto-refresh commits when branch selection changes.
                        state.selected_commit = 0;
                        if let Some(name) = state.selected_branch_name() {
                            return TimeVaultAction::RefreshCommits {
                                branch_name: name.to_string(),
                            };
                        }
                    }
                }
                PanelFocus::Right => {
                    if state.selected_commit > 0 {
                        state.selected_commit -= 1;
                    }
                }
            }
            TimeVaultAction::Continue
        }
        KeyCode::Down => {
            match state.focus {
                PanelFocus::Left => {
                    if !state.branches.is_empty()
                        && state.selected_branch < state.branches.len() - 1
                    {
                        state.selected_branch += 1;
                        state.selected_commit = 0;
                        if let Some(name) = state.selected_branch_name() {
                            return TimeVaultAction::RefreshCommits {
                                branch_name: name.to_string(),
                            };
                        }
                    }
                }
                PanelFocus::Right => {
                    if !state.commits.is_empty() && state.selected_commit < state.commits.len() - 1
                    {
                        state.selected_commit += 1;
                    }
                }
            }
            TimeVaultAction::Continue
        }
        KeyCode::Enter => match state.focus {
            PanelFocus::Left => {
                // Switch to selected branch (if not already active).
                if !state.selected_branch_is_active() {
                    state.mode = BrowserMode::ConfirmSwitch;
                }
                TimeVaultAction::Continue
            }
            PanelFocus::Right => {
                if !state.commits.is_empty() {
                    if state.selected_branch_is_active() {
                        // Restore within the active branch.
                        state.mode = BrowserMode::ConfirmRestore;
                    } else {
                        // Switch to the viewed branch first.
                        state.mode = BrowserMode::ConfirmSwitch;
                    }
                }
                TimeVaultAction::Continue
            }
        },
        KeyCode::Char('b') | KeyCode::Char('B') => {
            let branch_name = state.selected_branch_name().unwrap_or("?").to_string();

            let (commit_id, fork_source) = match state.focus {
                PanelFocus::Right => {
                    let commit = state.commits.get(state.selected_commit).cloned();
                    let id = commit.as_ref().map(|c| c.id.clone());
                    let source = commit.map(|c| ForkSource {
                        branch_name: branch_name.clone(),
                        commit: c,
                        is_branch_tip: false,
                    });
                    (id, source)
                }
                PanelFocus::Left => {
                    let head = state
                        .branches
                        .get(state.selected_branch)
                        .and_then(|b| b.head_commit.clone());
                    let id = head.as_ref().map(|c| c.id.clone());
                    let source = head.map(|c| ForkSource {
                        branch_name: branch_name.clone(),
                        commit: c,
                        is_branch_tip: true,
                    });
                    (id, source)
                }
            };
            if let Some(commit_id) = commit_id {
                state.mode = BrowserMode::NamingFork { commit_id };
                state.fork_name_input.clear();
                state.fork_name_error = None;
                state.fork_source = fork_source;
            }
            TimeVaultAction::Continue
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            // Delete only from left panel, not main or active.
            if state.focus == PanelFocus::Left
                && !state.selected_branch_is_main()
                && !state.selected_branch_is_active()
            {
                if let Some(name) = state.selected_branch_name() {
                    let branch_name = name.to_string();
                    state.delete_confirm_input.clear();
                    state.mode = BrowserMode::ConfirmDelete { branch_name };
                }
            }
            TimeVaultAction::Continue
        }
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if state.focus == PanelFocus::Left {
                use crate::history::cloud::CloudStatus;
                match &state.cloud_status {
                    CloudStatus::Offline => {
                        state.cloud_token_input.clear();
                        state.cloud_token_error = None;
                        state.mode = BrowserMode::LinkingCloud;
                    }
                    CloudStatus::Linked | CloudStatus::OutOfSync | CloudStatus::Error(_) => {
                        state.mode = BrowserMode::ConfirmPush;
                    }
                    CloudStatus::Syncing => {}
                }
            }
            TimeVaultAction::Continue
        }
        KeyCode::Char('v') | KeyCode::Char('V') => {
            if state.focus == PanelFocus::Left {
                use crate::history::cloud::CloudStatus;
                let is_linked = !matches!(
                    &state.cloud_status,
                    CloudStatus::Offline | CloudStatus::Syncing
                );
                if is_linked {
                    state.mode = BrowserMode::ConfirmPull;
                }
            }
            TimeVaultAction::Continue
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            if state.focus == PanelFocus::Left {
                use crate::history::cloud::CloudStatus;
                let is_linked = !matches!(
                    &state.cloud_status,
                    CloudStatus::Offline | CloudStatus::Syncing
                );
                if is_linked {
                    return TimeVaultAction::ChangeRepo;
                }
            }
            TimeVaultAction::Continue
        }
        KeyCode::Char('x') | KeyCode::Char('X') => {
            if state.focus == PanelFocus::Left {
                use crate::history::cloud::CloudStatus;
                let is_linked = !matches!(
                    &state.cloud_status,
                    CloudStatus::Offline | CloudStatus::Syncing
                );
                if is_linked {
                    state.mode = BrowserMode::ConfirmUnlink;
                }
            }
            TimeVaultAction::Continue
        }
        _ => TimeVaultAction::Continue,
    }
}

fn handle_confirm_restore(key: KeyEvent, state: &mut TimeVaultState) -> TimeVaultAction {
    match key.code {
        KeyCode::Enter => {
            if let Some(id) = state.selected_commit_id() {
                let commit_id = id.to_string();
                state.mode = BrowserMode::Browse;
                TimeVaultAction::Restore { commit_id }
            } else {
                state.mode = BrowserMode::Browse;
                TimeVaultAction::Continue
            }
        }
        KeyCode::Esc => {
            state.mode = BrowserMode::Browse;
            TimeVaultAction::Continue
        }
        _ => TimeVaultAction::Continue,
    }
}

fn handle_confirm_switch(key: KeyEvent, state: &mut TimeVaultState) -> TimeVaultAction {
    match key.code {
        KeyCode::Enter => {
            if let Some(name) = state.selected_branch_name() {
                let branch_name = name.to_string();
                state.mode = BrowserMode::Browse;
                TimeVaultAction::SwitchBranch { branch_name }
            } else {
                state.mode = BrowserMode::Browse;
                TimeVaultAction::Continue
            }
        }
        KeyCode::Esc => {
            state.mode = BrowserMode::Browse;
            TimeVaultAction::Continue
        }
        _ => TimeVaultAction::Continue,
    }
}

fn handle_confirm_delete(key: KeyEvent, state: &mut TimeVaultState) -> TimeVaultAction {
    let branch_name = if let BrowserMode::ConfirmDelete { branch_name } = &state.mode {
        branch_name.clone()
    } else {
        return TimeVaultAction::Continue;
    };

    match key.code {
        KeyCode::Enter => {
            if state.delete_confirm_input == branch_name {
                state.delete_confirm_input.clear();
                state.mode = BrowserMode::Browse;
                TimeVaultAction::DeleteBranch { branch_name }
            } else {
                TimeVaultAction::Continue
            }
        }
        KeyCode::Esc => {
            state.delete_confirm_input.clear();
            state.mode = BrowserMode::Browse;
            TimeVaultAction::Continue
        }
        KeyCode::Backspace => {
            state.delete_confirm_input.pop();
            TimeVaultAction::Continue
        }
        KeyCode::Char(c) => {
            state.delete_confirm_input.push(c);
            TimeVaultAction::Continue
        }
        _ => TimeVaultAction::Continue,
    }
}

fn handle_naming_fork(key: KeyEvent, state: &mut TimeVaultState) -> TimeVaultAction {
    match key.code {
        KeyCode::Esc => {
            state.mode = BrowserMode::Browse;
            state.fork_name_input.clear();
            state.fork_name_error = None;
            state.fork_source = None;
            TimeVaultAction::Continue
        }
        KeyCode::Backspace => {
            state.fork_name_input.pop();
            state.fork_name_error = None;
            TimeVaultAction::Continue
        }
        KeyCode::Enter => {
            let name = state.fork_name_input.clone();
            if name.is_empty() {
                state.fork_name_error = Some("name cannot be empty".to_string());
                return TimeVaultAction::Continue;
            }
            match validate_branch_name(&name) {
                Ok(()) => {
                    // Extract commit_id from the mode before changing it.
                    let commit_id = if let BrowserMode::NamingFork { commit_id } = &state.mode {
                        commit_id.clone()
                    } else {
                        return TimeVaultAction::Continue;
                    };
                    state.mode = BrowserMode::Browse;
                    state.fork_name_input.clear();
                    state.fork_name_error = None;
                    state.fork_source = None;
                    TimeVaultAction::Fork {
                        commit_id,
                        branch_name: name,
                    }
                }
                Err(e) => {
                    state.fork_name_error = Some(e.to_string());
                    TimeVaultAction::Continue
                }
            }
        }
        KeyCode::Char(c) => {
            // Auto-lowercase, only accept valid chars.
            let c = c.to_ascii_lowercase();
            if (c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
                && state.fork_name_input.len() < 16
            {
                state.fork_name_input.push(c);
                state.fork_name_error = None;
            }
            TimeVaultAction::Continue
        }
        _ => TimeVaultAction::Continue,
    }
}

fn handle_link_cloud(key: KeyEvent, state: &mut TimeVaultState) -> TimeVaultAction {
    match key.code {
        KeyCode::Esc => {
            state.cloud_token_input.clear();
            state.cloud_token_error = None;
            state.mode = BrowserMode::Browse;
            TimeVaultAction::Continue
        }
        KeyCode::Backspace => {
            state.cloud_token_input.pop();
            state.cloud_token_error = None;
            TimeVaultAction::Continue
        }
        KeyCode::Enter => {
            if state.cloud_token_input.is_empty() {
                state.cloud_token_error = Some("token cannot be empty".to_string());
                return TimeVaultAction::Continue;
            }
            let token = state.cloud_token_input.clone();
            state.cloud_token_input.clear();
            state.cloud_token_error = None;
            state.mode = BrowserMode::Browse; // Will transition to SelectingRepo on result
            TimeVaultAction::ValidateToken { token }
        }
        KeyCode::Char(c) => {
            if state.cloud_token_input.len() < 100 {
                state.cloud_token_input.push(c);
            }
            TimeVaultAction::Continue
        }
        _ => TimeVaultAction::Continue,
    }
}

fn handle_selecting_repo(key: KeyEvent, state: &mut TimeVaultState) -> TimeVaultAction {
    // List has existing repos + one "Create new" entry at the end.
    let total = state.cloud_repos.len() + 1;

    match key.code {
        KeyCode::Esc => {
            state.cloud_validated_token = None;
            state.cloud_repos.clear();
            state.cloud_repo_selected = 0;
            state.cloud_repo_input.clear();
            state.cloud_link_field = 0;
            state.cloud_token_error = None;
            state.mode = BrowserMode::Browse;
            TimeVaultAction::Continue
        }
        KeyCode::Up => {
            if state.cloud_repo_selected > 0 {
                state.cloud_repo_selected -= 1;
            }
            // Reset input field when switching away from "Create new"
            if state.cloud_repo_selected < state.cloud_repos.len() {
                state.cloud_link_field = 0;
            }
            TimeVaultAction::Continue
        }
        KeyCode::Down => {
            if state.cloud_repo_selected < total - 1 {
                state.cloud_repo_selected += 1;
            }
            TimeVaultAction::Continue
        }
        KeyCode::Enter => {
            let token = match state.cloud_validated_token.take() {
                Some(t) => t,
                None => return TimeVaultAction::Continue,
            };
            let repo_name = if state.cloud_repo_selected < state.cloud_repos.len() {
                state.cloud_repos[state.cloud_repo_selected].clone()
            } else {
                // "Create new" selected
                let name = state.cloud_repo_input.clone();
                if name.is_empty() {
                    state.cloud_token_error = Some("repo name cannot be empty".to_string());
                    state.cloud_validated_token = Some(token);
                    return TimeVaultAction::Continue;
                }
                name
            };
            state.cloud_repos.clear();
            state.cloud_repo_selected = 0;
            state.cloud_repo_input.clear();
            state.cloud_link_field = 0;
            state.cloud_token_error = None;
            state.mode = BrowserMode::Browse;
            TimeVaultAction::LinkCloud { token, repo_name }
        }
        KeyCode::Backspace => {
            // Only editable when "Create new" is selected
            if state.cloud_repo_selected >= state.cloud_repos.len() {
                state.cloud_repo_input.pop();
                state.cloud_token_error = None;
            }
            TimeVaultAction::Continue
        }
        KeyCode::Char(c) => {
            // Only accept input when "Create new" is selected
            if state.cloud_repo_selected >= state.cloud_repos.len() {
                let c = c.to_ascii_lowercase();
                if (c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
                    && state.cloud_repo_input.len() < 30
                {
                    state.cloud_repo_input.push(c);
                    state.cloud_token_error = None;
                }
            }
            TimeVaultAction::Continue
        }
        _ => TimeVaultAction::Continue,
    }
}

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

fn handle_divergence_resolution(key: KeyEvent, state: &mut TimeVaultState) -> TimeVaultAction {
    match key.code {
        KeyCode::Char('k') | KeyCode::Char('K') => {
            state.cloud_divergence = None;
            state.mode = BrowserMode::Browse;
            TimeVaultAction::ResolveKeepLocal
        }
        KeyCode::Char('c') | KeyCode::Char('C') => {
            state.cloud_divergence = None;
            state.mode = BrowserMode::Browse;
            TimeVaultAction::ResolveUseCloud
        }
        KeyCode::Char('b') | KeyCode::Char('B') => {
            state.cloud_divergence = None;
            state.mode = BrowserMode::Browse;
            TimeVaultAction::ResolveKeepBoth
        }
        KeyCode::Esc => {
            state.cloud_divergence = None;
            state.mode = BrowserMode::Browse;
            TimeVaultAction::Continue
        }
        _ => TimeVaultAction::Continue,
    }
}
