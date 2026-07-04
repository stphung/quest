# Act 2: The Pilgrimage of Souls — Design Dossier

> Last refreshed: 2026-07-04 @ 4ce9a57 | Sources: `src/vessel/`, `src/vessel/CLAUDE.md`, `docs/superpowers/specs/` (10 vessel specs), `tests/ferryman_tests.rs`, voyage_simulator runs, played via `QUEST_ACT2=1` fixtures

## The Player's Experience

Act 2 begins as a rumor inside Act 1. The first Zone 50 final-boss kill sets
the signal; from then on a whisper crosses the ticker about once a minute and
a `[V]` overlay shows a four-gate checklist — complete the Loom (28 patterns),
reach Ascension X, and accumulate 250,000 PR to burn in one all-or-nothing
action. The player watches a fuel bar fill over weeks. The burn is the act
break: everything Act 1 accumulated becomes the hull.

Then the game changes shape entirely. The Voyage runs on the wall clock
(2.64 game-minutes per real minute; a sea-day ≈ 9 real hours), not the tick
loop. The **maiden voyage** is the decision-rich crossing: ~26 named ports
over ~40 sea-days (≈ two real weeks), checking in a few times a day to chart
courses at junctions (committing closes the other roads for good), set trim,
stand the watch, answer recruit asks, choose refit doors, read arrival
scenes and Letters From Home. Cadence: a decision or scene every check-in,
a junction every few days, a chapter gateway roughly weekly, the Going-Dark
once — the night the mail stops.

Arrival opens three quiet rooms (manifest, keepsake chart, record) — and then
the **Reckoning** reframes the act: 100,000 souls wait in the dying world, the
ship sails back, and every landfall pays Salvage to spend on two yards
(Drive = speed, Shipwright = hold). Ferry runs (crossing 2+) are fully
hands-off — the ship sails herself in Drive-scaled time while the player's
choice compresses to speed-vs-salvation at each Reckoning. The era runs ~24
crossings over ~3 real months, the ramp going 37 → 8 sea-days while loads
climb from 7 crew to ~4,500 souls, the dark taking 1.1% of whoever still
waits each crossing.

## Design Intent

The de-facto design bible is scattered but real (no single consolidated doc):

- **Thesis** (`docs/superpowers/specs/2026-03-27-the-vessel-design.md`):
  Act 1 is "power in one place"; Act 2 is *passage*. "Act 2's fun is
  anticipation, choice, people, and consequence — the kinds of fun Act 1
  never touched… what accumulates while you're away is not numbers. It is
  *arrival*."
- **Direction choice** (`…/2026-07-02-act2-voyage-experience-exploration.md`):
  route/place as spine + souls/loss as heart; each check-in should be "an
  event with a face on it, not a status glance."
- **Per-slice intent** in specs 2–8: arrivals are payoff scenes, never tests;
  no dice anywhere; loss is authored-only; "traveling is not dead time."
- **Pacing targets** (`…/2026-07-03-vessel-ferryman-design.md:315-319`, the
  only home of the tuned numbers): C1 ≈ 14 real days; ramp 14 → ~3 real-days;
  **~19 crossings, ~3 real months, ~87% saved with skilled play**; reckless
  Drive-only play ~54% saved (skill margin is meant to be wide).

Note: specs 1–8 describe single-playthrough duration language ("20–200 days")
that spec 9 (the Ferryman) superseded; nothing in the tree says so explicitly.

## Mechanics & Constants

Launch gate (`src/vessel/mod.rs`): `LAUNCH_PR_COST` 250,000 (`mod.rs:116`),
`LAUNCH_REQUIRED_PATTERNS` 28 (`:119`), `LAUNCH_REQUIRED_ASCENSION` X (`:122`),
`WHISPER_INTERVAL_SECONDS` 60 (`:125`). Gate at `mod.rs:147-149`, burn `:158`.

