use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Draws the credits overlay as a centered modal dialog.
pub fn draw_credits_overlay(frame: &mut Frame) {
    let size = frame.area();

    let dialog_width = 44.min(size.width.saturating_sub(4));
    let dialog_height = 15.min(size.height.saturating_sub(4));

    let x = (size.width.saturating_sub(dialog_width)) / 2;
    let y = (size.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    frame.render_widget(Clear, dialog_area);

    let title = Line::from(vec![Span::styled(
        " Credits ",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]);

    let bold = Style::default().add_modifier(Modifier::BOLD);
    let dim = Style::default().fg(Color::DarkGray);

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("          "),
            Span::styled("\u{2694}  Q U E S T  \u{2694}", bold.fg(Color::Cyan)),
        ]),
        Line::from(Span::styled(
            "    A Terminal-Based Idle RPG",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled("  \u{2500}\u{2500}\u{2500} Forged By \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}", bold)),
        Line::from(vec![
            Span::raw("  Steven Phung (@stphung) "),
            Span::styled("Creator", dim),
        ]),
        Line::from(vec![
            Span::raw("  DH (@dhsu)         "),
            Span::styled("Contributor", dim),
        ]),
        Line::from(""),
        Line::from(Span::styled("  \u{2500}\u{2500}\u{2500} Built With \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}", bold)),
        Line::from("  Rust \u{00b7} Ratatui \u{00b7} Crossterm"),
        Line::from(""),
        Line::from(Span::styled("  [Esc] Close", dim)),
    ];

    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(super::themed_border_color(Color::Cyan)));
    let inner = super::render_themed_block(
        frame,
        dialog_area,
        block,
        Color::Cyan,
        super::BorderFxContext,
    );
    let paragraph = Paragraph::new(lines).alignment(Alignment::Left);

    frame.render_widget(paragraph, inner);
}
