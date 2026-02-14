# Achievement "NEW" Badges Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Show yellow star badges on recently unlocked achievements in the browser so users can find what's new.

**Architecture:** Add a transient `recently_unlocked` field to `Achievements` that's populated when the browser opens (from `pending_notifications`) and cleared when the browser closes. The browser rendering checks this field for star prefixes and category tab badges.

**Tech Stack:** Rust, Ratatui (rendering), existing achievement system

---

## Task 1: Add `recently_unlocked` Field and Methods

**Files:**
- Modify: `src/achievements/types.rs:203-237` (struct fields)
- Modify: `src/achievements/types.rs:275-277` (clear_pending_notifications)
- Test: `src/achievements/types.rs` (inline tests)

**Step 1: Write failing tests**

Add these tests at the end of the `mod tests` block in `src/achievements/types.rs`:

```rust
#[test]
fn test_clear_pending_moves_to_recently_unlocked() {
    let mut achievements = Achievements::default();
    achievements.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    achievements.unlock(AchievementId::BossHunterI, Some("Hero".to_string()));

    assert_eq!(achievements.pending_count(), 2);
    assert!(achievements.recently_unlocked.is_empty());

    achievements.clear_pending_notifications();

    assert_eq!(achievements.pending_count(), 0);
    assert_eq!(achievements.recently_unlocked.len(), 2);
    assert!(achievements.is_recently_unlocked(AchievementId::SlayerI));
    assert!(achievements.is_recently_unlocked(AchievementId::BossHunterI));
}

#[test]
fn test_clear_recently_unlocked() {
    let mut achievements = Achievements::default();
    achievements.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    achievements.clear_pending_notifications();

    assert!(!achievements.recently_unlocked.is_empty());

    achievements.clear_recently_unlocked();
    assert!(achievements.recently_unlocked.is_empty());
    assert!(!achievements.is_recently_unlocked(AchievementId::SlayerI));
}

#[test]
fn test_count_recently_unlocked_by_category() {
    let mut achievements = Achievements::default();
    // Unlock 2 combat achievements
    achievements.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    achievements.unlock(AchievementId::BossHunterI, Some("Hero".to_string()));
    // Unlock 1 level achievement
    achievements.unlock(AchievementId::Level10, Some("Hero".to_string()));

    achievements.clear_pending_notifications();

    assert_eq!(
        achievements.count_recently_unlocked_by_category(AchievementCategory::Combat),
        2
    );
    assert_eq!(
        achievements.count_recently_unlocked_by_category(AchievementCategory::Level),
        1
    );
    assert_eq!(
        achievements.count_recently_unlocked_by_category(AchievementCategory::Progression),
        0
    );
}

#[test]
fn test_recently_unlocked_not_serialized() {
    let mut achievements = Achievements::default();
    achievements.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    achievements.clear_pending_notifications();

    let json = serde_json::to_string(&achievements).unwrap();
    let loaded: Achievements = serde_json::from_str(&json).unwrap();

    assert!(loaded.recently_unlocked.is_empty());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib achievements::types::tests -- --nocapture 2>&1 | grep -E "(FAIL|error|cannot find)"`
Expected: Compilation errors — `recently_unlocked`, `is_recently_unlocked`, `clear_recently_unlocked`, `count_recently_unlocked_by_category` don't exist yet.

**Step 3: Add the field to the struct**

In `src/achievements/types.rs`, add after the `modal_queue` field (line ~232):

```rust
/// Achievements recently unlocked — visible as "NEW" badges in browser (not persisted)
#[serde(skip)]
pub recently_unlocked: Vec<AchievementId>,
```

**Step 4: Modify `clear_pending_notifications` to move items**

Change `clear_pending_notifications()` from:

```rust
pub fn clear_pending_notifications(&mut self) {
    self.pending_notifications.clear();
}
```

To:

```rust
pub fn clear_pending_notifications(&mut self) {
    self.recently_unlocked
        .extend(self.pending_notifications.drain(..));
}
```

**Step 5: Add three helper methods**

Add these methods to the `impl Achievements` block:

