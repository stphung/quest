//! Attribute rendering helpers for the stats panel.

use crate::character::attributes::AttributeType;
use crate::core::game_state::GameState;
use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Returns the display color for an attribute type.
pub(super) fn attr_color(attr_type: AttributeType) -> Color {
    match attr_type {
        AttributeType::Strength => Color::Red,
        AttributeType::Dexterity => Color::Green,
        AttributeType::Constitution => Color::Magenta,
        AttributeType::Intelligence => Color::Blue,
        AttributeType::Wisdom => Color::Cyan,
        AttributeType::Charisma => Color::Yellow,
    }
}

/// Returns a dim background color for an attribute gauge.
pub(super) fn attr_bg_color(attr_type: AttributeType) -> Color {
    match attr_type {
        AttributeType::Strength => Color::Rgb(28, 8, 8),
        AttributeType::Dexterity => Color::Rgb(8, 28, 8),
        AttributeType::Constitution => Color::Rgb(28, 8, 28),
        AttributeType::Intelligence => Color::Rgb(8, 8, 28),
        AttributeType::Wisdom => Color::Rgb(8, 28, 28),
        AttributeType::Charisma => Color::Rgb(28, 28, 8),
    }
}

/// Formats a modifier value with a sign prefix.
pub(super) fn format_modifier(modifier: i32) -> String {
    if modifier >= 0 {
        format!("+{}", modifier)
    } else {
        format!("{}", modifier)
    }
}

/// Draws all 6 attributes on a single line for M tier.
/// Format: "STR:24 DEX:18 CON:21 INT:15 WIS:12 CHA:16"
pub(super) fn draw_attributes_single_line(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    game_state: &GameState,
) {
    let mut spans = Vec::new();

    for (i, attr_type) in AttributeType::all().iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(" "));
        }
        let value = game_state.attributes.get(*attr_type);
        let color = attr_color(*attr_type);
        spans.push(Span::styled(
            format!("{}:", attr_type.abbrev()),
            Style::default().add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!("{}", value),
            Style::default().fg(color),
        ));
    }

    let paragraph = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}
