> Backported design record. Sources: docs/plans/2026-02-15-ticker-m-s-only-plan.md.

## 2026-02-15-ticker-m-s-only-plan.md

# Scope Loot Ticker to M/S Layouts Only

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Use the scrolling loot ticker only in M and S tier layouts; restore XL/L to the original side-by-side Loot + Combat panels.

**Architecture:** The PR added `LootTicker` state, `ticker.rs` renderer, and `tick_events.rs` event mapping — all of that stays. The change is purely in UI layout dispatch: XL/L reverts to the main branch's side-by-side info panel, while M and S both use ticker + combat-log-only.

**Tech Stack:** Rust, Ratatui

---

### Task 1: Restore XL/L info panel to side-by-side Loot + Combat

**Files:**
- Modify: `src/ui/info_panel.rs` (restore `draw_recent_gains`, `draw_loot_combat_compact` from main branch, update XL/L dispatch)

**Step 1: Restore `draw_recent_gains()` function**

Add back the loot panel renderer that was removed by the PR. Insert after `draw_combat_log()` (after line ~96):

```rust
/// Draws the loot panel (items, fish, etc.) with two-line format for equipment.
fn draw_recent_gains(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Loot ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();
    let max_lines = inner.height as usize;

    if game_state.recent_drops.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No gains yet",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for drop in game_state.recent_drops.iter() {
            if lines.len() >= max_lines {
                break;
            }

            let color = rarity_color(drop.rarity);
            let rarity_tag = format!("[{}]", drop.rarity.name());
            let equipped_tag = if drop.equipped { " \u{1F528}" } else { "" };

            // Line 1: icon [Rarity] Name 🔨  Slot
            let mut spans = vec![
                Span::styled(format!("{} ", drop.icon), Style::default().fg(color)),
                Span::styled(
                    format!("{} ", rarity_tag),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(&drop.name, Style::default().fg(color)),
                Span::styled(equipped_tag, Style::default().fg(Color::Green)),
            ];

            if !drop.slot.is_empty() {
                spans.push(Span::styled(
                    format!("  {}", drop.slot),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            lines.push(Line::from(spans));

            // Line 2: stat summary (only for equipment with stats)
            if !drop.stats.is_empty() && lines.len() < max_lines {
                lines.push(Line::from(Span::styled(
                    format!("  {}", drop.stats),
                    Style::default().fg(Color::DarkGray),
                )));
            }
        }
    }

    // Pad remaining space
    while lines.len() < max_lines {
        lines.push(Line::from(""));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}
```

**Step 2: Update XL/L dispatch in `draw_info_panel()`**

Change the `XL | L` arm from combat-log-only back to side-by-side:

```rust
SizeTier::XL | SizeTier::L => {
    // Full side-by-side with borders
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    draw_recent_gains(frame, chunks[0], game_state);
    draw_combat_log(frame, chunks[1], game_state);
}
```

This requires adding `Constraint, Direction, Layout` back to the imports. Current imports have only `Rect`. Update the ratatui `layout` import line to:

```rust
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    ...
};
```

**Step 3: Update S dispatch in `draw_info_panel()`**

Change the S arm from merged feed to compact combat log (ticker handles loot):

```rust
SizeTier::S => {
    // Compact combat log (ticker handles loot display)
    draw_combat_log_compact(frame, area, game_state);
}
```

**Step 4: Remove dead code**

Delete `draw_merged_feed()` — it's no longer called from any layout. Also remove the `pub(super)` visibility since nothing references it externally anymore.

**Step 5: Run tests**

