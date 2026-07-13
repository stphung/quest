# Design — Act 2 Release Polish

## Context

Three merged changes made Act 2 complete and safe; this one makes it *integrated* and closes the verification blind spots, so Phase 1 of issue #734 ends with the flip gate exercisable. Verification rows (root CLAUDE.md): achievements tests, `cargo test overlay_snapshot`, `cargo test input::replay_tests` / fuzz, `tests/save_compat_tests` + vessel tests, `QUEST_ACT2=1 cargo test flag_on`, full `make check`, plus drive-game passes for the assessment items.

## Goals / Non-Goals

**Goals:** Act 2 visible to achievements and the Time Vault; the act's weekly rhythm (chapter gateways) authored; the known verification gaps (small terminals, voyage fuzz, restore interplay) closed; the two open assessments (offline-return, veteran first boot) performed and either passed or fixed.

**Non-Goals:** wiki page (deferred by direction); any balance change (ward accepted as-is, documented); new mechanics (Phase 2, #733); flipping the flag.

## Decisions

### D1 — Achievements follow the existing handler pattern; a new `Vessel` category is NOT added

Six achievements: `TheBurn` (launch), `TheRootsOfLight` (first arrival), `FerrymanI/II/III` (1,000 / 10,000 / 50,000 souls delivered, driven by a new `total_souls_delivered` aggregate + milestone array, mirroring `SLAYER`-style tiers), `TheLastCrossing` (era complete), `TheCovenantKept` (era complete with zero souls lost across every crossing — trackable because `SoulStatus::Lost` is authored-only and crossing-scoped; the colony already knows delivered vs. taken, but the covenant is about *crew*, so it keys off a new small `souls_lost_lifetime` counter incremented where `mark_lost` lands). Handlers: `on_vessel_launched`, `on_vessel_arrived`, `on_souls_delivered(total)`, `on_last_crossing(covenant_kept)`. They are called from the same wiring points the state transitions already flow through: the launch confirm in `input/mod.rs`, and the delivery/era-end block in `main.rs`'s voyage branch (both have `global_achievements` in scope). Category: existing `Progression` (the browser's category list is a 9-variant UI surface; a tenth category for six achievements is disproportionate — revisit if Phase 2 grows the set). Points follow the existing tier system: launch/arrival Hard (50), Ferryman tiers Medium/Hard/Very Hard (25/50/100), Last Crossing Elite (250), Covenant Elite (250).

### D2 — SaveEvents: one via the input path, two via direct commit

`SaveEvent::{VesselLaunched, VesselArrived, LastCrossing}` with `description()` lines ("The Vessel launches — 250,000 Prestige Ranks burn", "The Vessel reaches the Tree", "The Last Crossing — the old world is empty"). The launch flows the normal way: `handle_vessel_overlay`'s successful burn returns `NeedsSaveWithEvent(VesselLaunched)` (replacing `NeedsSave` — replay tests updated to pin the new variant, per the input module's "assert the InputResult" rule). Arrival and era-end happen inside `main.rs`'s voyage branch, which bypasses `route_game_input` — they call `main_helpers::persistence::commit_save(state, &event, history_repo)` directly after the existing `save_files` call, matching the deferred-commit semantics. Alternative rejected: threading a `VoyageInputResult`-style event queue back through the loop — more machinery for the same two call sites.

### D3 — Gateway beats: authored content appended at scene-build time, dead code becomes live

`scenes.rs` gains four authored chapter-close beats (one per gateway: the Shallows' gate, the Drift Roads' gate, the Starless Deep's gate, and the Threshold into the Roots). The beat is appended to the gateway waypoint's `ScenePlayback` where arrival scenes are assembled, keyed by `route::is_chapter_gateway` — which loses its `#[allow(dead_code)]` along with `CHAPTER_GATEWAYS`. Content tone: a chapter closing behind the ship, felt at the helm (each beat names what the chapter *was* and what the water ahead stops being). A content-parity test asserts every maximal route sees exactly the gateway beats of the chapters it crossed.

### D4 — Small-terminal snapshots may reveal breakage; fixing it is in scope, re-blessing around it is not

Nine views get 60×24 snapshots (junction, trim, souls, watch, reckoning, dock, manifest, keepsake, record) via the existing strip/full dispatch. Any panic, overflow, or unreadable collapse found is a bug to fix in the render fns (width-aware truncation, the same patterns the chart strip already uses). Snapshot review rule applies: diffs are read, not blessed blind.

### D5 — Voyage fuzz mirrors the Act 1 fuzz harness's shape

A `fuzz_voyage_input_never_panics` test drives `handle_voyage_input` + `render_voyage` with hundreds of weighted-random keys per seed across four starting states (mid-leg, holding at a junction, arrived/harbor, era-over with colony). Excluded keys: none needed (voyage input has no browser/clipboard side effects); `Quit` results are ignored rather than honored so the loop keeps running. Panics print the seed, same as `fuzz_tests.rs`.

### D6 — Restore interplay: the vault rewinds the whole timeline, and the test pins it

Investigation overturned the initial assumption: the Time Vault repo is the **entire quest dir** (`HistoryRepo` stages `["*"]` and `restore_to` hard-resets), so `voyage.json` and `colony.json` are committed in every vault snapshot and rewound by every restore — exactly like Deep/Loom/Haven files, which live in the same repo. The coherent semantic that actually exists: **a restore rewinds the hero *and* the era together** — restoring to a pre-launch commit removes the in-progress crossing files, and a later launch begins a fresh crossing; restoring forward (the commit ids survive in the reflog) brings them back byte-identical. This is *better* than the initially-designed "rewind the hero, not the era" (which would desync a rewound hero from a months-later colony). The test pins both directions on a temp `HistoryRepo`. The keyed-by-`character_id` load behavior remains what protects the *non-vault* path (switching characters never inherits a crossing) and is already specced.

### D7 — Assessments are drive-game passes with written findings, fixes only on failure

Offline-return (long absence mid-crossing; mid-Ignition quit → relaunch) and the veteran first boot (fixture with signal + full prerequisites, watch modal→whisper→[V]→burn cadence) are run against real builds with `QUEST_ACT2=1`. Findings land in the change's tasks and #734; anything broken becomes a fix task here, anything merely debatable is recorded, not redesigned.

## Risks / Trade-offs

- **[Achievement retroactivity]** → veterans who launch after the flip get achievements naturally; nobody can have launched while dark, so no retroactive sync pass is needed. The souls tiers use the colony's lifetime counter, so a mid-era update credits correctly on the next delivery.
- **[`NeedsSave` → `NeedsSaveWithEvent` changes launch save behavior]** → it *adds* a history commit to an existing save; the replay tests pin the variant so a silent regression can't skip the save.
- **[60×24 may genuinely not fit some panels]** → the strip tier already exists for the chart; if a panel can't be made legible at 60×24, the fix is a designed compact variant or an explicit "terminal too small" notice — decided per panel during implementation, recorded in tasks.

## Migration Plan

Additive: new achievement ids (old `achievements.json` loads them as absent), new SaveEvent variants (serialization is forward-only), one new aggregate counter with `serde(default)`. No save migration. Corpus untouched.

## Open Questions

- None blocking. Ward decision is recorded via this change (docs/decisions.md), closing #734 item 1c-3.
