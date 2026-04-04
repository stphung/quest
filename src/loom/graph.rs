// Loom graph data layer — DAG representation of the production network.
// Uses petgraph's StableDiGraph to model extractors, shuttles, and pattern sinks
// with edges representing resource flows and their rates.

use petgraph::stable_graph::{NodeIndex, StableDiGraph};
use std::collections::HashMap;

use super::layout::compute_layout;
use super::logic::{node_native_resource, shuttle_effective_intake_cap};
use super::types::{LoomState, LoomUiState, NodeId, Resource};

/// A node in the Loom production graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LoomGraphNode {
    /// A fixed extractor node, identified by its NodeId.
    Extractor(NodeId),
    /// A player-built shuttle, identified by index into loom.persistent.shuttles[].
    Shuttle(usize),
    /// A pattern sink (consumption target), identified by index into loom.persistent.patterns[].
    PatternSink(usize),
}

/// An edge in the Loom production graph, representing resource flow.
#[derive(Debug, Clone)]
pub struct LoomEdge {
    /// Which resource flows along this edge.
    /// Retained for future rendering features (edge labels, filtering by resource type).
    #[allow(dead_code)]
    pub resource: Resource,
    /// Current measured flow rate (units/hr).
    pub current_rate: f64,
    /// Maximum theoretical flow rate (units/hr).
    pub max_rate: f64,
}

/// The complete Loom production graph.
#[derive(Debug)]
pub struct LoomGraph {
    /// The underlying directed graph.
    pub graph: StableDiGraph<LoomGraphNode, LoomEdge>,
    /// Reverse lookup from graph node identity to petgraph NodeIndex.
    pub node_indices: HashMap<LoomGraphNode, NodeIndex>,
}

/// Rebuild the graph and layout if dirty, then update edge rates and particle phases.
pub fn refresh_graph(
    ui: &mut LoomUiState,
    loom: &mut LoomState,
    canvas_width: f64,
    canvas_height: f64,
) {
    // Check if rebuild needed (from either UI flag or tick-path flag)
    let needs_rebuild = ui.graph_dirty || loom.graph_dirty || ui.loom_graph.is_none();

    if needs_rebuild {
        let graph = build_graph(loom);
        let layout = compute_layout(&graph, loom, canvas_width, canvas_height);

        // Preserve selected node if it still exists
        if let Some(selected) = ui.selected_graph_node {
            if graph.graph.node_weight(selected).is_none() {
                ui.selected_graph_node = None;
            }
        }
        // Default selection if none
        if ui.selected_graph_node.is_none() {
            ui.selected_graph_node = graph.graph.node_indices().next();
        }

        // Reset particle phases for new edges
        ui.particle_phases.clear();
        for edge_idx in graph.graph.edge_indices() {
            ui.particle_phases.insert(edge_idx, 0.0);
        }

        ui.loom_graph = Some(graph);
        ui.loom_layout = Some(layout);
        ui.graph_dirty = false;
        loom.graph_dirty = false;
    }

    // Update rates every tick
    if let Some(ref mut graph) = ui.loom_graph {
        update_edge_rates(graph, loom);
    }

    // Advance particle phases
    if let Some(ref graph) = ui.loom_graph {
        for edge_idx in graph.graph.edge_indices() {
            let edge = &graph.graph[edge_idx];
            let speed = if edge.max_rate > 0.0 {
                edge.current_rate / edge.max_rate
            } else {
                0.0
            };
            if let Some(phase) = ui.particle_phases.get_mut(&edge_idx) {
                *phase = (*phase + 0.05 * speed) % 1.0;
            }
        }
    }
}

