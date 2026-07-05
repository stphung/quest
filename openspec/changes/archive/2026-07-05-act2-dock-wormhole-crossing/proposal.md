## Why

The Ferryman era (ferry crossings 2+) is structurally one dimension: `sail →
Salvage → spend on {Drive | Shipwright | Ward} → repeat`, with Districts and
World Milestones as passive discovery axes rather than choices. Today's "Sail
Again" (`VoyageInputResult::SailAgain`, `voyage_input.rs`) is instantaneous —
the moment the player presses `N` at arrival, `VoyageState::begin_ferry()`
starts the next crossing with no intervening phase. Act 2's own Fun
Assessment (`docs/dossiers/act2-pilgrimage.md`) flags this directly: "ferry
runs: one choice per ~3 real days." There's no active management moment
between crossings, and the fully-hands-off ferry-run automation (auto
junctions, auto-skipped refit doors, auto-launch from the pier — see the
`vessel-act2` spec's "Maiden Voyage And Ferry Run Automation" requirement)
means the player experiences a ferry run passively even during the crossing
itself.

This proposal (`docs/explorations/2026-07-05-act2-systems-braiding.md`,
Session 6) gives the era a second decision point: a real-time "Dock" phase at
the Colony between crossings, capped off by a deliberate, one-way "wormhole
jump" whose timing is a genuine risk/reward call. It answers the exploration's
named gap without reopening Act 1 as an input — this is Act 2 systems (Dock
time, a new resource, the existing yards) feeding Act 2 systems, exactly the
corrected direction from the exploration's Session 5.

## What Changes

- Replace the instant "Sail Again" transition (crossings 2+ only; the maiden
  voyage's authored outbound leg is untouched) with a **Dock** phase: after a
  ferry crossing's arrival scene finishes, the player lands at the Colony in
  an active management view instead of being one keypress from the next
  crossing.
- Add a new resource, **Riftglass**, that accrues purely from real time spent
  docked (the mirror image of provisions accruing from time spent sailing) —
  not a recipe, not gated behind any district's existence. Existing yard
  levels and Districts may modify the *accrual rate* (exact modifiers are a
  design.md decision).
- Add a **wormhole jump** action, available once Riftglass has begun
  accruing: a one-way, no-undo commitment that ends the Dock phase and starts
  the next ferry crossing via the existing return-crossing machinery
  (route/phases/pace/weather/threats in `voyage.rs`, unchanged).
  - At **100% Riftglass charge**, the jump is the safe/patient option: the
    crossing starts exactly as ferry crossings do today (standard frontier
    point, full order).
  - At a **partial charge**, the jump trades less Dock time for a **harder**
    crossing. The exact mechanic is an open design question (see design.md)
    — candidates are an off-course start further back on the route DAG, a
    provisions/hull-wear deficit applied before the first league, or a
    guaranteed minor threat/weather roll on arrival. Whatever is chosen must
    be a deterministic function of charge level (no RNG-dressed-as-risk —
    the game's "no dice anywhere" pillar applies).
- **BREAKING (save-visible)**: `ColonyState` and/or `VoyageState` gain new
  persistent fields (Riftglass charge, Dock-phase state) — additive with
  `#[serde(default)]`, so existing saves load unaffected, but the ferry loop's
  behavior changes for any character with `crossings_completed >= 1` the next
  time they finish a crossing.

### Non-goals

- **No session-5 braid.** This proposal does not build veteran souls,
  Refinement, Star-Cord/Ironbound Timber/Broadwood, or ship-tier↔district
  mutual gating. It builds Dock/Wormhole/Riftglass standalone, on top of
  today's existing Drive/Shipwright/Ward yards and today's auto-founded
  Districts, exactly as scoped.
- **No change to the maiden voyage.** Crossing 1's authored outbound leg
  (souls, refit doors, threats, letters, decision-holding junctions) is
  untouched; Dock/Wormhole applies from the first arrival onward (ferry runs
  2+ only), per the exploration's Session 6 scope resolution.
- **No change to the Act 2 kill-switch.** Still dark by default behind
  `vessel::ACT2_ENABLED` / `QUEST_ACT2=1`.
- **No change to Districts as auto-founded-by-population** or to the
  Drive/Shipwright/Ward cost curves themselves — Riftglass is a new resource
  alongside Salvage, not a replacement.

## Capabilities

### New Capabilities

(none — this extends the existing Act 2 capability rather than introducing a
new one)

### Modified Capabilities

- `vessel-act2`: the "Maiden Voyage And Ferry Run Automation" requirement
  changes — a ferry run's return to the Colony no longer auto-transitions
  straight into the next crossing; it now enters a player-managed Dock phase
  gated by a new Riftglass charge-and-jump mechanic before the next crossing
  begins. The "Colony Ferry Loop Persistence" requirement gains a new
  resource (Riftglass) and a new Dock/wormhole-jump mechanic alongside the
  existing Salvage/yards economy.

## Impact

- **Code**: `src/vessel/colony.rs` (`ColonyState` gains Riftglass fields and
  accrual/jump methods), `src/vessel/voyage.rs` (`VoyagePhase` or a sibling
  state needs a Docked phase distinct from `HoldingStation`; `begin_ferry()`
  gains a charge-level parameter for the partial-charge deficit), `src/main.rs`
  (replaces the direct `SailAgain` → `begin_ferry()` wiring with Dock-phase
  entry/exit), `src/input/voyage_input.rs` (new input handling for the Dock
  view and the jump commitment), `src/ui/voyage_scene.rs` (new Dock-phase
  render), `src/vessel/persistence.rs` (Riftglass/Dock state persists in
  `colony.json` and/or `voyage.json`).
- **Balance**: touches the tuned ~19–24 crossing / ~88% saved / C1 ≈ 14
  real-days ferry-loop pacing documented in `src/vessel/CLAUDE.md` — Dock time
  is additional real time per crossing cycle, so the `voyage_simulator` and
  the balance numbers in `act2-pilgrimage.md` need re-validation once the
  charge rate and jump-risk numbers are set (see design.md and tasks.md).
- **Save compatibility**: new fields must be `#[serde(default)]` so
  `tests/fixtures/saves/` and `save_compat_tests` keep passing unmodified.
- **Docs**: `src/vessel/CLAUDE.md`'s Colony section and
  `docs/dossiers/act2-pilgrimage.md`'s Fun Assessment ("ferry runs: one choice
  per ~3 real days") need updating once this ships.
