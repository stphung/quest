# The Deep UI: Roster, Layers, and Recruit View — Design Specification

**Date:** 2026-02-23 (updated post-audit)
**Scope:** Tasks #4 and #5 — Roster stat clarity, Layer map visual improvements
**Also covers:** Recruit view (P0.1 critical bug), tab bar state indicators (P1.3)
**Author:** UX Designer (agent)
**Based on audit:** `docs/plans/deep-ui-audit.md`

---

## Executive Summary

The audit confirmed and extended the problems identified in the initial draft. Critical additions:

1. **The Recruit tab renders nothing** (dispatches to Roster instead) — P0.1 critical
2. **Construction missions drop their payload** — shows "Construction" not "Build Outpost" — P0.3
3. **Familiarity tier labels never shown** — raw % only — P1.1
4. **Tab bar carries no state indicators** — injuries, events, and completions require manual tab-switching to discover — P1.3
5. **Infrastructure costs absent** — players cannot plan builds — P1.5
6. **Build action missing from Layers view** — read-only when it should be actionable — P2.4
7. **Tier color suppressed in compact mode** — `let _ = tc` bug — P2.5
8. **Status offset fragile in compact Roster** — `rfind` pattern breaks on name matches — P3.4

This document covers all of these issues in implementation-ready detail.

---

## Part 1: Roster View — Stat Clarity and Merc Progression

### 1.1 Current Problems (Confirmed by Audit)

**Stats have no semantic meaning:**
- `Power: 14`, `Resilience: 12`, `Expertise: 8` — numbers with no context
- No indication that Power = combat effectiveness, Resilience = injury resistance, Expertise = event bonuses

**Status display is opaque:**
- `"Injured (2 missions)"` — measured in missions; players think in hours
- `"On mission #7"` — mission ID is meaningless
- No urgency gradient between Light/Moderate/Severe injuries

**Archetype identity is invisible:**
- Color only; role description never shown
- No indication which mission types the archetype benefits

**Progression is hidden:**
- Level shown, missions-to-next-level never shown
- No progress bar toward next level
- Audit correction: `missions_to_next_level(level) = 3 + level * 2`, cumulative from level 1

**Quality tier is not stored on Mercenary:**
- Audit confirmed: `quality` is not on the `Mercenary` struct; infer from stat delta vs archetype baseline, or add as a field (recommended)

**Status column offset is fragile (P3.4):**
- `line.rfind(status_label)` will misidentify position if merc name contains "Ready", "Injured", etc.
- Must use fixed column offsets derived from format string field widths

### 1.2 Redesigned Roster List — Compact Mode

**Header:**
```
  Name              Role  Lv  Pwr  Res  Status
```

**Row format** — fixed column offsets, no rfind:
```
▶ Gareth Ironwall  [VAN]   8   47   38   Ready
  Lyra Shadowfoot  [SCT]   5   28   22   ● Mission
  Aldric Mender    [MED]   3   18   20   ✖ 1 miss
  Finn the Cunning [SAB]   2   15   12   ⚕ Injured
```

Fixed column positions (to avoid fragile rfind):
- Col 1-2: cursor `▶ ` or `  `
- Col 3-16: name, max 14 chars
- Col 18-22: archetype abbreviation `[VAN]` in archetype color
- Col 24-25: level, right-aligned 2 chars
- Col 27-29: effective_power, right-aligned 3 chars
- Col 31-33: effective_resilience, right-aligned 3 chars
- Col 36+: status with glyph prefix

Status display format (use fixed status column, not rfind):
- `Ready` → `Color::Green`
- `● Mission` → `Color::Cyan` (U+25CF filled circle)
- `⚕ N miss` → `Color::Yellow` (N = missions_remaining)
- `✖ Lost` → `Color::Red` (U+2716 heavy X)

Archetype abbreviations and colors:
- `[VAN]` Vanguard → `Color::Red`
- `[SCT]` Scout → `Color::Green`
- `[ARC]` Arcanist → `Color::Magenta`
- `[MED]` Medic → `Color::Cyan`
- `[SAB]` Saboteur → `Color::Yellow`

Remove the blank-row gap between mercs — the status glyph (`●`/`⚕`/`✖`) provides visual rhythm. This allows more mercs on screen, which matters when guild rank cap reaches 15.

