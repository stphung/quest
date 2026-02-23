# Credits Screen Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a credits overlay accessible via `[C]` keybind showing contributors and technologies.

**Architecture:** New `GameOverlay::Credits` variant, a centered modal UI file (`credits_scene.rs`) following the same pattern as `help_overlay.rs`, input handling in the overlay priority chain and base game keybind.

**Tech Stack:** Ratatui (Block, Paragraph, Span, Clear), existing themed border system.

---

### Task 1: Add GameOverlay::Credits variant

**Files:**
- Modify: `src/input/types.rs:60-96` (GameOverlay enum)

**Step 1: Add the variant**

In `src/input/types.rs`, add `Credits` to the `GameOverlay` enum after `QuitConfirm`:

```rust
    /// Quit confirmation when pending challenges exist
    QuitConfirm,
    /// Credits screen overlay
    Credits,
    /// Bug report overlay with game state summary
    BugReport {
```

**Step 2: Verify it compiles**

Run: `cargo build 2>&1 | head -30`
Expected: Compiler warnings about non-exhaustive match patterns in `overlay.rs` and `input/mod.rs` (this is expected — we'll handle those in later tasks).

**Step 3: Commit**

```bash
git add src/input/types.rs
git commit -m "feat: add GameOverlay::Credits variant (#299)"
```

---

### Task 2: Create credits_scene.rs

**Files:**
- Create: `src/ui/credits_scene.rs`
- Modify: `src/ui/mod.rs:1` (add module declaration)

**Step 1: Create the credits scene file**

Create `src/ui/credits_scene.rs`:

```rust
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Draws the credits overlay as a centered modal dialog.
pub fn draw_credits_overlay(frame: &mut Frame) {
    let size = frame.area();

    let dialog_width = 44.min(size.width.saturating_sub(4));
    let dialog_height = 15.min(size.height.saturating_sub(4));

    let x = (size.width.saturating_sub(dialog_width)) / 2;
    let y = (size.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    frame.render_widget(Clear, dialog_area);

    let title = Line::from(vec![Span::styled(
        " Credits ",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]);

    let bold = Style::default().add_modifier(Modifier::BOLD);
    let dim = Style::default().fg(Color::DarkGray);

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("          "),
            Span::styled("\u{2694}  Q U E S T  \u{2694}", bold.fg(Color::Cyan)),
        ]),
        Line::from(Span::styled(
            "    A Terminal-Based Idle RPG",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled("  \u{2500}\u{2500}\u{2500} Forged By \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}", bold)),
        Line::from(vec![
            Span::raw("  Steven Phung (@stphung) "),
            Span::styled("Creator", dim),
        ]),
        Line::from(vec![
            Span::raw("  DH (@dhsu)         "),
            Span::styled("Contributor", dim),
        ]),
        Line::from(""),
        Line::from(Span::styled("  \u{2500}\u{2500}\u{2500} Built With \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}", bold)),
        Line::from("  Rust \u{00b7} Ratatui \u{00b7} Crossterm"),
        Line::from(""),
        Line::from(Span::styled("  [Esc] Close", dim)),
    ];

    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(super::themed_border_color(Color::Cyan)));
    let inner = super::render_themed_block(
        frame,
        dialog_area,
        block,
        Color::Cyan,
        super::BorderFxContext,
    );
    let paragraph = Paragraph::new(lines).alignment(Alignment::Left);

    frame.render_widget(paragraph, inner);
}
```

**Step 2: Register the module in mod.rs**

In `src/ui/mod.rs`, add after the `pub mod combat_effects;` line:

```rust
pub mod credits_scene;
```

**Step 3: Verify it compiles**

Run: `cargo build 2>&1 | head -30`
Expected: Compiles (may still warn about non-exhaustive matches from Task 1).

**Step 4: Commit**

```bash
git add src/ui/credits_scene.rs src/ui/mod.rs
git commit -m "feat: add credits overlay UI (#299)"
```

---

### Task 3: Wire up input handling

**Files:**
- Modify: `src/input/mod.rs:114-121` (overlay dismiss priority chain)
- Modify: `src/input/mod.rs:400-403` (base game keybind, near Help `?` handler)

**Step 1: Add Credits overlay dismiss handler**

In `src/input/mod.rs`, after the Help overlay handler (around line 116-121), add:

```rust
    // 0.76. Credits overlay
    if matches!(overlay, GameOverlay::Credits) {
        if matches!(key.code, KeyCode::Esc | KeyCode::Char('c') | KeyCode::Char('C')) {
            *overlay = GameOverlay::None;
        }
        return InputResult::Continue;
    }
```

**Step 2: Add `[C]` keybind in handle_base_game**

In `src/input/mod.rs` inside `handle_base_game`, after the `?` help handler (around line 400-403), add:

```rust
        KeyCode::Char('c') | KeyCode::Char('C') => {
            *overlay = GameOverlay::Credits;
            InputResult::Continue
        }
```

**Step 3: Verify it compiles**

Run: `cargo build 2>&1 | head -30`
Expected: May still warn about non-exhaustive match in `overlay.rs`.

**Step 4: Commit**

```bash
git add src/input/mod.rs
git commit -m "feat: wire up [C] keybind for credits overlay (#299)"
```

---

### Task 4: Wire up overlay rendering

**Files:**
- Modify: `src/main_helpers/overlay.rs:175-177` (match arm in draw_game_overlays)

**Step 1: Add Credits rendering arm**

In `src/main_helpers/overlay.rs`, in the `match overlay` block, add before the `GameOverlay::Help` arm:

```rust
        GameOverlay::Credits => {
            ui::credits_scene::draw_credits_overlay(frame);
        }
```

**Step 2: Verify it compiles cleanly**

Run: `cargo build 2>&1 | head -10`
Expected: Clean compile, no warnings about non-exhaustive matches.

**Step 3: Run full CI checks**

Run: `make check`
Expected: All checks pass (format, clippy, tests, build, audit).

**Step 4: Commit**

```bash
git add src/main_helpers/overlay.rs
git commit -m "feat: render credits overlay in game loop (#299)"
```

---

### Task 5: Update help overlay to mention credits keybind

**Files:**
- Modify: `src/ui/help_overlay.rs:39` (controls line)

**Step 1: Add [C] Credits to the controls section**

In `src/ui/help_overlay.rs`, change line 39:

```rust
        Line::from("  [U] Toggle Updates  [!] Report Bug  [Esc] Quit"),
```

to:

```rust
        Line::from("  [U] Toggle Updates  [C] Credits  [!] Bug  [Esc] Quit"),
```

**Step 2: Run full CI checks**

Run: `make check`
Expected: All checks pass.

**Step 3: Commit**

```bash
git add src/ui/help_overlay.rs
git commit -m "feat: add credits keybind to help overlay (#299)"
```
