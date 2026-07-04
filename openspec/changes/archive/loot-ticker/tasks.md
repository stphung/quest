> Backported implementation plan (completed — this work shipped).

## 2026-02-15-loot-ticker-plan.md

# Loot Ticker Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the vertical Loot panel with a 1-row horizontal scrolling ticker (stock-ticker style) that continuously displays all recent game events, saving ~5 rows of vertical space.

**Architecture:** Add a `LootTicker` struct to `GameState` (transient, not serialized). A new `src/ui/ticker.rs` renders the ticker by computing a visible window into a virtual string built from concatenated entries. Scroll offset advances fractionally each tick in `main.rs`. The existing `apply_tick_events()` in `tick_events.rs` is extended to push entries to the ticker.

**Tech Stack:** Rust, Ratatui (Span/Line/Paragraph), existing 100ms tick loop

**Design doc:** `docs/plans/2026-02-15-loot-ticker-design.md`

---

### Task 1: Add LootTicker State to GameState

**Files:**
- Modify: `src/core/game_state.rs`

**Step 1: Add TickerEntry and LootTicker structs**

Add after the `RecentDrop` struct (line 26) and before `const MAX_RECENT_DROPS`:

```rust
/// A single entry in the scrolling loot ticker.
#[derive(Debug, Clone)]
pub struct TickerEntry {
    /// Icon prefix (e.g., "\u{2694}" for sword, "\u{1F41F}" for fish)
    pub icon: &'static str,
    /// Pre-formatted display text (e.g., "[E] Shadowfang +8STR")
    pub text: String,
    /// Display color (rarity or event-type color)
    pub color: ratatui::style::Color,
    /// Whether to render bold
    pub bold: bool,
}

/// Scrolling loot ticker state. Transient (not serialized).
#[derive(Debug, Clone)]
pub struct LootTicker {
    /// Recent events displayed in the ticker
    pub entries: VecDeque<TickerEntry>,
    /// Fractional scroll offset (integer part = character position)
    pub scroll_offset: f64,
    /// Remaining pause ticks when a new event arrives (0 = scrolling)
    pub pause_ticks: u8,
}

/// Max entries in the ticker before oldest are evicted
const TICKER_MAX_ENTRIES: usize = 30;

/// Characters scrolled per tick (0.4 = ~4 chars/sec at 100ms ticks)
pub const TICKER_SCROLL_SPEED: f64 = 0.4;

/// Ticks to pause scrolling when a new event arrives (5 = 500ms)
pub const TICKER_PAUSE_TICKS: u8 = 5;

impl LootTicker {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::with_capacity(TICKER_MAX_ENTRIES),
            scroll_offset: 0.0,
            pause_ticks: 0,
        }
    }

    /// Add a new entry to the ticker. Evicts oldest if at capacity.
    pub fn push(&mut self, entry: TickerEntry) {
        if self.entries.len() >= TICKER_MAX_ENTRIES {
            self.entries.pop_back();
        }
        self.entries.push_front(entry);
        self.pause_ticks = TICKER_PAUSE_TICKS;
    }

    /// Advance the scroll offset by one tick. Call once per 100ms tick.
    pub fn tick(&mut self) {
        if self.pause_ticks > 0 {
            self.pause_ticks -= 1;
        } else {
            self.scroll_offset += TICKER_SCROLL_SPEED;
        }
    }
}

impl Default for LootTicker {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 2: Add `loot_ticker` field to `GameState`**

In the `GameState` struct, add after the `recent_drops` field (after line 97):

```rust
    /// Scrolling loot ticker state (transient, not saved)
    #[serde(skip)]
    pub loot_ticker: LootTicker,
```

**Step 3: Initialize in `GameState::new()`**

In the `Self { ... }` block in `GameState::new()`, add after `recent_drops`:

```rust
            loot_ticker: LootTicker::new(),
```

**Step 4: Write tests for LootTicker**

Add tests to the existing `#[cfg(test)] mod tests` in `game_state.rs`:

