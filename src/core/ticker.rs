use std::collections::VecDeque;

use ratatui::style::Color;

/// A colored text segment for multi-color ticker entries.
#[derive(Debug, Clone)]
pub struct TickerSegment {
    pub text: String,
    pub color: Color,
}

/// A single entry in the scrolling loot ticker.
#[derive(Debug, Clone)]
pub struct TickerEntry {
    /// Icon prefix (e.g., "\u{2694}" for sword, "\u{1F41F}" for fish)
    pub icon: &'static str,
    /// Pre-formatted display text (e.g., "[E] Shadowfang +8STR")
    pub text: String,
    /// Display color (rarity or event-type color)
    pub color: Color,
    /// Whether to render bold
    pub bold: bool,
    /// Optional multi-color segments (overrides `text`/`color` when present)
    pub segments: Option<Vec<TickerSegment>>,
}

/// Internal entry with its scroll birth time for independent positioning.
#[derive(Debug, Clone)]
struct TimedEntry {
    entry: TickerEntry,
    /// The scroll_offset value when this entry was pushed.
    born_at: f64,
}

/// Scrolling loot ticker state. Transient (not serialized).
///
/// Each entry independently enters from the right edge of the viewport and
/// scrolls left. Speed adapts smoothly: when entries queue far ahead (debt),
/// scroll accelerates to keep the ticker responsive; when the queue drains,
/// it decelerates back to the base speed.
#[derive(Debug, Clone)]
pub struct Ticker {
    entries: VecDeque<TimedEntry>,
    /// Fractional scroll offset (integer part = character position)
    pub scroll_offset: f64,
    /// Current scroll speed (chars per tick), smoothly interpolated
    current_speed: f64,
    /// Last known viewport width for cleanup calculations
    pub viewport_width: usize,
}

/// Max entries in the ticker before oldest are evicted
const TICKER_MAX_ENTRIES: usize = 30;

/// Base scroll speed in chars per tick (0.4 = ~4 chars/sec at 100ms ticks)
pub const TICKER_SCROLL_SPEED: f64 = 0.4;

/// Maximum scroll speed multiplier when catching up to queued entries
const TICKER_MAX_SPEED_MULT: f64 = 20.0;

/// SLO target: newest entry should appear on screen within this many ticks.
/// 30 ticks = 3 seconds at 100ms tick interval.
const TICKER_SLO_TICKS: f64 = 30.0;

/// Gap (in chars) between consecutive entries on screen
const ENTRY_GAP: usize = 3;

