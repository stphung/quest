//! Title browser overlay UI — lets the player select a displayed title.

use crate::achievements::{titles, Achievements};
use crate::ui::responsive::LayoutContext;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// UI state for the title browser overlay.
pub struct TitleBrowserState {
    pub showing: bool,
    pub selected_index: usize,
}

impl TitleBrowserState {
    pub fn new() -> Self {
        Self {
            showing: false,
            selected_index: 0,
        }
    }

    pub fn open(&mut self) {
        self.showing = true;
        self.selected_index = 0;
    }

    pub fn close(&mut self) {
        self.showing = false;
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self, max_items: usize) {
        if self.selected_index + 1 < max_items {
            self.selected_index += 1;
        }
    }
}

impl Default for TitleBrowserState {
    fn default() -> Self {
        Self::new()
    }
}

/// Center-scroll offset keeping the selected item visible (same pattern as
/// the achievement list panel).
fn compute_scroll_offset(total: usize, visible_height: usize, selected_index: usize) -> usize {
    if total <= visible_height {
        0
    } else {
        let max_scroll = total.saturating_sub(visible_height);
        selected_index
            .saturating_sub(visible_height / 2)
            .min(max_scroll)
    }
}

/// Render the title browser overlay.
pub fn render_title_browser(
    frame: &mut Frame,
    area: Rect,
    achievements: &Achievements,
    ui_state: &TitleBrowserState,
    character_name: &str,
    _layout_ctx: &LayoutContext,
) {
    frame.render_widget(Clear, area);

    let unlocked = titles::get_unlocked_titles(achievements);

    let block = Block::default()
        .title(format!(" Titles ({} unlocked) ", unlocked.len()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if unlocked.is_empty() {
        let msg = Paragraph::new("No titles unlocked yet. Keep adventuring!")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(msg, inner);
        return;
    }

    // Layout: title list, preview, help
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Title list
            Constraint::Length(3), // Preview
            Constraint::Length(1), // Help
        ])
        .split(inner);

    // Title list (scrolled so the selected item stays visible)
    let list_area = chunks[0];
    let total = unlocked.len();
    let visible_height = list_area.height as usize;
    let scroll_offset = compute_scroll_offset(total, visible_height, ui_state.selected_index);

    let mut lines = Vec::new();
    for (i, title_def) in unlocked
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_height)
    {
        let is_selected = i == ui_state.selected_index;
        let is_active = achievements.selected_title == Some(title_def.achievement_id);

        let marker = if is_selected { "> " } else { "  " };
        let active_suffix = if is_active { "  \u{2726} active" } else { "" };

        let style = if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else if is_active {
            Style::default().fg(Color::Magenta)
        } else {
            Style::default().fg(Color::White)
        };

        lines.push(Line::from(Span::styled(
            format!("{}{}{}", marker, title_def.title_text, active_suffix),
            style,
        )));
    }
    let list = Paragraph::new(lines);
    frame.render_widget(list, list_area);

    // Scroll indicators when content overflows
    if total > visible_height && visible_height > 0 {
        let max_scroll = total.saturating_sub(visible_height);
        let indicator_style = Style::default().fg(Color::DarkGray);
        if scroll_offset > 0 {
            let up = Paragraph::new(Line::from(Span::styled(" \u{25b2}", indicator_style)))
                .alignment(Alignment::Right);
            frame.render_widget(up, Rect::new(list_area.x, list_area.y, list_area.width, 1));
        }
        if scroll_offset < max_scroll {
            let down = Paragraph::new(Line::from(Span::styled(" \u{25bc}", indicator_style)))
                .alignment(Alignment::Right);
            frame.render_widget(
                down,
                Rect::new(
                    list_area.x,
                    list_area.y + list_area.height - 1,
                    list_area.width,
                    1,
                ),
            );
        }
    }

    // Preview
    let preview_title = if ui_state.selected_index < unlocked.len() {
        format!(
            "{}, {}",
            character_name, unlocked[ui_state.selected_index].title_text
        )
    } else {
        character_name.to_string()
    };
    let preview_block = Block::default()
        .title(" Preview ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let preview = Paragraph::new(Line::from(Span::styled(
        &preview_title,
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )))
    .block(preview_block);
    frame.render_widget(preview, chunks[1]);

    // Help
    let help = Paragraph::new("[Enter] Select  [Backspace] Clear  [Esc] Back")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[2]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_scroll_when_list_fits() {
        assert_eq!(compute_scroll_offset(5, 10, 0), 0);
        assert_eq!(compute_scroll_offset(5, 10, 4), 0);
        assert_eq!(compute_scroll_offset(10, 10, 9), 0);
    }

    #[test]
    fn test_no_scroll_near_top() {
        // Selection within the first half-window stays at offset 0
        assert_eq!(compute_scroll_offset(64, 10, 0), 0);
        assert_eq!(compute_scroll_offset(64, 10, 5), 0);
    }

    #[test]
    fn test_center_scroll_keeps_selection_visible() {
        // Selected item is centered once past the first half-window
        assert_eq!(compute_scroll_offset(64, 10, 20), 15);
        let offset = compute_scroll_offset(64, 10, 30);
        assert!((offset..offset + 10).contains(&30));
    }

    #[test]
    fn test_scroll_clamped_at_bottom() {
        // Offset never exceeds total - visible_height
        assert_eq!(compute_scroll_offset(64, 10, 63), 54);
        assert_eq!(compute_scroll_offset(64, 10, 60), 54);
    }

    #[test]
    fn test_zero_height_does_not_panic() {
        assert_eq!(compute_scroll_offset(64, 0, 63), 63);
    }

    #[test]
    fn test_move_down_clamps_at_last_item() {
        let mut state = TitleBrowserState::new();
        for _ in 0..100 {
            state.move_down(64);
        }
        assert_eq!(state.selected_index, 63);
    }
}
