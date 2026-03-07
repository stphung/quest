//! Loom of Worlds overlay input handling.
#![allow(dead_code)]

use super::types::InputResult;
use crate::loom::types::{LoomState, LoomUiState, LoomView};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

/// Top-level dispatcher for the Loom of Worlds overlay input.
pub(super) fn handle_loom(
    key: KeyEvent,
    loom_state: &mut LoomState,
    loom_ui: &mut LoomUiState,
) -> InputResult {
    match key.code {
        KeyCode::Esc => {
            loom_ui.open = false;
            InputResult::Continue
        }
        KeyCode::Tab => {
            // Tab cycles views, but ArchetypeSelection is sticky until an archetype is chosen.
            if loom_ui.view != LoomView::ArchetypeSelection {
                loom_ui.view = cycle_view(loom_ui.view);
                loom_ui.selected_node = 0;
            }
            InputResult::Continue
        }
        KeyCode::Up => {
            match loom_ui.view {
                LoomView::ArchetypeSelection => {
                    loom_ui.selected_archetype = loom_ui.selected_archetype.saturating_sub(1);
                }
                _ => {
                    loom_ui.selected_node = loom_ui.selected_node.saturating_sub(1);
                }
            }
            InputResult::Continue
        }
        KeyCode::Down => {
            match loom_ui.view {
                LoomView::ArchetypeSelection => {
                    if loom_ui.selected_archetype + 1 < 3 {
                        loom_ui.selected_archetype += 1;
                    }
                }
                _ => {
                    if loom_ui.selected_node + 1 < 6 {
                        loom_ui.selected_node += 1;
                    }
                }
            }
            InputResult::Continue
        }
        KeyCode::Enter => {
            if loom_ui.view == LoomView::ArchetypeSelection
                && loom_state.persistent.archetype.is_none()
            {
                let archetype = match loom_ui.selected_archetype {
                    0 => crate::loom::types::LoomArchetype::BurnBright,
                    1 => crate::loom::types::LoomArchetype::ReachWide,
                    _ => crate::loom::types::LoomArchetype::RunDeep,
                };
                loom_state.persistent.archetype = Some(archetype);
                loom_ui.view = LoomView::FlowView;
                return InputResult::NeedsSave;
            }
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}

/// Cycle through the non-ArchetypeSelection views.
fn cycle_view(current: LoomView) -> LoomView {
    match current {
        LoomView::FlowView => LoomView::ListDetail,
        LoomView::ListDetail => LoomView::Codex,
        LoomView::Codex => LoomView::FlowView,
        LoomView::ArchetypeSelection => LoomView::FlowView,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::KeyModifiers;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn make_ui(view: LoomView) -> LoomUiState {
        let mut ui = LoomUiState::new();
        ui.view = view;
        ui
    }

    #[test]
    fn esc_closes_overlay() {
        let mut state = LoomState::new();
        let mut ui = make_ui(LoomView::FlowView);
        ui.open = true;

        handle_loom(key(KeyCode::Esc), &mut state, &mut ui);
        assert!(!ui.open);
    }

    #[test]
    fn tab_cycles_views() {
        let mut state = LoomState::new();
        let mut ui = make_ui(LoomView::FlowView);

        handle_loom(key(KeyCode::Tab), &mut state, &mut ui);
        assert_eq!(ui.view, LoomView::ListDetail);

        handle_loom(key(KeyCode::Tab), &mut state, &mut ui);
        assert_eq!(ui.view, LoomView::Codex);

        handle_loom(key(KeyCode::Tab), &mut state, &mut ui);
        assert_eq!(ui.view, LoomView::FlowView);
    }

    #[test]
    fn tab_does_not_leave_archetype_selection() {
        let mut state = LoomState::new();
        let mut ui = make_ui(LoomView::ArchetypeSelection);

        handle_loom(key(KeyCode::Tab), &mut state, &mut ui);
        assert_eq!(ui.view, LoomView::ArchetypeSelection);
    }

    #[test]
    fn up_down_archetype_selection_clamps() {
        let mut state = LoomState::new();
        let mut ui = make_ui(LoomView::ArchetypeSelection);
        assert_eq!(ui.selected_archetype, 0);

        handle_loom(key(KeyCode::Up), &mut state, &mut ui);
        assert_eq!(ui.selected_archetype, 0, "should not go below 0");

        handle_loom(key(KeyCode::Down), &mut state, &mut ui);
        handle_loom(key(KeyCode::Down), &mut state, &mut ui);
        handle_loom(key(KeyCode::Down), &mut state, &mut ui);
        assert_eq!(ui.selected_archetype, 2, "should cap at 2");
    }

    #[test]
    fn enter_confirms_archetype_and_transitions_to_flow_view() {
        let mut state = LoomState::new();
        let mut ui = make_ui(LoomView::ArchetypeSelection);
        ui.selected_archetype = 1;

        let result = handle_loom(key(KeyCode::Enter), &mut state, &mut ui);
        assert!(matches!(result, InputResult::NeedsSave));
        assert_eq!(
            state.persistent.archetype,
            Some(crate::loom::types::LoomArchetype::ReachWide)
        );
        assert_eq!(ui.view, LoomView::FlowView);
    }

    #[test]
    fn enter_after_archetype_chosen_is_noop() {
        let mut state = LoomState::new();
        state.persistent.archetype = Some(crate::loom::types::LoomArchetype::BurnBright);
        let mut ui = make_ui(LoomView::ArchetypeSelection);

        let result = handle_loom(key(KeyCode::Enter), &mut state, &mut ui);
        assert!(matches!(result, InputResult::Continue));
    }

    #[test]
    fn up_down_node_selection_in_list_detail() {
        let mut state = LoomState::new();
        let mut ui = make_ui(LoomView::ListDetail);
        assert_eq!(ui.selected_node, 0);

        handle_loom(key(KeyCode::Up), &mut state, &mut ui);
        assert_eq!(ui.selected_node, 0, "should not go below 0");

        for _ in 0..10 {
            handle_loom(key(KeyCode::Down), &mut state, &mut ui);
        }
        assert_eq!(ui.selected_node, 5, "should cap at 5");
    }
}
