//! Achievement browser overlay UI.
//!
//! Displays a browsable list of achievements organized by category,
//! with a detail panel showing description and unlock status.

use crate::achievements::{
    get_achievement_def, get_achievements_by_category, AchievementCategory, AchievementId,
    Achievements,
};
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
            AchievementCategory::Combat => AchievementCategory::Level,
            AchievementCategory::Level => AchievementCategory::Progression,
            AchievementCategory::Progression => AchievementCategory::Challenges,
            AchievementCategory::Challenges => AchievementCategory::Exploration,
            AchievementCategory::Exploration => AchievementCategory::Combat,
        };
        self.selected_index = 0;
    }

    pub fn prev_category(&mut self) {
        self.selected_category = match self.selected_category {
            AchievementCategory::Combat => AchievementCategory::Exploration,
            AchievementCategory::Level => AchievementCategory::Combat,
            AchievementCategory::Progression => AchievementCategory::Level,
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
    _ctx: &super::responsive::LayoutContext,
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

        let new_count = achievements.count_recently_unlocked_by_category(cat);
        if new_count > 0 {
            spans.push(Span::styled(
                format!(" {} ({}/{}) +{} ", cat.name(), unlocked, total, new_count),
                style,
            ));
        } else {
            spans.push(Span::styled(
                format!(" {} ({}/{}) ", cat.name(), unlocked, total),
                style,
            ));
        }
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
            let is_new = achievements.is_recently_unlocked(def.id);

            let prefix = if is_selected { "> " } else { "  " };
            let checkmark = if is_unlocked { "[X] " } else { "[ ] " };

            let style = if is_unlocked {
                Style::default().fg(Color::Green)
            } else if is_selected {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let mut spans = vec![
                Span::styled(prefix, style),
                Span::styled(
                    checkmark,
                    if is_unlocked {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                ),
                Span::raw(format!("{} ", def.icon)),
                Span::styled(def.name, style),
            ];

            if is_new {
                spans.push(Span::styled(
                    " [NEW]",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ));
            }

            ListItem::new(Line::from(spans))
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
    let block = Block::default()
        .title(format!(" {} ", def.name))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if is_unlocked {
            Color::Green
        } else {
            Color::DarkGray
        }));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = Vec::new();

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

            // Show completed progress bar for milestone achievements
            if let Some(progress) = achievements.get_progress(def.id) {
                let display_current = progress.target;
                lines.push(Line::from(vec![
                    Span::styled("    [", Style::default().fg(Color::DarkGray)),
                    Span::styled("\u{2588}".repeat(20), Style::default().fg(Color::Green)),
                    Span::styled("] ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{}/{}", display_current, progress.target),
                        Style::default().fg(Color::Green),
                    ),
                ]));
            }

            if achievements.is_recently_unlocked(def.id) {
                lines.push(Line::from(Span::styled(
                    "[NEW] Recently unlocked!",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
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
                format!("    Progress: {}/{}", progress.current, progress.target),
                Style::default().fg(Color::Yellow),
            )));

            // Progress bar
            let bar_width = 20usize;
            let filled = if progress.target > 0 {
                (progress.current as usize * bar_width / progress.target as usize).min(bar_width)
            } else {
                0
            };
            let empty = bar_width - filled;
            lines.push(Line::from(vec![
                Span::styled("    [", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "\u{2588}".repeat(filled),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    "\u{2591}".repeat(empty),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled("] ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{}%", percent), Style::default().fg(Color::Yellow)),
            ]));
        }
    }

    let para = Paragraph::new(lines).wrap(Wrap { trim: true });
    frame.render_widget(para, inner);
}

/// Render the achievement unlocked celebration modal.
pub fn render_achievement_unlocked_modal(
    frame: &mut Frame,
    area: Rect,
    achievements: &[AchievementId],
    _ctx: &super::responsive::LayoutContext,
) {
    if achievements.is_empty() {
        return;
    }

    let is_single = achievements.len() == 1;
    let modal_height = if is_single {
        9u16.min(area.height.saturating_sub(4))
    } else {
        ((6 + achievements.len()).min(20) as u16).min(area.height.saturating_sub(4))
    };
    let modal_width = 50u16.min(area.width.saturating_sub(4));

    // Center the modal
    let x = area.x + (area.width.saturating_sub(modal_width)) / 2;
    let y = area.y + (area.height.saturating_sub(modal_height)) / 2;
    let modal_area = Rect::new(x, y, modal_width, modal_height);

    frame.render_widget(Clear, modal_area);

    let title = if is_single {
        " Achievement Unlocked! "
    } else {
        " Achievements Unlocked! "
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(modal_area);
    frame.render_widget(block, modal_area);

    let mut lines = vec![Line::from("")];

    if is_single {
        // Single achievement: show icon, name, and description
        if let Some(def) = get_achievement_def(achievements[0]) {
            lines.push(Line::from(Span::styled(
                format!("{}  {}", def.icon, def.name),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                def.description,
                Style::default().fg(Color::White),
            )));
        }
    } else {
        // Multiple achievements: show count and list
        lines.push(Line::from(Span::styled(
            format!("🏆  {} achievements!", achievements.len()),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        for id in achievements.iter().take(12) {
            if let Some(def) = get_achievement_def(*id) {
                lines.push(Line::from(Span::styled(
                    format!("  {}  {}", def.icon, def.name),
                    Style::default().fg(Color::White),
                )));
            }
        }

        if achievements.len() > 12 {
            lines.push(Line::from(Span::styled(
                format!("  ...and {} more", achievements.len() - 12),
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("[Enter] to continue", Style::default().fg(Color::DarkGray)),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled("A = Achievements", Style::default().fg(Color::Magenta)),
    ]));

    let para = Paragraph::new(lines).alignment(Alignment::Center);
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
        assert_eq!(state.selected_category, AchievementCategory::Level);
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
