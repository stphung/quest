//! Storm Sigil rendering helpers for the stats panel.

use crate::stormglass::sigils::{SigilGrade, StormSigils};
use ratatui::{
    layout::Position,
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

    // Storm sigil visual effects — scale with total grade score.
    let grade_total = sigil_grade_total(storm_sigils);
    if grade_total > 0 {
        paint_sigil_bg(frame, inner, grade_total);
        paint_sigil_heat_line(frame, inner, grade_total);
        paint_sigil_motes(frame, inner, grade_total);
    }
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

/// Sum of all etched sigil grade ordinals. F-=0, S+=20. Max = 5×20 = 100.
fn sigil_grade_total(storm_sigils: &StormSigils) -> u16 {
    storm_sigils
        .sigils
        .iter()
        .filter_map(|s| s.as_ref())
        .map(|s| s.grade as u16)
        .sum()
}

/// Storm-blue color dimmed by a factor. Used for bg tint and heat line.
fn storm_dim_color(grade_total: u16, dim: f64) -> Color {
    // Blend from cool blue at low grades to bright electric blue at high grades
    let t = (grade_total as f64 / 100.0).min(1.0);
    let r = (60.0 + t * 40.0) * dim;
    let g = (140.0 + t * 40.0) * dim;
    let b = (220.0 + t * 35.0) * dim;
    Color::Rgb(r as u8, g as u8, b as u8)
}

/// Faint storm-blue background tint scaling with grade total.
fn paint_sigil_bg(frame: &mut Frame, inner: ratatui::layout::Rect, grade_total: u16) {
    let t = (grade_total as f64 / 100.0).min(1.0);
    let dim = 0.03 + t * 0.04;
    let bg = storm_dim_color(grade_total, dim);

    let buf = frame.buffer_mut();
    for y in inner.y..inner.y + inner.height {
        for x in inner.x..inner.x + inner.width {
            if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
                cell.set_bg(bg);
            }
        }
    }
}

/// Glowing storm-blue heat line along the bottom row.
fn paint_sigil_heat_line(frame: &mut Frame, inner: ratatui::layout::Rect, grade_total: u16) {
    if inner.height == 0 {
        return;
    }
    let t = (grade_total as f64 / 100.0).min(1.0);
    let bg = storm_dim_color(grade_total, 0.08 + t * 0.10);

    let bottom_y = inner.y + inner.height - 1;
    let buf = frame.buffer_mut();
    for x in inner.x..inner.x + inner.width {
        if let Some(cell) = buf.cell_mut(Position::new(x, bottom_y)) {
            cell.set_bg(bg);
        }
    }
    if inner.height >= 3 {
        let bg2 = storm_dim_color(grade_total, 0.05 + t * 0.06);
        let row2_y = inner.y + inner.height - 2;
        for x in inner.x..inner.x + inner.width {
            if let Some(cell) = buf.cell_mut(Position::new(x, row2_y)) {
                cell.set_bg(bg2);
            }
        }
    }
}

/// Rising storm motes with size/color/trail progression.
fn paint_sigil_motes(frame: &mut Frame, inner: ratatui::layout::Rect, grade_total: u16) {
    use super::scene_fx::{current_millis, hash2d};

    let height = inner.height as usize;
    let width = inner.width as usize;
    if height == 0 || width == 0 {
        return;
    }

    // Scale 0-100 into 0.0-1.0
    let t = (grade_total as f64 / 100.0).min(1.0);

    let threshold = (90.0 - t * 78.0) as u32;
    let rise_rate = 600.0 - t * 510.0;
    let millis = current_millis() as f64;
    let rise_phase = (millis / rise_rate) as usize;
    let has_trails = t >= 0.7;

    let buf = frame.buffer_mut();
    for y in inner.y..inner.y + inner.height {
        for x in inner.x..inner.x + inner.width {
            let row = (y - inner.y) as usize;
            let col = (x - inner.x) as usize;

            let seed = hash2d(
                row.wrapping_add(rise_phase),
                col.wrapping_add(rise_phase / 4),
            );
            if !seed.is_multiple_of(threshold) {
                continue;
            }

            let Some(cell) = buf.cell_mut(Position::new(x, y)) else {
                continue;
            };
            if cell.symbol() != " " {
                continue;
            }

            let pulse = (millis * 0.003 + row as f64 * 0.8 + col as f64 * 0.6).sin();
            if pulse < 0.0 {
                continue;
            }

            let base_brightness = 80.0 + t * 100.0;
            let brightness = base_brightness + pulse * 75.0;
            let scale = (brightness / 255.0).min(1.0) * 0.35;

            // Size progression: small at low, larger at high
            let ch = if t >= 0.9 {
                match seed % 8 {
                    0 => '*',
                    1..=2 => '\u{2726}',
                    3..=4 => '\u{2022}',
                    _ => '\u{00b7}',
                }
            } else if t >= 0.7 {
                match seed % 6 {
                    0..=1 => '\u{2726}',
                    2..=3 => '\u{2022}',
                    _ => '\u{00b7}',
                }
            } else if t >= 0.4 {
                if seed.is_multiple_of(3) {
                    '\u{2022}'
                } else {
                    '\u{00b7}'
                }
            } else {
                '\u{00b7}'
            };

            // Storm-blue color
            let fg = Color::Rgb(
                (60.0 * scale) as u8,
                (180.0 * scale) as u8,
                (255.0 * scale) as u8,
            );
            cell.set_char(ch);
            cell.set_fg(fg);

            // Trail at high grades
            if has_trails && y + 1 < inner.y + inner.height {
                if let Some(trail_cell) = buf.cell_mut(Position::new(x, y + 1)) {
                    if trail_cell.symbol() == " " {
                        let trail_scale = scale * 0.4;
                        trail_cell.set_char('\u{00b7}');
                        trail_cell.set_fg(Color::Rgb(
                            (60.0 * trail_scale) as u8,
                            (180.0 * trail_scale) as u8,
                            (255.0 * trail_scale) as u8,
                        ));
                    }
                }
            }
        }
    }
}
