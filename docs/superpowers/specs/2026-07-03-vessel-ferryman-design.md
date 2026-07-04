# The Ferryman — The Reckoning & the Colony

**Parent spec:** [2026-03-27-the-vessel-design.md](2026-03-27-the-vessel-design.md)
**Sub-project:** 9 — the loop. Elevates Act 2 from a played-through story
to an incremental with a story.
**Depends on:** specs 2–7 (shipped — the crossing is the run) and spec 8
(The Price of Passage — the run's texture; strain/wear/scarcity are what
make a careful crossing outperform a careless one).
**Feeds:** Act 3 — whose gate **moves**: not the first arrival, but the
**Last Crossing** (see below).

## Elevation

Act 2 as shipped is a beautiful single playthrough: ~34–49 days, one
manifest, done. That is a story, not an incremental. This spec makes the
crossing the **run** and adds the layer incrementals live on: a number
that only goes up, an engine that compounds, and a next run that starts
bigger than the last.

**The fiction:** the old branch is dying, but it is not empty. The
Vessel delivers her souls to the living branch — and goes back for more.
You become **the ferryman**. Every crossing is still a small authored
story with real stakes (spec 8); the game above the stories is the race:
**how much of the old world can you carry across before the light goes
out?** The Going-Dark stops being one sad beat and becomes the pressure
on your growing number.

## Pillar Amendments (deliberate, in the open)

- **"No resettable loop" (anti-goal)** — amended. The crossing loop *is*
  Act 2's identity now. Nothing resets that matters: the loop consumes a
  finite world and builds a permanent colony; it ends (Act 3), it does
  not cycle in place.
- **"Doors close"** — scoped per crossing. Roads untaken this run gray
  for this run; the *chart's knowledge* accumulates forever (visited
  waypoints stay named across crossings — exploration becomes long-run
  content: 38 waypoints, 96 routes to have sailed). Souls lost stay
  lost. Refit doors stay once-ever — the hull is the same hull.
- **"Never a percentage"** — holds for the journey (the chart, the Tree
  on the horizon). The new Reckoning pane is *allowed to count*: it is a
  ledger, and ledgers are numbers.

## The Loop

```
LAUNCH (once, Act 1 ends — unchanged)
  └─ CROSSING 1 — the authored pilgrimage, specs 2–8 exactly as built
       └─ ARRIVAL — finale, manifest, harbor (spec 7, unchanged)
            └─ THE RECKONING — souls delivered, colony founded
                 └─ THE RUNNING BACK — elided return, ~⅓ crossing time
                      └─ CROSSING 2..N — ferry runs (below)
                           └─ ... until THE LAST CROSSING → Act 3 gate
```

**Crossing 1 is untouched.** The eight authored souls, the arcs, the
letters, the Going-Dark, the finale. Its outcomes become permanently
load-bearing: souls who came ashore are your **crew** for every future
crossing (Torvald keeps the helm for the rest of the game); the carved
names stay carved on a hull that keeps sailing.

**Ferry runs (crossing 2+):** the hold carries **passengers by the
count** — "38 souls berthed" — not new authored casts. Ports have souls
waiting ("12 souls wait at Candlewick Light"); you embark up to
capacity. Junction cards annotate souls waiting on each road, and rumors
reveal counts further ahead — the rumor economy finally prices the thing
you actually want. Crew (the authored survivors) still man stations,
still strain (spec 8); passengers are cargo that eats: provisions burn
scales with load, so spec 8's scarcity becomes load management.

