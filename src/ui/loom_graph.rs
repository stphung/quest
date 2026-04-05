//! Canvas-based graph renderer for the Loom DAG view.
//!
//! Renders the production graph with:
//! - Edges (Braille lines/polylines) with glow propagation and particle animation
//! - Nodes rendered as ratatui Gauge widgets overlaid on the canvas
//! - Selection highlighting

use std::collections::HashSet;

use petgraph::stable_graph::{EdgeIndex, NodeIndex};
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::Span,
    widgets::{
        canvas::{Canvas, Line as CanvasLine, Points},
        Block, Borders, Gauge,
    },
    Frame,
};

use crate::loom::graph::{LoomGraph, LoomGraphNode};
use crate::loom::layout::LoomLayout;
use crate::loom::logic::node_native_resource;
use crate::loom::types::{LoomState, LoomUiState, NodeId, Resource};

/// Width of each node gauge in terminal columns.
const NODE_WIDTH: u16 = 20;
/// Height of each node gauge in terminal rows (1 for gauge + 1 for border).
const NODE_HEIGHT: u16 = 3;

/// Render the Loom production graph: Canvas edges + Gauge widget nodes.
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

    let canvas_width = (area.width * 2) as f64; // Braille: 2 dots per cell horizontally
    let canvas_height = (area.height * 4) as f64; // Braille: 4 dots per cell vertically

    // Pre-compute glow set (edges feeding active pattern sinks).
    let glowing_edges = compute_glowing_edges(loom_graph, loom);

    // Pre-compute node info for gauge overlay positioning.
    let node_info: Vec<NodeRenderInfo> = layout
        .node_positions
        .iter()
        .filter_map(|(&ni, &(lx, ly))| {
            let node = loom_graph.graph.node_weight(ni)?;
            Some(build_node_render_info(
                ni,
                node,
                loom,
                lx,
                ly,
                &layout.bounds,
                area,
            ))
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

            // Build waypoint chain in canvas coords.
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

    // 1. Render canvas with edges only.
    let canvas = Canvas::default()
        .x_bounds([0.0, canvas_width])
        .y_bounds([0.0, canvas_height])
        .marker(Marker::Braille)
        .background_color(Color::Rgb(10, 5, 18))
        .paint(|ctx| {
            for seg in &edge_segments {
                for pair in seg.points.windows(2) {
                    ctx.draw(&CanvasLine::new(
                        pair[0].0, pair[0].1, pair[1].0, pair[1].1, seg.color,
                    ));
                }
                if !seg.particles.is_empty() {
                    let mut coords: Vec<(f64, f64)> = Vec::new();
                    for &(px, py) in &seg.particles {
                        coords.push((px, py));
                        coords.push((px + 0.5, py));
                        coords.push((px - 0.5, py));
                        coords.push((px, py + 0.5));
                        coords.push((px, py - 0.5));
                    }
                    ctx.draw(&Points {
                        coords: &coords,
                        color: Color::Rgb(255, 255, 200),
                    });
                }
            }
        });
    frame.render_widget(canvas, area);

    // 2. Overlay Gauge widgets for each node on top of the canvas.
    let selected = ui.selected_graph_node;
    for info in &node_info {
        let is_selected = selected == Some(info.ni);
        render_node_gauge(frame, info, is_selected);
    }
}

/// Pre-computed rendering data for a single graph node.
struct NodeRenderInfo {
    ni: NodeIndex,
    /// Terminal cell rect where this node's gauge renders.
    rect: Rect,
    label: String,
    color: Color,
    /// Fill fraction (0.0..1.0) — buffer fill for extractors/shuttles, progress for patterns.
    ratio: f64,
    /// Whether this is a pattern sink (renders differently).
    is_pattern: bool,
}

fn build_node_render_info(
    ni: NodeIndex,
    node: &LoomGraphNode,
    loom: &LoomState,
    lx: f64,
    ly: f64,
    bounds: &(f64, f64),
    area: Rect,
) -> NodeRenderInfo {
    // Convert layout coords to terminal cell position.
    let frac_x = if bounds.0 > 0.0 { lx / bounds.0 } else { 0.5 };
    let frac_y = if bounds.1 > 0.0 { ly / bounds.1 } else { 0.5 };

    let center_col = area.x + (frac_x * area.width as f64) as u16;
    let center_row = area.y + (frac_y * area.height as f64) as u16;

    // Clamp the gauge rect within the area.
    let half_w = NODE_WIDTH / 2;
    let half_h = NODE_HEIGHT / 2;
    let x = center_col.saturating_sub(half_w).max(area.x);
    let y = center_row.saturating_sub(half_h).max(area.y);
    let w = NODE_WIDTH.min(area.right().saturating_sub(x));
    let h = NODE_HEIGHT.min(area.bottom().saturating_sub(y));
    let rect = Rect::new(x, y, w, h);

    match node {
        LoomGraphNode::Extractor(id) => {
            let label = match id {
                NodeId::EmberSpindle => "Ember",
                NodeId::ReflectionLens => "Reflect",
                NodeId::VoidCondenser => "Void",
                NodeId::MemoryArchive => "Memory",
                NodeId::SilenceWell => "Silence",
                NodeId::ResonanceForge => "Reson",
            }
            .to_string();
            let resource = node_native_resource(*id);
            let color = resource_color(resource);
            let ext = &loom.persistent.nodes[id.index()];
            let ratio = if ext.buffer_capacity > 0.0 {
                (ext.buffer / ext.buffer_capacity).clamp(0.0, 1.0)
            } else {
                0.0
            };
            NodeRenderInfo {
                ni,
                rect,
                label,
                color,
                ratio,
                is_pattern: false,
            }
        }
        LoomGraphNode::Shuttle(idx) => {
            if *idx == usize::MAX {
                return NodeRenderInfo {
                    ni,
                    rect,
                    label: "NEW".to_string(),
                    color: Color::Rgb(100, 100, 100),
                    ratio: 0.0,
                    is_pattern: false,
                };
            }
            let (label, color, ratio) = if *idx < loom.persistent.shuttles.len() {
                let s = &loom.persistent.shuttles[*idx];
                let out = short_resource_name(s.output);
                let lbl = if s.under_construction {
                    format!("S{} Building..", idx)
                } else {
                    format!("S{}\u{2192}{}", idx, out)
                };
                let c = resource_color(s.output);
                let r = if s.buffer_capacity > 0.0 {
                    (s.buffer / s.buffer_capacity).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                (lbl, c, r)
            } else {
                (format!("S{}", idx), Color::Rgb(180, 180, 180), 0.0)
            };
            NodeRenderInfo {
                ni,
                rect,
                label,
                color,
                ratio,
                is_pattern: false,
            }
        }
        LoomGraphNode::PatternSink(idx) => {
            let (name, progress) = if *idx < loom.persistent.patterns.len() {
                let pat = &loom.persistent.patterns[*idx];
                (pat.name.clone(), compute_pattern_progress(pat))
            } else {
                ("Pattern".to_string(), 0.0)
            };
            let label = format!("\u{2605} {}", name);
            NodeRenderInfo {
                ni,
                rect,
                label,
                color: Color::Rgb(255, 200, 60),
                ratio: progress,
                is_pattern: true,
            }
        }
    }
}

/// Render a node as a ratatui Gauge widget at its terminal position.
fn render_node_gauge(frame: &mut Frame, info: &NodeRenderInfo, is_selected: bool) {
    if info.rect.width < 3 || info.rect.height < 1 {
        return;
    }

    let border_color = if is_selected {
        Color::White
    } else {
        info.color
    };
    let border_style = Style::default().fg(border_color);
    let label_style = if is_selected {
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(info.color)
    };

    let gauge_color = if info.is_pattern {
        Color::Rgb(255, 200, 60) // gold for pattern progress
    } else {
        info.color
    };

    let label = Span::styled(info.label.clone(), label_style);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style);

    let gauge = Gauge::default()
        .block(block)
        .gauge_style(Style::default().fg(gauge_color).bg(Color::Rgb(20, 15, 30)))
        .ratio(info.ratio.clamp(0.0, 1.0))
        .label(label);

    frame.render_widget(gauge, info.rect);
}

/// A pre-computed edge with its canvas-space polyline, color, and particle positions.
struct EdgeSegment {
    points: Vec<(f64, f64)>,
    color: Color,
    particles: Vec<(f64, f64)>,
}

/// Map layout-space coordinates to canvas-space coordinates.
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
    let cy = canvas_height - (ly * scale_y);
    (cx, cy)
}