impl Ticker {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::with_capacity(TICKER_MAX_ENTRIES),
            scroll_offset: 0.0,
            current_speed: TICKER_SCROLL_SPEED,
            viewport_width: 80,
        }
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Add a new entry to the ticker. Evicts oldest if at capacity.
    ///
    /// Each entry gets a `born_at` timestamp so it independently enters from
    /// the right edge. If entries arrive in rapid succession, they are spaced
    /// apart so they don't overlap on screen.
    pub fn push(&mut self, entry: TickerEntry) {
        // Space new entry after the previous one to avoid overlap
        let born_at = if let Some(prev) = self.entries.front() {
            let min_gap = Self::entry_char_len(&prev.entry) + ENTRY_GAP;
            self.scroll_offset.max(prev.born_at + min_gap as f64)
        } else {
            self.scroll_offset
        };

        if self.entries.len() >= TICKER_MAX_ENTRIES {
            self.entries.pop_back();
        }
        self.entries.push_front(TimedEntry { entry, born_at });
    }

    /// Advance the scroll offset by one tick. Call once per 100ms tick.
    ///
    /// Speed adapts smoothly based on "debt" — how far ahead the newest
    /// entry's born_at is from the current scroll position. When entries
    /// pile up faster than they scroll, speed ramps up gradually; when the
    /// queue drains, speed eases back to the base rate.
    pub fn tick(&mut self) {
        let target_speed = if let Some(newest) = self.entries.front() {
            let debt = (newest.born_at - self.scroll_offset).max(0.0);
            // SLO-aware: speed needed to display newest entry within SLO window
            let slo_speed = debt / TICKER_SLO_TICKS;
            let max_speed = TICKER_SCROLL_SPEED * TICKER_MAX_SPEED_MULT;
            slo_speed.max(TICKER_SCROLL_SPEED).min(max_speed)
        } else {
            TICKER_SCROLL_SPEED
        };

        // Instant speed adjustment — no lerp, meets SLO precisely
        self.current_speed = target_speed;
        self.scroll_offset += self.current_speed;
        self.cleanup_scrolled_entries();
    }

    /// Returns entries with their viewport column position (oldest first).
    /// Position is the left-edge column of the entry in the viewport.
    pub fn visible_entries(&self, viewport_width: usize) -> Vec<(&TickerEntry, isize)> {
        let vw = viewport_width as f64;
        self.entries
            .iter()
            .rev()
            .filter_map(|te| {
                let pos = (vw - (self.scroll_offset - te.born_at)) as isize;
                let entry_len = Self::entry_char_len(&te.entry) as isize;
                // Visible if any part is on screen
                if pos + entry_len > 0 && pos < viewport_width as isize {
                    Some((&te.entry, pos))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Remove entries that have fully scrolled off the left edge.
    fn cleanup_scrolled_entries(&mut self) {
        while let Some(oldest) = self.entries.back() {
            let entry_chars = Self::entry_char_len(&oldest.entry);
            let age = self.scroll_offset - oldest.born_at;
            // Off-screen when it has scrolled past the viewport + its own width
            if age > (self.viewport_width + entry_chars) as f64 {
                self.entries.pop_back();
            } else {
                break;
            }
        }
    }

    fn entry_char_len(entry: &TickerEntry) -> usize {
        let icon_len = if entry.icon.is_empty() {
            0
        } else {
            entry.icon.chars().count() + 1 // icon + space
        };
        let text_len = if let Some(segments) = &entry.segments {
            segments.iter().map(|s| s.text.chars().count()).sum()
        } else {
            entry.text.chars().count()
        };
        icon_len + text_len
    }
}

impl Default for Ticker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticker_new_is_empty() {
        let ticker = Ticker::new();
        assert!(ticker.is_empty());
        assert_eq!(ticker.scroll_offset, 0.0);
    }

    #[test]
    fn test_ticker_push_adds_entry() {
        let mut ticker = Ticker::new();
        ticker.push(TickerEntry {
            icon: "\u{2694}",
            text: "[R] Flamebrand".to_string(),
            color: Color::Yellow,
            bold: false,
            segments: None,
        });
        assert_eq!(ticker.len(), 1);
    }

    #[test]
    fn test_ticker_push_evicts_oldest() {
        let mut ticker = Ticker::new();
        for i in 0..TICKER_MAX_ENTRIES + 5 {
            ticker.push(TickerEntry {
                icon: "",
                text: format!("Item {i}"),
                color: Color::White,
                bold: false,
                segments: None,
            });
        }
        assert_eq!(ticker.len(), TICKER_MAX_ENTRIES);
    }

    #[test]
    fn test_ticker_tick_advances_offset() {
        let mut ticker = Ticker::new();
        assert_eq!(ticker.scroll_offset, 0.0);
        ticker.tick();
        assert!((ticker.scroll_offset - TICKER_SCROLL_SPEED).abs() < f64::EPSILON);
        ticker.tick();
        assert!((ticker.scroll_offset - 2.0 * TICKER_SCROLL_SPEED).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ticker_push_does_not_change_offset() {
        let mut ticker = Ticker::new();
        // Add an initial entry and advance scroll
        ticker.push(TickerEntry {
            icon: "",
            text: "First".to_string(),
            color: Color::White,
            bold: false,
            segments: None,
        });
        for _ in 0..10 {
            ticker.tick();
        }
        let offset_before = ticker.scroll_offset;
        assert!(offset_before > 0.0);

        // Push a new entry — offset should NOT change at all
        ticker.push(TickerEntry {
            icon: "\u{2694}",
            text: "Sword".to_string(),
            color: Color::Yellow,
            bold: false,
            segments: None,
        });
        assert_eq!(ticker.scroll_offset, offset_before);
    }

    #[test]
    fn test_ticker_cleanup_removes_scrolled_entries() {
        let mut ticker = Ticker::new();
        ticker.viewport_width = 10;
        ticker.push(TickerEntry {
            icon: "",
            text: "Old".to_string(), // 3 chars, born_at=0
            color: Color::White,
            bold: false,
            segments: None,
        });
        ticker.push(TickerEntry {
            icon: "",
            text: "New".to_string(), // born_at spaced after "Old"
            color: Color::White,
            bold: false,
            segments: None,
        });
        assert_eq!(ticker.len(), 2);

        // Old entry: born_at=0, off-screen when scroll > viewport(10) + len(3) = 13
        // At 0.4/tick, need 32.5 ticks → ~35 ticks
        for _ in 0..35 {
            ticker.tick();
        }

        // Oldest entry should have been cleaned up
        assert_eq!(ticker.len(), 1);
    }

    #[test]
    fn test_ticker_adaptive_speed_prevents_empty() {
        // Simulate rapid entry arrivals (every 15 ticks, like combat kills).
        // Without adaptive scroll speed, born_at debt grows unboundedly
        // and entries become permanently invisible after ~1 minute.
        let mut ticker = Ticker::new();
        ticker.viewport_width = 80;

        // Push 100 entries with 15 ticks between each (simulates ~2.5 min of play)
        for i in 0..100 {
            ticker.push(TickerEntry {
                icon: "\u{2728}",
                text: format!("+{} XP", 200 + i),
                color: Color::Green,
                bold: false,
                segments: None,
            });
            for _ in 0..15 {
                ticker.tick();
            }
        }

        // After 100 entries, visible_entries should still return something.
        // Without adaptive speed, this would return empty.
        let visible = ticker.visible_entries(80);
        assert!(
            !visible.is_empty(),
            "Ticker should still show entries after extended play"
        );
    }

    #[test]
    fn test_ticker_speed_ramps_up_with_debt() {
        let mut ticker = Ticker::new();
        ticker.viewport_width = 80;

        // Push several entries rapidly to build up debt
        for i in 0..10 {
            ticker.push(TickerEntry {
                icon: "",
                text: format!("Entry {i}"),
                color: Color::White,
                bold: false,
                segments: None,
            });
        }

        // Tick several times to let speed ramp up
        let initial_speed = ticker.current_speed;
        for _ in 0..20 {
            ticker.tick();
        }
        assert!(
            ticker.current_speed > initial_speed,
            "Speed should increase when entries are queued ahead"
        );
    }

    #[test]
    fn test_ticker_speed_decelerates_when_caught_up() {
        let mut ticker = Ticker::new();
        ticker.viewport_width = 80;

        // Push entries to build debt
        for i in 0..10 {
            ticker.push(TickerEntry {
                icon: "",
                text: format!("Entry {i}"),
                color: Color::White,
                bold: false,
                segments: None,
            });
        }

        // Let speed ramp up
        for _ in 0..50 {
            ticker.tick();
        }
        let peak_speed = ticker.current_speed;

        // Now tick without pushing — speed should decelerate as debt clears
        for _ in 0..200 {
            ticker.tick();
        }
        assert!(
            ticker.current_speed < peak_speed,
            "Speed should decrease when debt is paid off"
        );
    }

    #[test]
    fn test_ticker_scrolls_continuously() {
        let mut ticker = Ticker::new();
        ticker.push(TickerEntry {
            icon: "",
            text: "Test".to_string(),
            color: Color::White,
            bold: false,
            segments: None,
        });
        // Scrolling should advance every tick, no pauses
        let offset_before = ticker.scroll_offset;
        ticker.tick();
        assert!(ticker.scroll_offset > offset_before);
        let offset_after_one = ticker.scroll_offset;
        ticker.tick();
        assert!(ticker.scroll_offset > offset_after_one);
    }

    #[test]
    fn test_ticker_visible_entries_empty_when_no_entries() {
        let ticker = Ticker::new();
        let visible = ticker.visible_entries(80);
        assert!(visible.is_empty());
    }

    #[test]
    fn test_ticker_visible_entries_returns_on_screen_entry() {
        let mut ticker = Ticker::new();
        ticker.viewport_width = 80;
        ticker.push(TickerEntry {
            icon: "\u{2694}",
            text: "Flamebrand".to_string(),
            color: Color::Yellow,
            bold: true,
            segments: None,
        });
        // Entry born at scroll_offset=0, so pos = 80 - 0 = 80.
        // It enters screen only after scrolling at least 1 unit.
        ticker.tick();
        let visible = ticker.visible_entries(80);
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].0.text, "Flamebrand");
        assert!(visible[0].0.bold);
    }

    #[test]
    fn test_ticker_segmented_entry_length() {
        let mut ticker = Ticker::new();
        ticker.viewport_width = 80;
        ticker.push(TickerEntry {
            icon: "",
            text: String::new(),
            color: Color::White,
            bold: false,
            segments: Some(vec![
                TickerSegment {
                    text: "Hello".to_string(),
                    color: Color::Red,
                },
                TickerSegment {
                    text: " World".to_string(),
                    color: Color::Blue,
                },
            ]),
        });
        assert_eq!(ticker.len(), 1);
        // Scroll one tick so entry enters the viewport
        ticker.tick();
        let visible = ticker.visible_entries(80);
        assert_eq!(visible.len(), 1);
    }

    #[test]
    fn test_ticker_default_matches_new() {
        let ticker_new = Ticker::new();
        let ticker_default = Ticker::default();
        assert_eq!(ticker_new.scroll_offset, ticker_default.scroll_offset);
        assert_eq!(ticker_new.viewport_width, ticker_default.viewport_width);
        assert!(ticker_new.is_empty());
        assert!(ticker_default.is_empty());
    }
}
