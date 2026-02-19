//! Equipment detail overlay: shows all 7 equipment slots with full item details.

use super::responsive::LayoutContext;
use crate::core::game_state::GameState;
use crate::items::types::{AffixType, EquipmentSlot};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

const SLOT_ORDER: [EquipmentSlot; 7] = [
    EquipmentSlot::Weapon,
    EquipmentSlot::Armor,
    EquipmentSlot::Helmet,
    EquipmentSlot::Gloves,
    EquipmentSlot::Boots,
    EquipmentSlot::Amulet,
    EquipmentSlot::Ring,
];

const ATTR_LABELS: [&str; 6] = ["STR", "DEX", "CON", "INT", "WIS", "CHA"];

/// Render the equipment detail overlay in the given area.
pub fn render_equipment_detail(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    selected_slot: usize,
    enhancement_levels: &[u8; 7],
    _ctx: &LayoutContext,
) {
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Equipment ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    // Split inner area: top (slot list) and bottom (detail pane)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9), // 7 slot lines + 2 borders for inner block
            Constraint::Min(0),    // detail pane
        ])
        .split(inner);

    render_slot_list(
        frame,
        chunks[0],
        game_state,
        selected_slot,
        enhancement_levels,
    );
    render_detail_pane(
        frame,
        chunks[1],
        game_state,
        selected_slot,
        enhancement_levels,
    );
}

