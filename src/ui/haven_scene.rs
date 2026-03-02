//! Haven skill tree UI rendering.

use super::scene_fx::{
    current_millis, hash2d, lerp_rgb, put_cell, put_text, render_buffer, SceneCell,
};
use crate::core::game_state::GameState;
use crate::haven::{can_afford, tier_cost, Haven, HavenBonusType, HavenRoomId};
use crate::items::EquipmentSlot;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

/// Returns the icon of the highest unlocked Haven achievement, if any.
fn highest_haven_badge(achievements: &crate::achievements::Achievements) -> Option<&'static str> {
    use crate::achievements::AchievementId;

    let haven_achievements = [
        AchievementId::HavenArchitect,
        AchievementId::HavenBuilderII,
        AchievementId::HavenBuilderI,
        AchievementId::HavenDiscovered,
    ];

    for id in haven_achievements {
        if achievements.is_unlocked(id) {
            return crate::achievements::data::get_achievement_def(id).map(|def| def.icon);
        }
    }
    None
}

/// Paint a stone-toned backdrop: slate gradient + drifting dust motes.
fn paint_hearth_backdrop(buffer: &mut [Vec<SceneCell>], millis: u128) {
    let height = buffer.len();
    if height == 0 {
        return;
    }
    let width = buffer[0].len();

    // 1. Background gradient: near-black at top, weathered stone at bottom
    let top_rgb = (6u8, 8u8, 12u8);
    let bottom_rgb = (34u8, 40u8, 48u8);
    for (row, row_cells) in buffer.iter_mut().enumerate() {
        let t = if height <= 1 {
            0.0
        } else {
            row as f64 / (height - 1) as f64
        };
        let rgb = lerp_rgb(top_rgb, bottom_rgb, t);
        let bg = Color::Rgb(rgb.0, rgb.1, rgb.2);
        for cell in row_cells.iter_mut() {
            cell.bg = bg;
        }
    }

    // 2. Slow-drifting dust motes
    let mote_chars: &[char] = &['\u{00b7}', '\u{2022}']; // · •
    let mote_count = 8;
    let mote_speed = 2.1;
    let mote_hot = (152u8, 164u8, 182u8);
    let mote_cool = (78u8, 88u8, 102u8);

    for i in 0..mote_count {
        let seed = hash2d(i, 0);
        let col = (seed as usize) % width;
        let ch = mote_chars[(hash2d(i, 1) as usize) % mote_chars.len()];

        let phase_offset = (seed as f64) * 0.73;
        let pos = (phase_offset + millis as f64 * mote_speed / 1000.0) % height as f64;
        let row = (height - 1) as f64 - pos;

        let t = pos / height.max(1) as f64;
        let rgb = lerp_rgb(mote_hot, mote_cool, t);
        put_cell(
            buffer,
            row as i32,
            col as i32,
            ch,
            Color::Rgb(rgb.0, rgb.1, rgb.2),
        );
    }
}

/// Opening flourish: brief stone sheen and extra dust burst on Haven open.
fn paint_opening_hearth_fx(buffer: &mut [Vec<SceneCell>], millis: u128, open_elapsed_ms: u128) {
    const OPEN_FX_MS: f64 = 650.0;
    if open_elapsed_ms as f64 >= OPEN_FX_MS {
        return;
    }

    let height = buffer.len();
    if height == 0 {
        return;
    }
    let width = buffer[0].len();
    let progress = (open_elapsed_ms as f64 / OPEN_FX_MS).clamp(0.0, 1.0);
    let strength = 1.0 - progress;

    // Add a cool stone sheen from bottom to top.
    for (row, row_cells) in buffer.iter_mut().enumerate() {
        let row_t = if height <= 1 {
            0.0
        } else {
            row as f64 / (height - 1) as f64
        };
        let sheen = (strength * (0.2 + 0.8 * row_t)).clamp(0.0, 1.0);
        for cell in row_cells.iter_mut() {
            let current_rgb = match cell.bg {
                Color::Rgb(r, g, b) => (r, g, b),
                _ => (8, 8, 8),
            };
            let stoned = lerp_rgb(current_rgb, (102, 112, 128), sheen * 0.72);
            cell.bg = Color::Rgb(stoned.0, stoned.1, stoned.2);
        }
    }

    // Extra dust burst that quickly settles into the normal motes.
    let dust_count = (3.0 + strength * 16.0) as usize;
    let dust_chars: &[char] = &['\u{00b7}', '\u{2022}', ':'];
    for i in 0..dust_count {
        let seed = hash2d(i, 97);
        let col = (seed as usize) % width;
        let phase = (seed as f64) * 0.37;
        let rise = (phase + millis as f64 * 0.011) % (height as f64 + 6.0);
        let row = (height as f64 - 1.0 - rise).floor() as i32;
        if row < 0 {
            continue;
        }

        let t = (rise / height.max(1) as f64).clamp(0.0, 1.0);
        let rgb = lerp_rgb((202, 214, 236), (98, 110, 126), t);
        let ch = dust_chars[(hash2d(i, 311) as usize) % dust_chars.len()];
        put_cell(buffer, row, col as i32, ch, Color::Rgb(rgb.0, rgb.1, rgb.2));
    }
}

