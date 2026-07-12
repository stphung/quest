# Act 2 Release Hardening

## Why

Act 2 (the Vessel launch gate + Voyage/Ferryman era) is feature-complete and dark-shipped behind `vessel::ACT2_ENABLED = false`, but a release-readiness review (2026-07-12) found four structural gaps that would make flipping the switch a gamble: the flag-ON integration surface has zero automated coverage, the voyage balance gate never runs in CI, Act 2's save files have no cross-version compat corpus, and the ferry-era pacing rests on constants self-documented as "not yet simulator-validated". This change closes those gaps — plus the known doc/narrative drift — so a future one-line flip of `ACT2_ENABLED` is boring instead of risky.

## What Changes

- **Flag-ON smoke coverage**: a new integration-test binary runs with the Act 2 kill-switch enabled (via `QUEST_ACT2=1` set before the `OnceLock` caches) and exercises the gated surface that today only runs in production: whisper emission through the real tick loop (Stage 12c), tick-event → log/ticker mapping, the `[V]` hotkey opening the Vessel overlay through the input harness, and the launch flow up to `perform_launch`. (The `main.rs` loop branches remain e2e-only via the `drive-game` skill — they are binary-crate code that no test harness can reach; documented as such.)
- **Voyage simulator in CI**: `cargo run --release --bin voyage_simulator` joins the `balance` CI job and `scripts/ci-checks.sh` step 4, so route/economy edits that strand a strategy or blow the 20–200 day envelope fail PRs instead of passing silently. The two files stay in sync per the documented convention.
- **Ferry-era balance gates**: the two `#[ignore]`d manual tuning sweeps in `tests/ferryman_tests.rs` become asserted tests with tolerance bands (crossings, era length, % saved), and a missing **Ward-lean spend policy** is added so the documented ~94%-saved Ward line is reproducible by CI — this also removes the "not yet simulator-validated" caveat from the Riftglass/Dock constants by validating them (or surfacing that they need retuning).
- **Act 2 save-compat corpus**: committed mid-crossing `voyage.json` and mid-era `colony.json` fixtures under `tests/fixtures/saves/`, loaded through the real `vessel::persistence` load paths by `save_compat_tests`, giving Act 2 the same schema-evolution tripwire Act 1 saves already have.
- **Doc/narrative sync**: spec coverage for `last_crossing_complete` / the Last Crossing (implemented, currently unspecced); fix the two stale `src/vessel/CLAUDE.md` sections (era-length numbers, launch-transition animation); fix the stale time-scale doc comment in `voyage.rs` (says ~1 real month, actual ≈2 real weeks) and the stale `#[allow(dead_code)]` marker on `LONG_HOLD_PROVISIONS_CAP`; reconcile the pilgrims going-dark narrative-vs-code mismatch deliberately (docs say four ships go dark, code darkens only one — decision recorded in design.md).

## Non-goals

- **Flipping `ACT2_ENABLED`.** The kill-switch stays `false`; the release-guard test `act2_kill_switch_is_off_for_release` is untouched.
- **No gameplay, balance, or content changes** to the voyage, colony, or launch gate. If the promoted balance gates reveal the Riftglass constants miss the era's pacing targets, retuning is a follow-up change, not this one. (One possible exception, decided in design.md: authoring `dark_after` values for pilgrim ships if the narrative-vs-code reconciliation lands on the docs' side — invisible while dark-shipped.)
- **No Act 2 depth work** (veteran souls, salvage refinement, Act 1 bridges) — tracked separately from the systems-braiding exploration.
- **No era-end epilogue / Act 3 stub** — the `last_crossing_complete` dead-end is specced as-is, not resolved.

## Balance/progression impact

None intended. All work is tests, CI wiring, fixtures, specs, and docs. The new balance gates *assert* current measured behavior (balanced ≈29 crossings / ≈4.0 months / ≈87.5% saved; skill spread wide) with headroom bands, mirroring how `--check-progression` gates Act 1.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `vessel-act2`: three requirement-level additions — (1) the Last Crossing / `last_crossing_complete` era-end behavior (implemented but unspecced); (2) the ferry-era balance envelope as normative numbers (crossings/era-length/%-saved bands per spend policy), now CI-asserted; (3) the pilgrim going-dark behavior stated to match the reconciled decision.

## Impact

- **Code**: `src/vessel/mod.rs` (only if test-injection seam is needed; prefer env-var-first-process approach), `src/vessel/voyage.rs` (doc comments only), `src/vessel/pilgrims.rs` (only if reconciliation lands code-side), `src/vessel/CLAUDE.md`.
- **Tests**: new `tests/act2_flag_on_tests.rs`; `tests/ferryman_tests.rs` (de-ignore + Ward policy); `tests/save_compat_tests.rs` + new fixtures under `tests/fixtures/saves/`.
- **CI**: `.github/workflows/ci.yml` (`balance` job), `scripts/ci-checks.sh` (step 4) — kept in sync.
- **Specs/docs**: `openspec/specs/vessel-act2/spec.md` (via delta), `src/vessel/CLAUDE.md`, `docs/dossiers/act2-pilgrimage.md` (open questions #6–#8 resolve).
