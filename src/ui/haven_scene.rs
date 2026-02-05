//! Haven skill tree UI rendering.

use crate::core::game_state::GameState;
use crate::haven::{can_afford, tier_cost, Haven, HavenBonusType, HavenRoomId};
use crate::items::EquipmentSlot;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// Render a small Haven status indicator (for character select screen)
pub fn render_haven_indicator(frame: &mut Frame, area: Rect, haven: &Haven) {
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
    fishing_rank: u32,
) {
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Haven ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Split into summary bar, main content, and help
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Summary bar
            Constraint::Min(0),    // Main content (tree + detail)
            Constraint::Length(1), // Help
        ])
        .split(inner);

    // Summary bar - active bonuses
    render_summary_bar(frame, chunks[0], haven);

    // Main content - tree on left, detail on right
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[1]);

    render_skill_tree(frame, main_chunks[0], haven, selected_room);
    render_room_detail(
        frame,
        main_chunks[1],
        haven,
        selected_room,
        prestige_rank,
        fishing_rank,
    );

    // Help bar
    let help = Paragraph::new("[↑/↓] Navigate  [Enter] Build  [Esc] Close")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, chunks[2]);
}

/// Render the summary bar showing active bonuses
fn render_summary_bar(frame: &mut Frame, area: Rect, haven: &Haven) {
    let rooms_built = haven.rooms_built();
    let total_rooms = haven.total_rooms();

    let mut spans = vec![Span::styled(
        format!("Active bonuses ({}/{} rooms): ", rooms_built, total_rooms),
        Style::default().fg(Color::White),
    )];

    // Add each active bonus
    let bonus_types = [
        (HavenBonusType::DamagePercent, "+{}% DMG"),
        (HavenBonusType::XpGainPercent, "+{}% XP"),
        (HavenBonusType::DropRatePercent, "+{}% Drops"),
        (HavenBonusType::CritChancePercent, "+{}% Crit"),
        (HavenBonusType::HpRegenPercent, "+{}% HP Regen"),
        (HavenBonusType::AttackIntervalReduction, "-{}% Atk Interval"),
        (HavenBonusType::OfflineXpPercent, "+{}% Offline XP"),
        (HavenBonusType::ChallengeDiscoveryPercent, "+{}% Discovery"),
    ];

    let mut first = true;
    for (bonus_type, fmt) in bonus_types {
        let value = haven.get_bonus(bonus_type);
        if value > 0.0 {
            if !first {
                spans.push(Span::raw("  "));
            }
            spans.push(Span::styled(
                fmt.replace("{}", &format!("{:.0}", value)),
                Style::default().fg(Color::Yellow),
            ));
            first = false;
        }
    }

    if first {
        spans.push(Span::styled(
            "None yet",
            Style::default().fg(Color::DarkGray),
        ));
    }

    let summary = Paragraph::new(Line::from(spans));
    frame.render_widget(summary, area);
}

/// Render the skill tree list
fn render_skill_tree(frame: &mut Frame, area: Rect, haven: &Haven, selected_room: usize) {
    let block = Block::default()
        .title(" Skill Tree ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let items: Vec<ListItem> = HavenRoomId::ALL
        .iter()
        .enumerate()
        .map(|(i, room)| {
            let tier = haven.room_tier(*room);
            let unlocked = haven.is_room_unlocked(*room);
            let is_selected = i == selected_room;

            // Tier indicator: ★★★ for built tiers, ··· for unbuilt
            let tier_str = format!(
                "{}{}{}",
                if tier >= 1 { "★" } else { "·" },
                if tier >= 2 { "★" } else { "·" },
                if tier >= 3 { "★" } else { "·" }
            );

            // Room prefix based on state
            let prefix = if is_selected { "▶ " } else { "  " };

            // Indent based on tree depth
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
            };

            let style = if !unlocked {
                Style::default().fg(Color::DarkGray)
            } else if is_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if tier > 0 {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::White)
            };

            let lock_indicator = if !unlocked { "🔒 " } else { "" };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(
                    tier_str,
                    if tier > 0 {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                ),
                Span::raw(" "),
                Span::styled(indent, style),
                Span::styled(lock_indicator, Style::default().fg(Color::DarkGray)),
                Span::styled(room.name(), style),
            ]))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}