/// Render the summary bar into a scene buffer at the given row.
fn render_summary_bar(buffer: &mut [Vec<SceneCell>], row: i32, haven: &Haven) {
    let rooms_built = haven.rooms_built();
    let total_rooms = haven.total_rooms();

    let header = format!("Active bonuses ({}/{}): ", rooms_built, total_rooms);
    put_text(buffer, row, 0, &header, Color::White);
    let mut col = header.chars().count() as i32;

    let bonus_types = [
        (HavenBonusType::DamagePercent, "+{}% DMG"),
        (HavenBonusType::XpGainPercent, "+{}% XP"),
        (HavenBonusType::DropRatePercent, "+{}% Drops"),
        (HavenBonusType::CritChancePercent, "+{}% Crit"),
        (HavenBonusType::HpRegenPercent, "+{}% HP Regen"),
        (HavenBonusType::DoubleStrikeChance, "+{}% Double Strike"),
        (HavenBonusType::OfflineXpPercent, "+{}% Offline XP"),
        (HavenBonusType::ChallengeDiscoveryPercent, "+{}% Discovery"),
    ];

    let mut first = true;
    for (bonus_type, fmt) in bonus_types {
        let value = haven.get_bonus(bonus_type);
        if value > 0.0 {
            if !first {
                put_text(buffer, row, col, "  ", Color::White);
                col += 2;
            }
            let text = fmt.replace("{}", &format!("{:.0}", value));
            put_text(buffer, row, col, &text, Color::Yellow);
            col += text.chars().count() as i32;
            first = false;
        }
    }

    if first {
        put_text(buffer, row, col, "None yet", Color::DarkGray);
    }
}

/// Render a small Haven status indicator (for character select screen)
#[allow(dead_code)] // Planned for compact display mode
pub fn render_haven_indicator(
    frame: &mut Frame,
    area: Rect,
    haven: &Haven,
    _ctx: &super::responsive::LayoutContext,
) {
    if !haven.discovered {
        return; // Don't show anything if Haven not discovered
    }

    // Position in bottom-left corner
    let indicator_width = 30;
    let indicator_height = 2;
    let x = area.x + 2;
    let y = area.y + area.height.saturating_sub(indicator_height + 2);
    let indicator_area = Rect::new(x, y, indicator_width.min(area.width), indicator_height);

    let rooms_built = haven.rooms_built();
    let total_rooms = haven.total_rooms();

    let text = Paragraph::new(vec![Line::from(vec![
        Span::styled("🏠 Haven: ", Style::default().fg(Color::Cyan)),
        Span::styled(
            format!("{}/{} rooms", rooms_built, total_rooms),
            Style::default().fg(Color::White),
        ),
        Span::styled(" [H] View", Style::default().fg(Color::DarkGray)),
    ])]);
    frame.render_widget(text, indicator_area);
}

