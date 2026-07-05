> Backported implementation plan (completed — this work shipped).

## 2026-02-22-time-vault-ux-plan.md

# Time Vault UX Overhaul Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Upgrade the Time Vault overlay with an animated temporal backdrop, timeline graph, event-type icons/colors, and branch panel polish.

**Architecture:** The rendering switches from direct Ratatui widget calls to the SceneCell buffer pipeline (same pattern as `stormglass_scene.rs`). A backdrop is painted first, then panel content is rendered on top using `put_text`/`put_cell`. The controls bar stays as a regular Paragraph widget below the buffer.

**Tech Stack:** Ratatui, scene_fx.rs (SceneCell, put_text, put_cell, render_buffer, hash2d, lerp_rgb, current_millis)

---

### Task 1: Add event icon/color helper function

**Files:**
- Modify: `src/ui/time_vault_scene.rs`

**Step 1: Add the helper function**

Add a function that maps commit message prefixes to `(icon, color)` tuples. Place it after the existing `impl TimeVaultState` block (after line 86).

```rust
/// Map a commit message to an event-type icon and color.
fn event_icon_color(message: &str) -> (&'static str, Color) {
    let desc = message.split(" | ").next().unwrap_or(message);
    if desc.starts_with("Defeated") {
        ("\u{2694}", Color::LightRed)          // ⚔
    } else if desc.starts_with("Prestige") {
        ("\u{2605}", Color::Rgb(255, 215, 0))  // ★ gold
    } else if desc.starts_with("Won ") {
        ("\u{265f}", Color::Magenta)           // ♟
    } else if desc.starts_with("Completed") {
        ("\u{25c6}", Color::Green)             // ◆
    } else if desc.starts_with("Caught") || desc.starts_with("Fishing") {
        ("~", Color::Blue)
    } else if desc.starts_with("Built") || desc.starts_with("Upgraded") {
        ("\u{2302}", Color::Yellow)            // ⌂
    } else if desc.starts_with("Enhanced") {
        ("\u{2692}", Color::Cyan)              // ⚒
    } else if desc.starts_with("Achievement") {
        ("\u{2726}", Color::White)             // ✦
    } else if desc.starts_with("Chrono Surge") {
        ("\u{23e9}", Color::Cyan)              // ⏩
    } else {
        ("\u{00b7}", Color::DarkGray)          // ·
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo build 2>&1 | tail -3`
Expected: `Finished`

**Step 3: Commit**

```bash
git add src/ui/time_vault_scene.rs
git commit -m "feat(time-vault): add event icon/color helper"
```

---

### Task 2: Add backdrop constants and paint function

**Files:**
- Modify: `src/ui/time_vault_scene.rs`

**Step 1: Add imports and constants**

Add `scene_fx` imports and backdrop constants near the top of the file, after the existing `use` block (after line 13):

```rust
use super::scene_fx::{
    current_millis, hash2d, lerp_rgb, put_cell, put_text, render_buffer, SceneCell,
};

/// Temporal backdrop: dark navy top.
const VAULT_TOP_RGB: (u8, u8, u8) = (8, 12, 35);
/// Temporal backdrop: near-black bottom.
const VAULT_BOTTOM_RGB: (u8, u8, u8) = (3, 5, 15);
/// Dim cyan for timeline graph lines.
const TIMELINE_DIM: Color = Color::Rgb(40, 80, 120);
/// Number of drifting particles.
const PARTICLE_COUNT: usize = 5;
/// Particle drift speed (lower = slower).
const PARTICLE_SPEED: f64 = 0.8;
```

**Step 2: Add the backdrop paint function**

Add after the constants, before `event_icon_color`:

