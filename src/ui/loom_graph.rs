//! Canvas-based graph renderer for the Loom DAG view.
//!
//! Renders the production graph onto a ratatui Canvas widget with:
//! - Edges (lines/polylines) with glow propagation and particle animation
//! - Nodes (rectangles with labels) colored by resource type
//! - Selection highlighting

use std::collections::HashSet;

use petgraph::stable_graph::{EdgeIndex, NodeIndex};
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::canvas::{Canvas, Line as CanvasLine, Points},
    Frame,
};

use crate::loom::graph::{LoomGraph, LoomGraphNode};
use crate::loom::layout::LoomLayout;
use crate::loom::logic::node_native_resource;
use crate::loom::types::{LoomState, LoomUiState, Resource};

/// Half-width/height of a node rectangle in canvas coordinates.
const NODE_HALF_W: f64 = 3.0;
const NODE_HALF_H: f64 = 2.0;

/// Render the Loom production graph onto a Canvas widget.
pub fn render_graph_canvas(
    frame: &mut Frame,
    area: Rect,
    loom_graph: &LoomGraph,
    layout: &LoomLayout,
    ui: &LoomUiState,
    loom: &LoomState,
) {
    if area.width < 4 || area.height < 4 {
        return;
    }

    let canvas_width = area.width as f64;
    let canvas_height = (area.height * 2) as f64; // HalfBlock doubles vertical resolution

    // Pre-compute glow set (edges feeding active pattern sinks).
    let glowing_edges = compute_glowing_edges(loom_graph, loom);

    // Pre-compute node labels and colors.
    let node_info: Vec<(NodeIndex, (f64, f64), String, Color)> = layout
        .node_positions
        .iter()
        .filter_map(|(&ni, &(lx, ly))| {
            let node = loom_graph.graph.node_weight(ni)?;
            let (label, color) = node_label_color(node, loom);
            // Map layout coords to canvas coords.
            let (cx, cy) = layout_to_canvas(lx, ly, &layout.bounds, canvas_width, canvas_height);
            Some((ni, (cx, cy), label, color))
        })
        .collect();

    // Pre-compute edge segments.
    let edge_segments: Vec<EdgeSegment> = loom_graph
        .graph
        .edge_indices()
        .filter_map(|ei| {
            let (src_ni, tgt_ni) = loom_graph.graph.edge_endpoints(ei)?;
            let edge = loom_graph.graph.edge_weight(ei)?;
            let src_pos = layout.node_positions.get(&src_ni)?;
            let tgt_pos = layout.node_positions.get(&tgt_ni)?;

            let glowing = glowing_edges.contains(&ei);
            let color = if glowing {
                Color::Rgb(255, 200, 60) // gold/amber glow
            } else {
                Color::Rgb(60, 70, 100) // dim gray-blue
            };

            // Build waypoint chain (source -> dummies -> target) in canvas coords.
            let mut points = Vec::new();
            let (sx, sy) = layout_to_canvas(
                src_pos.0,
                src_pos.1,
                &layout.bounds,
                canvas_width,
                canvas_height,
            );
            points.push((sx, sy));

            if let Some(dummies) = layout.dummy_paths.get(&(src_ni, tgt_ni)) {
                for &(dx, dy) in dummies {
                    let (cx, cy) =
                        layout_to_canvas(dx, dy, &layout.bounds, canvas_width, canvas_height);
                    points.push((cx, cy));
                }
            }

            let (tx, ty) = layout_to_canvas(
                tgt_pos.0,
                tgt_pos.1,
                &layout.bounds,
                canvas_width,
                canvas_height,
            );
            points.push((tx, ty));

            // Particle positions (3 dots along the edge path based on phase).
            let particles = if edge.current_rate > 0.0 {
                let phase = ui.particle_phases.get(&ei).copied().unwrap_or(0.0);
                compute_particles(&points, phase)
            } else {
                Vec::new()
            };

            Some(EdgeSegment {
                points,
                color,
                particles,
            })
        })
        .collect();

    // Build and render the canvas.
    let canvas = Canvas::default()
        .x_bounds([0.0, canvas_width])
        .y_bounds([0.0, canvas_height])
        .marker(Marker::HalfBlock)
        .background_color(Color::Rgb(10, 5, 18))
        .paint(|ctx| {
            // 1. Draw edges (behind nodes).
            for seg in &edge_segments {
                for pair in seg.points.windows(2) {
                    ctx.draw(&CanvasLine::new(
                        pair[0].0, pair[0].1, pair[1].0, pair[1].1, seg.color,
                    ));
                }
                // Draw particle dots.
                if !seg.particles.is_empty() {
                    let coords: Vec<(f64, f64)> = seg.particles.clone();
                    ctx.draw(&Points {
                        coords: &coords,
                        color: Color::Rgb(255, 255, 200),
                    });
                }
            }

            // 2. Draw node rectangles.
            for &(ni, (cx, cy), ref _label, color) in &node_info {
                let selected = ui.selected_graph_node == Some(ni);
                let border_color = if selected { Color::White } else { color };
                draw_node_rect(ctx, cx, cy, border_color);
            }

            // 3. Draw node labels via ctx.print.
            for &(_, (cx, cy), ref label, color) in &node_info {
                let line = Line::from(Span::styled(label.clone(), Style::default().fg(color)));
                ctx.print(cx - 1.0, cy, line);
            }
        });

    frame.render_widget(canvas, area);
}

