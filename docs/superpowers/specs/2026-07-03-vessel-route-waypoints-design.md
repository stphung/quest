# The Route & the Waypoints

**Parent spec:** [2026-03-27-the-vessel-design.md](2026-03-27-the-vessel-design.md)
**Sub-project:** 2 of 7 — the load-bearing spec: every other Act 2 system
runs on the structures defined here.
**Companion:** [2026-07-03-vessel-underway-design.md](2026-07-03-vessel-underway-design.md)
(what happens *on* a road); spec 3 (Souls), spec 4 (Arrival Scenes) plug
into slots this spec defines.

## Overview

Act 2's world is a **static authored route graph** — waypoints and roads —
plus a **voyage state machine** that moves one ship through it on wall-clock
time. This spec defines the graph's data model and authoring rules, the
state machine and its transitions, road pricing, fog-of-route and rumors,
junction mechanics, drift, and the chart renderer. It also defines the
integration seams: how Act 2 boots after the launch burn, which Act 1
systems keep ticking, and where scenes/souls/travel plug in.

## The Route Graph

### Data model

Authored static data, Rust source like `zones/data.rs` (v1 has one route;
the graph is data so a second crossing could exist someday without new code).

```rust
pub struct Waypoint {
    pub id: WaypointId,              // stable u16
    pub name: &'static str,          // "The Drowned Choir"
    pub chapter: Chapter,            // I..IV
    pub chart_pos: (u16, u16),       // authored position on the virtual chart canvas
    pub scene: SceneRef,             // slot filled by spec 4 content
    pub features: &'static [Feature] // Harbor, Shipyard, RestStop, WayStation, LanternSite
}

pub struct Road {
    pub id: RoadId,
    pub from: WaypointId,
    pub to: WaypointId,
    pub base_days: f32,              // 1.0..3.0 (real days at Cruise, calm)
    pub base_provisions: u16,        // 12..55, chapter-ramped
    pub character: &'static [RoadTag], // Hungry, Lucky, Tolled, Singing, Dark, Kind…
    pub blurb: &'static str,         // one junction-card line, always shown
    pub known_stops: &'static [WaypointId], // shown on the card without rumors
    pub threat: Option<ThreatRef>,   // named threat living on this road (spec 4 scene)
}
```

A **junction** is any waypoint with more than one outgoing road. Nothing
else is special about it — the state machine treats "depart" with one road
as an automatic choice.

### Topology rules (authoring contract)

- **Spine-and-diamond**: branches split at a junction and **rejoin within
  the same chapter** (2–4 waypoints per branch). No branch crosses a
  chapter boundary; every chapter ends at a single gateway waypoint.
  This bounds authoring (branch content is scoped) and guarantees the
  chapter set-pieces (Going-Dark, finale) are on every route.
- **Acyclic, single sink**: the graph is a DAG; every road leads toward the
  Tree; waypoint 24 (the Roots of Light gateway) is the only sink. CI
  asserts this.
- Sizes for v1: **~24 waypoints on any taken path**, **~38 authored
  waypoints total** (untaken branches are real content, authored but
  possibly never seen), **7 junctions** (2 / 2 / 2 / 1 by chapter),
  **~46 roads**.
- **Content parity** (from No Right Path): every maximal route passes
  ≥ 5 soul-candidate scenes, ≥ 2 shipyards, ≥ 2 rest stops. CI asserts
  this by walking all routes.
- **Affordability invariant**: at every junction, the cheapest outgoing
  road costs ≤ `DRIFT_RECOVERY_PROVISIONS` (25). Combined with drift
  recovery, no state can strand the voyage. CI asserts this.

## The Voyage State Machine

```
                 depart (player, from junction card)
   ┌─────────────────────────────────────────────────┐
   ▼                                                  │
Traveling { road, departed_at, trim }                 │
   │ wall-clock: eta reached                          │
   ▼                                                  │
HoldingStation { waypoint, arrived_at,                │
                 scene_state: Waiting|Played }        │
   │ player plays the arrival scene (spec 4)          │
   ├── waypoint is chapter gateway → chapter beat     │
   └── player opens the junction card ────────────────┘

Traveling ── provisions hit 0 ──▶ Drifting { road, progress, since }
Drifting  ── recovery timer + scene ──▶ Traveling (resumed, +25 provisions)

HoldingStation at waypoint 24 ──▶ Arrived (finale sequence, spec 7)
```

