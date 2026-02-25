# The Deep — Onboarding and Contextual Help System Design

**Updated:** 2026-02-24 — Incorporates findings from `deep-ui-audit.md`

## Overview

The Deep is the most complex endgame system in Quest3, requiring players to understand
wall-clock mission timing, squad composition, layer progression, infrastructure investment,
and the prestige reset cycle — all before seeing meaningful payoff. This document defines
a contextual help strategy that communicates these mechanics inline, non-intrusively, and
progressively as players engage with the system.

**Design principle**: experienced players should never be slowed down. New players get
exactly enough information at the moment of need — no more.

This document covers onboarding design only. Companion documents define the broader
UX improvements (view hierarchy, visual polish) to which this onboarding layer attaches.

---

## 1. Audit Findings — Summary

The sys-architect audit (`deep-ui-audit.md`) identified the following issues that directly
affect new-player comprehension. This document addresses the onboarding dimension of each.

### Critical (P0) — Blocking Onboarding

| Issue | Description | Onboarding Impact |
|-------|-------------|------------------|
| P0.1 | Recruit tab has no rendering — shows Roster instead | Players cannot discover mercenary hiring |
| P0.2 | Mission `description` field never shown | Missions feel identical; purpose is opaque |
| P0.3 | Construction missions show "Construction" not "Build Outpost" etc. | Players cannot tell what they are building |

### High Impact (P1) — Core Comprehension Gaps

| Issue | Description | Onboarding Impact |
|-------|-------------|------------------|
| P1.1 | Familiarity tier labels never shown (Unknown/Mapped/Familiar/Mastered) | Players don't understand the progression system |
| P1.2 | Guild rank upgrade path completely hidden | No visible path to account-level advancement |
| P1.3 | Tab bar carries no state indicators | Events and injuries require navigation to discover |
| P1.4 | Power ratio shown as N/N but not as percentage | Success bands are unclear |
| P1.5 | Infrastructure build costs not shown in Layers view | Cannot plan without costs |

### Other Gaps Not in Audit

- The prestige reset mechanic (mercs reset, infrastructure persists) is never communicated
- Stat meanings (Power/Resilience/Expertise) have no in-panel descriptions
- Risk tier labels have no consequence explanation
- The auto-resolve event mechanic is the one well-explained mechanism; serve as reference

---

## 2. Onboarding Strategy

### Approach: Three-Layer Progressive Disclosure

Rather than a separate tutorial screen or help modal, information is embedded at the
point of need across three layers:

**Layer 1 — Structural fixes (P0/P1)**
These are not "help text" — they are missing information the UI should always show.
Mission descriptions, construction labels, familiarity tiers, build costs, and power
percentages are data the player needs to make decisions. They belong in the primary UI.

**Layer 2 — First-visit contextual hints**
For mechanics that are harder to discover (prestige cycle, stat meanings, risk consequences),
show one-line hints for the first N visits to a view. These fade out automatically — never
returning once the player has seen them. No toggle needed; no memory needed.

**Layer 3 — On-demand reference ([?] key)**
A per-view reference panel toggled with [?]. Experienced players ignore it. New players
who want depth can access a condensed reference at any time without leaving the overlay.

This approach is consistent with how Haven uses room descriptions and tier cost displays
to teach the system without a separate tutorial.

---

## 3. Discovery Modal — The First Impression

The discovery modal is the player's first contact with The Deep. Currently:

```
  The Deep Discovered!

  A scarred mercenary captain approaches,
  maps of underground passages in hand.
  "The Deep goes further than you know."

  Press [D] to visit.  [Enter] to dismiss.
```

**Problem**: No explanation of what The Deep is, what makes it different from other
systems, or that its infrastructure survives prestige (the core hook).

**Revised modal content**:

```
  The Deep Discovered!

  A scarred mercenary captain approaches,
  maps of underground passages in hand.
  "The Deep goes further than you know."

  Send mercenaries on hour-long expeditions.
  Earn Marks, items, and Prestige Rank fragments.
  Infrastructure you build here survives prestige.

  Press [D] to visit.  [Enter] to dismiss.
```