**The running back:** no decisions, no weather, empty hold, following
wind — a fixed timer (~⅓ of the crossing's days). Colony construction
resolves during it; one letter from the colony waits at the pier.

**Mail reverses:** the old world went dark (spec 6, permanent) — but as
the colony grows, *it* starts writing. District unlocks, births, the
harbor's first bell: Letters From the Colony reuse the letters machinery
wholesale, and the correspondence brightens as the old one darkened.

## The Race (entropy vs. capacity)

> **Doc-alignment note (2026-07-04):** this section and the two below it
> ("The Engine: Resonance", "The Colony") describe the *original* design for
> this sub-project. Both were superseded by the two follow-ups appended at
> the bottom of this file — first by the two-yard Drive/Shipwright rewrite
> (2026-07-04, "the two yards"), then by the shipped three-yard Ward
> redesign (commit d39ad67, addendum below, "the third yard"). Read the
> follow-ups as the authoritative design; the numbers below (Resonance,
> `souls_remaining` ~3,000, the original district table) are historical
> record only. `src/vessel/CLAUDE.md` is ground truth for what's shipped.

- **`souls_remaining`** — the dying world's finite pool (initial value
  ~3,000; tuned by probe — **shipped as `INITIAL_SOULS = 100,000`**,
  `colony.rs:21`). Visible from the first Reckoning.
- **The dimming** — on a deterministic schedule (seeded, per real day),
  harbors on the chart *go out*: a dimmed port has no souls waiting, and
  its glyph dims on the chart. Over the ferry era the player literally
  watches the old world go out, port by port, crossing by crossing.
  Souls at ports that dim before you reach them are lost to the dark —
  subtracted from `souls_remaining`, never from your tally.
- **Capacity grows** (colony Shipyard, below); the pool shrinks faster
  as the dark accelerates. Early: souls are everywhere, capacity is the
  limit. Late: capacity is huge, souls are scarce and far — route choice
  becomes triage.
- **The Last Crossing:** when `souls_remaining` hits zero, the next
  arrival is the end of the era — an authored finale (the empty harbor,
  the lamp, Sister Verity standing by the door). **This is Act 3's
  gate** (a new flag; `vessel_arrived` remains the record of the first
  arrival). Target era length: 2–4 months of real time at daily
  check-ins — comparable to a serious Act 1 — sim-gated, not hoped.

## The Engine: Resonance

The Loom resonated with the beacon all along; now the crossings do.
**Resonance** is a lifetime number that only rises:

- **Earned:** +1 per soul delivered; small bonuses for hope held bright
  at arrival, for first-time waypoints, for records broken.
