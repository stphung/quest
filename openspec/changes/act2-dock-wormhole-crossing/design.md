## Context

Today, a ferry crossing's arrival (`main.rs`, the `take_finale_playback()` /
`SailAgain` block, lines ~706-855) does three things in sequence with no
player-facing gap between them: (1) `col.deliver_crossing(...)` pays out
Salvage and folds new Districts/World Milestones into the Colony, (2) the
player reads the arrival scene and can freely spend Salvage via
`VoyageView::Reckoning` (`d`/`c`/`w` → `buy_drive`/`buy_capacity`/`buy_ward`
in `colony.rs`), and (3) pressing `N` (`VoyageInputResult::SailAgain`,
`voyage_input.rs:477`) instantly calls `VoyageState::begin_ferry()` and the
next crossing starts. There is no elapsed-real-time cost to step 3 — a player
can chain crossings back-to-back with no in-fiction reason not to, which is
exactly what `act2-pilgrimage.md`'s Fun Assessment flags as the era's weak
spot.

Two existing patterns this design reuses rather than reinvents:
- **Pure-function-of-time state** (`weather.rs`, `nights.rs`): both are
  computed from `(seed, hour/day)` with nothing stored — no incremental
  mutation, no drift, offline-safe by construction. Riftglass accrual is the
  same shape: monotonic, real-time-driven, safe to compute from an anchor
  timestamp instead of being ticked.