/// A pre-computed edge with its canvas-space polyline, color, and particle positions.
struct EdgeSegment {
    points: Vec<(f64, f64)>,
    color: Color,
    particles: Vec<(f64, f64)>,
}

/// Map layout-space coordinates to canvas-space coordinates.
/// Layout space: (0..bounds.0, 0..bounds.1) with y increasing downward.
/// Canvas space: (0..canvas_width, 0..canvas_height) with y increasing upward.
fn layout_to_canvas(
    lx: f64,
    ly: f64,
    bounds: &(f64, f64),
    canvas_width: f64,
    canvas_height: f64,
) -> (f64, f64) {
    let scale_x = if bounds.0 > 0.0 {
        canvas_width / bounds.0
    } else {
        1.0
    };
    let scale_y = if bounds.1 > 0.0 {
        canvas_height / bounds.1
    } else {
        1.0
    };
    let cx = lx * scale_x;
    // Y-axis inversion: canvas y increases upward, layout y increases downward.
    let cy = canvas_height - (ly * scale_y);
    (cx, cy)
}

/// Draw a small rectangle outline for a node at canvas coordinates (cx, cy).
fn draw_node_rect(ctx: &mut ratatui::widgets::canvas::Context<'_>, cx: f64, cy: f64, color: Color) {
    let x0 = cx - NODE_HALF_W;
    let x1 = cx + NODE_HALF_W;
    let y0 = cy - NODE_HALF_H;
    let y1 = cy + NODE_HALF_H;

    // Top edge
    ctx.draw(&CanvasLine::new(x0, y1, x1, y1, color));
    // Bottom edge
    ctx.draw(&CanvasLine::new(x0, y0, x1, y0, color));
    // Left edge
    ctx.draw(&CanvasLine::new(x0, y0, x0, y1, color));
    // Right edge
    ctx.draw(&CanvasLine::new(x1, y0, x1, y1, color));
}

/// Compute 3 particle positions along a polyline path based on phase (0.0..1.0).
fn compute_particles(path: &[(f64, f64)], phase: f64) -> Vec<(f64, f64)> {
    if path.len() < 2 {
        return Vec::new();
    }

    // Compute total path length.
    let mut total_len = 0.0;
    let mut seg_lengths = Vec::new();
    for pair in path.windows(2) {
        let dx = pair[1].0 - pair[0].0;
        let dy = pair[1].1 - pair[0].1;
        let len = (dx * dx + dy * dy).sqrt();
        seg_lengths.push(len);
        total_len += len;
    }
    if total_len < 0.001 {
        return Vec::new();
    }

    let mut particles = Vec::with_capacity(3);
    for i in 0..3 {
        // Evenly spaced particles, offset by phase.
        let t = (phase + i as f64 / 3.0) % 1.0;
        let target_dist = t * total_len;

        let mut accum = 0.0;
        for (seg_i, &seg_len) in seg_lengths.iter().enumerate() {
            if accum + seg_len >= target_dist || seg_i == seg_lengths.len() - 1 {
                let local_t = if seg_len > 0.001 {
                    (target_dist - accum) / seg_len
                } else {
                    0.0
                };
                let x = path[seg_i].0 + (path[seg_i + 1].0 - path[seg_i].0) * local_t;
                let y = path[seg_i].1 + (path[seg_i + 1].1 - path[seg_i].1) * local_t;
                particles.push((x, y));
                break;
            }
            accum += seg_len;
        }
    }
    particles
}

