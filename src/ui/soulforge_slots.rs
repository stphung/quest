//! Soulforge slot selection menu rendering.

use crate::enhancement::{
    enhancement_multiplier, fail_penalty, soul_tithe_cost, success_rate, EnhancementProgress,
    MAX_ENHANCEMENT_LEVEL,
};
use crate::items::EquipmentSlot;
use ratatui::style::Color;

use super::scene_fx::{lerp_rgb, put_text, put_text_centered, SceneCell};

pub(super) const SLOT_ORDER: [EquipmentSlot; 7] = [
    EquipmentSlot::Weapon,
    EquipmentSlot::Armor,
    EquipmentSlot::Helmet,
    EquipmentSlot::Gloves,
    EquipmentSlot::Boots,
    EquipmentSlot::Amulet,
    EquipmentSlot::Ring,
];

/// Enhancement level color based on tier
pub(super) fn level_color(level: u8) -> Color {
    match level {
        0 => Color::DarkGray,
        1..=4 => Color::White,
        5..=7 => Color::Yellow,
        8..=9 => Color::Magenta,
        10 => Color::Rgb(255, 215, 0),
        _ => Color::DarkGray,
    }
}

/// Render the equipment slot menu content into a scene buffer.
pub(super) fn render_menu_content(
    buffer: &mut [Vec<SceneCell>],
    millis: u128,
    soulforge_ui: &crate::enhancement::SoulforgeUiState,
    enhancement: &EnhancementProgress,
    prestige_rank: u32,
) {
    let h = buffer.len();
    let _w = if h > 0 { buffer[0].len() } else { return };

    // Flavor text (rows 0-2) with slow warm color pulse
    let flavor_lines = [
        "Ancient runes pulse with forgotten power.",
        "This forge tempers the soul, not the steel.",
        "All that you wield will strike truer.",
    ];
    let pulse_t = ((millis as f64 / 2000.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let pulse_rgb = lerp_rgb((220, 210, 190), (255, 180, 80), pulse_t);
    let flavor_fg = Color::Rgb(pulse_rgb.0, pulse_rgb.1, pulse_rgb.2);
    for (i, line) in flavor_lines.iter().enumerate() {
        put_text(buffer, i as i32, 0, line, flavor_fg);
    }

    // Row 3 is spacer

    // Equipment slot rows (row 4+, 7 rows)
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
            put_text(buffer, row, col, "> ", Color::Yellow);
        }
        col += 2;

        // Slot icon
        let icon = slot.icon();
        put_text(buffer, row, col, icon, Color::Reset);
        col += slot.icon_width() as i32;
        let icon_pad = if slot.icon_width() == 1 { 2 } else { 1 };
        col += icon_pad;

        // Slot name padded to 8 chars
        let name = format!("{:width$}", slot.name(), width = 8);
        put_text(buffer, row, col, &name, Color::White);
        col += 8;
        put_text(buffer, row, col, " ", Color::Reset);
        col += 1;

        // Enhancement level and target
        if current_level >= MAX_ENHANCEMENT_LEVEL {
            put_text(buffer, row, col, "+10 MAX    ", Color::Rgb(255, 215, 0));
        } else {
            let lvl_color = level_color(current_level);
            let lvl_str = format!("+{:<2}", current_level);
            put_text(buffer, row, col, &lvl_str, lvl_color);
            col += lvl_str.len() as i32;

            let target = current_level + 1;
            let arrow_str = format!(" \u{2192} +{:<2}", target);
            put_text(buffer, row, col, &arrow_str, level_color(target));
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
            put_text(buffer, row, col, &rate_str, rate_color);
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

    // Detail panel: 3 rows starting at row 12
    let detail_start_row = slot_start_row + 7 + 1;
    let selected_level = enhancement.level(soulforge_ui.selected_slot);

    if selected_level >= MAX_ENHANCEMENT_LEVEL {
        let bonus_label = "Bonus: ";
        let bonus_value = format!(
            "+{:.1}% Power",
            (enhancement_multiplier(MAX_ENHANCEMENT_LEVEL) - 1.0) * 100.0
        );
        put_text(buffer, detail_start_row, 0, bonus_label, Color::DarkGray);
        put_text(
            buffer,
            detail_start_row,
            bonus_label.len() as i32,
            &bonus_value,
            Color::Rgb(255, 215, 0),
        );
        put_text(
            buffer,
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
        let cost = crate::enhancement::enhancement_cost(target);
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
        put_text(buffer, detail_start_row, col, bonus_label, Color::DarkGray);
        col += bonus_label.len() as i32;
        let bonus_val = format!("+{:.1}% Power", bonus_pct);
        put_text(buffer, detail_start_row, col, &bonus_val, Color::Green);
        col += bonus_val.len() as i32;
        let rate_label = "  Rate: ";
        put_text(buffer, detail_start_row, col, rate_label, Color::DarkGray);
        col += rate_label.len() as i32;
        let rate_val = format!("{:.0}%", rate * 100.0);
        put_text(buffer, detail_start_row, col, &rate_val, rate_color);
        col += rate_val.len() as i32;
        let cost_label = "  Cost: ";
        put_text(buffer, detail_start_row, col, cost_label, Color::DarkGray);
        col += cost_label.len() as i32;
        let cost_val = format!("{} Prestige Ranks", cost);
        put_text(buffer, detail_start_row, col, &cost_val, cost_color);

        // Detail line 2: On failure
        let fail_label = "On failure: ";
        put_text(buffer, detail_start_row + 1, 0, fail_label, Color::DarkGray);
        let fail_col = fail_label.len() as i32;
        if penalty == 0 {
            put_text(
                buffer,
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
                buffer,
                detail_start_row + 1,
                fail_col,
                &fail_text,
                Color::Red,
            );
        }

        // Detail line 3: Soul Tithe hint (when available for the target level)
        if let Some(oc_cost) = soul_tithe_cost(target) {
            let oc_label = "Soul Tithe: ";
            put_text(buffer, detail_start_row + 2, 0, oc_label, Color::DarkGray);
            let oc_col = oc_label.len() as i32;
            let can_afford_oc = prestige_rank >= oc_cost;
            let oc_color = if can_afford_oc {
                Color::Rgb(100, 200, 255)
            } else {
                Color::DarkGray
            };
            let oc_text = format!("{} PR for 100% success", oc_cost);
            put_text(buffer, detail_start_row + 2, oc_col, &oc_text, oc_color);
        }
    }

    // Stats row
    let stats_row = detail_start_row + 3;
    {
        let mut col = 0i32;
        let a_label = "Attempts: ";
        put_text(buffer, stats_row, col, a_label, Color::DarkGray);
        col += a_label.len() as i32;
        let a_val = format!("{}", enhancement.total_attempts);
        put_text(buffer, stats_row, col, &a_val, Color::White);
        col += a_val.len() as i32;
        let s_label = " | Successes: ";
        put_text(buffer, stats_row, col, s_label, Color::DarkGray);
        col += s_label.len() as i32;
        let s_val = format!("{}", enhancement.total_successes);
        put_text(buffer, stats_row, col, &s_val, Color::Green);
        col += s_val.len() as i32;
        let f_label = " | Failures: ";
        put_text(buffer, stats_row, col, f_label, Color::DarkGray);
        col += f_label.len() as i32;
        let f_val = format!("{}", enhancement.total_failures);
        put_text(buffer, stats_row, col, &f_val, Color::Red);
    }

    // Help row
    let help_row = stats_row + 1;
    put_text(
        buffer,
        help_row,
        0,
        "[\u{2191}\u{2193}] Select  [Enter] Enhance  [Esc] Close",
        Color::DarkGray,
    );
}

/// Render the confirmation phase content into a scene buffer.
pub(super) fn render_confirming_content(
    buffer: &mut [Vec<SceneCell>],
    millis: u128,
    soulforge_ui: &crate::enhancement::SoulforgeUiState,
    enhancement: &EnhancementProgress,
    prestige_rank: u32,
) {
    let h = buffer.len();
    let w = if h > 0 { buffer[0].len() } else { return };

    let slot_index = soulforge_ui.selected_slot;
    let slot = SLOT_ORDER[slot_index];
    let current_level = enhancement.level(slot_index);
    let target_level = current_level + 1;
    let std_cost = crate::enhancement::enhancement_cost(target_level);
    let rate = success_rate(target_level);
    let bonus = enhancement_multiplier(target_level);
    let bonus_pct = (bonus - 1.0) * 100.0;
    let oc_cost_opt = soul_tithe_cost(target_level);
    let is_soul_tithe = soulforge_ui.soul_tithe && oc_cost_opt.is_some();

    // Active cost/rate based on selected mode
    let active_cost = if is_soul_tithe {
        oc_cost_opt.unwrap()
    } else {
        std_cost
    };
    let active_rate = if is_soul_tithe { 1.0 } else { rate };

    // Center content lines vertically (9 lines when soul tithe available, 7 otherwise)
    let has_mode_selector = oc_cost_opt.is_some();
    let content_height = if has_mode_selector { 10usize } else { 8usize };
    let top = if h > content_height {
        (h - content_height) / 2
    } else {
        0
    };

    // Title with pulsing yellow glow
    let title = format!("Enhance {} to +{}?", slot.name(), target_level);
    let pulse_t = ((millis as f64 / 600.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let title_rgb = lerp_rgb((200, 180, 50), (255, 255, 100), pulse_t);
    put_text_centered(
        buffer,
        (top + 1) as i32,
        w,
        &title,
        Color::Rgb(title_rgb.0, title_rgb.1, title_rgb.2),
    );

    // Mode selector row (when soul tithe is available for the target level)
    let detail_row_offset = if has_mode_selector {
        // Render mode selector at top+3
        let row = (top + 3) as i32;
        let oc_cost = oc_cost_opt.unwrap();

        // Build the two mode labels
        let std_label = format!("Standard: {:.0}% / {} PR", rate * 100.0, std_cost);
        let oc_label = format!("Soul Tithe: 100% / {} PR", oc_cost);
        let separator = "    ";
        let full_len = std_label.len() + separator.len() + oc_label.len();
        let start_col = (w as i32 - full_len as i32) / 2;
        let mut col = start_col;

        // Standard mode
        let std_fg = if !is_soul_tithe {
            Color::Yellow
        } else {
            Color::DarkGray
        };
        let std_prefix = if !is_soul_tithe { "\u{25b6} " } else { "  " };
        put_text(buffer, row, col - 2, std_prefix, std_fg);
        put_text(buffer, row, col, &std_label, std_fg);
        col += std_label.len() as i32;

        put_text(buffer, row, col, separator, Color::DarkGray);
        col += separator.len() as i32;

        // Soul Tithe mode
        let can_afford_oc = prestige_rank >= oc_cost;
        let oc_fg = if is_soul_tithe {
            Color::Rgb(100, 200, 255)
        } else if can_afford_oc {
            Color::DarkGray
        } else {
            Color::Rgb(80, 80, 80)
        };
        let oc_prefix = if is_soul_tithe { "\u{25b6} " } else { "  " };
        put_text(buffer, row, col - 2, oc_prefix, oc_fg);
        put_text(buffer, row, col, &oc_label, oc_fg);

        // Navigation hint
        put_text_centered(
            buffer,
            row + 1,
            w,
            "[\u{2190}\u{2192}] Switch mode",
            Color::DarkGray,
        );

        3 // detail lines shift down by 3 (mode selector + hint + spacer)
    } else {
        0
    };

    // Success rate + Cost + Remaining ranks
    let rate_color = if active_rate >= 1.0 {
        Color::Green
    } else if active_rate >= 0.5 {
        Color::Yellow
    } else {
        Color::Red
    };
    let remaining = prestige_rank.saturating_sub(active_cost);
    let detail_line = format!(
        "Success rate: {:.0}%  Cost: {} Prestige Ranks ({} \u{2192} {})",
        active_rate * 100.0,
        active_cost,
        prestige_rank,
        remaining
    );
    {
        let rate_str = format!("{:.0}%", active_rate * 100.0);
        let cost_str = format!("{} Prestige Ranks", active_cost);
        let remaining_str = format!("({} \u{2192} {})", prestige_rank, remaining);
        let full_len = detail_line.len();
        let start_col = (w as i32 - full_len as i32) / 2;
        let mut col = start_col;

        let detail_row = (top + 3 + detail_row_offset) as i32;

        let label1 = "Success rate: ";
        put_text(buffer, detail_row, col, label1, Color::White);
        col += label1.len() as i32;

        put_text(buffer, detail_row, col, &rate_str, rate_color);
        col += rate_str.len() as i32;

        let label2 = "  Cost: ";
        put_text(buffer, detail_row, col, label2, Color::White);
        col += label2.len() as i32;

        put_text(buffer, detail_row, col, &cost_str, Color::Cyan);
        col += cost_str.len() as i32;

        let label3 = " ";
        put_text(buffer, detail_row, col, label3, Color::Reset);
        col += label3.len() as i32;

        put_text(buffer, detail_row, col, &remaining_str, Color::DarkGray);
    }

    // On failure line
    let failure_row = (top + 4 + detail_row_offset) as i32;
    if is_soul_tithe {
        put_text_centered(
            buffer,
            failure_row,
            w,
            "On failure: guaranteed success",
            Color::Green,
        );
    } else {
        let penalty = fail_penalty(target_level);
        if penalty == 0 {
            put_text_centered(
                buffer,
                failure_row,
                w,
                "On failure: safe (no level loss)",
                Color::Green,
            );
        } else {
            let result_level = current_level.saturating_sub(penalty);
            let fail_text = format!(
                "On failure: -{} level{} (+{} \u{2192} +{})",
                penalty,
                if penalty > 1 { "s" } else { "" },
                current_level,
                result_level
            );
            put_text_centered(buffer, failure_row, w, &fail_text, Color::Red);
        }
    }

    // Bonus at target level
    let bonus_line = format!("Bonus at +{}: +{:.1}% Power", target_level, bonus_pct);
    put_text_centered(
        buffer,
        (top + 5 + detail_row_offset) as i32,
        w,
        &bonus_line,
        Color::Green,
    );

    // Help text
    let help_text = if has_mode_selector {
        "[Enter] Confirm  [\u{2190}\u{2192}] Mode  [Esc] Cancel"
    } else {
        "[Enter] Confirm  [Esc] Cancel"
    };
    put_text_centered(
        buffer,
        (top + 7 + detail_row_offset) as i32,
        w,
        help_text,
        Color::DarkGray,
    );
}
