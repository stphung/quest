//! Loom of Worlds overlay input handling.
#![allow(dead_code)]

use super::types::InputResult;
use crate::loom::types::{LoomNodeRef, LoomState, LoomUiState, LoomView};
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
            if loom_ui.view == LoomView::ArchetypeSelection
                && loom_state.persistent.archetype.is_none()
            {
                // Stay on ArchetypeSelection until archetype is picked.
            } else {
                loom_ui.view = cycle_view(loom_ui.view);
                loom_ui.selected_node = 0;
                loom_ui.codex_scroll = 0;
            }
            InputResult::Continue
        }
        KeyCode::Up => {
            match loom_ui.view {
                LoomView::ArchetypeSelection => {
                    loom_ui.selected_archetype = loom_ui.selected_archetype.saturating_sub(1);
                }
                LoomView::Codex => {
                    loom_ui.codex_scroll = loom_ui.codex_scroll.saturating_sub(1);
                }
                LoomView::FlowView => {
                    // 3x2 grid: Up moves up one row (subtract 2).
                    loom_ui.selected_node = loom_ui.selected_node.saturating_sub(2);
                    loom_ui.selected_pipe = 0;
                }
                LoomView::ListDetail => {
                    loom_ui.selected_node = loom_ui.selected_node.saturating_sub(1);
                    loom_ui.selected_pipe = 0;
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
                LoomView::Codex => {
                    loom_ui.codex_scroll = loom_ui.codex_scroll.saturating_add(1);
                }
                LoomView::FlowView => {
                    // Grid: Down moves down one row (add 2). Extends into refinery rows.
                    let total_nodes = 6 + loom_state.persistent.refineries.len();
                    if loom_ui.selected_node + 2 < total_nodes {
                        loom_ui.selected_node += 2;
                    }
                    loom_ui.selected_pipe = 0;
                }
                LoomView::ListDetail => {
                    if loom_ui.selected_node + 1 < 6 {
                        loom_ui.selected_node += 1;
                    }
                    loom_ui.selected_pipe = 0;
                }
            }
            InputResult::Continue
        }
        KeyCode::Left if loom_ui.view == LoomView::FlowView => {
            // 3x2 grid: Left moves to the left column (even index).
            if loom_ui.selected_node % 2 == 1 {
                loom_ui.selected_node -= 1;
                loom_ui.selected_pipe = 0;
            }
            InputResult::Continue
        }
        KeyCode::Right if loom_ui.view == LoomView::FlowView => {
            // Grid: Right moves to the right column (odd index). Extends into refinery rows.
            let total_nodes = 6 + loom_state.persistent.refineries.len();
            if loom_ui.selected_node.is_multiple_of(2) && loom_ui.selected_node + 1 < total_nodes {
                loom_ui.selected_node += 1;
                loom_ui.selected_pipe = 0;
            }
            InputResult::Continue
        }
        KeyCode::Left if loom_ui.view == LoomView::ListDetail => {
            adjust_selected_pipe(loom_state, loom_ui, -0.05);
            InputResult::NeedsSave
        }
        KeyCode::Right if loom_ui.view == LoomView::ListDetail => {
            adjust_selected_pipe(loom_state, loom_ui, 0.05);
            InputResult::NeedsSave
        }
        KeyCode::Char('u') | KeyCode::Char('U')
            if loom_ui.view == LoomView::ListDetail || loom_ui.view == LoomView::FlowView =>
        {
            let node_id = crate::loom::types::NodeId::ALL[loom_ui.selected_node.min(5)];
            if crate::loom::try_upgrade_node(loom_state, node_id) {
                InputResult::NeedsSave
            } else {
                InputResult::Continue
            }
        }
        KeyCode::Char('p') | KeyCode::Char('P') if loom_ui.view == LoomView::ListDetail => {
            // Cycle pipe selection within the selected node's outgoing pipes.
            let node_id = crate::loom::types::NodeId::ALL[loom_ui.selected_node.min(5)];
            let pipe_count = loom_state
                .persistent
                .pipes
                .iter()
                .filter(|p| p.from == LoomNodeRef::Extractor(node_id) && !p.under_construction)
                .count();
            if pipe_count > 0 {
                loom_ui.selected_pipe = (loom_ui.selected_pipe + 1) % pipe_count;
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
                crate::loom::select_archetype(loom_state, archetype);
                loom_ui.view = LoomView::FlowView;
                return InputResult::NeedsSave;
            }
            InputResult::Continue
        }
        KeyCode::Char('d') | KeyCode::Char('D')
            if loom_ui.view == LoomView::FlowView && loom_ui.selected_node >= 6 =>
        {
            let refinery_idx = loom_ui.selected_node - 6;
            if refinery_idx < loom_state.persistent.refineries.len() {
                crate::loom::demolish_refinery(loom_state, refinery_idx);
                // Clamp selection if we deleted the last item.
                let total = 6 + loom_state.persistent.refineries.len();
                if loom_ui.selected_node >= total && total > 0 {
                    loom_ui.selected_node = total - 1;
                }
                InputResult::NeedsSave
            } else {
                InputResult::Continue
            }
        }
        _ => InputResult::Continue,
    }
}