### 1.3 Redesigned Roster List — Split Mode (Left Panel)

**Header:**
```
  Name              Archetype   Lv  Pwr  Res   Status
```

**Row format** (single row per merc):
```
▶ Gareth Ironwall  Vanguard      8   47   38   Ready
  Lyra Shadowfoot  Scout         5   28   22   ● Active
  Aldric Mender    Medic         3   18   20   ⚕ 2 miss
  Finn the Cunning Saboteur      2   15   12   ✖ Lost
  [ Recruit slot — 60 Marks ]
```

Recruit slot hint shown when roster is below guild rank cap.

### 1.4 Redesigned Roster Detail Panel (Right Panel)

**Available merc example:**
```
Gareth Ironwall
Vanguard  ·  Level 8  ·  Missions: 14

Role: Frontline tank
  High Power + Resilience. Reduces squad
  casualties. Best on high-risk missions.

Stats
  Power:       47  (combat effectiveness)
  Resilience:  38  (reduces injury chance)
  Expertise:   10  (event bonuses and unlocks)

Progression
  Missions completed: 14
  To Level 9:  ██████░░░  6 / 9 missions
  At Lv9:  Pwr 50  Res 40  Exp 11

Status
  Ready for assignment
```

**Injured merc example:**
```
Aldric the Mender
Medic  ·  Level 3  ·  Missions: 4

Role: Squad healer
  Highest Resilience. Prevents permanent
  loss and reduces injury severity.

Stats
  Power:       18  (combat effectiveness)
  Resilience:  20  (reduces injury chance)
  Expertise:   16  (event bonuses and unlocks)

Progression
  Missions completed: 4
  To Level 4:  ████░░░░░  4 / 9 missions
  At Lv4:  Pwr 20  Res 22  Exp 18

Status
  Injured — Moderate
  Recovery: ~2 missions remaining (~12h)
  ████████░░░░  Returns after 2 missions
```

**On-mission merc example:**
```
Lyra Shadowfoot
Scout  ·  Level 5  ·  Missions: 9

Role: Recon specialist
  High Expertise. Better auto-resolve and
  early event reveals. Faster missions.

Stats
  Power:       28  (combat effectiveness)
  Resilience:  22  (reduces injury chance)
  Expertise:   22  (event bonuses and unlocks)

Progression
  Missions completed: 9
  To Level 6:  █████░░░░  5 / 11 missions
  At Lv6:  Pwr 31  Res 25  Exp 26

Status
  On Mission — Layer 2 Recon
  ██████████░░░░  68% — returns in ~3h 20m
  ETA: 14:32
```

### 1.5 Progress Bar Calculation

The XP formula from `mercenaries.rs`:
- `missions_to_next_level(level) = 3 + level * 2`
- Cumulative missions to reach level N = `sum(3 + i*2 for i in 1..N)`

Since `Mercenary` only stores `missions_completed` (total all-time), we compute:

```rust
fn missions_to_reach_level(target_level: u32) -> u32 {
    (1..target_level).map(|l| Mercenary::missions_to_next_level(l)).sum()
}

fn level_progress(merc: &Mercenary) -> (u32, u32) {
    let missions_at_current_level = missions_to_reach_level(merc.level);
    let missions_for_this_level = Mercenary::missions_to_next_level(merc.level);
    let progress = merc.missions_completed.saturating_sub(missions_at_current_level);
    (progress.min(missions_for_this_level), missions_for_this_level)
}
```

Bar format: `██████░░░  6 / 9 missions`
- Filled: `Color::Cyan` `█`
- Empty: `Color::Rgb(30, 40, 60)` `░`
- Bar width: `detail_inner_w.saturating_sub(16).min(20)`

### 1.6 Injury Display

Severity display based on `missions_remaining`:
- 1 mission: `Color::Yellow` — "Light injury"
- 2 missions: `Color::LightRed` — "Moderate injury"
- 3+ missions: `Color::Red` — "Severe injury"

Hour estimate: `missions_remaining * 6` (from `HOURS_PER_MISSION_EQUIVALENT = 6` in `mercenaries.rs`)

### 1.7 Level-Up Stat Preview

At Lv N+1, stats calculated with `stats_at_level()` from `mercenaries.rs`. Show the delta from current effective stats:

```
At Lv9:  Pwr +3→50  Res +2→40  Exp +2→12
```

Or simplified:
```
At Lv9:  Pwr 50  Res 40  Exp 12
```

Use `Color::DarkGray` for preview values (not yet earned).

### 1.8 Archetype Role Descriptions

```rust
fn archetype_role(archetype: MercArchetype) -> (&'static str, &'static str) {
    match archetype {
        MercArchetype::Vanguard  => (
            "Frontline tank",
            "High Power + Resilience. Reduces squad\ncasualties. Best on high-risk missions."
        ),
        MercArchetype::Scout     => (
            "Recon specialist",
            "High Expertise. Better auto-resolve and\nearly event reveals. Faster missions."
        ),
        MercArchetype::Arcanist  => (
            "Elemental expert",
            "Highest Expertise. Counters hazards and\nenvironmental dangers. Fragile."
        ),
        MercArchetype::Medic     => (
            "Squad healer",
            "Highest Resilience. Prevents permanent\nloss and reduces injury severity."
        ),
        MercArchetype::Saboteur  => (
            "Trap specialist",
            "High Expertise. Speeds missions and\nunlocks alternate routes."
        ),
    }
}
```

### 1.9 Fixed Status Column Implementation

Replace fragile `rfind` with a calculated fixed column:

```rust
// In render_roster_compact():
// Format string: "{cursor}{name:14} {abbrev:5} {lv:2} {pwr:3} {res:3}   {status}"
// cursor=2, name=14, space=1, abbrev=5, space=1, lv=2, space=2, pwr=3, space=2, res=3, gap=3
const STATUS_COL: i32 = 2 + 14 + 1 + 5 + 1 + 2 + 2 + 3 + 2 + 3 + 3; // = 40
put_text(buffer, row, STATUS_COL, status_glyph_and_label, status_color);
```

---

## Part 2: Recruit View — New Implementation (P0.1 Critical)

### 2.1 The Bug

`deep_scene.rs:196` dispatches `DeepView::Recruit` to `render_roster()`. The Recruit tab silently shows the Roster. This must be fixed.

### 2.2 New `render_recruit()` Function

Add to `src/ui/deep_roster.rs` (alongside existing Roster functions):

**ASCII Mockup — Compact mode:**
```
RECRUIT POOL        Roster: 3/5    Marks: 240

  Name                Role   Lv  Pwr  Res  Exp  Cost
  ─────────────────────────────────────────────────────
  Bram Ironwall       [VAN]   1   15   13    5    50M
► Kira Shadowfoot     [SCT]   1    9   11   13    35M
  Njord the Mender    [MED]   1    7   15   11    40M

  Pool refreshes in: 14h 32m
  [ Select with ↑/↓, recruit with Enter ]
```

**ASCII Mockup — Split mode (left panel — candidate list):**
```
RECRUIT POOL             Marks: 240 M

  Name              Archetype  Pwr  Res  Exp  Cost
  ─────────────────────────────────────────────────
  Bram Ironwall     Vanguard    15   13    5   50M
► Kira Shadowfoot   Scout        9   11   13   35M
  Njord the Mender  Medic        7   15   11   40M

  Pool refresh: 14h 32m
```

**ASCII Mockup — Split mode (right panel — candidate detail):**
```
Kira Shadowfoot
Scout  ·  Level 1  ·  Common

Role: Recon specialist
  High Expertise. Better auto-resolve and
  early event reveals. Faster missions.

Stats at Level 1
  Power:       9   (combat effectiveness)
  Resilience:  11  (reduces injury chance)
  Expertise:   13  (event bonuses and unlocks)

Cost: 35 Warband Marks
Balance: 240 Marks  →  205 after recruit

Roster: 3 / 5 slots used
  [ Enter ] Recruit    [ Esc ] Cancel
```

**Affordability feedback:**
- If `marks >= cost`: cost in `Color::Green`, action available
- If `marks < cost`: cost in `Color::Red`, "Insufficient Marks" flash message on Enter
- If `roster >= max_roster`: "Roster full" flash message, action blocked

**Pool refresh timer:**
- `refreshed_at + 24h - now` as `"Xh Ym"` countdown
- If `needs_refresh(now)` is true: `"Pool ready for refresh"` in `Color::Yellow`