```rust
/// Paint the temporal vault backdrop: dark gradient with slow-drifting cyan particles.
fn paint_vault_backdrop(buffer: &mut [Vec<SceneCell>], millis: u128) {
    let height = buffer.len();
    if height == 0 {
        return;
    }
    let width = buffer[0].len();

    // 1. Background gradient (top to bottom)
    for (row, row_cells) in buffer.iter_mut().enumerate() {
        let t = if height <= 1 {
            0.0
        } else {
            row as f64 / (height - 1) as f64
        };
        let rgb = lerp_rgb(VAULT_TOP_RGB, VAULT_BOTTOM_RGB, t);
        let bg = Color::Rgb(rgb.0, rgb.1, rgb.2);
        for cell in row_cells.iter_mut() {
            cell.bg = bg;
        }
    }

    // 2. Subtle particles drifting downward
    let particle_chars: &[char] = &['\u{00b7}', '\u{2022}', '\u{2726}'];
    let particle_hot: (u8, u8, u8) = (80, 160, 220);
    let particle_cool: (u8, u8, u8) = (20, 40, 80);
    for i in 0..PARTICLE_COUNT {
        let seed = hash2d(i, 0);
        let col = (seed as usize) % width;
        let ch = particle_chars[(hash2d(i, 1) as usize) % particle_chars.len()];

        let phase_offset = (seed as f64) * 0.73;
        let pos = (phase_offset + millis as f64 * PARTICLE_SPEED / 1000.0) % height as f64;
        let row = pos as i32;

        let t = pos / height.max(1) as f64;
        let rgb = lerp_rgb(particle_hot, particle_cool, t);
        put_cell(buffer, row, col as i32, ch, Color::Rgb(rgb.0, rgb.1, rgb.2));
    }

    // 3. Faint temporal shimmer
    let flash_phase = (millis / 120) as usize;
    for i in 0..2 {
        let seed = hash2d(flash_phase.wrapping_add(i), 99);
        let row = (seed as usize) % height;
        let col = (hash2d(flash_phase.wrapping_add(i), 111) as usize) % width;
        let brightness = 40 + ((seed % 30) as u8);
        put_cell(
            buffer,
            row as i32,
            col as i32,
            '\u{00b7}',
            Color::Rgb(brightness, brightness + 30, brightness + 80),
        );
    }
}
```

**Step 3: Verify it compiles**

Run: `cargo build 2>&1 | tail -3`
Expected: `Finished`

**Step 4: Commit**

```bash
git add src/ui/time_vault_scene.rs
git commit -m "feat(time-vault): add temporal backdrop paint function"
```

---

### Task 3: Rewrite draw_time_vault to use SceneCell buffer

**Files:**
- Modify: `src/ui/time_vault_scene.rs`

**Step 1: Replace the `draw_time_vault` function**

Replace the entire `draw_time_vault` function (lines 88-138) with a buffer-based renderer:

```rust
/// Render the Time Vault overlay.
pub fn draw_time_vault(frame: &mut Frame, area: Rect, state: &TimeVaultState) {
    // Full-screen overlay with padding
    let w = area.width.saturating_sub(4).min(90);
    let h = area.height.saturating_sub(4);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let overlay_area = Rect::new(x, y, w, h);

    frame.render_widget(Clear, overlay_area);

    // Reserve 1 row at top for title, 1 at bottom for controls
    let buf_w = w as usize;
    let buf_h = h.saturating_sub(2) as usize; // inside the double border
    if buf_w < 10 || buf_h < 5 {
        return;
    }

    let mut buffer = vec![vec![SceneCell::default(); buf_w]; buf_h];
    let millis = current_millis();
    paint_vault_backdrop(&mut buffer, millis);

    // Layout: branch panel on the left, snapshot panel on the right
    let branch_width = 20usize.min(buf_w / 3);
    let snap_x = branch_width + 1; // 1 col gap
    let snap_w = buf_w.saturating_sub(snap_x);

    paint_branch_panel(&mut buffer, state, branch_width);
    paint_snapshot_panel(&mut buffer, state, snap_x, snap_w);

    // Render buffer inside a bordered block
    let outer_block = Block::default()
        .title(
            Line::from(Span::styled(
                " TIME VAULT ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center),
        )
        .borders(Borders::ALL)
        .border_type(ratatui::widgets::BorderType::Double)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = outer_block.inner(overlay_area);
    frame.render_widget(outer_block, overlay_area);

    // Render the scene buffer into the inner area (above controls)
    let buffer_area = Rect::new(inner.x, inner.y, inner.width, inner.height.saturating_sub(1));
    render_buffer(frame, buffer_area, &buffer);

    // Controls bar at the bottom
    let controls_area = Rect::new(
        inner.x,
        inner.y + inner.height.saturating_sub(1),
        inner.width,
        1,
    );
    draw_controls(frame, controls_area, state);
}
```

