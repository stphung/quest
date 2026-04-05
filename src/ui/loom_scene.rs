//! The Loom of Worlds overlay — main UI renderer.
//!
//! Renders the graph view: pipeline diagram with extractors and shuttles.

use crate::loom::types::{LoomState, LoomUiState};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, Paragraph},
    Frame,
};

/// Border color for the Loom overlay.
const LOOM_BORDER_COLOR: Color = Color::Rgb(180, 120, 220);

/// Background color for the Loom overlay interior.
const LOOM_BG: Color = Color::Rgb(10, 5, 18);

/// Per-node identity colors used for port labels and highlighting.
fn node_color(id: crate::loom::types::NodeId) -> Color {
    use crate::loom::types::NodeId;
    match id {
        NodeId::EmberSpindle => Color::Rgb(255, 140, 50),
        NodeId::VoidCondenser => Color::Rgb(160, 80, 220),
        NodeId::ReflectionLens => Color::Rgb(80, 200, 220),
        NodeId::MemoryArchive => Color::Rgb(220, 200, 80),
        NodeId::SilenceWell => Color::Rgb(140, 140, 160),
        NodeId::ResonanceForge => Color::Rgb(80, 140, 255),
    }
}

/// Emoji icon for each resource, used as visual shorthand in sidebar and recipes.
fn resource_emoji(resource: &crate::loom::types::Resource) -> &'static str {
    use crate::loom::types::Resource;
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

/// Emoji icon for a node identity.
fn node_emoji(id: crate::loom::types::NodeId) -> &'static str {
    use crate::loom::types::NodeId;
    match id {
        NodeId::EmberSpindle => "\u{1f525}",   // 🔥
        NodeId::ReflectionLens => "\u{1f48e}", // 💎
        NodeId::VoidCondenser => "\u{1f300}",  // 🌀
        NodeId::MemoryArchive => "\u{1f4dc}",  // 📜
        NodeId::SilenceWell => "\u{1f311}",    // 🌑
        NodeId::ResonanceForge => "\u{1f514}", // 🔔
    }
}

// ── Main entry point ──────────────────────────────────────────────────────────

/// Render the Loom of Worlds overlay.
pub fn render_loom_overlay(
    frame: &mut Frame,
    area: Rect,
    loom_state: &mut LoomState,
    ui: &mut LoomUiState,
) {
    ui.throbber_frame = ui.throbber_frame.wrapping_add(1);
    frame.render_widget(Clear, area);

    let warp_label = if loom_state.time_warp > 1.0 {
        format!(" \u{23e9} {:.0}x ", loom_state.time_warp)
    } else {
        String::new()
    };
    let title = format!(" LOOM OF WORLDS \u{2014} Graph View{}", warp_label);
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(LOOM_BORDER_COLOR));

    let inner = {
        let inner = block.inner(area);
        frame.render_widget(block, area);
        inner
    };

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    // Minimum terminal size check.
    if area.width < 100 || area.height < 30 {
        let msg = Paragraph::new("Terminal too small for graph view (need 100\u{d7}30)")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Rgb(180, 120, 220)));
        frame.render_widget(msg, inner);
        return;
    }

    // Refresh graph data before rendering.
    // Layout bounds must match the canvas coordinate space used by the renderer
    // (Braille: 2 dots/cell horizontal, 4 dots/cell vertical, on the 70% top area).
    let graph_area_height = (inner.height as f64 * 0.7) as f64;
    let canvas_width = (inner.width as f64) * 2.0; // Braille 2x horizontal
    let canvas_height = graph_area_height * 4.0; // Braille 4x vertical
    crate::loom::graph::refresh_graph(ui, loom_state, canvas_width, canvas_height);

    // Split: top 70% graph canvas, bottom 30% detail panel.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(inner);

    // Render graph canvas if graph and layout are cached.
    if let (Some(ref graph), Some(ref layout_data)) = (&ui.loom_graph, &ui.loom_layout) {
        crate::ui::loom_graph::render_graph_canvas(
            frame,
            chunks[0],
            graph,
            layout_data,
            ui,
            loom_state,
        );
    } else {
        // Graph not yet built — show a brief loading message.
        let msg = Paragraph::new("Building graph\u{2026}")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Rgb(140, 100, 180)));
        frame.render_widget(msg, chunks[0]);
    }

    render_bottom_panel(frame, chunks[1], loom_state, ui);

    // Render build overlay on top if active.
    if ui.build.is_some() {
        render_build_overlay(frame, inner, loom_state, ui);
    }

    render_nav_hints(frame, area, &*ui);
}

