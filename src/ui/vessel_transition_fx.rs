//! Animated presentation for the 5-beat launch transition
//! (`vessel::transition`): "Ignition" — text breathes with a pulsing glow;
//! outward sonar-like rings emanate from screen center; the last two beats
//! add warp-speed streaks radiating from center, climaxing in a white
//! ignition flash that dissolves into the Void, where the Vessel's ship art
//! resolves out of the starfield.
//!
//! Reads `elapsed_ms` (time since the current beat began) and nothing else
//! that isn't already deterministic-per-frame, so it renders identically
//! given the same `(beat, elapsed_ms)` — a frozen UI clock and a fixed
//! `beat_started_ms` are enough to make it snapshot-testable.

use std::f64::consts::PI;

use ratatui::{layout::Rect, style::Color, Frame};

use crate::vessel::transition::{self, BEAT_COUNT};

use super::scene_fx::{
    display_width, hash2d, lerp_rgb, put_cell, put_text, render_buffer, SceneCell,
};
use super::vessel_scene::{GOLD, VESSEL_VIOLET_DIM};

/// Base color for a beat's text — beats darken/build/brighten across the
/// sequence (Farewell -> Unweaving -> Construction -> Launch -> Void).
fn beat_color(beat: u8) -> Color {
    match beat {
        1 => Color::DarkGray,
        2 => VESSEL_VIOLET_DIM,
        3 => super::vessel_scene::VESSEL_VIOLET,
        4 => GOLD,
        _ => Color::White,
    }
}

fn beat_rgb(beat: u8) -> (u8, u8, u8) {
    match beat_color(beat) {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::DarkGray => (110, 110, 110),
        Color::White => (255, 255, 255),
        _ => (200, 200, 200),
    }
}

/// Bottom "[Enter]" prompt and the top-right "N / 5 — Heading" marker.
fn draw_chrome(buf: &mut [Vec<SceneCell>], w: usize, h: usize, beat: u8) {
    if h >= 2 {
        put_text(
            buf,
            (h - 2) as i32,
            ((w as i32) - 7) / 2,
            "[Enter]",
            Color::DarkGray,
        );
    }
    let content = transition::beat(beat);
    let marker = format!(" {beat} / {BEAT_COUNT} \u{2014} {} ", content.heading);
    let marker_w = display_width(&marker);
    if w > marker_w {
        put_text(buf, 0, (w - marker_w - 1) as i32, &marker, Color::DarkGray);
    }
}

fn new_buffer(w: usize, h: usize) -> Vec<Vec<SceneCell>> {
    vec![vec![SceneCell::default(); w]; h]
}

/// Per-line starting column so a line of `text` sits centered in `width`.
fn centered_col(width: usize, text: &str) -> i32 {
    ((width as i32) - display_width(text) as i32) / 2
}

fn top_row_for(h: usize, content_lines: usize) -> i32 {
    let content_height = content_lines + 4;
    (h.saturating_sub(content_height) / 2) as i32
}

const BREATH_CYCLE_MS: f64 = 1400.0;
const STREAK_COUNT: usize = 28;
const FLASH_MS: f64 = 250.0;

pub fn render(frame: &mut Frame, area: Rect, beat: u8, elapsed_ms: u128) {
    let w = area.width as usize;
    let h = area.height as usize;
    if w == 0 || h == 0 {
        return;
    }
    let mut buf = new_buffer(w, h);

    let phase = (elapsed_ms as f64 % BREATH_CYCLE_MS) / BREATH_CYCLE_MS;
    let breathe = 0.5 + 0.5 * (2.0 * PI * phase).sin();

    render_sonar_rings(&mut buf, w, h, beat, elapsed_ms);

    if beat >= 4 {
        render_warp_streaks(&mut buf, w, h, beat, elapsed_ms);
    }

    let content = transition::beat(beat);
    let top = top_row_for(h, content.lines.len());
    let base = beat_rgb(beat);
    let bright = lerp_rgb(base, (255, 255, 255), 0.55);
    let text_rgb = lerp_rgb(base, bright, breathe);
    let text_color = Color::Rgb(text_rgb.0, text_rgb.1, text_rgb.2);
    if beat == BEAT_COUNT {
        render_void_ship(&mut buf, w, top, content.lines, text_color);
    } else {
        for (li, line) in content.lines.iter().enumerate() {
            put_cell_line(&mut buf, top + li as i32, w, line, text_color);
        }
    }

    if beat == BEAT_COUNT && (elapsed_ms as f64) < FLASH_MS {
        let t = (elapsed_ms as f64 / FLASH_MS).clamp(0.0, 1.0);
        let fade = lerp_rgb((255, 255, 255), (0, 0, 0), t);
        for row in buf.iter_mut() {
            for cell in row.iter_mut() {
                cell.bg = Color::Rgb(fade.0, fade.1, fade.2);
                if fade.0 > 140 {
                    cell.fg = Color::Black;
                }
            }
        }
    }

    draw_chrome(&mut buf, w, h, beat);
    render_buffer(frame, area, &buf);
}