### 2.3 Required Code Change in `deep_scene.rs`

```rust
// Before (bug):
DeepView::Roster | DeepView::Recruit => {
    super::deep_roster::render_roster(buffer, width, height, deep, ui, ctx);
}

// After (fix):
DeepView::Roster => {
    super::deep_roster::render_roster(buffer, width, height, deep, ui, ctx);
}
DeepView::Recruit => {
    super::deep_roster::render_recruit(buffer, width, height, deep, ui, ctx);
}
```

---

## Part 3: Layer Map — Depth Progression and Infrastructure Clarity

### 3.1 Current Problems (Confirmed by Audit)

**Familiarity tier labels never shown (P1.1):**
- Shows `"Intel: 42%"` — not the named level (Mapped), not its effect (-10% duration)
- The audit calls this a high-value/low-complexity fix

**Tier color suppressed in compact mode (P2.5):**
- `let _ = tc;` on line 155 of `deep_layers.rs` — intentionally ignores the tier color
- Layer numbers in compact mode render in `Color::White` instead of tier color

**Infrastructure costs hidden (P1.5):**
- Unbuilt infrastructure shows description but no Warband Marks cost
- Players cannot plan builds without knowing cost

**Build action missing (P2.4):**
- The Layers view is read-only; no keybind to build infrastructure
- Players must know to use the mission system for Construction

**Flat list with no depth metaphor:**
- Tier boundaries are not visible in the list
- All layers look the same regardless of depth

**Familiarity bar has no threshold markers:**
- Bar is filled 0-100% but thresholds at 25/50/75 are not marked
- No way to know how far away the next named tier is

### 3.2 Redesigned Layer List — Compact Mode

**Before (buggy, all White):**
```
  L 1   The Shallows    CLEAR    [2/4]
► L 4   Brackwater      FRONT
  L 5   ???
```

**After (tier colors, status glyphs, tier headers):**
```
════ The Shallows (L1-3) ═══════════
✓  L 1  The Mirefall   ▓▓▓▓▓▓▓▓▓▓  [OC  ]
✓  L 2  Dustbone       ▓▓▓▓▓▓▓░░░  [O   ]
✓  L 3  Ashcroft       ▓▓▓▓░░░░░░  [    ]
════ The Warrens (L4-7) ════════════
►  L 4  Brackwater     ▓░░░░░░░░░  [    ]  FRONTIER
?  L 5  ???
```

- Layer number colored in tier color (not suppressed)
- Status glyphs: `✓` cleared (`Color::Green`), `►` frontier+selected (`Color::Cyan`), `?` unknown (`Color::DarkGray`)
- 8-char familiarity mini-bar: `▓` filled `Color::Cyan`, `░` empty `Color::Rgb(30,40,60)`
- 6-char infra slot: `[OCWB]` where each letter is its initial or space
- Tier section headers in tier color with `════` dividers in `Color::Rgb(40,60,80)`

### 3.3 Redesigned Layer List — Split Mode (Left Panel)

**Header:**
```
  #    Name               Fam     Infra   Status
```

**Rows:**
```
════ The Shallows ════════════════════════════════════
✓  L 1  The Mirefall   ▓▓▓▓▓▓▓▓░░  [OC  ]  Cleared
✓  L 2  Dustbone       ▓▓▓▓▓▓░░░░  [O   ]  Cleared
✓  L 3  Ashcroft       ▓▓▓░░░░░░░  [    ]  Cleared
════ The Warrens ════════════════════════════════════
►  L 4  Brackwater     ▓░░░░░░░░░  [    ]  FRONTIER
   L 5  ???
```

Tier headers appear as rows at every tier boundary, using `═` character in `Color::Rgb(40,60,80)` with tier name in tier color.

### 3.4 Redesigned Layer Detail Panel — Cleared Layer

