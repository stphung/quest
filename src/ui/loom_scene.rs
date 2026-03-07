//! The Loom of Worlds overlay — main UI renderer.
#![allow(dead_code)]
//!
//! Dispatches to different view renderers based on `LoomUiState::view`:
//!   - ArchetypeSelection: choose your archetype at unlock
//!   - FlowView:           pipeline diagram placeholder
//!   - ListDetail:         node list + detail panel placeholder
//!   - Codex:              recipe codex placeholder

use crate::loom::patterns::{active_pattern_requirement_status, all_patterns_complete};
use crate::loom::types::{LoomArchetype, LoomState, LoomUiState, LoomView};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Border color for the Loom overlay.
const LOOM_BORDER_COLOR: Color = Color::Rgb(180, 120, 220);

// ── Main entry point ──────────────────────────────────────────────────────────

/// Render the Loom of Worlds overlay.
pub fn render_loom_overlay(
    frame: &mut Frame,
    area: Rect,
    loom_state: &LoomState,
    ui: &LoomUiState,
) {
    frame.render_widget(Clear, area);

    let view_name = match ui.view {
        LoomView::ArchetypeSelection => "Archetype Selection",
        LoomView::FlowView => "Flow View",
        LoomView::ListDetail => "Nodes",
        LoomView::Codex => "Recipe Codex",
    };

    let title = format!(" LOOM OF WORLDS \u{2014} {} ", view_name);
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

    match ui.view {
        LoomView::ArchetypeSelection => {
            render_archetype_selection(frame, inner, loom_state, ui);
        }
        LoomView::FlowView => {
            render_flow_view(frame, inner, loom_state);
        }
        LoomView::ListDetail => {
            render_list_detail(frame, inner, loom_state, ui);
        }
        LoomView::Codex => {
            render_codex(frame, inner, loom_state, ui);
        }
    }

    render_nav_hints(frame, area, ui);
}

// ── View renderers ────────────────────────────────────────────────────────────

fn render_archetype_selection(
    frame: &mut Frame,
    area: Rect,
    loom_state: &LoomState,
    ui: &LoomUiState,
) {
    let archetypes = [
        (
            LoomArchetype::BurnBright,
            "Burn Bright",
            "Ember Spindle + Void Condenser",
            "High throughput, volatile output",
        ),
        (
            LoomArchetype::ReachWide,
            "Reach Wide",
            "Reflection Lens + Memory Archive",
            "Broad coverage, pattern synergies",
        ),
        (
            LoomArchetype::RunDeep,
            "Run Deep",
            "Silence Well + Resonance Forge",
            "Efficient conversion, deep reactions",
        ),
    ];

    let already_chosen = loom_state.persistent.archetype;

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Choose your archetype to unlock the Loom:",
            Style::default().fg(Color::Rgb(200, 170, 240)),
        )),
        Line::from(""),
    ];

    for (i, (archetype, name, nodes, desc)) in archetypes.iter().enumerate() {
        let is_selected = ui.selected_archetype == i;
        let is_chosen = already_chosen == Some(*archetype);

        let prefix = if is_chosen {
            "\u{2713} "
        } else if is_selected {
            "\u{25b6} "
        } else {
            "  "
        };

        let name_color = if is_chosen {
            Color::Rgb(255, 215, 0)
        } else if is_selected {
            Color::White
        } else {
            Color::Rgb(140, 110, 180)
        };

        lines.push(Line::from(vec![
            Span::styled(prefix, Style::default().fg(name_color)),
            Span::styled(
                format!("[{}] {}", i + 1, name),
                Style::default().fg(name_color),
            ),
        ]));
        lines.push(Line::from(Span::styled(
            format!("     Nodes: {}", nodes),
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            format!("     {}", desc),
            Style::default().fg(Color::Rgb(100, 80, 140)),
        )));
        lines.push(Line::from(""));
    }

    if already_chosen.is_none() {
        lines.push(Line::from(Span::styled(
            "[Enter] Confirm selection",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "Archetype chosen. Use [Tab] to explore the Loom.",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let para = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .style(Style::default().bg(Color::Rgb(10, 5, 18)));
    frame.render_widget(para, area);
}

fn render_flow_view(frame: &mut Frame, area: Rect, loom_state: &LoomState) {
    // Reserve 4 rows at the bottom for the pattern bar when patterns are initialized.
    let has_patterns = !loom_state.persistent.patterns.is_empty();
    let pattern_bar_height = if has_patterns { 4u16 } else { 0u16 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(pattern_bar_height)])
        .split(area);

    let diagram_area = chunks[0];
    let pattern_area = chunks[1];

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            " Nodes \u{2014} Buffer Status",
            Style::default().fg(Color::Rgb(180, 120, 220)),
        )),
        Line::from(""),
    ];

    for node in &loom_state.persistent.nodes {
        lines.push(build_node_buffer_line(node, diagram_area.width));
    }

    let para = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .style(Style::default().bg(Color::Rgb(10, 5, 18)));
    frame.render_widget(para, diagram_area);

    if has_patterns {
        render_pattern_bar(frame, pattern_area, loom_state);
    }
}

