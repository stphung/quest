//! Shared helpers for The Deep UI views.

use ratatui::style::Color;

use super::scene_fx::{put_cell, put_text, SceneCell};

/// Format seconds as a compact duration string (e.g., "2h", "30m", "1h 30m").
/// Always shows at least "1m" for non-zero durations.
pub(super) fn format_hours(secs: u64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    if h == 0 {
        format!("{}m", m.max(1))
    } else if m == 0 {
        format!("{}h", h)
    } else {
        format!("{}h {}m", h, m)
    }
}

/// Render a block-character progress bar (`████░░░░`).
/// `ratio` is 0.0..=1.0. Bar fits within `width` chars.
pub(super) fn render_progress_bar(
    buffer: &mut [Vec<SceneCell>],
    row: i32,
    col: i32,
    width: usize,
    ratio: f64,
    filled_color: Color,
) {
    if width == 0 {
        return;
    }
    let filled = ((ratio * width as f64).round() as usize).min(width);
    for i in 0..filled {
        put_cell(buffer, row, col + i as i32, '\u{2588}', filled_color);
    }
    for i in filled..width {
        put_cell(
            buffer,
            row,
            col + i as i32,
            '\u{2591}',
            Color::Rgb(30, 40, 60),
        );
    }
}

/// Truncate a string to at most `max_chars` terminal characters.
pub(super) fn truncate_text(text: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    text.chars().take(max_chars).collect()
}

/// Draw a reusable Deep-themed card with optional title.
///
/// Coordinates are inclusive and expect screen-space positions.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_deep_card(
    buffer: &mut [Vec<SceneCell>],
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    border_color: Color,
    fill_color: Color,
    title: Option<&str>,
) {
    if left < 0 || top < 0 || right <= left + 1 || bottom <= top + 1 {
        return;
    }
    if buffer.is_empty() {
        return;
    }

    // Fill interior
    for row in (top + 1)..bottom {
        if row < 0 || row as usize >= buffer.len() {
            continue;
        }
        for col in (left + 1)..right {
            if col < 0 || col as usize >= buffer[row as usize].len() {
                continue;
            }
            buffer[row as usize][col as usize].bg = fill_color;
        }
    }

    // Border (style-aware)
    let glyphs = super::panel_border_chars();
    for col in (left + 1)..right {
        put_cell(buffer, top, col, glyphs.h, border_color);
        put_cell(buffer, bottom, col, glyphs.h, border_color);
    }
    for row in (top + 1)..bottom {
        put_cell(buffer, row, left, glyphs.v, border_color);
        put_cell(buffer, row, right, glyphs.v, border_color);
    }
    put_cell(buffer, top, left, glyphs.tl, border_color);
    put_cell(buffer, top, right, glyphs.tr, border_color);
    put_cell(buffer, bottom, left, glyphs.bl, border_color);
    put_cell(buffer, bottom, right, glyphs.br, border_color);

    // Optional title, clipped to interior width
    if let Some(title_text) = title {
        let title_inner = (right - left - 4).max(1) as usize;
        let clipped = truncate_text(title_text, title_inner);
        let label = format!(" {} ", clipped);
        put_text(buffer, top, left + 2, &label, border_color);
    }
}
