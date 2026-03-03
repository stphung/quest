# Unified Status Strip Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the disconnected floating combat text and inconsistent status strip renderers with a single reusable `draw_status_row()` component used by both combat and dungeon modes.

**Architecture:** A new `draw_status_row()` function renders one HP row as text spans: `Label:cur/max [████░░░░] | segment | segment ... flash`. Both `draw_status_strip_combat()` and `draw_status_strip_dungeon()` call it twice (player row + enemy row). Floating damage text is removed from the XL/L sprite area. Smart number abbreviation uses the existing `format_number_short()` from `game_common.rs`.

**Tech Stack:** Rust, Ratatui (Span/Line/Paragraph widgets)

---

### Task 1: Add `format_hp_label()` helper

**Files:**
- Modify: `src/ui/mod.rs` (add function near line 1028, next to `text_hp_bar()`)

**Step 1: Write the function**

Add `format_hp_label()` that formats `"Label:cur/max"` using `format_number_short` when values are large. Place it right after the existing `text_hp_bar()` function (line 1040).

```rust
/// Formats an HP label like "HP:340/500" or "Goblin:12.4K/25K" using short
/// number formatting for values >= 10,000.
fn format_hp_label(name: &str, current: u32, max: u32) -> String {
    let cur_s = game_common::format_number_short(current as u64);
    let max_s = game_common::format_number_short(max as u64);
    format!("{}:{}/{}", name, cur_s, max_s)
}
```

**Step 2: Verify it compiles**

Run: `cargo build 2>&1 | tail -3`
Expected: `Finished`

**Step 3: Commit**

```
feat(ui): add format_hp_label helper for status strip
```

---

### Task 2: Write `draw_status_row()` reusable component

**Files:**
- Modify: `src/ui/mod.rs` (add function near line 1042, replacing `render_hp_bar_with_flash`)

**Step 1: Write the function**

Replace `render_hp_bar_with_flash()` (lines 710-750) with `draw_status_row()`. The new function renders a full status row as a single `Line` of spans:

```rust
/// Renders one status strip row: `Label:cur/max [████░░░░] | seg | seg ... flash`
///
/// - `hp_label`: pre-formatted string like "HP:340/500"
/// - `hp_ratio`: 0.0..=1.0 fill ratio for the text bar
/// - `bar_color`: color for the filled portion of the text bar
/// - `segments`: additional info spans rendered after the bar (timers, DPS, room info)
/// - `flash`: optional damage flash rendered right-aligned at row end
fn draw_status_row(
    frame: &mut Frame,
    area: Rect,
    hp_label: &str,
    hp_ratio: f64,
    bar_color: Color,
    segments: &[Span],
    flash: Option<&crate::combat::types::DamageFlash>,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let bar_width = (area.width as usize / 6).clamp(4, 10);
    let hp_bar = text_hp_bar(hp_ratio, bar_width);

    let mut spans: Vec<Span> = Vec::with_capacity(8);
    spans.push(Span::styled(
        format!("{} ", hp_label),
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::styled(hp_bar, Style::default().fg(bar_color)));

    for seg in segments {
        spans.push(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
        spans.push(seg.clone());
    }

    // Flash: render right-aligned in remaining space
    if let Some(flash) = flash {
        let used: usize = spans.iter().map(|s| s.content.chars().count()).sum();
        let flash_len = flash.text.chars().count();
        let available = (area.width as usize).saturating_sub(used + 1);
        if flash_len <= available {
            // Pad to push flash to the right
            let pad = available.saturating_sub(flash_len);
            if pad > 0 {
                spans.push(Span::raw(" ".repeat(pad)));
            }
            let progress = 1.0 - (flash.remaining / crate::combat::types::DAMAGE_FLASH_DURATION);
            let style = if progress > 0.8 {
                Style::default().fg(Color::DarkGray)
            } else if progress > 0.6 {
                Style::default().fg(flash.color)
            } else {
                let mut s = Style::default().fg(flash.color);
                if flash.bold {
                    s = s.add_modifier(Modifier::BOLD);
                }
                s
            };
            spans.push(Span::styled(&flash.text, style));
        }
    }

    let line = Paragraph::new(Line::from(spans)).alignment(Alignment::Left);
    frame.render_widget(line, area);
}
```

**Step 2: Verify it compiles**

