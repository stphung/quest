> Backported design record. Sources: docs/superpowers/specs/2026-03-27-the-vessel-design.md, docs/superpowers/specs/2026-03-27-vessel-combat-design.md, docs/superpowers/specs/2026-03-27-vessel-crew-design.md, docs/superpowers/specs/2026-03-27-vessel-launch-gate-design.md, docs/superpowers/specs/2026-03-27-vessel-mode-transition-design.md, docs/superpowers/specs/2026-03-27-vessel-rooms-stats-design.md, docs/superpowers/specs/2026-07-02-act2-voyage-experience-exploration.md, docs/superpowers/specs/2026-07-03-vessel-arrival-design.md, docs/superpowers/specs/2026-07-03-vessel-arrival-scenes-design.md, docs/superpowers/specs/2026-07-03-vessel-ferryman-design.md, docs/superpowers/specs/2026-07-03-vessel-letters-going-dark-design.md, docs/superpowers/specs/2026-07-03-vessel-pace-rations-naming.md, docs/superpowers/specs/2026-07-03-vessel-price-of-passage-design.md, docs/superpowers/specs/2026-07-03-vessel-route-waypoints-design.md, docs/superpowers/specs/2026-07-03-vessel-souls-design.md, docs/superpowers/specs/2026-07-03-vessel-underway-design.md.

## 2026-03-27-the-vessel-design.md

# The Vessel — Act 2: The Crossing

**Issue:** Post-Loom endgame progression
**Direction:** The Pilgrimage of Souls — see
[2026-07-02-act2-voyage-experience-exploration.md](2026-07-02-act2-voyage-experience-exploration.md)
for the analysis that chose it.

## Elevation

Act 1 was about **power in one place**: a hero looping through a world,
growing without limit. Act 2 is about **passage**: everything that hero built
is burned into one ship, which crosses the void between a dying branch of
Yggdrasil and a living one — a one-way pilgrimage down a **route of named
places**, carrying the **last souls of the old world**, toward a **tree that
grows on the horizon** with every league made good.

Act 1's fun was growth, engines, and loot. Act 2's fun is anticipation,
choice, people, and consequence — the kinds of fun Act 1 never touched.
The idle contract survives (the ship sails while you sleep), but what
accumulates while you're away is not numbers. It is *arrival*.

## Design Pillars

1. **The journey is a place, not a bar.** Progress is movement along a
   constellation chart between named waypoints, with the destination visibly
   growing on the horizon. Never a percentage.
2. **Check-ins are appointments.** The core idle unit is a multi-day *leg*;
   the core active unit is an *arrival* — a short scene at a place with a
   name, where what the road promised is paid. The game tells you when
   you'll arrive.
3. **All choice lives on the map.** The junction is the game's one decision
   surface: you choose *roads*, informed by rumors, and then the road
   happens to you. Arrivals are never tests — no option menus, no
   risk/reward gambles at the dock. The risk was priced when you picked
   the road.
4. **Doors close.** Routes not taken stay untaken. Souls lost stay lost.
   This is the first part of the game where the player can't eventually
   have everything — that is the point.
5. **People over parts.** The ship's crew are named souls with arcs, not stat
   multipliers. The emotional payoff of the act is the manifest read at
   arrival: *who made it*.
6. **Small numbers, big moments.** No ship XP, no power budgets, no tier
   ladders. Two gauges, 7 berths, a route of ~24 stops. Every number fits in a sentence.

**Anti-goals** (deliberately absent, because Act 1 owns them): ship
levels/XP; a room-grid engine to optimize; an enemy-tier ladder; drop tables
and rarities; any resettable loop. Players who crave optimization get route
planning, refit trade-offs, and the rumor economy — strategy, not spreadsheet.

## Narrative Foundation

**Yggdrasil is dying.** Zone 50 (The Origin Thread) reveals the branch is
withering from the root. The fractures, the void, the corruption — symptoms
of a dying limb.

**The beacon.** A living branch, impossibly far, is calling. The Loom
resonates with it — the 28 patterns were unconsciously answering all along.

**The ship IS the Loom.** Woven fate becomes woven hull. 250,000 PR burns in
one breath to make the transformation. Everything the player built was
preparation.

**The passengers.** The old world cannot be saved, but some of it can be
carried. The faces of Act 1's systems — the mercenary captain who found you,
the Haven's warden, the fisher who taught you the leviathan's rhythm — board
the Vessel. They are what the journey is *for*.

**The destination.** The living branch. What is found there is Act 3.

## Launch (sub-project 1 — SHIPPED)

Unchanged and compatible: 28 patterns + Ascension X + Zone 50 + a single
all-or-nothing burn of 250,000 PR. Ships dark behind `vessel::ACT2_ENABLED`.
See [vessel-launch-gate-design.md](2026-03-27-vessel-launch-gate-design.md).

---

## The Shape of Act 2

### The Route

A constellation chart of **~24 named waypoints** across four chapters,
connected by legs, with **~7 junctions** where the route forks and the
player chooses. The chart is the main screen of Act 2.

| Chapter | Waypoints | Character |
|---------|-----------|-----------|
| I. The Shallows | 1–5 | The wake of the dying world — debris of the known, the last familiar stars, the first strangers |
| II. The Drift Roads | 6–12 | Old travel lanes between branches: way-stations, wreck-fields, other pilgrims' remains, trade |
| III. The Starless Deep | 13–18 | True void. The old world goes dark behind you. The route's hardest choices |
| IV. The Roots of Light | 19–24 | The tree's presence changes the void itself. Arrival |

- **Fog of route:** the chart shows the current leg and the next junction
  only. **Rumors** — bought, traded, or earned at waypoints — illuminate
  branches further ahead. Information is a resource; junction choices are
  only as good as what you've heard.
- **Roads not taken:** an unchosen branch grays out permanently. The chart
  keeps the names visible forever — a scar, not a checklist.
- **The horizon:** the tree renders at the top of the chart, growing from a
  faint mark at launch to a canopy that fills the sky at waypoint 24. This
  is the act's progress indicator — an illustration, not a bar.

### The Rhythm: Legs and Arrivals

