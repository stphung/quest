//! Soulforge UI rendering: equipment enhancement overlay with animations.

use crate::enhancement::{
    enhancement_cost, enhancement_multiplier, fail_penalty, success_rate, EnhancementProgress,
    SoulforgePhase, SoulforgeUiState, MAX_ENHANCEMENT_LEVEL,
};
use crate::items::EquipmentSlot;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::scene_fx::{current_millis, hash2d, lerp_rgb, put_cell, render_buffer, SceneCell};

const SLOT_ORDER: [EquipmentSlot; 7] = [
    EquipmentSlot::Weapon,
    EquipmentSlot::Armor,
    EquipmentSlot::Helmet,
    EquipmentSlot::Gloves,
    EquipmentSlot::Boots,
    EquipmentSlot::Amulet,
    EquipmentSlot::Ring,
];

/// Enhancement level color based on tier
fn level_color(level: u8) -> Color {
    match level {
        0 => Color::DarkGray,
        1..=4 => Color::White,
        5..=7 => Color::Yellow,
        8..=9 => Color::Magenta,
        10 => Color::Rgb(255, 215, 0),
        _ => Color::DarkGray,
    }
}

/// Write a string into the scene buffer at (row, col). Each char occupies 1 cell.
fn put_text(buffer: &mut [Vec<SceneCell>], row: i32, col: i32, text: &str, fg: Color) {
    for (i, ch) in text.chars().enumerate() {
        put_cell(buffer, row, col + i as i32, ch, fg);
    }
}

/// Write a string centered horizontally in the buffer.
fn put_text_centered(buffer: &mut [Vec<SceneCell>], row: i32, width: usize, text: &str, fg: Color) {
    let col = (width as i32 - text.chars().count() as i32) / 2;
    put_text(buffer, row, col, text, fg);
}

/// Parameters controlling the forge backdrop appearance.
struct ForgeBackdropParams {
    bottom_rgb: (u8, u8, u8),
    top_rgb: (u8, u8, u8),
    ember_count: usize,
    ember_speed: f64,
    ember_upward: bool,
    ember_hot: (u8, u8, u8),
    ember_cool: (u8, u8, u8),
    shimmer: bool,
}

impl ForgeBackdropParams {
    /// Standard warm forge glow (Menu, Confirming phases).
    fn normal() -> Self {
        Self {
            bottom_rgb: (120, 40, 15),
            top_rgb: (15, 8, 5),
            ember_count: 10,
            ember_speed: 5.0,
            ember_upward: true,
            ember_hot: (255, 160, 40),
            ember_cool: (80, 20, 5),
            shimmer: true,
        }
    }

    /// Intensified forge during hammering.
    fn hot() -> Self {
        Self {
            bottom_rgb: (180, 60, 20),
            top_rgb: (25, 12, 8),
            ember_count: 14,
            ember_speed: 7.0,
            ember_upward: true,
            ember_hot: (255, 200, 60),
            ember_cool: (120, 40, 10),
            shimmer: true,
        }
    }
}

/// Paint the forge backdrop into the buffer: gradient background, drifting embers, heat shimmer.
fn paint_forge_backdrop(buffer: &mut [Vec<SceneCell>], millis: u128, params: &ForgeBackdropParams) {
    let height = buffer.len();
    if height == 0 {
        return;
    }
    let width = buffer[0].len();

    // 1. Background gradient (top to bottom)
    for (row, row_cells) in buffer.iter_mut().enumerate() {
        let t = if height <= 1 {
            0.0
        } else {
            row as f64 / (height - 1) as f64
        };
        let rgb = lerp_rgb(params.top_rgb, params.bottom_rgb, t);
        let bg = Color::Rgb(rgb.0, rgb.1, rgb.2);
        for cell in row_cells.iter_mut() {
            cell.bg = bg;
        }
    }

    // 2. Drifting embers
    let ember_chars: &[char] = &['\u{00b7}', '\u{2022}', '*', '\u{2726}'];
    for i in 0..params.ember_count {
        let seed = hash2d(i, 0);
        let col = (seed as usize) % width;
        let ch = ember_chars[(hash2d(i, 1) as usize) % ember_chars.len()];

        let phase_offset = (seed as f64) * 0.73;
        let pos = (phase_offset + millis as f64 * params.ember_speed / 1000.0) % height as f64;
        let row_f = if params.ember_upward {
            (height - 1) as f64 - pos
        } else {
            pos
        };
        let row = row_f as i32;

        let t = pos / height.max(1) as f64;
        let rgb = lerp_rgb(params.ember_hot, params.ember_cool, t);
        put_cell(buffer, row, col as i32, ch, Color::Rgb(rgb.0, rgb.1, rgb.2));
    }

    // 3. Heat shimmer
    if params.shimmer {
        let shimmer_phase = millis as f64 / 150.0;
        for (row, row_cells) in buffer.iter_mut().enumerate() {
            for (col, cell) in row_cells.iter_mut().enumerate() {
                if hash2d(row, col).is_multiple_of(7) {
                    let shift =
                        ((shimmer_phase + row as f64 * 0.3 + col as f64 * 0.2).sin() * 8.0) as i16;
                    if let Color::Rgb(r, g, b) = cell.bg {
                        let new_r = (r as i16 + shift).clamp(0, 255) as u8;
                        cell.bg = Color::Rgb(new_r, g, b);
                    }
                }
            }
        }
    }
}

