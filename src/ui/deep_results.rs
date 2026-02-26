//! The Deep — Mission complete results modal.

use crate::deep::{DeepState, MercStatus, Mission, MissionOutcome};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::responsive::LayoutContext;

pub(super) fn render_mission_results(
    frame: &mut Frame,
    area: Rect,
    mission: &Mission,
    deep: &DeepState,
    _ctx: &LayoutContext,
) {
    let Some(result) = &mission.result else {
        return;
    };

    // Center the modal
    let modal_width = 56u16.min(area.width.saturating_sub(4));
    let modal_height = 20u16.min(area.height.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    frame.render_widget(Clear, modal_area);

    let (outcome_str, outcome_color) = match result.outcome {
        MissionOutcome::Success => ("SUCCESS", Color::Green),
        MissionOutcome::PartialSuccess => ("PARTIAL SUCCESS", Color::Yellow),
        MissionOutcome::Failure => ("FAILURE", Color::Red),
    };

    let block = Block::default()
        .title(" Mission Complete ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(super::themed_border_color(outcome_color)));
    let inner = super::render_themed_block(
        frame,
        modal_area,
        block,
        outcome_color,
        super::BorderFxContext,
    );

    let mut lines: Vec<Line> = Vec::new();

    // Mission header
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        format!(
            "{} \u{2014} Layer {} \u{2014} {}",
            mission.mission_type.display_name(),
            mission.layer,
            crate::deep::LayerTier::from_layer(mission.layer).display_name()
        ),
        Style::default().fg(Color::White),
    )]));

    // Outcome
    lines.push(Line::from(vec![
        Span::styled("Result: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            outcome_str,
            Style::default()
                .fg(outcome_color)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(""));

    // Rewards section
    lines.push(Line::from(Span::styled(
        "Rewards:",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )));

    if result.marks_earned > 0 {
        lines.push(Line::from(Span::styled(
            format!("  + {} Warband Marks", result.marks_earned),
            Style::default().fg(Color::Yellow),
        )));
    }
    if result.xp_earned > 0 {
        lines.push(Line::from(Span::styled(
            format!("  + {} XP", result.xp_earned),
            Style::default().fg(Color::Cyan),
        )));
    }
    if result.stormglass_earned > 0 {
        lines.push(Line::from(Span::styled(
            format!("  + {} Stormglass", result.stormglass_earned),
            Style::default().fg(Color::Rgb(100, 180, 255)),
        )));
    }
    if let Some(ilvl) = result.item_ilvl {
        lines.push(Line::from(Span::styled(
            format!("  + Item (ilvl {})", ilvl),
            Style::default().fg(Color::Magenta),
        )));
    }

    lines.push(Line::from(""));

    // Squad status section
    lines.push(Line::from(Span::styled(
        "Squad:",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )));

    for merc_id in &mission.squad {
        if let Some(merc) = deep.prestige.find_merc(*merc_id) {
            let (icon, label, color) = if result.lost_mercs.contains(merc_id) {
                ("\u{2717}", "lost", Color::Red)
            } else if result.injured_mercs.contains(merc_id) {
                let missions = match &merc.status {
                    MercStatus::Injured { missions_remaining } => *missions_remaining,
                    _ => 2,
                };
                let _ = missions;
                ("!", "injured", Color::Yellow)
            } else {
                ("\u{2713}", "returned safely", Color::Green)
            };

            // Level-up?
            let level_up_str = result
                .merc_level_ups
                .iter()
                .find(|(id, _)| *id == *merc_id)
                .map(|(_, gained)| format!(" (+{} level)", gained))
                .unwrap_or_default();

            lines.push(Line::from(vec![
                Span::styled(format!("  {} ", icon), Style::default().fg(color)),
                Span::styled(
                    format!(
                        "{} ({} L{})",
                        merc.name,
                        merc.archetype.display_name(),
                        merc.level
                    ),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!(" \u{2014} {}{}", label, level_up_str),
                    Style::default().fg(color),
                ),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "[Enter] Collect and Close",
        Style::default().fg(Color::DarkGray),
    )));

    let text = Paragraph::new(lines).alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(text, inner);
}
