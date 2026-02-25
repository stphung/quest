# Deep UI Audit — Information Gaps, UX Issues, and Improvement Opportunities

**Date:** 2026-02-23
**Scope:** All Deep UI files (`deep_scene.rs`, `deep_missions.rs`, `deep_roster.rs`, `deep_layers.rs`, `deep_events.rs`, `deep_results.rs`, `deep_input.rs`) plus supporting game logic.

---

## 1. Executive Summary — Top 5 Most Impactful Issues

### Issue 1: No Mission Description Shown (High Impact)
Every `AvailableMission` carries a `description` field populated with thematic, context-rich text (e.g., "Survey the Shallows for intel and entry points."). Neither the compact nor the split-panel New Mission view renders this field. Players see type name, layer number, duration, and risk label — but no flavor or contextual explanation of what the mission actually does.

### Issue 2: Familiarity System Is Nearly Invisible (High Impact)
Familiarity (0–100% intel per layer) is one of the three main progression levers in The Deep — it reduces mission durations by up to 30%, categorizes as Unknown/Mapped/Familiar/Mastered, and unlocks better auto-resolve. The Hub and New Mission views show no familiarity indicator. Only the Layers detail panel shows it, as a raw `%` number and bar labeled "Intel" rather than "Familiarity." The named tiers (Unknown/Mapped/Familiar/Mastered) and their effects are never communicated to the player.

### Issue 3: Success Probability Formula Is Opaque (High Impact)
The power threshold system — which determines success chance — is central to all decision-making. The UI shows "Min Power N" and a success forecast string ("Good odds — 60-90% success") but never explains what drives the outcome. Players cannot see the actual power ratio (squad power / threshold as a percentage), and the thresholds shown in the mission pool were pre-calculated without squad modifiers. Once inside squad assignment, showing `Pwr: 32/25  Good odds — 60-90% success` is correct but misses the ratio context (128% of threshold = in the 60-90% band).

### Issue 4: Guild Rank Upgrade Path Is Completely Hidden (High Impact)
Guild Rank is the primary account-level progression mechanic, gating roster size and concurrent missions. The Hub header shows `Guild: Freelancers (Rank 1)` and nothing more. There is no display of: the current upgrade cost (`guild_upgrade_cost` field exists in `DeepPersistent`), the required breakthrough layer to advance, or any indicator that the player is progressing toward the next rank. The Roster view duplicates this summary without adding upgrade information.

### Issue 5: The Recruit View Has No Rendering (High Impact)
`DeepView::Recruit` appears in the tab bar and has a complete input handler (`handle_recruit` in `deep_input.rs`), but there is no rendering function for it in any `deep_*.rs` UI file. The tab dispatches to `render_roster()` from the scene coordinator:

```rust
DeepView::Roster | DeepView::Recruit => {
    super::deep_roster::render_roster(...)
}
```

Pressing the Recruit tab silently shows the Roster view instead. Recruit candidates (`prestige.recruit_pool.candidates`) with their costs (`prestige.recruit_pool.recruit_costs`) are never displayed.

---

## 2. Information Gaps — Data That Exists But Isn't Shown

### 2.1 Hub View

| Missing Information | Where It Lives | Impact |
|---|---|---|
| Concurrent mission cap | `guild_rank.concurrent_missions()` | Player doesn't know how many more missions they can launch |
| Total Warband Marks earned (all-time) | Not tracked, but current balance only shown in header | Minor context loss |
| Guild rank upgrade requirements | `guild_rank.next()`, `guild_upgrade_cost`, `required_breakthrough_layer` | No path to progression visible |
| Merc availability summary | `prestige.available_merc_count()` | Player must tab to Roster to understand bench depth |
| Mission ETA (wall clock time) | `mission.ends_at` | Shown as elapsed/total, not "completes at HH:MM" |

### 2.2 New Mission View

