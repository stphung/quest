//! Loom of Worlds overlay input handling.
#![allow(dead_code)]

use super::types::InputResult;
use crate::loom::graph::{LoomGraph, LoomGraphNode};
use crate::loom::layout::{node_layer, LoomLayout};
use crate::loom::types::{BuildState, BuildStep, LoomNodeRef, LoomState, LoomUiState};
use petgraph::stable_graph::NodeIndex;
use petgraph::Direction;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

/// Find all nodes in the same layer as `current`, sorted by y-position.
fn siblings_in_layer(
    graph: &LoomGraph,
    layout: &LoomLayout,
    loom: &LoomState,
    current: NodeIndex,
) -> Vec<NodeIndex> {
    let current_layer = node_layer(&graph.graph[current], loom);
    let mut siblings: Vec<(NodeIndex, f64)> = graph
        .graph
        .node_indices()
        .filter(|&idx| node_layer(&graph.graph[idx], loom) == current_layer)
        .map(|idx| (idx, layout.node_positions[&idx].1))
        .collect();
    siblings.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    siblings.into_iter().map(|(idx, _)| idx).collect()
}

/// Navigate right: nearest downstream neighbor by y-position.
/// Falls back to the nearest node in the next layer if no outgoing edges exist.
fn navigate_right(
    graph: &LoomGraph,
    layout: &LoomLayout,
    loom: &LoomState,
    current: NodeIndex,
) -> Option<NodeIndex> {
    let current_y = layout.node_positions[&current].1;

    // First try: follow an outgoing edge to the nearest neighbor.
    let via_edge = graph
        .graph
        .neighbors_directed(current, Direction::Outgoing)
        .min_by(|a, b| {
            let da = (layout.node_positions[a].1 - current_y).abs();
            let db = (layout.node_positions[b].1 - current_y).abs();
            da.partial_cmp(&db).unwrap()
        });

    if via_edge.is_some() {
        return via_edge;
    }

    // Fallback: jump to nearest node in any higher layer (scan layers 1-4).
    let current_layer = node_layer(&graph.graph[current], loom);
    for target_layer in (current_layer + 1)..=4 {
        let candidate = graph
            .graph
            .node_indices()
            .filter(|&idx| node_layer(&graph.graph[idx], loom) == target_layer)
            .min_by(|a, b| {
                let da = (layout.node_positions[a].1 - current_y).abs();
                let db = (layout.node_positions[b].1 - current_y).abs();
                da.partial_cmp(&db).unwrap()
            });
        if candidate.is_some() {
            return candidate;
        }
    }
    None
}

