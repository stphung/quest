//! The Loom of Worlds overlay — main UI renderer.
//!
//! Renders the graph view: pipeline diagram with extractors and shuttles.

use crate::loom::types::{LoomState, LoomUiState};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
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
        Resource::CondensedEmber => "\u{1f4a0}", // 💠
        Resource::EmberEcho => "\u{1fae7}",      // 🫧
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
    let title = format!(" LOOM OF WORLDS {}", warp_label);
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
    // (Braille: 2 dots/cell horizontal, 4 dots/cell vertical, on the top area).
    // Split: graph canvas on top, detail panel on bottom.
    // During a build flow the bottom panel expands to 50% for the recipe browser.
    let build_active = ui.build.is_some();
    let (graph_pct, panel_pct) = if build_active { (50, 50) } else { (70, 30) };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(graph_pct),
            Constraint::Percentage(panel_pct),
        ])
        .split(inner);

    // Refresh graph data before rendering.
    // Use actual chunks[0] dimensions so layout bounds match exactly what the renderer uses.
    let canvas_width = chunks[0].width as f64 * 2.0; // Braille 2x horizontal
    let canvas_height = chunks[0].height as f64 * 4.0; // Braille 4x vertical
    crate::loom::graph::refresh_graph(ui, loom_state, canvas_width, canvas_height);

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
            crate::loom::BuildStep::SelectRecipe { cursor } => {
                // Two-column layout: left = scrollable recipe list, right = detail sidebar.
                let cols = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                    .split(inner);
                let left_area = cols[0];
                let right_area = cols[1];

                let all_recipes = crate::loom::recipes::all_recipes();
                let display_rows = build_recipe_display_rows(&build.available_recipes);
                let cursor_row = cursor_to_display_row(&display_rows, *cursor);

                // ── Left column: tier tabs (ratatui Tabs widget) + scrollable recipe list ──
                let left_rows = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(1), Constraint::Min(0)])
                    .split(left_area);

                // Render tier tabs using ratatui Tabs widget.
                let tiers = crate::loom::unlocked_tiers(loom);
                let tab_titles: Vec<String> =
                    tiers.iter().map(|t| format!(" Tier {} ", t)).collect();
                let selected_tab = tiers.iter().position(|&t| t == build.tier).unwrap_or(0);
                let tabs_widget = ratatui::widgets::Tabs::new(tab_titles)
                    .select(selected_tab)
                    .style(Style::default().fg(Color::Rgb(80, 70, 100)))
                    .highlight_style(
                        Style::default()
                            .fg(LOOM_BORDER_COLOR)
                            .add_modifier(Modifier::BOLD),
                    )
                    .divider(" ");
                frame.render_widget(tabs_widget, left_rows[0]);

                // Recipe list below tabs.
                let avail_h = left_rows[1].height as usize;
                let list_height = avail_h.saturating_sub(1); // hint line
                let total_rows = display_rows.len();

                let half = list_height / 2;
                let scroll_start = if cursor_row >= half {
                    (cursor_row - half).min(total_rows.saturating_sub(list_height))
                } else {
                    0
                };
                let scroll_end = (scroll_start + list_height).min(total_rows);

                let mut left_lines: Vec<Line> = Vec::new();

                // Scroll-up indicator.
                if scroll_start > 0 {
                    left_lines.push(Line::from(Span::styled(
                        " \u{25b2} more above",
                        Style::default().fg(Color::Rgb(80, 70, 100)),
                    )));
                }

                // Recipe list rows.
                for row in &display_rows[scroll_start..scroll_end] {
                    match row {
                        RecipeRowKind::Recipe {
                            recipe_list_idx,
                            global_idx,
                        } => {
                            let r = &all_recipes[*global_idx];
                            let is_selected = *recipe_list_idx == *cursor;

                            let sources_a =
                                crate::loom::eligible_sources_for_tier(loom, build.tier, r.input_a);
                            let sources_b =
                                crate::loom::eligible_sources_for_tier(loom, build.tier, r.input_b);
                            let missing_a = sources_a.is_empty();
                            let missing_b = sources_b.is_empty();
                            let buildable = !missing_a && !missing_b;

                            let prefix = if is_selected { " \u{25b6} " } else { "   " };
                            let style = if !buildable {
                                Style::default().fg(Color::Rgb(80, 50, 60))
                            } else if is_selected {
                                Style::default()
                                    .fg(Color::White)
                                    .add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(Color::Rgb(120, 100, 150))
                            };

                            let mut spans = vec![
                                Span::styled(
                                    format!(
                                        "{}{} {}",
                                        prefix,
                                        resource_emoji(&r.output),
                                        resource_name(&r.output),
                                    ),
                                    style,
                                ),
                                Span::styled(
                                    format!(
                                        "    \u{2190} {}+{}",
                                        resource_emoji(&r.input_a),
                                        resource_emoji(&r.input_b),
                                    ),
                                    Style::default().fg(Color::Rgb(80, 70, 110)),
                                ),
                            ];

                            if missing_a && missing_b {
                                spans.push(Span::styled(
                                    "  \u{2717} no sources",
                                    Style::default().fg(Color::Rgb(120, 50, 50)),
                                ));
                            } else if missing_a {
                                spans.push(Span::styled(
                                    format!("  \u{2717} no {}", resource_name(&r.input_a)),
                                    Style::default().fg(Color::Rgb(120, 50, 50)),
                                ));
                            } else if missing_b {
                                spans.push(Span::styled(
                                    format!("  \u{2717} no {}", resource_name(&r.input_b)),
                                    Style::default().fg(Color::Rgb(120, 50, 50)),
                                ));
                            }

                            left_lines.push(Line::from(spans));
                        }
                    }
                }

                // Scroll-down indicator.
                if scroll_end < total_rows {
                    left_lines.push(Line::from(Span::styled(
                        " \u{25bc} more below",
                        Style::default().fg(Color::Rgb(80, 70, 100)),
                    )));
                }

                frame.render_widget(
                    Paragraph::new(left_lines).style(Style::default().bg(LOOM_BG)),
                    left_rows[1],
                );

                // ── Right column: detail sidebar for selected recipe ──
                let mut right_lines: Vec<Line> = Vec::new();

                if let Some(&global_idx) = build.available_recipes.get(*cursor) {
                    let r = &all_recipes[global_idx];
                    let sources_a =
                        crate::loom::eligible_sources_for_tier(loom, build.tier, r.input_a);
                    let sources_b =
                        crate::loom::eligible_sources_for_tier(loom, build.tier, r.input_b);

                    // Recipe summary: output first, then inputs.
                    right_lines.push(Line::from(Span::styled(
                        format!(
                            " {} {}",
                            resource_emoji(&r.output),
                            resource_name(&r.output),
                        ),
                        Style::default().fg(Color::Rgb(180, 140, 220)),
                    )));
                    right_lines.push(Line::from(Span::styled(
                        format!(
                            " \u{2190} {} {} + {} {}",
                            resource_emoji(&r.input_a),
                            resource_name(&r.input_a),
                            resource_emoji(&r.input_b),
                            resource_name(&r.input_b),
                        ),
                        Style::default().fg(Color::Rgb(140, 110, 170)),
                    )));

                    // Tier.
                    right_lines.push(Line::from(Span::styled(
                        format!(" Tier {}", r.tier),
                        Style::default().fg(Color::Rgb(140, 110, 170)),
                    )));

                    // Output yield.
                    right_lines.push(Line::from(Span::styled(
                        format!(" Yield: {:.1}x", r.amount),
                        Style::default().fg(Color::Rgb(140, 110, 170)),
                    )));

                    // Source availability.
                    right_lines.push(Line::from(""));

                    let count_a = sources_a.len();
                    let (a_suffix, a_color) = if count_a > 0 {
                        (
                            format!("{} available \u{2713}", count_a),
                            Color::Rgb(100, 200, 120),
                        )
                    } else {
                        ("none \u{2717}".to_string(), Color::Rgb(200, 80, 80))
                    };
                    right_lines.push(Line::from(Span::styled(
                        format!(
                            " Source A ({} {}): {}",
                            resource_emoji(&r.input_a),
                            resource_name(&r.input_a),
                            a_suffix,
                        ),
                        Style::default().fg(a_color),
                    )));

                    let count_b = sources_b.len();
                    let (b_suffix, b_color) = if count_b > 0 {
                        (
                            format!("{} available \u{2713}", count_b),
                            Color::Rgb(100, 200, 120),
                        )
                    } else {
                        ("none \u{2717}".to_string(), Color::Rgb(200, 80, 80))
                    };
                    right_lines.push(Line::from(Span::styled(
                        format!(
                            " Source B ({} {}): {}",
                            resource_emoji(&r.input_b),
                            resource_name(&r.input_b),
                            b_suffix,
                        ),
                        Style::default().fg(b_color),
                    )));
                }

                frame.render_widget(
                    Paragraph::new(right_lines).style(Style::default().bg(LOOM_BG)),
                    right_area,
                );

                // SelectRecipe renders directly into two columns; skip the
                // shared single-Paragraph render below.
                return;
            }
            crate::loom::BuildStep::SelectSourcesA { cursor, toggle } => {
                let r = &recipes[build.recipe_index];
                let mut lines = vec![
                    Line::from(Span::styled(
                        format!(
                            " {} {} + {} {} \u{2192} {} {}",
                            resource_emoji(&r.input_a),
                            resource_name(&r.input_a),
                            resource_emoji(&r.input_b),
                            resource_name(&r.input_b),
                            resource_emoji(&r.output),
                            resource_name(&r.output),
                        ),
                        Style::default().fg(Color::Rgb(180, 140, 220)),
                    )),
                    Line::from(Span::styled(
                        format!(
                            " Select sources for {} {}:",
                            resource_emoji(&r.input_a),
                            resource_name(&r.input_a),
                        ),
                        Style::default().fg(Color::Rgb(140, 110, 170)),
                    )),
                ];
                for (i, src) in build.eligible_sources_a.iter().enumerate() {
                    let marker = if i == *cursor { "\u{25b6}" } else { " " };
                    let check = if toggle[i] { "[\u{2713}]" } else { "[ ]" };
                    let name = source_name(src, loom);
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
                if build.eligible_sources_a.is_empty() {
                    lines.push(Line::from(Span::styled(
                        " (no eligible sources)",
                        Style::default().fg(Color::Rgb(160, 80, 80)),
                    )));
                }
                lines
            }
            crate::loom::BuildStep::SelectSourcesB { cursor, toggle } => {
                let r = &recipes[build.recipe_index];
                let mut lines = vec![
                    Line::from(Span::styled(
                        format!(
                            " {} {} + {} {} \u{2192} {} {}",
                            resource_emoji(&r.input_a),
                            resource_name(&r.input_a),
                            resource_emoji(&r.input_b),
                            resource_name(&r.input_b),
                            resource_emoji(&r.output),
                            resource_name(&r.output),
                        ),
                        Style::default().fg(Color::Rgb(180, 140, 220)),
                    )),
                    Line::from(Span::styled(
                        format!(
                            " Select sources for {} {}:",
                            resource_emoji(&r.input_b),
                            resource_name(&r.input_b),
                        ),
                        Style::default().fg(Color::Rgb(140, 110, 170)),
                    )),
                ];
                // Show confirmed A sources dimmed.
                if !build.selected_sources_a.is_empty() {
                    lines.push(Line::from(Span::styled(
                        format!(
                            " Source A ({}): {}",
                            resource_name(&r.input_a),
                            build
                                .selected_sources_a
                                .iter()
                                .map(|s| source_name(s, loom))
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                        Style::default().fg(Color::Rgb(80, 70, 100)),
                    )));
                }
                for (i, src) in build.eligible_sources_b.iter().enumerate() {
                    let marker = if i == *cursor { "\u{25b6}" } else { " " };
                    let check = if toggle[i] { "[\u{2713}]" } else { "[ ]" };
                    let name = source_name(src, loom);
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
                if build.eligible_sources_b.is_empty() {
                    lines.push(Line::from(Span::styled(
                        " (no eligible sources)",
                        Style::default().fg(Color::Rgb(160, 80, 80)),
                    )));
                }
                lines
            }
            crate::loom::BuildStep::Confirm => {
                let r = &recipes[build.recipe_index];
                let cost = crate::loom::shuttle_build_cost_public(build.tier);
                vec![
                    Line::from(Span::styled(
                        format!(
                            " {} {} + {} {} \u{2192} {} {}",
                            resource_emoji(&r.input_a),
                            resource_name(&r.input_a),
                            resource_emoji(&r.input_b),
                            resource_name(&r.input_b),
                            resource_emoji(&r.output),
                            resource_name(&r.output),
                        ),
                        Style::default().fg(Color::Rgb(180, 140, 220)),
                    )),
                    Line::from(Span::styled(
                        format!(
                            " Step 4/4: Confirm  cost: {:.0} {} {}",
                            cost,
                            resource_emoji(&r.input_a),
                            resource_name(&r.input_a),
                        ),
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
            render_bottom_panel_pattern(frame, inner, loom, ui, *pat_idx);
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

    if !node.unlocked {
        let mut lines: Vec<Line> = Vec::new();
        let emoji = node_emoji(node_id);
        lines.push(Line::from(Span::styled(
            format!(" {} {} [Locked]", emoji, node_id.name()),
            Style::default().fg(Color::Rgb(80, 60, 110)),
        )));
        if node.unlock_progress > 0.0 {
            lines.push(Line::from(Span::styled(
                format!(" {:.1}/2.0h unlocking", node.unlock_progress),
                Style::default().fg(Color::Rgb(100, 80, 160)),
            )));
        }
        frame.render_widget(
            Paragraph::new(lines).style(Style::default().bg(LOOM_BG)),
            area,
        );
        return;
    }

    // ── P4 Layout: header + gauge (full width), then two-column body ──

    // Split vertically: header (1 line) + gauge (1 line) + blank (1 line) + body (rest).
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Length(1), // gauge
            Constraint::Length(1), // spacer
            Constraint::Min(0),    // body
        ])
        .split(area);

    // Header: emoji + name + level + rate.
    let emoji = node_emoji(node_id);
    let rate = logic::node_effective_rate(loom, node);
    let header = Line::from(Span::styled(
        format!(
            " {} {} \u{2022} Level {} \u{2022} {:.0}/hr",
            emoji,
            node_id.name(),
            node.level,
            rate
        ),
        Style::default().fg(node_color(node_id)),
    ));
    frame.render_widget(
        Paragraph::new(vec![header]).style(Style::default().bg(LOOM_BG)),
        rows[0],
    );

    // Full-width buffer gauge.
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
    frame.render_widget(gauge, rows[1]);

    // ── Two-column body: consumers (left) | upgrade (right) ──
    let body_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(rows[3]);

    // Left column: consumers list.
    let node_ref = crate::loom::types::LoomNodeRef::Extractor(node.id);
    let mut left_lines: Vec<Line> = Vec::new();
    left_lines.push(Line::from(Span::styled(
        " Consumers:",
        Style::default().fg(Color::Rgb(140, 120, 180)),
    )));
    let mut consumer_count = 0;
    for (i, shuttle) in loom.persistent.shuttles.iter().enumerate() {
        if shuttle.under_construction {
            continue;
        }
        if shuttle.sources_a.contains(&node_ref) || shuttle.sources_b.contains(&node_ref) {
            let out_emoji = resource_emoji(&shuttle.output);
            let out_name = resource_name(&shuttle.output);
            left_lines.push(Line::from(Span::styled(
                format!("   S{} \u{2192} {} {}", i, out_emoji, out_name),
                Style::default().fg(Color::Rgb(120, 100, 160)),
            )));
            consumer_count += 1;
        }
    }
    if consumer_count == 0 {
        left_lines.push(Line::from(Span::styled(
            "   (none)",
            Style::default().fg(Color::Rgb(80, 60, 100)),
        )));
    }
    left_lines.truncate(body_cols[0].height as usize);
    frame.render_widget(
        Paragraph::new(left_lines).style(Style::default().bg(LOOM_BG)),
        body_cols[0],
    );

    // Right column: upgrade info.
    let dim = Color::Rgb(120, 100, 160);
    let bright = Color::Rgb(180, 150, 220);
    let mut upgrade_lines: Vec<Line> = Vec::new();

    if node.upgrading {
        let total = logic::node_upgrade_duration(node.level);
        let elapsed = total - node.upgrade_remaining_secs;
        let progress = if total > 0.0 { elapsed / total } else { 1.0 };
        let remaining = crate::ui::loom_graph::format_duration(node.upgrade_remaining_secs);
        upgrade_lines.push(Line::from(Span::styled(
            format!(" \u{23f3} Upgrading to Level {}...", node.level + 1),
            Style::default().fg(Color::Rgb(220, 180, 60)),
        )));
        upgrade_lines.push(Line::from(Span::styled(
            format!(" {} remaining ({:.0}%)", remaining, progress * 100.0),
            Style::default().fg(Color::Rgb(180, 160, 80)),
        )));
        upgrade_lines.push(Line::from(Span::styled(
            " No production during upgrade",
            Style::default().fg(Color::Rgb(160, 80, 80)),
        )));
    } else if node.level < logic::MAX_NODE_LEVEL {
        let drain = node.buffer_capacity * 0.5;
        let can_afford = node.buffer >= drain;
        let duration = logic::node_upgrade_duration(node.level);
        let dur_str = crate::ui::loom_graph::format_duration(duration);
        let current_rate = logic::node_effective_rate(loom, node);
        let next_rate = node.base_rate * logic::node_level_multiplier(node.level + 1);
        let next_cap = node.base_rate * logic::node_level_multiplier(node.level + 1) * 10.0;

        upgrade_lines.push(Line::from(Span::styled(
            format!(
                " \u{2500}\u{2500} Upgrade to Level {} \u{2500}\u{2500}",
                node.level + 1
            ),
            Style::default().fg(bright),
        )));
        upgrade_lines.push(Line::from(Span::styled(
            format!(
                " Cost:       {:.0} {} (50% of buffer)",
                drain,
                resource_emoji(&logic::node_native_resource(node_id))
            ),
            Style::default().fg(if can_afford {
                dim
            } else {
                Color::Rgb(160, 60, 60)
            }),
        )));
        upgrade_lines.push(Line::from(Span::styled(
            format!(" Build time: {}", dur_str),
            Style::default().fg(dim),
        )));
        upgrade_lines.push(Line::from(Span::styled(
            format!(
                " Production: {:.0} \u{2192} {:.0}/hr",
                current_rate, next_rate
            ),
            Style::default().fg(dim),
        )));
        upgrade_lines.push(Line::from(Span::styled(
            format!(
                " Buffer:     {:.0} \u{2192} {:.0}",
                node.buffer_capacity, next_cap
            ),
            Style::default().fg(dim),
        )));
        upgrade_lines.push(Line::from(""));
        if !can_afford {
            upgrade_lines.push(Line::from(Span::styled(
                format!(" Need {:.0} in buffer (have {:.0})", drain, node.buffer),
                Style::default().fg(Color::Rgb(160, 60, 60)),
            )));
        }
    } else {
        upgrade_lines.push(Line::from(Span::styled(
            " \u{2713} Max Level",
            Style::default().fg(Color::Rgb(100, 200, 120)),
        )));
    };
    upgrade_lines.truncate(body_cols[1].height as usize);
    frame.render_widget(
        Paragraph::new(upgrade_lines).style(Style::default().bg(LOOM_BG)),
        body_cols[1],
    );
}

/// Render bottom panel content for a Shuttle node.
fn render_bottom_panel_shuttle(
    frame: &mut Frame,
    area: Rect,
    loom: &LoomState,
    shuttle_idx: usize,
) {
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
        let total = crate::loom::logic::shuttle_construction_secs(shuttle.tier);
        let elapsed = total - shuttle.construction_secs_remaining;
        let progress = if total > 0.0 {
            (elapsed / total).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let remaining = crate::ui::loom_graph::format_duration(shuttle.construction_secs_remaining);

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(area);

        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!(
                    " {} {} \u{2022} Tier {} \u{2022} Under Construction",
                    resource_emoji(&shuttle.output),
                    resource_name(&shuttle.output),
                    shuttle.tier
                ),
                Style::default().fg(Color::Rgb(180, 140, 220)),
            )))
            .style(Style::default().bg(LOOM_BG)),
            rows[0],
        );
        let gauge = Gauge::default()
            .ratio(progress)
            .label(format!("{} remaining", remaining))
            .gauge_style(
                Style::default()
                    .fg(Color::Rgb(180, 140, 220))
                    .bg(Color::Rgb(30, 20, 40)),
            );
        frame.render_widget(gauge, rows[1]);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!(
                    " {} {} + {} {}",
                    resource_emoji(&shuttle.input_a),
                    resource_name(&shuttle.input_a),
                    resource_emoji(&shuttle.input_b),
                    resource_name(&shuttle.input_b)
                ),
                Style::default().fg(Color::Rgb(120, 100, 160)),
            )))
            .style(Style::default().bg(LOOM_BG)),
            rows[2],
        );
        return;
    }

    // ── Layout B: output headline, recipe, gauge, output rate, demolish ──
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // output name + tier
            Constraint::Length(1), // recipe line
            Constraint::Length(1), // buffer gauge
            Constraint::Length(1), // spacer
            Constraint::Min(0),    // output rate + demolish
        ])
        .split(area);

    // Line 1: output name + tier
    let eo = resource_emoji(&shuttle.output);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!(
                " {} {} \u{2022} Tier {}",
                eo,
                resource_name(&shuttle.output),
                shuttle.tier
            ),
            Style::default().fg(Color::Rgb(180, 140, 220)),
        )))
        .style(Style::default().bg(LOOM_BG)),
        rows[0],
    );

    // Line 2: recipe (inputs → output)
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!(
                " {} {} + {} {}",
                resource_emoji(&shuttle.input_a),
                resource_name(&shuttle.input_a),
                resource_emoji(&shuttle.input_b),
                resource_name(&shuttle.input_b)
            ),
            Style::default().fg(Color::Rgb(120, 100, 160)),
        )))
        .style(Style::default().bg(LOOM_BG)),
        rows[1],
    );

    // Line 3: full-width buffer gauge
    let fill = if shuttle.buffer_capacity > 0.0 {
        (shuttle.buffer / shuttle.buffer_capacity).min(1.0)
    } else {
        0.0
    };
    let bar_color = if fill >= 0.90 {
        Color::Rgb(220, 180, 60)
    } else {
        Color::Rgb(60, 200, 100)
    };
    let gauge = Gauge::default()
        .ratio(fill)
        .label(format!(
            "{:.0}/{:.0}",
            shuttle.buffer, shuttle.buffer_capacity
        ))
        .gauge_style(Style::default().fg(bar_color).bg(Color::Rgb(30, 20, 40)));
    frame.render_widget(gauge, rows[2]);

    // Line 5: output rate + demolish (right-aligned)
    let output_rate = shuttle.output_rate_tracker.rate_per_hour();
    let mut bottom_lines: Vec<Line> = vec![Line::from(Span::styled(
        format!(" Output: {:.0}/hr", output_rate),
        Style::default().fg(Color::Rgb(100, 200, 120)),
    ))];
    bottom_lines.truncate(rows[4].height as usize);
    frame.render_widget(
        Paragraph::new(bottom_lines).style(Style::default().bg(LOOM_BG)),
        rows[4],
    );
}

