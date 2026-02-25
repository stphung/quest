# The Deep — Hub View and Mission Assignment UI Design

**Author:** UX Designer Agent
**Date:** 2026-02-23
**Updated:** 2026-02-23 (incorporating findings from `docs/plans/deep-ui-audit.md`)
**Scope:** Tasks #2 and #3 — Hub view information hierarchy and Mission assignment flow clarity
**Status:** Final — ready for implementation

---

## Executive Summary

After auditing all Deep UI files (`deep_missions.rs`, `deep_scene.rs`, `deep_roster.rs`, `deep_layers.rs`, `deep_events.rs`, `deep_input.rs`, `types.rs`, `CLAUDE.md`) plus cross-referencing the system audit (`docs/plans/deep-ui-audit.md`), I identified concrete gaps in information hierarchy, progressive disclosure, and player feedback. This document covers the Hub view and Mission assignment flow with ASCII mockups, specific color values, and exact function changes needed.

**Audit findings incorporated (P0 and P1 that touch Hub/Mission scope):**
- P0.2: Mission descriptions exist but are never shown — addressed in Part 2
- P0.3: Construction missions drop infrastructure type in labels — addressed in Part 2 Change 7
- P1.2: Guild rank upgrade path completely hidden — addressed in Part 1 Change 1
- P1.3: Tab bar has no state indicators — addressed in Part 1 Change 6 (new)
- P1.4: Power ratio should show percentage — addressed in Part 2 Change 2
- Bug 3.3: "No missions available" empty state misleading — addressed in Part 2 Change 8
- Bug 3.7: Flash message positioning conflict with content — addressed in Part 2 Change 9

---

## Part 1: Hub View Redesign

### 1.1 Current State Problems

**Identified issues in `render_hub()` (`deep_missions.rs:98-260`):**

1. **Flat information hierarchy** — Guild status header (row 0) shows rank, marks, roster, and frontier in a single dense string with no visual weight differentiation. The player cannot scan quickly.

2. **Missing guild progression context** — No indication of what's needed to advance guild rank. A new player cannot understand they need a Layer 3 breakthrough to reach Rank 2.

3. **Mission cards lack urgency signaling** — Completed missions and event-pending missions look nearly identical to active missions. The `⚡ Event pending!` suffix is correct in intent but gets buried in the line.

4. **Progress bar placement** — The bar is on row+1, which means you read the mission label, then look down for context. The opposite order from what's natural.

5. **Empty state misses opportunity** — "No active missions." + "[Tab] to Missions view" works but does nothing to orient a new player. No mention of what the Deep *is*, what Warband Marks are, or what to do first.

6. **Marks balance not prominent** — Marks appear in the header string. Before launching missions (which cost Marks), players need to see balance as a first-class status.

7. **Roster availability not surfaced** — No indication of how many mercs are free vs. on missions. Players can't tell if they have capacity to launch another mission.

8. **No mission slot capacity display** — Concurrent mission limit (from guild rank) is invisible. Players don't know if they've hit the cap.

### 1.2 Redesigned Hub View

#### Layout Structure

```
┌─ THE DEEP ─────────────────────────────────────────────────────────────┐
│ [Hub] [Missions] [Roster] [Layers] [Recruit]                           │  ← tab bar (existing)
│────────────────────────────────────────────────────────────────────────│
│ GUILD STATUS                                                           │  ← section A: guild block
│ Rank 2 — Sellswords        Mercs: 3/7   Missions: 1/1   ◆ 240 Marks   │
│ Frontier: Layer 3 (The Warrens)   Deepest: Layer 3                    │
│ Next rank needs: Layer 3 Breakthrough                                  │  ← only shown if can advance
│────────────────────────────────────────────────────────────────────────│
│ ACTIVE MISSIONS                                                        │  ← section B
│ ▶ [Expedition]  Layer 3 — The Warrens          ⚡ Event pending!       │  ← urgent state first
│   Squad: Gareth (Vanguard), Lyra (Scout)                              │
│   ██████████░░░░░░░░░░░░░░░░░  65%   2h 10m remaining                 │  ← bar then time
│ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─  │  ← soft divider
│   [Supply Run]  Layer 1 — The Shallows          43%   3h 45m left     │
│   Squad: Aldric (Medic)                                               │
│   ██████░░░░░░░░░░░░░░░░░░░░░░  43%                                    │
│────────────────────────────────────────────────────────────────────────│
│  [Tab] Switch view   [Enter] Select mission   [Esc] Close              │
└────────────────────────────────────────────────────────────────────────┘
```

#### Empty State (first-time player)

```
┌─ THE DEEP ─────────────────────────────────────────────────────────────┐
│ [Hub] [Missions] [Roster] [Layers] [Recruit]                           │
│────────────────────────────────────────────────────────────────────────│
│ GUILD STATUS                                                           │
│ Rank 1 — Freelancers        Mercs: 3/5   Missions: 0/1   ◆ 0 Marks    │
│ Frontier: Layer 1 (The Shallows)                                       │
│ Next rank needs: Layer 3 Breakthrough                                  │
│────────────────────────────────────────────────────────────────────────│
│                                                                        │
│                     No active missions.                                │
│                                                                        │
│        Your mercenary company is ready to descend.                     │
│        [Missions] tab → pick a mission and assign your squad.          │
│        Earn Warband Marks to grow your roster and buy infrastructure.  │
│                                                                        │
│────────────────────────────────────────────────────────────────────────│
│  [Tab] Switch view   [Enter] Select mission   [Esc] Close              │
└────────────────────────────────────────────────────────────────────────┘
```

#### Completed Mission (result waiting)

```
│ ✓ [Expedition]  Layer 2 — The Shallows         COMPLETE — collect!    │
│   Squad: Gareth, Lyra    ██████████████████████████████  100%         │
│   [Enter] to collect rewards                                           │
```

### 1.3 Information Hierarchy Principles Applied

**Visual weight hierarchy (lightest to heaviest):**
- Section labels (`GUILD STATUS`, `ACTIVE MISSIONS`): `Color::Rgb(80, 160, 220)` — deep blue
- Supporting context (frontier, deepest): `Color::DarkGray`
- Guild name: `Color::White`
- Marks balance: `Color::Yellow` with `◆` prefix for scannability
- Mission type: mission type color (existing `mission_type_color()`)
- Event pending badge: `Color::Yellow` with `⚡` prefix — rightmost, conspicuous
- Completed badge: `Color::Green` with `✓` prefix

**Ordering within mission card (top to bottom):**
1. Mission type + layer + tier + status badge (scan line)
2. Squad names (who is deployed)
3. Progress bar + percentage + time remaining (how far along)