**Full mockup:**
```
Layer 2 — Dustbone
The Shallows  ·  Cleared

Familiarity: Mapped (45%)
  ▓▓▓▓▓▓▓▓▓░░░░░░░░░░░░
  [0%]·········[25%]·····[50%]·····[75%]·[100%]
  Effect: -10% mission duration

Infrastructure  [2/4 built]
  [✓] Outpost      -25% duration on this layer
  [✓] Supply Cache +50% Marks from supply runs
  [ ] Watchtower   +25 intel instantly           74M
  [ ] Bridge       Skip this layer on deep push  85M

Duration after all bonuses:
  Supply Run:  2.0h → 1.35h
  Recon:       4.0h → 2.70h
  Expedition:  8.0h → 5.40h
```

**Key design decisions:**
- Familiarity label: `"Mapped (45%)"` not `"Intel: 45%"` — uses the game term from `FamiliarityLevel::display_name()`
- Familiarity color by tier: Unknown=`DarkGray`, Mapped=`Cyan`, Familiar=`Green`, Mastered=`Rgb(255,215,0)`
- Threshold markers on the bar row (below the fill bar)
- Effect text: concrete description of the named tier's bonus
- Infrastructure: `[✓]` built in `Color::Green`, `[ ]` unbuilt in `Color::DarkGray`
- Costs shown on right for unbuilt slots only, in `Color::DarkGray`
- Duration section: base hours × combined modifier factor, shown as `X.Xh → Y.Yh`

**Familiarity threshold marker implementation:**
```
Row 0:  ▓▓▓▓▓▓▓▓▓░░░░░░░░░░░░  (fill bar, 21 chars wide)
Row 1:  ▲        ▲        ▲    (tick marks at 25%, 50%, 75%)
```

Tick position calculation: `tick_col = col + (threshold * bar_width / 100)`

### 3.5 Redesigned Layer Detail Panel — Frontier Layer

```
Layer 4 — Brackwater
The Warrens  ·  FRONTIER

Familiarity: Unknown (8%)
  ▓░░░░░░░░░░░░░░░░░░░░
  [0%]·········[25%]·····[50%]·····[75%]·[100%]
  Effect: No duration bonus yet. Explore to unlock.

Power Required
  Supply Run:    25   (safe farming)
  Recon:         40   (low risk, build intel)
  Expedition:    55   (medium risk, primary XP)
  Breakthrough:  75   (one-time — clears this layer)

Infrastructure: Available after Breakthrough
  [ ] Outpost      -25% duration        Cost: 76M
  [ ] Supply Cache +50% Marks yield     Cost: 100M
  [ ] Watchtower   +25 intel instantly  Cost: 86M
  [ ] Bridge       Skip this layer      Cost: 120M

Next step: Send a Breakthrough mission to clear this layer.
```

**Power threshold section:** Uses `layer_power_thresholds(layer.index)` from `layers.rs`. Shows all four mission types with concise parenthetical explaining the mission.

**Infrastructure for uncleared layers:** Shown dimmed (`Color::DarkGray`) with costs visible but labeled "Available after Breakthrough."

### 3.6 Redesigned Layer Detail Panel — Unknown Layer

```
Layer 5 — ???
The Warrens  ·  UNKNOWN

 Nothing is known about this layer yet.
 Clear Layer 4 (Brackwater) to reveal it.

 Estimated power required:
  Breakthrough: ~95  (based on tier progression)
```

### 3.7 Infrastructure Build Action (P2.4)

Add a `[B]` keybind to the Layers detail view for uncleared→cleared layers with buildable slots:

**In `deep_layers.rs` detail panel footer:**
```
[↑/↓] Navigate   [B] Build Infrastructure   [Esc] Back
```

When `[B]` is pressed on a cleared layer with available slots and sufficient Marks, open a sub-menu:

```
BUILD INFRASTRUCTURE — Layer 2

  [O] Outpost       -25% duration            Cost: 72M
  [C] Supply Cache  +50% Mark yield          Cost: 90M
  [W] Watchtower    +25 intel on build        Cost: 82M
  [B] Bridge        Skip layer on deep push   Cost: 105M

  Balance: 240M
  [Enter] Build   [Esc] Cancel
```

The build action triggers `build_infrastructure()` from `layers.rs` and deducts Warband Marks. This requires adding a build sub-view state to `DeepUiState` (e.g., a `building_layer: Option<u32>` field) and a new input handler branch.

**Implementation note:** The input handler in `deep_input.rs` / `handle_infrastructure` already exists. Add a `KeyCode::Char('b') | KeyCode::Char('B')` branch that sets a new `ui.build_mode = true` flag, and render the sub-menu when that flag is set.

