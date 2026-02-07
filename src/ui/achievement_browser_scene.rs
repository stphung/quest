//! Achievement browser overlay UI.
//!
//! Displays a browsable list of achievements organized by category,
//! with a detail panel showing description and unlock status.

use crate::achievements::{get_achievements_by_category, AchievementCategory, Achievements};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// UI state for the achievement browser overlay.
pub struct AchievementBrowserState {
    pub showing: bool,
    pub selected_category: AchievementCategory,
    pub selected_index: usize,
}

impl AchievementBrowserState {
    pub fn new() -> Self {
        Self {
            showing: false,
            selected_category: AchievementCategory::Combat,
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

    pub fn next_category(&mut self) {
        self.selected_category = match self.selected_category {
            AchievementCategory::Combat => AchievementCategory::Progression,
            AchievementCategory::Progression => AchievementCategory::Challenges,
            AchievementCategory::Challenges => AchievementCategory::Exploration,
            AchievementCategory::Exploration => AchievementCategory::Combat,
        };
        self.selected_index = 0;
    }

    pub fn prev_category(&mut self) {
        self.selected_category = match self.selected_category {
            AchievementCategory::Combat => AchievementCategory::Exploration,
            AchievementCategory::Progression => AchievementCategory::Combat,
            AchievementCategory::Challenges => AchievementCategory::Progression,
            AchievementCategory::Exploration => AchievementCategory::Challenges,
        };
        self.selected_index = 0;
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

impl Default for AchievementBrowserState {
    fn default() -> Self {
        Self::new()
    }
}

/// Render the achievement browser overlay.
pub fn render_achievement_browser(
    frame: &mut Frame,
    area: Rect,
    achievements: &Achievements,
    ui_state: &AchievementBrowserState,
) {
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(
            " Achievements ({:.1}% Complete) ",
            achievements.unlock_percentage()
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Layout: Category tabs at top, list on left, detail on right, help at bottom
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Category tabs
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Help
        ])
        .split(inner);

    render_category_tabs(frame, chunks[0], achievements, ui_state);

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(chunks[1]);

    render_achievement_list(frame, content_chunks[0], achievements, ui_state);
    render_achievement_detail(frame, content_chunks[1], achievements, ui_state);

    let help = Paragraph::new("[</>] Category  [Up/Down] Select  [Esc] Close")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[2]);
}

fn render_category_tabs(
    frame: &mut Frame,
    area: Rect,
    achievements: &Achievements,
    ui_state: &AchievementBrowserState,
) {
    let mut spans = Vec::new();

    for cat in AchievementCategory::ALL {
        let (unlocked, total) = achievements.count_by_category(cat);

        let style = if cat == ui_state.selected_category {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        spans.push(Span::styled(
            format!(" {} ({}/{}) ", cat.name(), unlocked, total),
            style,
        ));
    }

    let tabs = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
    frame.render_widget(tabs, area);
}

fn render_achievement_list(
    frame: &mut Frame,
    area: Rect,
    achievements: &Achievements,
    ui_state: &AchievementBrowserState,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let category_achievements = get_achievements_by_category(ui_state.selected_category);

    let items: Vec<ListItem> = category_achievements
        .iter()
        .enumerate()
        .map(|(i, def)| {
            let is_unlocked = achievements.is_unlocked(def.id);
            let is_selected = i == ui_state.selected_index;

            let (icon, name) = if is_unlocked || !def.secret {
                (def.icon, def.name)
            } else {
                ("?", "???")
            };

            let prefix = if is_selected { "> " } else { "  " };
            let checkmark = if is_unlocked { "[X] " } else { "[ ] " };

            let style = if is_unlocked {
                Style::default().fg(Color::Green)
            } else if is_selected {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(
                    checkmark,
                    if is_unlocked {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                ),
                Span::raw(format!("{} ", icon)),
                Span::styled(name, style),
            ]))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}

fn render_achievement_detail(
    frame: &mut Frame,
    area: Rect,
    achievements: &Achievements,
    ui_state: &AchievementBrowserState,
) {
    let category_achievements = get_achievements_by_category(ui_state.selected_category);

    let Some(def) = category_achievements.get(ui_state.selected_index) else {
        return;
    };

    let is_unlocked = achievements.is_unlocked(def.id);
    let show_details = is_unlocked || !def.secret;

    let title = if show_details { def.name } else { "???" };
    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if is_unlocked {
            Color::Green
        } else {
            Color::DarkGray
        }));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = Vec::new();

    if show_details {
        // Icon and name
        lines.push(Line::from(Span::styled(
            format!("{} {}", def.icon, def.name),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        // Description
        lines.push(Line::from(Span::styled(
            def.description,
            Style::default().fg(Color::White),
        )));
        lines.push(Line::from(""));

        // Unlock status
        if is_unlocked {
            if let Some(record) = achievements.unlocked.get(&def.id) {
                let timestamp = chrono::DateTime::from_timestamp(record.unlocked_at, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                lines.push(Line::from(Span::styled(
                    format!("[X] Unlocked: {}", timestamp),
                    Style::default().fg(Color::Green),
                )));

                if let Some(ref char_name) = record.character_name {
                    lines.push(Line::from(Span::styled(
                        format!("    By: {}", char_name),
                        Style::default().fg(Color::DarkGray),
                    )));
                }
            }
        } else {
            lines.push(Line::from(Span::styled(
                "[ ] Not yet unlocked",
                Style::default().fg(Color::Red),
            )));

            // Show progress if applicable
            if let Some(progress) = achievements.get_progress(def.id) {
                let percent = if progress.target > 0 {
                    (progress.current as f64 / progress.target as f64 * 100.0) as u32
                } else {
                    0
                };
                lines.push(Line::from(Span::styled(
                    format!(
                        "    Progress: {}/{} ({}%)",
                        progress.current, progress.target, percent
                    ),
                    Style::default().fg(Color::Yellow),
                )));
            }
        }
    } else {
        lines.push(Line::from(Span::styled(
            "This achievement is hidden.",
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Unlock it to reveal its details.",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let para = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(para, inner);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_achievement_browser_state_navigation() {
        let mut state = AchievementBrowserState::new();

        // Initial state
        assert!(!state.showing);
        assert_eq!(state.selected_category, AchievementCategory::Combat);
        assert_eq!(state.selected_index, 0);

        // Open
        state.open();
        assert!(state.showing);
        assert_eq!(state.selected_index, 0);

        // Navigate categories
        state.next_category();
        assert_eq!(state.selected_category, AchievementCategory::Progression);
        state.next_category();
        assert_eq!(state.selected_category, AchievementCategory::Challenges);
        state.next_category();
        assert_eq!(state.selected_category, AchievementCategory::Exploration);
        state.next_category();
        assert_eq!(state.selected_category, AchievementCategory::Combat);

        state.prev_category();
        assert_eq!(state.selected_category, AchievementCategory::Exploration);

        // Navigate items
        state.move_down(10);
        assert_eq!(state.selected_index, 1);
        state.move_up();
        assert_eq!(state.selected_index, 0);
        state.move_up();
        assert_eq!(state.selected_index, 0); // Can't go below 0

        // Close
        state.close();
        assert!(!state.showing);
    }
}
