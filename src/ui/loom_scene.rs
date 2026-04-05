//! The Loom of Worlds overlay — main UI renderer.
//!
//! Dispatches to different view renderers based on `LoomUiState::view`:
//!   - GraphView:           pipeline diagram with extractors and shuttles
//!   - Codex:              recipe codex

use crate::loom::types::{LoomState, LoomUiState, LoomView};
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

    let view_name = match ui.view {
        LoomView::GraphView => "Graph View",
        LoomView::Codex => "Recipe Codex",
    };

    let warp_label = if loom_state.time_warp > 1.0 {
        format!(" \u{23e9} {:.0}x ", loom_state.time_warp)
    } else {
        String::new()
    };
    let title = format!(" LOOM OF WORLDS \u{2014} {}{}", view_name, warp_label);
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

    // Minimum terminal size check for graph view.
    if ui.view == LoomView::GraphView && (area.width < 100 || area.height < 30) {
        let msg = Paragraph::new("Terminal too small for graph view (need 100\u{d7}30)")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Rgb(180, 120, 220)));
        frame.render_widget(msg, inner);
        return;
    }

    // Refresh graph data before rendering if we're in graph view.
    // Layout bounds must match the canvas coordinate space used by the renderer
    // (Braille: 2 dots/cell horizontal, 4 dots/cell vertical, on the 70% top area).
    if ui.view == LoomView::GraphView {
        let graph_area_height = (inner.height as f64 * 0.7) as f64;
        let canvas_width = (inner.width as f64) * 2.0; // Braille 2x horizontal
        let canvas_height = graph_area_height * 4.0; // Braille 4x vertical
        crate::loom::graph::refresh_graph(ui, loom_state, canvas_width, canvas_height);
    }

    match ui.view {
        LoomView::GraphView => {
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
        }
        LoomView::Codex => {
            render_codex(frame, inner, loom_state, ui);
        }
    }

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
            render_bottom_panel_shuttle(frame, inner, loom, *idx);
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

    let output_rate = shuttle.output_rate_tracker.rate_per_hour();
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

/// Resource lists for each codex column.
const CODEX_BASE: [crate::loom::types::Resource; 6] = {
    use crate::loom::types::Resource;
    [
        Resource::Ember,
        Resource::VoidEssence,
        Resource::Reflection,
        Resource::Memory,
        Resource::Silence,
        Resource::Resonance,
    ]
};

const CODEX_CONFLUENCE: [crate::loom::types::Resource; 6] = {
    use crate::loom::types::Resource;
    [
        Resource::ForgedLight,
        Resource::CondensedEmber,
        Resource::EmberEcho,
        Resource::PurifiedVoid,
        Resource::EchoGlass,
        Resource::StillbornSong,
    ]
};

const CODEX_TERMINAL: [crate::loom::types::Resource; 1] =
    [crate::loom::types::Resource::WovenReality];

/// Get the resource at a given (column, row) in the codex graph.
fn codex_resource_at(col: usize, row: usize) -> Option<crate::loom::types::Resource> {
    match col {
        0 => CODEX_BASE.get(row).copied(),
        1 => CODEX_CONFLUENCE.get(row).copied(),
        2 => CODEX_TERMINAL.get(row).copied(),
        _ => None,
    }
}

/// Check if a recipe is discovered in the codex.
fn is_recipe_discovered(
    codex: &[crate::loom::types::CodexEntry],
    recipe: &crate::loom::recipes::Recipe,
) -> bool {
    codex.iter().any(|e| {
        e.discovered
            && e.output == recipe.output
            && e.node_nature == recipe.node_nature
            && e.inputs.len() == 2
            && ((e.inputs[0] == recipe.input_a && e.inputs[1] == recipe.input_b)
                || (e.inputs[0] == recipe.input_b && e.inputs[1] == recipe.input_a))
    })
}

/// Check if a confluence resource has been discovered (at least one recipe producing it is known).
fn is_resource_discovered(
    codex: &[crate::loom::types::CodexEntry],
    resource: crate::loom::types::Resource,
) -> bool {
    use crate::loom::types::Resource;
    // Base resources are always discovered.
    matches!(
        resource,
        Resource::Ember
            | Resource::Reflection
            | Resource::VoidEssence
            | Resource::Memory
            | Resource::Silence
            | Resource::Resonance
    ) || codex.iter().any(|e| e.discovered && e.output == resource)
}

