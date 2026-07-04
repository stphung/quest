> Backported design record. Sources: docs/design/responsive-ui-architecture.md, docs/design/responsive-ui-audit.md, docs/design/responsive-ui-breakpoints.md, docs/design/responsive-ui-game-priorities.md, docs/design/responsive-ui-wireframes.md.

## responsive-ui-architecture.md

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

## responsive-ui-audit.md

# Responsive UI Audit: Current Layout Components

## Table of Contents

1. [Main Game Layout](#1-main-game-layout)
2. [Left Panel: Stats Panel](#2-left-panel-stats-panel)
3. [Right Panel: Activity Area](#3-right-panel-activity-area)
4. [Bottom Panel: Info Panel](#4-bottom-panel-info-panel)
5. [Footer](#5-footer)
6. [Overlays and Modals](#6-overlays-and-modals)
7. [Full-Screen Views](#7-full-screen-views)
8. [Shared Components](#8-shared-components)
9. [Summary: Hardcoded Dimensions](#9-summary-hardcoded-dimensions)
10. [Component Priority Classification](#10-component-priority-classification)

---

## 1. Main Game Layout

**File:** `src/ui/mod.rs:41-140` (`draw_ui_with_update()`)

The main game screen uses a vertical-then-horizontal layout:

```
+----------------------------------------------+
| [Challenge Banner - 1 line, conditional]      |
+----------------------+------------------------+
|                      |   Zone Info (4h)       |
|  Stats Panel (50%)   +------------------------+
|                      |   Right Content        |
|                      |   (Min 10h)            |
+----------------------+------------------------+
|  Loot Panel (50%)    |  Combat Log (50%)      | <- 8h fixed
+----------------------+------------------------+
|  Footer (3h)                                  |
+----------------------------------------------+
| [Update Drawer - 12h, conditional]            |
+----------------------------------------------+
```

**Hardcoded constraints:**
- Challenge banner: `Length(1)` (conditional)
- Main content: `Min(0)` (flexible)
- Info panel (loot + combat log): `Length(8)` fixed
- Update drawer: `Length(12)` (conditional)
- Footer: `Length(3)` fixed
- Horizontal split: `Percentage(50)` / `Percentage(50)`

**Minimum implied height:** 8 (info) + 3 (footer) + some content = ~20 rows minimum for usability

---

## 2. Left Panel: Stats Panel

**File:** `src/ui/stats_panel.rs:33-62` (`draw_stats_panel()`)

Vertical layout within the left 50%:

| Section | Constraint | Contents | Essential? |
|---------|-----------|----------|------------|
| Header (name, level, XP bar) | `Length(4)` | Character name, level, rank, playtime, XP gauge | YES |
| Prestige Info | `Length(7)` | Rank, multiplier, CHA bonus, resets, fishing rank, fishing progress bar | YES |
| Attributes | `Length(14)` | 6 attributes x 2 lines each (emoji, name, value, modifier, cap) + borders | YES |
| Derived Stats | `Length(6)` | Max HP, physical/magic damage, defense, crit%, XP mult + borders | MEDIUM |
| Equipment | `Min(16)` | 7 equipment slots, each 1-3 lines (name+rarity, attrs, affixes) | MEDIUM |

**Total fixed height:** 4 + 7 + 14 + 6 = 31 lines, plus equipment needs Min(16) = **47 lines minimum**

**Key observations:**
- The header gracefully degrades: at `inner.height >= 2` shows text + XP bar, at 1 shows just XP bar
- Prestige info gracefully degrades: at `inner.height >= 5` shows full layout, at 1 shows just fishing bar
- Attributes are rigid: 6 x `Length(2)` = 12 lines (no degradation)
- Equipment truncates item names at 28 chars
- No width constraints; relies on percentage-based parent

---

## 3. Right Panel: Activity Area

**File:** `src/ui/mod.rs:181-248` (`draw_right_panel()`, `draw_right_content()`)

The right panel always has a fixed zone info header:

| Section | Constraint | Contents |
|---------|-----------|----------|
| Zone Info | `Length(4)` | Zone name, subzone, boss progress, next zone status |
| Right Content | `Min(10)` | Dispatched by activity (see below) |

### 3a. Combat Scene

**File:** `src/ui/combat_scene.rs:15-48`

| Section | Constraint | Contents | Essential? |
|---------|-----------|----------|------------|
| Player HP bar | `Length(1)` | Gauge with HP values | YES |
| Sprite + 3D view | `Min(5)` | Enemy ASCII sprite (10 lines tall), centered | YES |
| Enemy HP bar | `Length(1)` | Gauge or "Regenerating..."/"Spawning..." | YES |
| Combat Status | `Length(1)` | Spinner, attack timers, DPS | MEDIUM |

- `combat_3d.rs:14`: Has explicit minimum check: `if area.height < 3 || area.width < 20` shows "Area too small"
- Enemy sprites are all 14 chars wide x 10 lines tall (from `enemy_sprites.rs`)

### 3b. Dungeon View

**File:** `src/ui/mod.rs:251-275` (`draw_dungeon_view()`)

| Section | Constraint | Contents |
|---------|-----------|----------|
| Dungeon Map | `Min(map_height)` | `grid_size * 2 + 3` height; grid sizes: 5/7/9/11/13 |
| Combat Scene | `Min(5)` | Same as combat scene above |

**Map heights by dungeon size:**
- Small (5x5): `5*2+3 = 13` rows
- Medium (7x7): `7*2+3 = 17` rows
- Large (9x9): `9*2+3 = 21` rows
- Epic (11x11): `11*2+3 = 25` rows
- Legendary (13x13): `13*2+3 = 29` rows

**Map widths:** `grid_size * 4` cells wide (emoji = 2 chars + 2 for corridor)
- Small: 20, Medium: 28, Large: 36, Epic: 44, Legendary: 52

### 3c. Fishing Scene

**File:** `src/ui/fishing_scene.rs:39-67`

| Section | Constraint | Contents | Essential? |
|---------|-----------|----------|------------|
| Header | `Length(3)` | Spot name, bordered | YES |
| Water Animation | `Min(6)` | ASCII water + bobber | YES |
| Catch Progress | `Length(4)` | Caught X/Y fish, phase status | YES |
| Rank Info | `Length(5)` | Rank name, progress bar | MEDIUM |

**Total minimum:** 3 + 6 + 4 + 5 = **18 lines**

### 3d. Challenge Menu

**File:** `src/ui/challenge_menu_scene.rs:21-29`

Two views:
- **List view:** Bordered list of pending challenges, help text at bottom
- **Detail view:** Description (4h) + spacer + difficulty selector (5h) + spacer + outcomes (1h) + spacer + help (1h). ~13 lines minimum

### 3e. Minigame Scenes (all use `create_game_layout`)

All minigames use the shared layout from `game_common.rs:44-80`:

```
+------ Title -------------------------+---- Info ---+
|                                      |             |
|   [content area]                     |  [info]     |
|                                      |             |
| [status bar - 2 lines]              |             |
+--------------------------------------+-------------+
```

**Layout constraints:**
- Horizontal: `Min(20)` for content | `Length(info_panel_width)` for info
- Vertical (left side): `Min(content_min_height)` | `Length(2)` for status bar

| Minigame | Content Min Height | Info Width | Board Dimensions |
|----------|-------------------|------------|-----------------|
| Chess | 19 (1 move history + 18 board) | 22 | 43w x 18h (grid: 5*8+3 = 43w) |
| Go | 11 | 24 | 25w x 9h (BOARD_SIZE=9) |
| Morris | 13 | 24 | 25w x 13h |
| Gomoku | 15 | 22 | 29w x 15h (BOARD_SIZE=15, cells=2w) |
| Minesweeper | 10 | 24 | Grid varies: 9x9=18w, 16x16=32w, 30x16=60w, 20x16=40w |
| Rune | 6 | 22 | Variable height (guess history grows) |
| Flappy Bird | 15 | 18 | GAME_WIDTH x GAME_HEIGHT (scalable) |
| Snake | 20 | 16 | grid_width x (grid_height/2) (half-block rendering) |

---

## 4. Bottom Panel: Info Panel

**File:** `src/ui/info_panel.rs:12-23`

**Fixed height:** 8 rows (set by parent layout)

Horizontal split: 50% / 50%:
- **Left: Loot Panel** - Recent drops with rarity colors, 2-line format for equipment (name + stats)
- **Right: Combat Log** - Newest-first combat entries, color-coded (green=player, red=enemy, yellow=crit)

Both panels use `inner.height` to calculate max visible entries. Truncates messages to `inner.width`.

**Essential?** YES - Primary feedback for idle gameplay

---

## 5. Footer

**File:** `src/ui/stats_panel.rs:778-877` (`draw_footer()`)

**Fixed height:** 3 rows (border + 1 content line + border)

Contains horizontally:
- `[Esc] Quit`
- `[P] Prestige (Available!)` or `[P] Prestige (Need Lv.X)`
- `[H] Haven` (conditional on discovery)
- `[A] Achievements` (with pending count)
- `[Tab] Challenges` (conditional on pending count)
- Update status (checking spinner / up to date / new version)
- Version info in title bar

**Essential?** YES - Primary navigation and status

---

## 6. Overlays and Modals

All overlays use `frame.render_widget(Clear, area)` before rendering, and are centered on screen.

| Overlay | File | Dimensions | Trigger |
|---------|------|-----------|---------|
| Prestige Confirm | `prestige_confirm.rs` | 50w x 18h (capped to screen-4) | Press [P] |
| Achievement Unlocked | `achievement_browser_scene.rs:302` | 50w x 9-20h | Achievement triggers |
| Haven Discovery | `haven_scene.rs:427` | 50w x 7h | First Haven discovery |
| Haven Build Confirm | `haven_scene.rs:473` | 45w x 9h | Build in Haven |
| Storm Forge Confirm | `haven_scene.rs:547` | 50w x 12h | Forge Stormbreaker |
| Leviathan Encounter | `fishing_scene.rs:350` | 64w x 16h | Leviathan event |
| Offline Welcome | `game_common.rs:310` | 44w x 10-11h | Return after offline |
| Debug Menu | `debug_menu_scene.rs:13` | 35w x (options+4)h | Backtick key (debug mode) |
| Game Over (full) | `game_common.rs:179` | Full area, centered content 7h | Minesweeper/Rune end |
| Game Over (banner) | `game_common.rs:239` | Full width, 4-5h at bottom | Chess/Go/Morris/Gomoku end |

**Essential?** All are essential when shown (modal interactions)

---

## 7. Full-Screen Views

These replace the entire terminal, not just the right panel:

### 7a. Character Select

**File:** `src/ui/character_select.rs:24-111`

Layout:
- Title: `Length(3)`
- Main content (40%/60% horizontal split): Character list | Character details
- Haven tree (conditional, discovered): `Length(19)` - 17 lines of diamond layout
- Controls: `Length(4)`
- Margin: 2 on all sides

**Minimum dimensions:** ~40w x 30h (without Haven), ~40w x 49h (with Haven tree)

### 7b. Character Creation

**File:** `src/ui/character_creation.rs:26-111`

Layout: Title(3) + spacer(1) + input(4) + spacer(1) + rules(4) + validation(2) + filler + controls(3) = ~18 minimum + margins

### 7c. Character Delete

**File:** `src/ui/character_delete.rs:26-109`

Layout: Title(3) + spacer(1) + details(min) + spacer(1) + warning(5) + spacer(1) + input(4) + spacer(1) + controls(3) = ~19 minimum + margins

### 7d. Character Rename

**File:** `src/ui/character_rename.rs:28-126`

Layout: Title(3) + spacer(1) + details(min) + spacer(1) + input(4) + spacer(1) + rules(4) + validation(2) + spacer(1) + controls(3) = ~20 minimum + margins

### 7e. Achievement Browser

**File:** `src/ui/achievement_browser_scene.rs:85-128`

Full-screen overlay with:
- Category tabs: `Length(3)`
- Content (45%/55% horizontal split): Achievement list | Achievement detail
- Help: `Length(1)`

### 7f. Haven Tree

**File:** `src/ui/haven_scene.rs:43-94`

Full-screen overlay with:
- Summary bar: `Length(2)`
- Main content (40%/60% horizontal split): Skill tree list | Room detail
- Help: `Length(1)`

### 7g. Vault Selection (during prestige)

**File:** `src/ui/haven_scene.rs:642-755`

Full-screen overlay with:
- Instructions: `Length(2)`
- Item list: `Min(0)`
- Help: `Length(1)`

---

## 8. Shared Components

### 8a. Throbber / Spinner

**File:** `src/ui/throbber.rs`

- `spinner_char()`: Braille spinner (10 frames, 100ms cycle)
- `waiting_message(seed)`: 20 atmospheric messages for idle time

**Dimensions:** Single character + message text. No size constraints.

### 8b. Game Common Layout

**File:** `src/ui/game_common.rs`

- `create_game_layout()`: Standardized minigame layout
- `render_status_bar()`: 2-line status with controls
- `render_thinking_status_bar()`: AI thinking spinner
- `render_forfeit_status_bar()`: Forfeit confirmation
- `render_game_over_overlay()`: Full-area game over
- `render_game_over_banner()`: Bottom banner (4-5h)
- `render_info_panel_frame()`: Bordered info panel
- `render_offline_welcome()`: Centered 44x10 modal
- `format_number_short()`: Number abbreviation (K/M/B/T/Q)

### 8c. Enemy Sprites

**File:** `src/ui/enemy_sprites.rs`

6 sprite templates, all 14 chars wide x 10 lines tall:
- `SPRITE_ORC`, `SPRITE_TROLL`, `SPRITE_DRAKE`
- `SPRITE_BEAST`, `SPRITE_HORROR`, `SPRITE_CRUSHER`

### 8d. Debug Indicators

**File:** `src/ui/debug_menu_scene.rs:73-131`

- `render_debug_indicator()`: "[DEBUG] Saves disabled" top-right (22 chars)
- `render_save_indicator()`: "Saved HH:MM AM" or spinner, top-right

---

## 9. Summary: Hardcoded Dimensions

### Fixed Heights (cannot shrink)

| Component | Height | Location |
|-----------|--------|----------|
| Challenge banner | 1 | `mod.rs:61` |
| Info panel (loot+combat) | 8 | `mod.rs:82,91` |
| Footer | 3 | `mod.rs:84,92` |
| Update drawer | 12 | `mod.rs:83` |
| Zone info | 4 | `mod.rs:191` |
| Stats header | 4 | `stats_panel.rs:40` |
| Stats prestige | 7 | `stats_panel.rs:41` |
| Stats attributes | 14 | `stats_panel.rs:42` |
| Stats derived | 6 | `stats_panel.rs:43` |
| Stats equipment (min) | 16 | `stats_panel.rs:44` |

### Fixed Widths

| Component | Width | Location |
|-----------|-------|----------|
| Prestige confirm modal | 50 | `prestige_confirm.rs:16` |
| Haven discovery modal | 50 | `haven_scene.rs:429` |
| Haven build confirm | 45 | `haven_scene.rs:481` |
| Storm forge confirm | 50 | `haven_scene.rs:554` |
| Leviathan modal | 64 | `fishing_scene.rs:357` |
| Offline welcome modal | 44 | `game_common.rs:312` |
| Debug menu | 35 | `debug_menu_scene.rs:15` |
| Chess board | 43 | `chess_scene.rs:45` |
| Morris board | 25 | `morris_scene.rs:78` |
| Go board | 25 | `go_scene.rs:36` (9*3-2) |
| Gomoku board | 29 | `gomoku_scene.rs:44` (15*2-1) |

### Percentage-Based Splits

| Split | Ratio | Location |
|-------|-------|----------|
| Stats / Right panel | 50% / 50% | `mod.rs:109-110` |
| Loot / Combat log | 50% / 50% | `info_panel.rs:15-16` |
| Haven tree / detail | 40% / 60% | `haven_scene.rs:76-77` |
| Character list / details | 40% / 60% | `character_select.rs:58-59` |
| Achievement list / detail | 45% / 55% | `achievement_browser_scene.rs:117-118` |
| Minigame content / info | `Min(20)` / `Length(W)` | `game_common.rs:64-65` |

---

## 10. Component Priority Classification

### Tier 1: Critical (must always be visible during gameplay)

- **Player HP** - survival feedback
- **Enemy HP / combat status** - combat feedback
- **XP bar** - progression feedback
- **Zone info** - location awareness
- **Footer controls** - navigation
- **Loot panel** - idle reward feedback
- **Combat log** - action feedback

### Tier 2: Important (should be visible when space allows)

- **Character level + name** - identity
- **Prestige rank + multiplier** - progression context
- **Attributes (condensed)** - build awareness
- **Equipment (names only)** - gear awareness
- **Fishing rank** - secondary progression
- **Challenge banner** - pending notification

### Tier 3: Nice-to-Have (can be hidden or accessed via overlay)

- **Derived stats** - calculable from attributes
- **Equipment affixes/details** - secondary item info
- **Attribute modifiers** - derived from values
- **Attribute caps** - rarely changes
- **Play time** - non-gameplay info
- **DPS calculation** - derived stat
- **Update status** - non-gameplay
- **Save indicator** - non-gameplay

### Tier 4: On-Demand Only (overlays/modals)

- **Prestige confirmation dialog**
- **Achievement browser**
- **Haven tree**
- **Vault selection**
- **Game over overlays**
- **Offline welcome**
- **Debug menu**
- **Character management screens**

### Currently Unresponsive Components

The following components have **zero graceful degradation** and will break at small sizes:

1. **Stats attributes section** - rigid 6x `Length(2)` = 12 lines
2. **Equipment section** - `Min(16)` with no condensed mode
3. **Info panel** - rigid `Length(8)` with no shrink
4. **Footer** - rigid `Length(3)`, all controls on one line (will truncate)
5. **Zone info** - rigid `Length(4)`
6. **All minigame boards** - fixed pixel dimensions
7. **Dungeon map** - scales with dungeon size, can overflow
8. **Modals** - hardcoded widths (35-64 chars), only cap to screen with `min()`
9. **50/50 split** - no breakpoint to stack vertically

### Components with Some Graceful Degradation

1. **Stats header** - degrades at `inner.height < 2` (XP bar only)
2. **Prestige info** - degrades at `inner.height < 5` (fishing bar only)
3. **Combat 3D** - shows "Area too small" at `height < 3 || width < 20`
4. **Flappy Bird / Snake** - scale render buffer to available area
5. **Prestige confirm modal** - caps to `screen - 4`
6. **Leviathan / Haven modals** - cap with `.min(area.width)` / `.min(area.height)`

## responsive-ui-breakpoints.md

# Terminal Size Breakpoints and Content Priority Tiers

## Reference: Current Layout Constraints

From the [UI audit](responsive-ui-audit.md), the current layout requires:
- Stats panel: 47+ lines minimum (left 50%)
- Info panel: 8 lines (fixed)
- Footer: 3 lines (fixed)
- Zone info: 4 lines (right panel header)
- Right content: Min(10) for combat scene
- Total minimum: ~47 lines tall, ~80 columns wide

---

## Terminal Size Tiers

### Tier Definitions

| Tier | Columns | Rows | Common Scenarios |
|------|---------|------|-----------------|
| **XL** (current) | >= 120 | >= 40 | Full-screen on large monitors, typical developer terminal |
| **L** (standard) | 80-119 | 30-39 | Standard 80x24 terminal, half-screen windows |
| **M** (compact) | 60-79 | 24-29 | Small terminal window, tmux pane, laptop half-screen |
| **S** (minimal) | 40-59 | 16-23 | Very small terminal, mobile SSH, extreme split |

**Detection:** Check `frame.size()` at the start of `draw_ui_with_update()`. Width and height are independently evaluated; a terminal can be L-width but M-height.

**Note:** Below 40x16, the game should display a "Terminal too small" message.

---

## Content Priority Ranking

Every UI element is assigned a priority level. Higher priority = shown at smaller sizes.

### Priority 1: Essential (always visible, all tiers)

These elements provide the absolute minimum gameplay feedback:

| Element | Purpose | Min Size |
|---------|---------|----------|
| Player HP bar | Survival awareness | 1 line |
| Enemy HP bar / status | Combat state | 1 line |
| XP bar with level | Progression | 1 line |
| Current activity label | What's happening | 1 line |
| Quit hint | How to exit | Part of condensed footer |

### Priority 2: Core Gameplay (S tier and above)

| Element | Purpose | Min Size |
|---------|---------|----------|
| Zone / subzone name | Location | 1 line (condensed) |
| Boss progress | Zone progression | Combined with zone line |
| Loot feed (last 2-3 items) | Idle reward feedback | 3-4 lines |
| Combat log (last 2-3 entries) | Action feedback | 3-4 lines |
| Character name | Identity | Combined with level line |
| Prestige rank | Progression tier | 1 line |
| Key controls | Navigation | 1 line condensed |

### Priority 3: Important Context (M tier and above)

| Element | Purpose | Min Size |
|---------|---------|----------|
| Attributes (condensed, 2 rows of 3) | Build awareness | 2-3 lines |
| Equipment summary (names only) | Gear awareness | 7 lines |
| Prestige multiplier details | XP context | 1-2 lines |
| Fishing rank + bar | Secondary progression | 2 lines |
| Challenge banner | Notification | 1 line |
| DPS display | Combat info | Part of status line |
| Derived stats (condensed) | Combat numbers | 2-3 lines |

### Priority 4: Full Detail (L tier and above)

| Element | Purpose | Min Size |
|---------|---------|----------|
| Attributes with full layout (emojis, bars) | Detailed build | 12 lines |
| Equipment with stats + affixes | Full gear info | 16+ lines |
| Derived stats (full panel) | All combat numbers | 6 lines |
| Extended combat log (5+ entries) | Full battle history | 6+ lines |
| Extended loot feed (5+ items) | Full drop history | 6+ lines |
| Play time | Session tracking | 1 line |
| Attribute caps | Build limits | Part of attribute lines |
| Next zone status line | Forward progression | 1 line |

### Priority 5: Extra (XL tier only)

| Element | Purpose | Min Size |
|---------|---------|----------|
| Full attribute modifiers display | Min-max detail | Part of attr lines |
| Equipment affixes | Item depth | Extra lines per item |
| Update check status | Meta-game | Part of footer |
| Spacious layout | Visual comfort | Padding/margins |

---

## Tier Layouts

### XL Layout (>= 120 cols x >= 40 rows) -- NO CHANGES

This is the current layout exactly as it exists today.

```
+----------------------------------------------+
| [Challenge Banner]                            |
+----------------------+------------------------+
|  Stats Panel (50%)   | Zone Info              |
|  - Header (4h)       +------------------------+
|  - Prestige (7h)     | Combat/Activity        |
|  - Attributes (14h)  |                        |
|  - Derived (6h)      |                        |
|  - Equipment (16h+)  |                        |
+----------------------+------------------------+
| Loot (50%)           | Combat Log (50%)       | 8h
+----------------------+------------------------+
| Footer (3h)                                   |
+----------------------------------------------+
```

### L Layout (80-119 cols x 30-39 rows)

**Changes from XL:**
- Attributes condensed: 2 rows of 3 (STR/DEX/CON then INT/WIS/CHA) instead of 6 x 2-line rows -> saves 6 lines
- Derived stats condensed: 2 rows (HP/Phys/Magic on one, Def/Crit/XP on another) -> saves 3 lines
- Equipment: names only, no attr bonuses or affixes -> saves ~7 lines
- Info panel: reduce to `Length(6)` -> saves 2 lines
- Net savings: ~18 lines

```
+----------------------------------------------+
| [Challenge Banner]                            |
+----------------------+------------------------+
|  Stats Panel (50%)   | Zone Info (3h)         |
|  - Header (4h)       +------------------------+
|  - Prestige (5h)     | Combat/Activity        |
|  - Attrs compact(4h) |                        |
|  - Derived cmpct(4h) |                        |
|  - Equip names (9h)  |                        |
+----------------------+------------------------+
| Loot (50%)           | Combat Log (50%)       | 6h
+----------------------+------------------------+
| Footer (3h)                                   |
+----------------------------------------------+
```

### M Layout (60-79 cols x 24-29 rows)

**Changes from L:**
- Stats and activity panels **stacked vertically** instead of 50/50 horizontal
- Stats panel becomes a compact horizontal bar at top (2-3 lines)
- Zone info merged into stats bar
- Info panel reduced to 4 lines
- Footer condensed to 1 line (no border)
- Equipment hidden (accessible via overlay key)
- Derived stats hidden (accessible via overlay key)

```
+----------------------------------------------+
| Lv42 Warrior | P:12 Gold | Zone 5: Mountain  | 2h (stats bar)
+----------------------------------------------+
| STR:24 DEX:18 CON:21 INT:15 WIS:12 CHA:16   | 1h (attrs)
+----------------------------------------------+
|                                               |
|          Combat / Activity Area               | Min(8)
|         (full width, more vertical)           |
|                                               |
+----------------------------------------------+
| Loot (50%)           | Combat Log (50%)       | 4h
+----------------------------------------------+
| [Esc]Quit [P]Prestige [H]Haven [A]Achievem.  | 1h
+----------------------------------------------+
```

### S Layout (40-59 cols x 16-23 rows)

**Changes from M:**
- Stats bar reduced to single essential line
- Attributes hidden
- Info panel shows either loot OR combat log (tabbed), reduced to 3 lines
- Footer reduced to minimal key hints
- Activity area gets maximum space

```
+----------------------------------------------+
| Lv42 Gold P:12 | Zone 5 [Boss in 3]          | 1h
+----------------------------------------------+
| HP [========    ] 156/200                     | 1h
+----------------------------------------------+
|                                               |
|          Combat / Activity Area               | Min(6)
|         (full width, maximum space)           |
|                                               |
+----------------------------------------------+
| Foe HP [======= ] 89/120                     | 1h
+----------------------------------------------+
| [Epic] Darksteel Blade equipped!              | 3h (loot OR log)
| You deal 45 damage (CRIT!)                   |
| Enemy deals 12 damage                        |
+----------------------------------------------+
| Esc:Quit P:Prestige Tab:More                  | 1h
+----------------------------------------------+
```

---

## Breakpoint Decision Matrix

For each component, which tiers show it:

| Component | XL | L | M | S | Notes |
|-----------|:--:|:-:|:-:|:-:|-------|
| Player HP bar | Y | Y | Y | Y | Always |
| Enemy HP bar | Y | Y | Y | Y | Always |
| XP bar + level | Y | Y | Y | Y | Condensed at S |
| Zone name + boss | Y | Y | Y | Y | Condensed at S |
| Character name | Y | Y | Y | N | Merged into stats bar at M |
| Prestige rank | Y | Y | Y | Y | Condensed |
| Prestige multiplier detail | Y | Y | N | N | |
| Attributes (full 6x2) | Y | N | N | N | |
| Attributes (compact 2x3) | N | Y | N | N | |
| Attributes (single line) | N | N | Y | N | |
| Derived stats (full) | Y | N | N | N | |
| Derived stats (compact) | N | Y | N | N | |
| Equipment (full) | Y | N | N | N | |
| Equipment (names only) | N | Y | N | N | |
| Fishing rank + bar | Y | Y | N | N | Overlay access at M/S |
| Combat log (full) | Y | Y | Y | Y* | *Merged with loot at S |
| Loot panel (full) | Y | Y | Y | Y* | *Merged with combat at S |
| Info panel border/chrome | Y | Y | Y | N | |
| Footer (full 3h) | Y | Y | N | N | |
| Footer (compact 1h) | N | N | Y | Y | |
| Challenge banner | Y | Y | Y | N | |
| Play time | Y | Y | N | N | |
| DPS display | Y | Y | Y | N | |
| Update status | Y | Y | N | N | |
| Save indicator | Y | Y | N | N | |
| Spacious padding | Y | N | N | N | |

---

## Minigame Breakpoints

Minigames present a special challenge because their boards have fixed pixel dimensions.

| Minigame | Min Width | Min Height | Strategy at Small Sizes |
|----------|-----------|-----------|------------------------|
| Chess | 43 + 22 = 65w | 21h | Board cannot shrink; hide info panel below M |
| Go | 25 + 24 = 49w | 13h | Board fits at M; info panel collapses |
| Morris | 25 + 24 = 49w | 15h | Board fits at M; info panel collapses |
| Gomoku | 29 + 22 = 51w | 17h | Board fits at M; info panel collapses |
| Minesweeper | 18-60 + 24w | 11-18h | Varies by difficulty; some fit at M |
| Rune | Variable + 22w | 8h+ | Fits at most sizes |
| Flappy Bird | Scalable | Scalable | Already scales; works at M |
| Snake | Scalable | Scalable | Already scales; works at M |

**Strategy:** At M and below, minigame info panels move below the board or are hidden, giving the board full width. At S, chess is the only minigame that truly cannot fit and should display a "Terminal too small for this challenge" message.

---

## Implementation Constants

Proposed constants for `src/core/constants.rs` or a new `src/ui/responsive.rs`:

```rust
// Terminal size tier thresholds
pub const TIER_XL_MIN_COLS: u16 = 120;
pub const TIER_XL_MIN_ROWS: u16 = 40;
pub const TIER_L_MIN_COLS: u16 = 80;
pub const TIER_L_MIN_ROWS: u16 = 30;
pub const TIER_M_MIN_COLS: u16 = 60;
pub const TIER_M_MIN_ROWS: u16 = 24;
pub const TIER_S_MIN_COLS: u16 = 40;
pub const TIER_S_MIN_ROWS: u16 = 16;

// Absolute minimum (show "Terminal too small" below this)
pub const MIN_USABLE_COLS: u16 = 40;
pub const MIN_USABLE_ROWS: u16 = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TerminalTier {
    TooSmall,
    S,      // 40x16+
    M,      // 60x24+
    L,      // 80x30+
    XL,     // 120x40+
}

impl TerminalTier {
    pub fn from_size(cols: u16, rows: u16) -> Self {
        if cols < MIN_USABLE_COLS || rows < MIN_USABLE_ROWS {
            TerminalTier::TooSmall
        } else if cols >= TIER_XL_MIN_COLS && rows >= TIER_XL_MIN_ROWS {
            TerminalTier::XL
        } else if cols >= TIER_L_MIN_COLS && rows >= TIER_L_MIN_ROWS {
            TerminalTier::L
        } else if cols >= TIER_M_MIN_COLS && rows >= TIER_M_MIN_ROWS {
            TerminalTier::M
        } else {
            TerminalTier::S
        }
    }
}
```

---

## Migration Strategy

The breakpoint system should be implemented incrementally:

1. **Phase 1:** Add `TerminalTier` detection, pass tier to all draw functions. No behavior changes.
2. **Phase 2:** Implement L tier (condensed stats) -- minimal visual change, most compatible.
3. **Phase 3:** Implement M tier (stacked layout) -- significant layout change.
4. **Phase 4:** Implement S tier (minimal layout) -- maximum information density.

Each phase should be a separate PR to keep changes reviewable.

## responsive-ui-game-priorities.md

# Responsive UI: Game Information Hierarchy

This document defines what information matters at each terminal size, which elements
can be hidden or simplified, and minimum size requirements for interactive content.

---

## 1. Information Priority Tiers

### Tier 1: Critical (must show at ANY size)

These elements answer the core idle-game question: **"Am I progressing?"**

| Element | Current Source | Why Critical |
|---------|---------------|-------------|
| Character name + level | `stats_panel::draw_header` | Identity and progress anchor |
| XP bar (ratio or %) | `stats_panel::draw_header` | Primary progress indicator |
| Player HP / Enemy HP | `combat_scene` | Are we alive? Are we winning? |
| Combat status (fighting/regen/idle) | `combat_scene::draw_combat_status` | What is happening right now |
| Current zone + subzone | `stats_panel::draw_zone_info` | Where are we |
| Footer hotkeys ([Esc], [P], [Tab]) | `stats_panel::draw_footer` | How to interact |

**Minimum viable display (~40 cols x 12 rows):**
```
Hero (Lv.15 Warrior)  XP: 73%
Zone 3: Mountain Pass (2/3)
HP: ████████░░ 80/100
Foe: ██░░░░░░░ 25/90
[fighting] DPS: 42
[Esc] [P] Prestige [Tab] Challenges
```

### Tier 2: Important (show at small+, ~60x20)

These elements significantly enhance gameplay understanding.

| Element | Current Source | Why Important |
|---------|---------------|-------------|
| Prestige rank + tier name | `stats_panel::draw_prestige_info` | Key progression metric |
| Prestige XP multiplier | `stats_panel::draw_prestige_info` | Explains progression speed |
| Boss progress (kills until boss) | `stats_panel::draw_zone_info` | Near-term goal |
| Combat log (last 2-3 entries) | `info_panel::draw_combat_log` | Feedback on combat events |
| Loot panel (last 1-2 drops) | `info_panel::draw_recent_gains` | Reward feedback |
| Fishing rank (if fishing) | `stats_panel::draw_prestige_info` | Fishing progress |
| Dungeon status (if in dungeon) | `dungeon_map::DungeonStatusWidget` | Dungeon progress |
| Enemy sprite (simplified) | `combat_3d::render_combat_3d` | Visual engagement |
| Next zone hint | `stats_panel::draw_zone_info` | Long-term goal |

### Tier 3: Nice-to-have (show at medium+, ~100x30)

These add depth but a player can survive without them.

| Element | Current Source | Why Nice |
|---------|---------------|---------|
| All 6 attributes with values | `stats_panel::draw_attributes` | Detailed build info |
| Derived stats (all 6) | `stats_panel::draw_derived_stats` | Combat math |
| Equipment list (names + rarity) | `stats_panel::draw_equipment_section` | Gear overview |
| Full combat log (6+ entries) | `info_panel::draw_combat_log` | Detailed combat history |
| Full loot panel (5+ drops) | `info_panel::draw_recent_gains` | Drop history |
| Play time | `stats_panel::draw_header` | Session tracking |
| Fishing progress bar | `stats_panel::draw_prestige_info` | Granular fishing progress |
| DPS calculation | `combat_scene::draw_combat_status` | Optimization info |
| Attack timers (You/Foe) | `combat_scene::draw_combat_status` | Combat timing |
| Challenge banner | `draw_challenge_banner` | Discovery notification |
| Dungeon minimap (small) | `dungeon_map::DungeonMapWidget` | Spatial awareness |

### Tier 4: Full Detail (large only, ~120x40+)

The current full layout, as designed.

| Element | Current Source | Notes |
|---------|---------------|-------|
| Equipment with attribute bonuses | `stats_panel::draw_equipment_section` | Lines 2-3 per item |
| Equipment affixes | `stats_panel::draw_equipment_section` | Detailed item stats |
| Attribute progress bars + caps | `stats_panel::draw_attribute_row` | Build ceiling info |
| Full dungeon map | `dungeon_map::DungeonMapWidget` | Large maps (9x9+) |
| 3D ASCII combat rendering | `combat_3d::render_combat_3d` | Atmospheric visuals |
| Update drawer (changelog) | `stats_panel::draw_update_drawer` | Version info |
| Prestige multiplier breakdown | `stats_panel::draw_prestige_info` | CHA bonus detail |
| Prestige reset count | `stats_panel::draw_prestige_info` | Historical stat |

---

## 2. Current Layout Dimensions

The current layout requires approximately **100 cols x 40 rows** minimum:

```
Stats panel: 50% width (~50 cols)
  - Header:      4 rows
  - Prestige:    7 rows
  - Attributes: 14 rows
  - Derived:     6 rows
  - Equipment:  16+ rows

Right panel: 50% width (~50 cols)
  - Zone info:   4 rows
  - Combat/game: remaining (10+ rows)

Info panel: full width, 8 rows
  - Loot (50%) + Combat log (50%)

Footer: 3 rows
```

---

## 3. Minigame Size Requirements

Each minigame has hard minimum dimensions below which it cannot function.
The "content area" is the right panel minus zone info (4 rows) and borders.

### Board-based Games (need visual grid)

| Minigame | Board Size | Min Content WxH | Can Simplify? |
|----------|-----------|-----------------|--------------|
| Chess | 8x8 (5-char cells) | 44 x 21 | No - board is essential |
| Go | 9x9 (3-char cells) | 27 x 13 | No - board is essential |
| Gomoku | 15x15 (2-char cells) | 31 x 17 | Partial - could use scrolling viewport |
| Morris | 24 positions (fixed layout) | 27 x 15 | No - spatial layout is core mechanic |
| Minesweeper (Novice) | 9x9 (2-char cells) | 22 x 12 | No - grid is essential |
| Minesweeper (Master) | 20x16 (2-char cells) | 44 x 19 | Difficulty-dependent |
| Flappy Bird | 50x18 viewport | 54 x 21 | Could scale viewport |
| Snake | 26x26 (2-col cells) | 56 x 15 | Could scale grid |

### Non-visual Games (text-based interaction)

| Minigame | Min Content WxH | Can Simplify? |
|----------|-----------------|--------------|
| Rune Deciphering | 24 x 8 | Yes - already compact |

### Minigame Layout Overhead

Each minigame using `create_game_layout()` adds:
- Outer border: 2 cols + 2 rows
- Info panel: 22-24 cols on right side
- Status bar: 2 rows at bottom

So total overhead is ~26 cols wide, ~4 rows tall beyond the content area.

### Minimum Terminal Sizes for Minigames

| Minigame | Min Terminal Width | Min Terminal Height | Notes |
|----------|-------------------|--------------------|----|
| Chess | 120 (50 stats + 70 game) | 35 | Needs full board + info panel |
| Go | 100 (50 stats + 50 game) | 28 | 9x9 board is relatively compact |
| Gomoku | 110 (50 stats + 60 game) | 32 | 15x15 board is large |
| Morris | 100 (50 stats + 50 game) | 30 | Fixed position layout |
| Minesweeper | 96-120 (varies by diff) | 25-32 | Scales with difficulty |
| Rune | 96 (50 stats + 46 game) | 22 | Most compact minigame |
| Flappy Bird | 130 (50 stats + 80 game) | 35 | Wide viewport |
| Snake | 130 (50 stats + 80 game) | 35 | Large grid |

---

## 4. Size-Dependent Behavior Recommendations

### When terminal is too small for active minigame

If a minigame is active but the terminal is too small to render it:

1. **Show a "resize needed" message** with the minimum dimensions required
2. **Do NOT auto-forfeit** - the game should pause gracefully
3. **Keep game state** - resizing up should resume display immediately
4. **Show compact status** in place of the game board:
   ```
   Chess in progress (your move)
   Terminal too small: need 120x35, have 80x24
   [Esc] Forfeit
   ```

### Adaptive stats panel (left side)

| Terminal Height | Stats Panel Shows |
|----------------|------------------|
| < 15 rows | Name + level + XP bar only |
| 15-24 rows | + Prestige rank, + Zone info (moved to stats) |
| 25-34 rows | + Condensed attributes (2-col), + Condensed equipment |
| 35-44 rows | + Derived stats |
| 45+ rows | Full current layout |

| Terminal Width | Panel Split |
|---------------|-------------|
| < 60 cols | Single column, stacked layout |
| 60-79 cols | 40/60 split (narrower stats) |
| 80-99 cols | 50/50 split (current) |
| 100-119 cols | 50/50 split |
| 120+ cols | 50/50 or 40/60 (wider game area) |

### Adaptive info panel (bottom)

| Terminal Height | Info Panel |
|----------------|-----------|
| < 20 rows | Hidden entirely |
| 20-29 rows | 3 rows: single-line loot + combat log side by side |
| 30-39 rows | 5 rows: compact loot + combat |
| 40+ rows | 8 rows: full current layout |

### Adaptive footer

| Terminal Width | Footer Shows |
|---------------|-------------|
| < 40 cols | [Esc] [P] only |
| 40-59 cols | + [Tab] Challenges |
| 60-79 cols | + [H] Haven, [A] Achievements |
| 80+ cols | Full current footer |

---

## 5. Condensed Display Variants

### Condensed Attributes (for medium terminals)

Current: 14 rows (6 attrs x 2 rows + borders)

Condensed 2-column (6 rows):
```
Attributes
STR: 18 (+4)  INT: 12 (+1)
DEX: 15 (+2)  WIS: 14 (+2)
CON: 16 (+3)  CHA: 11 (+0)
```

### Condensed Equipment (for medium terminals)

Current: 16+ rows (1-3 lines per item)

Condensed (9 rows):
```
Equipment
Weapon: Iron Sword [Rare]
 Armor: Steel Plate [Epic]
Helmet: Leather Cap [Common]
Gloves: [Empty]
 Boots: Swift Sandals [Magic]
Amulet: [Empty]
  Ring: Emerald Band [Rare]
```

### Condensed Derived Stats (for medium terminals)

Current: 6 rows

Inline (2 rows):
```
HP:150 Phys:24 Mag:18 Def:12
Crit:8% XP:1.50x DPS:42
```

### Condensed Combat Scene (for small terminals)

Current: Player HP + 3D sprite + Enemy HP + Status (8+ rows)

Compact (4 rows):
```
You: ████████░░ 80/100
Foe: ██░░░░░░░ 25/90 (Goblin)
In Combat | You: 0.8s  Foe: 1.2s
DPS: 42 | Boss in 3 kills
```

### Condensed Zone Info (for very small terminals)

Current: 4 rows with borders

Inline (1 row):
```
Zone 3: Mountain Pass (2/3) [Boss in 3 kills]
```

---

## 6. Minigame Simplification Analysis

### Games that could have simplified/text-only versions

**Rune Deciphering**: Already nearly text-only. Could render in as little as
24x8 characters. Minimal adaptation needed.

**Minesweeper**: The grid is essential but could use single-character cells
instead of 2-char cells at small sizes, halving width requirements.

**Chess**: Could theoretically use a compact notation view showing the position
as FEN or a tiny 16x8 board (2-char cells), but this degrades the experience
significantly. Better to require minimum size.

### Games that cannot work small

**Go**: The 9x9 board with stone placement and territory visualization needs
spatial rendering. A text-only version would be unplayable.

**Gomoku**: 15x15 board is inherently large. Even with 1-char cells it needs
15x15 minimum. Could add viewport scrolling but pattern recognition across
the full board is core gameplay.

**Morris**: The board positions have specific spatial relationships (lines forming
mills). A non-graphical version would require the player to memorize positions
by number, making it unplayable in practice.

**Flappy Bird**: Real-time visual game. Cannot function without visual rendering.
Could potentially scale the viewport proportionally.

**Snake**: Real-time visual game. Could scale the grid down but needs a minimum
visual area. A 13x13 grid (half size) could work as a "compact mode."

### Recommendation

For terminals too small for a minigame:
- Do not offer simplified versions (the gameplay suffers too much)
- Instead, show a clear minimum size requirement before starting
- Allow the challenge to remain pending until the player resizes
- The menu itself (`challenge_menu_scene`) can render at any size since
  it is just a list

---

## 7. Idle-Specific Design Considerations

### The idle game paradox

This game auto-plays. At very small terminal sizes, the player's primary need is
passive monitoring: "Is my character progressing?" This means:

1. **XP bar is king** - the single most important visual element
2. **Level-up notifications** matter more than combat details
3. **Item drops** should be visible (even as single-line notifications)
4. **Zone advancement** is the big milestone indicator

### What players do at each engagement level

| Engagement | What they need to see | Terminal size |
|-----------|----------------------|--------------|
| Glance (1 sec) | Level, XP%, zone | Any size |
| Check-in (10 sec) | + HP, combat status, recent drops | Small |
| Active monitoring (1 min) | + Equipment, attributes, full logs | Medium |
| Active play (minigame) | Full minigame board + game info | Large |
| Deep review (haven/achievements) | Full overlay screens | Large |

### Auto-save indicator

At all sizes, if an autosave just occurred, a brief flash or indicator should
be visible so the player knows their progress is preserved. This is especially
important at small sizes where the player may be running the game in a small
tmux pane.

---

## 8. Summary: Size Tier Definitions

| Tier | Min Size | Layout Strategy |
|------|----------|----------------|
| Micro | 40x12 | Single column: name+XP, zone, HP bars, footer |
| Small | 60x20 | Single column: + prestige, compact combat, compact logs |
| Medium | 80x30 | Two columns: condensed stats + combat scene, compact logs |
| Large | 100x40 | Two columns: full stats + full combat, full logs (current layout) |
| XLarge | 120x45+ | Current layout with extra space for minigames |

The responsive system should detect terminal size on each frame and select
the appropriate tier, gracefully degrading or upgrading as the terminal
is resized.

## responsive-ui-wireframes.md

# Responsive UI Wireframes

Detailed ASCII wireframes for each terminal size tier, showing exactly how
the UI adapts. Synthesizes the [UI audit](responsive-ui-audit.md),
[breakpoints design](responsive-ui-breakpoints.md), and
[game information hierarchy](responsive-ui-game-priorities.md).

---

## Tier Overview

| Tier | Width | Height | Strategy |
|------|-------|--------|----------|
| XL | >= 120 | >= 40 | Current layout unchanged |
| L | 80-119 | 30-39 | Condensed stats, compact info panel |
| M | 60-79 | 24-29 | Stacked single-column layout |
| S | 40-59 | 16-23 | Minimal text-only layout |
| Too Small | < 40 | < 16 | "Resize terminal" message |

Width and height tiers are evaluated independently. A 100x22 terminal
would use L-width layout but M-height content density.

---

## 1. XL Layout (>= 120 cols x >= 40 rows)

**No changes from current layout.** This is the reference design.

```
120 cols
+----------------------------------------------------------+
| Challenge Banner (pending challenges notification)        | 1
+----------------------------+-----------------------------+
| Stats Panel (50%)          | Zone Info                    | 4
| +------------------------+ | Zone 3: Mountain Pass (2/3)  |
| | Hero (Lv.42 Warrior)   | | [Boss in 3 kills]           |
| | XP: ████████░░ 73.2%   | | Next: Zone 4 (unlocked)     |
| +------------------------+ +-----------------------------+
| | Prestige               | | Combat Scene                 |
| | Rank: 12 (Gold)        | |                              |
| | Mult: 2.50x + 0.30x   | |      /\                      |
| | Resets: 5              | |     /@@\                     |
| | Fishing: Expert (22)   | |    |@@@@|                    |
| | [fish progress bar]    | |    | o  o |                  |
| +------------------------+ |    | _/\_ |                  |
| | Attributes             | |    \------/                   |
| | STR: 24 (+7) [Cap:35]  | |     ||  ||                   |
| | DEX: 18 (+4) [Cap:35]  | |                              |
| | CON: 21 (+5) [Cap:35]  | | Player HP: ████████░░ 80%    |
| | INT: 15 (+2) [Cap:35]  | | Goblin:    ██░░░░░░ 25/90    |
| | WIS: 12 (+1) [Cap:35]  | | In Combat | You: 0.8s        |
| | CHA: 16 (+3) [Cap:35]  | |   Foe: 1.2s | DPS: 42        |
| +------------------------+ +-----------------------------+
| | Derived Stats          |
| | Max HP: 150            |
| | Physical: 24           |
| | Magic: 18              |
| | Defense: 12             |
| | Crit: 8%               |
| | XP Mult: 1.50x         |
| +------------------------+
| | Equipment              |
| | Weapon: Iron Sword     |
| |   +4 STR +2 DEX       |
| |   +5% DMG +3% Crit    |
| | Armor: Steel Plate     |
| |   +3 CON +2 STR       |
| | (... 5 more slots)     |
| +------------------------+
+----------------------------+-----------------------------+
| Loot Panel (50%)           | Combat Log (50%)            | 8
| [Rare] Darksteel Blade     | You deal 45 damage (CRIT!)  |
|   +5 STR +3 DEX  Equipped! | Goblin deals 12 damage      |
| [Common] Leather Cap       | You deal 23 damage           |
|                             | Goblin deals 15 damage       |
+----------------------------+-----------------------------+
| [Esc] Quit  [P] Prestige  [H] Haven  [A] Achievements   | 3
| [Tab] Challenges (2)  [U] Update (v1.5)                  |
| v2024-01-15 (abc123)                                      |
+----------------------------------------------------------+
```

**Total: ~47 rows used (fits in 40+ with some compression)**

---

## 2. L Layout (80-119 cols x 30-39 rows)

**Changes from XL:**
- Attributes condensed to 2-column format (6 rows -> 3 rows + border)
- Derived stats condensed to 2 lines
- Equipment shows names + rarity only (no attr bonuses/affixes)
- Info panel reduced from 8h to 6h
- Prestige section condensed (remove CHA breakdown, keep effective mult)

```
100 cols
+-------------------------------------------------------+
| [Challenge Banner]                                     | 1
+---------------------------+---------------------------+
| Stats Panel (50%)         | Zone Info                  | 3
| +-----------------------+ | Zone 3: Mountain (2/3)     |
| | Hero Lv.42 | 2h 15m   | | [Boss in 3 kills]          |
| | XP: ████████░░ 73.2%  | +---------------------------+
| +-----------------------+ | Combat Scene               |
| | P:12 Gold | 2.80x XP  | |                            |
| | Fish: Expert (22)      | |      /\                    |
| | [fish progress bar]    | |     /@@\                   |
| +-----------------------+ |    |@@@@|                   |
| | Attributes             | |    | o  o |                |
| | STR:24(+7) INT:15(+2) | |    | _/\_ |                |
| | DEX:18(+4) WIS:12(+1) | |    \------/                |
| | CON:21(+5) CHA:16(+3) | |     ||  ||                 |
| +-----------------------+ |                            |
| | HP:150 Phys:24 Mag:18 | | Player HP: ████████░░ 80%  |
| | Def:12 Crt:8% XP:1.5x | | Goblin:    ██░░░░░░ 25/90 |
| +-----------------------+ | In Combat | DPS: 42        |
| | Equipment              | +---------------------------+
| | Weapon [Rare] Iron Sw  |
| | Armor [Epic] Steel Pl  |
| | Helmet [Com] Leather   |
| | Gloves [Empty]         |
| | Boots [Mag] Swift San  |
| | Amulet [Empty]         |
| | Ring [Rare] Emerald    |
| +-----------------------+
+---------------------------+---------------------------+
| Loot (50%)                | Combat Log (50%)           | 6
| [Rare] Darksteel Blade    | You deal 45 damage (CRIT!) |
|   Equipped!                | Goblin deals 12 damage     |
| [Common] Leather Cap      | You deal 23 damage          |
+---------------------------+---------------------------+
| [Esc]Quit [P]Prestige [H]Haven [A]Ach [Tab]Chall(2)   | 3
| v2024-01-15 (abc123)                    Up to date     |
+-------------------------------------------------------+
```

**Total: ~33 rows (fits in 30-39)**

---

## 3. M Layout (60-79 cols x 24-29 rows)

**Major changes from L:**
- Single-column stacked layout (no 50/50 horizontal split)
- Stats become a compact header bar (2-3 lines)
- Attributes condensed to single line
- Equipment hidden (accessible via [E] key overlay)
- Derived stats hidden
- Footer condensed to 1 line (no border)
- Info panel reduced to 4 lines, full width
- Combat scene gets full width

```
70 cols
+--------------------------------------------------------------------+
| Hero Lv.42 | P:12 Gold 2.80x | Zone 3: Mountain (2/3)             | 1
+--------------------------------------------------------------------+
| STR:24 DEX:18 CON:21 INT:15 WIS:12 CHA:16     XP: ████░░ 73%     | 1
+--------------------------------------------------------------------+
|                                                                     |
|                        /\                                           |
|                       /@@\                                          |
|                      |@@@@|                                         |
|                      | o  o |                                       |
|                      | _/\_ |                                       |
|                      \------/                                       |
|                       ||  ||                                        |
|                                                                     |
|              Player HP: ████████████░░░░ 80%                        | Min(10)
|              Goblin:    ██████░░░░░░░░░░ 28%                        |
|                  In Combat | You: 0.8s  Foe: 1.2s | DPS: 42        |
|                         [Boss in 3 kills]                           |
+--------------------------------------------------------------------+
| [Rare] Darksteel Blade equipped!   You deal 45 damage (CRIT!)      | 4
| [Common] Leather Cap               Goblin deals 12 damage          |
|                                     You deal 23 damage              |
+--------------------------------------------------------------------+
| [Esc]Quit [P]Prestige [H]Haven [A]Ach [Tab]Chall [E]Equip          | 1
+--------------------------------------------------------------------+
```

**Total: ~24 rows (fits in 24-29)**

**Note:** At M width, the info panel merges loot and combat log into a
single full-width area with loot on the left and combat on the right,
similar to L but without borders.

---

## 4. S Layout (40-59 cols x 16-23 rows)

**Major changes from M:**
- Ultra-compact: single merged status line at top
- No attributes display
- Combat scene minimal: HP bars only, no sprite
- Loot and combat log merge into single scrolling feed
- Footer is minimal key hints
- No borders around sections

```
50 cols
Hero Lv.42 P:12 Gold  Zone 3: Mountain  1
XP: ████████████████░░░░░░ 73.2%         1
You: ████████████░░░░ 80/100 HP          1
Foe: ██████░░░░░░░░░░ 25/90 Goblin       1
In Combat | You: 0.8s | DPS: 42          1
[Boss in 3 kills]                        1
                                          |
         (empty space for activity)       | Min(4)
                                          |
[Rare] Darksteel Blade equipped!          |
You deal 45 damage (CRIT!)               | 5
Goblin deals 12 damage                    | (merged
You deal 23 damage                        |  feed)
Goblin deals 15 damage                    |
                                          |
Esc:Quit P:Prestige Tab:More              1
```

**Total: ~16 rows (fits in 16-23)**

**Key design decisions for S:**
- No borders or chrome anywhere (every row counts)
- HP bars use full-width gauge for readability
- Activity feed interleaves loot and combat entries chronologically
- Tab opens a quick menu for Haven/Achievements/Equipment
- Zone info merged into top status line

---

## 5. Too Small (< 40 cols or < 16 rows)

```
+--------------------------------------+
|                                       |
|    Terminal too small for Quest       |
|                                       |
|    Current: 35 x 12                   |
|    Minimum: 40 x 16                   |
|                                       |
|    Please resize your terminal.       |
|                                       |
+--------------------------------------+
```

---

## 6. Activity-Specific Wireframes

### 6a. Fishing Scene by Tier

**XL/L:** Current layout (fits in right panel)

**M:** Full-width fishing scene
```
70 cols
+--------------------------------------------------------------------+
| Hero Lv.42 | P:12 Gold | Zone 3: Mountain     Fish: Expert (22)   | 1
+--------------------------------------------------------------------+
| FISHING - Crystal Lake                                              | 1
|                                                                     |
|     ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~                                     |
|       ~~~~~~ O ~~~~~~                                               |
|     ~ ~ ~ ~ ~|~ ~ ~ ~ ~ ~ ~                                       | Min(6)
|              |                                                      |
|                                                                     |
| Caught: 3/8 fish      Waiting for bite...                          | 2
| Rank: Expert (22)     [████████░░] 14/20                           | 2
+--------------------------------------------------------------------+
| [Rare] Starfish +180 XP            Reeling in...                    | 4
| [Common] Trout +65 XP                                               |
+--------------------------------------------------------------------+
| Esc:Quit                                                             | 1
+--------------------------------------------------------------------+
```

**S:** Compact fishing
```
50 cols
Fish: Expert (22) Crystal Lake         1
Caught: 3/8 | Waiting for bite...       1
~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~             |
   ~~~~~~ O ~~~~~~                      | Min(4)
~ ~ ~ ~ ~|~ ~ ~ ~ ~ ~ ~               |
         |                              |
Rank: [████████░░] 14/20               1
[Rare] Starfish +180 XP                4
[Common] Trout +65 XP
Esc:Quit                                1
```

### 6b. Dungeon View by Tier

**XL/L:** Current layout (map + combat in right panel)

**M:** Full-width, map on top, combat below
```
70 cols
+--------------------------------------------------------------------+
| Hero Lv.42 | P:12 | Medium Dungeon | Rooms: 8/25 | [KEY]          | 1
+--------------------------------------------------------------------+
|                                                                     |
|        [Dungeon Map - centered, grid_size * 4 wide]                 |
|        [emoji rooms with corridors]                                 | map_h
|                                                                     |
+--------------------------------------------------------------------+
| You: ████████░░ 80/100    Skeleton: ██░░░░ 25/90                   | 2
| In Combat | DPS: 42                                                 |
+--------------------------------------------------------------------+
| You deal 45 damage           Skeleton deals 12 damage               | 3
+--------------------------------------------------------------------+
| Esc:Quit  Arrow:Move  Enter:Clear room                              | 1
+--------------------------------------------------------------------+
```

**S:** Map hidden, text-only dungeon progress
```
50 cols
Dungeon (Med) Rooms:8/25 [KEY]          1
Skeleton Room | You: 80/100 HP          1
Foe: 25/90 | Combat | DPS: 42          1
                                         |
You deal 45 damage (CRIT!)              | Min(4)
Skeleton deals 12 damage                |
                                         |
Esc:Quit Arrow:Move                      1
```

### 6c. Minigame Scenes by Tier

**XL:** Current layout (stats left, game right with info panel)

**L:** Stats panel narrower or hidden, game gets more space
```
100 cols
+-------------------------------------------------------+
| Hero Lv.42 P:12 Gold | Chess (Apprentice)             | 1
+-------------------------------------------------------+
| Moves: 4.Ba4 Nf6  3.Bb5 a6  2.Nf3 Nc6  1.e4 e5      | 1
|                                                         |
|   +----+----+----+----+----+----+----+----+             |
| 8 | R  | N  | B  | Q  | K  | B  | N  | R  |  Info     |
|   +----+----+----+----+----+----+----+----+  RULES     |
| 7 | P  | P  | P  | P  | P  | P  | P  | P  |  Check    |
|   +----+----+----+----+----+----+----+----+  mate the  | 18
| 6 |    |    |    |    |    |    |    |    |  enemy      |
|   +----+----+----+----+----+----+----+----+  king.     |
| 5 |    |    |    |    |    |    |    |    |             |
|   +----+----+----+----+----+----+----+----+  You: KQRB |
| 4 |    |    |    |    |    |    |    |    |  Foe: KQRB  |
|   +----+----+----+----+----+----+----+----+             |
| 3 |    |    |    |    |    |    |    |    |             |
|   +----+----+----+----+----+----+----+----+             |
| 2 | p  | p  | p  | p  | p  | p  | p  | p  |           |
|   +----+----+----+----+----+----+----+----+             |
| 1 | r  | n  | b  | q  | k  | b  | n  | r  |           |
|   +----+----+----+----+----+----+----+----+             |
|     A    B    C    D    E    F    G    H                 |
|                                                         |
| Your move                                               | 2
| [Arrows] Move  [Enter] Select  [Esc] Forfeit            |
+-------------------------------------------------------+
```

**M:** Stats panel hidden during minigame, game gets full width
```
70 cols
+--------------------------------------------------------------------+
| Chess (Apprentice) | Hero Lv.42                                     | 1
+--------------------------------------------------------------------+
| [Chess board at full width, info panel below or hidden]             |
|                                                                     |
| (board renders with available space, info panel collapsed to        |
|  a single status line below the board if height is tight)           |
|                                                                     |
+--------------------------------------------------------------------+
| Your move | [Arrows] Move [Enter] Select [Esc] Forfeit              | 1
+--------------------------------------------------------------------+
```

**S:** Terminal too small message for board games
```
50 cols
+------------------------------------------------+
|                                                 |
|  Chess in progress (your move)                  |
|                                                 |
|  Terminal too small to display board.            |
|  Need: 65 x 24   Have: 50 x 18                 |
|                                                 |
|  Please resize your terminal.                   |
|                                                 |
|  [Esc] Forfeit                                  |
+------------------------------------------------+
```

**Exception:** Rune Deciphering works at M and possibly S:
```
50 cols (S tier, rune game)
Rune Deciphering (Apprentice)           1
 1: A B C D   * o . .                   |
 2: D A B C   . * o .                   |
                                         | 6+
 3: B C _ _                             |
Runes: A B C D E F                      |
                                         |
Deciphering... 4 left                    1
[<>]Move [^v]Cycle [Enter]Go [Esc]Quit  1
```

### 6d. Haven Overlay by Tier

**XL/L:** Current full-screen overlay with skill tree + detail panel

**M:** Simplified list view
```
70 cols
+--------------------------------------------------------------------+
| Haven (8/14 rooms built)                                            |
+--------------------------------------------------------------------+
| Active: +15% DMG  +10% XP  +5% Drops  +3% Crit                    | 2
+--------------------------------------------------------------------+
| > Hearthstone ★★★   +15% DMG                          Cost: --    |
|   Armory ★★·        +10% Drop Rate                    Cost: 3 PR   |
|   TrainingYard ★··   +5% Crit                         Cost: 5 PR   |
|   Watchtower ···     Locked (needs TrainingYard)                    | Min(8)
|   Bedroom ★★★       +20% XP                           Cost: --    |
|   Garden ★··         +3% Discovery                    Cost: 4 PR   |
|   (... scrollable)                                                  |
+--------------------------------------------------------------------+
| [Up/Down] Navigate  [Enter] Build  [Esc] Close                     | 1
+--------------------------------------------------------------------+
```

**S:** Compact haven status only
```
50 cols
Haven (8/14 rooms)                       1
Bonuses: +15%DMG +10%XP +5%Drop +3%Crit 1
                                          |
> Hearthstone ★★★                        |
  Armory ★★·  [3 PR to upgrade]         |
  TrainingYard ★··                       | Min(6)
  Watchtower ··· [Locked]                |
  Bedroom ★★★                           |
  Garden ★··                             |
                                          |
Up/Down:Move Enter:Build Esc:Close        1
```

### 6e. Achievement Browser by Tier

**XL/L:** Current full overlay with tabs + list + detail

**M:** Tabs + list only, detail on Enter
```
70 cols
+--------------------------------------------------------------------+
| Achievements (42.5% Complete)                                       |
+--------------------------------------------------------------------+
| Combat(3/8) Level(4/6) Progress(2/5) Challenge(1/4) Explore(0/3)  | 2
+--------------------------------------------------------------------+
| [X] First Blood - Defeat your first enemy                          |
| [X] Warrior - Defeat 100 enemies                                   |
| [ ] Champion - Defeat 1000 enemies (Progress: 456/1000)           | Min(8)
| [ ] Slayer - Defeat 5000 enemies                                   |
| > [ ] Legend - Defeat 10000 enemies                                 |
|                                                                     |
+--------------------------------------------------------------------+
| [</>] Category  [Up/Down] Select  [Enter] Detail  [Esc] Close     | 1
+--------------------------------------------------------------------+
```

---

## 7. Transition Points

### Width Transitions

| At Width | Change |
|----------|--------|
| 120 -> 119 | Enter L tier: condense attributes, equipment, derived stats |
| 80 -> 79 | Enter M tier: switch to stacked layout, hide equipment |
| 60 -> 59 | Enter S tier: remove all borders, minimal text layout |
| 40 -> 39 | Too small: show resize message |

### Height Transitions

| At Height | Change |
|-----------|--------|
| 40 -> 39 | L tier: reduce info panel 8->6, condense prestige section |
| 30 -> 29 | M tier: compact header bar, hide derived stats |
| 24 -> 23 | Aggressive M: hide attributes line |
| 16 -> 15 | Too small for gameplay |

### Hysteresis

To prevent flickering when the terminal is exactly at a breakpoint,
use a 2-unit hysteresis buffer:

```rust
// Upgrade threshold = breakpoint value
// Downgrade threshold = breakpoint value - 2
// Example: L->XL at 120 cols, XL->L at 118 cols
```

---

## 8. Overlay Adaptation

All modals (prestige confirm, haven discovery, achievement unlock, etc.)
should adapt to available space:

| Modal | XL/L Size | M Size | S Size |
|-------|-----------|--------|--------|
| Prestige Confirm | 50x18 centered | 50x18 or full-width | Full-screen |
| Achievement Unlock | 50x9 centered | 50x9 centered | Full-width, compact |
| Haven Discovery | 50x7 centered | 50x7 centered | Full-width |
| Leviathan Encounter | 64x16 centered | Full-width x 16 | Full-width, truncated |
| Offline Welcome | 44x10 centered | 44x10 centered | Full-width |

**Rule:** If a modal's hardcoded width exceeds 80% of terminal width,
render it as full-width with 1-column padding on each side.

---

## 9. Summary of Line Budgets

| Tier | Total Rows | Chrome/Fixed | Content |
|------|-----------|-------------|---------|
| XL | 40+ | ~14 (banner+info+footer+zone) | 26+ |
| L | 30-39 | ~10 (banner+info+footer) | 20-29 |
| M | 24-29 | ~4 (header+attrs+info+footer) | 20-25 |
| S | 16-23 | ~3 (status+footer) | 13-20 |
