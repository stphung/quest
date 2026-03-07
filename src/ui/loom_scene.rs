//! The Loom of Worlds overlay — main UI renderer.
#![allow(dead_code)]
//!
//! Dispatches to different view renderers based on `LoomUiState::view`:
//!   - ArchetypeSelection: choose your archetype at unlock
//!   - FlowView:           pipeline diagram placeholder
//!   - ListDetail:         node list + detail panel placeholder
//!   - Codex:              recipe codex placeholder

use crate::loom::types::{LoomArchetype, LoomState, LoomUiState, LoomView};
use ratatui::{
    layout::{Alignment, Rect},
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
            render_flow_view(frame, inner);
        }
        LoomView::ListDetail => {
            render_list_detail(frame, inner, ui);
        }
        LoomView::Codex => {
            render_codex(frame, inner, loom_state);
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

fn render_flow_view(frame: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Flow View \u{2014} Pipeline Diagram",
            Style::default().fg(Color::Rgb(180, 120, 220)),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Node pipeline visualization coming soon.",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "EmberSpindle  \u{2192}  VoidCondenser  \u{2192}  [output]",
            Style::default().fg(Color::Rgb(120, 80, 160)),
        )),
        Line::from(Span::styled(
            "ReflectionLens  \u{2192}  MemoryArchive  \u{2192}  [output]",
            Style::default().fg(Color::Rgb(120, 80, 160)),
        )),
        Line::from(Span::styled(
            "SilenceWell  \u{2192}  ResonanceForge  \u{2192}  [output]",
            Style::default().fg(Color::Rgb(120, 80, 160)),
        )),
    ];

    let para = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .style(Style::default().bg(Color::Rgb(10, 5, 18)));
    frame.render_widget(para, area);
}

fn render_list_detail(frame: &mut Frame, area: Rect, ui: &LoomUiState) {
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

    for (i, node) in nodes.iter().enumerate() {
        let is_selected = ui.selected_node == i;
        let prefix = if is_selected { "\u{25b6} " } else { "  " };
        let color = if is_selected {
            Color::White
        } else {
            Color::Rgb(120, 80, 160)
        };
        lines.push(Line::from(vec![
            Span::styled(prefix, Style::default().fg(color)),
            Span::styled(node.name(), Style::default().fg(color)),
            Span::styled(
                format!(" \u{2014} {:?}", node.nature()),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }

    let para = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .style(Style::default().bg(Color::Rgb(10, 5, 18)));
    frame.render_widget(para, area);
}

fn render_codex(frame: &mut Frame, area: Rect, loom_state: &LoomState) {
    let discovered: Vec<_> = loom_state
        .persistent
        .codex
        .iter()
        .filter(|e| e.discovered)
        .collect();

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Recipe Codex",
            Style::default().fg(Color::Rgb(180, 120, 220)),
        )),
        Line::from(""),
    ];

    if discovered.is_empty() {
        lines.push(Line::from(Span::styled(
            "No recipes discovered yet.",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(Span::styled(
            "Process resources through nodes to reveal reactions.",
            Style::default().fg(Color::Rgb(80, 60, 100)),
        )));
    } else {
        for entry in &discovered {
            let inputs: Vec<&str> = entry.inputs.iter().map(|r| resource_name(r)).collect();
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{} + {:?}", inputs.join(" + "), entry.node_nature),
                    Style::default().fg(Color::Rgb(160, 100, 200)),
                ),
                Span::styled(" \u{2192} ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!(
                        "{} (x{:.1})",
                        resource_name(&entry.output),
                        entry.output_amount
                    ),
                    Style::default().fg(Color::Rgb(220, 180, 255)),
                ),
            ]));
        }
    }

    let para = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .style(Style::default().bg(Color::Rgb(10, 5, 18)));
    frame.render_widget(para, area);
}

// ── Navigation hints ──────────────────────────────────────────────────────────

fn render_nav_hints(frame: &mut Frame, area: Rect, ui: &LoomUiState) {
    if area.height < 3 {
        return;
    }

    let hints = if ui.view == LoomView::ArchetypeSelection {
        " [Up/Down] Select  [Enter] Confirm  [Esc] Close "
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
