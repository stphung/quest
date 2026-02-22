//! Debug menu UI rendering.

use crate::utils::debug_menu::{DebugMenu, DEBUG_CATEGORIES, DEBUG_OPTIONS};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

/// Render the debug menu overlay
pub fn render_debug_menu(
    frame: &mut Frame,
    area: Rect,
    menu: &DebugMenu,
    _ctx: &super::responsive::LayoutContext,
) {
    let visible_options = menu.visible_option_indices();
    let max_options_in_any_tab = DEBUG_CATEGORIES
        .iter()
        .map(|category| category.option_indices().len())
        .max()
        .unwrap_or(1);
    let max_width = area.width.saturating_sub(2);
    let menu_width = if max_width >= 44 {
        max_width.min(72)
    } else {
        max_width.max(1)
    };

    // tabs + options + help + borders
    let max_height = area.height.saturating_sub(2);
    let desired_height = (max_options_in_any_tab + 6) as u16;
    let menu_height = if max_height >= 8 {
        desired_height.min(max_height)
    } else {
        max_height.max(1)
    };

    let x = area.x + (area.width.saturating_sub(menu_width)) / 2;
    let y = area.y + (area.height.saturating_sub(menu_height)) / 2;

    let menu_area = Rect {
        x,
        y,
        width: menu_width,
        height: menu_height,
    };

    // Clear background
    frame.render_widget(Clear, menu_area);

    let block = Block::default()
        .title(" Debug Menu ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(menu_area);
    frame.render_widget(block, menu_area);

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(inner);
    let tabs_area = sections[0];
    let list_area = sections[1];
    let help_area = sections[2];

    let mut tab_spans = Vec::new();
    for (i, category) in DEBUG_CATEGORIES.iter().enumerate() {
        if i > 0 {
            tab_spans.push(Span::raw(" "));
        }
        let label = format!("[{}]", category.label());
        let style = if i == menu.selected_category {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        tab_spans.push(Span::styled(label, style));
    }
    frame.render_widget(Paragraph::new(Line::from(tab_spans)), tabs_area);

    let visible_rows = list_area.height as usize;
    let start = if menu.selected_index >= visible_rows {
        menu.selected_index + 1 - visible_rows
    } else {
        0
    };
    let items: Vec<ListItem> = visible_options
        .iter()
        .enumerate()
        .skip(start)
        .take(visible_rows)
        .map(|(i, option_index)| {
            let prefix = if i == menu.selected_index { "> " } else { "  " };
            let style = if i == menu.selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(format!("{}{}", prefix, DEBUG_OPTIONS[*option_index])).style(style)
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, list_area);

    let help =
        Paragraph::new("[Tab/Shift+Tab] Category  [↑/↓] Navigate  [Enter] Trigger  [`] Close")
            .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help, help_area);
}

/// Render the debug mode indicator (shows saves are disabled)
pub fn render_debug_indicator(
    frame: &mut Frame,
    area: Rect,
    _ctx: &super::responsive::LayoutContext,
) {
    let text = "[DEBUG] Saves disabled";
    let indicator = Paragraph::new(Line::from(text)).style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );

    // Position in top-right corner
    let width = text.len() as u16 + 1;
    let x = area.x + area.width.saturating_sub(width);
    let indicator_area = Rect {
        x,
        y: area.y,
        width,
        height: 1,
    };

    frame.render_widget(indicator, indicator_area);
}

/// Render the save indicator (spinner while saving, timestamp after)
/// `is_saving` should be true for ~1 second after a save completes
pub fn render_save_indicator(
    frame: &mut Frame,
    area: Rect,
    is_saving: bool,
    last_save_time: Option<chrono::DateTime<chrono::Local>>,
    _ctx: &super::responsive::LayoutContext,
) {
    use super::throbber::spinner_char;

    let text = if is_saving {
        format!("{} Saving...", spinner_char())
    } else if let Some(time) = last_save_time {
        format!("Saved {}", time.format("%-I:%M %p"))
    } else {
        return; // No save yet, don't show anything
    };

    let color = if is_saving {
        Color::Yellow
    } else {
        Color::DarkGray
    };

    let indicator = Paragraph::new(Line::from(text.clone())).style(Style::default().fg(color));

    // Position in top-right corner
    let width = text.len() as u16 + 1;
    let x = area.x + area.width.saturating_sub(width);
    let indicator_area = Rect {
        x,
        y: area.y,
        width,
        height: 1,
    };

    frame.render_widget(indicator, indicator_area);
}