The previous ordering had the scan line then bar then squad. Time remaining is more actionable than squad identity at a glance, so bar+time before squad.

### 1.4 Specific Code Changes — Hub View

**File:** `src/ui/deep_missions.rs`

#### Change 1: Guild status block (`render_hub()` lines 117-137)

Replace the two-line flat header with a structured 4-row guild block:

```rust
// ── Guild Status Block (rows 0-3) ──
let rank = deep.persistent.guild_rank;
let marks = deep.prestige.warband_marks;
let roster_count = deep.prestige.roster.len();
let available_mercs = deep.prestige.available_merc_count();
let max_roster = rank.max_roster() as usize;
let active_count = deep.prestige.active_mission_count() as u32;
let max_concurrent = rank.concurrent_missions();
let frontier = deep.persistent.frontier_layer();
let deepest = deep.persistent.deepest_layer_reached;

// Row 0: Section label
put_text(buffer, 0, 1, "GUILD STATUS", SECTION_LABEL_COLOR);

// Row 1: Rank name + key numeric stats
let marks_str = format!("\u{25c6} {} Marks", marks); // ◆ symbol
let rank_line = format!(
    "Rank {} \u{2014} {:12}  Mercs: {}/{}   Missions: {}/{}   {}",
    rank.0,
    rank.display_name(),
    roster_count, max_roster,
    active_count, max_concurrent,
    marks_str,
);
put_text(buffer, 1, 1, &rank_line, Color::White);
// Recolor marks ◆ in Yellow
let marks_col = rank_line.find('\u{25c6}').unwrap_or(0) as i32 + 1;
put_text(buffer, 1, marks_col, &marks_str, Color::Yellow);

// Row 2: Frontier info
let frontier_tier = crate::deep::LayerTier::from_layer(frontier);
let frontier_str = format!(
    "Frontier: Layer {} ({})   Deepest ever: Layer {}",
    frontier,
    frontier_tier.display_name(),
    deepest.max(1),
);
put_text(buffer, 2, 1, &frontier_str, Color::DarkGray);

// Row 3 (optional): Next rank requirement
if rank.can_advance() {
    if let Some(next) = rank.next() {
        if let Some(needed_layer) = next.required_breakthrough_layer() {
            let progress_text = format!(
                "Advance to {}: complete Layer {} Breakthrough",
                next.display_name(), needed_layer
            );
            put_text(buffer, 3, 1, &progress_text, Color::Rgb(50, 100, 60));
        }
    }
}

// ── Separator ──
let sep_row = 4i32;
let sep: String = "\u{2500}".repeat(width.saturating_sub(2));
put_text(buffer, sep_row, 1, &sep, Color::Rgb(40, 60, 80));
```

#### Change 2: Mission card order and content (`render_hub()` active mission loop)

Reorder card to: status line → squad line → bar line. Add time remaining to bar line.

```rust
// Line 1: Mission type + layer + tier + status badge
let total_secs = (mission.ends_at - mission.started_at).num_seconds().max(1) as u64;
let elapsed_secs = (now - mission.started_at).num_seconds().max(0) as u64;
let remaining_secs = total_secs.saturating_sub(elapsed_secs);

let status_badge = match &mission.status {
    MissionStatus::EventPending => "  \u{26a1} EVENT PENDING",   // ⚡
    MissionStatus::Completed    => "  \u{2713} COMPLETE \u{2014} [Enter]", // ✓
    _                           => "",
};
let badge_color = match &mission.status {
    MissionStatus::EventPending => Color::Yellow,
    MissionStatus::Completed    => Color::Green,
    _                           => tc,
};

let tier_name = crate::deep::LayerTier::from_layer(mission.layer).display_name();
let line1 = format!(
    "{}[{}]  Layer {} \u{2014} {}{}",
    cursor,
    type_name,
    mission.layer,
    tier_name,
    status_badge,
);
put_text(buffer, row, 1, &line1, tc);
put_text(buffer, row, 1, cursor, if is_selected { Color::Cyan } else { Color::DarkGray });
// Recolor badge
if !status_badge.is_empty() {
    let badge_col = line1.find('\u{26a1}').or_else(|| line1.find('\u{2713}')).unwrap_or(0) as i32 + 1;
    put_text(buffer, row, badge_col, status_badge.trim(), badge_color);
}

// Line 2: Squad names
let squad_label = format!("  Squad: {}", squad_str);
put_text(buffer, row + 1, 1, &squad_label, Color::DarkGray);

// Line 3: Progress bar + % + time remaining
let pct = (progress * 100.0) as u32;
let bar_width = (width.saturating_sub(20)).min(28);
render_progress_bar(buffer, row + 2, 3, bar_width, progress, tc);
let time_str = if remaining_secs > 0 {
    format!("  {}%   {} left", pct, format_hours(remaining_secs))
} else {
    format!("  {}%   done", pct)
};
put_text(buffer, row + 2, 3 + bar_width as i32, &time_str, Color::DarkGray);

row += 4; // 3 content rows + 1 blank gap between missions
```

#### Change 3: Empty state copy

```rust
// Three-line empty state with action guidance
let mid = missions_top + content_height as i32 / 2;
put_text_centered(buffer, mid - 1, width, "No active missions.", Color::DarkGray);
put_text_centered(buffer, mid, width, "Your company is ready to descend.", Color::Rgb(60, 80, 120));
put_text_centered(buffer, mid + 1, width, "[Missions] tab \u{2192} pick a mission and assign squad.", Color::Rgb(50, 70, 100));
if deep.prestige.warband_marks == 0 {
    put_text_centered(buffer, mid + 2, width, "Supply Runs are free \u{2014} start there.", Color::Rgb(40, 80, 50));
}
```

#### Change 4: Add `SECTION_LABEL_COLOR` constant

At the top of `deep_missions.rs`:

```rust
const SECTION_LABEL_COLOR: Color = Color::Rgb(80, 160, 220);
```

#### Change 5: Compact mode adjustments

For S-tier, condense guild block to 2 rows:
- Row 0: `Rank 1 Freelancers  3/5 Mercs  0/1 Missions  ◆240M`
- Row 1: `Frontier: L3 Warrens` (no next-rank progression text)
- Skip separator, go straight to missions

#### Change 6: Tab bar state indicators (NEW — from audit P1.3)

**File:** `src/ui/deep_scene.rs` — `render_tab_bar()` function (lines 119-138)

The current tab bar shows only the active tab highlighted. Add compact state badges after each label:

| Tab | Badge condition | Badge | Color |
|-----|----------------|-------|-------|
| Hub | N results await | `✓N` | `Green` |
| Hub | N events pending | `⚡N` | `Yellow` |
| Missions | N missions available | `·N` | `Cyan` |
| Roster | N mercs injured/lost | `!N` | `Yellow` |
| Recruit | Pool has candidates | `·` | `Cyan` |