| Missing Information | Where It Lives | Impact |
|---|---|---|
| Mission description | `AvailableMission.description` | No flavor; missions feel identical except for type label |
| Power ratio as percentage | `squad_power / min_squad_power * 100` | Success band unclear without ratio context |
| Duration modifiers active on target layer | `layer_record.infrastructure`, `layer_record.familiarity` | Player shown duration already reduced, but doesn't see why |
| Construction target infrastructure detail | `MissionType::Construction(infra)` | Display name shows "Construction" but not what's being built in the list view |
| Event count for mission type | `mission_type.max_events()` | Breakthrough has up to 5 events; player doesn't know to expect interruptions |
| Archetype benefit explanation | Per-archetype effects are implicit | Scout/Arcanist recommended but no tooltip on *why* |

### 2.3 Roster View

| Missing Information | Where It Lives | Impact |
|---|---|---|
| XP-to-next-level progress | `Mercenary::missions_to_next_level(level)`, `merc.missions_completed` | Leveling feels random; no progress bar |
| Injury recovery countdown | `MercStatus::Injured { missions_remaining }` | Shows "Injured (2 missions)" but doesn't clarify what triggers recovery |
| Merc quality tier (Common/Uncommon/Rare/Elite) | Not stored on `Mercenary` struct | Cannot show quality label post-recruit |
| Recruit pool refresh timer | `recruit_pool.refreshed_at + 24h` | Player doesn't know when pool refreshes |
| Recruit candidate stats | `prestige.recruit_pool.candidates` | Entire Recruit tab missing rendering |
| Recruit costs | `prestige.recruit_pool.recruit_costs` | Cannot compare recruit value |

### 2.4 Layers View

| Missing Information | Where It Lives | Impact |
|---|---|---|
| Familiarity tier label | `FamiliarityLevel::from_familiarity(pct)` in `layers.rs` | Shows 42% but not "Mapped" |
| Duration reduction total | `layer_record.total_duration_reduction()` | Player cannot see combined modifier effect |
| Infrastructure build cost | `infrastructure_build_cost(infra, layer)` in `layers.rs` | No way to plan without knowing cost |
| Infrastructure build action | No build keybind in `handle_infrastructure` | Players can view but never build from the Layers view |
| Layer difficulty rating | `layer_record` tier + power threshold data | No sense of how hard the next push will be |
| Next guild rank breakthrough requirement | `guild_rank.next()?.required_breakthrough_layer()` | Cannot plan which layer to push toward |

### 2.5 Event Response View

| Missing Information | Where It Lives | Impact |
|---|---|---|
| Archetype bonus explanation | `event.archetype_bonus` field | Shows required archetype in `[TAG]` but `archetype_bonus` (a separate "improves outcome" archetype) is never displayed |
| Consequence magnitude | Time delta shown as "delay/faster" but no duration | Player cannot weigh `+3h delay` vs `safe` without numbers |
| Current mission progress bar | Progress is calculated but only shown as `%` in header | Visual progress bar would reinforce urgency |
| Choice outcome probabilities | Whether risky choices show partial odds | `is_risky` flag is used for consequence tag but risk % unclear |

### 2.6 Mission Results Modal

| Missing Information | Where It Lives | Impact |
|---|---|---|
| Familiarity gained | Not shown (applied in `resolve_mission`) | Player misses feedback loop |
| Exact injury recovery duration | `MercStatus::Injured { missions_remaining }` | Shows "injured" but not "back in 2 missions" |
| Marks balance after collection | `prestige.warband_marks` after resolve | No post-result balance shown |
| Layer cleared notification | Breakthrough clearing a layer is only detectable from Layers view | Major milestone has no celebration moment |

---

## 3. UX Issues — Confusing Flows and Unclear Feedback

### 3.1 Recruit Tab Shows Roster Instead of Recruit Candidates
The `DeepView::Recruit` tab exists in the tab bar and pressing Tab cycles to it, but `deep_scene.rs` dispatches it to `render_roster()`. Players pressing Tab to "Recruit" see the Roster view with no indication something went wrong. The input handler for Recruit (`handle_recruit`) correctly navigates the recruit pool, creating an invisible mismatch between state and rendering.

**File:** `src/ui/deep_scene.rs:196` — the `DeepView::Roster | DeepView::Recruit` branch.