```rust
    #[test]
    fn test_loot_ticker_new_is_empty() {
        let ticker = LootTicker::new();
        assert!(ticker.entries.is_empty());
        assert_eq!(ticker.scroll_offset, 0.0);
        assert_eq!(ticker.pause_ticks, 0);
    }

    #[test]
    fn test_loot_ticker_push_adds_entry() {
        let mut ticker = LootTicker::new();
        ticker.push(TickerEntry {
            icon: "\u{2694}",
            text: "[R] Flamebrand".to_string(),
            color: ratatui::style::Color::Yellow,
            bold: false,
        });
        assert_eq!(ticker.entries.len(), 1);
        assert_eq!(ticker.entries[0].text, "[R] Flamebrand");
        assert_eq!(ticker.pause_ticks, TICKER_PAUSE_TICKS);
    }

    #[test]
    fn test_loot_ticker_push_evicts_oldest() {
        let mut ticker = LootTicker::new();
        for i in 0..TICKER_MAX_ENTRIES + 5 {
            ticker.push(TickerEntry {
                icon: "",
                text: format!("Item {i}"),
                color: ratatui::style::Color::White,
                bold: false,
            });
        }
        assert_eq!(ticker.entries.len(), TICKER_MAX_ENTRIES);
        // Most recent should be at front
        assert_eq!(ticker.entries[0].text, "Item 34");
    }

    #[test]
    fn test_loot_ticker_tick_advances_offset() {
        let mut ticker = LootTicker::new();
        assert_eq!(ticker.scroll_offset, 0.0);
        ticker.tick();
        assert!((ticker.scroll_offset - TICKER_SCROLL_SPEED).abs() < f64::EPSILON);
        ticker.tick();
        assert!((ticker.scroll_offset - 2.0 * TICKER_SCROLL_SPEED).abs() < f64::EPSILON);
    }

    #[test]
    fn test_loot_ticker_pause_on_new_entry() {
        let mut ticker = LootTicker::new();
        ticker.push(TickerEntry {
            icon: "",
            text: "Test".to_string(),
            color: ratatui::style::Color::White,
            bold: false,
        });
        // Should be paused
        assert_eq!(ticker.pause_ticks, TICKER_PAUSE_TICKS);
        let offset_before = ticker.scroll_offset;
        ticker.tick();
        // Offset should NOT advance while paused
        assert_eq!(ticker.scroll_offset, offset_before);
        assert_eq!(ticker.pause_ticks, TICKER_PAUSE_TICKS - 1);
    }

    #[test]
    fn test_loot_ticker_resumes_after_pause() {
        let mut ticker = LootTicker::new();
        ticker.push(TickerEntry {
            icon: "",
            text: "Test".to_string(),
            color: ratatui::style::Color::White,
            bold: false,
        });
        // Exhaust pause ticks
        for _ in 0..TICKER_PAUSE_TICKS {
            ticker.tick();
        }
        assert_eq!(ticker.pause_ticks, 0);
        let offset_before = ticker.scroll_offset;
        ticker.tick();
        assert!(ticker.scroll_offset > offset_before);
    }
```

**Step 5: Run tests**

Run: `cargo test -p quest --lib core::game_state`

Expected: All existing + new tests pass.

**Step 6: Commit**

```bash
git add src/core/game_state.rs
git commit -m "feat: add LootTicker state struct to GameState"
```

---

### Task 2: Create Ticker Renderer

**Files:**
- Create: `src/ui/ticker.rs`
- Modify: `src/ui/mod.rs` (add `pub mod ticker;`)

**Step 1: Create `src/ui/ticker.rs`**

```rust
//! Scrolling loot ticker renderer.
//!
//! Renders a 1-row horizontal ticker by computing a visible window
//! into a virtual string built from concatenated TickerEntry spans.

use crate::core::game_state::LootTicker;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
    layout::Rect,
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
            segments.push((
                format!("{} ", entry.icon),
                Style::default().fg(entry.color),
            ));
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
            let slice_start = if effective_start < window_start {
                window_start - effective_start
            } else {
                0
            };
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
        let segments = vec![
            ("ABCDE".to_string(), Style::default()),
        ];
        // Offset 3, width 4, total 5 -> should get "DE" + "AB"
        let spans = extract_visible_spans(&segments, 3, 4, 5);
        let text: String = spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(text, "DEAB");
    }
}
```

**Step 2: Register the module in `src/ui/mod.rs`**

Add after `mod throbber;` (line 31):

```rust
pub mod ticker;
```

**Step 3: Run tests**

Run: `cargo test -p quest --lib ui::ticker`

Expected: All tests pass.

**Step 4: Commit**

```bash
git add src/ui/ticker.rs src/ui/mod.rs
git commit -m "feat: add ticker renderer with smooth scrolling and wrap-around"
```

---

### Task 3: Wire Ticker into XL/L Layout

**Files:**
- Modify: `src/ui/mod.rs` (layout constraints in `draw_xl_l_layout`)
- Modify: `src/ui/info_panel.rs` (XL/L renders combat log full-width)

**Step 1: Update XL/L layout constraints**

In `src/ui/mod.rs` `draw_xl_l_layout()`, replace the vertical layout constraints (lines 144-163) to insert a ticker row between stats and info area.

The current constraint pattern for `v_chunks` becomes:
- `Constraint::Length(stats_height)` - stats + right panel
- `Constraint::Length(1)` - **NEW: ticker row**
- `Constraint::Min(6)` - combat log (full width, was "Loot + Combat")
- Optional: `Constraint::Length(12)` - update drawer
- `Constraint::Length(4)` - footer

