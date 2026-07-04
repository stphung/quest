// Loom graph layout engine — Sugiyama-style layered DAG layout.
// Assigns nodes to layers by type/tier, minimizes edge crossings via
// barycenter heuristic, then assigns (x, y) coordinates for rendering.

use petgraph::stable_graph::NodeIndex;
use petgraph::Direction;
use std::collections::HashMap;

use super::graph::{LoomGraph, LoomGraphNode};
use super::types::LoomState;

/// Computed layout positions for every node in a LoomGraph.
#[derive(Debug)]
pub struct LoomLayout {
    /// Map from petgraph NodeIndex to (x, y) position in layout space.
    pub node_positions: HashMap<NodeIndex, (f64, f64)>,
    /// For long edges spanning >1 layer, intermediate waypoints keyed by (source, target).
    pub dummy_paths: HashMap<(NodeIndex, NodeIndex), Vec<(f64, f64)>>,
    /// Bounding box (width, height) of the layout.
    pub bounds: (f64, f64),
}

/// Determine the layer (column) for a graph node.
/// Extractors=0, T1 shuttle=1, T2 shuttle=2, T3 shuttle=3, PatternSinks=4.
pub fn node_layer(node: &LoomGraphNode, loom: &LoomState) -> usize {
    match node {
        LoomGraphNode::Extractor(_) => 0,
        LoomGraphNode::Shuttle(idx) => loom
            .persistent
            .shuttles
            .get(*idx)
            .map(|s| s.tier as usize)
            .unwrap_or(1),
        LoomGraphNode::PatternSink(_) => 4,
    }
}

/// Compute a Sugiyama-style layered layout for the Loom production graph.
///
/// The layout proceeds in four phases:
/// 1. Layer assignment (by node type / shuttle tier)
/// 2. Crossing minimization (barycenter heuristic, 3 sweeps)
/// 3. Coordinate assignment (even spacing with margins)
/// 4. Dummy path generation (linear interpolation for long edges)
pub fn compute_layout(lg: &LoomGraph, loom: &LoomState, width: f64, height: f64) -> LoomLayout {
    let node_count = lg.graph.node_count();

    // Empty graph → empty layout
    if node_count == 0 {
        return LoomLayout {
            node_positions: HashMap::new(),
            dummy_paths: HashMap::new(),
            bounds: (width, height),
        };
    }

    // Collect all node indices
    let all_nodes: Vec<NodeIndex> = lg.graph.node_indices().collect();

    // Single node → center it
    if all_nodes.len() == 1 {
        let mut positions = HashMap::new();
        positions.insert(all_nodes[0], (width / 2.0, height / 2.0));
        return LoomLayout {
            node_positions: positions,
            dummy_paths: HashMap::new(),
            bounds: (width, height),
        };
    }

    // --- Phase 1: Layer assignment ---
    let mut layer_nodes: Vec<Vec<NodeIndex>> = vec![Vec::new(); 5];
    for &ni in &all_nodes {
        if let Some(node) = lg.graph.node_weight(ni) {
            let layer = node_layer(node, loom);
            layer_nodes[layer].push(ni);
        }
    }

    // --- Phase 2: Crossing minimization (barycenter heuristic) ---
    // 3 sweeps; odd sweeps also do a backward pass.
    for sweep in 0..3 {
        // Forward pass: layers 1..4, reorder by incoming neighbor positions
        for l in 1..5 {
            if layer_nodes[l].is_empty() || layer_nodes[l - 1].is_empty() {
                continue;
            }
            let prev_positions = index_positions(&layer_nodes[l - 1]);
            barycenter_reorder(
                &mut layer_nodes[l],
                &prev_positions,
                &lg.graph,
                Direction::Incoming,
            );
        }

        // Backward pass on odd sweeps: layers 3..0, reorder by outgoing neighbor positions
        // Skip layer 0 (extractors keep fixed order).
        if sweep % 2 == 1 {
            for l in (1..4).rev() {
                if layer_nodes[l].is_empty() || layer_nodes[l + 1].is_empty() {
                    continue;
                }
                let next_positions = index_positions(&layer_nodes[l + 1]);
                barycenter_reorder(
                    &mut layer_nodes[l],
                    &next_positions,
                    &lg.graph,
                    Direction::Outgoing,
                );
            }
        }
    }

    // --- Phase 3: Coordinate assignment ---
    let active_layers: Vec<usize> = (0..5).filter(|l| !layer_nodes[*l].is_empty()).collect();
    let num_active = active_layers.len();

    // X positions for each active layer.
    // Generous margins to prevent long gauge-bar labels from clipping at edges.
    // Right margin is larger because pattern sink labels tend to be the widest.
    let margin_left = width * 0.12;
    let margin_right = width * 0.18;
    let usable_w = width - margin_left - margin_right;
    let layer_x: HashMap<usize, f64> = if num_active == 1 {
        let mut m = HashMap::new();
        m.insert(active_layers[0], width / 2.0);
        m
    } else {
        active_layers
            .iter()
            .enumerate()
            .map(|(i, &l)| {
                let x = margin_left + usable_w * (i as f64) / (num_active as f64 - 1.0);
                (l, x)
            })
            .collect()
    };

    // Y positions for nodes within each layer
    let margin_y = height * 0.10;
    let usable_h = height - 2.0 * margin_y;
    let mut node_positions = HashMap::new();

    for l in 0..5 {
        let nodes = &layer_nodes[l];
        if nodes.is_empty() {
            continue;
        }
        let x = layer_x[&l];
        let count = nodes.len();
        for (i, &ni) in nodes.iter().enumerate() {
            let y = if count == 1 {
                height / 2.0
            } else {
                margin_y + usable_h * (i as f64) / (count as f64 - 1.0)
            };
            node_positions.insert(ni, (x, y));
        }
    }

    // --- Phase 4: Dummy paths for long edges ---
    let mut dummy_paths = HashMap::new();
    let node_layers: HashMap<NodeIndex, usize> = all_nodes
        .iter()
        .filter_map(|&ni| lg.graph.node_weight(ni).map(|n| (ni, node_layer(n, loom))))
        .collect();

    for edge_idx in lg.graph.edge_indices() {
        if let Some((src, tgt)) = lg.graph.edge_endpoints(edge_idx) {
            let src_layer = match node_layers.get(&src) {
                Some(&l) => l,
                None => continue,
            };
            let tgt_layer = match node_layers.get(&tgt) {
                Some(&l) => l,
                None => continue,
            };
            let span = if tgt_layer > src_layer {
                tgt_layer - src_layer
            } else {
                continue; // skip backward or same-layer edges
            };
            if span <= 1 {
                continue;
            }

            let src_pos = node_positions[&src];
            let tgt_pos = node_positions[&tgt];
            let mut waypoints = Vec::with_capacity(span - 1);
            for step in 1..span {
                let t = step as f64 / span as f64;
                let x = src_pos.0 + (tgt_pos.0 - src_pos.0) * t;
                let y = src_pos.1 + (tgt_pos.1 - src_pos.1) * t;
                waypoints.push((x, y));
            }
            dummy_paths.insert((src, tgt), waypoints);
        }
    }

    LoomLayout {
        node_positions,
        dummy_paths,
        bounds: (width, height),
    }
}

