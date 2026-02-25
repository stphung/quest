//! Equipment rendering helpers for the stats panel.

use crate::core::game_state::GameState;
use ratatui::{
    layout::Rect,
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

    let total_power: u32 = slot_order
        .iter()
        .filter_map(|slot| game_state.equipment.get(*slot).as_ref())
        .map(|item| item.power())
        .sum();
    let title = if total_power > 0 {
        format!(" Equipment \u{26A1}{} ", total_power)
    } else {
        " Equipment ".to_string()
    };
    let block = Block::default().borders(Borders::ALL).title(title);
    let block = super::themed_block(block);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    super::apply_themed_border_fx(frame, area, Color::White, super::BorderFxContext);

    let width = inner.width as usize;
    // Right-side columns: " Legendary  T9  100  ⚡999" = 27 chars fixed
    //   rarity(9) + gap(2) + tier(2) + gap(2) + ilvl(3) + gap(1) + power(~6) + trailing(2) = 27
    let right_cols = 27;
    // Left side: "Weapon  " = 8 chars
    let slot_col = 8;
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
            spans.push(Span::styled(
                format!(" \u{26A1}{}", item.power()),
                Style::default().fg(Color::Cyan),
            ));

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
}
