# Tasks — Act 2 Release Polish

## 1. Time Vault save events (smallest, unblocks the launch replay-test update)

- [x] 1.1 Add `SaveEvent::{VesselLaunched, VesselArrived, LastCrossing}` + `description()` lines (`src/history/types.rs`)
- [x] 1.2 Launch: `handle_vessel_overlay` returns `NeedsSaveWithEvent(VesselLaunched)` on a successful burn; update the flag-on replay test to pin the new variant
- [x] 1.3 Arrival + era end: call `main_helpers::persistence::commit_save` from the voyage branch after their `save_files` (first arrival only for `VesselArrived`; era-end once via the epilogue take or `last_crossing_complete` transition)
- [x] 1.4 History tests for the three descriptions + a commit-once assertion where feasible

## 2. Vessel achievements

- [x] 2.1 `AchievementId` variants + `ALL_ACHIEVEMENTS` defs (Progression category, tiered points per design D1): TheBurn, TheRootsOfLight, FerrymanI/II/III, TheLastCrossing, TheCovenantKept
- [x] 2.2 Aggregates: `total_souls_delivered` (+ milestone array), `souls_lost_lifetime` (incremented at the `mark_lost` landing point), both `serde(default)`
- [x] 2.3 Handlers: `on_vessel_launched`, `on_vessel_arrived`, `on_souls_delivered(total)`, `on_last_crossing(covenant_kept)`; wire from launch confirm (`input/mod.rs`) and the voyage delivery/era-end block (`main.rs`)
- [x] 2.4 Unit tests: tier crossings, covenant true/false, one-time unlock, serde defaults on old `achievements.json`

## 3. Chapter-gateway beats

- [x] 3.1 Author the four chapter-close beats in `scenes.rs`; append at scene assembly for gateway waypoints; un-dead-code `CHAPTER_GATEWAYS`/`is_chapter_gateway`
- [x] 3.2 Content-parity test: every maximal route sees exactly its crossed chapters' gateway beats; update any affected scene/overlay snapshots (review diffs)

## 4. Verification hardening

- [x] 4.1 60×24 overlay snapshots for junction, trim, souls, watch, reckoning, dock, manifest, keepsake, record — fix any layout breakage found (compact variant or explicit too-small notice per design D4)
- [x] 4.2 Voyage input fuzz: `fuzz_voyage_input_never_panics` across mid-leg / junction / arrived / era-over states, seed-printed
- [x] 4.3 Time Vault interplay test (design D6): pre-launch restore with live `voyage.json`/`colony.json` — voyage not entered, files intact, re-launch resumes the crossing; document the semantic in `src/vessel/CLAUDE.md` gotchas

## 5. Assessments (drive-game, fix only if broken)

- [x] 5.1 Offline-return: long absence mid-crossing; mid-Ignition quit → relaunch — record findings
- [x] 5.2 Veteran first boot (`QUEST_ACT2=1`, fully-qualified fixture): modal → whispers → `[V]` → burn cadence — record findings

## 6. Decisions + docs

- [x] 6.1 `docs/decisions.md`: ward-lean branch accepted as-is (~7.2 mo / ~93%, intended slow branch); dossier note; resolves #734 1c-3
- [x] 6.2 Record in #734: wiki page deferred by direction (1c-1 closed as "not now")
- [x] 6.3 `src/vessel/CLAUDE.md`: save events, achievements wiring, gateway beats, restore semantic

## 7. Verification

- [x] 7.1 Targeted: achievements tests, `cargo test --test vessel_launch_gate_test` + flag-on subset (`QUEST_ACT2=1 cargo test flag_on`), `cargo test overlay_snapshot`, fuzz run, `tests/save_compat_tests`
- [x] 7.2 `make check`
- [x] 7.3 Archive change; `openspec validate --specs`; push; draft PR
