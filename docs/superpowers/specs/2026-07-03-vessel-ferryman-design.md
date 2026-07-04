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

- **`souls_remaining`** — the dying world's finite pool (initial value
  ~3,000; tuned by probe). Visible from the first Reckoning.
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
