> Backported implementation plan (completed — this work shipped).

## 2026-02-21-sigil-panel-layout-plan.md

# Storm Sigil Panel Layout Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Move Storm Sigils from a sub-panel inside Equipment to a dedicated top-level panel between Attributes and Equipment in the stats panel.

**Architecture:** Extract the sigil rendering from `stats_equipment.rs` into a new `stats_sigils.rs` module. Modify `stats_panel.rs` to conditionally insert the sigil panel constraint. Remove sigil-related code from equipment rendering.

**Tech Stack:** Rust, Ratatui

---

### Task 1: Create `stats_sigils.rs` with dedicated sigil panel renderer

**Files:**
- Create: `src/ui/stats_sigils.rs`

**Step 1: Create the new file**

```rust
//! Storm Sigil rendering helpers for the stats panel.

use crate::stormglass::sigils::{SigilGrade, StormSigils};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Draws the Storm Sigils panel as a dedicated section.
/// Only call when `storm_sigils.etched_count() > 0`.
pub(super) fn draw_sigils_panel(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    storm_sigils: &StormSigils,
) {
    let etched = storm_sigils.etched_count();
    let title = format!(
        " \u{16B1} Storm Sigils ({}/{}) ",
        etched, storm_sigils.slots_unlocked
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(100, 180, 255)))
        .title(title);
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let width = inner.width as usize;
    let mut lines = Vec::new();

    for sigil in storm_sigils.sigils.iter().flatten() {
        let icon = sigil.effect.icon();
        let short = sigil.effect.short_name();
        let value_label = sigil.effect.format_value(sigil.value);
        let grade_str = sigil.grade.label();
        let grade_padded = format!("{:<2}", grade_str);
        let grade_color = sigil_grade_color(sigil.grade);

        let left = format!("{} {}", icon, short);
        let right = format!("{}  {}", value_label, grade_padded);
        let left_display_w = unicode_width::UnicodeWidthStr::width(left.as_str());
        let right_len = right.len();
        let pad = width.saturating_sub(left_display_w + right_len + 3);

        let grade_style = if grade_str.ends_with('+') {
            Style::default()
                .fg(grade_color)
                .add_modifier(Modifier::BOLD)
        } else if grade_str.ends_with('-') {
            Style::default().fg(grade_color).add_modifier(Modifier::DIM)
        } else {
            Style::default().fg(grade_color)
        };

        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(left, Style::default().fg(Color::White)),
            Span::raw(" ".repeat(pad.max(1))),
            Span::styled(value_label, Style::default().fg(Color::Rgb(100, 180, 255))),
            Span::styled(format!("  {}", grade_padded), grade_style),
        ]));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

/// Returns the color for a sigil grade tier letter.
fn sigil_grade_color(grade: SigilGrade) -> Color {
    match grade.tier_letter() {
        'S' => Color::Rgb(255, 215, 0),
        'A' => Color::Green,
        'B' => Color::Cyan,
        'C' => Color::White,
        'D' => Color::Gray,
        'E' => Color::DarkGray,
        _ => Color::Red,
    }
}
```

**Step 2: Verify it compiles**

Run: `cargo check 2>&1 | head -5`
Expected: May fail because mod not registered yet — that's fine, Task 2 handles it.

---

### Task 2: Register the module and wire up the stats panel

**Files:**
- Modify: `src/ui/mod.rs:40-43` — add `mod stats_sigils;` between `stats_equipment` and `stats_panel`
- Modify: `src/ui/stats_panel.rs:1-59` — add conditional sigil panel, remove `storm_sigils` param from equipment call

**Step 1: Add the module declaration**

In `src/ui/mod.rs`, add `mod stats_sigils;` after line 41 (`mod stats_equipment;`):

```
mod stats_equipment;
mod stats_sigils;
mod stats_panel;
```

**Step 2: Update `draw_stats_panel` in `stats_panel.rs`**

Add the import at top (line 6, after `draw_prestige_info`):
```rust
use super::stats_sigils::draw_sigils_panel;
```

Replace the layout and draw calls in the `XL | L` arm (lines 34-55) with:

```rust
        SizeTier::XL | SizeTier::L => {
            let etched = game_state.storm_sigils.etched_count();
            let mut constraints = vec![
                Constraint::Length(4),  // header
                Constraint::Length(5),  // prestige
                Constraint::Length(4),  // fishing
                Constraint::Length(5),  // attributes
            ];
            if etched > 0 {
                constraints.push(Constraint::Length(etched as u16 + 2)); // sigils
            }
            constraints.push(Constraint::Min(0)); // equipment

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(area);

            let mut idx = 0;
            draw_header(frame, chunks[idx], game_state, achievements);
            idx += 1;
            draw_prestige_info(frame, chunks[idx], game_state, achievements);
            idx += 1;
            draw_fishing_panel(frame, chunks[idx], game_state, achievements);
            idx += 1;
            draw_attributes_compact(frame, chunks[idx], game_state);
            idx += 1;
            if etched > 0 {
                draw_sigils_panel(frame, chunks[idx], &game_state.storm_sigils);
                idx += 1;
            }
            draw_equipment_names_only(
                frame,
                chunks[idx],
                game_state,
                enhancement_levels,
            );
        }
```

**Step 3: Update `draw_equipment_names_only` signature**

In `src/ui/stats_equipment.rs`, remove the `storm_sigils` parameter from `draw_equipment_names_only` (line 27-33):

Change:
```rust
pub(super) fn draw_equipment_names_only(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    enhancement_levels: &[u8; 7],
    storm_sigils: &StormSigils,
) {
```

To:
```rust
pub(super) fn draw_equipment_names_only(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    enhancement_levels: &[u8; 7],
) {
```

**Step 4: Verify it compiles**

Run: `cargo check 2>&1 | head -10`
Expected: Errors about unused imports and the removed sigil code still in `stats_equipment.rs` — Task 3 cleans that up.

---

### Task 3: Remove sigil sub-panel from equipment

**Files:**
- Modify: `src/ui/stats_equipment.rs` — remove sigil splitting logic and sub-panel renderer

**Step 1: Remove the sigil import and sub-panel splitting**

In `stats_equipment.rs`:

1. Remove `use crate::stormglass::sigils::{SigilGrade, StormSigils};` from line 4 (keep `SigilGrade` only if stormglass_scene needs re-export — it doesn't, it has its own copy).

2. Replace lines 34-53 (the equipment block + sigil area splitting) with just the equipment block:

```rust
    let block = Block::default().borders(Borders::ALL).title(" Equipment ");
    let inner = block.inner(area);
    frame.render_widget(block, area);
```

Remove the old code:
```rust
    // Split inner area: 7 lines for equipment, remainder for sigils
    let etched = storm_sigils.etched_count();
    let (equip_area, sigil_area) = if etched > 0 && inner.height > 9 {
        ...
    };
```

3. Replace `equip_area` with `inner` in the equipment rendering code (line 55 `let width = equip_area.width...` and line 135 `frame.render_widget(paragraph, equip_area)`).

4. Remove the sigil sub-panel render call at lines 137-140:
```rust
    // Render sigil sub-panel if any are etched
    if let Some(sigil_area) = sigil_area {
        draw_sigil_sub_panel(frame, sigil_area, storm_sigils);
    }
```

5. Remove the entire `draw_sigil_sub_panel` function (lines 143-196).

6. Remove the `sigil_grade_color` function (lines 198-209) — it's now in `stats_sigils.rs`.

7. Clean up unused imports: remove `Direction`, `Layout`, `Constraint` if no longer used. Remove `SigilGrade`, `StormSigils` imports.

**Step 2: Verify it compiles**

Run: `cargo check 2>&1 | head -10`
Expected: PASS (no errors)

**Step 3: Run all tests**

Run: `cargo test 2>&1 | tail -5`
Expected: All tests pass (UI rendering isn't directly tested)

**Step 4: Run full CI checks**

Run: `make check 2>&1 | tail -20`
Expected: All checks pass (format, clippy, test, build)

**Step 5: Commit**

```bash
git add src/ui/stats_sigils.rs src/ui/stats_equipment.rs src/ui/stats_panel.rs src/ui/mod.rs
git commit -m "refactor(ui): move Storm Sigils to dedicated stats panel section

Extract sigil rendering from Equipment sub-panel into its own
stats_sigils.rs module. Sigil panel appears between Attributes
and Equipment when at least 1 sigil is etched, with dynamic height.
Equipment block is now purely equipment again."
```
