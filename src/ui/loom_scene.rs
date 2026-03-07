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
    use crate::loom::types::NodeId;
    use crate::loom::{node_effective_rate, node_native_resource, pipe_flow_rate};

    // Reserve 4 rows at the bottom for the pattern bar when patterns are initialized.
    let has_patterns = !loom_state.persistent.patterns.is_empty();
    let pattern_bar_height = if has_patterns { 4u16 } else { 0u16 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(pattern_bar_height)])
        .split(area);

    let diagram_area = chunks[0];
    let pattern_area = chunks[1];

    // Split diagram into left (node diagram) and right (stockpiles) panels.
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(26)])
        .split(diagram_area);

    let diagram_left = h_chunks[0];
    let stockpile_right = h_chunks[1];

    // ── Left: node flow diagram ───────────────────────────────────────────────

    // Archetype pairs: each row shows left-node ──pipe──▶ right-node.
    // Rows: (EmberSpindle, VoidCondenser), (ReflectionLens, MemoryArchive), (SilenceWell, ResonanceForge).
    let pairs: [(NodeId, NodeId); 3] = [
        (NodeId::EmberSpindle, NodeId::VoidCondenser),
        (NodeId::ReflectionLens, NodeId::MemoryArchive),
        (NodeId::SilenceWell, NodeId::ResonanceForge),
    ];

    let mut lines: Vec<Line> = vec![Line::from("")];

    for (left_id, right_id) in &pairs {
        let Some(left_node) = loom_state
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == *left_id)
        else {
            continue;
        };
        let Some(right_node) = loom_state
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == *right_id)
        else {
            continue;
        };

        let left_rate = node_effective_rate(loom_state, left_node);
        let right_rate = node_effective_rate(loom_state, right_node);

        // Find active pipe between this pair in either direction.
        let lr_pipe_idx = loom_state
            .persistent
            .pipes
            .iter()
            .enumerate()
            .find_map(|(i, p)| {
                if p.from == *left_id && p.to == *right_id && !p.under_construction {
                    Some((i, true))
                } else if p.from == *right_id && p.to == *left_id && !p.under_construction {
                    Some((i, false))
                } else {
                    None
                }
            });

        // Also check for any pipe from left to anywhere else (cross-pair).
        let left_has_cross = loom_state.persistent.pipes.iter().any(|p| {
            p.from == *left_id && p.to != *right_id && p.to != *left_id && !p.under_construction
        });
        let right_has_cross = loom_state.persistent.pipes.iter().any(|p| {
            p.from == *right_id && p.to != *left_id && p.to != *right_id && !p.under_construction
        });

        // Line 1: node names + level + stall.
        lines.push(build_flow_node_header_line(
            left_node,
            right_node,
            left_has_cross,
            right_has_cross,
        ));

        // Line 2: buffer bars.
        lines.push(build_flow_buffer_line(left_node, right_node));

        // Line 3: production rate + pipe connector.
        let pipe_label = match &lr_pipe_idx {
            Some((idx, left_to_right)) => {
                let flow = pipe_flow_rate(loom_state, *idx);
                let arrow = if *left_to_right {
                    "\u{25b6}"
                } else {
                    "\u{25c4}"
                };
                let tier = loom_state.persistent.pipes[*idx].tier;
                format!(
                    "{}{:.1}/hr{}",
                    if *left_to_right {
                        "\u{2500}\u{2500}"
                    } else {
                        ""
                    },
                    flow,
                    arrow
                ) + &format!("[{:?}]", tier)
            }
            None => {
                // Check for construction pipe.
                let constr = loom_state.persistent.pipes.iter().any(|p| {
                    (p.from == *left_id && p.to == *right_id
                        || p.from == *right_id && p.to == *left_id)
                        && p.under_construction
                });
                if constr {
                    "\u{2508}\u{2508}[build]\u{2508}\u{2508}".to_string()
                } else {
                    "           ".to_string()
                }
            }
        };

        lines.push(build_flow_rate_line(
            left_node,
            left_rate,
            right_node,
            right_rate,
            &pipe_label,
        ));

        // Gap between pairs.
        lines.push(Line::from(""));
    }

    // Show cross-pair pipes summary if any exist.
    let cross_pipes: Vec<_> = loom_state
        .persistent
        .pipes
        .iter()
        .enumerate()
        .filter(|(_, p)| {
            // Cross-pair: not within same archetype pair.
            let same_pair = pairs
                .iter()
                .any(|(l, r)| (p.from == *l && p.to == *r) || (p.from == *r && p.to == *l));
            !same_pair && !p.under_construction
        })
        .collect();

    if !cross_pipes.is_empty() {
        lines.push(Line::from(Span::styled(
            " Cross-node pipes:",
            Style::default().fg(Color::Rgb(140, 100, 180)),
        )));
        for (idx, pipe) in &cross_pipes {
            let flow = pipe_flow_rate(loom_state, *idx);
            let resource = node_native_resource(pipe.from);
            lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(
                    pipe.from.name(),
                    Style::default().fg(Color::Rgb(180, 140, 220)),
                ),
                Span::styled(
                    format!(" \u{2500}{:.1}/hr\u{25b6} ", flow),
                    Style::default().fg(Color::Rgb(100, 160, 100)),
                ),
                Span::styled(
                    pipe.to.name(),
                    Style::default().fg(Color::Rgb(180, 140, 220)),
                ),
                Span::styled(
                    format!(" ({}) {:?}", resource_name(&resource), pipe.tier),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }
    }

    let para = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .style(Style::default().bg(Color::Rgb(10, 5, 18)));
    frame.render_widget(para, diagram_left);

    // ── Right: stockpiles panel ───────────────────────────────────────────────
    render_stockpiles_panel(frame, stockpile_right, loom_state);

    if has_patterns {
        render_pattern_bar(frame, pattern_area, loom_state);
    }
}

