//! Debug menu UI rendering.

use crate::achievements::UiBorderStyle;
use crate::utils::debug_menu::{
    option_count_for_category, option_label_for_index, DebugCategory, DebugMenu, DEBUG_CATEGORIES,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs},
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

    // Tab bar using Ratatui Tabs widget.
    let tab_titles: Vec<&str> = DEBUG_CATEGORIES.iter().map(|c| c.label()).collect();
    let tabs = Tabs::new(tab_titles)
        .select(menu.selected_category)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .divider(" ");
    frame.render_widget(tabs, tabs_area);

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

    // Scroll indicators: show ▲/▼ when items are clipped.
    let total_items = visible_options.len();
    if total_items > visible_rows {
        if start > 0 {
            let arrow = Span::styled("\u{25b2}", Style::default().fg(Color::DarkGray));
            let arrow_area = Rect {
                x: options_area.x + options_area.width.saturating_sub(2),
                y: options_area.y,
                width: 1,
                height: 1,
            };
            frame.render_widget(Paragraph::new(Line::from(arrow)), arrow_area);
        }
        if start + visible_rows < total_items {
            let arrow = Span::styled("\u{25bc}", Style::default().fg(Color::DarkGray));
            let arrow_area = Rect {
                x: options_area.x + options_area.width.saturating_sub(2),
                y: options_area.y + options_area.height.saturating_sub(1),
                width: 1,
                height: 1,
            };
            frame.render_widget(Paragraph::new(Line::from(arrow)), arrow_area);
        }
    }

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
        UiBorderStyle::HeavyCorner => "Heavy corners with clean lines",
        UiBorderStyle::MicroDash => "Subtle minimalist dash frame",
        UiBorderStyle::HeaderRail => "Strong top rail for hierarchy",
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
        UiBorderStyle::HeavyCorner => Color::Yellow,
        UiBorderStyle::MicroDash => Color::Yellow,
        UiBorderStyle::HeaderRail => Color::Yellow,
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

/// A single segment of the save/commit/push indicator.
#[cfg_attr(test, derive(Debug, PartialEq))]
struct IndicatorSegment {
    content: String,
    color: Color,
}

/// Build the indicator segments based on current state.
///
/// - Save segment is always shown.
/// - Commit segment is shown only when `has_history_repo` is true.
/// - Push segment is shown only when `has_cloud_config` is true.
///
/// Before any activity, segments show dimmed icons with `---` placeholder.
#[allow(clippy::too_many_arguments)]
fn build_save_indicator_segments(
    is_saving: bool,
    last_save_time: Option<chrono::DateTime<chrono::Local>>,
    is_committing: bool,
    last_commit_time: Option<chrono::DateTime<chrono::Local>>,
    is_pushing: bool,
    last_push_time: Option<chrono::DateTime<chrono::Local>>,
    has_history_repo: bool,
    has_cloud_config: bool,
) -> Vec<IndicatorSegment> {
    use super::throbber::spinner_char;

    // Fixed text width so the indicator doesn't bounce when segments
    // transition between spinner / timestamp / placeholder states.
    // Max timestamp: "12:45 PM" = 8 chars.
    const TEXT_WIDTH: usize = 8;

    /// Format the text portion of a segment, right-padded to `TEXT_WIDTH`.
    fn pad(text: &str) -> String {
        format!("{:<width$}", text, width = TEXT_WIDTH)
    }

    let mut segments = Vec::new();

    // Save segment (always present)
    if is_saving {
        segments.push(IndicatorSegment {
            content: format!("\u{1F4BE} {}", pad(&spinner_char().to_string())),
            color: Color::Yellow,
        });
    } else if let Some(time) = last_save_time {
        segments.push(IndicatorSegment {
            content: format!("\u{1F4BE} {}", pad(&time.format("%-I:%M %p").to_string())),
            color: Color::DarkGray,
        });
    } else {
        segments.push(IndicatorSegment {
            content: format!("\u{1F4BE} {}", pad("---")),
            color: Color::DarkGray,
        });
    }

    // Commit segment (only if history repo exists)
    if has_history_repo {
        if is_committing {
            segments.push(IndicatorSegment {
                content: format!("\u{1F4DD} {}", pad(&spinner_char().to_string())),
                color: Color::Yellow,
            });
        } else if let Some(time) = last_commit_time {
            segments.push(IndicatorSegment {
                content: format!("\u{1F4DD} {}", pad(&time.format("%-I:%M %p").to_string())),
                color: Color::DarkGray,
            });
        } else {
            segments.push(IndicatorSegment {
                content: format!("\u{1F4DD} {}", pad("---")),
                color: Color::DarkGray,
            });
        }
    }

    // Push segment (only if cloud config exists)
    if has_cloud_config {
        if is_pushing {
            segments.push(IndicatorSegment {
                content: format!("\u{2601} {}", pad(&spinner_char().to_string())),
                color: Color::Cyan,
            });
        } else if let Some(time) = last_push_time {
            segments.push(IndicatorSegment {
                content: format!("\u{2601} {}", pad(&time.format("%-I:%M %p").to_string())),
                color: Color::DarkGray,
            });
        } else {
            segments.push(IndicatorSegment {
                content: format!("\u{2601} {}", pad("---")),
                color: Color::DarkGray,
            });
        }
    }

    segments
}

