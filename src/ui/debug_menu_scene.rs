//! Debug menu UI rendering.

use crate::debug_menu::{DebugMenu, DEBUG_OPTIONS};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

/// Render the debug menu overlay
pub fn render_debug_menu(frame: &mut Frame, area: Rect, menu: &DebugMenu) {
    // Center the menu
    let menu_width = 35;
    let menu_height = (DEBUG_OPTIONS.len() + 4) as u16; // options + border + help
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

    // Menu items
    let items: Vec<ListItem> = DEBUG_OPTIONS
        .iter()
        .enumerate()
        .map(|(i, option)| {
            let prefix = if i == menu.selected_index { "> " } else { "  " };
            let style = if i == menu.selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(format!("{}{}", prefix, option)).style(style)
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);

    // Help text at bottom
    if inner.height > DEBUG_OPTIONS.len() as u16 {
        let help_area = Rect {
            x: inner.x,
            y: inner.y + inner.height - 1,
            width: inner.width,
            height: 1,
        };
        let help = Paragraph::new("[↑/↓] Navigate  [Enter] Trigger  [`] Close")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help, help_area);
    }
}

/// Render the debug mode indicator
pub fn render_debug_indicator(frame: &mut Frame, area: Rect) {
    let indicator = Paragraph::new(Line::from("[DEBUG]")).style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );

    // Position in top-right corner
    let x = area.x + area.width.saturating_sub(9);
    let indicator_area = Rect {
        x,
        y: area.y,
        width: 9,
        height: 1,
    };

    frame.render_widget(indicator, indicator_area);
}
