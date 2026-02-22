//! Achievement browser list panel rendering.

use crate::achievements::{get_achievements_by_category, get_title_text, Achievements};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use super::achievement_browser_scene::AchievementBrowserState;

/// Render the scrollable achievement list panel.
pub(super) fn render_achievement_list(
    frame: &mut Frame,
    area: Rect,
    achievements: &Achievements,
    ui_state: &AchievementBrowserState,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(super::themed_border_color(Color::DarkGray)));
    let inner =
        super::render_themed_block(frame, area, block, Color::DarkGray, super::BorderFxContext);

    let category_achievements = get_achievements_by_category(ui_state.selected_category);
    let total = category_achievements.len();
    let visible_height = inner.height as usize;

    // Calculate scroll offset to keep selected item visible (center-scroll)
    let max_scroll = total.saturating_sub(visible_height);
    let scroll_offset = if total <= visible_height {
        0
    } else {
        ui_state
            .selected_index
            .saturating_sub(visible_height / 2)
            .min(max_scroll)
    };

    let items: Vec<ListItem> = category_achievements
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(visible_height)
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

            if let Some(title) = get_title_text(def.id) {
                spans.push(Span::styled(
                    format!("  \u{2726} {title}"),
                    Style::default().fg(Color::Magenta),
                ));
            }

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

    // Scroll indicators when content overflows
    if total > visible_height {
        let indicator_style = Style::default().fg(Color::DarkGray);
        if scroll_offset > 0 {
            let up = Paragraph::new(Line::from(Span::styled(" \u{25b2}", indicator_style)))
                .alignment(Alignment::Right);
            frame.render_widget(up, Rect::new(inner.x, inner.y, inner.width, 1));
        }
        if scroll_offset < max_scroll {
            let down = Paragraph::new(Line::from(Span::styled(" \u{25bc}", indicator_style)))
                .alignment(Alignment::Right);
            frame.render_widget(
                down,
                Rect::new(inner.x, inner.y + inner.height - 1, inner.width, 1),
            );
        }
    }
}