### 3.8 Tier Color Bug Fix (P2.5)

In `src/ui/deep_layers.rs` line 155, change:
```rust
// Before (bug — tier color computed then discarded):
let _ = tc;
row += 1;

// After (fix — apply tier color to layer number):
put_text(buffer, row, 3, &format!("L{:2}", layer.index), tc);
row += 1;
```

This matches the behavior already present in the split view.

### 3.9 Familiarity Tier Colors

Consistent color mapping for familiarity tier labels:
- Unknown (0-24%): `Color::DarkGray`
- Mapped (25-49%): `Color::Cyan`
- Familiar (50-74%): `Color::Green`
- Mastered (75-100%): `Color::Rgb(255, 215, 0)` (gold)

---

## Part 4: Tab Bar State Indicators (P1.3)

### 4.1 Current Tab Bar

```
[Hub] [Missions] [Roster] [Layers] [Recruit]
```

Active tab in `Color::Rgb(80, 160, 220)`, others in `Color::DarkGray`.

### 4.2 Proposed Tab Bar with State Badges

```
[Hub] [Missions 3] [Roster ⚡2] [Layers] [Recruit ●]
```

Badge rules (badges appear only when their tab is not active):

| Tab | Badge | Condition | Color |
|-----|-------|-----------|-------|
| Hub | `⚡N` | N events pending | `Color::Yellow` |
| Hub | `✓N` | N results awaiting collection | `Color::Green` |
| Missions | `N` | N missions in pool | `Color::Cyan` |
| Roster | `⚕N` | N mercs injured or lost | `Color::Yellow` |
| Recruit | `●` | Pool has candidates and is fresh | `Color::Cyan` |
| Recruit | `!` | Pool refresh due in < 1h | `Color::Yellow` |

**Implementation in `deep_scene.rs`:**
```rust
fn tab_badge(view: DeepView, deep: &DeepState, now: DateTime<Utc>) -> Option<(String, Color)> {
    match view {
        DeepView::Hub => {
            let events = deep.prestige.active_missions.iter()
                .filter(|m| m.has_pending_event()).count();
            let results = deep.prestige.pending_results.len();
            if events > 0 {
                Some((format!("⚡{}", events), Color::Yellow))
            } else if results > 0 {
                Some((format!("✓{}", results), Color::Green))
            } else {
                None
            }
        }
        DeepView::NewMission => {
            let n = deep.prestige.available_missions.len();
            if n > 0 { Some((n.to_string(), Color::Cyan)) } else { None }
        }
        DeepView::Roster => {
            let injured = deep.prestige.roster.iter()
                .filter(|m| !matches!(m.status, MercStatus::Available | MercStatus::OnMission(_)))
                .count();
            if injured > 0 { Some((format!("⚕{}", injured), Color::Yellow)) } else { None }
        }
        DeepView::Recruit => {
            if deep.prestige.recruit_pool.candidates.is_empty() {
                None
            } else if deep.prestige.recruit_pool.needs_refresh(now) {
                Some(("!".to_string(), Color::Yellow))
            } else {
                Some(("●".to_string(), Color::Cyan))
            }
        }
        DeepView::Infrastructure | DeepView::EventResponse => None,
    }
}
```

Tab label rendering with badge:
```rust
let label = format!("[{}{}]",
    tab.tab_label(),
    badge.as_ref().map(|(s, _)| format!(" {}", s)).unwrap_or_default()
);
put_text(buffer, tab_row, col, &label, if is_active { active_color } else { inactive_color });
if let Some((badge_str, badge_color)) = &badge {
    // Re-render just the badge portion in badge color
    let badge_col = col + tab.tab_label().len() as i32 + 2; // "[label "
    put_text(buffer, tab_row, badge_col, badge_str, *badge_color);
}
```

---

## Part 5: Construction Mission Label Fix (P0.3)

**In `src/ui/deep_missions.rs`**, wherever `m.mission_type.display_name()` is used in the mission list:

```rust
// Before:
let type_label = m.mission_type.display_name().to_string();

// After:
let type_label = match m.mission_type {
    MissionType::Construction(infra) => format!("Build {}", infra.display_name()),
    other => other.display_name().to_string(),
};
```

