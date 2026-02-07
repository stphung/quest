//! Shared UI components for minigames.

use crate::core::game_logic::OfflineReport;
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

/// Render a compact game-over banner at the bottom of an area.
///
/// Unlike `render_game_over_overlay`, this does NOT clear the area,
/// allowing the board to remain visible behind it. The banner is 5 lines tall
/// and appears at the bottom of the given area.
///
/// # Arguments
/// * `frame` - The frame to render to
/// * `area` - The area where the banner should appear (banner at bottom)
/// * `result_type` - Win/Loss/Draw/Forfeit for coloring
/// * `title` - Main result text (e.g., "VICTORY!" or "DEFEAT")
/// * `message` - Explanation of how the game ended
/// * `reward` - Reward text (empty string if no reward)
pub fn render_game_over_banner(
    frame: &mut Frame,
    area: Rect,
    result_type: GameResultType,
    title: &str,
    message: &str,
    reward: &str,
) {
    let banner_height: u16 = if reward.is_empty() { 4 } else { 5 };
    let banner_y = area.y + area.height.saturating_sub(banner_height);

    let banner_area = Rect {
        x: area.x,
        y: banner_y,
        width: area.width,
        height: banner_height,
    };

    // Clear just the banner area
    frame.render_widget(Clear, banner_area);

    let title_color = result_type.color();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(title_color));

    let inner = block.inner(banner_area);
    frame.render_widget(block, banner_area);

    let mut lines = vec![Line::from(vec![
        Span::styled(
            title,
            Style::default()
                .fg(title_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" - "),
        Span::styled(message, Style::default().fg(Color::White)),
    ])];

    if !reward.is_empty() {
        lines.push(Line::from(Span::styled(
            reward,
            Style::default().fg(Color::Cyan),
        )));
    }

    lines.push(Line::from(Span::styled(
        "[Press any key]",
        Style::default().fg(Color::DarkGray),
    )));

    let text = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(text, inner);
}

/// Render an info panel frame with standard " Info " title and DarkGray border.
///
/// Returns the inner Rect for content rendering.
pub fn render_info_panel_frame(frame: &mut Frame, area: Rect) -> Rect {
    let block = Block::default()
        .title(" Info ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    inner
}

/// Render the "Welcome Back" overlay for offline progression.
pub fn render_offline_welcome(frame: &mut Frame, area: Rect, report: &OfflineReport) {
    // Centered modal box
    let modal_width = 44u16;
    let modal_height = if report.level_before < report.level_after {
        11
    } else {
        10
    };
    let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .title(" Your quest continues... ");

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    // Format time away
    let hours = report.elapsed_seconds / 3600;
    let minutes = (report.elapsed_seconds % 3600) / 60;
    let away_str = if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    };

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  Away for: {}", away_str),
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            if report.haven_bonus_percent > 0.0 {
                format!(
                    "  Offline rate: {:.0}% (Haven: +{:.0}%)",
                    report.offline_rate_percent, report.haven_bonus_percent
                )
            } else {
                format!("  Offline rate: {:.0}%", report.offline_rate_percent)
            },
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!(
                "  ⚔️  XP Gained:  {:>10}",
                format_number_short(report.xp_gained)
            ),
            Style::default().fg(Color::Cyan),
        )),
    ];

    if report.level_before < report.level_after {
        lines.push(Line::from(Span::styled(
            format!(
                "  📈 Levels:      {:>10}",
                format!(
                    "+{} ({} → {})",
                    report.level_after - report.level_before,
                    report.level_before,
                    report.level_after
                )
            ),
            Style::default().fg(Color::Green),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Press any key to continue",
        Style::default().fg(Color::DarkGray),
    )));

    let text = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(text, inner);
}

/// Format a number with abbreviated suffixes (K, M, B, T, Q).
pub fn format_number_short(n: u64) -> String {
    // (threshold, divisor, suffix)
    const TIERS: &[(u64, f64, &str)] = &[
        (1_000_000_000_000_000, 1e15, "Q"),
        (1_000_000_000_000, 1e12, "T"),
        (1_000_000_000, 1e9, "B"),
        (1_000_000, 1e6, "M"),
        (10_000, 1e3, "K"),
    ];

    for &(threshold, divisor, suffix) in TIERS {
        if n >= threshold {
            return format!("{:.1}{}", n as f64 / divisor, suffix);
        }
    }
    n.to_string()
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