/// Compute 3 particle positions along a polyline path based on phase (0.0..1.0).
fn compute_particles(path: &[(f64, f64)], phase: f64) -> Vec<(f64, f64)> {
    if path.len() < 2 {
        return Vec::new();
    }

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
fn compute_glowing_edges(lg: &LoomGraph, loom: &LoomState) -> HashSet<EdgeIndex> {
    let mut glowing = HashSet::new();
    let mut visited_nodes: HashSet<NodeIndex> = HashSet::new();

    let mut queue: Vec<NodeIndex> = Vec::new();
    for (gn, &ni) in &lg.node_indices {
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

/// Short display name for a resource.
fn short_resource_name(resource: Resource) -> &'static str {
    match resource {
        Resource::Ember => "Ember",
        Resource::Reflection => "Reflect",
        Resource::VoidEssence => "Void",
        Resource::Memory => "Memory",
        Resource::Silence => "Silence",
        Resource::Resonance => "Reson",
        Resource::ForgedLight => "FLight",
        Resource::EchoGlass => "EGlass",
        Resource::StillbornSong => "SSong",
        _ => "???",
    }
}

/// Compute overall pattern progress as a fraction (0.0..1.0).
fn compute_pattern_progress(pattern: &crate::loom::types::WovenPattern) -> f64 {
    if pattern.requirements.is_empty() {
        return 0.0;
    }
    let total: f64 = pattern
        .requirements
        .iter()
        .map(|r| {
            if r.completed {
                1.0
            } else if r.sustain_duration_secs > 0.0 {
                (r.sustained_secs / r.sustain_duration_secs).min(1.0)
            } else {
                0.0
            }
        })
        .sum();
    total / pattern.requirements.len() as f64
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
