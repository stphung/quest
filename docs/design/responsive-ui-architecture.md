# Responsive UI: Architecture and Implementation Plan

This document provides a concrete technical plan for implementing responsive terminal
UI in Quest. It synthesizes findings from the [UI audit](responsive-ui-audit.md),
[breakpoint design](responsive-ui-breakpoints.md),
[wireframes](responsive-ui-wireframes.md), and
[game information hierarchy](responsive-ui-game-priorities.md).

---

## 1. Core Abstraction: `LayoutContext`

### 1.1 New File: `src/ui/responsive.rs`

A single new module houses all responsive logic. No new dependencies required.

```rust
/// Terminal size tier — determined once per frame, passed everywhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SizeTier {
    TooSmall,
    S,   // 40x16+
    M,   // 60x24+
    L,   // 80x30+
    XL,  // 120x40+
}

/// Independent width/height tier — allows "L-width but M-height" combinations.
#[derive(Debug, Clone, Copy)]
pub struct LayoutContext {
    pub width_tier: SizeTier,
    pub height_tier: SizeTier,
    /// The effective tier: min(width_tier, height_tier).
    /// Use this when a single tier value is needed.
    pub tier: SizeTier,
    /// Raw terminal dimensions for fine-grained decisions.
    pub cols: u16,
    pub rows: u16,
}
```

**Why `LayoutContext` instead of just `SizeTier`?**

Width and height are evaluated independently. A 100-column x 22-row terminal
is L-width but S-height. Components that only care about vertical space (like
the stats panel section count) can check `height_tier` alone. Components that
only care about horizontal space (like the 50/50 vs stacked split) can check
`width_tier` alone. The `tier` field provides a single conservative answer
for code that does not want to think about this distinction.

### 1.2 Detection Logic

```rust
// Thresholds (with hysteresis handled at call site)
const XL_MIN_COLS: u16 = 120;
const XL_MIN_ROWS: u16 = 40;
const L_MIN_COLS: u16 = 80;
const L_MIN_ROWS: u16 = 30;
const M_MIN_COLS: u16 = 60;
const M_MIN_ROWS: u16 = 24;
const S_MIN_COLS: u16 = 40;
const S_MIN_ROWS: u16 = 16;

impl LayoutContext {
    pub fn from_frame(frame: &Frame) -> Self {
        let size = frame.size();
        let cols = size.width;
        let rows = size.height;

        let width_tier = classify(cols, XL_MIN_COLS, L_MIN_COLS, M_MIN_COLS, S_MIN_COLS);
        let height_tier = classify(rows, XL_MIN_ROWS, L_MIN_ROWS, M_MIN_ROWS, S_MIN_ROWS);
        let tier = width_tier.min(height_tier);

        LayoutContext { width_tier, height_tier, tier, cols, rows }
    }
}

fn classify(val: u16, xl: u16, l: u16, m: u16, s: u16) -> SizeTier {
    if val >= xl { SizeTier::XL }
    else if val >= l { SizeTier::L }
    else if val >= m { SizeTier::M }
    else if val >= s { SizeTier::S }
    else { SizeTier::TooSmall }
}
```

**Hysteresis:** Deferred to Phase 2+. The classification above is stateless.
If flickering becomes an issue, we can add a `previous_tier` field to
`LayoutContext` and implement a 2-unit buffer at the thresholds. For now,
stateless detection is simpler and sufficient because ratatui redraws every
frame — a one-frame "wrong tier" is invisible at 10fps.

### 1.3 Where Detection Happens

Detection happens **once per frame** at the top of `draw_ui_with_update()` in
`src/ui/mod.rs`. The resulting `LayoutContext` is passed down to every draw function.

```rust
pub fn draw_ui_with_update(frame: &mut Frame, ...) {
    let ctx = LayoutContext::from_frame(frame);

    if ctx.tier == SizeTier::TooSmall {
        render_too_small(frame, ctx);
        return;
    }

    // ... rest of layout, passing ctx to all functions
}
```

---

## 2. Signature Changes

### 2.1 Approach: Add `&LayoutContext` Parameter

Every draw function receives `&LayoutContext`. This is a small, `Copy`-able struct
so it is cheap to pass around. We do NOT store it in `GameState` (it is UI-only
and changes every frame).

### 2.2 Functions That Need the Parameter