Voyage (`src/vessel/voyage.rs`): `GAME_MINUTES_PER_REAL_MINUTE` 2.64 (`:81`),
`PROVISIONS_CAP` 100 (`:33`, 150 with Long Hold `:35`), `DRIFT_RECOVERY_PROVISIONS`
25 (`:40`) / `DRIFT_RECOVERY_HOURS` 36 (`:41`) — also the affordability floor
asserted at `route.rs:1384-1391`, so a dry hold always means drifting, never
stranding. `HOPE_MAX` 10 (`:47`), `LAUNCH_HOPE` 7 (`:73`),
`HOLD_STATION_GRACE_DAYS` 3 (`:43`), `PORT_CALL_GAME_MINUTES` 360 (`:63`).

Route (`src/vessel/route.rs`): 38 waypoints (`:194`), 45 roads (`:620`), 8
rumors (`:1143`), 4 chapters (`:27`), 7 junctions 2/2/2/1 (`:1373-1380`);
spine-and-diamond DAG, single sink = the Tree (`:186`).

Souls (`src/vessel/souls.rs`): 8 authored souls for `CREW` = 7 seats (`:16`) —
Torvald/Cormac (Helm), Eir/Ysolt (Tender), Runa/Maren (Watch), Sefa and
Brother Wren unaffined; 3 aboard at launch, 5 found. Arcs advance on
`ARC_BEAT_REST_DAYS` 2 (`:18`); `LOSS_HOPE_COST` 3 (`:21`), authored-scenes
only; `FAREWELL_HOPE_COST` 1 (`:23`).

Colony (`src/vessel/colony.rs`): `INITIAL_SOULS` 100,000 (`:21`);
`DRIVE_DECAY` 0.70 to floor 0.05 ≈ 20× (`:36,:39`); `BASE_CAPACITY` 180 ×
`CAP_GROWTH` 1.36 (`:26,:30`); Salvage = 3 + carried/30 per landfall
(`:44,:46`); yard costs 4×1.5^L / 5×1.42^L (`:54-58`);
`DARK_TAKES_EACH_CROSSING` 0.011 (`:64`). Six districts found at pop.
500 → 66,000 (`:92-99`), each adding +110…+320 expedition size (`:105-113`).

Derived numbers players feel: maiden voyage ≈ two real weeks; ferry-run floor
≈ 3 real days; max hold ≈ 4,500+ souls; era ≈ 3 real months.

## Interrelations

```
Act 1 (everything)          Act 2 (the passage)
  Loom 28 patterns  ─┐
  Ascension X       ─┼─► Launch gate ─► Voyage ─► Colony era
  250,000 PR (burn) ─┘        │            │          │
  Zone 50 kill ─► signal ─────┘            │     Salvage ─► Drive/Shipwright
                                           │          │
  Deep/Loom/etc. idle beneath ◄── untouched┘     souls delivered ─► districts
                                                       │
                                              vessel_arrived ─► (Act 3 hook)
```

- **In**: the launch gate deliberately braids *all* of Act 1 (Loom, Ascension,
  PR economy, Zone 50) into one burn — the act's strongest cross-system edge.
