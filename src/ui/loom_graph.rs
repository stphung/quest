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

    let canvas_width = (area.width * 2) as f64; // Braille: 2 dots per cell horizontally
    let canvas_height = (area.height * 4) as f64; // Braille: 4 dots per cell vertically

    // Pre-compute glow set (edges feeding active pattern sinks).
    let glowing_edges = compute_glowing_edges(loom_graph, loom);

    // Pre-compute node rendering data: position, label, gauge fill, color.
    let node_info: Vec<NodeRenderInfo> = layout
        .node_positions
        .iter()
        .filter_map(|(&ni, &(lx, ly))| {
            let node = loom_graph.graph.node_weight(ni)?;
            let info = build_node_render_info(
                ni,
                node,
                loom,
                &layout.bounds,
                canvas_width,
                canvas_height,
                lx,
                ly,
            );
            Some(info)
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

    let selected = ui.selected_graph_node;

    // Build and render the canvas.
    let canvas = Canvas::default()
        .x_bounds([0.0, canvas_width])
        .y_bounds([0.0, canvas_height])
        .marker(Marker::Braille)
        .background_color(Color::Rgb(10, 5, 18))
        .paint(|ctx| {
            // 1. Draw edges (behind nodes).
            for seg in &edge_segments {
                for pair in seg.points.windows(2) {
                    ctx.draw(&CanvasLine::new(
                        pair[0].0, pair[0].1, pair[1].0, pair[1].1, seg.color,
                    ));
                }
                // Draw particle dots — small cross clusters for visibility with Braille.
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

            // 2. Draw nodes as gauge bars via ctx.print (text-resolution, always crisp).
            let has_selection = selected.is_some();
            for info in &node_info {
                let is_selected = selected == Some(info.ni);
                render_gauge_node(ctx, info, is_selected, has_selection);
            }
        });

    frame.render_widget(canvas, area);
}

/// Pre-computed rendering data for a single graph node.
struct NodeRenderInfo {
    ni: NodeIndex,
    cx: f64,
    cy: f64,
    label: String,
    /// Display width of label in terminal columns (accounts for double-width emoji).
    label_display_width: usize,
    color: Color,
    /// Fill fraction (0.0..1.0) for the gauge bar.
    fill: f64,
    /// Short text after the gauge (rate or progress info).
    rate_text: String,
    /// Gauge width in characters.
    gauge_width: usize,
}

/// Resource emoji for display.
fn resource_emoji(resource: Resource) -> &'static str {
    match resource {
        Resource::Ember => "\u{1f525}",          // 🔥
        Resource::Reflection => "\u{1f48e}",     // 💎
        Resource::VoidEssence => "\u{1f300}",    // 🌀
        Resource::Memory => "\u{1f4dc}",         // 📜
        Resource::Silence => "\u{1f311}",        // 🌑
        Resource::Resonance => "\u{1f514}",      // 🔔
        Resource::ForgedLight => "\u{2728}",     // ✨
        Resource::EchoGlass => "\u{1fa9e}",      // 🪞
        Resource::StillbornSong => "\u{1f3b5}",  // 🎵
        Resource::CondensedEmber => "\u{1f536}", // 🔶
        Resource::EmberEcho => "\u{1f538}",      // 🔸
        Resource::PurifiedVoid => "\u{1f49c}",   // 💜
        Resource::WovenReality => "\u{1f310}",   // 🌐
    }
}

/// Compute the terminal display width of a string (emoji = 2 columns).
fn display_width(s: &str) -> usize {
    use unicode_width::UnicodeWidthStr;
    UnicodeWidthStr::width(s)
}

/// Gauge bar characters.
const GAUGE_FULL: char = '▰';
const GAUGE_EMPTY: char = '▱';

fn build_node_render_info(
    ni: NodeIndex,
    node: &LoomGraphNode,
    loom: &LoomState,
    bounds: &(f64, f64),
    canvas_width: f64,
    canvas_height: f64,
    lx: f64,
    ly: f64,
) -> NodeRenderInfo {
    let (cx, cy) = layout_to_canvas(lx, ly, bounds, canvas_width, canvas_height);
    let gauge_width = 8;
    let warp = loom.time_warp.max(1.0); // divide displayed rates by time warp factor

    match node {
        LoomGraphNode::Extractor(id) => {
            let ext = &loom.persistent.nodes[id.index()];
            let resource = node_native_resource(*id);
            let emoji = resource_emoji(resource);
            let label = format!("{} {} L{}", emoji, id.name(), ext.level);
            let label_display_width = display_width(&label);
            let color = resource_color_render(resource);
            let cap = ext.buffer_capacity;
            let fill = if cap > 0.0 {
                (ext.buffer / cap).clamp(0.0, 1.0)
            } else {
                0.0
            };
            // rate_trackers already stores un-warped rates (divided by warp in tick_stages),
            // so don't divide again here.
            let rate = loom
                .rate_trackers
                .get(&resource)
                .map(|t| t.rate_per_hour())
                .unwrap_or(0.0);
            let rate_text = format!("{:.0}/hr", rate);
            NodeRenderInfo {
                ni,
                cx,
                cy,
                label,
                label_display_width,
                color,
                fill,
                rate_text,
                gauge_width,
            }
        }
        LoomGraphNode::Shuttle(idx) => {
            if *idx == usize::MAX {
                return NodeRenderInfo {
                    ni,
                    cx,
                    cy,
                    label: "NEW".to_string(),
                    label_display_width: 3,
                    color: Color::Rgb(100, 100, 100),
                    fill: 0.0,
                    rate_text: String::new(),
                    gauge_width,
                };
            }
            if *idx < loom.persistent.shuttles.len() {
                let s = &loom.persistent.shuttles[*idx];
                let in_a = resource_emoji(s.input_a);
                let in_b = resource_emoji(s.input_b);
                let out_emoji = resource_emoji(s.output);
                let out_name = short_resource_name(s.output);
                let label = format!(
                    "{}+{}\u{2192}{} {} L{}",
                    in_a, in_b, out_emoji, out_name, s.level
                );
                let label_display_width = display_width(&label);
                let color = resource_color_render(s.output);
                let fill = if s.buffer_capacity > 0.0 {
                    (s.buffer / s.buffer_capacity).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let rate = s.output_rate_tracker.rate_per_hour() / warp;
                let rate_text = format!("{:.0}/hr", rate);
                NodeRenderInfo {
                    ni,
                    cx,
                    cy,
                    label,
                    label_display_width,
                    color,
                    fill,
                    rate_text,
                    gauge_width,
                }
            } else {
                NodeRenderInfo {
                    ni,
                    cx,
                    cy,
                    label: format!("S{}", idx),
                    label_display_width: 2,
                    color: Color::Rgb(180, 180, 180),
                    fill: 0.0,
                    rate_text: String::new(),
                    gauge_width,
                }
            }
        }
        LoomGraphNode::PatternSink(idx) => {
            if *idx < loom.persistent.patterns.len() {
                let pat = &loom.persistent.patterns[*idx];
                let progress = compute_pattern_progress(pat);
                let label = format!("\u{2b50} {}", pat.name); // ⭐
                let label_display_width = display_width(&label);
                let rate_text = format_pattern_status(pat);
                NodeRenderInfo {
                    ni,
                    cx,
                    cy,
                    label,
                    label_display_width,
                    color: Color::Rgb(255, 200, 60),
                    fill: progress,
                    rate_text,
                    gauge_width: 10,
                }
            } else {
                NodeRenderInfo {
                    ni,
                    cx,
                    cy,
                    label: "\u{2b50} Pattern".to_string(),
                    label_display_width: 9,
                    color: Color::Rgb(255, 200, 60),
                    fill: 0.0,
                    rate_text: String::new(),
                    gauge_width: 10,
                }
            }
        }
    }
}

/// Render a node as up to three text lines at canvas coordinates:
///   Line 1: `Label ▰▰▰▰▱▱▱▱`
///   Line 2: `     42/hr`        (centered, dimmed)
///   Line 3: `▔▔▔▔▔▔▔▔▔▔▔▔▔▔`   (underline bar, selected only)
///
/// All nodes render at full color. Selected node gets a bright white
/// underline bar beneath the rate text.
fn render_gauge_node(
    ctx: &mut ratatui::widgets::canvas::Context<'_>,
    info: &NodeRenderInfo,
    is_selected: bool,
    _has_selection: bool,
) {
    let filled = (info.fill * info.gauge_width as f64).round() as usize;
    let empty = info.gauge_width.saturating_sub(filled);

    let gauge_str: String = std::iter::repeat(GAUGE_FULL)
        .take(filled)
        .chain(std::iter::repeat(GAUGE_EMPTY).take(empty))
        .collect();

    // All nodes at full resource color.
    let label_color = info.color;
    let rate_color = Color::Rgb(100, 100, 120);

    // Each text row in Canvas Braille space occupies 4 braille dots vertically.
    // We render 2 lines (label+gauge, then rate) spaced 4 dots apart.
    let row_height = 4.0; // 1 terminal row = 4 braille dots

    // Line 1: label + gauge (centered on node position)
    let label_display_w = info.label_display_width + 1 + info.gauge_width;
    let half_w = label_display_w as f64 / 2.0;

    let line1 = Line::from(vec![
        Span::styled(format!("{} ", info.label), Style::default().fg(label_color)),
        Span::styled(gauge_str, Style::default().fg(label_color)),
    ]);
    ctx.print(info.cx - half_w, info.cy + row_height, line1);

    // Line 2: rate text right-aligned under the gauge portion of line 1.
    if !info.rate_text.is_empty() {
        let line2 = Line::from(Span::styled(
            info.rate_text.clone(),
            Style::default().fg(rate_color),
        ));
        let right_edge = info.cx + half_w; // right edge of line 1
        let rate_x = right_edge - info.rate_text.len() as f64;
        ctx.print(rate_x, info.cy, line2);
    }

    // Line 3 (selected only): bright white underline bar well below rate text.
    if is_selected {
        let bar: String = std::iter::repeat('\u{2594}')
            .take(label_display_w)
            .collect();
        let line3 = Line::from(Span::styled(bar, Style::default().fg(Color::White)));
        ctx.print(info.cx - half_w, info.cy - 2.0 * row_height, line3);
    }
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

/// Short display name for a resource (used in shuttle labels).
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
        Resource::CondensedEmber => "CEmber",
        Resource::EmberEcho => "EEcho",
        Resource::PurifiedVoid => "PVoid",
        Resource::WovenReality => "WReal",
    }
}

/// Compute overall pattern progress as a fraction (0.0..1.0).
///
/// Each requirement contributes 1.0 when completed, or partial progress
/// based on sustained_secs / sustain_duration_secs when in progress.
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
fn resource_color_render(resource: Resource) -> Color {
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
        Resource::CondensedEmber => Color::Rgb(230, 160, 50),
        Resource::EmberEcho => Color::Rgb(220, 140, 80),
        Resource::PurifiedVoid => Color::Rgb(180, 100, 240),
        Resource::WovenReality => Color::Rgb(100, 200, 200),
    }
}

