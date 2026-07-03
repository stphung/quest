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
- **A station** (helm, tender, watch, keel) — souls at stations recolor
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
| 2 | The Route & the Waypoints | chart structure, leg/arrival state machine, junction choice rules, road character & pricing, fog & rumors, tree-on-horizon render | **Next to write** |
| 3 | The Souls | roster, stations, arcs, hope, loss & memorial, offline covenant | Not started |
| 4 | Arrival Scenes | scene delivery system, road/refit/soul recoloring, scene shapes, writing pipeline | Not started |
| 5 | Underway — Weather, Trim & the Watch | void weather, trim postures, night types & watch rotation, the log, pilgrim ships, drift, refits | **Spec written** — [2026-07-03-vessel-underway-design.md](2026-07-03-vessel-underway-design.md) |
| 6 | Letters From Home & the Going-Dark | care packages, letter templates, the midpoint event | Not started |
| 7 | The Arrival | final approach, manifest, keepsake chart, Act 3 gate | Not started |

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
