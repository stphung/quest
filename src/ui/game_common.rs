//! Shared UI components for minigames.

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Render a standardized status bar (2 lines: status message + controls).
///
/// # Arguments
/// * `frame` - The frame to render to
/// * `area` - A 2-line area at the bottom of the game panel
/// * `status_text` - The status message to display (line 1)
/// * `status_color` - Color for the status message
/// * `controls` - Slice of (key, action) pairs, e.g., `[("[Enter]", "Select"), ("[Esc]", "Quit")]`
pub fn render_status_bar(
    frame: &mut Frame,
    area: Rect,
    status_text: &str,
    status_color: Color,
    controls: &[(&str, &str)],
) {
    if area.height < 1 {
        return;
    }

    // Line 1: Status message (centered)
    let status = Paragraph::new(status_text)
        .style(Style::default().fg(status_color))
        .alignment(Alignment::Center);
    frame.render_widget(status, Rect { height: 1, ..area });

    // Line 2: Controls (centered)
    if area.height >= 2 && !controls.is_empty() {
        let mut spans = Vec::new();
        for (i, (key, action)) in controls.iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled("  ", Style::default()));
            }
            spans.push(Span::styled(*key, Style::default().fg(Color::White)));
            spans.push(Span::styled(
                format!(" {}", action),
                Style::default().fg(Color::DarkGray),
            ));
        }

        let controls_line = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
        frame.render_widget(
            controls_line,
            Rect {
                y: area.y + 1,
                height: 1,
                ..area
            },
        );
    }
}

/// Render a standardized status bar with a spinner for AI thinking state.
///
/// Uses a braille spinner animation (100ms per frame).
pub fn render_thinking_status_bar(frame: &mut Frame, area: Rect, message: &str) {
    use std::time::{SystemTime, UNIX_EPOCH};

    const SPINNER: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let frame_idx = ((millis / 100) % 10) as usize;
    let spinner = SPINNER[frame_idx];

    let status_text = format!("{} {}", spinner, message);
    render_status_bar(frame, area, &status_text, Color::Yellow, &[]);
}
