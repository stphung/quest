//! Storm Sigil rendering helpers for the stats panel.

use crate::stormglass::sigils::{SigilGrade, StormSigils};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Draws the Storm Sigils panel as a dedicated section.
/// Only call when `storm_sigils.etched_count() > 0`.
pub(super) fn draw_sigils_panel(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    storm_sigils: &StormSigils,
) {
    let etched = storm_sigils.etched_count();
    let title = format!(
        " \u{16B1} Storm Sigils ({}/{}) ",
        etched, storm_sigils.slots_unlocked
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(super::themed_border_color(Color::Rgb(100, 180, 255))))
        .title(title);
    let block = super::themed_block(block);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    super::apply_themed_border_fx(
        frame,
        area,
        Color::Rgb(100, 180, 255),
        super::BorderFxContext,
    );

    let width = inner.width as usize;
    let mut lines = Vec::new();

    for (i, slot) in storm_sigils.sigils.iter().enumerate() {
        if i < storm_sigils.slots_unlocked as usize {
            // Unlocked slot: etched or empty
            if let Some(sigil) = slot {
                let icon = sigil.effect.icon();
                let short = sigil.effect.short_name();
                let value_label = sigil.effect.format_value(sigil.value);
                let grade_str = sigil.grade.label();
                let grade_padded = format!("{:<2}", grade_str);
                let grade_color = sigil_grade_color(sigil.grade);

                let left = format!("{} {}", icon, short);
                let right = format!("{}  {}", value_label, grade_padded);
                let left_display_w = unicode_width::UnicodeWidthStr::width(left.as_str());
                let right_len = right.len();
                let pad = width.saturating_sub(left_display_w + right_len + 3);

                let grade_style = if grade_str.ends_with('+') {
                    Style::default()
                        .fg(grade_color)
                        .add_modifier(Modifier::BOLD)
                } else if grade_str.ends_with('-') {
                    Style::default().fg(grade_color).add_modifier(Modifier::DIM)
                } else {
                    Style::default().fg(grade_color)
                };

                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(left, Style::default().fg(Color::White)),
                    Span::raw(" ".repeat(pad.max(1))),
                    Span::styled(value_label, Style::default().fg(Color::Rgb(100, 180, 255))),
                    Span::styled(format!("  {}", grade_padded), grade_style),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled("\u{00b7} empty", Style::default().fg(Color::DarkGray)),
                ]));
            }
        } else {
            // Locked slot
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("\u{1f512} locked", Style::default().fg(Color::DarkGray)),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

/// Returns the color for a sigil grade tier letter.
fn sigil_grade_color(grade: SigilGrade) -> Color {
    match grade.tier_letter() {
        'S' => Color::Rgb(255, 215, 0),
        'A' => Color::Green,
        'B' => Color::Cyan,
        'C' => Color::White,
        'D' => Color::Gray,
        'E' => Color::DarkGray,
        _ => Color::Red,
    }
}