fn render_list_detail(frame: &mut Frame, area: Rect, loom_state: &LoomState, ui: &LoomUiState) {
    use crate::loom::types::NodeId;

    let nodes = NodeId::ALL;
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Nodes",
            Style::default().fg(Color::Rgb(180, 120, 220)),
        )),
        Line::from(""),
    ];

    for (i, node_id) in nodes.iter().enumerate() {
        let is_selected = ui.selected_node == i;
        let prefix = if is_selected { "\u{25b6} " } else { "  " };
        let color = if is_selected {
            Color::White
        } else {
            Color::Rgb(120, 80, 160)
        };
        lines.push(Line::from(vec![
            Span::styled(prefix, Style::default().fg(color)),
            Span::styled(node_id.name(), Style::default().fg(color)),
            Span::styled(
                format!(" \u{2014} {:?}", node_id.nature()),
                Style::default().fg(Color::DarkGray),
            ),
        ]));

        // Show outgoing pipes for the selected node.
        if is_selected {
            let outgoing_pipes: Vec<_> = loom_state
                .persistent
                .pipes
                .iter()
                .filter(|p| p.from == *node_id && !p.under_construction)
                .collect();

            if outgoing_pipes.is_empty() {
                lines.push(Line::from(Span::styled(
                    "     no outgoing pipes",
                    Style::default().fg(Color::Rgb(60, 45, 80)),
                )));
            } else {
                for (pipe_i, pipe) in outgoing_pipes.iter().enumerate() {
                    let is_pipe_selected = ui.selected_pipe == pipe_i;
                    let pipe_prefix = if is_pipe_selected {
                        "   \u{25b8} "
                    } else {
                        "     "
                    };
                    let pipe_color = if is_pipe_selected {
                        Color::Rgb(220, 190, 255)
                    } else {
                        Color::Rgb(100, 70, 130)
                    };
                    let ratio_pct = (pipe.split_ratio * 100.0).round() as u32;
                    let bar = build_ratio_bar(pipe.split_ratio, 10);
                    lines.push(Line::from(vec![
                        Span::styled(pipe_prefix, Style::default().fg(pipe_color)),
                        Span::styled(
                            format!("\u{2192} {}", pipe.to.name()),
                            Style::default().fg(pipe_color),
                        ),
                        Span::styled(
                            format!("  [{:>3}%] {}", ratio_pct, bar),
                            Style::default().fg(pipe_color),
                        ),
                        Span::styled(
                            format!("  {:?}", pipe.tier),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]));
                }
                lines.push(Line::from(Span::styled(
                    "   [P] Select pipe  [\u{2190}/\u{2192}] Adjust ratio",
                    Style::default().fg(Color::DarkGray),
                )));
            }
        }
    }

    let para = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .style(Style::default().bg(Color::Rgb(10, 5, 18)));
    frame.render_widget(para, area);
}

/// Build a compact inline ratio bar (e.g. "████░░░░░░" for 40%).
fn build_ratio_bar(ratio: f64, width: usize) -> String {
    let filled = ((ratio.clamp(0.0, 1.0) * width as f64) as usize).min(width);
    let empty = width.saturating_sub(filled);
    format!("{}{}", "\u{2588}".repeat(filled), "\u{2591}".repeat(empty))
}

