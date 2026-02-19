//! Soulforge animation effects: hammering, success, failure rendering.

use crate::enhancement::{enhancement_multiplier, EnhancementProgress, SoulforgeUiState};
use ratatui::style::Color;

use super::scene_fx::{current_millis, hash2d, lerp_rgb, put_cell, put_text, SceneCell};
use super::soulforge_slots::SLOT_ORDER;

/// Write a string centered horizontally in the buffer.
fn put_text_centered(buffer: &mut [Vec<SceneCell>], row: i32, width: usize, text: &str, fg: Color) {
    let col = (width as i32 - text.chars().count() as i32) / 2;
    put_text(buffer, row, col, text, fg);
}

/// Render spark particles that spray from a point.
fn render_sparks(
    buffer: &mut [Vec<SceneCell>],
    center_row: i32,
    center_col: i32,
    strike_tick: u8,
    current_tick: u8,
    spark_count: usize,
) {
    let age = current_tick.saturating_sub(strike_tick);
    if age > 10 {
        return;
    }

    let spark_chars: &[char] = &['\u{2726}', '\u{00b7}', '*', '\u{2727}'];
    let t = age as f64;

    for i in 0..spark_count {
        let seed = hash2d(strike_tick as usize, i);
        let ch = spark_chars[(seed as usize) % spark_chars.len()];

        let angle = 0.52 + (seed % 1000) as f64 / 1000.0 * 2.1;
        let speed = 1.5 + (hash2d(i, strike_tick as usize) % 100) as f64 / 100.0 * 2.0;

        let vx = angle.cos() * speed;
        let vy = angle.sin() * speed;
        let gravity = 0.15;

        let col = center_col as f64 + vx * t;
        let row = center_row as f64 - vy * t + 0.5 * gravity * t * t;

        let color_t = age as f64 / 10.0;
        let rgb = if color_t < 0.3 {
            lerp_rgb((255, 255, 200), (255, 200, 60), color_t / 0.3)
        } else {
            lerp_rgb((255, 200, 60), (120, 30, 5), (color_t - 0.3) / 0.7)
        };

        put_cell(
            buffer,
            row.round() as i32,
            col.round() as i32,
            ch,
            Color::Rgb(rgb.0, rgb.1, rgb.2),
        );
    }
}

