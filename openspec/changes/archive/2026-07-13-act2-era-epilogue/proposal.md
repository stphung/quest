# Act 2 Era Epilogue — an Authored Ending for the Ferry Era

## Why

The Last Crossing is the ferry era's terminal state, and today it dead-ends: the arrival that empties the world gets one one-line modal, then the player is left on a Dock screen that renders a 0%-charged Riftglass bar with a "jump now" cost preview whose `[J]` silently does nothing (`main.rs` guards the jump on `era_over()`, but the view doesn't know). This is the last true release blocker from the 2026-07-12 readiness review: the ending must feel *authored* — "the story ended," not "the game broke" — before `ACT2_ENABLED` can flip.

## What Changes

- **A multi-beat era-end epilogue** replaces the single "The last crossing" `SceneModal`: a new one-shot `ColonyState::take_era_end_playback()` (mirroring `VoyageState::take_finale_playback()`) builds a state-conditioned `ScenePlayback` — souls delivered, souls the dark took, crossings sailed, era length, the six districts standing, the Sister Verity and the door ajar (the Act 3 hook, teased not opened). A persisted `era_end_shown: bool` (`serde(default)`) makes it play exactly once and survive reload — including for a player who quit the moment the era ended.
- **The post-era Dock view becomes a quiet harbor**: when `colony.era_over()`, the charge bar and jump preview are replaced with an authored resting state (there is no one left to carry; the rift is quiet). Fixes the misleading no-op jump preview.
- **The Record view gains a permanent era-complete summary** when the era is over — crossings, delivered, taken by the dark, days at sea, districts — so the arrived-harbor rooms (Manifest/Keepsake/Record) serve as the lasting post-era state the spec already promises.
- **The Last Crossing finally gets automated coverage**: lib tests for the one-shot playback (fires once, persists, numbers conditioned on colony state) plus overlay snapshots of the era-over Dock and Record via a new `fixtures::colony_era_complete()`.

## Non-goals

- **No Act 3 content** — `last_crossing_complete` and `vessel_arrived` remain unconsumed gates; the epilogue *references* the door ajar, it does not open it.
- **No title-screen or out-of-voyage surface** for the era record (held-for-later dossier question, unchanged).
- **No balance changes** — no constants move; the era's pacing is untouched.
- **Not flipping `ACT2_ENABLED`.**

## Balance/progression impact

None. Content, one persisted boolean, and render changes only. The era-end *trigger* (`era_over()`, `last_crossing_complete`) is untouched — only what the player sees after it fires.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `vessel-act2`: **The Last Crossing Ends The Era** is strengthened — the era end SHALL play a multi-beat authored epilogue exactly once (state-conditioned, reload-safe), the Dock view SHALL show a quiet-harbor resting state instead of charge/jump affordances, and the Record view SHALL carry a permanent era summary.

## Impact

- **Code**: `src/vessel/colony.rs` (`era_end_shown` field, `take_era_end_playback()`), `src/vessel/scenes.rs` (epilogue beat content), `src/main.rs` (era_over branch plays the epilogue via `scene_play`; clear-screen check for the reload case), `src/ui/voyage_scene.rs` (Dock era-over branch, Record era summary), `src/fixtures.rs` (`colony_era_complete()`).
- **Tests**: colony unit tests + `tests/ferryman_tests.rs` (playback one-shot/persistence), `src/ui/overlay_snapshot_tests.rs` (era-over Dock + Record snapshots).
- **Specs**: `openspec/specs/vessel-act2/spec.md` via delta (MODIFIED Last Crossing requirement).
- **Persistence**: one new `ColonyState` field with `serde(default)` — old colony saves load unchanged (corpus fixture unaffected; it stores a mid-era colony).