- **Transitions on wall-clock are computed lazily** (on tick/load), exactly
  like the Loom's timers — no background scheduling. `game_tick` in Act 2
  mode calls `voyage::tick(now)` which advances Traveling→HoldingStation,
  applies per-day drains, steps weather/nights (Underway spec), and appends
  log entries.
- **Arrivals wait**: `HoldingStation.scene_state == Waiting` blocks
  departure; nothing else advances the route. Soft pressure: after 3 full
  days holding, hope decays 1/day (never below "steady" — the eager-souls
  rule, resolving the parent spec's open question).
- **Drift**: entering Drifting fires a log entry; recovery is
  `DRIFT_RECOVERY_HOURS = 36` of wall-clock, then a recovery scene
  (authored per chapter, spec 4) plays on next open; resume with
  provisions = 25. The covenant: drift never touches souls.

### Time model

Wall-clock via `chrono`, same rules as the Loom (Chrono Surge does not
accelerate the voyage; debug time-warp does). `day_index =
(now - launched_at).days` — the seed input for the Underway spec's
determinism. All durations honor a `VOYAGE_TIME_SCALE` dev/test multiplier
(default 1.0; the simulator and drive-game fixtures set e.g. 1440× so a
"day" is a minute).

## Road Pricing and the Junction Card

The junction card is **computed display data** — one struct the UI renders
and tests snapshot:

```rust
pub struct RoadCard {
    pub name: &'static str,
    pub stops: Vec<StopLine>,      // known_stops ∪ rumor-revealed stops, with notes
    pub days_estimate: RangeLabel, // base_days × current trim × visible weather
    pub provisions_price: u16,     // same composition, rounded, final number
    pub annotations: Vec<Annotation>, // rumor lines, soul counsel, refit effects
    pub affordable: bool,          // price ≤ current provisions
}
```

- Prices shown are **final computed integers** (Underway's rule). The base
  price is the road's promise; trim/weather/refits are how the player beats
  or worsens it.
- **Soul counsel**: souls with relevant dossier lines contribute one
  annotation each ("Torvald reads the Teeth: passable"). Free information,
  keyed to who's aboard — spec 3 provides the lookup.
- Unaffordable roads render locked with the price in red — visible, never
  selectable (the affordability invariant guarantees at least one open
  road).
- **Committing** marks sibling roads `Untaken` permanently: they render
  grayed with names forever, and their `known_stops` are never expanded
  further (No Right Path rule 3 — names, not contents).

## Fog of Route and Rumors

- **Visibility horizon**: the current road, the arrival waypoint, and —
  once at a junction — each outgoing road's card. Beyond that: chapter
  names and unlabeled `◌` marks only.
- ```rust
  pub struct Rumor {
      pub id: RumorId,
      pub text: &'static str,           // the line the player reads
      pub subject: RumorSubject,        // Road(RoadId) | Place(WaypointId) | Weather(WeatherRef)
      pub learned_at: WaypointId,       // provenance, shown in the rumor list
  }
  ```
- Rumors are **held forever** (weather rumors render struck-through once
  their weather dissipates — same inventory, aging display; resolves the
  Underway open question). Acquisition channels: arrival scenes (authored),
  pilgrim hails (Underway), way-station purchase (flat 6 provisions,
  feature-gated to `WayStation` waypoints, one per visit).
- Effect: a rumor whose subject is visible from the current junction adds
  its line to that road's card and may reveal a stop. Rumors about
  unreachable/passed subjects still display in the rumor list (flavor and
  foreshadowing) — they are never wasted inventory slots because there is
  no inventory limit.

## The Chart Renderer

- The route is drawn on a **virtual canvas** (authored `chart_pos`, roughly
  120×90 cells for v1); the viewport follows the Vessel with the Tree
  anchored top. Rendering is pure: `render_chart(voyage, route, weather,
  pilgrims, viewport) -> Buffer`, snapshot-tested at the standard tiers.
- Glyphs: `◉` visited · `○` known ahead · `◌` rumored/unknown · `✕` untaken
  (grayed, named) · `◆` the Vessel (pulsing) · `✦` pilgrim lights ·
  `☼` lit lanterns · `≋ ▒ ≈` weather (Underway spec owns their motion).
- **The Tree on the horizon**: 6 authored art stages selected by chapter ×
  progress-within-chapter; rendered at the top of the chart panel. Stage
  art lives with the route data. Never a percentage anywhere on screen.
- Small tiers (S/M) collapse to a linear strip (current road + next stop +
  tree stage glyph) — same information, one line.

## Booting Act 2 (mode routing)

Salvaged from the superseded mode-transition spec:

- `state.vessel_launched == true` routes `main.rs` into the Act 2 loop:
  Act 1's combat/zones/fishing/challenge stages are skipped; input maps to
  the chart surfaces (`[T]` trim, `[W]` watches, `[H]` hail, `[R]` rumors,
  `[S]` souls, `[L]` log, `[Enter]` context action).
- **First boot after the burn** plays the 5-beat transition (kept from the
  old spec), then `voyage::begin(now)` creates `VoyageState` at waypoint 0
  with the three launch souls and 100 provisions.
- **Act 1 keeps ticking in the background** for exactly one purpose:
  the Loom's WR→PR output funds the care packages until the Going-Dark
  (spec 6 owns the conversion). Combat, zones, discovery rolls: off.
- **Persistence**: `voyage.json` in the quest dir (Deep/Loom pattern),
  keyed by `character_id`, serde with defaults; `vessel_launched` stays on
  `GameState`. Save-compat corpus gains a mid-voyage fixture.
- **Fixtures**: `mkstate --voyage <waypoint> [--souls n] [--provisions n]
  [--day n]` writes a mid-crossing state; `VOYAGE_TIME_SCALE` makes legs
  minutes long for drive-game verification.

## Constants (first pass)

| Constant | Value | Note |
|----------|-------|------|
| `PROVISIONS_CAP` | 100 (150 with Long Hold) | one bar |
| `LAUNCH_PROVISIONS` | 100 | full hold |
| `ROAD_COST_RANGE` | 12–55 | chapter-ramped |
| `DRIFT_RECOVERY_PROVISIONS` | 25 | = affordability floor |
| `DRIFT_RECOVERY_HOURS` | 36 | |
| `HOLD_STATION_GRACE_DAYS` | 3 | then hope −1/day, floor "steady" |
| `RUMOR_PRICE` | 6 provisions | way-stations, one per visit |
| `BASE_DAYS_RANGE` | 1.0–3.0 | per road, at Cruise/calm |

Pacing check: ~24 waypoints × (≈1.8-day legs + ≈0.5-day holds) ≈ **165
days ≈ 5.5 months**, inside the parent's 5–8 month envelope with slack for
Mourn trims and drift.

## Testing

- **Graph invariants (CI)**: DAG/single-sink; branch rejoin within chapter;
  affordability at every junction; content parity walk (souls/shipyards/
  rest stops per maximal route); every `SceneRef` resolves.
- **State machine**: every transition, including drift entry mid-road,
  recovery resume at correct progress, hold-station hope grace, and
  arrival-waits blocking.
- **Offline equivalence**: N days lazy-ticked on load == N days ticked
  live (shared property test with the Underway spec, seeded).
- **Junction card**: pricing composition against trim/weather fixtures;
  unaffordable locking; rumor annotation attach/expiry; sibling graying is
  permanent across save/load.
- **Chart**: snapshot tests per size tier at fixed waypoints; tree stages;
  the untaken-roads render.
- **Simulator**: a `voyage_simulator` bin (Deep-simulator pattern) that
  plays random/greedy/kind strategies to the Tree and asserts: always
  arrives, day counts within envelope, no invariant violations — the
  balance gate for route authoring.

## Open Questions

- Whether the chapter gateway waypoints should be junction-free "breather"
  stops (lean: yes — set-pieces shouldn't share a screen with a choice).
- Chart canvas authoring ergonomics: hand-placed `chart_pos` vs a small
  layout tool; v1 lean: hand-placed, 38 waypoints is tractable.
- Whether rumors purchased at way-stations should be curated per station
  (authored relevance) or drawn nearest-first (lean: authored list per
  station, 2–3 each — it's ~30 lines of data and always relevant).
- Exact route content map (which branches hold which souls/shipyards/
  threats) — an authoring worksheet to produce *with* spec 4's scene list,
  not ahead of it.
