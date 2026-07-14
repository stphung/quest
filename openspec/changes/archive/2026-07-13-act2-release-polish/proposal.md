# Act 2 Release Polish — Integration + Verification Hardening

## Why

Act 2 is mechanically complete (hardening #731, retune #731, epilogue #732) but still sealed off from the game's meta-systems — achievements don't know it exists, the Time Vault has no record of its three biggest moments — and its verification has known blind spots (no small-terminal snapshots for most voyage panels, no fuzz coverage of voyage input, an unexercised Time Vault-restore-across-the-act-boundary path). This is Phase 1 of the release plan (issue #734): after it, Act 2 is finished *as designed today* and the flip gate is exercisable. `ACT2_ENABLED` stays off.

Per direction (2026-07-13): **no player-facing wiki page for now** (dropped from this phase), and the **ward-lean branch is accepted as-is** (~7.2-month slow branch — documented as a resolved decision, no balance change).

## What Changes

- **Vessel achievements** (6): launch (The Burn), first arrival (The Roots of Light), souls-delivered tiers Ferryman I/II/III (1,000 / 10,000 / 50,000), era completion (The Last Crossing), and the covenant badge (complete the era with no soul lost). New `AchievementId` variants + defs + `on_*` handlers called from the existing vessel wiring points; a new `total_souls_delivered` aggregate counter.
- **Time Vault `SaveEvent` variants** (3): `VesselLaunched`, `VesselArrived`, `LastCrossing`. The launch's input result upgrades `NeedsSave` → `NeedsSaveWithEvent(VesselLaunched)`; arrival and era-end commit directly via `main_helpers::persistence::commit_save` from the voyage branch (which routes around `route_game_input`).
- **Chapter-gateway beats**: the 4 chapter-ending gateway arrivals (currently indistinct — `CHAPTER_GATEWAYS`/`is_chapter_gateway` are dead code) each gain one authored ceremony beat appended to their arrival scene, marking the act's weekly rhythm.
- **Small-terminal snapshots**: 60×24 overlay snapshots for the voyage views that only exist at 160×45 (junction, trim, souls, watch, reckoning, dock, manifest, keepsake, record) — fixing any layout breakage they reveal.
- **Voyage input fuzzing**: extend `fuzz_tests.rs`-style no-panic coverage to `handle_voyage_input` + `render_voyage` across voyage states (mid-leg, junction, arrived, era-over).
- **Time Vault ↔ Act 2 account files**: a test pinning the restore-across-the-act-boundary behavior — a pre-launch character save loaded while `voyage.json`/`colony.json` exist for the same character id must be benign (voyage not entered while `vessel_launched` is false; files intact; consistent with how Deep/Loom/Haven account files already survive restores).
- **Offline-return + first-boot assessment** (verify, fix only if broken): long-absence return mid-crossing, mid-Ignition quit → relaunch, and the already-qualified veteran's first boot (`QUEST_ACT2=1` fixtures via drive-game).
- **Ward decision documented**: `docs/decisions.md` entry + dossier note — ~7.2-month ward-lean accepted as the intended slow branch.

## Non-goals

- **No player-facing wiki page** (deferred by direction).
- **No balance changes** — the ward branch is documented, not retuned; no constants move.
- **No new mechanics** — gateway beats are authored content into existing scaffolding; extension work is Phase 2 (#733).
- **Not flipping `ACT2_ENABLED`.**

## Balance/progression impact

None. Achievements/events observe existing state transitions; content and tests only.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `vessel-act2`: (1) launch, first arrival, and the Last Crossing SHALL be recorded as Time Vault save events; (2) chapter-gateway arrivals SHALL play an authored chapter-close beat; (3) the Act 2 account files SHALL survive a character-save restore from before the launch (keyed-by-character behavior stated for the restore case).
- `achievements`: Vessel/ferry-era achievements added as requirements (6 achievements with exact thresholds).

## Impact

- **Code**: `src/achievements/` (ids, defs, handlers, milestones), `src/history/types.rs` (SaveEvent variants), `src/input/mod.rs` (launch event), `src/main.rs` (arrival/era-end commits + achievement calls), `src/vessel/scenes.rs` + `route.rs` (gateway beats; dead-code attrs removed), `src/vessel/CLAUDE.md`, `docs/decisions.md`, dossier.
- **Tests**: achievements unit tests, history/save-event tests, gateway-beat tests, new 60×24 overlay snapshots, voyage fuzz tests, Time Vault interplay test in `tests/`.
