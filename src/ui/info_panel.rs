use super::responsive::LayoutContext;
use crate::core::game_state::GameState;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Safely truncate a string to fit within `max_width` bytes, respecting char boundaries.
fn truncate_to_width(s: &str, max_width: usize) -> String {
    if s.len() <= max_width {
        return s.to_string();
    }
    let limit = max_width.saturating_sub(3); // room for "…" (3 bytes UTF-8)
    let boundary = s.floor_char_boundary(limit);
    format!("{}…", &s[..boundary])
}

/// Draws the compact combat log for M/S tiers.
pub fn draw_info_panel(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    _ctx: &LayoutContext,
) {
    draw_combat_log_compact(frame, area, game_state);
}

/// Compact combat log for M tier (no borders, no loot side).
fn draw_combat_log_compact(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let mut lines: Vec<Line> = Vec::new();
    let max_entries = area.height as usize;
    let max_width = area.width as usize;

    for entry in game_state
        .combat_state
        .combat_log
        .iter()
        .rev()
        .take(max_entries)
    {
        let color = if entry.is_player_action {
            if entry.is_crit {
                Color::Yellow
            } else {
                Color::Green
            }
        } else {
            Color::Red
        };
        let msg = truncate_to_width(&entry.message, max_width);
        lines.push(Line::from(Span::styled(msg, Style::default().fg(color))));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}
