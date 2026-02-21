//! Equipment rendering helpers for the stats panel.

use crate::core::game_state::GameState;
use crate::stormglass::sigils::{SigilGrade, StormSigils};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
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
/// If sigils are inscribed, renders a sub-panel below equipment rows.
pub(super) fn draw_equipment_names_only(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    enhancement_levels: &[u8; 7],
    storm_sigils: &StormSigils,
) {
    let block = Block::default().borders(Borders::ALL).title(" Equipment ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Split inner area: 7 lines for equipment, remainder for sigils
    let inscribed = storm_sigils.inscribed_count();
    let (equip_area, sigil_area) = if inscribed > 0 && inner.height > 9 {
        // Sigil sub-panel needs: 1 blank + 1 title border + inscribed lines + 1 bottom border = inscribed + 3
        let sigil_height = (inscribed as u16 + 3).min(inner.height.saturating_sub(7));
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(inner.height.saturating_sub(sigil_height)),
                Constraint::Length(sigil_height),
            ])
            .split(inner);
        (chunks[0], Some(chunks[1]))
    } else {
        (inner, None)
    };

    let width = equip_area.width as usize;
    // Right-side columns: " Legendary  T9  100  ⚡999" = 27 chars fixed
    //   rarity(9) + gap(2) + tier(2) + gap(2) + ilvl(3) + gap(1) + power(~6) + trailing(2) = 27
    let right_cols = 27;
    // Left side: "Weapon  " = 8 chars
    let slot_col = 8;
    // Name gets whatever remains
    let name_max = width.saturating_sub(slot_col + right_cols);

    let mut lines = Vec::new();

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
    frame.render_widget(paragraph, equip_area);

    // Render sigil sub-panel if any are inscribed
    if let Some(sigil_area) = sigil_area {
        draw_sigil_sub_panel(frame, sigil_area, storm_sigils);
    }
}

/// Renders the Storm Sigils sub-panel below equipment.
fn draw_sigil_sub_panel(frame: &mut Frame, area: Rect, storm_sigils: &StormSigils) {
    let inscribed = storm_sigils.inscribed_count();
    let title = format!(
        " Storm Sigils ({}/{}) ",
        inscribed, storm_sigils.slots_unlocked
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(100, 180, 255)))
        .title(title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let width = inner.width as usize;
    let mut lines = Vec::new();

    for sigil in storm_sigils.sigils.iter().flatten() {
        let name = sigil.effect.sigil_name();
        let value_str = format!("+{:.1}%", sigil.value);
        let grade_str = sigil.grade.label();
        let grade_color = sigil_grade_color(sigil.grade);

        // Layout: "  Name              +12.3%  A+"
        let right_part = format!("{}  {}", value_str, grade_str);
        let right_len = right_part.len();
        let name_max = width.saturating_sub(right_len + 4); // 2 indent + 2 gap
        let display_name = if name.len() > name_max && name_max > 3 {
            format!("{}...", &name[..name_max - 3])
        } else {
            name.to_string()
        };
        let pad = name_max.saturating_sub(display_name.len());

        let grade_style = if grade_str.ends_with('+') {
            Style::default()
                .fg(grade_color)
                .add_modifier(Modifier::BOLD)
        } else if grade_str.ends_with('-') {
            Style::default().fg(grade_color).add_modifier(Modifier::DIM)
        } else {
            Style::default().fg(grade_color)
        };

        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(display_name, Style::default().fg(Color::White)),
            Span::raw(" ".repeat(pad)),
            Span::styled(
                format!("  {}", value_str),
                Style::default().fg(Color::Rgb(100, 180, 255)),
            ),
            Span::styled(format!("  {}", grade_str), grade_style),
        ]));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

/// Returns the color for a sigil grade tier letter.
fn sigil_grade_color(grade: SigilGrade) -> Color {
    match grade.tier_letter() {
        'S' => Color::Rgb(255, 215, 0), // Gold
        'A' => Color::Green,
        'B' => Color::Cyan,
        'C' => Color::White,
        'D' => Color::Gray,
        'E' => Color::DarkGray,
        _ => Color::Red, // F
    }
}