// ── View renderers ────────────────────────────────────────────────────────────

// ── Bottom panel (Graph View detail) ─────────────────────────────────────────

/// Render the bottom detail panel in Graph View.
///
/// Shows contextual information depending on what is selected:
/// - Build flow in progress: step summary
/// - Extractor node: name, level, rate, buffer, upgrade cost
/// - Shuttle node: recipe, tier, level, buffer, output rate, upgrade cost
/// - Pattern sink: name, requirements with progress bars
/// - Nothing selected: introductory guidance
fn render_bottom_panel(frame: &mut Frame, area: Rect, loom: &LoomState, ui: &LoomUiState) {
    use crate::loom::graph::LoomGraphNode;

    let current_shuttles = loom.persistent.shuttles.len();
    let max_shuttles = loom.persistent.max_shuttles();
    let title = format!(
        " Detail  [Shuttles: {}/{}] ",
        current_shuttles, max_shuttles
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(LOOM_BORDER_COLOR));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width < 10 {
        return;
    }

    // ── Build flow active ────────────────────────────────────────────────────
    if let Some(build) = &ui.build {
        let recipes = crate::loom::recipes::all_recipes();
        let lines: Vec<Line> = match &build.step {
            crate::loom::BuildStep::SelectRecipe { .. } => {
                vec![
                    Line::from(Span::styled(
                        " Building Shuttle...",
                        Style::default().fg(Color::Rgb(180, 140, 220)),
                    )),
                    Line::from(Span::styled(
                        format!(" Step 1/4: Select recipe (T{})", build.tier),
                        Style::default().fg(Color::Rgb(140, 110, 170)),
                    )),
                ]
            }
            crate::loom::BuildStep::SelectSourcesA { .. } => {
                let r = &recipes[build.recipe_index];
                vec![
                    Line::from(Span::styled(
                        format!(
                            " {} + {} \u{2192} {}",
                            resource_name(&r.input_a),
                            resource_name(&r.input_b),
                            resource_name(&r.output),
                        ),
                        Style::default().fg(Color::Rgb(180, 140, 220)),
                    )),
                    Line::from(Span::styled(
                        format!(
                            " Step 2/4: Select sources for {}",
                            resource_name(&r.input_a)
                        ),
                        Style::default().fg(Color::Rgb(140, 110, 170)),
                    )),
                ]
            }
            crate::loom::BuildStep::SelectSourcesB { .. } => {
                let r = &recipes[build.recipe_index];
                vec![
                    Line::from(Span::styled(
                        format!(
                            " {} + {} \u{2192} {}",
                            resource_name(&r.input_a),
                            resource_name(&r.input_b),
                            resource_name(&r.output),
                        ),
                        Style::default().fg(Color::Rgb(180, 140, 220)),
                    )),
                    Line::from(Span::styled(
                        format!(
                            " Step 3/4: Select sources for {}",
                            resource_name(&r.input_b)
                        ),
                        Style::default().fg(Color::Rgb(140, 110, 170)),
                    )),
                ]
            }
            crate::loom::BuildStep::Confirm => {
                let r = &recipes[build.recipe_index];
                vec![
                    Line::from(Span::styled(
                        format!(
                            " {} + {} \u{2192} {}",
                            resource_name(&r.input_a),
                            resource_name(&r.input_b),
                            resource_name(&r.output),
                        ),
                        Style::default().fg(Color::Rgb(180, 140, 220)),
                    )),
                    Line::from(Span::styled(
                        " Step 4/4: Confirm build [Enter]",
                        Style::default().fg(Color::Rgb(100, 200, 120)),
                    )),
                ]
            }
            crate::loom::BuildStep::Blocked { message } => {
                vec![
                    Line::from(Span::styled(
                        " Build Blocked",
                        Style::default().fg(Color::Rgb(220, 60, 60)),
                    )),
                    Line::from(Span::styled(
                        format!(" {}", message),
                        Style::default().fg(Color::Rgb(160, 100, 100)),
                    )),
                ]
            }
        };
        frame.render_widget(
            Paragraph::new(lines).style(Style::default().bg(LOOM_BG)),
            inner,
        );
        return;
    }

    // ── Node selected ────────────────────────────────────────────────────────
    let (graph, selected_ni) = match (&ui.loom_graph, ui.selected_graph_node) {
        (Some(g), Some(ni)) => (g, ni),
        _ => {
            // Nothing selected — show context-aware guidance.
            let msg = if loom.persistent.shuttles.is_empty() {
                " Press [B] to build your first shuttle."
            } else {
                " Use arrow keys to navigate the graph."
            };
            let lines = vec![
                Line::from(""),
                Line::from(Span::styled(
                    msg,
                    Style::default().fg(Color::Rgb(140, 110, 170)),
                )),
            ];
            frame.render_widget(
                Paragraph::new(lines).style(Style::default().bg(LOOM_BG)),
                inner,
            );
            return;
        }
    };

    let graph_node = match graph.graph.node_weight(selected_ni) {
        Some(n) => n,
        None => return,
    };

    match graph_node {
        LoomGraphNode::Extractor(node_id) => {
            render_bottom_panel_extractor(frame, inner, loom, *node_id);
        }
        LoomGraphNode::Shuttle(idx) => {
            if ui.demolish_pending {
                let name = if *idx < loom.persistent.shuttles.len() {
                    let s = &loom.persistent.shuttles[*idx];
                    format!("S{} ({:?})", idx, s.output)
                } else {
                    format!("S{}", idx)
                };
                let lines = vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        format!(" Demolish {}?", name),
                        Style::default().fg(Color::Rgb(255, 100, 100)),
                    )),
                    Line::from(Span::styled(
                        " Press [D] again to confirm, any other key to cancel.",
                        Style::default().fg(Color::Rgb(180, 140, 140)),
                    )),
                ];
                frame.render_widget(
                    Paragraph::new(lines).style(Style::default().bg(LOOM_BG)),
                    inner,
                );
            } else {
                render_bottom_panel_shuttle(frame, inner, loom, *idx);
            }
        }
        LoomGraphNode::PatternSink(pat_idx) => {
            render_bottom_panel_pattern(frame, inner, loom, *pat_idx);
        }
    }
}