Update the area assignments after the layout split to account for the new chunk index.

**Step 2: Call `draw_ticker` and full-width combat log**

After the area assignments, add:

```rust
// Draw the scrolling loot ticker
ticker::draw_ticker(frame, ticker_area, &game_state.loot_ticker);
```

Replace the `info_panel::draw_info_panel(frame, info_area, game_state, ctx)` call with a full-width combat log call:

```rust
info_panel::draw_combat_log_full_width(frame, info_area, game_state);
```

**Step 3: Add `draw_combat_log_full_width` to info_panel.rs**

Add a new public function to `src/ui/info_panel.rs`:

```rust
/// Draws the combat log at full width (used when ticker replaces the loot panel).
pub fn draw_combat_log_full_width(frame: &mut Frame, area: Rect, game_state: &GameState) {
    draw_combat_log(frame, area, game_state);
}
```

And make `draw_combat_log` still callable from the full-width function (it already takes any Rect).

**Step 4: Update `draw_info_panel` for XL/L tiers**

In `draw_info_panel()`, change the XL/L branch to render only the full-width combat log (since the ticker is now rendered separately in `mod.rs`):

```rust
SizeTier::XL | SizeTier::L => {
    draw_combat_log(frame, area, game_state);
}
```

**Step 5: Run and verify**

Run: `cargo build`

Expected: Compiles without errors.

Run: `cargo test`

Expected: All tests pass.

**Step 6: Commit**

```bash
git add src/ui/mod.rs src/ui/info_panel.rs
git commit -m "feat: integrate ticker into XL/L layout, combat log goes full width"
```

---

### Task 4: Wire Ticker into M Layout

**Files:**
- Modify: `src/ui/mod.rs` (`draw_m_layout`)
- Modify: `src/ui/info_panel.rs` (M tier uses ticker instead of loot half)

**Step 1: Update M layout constraints**

In `draw_m_layout()`, replace the info panel constraint `Constraint::Length(4)` with:
- `Constraint::Length(1)` for ticker
- `Constraint::Length(3)` for compact combat log

Or simpler: Replace the info panel section with a 1-row ticker + compact combat log.

**Step 2: Render ticker in M layout**

Call `ticker::draw_ticker(frame, ticker_area, &game_state.loot_ticker)` for the ticker row.

For the remaining combat area, call `info_panel::draw_info_panel(frame, combat_area, game_state, ctx)` — the M-tier branch in `draw_info_panel` will need updating to render only the combat log compact.

**Step 3: Update M-tier branch in `draw_info_panel`**

Change the M-tier branch to render only compact combat log (no loot half):

```rust
SizeTier::M => {
    draw_combat_log_compact(frame, area, game_state);
}
```

Add `draw_combat_log_compact`:

```rust
fn draw_combat_log_compact(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let mut lines: Vec<Line> = Vec::new();
    let max_entries = area.height as usize;
    let max_width = area.width as usize;

    for entry in game_state
        .combat_state
        .combat_log
        .iter()
        .rev()
        .take(max_entries)
    {
        let color = if entry.is_player_action {
            if entry.is_crit { Color::Yellow } else { Color::Green }
        } else {
            Color::Red
        };
        let msg = truncate_to_width(&entry.message, max_width);
        lines.push(Line::from(Span::styled(msg, Style::default().fg(color))));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}
```

**Step 4: Run and verify**

Run: `cargo build && cargo test`

Expected: Compiles and all tests pass.

**Step 5: Commit**

```bash
git add src/ui/mod.rs src/ui/info_panel.rs
git commit -m "feat: integrate ticker into M-tier layout"
```

---

### Task 5: Push TickEvents to Ticker

**Files:**
- Modify: `src/tick_events.rs`

This is the critical integration step. The existing `apply_tick_events()` function processes all `TickEvent` variants and updates the combat log. We extend it to also push entries to `game_state.loot_ticker`.

**Step 1: Add ticker pushes for each relevant event type**

In `apply_tick_events()`, add `loot_ticker.push()` calls for these events:

```rust
TickEvent::ItemDropped { item_name, rarity, equipped, slot, stats, .. } => {
    let rarity_initial = match rarity {
        Rarity::Common => "C",
        Rarity::Magic => "M",
        Rarity::Rare => "R",
        Rarity::Epic => "E",
        Rarity::Legendary => "L",
    };
    let equip_tag = if *equipped { " \u{1F528}" } else { "" };
    let stat_suffix = if stats.is_empty() {
        String::new()
    } else {
        format!(" {}", stats)
    };
    let text = format!("[{}] {}{}{}", rarity_initial, item_name, stat_suffix, equip_tag);
    let color = rarity_color(*rarity);
    game_state.loot_ticker.push(TickerEntry {
        icon: "\u{2694}",
        text,
        color,
        bold: matches!(rarity, Rarity::Epic | Rarity::Legendary),
    });
}

TickEvent::FishCaught { fish_name, rarity, .. } => {
    let rarity_initial = match rarity {
        Rarity::Common => "C",
        Rarity::Magic => "M",
        Rarity::Rare => "R",
        Rarity::Epic => "E",
        Rarity::Legendary => "L",
    };
    let text = format!("{} [{}]", fish_name, rarity_initial);
    let color = rarity_color(*rarity);
    game_state.loot_ticker.push(TickerEntry {
        icon: "\u{1F41F}",
        text,
        color,
        bold: false,
    });
}

TickEvent::LeveledUp { new_level } => {
    game_state.loot_ticker.push(TickerEntry {
        icon: "\u{2B06}",
        text: format!("Level {}!", new_level),
        color: Color::Green,
        bold: true,
    });
}

TickEvent::SubzoneBossDefeated { result, message, .. } => {
    // Only push zone advancement events (not just boss kills)
    if matches!(result, BossDefeatResult::SubzoneAdvance { .. } | BossDefeatResult::ZoneAdvance { .. }) {
        game_state.loot_ticker.push(TickerEntry {
            icon: "\u{1F5FA}",
            text: message.clone(),
            color: Color::Cyan,
            bold: false,
        });
    }
    // existing combat log push
}

TickEvent::DungeonCompleted { message, .. } => {
    game_state.loot_ticker.push(TickerEntry {
        icon: "\u{1F3F0}",
        text: "Dungeon Complete!".to_string(),
        color: Color::Magenta,
        bold: true,
    });
    // existing combat log push
}

TickEvent::AchievementUnlocked { name, .. } => {
    game_state.loot_ticker.push(TickerEntry {
        icon: "\u{1F3C6}",
        text: name.clone(),
        color: Color::Yellow,
        bold: true,
    });
    // existing combat log push
}
```

**Step 2: Add the rarity_color helper and necessary imports**

Add at the top of `tick_events.rs`:

```rust
use crate::core::game_state::TickerEntry;
use crate::items::types::Rarity;
use ratatui::style::Color;
```

Add a helper function:

```rust
fn rarity_color(rarity: Rarity) -> Color {
    match rarity {
        Rarity::Common => Color::Gray,
        Rarity::Magic => Color::Blue,
        Rarity::Rare => Color::Yellow,
        Rarity::Epic => Color::Magenta,
        Rarity::Legendary => Color::Rgb(255, 165, 0),
    }
}
```

Note: `tick_events.rs` is a binary-only module (not in `lib.rs`), so it can import `ratatui::style::Color` without affecting the library crate.

**Step 3: Run and verify**

Run: `cargo build && cargo test`

Expected: Compiles and all tests pass.

**Step 4: Commit**

```bash
git add src/tick_events.rs
git commit -m "feat: push TickEvents to loot ticker for all event types"
```

---

### Task 6: Advance Ticker Scroll in Game Loop

**Files:**
- Modify: `src/main.rs`

**Step 1: Add ticker tick advancement**

In `src/main.rs`, after the `apply_tick_events()` call (line 1082) and before the visual effects update, add:

```rust
// Advance loot ticker scroll
state.loot_ticker.tick();
```

This is all that's needed - the `LootTicker::tick()` method handles pause countdown and scroll offset advancement.

**Step 2: Run full test suite**

Run: `cargo build && cargo test`

Expected: Compiles and all tests pass.

**Step 3: Manual verification**

Run: `cargo run`

Expected: A scrolling ticker bar appears between the stats/combat panels and the combat log. Items, fish, level-ups, and other events appear in the ticker as they occur. The ticker pauses briefly when new events arrive, then resumes scrolling.

**Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: advance loot ticker scroll in game loop"
```

---

### Task 7: Clean Up and Polish

**Files:**
- Modify: `src/ui/info_panel.rs` (remove dead code if any)
- Modify: `src/ui/mod.rs` (remove unused imports if any)

**Step 1: Remove `draw_loot_combat_compact` if fully replaced**

If the M-tier loot half is now fully replaced by the ticker, remove `draw_loot_combat_compact()` from `info_panel.rs`. Keep `draw_recent_gains()` for now since `recent_drops` is still populated (backward compat).

**Step 2: Run clippy and tests**

Run: `cargo clippy --all-targets -- -D warnings && cargo test`

Expected: No warnings, all tests pass.

**Step 3: Run `make check` (full CI)**

Run: `make check`

Expected: All CI checks pass.

**Step 4: Commit**

```bash
git add -A
git commit -m "refactor: clean up dead code after loot ticker integration"
```