The badge is appended inside the `[Label]` brackets: `[Hub⚡2]` or `[Roster!1]`.

```rust
fn render_tab_bar(buffer: &mut [Vec<SceneCell>], width: usize, active: DeepView, deep: &DeepState) {
    let mut col = 1i32;
    for (i, &tab) in DeepView::TABS.iter().enumerate() {
        if i > 0 {
            put_text(buffer, 0, col, " ", Color::DarkGray);
            col += 1;
        }

        // Compute badge for this tab
        let (badge, badge_color) = match tab {
            DeepView::Hub => {
                let events = deep.prestige.active_missions.iter()
                    .filter(|m| m.has_pending_event()).count();
                let results = deep.prestige.pending_results.len();
                if events > 0 {
                    (format!("\u{26a1}{}", events), Color::Yellow)
                } else if results > 0 {
                    (format!("\u{2713}{}", results), Color::Green)
                } else {
                    (String::new(), Color::DarkGray)
                }
            }
            DeepView::NewMission => {
                let n = deep.prestige.available_missions.len();
                if n > 0 { (format!("\u{00b7}{}", n), Color::Cyan) }
                else { (String::new(), Color::DarkGray) }
            }
            DeepView::Roster => {
                let injured = deep.prestige.roster.iter()
                    .filter(|m| matches!(m.status, MercStatus::Injured { .. } | MercStatus::Lost))
                    .count();
                if injured > 0 { (format!("!{}", injured), Color::Yellow) }
                else { (String::new(), Color::DarkGray) }
            }
            DeepView::Recruit => {
                let n = deep.prestige.recruit_pool.candidates.len();
                if n > 0 { ("\u{00b7}".to_string(), Color::Cyan) }
                else { (String::new(), Color::DarkGray) }
            }
            _ => (String::new(), Color::DarkGray),
        };

        let label = tab.tab_label();
        let full = if badge.is_empty() {
            format!("[{}]", label)
        } else {
            format!("[{}{}]", label, badge)
        };
        let tab_color = if tab == active {
            Color::Rgb(80, 160, 220)
        } else {
            Color::DarkGray
        };
        put_text(buffer, 0, col, &full, tab_color);
        // Overcolor badge portion only when not active
        if !badge.is_empty() && tab != active {
            let badge_col = col + 1 + label.len() as i32;
            put_text(buffer, 0, badge_col, &badge, badge_color);
        }
        col += full.len() as i32;
    }

    // Separator (existing logic, unchanged)
    let sep: String = "\u{2500}".repeat(width.saturating_sub(2));
    let remaining = (width as i32 - col - 1).max(0) as usize;
    put_text(buffer, 0, col + 1, &sep[..remaining.min(sep.len())], Color::Rgb(40, 60, 80));
}
```

Note: `render_tab_bar()` signature gains a `deep: &DeepState` parameter. Update the call site in `render_deep_overlay()` accordingly.

---

## Part 2: Mission Assignment Flow Redesign

### 2.1 Current State Problems

**Identified issues in `render_new_mission()` and `render_new_mission_split()` (`deep_missions.rs:264-598`):**

1. **Mission type mystery** — Each mission in the list shows `[Supply Run]  L1  2h  Safe` but never explains what a Supply Run *is*. Players unfamiliar with the system must Tab to Roster, look around, and guess. The `description` field on `AvailableMission` exists but is never rendered.

2. **Power requirement ambiguity** — `Requires: Min Power 25` tells players a number but not whether their current squad meets it. The ratio feedback exists in the summary row (at the bottom) but the two pieces of information are visually separated, forcing vertical scanning.

3. **Cost balance not co-located** — Mission cost appears in the list line as `25M` but current Marks balance only appears in the Hub view. Players leave the Missions tab, note their balance, come back, and verify — unnecessary cognitive round-trip.

4. **Archetype requirements buried** — "Arcanist required" appears below duration and cost in the detail panel. Required archetypes gate entire missions and should be prominently flagged, especially when the player lacks that archetype.

5. **Squad assignment flow is two-phase but feels one-phase** — The list → squad picker flow is correct but poorly communicated. The split-panel design shows the squad picker immediately in the right panel without indicating that [Enter] activates it and changes the cursor target. New players think the list is the whole experience.

6. **No archetype squad summary** — When building a squad, players see names and levels but don't see a quick archetype summary. If a mission wants a Saboteur, players need to remember which merc is which archetype.

7. **Success probability labels are inconsistent** — "60-90% success" and "Good odds" say the same thing redundantly. "Overpowered — 95% success, faster" is the most useful line but feels like a bonus, not a feature.

8. **No feedback on duration modifiers** — The displayed duration (`2h`) is the base duration. Players don't know that their Outpost on Layer 1 is already factored in, or that adding a Saboteur would reduce it further.

9. **Available Marks not shown in Missions view** — If cost > 0, the number shows in the list. But current balance is invisible. A player with 20 Marks looking at a 25-Mark mission has no direct affordability signal.

10. **Construction mission type is opaque** (audit P0.3) — `MissionType::Construction(Infrastructure::Outpost)` renders as `"Construction"` in the list. Players cannot see what infrastructure they are building without opening the detail panel.

11. **Power ratio lacks percentage context** (audit P1.4) — `Power: 32/25` is shown but `128%` — the number that maps to a success band — is never computed or displayed. Players must do math to understand whether 32 is well over or barely over threshold.

12. **"No missions available" message is misleading** (audit bug 3.3) — On a fresh prestige with nothing queued, "Complete active missions to refresh the pool" gives wrong guidance. The pool refresh is time-based and independent of mission completion.

13. **Flash message overlaps content** (audit bug 3.7) — `flash_message` is rendered at `height - 2` but `content_bottom` is also `height - 2`. Long mission lists can write over the flash message row before it can be read.

### 2.2 Redesigned Mission Assignment Flow

#### Phase 1: Mission List (before Enter)

```
┌ AVAILABLE MISSIONS ──────────────────┬─ MISSION DETAIL ────────────────┐
│ AVAILABLE                            │ Layer 3 — The Warrens           │
│                                      │                                 │
│ ▶ [Expedition]  L3  10h  Medium  20M │ Expedition                      │
│   [Supply Run]  L1  2h   Safe    —   │ Primary progression mission.    │
│   [Recon]       L3  6h   Low     5M  │ Longer, riskier than Recon,     │
│                                      │ earns more Marks and XP.        │
│                                      │                                 │
│                                      │ Duration:  10h (base)           │
│                                      │ Risk:      Medium               │
│                                      │ Cost:      20 Marks  (have 240) │
│                                      │ Reward:    Marks + XP + items   │
│                                      │                                 │
│                                      │ Requires:                       │
│                                      │   Min Power  100                │
│                                      │ ⚠ Arcanist required (none in    │
│                                      │   roster!) — can still attempt  │
│                                      │ ★ Scout recommended             │
│                                      │                                 │
│                                      │ [Enter] to assign squad         │
└──────────────────────────────────────┴─────────────────────────────────┘
  [↑/↓] Select Mission   [Enter] Assign Squad   [Esc] Back         ◆ 240 M
```