/// Render bottom panel content for an Extractor node.
fn render_bottom_panel_extractor(
    frame: &mut Frame,
    area: Rect,
    loom: &LoomState,
    node_id: crate::loom::types::NodeId,
) {
    use crate::loom::logic;

    let node = match loom.persistent.nodes.iter().find(|n| n.id == node_id) {
        Some(n) => n,
        None => return,
    };

    // Split into 2 columns: info (left) | buffer gauge + upgrade (right).
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    // ── Left column: identity and stats ──
    let emoji = node_emoji(node_id);
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        format!(" {} {}", emoji, node_id.name()),
        Style::default().fg(node_color(node_id)),
    )));

    if !node.unlocked {
        lines.push(Line::from(Span::styled(
            " [Locked]",
            Style::default().fg(Color::Rgb(80, 60, 110)),
        )));
        if node.unlock_progress > 0.0 {
            lines.push(Line::from(Span::styled(
                format!(" {:.1}/2.0h unlocking", node.unlock_progress),
                Style::default().fg(Color::Rgb(100, 80, 160)),
            )));
        }
    } else {
        let rate = logic::node_effective_rate(loom, node);
        lines.push(Line::from(Span::styled(
            format!(" Lv {} \u{2022} {:.0}/hr", node.level, rate),
            Style::default().fg(Color::Rgb(100, 200, 120)),
        )));

        // Consumer count.
        let node_ref = crate::loom::types::LoomNodeRef::Extractor(node.id);
        let consumer_count = loom
            .persistent
            .shuttles
            .iter()
            .filter(|s| {
                !s.under_construction
                    && (s.sources_a.contains(&node_ref) || s.sources_b.contains(&node_ref))
            })
            .count();
        if consumer_count > 0 {
            lines.push(Line::from(Span::styled(
                format!(
                    " {} consumer{}",
                    consumer_count,
                    if consumer_count == 1 { "" } else { "s" }
                ),
                Style::default().fg(Color::Rgb(120, 100, 160)),
            )));
        }
    }

    lines.truncate(area.height as usize);
    frame.render_widget(
        Paragraph::new(lines).style(Style::default().bg(LOOM_BG)),
        cols[0],
    );

    // ── Right column: buffer gauge + upgrade ──
    if !node.unlocked {
        return;
    }

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(cols[1]);

    // Buffer gauge.
    let fill = if node.buffer_capacity > 0.0 {
        (node.buffer / node.buffer_capacity).min(1.0)
    } else {
        0.0
    };
    let bar_color = if node.stalled || fill >= 0.90 {
        Color::Rgb(220, 60, 60)
    } else if fill >= 0.75 {
        Color::Rgb(220, 180, 60)
    } else {
        Color::Rgb(60, 200, 100)
    };
    let label = format!("{:.0}/{:.0}", node.buffer, node.buffer_capacity);
    let gauge = Gauge::default()
        .ratio(fill)
        .label(label)
        .gauge_style(Style::default().fg(bar_color).bg(Color::Rgb(30, 20, 40)));
    frame.render_widget(gauge, right_chunks[0]);

    // Upgrade info.
    let resource = logic::node_native_resource(node.id);
    let re = resource_emoji(&resource);
    let upgrade_line = if node.level < logic::MAX_NODE_LEVEL {
        let cost = logic::node_upgrade_cost(loom, node.id);
        let can_afford = node.buffer >= cost;
        let color = if can_afford {
            Color::Rgb(100, 200, 120)
        } else {
            Color::Rgb(80, 60, 100)
        };
        Line::from(vec![
            Span::styled(
                " [U] ",
                Style::default().fg(if can_afford {
                    Color::Rgb(200, 180, 240)
                } else {
                    Color::DarkGray
                }),
            ),
            Span::styled(
                format!("Lv{} ({:.0}{} cost)", node.level + 1, cost, re),
                Style::default().fg(color),
            ),
        ])
    } else {
        Line::from(Span::styled(
            " Max Level",
            Style::default().fg(Color::Rgb(100, 200, 120)),
        ))
    };
    frame.render_widget(
        Paragraph::new(vec![upgrade_line]).style(Style::default().bg(LOOM_BG)),
        right_chunks[1],
    );
}

