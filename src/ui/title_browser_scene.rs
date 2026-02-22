//! Title browser overlay UI — lets the player select a displayed title.

use crate::achievements::{titles, Achievements};
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

/// Render the title browser overlay.
pub fn render_title_browser(
    frame: &mut Frame,
    area: Rect,
    achievements: &Achievements,
    ui_state: &TitleBrowserState,
    character_name: &str,
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

    // Title list
    let mut lines = Vec::new();
    for (i, title_def) in unlocked.iter().enumerate() {
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
    frame.render_widget(list, chunks[0]);

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
