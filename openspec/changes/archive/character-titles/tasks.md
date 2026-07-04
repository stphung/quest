> Backported implementation plan (completed — this work shipped).

## 2026-02-22-character-titles-plan.md

# Character Titles Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Allow players to earn and display titles as name suffixes (e.g. "Evaa, Eternal") earned through 29 curated achievements, with a dedicated title picker UI accessed from the achievement browser.

**Architecture:** Title definitions are a static mapping from `AchievementId` to display text in a new `src/achievements/titles.rs` module. The selected title is stored in the `Achievements` struct (account-wide, persisted). A `TitleBrowserState` UI state drives a title picker overlay accessible via `[T]` from the achievement browser. Title display is integrated into the stats panel header and character select screen.

**Tech Stack:** Rust, Ratatui, Serde (JSON persistence)

---

### Task 1: Title definitions module

**Files:**
- Create: `src/achievements/titles.rs`
- Modify: `src/achievements/mod.rs:6-19`

**Step 1: Create `src/achievements/titles.rs`**

```rust
//! Title definitions — maps curated achievements to display text.

use super::types::AchievementId;

/// A title that can be earned and displayed after the character name.
pub struct TitleDef {
    pub achievement_id: AchievementId,
    pub title_text: &'static str,
}

/// All available titles, in display order.
pub const ALL_TITLES: &[TitleDef] = &[
    // Level & Prestige
    TitleDef { achievement_id: AchievementId::Level250, title_text: "Legendary" },
    TitleDef { achievement_id: AchievementId::Level500, title_text: "Mythic" },
    TitleDef { achievement_id: AchievementId::Level1000, title_text: "Immortal" },
    TitleDef { achievement_id: AchievementId::Level1500, title_text: "Transcendent" },
    TitleDef { achievement_id: AchievementId::PrestigeXXV, title_text: "Diamond" },
    TitleDef { achievement_id: AchievementId::PrestigeL, title_text: "Emerald" },
    TitleDef { achievement_id: AchievementId::PrestigeLXX, title_text: "Obsidian" },
    TitleDef { achievement_id: AchievementId::Eternal, title_text: "Eternal" },
    // Combat
    TitleDef { achievement_id: AchievementId::SlayerV, title_text: "Slayer" },
    TitleDef { achievement_id: AchievementId::SlayerX, title_text: "Destroyer" },
    TitleDef { achievement_id: AchievementId::SlayerXV, title_text: "Annihilator" },
    TitleDef { achievement_id: AchievementId::BossHunterV, title_text: "Boss Hunter" },
    TitleDef { achievement_id: AchievementId::BossHunterX, title_text: "Bane of Bosses" },
    TitleDef { achievement_id: AchievementId::BossHunterXV, title_text: "Godslayer" },
    // Challenges
    TitleDef { achievement_id: AchievementId::GrandChampion, title_text: "Grand Champion" },
    TitleDef { achievement_id: AchievementId::ChessMaster, title_text: "Grandmaster" },
    TitleDef { achievement_id: AchievementId::GoMaster, title_text: "Sovereign" },
    TitleDef { achievement_id: AchievementId::MorrisMaster, title_text: "Millwright" },
    TitleDef { achievement_id: AchievementId::GomokuMaster, title_text: "Five-Stone Sage" },
    TitleDef { achievement_id: AchievementId::MinesweeperMaster, title_text: "Trapbreaker" },
    TitleDef { achievement_id: AchievementId::RuneMaster, title_text: "Runeweaver" },
    TitleDef { achievement_id: AchievementId::FlappyMaster, title_text: "Skypiercer" },
    TitleDef { achievement_id: AchievementId::SnakeMaster, title_text: "Serpent Lord" },
    TitleDef { achievement_id: AchievementId::ContainmentBreachMaster, title_text: "Warden" },
    TitleDef { achievement_id: AchievementId::SigilSurgeMaster, title_text: "Sigil Savant" },
    // Exploration
    TitleDef { achievement_id: AchievementId::StormLeviathan, title_text: "Leviathan Slayer" },
    TitleDef { achievement_id: AchievementId::FishermanIV, title_text: "Master Angler" },
    TitleDef { achievement_id: AchievementId::HavenArchitect, title_text: "Architect" },
    TitleDef { achievement_id: AchievementId::MasterSmith, title_text: "Soulforged" },
];

/// Get the title text for an achievement, if it grants a title.
pub fn get_title_text(id: AchievementId) -> Option<&'static str> {
    ALL_TITLES
        .iter()
        .find(|t| t.achievement_id == id)
        .map(|t| t.title_text)
}

/// Get all titles the player has unlocked, in display order.
pub fn get_unlocked_titles(achievements: &super::types::Achievements) -> Vec<&'static TitleDef> {
    ALL_TITLES
        .iter()
        .filter(|t| achievements.is_unlocked(t.achievement_id))
        .collect()
}
```

