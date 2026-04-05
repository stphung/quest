//! Canvas-based graph renderer for the Loom DAG view.
//!
//! Renders the production graph onto a ratatui Canvas widget with:
//! - Edges (lines/polylines) with glow propagation and particle animation
//! - Nodes (rectangles with labels) colored by resource type
//! - Selection highlighting

use petgraph::stable_graph::NodeIndex;
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

/// Y-offset for node dot/edge connection point, below the text center.
/// Text renders at cy (rate) and cy+4 (label); the dot sits one row below rate.
const NODE_DOT_Y_OFFSET: f64 = -4.0;

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

            // V3: Gold edges into pattern sink when sustaining, dim otherwise.
            let tgt_node = loom_graph.graph.node_weight(tgt_ni)?;
            let color = if let LoomGraphNode::PatternSink(pat_idx) = tgt_node {
                // Check if pattern is actively sustaining.
                let is_sustaining = if *pat_idx < loom.persistent.patterns.len() {
                    let pat = &loom.persistent.patterns[*pat_idx];
                    let rates: std::collections::HashMap<_, _> = loom
                        .rate_trackers
                        .iter()
                        .map(|(r, t)| (*r, t.rate_per_hour()))
                        .collect();
                    !pat.completed
                        && pat.requirements.iter().all(|req| {
                            req.completed
                                || rates.get(&req.resource).copied().unwrap_or(0.0)
                                    >= req.required_rate
                        })
                } else {
                    false
                };
                if is_sustaining {
                    Color::Rgb(255, 200, 60) // gold
                } else {
                    Color::Rgb(60, 70, 100) // dim
                }
            } else {
                Color::Rgb(60, 70, 100) // dim for non-pattern edges
            };

            // Particle color = source node's resource color.
            let src_node = loom_graph.graph.node_weight(src_ni)?;
            let particle_color = match src_node {
                LoomGraphNode::Extractor(id) => resource_color_render(node_native_resource(*id)),
                LoomGraphNode::Shuttle(idx) => {
                    if *idx < loom.persistent.shuttles.len() {
                        resource_color_render(loom.persistent.shuttles[*idx].output)
                    } else {
                        Color::Rgb(255, 255, 200)
                    }
                }
                LoomGraphNode::PatternSink(_) => Color::Rgb(255, 200, 60),
            };

            // Build waypoint chain (source -> dummies -> target) in canvas coords.
            // Offset endpoints to match the node dot position (below text).
            let mut points = Vec::new();
            let (sx, sy) = layout_to_canvas(
                src_pos.0,
                src_pos.1,
                &layout.bounds,
                canvas_width,
                canvas_height,
            );
            points.push((sx, sy + NODE_DOT_Y_OFFSET));

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
            points.push((tx, ty + NODE_DOT_Y_OFFSET));

            // Particle positions (3 dots along the edge path based on phase).
            let particles = if edge.current_rate > 0.0 {
                let phase = ui.particle_phases.get(&ei).copied().unwrap_or(0.0);
                compute_particles(&points, phase)
            } else {
                Vec::new()
            };

            // Edge label: rate + resource emoji at midpoint.
            // current_rate is already un-warped (normalized in update_edge_rates).
            let rate = edge.current_rate;
            let label = if rate > 0.5 {
                format!("{:.0}{}/hr", rate, resource_emoji(edge.resource))
            } else {
                String::new()
            };
            // Midpoint of the path.
            let mid_idx = points.len() / 2;
            let label_pos = if points.len() >= 2 {
                let a = points[mid_idx.saturating_sub(1)];
                let b = points[mid_idx.min(points.len() - 1)];
                ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0)
            } else {
                points[0]
            };

            // Only show label if this edge connects to the selected node.
            let connected_to_selected = ui
                .selected_graph_node
                .map(|sel| src_ni == sel || tgt_ni == sel)
                .unwrap_or(false);

            Some(EdgeSegment {
                points,
                color,
                particles,
                particle_color,
                label,
                label_pos,
                show_label: connected_to_selected,
            })
        })
        .collect();

    let selected = ui.selected_graph_node;
    let frame_count = ui.throbber_frame;

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
                // Draw particles with trailing dots.
                if !seg.particles.is_empty() {
                    // Heads: bright cross clusters.
                    let mut head_coords: Vec<(f64, f64)> = Vec::new();
                    for p in &seg.particles {
                        let (px, py) = p.head;
                        head_coords.push((px, py));
                        head_coords.push((px + 0.5, py));
                        head_coords.push((px - 0.5, py));
                        head_coords.push((px, py + 0.5));
                        head_coords.push((px, py - 0.5));
                    }
                    ctx.draw(&Points {
                        coords: &head_coords,
                        color: seg.particle_color,
                    });

                    // Trails: single dimmer dots behind each head.
                    let trail_color = dim_color(seg.particle_color);
                    let mut trail_coords: Vec<(f64, f64)> = Vec::new();
                    for p in &seg.particles {
                        for &(tx, ty) in &p.trail {
                            trail_coords.push((tx, ty));
                        }
                    }
                    if !trail_coords.is_empty() {
                        ctx.draw(&Points {
                            coords: &trail_coords,
                            color: trail_color,
                        });
                    }
                }
            }

            // 2. Draw edge labels (only for edges connected to selected node).
            let dim_label = Color::Rgb(100, 90, 130);
            for seg in &edge_segments {
                if seg.show_label && !seg.label.is_empty() {
                    let lbl = Line::from(Span::styled(
                        seg.label.clone(),
                        Style::default().fg(dim_label),
                    ));
                    // Offset slightly above the edge midpoint so it doesn't sit on the line.
                    ctx.print(seg.label_pos.0, seg.label_pos.1 + 3.0, lbl);
                }
            }

            // 3. Draw single node marker below text (Braille layer).
            // This is where edges connect — see NODE_DOT_Y_OFFSET.
            for info in &node_info {
                let mut dot_coords: Vec<(f64, f64)> = Vec::new();
                let dot_y = info.cy + NODE_DOT_Y_OFFSET;
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        dot_coords.push((info.cx + dx as f64, dot_y + dy as f64));
                    }
                }
                ctx.draw(&Points {
                    coords: &dot_coords,
                    color: info.color,
                });
            }

            // 4. Draw nodes as gauge bars via ctx.print (text-resolution, always crisp).
            let has_selection = selected.is_some();
            let char_w = 2.0_f64;
            let row_h = 4.0_f64;
            for info in &node_info {
                let is_selected = selected == Some(info.ni);
                render_gauge_node(ctx, info, is_selected, has_selection, frame_count);

                // 5. Draw pattern requirement lines below the node dot.
                if !info.req_lines.is_empty() {
                    let dot_y = info.cy + NODE_DOT_Y_OFFSET;
                    // Start 2 rows below the dot, then one row per requirement.
                    let req_start_y = dot_y - 2.0 * row_h;
                    for (i, (text, color)) in info.req_lines.iter().enumerate() {
                        let req_y = req_start_y - (i as f64) * row_h;
                        let half_w = text.len() as f64 * char_w / 2.0;
                        let line =
                            Line::from(Span::styled(text.clone(), Style::default().fg(*color)));
                        ctx.print(info.cx - half_w, req_y, line);
                    }
                }
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
    /// Whether this pattern node is actively sustaining (all requirements met).
    is_sustaining: bool,
    /// Pattern requirement lines to render below the node (only for PatternSink).
    req_lines: Vec<(String, Color)>,
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
            let emoji = if ext.upgrading {
                "\u{23f3}" // hourglass for upgrading
            } else {
                resource_emoji(resource)
            };
            let label = format!("{} {} L{}", emoji, id.name(), ext.level);
            let label_display_width = display_width(&label);
            let color = if ext.upgrading {
                Color::Rgb(120, 100, 60) // dimmed amber for upgrading
            } else {
                resource_color_render(resource)
            };
            let (fill, rate_text) = if ext.upgrading {
                // Show upgrade progress as gauge fill.
                let total = crate::loom::logic::node_upgrade_duration(ext.level);
                let elapsed = total - ext.upgrade_remaining_secs;
                let progress = if total > 0.0 {
                    (elapsed / total).clamp(0.0, 1.0)
                } else {
                    1.0
                };
                let remaining = format_duration(ext.upgrade_remaining_secs);
                (progress, format!("\u{23f3}{}", remaining))
            } else {
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
                (fill, format!("{:.0}/hr", rate))
            };
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
                is_sustaining: false,
                req_lines: vec![],
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
                    is_sustaining: false,
                    req_lines: vec![],
                };
            }
            if *idx < loom.persistent.shuttles.len() {
                let s = &loom.persistent.shuttles[*idx];
                let out_emoji = resource_emoji(s.output);
                let out_name = short_resource_name(s.output);
                let label = format!("{} {}", out_emoji, out_name);
                let label_display_width = display_width(&label);
                let color = if s.under_construction {
                    Color::Rgb(120, 100, 60) // dimmed amber
                } else {
                    resource_color_render(s.output)
                };
                let (fill, rate_text) = if s.under_construction {
                    // Show construction progress.
                    let total = crate::loom::logic::shuttle_construction_ticks(s.tier) as f64;
                    let elapsed = total - s.construction_ticks_remaining as f64;
                    let progress = if total > 0.0 {
                        (elapsed / total).clamp(0.0, 1.0)
                    } else {
                        1.0
                    };
                    // Convert remaining ticks to seconds (100ms per tick), adjusted for warp.
                    let remaining_secs = s.construction_ticks_remaining as f64 * 0.1 / warp;
                    (
                        progress,
                        format!("\u{23f3}{}", format_duration(remaining_secs)),
                    )
                } else {
                    let fill = if s.buffer_capacity > 0.0 {
                        (s.buffer / s.buffer_capacity).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    let rate = s.output_rate_tracker.rate_per_hour() / warp;
                    (fill, format!("{:.0}/hr", rate))
                };
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
                    is_sustaining: false,
                    req_lines: vec![],
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
                    is_sustaining: false,
                    req_lines: vec![],
                }
            }
        }
        LoomGraphNode::PatternSink(idx) => {
            if *idx < loom.persistent.patterns.len() {
                let pat = &loom.persistent.patterns[*idx];
                let progress = compute_pattern_progress(pat);

                // Check if ALL requirements are currently being met.
                let rates: std::collections::HashMap<_, _> = loom
                    .rate_trackers
                    .iter()
                    .map(|(r, t)| (*r, t.rate_per_hour()))
                    .collect();
                let is_sustaining = !pat.completed
                    && pat.requirements.iter().all(|req| {
                        req.completed
                            || rates.get(&req.resource).copied().unwrap_or(0.0) >= req.required_rate
                    });

                // C: Play/pause prefix.
                let status_icon = if pat.completed {
                    "\u{2713}" // ✓
                } else if is_sustaining {
                    "\u{25b6}" // ▶
                } else {
                    "\u{23f8}" // ⏸
                };
                let label = format!("{} {}", status_icon, pat.name);
                let label_display_width = display_width(&label);
                let rate_text = format_pattern_status(pat);

                // R2: Build requirement lines for display below the node.
                let req_lines: Vec<(String, Color)> = pat
                    .requirements
                    .iter()
                    .map(|req| {
                        let emoji = resource_emoji(req.resource);
                        let current = rates.get(&req.resource).copied().unwrap_or(0.0);
                        let met = req.completed || current >= req.required_rate;
                        let (icon, color) = if met {
                            ("\u{2713}", Color::Rgb(80, 180, 100)) // ✓ green
                        } else {
                            ("\u{2717}", Color::Rgb(160, 60, 60)) // ✗ red
                        };
                        (
                            format!("{} {} {:.0}/hr", icon, emoji, req.required_rate),
                            color,
                        )
                    })
                    .collect();

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
                    is_sustaining,
                    req_lines,
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
                    is_sustaining: false,
                    req_lines: vec![],
                }
            }
        }
    }
}

