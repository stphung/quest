# Deep Hub UI Improvements — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Overhaul The Deep's Hub screen for better scannability, emotional engagement, and edge case handling based on team review.

**Architecture:** Changes are scoped to UI rendering files (`src/ui/deep_*.rs`), UI state (`src/deep/types.rs`), and minor integration points. No game logic changes. All rendering is read-only against `DeepState`.

**Tech Stack:** Rust, Ratatui, `scene_fx` buffer-based rendering

---

## Task 1: Tab System Improvements (types.rs + deep_scene.rs)

**Files:**
- Modify: `src/deep/types.rs` — Reorder `TABS` const, add `EventResponse` to TABS
- Modify: `src/ui/deep_scene.rs:252-372` — `render_tab_bar()` overflow handling

### Step 1: Reorder TABS in types.rs

In `src/deep/types.rs`, change the `TABS` const (line ~1011):

```rust
pub const TABS: &[DeepView] = &[
    DeepView::Hub,
    DeepView::EventResponse,  // moved to 2nd — most time-critical
    DeepView::NewMission,
    DeepView::Roster,
    DeepView::Recruit,
    DeepView::Infrastructure,
];
```

Update `tab_label()` — rename "Event" to "Events":
```rust
DeepView::EventResponse => "Events",
```

### Step 2: Tab bar overflow handling in deep_scene.rs

In `render_tab_bar()` (line ~252), after computing all tab labels, check if total width exceeds available space. If so, use abbreviated labels:

```rust
// After computing total tab width, check overflow
let total_tab_width: usize = DeepView::TABS.iter().enumerate().map(|(i, tab)| {
    let label = tab.tab_label();
    let badge_len = /* compute badge length */;
    if i > 0 { 1 } else { 0 } + label.len() + badge_len + 2 // brackets
}).sum();

let use_abbrev = total_tab_width + 2 > width;

// Abbreviated labels
fn abbrev_label(view: DeepView) -> &'static str {
    match view {
        DeepView::Hub => "H",
        DeepView::EventResponse => "Evt",
        DeepView::NewMission => "Msn",
        DeepView::Roster => "Rst",
        DeepView::Recruit => "Rec",
        DeepView::Infrastructure => "Lyr",
    }
}
```

When `use_abbrev` is true, use `abbrev_label()` and drop badge counts (keep symbols only).

### Step 3: Run `cargo test` and `cargo clippy`

### Step 4: Commit

```
feat(deep-ui): reorder tabs, add overflow abbreviation
```

---

## Task 2: Persistent Status Summary Bar (deep_scene.rs)

**Files:**
- Modify: `src/ui/deep_scene.rs:377-489` — `render_deep_overlay()`

### Step 1: Add status bar rendering

In `render_deep_overlay()`, after painting the backdrop and before rendering the tab bar, render a one-line status summary at row 0. Push the tab bar down to row 1, separator to row 2, and content starts at row 3.

The status bar shows:
```
⬡ {rank_name}   ◆ {marks} Marks   {active}/{max} missions   Next: ~{time}
```

Logic:
- Rank display name from `deep.persistent.guild_rank.display_name()`
- Marks from `deep.prestige.warband_marks`
- Active/max from `deep.prestige.active_mission_count()` / `effective_concurrent_missions()`
- "Next complete" = minimum remaining time across active missions, or "None active"

Colors: Rank in white, Marks in amber RGB(220,180,60), mission count in cyan, time in DarkGray.

### Step 2: Adjust content offset

Change `content_buffer = &mut buffer[2..]` to `&mut buffer[3..]` and update `content_height` accordingly.

### Step 3: Run `cargo test` and `cargo clippy`

### Step 4: Commit

```
feat(deep-ui): add persistent status summary bar above tabs
```

---

## Task 3: Hub Mission List Redesign (deep_missions.rs)

**Files:**
- Modify: `src/ui/deep_missions.rs:266-797` — `render_hub()` rewrite

### Step 1: Labeled section separators