/// Render bottom panel content for a Shuttle node.
fn render_bottom_panel_shuttle(
    frame: &mut Frame,
    area: Rect,
    loom: &LoomState,
    shuttle_idx: usize,
) {
    use crate::loom::logic;

    let shuttle = match loom.persistent.shuttles.get(shuttle_idx) {
        Some(s) => s,
        None => {
            let lines = vec![Line::from(Span::styled(
                " [No shuttle]",
                Style::default().fg(Color::Rgb(80, 60, 110)),
            ))];
            frame.render_widget(
                Paragraph::new(lines).style(Style::default().bg(LOOM_BG)),
                area,
            );
            return;
        }
    };

    if shuttle.under_construction {
        let ticks = shuttle.construction_ticks_remaining;
        let warp = loom.time_warp.max(1.0) as u32;
        let secs = ticks / (10 * warp);
        let lines = vec![
            Line::from(Span::styled(
                format!(" Shuttle {} \u{2014} Under Construction", shuttle_idx),
                Style::default().fg(Color::Rgb(160, 120, 200)),
            )),
            Line::from(Span::styled(
                format!(" ~{}s remaining", secs),
                Style::default().fg(Color::Rgb(100, 80, 130)),
            )),
        ];
        frame.render_widget(
            Paragraph::new(lines).style(Style::default().bg(LOOM_BG)),
            area,
        );
        return;
    }

    // Split into 3 columns: recipe | buffer+rate | status+upgrade.
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35),
            Constraint::Percentage(35),
            Constraint::Percentage(30),
        ])
        .split(area);

    // ── Left column: recipe and tier ──
    let ea = resource_emoji(&shuttle.input_a);
    let eb = resource_emoji(&shuttle.input_b);
    let eo = resource_emoji(&shuttle.output);
    let mut left_lines: Vec<Line> = vec![
        Line::from(Span::styled(
            format!(
                " Shuttle {} \u{2022} T{} Lv{}",
                shuttle_idx, shuttle.tier, shuttle.level
            ),
            Style::default().fg(Color::Rgb(180, 140, 220)),
        )),
        Line::from(Span::styled(
            format!(
                " {}{}+{}{}\u{2192}{}{}",
                ea,
                resource_name(&shuttle.input_a),
                eb,
                resource_name(&shuttle.input_b),
                eo,
                resource_name(&shuttle.output),
            ),
            Style::default().fg(Color::Rgb(160, 120, 200)),
        )),
        Line::from(Span::styled(
            format!(" Yield: x{:.1}/cycle", shuttle.amount),
            Style::default().fg(Color::Rgb(100, 200, 120)),
        )),
    ];
    left_lines.truncate(area.height as usize);
    frame.render_widget(
        Paragraph::new(left_lines).style(Style::default().bg(LOOM_BG)),
        cols[0],
    );

    // ── Middle column: buffer gauge + output rate ──
    let mid_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(cols[1]);

    let fill = if shuttle.buffer_capacity > 0.0 {
        (shuttle.buffer / shuttle.buffer_capacity).min(1.0)
    } else {
        0.0
    };
    let bar_color = if fill >= 0.90 {
        Color::Rgb(220, 60, 60)
    } else if fill >= 0.75 {
        Color::Rgb(220, 180, 60)
    } else {
        Color::Rgb(60, 200, 100)
    };
    let gauge_label = format!("{:.0}/{:.0}", shuttle.buffer, shuttle.buffer_capacity);
    let gauge = Gauge::default()
        .ratio(fill)
        .label(gauge_label)
        .gauge_style(Style::default().fg(bar_color).bg(Color::Rgb(30, 20, 40)));
    frame.render_widget(gauge, mid_chunks[0]);

    let output_rate = shuttle.output_rate_tracker.rate_per_hour() / loom.time_warp.max(1.0);
    let mut mid_lines: Vec<Line> = vec![Line::from(Span::styled(
        format!(
            " Output: {:.0}/hr {}",
            output_rate,
            resource_name(&shuttle.output)
        ),
        Style::default().fg(Color::Rgb(100, 200, 120)),
    ))];
    let cap = logic::shuttle_effective_intake_cap(shuttle.tier, shuttle.level);
    mid_lines.push(Line::from(Span::styled(
        format!(" Intake cap: {:.0}/hr", cap),
        Style::default().fg(Color::Rgb(120, 100, 160)),
    )));
    mid_lines.truncate(mid_chunks[1].height as usize);
    frame.render_widget(
        Paragraph::new(mid_lines).style(Style::default().bg(LOOM_BG)),
        mid_chunks[1],
    );

    // ── Right column: status + upgrade ──
    let status_text = if shuttle.stalled {
        " STALLED"
    } else {
        " Running"
    };
    let status_color = if shuttle.stalled {
        Color::Rgb(220, 60, 60)
    } else {
        Color::Rgb(80, 200, 120)
    };

    let mut right_lines: Vec<Line> = vec![Line::from(Span::styled(
        status_text,
        Style::default().fg(status_color),
    ))];

    // Upgrade cost (shuttle upgrade costs 100 * level^1.2 from buffer).
    let cost = 100.0 * (shuttle.level as f64).powf(1.2);
    let can_afford = shuttle.buffer >= cost;
    let cost_color = if can_afford {
        Color::Rgb(100, 200, 120)
    } else {
        Color::Rgb(80, 60, 100)
    };
    right_lines.push(Line::from(vec![
        Span::styled(
            " [U] ",
            Style::default().fg(if can_afford {
                Color::Rgb(200, 180, 240)
            } else {
                Color::DarkGray
            }),
        ),
        Span::styled(
            format!("Lv{} ({:.0} cost)", shuttle.level + 1, cost),
            Style::default().fg(cost_color),
        ),
    ]));
    right_lines.push(Line::from(Span::styled(
        " [D]emolish",
        Style::default().fg(Color::DarkGray),
    )));
    right_lines.truncate(area.height as usize);
    frame.render_widget(
        Paragraph::new(right_lines).style(Style::default().bg(LOOM_BG)),
        cols[2],
    );
}

