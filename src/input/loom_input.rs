//! Loom of Worlds overlay input handling.
#![allow(dead_code)]

use super::types::InputResult;
use crate::loom::types::{BuildState, BuildStep, LoomNodeRef, LoomState, LoomUiState, LoomView};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

/// Top-level dispatcher for the Loom of Worlds overlay input.
pub(super) fn handle_loom(
    key: KeyEvent,
    loom_state: &mut LoomState,
    loom_ui: &mut LoomUiState,
) -> InputResult {
    // If build flow is active, route input there first.
    if loom_ui.build.is_some() {
        return handle_build_input(key, loom_state, loom_ui);
    }

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
                }
                LoomView::ListDetail => {
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
                LoomView::Codex => {
                    loom_ui.codex_scroll = loom_ui.codex_scroll.saturating_add(1);
                }
                LoomView::FlowView => {
                    // Grid: Down moves down one row (add 2). Extends into refinery rows.
                    let total_nodes = 6 + loom_state.persistent.refineries.len();
                    if loom_ui.selected_node + 2 < total_nodes {
                        loom_ui.selected_node += 2;
                    }
                }
                LoomView::ListDetail => {
                    if loom_ui.selected_node + 1 < 6 {
                        loom_ui.selected_node += 1;
                    }
                }
            }
            InputResult::Continue
        }
        KeyCode::Left if loom_ui.view == LoomView::FlowView => {
            // 3x2 grid: Left moves to the left column (even index).
            if loom_ui.selected_node % 2 == 1 {
                loom_ui.selected_node -= 1;
            }
            InputResult::Continue
        }
        KeyCode::Right if loom_ui.view == LoomView::FlowView => {
            // Grid: Right moves to the right column (odd index). Extends into refinery rows.
            let total_nodes = 6 + loom_state.persistent.refineries.len();
            if loom_ui.selected_node.is_multiple_of(2) && loom_ui.selected_node + 1 < total_nodes {
                loom_ui.selected_node += 1;
            }
            InputResult::Continue
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
        KeyCode::Char('b') | KeyCode::Char('B')
            if loom_ui.view == LoomView::FlowView || loom_ui.view == LoomView::ListDetail =>
        {
            start_build(loom_state, loom_ui);
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

/// Start the build refinery flow.
fn start_build(loom_state: &LoomState, loom_ui: &mut LoomUiState) {
    use crate::loom::types::BuildStep;
    let tiers = crate::loom::unlocked_tiers(loom_state);
    if tiers.is_empty() {
        // Show a message explaining why building is locked.
        let completed = loom_state
            .persistent
            .patterns
            .iter()
            .filter(|p| p.completed)
            .count();
        loom_ui.build = Some(BuildState {
            step: BuildStep::Blocked {
                message: format!(
                    "Complete 1 pattern to unlock T1 refineries ({}/1 done)",
                    completed
                ),
            },
            tier: 1,
            recipe_index: 0,
            available_recipes: Vec::new(),
            eligible_sources_a: Vec::new(),
            eligible_sources_b: Vec::new(),
            selected_sources_a: Vec::new(),
            selected_sources_b: Vec::new(),
        });
        return;
    }
    // Default to lowest unlocked tier.
    let tier = tiers[0];
    let available: Vec<usize> = crate::loom::recipes::all_recipes()
        .iter()
        .enumerate()
        .filter(|(_, r)| r.tier == tier)
        .map(|(i, _)| i)
        .collect();
    if available.is_empty() {
        return;
    }
    loom_ui.build = Some(BuildState {
        step: BuildStep::SelectRecipe { cursor: 0 },
        tier,
        recipe_index: available[0],
        available_recipes: available,
        eligible_sources_a: Vec::new(),
        eligible_sources_b: Vec::new(),
        selected_sources_a: Vec::new(),
        selected_sources_b: Vec::new(),
    });
}

/// Handle input while in the build refinery flow.
fn handle_build_input(
    key: KeyEvent,
    loom_state: &mut LoomState,
    loom_ui: &mut LoomUiState,
) -> InputResult {
    // Esc always cancels the build flow.
    if key.code == KeyCode::Esc {
        loom_ui.build = None;
        return InputResult::Continue;
    }

    let build = loom_ui.build.as_mut().unwrap();

    match &mut build.step {
        BuildStep::Blocked { .. } => {
            // Any key dismisses the blocked message.
            loom_ui.build = None;
            InputResult::Continue
        }
        BuildStep::SelectRecipe { cursor } => {
            let recipes = crate::loom::recipes::all_recipes();
            match key.code {
                KeyCode::Up => {
                    *cursor = cursor.saturating_sub(1);
                }
                KeyCode::Down => {
                    if *cursor + 1 < build.available_recipes.len() {
                        *cursor += 1;
                    }
                }
                KeyCode::Enter => {
                    let recipe_idx = build.available_recipes[*cursor];
                    build.recipe_index = recipe_idx;
                    let recipe = &recipes[recipe_idx];
                    // Compute eligible sources for input A.
                    let sources_a = crate::loom::eligible_sources_for_tier(
                        loom_state,
                        build.tier,
                        recipe.input_a,
                    );
                    let sources_b = crate::loom::eligible_sources_for_tier(
                        loom_state,
                        build.tier,
                        recipe.input_b,
                    );
                    build.eligible_sources_a = sources_a.clone();
                    build.eligible_sources_b = sources_b.clone();
                    // If no eligible sources for either input, can't proceed.
                    if sources_a.is_empty() || sources_b.is_empty() {
                        // Stay on recipe select — player needs more nodes unlocked.
                        return InputResult::Continue;
                    }
                    let toggle = vec![false; sources_a.len()];
                    build.step = BuildStep::SelectSourcesA { cursor: 0, toggle };
                }
                _ => {}
            }
            InputResult::Continue
        }
        BuildStep::SelectSourcesA { cursor, toggle } => {
            match key.code {
                KeyCode::Up => {
                    *cursor = cursor.saturating_sub(1);
                }
                KeyCode::Down => {
                    if *cursor + 1 < toggle.len() {
                        *cursor += 1;
                    }
                }
                KeyCode::Char(' ') => {
                    // Toggle source selection.
                    toggle[*cursor] = !toggle[*cursor];
                }
                KeyCode::Enter => {
                    let selected: Vec<LoomNodeRef> = build
                        .eligible_sources_a
                        .iter()
                        .zip(toggle.iter())
                        .filter(|(_, &t)| t)
                        .map(|(&s, _)| s)
                        .collect();
                    if selected.is_empty() {
                        return InputResult::Continue; // Must select at least one.
                    }
                    // Store selected sources and move to input B.
                    let build = loom_ui.build.as_mut().unwrap();
                    build.selected_sources_a = selected;
                    let toggle_b = vec![false; build.eligible_sources_b.len()];
                    build.step = BuildStep::SelectSourcesB {
                        cursor: 0,
                        toggle: toggle_b,
                    };
                }
                _ => {}
            }
            InputResult::Continue
        }
        BuildStep::SelectSourcesB { cursor, toggle } => {
            match key.code {
                KeyCode::Up => {
                    *cursor = cursor.saturating_sub(1);
                }
                KeyCode::Down => {
                    if *cursor + 1 < toggle.len() {
                        *cursor += 1;
                    }
                }
                KeyCode::Char(' ') => {
                    toggle[*cursor] = !toggle[*cursor];
                }
                KeyCode::Enter => {
                    let selected: Vec<LoomNodeRef> = build
                        .eligible_sources_b
                        .iter()
                        .zip(toggle.iter())
                        .filter(|(_, &t)| t)
                        .map(|(&s, _)| s)
                        .collect();
                    if selected.is_empty() {
                        return InputResult::Continue;
                    }
                    let build = loom_ui.build.as_mut().unwrap();
                    build.selected_sources_b = selected;
                    build.step = BuildStep::Confirm;
                }
                _ => {}
            }
            InputResult::Continue
        }
        BuildStep::Confirm => match key.code {
            KeyCode::Enter => {
                let recipes = crate::loom::recipes::all_recipes();
                let recipe = &recipes[build.recipe_index];
                let sources_a = build.selected_sources_a.clone();
                let sources_b = build.selected_sources_b.clone();
                let result = crate::loom::build_refinery(
                    loom_state,
                    recipe.input_a,
                    recipe.input_b,
                    recipe.node_nature,
                    sources_a,
                    sources_b,
                );
                loom_ui.build = None;
                match result {
                    Ok(_) => InputResult::NeedsSave,
                    Err(_) => InputResult::Continue,
                }
            }
            _ => InputResult::Continue,
        },
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
                vec![crate::loom::types::LoomNodeRef::Extractor(
                    crate::loom::types::NodeId::EmberSpindle,
                )],
                vec![crate::loom::types::LoomNodeRef::Extractor(
                    crate::loom::types::NodeId::VoidCondenser,
                )],
            ));
        let mut ui = make_ui(LoomView::FlowView);
        ui.selected_node = 4; // Bottom-left extractor.

        handle_loom(key(KeyCode::Down), &mut state, &mut ui);
        assert_eq!(ui.selected_node, 6, "should enter refinery area");

        handle_loom(key(KeyCode::Up), &mut state, &mut ui);
        assert_eq!(ui.selected_node, 4, "should return to extractors");
    }
}