#### Phase 2: Squad Assignment (after Enter)

```
┌ ASSIGN SQUAD ────────────────────────┬─ SQUAD SUMMARY ─────────────────┐
│ Expedition  Layer 3  10h  20M        │ Cost: 20 Marks   Balance: 240   │
│                                      │                                 │
│ [↑/↓] Select   [Space] Toggle        │ Squad Power: 68 / 100           │
│                                      │ ████████░░░░░░░░░░░░░░░░░░░░   │
│ [ ] Gareth      Vanguard  L3  P:20   │ Risky — ~30% success            │
│ [✓] Lyra        Scout     L2  P:14   │                                 │
│ [ ] Aldric      Medic     L1  P: 8   │ Archetypes in squad:            │
│ [ ] Theron      Arcanist  L4  P:16 ← │   Scout (Lyra)                 │
│     Vex         Saboteur  L2  P:12   │   (!) Arcanist required         │
│     (injured: 2 missions)            │   ★  Scout recommended — present│
│                                      │                                 │
│                                      │ Add Theron (Arcanist) to meet   │
│                                      │ the requirement. Power: 84/100  │
│                                      │ Risky — ~30% (Arcanist present) │
│                                      │                                 │
│                                      │ [Enter] Launch Mission          │
└──────────────────────────────────────┴─────────────────────────────────┘
  [Space] Toggle   [Enter] Launch Mission   [Esc] Cancel         ◆ 220 M
```

### 2.3 Phase 1 Design Specifics

#### Mission list line format

Current: `▶ [Supply Run]  L1  2h  Safe`
Proposed: `▶ [Supply Run]  L1  2h   Safe   —`

For compactness the list stays the same width. The innovation is in the **detail panel**.

#### Detail panel improvements

**New fields to render:**

1. **Mission description** — Render `available_mission.description` field (already exists in `AvailableMission` struct, currently unused in UI). Wrap at 2 lines max with `detail_inner_w` constraint.

2. **Marks balance co-located with cost:**
   ```
   Cost:     20 Marks   (have 240)
   ```
   - If `marks >= cost`: render `(have N)` in `Color::Rgb(60, 180, 80)` (soft green)
   - If `marks < cost`: render `(have N — INSUFFICIENT)` in `Color::LightRed`
   - If `marks_cost == 0`: render `Cost:     Free`

3. **Archetype requirement prominence:**
   - Required archetype: Show with `⚠` prefix in `Color::Yellow` if archetype missing from roster, `Color::Green` if present
   - Recommended archetype: Show with `★` prefix in `Color::Cyan` if present, `Color::DarkGray` if absent
   - Check the full roster (not just available mercs) since players may want to know if they have the archetype at all

4. **"[Enter] to assign squad" hint** at bottom of detail panel, `Color::Rgb(50, 100, 50)`

5. **Duration modifier hint** (when infrastructure active):
   ```
   Duration:  10h   (Outpost: -25% → 7.5h effective)
   ```
   Query `deep.persistent.layer_record(m.layer)` for infrastructure and familiarity; show effective time if modifiers exist.

### 2.4 Phase 2 Design Specifics

#### Left panel changes (merc list during squad staging)

Current: Shows all roster mercs including unavailable.
Proposed: Separate available vs. unavailable with a visual group break.

```
Available:
  [✓] Gareth    Vanguard  L3  Pwr:20
  [ ] Lyra      Scout     L2  Pwr:14
  [ ] Theron    Arcanist  L4  Pwr:16
─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─
Unavailable:
      Vex       Saboteur  (on mission)
      Aldric    Medic     (injured: 2)
```

The divider `─ ─ ─` uses `Color::Rgb(40, 60, 80)`. Unavailable mercs are not selectable (cursor skips them).

#### Right panel (squad summary — new)

Replace the current bottom-of-panel power summary with a dedicated right panel that updates reactively as mercs are toggled.

**Sections:**

1. **Cost and balance header:**
   ```
   Cost: 20 Marks     Balance: 240
   ```
   Color cost red if insufficient, green if sufficient.

2. **Squad power meter:**
   ```
   Squad Power:  68 / 100
   ████████████████░░░░░░░░░░░░░░░   68%
   ```
   Bar color matches forecast: Green (good), Yellow (risky), Red (fail).

3. **Success forecast (prominent):**
   ```
   Risky — ~30% success
   ```
   Use large labels. For safe missions: `Always succeeds — no power check`.

4. **Archetype summary:**
   List archetypes present in staged squad. Highlight required/recommended.
   ```
   Archetypes in squad:
     Scout (Lyra)
   (!) Arcanist required — not present
   ★  Scout recommended — present
   ```
   - `(!)` prefix: `Color::Yellow`
   - `★` prefix: `Color::Cyan`

5. **Smart hint line** (contextual tip):
   - If required archetype missing and is in roster: `Add [Name] ([Arch]) to meet requirement.`
   - If squad empty: `Select mercs with [Space]`
   - If squad overpowered: `Overpowered — mission will be faster!`
   - If insufficient marks: `Not enough Marks — earn more via Supply Runs`

6. **Launch action hint:**
   ```
   [Enter] Launch Mission
   ```
   `Color::Rgb(60, 180, 80)` when squad is non-empty, `Color::DarkGray` when empty.

### 2.5 Success Probability Display Improvements

Current labels are inconsistent across compact and split views. Standardize to:

| Power Ratio | Label | Color |
|---|---|---|
| Squad empty | `Select mercs with [Space]` | `DarkGray` |
| Safe mission type | `Always succeeds` | `Green` |
| `>= 150%` of min | `Overpowered — 95% + faster` | `Rgb(80, 220, 120)` |
| `>= 100%` of min | `Good — 60-90% success` | `Green` |
| `>= 75%` of min | `Risky — ~30% success` | `Yellow` |
| `< 75%` of min | `Likely to fail` | `LightRed` |

The label should always be on its own line, never appended as a suffix.

### 2.6 Specific Code Changes — Mission Assignment

**File:** `src/ui/deep_missions.rs`

#### Change 1: Add `render_mission_detail_phase1()` helper

