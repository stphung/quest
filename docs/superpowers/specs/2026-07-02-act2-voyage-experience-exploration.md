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
