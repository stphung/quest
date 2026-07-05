//! Animated presentation shells for the 5-beat launch transition
//! (`vessel::transition`). Three visually distinct treatments of the same
//! authored beats, selected by `vessel::transition::variant()` — a design
//! comparison harness, not three permanent game modes. Once one is chosen
//! the others should be deleted along with the variant switch.
//!
//! - **Variant 1 "Unweaving"**: text decodes from scrambled runes into
//!   place, cascading left-to-right; a field of drifting thread-motes
//!   thickens beat over beat; the final Void beat's ship glyphs fly in from
//!   scattered points and settle.
//! - **Variant 2 "Ignition"**: text breathes with a pulsing glow; outward
//!   sonar-like rings emanate from screen center; the last two beats add
//!   warp-speed streaks radiating from center, climaxing in a white ignition
//!   flash that dissolves into the Void.
//! - **Variant 3 "Genesis Montage"**: each beat's text slides in from a
//!   different edge with a fading motion trail (a cut's worth of "camera"
//!   per beat); the Construction beat additionally raises a hull-building
//!   progress bar.
//!
//! All three read `elapsed_ms` (time since the current beat began) and nothing
//! else that isn't already deterministic-per-frame, so they render identically
//! given the same `(beat, elapsed_ms)` — a frozen UI clock and a fixed
//! `beat_started_ms` are enough to make them snapshot-testable.

use std::f64::consts::PI;

use ratatui::{layout::Rect, style::Color, Frame};

use crate::vessel::transition::{self, BEAT_COUNT};

use super::scene_fx::{
    display_width, hash2d, lerp_rgb, put_cell, put_text, render_buffer, SceneCell,
};
use super::vessel_scene::{GOLD, VESSEL_VIOLET, VESSEL_VIOLET_DIM};

