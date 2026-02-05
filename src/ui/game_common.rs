//! Shared UI components for minigames.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Layout areas returned by `create_game_layout`.
pub struct GameLayout {
    /// Main content area (board/grid) - top left, inside outer border
    pub content: Rect,
    /// Status bar area (2 lines) - bottom left, inside outer border
    pub status_bar: Rect,
    /// Info panel area - right side, with its own border
    pub info_panel: Rect,
}

/// Create a standardized game layout with outer border.
///
/// Layout structure (matches Morris/Chess pattern):
/// ```text
/// ┌─ Title ─────────────────────────┬─ Info ──────┐
/// │                                 │             │
/// │   [content area]                │  [info]     │
/// │                                 │             │
/// │ [status bar - 2 lines]          │             │
/// └─────────────────────────────────┴─────────────┘
/// ```
///
/// # Arguments
/// * `frame` - The frame to render to
/// * `area` - The full area to use
/// * `title` - Title for the outer border (e.g., " Gomoku ")
/// * `border_color` - Color for the outer border
/// * `content_min_height` - Minimum height for the content area
/// * `info_panel_width` - Width of the info panel (typically 22-24)
///
/// # Returns
/// A `GameLayout` struct containing the areas for content, status bar, and info panel.
pub fn create_game_layout(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    border_color: Color,
    content_min_height: u16,
    info_panel_width: u16,
) -> GameLayout {
    frame.render_widget(Clear, area);

    // Outer border around entire game area
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Horizontal split: content area (left) | info panel (right)
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(20), Constraint::Length(info_panel_width)])
        .split(inner);

    // Left side: content (top) + status bar (bottom 2 lines)
    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(content_min_height), Constraint::Length(2)])
        .split(h_chunks[0]);

    GameLayout {
        content: v_chunks[0],
        status_bar: v_chunks[1],
        info_panel: h_chunks[1],
    }
}

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

/// Game result type for the shared overlay.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GameResultType {
    Win,
    Loss,
    Draw,
    Forfeit,
}

impl GameResultType {
    /// Get the color for this result type.
    pub fn color(self) -> Color {
        match self {
            GameResultType::Win => Color::Green,
            GameResultType::Loss => Color::Red,
            GameResultType::Draw => Color::Yellow,
            GameResultType::Forfeit => Color::Gray,
        }
    }
}

/// Render a full-screen game over overlay (Chess/Morris style).
///
/// Fills the entire area with a bordered overlay containing:
/// - Title (bold, colored by result)
/// - Message describing the outcome
/// - Reward text
/// - "[Press any key]"
pub fn render_game_over_overlay(
    frame: &mut Frame,
    area: Rect,
    result_type: GameResultType,
    title: &str,
    message: &str,
    reward: &str,
) {
    frame.render_widget(Clear, area);

    let title_color = result_type.color();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(title_color));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let content_height: u16 = 7;
    let y_offset = inner.y + (inner.height.saturating_sub(content_height)) / 2;

    let lines = vec![
        Line::from(Span::styled(
            title,
            Style::default()
                .fg(title_color)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(message, Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(Span::styled(reward, Style::default().fg(Color::Cyan))),
        Line::from(""),
        Line::from(Span::styled(
            "[Press any key]",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let text = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(
        text,
        Rect::new(inner.x, y_offset, inner.width, content_height),
    );
}

/// Forfeit confirmation status text.
pub const FORFEIT_STATUS_TEXT: &str = "Forfeit game?";

/// Forfeit confirmation status color.
pub const FORFEIT_STATUS_COLOR: Color = Color::Red;

/// Forfeit confirmation controls.
pub const FORFEIT_CONTROLS: &[(&str, &str)] = &[("[Esc]", "Confirm"), ("[Any]", "Cancel")];

/// Render the forfeit confirmation status bar.
///
/// Call this when `forfeit_pending` is true. Returns `true` if rendered.
/// Use this at the start of your status bar function for consistent forfeit UI.
///
/// # Example
/// ```ignore
/// fn render_status_bar_content(frame: &mut Frame, area: Rect, game: &MyGame) {
///     if render_forfeit_status_bar(frame, area, game.forfeit_pending) {
///         return;
///     }
///     // ... normal status bar logic
/// }
/// ```
pub fn render_forfeit_status_bar(frame: &mut Frame, area: Rect, forfeit_pending: bool) -> bool {
    if !forfeit_pending {
        return false;
    }
    render_status_bar(
        frame,
        area,
        FORFEIT_STATUS_TEXT,
        FORFEIT_STATUS_COLOR,
        FORFEIT_CONTROLS,
    );
    true
}