- **During**: zero mechanical interaction with Act 1 — deliberate ("one-way
  passage"); Act 1 idles untouched beneath. Narrative callbacks only
  (Torvald is the Deep guild's captain).
- **Out**: `vessel_arrived` is the authored Act 3 hook. Salvage/districts are
  Act 2-internal currencies; nothing else consumes them (by design, but note:
  Records and keepsakes have no external surface either).
- **Dangling edge**: at the Drive floor (Lv 10+), the Reckoning still offers
  the next Drive level at full price for zero speed gain (seen in play;
  `colony.rs:36-39` floor vs the yard's unbounded offer).

## Balance Evidence

*2026-07-04, voyage_simulator `--runs 5 --strategy all` + `ferryman_tests` era
runs, at 4ce9a57:*

| Intent (spec 9) | Measured | Verdict |
|---|---|---|
| C1 ≈ 14 real days | 40 sea-days ≈ 15 real days (cheapest line) | ✓ |
| Ramp floor ~3 real days | 8 sea-days ≈ 3 real days | ✓ |
| ~19 crossings / era | **24 crossings** (balanced policy) | ~25% over |
| ~87% saved, skilled | 86.9% souls-first; 82.9% balanced | ✓ |
| Reckless ~54% | 54.3% | ✓ |
| Care beats carelessness | cheapest day 40 vs priciest day 49-52 (9-10 drifts, 6 scars) | ✓ |
| No stranding ever | pure-neglect arrives day 84-89 | ✓ |

**Red flag**: hope ended 10/10 with minimum 7 in 24 of 25 runs (only pure
neglect ever dipped, to 1). The second gauge — the one spec 3 calls the
heart of the act — never engages under any attentive strategy: the Long
Silence never fires, Mourn trim has no reason to exist, `HOPE_FLOOR_STEADY`
never binds.

## Fun Assessment

*2026-07-04, scored against the seven Act 1 benchmarks after simulator runs +
played sessions (launch overlay, chart, junction, souls, Reckoning at
160×45):*

| # | Heuristic | Score | Evidence |
|---|---|---|---|
| 1 | Visible next goal | 4/5 | Gate checklist + fuel bar pre-launch; next-beat timers, watch forecast, district thresholds in-voyage. Missing: champion row omits its target (Asc X); no era-level projection ("~N crossings left"). |
| 2 | Wall → reset → power | 4/5 | The ferry loop is a true earned ramp (37→8 days, 180→4,500 hold — felt hard). But no resistance mid-era: nothing ever pushes back, the dark's toll is invisible pressure. |
| 3 | Discovery cadence | 3/5 | Maiden voyage drips new nouns constantly (weather, nights, souls, rumors, refits, letters, pilgrims). Post-maiden: ~23 crossings / ~2.5 months where the only new nouns are 6 district thresholds. Front-loaded, then flat. |
| 4 | Cross-system braiding | 3/5 | Launch gate braids all of Act 1 into the burn (excellent); the voyage itself is deliberately an island. Flagged as an intentional departure — confirm. |
| 5 | Decision density | 3/5 | Maiden voyage: right density (junction every few days, trim/watch/asks between; sim confirms decisions matter, 40 vs 52 days). Ferry runs: one D/C choice per ~3 real days. |
| 6 | Anticipation instruments | 5/5 | Whispers, fuel bar, watch forecast, rumors-as-purchased-foresight, sequenced letters, the Tree growing on the chart. The act's strongest suit. |
| 7 | Stakes and texture | 3/5 | Texture is rich (weather, nights, scars, authored loss). Stakes are soft: hope pinned at max (see Balance), loss nearly unreachable, drift the only bite. |

**Where Act 2 deliberately breaks Act 1's patterns** (confirm, don't "fix"):
wall-clock instead of ticks; no failure states; no RNG in outcomes; the
voyage severed from Act 1 systems. All four are stated intent in the specs.

## Open Questions & Decision History

Asked 2026-07-04 (see `docs/decisions.md` for outcomes once resolved):
1. Hope gauge never engages — tune, redesign, or demote?
2. Post-maiden discovery drought — accept, add per-crossing beats, or new mid-era noun?
3. Era length 24 vs intended ~19 crossings — retune or re-state intent?
4. Launch transition is one static screen where spec 4 designed 5 beats — build or keep?

Held for a later round (not yet asked):
- Should Records/keepsakes surface anywhere outside the arrived harbor
  (e.g. title screen), given the era ends and the files remain?
- Drive-floor yard offer (zero-gain purchase) — small fix, queued.
- No player-facing wiki page exists for Act 2 (correct while dark-shipped;
  becomes a launch-checklist item).

Factual drift found and fixed this refresh: `game_state.rs:86` and
`mkstate.rs:113` said "100,000 PR" (actual 250,000); `route.rs:619` said
"~46" roads (actual 45).
