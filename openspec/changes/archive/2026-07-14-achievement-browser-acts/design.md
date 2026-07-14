# Design — Achievement Browser Act Sections

## Context

Option A from the 2026-07-14 design review: a top act row (Act I · The Ascent / Act II · The Crossing), each act showing only its own subsection tabs. The browser today is a flat 9-category tab row (`AchievementCategory::ALL`) with `cycle_category` wrapping across all of them. Verification rows: achievements tests, `cargo test input::replay_tests`, browser rendering via drive-game screenshot; full `make check`.

## Goals / Non-Goals

**Goals:** dedicated act sections with per-act subsections and summaries; Act 2's set gets room to grow (#733); teaser ruling intact while dark.

**Non-Goals:** unlock logic, persistence, points; other option variants.

## Decisions

### D1 — Acts are groupings OF categories, not a parallel structure

`AchievementCategory` gains `Voyage`/`Ferry`/`Era` (12 total); `Act::categories()` returns each act's ordered slice (Act I: the original nine incl. Stats; Act II: the three new). `category.act()` is derived. The seven Vessel defs change category. Everything downstream (`count_by_category`, `get_achievements_by_category`, notifications) works unchanged — no new counting machinery. Alternative rejected: separate act/section metadata beside categories — two parallel taxonomies to keep in sync.

### D2 — Stats stays in Act I's row

The Stats tab is an account-wide special view with no achievements of its own. Rendering it in both act rows doubles special-casing for no content; it stays the last tab of Act I. Revisit if an Act 2 stats view ever exists.

### D3 — Navigation: `[Tab]` toggles act; cycling is act-scoped

`AchievementBrowserState` gains `selected_act`; `toggle_act()` selects the other act's first subsection and resets the list index; `cycle_category` wraps within `selected_act.categories()`. `[Tab]` is free inside the browser (it opens the challenge menu only on the base screen, step 8 of the dispatch chain — the browser intercepts at step 0.5). Help line gains `[Tab] Act`.

### D4 — Act row rendering + the dark teaser

A new act header row above the subsection tabs: each act label with `(unlocked/total)` summed over its categories and its points. Selected act highlighted (bold gold, ▶ marker); Act II's label renders DarkGray while `!act2_enabled()` (rows remain browsable — the 2026-07-13 visible-but-unearnable ruling). The tabs row beneath lists only the selected act's categories, so the strip stays short at 60×24.

## Risks / Trade-offs

- **[Category enum is matched exhaustively somewhere unknown]** → compiler finds every site (`name()`, tabs, any match); this is the point of doing it as enum variants.
- **[Tests pin Progression counts]** → the visibility test and any count tests update to the new subsections; the count math itself is unchanged.
- **[Muscle memory: </> used to reach Stats from Combat in one left-wrap]** → wrapping now stays inside Act I for those tabs; acceptable, and Tab is one keystroke.

## Migration Plan

Static-def and UI change only; no saved state involved (`selected_title` is id-based; browser state is transient). No corpus impact.

## Open Questions

- None blocking.
