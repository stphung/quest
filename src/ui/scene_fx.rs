//! Shared utilities for layered ASCII scene rendering.

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    Frame,
};
use std::cell::RefCell;
use unicode_width::UnicodeWidthChar;

/// Cell in a scene render buffer.
#[derive(Clone, Copy)]
pub struct SceneCell {
    pub ch: char,
    pub fg: Color,
    pub bg: Color,
    /// True if this cell is the continuation of a wide (2-column) character
    /// in the previous cell. Skipped during rendering.
    pub wide_cont: bool,
}

impl SceneCell {
    /// Create a new cell with the given character, foreground, and background colors.
    pub fn new(ch: char, fg: Color, bg: Color) -> Self {
        Self {
            ch,
            fg,
            bg,
            wide_cont: false,
        }
    }
}

impl Default for SceneCell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: Color::Reset,
            bg: Color::Reset,
            wide_cont: false,
        }
    }
}

/// Current time in milliseconds since UNIX epoch (freezable in tests).
pub fn current_millis() -> u128 {
    super::clock::now_millis()
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
    buffer[row][col] = SceneCell {
        ch,
        fg,
        bg,
        wide_cont: false,
    };
}

/// Write a string into the scene buffer at (row, col).
/// Wide characters (emoji, CJK) occupy 2 buffer cells: the character cell
/// plus a continuation cell that is skipped during rendering.
pub fn put_text(buffer: &mut [Vec<SceneCell>], row: i32, col: i32, text: &str, fg: Color) {
    let mut pos = 0i32;
    for ch in text.chars() {
        put_cell(buffer, row, col + pos, ch, fg);
        let w = UnicodeWidthChar::width(ch).unwrap_or(1);
        if w == 2 {
            // Mark next cell as continuation of this wide character
            let cont_col = (col + pos + 1) as usize;
            let r = row as usize;
            if row >= 0 && r < buffer.len() && cont_col < buffer[r].len() {
                let bg = buffer[r][cont_col].bg;
                buffer[r][cont_col] = SceneCell {
                    ch: ' ',
                    fg,
                    bg,
                    wide_cont: true,
                };
            }
        }
        pos += w as i32;
    }
}

/// Display width of a string in terminal columns (accounts for wide characters).
pub fn display_width(text: &str) -> usize {
    text.chars()
        .map(|ch| UnicodeWidthChar::width(ch).unwrap_or(1))
        .sum()
}

/// Write a string centered horizontally in the buffer (display-width-aware).
pub fn put_text_centered(
    buffer: &mut [Vec<SceneCell>],
    row: i32,
    width: usize,
    text: &str,
    fg: Color,
) {
    let col = (width as i32 - display_width(text) as i32) / 2;
    put_text(buffer, row, col.max(0), text, fg);
}

/// Write a string centered, wrapping onto multiple rows if it exceeds `width`.
/// Wraps at word boundaries when possible. Returns the number of rows used.
pub fn put_text_centered_wrap(
    buffer: &mut [Vec<SceneCell>],
    start_row: i32,
    width: usize,
    text: &str,
    fg: Color,
) -> usize {
    if width == 0 {
        return 0;
    }
    let lines = wrap_text(text, width);
    for (i, line) in lines.iter().enumerate() {
        put_text_centered(buffer, start_row + i as i32, width, line, fg);
    }
    lines.len()
}

/// Word-wrap text to fit within `max_width` display columns.
fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0usize;

    for word in text.split_whitespace() {
        let word_width = display_width(word);
        if current_width == 0 {
            // First word on line — always take it even if it's too wide
            current_line.push_str(word);
            current_width = word_width;
        } else if current_width + 1 + word_width <= max_width {
            // Fits with a space
            current_line.push(' ');
            current_line.push_str(word);
            current_width += 1 + word_width;
        } else {
            // Doesn't fit — start a new line
            lines.push(current_line);
            current_line = word.to_string();
            current_width = word_width;
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

/// Flushes a scene buffer directly into the frame's cell buffer.
/// Continuation cells (after wide characters) are skipped so the terminal
/// renders wide chars at their correct 2-column width.
///
/// This is the per-frame flush for every layered scene (combat sprites and
/// all animated overlays), so it writes cells in a single pass instead of
/// building per-row `Span`/`String`/`Paragraph` allocations. The glyph
/// placement rules deliberately mirror the previous `Paragraph` pipeline:
/// control characters are dropped, zero-width characters render nothing
/// (neither advances the column), a wide glyph advances two columns without
/// touching the shadowed cell, and the row truncates at the first glyph that
/// no longer fits.
pub fn render_buffer(frame: &mut Frame, area: Rect, buffer: &[Vec<SceneCell>]) {
    let buf = frame.buffer_mut();
    for (row, row_data) in buffer.iter().enumerate() {
        if row as u16 >= area.height {
            break;
        }
        let y = area.y + row as u16;

        // Column offset within the area; advances only when a glyph renders,
        // matching how the old string-based path laid out glyphs.
        let mut x: u16 = 0;
        for cell in row_data.iter().take(area.width as usize) {
            // Skip continuation cells — the wide char in the previous cell
            // already occupies 2 terminal columns.
            if cell.wide_cont {
                continue;
            }
            if cell.ch.is_control() {
                continue;
            }
            let width = UnicodeWidthChar::width(cell.ch).unwrap_or(0) as u16;
            if width == 0 {
                continue;
            }
            if x + width > area.width {
                break;
            }
            if let Some(frame_cell) = buf.cell_mut((area.x + x, y)) {
                frame_cell
                    .set_char(cell.ch)
                    .set_style(Style::default().fg(cell.fg).bg(cell.bg));
            }
            x += width;
        }
    }
}

thread_local! {
    static SCENE_BUFFER: RefCell<Vec<Vec<SceneCell>>> = const { RefCell::new(Vec::new()) };
}

/// Acquire a reusable scene buffer, resized and cleared to `SceneCell::default()`.
///
/// The returned guard holds a mutable borrow on a thread-local buffer that
/// persists across frames, avoiding per-frame heap allocation.  When the
/// terminal is resized the buffer rows/columns are grown (never shrunk) so
/// that a resize only costs one allocation rather than one per frame.
///
/// The caller receives `&mut Vec<Vec<SceneCell>>` via the provided closure.
pub fn with_scene_buffer<F, R>(width: usize, height: usize, f: F) -> R
where
    F: FnOnce(&mut Vec<Vec<SceneCell>>) -> R,
{
    SCENE_BUFFER.with(|cell| {
        let mut buf = cell.borrow_mut();

        // Grow rows if needed
        if buf.len() < height {
            buf.resize_with(height, || vec![SceneCell::default(); width]);
        }

        // Ensure every row has enough columns and reset to default
        let default = SceneCell::default();
        for row in buf.iter_mut().take(height) {
            if row.len() < width {
                row.resize(width, default);
            }
            for cell in row.iter_mut().take(width) {
                *cell = default;
            }
        }

        // Truncate extra rows so downstream code sees the right dimensions
        buf.truncate(height);
        for row in buf.iter_mut() {
            row.truncate(width);
        }

        f(&mut buf)
    })
}

/// Clamps an i16 to the u8 range [0, 255].
pub fn clamp_u8(value: i16) -> u8 {
    value.clamp(0, 255) as u8
}
