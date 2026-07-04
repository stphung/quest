# Vessel — Act 2 Launch Gate & Voyage

The dark-shipped Act 2: after Zone 50 falls, a signal from Yggdrasil is discovered; burning 250,000 Prestige Ranks launches the Vessel into a wall-clock crossing (the Voyage) toward the Tree. Fully implemented but invisible by default behind a compile-time kill-switch — see [The Act 2 Kill-Switch](#the-act-2-kill-switch).

## Files

| File | Purpose |
|------|---------|
| `mod.rs` | Kill-switch (`ACT2_ENABLED`, `act2_enabled()`), launch gate (`can_launch`, `perform_launch`), whisper rotation, `VoyageUiState`/`VoyageView`/`ScenePlay`/`SceneModal` (transient UI state) |
| `route.rs` | The static authored route graph: 38 waypoints, 45 roads, 8 rumors, 4 chapters, 7 junctions — a spine-and-diamond DAG from the Last Harbor to the Roots of Light |
| `voyage.rs` | `VoyageState` — the whole crossing's state machine: lazy wall-clock tick, phases (Traveling/Drifting/HoldingStation/Arrived), trim, the provisions gauge, arcs, mail, refits, night resolution. **Hope was retired** — the Ward yard (`colony.rs`) is Act 2's attrition lever now |
| `souls.rs` | The 8-person authored roster: stations (Helm/Tender/Watch), affinities, arcs (3 beats + resolution), junction counsel lines |
| `scenes.rs` | Arrival scene content: beats + state-conditioned color lines + payout, one `SceneDef` per waypoint; drift-recovery scenes per chapter; the finale |
| `weather.rs` | Void weather (Current/SilenceBank/Squall) — a pure function of `(voyage_seed, game-hour)`, no stored state |
| `nights.rs` | Night typing and outcomes (Quiet/Cold/Hungry/Singing/Strange), the watch-standing outcome matrix |
| `pilgrims.rs` | 5 authored pilgrim ships on cyclic scripted routes; hailing (once per ship) |
| `letters.rs` | 12 sequenced "Letters From Home" plus the Last Letter and the Going-Dark event |
| `refits.rs` | 3 permanent A/B refit door pairs (6 refits), offered at the first 3 distinct shipyards |
| `junction.rs` | `RoadCard` — computed display data for a junction's roads (final prices, stops, rumor annotations, soul counsel, affordability) |
| `persistence.rs` | `voyage.json` save/load, keyed by `character_id` (mirrors the Deep/Loom account-file pattern) |
| `colony.rs` | `ColonyState` — the ferry loop's persistent spine (`colony.json`): souls delivered/remaining, the two yard tracks (Drive/Shipwright levels) and their `Salvage` economy, districts, dimming order, records, era end. Survives every crossing where the voyage is replaced |

## The Act 2 Kill-Switch

`vessel::ACT2_ENABLED: bool = false` (in `mod.rs`) is the compile-time default. `vessel::act2_enabled()` is the runtime check used everywhere: it OR's the constant with the `QUEST_ACT2=1` environment variable, cached once in a `OnceLock` (so changing the env var mid-process has no effect).

While disabled: no discovery modal, no ticker whispers, no stats-panel row, no `[V]` hotkey, no path to the launch burn — the Vessel is fully invisible. **Zone 50 detection still sets `GameState::vessel_signal_discovered` in saves even when the kill-switch is off**, so already-qualified players light up the instant Act 2 is enabled later — this is why the flag-set in `tick_stages.rs` is *not* itself gated on `act2_enabled()`, only the events/UI that surface it are.

Enabling Act 2 for real is a one-line flip of `ACT2_ENABLED` to `true` (with a matching update to the release-guard test `act2_kill_switch_is_off_for_release` in `mod.rs`). For local preview/testing without recompiling, set `QUEST_ACT2=1`.

## Key Types

### `GameState` fields (`core/game_state.rs`, documented fully in `src/core/CLAUDE.md`)
- `vessel_signal_discovered: bool` — persistent; set the first time the Zone 50 final boss falls
- `vessel_launched: bool` — persistent; set by `perform_launch()`
- `vessel_arrived: bool` — persistent; set when the Voyage reaches the Tree (`VoyageState::take_finale_playback()`)
- `vessel_last_whisper_at: u64` — transient; play-time seconds when the last ticker whisper fired

### `VoyageUiState` / `VoyageView` / `SceneModal` (`mod.rs`)
Transient (not serialized) UI state for the Crossing screen — lives alongside the voyage like `DeepUiState` does for the Deep.
```rust
pub struct VoyageUiState {
    pub view: VoyageView,
    pub scene_play: Option<ScenePlay>,       // multi-beat arrival/finale scene being read
    pub scene_modal: Option<SceneModal>,     // one-line moment being read
    pub moments: VecDeque<SceneModal>,       // queued moments shown one at a time
}

pub enum VoyageView {
    Chart, Junction { selected }, Trim { selected }, Rumors,
    Souls { selected }, Watch { selected }, Farewell { selected },
    Manifest { scroll }, Keepsake { x, y }, Record { scroll }, // arrived-only, spec 7
}
```

### `VoyageState` (`voyage.rs`)
The whole crossing. Persisted to `voyage.json` via `persistence.rs`. Key fields: `phase: VoyagePhase`, `trim: Trim`, `provisions: f64` / `provisions_cap: f64`, `visited: Vec<WaypointId>`, `untaken: Vec<RoadId>`, `souls: Vec<SoulState>`, `refits: Vec<RefitId>`, `log: Vec<LogEntry>`, plus per-system bookkeeping (rumors, night assignments, letters, weather-derived silence tracking). (Legacy `hope`/`long_silence`/`hard_rations` fields on older saves are ignored on load — the hope system is retired.)

```rust
pub enum VoyagePhase {
    Traveling { road, departed_at_min, progress_days },
    Drifting { road, progress_days, since_min },      // hold ran dry mid-road
    HoldingStation { waypoint, arrived_at_min, scene_state, arrived_by },
    Arrived { at_min },                                // reached the Tree
}
```

Key methods: `begin()`, `tick(now)` (lazy wall-clock advance), `play_arrival_scene()`, `depart(road_id)`, `set_trim()`, `set_station()`, `accept_ask()` / `decline_ask()` / `farewell()` / `mark_lost()`, `buy_rumor()`, `hail()`, `take_finale_playback()`.

### `Trim` (`voyage.rs`)
The one posture dial (player-facing **Pace**): `Run`/Grueling (faster, hungrier), `Cruise`/Steady (default), `Quiet`/Easy (slower, thriftier, hears silence-banks), `Mourn`/Restful (slowest, **thriftiest hold** — 0.80× burn, its identity now that hope is retired). Composes into leg time and provisions burn alongside station bonuses — see `time_mult_with()` / `provisions_mult_with()`.

### `SoulState` / `SoulStatus` / `SoulId` (`voyage.rs` state, `souls.rs` content)
`SoulStatus`: `Aboard`, `Declined` (permanent), `Ashore` (farewelled, remembered), `Lost` (authored scenes only — `mark_lost()` is never called from tick-driven code, "the covenant in one sentence"). A soul's `arc_beat` advances through `SoulDef::arc` (3 beats + a resolution) as `rest_minutes` accumulates (`ARC_BEAT_REST_DAYS` days), gated by an `ArcTrigger` (`Aboard`, `ReachChapter`, `VisitFeature`, `VisitWaypoint`).

### `WaypointId` / `RoadId` / `RumorId` / `Chapter` / `Feature` / `Waypoint` / `Road` (`route.rs`)
Ordinal newtypes indexing static tables (`WAYPOINTS`, `ROADS`, `RUMORS`). `Chapter`: Shallows → DriftRoads → StarlessDeep → RootsOfLight, each ending at a single gateway waypoint. `Feature`: `Harbor`, `Shipyard`, `RestStop`, `WayStation`, `LanternSite`, `SoulCandidate`.

### `RoadCard` (`junction.rs`)
Fully-composed display data for one road out of a junction: final integer price at current trim, rounded days label, known + rumor-revealed stops, authored character tags, soul counsel lines, named threat (if any), `affordable`/`selectable`.

### `WeatherObj` / `WeatherKind` (`weather.rs`), `NightKind` / `NightOutcome` (`nights.rs`), `RefitId` / `RefitPair` (`refits.rs`), `PilgrimShip` (`pilgrims.rs`), `LetterDef` (`letters.rs`), `SceneDef` / `ColorKey` (`scenes.rs`)
Content/read-model types for each sub-system; see their file headers for the authoring rules (weather and nights are pure functions of `(voyage_seed, hour/day)` — no stored state, so offline resolution and live play always see identical results).

## How It Works

### The Launch Gate (sub-project 1)
1. **Discovery**: the first time the player defeats the Zone 50 final boss (`BossDefeatResult::LoomZoneCycle { zone_id: 50 }`), `tick_stages.rs::process_combat_events` sets `state.vessel_signal_discovered = true` and emits `TickEvent::VesselSignalDiscovered` — unconditionally, not gated on `act2_enabled()` (see kill-switch note above).
2. **Whispers**: once discovered and until launch, `tick_stages::tick_vessel_whispers()` (tick Stage 12c, gated on `act2_enabled()`) emits an atmospheric `TickEvent::VesselWhisper` roughly every `WHISPER_INTERVAL_SECONDS` (60s) of play time, rotating deterministically through `VESSEL_WHISPERS` by `whisper_message(index)`.
3. **Gate**: `can_launch(state, completed_patterns)` requires all four: signal discovered, not already launched, `ascension_level >= LAUNCH_REQUIRED_ASCENSION` (X), `completed_patterns >= LAUNCH_REQUIRED_PATTERNS` (28), and `prestige_rank >= LAUNCH_PR_COST` (250,000).
4. **Burn**: `perform_launch()` subtracts `LAUNCH_PR_COST` from `prestige_rank` in one all-or-nothing action, recalculates prestige bonuses, and sets `vessel_launched = true`. No partial spend — it's refused entirely if any gate is unmet.

### The Voyage (sub-project 2+)
Once `vessel_launched` (and `act2_enabled()`), `main.rs`'s game loop hands control to the Voyage: Act 1 systems idle untouched underneath while the Crossing screen owns rendering and input.

- **Lazy tick, whole game-minutes**: `VoyageState::tick(now)` computes elapsed game minutes since `launched_at` and steps `step_minute()` in a loop until caught up. This makes `tick(t2)` produce bitwise-identical state to `tick(t1); tick(t2)` — the "offline equivalence" property that lets long absences resolve exactly like live play. The clock is `GAME_MINUTES_PER_REAL_MINUTE = 2.64` (a sea-day passes in ~9 real hours), so the maiden voyage's ~37 sea-days is **about two real weeks**. The campaign's ramp is *earned*, not compressed: the clock never changes, only the Drive level shortens each crossing. Fixtures and tests express exact offsets through `real_duration_for_game_minutes()` so they are scale-agnostic.
- **Auto-sail (pacing)**: a mid-crossing port with no decision — exactly one road out, no recruit ask, no refit door — gets a `PORT_CALL_GAME_MINUTES` port call, then the ship sails herself; the arrival scene is played by the engine and queued in `unread_scenes` for the ferryman to read on return (drained one at a time by `main.rs`). On the **maiden voyage** (crossing 1) decisions hold the ship: junctions, asks, refit doors, and the pier (`arrived_by: None`). On a **ferry run** (crossing 2+, `crossing_number > 1`) the whole crossing is hands-off — she auto-navigates junctions too (taking the first road), skips refit doors, and launches herself from the pier — so the crossing completes autonomously in Drive-scaled time. The passenger load also stops deepening her provisions burn on ferry runs (`provisions_mult_with`): no one meters rations, so the crossing's length answers to Drive alone, not the headcount.
- **Route**: a spine-and-diamond DAG (`route.rs`) — branches split at 7 junctions and rejoin within the same chapter; each chapter ends at a single gateway waypoint; the Tree (`ROUTE_SINK`, waypoint 37) is the graph's only sink.
- **Gauges**: `provisions` (burn while traveling, composed from trim × tender-station × weather; running dry drifts in place). Speed = Drive × Pace × helm-station × Storm Sail. (Hope, the old "wind" gauge, is **retired** — see the note in the file table.)
- **Affordability invariant**: at every junction, the cheapest outgoing road never costs more than `DRIFT_RECOVERY_PROVISIONS` (25) — asserted in `route.rs` tests — so running the hold dry always means drifting in place (36-hour recovery), never getting stuck.
- **Souls**: recruit asks block departure until answered; stations (Helm/Tender/Watch) grant multipliers; arcs pay rumors (and fire as log moments) on a rest-day timer; farewelling frees a crew seat; loss is authored-scene-only.
- **Refits**: the first 3 distinct shipyards visited each offer one permanent A/B door (`REFIT_PAIRS`); picking one closes the other forever.
- **Weather & nights**: weather is a pure function of `(voyage_seed, hour)` so it never needs saving; nights are typed per day from `(voyage_seed, day)` and price provisions (and soul strain) based on who stands the watch.
- **Letters & the Going-Dark**: one letter per Chapter I/II arrival, delivered at ports (never on timers); the Threshold hands over the Last Letter; the first arrival past the Last Lantern is the night the mail does not come (`gone_dark`).
- **Finale**: `take_finale_playback()` fires once on arrival, setting `state.vessel_arrived = true` — the hook a future Act 3 would key off, the way Act 2 keys off `vessel_launched`.
- **Persistence**: `voyage.json` (via `persistence.rs`), keyed by `character_id` so a different character never inherits a crossing in progress.

## Integration Points

- **`core/tick.rs`**: Stage 12c calls `tick_stages::tick_vessel_whispers()`, gated on `crate::vessel::act2_enabled()`.
- **`core/tick_stages.rs`**: `process_combat_events()` sets `vessel_signal_discovered` and emits `TickEvent::VesselSignalDiscovered` on the first Zone 50 final-boss kill (unconditional); `tick_vessel_whispers()` emits `TickEvent::VesselWhisper`.
- **`core/tick_types.rs`**: `TickEvent::VesselSignalDiscovered`, `TickEvent::VesselWhisper { message }` variants.
- **`core/game_state.rs`**: `vessel_signal_discovered`, `vessel_launched`, `vessel_arrived` (persistent), `vessel_last_whisper_at` (transient) fields; round-tripped through `game_state_serde.rs`'s `FlatGameState` and `character/persistence.rs`.
- **`tick_events.rs`**: `apply_tick_events()` maps `VesselSignalDiscovered`/`VesselWhisper` to a combat-log entry and a `TickerEntry` push (both gated on `act2_enabled()`); returns `TickEventFlags::vessel_signal_discovered` for `main.rs` to act on.
- **`input/mod.rs`**: `[V]` hotkey opens `GameOverlay::Vessel { confirm_pending: false }` when `state.vessel_signal_discovered && vessel::act2_enabled()`. `handle_vessel_overlay()` (step 2.95 in the dispatch chain, before Vault/Prestige) handles Enter (open confirm / burn via `perform_launch()` / dismiss if already launched) and Esc (back out of confirm, then close).
- **`input/voyage_input.rs`**: `handle_voyage_input()` / `VoyageInputResult` (`Handled`, `HandledNeedsSave`, `Quit`, `Ignored`) — routed directly from `main.rs`'s Act 2 loop branch (not through `handle_game_input()`), since the Voyage replaces the whole screen once launched.
- **`main_helpers/overlay.rs`**: `draw_game_overlays()` renders `GameOverlay::Vessel` via `ui::vessel_scene::render_vessel_overlay()` and `GameOverlay::VesselDiscovery` via `ui::vessel_scene::render_vessel_discovery_modal()`.
- **`main.rs`**: two distinct wiring points — (1) `tick_flags.vessel_signal_discovered` pushes `GameOverlay::VesselDiscovery` onto the pending-overlay queue; (2) once `state.vessel_launched && vessel::act2_enabled()`, the `'game_loop` takes an early branch each iteration that ticks the voyage, drains soul/letter/recovery/finale events into `voyage_ui.moments`, renders `ui::voyage_scene::render_voyage()`, and routes keys through `voyage_input` instead of the normal `handle_game_input()` path — then `continue`s, skipping the rest of the Act 1 loop body entirely.
- **`ui/vessel_scene.rs`**: `render_vessel_discovery_modal()` (one-time celebration), `render_vessel_overlay()` (full-screen construction/launch-confirmation overlay).
- **`ui/voyage_scene.rs`**: `render_voyage()` — the Crossing main screen, dispatching on `VoyageUiState::view` to per-panel renderers (chart, junction, trim, rumors, souls, watch, farewell, manifest, keepsake chart, record); imports `vessel_scene::VESSEL_VIOLET` for consistent theming.
- **`ui/stats_panel.rs`**: adds a "Vessel signal" row to the hero panel height when `vessel_signal_discovered && act2_enabled()`.
- **`input/types.rs`**: `GameOverlay::VesselDiscovery` and `GameOverlay::Vessel { confirm_pending: bool }` variants.
- **`bin/mkstate.rs`**: fixture flags can set `vessel_signal_discovered` for drive-game / screenshot scenarios.

## Key Constants

### Launch Gate (`mod.rs`)
| Constant | Value | Notes |
|----------|-------|-------|
| `LAUNCH_PR_COST` | 250,000 | Burned in one all-or-nothing action |
| `LAUNCH_REQUIRED_PATTERNS` | 28 | All Woven Patterns (the complete Loom becomes the hull) |
| `LAUNCH_REQUIRED_ASCENSION` | 10 (Ascension X) | |
| `WHISPER_INTERVAL_SECONDS` | 60 | Play-time seconds between ticker whispers |

### Voyage (`voyage.rs`)
| Constant | Value | Notes |
|----------|-------|-------|
| `MINUTES_PER_DAY` | 1440 | Game minutes |
| `GAME_MINUTES_PER_REAL_MINUTE` | 2.64 | A sea-day ≈ 9 real hours; maiden voyage ≈ two real weeks; `QUEST_VOYAGE_TIME_SCALE` env overrides for dev |
| `PORT_CALL_GAME_MINUTES` | 360 | Hold at a no-decision port before auto-sail |
| `PROVISIONS_CAP` | 100.0 | 150.0 with the Long Hold refit |
| `LAUNCH_PROVISIONS` | 100.0 | Hold is full at launch |
| `DRIFT_RECOVERY_PROVISIONS` | 25 | Also the affordability floor (every junction's cheapest road) |
| `DRIFT_RECOVERY_HOURS` | 36 | |
| `RUMOR_PRICE` | 6.0 | Per way-station visit |

(The hope constants — `HOPE_MAX`, `HOPE_FLOOR_STEADY`, `LAUNCH_HOPE`, `HOPE_SPEND_FLOOR`, `PRESS_*`, `HARD_RATIONS_BURN_MULT`, `HOLD_STATION_GRACE_DAYS` — and the Press-the-helm / Rations levers were removed when hope was retired.)

### Colony / the ferry loop (`colony.rs`)

The crossing is the *run*; the Colony is what persists above it. Each crossing delivers souls (the headline number only ever rises), pays out **Salvage**, and the dark takes a toll of whoever still waits — **a share every day the crossing is underway** (`dark_toll_for_days(days)`), so a slow crossing bleeds longer and a fast one starves it. Three yards spend Salvage, and the choice between them is the loop's decision (`[D]`/`[C]`/`[W]` in the Reckoning):

- **Drive** (`drive_level`) — each level multiplies every future crossing's sail-time by `DRIVE_DECAY`, compounding down to `DRIVE_FLOOR`. Level 0 is the maiden voyage (the slowest crossing there is); the ramp is entirely earned by buying levels.
- **Shipwright** (`cap_level`) — each level multiplies the hold by `CAP_GROWTH`. `expedition_size()` = `BASE_CAPACITY × CAP_GROWTH^cap_level + district bonuses`.
- **Ward** (`ward_level`) — each level multiplies the dark's per-**day** rate by `WARD_DECAY`, compounding down to `WARD_TOLL_FLOOR` (never zero — a residual bite always remains). `dark_daily_rate()` = `DARK_TAKES_PER_DAY × WARD_DECAY^ward_level`; the toll over a crossing of `days` compounds it (`dark_toll_for_days`). The attrition axis, front-loaded: it spares the most while the world is full and the level is low, so it wants buying early.

The tension is speed-vs-salvation, a **wide** margin, and now *all three* yards bear on it because the toll is per-day: **Drive** (fewer days per crossing) and **Shipwright** (fewer crossings to empty the world) both cut the total days the world spends waiting, and the **Ward** softens the daily bite. A souls-first line (big hold, just enough Drive to stay quick) saves most of the world (**~88% saved**); leaning on the Ward pushes it to **~94%** at the cost of more crossings; both naive extremes are traps — pure Drive never widens the hold so the world stays full and bleeds (**~70%**), and pure Shipwright runs crossings so slow the dark eats them (**~74%**). Skill is rewarded, not marginal. The ramp is a **buildup then a fast-fun stretch**: a two-week maiden voyage (the costliest crossing there is — slowest, so the dark feeds longest), ~14 → ~3 real-days over the first handful, then ~3-day turnarounds while the loads climb into the thousands. Tuned to **~19–24 crossings, ~3 real months, ~88% saved with skilled play, C1 ≈ 14 real days**.

| Constant | Value | Notes |
|----------|-------|-------|
| `INITIAL_SOULS` | 100,000 | The dying world's pool; the era spends it down |
| `DRIVE_DECAY` / `DRIVE_FLOOR` | 0.70 / 0.05 | Sail-time ×0.70 per Drive level, floored at 0.05 (≈20× top speed) |
| `BASE_CAPACITY` / `CAP_GROWTH` | 180 / 1.36 | Hold at Shipwright 0; ×1.36 per level |
| `SALVAGE_AT_LANDFALL` / `SOULS_PER_SALVAGE` | 3 / 30 | Salvage earned = 3 + carried/30 per crossing |
| `STARTING_SALVAGE` | 40 | Founding grant — the ramp takes hold from the second crossing |
| `DRIVE_COST_BASE` / `DRIVE_COST_GROWTH` | 4 / 1.5 | Drive level L costs `4×1.5^L` Salvage |
| `CAP_COST_BASE` / `CAP_COST_GROWTH` | 5 / 1.42 | Shipwright level L costs `5×1.42^L` Salvage |
| `DARK_TAKES_PER_DAY` | 0.0006 | Base fraction of the still-waiting world the dark takes **each day** underway — compounds over a crossing's days; the toll the Ward buys down |
| `WARD_DECAY` / `WARD_TOLL_FLOOR` | 0.72 / 0.12 | Toll ×0.72 per Ward level (~28% off), floored at 0.12× base (≤88% cut, never zero) |
| `WARD_COST_BASE` / `WARD_COST_GROWTH` | 5 / 1.45 | Ward level L costs `5×1.45^L` Salvage — between Drive and Shipwright |
| districts | Quay 500 … Charthouse 66,000 | Founded by population (= souls delivered); each adds a standing hold bonus |

Buying is player-driven (`buy_drive`/`buy_capacity`/`buy_ward`, wired through `VoyageInputResult::BuyDrive`/`BuyCapacity`/`BuyWard` → `main.rs` → the colony); there is no in-game auto-invest (the balanced line lives only in the tests/sim as a policy helper).

### Souls (`souls.rs`)
| Constant | Value | Notes |
|----------|-------|-------|
| `CREW` | 7 | 8 souls total (3 launch + 5 found) compete for 7 |
| `ARC_BEAT_REST_DAYS` | 2 | Rest days before a ready beat fires |

### Route (`route.rs`)
38 waypoints, 45 roads, 8 rumors, 4 chapters, 7 junctions (2/2/2/1 by chapter). Every maximal route sees ~24 waypoints and passes >=5 soul-candidate scenes, >=2 shipyards, >=2 rest stops (content-parity tests). Road prices range 12-55 provisions (chapter-ramped); base days 1.0-3.0 at Cruise.

## Known Invariants and Gotchas

- **`ACT2_ENABLED` is `false` in every committed build.** `mod.rs::act2_kill_switch_is_off_for_release` fails the moment someone flips it without updating the test — treat a failure there as "did we mean to ship Act 2?", not a bug to route around.
- **Zone 50 detection is unconditional.** `vessel_signal_discovered` is set by combat processing regardless of `act2_enabled()`; only the *presentation* (log line, ticker, modal, hotkey) is gated. This is deliberate so flipping the switch later doesn't require replaying Zone 50.
- **`act2_enabled()` caches the env var once** (`OnceLock`) — setting `QUEST_ACT2` after the process starts has no effect.
- **`mark_lost()` is reachable only from authored scenes** (currently the Thorns threat ledger in `voyage.rs`) — no tick-driven path calls it; this is the game's only permanent-loss mechanic and it is intentionally never random.
- **Offline equivalence / chunking invariance is load-bearing.** `VoyageState::tick()` must produce identical results whether called once after a long gap or many times across small gaps (`tick_is_chunking_invariant` test). Any change to `step_minute()` must preserve this.
- **`voyage.json` is keyed by `character_id`**, not by save slot — loading a different character's save will not pick up a stale voyage file (`load_voyage` checks and discards on mismatch).
- **The launch-cost doc comment in `core/game_state.rs`** (on the `vessel_launched` field) currently says "100,000 PR"; the actual, tested, authoritative cost is `vessel::LAUNCH_PR_COST = 250_000` as used here and in root `CLAUDE.md`.
