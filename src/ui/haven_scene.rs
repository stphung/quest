//! Haven skill tree UI rendering.

use super::scene_fx::{current_millis, hash2d, lerp_rgb, put_cell, render_buffer, SceneCell};
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

/// Write a string into the scene buffer at (row, col).
fn put_text(buffer: &mut [Vec<SceneCell>], row: i32, col: i32, text: &str, fg: Color) {
    for (i, ch) in text.chars().enumerate() {
        put_cell(buffer, row, col + i as i32, ch, fg);
    }
}

/// Paint a warm hearth glow backdrop: gentle gradient + slow-drifting motes.
fn paint_hearth_backdrop(buffer: &mut [Vec<SceneCell>], millis: u128) {
    let height = buffer.len();
    if height == 0 {
        return;
    }
    let width = buffer[0].len();

    // 1. Background gradient: near-black at top, warm grey-sage at bottom
    let top_rgb = (8u8, 8u8, 8u8);
    let bottom_rgb = (30u8, 35u8, 28u8);
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

    // 2. Slow-drifting motes (8 particles, ~0.5x Soulforge speed)
    let mote_chars: &[char] = &['\u{00b7}', '\u{2022}']; // · •
    let mote_count = 8;
    let mote_speed = 2.5;
    let mote_hot = (70u8, 85u8, 60u8);
    let mote_cool = (30u8, 35u8, 28u8);

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

/// Render the summary bar into a scene buffer at the given row.
fn render_summary_bar(buffer: &mut [Vec<SceneCell>], row: i32, haven: &Haven) {
    let rooms_built = haven.rooms_built();
    let total_rooms = haven.total_rooms();

    let header = format!("Active bonuses ({}/{} rooms): ", rooms_built, total_rooms);
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

/// Render the skill tree panel into a scene buffer.
fn render_skill_tree(
    buffer: &mut [Vec<SceneCell>],
    left: i32,
    top: i32,
    panel_width: usize,
    panel_height: usize,
    haven: &Haven,
    selected_room: usize,
) {
    let border_fg = Color::DarkGray;
    let right = left + panel_width as i32 - 1;
    let bottom = top + panel_height as i32 - 1;

    // Draw border
    put_cell(buffer, top, left, '\u{250c}', border_fg);
    put_cell(buffer, top, right, '\u{2510}', border_fg);
    put_cell(buffer, bottom, left, '\u{2514}', border_fg);
    put_cell(buffer, bottom, right, '\u{2518}', border_fg);
    for c in (left + 1)..right {
        put_cell(buffer, top, c, '\u{2500}', border_fg);
        put_cell(buffer, bottom, c, '\u{2500}', border_fg);
    }
    for r in (top + 1)..bottom {
        put_cell(buffer, r, left, '\u{2502}', border_fg);
        put_cell(buffer, r, right, '\u{2502}', border_fg);
    }
    put_text(buffer, top, left + 1, " Skill Tree ", border_fg);

    let inner_left = left + 1;
    let content_top = top + 1;

    for (i, room) in HavenRoomId::ALL.iter().enumerate() {
        let row = content_top + i as i32;
        if row >= bottom {
            break;
        }

        let tier = haven.room_tier(*room);
        let unlocked = haven.is_room_unlocked(*room);
        let is_selected = i == selected_room;

        let max_t = room.max_tier();
        let tier_str: String = (1..=max_t)
            .map(|t| if tier >= t { "\u{2605}" } else { "\u{00b7}" })
            .collect::<Vec<_>>()
            .join("");

        let prefix = if is_selected { "\u{25b6} " } else { "  " };

        let indent = match room {
            HavenRoomId::Hearthstone => "",
            HavenRoomId::Armory | HavenRoomId::Bedroom => "  ",
            HavenRoomId::TrainingYard
            | HavenRoomId::TrophyHall
            | HavenRoomId::Garden
            | HavenRoomId::Library => "    ",
            HavenRoomId::Watchtower
            | HavenRoomId::AlchemyLab
            | HavenRoomId::FishingDock
            | HavenRoomId::Workshop => "      ",
            HavenRoomId::WarRoom | HavenRoomId::Vault => "        ",
            HavenRoomId::StormForge => "          ",
        };

        let style_fg = if !unlocked {
            Color::DarkGray
        } else if is_selected {
            Color::Cyan
        } else if tier > 0 {
            Color::Green
        } else {
            Color::White
        };

        let tier_fg = if tier > 0 {
            Color::Yellow
        } else {
            Color::DarkGray
        };

        let lock_indicator = if !unlocked { "\u{2716} " } else { "" };

        let mut col = inner_left;
        put_text(buffer, row, col, prefix, style_fg);
        col += prefix.chars().count() as i32;
        put_text(buffer, row, col, &tier_str, tier_fg);
        col += tier_str.chars().count() as i32;
        put_text(buffer, row, col, " ", style_fg);
        col += 1;
        put_text(buffer, row, col, indent, style_fg);
        col += indent.chars().count() as i32;
        put_text(buffer, row, col, lock_indicator, Color::DarkGray);
        col += lock_indicator.chars().count() as i32;
        put_text(buffer, row, col, room.name(), style_fg);

        // Highlight selected row background
        if is_selected {
            let highlight_bg = Color::Rgb(30, 22, 12);
            let row_usize = row as usize;
            if row_usize < buffer.len() {
                for c in (inner_left as usize)..((right) as usize) {
                    if c < buffer[row_usize].len() {
                        buffer[row_usize][c].bg = highlight_bg;
                    }
                }
            }
        }
    }
}

/// Simple word-wrap: break text into lines that fit within `max_width` characters.
fn word_wrap(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.chars().count() + 1 + word.chars().count() <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    lines
}

/// Render the room detail panel into a scene buffer.
#[allow(clippy::too_many_arguments)]
fn render_room_detail(
    buffer: &mut [Vec<SceneCell>],
    left: i32,
    top: i32,
    panel_width: usize,
    panel_height: usize,
    haven: &Haven,
    selected_room: usize,
    prestige_rank: u32,
    achievements: &crate::achievements::Achievements,
) {
    let room = HavenRoomId::ALL[selected_room];
    let tier = haven.room_tier(room);
    let unlocked = haven.is_room_unlocked(room);

    let border_fg = if unlocked {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let right = left + panel_width as i32 - 1;
    let bottom = top + panel_height as i32 - 1;
    put_cell(buffer, top, left, '\u{250c}', border_fg);
    put_cell(buffer, top, right, '\u{2510}', border_fg);
    put_cell(buffer, bottom, left, '\u{2514}', border_fg);
    put_cell(buffer, bottom, right, '\u{2518}', border_fg);
    for c in (left + 1)..right {
        put_cell(buffer, top, c, '\u{2500}', border_fg);
        put_cell(buffer, bottom, c, '\u{2500}', border_fg);
    }
    for r in (top + 1)..bottom {
        put_cell(buffer, r, left, '\u{2502}', border_fg);
        put_cell(buffer, r, right, '\u{2502}', border_fg);
    }
    let title = format!(" {} ", room.name());
    put_text(buffer, top, left + 1, &title, border_fg);

    let inner_left = left + 1;
    let inner_width = (panel_width - 2).max(1);
    let mut row = top + 1;

    // Description (word-wrapped)
    let desc = room.description();
    let wrapped = word_wrap(desc, inner_width);
    for line in &wrapped {
        if row >= bottom {
            break;
        }
        put_text(buffer, row, inner_left, line, Color::White);
        row += 1;
    }
    row += 1;

    // Bonuses
    if row < bottom {
        put_text(buffer, row, inner_left, "Bonuses:", Color::White);
        row += 1;
    }
    let max_tier = room.max_tier();
    for t in 1..=max_tier {
        if row >= bottom {
            break;
        }
        let is_built = t <= tier;
        let is_next = t == tier + 1 && tier < max_tier;
        let style_fg = if is_built {
            Color::Green
        } else if is_next {
            Color::Yellow
        } else {
            Color::DarkGray
        };
        let marker = if is_next { "\u{25b6} " } else { "  " };
        let tier_label = format!("T{}: ", t);
        let bonus_text = room.format_bonus(t);
        let mut col = inner_left;
        put_text(buffer, row, col, marker, style_fg);
        col += marker.chars().count() as i32;
        put_text(buffer, row, col, &tier_label, Color::DarkGray);
        col += tier_label.chars().count() as i32;
        put_text(buffer, row, col, &bonus_text, style_fg);
        row += 1;
    }
    row += 1;

    // Requirements
    let parents = room.parents();
    if !parents.is_empty() && row < bottom {
        put_text(buffer, row, inner_left, "Requires:", Color::White);
        row += 1;
        for parent in parents {
            if row >= bottom {
                break;
            }
            let parent_tier = haven.room_tier(*parent);
            let is_built = parent_tier > 0;
            let (marker, style_fg) = if is_built {
                ("\u{2713}", Color::Green)
            } else {
                ("\u{2717}", Color::Red)
            };
            let tier_info = if parent_tier > 0 {
                format!(" (T{})", parent_tier)
            } else {
                String::new()
            };
            let mut col = inner_left;
            put_text(buffer, row, col, &format!("  {} ", marker), style_fg);
            col += 4;
            put_text(buffer, row, col, parent.name(), style_fg);
            col += parent.name().chars().count() as i32;
            put_text(buffer, row, col, &tier_info, Color::DarkGray);
            row += 1;
        }
        row += 1;
    }

    // Cost info
    if row >= bottom {
        return;
    }
    if !unlocked {
        put_text(buffer, row, inner_left, "\u{2716} Locked", Color::Red);
        row += 1;
        if row < bottom {
            put_text(
                buffer,
                row,
                inner_left,
                "Build all required rooms first",
                Color::DarkGray,
            );
        }
    } else if tier < room.max_tier() {
        let next_tier = tier + 1;
        let cost = tier_cost(room, next_tier);
        let can_afford_it = can_afford(room, haven, prestige_rank);
        let cost_fg = if can_afford_it {
            Color::Green
        } else {
            Color::Red
        };
        let cost_text = format!("{} Prestige Ranks", cost);
        put_text(buffer, row, inner_left, "Cost: ", Color::DarkGray);
        put_text(buffer, row, inner_left + 6, &cost_text, cost_fg);
        row += 1;
        if row < bottom {
            let have_text = format!("{} Prestige Ranks", prestige_rank);
            put_text(buffer, row, inner_left, "You have: ", Color::DarkGray);
            put_text(buffer, row, inner_left + 10, &have_text, Color::White);
        }
    } else if room == HavenRoomId::StormForge {
        use crate::achievements::AchievementId;
        let has_stormbreaker = achievements.is_unlocked(AchievementId::TheStormbreaker);
        if has_stormbreaker {
            put_text(
                buffer,
                row,
                inner_left,
                "\u{2726} Stormbreaker forged!",
                Color::Yellow,
            );
            row += 1;
            if row < bottom {
                put_text(
                    buffer,
                    row,
                    inner_left,
                    "Zone 10 boss accessible",
                    Color::Green,
                );
            }
        } else {
            put_text(
                buffer,
                row,
                inner_left,
                "Press [Enter] to forge",
                Color::Yellow,
            );
            row += 1;
            if row < bottom {
                put_text(
                    buffer,
                    row,
                    inner_left,
                    "Requires: Storm Leviathan + 25 PR",
                    Color::DarkGray,
                );
            }
        }
    } else {
        put_text(
            buffer,
            row,
            inner_left,
            "\u{2713} Max tier reached",
            Color::Green,
        );
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
pub fn render_haven_tree(
    frame: &mut Frame,
    area: Rect,
    haven: &Haven,
    selected_room: usize,
    prestige_rank: u32,
    achievements: &crate::achievements::Achievements,
    _ctx: &super::responsive::LayoutContext,
) {
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Haven ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let height = inner.height as usize;
    let width = inner.width as usize;
    if height == 0 || width == 0 {
        return;
    }

    // Create scene buffer and paint hearth backdrop
    let mut buffer = vec![vec![SceneCell::default(); width]; height];
    let millis = current_millis();
    paint_hearth_backdrop(&mut buffer, millis);

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

    render_skill_tree(
        &mut buffer,
        0,
        content_top,
        tree_width,
        content_height,
        haven,
        selected_room,
    );
    render_room_detail(
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
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

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
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

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
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

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
pub fn render_vault_selection(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    vault_slots: u8,
    selected_index: usize,
    selected_items: &[EquipmentSlot],
    _ctx: &super::responsive::LayoutContext,
) {
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(
            " Vault - Choose {} Item(s) to Preserve ",
            vault_slots
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    frame.render_widget(block, area);

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
            "Select up to {} item(s) to keep through prestige. ({}/{} selected)",
            vault_slots,
            selected_items.len(),
            vault_slots
        ),
        Style::default().fg(Color::White),
    ))]);
    frame.render_widget(instructions, chunks[0]);

    // Get all equipped items
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

            let prefix = if is_selected { "▶ " } else { "  " };
            let checkbox = if is_preserved { "[✓] " } else { "[ ] " };

            let (slot_name, item_text, style) = if let Some(item) = item.as_ref() {
                let rarity_color = match item.rarity {
                    crate::items::Rarity::Common => Color::White,
                    crate::items::Rarity::Magic => Color::Green,
                    crate::items::Rarity::Rare => Color::Blue,
                    crate::items::Rarity::Epic => Color::Magenta,
                    crate::items::Rarity::Legendary => Color::Yellow,
                    crate::items::Rarity::Mythic => Color::Rgb(255, 215, 0),
                };
                (
                    format!("{:8}", format!("{:?}", slot)),
                    item.display_name.clone(),
                    Style::default().fg(rarity_color),
                )
            } else {
                (
                    format!("{:8}", format!("{:?}", slot)),
                    "(empty)".to_string(),
                    Style::default().fg(Color::DarkGray),
                )
            };

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

            ListItem::new(Line::from(vec![
                Span::styled(prefix, prefix_style),
                Span::styled(checkbox, checkbox_style),
                Span::styled(slot_name, Style::default().fg(Color::DarkGray)),
                Span::styled(item_text, style),
            ]))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, chunks[1]);

    // Help bar
    let help =
        Paragraph::new("[↑/↓] Navigate  [Enter] Toggle  [Space] Confirm Prestige  [Esc] Cancel")
            .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[2]);
}