/// Build the header line for a node pair: "Name Lv stall  |  Name Lv stall".
fn build_flow_node_header_line(
    left: &crate::loom::types::LoomNode,
    right: &crate::loom::types::LoomNode,
    left_cross: bool,
    right_cross: bool,
) -> Line<'static> {
    fn node_header(node: &crate::loom::types::LoomNode, has_cross: bool) -> Vec<Span<'static>> {
        if !node.unlocked {
            return vec![Span::styled(
                format!("  {:<16} [locked]", node.id.name()),
                Style::default().fg(Color::Rgb(50, 38, 65)),
            )];
        }
        let stall = if node.stalled {
            Span::styled(" \u{26a0}", Style::default().fg(Color::Rgb(220, 60, 60)))
        } else {
            Span::styled("  ", Style::default())
        };
        let cross = if has_cross {
            Span::styled("\u{2197}", Style::default().fg(Color::Rgb(160, 120, 200)))
        } else {
            Span::styled(" ", Style::default())
        };
        // Confluence nodes (non-base-nature) get a ✦ marker.
        let marker = match node.id.nature() {
            crate::loom::types::NodeNature::Heat
            | crate::loom::types::NodeNature::Form
            | crate::loom::types::NodeNature::Void
            | crate::loom::types::NodeNature::Pattern
            | crate::loom::types::NodeNature::Stillness
            | crate::loom::types::NodeNature::Vibration => " ",
        };
        vec![
            Span::styled(
                format!(" {:<16}", node.id.name()),
                Style::default().fg(Color::Rgb(200, 160, 240)),
            ),
            Span::styled(
                format!("L{}", node.level),
                Style::default().fg(Color::Rgb(120, 90, 160)),
            ),
            stall,
            cross,
            Span::styled(marker, Style::default()),
        ]
    }

    let mut spans = node_header(left, left_cross);
    spans.push(Span::styled(
        "  \u{2502}  ",
        Style::default().fg(Color::Rgb(60, 45, 80)),
    ));
    spans.extend(node_header(right, right_cross));
    Line::from(spans)
}

/// Build the buffer bar line for a node pair.
fn build_flow_buffer_line(
    left: &crate::loom::types::LoomNode,
    right: &crate::loom::types::LoomNode,
) -> Line<'static> {
    fn node_buffer(node: &crate::loom::types::LoomNode) -> Vec<Span<'static>> {
        if !node.unlocked {
            return vec![Span::styled(format!("{:26}", ""), Style::default())];
        }
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
        let filled = ((fill * 8.0) as usize).min(8);
        let empty = 8usize.saturating_sub(filled);
        vec![
            Span::styled("  [", Style::default().fg(Color::DarkGray)),
            Span::styled("\u{2588}".repeat(filled), Style::default().fg(bar_color)),
            Span::styled(
                "\u{2591}".repeat(empty),
                Style::default().fg(Color::Rgb(40, 30, 55)),
            ),
            Span::styled("] ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:>4.1}/{:.1}", node.buffer, node.buffer_capacity),
                Style::default().fg(bar_color),
            ),
        ]
    }

    let mut spans = node_buffer(left);
    spans.push(Span::styled(
        "  \u{2502}  ",
        Style::default().fg(Color::Rgb(60, 45, 80)),
    ));
    spans.extend(node_buffer(right));
    Line::from(spans)
}