### 3.2 Squad Assignment Focus State Is Not Visually Distinguished
In the split-panel New Mission view, focus shifts between the mission list (left) and squad assignment panel (right) when Enter is pressed (`staging_mission_index` toggles). The left panel cursor disappears (becomes `"  "`) to indicate the list is no longer focused, but the right panel doesn't gain a visual "active" indicator. A player skimming the screen cannot immediately identify which panel is interactive.

**File:** `src/ui/deep_missions.rs:458` — `is_sel` condition checks `staging_mission_index.is_none()` for the left panel cursor, but the right panel has no compensating border highlight or header color change.

### 3.3 "No Missions Available" Empty State Is Misleading
When `available_missions` is empty, the message reads "Complete active missions to refresh the pool." However, the pool refresh logic in `missions.rs` is tied to game ticks (not mission completion alone). On a fresh prestige, before any missions are queued, the player sees this message with no actionable path. The empty state should explain discovery state and whether the pool needs a specific trigger.

**File:** `src/ui/deep_missions.rs:303-318`

### 3.4 Construction Mission Type Not Shown in List
In the mission list, `MissionType::Construction(Infrastructure::Outpost)` renders `display_name()` as `"Construction"` — the infrastructure type being built is dropped. Players queuing up a Construction mission have no way to know it will build an Outpost vs. a Bridge until they read the detail panel.

**File:** `src/ui/deep_missions.rs:358` — the `format!` call for the compact list row uses `m.mission_type.display_name()` without a Construction payload branch.

**Suggested fix:** Add a variant: if `MissionType::Construction(infra)`, use `format!("Build {}", infra.display_name())`.

### 3.5 Event Auto-Resolve Timer Uses Inconsistent Countdown Units
The event auto-resolve countdown switches from minutes to seconds at the 5-minute mark (`remaining < 5 * 60`), which is a reasonable UX touch. However, the `AUTO_RESOLVE_SECS = 30 * 60` constant (30 minutes) is hardcoded in the UI file (`deep_events.rs:155`) rather than sourced from a game logic constant. If the logic timeout changes, the UI countdown becomes incorrect.

**File:** `src/ui/deep_events.rs:155`

### 3.6 Hub Enter Key Behavior Is Asymmetric
In the Hub view, `[Enter]` on a completed mission dismisses the results modal. But `[Enter]` on an active mission with a pending event navigates to the EventResponse view. For active missions without events, `[Enter]` silently does nothing. Players pressing Enter on active missions expect some feedback (e.g., mission detail, or a tooltip explaining no action is available).

**File:** `src/input/deep_input.rs:54-106`

### 3.7 Flash Message Positioning Conflict
The flash message in the New Mission view is rendered at `height - 2` (one row above the footer), but the content rendering also uses `content_bottom = height - 2`. Long mission lists can overwrite the flash message area. The message may be obscured by mission list rows that extend to `content_bottom`.

**File:** `src/ui/deep_missions.rs:292-295`

### 3.8 Roster View Compact: Status Color Column Offset Brittle
In compact roster view, the status column offset uses `line.rfind(status_label)` to find the color position — this is fragile because if the merc's name contains a substring matching the status label (e.g., a merc named "Ready" with status "Ready"), the `rfind` will find the wrong position.

**File:** `src/ui/deep_roster.rs:138` — `let status_col = line.rfind(status_label).map(...)`

---

## 4. Visual Issues — Hierarchy, Spacing, Color Usage

### 4.1 Tab Bar Labels Don't Communicate Current State
The tab bar shows `[Hub] [Missions] [Roster] [Layers] [Recruit]` with the active tab in `Color::Rgb(80, 160, 220)` (cyan) and others in `Color::DarkGray`. However, tabs don't carry any state indicators:
- Missions tab with a pending event shows no indicator (the Hub shows `⚡` but you have to navigate there first)
- Roster tab with injured mercs shows no indicator
- Recruit tab when pool is fresh shows no indicator

**Comparison:** The stats panel shows a `[D]` indicator in the main game HUD with state-dependent color (Cyan for running, Yellow for event, Green for done), but the Deep overlay's own tab bar conveys none of this information.

**File:** `src/ui/deep_scene.rs:119-138`