fn render_codex(frame: &mut Frame, area: Rect, loom_state: &LoomState, ui: &LoomUiState) {
    use crate::loom::recipes::{all_recipes, recipes_producing, recipes_using};

    let discovered_count = loom_state
        .persistent
        .codex
        .iter()
        .filter(|e| e.discovered)
        .count();
    let total_recipes = all_recipes().len();

    // Split area into top (topology graph ~60%) and bottom (detail panel ~40%).
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(58),
            Constraint::Percentage(38),
            Constraint::Length(1), // footer
        ])
        .split(area);
    let graph_area = chunks[0];
    let detail_area = chunks[1];
    let footer_area = chunks[2];

    // ── Top half: Topology graph ──────────────────────────────────────────────
    let mut graph_lines: Vec<Line<'static>> = Vec::new();

    graph_lines.push(Line::from(""));

    // Build header line with column labels.
    {
        let mut spans = vec![Span::raw("  ")];
        let header_style = Style::default().fg(Color::Rgb(140, 100, 180));
        spans.push(Span::styled(format!("{:<24}", "BASE"), header_style));
        spans.push(Span::styled(format!("{:<26}", "CONFLUENCE"), header_style));
        spans.push(Span::styled("TERMINAL", header_style));
        graph_lines.push(Line::from(spans));
    }
    graph_lines.push(Line::from(""));

    // Build the graph rows. We'll render the maximum column length (6 rows).
    let max_rows = 6usize;
    let codex = &loom_state.persistent.codex;

    // Primary connection map: base resources → their primary confluence target.
    // Used for rendering connection indicators.
    let connections: &[(usize, usize)] = &[
        (0, 0), // Ember → ForgedLight
        (1, 0), // VoidEssence → ForgedLight
        (3, 4), // Memory → EchoGlass
        (4, 4), // Silence → EchoGlass
        (5, 5), // Resonance → StillbornSong
    ];
    // Confluence → Terminal connections.
    let conf_to_term: &[usize] = &[0, 4]; // ForgedLight, EchoGlass → WovenReality

    for row in 0..max_rows {
        let mut spans: Vec<Span<'static>> = Vec::new();
        spans.push(Span::raw("  "));

        // ── Base column ──
        if let Some(res) = codex_resource_at(0, row) {
            let is_selected = ui.codex_column == 0 && ui.codex_row == row;
            let name = resource_name(&res);
            let emoji = resource_emoji(&res);
            let style = if is_selected {
                Style::default()
                    .fg(Color::Rgb(255, 255, 200))
                    .bg(Color::Rgb(60, 40, 80))
            } else {
                Style::default().fg(Color::Rgb(180, 140, 220))
            };
            let cursor = if is_selected { "\u{25b6} " } else { "  " };
            spans.push(Span::styled(format!("{}{} {}", cursor, emoji, name), style));
            // Pad to column width.
            let label_len = 2 + 2 + 1 + name.len(); // cursor(2) + emoji(2) + space(1) + name
            let pad = 22usize.saturating_sub(label_len);

            // Connection indicator: check if this base resource connects to a confluence.
            let has_connection = connections.iter().any(|(b, _)| *b == row);
            if has_connection {
                let pad_str = " ".repeat(pad.saturating_sub(3));
                spans.push(Span::styled(
                    format!("{}\u{2500}\u{2500}\u{25b6}", pad_str),
                    Style::default().fg(Color::Rgb(80, 60, 120)),
                ));
            } else {
                spans.push(Span::raw(" ".repeat(pad)));
            }
        } else {
            spans.push(Span::raw("                        "));
        }

        // ── Confluence column ──
        if let Some(res) = codex_resource_at(1, row) {
            let is_selected = ui.codex_column == 1 && ui.codex_row == row;
            let discovered = is_resource_discovered(codex, res);
            let name = if discovered {
                resource_name(&res)
            } else {
                "???"
            };
            let emoji = if discovered {
                resource_emoji(&res)
            } else {
                "\u{2753}"
            }; // ❓
            let tier_mark = if discovered { "\u{2726} " } else { "  " }; // ✦

            let style = if is_selected {
                Style::default()
                    .fg(Color::Rgb(255, 255, 200))
                    .bg(Color::Rgb(60, 40, 80))
            } else if discovered {
                Style::default().fg(Color::Rgb(200, 160, 240))
            } else {
                Style::default().fg(Color::Rgb(60, 45, 80))
            };

            let cursor = if is_selected { "\u{25b6} " } else { "  " };
            spans.push(Span::styled(
                format!("{}{}{} {}", cursor, tier_mark, emoji, name),
                style,
            ));
            // Pad and add terminal connection if applicable.
            let label_len = 2 + 2 + 2 + 1 + name.len();
            let pad = 24usize.saturating_sub(label_len);

            let has_term_conn = conf_to_term.contains(&row);
            if has_term_conn {
                let pad_str = " ".repeat(pad.saturating_sub(3));
                spans.push(Span::styled(
                    format!("{}\u{2500}\u{2500}\u{25b6}", pad_str),
                    Style::default().fg(Color::Rgb(80, 60, 120)),
                ));
            } else {
                spans.push(Span::raw(" ".repeat(pad)));
            }
        } else {
            spans.push(Span::raw("                          "));
        }

        // ── Terminal column ──
        if let Some(res) = codex_resource_at(2, row) {
            let is_selected = ui.codex_column == 2 && ui.codex_row == row;
            let discovered = is_resource_discovered(codex, res);
            let name = if discovered {
                resource_name(&res)
            } else {
                "???"
            };
            let emoji = if discovered {
                resource_emoji(&res)
            } else {
                "\u{2753}"
            };
            let star = if discovered { "\u{2605} " } else { "  " }; // ★

            let style = if is_selected {
                Style::default()
                    .fg(Color::Rgb(255, 255, 200))
                    .bg(Color::Rgb(60, 40, 80))
            } else if discovered {
                Style::default().fg(Color::Rgb(255, 215, 0))
            } else {
                Style::default().fg(Color::Rgb(60, 45, 80))
            };

            let cursor = if is_selected { "\u{25b6} " } else { "  " };
            spans.push(Span::styled(
                format!("{}{}{} {}", cursor, star, emoji, name),
                style,
            ));
        }

        graph_lines.push(Line::from(spans));
    }

    let graph_para = Paragraph::new(graph_lines)
        .alignment(Alignment::Left)
        .style(Style::default().bg(LOOM_BG));
    frame.render_widget(graph_para, graph_area);

    // ── Bottom half: Detail panel ─────────────────────────────────────────────
    let selected_resource = codex_resource_at(ui.codex_column, ui.codex_row);
    let mut detail_lines: Vec<Line<'static>> = Vec::new();

    if let Some(res) = selected_resource {
        let res_discovered = is_resource_discovered(codex, res);
        let display_name = if res_discovered {
            format!("{} {}", resource_emoji(&res), resource_name(&res))
        } else {
            "???".to_string()
        };

        // Separator + header.
        let sep_width = area.width as usize;
        let sep_line = format!(
            " \u{2500}\u{2500}\u{2500} {} {}",
            display_name,
            "\u{2500}".repeat(sep_width.saturating_sub(display_name.len() + 6))
        );
        detail_lines.push(Line::from(Span::styled(
            sep_line,
            Style::default().fg(Color::Rgb(100, 70, 140)),
        )));

        // Split detail area into two sub-columns.
        let producing = recipes_producing(res);
        let using = recipes_using(res);

        // "Made from:" column
        let made_from_header = Line::from(vec![
            Span::styled(
                "  Made from:                        ",
                Style::default().fg(Color::Rgb(140, 100, 180)),
            ),
            Span::styled("Used in:", Style::default().fg(Color::Rgb(140, 100, 180))),
        ]);
        detail_lines.push(made_from_header);

        let max_detail_rows = (detail_area.height as usize).saturating_sub(3);
        let max_entries = producing.len().max(using.len()).min(max_detail_rows);

        for i in 0..max_entries {
            let mut spans: Vec<Span<'static>> = Vec::new();

            // Left sub-column: "Made from" recipes.
            if i < producing.len() {
                let recipe = &producing[i];
                let disc = is_recipe_discovered(codex, recipe);
                if disc {
                    let text = format!(
                        "   {} + {} @ {} (x{:.1}) \u{2713}",
                        resource_name(&recipe.input_a),
                        resource_name(&recipe.input_b),
                        node_nature_name(recipe.node_nature),
                        recipe.amount,
                    );
                    // Pad to ~36 chars.
                    let padded = format!("{:<36}", text);
                    spans.push(Span::styled(
                        padded,
                        Style::default().fg(Color::Rgb(160, 130, 200)),
                    ));
                } else {
                    let text = "   ??? + ??? @ ??? \u{2192} ???";
                    let padded = format!("{:<36}", text);
                    spans.push(Span::styled(
                        padded,
                        Style::default().fg(Color::Rgb(60, 45, 80)),
                    ));
                }
            } else {
                spans.push(Span::raw("                                    "));
            }

            // Right sub-column: "Used in" recipes.
            if i < using.len() {
                let recipe = &using[i];
                let disc = is_recipe_discovered(codex, recipe);
                if disc {
                    let other_input = if recipe.input_a == res {
                        resource_name(&recipe.input_b)
                    } else {
                        resource_name(&recipe.input_a)
                    };
                    let output_discovered = is_resource_discovered(codex, recipe.output);
                    let output_name = if output_discovered {
                        resource_name(&recipe.output)
                    } else {
                        "???"
                    };
                    let text = format!(
                        "+ {} @ {} \u{2192} {} (x{:.1})",
                        other_input,
                        node_nature_name(recipe.node_nature),
                        output_name,
                        recipe.amount,
                    );
                    spans.push(Span::styled(
                        text,
                        Style::default().fg(Color::Rgb(160, 130, 200)),
                    ));
                } else {
                    spans.push(Span::styled(
                        "+ ??? @ ??? \u{2192} ???",
                        Style::default().fg(Color::Rgb(60, 45, 80)),
                    ));
                }
            }

            detail_lines.push(Line::from(spans));
        }

        // Show remaining undiscovered counts if we couldn't fit them all.
        let remaining_producing = producing
            .iter()
            .skip(max_entries)
            .filter(|r| !is_recipe_discovered(codex, r))
            .count();
        let remaining_using = using
            .iter()
            .skip(max_entries)
            .filter(|r| !is_recipe_discovered(codex, r))
            .count();

        if remaining_producing > 0 || remaining_using > 0 {
            let mut spans: Vec<Span<'static>> = Vec::new();
            if remaining_producing > 0 {
                let text = format!("   + {} undiscovered...", remaining_producing);
                spans.push(Span::styled(
                    format!("{:<36}", text),
                    Style::default().fg(Color::Rgb(60, 45, 80)),
                ));
            } else {
                spans.push(Span::raw("                                    "));
            }
            if remaining_using > 0 {
                spans.push(Span::styled(
                    format!("+ {} undiscovered...", remaining_using),
                    Style::default().fg(Color::Rgb(60, 45, 80)),
                ));
            }
            detail_lines.push(Line::from(spans));
        }
    }

    let detail_para = Paragraph::new(detail_lines)
        .alignment(Alignment::Left)
        .style(Style::default().bg(LOOM_BG));
    frame.render_widget(detail_para, detail_area);

    // ── Footer ────────────────────────────────────────────────────────────────
    let footer_line = Line::from(vec![
        Span::styled(
            format!("  {}/{} Discovered", discovered_count, total_recipes),
            Style::default().fg(Color::DarkGray),
        ),
        Span::raw("                    "),
        Span::styled(
            "\u{2190}\u{2192} Column  \u{2191}\u{2193} Resource  Tab: Flow",
            Style::default().fg(Color::Rgb(80, 60, 120)),
        ),
    ]);
    let footer_para = Paragraph::new(vec![footer_line])
        .alignment(Alignment::Left)
        .style(Style::default().bg(LOOM_BG));
    frame.render_widget(footer_para, footer_area);
}

/// Returns the display name for a NodeNature.
fn node_nature_name(nature: crate::loom::types::NodeNature) -> &'static str {
    use crate::loom::types::NodeNature;
    match nature {
        NodeNature::Heat => "Heat",
        NodeNature::Form => "Form",
        NodeNature::Void => "Void",
        NodeNature::Pattern => "Pattern",
        NodeNature::Stillness => "Stillness",
        NodeNature::Vibration => "Vibration",
    }
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
    } else if ui.view == LoomView::GraphView {
        " [Tab] Switch View  [Arrows] Navigate  [U] Upgrade  [B] Build  [D] Demolish  [Esc] Close "
    } else {
        " [Tab] Switch View  [Up/Down] Navigate  [Esc] Close "
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
