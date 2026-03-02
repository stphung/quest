//! Attribute rendering helpers for the stats panel.

use crate::character::attributes::AttributeType;
use crate::core::game_state::GameState;
use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
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

/// Formats a modifier value with a sign prefix.
pub(super) fn format_modifier(modifier: i32) -> String {
    if modifier >= 0 {
        format!("+{}", modifier)
    } else {
        format!("{}", modifier)
    }
}

/// Draws attributes in a compact 2-column layout.
/// 3 rows: STR/INT, DEX/WIS, CON/CHA with modifiers.
pub(super) fn draw_attributes_compact(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    game_state: &GameState,
) {
    let cap = game_state.get_attribute_cap();
    let attrs_block = Block::default().borders(Borders::ALL).title(" Attributes ");
    let attrs_block = super::themed_block(attrs_block);
    let inner = attrs_block.inner(area);
    frame.render_widget(attrs_block, area);
    super::apply_themed_border_fx(frame, area, Color::White, super::BorderFxContext);

    let gold = Color::Rgb(255, 215, 0);
    const BAR_WIDTH: usize = 8;

    let pairs = [
        (AttributeType::Strength, AttributeType::Intelligence),
        (AttributeType::Dexterity, AttributeType::Wisdom),
        (AttributeType::Constitution, AttributeType::Charisma),
    ];

    let mut lines = Vec::new();
    for (left, right) in &pairs {
        let l_val = game_state.attributes.get(*left);
        let l_mod = game_state.attributes.modifier(*left);
        let r_val = game_state.attributes.get(*right);
        let r_mod = game_state.attributes.modifier(*right);

        let l_fg = if l_val >= cap { gold } else { attr_color(*left) };
        let r_fg = if r_val >= cap { gold } else { attr_color(*right) };

        let l_filled = if cap > 0 {
            ((l_val as f64 / cap as f64) * BAR_WIDTH as f64).round() as usize
        } else {
            0
        }
        .min(BAR_WIDTH);
        let r_filled = if cap > 0 {
            ((r_val as f64 / cap as f64) * BAR_WIDTH as f64).round() as usize
        } else {
            0
        }
        .min(BAR_WIDTH);

        let l_mod_str = format_modifier(l_mod);
        let r_mod_str = format_modifier(r_mod);

        lines.push(Line::from(vec![
            Span::styled(
                format!("{} ", left.abbrev()),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw("["),
            Span::styled(
                "\u{2588}".repeat(l_filled),
                Style::default().fg(l_fg),
            ),
            Span::styled(
                "\u{2591}".repeat(BAR_WIDTH - l_filled),
                Style::default().fg(Color::DarkGray),
            ),
            Span::raw("] "),
            Span::styled(
                format!("{}/{}", l_val, cap),
                Style::default().fg(l_fg),
            ),
            Span::raw(format!(" {:>3}   ", l_mod_str)),
            Span::styled(
                format!("{} ", right.abbrev()),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw("["),
            Span::styled(
                "\u{2588}".repeat(r_filled),
                Style::default().fg(r_fg),
            ),
            Span::styled(
                "\u{2591}".repeat(BAR_WIDTH - r_filled),
                Style::default().fg(Color::DarkGray),
            ),
            Span::raw("] "),
            Span::styled(
                format!("{}/{}", r_val, cap),
                Style::default().fg(r_fg),
            ),
            Span::raw(format!(" {:>3}", r_mod_str)),
        ]));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
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