### 4.2 Hub Header Hierarchy: All Text Is Same Color
The Hub header uses `Color::White` for the guild/marks line and `Color::DarkGray` for the subheader. The most actionable piece of information (Warband Marks balance) is embedded in the middle of a white-on-dark string with no visual emphasis. By comparison, Haven's header uses distinct color bands per piece of data.

**File:** `src/ui/deep_missions.rs:117-133`

### 4.3 Layer Tier Colors Are Not Consistently Applied
`layer_tier_color()` in `deep_layers.rs` maps tiers to colors (Shallows=Green, Warrens=Yellow, Hollows=Magenta, SunkenReach=Cyan, Abyss=LightRed, Void=Gold). However, in the compact list view the tier color is computed and then suppressed with `let _ = tc;` (line 155), so the layer number column is rendered in `Color::White` instead of the tier color. The split view correctly applies the tier color to the layer number. This creates an inconsistency between compact and full renderings.

**File:** `src/ui/deep_layers.rs:155` — `let _ = tc;` suppresses intentional color.

### 4.4 Progress Bar in Hub Uses Mission Type Color for Both Fill and Border
The progress bar `render_progress_bar()` uses the mission type color (`tc`) for filled cells and `Color::Rgb(30, 40, 60)` for empty cells. This is correct behavior, but in compact mode (S tier) the bar is only 12 characters wide, making it nearly unreadable against the dark backdrop. The backdrop uses `bottom_rgb = (2, 3, 8)` which makes `Color::Rgb(30, 40, 60)` nearly invisible for the empty portion.

**File:** `src/ui/deep_missions.rs:246`

### 4.5 Event View Title Uses All-Caps Instead of Bold
The event title is rendered with `.to_uppercase()` as a substitute for bold styling:
```rust
put_text_centered(buffer, narrative_top + 1, width, &event.title.to_uppercase(), Color::White);
```
Other overlays (Soulforge, Haven) use `Modifier::BOLD` via Ratatui's `Paragraph` with styled spans for emphasis. The scene buffer approach used here can't apply text modifiers to individual cells in `put_text`, but the event view could use a dedicated Ratatui `Paragraph` widget for the narrative section.

**File:** `src/ui/deep_events.rs:118-124`

### 4.6 Mission Results Modal Is Always 56-Wide Regardless of Terminal Size
The results modal clamps to `56u16.min(area.width - 4)` with a fixed-height layout. On wide terminals (XL tier), it renders as a small centered box while the overlay behind it is full-screen. Haven's details panel and Soulforge's animation panels fill their allocated regions proportionally. The results modal would benefit from adaptive sizing or at minimum a wider cap on XL.

**File:** `src/ui/deep_results.rs:26`

### 4.7 Separation Lines Use Same Color in All Views
Inner panel dividers use `Color::Rgb(40, 60, 80)` uniformly across Hub, New Mission, Roster, and Layers. Haven uses `Color::Rgb(60, 72, 84)` for borders and varies the saturation between inner and outer elements. The uniform separator color makes the Deep UI feel visually flat compared to Haven's subtle layering.

---

## 5. Comparison with Other Overlays

### 5.1 Haven Overlay (Gold Standard)
Haven's implementation provides a strong reference for Deep's missing polish:

| Feature | Haven | The Deep |
|---|---|---|
| Room description (word-wrapped) | Yes, `word_wrap()` in `haven_tree.rs` | Mission description field exists but not rendered |
| Tier progression shown inline | Yes, T1-T4 with next-tier arrow marker | No level/progression shown for mercs or guild |
| Cost shown before action | Yes, PR cost per tier shown in detail panel | Infrastructure costs not shown in Layers view |
| Prestige rank check for affordability | Yes, `can_afford()` with color feedback | No affordability check shown for recruitment |
| Bordered inner panels with titles | Yes, `render_room_detail()` draws its own border | Deep detail panels are borderless text blocks |
| Achievement badge in header | Yes, `highest_haven_badge()` | No achievement integration |

### 5.2 Soulforge Overlay
Soulforge's animation and feedback loop are relevant to mission results:

| Feature | Soulforge | The Deep |
|---|---|---|
| Animated outcome display | Hammering/success/failure particle effects | Static text-only results modal |
| Clear success/failure feedback | Screen-filling effect with color | Color border only |
| Slot state indicators in summary | Enhancement level bars per slot | No persistent state summary |