This is a two-line change that immediately makes Construction missions self-describing. "Build Outpost", "Build Bridge", "Build Supply Cache", "Build Watchtower" — no ambiguity.

---

## Part 6: Implementation Notes and Helper Functions

### 6.1 New Helper Functions for `deep_roster.rs`

```rust
/// Archetype 3-letter abbreviation in brackets.
fn archetype_abbrev(archetype: MercArchetype) -> &'static str {
    match archetype {
        MercArchetype::Vanguard  => "[VAN]",
        MercArchetype::Scout     => "[SCT]",
        MercArchetype::Arcanist  => "[ARC]",
        MercArchetype::Medic     => "[MED]",
        MercArchetype::Saboteur  => "[SAB]",
    }
}

/// Role tag and description for the detail panel.
fn archetype_role_desc(archetype: MercArchetype) -> (&'static str, &'static str) {
    match archetype {
        MercArchetype::Vanguard  => ("Frontline tank",
            "High Power + Resilience. Reduces squad casualties."),
        MercArchetype::Scout     => ("Recon specialist",
            "High Expertise. Better auto-resolve, faster missions."),
        MercArchetype::Arcanist  => ("Elemental expert",
            "Highest Expertise. Counters hazards. Fragile."),
        MercArchetype::Medic     => ("Squad healer",
            "Highest Resilience. Prevents permanent loss."),
        MercArchetype::Saboteur  => ("Trap specialist",
            "High Expertise. Speeds missions, alternate routes."),
    }
}

/// Cumulative missions completed to reach a given level (level 1 = 0).
fn missions_to_reach_level(level: u32) -> u32 {
    (1..level).map(|l| Mercenary::missions_to_next_level(l)).sum()
}

/// (progress_within_level, missions_needed_for_this_level)
fn level_progress(merc: &Mercenary) -> (u32, u32) {
    let base = missions_to_reach_level(merc.level);
    let needed = Mercenary::missions_to_next_level(merc.level);
    let progress = merc.missions_completed.saturating_sub(base).min(needed);
    (progress, needed)
}

/// Injury severity label and color from missions_remaining.
fn injury_severity_display(missions_remaining: u32) -> (&'static str, Color) {
    match missions_remaining {
        1 => ("Light injury", Color::Yellow),
        2 => ("Moderate injury", Color::LightRed),
        _ => ("Severe injury", Color::Red),
    }
}

/// Hour estimate from missions_remaining (1 mission ≈ 6h).
fn injury_hours_estimate(missions_remaining: u32) -> u32 {
    missions_remaining * 6
}
```

### 6.2 New Helper Functions for `deep_layers.rs`

```rust
/// Familiarity level label and color.
fn familiarity_label_color(familiarity: u8) -> (&'static str, Color) {
    match familiarity {
        0..=24  => ("Unknown",  Color::DarkGray),
        25..=49 => ("Mapped",   Color::Cyan),
        50..=74 => ("Familiar", Color::Green),
        _       => ("Mastered", Color::Rgb(255, 215, 0)),
    }
}

/// Effect text for a familiarity level.
fn familiarity_effect_text(familiarity: u8) -> &'static str {
    match familiarity {
        0..=24  => "No duration bonus yet",
        25..=49 => "-10% mission duration",
        50..=74 => "-20% mission duration",
        _       => "-30% duration, +15% Mark yield",
    }
}

/// Infrastructure 4-slot display string: "[OCWB]".
fn infra_slots_str(layer: &LayerRecord) -> String {
    let o = if layer.has_infrastructure(Infrastructure::Outpost)     { 'O' } else { ' ' };
    let c = if layer.has_infrastructure(Infrastructure::SupplyCache) { 'C' } else { ' ' };
    let w = if layer.has_infrastructure(Infrastructure::Watchtower)  { 'W' } else { ' ' };
    let b = if layer.has_infrastructure(Infrastructure::Bridge)      { 'B' } else { ' ' };
    format!("[{}{}{}{}]", o, c, w, b)
}

/// Familiarity bar with threshold markers, rendered into scene buffer.
/// Renders two rows: fill bar at `row`, tick marks at `row+1`.
fn render_familiarity_bar_with_thresholds(
    buffer: &mut [Vec<SceneCell>],
    row: i32, col: i32, bar_width: usize, familiarity: u8,
) {
    let ratio = familiarity as f64 / 100.0;
    let filled = ((ratio * bar_width as f64).round() as usize).min(bar_width);
    for i in 0..filled {
        put_cell(buffer, row, col + i as i32, '▓', Color::Cyan);
    }
    for i in filled..bar_width {
        put_cell(buffer, row, col + i as i32, '░', Color::Rgb(30, 40, 60));
    }
    // Threshold tick marks below
    for threshold in [25usize, 50, 75] {
        let tick_col = col + (threshold * bar_width / 100) as i32;
        put_cell(buffer, row + 1, tick_col, '▲', Color::DarkGray);
    }
}

/// Duration in seconds as "X.Xh".
fn format_hours(secs: u64) -> String {
    format!("{:.1}h", secs as f64 / 3600.0)
}

/// Compute duration after infrastructure and familiarity bonuses.
fn effective_duration_hours(
    tier: LayerTier,
    mission_type: MissionType,
    layer: &LayerRecord,
) -> f64 {
    let base = crate::deep::layers::base_mission_duration_secs(tier, mission_type) as f64;
    let outpost = if layer.has_infrastructure(Infrastructure::Outpost) { 0.75 } else { 1.0 };
    let fam = crate::deep::layers::FamiliarityLevel::from_familiarity(layer.familiarity)
        .duration_factor();
    (base * outpost * fam) / 3600.0
}
```

