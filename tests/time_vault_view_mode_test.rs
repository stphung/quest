//! Tests for Time Vault view mode switching and graph navigation.

use quest::history::graph_layout::build_graph_layout;
use quest::history::types::CommitInfo;
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

// ── Graph navigation tests ──

fn make_commits(n: usize) -> Vec<CommitInfo> {
    (0..n)
        .rev()
        .map(|i| CommitInfo {
            id: format!("c{}", i + 1),
            message: format!("Lv{}", (i + 1) * 10),
            timestamp: (i as i64 + 1) * 100,
            level: ((i + 1) * 10) as u32,
            prestige: i as u32,
            zone: (i + 1) as u32,
            playtime: 0,
        })
        .collect()
}

#[test]
fn graph_down_moves_selection() {
    let commits = make_commits(3);
    let branches = vec![("main".to_string(), commits)];
    let layout = build_graph_layout(&branches);
    let mut state = TimeVaultState::new(vec![], vec![]);
    state.view_mode = ViewMode::Graph;
    state.graph.layout = Some(layout);
    assert_eq!(state.graph.selected_row, 0);
    let _ = handle_time_vault_input(key(KeyCode::Down), &mut state);
    assert_eq!(state.graph.selected_row, 1);
}

#[test]
fn graph_up_at_top_stays() {
    let commits = make_commits(1);
    let branches = vec![("main".to_string(), commits)];
    let layout = build_graph_layout(&branches);
    let mut state = TimeVaultState::new(vec![], vec![]);
    state.view_mode = ViewMode::Graph;
    state.graph.layout = Some(layout);
    let _ = handle_time_vault_input(key(KeyCode::Up), &mut state);
    assert_eq!(state.graph.selected_row, 0);
}

#[test]
fn graph_down_clamps_at_bottom() {
    let commits = make_commits(2);
    let branches = vec![("main".to_string(), commits)];
    let layout = build_graph_layout(&branches);
    let mut state = TimeVaultState::new(vec![], vec![]);
    state.view_mode = ViewMode::Graph;
    state.graph.layout = Some(layout);
    // Move down twice — should clamp at row 1 (2 commits = rows 0,1)
    let _ = handle_time_vault_input(key(KeyCode::Down), &mut state);
    let _ = handle_time_vault_input(key(KeyCode::Down), &mut state);
    assert_eq!(state.graph.selected_row, 1);
}

#[test]
fn graph_left_right_navigation() {
    let commits = make_commits(2);
    let branches = vec![
        ("main".to_string(), commits.clone()),
        ("dev".to_string(), commits),
    ];
    let layout = build_graph_layout(&branches);
    let mut state = TimeVaultState::new(vec![], vec![]);
    state.view_mode = ViewMode::Graph;
    state.graph.layout = Some(layout);
    assert_eq!(state.graph.selected_col, 0);
    let _ = handle_time_vault_input(key(KeyCode::Right), &mut state);
    assert_eq!(state.graph.selected_col, 1);
    // Right again — should clamp at 1 (2 columns)
    let _ = handle_time_vault_input(key(KeyCode::Right), &mut state);
    assert_eq!(state.graph.selected_col, 1);
    // Left returns to 0
    let _ = handle_time_vault_input(key(KeyCode::Left), &mut state);
    assert_eq!(state.graph.selected_col, 0);
    // Left again — stays at 0
    let _ = handle_time_vault_input(key(KeyCode::Left), &mut state);
    assert_eq!(state.graph.selected_col, 0);
}

#[test]
fn graph_esc_closes() {
    let mut state = TimeVaultState::new(vec![], vec![]);
    state.view_mode = ViewMode::Graph;
    let result = handle_time_vault_input(key(KeyCode::Esc), &mut state);
    // TimeVaultAction::Close is mapped to closing the overlay; it should not
    // be TimeVaultAction::Continue. We check by matching the action type.
    // Since the test uses handle_time_vault_input which returns TimeVaultAction,
    // and Close maps to overlay close + InputResult::Continue in input/mod.rs,
    // we verify the action is not Continue.
    assert!(matches!(
        result,
        quest::input::time_vault_input::TimeVaultAction::Close
    ));
}

#[test]
fn graph_pressing_g_returns_build_graph() {
    let mut state = TimeVaultState::new(vec![], vec![]);
    let result = handle_time_vault_input(key(KeyCode::Char('g')), &mut state);
    assert!(matches!(
        result,
        quest::input::time_vault_input::TimeVaultAction::BuildGraph
    ));
    assert!(matches!(state.view_mode, ViewMode::Graph));
}