Note: The design doc listed `SoulforgeX` but the actual achievement ID for +10 enhancement is `MasterSmith`. Use `MasterSmith`.

**Step 2: Register the module in `src/achievements/mod.rs`**

Add `pub mod titles;` after line 13 (`pub mod unlock;`), and add a re-export: `pub use titles::{get_title_text, get_unlocked_titles};`

**Step 3: Run `cargo build` to verify it compiles**

Expected: compiles with no errors.

**Step 4: Commit**

```bash
git add src/achievements/titles.rs src/achievements/mod.rs
git commit -m "feat(titles): add title definitions module with 29 curated titles"
```

---

### Task 2: Add `selected_title` to Achievements struct

**Files:**
- Modify: `src/achievements/types.rs:264-281` (Achievements struct)
- Test: `src/achievements/persistence.rs` (existing serialization test)

**Step 1: Write the failing test**

Add to the bottom of the `#[cfg(test)] mod tests` block in `src/achievements/persistence.rs`:

```rust
#[test]
fn test_selected_title_persists() {
    let mut achievements = Achievements::default();
    achievements.unlock(AchievementId::SlayerV, Some("Hero".to_string()));
    achievements.selected_title = Some(AchievementId::SlayerV);

    let json = serde_json::to_string_pretty(&achievements).unwrap();
    let loaded: Achievements = serde_json::from_str(&json).unwrap();

    assert_eq!(loaded.selected_title, Some(AchievementId::SlayerV));
}

#[test]
fn test_selected_title_defaults_to_none() {
    // Simulate loading old save without selected_title field
    let json = r#"{"unlocked":{},"progress":{},"total_kills":0,"total_bosses_defeated":0,"total_fish_caught":0,"total_dungeons_completed":0,"total_minigame_wins":0,"highest_prestige_rank":0,"highest_level":0,"highest_fishing_rank":0,"zones_fully_cleared":0,"expanse_cycles_completed":0}"#;
    let loaded: Achievements = serde_json::from_str(json).unwrap();
    assert_eq!(loaded.selected_title, None);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib test_selected_title -- -v`
Expected: FAIL — `selected_title` field doesn't exist yet.

**Step 3: Add `selected_title` field to `Achievements` struct**

In `src/achievements/types.rs`, add this field after the `expanse_cycles_completed` field (line 281), before the `#[serde(skip)]` fields:

```rust
    /// Currently selected title (account-wide).
    #[serde(default)]
    pub selected_title: Option<AchievementId>,
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --lib test_selected_title -- -v`
Expected: PASS

**Step 5: Commit**

```bash
git add src/achievements/types.rs src/achievements/persistence.rs
git commit -m "feat(titles): add selected_title field to Achievements"
```

---

### Task 3: Title display in stats panel header

**Files:**
- Modify: `src/ui/stats_panel.rs:88-91` (draw_header function)
- Modify: `src/ui/stats_panel.rs:406-412` (draw_compact_stats_bar function)

**Step 1: Update `draw_header()` to show title**

In `src/ui/stats_panel.rs`, the header title is built at lines 88-91. Replace that block with:

```rust
    let title_suffix = achievements
        .selected_title
        .and_then(|id| {
            if achievements.is_unlocked(id) {
                crate::achievements::titles::get_title_text(id)
            } else {
                None
            }
        });
    let name_with_title = match title_suffix {
        Some(title) => format!("{}, {}", game_state.character_name, title),
        None => game_state.character_name.clone(),
    };
    let header_title = match highest_level_badge(achievements) {
        Some(icon) => format!(" {} {} ", name_with_title, icon),
        None => format!(" {} ", name_with_title),
    };
```

**Step 2: Update `draw_compact_stats_bar()` to show title**

In `src/ui/stats_panel.rs` around line 406-412, the compact name span uses `game_state.character_name`. Update it to include the title:

```rust
    let compact_name = match achievements
        .selected_title
        .and_then(|id| {
            if achievements.is_unlocked(id) {
                crate::achievements::titles::get_title_text(id)
            } else {
                None
            }
        }) {
        Some(title) => format!(" {}, {} ", game_state.character_name, title),
        None => format!(" {} ", game_state.character_name),
    };
```

Then use `compact_name` in place of `format!(" {} ", game_state.character_name)` in the `Span::styled(...)` call.

Note: `draw_compact_stats_bar` needs `achievements` passed in. Check the function signature — it already receives `achievements` as a parameter.

**Step 3: Verify it compiles**

Run: `cargo build`
Expected: compiles with no errors.

**Step 4: Commit**

```bash
git add src/ui/stats_panel.rs
git commit -m "feat(titles): display selected title in stats panel header"
```

---

### Task 4: Title display in character select screen

**Files:**
- Modify: `src/ui/character_select.rs:343-346` (character list)
- Modify: `src/ui/character_select.rs:408-413` (character details)

The character select screen needs access to the achievements to look up the selected title. Check whether `render_character_select` already receives achievements as a parameter — if not, it needs to be threaded through.

**Step 1: Check function signatures and add achievements parameter if needed**

Look at `CharacterSelectScreen::draw()` and its callers. The character select screen may not currently have access to achievements. If not, add `achievements: &Achievements` to the relevant draw methods and thread it from `main_helpers/character_screens.rs`.

**Step 2: Update character list display**

In the character list rendering (around line 343-346), update the format string:

```rust
let title_suffix = achievements
    .selected_title
    .and_then(|id| {
        if achievements.is_unlocked(id) {
            crate::achievements::titles::get_title_text(id)
        } else {
            None
        }
    });
let name_display = match title_suffix {
    Some(title) => format!("{}, {}", character.character_name, title),
    None => character.character_name.clone(),
};
// Then use name_display in the format string
format!("{} (Lv {} {})", name_display, character.character_level, prestige_name)
```

**Step 3: Update character details display**

In the details panel (around line 408-413), update the name span similarly.

**Step 4: Verify it compiles**

Run: `cargo build`
Expected: compiles with no errors.

**Step 5: Commit**

```bash
git add src/ui/character_select.rs src/main_helpers/character_screens.rs
git commit -m "feat(titles): display selected title in character select screen"
```

---

### Task 5: Title browser UI state and scene

**Files:**
- Create: `src/ui/title_browser_scene.rs`
- Modify: `src/ui/mod.rs` (register new module)

**Step 1: Create `src/ui/title_browser_scene.rs`**

