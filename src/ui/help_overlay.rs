use crate::core::constants::WIKI_URL;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Draws the help overlay as a centered modal dialog.
pub fn draw_help_overlay(frame: &mut Frame) {
    let size = frame.area();

    let dialog_width = 56.min(size.width.saturating_sub(4));
    let dialog_height = 14.min(size.height.saturating_sub(4));

    let x = (size.width.saturating_sub(dialog_width)) / 2;
    let y = (size.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    frame.render_widget(Clear, dialog_area);

    let title = Line::from(vec![Span::styled(
        " Help ",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Controls",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  [P] Prestige  [H] Haven  [S] Soulforge  [G] Stormglass"),
        Line::from("  [A] Achievements    [Tab] Challenges"),
        Line::from("  [U] Toggle Updates  [!] Report Bug  [Esc] Quit"),
        Line::from(""),
        Line::from(Span::styled(
            "  Quest Wiki",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  {}", WIKI_URL),
            Style::default().fg(Color::Cyan),
        )),
        Line::from("  Guides for combat, zones, prestige, and more."),
        Line::from(""),
        Line::from(Span::styled(
            "  [Esc] Close",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(title)
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .alignment(Alignment::Left);

    frame.render_widget(paragraph, dialog_area);
}