/// Build the full production graph from current Loom state.
///
/// Adds unlocked extractors, all shuttles, and visible pattern sinks as nodes.
/// Connects shuttle source references as edges, and infers edges from shuttles
/// to pattern sinks when a shuttle's output matches a pattern's required resource.
pub fn build_graph(loom: &LoomState) -> LoomGraph {
    let mut graph = StableDiGraph::new();
    let mut node_indices = HashMap::new();

    // 1. Add unlocked extractors as nodes
    for node in &loom.persistent.nodes {
        if node.unlocked {
            let gn = LoomGraphNode::Extractor(node.id);
            let idx = graph.add_node(gn.clone());
            node_indices.insert(gn, idx);
        }
    }

    // 2. Add all shuttles (including under construction)
    for (i, _shuttle) in loom.persistent.shuttles.iter().enumerate() {
        let gn = LoomGraphNode::Shuttle(i);
        let idx = graph.add_node(gn.clone());
        node_indices.insert(gn, idx);
    }

    // 3. Add visible pattern sinks
    for pat_idx in visible_pattern_indices(loom) {
        let gn = LoomGraphNode::PatternSink(pat_idx);
        let idx = graph.add_node(gn.clone());
        node_indices.insert(gn, idx);
    }

    // 4. Add edges from shuttle sources to shuttles
    for (shuttle_idx, shuttle) in loom.persistent.shuttles.iter().enumerate() {
        let shuttle_node = node_indices
            .get(&LoomGraphNode::Shuttle(shuttle_idx))
            .copied();
        let shuttle_ni = match shuttle_node {
            Some(ni) => ni,
            None => continue,
        };

        let num_sources_a = shuttle.sources_a.len().max(1) as f64;
        let max_rate_a = shuttle_effective_intake_cap(shuttle.tier, shuttle.level) / num_sources_a;

        for source_ref in &shuttle.sources_a {
            let source_gn = match source_ref {
                super::types::LoomNodeRef::Extractor(nid) => LoomGraphNode::Extractor(*nid),
                super::types::LoomNodeRef::Shuttle(idx) => LoomGraphNode::Shuttle(*idx),
            };
            if let Some(&source_ni) = node_indices.get(&source_gn) {
                let resource = match source_ref {
                    super::types::LoomNodeRef::Extractor(nid) => node_native_resource(*nid),
                    super::types::LoomNodeRef::Shuttle(idx) => {
                        loom.persistent.shuttles[*idx].output
                    }
                };
                graph.add_edge(
                    source_ni,
                    shuttle_ni,
                    LoomEdge {
                        resource,
                        current_rate: 0.0,
                        max_rate: max_rate_a,
                    },
                );
            }
        }

        let num_sources_b = shuttle.sources_b.len().max(1) as f64;
        let max_rate_b = shuttle_effective_intake_cap(shuttle.tier, shuttle.level) / num_sources_b;

        for source_ref in &shuttle.sources_b {
            let source_gn = match source_ref {
                super::types::LoomNodeRef::Extractor(nid) => LoomGraphNode::Extractor(*nid),
                super::types::LoomNodeRef::Shuttle(idx) => LoomGraphNode::Shuttle(*idx),
            };
            if let Some(&source_ni) = node_indices.get(&source_gn) {
                let resource = match source_ref {
                    super::types::LoomNodeRef::Extractor(nid) => node_native_resource(*nid),
                    super::types::LoomNodeRef::Shuttle(idx) => {
                        loom.persistent.shuttles[*idx].output
                    }
                };
                graph.add_edge(
                    source_ni,
                    shuttle_ni,
                    LoomEdge {
                        resource,
                        current_rate: 0.0,
                        max_rate: max_rate_b,
                    },
                );
            }
        }
    }

    // 5. Add inferred edges from shuttles to pattern sinks
    for pat_idx in visible_pattern_indices(loom) {
        let sink_ni = match node_indices.get(&LoomGraphNode::PatternSink(pat_idx)) {
            Some(&ni) => ni,
            None => continue,
        };
        let pattern = &loom.persistent.patterns[pat_idx];

        for req in &pattern.requirements {
            if req.completed {
                continue;
            }
            // Find shuttles that produce this resource
            for (shuttle_idx, shuttle) in loom.persistent.shuttles.iter().enumerate() {
                if shuttle.output == req.resource {
                    if let Some(&shuttle_ni) =
                        node_indices.get(&LoomGraphNode::Shuttle(shuttle_idx))
                    {
                        graph.add_edge(
                            shuttle_ni,
                            sink_ni,
                            LoomEdge {
                                resource: req.resource,
                                current_rate: 0.0,
                                max_rate: 0.0, // inferred edges have no intake cap
                            },
                        );
                    }
                }
            }
        }
    }

    LoomGraph {
        graph,
        node_indices,
    }
}

/// Returns indices of patterns visible in the graph view.
///
/// Shows only the active pattern (if not completed) — at most 1 sink node.
pub fn visible_pattern_indices(loom: &LoomState) -> Vec<usize> {
    let patterns = &loom.persistent.patterns;
    if patterns.is_empty() {
        return Vec::new();
    }
    let active = loom.persistent.active_pattern;
    if active < patterns.len() && !patterns[active].completed {
        vec![active]
    } else {
        Vec::new()
    }
}

