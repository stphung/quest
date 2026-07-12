# Design — Act 2 Release Hardening

## Context

Act 2 ships dark behind `vessel::ACT2_ENABLED = false` (`src/vessel/mod.rs`), with `act2_enabled()` OR-ing in the `QUEST_ACT2=1` env var, **cached once per process in a `OnceLock`**. That cache is the central design constraint of this change: a test process cannot toggle the flag after the first check, so all existing tests either assert the flag is off or bypass it by driving `VoyageState`/`ColonyState` directly. The result (release-readiness review, 2026-07-12): the engine is thoroughly tested, but the *wiring that surfaces it when the switch flips* has zero automated coverage, the voyage balance binary never runs in CI, Act 2's two account files (`voyage.json`, `colony.json`) have no compat corpus, and the ferry-era pacing constants are self-documented as unvalidated.

A second constraint discovered during design: the crate split. `src/input/`, `src/tick_events.rs`, and `main.rs` are **binary-crate** modules (`src/main.rs` declares `mod input;` etc.); only `src/core/`, `src/vessel/`, etc. are in the library. Integration tests under `tests/` can reach the lib but not the input harness or the tick-event → ticker mapping. Flag-ON coverage therefore needs two delivery vehicles, not one.

## Goals / Non-Goals

**Goals:**
- Every `act2_enabled()`-gated code path that a test harness *can* reach is exercised at least once with the flag ON, in CI.
- `voyage_simulator`'s structural assertions and the ferry-era pacing sweeps gate PRs.
- A serde break to `VoyageState`/`ColonyState` turns a silent player-save wipe into a red test, same as Act 1 saves.
- The four known doc/narrative drifts are resolved and the Last Crossing is specced.

**Non-Goals:**
- Flipping `ACT2_ENABLED`; any gameplay/balance/content change; retuning Riftglass constants if the new gates reveal a miss (that would be a follow-up change with its own proposal); testing `main.rs` loop branches in-process (binary-crate `fn main` internals — remains e2e via the `drive-game` skill).

## Decisions

### D1 — Flag-ON coverage via two vehicles: an env-priming integration binary (lib surface) + self-skipping `flag_on` tests run in a dedicated CI process (bin surface)

- **Lib surface** (`core/tick.rs` Stage 12c whisper gating, discovery → whisper tick pipeline, launch gate flow): new `tests/act2_flag_on_tests.rs`. Each test calls a shared `enable_act2()` helper — a `std::sync::Once` that sets `QUEST_ACT2=1` (safe in edition 2021) and immediately primes `vessel::act2_enabled()`, before anything else in the process can cache it dark. Every test in this binary runs flag-ON; the binary is its own process, so it cannot contaminate other test binaries.
- **Bin surface** (`input/mod.rs` `[V]` hotkey → Vessel overlay via the replay harness; `tick_events.rs` mapping of `VesselSignalDiscovered`/`VesselWhisper` to combat-log + ticker): new tests named with a `flag_on_` prefix that **self-skip** when dark (`if !vessel::act2_enabled() { return; }`). In the ordinary `cargo test` process they are green no-ops; a dedicated CI step `QUEST_ACT2=1 cargo test flag_on` re-runs the already-built test binaries in a fresh process where the `OnceLock` caches ON, and the name filter selects exactly these tests (the existing dark assertions like `vessel_overlay_hotkey_stays_inert_dark` are filtered out, so the two worlds never meet in one process).
- **Alternatives rejected:**
  - *`#[cfg(test)]` injection seam inside `act2_enabled()`* — dead on arrival twice over: integration tests compile the lib without `cfg(test)`, and the bin crate's tests link a non-test lib. The seam would only serve lib unit tests, the one place that doesn't need it.
  - *Cargo feature flag enabling an override* — a self-dev-dependency trick; pollutes the feature set of a game crate for a test-only concern and risks feature-unification surprises.
  - *Subprocess-spawning tests (test spawns `cargo test` with env)* — nested cargo in CI is slow and fragile; the dedicated-step approach gets the same process isolation from CI's own process model for free.
- **Untestable remainder, documented rather than forced:** `ui/stats_panel.rs`'s gated row and the `main.rs` transition/Voyage loop branches share a process with dark-asserting tests (lib unit tests) or live in `fn main` respectively. They stay on the `drive-game` e2e path; `src/vessel/CLAUDE.md` gets a "flag-ON test map" note saying exactly what is covered where, so the gap is a known, named one.

### D2 — `voyage_simulator` joins the Balance gate in both CI files

`.github/workflows/ci.yml`'s `balance` job and `scripts/ci-checks.sh` step 4 both gain `cargo run --release --bin voyage_simulator` after the existing `--check-progression` run (defaults: all 5 strategies; measured runtime is seconds — build shares the release profile already used by the progression check). The two files are maintained in parallel per the documented convention; both are edited in the same task. Alternative rejected: running it inside `cargo test` as a `#[test]` wrapper — it would double the test job's wall time with a release build and lose the binary's CLI ergonomics for local tuning.

### D3 — Promote the ferryman sweeps to asserted gates with headroom bands; add the missing Ward policy

