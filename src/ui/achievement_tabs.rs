//! Achievement browser category tab rendering.

use crate::achievements::{AchievementCategory, Achievements};
use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use super::achievement_browser_scene::AchievementBrowserState;

/// Render category tabs at the top of the achievement browser.
pub(super) fn render_category_tabs(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    achievements: &Achievements,
    ui_state: &AchievementBrowserState,
) {
    let mut spans = Vec::new();

    for cat in AchievementCategory::ALL {
        let style = if cat == ui_state.selected_category {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        if cat == AchievementCategory::Stats {
            spans.push(Span::styled(" Stats ", style));
        } else {
            let (unlocked, total) = achievements.count_by_category(cat);
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
    }

    let tabs = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
    frame.render_widget(tabs, area);
}
