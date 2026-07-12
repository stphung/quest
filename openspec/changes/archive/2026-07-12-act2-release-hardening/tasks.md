# Tasks — Act 2 Release Hardening

## 1. Ferry-era balance gates (do first: proves the bands hold before anything depends on them)

- [x] 1.1 Add a `ward_lean_spend` policy to `tests/ferryman_tests.rs` (Ward kept ahead, then balanced spend) and include it in the sweep output
- [x] 1.2 De-`#[ignore]` `strategy_sweep` and `dock_time_across_charge_policies`; convert eprintln-only sweeps into asserted gates with the D3 bands (balanced: 20–40 crossings / 3–6 mo / ≥82% saved; drive-only ≤74%; ward-lean ≥90%; full-charge ≥ 0%-charge saved), keeping the diagnostic output
- [x] 1.3 Run `cargo test --release --test ferryman_tests` and record the measured values next to the assertions as comments; if any band fails with current constants, STOP and surface (per design D3) rather than tuning constants
- [x] 1.4 Remove the "not yet simulator-validated" caveats from `colony.rs`/`voyage.rs` doc comments and `src/vessel/CLAUDE.md:183` (they are now CI-validated), pointing at the ferryman gates instead

## 2. Save-compat corpus for Act 2 account files

- [x] 2.1 Generate a mid-crossing `voyage.json` (underway on a road, stations staffed, ≥1 refit chosen, letters/rumor state non-empty, partial provisions) and a mid-era `colony.json` (several yard levels, ≥2 districts, docked with partial Riftglass charge) from current code with `character_id = "corpus-fixture"`, and commit them under `tests/fixtures/saves/v1/`
- [x] 2.2 Add `save_compat_tests.rs` tests loading both through the real `vessel::persistence` load paths (fixture dir override), asserting load-bearing fields survive (phase, provisions bits, crossing_number, yard levels, dock state, souls statuses) and that the `character_id`-mismatch discard path still discards
- [x] 2.3 Note in `tests/fixtures/saves/README.md` that vessel files joined the v1 corpus (additive coverage, not a format migration)

## 3. Flag-ON smoke coverage

- [x] 3.1 Create `tests/act2_flag_on_tests.rs` with a `Once`-based `enable_act2()` helper (sets `QUEST_ACT2=1` then primes `vessel::act2_enabled()`); file-top comment stating the whole-binary-runs-ON invariant
- [x] 3.2 In that binary, cover the lib-side gated surface: whisper stage emits `TickEvent::VesselWhisper` through the real `game_tick_with_context` loop at the 60s cadence after discovery; discovery event flows (Zone 50 kill → `vessel_signal_discovered` + `TickEvent::VesselSignalDiscovered`); `can_launch`/`perform_launch` full flow with the flag on
- [x] 3.3 Add self-skipping `flag_on_`-prefixed tests in the bin crate: `src/input/replay_tests.rs` — `[V]` opens `GameOverlay::Vessel` when discovered, Enter/Esc confirm flow reaches `perform_launch`; `src/tick_events.rs` tests — `VesselSignalDiscovered`/`VesselWhisper` map to combat-log + ticker entries and set the `TickEventFlags`. Each guards `if !vessel::act2_enabled() { return; }`
- [x] 3.4 Verify both worlds locally: `cargo test flag_on` (all self-skip, green) and `QUEST_ACT2=1 cargo test flag_on` (all exercise, green); plus `cargo test --test act2_flag_on_tests`
- [x] 3.5 Document the flag-ON test map in `src/vessel/CLAUDE.md` (what is covered where; stats-panel row and `main.rs` loop branches remain `drive-game`-only, per design D1)

## 4. CI wiring

- [x] 4.1 Add `cargo run --release --bin voyage_simulator` to the `balance` job in `.github/workflows/ci.yml` and to `scripts/ci-checks.sh` step 4 (kept in sync; comment noting the pairing)
- [x] 4.2 Add the `QUEST_ACT2=1 cargo test flag_on` step to the `test` job in `ci.yml` and to `scripts/ci-checks.sh` (comment: tests self-skip when dark; this step is what runs them)
- [x] 4.3 Confirm `scripts/ci-checks.sh` passes end-to-end locally (`make check`)

## 5. Doc/narrative sync

- [x] 5.1 Fix `src/vessel/CLAUDE.md`: era-length line to the measured range (29 crossings / ~4.0 mo / 87.5% balanced; ward-lean slower/higher), and the Launch Transition section to describe the animated "Ignition" (`src/ui/vessel_transition_fx.rs`)
- [x] 5.2 Fix `voyage.rs:67-73` time-scale doc comment (sea-day ≈ 9 real hours; maiden ≈ two real weeks) and remove the stale `#[allow(dead_code)]` + "lands with spec 4" comment on `LONG_HOLD_PROVISIONS_CAP` (it is live via the Long Hold refit)
- [x] 5.3 Reword the four "going dark like the other four" occurrences in `docs/dossiers/act2-pilgrimage.md` to match code (only the Grief of Alden darkens; design D5); resolve dossier open questions #6–#8 with pointers to this change
- [x] 5.4 Ensure `GameState::last_crossing_complete` doc comment and `src/vessel/CLAUDE.md` name the Last Crossing behavior now specced in the delta

## 6. Verification

- [x] 6.1 Targeted rows from CLAUDE.md's verification table: `cargo test --test save_compat_tests`, `cargo test --test ferryman_tests --release`, `cargo test --test vessel_launch_gate_test`, `cargo test --test act2_flag_on_tests`, `QUEST_ACT2=1 cargo test flag_on`, `cargo run --bin voyage_simulator`
- [x] 6.2 Full gate: `make check`
- [x] 6.3 `openspec validate --change act2-release-hardening`