/// Render the Haven skill tree screen
#[allow(clippy::too_many_arguments)]
pub fn render_haven_tree(
    frame: &mut Frame,
    area: Rect,
    haven: &Haven,
    selected_room: usize,
    open_elapsed_ms: Option<u128>,
    prestige_rank: u32,
    achievements: &crate::achievements::Achievements,
    _ctx: &super::responsive::LayoutContext,
) {
    frame.render_widget(Clear, area);

    let haven_title = match highest_haven_badge(achievements) {
        Some(icon) => format!(" Haven {} ", icon),
        None => " Haven ".to_string(),
    };
    let border_color = Color::Rgb(138, 148, 166);
    let block = Block::default()
        .title(haven_title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(super::themed_border_color(border_color)));
    let inner =
        super::render_themed_block(frame, area, block, border_color, super::BorderFxContext);

    let height = inner.height as usize;
    let width = inner.width as usize;
    if height == 0 || width == 0 {
        return;
    }

    // Create scene buffer and paint hearth backdrop
    let mut buffer = vec![vec![SceneCell::default(); width]; height];
    let millis = current_millis();
    paint_hearth_backdrop(&mut buffer, millis);
    if let Some(elapsed) = open_elapsed_ms {
        paint_opening_hearth_fx(&mut buffer, millis, elapsed);
    }

    // Layout: summary (row 0-1), main content, help (last row)
    let summary_rows = 2usize;
    let help_row = (height - 1) as i32;
    let content_top = summary_rows as i32;
    let content_height = height.saturating_sub(summary_rows + 1);

    // Summary bar
    render_summary_bar(&mut buffer, 0, haven);

    // Main content: skill tree (40%) on left, room detail (60%) on right
    let tree_width = (width * 40 / 100).max(10).min(width);
    let detail_left = tree_width as i32;
    let detail_width = width.saturating_sub(tree_width);

    super::haven_tree::render_skill_tree(
        &mut buffer,
        0,
        content_top,
        tree_width,
        content_height,
        haven,
        selected_room,
    );
    super::haven_details::render_room_detail(
        &mut buffer,
        detail_left,
        content_top,
        detail_width,
        content_height,
        haven,
        selected_room,
        prestige_rank,
        achievements,
    );

    // Help bar
    put_text(
        &mut buffer,
        help_row,
        0,
        "[\u{2191}/\u{2193}] Navigate  [Enter] Build/Forge  [Esc] Close",
        Color::DarkGray,
    );

    // Flush buffer to frame
    render_buffer(frame, inner, &buffer);

    // Apply the currently selected border style accents to the internal panels.
    let tree_area = Rect::new(
        inner.x,
        inner.y + content_top as u16,
        tree_width as u16,
        content_height as u16,
    );
    let detail_area = Rect::new(
        inner.x + detail_left as u16,
        inner.y + content_top as u16,
        detail_width as u16,
        content_height as u16,
    );
    super::apply_themed_border_fx(frame, tree_area, Color::DarkGray, super::BorderFxContext);
    let detail_border_color = if HavenRoomId::ALL
        .get(selected_room)
        .is_some_and(|room| haven.is_room_unlocked(*room))
    {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    super::apply_themed_border_fx(
        frame,
        detail_area,
        detail_border_color,
        super::BorderFxContext,
    );
}

/// Render the Haven discovery modal
pub fn render_haven_discovery_modal(
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
        .border_style(Style::default().fg(super::themed_border_color(Color::Yellow)));
    let inner = super::render_themed_block(
        frame,
        modal_area,
        block,
        Color::Yellow,
        super::BorderFxContext,
    );

    let text = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "\u{1f3e0} You discovered a Haven!",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Through the trees, a clearing opens up.",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::ITALIC),
        )),
        Line::from(Span::styled(
            "Ancient stones hum with quiet power.",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::ITALIC),
        )),
        Line::from(Span::styled(
            "This place could become a stronghold.",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::ITALIC),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press [H] to visit. [Enter] to dismiss.",
            Style::default().fg(Color::DarkGray),
        )),
    ])
    .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(text, inner);
}

/// Render the build confirmation overlay
pub fn render_build_confirmation(
    frame: &mut Frame,
    area: Rect,
    room: HavenRoomId,
    haven: &Haven,
    prestige_rank: u32,
    _ctx: &super::responsive::LayoutContext,
) {
    // Center the modal
    let modal_width = 45u16.min(area.width.saturating_sub(4));
    let modal_height = 9u16.min(area.height.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    frame.render_widget(Clear, modal_area);

    let tier = haven.room_tier(room);
    let next_tier = tier + 1;
    let cost = tier_cost(room, next_tier);
    let can_afford_it = can_afford(room, haven, prestige_rank);

    let title = if tier == 0 {
        format!(" Build {}? ", room.name())
    } else {
        format!(" Upgrade {} to T{}? ", room.name(), next_tier)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(super::themed_border_color(Color::Yellow)));
    let inner = super::render_themed_block(
        frame,
        modal_area,
        block,
        Color::Yellow,
        super::BorderFxContext,
    );

    let cost_style = if can_afford_it {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Red)
    };

    let text = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Cost: ", Style::default().fg(Color::White)),
            Span::styled(format!("{} Prestige Ranks", cost), cost_style),
        ]),
        Line::from(vec![
            Span::styled("Bonus: ", Style::default().fg(Color::White)),
            Span::styled(
                room.format_bonus(next_tier),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(""),
        Line::from(if can_afford_it {
            Span::styled(
                "[Enter] Confirm  [Esc] Cancel",
                Style::default().fg(Color::DarkGray),
            )
        } else {
            Span::styled("Insufficient resources", Style::default().fg(Color::Red))
        }),
    ])
    .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(text, inner);
}