/// Render bottom panel content for a PatternSink node.
fn render_bottom_panel_pattern(
    frame: &mut Frame,
    area: Rect,
    loom: &LoomState,
    ui: &LoomUiState,
    pat_idx: usize,
) {
    let pattern = match loom.persistent.patterns.get(pat_idx) {
        Some(p) => p,
        None => return,
    };

    let title_color = if pattern.completed {
        Color::Rgb(100, 200, 120)
    } else {
        Color::Rgb(180, 140, 220)
    };

    // Overall progress — since all requirements advance simultaneously,
    // use the minimum progress across all requirements as the single percentage.
    let overall_progress = if pattern.completed {
        1.0
    } else {
        pattern
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
            .fold(f64::INFINITY, f64::min)
            .min(1.0)
    };
    let overall_pct = (overall_progress * 100.0).round() as u32;

    // Remaining time (max across incomplete requirements, since they advance together).
    let remaining_secs: f64 = pattern
        .requirements
        .iter()
        .filter(|r| !r.completed)
        .map(|r| (r.sustain_duration_secs - r.sustained_secs).max(0.0))
        .fold(0.0_f64, f64::max);

    let progress_text = if pattern.completed {
        "100% \u{2713}".to_string()
    } else {
        format!(
            "{}% ({} left)",
            overall_pct,
            crate::ui::loom_graph::format_duration(remaining_secs)
        )
    };

    // Chapter header line.
    let chapter = crate::loom::discovery::pattern_chapter(pattern.index);
    let mut lines: Vec<Line> = Vec::new();
    if !chapter.is_empty() {
        let chapter_text = format!(" \u{2500}\u{2500} {} ", chapter);
        let pad_len = area.width as usize - chapter_text.len().min(area.width as usize);
        let padded = format!("{}{}", chapter_text, "\u{2500}".repeat(pad_len));
        lines.push(Line::from(Span::styled(
            padded,
            Style::default().fg(Color::Rgb(160, 130, 200)),
        )));
    }

    // Diamond progress trail: ◇◇◇◇◆◇◇◇ showing position in sequence.
    let total_patterns = loom.persistent.patterns.len();
    if total_patterns > 0 {
        let mut diamond_spans: Vec<Span> = vec![Span::raw(" ")];
        for i in 0..total_patterns {
            let ch = if loom.persistent.patterns[i].completed {
                "\u{25c6}" // ◆ filled (completed)
            } else if i == pat_idx {
                // Slow blink: alternate filled/hollow every ~0.5s (5 frames at 10fps).
                if (ui.throbber_frame / 5).is_multiple_of(2) {
                    "\u{25c6}" // ◆ filled
                } else {
                    "\u{25c7}" // ◇ hollow
                }
            } else {
                "\u{25c7}" // ◇ hollow (future)
            };
            let color = if i == pat_idx {
                Color::Rgb(255, 200, 60) // gold for current
            } else if loom.persistent.patterns[i].completed {
                Color::Rgb(100, 200, 120) // green for completed
            } else {
                Color::Rgb(60, 50, 80) // dim for future
            };
            diamond_spans.push(Span::styled(ch, Style::default().fg(color)));
        }
        diamond_spans.push(Span::styled(
            format!("  {}/{}", pat_idx + 1, total_patterns),
            Style::default().fg(Color::Rgb(80, 70, 100)),
        ));
        lines.push(Line::from(diamond_spans));
    }

    // Pattern name.
    lines.push(Line::from(Span::styled(
        format!(" Pattern #{}: {}", pat_idx + 1, pattern.name),
        Style::default().fg(title_color),
    )));

    // We'll render a Gauge widget separately below the text lines.
    // Store the progress info for later; insert a blank line as placeholder.
    let gauge_label = progress_text;
    let gauge_ratio = overall_progress.clamp(0.0, 1.0);
    lines.push(Line::from("")); // placeholder for gauge row

    // Flavor text (italic, muted purple, quoted).
    if !pattern.flavor.is_empty() {
        lines.push(Line::from(Span::styled(
            format!(" \u{201c}{}\u{201d}", pattern.flavor),
            Style::default()
                .fg(Color::Rgb(130, 110, 160))
                .add_modifier(Modifier::ITALIC),
        )));
        lines.push(Line::from(""))
    }

    // Check if all requirements are currently being met (for status indicator).
    let rates: std::collections::HashMap<_, _> = loom
        .rate_trackers
        .iter()
        .map(|(r, t)| (*r, t.rate_per_hour()))
        .collect();

    // Per-requirement rows: show resource, required rate, and whether currently met.
    for req in &pattern.requirements {
        let re = resource_emoji(&req.resource);
        let rn = resource_name(&req.resource);
        let current_rate = rates.get(&req.resource).copied().unwrap_or(0.0);
        let is_met = current_rate >= req.required_rate;

        let status_icon = if req.completed {
            ("\u{2713}", Color::Rgb(100, 200, 120)) // ✓ green
        } else if is_met {
            ("\u{25cf}", Color::Rgb(100, 200, 120)) // ● green (currently sustaining)
        } else {
            ("\u{25cb}", Color::Rgb(180, 60, 60)) // ○ red (not met)
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!(" {} ", status_icon.0),
                Style::default().fg(status_icon.1),
            ),
            Span::styled(
                format!("{}{} ", re, rn),
                Style::default().fg(Color::Rgb(140, 120, 180)),
            ),
            Span::styled(
                format!("{:.0}/hr needed", req.required_rate),
                Style::default().fg(Color::Rgb(100, 80, 140)),
            ),
            Span::styled(
                format!("  ({:.0}/hr now)", current_rate),
                Style::default().fg(if is_met {
                    Color::Rgb(80, 160, 100)
                } else {
                    Color::Rgb(150, 60, 60)
                }),
            ),
        ]));
    }

    // Find which line is the gauge placeholder (the blank line after the pattern name).
    // It's the line right after "Pattern #N: Name".
    let gauge_line_idx = lines
        .iter()
        .position(|l| l.spans.is_empty() || (l.spans.len() == 1 && l.spans[0].content.is_empty()));

    lines.truncate(area.height as usize);
    frame.render_widget(
        Paragraph::new(lines).style(Style::default().bg(LOOM_BG)),
        area,
    );

    // Overlay the Gauge widget on the placeholder row.
    if let Some(row) = gauge_line_idx {
        if (row as u16) < area.height {
            let gauge_area = Rect::new(
                area.x + 1,
                area.y + row as u16,
                area.width.saturating_sub(2),
                1,
            );
            let gauge = Gauge::default()
                .ratio(gauge_ratio)
                .label(gauge_label)
                .gauge_style(Style::default().fg(title_color).bg(Color::Rgb(30, 20, 40)));
            frame.render_widget(gauge, gauge_area);
        }
    }
}