Extract the right panel of `render_new_mission_split()` into a dedicated function that renders the full Phase 1 detail:

```rust
fn render_mission_detail_phase1(
    buffer: &mut [Vec<SceneCell>],
    deep: &DeepState,
    ui: &DeepUiState,
    mission: &AvailableMission,
    detail_inner_left: i32,
    detail_inner_w: i32,
    content_top: i32,
    content_bottom: i32,
) {
    let mut row = content_top;

    // Layer + tier heading
    let tier_name = crate::deep::LayerTier::from_layer(mission.layer).display_name();
    put_text(buffer, row, detail_inner_left, &format!("Layer {} \u{2014} {}", mission.layer, tier_name), Color::White);
    row += 1;

    // Mission type name (colored)
    let tc = mission_type_color(mission.mission_type);
    put_text(buffer, row, detail_inner_left, mission.mission_type.display_name(), tc);
    row += 1;

    // Description (word-wrapped, 2 lines max)
    if !mission.description.is_empty() {
        let max_w = (detail_inner_w - 1).max(10) as usize;
        let words: Vec<&str> = mission.description.split_whitespace().collect();
        let mut line_buf = String::new();
        let mut lines_rendered = 0;
        for word in &words {
            if lines_rendered >= 2 { break; }
            if line_buf.len() + word.len() + 1 > max_w && !line_buf.is_empty() {
                put_text(buffer, row, detail_inner_left, &line_buf, Color::DarkGray);
                row += 1;
                lines_rendered += 1;
                line_buf.clear();
            }
            if !line_buf.is_empty() { line_buf.push(' '); }
            line_buf.push_str(word);
        }
        if !line_buf.is_empty() && lines_rendered < 2 {
            put_text(buffer, row, detail_inner_left, &line_buf, Color::DarkGray);
            row += 1;
        }
    }
    row += 1;

    // Duration — show effective if modifiers apply
    let layer_record = deep.persistent.layer_record(mission.layer);
    let duration_reduction = layer_record.map(|l| l.total_duration_reduction()).unwrap_or(0.0);
    let effective_secs = (mission.duration_secs as f64 * (1.0 - duration_reduction)) as u64;
    let dur_str = if duration_reduction > 0.01 {
        format!("Duration:  {}   (\u{2192} {} effective)", format_hours(mission.duration_secs), format_hours(effective_secs))
    } else {
        format!("Duration:  {}", format_hours(mission.duration_secs))
    };
    put_text(buffer, row, detail_inner_left, &dur_str, Color::DarkGray);
    row += 1;

    // Risk
    let risk_str = format!("Risk:      {}", risk_label(mission.mission_type.risk_tier()));
    put_text(buffer, row, detail_inner_left, &risk_str, risk_color(mission.mission_type.risk_tier()));
    row += 1;

    // Cost with affordability
    let marks = deep.prestige.warband_marks;
    if mission.marks_cost > 0 {
        let (afford_str, afford_color) = if marks >= mission.marks_cost {
            (format!("   (have {})", marks), Color::Rgb(60, 180, 80))
        } else {
            (format!("   (have {} \u{2014} INSUFFICIENT)", marks), Color::LightRed)
        };
        put_text(buffer, row, detail_inner_left, &format!("Cost:      {} Marks", mission.marks_cost), Color::Yellow);
        put_text(buffer, row, detail_inner_left + format!("Cost:      {} Marks", mission.marks_cost).len() as i32, &afford_str, afford_color);
    } else {
        put_text(buffer, row, detail_inner_left, "Cost:      Free", Color::Rgb(60, 180, 80));
    }
    row += 1;

    put_text(buffer, row, detail_inner_left, "Reward:    Marks + XP + items", Color::DarkGray);
    row += 2;

    // Requirements section
    put_text(buffer, row, detail_inner_left, "Requires:", Color::Cyan);
    row += 1;

    // Power (always shown)
    put_text(buffer, row, detail_inner_left, &format!("  Min Power  {}", mission.min_squad_power), Color::White);
    row += 1;

    // Required archetype — check against full roster
    if let Some(req_arch) = mission.required_archetype {
        let in_roster = deep.prestige.roster.iter().any(|m| m.archetype == req_arch);
        let (prefix, label_color) = if in_roster {
            ("\u{2713} ", Color::Green)
        } else {
            ("\u{26a0} ", Color::Yellow)
        };
        let suffix = if !in_roster { " (not in roster!)" } else { " (required)" };
        put_text(
            buffer, row, detail_inner_left,
            &format!("  {}{}{}", prefix, req_arch.display_name(), suffix),
            label_color,
        );
        row += 1;
    }

    // Recommended archetype
    if let Some(rec_arch) = mission.recommended_archetype {
        let in_roster = deep.prestige.roster.iter().any(|m| m.archetype == rec_arch);
        let (prefix, color) = if in_roster { ("\u{2605} ", Color::Cyan) } else { ("  ", Color::DarkGray) };
        put_text(
            buffer, row, detail_inner_left,
            &format!("  {}{} recommended", prefix, rec_arch.display_name()),
            color,
        );
        row += 1;
    }
    row += 1;

    // Action hint at bottom
    let hint_row = (content_bottom - 2).max(row);
    put_text(buffer, hint_row, detail_inner_left, "[Enter] Assign squad \u{2192}", Color::Rgb(50, 120, 60));
}
```

#### Change 2: Add `render_squad_summary_panel()` helper

New right panel for Phase 2:

```rust
fn render_squad_summary_panel(
    buffer: &mut [Vec<SceneCell>],
    deep: &DeepState,
    ui: &DeepUiState,
    mission: &AvailableMission,
    detail_inner_left: i32,
    detail_inner_w: i32,
    content_top: i32,
    content_bottom: i32,
) {
    let squad_power: u32 = ui.staged_squad.iter()
        .filter_map(|id| deep.prestige.find_merc(*id))
        .map(|m| m.effective_power())
        .sum();
    let min = mission.min_squad_power;
    let marks = deep.prestige.warband_marks;
    let is_safe = matches!(mission.mission_type, MissionType::SupplyRun | MissionType::Construction(_));
    let can_afford = marks >= mission.marks_cost;

    let mut row = content_top;

    // Cost + balance
    if mission.marks_cost > 0 {
        let cost_color = if can_afford { Color::Green } else { Color::LightRed };
        put_text(buffer, row, detail_inner_left,
            &format!("Cost: {} Marks     Balance: {}", mission.marks_cost, marks),
            Color::White);
        // Recolor cost portion
        put_text(buffer, row, detail_inner_left + 6, &format!("{}", mission.marks_cost), cost_color);
    }
    row += 2;

    // Power meter
    let power_str = format!("Squad Power:  {} / {}", squad_power, min);
    let ratio = if min == 0 { 1.0 } else { squad_power as f64 / min as f64 };
    let (bar_color, forecast_label, forecast_color) = if ui.staged_squad.is_empty() {
        (Color::DarkGray, "Select mercs with [Space]", Color::DarkGray)
    } else if is_safe {
        (Color::Green, "Always succeeds", Color::Green)
    } else if ratio >= 1.5 {
        (Color::Rgb(80, 220, 120), "Overpowered \u{2014} 95% + faster!", Color::Rgb(80, 220, 120))
    } else if ratio >= 1.0 {
        (Color::Green, "Good \u{2014} 60-90% success", Color::Green)
    } else if ratio >= 0.75 {
        (Color::Yellow, "Risky \u{2014} ~30% success", Color::Yellow)
    } else {
        (Color::LightRed, "Likely to fail", Color::LightRed)
    };

    put_text(buffer, row, detail_inner_left, &power_str, Color::White);
    row += 1;
    let bar_w = (detail_inner_w as usize).saturating_sub(2).min(24);
    render_progress_bar(buffer, row, detail_inner_left, bar_w, ratio.min(1.0), bar_color);
    row += 1;
    put_text(buffer, row, detail_inner_left, forecast_label, forecast_color);
    row += 2;

    // Archetype summary
    put_text(buffer, row, detail_inner_left, "Archetypes in squad:", Color::Cyan);
    row += 1;

    let squad_archetypes: Vec<crate::deep::MercArchetype> = ui.staged_squad.iter()
        .filter_map(|id| deep.prestige.find_merc(*id))
        .map(|m| m.archetype)
        .collect();

    if squad_archetypes.is_empty() {
        put_text(buffer, row, detail_inner_left, "  (none selected)", Color::DarkGray);
        row += 1;
    } else {
        // Deduplicate and list present archetypes
        let mut seen = std::collections::HashSet::new();
        for &arch in &squad_archetypes {
            if seen.insert(arch) {
                if row >= content_bottom - 3 { break; }
                let name = deep.prestige.roster.iter()
                    .find(|m| m.archetype == arch && ui.staged_squad.contains(&m.id))
                    .map(|m| m.name.as_str())
                    .unwrap_or("");
                put_text(buffer, row, detail_inner_left,
                    &format!("  {} ({})", arch.display_name(), name),
                    archetype_color(arch));
                row += 1;
            }
        }
    }

    // Required archetype check
    if let Some(req_arch) = mission.required_archetype {
        let req_present = squad_archetypes.contains(&req_arch);
        let (prefix, color, suffix) = if req_present {
            ("\u{2713} ", Color::Green, " required \u{2014} present")
        } else {
            ("(!) ", Color::Yellow, " required \u{2014} missing!")
        };
        if row < content_bottom - 3 {
            put_text(buffer, row, detail_inner_left,
                &format!("{}{}{}", prefix, req_arch.display_name(), suffix),
                color);
            row += 1;
        }
    }

    // Recommended archetype check
    if let Some(rec_arch) = mission.recommended_archetype {
        let rec_present = squad_archetypes.contains(&rec_arch);
        let (prefix, color, suffix) = if rec_present {
            ("\u{2605} ", Color::Cyan, " recommended \u{2014} present")
        } else {
            ("  ", Color::DarkGray, " recommended")
        };
        if row < content_bottom - 3 {
            put_text(buffer, row, detail_inner_left,
                &format!("{}{}{}", prefix, rec_arch.display_name(), suffix),
                color);
            row += 1;
        }
    }

    // Smart contextual hint
    row += 1;
    let hint = if !can_afford && mission.marks_cost > 0 {
        Some(("Earn Marks via Supply Runs (free)", Color::DarkGray))
    } else if ui.staged_squad.is_empty() {
        Some(("Select mercs with [Space]", Color::DarkGray))
    } else if ratio >= 1.5 {
        Some(("Overpowered \u{2014} mission will complete faster!", Color::Rgb(80, 220, 120)))
    } else if let Some(req_arch) = mission.required_archetype {
        let req_present = squad_archetypes.contains(&req_arch);
        if !req_present {
            // Find the archetype in roster
            let merc_with_arch = deep.prestige.roster.iter()
                .find(|m| m.archetype == req_arch && m.is_available());
            if let Some(m) = merc_with_arch {
                Some((format!("Add {} ({}) to meet requirement", m.name, req_arch.display_name()).into(), Color::Yellow))
            } else {
                Some(("Recruit a {} — check [Recruit] tab".to_string().into(), Color::Yellow))
            }
        } else { None }
    } else { None };

    if let Some((hint_text, hint_color)) = hint {
        let hint_row = (content_bottom - 3).max(row);
        if hint_row < content_bottom {
            put_text(buffer, hint_row, detail_inner_left, &hint_text.to_string(), hint_color);
        }
    }

    // Launch action at bottom
    let launch_row = content_bottom - 1;
    let (launch_color, launch_label) = if ui.staged_squad.is_empty() {
        (Color::DarkGray, "[Enter] Launch Mission")
    } else {
        (Color::Rgb(60, 180, 80), "[Enter] Launch Mission")
    };
    put_text(buffer, launch_row, detail_inner_left, launch_label, launch_color);
}
```

#### Change 3: Update `render_new_mission_split()` to use phase-specific panels

```rust
// In render_new_mission_split(), replace the detail rendering block:
let staging = ui.staging_mission_index.is_some();
let detail_idx = ui.staging_mission_index.unwrap_or(ui.selected_index);
let Some(m) = available.get(detail_idx) else { return; };

if !staging {
    render_mission_detail_phase1(buffer, deep, ui, m, detail_inner_left, detail_inner_w, content_top, content_bottom);
} else {
    render_squad_summary_panel(buffer, deep, ui, m, detail_inner_left, detail_inner_w, content_top, content_bottom);
    // Also update left panel heading
    put_text(buffer, content_top, detail_inner_left - list_width as i32 + 1, "ASSIGN SQUAD", SECTION_LABEL_COLOR);
}
```

#### Change 4: Left panel merc list with group separator (Phase 2)

Replace the current single-pass merc list in Phase 2 with grouped rendering:

```rust
// In render_new_mission_split() merc list section (during squad staging):
let available_roster: Vec<_> = deep.prestige.roster.iter().enumerate()
    .filter(|(_, m)| m.is_available()).collect();
let unavailable_roster: Vec<_> = deep.prestige.roster.iter().enumerate()
    .filter(|(_, m)| !m.is_available()).collect();

// Render available mercs
let mut row = squad_label_row + 1;
for (ri, merc) in &available_roster {
    if row >= content_bottom - 2 { break; }
    let is_sel = *ri == ui.selected_index;
    // ... render with cursor and checkbox ...
    row += 1;
}

// Group separator
if !unavailable_roster.is_empty() && row < content_bottom - 2 {
    let sep = "\u{2500} \u{2500} \u{2500}".repeat(list_width / 6);
    put_text(buffer, row, 1, &sep[..list_width.min(sep.len())], Color::Rgb(40, 60, 80));
    row += 1;
    // Render unavailable (not selectable, cursor does not stop here)
    for (_, merc) in &unavailable_roster {
        if row >= content_bottom { break; }
        let avail_str = match &merc.status {
            MercStatus::OnMission(_) => "on mission".to_string(),
            MercStatus::Injured { missions_remaining } => format!("injured: {}", missions_remaining),
            MercStatus::Lost => "lost".to_string(),
            _ => String::new(),
        };
        put_text(buffer, row, 3,
            &format!("  {:14} {:8} ({})", &merc.name[..merc.name.len().min(14)], merc.archetype.display_name(), avail_str),
            Color::Rgb(50, 60, 70));
        row += 1;
    }
}
```

#### Change 5: Footer — show Marks balance

Add Marks balance to footer in both Phase 1 and Phase 2:

```rust
// At start of render_new_mission(), compute marks display
let marks_display = format!("\u{25c6} {} M", deep.prestige.warband_marks);
// Render in footer right-aligned
let footer_col = (width as i32 - marks_display.len() as i32 - 2).max(1);
put_text(buffer, height as i32 - 1, footer_col, &marks_display, Color::Yellow);
```

#### Change 6: Input — skip unavailable mercs during squad assignment

In `handle_squad_assignment()` (`deep_input.rs:157-281`), filter mercs so cursor only lands on available ones:

```rust
// Replace available_mercs computation to use only available mercs for navigation
let available_mercs: Vec<(usize, u64)> = deep_state.prestige.roster.iter()
    .enumerate()
    .filter(|(_, m)| m.is_available())
    .map(|(i, m)| (i, m.id))
    .collect();
// Navigation and Space toggle operate on this filtered list
// selected_index refers to position within available_mercs, not full roster
```

#### Change 7: Construction mission type label (P0.3 from audit)

**File:** `src/ui/deep_missions.rs` — compact list line 358, split list line 468

In both compact and split mission list renderers, replace:
```rust
let type_name = m.mission_type.display_name();
```
with:
```rust
let type_label: String = match m.mission_type {
    MissionType::Construction(infra) => format!("Build {}", infra.display_name()),
    other => other.display_name().to_string(),
};
// Use type_label wherever type_name was previously used in the format! call
```

This makes `"Build Outpost"`, `"Build Watchtower"` etc. appear in the list, letting players immediately know what they are queueing without opening the detail panel.

#### Change 8: Empty mission pool message (audit bug 3.3)

**File:** `src/ui/deep_missions.rs` — `render_new_mission()` lines 303-318

Replace the misleading "Complete active missions to refresh the pool" with accurate guidance:

```rust
if available.is_empty() {
    let mid = content_top + content_height as i32 / 2;
    put_text_centered(buffer, mid - 1, width, "No missions available.", Color::DarkGray);

    // Accurate guidance based on actual state
    let active_count = deep.prestige.active_mission_count();
    if active_count == 0 && deep.prestige.roster.is_empty() {
        put_text_centered(buffer, mid, width, "Recruit mercenaries in [Recruit] tab first.", Color::Rgb(50, 70, 100));
    } else if active_count > 0 {
        put_text_centered(buffer, mid, width, "Mission pool refreshes over time.", Color::Rgb(50, 70, 100));
        put_text_centered(buffer, mid + 1, width, "Check back after your current missions complete.", Color::Rgb(40, 55, 80));
    } else {
        put_text_centered(buffer, mid, width, "Mission pool refreshes periodically.", Color::Rgb(50, 70, 100));
        put_text_centered(buffer, mid + 1, width, "Return in a few minutes.", Color::Rgb(40, 55, 80));
    }
    return;
}
```

#### Change 9: Flash message positioning fix (audit bug 3.7)

**File:** `src/ui/deep_missions.rs` — `render_new_mission()` lines 292-299

Reserve one extra row so flash messages are never overwritten by content:

```rust
// Flash row is height - 2; content must stop at height - 3
if let Some(msg) = &ui.flash_message {
    put_text(buffer, height as i32 - 2, 1, msg, Color::LightRed);
}
put_text(buffer, height as i32 - 1, 1, footer, Color::DarkGray);

let content_top = 0i32;
let content_bottom = height as i32 - 3; // was height - 2; now leaves room for flash + footer
```

#### Change 10: Power ratio percentage display (P1.4 from audit)

In `render_squad_summary_panel()` (new helper) and in the compact power summary row, show the power percentage inline:

```rust
let ratio_pct = if min == 0 { 999u32 } else { squad_power * 100 / min };
let power_str = if min == 0 || is_safe {
    format!("Squad Power:  {}", squad_power)
} else {
    format!("Squad Power:  {} / {}  ({}%)", squad_power, min, ratio_pct)
};
put_text(buffer, row, detail_inner_left, &power_str, Color::White);
// Recolor the percentage based on success band
let pct_color = if ratio_pct >= 150 { Color::Rgb(80, 220, 120) }
    else if ratio_pct >= 100 { Color::Green }
    else if ratio_pct >= 75 { Color::Yellow }
    else { Color::LightRed };
// Find position of "(N%)" and recolor that segment
let pct_str = format!("({}%)", ratio_pct);
if let Some(pos) = power_str.find('(') {
    put_text(buffer, row, detail_inner_left + pos as i32, &pct_str, pct_color);
}
```

---

## Part 3: Compact Mode (S-tier) Adaptations

For S-tier (width < 60 or height tier S), the designs simplify to single-column:

### Hub compact

```
Rank 1 Freelancers  3/5  0/1 concurrent  ◆240M
Frontier: L3 Warrens
─────────────────────────────────────────
MISSIONS
▶ [Expedition] L3  65%  ⚡ Event!
  Gareth, Lyra  ████████░░  2h 10m left
  [Supply Run] L1  43%
  Aldric  █████░░░░░  3h 45m left
─────────────────────────────────────────
[Tab]Switch [Enter]Select [Esc]Close
```

### Mission compact (Phase 1)

Single column mission list with description on selection:

```
▶ [Expedition] L3 10h Medium 20M
  [Supply Run] L1 2h  Safe  Free
  [Recon]      L3 6h  Low   5M

Selected: Expedition — primary progression mission.
Min Power: 100   ⚠ Arcanist required (missing!)
Cost: 20M  (have 240)  [Enter] Assign squad
```