fn render_codex(frame: &mut Frame, area: Rect, loom_state: &LoomState, ui: &LoomUiState) {
    use crate::loom::recipes::all_recipes;

    let discovered_count = loom_state
        .persistent
        .codex
        .iter()
        .filter(|e| e.discovered)
        .count();
    let total_recipes = all_recipes().len();
    let hint_indices = crate::loom::logic::codex_hint_indices(&loom_state.persistent.codex);
    let hint_count = hint_indices.len();

    let count_label = format!("  {}/{} Discovered", discovered_count, total_recipes);

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Recipe Codex",
                Style::default().fg(Color::Rgb(180, 120, 220)),
            ),
            Span::styled(count_label, Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(""),
    ];

    if discovered_count == 0 && hint_count == 0 {
        lines.push(Line::from(Span::styled(
            "No recipes discovered yet.",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            "Process resources through nodes to reveal reactions.",
            Style::default().fg(Color::Rgb(80, 60, 100)),
        )));
    } else {
        // Discovered recipes with proper nature names and output multiplier.
        for entry in loom_state.persistent.codex.iter().filter(|e| e.discovered) {
            let inputs: Vec<&str> = entry.inputs.iter().map(|r| resource_name(r)).collect();
            let nature = node_nature_name(entry.node_nature);
            lines.push(Line::from(vec![
                Span::styled(
                    format!(" {} @ {}", inputs.join(" + "), nature),
                    Style::default().fg(Color::Rgb(160, 100, 200)),
                ),
                Span::styled(" \u{2192} ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    resource_name(&entry.output),
                    Style::default().fg(Color::Rgb(220, 180, 255)),
                ),
                Span::styled(
                    format!(" (x{:.1})", entry.output_amount),
                    Style::default().fg(Color::Rgb(130, 90, 170)),
                ),
            ]));
        }

        // "???" hints for adjacent undiscovered recipes — show all of them (scrollable).
        if hint_count > 0 {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!(
                    " {} adjacent reaction{} hinted:",
                    hint_count,
                    if hint_count == 1 { "" } else { "s" }
                ),
                Style::default().fg(Color::DarkGray),
            )));
            for _ in 0..hint_count {
                lines.push(Line::from(Span::styled(
                    " ??? + ??? @ ??? \u{2192} ???",
                    Style::default().fg(Color::Rgb(60, 45, 80)),
                )));
            }
        }
    }

    // Clamp scroll so it can't scroll past the last line.
    let total_lines = lines.len();
    let visible_rows = area.height as usize;
    let scroll = ui
        .codex_scroll
        .min(total_lines.saturating_sub(visible_rows));

    let para = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .style(Style::default().bg(Color::Rgb(10, 5, 18)))
        .scroll((scroll as u16, 0));
    frame.render_widget(para, area);
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

// ── Buffer bar helpers ────────────────────────────────────────────────────────

/// Build a single line showing a node's name, lock status, buffer bar, and stall warning.
fn build_node_buffer_line(node: &crate::loom::types::LoomNode, width: u16) -> Line<'static> {
    if !node.unlocked {
        return Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(
                format!("{:<18}", node.id.name()),
                Style::default().fg(Color::Rgb(60, 45, 80)),
            ),
            Span::styled("[locked]", Style::default().fg(Color::Rgb(60, 45, 80))),
        ]);
    }

    let fill_ratio = if node.buffer_capacity > 0.0 {
        (node.buffer / node.buffer_capacity).min(1.0)
    } else {
        0.0
    };

    // Color: green normal, yellow >75%, red >90% or stalled.
    let bar_color = if node.stalled || fill_ratio >= 0.90 {
        Color::Rgb(220, 60, 60)
    } else if fill_ratio >= 0.75 {
        Color::Rgb(220, 180, 60)
    } else {
        Color::Rgb(60, 200, 100)
    };

    // Bar width: total width minus name(18) minus brackets(2) minus spacing(4) minus stock text(12).
    let bar_width = (width as usize)
        .saturating_sub(18 + 2 + 4 + 12)
        .clamp(4, 20);
    let filled = ((fill_ratio * bar_width as f64) as usize).min(bar_width);
    let empty = bar_width - filled;

    let stall_marker = if node.stalled { " \u{26a0}STALL" } else { "" };

    Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            format!("{:<18}", node.id.name()),
            Style::default().fg(Color::Rgb(180, 140, 220)),
        ),
        Span::styled("[", Style::default().fg(Color::DarkGray)),
        Span::styled("\u{2588}".repeat(filled), Style::default().fg(bar_color)),
        Span::styled(
            "\u{2591}".repeat(empty),
            Style::default().fg(Color::Rgb(40, 30, 55)),
        ),
        Span::styled("]", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!(
                " {:>5.1}/{:<5.1}{}",
                node.buffer, node.buffer_capacity, stall_marker
            ),
            Style::default().fg(bar_color),
        ),
    ])
}

// ── Pattern bar ───────────────────────────────────────────────────────────────