/// Build the production rate + pipe connector line.
fn build_flow_rate_line(
    left: &crate::loom::types::LoomNode,
    left_rate: f64,
    right: &crate::loom::types::LoomNode,
    right_rate: f64,
    pipe_label: &str,
) -> Line<'static> {
    let left_rate_str = if left.unlocked {
        format!("  {:.1}/hr", left_rate)
    } else {
        format!("{:10}", "")
    };
    let right_rate_str = if right.unlocked {
        format!("  {:.1}/hr", right_rate)
    } else {
        format!("{:10}", "")
    };

    // Pipe label centered in 11 chars.
    let pipe_center = format!("{:^11}", pipe_label);

    Line::from(vec![
        Span::styled(
            left_rate_str,
            Style::default().fg(Color::Rgb(100, 160, 100)),
        ),
        Span::styled(pipe_center, Style::default().fg(Color::Rgb(140, 100, 180))),
        Span::styled(
            right_rate_str,
            Style::default().fg(Color::Rgb(100, 160, 100)),
        ),
    ])
}

/// Render the stockpiles side panel.
fn render_stockpiles_panel(frame: &mut Frame, area: Rect, loom_state: &LoomState) {
    use crate::loom::types::Resource;

    let block = Block::default()
        .title(" Stockpiles ")
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(Color::Rgb(60, 45, 80)));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Base resources.
    let base_resources = [
        Resource::Ember,
        Resource::Reflection,
        Resource::VoidEssence,
        Resource::Memory,
        Resource::Silence,
        Resource::Resonance,
    ];
    // Confluence + reaction products.
    let derived_resources = [
        Resource::ForgedLight,
        Resource::EchoGlass,
        Resource::StillbornSong,
        Resource::CondensedEmber,
        Resource::EmberEcho,
        Resource::PurifiedVoid,
        Resource::WovenReality,
    ];

    let mut lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(Span::styled(
            " Base",
            Style::default().fg(Color::Rgb(120, 90, 140)),
        )),
    ];

    for res in &base_resources {
        let amount = loom_state
            .persistent
            .stockpiles
            .get(res)
            .copied()
            .unwrap_or(0.0);
        let color = if amount > 0.0 {
            Color::Rgb(200, 160, 240)
        } else {
            Color::Rgb(50, 38, 65)
        };
        lines.push(Line::from(Span::styled(
            format!(" {:<12}{:>5.1}", resource_name(res), amount),
            Style::default().fg(color),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        " Derived",
        Style::default().fg(Color::Rgb(120, 90, 140)),
    )));

    for res in &derived_resources {
        let amount = loom_state
            .persistent
            .stockpiles
            .get(res)
            .copied()
            .unwrap_or(0.0);
        if amount > 0.0 {
            lines.push(Line::from(Span::styled(
                format!(" {:<12}{:>5.1}", resource_name(res), amount),
                Style::default().fg(Color::Rgb(220, 180, 255)),
            )));
        }
    }

    let any_derived = derived_resources.iter().any(|r| {
        loom_state
            .persistent
            .stockpiles
            .get(r)
            .copied()
            .unwrap_or(0.0)
            > 0.0
    });
    if !any_derived {
        lines.push(Line::from(Span::styled(
            " (none yet)",
            Style::default().fg(Color::Rgb(50, 38, 65)),
        )));
    }

    lines.truncate(inner.height as usize);
    frame.render_widget(
        Paragraph::new(lines).style(Style::default().bg(Color::Rgb(10, 5, 18))),
        inner,
    );
}

