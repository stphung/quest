# Unified Deep Panel Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the Power Cores panel with a unified "The Deep" panel that shows guild rank, missions, crew status, and compact core readiness badges in 8 rows.

**Architecture:** Rename/rewrite `draw_power_cores_panel()` in `src/ui/stats_prestige.rs` to `draw_deep_panel()`. The function signature gains access to `DeepState` (already passed). The layout in `stats_panel.rs` changes the visibility condition from "any unlocked core" to "Deep discovered". Height stays at 8 rows.

**Tech Stack:** Ratatui (Span, Line, Paragraph, Block), chrono (Utc::now for mission ETA), existing Deep types.

---

### Task 1: Add helper function for next mission ETA

**Files:**
- Modify: `src/ui/stats_prestige.rs`

**Step 1: Write the failing test**

Add at the bottom of the existing `#[cfg(test)] mod tests` block in `src/ui/stats_prestige.rs`:

```rust
#[test]
fn test_next_mission_eta_no_missions() {
    let prestige = crate::deep::DeepPrestige::default();
    assert_eq!(next_mission_eta_secs(&prestige), None);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib ui::stats_prestige::tests::test_next_mission_eta_no_missions`
Expected: FAIL — `next_mission_eta_secs` not found

**Step 3: Write minimal implementation**

Add above the `#[cfg(test)]` block in `src/ui/stats_prestige.rs`:

```rust
/// Returns seconds until the next active mission completes, or None if no active missions.
fn next_mission_eta_secs(prestige: &crate::deep::DeepPrestige) -> Option<i64> {
    let now = chrono::Utc::now();
    prestige
        .active_missions
        .iter()
        .filter(|m| matches!(m.status, crate::deep::MissionStatus::Active | crate::deep::MissionStatus::EventPending))
        .map(|m| (m.ends_at - now).num_seconds().max(0))
        .min()
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib ui::stats_prestige::tests::test_next_mission_eta_no_missions`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ui/stats_prestige.rs
git commit -m "feat: add next_mission_eta_secs helper for Deep panel"
```

---

### Task 2: Add helper function for pending event count

**Files:**
- Modify: `src/ui/stats_prestige.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_pending_event_count_no_events() {
    let prestige = crate::deep::DeepPrestige::default();
    assert_eq!(pending_event_count(&prestige), 0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib ui::stats_prestige::tests::test_pending_event_count_no_events`
Expected: FAIL

**Step 3: Write minimal implementation**

```rust
/// Count of active missions with pending events needing player response.
fn pending_event_count(prestige: &crate::deep::DeepPrestige) -> usize {
    prestige
        .active_missions
        .iter()
        .filter(|m| m.has_pending_event())
        .count()
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib ui::stats_prestige::tests::test_pending_event_count_no_events`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ui/stats_prestige.rs
git commit -m "feat: add pending_event_count helper for Deep panel"
```

---

### Task 3: Add helper for core summary data

**Files:**
- Modify: `src/ui/stats_prestige.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_core_summary_no_cores() {
    let achievements = Achievements::default();
    let deep = crate::deep::DeepState::default();
    let summary = core_summary(&achievements, &deep);
    assert_eq!(summary.ready_count, 0);
    assert_eq!(summary.ready_pr, 0);
    assert_eq!(summary.unlocked_count, 0);
    assert_eq!(summary.total_pr_per_day, 0);
    assert!(summary.next_ready_secs.is_none());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib ui::stats_prestige::tests::test_core_summary_no_cores`
Expected: FAIL

**Step 3: Write minimal implementation**

```rust
struct CoreSummary {
    ready_count: usize,
    ready_pr: u32,
    unlocked_count: usize,
    total_pr_per_day: u32,
    next_ready_secs: Option<i64>,
    /// Per-core status: (is_unlocked, is_ready, time_remaining_secs, required_layer, pr_per_day)
    cores: Vec<CoreBadge>,
}

struct CoreBadge {
    unlocked: bool,
    ready: bool,
    remaining_secs: i64,
    required_layer: u32,
}

fn core_summary(
    achievements: &crate::achievements::Achievements,
    deep: &crate::deep::DeepState,
) -> CoreSummary {
    use crate::power_cores::types::{fill_duration_secs, fill_ratio, ALL_POWER_CORES};
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let mut summary = CoreSummary {
        ready_count: 0,
        ready_pr: 0,
        unlocked_count: 0,
        total_pr_per_day: 0,
        next_ready_secs: None,
        cores: Vec::new(),
    };

    for core in ALL_POWER_CORES {
        let is_unlocked = achievements.is_unlocked(core.achievement_id);

        if is_unlocked {
            summary.unlocked_count += 1;
            summary.total_pr_per_day += core.pr_per_day;

            let fill_secs = fill_duration_secs(core.pr_per_day);
            let last_granted = deep
                .persistent
                .power_core_last_granted
                .get(&core.achievement_id)
                .copied()
                .unwrap_or(0);
            let elapsed = (now - last_granted).max(0);
            let ratio = fill_ratio(elapsed, fill_secs);
            let remaining = (fill_secs - elapsed).max(0);

            if ratio >= 1.0 {
                summary.ready_count += 1;
                summary.ready_pr += core.pr_per_day;
                summary.cores.push(CoreBadge {
                    unlocked: true,
                    ready: true,
                    remaining_secs: 0,
                    required_layer: core.required_layer,
                });
            } else {
                if let Some(current_next) = summary.next_ready_secs {
                    if remaining < current_next {
                        summary.next_ready_secs = Some(remaining);
                    }
                } else {
                    summary.next_ready_secs = Some(remaining);
                }
                summary.cores.push(CoreBadge {
                    unlocked: true,
                    ready: false,
                    remaining_secs: remaining,
                    required_layer: core.required_layer,
                });
            }
        } else {
            summary.cores.push(CoreBadge {
                unlocked: false,
                ready: false,
                remaining_secs: 0,
                required_layer: core.required_layer,
            });
        }
    }

    summary
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib ui::stats_prestige::tests::test_core_summary_no_cores`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ui/stats_prestige.rs
git commit -m "feat: add core_summary helper for Deep panel"
```

---

### Task 4: Rewrite draw_power_cores_panel to draw_deep_panel

**Files:**
- Modify: `src/ui/stats_prestige.rs` — replace `draw_power_cores_panel()` with `draw_deep_panel()`

**Step 1: Rename and rewrite the function**

Replace the entire `draw_power_cores_panel` function (lines 547-656 of `src/ui/stats_prestige.rs`) with `draw_deep_panel`. The new function renders 6 content rows:

```rust
/// Draws the unified Deep panel: guild rank, missions, crew, and power core status.
///
/// Shows when The Deep is discovered. 8 rows total (6 content + 2 border).
pub(super) fn draw_deep_panel(
    frame: &mut Frame,
    area: Rect,
    achievements: &crate::achievements::Achievements,
    deep: &DeepState,
) {
    const AMBER: Color = Color::Rgb(220, 180, 60);
    const CORE_AMBER: Color = Color::Rgb(255, 165, 0);

    if !deep.persistent.discovered {
        return;
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" The Deep ")
        .border_style(Style::default().fg(super::themed_border_color(CORE_AMBER)));
    let inner = super::render_themed_block(frame, area, block, CORE_AMBER, super::BorderFxContext);

    let mut lines: Vec<Line> = Vec::new();
    let width = inner.width as usize;

    // Row 1: Guild rank + Warband Marks
    {
        let rank_name = deep.persistent.guild_rank.display_name();
        let marks = deep.prestige.warband_marks;
        let marks_str = format!("\u{25c6} {} Warband Marks", marks);
        let rank_part = format!("\u{2b21} {}", rank_name);
        let padding = width.saturating_sub(rank_part.len() + marks_str.len());

        lines.push(Line::from(vec![
            Span::styled("\u{2b21} ", Style::default().fg(Color::White)),
            Span::styled(rank_name.to_string(), Style::default().fg(Color::White)),
            Span::raw(" ".repeat(padding)),
            Span::styled(
                format!("\u{25c6} ", ),
                Style::default().fg(AMBER),
            ),
            Span::styled(
                format!("{} Warband Marks", marks),
                Style::default().fg(AMBER),
            ),
        ]));
    }

    // Row 2: Missions + Next completion timer
    {
        let active = deep.prestige.active_mission_count();
        let max_concurrent = crate::deep::effective_concurrent_missions(
            deep.persistent.guild_rank,
            deep.persistent.deepest_layer_reached,
        );
        let mission_str = format!("Missions {}/{}", active, max_concurrent);

        let eta = next_mission_eta_secs(&deep.prestige);
        let eta_str = match eta {
            Some(secs) => format!("\u{25f7} Next: ~{}", format_eta(secs as u64)),
            None => "\u{25f7} idle".to_string(),
        };
        let eta_color = match eta {
            Some(secs) if secs < 900 => Color::Yellow,
            Some(_) => Color::DarkGray,
            None => Color::DarkGray,
        };

        let padding = width.saturating_sub(mission_str.len() + eta_str.len());

        lines.push(Line::from(vec![
            Span::styled(mission_str, Style::default().fg(Color::Cyan)),
            Span::raw(" ".repeat(padding)),
            Span::styled(eta_str, Style::default().fg(eta_color)),
        ]));
    }

    // Row 3: Crew glyphs + Frontier + Events
    {
        let mut crew_spans: Vec<Span> = Vec::new();
        let mut available: Vec<&crate::deep::Mercenary> = Vec::new();
        let mut on_mission: Vec<&crate::deep::Mercenary> = Vec::new();
        let mut injured: Vec<&crate::deep::Mercenary> = Vec::new();

        for merc in &deep.prestige.roster {
            match merc.status {
                crate::deep::MercStatus::Available => available.push(merc),
                crate::deep::MercStatus::OnMission(_) => on_mission.push(merc),
                crate::deep::MercStatus::Injured { .. } => injured.push(merc),
                crate::deep::MercStatus::Lost => {} // skip
            }
        }

        // Available mercs: ♦ (green)
        if !available.is_empty() {
            crew_spans.push(Span::styled(
                "\u{2666}".repeat(available.len()),
                Style::default().fg(Color::Green),
            ));
        }
        // Space between groups
        if !available.is_empty() && (!on_mission.is_empty() || !injured.is_empty()) {
            crew_spans.push(Span::raw(" "));
        }
        // On mission: ♢ (cyan)
        if !on_mission.is_empty() {
            crew_spans.push(Span::styled(
                "\u{2662}".repeat(on_mission.len()),
                Style::default().fg(Color::Cyan),
            ));
        }
        if !on_mission.is_empty() && !injured.is_empty() {
            crew_spans.push(Span::raw(" "));
        }
        // Injured: ✝ (red)
        if !injured.is_empty() {
            crew_spans.push(Span::styled(
                "\u{271d}".repeat(injured.len()),
                Style::default().fg(Color::Red),
            ));
        }

        let crew_width: usize = available.len() + on_mission.len() + injured.len()
            + if !available.is_empty() && (!on_mission.is_empty() || !injured.is_empty()) { 1 } else { 0 }
            + if !on_mission.is_empty() && !injured.is_empty() { 1 } else { 0 };

        // Right side: Frontier + events
        let frontier = deep.persistent.frontier_layer();
        let events = pending_event_count(&deep.prestige);
        let frontier_str = format!("Frontier L{}", frontier);
        let event_str = if events > 0 {
            format!("  \u{26a1}{}", events)
        } else {
            String::new()
        };
        let right_str_len = frontier_str.len() + event_str.len();

        let padding = width.saturating_sub(crew_width + right_str_len);
        crew_spans.push(Span::raw(" ".repeat(padding)));
        crew_spans.push(Span::styled(
            frontier_str,
            Style::default().fg(Color::Rgb(120, 140, 170)),
        ));
        if events > 0 {
            crew_spans.push(Span::styled(
                event_str,
                Style::default().fg(Color::Yellow),
            ));
        }

        lines.push(Line::from(crew_spans));
    }

    // Row 4: Separator
    {
        let sep = "\u{2500}".repeat(width);
        lines.push(Line::from(Span::styled(
            sep,
            Style::default().fg(Color::DarkGray),
        )));
    }

    // Rows 5-6: Core summary + badges
    let summary = core_summary(achievements, deep);
    {
        // Row 5: "Cores: N ✓ Ready (+X PR)  ·  Next: Xh Ym" or "Cores: locked ..."
        let mut spans: Vec<Span> = Vec::new();

        if summary.unlocked_count == 0 {
            let left = "Cores: locked";
            let right = "First core at L3";
            let padding = width.saturating_sub(left.len() + right.len());
            spans.push(Span::styled(left.to_string(), Style::default().fg(Color::DarkGray)));
            spans.push(Span::raw(" ".repeat(padding)));
            spans.push(Span::styled(right.to_string(), Style::default().fg(Color::DarkGray)));
        } else if summary.ready_count > 0 && summary.next_ready_secs.is_none() {
            // All unlocked cores are ready
            let left = format!(
                "Cores: {} \u{2713} Ready (+{} PR)",
                summary.ready_count, summary.ready_pr
            );
            let right = "All ready!";
            let padding = width.saturating_sub(left.len() + 4 + right.len());
            spans.push(Span::styled("Cores: ", Style::default().fg(Color::DarkGray)));
            spans.push(Span::styled(
                format!("{} \u{2713} Ready", summary.ready_count),
                Style::default().fg(Color::Green),
            ));
            spans.push(Span::styled(
                format!(" (+{} PR)", summary.ready_pr),
                Style::default().fg(Color::Green),
            ));
            spans.push(Span::raw(" ".repeat(padding.saturating_sub("Cores: ".len()))));
            spans.push(Span::styled(
                "All ready!".to_string(),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            let ready_part = if summary.ready_count > 0 {
                format!(
                    "{} \u{2713} Ready (+{} PR)",
                    summary.ready_count, summary.ready_pr
                )
            } else {
                "0 \u{2713} Ready".to_string()
            };
            let next_part = match summary.next_ready_secs {
                Some(secs) => format!("Next: {}", format_eta(secs as u64)),
                None => String::new(),
            };
            let left_len = "Cores: ".len() + ready_part.len();
            let right_len = next_part.len();
            let padding = width.saturating_sub(left_len + 5 + right_len);

            spans.push(Span::styled("Cores: ", Style::default().fg(Color::DarkGray)));
            if summary.ready_count > 0 {
                spans.push(Span::styled(
                    ready_part,
                    Style::default().fg(Color::Green),
                ));
            } else {
                spans.push(Span::styled(
                    ready_part,
                    Style::default().fg(Color::DarkGray),
                ));
            }
            spans.push(Span::raw(" ".repeat(padding)));
            spans.push(Span::styled(
                format!("  \u{00b7}  {}", next_part),
                Style::default().fg(Color::DarkGray),
            ));
        }

        lines.push(Line::from(spans));
    }

    {
        // Row 6: Per-core badges + PR/day
        let mut spans: Vec<Span> = Vec::new();

        for (i, badge) in summary.cores.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" "));
            }
            if badge.unlocked {
                spans.push(Span::styled(
                    "\u{2742}",
                    Style::default().fg(CORE_AMBER),
                ));
                if badge.ready {
                    spans.push(Span::styled(
                        "\u{2713}",
                        Style::default().fg(Color::Green),
                    ));
                } else {
                    let time = format_core_time_short(badge.remaining_secs);
                    spans.push(Span::styled(
                        time,
                        Style::default().fg(Color::DarkGray),
                    ));
                }
            } else {
                spans.push(Span::styled(
                    format!("\u{25c7}L{}", badge.required_layer),
                    Style::default().fg(Color::DarkGray),
                ));
            }
        }

        // Right-align PR/day
        let pr_str = format!("+{} PR/day", summary.total_pr_per_day);
        // Calculate current badge width for padding
        let badge_text: String = summary.cores.iter().enumerate().map(|(i, b)| {
            let prefix = if i > 0 { " " } else { "" };
            if b.unlocked {
                if b.ready {
                    format!("{}\u{2742}\u{2713}", prefix)
                } else {
                    format!("{}\u{2742}{}", prefix, format_core_time_short(b.remaining_secs))
                }
            } else {
                format!("{}\u{25c7}L{}", prefix, b.required_layer)
            }
        }).collect();
        let badge_width = badge_text.chars().count();
        let padding = width.saturating_sub(badge_width + pr_str.len());

        if summary.total_pr_per_day > 0 {
            spans.push(Span::raw(" ".repeat(padding)));
            spans.push(Span::styled(pr_str, Style::default().fg(CORE_AMBER)));
        }

        lines.push(Line::from(spans));
    }

    let para = Paragraph::new(lines);
    frame.render_widget(para, inner);
}

/// Format seconds into short form for core badges: "1h", "2h", "45m", "11h"
fn format_core_time_short(secs: i64) -> String {
    let secs = secs.max(0) as u64;
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    if hours > 0 {
        format!("{}h", hours)
    } else {
        format!("{}m", minutes.max(1))
    }
}
```

**Step 2: Run clippy and fix any issues**

Run: `cargo clippy --all-targets -- -D warnings`

**Step 3: Commit**

```bash
git add src/ui/stats_prestige.rs
git commit -m "feat: replace Power Cores panel with unified Deep panel"
```

---

### Task 5: Update stats_panel.rs to use draw_deep_panel

**Files:**
- Modify: `src/ui/stats_panel.rs`

**Step 1: Update the import**

In `src/ui/stats_panel.rs` line 7, change the import:
```rust
// Before:
use super::stats_prestige::{
    draw_fishing_panel, draw_power_cores_panel, draw_prestige_info, format_eta,
};

// After:
use super::stats_prestige::{
    draw_deep_panel, draw_fishing_panel, draw_prestige_info, format_eta,
};
```

**Step 2: Update the visibility condition**

Replace the `unlocked_cores` / `power_cores_height` logic (lines 82-87) with:

```rust
// Before:
let unlocked_cores = crate::power_cores::get_unlocked_cores(achievements).len();
let power_cores_height = if unlocked_cores > 0 {
    crate::power_cores::ALL_POWER_CORES.len() as u16 + 2
} else {
    0
};

// After:
let deep_panel_height: u16 = if deep.persistent.discovered { 8 } else { 0 };
```

**Step 3: Update the constraint**

```rust
// Before:
if power_cores_height > 0 {
    constraints.push(Constraint::Length(power_cores_height));
}

// After:
if deep_panel_height > 0 {
    constraints.push(Constraint::Length(deep_panel_height));
}
```

**Step 4: Update the render call**

```rust
// Before:
if power_cores_height > 0 {
    draw_power_cores_panel(frame, chunks[idx], achievements, deep);
    idx += 1;
}

// After:
if deep_panel_height > 0 {
    draw_deep_panel(frame, chunks[idx], achievements, deep);
    idx += 1;
}
```

**Step 5: Run the full check**

Run: `make check`
Expected: All checks pass (format, clippy, tests, build)

**Step 6: Commit**

```bash
git add src/ui/stats_panel.rs
git commit -m "feat: wire draw_deep_panel into stats panel layout"
```

---

### Task 6: Clean up old draw_power_cores_panel references

**Files:**
- Modify: `src/ui/stats_prestige.rs` — remove old function if not already replaced
- Modify: `src/ui/mod.rs` — check for any remaining references

**Step 1: Search for remaining references**

Run: `cargo build 2>&1 | grep -i "power_cores_panel\|unused"` to check for dead code warnings.

**Step 2: Remove any dead code**

Remove `draw_power_cores_panel` if it still exists alongside `draw_deep_panel`. Remove any unused imports.

**Step 3: Run full checks**

Run: `make check`
Expected: All checks pass with no warnings

**Step 4: Commit**

```bash
git add -A
git commit -m "chore: remove dead draw_power_cores_panel code"
```

---

### Task 7: Visual verification and edge case testing

**Step 1: Run the game and verify all states**

Run: `cargo run`

Verify:
- Deep not discovered → panel is hidden
- Deep discovered, no cores → "Cores: locked" row with L3 target
- Cores filling → time remaining shown in badges
- Cores ready → green ✓ badges
- All cores ready → "All ready!" in green bold
- Crew glyphs reflect actual roster state
- Mission count and ETA update correctly
- Events badge shows when missions have pending events

**Step 2: Use debug menu to test different states**

Test with different guild ranks, roster sizes, mission counts, and core unlock levels.

**Step 3: Final commit if any adjustments needed**

```bash
git add -A
git commit -m "fix: adjust Deep panel rendering for edge cases"
```