```rust
/// Clear recently unlocked list (call when achievement browser closes).
pub fn clear_recently_unlocked(&mut self) {
    self.recently_unlocked.clear();
}

/// Check if an achievement was recently unlocked (for NEW badge in browser).
pub fn is_recently_unlocked(&self, id: AchievementId) -> bool {
    self.recently_unlocked.contains(&id)
}

/// Count recently unlocked achievements in a category (for tab badges).
pub fn count_recently_unlocked_by_category(&self, category: AchievementCategory) -> usize {
    use super::data::ALL_ACHIEVEMENTS;
    self.recently_unlocked
        .iter()
        .filter(|id| {
            ALL_ACHIEVEMENTS
                .iter()
                .any(|a| a.id == **id && a.category == category)
        })
        .count()
}
```

**Step 6: Run tests to verify they pass**

Run: `cargo test --lib achievements::types::tests`
Expected: All tests pass, including the 4 new ones.

**Step 7: Run clippy**

Run: `cargo clippy --all-targets -- -D warnings`
Expected: No warnings.

**Step 8: Commit**

```bash
git add src/achievements/types.rs
git commit -m "feat: add recently_unlocked tracking to Achievements"
```

---

## Task 2: Update Browser Rendering — Category Tabs

**Files:**
- Modify: `src/ui/achievement_browser_scene.rs:85-89` (render function signature)
- Modify: `src/ui/achievement_browser_scene.rs:131-158` (render_category_tabs)

**Step 1: Update `render_achievement_browser` to pass `achievements` to sub-functions**

The function already passes `achievements` to all sub-functions. No change needed — `achievements` is already available in `render_category_tabs`.

**Step 2: Update `render_category_tabs` to show NEW badge count**

In `src/ui/achievement_browser_scene.rs`, change the `render_category_tabs` function. Replace the span creation inside the loop (lines ~150-153):

From:
```rust
spans.push(Span::styled(
    format!(" {} ({}/{}) ", cat.name(), unlocked, total),
    style,
));
```

To:
```rust
let new_count = achievements.count_recently_unlocked_by_category(cat);
if new_count > 0 {
    spans.push(Span::styled(
        format!(" {} ({}/{}) ", cat.name(), unlocked, total),
        style,
    ));
    spans.push(Span::styled(
        format!("\u{2022}{} ", new_count),
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ));
} else {
    spans.push(Span::styled(
        format!(" {} ({}/{}) ", cat.name(), unlocked, total),
        style,
    ));
}
```

**Step 3: Run clippy and tests**

Run: `cargo clippy --all-targets -- -D warnings && cargo test --quiet`
Expected: Pass.

**Step 4: Commit**

```bash
git add src/ui/achievement_browser_scene.rs
git commit -m "feat: show NEW count badges on achievement category tabs"
```

---

## Task 3: Update Browser Rendering — Achievement List Stars

**Files:**
- Modify: `src/ui/achievement_browser_scene.rs:160-211` (render_achievement_list)

**Step 1: Update the list item rendering to show star prefix**

In `render_achievement_list`, change the prefix logic (lines ~178-204). Replace the mapping closure:

From:
```rust
.map(|(i, def)| {
    let is_unlocked = achievements.is_unlocked(def.id);
    let is_selected = i == ui_state.selected_index;

    let prefix = if is_selected { "> " } else { "  " };
    let checkmark = if is_unlocked { "[X] " } else { "[ ] " };

    let style = if is_unlocked {
        Style::default().fg(Color::Green)
    } else if is_selected {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    ListItem::new(Line::from(vec![
        Span::styled(prefix, style),
        Span::styled(
            checkmark,
            if is_unlocked {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ),
        Span::raw(format!("{} ", def.icon)),
        Span::styled(def.name, style),
    ]))
})
```

To:
```rust
.map(|(i, def)| {
    let is_unlocked = achievements.is_unlocked(def.id);
    let is_selected = i == ui_state.selected_index;
    let is_new = achievements.is_recently_unlocked(def.id);

    let prefix = if is_selected && is_new {
        ">\u{2605}"
    } else if is_selected {
        "> "
    } else if is_new {
        " \u{2605}"
    } else {
        "  "
    };
    let checkmark = if is_unlocked { "[X] " } else { "[ ] " };

    let style = if is_unlocked {
        Style::default().fg(Color::Green)
    } else if is_selected {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let prefix_style = if is_new {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        style
    };

    ListItem::new(Line::from(vec![
        Span::styled(prefix, prefix_style),
        Span::styled(
            checkmark,
            if is_unlocked {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ),
        Span::raw(format!("{} ", def.icon)),
        Span::styled(def.name, style),
    ]))
})
```