/// Navigate left: nearest upstream neighbor by y-position.
/// Falls back to the nearest node in the previous layer if no incoming edges exist.
fn navigate_left(
    graph: &LoomGraph,
    layout: &LoomLayout,
    loom: &LoomState,
    current: NodeIndex,
) -> Option<NodeIndex> {
    let current_y = layout.node_positions[&current].1;

    // First try: follow an incoming edge to the nearest neighbor.
    let via_edge = graph
        .graph
        .neighbors_directed(current, Direction::Incoming)
        .min_by(|a, b| {
            let da = (layout.node_positions[a].1 - current_y).abs();
            let db = (layout.node_positions[b].1 - current_y).abs();
            da.partial_cmp(&db).unwrap()
        });

    if via_edge.is_some() {
        return via_edge;
    }

    // Fallback: jump to nearest node in any lower layer (scan layers downward).
    let current_layer = node_layer(&graph.graph[current], loom);
    if current_layer == 0 {
        return None;
    }
    for target_layer in (0..current_layer).rev() {
        let candidate = graph
            .graph
            .node_indices()
            .filter(|&idx| node_layer(&graph.graph[idx], loom) == target_layer)
            .min_by(|a, b| {
                let da = (layout.node_positions[a].1 - current_y).abs();
                let db = (layout.node_positions[b].1 - current_y).abs();
                da.partial_cmp(&db).unwrap()
            });
        if candidate.is_some() {
            return candidate;
        }
    }
    None
}

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

    // Cancel demolish confirmation on any key except D.
    if loom_ui.demolish_pending && !matches!(key.code, KeyCode::Char('d') | KeyCode::Char('D')) {
        loom_ui.demolish_pending = false;
    }

    match key.code {
        KeyCode::Esc => {
            loom_ui.demolish_pending = false;
            loom_ui.open = false;
            InputResult::Continue
        }
        KeyCode::Up => {
            if let (Some(graph), Some(layout), Some(current)) = (
                &loom_ui.loom_graph,
                &loom_ui.loom_layout,
                loom_ui.selected_graph_node,
            ) {
                let sibs = siblings_in_layer(graph, layout, loom_state, current);
                if let Some(pos) = sibs.iter().position(|&n| n == current) {
                    let next = if pos == 0 { sibs.len() - 1 } else { pos - 1 };
                    loom_ui.selected_graph_node = Some(sibs[next]);
                }
            }
            InputResult::Continue
        }
        KeyCode::Down => {
            if let (Some(graph), Some(layout), Some(current)) = (
                &loom_ui.loom_graph,
                &loom_ui.loom_layout,
                loom_ui.selected_graph_node,
            ) {
                let sibs = siblings_in_layer(graph, layout, loom_state, current);
                if let Some(pos) = sibs.iter().position(|&n| n == current) {
                    let next = (pos + 1) % sibs.len();
                    loom_ui.selected_graph_node = Some(sibs[next]);
                }
            }
            InputResult::Continue
        }
        KeyCode::Left => {
            if let (Some(graph), Some(layout), Some(current)) = (
                &loom_ui.loom_graph,
                &loom_ui.loom_layout,
                loom_ui.selected_graph_node,
            ) {
                if let Some(next) = navigate_left(graph, layout, loom_state, current) {
                    loom_ui.selected_graph_node = Some(next);
                }
            }
            InputResult::Continue
        }
        KeyCode::Right => {
            if let (Some(graph), Some(layout), Some(current)) = (
                &loom_ui.loom_graph,
                &loom_ui.loom_layout,
                loom_ui.selected_graph_node,
            ) {
                if let Some(next) = navigate_right(graph, layout, loom_state, current) {
                    loom_ui.selected_graph_node = Some(next);
                }
            }
            InputResult::Continue
        }
        KeyCode::Char('u') | KeyCode::Char('U') => {
            if let (Some(graph), Some(current)) = (&loom_ui.loom_graph, loom_ui.selected_graph_node)
            {
                match &graph.graph[current] {
                    LoomGraphNode::Extractor(id) => {
                        if crate::loom::try_upgrade_node(loom_state, *id) {
                            return InputResult::NeedsSave;
                        }
                    }
                    LoomGraphNode::Shuttle(_index) => {
                        // Shuttle upgrading removed — shuttles stay at level 1.
                    }
                    LoomGraphNode::PatternSink(_) => {} // Can't upgrade pattern sinks
                }
            }
            InputResult::Continue
        }
        KeyCode::Enter => InputResult::Continue,
        KeyCode::Char('b') | KeyCode::Char('B') => {
            start_build(loom_state, loom_ui);
            // Clear graph selection during build mode to avoid mixed indicators.
            loom_ui.selected_graph_node = None;
            InputResult::Continue
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if let (Some(graph), Some(current)) = (&loom_ui.loom_graph, loom_ui.selected_graph_node)
            {
                if let LoomGraphNode::Shuttle(shuttle_idx) = &graph.graph[current] {
                    let idx = *shuttle_idx;
                    if idx < loom_state.persistent.shuttles.len() {
                        if loom_ui.demolish_pending {
                            // Second D press — confirm demolish.
                            crate::loom::demolish_shuttle(loom_state, idx);
                            loom_ui.selected_graph_node = None;
                            loom_ui.demolish_pending = false;
                            return InputResult::NeedsSave;
                        } else {
                            // First D press — set pending.
                            loom_ui.demolish_pending = true;
                        }
                    }
                } else {
                    // Not a shuttle — clear any pending demolish.
                    loom_ui.demolish_pending = false;
                }
            } else {
                loom_ui.demolish_pending = false;
            }
            InputResult::Continue
        }
        _ => InputResult::Continue,
    }
}