**Rationale for each added line**:
- "Send mercenaries on hour-long expeditions" — establishes the real-time mechanic
- "Earn Marks, items, and Prestige Rank fragments" — establishes the reward structure
- "Infrastructure you build here survives prestige" — the generational hook; the key
  reason to invest in The Deep even near prestige time

**Implementation**: `src/ui/deep_scene.rs: render_deep_discovery_modal()`.
Add three `Line::from(Span::styled(..., Color::Cyan))` entries before the footer hint.
Modal height increases from 11 to 14 to accommodate; cap at `area.height.saturating_sub(4)`.

---

## 4. Hub View — First-Visit Onboarding

### 4.1 Guild Rank Upgrade Path (P1.2 — structural fix)

The audit confirmed guild rank upgrade requirements are completely hidden. This is a
structural data gap, not a tooltip problem. The hub header should always show:

**Current header (two lines)**:
```
  Guild: Freelancers (Rank 1)    Marks: 1,240
  Frontier: Layer 3 (The Shallows)    Mercs: 3/5
```

**Revised header (three lines)**:
```
  Guild: Freelancers (Rank 1)    Marks: 1,240    Concurrent: 1/1
  Frontier: Layer 3 (The Shallows)    Mercs: 3/5    Available: 2
  Next rank: Layer 3 Breakthrough  →  Rank 2 (Sellswords, 7 mercs, 1 concurrent)
```

Third line uses `Color::Rgb(60, 100, 140)` — distinct from the header rows but not
alarming. When already at max rank (5/Legion), replace with "Max guild rank reached."

**Data sources**:
- `guild_rank.concurrent_missions()` — concurrent cap
- `prestige.available_merc_count()` — ready mercs
- `guild_rank.next()` — next rank name
- `guild_rank.next()?.required_breakthrough_layer()` — layer requirement
- Next rank max roster size from `GUILD_RANK_STATS`

### 4.2 Empty State Messaging

The empty state message when no missions are active is the primary teaching moment
for new players. Current text is too terse.

**Current**:
```
  No active missions.
  [Tab] to Missions view to deploy a squad.
```

**Revised** (first visit only, `hub_visit_count == 0`):
```
  No active missions running.

  Start with a Supply Run — safe income, no risk, 2-4 hours.
  Missions continue while the game is closed.

  [Tab] Switch View  to deploy your first squad.
```

After `hub_visit_count >= 1`, the hint collapses back to:
```
  No active missions.
  [Tab] Switch View  to deploy a squad.
```

Color: hint lines in `Color::Rgb(50, 80, 110)`.

### 4.3 Prestige Cycle Hint (First Visit Only)

When `hub_visit_count == 0`, show a one-line persistent reminder between the header
and the mission list area:

```
  ──────────────────────────────────────────────────────────────
  Tip: Mercs and Marks reset on prestige. Infrastructure persists.
  ──────────────────────────────────────────────────────────────
```

Color: `Color::Rgb(50, 80, 110)`. Shown for exactly one session (first visit to Hub).
Clears permanently when `hub_visit_count` increments past 1.

---

## 5. New Mission View — Onboarding

### 5.1 Mission Description (P0.2 — structural fix, always shown)

The `AvailableMission.description` field contains thematic context for every mission.
It is never rendered. This should always be visible in the detail panel — not a hint,
not toggled, always present.

**Revised detail panel layout (split view)**:
```
  Layer 3 — The Shallows
  Recon  ·  Low risk

  Survey the Shallows for entry points    ← description (word-wrapped, 2 lines max)
  and note hazards for future squads.

  Duration: 4h 0m
  Reward:   Marks + Familiarity (intel)

  Requires:
    Min Power 15
    Scout recommended
```

**Implementation**: In `render_new_mission_split()` in `deep_missions.rs`, after the
layer/tier header row, word-wrap `m.description` to `detail_inner_w` characters and
render in `Color::DarkGray`. Use the existing word-wrap pattern from the event view
(split on whitespace, accumulate to line width limit). Maximum 2 lines to preserve space.