/// Build a map from NodeIndex to its position (index) within the layer ordering.
fn index_positions(layer: &[NodeIndex]) -> HashMap<NodeIndex, f64> {
    layer
        .iter()
        .enumerate()
        .map(|(i, &ni)| (ni, i as f64))
        .collect()
}

/// Reorder `layer` by the barycenter (average position) of each node's neighbors
/// in the adjacent layer, according to the given direction.
fn barycenter_reorder(
    layer: &mut Vec<NodeIndex>,
    adj_positions: &HashMap<NodeIndex, f64>,
    graph: &petgraph::stable_graph::StableDiGraph<LoomGraphNode, super::graph::LoomEdge>,
    direction: Direction,
) {
    let mut barycenters: Vec<(NodeIndex, f64)> = layer
        .iter()
        .map(|&ni| {
            let neighbors: Vec<NodeIndex> = graph.neighbors_directed(ni, direction).collect();
            let relevant: Vec<f64> = neighbors
                .iter()
                .filter_map(|n| adj_positions.get(n).copied())
                .collect();
            let bc = if relevant.is_empty() {
                // Keep current position when no neighbors in adjacent layer
                adj_positions
                    .values()
                    .copied()
                    .reduce(f64::max)
                    .unwrap_or(0.0)
                    + 1.0
            } else {
                relevant.iter().sum::<f64>() / relevant.len() as f64
            };
            (ni, bc)
        })
        .collect();

    barycenters.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    *layer = barycenters.into_iter().map(|(ni, _)| ni).collect();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loom::graph::build_graph;
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
    fn test_layout_assigns_layers_by_node_type() {
        let mut loom = loom_with_unlocked_extractors(6);

        // Add a T1 shuttle
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
        let layout = compute_layout(&lg, &loom, 100.0, 100.0);

        // Find an extractor and the shuttle
        let ext_ni = lg.node_indices[&LoomGraphNode::Extractor(NodeId::EmberSpindle)];
        let shuttle_ni = lg.node_indices[&LoomGraphNode::Shuttle(0)];

        let ext_x = layout.node_positions[&ext_ni].0;
        let shuttle_x = layout.node_positions[&shuttle_ni].0;

        assert!(
            shuttle_x > ext_x,
            "shuttle x ({}) should be greater than extractor x ({})",
            shuttle_x,
            ext_x
        );
    }

    #[test]
    fn test_layout_positions_within_bounds() {
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

        let width = 200.0;
        let height = 150.0;
        let lg = build_graph(&loom);
        let layout = compute_layout(&lg, &loom, width, height);

        for &(x, y) in layout.node_positions.values() {
            assert!(
                x >= 0.0 && x <= width,
                "x={} out of bounds [0, {}]",
                x,
                width
            );
            assert!(
                y >= 0.0 && y <= height,
                "y={} out of bounds [0, {}]",
                y,
                height
            );
        }
    }

    #[test]
    fn test_layout_single_node() {
        let loom = loom_with_unlocked_extractors(1);
        let lg = build_graph(&loom);

        let width = 80.0;
        let height = 60.0;
        let layout = compute_layout(&lg, &loom, width, height);

        assert_eq!(layout.node_positions.len(), 1);

        let pos = layout.node_positions.values().next().unwrap();
        assert!(
            (pos.0 - width / 2.0).abs() < f64::EPSILON,
            "single node x should be centered at {}, got {}",
            width / 2.0,
            pos.0
        );
        assert!(
            (pos.1 - height / 2.0).abs() < f64::EPSILON,
            "single node y should be centered at {}, got {}",
            height / 2.0,
            pos.1
        );
    }

    #[test]
    fn test_layout_empty_graph() {
        // No unlocked extractors, no shuttles, no patterns => 0 nodes.
        let loom = loom_with_unlocked_extractors(0);
        let lg = build_graph(&loom);
        assert_eq!(lg.graph.node_count(), 0);

        let width = 120.0;
        let height = 90.0;
        let layout = compute_layout(&lg, &loom, width, height);

        assert!(layout.node_positions.is_empty());
        assert!(layout.dummy_paths.is_empty());
        assert_eq!(layout.bounds, (width, height));
    }

    #[test]
    fn test_layout_only_one_active_layer_centers_x() {
        // Multiple extractors, no shuttles/patterns => only layer 0 is active.
        let loom = loom_with_unlocked_extractors(4);
        let lg = build_graph(&loom);
        assert_eq!(lg.graph.node_count(), 4);

        let width = 200.0;
        let height = 100.0;
        let layout = compute_layout(&lg, &loom, width, height);

        // With a single active layer, every node should be centered on x.
        for &(x, _y) in layout.node_positions.values() {
            assert!(
                (x - width / 2.0).abs() < f64::EPSILON,
                "expected x centered at {}, got {}",
                width / 2.0,
                x
            );
        }
        // No edges at all => no dummy paths.
        assert!(layout.dummy_paths.is_empty());
    }

    #[test]
    fn test_layout_disconnected_node_uses_fallback_barycenter() {
        // A shuttle with no sources_a/sources_b is disconnected from layer 0,
        // exercising the "no neighbors in adjacent layer" branch of
        // barycenter_reorder (relevant.is_empty()).
        let mut loom = loom_with_unlocked_extractors(6);
        let disconnected = Shuttle::new(
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            Resource::ForgedLight,
            1.0,
            1,
            vec![], // no sources for input A
            vec![], // no sources for input B
        );
        loom.persistent.shuttles.push(disconnected);

        let lg = build_graph(&loom);
        assert_eq!(lg.graph.edge_count(), 0, "shuttle should have no edges");

        let layout = compute_layout(&lg, &loom, 300.0, 200.0);
        // Should still assign a position without panicking.
        assert_eq!(layout.node_positions.len(), 7);
        for &(x, y) in layout.node_positions.values() {
            assert!((0.0..=300.0).contains(&x));
            assert!((0.0..=200.0).contains(&y));
        }
    }

    #[test]
    fn test_layout_backward_sweep_reorders_multi_tier_chain() {
        // A T1 -> T2 shuttle chain activates layers 0, 1, 2 so both the
        // forward (incoming) and the odd-sweep backward (outgoing) barycenter
        // passes run.
        let mut loom = loom_with_unlocked_extractors(6);

        let t1 = Shuttle::new(
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            Resource::ForgedLight,
            1.0,
            1,
            vec![LoomNodeRef::Extractor(NodeId::EmberSpindle)],
            vec![LoomNodeRef::Extractor(NodeId::VoidCondenser)],
        );
        loom.persistent.shuttles.push(t1);

        let t2 = Shuttle::new(
            Resource::ForgedLight,
            Resource::Memory,
            NodeNature::Pattern,
            Resource::EchoGlass,
            1.0,
            2,
            vec![LoomNodeRef::Shuttle(0)],
            vec![LoomNodeRef::Extractor(NodeId::MemoryArchive)],
        );
        loom.persistent.shuttles.push(t2);

        let lg = build_graph(&loom);
        let layout = compute_layout(&lg, &loom, 400.0, 300.0);

        let t1_ni = lg.node_indices[&LoomGraphNode::Shuttle(0)];
        let t2_ni = lg.node_indices[&LoomGraphNode::Shuttle(1)];
        let ext_ni = lg.node_indices[&LoomGraphNode::Extractor(NodeId::EmberSpindle)];

        let ext_x = layout.node_positions[&ext_ni].0;
        let t1_x = layout.node_positions[&t1_ni].0;
        let t2_x = layout.node_positions[&t2_ni].0;

        assert!(ext_x < t1_x, "extractor layer should precede T1 layer");
        assert!(t1_x < t2_x, "T1 layer should precede T2 layer");
    }

    #[test]
    fn test_layout_generates_dummy_path_for_long_span_edge() {
        // A pattern sink (layer 4) fed directly by an extractor (layer 0)
        // spans 4 layers, so compute_layout should synthesize 3 waypoints.
        let mut loom = loom_with_unlocked_extractors(6);
        loom.persistent.patterns.push(WovenPattern {
            index: 0,
            name: "Long Span".to_string(),
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

        let lg = build_graph(&loom);
        let layout = compute_layout(&lg, &loom, 500.0, 300.0);

        let ext_ni = lg.node_indices[&LoomGraphNode::Extractor(NodeId::EmberSpindle)];
        let sink_ni = lg.node_indices[&LoomGraphNode::PatternSink(0)];

        let waypoints = layout
            .dummy_paths
            .get(&(ext_ni, sink_ni))
            .expect("long-span edge should get a dummy path");
        assert_eq!(waypoints.len(), 3, "span of 4 should produce 3 waypoints");

        // Waypoints should move monotonically from source toward target on x.
        let ext_x = layout.node_positions[&ext_ni].0;
        let sink_x = layout.node_positions[&sink_ni].0;
        let mut prev = ext_x;
        for &(x, _y) in waypoints {
            if ext_x <= sink_x {
                assert!(x >= prev - 1e-9, "waypoints should move toward target");
            }
            prev = x;
        }
    }

    #[test]
    fn test_layout_adjacent_layer_edge_has_no_dummy_path() {
        // Extractor -> T1 shuttle is a span-1 edge; no dummy path expected.
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

        let lg = build_graph(&loom);
        let layout = compute_layout(&lg, &loom, 200.0, 150.0);
        assert!(
            layout.dummy_paths.is_empty(),
            "adjacent-layer edges should not need dummy paths"
        );
    }

    #[test]
    fn test_node_layer_direct() {
        let loom = loom_with_unlocked_extractors(1);
        assert_eq!(
            node_layer(&LoomGraphNode::Extractor(NodeId::EmberSpindle), &loom),
            0
        );
        assert_eq!(node_layer(&LoomGraphNode::PatternSink(0), &loom), 4);
        // Unknown shuttle index falls back to layer 1.
        assert_eq!(node_layer(&LoomGraphNode::Shuttle(99), &loom), 1);
    }

    #[test]
    fn test_node_layer_uses_shuttle_tier() {
        let mut loom = loom_with_unlocked_extractors(6);
        let t3 = Shuttle::new(
            Resource::Ember,
            Resource::VoidEssence,
            NodeNature::Heat,
            Resource::WovenReality,
            1.0,
            3,
            vec![],
            vec![],
        );
        loom.persistent.shuttles.push(t3);
        assert_eq!(node_layer(&LoomGraphNode::Shuttle(0), &loom), 3);
    }
}