/// Compute glowing edges via BFS upstream from active pattern sinks.
///
/// A pattern sink "glows" if any of its requirements has `sustained_secs > 0` and `!completed`.
/// All edges feeding a glowing sink (recursively upstream) also glow.
fn compute_glowing_edges(lg: &LoomGraph, loom: &LoomState) -> HashSet<EdgeIndex> {
    let mut glowing = HashSet::new();
    let mut visited_nodes: HashSet<NodeIndex> = HashSet::new();

    // Find glowing pattern sinks.
    let mut queue: Vec<NodeIndex> = Vec::new();
    for (&ref gn, &ni) in &lg.node_indices {
        if let LoomGraphNode::PatternSink(pat_idx) = gn {
            if *pat_idx < loom.persistent.patterns.len() {
                let pat = &loom.persistent.patterns[*pat_idx];
                let is_glowing = pat
                    .requirements
                    .iter()
                    .any(|r| r.sustained_secs > 0.0 && !r.completed);
                if is_glowing {
                    queue.push(ni);
                    visited_nodes.insert(ni);
                }
            }
        }
    }

    // BFS upstream: for each node, mark all incoming edges as glowing,
    // and enqueue their source nodes.
    while let Some(node) = queue.pop() {
        for edge_ref in lg.graph.edges_directed(node, Direction::Incoming) {
            glowing.insert(edge_ref.id());
            let source = edge_ref.source();
            if visited_nodes.insert(source) {
                queue.push(source);
            }
        }
    }

    glowing
}

/// Returns a 2-letter label and color for a graph node.
fn node_label_color(node: &LoomGraphNode, loom: &LoomState) -> (String, Color) {
    match node {
        LoomGraphNode::Extractor(id) => {
            let label = match id {
                crate::loom::types::NodeId::EmberSpindle => "ES",
                crate::loom::types::NodeId::ReflectionLens => "RL",
                crate::loom::types::NodeId::VoidCondenser => "VC",
                crate::loom::types::NodeId::MemoryArchive => "MA",
                crate::loom::types::NodeId::SilenceWell => "SW",
                crate::loom::types::NodeId::ResonanceForge => "RF",
            };
            let resource = node_native_resource(*id);
            (label.to_string(), resource_color(resource))
        }
        LoomGraphNode::Shuttle(idx) => {
            let label = format!("S{}", idx);
            let color = if *idx < loom.persistent.shuttles.len() {
                resource_color(loom.persistent.shuttles[*idx].output)
            } else {
                Color::Rgb(180, 180, 180)
            };
            (label, color)
        }
        LoomGraphNode::PatternSink(idx) => {
            let label = format!("P{}", idx);
            (label, Color::Rgb(220, 200, 255))
        }
    }
}

/// Map a resource to its display color.
fn resource_color(resource: Resource) -> Color {
    match resource {
        Resource::Ember => Color::Rgb(255, 140, 0),
        Resource::Reflection => Color::Rgb(100, 180, 255),
        Resource::VoidEssence => Color::Rgb(160, 80, 220),
        Resource::Memory => Color::Rgb(200, 180, 120),
        Resource::Silence => Color::Rgb(120, 120, 160),
        Resource::Resonance => Color::Rgb(200, 100, 100),
        Resource::ForgedLight => Color::Rgb(255, 220, 100),
        Resource::EchoGlass => Color::Rgb(140, 220, 200),
        Resource::StillbornSong => Color::Rgb(180, 140, 200),
        _ => Color::Rgb(180, 180, 180),
    }
}