Replace flat `─` separators with labeled rules:
```rust
fn render_section_rule(buffer: &mut [Vec<SceneCell>], row: i32, width: usize, label: &str, count: Option<usize>) {
    let count_str = count.map(|c| format!(" ({})", c)).unwrap_or_default();
    let prefix = format!("── {} ", label);
    let suffix_len = count_str.len();
    let rule_len = width.saturating_sub(prefix.len() + suffix_len + 2);
    let rule: String = "─".repeat(rule_len);
    put_text(buffer, row, 1, &prefix, SECTION_LABEL_COLOR);
    put_text(buffer, row, 1 + prefix.len() as i32, &rule, Color::Rgb(40, 60, 80));
    if !count_str.is_empty() {
        put_text(buffer, row, (width as i32 - suffix_len as i32 - 1), &count_str, Color::DarkGray);
    }
}
```

### Step 2: Pre-attentive mission status glyphs

Replace the current mission rendering with glyph-prefixed 2-line cards:

Completed missions (pending_results):
```
[✓] {type} — Layer {n} {tier}      COLLECT → [Enter]
```

Active missions with events:
```
[!] {type} — Layer {n}   {lead_merc} leads    ⚡ EVENT
    ████████▒▒▒▒  {pct}%   ~{time} left
```

Active missions normal:
```
[▶] {type} — Layer {n}   {lead_merc} leads
    ████████▒▒▒▒  {pct}%   ~{time} left
```

Lead merc = first merc in squad by id lookup.

Sort order: completed first, then event-pending, then by time remaining ascending.

### Step 3: Completed vs active separator

Insert `render_section_rule()` between completed and active sections:
```
── COMPLETED ──────────────── (1)
[✓] ...

── ACTIVE ─────────────────── (2)
[▶] ...
```

### Step 4: Progress bar visual update

Change empty bar character from `░` (U+2591) to `▒` (U+2592) for mission progress bars. Keep `░` for familiarity bars elsewhere.

### Step 5: QA fix — "Resolving..." state

When `progress >= 1.0` and status is not `EventPending`:
```rust
if progress >= 1.0 && !matches!(mission.status, MissionStatus::EventPending) {
    // Show "Resolving..." instead of "0h 00m"
    put_text(buffer, row + 2, 3 + bar_width as i32, "  Resolving...", Color::Green);
}
```

### Step 6: QA fix — progress bar pulse at >95%

When progress > 0.95, alternate bar fill color between normal and brighter variant based on `millis`:
```rust
let bar_color = if progress > 0.95 {
    let pulse = (millis / 500) % 2 == 0;
    if pulse { Color::Rgb(120, 220, 160) } else { tc }
} else { tc };
```

### Step 7: Run `cargo test` and `cargo clippy`

### Step 8: Commit

```
feat(deep-ui): redesign Hub mission list with status glyphs and sections
```

---

## Task 4: Hub Empty State & Onboarding (deep_missions.rs)

**Files:**
- Modify: `src/ui/deep_missions.rs:466-587` — empty state in `render_hub()`

### Step 1: Actionable empty state

Replace centered text tips with a structured action panel:

```rust
// When no active missions and no pending results
let mid = missions_top + remaining_space as i32 / 2;

// Show warband log first if available (keep existing)
// Then show action panel below:
put_text(buffer, action_row, 3, "[N] New Mission", Color::Rgb(80, 160, 220));
put_text(buffer, action_row, 20, "— Send your first squad", Color::DarkGray);
action_row += 1;
put_text(buffer, action_row, 3, "[R] Recruit", Color::Rgb(80, 160, 220));
put_text(buffer, action_row, 20, "— Hire mercenaries", Color::DarkGray);
action_row += 1;
put_text(buffer, action_row, 3, "[L] Layers", Color::Rgb(80, 160, 220));
put_text(buffer, action_row, 20, "— View explored territory", Color::DarkGray);
```

Context-sensitive: only show [N] if there are available missions, show "Supply Runs are free" if marks == 0.

### Step 2: QA fix — injured roster deadlock guidance

When all mercs are injured and no missions active:
```rust
if deep.prestige.roster.iter().all(|m| matches!(m.status, MercStatus::Injured { .. }))
    && deep.prestige.active_missions.is_empty()
{
    put_text(buffer, row, 1,
        "Your mercs are recovering. They'll be ready after the next mission resolves.",
        Color::Rgb(80, 80, 120));
}
```