/// Render the 7-slot equipment list with cursor highlight.
/// Uses the same column alignment as `draw_equipment_names_only` in stats_panel.
fn render_slot_list(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    selected_slot: usize,
    enhancement_levels: &[u8; 7],
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let width = inner.width as usize;
    // Match draw_equipment_names_only column layout:
    // cursor(2) + slot(8) + name(variable) + right_cols(27)
    let cursor_col = 2; // "▸ " or "  "
    let slot_col = 8; // "{:>6}  "
    let right_cols = 27; // rarity(9) + tier(4) + zone(4) + power(~6) + gaps
    let name_max = width.saturating_sub(cursor_col + slot_col + right_cols);

    let mut lines: Vec<Line> = Vec::with_capacity(7);

    for (i, slot) in SLOT_ORDER.iter().enumerate() {
        let item = game_state.equipment.get(*slot);
        let is_selected = i == selected_slot;
        let cursor = if is_selected { "\u{25b8} " } else { "  " };

        if let Some(item) = item.as_ref() {
            let rc = super::rarity_color(item.rarity);
            let tc = super::tier_color(item.tier);
            let enh_level = enhancement_levels[i];
            let enh_prefix = crate::enhancement::enhancement_prefix(enh_level);
            let prefix_len = enh_prefix.len();

            // Truncate name to fit column layout
            let max_name_len = name_max.saturating_sub(prefix_len);
            let item_name = if item.display_name.len() > max_name_len && max_name_len > 3 {
                format!("{}...", &item.display_name[..max_name_len - 3])
            } else {
                item.display_name.clone()
            };
            let name_len = prefix_len + item_name.len();
            let pad = name_max.saturating_sub(name_len);

            if is_selected {
                let yellow = Style::default().fg(Color::Yellow);
                let mut spans = vec![
                    Span::styled(cursor, yellow),
                    Span::styled(format!("{:>6}  ", slot.name()), yellow),
                ];
                if !enh_prefix.is_empty() {
                    spans.push(Span::styled(enh_prefix, yellow));
                }
                spans.push(Span::styled(item_name, yellow));
                spans.push(Span::raw(" ".repeat(pad)));
                spans.push(Span::styled(format!("{:>9}", item.rarity.name()), yellow));
                spans.push(Span::styled(format!("  T{}", item.tier), yellow));
                spans.push(Span::styled(format!("  Z{}", item.ilvl / 10), yellow));
                spans.push(Span::styled(format!(" \u{26A1}{}", item.power()), yellow));
                lines.push(Line::from(spans));
            } else {
                let mut spans = vec![
                    Span::styled(cursor, Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:>6}  ", slot.name()),
                        Style::default().add_modifier(Modifier::BOLD),
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
                    Style::default().fg(tc),
                ));
                spans.push(Span::styled(
                    format!("  Z{}", item.ilvl / 10),
                    Style::default().fg(Color::DarkGray),
                ));
                spans.push(Span::styled(
                    format!(" \u{26A1}{}", item.power()),
                    Style::default().fg(Color::Cyan),
                ));
                lines.push(Line::from(spans));
            }
        } else {
            let style = if is_selected {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            lines.push(Line::from(vec![
                Span::styled(cursor, style),
                Span::styled(
                    format!("{:>6}  ", slot.name()),
                    if is_selected {
                        style
                    } else {
                        Style::default().add_modifier(Modifier::BOLD)
                    },
                ),
                Span::styled("[Empty]", Style::default().fg(Color::DarkGray)),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

/// Render the detail pane showing full info for the selected item.
fn render_detail_pane(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    selected_slot: usize,
    enhancement_levels: &[u8; 7],
) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    let slot = SLOT_ORDER[selected_slot];
    let item = game_state.equipment.get(slot);

    match item.as_ref() {
        Some(item) => {
            let mut lines: Vec<Line> = Vec::new();
            let rc = super::rarity_color(item.rarity);
            let tc = super::tier_color(item.tier);

            // Line 1: Item display name in rarity color, bold
            lines.push(Line::from(Span::styled(
                format!(" {}", item.display_name),
                Style::default().fg(rc).add_modifier(Modifier::BOLD),
            )));

            // Line 2: Rarity, Tier, Zone, Power (right-aligned power)
            let left_part = vec![
                Span::styled(" ", Style::default()),
                Span::styled(item.rarity.name(), Style::default().fg(rc)),
                Span::styled(" ", Style::default()),
                Span::styled(format!("T{}", item.tier), Style::default().fg(tc)),
                Span::styled(" ", Style::default()),
                Span::styled(
                    format!("Z{}", item.ilvl / 10),
                    Style::default().fg(Color::DarkGray),
                ),
            ];
            // Calculate left text width for power alignment
            let left_width: usize = left_part.iter().map(|s| s.content.len()).sum();
            let power_text = format!("\u{26a1} Power: {}", item.power());
            let available = area.width as usize;
            let padding = available.saturating_sub(left_width + power_text.len() + 1);
            let mut info_spans = left_part;
            info_spans.push(Span::raw(" ".repeat(padding)));
            info_spans.push(Span::styled(power_text, Style::default().fg(Color::Cyan)));
            lines.push(Line::from(info_spans));

            // Blank line
            lines.push(Line::from(""));

            // Attributes: Show non-zero attribute bonuses
            let attrs = item.attributes.as_array();
            let mut attr_spans: Vec<Span> = vec![Span::styled(" ", Style::default())];
            let mut has_attrs = false;
            for (value, label) in attrs.iter().zip(ATTR_LABELS.iter()) {
                if *value > 0 {
                    if has_attrs {
                        attr_spans.push(Span::styled("  ", Style::default()));
                    }
                    attr_spans.push(Span::styled(
                        format!("+{} {}", value, label),
                        Style::default().fg(Color::Green),
                    ));
                    has_attrs = true;
                }
            }
            if has_attrs {
                lines.push(Line::from(attr_spans));
            }

            // Blank line before affixes
            if !item.affixes.is_empty() {
                lines.push(Line::from(""));
            }

            // Affixes
            for affix in &item.affixes {
                let text = match affix.affix_type {
                    AffixType::DamagePercent => format!("+{:.0}% Damage", affix.value),
                    AffixType::CritChance => format!("+{:.0}% Crit Chance", affix.value),
                    AffixType::CritMultiplier => format!("+{:.1}x Crit Damage", affix.value),
                    AffixType::AttackSpeed => format!("+{:.0}% Attack Speed", affix.value),
                    AffixType::HPBonus => format!("+{:.0} HP", affix.value),
                    AffixType::DamageReduction => format!("+{:.0}% Damage Reduction", affix.value),
                    AffixType::HPRegen => format!("+{:.0} HP Regen", affix.value),
                    AffixType::DamageReflection => {
                        format!("+{:.0}% Damage Reflection", affix.value)
                    }
                    AffixType::XPGain => format!("+{:.0}% XP Gain", affix.value),
                    AffixType::Unknown => continue,
                };
                lines.push(Line::from(Span::styled(
                    format!(" {}", text),
                    Style::default().fg(Color::Cyan),
                )));
            }

            // Enhancement info (if level > 0)
            let enh_level = enhancement_levels[selected_slot];
            if enh_level > 0 {
                lines.push(Line::from(""));
                let multiplier = crate::enhancement::enhancement_multiplier(enh_level);
                let enh_style = super::stats_panel::enhancement_style(enh_level);
                lines.push(Line::from(vec![
                    Span::styled(" Enhancement: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(format!("+{} ({:.2}x)", enh_level, multiplier), enh_style),
                ]));
            }

            // God item info
            if let Some(god_item_id) = item.god_item_id {
                let def = crate::god_items::types::get_god_item_definition(god_item_id);
                let gold = Color::Rgb(255, 215, 0);

                lines.push(Line::from(""));

                // Passive line
                let passive_text = match def.passive {
                    crate::god_items::types::GodItemPassive::DivineBulwark {
                        damage_reduction_percent,
                    } => format!(
                        "\u{2605} Divine Bulwark: +{:.0}% damage reduction",
                        damage_reduction_percent
                    ),
                    crate::god_items::types::GodItemPassive::Windborne {
                        attack_speed_percent,
                    } => format!(
                        "\u{2605} Windborne: +{:.0}% attack speed",
                        attack_speed_percent
                    ),
                    crate::god_items::types::GodItemPassive::GiantsMight { damage_percent } => {
                        format!("\u{2605} Giant's Might: +{:.0}% damage", damage_percent)
                    }
                };
                lines.push(Line::from(Span::styled(
                    format!(" {}", passive_text),
                    Style::default().fg(gold),
                )));

                // Bonus lines
                for bonus in &def.bonuses {
                    let bonus_text = match bonus {
                        crate::god_items::types::GodItemBonus::Swiftstrider {
                            regen_reduction_percent,
                        } => format!(
                            "\u{2605} Swiftstrider: {:.0}% regen reduction",
                            regen_reduction_percent
                        ),
                        crate::god_items::types::GodItemBonus::Swiftfoot {
                            dungeon_speed_percent,
                        } => format!(
                            "\u{2605} Swiftfoot: {:.0}% dungeon speed",
                            dungeon_speed_percent
                        ),
                        crate::god_items::types::GodItemBonus::NimbleHands {
                            fishing_reduction_percent,
                        } => format!(
                            "\u{2605} Nimble Hands: {:.0}% fishing speed",
                            fishing_reduction_percent
                        ),
                    };
                    lines.push(Line::from(Span::styled(
                        format!(" {}", bonus_text),
                        Style::default().fg(gold),
                    )));
                }

                // XP bonus from affix (already shown in affixes section, but god items
                // have it as a notable bonus too — skip duplicate)
            }

            // Footer line at bottom
            // Calculate remaining space and add footer at the end
            let content_lines = lines.len() as u16;
            let remaining = area.height.saturating_sub(content_lines + 1);
            for _ in 0..remaining {
                lines.push(Line::from(""));
            }
            lines.push(Line::from(Span::styled(
                "[\u{2191}\u{2193}] Select    [E/Esc] Close",
                Style::default().fg(Color::DarkGray),
            )));

            let paragraph = Paragraph::new(lines).alignment(Alignment::Left);
            frame.render_widget(paragraph, area);
        }
        None => {
            // Empty slot
            let mut lines: Vec<Line> = Vec::new();

            // Center vertically
            let center_row = area.height / 2;
            for _ in 0..center_row {
                lines.push(Line::from(""));
            }
            lines.push(Line::from(Span::styled(
                "[No item equipped]",
                Style::default().fg(Color::DarkGray),
            )));

            // Fill to footer
            let content_lines = lines.len() as u16;
            let remaining = area.height.saturating_sub(content_lines + 1);
            for _ in 0..remaining {
                lines.push(Line::from(""));
            }
            lines.push(Line::from(Span::styled(
                "[\u{2191}\u{2193}] Select    [E/Esc] Close",
                Style::default().fg(Color::DarkGray),
            )));

            let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
            frame.render_widget(paragraph, area);
        }
    }
}