### 5.2 Construction Mission Label (P0.3 — structural fix, always shown)

Replace `m.mission_type.display_name()` in mission list rows with:

```rust
let type_label = match &m.mission_type {
    MissionType::Construction(infra) => format!("Build {}", infra.display_name()),
    other => other.display_name().to_string(),
};
```

Apply to both compact and split list panels. This is a one-line fix with zero complexity.

### 5.3 Mission Type Descriptions (First-Visit Contextual Hints)

When `mission_visit_count < 5`, show a one-line description after the risk line in
the detail panel. These describe what the mission type accomplishes, not just its label:

| Mission Type | One-Line Description |
|-------------|---------------------|
| Supply Run | Safe income — always returns, earns Marks reliably |
| Recon | Raises layer familiarity — cuts future mission times |
| Expedition | Core rewards — items, Marks, and merc XP |
| Breakthrough | Clears the frontier — unlocks the next layer |
| Construction | Builds permanent infrastructure — survives prestige |

Color: `Color::Rgb(50, 80, 110)` on first 5 visits, then hidden.

### 5.4 Power Ratio Percentage (P1.4 — structural fix, always shown)

Replace `Power: 32/25` with `Power: 32/25  (128%)` in both compact and split views.
Color the percentage based on success band:

| Ratio | Color |
|-------|-------|
| >= 150% | `Color::Rgb(80, 220, 120)` (bright green — overpowered) |
| >= 100% | `Color::Green` |
| >= 75% | `Color::Yellow` |
| < 75% | `Color::LightRed` |

Safe missions (Supply Run, Construction) show `Always succeeds` instead of a percentage.

### 5.5 Risk Consequence Descriptions (First-Visit Hints)

When `mission_visit_count < 5`, enhance the risk display with a consequence note:

| Risk | Current | Enhanced |
|------|---------|---------|
| Safe | `Safe` | `Safe  — no injuries, guaranteed return` |
| Low | `Low` | `Low   — rare injuries, Marks lost on failure` |
| Medium | `Medium` | `Medium — injuries likely on failure` |
| High | `High` | `High  — injuries or death possible on failure` |

Consequence text in `Color::DarkGray`. Collapse to just the label after 5 visits.

### 5.6 Panel Focus Indicator (UX Fix — always shown)

The audit noted (3.2) that the squad assignment panel has no visual "active" indicator
when focus moves to it. Add a header row color change: when `staging_mission_index.is_some()`,
render `"Assign Squad:"` in `Color::Rgb(80, 160, 220)` (DEEP_BORDER_COLOR — active).
When unfocused, render in `Color::DarkGray`. This is a low-effort signal with immediate clarity.

---

## 6. Roster View — Onboarding

### 6.1 Stat Descriptions (First-Visit Hints)

In the split-panel detail view, add inline descriptions after each stat value.
Show when `roster_visit_count < 3`:

```
  Stats:
    Power:      52      — drives mission success
    Resilience: 48      — reduces injury risk
    Expertise:  24      — unlocks archetype event choices
```

The `— description` text in `Color::DarkGray`. After 3 visits, collapse to stat values only.

**Implementation**: In `render_roster_split()` in `deep_roster.rs`, conditionally append
the description string to each `put_text` stat line based on visit count.

### 6.2 Injury Recovery Explanation

The audit noted (2.3) that injury shows "Injured (2 missions)" with no explanation of what
"missions" means in the context of wall-clock recovery.

**Current**: `"Injured (3 missions)"`
**Proposed**: `"Injured — recovers after 3 missions complete (approx. 18h)"`

The approximate hour estimate is `missions_remaining * 6` (average mission duration).
Show in compact form on the list row, full form in the detail panel.

### 6.3 Merc Leveling Progress (P2.1 from audit)

In the detail panel, add a leveling progress indicator after the level/missions line:

```
  Level: 3    Missions: 6

  Progress to Lv4:  [███████░░] 7/9 missions
```

Where `missions_to_next_level(3) = 3 + 3*2 = 9` and `missions_completed % 9 = 6`.
Rendered as a compact block bar using the existing `render_progress_bar` pattern.

Bar color: `Color::Cyan`. Empty cells: `Color::Rgb(30, 40, 60)`.

---

## 7. Recruit View — Must Be Implemented (P0.1)

The Recruit tab currently renders the Roster view silently. The audit identified this as
a critical bug. From an onboarding perspective, the Recruit view is also the primary
teaching moment for the guild economics (Marks → mercs → stronger squads).

### 7.1 Recruit View Layout (L/XL)

```
  Recruits: 4 candidates    Pool refreshes in: 16h 20m
  Roster: 3/5 open slots    Marks: 1,240
  ─────────────────────────────────────────────────────────
  ┌─ CANDIDATES ──────────────────┐  ┌─ CANDIDATE DETAIL ──────────────┐
  │ ► Gareth   Vanguard  Common  │  │ Gareth the Ironclad             │
  │   Mira     Medic     Common  │  │ Archetype: Vanguard             │
  │   Thorne   Arcanist  Uncommon│  │ Quality:   Common               │
  │   Kira     Scout     Common  │  │                                 │
  │                              │  │ Stats:                          │
  │   [Recruit slot: 2 open]     │  │   Power:      14  — high        │
  │                              │  │   Resilience: 12  — medium      │
  │                              │  │   Expertise:   4  — low         │
  │                              │  │                                 │
  │                              │  │ Cost: 60 Marks  [affordable]    │
  │                              │  │                                 │
  │                              │  │ [Enter] Recruit  [Esc] Back     │
  └──────────────────────────────┘  └─────────────────────────────────┘
  [↑/↓] Navigate  [Enter] Recruit  [Esc] Back
```

### 7.2 Recruit View Layout (S — compact)

```
  Recruits: 4    Marks: 1,240    Roster: 3/5
  ► Gareth   Vanguard  Pwr:14  60M  [Recruit]
    Mira     Medic     Pwr:6   45M
    Thorne   Arcanist  Pwr:10  80M  [Uncommon]
    Kira     Scout     Pwr:8   55M
  Pool refreshes in: 16h 20m
  [↑/↓] Navigate  [Enter] Recruit  [Esc] Back
```

### 7.3 First-Visit Onboarding Hint for Recruit View

When `recruit_visit_count == 0`, show above the candidate list:

```
  Tip: Stronger archetypes unlock at higher Guild Ranks.
       Pool refreshes every 24 hours.
```

Color: `Color::Rgb(50, 80, 110)`. Shown once.

### 7.4 Affordability Feedback

