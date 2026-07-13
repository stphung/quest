# Design — Act 2 Era Epilogue

## Context

The era-end path today (`main.rs`, inside the `take_finale_playback()` delivery branch): `col.era_over()` → set `last_crossing_complete`, push one `SceneModal` ("The last crossing"), skip the dock. Three problems: the ending is one line where every other act beat is a multi-beat scene; a player who quits at that exact frame never sees even the line (the modal queue is transient); and the post-era Dock view still renders charge/jump affordances that silently no-op. The finale machinery (`take_finale_playback()` → `ScenePlayback` → `ScenePlay`) is the proven pattern for "authored, one-shot, save-derived scene" — the epilogue mirrors it at the colony level.

Verification rows (root CLAUDE.md): vessel tests (`ferryman_tests`), `cargo test overlay_snapshot` for the new Dock/Record states, full `make check`. The experiential intent, for a future designer: the epilogue is the era's *Reckoning inverted* — the numbers the player has been spending against all era (delivered, waiting, the daily toll) read back one final time as an account settled; it should land as earned quiet, not fanfare. Sister Verity and the lamp-lit door close the act the way `FINALE_HARBOR`/`FINALE_LAMP` closed the maiden crossing — the same two images, now with the door ajar. Cross-act narrative context lives in `docs/dossiers/world-and-narrative.md`; this scene must stay consistent with it (rebirth cycles, the ferryman's rest).

## Goals / Non-Goals

**Goals:** the era end plays a multi-beat, state-conditioned epilogue exactly once, reload-safe; the post-era Dock reads as an authored resting state; the Record view holds the era's account permanently; the Last Crossing requirement gets automated coverage.

**Non-Goals:** Act 3 content (the door stays ajar, unopened); any surface outside the voyage screens; balance/constants; changing when or how the era ends.

## Decisions

### D1 — The playback lives on `ColonyState`, not in `main.rs`

`take_era_end_playback()` is a method on `ColonyState` guarded by a new persisted `era_end_shown: bool` (`serde(default)`, so every existing colony save loads with it false — harmless, since the take also requires `era_over()`). Rationale: (a) lib-testable — the one-shot and its conditioned text can be asserted without the binary crate, unlike today's inline modal (this is what gives the spec's Last Crossing scenarios their missing test); (b) reload-safe by construction — the flag is in `colony.json`, so an interrupted era end replays on next boot, exactly like `finale_shown` does for the arrival finale. Alternative rejected: a transient `GameState` flag — wrong home (the era is colony state) and it would double-play after prestige-independent reloads.

### D2 — Triggered from the clear-screen drain, not only the delivery branch

`main.rs` currently plays queued content when the screen is clear (`scene_play.is_none() && scene_modal.is_none()` drains `moments`/`unread_scenes`). The epilogue check joins that drain: when clear, and `v.arrived()`, and the colony says `era_over() && !era_end_shown`, take and play it. The delivery branch keeps setting `last_crossing_complete` and stops pushing the old modal. Rationale: the delivery branch already sets `scene_play` to the *arrival finale* in the same frame — queueing the epilogue there would clobber or race it; the drain ordering gives finale → landfall/district/milestone moments → epilogue for free, and the same check catches the quit-at-era-end reload with zero extra code. Gate on `v.arrived()` so a freshly-loaded mid-crossing save (era mathematically over but ship still sailing — impossible today, cheap to guard) can't fire it early.

### D3 — Content: authored beats in `scenes.rs`, numbers formatted from colony state

`scenes.rs` gains the epilogue content beside the finale constants: fixed opening/closing beats plus builder fns for the conditioned middle (mirroring `finale_carved_beat`). The account beat reads the settled numbers: `souls_delivered` carried home, `INITIAL_SOULS − souls_delivered` taken by the dark, `crossings_completed` crossings, total days at sea (`CrossingRecords`), districts standing. Closing beats reprise the Verity and the lamp-lit door — now ajar (`last_crossing_complete` is the hook; the scene *says* the door stands open, it does not go through). Pure function of `ColonyState` — the same era always reads the same epilogue, per the game's determinism pillar. Payout note: "the era is over" (matching the finale's "the crossing is over").

### D4 — Post-era Dock and Record render from `era_over()`, no new state

`render_dock`: a new early branch when `colony.era_over()` — the quiet harbor (authored lines: the rift is quiet, no one left to carry, the ship at rest), no charge bar, no jump preview, no `[J]` affordance. `render_record`: when era over, an era-account block (same numbers as D3's account beat) renders above the crossing log, permanent. Both are pure reads of existing colony fields — no new persisted state beyond D1's flag. Covered by two new overlay snapshots via `fixtures::colony_era_complete()` (souls_remaining 0, full districts, records populated, `era_end_shown` true).

## Risks / Trade-offs

- **[Epilogue could double-fire with the finale in one frame]** → it can't: the delivery branch sets `scene_play` to the finale, and the epilogue only fires from the drain when `scene_play` is empty; the one-shot flag then closes it forever. Asserted by the one-shot test.
- **[New colony field vs. frozen corpus]** → `serde(default)`; the corpus `colony.json` is mid-era (`era_over()` false) so the field is inert there; `old saves load` tests extended for the missing-field case.
- **[Record layout overflow at small sizes]** → the era block is a handful of short lines above existing content inside the same scroll; snapshot at the standard tier. (The known small-terminal snapshot gap for voyage panels is tracked separately — pre-flip QA list, not this change.)

## Migration Plan

Additive field + content; no save migration. Old saves that are *already past* the era end (era over, modal long gone) will play the epilogue once on next load — treated as a feature (they finally get the ending), noted in the tasks so it's a conscious ship decision.

## Open Questions

- None blocking. Held-for-later (unchanged from the dossier): surfacing the era record outside the arrived harbor (title screen) — deliberately out of scope here.
