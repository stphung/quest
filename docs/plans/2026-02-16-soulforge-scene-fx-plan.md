# Soulforge Scene FX Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add scene_fx-powered visual effects to all 5 Soulforge overlay phases — ember/furnace backdrop, spark showers on hammer strikes, golden burst on success, ash decay on failure.

**Architecture:** Convert `soulforge_scene.rs` from pure Ratatui `Paragraph`/`Span` widgets to `SceneCell` buffer rendering (same pattern as `fishing_scene.rs` and `combat_3d.rs`). Each phase function creates its own buffer, paints a parameterized forge backdrop, renders text via `put_cell()`, then flushes with `render_buffer()`. The entry point (`render_soulforge`) and Block border stay unchanged.

**Tech Stack:** Ratatui, scene_fx (`SceneCell`, `put_cell`, `render_buffer`, `hash2d`, `lerp_rgb`, `current_millis`)

**Worktree:** `/Users/stphung/workspace/quest/.worktrees/soulforge-scene-fx` (branch: `feature/soulforge-scene-fx`)

**Design doc:** `docs/plans/2026-02-16-soulforge-scene-fx-design.md`

**Important:** This is purely UI rendering code. There are no unit-testable behaviors — verification is `cargo build` (compilation) + visual testing by running the game with `--debug` flag and triggering Soulforge via the debug menu. Each task ends with a build check and commit.

---

### Task 1: Add scene_fx imports and text helper functions

**Files:**
- Modify: `src/ui/soulforge_scene.rs` (top of file)

**Step 1: Add scene_fx imports**

Add to the existing import block at the top of `soulforge_scene.rs`:

```rust
use super::scene_fx::{current_millis, hash2d, lerp_rgb, put_cell, render_buffer, SceneCell};
```

