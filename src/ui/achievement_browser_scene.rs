//! Achievement browser overlay UI.
//!
//! Displays a browsable list of achievements organized by category,
//! with a detail panel showing description and unlock status.

use super::achievement_details::format_number;
use crate::achievements::{get_achievement_def, AchievementCategory, AchievementId, Achievements};
use crate::enhancement::EnhancementProgress;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
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
            AchievementCategory::Exploration => AchievementCategory::Stats,
            AchievementCategory::Stats => AchievementCategory::Combat,
        };
        self.selected_index = 0;
    }

    pub fn prev_category(&mut self) {
        self.selected_category = match self.selected_category {
            AchievementCategory::Combat => AchievementCategory::Stats,
            AchievementCategory::Stats => AchievementCategory::Exploration,
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
    enhancement: &EnhancementProgress,
    _ctx: &super::responsive::LayoutContext,
) {
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(
            " Achievements ({}/{} pts, {:.1}%) ",
            format_number(achievements.achievement_score() as u64),
            format_number(Achievements::max_achievement_score() as u64),
            achievements.unlock_percentage()
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(super::themed_border_color(Color::Yellow)));
    let inner =
        super::render_themed_block(frame, area, block, Color::Yellow, super::BorderFxContext);

    // Layout: Category tabs at top, list on left, detail on right, help at bottom
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Category tabs
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Help
        ])
        .split(inner);

    super::achievement_tabs::render_category_tabs(frame, chunks[0], achievements, ui_state);

    // Content area: stats view or list+detail
    if ui_state.selected_category == AchievementCategory::Stats {
        super::achievement_details::render_stats_view(
            frame,
            chunks[1],
            achievements,
            enhancement,
            ui_state.selected_index,
        );
    } else {
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(chunks[1]);

        super::achievement_list::render_achievement_list(
            frame,
            content_chunks[0],
            achievements,
            ui_state,
        );
        super::achievement_details::render_achievement_detail(
            frame,
            content_chunks[1],
            achievements,
            ui_state,
        );
    }

    let help = Paragraph::new("[</>] Category  [Up/Down] Select  [T] Titles  [Esc] Close")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[2]);
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
        .border_style(Style::default().fg(super::themed_border_color(Color::Yellow)));
    let inner = super::render_themed_block(
        frame,
        modal_area,
        block,
        Color::Yellow,
        super::BorderFxContext,
    );

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
            lines.push(Line::from(Span::styled(
                format!("+{} pts", def.points),
                Style::default().fg(Color::Cyan),
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

        let total_pts: u32 = achievements
            .iter()
            .filter_map(|id| get_achievement_def(*id))
            .map(|def| def.points)
            .sum();
        lines.push(Line::from(Span::styled(
            format!("+{} pts", total_pts),
            Style::default().fg(Color::Cyan),
        )));
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
        assert_eq!(state.selected_category, AchievementCategory::Stats);
        state.next_category();
        assert_eq!(state.selected_category, AchievementCategory::Combat);

        state.prev_category();
        assert_eq!(state.selected_category, AchievementCategory::Stats);
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

    #[test]
    fn test_stats_tab_navigation() {
        let mut state = AchievementBrowserState::new();

        // Navigate to Stats tab
        state.next_category(); // Level
        state.next_category(); // Progression
        state.next_category(); // Challenges
        state.next_category(); // Exploration
        state.next_category(); // Stats
        assert_eq!(state.selected_category, AchievementCategory::Stats);

        // Stats wraps to Combat
        state.next_category();
        assert_eq!(state.selected_category, AchievementCategory::Combat);

        // Backward from Combat goes to Stats
        state.prev_category();
        assert_eq!(state.selected_category, AchievementCategory::Stats);

        // Backward from Stats goes to Exploration
        state.prev_category();
        assert_eq!(state.selected_category, AchievementCategory::Exploration);
    }

    #[test]
    fn test_format_number() {
        use super::super::achievement_details::format_number;
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(999), "999");
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(12847), "12,847");
        assert_eq!(format_number(1000000), "1,000,000");
    }
}
