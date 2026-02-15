# Loot Ticker Design

## Overview

Replace the current vertical Loot panel with a 1-row horizontal scrolling ticker (stock-ticker style) that continuously displays recent game events. This saves ~5 rows of vertical space and gives the combat log full width.

## Layout Change

### XL/L Tier - Before

```
+---------------------------+--------------------------+
|   Stats Panel (50%)       |   Combat Scene (50%)     | <- 27 rows
+---------------+--------------------------------------+
|  Loot (50%)   |   Combat Log (50%)                   | <- Min 6 rows
+---------------+--------------------------------------+
|  Footer (4 rows)                                     |
+------------------------------------------------------+
```

### XL/L Tier - After

```
+---------------------------+--------------------------+
|   Stats Panel (50%)       |   Combat Scene (50%)     | <- 27 rows
+------------------------------------------------------+
| [E] Shadowfang +8STR ... Fish [R] ... Lv 42! ...    | <- 1 row ticker
+------------------------------------------------------+
|  Combat Log (full width, grows)                      | <- Min 6 rows
+------------------------------------------------------+
|  Footer (4 rows)                                     |
+------------------------------------------------------+
```

### M Tier - After

```
+------------------------------------------------------+
| Lv42 P10 HP:450/450  STR18 DEX14 CON16               | <- compact stats
| XP: 12,340/18,520                                     | <- XP bar
|                                                       |
|              [Combat / Activity Area]                 | <- grows
|                                                       |
+------------------------------------------------------+
| [E] Shadowfang ... Fish ... Lv 42! ...               | <- 1 row ticker
+------------------------------------------------------+
|  [P] Prestige  [H] Haven  [Q] Quit                   |
+------------------------------------------------------+
```

### S Tier

No ticker (too narrow). Keep existing merged feed.

## Ticker Content

All recent game events scroll in a single horizontal bar. Each event is separated by ` ... ` (DarkGray).

### Event Formats

| Event          | Icon | Format                        | Color        |
|----------------|------|-------------------------------|--------------|
| Item drop      | sword| `[R] Name +stats` (hammer=equipped) | Rarity color |
| Fish catch     | fish | `Name [R]`                    | Rarity color |
| Level up       | up   | `Level N!`                    | Green        |
| Zone change    | map  | `Zone Name`                   | Cyan         |
| Dungeon event  | castle| `Dungeon Complete!`          | Magenta      |
| Achievement    | trophy| `Achievement Name`           | Yellow       |

Rarity initials: [C]ommon, [M]agic, [R]are, [E]pic, [L]egendary.

Rarity colors: Common=Gray, Magic=Blue, Rare=Yellow, Epic=Magenta, Legendary=Orange(RGB 255,165,0).

## Smooth Scrolling

### Mechanism

Time-based sub-character scrolling using a `f64` scroll offset:

- Scroll speed: ~4 characters/second (0.4 chars per 100ms tick)
- The integer part of `scroll_offset` determines the visible character window into the virtual ticker string
- Each tick: `scroll_offset += 0.4` (unless paused)

### Pause on New Event

When a new event arrives, the ticker pauses for 500ms (5 ticks) so the player notices new loot. Then scrolling resumes.

### Wrap-Around

The ticker content is circular. When the last event scrolls past the left edge, it wraps to appear again from the right. The virtual string is conceptually doubled to create seamless looping.

## State

### New struct: `LootTicker` (in `game_state.rs`)

```rust
pub struct LootTicker {
    pub entries: VecDeque<TickerEntry>,  // Max ~30 entries
    pub scroll_offset: f64,              // Fractional character position
    pub pause_ticks: u8,                 // Remaining pause ticks on new item
}

pub struct TickerEntry {
    pub icon: &'static str,
    pub text: String,          // Pre-formatted display text
    pub color: Color,          // Rarity/event color
    pub bold: bool,
}
```

`LootTicker` is transient (not serialized). It lives on `GameState` alongside `recent_drops`.

### Constants

- `TICKER_SCROLL_SPEED: f64 = 0.4` (chars per tick)
- `TICKER_PAUSE_TICKS: u8 = 5` (500ms pause on new event)
- `TICKER_MAX_ENTRIES: usize = 30`
- `TICKER_SEPARATOR: &str = " ... "` (with DarkGray color)

## Rendering

### New file: `src/ui/ticker.rs`

Single public function:

```rust
pub fn draw_ticker(frame: &mut Frame, area: Rect, ticker: &LootTicker)
```

Algorithm:
1. Build a virtual string by concatenating all entries with separators
2. Track color spans alongside character positions
3. Use `scroll_offset` (integer part) to compute visible window = `[offset .. offset + area.width]`
4. For wrap-around: if the visible window extends past the end, wrap to the beginning
5. Render the visible slice as a `Line` of colored `Span`s

### Tick Advancement

In `main.rs` game loop, after `game_tick()`:

```rust
if ticker.pause_ticks > 0 {
    ticker.pause_ticks -= 1;
} else {
    ticker.scroll_offset += TICKER_SCROLL_SPEED;
}
// Wrap offset when it exceeds total content length
let total_len = ticker.total_rendered_width();
if total_len > 0 {
    ticker.scroll_offset %= total_len as f64;
}
```

## Integration Points

### TickEvent Mapping (in `main.rs`)

The existing TickEvent handler in `main.rs` already processes all event types. For each event that currently calls `add_recent_drop()` or `add_log_entry()`, also push a `TickerEntry`:

- `TickEvent::ItemDropped` -> ticker entry with rarity color, slot, stats
- `TickEvent::FishCaught` -> ticker entry with fish name and rarity
- `TickEvent::LeveledUp` -> ticker entry "Level N!" in green
- `TickEvent::SubzoneBossDefeated` -> ticker entry with zone progress
- `TickEvent::DungeonCompleted` -> ticker entry in magenta
- `TickEvent::AchievementUnlocked` -> ticker entry in yellow

### Layout Changes

In `draw_xl_l_layout()` (`src/ui/mod.rs`):
- Replace the current `Constraint::Min(6)` info area with:
  - `Constraint::Length(1)` for ticker
  - `Constraint::Min(6)` for full-width combat log
- Remove the 50/50 horizontal split in `draw_info_panel()` for XL/L tiers
- Combat log renders full-width via `draw_combat_log()`

In `draw_m_layout()`:
- Replace `Constraint::Length(4)` info panel with:
  - `Constraint::Length(1)` for ticker
  - `Constraint::Length(3)` or remove compact loot section

### What Gets Modified

- `src/ui/mod.rs` - Layout constraints for XL/L and M tiers
- `src/ui/info_panel.rs` - XL/L renders combat log full-width; M tier uses ticker instead of loot half
- `src/core/game_state.rs` - Add `LootTicker` struct and field on `GameState`
- `src/main.rs` - Tick advancement, TickEvent-to-ticker mapping

### What Gets Added

- `src/ui/ticker.rs` - Ticker rendering

### What Gets Removed

- Nothing removed outright. `recent_drops` and `draw_recent_gains()` kept for backward compatibility initially. The ticker becomes the primary loot display.

## Responsive Behavior

| Tier | Ticker Behavior |
|------|----------------|
| XL/L | Full ticker between panels and full-width combat log |
| M    | Full ticker above footer, combat log above it |
| S    | No ticker - keep merged feed (too narrow for scrolling) |
