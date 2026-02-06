use crate::character::prestige::{get_next_prestige_tier, get_prestige_tier};
use crate::core::game_state::GameState;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Draws the prestige confirmation dialog as an overlay
pub fn draw_prestige_confirm(frame: &mut Frame, game_state: &GameState) {
    let size = frame.area();

    // Calculate dialog size and position (centered)
    let dialog_width = 50.min(size.width.saturating_sub(4));
    let dialog_height = 18.min(size.height.saturating_sub(4));

    let x = (size.width.saturating_sub(dialog_width)) / 2;
    let y = (size.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    // Clear the area behind the dialog
    frame.render_widget(Clear, dialog_area);

    // Get current and next prestige info
    let current_tier = get_prestige_tier(game_state.prestige_rank);
    let next_tier = get_next_prestige_tier(game_state.prestige_rank);
    let current_cap = game_state.get_attribute_cap();
    let next_cap = current_cap + 5;

    // Build the dialog content
    let title = Line::from(vec![Span::styled(
        " Confirm Prestige ",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )]);

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Prestiging will reset:",
            Style::default().fg(Color::Red),
        )),
        Line::from("  - Level and XP"),
        Line::from("  - All attributes"),
        Line::from("  - All equipped items"),
        Line::from("  - Current dungeon progress"),
        Line::from(""),
        Line::from(Span::styled(
            "You will gain:",
            Style::default().fg(Color::Green),
        )),
        Line::from(vec![
            Span::raw("  - Prestige: "),
            Span::styled(current_tier.name, Style::default().fg(Color::Cyan)),
            Span::raw(" -> "),
            Span::styled(
                next_tier.name,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("  - XP Multiplier: "),
            Span::styled(
                format!("{:.2}x", current_tier.multiplier),
                Style::default().fg(Color::Cyan),
            ),
            Span::raw(" -> "),
            Span::styled(
                format!("{:.2}x", next_tier.multiplier),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("  - Attribute Cap: "),
            Span::styled(format!("{}", current_cap), Style::default().fg(Color::Cyan)),
            Span::raw(" -> "),
            Span::styled(
                format!("{}", next_cap),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(""),
    ];

    // Add button hints
    lines.push(Line::from(vec![
        Span::raw("      "),
        Span::styled(
            "[Y] Yes, Prestige",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("    "),
        Span::styled(
            "[N] Cancel",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
    ]));

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(title)
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .alignment(Alignment::Left);

    frame.render_widget(paragraph, dialog_area);
}
