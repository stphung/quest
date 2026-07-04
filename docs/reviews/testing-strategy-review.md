# Quest Testing Strategy Review

**Scope:** Full-codebase audit of testing strategy — CI gates, coverage measurement, simulators, and per-module test quality. Conducted via 5 parallel research passes across all `src/` modules plus the cross-cutting CI/tooling layer.

**Bottom line:** The test suite is unusually mature for a project this size (2,400+ inline unit tests, 3,000+ integration tests, seeded-RNG discipline almost everywhere, three purpose-built simulators, a well-designed save-compatibility corpus). The most valuable next steps aren't "write more tests" — they're fixing a coverage *measurement* gap that's been silently hiding its own blind spots, and closing a handful of specific holes the measurement gap let slip through, including two live production bugs the review turned up along the way.

---

## Top 5, ranked

### 1. The 90%-line coverage gate measures the wrong thing (High) — confirmed by actually running it

Measured directly (`cargo-llvm-cov`, installed fresh for this check):

| Invocation | Line coverage |
|---|---|
| `--lib` only, current CI exclusion regex (what CI actually runs) | **90.57%** |
| Full run (adds `tests/`'s ~3,000 integration tests + bin-crate inline tests), same exclusion regex | **81.44%** |
| Full run, zero exclusions | **65.23%** |

The 9-point drop between the first two rows isn't only "`tests/` doesn't run under `--lib`" — it's structural. `input/`, `main_helpers/`, and `tick_events` are declared with `mod` **only in `src/main.rs`**, never in `src/lib.rs` (confirmed via `grep '^mod \|^pub mod '` on both files). `tests/` integration tests link against the `quest` *library* crate, so they cannot reach those modules under any coverage invocation — not a measurement-command problem, a crate-boundary problem. Their real coverage can only come from inline `#[cfg(test)]` blocks inside the binary crate itself, and per-file numbers show that's thin: `main_helpers/input_routing.rs` and `main_helpers/persistence.rs` are a literal **0.00%**, as are `input/soulforge_input.rs`, `input/stormglass_input.rs`, and `input/minigame_input.rs`; `input/haven_input.rs` sits at 5.68%. (`main.rs` itself is 0.00% too, which is expected and fine — it's a thin entry point.)

The exclusion-list claim checks out with real numbers, though it's more nuanced than "all excluded files are secretly fine": re-running coverage with the exclusion regex removed, of the 6 currently-excluded files, `enhancement/logic.rs` (100%), `stormglass/earning.rs` (100%), `stormglass/types.rs` (97.27%), `deep/discovery.rs` (86.84%), and `stormglass/spending.rs` (76.19%) are genuinely well-tested and should not be blanket-excluded from the gate. Only `deep/persistence.rs` (28.57%) and `enhancement/persistence.rs` (9.52%) are the legitimately-hard-to-test I/O glue the exclusion list was presumably meant for.

**Fix, in two independent parts:**
1. ✅ **Done** (this PR): dropped `enhancement/logic.rs`, `stormglass/{earning,spending,types}.rs`, and `deep/discovery.rs` from the ignore regex in both `ci.yml` and `ci-checks.sh` — they were already covered (76–100%), so the gate should watch them. Kept `deep/persistence.rs` and `enhancement/persistence.rs` excluded (28.57%/9.52% — genuinely hard-to-test I/O glue). Verified locally: the gate still clears comfortably at 91.03% (up slightly from 90.57%, since the newly-included files pull the average up rather than down).
2. **Tracked in [#673](https://github.com/stphung/quest/issues/673)**: `input/`, `main_helpers/`, `tick_events` (bin-only, structurally unreachable by `tests/`) need their own explicit coverage measurement — either move testable logic into `lib.rs`-reachable modules (the input-replay harness pattern already does this for some handlers, e.g. `time_vault_input.rs` at 95.30% and `harness.rs` at 85.35% show it works when done), or add a second, separate coverage gate scoped to the binary crate. That issue tracks writing the missing tests first (several files are a literal 0%) before gating on the number, so CI doesn't just start failing for no immediate benefit.

Separately, add a small test asserting `ci.yml`'s and `ci-checks.sh`'s coverage-ignore regexes stay identical — they've already drifted once (documented in CLAUDE.md) and nothing currently prevents it from recurring.

### 2. Two live production bugs surfaced during the audit (High)
Not testing-strategy gaps in the abstract — real bugs, found because the review went looking for untested clock/ordering-sensitive code:
- **Nondeterministic Deep roster rendering**: `deep_missions::render_roster` iterates a `HashMap` via `.values()`, so mercenary roster order can visibly shuffle frame-to-frame in the real game. It's currently *known* (documented as a snapshot-test exclusion in `overlay_snapshot_tests.rs:14-16`) but not fixed. Switching to a stable-ordered structure (sort by ID, or a `Vec`/`BTreeMap`) fixes the bug and unlocks snapshot coverage in one move.
- **Direct wall-clock reads bypassing the UI clock rule**: `stormglass/types.rs:96,220` (`ExchangeUiState.created_at_ms`) and `input/stormglass_input.rs:19` call `SystemTime::now()` directly, and `ui/stormglass_scene.rs` reads it straight through for animation/elapsed-time math — exactly the pattern `src/ui/CLAUDE.md` prohibits. It's currently masked because the Stormglass rolling-phase scene is *excluded* from the determinism-checking snapshot tests rather than fixed. Same issue, different corner: the character-select splash reads `Utc::now()` directly and is excluded from snapshots for the same reason.

Both should route through the existing freezable `ui/clock.rs` abstraction; once they do, the existing exclusions in `overlay_snapshot_tests.rs` can be deleted and real coverage restored.

### 3. Item drop RNG isn't threaded through the seeded-RNG discipline (High)
`try_drop_from_mob`/`try_drop_from_boss` (`src/items/drops.rs:27,53`) call `rand::rng()` directly instead of accepting `rng: &mut R` like every other RNG-consuming function in `core/`. The production call site (`tick_stages.rs:533-551`) doesn't thread the tick's seeded RNG in either. This is the one non-deterministic path inside `game_tick_with_context()` — it silently undermines `--seed` reproducibility in the simulator and blocks any future deterministic-replay tooling. Tests compensate with large-trial statistical assertions, which prove distribution correctness but can't pin an exact sequence, and are architecturally inconsistent with the rest of the codebase (`core/CLAUDE.md`'s "Known Issues: None currently" note about threaded RNG omits this file).

### 4. Input-routing and Act 2's live integration surface have no automated tripwire (High)
- `main_helpers/input_routing.rs` (478 lines) and `persistence.rs` (118 lines) — the layer that turns an `InputResult` into an actual save/git-commit — have zero tests, and the input-replay harness stops one layer short: it asserts the `InputResult` variant but never routes it through `route_game_input()`/`dispatch_time_vault_action()` to confirm the save/commit/reload sequence actually happens. A wrong branch here is silent save loss with no automated catch, the exact risk class the module's own docs warn about.
- No test anywhere references `QUEST_ACT2` (confirmed by repo-wide grep). The `ACT2_ENABLED`-gated call sites (`tick_vessel_whispers`, the `[V]` hotkey, the stats-panel row) are never exercised end-to-end with the flag on — the large Voyage test suites (162–431 lines each) correctly bypass the flag by calling `VoyageState`/route logic directly, but that means the integration surface the flag actually controls has zero regression coverage.
- Several full-screen overlays in the same family as the tested ones have no snapshot coverage: `ascension_scene.rs`, `achievement_browser_scene.rs`, `title_browser_scene.rs`.

### 5. A tautological assertion and a thin end-to-end prestige path (Medium)
`tests/game_tick_tests/game_tick_behavior_test.rs:187-190` (`test_combat_kill_xp_can_cause_level_up`) asserts `character_level > level_before || character_xp < xp_needed` — true almost by construction of the level-up loop, so it can't fail on a broken level-up path. Worth auditing `test_level_up_triggers_achievement_sync` nearby for the same pattern. Separately, `test_combat_to_prestige_full_loop` manually re-orchestrates `update_combat` + `apply_tick_xp` rather than driving the real `game_tick_with_context()` entry point through a multi-hundred-tick prestige cycle — a stage-ordering bug in `tick.rs` around a real prestige action could go undetected.

---

## Per-area findings

### Core / Combat / Character / Zones / Dungeon
**Strong:** exact-boundary tests at P5/P10/P15/P20, a genuine 20k-tick kill→levelup→prestige integration test, dungeon generation invariants (BFS reachability, exactly-one-boss, boss-is-dead-end) across all 5 size tiers, broad seeded-RNG discipline (54 files use `ChaCha8Rng`), floor-clamp tests on damage/HP underflow.

**Gaps:**
- No same-seed determinism test for `distribute_level_up_points` (`core/xp.rs`) or combat crit rolls — the two RNG-sensitive functions the module's own docs flag as deliberately threaded.
- Dungeon integration tests are scattered across 4 different `tests/` files with no dedicated `dungeon_tests.rs` — fine today, but ownership is unclear for future changes.
- `FrontierBackoff` / Loom's triple-gate (patterns + ascension + prestige) have thinner boundary coverage than early zones — no P2000/P5000-style exact-boundary test.
- `save_compat_tests.rs` doesn't include a fixture with an in-progress `active_dungeon` (partial room-clear state).

### Items / Enhancement / Ascension / Deep / Stormglass / Power Cores / God Items
**Strong:** item drop-rate distributions (thousands-of-trials statistical tests plus hard invariants — mobs never Legendary, bosses never Common), enhancement success-rate table cross-checked against Soul Tithe pricing, Power Cores tick/offline-catchup fully unit-tested, God Items (22 tests) solidly covered despite being easy to overlook.

**Gaps beyond #1 and #3 above:**
- No test combines Ascension's multiplier with Enhancement's multiplier through the real combat pipeline order (base → Giant's Might → Haven → prestige flat → ascension mult → defense) — each is tested in isolation only.

### Haven / Achievements / Challenges / History / Loom / Vessel
**Strong:** all vessel/history test suites use properly seeded RNG and fixed injectable timestamps (a genuine strength — easy to get wrong in a wall-clock-simulated system like the Voyage); achievements have 405 tests across `tests/achievement_tests/` despite zero inline tests in the handler files (an unusual but acceptable split).

**Gaps:**
- The `go` minigame (36 tests) has **zero forfeit-pattern tests**, despite implementing the same double-Esc flow as the other 13 minigames — the only one of the 14 with this hole.
- Weakest minigames by test depth: `sudoku` (11 tests, one shallow forfeit test), `vault_warden` (20 tests, single generic forfeit test), `runic_lights`/`runic_shift`/`jezzball` (21 tests each, minimal forfeit coverage). Contrast with `chess` (58 tests, 9 dedicated forfeit tests) and `morris` (63 tests). No minigame has a systematic all-4-difficulty-tiers parity check.
- `loom/layout.rs` (Sugiyama layout algorithm) has only 3 inline tests and is also excluded from the coverage gate — no backstop of either kind.
- `history/cloud.rs`'s actual network operations (`push_all_branches`, `fetch_all`, `check_divergence`, `backup_and_reset`) have no coverage — understandable without an HTTP-mocking layer, but the divergence/backup state machine is mockable in principle and worth a look.
- Wall-clock-based (`Instant::now()`) timing assertions in `haven_dungeon_coverage_test.rs` and `achievements_expanded_test.rs` (500–1000ms thresholds) are a latent CI-flakiness risk, though margins look generous today.

### UI / Input / Fixtures / Main Helpers / Utils
Covered in items #2 and #4 above (the HashMap-ordering bug, the clock-bypass bug, input-routing gap, Act 2 integration gap). Additionally:
- `snapshot_all_minigames` only captures each game's freshly-started state — never win/loss/draw overlays or forfeit-pending prompts, which are shared code paths rendered nowhere in the snapshot suite.
- `utils/build_info.rs`/`utils/updater.rs` coverage exclusions are appropriate (pure-logic slices are already tested; excluded code is genuine network/binary-replacement I/O). `utils/debug_menu.rs` has 53 tests and its CI-only exclusion looks like incidental noise reduction rather than a real gap.

### CI, simulators, and cross-cutting tooling
- **Criterion benchmarks are inert**: `benches/game_tick.rs` is never invoked by CI or `make check` (confirmed by grep — zero hits), and no committed baseline exists for local comparison either. The hottest path in the game (the 100ms tick loop) has no automated perf-regression guardrail.
- **Progression-check scenarios are appropriately coarse for a smoke test, not a balance-regression detector**: all 18 assertions are one-sided (`≥`/`≤`) with ~2–4x slack baked in from observed values, and most have no upper bound at all — a 10x swing in PR income in the *generous* direction would pass silently. Fine for "the game still progresses," not sufficient for "the game is still balanced."
- **`save_compat_tests.rs` is a strong, well-documented pattern but under-exercised by real history**: only one save-format generation (`v1/`) exists, so the documented `v2/` regeneration workflow has never actually been dry-run against a real save-breaking change.
- **Missing capabilities worth considering:**
  - Property-based testing (proptest) for RNG-heavy generation (item generation, sigil grades, Loom pattern generation) — current tests are example-based even where seeded, and could miss edge-of-domain values.
  - Fuzzing the save-file JSON parser — it's the one place the game parses adversarial-shaped (hand-edited/corrupted) input, and `save_compat_tests.rs` only covers known-good historical formats, not malformed ones.
  - `cargo-mutants` on a few high-value modules (combat damage pipeline, enhancement rolls) to validate the suite catches logic bugs rather than just executing lines — especially relevant given finding #1's revelation that the coverage number is already a weaker signal than assumed.

---

## Suggested sequencing

1. ✅ Rebuild the lib-reachable exclusion list from measured coverage (#1, part 1) — landed in this PR.
2. Fix the two live bugs (#2) — small, isolated, immediately valuable regardless of any strategy change.
3. Give `input/`/`main_helpers/` real coverage, then their own gate (#1, part 2) — tracked in [#673](https://github.com/stphung/quest/issues/673).
4. Thread seeded RNG through item drops (#3) — restores determinism guarantees the rest of the codebase already relies on.
5. Add a routing-level test for `input_routing.rs`'s save/commit/reload sequence and an end-to-end `QUEST_ACT2=1` smoke test (#4) — the `input_routing.rs` piece is the top item in #673; the Act 2 smoke test is separate follow-up.
6. Everything else in "Per-area findings" is real but lower-urgency — worth working through opportunistically alongside feature work in each area, rather than as a dedicated pass.
