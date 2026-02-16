# Wiki Link Feature Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Surface the wiki URL (`github.com/stphung/quest/wiki`) to players in three contextual locations: a Help overlay (`?` key), the character select controls bar, and the update drawer.

**Architecture:** Add a `WIKI_URL` constant, a new `GameOverlay::Help` variant, a new `help_overlay.rs` rendering file following the `prestige_confirm.rs` pattern, and wire up input routing for `?` in both game and character select screens.

**Tech Stack:** Rust, Ratatui (Paragraph, Block, Clear widgets)

---

### Task 1: Add WIKI_URL constant

**Files:**
- Modify: `src/core/constants.rs:185` (end of file)

**Step 1: Add the constant**

Append at end of `src/core/constants.rs`:

```rust
// Wiki
pub const WIKI_URL: &str = "github.com/stphung/quest/wiki";
```

**Step 2: Verify it compiles**

Run: `cargo build 2>&1 | tail -5`
Expected: compiles with no errors

**Step 3: Commit**

```bash
git add src/core/constants.rs
git commit -m "feat: add WIKI_URL constant"
```

---

### Task 2: Add Help variant to GameOverlay enum

**Files:**
- Modify: `src/input.rs:90-113` (GameOverlay enum)

**Step 1: Add Help variant**

Add `Help,` after `None,` in the `GameOverlay` enum at `src/input.rs:91`:

```rust
pub enum GameOverlay {
    None,
    Help,
    HavenDiscovery,
    // ... rest unchanged
}
```

**Step 2: Verify it compiles**