### 6.3 Import Additions

In `src/ui/deep_layers.rs`, add:
```rust
use crate::deep::layers::{
    FamiliarityLevel, infrastructure_build_cost, layer_power_thresholds,
    base_mission_duration_secs,
};
```

In `src/ui/deep_roster.rs`, add:
```rust
use crate::deep::mercenaries::stats_at_level;
```

### 6.4 Recommended Type Change

Add `pub quality: MercQuality` to `Mercenary` in `src/deep/types.rs` and move `MercQuality` into `types.rs` (or re-export from `mercenaries.rs`). Update `generate_mercenary` to store it. Without this, quality display requires stat inference which is imprecise.

---

## Part 7: Responsive Degradation

### XL/L (width >= 80)
- Full split layout
- Tier section headers, full detail panels
- Infrastructure detail cards with costs
- Duration-after-bonuses calculations
- Tab bar with state badges

### M (60-79)
- Abbreviated split (50/50)
- Tier section headers condensed to `═ Shallows ═`
- Detail panel: role description 1 line only, no duration math
- Infrastructure: name + main effect, no costs

### S (< 60)
- Compact single-column list
- Tier section headers as 1-char indicators (tier color on layer number)
- No detail panel; status shown below selected item inline
- Tab bar badges still rendered (they fit in < 5 chars)

---

## Part 8: Summary of Changes Required

### `src/ui/deep_scene.rs`
- Fix `DeepView::Recruit` dispatch to call `render_recruit()` instead of `render_roster()`
- Add `tab_badge()` helper and update tab bar rendering to show badges

### `src/ui/deep_roster.rs`
- Add `render_recruit()` function (entire new function)
- Rewrite compact roster header and rows with fixed column offsets (no rfind)
- Rewrite split roster detail panel with role description, stat hints, progress bar, injury hours estimate
- Add all helper functions from section 6.1

### `src/ui/deep_layers.rs`
- Fix compact mode tier color bug (remove `let _ = tc`)
- Add tier section headers to both compact and split list
- Expand familiarity display to named tier + effect text
- Add familiarity bar with threshold tick marks
- Expand infrastructure display to include costs and concrete descriptions
- Add duration-after-bonuses section for cleared layers
- Add power threshold section for frontier layers
- Add build sub-menu state and `[B]` keybind handling
- Add all helper functions from section 6.2

### `src/ui/deep_missions.rs`
- Fix Construction mission label (P0.3): replace `display_name()` with match on Construction payload

### `src/deep/types.rs` (optional but recommended)
- Add `quality: MercQuality` field to `Mercenary`

### `src/deep/mercenaries.rs` (if quality field added)
- Re-export `MercQuality` or move it to `types.rs`
- Set `quality` field in `generate_mercenary()`