### 5.3 Stormglass Overlay
Stormglass shows how to handle time-gated elements:

| Feature | Stormglass | The Deep |
|---|---|---|
| Countdown timer display | `Chrono Surge` speed ramp with timer | Mission ETA shown as elapsed/total only |
| Daily reset indicator | Daily rotation label | Recruit pool refresh timer not shown |
| Phase-aware layout | `ExchangePhase` drives rendering | `DeepView` drives rendering (similar pattern, well implemented) |

### 5.4 Overall Assessment
The Deep UI is functionally correct and architecturally consistent with other overlays (scene buffer, tab bar, split panels). The main gaps versus the mature Haven/Soulforge overlays are:
1. **Description/context text** — Haven shows room descriptions; Deep doesn't show mission descriptions
2. **Progression feedback** — Haven shows tier arrows and costs; Deep shows no upgrade paths
3. **State indicators on tabs** — Deep tabs are stateless; event/completion states require navigation to discover
4. **Empty Recruit view** — a tab that renders nothing

---

## 6. Recommended Improvements (Prioritized)

### P0 — Critical Bugs / Missing Features

**P0.1 — Implement Recruit View Rendering**
The Recruit tab dispatches to `render_roster()`. Create `render_recruit()` in `deep_roster.rs` that shows:
- Candidate list with name, archetype, stats (power/resilience/expertise), and cost
- Recruitment cost in Warband Marks with affordability color (Green/Red)
- Pool refresh countdown (`recruit_pool.refreshed_at + 24h - now`)
- Roster capacity indicator
- `[Enter] Recruit` action and flash message for insufficient marks or full roster

**P0.2 — Show Mission Description in Detail Panel**
In `render_new_mission_split()`, add `AvailableMission.description` as the first item in the detail panel after the layer/tier header. Use `word_wrap()` (import from `haven_tree.rs`) to wrap to `detail_inner_w`. This is zero-cost to compute — the data is already available.

**P0.3 — Fix Construction Mission Label**
Change the mission list display to show `"Build Outpost"` / `"Build Bridge"` etc. instead of `"Construction"`:
```rust
// In deep_missions.rs, replace display_name() call with:
let type_label = match m.mission_type {
    MissionType::Construction(infra) => format!("Build {}", infra.display_name()),
    other => other.display_name().to_string(),
};
```

### P1 — High Value / Low Complexity

**P1.1 — Add Familiarity Tier Label to Layers Detail**
In `render_layers_split()`, replace raw `"Intel:  42%"` with `"Familiarity: 42%  [Mapped]"` where the bracket label is colored per tier:
- Unknown (0-24%) → `Color::DarkGray`
- Mapped (25-49%) → `Color::Cyan`
- Familiar (50-74%) → `Color::Green`
- Mastered (75-100%) → `Color::Rgb(255, 215, 0)` (Gold)

**P1.2 — Show Guild Rank Upgrade Path in Hub**
Add a third header line in the Hub showing:
- Current concurrent mission count and cap: `Concurrent: 0/1`
- Next rank requirements: `Next Rank: Layer 3 Breakthrough  →  Rank 2 (Sellswords)`
Or fold into the existing subheader with color-coded upgrade info.

**P1.3 — Add State Indicators to Tab Bar**
After each tab label, append a state badge when relevant:
- `[Hub]` — show `●N` in Yellow when N events are pending, `✓N` in Green when N results await
- `[Missions]` — show `N` in Cyan for available mission count
- `[Roster]` — show `!N` in Yellow when N mercs are injured or lost
- `[Recruit]` — show `●` in Cyan when pool is fresh / has candidates

**P1.4 — Display Power Ratio as Percentage**
In the squad assignment summary, replace `"Power: 32/25"` with `"Power: 32/25  (128%)"`:
```rust
let ratio_pct = if min == 0 { 999 } else { squad_power * 100 / min };
let power_line = format!("Power: {}/{} ({}%)", squad_power, min, ratio_pct);
```
This directly communicates which success band the player is in.