```rust
//! Title browser overlay UI — lets the player select a displayed title.

use crate::achievements::{titles, Achievements};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// UI state for the title browser overlay.
pub struct TitleBrowserState {
    pub showing: bool,
    pub selected_index: usize,
}

impl TitleBrowserState {
    pub fn new() -> Self {
        Self {
            showing: false,
            selected_index: 0,
        }
    }

    pub fn open(&mut self) {
        self.showing = true;
        self.selected_index = 0;
    }

    pub fn close(&mut self) {
        self.showing = false;
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self, max_items: usize) {
        if self.selected_index + 1 < max_items {
            self.selected_index += 1;
        }
    }
}

impl Default for TitleBrowserState {
    fn default() -> Self {
        Self::new()
    }
}

/// Render the title browser overlay.
pub fn render_title_browser(
    frame: &mut Frame,
    area: Rect,
    achievements: &Achievements,
    ui_state: &TitleBrowserState,
    character_name: &str,
) {
    frame.render_widget(Clear, area);

    let unlocked = titles::get_unlocked_titles(achievements);

    let block = Block::default()
        .title(format!(" Titles ({} unlocked) ", unlocked.len()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if unlocked.is_empty() {
        let msg = Paragraph::new("No titles unlocked yet. Keep adventuring!")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(msg, inner);
        return;
    }

    // Layout: title list, preview, help
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Title list
            Constraint::Length(3), // Preview
            Constraint::Length(1), // Help
        ])
        .split(inner);

    // Title list
    let mut lines = Vec::new();
    for (i, title_def) in unlocked.iter().enumerate() {
        let is_selected = i == ui_state.selected_index;
        let is_active = achievements.selected_title == Some(title_def.achievement_id);

        let marker = if is_selected { "> " } else { "  " };
        let active_suffix = if is_active { "  ✦ active" } else { "" };

        let style = if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else if is_active {
            Style::default().fg(Color::Magenta)
        } else {
            Style::default().fg(Color::White)
        };

        lines.push(Line::from(Span::styled(
            format!("{}{}{}", marker, title_def.title_text, active_suffix),
            style,
        )));
    }
    let list = Paragraph::new(lines);
    frame.render_widget(list, chunks[0]);

    // Preview
    let preview_title = if ui_state.selected_index < unlocked.len() {
        format!("{}, {}", character_name, unlocked[ui_state.selected_index].title_text)
    } else {
        character_name.to_string()
    };
    let preview_block = Block::default()
        .title(" Preview ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let preview = Paragraph::new(Line::from(Span::styled(
        &preview_title,
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )))
    .block(preview_block);
    frame.render_widget(preview, chunks[1]);

    // Help
    let help = Paragraph::new("[Enter] Select  [Backspace] Clear  [Esc] Back")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(help, chunks[2]);
}
```

**Step 2: Register in `src/ui/mod.rs`**

Add `pub mod title_browser_scene;` alongside the other scene modules. Add the re-export if needed for the `TitleBrowserState` type.

**Step 3: Verify it compiles**

Run: `cargo build`

**Step 4: Commit**

```bash
git add src/ui/title_browser_scene.rs src/ui/mod.rs
git commit -m "feat(titles): add title browser scene UI"
```

---

### Task 6: Wire up title browser input handling (game overlay)

**Files:**
- Modify: `src/input/types.rs:74-75` (GameOverlay::Achievements variant)
- Modify: `src/input/mod.rs:66-81` (achievement browser input handling)
- Modify: `src/main_helpers/overlay.rs:141-149` (achievement browser rendering)

**Step 1: Add `title_browser` state to `GameOverlay::Achievements`**

In `src/input/types.rs`, update the `Achievements` variant to include the title browser state:

```rust
    Achievements {
        browser: crate::ui::achievement_browser_scene::AchievementBrowserState,
        title_browser: crate::ui::title_browser_scene::TitleBrowserState,
    },
```

**Step 2: Update all `GameOverlay::Achievements` construction sites**

There are two places that construct this variant:
1. `src/input/mod.rs:354-356` — add `title_browser: crate::ui::title_browser_scene::TitleBrowserState::new()`
2. `src/main_helpers/character_screens.rs:220` (if it constructs this variant) — same addition

**Step 3: Update achievement browser input handling in `src/input/mod.rs:66-81`**

Add title browser input routing. When the title browser is showing, it captures input. When `[T]` is pressed in the achievement browser, it opens the title browser:

```rust
    // 0.5. Achievement browser overlay
    if let GameOverlay::Achievements { ref mut browser, ref mut title_browser } = overlay {
        // Title browser takes priority when open
        if title_browser.showing {
            let unlocked = crate::achievements::titles::get_unlocked_titles(achievements);
            match key.code {
                KeyCode::Esc => title_browser.close(),
                KeyCode::Up => title_browser.move_up(),
                KeyCode::Down => title_browser.move_down(unlocked.len()),
                KeyCode::Enter => {
                    if let Some(title_def) = unlocked.get(title_browser.selected_index) {
                        achievements.selected_title = Some(title_def.achievement_id);
                        title_browser.close();
                    }
                }
                KeyCode::Backspace => {
                    achievements.selected_title = None;
                    title_browser.close();
                }
                _ => {}
            }
            return InputResult::Continue;
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('a') | KeyCode::Char('A') => {
                achievements.clear_recently_unlocked();
                *overlay = GameOverlay::None;
            }
            KeyCode::Left => browser.prev_category(),
            KeyCode::Right => browser.next_category(),
            KeyCode::Up => browser.move_up(),
            KeyCode::Down => {
                let count = get_achievements_by_category(browser.selected_category).len();
                browser.move_down(count);
            }
            KeyCode::Char('t') | KeyCode::Char('T') => {
                title_browser.open();
            }
            _ => {}
        }
        return InputResult::Continue;
    }
```

**Step 4: Update rendering in `src/main_helpers/overlay.rs:141-149`**

Update the `GameOverlay::Achievements` match arm to render the title browser when showing:

```rust
        GameOverlay::Achievements { browser, title_browser } => {
            if title_browser.showing {
                ui::title_browser_scene::render_title_browser(
                    frame,
                    area,
                    global_achievements,
                    title_browser,
                    &state.character_name,
                );
            } else {
                ui::achievement_browser_scene::render_achievement_browser(
                    frame,
                    area,
                    global_achievements,
                    browser,
                    enhancement,
                    ctx,
                );
            }
        }
```

**Step 5: Update all other pattern matches on `GameOverlay::Achievements`**

Search for `GameOverlay::Achievements` in `src/main.rs` (lines 345 and 389) and update the destructuring pattern to include `title_browser: _` or just `..`.

**Step 6: Verify it compiles**

Run: `cargo build`

**Step 7: Commit**

```bash
git add src/input/types.rs src/input/mod.rs src/main_helpers/overlay.rs src/main.rs
git commit -m "feat(titles): wire up title browser input and rendering"
```

---

### Task 7: Wire up title browser in character select screen

**Files:**
- Modify: `src/main_helpers/character_screens.rs:194-214` (achievement browser input in character select)

**Step 1: Add title browser state to character select**

The character select screen stores its own `AchievementBrowserState`. It also needs a `TitleBrowserState`. Add it alongside the browser state in the character select handler, and wire up `[T]` input and title browser input handling, following the same pattern as Task 6.

**Step 2: Update rendering to show title browser when active**

When `title_browser.showing` is true in the character select's achievement browser, render the title browser instead of the achievement browser.

**Step 3: Verify it compiles**

Run: `cargo build`

**Step 4: Commit**

```bash
git add src/main_helpers/character_screens.rs
git commit -m "feat(titles): wire up title browser in character select screen"
```

---

### Task 8: Update help text

**Files:**
- Modify: `src/ui/achievement_browser_scene.rs:147` (help bar)

**Step 1: Update the achievement browser help bar**

Change line 147 from:
```rust
    let help = Paragraph::new("[</>] Category  [Up/Down] Select  [Esc] Close")
```
to:
```rust
    let help = Paragraph::new("[</>] Category  [Up/Down] Select  [T] Titles  [Esc] Close")
```

**Step 2: Verify it compiles**

Run: `cargo build`

**Step 3: Commit**

```bash
git add src/ui/achievement_browser_scene.rs
git commit -m "feat(titles): add [T] key hint to achievement browser help bar"
```

---

### Task 9: Title validation on load

**Files:**
- Modify: `src/achievements/titles.rs` (add validation function)

**Step 1: Add validation logic**

Add to `src/achievements/titles.rs`:

```rust
/// Validate the selected title — clear it if the achievement isn't unlocked.
pub fn validate_selected_title(achievements: &mut super::types::Achievements) {
    if let Some(id) = achievements.selected_title {
        if !achievements.is_unlocked(id) || get_title_text(id).is_none() {
            achievements.selected_title = None;
        }
    }
}
```

