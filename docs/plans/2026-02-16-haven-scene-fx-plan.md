# Haven Scene FX Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add subtle warm hearth glow backdrop effects to the Haven skill tree overlay using SceneCell buffer rendering.

**Architecture:** Convert `render_haven_tree` and its sub-functions (`render_summary_bar`, `render_skill_tree`, `render_room_detail`) from Ratatui widget rendering to SceneCell buffer rendering. A single buffer is created for the `inner` area, painted with a warm gradient + slow motes, then all text/borders rendered via `put_cell`/`put_text`. The outer Block border stays as a Ratatui widget. Modals are unchanged.

**Tech Stack:** Rust, Ratatui 0.30, scene_fx utilities (SceneCell, render_buffer, put_cell, hash2d, lerp_rgb, current_millis)

**Design doc:** `docs/plans/2026-02-16-haven-scene-fx-design.md`

---

### Task 1: Add scene_fx imports and text helpers

**Files:**
- Modify: `src/ui/haven_scene.rs:1-12` (imports)

**Context:** The haven_scene currently imports only Ratatui types. We need scene_fx utilities for buffer rendering, plus local `put_text` and `put_text_centered` helpers identical to those in `soulforge_scene.rs`.

**Step 1: Add scene_fx imports to haven_scene.rs**

At the top of `src/ui/haven_scene.rs`, after the existing imports, add:

```rust
use super::scene_fx::{current_millis, hash2d, lerp_rgb, put_cell, render_buffer, SceneCell};
```

Also add `#[allow(unused_imports)]` above this line temporarily (removed in final cleanup task) since the imports won't be used until later tasks.

**Step 2: Add `put_text` helper function**

After the imports section, add:

```rust
/// Write a string into the scene buffer at (row, col).
fn put_text(buffer: &mut [Vec<SceneCell>], row: i32, col: i32, text: &str, fg: Color) {
    for (i, ch) in text.chars().enumerate() {
        put_cell(buffer, row, col + i as i32, ch, fg);
    }
}
```

Add `#[allow(dead_code)]` above this function temporarily.

**Step 3: Add `paint_hearth_backdrop` function**

This is the core backdrop painter — much simpler than Soulforge's `paint_forge_backdrop` because there's only one set of parameters (no phases).

```rust
/// Paint a warm hearth glow backdrop: gentle gradient + slow-drifting motes.
fn paint_hearth_backdrop(buffer: &mut [Vec<SceneCell>], millis: u128) {
    let height = buffer.len();
    if height == 0 {
        return;
    }
    let width = buffer[0].len();

    // 1. Background gradient: near-black at top, warm amber at bottom
    let top_rgb = (10u8, 8u8, 6u8);
    let bottom_rgb = (60u8, 35u8, 15u8);
    for (row, row_cells) in buffer.iter_mut().enumerate() {
        let t = if height <= 1 {
            0.0
        } else {
            row as f64 / (height - 1) as f64
        };
        let rgb = lerp_rgb(top_rgb, bottom_rgb, t);
        let bg = Color::Rgb(rgb.0, rgb.1, rgb.2);
        for cell in row_cells.iter_mut() {
            cell.bg = bg;
        }
    }

    // 2. Slow-drifting motes (4-6 particles, ~0.3x Soulforge speed)
    let mote_chars: &[char] = &['\u{00b7}', '\u{2022}']; // · •
    let mote_count = 5;
    let mote_speed = 1.5; // pixels per second (vs Soulforge's 5.0)
    let mote_hot = (140u8, 90u8, 30u8);
    let mote_cool = (60u8, 35u8, 15u8);

    for i in 0..mote_count {
        let seed = hash2d(i, 0);
        let col = (seed as usize) % width;
        let ch = mote_chars[(hash2d(i, 1) as usize) % mote_chars.len()];

        let phase_offset = (seed as f64) * 0.73;
        let pos = (phase_offset + millis as f64 * mote_speed / 1000.0) % height as f64;
        let row = (height - 1) as f64 - pos; // upward drift

        let t = pos / height.max(1) as f64;
        let rgb = lerp_rgb(mote_hot, mote_cool, t);
        put_cell(buffer, row as i32, col as i32, ch, Color::Rgb(rgb.0, rgb.1, rgb.2));
    }

    // No shimmer — Haven is calm and still
}
```