**Phase 1 (infrastructure — tier detection, no behavior changes):**

All of these functions gain `ctx: &LayoutContext` as a new parameter:

| File | Function | Notes |
|------|----------|-------|
| `mod.rs` | `draw_ui_with_update()` | Creates ctx, passes it down |
| `mod.rs` | `draw_challenge_banner()` | Needs ctx to hide at S tier |
| `mod.rs` | `draw_right_panel()` | Needs ctx for zone info height |
| `mod.rs` | `draw_right_content()` | Needs ctx for minigame sizing |
| `mod.rs` | `draw_dungeon_view()` | Needs ctx for map/combat split |
| `mod.rs` | `draw_dungeon_panel()` | Pass-through |
| `stats_panel.rs` | `draw_stats_panel()` | Needs ctx for section layout |
| `stats_panel.rs` | `draw_header()` | Needs ctx for condensed mode |
| `stats_panel.rs` | `draw_prestige_info()` | Needs ctx for condensed mode |
| `stats_panel.rs` | `draw_attributes()` | Needs ctx to choose layout |
| `stats_panel.rs` | `draw_derived_stats()` | Needs ctx to choose layout |
| `stats_panel.rs` | `draw_equipment_section()` | Needs ctx for names-only mode |
| `stats_panel.rs` | `draw_zone_info()` | Needs ctx for condensed mode |
| `stats_panel.rs` | `draw_footer()` | Needs ctx for compact footer |
| `info_panel.rs` | `draw_info_panel()` | Needs ctx for merged feed at S |
| `combat_scene.rs` | `draw_combat_scene()` | Needs ctx for sprite hiding |
| `fishing_scene.rs` | `render_fishing_scene()` | Needs ctx for layout |
| `game_common.rs` | `create_game_layout()` | Needs ctx for info panel hiding |

**Minigame scenes** (can defer to Phase 4, but add parameter in Phase 1 for
forward compatibility):

All `render_*_scene()` functions in chess/go/morris/gomoku/minesweeper/rune/
flappy/snake scene files gain `ctx: &LayoutContext`.

**Full-screen views** (can defer, lower priority):

haven_scene, achievement_browser_scene, prestige_confirm, character_select/
creation/delete/rename, debug_menu_scene.

### 2.3 Why Not a Trait or Global?

- **Global/thread-local**: Would require unsafe or mutex; harder to test.
- **Trait**: No polymorphism needed — all draw functions are plain functions.
- **Parameter**: Simple, explicit, testable. Grep-able. Matches existing patterns
  (game_state is already passed everywhere the same way).

---

## 3. Layout Strategy by Tier

### 3.1 Top-Level Layout (`mod.rs`)

The main `draw_ui_with_update()` function changes its layout strategy based on
`ctx.tier`:

```rust
match ctx.tier {
    SizeTier::XL | SizeTier::L => {
        // Two-column layout: stats left (50%), activity right (50%)
        // Info panel and footer at bottom
        draw_xl_l_layout(frame, ctx, game_state, ...);
    }
    SizeTier::M => {
        // Stacked layout: compact header, full-width activity, compact footer
        draw_m_layout(frame, ctx, game_state, ...);
    }
    SizeTier::S => {
        // Minimal: status line, HP bars, activity feed, footer
        draw_s_layout(frame, ctx, game_state, ...);
    }
    SizeTier::TooSmall => {
        render_too_small(frame, ctx);
    }
}
```

Each layout function is a private function in `mod.rs` that handles the
`Layout::default().direction(...).constraints(...)` for that tier. This
prevents a single monolithic function with tier-branching everywhere.

**Important:** XL and L share the same top-level layout structure (two-column
with stats + activity). The difference is only in the stats panel's internal
condensation. So they share a single `draw_xl_l_layout()` function, and the
stats panel itself checks `ctx.tier` internally.

### 3.2 Stats Panel Strategy

The stats panel in `draw_stats_panel()` selects which sections to include
based on `ctx.height_tier`:

```rust
pub fn draw_stats_panel(frame: &mut Frame, area: Rect, game_state: &GameState, ctx: &LayoutContext) {
    match ctx.height_tier {
        SizeTier::XL => {
            // Full layout: header(4) + prestige(7) + attrs(14) + derived(6) + equip(min 16) = 47
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(4),   // Header
                    Constraint::Length(7),   // Prestige
                    Constraint::Length(14),  // Attributes (full 6x2)
                    Constraint::Length(6),   // Derived stats
                    Constraint::Min(16),     // Equipment (full)
                ])
                .split(area);
            draw_header(frame, chunks[0], game_state, ctx);
            draw_prestige_info(frame, chunks[1], game_state, ctx);
            draw_attributes(frame, chunks[2], game_state, ctx);
            draw_derived_stats(frame, chunks[3], game_state, ctx);
            draw_equipment_section(frame, chunks[4], game_state, ctx);
        }
        SizeTier::L => {
            // Condensed: header(4) + prestige(5) + attrs_compact(4) + derived_compact(3) + equip_names(9)
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(4),
                    Constraint::Length(5),
                    Constraint::Length(4),
                    Constraint::Length(3),
                    Constraint::Min(9),
                ])
                .split(area);
            draw_header(frame, chunks[0], game_state, ctx);
            draw_prestige_info(frame, chunks[1], game_state, ctx);
            draw_attributes_compact(frame, chunks[2], game_state);
            draw_derived_stats_compact(frame, chunks[3], game_state);
            draw_equipment_names_only(frame, chunks[4], game_state);
        }
        _ => {
            // M and S don't use stats panel (handled by stacked layout)
        }
    }
}
```

**New private functions needed in stats_panel.rs:**
- `draw_attributes_compact()` — 2-column, 3 rows of attribute pairs
- `draw_derived_stats_compact()` — 2 inline rows
- `draw_equipment_names_only()` — 7 lines, one per slot, name + rarity tag only

These are straightforward subsets of existing rendering code. The existing full
functions remain unchanged for XL tier.

### 3.3 Info Panel Strategy

```rust
pub fn draw_info_panel(frame: &mut Frame, area: Rect, game_state: &GameState, ctx: &LayoutContext) {
    match ctx.tier {
        SizeTier::XL | SizeTier::L => {
            // Current: 50/50 side-by-side with borders
            draw_loot_combat_split(frame, area, game_state);
        }
        SizeTier::M => {
            // Narrower side-by-side, compact (no borders or minimal borders)
            draw_loot_combat_compact(frame, area, game_state);
        }
        SizeTier::S => {
            // Merged chronological feed (loot + combat interleaved)
            draw_merged_feed(frame, area, game_state);
        }
        SizeTier::TooSmall => {}
    }
}
```

**New function needed:** `draw_merged_feed()` — interleaves loot and combat
entries by timestamp into a single scrolling list.

### 3.4 Footer Strategy

```rust
pub fn draw_footer(frame: &mut Frame, area: Rect, ..., ctx: &LayoutContext) {
    match ctx.tier {
        SizeTier::XL | SizeTier::L => {
            // Current: 3 rows with block borders, all controls + update status
            draw_footer_full(frame, area, ...);
        }
        SizeTier::M => {
            // 1 row, no borders: [Esc]Quit [P]Prestige [H]Haven [A]Ach [Tab]Chall [E]Equip
            draw_footer_compact(frame, area, ...);
        }
        SizeTier::S => {
            // 1 row, minimal: Esc:Quit P:Prestige Tab:More
            draw_footer_minimal(frame, area, ...);
        }
        SizeTier::TooSmall => {}
    }
}
```

### 3.5 M-Tier Stacked Layout

For M tier, `draw_m_layout()` in `mod.rs` creates:

```rust
fn draw_m_layout(frame: &mut Frame, ctx: &LayoutContext, game_state: &GameState, ...) {
    let area = frame.size(); // or adjusted for banner
    let show_attrs = ctx.rows >= 26; // hide attrs line if very tight

    let mut constraints = vec![
        Constraint::Length(1),   // Compact stats bar (name, level, prestige, zone)
    ];
    if show_attrs {
        constraints.push(Constraint::Length(1)); // Condensed attrs single line
    }
    constraints.push(Constraint::Length(1));     // XP bar
    constraints.push(Constraint::Min(8));        // Activity area (full width)
    constraints.push(Constraint::Length(4));      // Info panel
    constraints.push(Constraint::Length(1));      // Footer

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    // Render each section...
    draw_compact_stats_bar(frame, chunks[0], game_state, ctx);
    // ...etc
}
```