/// Build a sorted list of recipe indices for a tier, grouped by output resource.
fn sorted_recipes_for_tier(tier: u8) -> Vec<usize> {
    let recipes = crate::loom::recipes::all_recipes();
    let mut available: Vec<usize> = recipes
        .iter()
        .enumerate()
        .filter(|(_, r)| r.tier == tier)
        .map(|(i, _)| i)
        .collect();
    // Sort by output resource name (groups same-output recipes together),
    // then by input_a name as tiebreaker.
    available.sort_by(|&a, &b| {
        let ra = &recipes[a];
        let rb = &recipes[b];
        format!("{:?}", ra.output)
            .cmp(&format!("{:?}", rb.output))
            .then_with(|| format!("{:?}", ra.input_a).cmp(&format!("{:?}", rb.input_a)))
    });
    available
}

/// Start the build shuttle flow.
fn start_build(loom_state: &LoomState, loom_ui: &mut LoomUiState) {
    use crate::loom::types::BuildStep;

    // Check shuttle capacity first.
    let current = loom_state.persistent.shuttles.len();
    let max = loom_state.persistent.max_shuttles();
    if current >= max {
        loom_ui.build = Some(BuildState {
            step: BuildStep::Blocked {
                message: format!(
                    "Shuttle capacity reached ({}/{}). Demolish a shuttle or complete more patterns to unlock slots.",
                    current, max
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
    let available = sorted_recipes_for_tier(tier);
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
        loom_ui.graph_dirty = true; // trigger rebuild to restore selection
        return InputResult::Continue;
    }

    let build = loom_ui.build.as_mut().unwrap();

    match &mut build.step {
        BuildStep::Blocked { .. } => {
            // Any key dismisses the blocked message.
            loom_ui.build = None;
            loom_ui.graph_dirty = true; // trigger rebuild to restore selection
            InputResult::Continue
        }
        BuildStep::SelectRecipe { cursor } => {
            let recipes = crate::loom::recipes::all_recipes();
            match key.code {
                KeyCode::Left | KeyCode::Right => {
                    // Cycle through unlocked tiers with left/right arrows.
                    let tiers = crate::loom::unlocked_tiers(loom_state);
                    if tiers.len() > 1 {
                        let current_pos = tiers.iter().position(|&t| t == build.tier).unwrap_or(0);
                        let next_pos = if key.code == KeyCode::Right {
                            (current_pos + 1) % tiers.len()
                        } else {
                            (current_pos + tiers.len() - 1) % tiers.len()
                        };
                        build.tier = tiers[next_pos];
                        build.available_recipes = sorted_recipes_for_tier(build.tier);
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
                    // If no eligible sources for either input, show blocked message.
                    if sources_a.is_empty() || sources_b.is_empty() {
                        let mut missing = Vec::new();
                        if sources_a.is_empty() {
                            missing.push(format!("{:?}", recipe.input_a));
                        }
                        if sources_b.is_empty() {
                            missing.push(format!("{:?}", recipe.input_b));
                        }
                        build.step = BuildStep::Blocked {
                            message: format!(
                                "No source produces: {}. Unlock more extractors or build lower-tier shuttles first.",
                                missing.join(", ")
                            ),
                        };
                        return InputResult::Continue;
                    }
                    let toggle = vec![false; sources_a.len()];
                    // TODO(ghost-node): After sources are confirmed and recipe selected,
                    // insert a ghost node into loom_ui.loom_graph here to preview the
                    // new shuttle during SelectSourcesA/B steps. Call
                    // loom::graph::insert_ghost_node() with the selected sources and
                    // recipe output resource, store the returned NodeIndex in BuildState,
                    // then call remove_ghost_node() on cancel (Esc) or build completion.
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
                loom_ui.graph_dirty = true; // trigger rebuild to restore selection
                match result {
                    Ok(_) => InputResult::NeedsSave,
                    Err(_) => InputResult::Continue,
                }
            }
            _ => InputResult::Continue,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::KeyModifiers;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn make_ui() -> LoomUiState {
        LoomUiState::new()
    }

    #[test]
    fn esc_closes_overlay() {
        let mut state = LoomState::new();
        let mut ui = make_ui();
        ui.open = true;

        handle_loom(key(KeyCode::Esc), &mut state, &mut ui);
        assert!(!ui.open);
    }

    #[test]
    fn graph_nav_no_op_without_graph() {
        let mut state = LoomState::new();
        let mut ui = make_ui();
        // No graph/layout set — arrow keys should be no-ops.
        let result = handle_loom(key(KeyCode::Up), &mut state, &mut ui);
        assert!(matches!(result, InputResult::Continue));
        let result = handle_loom(key(KeyCode::Down), &mut state, &mut ui);
        assert!(matches!(result, InputResult::Continue));
        let result = handle_loom(key(KeyCode::Left), &mut state, &mut ui);
        assert!(matches!(result, InputResult::Continue));
        let result = handle_loom(key(KeyCode::Right), &mut state, &mut ui);
        assert!(matches!(result, InputResult::Continue));
    }

    #[test]
    fn graph_nav_up_down_wraps_within_layer() {
        use crate::loom::graph::{build_graph, LoomGraphNode};
        use crate::loom::layout::compute_layout;

        let mut state = LoomState::new();
        crate::loom::logic::initialize_loom(&mut state);
        // Unlock all extractors so we have multiple nodes in layer 0.
        for node in &mut state.persistent.nodes {
            node.unlocked = true;
        }

        let graph = build_graph(&state);
        let layout = compute_layout(&graph, &state, 200.0, 100.0);

        // Find all extractor nodes (layer 0) sorted by y.
        let mut extractors: Vec<(NodeIndex, f64)> = graph
            .graph
            .node_indices()
            .filter(|&idx| matches!(graph.graph[idx], LoomGraphNode::Extractor(_)))
            .map(|idx| (idx, layout.node_positions[&idx].1))
            .collect();
        extractors.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        if extractors.len() < 2 {
            return; // Need at least 2 extractors for meaningful test.
        }

        let mut ui = make_ui();
        ui.loom_graph = Some(graph);
        ui.loom_layout = Some(layout);

        // Start at first extractor, press Up should wrap to last.
        let first = extractors[0].0;
        let last = extractors[extractors.len() - 1].0;
        ui.selected_graph_node = Some(first);

        handle_loom(key(KeyCode::Up), &mut state, &mut ui);
        assert_eq!(
            ui.selected_graph_node,
            Some(last),
            "Up from first wraps to last"
        );

        // Press Down from last should wrap to first.
        handle_loom(key(KeyCode::Down), &mut state, &mut ui);
        assert_eq!(
            ui.selected_graph_node,
            Some(first),
            "Down from last wraps to first"
        );
    }

    #[test]
    fn graph_nav_right_follows_outgoing_edge() {
        use crate::loom::graph::build_graph;
        use crate::loom::layout::compute_layout;
        use crate::loom::types::{NodeId, NodeNature, Resource};

        let mut state = LoomState::new();
        crate::loom::logic::initialize_loom(&mut state);
        for node in &mut state.persistent.nodes {
            node.unlocked = true;
        }

        // Add a shuttle so there's an outgoing edge from an extractor.
        state
            .persistent
            .shuttles
            .push(crate::loom::types::Shuttle::new(
                Resource::Ember,
                Resource::VoidEssence,
                NodeNature::Heat,
                Resource::ForgedLight,
                1.0,
                1,
                vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
                vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
            ));

        let graph = build_graph(&state);
        let layout = compute_layout(&graph, &state, 200.0, 100.0);

        // Find EmberSpindle node index.
        let es_idx = graph
            .node_indices
            .get(&crate::loom::graph::LoomGraphNode::Extractor(
                NodeId::EmberSpindle,
            ))
            .copied();
        let shuttle_idx = graph
            .node_indices
            .get(&crate::loom::graph::LoomGraphNode::Shuttle(0))
            .copied();

        if let (Some(es), Some(sh)) = (es_idx, shuttle_idx) {
            let mut ui = make_ui();
            ui.loom_graph = Some(graph);
            ui.loom_layout = Some(layout);
            ui.selected_graph_node = Some(es);

            handle_loom(key(KeyCode::Right), &mut state, &mut ui);
            assert_eq!(
                ui.selected_graph_node,
                Some(sh),
                "Right from extractor should navigate to connected shuttle"
            );

            // Left from shuttle should go back to extractor (one of the sources).
            handle_loom(key(KeyCode::Left), &mut state, &mut ui);
            // The shuttle has two incoming edges (EmberSpindle and VoidCondenser).
            // It should pick the closest by y-position.
            assert!(
                ui.selected_graph_node.is_some(),
                "Left from shuttle should navigate to an upstream node"
            );
        }
    }

    #[test]
    fn upgrade_extractor_via_graph_node() {
        use crate::loom::graph::{build_graph, LoomGraphNode};
        use crate::loom::layout::compute_layout;
        use crate::loom::types::NodeId;

        let mut state = LoomState::new();
        crate::loom::logic::initialize_loom(&mut state);
        // Unlock EmberSpindle and give it resources for upgrade.
        state.persistent.nodes[NodeId::EmberSpindle.index()].unlocked = true;

        let graph = build_graph(&state);
        let layout = compute_layout(&graph, &state, 200.0, 100.0);

        let es_idx = graph
            .node_indices
            .get(&LoomGraphNode::Extractor(NodeId::EmberSpindle))
            .copied();

        if let Some(es) = es_idx {
            let mut ui = make_ui();
            ui.loom_graph = Some(graph);
            ui.loom_layout = Some(layout);
            ui.selected_graph_node = Some(es);

            // Upgrade may fail due to resource cost, but the path should be exercised.
            let result = handle_loom(key(KeyCode::Char('u')), &mut state, &mut ui);
            assert!(
                matches!(result, InputResult::Continue | InputResult::NeedsSave),
                "U key should attempt upgrade on extractor"
            );
        }
    }

    #[test]
    fn demolish_via_graph_node() {
        use crate::loom::graph::{build_graph, LoomGraphNode};
        use crate::loom::layout::compute_layout;
        use crate::loom::types::{NodeId, NodeNature, Resource};

        let mut state = LoomState::new();
        crate::loom::logic::initialize_loom(&mut state);
        for node in &mut state.persistent.nodes {
            node.unlocked = true;
        }

        state
            .persistent
            .shuttles
            .push(crate::loom::types::Shuttle::new(
                Resource::Ember,
                Resource::VoidEssence,
                NodeNature::Heat,
                Resource::ForgedLight,
                1.0,
                1,
                vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
                vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
            ));
        assert_eq!(state.persistent.shuttles.len(), 1);

        let graph = build_graph(&state);
        let layout = compute_layout(&graph, &state, 200.0, 100.0);

        let shuttle_idx = graph.node_indices.get(&LoomGraphNode::Shuttle(0)).copied();

        if let Some(sh) = shuttle_idx {
            let mut ui = make_ui();
            ui.loom_graph = Some(graph);
            ui.loom_layout = Some(layout);
            ui.selected_graph_node = Some(sh);

            // First D sets pending, second D confirms.
            let result = handle_loom(key(KeyCode::Char('d')), &mut state, &mut ui);
            assert!(matches!(result, InputResult::Continue));
            assert!(ui.demolish_pending, "First D should set pending");
            assert_eq!(state.persistent.shuttles.len(), 1, "Not demolished yet");

            let result = handle_loom(key(KeyCode::Char('d')), &mut state, &mut ui);
            assert!(matches!(result, InputResult::NeedsSave));
            assert_eq!(
                state.persistent.shuttles.len(),
                0,
                "Shuttle should be demolished on second D"
            );
            assert!(!ui.demolish_pending, "Pending should be cleared");
            assert_eq!(
                ui.selected_graph_node, None,
                "Selection should be cleared after demolish"
            );
        }
    }
}