- **Colony-owned persistence surviving the Voyage's replacement**
  (`colony.rs`'s file header: "Survives every crossing where the voyage is
  replaced"). `VoyageState` is wholesale replaced by `begin_ferry()` on every
  new crossing, so any state that must survive *across* the Dock-to-jump
  transition belongs on `ColonyState`, not `VoyageState`.

Districts today (`District::ALL`, 6 variants, `colony.rs:86-153`) are
auto-founded purely by population threshold and each contributes only a flat
capacity bonus to `expedition_size()` — the flavor text (e.g. Beacon: "each
crossing builds more drive") is aspirational prose, not a mechanically
distinct effect per district. This confirms the exploration's diagnosis:
districts are one mechanical lever wearing six skins. This proposal does not
fix that (out of scope, see proposal.md Non-goals) but it means "let a
District modify the Riftglass rate" would require inventing the first
mechanically-distinct district, which is real added scope — see Decision 3.

## Goals / Non-Goals

**Goals:**
- Give the Ferryman era a second, real-time-gated decision point between
  crossings without touching the maiden voyage's authored outbound leg or
  the existing Voyage return-crossing machinery (route/phases/pace/weather/
  threats in `voyage.rs`).
- Make the timing of the wormhole jump a genuine, deterministic risk/reward
  choice — full charge is safe and slow, partial charge is fast and costly.
- Keep the change additive and save-compatible: no existing save should
  behave differently until its character's next crossing arrival.

**Non-Goals:**
- Building session 5's ship-tier/district mutual-gating/Refinement braid (see
  proposal.md).
- Changing what Districts mechanically do, or adding a 7th district, unless
  a later change deliberately extends this one (see Decision 3).
- Pinning the *final, simulator-validated* magnitude of the partial-charge
  deficit or `RIFTGLASS_BASE_HOURS_TO_FULL` — this document pins starting
  values (Decision 4) so implementation isn't blocked, but tasks.md section 7
  still runs the `voyage_simulator` pass to confirm they don't blow the
  ferry loop's ~19–24 crossing / ~88% saved / ~3-real-month targets before
  the numbers are considered final.

## Decisions

### Decision 1: Dock state lives on `ColonyState`, not `VoyageState`

Add:
```rust
// colony.rs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DockState {
    pub docked_at: DateTime<Utc>,
}

pub struct ColonyState {
    // ...existing fields...
    #[serde(default)]
    pub dock: Option<DockState>,
}
```

`Some(DockState { docked_at })` is set the moment a crossing's arrival
delivers (in the `take_finale_playback()` block of `main.rs`, immediately
after `col.deliver_crossing(...)`), for **every** arrival from the maiden
voyage onward — since the maiden voyage's arrival is the first transition
into the ferry loop, gating "does Dock apply" is simply "is `colony` some" (a
`ColonyState` only exists after the first arrival calls
`ColonyState::found()`), no separate crossing-number check needed.

Riftglass charge is a **pure function of elapsed real time**, mirroring
`weather.rs`'s pure-function style — no incremental accrual to tick, no risk
of drift on long absences:

```rust
impl ColonyState {
    /// Real-time hours to reach full charge at the current Drive level.
    pub fn riftglass_hours_to_full(&self) -> f64 {
        RIFTGLASS_BASE_HOURS_TO_FULL / self.riftglass_rate_mult()
    }

    /// 0.0 (just docked) to 1.0 (fully charged), given a Drive-scaled rate.
    /// Capped at 1.0 (Decision 4) — no decay or benefit from overcharging.
    pub fn riftglass_charge(&self, now: DateTime<Utc>) -> f64 {
        let Some(dock) = &self.dock else { return 0.0 };
        let elapsed_hours = (now - dock.docked_at).num_seconds() as f64 / 3600.0;
        (elapsed_hours / self.riftglass_hours_to_full()).min(1.0)
    }
}
```

`RIFTGLASS_BASE_HOURS_TO_FULL = 24.0` — about a real day to fully charge at
Drive level 0 (the maiden voyage's first Dock), per product direction. This
is a starting value, not a final balance number — see Decision 4's note on
simulator validation.

**Alternative considered**: model Dock as a new `VoyagePhase::Docked { .. }`
variant alongside `Traveling`/`Drifting`/`HoldingStation`/`Arrived`. Rejected
— `VoyageState` is destroyed and rebuilt by `begin_ferry()` on every new
crossing (`voyage.rs:544`), so anything that must persist *through* the
Dock-to-jump boundary (the charge timer) would need to be threaded through
`begin_ferry()`'s constructor anyway. Keeping it on `ColonyState`, which
already has exactly this "survives voyage replacement" property, avoids that
plumbing and matches the file's own stated ownership boundary.

### Decision 2: New `VoyageView::Dock`, `Jump` replaces `SailAgain`

Add `VoyageView::Dock` (`mod.rs`) rendered whenever `colony.dock.is_some()`
in place of leaving the player on the arrival `Record`/`Chart` view with an
instantly-available `N`. The existing `Reckoning` view (Drive/Shipwright/Ward
purchases) and `Record` view stay reachable from Dock exactly as today — Dock
adds a screen, it doesn't remove access to the yards.

`voyage_input.rs`: replace the `KeyCode::Char('n' | 'N') if voyage.arrived()`
arm's result from `SailAgain` to a new `VoyageInputResult::Jump`, available
only from `VoyageView::Dock`. `main.rs`'s handler for `Jump` reads
`colony.riftglass_charge(Utc::now())`, clears `colony.dock = None`, and calls
`begin_ferry()` (signature gains a `charge: f64` parameter — see Decision 4)
instead of the current unconditional call.

**Alternative considered**: keep the key binding and semantics of
`SailAgain` and just gate it behind a minimum elapsed time. Rejected — this
gives the player no visible charge feedback (a hidden timer is not "legible
at a glance," the same bar the exploration's own legacy-signal test applies)
and doesn't support the partial-charge trade-off, which requires exposing a
continuous value the player can act on early.

### Decision 3: Rate modifier comes from Drive level only, not a District

`riftglass_rate_mult()` reads only `self.drive_time_mult()` (already inverted
— faster ship, faster rate): `riftglass_rate_mult = 1.0 / drive_time_mult()`,
i.e. the same multiplier Drive already grants to crossing speed, reused
verbatim for charge speed. This needs no new content and is thematically
consistent — "the yard that makes the ship faster also punches the rift
faster."

**Alternative considered** (closer to the exploration's original phrasing,
"a district built for it... charges the rift faster"): introduce a 7th
District specifically for Riftglass. Rejected for this proposal's scope —
today's Districts are populaton-gated, flat-bonus, and mechanically
undifferentiated (see Context); making one of them do something functionally
different (modify a rate rather than add flat capacity) is the first crack
in that pattern and deserves its own design pass, not a rider on this
change. Revisit once/if a Districts-differentiation change is proposed
separately — `riftglass_rate_mult()` is a single function, trivial to extend
with a district term later without touching its call sites.

### Decision 4: `begin_ferry()` takes a `charge: f64`; a partial charge pre-applies a provisions/hull-wear deficit

Resolved (product direction): a partial-charge jump costs the next crossing a
deterministic **provisions deficit and hull-wear penalty**, scaled linearly
by how far short of full the charge was — candidates (a) off-course routing
and (c) guaranteed threat/weather (evaluated and rejected below) are not
built in this change.

```rust
pub fn begin_ferry(
    character_id: String,
    voyage_seed: u64,
    now: DateTime<Utc>,
    colony: &ColonyState,
    crew: Vec<SoulState>,
    charge: f64, // 0.0..=1.0, capped (Riftglass never overcharges)
) -> Self
```

Inside `begin_ferry()`, after the normal full-provisions/zero-wear
construction:

```rust
let deficit = 1.0 - charge.clamp(0.0, 1.0);
v.provisions = (v.provisions - MAX_PARTIAL_CHARGE_PROVISIONS_DEFICIT * deficit).max(0.0);
v.hull_wear = (MAX_PARTIAL_CHARGE_HULL_WEAR as f64 * deficit).round() as u8;
```

Starting constants (subject to `voyage_simulator` tuning, tasks.md section
7, before treated as final):
- `MAX_PARTIAL_CHARGE_PROVISIONS_DEFICIT = 40.0` — at charge 0.0 (an
  immediate jump with no Dock time at all), the hold starts 40 of its 100
  cap short, roughly 1.5× the drift-recovery affordability floor
  (`DRIFT_RECOVERY_PROVISIONS = 25`) — noticeable but not an instant Drift.
- `MAX_PARTIAL_CHARGE_HULL_WEAR = 3` — at charge 0.0, the crossing starts
  already halfway up the existing 0..=6 (`HULL_WEAR_MAX`) scar scale, each
  point adding 5% provisions burn (`WEAR_BURN_PER_SCAR`) — compounds with
  the provisions deficit rather than being a separate, unrelated cost axis.

At `charge = 1.0`, `deficit = 0.0` and both terms vanish — the crossing
begins exactly as `begin_ferry()` does today (the "no penalty at full
charge" scenario in the delta spec).

**Alternatives considered and rejected** (see prior draft's evaluation,
preserved here for the design record):
- **Off-course landing / starts further back on the route DAG.** Doesn't
  map cleanly onto the current code: every ferry crossing already begins at
  `ROUTE_START` (`begin_ferry()` calls `begin()` first), so there is no
  "further back" waypoint to place the ship at without authoring new
  pre-Last-Harbor route content — real new scope this change doesn't take
  on. The provisions/hull-wear deficit above delivers the same "you left in
  a hurry and it shows" feeling without needing new route content.
- **Guaranteed minor threat / worse starting weather.** Weather is a pure
  function of `(voyage_seed, hour)` (`weather.rs`) with no stored state;
  biasing it by charge level would compromise a purity property that
  offline-equivalence elsewhere depends on, and threat ledgers aren't
  modeled as an on-arrival guaranteed event today. Left as a possible later
  layer once threat/weather content exists to bias, not part of this change.

## Risks / Trade-offs

- **[Risk] Balance drift.** Dock time adds real-world duration to every
  ferry cycle, and the ferry loop is tuned to ~19–24 crossings / ~88% saved /
  C1 ≈ 14 real-days (`src/vessel/CLAUDE.md`). If `RIFTGLASS_BASE_HOURS_TO_FULL`
  is set too high, the era could blow well past its ~3-real-month target.
  → **Mitigation**: `voyage_simulator` (tasks.md) must model Dock time
  explicitly per strategy and the balance numbers in `act2-pilgrimage.md`
  re-validated before this ships live (it's still behind the kill-switch, so
  "before this ships live" means before flipping `ACT2_ENABLED`, not before
  merging — but `--check-progression` and the simulator should still model it
  from day one so the numbers aren't a surprise later).
- **[Risk] Save compatibility.** `ColonyState` gains a new `Option<DockState>`
  field. → **Mitigation**: `#[serde(default)]` (defaults to `None`), verified
  by `save_compat_tests` against the committed fixture corpus — an existing
  save simply has no Dock in progress on load, which is correct (it wasn't
  docked when it was saved under the old code either).
- **[Trade-off] Losing "Sail Again" as an instant action removes a
  zero-friction path some players may have relied on for fast-forwarding
  through the ferry loop.** This is the intended trade — it's the exact
  mechanism answering "one choice per ~3 real days" — but it does mean a
  player who wants to idle through many crossings quickly now needs the
  100%-charge patient path every time rather than an instant chain. Full
  charge is designed to still be the *safe, standard* option, not a
  punishment, so this should read as "there's now a wait" rather than "the
  fast option got worse."

## Open Questions

All four questions from the prior draft are resolved by product direction;
kept here as a record of the resolution rather than removed outright:

1. ~~Partial-charge risk shape~~ — **Resolved**: provisions/hull-wear
   deficit (Decision 4). The route-DAG and guaranteed-threat/weather
   alternatives were evaluated and explicitly rejected for this change (see
   Decision 4's "Alternatives considered and rejected").
2. ~~Does Riftglass cap at 100% or decay if left overcharged?~~ —
   **Resolved**: caps at 1.0 (Decision 1/4). No decay-past-full mechanic in
   this change.
3. ~~Should `riftglass_rate_mult()` anticipate session 5's braid?~~ —
   **Resolved: no.** Reads only `drive_time_mult()` (Decision 3). No
   forward-compatibility shim added; extending it later with a district or
   ship-tier term is a one-function change if session 5 ships.
4. ~~Riftglass naming~~ — **Resolved: keep "Riftglass."**

**Remaining before this is fully final** (not blocking `/opsx:apply`, but
tracked so it isn't forgotten): the exact magnitudes —
`RIFTGLASS_BASE_HOURS_TO_FULL = 24.0`,
`MAX_PARTIAL_CHARGE_PROVISIONS_DEFICIT = 40.0`, and
`MAX_PARTIAL_CHARGE_HULL_WEAR = 3` — are starting values chosen for
plausibility, not simulator-validated. Tasks.md section 7 runs
`voyage_simulator` against them and adjusts if they push the ferry loop
outside its tuned ~19–24 crossing / ~88% saved / ~3-real-month envelope.

## Verification

- `cargo test --test save_compat_tests` — the new `Option<DockState>` field
  must not break loading the committed save corpus.
- Targeted: `cargo test` (unit tests for `ColonyState::riftglass_charge` /
  `riftglass_rate_mult`, `VoyageState::begin_ferry`'s new `charge` parameter,
  and `voyage_input.rs`'s `Jump` handling — extend `input::replay_tests` for
  the new key path per the root CLAUDE.md's Keyboard Input row).
- `cargo test overlay_snapshot` / `cargo test snapshot` — the new
  `VoyageView::Dock` render needs a snapshot; re-bless with
  `INSTA_UPDATE=always cargo test snapshot` and review the diff.
- `cargo run --bin voyage_simulator -- --runs <n> --strategy <profile>` —
  extend the simulator to model Dock time and a charge policy per strategy
  (full-charge-always vs. jump-early), asserting every strategy still
  completes and reporting era length / percent saved so the balance risk
  above is caught before merge, not after.
- `cargo run --release --bin simulator -- --check-progression` — the
  endgame-systems scenario reaches Act 2 in some seeds; confirm it still
  passes with Dock time in the loop.
- `make check` before pushing, per repository convention.
