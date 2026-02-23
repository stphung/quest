//! Input handling for the Timeline Browser overlay.

use crate::history::validate_branch_name;
use crate::ui::timeline_scene::{BrowserMode, PanelFocus, TimelineBrowserState};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

/// Actions that the timeline browser can request from the main loop.
pub enum TimelineAction {
    /// Close the overlay, no action taken.
    Close,
    /// Continue processing (state was mutated in place).
    Continue,
    /// Restore to a specific commit by its short SHA.
    Restore { commit_id: String },
    /// Switch to a different branch and reload commits.
    RefreshCommits { branch_name: String },
    /// Fork a new timeline at the given commit.
    ForkTimeline {
        commit_id: String,
        branch_name: String,
    },
    /// Switch the active branch to the named timeline.
    SwitchTimeline { branch_name: String },
    /// Delete a branch by name.
    DeleteTimeline { branch_name: String },
}

/// Handle keyboard input for the timeline browser overlay.
///
/// Dispatches by the current `BrowserMode` (state machine):
/// - **Browse**: Tab toggles focus, Up/Down navigates, Enter context-sensitive,
///   F starts fork (right panel), D starts delete (left panel).
/// - **ConfirmRestore**: Enter confirms, Esc cancels.
/// - **ConfirmDelete**: Enter confirms, Esc cancels.
/// - **NamingFork**: Character input, Backspace, Enter validates, Esc cancels.
pub fn handle_timeline_input(key: KeyEvent, state: &mut TimelineBrowserState) -> TimelineAction {
    match &state.mode {
        BrowserMode::Browse => handle_browse(key, state),
        BrowserMode::ConfirmRestore => handle_confirm_restore(key, state),
        BrowserMode::ConfirmDelete => handle_confirm_delete(key, state),
        BrowserMode::NamingFork { .. } => handle_naming_fork(key, state),
    }
}

fn handle_browse(key: KeyEvent, state: &mut TimelineBrowserState) -> TimelineAction {
    match key.code {
        KeyCode::Esc => TimelineAction::Close,
        KeyCode::Tab | KeyCode::BackTab => {
            state.focus = match state.focus {
                PanelFocus::Left => PanelFocus::Right,
                PanelFocus::Right => PanelFocus::Left,
            };
            TimelineAction::Continue
        }
        KeyCode::Up => {
            match state.focus {
                PanelFocus::Left => {
                    if state.selected_branch > 0 {
                        state.selected_branch -= 1;
                        // Auto-refresh commits when branch selection changes.
                        state.selected_commit = 0;
                        if let Some(name) = state.selected_branch_name() {
                            return TimelineAction::RefreshCommits {
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
            TimelineAction::Continue
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
                            return TimelineAction::RefreshCommits {
                                branch_name: name.to_string(),
                            };
                        }
                    }
                }
                PanelFocus::Right => {
                    if !state.commits.is_empty()
                        && state.selected_commit < state.commits.len() - 1
                    {
                        state.selected_commit += 1;
                    }
                }
            }
            TimelineAction::Continue
        }
        KeyCode::Enter => match state.focus {
            PanelFocus::Left => {
                // Switch to selected branch (if not already active).
                if !state.selected_branch_is_active() {
                    if let Some(name) = state.selected_branch_name() {
                        return TimelineAction::SwitchTimeline {
                            branch_name: name.to_string(),
                        };
                    }
                }
                TimelineAction::Continue
            }
            PanelFocus::Right => {
                // Begin restore confirmation.
                if !state.commits.is_empty() {
                    state.mode = BrowserMode::ConfirmRestore;
                }
                TimelineAction::Continue
            }
        },
        KeyCode::Char('f') | KeyCode::Char('F') => {
            // Fork only from right panel with a selected commit.
            if state.focus == PanelFocus::Right {
                if let Some(commit_id) = state.selected_commit_id() {
                    state.mode = BrowserMode::NamingFork {
                        commit_id: commit_id.to_string(),
                    };
                    state.fork_name_input.clear();
                    state.fork_name_error = None;
                }
            }
            TimelineAction::Continue
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            // Delete only from left panel, not main or active.
            if state.focus == PanelFocus::Left
                && !state.selected_branch_is_main()
                && !state.selected_branch_is_active()
            {
                state.mode = BrowserMode::ConfirmDelete;
            }
            TimelineAction::Continue
        }
        _ => TimelineAction::Continue,
    }
}

fn handle_confirm_restore(key: KeyEvent, state: &mut TimelineBrowserState) -> TimelineAction {
    match key.code {
        KeyCode::Enter => {
            if let Some(id) = state.selected_commit_id() {
                let commit_id = id.to_string();
                state.mode = BrowserMode::Browse;
                TimelineAction::Restore { commit_id }
            } else {
                state.mode = BrowserMode::Browse;
                TimelineAction::Continue
            }
        }
        KeyCode::Esc => {
            state.mode = BrowserMode::Browse;
            TimelineAction::Continue
        }
        _ => TimelineAction::Continue,
    }
}

fn handle_confirm_delete(key: KeyEvent, state: &mut TimelineBrowserState) -> TimelineAction {
    match key.code {
        KeyCode::Enter => {
            if let Some(name) = state.selected_branch_name() {
                let branch_name = name.to_string();
                state.mode = BrowserMode::Browse;
                TimelineAction::DeleteTimeline { branch_name }
            } else {
                state.mode = BrowserMode::Browse;
                TimelineAction::Continue
            }
        }
        KeyCode::Esc => {
            state.mode = BrowserMode::Browse;
            TimelineAction::Continue
        }
        _ => TimelineAction::Continue,
    }
}

fn handle_naming_fork(key: KeyEvent, state: &mut TimelineBrowserState) -> TimelineAction {
    match key.code {
        KeyCode::Esc => {
            state.mode = BrowserMode::Browse;
            state.fork_name_input.clear();
            state.fork_name_error = None;
            TimelineAction::Continue
        }
        KeyCode::Backspace => {
            state.fork_name_input.pop();
            state.fork_name_error = None;
            TimelineAction::Continue
        }
        KeyCode::Enter => {
            let name = state.fork_name_input.clone();
            if name.is_empty() {
                state.fork_name_error = Some("name cannot be empty".to_string());
                return TimelineAction::Continue;
            }
            match validate_branch_name(&name) {
                Ok(()) => {
                    // Extract commit_id from the mode before changing it.
                    let commit_id =
                        if let BrowserMode::NamingFork { commit_id } = &state.mode {
                            commit_id.clone()
                        } else {
                            return TimelineAction::Continue;
                        };
                    state.mode = BrowserMode::Browse;
                    state.fork_name_input.clear();
                    state.fork_name_error = None;
                    TimelineAction::ForkTimeline {
                        commit_id,
                        branch_name: name,
                    }
                }
                Err(e) => {
                    state.fork_name_error = Some(e.to_string());
                    TimelineAction::Continue
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
            TimelineAction::Continue
        }
        _ => TimelineAction::Continue,
    }
}