/// Render a node as two text lines at canvas coordinates:
///   Line 1: `「Label ▰▰▰▰▱▱▱▱」`  (brackets if selected, white text)
///   Line 2: `             42/hr`    (right-aligned under gauge)
fn render_gauge_node(
    ctx: &mut ratatui::widgets::canvas::Context<'_>,
    info: &NodeRenderInfo,
    is_selected: bool,
    _has_selection: bool,
    frame_count: u32,
) {
    let filled = (info.fill * info.gauge_width as f64).round() as usize;
    let empty = info.gauge_width.saturating_sub(filled);

    // B: Shimmer effect on gauge when sustaining — a bright character ripples through.
    let gauge_str: String = if info.is_sustaining && filled > 0 {
        let shimmer_pos = (frame_count as usize / 3) % filled; // ripple every 3 frames
        (0..info.gauge_width)
            .map(|i| {
                if i < filled {
                    if i == shimmer_pos {
                        '▱'
                    } else {
                        GAUGE_FULL
                    } // bright gap ripples through
                } else {
                    GAUGE_EMPTY
                }
            })
            .collect()
    } else {
        std::iter::repeat(GAUGE_FULL)
            .take(filled)
            .chain(std::iter::repeat(GAUGE_EMPTY).take(empty))
            .collect()
    };

    // A: Pulse color when sustaining.
    let base_color = if info.is_sustaining {
        if (frame_count / 5) % 2 == 0 {
            Color::Rgb(255, 220, 100) // bright gold
        } else {
            Color::Rgb(255, 255, 200) // bright white-gold
        }
    } else {
        info.color
    };

    let label_color = if is_selected {
        Color::White
    } else {
        base_color
    };
    let rate_color = Color::Rgb(100, 100, 120);

    // Canvas Braille space: 2 dots per terminal column, 4 dots per terminal row.
    let char_w = 2.0; // 1 terminal character = 2 braille x-dots
    let row_h = 4.0; // 1 terminal row = 4 braille y-dots

    // Line 1: [bracket] label gauge [bracket]
    // Selected brackets pulse between white and resource color (~1s cycle).
    let bracket_color = if is_selected {
        if (frame_count / 5) % 2 == 0 {
            Color::White
        } else {
            info.color
        }
    } else {
        info.color
    };

    let mut spans = Vec::new();
    if is_selected {
        spans.push(Span::styled("\u{300c}", Style::default().fg(bracket_color)));
        // 「
    }
    spans.push(Span::styled(
        format!("{} ", info.label),
        Style::default().fg(label_color),
    ));
    spans.push(Span::styled(gauge_str, Style::default().fg(label_color)));
    if is_selected {
        spans.push(Span::styled("\u{300d}", Style::default().fg(bracket_color)));
        // 」
    }

    let bracket_extra = if is_selected { 2 } else { 0 }; // 「」 are 1 wide each
    let label_display_w = info.label_display_width + 1 + info.gauge_width + bracket_extra;
    let half_w_px = label_display_w as f64 * char_w / 2.0;

    let line1 = Line::from(spans);
    ctx.print(info.cx - half_w_px, info.cy + row_h, line1);

    // Line 2: rate text right-aligned under the gauge portion of line 1.
    if !info.rate_text.is_empty() {
        let line2 = Line::from(Span::styled(
            info.rate_text.clone(),
            Style::default().fg(rate_color),
        ));
        let right_edge = info.cx + half_w_px;
        let rate_w_px = info.rate_text.len() as f64 * char_w;
        ctx.print(right_edge - rate_w_px, info.cy, line2);
    }
}