/// Render bottom panel content for a PatternSink node.
fn render_bottom_panel_pattern(frame: &mut Frame, area: Rect, loom: &LoomState, pat_idx: usize) {
    let pattern = match loom.persistent.patterns.get(pat_idx) {
        Some(p) => p,
        None => return,
    };

    let completed_marker = if pattern.completed { " \u{2713}" } else { "" };
    let title_color = if pattern.completed {
        Color::Rgb(100, 200, 120)
    } else {
        Color::Rgb(180, 140, 220)
    };

    // First line: pattern name.
    let mut lines: Vec<Line> = vec![Line::from(Span::styled(
        format!(
            " Pattern #{}: {}{}",
            pat_idx + 1,
            pattern.name,
            completed_marker
        ),
        Style::default().fg(title_color),
    ))];

    // Split remaining area into requirement rows.
    // Each requirement: resource emoji, name, required rate, sustained time progress.
    for req in &pattern.requirements {
        let re = resource_emoji(&req.resource);
        let rn = resource_name(&req.resource);
        let req_completed = req.completed;
        let progress = if req.sustain_duration_secs > 0.0 {
            (req.sustained_secs / req.sustain_duration_secs).min(1.0)
        } else if req_completed {
            1.0
        } else {
            0.0
        };

        let status_color = if req_completed {
            Color::Rgb(100, 200, 120)
        } else if progress > 0.0 {
            Color::Rgb(220, 180, 60)
        } else {
            Color::Rgb(80, 60, 110)
        };

        let sustained_mins = req.sustained_secs / 60.0;
        let total_mins = req.sustain_duration_secs / 60.0;
        let bar_width = 10usize;
        let filled = (progress * bar_width as f64).round() as usize;
        let bar_str: String = (0..bar_width)
            .map(|i| if i < filled { '\u{2588}' } else { '\u{2591}' })
            .collect();

        lines.push(Line::from(vec![
            Span::styled(format!(" {}{} ", re, rn), Style::default().fg(status_color)),
            Span::styled(
                format!("{:.0}/hr ", req.required_rate),
                Style::default().fg(Color::Rgb(140, 110, 170)),
            ),
            Span::styled(bar_str, Style::default().fg(status_color)),
            Span::styled(
                format!(" {:.0}/{:.0}m", sustained_mins, total_mins),
                Style::default().fg(Color::Rgb(120, 100, 160)),
            ),
            if req_completed {
                Span::styled(" \u{2713}", Style::default().fg(Color::Rgb(100, 200, 120)))
            } else {
                Span::raw("")
            },
        ]));
    }

    lines.truncate(area.height as usize);
    frame.render_widget(
        Paragraph::new(lines).style(Style::default().bg(LOOM_BG)),
        area,
    );
}

