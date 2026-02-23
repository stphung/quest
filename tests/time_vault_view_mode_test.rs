//! Tests for Time Vault view mode switching.

use quest::input::time_vault_input::handle_time_vault_input;
use quest::ui::time_vault_scene::{TimeVaultState, ViewMode};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::empty())
}

#[test]
fn view_mode_default_is_browse() {
    let state = TimeVaultState::new(vec![], vec![]);
    assert!(matches!(state.view_mode, ViewMode::Browse));
}

#[test]
fn pressing_g_switches_to_graph_view() {
    let mut state = TimeVaultState::new(vec![], vec![]);
    let _ = handle_time_vault_input(key(KeyCode::Char('g')), &mut state);
    assert!(matches!(state.view_mode, ViewMode::Graph));
}

#[test]
fn pressing_c_switches_to_compare_view() {
    let mut state = TimeVaultState::new(vec![], vec![]);
    let _ = handle_time_vault_input(key(KeyCode::Char('c')), &mut state);
    assert!(matches!(state.view_mode, ViewMode::Compare));
}

#[test]
fn pressing_b_switches_to_browse_view() {
    let mut state = TimeVaultState::new(vec![], vec![]);
    state.view_mode = ViewMode::Graph;
    let _ = handle_time_vault_input(key(KeyCode::Char('b')), &mut state);
    assert!(matches!(state.view_mode, ViewMode::Browse));
}

#[test]
fn graph_state_default_selection() {
    let state = TimeVaultState::new(vec![], vec![]);
    assert_eq!(state.graph.selected_col, 0);
    assert_eq!(state.graph.selected_row, 0);
    assert_eq!(state.graph.scroll_offset, 0);
}

#[test]
fn compare_state_defaults() {
    let state = TimeVaultState::new(vec![], vec![]);
    assert!(state.compare.left_branch.is_none());
    assert!(state.compare.right_branch.is_none());
    assert_eq!(state.compare.scroll_offset, 0);
}
