//! The Deep — Event response modal rendering.

use crate::deep::{DeepState, DeepUiState};
use chrono::Utc;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use super::deep_missions::archetype_color;
use super::deep_shared::format_hours;
use super::responsive::{LayoutContext, SizeTier};
use super::scene_fx::current_millis;

/// Auto-resolve fires after 30 minutes of no response.
const AUTO_RESOLVE_SECS: u64 = 30 * 60;

fn urgency_status(remaining_secs: u64) -> (&'static str, Color) {
    if remaining_secs <= 5 * 60 {
        ("CRITICAL", Color::LightRed)
    } else if remaining_secs <= 15 * 60 {
        ("URGENT", Color::Yellow)
    } else {
        ("STABLE", Color::Green)
    }
}

// ── Frame-based event modal (rendered over Hub) ─────────────────────────────

/// Render the event response as a centered modal overlay using Frame widgets.
pub(super) fn render_event_modal(
    frame: &mut Frame,
    area: Rect,
    deep: &DeepState,
    ui: &DeepUiState,
    ctx: &LayoutContext,
) {
    // Find the mission with the pending event
    let mission = ui
        .event_mission_id
        .and_then(|id| deep.session.active_missions.iter().find(|m| m.id == id));

    let Some(mission) = mission else {
        return;
    };

    let event = mission.events.iter().find(|e| !e.is_resolved());
    let Some(event) = event else {
        return;
    };

    let now = Utc::now();
    let seconds_since_fired = (now - event.fired_at).num_seconds().max(0) as u64;
    let remaining = AUTO_RESOLVE_SECS.saturating_sub(seconds_since_fired);
    let ratio = remaining as f64 / AUTO_RESOLVE_SECS as f64;
    let (urgency_label, base_urgency_color) = urgency_status(remaining);
    let pulse_on = remaining <= 5 * 60 && (current_millis() / 350).is_multiple_of(2);
    let urgency_color = if pulse_on {
        Color::Red
    } else {
        base_urgency_color
    };

    // Modal sizing
    let target_width = match ctx.tier {
        SizeTier::XL => 80u16,
        SizeTier::L => 70u16,
        SizeTier::M => 62u16,
        SizeTier::S | SizeTier::TooSmall => 54u16,
    };
    let choice_count = event.choices.len() + 1; // +1 for safe fallback
    let base_rows = 12u16; // header + narrative + timer + footer
    let choice_rows = (choice_count as u16) * 2; // 2 rows per choice
    let modal_height = (base_rows + choice_rows)
        .max(18)
        .min(area.height.saturating_sub(4));
    let modal_width = target_width.min(area.width.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    frame.render_widget(Clear, modal_area);

    let border_color = if pulse_on { Color::Red } else { urgency_color };
    let title = format!(" \u{26a1} Event \u{2014} {} ", urgency_label);
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(super::themed_border_color(border_color)));
    let inner = super::render_themed_block(
        frame,
        modal_area,
        block,
        border_color,
        super::BorderFxContext,
    );

    if inner.height < 4 || inner.width < 10 {
        return;
    }

    let iw = inner.width as usize;

    // Build content lines
    let mut lines: Vec<Line<'_>> = Vec::new();

    // Mission info line
    let progress = mission.progress(now);
    let mission_info = format!(
        "{}  Layer {}  \u{2014}  {}% complete",
        mission.mission_type.display_name(),
        mission.layer,
        (progress * 100.0) as u32,
    );
    lines.push(Line::from(Span::styled(
        mission_info,
        Style::default().fg(Color::DarkGray),
    )));

    // Squad names
    let squad_names: Vec<String> = mission
        .squad
        .iter()
        .filter_map(|id| deep.session.find_merc(*id))
        .map(|m| format!("{} ({})", m.name, m.archetype.display_name()))
        .collect();
    if !squad_names.is_empty() {
        let squad_text = format!("Squad: {}", squad_names.join(", "));
        let max_len = iw.saturating_sub(2);
        let display: String = if squad_text.chars().count() > max_len {
            squad_text.chars().take(max_len).collect()
        } else {
            squad_text
        };
        lines.push(Line::from(Span::styled(
            display,
            Style::default().fg(Color::DarkGray),
        )));
    }

    // Separator
    let sep: String = "\u{2500}".repeat(iw.saturating_sub(2));
    lines.push(Line::from(Span::styled(
        sep,
        Style::default().fg(Color::Rgb(40, 60, 80)),
    )));

    // Event title
    let title_text = format!("\u{26a1} {}", event.title);
    lines.push(Line::from(Span::styled(
        title_text,
        Style::default()
            .fg(urgency_color)
            .add_modifier(Modifier::BOLD),
    )));

    // Event description (word-wrapped)
    let desc_words: Vec<&str> = event.description.split_whitespace().collect();
    let max_line_w = iw.saturating_sub(4).max(10);
    let mut line_buf = String::new();
    for word in &desc_words {
        if line_buf.len() + word.len() + 1 > max_line_w && !line_buf.is_empty() {
            lines.push(Line::from(Span::styled(
                format!("  {}", line_buf),
                Style::default().fg(Color::White),
            )));
            line_buf.clear();
        }
        if !line_buf.is_empty() {
            line_buf.push(' ');
        }
        line_buf.push_str(word);
    }
    if !line_buf.is_empty() {
        lines.push(Line::from(Span::styled(
            format!("  {}", line_buf),
            Style::default().fg(Color::White),
        )));
    }

    // Blank line before timer
    lines.push(Line::from(""));

    // Timer line
    let mm = remaining / 60;
    let ss = remaining % 60;
    let bar_width = 20usize.min(iw.saturating_sub(30));
    let filled = ((ratio * bar_width as f64).round() as usize).min(bar_width);
    let bar: String = (0..bar_width)
        .map(|i| if i < filled { '\u{2588}' } else { '\u{2591}' })
        .collect();
    let timer_text = format!(
        "  Response Timer [{}]  {}  {:02}:{:02}",
        urgency_label, bar, mm, ss
    );
    lines.push(Line::from(Span::styled(
        timer_text,
        Style::default().fg(urgency_color),
    )));

    // Blank line before choices
    lines.push(Line::from(""));

    // Squad archetypes for checking availability
    let squad_archetypes: Vec<crate::deep::MercArchetype> = mission
        .squad
        .iter()
        .filter_map(|id| deep.session.find_merc(*id))
        .map(|m| m.archetype)
        .collect();

    // Choices
    for (ci, choice) in event.choices.iter().enumerate() {
        let is_sel = ci == ui.event_choice_index;
        let is_available = choice
            .required_archetype
            .map(|a| squad_archetypes.contains(&a))
            .unwrap_or(true);

        let cursor = if is_sel { "\u{25b6} " } else { "  " };
        let num = format!("[{}] ", ci + 1);

        let arch_tag = match choice.required_archetype {
            Some(arch) => format!("[{}] ", arch.display_name().to_uppercase()),
            None => String::new(),
        };

        let consequence = if choice.is_risky {
            match choice.risk_percent {
                Some(pct) => format!(" \u{2014} ~{}% injury risk", pct),
                None => " \u{2014} risky".to_string(),
            }
        } else if choice.time_delta_secs > 0 {
            format!(" \u{2014} +{}", format_hours(choice.time_delta_secs as u64))
        } else if choice.time_delta_secs < 0 {
            format!(
                " \u{2014} -{}",
                format_hours(choice.time_delta_secs.unsigned_abs())
            )
        } else {
            " \u{2014} safe".to_string()
        };

        let tag = if !is_available {
            (" LOCKED", Color::DarkGray)
        } else if choice.is_risky {
            (" RISK", Color::Yellow)
        } else {
            (" SAFE", Color::Green)
        };

        let label_color = if is_available {
            Color::White
        } else {
            Color::DarkGray
        };
        let cursor_color = if is_sel { Color::Cyan } else { Color::DarkGray };
        let arch_color = match choice.required_archetype {
            Some(arch) if is_available => archetype_color(arch),
            _ => Color::DarkGray,
        };

        let mut spans = vec![
            Span::styled(cursor, Style::default().fg(cursor_color)),
            Span::styled(num, Style::default().fg(Color::DarkGray)),
            Span::styled(arch_tag, Style::default().fg(arch_color)),
            Span::styled(choice.label.clone(), Style::default().fg(label_color)),
            Span::styled(consequence, Style::default().fg(Color::DarkGray)),
            Span::styled(tag.0, Style::default().fg(tag.1)),
        ];
        if !is_available {
            if let Some(arch) = choice.required_archetype {
                spans.push(Span::styled(
                    format!(" ({} missing)", arch.display_name()),
                    Style::default().fg(Color::DarkGray),
                ));
            }
        }
        lines.push(Line::from(spans));
    }

    // Safe fallback option
    let safe_idx = event.choices.len();
    let is_sel = ui.event_choice_index == safe_idx;
    let cursor = if is_sel { "\u{25b6} " } else { "  " };
    let cursor_color = if is_sel { Color::Cyan } else { Color::DarkGray };
    lines.push(Line::from(vec![
        Span::styled(cursor, Style::default().fg(cursor_color)),
        Span::styled(
            format!("[{}] ", safe_idx + 1),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(
            "Play it safe (no risk, no delay)",
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(" SAFE", Style::default().fg(Color::Green)),
    ]));

    // Blank + footer
    lines.push(Line::from(""));
    let footer = match ctx.tier {
        SizeTier::S => "[\u{2191}/\u{2193}] Choose  [Enter/1-3] Confirm  [Esc] Close",
        _ => "[\u{2191}/\u{2193}] Choose  [Enter] Confirm  [1-3] Quick pick  [Esc] Close",
    };
    lines.push(Line::from(Span::styled(
        footer,
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}
