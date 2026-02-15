//! Scrolling loot ticker renderer.
//!
//! Renders a 1-row horizontal ticker by computing a visible window
//! into a virtual string built from concatenated TickerEntry spans.

#![allow(dead_code)]

use crate::core::game_state::LootTicker;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Separator between ticker entries.
const SEPARATOR: &str = " \u{00B7}\u{00B7}\u{00B7} ";

/// Renders the scrolling loot ticker into a 1-row area.
pub fn draw_ticker(frame: &mut Frame, area: Rect, ticker: &LootTicker) {
    if area.height == 0 || area.width == 0 {
        return;
    }

    if ticker.entries.is_empty() {
        let line = Line::from(Span::styled(
            "  Awaiting adventure...",
            Style::default().fg(Color::DarkGray),
        ));
        frame.render_widget(Paragraph::new(line), area);
        return;
    }

    // Build the virtual ticker content as a list of (text, style) segments.
    // We iterate entries oldest-to-newest (back to front) so the ticker
    // reads chronologically from left to right.
    let segments = build_segments(ticker);

    // Calculate total virtual width
    let total_width: usize = segments.iter().map(|(text, _)| text.len()).sum();
    if total_width == 0 {
        return;
    }

    let visible_width = area.width as usize;

    // Wrap the scroll offset within total_width for seamless looping
    let offset = (ticker.scroll_offset as usize) % total_width;

    // Extract the visible slice, handling wrap-around
    let spans = extract_visible_spans(&segments, offset, visible_width, total_width);

    let line = Line::from(spans);
    frame.render_widget(Paragraph::new(line), area);
}

/// Builds a flat list of (text, style) segments from ticker entries.
/// Entries are iterated back-to-front (oldest first) for left-to-right chronology.
fn build_segments(ticker: &LootTicker) -> Vec<(String, Style)> {
    let mut segments = Vec::new();
    let sep_style = Style::default().fg(Color::DarkGray);

    for (i, entry) in ticker.entries.iter().rev().enumerate() {
        if i > 0 {
            segments.push((SEPARATOR.to_string(), sep_style));
        }

        // Icon + space
        if !entry.icon.is_empty() {
            segments.push((format!("{} ", entry.icon), Style::default().fg(entry.color)));
        }

        // Main text
        let mut style = Style::default().fg(entry.color);
        if entry.bold {
            style = style.add_modifier(Modifier::BOLD);
        }
        segments.push((entry.text.clone(), style));
    }

    // Add trailing separator for seamless loop
    if !segments.is_empty() {
        segments.push((SEPARATOR.to_string(), sep_style));
    }

    segments
}

/// Extracts visible Span slice from segments at the given offset.
/// Handles wrap-around when offset + visible_width > total_width.
fn extract_visible_spans(
    segments: &[(String, Style)],
    offset: usize,
    visible_width: usize,
    total_width: usize,
) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut chars_emitted = 0;

    // We may need to traverse the segments twice for wrap-around
    let mut global_pos = 0;

    // Find starting segment and offset within it
    for pass in 0..2 {
        for (text, style) in segments {
            let seg_len = text.len();
            let seg_start = global_pos;
            let seg_end = seg_start + seg_len;

            global_pos = seg_end;

            // Current position in the virtual (possibly wrapped) space
            let effective_start = seg_start + pass * total_width;
            let effective_end = seg_end + pass * total_width;

            let window_start = offset;
            let window_end = offset + visible_width;

            // Check if this segment overlaps with the visible window
            if effective_end <= window_start || effective_start >= window_end {
                continue;
            }

            // Calculate the slice of this segment that's visible
            let slice_start = window_start.saturating_sub(effective_start);
            let remaining = visible_width - chars_emitted;
            let slice_end = (seg_len).min(slice_start + remaining);

            if slice_start < slice_end {
                let visible_text = &text[slice_start..slice_end];
                spans.push(Span::styled(visible_text.to_string(), *style));
                chars_emitted += slice_end - slice_start;
            }

            if chars_emitted >= visible_width {
                return spans;
            }
        }

        // Reset for second pass (wrap-around)
        global_pos = 0;
    }

    spans
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::game_state::TickerEntry;

    fn make_ticker(entries: Vec<(&'static str, &str, Color)>) -> LootTicker {
        let mut ticker = LootTicker::new();
        for (icon, text, color) in entries.into_iter().rev() {
            ticker.push(TickerEntry {
                icon,
                text: text.to_string(),
                color,
                bold: false,
            });
        }
        ticker
    }

    #[test]
    fn test_build_segments_empty() {
        let ticker = LootTicker::new();
        let segments = build_segments(&ticker);
        assert!(segments.is_empty());
    }

    #[test]
    fn test_build_segments_single_entry() {
        let ticker = make_ticker(vec![("\u{2694}", "[R] Sword", Color::Yellow)]);
        let segments = build_segments(&ticker);
        // Should have: icon+space, text, trailing separator
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].0, "\u{2694} ");
        assert_eq!(segments[1].0, "[R] Sword");
        assert_eq!(segments[2].0, SEPARATOR);
    }

    #[test]
    fn test_build_segments_multiple_entries() {
        let ticker = make_ticker(vec![
            ("\u{2694}", "[R] Sword", Color::Yellow),
            ("\u{1F41F}", "Trout [C]", Color::Gray),
        ]);
        let segments = build_segments(&ticker);
        // icon, text, sep, icon, text, trailing_sep
        assert_eq!(segments.len(), 6);
    }

    #[test]
    fn test_extract_visible_at_zero() {
        let segments = vec![
            ("Hello".to_string(), Style::default()),
            (" World".to_string(), Style::default()),
        ];
        let spans = extract_visible_spans(&segments, 0, 5, 11);
        let text: String = spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(text, "Hello");
    }

    #[test]
    fn test_extract_visible_with_offset() {
        let segments = vec![
            ("Hello".to_string(), Style::default()),
            (" World".to_string(), Style::default()),
        ];
        let spans = extract_visible_spans(&segments, 5, 6, 11);
        let text: String = spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(text, " World");
    }

    #[test]
    fn test_extract_visible_wraps_around() {
        let segments = vec![("ABCDE".to_string(), Style::default())];
        // Offset 3, width 4, total 5 -> should get "DE" + "AB"
        let spans = extract_visible_spans(&segments, 3, 4, 5);
        let text: String = spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(text, "DEAB");
    }
}