/// Insert a temporary ghost node for build preview.
/// Uses usize::MAX as sentinel index to indicate a not-yet-built shuttle.
/// Called by `loom_input` build flow once the ghost-node TODO is implemented.
#[allow(dead_code)]
pub fn insert_ghost_node(
    lg: &mut LoomGraph,
    _tier: u8,
    sources: &[super::types::LoomNodeRef],
    output_resource: super::types::Resource,
) -> NodeIndex {
    let ghost = LoomGraphNode::Shuttle(usize::MAX);
    let idx = lg.graph.add_node(ghost.clone());
    lg.node_indices.insert(ghost, idx);

    // Add edges from sources to ghost node
    for source_ref in sources {
        let source_gn = match source_ref {
            super::types::LoomNodeRef::Extractor(id) => LoomGraphNode::Extractor(*id),
            super::types::LoomNodeRef::Shuttle(i) => LoomGraphNode::Shuttle(*i),
        };
        if let Some(&source_idx) = lg.node_indices.get(&source_gn) {
            lg.graph.add_edge(
                source_idx,
                idx,
                LoomEdge {
                    resource: output_resource,
                    current_rate: 0.0,
                    max_rate: 0.0,
                },
            );
        }
    }
    idx
}

/// Remove the ghost node and its edges.
#[allow(dead_code)]
pub fn remove_ghost_node(lg: &mut LoomGraph, ghost_idx: NodeIndex) {
    lg.graph.remove_node(ghost_idx);
    lg.node_indices.retain(|_, &mut v| v != ghost_idx);
}

