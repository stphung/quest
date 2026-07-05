## 1. Colony: Riftglass and Dock state

- [ ] 1.1 Add `DockState { docked_at: DateTime<Utc> }` and
      `ColonyState.dock: Option<DockState>` (`#[serde(default)]`) to
      `src/vessel/colony.rs`.
- [ ] 1.2 Add `RIFTGLASS_BASE_HOURS_TO_FULL: f64 = 24.0` (design.md Decision
      1) and `ColonyState::riftglass_rate_mult()` (`1.0 /
      self.drive_time_mult()`, design.md Decision 3).
- [ ] 1.3 Add `ColonyState::riftglass_charge(&self, now: DateTime<Utc>) -> f64`
      as a pure function of `dock.docked_at`, `now`, and
      `riftglass_rate_mult()`, capped at 1.0 (design.md Decision 1/4 — no
      overcharge decay).
- [ ] 1.4 Add `ColonyState::dock(&mut self, now: DateTime<Utc>)` (sets
      `self.dock = Some(DockState { docked_at: now })`) and
      `ColonyState::undock(&mut self)` (sets `self.dock = None`).
- [ ] 1.5 Unit tests: charge is 0 immediately after docking, rises linearly
      with elapsed time, is identical whether queried once after a long gap
      or repeatedly across short gaps (mirrors `tick_is_chunking_invariant`'s
      intent), and scales with Drive level per the Decision-3 formula.

## 2. Voyage: partial-charge penalty and `begin_ferry` signature

- [ ] 2.1 Add `MAX_PARTIAL_CHARGE_PROVISIONS_DEFICIT: f64 = 40.0` and
      `MAX_PARTIAL_CHARGE_HULL_WEAR: u8 = 3` to `src/vessel/voyage.rs`
      (design.md Decision 4).
- [ ] 2.2 Extend `VoyageState::begin_ferry()` with a `charge: f64` parameter;
      add a private helper (e.g. `apply_partial_charge_penalty`) that, per
      design.md Decision 4's formula, deducts
      `MAX_PARTIAL_CHARGE_PROVISIONS_DEFICIT * (1.0 - charge)` from
      `provisions` (floored at 0) and sets `hull_wear` to
      `(MAX_PARTIAL_CHARGE_HULL_WEAR as f64 * (1.0 - charge)).round() as u8`;
      a no-op at `charge >= 1.0`.
- [ ] 2.3 Unit tests: `charge = 1.0` produces starting conditions identical to
      today's `begin_ferry()` (regression-proofs the "no penalty at full
      charge" scenario); a range of partial charges produces a monotonically
      increasing penalty as charge decreases, deterministically (same charge
      in ⇒ same penalty out, no RNG).

## 3. Input and main loop wiring

- [ ] 3.1 Add `VoyageInputResult::Jump` to `src/input/voyage_input.rs`,
      replacing the `SailAgain`-producing key arm; wire it to a new
      `VoyageView::Dock` (only available/meaningful while
      `colony.dock.is_some()`).
- [ ] 3.2 Add `VoyageView::Dock` to `src/vessel/mod.rs`'s `VoyageUiState`
      view enum; handle its key input (view a Riftglass readout, jump
      confirmation, and navigation to the existing `Reckoning`/`Record`
      views) in `voyage_input.rs`.
- [ ] 3.3 In `src/main.rs`'s arrival-finale block (where
      `col.deliver_crossing(...)` runs today), call `colony.dock(now)`
      instead of leaving the player one keypress from `SailAgain`.
- [ ] 3.4 In `src/main.rs`'s input-result handling, replace the
      `VoyageInputResult::SailAgain` arm with a `Jump` arm: read
      `colony.riftglass_charge(Utc::now())`, call `colony.undock()`, and
      call `begin_ferry(..., charge)` with that value.
- [ ] 3.5 Extend `src/input/replay_tests.rs` for the new Dock → Jump key
      path, asserting the resulting `VoyageInputResult`/state per the root
      CLAUDE.md's Keyboard Input verification guidance.

## 4. UI rendering

- [ ] 4.1 Add `render_dock()` to `src/ui/voyage_scene.rs`: show Riftglass
      charge (a bar/percentage), the Drive-derived charge rate, and the jump
      action (with an explicit "this is a one-way, no-undo commitment"
      confirmation matching the Vessel's other all-or-nothing choices), plus
      navigation hints to Reckoning/Record.
- [ ] 4.2 Add/update `cargo test overlay_snapshot` coverage for the new Dock
      view across the responsive size tiers; re-bless with
      `INSTA_UPDATE=always cargo test snapshot` and review the diff before
      committing.

## 5. Persistence

- [ ] 5.1 Confirm `colony.json` round-trips the new `dock` field via
      `src/vessel/persistence.rs` (no code change expected beyond serde
      derives already added in section 1, but verify explicitly).
- [ ] 5.2 Run `cargo test --test save_compat_tests` against the committed
      save corpus (`tests/fixtures/saves/`) to confirm existing saves still
      load with `dock: None`.

## 6. Balance validation

- [ ] 6.1 Extend `src/bin/voyage_simulator.rs` to model Dock time and a
      charge policy per strategy (e.g. always-full-charge vs. jump-at-a-fixed
      partial-charge threshold), reporting era length and percent-saved
      alongside the existing metrics.
- [ ] 6.2 Run `cargo run --bin voyage_simulator -- --runs <n> --strategy
      <profile>` across strategies/seeds; confirm the ~19–24 crossing / ~88%
      saved / C1 ≈ 14 real-days targets in `src/vessel/CLAUDE.md` still hold
      with Dock time included. If `RIFTGLASS_BASE_HOURS_TO_FULL = 24.0`,
      `MAX_PARTIAL_CHARGE_PROVISIONS_DEFICIT = 40.0`, or
      `MAX_PARTIAL_CHARGE_HULL_WEAR = 3` push the era outside that envelope,
      adjust the constants (design.md Decisions 1 and 4) and re-run before
      treating them as final.
- [ ] 6.3 Run `cargo run --release --bin simulator -- --check-progression`
      to confirm the endgame-systems scenario is unaffected.

## 7. Docs

- [ ] 7.1 Update `src/vessel/CLAUDE.md`'s Colony section (file table,
      "How It Works," Key Constants) to describe Dock/Riftglass/Jump and the
      new constants.
- [ ] 7.2 Update `docs/dossiers/act2-pilgrimage.md`'s Fun Assessment entry
      that currently flags "ferry runs: one choice per ~3 real days," since
      this change directly addresses it.

## 8. Final verification

- [ ] 8.1 Run the full targeted suite for this change: `cargo test`,
      `cargo test overlay_snapshot`, `cargo test --test save_compat_tests`,
      `cargo run --bin voyage_simulator`, `cargo run --release --bin
      simulator -- --check-progression`.
- [ ] 8.2 Run `make check` before pushing.