/// Render the room detail panel
fn render_room_detail(
    frame: &mut Frame,
    area: Rect,
    haven: &Haven,
    selected_room: usize,
    prestige_rank: u32,
    fishing_rank: u32,
) {
    let room = HavenRoomId::ALL[selected_room];
    let tier = haven.room_tier(room);
    let unlocked = haven.is_room_unlocked(room);

    let title = format!(" {} ", room.name());
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if unlocked {
            Color::Cyan
        } else {
            Color::DarkGray
        }));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Description
            Constraint::Length(1), // Spacer
            Constraint::Length(4), // Bonus info
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Cost info
            Constraint::Min(0),    // Padding
        ])
        .split(inner);

    // Description
    let desc = Paragraph::new(room.description())
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });
    frame.render_widget(desc, chunks[0]);

    // Bonus info
    let mut bonus_lines = vec![];

    if tier > 0 {
        bonus_lines.push(Line::from(vec![
            Span::styled("Current: ", Style::default().fg(Color::DarkGray)),
            Span::styled(room.format_bonus(tier), Style::default().fg(Color::Green)),
        ]));
    }

    if tier < 3 {
        let next_tier = tier + 1;
        bonus_lines.push(Line::from(vec![
            Span::styled(
                format!("T{}: ", next_tier),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                room.format_bonus(next_tier),
                Style::default().fg(Color::Yellow),
            ),
        ]));
    } else {
        bonus_lines.push(Line::from(Span::styled(
            "Max tier reached",
            Style::default().fg(Color::Green),
        )));
    }

    let bonus_para = Paragraph::new(bonus_lines);
    frame.render_widget(bonus_para, chunks[2]);

    // Cost info
    if !unlocked {
        // Show what's needed to unlock
        let parents = room.parents();
        let parent_names: Vec<&str> = parents.iter().map(|p| p.name()).collect();
        let cost_text = Paragraph::new(vec![
            Line::from(Span::styled("🔒 Locked", Style::default().fg(Color::Red))),
            Line::from(Span::styled(
                format!("Requires: {}", parent_names.join(" + ")),
                Style::default().fg(Color::DarkGray),
            )),
        ]);
        frame.render_widget(cost_text, chunks[4]);
    } else if tier < 3 {
        let next_tier = tier + 1;
        let cost = tier_cost(next_tier);
        let can_afford_it = can_afford(room, haven, prestige_rank, fishing_rank);

        let cost_style = if can_afford_it {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Red)
        };

        let cost_text = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Cost: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{}P {}F", cost.prestige_ranks, cost.fishing_ranks),
                    cost_style,
                ),
            ]),
            Line::from(vec![
                Span::styled("You have: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{}P {}F", prestige_rank, fishing_rank),
                    Style::default().fg(Color::White),
                ),
            ]),
        ]);
        frame.render_widget(cost_text, chunks[4]);
    }
}

/// Render the Haven discovery modal
pub fn render_haven_discovery_modal(frame: &mut Frame, area: Rect) {
    // Center the modal
    let modal_width = 50;
    let modal_height = 7;
    let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(
        x,
        y,
        modal_width.min(area.width),
        modal_height.min(area.height),
    );

    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(" Discovery! ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    let text = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "🏠 You discovered a Haven!",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press [H] to visit and build your base.",
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            "[Enter] to continue",
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
    fishing_rank: u32,
) {
    // Center the modal
    let modal_width = 45;
    let modal_height = 9;
    let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(
        x,
        y,
        modal_width.min(area.width),
        modal_height.min(area.height),
    );

    frame.render_widget(Clear, modal_area);

    let tier = haven.room_tier(room);
    let next_tier = tier + 1;
    let cost = tier_cost(next_tier);
    let can_afford_it = can_afford(room, haven, prestige_rank, fishing_rank);

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
            Span::styled(
                format!("{}P {}F", cost.prestige_ranks, cost.fishing_ranks),
                cost_style,
            ),
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

/// Render the Vault item selection screen (shown during prestige when Vault is built)
pub fn render_vault_selection(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    vault_slots: u8,
    selected_index: usize,
    selected_items: &[EquipmentSlot],
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