// ── Build Shuttle Overlay ────────────────────────────────────────────────────

/// Which kind of row appears in the grouped recipe display list.
enum RecipeRowKind {
    /// A selectable recipe row. `recipe_list_idx` indexes into
    /// `build.available_recipes`; `global_idx` is the index into `all_recipes()`.
    Recipe {
        recipe_list_idx: usize,
        #[allow(dead_code)]
        global_idx: usize,
    },
}

/// Build the grouped display rows from sorted `available_recipes`.
/// Inserts a header whenever the output resource changes.
fn build_recipe_display_rows(available: &[usize]) -> Vec<RecipeRowKind> {
    let mut rows = Vec::new();
    for (list_idx, &global_idx) in available.iter().enumerate() {
        rows.push(RecipeRowKind::Recipe {
            recipe_list_idx: list_idx,
            global_idx,
        });
    }
    rows
}

/// Map a recipe cursor (index into `available_recipes`) to a display row index.
/// With headers removed, this is a direct 1:1 mapping.
fn cursor_to_display_row(rows: &[RecipeRowKind], cursor: usize) -> usize {
    for (i, row) in rows.iter().enumerate() {
        let RecipeRowKind::Recipe {
            recipe_list_idx, ..
        } = row;
        if *recipe_list_idx == cursor {
            return i;
        }
    }
    0
}