/// A pre-computed edge with its canvas-space polyline, color, particle positions, and label.
struct EdgeSegment {
    points: Vec<(f64, f64)>,
    color: Color,
    particles: Vec<ParticleWithTrail>,
    /// Color for particles (source node's resource color).
    particle_color: Color,
    /// Rate label text (e.g., "42🔥/hr"). Empty if rate ~0.
    label: String,
    /// Canvas position for the label (30% along edge path).
    label_pos: (f64, f64),
    /// Whether to show the label (only for edges connected to selected node).
    show_label: bool,
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
/// Dim a color to ~50% brightness for trail dots.
fn dim_color(c: Color) -> Color {
    match c {
        Color::Rgb(r, g, b) => Color::Rgb(r / 2, g / 2, b / 2),
        _ => Color::Rgb(80, 80, 80),
    }
}

/// A particle with its position and trailing positions.
struct ParticleWithTrail {
    head: (f64, f64),
    trail: Vec<(f64, f64)>,
}

/// Compute 7 particle positions along a polyline path, each with 2 trailing dots.
fn compute_particles(path: &[(f64, f64)], phase: f64) -> Vec<ParticleWithTrail> {
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

    let num_particles = 7;
    let trail_count = 2;
    let trail_spacing = total_len * 0.015; // spacing between head and each trail dot

    let point_at_dist = |dist: f64| -> (f64, f64) {
        let d = dist.clamp(0.0, total_len);
        let mut accum = 0.0;
        for (seg_i, &seg_len) in seg_lengths.iter().enumerate() {
            if accum + seg_len >= d || seg_i == seg_lengths.len() - 1 {
                let local_t = if seg_len > 0.001 {
                    (d - accum) / seg_len
                } else {
                    0.0
                };
                let x = path[seg_i].0 + (path[seg_i + 1].0 - path[seg_i].0) * local_t;
                let y = path[seg_i].1 + (path[seg_i + 1].1 - path[seg_i].1) * local_t;
                return (x, y);
            }
            accum += seg_len;
        }
        *path.last().unwrap()
    };

    let mut particles = Vec::with_capacity(num_particles);
    for i in 0..num_particles {
        let t = (phase + i as f64 / num_particles as f64) % 1.0;
        let head_dist = t * total_len;
        let head = point_at_dist(head_dist);

        let mut trail = Vec::with_capacity(trail_count);
        for ti in 1..=trail_count {
            let trail_dist = head_dist - trail_spacing * ti as f64;
            // Wrap around if trail goes past the start.
            let wrapped = if trail_dist < 0.0 {
                trail_dist + total_len
            } else {
                trail_dist
            };
            trail.push(point_at_dist(wrapped));
        }

        particles.push(ParticleWithTrail { head, trail });
    }
    particles
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
    let total_secs = secs.round() as u64;
    if total_secs == 0 {
        return "0s".to_string();
    }
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let s = total_secs % 60;
    if hours > 0 {
        format!("{}h {:02}m {:02}s", hours, mins, s)
    } else if mins > 0 {
        format!("{}m {:02}s", mins, s)
    } else {
        format!("{}s", s)
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
