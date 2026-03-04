//! Equipment rendering helpers for the stats panel.

use crate::core::game_state::GameState;
use crate::stormglass::sigils::StormSigils;
use ratatui::{
    layout::{Constraint, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
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
/// When storm sigils are etched, renders a sigil subsection below the equipment slots.
pub(super) fn draw_equipment_names_only(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    enhancement_levels: &[u8; 7],
    storm_sigils: &StormSigils,
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
    let has_sigils = storm_sigils.etched_count() > 0;
    let sigil_slot_count = crate::stormglass::sigils::MAX_SIGIL_SLOTS as u16;

    // Compute content height so the bordered block doesn't extend into empty space.
    // 7 equipment slots + (optional: 1 separator + 5 sigil slots) + 2 for borders
    let content_height: u16 = 7 + if has_sigils { 1 + sigil_slot_count } else { 0 } + 2;
    let block_height = content_height.min(area.height);
    let block_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: block_height,
    };

    let block = Block::default().borders(Borders::ALL).title(title);
    let block = super::themed_block(block);
    let inner = block.inner(block_area);
    frame.render_widget(block, block_area);
    super::apply_themed_border_fx(frame, block_area, Color::White, super::BorderFxContext);

    // Split inner area: equipment slots, optional titled separator + sigil slots
    let (equip_area, sigil_area) = if has_sigils {
        let chunks = Layout::vertical([
            Constraint::Length(7),                // 7 equipment slots
            Constraint::Length(1), // titled separator: ──── ᚱ Storm Sigils (N/5) ────
            Constraint::Length(sigil_slot_count), // 5 sigil slot lines
        ])
        .split(inner);
        (chunks[0], Some((chunks[1], chunks[2])))
    } else {
        (inner, None)
    };

    let mut equip_rows = Vec::new();

    for (idx, slot_enum) in slot_order.iter().enumerate() {
        let item = game_state.equipment.get(*slot_enum);
        let slot_label = slot_enum.name();

        if let Some(item) = item {
            let rarity_color = super::rarity_color(item.rarity);
            let enh_level = enhancement_levels[idx];
            let prefix = crate::enhancement::enhancement_prefix(enh_level);

            // Name cell: enhancement prefix + item name
            let mut name_spans = Vec::new();
            if !prefix.is_empty() {
                name_spans.push(Span::styled(prefix, enhancement_style(enh_level)));
            }
            name_spans.push(Span::styled(
                item.display_name.clone(),
                Style::default().fg(rarity_color),
            ));

            // Power cell: ⚡base[+bonus]
            let base_power = item.power();
            let enh_mult = crate::enhancement::enhancement_multiplier(enh_level);
            let enh_bonus = (base_power as f64 * enh_mult).round() as u32 - base_power;
            let mut power_spans = vec![Span::styled(
                format!("\u{26A1}{}", base_power),
                Style::default().fg(Color::Cyan),
            )];
            if enh_bonus > 0 {
                power_spans.push(Span::styled(
                    format!("+{}", enh_bonus),
                    enhancement_style(enh_level),
                ));
            }

            equip_rows.push(Row::new([
                Cell::from(Span::styled(
                    format!("{:>6}", slot_label),
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Cell::from(Line::from(name_spans)),
                Cell::from(Span::styled(
                    format!("{:>9}", item.rarity.name()),
                    Style::default().fg(rarity_color),
                )),
                Cell::from(Span::styled(
                    format!("T{}", item.tier),
                    Style::default().fg(super::tier_color(item.tier)),
                )),
                Cell::from(Span::styled(
                    format!("Z{}", item.ilvl / 10),
                    Style::default().fg(Color::DarkGray),
                )),
                Cell::from(Line::from(power_spans)),
            ]));
        } else {
            equip_rows.push(Row::new([
                Cell::from(Span::styled(
                    format!("{:>6}", slot_label),
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Cell::from(Span::styled(
                    "[Empty]",
                    Style::default().fg(Color::DarkGray),
                )),
                Cell::default(),
                Cell::default(),
                Cell::default(),
                Cell::default(),
            ]));
        }
    }

    let equip_widths = [
        Constraint::Length(6),  // Slot
        Constraint::Min(4),     // Name (fills remaining)
        Constraint::Length(9),  // Rarity
        Constraint::Length(2),  // Tier
        Constraint::Length(3),  // Zone
        Constraint::Length(10), // Power
    ];
    let equip_table = Table::new(equip_rows, equip_widths).column_spacing(1);
    frame.render_widget(equip_table, equip_area);

    // Soulforge visual effects scale with total enhancement level.
    // Apply only to the equipment rows (not the sigil section).
    let total_enh: u16 = enhancement_levels.iter().map(|&l| l as u16).sum();
    if total_enh > 0 {
        paint_soulforge_bg(frame, equip_area, enhancement_levels);
        paint_soulforge_heat_line(frame, equip_area, enhancement_levels);
        paint_soulforge_motes(frame, equip_area, total_enh, enhancement_levels);
    }

    // Render sigil subsection if sigils are etched
    if let Some((separator_area, sigils_area)) = sigil_area {
        draw_sigil_separator(frame, separator_area, storm_sigils);
        draw_sigil_slots(frame, sigils_area, storm_sigils);

        // Sigil visual effects — scale with total grade score.
        let grade_total = sigil_grade_total(storm_sigils);
        if grade_total > 0 {
            paint_sigil_bg(frame, sigils_area, grade_total);
            paint_sigil_heat_line(frame, sigils_area, grade_total);
            paint_sigil_motes(frame, sigils_area, grade_total);
        }
    }
}

/// Draws a titled separator: `──── ᚱ Storm Sigils (N/5) ────`
/// Matches the style used for Prestige and Attributes headers in the hero panel.
fn draw_sigil_separator(frame: &mut Frame, area: Rect, storm_sigils: &StormSigils) {
    let dw = super::scene_fx::display_width;
    let etched = storm_sigils.etched_count();
    let label = format!(
        " \u{16B1} Storm Sigils ({}/{}) ",
        etched, storm_sigils.slots_unlocked
    );
    let label_w = dw(&label);
    let total_w = area.width as usize;
    let remaining = total_w.saturating_sub(label_w);
    let left_dashes = remaining / 2;
    let right_dashes = remaining - left_dashes;
    let storm_blue = Color::Rgb(100, 180, 255);
    let sep = Paragraph::new(Line::from(vec![
        Span::styled(
            "\u{2500}".repeat(left_dashes),
            Style::default().fg(storm_blue),
        ),
        Span::styled(
            label,
            Style::default().fg(storm_blue).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "\u{2500}".repeat(right_dashes),
            Style::default().fg(storm_blue),
        ),
    ]));
    frame.render_widget(sep, area);
}

/// Draws the 5 sigil slot lines within the equipment panel.
/// Each etched sigil: [Icon+Name] [LineGauge with value label] [Grade]
fn draw_sigil_slots(frame: &mut Frame, area: Rect, storm_sigils: &StormSigils) {
    use ratatui::widgets::LineGauge;

    let row_constraints: Vec<Constraint> = storm_sigils
        .sigils
        .iter()
        .map(|_| Constraint::Length(1))
        .collect();
    let rows = Layout::vertical(row_constraints).split(area);

    for (i, slot) in storm_sigils.sigils.iter().enumerate() {
        if i >= rows.len() {
            break;
        }
        let row_area = rows[i];

        if i < storm_sigils.slots_unlocked as usize {
            if let Some(sigil) = slot {
                let icon = sigil.effect.icon();
                let short = sigil.effect.short_name();
                let grade_str = sigil.grade.label();
                let grade_color = sigil_grade_color(sigil.grade);

                let grade_style = if grade_str.ends_with('+') {
                    Style::default()
                        .fg(grade_color)
                        .add_modifier(Modifier::BOLD)
                } else if grade_str.ends_with('-') {
                    Style::default().fg(grade_color).add_modifier(Modifier::DIM)
                } else {
                    Style::default().fg(grade_color)
                };

                let (min_val, max_val) = sigil.effect.range();
                let ratio = if max_val > min_val {
                    ((sigil.value - min_val) / (max_val - min_val)).clamp(0.0, 1.0)
                } else {
                    1.0
                };
                let value_label = if sigil.effect
                    == crate::stormglass::sigils::SigilEffectType::RegenDelayPercent
                {
                    format!("-{:.1}%", sigil.value)
                } else {
                    format!("+{:.1}%", sigil.value)
                };

                // Split row: [name 14] [gauge fills] [value 7] [grade 3]
                let cols = Layout::horizontal([
                    Constraint::Length(14),
                    Constraint::Min(6),
                    Constraint::Length(7),
                    Constraint::Length(3),
                ])
                .split(row_area);

                let name_para = Paragraph::new(Span::styled(
                    format!("{} {}", icon, short),
                    Style::default().fg(Color::White),
                ));
                frame.render_widget(name_para, cols[0]);

                let gauge = LineGauge::default()
                    .filled_style(
                        Style::default()
                            .fg(grade_color)
                            .add_modifier(Modifier::BOLD),
                    )
                    .unfilled_style(Style::default().fg(Color::DarkGray))
                    .label("")
                    .ratio(ratio);
                frame.render_widget(gauge, cols[1]);

                let value_para = Paragraph::new(Span::styled(
                    format!("{:>7}", value_label),
                    Style::default().fg(Color::White),
                ));
                frame.render_widget(value_para, cols[2]);

                let grade_para =
                    Paragraph::new(Span::styled(format!(" {:<2}", grade_str), grade_style));
                frame.render_widget(grade_para, cols[3]);
            } else {
                let empty_para = Paragraph::new(Span::styled(
                    "\u{00b7} empty",
                    Style::default().fg(Color::DarkGray),
                ));
                frame.render_widget(empty_para, row_area);
            }
        } else {
            let locked_para = Paragraph::new(Span::styled(
                "\u{1f512} locked",
                Style::default().fg(Color::DarkGray),
            ));
            frame.render_widget(locked_para, row_area);
        }
    }
}

/// Returns the color for a sigil grade tier letter.
fn sigil_grade_color(grade: crate::stormglass::sigils::SigilGrade) -> Color {
    match grade.tier_letter() {
        'S' => Color::Rgb(255, 215, 0),
        'A' => Color::Green,
        'B' => Color::Cyan,
        'C' => Color::White,
        'D' => Color::Gray,
        'E' => Color::DarkGray,
        _ => Color::Red,
    }
}

/// Sum of all etched sigil grade ordinals. F-=0, S+=20. Max = 5x20 = 100.
fn sigil_grade_total(storm_sigils: &StormSigils) -> u16 {
    storm_sigils
        .sigils
        .iter()
        .filter_map(|s| s.as_ref())
        .map(|s| s.grade as u16)
        .sum()
}

// --- Soulforge visual effects (equipment section only) ---

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

    // Density: threshold 90 (sparse) -> 12 (dense)
    let threshold = (90.0 - t * 78.0) as u32;
    // Rise speed: 600ms (slow) -> 90ms (fast) per row
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

// --- Storm sigil visual effects (sigil section only) ---

/// Storm-blue color dimmed by a factor. Used for bg tint and heat line.
fn storm_dim_color(grade_total: u16, dim: f64) -> Color {
    // Blend from cool blue at low grades to bright electric blue at high grades
    let t = (grade_total as f64 / 100.0).min(1.0);
    let r = (60.0 + t * 40.0) * dim;
    let g = (140.0 + t * 40.0) * dim;
    let b = (220.0 + t * 35.0) * dim;
    Color::Rgb(r as u8, g as u8, b as u8)
}

/// Faint storm-blue background tint scaling with grade total.
fn paint_sigil_bg(frame: &mut Frame, inner: Rect, grade_total: u16) {
    let t = (grade_total as f64 / 100.0).min(1.0);
    let dim = 0.03 + t * 0.04;
    let bg = storm_dim_color(grade_total, dim);

    let buf = frame.buffer_mut();
    for y in inner.y..inner.y + inner.height {
        for x in inner.x..inner.x + inner.width {
            if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
                cell.set_bg(bg);
            }
        }
    }
}

/// Glowing storm-blue heat line along the bottom row.
fn paint_sigil_heat_line(frame: &mut Frame, inner: Rect, grade_total: u16) {
    if inner.height == 0 {
        return;
    }
    let t = (grade_total as f64 / 100.0).min(1.0);
    let bg = storm_dim_color(grade_total, 0.08 + t * 0.10);

    let bottom_y = inner.y + inner.height - 1;
    let buf = frame.buffer_mut();
    for x in inner.x..inner.x + inner.width {
        if let Some(cell) = buf.cell_mut(Position::new(x, bottom_y)) {
            cell.set_bg(bg);
        }
    }
    if inner.height >= 3 {
        let bg2 = storm_dim_color(grade_total, 0.05 + t * 0.06);
        let row2_y = inner.y + inner.height - 2;
        for x in inner.x..inner.x + inner.width {
            if let Some(cell) = buf.cell_mut(Position::new(x, row2_y)) {
                cell.set_bg(bg2);
            }
        }
    }
}

/// Rising storm motes with size/color/trail progression.
fn paint_sigil_motes(frame: &mut Frame, inner: Rect, grade_total: u16) {
    use super::scene_fx::{current_millis, hash2d};

    let height = inner.height as usize;
    let width = inner.width as usize;
    if height == 0 || width == 0 {
        return;
    }

    // Scale 0-100 into 0.0-1.0
    let t = (grade_total as f64 / 100.0).min(1.0);

    let threshold = (90.0 - t * 78.0) as u32;
    let rise_rate = 600.0 - t * 510.0;
    let millis = current_millis() as f64;
    let rise_phase = (millis / rise_rate) as usize;
    let has_trails = t >= 0.7;

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
            let scale = (brightness / 255.0).min(1.0) * 0.35;

            // Size progression: small at low, larger at high
            let ch = if t >= 0.9 {
                match seed % 8 {
                    0 => '*',
                    1..=2 => '\u{2726}',
                    3..=4 => '\u{2022}',
                    _ => '\u{00b7}',
                }
            } else if t >= 0.7 {
                match seed % 6 {
                    0..=1 => '\u{2726}',
                    2..=3 => '\u{2022}',
                    _ => '\u{00b7}',
                }
            } else if t >= 0.4 {
                if seed.is_multiple_of(3) {
                    '\u{2022}'
                } else {
                    '\u{00b7}'
                }
            } else {
                '\u{00b7}'
            };

            // Storm-blue color
            let fg = Color::Rgb(
                (60.0 * scale) as u8,
                (180.0 * scale) as u8,
                (255.0 * scale) as u8,
            );
            cell.set_char(ch);
            cell.set_fg(fg);

            // Trail at high grades
            if has_trails && y + 1 < inner.y + inner.height {
                if let Some(trail_cell) = buf.cell_mut(Position::new(x, y + 1)) {
                    if trail_cell.symbol() == " " {
                        let trail_scale = scale * 0.4;
                        trail_cell.set_char('\u{00b7}');
                        trail_cell.set_fg(Color::Rgb(
                            (60.0 * trail_scale) as u8,
                            (180.0 * trail_scale) as u8,
                            (255.0 * trail_scale) as u8,
                        ));
                    }
                }
            }
        }
    }
}