fn render_list_detail(frame: &mut Frame, area: Rect, loom_state: &LoomState, ui: &LoomUiState) {
    use crate::loom::types::NodeId;
    use crate::loom::{node_effective_rate, node_upgrade_cost};

    // Split into left list (60%) and right detail panel (40%).
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(area);
    let list_area = h_chunks[0];
    let detail_area = h_chunks[1];

    let nodes = NodeId::ALL;
    let selected_node_id = nodes[ui.selected_node.min(nodes.len() - 1)];

    // ── Left: node list ────────────────────────────────────────────────────────
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Nodes",
            Style::default().fg(Color::Rgb(180, 120, 220)),
        )),
        Line::from(""),
    ];

    for (i, node_id) in nodes.iter().enumerate() {
        let Some(node) = loom_state
            .persistent
            .nodes
            .iter()
            .find(|n| n.id == *node_id)
        else {
            continue;
        };
        let is_selected = ui.selected_node == i;
        let prefix = if is_selected { "\u{25b6} " } else { "  " };

        let color = if !node.unlocked {
            Color::Rgb(60, 45, 80)
        } else if is_selected {
            Color::White
        } else {
            Color::Rgb(120, 80, 160)
        };

        let stall_marker = if node.stalled {
            Span::styled(" \u{26a0}", Style::default().fg(Color::Rgb(220, 60, 60)))
        } else {
            Span::raw("")
        };
        let lock_marker = if !node.unlocked {
            Span::styled(" [locked]", Style::default().fg(Color::Rgb(60, 45, 80)))
        } else {
            Span::raw("")
        };

        lines.push(Line::from(vec![
            Span::styled(prefix, Style::default().fg(color)),
            Span::styled(
                format!("{:<16}", node_id.name()),
                Style::default().fg(color),
            ),
            if node.unlocked {
                Span::styled(
                    format!("L{}", node.level),
                    Style::default().fg(Color::Rgb(100, 75, 140)),
                )
            } else {
                Span::raw("")
            },
            stall_marker,
            lock_marker,
        ]));

        // Show pipes for the selected node inline in the list.
        if is_selected && node.unlocked {
            let outgoing_pipes: Vec<_> = loom_state
                .persistent
                .pipes
                .iter()
                .filter(|p| p.from == *node_id && !p.under_construction)
                .collect();
            let construction_pipes: Vec<_> = loom_state
                .persistent
                .pipes
                .iter()
                .filter(|p| p.from == *node_id && p.under_construction)
                .collect();

            if outgoing_pipes.is_empty() && construction_pipes.is_empty() {
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
                    let bar = build_ratio_bar(pipe.split_ratio, 8);
                    lines.push(Line::from(vec![
                        Span::styled(pipe_prefix, Style::default().fg(pipe_color)),
                        Span::styled(
                            format!("\u{2192} {:<14}", pipe.to.name()),
                            Style::default().fg(pipe_color),
                        ),
                        Span::styled(
                            format!("{:>3}% {}", ratio_pct, bar),
                            Style::default().fg(pipe_color),
                        ),
                    ]));
                }
                for pipe in &construction_pipes {
                    lines.push(Line::from(vec![
                        Span::styled("     ", Style::default()),
                        Span::styled(
                            format!("\u{2508} {} ", pipe.to.name()),
                            Style::default().fg(Color::Rgb(80, 60, 110)),
                        ),
                        Span::styled("[building]", Style::default().fg(Color::Rgb(80, 60, 110))),
                    ]));
                }
            }
            if !outgoing_pipes.is_empty() {
                lines.push(Line::from(Span::styled(
                    "   [P] Cycle pipe  [\u{2190}/\u{2192}] Ratio",
                    Style::default().fg(Color::DarkGray),
                )));
            }
        }
    }

    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Left)
            .style(Style::default().bg(Color::Rgb(10, 5, 18))),
        list_area,
    );

    // ── Right: selected node detail panel ─────────────────────────────────────
    let detail_block = Block::default()
        .title(format!(" {} ", selected_node_id.name()))
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(Color::Rgb(80, 60, 110)));
    let detail_inner = detail_block.inner(detail_area);
    frame.render_widget(detail_block, detail_area);

    let Some(selected_node) = loom_state
        .persistent
        .nodes
        .iter()
        .find(|n| n.id == selected_node_id)
    else {
        return;
    };

    let mut detail_lines: Vec<Line> = vec![Line::from("")];

    if !selected_node.unlocked {
        detail_lines.push(Line::from(Span::styled(
            " [Locked]",
            Style::default().fg(Color::Rgb(80, 60, 110)),
        )));
        detail_lines.push(Line::from(Span::styled(
            " Unlock via neighbor",
            Style::default().fg(Color::Rgb(60, 45, 80)),
        )));
        detail_lines.push(Line::from(""));
        // Show unlock progress bar if partially unlocked.
        if selected_node.unlock_progress > 0.0 {
            let prog_pct = (selected_node.unlock_progress / 2.0).min(1.0);
            let filled = ((prog_pct * 10.0) as usize).min(10);
            let empty = 10usize.saturating_sub(filled);
            detail_lines.push(Line::from(vec![
                Span::styled(" Progress [", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "\u{2588}".repeat(filled),
                    Style::default().fg(Color::Rgb(100, 80, 160)),
                ),
                Span::styled(
                    "\u{2591}".repeat(empty),
                    Style::default().fg(Color::Rgb(40, 30, 55)),
                ),
                Span::styled("]", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!(" {:.1}h", selected_node.unlock_progress),
                    Style::default().fg(Color::Rgb(100, 80, 160)),
                ),
            ]));
        }
    } else {
        // Nature and native resource.
        let nature_name = node_nature_name(selected_node_id.nature());
        let native = crate::loom::node_native_resource(selected_node_id);
        detail_lines.push(Line::from(vec![
            Span::styled(" Nature: ", Style::default().fg(Color::DarkGray)),
            Span::styled(nature_name, Style::default().fg(Color::Rgb(180, 140, 220))),
        ]));
        detail_lines.push(Line::from(vec![
            Span::styled(" Produces: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                resource_name(&native),
                Style::default().fg(Color::Rgb(220, 180, 255)),
            ),
        ]));
        detail_lines.push(Line::from(""));

        // Buffer bar.
        let fill = if selected_node.buffer_capacity > 0.0 {
            (selected_node.buffer / selected_node.buffer_capacity).min(1.0)
        } else {
            0.0
        };
        let bar_color = if selected_node.stalled || fill >= 0.90 {
            Color::Rgb(220, 60, 60)
        } else if fill >= 0.75 {
            Color::Rgb(220, 180, 60)
        } else {
            Color::Rgb(60, 200, 100)
        };
        let filled_cells = ((fill * 12.0) as usize).min(12);
        let empty_cells = 12usize.saturating_sub(filled_cells);
        detail_lines.push(Line::from(vec![
            Span::styled(" Buffer [", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "\u{2588}".repeat(filled_cells),
                Style::default().fg(bar_color),
            ),
            Span::styled(
                "\u{2591}".repeat(empty_cells),
                Style::default().fg(Color::Rgb(40, 30, 55)),
            ),
            Span::styled("]", Style::default().fg(Color::DarkGray)),
        ]));
        detail_lines.push(Line::from(vec![
            Span::styled(
                format!(
                    " {:.1}/{:.1}",
                    selected_node.buffer, selected_node.buffer_capacity
                ),
                Style::default().fg(bar_color),
            ),
            if selected_node.stalled {
                Span::styled(
                    " \u{26a0} STALLED",
                    Style::default().fg(Color::Rgb(220, 60, 60)),
                )
            } else {
                Span::raw("")
            },
        ]));
        detail_lines.push(Line::from(""));

        // Production rate.
        let rate = node_effective_rate(loom_state, selected_node);
        detail_lines.push(Line::from(vec![
            Span::styled(" Rate: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:.1}/hr", rate),
                Style::default().fg(Color::Rgb(100, 200, 120)),
            ),
            Span::styled(
                format!(" (Lv{})", selected_node.level),
                Style::default().fg(Color::Rgb(80, 60, 100)),
            ),
        ]));
        detail_lines.push(Line::from(""));

        // Upgrade cost.
        let upgrade_cost = node_upgrade_cost(loom_state, selected_node_id);
        let can_upgrade = selected_node.buffer >= upgrade_cost;
        let upgrade_color = if can_upgrade {
            Color::Rgb(180, 255, 180)
        } else {
            Color::Rgb(100, 75, 80)
        };
        detail_lines.push(Line::from(vec![
            Span::styled(" Upgrade: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:.0} {}", upgrade_cost, resource_name(&native)),
                Style::default().fg(upgrade_color),
            ),
        ]));
        if can_upgrade {
            detail_lines.push(Line::from(Span::styled(
                " [U] Upgrade node",
                Style::default().fg(Color::Rgb(120, 200, 120)),
            )));
        } else {
            detail_lines.push(Line::from(Span::styled(
                " (insufficient buffer)",
                Style::default().fg(Color::Rgb(80, 60, 70)),
            )));
        }

        // Incoming pipe count.
        let incoming = loom_state
            .persistent
            .pipes
            .iter()
            .filter(|p| p.to == selected_node_id && !p.under_construction)
            .count();
        let outgoing = loom_state
            .persistent
            .pipes
            .iter()
            .filter(|p| p.from == selected_node_id && !p.under_construction)
            .count();
        if incoming > 0 || outgoing > 0 {
            detail_lines.push(Line::from(""));
            detail_lines.push(Line::from(vec![
                Span::styled(" Pipes: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("\u{2190}{} \u{2192}{}", incoming, outgoing),
                    Style::default().fg(Color::Rgb(140, 100, 180)),
                ),
            ]));
        }
    }

    detail_lines.truncate(detail_inner.height as usize);
    frame.render_widget(
        Paragraph::new(detail_lines).style(Style::default().bg(Color::Rgb(10, 5, 18))),
        detail_inner,
    );
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
        " [Tab] Switch View  [Up/Down] Node  [U] Upgrade  [P] Pipe  [\u{2190}/\u{2192}] Ratio  [Esc] Close "
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