**New functions needed:**
- `draw_compact_stats_bar()` — single line: `Hero Lv.42 | P:12 Gold 2.80x | Zone 3: Mountain (2/3)`
- `draw_attributes_single_line()` — `STR:24 DEX:18 CON:21 INT:15 WIS:12 CHA:16`

### 3.6 S-Tier Minimal Layout

```rust
fn draw_s_layout(frame: &mut Frame, ctx: &LayoutContext, game_state: &GameState, ...) {
    let area = frame.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Status line (name, level, prestige, zone)
            Constraint::Length(1),  // XP bar
            Constraint::Length(1),  // Player HP
            Constraint::Length(1),  // Enemy HP + name
            Constraint::Length(1),  // Combat status (fighting/regen)
            Constraint::Min(4),    // Activity / merged feed
            Constraint::Length(1), // Footer
        ])
        .split(area);

    // Each renders a single line, borderless
}
```

---

## 4. Minigame Adaptation

### 4.1 Shared Layout Changes

`create_game_layout()` in `game_common.rs` gains `ctx: &LayoutContext` and
adjusts the info panel:

```rust
pub fn create_game_layout(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    border_color: Color,
    content_min_height: u16,
    info_panel_width: u16,
    ctx: &LayoutContext,
) -> GameLayout {
    // At M tier and below, hide the info panel to give the board more space
    let effective_info_width = if ctx.width_tier >= SizeTier::L {
        info_panel_width
    } else {
        0 // info panel hidden; board gets full width
    };

    // ... rest of layout with effective_info_width
}
```

### 4.2 "Terminal Too Small" for Board Games

Each minigame scene checks at the top whether the available area is sufficient
for its board:

```rust
pub fn render_chess_scene(frame: &mut Frame, area: Rect, game: &ChessGame, ctx: &LayoutContext) {
    const MIN_WIDTH: u16 = 45;  // board width without info panel
    const MIN_HEIGHT: u16 = 22; // board + status bar

    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_minigame_too_small(frame, area, "Chess", MIN_WIDTH, MIN_HEIGHT);
        return;
    }
    // ... normal rendering
}
```

**New shared function:** `render_minigame_too_small()` in `game_common.rs`:

```rust
pub fn render_minigame_too_small(
    frame: &mut Frame,
    area: Rect,
    game_name: &str,
    min_width: u16,
    min_height: u16,
) {
    frame.render_widget(Clear, area);
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("{} in progress", game_name),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("Terminal too small to display board."),
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            format!("Need: {}x{}   Have: {}x{}", min_width, min_height, area.width, area.height),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled("Please resize your terminal.", Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(Span::styled("[Esc] Forfeit", Style::default().fg(Color::DarkGray))),
    ];
    let text = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(text, area);
}
```

### 4.3 Minigame-Specific Size Thresholds

| Minigame | Board-Only Min W x H | With Info Panel Min W x H |
|----------|---------------------|--------------------------|
| Chess | 45 x 22 | 67 x 22 |
| Go | 27 x 14 | 51 x 14 |
| Morris | 27 x 16 | 51 x 16 |
| Gomoku | 31 x 18 | 53 x 18 |
| Minesweeper | 20-62 x 13-20 | 44-86 x 13-20 |
| Rune | 26 x 10 | 48 x 10 |
| Flappy Bird | 30 x 12 | 48 x 12 |
| Snake | 20 x 12 | 36 x 12 |

At M-width (60-79) without info panel, all games fit except Chess at the low
end. At S-width (40-59), only Rune and possibly small Minesweeper fit.

---

## 5. Overlay and Modal Adaptation

### 5.1 General Rule

All modals should cap their dimensions to the available area:

```rust
let modal_width = DESIRED_WIDTH.min(area.width.saturating_sub(2));
let modal_height = DESIRED_HEIGHT.min(area.height.saturating_sub(2));
```

Most modals already do `min()` capping for height. We need to add width capping
where it is missing.

### 5.2 Specific Modal Changes

| Modal | Current Size | Change |
|-------|-------------|--------|
| Prestige Confirm | 50w x 18h | Add `min(area.width - 2)` for width |
| Haven Discovery | 50w x 7h | Add `min(area.width - 2)` for width |
| Haven Build | 45w x 9h | Add `min(area.width - 2)` for width |
| Storm Forge | 50w x 12h | Add `min(area.width - 2)` for width |
| Leviathan | 64w x 16h | Add `min(area.width - 2)` for width |
| Offline Welcome | 44w x 10h | Already small enough for S tier |
| Achievement Unlock | 50w x 9-20h | Add width capping |
| Debug Menu | 35w x Nh | Already small enough |