/// Base color for a beat's text — beats darken/build/brighten across the
/// sequence (Farewell -> Unweaving -> Construction -> Launch -> Void).
fn beat_color(beat: u8) -> Color {
    match beat {
        1 => Color::DarkGray,
        2 => VESSEL_VIOLET_DIM,
        3 => VESSEL_VIOLET,
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

pub fn render(frame: &mut Frame, area: Rect, beat: u8, elapsed_ms: u128) {
    match transition::variant() {
        2 => variant2(frame, area, beat, elapsed_ms),
        3 => variant3(frame, area, beat, elapsed_ms),
        _ => variant1(frame, area, beat, elapsed_ms),
    }
}

/// Bottom "[Enter]" prompt and the top-right "N / 5 — Heading" marker, common
/// chrome across all three variants.
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

fn ease_out_cubic(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

// ─────────────────────────── Variant 1: Unweaving ───────────────────────────

const RUNES: &[char] = &[
    '\u{16A0}', '\u{16A2}', '\u{16A6}', '\u{16A8}', '\u{16B1}', '\u{16B2}', '\u{16B7}', '\u{16B9}',
    '\u{16BA}', '\u{16BE}', '\u{16C1}', '\u{16C3}', '\u{16C7}', '\u{16C8}', '\u{16CA}', '\u{16CF}',
    '\u{16D2}', '\u{16D6}', '\u{16D7}', '\u{16DA}', '\u{16DC}', '\u{16DE}', '\u{16DF}', '\u{16E6}',
];

const TEXT_REVEAL_MS: f64 = 700.0;
const SHIP_FLY_MS: f64 = 1200.0;

fn variant1(frame: &mut Frame, area: Rect, beat: u8, elapsed_ms: u128) {
    let w = area.width as usize;
    let h = area.height as usize;
    if w == 0 || h == 0 {
        return;
    }
    let mut buf = new_buffer(w, h);
    let millis = super::clock::now_millis() as f64;

    // Thread-mote backdrop: sparse at Farewell, dense by the Void — the
    // Loom's fabric coming undone.
    let density_mod = match beat {
        1 => 160,
        2 => 120,
        3 => 90,
        4 => 65,
        _ => 45,
    };
    let mote_chars = ['\u{00b7}', '\u{2500}', '\u{2502}', '\u{2504}'];
    for row in 0..h {
        for col in 0..w {
            let seed = hash2d(row, col + beat as usize * 4096);
            if !seed.is_multiple_of(density_mod) {
                continue;
            }
            let phase = (seed % 628) as f64 / 100.0;
            let t = (0.5 + 0.5 * (millis / 320.0 + phase).sin()).clamp(0.0, 1.0);
            let rgb = lerp_rgb((25, 22, 38), (95, 78, 130), t);
            let ch = mote_chars[(seed as usize / 7) % mote_chars.len()];
            put_cell(
                &mut buf,
                row as i32,
                col as i32,
                ch,
                Color::Rgb(rgb.0, rgb.1, rgb.2),
            );
        }
    }

    let content = transition::beat(beat);
    let top = top_row_for(h, content.lines.len());

    if beat == BEAT_COUNT {
        render_reformation(&mut buf, w, top, content.lines, elapsed_ms);
    } else {
        render_decode_text(&mut buf, w, top, content.lines, beat, elapsed_ms);
    }

    draw_chrome(&mut buf, w, h, beat);
    render_buffer(frame, area, &buf);
}

fn render_decode_text(
    buf: &mut [Vec<SceneCell>],
    w: usize,
    top: i32,
    lines: &[&str],
    beat: u8,
    elapsed_ms: u128,
) {
    let total_chars: usize = lines
        .iter()
        .map(|l| l.chars().count())
        .sum::<usize>()
        .max(1);
    let color = beat_color(beat);
    let dim_color = Color::Rgb(90, 80, 110);
    let scramble_tick = (elapsed_ms / 70) as usize;

    let mut seen = 0usize;
    for (li, line) in lines.iter().enumerate() {
        let row = top + li as i32;
        let col0 = centered_col(w, line);
        for (ci, ch) in line.chars().enumerate() {
            let reveal_at = (seen as f64 / total_chars as f64) * TEXT_REVEAL_MS;
            seen += 1;
            let col = col0 + ci as i32;
            if ch == ' ' {
                continue;
            }
            if (elapsed_ms as f64) >= reveal_at {
                put_cell(buf, row, col, ch, color);
            } else {
                let glyph = RUNES[(hash2d(li * 97 + ci, scramble_tick) as usize) % RUNES.len()];
                put_cell(buf, row, col, glyph, dim_color);
            }
        }
    }
}

fn render_reformation(
    buf: &mut [Vec<SceneCell>],
    w: usize,
    top: i32,
    lines: &[&str],
    elapsed_ms: u128,
) {
    let t = ease_out_cubic(elapsed_ms as f64 / SHIP_FLY_MS);
    for (li, line) in lines.iter().enumerate() {
        let row = top + li as i32;
        let col0 = centered_col(w, line);
        for (ci, ch) in line.chars().enumerate() {
            let col = col0 + ci as i32;
            if ch == ' ' {
                continue;
            }
            if ch == '\u{00b7}' {
                // Ambient stars twinkle immediately, no fly-in.
                let phase = (hash2d(li, ci) % 628) as f64 / 100.0;
                let millis = super::clock::now_millis() as f64;
                let s = 0.5 + 0.5 * (millis / 400.0 + phase).sin();
                let rgb = lerp_rgb((70, 70, 80), (220, 220, 230), s);
                put_cell(buf, row, col, ch, Color::Rgb(rgb.0, rgb.1, rgb.2));
                continue;
            }
            // Ship glyph: flies in from a deterministic scattered offset.
            let seed = hash2d(li * 131 + ci, 7);
            let dx = ((seed % 29) as f64 - 14.0) * 1.4;
            let dy = ((seed / 29 % 13) as f64 - 6.0) * 1.4;
            let cur_col = col as f64 + dx * (1.0 - t);
            let cur_row = row as f64 + dy * (1.0 - t);
            let rgb = lerp_rgb((70, 65, 90), (230, 215, 245), t);
            put_cell(
                buf,
                cur_row.round() as i32,
                cur_col.round() as i32,
                ch,
                Color::Rgb(rgb.0, rgb.1, rgb.2),
            );
        }
    }
}

// ─────────────────────────── Variant 2: Ignition ───────────────────────────

const BREATH_CYCLE_MS: f64 = 1400.0;
const STREAK_COUNT: usize = 28;
const FLASH_MS: f64 = 250.0;

fn variant2(frame: &mut Frame, area: Rect, beat: u8, elapsed_ms: u128) {
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
    for (li, line) in content.lines.iter().enumerate() {
        put_cell_line(
            &mut buf,
            top + li as i32,
            w,
            line,
            Color::Rgb(text_rgb.0, text_rgb.1, text_rgb.2),
        );
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

// ─────────────────────── Variant 3: Genesis Montage ────────────────────────

const SLIDE_MS: f64 = 500.0;

#[derive(Clone, Copy)]
enum SlideDir {
    Top,
    Left,
    Right,
    Bottom,
}

fn slide_dir(beat: u8) -> SlideDir {
    match beat {
        1 => SlideDir::Top,
        2 => SlideDir::Left,
        3 => SlideDir::Right,
        _ => SlideDir::Bottom,
    }
}

fn variant3(frame: &mut Frame, area: Rect, beat: u8, elapsed_ms: u128) {
    let w = area.width as usize;
    let h = area.height as usize;
    if w == 0 || h == 0 {
        return;
    }
    let mut buf = new_buffer(w, h);

    let content = transition::beat(beat);
    let top = top_row_for(h, content.lines.len());
    let dir = slide_dir(beat);
    let color = beat_color(beat);

    let t = ease_out_cubic(elapsed_ms as f64 / SLIDE_MS);
    let remaining = 1.0 - t;
    let max_offset = 18.0;

    // Trailing ghosts first (further along the motion path, dimmer), then
    // the settled line on top so it always reads clearly once it arrives.
    if remaining > 0.01 {
        for &frac in &[0.85, 0.55] {
            let ghost_off = remaining * max_offset * frac;
            draw_slid_lines(
                &mut buf,
                w,
                top,
                content.lines,
                dir,
                ghost_off,
                dim(color, 0.28),
            );
        }
    }
    let offset = remaining * max_offset;
    draw_slid_lines(&mut buf, w, top, content.lines, dir, offset, color);

    if beat == 3 {
        render_hull_bar(&mut buf, w, h, elapsed_ms);
    }

    draw_chrome(&mut buf, w, h, beat);
    render_buffer(frame, area, &buf);
}

fn dim(color: Color, factor: f64) -> Color {
    let rgb = match color {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::DarkGray => (110, 110, 110),
        Color::White => (255, 255, 255),
        _ => (180, 180, 180),
    };
    let out = lerp_rgb((10, 9, 14), rgb, factor);
    Color::Rgb(out.0, out.1, out.2)
}

fn draw_slid_lines(
    buf: &mut [Vec<SceneCell>],
    w: usize,
    top: i32,
    lines: &[&str],
    dir: SlideDir,
    offset: f64,
    color: Color,
) {
    for (li, line) in lines.iter().enumerate() {
        let base_row = top + li as i32;
        let base_col = centered_col(w, line);
        let (row, col) = match dir {
            SlideDir::Top => (base_row - offset.round() as i32, base_col),
            SlideDir::Bottom => (base_row + offset.round() as i32, base_col),
            SlideDir::Left => (base_row, base_col - (offset * 2.0).round() as i32),
            SlideDir::Right => (base_row, base_col + (offset * 2.0).round() as i32),
        };
        put_text(buf, row, col, line, color);
    }
}

fn render_hull_bar(buf: &mut [Vec<SceneCell>], w: usize, h: usize, elapsed_ms: u128) {
    const BAR_MS: f64 = 1400.0;
    let ratio = (elapsed_ms as f64 / BAR_MS).clamp(0.0, 1.0);
    let bar_row = (h as i32) - 4;
    let bar_width = w.saturating_sub(20).max(4);
    let start_col = ((w - bar_width) / 2) as i32;
    let filled = (ratio * bar_width as f64).round() as usize;

    put_text(
        buf,
        bar_row - 1,
        centered_col(w, "raising the hull"),
        "raising the hull",
        Color::DarkGray,
    );
    for i in 0..bar_width {
        let ch = if i < filled { '\u{2588}' } else { '\u{2591}' };
        let color = if i < filled {
            VESSEL_VIOLET
        } else {
            Color::Rgb(50, 42, 65)
        };
        put_cell(buf, bar_row, start_col + i as i32, ch, color);
    }
}