/// Render the soulforge overlay
pub fn render_soulforge(
    frame: &mut Frame,
    area: Rect,
    soulforge_ui: &SoulforgeUiState,
    enhancement: &EnhancementProgress,
    prestige_rank: u32,
    _ctx: &super::responsive::LayoutContext,
) {
    // Center overlay: 62 wide, 24 tall (or fit to terminal)
    let overlay_width = 62u16.min(area.width.saturating_sub(4));
    let overlay_height = 24u16.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(overlay_width)) / 2;
    let y = area.y + (area.height.saturating_sub(overlay_height)) / 2;
    let overlay_area = Rect::new(x, y, overlay_width, overlay_height);

    frame.render_widget(Clear, overlay_area);

    let title = format!(
        " \u{2692} The Soulforge  [Prestige Ranks: {}] ",
        prestige_rank
    );
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    match soulforge_ui.phase {
        SoulforgePhase::Menu => {
            render_menu(frame, inner, soulforge_ui, enhancement, prestige_rank);
        }
        SoulforgePhase::Confirming => {
            render_confirming(frame, inner, soulforge_ui, enhancement, prestige_rank);
        }
        SoulforgePhase::Hammering => {
            render_hammering(frame, inner, soulforge_ui, enhancement);
        }
        SoulforgePhase::ResultSuccess => {
            render_success(frame, inner, soulforge_ui);
        }
        SoulforgePhase::ResultFailure => {
            render_failure(frame, inner, soulforge_ui);
        }
    }
}

