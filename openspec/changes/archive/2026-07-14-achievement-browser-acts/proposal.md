# Achievement Browser: Act Sections (Option A)

## Why

The seven Vessel achievements landed at the tail of the Progression tab — functional, but Act 2 deserves its own home, and Phase 2 content (#733) will keep growing the set. Per direction (2026-07-14, "Option A"): the browser gains a top-level act selector — **Act I · The Ascent** and **Act II · The Crossing** — each act showing only its own subsection tabs, with per-act score summaries.

## What Changes

- **`Act` enum** (`ActI`/`ActII`) with display names and per-act category lists; **`AchievementCategory` grows from 9 to 12** — three new Act II subsections: **The Voyage** (The Burn, The Roots of Light), **The Ferry** (Ferryman I–III), **The Era** (The Last Crossing, The Covenant Kept). The seven Vessel achievements move from Progression into these. Stats stays in Act I's row (account-wide view, unchanged content).
- **Browser navigation**: a new act row above the category tabs; `[Tab]` toggles act (resetting to that act's first subsection), `</>`/arrows cycle subsections *within* the selected act, list/details unchanged. The act row shows each act's unlocked/total and points.
- **Teaser ruling preserved**: Act II's row and rows are visible (locked) while dark, per the 2026-07-13 decision — the act label renders dimmed until `act2_enabled()`.

## Non-goals

- No changes to unlock logic, points, save events, or the achievements' persistence — categories live on static defs only.
- No landing page (Option C) or in-list headers (Option D).
- Not flipping `ACT2_ENABLED`.

## Balance/progression impact

None — presentation and category mapping only.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `achievements`: the Categories/Count/Scoring requirement's nine categories become twelve grouped under two acts with act-aware browser navigation; the Vessel Act Milestones requirement's category changes from Progression to the three Act II subsections.

## Impact

- **Code**: `src/achievements/types.rs` (Act enum, category variants, act mapping), `src/achievements/data.rs` (vessel defs' categories), `src/ui/achievement_browser_scene.rs` + `achievement_tabs.rs` (act row, act-scoped cycling), `src/input/mod.rs` (Tab in browser).
- **Tests**: act-partition unit tests, browser cycling/replay tests, updated visibility + count tests, drive-game screenshot.
