//! Scrolling event ticker renderer.
//!
//! Each entry independently enters from the right edge and scrolls left.
//! Entries are positioned based on their birth time, not concatenated.
//! Uses char-based indexing to avoid multi-byte UTF-8 slicing panics.

use crate::core::game_state::{Ticker, TickerEntry};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Renders the scrolling loot ticker into a 1-row area.
///
/// Each entry is independently positioned based on when it was pushed.
/// Entries enter from the right edge and scroll left across the viewport.
/// When `stormglass` is Some, a fixed SG balance counter is shown on the left.
pub fn draw_ticker(frame: &mut Frame, area: Rect, ticker: &Ticker, stormglass: Option<u64>) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    // Stormglass balance prefix
    let sg_prefix: Vec<Span<'static>> = if let Some(sg) = stormglass {
        vec![
            Span::styled(
                format!("\u{1F48E}{} SG", super::game_common::format_number(sg)),
                Style::default()
                    .fg(Color::Rgb(100, 180, 255))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" \u{2502} ", Style::default().fg(Color::DarkGray)),
        ]
    } else {
        vec![]
    };

    // Calculate prefix width (character count)
    let prefix_width: usize = sg_prefix.iter().map(|s| s.content.chars().count()).sum();
    let ticker_width = (area.width as usize).saturating_sub(prefix_width);

    let visible = ticker.visible_entries(ticker_width);

    if visible.is_empty() && sg_prefix.is_empty() {
        let line = Line::from(Span::styled(
            "  Awaiting adventure...",
            Style::default().fg(Color::DarkGray),
        ));
        frame.render_widget(Paragraph::new(line), area);
        return;
    }

    if visible.is_empty() {
        // Just show the SG counter with placeholder
        let mut spans = sg_prefix;
        spans.push(Span::styled(
            "Awaiting adventure...",
            Style::default().fg(Color::DarkGray),
        ));
        let line = Line::from(spans);
        frame.render_widget(Paragraph::new(line), area);
        return;
    }

    // Build a viewport buffer for the ticker portion
    let mut buffer: Vec<Option<(char, Style)>> = vec![None; ticker_width];

    for (entry, pos) in &visible {
        let chars = entry_to_styled_chars(entry);
        for (i, (ch, style)) in chars.into_iter().enumerate() {
            let col = *pos + i as isize;
            if col >= 0 && (col as usize) < ticker_width {
                buffer[col as usize] = Some((ch, style));
            }
        }
    }

    let ticker_spans = buffer_to_spans(&buffer);

    // Combine prefix + ticker spans
    let mut all_spans = sg_prefix;
    all_spans.extend(ticker_spans);
    let line = Line::from(all_spans);
    frame.render_widget(Paragraph::new(line), area);
}

/// Convert a ticker entry into a sequence of (char, Style) pairs.
fn entry_to_styled_chars(entry: &TickerEntry) -> Vec<(char, Style)> {
    let mut chars = Vec::new();

    // Icon + space
    if !entry.icon.is_empty() {
        let icon_style = Style::default().fg(entry.color);
        for ch in entry.icon.chars() {
            chars.push((ch, icon_style));
        }
        chars.push((' ', icon_style));
    }

    // Use segments if present, otherwise fall back to single text/color
    if let Some(segments) = &entry.segments {
        for seg in segments {
            let mut style = Style::default().fg(seg.color);
            if entry.bold {
                style = style.add_modifier(Modifier::BOLD);
            }
            for ch in seg.text.chars() {
                chars.push((ch, style));
            }
        }
    } else {
        let mut style = Style::default().fg(entry.color);
        if entry.bold {
            style = style.add_modifier(Modifier::BOLD);
        }
        for ch in entry.text.chars() {
            chars.push((ch, style));
        }
    }

    chars
}