Remove `Clear` from the ratatui widget imports in render_menu/render_confirming/etc. (they won't need it anymore — but keep it in `render_soulforge` since the entry point still clears).

**Step 2: Add put_text helper**

Add after the `level_color()` function:

```rust
/// Write a string into the scene buffer at (row, col). Each char occupies 1 cell.
fn put_text(buffer: &mut [Vec<SceneCell>], row: i32, col: i32, text: &str, fg: Color) {
    for (i, ch) in text.chars().enumerate() {
        put_cell(buffer, row, col + i as i32, ch, fg);
    }
}

/// Write a string centered horizontally in the buffer.
fn put_text_centered(buffer: &mut [Vec<SceneCell>], row: i32, width: usize, text: &str, fg: Color) {
    let col = (width as i32 - text.len() as i32) / 2;
    put_text(buffer, row, col, text, fg);
}
```

**Step 3: Verify build**

Run: `cargo build --all-targets 2>&1 | tail -5` in the worktree directory.
Expected: Compiles with possible unused import warnings (acceptable until we convert the phase functions).

**Step 4: Commit**

```bash
git add src/ui/soulforge_scene.rs
git commit -m "feat(soulforge): add scene_fx imports and text helper functions"
```

---

### Task 2: Add forge backdrop infrastructure

**Files:**
- Modify: `src/ui/soulforge_scene.rs`

**Step 1: Add ForgeBackdropParams struct and presets**

Add after the text helpers from Task 1:

```rust
/// Parameters controlling the forge backdrop appearance.
struct ForgeBackdropParams {
    bottom_rgb: (u8, u8, u8),
    top_rgb: (u8, u8, u8),
    ember_count: usize,
    ember_speed: f64,    // rows per second
    ember_upward: bool,  // true = rise, false = fall
    ember_hot: (u8, u8, u8),
    ember_cool: (u8, u8, u8),
    shimmer: bool,
}

impl ForgeBackdropParams {
    /// Standard warm forge glow (Menu, Confirming phases).
    fn normal() -> Self {
        Self {
            bottom_rgb: (120, 40, 15),
            top_rgb: (15, 8, 5),
            ember_count: 10,
            ember_speed: 5.0,
            ember_upward: true,
            ember_hot: (255, 160, 40),
            ember_cool: (80, 20, 5),
            shimmer: true,
        }
    }

    /// Intensified forge during hammering.
    fn hot() -> Self {
        Self {
            bottom_rgb: (180, 60, 20),
            top_rgb: (25, 12, 8),
            ember_count: 14,
            ember_speed: 7.0,
            ember_upward: true,
            ember_hot: (255, 200, 60),
            ember_cool: (120, 40, 10),
            shimmer: true,
        }
    }

    /// Golden glow for success result.
    fn golden() -> Self {
        Self {
            bottom_rgb: (200, 170, 50),
            top_rgb: (40, 30, 10),
            ember_count: 20,
            ember_speed: 8.0,
            ember_upward: true,
            ember_hot: (255, 230, 100),
            ember_cool: (200, 150, 30),
            shimmer: true,
        }
    }

    /// Cold ash for failure result.
    fn ash() -> Self {
        Self {
            bottom_rgb: (40, 40, 45),
            top_rgb: (15, 15, 18),
            ember_count: 4,
            ember_speed: 2.0,
            ember_upward: false,
            ember_hot: (80, 30, 10),
            ember_cool: (30, 15, 10),
            shimmer: false,
        }
    }
}
```

**Step 2: Add paint_forge_backdrop function**

```rust
/// Paint the forge backdrop into the buffer: gradient background, drifting embers, heat shimmer.
fn paint_forge_backdrop(
    buffer: &mut [Vec<SceneCell>],
    millis: u128,
    params: &ForgeBackdropParams,
) {
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
        let rgb = lerp_rgb(params.top_rgb, params.bottom_rgb, t);
        let bg = Color::Rgb(rgb.0, rgb.1, rgb.2);
        for cell in row_cells.iter_mut() {
            cell.bg = bg;
        }
    }

    // 2. Drifting embers
    let ember_chars: &[char] = &['\u{00b7}', '\u{2022}', '*', '\u{2726}'];
    for i in 0..params.ember_count {
        let seed = hash2d(i, 0);
        let col = (seed as usize) % width;
        let ch = ember_chars[(hash2d(i, 1) as usize) % ember_chars.len()];

        // Each ember drifts at ember_speed rows/sec with a unique phase offset
        let phase_offset = (seed as f64) * 0.73;
        let pos = (phase_offset + millis as f64 * params.ember_speed / 1000.0) % height as f64;
        let row_f = if params.ember_upward {
            (height - 1) as f64 - pos
        } else {
            pos
        };
        let row = row_f as i32;

        // Color fades from hot (bottom) to cool (top) for upward, reverse for downward
        let t = pos / height.max(1) as f64;
        let rgb = lerp_rgb(params.ember_hot, params.ember_cool, t);
        put_cell(buffer, row, col as i32, ch, Color::Rgb(rgb.0, rgb.1, rgb.2));
    }

    // 3. Heat shimmer — subtle red channel oscillation on sparse cells
    if params.shimmer {
        let shimmer_phase = millis as f64 / 150.0;
        for (row, row_cells) in buffer.iter_mut().enumerate() {
            for (col, cell) in row_cells.iter_mut().enumerate() {
                if hash2d(row, col).is_multiple_of(7) {
                    let shift = ((shimmer_phase + row as f64 * 0.3 + col as f64 * 0.2).sin()
                        * 8.0) as i16;
                    if let Color::Rgb(r, g, b) = cell.bg {
                        let new_r = (r as i16 + shift).clamp(0, 255) as u8;
                        cell.bg = Color::Rgb(new_r, g, b);
                    }
                }
            }
        }
    }
}
```

**Step 3: Verify build**

Run: `cargo build --all-targets 2>&1 | tail -5`
Expected: Compiles (unused warnings OK).

**Step 4: Commit**

```bash
git add src/ui/soulforge_scene.rs
git commit -m "feat(soulforge): add forge backdrop infrastructure with gradient, embers, shimmer"
```

---

### Task 3: Convert Menu phase to scene buffer

**Files:**
- Modify: `src/ui/soulforge_scene.rs` — rewrite `render_menu()`

**Step 1: Rewrite render_menu**

Replace the entire `render_menu` function. The new version:

1. Creates a `SceneCell` buffer sized to `area`
2. Calls `paint_forge_backdrop()` with `ForgeBackdropParams::normal()`
3. Renders all text content via `put_text()` / `put_cell()`:
   - Row 0-2: Flavor text (hardcoded line breaks to fit width)
   - Row 4-10: 7 equipment slot rows with selection indicator, icon, name, level, success rate
   - Row 12-14: Detail panel for selected slot (bonus, rate, cost, failure info)
   - Row 16: Stats line (attempts/successes/failures)
   - Row 17: Help line
4. Calls `render_buffer(frame, area, &buffer)`

Key layout details for the slot rows (same info as current, just via put_cell):

```rust
fn render_menu(
    frame: &mut Frame,
    area: Rect,
    soulforge_ui: &SoulforgeUiState,
    enhancement: &EnhancementProgress,
    prestige_rank: u32,
) {
    let width = area.width as usize;
    let height = area.height as usize;
    if width < 20 || height < 10 {
        return;
    }
    let mut buffer = vec![vec![SceneCell::default(); width]; height];
    let millis = current_millis();
    paint_forge_backdrop(&mut buffer, millis, &ForgeBackdropParams::normal());

    // Flavor text (rows 0-2) with slow warm pulse
    let pulse_t = ((millis as f64 / 3000.0).sin() * 0.5 + 0.5) as f64;
    let flavor_rgb = lerp_rgb((180, 160, 140), (220, 190, 150), pulse_t);
    let flavor_color = Color::Rgb(flavor_rgb.0, flavor_rgb.1, flavor_rgb.2);
    put_text(&mut buffer, 0, 1, "Ancient runes pulse with forgotten power.", flavor_color);
    put_text(&mut buffer, 1, 1, "This forge tempers the soul, not the steel.", flavor_color);
    put_text(&mut buffer, 2, 1, "All that you wield will strike truer.", flavor_color);

    // Equipment slot rows (rows 4-10)
    for (i, slot) in SLOT_ORDER.iter().enumerate() {
        let row = (4 + i) as i32;
        let is_selected = i == soulforge_ui.selected_slot;
        let current_level = enhancement.level(i);

        // Highlight background for selected row
        if is_selected {
            for col in 0..width {
                buffer[row as usize][col].bg = Color::Rgb(40, 40, 20);
            }
        }

        let mut col: i32 = 1;

        // Selection indicator
        if is_selected {
            put_text(&mut buffer, row, col, "> ", Color::Yellow);
        }
        col += 2;

        // Slot icon
        let icon = slot.icon();
        put_text(&mut buffer, row, col, icon, Color::White);
        col += slot.icon_width() as i32 + 1;

        // Slot name (padded to 8 chars)
        let name = format!("{:<8}", slot.name());
        put_text(&mut buffer, row, col, &name, Color::White);
        col += 9;

        // Enhancement level and target
        if current_level >= MAX_ENHANCEMENT_LEVEL {
            put_text(
                &mut buffer, row, col, "+10 MAX",
                Color::Rgb(255, 215, 0),
            );
        } else {
            let lvl_color = level_color(current_level);
            let target = current_level + 1;
            let lvl_text = format!("+{:<2}", current_level);
            put_text(&mut buffer, row, col, &lvl_text, lvl_color);
            col += 3;

            let arrow = format!(" \u{2192} +{:<2}", target);
            put_text(&mut buffer, row, col, &arrow, level_color(target));
            col += arrow.len() as i32;

            let rate = success_rate(target);
            let rate_color = if rate >= 1.0 {
                Color::Green
            } else if rate >= 0.5 {
                Color::Yellow
            } else {
                Color::Red
            };
            let rate_text = format!(" {:>3.0}% Success", rate * 100.0);
            put_text(&mut buffer, row, col, &rate_text, rate_color);
        }
    }

    // Detail panel for selected slot (rows 12-14)
    let selected_level = enhancement.level(soulforge_ui.selected_slot);
    let detail_row = 12i32;

    if selected_level >= MAX_ENHANCEMENT_LEVEL {
        let bonus_text = format!(
            "Bonus: +{:.1}% Power",
            (enhancement_multiplier(MAX_ENHANCEMENT_LEVEL) - 1.0) * 100.0
        );
        put_text(&mut buffer, detail_row, 1, "Bonus: ", Color::DarkGray);
        put_text(
            &mut buffer, detail_row, 8,
            &format!("+{:.1}% Power", (enhancement_multiplier(MAX_ENHANCEMENT_LEVEL) - 1.0) * 100.0),
            Color::Rgb(255, 215, 0),
        );
        put_text(&mut buffer, detail_row + 1, 1, "Maximum enhancement reached.", Color::DarkGray);
    } else {
        let target = selected_level + 1;
        let bonus_pct = (enhancement_multiplier(target) - 1.0) * 100.0;
        let rate = success_rate(target);
        let cost = enhancement_cost(target);
        let can_afford = prestige_rank >= cost;
        let penalty = fail_penalty(target);

        // Row 1: Bonus, Rate, Cost
        let mut col = 1i32;
        put_text(&mut buffer, detail_row, col, "Bonus: ", Color::DarkGray);
        col += 7;
        let bonus_str = format!("+{:.1}% Power", bonus_pct);
        put_text(&mut buffer, detail_row, col, &bonus_str, Color::Green);
        col += bonus_str.len() as i32;
        put_text(&mut buffer, detail_row, col, "  Rate: ", Color::DarkGray);
        col += 8;
        let rate_color = if rate >= 1.0 { Color::Green } else if rate >= 0.5 { Color::Yellow } else { Color::Red };
        let rate_str = format!("{:.0}%", rate * 100.0);
        put_text(&mut buffer, detail_row, col, &rate_str, rate_color);
        col += rate_str.len() as i32;
        put_text(&mut buffer, detail_row, col, "  Cost: ", Color::DarkGray);
        col += 8;
        let cost_color = if can_afford { Color::Cyan } else { Color::Red };
        let cost_str = format!("{} Prestige Ranks", cost);
        put_text(&mut buffer, detail_row, col, &cost_str, cost_color);

        // Row 2: On failure
        put_text(&mut buffer, detail_row + 1, 1, "On failure: ", Color::DarkGray);
        if penalty == 0 {
            put_text(&mut buffer, detail_row + 1, 13, "safe (no level loss)", Color::Green);
        } else {
            let result_level = selected_level.saturating_sub(penalty);
            let fail_text = format!(
                "-{} level{} (+{} \u{2192} +{})",
                penalty,
                if penalty > 1 { "s" } else { "" },
                selected_level,
                result_level
            );
            put_text(&mut buffer, detail_row + 1, 13, &fail_text, Color::Red);
        }
    }

    // Stats row (row 16)
    let stats_row = 16i32;
    let mut col = 1i32;
    put_text(&mut buffer, stats_row, col, "Attempts: ", Color::DarkGray);
    col += 10;
    let attempts_str = format!("{}", enhancement.total_attempts);
    put_text(&mut buffer, stats_row, col, &attempts_str, Color::White);
    col += attempts_str.len() as i32;
    put_text(&mut buffer, stats_row, col, " | Successes: ", Color::DarkGray);
    col += 14;
    let success_str = format!("{}", enhancement.total_successes);
    put_text(&mut buffer, stats_row, col, &success_str, Color::Green);
    col += success_str.len() as i32;
    put_text(&mut buffer, stats_row, col, " | Failures: ", Color::DarkGray);
    col += 13;
    let fail_str = format!("{}", enhancement.total_failures);
    put_text(&mut buffer, stats_row, col, &fail_str, Color::Red);

    // Help row (row 17)
    put_text(
        &mut buffer, 17, 1,
        "[\u{2191}\u{2193}] Select  [Enter] Enhance  [Esc] Close",
        Color::DarkGray,
    );

    render_buffer(frame, area, &buffer);
}
```

**Step 2: Verify build**

Run: `cargo build --all-targets 2>&1 | tail -5`
Expected: Compiles successfully.

**Step 3: Commit**

```bash
git add src/ui/soulforge_scene.rs
git commit -m "feat(soulforge): convert menu phase to scene_fx buffer with forge backdrop"
```

---

### Task 4: Convert Confirming phase to scene buffer

**Files:**
- Modify: `src/ui/soulforge_scene.rs` — rewrite `render_confirming()`

**Step 1: Rewrite render_confirming**

Replace the `render_confirming` function. Same pattern as menu but with intensified backdrop and centered text:

```rust
fn render_confirming(
    frame: &mut Frame,
    area: Rect,
    soulforge_ui: &SoulforgeUiState,
    enhancement: &EnhancementProgress,
    prestige_rank: u32,
) {
    let width = area.width as usize;
    let height = area.height as usize;
    if width < 20 || height < 10 {
        return;
    }
    let mut buffer = vec![vec![SceneCell::default(); width]; height];
    let millis = current_millis();

    // Slightly intensified backdrop for confirming
    let mut params = ForgeBackdropParams::normal();
    params.bottom_rgb = (150, 50, 18);
    params.ember_count = 12;
    paint_forge_backdrop(&mut buffer, millis, &params);

    let slot_index = soulforge_ui.selected_slot;
    let slot = SLOT_ORDER[slot_index];
    let current_level = enhancement.level(slot_index);
    let target_level = current_level + 1;
    let cost = enhancement_cost(target_level);
    let rate = success_rate(target_level);
    let bonus_pct = (enhancement_multiplier(target_level) - 1.0) * 100.0;

    // Center content vertically
    let start_row = (height / 2).saturating_sub(4) as i32;

    // Title with pulsing glow
    let pulse = ((millis as f64 / 500.0).sin() * 0.5 + 0.5) as f64;
    let title_rgb = lerp_rgb((255, 200, 0), (255, 240, 100), pulse);
    let title = format!("Enhance {} to +{}?", slot.name(), target_level);
    put_text_centered(&mut buffer, start_row + 1, width, &title, Color::Rgb(title_rgb.0, title_rgb.1, title_rgb.2));

    // Rate and cost
    let rate_color = if rate >= 0.5 { Color::Green } else { Color::Red };
    let rate_str = format!("Success rate: {:.0}%", rate * 100.0);
    let cost_str = format!("  Cost: {} Prestige Ranks", cost);
    let remaining = format!(" ({} \u{2192} {})", prestige_rank, prestige_rank.saturating_sub(cost));
    // Build combined line and center it
    let combined = format!("{}{}{}", rate_str, cost_str, remaining);
    let line_start = ((width as i32 - combined.len() as i32) / 2).max(1);
    put_text(&mut buffer, start_row + 3, line_start, &rate_str, rate_color);
    put_text(&mut buffer, start_row + 3, line_start + rate_str.len() as i32, &cost_str, Color::Cyan);
    put_text(&mut buffer, start_row + 3, line_start + rate_str.len() as i32 + cost_str.len() as i32, &remaining, Color::DarkGray);

    // Bonus line
    let bonus_line = format!("Bonus at +{}: +{:.1}% Power", target_level, bonus_pct);
    put_text_centered(&mut buffer, start_row + 4, width, &bonus_line, Color::Green);

    // Help
    put_text_centered(&mut buffer, start_row + 6, width, "[Enter] Confirm  [Esc] Cancel", Color::DarkGray);

    render_buffer(frame, area, &buffer);
}
```

**Step 2: Verify build**

Run: `cargo build --all-targets 2>&1 | tail -5`

**Step 3: Commit**

```bash
git add src/ui/soulforge_scene.rs
git commit -m "feat(soulforge): convert confirming phase to scene_fx buffer"
```

---

### Task 5: Convert Hammering phase with spark effects

**Files:**
- Modify: `src/ui/soulforge_scene.rs` — rewrite `render_hammering()`

This is the most complex task. The hammering phase adds spark particle physics on hammer strikes plus anvil glow and afterimage effects.

**Step 1: Add spark rendering helper**

Add before `render_hammering`:

```rust
/// Render spark particles that spray from a point. Sparks are fully derived from
/// animation_tick — no mutable state needed.
/// `strike_tick` is the tick when the strike started (14, 30, or 46).
/// `current_tick` is the current animation_tick.
fn render_sparks(
    buffer: &mut [Vec<SceneCell>],
    center_row: i32,
    center_col: i32,
    strike_tick: u8,
    current_tick: u8,
    spark_count: usize,
) {
    let age = current_tick.saturating_sub(strike_tick);
    if age > 10 {
        return; // sparks expired
    }

    let spark_chars: &[char] = &['\u{2726}', '\u{00b7}', '*', '\u{2727}'];
    let t = age as f64;

    for i in 0..spark_count {
        let seed = hash2d(strike_tick as usize, i);
        let ch = spark_chars[(seed as usize) % spark_chars.len()];

        // Fan angle: 30-150 degrees (in radians: ~0.52 - 2.62)
        let angle = 0.52 + (seed % 1000) as f64 / 1000.0 * 2.1;
        let speed = 1.5 + (hash2d(i, strike_tick as usize) % 100) as f64 / 100.0 * 2.0;

        // x = center + vx * t, y = center - vy * t + 0.5 * gravity * t^2
        let vx = angle.cos() * speed;
        let vy = angle.sin() * speed;
        let gravity = 0.15;

        let col = center_col as f64 + vx * t;
        let row = center_row as f64 - vy * t + 0.5 * gravity * t * t;

        // Color: bright white/yellow → orange → dark red
        let color_t = age as f64 / 10.0;
        let rgb = if color_t < 0.3 {
            lerp_rgb((255, 255, 200), (255, 200, 60), color_t / 0.3)
        } else {
            lerp_rgb((255, 200, 60), (120, 30, 5), (color_t - 0.3) / 0.7)
        };

        put_cell(
            buffer,
            row.round() as i32,
            col.round() as i32,
            ch,
            Color::Rgb(rgb.0, rgb.1, rgb.2),
        );
    }
}
```

**Step 2: Rewrite render_hammering**

Replace the `render_hammering` function:

```rust
fn render_hammering(
    frame: &mut Frame,
    area: Rect,
    soulforge_ui: &SoulforgeUiState,
    enhancement: &EnhancementProgress,
) {
    let width = area.width as usize;
    let height = area.height as usize;
    if width < 20 || height < 10 {
        return;
    }
    let mut buffer = vec![vec![SceneCell::default(); width]; height];
    let millis = current_millis();
    paint_forge_backdrop(&mut buffer, millis, &ForgeBackdropParams::hot());

    let tick = soulforge_ui.animation_tick;
    let is_strike = matches!(tick, 14..=16 | 30..=32 | 46..=48);

    // Hammer ASCII art (centered in buffer)
    let hammer_raised = [
        "       ___  ",
        "      |   | ",
        "      |___|/",
        "        |   ",
        "        |   ",
    ];
    let hammer_strike = [
        "            ",
        "            ",
        "   ___      ",
        "  |   |___  ",
        "  |___|/    ",
    ];

    let anvil = [
        "    _________    ",
        "   /         \\   ",
        "  /___________\\  ",
        "      |   |      ",
        "   ___|   |___   ",
        "  |___________|  ",
    ];

    let hammer = if is_strike { &hammer_strike } else { &hammer_raised };

    // Position: hammer starts at row 2, anvil below hammer at row 8
    let hammer_start_row = 2i32;
    let anvil_start_row = 8i32;
    let art_center_col = (width / 2) as i32 - 6;

    // Hammer afterimage: on first tick of a strike, show raised position dimmed
    if matches!(tick, 14 | 30 | 46) {
        for (i, line) in hammer_raised.iter().enumerate() {
            let row = hammer_start_row + i as i32;
            put_text(&mut buffer, row, art_center_col, line, Color::Rgb(60, 30, 15));
        }
    }

    // Main hammer
    let hammer_color = if is_strike {
        Color::White
    } else {
        Color::Rgb(140, 130, 120)
    };
    for (i, line) in hammer.iter().enumerate() {
        let row = hammer_start_row + i as i32;
        put_text(&mut buffer, row, art_center_col, line, hammer_color);
    }

    // Anvil with glow effect
    let anvil_color = if is_strike {
        // Pulse from gray to warm orange on strike
        let glow_t = ((tick as f64 - 14.0).abs() % 3.0) / 3.0;
        let rgb = lerp_rgb((200, 140, 40), (100, 90, 80), glow_t);
        Color::Rgb(rgb.0, rgb.1, rgb.2)
    } else {
        // Subtle warm tint between strikes
        Color::Rgb(90, 80, 70)
    };
    for (i, line) in anvil.iter().enumerate() {
        let row = anvil_start_row + i as i32;
        put_text(&mut buffer, row, art_center_col, line, anvil_color);
    }

    // Spark shower on strikes
    let anvil_contact_row = anvil_start_row; // top of anvil
    let anvil_contact_col = (width / 2) as i32;
    for &strike_start in &[14u8, 30, 46] {
        if tick >= strike_start && tick <= strike_start + 10 {
            render_sparks(&mut buffer, anvil_contact_row, anvil_contact_col, strike_start, tick, 10);
        }
    }

    // Progress bar (row height-4)
    let progress = tick as f64 / 50.0;
    let bar_row = (height - 4) as i32;
    let bar_width = width.saturating_sub(6);
    let fill_exact = progress * bar_width as f64;
    let full_cells = fill_exact as usize;

    put_text(&mut buffer, bar_row, 2, "[", Color::DarkGray);
    // Filled portion with pulsing warm gradient
    let bar_pulse = ((millis as f64 / 300.0).sin() * 0.5 + 0.5) as f64;
    let bar_rgb = lerp_rgb((180, 80, 10), (255, 200, 40), bar_pulse);
    let bar_color = Color::Rgb(bar_rgb.0, bar_rgb.1, bar_rgb.2);
    for i in 0..full_cells.min(bar_width) {
        put_cell(&mut buffer, bar_row, 3 + i as i32, '\u{2588}', bar_color);
    }
    // Partial block
    let fraction = fill_exact - full_cells as f64;
    let blocks: &[char] = &['\u{258f}', '\u{258e}', '\u{258d}', '\u{258c}', '\u{258b}', '\u{258a}', '\u{2589}', '\u{2588}'];
    let partial_idx = (fraction * 8.0) as usize;
    if partial_idx > 0 && full_cells < bar_width {
        put_cell(&mut buffer, bar_row, 3 + full_cells as i32, blocks[partial_idx - 1], bar_color);
    }
    put_text(&mut buffer, bar_row, (3 + bar_width) as i32, "]", Color::DarkGray);

    // Item label below bar
    let slot_index = soulforge_ui.selected_slot;
    let slot = SLOT_ORDER[slot_index];
    let current_level = enhancement.level(slot_index);
    let item_display = format!("{} +{}", slot.name(), current_level);
    put_text_centered(&mut buffer, bar_row + 1, width, &item_display, Color::White);

    render_buffer(frame, area, &buffer);
}
```

**Step 3: Verify build**

Run: `cargo build --all-targets 2>&1 | tail -5`

**Step 4: Commit**

```bash
git add src/ui/soulforge_scene.rs
git commit -m "feat(soulforge): convert hammering phase with spark shower and anvil glow"
```

---

### Task 6: Convert Success phase with golden burst

**Files:**
- Modify: `src/ui/soulforge_scene.rs` — rewrite `render_success()`

**Step 1: Rewrite render_success**

```rust
fn render_success(frame: &mut Frame, area: Rect, soulforge_ui: &SoulforgeUiState) {
    let width = area.width as usize;
    let height = area.height as usize;
    if width < 20 || height < 10 {
        return;
    }
    let mut buffer = vec![vec![SceneCell::default(); width]; height];
    let millis = current_millis();
    let tick = soulforge_ui.animation_tick;

    // Golden backdrop — intensifies over first 30 ticks then settles
    let intensity = (tick as f64 / 30.0).min(1.0);
    let mut params = ForgeBackdropParams::normal();
    params.bottom_rgb = lerp_rgb((120, 40, 15), (200, 170, 50), intensity);
    params.top_rgb = lerp_rgb((15, 8, 5), (40, 30, 10), intensity);
    params.ember_count = 10 + (intensity * 10.0) as usize;
    params.ember_speed = 5.0 + intensity * 3.0;
    params.ember_hot = lerp_rgb((255, 160, 40), (255, 230, 100), intensity);
    params.ember_cool = lerp_rgb((80, 20, 5), (200, 150, 30), intensity);
    paint_forge_backdrop(&mut buffer, millis, &params);

    // Sparkle twinkle — random characters appear and fade
    let sparkle_chars: &[char] = &['\u{2726}', '\u{2727}', '*'];
    for i in 0..15 {
        let seed = hash2d(i, tick as usize / 3);
        let row = (seed as usize) % height;
        let col = (hash2d(i + 100, tick as usize / 3) as usize) % width;
        let ch = sparkle_chars[(seed as usize) % sparkle_chars.len()];
        let phase = (tick % 4) as f64 / 4.0;
        let bright = ((phase + i as f64 * 0.3).sin() * 0.5 + 0.5) as f64;
        let rgb = lerp_rgb((100, 80, 20), (255, 230, 100), bright);
        put_cell(&mut buffer, row as i32, col as i32, ch, Color::Rgb(rgb.0, rgb.1, rgb.2));
    }

    let result = soulforge_ui.last_result.as_ref().unwrap();
    let slot = SLOT_ORDER[result.slot_index];
    let bonus_pct = (enhancement_multiplier(result.new_level) - 1.0) * 100.0;

    let center_row = (height / 2).saturating_sub(3) as i32;

    // Sparkle border line
    let sparkle_line: String = (0..width.saturating_sub(4))
        .map(|i| {
            let ch_idx = (hash2d(i, tick as usize) as usize) % sparkle_chars.len();
            sparkle_chars[ch_idx]
        })
        .collect();
    put_text_centered(&mut buffer, center_row, width, &sparkle_line, Color::Yellow);

    // SUCCESS! text — pulse between yellow and gold
    let title_color = if tick % 4 < 2 {
        Color::Yellow
    } else {
        Color::Rgb(255, 215, 0)
    };
    put_text_centered(&mut buffer, center_row + 2, width, "SUCCESS!", title_color);

    // Slot result
    let result_text = format!("{} is now +{}!", slot.name(), result.new_level);
    put_text_centered(&mut buffer, center_row + 4, width, &result_text, Color::Green);

    // Power bonus
    let power_text = format!("+{:.1}% Power", bonus_pct);
    put_text_centered(&mut buffer, center_row + 5, width, &power_text, Color::Yellow);

    // Bottom sparkle line
    put_text_centered(&mut buffer, center_row + 7, width, &sparkle_line, Color::Yellow);

    // Continue prompt
    put_text_centered(&mut buffer, center_row + 8, width, "Press any key to continue", Color::DarkGray);

    render_buffer(frame, area, &buffer);
}
```

**Step 2: Verify build**

Run: `cargo build --all-targets 2>&1 | tail -5`

**Step 3: Commit**

```bash
git add src/ui/soulforge_scene.rs
git commit -m "feat(soulforge): convert success phase with golden burst and sparkle effects"
```

---

### Task 7: Convert Failure phase with ash decay

**Files:**
- Modify: `src/ui/soulforge_scene.rs` — rewrite `render_failure()`

**Step 1: Rewrite render_failure**

```rust
fn render_failure(frame: &mut Frame, area: Rect, soulforge_ui: &SoulforgeUiState) {
    let width = area.width as usize;
    let height = area.height as usize;
    if width < 20 || height < 10 {
        return;
    }
    let mut buffer = vec![vec![SceneCell::default(); width]; height];
    let millis = current_millis();
    let tick = soulforge_ui.animation_tick;

    // Rapid cooling over first 8 ticks, then stays cold
    let cool_t = (tick as f64 / 8.0).min(1.0);
    let mut params = ForgeBackdropParams::normal();
    params.bottom_rgb = lerp_rgb((120, 40, 15), (40, 40, 45), cool_t);
    params.top_rgb = lerp_rgb((15, 8, 5), (15, 15, 18), cool_t);
    params.ember_count = (10.0 * (1.0 - cool_t * 0.6)) as usize;
    params.ember_speed = 5.0 * (1.0 - cool_t * 0.6);
    params.ember_hot = lerp_rgb((255, 160, 40), (80, 30, 10), cool_t);
    params.ember_cool = lerp_rgb((80, 20, 5), (30, 15, 10), cool_t);
    params.shimmer = cool_t < 0.5;
    // After full cooling, embers fall instead of rise
    if cool_t >= 1.0 {
        params.ember_upward = false;
    }
    paint_forge_backdrop(&mut buffer, millis, &params);

    // Crack characters appear at deterministic positions, spreading over time
    let crack_count = ((cool_t * 8.0) as usize).min(8);
    for i in 0..crack_count {
        let seed = hash2d(i + 200, 0);
        let row = (seed as usize) % height;
        let col = (hash2d(i + 200, 1) as usize) % width;
        put_cell(&mut buffer, row as i32, col as i32, '\u{2573}', Color::Rgb(80, 30, 30));
    }

    let result = soulforge_ui.last_result.as_ref().unwrap();

    let center_row = (height / 2).saturating_sub(3) as i32;

    // Crack border line
    let crack_line = " \u{2573}  \u{2573}  \u{2573}  \u{2573}  \u{2573} ";
    put_text_centered(&mut buffer, center_row, width, crack_line, Color::Red);

    // FAILED! text with shake
    let shake_offset = if tick < 5 {
        if tick.is_multiple_of(2) { 1i32 } else { -1i32 }
    } else {
        0
    };
    let failed_col = ((width as i32 - 7) / 2) + shake_offset;
    put_text(&mut buffer, center_row + 2, failed_col, "FAILED!", Color::Red);

    // Level change info
    let level_text = if result.old_level == result.new_level {
        format!("Enhancement failed! +{} (no change)", result.old_level)
    } else {
        format!(
            "Enhancement failed! +{} \u{2192} +{}",
            result.old_level, result.new_level
        )
    };
    put_text_centered(&mut buffer, center_row + 4, width, &level_text, Color::Red);

    // Bottom crack line
    put_text_centered(&mut buffer, center_row + 6, width, crack_line, Color::Red);

    // Continue prompt
    put_text_centered(&mut buffer, center_row + 8, width, "Press any key to continue", Color::DarkGray);

    render_buffer(frame, area, &buffer);
}
```

**Step 2: Verify build**

Run: `cargo build --all-targets 2>&1 | tail -5`

**Step 3: Commit**

```bash
git add src/ui/soulforge_scene.rs
git commit -m "feat(soulforge): convert failure phase with ash decay and crack effects"
```

---

### Task 8: Clean up unused imports and visual verification

**Files:**
- Modify: `src/ui/soulforge_scene.rs` — remove unused Ratatui imports

**Step 1: Remove unused imports**

After converting all 5 phase functions, several Ratatui imports are no longer needed. Remove unused items from the import block. The file should now import:

```rust
use crate::enhancement::{
    enhancement_cost, enhancement_multiplier, fail_penalty, success_rate, EnhancementProgress,
    SoulforgePhase, SoulforgeUiState, MAX_ENHANCEMENT_LEVEL,
};
use crate::items::EquipmentSlot;
use ratatui::{
    layout::Rect,
    style::Color,
    widgets::{Block, Borders, Clear},
    Frame,
};
use super::scene_fx::{current_millis, hash2d, lerp_rgb, put_cell, render_buffer, SceneCell};
```

The exact set of needed Ratatui imports depends on what `render_soulforge()` (entry point) and `render_soulforge_discovery_modal()` still use. Keep `Clear`, `Block`, `Borders` for those. Remove `Alignment`, `Constraint`, `Direction`, `Layout`, `Modifier`, `Style`, `Line`, `Span`, `Paragraph`, `Wrap` if they're only used by the discovery modal — check carefully.

**Step 2: Run full CI checks**

Run: `make check` in the worktree directory.
Expected: All checks pass (fmt, clippy, tests, build, audit).

If clippy warns about unused imports or dead code, fix them.

**Step 3: Visual verification**

Run the game with debug mode to test all Soulforge phases:

```bash
cargo run -- --debug
```

1. Press backtick (`` ` ``) to open debug menu
2. Select "Trigger Soulforge Discovery" to enable Soulforge
3. Press `S` to open Soulforge overlay
4. **Menu phase**: Verify forge backdrop with drifting embers behind slot list
5. **Confirming phase**: Select a slot, press Enter — verify intensified backdrop
6. **Hammering phase**: Press Enter to confirm — verify spark shower on hammer strikes, anvil glow, pulsing progress bar
7. **Success phase**: If enhancement succeeds — verify golden burst with sparkles
8. **Failure phase**: If enhancement fails (need +5 or higher) — verify ash decay with cracks

Iterate on visual tuning if needed (color values, ember count, spark angles, etc.).

**Step 4: Final commit**

```bash
git add src/ui/soulforge_scene.rs
git commit -m "refactor(soulforge): clean up unused imports after scene_fx conversion"
```