/// Render the Storm Forge confirmation overlay
pub fn render_forge_confirmation(
    frame: &mut Frame,
    area: Rect,
    achievements: &crate::achievements::Achievements,
    prestige_rank: u32,
    _ctx: &super::responsive::LayoutContext,
) {
    // Center the modal
    let modal_width = 50u16.min(area.width.saturating_sub(4));
    let modal_height = 12u16.min(area.height.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(" Forge Stormbreaker? ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(super::themed_border_color(Color::Yellow)));
    let inner = super::render_themed_block(
        frame,
        modal_area,
        block,
        Color::Yellow,
        super::BorderFxContext,
    );

    let (has_leviathan, has_prestige, can_forge) =
        crate::haven::can_forge_stormbreaker(achievements, prestige_rank);

    let leviathan_check = if has_leviathan { "✓" } else { "✗" };
    let leviathan_style = if has_leviathan {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Red)
    };

    let prestige_after = prestige_rank.saturating_sub(25);

    let text = Paragraph::new(vec![
        Line::from(""),
        // "Requires:" section - Cyan header (not consumed)
        Line::from(Span::styled(
            "Requires:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(format!("  {} ", leviathan_check), leviathan_style),
            Span::styled("Storm Leviathan caught", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        // "Cost:" section - Yellow header (will be spent)
        Line::from(Span::styled(
            "Cost:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![Span::styled(
            format!(
                "  ⚡ 25 Prestige Ranks ({} → {})",
                prestige_rank, prestige_after
            ),
            if has_prestige {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Red)
            },
        )]),
        Line::from(""),
        Line::from(Span::styled(
            "Grants access to Zone 10's final boss",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(if can_forge {
            Span::styled(
                "[Enter] Forge  [Esc] Cancel",
                Style::default().fg(Color::DarkGray),
            )
        } else {
            Span::styled(
                "Requirements not met  [Esc] Cancel",
                Style::default().fg(Color::Red),
            )
        }),
    ])
    .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(text, inner);
}

/// Render the Vault item selection screen (shown during prestige when Vault is built)
#[allow(clippy::too_many_arguments)]
pub fn render_vault_selection(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    vault_slots: u8,
    selected_index: usize,
    selected_items: &[EquipmentSlot],
    enhancement_levels: &[u8; 7],
    confirm_pending: bool,
    _ctx: &super::responsive::LayoutContext,
) {
    // Center the vault as a modal: 7 item rows + 2 instruction + 1 help + 2 border + 1 gap = 13
    let modal_height = 13_u16;
    // Enough width for full item detail columns
    let modal_width = area.width.min(78);
    let modal_x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let modal_y = area.y + (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(modal_x, modal_y, modal_width, modal_height);

    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(format!(
            " Vault \u{2014} Preserve {} Item{} ",
            vault_slots,
            if vault_slots == 1 { "" } else { "s" }
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(super::themed_border_color(Color::Yellow)));
    let inner = super::render_themed_block(
        frame,
        modal_area,
        block,
        Color::Yellow,
        super::BorderFxContext,
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Instructions
            Constraint::Min(0),    // Item list
            Constraint::Length(1), // Help
        ])
        .split(inner);

    // Instructions
    let instructions = Paragraph::new(vec![Line::from(Span::styled(
        format!(
            " Select items to keep through prestige. ({}/{} selected)",
            selected_items.len(),
            vault_slots
        ),
        Style::default().fg(Color::White),
    ))]);
    frame.render_widget(instructions, chunks[0]);

    // Column layout matching equipment panel:
    // "▶ [✓] Weapon  +5 Name...       Legendary  T9  Z10  ⚡999"
    //  ^cursor(2) ^check(4) ^slot(8)  ^name(flex)  ^right_cols(27)
    let width = inner.width as usize;
    let cursor_col = 2; // "▶ " or "  "
    let check_col = 4; // "[✓] " or "[ ] "
    let slot_col = 8; // "Weapon  "
                      // Right-side: " Legendary  T9  Z10 ⚡999" = 27 chars
    let right_cols = 27;
    let left_fixed = cursor_col + check_col + slot_col;
    let name_max = width.saturating_sub(left_fixed + right_cols);

    let slots = [
        EquipmentSlot::Weapon,
        EquipmentSlot::Armor,
        EquipmentSlot::Helmet,
        EquipmentSlot::Gloves,
        EquipmentSlot::Boots,
        EquipmentSlot::Amulet,
        EquipmentSlot::Ring,
    ];

    let items: Vec<ListItem> = slots
        .iter()
        .enumerate()
        .map(|(i, slot)| {
            let item = game_state.equipment.get(*slot);
            let is_selected = i == selected_index;
            let is_preserved = selected_items.contains(slot);

            let prefix = if is_selected { "\u{25b6} " } else { "  " };
            let checkbox = if is_preserved { "[\u{2713}] " } else { "[ ] " };

            let prefix_style = if is_selected {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            };
            let checkbox_style = if is_preserved {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            if let Some(item) = item.as_ref() {
                let rc = super::rarity_color(item.rarity);
                let enh_level = enhancement_levels[i];
                let enh_prefix = crate::enhancement::enhancement_prefix(enh_level);
                let prefix_len = enh_prefix.len();

                // Truncate name if needed (matching equipment panel pattern)
                let max_name_len = name_max.saturating_sub(prefix_len);
                let item_name = if item.display_name.len() > max_name_len && max_name_len > 3 {
                    format!("{}...", &item.display_name[..max_name_len - 3])
                } else {
                    item.display_name.clone()
                };
                let name_len = prefix_len + item_name.len();
                let pad = name_max.saturating_sub(name_len);

                let mut spans = vec![
                    Span::styled(prefix, prefix_style),
                    Span::styled(checkbox, checkbox_style),
                    Span::styled(
                        format!("{:>6}  ", slot.name()),
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    ),
                ];
                if !enh_prefix.is_empty() {
                    spans.push(Span::styled(
                        enh_prefix,
                        super::stats_panel::enhancement_style(enh_level),
                    ));
                }
                spans.push(Span::styled(item_name, Style::default().fg(rc)));
                spans.push(Span::raw(" ".repeat(pad)));
                spans.push(Span::styled(
                    format!("{:>9}", item.rarity.name()),
                    Style::default().fg(rc),
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
                    format!(" \u{26a1}{}", base_power),
                    Style::default().fg(Color::Cyan),
                ));
                let enh_bonus = {
                    let mult = crate::enhancement::enhancement_multiplier(enhancement_levels[i]);
                    (base_power as f64 * mult).round() as u32 - base_power
                };
                if enh_bonus > 0 {
                    spans.push(Span::styled(
                        format!("+{}", enh_bonus),
                        super::stats_equipment::enhancement_style(enhancement_levels[i]),
                    ));
                }

                ListItem::new(Line::from(spans))
            } else {
                ListItem::new(Line::from(vec![
                    Span::styled(prefix, prefix_style),
                    Span::styled(checkbox, checkbox_style),
                    Span::styled(
                        format!("{:>6}  ", slot.name()),
                        Style::default()
                            .fg(Color::DarkGray)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("[Empty]", Style::default().fg(Color::DarkGray)),
                ]))
            }
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, chunks[1]);

    // Help bar
    let help =
        Paragraph::new("[↑/↓] Navigate  [Space] Toggle  [Enter] Confirm Prestige  [Esc] Cancel")
            .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[2]);

    // Under-selection confirmation modal (layered on top)
    if confirm_pending {
        let selected = selected_items.len();

        let confirm_width = 42_u16;
        let confirm_height = 7_u16;
        let confirm_x = modal_area.x + (modal_area.width.saturating_sub(confirm_width)) / 2;
        let confirm_y = modal_area.y + (modal_area.height.saturating_sub(confirm_height)) / 2;
        let confirm_area = Rect::new(confirm_x, confirm_y, confirm_width, confirm_height);

        frame.render_widget(Clear, confirm_area);

        let confirm_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(super::themed_border_color(Color::Yellow)));
        let confirm_inner = super::render_themed_block(
            frame,
            confirm_area,
            confirm_block,
            Color::Yellow,
            super::BorderFxContext,
        );

        let lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                format!(
                    "  Only {} of {} vault slot{} used.",
                    selected,
                    vault_slots,
                    if vault_slots == 1 { "" } else { "s" }
                ),
                Style::default().fg(Color::Yellow),
            )),
            Line::from(Span::styled(
                "  Remaining equipment will be lost.",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("  [Enter] Prestige   ", Style::default().fg(Color::Yellow)),
                Span::styled("[Esc] Go Back", Style::default().fg(Color::DarkGray)),
            ]),
        ];

        let confirm_text = Paragraph::new(lines);
        frame.render_widget(confirm_text, confirm_inner);
    }
}
