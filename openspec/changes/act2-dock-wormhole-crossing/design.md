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
- Deciding the exact partial-charge risk formula in this document — one
  open question below is left genuinely open per explicit product direction,
  with candidates and their implementation cost laid out so a follow-up
  design pass (or a quick decision before `/opsx:apply`) can resolve it
  cheaply.

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
    pub fn riftglass_charge(&self, now: DateTime<Utc>) -> f64 {
        let Some(dock) = &self.dock else { return 0.0 };
        let elapsed_hours = (now - dock.docked_at).num_seconds() as f64 / 3600.0;
        (elapsed_hours / self.riftglass_hours_to_full()).min(1.0) // see Open Question 2 re: overcharge
    }
}
```

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

### Decision 4: `begin_ferry()` takes a `charge: f64` and applies the (TBD) deficit

```rust
pub fn begin_ferry(
    character_id: String,
    voyage_seed: u64,
    now: DateTime<Utc>,
    colony: &ColonyState,
    crew: Vec<SoulState>,
    charge: f64, // 0.0..=1.0 (or higher if overcharge is allowed, see Open Q2)
) -> Self
```

The deficit application is isolated to one call inside `begin_ferry()` (e.g.
`v.apply_partial_charge_penalty(charge)`) so resolving Open Question 1 later
is a localized change, not a re-plumb.

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

1. **Partial-charge risk shape (the proposal's central unresolved
   question).** Candidates from `docs/explorations/2026-07-05-act2-systems-
   braiding.md` Session 6, each evaluated for implementation fit against the
   current code:
   - **(a) Off-course landing / starts further back on the route DAG.**
     Doesn't map cleanly: every ferry crossing already begins at
     `ROUTE_START` (`begin_ferry()` calls `begin()` first), so there is no
     "further back" waypoint to place the ship at. Implementing this
     faithfully would mean either extending `route.rs` with pre-Last-Harbor
     content (real new authored/graph work) or simulating "lost time" as a
     pre-applied time debt (e.g. `processed_minutes` starts negative or
     `provisions` starts pre-burned by an amount scaling with the deficit) —
     which is really candidate (b) wearing candidate (a)'s name. Recommend
     folding (a) into (b) unless new route content is explicitly wanted.
   - **(b) Provisions deficit / hull wear pre-applied.** Cheapest to
     implement: `begin_ferry()` already sets `provisions`/`hull_wear` at
     construction; scaling a deduction by `(1.0 - charge)` is a few lines
     with no new content. Directly deterministic, per the "no dice
     anywhere" pillar.
   - **(c) Guaranteed minor threat / worse starting weather.** Weather is
     already a pure function of `(voyage_seed, hour)` (`weather.rs`) with no
     stored state — "guaranteed worse weather" would mean biasing that pure
     function by charge level, which is a bigger change to a function
     whose purity is load-bearing elsewhere (offline-equivalence). Threat
     ledgers (Ossuary Warden/Silence/Thorns, per the exploration's
     Challenges row) aren't modeled as an on-arrival guaranteed event in
     `voyage.rs` today — would need new content, not just a parameter.
   - **Recommendation (not a decision — user explicitly deferred this):**
     (b) is the lowest-risk, most legible starting point (a single
     deterministic deduction, reusing existing `provisions`/`hull_wear`
     fields, no new route or weather content) and can be layered with (c)
     later once threat/weather content exists to bias. Resolve before
     `/opsx:apply` reaches the `begin_ferry()` task, since `tasks.md` needs a
     concrete formula to implement against.
2. **Does Riftglass cap at 100% or decay if left overcharged?** Both are
   expressible as pure functions of elapsed time (see Decision 1's
   `riftglass_charge()` — a capped version is `.min(1.0)`; a decay-past-full
   version is a piecewise function of `elapsed_hours` past
   `riftglass_hours_to_full()`), so this doesn't change the architecture,
   only the formula body. Leaning toward **cap at 1.0** for the first cut —
   simpler to reason about and to render as a bar — with decay as a possible
   follow-up if playtesting shows no reason to ever jump before max.
3. **Should `riftglass_rate_mult()` anticipate session 5's (unbuilt)
   ship/district braid?** No — per the standalone-scope decision, it reads
   only `drive_time_mult()` today (Decision 3). No forward-compatibility
   shim is added; if session 5 ships later, extending
   `riftglass_rate_mult()` with a district or ship-tier term is a one-
   function change.
4. **Riftglass naming.** "Riftglass" is the exploration doc's placeholder.
   Keep it unless a better name surfaces during `/opsx:apply` — not worth
   blocking implementation on.

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