/// Render the always-visible pattern progress bar at the bottom of Flow View.
///
/// Shows: pattern name + index, per-requirement checkmarks, sustain progress bar.
/// When all patterns are complete, shows a completion message.
fn render_pattern_bar(frame: &mut Frame, area: Rect, loom_state: &LoomState) {
    if area.height == 0 {
        return;
    }

    let empty_rates = std::collections::HashMap::new();
    let req_status = active_pattern_requirement_status(&loom_state.persistent, &empty_rates);
    let all_done = all_patterns_complete(&loom_state.persistent);

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::Rgb(100, 60, 140)));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    if all_done {
        let line = Line::from(Span::styled(
            " \u{2728} Loom Mended \u{2014} All 18 Patterns Complete",
            Style::default().fg(Color::Rgb(255, 215, 0)),
        ));
        frame.render_widget(
            Paragraph::new(line).style(Style::default().bg(Color::Rgb(10, 5, 18))),
            inner,
        );
        return;
    }

    let Some(pattern) = loom_state
        .persistent
        .patterns
        .get(loom_state.persistent.active_pattern)
    else {
        return;
    };

    let pattern_count = loom_state.persistent.patterns.len();
    let active_idx = loom_state.persistent.active_pattern;

    // Line 1: pattern name and index.
    let pattern_title = format!(
        " PATTERN {}/{}: \"{}\"",
        active_idx + 1,
        pattern_count,
        pattern.name
    );

    // Line 2: per-requirement status chips (check/cross + resource + rate).
    let mut req_spans: Vec<Span> = vec![Span::raw(" ")];
    for (i, req) in pattern.requirements.iter().enumerate() {
        let met = req_status.get(i).copied().unwrap_or(false);
        let check = if met { "\u{2713}" } else { "\u{2717}" };
        let color = if met {
            Color::Rgb(80, 200, 120)
        } else {
            Color::Rgb(200, 80, 80)
        };
        let res_name = resource_name(&req.resource);
        req_spans.push(Span::styled(
            format!("{} {} {:.0}/hr ", check, res_name, req.rate_per_hour),
            Style::default().fg(color),
        ));
        if i + 1 < pattern.requirements.len() {
            req_spans.push(Span::styled(
                "\u{2502} ",
                Style::default().fg(Color::DarkGray),
            ));
        }
    }

    // Line 3: progress bar.
    let progress_line = build_progress_line(
        pattern.sustained_seconds,
        pattern.sustain_seconds,
        inner.width,
    );

    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            pattern_title,
            Style::default().fg(Color::Rgb(200, 160, 240)),
        )),
        Line::from(req_spans),
        progress_line,
    ];
    lines.truncate(inner.height as usize);

    frame.render_widget(
        Paragraph::new(lines).style(Style::default().bg(Color::Rgb(10, 5, 18))),
        inner,
    );
}

/// Build the sustain progress bar line.
fn build_progress_line(sustained: u32, total: u32, width: u16) -> Line<'static> {
    if total == 0 {
        return Line::from("");
    }

    let elapsed_str = format_mmss(sustained);
    let total_str = format_mmss(total);

    let label_prefix = " Weaving: ";
    let time_suffix = format!("  {}/{}", elapsed_str, total_str);
    let bar_width = (width as usize)
        .saturating_sub(label_prefix.len() + time_suffix.len() + 2)
        .min(40);

    let filled = ((sustained as usize * bar_width) / (total as usize)).min(bar_width);
    let empty = bar_width.saturating_sub(filled);

    let bar = format!(
        "{}{}{}{}",
        label_prefix,
        "\u{2588}".repeat(filled),
        "\u{2591}".repeat(empty),
        time_suffix,
    );

    Line::from(Span::styled(
        bar,
        Style::default().fg(Color::Rgb(160, 100, 220)),
    ))
}

fn format_mmss(seconds: u32) -> String {
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

// ── Navigation hints ──────────────────────────────────────────────────────────

fn render_nav_hints(frame: &mut Frame, area: Rect, ui: &LoomUiState) {
    if area.height < 3 {
        return;
    }

    let hints = if ui.view == LoomView::ArchetypeSelection {
        " [Up/Down] Select  [Enter] Confirm  [Esc] Close "
    } else if ui.view == LoomView::ListDetail {
        " [Tab] Switch View  [Up/Down] Node  [P] Pipe  [\u{2190}/\u{2192}] Ratio  [Esc] Close "
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
        Resource::VoidEssence => "VoidEssence",
        Resource::Memory => "Memory",
        Resource::Silence => "Silence",
        Resource::Resonance => "Resonance",
        Resource::ForgedLight => "ForgedLight",
        Resource::EchoGlass => "EchoGlass",
        Resource::StillbornSong => "StillbornSong",
        Resource::CondensedEmber => "CondensedEmber",
        Resource::EmberEcho => "EmberEcho",
        Resource::PurifiedVoid => "PurifiedVoid",
        Resource::WovenReality => "WovenReality",
    }
}
