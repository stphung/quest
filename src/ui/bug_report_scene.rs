//! Bug report overlay: shows a compact game-state preview and clipboard status.

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Draws the bug report overlay as a centered modal dialog.
pub fn draw_bug_report_overlay(
    frame: &mut Frame,
    summary: &str,
    clipboard_ready: bool,
    error: &Option<String>,
) {
    let size = frame.area();

    // Size the dialog to fit the summary + controls
    let summary_lines = summary.lines().count() as u16;
    let w = 60.min(size.width.saturating_sub(4));
    let h = (summary_lines + 8).min(size.height.saturating_sub(4));
    let x = (size.width.saturating_sub(w)) / 2;
    let y = (size.height.saturating_sub(h)) / 2;
    let area = Rect::new(x, y, w, h);

    frame.render_widget(Clear, area);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));

    // Game state summary
    for line in summary.lines() {
        lines.push(Line::from(format!("  {line}")));
    }

    lines.push(Line::from(""));

    // Clipboard status
    if let Some(ref err) = error {
        lines.push(Line::from(Span::styled(
            format!("  Clipboard error: {err}"),
            Style::default().fg(Color::Red),
        )));
    } else if clipboard_ready {
        lines.push(Line::from(Span::styled(
            "  Save data copied to clipboard.",
            Style::default().fg(Color::Green),
        )));
    }

    lines.push(Line::from(""));

    // Controls
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            "[Enter] Open in Browser",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("    "),
        Span::styled(
            "[Esc] Cancel",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    let title = Line::from(Span::styled(
        " Bug Report ",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ));

    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(super::themed_border_color(Color::Yellow)));
    let inner =
        super::render_themed_block(frame, area, block, Color::Yellow, super::BorderFxContext);
    let paragraph = Paragraph::new(lines);

    frame.render_widget(paragraph, inner);
}

/// Draws a minimal centered modal showing a URL the user can copy manually.
pub fn draw_browser_link_modal(frame: &mut Frame, url: &str) {
    let size = frame.area();

    let w = ((url.len() as u16) + 8)
        .max(30)
        .min(size.width.saturating_sub(4));
    let h = 7u16.min(size.height.saturating_sub(4));
    let x = (size.width.saturating_sub(w)) / 2;
    let y = (size.height.saturating_sub(h)) / 2;
    let area = Rect::new(x, y, w, h);

    frame.render_widget(Clear, area);

    let lines = vec![
        Line::from(""),
        Line::from(Span::raw("  Visit:")),
        Line::from(Span::styled(
            format!("  {url}"),
            Style::default().fg(Color::Cyan),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  [Esc] Close",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(Paragraph::new(lines), inner);
}
