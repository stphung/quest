> Backported implementation plan (completed — this work shipped).

## 2026-02-21-sigil-roll-animation-plan.md

# Sigil Roll Animation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a 4-phase rune inscription ritual animation when rolling for sigils in the Stormglass Exchange.

**Architecture:** New `SigilRolling` variant in `ExchangePhase` with `current_millis()`-based timing. Choices are pre-generated before animation starts (cosmetic only). Animation renders in the existing scene buffer system. Any key after 200ms skips to `SigilPick`.

**Tech Stack:** Rust, Ratatui scene buffer (`scene_fx.rs`), existing `stormglass_scene.rs` rendering patterns.

---

### Task 1: Add SigilRolling State and Fields

**Files:**
- Modify: `src/stormglass/types.rs:44-58` (ExchangePhase enum)
- Modify: `src/stormglass/types.rs:69-82` (ExchangeUiState struct)
- Modify: `src/stormglass/types.rs:86-113` (new() and open() methods)
- Modify: `src/stormglass/types.rs:115-126` (close() method)

**Step 1: Add `SigilRolling` variant to `ExchangePhase`**

In `src/stormglass/types.rs`, add `SigilRolling` after `SigilRerollConfirm`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExchangePhase {
    Menu,
    InvokeTrialConfirm,
    InvokeTrial,
    InvokeTrialForfeitConfirm,
    ChronoSurge,
    // Storm Sigils phases
    SigilsList,
    SigilUnlockConfirm,
    SigilInscribeConfirm,
    SigilRerollConfirm,
    SigilRolling,       // NEW: animation phase
    SigilPick,
    SigilForfeitConfirm,
    SigilResult,
}
```

**Step 2: Add animation fields to `ExchangeUiState`**

Add two fields after `sigil_target_slot`:

```rust
pub struct ExchangeUiState {
    // ... existing fields ...
    pub sigil_target_slot: usize,
    pub sigil_animation_start_ms: Option<u128>,  // NEW
    pub sigil_animation_skipped: bool,            // NEW
}
```

**Step 3: Initialize new fields in `new()`, `open()`, and `close()`**

In `new()`:
```rust
sigil_animation_start_ms: None,
sigil_animation_skipped: false,
```

In `open()`:
```rust
self.sigil_animation_start_ms = None;
self.sigil_animation_skipped = false;
```

In `close()`:
```rust
self.sigil_animation_start_ms = None;
self.sigil_animation_skipped = false;
```

**Step 4: Build and verify**

Run: `cargo build`
Expected: Compiler errors in `stormglass_input.rs` and `stormglass_scene.rs` about non-exhaustive match on `ExchangePhase` — that's expected, we fix those in Tasks 2 and 3.

**Step 5: Commit**

```bash
git add src/stormglass/types.rs
git commit -m "feat(sigils): add SigilRolling phase and animation state fields"
```

---

### Task 2: Wire Up Input Handling for SigilRolling

**Files:**
- Modify: `src/input/stormglass_input.rs:19-36` (main match dispatch)
- Modify: `src/input/stormglass_input.rs:293-317` (handle_sigil_inscribe_confirm)
- Modify: `src/input/stormglass_input.rs:319-346` (handle_sigil_reroll_confirm)
- Add new function: `handle_sigil_rolling` at bottom of file

**Step 1: Add `SigilRolling` arm to the main dispatch match**

In `handle_stormglass_exchange()`, add a new arm after `SigilRerollConfirm`:

```rust
ExchangePhase::SigilRolling => handle_sigil_rolling(key, exchange_ui),
```

**Step 2: Change inscribe confirm to transition to SigilRolling instead of SigilPick**

In `handle_sigil_inscribe_confirm`, change the `'y'` handler (lines 299-308). Replace:
```rust
exchange_ui.phase = ExchangePhase::SigilPick;
```
with:
```rust
exchange_ui.sigil_animation_start_ms =
    Some(crate::ui::scene_fx::current_millis());
exchange_ui.sigil_animation_skipped = false;
exchange_ui.phase = ExchangePhase::SigilRolling;
```

**Step 3: Change reroll confirm to transition to SigilRolling instead of SigilPick**

In `handle_sigil_reroll_confirm`, change the `'y'` handler (lines 325-337). Replace:
```rust
exchange_ui.phase = ExchangePhase::SigilPick;
```
with:
```rust
exchange_ui.sigil_animation_start_ms =
    Some(crate::ui::scene_fx::current_millis());