Add `#[allow(dead_code)]` above this function temporarily.

**Step 4: Verify it compiles**

Run: `cargo build 2>&1 | tail -3`
Expected: compiles with no errors (dead_code warnings suppressed)

**Step 5: Commit**

```bash
git add src/ui/haven_scene.rs
git commit -m "feat(haven): add scene_fx infrastructure - imports, text helper, hearth backdrop"
```

---

### Task 2: Convert render_summary_bar to buffer

**Files:**
- Modify: `src/ui/haven_scene.rs` — `render_summary_bar` function

**Context:** `render_summary_bar` currently renders a single line of active bonuses as a Ratatui `Paragraph`. We need to convert it to write into a buffer. The function signature changes to accept a buffer + row offset instead of frame + area.

**Step 1: Create buffer-based `render_summary_bar_buf`**

Replace the existing `render_summary_bar` function with a buffer-based version. Keep the old signature but have it delegate to the buffer version (we'll remove the old one when `render_haven_tree` is converted in Task 4).

Add a new function:

```rust
/// Render the summary bar into a scene buffer at the given row.
fn render_summary_bar_buf(buffer: &mut [Vec<SceneCell>], row: i32, haven: &Haven) {
    let rooms_built = haven.rooms_built();
    let total_rooms = haven.total_rooms();

    let header = format!("Active bonuses ({}/{} rooms): ", rooms_built, total_rooms);
    put_text(buffer, row, 0, &header, Color::White);
    let mut col = header.chars().count() as i32;

    let bonus_types = [
        (HavenBonusType::DamagePercent, "+{}% DMG"),
        (HavenBonusType::XpGainPercent, "+{}% XP"),
        (HavenBonusType::DropRatePercent, "+{}% Drops"),
        (HavenBonusType::CritChancePercent, "+{}% Crit"),
        (HavenBonusType::HpRegenPercent, "+{}% HP Regen"),
        (HavenBonusType::DoubleStrikeChance, "+{}% Double Strike"),
        (HavenBonusType::OfflineXpPercent, "+{}% Offline XP"),
        (HavenBonusType::ChallengeDiscoveryPercent, "+{}% Discovery"),
    ];

    let mut first = true;
    for (bonus_type, fmt) in bonus_types {
        let value = haven.get_bonus(bonus_type);
        if value > 0.0 {
            if !first {
                put_text(buffer, row, col, "  ", Color::White);
                col += 2;
            }
            let text = fmt.replace("{}", &format!("{:.0}", value));
            put_text(buffer, row, col, &text, Color::Yellow);
            col += text.chars().count() as i32;
            first = false;
        }
    }

    if first {
        put_text(buffer, row, col, "None yet", Color::DarkGray);
    }
}
```

Add `#[allow(dead_code)]` temporarily.

**Step 2: Verify it compiles**

Run: `cargo build 2>&1 | tail -3`
Expected: compiles cleanly

**Step 3: Commit**

```bash
git add src/ui/haven_scene.rs
git commit -m "feat(haven): add buffer-based summary bar renderer"
```

---

### Task 3: Convert render_skill_tree to buffer

**Files:**
- Modify: `src/ui/haven_scene.rs` — `render_skill_tree` function

**Context:** `render_skill_tree` renders a bordered list of 14 rooms with tier indicators, selection arrows, and indentation. The buffer version draws box-drawing border characters via `put_cell`, then renders each room line with `put_text`.

**Step 1: Create buffer-based `render_skill_tree_buf`**

```rust
/// Render the skill tree panel into a scene buffer.
/// `left` and `top` are the top-left coordinates of this panel within the buffer.
/// `panel_width` and `panel_height` are the panel dimensions including borders.
fn render_skill_tree_buf(
    buffer: &mut [Vec<SceneCell>],
    left: i32,
    top: i32,
    panel_width: usize,
    panel_height: usize,
    haven: &Haven,
    selected_room: usize,
) {
    let border_fg = Color::DarkGray;

    // Draw border
    let right = left + panel_width as i32 - 1;
    let bottom = top + panel_height as i32 - 1;
    // Corners
    put_cell(buffer, top, left, '\u{250c}', border_fg);     // ┌
    put_cell(buffer, top, right, '\u{2510}', border_fg);     // ┐
    put_cell(buffer, bottom, left, '\u{2514}', border_fg);   // └
    put_cell(buffer, bottom, right, '\u{2518}', border_fg);  // ┘
    // Horizontal edges
    for c in (left + 1)..right {
        put_cell(buffer, top, c, '\u{2500}', border_fg);     // ─
        put_cell(buffer, bottom, c, '\u{2500}', border_fg);
    }
    // Vertical edges
    for r in (top + 1)..bottom {
        put_cell(buffer, r, left, '\u{2502}', border_fg);    // │
        put_cell(buffer, r, right, '\u{2502}', border_fg);
    }
    // Title
    let title = " Skill Tree ";
    put_text(buffer, top, left + 1, title, border_fg);

    // Render room rows inside the border
    let inner_left = left + 1;
    let content_top = top + 1;

    for (i, room) in HavenRoomId::ALL.iter().enumerate() {
        let row = content_top + i as i32;
        if row >= bottom {
            break;
        }

        let tier = haven.room_tier(*room);
        let unlocked = haven.is_room_unlocked(*room);
        let is_selected = i == selected_room;

        // Tier indicator
        let max_t = room.max_tier();
        let tier_str: String = (1..=max_t)
            .map(|t| if tier >= t { "\u{2605}" } else { "\u{00b7}" })
            .collect::<Vec<_>>()
            .join("");

        let prefix = if is_selected { "\u{25b6} " } else { "  " };

        let indent = match room {
            HavenRoomId::Hearthstone => "",
            HavenRoomId::Armory | HavenRoomId::Bedroom => "  ",
            HavenRoomId::TrainingYard
            | HavenRoomId::TrophyHall
            | HavenRoomId::Garden
            | HavenRoomId::Library => "    ",
            HavenRoomId::Watchtower
            | HavenRoomId::AlchemyLab
            | HavenRoomId::FishingDock
            | HavenRoomId::Workshop => "      ",
            HavenRoomId::WarRoom | HavenRoomId::Vault => "        ",
            HavenRoomId::StormForge => "          ",
        };

        let style_fg = if !unlocked {
            Color::DarkGray
        } else if is_selected {
            Color::Cyan
        } else if tier > 0 {
            Color::Green
        } else {
            Color::White
        };

        let tier_fg = if tier > 0 {
            Color::Yellow
        } else {
            Color::DarkGray
        };

        let lock_indicator = if !unlocked { "\u{1f512} " } else { "" };

        // Write components: prefix, tier stars, space, indent, lock, name
        let mut col = inner_left;
        put_text(buffer, row, col, prefix, style_fg);
        col += prefix.chars().count() as i32;
        put_text(buffer, row, col, &tier_str, tier_fg);
        col += tier_str.chars().count() as i32;
        put_text(buffer, row, col, " ", style_fg);
        col += 1;
        put_text(buffer, row, col, indent, style_fg);
        col += indent.chars().count() as i32;
        put_text(buffer, row, col, lock_indicator, Color::DarkGray);
        col += lock_indicator.chars().count() as i32;
        put_text(buffer, row, col, room.name(), style_fg);

        // Highlight selected row background
        if is_selected {
            let highlight_bg = Color::Rgb(30, 22, 12);
            let row_usize = row as usize;
            if row_usize < buffer.len() {
                for c in (inner_left as usize)..((right) as usize) {
                    if c < buffer[row_usize].len() {
                        buffer[row_usize][c].bg = highlight_bg;
                    }
                }
            }
        }
    }
}
```

Add `#[allow(dead_code)]` temporarily.

**Step 2: Verify it compiles**

Run: `cargo build 2>&1 | tail -3`
Expected: compiles cleanly

**Step 3: Commit**

```bash
git add src/ui/haven_scene.rs
git commit -m "feat(haven): add buffer-based skill tree renderer with warm highlight"
```

---

### Task 4: Convert render_room_detail to buffer

**Files:**
- Modify: `src/ui/haven_scene.rs` — `render_room_detail` function

**Context:** `render_room_detail` is the most complex sub-function. It renders the room description, bonus tiers, requirements, and cost info in the right panel. It currently uses `Paragraph` widgets with `Wrap` for the description.

**Step 1: Create buffer-based `render_room_detail_buf`**

```rust
/// Render the room detail panel into a scene buffer.
fn render_room_detail_buf(
    buffer: &mut [Vec<SceneCell>],
    left: i32,
    top: i32,
    panel_width: usize,
    panel_height: usize,
    haven: &Haven,
    selected_room: usize,
    prestige_rank: u32,
    achievements: &crate::achievements::Achievements,
) {
    let room = HavenRoomId::ALL[selected_room];
    let tier = haven.room_tier(room);
    let unlocked = haven.is_room_unlocked(room);

    let border_fg = if unlocked {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    // Draw border
    let right = left + panel_width as i32 - 1;
    let bottom = top + panel_height as i32 - 1;
    put_cell(buffer, top, left, '\u{250c}', border_fg);
    put_cell(buffer, top, right, '\u{2510}', border_fg);
    put_cell(buffer, bottom, left, '\u{2514}', border_fg);
    put_cell(buffer, bottom, right, '\u{2518}', border_fg);
    for c in (left + 1)..right {
        put_cell(buffer, top, c, '\u{2500}', border_fg);
        put_cell(buffer, bottom, c, '\u{2500}', border_fg);
    }
    for r in (top + 1)..bottom {
        put_cell(buffer, r, left, '\u{2502}', border_fg);
        put_cell(buffer, r, right, '\u{2502}', border_fg);
    }
    // Title
    let title = format!(" {} ", room.name());
    put_text(buffer, top, left + 1, &title, border_fg);

    let inner_left = left + 1;
    let inner_width = (panel_width - 2).max(1);
    let mut row = top + 1;

    // Description (word-wrap manually)
    let desc = room.description();
    let wrapped = word_wrap(desc, inner_width);
    for line in &wrapped {
        if row >= bottom {
            break;
        }
        put_text(buffer, row, inner_left, line, Color::White);
        row += 1;
    }

    row += 1; // spacer

    // Bonuses
    if row < bottom {
        put_text(buffer, row, inner_left, "Bonuses:", Color::White);
        row += 1;
    }
    let max_tier = room.max_tier();
    for t in 1..=max_tier {
        if row >= bottom {
            break;
        }
        let is_built = t <= tier;
        let is_next = t == tier + 1 && tier < max_tier;
        let style_fg = if is_built {
            Color::Green
        } else if is_next {
            Color::Yellow
        } else {
            Color::DarkGray
        };

        let marker = if is_next { "\u{25b6} " } else { "  " };
        let tier_label = format!("T{}: ", t);
        let bonus_text = room.format_bonus(t);

        let mut col = inner_left;
        put_text(buffer, row, col, marker, style_fg);
        col += marker.chars().count() as i32;
        put_text(buffer, row, col, &tier_label, Color::DarkGray);
        col += tier_label.chars().count() as i32;
        put_text(buffer, row, col, &bonus_text, style_fg);
        row += 1;
    }

    row += 1; // spacer

    // Requirements
    let parents = room.parents();
    if !parents.is_empty() && row < bottom {
        put_text(buffer, row, inner_left, "Requires:", Color::White);
        row += 1;

        for parent in parents {
            if row >= bottom {
                break;
            }
            let parent_tier = haven.room_tier(*parent);
            let is_built = parent_tier > 0;
            let (marker, style_fg) = if is_built {
                ("\u{2713}", Color::Green)
            } else {
                ("\u{2717}", Color::Red)
            };

            let tier_info = if parent_tier > 0 {
                format!(" (T{})", parent_tier)
            } else {
                String::new()
            };

            let mut col = inner_left;
            put_text(buffer, row, col, &format!("  {} ", marker), style_fg);
            col += 4;
            put_text(buffer, row, col, parent.name(), style_fg);
            col += parent.name().chars().count() as i32;
            put_text(buffer, row, col, &tier_info, Color::DarkGray);
            row += 1;
        }

        row += 1; // spacer
    }

    // Cost info
    if row >= bottom {
        return;
    }

    if !unlocked {
        put_text(buffer, row, inner_left, "\u{1f512} Locked", Color::Red);
        row += 1;
        if row < bottom {
            put_text(
                buffer,
                row,
                inner_left,
                "Build all required rooms first",
                Color::DarkGray,
            );
        }
    } else if tier < room.max_tier() {
        let next_tier = tier + 1;
        let cost = tier_cost(room, next_tier);
        let can_afford_it = can_afford(room, haven, prestige_rank);
        let cost_fg = if can_afford_it {
            Color::Green
        } else {
            Color::Red
        };

        let cost_text = format!("{} Prestige Ranks", cost);
        put_text(buffer, row, inner_left, "Cost: ", Color::DarkGray);
        put_text(buffer, row, inner_left + 6, &cost_text, cost_fg);
        row += 1;
        if row < bottom {
            let have_text = format!("{} Prestige Ranks", prestige_rank);
            put_text(buffer, row, inner_left, "You have: ", Color::DarkGray);
            put_text(buffer, row, inner_left + 10, &have_text, Color::White);
        }
    } else if room == HavenRoomId::StormForge {
        use crate::achievements::AchievementId;
        let has_stormbreaker = achievements.is_unlocked(AchievementId::TheStormbreaker);

        if has_stormbreaker {
            put_text(buffer, row, inner_left, "\u{26a1} Stormbreaker forged!", Color::Yellow);
            row += 1;
            if row < bottom {
                put_text(buffer, row, inner_left, "Zone 10 boss accessible", Color::Green);
            }
        } else {
            put_text(buffer, row, inner_left, "Press [Enter] to forge", Color::Yellow);
            row += 1;
            if row < bottom {
                put_text(
                    buffer,
                    row,
                    inner_left,
                    "Requires: Storm Leviathan + 25 PR",
                    Color::DarkGray,
                );
            }
        }
    } else {
        put_text(buffer, row, inner_left, "\u{2713} Max tier reached", Color::Green);
    }
}

/// Simple word-wrap: break text into lines that fit within `max_width` characters.
fn word_wrap(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.chars().count() + 1 + word.chars().count() <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    lines
}
```

Add `#[allow(dead_code)]` temporarily on both functions.

**Step 2: Verify it compiles**

Run: `cargo build 2>&1 | tail -3`
Expected: compiles cleanly

**Step 3: Commit**

```bash
git add src/ui/haven_scene.rs
git commit -m "feat(haven): add buffer-based room detail renderer with word wrap"
```

---

### Task 5: Wire up render_haven_tree to use buffer rendering

**Files:**
- Modify: `src/ui/haven_scene.rs` — `render_haven_tree` function + remove old sub-functions

**Context:** This is the integration task. Rewrite `render_haven_tree` to create a SceneCell buffer, paint the hearth backdrop, call the buffer-based sub-functions, then flush with `render_buffer`. Remove the old widget-based `render_summary_bar`, `render_skill_tree`, and `render_room_detail` functions.

**Step 1: Rewrite `render_haven_tree`**

Replace the body of `render_haven_tree` (keep the same public signature):

```rust
pub fn render_haven_tree(
    frame: &mut Frame,
    area: Rect,
    haven: &Haven,
    selected_room: usize,
    prestige_rank: u32,
    achievements: &crate::achievements::Achievements,
    _ctx: &super::responsive::LayoutContext,
) {
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Haven ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let height = inner.height as usize;
    let width = inner.width as usize;
    if height == 0 || width == 0 {
        return;
    }

    // Create scene buffer and paint hearth backdrop
    let mut buffer = vec![vec![SceneCell::default(); width]; height];
    let millis = current_millis();
    paint_hearth_backdrop(&mut buffer, millis);

    // Layout: summary (row 0-1), main content, help (last row)
    let summary_rows = 2usize;
    let help_row = (height - 1) as i32;
    let content_top = summary_rows as i32;
    let content_height = height.saturating_sub(summary_rows + 1);

    // Summary bar
    render_summary_bar_buf(&mut buffer, 0, haven);

    // Main content: skill tree (40%) on left, room detail (60%) on right
    let tree_width = (width * 40 / 100).max(10);
    let detail_left = tree_width as i32;
    let detail_width = width - tree_width;

    render_skill_tree_buf(
        &mut buffer,
        0,
        content_top,
        tree_width,
        content_height,
        haven,
        selected_room,
    );
    render_room_detail_buf(
        &mut buffer,
        detail_left,
        content_top,
        detail_width,
        content_height,
        haven,
        selected_room,
        prestige_rank,
        achievements,
    );

    // Help bar
    put_text(
        &mut buffer,
        help_row,
        0,
        "[\u{2191}/\u{2193}] Navigate  [Enter] Build/Forge  [Esc] Close",
        Color::DarkGray,
    );

    // Flush buffer to frame
    render_buffer(frame, inner, &buffer);
}
```

**Step 2: Delete old widget-based functions**

Remove the old `render_summary_bar`, `render_skill_tree`, and `render_room_detail` functions (the ones that take `frame: &mut Frame, area: Rect` parameters). Keep only the `_buf` versions.

**Step 3: Remove unused Ratatui imports**

Remove from the import block any types no longer used by the remaining functions. The `render_haven_tree` function no longer needs `Layout`, `Direction`, `Constraint`, `List`, `ListItem`, or `Wrap`. Keep `Block`, `Borders`, `Clear`, `Paragraph`, `Span`, `Line`, `Style`, `Color`, `Modifier`, `Rect`, `Alignment`, `Frame` — the modals still use them.

Check which imports are actually still needed by the modal functions (`render_haven_discovery_modal`, `render_build_confirmation`, `render_forge_confirmation`, `render_vault_selection`). Those use: `Block`, `Borders`, `Clear`, `Paragraph`, `Line`, `Span`, `Style`, `Color`, `Modifier`, `Rect`, `Alignment`, `Frame`, `Layout`, `Direction`, `Constraint`, `List`, `ListItem`.

Actually, `render_vault_selection` uses `Layout`, `Direction`, `Constraint`, `List`, and `ListItem`. So keep all Ratatui imports — the modals still use them.

**Step 4: Remove `#[allow(dead_code)]` and `#[allow(unused_imports)]` annotations**

Remove the temporary allow annotations added in Tasks 1-4.

**Step 5: Verify it compiles and tests pass**

Run: `cargo build 2>&1 | tail -3`
Run: `cargo test 2>&1 | tail -5`
Expected: compiles cleanly, all tests pass

**Step 6: Run clippy**

Run: `cargo clippy --all-targets -- -D warnings 2>&1 | tail -5`
Expected: no warnings

**Step 7: Commit**

```bash
git add src/ui/haven_scene.rs
git commit -m "feat(haven): convert haven tree view to scene_fx buffer with warm hearth backdrop"
```

---

### Task 6: Final cleanup and verification

**Files:**
- Modify: `src/ui/haven_scene.rs` — Any remaining cleanup

**Step 1: Run full CI check**

Run: `make check`
Expected: All checks pass (format, clippy, tests, build, audit)

**Step 2: Fix any issues found**

If `cargo fmt --check` fails, run `make fmt` then re-check.
If clippy warnings appear, fix them.

**Step 3: Rename `_buf` functions**

Now that the old functions are gone, rename the buffer-based functions to drop the `_buf` suffix since they're the only versions:
- `render_summary_bar_buf` → `render_summary_bar`
- `render_skill_tree_buf` → `render_skill_tree`
- `render_room_detail_buf` → `render_room_detail`

Update the call sites in `render_haven_tree` accordingly.

**Step 4: Run full CI check again**

Run: `make check`
Expected: All checks pass

**Step 5: Commit**

```bash
git add src/ui/haven_scene.rs
git commit -m "refactor(haven): rename buffer functions, final cleanup"
```