- **Spends nothing. Multiplies everything:** sail speed and arrival
  provision yields scale as `1 + f(resonance)` with a soft cap (curve
  tuned by probe; intent: each crossing noticeably faster/richer than
  the last, ~2× by the era's end).
- Souls → Resonance → faster crossings → more souls: the compounding
  loop, with spec 8 deciding how much each crossing actually delivers.

## The Colony

**Population = souls delivered** (the headline number, minus nothing —
the colony keeps everyone). Districts unlock at population thresholds;
all of them, in order — the colony is pure growth; choices live on the
water:

| Population | District | Standing bonus |
|-----------:|----------|----------------|
| 25 | The Quay | the running back ⅓ → ¼ of crossing time |
| 60 | The Granary | provisions cap +25; way-stations pay +10% |
| 150 | The Hearth | launch hope 7 → 8; RestStops heal strain fully |
| 400 | The Shipyard | passenger capacity ×1.5; **Drydock**: mend hull at the Tree |
| 1000 | The Beacon | resonance earn rate ×1.5; one extra rumor held at launch |
| 2500 | The Charthouse | chart knowledge persists (fog lifts permanently); souls-waiting visible one junction further |

(Thresholds and effects are first-pass; the probe tunes them so each
district lands roughly every 2–4 crossings mid-era.)

**Persist vs. reset** — the incremental contract, explicit:

- **Persists forever:** population, districts, Resonance, lifetime
  tallies and records, crew roster and carved names, refits, hull wear
  (unless mended — a scarred ferry is a long-run cost), chart knowledge.
- **Fresh each crossing:** provisions, hope (7 + Hearth), per-run road
  closures, weather and nights (new seed per crossing — variety is
  free), passenger load. Crew strain heals at the colony: coming home
  is rest.

## The Reckoning (the pane)

A full-screen surface, `[L]` from the chart (and a harbor room after
arrivals). The numbers screen the act has never had, in the terminal-
incremental idiom — big counters, a rate line, thresholds that light:

```
┌ ☼ The Reckoning ────────────────────────────────────┐
│           SOULS CARRIED OUT OF THE DARK              │
│                    1 , 2 4 7                         │
│        souls remaining in the old world: 1,613       │
│                                                      │
│  Resonance 312 · the Vessel sails 1.6× her old self  │
│  this crossing: 44 aboard · 212 leagues · day 9      │
│                                                      │
│  THE COLONY — pop. 1,247                             │
│  ✓ Quay  ✓ Granary  ✓ Hearth  ✓ Shipyard             │
│  ▸ The Beacon at 1,000 ✓ · The Charthouse at 2,500   │
│                                                      │
│  RECORDS  fastest crossing 21d · most carried 61     │
│  9 crossings · 2,912 leagues · 148 nights stood      │
└──────────────────────────────────────────────────────┘
```

Live elements: the headline count ticks up on delivery; the rate line
pulses with the resonance multiplier; a district row lights the moment
its threshold passes (with its colony letter queued as a moment).

## Data Model (build scope)

```rust
// New: colony.json (account-adjacent, keyed like voyage.json)
pub struct ColonyState {
    souls_delivered: u64,          // the number
    souls_remaining: u64,          // the pressure
    resonance: u64,
    crossings_completed: u32,
    records: CrossingRecords,      // fastest, most carried, leagues, nights
    dimmed_ports: Vec<WaypointId>, // the world going out
    // districts derive from souls_delivered — never stored
}

// VoyageState — serde(default)
passengers: u32,                   // ferry runs; 0 on crossing 1
crossing_number: u32,              // 1 = the authored pilgrimage
returning_until_min: Option<u64>,  // the running back

// Waypoint pickup: souls_waiting(port, crossing, dimming) — pure fn
// Act 3 gate: last_crossing_complete (GameState, serde default)
```

Chunking-invariant and offline == live bitwise throughout, as ever; the
dimming schedule is a pure function of (era seed, day) like weather.

## What This Spec Does NOT Add

No new authored souls after crossing 1 (notable pilgrims are future
content). No colony management minigame — the colony builds itself;
choices stay on the water. No second currency beyond Resonance. No dice.
No Act 3 content beyond the Last Crossing gate. No prestige *reset* —
the loop consumes and builds, never rewinds.

## Testing / Sim Gates

- Ferry-era simulator: play eras end-to-end across strategies; assert
  the era completes (pool exhausts), era length lands in the target
  window at daily check-ins, and every district unlocks mid-era
- Resonance curve: crossing N+1 median days < crossing N (until the
  soft cap); final-era crossings ≈ 2× first-crossing speed
- Triage pressure exists: late-era crossings leave souls unreachable
  (dimming beats capacity) in at least some strategies — the race is
  real, sim-proven
- Crossing 1 byte-identical to spec 7 behavior (regression gate)
- Save compat: colony.json absent → pre-ferryman saves resume mid-
  crossing-1 cleanly; all new fields serde(default)
- The Reckoning pane snapshots (XL + strip) + double-render determinism

## Open Questions

- Whether passengers should have *any* texture (a manifest line naming
  a family per ~20 souls — flavor, no mechanics). Lean: yes, one line
  per embarkation, authored pool, purely cosmetic.
- Whether records should feed Resonance (lean: tiny one-time bonuses —
  records are for the shelf, not the engine).
- Whether the dimming should ever pause (mercy windows) — lean: no;
  the covenant protects *your* ship, not the world. The dark is the
  antagonist and it does not check in.
- Era seed: one per account (deterministic era) vs. per crossing —
  lean: per account, so the dimming order is *your* world's story.

## Follow-up shipped: the dimming render

The dimming *schedule* landed with the first Ferryman PR, but the chart did
not yet draw it — the deferred "port-by-port dimming visual." That render is
now in: on a ferry-era crossing the chart draws snuffed ports as a cold `⊘`
("gone dark") with their roads faded and a matching legend entry, while the
home pier, the Tree, and the lit path ahead still stand. The keepsake chart
never dims (it is a memento of one crossing, not a live map of the world).

Fixing the render surfaced a pacing bug it would otherwise have exposed: the
old schedule (`dimmed_as_of(crossings_completed × 35)` days against a
`port_dim_day` that maxed ~150) blacked the whole map out by ~crossing 5,
while the population takes the full ~28-crossing era to empty — a static
blackout for 80% of the era. Replaced with `dark_ports()`, which keeps
`port_dim_order` (this world's deterministic story, home-biased) but paces
the blackout to how empty the world actually is: the fraction of ports dark
tracks `1 − souls_remaining / INITIAL_SOULS`, so the chart empties in step
with the manifest. Purely cosmetic — no souls, resonance, capacity, or
routing changed.

## Follow-up shipped: fewer, weightier crossings

The first cut ran ~28 crossings, of which ~17 were mechanically identical
(deliver 40–60, dark takes ~50, repeat) and the top district was unreachable
(the dark took 53% of the world, capping delivery at ~1,421 of 3,000). The
"numbers go up" spine ran dry around crossing 11 while the era ground on for
another ~two real-world years. Rebalanced to **a short era of big, deliberate
crossings** — one district founded per crossing, the whole colony reached:

- **The colony grows the ship.** `ferry_capacity` is now the launch base plus
  every founded district's berths, so cohorts swell 270 → 410 → 580 → 790 as
  the colony does — the growth you watch is the size of each delivery.
- **The dark is a per-crossing bite,** not a per-day drip: `dark_toll` takes a
  fixed share (`DARK_TAKES_EACH_CROSSING`) of whoever is still waiting — hard
  while the world is full (you're losing the race), easing as it empties (you
  carry the last of them home yourself).
- **Result (sim-proven, `run_era`):** 6 crossings, 36 → 32 days each,
  ~2,400 of 3,000 saved (80%), one district per crossing, the Charthouse
  landing on the finale. `RESONANCE_FOR_HALF_SPEEDUP` raised to 2,500 so a
  short era's crossings stay weighty rather than snapping to the speed floor;
  `PROVISIONS_PER_PASSENGER` lowered so cohorts of hundreds still make the
  crossing.

The tuning knobs were also renamed for legibility — `FERRY_BERTHS_AT_LAUNCH`,
`District::added_berths` / `District::founded_at`, `DARK_TAKES_EACH_CROSSING`,
`RESONANCE_FOR_HALF_SPEEDUP`, `FASTEST_CROSSING_TIME_MULT`,
`PROVISIONS_PER_PASSENGER` — so the era's shape reads without a decoder.

## Follow-up shipped: the three-month campaign (scale, ramp, auto-sail)

The big-world pass made the era huge but left it real-time honest (~3 years
at 1:1) and flat-paced. Reshaped into a **~3-real-month campaign with a felt
ramp** — the maiden voyage is the slowest crossing of the era, and the ferry
never stops accelerating:

- **Time at sea is compressed**: `GAME_MINUTES_PER_REAL_MINUTE = 24` — one
  sea-day per real hour. The maiden voyage sails ~1.5 real days; late
  crossings turn around between a morning and an evening check-in. Fixtures
  and tests express exact offsets via `real_duration_for_game_minutes()`,
  so they are scale-agnostic.
- **The ramp is the point**: `FASTEST_CROSSING_TIME_MULT` 0.5 → 0.2 (up to
  5× her launch speed) and `DRIVE_FOR_HALF_SPEEDUP` 2,500 → 6,000 — felt
  early, still climbing at the era's end. Sim: 36 → 9 sail-days over 59
  crossings.
- **More crossings**: `EXPEDITION_PER_1000_DELIVERED` 75 → 35 and
  `DARK_TAKES_EACH_CROSSING` 0.7% → 0.45% stretch the 100k world across
  **59 crossings, ~83% saved**, districts spread crossing ~4 to ~54.
- **Auto-sail** (the compression made it necessary): a mid-crossing port
  with no decision — one road out, no ask, no refit door — gets a
  6-game-hour port call, then the ship sails herself. The scene is played
  by the engine and queued (`unread_scenes`, serialized) for the ferryman
  to read on return, oldest first. Decisions always hold the ship:
  junctions, asks, refit doors, and the pier (`arrived_by: None`) — launch
  and `Sail again` are never the engine's. Without this, ~20 waits ×
  59 crossings would have made the era mostly waiting; with it, a crossing
  asks ~3–5 decisions and the era ~2 a day.

## Follow-up (2026-07-04): the two yards — Drive & Shipwright, earned not compressed

The ramp is now a **choice**, and it is earned by the ship getting faster, not by the clock speeding up. The old cumulative-Drive-speeds-everything model is replaced by two Salvage-bought tracks, decoupled from delivery so the acceleration doesn't wait on the slow early crossings.

- **Two yards, one currency.** Each crossing pays out **Salvage** (`SALVAGE_AT_LANDFALL + carried/SOULS_PER_SALVAGE`). On arrival (the Reckoning, `[D]`/`[C]`) the ferryman spends it:
  - **Drive** (`drive_level`): crossing sail-time ×`DRIVE_DECAY` (0.70) per level, floored at `DRIVE_FLOOR` (0.05 ≈ 20× top speed). Level 0 = the maiden voyage, the slowest crossing there is.
  - **Shipwright** (`cap_level`): hold ×`CAP_GROWTH` (1.36) per level. `expedition_size` = `BASE_CAPACITY (180) × CAP_GROWTH^cap_level + district bonuses` (the per-1000-delivered term is gone — the hold only grows when you pay for it). `STARTING_SALVAGE` 40 so the ramp bites from the second crossing.
- **Uniform clock.** `GAME_MINUTES_PER_REAL_MINUTE` 24 → **2.64** (a sea-day ≈ 9 real hours). No per-crossing time scale: the same clock runs every crossing, and only the earned Drive level shortens it. **C1 ≈ 14 real days (two weeks)**; a buildup over the first handful (14 → ~4 real-days), then a long fast-fun stretch of ~3-real-day turnarounds while the loads climb into the thousands.
- **The decision is real, and the margin is wide** (`DARK_TAKES_EACH_CROSSING` 0.0045 → **0.011** — the toll that makes crossing-count matter for souls saved). Reckless Drive-only runs ~85 near-empty crossings and bleeds the world to the dark (**~54% saved**); a souls-first line (lean into the hold, just enough Drive to stay quick) carries most of it home (**~87% saved**) — skill is rewarded, not marginal. Tuned to **~19 crossings, ~3 real months, ~87% saved with skilled play**.
- **Ferry runs are fully hands-off** (required for the crossing to complete autonomously in Drive-scaled time): crossing 2+ auto-navigates junctions (first road), skips refit doors, launches itself from the pier, and the passenger load no longer deepens the provisions burn (no one meters rations on a ferry run). The maiden voyage is unchanged — decision-rich, navigated by the ferryman.
- **Superseded constants**: `EXPEDITION_AT_LAUNCH`, `EXPEDITION_PER_1000_DELIVERED`, `FASTEST_CROSSING_TIME_MULT`, `DRIVE_FOR_HALF_SPEEDUP`, and the cumulative `drive` field are gone; `BASE_CAPACITY`, `CAP_GROWTH`, `DRIVE_DECAY`, `DRIVE_FLOOR`, `SALVAGE_*`, `DRIVE_COST_*`, `CAP_COST_*`, and the `drive_level`/`cap_level`/`salvage` fields replace them.

> **Doc-alignment note (2026-07-04):** everything above this point in the
> "two yards" follow-up was itself superseded the same day by commit
> d39ad67 — see the next follow-up below. `DARK_TAKES_EACH_CROSSING` no
> longer exists (replaced by a per-day rate); the "~19 crossings, ~87%
> saved" figure is now a range, not a single target; and the Reckoning has
> three purchases, not two.

## Follow-up (2026-07-04, later the same day): the third yard — Ward, and per-day attrition

Shipped in commit d39ad67 ("Act 2: retire Hope into a three-yard Reckoning"). Two changes, both aimed at the same problem: the Hope gauge above (see the retired sections earlier in this file, and every `hope`-referencing passage in specs 3, 5, 6, and 8) never engaged in play — balance-sim evidence showed it pinned at its maximum under every attentive strategy. Rather than tune a gauge nobody was reading, this redesign retires it and gives its job to a yard the player actually buys.

- **Hope is retired, full stop.** `HOPE_MAX`, `LAUNCH_HOPE`, `HOPE_FLOOR_STEADY`, `HOPE_SPEND_FLOOR`, `PRESS_*` (Press-the-helm), `HARD_RATIONS_BURN_MULT` — all removed from `voyage.rs`. Every "hope +N" / "hope −N" beat described elsewhere in the spec tree (letters, arcs, night outcomes, district bonuses, refits) no longer prices anything; those beats now either cost nothing or were replaced by a different mechanic (the Silence-bank's hope-drain became a strain hit on the Helm soul — see spec 5's underway doc for the current version).
- **A third yard: Ward** (`ward_level`, `colony.rs`). Costs `WARD_COST_BASE (5) × WARD_COST_GROWTH (1.45)^L` Salvage — priced between Drive and Shipwright. Each level multiplies the dark's toll rate by `WARD_DECAY` (0.72), compounding down to `WARD_TOLL_FLOOR` (0.12× base — the dark's bite is never fully closed, just dampened).
- **The toll changed shape**: from a flat per-crossing tax (`DARK_TAKES_EACH_CROSSING`, retired) to a **per-day rate** (`DARK_TAKES_PER_DAY = 0.0006`, compounding over the crossing's length via `dark_toll_for_days()`). This is the mechanically load-bearing change: now *all three* yards answer to the same pressure — Drive (fewer days per crossing) and Shipwright (fewer crossings to empty the world) both cut the total days the world spends waiting, on top of Ward directly softening the daily bite.
- **The Reckoning is now a three-way comparison** (`[D]`/`[C]`/`[W]`), each purchase shown with a live before→after number (e.g. Ward: "0.060%/day → 0.043%/day · ~83 fewer lost over a crossing") rather than just a level-up button.
- **Re-tuned, and re-stated as a range rather than a single target.** Verified via `ferryman_tests::strategy_sweep` (2026-07-04): a balanced spend saves ~88% across ~19–24 crossings / ~3 real months; the two naive traps (Drive-only, Shipwright-only) land at ~70–74%; leaning hard on Ward pushes to ~94% saved but stretches the era to ~30+ crossings / ~5 real months. All three are treated as valid skilled lines (see `docs/decisions.md`, "Act 2 Ward Pacing") — the era's stated length is now "~3–5 real months" depending on how the player spends, not a single number to hit.
- **Fixed (2026-07-04)**: at the Drive/Ward floors, `buy_drive()`/`buy_ward()` (`colony.rs`) now refuse the purchase outright via `drive_maxed()`/`ward_maxed()` — previously the yard let a player escalate Salvage into a level that bought zero further gain; the Reckoning UI now hides the cost line entirely once maxed instead of showing an unaffordable price.