exchange_ui.sigil_animation_skipped = false;
exchange_ui.phase = ExchangePhase::SigilRolling;
```

**Step 4: Add the `handle_sigil_rolling` function**

At the bottom of the file, add:

```rust
/// Minimum display time before skip is accepted (prevents key-repeat instant-skip).
const SIGIL_ANIMATION_SKIP_FLOOR_MS: u128 = 200;

/// Total animation duration before auto-transitioning to pick.
const SIGIL_ANIMATION_DURATION_MS: u128 = 4000;

fn handle_sigil_rolling(key: KeyEvent, exchange_ui: &mut ExchangeUiState) -> InputResult {
    // Any key skips after the minimum display floor
    let elapsed = exchange_ui
        .sigil_animation_start_ms
        .map(|start| crate::ui::scene_fx::current_millis().saturating_sub(start))
        .unwrap_or(0);

    if elapsed >= SIGIL_ANIMATION_SKIP_FLOOR_MS {
        // Skip to pick phase
        exchange_ui.sigil_animation_start_ms = None;
        exchange_ui.sigil_animation_skipped = false;
        exchange_ui.phase = ExchangePhase::SigilPick;
    }
    // Before the floor, ignore the keypress
    InputResult::Continue
}
```

**Step 5: Add auto-transition check for animation timeout**

The animation also needs to auto-transition when 4000ms elapses with no keypress. This is handled in the render function (Task 3) — when elapsed >= `SIGIL_ANIMATION_DURATION_MS`, the render function transitions the phase. However, since `stormglass_scene.rs` only has read access to `exchange_ui` via `&ExchangeUiState`, we need to handle the timeout in the input layer.

Add a public function that `main.rs` can call each tick to check for animation completion:

```rust
/// Check if sigil animation has completed and should auto-transition to pick.
/// Called from the main game loop each tick.
pub fn check_sigil_animation_timeout(exchange_ui: &mut ExchangeUiState) {
    if exchange_ui.phase != ExchangePhase::SigilRolling {
        return;
    }
    let elapsed = exchange_ui
        .sigil_animation_start_ms
        .map(|start| crate::ui::scene_fx::current_millis().saturating_sub(start))
        .unwrap_or(0);

    if elapsed >= SIGIL_ANIMATION_DURATION_MS {
        exchange_ui.sigil_animation_start_ms = None;
        exchange_ui.sigil_animation_skipped = false;
        exchange_ui.phase = ExchangePhase::SigilPick;
    }
}
```

**Step 6: Call the timeout check from `main.rs`**

In `src/main.rs`, find where `exchange_ui` is used in the main loop (near line 339 or the tick/draw section). Add after the tick processing and before rendering:

```rust
crate::input::stormglass_input::check_sigil_animation_timeout(&mut exchange_ui);
```

Note: `stormglass_input.rs` is currently a private module (`mod stormglass_input;` in `input/mod.rs`). You'll need to either:
- Make `check_sigil_animation_timeout` accessible by re-exporting it from `input/mod.rs`, or
- Move the timeout check function to a public location.

Add to `src/input/mod.rs`:
```rust
pub use stormglass_input::check_sigil_animation_timeout;
```

Then in `main.rs`:
```rust
crate::input::check_sigil_animation_timeout(&mut exchange_ui);
```

**Step 7: Build and run tests**

Run: `cargo build && cargo test`
Expected: Build succeeds. Tests pass. The only remaining compiler error should be a non-exhaustive match in `stormglass_scene.rs` (fixed in Task 3).

**Step 8: Commit**

```bash
git add src/input/stormglass_input.rs src/input/mod.rs src/main.rs
git commit -m "feat(sigils): wire SigilRolling input handling and auto-transition"
```

---

### Task 3: Implement Animation Rendering

**Files:**
- Modify: `src/ui/stormglass_scene.rs:127-149` (render dispatch match)
- Add new function: `render_sigil_rolling` in `stormglass_scene.rs`

**Step 1: Add `SigilRolling` arm to the render dispatch**

In `render_stormglass_exchange()`, add after the `SigilRerollConfirm` arm:

```rust
ExchangePhase::SigilRolling => render_sigil_rolling(frame, area, exchange_ui),
```

**Step 2: Add animation constants at the top of the file**

Near the existing `ELECTRIC_BLUE` constant:

```rust
/// Rune circle purple color.
const RUNE_PURPLE: Color = Color::Rgb(180, 160, 255);
/// Inscription flash color (warm white).
const INSCRIPTION_FLASH: Color = Color::Rgb(255, 255, 200);
/// Phase timing boundaries in milliseconds.
const PHASE1_END_MS: u128 = 1400;
const PHASE2_END_MS: u128 = 2800;
const PHASE3_END_MS: u128 = 3500;
const PHASE4_END_MS: u128 = 4000;
```

**Step 3: Implement `render_sigil_rolling`**

Add this function. It computes elapsed time from `exchange_ui.sigil_animation_start_ms`, determines the current sub-phase, and renders accordingly.

```rust
/// Render the sigil rolling animation (4 sub-phases).
fn render_sigil_rolling(
    frame: &mut Frame,
    area: Rect,
    exchange_ui: &ExchangeUiState,
) {
    let overlay_width = 52u16.min(area.width.saturating_sub(4));
    let overlay_height = 16u16.min(area.height.saturating_sub(2));
    let x = area.x + (area.width.saturating_sub(overlay_width)) / 2;
    let y = area.y + (area.height.saturating_sub(overlay_height)) / 2;
    let overlay_area = Rect::new(x, y, overlay_width, overlay_height);

    frame.render_widget(Clear, overlay_area);

    let block = Block::default()
        .title(Line::from(Span::styled(
            " \u{16B1} Inscribing Sigils... \u{16B1} ",
            Style::default()
                .fg(ELECTRIC_BLUE)
                .add_modifier(Modifier::BOLD),
        )))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ELECTRIC_BLUE));

    let inner = block.inner(overlay_area);
    frame.render_widget(block, overlay_area);

    let w = inner.width as usize;
    let h = inner.height as usize;
    if w == 0 || h == 0 {
        return;
    }

    let mut buffer = vec![vec![SceneCell::default(); w]; h];
    let now = current_millis();
    let elapsed = exchange_ui
        .sigil_animation_start_ms
        .map(|start| now.saturating_sub(start))
        .unwrap_or(0);

    // Intensified storm backdrop during animation
    let intensity = (elapsed as f64 / PHASE1_END_MS as f64).min(1.0);
    let params = StormBackdropParams {
        top_rgb: lerp_rgb((10, 15, 40), (20, 30, 70), intensity),
        bottom_rgb: (5, 5, 15),
        particle_count: 8 + (intensity * 12.0) as usize,
        particle_speed: 1.5 + intensity * 2.0,
        shimmer: true,
    };
    paint_storm_backdrop(&mut buffer, now, &params);

    // Clear rows for content
    for i in 0..h {
        clear_row_chars(&mut buffer, i as i32);
    }

    if elapsed < PHASE1_END_MS {
        // Phase 1: Energy Gathering — rune circle builds in center
        render_energy_gathering(&mut buffer, w, h, elapsed, now);
    } else if elapsed < PHASE2_END_MS {
        // Phase 2: Sigil Etching — names etch character-by-character
        render_sigil_etching(&mut buffer, w, h, elapsed, exchange_ui);
    } else if elapsed < PHASE3_END_MS {
        // Phase 3: Grade Reveal — grade labels fade in
        render_grade_reveal(&mut buffer, w, h, elapsed, exchange_ui);
    } else {
        // Phase 4: Transition — all solidified with cursor
        render_animation_transition(&mut buffer, w, h, exchange_ui);
    }

    // Help row at bottom
    let help_row = (h as i32) - 1;
    let help_text = if elapsed >= 200 {
        "[Any key] Skip"
    } else {
        ""
    };
    put_text_centered(&mut buffer, help_row, w, help_text, Color::DarkGray);

    render_buffer(frame, inner, &buffer);
}
```

**Step 4: Implement the 4 sub-phase render helpers**

Add these helper functions below `render_sigil_rolling`:

```rust
/// Phase 1: Energy gathering — rune circle builds, particles orbit.
fn render_energy_gathering(
    buffer: &mut [Vec<SceneCell>],
    w: usize,
    h: usize,
    elapsed: u128,
    now: u128,
) {
    let center_row = h as i32 / 2;
    let center_col = w as i32 / 2;
    let progress = elapsed as f64 / PHASE1_END_MS as f64;

    // Progressive rune circle character
    let circle_ch = match (progress * 4.0) as usize {
        0 => '\u{00b7}', // ·
        1 => '\u{25e6}', // ◦
        2 => '\u{25cb}', // ○
        _ => '\u{25ce}', // ◎
    };

    // Rune circle: ring of characters around center
    let radius = 2.0 + progress * 2.0;
    let num_points = 8 + (progress * 8.0) as usize;
    for i in 0..num_points {
        let angle = (i as f64 / num_points as f64) * std::f64::consts::TAU
            + (now as f64 / 1000.0); // slow rotation
        let row = center_row + (angle.sin() * radius) as i32;
        let col = center_col + (angle.cos() * radius * 2.0) as i32; // 2x for char aspect ratio
        put_cell(buffer, row, col, circle_ch, RUNE_PURPLE);
    }

    // Center text: "Gathering energy..."
    let phase_text = match (progress * 3.0) as usize {
        0 => ".",
        1 => "..",
        _ => "...",
    };
    let text = format!("Gathering energy{}", phase_text);
    let pulse_t = ((now as f64 / 300.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let text_rgb = lerp_rgb((120, 100, 200), (200, 180, 255), pulse_t);
    put_text_centered(
        buffer,
        center_row,
        w,
        &text,
        Color::Rgb(text_rgb.0, text_rgb.1, text_rgb.2),
    );

    // Orbiting glyphs converging toward center
    let orbit_chars = ['\u{26a1}', '\u{2726}', '\u{25c8}']; // ⚡ ✦ ◈
    let orbit_radius = radius + 3.0 - progress * 2.0; // converges inward
    for (i, &ch) in orbit_chars.iter().enumerate() {
        let angle = (i as f64 / orbit_chars.len() as f64) * std::f64::consts::TAU
            + (now as f64 / 500.0);
        let row = center_row + (angle.sin() * orbit_radius) as i32;
        let col = center_col + (angle.cos() * orbit_radius * 2.0) as i32;
        put_cell(buffer, row, col, ch, ELECTRIC_BLUE);
    }
}

/// Phase 2: Sigil etching — names etch character-by-character.
fn render_sigil_etching(
    buffer: &mut [Vec<SceneCell>],
    w: usize,
    h: usize,
    elapsed: u128,
    exchange_ui: &ExchangeUiState,
) {
    let phase_elapsed = elapsed.saturating_sub(PHASE1_END_MS);
    let phase_duration = PHASE2_END_MS - PHASE1_END_MS; // 1400ms
    let stagger_ms: u128 = 150; // stagger between sigils

    let center_row = h as i32 / 2;

    for (i, choice) in exchange_ui.sigil_choices.iter().enumerate() {
        let Some(sigil) = choice else { continue };
        let name = sigil.effect.format_value(sigil.value);
        let sigil_start = i as u128 * stagger_ms;
        let sigil_elapsed = phase_elapsed.saturating_sub(sigil_start);

        // Calculate how many characters to show
        let chars_per_ms = name.chars().count() as f64
            / (phase_duration.saturating_sub(stagger_ms * 2)) as f64;
        let chars_to_show =
            (sigil_elapsed as f64 * chars_per_ms).min(name.chars().count() as f64) as usize;

        let row = center_row - 1 + i as i32;
        let partial: String = name.chars().take(chars_to_show).collect();

        // Check if this sigil just completed etching (flash effect)
        let just_completed = chars_to_show == name.chars().count()
            && sigil_elapsed < (phase_duration.saturating_sub(sigil_start) + 100);
        let fg = if just_completed
            && (sigil_elapsed % 200) < 100
        {
            INSCRIPTION_FLASH
        } else {
            Color::White
        };

        let col = ((w as i32) - name.chars().count() as i32) / 2;
        put_text(buffer, row, col, &partial, fg);

        // Typing cursor at end of partial text
        if chars_to_show < name.chars().count() {
            let cursor_col = col + chars_to_show as i32;
            put_cell(buffer, row, cursor_col, '\u{2588}', RUNE_PURPLE); // █
        }
    }
}

/// Phase 3: Grade reveal — grade labels fade in below names.
fn render_grade_reveal(
    buffer: &mut [Vec<SceneCell>],
    w: usize,
    h: usize,
    elapsed: u128,
    exchange_ui: &ExchangeUiState,
) {
    let phase_elapsed = elapsed.saturating_sub(PHASE2_END_MS);
    let phase_duration = PHASE3_END_MS - PHASE2_END_MS; // 700ms
    let reveal_progress = (phase_elapsed as f64 / phase_duration as f64).min(1.0);

    let center_row = h as i32 / 2;

    for (i, choice) in exchange_ui.sigil_choices.iter().enumerate() {
        let Some(sigil) = choice else { continue };
        let name = sigil.effect.format_value(sigil.value);
        let row = center_row - 1 + i as i32;

        // Full name (already etched)
        let col = ((w as i32) - name.chars().count() as i32) / 2;
        put_text(buffer, row, col, &name, Color::White);

        // Grade label fades in with stagger
        let grade_stagger = i as f64 * 0.15; // 15% stagger per sigil
        let grade_progress =
            ((reveal_progress - grade_stagger) / (1.0 - grade_stagger * 2.0)).clamp(0.0, 1.0);

        if grade_progress > 0.0 {
            let grade_str = sigil.grade.label();
            let grade_fg = sigil_grade_color(sigil.grade);

            // Fade in: lerp from dark to grade color
            let (r, g, b) = match grade_fg {
                Color::Rgb(r, g, b) => (r, g, b),
                Color::Green => (0, 128, 0),
                Color::Cyan => (0, 128, 128),
                Color::White => (200, 200, 200),
                Color::Gray => (128, 128, 128),
                Color::DarkGray => (80, 80, 80),
                Color::Red => (200, 0, 0),
                _ => (200, 200, 200),
            };
            let faded = lerp_rgb((20, 20, 30), (r, g, b), grade_progress);
            let fg = Color::Rgb(faded.0, faded.1, faded.2);

            // Place grade right-aligned on the same row
            let grade_col = col + name.chars().count() as i32 + 2;
            put_text(buffer, row, grade_col, grade_str, fg);
        }
    }
}

/// Phase 4: Transition — solidified with cursor on first choice.
fn render_animation_transition(
    buffer: &mut [Vec<SceneCell>],
    w: usize,
    h: usize,
    exchange_ui: &ExchangeUiState,
) {
    let center_row = h as i32 / 2;

    for (i, choice) in exchange_ui.sigil_choices.iter().enumerate() {
        let Some(sigil) = choice else { continue };
        let name = sigil.effect.format_value(sigil.value);
        let row = center_row - 1 + i as i32;

        let col = ((w as i32) - name.chars().count() as i32) / 2;
        put_text(buffer, row, col, &name, Color::White);

        // Grade label fully visible
        let grade_str = sigil.grade.label();
        let grade_fg = sigil_grade_color(sigil.grade);
        let grade_col = col + name.chars().count() as i32 + 2;
        put_text(buffer, row, grade_col, grade_str, grade_fg);

        // Cursor on first choice
        if i == 0 {
            put_text(buffer, row, col - 3, "> ", Color::Yellow);
        }
    }
}
```

**Step 5: Build and run tests**

Run: `cargo build && cargo test`
Expected: All tests pass. No compiler errors.

**Step 6: Commit**

```bash
git add src/ui/stormglass_scene.rs
git commit -m "feat(sigils): implement 4-phase rune inscription animation"
```

---

### Task 4: Add Integration Tests

**Files:**
- Modify: `tests/storm_sigils_test.rs`

**Step 1: Add state machine transition tests**

Add tests that verify the `ExchangePhase` transitions work correctly:

```rust
// ── Animation State Machine Tests ──────────────────────────────────────

#[test]
fn test_sigil_rolling_phase_exists() {
    use quest::stormglass::types::ExchangePhase;
    let phase = ExchangePhase::SigilRolling;
    assert_eq!(phase, ExchangePhase::SigilRolling);
}

#[test]
fn test_exchange_ui_animation_fields_default() {
    use quest::stormglass::types::ExchangeUiState;
    let ui = ExchangeUiState::new();
    assert!(ui.sigil_animation_start_ms.is_none());
    assert!(!ui.sigil_animation_skipped);
}

#[test]
fn test_exchange_ui_open_resets_animation_fields() {
    use quest::stormglass::types::ExchangeUiState;
    let mut ui = ExchangeUiState::new();
    ui.sigil_animation_start_ms = Some(12345);
    ui.sigil_animation_skipped = true;
    ui.open();
    assert!(ui.sigil_animation_start_ms.is_none());
    assert!(!ui.sigil_animation_skipped);
}

#[test]
fn test_exchange_ui_close_resets_animation_fields() {
    use quest::stormglass::types::ExchangeUiState;
    let mut ui = ExchangeUiState::new();
    ui.sigil_animation_start_ms = Some(99999);
    ui.sigil_animation_skipped = true;
    ui.close();
    assert!(ui.sigil_animation_start_ms.is_none());
    assert!(!ui.sigil_animation_skipped);
}
```

**Step 2: Add timeout transition test**

```rust
#[test]
fn test_check_sigil_animation_timeout_transitions_after_duration() {
    use quest::input::check_sigil_animation_timeout;
    use quest::stormglass::types::{ExchangePhase, ExchangeUiState};
    use quest::ui::scene_fx::current_millis;

    let mut ui = ExchangeUiState::new();
    ui.phase = ExchangePhase::SigilRolling;
    // Set start time far in the past (>4000ms ago)
    ui.sigil_animation_start_ms = Some(current_millis().saturating_sub(5000));

    check_sigil_animation_timeout(&mut ui);
    assert_eq!(ui.phase, ExchangePhase::SigilPick);
    assert!(ui.sigil_animation_start_ms.is_none());
}

#[test]
fn test_check_sigil_animation_timeout_no_op_when_not_rolling() {
    use quest::input::check_sigil_animation_timeout;
    use quest::stormglass::types::{ExchangePhase, ExchangeUiState};

    let mut ui = ExchangeUiState::new();
    ui.phase = ExchangePhase::SigilPick;

    check_sigil_animation_timeout(&mut ui);
    // Phase unchanged
    assert_eq!(ui.phase, ExchangePhase::SigilPick);
}

#[test]
fn test_check_sigil_animation_timeout_no_transition_when_recent() {
    use quest::input::check_sigil_animation_timeout;
    use quest::stormglass::types::{ExchangePhase, ExchangeUiState};
    use quest::ui::scene_fx::current_millis;

    let mut ui = ExchangeUiState::new();
    ui.phase = ExchangePhase::SigilRolling;
    // Set start time to now (0ms elapsed)
    ui.sigil_animation_start_ms = Some(current_millis());

    check_sigil_animation_timeout(&mut ui);
    // Should NOT transition — animation still running
    assert_eq!(ui.phase, ExchangePhase::SigilRolling);
}
```

**Step 3: Run tests**

Run: `cargo test --test storm_sigils_test`
Expected: All tests pass.

**Step 4: Run full test suite**

Run: `cargo test`
Expected: All ~1394 tests pass.

**Step 5: Commit**

```bash
git add tests/storm_sigils_test.rs
git commit -m "test(sigils): add animation state machine integration tests"
```

---

### Task 5: Final Verification

**Step 1: Run full CI checks**

Run: `make check`
Expected: All checks pass (format, clippy, tests, build, audit).

**Step 2: Manual testing**

Run: `cargo run -- --debug`

1. Open Stormglass Exchange (G key)
2. Navigate to Storm Sigils
3. Select an empty slot and inscribe
4. Confirm with Y
5. Verify: animation plays (~4 seconds) with energy gathering, sigil etching, grade reveal, then transitions to pick screen
6. Repeat and press a key during animation — verify it skips to pick screen
7. Verify pick screen works normally after skip
8. Test reroll path: inscribe a sigil, then reroll it — animation should play again

**Step 3: Verify skip floor**

1. Hold Enter on the confirm dialog — animation should NOT be instantly skipped
2. Release, wait 200ms, press any key — should skip

**Step 4: Commit any fixes if needed**

If manual testing reveals issues, fix and commit.