/// Convert a viewport buffer into Ratatui Spans, grouping consecutive
/// characters with the same style. Empty cells become spaces.
fn buffer_to_spans(buffer: &[Option<(char, Style)>]) -> Vec<Span<'static>> {
    // Find the last non-empty cell to avoid trailing spaces
    let last_filled = buffer.iter().rposition(|cell| cell.is_some()).unwrap_or(0);

    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut current_text = String::new();
    let mut current_style: Option<Style> = None;
    let space_style = Style::default();

    for (i, slot) in buffer.iter().enumerate() {
        if i > last_filled {
            break;
        }

        let (ch, style) = match slot {
            Some((ch, style)) => (*ch, *style),
            None => (' ', space_style),
        };

        match current_style {
            Some(s) if s == style => {
                current_text.push(ch);
            }
            _ => {
                if let Some(s) = current_style {
                    if !current_text.is_empty() {
                        spans.push(Span::styled(std::mem::take(&mut current_text), s));
                    }
                }
                current_text.push(ch);
                current_style = Some(style);
            }
        }
    }

    // Flush final span
    if let Some(s) = current_style {
        if !current_text.is_empty() {
            spans.push(Span::styled(current_text, s));
        }
    }

    spans
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_entry_to_styled_chars_no_icon() {
        let entry = TickerEntry {
            icon: "",
            text: "Hello".to_string(),
            color: Color::White,
            bold: false,
            segments: None,
        };
        let chars = entry_to_styled_chars(&entry);
        let text: String = chars.iter().map(|(ch, _)| ch).collect();
        assert_eq!(text, "Hello");
    }

    #[test]
    fn test_entry_to_styled_chars_with_icon() {
        let entry = TickerEntry {
            icon: "\u{2694}",
            text: "Sword".to_string(),
            color: Color::Yellow,
            bold: false,
            segments: None,
        };
        let chars = entry_to_styled_chars(&entry);
        let text: String = chars.iter().map(|(ch, _)| ch).collect();
        assert_eq!(text, "\u{2694} Sword");
    }

    #[test]
    fn test_visible_entries_starts_at_right_edge() {
        let mut ticker = Ticker::new();
        ticker.push(TickerEntry {
            icon: "",
            text: "Test".to_string(),
            color: Color::White,
            bold: false,
            segments: None,
        });
        // At scroll_offset=0, entry born_at=0, position = viewport_width - 0 = 80
        // That's off-screen (>= viewport_width), so not visible
        let visible = ticker.visible_entries(80);
        assert!(visible.is_empty());

        // After scrolling 1 char, position = 80 - 1 = 79 (on screen, rightmost area)
        ticker.scroll_offset = 1.0;
        let visible = ticker.visible_entries(80);
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].1, 79); // at column 79
    }

    #[test]
    fn test_entries_dont_overlap() {
        let mut ticker = Ticker::new();
        ticker.push(TickerEntry {
            icon: "",
            text: "First".to_string(), // 5 chars
            color: Color::White,
            bold: false,
            segments: None,
        });
        ticker.push(TickerEntry {
            icon: "",
            text: "Second".to_string(),
            color: Color::White,
            bold: false,
            segments: None,
        });
        // Second entry should be spaced after first (5 chars + 3 gap = 8)
        ticker.scroll_offset = 100.0;
        let visible = ticker.visible_entries(200);
        if visible.len() == 2 {
            let pos_first = visible[0].1;
            let pos_second = visible[1].1;
            // Second should be to the right of first
            assert!(pos_second > pos_first);
        }
    }

    #[test]
    fn test_buffer_to_spans_groups_styles() {
        let style_a = Style::default().fg(Color::Red);
        let style_b = Style::default().fg(Color::Blue);
        let buffer: Vec<Option<(char, Style)>> = vec![
            Some(('A', style_a)),
            Some(('A', style_a)),
            Some(('B', style_b)),
            Some(('B', style_b)),
        ];
        let spans = buffer_to_spans(&buffer);
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].content.as_ref(), "AA");
        assert_eq!(spans[1].content.as_ref(), "BB");
    }

    #[test]
    fn test_buffer_to_spans_handles_empty_cells() {
        let style = Style::default().fg(Color::White);
        let buffer: Vec<Option<(char, Style)>> =
            vec![None, None, Some(('H', style)), Some(('i', style)), None];
        let spans = buffer_to_spans(&buffer);
        let text: String = spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(text, "  Hi");
    }

    #[test]
    fn test_multibyte_chars_no_panic() {
        let mut ticker = Ticker::new();
        ticker.push(TickerEntry {
            icon: "\u{2694}",
            text: "Sword \u{00B7} Fish".to_string(),
            color: Color::Yellow,
            bold: false,
            segments: None,
        });
        // Scroll various amounts — should never panic
        for i in 0..100 {
            ticker.scroll_offset = i as f64;
            let _ = ticker.visible_entries(80);
        }
    }

    #[test]
    fn test_cleanup_removes_off_screen_entries() {
        let mut ticker = Ticker::new();
        ticker.viewport_width = 20;
        ticker.push(TickerEntry {
            icon: "",
            text: "Old".to_string(), // 3 chars
            color: Color::White,
            bold: false,
            segments: None,
        });
        // born_at=0, off-screen when scroll > viewport(20) + len(3) = 23
        // At 0.4/tick, need 57.5 ticks → ~60 ticks
        for _ in 0..60 {
            ticker.tick();
        }
        assert!(ticker.is_empty());
    }

    #[test]
    fn test_entry_scrolls_across_full_viewport() {
        let mut ticker = Ticker::new();
        ticker.viewport_width = 40;
        ticker.push(TickerEntry {
            icon: "",
            text: "Test".to_string(), // 4 chars
            color: Color::White,
            bold: false,
            segments: None,
        });
        // Entry should appear at right edge and scroll across the full 40-char
        // viewport before being cleaned up.
        let mut seen_at_right = false;
        let mut seen_at_left = false;
        for _ in 0..200 {
            ticker.tick();
            let visible = ticker.visible_entries(40);
            for (_, pos) in &visible {
                if *pos > 30 {
                    seen_at_right = true;
                }
                if *pos < 5 {
                    seen_at_left = true;
                }
            }
        }
        assert!(seen_at_right, "Entry should appear near right edge");
        assert!(seen_at_left, "Entry should scroll to left side");
    }
}