- **A leg** (waypoint → waypoint) is the idle unit: **1–3 real days** — and
  not dead time. The void has **weather** (currents, silence-banks, squalls
  that move on the chart in real time), the ship has **trim** (one posture
  dial priced against today's weather), and the night has **watches**
  (typed nights, scheduled souls, a log the nights write themselves).
  Other pilgrims' lights cross the chart on their own roads. See
  [2026-07-03-vessel-underway-design.md](2026-07-03-vessel-underway-design.md).
  The footer always reads: *Arriving at ⟨place⟩ in ~⟨time⟩.*
- **An arrival** is the active unit: a scene at a named place that **pays
  off the road you chose** — the shipyard the rumor promised, the survivor
  the wreck held, the toll the hungry road took. You read it, you collect
  it, you depart. **No option menus.** Arrivals still wait for the player:
  the Vessel holds station until you play the scene. The appointment keeps.
- **A junction** is the choice unit — the game's only one: 2–3 onward roads,
  each a package of stops with visible character (this road has a shipyard;
  that one is hungry but fast; the third, no one has word of) annotated by
  whatever rumors you hold. Chosen once. No takebacks. Everything a
  risk/reward menu would have asked at a waypoint is asked here instead,
  where it is strategy rather than a dice roll.

What a waypoint delivers is determined by three things the player already
controls: **the road chosen** (a hungry road costs provisions; a lucky one
gives them), **the refits carried** (they recolor hazards into non-events
or worse), and **the souls aboard** (Runa aboard means the Drowned Choir is
answered — automatically, as a scene, not as a button). Scene *shapes* —
way-station, wreck, haven, shrine, shipyard — are content templates for
writing, not decision templates. Named threats — Níðhöggr's Fang and its
kin — live on specific roads; whether their scene is a toll, a terror, or
a passage depends on what you brought, all of it knowable at the junction
if your rumors were good.

Two decisions do survive outside junctions, because they are
junction-shaped (exclusive, permanent, strategic — never gambles):
**berths** (a soul asks to board; seven berths; at capacity someone must be
declined) and **refits** (a shipyard offers A or B, once).

### No Right Path

There is deliberately no optimal route, and no equal ones either — choices
matter through **difference, memory, and irreversibility**, never
optimality. The authoring rules that guarantee it:

1. **Roads differ in kind, never in amount.** Branches at a junction trade
   incommensurables — time vs a place vs a person vs a refit. If two
   branches can be compared on one axis, the junction is broken. This is
   what defeats optimization; symmetric balance would not.
2. **The game remembers.** Grayed roads keep their names all the way to the
   Tree; losses stay carved; the manifest recounts the causal chains. A
   choice never referenced again didn't matter, whatever it cost.
3. **Never reveal what an untaken road held.** Names, not contents — the
   player's imagination of the road not taken must stay bigger than
   anything we could write. Itemizing missed content converts difference
   into regret and reintroduces "the right path" as the one you skipped.
4. **You cannot lose the crossing — only have a different one.** Every
   route reaches the Tree; drift floors every failure. Locally worse
   outcomes exist (the Teeth), but they are priced, chosen, and become
   story — never "restart" (there is no restart; one crossing per save).
5. **Content parity, not content equality.** Every route family passes
   enough soul-candidates and at least one shipyard that no route locks
   the player out of a full voyage — different souls, not fewer.
6. **No grades.** The manifest's numbers are memory, never score: no
   ranks, no "true ending," no epilogue variant that outranks another.

Side effect worth noting: this makes the act spoiler-proof — a wiki that
maps every road is just free rumors, and the game already sells rumors.
A fully-informed player still faces the same incommensurable trades.

### The Souls

**Three board at launch** — faces from the systems the player mastered
(the Deep's captain, the Haven's warden, the fisher). **Up to five more**
can be found along the route; the hull berths **seven**, so late recruits
force real choices. Each soul has:

- **A face and a voice** — one line of personality that colors ambient log
  moments and arrival scenes.
- **A station** (helm, tender, watch) — souls at stations recolor
  what roads yield ("with Torvald at the helm, the Teeth
  are a passage, not a toll"). Assignment matters; arithmetic stays out of
  sight.
- **An arc** — one personal thread that unfolds at particular places along
  particular roads (part of what a road is *worth* at the junction) and
  resolves before arrival. Arcs pay out in hope, in rumors, and in the
  manifest.
- **Mortality.** Souls can be lost — to roads chosen badly, to the deep
  void, to their own arcs. Loss is permanent. Each loss carves a name into
  the hull, visible in the ship art for the rest of the game. (The offline
  covenant from the old spec set stands: **no soul is ever lost while the
  player is away.**)

**Hope** is the second of the game's two gauges (see Resource Model) —
moved by losses, arcs, letters kept, and small kindnesses. It cannot be
spent and cannot be bought. It has one mechanical effect: high hope is
wind — legs run faster; low hope drags days onto every crossing, and at
bottom the ship falls into the Long Silence (arcs pause, legs crawl) until
a rest stop breaks it. Hope is the caravan answer to Act 1's power curve:
the number that measures *how the people are doing*.

> **Doc-alignment note (2026-07-04):** this is the act's founding thesis doc
> and reads mostly true today — Souls, Provisions, letters, the arrival —
> but several concrete details below have since changed. **Hope was retired
> entirely** (commit d39ad67; see `docs/superpowers/specs/2026-07-03-vessel-ferryman-design.md`'s
> two Follow-up sections for the full story) — every passage below
> describing Hope as "the second of two gauges," the Long Silence, or hope
> as wind/currency is now historical, not current design. What replaced it:
> speed comes from Trim (player-facing **Pace**, per the 2026-07-03 naming
> pass) and station bonuses only; the equivalent "how are the people doing"
> pressure lives in the Colony era's Ward yard (post-maiden-voyage only —
> see `src/vessel/colony.rs`), not in the crossing itself. Also stale:
> **refits** shipped as **6** (3 A/B pairs), not ~3 (`refits.rs`); the
> **route** is a 38-waypoint, 45-road, 8-rumor branching DAG (`route.rs`),
> not a single 24-stop line — "waypoint 24" below describes one maximal
> traversal, not the authored total. `src/vessel/CLAUDE.md` is ground truth
> for current constants and shape.

### The Vessel Underway

### Resource Model — two gauges, nothing else

**Provisions** is the whole material economy in one bar. Every road has one
cost, stated on its junction card (*"~9 days · 40 provisions"*) — long roads
cost more, dangerous roads cost more, holding station sips a little. Places
refill it (harbors, rest stops) and letters from home top it up until the
Going-Dark. At zero, the ship drifts mid-leg into a recovery scene — a
setback with a story, never a death.

**Hope** is the people gauge, defined with the Souls above: unspendable,
one effect (the wind), and the lens the finale reads through.

Everything else the player holds is deliberately **not** a resource:

- **Rumors** are annotations on the chart, not a currency — held forever,
  acquisition priced into scenes and way-stations.
- **Keepsakes** have no mechanics: mementos that appear in the manifest,
  and occasionally the only tender a stranger will accept — flavor, not
  economy.
- **Berths** are a slot limit (seven), not an upkeep cost. Souls eat
  nothing; the berth question is always *who*, never *how many can we
  feed*.
- **Refits**: ~3 one-time A/B choices at shipyards that permanently
  re-price roads (*Storm Sail:* legs faster, dangerous roads cost more ·
  *Long Hold:* +provisions cap · *Quiet Keel:* named threats cost less).
  Configuration, not consumables.

No conversions, no crafting, no production trickle: sources are places and
letters, sinks are roads. Every resource problem has the same answer the
game wants you to reach — *look at the chart and pick a road*.

**The travel layer** replaces auto-combat (see the Underway spec): weather
prices time, hope, and provisions; trim and the watch rotation are the
standing choices between junctions; the log is the voice of the crossing.
The player's defenses are route choice, refits, trim, and the watch —
not DPS.

### Letters From Home — and the Going-Dark

The old world does not trickle PR to the ship. It **writes**.

- At each arrival in Chapters I–II, a **care package** waits: provisions
  and a keepsake — contents scaled by what the Loom produced during the
  leg — wrapped in a short letter that references the player's own Act 1
  history (the Haven they built, the leviathan they caught).
- **The Going-Dark:** at the threshold of Chapter III (~60% of the route),
  the dying branch finally dies. One last letter — a goodbye — and then
  silence. No more packages. The ship is truly alone, and every provision
  from here is earned from the void itself. This is the act's midpoint twist,
  the moment the journey's stakes become real, and the clean mechanical end
  of the supply line (no taper math, no transmission-rate balancing).

### The Arrival

Waypoint 24: the Roots of Light. The final approach is a scripted sequence —
the canopy filling the screen, the souls at the rail. Then the **manifest**:
every soul named — who boarded, who joined, who was lost and where their
name is carved — and what each carries into the new world. The chart of the
whole crossing, roads-not-taken grayed forever, becomes a keepsake screen.
The tree is reached. Act 3 is its own story.

---

## What Playing Act 2 Feels Like

**The first hour.** The burn. The five-beat transition. Then — quiet. A
chart with one lit leg, three souls settling in, the tree a faint mark on
the horizon. The log murmurs. The footer says *Arriving at the Shoal of
Lanterns in ~26h.* You close the game. Nothing needs you. That's new.

**A check-in, week two (Chapter I).** You open the game at lunch. The leg
finished overnight: the Vessel holds station at a wreck of the first pilgrim
fleet — the stop this road was known for. The scene plays: Eir searches the
wreck and finds medicine; because Runa stands the watch, she hears what
nests inside before it hears the ship, and it stays sleeping. A rumor
learned, a rune fragment nobody can read yet. A care package from home
waits below: provisions and a letter that mentions your Haven by name.
Then the junction — the only decision of the day: the short road is hungry
(a rumor from two stops back), the long road has a shipyard. You take the
long road. Six minutes. Done. The footer says ~2 days.

**A check-in, month three (the Going-Dark, Chapter III).** The last letter
arrived two stops ago; you've reread it twice. Provisions are thinner than
you'd like — you took the hermit's road knowing it tithes, because
hope rises where he keeps his lanterns, and high hope has made the legs
faster. Runa's arc came to a head last arrival; she stood the watch and
something in the dark *answered her song* — a rumor no trader could have
sold you. At the junction, Torvald reads one road and the warden reads the
other, and their counsel disagrees. You side with Torvald. You're wrong
about that, but you won't know for two more stops.

**A check-in, month six (Chapter IV).** The tree owns half the horizon.
One name is carved into the hull; you still route around hazards you'd have
shrugged at in month two, because seven berths hold five souls now and every
one of them has a face. The chart behind you is a braid of lit and grayed
roads. The footer says *Arriving at the Roots of Light in ~3 days*, and for
the first time in this game, you don't want the number to go faster.

**The player's verbs, throughout:** read, choose, route, assign, keep.
Never: grind, optimize, reset.

## Pacing Skeleton

- ~24 waypoints × 1–3-day legs (+ hold-station time at arrivals) ≈
  **5–8 months** — matching the established Act 2 duration target while
  making its texture episodic instead of continuous.
- ~7 junction choices, ~4 refit choices, a handful of berth decisions,
  6–8 soul arcs, ~30 authored arrival scenes (24 waypoints + branch
  variants), 1 midpoint set-piece (Going-Dark), 1 finale (Arrival).
- Active time: 3–10 minutes per arrival (read the scene, collect, choose
  the road when there is one), every 1–3 days — a fraction of Act 1's
  engagement but far denser in consequence.

## Spec Queue (rewritten)

| # | Spec | Scope | Status |
|---|------|-------|--------|
| 1 | Launch Gate | gates, burn, kill-switch | **Shipped** |
| 2 | The Route & the Waypoints | route graph data model, voyage state machine, junction cards & pricing, fog & rumors, chart renderer, mode routing | **Shipped** — [2026-07-03-vessel-route-waypoints-design.md](2026-07-03-vessel-route-waypoints-design.md) |
| 3 | The Souls | roster, stations, arcs, hope, loss & memorial, offline covenant | **Shipped** — [2026-07-03-vessel-souls-design.md](2026-07-03-vessel-souls-design.md) |
| 4 | Arrival Scenes | scene delivery system, road/refit/soul recoloring, scene shapes, writing pipeline | **Shipped** — [2026-07-03-vessel-arrival-scenes-design.md](2026-07-03-vessel-arrival-scenes-design.md) |
| 5 | Underway — Weather, Trim & the Watch | void weather, trim postures, night types & watch rotation, the log, pilgrim ships, drift, refits | **Shipped** — [2026-07-03-vessel-underway-design.md](2026-07-03-vessel-underway-design.md) |
| 6 | Letters From Home & the Going-Dark | care packages, letter templates, the midpoint event | **Shipped** — [2026-07-03-vessel-letters-going-dark-design.md](2026-07-03-vessel-letters-going-dark-design.md) |
| 7 | The Arrival | final approach, manifest, keepsake chart, Act 3 gate | **Shipped** — [2026-07-03-vessel-arrival-design.md](2026-07-03-vessel-arrival-design.md) |
| 8 | The Price of Passage | scarcity pass, hope sinks, strain ledger, hull wear & mend-vs-refit | **Shipped** — [2026-07-03-vessel-price-of-passage-design.md](2026-07-03-vessel-price-of-passage-design.md) |
| 9 | The Ferryman | the crossing loop, souls delivered & the dimming race, Resonance, the Colony, the Reckoning pane, Act 3 gate moves to the Last Crossing | **Shipped** — [2026-07-03-vessel-ferryman-design.md](2026-07-03-vessel-ferryman-design.md) |
| 10 | Pace & Rations naming | rename Trim→Pace (Grueling/Steady/Easy/Restful) and the rations toggle (Filling/Bare Bones), Oregon Trail register; strings only | **Shipped** — [2026-07-03-vessel-pace-rations-naming.md](2026-07-03-vessel-pace-rations-naming.md) |

The old sub-specs 2–5 (mode transition, rooms & stats, auto-combat, crew)
are **superseded** by this direction; salvage from them what still fits
(the 5-beat launch transition, gauge/drift math, offline rules, crew
capacity bones).

## Open Questions (to settle in spec 2)

- Exact waypoint count and junction placement per chapter; how much of each
  branch is exclusive content vs shared spine.
- Whether legs pause at arrival indefinitely with zero pressure, or hope
  ticks gently down after ~3 days of holding station (proposed: the latter,
  softly — the souls are eager).
- How rumor acquisition is priced (supplies? keepsakes? a soul's time?).
- How much variance a road carries beyond its known character — proposed:
  little; a road should mostly keep its promises, so junctions stay
  strategy rather than gambling at one remove.
- Whether the rune-fragment shortcut meta-puzzle (Deadreckoning garnish)
  ships in v1 or as a post-arrival content update.

## 2026-03-27-vessel-combat-design.md

# Vessel Auto-Combat

> **⚠ SUPERSEDED (2026-07-02).** Act 2's direction changed to *The Pilgrimage
> of Souls* — see the rewritten parent spec and
> [2026-07-02-act2-voyage-experience-exploration.md](2026-07-02-act2-voyage-experience-exploration.md).
> Replaced by leg ambience and named waypoint threats (spec 5: The Vessel Underway). The enemy-tier ladder and scaling formulas are cut. Kept for salvage, not for implementation.
>
> **Confirmed abandoned (2026-07-04 doc-alignment pass):** nothing in this
> spec shipped. There is no combat module, no Firepower/Hull/Engines/Sensors
> ship stats, no distance-based encounter scaling, and no Norse boss gates
> anywhere in `src/vessel/`. The Voyage has no combat system at all — weather
> (`weather.rs`), nights (`nights.rs`), and named waypoint threats (route/scene
> content) fully replaced it, per `src/vessel/CLAUDE.md`.

**Parent spec:** `docs/superpowers/specs/2026-03-27-the-vessel-design.md`
**Sub-project:** 4 of 7
**Depends on:** Sub-project 3 (Room System & Ship Stats)

## Overview

Ship combat uses the same tick-based auto-attack engine as Act 1 hero combat, reskinned as ship volleys. The visual presentation is minimal — a void star field with threat indicators, HP bars, and damage numbers. Enemies scale smoothly with distance, with elite encounters at milestones and Norse mythological bosses at major distance gates.

## Combat Engine

Reuses the existing combat pipeline with ship stats mapped to hero stats. Note: the Act 1 combat stat pipeline was migrated from `u32` to `u64` (#619) — vessel combat stats should be `u64` from the start.

| Ship Stat | Maps To | Role |
|-----------|---------|------|
| Firepower | Player attack power | Damage dealt per volley |
| Hull (current) | Player HP | Damage absorbed before drift |
| Hull (max) | Player max HP | Total survivability |
| Engines | Evasion | Chance to dodge enemy volleys |
| Sensors | Crit chance | Higher Sensors = more crits |

### Attack Timing

Same tick-based intervals as Act 1:
- **Ship attack interval:** 1.5s (same as player)
- **Enemy attack interval:** varies by type (2.0s normal, 1.5s elite, 1.2s boss)

### Damage Pipeline

Full pipeline defined in the Room System & Ship Stats spec (`vessel-rooms-stats-design.md`). Summary:

```
Ship damage to enemy:
  Final Firepower + Rune Array flat bonus (optional) → - enemy defense → min 1 → crit check (Sensors-based) → final damage

Enemy damage to ship:
  Enemy attack → - Hull defense → min 1 → Engines evasion check → final damage to Hull HP
```

**Crit:** `chance = 5% + (Sensors / (Sensors + enemy_stealth)) × 25%`, capped at 30%. Base crit multiplier: 2.0x (components can increase).

**Evasion:** `dodge_chance = Engines / (Engines + enemy_accuracy)`, capped at 50%. A dodge negates the entire volley.

**Rune Array:** If the Rune Array room is built, old-world Transmissions convert into a flat Firepower bonus added before enemy defense. This is optional — costs a room slot and power budget.

### HP and Regen

- **Enemy HP:** determined by distance formula + type modifier
- **Ship Hull:** current hull is the ship's HP. Damaged hull stays damaged until repaired (Workshop room, supplies, or events).
- **No auto-regen between fights.** Hull damage persists. This is the key tension — each fight costs hull that must be actively repaired. Unlike Act 1 where HP regens after kills.

### Kill Rewards

On enemy defeat:
- **Ship XP** — `xp = 50 × (1 + distance/50)^0.5` × type modifier (common 1x, elite 2.5x, boss 10x). At the plateau (~48 kills/day at 9,000 ly ≈ 670 XP each) the ship gains a level every ~2-3 days against the `500 × level^1.3` curve.
- **Salvage** — primary upgrade currency (for room builds/levels)
- **Components** — rare drops, chance scales with enemy tier
- **Fuel/Supplies** — small amounts from scavenging the wreck

## Encounter Frequency

Two components: an **ambient wall-clock rate** (the void is never fully empty) plus a **speed-driven rate** (faster ship = more fights per day):

```
encounters_per_day = 3 + 1.5 × speed_ly_per_day     (capped at 48/day — one per 30 min)
```

- At launch (0.1 ly/day): ~3/day — a fight every ~8 hours. Sparse and lonely, but alive, and enough salvage income to bootstrap the first room builds.
- Mid voyage (10 ly/day): ~18/day.
- Cruise plateau (60-100 ly/day): capped at 48/day — constant combat.

**The ambient component continues while the ship is halted** — blocked at a boss milestone or recovering in drift, the void comes to you. This guarantees salvage income even when progress is stopped, so a too-weak ship grinds up to a boss rather than soft-locking.

Between encounters, the void view shows the peaceful star field.

## Enemy Scaling

### Normal Enemies (formula-based, smooth power curves)

Distance spans three orders of magnitude, so scaling is **sublinear** — linear scaling would produce four-digit attack values no ship stat budget can match. First-pass formulas (simulator-validated before implementation):

```
scale = 1 + distance / 50

enemy_hp       = 20 × scale^0.90
enemy_attack   =  5 × scale^0.70
enemy_defense  =  3 × scale^0.65
enemy_accuracy = 10 × scale^0.50
enemy_stealth  = 10 × scale^0.50
```

Anchors (common enemies, before type modifiers):

| Distance | HP | Attack | Defense | Accuracy |
|----------|-----|--------|---------|----------|
| 0 ly | 20 | 5 | 3 | 10 |
| 100 ly | 54 | 11 | 6 | 17 |
| 1,000 ly | 310 | 42 | 22 | 46 |
| 9,000 ly | 2,150 | 190 | 88 | 135 |

The exponents are tuned so late-voyage ship stats (final stats in the hundreds after all four layers) stay in the same magnitude band as enemy stats — fights get harder but the subtractive damage pipeline never degenerates into always-min-1 or one-shots.

Stats have ±15% random variance per encounter.

### Enemy Types

Type thresholds sit on the same geometric ladder as room slots, so new enemies appear at a steady wall-clock cadence.

**Common — Void Creatures** (70% of encounters):
| Type | Distance | Modifier | Flavor |
|------|----------|----------|--------|
| Void Wisp | 0+ ly | 0.5x stats | Faint, barely hostile |
| Branch Parasite | 25+ ly | 0.8x stats | Feeds on wood-matter |
| Root Worm | 100+ ly | 1.0x stats | Burrowing void dweller |
| Cosmic Stalker | 400+ ly | 1.2x stats | Hunts between branches |
| Void Leviathan | 1,600+ ly | 1.5x stats | Massive, slow, devastating |
| Abyss Tendril | 4,000+ ly | 1.8x stats | Reaches from the deep void |
| Entropy Shade | 6,400+ ly | 2.0x stats | Reality itself dissolving |

The highest-tier type available for the current distance is used, with a weighted random roll favoring newer types.

**Elite — Lost Vessels** (20% of encounters):
| Type | Distance | Modifier | Special |
|------|----------|----------|---------|
| Drifting Hulk | 100+ ly | 1.5x stats | Slow attacks, high HP |
| Ghost Frigate | 640+ ly | 1.8x stats | Fast attacks, evasive |
| Corrupted Warship | 2,500+ ly | 2.2x stats | Heavy damage, heavy defense |
| Abyssal Dreadnought | 6,400+ ly | 2.8x stats | End-tier elite |

Elites always drop salvage. Higher component drop rate (20% vs 5% for common).

**Bosses — Norse Mythological** (at distance milestones):
| Boss | Distance | Modifier | Drop |
|------|----------|----------|------|
| Níðhöggr's Fang | 50 ly | 3x stats | Guaranteed component + room unlock |
| Hræsvelgr's Wake | 400 ly | 4x stats | Guaranteed component |
| Jörmungandr Fragment | 1,600 ly | 5x stats | Guaranteed rare component |
| Fenrir's Shadow | 4,500 ly | 7x stats | Guaranteed rare component |
| Surtr's Ember | 9,200 ly | 10x stats | Guaranteed legendary component |

Under the intended speed trajectory these land roughly at months ~2.5, 4, 5, 6, and 7.5 of the voyage — one boss per phase of the ship's growth.

Bosses are one-time encounters. They block passage — you must defeat them to continue past their distance milestone. Attack interval: 1.2s. After defeat, a narrative moment plays.

## Combat Visuals

Minimal HUD overlay on the void star field. No detailed ship sprites fighting.

```
┌─ The Void ──────────────────────────────┐
│                                         │
│  · ·    ·        ·    ·                 │
│     ·        ╱═══╲        ·    ·        │
│  ·          ╱ ◆◆◆ ╲                    │
│            ═══════════    ·             │
│  ·    ·       ║║║║      ·          ·   │
│         ·          ·         ·          │
│                                         │
│  ── THREAT ──────────────────────       │
│  Root Worm                ★ Common      │
│  HP: ██████████░░░░  68%                │
│                                         │
│  Ship Hull: █████████░  92%             │
│                                         │
│  -12 ↑  -8 ↓    -15 ↑   DODGE          │
│                                         │
├─────────────────────────────────────────┤
│  ⚔ Root Worm takes 12 damage            │
│  ↓ Ship hull -8 (92%)                   │
│  ⚔ Root Worm takes 15 damage            │
│  ✦ Root Worm evaded! (DODGE)            │
└─────────────────────────────────────────┘
```

- Damage numbers float briefly above/below the ship art
- "DODGE" flashes when Engines evasion triggers
- Enemy name and type shown with a tier indicator (★ Common, ★★ Elite, ★★★ Boss)
- HP bars for both enemy and ship hull
- Combat log scrolls at the bottom of the void view

When no encounter is active, the threat area is empty and the void feels peaceful.

## Death Handling

If hull reaches 0 during combat:
- Combat ends immediately (enemy doesn't finish you off)
- Ship enters **drift state** (defined in sub-project 2)
- Current enemy despawns (not defeated, no loot)
- No distance is lost from the combat itself

Boss encounters that reduce hull to 0: boss resets to full HP. Player must recover from drift and try again.

## Loot System

### Salvage

Primary currency. Dropped by all enemies (sublinear, same rationale as enemy scaling — room costs are static, so linear salvage would trivialize late-game upgrades):

```
salvage_base = 5 × (1 + distance/50)^0.6
```

Anchors: ~5 at launch, ~31 at 1,000 ly, ~113 at 9,000 ly.

- Common enemies: 1.0x salvage
- Elite enemies: 2.5x salvage
- Bosses: 10x salvage

### Components

Dropped by enemies, installed in room component slots:

| Source | Drop Rate | Quality |
|--------|-----------|---------|
| Common enemy | 5% | Common component |
| Elite enemy | 20% | Uncommon+ component |
| Boss | 100% | Rare+ component |

Component quality tiers (like item rarity): Common, Uncommon, Rare, Legendary. Higher quality = bigger stat bonuses or more unique effects.

### Fuel & Supplies

Small amounts scavenged from defeated enemies:
- Common: 0-1 fuel, 0-1 supplies
- Elite: 2-4 fuel, 1-2 supplies
- Boss: 25 fuel, 15 supplies

Deliberately lean: scavenging covers most fuel drain through the mid voyage but runs a deficit at the cruise plateau, where harvesting + Refinery take over (see mode-transition spec, Fuel economy design intent).

## Files

| File | Change |
|------|--------|
| `src/vessel/combat.rs` | New: encounter generation, damage pipeline, evasion, loot drops |
| `src/vessel/enemies.rs` | New: enemy types, scaling formulas, boss definitions |
| `src/vessel/types.rs` | Modify: add combat state, encounter tracking to VesselState |
| `src/vessel/tick.rs` | Modify: integrate combat ticks into voyage tick |
| `src/ui/vessel_scene.rs` | Modify: add threat HUD, combat log, damage numbers |

## Testing

- Unit test: enemy stat scaling formula at various distances
- Unit test: encounter frequency scales with speed and distance
- Unit test: evasion calculation from Engines stat
- Unit test: damage pipeline (Firepower → defense → min 1)
- Unit test: hull damage persists between encounters
- Unit test: drift triggers on hull reaching 0
- Unit test: boss blocks passage (can't pass milestone without defeating)
- Unit test: loot drops scale with distance and enemy type
- Unit test: boss encounters are one-time (don't repeat after defeat)
- Unit test: elite/common/boss spawn rates match expected distribution

## 2026-03-27-vessel-crew-design.md

# Vessel Crew System

> **⚠ SUPERSEDED (2026-07-02).** Act 2's direction changed to *The Pilgrimage
> of Souls* — see the rewritten parent spec and
> [2026-07-02-act2-voyage-experience-exploration.md](2026-07-02-act2-voyage-experience-exploration.md).
> Rewritten as The Souls (spec 3): arcs, hope, and memorial replace stat multipliers. Capacity, injury, and offline-protection bones survive. Kept for salvage, not for implementation.
>
> **Confirmed abandoned (2026-07-04 doc-alignment pass):** nothing in this
> spec shipped — no Specialty/ShipTrait/RoomTrait system, no skill levels,
> no `[C]` crew screen, no `crew.rs`. What shipped instead (`src/vessel/souls.rs`)
> is 8 authored named souls competing for 7 fixed berths across 3 stations
> (Helm/Tender/Watch) — structurally unrelated to this doc's design. Note
> also: "hope" in this note's own superseded-by text is itself now stale —
> Hope was retired entirely (commit d39ad67); memorial/loss (`mark_lost()`)
> survives, arcs survive, hope does not.

**Parent spec:** `docs/superpowers/specs/2026-03-27-the-vessel-design.md`
**Sub-project:** 5 of 7
**Depends on:** Sub-project 3 (Room System & Ship Stats)

## Overview

The Vessel carries 5-8 named crew members, each a unique individual with a specialty, a ship trait, and a room trait. Crew are found during the voyage and assigned to rooms to boost their output. Every crew member matters — losing one is significant.

## Crew Member Definition

Each crew member has:

- **Name** — generated (first name + epithet, like Deep mercs)
- **Specialty** (1 of 6) — determines room match bonus
- **Ship trait** (1 of 6) — passive bonus, always active
- **Room trait** (1 of 6) — bonus that only affects their assigned room
- **Skill level** (1-10) — grows passively over time while stationed in a room
- **Status** — Active, Injured, or Lost

### Specialties

Each specialty has primary rooms (high multiplier) and secondary rooms (moderate multiplier). All other rooms get a small mismatch multiplier.

| Specialty | Primary Rooms | Secondary Rooms |
|-----------|--------------|-----------------|
| Weapons | Weapons Bay | Hull Plating |
| Engineering | Reactor, Workshop | Engines |
| Navigation | Engines, Cartography Deck | Sensors Array |
| Medicine | Medbay, Life Support | Garden |
| Lore | Shrine, Fate Loom | Void Lens |
| Salvage | Forge, Shuttle Bay | Cargo Hold |

### Ship Traits (always active, regardless of room assignment)

| Trait | Effect |
|-------|--------|
| Lucky | +5% component drop rate |
| Resourceful | +10% salvage from encounters |
| Cautious | Ship takes 5% less hull damage |
| Inspiring | All other crew gain skill 10% faster |
| Vigilant | +5% evasion chance |
| Hardy | +5% fuel efficiency |

### Room Traits (only affect the assigned room)

| Trait | Effect |
|-------|--------|
| Industrious | Room output +15% |
| Efficient | Room power cost -1 |
| Precise | Room stat contribution +10% |
| Inventive | Room component effects +20% |
| Tireless | Room functions at full even when ship is over power budget |
| Adaptive | No specialty mismatch penalty in this room |

## Crew Multiplier

The crew multiplier is the third of four layers in the ship stat formula: `final_stat = base × room_multiplier × crew_multiplier × component_multiplier` (see `vessel-rooms-stats-design.md`).

Crew multiplier for a stat = product of all crew contributions to that stat. Most crew only contribute to the stat their assigned room affects.

### Specialty Match Bonus

Based on crew skill level and room match:

| Match | Multiplier at Lv 1 | Multiplier at Lv 10 |
|-------|-------------------|---------------------|
| Primary room | 1.10 | 1.50 |
| Secondary room | 1.05 | 1.25 |
| Other room | 1.02 | 1.10 |
| No crew assigned | 1.00 | — |

Formula: `match_base + (skill_level - 1) × growth_per_level`

| Match | Base | Growth/Level |
|-------|------|-------------|
| Primary | 1.10 | +0.044 |
| Secondary | 1.05 | +0.022 |
| Other | 1.02 | +0.009 |

## Skill Growth

Crew gain skill XP passively while stationed in a room. Time-based, not event-based.

- **Growth rate:** 1 skill level per ~3 days of being stationed (real wall-clock time)
- **XP curve:** `skill_xp_required(level) = 3 × 86400 × level` seconds (3 days × level number)
- Level 1→2: 3 days. Level 9→10: 27 days. Total to max: ~135 days.
- **Unassigned crew don't gain skill.**

Reassigning a crew member to a different room resets nothing — skill level is universal, not per-room.

## Crew Capacity

Base crew capacity: 2. Increased by Life Support room (+1 per level). Max capacity: 8 (Life Support level 6+).

| Life Support Level | Crew Capacity |
|-------------------|---------------|
| None built | 2 |
| Level 1 | 3 |
| Level 2 | 4 |
| Level 3 | 5 |
| Level 4 | 6 |
| Level 5 | 7 |
| Level 6+ | 8 |

## Recruitment

Crew are found during the voyage through narrative moments, not recruited from a pool. Every recruitment is a story event with a cost or risk.

### Sources

- **Rescue events** — "A drifting pod pings your sensors. Investigate? (costs 20 supplies)" → find a survivor
- **Trading posts** — hire a crew member for salvage (scaling cost with distance)
- **Derelict exploration** — explore a wreck, find someone in stasis
- **Boss rewards** — defeating a Norse boss may trigger a recruitment event with a higher starting skill crew member

### Frequency

Random with pity timer, measured in **wall-clock time** (distance is exponential across the voyage, so a distance-based pity would bunch all recruitment into the final months): recruitment events appear organically with a guarantee of at least one opportunity every ~3 weeks. The first opportunity appears within the first day of the voyage. Total across the ~8-month voyage: ~8-12 opportunities, so the player can be somewhat selective.

### Information Reveal

On recruitment, the player sees the crew member's **name and specialty only**. Traits are hidden and reveal over time:

- **Ship trait:** reveals after 1 day aboard the ship
- **Room trait:** reveals after 3 days stationed in any room

Until revealed, traits show as "???" in the crew panel. This creates a "getting to know your crew" arc.

### Capacity and Hard Choices

Recruitment is gated by Life Support crew capacity. If the ship is at capacity when a recruitment event fires:

- The player sees the new person's name and specialty
- They must choose: **dismiss an existing crew member** to make room, or **let the new person go**
- Dismissed crew are gone forever
- This creates meaningful "is this person better than who I have?" decisions

If the ship is below capacity, the player can simply accept or decline.

### Dismissal

Crew can be dismissed at any time from the crew management screen to free capacity. Dismissed crew are gone forever — no undo, no re-recruitment.

## Injuries and Loss

### Injuries

Crew can be injured during combat or dangerous events:
- **Light injury:** 1 day recovery. Crew can't be assigned during recovery.
- **Severe injury:** 3 day recovery.
- **Medbay reduces recovery time** by 10% per level.

Injury chance per combat encounter: 5% base, reduced by Hull stat and Cautious trait.

### Permanent Loss

Crew can be permanently lost from:
- **Catastrophic events** — rare decision events with high-risk choices
- **Hull reaching 0** — 20% chance per crew member of being lost when entering drift from combat

Lost crew are gone forever. This is the primary emotional stakes of the voyage.

**Offline protection:** permanent loss never happens offline — the drift crew-loss roll is skipped during offline resolution (see mode-transition spec, Offline Progression). Losing a named crew member must always trace to a decision the player was present for.

### Injury Protection

- Medbay prevents severe injuries from becoming permanent loss (same pattern as Deep Medic archetype)
- Cautious ship trait reduces injury chance
- High Hull stat reduces injury chance

## Supplies

Crew consume supplies proportional to headcount:
- **Drain rate:** 1 supply per crew member per day
- At 6 crew: 6 supplies/day
- Garden room generates supplies to offset this

When supplies hit 0:
- All crew effectiveness drops by 50% (multipliers halved)
- Morale penalty: skill growth pauses
- No crew loss from starvation — they survive but perform poorly

## Starting State

At launch:
- 0 crew aboard
- Crew capacity: 2 (no Life Support built yet)
- First crew member is offered during the first decision event (within the first day of the voyage)

## UI: Crew Management

Accessed via `[C]` hotkey from the voyage screen:

```
┌─ Crew (3/6 capacity) ────────────────────────────────┐
│                                                       │
│  1. Brynn                                             │
│     Specialty: Navigation  Skill: Lv 4                │
│     Ship: Vigilant (+5% evasion)                      │
│     Room: Precise (+10% stat contribution)            │
│     Assigned: Engines                        [Active] │
│                                                       │
│  2. Kael                                              │
│     Specialty: Weapons     Skill: Lv 2                │
│     Ship: Resourceful (+10% salvage)                  │
│     Room: Industrious (+15% output)                   │
│     Assigned: Weapons Bay                    [Active] │
│                                                       │
│  3. Lyra                                              │
│     Specialty: Lore        Skill: Lv 1                │
│     Ship: Lucky (+5% component drops)                 │
│     Room: Efficient (-1 power)                        │
│     Assigned: (none)                      [Unassigned]│
│                                                       │
│  [A] Assign  [U] Unassign  [D] Dismiss  [Esc] Back   │
└───────────────────────────────────────────────────────┘
```

Assigning shows a room picker with match quality indicators (Primary/Secondary/Other).

## Files

| File | Change |
|------|--------|
| `src/vessel/crew.rs` | New: crew generation, skill growth, injury, recruitment, assignment |
| `src/vessel/types.rs` | Modify: add CrewMember, Specialty, ShipTrait, RoomTrait to VesselState |
| `src/vessel/stats.rs` | Modify: integrate crew multiplier into stat derivation |
| `src/vessel/tick.rs` | Modify: tick skill growth, supply drain from crew |
| `src/ui/vessel_crew_scene.rs` | New: crew management overlay rendering |
| `src/input/vessel_input.rs` | Modify: add [C] hotkey and crew management input |

## Testing

- Unit test: crew multiplier calculation for primary/secondary/other match at various skill levels
- Unit test: skill growth rate (time to level up)
- Unit test: crew capacity scales with Life Support level
- Unit test: supply drain proportional to crew count
- Unit test: injury recovery time reduced by Medbay
- Unit test: crew loss chance on drift
- Unit test: each ship trait applies correct bonus
- Unit test: each room trait applies correct bonus
- Unit test: supplies at 0 halves crew effectiveness
- Unit test: dismiss removes crew permanently

## 2026-03-27-vessel-launch-gate-design.md

# Vessel Launch Gate & Construction Overlay

**Parent spec:** `docs/superpowers/specs/2026-03-27-the-vessel-design.md`
**Sub-project:** 1 of 7

## Overview

After conquering Zone 50, the player begins seeing hints about the dying branch of Yggdrasil. A new `[V]` hotkey opens the Vessel overlay showing progress toward launch. Prestige rank keeps ticking up as normal; the fuel gate is simply **holding 250,000 PR at once**. Launch is a single all-or-nothing burn — confirming deducts the full 250,000 in one action. There is no partial banking and no fuel accumulator. This sub-project covers everything up to the launch confirmation — no Act 2 gameplay yet.

## Phases

### Phase 1: Ticker Hints (after Z50 first clear)

When the player first clears Zone 50's final boss (subzone 5 boss kill), a `VesselSignalDiscovered` tick event fires. This sets a persistent flag `vessel_signal_discovered: bool` on `GameState`.

Once the flag is set, atmospheric ticker messages appear periodically (mixed in with normal loot):
- "The Loom trembles. Something distant answers."
- "A signal pulses from beyond the branches."
- "The Origin Thread frays. The roots grow cold."
- "The weave resonates with something far away."
- "Yggdrasil shudders. A beacon calls."

These fire roughly every 60 seconds. They use a dim color (e.g. `Color::Rgb(120, 90, 160)`) with a `✦` icon.

**Implementation note:** the scrolling ticker (`src/core/ticker.rs`) is purely event-driven — nothing currently pushes to it on a timer. This needs a small new mechanism: a wall-clock check in the tick system (elapsed >= 60s since last hint → push a random hint entry via `Ticker::push`). The closest existing wall-clock rotation precedent is the Deep hub's atmosphere rotation (`millis / 8000 % len` in `src/ui/deep_missions.rs`), but that rotates panel text rather than pushing ticker entries.

### Phase 2: Stats Panel Indicator

Once `vessel_signal_discovered` is true, the stats panel shows a new line:

```
Prestige: P52,340 (Eternal)
Ascension: X (324x)
✦ The branch withers...        [V] Vessel
```

The `[V] Vessel` hint pulses between dim and bright (using the existing tick millis for animation). Pressing `[V]` opens the Vessel overlay.

### Phase 3: Vessel Overlay

A full-screen overlay (like Haven, Deep, Soulforge) opened via `[V]`. Contains:

**Construction screen** showing the four requirements with check/cross indicators, a rank-progress bar toward the 250,000 burn, and a generation rate estimate.

The overlay renders into the scene buffer (same pattern as Deep overlay — `render_vessel_scene()` paints to a buffer, then the buffer is flushed to frame).

### Phase 4: The 250,000 PR Gate

**Prestige rank never freezes and PR grants are never diverted.** PR keeps ticking up from all sources exactly as before (WR→PR, Power Cores, challenges, prestige actions — none of that code is touched). The fuel gate is simply: **the player must hold 250,000 PR at the moment of launch.** There is no partial banking, no transfer controls, no `vessel_fuel` accumulator — the burn happens once, in full, when launch is confirmed.

The overlay shows current rank against the threshold:

```
Prestige: P152,840 / 250,000       ██████░░░░  61%
Income: ~7,320 PR/day — ready in ~13 days
```

**Consequences of this model:**
- The hero fights at full strength for the entire wait — rank (and its bonuses) stays intact until the single burn at launch.
- A veteran already holding 250k+ can launch the moment the signal appears. They earned it.
- A player arriving at the gate near P50,000 climbs to 250,000: ~108 days (~3.6 months) at typical pattern-28 rates (75 PR/hr), ~27 days at a maxed Loom (303 PR/hr). Maxing the extractors is effectively part of the launch grind at this price.
- The burn leaves `rank - 250,000` behind (usually a small remainder). Post-launch rank only matters to the background supply line (sub-project 7), and zone unlocks cannot re-lock (`sync_account_zone_unlocks` in `src/zones/access.rs` never removes an unlock), so there is no reason to over-save beyond the threshold.
- One dramatic moment instead of many small ones: the player watches everything they accumulated vanish in a single confirmed action. That IS the launch.

The overlay shows:
- Current prestige rank against the 250,000 threshold, with a progress bar (live — rank keeps ticking up during the watch)
- Estimated PR/day generation rate and days until the threshold is reached
- The other three requirements with ✓/✗

### Phase 5: Launch Ready

When `prestige_rank >= 250,000` and the other gates are met (28 patterns, Ascension X, signal discovered), the overlay changes to show a "Launch" option. The requirements section shows all green checkmarks. A `[Enter] Launch into the Void` prompt appears.

### Phase 6: Launch Confirmation

Pressing Enter on the ready screen shows a confirmation modal:
- The burn, stated plainly: `P253,218  →  P3,218` (before → after)
- Lists what will happen (250,000 PR consumed in one burn, the Loom transforms, the voyage begins)
- "There is no return."
- `[Enter] Confirm  [Esc] Cancel`

On confirm, in one action:

```
state.prestige_rank -= 250_000;
state.recalculate_prestige_bonuses();
state.derived_stats_dirty = true;
state.vessel_launched = true;
```

The actual mode transition to Act 2 is handled by a future sub-project — for now, launch performs the burn, sets the flag, and shows a "Coming soon" message or similar placeholder.

## State Model

New fields on `GameState` (with `#[serde(default)]`):

```rust
/// True after Z50 final boss first kill — enables ticker hints and [V] hotkey
pub vessel_signal_discovered: bool,
/// True after player confirms launch — triggers Act 2 mode shift (future sub-project)
pub vessel_launched: bool,
```

No separate module and no fuel accumulator needed. These two fields on `GameState` are sufficient for sub-project 1 — the fuel gate reads `prestige_rank` directly.

## Launch Burn Logic

A single gate check and a single deduction in the launch confirmation handler — no changes to any PR grant site:

```
fn can_launch(state: &GameState, loom: &LoomState) -> bool {
    state.vessel_signal_discovered
        && !state.vessel_launched
        && state.ascension_level >= 10
        && crate::loom::all_patterns_complete(&loom.persistent)
        && state.prestige_rank >= 250_000
}
```

(`vessel_signal_discovered` already implies Z50 was cleared — no separate `z50_cleared` flag is needed.)

The deduction happens exactly once, at confirmation (see Phase 6). This is the simplest possible model: PR grants remain untouched at all five sites (prestige action, WR→PR, Power Cores online/offline, challenge rewards), achievement passive-PR tracking (#612) keeps working unmodified, there is no partial state to persist, and the hero fights at full prestige bonuses for the entire wait.

## Z50 Detection

Zone 50 is a Loom cap zone — killing its subzone 5 boss produces `BossDefeatResult::LoomZoneCycle { zone_id: 50 }` (see `src/zones/boss_defeat.rs`). There is no distinct "final boss of the game" result; Z50 cycles like any cap zone.

Detection follows the exact precedent of the Deep discovery in `src/core/tick_stages.rs` (`process_combat_events`, ~line 490): match the defeat result, set the flag, emit a tick event:

```rust
if matches!(defeat_result, BossDefeatResult::LoomZoneCycle { zone_id: 50 })
    && !state.vessel_signal_discovered
{
    state.vessel_signal_discovered = true;
    result.events.push(TickEvent::VesselSignalDiscovered);
}
```

The presentation layer handles the event the same way `TickEvent::DeepDiscovered` is handled (`src/tick_events.rs` → flag → `src/main.rs` pushes a discovery overlay onto `pending_overlays`).

## Input Integration

The codebase has two overlay mechanisms: the `GameOverlay` enum (`src/input/types.rs`) for one-shot modals/celebrations, and standalone `*UiState` structs (like `DeepUiState`, `LoomUiState`) held in `main.rs` for large interactive scenes. The Vessel uses both:

- **Discovery modal:** `GameOverlay::VesselDiscovery` unit variant — the one-time reveal celebration when the signal is discovered (same as `DeepDiscovery`)
- **Construction overlay:** a new `VesselUiState` struct with a `showing: bool` (the Deep/Loom pattern), dispatched from Step 2 of the input priority chain via a new `src/input/vessel_input.rs`
- **Hotkey:** `[V]` in the base-hotkey block (Step 9 of the priority chain in `src/input/mod.rs`), gated by `vessel_signal_discovered`
- **Within overlay:** `[Esc]` closes. `[Enter]` triggers launch when ready. Arrow keys unused for now.

## UI Rendering

Single scene rendered into a buffer (like Deep). No sub-views for sub-project 1 — just the construction screen.

Layout:
```
Row 0-8:   Ship ASCII art + narrative text
Row 9:     Separator
Row 10-13: Four requirement lines with ✓/✗ (PR line shows rank / 250,000)
Row 14-15: Rank progress bar toward 250,000
Row 16:    Generation rate + days-until-ready estimate
Row 17:    Footer ([Enter] Launch / [Esc] Close)
```

## Files Changed

| File | Change |
|------|--------|
| `src/core/game_state.rs` | Add `vessel_signal_discovered`, `vessel_launched` fields |
| `src/core/tick_types.rs` | Add `VesselSignalDiscovered` tick event variant |
| `src/core/tick_stages.rs` | Detect Z50 boss kill, emit event |
| `src/tick_events.rs` | Handle `VesselSignalDiscovered` flag, add ticker messages |
| `src/input/mod.rs` | Add `[V]` hotkey, vessel overlay dispatch |
| `src/input/types.rs` | Add `GameOverlay::VesselDiscovery` variant + `VesselUiState` |
| `src/ui/vessel_scene.rs` | New file: render construction overlay |
| `src/ui/mod.rs` | Register vessel_scene module |
| `src/ui/stats_panel.rs` | Show vessel indicator line |
| `src/main.rs` | Wire vessel overlay into render loop |

## Release Staging (kill-switch)

Sub-project 1 can merge to main **dark**: `vessel::ACT2_ENABLED = false` keeps the entire feature invisible until Act 2 is deliberately launched.

| Layer | Behavior while disabled |
|-------|------------------------|
| Z50 detection | **Still records** `vessel_signal_discovered` in saves (silently) — qualified players light up the instant Act 2 is enabled, no re-kill needed |
| Discovery modal, log line, ticker entry | Suppressed (`src/tick_events.rs`) |
| Ticker whispers | Suppressed (stage 12c gate in `src/core/tick.rs`) |
| Stats panel row | Hidden (`src/ui/stats_panel.rs`) |
| `[V]` hotkey | Inert (`src/input/mod.rs`) — with no overlay, the launch burn is unreachable |

**To launch Act 2:** flip `ACT2_ENABLED` to `true` in `src/vessel/mod.rs` and update the `act2_kill_switch_is_off_for_release` release-guard test in the same file (deliberately a two-line change so the switch can't flip by accident). The push-to-main release pipeline ships it like any other change.

**To preview on any build** (dev, beta testers, drive-game screenshots): run with `QUEST_ACT2=1` in the environment — the runtime check `vessel::act2_enabled()` honors the override without recompiling.

## Testing

- Unit test: `can_launch` requires all four gates (signal, Ascension X, 28 patterns, 250,000 PR)
- Unit test: launch deducts exactly 250,000, recalculates prestige bonuses, sets `vessel_launched`
- Unit test: launch refused below 250,000 PR and after already launched
- Unit test: PR grants (WR→PR, Power Cores, challenges) are untouched during the wait — rank keeps rising
- Unit test: `vessel_signal_discovered` set on Z50 boss kill
- Unit test: serde round-trip for new GameState fields (backwards compat)
- Add save-format compatibility fixtures for the new fields (fixture corpus from #626)
- Snapshot test: construction overlay scene (full-frame TUI snapshot infra from #623/#624)

## 2026-03-27-vessel-mode-transition-design.md

# Vessel Mode Transition & Basic Voyage Shell

> **⚠ SUPERSEDED (2026-07-02).** Act 2's direction changed to *The Pilgrimage
> of Souls* — see the rewritten parent spec and
> [2026-07-02-act2-voyage-experience-exploration.md](2026-07-02-act2-voyage-experience-exploration.md).
> The 5-beat launch transition, gauge/drift model, and offline rules survive into the new spec queue; the continuous-distance shell does not. Kept for salvage, not for implementation.
>
> **Doc-alignment note (2026-07-04):** confirmed the continuous-distance
> shell (fuel/hull/distance/void-matter/harvesting below) is fully
> abandoned — no such fields exist anywhere in `src/vessel/`; Provisions +
> the route-graph model replaced it entirely.
>
> **The 5-beat launch transition shipped this same day**, as part of this
> design-iteration pass: `src/vessel/transition.rs` (the beat content and
> `LaunchTransitionState` state machine), rendered by
> `ui::vessel_scene::render_launch_transition()`, gated by the new
> persistent `GameState::vessel_transition_played` flag and wired into
> `main.rs`'s `'game_loop` right before the Voyage takes over. The five
> beats (Farewell/Unweaving/Construction/Launch/Void) keep the story and
> structure below, with two implementation differences from the original
> design: presentation is static full-screen text per beat exactly as this
> spec allowed ("static text screens per beat are sufficient" — no
> character-scatter/dissolve animation was added), and a small `"N / 5 —
> <heading>"` marker was added in the corner (not in the original design)
> so the player always knows a fixed-length sequence is playing, not an
> indefinite loading screen. Covered by `overlay_snapshot_tests.rs`
> (`snapshot_launch_transition_first_beat`/`_final_beat`) and
> `transition.rs`'s own unit tests.

**Parent spec:** `docs/superpowers/specs/2026-03-27-the-vessel-design.md`
**Sub-project:** 2 of 7
**Depends on:** Sub-project 1 (Launch Gate)

## Overview

After the player confirms launch in the Vessel overlay, a 5-beat narrative transition plays where the old UI visually transforms into the Act 2 UI. Then the basic voyage shell appears: two-column layout with ship stats on the left and void view on the right. The ship moves, fuel drains, hull exists, and the ticker shows voyage events. No combat, rooms, crew, or events yet — just the skeleton.

## Launch Transition: 5-Beat Narrative Sequence

Each beat advances with Enter. The UI visually transforms as the story progresses.

### Beat 1: Farewell

The normal game UI is still visible but dimming (darken all colors by ~50%). Text overlay centered:

```
The Origin Thread has spoken.
This branch of Yggdrasil is dying.
Everything you built was preparation for this moment.

                         [Enter]
```

### Beat 2: Unweaving

The stats panel and combat scene dissolve — characters scatter/fall (random positions each frame, like static noise). The left panel disintegrates first.

```
The Loom unweaves itself.
Twenty-eight patterns unspool into something new.
Woven fate becomes woven hull.

                         [Enter]
```

### Beat 3: Construction

The scattered characters reform into a ship shape in the center. ASCII art builds itself (reveal line by line or character by character).

```
Reality folds. The Deep's roots become a keel.
Haven's stones become ballast. The forge becomes an engine.
A vessel takes shape from everything you were.

                         [Enter]
```

### Beat 4: Launch

Ship art complete and centered. Stars begin appearing. A bright line extends rightward toward the destination beacon.

```
A signal from 10,000 light-years away.
A living branch. The last hope of a dying tree.

The Vessel launches into the void.

                         [Enter]
```

### Beat 5: Void

Stars streak horizontally (speed lines). Ship is small, centered. Screen is mostly dark. This is the final beat before the Act 2 UI appears.

```
         ·  ·
   ·          ·    ·
·  ·    ╱═══╲
       ╱ ◆◆◆ ╲    ·
·     ╱═══════╲        ·
     ═══════════    ·
·        ║║║║          ·
     ·        ·  ·

                         [Enter]
```

After beat 5, transition to the Act 2 UI.

## Implementation Notes for Transition

The transition is a sequence of full-screen renders, not an overlay. While the transition is playing:
- Game tick is paused (or runs but input is blocked)
- Each beat renders a full frame
- Enter advances to next beat
- No Esc/cancel — launch was already confirmed

The visual effects (dimming, dissolving, reforming) can be simple:
- Beat 1: render normal UI with darkened colors
- Beat 2: fill screen with random characters from a sparse set, fading
- Beat 3: reveal ship ASCII art progressively
- Beat 4: ship art + star field + beacon line
- Beat 5: star streaks + small ship

These can be refined later. For initial implementation, static text screens per beat are sufficient — animation can be layered on.

## Act 2 UI: Two-Column Voyage Shell

Same structure as Act 1 (stats left, activity right, ticker bottom) but with new content.

### Left Panel: Ship Stats

```
┌─ The Vessel ──────────────┐
│ Distance: 0.3 / 10,000 ly │
│ Speed:    0.1 ly/day       │
│                            │
│ ── STATS ──                │
│ Firepower:   12            │
│ Hull:        80/80         │
│ Engines:     1             │
│ Sensors:     5             │
│                            │
│ ── RESOURCES ──            │
│ Fuel:    ████████░░ 80%    │
│ Hull:    ██████████ 100%   │
│ Supplies:███████░░░ 70%    │
│                            │
│ ── CREW ──                 │
│ 0/8 aboard                 │
│ (none yet)                 │
│                            │
│ ── ROOMS ──                │
│ 4/20 slots                 │
│ Reactor     Lv 1           │
│ Engines     Lv 1           │
│                            │
│ Transmissions: 1,100 PR/d  │
└────────────────────────────┘
```

Shows at a glance: how far you've gone, how fast, your stats, resource bars, crew roster summary, built rooms, and supply line income.

### Right Panel: Void View

```
┌─ The Void ──────────────────────────────┐
│                                         │
│  · ·    ·        ·    ·                 │
│     ·        ╱═══╲        ·    ·        │
│  ·          ╱ ◆◆◆ ╲                    │
│            ═══════════    ·             │
│  ·    ·       ║║║║      ·          ·   │
│         ·          ·         ·          │
│  ·           ·          ·               │
│                                         │
│  (The void stretches ahead)             │
│                                         │
│                                         │
├─────────────────────────────────────────┤
│  ✦ Launched from the dying branch       │
│  ✦ Fuel harvested: +2 from void matter  │
└─────────────────────────────────────────┘
```

The void view has:
- Star field background (animated, parallax with speed)
- Ship ASCII art centered
- Combat area (below ship, empty in sub-project 2)
- Lower section: recent events (like the existing ticker/combat log area)

### Bottom: Ticker

Same scrolling ticker component, now showing voyage events instead of loot drops.

### Footer

```
[R] Rooms  [C] Crew  [E] Events  [Esc] Menu
```

These hotkeys are stubs in sub-project 2 — they'll be wired in later sub-projects.

## Basic Voyage Tick Model

The voyage runs on the same 100ms tick as Act 1. Per tick:

1. **Distance increment:** `distance += speed_ly_per_day / (10 * 86400)` (10 ticks/sec, 86400 sec/day). Speed derives from the final Engines stat (`Engines² / 1,000`, see rooms spec); in this sub-project (no rooms system yet) it is the constant 0.1 ly/day.
2. **Fuel drain:** `fuel_drain_per_day = 2 + 0.8 × speed`, applied per tick. Faster travel costs more per day but *less per light-year* — economies of scale reward engine investment.
3. **Harvesting:** `void_matter_per_day = 2 + 0.2 × Sensors(final)`, applied per tick (see Void Matter Harvesting below).
4. **Supply drain:** `supplies -= supply_drain_per_tick` (proportional to crew count; 0 crew = 0 drain)
5. **Drift check:** if fuel <= 0 or hull <= 0, enter drift state

Starting values:
- Distance: 0.0 ly
- Speed: 0.1 ly/day (Engines 10)
- Fuel: 100% (1,000 units, drain ~2.1/day at launch speed — a ~480-day tank, deliberately generous before the Refinery unlocks at 40 ly)
- Hull: 100% (100 HP; no drain without combat)
- Supplies: 100% (500 units; no drain without crew)
- Void matter: 0 (capacity 200)

These numbers are first-pass — the vessel simulator validates them with combat and harvesting in the loop.

## Void Matter Harvesting

The third resource pillar, alongside combat scavenging and production rooms. The ship's collectors passively sweep void matter as it travels — no player action required, fitting the idle design.

- **Harvest rate:** `void_matter_per_day = 2 + 0.2 × Sensors(final)`. At launch (Sensors 10): 4 VM/day. Late voyage (Sensors ~200+): 40+ VM/day. This gives Sensors a continuous economic role on top of crits and event reveals.
- **Storage:** void matter has its own cap (200 base, +10% per Cargo Hold level).
- **Uses:** (1) the **Fuel Refinery** converts it to fuel at 2 VM → 1 fuel, throughput 5 VM/day per Refinery level; (2) the **Forge** consumes it as a crafting input for components; (3) some decision events ask for it ("feed the anomaly").
- **Without a Refinery** (unlocks at 40 ly), void matter accumulates but cannot become fuel — the early game runs on the launch tank and combat scavenge.
- **Density pockets:** decision events and Sensors discoveries can grant burst harvests ("+50 VM from a nebula pocket").

**Fuel economy design intent:** combat scavenging covers roughly 60-120% of drain through the early and mid voyage; at the cruise plateau scavenging alone runs a deficit and harvesting + Refinery closes the gap, with event caches as the buffer. If the simulator shows the late-game deficit is structural, the designed relief valve is an **engine throttle** (run below max speed to cut drain) rather than more income.

## Drift State

When fuel hits 0:
- Speed drops to 0
- Ship stops moving
- "DRIFT" indicator appears on the void view
- Transmissions from old world slowly restore fuel (1 unit per transmission PR, converted via base rate)
- Player can't do much except wait for recovery

When hull hits 0:
- Same drift state
- Transmissions slowly restore hull

Recovery from drift takes time but is automatic. No player death.

## Vessel State Model

New struct for Act 2 state:

```rust
pub struct VesselState {
    pub distance_ly: f64,        // Current distance traveled
    pub speed_ly_per_day: f64,   // Current speed
    pub fuel: f64,               // 0.0 - 1000.0
    pub fuel_capacity: f64,      // Max fuel
    pub hull: f64,               // 0.0 - 100.0
    pub hull_max: f64,           // Max hull
    pub supplies: f64,           // 0.0 - 500.0
    pub supplies_capacity: f64,  // Max supplies
    pub void_matter: f64,        // 0.0 - void_matter_capacity
    pub void_matter_capacity: f64, // Max void matter (200 base)
    pub drifting: bool,          // In drift state
    pub ship_level: u32,         // XP-based level
    pub ship_xp: u64,            // Accumulated XP
    // Room and crew fields added in later sub-projects
}
```

Persisted to `~/.quest/vessel.json` (separate file, like Deep and Loom).

## Offline Progression

Same wiring pattern as offline XP and Deep mission resolution (`resolve_deep_offline` in `src/main_helpers/offline.rs` is the template). One design principle governs all of it: **the voyage continues while you're away, but nothing irreversible happens to people.** Capped at 7 days (same as Act 1); beyond the cap the ship holds station.

| System | Offline behavior |
|--------|-----------------|
| Travel, fuel/supply drain, harvesting | 100% rate — these are wall-clock systems already |
| Workshop repair, Garden production | 100% rate |
| Crew skill growth | 100% rate (spec'd as wall-clock ~3 days/level; pausing it would contradict the crew spec) |
| Combat | Resolved statistically at **50% encounter rate** using expected-value outcomes (no crit/variance rolls): ship XP, salvage, and scavenged fuel/supplies granted in aggregate; hull takes expected net damage (incoming minus repair) |
| Crew injuries | Possible offline at reduced rate, but **never permanent loss** — the drift crew-loss roll is skipped offline |
| Bosses | **Never auto-fought.** The ship halts at the boss milestone; ambient encounters continue banking salvage (see combat spec); the boss waits for the player |
| Drift | If expected hull or fuel hits 0 mid-offline, combat stops at that timestamp and the remaining offline time applies drift recovery instead |
| Decision events | Don't fire on their wall-clock schedule; up to **3 queue** as "signals logged by the Sensors array," presented one at a time on return. Events beyond the queue cap auto-resolve via the safe path (no resource loss, rewards missed). Recruitment (pity-timer) events always queue and never expire |

Rationale: full-rate travel preserves the idle fantasy ("my ship sailed while I slept"). Half-rate combat means an offline player arrives at distance-gated content slightly underleveled — which self-corrects, because a too-weak ship loses hull and drift halts travel. And permanent crew loss must always trace to a decision the player was present for; losing a named crew member while asleep is rage-quit material.

## Files

| File | Change |
|------|--------|
| `src/vessel/mod.rs` | New module: public API re-exports |
| `src/vessel/types.rs` | New: VesselState, VesselUiState |
| `src/vessel/tick.rs` | New: voyage tick (distance, fuel, drift) |
| `src/vessel/persistence.rs` | New: save/load vessel.json |
| `src/vessel/transition.rs` | New: launch transition beat state machine |
| `src/ui/vessel_scene.rs` | New: Act 2 main render (two-column layout) |
| `src/ui/vessel_transition.rs` | New: transition beat rendering |
| `src/input/vessel_input.rs` | New: Act 2 input handling |
| `src/main.rs` | Wire vessel tick, render, save/load, mode switching |
| `src/core/game_state.rs` | Add `vessel_launched` check for mode routing |
| `src/lib.rs` | Register vessel module |

## Testing

- Unit test: distance increments correctly per tick at given speed
- Unit test: fuel drains proportional to speed
- Unit test: drift triggers when fuel hits 0
- Unit test: drift triggers when hull hits 0
- Unit test: offline progression calculates correct distance/fuel
- Unit test: offline combat resolves at 50% encounter rate with expected-value outcomes
- Unit test: offline drift mid-window stops combat and applies recovery for remaining time
- Unit test: offline event queue caps at 3, overflow auto-resolves safe
- Unit test: harvesting rate scales with Sensors; void matter respects capacity
- Unit test: transition beat state machine advances correctly
- Unit test: serde round-trip for VesselState
- Snapshot tests: voyage shell layout and each transition beat (full-frame TUI snapshot infra from #623/#624)

## 2026-03-27-vessel-rooms-stats-design.md

# Vessel Room System & Ship Stats

> **⚠ SUPERSEDED (2026-07-02).** Act 2's direction changed to *The Pilgrimage
> of Souls* — see the rewritten parent spec and
> [2026-07-02-act2-voyage-experience-exploration.md](2026-07-02-act2-voyage-experience-exploration.md).
> Replaced by waypoint Refits (spec 5: The Vessel Underway). Ship XP/levels, the room grid, and the Reactor power budget are cut. Kept for salvage, not for implementation.
>
> **Confirmed abandoned (2026-07-04 doc-alignment pass):** nothing in this
> spec shipped — `src/vessel/mod.rs`'s module list has no `rooms`/`components`/
> `stats` file, and no Room/Component/Reactor types exist anywhere in
> `src/vessel/`. The only surviving word "room(s)" in the codebase is flavor
> prose describing the harbor, never a system.

**Parent spec:** `docs/superpowers/specs/2026-03-27-the-vessel-design.md`
**Sub-project:** 3 of 7
**Depends on:** Sub-project 2 (Voyage Shell)

## Overview

The ship's power comes from four multiplicative layers: base ship stats (from XP/leveling), room bonuses (from built and upgraded rooms), crew assignments (from stationed crew members), and component bonuses (from items installed in room slots). Rooms are the core build system — ~20 types unlocked at distance milestones, built into slots, upgraded with levels and components, constrained by Reactor power.

There is no free scaling from distance — all power is earned through rooms, crew, components, and ship XP. The Rune Array room can optionally convert old-world Transmissions into combat bonuses, but this costs a room slot.

## Ship Stats

| Stat | Combat Role | Non-Combat Role |
|------|------------|-----------------|
| Firepower | Damage dealt to enemies | — |
| Hull | HP pool + defense value | — |
| Engines | Evasion (dodge chance) | Distance traveled per day |
| Sensors | Crit chance | Detection range for encounters/derelicts |

## Ship Stat Formula

Each stat is computed independently through the same pipeline:

```
Final stat = base_stat × room_multiplier × crew_multiplier × component_multiplier
```

- **base_stat**: From ship level (gained via combat XP). Starts at 10, grows per level.
- **room_multiplier**: `1.0 + sum of all active room contributions for that stat`. A level 5 Weapons Bay adds +0.50 to the Firepower room multiplier.
- **crew_multiplier**: `1.0 + crew bonus`. Based on specialty match and skill level of crew assigned to relevant rooms.
- **component_multiplier**: `1.0 + sum of stat component bonuses` from component slots in relevant rooms.

Example: Base Firepower 30 × Room 1.5 × Crew 1.3 × Components 1.2 = 70 Firepower.

## Combat Pipelines

### Attack Pipeline (ship → enemy)

```
1. Final Firepower (base × room × crew × component)
2. + Rune Array flat bonus (if built; converts Transmissions to flat damage)
3. - Enemy defense
4. Min 1
5. Crit check: chance = 5% + (Sensors / (Sensors + enemy_stealth)) × 25%
   Cap: 30%. Crit multiplier: 2.0x (components can increase)
6. Apply damage to enemy HP
```

### Defense Pipeline (enemy → ship)

```
1. Enemy base damage
2. - Hull defense (Final Hull stat used as defense value)
3. Min 1
4. Evasion check: dodge_chance = Engines / (Engines + enemy_accuracy)
   Cap: 50%. Dodge negates entire attack.
5. Apply damage to current Hull HP
```

### Key Differences from Act 1

- **No prestige/ascension multiplier** — clean break, ship earns all power
- **No hull regen between fights** — damage persists, must be repaired via Workshop/events
- **Evasion replaces damage reduction** — Engines-based dodge instead of flat DR
- **Sensors drive crits** — gives exploration stat a combat role
- **Rune Array is optional** — old-world Transmissions only become combat power if you invest a room slot

## Ship Leveling

The ship gains XP from combat encounters (like the hero). XP curve: `500 × level^1.3` per level. No max level cap — diminishing returns serve as the natural ceiling.

Per level, base stats increase:
- Firepower: +2
- Hull: +3 (HP scales slightly faster)
- Engines: +1 (speed is powerful, grows slowly)
- Sensors: +1

## Speed Formula (Engines → ly/day)

Speed is derived from the **final** Engines stat (after all four layers):

```
speed_ly_per_day = Engines² / 1,000     (capped at 100 ly/day)
```

| Engines (final) | Speed |
|-----------------|-------|
| 10 (launch) | 0.1 ly/day |
| 25 | 0.63 ly/day |
| 40 | 1.6 ly/day |
| 80 | 6.4 ly/day |
| 150 | 22.5 ly/day |
| 250 | 62.5 ly/day |
| 316+ | 100 ly/day (cap) |

The mapping is quadratic on purpose: the four stat layers each grow roughly linearly, and squaring turns steady investment into the exponential speed ramp the ~8-month duration target requires (see parent spec, Distance and Progression). The cap keeps the final approach from trivializing and bounds encounter frequency.

Sanity check on reachability: late-voyage base Engines ≈ 10 + ship level (~70-80), × room 2.0 (Engines room Lv 10) × crew ~1.5 (primary specialist Lv 10) × components ~1.3 ≈ 270-310 — the plateau is reachable but requires investment in all four layers.

## Reactor Power Budget

The Reactor is a room that produces power points. All other rooms consume power.

### Power Production

Reactor at level N produces: `10 + (N-1) × 5` power points.

| Reactor Level | Power Produced |
|---------------|----------------|
| 1 | 10 |
| 3 | 20 |
| 5 | 30 |
| 7 | 40 |
| 10 | 55 |

### Power Consumption

Each room has a base power cost by category that increases with room level.

**Base costs by category:**

| Category | Rooms | Base Cost |
|----------|-------|-----------|
| Core | Reactor, Hull Plating, Engines, Weapons Bay | 5 |
| Survival | Fuel Refinery, Cargo Hold, Life Support | 3 |
| Exploration | Sensors Array, Shuttle Bay, Cartography Deck | 4 |
| Crew/Narrative | Quarters, Medbay, Shrine | 2 |
| Production | Forge, Garden, Workshop | 3 |
| Special | Rune Array, Void Lens, Fate Loom | 6 |

**Level scaling:** Room power cost = `base_cost + (level - 1)`. A level 5 Weapons Bay costs `5 + 4 = 9` power.

Note: The Reactor itself costs 0 power (it produces, doesn't consume).

### Over-Budget Behavior

- Rooms can be **toggled on/off**. Inactive rooms cost 0 power and contribute nothing.
- If total active room cost exceeds Reactor output, the ship is **over-budget**. Over-budget rooms run at 50% effectiveness (stat contributions halved). All active rooms share the penalty equally — there's no priority system.
- The UI clearly shows power used/available and highlights rooms in red when over budget.

## Room System

### Slot Unlocks

The ship launches with **4 slots** (2 filled — Reactor, Engines — plus 2 empty, so the starting salvage has somewhere to go). 16 more unlock at geometrically spaced distances (~×1.6 apart). Under the exponential speed curve this means **a new slot roughly every 1-2 weeks of wall-clock** across the whole voyage:

| Slot # | Distance | Slot # | Distance |
|--------|----------|--------|----------|
| 1-4 | 0 ly (launch) | 13 | 400 ly |
| 5 | 10 ly | 14 | 640 ly |
| 6 | 16 ly | 15 | 1,000 ly |
| 7 | 25 ly | 16 | 1,600 ly |
| 8 | 40 ly | 17 | 2,500 ly |
| 9 | 64 ly | 18 | 4,000 ly |
| 10 | 100 ly | 19 | 6,400 ly |
| 11 | 160 ly | 20 | 9,000 ly |
| 12 | 250 ly | | |

### Room Type Unlocks

Room types unlock at distance milestones (aligned with slot-ladder rungs). Not all types available from the start.

| Distance | Room Types Unlocked |
|----------|-------------------|
| 0 ly | Reactor, Engines, Hull Plating, Weapons Bay, Cargo Hold, Quarters |
| 40 ly | Fuel Refinery, Life Support, Sensors Array |
| 250 ly | Workshop, Garden, Medbay |
| 1,000 ly | Forge, Shuttle Bay, Shrine |
| 2,500 ly | Cartography Deck, Rune Array |
| 6,400 ly | Void Lens, Fate Loom |

### Building a Room

- Select an empty slot
- Choose from unlocked room types
- Pay a salvage cost (scales with room category: Core 100, Survival 60, Exploration 80, Crew 40, Production 60, Special 150)
- Room is built instantly (no construction delay — the Loom already taught patience)

### Rebuilding

Demolishing a room frees the slot but costs 50% of the original build cost in salvage (demolition fee). Components in the room are returned to inventory. The slot becomes empty and can be rebuilt.

### Room Upgrades: Levels

Rooms level from 1 to 10. Each level costs salvage scaling with current level and room category:

```
upgrade_cost = base_build_cost × level × 1.5
```

Each level increases the room's stat multiplier contribution by a fixed amount per room type.

### Room Upgrades: Components

Each room has 2-3 component slots (depending on category):
- Core/Special rooms: 3 slots
- All others: 2 slots

Components are found from combat loot, derelict exploration, and decision events. They modify the room:
- **Stat components** — flat bonus to a specific stat multiplier (e.g. "+0.1 Firepower multiplier")
- **Efficiency components** — reduce power consumption (e.g. "-1 power cost")
- **Special components** — unique effects (e.g. "Weapons Bay fires twice per encounter", "Garden produces fuel as well as supplies")

Components can be swapped freely (no cost to remove/replace). They're an inventory you manage.

## Room Stat Contributions

Each room contributes a multiplier bonus to one or more stats. Base contribution at level 1, scaling with level.

**Per-level multiplier bonus (added to room's contribution per level):**

| Room | Primary Stat | Bonus/Level | Secondary |
|------|-------------|-------------|-----------|
| Weapons Bay | Firepower | +0.10 | — |
| Hull Plating | Hull | +0.10 | — |
| Engines | Engines | +0.10 | — |
| Sensors Array | Sensors | +0.10 | — |
| Fuel Refinery | — | — | Converts void matter → fuel: 5 VM/day throughput per level at 2 VM : 1 fuel (see mode-transition spec, Void Matter Harvesting) |
| Cargo Hold | — | — | +10% resource capacity/level |
| Life Support | — | — | +1 crew capacity/level (ship base capacity is 2 with none built) |
| Quarters | — | — | +5% crew effectiveness/level |
| Medbay | — | — | Crew injury recovery speed +10%/level |
| Shrine | All stats | +0.02 | — |
| Forge | — | — | Component crafting (future) |
| Garden | — | — | +5 supplies/day per level |
| Workshop | Hull | +0.03 | +2 hull repair/day per level |
| Shuttle Bay | Sensors | +0.05 | Enables boarding events |
| Cartography Deck | Sensors | +0.05 | Reveals upcoming encounters |
| Rune Array | — | — | +10% transmission efficiency/level |
| Void Lens | Sensors | +0.08 | Unlocks hidden encounters |
| Fate Loom | All stats | +0.03 | Weave minor ship patterns |

A level 5 Weapons Bay contributes: `5 × 0.10 = 0.50` to the Firepower room multiplier. Combined room multiplier for Firepower = `1.0 + sum of all Firepower contributions`.

## Starting State (at launch)

The Vessel starts with:
- 4 room slots: **Reactor** (Lv 1) and **Engines** (Lv 1) built, 2 empty
- Ship level 1 (base stats: Firepower 10, Hull 10, Engines 10, Sensors 10 → speed 0.1 ly/day)
- 0 crew
- Some starting salvage (enough for 2-3 room builds)
- Room types available: Reactor, Engines, Hull Plating, Weapons Bay, Cargo Hold, Quarters

## UI: Room Management

Accessed via `[R]` hotkey from the voyage screen. Shows a grid/list of all slots:

```
┌─ Rooms (3/20 slots) ──── Power: 14/15 ─────────────┐
│                                                      │
│  1. [Reactor]      Lv 3   ⚡ produces 20            │
│  2. [Engines]      Lv 2   ⚡ 6   Eng +0.20          │
│  3. [Weapons Bay]  Lv 1   ⚡ 5   Fpr +0.10  ●●○     │
│  4. (empty)                                          │
│  5. (locked — 1,500 ly)                              │
│  ...                                                 │
│                                                      │
│  Selected: Weapons Bay                               │
│  Firepower: +0.10 multiplier                         │
│  Power cost: 5                                       │
│  Components: [Empty] [Empty] [Empty]                 │
│  Upgrade to Lv 2: 150 Salvage                        │
│                                                      │
│  [U] Upgrade  [T] Toggle  [D] Demolish  [Esc] Back  │
└──────────────────────────────────────────────────────┘
```

## Files

| File | Change |
|------|--------|
| `src/vessel/rooms.rs` | New: room types, stats, costs, power budget, build/demolish/upgrade |
| `src/vessel/components.rs` | New: component types, slot management, inventory |
| `src/vessel/stats.rs` | New: ship stat derivation (base × room × crew) |
| `src/vessel/types.rs` | Modify: add Room, Component, RoomSlot to VesselState |
| `src/ui/vessel_rooms_scene.rs` | New: room management overlay rendering |
| `src/input/vessel_input.rs` | Modify: add [R] hotkey and room management input |

## Testing

- Unit test: stat formula (base × room × crew × component) produces correct values
- Unit test: speed formula anchors (Engines 10 → 0.1 ly/day, 100 ly/day cap at 316+)
- Unit test: Reactor power production at each level
- Unit test: room power cost scales with base + level
- Unit test: over-budget penalty applies 50% to all rooms
- Unit test: room type unlock gating by distance
- Unit test: room slot unlock gating by distance
- Unit test: build cost and demolish refund calculations
- Unit test: upgrade cost formula
- Unit test: room stat contribution scales with level
- Unit test: component slot limits by room category
- Unit test: toggling rooms on/off affects power budget

## 2026-07-02-act2-voyage-experience-exploration.md

# Act 2 Voyage Experience — Design Exploration

**Status:** Exploration — precedes rewriting sub-projects 2–7
**Question:** The player is taking a journey to the living branch of Yggdrasil.
What should that journey *feel* like — and is the currently-specced design the
right one?

Only sub-project 1 (the launch gate) has shipped, and it is compatible with
every direction in this document: the burn happens, then Act 2 begins. Nothing
downstream is built. This is the moment to choose the experience.

---

## 1. What kind of fun does Act 1 already own?

Before designing the voyage, inventory the fun the player has already had for
hundreds of hours — because Act 2 should not re-serve it with new nouns.

| Kind of fun | Act 1 delivery | Strength |
|-------------|----------------|----------|
| **Growth** (number-go-up) | XP, prestige, ascension, item power | Dominant |
| **Engine-building** (optimize a machine) | Loom production chains, Haven tree, power/sigil loadouts | Dominant |
| **Collection** (loot, tiers) | Item drops T0–T9, god items, achievements | Strong |
| **Active skill** (challenge) | 14 minigames | Strong |
| **Discovery cadence** (new system unlocks) | Haven → Soulforge → Deep → Loom | Strong, front-loaded |
| **Light management** (send/wait/resolve) | Deep expeditions | Moderate |

And what Act 1 has **never** delivered:

| Absent fun | Why it's absent |
|------------|-----------------|
| **Meaningful choice** | No decision in Act 1 closes a door; every path is eventually walkable |
| **Consequence / irreversibility** | Everything loops — zones cycle, prestige resets, death costs seconds. The 250k burn is the *first* irreversible act in the game |
| **People / fellowship / loss** | Deep mercs are stat blocks; nobody has a name you remember |
| **Place / journey** | Act 1 happens *in place*. Zones are difficulty bands, not geography; you end where you started, stronger |
| **Scarcity** | The economy is faucet-only; nothing is ever truly spent except the burn |
| **Mystery** | Every system is legible, wiki-able, optimizable on sight |

**The core observation:** the currently-specced Act 2 (rooms + ship stats +
auto-combat + resource drains) rebuilds Act 1's dominant fun with new nouns —
the ship is the hero (XP, levels, stats), rooms are the Haven/Loom (build,
upgrade, power budget), distance is the zone counter, salvage is XP/loot. The
genuinely *new* fun in the spec set — crew you can lose, decisions with
consequences, one-way travel — lives in the parts that are subordinated
(crew as stat multipliers) or not yet designed (specs 6–7). The skeleton got
specced first; the soul got deferred.

**Design goal for the voyage:** flip the hierarchy. The journey's primary fun
should come from the left column Act 1 never touched — place, people, choice,
consequence, mystery — with a light growth/management spine underneath because
this is still an idle game and the idle rhythm must survive.

One more Act 1 fact worth honoring: the **check-in psychology**. Act 1 players
open the game, see what accumulated, make 2–3 decisions, close it. The voyage
should make each check-in *an event with a face on it*, not a status glance.

---

## 2. Four parallel directions

### Direction A — The Shipwright (current spec set, baseline)

*You grow a ship the way you grew a hero.*

Distance accrues continuously; encounters auto-resolve; salvage buys room
levels; crew multiply room outputs; drift punishes neglect. Specs 2–5 as
written.

- **Fun profile:** growth, engine-building, collection — Act 1's exact profile.
- **Strengths:** proven idle loop; lowest design risk; specs already written.
- **The honest critique:** at month 8 of Act 1-flavored play, the player
  boards a ship and finds… Act 1 with a hull. The Reactor power budget *is*
  the Loom. Distance *is* the zone number. The one-way journey — the whole
  premise — is reduced to a progress bar that only moves right.
- **Keep from A regardless of direction:** the three resource gauges
  (fuel/hull/supplies) as ambient tension, drift as the failure state, and
  the offline rules (travel continues, people are safe).

### Direction B — The Pilgrimage (place-driven)

*The void is not 10,000 light-years of nothing. It is a string of named places
no one has ever come back from.*

- **Structure:** replace the continuous-distance model with a **route map** of
  ~24 named waypoints on a branching constellation chart — dead branches,
  drowned worlds, a lighthouse tended by something old, the wreck-field of
  everyone who tried this before. The map is the main screen; your vessel is a
  dot crawling along an edge; **the tree renders on the horizon and visibly
  grows** at every waypoint — the progress bar is a living illustration.
- **The idle unit is the LEG:** waypoint to waypoint takes 1–3 real days, each
  leg flavored (a current that speeds you, a debris field that eats hull, a
  silence that unsettles the crew). Ambient encounters and whispers keep the
  ticker alive mid-leg. The game tells you: *"Arriving at the Drowned Choir
  in ~14h."* — check-ins become appointments.
- **The active unit is the ARRIVAL:** each waypoint is a 5–15 minute vignette —
  narrative scene, a real choice, a trade/refit screen, sometimes a soul to
  take aboard, sometimes a hazard to run. This is where sub-project 6's
  "decision events" live — not as random popups on a timer, but as *places*.
- **Choice = route:** at each junction, 2–3 onward legs with visible
  trade-offs (long/safe, short/hungry, unknown/rumored). **The branch you skip
  stays unseen forever** — the first time the game has ever let content go.
  Rumors gathered at waypoints change what you know about upcoming branches,
  so information is a resource.
- **Supply line reframe (spec 7):** old-world transmissions stop being a
  trickle and become a **care package waiting at each waypoint** — contents
  scale with what your Loom produced during the leg. Letters from home,
  literally: event-shaped, not faucet-shaped.
- **Fun profile:** anticipation, choice-consequence, discovery of *places*,
  narrative. Idle-fit is excellent (legs = away time, arrivals = sessions).
- **Costs/risks:** content authoring (~24 waypoint vignettes + branch variants)
  is the real price. Mitigation: waypoints share a small set of mechanical
  templates (trade / hazard / recruit / mystery / rest) under unique writing.

### Direction C — The Crossing (people-driven)

*The Vessel is not a machine you upgrade. It is the last boat out, and it is
full of people you already know.*

- **Structure:** the ship carries **souls** — 8–12 named survivors of the dying
  branch, seeded from the systems the player has lived with for months: the
  scarred mercenary captain who found you (the Deep), the Haven warden, the
  fisher who taught you the leviathan's rhythm, the keeper of the challenge
  halls. Their manifest IS the game state. Each soul has a need, a bond, and
  an arc that unfolds across the voyage.
- **The central meter is HOPE**, not fuel — a caravan-morale gauge moved by
  choices, losses, and small victories. Low hope makes everything worse; high
  hope is the wind at your back. (Banner Saga's caravan, in idle form.)
- **Events are interpersonal:** two souls feud over rations; a confession at
  the halfway mark; someone falls ill and the medbay question is *who sits
  with them*; a wedding under alien stars. Choices trade speed vs supplies vs
  people.
- **Loss is permanent and memorialized:** a death carves a name into the hull —
  visible in the ship art for the rest of the game. The arrival scene at the
  tree enumerates *who made it*. That list is the emotional payoff of the
  entire act.
- **Stations, not multipliers:** souls crew stations (helm, tender, watch);
  their arcs — not their stat lines — change how stations perform. The ship
  layer shrinks to three gauges because the people are the systems.
- **Fun profile:** fellowship, drama, expression, loss — the entire column
  Act 1 never touched. Highest emotional ceiling of any direction.
- **Costs/risks:** heaviest writing load per soul; permadeath must respect the
  established offline rule (nothing irreversible happens to people while
  you're away — already designed in sub-project 2's offline table).

### Direction D — Deadreckoning (mystery-driven)

*Nobody has a map. The beacon is a bearing, not a route.*

- **Structure:** the chart starts as **fog**. You set headings; the vessel
  sails them; what you find gets inked onto your chart permanently. Progress
  is measured in *what you know*: currents discovered, hazards charted,
  shortcut runes decoded. (Outer Wilds' core trick — the player's knowledge,
  not the character's stats, is the thing that grows — has never been done in
  an idle game.)
- **The route-rune meta-puzzle:** fragments of an old wayfinder's rune are
  scattered in wrecks and hermitages. Each decoded fragment (a small active
  puzzle, kin to the challenge minigames but diegetic) reveals a piece of the
  true route. The tree is reachable without them — but weeks slower.
- **Wrong headings aren't failure:** they find things — wrecks with charts,
  a hermit with rumors, a current that flings you sideways into unexplored
  fog. Time is the only cost, and this is an idle game: time is the one
  currency the player has infinite patience for.
- **Auto-navigator fallback:** players who never engage get a slow, steady
  bearing-to-beacon crawl (the established auto-resolve-safe pattern), so the
  idle contract survives.
- **Fun profile:** mystery, mastery, discovery-as-progress. The most novel;
  the only direction where a wiki can't spoil the feeling (knowing the map
  exists ≠ your chart being drawn).
- **Costs/risks:** hardest to build well; navigation UIs in a TUI need real
  care; the fog must hide *content*, not just geometry, or it's a slow bar
  with extra steps.

---

## 3. Comparison

| | A — Shipwright | B — Pilgrimage | C — Crossing | D — Deadreckoning |
|---|---|---|---|---|
| Primary fun | growth, engine | anticipation, choice, place | fellowship, loss | mystery, mastery |
| Overlap with Act 1 | **near-total** | low | none | none |
| Idle-rhythm fit | excellent | **excellent** (legs/arrivals) | good | good (needs fallback) |
| Check-in feel | status glance | **appointment with a place** | "how are my people" | "what did we find" |
| Authoring cost | low (numbers) | high (vignettes) | highest (character arcs) | medium (systems > words) |
| Systems cost | low (specced) | medium | medium | high |
| Emotional ceiling | low | medium | **highest** | medium |
| Wiki-proof | no | partly | partly | **yes** |
| Honors "journey to the tree" | weakly (a bar) | **literally (a map + horizon)** | via who survives it | via the charting of it |

---

## 4. Recommendation — The Pilgrimage of Souls

No single direction is the answer; the strongest voyage is a deliberate blend
with a clear hierarchy:

1. **Spine: Direction B.** The route map of named waypoints replaces the
   continuous-distance model. Legs are the idle unit; arrivals are the active
   unit; junctions are the strategy; the tree grows on the horizon. This is
   the smallest structural change that makes the journey *feel* like a journey,
   and it gives specs 6 (events → arrival vignettes) and 7 (supply line →
   care packages) natural, better-shaped homes.

2. **Heart: Direction C, scaled to fit.** Six to eight souls, not twelve —
   each one a face from an Act 1 system, each with one arc, one bond, and one
   station. Hope as a visible gauge beside fuel/hull/supplies. Permanent,
   memorialized loss. The crew spec is rewritten from "stat multipliers with
   reveal timers" to "people whose stories modify the ship" — the multiplier
   math can stay underneath, but it is never the headline.

3. **Garnish: one Deadreckoning mechanic, not the whole system.** The route
   map reveals only one junction ahead; rumors bought/earned at waypoints
   illuminate further. Optionally, 3–5 route-rune fragments hide along the way
   and decode into shortcuts. Fog of *route*, not fog of everything.

4. **Demote, don't delete, Direction A.** Three gauges + drift stay as ambient
   tension. Rooms shrink from a 20-slot Reactor-budget engine to **refits** —
   a handful of meaningful hull choices made at waypoints (you can't hold both
   the Void Lens and the Long Hold; pick). Auto-combat stays as leg ambience
   with occasional named threats at waypoints, not a second combat game.
   Ship XP/levels are cut entirely — growth lives in Act 1; the voyage's
   progression is *distance made, souls kept, route known*.

### What this means for the spec queue

| Existing spec | Fate under the recommendation |
|---|---|
| 1. Launch gate (shipped) | Unchanged — compatible as-is |
| 2. Mode transition & shell | Rewrite the shell: route map main screen, leg/arrival loop, tree-on-horizon progress; keep the 5-beat transition, gauges, drift, offline rules |
| 3. Rooms & stats | Collapse into **Refits** (waypoint choices); cut ship XP/levels, Reactor budget |
| 4. Auto-combat | Shrink to leg ambience + named waypoint threats; cut the enemy-tier ladder |
| 5. Crew | Rewrite as **Souls** — arcs, bonds, hope, memorial; keep capacity/injury/offline-protection bones |
| 6. Decision events (unwritten) | Becomes **Arrival Vignettes + Junctions** — the heart of the act |
| 7. Supply line (unwritten) | Becomes **Letters From Home** — care packages at waypoints |

### The next spec to write

**"The Route & the Waypoints"** — the new sub-project 2: the constellation
map (structure, junction count, branch shapes), the leg/arrival state machine,
waypoint mechanical templates (trade / hazard / recruit / mystery / rest),
rumor/information rules, and the tree-on-horizon progression render. It is the
load-bearing spec every other rewrite hangs from — exactly the role the
distance model played in the old set, but shaped around the fun Act 1 never had.

Open questions to settle in that spec: total voyage duration target (the
~8-month target likely survives; ~24 waypoints × ~1.5-day legs + branches
lands near it); how junction content that goes unseen affects the completionist
itch (proposed answer: the chart records the names of roads not taken — a
scar, not a checklist); and whether arrivals can queue offline (proposed:
the vessel holds station at the waypoint until you arrive — the place waits,
the appointment keeps).

## 2026-07-03-vessel-arrival-design.md

# The Arrival

**Parent spec:** [2026-03-27-the-vessel-design.md](2026-03-27-the-vessel-design.md)
**Sub-project:** 7 of 7 — the last spec of the act.
**Depends on:** specs 2–6 (all shipped). Everything this spec renders
already exists in the save: `visited`, `untaken`, `keepsakes`, the Log,
the letters kept, the carved names, the soul roster with its resolved
arcs, and W37's authored three-beat scene behind `take_finale_playback()`.
**Feeds:** Act 3 (a single flag and a single face; nothing more).

## Overview

The crossing ends. This spec is about *how it ends being worth the
20–200 days it took*: the final approach staged as the act's longest
scene, and then — instead of a victory screen — three quiet rooms the
player can sit in for as long as they like. **The manifest** (who came,
who joined, who was lost and where their name is carved). **The keepsake
chart** (the whole crossing, the roads not taken crossed out forever).
**The record** (the Log, complete, with the letters bound in).

The design law that has governed every spec still governs the last one:
**memory, never score** (No Right Path, rule 6). There is no grade, no
percentage, no "souls saved: 6/8", no comparison against a route you did
not sail. The manifest is a ship's document, not a report card. The
numbers that appear (days at sea, letters kept) are facts a captain would
know, not metrics a game is judging.

And the act keeps its other promise to the very end: **what an untaken
road held is never revealed** (rule 3). The keepsake chart shows the shape
of the roads you crossed out, not their contents. The fog over unvisited
waypoints never lifts. You will wonder. That is the point.

## The Final Approach

Arrival at W37 currently fires the three authored beats ("The Tree." /
the sight of it / lines thrown, hands catching). Spec 7 extends the
finale playback — one staged sequence, read at the player's pace like any
scene, built at `take_finale_playback()` time from the save:

1. **The Tree** — W37's existing three beats, unchanged.
2. **The rail** — one authored line per soul *aboard*, in boarding order,
   each in their own voice, each colored by their resolved arc (a soul
   whose resolution fired at the sink still counts as resolved — the
   engine already guarantees no arc is left dangling). Souls who stepped
   ashore earlier get no rail beat; their goodbye already happened and is
   in the Log.
3. **The carved names** — if any soul was lost, one beat: the crew
   gathers at the rail where the names are carved, and reads them within
   sight of the Tree. Skipped entirely on a lossless crossing (absence of
   grief is not announced; it is simply a shorter scene).
4. **The harbor** — the Sister Verity is moored in the root-harbor
   (already authored: `dark_after: None`, "a face for Act 3"). Her
   mate waves the Vessel in. One beat, and she is the only outward hook
   this act plants.
5. **The lamp** — the closing beat. A door in the root-wall, closed, with
   a lamp lit beside it. Nobody says "Act 3". The beat says the harbor
   has rooms, the rooms have doors, and one of them is yours, later.

The whole sequence is still one `ScenePlayback` through the existing
pager — no new UI machinery. `finale_shown` still latches it to exactly
one showing; the harbor screen (below) is what you return to forever.

## The Harbor (the Arrived state's home screen)

After the finale, the voyage screen's Arrived state stops being a stub
and becomes a small permanent place: the chart panel shows the Vessel
moored at the Tree (sea calm, no weather objects — `weather_at` simply
isn't consulted), and the side panel lists what there is to sit with:

| Key | Room |
|-----|------|
| `[M]` | The manifest |
| `[K]` | The keepsake chart |
| `[R]` | The record (the Log, complete) |
| `[Q]` | Back to the title flow, as today |

Nothing ticks. Provisions and hope gauges are retired from the panel
(the crossing is over; the gauges were the crossing). Time-at-the-Tree
is not measured. This screen is deliberately a museum, not a lobby.

> **Doc-alignment note (2026-07-04):** "hope" above described a
> per-arrival retirement; as of commit d39ad67, Hope is retired from the
> game entirely, not just from this panel — there is no hope gauge left to
> retire anywhere in the Voyage. The framing above is now redundant with
> the broader retirement rather than wrong.

## The Manifest

One scrollable panel, a ship's document in four parts:

1. **The crossing** — vessel name, the launch date and arrival date (game
   dates), days at sea (from `at_min`), the chapters crossed. Facts only.
2. **The souls** — every soul *met*, grouped by how their story ended:
   - **Came ashore** (aboard at arrival): name, station history's last
     post, and their arc's resolution beat *title* (the Log has the text).
   - **Went their own way** (ashore/farewell): name, the waypoint where
     they stepped off.
   - **Carved** (lost): name, and the line already used at the memorial.
   Souls never met are **not listed** — the manifest records the crossing
   that happened, not the cast that exists (rule 3 again, applied to
   people).
3. **The hold** — keepsakes, in the order collected, each with the
   waypoint it came from; then "Letters kept: N" and the senders (the
   letters themselves remain readable in the record).
4. **The wake** — rumors heard, refits taken (and, for each refit taken,
   the name of the door that closed — the one place the manifest admits a
   road not taken, because the player chose it looking at both).

No totals, no ranks, no stars.

## The Keepsake Chart

`[K]` opens the chart full-screen — the same renderer, three changes:

- **Pan** with arrow keys (the full canvas is bigger than any terminal;
  until now the viewport followed the Vessel; now it follows the player).
- **The sailed route** renders bright: visited waypoints ◉, roads sailed
  solid, in visit order discernible by the line style already used.
- **Untaken roads** keep their `✕` forever, and unvisited waypoints keep
  their fog glyph and their namelessness. The chart never becomes a map
  of the world; it stays a map of *your crossing* through it.

No new data: `visited` and `untaken` are already the full record.

## The Act 3 Gate

One new persisted fact and nothing else:

```rust
// GameState (serde default: false — every existing save loads clean)
pub vessel_arrived: bool,
```

Set exactly once, when the finale playback is taken (same latch as
`finale_shown`, but on the account-visible side: `main.rs` sets it and
saves when it surfaces the finale). Act 3, whenever it is designed, keys
off `vessel_arrived` the way Act 2 keyed off `vessel_launched` — and the
Sister Verity in the harbor is its authored face. This spec deliberately
plants *no other* Act 3 machinery: no currencies, no unlocks, no teaser
menu. The closed door with the lamp is a sentence in a scene, not a
locked UI element (locked UI is a promise with a countdown; a lit lamp is
a promise without one).

The kill-switch discipline is unchanged: all of this ships dark behind
`ACT2_ENABLED = false`, and enabling the act remains the deliberate
two-line PR it has been since spec 1.

## Data Model (build scope)

```rust
// GameState — the only cross-act surface
vessel_arrived: bool,                    // serde(default)

// src/vessel/souls.rs — authored additions
pub struct SoulDef {
    // ...existing...
    /// One line at the rail, in their voice, read at the finale
    /// if they are aboard at arrival.
    pub rail_line: &'static str,
}

// src/vessel/voyage.rs
// take_finale_playback() grows from "W37's scene" to the staged
// sequence above (rail beats + carved names + harbor + lamp appended
// to the authored beats). Pure function of VoyageState — offline ==
// live, chunking-invariant, same as everything else.

// Manifest/chart/record are pure render-side reads of VoyageState —
// no new serialized voyage fields at all.
```

UI: `VoyageView` grows `Manifest` and a pannable chart offset for the
Arrived state; input routes `[M]`/`[K]`/`[R]` only when
`voyage.arrived()`. The record reuses the existing Log panel rendering
with the pager.

## What This Spec Does NOT Add

No score, grade, rank, or completion percentage. No reveal of untaken
roads or unmet souls. No new-game-plus, no second crossing, no "sail
again" prompt. No Act 3 content beyond one flag and one moored ship. No
post-arrival economy — nothing at the Tree costs or pays anything. No
changes to Act 1 (still frozen fiction; whether its numbers ever render
again is Act 3's question, unanswered here on purpose).

## Testing

- The finale sequence is a pure function of the save: same VoyageState →
  identical beats (snapshot the playback for a fixture crossing with a
  loss, a farewell, and a full hold; and for a minimal lossless one).
- Rail beats appear for exactly the souls aboard at arrival, in boarding
  order; the carved-names beat appears iff `carved_names()` is non-empty.
- `vessel_arrived` is set exactly once, persists, and old saves load
  with it false (save-compat corpus).
- Manifest lists only met souls; groups match roster statuses; keepsakes
  and letters match the hold; no numeric field beyond days and counts.
- Keepsake chart: pan clamps to canvas; fog glyphs and `✕` render for a
  fixture with untaken junctions; no unvisited waypoint name appears
  anywhere in the buffer (grep the frame — this is rule 3 as a test).
- Overlay snapshots for harbor screen, manifest, and keepsake chart at
  XL and S tiers; input tests for [M]/[K]/[R] gating on `arrived()`.
- The full-crossing simulator still lands inside the 20–200 day envelope
  and `finale_shown`/`vessel_arrived` latch once across save/load.

## Open Questions

- Whether the record `[R]` should paginate the letters as full re-readable
  text or keep the Log's one-line "kept" entries (lean: full text — the
  letters are the act's best writing and the Going-Dark made them finite).
- Whether the harbor screen should show the Sister Verity's lamp on the
  chart as a persistent glyph (lean: yes, one ☀ at the sink — cheap, and
  it keeps the Act 3 face visible without a single word of UI).
- Whether arrival should be announced to the Act 1 title screen (e.g. the
  character select shows "— arrived —" instead of a zone). Lean: yes but
  trivial; decide at build time.

## 2026-07-03-vessel-arrival-scenes-design.md

# Arrival Scenes — Delivery, Payoffs, Refits, and the Threats

**Parent spec:** [2026-03-27-the-vessel-design.md](2026-03-27-the-vessel-design.md)
**Sub-project:** 4 of 7
**Depends on:** spec 2 (shipped — waypoints carry `SceneRef` slots, drift
carries a recovery-scene flag), spec 3 (shipped — boarding asks, farewells,
`mark_lost`, counsel). **Feeds:** spec 6 (the Going-Dark is a scene), spec 7
(keepsakes and resolutions read into the manifest).

## Overview

Arrivals are **payoff scenes, never tests**. The player already made the
choice — at the junction, with the price on the card. The arrival is where
that choice pays: in provisions, in people, in one more piece of the world
seen before it ends. This spec defines the scene system that delivers them:
the beat format, the recoloring model, the real waypoint economy (replacing
the interim +20 gift), refits at shipyards, and the three named threats —
the only places a soul can be lost, and never by dice.

The one structural rule, inherited and absolute: **scenes contain no option
menus.** The only interactive moments inside scenes are *doors* the design
already sanctions — a soul's ask (board / decline), a farewell, and a refit
(a permanent A/B). Nothing in a scene is ever a gamble, a skill check, or a
timed prompt.

## The Scene Format

A scene is an authored sequence of **1–4 beats** — short paragraphs shown
one at a time in the scene view (`[Enter]` advances; the last beat shows
the payout in small print). Beats are text plus optional **color lines**
selected by state. Structure:

```rust
pub struct SceneDef {
    id: &'static str,               // matches route.rs SceneRef ids
    beats: &'static [SceneBeat],
    payout: ScenePayout,            // applied when the scene completes
    door: Option<SceneDoor>,        // Ask(soul) | Refit(pair) — at most one
}

pub struct SceneBeat {
    text: &'static str,
    colors: &'static [ColorLine],   // 0..n; matching lines append, in order
}

pub struct ColorLine {
    when: ColorKey,                 // deterministic, state-derived
    text: &'static str,
}

pub enum ColorKey {
    SoulAboard(SoulId),             // "Sefa steps forward and sings."
    ArrivedBy(RoadId),              // the road colors the landfall
    TrimIs(Trim),                   // arriving at Mourn reads differently
    KnowsRumor(RumorId),            // foreknowledge acknowledged
    ChapterIs(Chapter),
    HopeAtLeast(u8), HopeBelow(u8),
    Drifted,                        // the leg included a drift
}
```

> **Doc-alignment note (2026-07-04):** `HopeAtLeast`/`HopeBelow` never
> shipped — Hope was retired entirely (commit d39ad67) before this variant
> would have been used. Current `ColorKey` (`scenes.rs:18-23`) is
> `SoulAboard/ArrivedBy/TrimIs/KnowsRumor/Drifted` — no chapter or hope
> variants. Likewise every `hope: i8` payout field and every `hope ±N`
> beat described below (the payout table, the Ossuary Warden, the Silence
> itself, `mark_lost`) no longer prices anything — those beats are either
> free now or (Silence-bank specifically) replaced by a strain hit on the
> Helm soul, per the underway spec's alignment note. One refit is also now
> a no-op: **Lantern Mast** (row 3 below) has a name and blurb (`refits.rs`)
> but nothing reads it — unlike its siblings, it isn't wired into any
> gameplay effect. And **Mourning Colors**' stated "×0.80 provisions" was
> written against Mourn's old 0.90 base; Mourn's shipped base is already
> 0.80 post-Hope-retirement (`voyage.rs`), with the refit stacking it
> further to 0.70 — the number below no longer matches either value.

Rules that keep this writable and honest:

- **Determinism**: color lines are pure state functions — same save, same
  scene, always. No RNG anywhere in scenes.
- **Budget**: base beats ~40–80 words; color lines one sentence each. A
  waypoint's full scene tops out around a screen of text. 38 waypoints ×
  (2–3 beats + 3–6 color lines) is the writing cost — the route content
  map worksheet (below) tracks it.
- **Scenes play once.** Waypoints are visited once per crossing; the scene
  state machine (spec 2's `Waiting → Played`) already enforces it.
- The existing one-line `placeholder` in `SceneRef` remains the fallback
  for any scene not yet authored — the game never blocks on writing.

## The Real Economy (retiring the +20 gift)

The interim uniform gift is replaced by **authored payouts** in the scene
data. Provisioning becomes part of the fiction: a market has stock, a reef
does not. Amounts by waypoint kind (authoring guideline, tuned by the
simulator's no-drift-at-Cruise target):

| Waypoint kind | Provisions payout | Notes |
|---------------|-------------------|-------|
| Harbor (start/end) | fill to cap | the pier gives what it has |
| WayStation | +25 | markets; rumors still cost 6 |
| RestStop | +20 | and breaks the Long Silence (shipped) |
| Shipyard | +15 | yards feed crews; refit door lives here |
| LanternSite | +12 | keepers' stores, shared |
| plain waypoint | +8..+15 authored | the scene says why |
| threat waypoints | +0..+5 | the reef gives nothing freely |

Other payout fields: `hope: i8` (a few scenes move it ±1, authored
sparingly), `rumor: Option<RumorId>` (scene-granted knowledge), and
`keepsake: Option<KeepsakeId>` — **keepsakes have no mechanics**: named
mementos ("a knot of Whale-Road baleen") that accumulate in the manifest
for spec 7. ~10 authored keepsakes across the route.

Simulator gate update: at Cruise, an attended crossing on any strategy
should complete with **at most one drift**; Quiet/Mourn with none. The
per-waypoint table above is tuned against that assertion.

## Refits — the three A/B doors

At the first **three distinct shipyards** visited, the yard offers one
permanent either/or refit (authored pairs, in a fixed sequence so route
choice decides *which yards*, not *which pairs*):

| # | A | B |
|---|---|---|
| 1 | **Storm Sail** — all legs 10% faster | **Long Hold** — provisions cap 150 |
| 2 | **Quiet Keel** — named-threat roads take their kinder ledger row | **Deep Larder** — drift recovery restores 40 |
| 3 | **Lantern Mast** — unknown waypoints (`◌`) within one junction show their names | **Mourning Colors** — Mourn trim also x0.80 provisions |

Refit rules: choosing A closes B forever (doors close); a refit is
configuration, not a consumable; effects compose into the same integer
prices everything else uses. Four shipyards are authored (Graywater,
Saint Elm's, Drift's End, the Ember Hold) and content parity guarantees
≥2 per route — so every crossing sees at least two refit doors, most
see three.

## The Threats — where loss lives

Three named threats exist (spec 2 authored them onto roads). Each is a
scene at the road's destination with a **ledger, not a roll**: the outcome
is a pure function of state the player controlled, and the junction card
told them what mattered (via tags, counsel, and rumors).

| Threat | Road | The ledger (checked in order) | Worst outcome |
|--------|------|-------------------------------|---------------|
| **The Ossuary Warden** | over the reef (R9) | Sefa aboard (she sings the office) → safe passage, keepsake · arrived at Quiet/Mourn trim → it takes provisions (−15) · else → it takes provisions (−15) **and hope (−2)** | priced, never a soul |
| **The Silence itself** | the silent road (R29) | any soul resting (unstationed) that leg → they anchor the crew, safe · else → hope −2 and the leg's log stays blank | priced, never a soul |
| **The Thorns** | the thorn run (R42) | Cormac aboard **and** at Helm → one clean line, keepsake · Quiet Keel refit → hull holds, provisions −10 · else → **a soul at a station is lost** (helm first, then tender, then watch; resting souls are never taken) | the game's only loss |

Loss design notes:

- The Thorns are the only loss in Act 2 v1 — one road, Chapter IV, the
  fast expensive option at the last junction, with a threat line on the
  card, Cormac's counsel speaking directly to it, and two independent
  outs (the pilot, the refit). A player who loses someone chose a named
  danger over a flowered road and sent a crewed ship through unprepared.
  That is "priced, chosen, and become story."
- Loss takes a *stationed* soul: the post was the exposure. Resting souls
  are below. This makes the stations trade physical, and it means the
  player's standing choices — not a table lookup of who's expendable —
  decide who was in harm's way.
- The scene calls spec 3's `mark_lost` (hope −3, hull carving, counsel
  silenced) and plays a memorial beat. The covenant is untouched: the
  ledger only ever runs when the player chose the road while present.

## Boarding, Farewell, and Recovery Scenes

Already mechanically shipped in spec 3 as modals; spec 4 gives them scene
bodies:

- **Boarding asks** become 2-beat scenes (the meeting, then the ask door).
  Per-soul, authored at each of their sites — Ysolt met at Saint Elm's is
  mending hulls; met at the Beacon Graveyard she is salvaging lenses.
- **Farewells** become 1-beat scenes with a color line per departing soul.
- **Drift recovery** gets four authored scenes, one per chapter (the same
  event reads differently in the Shallows than the Starless Deep).
- **Chapter gateways** (the Shallows Gate, Drift's End, Deepgate) get
  slightly longer scenes (3–4 beats) — the act breaks.

## UI

The one-line `SceneModal` grows into a **scene view**: centered column,
one beat at a time, `[Enter]` next / `[Esc]` skip-to-end, payout line in
small print on the final beat ("the hold gains 20 · a rumor learned").
Doors render as the existing ask/refit prompt after the last beat. The
log keeps every played scene's title (the crossing's story so far, one
line each, readable from the Rumors panel renamed **the Log**).

## Writing Pipeline — the route content map

A worksheet (`docs/superpowers/specs/act2-route-content-map.md`, authored
with the implementation) with one row per waypoint: kind, payout, scene
beats summary, color lines available there, door if any, keepsake if any,
threat ledger if any. It is the single place to see coverage and tone
drift, and the PR artifact reviewers read instead of 38 scene files.

## What This Spec Does NOT Add

No option menus in scenes. No dice. No new gauges or currencies. No
procedural text. No letters (spec 6), no finale (spec 7), no weather or
nights (spec 5).

## Testing

- Scene engine: every `SceneRef` id resolves to a `SceneDef` or falls back
  to the placeholder; color lines select deterministically; payouts apply
  exactly once (save/load mid-scene replays cleanly)
- Economy: simulator asserts ≤1 drift at Cruise, 0 at Quiet/Mourn, all
  strategies still arrive in the envelope
- Refits: three doors per crossing when three shipyards are visited; A
  closes B; effects compose into card integers (snapshot-tested)
- Threat ledgers: full outcome matrix per threat as table tests — every
  row reachable, the Thorns' loss row only with a stationed soul, and
  `mark_lost` never called from any other scene
- Covenant unchanged: the 60-day offline property test still passes (no
  scene runs without the player)

## Open Questions

- Whether the Warden's provisions toll should scale with hold contents
  (lean: flat — predictable prices are the game's whole grammar).
- Whether a second loss ledger should exist in Chapter III for players
  who want higher stakes earlier (lean: not in v1 — one perfect loss
  beats two adequate ones).
- Keepsake count and whether any appear in Act 3 (park for spec 7).

## 2026-07-03-vessel-ferryman-design.md

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

## 2026-07-03-vessel-letters-going-dark-design.md

# Letters From Home & the Going-Dark

> **Doc-alignment note (2026-07-04):** every "hope ±N" price and the
> `MAIL_FAILS_HOPE_COST` constant below are stale — Hope was retired
> entirely (commit d39ad67). The shipped `LetterDef` (`letters.rs:22-28`)
> has only `sender/text/postscript`, no hope field; the mail-fails
> (Going-Dark) beat still fires exactly as described narratively, but costs
> nothing mechanically now — unlike the Silence-bank (which got a strain-
> soul mechanic to replace its hope cost), this beat's price was simply
> removed with no substitute. `LETTER_PARCEL_PROVISIONS` (provisions, not
> hope) is unaffected and still shipped as designed.

**Parent spec:** [2026-03-27-the-vessel-design.md](2026-03-27-the-vessel-design.md)
**Sub-project:** 6 of 7
**Depends on:** specs 2–5 (shipped — arrivals, the Log, the moments queue,
the two-gauge economy). **Feeds:** spec 7 (the manifest keeps the letters;
the Going-Dark shapes what the arrival means).

## Overview

Act 1's world does not vanish at launch — it writes. **Letters from home**
arrive at every port in the first half of the crossing: the Haven's new
warden, the Deep's guild, the fisher-fleets, the Loom-tenders you left at
the wheel. Each letter is a voice, a small parcel, and proof that the
world you spent a quarter-million lives building is still holding the
lamp for you.

Then, at the Threshold, the postmaster hands you the last one. And
somewhere past the Last Lantern comes the crossing's quietest, heaviest
beat: **the night the mail does not come.** The world behind you has gone
out. From there to the Tree, the only lights are the ones you carry.

This spec also formally settles Act 1's runtime fate (deferred twice):
after launch, Act 1 is **frozen fiction** — its state never ticks again,
and the letters are its only interface. The Going-Dark closes even that.

## Letters From Home

### Delivery model — no timers, no anxiety

Letters are delivered **at arrivals**, not on clocks: the mail catches up
to you at ports (the fiction), and nothing can be missed by being away
(the covenant). One letter waits at each waypoint arrival while the mail
still flows:

- **Flows**: Chapters I and II, and the Threshold (W23) — the final letter.
- **Never flows**: anywhere in Chapter III past the Threshold, or Chapter
  IV. The Going-Dark is one-way.

Delivery mechanics: after the arrival scene plays, the letter surfaces as
a **moment** (the existing one-line modal queue), and is kept forever —
the Log grows a "Letters kept" section, and spec 7's manifest binds them.

### The letters themselves

An authored, **sequenced** table (letter N is the Nth received — the
sequence tells a story of home slowly changing), ~12 letters plus the
final one. Senders rotate through the world the player actually built:

| Sender | Voice | Example beat |
|--------|-------|--------------|
| The Haven's new warden | formal, trying too hard | the rooms you built, kept warm; your bunk left made |
| The Deep guild | terse, professional | Layer 30 holds; the mercenaries drink to Torvald |
| The fisher-fleets | weather-worn, fond | Runa's old skiff found a new pair of hands |
| The Loom-tenders | reverent, precise | the Loom still hums your patterns; they send its "surplus" |
| Children of the harbor | crayon-honest | drawings of the Vessel, dictated postscripts |

Each letter carries:

- **Text** (2–4 sentences, authored; the sequence darkens subtly — later
  letters mention dimming zones, quieter harbors — foreshadowing without
  announcing)
- **A parcel**: +5 provisions ("the Loom's surplus", "the fleet's salt
  catch") — small on purpose; letters are a hope economy, not a second
  pantry
- **Hope +1 on roughly every third letter** (authored per letter, not
  rolled)

Soul color: letters may carry one postscript line keyed to a soul aboard
(`ColorKey`-style, reusing spec 4's machinery): the guild writes to
Torvald; a child asks after Runa's skiff.

### Act 1, settled

- At launch, Act 1's state is frozen exactly as the burn left it. The
  game tick never runs for that character again (already true in code;
  now it is design law, not a deferral).
- The letters are the *fictional interface* to the frozen world: the
  Loom's "surplus" in the parcels is narrative, not a computed WR figure.
- The stats panel's Act 1 numbers, if ever shown again, are a museum
  (spec 7 / Act 3 decide whether to show them).

## The Going-Dark

Three authored beats, all **location-triggered** (never time-triggered —
nothing this heavy happens while the player is away):

1. **The Threshold (W23)** — already authored in spec 4 ("the last place a
   letter can reach you"). Its letter is **the Last Letter**: longer,
   co-signed by every sender, and it knows what it is. The scene's
   existing Mourn color line stands.
2. **The Last Lantern (W24)** — the keeper sells stores and stories, and
   mentions, carefully, that the mail packet is late. First time that has
   happened. (One line added to the existing scene; dread, not event.)
3. **The night the mail does not come** — the first arrival *after* W24
   (whichever road was chosen): the crew gathers on deck out of habit at
   mail-hour, and nothing comes. The world behind has gone out. Authored
   moment, **hope −2** (priced, once, and the Long Silence is reachable
   from it if hope was already low — the design accepts this; Chapter III
   is the dark chapter), and the Log's entry is the act's shortest:
   *"No letters. There will be no more letters."*

After the third beat: `gone_dark = true`. No letters, ever again. The
chart's southern edge (home) dims a shade in the palette — the one
cosmetic touch.

### Why this works on the two gauges

The letters' +5 parcels quietly supplement Chapters I–II (the safe water).
Their removal lands exactly as Chapter III's roads get expensive — the
chapter *feels* darker because the economy genuinely tightened, without a
single number on a card changing. Hope loses its steady letter drip at
the same time, making Mourn trim and rest-day arcs the chapter's hope
engines — the people become the light, which is the act's whole argument.

## Data Model (build scope)

```rust
// src/vessel/letters.rs — authored table
pub struct LetterDef {
    sender: &'static str,
    text: &'static str,
    hope: u8,                       // 0 or 1; the Last Letter pays 2
    postscript: Option<(SoulId, &'static str)>,
}
pub const LETTERS: [LetterDef; 12];
pub const LAST_LETTER: LetterDef;   // delivered at the Threshold
pub const LETTER_PARCEL_PROVISIONS: f64 = 5.0;
pub const MAIL_FAILS_HOPE_COST: u8 = 2;

// VoyageState (serde defaults; old saves continue cleanly)
letters_received: u8,               // index into the sequence
gone_dark: bool,                    // set by the third beat
```

Engine: `arrive_at` delivers (queue a `SoulEvent`-style letter event for
the UI; parcel + hope apply at delivery, exactly once); the third beat
triggers on the first arrival whose predecessor-in-visited is W24-or-later
in Chapter III. All lazily evaluated, chunking-invariant, offline == live
bitwise.

## UI

| Surface | Content |
|---------|---------|
| Moments queue | each letter as a titled moment ("A letter from the Haven"), read like arc beats |
| The Log | new "Letters kept: N" line; letters listed with senders |
| Chart | home edge dims after the Going-Dark |
| Vessel panel | nothing — letters are moments, not a gauge |

## What This Spec Does NOT Add

No timers or missable mail. No reply mechanic. No new gauges. No
resurrection of Act 1 ticking. No Going-Dark choice — it is weather, the
act's largest weather, and it happens to everyone.

## Testing

- A letter at every Chapter I/II arrival, in sequence; parcels and hope
  apply exactly once; save/load mid-sequence resumes correctly
- The Last Letter delivers at W23 and nowhere else; no letter ever
  delivers past it
- The mail-fails beat fires exactly once, at the first arrival after W24,
  with its hope price; `gone_dark` is permanent
- Economy gates re-run with parcels in (Cruise ≤1 drift, Mourn 0 — the
  parcels may buy back a notch of Chapter I/II tuning if needed)
- Offline equivalence bitwise with letters in play; the covenant test
  extended: crossing into Chapter III offline queues the beats, harms
  nothing beyond their stated prices

## Open Questions

- Letter count: 12 + Last (lean) vs scaling with route length (~14 max
  arrivals before W23 on the longest route; extra arrivals past 12 simply
  get no letter — "the mail is thinning" — which foreshadows for free).
- Whether the mail-fails beat's hope −2 should be softened if hope is
  already ≤2 (lean: no — the floor mechanics already catch it, and the
  Long Silence in Chapter III is thematically correct).
- Whether Act 3 reopens anything of home (park for the Act 3 elevation;
  nothing in this spec forecloses it).

## 2026-07-03-vessel-pace-rations-naming.md

# Pace & Rations — the naming pass

**Parent spec:** [2026-03-27-the-vessel-design.md](2026-03-27-the-vessel-design.md)
**Sub-project:** 10 — a pure legibility pass on spec 5's Trim dial and
spec 8's rations toggle. No mechanics change.
**Depends on:** specs 5 and 8 (shipped/in-flight).

## The problem

The posture dial's four values — **Run · Cruise · Quiet · Mourn** — read
as *muddy*: they mix two metaphor families (two speeds, two moods), so
the eye can't sort them on one axis, and you can't tell which trades what.
Provisions, Hope, and Hull are legible because each names a single
concrete thing; the postures weren't.

## The fix

Name the dial and its values in **Oregon Trail's register** — by the toll
each takes on the party, not by an abstract speed. OT's own dials were
*Pace* (Steady/Strenuous/Grueling) and *Rations* (Filling/Meager/Bare
Bones); every word is a cost you can feel. We borrow the voice.

### Pace (was "Trim")

The panel label becomes **Pace**, and the four values sort top-to-bottom
as hardest→gentlest — which, being a pace, is also fastest→slowest:

| Was | Now | The toll (shown in the panel line) |
|-----|-----|------------------------------------|
| Run | **Grueling** | fastest — the hold empties and she scars |
| Cruise | **Steady** | the honest middle; never wrong, never best |
| Quiet | **Easy** | slower, sparing — quiet enough to hear the dark |
| Mourn | **Restful** | slowest — the crew mends and hope climbs |

The anti-muddy principle: **the name carries only the pace; the
one-line description under it carries the side-effect** (thrift,
listening, hope-healing). A name means one thing.

### Rations (was "Hard rations" toggle)

The two-state toggle from spec 8 is renamed in the same voice — still a
toggle, no third tier:

| State | Now | Effect (unchanged) |
|-------|-----|--------------------|
| off | **Filling** | the crew eats their fill |
| on | **Bare Bones** | burn ×0.75, hope −1/day |

Together, Act 2's supply layer now reads as a deliberate Oregon Trail
homage — a **Pace** dial and a **Rations** dial, both named by feel.

> **Doc-alignment note (2026-07-04):** the **Pace** rename shipped exactly
> as designed — `voyage.rs`'s `display_name()` returns
> Grueling/Steady/Easy/Restful for the `Trim` enum, matching the table
> above. Restful's "hope climbs" side-effect is stale (Hope retired,
> commit d39ad67); its shipped identity is purely thrift (×0.80 burn, "the
> thriftiest hold"). The **Rations toggle described here never shipped at
> all, and no longer exists as a concept** — `HARD_RATIONS_BURN_MULT` and
> the Press-the-helm/Hard-Rations mechanics were removed in the same
> Hope-retirement commit (grep confirms zero hits for
> `Rations`/`Filling`/`Bare Bones` as a mechanic anywhere in `src/vessel/`
> or `src/input/voyage_input.rs` — only flavor-text comments survive).
> Act 2's supply layer today is Pace alone, not a Pace-and-Rations pair.

## Scope (what does NOT change)

- **No mechanics.** Every multiplier, gate, and effect is identical;
  this is strings only. The `Trim` enum, its variants (`Run`/`Cruise`/
  `Quiet`/`Mourn`), `hard_rations: bool`, and the `[T]` hotkey and
  `VoyageView::Trim` view all keep their **internal** names — engine
  terms, never shown. Only `display_name()`, the panel title/label, the
  description lines, and the rations row change.
- **No save impact** (no serialized field touched).
- A third rations tier (OT's *Meager* middle) is deliberately **not**
  added here — that's a mechanics change that would want sim re-tuning
  the Ferryman work (spec 9) will re-touch anyway. Parked.

## Testing

- UI snapshots re-blessed (Pace panel, vessel gauge line, strip); no
  test asserts the old display strings, so the rename is string-only
- `make check` green

## 2026-07-03-vessel-price-of-passage-design.md

# The Price of Passage

**Parent spec:** [2026-03-27-the-vessel-design.md](2026-03-27-the-vessel-design.md)
**Sub-project:** 8 — the act's first balance revision, after the queue.
**Depends on:** specs 2–7 (all shipped). **Instrument:** the voyage
simulator and the balance probe methodology (160 instrumented crossings,
2026-07-03) that produced the diagnosis below.

## Diagnosis

The act has decision surfaces but no shortage. Across 160 simulated
crossings (4 strategies × 4 check-in cadences × staffed/unstaffed × 5
seeds): the Long Silence fired **zero** times, every staffed strategy
ended at hope 10, a staffed cheap-route ship never drifted, and every
manifest read alike — full berths, sound crew, nothing carved. Junctions,
berths, refits, trim, and watches all exist; none of them charge a price
in a currency the player is short of.

The fix is not dice and not danger of failure. It is **scarcity plus
ledgers**: make the currencies short, give hope a spend, let neglect
accumulate on people and hull as visible, caused, waiting consequences —
and let the manifest say what the crossing cost.

## The Law (unchanged, sharpened)

The stake is never *whether* you arrive. It is **who arrives, and in
what state**. The crossing stays unlosable; the manifest becomes volatile.
Everything below is a pure function of prior choices with the cause shown
— no rolls, ever — and nothing heavy lands offline: consequences are
acquired at deterministic sim-time events (like nights) and *surface* at
the next check-in as moments, exactly like arc beats today.

## 1. Scarcity (the enabler)

Way-station and rest-stop payouts come down until the cheap route
genuinely squeezes:

- Way-stations: 25–28 → **20–23** provisions
- Rest stops: 18–25 → **15–20**
- Letters' parcels, threat tolls, drift recovery: unchanged (the
  covenant's floors are not the problem)

**Tuning target** (sim-gated): a staffed, attentive cheapest crossing
floors below 10 provisions at least once and still completes with 0
drifts — tense, not punished. An unstaffed or inattentive one drifts ~1–2
times, as today.

> **Doc-alignment note (2026-07-04):** this section's fix — giving Hope
> spends (Press the helm, Hard rations) so the gauge would finally engage —
> **did not hold**: balance-sim evidence later showed Hope still pinned at
> its maximum under every attentive strategy even with these sinks in
> place, and commit d39ad67 retired Hope entirely rather than add a third
> sink. Press the helm (`[P]`), Hard rations, and every `hope`-gated
> mechanic in this section are gone — see
> `docs/superpowers/specs/2026-07-03-vessel-ferryman-design.md`'s Ward
> follow-up and `docs/decisions.md` ("Act 2 Ferryman Era: Retiring Hope")
> for what replaced this diagnosis's fix. Sections 3–4 below (strain, hull
> wear/scars) are unaffected and shipped as designed
> (`HULL_WEAR_MAX`/`WEAR_BURN_PER_SCAR` in `voyage.rs`).

## 2. Hope becomes a wallet

Hope's sources stand; it gains **sinks** — real purchases at check-in
moments, both showing final prices (composition law unchanged):

1. **Press the helm** `[P]` — while Traveling, once per leg: −2 hope,
   the leg's *remaining* time ×0.85. The crew digs deep; days are what
   it buys. Pressing twice within one chapter strains the helm soul
   (see §3) — the sink feeds the ledger.
2. **Hard rations** — a standing toggle beside trim: provisions burn
   ×0.75 while on; hope −1 per full day on. Oregon Trail's dial, priced
   in the act's own currencies. Turning it off is free; the hunger isn't.

Guard: sinks require hope ≥ 3. You can be *worn down* into the Long
Silence; you cannot *buy* your way in. (The wind bonus at 8+ now has an
opportunity cost — holding bright hope means declining to spend it.)

## 3. The strain ledger (sickness without dice)

Per met soul: `strain: 0 | 1 | 2` — sound, **strained**, **worn**.
Every acquisition is announced with its cause, at the next check-in.

**Causes** (deterministic, sim-time):
- Standing **3 consecutive nights** on watch → the stander strains
- Crossing a **squall at Run** while on any post → the posted soul strains
- **Helm through a silence bank** un-Quiet → the helm soul strains
- **Pressing the helm twice in one chapter** → the helm soul strains

**Effects:**
- Strained (1): affinity stops counting — helm/tender multipliers revert
  to the unaffine values, an affine watcher counts as merely Stood; rest
  accrues at half pace (a hurt person heals before they story-tell)
- Worn (2): cannot hold a post; arc paused entirely

**Recovery:** RestStop arrivals heal every **off-post** soul by one
level (posts are not rest — relieving someone is the decision). Surfaced
as a moment: "Runa mends at the Mirrorcalm."

**The teeth:** threat-road ledgers take the **most strained stationed
soul first** (ties break helm→tender→watch as today). The Thorns row
changes from "exposure is the post" to "exposure is the post, weakest
first." A loss now traces through three visible decisions — who stood
too many watches, who wasn't rested, which road you then chose.

**Manifest:** arrival state shows it forever — "came ashore, worn."

## 4. Hull wear (the wagon axle)

`hull_wear: 0..=6` — **scars**, counted in words on the vessel panel
("sound", "scarred ×3"), never a bar.

**Sources** (+1, announced with cause): a drift; a squall crossed at
Run; the threat rows where the ship takes the road on her own skin
(the Thorns quiet-keel and all-below rows, the Warden's hurried row);
pressing the helm at wear ≥ 4.

**Effect:** provisions burn ×(1 + 0.05 × wear). A six-scarred hull eats
30% more — wear compounds into the scarcity axis, not into speed. She
always sails; she just gets hungry.

**Repair — the third door:** shipyards offer **A / B / mend her**.
Mending zeroes wear and *closes that yard's refit pair forever* (the
doors-close pillar, now load-bearing: a hard-driven ship may finish with
one refit where a gentle one carries three). This is the only repair;
wear otherwise rides to the Tree and into the manifest ("she arrived
carrying four scars").

## Data Model (build scope)

```rust
// SoulState — serde(default)
strain: u8,                       // 0 sound, 1 strained, 2 worn
consecutive_watches: u8,          // resets on a night not stood

// VoyageState — serde(default)
hull_wear: u8,                    // 0..=6
hard_rations: bool,
pressed_this_leg: bool,
presses_this_chapter: u8,         // resets at chapter boundary
strain_events: Vec<StrainEvent>,  // surfaced like soul_events

// Composition law grows one term each, still shown as final integers:
// time = base × trim × wind × helm × press (× StormSail)
// provisions = base × trim × tender × rations × wear (× MourningColors)
```

Threat ledgers read strain; `choose_refit` gains the mend arm; RestStop
arrival heals; nights/squalls/banks acquire strain in `step_minute` /
`resolve_night` (chunking-invariant, offline == live bitwise, as ever).

## UI

| Surface | Change |
|---------|--------|
| Vessel panel | hull line ("scarred ×2 — the hold pays 10% more"); rations state; `[P] Press the helm` when available |
| Trim panel | hard-rations toggle row, priced like trims (final numbers) |
| Souls panel | strain shown per soul with its cause; worn souls can't cycle onto posts |
| Watch panel | "third night in a row" warning before it costs |
| Shipyard modal | three doors: A / B / mend |
| Moments | strain acquired/healed, wear taken — one line, cause named |
| Manifest / finale | strain state and scars recorded; carved-names beat unchanged |

## What This Spec Does NOT Add

No dice. No offline decay — everything acquires at deterministic
sim-time events and waits at the next check-in. No losable crossing —
drift recovery, the affordability invariant, and the never-locked
cheapest road all stand. No HP bars, no percentages — strain and wear
are small integers with words. No new berths math, no new route content.

## Testing / Sim Gates

- All strategies still arrive; 20–200 day envelope holds
- Cheapest staffed + attentive: 0 drifts, min provisions < 10 (the
  squeeze exists), 0 strains with good watch rotation
- Neglect profile (unstaffed, 48h check-ins): ≥1 strain and ≥2 wear at
  arrival — the manifest finally varies
- Hard-rations-abuse profile: hope min ≤ 2 (the gauge finally bites);
  Long Silence reachable but only through sustained choice
- Run-everywhere profile: ≥2 wear; mend-vs-refit decision reached
- Strain determinism: same seed + same assignments → identical ledger;
  offline == live bitwise through save/load
- Threat exposure: staged crossings prove most-strained-first ordering
- Save compat: all new fields serde(default); pre-spec-8 voyage.json
  loads sound, unworn, full-rationed

## Open Questions

- Whether Mourn trim should also heal strain at sea (lean: no — RestStops
  keep their monopoly on mending people; Mourn already owns hope)
- Whether wear should show on the chart's ship glyph at high scars
  (lean: yes at 4+, a color shift, no new glyph)
- Whether the Outfitting (menu item 6) and ford/ferry/caulk junctions
  (item 5) become spec 9 after this lands (lean: evaluate with fresh
  probe data — scarcity may make existing junctions bite enough)

## 2026-07-03-vessel-route-waypoints-design.md

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
  **~46 roads** (shipped as exactly **45**, `route.rs:620`).
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
  rule, resolving the parent spec's open question). **Stale (2026-07-04)**:
  Hope was retired entirely (commit d39ad67), and `HOLD_STATION_GRACE_DAYS`
  was removed along with it (`CLAUDE.md`'s retired-constants note) — there
  is currently no soft pressure at all for holding station indefinitely.
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

## 2026-07-03-vessel-souls-design.md

# The Souls — Roster, Stations, Arcs, and the Wind

> **Doc-alignment note (2026-07-04):** the roster, stations, and arcs
> below shipped as designed and remain accurate (8 souls, 7 berths,
> Helm/Tender/Watch, `ARC_BEAT_REST_DAYS = 2`). **"Hope is the Wind"
> (the whole section below) did not survive** — Hope was retired entirely
> (commit d39ad67), so it has no mechanical effect, no wind, no Long
> Silence, and no bands. Every "hope ±N" beat payout, `LOSS_HOPE_COST = 3`,
> and `FAREWELL_HOPE_COST = 1` are gone too — `farewell()` and `mark_lost()`
> (`voyage.rs`) cost nothing now. Loss stays authored-scenes-only and
> memorialized exactly as designed; only its hope price is gone.

**Parent spec:** [2026-03-27-the-vessel-design.md](2026-03-27-the-vessel-design.md)
**Sub-project:** 3 of 7
**Depends on:** spec 2 (shipped — route graph, voyage state machine, junction
cards). **Feeds:** spec 4 (arrival scenes are where souls board, speak, and
are lost), spec 5 (the watch reads affinity; watch-vs-rest), spec 7 (the
manifest).

## Overview

Act 1 was a solo power fantasy. Act 2's cast is the point: **the Vessel
carries people, and the game's second gauge measures how they are doing.**
This spec defines the roster (who exists, where they are met), stations
(the standing assignment loop), arcs (each soul's personal thread), hope's
one mechanical effect (the wind), and loss (authored, priced, memorialized,
and never offline).

Everything here follows the parent pillars: small numbers, no hidden
arithmetic on screen, doors close, and the covenant — nothing harms a soul
while the player is away.

## The Roster

### Three board at launch

Faces from the systems the player mastered. Their dossiers are the tutorial
for every soul mechanic.

| Soul | Origin | Voice (one line) | Affinity |
|------|--------|------------------|----------|
| **Torvald** | the Deep's guild captain | "I've been lower than dark. It blinks first." | Helm |
| **Eir** | the Haven's warden | "A ship is a house that argues with the sea." | Tender |
| **Runa** | the fisher | "Everything worth catching sings first." | Watch |

A soul has exactly **one aptitude axis: affinity** (Helm, Tender, Watch, or
none). It does double duty — it strengthens the matching station's effect
*and* it is what the night system reads (spec 5): there is no separate
per-soul night-suitability table. Runa at Watch answers singing nights
because Watch-affinity is what answering nights *is*.

### Five found along the route

Each recruitable soul has **one site per branch arm** at the junction diamond
where they are met, so every route meets every recruitable — *different
scenes, same person* (content-parity rule 5: different souls never means
fewer souls). Sites use the existing `Feature::SoulCandidate` waypoints.

| Soul | Met at (any one of) | Who they are | Affinity |
|------|---------------------|--------------|----------|
| **Maren** | the Lightship Vigil (W1, spine) | the last lightship keeper; asks to see one more lit lantern | Watch |
| **Sefa** | the Drowned Choir (W3) · the Kelp Meadows (W5) | the last cantor of the drowned parishes | — |
| **Ysolt** | Saint Elm's Rest (W14) · the Beacon Graveyard (W16) · the Pilgrims' Buoy (W18) | a mender of hulls and of the other kind of damage | Tender |
| **Cormac** | the Whale Roads (W20) · the Smugglers' Slip (W21) | a pilot who knows the roads nobody charts | Helm |
| **Brother Wren** | the Choir of Bones (W26) · the Sleepers' Trench (W28) | woken from a years-long sleep; remembers the deep from inside | — |

The remaining soul-candidate waypoints (W9 the Ossuary Reef, W12 the
Wandering Fair) host **arc beats**, not recruitments.

Each station has exactly two affine souls (whose route determines which
you meet first), and two souls — Sefa and Brother Wren — have none: their
whole value is counsel and their arcs. A no-affinity soul is the spec's
proof that stations are not the only reason to want someone aboard.

### Berths: seven

3 launch + 5 found = **8 possible asks against 7 berths.** A player who says
yes to everyone faces exactly one **farewell** — naming who steps ashore at
the next waypoint (a scene, a manifest line, and a hope cost; never a
death). Declining an ask is permanent: that soul's door closes, their name
stays on the chart. The berth question is always *who*, never upkeep —
souls eat nothing.

Boarding and farewell are **scenes** (spec 4 delivers them); this spec owns
the state machine: `Ask → Aboard | Declined`, `Aboard → Ashore (farewell) |
Lost (authored only) | Arrived`.

## Stations

**Three standing posts** — one per system the ship actually runs on: time
(Helm), provisions (Tender), nights (Watch). Assigned from the Souls panel
(`[S]`), persistent until changed (including offline). One soul per
station; an unassigned station simply lacks the effect. **A soul at a
station is not resting** — the standing coverage-vs-story trade, and the
core reason a 7-berth roster matters: three posts, and arcs only move for
the souls you let rest.

| Station | Standing effect (any soul) | With affinity soul |
|---------|---------------------------|--------------------|
| **Helm** | legs 4% faster | 8% faster |
| **Tender** | leg provisions −5% | −10% |
| **Watch** | stands every typed night by default (editable per night from spec 5's forecast panel) | typed nights resolve one grade kinder |

(There is deliberately no fourth post. An earlier draft had a "Keel"
station for threat/dark-road protection; it was cut — threat pricing
belongs to the road card and spec 4's scenes, not to a passive slot.)

Rules that keep the arithmetic invisible on screen:

- Effects surface **only as final prices** — the junction card and trim
  panel already print composed integers; stations change those integers,
  never show a multiplier.
- Composition: `time = base × trim × wind × helm`, provisions likewise
  with tender. Order is fixed and documented in code.
- Counsel, not buffs, is the visible face: at a junction, each soul aboard
  may contribute **one line of counsel** on one card, in their voice
  ("Torvald: 'The narrows are honest. Hungry, but honest.'") — authored per
  (soul × road), a bounded writing table (~2 lines per soul per junction
  where they have an opinion; most souls are silent at most junctions).

## Arcs

One personal thread per soul: **three beats and a resolution**, unfolding
at particular places and paid for in rest days.

- **Beat advancement**: a beat becomes *ready* when its trigger is met
  (reach a tagged place, sail a tagged road, a chapter boundary — all
  authored per-soul); a ready beat **fires after the soul accumulates 2
  rest days** (days neither stationed nor standing watch). The Souls panel
  shows `rest days to next beat: 1 of 2` — the price is always visible.
- **Beat payout**: a log entry in the soul's voice, and one of: hope +1,
  a rumor (their private knowledge becomes a chart annotation), or a
  junction counsel upgrade. Resolutions pay hope +2 and a manifest line.
- **Arcs resolve before the Roots**: any unresolved arc fast-forwards its
  resolution scene during the Chapter IV approach — nobody's story is left
  dangling at the finale (spec 7 reads resolutions into the manifest).
- Beats never punish: an ignored arc simply waits. The only cost of
  neglect is time (and the finale reflecting what was and wasn't heard).

Example (Sefa): boards singing a lament (beat 0) → *the Ossuary Reef or
any LanternSite*: she asks to sing for the dead (beat 1, hope +1) → *first
silence-bank survived* (Chapter III): her voice is the thing the silence
couldn't take (beat 2, rumor) → resolution: she teaches the crew the
evening office; singing nights are kind to everyone forever (resolution,
hope +2, manifest line).

## Hope is the Wind

Hope gets its one mechanical effect (parent spec: "high hope is wind").
Bands, composed into leg time like trim:

| Hope | Label range | Wind |
|------|-------------|------|
| 8–10 | high / singing / radiant | legs 10% faster |
| 5–7 | steady / warm / bright | — |
| 3–4 | low / uneasy | legs 10% slower |
| 1–2 | guttering / failing | legs 25% slower |
| 0 | ashen | **the Long Silence** |

**The Long Silence**: legs crawl (+40%) and arcs pause. It breaks at the
next RestStop arrival (the scene plays a fire relit) — hope returns to 3
("low"). It is a valley, not a fail state; the crossing still cannot be
lost.

What moves hope (consolidated; all sources land in the two-gauge economy):

- **Up**: arc beats (+1) and resolutions (+2), Mourn trim at sea (+1/day),
  kept letters (spec 6), a handful of authored scene payoffs (spec 4)
- **Down**: hold-station past grace (−1/day, floor "steady"), soul loss
  (−3), farewell (−1), a named-threat scene gone badly (authored, priced
  on the card), silence-banks unshielded (spec 5)

## Loss and the Memorial

- Souls are lost **only in authored scenes** attached to named threats —
  the threat was on the junction card, the road was chosen, and the scene
  offered a priced alternative. No dice: loss follows from a choice whose
  stakes were stated (parent rule: catastrophes are priced, chosen, and
  become story).
- **The covenant, mechanical form**: no tick-driven code path may reduce
  the roster. Nights, weather, drift, hold-station, offline resolution —
  none touch souls. CI enforces this as a property test (simulate
  arbitrary offline windows; roster count is invariant).
- A loss carves the soul's name into the hull: the ship art gains a
  carved-name line for the rest of the game (Act 3 included), the arc
  becomes a memorial manifest line, hope −3, and their counsel lines go
  silent — the junction feels emptier, which is the design intent.

## UI Surfaces

| Surface | Content |
|---------|---------|
| Souls panel (`[S]`) | roster with faces/voice lines, station assignment, arc status ("rest days to next beat"), dossier per soul |
| Junction cards | counsel lines (soul voice + road read) appended to existing annotations |
| Chart | recruit sites render their `SoulCandidate` marker with a small `☺`-style accent once known |
| Boarding ask / farewell | scene modals (spec 4 shapes; engine provides the state transitions) |
| Ship art | carved names after losses |
| Vessel panel | hope line gains its wind arrow (`hope: bright ↑` when wind aids, `↓` when it drags) |

## Data Model (engine-side, this spec's build scope)

```rust
pub struct SoulDef {            // authored table, like route.rs
    id: SoulId,
    name: &'static str,
    voice: &'static str,        // the one-line personality
    origin: &'static str,
    affinity: Option<Station>,  // one axis: station bonus AND night behavior
    sites: &'static [WaypointId],  // empty = boards at launch
    arc: &'static [ArcBeat],    // trigger + payout per beat
    counsel: &'static [(RoadId, &'static str)],
}

pub struct SoulState {          // in voyage.json
    soul: SoulId,
    status: SoulStatus,         // Aboard | Declined | Ashore | Lost | (implicit: NotMet)
    station: Option<Station>,
    arc_beat: u8,
    rest_day_minutes: u64,      // accumulates only while resting
}
```

Constants: `BERTHS = 7`, `ARC_BEAT_REST_DAYS = 2`, wind bands above,
station multipliers above, `LOSS_HOPE_COST = 3`, `FAREWELL_HOPE_COST = 1`.

## What This Spec Does NOT Add

No soul stats, levels, or equipment. No morale-per-soul (hope is one
shared gauge). No procedural souls — eight authored people, full stop. No
soul death outside authored scenes. No upkeep. No fourth station, and no
per-soul night tables — affinity is the single aptitude axis, read by
stations and nights alike. The load-bearing triangle is
**stations ↔ arcs ↔ wind**; everything else on a soul is voice.

## Testing

- Roster invariants: every maximal route passes at least one site of every
  recruitable soul (per-soul cut across all 96 routes — extends the
  existing parity test)
- Berth overflow: 8 asks against 7 berths forces exactly one
  farewell-or-decline; both paths permanent
- Covenant property test: no sequence of offline ticks changes roster
  count or fires a loss
- Wind: hope bands compose into leg time deterministically; Long Silence
  pauses arcs and breaks at a RestStop; offline == live still bitwise
- Arc engine: beats gate on trigger + rest days; stationed souls
  accumulate none; resolution fast-forward at Chapter IV
- Station effects: final card prices shift by the documented composition;
  snapshot tests cover the Souls panel and counsel-bearing junction cards

## Open Questions

- Farewell timing: immediate at the current waypoint vs "at the next
  harbor" (lean: next waypoint with a scene, so it lands as story).
- Whether counsel lines should ever disagree with the card's own tags
  (lean: yes, rarely, and authored — Torvald being wrong once is worth
  more than him being a UI hint forever).
- Hull carving for *farewelled* souls (lean: no — carvings are for the
  lost; the manifest remembers the ashore).

## 2026-07-03-vessel-underway-design.md

# Underway — Weather, Trim, and the Watch

> **Doc-alignment note (2026-07-04):** every hope-priced effect below
> (Silence-bank "hope −1/day," Mourn "the only trim that raises hope,"
> Strange-night-unstood "hope −2") is stale — Hope was retired entirely
> (commit d39ad67). Two of these were replaced, not just deleted: the
> Silence-bank at non-Quiet trim now strains the Helm soul instead
> (`StrainCause::SilenceHelm`, `voyage.rs`) — a genuine mechanic swap — and
> Mourn's identity was re-authored as the thriftiest hold (×0.80 burn)
> rather than a hope-gaining trim (`voyage.rs`, explicit comment: "it no
> longer raises hope... its identity now that hope is retired"). The
> Strange-night cost is provisions (−3.0), not hope, and was already
> provisions-priced before the retirement in the shipped build. "The two
> existing gauges" language just below no longer applies — Provisions is
> now the only crossing-level gauge; the equivalent pressure during the
> Colony era lives in the Ward yard instead (`colony.rs`).
**Sub-project:** 5 of 7 (replaces the old "Vessel Underway" scope)
**Mockups:** the "Traveling — Mid-Leg Check-In" set (four screens: return/log,
living chart, trim, watches)

## Overview

Traveling is not dead time with flavor text. While the Vessel is on a leg:
**the void has weather** (objects that move on the chart in real time),
**the ship has trim** (one posture dial, priced against today's weather),
and **the night has watches** (typed nights, scheduled souls, and a log the
nights write themselves). Other pilgrims' ships cross the same chart on
their own roads.

The design contract, inherited and kept: **no dice, no menus**. Every travel
effect is a standing choice with a stated price (trim, watch assignment) or
a world event that was visible before it arrived (weather on the chart,
night types in the forecast). All effects land in the two existing gauges,
time, rumors, or the log — no new resources.

**The check-in loop** (2–4 minutes, any time mid-leg):
1. **Read** — the log of the nights since you left (written, not tallied)
2. **Look** — the chart: what weather moved, who else is out there
3. **Trim** — one dial if the weather changed
4. **Watch** — fill any unassigned typed night in the forecast

Design budget: **at most one thing per check-in actually asks** (an
unassigned strange night, a current crossing the road). Everything else is
readable state.

---

## Void Weather

### Object types (v1: exactly three)

| Type | On the chart | Effect while it overlaps your leg | Trim interaction |
|------|--------------|-----------------------------------|------------------|
| **Current** (e.g. "the Northing") | `≋ ≋ ≋` band with a bearing, drifting across edges | None passively — currents are *opportunity* | **Run** rides it: big time gain at no extra provisions burn. Against-bearing currents make Run cost more instead |
| **Silence-bank** | `▒▒▒` shaded region | Hope −1/day while inside | **Quiet** nullifies the hope drain and can hear what the silence hides (a rumor, once per bank) |
| **Squall** | `≈≈*≈` flickering patch, fast-moving | Provisions −2/day while inside | **Mourn**/**Quiet** shelter through it (half cost); **Run** through a squall doubles its tax |

Weather never damages souls and never touches arcs. It prices time, hope,
and provisions — the things the player already trades.

### Lifecycle and visibility

- Weather objects live on the **route graph** (edges and small edge-regions),
  not free 2D space. They spawn, drift edge-to-edge on a bearing, and
  dissipate after **2–5 real days**.
- **At most 2** weather objects may affect the player's current leg at once;
  at most 4 are visible on the chart (the rest of the void's weather is
  simply not shown).
- The chart shows weather within one junction's distance — same horizon as
  the fog-of-route. Rumors can report weather further out ("the Northing
  runs the Tollgate road all season"), and such rumors are *forecasts*,
  aging out when the weather does.
- Movement steps happen on wall-clock hours (deterministic; see Determinism)
  so a check-in every few hours genuinely sees things move.

### Generation

Each chapter has an authored **weather deck**: 6–10 template cards
(type, strength, typical bearing, flavor name pool) with chapter character —
the Shallows run mild currents; the Starless Deep deals long silence-banks;
squalls cluster at chapter boundaries. Draws and drift steps are seeded
(see Determinism). Named story weather (e.g. a scripted silence-bank at the
Going-Dark) is placed by the route script, not drawn.

## Trim

One posture, settable any time aboard, persistent until changed
(including offline). Base table:

| Trim | Leg time | Leg provisions | Extra |
|------|----------|----------------|-------|
| **Run** | ×0.80 | ×1.30 | Pilgrim ships you're pacing fall behind |
| **Cruise** | ×1.00 | ×1.00 | The default; never wrong, never best |
| **Quiet** | ×1.20 | ×0.90 | *Hears more*: silence-banks yield their rumor; singing/strange nights resolve one grade kinder |
| **Mourn** | ×1.40 | ×0.90 | Hope +1/day; the only trim that raises hope at sea |

Weather multiplies on top (composition rule:
`time = base_time × trim_time × weather_time(trim, weather)`), and the trim
panel always shows the **final computed prices** — "arrive ~14h early ·
provisions −9 over the leg" — never the multipliers. Small integers on
screen; arithmetic stays behind the curtain (pillar: every number fits in
a sentence).

ETA in the footer re-derives from trim + live weather. The road's junction
card promised a *base* price; trim and weather are how the player does
better (or deliberately worse) than the promise.

## Nights and the Watch

### Night types

Every leg-night has a type, forecast **3 nights ahead** on the chart panel:

| Type | Frequency | Stood (any soul) | Unstood |
|------|-----------|------------------|---------|
| **Quiet night** | ~40% | a log line; nothing asked | — (quiet nights need no watch) |
| **Cold night** | ~15% | provisions −2 instead of −5 | provisions −5 |
| **Hungry night** | ~15% | provisions −3 instead of −6 | provisions −6 |
| **Singing night** | ~15% | a log line of the song | a log line of something missed |
| **Strange night** | ~15%, **max 1 per leg** | lore log line; nothing taken | hope −2, and "the night keeps whatever it was going to say" |

A **Watch-affine** soul (spec 3) standing any typed night resolves it one
grade kinder: cold/hungry costs drop to −1/−2, singing nights yield the
rumor or arc beat, strange nights pay out their lore/unique rumor.

Night suitability is not a separate table: nights read the soul's single
**affinity** axis (spec 3 — the same property that strengthens their
matching station). A Watch-affine soul (Runa, Maren) stands any typed
night one grade kinder; every other soul stands it at the base outcome.
No hidden stats, one axis per soul. The Watch-stationed soul covers typed
nights by default; the forecast panel edits per night.

### Scheduling rules

- One soul per night, assigned from the forecast panel; assignments persist
  and can be edited until the night begins.
- **A soul on watch is not resting, and arcs only advance on rest days.**
  This is the travel layer's standing trade-off: coverage vs story. A soul's
  arc panel shows rest-days-to-next-beat so the price is always visible.
- Unassigned typed nights resolve "unstood" (table above) — allowed, priced,
  never catastrophic. **No night ever injures or removes a soul.**
- Quiet nights need no assignment and don't interrupt anyone's rest.

### The log

Every night writes **one entry**: template pools keyed by
`(soul × night type × outcome)`, with weather and road salt mixed in.
Entries are first-person, 2–3 lines, in the soul's voice. Returning after
time away opens on the unread log — **idle time produces prose, not
tallies** — with any mechanical deltas appended in small print under each
entry.

The full log persists for the whole crossing and becomes part of the
arrival keepsake (alongside the chart). Content budget: ~12 templates per
soul-voice × 5 night types with outcome variants ≈ **~120 short entries**
per authored soul set — the largest single writing cost of this spec, and
the most-read text in Act 2.

## Other Pilgrims

**Five authored ships**, not a simulation. Each has a name, a silhouette,
a one-line character, and a **route script** (which roads, which real-day
windows, where her road coincides with plausible player routes).

- **On the chart** they render as moving lights with names, visible inside
  the same one-junction horizon.
- **Hail** (`[H]`, once per meeting): a short exchange — they trade rumors
  about roads *they* have sailed (pilgrim rumors are the only way to hear
  about roads behind you or parallel to you).
- **Matched course**: while your roads coincide and trims are compatible
  (not Run), watches are shared — one typed night per coincident stretch is
  covered by *their* crew, and the log entry arrives in a stranger's voice.
- **Scripted fates**: one ship goes dark in Chapter II (foreshadowing the
  Going-Dark); at least one survives to the Tree and stands in the harbor
  at arrival — a face for Act 3. Fates are authored, not simulated; the
  player's choices don't save or doom other ships (their story is weather,
  not consequence — this keeps the authoring bounded).

## Determinism, Offline, and the Covenant

- **Seeded**: weather draws/drift, night-type sequences, and template
  selection derive from `(voyage_seed, day_index)` — offline resolution
  and live play produce identical worlds; save-scumming changes nothing.
- **Offline**: trim holds; scheduled watches stand as assigned; unassigned
  typed nights resolve unstood; weather moves on its wall-clock steps;
  pilgrims follow their scripts; the log accumulates. Arrivals still hold
  station for the player (unchanged).
- **The covenant extends to travel**: nothing while the player is away —
  weather, nights, silence-banks — ever injures, removes, or advances-to-
  loss a soul. The worst offline outcome is priced provisions/hope and a
  night that kept its secret.
- **Anti-anxiety rule**: unread log entries never expire, and no travel
  choice has a deadline shorter than "before that night begins" /
  "while the weather lasts" — both visible on the forecast.

## UI Surfaces

| Surface | Content |
|---------|---------|
| Chart (main screen) | weather objects on edges, pilgrim lights, wake lanterns, trim + tonight's watch in the Vessel panel, weather summary block |
| Return view | unread log entries (auto-opens after >12h away), then to chart |
| Trim panel (`[T]`) | four postures with final computed prices against live weather |
| Watch panel (`[W]`) | 3-night forecast, affinity notes per soul, rest/arc status |
| Hail (`[H]`) | pilgrim exchange when one is in range |

## What This Spec Does NOT Add

No new gauges. No combat. No mini-games at watch change. No free 2D sailing
(the route graph stands). No procedural pilgrim AI. No weather that blocks
a road outright (weather prices roads; junctions choose them).

## Testing

- Weather lifecycle: spawn/drift/dissipate deterministic from seed; ≤2 on
  leg, ≤4 on chart invariants hold
- Trim composition: final price tables for all trim × weather pairs; ETA
  re-derivation matches footer
- Night resolution: each type × (suited / unsuited / unstood) outcome
  matrix; strange-night cap of 1 per leg; no-harm covenant (property test:
  no offline sequence reduces the soul roster)
- Offline equivalence: N days simulated offline == same N days ticked live
  with identical seeds (the determinism property, the spec's load-bearing
  test)
- Log: every (soul × night × outcome) key resolves to a template; unread
  queue survives save/load
- Pilgrim scripts: coincidence windows compute correctly against arbitrary
  player routes; hail-once-per-meeting enforced

## Open Questions

- Whether matched-course sharing should also ease provisions slightly
  (convoy economics) or stay watch-only (current lean: watch-only — one
  effect per system).
- Whether weather rumors (forecasts) occupy the same rumor inventory as
  road rumors or a separate short-lived slot (lean: same inventory, aging
  entries marked).
- Log volume tuning: one entry per night may be too chatty at 4-day legs ×
  5 months; possible digest mode ("three quiet nights passed") for
  quiet-night runs.
- Whether Mourn is selectable with nothing to mourn (mockup says yes — "the
  option is always here" — it reads as intent; confirm it can't be abused
  as a hope pump given ×1.40 time is the natural brake).
