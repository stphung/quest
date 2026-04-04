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
            loom_ui.view = cycle_view(loom_ui.view);
            loom_ui.selected_node = 0;
            loom_ui.codex_column = 0;
            loom_ui.codex_row = 0;
            InputResult::Continue
        }
        KeyCode::Up => {
            match loom_ui.view {
                LoomView::Codex => {
                    loom_ui.codex_row = loom_ui.codex_row.saturating_sub(1);
                }
                LoomView::FlowView => {
                    // Diamond layout: 0=ES, 1=RL, 2=RF, 3=VC, 4=SW, 5=MA, 6+=shuttles
                    // Up moves to the previous row, preserving left/right position.
                    loom_ui.selected_node = match loom_ui.selected_node {
                        0 => 0,              // ES: stay
                        1 | 2 => 0,          // RL/RF → ES
                        3 => 1,              // VC → RL
                        4 => 2,              // SW → RF
                        5 => 3,              // MA → VC (default left)
                        6 => 5,              // first shuttle → MA
                        n if n > 6 => n - 1, // shuttle list: move up one
                        n => n,
                    };
                }
            }
            InputResult::Continue
        }
        KeyCode::Down => {
            match loom_ui.view {
                LoomView::Codex => {
                    let max_row = codex_column_len(loom_ui.codex_column).saturating_sub(1);
                    if loom_ui.codex_row < max_row {
                        loom_ui.codex_row += 1;
                    }
                }
                LoomView::FlowView => {
                    // Diamond layout: down moves to the next row.
                    let total_nodes = 6 + loom_state.persistent.shuttles.len();
                    loom_ui.selected_node = match loom_ui.selected_node {
                        0 => 1,     // ES → RL (default left)
                        1 => 3,     // RL → VC
                        2 => 4,     // RF → SW
                        3 | 4 => 5, // VC/SW → MA
                        5 => {
                            // MA → first shuttle (if any)
                            if total_nodes > 6 {
                                6
                            } else {
                                5
                            }
                        }
                        n if n >= 6 => {
                            // Shuttle list: move down one
                            if n + 1 < total_nodes {
                                n + 1
                            } else {
                                n
                            }
                        }
                        n => n,
                    };
                }
            }
            InputResult::Continue
        }
        KeyCode::Left if loom_ui.view == LoomView::Codex => {
            if loom_ui.codex_column > 0 {
                loom_ui.codex_column -= 1;
                let max_row = codex_column_len(loom_ui.codex_column).saturating_sub(1);
                if loom_ui.codex_row > max_row {
                    loom_ui.codex_row = max_row;
                }
            }
            InputResult::Continue
        }
        KeyCode::Right if loom_ui.view == LoomView::Codex => {
            if loom_ui.codex_column < 2 {
                loom_ui.codex_column += 1;
                let max_row = codex_column_len(loom_ui.codex_column).saturating_sub(1);
                if loom_ui.codex_row > max_row {
                    loom_ui.codex_row = max_row;
                }
            }
            InputResult::Continue
        }
        KeyCode::Left if loom_ui.view == LoomView::FlowView => {
            // Diamond layout: left toggles to left node on pair rows.
            match loom_ui.selected_node {
                2 => loom_ui.selected_node = 1, // RF → RL
                4 => loom_ui.selected_node = 3, // SW → VC
                _ => {}
            }
            InputResult::Continue
        }
        KeyCode::Right if loom_ui.view == LoomView::FlowView => {
            // Diamond layout: right toggles to right node on pair rows.
            match loom_ui.selected_node {
                1 => loom_ui.selected_node = 2, // RL → RF
                3 => loom_ui.selected_node = 4, // VC → SW
                _ => {}
            }
            InputResult::Continue
        }
        KeyCode::Char('u') | KeyCode::Char('U')
            if loom_ui.view == LoomView::FlowView && loom_ui.selected_node < 6 =>
        {
            // Diamond layout grid_ids: 0=ES, 1=RL, 2=RF, 3=VC, 4=SW, 5=MA
            let grid_ids = [
                crate::loom::types::NodeId::EmberSpindle,
                crate::loom::types::NodeId::ReflectionLens,
                crate::loom::types::NodeId::ResonanceForge,
                crate::loom::types::NodeId::VoidCondenser,
                crate::loom::types::NodeId::SilenceWell,
                crate::loom::types::NodeId::MemoryArchive,
            ];
            let node_id = grid_ids[loom_ui.selected_node];
            if crate::loom::try_upgrade_node(loom_state, node_id) {
                InputResult::NeedsSave
            } else {
                InputResult::Continue
            }
        }
        KeyCode::Enter => InputResult::Continue,
        KeyCode::Char('b') | KeyCode::Char('B') if loom_ui.view == LoomView::FlowView => {
            start_build(loom_state, loom_ui);
            InputResult::Continue
        }
        KeyCode::Char('d') | KeyCode::Char('D')
            if loom_ui.view == LoomView::FlowView && loom_ui.selected_node >= 6 =>
        {
            let shuttle_idx = loom_ui.selected_node - 6;
            if shuttle_idx < loom_state.persistent.shuttles.len() {
                crate::loom::demolish_shuttle(loom_state, shuttle_idx);
                // Clamp selection if we deleted the last item.
                let total = 6 + loom_state.persistent.shuttles.len();
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

/// Start the build shuttle flow.
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
                    "Complete 1 pattern to unlock T1 shuttles ({}/1 done)",
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

/// Handle input while in the build shuttle flow.
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
                KeyCode::Tab | KeyCode::BackTab => {
                    // Cycle through unlocked tiers.
                    let tiers = crate::loom::unlocked_tiers(loom_state);
                    if tiers.len() > 1 {
                        let current_pos = tiers.iter().position(|&t| t == build.tier).unwrap_or(0);
                        let next_pos = if key.code == KeyCode::Tab {
                            (current_pos + 1) % tiers.len()
                        } else {
                            (current_pos + tiers.len() - 1) % tiers.len()
                        };
                        build.tier = tiers[next_pos];
                        build.available_recipes = recipes
                            .iter()
                            .enumerate()
                            .filter(|(_, r)| r.tier == build.tier)
                            .map(|(i, _)| i)
                            .collect();
                        *cursor = 0;
                        if !build.available_recipes.is_empty() {
                            build.recipe_index = build.available_recipes[0];
                        }
                    }
                }
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
                let result = crate::loom::build_shuttle(
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

/// Number of resources in a codex column.
fn codex_column_len(col: usize) -> usize {
    match col {
        0 => 6,
        1 => 6,
        2 => 1,
        _ => 0,
    }
}

/// Cycle through views.
fn cycle_view(current: LoomView) -> LoomView {
    match current {
        LoomView::FlowView => LoomView::Codex,
        LoomView::Codex => LoomView::FlowView,
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
        assert_eq!(ui.view, LoomView::Codex);

        handle_loom(key(KeyCode::Tab), &mut state, &mut ui);
        assert_eq!(ui.view, LoomView::FlowView);
    }

    #[test]
    fn left_right_no_op_outside_flow_view() {
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
    fn test_navigation_extends_to_shuttles() {
        let mut state = LoomState::new();
        crate::loom::logic::initialize_loom(&mut state);
        state
            .persistent
            .shuttles
            .push(crate::loom::types::Shuttle::new(
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
        ui.selected_node = 4; // SW in diamond layout

        handle_loom(key(KeyCode::Down), &mut state, &mut ui);
        assert_eq!(ui.selected_node, 5, "SW → MA");

        handle_loom(key(KeyCode::Down), &mut state, &mut ui);
        assert_eq!(ui.selected_node, 6, "MA → shuttle area");

        handle_loom(key(KeyCode::Up), &mut state, &mut ui);
        assert_eq!(ui.selected_node, 5, "shuttle → MA");
    }
}