Run: `cargo build 2>&1 | tail -3`
Expected: `Finished` (function exists but is unused — clippy will warn, that's fine for now)

**Step 3: Commit**

```
feat(ui): add draw_status_row reusable component
```

---

### Task 3: Rewrite `draw_status_strip_combat()` to use `draw_status_row()`

**Files:**
- Modify: `src/ui/mod.rs:1044-1177` — rewrite `draw_status_strip_combat()`

**Step 1: Rewrite the function**

Replace the entire body of `draw_status_strip_combat()` (lines 1044-1177) with calls to `draw_status_row()`:

```rust
/// Combat/idle status strip: player HP + attack timer + DPS on row 1,
/// enemy HP + enemy timer (or enrage for bosses) on row 2.
fn draw_status_strip_combat(frame: &mut Frame, row0: Rect, row1: Rect, game_state: &GameState) {
    let hp = &game_state.combat_state;
    let derived = game_state.cached_derived_stats;

    // DPS calculation
    let base_dps = derived.total_damage() as f64 / ATTACK_INTERVAL_SECONDS;
    let effective_dps = base_dps
        * (1.0 + (derived.crit_chance_percent as f64 / 100.0) * (derived.crit_multiplier - 1.0));

    let hp_ratio = if hp.player_max_hp > 0 {
        (hp.player_current_hp as f64 / hp.player_max_hp as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let player_label = format_hp_label("HP", hp.player_current_hp, hp.player_max_hp);

    // Row 1: Player HP + segments
    let mut player_segs: Vec<Span> = Vec::new();
    if hp.current_enemy.is_some() {
        let player_interval = ATTACK_INTERVAL_SECONDS / derived.attack_speed_multiplier;
        let player_next = (player_interval - hp.player_attack_timer).max(0.0);
        let player_style = if player_next < 0.3 {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Green)
        };
        player_segs.push(Span::styled(format!("You:{:.1}s", player_next), player_style));
    }
    player_segs.push(Span::styled(
        format!("DPS:{:.0}", effective_dps),
        Style::default().fg(Color::DarkGray),
    ));

    draw_status_row(
        frame,
        row0,
        &player_label,
        hp_ratio,
        Color::Green,
        &player_segs,
        hp.player_damage_floats.last(),
    );

    // Row 2: Enemy HP + segments, or idle message
    if let Some(enemy) = &hp.current_enemy {
        let enemy_ratio = if enemy.max_hp > 0 {
            (enemy.current_hp as f64 / enemy.max_hp as f64).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let is_boss = game_state.zone_progression.fighting_boss || enemy.name.starts_with("Boss ");
        let hp_color = if is_boss {
            Color::LightRed
        } else {
            let zone_id = game_state
                .active_dungeon
                .as_ref()
                .map(|d| d.zone_id)
                .unwrap_or(game_state.zone_progression.current_zone_id);
            enemy_sprites::zone_palette(zone_id).primary
        };

        // Truncate enemy name
        let max_name_len = 12usize;
        let name: String = enemy.name.chars().take(max_name_len).collect();
        let enemy_label = format_hp_label(&name, enemy.current_hp, enemy.max_hp);

        let mut enemy_segs: Vec<Span> = Vec::new();
        if game_state.zone_progression.fighting_boss {
            let remaining = (BOSS_ENRAGE_SECONDS - hp.boss_fight_timer).max(0.0);
            let enrage_style = if remaining < 5.0 {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else if remaining < 10.0 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Cyan)
            };
            enemy_segs.push(Span::styled(format!("\u{26a1}Enrage:{:.0}s", remaining), enrage_style));
        } else {
            let enemy_interval = effective_enemy_attack_interval(game_state);
            let enemy_next = (enemy_interval - hp.enemy_attack_timer).max(0.0);
            let enemy_style = if enemy_next < 0.3 {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Red)
            };
            enemy_segs.push(Span::styled(format!("Foe:{:.1}s", enemy_next), enemy_style));
        }

        draw_status_row(
            frame,
            row1,
            &enemy_label,
            enemy_ratio,
            hp_color,
            &enemy_segs,
            hp.enemy_damage_floats.last(),
        );
    } else {
        let spinner = throbber::spinner_char();
        let msg = throbber::waiting_message(game_state.character_xp);
        let text = Paragraph::new(Line::from(vec![
            Span::styled(
                format!("{} {}", spinner, msg),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                format!(" | DPS:{:.0}", effective_dps),
                Style::default().fg(Color::DarkGray),
            ),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(text, row1);
    }
}
```

**Step 2: Verify it compiles and tests pass**

Run: `cargo build 2>&1 | tail -3` then `cargo test 2>&1 | tail -5`

**Step 3: Commit**

```
refactor(ui): rewrite combat status strip to use draw_status_row
```

---

### Task 4: Rewrite `draw_status_strip_dungeon()` to use `draw_status_row()`

**Files:**
- Modify: `src/ui/mod.rs:960-1026` — rewrite `draw_status_strip_dungeon()`

**Step 1: Rewrite the function**

Replace the entire body of `draw_status_strip_dungeon()` with calls to `draw_status_row()`:

```rust
/// Dungeon status strip: player HP with room/key info on row 1, enemy HP or status on row 2.
fn draw_status_strip_dungeon(frame: &mut Frame, row0: Rect, row1: Rect, game_state: &GameState) {
    let hp = &game_state.combat_state;
    let derived = game_state.cached_derived_stats;

    let hp_ratio = if hp.player_max_hp > 0 {
        (hp.player_current_hp as f64 / hp.player_max_hp as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let player_label = format_hp_label("HP", hp.player_current_hp, hp.player_max_hp);

    // Row 1: Player HP + attack timer + room/key info
    let mut player_segs: Vec<Span> = Vec::new();
    if hp.current_enemy.is_some() {
        let player_interval = ATTACK_INTERVAL_SECONDS / derived.attack_speed_multiplier;
        let player_next = (player_interval - hp.player_attack_timer).max(0.0);
        let player_style = if player_next < 0.3 {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Green)
        };
        player_segs.push(Span::styled(format!("You:{:.1}s", player_next), player_style));
    }
    if let Some(ref dungeon) = game_state.active_dungeon {
        let cleared = dungeon.rooms_cleared;
        let total = dungeon.room_count();
        let key_str = if dungeon.has_key { " \u{1f511}" } else { "" };
        player_segs.push(Span::styled(
            format!("Rm {}/{}{}", cleared, total, key_str),
            Style::default().fg(Color::Magenta),
        ));
    }

    draw_status_row(
        frame,
        row0,
        &player_label,
        hp_ratio,
        Color::Green,
        &player_segs,
        hp.player_damage_floats.last(),
    );

    // Row 2: Enemy HP + attack timer, or exploring message
    if let Some(enemy) = &hp.current_enemy {
        let enemy_ratio = if enemy.max_hp > 0 {
            (enemy.current_hp as f64 / enemy.max_hp as f64).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let is_boss = enemy.name.starts_with("Boss ");
        let hp_color = if is_boss {
            Color::LightRed
        } else {
            let zone_id = game_state
                .active_dungeon
                .as_ref()
                .map(|d| d.zone_id)
                .unwrap_or(game_state.zone_progression.current_zone_id);
            enemy_sprites::zone_palette(zone_id).primary
        };

        let max_name_len = 12usize;
        let name: String = enemy.name.chars().take(max_name_len).collect();
        let enemy_label = format_hp_label(&name, enemy.current_hp, enemy.max_hp);

        let mut enemy_segs: Vec<Span> = Vec::new();
        let enemy_interval = effective_enemy_attack_interval(game_state);
        let enemy_next = (enemy_interval - hp.enemy_attack_timer).max(0.0);
        let enemy_style = if enemy_next < 0.3 {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Red)
        };
        enemy_segs.push(Span::styled(format!("Foe:{:.1}s", enemy_next), enemy_style));

        draw_status_row(
            frame,
            row1,
            &enemy_label,
            enemy_ratio,
            hp_color,
            &enemy_segs,
            hp.enemy_damage_floats.last(),
        );
    } else {
        let spinner = throbber::spinner_char();
        let text = Paragraph::new(Line::from(Span::styled(
            format!("{} Exploring the dungeon...", spinner),
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::ITALIC),
        )))
        .alignment(Alignment::Center);
        frame.render_widget(text, row1);
    }
}
```

**Step 2: Verify it compiles and tests pass**

Run: `cargo build 2>&1 | tail -3` then `cargo test 2>&1 | tail -5`

**Step 3: Commit**

```
refactor(ui): rewrite dungeon status strip to use draw_status_row
```

---

### Task 5: Update S-tier HP functions and remove old code

**Files:**
- Modify: `src/ui/mod.rs` — update `draw_s_player_hp()` (line 753) and `draw_s_enemy_hp()` (line 771) to use `draw_status_row()`, then delete `render_hp_bar_with_flash()` (lines 710-750)

**Step 1: Rewrite S-tier functions**

Replace `draw_s_player_hp()`:

```rust
fn draw_s_player_hp(frame: &mut Frame, area: Rect, game_state: &GameState) {
    let ratio = game_state.combat_state.player_current_hp as f64
        / game_state.combat_state.player_max_hp as f64;
    let label = format_hp_label("HP", game_state.combat_state.player_current_hp, game_state.combat_state.player_max_hp);
    draw_status_row(
        frame,
        area,
        &label,
        ratio,
        Color::Green,
        &[],
        game_state.combat_state.player_damage_floats.last(),
    );
}
```

Replace `draw_s_enemy_hp()`:

```rust
fn draw_s_enemy_hp(frame: &mut Frame, area: Rect, game_state: &GameState) {
    if let Some(enemy) = &game_state.combat_state.current_enemy {
        let ratio = enemy.current_hp as f64 / enemy.max_hp as f64;
        let label = format_hp_label(&enemy.name, enemy.current_hp, enemy.max_hp);
        draw_status_row(
            frame,
            area,
            &label,
            ratio,
            Color::Red,
            &[],
            game_state.combat_state.enemy_damage_floats.last(),
        );
    }
}
```

**Step 2: Delete `render_hp_bar_with_flash()`**

Remove the entire function at lines 710-750 — it has no remaining callers.

**Step 3: Verify it compiles and tests pass**

Run: `cargo clippy --all-targets -- -D warnings 2>&1 | tail -5` then `cargo test 2>&1 | tail -5`

**Step 4: Commit**

```
refactor(ui): migrate S-tier HP to draw_status_row, remove render_hp_bar_with_flash
```

---

### Task 6: Remove floating damage text from XL/L combat scene

**Files:**
- Modify: `src/ui/combat_scene.rs:113-133` — remove `draw_floating_damage()` calls from `draw_combat_full()`

**Step 1: Remove the calls**

In `draw_combat_full()`, remove both `draw_floating_damage()` calls (lines 128 and 131):

```rust
fn draw_combat_full(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    _achievements: &crate::achievements::Achievements,
) {
    let is_regen = game_state.combat_state.is_regenerating;

    if is_regen {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(5)])
            .split(area);
        draw_regen_throbber(frame, chunks[0], game_state);
        render_combat_3d(frame, chunks[1], game_state);
    } else {
        render_combat_3d(frame, area, game_state);
    }
}
```

Note: Keep `draw_floating_damage()` call in `draw_combat_compact()` (M tier, line 180) — M tier still has HP bars adjacent to the sprite.

**Step 2: Run clippy to check for dead code**

Run: `cargo clippy --all-targets -- -D warnings 2>&1 | tail -10`

If `draw_floating_damage` and `render_float_text` show dead code warnings (they're still used by M tier's `draw_combat_compact`), they should NOT be removed. If they become fully unused, remove them.

**Step 3: Verify tests pass**

Run: `cargo test 2>&1 | tail -5`

**Step 4: Commit**

```
refactor(ui): remove floating damage text from XL/L combat scene
```

---

### Task 7: Final verification

**Step 1: Run full CI check**

Run: `make check`

All checks must pass: fmt, clippy, test, build.

**Step 2: Manual visual test**

Run: `cargo run`

Verify:
- Combat mode: player and enemy rows show HP bar + timers + DPS + damage flash numbers
- Dungeon mode: same layout with room/key info instead of DPS
- Boss mode: enrage timer appears in enemy row
- Regen: player row shows heal flash, enemy row shows idle message
- S tier (shrink terminal < 60 wide): HP rows still render correctly
- M tier: compact combat scene still has floating damage over sprite (unchanged)
- XL/L tier: no floating text over sprite — damage only in status strip

**Step 3: Commit if any formatting fixups needed**

Run: `make fmt` then commit if changes.