---

## 6. Code Organization

### 6.1 New Files

| File | Purpose |
|------|---------|
| `src/ui/responsive.rs` | `SizeTier`, `LayoutContext`, constants, `render_too_small()` |

### 6.2 Modified Files

| File | Changes |
|------|---------|
| `src/ui/mod.rs` | Import responsive, create ctx, split into tier layout functions |
| `src/ui/stats_panel.rs` | Add ctx param, add compact/condensed draw functions |
| `src/ui/info_panel.rs` | Add ctx param, add merged feed function |
| `src/ui/combat_scene.rs` | Add ctx param, sprite hiding at small sizes |
| `src/ui/game_common.rs` | Add ctx param to `create_game_layout()`, add `render_minigame_too_small()` |
| `src/ui/fishing_scene.rs` | Add ctx param, stacked layout at M |
| `src/ui/chess_scene.rs` | Add ctx param, size check |
| `src/ui/go_scene.rs` | Add ctx param, size check |
| `src/ui/morris_scene.rs` | Add ctx param, size check |
| `src/ui/gomoku_scene.rs` | Add ctx param, size check |
| `src/ui/minesweeper_scene.rs` | Add ctx param, size check |
| `src/ui/rune_scene.rs` | Add ctx param |
| `src/ui/flappy_scene.rs` | Add ctx param |
| `src/ui/snake_scene.rs` | Add ctx param |
| `src/ui/haven_scene.rs` | Modal width capping |
| `src/ui/prestige_confirm.rs` | Modal width capping |
| `src/ui/achievement_browser_scene.rs` | Modal width capping |
| `src/ui/challenge_menu_scene.rs` | Add ctx param |
| `src/ui/dungeon_map.rs` | No changes needed (widget-based, already size-aware) |

### 6.3 No Changes Needed

| File | Reason |
|------|--------|
| `src/ui/enemy_sprites.rs` | Data only, no rendering decisions |
| `src/ui/throbber.rs` | Single char + text, inherently responsive |
| `src/ui/combat_effects.rs` | Effect types only |
| `src/ui/combat_3d.rs` | Already has "area too small" check |
| All `src/` non-UI modules | UI changes are isolated to `src/ui/` |

---

## 7. Avoiding Code Duplication

### 7.1 Principle: Compose, Don't Branch

Bad (duplicates rendering logic):
```rust
if ctx.tier >= SizeTier::L {
    // 20 lines of full attribute rendering
} else {
    // 20 lines of compact attribute rendering (copy-pasted with tweaks)
}
```

Good (separate functions, shared helpers):
```rust
// Each is a focused function with clear responsibility
fn draw_attributes_full(frame, area, game_state) { ... }
fn draw_attributes_compact(frame, area, game_state) { ... }
fn draw_attributes_single_line(frame, area, game_state) { ... }

// Shared helper used by all three:
fn format_attribute_value(attr: &Attribute) -> String { ... }
```

### 7.2 Shared Formatting Helpers

Extract these from existing code in stats_panel.rs:

- `format_attribute_value(name, value, modifier)` -> `"STR:24(+7)"`
- `format_equipment_summary(item)` -> `"[Rare] Iron Sword"`
- `format_derived_stat(name, value)` -> `"HP:150"`
- `format_prestige_summary(rank, tier_name, multiplier)` -> `"P:12 Gold 2.80x"`
- `format_zone_summary(zone, subzone, boss_kills)` -> `"Zone 3: Mountain (2/3)"`

These helpers can be used by the full, compact, and single-line variants.

### 7.3 The `draw_right_content` Dispatch

Currently `draw_right_content()` dispatches to minigame scenes with different
signatures. After adding `ctx`, all calls consistently pass it through:

```rust
fn draw_right_content(frame: &mut Frame, area: Rect, game_state: &GameState, ctx: &LayoutContext) {
    match &game_state.active_minigame {
        Some(ActiveMinigame::Chess(game)) => chess_scene::render_chess_scene(frame, area, game, ctx),
        // ... same pattern for all minigames
    }
}
```

---

## 8. Testing Strategy

