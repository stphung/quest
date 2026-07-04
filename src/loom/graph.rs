// Loom graph data layer — DAG representation of the production network.
// Uses petgraph's StableDiGraph to model extractors, shuttles, and pattern sinks
// with edges representing resource flows and their rates.

use petgraph::stable_graph::{NodeIndex, StableDiGraph};
use std::collections::HashMap;

use super::layout::compute_layout;
use super::logic::{node_effective_rate, node_native_resource};
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

    // Advance particle phases.
    // Uses sqrt easing so low rates are visible and high rates don't get frenetic.
    // Base step 0.02 * sqrt(utilization) = ~4-5 second cycle at full rate.
    if let Some(ref graph) = ui.loom_graph {
        for edge_idx in graph.graph.edge_indices() {
            let edge = &graph.graph[edge_idx];
            let utilization = if edge.max_rate > 0.0 {
                (edge.current_rate / edge.max_rate).clamp(0.0, 1.5)
            } else {
                0.0
            };
            let speed = utilization.sqrt(); // easing: diminishing returns at high rates
            if let Some(phase) = ui.particle_phases.get_mut(&edge_idx) {
                *phase = (*phase + 0.02 * speed) % 1.0;
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

        for source_ref in &shuttle.sources_a {
            let source_gn = match source_ref {
                super::types::LoomNodeRef::Extractor(nid) => LoomGraphNode::Extractor(*nid),
                super::types::LoomNodeRef::Shuttle(idx) => LoomGraphNode::Shuttle(*idx),
            };
            if let Some(&source_ni) = node_indices.get(&source_gn) {
                let resource = match source_ref {
                    super::types::LoomNodeRef::Extractor(nid) => node_native_resource(*nid),
                    super::types::LoomNodeRef::Shuttle(idx) => {
                        match loom.persistent.shuttles.get(*idx) {
                            Some(s) => s.output,
                            None => continue, // Stale reference — skip edge
                        }
                    }
                };
                // max_rate = source effective rate / number of sources in this slot
                let max_rate_a = match source_ref {
                    super::types::LoomNodeRef::Extractor(nid) => {
                        let node = &loom.persistent.nodes[nid.index()];
                        node_effective_rate(loom, node) / num_sources_a
                    }
                    super::types::LoomNodeRef::Shuttle(_) => 0.0, // estimated at update time
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

        for source_ref in &shuttle.sources_b {
            let source_gn = match source_ref {
                super::types::LoomNodeRef::Extractor(nid) => LoomGraphNode::Extractor(*nid),
                super::types::LoomNodeRef::Shuttle(idx) => LoomGraphNode::Shuttle(*idx),
            };
            if let Some(&source_ni) = node_indices.get(&source_gn) {
                let resource = match source_ref {
                    super::types::LoomNodeRef::Extractor(nid) => node_native_resource(*nid),
                    super::types::LoomNodeRef::Shuttle(idx) => {
                        match loom.persistent.shuttles.get(*idx) {
                            Some(s) => s.output,
                            None => continue, // Stale reference — skip edge
                        }
                    }
                };
                // max_rate = source effective rate / number of sources in this slot
                let max_rate_b = match source_ref {
                    super::types::LoomNodeRef::Extractor(nid) => {
                        let node = &loom.persistent.nodes[nid.index()];
                        node_effective_rate(loom, node) / num_sources_b
                    }
                    super::types::LoomNodeRef::Shuttle(_) => 0.0, // estimated at update time
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
            // Skip completed requirements — except for eternal patterns, which
            // always show their resource flow (the edge IS the visualization).
            if req.completed && !pattern.eternal {
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
                                // max_rate = 0 means update_edge_rates will set it
                                // dynamically. See particle speed fix below.
                                max_rate: 0.0,
                            },
                        );
                    }
                }
            }

            // Also add extractor→pattern edges for base resources not covered by shuttles
            let shuttle_covers = loom
                .persistent
                .shuttles
                .iter()
                .any(|s| s.output == req.resource);
            if !shuttle_covers {
                // Find extractors that natively produce this resource
                for node in &loom.persistent.nodes {
                    if node.unlocked && node_native_resource(node.id) == req.resource {
                        if let Some(&extractor_ni) =
                            node_indices.get(&LoomGraphNode::Extractor(node.id))
                        {
                            graph.add_edge(
                                extractor_ni,
                                sink_ni,
                                LoomEdge {
                                    resource: req.resource,
                                    current_rate: 0.0,
                                    max_rate: 0.0, // updated dynamically per tick
                                },
                            );
                        }
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

/// Update edge rates using contention-aware flow computation.
///
/// Computes actual per-edge flow rates considering:
/// - Source production rates (extractor base rate or shuttle output rate)
/// - Number of consumers sharing each source
/// - Intake cap per shuttle tier/level
///
/// All rates are normalized to un-warped (real-time) values.
pub fn update_edge_rates(lg: &mut LoomGraph, loom: &LoomState) {
    use super::types::LoomNodeRef;

    let persistent = &loom.persistent;

    // Step 1: Count consumers per source (same as tick_shuttle_pull).
    let mut consumer_count: HashMap<LoomNodeRef, usize> = HashMap::new();
    for shuttle in &persistent.shuttles {
        if shuttle.under_construction {
            continue;
        }
        for &src in shuttle.sources_a.iter().chain(shuttle.sources_b.iter()) {
            *consumer_count.entry(src).or_insert(0) += 1;
        }
    }

    // Step 2: Compute source production rates (un-warped).
    // Extractors: use node_effective_rate (base * level_mult * throughput_mult).
    // Shuttles: use output_rate_tracker / warp.
    let source_rates: HashMap<LoomNodeRef, f64> = {
        let mut rates = HashMap::new();
        for node in &persistent.nodes {
            if node.unlocked {
                let rate = if node.upgrading {
                    0.0 // No production while upgrading.
                } else {
                    super::logic::node_effective_rate(loom, node)
                };
                rates.insert(LoomNodeRef::Extractor(node.id), rate);
            }
        }
        for (i, shuttle) in persistent.shuttles.iter().enumerate() {
            let rate = if shuttle.under_construction {
                0.0
            } else {
                // output_rate_tracker already stores un-warped values
                // (divided by warp on push in tick_shuttle_pull).
                shuttle.output_rate_tracker.rate_per_hour()
            };
            rates.insert(LoomNodeRef::Shuttle(i), rate);
        }
        rates
    };

    // Step 3: For each edge, compute the actual flow rate.
    let edge_indices: Vec<_> = lg.graph.edge_indices().collect();
    for edge_idx in edge_indices {
        let (source_ni, target_ni) = match lg.graph.edge_endpoints(edge_idx) {
            Some(ep) => ep,
            None => continue,
        };
        let source_node = match lg.graph.node_weight(source_ni) {
            Some(n) => n.clone(),
            None => continue,
        };
        let target_node = match lg.graph.node_weight(target_ni) {
            Some(n) => n.clone(),
            None => continue,
        };

        let rate = match (&source_node, &target_node) {
            // Extractor/Shuttle → Shuttle: use contention-aware flow.
            // Under-construction shuttles can't pull — zero flow on all incoming edges.
            (_, LoomGraphNode::Shuttle(shuttle_idx))
                if *shuttle_idx < persistent.shuttles.len() =>
            {
                let shuttle = &persistent.shuttles[*shuttle_idx];
                if shuttle.under_construction {
                    0.0
                } else {
                    let src_ref = match &source_node {
                        LoomGraphNode::Extractor(id) => LoomNodeRef::Extractor(*id),
                        LoomGraphNode::Shuttle(idx) => LoomNodeRef::Shuttle(*idx),
                        _ => continue,
                    };
                    let available = source_rates.get(&src_ref).copied().unwrap_or(0.0);
                    let consumers = consumer_count.get(&src_ref).copied().unwrap_or(1).max(1);
                    available / consumers as f64
                }
            }
            // Anything → PatternSink: use the source's full production rate.
            // Patterns measure aggregate rates via rate_trackers, not buffer consumption.
            // The rate shown = what the pattern's sustain logic actually sees.
            (_, LoomGraphNode::PatternSink(_)) => {
                let src_ref = match &source_node {
                    LoomGraphNode::Extractor(id) => LoomNodeRef::Extractor(*id),
                    LoomGraphNode::Shuttle(idx) => LoomNodeRef::Shuttle(*idx),
                    _ => continue,
                };
                source_rates.get(&src_ref).copied().unwrap_or(0.0)
            }
            _ => 0.0,
        };

        if let Some(edge) = lg.graph.edge_weight_mut(edge_idx) {
            edge.current_rate = rate;
            // Keep max_rate in sync for edges that were initialized with max_rate=0
            // (shuttle→shuttle, pattern sink edges). This ensures the utilization
            // ratio (current/max) stays ~1.0 so particles flow at full speed.
            if edge.max_rate < 0.1 || matches!(&target_node, LoomGraphNode::PatternSink(_)) {
                edge.max_rate = rate.max(0.1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loom::types::*;
    use petgraph::visit::EdgeRef;

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
            flavor: String::new(),
            eternal: false,
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
    fn test_extractor_pattern_edge_for_base_resource() {
        let mut loom = loom_with_unlocked_extractors(6);

        // Pattern requiring Ember (a base resource) with no shuttle producing Ember
        loom.persistent.patterns.push(WovenPattern {
            index: 0,
            name: "Base Test".to_string(),
            requirements: vec![PatternRequirement {
                resource: Resource::Ember,
                required_rate: 5.0,
                sustain_duration_secs: 1800.0,
                sustained_secs: 0.0,
                completed: false,
                amount: 0.0,
                accumulated: 0.0,
            }],
            completed: false,
            flavor: String::new(),
            eternal: false,
        });
        loom.persistent.active_pattern = 0;

        let lg = build_graph(&loom);

        // 6 extractors + 1 pattern sink = 7 nodes
        assert_eq!(lg.graph.node_count(), 7, "should have 7 nodes");

        // 1 inferred edge from EmberSpindle extractor to pattern sink
        assert_eq!(
            lg.graph.edge_count(),
            1,
            "should have 1 extractor->pattern edge"
        );

        let sink_ni = lg.node_indices[&LoomGraphNode::PatternSink(0)];
        let incoming: Vec<_> = lg
            .graph
            .edges_directed(sink_ni, petgraph::Direction::Incoming)
            .collect();
        assert_eq!(incoming.len(), 1);
        assert_eq!(incoming[0].weight().resource, Resource::Ember);
    }

    #[test]
    fn test_extractor_pattern_edge_not_added_when_shuttle_covers() {
        let mut loom = loom_with_unlocked_extractors(6);

        // Add a shuttle that produces Ember
        let shuttle = Shuttle::new(
            Resource::Reflection,
            Resource::VoidEssence,
            NodeNature::Heat,
            Resource::Ember, // shuttle output is Ember
            1.0,
            1,
            vec![LoomNodeRef::Extractor(NodeId::ReflectionLens)],
            vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
        );
        loom.persistent.shuttles.push(shuttle);

        // Pattern requiring Ember — shuttle covers it, so no extractor edge
        loom.persistent.patterns.push(WovenPattern {
            index: 0,
            name: "Covered Test".to_string(),
            requirements: vec![PatternRequirement {
                resource: Resource::Ember,
                required_rate: 5.0,
                sustain_duration_secs: 1800.0,
                sustained_secs: 0.0,
                completed: false,
                amount: 0.0,
                accumulated: 0.0,
            }],
            completed: false,
            flavor: String::new(),
            eternal: false,
        });
        loom.persistent.active_pattern = 0;

        let lg = build_graph(&loom);

        let sink_ni = lg.node_indices[&LoomGraphNode::PatternSink(0)];
        let incoming: Vec<_> = lg
            .graph
            .edges_directed(sink_ni, petgraph::Direction::Incoming)
            .collect();
        // Only the shuttle->pattern edge, no extractor->pattern edge
        assert_eq!(incoming.len(), 1);
        assert_eq!(incoming[0].weight().resource, Resource::Ember);

        // Verify it comes from shuttle, not extractor
        let source_ni = incoming[0].source();
        let source_node = lg.graph.node_weight(source_ni).unwrap();
        assert!(matches!(source_node, LoomGraphNode::Shuttle(0)));
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
                flavor: String::new(),
                eternal: false,
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
                flavor: String::new(),
                eternal: false,
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

    #[test]
    fn test_update_edge_rates_zero_for_upgrading_extractor() {
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
        loom.persistent.nodes[NodeId::EmberSpindle.index()].upgrading = true;

        let mut lg = build_graph(&loom);
        update_edge_rates(&mut lg, &loom);

        let ember_ni = lg.node_indices[&LoomGraphNode::Extractor(NodeId::EmberSpindle)];
        let outgoing: Vec<_> = lg
            .graph
            .edges_directed(ember_ni, petgraph::Direction::Outgoing)
            .collect();
        assert_eq!(outgoing.len(), 1);
        assert_eq!(
            outgoing[0].weight().current_rate,
            0.0,
            "upgrading extractor should produce zero flow"
        );
    }

    #[test]
    fn test_update_edge_rates_zero_for_under_construction_shuttle() {
        let mut loom = loom_with_unlocked_extractors(6);
        let mut shuttle = Shuttle::new(
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            Resource::ForgedLight,
            1.0,
            1,
            vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
            vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
        );
        shuttle.under_construction = true;
        loom.persistent.shuttles.push(shuttle);

        let mut lg = build_graph(&loom);
        update_edge_rates(&mut lg, &loom);

        let shuttle_ni = lg.node_indices[&LoomGraphNode::Shuttle(0)];
        let incoming: Vec<_> = lg
            .graph
            .edges_directed(shuttle_ni, petgraph::Direction::Incoming)
            .collect();
        assert_eq!(incoming.len(), 2);
        for edge in incoming {
            assert_eq!(
                edge.weight().current_rate,
                0.0,
                "shuttle under construction should not pull any flow"
            );
        }
    }

    #[test]
    fn test_update_edge_rates_contention_splits_shared_source() {
        let mut loom = loom_with_unlocked_extractors(6);
        // Two T1 shuttles both pull input A from the same extractor.
        let shuttle_a = Shuttle::new(
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            Resource::ForgedLight,
            1.0,
            1,
            vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
            vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
        );
        let shuttle_b = Shuttle::new(
            Resource::Ember,
            Resource::Memory,
            NodeNature::Heat,
            Resource::EchoGlass,
            1.0,
            1,
            vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
            vec![LoomNodeRef::Extractor(NodeId::MemoryArchive)],
        );
        loom.persistent.shuttles.push(shuttle_a);
        loom.persistent.shuttles.push(shuttle_b);

        let mut lg = build_graph(&loom);
        update_edge_rates(&mut lg, &loom);

        let ember_ni = lg.node_indices[&LoomGraphNode::Extractor(NodeId::EmberSpindle)];
        let outgoing: Vec<_> = lg
            .graph
            .edges_directed(ember_ni, petgraph::Direction::Outgoing)
            .collect();
        assert_eq!(outgoing.len(), 2, "both shuttles pull from EmberSpindle");
        let rates: Vec<f64> = outgoing.iter().map(|e| e.weight().current_rate).collect();
        assert!(
            (rates[0] - rates[1]).abs() < 1e-9,
            "contention should split the source evenly: {:?}",
            rates
        );
    }

    #[test]
    fn test_update_edge_rates_pattern_sink_uses_full_source_rate() {
        let mut loom = loom_with_unlocked_extractors(6);
        loom.persistent.patterns.push(WovenPattern {
            index: 0,
            name: "Sink Test".to_string(),
            requirements: vec![PatternRequirement {
                resource: Resource::Ember,
                required_rate: 5.0,
                sustain_duration_secs: 100.0,
                sustained_secs: 0.0,
                completed: false,
                amount: 0.0,
                accumulated: 0.0,
            }],
            completed: false,
            flavor: String::new(),
            eternal: false,
        });
        loom.persistent.active_pattern = 0;

        let mut lg = build_graph(&loom);
        update_edge_rates(&mut lg, &loom);

        let sink_ni = lg.node_indices[&LoomGraphNode::PatternSink(0)];
        let incoming: Vec<_> = lg
            .graph
            .edges_directed(sink_ni, petgraph::Direction::Incoming)
            .collect();
        assert_eq!(incoming.len(), 1);
        // Full extractor rate should flow into the pattern sink (not divided
        // by any consumer count), and max_rate should track it (>= 0.1 floor).
        let edge = incoming[0].weight();
        assert!(edge.current_rate > 0.0);
        assert!(edge.max_rate >= 0.1);
    }

    #[test]
    fn test_refresh_graph_builds_on_first_call() {
        let mut loom = loom_with_unlocked_extractors(6);
        let mut ui = LoomUiState::new();
        assert!(ui.loom_graph.is_none());

        refresh_graph(&mut ui, &mut loom, 400.0, 300.0);

        assert!(ui.loom_graph.is_some());
        assert!(ui.loom_layout.is_some());
        assert!(!ui.graph_dirty);
        assert!(!loom.graph_dirty);
        assert!(
            ui.selected_graph_node.is_some(),
            "a default selection should be assigned"
        );
        assert_eq!(ui.particle_phases.len(), 0, "no edges yet without shuttles");
    }

    #[test]
    fn test_refresh_graph_skips_rebuild_when_clean() {
        let mut loom = loom_with_unlocked_extractors(6);
        let mut ui = LoomUiState::new();
        refresh_graph(&mut ui, &mut loom, 400.0, 300.0);

        let selected_before = ui.selected_graph_node;
        // Neither dirty flag set and graph already present => should not rebuild,
        // just update rates/particles.
        refresh_graph(&mut ui, &mut loom, 400.0, 300.0);
        assert_eq!(ui.selected_graph_node, selected_before);
    }

    #[test]
    fn test_refresh_graph_rebuilds_when_loom_flag_dirty() {
        let mut loom = loom_with_unlocked_extractors(6);
        let mut ui = LoomUiState::new();
        refresh_graph(&mut ui, &mut loom, 400.0, 300.0);

        loom.graph_dirty = true;
        refresh_graph(&mut ui, &mut loom, 400.0, 300.0);
        assert!(!loom.graph_dirty, "flag should be cleared after rebuild");
    }

    #[test]
    fn test_refresh_graph_resets_invalid_selection() {
        let mut loom = loom_with_unlocked_extractors(6);
        let mut ui = LoomUiState::new();
        refresh_graph(&mut ui, &mut loom, 400.0, 300.0);

        // Point selection at a node index that cannot exist in a freshly
        // rebuilt graph (only 6 nodes exist).
        ui.selected_graph_node = Some(NodeIndex::new(999));
        ui.graph_dirty = true;
        refresh_graph(&mut ui, &mut loom, 400.0, 300.0);

        let selected = ui.selected_graph_node.expect("should reassign a default");
        assert!(lg_contains(&ui, selected));
    }

    fn lg_contains(ui: &LoomUiState, ni: NodeIndex) -> bool {
        ui.loom_graph
            .as_ref()
            .map(|g| g.graph.node_weight(ni).is_some())
            .unwrap_or(false)
    }

    #[test]
    fn test_refresh_graph_advances_particle_phase_with_flow() {
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
        let mut ui = LoomUiState::new();

        refresh_graph(&mut ui, &mut loom, 400.0, 300.0);
        assert!(
            !ui.particle_phases.is_empty(),
            "edges should have particle phase entries"
        );
        let phase_before: f64 = ui.particle_phases.values().copied().sum();

        // Second call without any dirty flags: rates recompute and phases advance.
        refresh_graph(&mut ui, &mut loom, 400.0, 300.0);
        let phase_after: f64 = ui.particle_phases.values().copied().sum();
        assert!(
            phase_after >= phase_before,
            "particle phase should not decrease: before={} after={}",
            phase_before,
            phase_after
        );
    }
}
