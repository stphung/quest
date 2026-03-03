//! Equipment rendering helpers for the stats panel.

use crate::core::game_state::GameState;
use ratatui::{
    layout::{Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Returns the style for an enhancement prefix based on its color tier.
pub(super) fn enhancement_style(level: u8) -> Style {
    let (r, g, b) = crate::enhancement::enhancement_color_rgb(level);
    let tier = crate::enhancement::enhancement_color_tier(level);
    let style = Style::default().fg(Color::Rgb(r, g, b));
    match tier {
        2..=4 => style.add_modifier(Modifier::BOLD),
        _ => style,
    }
}

/// Draws equipment with name + rarity color only, one line per slot (L tier).
/// Table layout: Slot  Name  Rarity  Tier  ilvl (right-aligned columns).
pub(super) fn draw_equipment_names_only(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    enhancement_levels: &[u8; 7],
) {
    use crate::items::EquipmentSlot;
    let slot_order = [
        EquipmentSlot::Weapon,
        EquipmentSlot::Armor,
        EquipmentSlot::Helmet,
        EquipmentSlot::Gloves,
        EquipmentSlot::Boots,
        EquipmentSlot::Amulet,
        EquipmentSlot::Ring,
    ];

    let (base_total, enhanced_total) =
        slot_order
            .iter()
            .enumerate()
            .fold((0u32, 0u32), |(base, enh), (idx, slot)| {
                if let Some(item) = game_state.equipment.get(*slot).as_ref() {
                    let bp = item.power();
                    let mult = crate::enhancement::enhancement_multiplier(enhancement_levels[idx]);
                    let ep = (bp as f64 * mult).round() as u32;
                    (base + bp, enh + ep)
                } else {
                    (base, enh)
                }
            });
    let enh_total_bonus = enhanced_total.saturating_sub(base_total);
    let title = if enhanced_total > 0 {
        if enh_total_bonus > 0 {
            format!(" Equipment \u{26A1}{}+{} ", base_total, enh_total_bonus)
        } else {
            format!(" Equipment \u{26A1}{} ", base_total)
        }
    } else {
        " Equipment ".to_string()
    };
    let block = Block::default().borders(Borders::ALL).title(title);
    let block = super::themed_block(block);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    super::apply_themed_border_fx(frame, area, Color::White, super::BorderFxContext);

    let width = inner.width as usize;
    let slot_col = 8; // "Weapon  " = 8 chars
    // Compute right columns dynamically to fit actual power values.
    // Layout per row: "{:>9}  T{t}  Z{z} ⚡{pow}[+{bonus}]"
    let digit_count = |n: u32| -> usize {
        if n == 0 {
            return 1;
        }
        let mut count = 0;
        let mut v = n;
        while v > 0 {
            count += 1;
            v /= 10;
        }
        count
    };
    let right_cols = slot_order
        .iter()
        .enumerate()
        .filter_map(|(idx, slot)| {
            let item = game_state.equipment.get(*slot).as_ref()?;
            let bp = item.power();
            let mult = crate::enhancement::enhancement_multiplier(enhancement_levels[idx]);
            let eb = (bp as f64 * mult).round() as u32 - bp;
            // Display-cell widths:
            //   rarity {:>9} = 9, "  T{}" = 3+digits, "  Z{}" = 3+digits,
            //   " ⚡{}" = 2+digits (⚡ = 1 cell), "+{}" = 1+digits if bonus > 0
            let mut w = 9
                + 3
                + digit_count(item.tier as u32)
                + 3
                + digit_count(item.ilvl / 10)
                + 2
                + digit_count(bp);
            if eb > 0 {
                w += 1 + digit_count(eb);
            }
            Some(w)
        })
        .max()
        .unwrap_or(18);
    // Name gets whatever remains
    let name_max = width.saturating_sub(slot_col + right_cols);

    let mut lines = Vec::new();

    for (idx, slot_enum) in slot_order.iter().enumerate() {
        let item = game_state.equipment.get(*slot_enum);
        let slot_label = slot_enum.name();
        if let Some(item) = item {
            let rarity_color = super::rarity_color(item.rarity);

            let enh_level = enhancement_levels[idx];
            let prefix = crate::enhancement::enhancement_prefix(enh_level);
            let prefix_len = prefix.len();

            let max_name_len = name_max.saturating_sub(prefix_len);
            let item_name = if item.display_name.len() > max_name_len && max_name_len > 3 {
                format!("{}...", &item.display_name[..max_name_len - 3])
            } else {
                item.display_name.clone()
            };
            let name_len = prefix_len + item_name.len();
            let pad = name_max.saturating_sub(name_len);

            let mut spans = vec![Span::styled(
                format!("{:>6}  ", slot_label),
                Style::default().add_modifier(Modifier::BOLD),
            )];
            if !prefix.is_empty() {
                spans.push(Span::styled(prefix, enhancement_style(enh_level)));
            }
            spans.push(Span::styled(item_name, Style::default().fg(rarity_color)));
            spans.push(Span::raw(" ".repeat(pad)));
            spans.push(Span::styled(
                format!("{:>9}", item.rarity.name()),
                Style::default().fg(rarity_color),
            ));
            spans.push(Span::styled(
                format!("  T{}", item.tier),
                Style::default().fg(super::tier_color(item.tier)),
            ));
            spans.push(Span::styled(
                format!("  Z{}", item.ilvl / 10),
                Style::default().fg(Color::DarkGray),
            ));
            let base_power = item.power();
            spans.push(Span::styled(
                format!(" \u{26A1}{}", base_power),
                Style::default().fg(Color::Cyan),
            ));
            let enh_bonus = {
                let mult = crate::enhancement::enhancement_multiplier(enhancement_levels[idx]);
                (base_power as f64 * mult).round() as u32 - base_power
            };
            if enh_bonus > 0 {
                spans.push(Span::styled(
                    format!("+{}", enh_bonus),
                    enhancement_style(enhancement_levels[idx]),
                ));
            }

            lines.push(Line::from(spans));
        } else {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{:>6}  ", slot_label),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled("[Empty]", Style::default().fg(Color::DarkGray)),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);

    // Soulforge visual effects scale with total enhancement level.
    let total_enh: u16 = enhancement_levels.iter().map(|&l| l as u16).sum();
    if total_enh > 0 {
        paint_soulforge_bg(frame, inner, enhancement_levels);
        paint_soulforge_heat_line(frame, inner, enhancement_levels);
        paint_soulforge_motes(frame, inner, total_enh, enhancement_levels);
    }
}

/// Returns a dimmed version of the soulforge color for the given enhancement levels.
/// Uses the highest level's color tier, scaled by average intensity.
fn soulforge_dim_color(enhancement_levels: &[u8; 7], dim: f64) -> (u8, u8, u8) {
    let max_level = enhancement_levels.iter().copied().max().unwrap_or(0);
    let (cr, cg, cb) = crate::enhancement::enhancement_color_rgb(max_level);
    (
        (cr as f64 * dim) as u8,
        (cg as f64 * dim) as u8,
        (cb as f64 * dim) as u8,
    )
}

/// Paints a faint soulforge-colored background tint on the equipment panel.
fn paint_soulforge_bg(frame: &mut Frame, inner: Rect, enhancement_levels: &[u8; 7]) {
    let avg = enhancement_levels.iter().map(|&l| l as f64).sum::<f64>() / 7.0;
    let intensity = (avg / 10.0).min(1.0);
    let dim = 0.03 + intensity * 0.04;
    let (r, g, b) = soulforge_dim_color(enhancement_levels, dim);
    let bg = Color::Rgb(r, g, b);

    let buf = frame.buffer_mut();
    for y in inner.y..inner.y + inner.height {
        for x in inner.x..inner.x + inner.width {
            if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
                cell.set_bg(bg);
            }
        }
    }
}

/// Paints a glowing heat line along the bottom row — the forge source.
/// Faint ember at low levels, bright at max. Motes appear to rise from here.
fn paint_soulforge_heat_line(frame: &mut Frame, inner: Rect, enhancement_levels: &[u8; 7]) {
    if inner.height == 0 {
        return;
    }
    let avg = enhancement_levels.iter().map(|&l| l as f64).sum::<f64>() / 7.0;
    let intensity = (avg / 10.0).min(1.0);
    // Bottom row bg: dim 8% at low, 18% at max
    let dim = 0.08 + intensity * 0.10;
    let (r, g, b) = soulforge_dim_color(enhancement_levels, dim);
    let bg = Color::Rgb(r, g, b);

    let bottom_y = inner.y + inner.height - 1;
    let buf = frame.buffer_mut();
    for x in inner.x..inner.x + inner.width {
        if let Some(cell) = buf.cell_mut(Position::new(x, bottom_y)) {
            cell.set_bg(bg);
        }
    }
    // Second-to-bottom row gets a lighter glow if panel is tall enough
    if inner.height >= 3 {
        let dim2 = 0.05 + intensity * 0.06;
        let (r2, g2, b2) = soulforge_dim_color(enhancement_levels, dim2);
        let bg2 = Color::Rgb(r2, g2, b2);
        let row2_y = inner.y + inner.height - 2;
        for x in inner.x..inner.x + inner.width {
            if let Some(cell) = buf.cell_mut(Position::new(x, row2_y)) {
                cell.set_bg(bg2);
            }
        }
    }
}

/// Selects mote character based on enhancement tier (A: size progression).
fn mote_char_for_tier(seed: u32, avg_level: f64) -> char {
    if avg_level >= 9.0 {
        // +10 tier: ·, •, ✦, *
        match seed % 8 {
            0 => '*',
            1..=2 => '\u{2726}', // ✦
            3..=4 => '\u{2022}', // •
            _ => '\u{00b7}',     // ·
        }
    } else if avg_level >= 7.0 {
        // +8-9 tier: ·, •, ✦
        match seed % 6 {
            0..=1 => '\u{2726}', // ✦
            2..=3 => '\u{2022}', // •
            _ => '\u{00b7}',     // ·
        }
    } else if avg_level >= 4.0 {
        // +5-7 tier: · and •
        if seed.is_multiple_of(3) {
            '\u{2022}' // •
        } else {
            '\u{00b7}' // ·
        }
    } else {
        '\u{00b7}' // · only at low levels
    }
}

/// Computes mote foreground color based on soulforge tier (B: color evolution).
fn mote_color(enhancement_levels: &[u8; 7], brightness: f64) -> Color {
    let max_level = enhancement_levels.iter().copied().max().unwrap_or(0);
    let (cr, cg, cb) = crate::enhancement::enhancement_color_rgb(max_level);
    // Scale the tier color by brightness (0.0-1.0 range)
    let scale = (brightness / 255.0).min(1.0);
    Color::Rgb(
        (cr as f64 * scale * 0.35) as u8,
        (cg as f64 * scale * 0.35) as u8,
        (cb as f64 * scale * 0.35) as u8,
    )
}

/// Paints rising soulforge motes with size progression, color evolution, and trails.
fn paint_soulforge_motes(
    frame: &mut Frame,
    inner: Rect,
    total_enh: u16,
    enhancement_levels: &[u8; 7],
) {
    use super::scene_fx::{current_millis, hash2d};

    let height = inner.height as usize;
    let width = inner.width as usize;
    if height == 0 || width == 0 {
        return;
    }

    let t = (total_enh as f64 / 70.0).min(1.0);
    let avg_level = enhancement_levels.iter().map(|&l| l as f64).sum::<f64>() / 7.0;

    // Density: threshold 90 (sparse) → 12 (dense)
    let threshold = (90.0 - t * 78.0) as u32;
    // Rise speed: 600ms (slow) → 90ms (fast) per row
    let rise_rate = 600.0 - t * 510.0;
    let millis = current_millis() as f64;
    let rise_phase = (millis / rise_rate) as usize;

    // C: trails at high levels (+8+)
    let has_trails = avg_level >= 7.0;

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

            // A: size progression
            let ch = mote_char_for_tier(seed, avg_level);
            // B: color evolution
            let fg = mote_color(enhancement_levels, brightness);

            cell.set_char(ch);
            cell.set_fg(fg);

            // C: trail — place a dimmer dot one row below the mote
            if has_trails && y + 1 < inner.y + inner.height {
                if let Some(trail_cell) = buf.cell_mut(Position::new(x, y + 1)) {
                    if trail_cell.symbol() == " " {
                        trail_cell.set_char('\u{00b7}');
                        trail_cell.set_fg(mote_color(enhancement_levels, brightness * 0.4));
                    }
                }
            }
        }
    }
}
