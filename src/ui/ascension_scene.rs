//! Ascension confirmation dialog overlay.

use super::responsive::LayoutContext;
use super::stats_prestige::to_roman;
use crate::ascension::can_ascend;
use crate::ascension::types::{
    ascension_combat_multiplier, ascension_cost, ascension_deep_gate, MAX_ASCENSION_LEVEL,
};
use crate::core::game_state::GameState;
use crate::deep::DeepState;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

const GOLD: Color = Color::Rgb(255, 215, 0);

/// Render the Ascension confirmation dialog.
pub fn render_ascension_confirm(
    frame: &mut Frame,
    area: Rect,
    state: &GameState,
    deep_state: &DeepState,
    _ctx: &LayoutContext,
) {
    let modal_width = 60u16.min(area.width.saturating_sub(4));
    let modal_height = 18u16.min(area.height.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(
            Line::from(Span::styled(
                " Ascension ",
                Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(super::themed_border_color(GOLD)));
    let inner = super::render_themed_block(frame, modal_area, block, GOLD, super::BorderFxContext);

    let current_level = state.ascension_level;
    let current_mult = ascension_combat_multiplier(current_level);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));

    // Current level
    if current_level == 0 {
        lines.push(Line::from(Span::styled(
            "Not yet Ascended",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        lines.push(Line::from(vec![
            Span::styled("Current: ", Style::default().fg(Color::White)),
            Span::styled(
                format!(
                    "Ascension {} ({:.1}x)",
                    to_roman(current_level),
                    current_mult
                ),
                Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    lines.push(Line::from(""));

    if current_level >= MAX_ASCENSION_LEVEL {
        // At max level — show capped message
        lines.push(Line::from(Span::styled(
            "Maximum Ascension reached.",
            Style::default()
                .fg(GOLD)
                .add_modifier(Modifier::BOLD | Modifier::ITALIC),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("Power \u{00d7}{:.1}", current_mult),
            Style::default().fg(Color::Cyan),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "[Esc] Close",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        // Normal next-level dialog
        let next_level = current_level + 1;
        let next_mult = ascension_combat_multiplier(next_level);
        let cost = ascension_cost(next_level);
        let deep_gate = ascension_deep_gate(next_level);
        let deepest = deep_state.persistent.deepest_layer_reached;
        let can_afford_pr = state.prestige_rank >= cost;
        let can_pass_gate = deep_gate.is_none_or(|g| deepest >= g);
        let eligible = can_ascend(current_level, state.prestige_rank, deepest);

        // Next level
        lines.push(Line::from(vec![
            Span::styled("Next: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("Ascension {} ({:.1}x)", to_roman(next_level), next_mult),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));

        lines.push(Line::from(""));

        // Cost
        let cost_color = if can_afford_pr {
            Color::Green
        } else {
            Color::Red
        };
        lines.push(Line::from(vec![
            Span::styled("Cost: ", Style::default().fg(Color::White)),
            Span::styled(format!("{} PR", cost), Style::default().fg(cost_color)),
            Span::styled(
                format!("  (have: {})", state.prestige_rank),
                Style::default().fg(Color::DarkGray),
            ),
        ]));

        // Deep layer requirement
        if let Some(required_layer) = deep_gate {
            let met = can_pass_gate;
            let color = if met { Color::Green } else { Color::Red };
            lines.push(Line::from(vec![
                Span::styled("Requires: ", Style::default().fg(Color::White)),
                Span::styled(
                    format!("Deep Layer {}", required_layer),
                    Style::default().fg(color),
                ),
                Span::styled(
                    format!("  (reached: {})", deepest),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        } else {
            lines.push(Line::from(Span::styled(
                "No Deep requirement (PR only)",
                Style::default().fg(Color::DarkGray),
            )));
        }

        lines.push(Line::from(""));

        // Multiplier jump
        lines.push(Line::from(Span::styled(
            format!(
                "Power \u{00d7}{:.1} \u{2192} \u{00d7}{:.1}",
                current_mult, next_mult
            ),
            Style::default().fg(Color::Cyan),
        )));
        lines.push(Line::from(Span::styled(
            "Ascension survives prestige.",
            Style::default().fg(Color::DarkGray),
        )));

        lines.push(Line::from(""));

        // Action hints
        if eligible {
            lines.push(Line::from(vec![
                Span::styled(
                    "[Y] Ascend",
                    Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
                ),
                Span::raw("    "),
                Span::styled("[Esc] Cancel", Style::default().fg(Color::DarkGray)),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled("[Y] Ascend", Style::default().fg(Color::DarkGray)),
                Span::raw("    "),
                Span::styled("[Esc] Cancel", Style::default().fg(Color::DarkGray)),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(paragraph, inner);
}
