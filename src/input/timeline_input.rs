//! Input handling for the Timeline Browser overlay.

use crate::ui::timeline_scene::TimelineBrowserState;
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
}

/// Handle keyboard input for the timeline browser overlay.
pub fn handle_timeline_input(key: KeyEvent, state: &mut TimelineBrowserState) -> TimelineAction {
    match key.code {
        KeyCode::Esc => {
            if state.confirm_pending {
                state.confirm_pending = false;
                TimelineAction::Continue
            } else {
                TimelineAction::Close
            }
        }
        KeyCode::Up => {
            state.confirm_pending = false;
            if state.selected_commit > 0 {
                state.selected_commit -= 1;
            }
            TimelineAction::Continue
        }
        KeyCode::Down => {
            state.confirm_pending = false;
            if !state.commits.is_empty() && state.selected_commit < state.commits.len() - 1 {
                state.selected_commit += 1;
            }
            TimelineAction::Continue
        }
        KeyCode::Left => {
            state.confirm_pending = false;
            if !state.branches.is_empty() && state.selected_branch > 0 {
                state.selected_branch -= 1;
                state.selected_commit = 0;
                let branch_name = state.branches[state.selected_branch].name.clone();
                TimelineAction::RefreshCommits { branch_name }
            } else {
                TimelineAction::Continue
            }
        }
        KeyCode::Right => {
            state.confirm_pending = false;
            if !state.branches.is_empty() && state.selected_branch < state.branches.len() - 1 {
                state.selected_branch += 1;
                state.selected_commit = 0;
                let branch_name = state.branches[state.selected_branch].name.clone();
                TimelineAction::RefreshCommits { branch_name }
            } else {
                TimelineAction::Continue
            }
        }
        KeyCode::Enter => {
            if state.commits.is_empty() {
                return TimelineAction::Continue;
            }
            if state.confirm_pending {
                let commit_id = state.commits[state.selected_commit].id.clone();
                TimelineAction::Restore { commit_id }
            } else {
                state.confirm_pending = true;
                TimelineAction::Continue
            }
        }
        _ => TimelineAction::Continue,
    }
}