`tests/ferryman_tests.rs`'s `strategy_sweep` and `dock_time_across_charge_policies` lose `#[ignore]` and gain assertions with ~2x-style headroom in the same spirit as `simulator --check-progression` (coarse facts, not exact numbers): balanced line lands in 20–40 crossings, 3–6 real months, ≥82% saved; drive-only ≤74% (the reckless trap stays a trap); full-charge jumping saves ≥ jump-at-0%. A `ward_lean_spend` policy (Ward kept ahead, then balanced) is added to the sweep and gated at ≥90% saved / a slower era than balanced — making the documented "~94% saved, deliberate slower branch" reproducible instead of folklore. Measured baseline (2026-07-12, this workspace): balanced 29 crossings / 4.0 mo / 87.5%; cap-lean 18 / 3.2 mo / 88.8%; drive-only 101 / 11.4 mo / 70.5%; cap-only 15 / 9.6 mo / 74.2%; full-charge 87.5% vs 0%-jump 84.5%. These gates are what retire the "not yet simulator-validated" caveats on `RIFTGLASS_BASE_HOURS_TO_FULL` / partial-charge constants: the constants become validated-by-CI against the era's targets. If a band cannot hold with today's constants, the band is set to today's truth and the retune is escalated as a follow-up — this change never moves a gameplay constant.

### D4 — Extend the v1 save corpus additively with `voyage.json` / `colony.json`

`tests/fixtures/saves/v1/` gains two new frozen files: a mid-crossing `VoyageState` (underway on a road, souls staffed, some rumors/refits/letters state, non-trivial provisions) and a mid-era `ColonyState` (several yard levels, districts founded, docked with partial Riftglass charge). They are generated once from current code (same approach as the corpus's `regenerate_save_corpus` helper), then frozen under the corpus's existing rules. `save_compat_tests.rs` loads them through the **real** load paths (`vessel::persistence::load_voyage` / `load_colony` pointed at the fixture dir, exercising the `character_id` keying), not bare `serde_json::from_str`, and asserts a handful of load-bearing fields survive. Adding files to v1 is additive coverage of previously-uncovered state, not an edit of frozen files; the README's rules section gets one line saying vessel files joined the corpus on this date. Alternative rejected: starting a v2 generation — v2 signals a *format migration*, which this is not, and would double every existing file for no coverage gain.

### D5 — Pilgrims going-dark: reconcile docs to code

The code is deliberate and self-consistent: `pilgrims.rs` states the design intent in its module doc ("their fates are authored… their story is weather, not consequence"), authors `dark_after: Some(40)` for exactly one ship (the Grief of Alden, "she goes dark in the middle water"), keeps the Sister Verity as "a face for Act 3", and enshrines it in `the_grief_of_alden_goes_dark_and_the_verity_sails_on`. The dossier's repeated "…rather than eventually going dark like the other four" is the drift — it overstates one authored fate into four. **Fix the dossier wording** (4 occurrences in `docs/dossiers/act2-pilgrimage.md`) to "unlike the Grief of Alden, which goes dark in the middle water"-style phrasing; no code change. This preserves the proposal's no-behavior-change promise and respects the authored-content covenant (darkening three more ships would be new narrative content, out of scope; if the design ever wants it, that's an authored-content change with its own proposal). The capability spec delta states the actual behavior (five ships, one authored darkening, hail-once) so spec, code, and dossier finally agree.

### D6 — Spec delta scope: `vessel-act2` only

Three requirement additions/changes, all documenting shipped behavior: the Last Crossing (`era_over()` → final modal, `last_crossing_complete` set, no dock offered — the Act 3 gate alongside `vessel_arrived`); the ferry-era balance envelope as normative coarse bands (mirroring how progression facts are specced for Act 1); pilgrim ships' authored fates. Persistence corpus and CI wiring are verification infrastructure, not capability requirements — no `persistence` spec delta.

## Risks / Trade-offs

- **[Env-var tests are process-order sensitive]** → the `Once`-based `enable_act2()` helper is the *first line of every test* in the integration binary, and the binary contains nothing that wants the flag dark; a comment at the top of the file states the invariant.
- **[`flag_on` name-filter convention is implicit]** → the CI step and `ci-checks.sh` carry a comment ("self-skipping when dark; this step is what actually runs them"); the tests' self-skip guard means a developer running `cargo test` locally can never be broken by them, only CI exercises them.
- **[Balance bands could flake across seeds/platforms]** → the sweeps are fully deterministic (seeded, pure-function weather/nights, no wall clock — verified by `the_partial_charge_penalty_is_deterministic` and chunking-invariance tests), so bands are stable; headroom exists to absorb deliberate future tuning, and the bands' widths are recorded next to the assertions.
- **[Adding release-profile sim runs lengthens the Balance job]** → voyage_simulator reuses the release build cache from the progression check; measured cost is seconds.
- **[Corpus fixtures embed a `character_id`]** → fixtures use a fixed, obviously-synthetic id (`"corpus-fixture"`) and the test loads with that same id, also asserting the mismatch-discard path stays intact.

## Migration Plan

Pure additive test/CI/doc change — no deploy or rollback concerns. Land order inside the change: fixtures + tests first (prove green), CI wiring second, spec/doc sync last. If the D3 bands fail against current constants, stop and surface before adjusting anything.

## Open Questions

- None blocking. One deliberate deferral: whether the Riftglass pacing *targets* themselves (≈3-month balanced era) are right is a design question for the depth/extension track, not this hardening change — the gates here freeze current behavior so any future retune is visible and intentional.
