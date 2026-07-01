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
    // Widest ⚡base[+bonus] string across slots; the column sizes to fit so
    // 4-digit bonuses don't get truncated (issue #601).
    let mut power_col_width: u16 = 0;

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
            let base_text = format!("\u{26A1}{}", base_power);
            let bonus_text = if enh_bonus > 0 {
                format!("+{}", enh_bonus)
            } else {
                String::new()
            };
            let cell_width = (super::scene_fx::display_width(&base_text) + bonus_text.len()) as u16;
            power_col_width = power_col_width.max(cell_width);
            let mut power_spans = vec![Span::styled(base_text, Style::default().fg(Color::Cyan))];
            if !bonus_text.is_empty() {
                power_spans.push(Span::styled(bonus_text, enhancement_style(enh_level)));
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
        Constraint::Length(6),               // Slot
        Constraint::Min(4),                  // Name (fills remaining)
        Constraint::Length(9),               // Rarity
        Constraint::Length(2),               // Tier
        Constraint::Length(3),               // Zone
        Constraint::Length(power_col_width), // Power (sized to widest ⚡base+bonus)
    ];
    let equip_table = Table::new(equip_rows, equip_widths).column_spacing(1);
    frame.render_widget(equip_table, equip_area);

    // Soulforge visual effects scale with total enhancement level.
    // Apply only to the equipment rows (not the sigil section).
    let total_enh: u16 = enhancement_levels.iter().map(|&l| l as u16).sum();
    if total_enh > 0 {
        paint_soulforge_gradient(frame, equip_area, enhancement_levels);
        paint_soulforge_motes(frame, equip_area, total_enh, enhancement_levels);
    }

    // Render sigil subsection if sigils are etched
    if let Some((separator_area, sigils_area)) = sigil_area {
        draw_sigil_separator(frame, separator_area, storm_sigils);
        draw_sigil_slots(frame, sigils_area, storm_sigils);

        // Sigil visual effects — scale with total grade score.
        let grade_total = sigil_grade_total(storm_sigils);
        if grade_total > 0 {
            paint_sigil_gradient(frame, sigils_area, grade_total);
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

// --- Shared HalfBlock gradient helper ---

/// Applies a vertical gradient to an area using HalfBlock pixel resolution.
/// Empty cells get `▀` with fg=top pixel, bg=bottom pixel (2x vertical resolution).
/// Text cells get bg set to the bottom pixel color (preserves readability).
fn paint_halfblock_gradient(
    frame: &mut Frame,
    area: Rect,
    gradient_fn: impl Fn(usize) -> (u8, u8, u8),
) {
    let total_py = area.height as usize * 2;
    if total_py == 0 {
        return;
    }

    let buf = frame.buffer_mut();
    for y in area.y..area.y + area.height {
        let py_top = ((y - area.y) as usize) * 2;
        let py_bot = (py_top + 1).min(total_py - 1);
        let c_top = gradient_fn(py_top);
        let c_bot = gradient_fn(py_bot);
        let fg_top = Color::Rgb(c_top.0, c_top.1, c_top.2);
        let bg_bot = Color::Rgb(c_bot.0, c_bot.1, c_bot.2);

        for x in area.x..area.x + area.width {
            if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
                if cell.symbol() == " " || cell.symbol() == "\u{2580}" {
                    cell.set_char('\u{2580}'); // ▀
                    cell.set_fg(fg_top);
                    cell.set_bg(bg_bot);
                } else {
                    cell.set_bg(bg_bot);
                }
            }
        }
    }
}

// --- Soulforge visual effects (equipment section only) ---

/// Soulforge gradient: dark at top, ember glow at bottom. 2x vertical resolution.
fn paint_soulforge_gradient(frame: &mut Frame, inner: Rect, enhancement_levels: &[u8; 7]) {
    let max_level = enhancement_levels.iter().copied().max().unwrap_or(0);
    let (cr, cg, cb) = crate::enhancement::enhancement_color_rgb(max_level);
    let avg = enhancement_levels.iter().map(|&l| l as f64).sum::<f64>() / 7.0;
    let intensity = (avg / 10.0).min(1.0);

    let base_dim = 0.03 + intensity * 0.04;
    let heat_dim = 0.08 + intensity * 0.10;
    let total_py = inner.height as usize * 2;

    paint_halfblock_gradient(frame, inner, |py| {
        let t = if total_py <= 1 {
            0.0
        } else {
            py as f64 / (total_py - 1) as f64
        };
        let dim = base_dim + (heat_dim - base_dim) * t.powf(2.5);
        (
            (cr as f64 * dim) as u8,
            (cg as f64 * dim) as u8,
            (cb as f64 * dim) as u8,
        )
    });
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
            if cell.symbol() != " " && cell.symbol() != "\u{2580}" {
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
                    if trail_cell.symbol() == " " || trail_cell.symbol() == "\u{2580}" {
                        trail_cell.set_char('\u{00b7}');
                        trail_cell.set_fg(mote_color(enhancement_levels, brightness * 0.4));
                    }
                }
            }
        }
    }
}

// --- Storm sigil visual effects (sigil section only) ---

/// Storm sigil gradient: dark at top, storm-blue glow at bottom. 2x vertical resolution.
fn paint_sigil_gradient(frame: &mut Frame, inner: Rect, grade_total: u16) {
    let t = (grade_total as f64 / 100.0).min(1.0);
    let sr = 60.0 + t * 40.0;
    let sg = 140.0 + t * 40.0;
    let sb = 220.0 + t * 35.0;

    let base_dim = 0.03 + t * 0.04;
    let heat_dim = 0.08 + t * 0.10;
    let total_py = inner.height as usize * 2;

    paint_halfblock_gradient(frame, inner, |py| {
        let pt = if total_py <= 1 {
            0.0
        } else {
            py as f64 / (total_py - 1) as f64
        };
        let dim = base_dim + (heat_dim - base_dim) * pt.powf(2.0);
        ((sr * dim) as u8, (sg * dim) as u8, (sb * dim) as u8)
    });
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
            if cell.symbol() != " " && cell.symbol() != "\u{2580}" {
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
                    if trail_cell.symbol() == " " || trail_cell.symbol() == "\u{2580}" {
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
