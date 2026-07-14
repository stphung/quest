# Tasks — Achievement Browser Act Sections

## 1. Data model

- [x] 1.1 `Act` enum (`ActI`/`ActII`) + `name()`; `AchievementCategory` gains `Voyage`/`Ferry`/`Era` (ALL → 12); `Act::categories()` and `AchievementCategory::act()`; partition unit test (every category in exactly one act, in order)
- [x] 1.2 Move the seven Vessel defs to their subsections (Burn/Roots → Voyage; Ferryman I–III → Ferry; LastCrossing/Covenant → Era); update the visibility + count tests that referenced Progression

## 2. Browser UI + input

- [x] 2.1 `AchievementBrowserState`: `selected_act`, `toggle_act()`, act-scoped `cycle_category`
- [x] 2.2 Act header row (per-act unlocked/total + points, ▶ highlight, Act II dimmed while dark) above act-scoped subsection tabs; help line gains `[Tab] Act`
- [x] 2.3 `[Tab]` toggles act in the browser input handler; replay tests: Tab toggles act, cycling wraps within act, list index resets

## 3. Verification + docs

- [x] 3.1 Suites: achievements tests, replay tests, `QUEST_ACT2=1 cargo test flag_on`, snapshot suites (re-bless any browser-adjacent diffs after review)
- [x] 3.2 Drive-game screenshots: Act I row and Act II row (dark), for the PR
- [x] 3.3 `src/achievements/CLAUDE.md` (12 categories, act grouping, navigation); archive change; `openspec validate --specs`; `make check`; push to the PR #740 branch and update its description