/// Update edge current_rate values from live rate trackers.
///
/// - Edges from extractors: use the resource's global rate tracker
/// - Edges from shuttles: use the shuttle's output_rate_tracker
/// - Edges from pattern sinks (if any): 0.0
pub fn update_edge_rates(lg: &mut LoomGraph, loom: &LoomState) {
    let edge_indices: Vec<_> = lg.graph.edge_indices().collect();
    for edge_idx in edge_indices {
        let (source_ni, _target_ni) = match lg.graph.edge_endpoints(edge_idx) {
            Some(endpoints) => endpoints,
            None => continue,
        };
        let source_node = match lg.graph.node_weight(source_ni) {
            Some(n) => n.clone(),
            None => continue,
        };

        let rate = match &source_node {
            LoomGraphNode::Extractor(node_id) => {
                let resource = node_native_resource(*node_id);
                loom.rate_trackers
                    .get(&resource)
                    .map(|t| t.rate_per_hour())
                    .unwrap_or(0.0)
            }
            LoomGraphNode::Shuttle(idx) => {
                if *idx < loom.persistent.shuttles.len() {
                    loom.persistent.shuttles[*idx]
                        .output_rate_tracker
                        .rate_per_hour()
                } else {
                    0.0
                }
            }
            LoomGraphNode::PatternSink(_) => 0.0,
        };

        if let Some(edge) = lg.graph.edge_weight_mut(edge_idx) {
            edge.current_rate = rate;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loom::types::*;

    /// Helper: create a minimal LoomState with N unlocked extractors.
    fn loom_with_unlocked_extractors(count: usize) -> LoomState {
        let mut state = LoomState::new();
        for (i, node) in state.persistent.nodes.iter_mut().enumerate() {
            node.unlocked = i < count;
        }
        state
    }

    #[test]
    fn test_empty_loom_has_only_unlocked_extractors() {
        let loom = loom_with_unlocked_extractors(1);
        let lg = build_graph(&loom);

        assert_eq!(lg.graph.node_count(), 1, "should have exactly 1 node");
        assert_eq!(lg.graph.edge_count(), 0, "should have 0 edges");

        // Verify the node is the first extractor
        let node = lg.graph.node_weight(NodeIndex::new(0)).unwrap();
        assert_eq!(*node, LoomGraphNode::Extractor(NodeId::EmberSpindle));
    }

    #[test]
    fn test_shuttle_creates_edges_from_sources() {
        let mut loom = loom_with_unlocked_extractors(6);

        // Add a T1 shuttle with 2 sources (one per input slot)
        let shuttle = Shuttle::new(
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            Resource::ForgedLight,
            1.0,
            1,
            vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
            vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
        );
        loom.persistent.shuttles.push(shuttle);

        let lg = build_graph(&loom);

        // 6 extractors + 1 shuttle = 7 nodes
        assert_eq!(lg.graph.node_count(), 7, "should have 7 nodes");
        // 1 edge from EmberSpindle -> Shuttle, 1 from VoidCondenser -> Shuttle
        assert_eq!(lg.graph.edge_count(), 2, "should have 2 edges");

        // Verify edge resources
        let shuttle_ni = lg.node_indices[&LoomGraphNode::Shuttle(0)];
        let incoming: Vec<_> = lg
            .graph
            .edges_directed(shuttle_ni, petgraph::Direction::Incoming)
            .collect();
        assert_eq!(incoming.len(), 2);

        let resources: Vec<Resource> = incoming.iter().map(|e| e.weight().resource).collect();
        assert!(resources.contains(&Resource::Ember));
        assert!(resources.contains(&Resource::VoidEssence));
    }

    #[test]
    fn test_pattern_sink_gets_inferred_edges() {
        let mut loom = loom_with_unlocked_extractors(6);

        // Add a shuttle producing ForgedLight
        let shuttle = Shuttle::new(
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            Resource::ForgedLight,
            1.0,
            1,
            vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
            vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
        );
        loom.persistent.shuttles.push(shuttle);

        // Add a pattern requiring ForgedLight
        loom.persistent.patterns.push(WovenPattern {
            index: 0,
            name: "Test Pattern".to_string(),
            requirements: vec![PatternRequirement {
                resource: Resource::ForgedLight,
                required_rate: 10.0,
                sustain_duration_secs: 3600.0,
                sustained_secs: 0.0,
                completed: false,
                amount: 0.0,
                accumulated: 0.0,
            }],
            completed: false,
        });
        loom.persistent.active_pattern = 0;

        let lg = build_graph(&loom);

        // 6 extractors + 1 shuttle + 1 pattern sink = 8 nodes
        assert_eq!(lg.graph.node_count(), 8, "should have 8 nodes");

        // 2 source edges to shuttle + 1 inferred edge from shuttle to pattern sink = 3
        assert_eq!(lg.graph.edge_count(), 3, "should have 3 edges");

        // Verify the inferred edge
        let sink_ni = lg.node_indices[&LoomGraphNode::PatternSink(0)];
        let incoming: Vec<_> = lg
            .graph
            .edges_directed(sink_ni, petgraph::Direction::Incoming)
            .collect();
        assert_eq!(incoming.len(), 1);
        assert_eq!(incoming[0].weight().resource, Resource::ForgedLight);
    }

    #[test]
    fn test_visible_pattern_indices_empty() {
        let loom = LoomState::new();
        assert!(visible_pattern_indices(&loom).is_empty());
    }

    #[test]
    fn test_visible_pattern_indices_only_active() {
        let mut loom = LoomState::new();
        for i in 0..5 {
            loom.persistent.patterns.push(WovenPattern {
                index: i,
                name: format!("Pattern {}", i),
                requirements: vec![],
                completed: false,
            });
        }
        loom.persistent.active_pattern = 0;
        let visible = visible_pattern_indices(&loom);
        assert_eq!(visible.len(), 1, "only active pattern shown");
        assert_eq!(visible, vec![0]);
    }

    #[test]
    fn test_visible_pattern_indices_skips_completed() {
        let mut loom = LoomState::new();
        for i in 0..4 {
            loom.persistent.patterns.push(WovenPattern {
                index: i,
                name: format!("Pattern {}", i),
                requirements: vec![],
                completed: i == 0, // first is completed
            });
        }
        loom.persistent.active_pattern = 0;
        // active (0) is completed so not shown; no sink nodes visible
        let visible = visible_pattern_indices(&loom);
        assert!(visible.is_empty(), "completed active pattern not shown");
    }

    #[test]
    fn test_update_edge_rates_from_tracker() {
        let mut loom = loom_with_unlocked_extractors(6);

        let shuttle = Shuttle::new(
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            Resource::ForgedLight,
            1.0,
            1,
            vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
            vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
        );
        loom.persistent.shuttles.push(shuttle);

        // Populate rate tracker for Ember
        let tracker = loom.rate_trackers.entry(Resource::Ember).or_default();
        // Push enough to get a non-zero rate
        for _ in 0..10 {
            tracker.push(1.0);
        }

        let mut lg = build_graph(&loom);
        update_edge_rates(&mut lg, &loom);

        // Find the edge from EmberSpindle to Shuttle
        let ember_ni = lg.node_indices[&LoomGraphNode::Extractor(NodeId::EmberSpindle)];
        let outgoing: Vec<_> = lg
            .graph
            .edges_directed(ember_ni, petgraph::Direction::Outgoing)
            .collect();
        assert_eq!(outgoing.len(), 1);
        assert!(
            outgoing[0].weight().current_rate > 0.0,
            "edge rate should be non-zero after tracker update"
        );
    }
}