Run: `cargo test`
Expected: All 1,210 tests pass. No tests reference `draw_merged_feed` directly (it's a private UI function).

**Step 6: Run clippy**

Run: `cargo clippy --all-targets -- -D warnings`
Expected: Clean (no dead code warnings since `draw_merged_feed` is removed).

---

### Task 2: Restore XL/L layout in mod.rs (remove ticker row)

**Files:**
- Modify: `src/ui/mod.rs` (revert `draw_xl_l_layout` to pre-PR layout, no ticker row)

**Step 1: Remove ticker from XL/L vertical layout**

In `draw_xl_l_layout()`, change the vertical constraints from:
```
stats_height | ticker (1) | combat log (Min 6) | [update drawer] | footer (4)
```
back to:
```
stats_height | info panels (Min 6) | [update drawer] | footer (4)
```

For the non-update-drawer branch, change:
```rust
Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(stats_height), // Main content (stats + right panel)
        Constraint::Min(6),               // Full-width Loot + Combat (grows)
        Constraint::Length(4),            // Full-width footer (2 rows)
    ])
    .split(main_area)
```

For the update-drawer branch, change:
```rust
Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(stats_height), // Main content (stats + right panel)
        Constraint::Min(6),               // Full-width Loot + Combat (grows)
        Constraint::Length(20),           // Update drawer panel
        Constraint::Length(4),            // Full-width footer (2 rows)
    ])
    .split(main_area)
```

**Step 2: Update chunk index assignments**

Remove `ticker_area` variable. Update the index assignments:
```rust
let content_area = v_chunks[0];
let info_area = v_chunks[1];
let (update_drawer_area, footer_area) = if show_update_drawer {
    (Some(v_chunks[2]), v_chunks[3])
} else {
    (None, v_chunks[2])
};
```

**Step 3: Remove ticker draw call in XL/L**

Delete this line:
```rust
ticker::draw_ticker(frame, ticker_area, &game_state.loot_ticker);
```

**Step 4: Update the combat log comment**

Change comment from "no loot panel — ticker handles loot display" to "Full-width Loot + Combat panels":
```rust
// Draw full-width Loot + Combat panels
info_panel::draw_info_panel(frame, info_area, game_state, ctx);
```

**Step 5: Run tests and clippy**

Run: `cargo test && cargo clippy --all-targets -- -D warnings`
Expected: All pass, no warnings.

---

### Task 3: Add ticker row to S layout

**Files:**
- Modify: `src/ui/mod.rs` (`draw_s_layout`)

**Step 1: Add ticker to standard S layout constraints**

Change the standard S layout (non-special-activity) from:
```
status(1) | XP(1) | player HP(1) | enemy HP(1) | combat status(1) | merged feed(Min 4) | footer(1)
```
to:
```
status(1) | XP(1) | player HP(1) | enemy HP(1) | combat status(1) | ticker(1) | combat log(Min 3) | footer(1)
```

```rust
let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(1), // Status line
        Constraint::Length(1), // XP bar
        Constraint::Length(1), // Player HP
        Constraint::Length(1), // Enemy HP + name
        Constraint::Length(1), // Combat status
        Constraint::Length(1), // Loot ticker
        Constraint::Min(3),    // Combat log
        Constraint::Length(1), // Footer
    ])
    .split(area);
```

**Step 2: Add ticker draw call and update info panel call**

After the combat status line (`combat_scene::draw_combat_scene(frame, chunks[4], ...)`), add:

```rust
// Loot ticker
ticker::draw_ticker(frame, chunks[5], &game_state.loot_ticker);

// Compact combat log (ticker handles loot display)
info_panel::draw_info_panel(frame, chunks[6], game_state, ctx);
```

Update the footer index:
```rust
// Minimal footer
stats_panel::draw_footer_minimal(frame, chunks[7], game_state);
```

**Step 3: Run full CI check**

Run: `cargo fmt && cargo test && cargo clippy --all-targets -- -D warnings`
Expected: All pass.

**Step 4: Commit all changes**

```bash
git add src/ui/mod.rs src/ui/info_panel.rs
git commit -m "feat: scope loot ticker to M/S layouts, restore XL/L side-by-side panels"
```

---

### Task 4: Fix jarring scroll shift on new entries

**Files:**
- Modify: `src/core/game_state.rs` (`LootTicker::push`)

The known issue from the PR: when new entries arrive, the text shifts jarringly because entries are prepended to the front of the deque, changing the virtual string under the scroll offset. Fix by adjusting `scroll_offset` to compensate for the newly inserted content length.

**Step 1: Write the failing test**

In `src/core/game_state.rs` tests section:

```rust
#[test]
fn test_loot_ticker_push_compensates_scroll_offset() {
    let mut ticker = LootTicker::new();
    // Add an initial entry and advance scroll
    ticker.push(TickerEntry {
        icon: "",
        text: "First".to_string(),
        color: ratatui::style::Color::White,
        bold: false,
    });
    // Advance scroll past the pause
    for _ in 0..10 {
        ticker.tick();
    }
    let offset_before = ticker.scroll_offset;
    assert!(offset_before > 0.0);

    // Push a new entry — offset should increase to compensate
    ticker.push(TickerEntry {
        icon: "\u{2694}",
        text: "Sword".to_string(),
        color: ratatui::style::Color::Yellow,
        bold: false,
    });
    // Offset should be greater than before (compensated for new content)
    assert!(ticker.scroll_offset > offset_before);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_loot_ticker_push_compensates_scroll_offset`
Expected: FAIL — currently push resets pause but doesn't adjust offset.

**Step 3: Implement offset compensation**

In `LootTicker::push()`, before inserting the new entry, calculate the char length that will be inserted at the front and add it to `scroll_offset`:

```rust
pub fn push(&mut self, entry: TickerEntry) {
    // Compensate scroll offset for content prepended at the front.
    // New entry chars: icon + space (if icon) + text + trailing separator " ··· " (5 chars)
    if !self.entries.is_empty() {
        let icon_len = if entry.icon.is_empty() {
            0
        } else {
            entry.icon.chars().count() + 1 // icon + space
        };
        let text_len = entry.text.chars().count();
        let separator_len = 5; // " ··· "
        self.scroll_offset += (icon_len + text_len + separator_len) as f64;
    }

    if self.entries.len() >= TICKER_MAX_ENTRIES {
        self.entries.pop_back();
    }
    self.entries.push_front(entry);
    self.pause_ticks = TICKER_PAUSE_TICKS;
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_loot_ticker_push_compensates_scroll_offset`
Expected: PASS

**Step 5: Run full test suite**

Run: `cargo test`
Expected: All pass. Existing ticker tests should still pass since they don't assert exact offset values after push.

**Step 6: Commit**

```bash
git add src/core/game_state.rs
git commit -m "fix: compensate ticker scroll offset on new entry to prevent jarring shift"
```

---

### Task 5: Final validation

**Step 1: Run full CI checks**

Run: `make check` (or `scripts/ci-checks.sh`)
Expected: All pass (fmt, clippy, test, build, audit).

**Step 2: Visual verification plan** (manual)

- [ ] XL terminal (120x40+): Loot panel (left) + Combat log (right) side-by-side, no ticker
- [ ] L terminal (80x30+): Same as XL
- [ ] M terminal (60x24+): Ticker row + compact combat log, no loot panel
- [ ] S terminal (40x16+): Ticker row + compact combat log, no merged feed