### 8.1 Unit Tests for LayoutContext

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xl_classification() {
        let ctx = LayoutContext::from_size(120, 40);
        assert_eq!(ctx.tier, SizeTier::XL);
    }

    #[test]
    fn test_mixed_tiers() {
        let ctx = LayoutContext::from_size(120, 22);
        assert_eq!(ctx.width_tier, SizeTier::XL);
        assert_eq!(ctx.height_tier, SizeTier::S);
        assert_eq!(ctx.tier, SizeTier::S); // min of both
    }

    #[test]
    fn test_too_small() {
        let ctx = LayoutContext::from_size(39, 20);
        assert_eq!(ctx.tier, SizeTier::TooSmall);
    }
}
```

For testing, add `LayoutContext::from_size(cols, rows)` constructor that does
not require a Frame.

### 8.2 Visual Testing

No automated visual regression tests (ratatui has no built-in snapshot testing
in this project). Manual testing approach:

1. Run game, resize terminal to each tier boundary
2. Verify layout transitions are smooth (no panics, no overlapping content)
3. Verify "too small" message appears below 40x16
4. Verify minigame "too small" messages appear correctly

### 8.3 Integration Tests

Existing tests do not exercise UI (UI module is private in lib.rs). No new
integration tests needed for Phase 1, since the behavior is unchanged.

---

## 9. Phased Implementation Plan

### Phase 1: Infrastructure (1 PR)

**Goal:** Add `LayoutContext` detection, pass it through all draw functions,
zero behavior changes. All tiers render identically to current behavior.

**Changes:**
1. Create `src/ui/responsive.rs` with `SizeTier`, `LayoutContext`, constants
2. Add `ctx: &LayoutContext` parameter to `draw_ui_with_update()` and all
   functions it calls (cascading through the call tree)
3. Create `LayoutContext` at top of `draw_ui_with_update()`, pass it everywhere
4. Add `render_too_small()` function (only triggers below 40x16)
5. All existing behavior remains exactly the same (XL rendering always used)
6. Add unit tests for `LayoutContext::from_size()`

**Files touched:** ~20 UI files (parameter addition only)
**Risk:** Low — purely additive, no behavior change
**Verification:** `make check` passes, game renders identically

### Phase 2: L Tier - Condensed Stats (1 PR)

**Goal:** When terminal is 80-119 cols x 30-39 rows, condense the stats panel.

**Changes:**
1. Add `draw_attributes_compact()` in stats_panel.rs (2-column, 3 rows)
2. Add `draw_derived_stats_compact()` in stats_panel.rs (2 inline rows)
3. Add `draw_equipment_names_only()` in stats_panel.rs (1 line per slot)
4. Modify `draw_stats_panel()` to branch on `ctx.height_tier`
5. Reduce info panel height from 8 to 6 at L tier
6. Extract shared formatting helpers

**Files touched:** `stats_panel.rs`, `mod.rs`, `info_panel.rs`
**Risk:** Low — stats panel internals only, no layout restructure
**Verification:** Resize terminal to 100x35, verify condensed stats

### Phase 3: M Tier - Stacked Layout (1 PR)

**Goal:** At 60-79 cols x 24-29 rows, switch to single-column stacked layout.

**Changes:**
1. Add `draw_m_layout()` in mod.rs
2. Add `draw_compact_stats_bar()` — single-line header
3. Add `draw_attributes_single_line()` — all 6 attrs on one line
4. Modify `draw_footer()` to render 1-line compact at M tier
5. Modify `draw_info_panel()` for compact mode at M tier
6. Combat scene: hide sprite at M height, show only HP bars + status

**Files touched:** `mod.rs`, `stats_panel.rs`, `info_panel.rs`, `combat_scene.rs`
**Risk:** Medium — new top-level layout path, but isolated to M tier
**Verification:** Resize to 70x26, verify stacked layout

### Phase 4: S Tier - Minimal Layout (1 PR)

**Goal:** At 40-59 cols x 16-23 rows, render minimal text-only layout.

**Changes:**
1. Add `draw_s_layout()` in mod.rs
2. Add `draw_merged_feed()` in info_panel.rs (interleaved loot + combat)
3. Add `draw_footer_minimal()` — `Esc:Quit P:Prestige Tab:More`
4. Borderless rendering throughout
5. Challenge banner hidden at S tier

**Files touched:** `mod.rs`, `stats_panel.rs`, `info_panel.rs`
**Risk:** Medium — most different from current layout
**Verification:** Resize to 50x18, verify minimal layout

### Phase 5: Minigame + Modal Adaptation (1 PR)

**Goal:** Minigames handle small terminals gracefully; modals cap to screen size.

**Changes:**
1. Add `render_minigame_too_small()` to game_common.rs
2. Add size checks to all minigame `render_*()` functions
3. Modify `create_game_layout()` to hide info panel at M tier
4. Add width capping to all modals (prestige, haven, leviathan, etc.)
5. Haven/achievement overlays adapt to M/S tiers

**Files touched:** `game_common.rs`, all `*_scene.rs`, `haven_scene.rs`,
`prestige_confirm.rs`, `achievement_browser_scene.rs`
**Risk:** Low — each change is independent and localized
**Verification:** Resize during active minigame, verify "too small" messages

### Phase 6: Full-Screen View Adaptation (1 PR)

**Goal:** Character select, creation, deletion, rename screens adapt to
smaller terminals.

**Changes:**
1. Character select: condense layout at M, hide Haven tree at S
2. Character creation/delete/rename: reduce margins, condense at M
3. Achievement browser: list-only at M, detail on Enter
4. Haven scene: list view at M, compact at S

**Files touched:** `character_select.rs`, `character_creation.rs`,
`character_delete.rs`, `character_rename.rs`, `haven_scene.rs`,
`achievement_browser_scene.rs`
**Risk:** Low — each is independent
**Verification:** Open each view at M and S sizes

---

## 10. Migration Checklist

For each phase, ensure:

- [ ] `make check` passes (format, lint, test, build)
- [ ] Game runs at XL (120x40+) with no visual changes
- [ ] Game runs at target tier with correct layout
- [ ] No panics at any terminal size down to 1x1
- [ ] Overlays/modals render correctly at target tier
- [ ] Active minigames show "too small" message when appropriate
- [ ] Keyboard input still works correctly at all tiers
- [ ] Update CLAUDE.md for src/ui/ with responsive patterns

---

## 11. Key Design Decisions

### D1: Stateless per-frame detection (not cached)

Ratatui redraws every frame. Caching the tier adds complexity (when to
invalidate?) with no benefit. `LayoutContext::from_frame()` is trivially
cheap — two comparisons on u16 values.

### D2: Independent width/height tiers

A 120x22 terminal should use L-width two-column layout but S-height content
density. Independent evaluation gives the best result for each axis without
requiring a complex matrix of size combinations.

### D3: Separate layout functions per tier (not one mega-function)

`draw_xl_l_layout()`, `draw_m_layout()`, `draw_s_layout()` are cleaner than
one function with `if ctx.tier >= M { ... }` scattered throughout. Each layout
function is ~30-50 lines and self-contained.

### D4: Parameter passing over globals

`&LayoutContext` is explicit, testable, and follows the existing pattern of
passing `&GameState` everywhere. No hidden dependencies.

### D5: XL is "current behavior, unchanged"

Phase 1 is completely safe because XL tier is always selected until tier-
specific code is added in later phases. This means the infrastructure PR
cannot introduce visual regressions.

### D6: No hysteresis in Phase 1

Flickering at tier boundaries is theoretically possible but practically
unlikely at 10fps refresh. If it becomes a user-reported issue, add a
2-frame debounce in LayoutContext. Premature optimization avoided.

### D7: Minigames show "too small" rather than degrading

Board games with fixed layouts cannot meaningfully render in half the space.
A clear "resize your terminal" message is better UX than a broken board.

---

## 12. Dependency Graph

```
Phase 1: Infrastructure ──────────────────┐
    │                                      │
    ├──> Phase 2: L Tier (condensed)       │
    │        │                              │
    │        ├──> Phase 3: M Tier (stacked) │
    │        │        │                     │
    │        │        └──> Phase 4: S Tier  │
    │        │                              │
    │        └──> Phase 5: Minigames/Modals │
    │                                       │
    └──> Phase 6: Full-screen views ────────┘
```

Phases 2-4 are sequential (each builds on the previous tier).
Phase 5 depends only on Phase 1 (and is independent of 2-4).
Phase 6 depends only on Phase 1 (and is independent of 2-5).

Phases 5 and 6 can be developed in parallel with Phases 2-4.