// ── Build Shuttle Overlay ────────────────────────────────────────────────────

fn render_build_overlay(frame: &mut Frame, area: Rect, loom_state: &LoomState, ui: &LoomUiState) {
    let build = match &ui.build {
        Some(b) => b,
        None => return,
    };

    let popup_w = area.width.clamp(30, 50);
    let popup_h = area.height.clamp(10, 20);
    let popup_x = area.x + (area.width.saturating_sub(popup_w)) / 2;
    let popup_y = area.y + (area.height.saturating_sub(popup_h)) / 2;
    let popup = Rect::new(popup_x, popup_y, popup_w, popup_h);

    frame.render_widget(Clear, popup);

    let recipes = crate::loom::recipes::all_recipes();

    let (title, lines) = match &build.step {
        crate::loom::BuildStep::SelectRecipe { cursor } => {
            let mut lines = Vec::new();
            let unlocked_tiers = crate::loom::unlocked_tiers(loom_state);
            let tier_labels: String = unlocked_tiers
                .iter()
                .map(|t| {
                    if *t == build.tier {
                        format!("[T{}]", t)
                    } else {
                        format!(" T{} ", t)
                    }
                })
                .collect::<Vec<_>>()
                .join("");
            lines.push(Line::from(Span::styled(
                format!(" {} Recipes:", tier_labels),
                Style::default().fg(Color::Rgb(180, 140, 220)),
            )));
            if unlocked_tiers.len() > 1 {
                lines.push(Line::from(Span::styled(
                    " [Tab] Switch Tier",
                    Style::default().fg(Color::DarkGray),
                )));
            }
            lines.push(Line::from(""));
            for (i, &ridx) in build.available_recipes.iter().enumerate() {
                let r = &recipes[ridx];
                let marker = if i == *cursor { "\u{25b6} " } else { "  " };
                let color = if i == *cursor {
                    Color::White
                } else {
                    Color::Rgb(140, 110, 170)
                };
                lines.push(Line::from(Span::styled(
                    format!(
                        "{}{} + {} \u{2192} {} ({:.1}x)",
                        marker,
                        resource_name(&r.input_a),
                        resource_name(&r.input_b),
                        resource_name(&r.output),
                        r.amount,
                    ),
                    Style::default().fg(color),
                )));
            }
            lines.push(Line::from(""));
            let cost = crate::loom::shuttle_build_cost_public(build.tier);
            lines.push(Line::from(Span::styled(
                format!(" Build cost: {:.0} of input A resource", cost),
                Style::default().fg(Color::Rgb(100, 80, 130)),
            )));
            (" Build Shuttle ", lines)
        }
        crate::loom::BuildStep::SelectSourcesA { cursor, toggle } => {
            let r = &recipes[build.recipe_index];
            let mut lines = Vec::new();
            lines.push(Line::from(Span::styled(
                format!(
                    " {} + {} \u{2192} {}",
                    resource_name(&r.input_a),
                    resource_name(&r.input_b),
                    resource_name(&r.output)
                ),
                Style::default().fg(Color::Rgb(180, 140, 220)),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!(" Select sources for {}:", resource_name(&r.input_a)),
                Style::default().fg(Color::Rgb(140, 110, 170)),
            )));
            for (i, src) in build.eligible_sources_a.iter().enumerate() {
                let marker = if i == *cursor { "\u{25b6}" } else { " " };
                let check = if toggle[i] { "[\u{2713}]" } else { "[ ]" };
                let name = source_display_name(src, loom_state);
                let color = if i == *cursor {
                    Color::White
                } else {
                    Color::Rgb(140, 110, 170)
                };
                lines.push(Line::from(Span::styled(
                    format!(" {} {} {}", marker, check, name),
                    Style::default().fg(color),
                )));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                " [Space] Toggle  [Enter] Next",
                Style::default().fg(Color::DarkGray),
            )));
            (" Sources: Input A ", lines)
        }
        crate::loom::BuildStep::SelectSourcesB { cursor, toggle } => {
            let r = &recipes[build.recipe_index];
            let mut lines = Vec::new();
            lines.push(Line::from(Span::styled(
                format!(
                    " {} + {} \u{2192} {}",
                    resource_name(&r.input_a),
                    resource_name(&r.input_b),
                    resource_name(&r.output)
                ),
                Style::default().fg(Color::Rgb(180, 140, 220)),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!(" Select sources for {}:", resource_name(&r.input_b)),
                Style::default().fg(Color::Rgb(140, 110, 170)),
            )));
            for (i, src) in build.eligible_sources_b.iter().enumerate() {
                let marker = if i == *cursor { "\u{25b6}" } else { " " };
                let check = if toggle[i] { "[\u{2713}]" } else { "[ ]" };
                let name = source_display_name(src, loom_state);
                let color = if i == *cursor {
                    Color::White
                } else {
                    Color::Rgb(140, 110, 170)
                };
                lines.push(Line::from(Span::styled(
                    format!(" {} {} {}", marker, check, name),
                    Style::default().fg(color),
                )));
            }
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                " [Space] Toggle  [Enter] Next",
                Style::default().fg(Color::DarkGray),
            )));
            (" Sources: Input B ", lines)
        }
        crate::loom::BuildStep::Confirm => {
            let r = &recipes[build.recipe_index];
            let mut lines = Vec::new();
            lines.push(Line::from(Span::styled(
                format!(
                    " {} + {} \u{2192} {}",
                    resource_name(&r.input_a),
                    resource_name(&r.input_b),
                    resource_name(&r.output)
                ),
                Style::default().fg(Color::Rgb(180, 140, 220)),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                " Sources A:",
                Style::default().fg(Color::Rgb(140, 110, 170)),
            )));
            for src in &build.selected_sources_a {
                lines.push(Line::from(Span::styled(
                    format!("   {}", source_display_name(src, loom_state)),
                    Style::default().fg(Color::Rgb(120, 100, 160)),
                )));
            }
            lines.push(Line::from(Span::styled(
                " Sources B:",
                Style::default().fg(Color::Rgb(140, 110, 170)),
            )));
            for src in &build.selected_sources_b {
                lines.push(Line::from(Span::styled(
                    format!("   {}", source_display_name(src, loom_state)),
                    Style::default().fg(Color::Rgb(120, 100, 160)),
                )));
            }
            lines.push(Line::from(""));
            let cost = crate::loom::shuttle_build_cost_public(build.tier);
            let available = crate::loom::logic::available_resource(loom_state, r.input_a);
            let can_afford = available >= cost;
            let cost_color = if can_afford {
                Color::Rgb(100, 180, 100)
            } else {
                Color::Rgb(180, 80, 80)
            };
            lines.push(Line::from(Span::styled(
                format!(
                    " Cost: {:.0} {} (have {:.0})",
                    cost,
                    resource_name(&r.input_a),
                    available
                ),
                Style::default().fg(cost_color),
            )));
            lines.push(Line::from(""));
            if can_afford {
                lines.push(Line::from(Span::styled(
                    " [Enter] Build  [Esc] Cancel",
                    Style::default().fg(Color::DarkGray),
                )));
            } else {
                lines.push(Line::from(Span::styled(
                    " Insufficient resources. [Esc] Cancel",
                    Style::default().fg(Color::Rgb(180, 80, 80)),
                )));
            }
            (" Confirm Build ", lines)
        }
        crate::loom::BuildStep::Blocked { message } => {
            let mut lines = Vec::new();
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!(" {}", message),
                Style::default().fg(Color::Rgb(180, 120, 80)),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                " Press any key to dismiss",
                Style::default().fg(Color::DarkGray),
            )));
            (" Cannot Build ", lines)
        }
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(160, 120, 200)));
    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let para = Paragraph::new(lines);
    frame.render_widget(para, inner);
}