**Step 2: Run clippy and tests**

Run: `cargo clippy --all-targets -- -D warnings && cargo test --quiet`
Expected: Pass.

**Step 3: Commit**

```bash
git add src/ui/achievement_browser_scene.rs
git commit -m "feat: show star prefix on recently unlocked achievements in list"
```

---

## Task 4: Update Browser Rendering — Detail Panel

**Files:**
- Modify: `src/ui/achievement_browser_scene.rs:213-300` (render_achievement_detail)

**Step 1: Add "Recently unlocked!" note in detail panel**

In `render_achievement_detail`, after the unlock timestamp section (after line ~273, after the `if let Some(ref char_name)` block), add:

```rust
if achievements.is_recently_unlocked(def.id) {
    lines.push(Line::from(Span::styled(
        "\u{2605} Recently unlocked!",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )));
}
```

This goes inside the `if is_unlocked` block, after the character name display.

**Step 2: Run clippy and tests**

Run: `cargo clippy --all-targets -- -D warnings && cargo test --quiet`
Expected: Pass.

**Step 3: Commit**

```bash
git add src/ui/achievement_browser_scene.rs
git commit -m "feat: show 'Recently unlocked!' in achievement detail panel"
```

---

## Task 5: Wire Up Clearing on Browser Close

**Files:**
- Modify: `src/input.rs:144-156` (GameOverlay::Achievements close handler)
- Modify: `src/main.rs:528` (achievement_browser.close in char select)

**Step 1: Update input.rs to clear recently_unlocked on close**

The `handle_game_input` function in `src/input.rs` needs `achievements` passed to the overlay close handler. Looking at lines 144-156, when `Esc` or `A` is pressed:

Change the close handler in `input.rs` (lines 144-156):

From:
```rust
if let GameOverlay::Achievements { ref mut browser } = overlay {
    match key.code {
        KeyCode::Esc | KeyCode::Char('a') | KeyCode::Char('A') => {
            *overlay = GameOverlay::None;
        }
```

To:
```rust
if let GameOverlay::Achievements { ref mut browser } = overlay {
    match key.code {
        KeyCode::Esc | KeyCode::Char('a') | KeyCode::Char('A') => {
            achievements.clear_recently_unlocked();
            *overlay = GameOverlay::None;
        }
```

Verify that `achievements` is already available in this function. Check the function signature at line ~120: `achievements: &mut crate::achievements::Achievements` — yes, it's available via `handle_game_input`.

**Step 2: Update main.rs to clear recently_unlocked on close (char select path)**

In `src/main.rs` line 528, change:

From:
```rust
KeyCode::Esc => achievement_browser.close(),
```

To:
```rust
KeyCode::Esc => {
    global_achievements.clear_recently_unlocked();
    achievement_browser.close();
}
```

**Step 3: Update main.rs to populate recently_unlocked on open (char select path)**

In `src/main.rs` line ~571-573, the character select browser open doesn't call `clear_pending_notifications()`. Add it:

From:
```rust
if matches!(key_event.code, KeyCode::Char('a') | KeyCode::Char('A')) {
    achievement_browser.open();
    continue;
}
```

To:
```rust
if matches!(key_event.code, KeyCode::Char('a') | KeyCode::Char('A')) {
    global_achievements.clear_pending_notifications();
    achievement_browser.open();
    continue;
}
```

**Step 4: Run clippy and full tests**

Run: `cargo clippy --all-targets -- -D warnings && cargo test --quiet`
Expected: Pass.

**Step 5: Commit**

```bash
git add src/input.rs src/main.rs
git commit -m "feat: clear recently_unlocked when achievement browser closes"
```

---

## Task 6: Final Validation

**Step 1: Run full CI checks**

Run: `make check`
Expected: All 5 checks pass (fmt, clippy, test, audit, coverage if available).

**Step 2: Verify with the game**

Run: `cargo run -- --debug`
- Use debug menu (backtick) to trigger achievements
- Press 'A' to open browser
- Verify: category tabs show dot-count badges (e.g., `Combat (3/8) \u{2022}2`)
- Verify: recently unlocked achievements have yellow star prefix
- Verify: detail panel shows "Recently unlocked!" for new achievements
- Close browser (Esc)
- Reopen browser — verify badges are gone

**Step 3: Final commit if any formatting changes needed**

Run: `make fmt && make check`