Cost display uses color-coded affordability (matching Haven's pattern):
- `Color::Green` when marks >= cost
- `Color::Red` when marks < cost
- Error flash message "Insufficient Warband Marks" when [Enter] pressed on unaffordable recruit

### 7.5 Quality Color

Candidate quality is shown with rarity-adjacent colors:

| Quality | Color |
|---------|-------|
| Common | `Color::White` |
| Uncommon | `Color::Green` |
| Rare | `Color::Yellow` |
| Elite | `Color::Magenta` |

---

## 8. Layers View — Onboarding

### 8.1 Familiarity Tier Labels (P1.1 — structural fix, always shown)

Replace `"Intel:  42%  [bar]"` with `"Familiarity: 42%  [Mapped -10%]  [bar]"`.

Tier label in brackets, colored by tier:

| Level | % Range | Label | Duration Reduction | Color |
|-------|---------|-------|-------------------|-------|
| Unknown | 0–24% | `[Unknown]` | none | `Color::DarkGray` |
| Mapped | 25–49% | `[Mapped]` | -10% | `Color::Cyan` |
| Familiar | 50–74% | `[Familiar]` | -20% | `Color::Green` |
| Mastered | 75–100% | `[Mastered]` | -30% | `Color::Rgb(255, 215, 0)` |

Include the reduction percentage in the label. This communicates both the tier name
and its mechanical effect without requiring a tooltip.

### 8.2 Total Duration Reduction Display

After the familiarity bar, add a "Combined reduction" line that shows the total modifier
from all sources (Outpost + familiarity + Saboteur):

```
  Familiarity: 75%  [Mastered -30%]  ████████████████████████░░░░░
  Duration reduction: -55%  (Outpost -25%  Mastered -30%)
```

The summary line uses `Color::Cyan` for the total and `Color::DarkGray` for the breakdown.
This is particularly valuable for players deciding whether to build an Outpost on a
layer that already has high familiarity.

**Data source**: `layer.total_duration_reduction()` method exists; breakdown comes from
checking `layer.has_infrastructure(Infrastructure::Outpost)` and `familiarity_level`.

### 8.3 Infrastructure Build Costs (P1.5 — structural fix, always shown)

For unbuilt infrastructure in the detail panel, show the Warband Marks cost:

**Current**:
```
  [ ] Watchtower   +intel
```

**Revised**:
```
  [ ] Watchtower   +intel, +25 familiarity on build    280M
```

Cost in `Color::Yellow` when affordable, `Color::Red` when not.
Call `infrastructure_build_cost(infra, layer.index)` for each unbuilt slot.

### 8.4 First-Visit Infrastructure Context Hint

When `layer_visit_count < 3`, show a one-line hint at the bottom of the infrastructure
list in the detail panel:

```
  Build via Construction missions (safe, 4-8h). Permanent.
```

Color: `Color::Rgb(50, 80, 110)`. Disappears after 3 visits.

### 8.5 Next Guild Rank Breakthrough Target

In the layer list, add a visual marker for the layer required for the next guild rank
upgrade. For example, if Layer 7 Breakthrough unlocks Rank 3, annotate Layer 7 in the
list:

```
  L7  The Warrens  [CLEAR]  ★ Rank 3 unlock
```

The `★` marker in `Color::Rgb(255, 215, 0)` (gold) draws attention to the strategic
layer milestone. Only show when the player hasn't yet reached that rank.

---

## 9. Event Response View — Onboarding

### 9.1 First-Event Hint

When `event_visit_count == 0`, show a two-line explanation above the choices:

```
  Your choice affects outcome and timing. Events auto-resolve safely if ignored.
```

Color: `Color::Rgb(50, 80, 110)`. Collapsed after first visit — the existing auto-resolve
countdown already serves as the ongoing reminder.

### 9.2 Unavailable Choice Labels (Audit Finding 3.x)

When an archetype-gated choice cannot be selected because the required archetype is not
in the squad, add a parenthetical explanation:

**Current**: `[VANGUARD]  Break through the rubble` (in DarkGray, no explanation)
**Revised**: `[VANGUARD]  Break through the rubble  (Vanguard not in squad)`

The parenthetical in `Color::Rgb(80, 80, 80)` — visibly different from DarkGray content.

### 9.3 Consequence Time Delta (P3.2 from audit — always shown)

Replace `"— delay"` / `"— faster"` with explicit durations:

```rust
let consequence = match (choice.is_risky, choice.time_delta_secs) {
    (true, _) => "— risky".to_string(),
    (false, d) if d > 0 => format!("— +{}", format_hours(d as u64)),
    (false, d) if d < 0 => format!("— -{}", format_hours(d.unsigned_abs())),
    _ => "— safe".to_string(),
};
```

This is a one-line change to `deep_events.rs` with high player value.

---

## 10. Mission Results Modal — Onboarding

### 10.1 Familiarity Gained (P2.3 — always shown)

Add a familiarity line to the rewards section:

```
  Rewards:
    + 380 Warband Marks
    + Familiarity on Layer 3: +5%  (now 47%, Mapped)
```

`familiarity_gain(mission_type)` is deterministic. Show the new level label if it
crossed a tier boundary: `+5% → Mapped!` in `Color::Cyan`.

### 10.2 Breakthrough Layer Cleared Celebration (P2.6 — always shown)

When a Breakthrough mission succeeds, add a prominent centered line:

```
  ★  LAYER 3 CLEARED — Layer 4 Unlocked!  ★
```

Color: `Color::Rgb(255, 215, 0)` (Gold). Centered. This is the major milestone
players push toward; it deserves a moment of recognition.

### 10.3 Post-Collection Balance Preview

After collecting rewards, flash the updated mark balance in the modal before dismissal:

```
  [Enter] Collect and Close
  Marks after: 1,620
```

The "Marks after" line appears immediately when [Enter] is pressed (before the modal
closes), giving players confirmation of the economic transaction before the view clears.

---

## 11. Tab Bar State Indicators (P1.3)

The tab bar is the navigation hub for the entire overlay. Adding state indicators
eliminates the need to navigate to each view to discover pending events or fresh recruits.

### 11.1 Indicator Design

Append compact badges after each tab label:

| Tab | Condition | Badge | Color |
|-----|-----------|-------|-------|
| `[Hub]` | Mission complete, awaiting collect | `✓` | `Color::Green` |
| `[Hub]` | Mission event pending | `⚡` | `Color::Yellow` |
| `[Missions]` | Missions available in pool | count `·N` | `Color::Cyan` |
| `[Roster]` | Mercs injured or lost | `!N` | `Color::Yellow` |
| `[Recruit]` | Pool has candidates | `●` | `Color::Cyan` |

**Priority**: When multiple conditions apply, show the most urgent (event pending >
mission complete > injured mercs > mission count > recruit available).

**Examples**:
```
  [Hub ⚡] [Missions] [Roster !2] [Layers] [Recruit ●]
```

### 11.2 Implementation

In `render_tab_bar()` in `deep_scene.rs`, pass `&DeepState` and append indicators
after the tab label before closing `]`. The function currently takes only `active: DeepView`.

```rust
fn render_tab_bar(
    buffer: &mut [Vec<SceneCell>],
    width: usize,
    active: DeepView,
    deep: &DeepState,  // add this
) {
    // ... for each tab:
    let badge = match tab {
        DeepView::Hub => {
            if deep.prestige.has_any_pending_event() { " ⚡" }
            else if !deep.prestige.pending_results.is_empty() { " ✓" }
            else { "" }
        }
        DeepView::Roster => {
            let injured = deep.prestige.roster.iter()
                .filter(|m| matches!(m.status, MercStatus::Injured { .. } | MercStatus::Lost))
                .count();
            if injured > 0 { &format!(" !{}", injured) } else { "" }
        }
        DeepView::Recruit => {
            if !deep.prestige.recruit_pool.candidates.is_empty() { " ●" } else { "" }
        }
        _ => "",
    };
    let label = format!("[{}{}]", tab.tab_label(), badge);
```

---

## 12. [?] Help Key — On-Demand Reference

Pressing [?] at any point in the Deep overlay toggles a compact reference panel.
This is the explicit escape hatch — for players who want depth, not the default experience.

### 12.1 UI Behavior

- [?] toggles `ui.show_help: bool`
- When `show_help` is true, render a 36-column reference panel on the right side (L/XL)
  or a full-width overlay (S/M), over the current view content
- Content is specific to `ui.view` (different text per view)
- No border on the reference panel — it floats visually over the scene buffer
- Background cells: `Color::Rgb(5, 8, 15)` (darker than backdrop, creates depth)
- A single `[?] Help` label in the footer row indicates the toggle is available

### 12.2 Hub Reference Content

```
  THE DEEP — Quick Reference

  Warband Marks
  Earned from missions. Resets on prestige.
  Spend on recruits and infrastructure.

  Guild Rank
  Rank 1 Freelancers  5 mercs  1 mission
  Rank 2 Sellswords   7 mercs  1 mission
  Rank 3 Company      9 mercs  2 missions
  Rank 4 Battalion   12 mercs  3 missions
  Rank 5 Legion      15 mercs  4 missions
  Advance by clearing breakthrough layers.

  Prestige Cycle
  Resets: mercs, Marks, active missions
  Survives: guild rank, cleared layers,
            infrastructure, familiarity
```

### 12.3 Mission Reference Content

```
  MISSION TYPES

  Supply Run   2-4h  Safe
  Safe income. Always returns.
  Use on cleared layers.

  Recon        4-8h  Low
  Raises layer familiarity.
  Cuts future mission times.

  Expedition   8-16h  Medium
  Main rewards: items + Marks.
  Use on frontier layers.

  Breakthrough 18-24h  High
  Clears frontier. Unlocks next.
  Earns 0.5 Prestige Rank.

  Construction 4-8h  Safe
  Builds infrastructure.
  Permanent — survives prestige.

  POWER GUIDE
  >= 150% threshold: 95% success
  >= 100% threshold: 60-90% success
  >= 75% threshold:  ~30% success
  < 75% threshold:   likely fail
```

### 12.4 Roster Reference Content

```
  MERC STATS

  Power      — mission success driver
  Resilience — injury resistance
  Expertise  — enables special choices

  ARCHETYPES
  Vanguard   high power, durable
  Scout      recon duration bonus
  Arcanist   expedition bonuses
  Medic      reduces squad injuries
  Saboteur   cuts mission time -10-15%

  LEVELING
  Complete missions to gain XP.
  Missions to level = 3 + level * 2
  Max level: 20

  INJURIES
  Light:     4-8h  (≈1 mission)
  Moderate:  8-12h (≈2 missions)
  Severe:   12-16h (≈3 missions)
  Injured mercs cannot be assigned.
```

### 12.5 Layer Reference Content

```
  FAMILIARITY (Intel %)
  Unknown  0-24%   no reduction
  Mapped   25-49%  -10% durations
  Familiar 50-74%  -20% durations
  Mastered 75-100% -30% durations

  Gain familiarity by running missions.
  Recon gives +15%, most others +5-10%.

  INFRASTRUCTURE (permanent)
  Outpost      -25% all mission times
  Supply Cache +50% Marks on Supply Runs
  Watchtower   +25 familiarity on build
  Bridge       -2h on deeper missions

  BUILD VIA Construction missions (safe).
  Costs scale with layer depth.
  Max 4 infrastructure per layer.
```

---

## 13. Visit Count Implementation

### 13.1 Data Structure Changes

Add to `DeepUiState` in `src/deep/types.rs`:

```rust
pub struct DeepUiState {
    // existing fields...
    pub hub_visit_count: u8,
    pub mission_visit_count: u8,
    pub roster_visit_count: u8,
    pub layer_visit_count: u8,
    pub event_visit_count: u8,
    pub recruit_visit_count: u8,
    pub show_help: bool,
}
```

No persistence needed — visit counts are per-session. Players who restart the game
after a long absence benefit from seeing contextual hints again.

### 13.2 Counter Increment Logic

In `src/input/deep_input.rs`, when `ui.view` changes to a new sub-view:

```rust
fn switch_view(ui: &mut DeepUiState, target: DeepView) {
    ui.view = target;
    ui.selected_index = 0;
    // Increment visit counter for the new view (saturating at 255)
    match target {
        DeepView::Hub => ui.hub_visit_count = ui.hub_visit_count.saturating_add(1),
        DeepView::NewMission => ui.mission_visit_count = ui.mission_visit_count.saturating_add(1),
        DeepView::Roster => ui.roster_visit_count = ui.roster_visit_count.saturating_add(1),
        DeepView::Infrastructure => ui.layer_visit_count = ui.layer_visit_count.saturating_add(1),
        DeepView::EventResponse => ui.event_visit_count = ui.event_visit_count.saturating_add(1),
        DeepView::Recruit => ui.recruit_visit_count = ui.recruit_visit_count.saturating_add(1),
    }
}
```

### 13.3 Rendering Pattern

```rust
// Standard conditional hint pattern
if ui.mission_visit_count < 5 {
    put_text(buffer, row, detail_col, hint_text, Color::Rgb(50, 80, 110));
    row += 1;
}
```

The threshold `< 5` for mission hints and `< 3` for stat descriptions is a design
judgment — mission type hints need more repetitions to be internalized since players
return to New Mission view frequently; stat descriptions are needed fewer times.

---

## 14. Size Tier Handling

All onboarding elements respect the existing responsive size tiers:

| Tier | Hub hints | Detail descriptions | [?] help | Tab badges |
|------|-----------|--------------------|-----------|----|
| TooSmall | No rendering | No rendering | No | No |
| S | 1-line hints only | None (no detail panel) | Full-width modal | Abbreviated |
| M | 1-line hints | Short descriptions | Full-width modal | Yes |
| L/XL | Full hints | Full descriptions with legend | Right-panel | Yes |

For S tier, the tab badges use single characters only: `⚡` `✓` `!` `●`.

---

## 15. Files Requiring Changes

| File | Change | Priority |
|------|--------|----------|
| `src/deep/types.rs` | Add visit counters + `show_help` to `DeepUiState` | P0 |
| `src/input/deep_input.rs` | Increment visit counters on view switch; [?] toggle | P0 |
| `src/ui/deep_scene.rs` | Update discovery modal; add `DeepState` param to `render_tab_bar`; tab badges | P0 |
| `src/ui/deep_missions.rs` | Mission description; construction label; power %; risk hints; mission type hints | P0 |
| `src/ui/deep_roster.rs` | Stat descriptions; injury detail; leveling progress; add `render_recruit()` | P0 |
| `src/ui/deep_layers.rs` | Familiarity tier labels; total reduction; infra costs; infra hint; guild rank milestone | P1 |
| `src/ui/deep_events.rs` | First-event hint; unavailable choice labels; explicit time deltas | P1 |
| `src/ui/deep_results.rs` | Familiarity gained; breakthrough celebration; post-collect balance | P2 |

---

## 16. Acceptance Criteria

A new player encountering The Deep for the first time should, after their first session:

1. **Understand the time scale** — know missions take hours, the game can be closed
2. **Know what Supply Run vs Breakthrough are for** — mission descriptions make this clear
3. **Know what failure means** — risk consequence descriptions explain injury probability
4. **Know how power affects success** — power percentage communicates success bands
5. **Know familiarity builds over time** — tier labels show the progression path
6. **Know infrastructure is permanent** — discovery modal and infra hint surface this
7. **Know the prestige cycle** — hub tip and discovery modal both mention it
8. **Be able to hire mercs** — Recruit view is now functional with full candidate details

An experienced player should:

1. **Never be slowed by hints** — hints disappear after 3-5 visits to each view
2. **Access help on demand** — [?] key provides per-view reference at any time
3. **See actionable tab state at a glance** — tab badges show events/injuries/recruits
4. **Not lose workflow** — all new content fits in existing layout without new modals

---

## 17. Design Decisions and Rationale

**Why not a separate tutorial modal?**
The Deep has six sub-views, each with distinct mechanics. A single tutorial modal would
either be impossibly long or incomplete. Inline progressive disclosure ensures players
receive information when they can immediately act on it.

**Why visit counts instead of persistent flags?**
Resetting on restart is a feature, not a bug. Players who haven't played in weeks
benefit from seeing hints again. The cost of showing a hint to an experienced player
for one session is negligible.

**Why [?] instead of inline expandable sections?**
The scene buffer rendering model (`put_text` into `Vec<Vec<SceneCell>>`) doesn't
support interactive expandable sections without significant complexity. A toggle-key
approach fits the existing interaction model (similar to how Stormglass uses keybinds
for phase transitions) and gives experienced players a clear escape hatch.

**Why show power percentage alongside raw numbers?**
The success forecast strings ("60-90% success", "Overpowered — 95%") are useful but
require players to mentally map the ratio to the forecast. Showing `(128%)` alongside
the threshold makes this mapping explicit and teaches the mechanic rather than hiding it.
