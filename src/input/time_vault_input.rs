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
    LinkCloud {
        token: String,
        repo_name: String,
        private: bool,
    },
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
    /// Update the PAT (new token validated and ready to save).
    UpdateToken { token: String },
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
        BrowserMode::UpdatingToken => handle_updating_token(key, state),
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
                    CloudStatus::Linked
                    | CloudStatus::OutOfSync
                    | CloudStatus::TokenExpired
                    | CloudStatus::Error(_) => {
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
        KeyCode::Char('p') | KeyCode::Char('P') => {
            if state.focus == PanelFocus::Left {
                use crate::history::cloud::CloudStatus;
                let is_linked = matches!(
                    &state.cloud_status,
                    CloudStatus::Linked
                        | CloudStatus::OutOfSync
                        | CloudStatus::TokenExpired
                        | CloudStatus::Error(_)
                );
                if is_linked {
                    state.cloud_token_input.clear();
                    state.cloud_token_error = None;
                    state.mode = BrowserMode::UpdatingToken;
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
            let (repo_name, private) = if state.cloud_repo_selected < state.cloud_repos.len() {
                let repo = &state.cloud_repos[state.cloud_repo_selected];
                (repo.name.clone(), repo.private)
            } else {
                // "Create new" selected
                let name = state.cloud_repo_input.clone();
                if name.is_empty() {
                    state.cloud_token_error = Some("repo name cannot be empty".to_string());
                    state.cloud_validated_token = Some(token);
                    return TimeVaultAction::Continue;
                }
                (name, state.cloud_new_repo_private)
            };
            state.cloud_repos.clear();
            state.cloud_repo_selected = 0;
            state.cloud_repo_input.clear();
            state.cloud_link_field = 0;
            state.cloud_token_error = None;
            state.mode = BrowserMode::Browse;
            TimeVaultAction::LinkCloud {
                token,
                repo_name,
                private,
            }
        }
        KeyCode::Backspace => {
            // Only editable when "Create new" is selected
            if state.cloud_repo_selected >= state.cloud_repos.len() {
                state.cloud_repo_input.pop();
                state.cloud_token_error = None;
            }
            TimeVaultAction::Continue
        }
        KeyCode::Char('v') | KeyCode::Char('V')
            if state.cloud_repo_selected >= state.cloud_repos.len() =>
        {
            state.cloud_new_repo_private = !state.cloud_new_repo_private;
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

fn handle_updating_token(key: KeyEvent, state: &mut TimeVaultState) -> TimeVaultAction {
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
            state.mode = BrowserMode::Browse;
            TimeVaultAction::UpdateToken { token }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::cloud::{CloudStatus, RepoInfo};
    use crate::history::types::{CommitInfo, TimelineInfo};
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn make_commit(id: &str) -> CommitInfo {
        CommitInfo {
            id: id.into(),
            message: "test".into(),
            timestamp: 0,
            level: 1,
            prestige: 0,
            zone: 1,
            playtime: 0,
        }
    }

    fn make_branch(name: &str, active: bool) -> TimelineInfo {
        TimelineInfo {
            name: name.into(),
            is_active: active,
            head_commit: Some(make_commit("abc1234")),
        }
    }

    fn browsing_state() -> TimeVaultState {
        let branches = vec![
            make_branch("main", true),
            make_branch("fork-a", false),
            make_branch("fork-b", false),
        ];
        let commits = vec![make_commit("abc1234"), make_commit("def5678")];
        TimeVaultState::new(branches, commits)
    }

    // ── 1. Browse Mode — Navigation ────────────────────────────────────

    #[test]
    fn esc_closes_vault() {
        let mut state = browsing_state();
        let result = handle_time_vault_input(key(KeyCode::Esc), &mut state);
        assert!(matches!(result, TimeVaultAction::Close));
    }

    #[test]
    fn tab_toggles_focus_left_to_right() {
        let mut state = browsing_state();
        assert_eq!(state.focus, PanelFocus::Left);
        let result = handle_time_vault_input(key(KeyCode::Tab), &mut state);
        assert!(matches!(result, TimeVaultAction::Continue));
        assert_eq!(state.focus, PanelFocus::Right);
    }

    #[test]
    fn backtab_toggles_focus_right_to_left() {
        let mut state = browsing_state();
        state.focus = PanelFocus::Right;
        let result = handle_time_vault_input(key(KeyCode::BackTab), &mut state);
        assert!(matches!(result, TimeVaultAction::Continue));
        assert_eq!(state.focus, PanelFocus::Left);
    }

    #[test]
    fn up_decrements_branch() {
        let mut state = browsing_state();
        state.selected_branch = 1;
        let result = handle_time_vault_input(key(KeyCode::Up), &mut state);
        assert_eq!(state.selected_branch, 0);
        assert!(
            matches!(result, TimeVaultAction::RefreshCommits { branch_name } if branch_name == "main")
        );
    }

    #[test]
    fn up_at_zero_stays() {
        let mut state = browsing_state();
        assert_eq!(state.selected_branch, 0);
        let result = handle_time_vault_input(key(KeyCode::Up), &mut state);
        assert_eq!(state.selected_branch, 0);
        assert!(matches!(result, TimeVaultAction::Continue));
    }

    #[test]
    fn down_increments_branch() {
        let mut state = browsing_state();
        assert_eq!(state.selected_branch, 0);
        let result = handle_time_vault_input(key(KeyCode::Down), &mut state);
        assert_eq!(state.selected_branch, 1);
        assert!(
            matches!(result, TimeVaultAction::RefreshCommits { branch_name } if branch_name == "fork-a")
        );
    }

    #[test]
    fn down_at_max_stays() {
        let mut state = browsing_state();
        state.selected_branch = 2; // last branch
        let result = handle_time_vault_input(key(KeyCode::Down), &mut state);
        assert_eq!(state.selected_branch, 2);
        assert!(matches!(result, TimeVaultAction::Continue));
    }

    #[test]
    fn right_panel_up_down() {
        let mut state = browsing_state();
        state.focus = PanelFocus::Right;
        assert_eq!(state.selected_commit, 0);

        // Down → selected_commit = 1
        handle_time_vault_input(key(KeyCode::Down), &mut state);
        assert_eq!(state.selected_commit, 1);

        // Up → selected_commit = 0
        handle_time_vault_input(key(KeyCode::Up), &mut state);
        assert_eq!(state.selected_commit, 0);
    }

    // ── 2. Browse Mode — Actions ───────────────────────────────────────

    #[test]
    fn enter_left_non_active_starts_switch() {
        let mut state = browsing_state();
        state.selected_branch = 1; // fork-a, not active
        let result = handle_time_vault_input(key(KeyCode::Enter), &mut state);
        assert!(matches!(result, TimeVaultAction::Continue));
        assert_eq!(state.mode, BrowserMode::ConfirmSwitch);
    }

    #[test]
    fn enter_left_active_no_op() {
        let mut state = browsing_state();
        // selected_branch=0 is "main" which is active
        let result = handle_time_vault_input(key(KeyCode::Enter), &mut state);
        assert!(matches!(result, TimeVaultAction::Continue));
        assert_eq!(state.mode, BrowserMode::Browse);
    }

    #[test]
    fn enter_right_active_starts_restore() {
        let mut state = browsing_state();
        state.focus = PanelFocus::Right;
        // selected_branch=0 is "main" which is active
        let result = handle_time_vault_input(key(KeyCode::Enter), &mut state);
        assert!(matches!(result, TimeVaultAction::Continue));
        assert_eq!(state.mode, BrowserMode::ConfirmRestore);
    }

    #[test]
    fn enter_right_non_active_starts_switch() {
        let mut state = browsing_state();
        state.focus = PanelFocus::Right;
        state.selected_branch = 1; // fork-a, not active
        let result = handle_time_vault_input(key(KeyCode::Enter), &mut state);
        assert!(matches!(result, TimeVaultAction::Continue));
        assert_eq!(state.mode, BrowserMode::ConfirmSwitch);
    }

    #[test]
    fn b_starts_fork_right_panel() {
        let mut state = browsing_state();
        state.focus = PanelFocus::Right;
        let result = handle_time_vault_input(key(KeyCode::Char('b')), &mut state);
        assert!(matches!(result, TimeVaultAction::Continue));
        assert_eq!(
            state.mode,
            BrowserMode::NamingFork {
                commit_id: "abc1234".to_string()
            }
        );
        assert!(state.fork_source.is_some());
        assert!(!state.fork_source.as_ref().unwrap().is_branch_tip);
    }

    #[test]
    fn b_starts_fork_left_panel() {
        let mut state = browsing_state();
        state.focus = PanelFocus::Left;
        state.selected_branch = 1; // fork-a
        let result = handle_time_vault_input(key(KeyCode::Char('b')), &mut state);
        assert!(matches!(result, TimeVaultAction::Continue));
        assert!(matches!(
            state.mode,
            BrowserMode::NamingFork { commit_id: _ }
        ));
        assert!(state.fork_source.is_some());
        assert!(state.fork_source.as_ref().unwrap().is_branch_tip);
    }

    #[test]
    fn d_starts_delete_non_main_non_active() {
        let mut state = browsing_state();
        state.focus = PanelFocus::Left;
        state.selected_branch = 1; // fork-a: not main, not active
        let result = handle_time_vault_input(key(KeyCode::Char('d')), &mut state);
        assert!(matches!(result, TimeVaultAction::Continue));
        assert_eq!(
            state.mode,
            BrowserMode::ConfirmDelete {
                branch_name: "fork-a".to_string()
            }
        );
    }

    #[test]
    fn d_ignored_on_main() {
        let mut state = browsing_state();
        state.focus = PanelFocus::Left;
        state.selected_branch = 0; // main
        handle_time_vault_input(key(KeyCode::Char('d')), &mut state);
        assert_eq!(state.mode, BrowserMode::Browse);
    }

    #[test]
    fn d_ignored_on_active() {
        let mut state = browsing_state();
        state.focus = PanelFocus::Left;
        // Make fork-a the active branch (not main) to isolate the is_active guard
        state.branches[0].is_active = false;
        state.branches[1].is_active = true;
        state.selected_branch = 1; // fork-a is active
        handle_time_vault_input(key(KeyCode::Char('d')), &mut state);
        assert_eq!(state.mode, BrowserMode::Browse);
    }

    // ── 3. Browse Mode — Cloud Keys ────────────────────────────────────

    #[test]
    fn c_offline_starts_linking() {
        let mut state = browsing_state();
        state.cloud_status = CloudStatus::Offline;
        state.focus = PanelFocus::Left;
        let result = handle_time_vault_input(key(KeyCode::Char('c')), &mut state);
        assert!(matches!(result, TimeVaultAction::Continue));
        assert_eq!(state.mode, BrowserMode::LinkingCloud);
    }

    #[test]
    fn c_linked_starts_push() {
        let mut state = browsing_state();
        state.cloud_status = CloudStatus::Linked;
        state.focus = PanelFocus::Left;
        let result = handle_time_vault_input(key(KeyCode::Char('c')), &mut state);
        assert!(matches!(result, TimeVaultAction::Continue));
        assert_eq!(state.mode, BrowserMode::ConfirmPush);
    }

    #[test]
    fn v_linked_starts_pull() {
        let mut state = browsing_state();
        state.cloud_status = CloudStatus::Linked;
        state.focus = PanelFocus::Left;
        let result = handle_time_vault_input(key(KeyCode::Char('v')), &mut state);
        assert!(matches!(result, TimeVaultAction::Continue));
        assert_eq!(state.mode, BrowserMode::ConfirmPull);
    }

    #[test]
    fn x_linked_starts_unlink() {
        let mut state = browsing_state();
        state.cloud_status = CloudStatus::Linked;
        state.focus = PanelFocus::Left;
        let result = handle_time_vault_input(key(KeyCode::Char('x')), &mut state);
        assert!(matches!(result, TimeVaultAction::Continue));
        assert_eq!(state.mode, BrowserMode::ConfirmUnlink);
    }

    #[test]
    fn p_linked_starts_update_token() {
        let mut state = browsing_state();
        state.cloud_status = CloudStatus::Linked;
        state.focus = PanelFocus::Left;
        let result = handle_time_vault_input(key(KeyCode::Char('p')), &mut state);
        assert!(matches!(result, TimeVaultAction::Continue));
        assert_eq!(state.mode, BrowserMode::UpdatingToken);
    }

    #[test]
    fn r_linked_returns_change_repo() {
        let mut state = browsing_state();
        state.cloud_status = CloudStatus::Linked;
        state.focus = PanelFocus::Left;
        let result = handle_time_vault_input(key(KeyCode::Char('r')), &mut state);
        assert!(matches!(result, TimeVaultAction::ChangeRepo));
    }

    // ── 4. Confirm Restore ─────────────────────────────────────────────

    #[test]
    fn confirm_restore_enter() {
        let mut state = browsing_state();
        state.mode = BrowserMode::ConfirmRestore;
        let result = handle_time_vault_input(key(KeyCode::Enter), &mut state);
        assert!(matches!(result, TimeVaultAction::Restore { commit_id } if commit_id == "abc1234"));
        assert_eq!(state.mode, BrowserMode::Browse);
    }

    #[test]
    fn confirm_restore_esc() {
        let mut state = browsing_state();
        state.mode = BrowserMode::ConfirmRestore;
        let result = handle_time_vault_input(key(KeyCode::Esc), &mut state);
        assert!(matches!(result, TimeVaultAction::Continue));
        assert_eq!(state.mode, BrowserMode::Browse);
    }

    #[test]
    fn confirm_restore_other_key() {
        let mut state = browsing_state();
        state.mode = BrowserMode::ConfirmRestore;
        let result = handle_time_vault_input(key(KeyCode::Char('z')), &mut state);
        assert!(matches!(result, TimeVaultAction::Continue));
        assert_eq!(state.mode, BrowserMode::ConfirmRestore);
    }

    // ── 5. Confirm Switch ──────────────────────────────────────────────

    #[test]
    fn confirm_switch_enter() {
        let mut state = browsing_state();
        state.selected_branch = 1; // fork-a
        state.mode = BrowserMode::ConfirmSwitch;
        let result = handle_time_vault_input(key(KeyCode::Enter), &mut state);
        assert!(
            matches!(result, TimeVaultAction::SwitchBranch { branch_name } if branch_name == "fork-a")
        );
        assert_eq!(state.mode, BrowserMode::Browse);
    }

    #[test]
    fn confirm_switch_esc() {
        let mut state = browsing_state();
        state.mode = BrowserMode::ConfirmSwitch;
        let result = handle_time_vault_input(key(KeyCode::Esc), &mut state);
        assert!(matches!(result, TimeVaultAction::Continue));
        assert_eq!(state.mode, BrowserMode::Browse);
    }

    // ── 6. Confirm Delete ──────────────────────────────────────────────

    #[test]
    fn confirm_delete_type_name() {
        let mut state = browsing_state();
        state.mode = BrowserMode::ConfirmDelete {
            branch_name: "fork-a".to_string(),
        };
        for c in ['f', 'o', 'r', 'k'] {
            handle_time_vault_input(key(KeyCode::Char(c)), &mut state);
        }
        assert_eq!(state.delete_confirm_input, "fork");
    }

    #[test]
    fn confirm_delete_backspace() {
        let mut state = browsing_state();
        state.mode = BrowserMode::ConfirmDelete {
            branch_name: "fork-a".to_string(),
        };
        for c in ['a', 'b', 'c'] {
            handle_time_vault_input(key(KeyCode::Char(c)), &mut state);
        }
        assert_eq!(state.delete_confirm_input, "abc");
        handle_time_vault_input(key(KeyCode::Backspace), &mut state);
        assert_eq!(state.delete_confirm_input, "ab");
    }

    #[test]
    fn confirm_delete_correct_name() {
        let mut state = browsing_state();
        state.mode = BrowserMode::ConfirmDelete {
            branch_name: "fork-a".to_string(),
        };
        for c in "fork-a".chars() {
            handle_time_vault_input(key(KeyCode::Char(c)), &mut state);
        }
        let result = handle_time_vault_input(key(KeyCode::Enter), &mut state);
        assert!(
            matches!(result, TimeVaultAction::DeleteBranch { branch_name } if branch_name == "fork-a")
        );
        assert_eq!(state.mode, BrowserMode::Browse);
    }

    #[test]
    fn confirm_delete_wrong_name() {
        let mut state = browsing_state();
        state.mode = BrowserMode::ConfirmDelete {
            branch_name: "fork-a".to_string(),
        };
        for c in "wrong".chars() {
            handle_time_vault_input(key(KeyCode::Char(c)), &mut state);
        }
        let result = handle_time_vault_input(key(KeyCode::Enter), &mut state);
        assert!(matches!(result, TimeVaultAction::Continue));
        // Stays in ConfirmDelete mode
        assert!(matches!(state.mode, BrowserMode::ConfirmDelete { .. }));
    }

    // ── 7. Naming Fork ─────────────────────────────────────────────────

    #[test]
    fn fork_name_char_input() {
        let mut state = browsing_state();
        state.mode = BrowserMode::NamingFork {
            commit_id: "abc".to_string(),
        };
        for c in ['a', 'b', 'c'] {
            handle_time_vault_input(key(KeyCode::Char(c)), &mut state);
        }
        assert_eq!(state.fork_name_input, "abc");
    }

    #[test]
    fn fork_name_auto_lowercase() {
        let mut state = browsing_state();
        state.mode = BrowserMode::NamingFork {
            commit_id: "abc".to_string(),
        };
        handle_time_vault_input(key(KeyCode::Char('A')), &mut state);
        assert_eq!(state.fork_name_input, "a");
    }

    #[test]
    fn fork_name_rejects_invalid() {
        let mut state = browsing_state();
        state.mode = BrowserMode::NamingFork {
            commit_id: "abc".to_string(),
        };
        handle_time_vault_input(key(KeyCode::Char('!')), &mut state);
        assert_eq!(state.fork_name_input, "");
    }

    #[test]
    fn fork_name_max_length() {
        let mut state = browsing_state();
        state.mode = BrowserMode::NamingFork {
            commit_id: "abc".to_string(),
        };
        // Type 17 characters — only 16 should be accepted
        for _ in 0..17 {
            handle_time_vault_input(key(KeyCode::Char('a')), &mut state);
        }
        assert_eq!(state.fork_name_input.len(), 16);
    }

    #[test]
    fn fork_name_submit_valid() {
        let mut state = browsing_state();
        state.mode = BrowserMode::NamingFork {
            commit_id: "abc".to_string(),
        };
        state.fork_name_input = "my-fork".to_string();
        let result = handle_time_vault_input(key(KeyCode::Enter), &mut state);
        assert!(
            matches!(result, TimeVaultAction::Fork { commit_id, branch_name } if commit_id == "abc" && branch_name == "my-fork")
        );
        assert_eq!(state.mode, BrowserMode::Browse);
    }

    #[test]
    fn fork_name_submit_empty() {
        let mut state = browsing_state();
        state.mode = BrowserMode::NamingFork {
            commit_id: "abc".to_string(),
        };
        // fork_name_input is empty by default
        let result = handle_time_vault_input(key(KeyCode::Enter), &mut state);
        assert!(matches!(result, TimeVaultAction::Continue));
        assert!(state.fork_name_error.is_some());
    }

    // ── 8. Cloud Linking ───────────────────────────────────────────────

    #[test]
    fn link_cloud_token_input() {
        let mut state = browsing_state();
        state.mode = BrowserMode::LinkingCloud;
        for c in ['g', 'h', 'p'] {
            handle_time_vault_input(key(KeyCode::Char(c)), &mut state);
        }
        assert_eq!(state.cloud_token_input, "ghp");
    }

    #[test]
    fn link_cloud_submit_empty() {
        let mut state = browsing_state();
        state.mode = BrowserMode::LinkingCloud;
        let result = handle_time_vault_input(key(KeyCode::Enter), &mut state);
        assert!(matches!(result, TimeVaultAction::Continue));
        assert!(state.cloud_token_error.is_some());
    }

    #[test]
    fn link_cloud_submit_valid() {
        let mut state = browsing_state();
        state.mode = BrowserMode::LinkingCloud;
        state.cloud_token_input = "ghp_test".to_string();
        let result = handle_time_vault_input(key(KeyCode::Enter), &mut state);
        assert!(matches!(result, TimeVaultAction::ValidateToken { token } if token == "ghp_test"));
    }

    // ── 9. Selecting Repo ──────────────────────────────────────────────

    #[test]
    fn select_repo_up_down() {
        let mut state = browsing_state();
        state.mode = BrowserMode::SelectingRepo;
        state.cloud_repos = vec![
            RepoInfo {
                name: "repo-a".to_string(),
                private: false,
            },
            RepoInfo {
                name: "repo-b".to_string(),
                private: true,
            },
        ];
        state.cloud_repo_selected = 0;

        // Down → 1
        handle_time_vault_input(key(KeyCode::Down), &mut state);
        assert_eq!(state.cloud_repo_selected, 1);

        // Up → 0
        handle_time_vault_input(key(KeyCode::Up), &mut state);
        assert_eq!(state.cloud_repo_selected, 0);
    }

    #[test]
    fn select_repo_enter_existing() {
        let mut state = browsing_state();
        state.mode = BrowserMode::SelectingRepo;
        state.cloud_repos = vec![RepoInfo {
            name: "repo".to_string(),
            private: false,
        }];
        state.cloud_repo_selected = 0;
        state.cloud_validated_token = Some("tok".to_string());
        let result = handle_time_vault_input(key(KeyCode::Enter), &mut state);
        assert!(
            matches!(result, TimeVaultAction::LinkCloud { token, repo_name, private } if token == "tok" && repo_name == "repo" && !private)
        );
    }

    #[test]
    fn select_repo_enter_new() {
        let mut state = browsing_state();
        state.mode = BrowserMode::SelectingRepo;
        state.cloud_repos = vec![RepoInfo {
            name: "existing".to_string(),
            private: false,
        }];
        state.cloud_repo_selected = 1; // "Create new" entry
        state.cloud_repo_input = "my-repo".to_string();
        state.cloud_validated_token = Some("tok".to_string());
        state.cloud_new_repo_private = true;
        let result = handle_time_vault_input(key(KeyCode::Enter), &mut state);
        assert!(
            matches!(result, TimeVaultAction::LinkCloud { token, repo_name, private } if token == "tok" && repo_name == "my-repo" && private)
        );
    }

    #[test]
    fn select_repo_enter_new_empty() {
        let mut state = browsing_state();
        state.mode = BrowserMode::SelectingRepo;
        state.cloud_repos = vec![RepoInfo {
            name: "existing".to_string(),
            private: false,
        }];
        state.cloud_repo_selected = 1; // "Create new" entry
        state.cloud_repo_input.clear();
        state.cloud_validated_token = Some("tok".to_string());
        let result = handle_time_vault_input(key(KeyCode::Enter), &mut state);
        assert!(matches!(result, TimeVaultAction::Continue));
        assert!(state.cloud_token_error.is_some());
    }

    #[test]
    fn select_repo_v_toggles_visibility() {
        let mut state = browsing_state();
        state.mode = BrowserMode::SelectingRepo;
        state.cloud_repos = vec![RepoInfo {
            name: "existing".to_string(),
            private: false,
        }];
        // Select "Create new" entry (index >= repos.len())
        state.cloud_repo_selected = 1;
        assert!(state.cloud_new_repo_private); // default is true
        handle_time_vault_input(key(KeyCode::Char('v')), &mut state);
        assert!(!state.cloud_new_repo_private);
        handle_time_vault_input(key(KeyCode::Char('v')), &mut state);
        assert!(state.cloud_new_repo_private);
    }

    #[test]
    fn select_repo_v_on_existing_is_char() {
        let mut state = browsing_state();
        state.mode = BrowserMode::SelectingRepo;
        state.cloud_repos = vec![RepoInfo {
            name: "existing".to_string(),
            private: false,
        }];
        state.cloud_repo_selected = 0; // Existing repo selected
        let initial_private = state.cloud_new_repo_private;
        handle_time_vault_input(key(KeyCode::Char('v')), &mut state);
        // 'v' on existing repo should NOT toggle visibility
        assert_eq!(state.cloud_new_repo_private, initial_private);
    }

    // ── 10. Divergence Resolution ──────────────────────────────────────

    #[test]
    fn divergence_k_keep_local() {
        let mut state = browsing_state();
        state.mode = BrowserMode::DivergenceResolution;
        let result = handle_time_vault_input(key(KeyCode::Char('k')), &mut state);
        assert!(matches!(result, TimeVaultAction::ResolveKeepLocal));
        assert_eq!(state.mode, BrowserMode::Browse);
        assert!(state.cloud_divergence.is_none());
    }

    #[test]
    fn divergence_c_use_cloud() {
        let mut state = browsing_state();
        state.mode = BrowserMode::DivergenceResolution;
        let result = handle_time_vault_input(key(KeyCode::Char('c')), &mut state);
        assert!(matches!(result, TimeVaultAction::ResolveUseCloud));
        assert_eq!(state.mode, BrowserMode::Browse);
    }

    #[test]
    fn divergence_b_keep_both() {
        let mut state = browsing_state();
        state.mode = BrowserMode::DivergenceResolution;
        let result = handle_time_vault_input(key(KeyCode::Char('b')), &mut state);
        assert!(matches!(result, TimeVaultAction::ResolveKeepBoth));
        assert_eq!(state.mode, BrowserMode::Browse);
    }

    #[test]
    fn divergence_esc_cancels() {
        let mut state = browsing_state();
        state.mode = BrowserMode::DivergenceResolution;
        let result = handle_time_vault_input(key(KeyCode::Esc), &mut state);
        assert!(matches!(result, TimeVaultAction::Continue));
        assert_eq!(state.mode, BrowserMode::Browse);
    }

    // ── 11. Confirm Push ───────────────────────────────────────────────

    #[test]
    fn confirm_push_enter() {
        let mut state = browsing_state();
        state.mode = BrowserMode::ConfirmPush;
        let result = handle_time_vault_input(key(KeyCode::Enter), &mut state);
        assert!(matches!(result, TimeVaultAction::PushCloud));
        assert_eq!(state.mode, BrowserMode::Browse);
    }

    #[test]
    fn confirm_push_esc() {
        let mut state = browsing_state();
        state.mode = BrowserMode::ConfirmPush;
        let result = handle_time_vault_input(key(KeyCode::Esc), &mut state);
        assert!(matches!(result, TimeVaultAction::Continue));
        assert_eq!(state.mode, BrowserMode::Browse);
    }

    // ── 12. Confirm Pull ───────────────────────────────────────────────

    #[test]
    fn confirm_pull_enter() {
        let mut state = browsing_state();
        state.mode = BrowserMode::ConfirmPull;
        let result = handle_time_vault_input(key(KeyCode::Enter), &mut state);
        assert!(matches!(result, TimeVaultAction::PullCloud));
        assert_eq!(state.mode, BrowserMode::Browse);
    }

    #[test]
    fn confirm_pull_esc() {
        let mut state = browsing_state();
        state.mode = BrowserMode::ConfirmPull;
        let result = handle_time_vault_input(key(KeyCode::Esc), &mut state);
        assert!(matches!(result, TimeVaultAction::Continue));
        assert_eq!(state.mode, BrowserMode::Browse);
    }

    // ── 13. Confirm Unlink ─────────────────────────────────────────────

    #[test]
    fn confirm_unlink_enter() {
        let mut state = browsing_state();
        state.mode = BrowserMode::ConfirmUnlink;
        let result = handle_time_vault_input(key(KeyCode::Enter), &mut state);
        assert!(matches!(result, TimeVaultAction::UnlinkCloud));
        assert_eq!(state.mode, BrowserMode::Browse);
    }

    #[test]
    fn confirm_unlink_esc() {
        let mut state = browsing_state();
        state.mode = BrowserMode::ConfirmUnlink;
        let result = handle_time_vault_input(key(KeyCode::Esc), &mut state);
        assert!(matches!(result, TimeVaultAction::Continue));
        assert_eq!(state.mode, BrowserMode::Browse);
    }

    // ── 14. Updating Token ─────────────────────────────────────────────

    #[test]
    fn updating_token_char_input() {
        let mut state = browsing_state();
        state.mode = BrowserMode::UpdatingToken;
        for c in ['g', 'h', 'p'] {
            handle_time_vault_input(key(KeyCode::Char(c)), &mut state);
        }
        assert_eq!(state.cloud_token_input, "ghp");
    }

    #[test]
    fn updating_token_backspace() {
        let mut state = browsing_state();
        state.mode = BrowserMode::UpdatingToken;
        state.cloud_token_input = "ghp_".to_string();
        state.cloud_token_error = Some("old error".to_string());
        handle_time_vault_input(key(KeyCode::Backspace), &mut state);
        assert_eq!(state.cloud_token_input, "ghp");
        assert!(state.cloud_token_error.is_none());
    }

    #[test]
    fn updating_token_submit_empty() {
        let mut state = browsing_state();
        state.mode = BrowserMode::UpdatingToken;
        let result = handle_time_vault_input(key(KeyCode::Enter), &mut state);
        assert!(matches!(result, TimeVaultAction::Continue));
        assert!(state.cloud_token_error.is_some());
    }

    #[test]
    fn updating_token_submit_valid() {
        let mut state = browsing_state();
        state.mode = BrowserMode::UpdatingToken;
        state.cloud_token_input = "ghp_new".to_string();
        let result = handle_time_vault_input(key(KeyCode::Enter), &mut state);
        assert!(matches!(result, TimeVaultAction::UpdateToken { token } if token == "ghp_new"));
        assert_eq!(state.mode, BrowserMode::Browse);
        assert!(state.cloud_token_input.is_empty());
    }

    #[test]
    fn updating_token_esc() {
        let mut state = browsing_state();
        state.mode = BrowserMode::UpdatingToken;
        state.cloud_token_input = "partial".to_string();
        let result = handle_time_vault_input(key(KeyCode::Esc), &mut state);
        assert!(matches!(result, TimeVaultAction::Continue));
        assert_eq!(state.mode, BrowserMode::Browse);
        assert!(state.cloud_token_input.is_empty());
    }

    // ── 15. Cancel / Esc paths ─────────────────────────────────────────

    #[test]
    fn fork_naming_esc_clears_state() {
        let mut state = browsing_state();
        state.mode = BrowserMode::NamingFork {
            commit_id: "abc".to_string(),
        };
        state.fork_name_input = "partial".to_string();
        state.fork_name_error = Some("some error".to_string());
        state.fork_source = Some(ForkSource {
            branch_name: "main".to_string(),
            commit: make_commit("abc"),
            is_branch_tip: true,
        });
        let result = handle_time_vault_input(key(KeyCode::Esc), &mut state);
        assert!(matches!(result, TimeVaultAction::Continue));
        assert_eq!(state.mode, BrowserMode::Browse);
        assert!(state.fork_name_input.is_empty());
        assert!(state.fork_name_error.is_none());
        assert!(state.fork_source.is_none());
    }

    #[test]
    fn fork_naming_backspace() {
        let mut state = browsing_state();
        state.mode = BrowserMode::NamingFork {
            commit_id: "abc".to_string(),
        };
        state.fork_name_input = "test".to_string();
        state.fork_name_error = Some("old error".to_string());
        handle_time_vault_input(key(KeyCode::Backspace), &mut state);
        assert_eq!(state.fork_name_input, "tes");
        assert!(state.fork_name_error.is_none());
    }

    #[test]
    fn link_cloud_esc_clears_state() {
        let mut state = browsing_state();
        state.mode = BrowserMode::LinkingCloud;
        state.cloud_token_input = "partial".to_string();
        state.cloud_token_error = Some("error".to_string());
        let result = handle_time_vault_input(key(KeyCode::Esc), &mut state);
        assert!(matches!(result, TimeVaultAction::Continue));
        assert_eq!(state.mode, BrowserMode::Browse);
        assert!(state.cloud_token_input.is_empty());
        assert!(state.cloud_token_error.is_none());
    }

    #[test]
    fn link_cloud_backspace() {
        let mut state = browsing_state();
        state.mode = BrowserMode::LinkingCloud;
        state.cloud_token_input = "ghp_".to_string();
        state.cloud_token_error = Some("error".to_string());
        handle_time_vault_input(key(KeyCode::Backspace), &mut state);
        assert_eq!(state.cloud_token_input, "ghp");
        assert!(state.cloud_token_error.is_none());
    }

    #[test]
    fn confirm_delete_esc_clears_state() {
        let mut state = browsing_state();
        state.mode = BrowserMode::ConfirmDelete {
            branch_name: "fork-a".to_string(),
        };
        state.delete_confirm_input = "for".to_string();
        let result = handle_time_vault_input(key(KeyCode::Esc), &mut state);
        assert!(matches!(result, TimeVaultAction::Continue));
        assert_eq!(state.mode, BrowserMode::Browse);
        assert!(state.delete_confirm_input.is_empty());
    }

    #[test]
    fn select_repo_esc_clears_state() {
        let mut state = browsing_state();
        state.mode = BrowserMode::SelectingRepo;
        state.cloud_validated_token = Some("tok".to_string());
        state.cloud_repos = vec![RepoInfo {
            name: "repo".to_string(),
            private: false,
        }];
        state.cloud_repo_selected = 1;
        state.cloud_repo_input = "partial".to_string();
        state.cloud_token_error = Some("error".to_string());
        let result = handle_time_vault_input(key(KeyCode::Esc), &mut state);
        assert!(matches!(result, TimeVaultAction::Continue));
        assert_eq!(state.mode, BrowserMode::Browse);
        assert!(state.cloud_validated_token.is_none());
        assert!(state.cloud_repos.is_empty());
        assert_eq!(state.cloud_repo_selected, 0);
        assert!(state.cloud_repo_input.is_empty());
        assert!(state.cloud_token_error.is_none());
    }

    #[test]
    fn select_repo_backspace_on_create_new() {
        let mut state = browsing_state();
        state.mode = BrowserMode::SelectingRepo;
        state.cloud_repos = vec![RepoInfo {
            name: "existing".to_string(),
            private: false,
        }];
        state.cloud_repo_selected = 1; // "Create new"
        state.cloud_repo_input = "test".to_string();
        state.cloud_token_error = Some("error".to_string());
        handle_time_vault_input(key(KeyCode::Backspace), &mut state);
        assert_eq!(state.cloud_repo_input, "tes");
        assert!(state.cloud_token_error.is_none());
    }

    // ── 16. Edge cases ─────────────────────────────────────────────────

    #[test]
    fn cloud_keys_ignored_on_right_panel() {
        let mut state = browsing_state();
        state.cloud_status = CloudStatus::Offline;
        state.focus = PanelFocus::Right;
        handle_time_vault_input(key(KeyCode::Char('c')), &mut state);
        assert_eq!(state.mode, BrowserMode::Browse);
    }

    #[test]
    fn cloud_keys_ignored_when_syncing() {
        let mut state = browsing_state();
        state.cloud_status = CloudStatus::Syncing;
        state.focus = PanelFocus::Left;
        // 'c' during Syncing should be a no-op
        handle_time_vault_input(key(KeyCode::Char('c')), &mut state);
        assert_eq!(state.mode, BrowserMode::Browse);
        // 'v' during Syncing should be a no-op (not linked, not offline)
        handle_time_vault_input(key(KeyCode::Char('v')), &mut state);
        assert_eq!(state.mode, BrowserMode::Browse);
    }

    #[test]
    fn branch_change_resets_selected_commit() {
        let mut state = browsing_state();
        state.selected_commit = 1;
        state.selected_branch = 0;
        handle_time_vault_input(key(KeyCode::Down), &mut state);
        assert_eq!(state.selected_branch, 1);
        assert_eq!(state.selected_commit, 0);
    }
}