**Step 2: Verify it compiles** (will have errors — the paint functions don't exist yet)

This step is combined with Task 4.

---

### Task 4: Rewrite branch panel as buffer painter

**Files:**
- Modify: `src/ui/time_vault_scene.rs`

**Step 1: Replace `draw_branch_panel` with `paint_branch_panel`**

Replace the old `draw_branch_panel` function with one that paints into the SceneCell buffer:

```rust
/// Paint the branch list into the scene buffer.
fn paint_branch_panel(buffer: &mut [Vec<SceneCell>], state: &TimeVaultState, width: usize) {
    let height = buffer.len();
    let focused = state.focus == PanelFocus::Left;

    // Panel title
    let title_color = if focused { Color::Cyan } else { Color::White };
    put_text(buffer, 0, 1, "Branches", title_color);

    // Thin separator
    let sep_color = if focused {
        Color::Rgb(40, 80, 120)
    } else {
        Color::DarkGray
    };
    let sep: String = "\u{2500}".repeat(width.saturating_sub(1));
    put_text(buffer, 1, 0, &sep, sep_color);

    // Branch list
    for (i, branch) in state.branches.iter().enumerate() {
        let row = 2 + i as i32;
        if row >= height as i32 {
            break;
        }

        let marker = if branch.is_active {
            "\u{25cf}" // ●
        } else {
            "\u{25cb}" // ○
        };

        let is_selected = i == state.selected_branch;
        let marker_color = if branch.is_active {
            Color::Green
        } else {
            Color::DarkGray
        };
        let name_style = if is_selected {
            Color::Yellow
        } else if branch.is_active {
            Color::Green
        } else {
            Color::White
        };

        put_text(buffer, row, 1, marker, marker_color);
        let label = format!(" {}", branch.name);
        put_text(buffer, row, 3, &label, name_style);
    }
}
```

**Step 2: Verify it compiles** (snapshot panel still needed — combined with Task 5)

---

### Task 5: Rewrite snapshot panel as buffer painter with timeline graph and icons

**Files:**
- Modify: `src/ui/time_vault_scene.rs`

**Step 1: Replace `draw_commit_panel` with `paint_snapshot_panel`**

Replace the old `draw_commit_panel` function:

```rust
/// Paint the snapshot timeline into the scene buffer.
fn paint_snapshot_panel(
    buffer: &mut [Vec<SceneCell>],
    state: &TimeVaultState,
    x_offset: usize,
    width: usize,
) {
    let height = buffer.len();
    let focused = state.focus == PanelFocus::Right;

    // Panel title
    let title_color = if focused { Color::Cyan } else { Color::White };
    put_text(buffer, 0, x_offset as i32 + 1, "Snapshots", title_color);

    // Thin separator
    let sep_color = if focused {
        Color::Rgb(40, 80, 120)
    } else {
        Color::DarkGray
    };
    let sep: String = "\u{2500}".repeat(width.saturating_sub(1));
    put_text(buffer, 1, x_offset as i32, &sep, sep_color);

    if state.commits.is_empty() {
        put_text(
            buffer,
            3,
            x_offset as i32 + 2,
            "No snapshots yet",
            Color::DarkGray,
        );
        return;
    }

    // Each card: 4 rows (description, date, stats, separator)
    let card_height = 4usize;
    let available_rows = height.saturating_sub(2); // below title+sep
    let visible_cards = (available_rows / card_height).max(1);

    // Scroll so selected commit is visible
    let scroll_offset = if state.selected_commit >= visible_cards {
        state.selected_commit - visible_cards + 1
    } else {
        0
    };

    let x = x_offset as i32;
    let mut row = 2i32; // start below title + separator
    let total_visible = state.commits.len().saturating_sub(scroll_offset);

    for (vi, (i, commit)) in state
        .commits
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .enumerate()
    {
        if row + 3 > height as i32 {
            break;
        }

        let is_selected = i == state.selected_commit;
        let is_last = vi == total_visible - 1 || row + 4 + 3 > height as i32;
        let (icon, icon_color) = event_icon_color(&commit.message);

        // Timeline node
        let node = if is_selected {
            "\u{25cf}" // ●
        } else {
            "\u{25cb}" // ○
        };
        let node_color = if is_selected {
            Color::Yellow
        } else {
            Color::Cyan
        };
        put_text(buffer, row, x + 2, node, node_color);

        // Icon
        put_text(buffer, row, x + 4, icon, icon_color);

        // Description
        let desc = commit.message.split(" | ").next().unwrap_or(&commit.message);
        let desc_color = if is_selected { Color::Yellow } else { Color::White };
        let icon_width = super::scene_fx::display_width(icon);
        put_text(buffer, row, x + 4 + icon_width as i32 + 1, desc, desc_color);

        // Timeline connector for rows below
        let dim = if is_selected {
            Color::Yellow
        } else {
            Color::DarkGray
        };

        // Date line
        let connector = if is_last { " " } else { "\u{2502}" }; // │
        put_text(buffer, row + 1, x + 2, connector, TIMELINE_DIM);
        let datetime = chrono::DateTime::from_timestamp(commit.timestamp, 0)
            .map(|dt| {
                dt.with_timezone(&chrono::Local)
                    .format("%b %d, %Y  %l:%M %p")
                    .to_string()
            })
            .unwrap_or_else(|| "Unknown".to_string());
        put_text(buffer, row + 1, x + 6, &datetime, dim);

        // Stats line
        let connector2 = if is_last { " " } else { "\u{2502}" }; // │
        put_text(buffer, row + 2, x + 2, connector2, TIMELINE_DIM);
        let hours = commit.playtime / 3600;
        let minutes = (commit.playtime % 3600) / 60;
        let stats = format!(
            "Lv{} \u{00b7} P{} \u{00b7} Zone {} \u{00b7} {}h {:02}m",
            commit.level, commit.prestige, commit.zone, hours, minutes
        );
        put_text(buffer, row + 2, x + 6, &stats, dim);

        // Separator line (thin ─── with connector)
        if !is_last {
            put_text(buffer, row + 3, x + 2, "\u{2502}", TIMELINE_DIM); // │
            let card_sep: String = "\u{2500}".repeat(width.saturating_sub(8));
            put_text(buffer, row + 3, x + 4, &card_sep, Color::Rgb(30, 50, 80));
        }

        row += card_height as i32;
    }
}
```

**Step 2: Remove the old `draw_branch_panel` and `draw_commit_panel` functions**

Delete the now-unused widget-based panel functions. The `draw_controls` function stays.

**Step 3: Verify it compiles**

Run: `cargo build 2>&1 | tail -3`
Expected: `Finished`

**Step 4: Commit**

```bash
git add src/ui/time_vault_scene.rs
git commit -m "feat(time-vault): buffer-based rendering with backdrop, timeline graph, and icons"
```

---

### Task 6: Update controls bar with dot separators

**Files:**
- Modify: `src/ui/time_vault_scene.rs`

**Step 1: Update the Browse mode controls**

In `draw_controls`, change the `BrowserMode::Browse` match arms to use `\u{00b7}` (·) separators:

For the left panel focus controls, replace `Span::raw("  ")` separators between control groups with:
```rust
Span::styled("  \u{00b7}  ", Style::default().fg(Color::Rgb(40, 80, 120)))
```

For the right panel focus controls, same change.

**Step 2: Verify it compiles**

Run: `cargo build 2>&1 | tail -3`
Expected: `Finished`

**Step 3: Commit**

```bash
git add src/ui/time_vault_scene.rs
git commit -m "feat(time-vault): dot separators in controls bar"
```

---

### Task 7: Clean up unused imports and dead code

**Files:**
- Modify: `src/ui/time_vault_scene.rs`

**Step 1: Remove unused imports**

After removing the widget-based panel functions, these imports from ratatui are likely unused: `List`, `ListItem`, possibly `Constraint`, `Direction`, `Layout`. Run clippy to identify them.

**Step 2: Run clippy**

Run: `cargo clippy --all-targets -- -D warnings 2>&1 | tail -10`
Expected: Clean or with specific unused import warnings to fix.

**Step 3: Fix any warnings**

Remove unused imports identified by clippy.

**Step 4: Run tests**

Run: `cargo test 2>&1 | tail -10`
Expected: All tests pass.

**Step 5: Commit**

```bash
git add src/ui/time_vault_scene.rs
git commit -m "chore(time-vault): clean up unused imports"
```

---

### Task 8: Manual verification

**Step 1: Run the game**

Run: `cargo run`

**Step 2: Test the Time Vault**

- Press `T` to open Time Vault
- Verify animated backdrop (dark blue gradient with drifting cyan particles)
- Verify timeline graph (`●`/`○` nodes with `│` connectors)
- Verify event icons and colors (⚔ red, ★ gold, ♟ magenta, etc.)
- Verify branch panel shows `●`/`○` markers
- Verify `Tab` switches focus between panels
- Verify `Up`/`Down` navigation works
- Verify controls bar has `·` dot separators
- Press `Esc` to close

**Step 3: Final commit if any tweaks needed**

---

## Summary

| Task | Description | Files |
|------|------------|-------|
| 1 | Event icon/color helper | `time_vault_scene.rs` |
| 2 | Backdrop constants + paint function | `time_vault_scene.rs` |
| 3 | Rewrite main draw function to buffer | `time_vault_scene.rs` |
| 4 | Branch panel as buffer painter | `time_vault_scene.rs` |
| 5 | Snapshot panel with timeline graph + icons | `time_vault_scene.rs` |
| 6 | Dot separators in controls | `time_vault_scene.rs` |
| 7 | Clean up unused imports | `time_vault_scene.rs` |
| 8 | Manual verification | — |

All changes are in a single file (`src/ui/time_vault_scene.rs`). No new files, no test changes, no data model changes.
