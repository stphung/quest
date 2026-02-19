//! Shared utilities for layered ASCII scene rendering.

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use std::time::{SystemTime, UNIX_EPOCH};

/// Cell in a scene render buffer.
#[derive(Clone, Copy)]
pub struct SceneCell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
}

impl Default for SceneCell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: Color::Reset,
            bg: Color::Reset,
        }
    }
}

/// Current time in milliseconds since UNIX epoch.
pub fn current_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

/// 2D deterministic hash useful for star/noise placement.
pub fn hash2d(row: usize, col: usize) -> u32 {
    let seed = (row as u32)
        .wrapping_mul(1664525)
        .wrapping_add((col as u32).wrapping_mul(1013904223));
    seed ^ (seed >> 13)
}

pub fn lerp_channel(start: u8, end: u8, t: f64) -> u8 {
    let t = t.clamp(0.0, 1.0);
    (start as f64 + (end as f64 - start as f64) * t).round() as u8
}

pub fn lerp_rgb(start: (u8, u8, u8), end: (u8, u8, u8), t: f64) -> (u8, u8, u8) {
    (
        lerp_channel(start.0, end.0, t),
        lerp_channel(start.1, end.1, t),
        lerp_channel(start.2, end.2, t),
    )
}

/// Writes a cell while preserving its current background color.
pub fn put_cell(buffer: &mut [Vec<SceneCell>], row: i32, col: i32, ch: char, fg: Color) {
    if row < 0 || col < 0 {
        return;
    }
    let row = row as usize;
    let col = col as usize;
    if row >= buffer.len() || col >= buffer[row].len() {
        return;
    }

    let bg = buffer[row][col].bg;
    buffer[row][col] = SceneCell { ch, fg, bg };
}

/// Draws a simple line with `/`, `\` or `|` glyphs based on slope.
pub fn draw_line(
    buffer: &mut [Vec<SceneCell>],
    mut x0: i32,
    mut y0: i32,
    x1: i32,
    y1: i32,
    fg: Color,
) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    let line_char = if (x1 - x0).abs() <= 1 {
        '|'
    } else if x1 > x0 {
        '\\'
    } else {
        '/'
    };

    loop {
        put_cell(buffer, y0, x0, line_char, fg);
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

/// Flushes a scene buffer to the frame with run-length style batching by color.
pub fn render_buffer(frame: &mut Frame, area: Rect, buffer: &[Vec<SceneCell>]) {
    for (row, row_data) in buffer.iter().enumerate() {
        if row as u16 >= area.height {
            break;
        }

        let mut spans = Vec::new();
        let mut current_fg = Color::Reset;
        let mut current_bg = Color::Reset;
        let mut current_text = String::new();

        for cell in row_data.iter().take(area.width as usize) {
            if (cell.fg != current_fg || cell.bg != current_bg) && !current_text.is_empty() {
                spans.push(Span::styled(
                    std::mem::take(&mut current_text),
                    Style::default().fg(current_fg).bg(current_bg),
                ));
            }

            current_fg = cell.fg;
            current_bg = cell.bg;
            current_text.push(cell.ch);
        }

        if !current_text.is_empty() {
            spans.push(Span::styled(
                current_text,
                Style::default().fg(current_fg).bg(current_bg),
            ));
        }

        let row_area = Rect::new(area.x, area.y + row as u16, area.width, 1);
        frame.render_widget(Paragraph::new(Line::from(spans)), row_area);
    }
}

/// Clamps an i16 to the u8 range [0, 255].
pub fn clamp_u8(value: i16) -> u8 {
    value.clamp(0, 255) as u8
}