### Step 3: Run `cargo test` and `cargo clippy`

### Step 4: Commit

```
feat(deep-ui): actionable empty state with context-sensitive guidance
```

---

## Task 5: Amber Marks Color & Marks-to-Goal Display (deep_missions.rs + deep_roster.rs + deep_layers.rs)

**Files:**
- Modify: `src/ui/deep_missions.rs` — all `Color::Yellow` for marks → `Color::Rgb(220, 180, 60)`
- Modify: `src/ui/deep_roster.rs` — same
- Modify: `src/ui/deep_layers.rs` — same
- Modify: `src/ui/deep_results.rs` — same

### Step 1: Define MARKS_COLOR constant

In `src/ui/deep_missions.rs`, add near the top:
```rust
/// Amber color for Warband Marks currency displays.
const MARKS_COLOR: Color = Color::Rgb(220, 180, 60);
```

Replace all `Color::Yellow` that specifically colors marks/costs with `MARKS_COLOR`. Leave non-marks Yellow uses (event badges, warnings) unchanged.

Export from deep_missions for use in sibling files:
```rust
pub(super) const MARKS_COLOR: Color = Color::Rgb(220, 180, 60);
```

### Step 2: Marks relative to next purchase (Hub only)

In the guild status block of `render_hub()`, replace raw marks display:

```rust
// Find cheapest affordable action
let cheapest_recruit = deep.prestige.recruit_pool.recruit_costs.iter().min().copied().unwrap_or(0);
let cheapest_infra = /* find cheapest unbuilt infra across frontier layers */;
let next_goal = if cheapest_recruit > 0 && marks < cheapest_recruit {
    format!("{} / {} — Next recruit", marks, cheapest_recruit)
} else if cheapest_infra > 0 && marks < cheapest_infra {
    format!("{} / {} — Next infrastructure", marks, cheapest_infra)
} else {
    format!("{}", marks)
};
```

### Step 3: Run `cargo test` and `cargo clippy`

### Step 4: Commit

```
feat(deep-ui): amber marks color, marks-to-goal display
```

---

## Task 6: QA Fixes Bundle (types.rs + deep_scene.rs + deep_missions.rs)

**Files:**
- Modify: `src/deep/types.rs` — generation_number default
- Modify: `src/ui/deep_scene.rs` — badge visibility footer
- Modify: `src/ui/deep_missions.rs` — squad name overflow, mission card truncation

### Step 1: Gen.0 fix

In `render_compact_hub()` (deep_missions.rs line ~128):
```rust
let gen_label = format!("Gen.{}", deep.prestige.generation_number.max(1));
```

In `render_hub()` full layout (line ~343), same pattern — already shows conditionally for gen > 1, which is correct.

### Step 2: Squad name overflow

In the mission card rendering, cap squad string:
```rust
let max_squad_w = width.saturating_sub(12);
let squad_display = if squad_str.len() > max_squad_w {
    let first_name = squad_names.first().map(|s| s.as_str()).unwrap_or("");
    format!("{}, +{} others", first_name, squad_names.len() - 1)
} else {
    squad_str
};
```

### Step 3: Mission card truncation

In `render_new_mission_split()` (line ~1258), apply the already-computed `max_name_w`:
```rust
let display_name = &type_name[..type_name.len().min(max_name_w)];
```

### Step 4: Event badge footer reminder

In `render_deep_overlay()` or in each sub-view's footer, when not on Hub/EventResponse tab and events are pending:
```rust
if ui.view != DeepView::Hub && ui.view != DeepView::EventResponse
    && deep.prestige.has_any_pending_event()
{
    put_text(buffer, height - 1, width / 2, "⚡ Event pending — [Tab] to Hub", Color::Yellow);
}
```

### Step 5: Run `cargo test` and `cargo clippy`

### Step 6: Commit

```
fix(deep-ui): Gen.0, squad overflow, card truncation, event badge reminder
```

---

## Task 7: Border Pulse Animation (deep_scene.rs)

**Files:**
- Modify: `src/ui/deep_scene.rs:377-402` — `render_deep_overlay()` border rendering

