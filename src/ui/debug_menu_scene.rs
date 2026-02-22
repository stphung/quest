//! Debug menu UI rendering.

use crate::achievements::UiBorderStyle;
use crate::utils::debug_menu::{
    option_count_for_category, option_label_for_index, DebugCategory, DebugMenu, DEBUG_CATEGORIES,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
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
    active_border_style: UiBorderStyle,
    _ctx: &super::responsive::LayoutContext,
) {
    let visible_options = menu.visible_option_indices();
    let max_options_in_any_tab = DEBUG_CATEGORIES
        .iter()
        .map(|category| option_count_for_category(*category))
        .max()
        .unwrap_or(1);
    let max_width = area.width.saturating_sub(2);
    let menu_width = if max_width >= 44 {
        max_width.min(72)
    } else {
        max_width.max(1)
    };

    // tabs + content + help + borders
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
        .border_style(Style::default().fg(super::themed_border_color(Color::Yellow)));
    let block = super::themed_block(block);

    let inner = block.inner(menu_area);
    frame.render_widget(block, menu_area);
    super::apply_themed_border_fx(frame, menu_area, Color::Yellow, super::BorderFxContext);

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
    let is_border_category = menu.current_category() == DebugCategory::Borders;

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

    let (options_area, preview_area) = if is_border_category && list_area.height >= 10 {
        let option_rows = (visible_options.len() as u16 + 1)
            .min(list_area.height.saturating_sub(6))
            .max(3);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(option_rows), Constraint::Min(5)])
            .split(list_area);
        (chunks[0], Some(chunks[1]))
    } else {
        (list_area, None)
    };

    let visible_rows = options_area.height as usize;
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
            let is_active_border_style =
                crate::utils::debug_menu::border_style_for_option_index(*option_index)
                    == Some(active_border_style);
            let style = if i == menu.selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if is_active_border_style {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::White)
            };
            let active_tag = if is_active_border_style {
                " [active]"
            } else {
                ""
            };
            ListItem::new(format!(
                "{}{}{}",
                prefix,
                option_label_for_index(*option_index),
                active_tag
            ))
            .style(style)
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, options_area);

    if let (Some(preview), Some(selected_style)) = (preview_area, menu.selected_border_style()) {
        render_border_preview(frame, preview, selected_style, active_border_style);
    }

    let help = if is_border_category {
        Paragraph::new("[Tab/Shift+Tab] Category  [↑/↓] Preview  [Enter] Set Style  [`] Close")
            .style(Style::default().fg(Color::DarkGray))
    } else {
        Paragraph::new("[Tab/Shift+Tab] Category  [↑/↓] Navigate  [Enter] Trigger  [`] Close")
            .style(Style::default().fg(Color::DarkGray))
    };
    frame.render_widget(help, help_area);
}

fn render_border_preview(
    frame: &mut Frame,
    area: Rect,
    preview_style: UiBorderStyle,
    active_style: UiBorderStyle,
) {
    if area.width < 24 || area.height < 5 {
        let msg = Paragraph::new("Grow terminal to preview border styles.")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(msg, area);
        return;
    }

    let border_color = preview_border_color(preview_style);
    let style_note = match preview_style {
        UiBorderStyle::Classic => "Default utility frame",
        UiBorderStyle::Rounded => "Softer premium corners",
        UiBorderStyle::Double => "Prestige frame",
        UiBorderStyle::Thick => "Heavy legendary emphasis",
        UiBorderStyle::Dashed => "Light segmented frame",
        UiBorderStyle::HeavyDashed => "Bold segmented frame",
        UiBorderStyle::TripleDashed => "Refined ornate dash pattern",
        UiBorderStyle::HeavyTripleDashed => "Max-contrast elite frame",
        UiBorderStyle::QuadDashed => "Fine technical dash pattern",
        UiBorderStyle::HeavyQuadDashed => "Dense legendary dash pattern",
    };

    let preview_block = Block::default()
        .title(format!(" Border Preview: {} ", preview_style.label()))
        .borders(Borders::ALL)
        .border_type(super::border_type_for_style(preview_style))
        .border_style(
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        );
    let preview_inner = preview_block.inner(area);
    frame.render_widget(preview_block, area);
    super::apply_border_fx_for_style(
        frame,
        area,
        border_color,
        preview_style,
        super::BorderFxContext,
    );

    if preview_inner.height < 3 {
        return;
    }

    let preview_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(preview_inner);
    let active_line = if active_style == preview_style {
        format!("{} (currently active)", style_note)
    } else {
        style_note.to_string()
    };
    frame.render_widget(
        Paragraph::new(active_line)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center),
        preview_chunks[0],
    );

    if preview_chunks[1].height < 3 {
        return;
    }

    let sample = Block::default()
        .title(" Reward Panel ")
        .borders(Borders::ALL)
        .border_type(super::border_type_for_style(preview_style))
        .border_style(
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        );
    let sample_inner = sample.inner(preview_chunks[1]);
    frame.render_widget(sample, preview_chunks[1]);
    super::apply_border_fx_for_style(
        frame,
        preview_chunks[1],
        border_color,
        preview_style,
        super::BorderFxContext,
    );

    if sample_inner.height > 0 {
        let copy = Paragraph::new(vec![
            Line::from("Legendary Reward Border"),
            Line::from("Tier: Epic"),
            Line::from("Press Enter to apply globally"),
        ])
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::White));
        frame.render_widget(copy, sample_inner);
    }
}

fn preview_border_color(style: UiBorderStyle) -> Color {
    match style {
        UiBorderStyle::Classic => Color::Yellow,
        UiBorderStyle::Rounded => Color::Yellow,
        UiBorderStyle::Double => Color::Yellow,
        UiBorderStyle::Thick => Color::Yellow,
        UiBorderStyle::Dashed => Color::Yellow,
        UiBorderStyle::HeavyDashed => Color::Yellow,
        UiBorderStyle::TripleDashed => Color::Yellow,
        UiBorderStyle::HeavyTripleDashed => Color::Yellow,
        UiBorderStyle::QuadDashed => Color::Yellow,
        UiBorderStyle::HeavyQuadDashed => Color::Yellow,
    }
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
