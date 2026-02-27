//! Shared helpers for The Deep UI views.

use ratatui::style::Color;

use super::scene_fx::{put_cell, SceneCell};

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