// ── Navigation hints ──────────────────────────────────────────────────────────

fn render_nav_hints(frame: &mut Frame, area: Rect, ui: &LoomUiState) {
    if area.height < 3 {
        return;
    }

    let hints = if let Some(build) = &ui.build {
        match &build.step {
            crate::loom::BuildStep::SelectRecipe { .. } => {
                " [\u{2191}\u{2193}] Select  [\u{2190}\u{2192}] Tier  [Enter] Next  [Esc] Cancel "
            }
            crate::loom::BuildStep::SelectSourcesA { .. }
            | crate::loom::BuildStep::SelectSourcesB { .. } => {
                " [\u{2191}\u{2193}] Navigate  [Space] Toggle  [Enter] Confirm  [Esc] Back "
            }
            crate::loom::BuildStep::Confirm => " [Enter] Build  [Esc] Cancel ",
            crate::loom::BuildStep::Blocked { .. } => " Press any key to dismiss ",
        }
    } else if ui.demolish_pending {
        " [D] Confirm Demolish  [Any] Cancel "
    } else {
        " [\u{2191}\u{2193}\u{2190}\u{2192}] Navigate  [U] Upgrade  [B] Build  [D] Demolish  [Esc] Close "
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

fn source_name(src: &crate::loom::types::LoomNodeRef, loom: &LoomState) -> String {
    match src {
        crate::loom::types::LoomNodeRef::Extractor(id) => {
            let node = &loom.persistent.nodes[id.index()];
            format!(
                "{} {} L{}",
                resource_emoji(&crate::loom::logic::node_native_resource(*id)),
                id.name(),
                node.level
            )
        }
        crate::loom::types::LoomNodeRef::Shuttle(idx) => {
            if let Some(s) = loom.persistent.shuttles.get(*idx) {
                format!(
                    "S{} {} {} L{}",
                    idx,
                    resource_emoji(&s.output),
                    resource_name(&s.output),
                    s.level
                )
            } else {
                format!("S{}", idx)
            }
        }
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
