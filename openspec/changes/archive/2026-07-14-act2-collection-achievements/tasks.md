# Tasks — Act 2 Collection Achievements

## 1. Achievement definitions and aggregates

- [x] 1.1 Add 7 `AchievementId` variants (EveryStarAHarbor, CompanyOnTheRoad, EarToTheWater, ThreeDoorsOpened, TheFullTable, HeavyLading, TheSwiftPassage); bump `VARIANT_COUNT` 247 → 254; add `waypoints_docked_mask: u64` and `pilgrims_hailed_mask: u8` (`serde(default)`) to `Achievements` (src/achievements/types.rs)
- [x] 1.2 Add 7 `AchievementDef`s in their Act II categories with points per design D6, doc-commented threshold constants `HEAVY_LADING_SOULS = 2_500` / `SWIFT_PASSAGE_DAYS = 8`, and the Wayfarer title mapping for Every Star a Harbor (src/achievements/data.rs)
- [x] 1.3 Update count/sanity tests that pin totals and category slices (types.rs partition test, data.rs sanity slices, any 247 literals)

## 2. Handlers and wiring

- [x] 2.1 Implement `on_voyage_observed(visited_mask, hailed_mask, rumors_known, refits_taken)` — union masks, evaluate the four collection unlocks against vessel constants (`WAYPOINTS.len()`, `PILGRIMS.len()`, `RUMORS.len()`, `REFIT_PAIRS.len()`) (src/achievements/handlers.rs)
- [x] 2.2 Implement `on_landfall(carried, days, aboard)` — evaluate Heavy Lading / The Swift Passage / The Full Table (threshold constants + `souls::CREW`) (src/achievements/handlers.rs)
- [x] 2.3 Call `on_voyage_observed` each voyage frame (mask conversion at call site) and `on_landfall` in the delivery block alongside `on_crossing_delivered` (src/main.rs voyage branch)

## 3. Tests

- [x] 3.1 Handler unit tests: union accumulation across simulated crossing resets, idempotent re-observation, each unlock fires once at its exact target, `WAYPOINTS.len() <= 64` mask-width guard (src/achievements/handlers.rs)
- [x] 3.2 Extend `vessel_visibility_tests` — the seven new rows are listed-but-locked while dark; total_count still equals `VARIANT_COUNT`
- [x] 3.3 Threshold feasibility pins in the era harness: balanced era satisfies `most_carried >= HEAVY_LADING_SOULS` and `fastest_days < SWIFT_PASSAGE_DAYS`; formula floor `BASE_CAPACITY * CAP_GROWTH^7 >= HEAVY_LADING_SOULS` (tests/ferryman_tests.rs)
- [x] 3.4 End-to-end coverage: `the_collection_observers_read_a_real_crossing` in tests/ferryman_tests.rs plays a real maiden voyage and feeds the observers exactly as `main.rs` does (The Full Table unlocks; Swift/Heavy correctly do not). Landed beside the era harness rather than the flag-on suite: the handlers are not themselves act2-gated — the voyage loop that calls them is
- [x] 3.5 Save-compat: corpus loads land the new serde-default fields at 0 (extend vessel corpus assertions if the corpus carries achievements)

## 4. UI and docs

- [x] 4.1 Re-bless affected snapshots (act row 0/7 → 0/14, max-score header) after reviewing diffs
- [x] 4.2 Update `src/achievements/CLAUDE.md` (count, aggregates, new handlers) and `src/vessel/CLAUDE.md` (observation call sites)

## 5. Verification

- [x] 5.1 Targeted: `cargo test --test ferryman_tests --test act2_flag_on_tests --test save_compat_tests`, achievements unit tests, `QUEST_ACT2=1 cargo test flag_on`, `cargo test snapshot`
- [x] 5.2 `make check` end-to-end