/// Adjust the split ratio of the currently selected pipe for the selected node.
///
/// `delta` is ±0.05 per keypress. The adjustment is clamped so that:
/// - The pipe ratio stays in [0.0, 1.0]
/// - The total for all outgoing pipes from the source stays ≤ 1.0
///
/// Silently no-ops if the selected node has no active outgoing pipes.
fn adjust_selected_pipe(loom_state: &mut LoomState, loom_ui: &LoomUiState, delta: f64) {
    let node_id = crate::loom::types::NodeId::ALL[loom_ui.selected_node.min(5)];

    // Collect indices of active outgoing pipes for this node (stable order).
    let pipe_indices: Vec<usize> = loom_state
        .persistent
        .pipes
        .iter()
        .enumerate()
        .filter(|(_, p)| p.from == LoomNodeRef::Extractor(node_id) && !p.under_construction)
        .map(|(i, _)| i)
        .collect();

    if pipe_indices.is_empty() {
        return;
    }

    let selected = loom_ui.selected_pipe.min(pipe_indices.len() - 1);
    let pipe_idx = pipe_indices[selected];
    let _ = crate::loom::adjust_split_ratio(loom_state, pipe_idx, delta);
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

    #[test]
    fn up_down_resets_selected_pipe() {
        let mut state = LoomState::new();
        let mut ui = make_ui(LoomView::ListDetail);
        ui.selected_pipe = 2;

        handle_loom(key(KeyCode::Down), &mut state, &mut ui);
        assert_eq!(
            ui.selected_pipe, 0,
            "moving node should reset pipe selection"
        );

        ui.selected_pipe = 2;
        handle_loom(key(KeyCode::Up), &mut state, &mut ui);
        assert_eq!(
            ui.selected_pipe, 0,
            "moving node should reset pipe selection"
        );
    }

    #[test]
    fn left_right_no_op_outside_list_detail() {
        let mut state = LoomState::new();
        let mut ui = make_ui(LoomView::FlowView);

        // Left/Right in FlowView should not be handled as ratio adjustment.
        // It falls through to the catch-all Continue branch.
        let result = handle_loom(key(KeyCode::Left), &mut state, &mut ui);
        assert!(matches!(result, InputResult::Continue));

        let result = handle_loom(key(KeyCode::Right), &mut state, &mut ui);
        assert!(matches!(result, InputResult::Continue));
    }

    #[test]
    fn left_right_adjusts_split_ratio() {
        use crate::loom::{logic::select_archetype, types::LoomArchetype, types::NodeId};

        let mut state = LoomState::new();
        select_archetype(&mut state, LoomArchetype::BurnBright);

        // Unlock VoidCondenser and add a pipe with ratio 0.5.
        state
            .persistent
            .nodes
            .iter_mut()
            .find(|n| n.id == NodeId::VoidCondenser)
            .unwrap()
            .unlocked = true;
        state.persistent.pipes.push(crate::loom::types::Pipe {
            from: LoomNodeRef::Extractor(NodeId::EmberSpindle),
            to: LoomNodeRef::Extractor(NodeId::VoidCondenser),
            tier: crate::loom::types::PipeTier::T1,
            split_ratio: 0.5,
            under_construction: false,
            construction_ticks_remaining: 0,
        });

        let mut ui = make_ui(LoomView::ListDetail);
        // EmberSpindle is node index 0 (NodeId::ALL[0]).
        ui.selected_node = 0;
        ui.selected_pipe = 0;

        // Right increases ratio by 0.05.
        let result = handle_loom(key(KeyCode::Right), &mut state, &mut ui);
        assert!(matches!(result, InputResult::NeedsSave));
        assert!(
            (state.persistent.pipes[0].split_ratio - 0.55).abs() < 0.001,
            "ratio should be 0.55, got {}",
            state.persistent.pipes[0].split_ratio
        );

        // Left decreases ratio by 0.05.
        handle_loom(key(KeyCode::Left), &mut state, &mut ui);
        assert!(
            (state.persistent.pipes[0].split_ratio - 0.5).abs() < 0.001,
            "ratio should return to 0.5, got {}",
            state.persistent.pipes[0].split_ratio
        );
    }

    #[test]
    fn left_right_no_op_when_no_pipes() {
        use crate::loom::{logic::select_archetype, types::LoomArchetype};

        let mut state = LoomState::new();
        select_archetype(&mut state, LoomArchetype::BurnBright);
        // No pipes added.

        let mut ui = make_ui(LoomView::ListDetail);
        ui.selected_node = 0;

        // Should not panic, returns NeedsSave (adjust_selected_pipe silently no-ops).
        let result = handle_loom(key(KeyCode::Right), &mut state, &mut ui);
        assert!(matches!(result, InputResult::NeedsSave));
        assert!(state.persistent.pipes.is_empty());
    }

    #[test]
    fn test_navigation_extends_to_refineries() {
        use crate::loom::{logic::select_archetype, types::LoomArchetype};

        let mut state = LoomState::new();
        select_archetype(&mut state, LoomArchetype::BurnBright);
        state
            .persistent
            .refineries
            .push(crate::loom::types::Refinery::new(
                crate::loom::types::Resource::Ember,
                crate::loom::types::Resource::VoidEssence,
                crate::loom::types::NodeNature::Heat,
                crate::loom::types::Resource::ForgedLight,
                1.0,
                1,
            ));
        let mut ui = make_ui(LoomView::FlowView);
        ui.selected_node = 4; // Bottom-left extractor.

        handle_loom(key(KeyCode::Down), &mut state, &mut ui);
        assert_eq!(ui.selected_node, 6, "should enter refinery area");

        handle_loom(key(KeyCode::Up), &mut state, &mut ui);
        assert_eq!(ui.selected_node, 4, "should return to extractors");
    }

    #[test]
    fn p_cycles_pipe_selection() {
        use crate::loom::{logic::select_archetype, types::LoomArchetype, types::NodeId};

        let mut state = LoomState::new();
        select_archetype(&mut state, LoomArchetype::BurnBright);

        // Unlock all nodes, add 2 outgoing pipes from EmberSpindle.
        for node in &mut state.persistent.nodes {
            node.unlocked = true;
        }
        state.persistent.pipes.push(crate::loom::types::Pipe {
            from: LoomNodeRef::Extractor(NodeId::EmberSpindle),
            to: LoomNodeRef::Extractor(NodeId::VoidCondenser),
            tier: crate::loom::types::PipeTier::T1,
            split_ratio: 0.5,
            under_construction: false,
            construction_ticks_remaining: 0,
        });
        state.persistent.pipes.push(crate::loom::types::Pipe {
            from: LoomNodeRef::Extractor(NodeId::EmberSpindle),
            to: LoomNodeRef::Extractor(NodeId::MemoryArchive),
            tier: crate::loom::types::PipeTier::T1,
            split_ratio: 0.5,
            under_construction: false,
            construction_ticks_remaining: 0,
        });

        let mut ui = make_ui(LoomView::ListDetail);
        ui.selected_node = 0;
        ui.selected_pipe = 0;

        handle_loom(key(KeyCode::Char('p')), &mut state, &mut ui);
        assert_eq!(ui.selected_pipe, 1);

        handle_loom(key(KeyCode::Char('p')), &mut state, &mut ui);
        assert_eq!(ui.selected_pipe, 0, "should wrap around");
    }
}