### Mission compact (Phase 2)

```
[Expedition] Layer 3  10h  20M

[✓] Gareth   Vanguard  L3
[ ] Lyra     Scout     L2
[ ] Theron   Arcanist  L4
─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─
    Vex      (on mission)
    Aldric   (injured: 2)

Power: 20/100  Risky 30%
(!) Arcanist required — add Theron
[Space] Toggle  [Enter] Launch  [Esc] Cancel
```

---

## Part 4: Summary of All File Changes

### `src/ui/deep_missions.rs`

| Change | Function | Lines | Description |
|--------|----------|-------|-------------|
| Guild status block | `render_hub()` | 117-137 | 4-row structured block with marks, capacity, next-rank hint |
| Mission card order | `render_hub()` | 175-251 | Reorder to: type+status → squad → bar+time |
| Empty state copy (Hub) | `render_hub()` | 147-151 | 4-line actionable empty state |
| `SECTION_LABEL_COLOR` constant | module level | — | `Color::Rgb(80, 160, 220)` |
| Footer Marks display | `render_new_mission()` | 291-295 | Right-aligned `◆ N M` in footer |
| Phase 1 detail panel | New fn `render_mission_detail_phase1()` | — | Full detail with description, affordability, archetype warning |
| Phase 2 squad summary | New fn `render_squad_summary_panel()` | — | Reactive squad summary with power meter and archetype checks |
| Phase dispatch in split | `render_new_mission_split()` | 489-597 | Route to phase1 or phase2 helper |
| Left panel grouped mercs | `render_new_mission_split()` | 538-562 | Available/unavailable groups with separator |
| Compact Phase 1 detail | `render_new_mission_compact()` | 342-426 | Show description + affordability under selected mission |
| Construction label fix (P0.3) | `render_new_mission_compact()`, `render_new_mission_split()` | 358, 468 | Use `"Build {infra}"` instead of `"Construction"` |
| Power ratio percentage (P1.4) | `render_squad_summary_panel()`, compact power line | new fn | Show `(128%)` after `Power: N/M` |
| Empty pool message fix (bug 3.3) | `render_new_mission()` | 303-318 | Replace misleading "complete missions" hint with accurate guidance |
| Flash message positioning fix (bug 3.7) | `render_new_mission()` | 292-295 | Reserve flash row at `height - 2`, set `content_bottom = height - 3` |

### `src/ui/deep_scene.rs`

| Change | Function | Lines | Description |
|--------|----------|-------|-------------|
| Tab bar state badges (P1.3) | `render_tab_bar()` | 119-138 | Add ⚡/✓/·/! badges to tab labels; add `deep: &DeepState` parameter |

### `src/input/deep_input.rs`

| Change | Function | Lines | Description |
|--------|----------|-------|-------------|
| Available-only cursor | `handle_squad_assignment()` | 162-168 | Filter roster to only available mercs for navigation |
| Warn on missing arch | `handle_squad_assignment()` | Enter branch | Flash message if required archetype missing on launch |

### No changes needed to:
- `deep_events.rs` — Event view is solid (bug 3.5 auto-resolve constant is low priority)
- `deep_roster.rs` — Addressed in Task #4
- `deep_layers.rs` — Addressed in Task #5
- `types.rs` — All required data structures already exist

---

## Part 5: Design Rationale

### Why group-separated merc list during squad staging?

Players making squad decisions need to answer two questions: "Who is available?" and "Who can I add?" The current flat list mixes available and unavailable mercs. Grouping eliminates scanning overhead and reduces the chance of accidentally fixating on mercs that cannot be selected.

### Why co-locate Marks balance with mission cost?

Affordability is a binary go/no-go decision made immediately when viewing a mission. Requiring players to navigate to Hub → check balance → navigate back → assess mission breaks flow. The pattern of showing balance inline (similar to how shops show "You have X gold") is universal in RPG UIs.

### Why show required archetype in mission list vs. buried in details?

Required archetypes are hard constraints, not preferences. A player without a Saboteur who is browsing missions should immediately see which missions are gated. The `⚠` prefix on the list line (for missing required archetypes) lets players filter by eye without reading each detail panel.

### Why reorder Hub mission cards (bar before squad)?

Progress bar + time remaining answers "when does this come back?" — the most frequent scan question for idle players checking in. Squad identity answers "who is deployed?" — important context but not the first question. The reordering matches the mental model: players glance at The Deep to see progress, not to remember roster assignments.

### Why a Phase 1 / Phase 2 split concept?

The current `staging_mission_index` approach already encodes two phases but renders them ambiguously. Making the phase explicit in rendering (different right panel content, different heading) reduces the cognitive overhead of understanding "am I selecting a mission or selecting a squad right now?"

### Why tab bar state badges?

The Deep is an asynchronous system — events fire while players are away. The stats panel `[D]` indicator tells players something needs attention, but once inside the overlay the tab bar gives no guidance on *where* to go. A player with a pending event and a completed mission who opens The Deep should immediately see `[Hub⚡1✓1]` and navigate directly, not discover the state by cycling tabs.

### Why fix the "no missions available" empty state message?

"Complete active missions to refresh the pool" implies a causal relationship that doesn't exist. The mission pool refreshes on a timer, not on mission completion. Giving players false mental models leads to confusion when the pool stays empty after missions complete. The fix branches on actual state (no mercs, missions active, or just waiting) to give accurate, actionable guidance.

### Why show power as a percentage?

The success bands are defined by power ratios: `<75%` = fail, `75-99%` = risky (30%), `100-149%` = good (60-90%), `150%+` = overpowered (95%). Showing `Power: 32/25` requires players to compute `32/25 = 128%` mentally to know they are in the "good" band. Showing `(128%)` colored green eliminates the math and makes the band membership immediately scannable.

---

## Appendix: Color Palette Reference for The Deep UI

| Element | Color | Hex approx |
|---------|-------|-----------|
| Section labels | `Rgb(80, 160, 220)` | Deep blue |
| Marks/currency | `Yellow` | Bright yellow |
| Marks ◆ icon | `Yellow` | — |
| Good/affordable | `Rgb(60, 180, 80)` | Soft green |
| Warning/missing | `Yellow` | — |
| Error/insufficient | `LightRed` | — |
| Overpowered | `Rgb(80, 220, 120)` | Bright green |
| Risky | `Yellow` | — |
| Fail | `LightRed` | — |
| Background separators | `Rgb(40, 60, 80)` | Dark slate |
| Inactive/unavailable | `DarkGray` | — |
| Required archetype present | `Green` | — |
| Recommended archetype present | `Cyan` | — |
| Event pending badge | `Yellow` | — |
| Completed badge | `Green` | — |