fn source_display_name(src: &crate::loom::types::LoomNodeRef, loom_state: &LoomState) -> String {
    match src {
        crate::loom::types::LoomNodeRef::Extractor(node_id) => {
            let rate = loom_state
                .persistent
                .nodes
                .iter()
                .find(|n| n.id == *node_id)
                .map(|n| crate::loom::node_effective_rate(loom_state, n))
                .unwrap_or(0.0);
            format!("{} ({:.0}/hr)", node_id.name(), rate)
        }
        crate::loom::types::LoomNodeRef::Shuttle(idx) => {
            if let Some(r) = loom_state.persistent.shuttles.get(*idx) {
                format!("R{} {} ({:.1}x)", idx, resource_name(&r.output), r.amount)
            } else {
                format!("R{} (unknown)", idx)
            }
        }
    }
}

// ── Navigation hints ──────────────────────────────────────────────────────────

fn render_nav_hints(frame: &mut Frame, area: Rect, ui: &LoomUiState) {
    if area.height < 3 {
        return;
    }

    let hints = if ui.build.is_some() {
        " [Up/Down] Select  [Space] Toggle  [Enter] Confirm  [Esc] Cancel "
    } else {
        " [Arrows] Navigate  [U] Upgrade  [B] Build  [D] Demolish  [Esc] Close "
    };

    let hint_line = Line::from(Span::styled(hints, Style::default().fg(Color::DarkGray)));

    let hint_area = Rect::new(
        area.x + 1,
        area.y + area.height - 1,
        area.width.saturating_sub(2),
        1,
    );

    let para = Paragraph::new(hint_line).alignment(Alignment::Center);
    frame.render_widget(para, hint_area);
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn resource_name(resource: &crate::loom::types::Resource) -> &'static str {
    use crate::loom::types::Resource;
    match resource {
        Resource::Ember => "Ember",
        Resource::Reflection => "Reflection",
        Resource::VoidEssence => "Void Essence",
        Resource::Memory => "Memory",
        Resource::Silence => "Silence",
        Resource::Resonance => "Resonance",
        Resource::ForgedLight => "Forged Light",
        Resource::EchoGlass => "Echo Glass",
        Resource::StillbornSong => "Stillborn Song",
        Resource::CondensedEmber => "Condensed Ember",
        Resource::EmberEcho => "Ember Echo",
        Resource::PurifiedVoid => "Purified Void",
        Resource::WovenReality => "Woven Reality",
    }
}

// ── Discovery modal ─────────────────────────────────────────────────────────

pub fn render_loom_discovery_modal(
    frame: &mut Frame,
    area: Rect,
    _ctx: &super::responsive::LayoutContext,
) {
    let modal_width = 56u16.min(area.width.saturating_sub(4));
    let modal_height = 12u16.min(area.height.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(" \u{25b6} New System Unlocked \u{25c0} ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(super::themed_border_color(LOOM_BORDER_COLOR)));
    let inner = super::render_themed_block(
        frame,
        modal_area,
        block,
        LOOM_BORDER_COLOR,
        super::BorderFxContext,
    );

    let text = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "\u{1f9f5} The Loom of Worlds awakens!",
            Style::default().fg(Color::LightMagenta),
        )),
        Line::from(""),
        Line::from("A vast pipeline network hums with"),
        Line::from("potential. Six nodes await your command."),
        Line::from(""),
        Line::from(Span::styled(
            "Press [L] to open the Loom.",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "[Enter/Esc] Dismiss",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .alignment(Alignment::Center);
    frame.render_widget(text, inner);
}