/// Format seconds as human-friendly duration (e.g., "1h 18m", "42m", "5m").
pub fn format_duration(secs: f64) -> String {
    let total_mins = (secs / 60.0).round() as u64;
    if total_mins == 0 {
        return "<1m".to_string();
    }
    let hours = total_mins / 60;
    let mins = total_mins % 60;
    if hours > 0 && mins > 0 {
        format!("{}h {}m", hours, mins)
    } else if hours > 0 {
        format!("{}h", hours)
    } else {
        format!("{}m", mins)
    }
}

/// Format pattern progress as "X% (Yh Zm left)" or "100% ✓".
pub fn format_pattern_status(pattern: &crate::loom::types::WovenPattern) -> String {
    let progress = compute_pattern_progress(pattern);
    let pct = (progress * 100.0).round() as u32;

    if pct >= 100 {
        return "100% \u{2713}".to_string();
    }

    // Compute remaining time from incomplete requirements.
    let remaining_secs: f64 = pattern
        .requirements
        .iter()
        .filter(|r| !r.completed)
        .map(|r| (r.sustain_duration_secs - r.sustained_secs).max(0.0))
        .sum();

    if remaining_secs > 0.0 {
        format!("{}% ({} left)", pct, format_duration(remaining_secs))
    } else {
        format!("{}%", pct)
    }
}