**Step 2: Call validation after loading achievements**

In the places where achievements are loaded (search for `load_achievements()` calls), call `titles::validate_selected_title(&mut achievements)` immediately after loading.

**Step 3: Write a test**

Add to `src/achievements/titles.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::achievements::types::Achievements;

    #[test]
    fn test_get_title_text_exists() {
        assert_eq!(get_title_text(AchievementId::Eternal), Some("Eternal"));
        assert_eq!(get_title_text(AchievementId::SlayerV), Some("Slayer"));
    }

    #[test]
    fn test_get_title_text_none_for_non_title() {
        assert_eq!(get_title_text(AchievementId::SlayerI), None);
        assert_eq!(get_title_text(AchievementId::Level10), None);
    }

    #[test]
    fn test_get_unlocked_titles_empty() {
        let achievements = Achievements::default();
        assert!(get_unlocked_titles(&achievements).is_empty());
    }

    #[test]
    fn test_get_unlocked_titles_filters() {
        let mut achievements = Achievements::default();
        achievements.unlock(AchievementId::SlayerV, Some("Hero".to_string()));
        achievements.unlock(AchievementId::SlayerI, Some("Hero".to_string())); // no title
        let titles = get_unlocked_titles(&achievements);
        assert_eq!(titles.len(), 1);
        assert_eq!(titles[0].title_text, "Slayer");
    }

    #[test]
    fn test_validate_clears_invalid() {
        let mut achievements = Achievements::default();
        // Set title without unlocking — should be cleared
        achievements.selected_title = Some(AchievementId::Eternal);
        validate_selected_title(&mut achievements);
        assert_eq!(achievements.selected_title, None);
    }

    #[test]
    fn test_validate_keeps_valid() {
        let mut achievements = Achievements::default();
        achievements.unlock(AchievementId::SlayerV, Some("Hero".to_string()));
        achievements.selected_title = Some(AchievementId::SlayerV);
        validate_selected_title(&mut achievements);
        assert_eq!(achievements.selected_title, Some(AchievementId::SlayerV));
    }

    #[test]
    fn test_validate_clears_non_title_achievement() {
        let mut achievements = Achievements::default();
        achievements.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
        achievements.selected_title = Some(AchievementId::SlayerI); // unlocked but not a title
        validate_selected_title(&mut achievements);
        assert_eq!(achievements.selected_title, None);
    }
}
```

**Step 4: Run tests**

Run: `cargo test --lib achievements::titles -- -v`
Expected: all pass.

**Step 5: Commit**

```bash
git add src/achievements/titles.rs src/main.rs
git commit -m "feat(titles): add title validation on load with tests"
```

---

### Task 10: Save achievements when title changes

**Files:**
- Modify: `src/input/mod.rs` (title selection triggers save)

**Step 1: Return `InputResult::NeedsSave` when title changes**

In Task 6's input handling, when `[Enter]` sets a title or `[Backspace]` clears it, return `InputResult::NeedsSave` instead of `InputResult::Continue` so the achievements file is saved.

Check what `NeedsSave` triggers in the game loop — it should call `save_achievements()`. If `NeedsSave` only saves character state and not achievements, use the appropriate result or add a save call. The `save_all` function in `main_helpers/persistence.rs` saves both.

**Step 2: Verify it compiles**

Run: `cargo build`

**Step 3: Commit**

```bash
git add src/input/mod.rs
git commit -m "feat(titles): save achievements when title is changed"
```

---

### Task 11: Final integration test and cleanup

**Step 1: Run full test suite**

Run: `cargo test`
Expected: all tests pass, no regressions.

**Step 2: Run full CI checks**

Run: `make check`
Expected: format, clippy, tests, build, audit all pass.

**Step 3: Fix any clippy warnings or formatting issues**

Run: `make fmt` if needed.

**Step 4: Final commit if any fixes needed**

```bash
git add -A
git commit -m "chore: fix formatting and clippy for titles feature"
```