fn put_cell_line(buf: &mut [Vec<SceneCell>], row: i32, w: usize, line: &str, color: Color) {
    let col0 = centered_col(w, line);
    put_text(buf, row, col0, line, color);
}

/// Draws the Void beat's authored ship art as one aligned block rather than
/// independently centering each line: the art's lines vary in raw character
/// count (leading spaces are part of its hand-authored layout), so centering
/// each one by its own width — as `put_cell_line` does for plain sentences —
/// drifts the hull/sail/mast glyphs sideways relative to each other and
/// breaks the silhouette. A single shared column, sized to the widest line,
/// keeps every row's indentation relative to the others intact. Star glyphs
/// twinkle independently of the ship glow; space characters are skipped so
/// the sonar-ring backdrop shows through the ship's negative space.
fn render_void_ship(
    buf: &mut [Vec<SceneCell>],
    w: usize,
    top: i32,
    lines: &[&str],
    ship_color: Color,
) {
    let block_width = lines.iter().map(|l| display_width(l)).max().unwrap_or(0);
    let col0 = ((w as i32) - block_width as i32) / 2;
    let millis = super::clock::now_millis() as f64;

    for (li, line) in lines.iter().enumerate() {
        let row = top + li as i32;
        for (ci, ch) in line.chars().enumerate() {
            let col = col0 + ci as i32;
            if ch == ' ' {
                continue;
            }
            if ch == '\u{00b7}' {
                let phase = (hash2d(li, ci) % 628) as f64 / 100.0;
                let s = 0.5 + 0.5 * (millis / 400.0 + phase).sin();
                let rgb = lerp_rgb((70, 70, 80), (220, 220, 230), s);
                put_cell(buf, row, col, ch, Color::Rgb(rgb.0, rgb.1, rgb.2));
            } else {
                put_cell(buf, row, col, ch, ship_color);
            }
        }
    }
}

fn render_sonar_rings(buf: &mut [Vec<SceneCell>], w: usize, h: usize, beat: u8, elapsed_ms: u128) {
    let cx = w as f64 / 2.0;
    let cy = h as f64 / 2.0;
    let max_dist = (cx.max(cy)) * 1.5;
    // Rings expand outward continuously and wrap, speeding up with each beat.
    let speed = 18.0 + beat as f64 * 6.0;
    let ring_front = (elapsed_ms as f64 / speed) % (max_dist + 30.0);
    let base = beat_rgb(beat);

    for row in 0..h {
        for col in 0..w {
            let dy = (row as f64 - cy) * 2.0;
            let dx = col as f64 - cx;
            let dist = (dx * dx + dy * dy).sqrt();
            let delta = (dist - ring_front).abs();
            if delta < 1.3 {
                let strength = 1.0 - (delta / 1.3);
                let rgb = lerp_rgb((15, 15, 20), base, strength * 0.8);
                put_cell(
                    buf,
                    row as i32,
                    col as i32,
                    '\u{00b7}',
                    Color::Rgb(rgb.0, rgb.1, rgb.2),
                );
            } else if hash2d(row, col).is_multiple_of(211) {
                put_cell(
                    buf,
                    row as i32,
                    col as i32,
                    '\u{00b7}',
                    Color::Rgb(30, 28, 40),
                );
            }
        }
    }
}

fn render_warp_streaks(buf: &mut [Vec<SceneCell>], w: usize, h: usize, beat: u8, elapsed_ms: u128) {
    let cx = w as f64 / 2.0;
    let cy = h as f64 / 2.0;
    let ramp = elapsed_ms as f64;
    let ramp_chars = ['\u{00b7}', '.', '-', '=', '\u{2261}'];

    for i in 0..STREAK_COUNT {
        let angle = (i as f64 / STREAK_COUNT as f64) * 2.0 * PI + (beat as f64 * 0.3);
        let speed = 0.06 + (hash2d(i, beat as usize) % 100) as f64 / 100.0 * 0.10;
        let length = (ramp * speed).min(cx.max(cy) * 1.4);
        let steps = length as usize;
        for s in 0..steps {
            let dist = s as f64;
            let col = cx + angle.cos() * dist;
            let row = cy + angle.sin() * dist * 0.5;
            let bucket = ((dist / length.max(1.0)) * (ramp_chars.len() as f64 - 1.0)) as usize;
            let ch = ramp_chars[bucket.min(ramp_chars.len() - 1)];
            let brightness = (dist / length.max(1.0)).clamp(0.0, 1.0);
            let rgb = lerp_rgb((40, 35, 20), (255, 235, 160), brightness);
            put_cell(
                buf,
                row.round() as i32,
                col.round() as i32,
                ch,
                Color::Rgb(rgb.0, rgb.1, rgb.2),
            );
        }
    }
}
