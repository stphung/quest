//! Scrolling loot ticker renderer.
//!
//! Renders a 1-row horizontal ticker by computing a visible window
//! into a virtual character array built from concatenated TickerEntry spans.
//! Uses char-based indexing to avoid multi-byte UTF-8 slicing panics.

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

/// A character with its associated style for the virtual ticker content.
struct StyledChar {
    ch: char,
    style: Style,
}

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

    let chars = build_chars(ticker);
    if chars.is_empty() {
        return;
    }

    let visible_width = area.width as usize;
    let offset = (ticker.scroll_offset as usize) % chars.len();

    let spans = extract_visible_spans(&chars, offset, visible_width);
    let line = Line::from(spans);
    frame.render_widget(Paragraph::new(line), area);
}

/// Builds a flat list of styled characters from ticker entries.
/// Entries are iterated back-to-front (oldest first) for left-to-right chronology.
fn build_chars(ticker: &LootTicker) -> Vec<StyledChar> {
    let mut chars = Vec::new();
    let sep_style = Style::default().fg(Color::DarkGray);

    for (i, entry) in ticker.entries.iter().rev().enumerate() {
        // Add separator between entries
        if i > 0 {
            for ch in SEPARATOR.chars() {
                chars.push(StyledChar {
                    ch,
                    style: sep_style,
                });
            }
        }

        // Icon + space
        if !entry.icon.is_empty() {
            let icon_style = Style::default().fg(entry.color);
            for ch in entry.icon.chars() {
                chars.push(StyledChar {
                    ch,
                    style: icon_style,
                });
            }
            chars.push(StyledChar {
                ch: ' ',
                style: icon_style,
            });
        }

        // Main text
        let mut style = Style::default().fg(entry.color);
        if entry.bold {
            style = style.add_modifier(Modifier::BOLD);
        }
        for ch in entry.text.chars() {
            chars.push(StyledChar { ch, style });
        }
    }

    // Trailing separator for seamless loop
    if !chars.is_empty() {
        for ch in SEPARATOR.chars() {
            chars.push(StyledChar {
                ch,
                style: sep_style,
            });
        }
    }

    chars
}

/// Extracts visible Spans from the char array at the given offset.
/// Handles wrap-around by indexing modulo total_len.
fn extract_visible_spans(
    chars: &[StyledChar],
    offset: usize,
    visible_width: usize,
) -> Vec<Span<'static>> {
    if chars.is_empty() {
        return Vec::new();
    }

    let total_len = chars.len();
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut current_text = String::new();
    let mut current_style: Option<Style> = None;

    for i in 0..visible_width {
        let idx = (offset + i) % total_len;
        let sc = &chars[idx];

        match current_style {
            Some(s) if s == sc.style => {
                current_text.push(sc.ch);
            }
            _ => {
                // Flush previous span
                if let Some(s) = current_style {
                    if !current_text.is_empty() {
                        spans.push(Span::styled(std::mem::take(&mut current_text), s));
                    }
                }
                current_text.push(sc.ch);
                current_style = Some(sc.style);
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
    fn test_build_chars_empty() {
        let ticker = LootTicker::new();
        let chars = build_chars(&ticker);
        assert!(chars.is_empty());
    }

    #[test]
    fn test_build_chars_single_entry_no_icon() {
        let mut ticker = LootTicker::new();
        ticker.push(TickerEntry {
            icon: "",
            text: "Hello".to_string(),
            color: Color::White,
            bold: false,
        });
        let chars = build_chars(&ticker);
        // "Hello" + trailing separator " ··· "
        let text: String = chars.iter().map(|sc| sc.ch).collect();
        assert!(text.starts_with("Hello"));
        assert!(text.ends_with(' ')); // trailing separator ends with space
    }

    #[test]
    fn test_build_chars_with_icon() {
        let ticker = make_ticker(vec![("\u{2694}", "Sword", Color::Yellow)]);
        let chars = build_chars(&ticker);
        let text: String = chars.iter().map(|sc| sc.ch).collect();
        // Should contain: icon + space + text + trailing separator
        assert!(text.starts_with("\u{2694} Sword"));
    }

    #[test]
    fn test_build_chars_multiple_entries_have_separators() {
        let ticker = make_ticker(vec![
            ("", "First", Color::White),
            ("", "Second", Color::White),
        ]);
        let chars = build_chars(&ticker);
        let text: String = chars.iter().map(|sc| sc.ch).collect();
        // Should contain both entries with separator between them
        assert!(text.contains("First"));
        assert!(text.contains("Second"));
        assert!(text.contains("\u{00B7}\u{00B7}\u{00B7}"));
    }

    #[test]
    fn test_extract_visible_at_zero() {
        let chars: Vec<StyledChar> = "Hello World"
            .chars()
            .map(|ch| StyledChar {
                ch,
                style: Style::default(),
            })
            .collect();
        let spans = extract_visible_spans(&chars, 0, 5);
        let text: String = spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(text, "Hello");
    }

    #[test]
    fn test_extract_visible_with_offset() {
        let chars: Vec<StyledChar> = "Hello World"
            .chars()
            .map(|ch| StyledChar {
                ch,
                style: Style::default(),
            })
            .collect();
        let spans = extract_visible_spans(&chars, 6, 5);
        let text: String = spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(text, "World");
    }

    #[test]
    fn test_extract_visible_wraps_around() {
        let chars: Vec<StyledChar> = "ABCDE"
            .chars()
            .map(|ch| StyledChar {
                ch,
                style: Style::default(),
            })
            .collect();
        // Offset 3, width 4 -> "DE" + "AB"
        let spans = extract_visible_spans(&chars, 3, 4);
        let text: String = spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(text, "DEAB");
    }

    #[test]
    fn test_extract_visible_with_multibyte_chars() {
        // This is the critical test - verify no panic with multi-byte chars
        let chars: Vec<StyledChar> = "\u{2694} Sword \u{00B7}\u{00B7}\u{00B7} \u{1F41F} Fish"
            .chars()
            .map(|ch| StyledChar {
                ch,
                style: Style::default(),
            })
            .collect();
        // Try various offsets to ensure no panics
        for offset in 0..chars.len() {
            let _ = extract_visible_spans(&chars, offset, 10);
        }
    }

    #[test]
    fn test_extract_visible_groups_same_style() {
        let mut chars = Vec::new();
        let style_a = Style::default().fg(Color::Red);
        let style_b = Style::default().fg(Color::Blue);
        for ch in "AAA".chars() {
            chars.push(StyledChar { ch, style: style_a });
        }
        for ch in "BBB".chars() {
            chars.push(StyledChar { ch, style: style_b });
        }
        let spans = extract_visible_spans(&chars, 0, 6);
        // Should produce exactly 2 spans (one red, one blue), not 6
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].content.as_ref(), "AAA");
        assert_eq!(spans[1].content.as_ref(), "BBB");
    }
}