/// Render the multi-segment save/commit/push indicator.
///
/// Displays up to three icon segments separated by double spaces:
/// - 💾 Save: always shown when there is save state
/// - 📝 Commit: shown only if `has_history_repo` is true
/// - ☁ Push: shown only if `has_cloud_config` is true
#[allow(clippy::too_many_arguments)]
pub fn render_save_indicator(
    frame: &mut Frame,
    area: Rect,
    is_saving: bool,
    last_save_time: Option<chrono::DateTime<chrono::Local>>,
    is_committing: bool,
    last_commit_time: Option<chrono::DateTime<chrono::Local>>,
    is_pushing: bool,
    last_push_time: Option<chrono::DateTime<chrono::Local>>,
    has_history_repo: bool,
    has_cloud_config: bool,
    _ctx: &super::responsive::LayoutContext,
) {
    let segments = build_save_indicator_segments(
        is_saving,
        last_save_time,
        is_committing,
        last_commit_time,
        is_pushing,
        last_push_time,
        has_history_repo,
        has_cloud_config,
    );

    // Save segment is always present, so segments is never empty.
    if segments.is_empty() {
        return;
    }

    let mut spans: Vec<Span> = Vec::new();
    for (i, seg) in segments.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        spans.push(Span::styled(&seg.content, Style::default().fg(seg.color)));
    }

    let line = Line::from(spans);
    let display_width = line.width() as u16 + 1;

    // Position in top-right corner
    let x = area.x + area.width.saturating_sub(display_width);
    let indicator_area = Rect {
        x,
        y: area.y,
        width: display_width,
        height: 1,
    };

    frame.render_widget(Paragraph::new(line), indicator_area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;

    fn make_time() -> chrono::DateTime<chrono::Local> {
        Local::now()
    }

    #[test]
    fn save_only_saving() {
        let segments =
            build_save_indicator_segments(true, None, false, None, false, None, false, false);
        assert_eq!(segments.len(), 1);
        assert!(segments[0].content.starts_with("\u{1F4BE}"));
        assert_eq!(segments[0].color, Color::Yellow);
    }

    #[test]
    fn save_only_saved() {
        let time = make_time();
        let segments = build_save_indicator_segments(
            false,
            Some(time),
            false,
            None,
            false,
            None,
            false,
            false,
        );
        assert_eq!(segments.len(), 1);
        assert!(segments[0].content.starts_with("\u{1F4BE} "));
        assert_eq!(segments[0].color, Color::DarkGray);
    }

    #[test]
    fn save_and_commit() {
        let time = make_time();
        let segments = build_save_indicator_segments(
            false,
            Some(time),
            false,
            Some(time),
            false,
            None,
            true,
            false,
        );
        assert_eq!(segments.len(), 2);
        assert!(segments[0].content.starts_with("\u{1F4BE} "));
        assert!(segments[1].content.starts_with("\u{1F4DD} "));
    }

    #[test]
    fn all_three() {
        let time = make_time();
        let segments = build_save_indicator_segments(
            false,
            Some(time),
            false,
            Some(time),
            false,
            Some(time),
            true,
            true,
        );
        assert_eq!(segments.len(), 3);
        assert!(segments[0].content.starts_with("\u{1F4BE} "));
        assert!(segments[1].content.starts_with("\u{1F4DD} "));
        assert!(segments[2].content.starts_with("\u{2601} "));
    }

    #[test]
    fn no_commit_without_history() {
        let time = make_time();
        let segments = build_save_indicator_segments(
            false,
            Some(time),
            false,
            Some(time),
            false,
            None,
            false, // no history repo
            false,
        );
        assert_eq!(segments.len(), 1);
        assert!(segments[0].content.starts_with("\u{1F4BE} "));
    }

    #[test]
    fn no_push_without_cloud() {
        let time = make_time();
        let segments = build_save_indicator_segments(
            false,
            Some(time),
            false,
            Some(time),
            false,
            Some(time),
            true,
            false, // no cloud config
        );
        assert_eq!(segments.len(), 2);
        assert!(segments[0].content.starts_with("\u{1F4BE} "));
        assert!(segments[1].content.starts_with("\u{1F4DD} "));
    }

    #[test]
    fn default_all_configured() {
        let segments =
            build_save_indicator_segments(false, None, false, None, false, None, true, true);
        assert_eq!(segments.len(), 3);
        // "---" padded to 8 chars
        assert_eq!(segments[0].content, "\u{1F4BE} ---     ");
        assert_eq!(segments[1].content, "\u{1F4DD} ---     ");
        assert_eq!(segments[2].content, "\u{2601} ---     ");
        assert!(segments.iter().all(|s| s.color == Color::DarkGray));
    }

    #[test]
    fn default_no_history_no_cloud() {
        let segments =
            build_save_indicator_segments(false, None, false, None, false, None, false, false);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].content, "\u{1F4BE} ---     ");
        assert_eq!(segments[0].color, Color::DarkGray);
    }

    #[test]
    fn default_with_history_only() {
        let segments =
            build_save_indicator_segments(false, None, false, None, false, None, true, false);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].content, "\u{1F4BE} ---     ");
        assert_eq!(segments[1].content, "\u{1F4DD} ---     ");
    }

    #[test]
    fn fixed_width_segments() {
        use unicode_width::UnicodeWidthStr;
        // All segment states should produce the same display width
        let saving =
            build_save_indicator_segments(true, None, false, None, false, None, false, false);
        let default =
            build_save_indicator_segments(false, None, false, None, false, None, false, false);
        let saved = build_save_indicator_segments(
            false,
            Some(make_time()),
            false,
            None,
            false,
            None,
            false,
            false,
        );
        let saving_w = UnicodeWidthStr::width(saving[0].content.as_str());
        let default_w = UnicodeWidthStr::width(default[0].content.as_str());
        let saved_w = UnicodeWidthStr::width(saved[0].content.as_str());
        assert_eq!(saving_w, default_w);
        assert_eq!(saving_w, saved_w);
    }
}