/// Render the equipment slot menu using scene buffer with forge backdrop.
fn render_menu(
    frame: &mut Frame,
    area: Rect,
    soulforge_ui: &SoulforgeUiState,
    enhancement: &EnhancementProgress,
    prestige_rank: u32,
) {
    let w = area.width as usize;
    let h = area.height as usize;
    if w == 0 || h == 0 {
        return;
    }

    // 1. Create buffer and paint forge backdrop
    let mut buffer = vec![vec![SceneCell::default(); w]; h];
    let millis = current_millis();
    paint_forge_backdrop(&mut buffer, millis, &ForgeBackdropParams::normal());

    // 2. Flavor text (rows 0-2) with slow warm color pulse
    let flavor_lines = [
        "Ancient runes pulse with forgotten power.",
        "This forge tempers the soul, not the steel.",
        "All that you wield will strike truer.",
    ];
    // Slow warm pulse: oscillate between warm white and golden amber
    let pulse_t = ((millis as f64 / 2000.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let pulse_rgb = lerp_rgb((220, 210, 190), (255, 180, 80), pulse_t);
    let flavor_fg = Color::Rgb(pulse_rgb.0, pulse_rgb.1, pulse_rgb.2);
    for (i, line) in flavor_lines.iter().enumerate() {
        put_text(&mut buffer, i as i32, 0, line, flavor_fg);
    }

    // Row 3 is spacer

    // 3. Equipment slot rows (row 4+, 7 rows)
    let slot_start_row = 4i32;
    for (i, slot) in SLOT_ORDER.iter().enumerate() {
        let row = slot_start_row + i as i32;
        if row >= h as i32 {
            break;
        }
        let is_selected = i == soulforge_ui.selected_slot;
        let current_level = enhancement.level(i);

        let mut col = 0i32;

        // Selection indicator
        if is_selected {
            put_text(&mut buffer, row, col, "> ", Color::Yellow);
        }
        col += 2;

        // Slot icon (put raw text; advance by icon_width then add padding)
        let icon = slot.icon();
        put_text(&mut buffer, row, col, icon, Color::Reset);
        col += slot.icon_width() as i32;
        // Pad narrow (width 1) icons with 2 spaces, wide (width 2) with 1, to align columns
        let icon_pad = if slot.icon_width() == 1 { 2 } else { 1 };
        col += icon_pad;

        // Slot name padded to 8 chars
        let name = format!("{:width$}", slot.name(), width = 8);
        put_text(&mut buffer, row, col, &name, Color::White);
        col += 8;
        put_text(&mut buffer, row, col, " ", Color::Reset);
        col += 1;

        // Enhancement level and target
        if current_level >= MAX_ENHANCEMENT_LEVEL {
            put_text(
                &mut buffer,
                row,
                col,
                "+10 MAX    ",
                Color::Rgb(255, 215, 0),
            );
        } else {
            let lvl_color = level_color(current_level);
            let lvl_str = format!("+{:<2}", current_level);
            put_text(&mut buffer, row, col, &lvl_str, lvl_color);
            col += lvl_str.len() as i32;

            let target = current_level + 1;
            let arrow_str = format!(" \u{2192} +{:<2}", target);
            put_text(&mut buffer, row, col, &arrow_str, level_color(target));
            col += arrow_str.len() as i32;

            let rate = success_rate(target);
            let rate_color = if rate >= 1.0 {
                Color::Green
            } else if rate >= 0.5 {
                Color::Yellow
            } else {
                Color::Red
            };
            let rate_str = format!(" {:>3.0}% Success", rate * 100.0);
            put_text(&mut buffer, row, col, &rate_str, rate_color);
        }

        // Selected row gets highlighted background
        if is_selected {
            let highlight_bg = Color::Rgb(40, 40, 20);
            if (row as usize) < h {
                for cell in buffer[row as usize].iter_mut() {
                    cell.bg = highlight_bg;
                }
            }
        }
    }

    // Row after slots: spacer (slot_start_row + 7 = row 11)
    // Detail panel: 3 rows starting at row 12
    let detail_start_row = slot_start_row + 7 + 1; // row 12
    let selected_level = enhancement.level(soulforge_ui.selected_slot);

    if selected_level >= MAX_ENHANCEMENT_LEVEL {
        // Line 1: Bonus
        let bonus_label = "Bonus: ";
        let bonus_value = format!(
            "+{:.1}% Power",
            (enhancement_multiplier(MAX_ENHANCEMENT_LEVEL) - 1.0) * 100.0
        );
        put_text(
            &mut buffer,
            detail_start_row,
            0,
            bonus_label,
            Color::DarkGray,
        );
        put_text(
            &mut buffer,
            detail_start_row,
            bonus_label.len() as i32,
            &bonus_value,
            Color::Rgb(255, 215, 0),
        );
        // Line 2: Max reached
        put_text(
            &mut buffer,
            detail_start_row + 1,
            0,
            "Maximum enhancement reached.",
            Color::DarkGray,
        );
    } else {
        let target = selected_level + 1;
        let bonus = enhancement_multiplier(target);
        let bonus_pct = (bonus - 1.0) * 100.0;
        let rate = success_rate(target);
        let cost = enhancement_cost(target);
        let can_afford = prestige_rank >= cost;
        let penalty = fail_penalty(target);

        let rate_color = if rate >= 1.0 {
            Color::Green
        } else if rate >= 0.5 {
            Color::Yellow
        } else {
            Color::Red
        };
        let cost_color = if can_afford { Color::Cyan } else { Color::Red };

        // Detail line 1: Bonus, Rate, Cost
        let mut col = 0i32;
        let bonus_label = "Bonus: ";
        put_text(
            &mut buffer,
            detail_start_row,
            col,
            bonus_label,
            Color::DarkGray,
        );
        col += bonus_label.len() as i32;
        let bonus_val = format!("+{:.1}% Power", bonus_pct);
        put_text(&mut buffer, detail_start_row, col, &bonus_val, Color::Green);
        col += bonus_val.len() as i32;
        let rate_label = "  Rate: ";
        put_text(
            &mut buffer,
            detail_start_row,
            col,
            rate_label,
            Color::DarkGray,
        );
        col += rate_label.len() as i32;
        let rate_val = format!("{:.0}%", rate * 100.0);
        put_text(&mut buffer, detail_start_row, col, &rate_val, rate_color);
        col += rate_val.len() as i32;
        let cost_label = "  Cost: ";
        put_text(
            &mut buffer,
            detail_start_row,
            col,
            cost_label,
            Color::DarkGray,
        );
        col += cost_label.len() as i32;
        let cost_val = format!("{} Prestige Ranks", cost);
        put_text(&mut buffer, detail_start_row, col, &cost_val, cost_color);

        // Detail line 2: On failure
        let fail_label = "On failure: ";
        put_text(
            &mut buffer,
            detail_start_row + 1,
            0,
            fail_label,
            Color::DarkGray,
        );
        let fail_col = fail_label.len() as i32;
        if penalty == 0 {
            put_text(
                &mut buffer,
                detail_start_row + 1,
                fail_col,
                "safe (no level loss)",
                Color::Green,
            );
        } else {
            let result_level = selected_level.saturating_sub(penalty);
            let fail_text = format!(
                "-{} level{} (+{} \u{2192} +{})",
                penalty,
                if penalty > 1 { "s" } else { "" },
                selected_level,
                result_level
            );
            put_text(
                &mut buffer,
                detail_start_row + 1,
                fail_col,
                &fail_text,
                Color::Red,
            );
        }
    }

    // Stats row (detail_start_row + 3 = row 15, accounting for spacer)
    let stats_row = detail_start_row + 3;
    {
        let mut col = 0i32;
        let a_label = "Attempts: ";
        put_text(&mut buffer, stats_row, col, a_label, Color::DarkGray);
        col += a_label.len() as i32;
        let a_val = format!("{}", enhancement.total_attempts);
        put_text(&mut buffer, stats_row, col, &a_val, Color::White);
        col += a_val.len() as i32;
        let s_label = " | Successes: ";
        put_text(&mut buffer, stats_row, col, s_label, Color::DarkGray);
        col += s_label.len() as i32;
        let s_val = format!("{}", enhancement.total_successes);
        put_text(&mut buffer, stats_row, col, &s_val, Color::Green);
        col += s_val.len() as i32;
        let f_label = " | Failures: ";
        put_text(&mut buffer, stats_row, col, f_label, Color::DarkGray);
        col += f_label.len() as i32;
        let f_val = format!("{}", enhancement.total_failures);
        put_text(&mut buffer, stats_row, col, &f_val, Color::Red);
    }

    // Help row (stats_row + 1 = row 16)
    let help_row = stats_row + 1;
    put_text(
        &mut buffer,
        help_row,
        0,
        "[\u{2191}\u{2193}] Select  [Enter] Enhance  [Esc] Close",
        Color::DarkGray,
    );

    // 4. Flush buffer to frame
    render_buffer(frame, area, &buffer);
}

/// Render the confirmation phase using scene buffer with intensified forge backdrop.
fn render_confirming(
    frame: &mut Frame,
    area: Rect,
    soulforge_ui: &SoulforgeUiState,
    enhancement: &EnhancementProgress,
    prestige_rank: u32,
) {
    let w = area.width as usize;
    let h = area.height as usize;
    if w == 0 || h == 0 {
        return;
    }

    // 1. Create buffer and paint slightly intensified forge backdrop
    let mut buffer = vec![vec![SceneCell::default(); w]; h];
    let millis = current_millis();
    let mut params = ForgeBackdropParams::normal();
    params.bottom_rgb = (150, 50, 18);
    params.ember_count = 12;
    paint_forge_backdrop(&mut buffer, millis, &params);

    // 2. Compute enhancement data
    let slot_index = soulforge_ui.selected_slot;
    let slot = SLOT_ORDER[slot_index];
    let current_level = enhancement.level(slot_index);
    let target_level = current_level + 1;
    let cost = enhancement_cost(target_level);
    let rate = success_rate(target_level);
    let bonus = enhancement_multiplier(target_level);
    let bonus_pct = (bonus - 1.0) * 100.0;

    // 3. Center 7 content lines vertically (title, blank, rate/cost, bonus, blank, help)
    //    Layout: blank, title, blank, rate+cost, bonus, blank, help = 7 lines
    let content_height = 7usize;
    let top = if h > content_height {
        (h - content_height) / 2
    } else {
        0
    };

    // Row 0 (top+0): blank (part of vertical centering)
    // Row 1 (top+1): "Enhance [SlotName] to +[target]?" with pulsing yellow glow
    let title = format!("Enhance {} to +{}?", slot.name(), target_level);
    let pulse_t = ((millis as f64 / 600.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let title_rgb = lerp_rgb((200, 180, 50), (255, 255, 100), pulse_t);
    put_text_centered(
        &mut buffer,
        (top + 1) as i32,
        w,
        &title,
        Color::Rgb(title_rgb.0, title_rgb.1, title_rgb.2),
    );

    // Row 2 (top+2): blank

    // Row 3 (top+3): Success rate + Cost + Remaining ranks
    let rate_color = if rate >= 0.5 {
        Color::Green
    } else {
        Color::Red
    };
    let remaining = prestige_rank.saturating_sub(cost);
    let detail_line = format!(
        "Success rate: {:.0}%  Cost: {} Prestige Ranks ({} \u{2192} {})",
        rate * 100.0,
        cost,
        prestige_rank,
        remaining
    );
    // Render piece by piece for per-segment coloring
    {
        let rate_str = format!("{:.0}%", rate * 100.0);
        let cost_str = format!("{} Prestige Ranks", cost);
        let remaining_str = format!("({} \u{2192} {})", prestige_rank, remaining);
        let full_len = detail_line.len();
        let start_col = (w as i32 - full_len as i32) / 2;
        let mut col = start_col;

        let label1 = "Success rate: ";
        put_text(&mut buffer, (top + 3) as i32, col, label1, Color::White);
        col += label1.len() as i32;

        put_text(&mut buffer, (top + 3) as i32, col, &rate_str, rate_color);
        col += rate_str.len() as i32;

        let label2 = "  Cost: ";
        put_text(&mut buffer, (top + 3) as i32, col, label2, Color::White);
        col += label2.len() as i32;

        put_text(&mut buffer, (top + 3) as i32, col, &cost_str, Color::Cyan);
        col += cost_str.len() as i32;

        let label3 = " ";
        put_text(&mut buffer, (top + 3) as i32, col, label3, Color::Reset);
        col += label3.len() as i32;

        put_text(
            &mut buffer,
            (top + 3) as i32,
            col,
            &remaining_str,
            Color::DarkGray,
        );
    }

    // Row 4 (top+4): Bonus at target level
    let bonus_line = format!("Bonus at +{}: +{:.1}% Power", target_level, bonus_pct);
    put_text_centered(&mut buffer, (top + 4) as i32, w, &bonus_line, Color::Green);

    // Row 5 (top+5): blank

    // Row 6 (top+6): Help text
    put_text_centered(
        &mut buffer,
        (top + 6) as i32,
        w,
        "[Enter] Confirm  [Esc] Cancel",
        Color::DarkGray,
    );

    // 4. Flush buffer to frame
    render_buffer(frame, area, &buffer);
}

/// Render spark particles that spray from a point. Fully derived from animation_tick -- no mutable state.
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

        // Fan angle: 30-150 degrees (0.52 - 2.62 radians)
        let angle = 0.52 + (seed % 1000) as f64 / 1000.0 * 2.1;
        let speed = 1.5 + (hash2d(i, strike_tick as usize) % 100) as f64 / 100.0 * 2.0;

        let vx = angle.cos() * speed;
        let vy = angle.sin() * speed;
        let gravity = 0.15;

        let col = center_col as f64 + vx * t;
        let row = center_row as f64 - vy * t + 0.5 * gravity * t * t;

        // Color: bright white/yellow -> orange -> dark red
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

/// Render the hammering animation
fn render_hammering(
    frame: &mut Frame,
    area: Rect,
    soulforge_ui: &SoulforgeUiState,
    enhancement: &EnhancementProgress,
) {
    let w = area.width as usize;
    let h = area.height as usize;
    if w == 0 || h == 0 {
        return;
    }

    let tick = soulforge_ui.animation_tick;
    let millis = current_millis();

    // 1. Create buffer and paint forge backdrop
    let mut buffer = vec![vec![SceneCell::default(); w]; h];
    paint_forge_backdrop(&mut buffer, millis, &ForgeBackdropParams::hot());

    // 2. Determine if current tick is a strike
    let is_strike = matches!(tick, 14..=16 | 30..=32 | 46..=48);
    let is_strike_start = matches!(tick, 14 | 30 | 46);

    // Hammer ASCII art
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

    // Center the art vertically (5 hammer + 1 gap + 6 anvil = 12 rows)
    let total_art_height = 12;
    let art_top = if h > total_art_height + 4 {
        (h - total_art_height - 4) / 2
    } else {
        0
    };

    // Horizontal center for the widest piece (anvil at 17 chars)
    let anvil_width = 17;
    let hammer_width = 12;
    let art_center_col = (w as i32) / 2;
    let anvil_col = art_center_col - anvil_width / 2;
    let hammer_col = art_center_col - hammer_width / 2;

    // 3. Hammer afterimage: on first tick of each strike, render raised hammer dimly
    if is_strike_start {
        let dim_color = Color::Rgb(50, 40, 35);
        for (i, line) in hammer_raised.iter().enumerate() {
            let row = art_top as i32 + i as i32;
            put_text(&mut buffer, row, hammer_col, line, dim_color);
        }
    }

    // 4. Render hammer (raised or striking)
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
        put_text(&mut buffer, row, hammer_col, line, hammer_fg);
    }

    // 5. Anvil glow: pulse from gray to warm orange on strikes
    let anvil_top = art_top + hammer_raised.len() + 1; // 1 row gap between hammer and anvil
    let anvil_base_rgb = (90, 80, 70);
    let anvil_hot_rgb = (200, 140, 40);

    // Compute anvil glow intensity based on proximity to last strike
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
    // Glow fades over 8 ticks after a strike
    let glow_t = if closest_strike_age <= 8 {
        1.0 - closest_strike_age as f64 / 8.0
    } else {
        0.0
    };
    // Keep a subtle warm tint between strikes
    let glow_t = glow_t.max(0.1);
    let anvil_rgb = lerp_rgb(anvil_base_rgb, anvil_hot_rgb, glow_t);
    let anvil_fg = Color::Rgb(anvil_rgb.0, anvil_rgb.1, anvil_rgb.2);

    for (i, line) in anvil.iter().enumerate() {
        let row = anvil_top as i32 + i as i32;
        put_text(&mut buffer, row, anvil_col, line, anvil_fg);
    }

    // 6. Spark shower: for each strike start, render sparks if within 10 ticks
    let spark_center_row = anvil_top as i32; // top of anvil
    let spark_center_col = art_center_col;
    for &st in &strike_ticks {
        if tick >= st && tick.saturating_sub(st) <= 10 {
            render_sparks(
                &mut buffer,
                spark_center_row,
                spark_center_col,
                st,
                tick,
                10,
            );
        }
    }

    // 7. Progress bar with pulsing fill color
    let bar_row = (anvil_top + anvil.len() + 1) as i32;
    let bar_width = w.saturating_sub(6);
    let progress = tick as f64 / 50.0;
    let fill_exact = progress * bar_width as f64;
    let full_cells = fill_exact as usize;
    let fraction = fill_exact - full_cells as f64;

    // Fractional block characters: ▏▎▍▌▋▊▉█ (1/8 to 8/8)
    let blocks: &[char] = &[
        '\u{258f}', '\u{258e}', '\u{258d}', '\u{258c}', '\u{258b}', '\u{258a}', '\u{2589}',
        '\u{2588}',
    ];

    // Pulsing fill color: oscillate between dark orange and bright yellow
    let pulse_t = ((millis as f64 / 200.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let fill_rgb = lerp_rgb((200, 120, 20), (255, 255, 80), pulse_t);
    let fill_fg = Color::Rgb(fill_rgb.0, fill_rgb.1, fill_rgb.2);
    let bracket_fg = Color::DarkGray;
    let bar_start_col = 3i32; // "  [" offset

    put_text(&mut buffer, bar_row, bar_start_col - 1, "[", bracket_fg);
    for i in 0..bar_width {
        let col = bar_start_col + i as i32;
        if i < full_cells {
            put_cell(&mut buffer, bar_row, col, '\u{2588}', fill_fg);
        } else if i == full_cells {
            let partial_idx = (fraction * 8.0) as usize;
            if partial_idx > 0 {
                put_cell(&mut buffer, bar_row, col, blocks[partial_idx - 1], fill_fg);
            } else {
                put_cell(&mut buffer, bar_row, col, ' ', bracket_fg);
            }
        } else {
            put_cell(&mut buffer, bar_row, col, ' ', bracket_fg);
        }
    }
    put_text(
        &mut buffer,
        bar_row,
        bar_start_col + bar_width as i32,
        "]",
        bracket_fg,
    );

    // 8. Item label centered below bar
    let slot_index = soulforge_ui.selected_slot;
    let slot = SLOT_ORDER[slot_index];
    let current_level = enhancement.level(slot_index);
    let item_display = format!("{} +{}", slot.name(), current_level);
    let label_row = bar_row + 1;
    put_text_centered(&mut buffer, label_row, w, &item_display, Color::White);

    // 9. Flush buffer to frame
    render_buffer(frame, area, &buffer);
}

/// Render the success animation with golden burst and sparkle effects.
fn render_success(frame: &mut Frame, area: Rect, soulforge_ui: &SoulforgeUiState) {
    let w = area.width as usize;
    let h = area.height as usize;
    if w == 0 || h == 0 {
        return;
    }

    let tick = soulforge_ui.animation_tick;
    let millis = current_millis();

    // 1. Create buffer and paint backdrop that transitions from normal forge to golden
    let mut buffer = vec![vec![SceneCell::default(); w]; h];
    let intensity = (tick as f64 / 30.0).min(1.0);
    let params = ForgeBackdropParams {
        bottom_rgb: lerp_rgb((120, 40, 15), (200, 170, 50), intensity),
        top_rgb: lerp_rgb((15, 8, 5), (40, 30, 10), intensity),
        ember_count: 10 + (10.0 * intensity) as usize,
        ember_speed: 5.0 + 3.0 * intensity,
        ember_upward: true,
        ember_hot: lerp_rgb((255, 160, 40), (255, 230, 100), intensity),
        ember_cool: lerp_rgb((80, 20, 5), (200, 150, 30), intensity),
        shimmer: true,
    };
    paint_forge_backdrop(&mut buffer, millis, &params);

    // 2. Sparkle twinkle: ~15 sparkle characters at random positions
    let sparkle_chars: &[char] = &['\u{2726}', '\u{2727}', '*'];
    for i in 0..15 {
        let phase_seed = hash2d(i, tick as usize / 3);
        let row = (phase_seed as usize) % h;
        let col = (hash2d(i + 100, tick as usize / 3) as usize) % w;
        let ch = sparkle_chars[(hash2d(i, 0) as usize) % sparkle_chars.len()];

        // Sine-based brightness oscillation
        let brightness =
            ((millis as f64 / 300.0 + i as f64 * 1.7).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
        let rgb = lerp_rgb((80, 60, 20), (255, 230, 100), brightness);
        put_cell(
            &mut buffer,
            row as i32,
            col as i32,
            ch,
            Color::Rgb(rgb.0, rgb.1, rgb.2),
        );
    }

    // 3. Display centered content
    let result = soulforge_ui.last_result.as_ref().unwrap();
    let slot = SLOT_ORDER[result.slot_index];
    let item_name = slot.name();
    let bonus = enhancement_multiplier(result.new_level);
    let bonus_pct = (bonus - 1.0) * 100.0;

    // Build sparkle border line (width of buffer)
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

    // Content block: sparkle, blank, SUCCESS!, blank, item line, power line, blank, sparkle, help
    let content_height = 9usize;
    let top = if h > content_height {
        (h - content_height) / 2
    } else {
        0
    };

    // Top sparkle border
    put_text_centered(&mut buffer, top as i32, w, &sparkle_border, Color::Yellow);

    // "SUCCESS!" pulsing between yellow and gold
    let title_color = if tick % 4 < 2 {
        Color::Yellow
    } else {
        Color::Rgb(255, 215, 0)
    };
    put_text_centered(&mut buffer, (top + 2) as i32, w, "SUCCESS!", title_color);

    // "[SlotName] is now +[new_level]!" in green
    let item_line = format!("{} is now +{}!", item_name, result.new_level);
    put_text_centered(&mut buffer, (top + 4) as i32, w, &item_line, Color::Green);

    // "+[bonus]% Power" in yellow
    let power_line = format!("+{:.1}% Power", bonus_pct);
    put_text_centered(&mut buffer, (top + 5) as i32, w, &power_line, Color::Yellow);

    // Bottom sparkle border
    put_text_centered(
        &mut buffer,
        (top + 7) as i32,
        w,
        &sparkle_border,
        Color::Yellow,
    );

    // "Press any key to continue" in dark gray
    put_text_centered(
        &mut buffer,
        (top + 8) as i32,
        w,
        "Press any key to continue",
        Color::DarkGray,
    );

    // 4. Flush buffer to frame
    render_buffer(frame, area, &buffer);
}

/// Render the failure animation with ash decay and crack effects.
fn render_failure(frame: &mut Frame, area: Rect, soulforge_ui: &SoulforgeUiState) {
    let w = area.width as usize;
    let h = area.height as usize;
    if w == 0 || h == 0 {
        return;
    }

    let tick = soulforge_ui.animation_tick;
    let millis = current_millis();

    // 1. Create buffer and paint backdrop that rapidly cools over first 8 ticks
    let mut buffer = vec![vec![SceneCell::default(); w]; h];
    let cool_t = (tick as f64 / 8.0).min(1.0);
    let params = ForgeBackdropParams {
        bottom_rgb: lerp_rgb((120, 40, 15), (40, 40, 45), cool_t),
        top_rgb: lerp_rgb((15, 8, 5), (15, 15, 18), cool_t),
        ember_count: 10 - (6.0 * cool_t) as usize, // 10 -> 4
        ember_speed: 5.0 - 3.0 * cool_t,           // 5.0 -> 2.0
        ember_upward: cool_t < 1.0,                // embers fall after full cooling
        ember_hot: lerp_rgb((255, 160, 40), (80, 30, 10), cool_t),
        ember_cool: lerp_rgb((80, 20, 5), (30, 15, 10), cool_t),
        shimmer: cool_t <= 0.5,
    };
    paint_forge_backdrop(&mut buffer, millis, &params);

    // 2. Crack characters at deterministic positions, spreading with cool_t
    let max_cracks = 8usize;
    let active_cracks = ((cool_t * max_cracks as f64).ceil() as usize).min(max_cracks);
    let crack_char = '\u{2573}'; // ╳
    let crack_rgb = lerp_rgb((180, 60, 20), (80, 80, 90), cool_t);
    let crack_fg = Color::Rgb(crack_rgb.0, crack_rgb.1, crack_rgb.2);
    for i in 0..active_cracks {
        let seed = hash2d(i + 42, i * 7 + 13);
        let row = (seed as usize) % h;
        let col = (hash2d(i + 99, i * 3 + 5) as usize) % w;
        put_cell(&mut buffer, row as i32, col as i32, crack_char, crack_fg);
    }

    // 3. Display centered content
    let result = soulforge_ui.last_result.as_ref().unwrap();

    let crack_line = " \u{2573}  \u{2573}  \u{2573}  \u{2573}  \u{2573} ";
    let level_drop = if result.old_level == result.new_level {
        format!("+{} (no change)", result.old_level)
    } else {
        format!("+{} \u{2192} +{}", result.old_level, result.new_level)
    };
    let result_line = format!("Enhancement failed! {}", level_drop);

    // Content block: crack border, blank, FAILED!, blank, result, blank, crack border, blank, help
    let content_height = 9usize;
    let top = if h > content_height {
        (h - content_height) / 2
    } else {
        0
    };

    // Top crack border line
    put_text_centered(&mut buffer, top as i32, w, crack_line, Color::Red);

    // "FAILED!" with shake effect: +1/-1 col offset for first 5 ticks
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
        &mut buffer,
        (top + 2) as i32,
        failed_col,
        failed_text,
        Color::Red,
    );

    // Enhancement result line
    put_text_centered(&mut buffer, (top + 4) as i32, w, &result_line, Color::Red);

    // Bottom crack border line
    put_text_centered(&mut buffer, (top + 6) as i32, w, crack_line, Color::Red);

    // "Press any key to continue" in dark gray
    put_text_centered(
        &mut buffer,
        (top + 8) as i32,
        w,
        "Press any key to continue",
        Color::DarkGray,
    );

    // 4. Flush buffer to frame
    render_buffer(frame, area, &buffer);
}

/// Render the Soulforge discovery modal
pub fn render_soulforge_discovery_modal(
    frame: &mut Frame,
    area: Rect,
    _ctx: &super::responsive::LayoutContext,
) {
    // Center the modal
    let modal_width = 52u16.min(area.width.saturating_sub(4));
    let modal_height = 10u16.min(area.height.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(" \u{25b6} New System Unlocked \u{25c0} ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    let text = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "\u{2692}\u{fe0f} You've uncovered an ancient Soulforge!",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Ancient runes pulse with forgotten power.",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::ITALIC),
        )),
        Line::from(Span::styled(
            "This forge tempers the soul, not the steel.",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::ITALIC),
        )),
        Line::from(Span::styled(
            "All that you wield will strike truer.",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::ITALIC),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press [S] to visit. [Enter] to dismiss.",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .alignment(Alignment::Center);
    frame.render_widget(text, inner);
}