**P1.5 — Show Infrastructure Build Costs in Layers View**
For unbuilt infrastructure slots in the detail panel, append the Warband Marks cost:
```rust
// In render_layers_split():
let cost = crate::deep::layers::infrastructure_build_cost(*infra, layer.index);
let cost_str = if built { String::new() } else { format!("  {}M", cost) };
put_text(buffer, row, col, &format!("{:12}  {}{}", name, desc, cost_str), color);
```

### P2 — Medium Value / Moderate Complexity

**P2.1 — Show Merc Leveling Progress in Roster**
In the detail panel, add an XP-style missions progress bar:
```
Level 3  [██████░░░░]  6/8 missions  →  Lv4
```
Where `missions_to_next_level(3) = 3 + 3*2 = 9`, and progress is `missions_completed % 9`.

**P2.2 — Show Mission ETA as Wall-Clock Time**
For active missions, add an ETA line: `"ETA: 14:32"` (local time when mission completes). This is especially valuable for multi-hour Breakthrough missions where players return to check progress.

**P2.3 — Show Familiarity Gained in Mission Results**
Add a line to the results modal: `"+ N% Familiarity on Layer X"` after the rewards section. The gain is deterministic from mission type (`familiarity_gain()` in `layers.rs`).

**P2.4 — Add Infrastructure Build Action to Layers View**
Currently the Layers view is read-only. Add `[B]` keybind to open a build sub-menu from the detail panel when the selected layer has buildable infrastructure slots and sufficient Marks.

**P2.5 — Fix Layer Tier Color in Compact Mode**
Remove `let _ = tc;` in `render_layers_compact()` (line 155) and apply `tc` to the layer number column, matching the split view behavior.

**P2.6 — Add Breakthrough Layer Cleared Celebration to Results Modal**
When `mission.mission_type == Breakthrough` and `result.outcome` is Success/PartialSuccess, add a prominent celebration line:
```
LAYER N CLEARED — New Depth Unlocked!
```
Colored in `Color::Rgb(255, 215, 0)` (Gold) centered in the modal.

### P3 — Polish / Low Priority

**P3.1 — Add Archetype Benefit Tooltips to Squad Assignment**
When a recommended archetype is present in the squad, highlight their name with the recommendation color and add a brief tooltip line explaining the benefit (e.g., "Scout reduces mission duration").

**P3.2 — Show Event Consequence Time Delta as Explicit Duration**
In the event choices, replace `"— delay"` / `"— faster"` with `"— +2h"` / `"— -1h"` using `format_hours(choice.time_delta_secs.abs() as u64)`.

**P3.3 — Extract Auto-Resolve Timer Constant to Game Logic**
Move `AUTO_RESOLVE_SECS = 30 * 60` from `deep_events.rs` to `src/deep/types.rs` or `missions.rs` so UI and logic agree on the timeout value.

**P3.4 — Fix Roster Compact Status Color Offset**
Replace `line.rfind(status_label)` with a fixed column offset calculated from the format string field widths to avoid fragile string searching.

**P3.5 — Adaptive Results Modal Width**
Change the modal width cap from a fixed 56 to a responsive value based on `ctx.tier`:
```rust
let modal_width = match ctx.tier {
    SizeTier::S => 50u16,
    SizeTier::M => 60u16,
    _ => 72u16,
}.min(area.width.saturating_sub(4));
```

---

## Appendix A — File-Level Summary

| File | Status | Key Issues |
|---|---|---|
| `deep_scene.rs` | Functional, solid | Tab bar lacks state indicators; opening animation is good |
| `deep_missions.rs` | Functional, incomplete | Missing: description rendering, construction label, power ratio % |
| `deep_roster.rs` | Functional, missing Recruit | No `render_recruit()` function; status offset fragile |
| `deep_layers.rs` | Functional, read-only | Missing: familiarity tier label, build costs, build action, tier color in compact |
| `deep_events.rs` | Functional | Auto-resolve const hardcoded; consequence magnitudes implicit |
| `deep_results.rs` | Functional | Missing: familiarity gain, layer cleared celebration, adaptive sizing |
| `deep_input.rs` | Correct | Hub Enter on non-event missions is a no-op with no feedback |
