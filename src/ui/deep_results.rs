//! The Deep — Mission complete results modal.

use crate::deep::{DeepState, LayerTier, Mercenary, Mission, MissionOutcome};
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
    // Extra rows needed for per-merc progress bars (1 bar line per surviving squad member)
    let squad_bar_rows = mission.squad.len() as u16;
    let modal_height = (20 + squad_bar_rows).min(area.height.saturating_sub(4));
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
            Style::default().fg(Color::Rgb(220, 180, 60)),
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
            let level_up = result
                .merc_level_ups
                .iter()
                .find(|(id, _)| *id == *merc_id)
                .map(|(_, gained)| *gained);

            let (icon, label, color) = if result.lost_mercs.contains(merc_id) {
                ("\u{2717}", "lost".to_string(), Color::Red)
            } else if result.injured_mercs.contains(merc_id) {
                ("!", "injured".to_string(), Color::Yellow)
            } else if let Some(gained) = level_up {
                let old_level = merc.level.saturating_sub(gained);
                (
                    "\u{2605}",
                    format!("L{} \u{2192} L{}!", old_level, merc.level),
                    Color::Rgb(255, 215, 0),
                )
            } else {
                ("\u{2713}", "safe".to_string(), Color::Green)
            };

            // Build merc header line
            let level_display = if level_up.is_some() {
                String::new() // level info is in the label
            } else {
                format!(" L{}", merc.level)
            };
            lines.push(Line::from(vec![
                Span::styled(format!("  {} ", icon), Style::default().fg(color)),
                Span::styled(
                    format!(
                        "{} ({}{})",
                        merc.name,
                        merc.archetype.display_name(),
                        level_display,
                    ),
                    Style::default().fg(Color::White),
                ),
                Span::styled(format!(" \u{2014} {}", label), Style::default().fg(color)),
            ]));

            // Progress bar toward next level (skip for lost mercs and max level)
            if !result.lost_mercs.contains(merc_id) && merc.level < 20 {
                let needed = Mercenary::missions_to_next_level(merc.level);
                let progress = if needed > 0 {
                    merc.missions_completed % needed
                } else {
                    0
                };
                let bar_width = 10usize;
                let ratio = if needed > 0 {
                    progress as f64 / needed as f64
                } else {
                    0.0
                };
                let filled = ((ratio * bar_width as f64).round() as usize).min(bar_width);
                let bar: String =
                    "\u{2588}".repeat(filled) + &"\u{2591}".repeat(bar_width - filled);
                let bar_color = if level_up.is_some() {
                    Color::Rgb(255, 215, 0)
                } else {
                    Color::Cyan
                };
                lines.push(Line::from(vec![
                    Span::styled("    ", Style::default()),
                    Span::styled(bar, Style::default().fg(bar_color)),
                    Span::styled(
                        format!(" {}/{} to L{}", progress, needed, merc.level + 1),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
            }
        }
    }

    // Debrief flavor text
    let tier = LayerTier::from_layer(mission.layer);
    let flavor = debrief_flavor(tier, &result.outcome);
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        flavor,
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC),
    )));

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "[Enter] Collect and Close",
        Style::default().fg(Color::DarkGray),
    )));

    let text = Paragraph::new(lines).alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(text, inner);
}

/// Tier-based procedural flavor text for mission debriefs.
fn debrief_flavor(tier: LayerTier, outcome: &MissionOutcome) -> &'static str {
    match (tier, outcome) {
        (LayerTier::Shallows, MissionOutcome::Success) => {
            "The squad reports stable tunnels and breathable air."
        }
        (LayerTier::Shallows, _) => "The upper passages proved more treacherous than expected.",
        (LayerTier::Warrens, MissionOutcome::Success) => {
            "The Warrens yielded their secrets reluctantly."
        }
        (LayerTier::Warrens, _) => "Something in the Warrens was waiting for them.",
        (LayerTier::Hollows, MissionOutcome::Success) => {
            "The bioluminescence guided them deeper than planned."
        }
        (LayerTier::Hollows, _) => "The spore clouds were thicker than the maps suggested.",
        (LayerTier::SunkenReach, MissionOutcome::Success) => {
            "The seals parted for them. Not everyone finds that reassuring."
        }
        (LayerTier::SunkenReach, _) => "The water pressure claimed equipment. And patience.",
        (LayerTier::Abyss, MissionOutcome::Success) => {
            "They returned with six days of rations consumed. They were gone four hours."
        }
        (LayerTier::Abyss, _) => "Time moved differently down there. It always does.",
        (LayerTier::Void, MissionOutcome::Success) => {
            "What they found cannot be mapped. Only remembered."
        }
        (LayerTier::Void, _) => "The Void gives nothing freely. The cost is always personal.",
    }
}