### Step 1: Slow pulse on border color

Replace the static `DEEP_BORDER_COLOR` with a pulsing variant:

```rust
fn pulsing_border_color(millis: u128) -> Color {
    // 5-second cycle (5000ms), smooth sine wave
    let phase = (millis as f64 / 5000.0 * std::f64::consts::TAU).sin();
    let t = (phase * 0.5 + 0.5) as f64; // 0.0 to 1.0
    let r = (60.0 + t * 40.0) as u8;  // 60-100
    let g = (120.0 + t * 70.0) as u8; // 120-190
    let b = (180.0 + t * 75.0) as u8; // 180-255
    Color::Rgb(r, g, b)
}
```

Apply in `render_deep_overlay()`:
```rust
let border_color = pulsing_border_color(current_millis());
let block = Block::default()
    .title(" THE DEEP ")
    .borders(Borders::ALL)
    .border_style(Style::default().fg(super::themed_border_color(border_color)));
```

### Step 2: Run `cargo test` and `cargo clippy`

### Step 3: Commit

```
feat(deep-ui): slow breathing border pulse animation
```

---

## Task 8: Lead Merc Names + Debrief Flavor (deep_missions.rs + deep_results.rs)

**Files:**
- Modify: `src/ui/deep_missions.rs` — add lead merc name to mission cards
- Modify: `src/ui/deep_results.rs` — add flavor text to debrief

### Step 1: Lead merc name helper

In `deep_missions.rs`, add helper:
```rust
/// Get the lead merc's name (first squad member) for display.
fn lead_merc_name(deep: &DeepState, squad: &[u64]) -> String {
    squad.first()
        .and_then(|id| deep.prestige.find_merc(*id))
        .map(|m| m.name.clone())
        .unwrap_or_else(|| "Unknown".to_string())
}
```

Use in mission card rendering:
```rust
let leader = lead_merc_name(deep, &mission.squad);
// Line 1: [▶] Supply Run — Layer 3   Varek leads
```

### Step 2: Debrief flavor text

In `deep_results.rs`, add tier-based procedural flavor after the rewards section:

```rust
fn debrief_flavor(tier: LayerTier, outcome: MissionOutcome) -> &'static str {
    match (tier, outcome) {
        (LayerTier::Shallows, MissionOutcome::Success) => "The squad reports stable tunnels and breathable air.",
        (LayerTier::Shallows, _) => "The upper passages proved more treacherous than expected.",
        (LayerTier::Warrens, MissionOutcome::Success) => "The Warrens yielded their secrets reluctantly.",
        (LayerTier::Warrens, _) => "Something in the Warrens was waiting for them.",
        (LayerTier::Hollows, MissionOutcome::Success) => "The bioluminescence guided them deeper than planned.",
        (LayerTier::Hollows, _) => "The spore clouds were thicker than the maps suggested.",
        (LayerTier::SunkenReach, MissionOutcome::Success) => "The seals parted for them. Not everyone finds that reassuring.",
        (LayerTier::SunkenReach, _) => "The water pressure claimed equipment. And patience.",
        (LayerTier::Abyss, MissionOutcome::Success) => "They returned with six days of rations consumed. They were gone four hours.",
        (LayerTier::Abyss, _) => "Time moved differently down there. It always does.",
        (LayerTier::Void, MissionOutcome::Success) => "What they found cannot be mapped. Only remembered.",
        (LayerTier::Void, _) => "The Void gives nothing freely. The cost is always personal.",
    }
}
```

Add this flavor line to the mission results modal after the rewards section, styled italic in DarkGray.

### Step 3: Run `cargo test` and `cargo clippy`

### Step 4: Commit

```
feat(deep-ui): lead merc names on missions, debrief flavor text
```

---

## Execution Order

Tasks 1-2 must go first (tab system + status bar change the layout offset).
Tasks 3-8 are independent after that and can be parallelized.

Suggested serial order if not parallelizing:
1 → 2 → 3 → 4 → 5 → 6 → 7 → 8

## Verification

After all tasks:
```bash
make check   # Full CI: fmt, clippy, test, build, audit
cargo run    # Visual verification with --debug flag to trigger Deep
```
