## 1. Resolve the open design question

- [ ] 1.1 Decide the partial-charge penalty formula (design.md Open Question
      1) — recommended default is a provisions/hull-wear deficit scaled by
      `(1.0 - charge)`, applied inside `begin_ferry()`. Pin the exact
      constants (deficit curve, min/max) before starting section 3.
- [ ] 1.2 Decide overcharge behavior (design.md Open Question 2): cap
      Riftglass at 1.0 (recommended default), or allow decay past full.

## 2. Colony: Riftglass and Dock state

- [ ] 2.1 Add `DockState { docked_at: DateTime<Utc> }` and
      `ColonyState.dock: Option<DockState>` (`#[serde(default)]`) to
      `src/vessel/colony.rs`.
- [ ] 2.2 Add `RIFTGLASS_BASE_HOURS_TO_FULL` constant and
      `ColonyState::riftglass_rate_mult()` (`1.0 / self.drive_time_mult()`).
- [ ] 2.3 Add `ColonyState::riftglass_charge(&self, now: DateTime<Utc>) -> f64`
      as a pure function of `dock.docked_at`, `now`, and
      `riftglass_rate_mult()`, capped/decayed per task 1.2's decision.
- [ ] 2.4 Add `ColonyState::dock(&mut self, now: DateTime<Utc>)` (sets
      `self.dock = Some(DockState { docked_at: now })`) and
      `ColonyState::undock(&mut self)` (sets `self.dock = None`).
- [ ] 2.5 Unit tests: charge is 0 immediately after docking, rises linearly
      with elapsed time, is identical whether queried once after a long gap
      or repeatedly across short gaps (mirrors `tick_is_chunking_invariant`'s
      intent), and scales with Drive level per the Decision-3 formula.

## 3. Voyage: partial-charge penalty and `begin_ferry` signature

- [ ] 3.1 Extend `VoyageState::begin_ferry()` with a `charge: f64` parameter;
      add a private helper (e.g. `apply_partial_charge_penalty`) that applies
      task 1.1's formula when `charge < 1.0` and is a no-op at `charge >=
      1.0`.
- [ ] 3.2 Unit tests: `charge = 1.0` produces starting conditions identical to
      today's `begin_ferry()` (regression-proofs the "no penalty at full
      charge" scenario); a range of partial charges produces a monotonically
      increasing penalty as charge decreases, deterministically (same charge
      in ⇒ same penalty out, no RNG).

## 4. Input and main loop wiring

- [ ] 4.1 Add `VoyageInputResult::Jump` to `src/input/voyage_input.rs`,
      replacing the `SailAgain`-producing key arm; wire it to a new
      `VoyageView::Dock` (only available/meaningful while
      `colony.dock.is_some()`).
- [ ] 4.2 Add `VoyageView::Dock` to `src/vessel/mod.rs`'s `VoyageUiState`
      view enum; handle its key input (view a Riftglass readout, jump
      confirmation, and navigation to the existing `Reckoning`/`Record`
      views) in `voyage_input.rs`.
- [ ] 4.3 In `src/main.rs`'s arrival-finale block (where
      `col.deliver_crossing(...)` runs today), call `colony.dock(now)`
      instead of leaving the player one keypress from `SailAgain`.
- [ ] 4.4 In `src/main.rs`'s input-result handling, replace the
      `VoyageInputResult::SailAgain` arm with a `Jump` arm: read
      `colony.riftglass_charge(Utc::now())`, call `colony.undock()`, and
      call `begin_ferry(..., charge)` with that value.
- [ ] 4.5 Extend `src/input/replay_tests.rs` for the new Dock → Jump key
      path, asserting the resulting `VoyageInputResult`/state per the root
      CLAUDE.md's Keyboard Input verification guidance.

## 5. UI rendering

- [ ] 5.1 Add `render_dock()` to `src/ui/voyage_scene.rs`: show Riftglass
      charge (a bar/percentage), the Drive-derived charge rate, and the jump
      action (with an explicit "this is a one-way, no-undo commitment"
      confirmation matching the Vessel's other all-or-nothing choices), plus
      navigation hints to Reckoning/Record.
- [ ] 5.2 Add/update `cargo test overlay_snapshot` coverage for the new Dock
      view across the responsive size tiers; re-bless with
      `INSTA_UPDATE=always cargo test snapshot` and review the diff before
      committing.

## 6. Persistence

- [ ] 6.1 Confirm `colony.json` round-trips the new `dock` field via
      `src/vessel/persistence.rs` (no code change expected beyond serde
      derives already added in section 2, but verify explicitly).
- [ ] 6.2 Run `cargo test --test save_compat_tests` against the committed
      save corpus (`tests/fixtures/saves/`) to confirm existing saves still
      load with `dock: None`.

## 7. Balance validation

- [ ] 7.1 Extend `src/bin/voyage_simulator.rs` to model Dock time and a
      charge policy per strategy (e.g. always-full-charge vs. jump-at-a-fixed
      partial-charge threshold), reporting era length and percent-saved
      alongside the existing metrics.
- [ ] 7.2 Run `cargo run --bin voyage_simulator -- --runs <n> --strategy
      <profile>` across strategies/seeds; confirm the ~19–24 crossing / ~88%
      saved / C1 ≈ 14 real-days targets in `src/vessel/CLAUDE.md` still hold
      with Dock time included, and update that doc's numbers if
      `RIFTGLASS_BASE_HOURS_TO_FULL` or the penalty constants shift them.
- [ ] 7.3 Run `cargo run --release --bin simulator -- --check-progression`
      to confirm the endgame-systems scenario is unaffected.

## 8. Docs

- [ ] 8.1 Update `src/vessel/CLAUDE.md`'s Colony section (file table,
      "How It Works," Key Constants) to describe Dock/Riftglass/Jump and the
      new constants.
- [ ] 8.2 Update `docs/dossiers/act2-pilgrimage.md`'s Fun Assessment entry
      that currently flags "ferry runs: one choice per ~3 real days," since
      this change directly addresses it.

## 9. Final verification

- [ ] 9.1 Run the full targeted suite for this change: `cargo test`,
      `cargo test overlay_snapshot`, `cargo test --test save_compat_tests`,
      `cargo run --bin voyage_simulator`, `cargo run --release --bin
      simulator -- --check-progression`.
- [ ] 9.2 Run `make check` before pushing.