Run: `cargo build 2>&1 | tail -5`
Expected: compiles (Help variant is unused but that's OK — no exhaustive match warnings because the existing `GameOverlay::None => {}` catch-all handles it)

Wait — the overlay dispatch in `main.rs:182` uses a `match` on `GameOverlay`. We need to add a `Help` arm there too, but we'll do that in Task 3 after creating the rendering function.

**Step 3: Check if there's a compile error from the match**

Run: `cargo build 2>&1 | tail -20`

If the match in `main.rs` is non-exhaustive, add a temporary placeholder:

In `main.rs` `draw_overlays()` function, add before the `GameOverlay::None => {}` arm:

```rust
GameOverlay::Help => {}
```

This will be replaced in Task 4 when we wire up the rendering.

**Step 4: Commit**

```bash
git add src/input.rs src/main.rs
git commit -m "feat: add Help variant to GameOverlay enum"
```

---

### Task 3: Create help overlay rendering

**Files:**
- Create: `src/ui/help_overlay.rs`

**Step 1: Create the help overlay file**

Create `src/ui/help_overlay.rs` following the `prestige_confirm.rs` pattern (centered `Clear` + bordered `Paragraph`):

```rust
use crate::core::constants::WIKI_URL;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Draws the help overlay as a centered modal dialog.
pub fn draw_help_overlay(frame: &mut Frame) {
    let size = frame.area();

    // Calculate dialog size and position (centered)
    let dialog_width = 56.min(size.width.saturating_sub(4));
    let dialog_height = 14.min(size.height.saturating_sub(4));

    let x = (size.width.saturating_sub(dialog_width)) / 2;
    let y = (size.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    // Clear the area behind the dialog
    frame.render_widget(Clear, dialog_area);

    let title = Line::from(vec![Span::styled(
        " Help ",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Controls",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  [P] Prestige    [H] Haven    [S] Soulforge"),
        Line::from("  [A] Achievements    [Tab] Challenges"),
        Line::from("  [U] Toggle Updates    [Esc] Quit"),
        Line::from(""),
        Line::from(Span::styled(
            "  Quest Wiki",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("  {}", WIKI_URL),
            Style::default().fg(Color::Cyan),
        )),
        Line::from("  Guides for combat, zones, prestige, and more."),
        Line::from(""),
        Line::from(Span::styled(
            "  [Esc] Close",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(title)
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .alignment(Alignment::Left);

    frame.render_widget(paragraph, dialog_area);
}
```

**Step 2: Register module in `src/ui/mod.rs`**

Add to the module declarations at the top of `src/ui/mod.rs` (alphabetical order, after `pub mod gomoku_scene;`):

```rust
pub mod help_overlay;
```

**Step 3: Verify it compiles**

Run: `cargo build 2>&1 | tail -5`
Expected: compiles with no errors

**Step 4: Commit**

```bash
git add src/ui/help_overlay.rs src/ui/mod.rs
git commit -m "feat: add help overlay rendering"
```

---

### Task 4: Wire up Help overlay rendering in main.rs

**Files:**
- Modify: `src/main.rs:182-236` (draw_overlays match)

**Step 1: Replace the placeholder Help arm**

In `main.rs`, find the `draw_overlays()` function's match statement. Replace the `GameOverlay::Help => {}` placeholder with:

```rust
GameOverlay::Help => {
    ui::help_overlay::draw_help_overlay(frame);
}
```

**Step 2: Verify it compiles**

Run: `cargo build 2>&1 | tail -5`
Expected: compiles with no errors

**Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: wire up help overlay rendering in draw_overlays"
```

---

### Task 5: Add `?` input routing for main game screen

**Files:**
- Modify: `src/input.rs:144-250` (handle_game_input) and `src/input.rs:772-823` (handle_base_game)

**Step 1: Add Help overlay input handling**

In `handle_game_input()`, add a new block after the Achievements handler (after line 173, before Haven discovery at line 176) to handle the Help overlay:

```rust
// 0.75. Help overlay
if matches!(overlay, GameOverlay::Help) {
    if matches!(key.code, KeyCode::Esc | KeyCode::Char('?')) {
        *overlay = GameOverlay::None;
    }
    return InputResult::Continue;
}
```

**Step 2: Add `?` key to base game handler**

In `handle_base_game()`, add a new match arm before the catch-all `_ => InputResult::Continue`:

```rust
KeyCode::Char('?') => {
    *overlay = GameOverlay::Help;
    InputResult::Continue
}
```

**Step 3: Verify it compiles**

Run: `cargo build 2>&1 | tail -5`
Expected: compiles with no errors

**Step 4: Commit**

```bash
git add src/input.rs
git commit -m "feat: add ? key to open/close help overlay in game"
```

---

### Task 6: Add `?` input routing for character select screen

**Files:**
- Modify: `src/main.rs:557-625` (Screen::CharacterSelect input handling)

**Step 1: Add help overlay state variable**

In `main.rs`, near the other screen state variables (around line 437-441), add:

```rust
let mut help_overlay_showing = false;
```

**Step 2: Add help overlay rendering to character select draw**

In the `Screen::CharacterSelect` draw closure (around line 517-555), add after the Soulforge overlay rendering:

```rust
// Draw Help overlay if open
if help_overlay_showing {
    ui::help_overlay::draw_help_overlay(f);
}
```

**Step 3: Add help overlay input blocking**

In the character select input handling (around line 558-625), add a new block after the Soulforge overlay handler and before the achievement browser handler:

```rust
// Handle Help overlay (blocks other input when open)
if help_overlay_showing {
    if matches!(key_event.code, KeyCode::Esc | KeyCode::Char('?')) {
        help_overlay_showing = false;
    }
    continue;
}
```

**Step 4: Add `?` keybinding**

In the character select input handling, add a new block after the achievement browser shortcut (after line 612) and before the `let input = match key_event.code` block:

```rust
// Help overlay shortcut
if key_event.code == KeyCode::Char('?') {
    help_overlay_showing = true;
    continue;
}
```

**Step 5: Reset help overlay on screen transition**

When transitioning away from character select to the game (where `SelectResult::LoadCharacter` is handled), reset the help overlay state:

```rust
help_overlay_showing = false;
```

Add this near the existing state resets when a character is loaded (around line 631).

**Step 6: Verify it compiles**

Run: `cargo build 2>&1 | tail -5`
Expected: compiles with no errors

**Step 7: Commit**

```bash
git add src/main.rs
git commit -m "feat: add ? key to open help overlay from character select"
```

---

### Task 7: Add `[?] Help` to character select controls bar

**Files:**
- Modify: `src/ui/character_select.rs:211-258` (draw_controls method)

**Step 1: Add `[?] Help` to the controls**

In `draw_controls()`, modify the second line of controls for both compact and non-compact modes.

For the **compact** variant (line 225-236), change the second line from:
```rust
Line::from(Span::styled(
    "[A] Achv",
    ...
)),
```
to:
```rust
Line::from(vec![
    Span::styled(
        "[A] Achv",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ),
    Span::styled(
        "  [?] Help",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ),
]),
```

For the **non-compact** variant (line 242-257), change the second line from:
```rust
Line::from(Span::styled(
    "[A] Achievements",
    ...
)),
```
to:
```rust
Line::from(vec![
    Span::styled(
        "[A] Achievements",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ),
    Span::styled(
        "    [?] Help",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ),
]),
```

**Step 2: Verify it compiles**

Run: `cargo build 2>&1 | tail -5`
Expected: compiles with no errors

**Step 3: Commit**

```bash
git add src/ui/character_select.rs
git commit -m "feat: add [?] Help to character select controls bar"
```

---

### Task 8: Add wiki URL to update drawer

**Files:**
- Modify: `src/ui/stats_panel.rs:883-891` (draw_update_drawer footer section)

**Step 1: Add wiki URL line**

In `draw_update_drawer()`, find the footer section (around line 883-891). After the "Run 'quest update' to install" line and before the `[U] Close` span, add the wiki URL. Replace the existing footer `lines.push(...)` block:

```rust
// Add empty line and footer
lines.push(Line::from(vec![]));
lines.push(Line::from(vec![Span::styled(
    format!("  Run 'quest update' to install"),
    Style::default().fg(Color::DarkGray),
)]));
lines.push(Line::from(vec![
    Span::styled(
        format!("  Wiki: {}", crate::core::constants::WIKI_URL),
        Style::default().fg(Color::DarkGray),
    ),
    Span::raw("                              "),
    Span::styled("[U] Close", Style::default().fg(Color::Yellow)),
]));
```

This replaces the existing lines 883-891 which currently combine the "Run 'quest update'" text with the `[U] Close` button on one line. Now the wiki URL sits on its own line between them.

**Step 2: Verify it compiles**

Run: `cargo build 2>&1 | tail -5`
Expected: compiles with no errors

**Step 3: Commit**

```bash
git add src/ui/stats_panel.rs
git commit -m "feat: add wiki URL to update drawer"
```

---

### Task 9: Final verification

**Step 1: Run full CI checks**

Run: `make check`
Expected: All checks pass (format, clippy, tests, build, audit)

**Step 2: Fix any clippy or formatting issues**

Run: `make fmt` if formatting fails, then re-run `make check`.

**Step 3: Create final commit if any fixups needed**

```bash
git add -A
git commit -m "fix: address clippy/fmt issues from wiki link feature"
```

**Step 4: Verify the feature end-to-end**

Run: `cargo run` and:
1. On character select screen, verify `[?] Help` appears in controls
2. Press `?` — verify help overlay with controls + wiki URL appears
3. Press `Esc` or `?` — verify overlay dismisses
4. Start a game, press `?` — verify help overlay appears in-game
5. If update drawer is visible, verify wiki URL line appears