/// Render the hammering animation content into a scene buffer.
pub(super) fn render_hammering_content(
    buffer: &mut [Vec<SceneCell>],
    soulforge_ui: &SoulforgeUiState,
    enhancement: &EnhancementProgress,
) {
    let h = buffer.len();
    let w = if h > 0 { buffer[0].len() } else { return };

    let tick = soulforge_ui.animation_tick;
    let millis = current_millis();

    let is_strike = matches!(tick, 14..=16 | 30..=32 | 46..=48);
    let is_strike_start = matches!(tick, 14 | 30 | 46);

    let hammer_raised = [
        "       ___  ",
        "      |   | ",
        "      |___|/",
        "        |   ",
        "        |   ",
    ];
    let hammer_strike = [
        "            ",
        "            ",
        "   ___      ",
        "  |   |___  ",
        "  |___|/    ",
    ];

    let anvil = [
        "    _________    ",
        "   /         \\   ",
        "  /___________\\  ",
        "      |   |      ",
        "   ___|   |___   ",
        "  |___________|  ",
    ];

    let total_art_height = 12;
    let art_top = if h > total_art_height + 4 {
        (h - total_art_height - 4) / 2
    } else {
        0
    };

    let anvil_width = 17;
    let hammer_width = 12;
    let art_center_col = (w as i32) / 2;
    let anvil_col = art_center_col - anvil_width / 2;
    let hammer_col = art_center_col - hammer_width / 2;

    // Hammer afterimage
    if is_strike_start {
        let dim_color = Color::Rgb(50, 40, 35);
        for (i, line) in hammer_raised.iter().enumerate() {
            let row = art_top as i32 + i as i32;
            put_text(buffer, row, hammer_col, line, dim_color);
        }
    }

    // Render hammer
    let hammer = if is_strike {
        &hammer_strike
    } else {
        &hammer_raised
    };

    let hammer_fg = if is_strike {
        Color::White
    } else {
        Color::Rgb(160, 150, 140)
    };

    for (i, line) in hammer.iter().enumerate() {
        let row = art_top as i32 + i as i32;
        put_text(buffer, row, hammer_col, line, hammer_fg);
    }

    // Anvil glow
    let anvil_top = art_top + hammer_raised.len() + 1;
    let anvil_base_rgb = (90, 80, 70);
    let anvil_hot_rgb = (200, 140, 40);

    let strike_ticks: [u8; 3] = [14, 30, 46];
    let mut closest_strike_age: u8 = 255;
    for &st in &strike_ticks {
        if tick >= st {
            let age = tick - st;
            if age < closest_strike_age {
                closest_strike_age = age;
            }
        }
    }
    let glow_t = if closest_strike_age <= 8 {
        1.0 - closest_strike_age as f64 / 8.0
    } else {
        0.0
    };
    let glow_t = glow_t.max(0.1);
    let anvil_rgb = lerp_rgb(anvil_base_rgb, anvil_hot_rgb, glow_t);
    let anvil_fg = Color::Rgb(anvil_rgb.0, anvil_rgb.1, anvil_rgb.2);

    for (i, line) in anvil.iter().enumerate() {
        let row = anvil_top as i32 + i as i32;
        put_text(buffer, row, anvil_col, line, anvil_fg);
    }

    // Spark shower
    let spark_center_row = anvil_top as i32;
    let spark_center_col = art_center_col;
    for &st in &strike_ticks {
        if tick >= st && tick.saturating_sub(st) <= 10 {
            render_sparks(buffer, spark_center_row, spark_center_col, st, tick, 10);
        }
    }

    // Progress bar with pulsing fill color
    let bar_row = (anvil_top + anvil.len() + 1) as i32;
    let bar_width = w.saturating_sub(6);
    let progress = tick as f64 / 50.0;
    let fill_exact = progress * bar_width as f64;
    let full_cells = fill_exact as usize;
    let fraction = fill_exact - full_cells as f64;

    let blocks: &[char] = &[
        '\u{258f}', '\u{258e}', '\u{258d}', '\u{258c}', '\u{258b}', '\u{258a}', '\u{2589}',
        '\u{2588}',
    ];

    let pulse_t = ((millis as f64 / 200.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let fill_rgb = lerp_rgb((200, 120, 20), (255, 255, 80), pulse_t);
    let fill_fg = Color::Rgb(fill_rgb.0, fill_rgb.1, fill_rgb.2);
    let bracket_fg = Color::DarkGray;
    let bar_start_col = 3i32;

    put_text(buffer, bar_row, bar_start_col - 1, "[", bracket_fg);
    for i in 0..bar_width {
        let col = bar_start_col + i as i32;
        if i < full_cells {
            put_cell(buffer, bar_row, col, '\u{2588}', fill_fg);
        } else if i == full_cells {
            let partial_idx = (fraction * 8.0) as usize;
            if partial_idx > 0 {
                put_cell(buffer, bar_row, col, blocks[partial_idx - 1], fill_fg);
            } else {
                put_cell(buffer, bar_row, col, ' ', bracket_fg);
            }
        } else {
            put_cell(buffer, bar_row, col, ' ', bracket_fg);
        }
    }
    put_text(
        buffer,
        bar_row,
        bar_start_col + bar_width as i32,
        "]",
        bracket_fg,
    );

    // Item label centered below bar
    let slot_index = soulforge_ui.selected_slot;
    let slot = SLOT_ORDER[slot_index];
    let current_level = enhancement.level(slot_index);
    let item_display = format!("{} +{}", slot.name(), current_level);
    let label_row = bar_row + 1;
    put_text_centered(buffer, label_row, w, &item_display, Color::White);
}

/// Render the success animation content into a scene buffer.
pub(super) fn render_success_content(
    buffer: &mut [Vec<SceneCell>],
    soulforge_ui: &SoulforgeUiState,
) {
    let h = buffer.len();
    let w = if h > 0 { buffer[0].len() } else { return };

    let tick = soulforge_ui.animation_tick;
    let millis = current_millis();

    // Sparkle twinkle
    let sparkle_chars: &[char] = &['\u{2726}', '\u{2727}', '*'];
    for i in 0..15 {
        let phase_seed = hash2d(i, tick as usize / 3);
        let row = (phase_seed as usize) % h;
        let col = (hash2d(i + 100, tick as usize / 3) as usize) % w;
        let ch = sparkle_chars[(hash2d(i, 0) as usize) % sparkle_chars.len()];

        let brightness =
            ((millis as f64 / 300.0 + i as f64 * 1.7).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
        let rgb = lerp_rgb((80, 60, 20), (255, 230, 100), brightness);
        put_cell(
            buffer,
            row as i32,
            col as i32,
            ch,
            Color::Rgb(rgb.0, rgb.1, rgb.2),
        );
    }

    // Display centered content
    let result = soulforge_ui.last_result.as_ref().unwrap();
    let slot = SLOT_ORDER[result.slot_index];
    let item_name = slot.name();
    let bonus = enhancement_multiplier(result.new_level);
    let bonus_pct = (bonus - 1.0) * 100.0;

    let sparkle_border: String = (0..w)
        .map(|i| {
            if i % 2 == 0 {
                let idx = (i / 2 + tick as usize) % sparkle_chars.len();
                sparkle_chars[idx]
            } else {
                ' '
            }
        })
        .collect();

    let content_height = 9usize;
    let top = if h > content_height {
        (h - content_height) / 2
    } else {
        0
    };

    put_text_centered(buffer, top as i32, w, &sparkle_border, Color::Yellow);

    let title_color = if tick % 4 < 2 {
        Color::Yellow
    } else {
        Color::Rgb(255, 215, 0)
    };
    put_text_centered(buffer, (top + 2) as i32, w, "SUCCESS!", title_color);

    let item_line = format!("{} is now +{}!", item_name, result.new_level);
    put_text_centered(buffer, (top + 4) as i32, w, &item_line, Color::Green);

    let power_line = format!("+{:.1}% Power", bonus_pct);
    put_text_centered(buffer, (top + 5) as i32, w, &power_line, Color::Yellow);

    put_text_centered(buffer, (top + 7) as i32, w, &sparkle_border, Color::Yellow);

    put_text_centered(
        buffer,
        (top + 8) as i32,
        w,
        "Press any key to continue",
        Color::DarkGray,
    );
}

/// Render the failure animation content into a scene buffer.
pub(super) fn render_failure_content(
    buffer: &mut [Vec<SceneCell>],
    soulforge_ui: &SoulforgeUiState,
) {
    let h = buffer.len();
    let w = if h > 0 { buffer[0].len() } else { return };

    let tick = soulforge_ui.animation_tick;

    let result = soulforge_ui.last_result.as_ref().unwrap();

    let crack_line = " \u{2573}  \u{2573}  \u{2573}  \u{2573}  \u{2573} ";
    let level_drop = if result.old_level == result.new_level {
        format!("+{} (no change)", result.old_level)
    } else {
        format!("+{} \u{2192} +{}", result.old_level, result.new_level)
    };
    let result_line = format!("Enhancement failed! {}", level_drop);

    let content_height = 9usize;
    let top = if h > content_height {
        (h - content_height) / 2
    } else {
        0
    };

    put_text_centered(buffer, top as i32, w, crack_line, Color::Red);

    let shake_offset: i32 = if tick < 5 {
        if tick.is_multiple_of(2) {
            1
        } else {
            -1
        }
    } else {
        0
    };
    let failed_text = "FAILED!";
    let failed_col = (w as i32 - failed_text.len() as i32) / 2 + shake_offset;
    put_text(
        buffer,
        (top + 2) as i32,
        failed_col,
        failed_text,
        Color::Red,
    );

    put_text_centered(buffer, (top + 4) as i32, w, &result_line, Color::Red);
    put_text_centered(buffer, (top + 6) as i32, w, crack_line, Color::Red);

    put_text_centered(
        buffer,
        (top + 8) as i32,
        w,
        "Press any key to continue",
        Color::DarkGray,
    );
}
