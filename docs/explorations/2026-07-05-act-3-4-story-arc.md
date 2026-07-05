# Exploring: the story arc for Act 3 and Act 4

**Status**: pre-commitment exploration, crystallized through an interactive
`/opsx:explore` pass (see History at the bottom). Act 3 now has a concrete
working direction; Act 4 is deliberately still open. Nothing here should be
implemented — graduate it into `docs/dossiers/world-and-narrative.md` (the
"arc across acts" table) and then into an `/opsx:propose` change when someone
is ready to build.

**Prompted by**: `world-and-narrative.md`'s own "Open narrative questions"
list (Act 3/4 rows are "TBD" / "Not designed"), plus three concrete hooks
Act 2 already built for whoever answers them — `vessel_arrived`,
`last_crossing_complete`, and the Sister Verity.

## Where the story actually stands right now

Nothing here is invented — it's what's already shipped (dark) and waiting:

- **`vessel_arrived`** fires at first landfall. The normative spec calls this
  "the durable hook a future Act 3 keys off" — but it's the *shallow* hook.
- **`last_crossing_complete`** fires only once the old world is fully
  emptied and the Ferryman era ends. `colony.rs`'s own module doc calls this
  one outright: *"the next arrival is the Last Crossing: Act 3's gate."* This
  is the deep hook, and the one this exploration keys the whole act off.
- **The Sister Verity** is a named pilgrim ship, authored to reach the Tree
  and *wait there* rather than going dark like the other four pilgrims. She's
  a face already standing at the harbor, with no scene to spend her on yet.
- **Letters Home** thin out and stop as the Vessel leaves the shore — a
  "going dark" device explicitly flagged in `world-and-narrative.md` as
  reusable, with an open question attached: does it ever pay off?

So the game already has, sitting unused: an empty gate flag, a person waiting
at a door, and a silence that's never been broken.

## The pattern so far, and the unclaimed vector

| Act | Verb | Shape |
|---|---|---|
| 1 — Ascent | **Climb** | up 50 zones |
| (Deep, inside Act 1) | **Descend** | down numbered Layers |
| 2 — Crossing | **Cross / ferry** | outward, across the dark, repeatable until the well runs dry |
| 3 — The Rooting | **Root / Graft** | *in* — down into Yggdrasil's roots, decided below |
| 4 | ? | deliberately still open |

Each act so far has also *deliberately broken* the previous act's design
pattern rather than extending it (Act 2's Fun Assessment dossier section
says this outright: "confirm, don't fix"). Up, down, and across were spent —
the unclaimed vector was **in**: every currency name (the Loom *of Worlds*,
Ascension, the Storm) has been gesturing at Norse cosmology without cashing
the check. Act 3 is where the Tree stops being a destination on the horizon
and becomes the place the story happens *inside*.

## Act 3: The Rooting — the decided direction

Reached by working through the candidate verbs (Root/Graft, Weave, Tend,
Search — see Alternatives Considered) and picking Root/Graft, then resolving
five follow-on questions about how it actually plays. This is now concrete
enough to design against, not just a vibe:

```
last_crossing_complete fires
        │
        ▼
  Cold open (authored scene): the Letters-Home silence breaks;
  the Sister Verity's own scene plays
        │
        ▼
  The Grove + Wyrd unlock immediately
        │
        ▼
  (soft ramp — a narrative beat, not a number)
        │
        ▼
  Root Zones open
```

- **Gate**: `last_crossing_complete` is the whole gate — no additional
  ceremony beyond the cold-open scene, consistent with it already being
  named "Act 3's gate" in the code's own doc.
- **Cold open**: the two dangling threads from Act 2 get spent immediately,
  before any new system opens — the going-dark silence breaks, and the
  Verity (authored to reach the Tree and wait) finally gets her scene.
- **Cadence**: idle-first, like Act 1 — tick-driven, no wall-clock check-in
  cadence, no new failure states. This is a deliberate contrast with Act 2
  (which was wall-clock and consequence-bearing) — Act 3 swings back toward
  Act 1's contract rather than extending Act 2's.
- **Three combined axes** (Act 1 itself layers zones + production (Loom) +
  base-building (Haven) rather than picking one, so Act 3 does the same):
  1. **The Grove** — a wholly new colony-growth system, its own overlay and
     name (not a Haven branch) — signals Act 3 as genuinely new territory.
     This is where the rescued colony (Act 2's payoff, thousands of souls)
     actually takes root in the living branch.
  2. **Root Zones** — a finite, authored band (mirrors Zones 1-10 and the
     Loom's 31-50, not an open-ended Expanse-style tail), fought against a
     purpose-built decay/rot bestiary — new content, not reskinned
     archetypes, specifically so combat reads as "defending new growth"
     rather than generic climbing.
  3. **Wyrd** — a light secondary fate currency (the Norns/Weave idea,
     folded in as a minor thread rather than made the primary system),
     spent as a modest permanent lever — a small mini-Ascension scoped to
     Act 3, giving it a power role like every other endgame currency
     instead of staying purely cosmetic.
- **Loom stays completely untouched** — no interaction, its own island,
  the same relationship Act 2 had to Act 1's systems.
- **Stakes**: accept the same idle-first, low-mechanical-stakes tradeoff
  Act 1 makes (which scored 2/5 on "stakes and texture" by design) — Act 3
  doesn't need to reinvent consequence to be a good idle act; that's Act 2's
  job, not Act 3's.

## What this resolves against the open narrative questions

| Question (from `world-and-narrative.md`) | Answer under The Rooting |
|---|---|
| What is at the Tree? | A threshold, not a destination — the roots, and a branch (the Grove) to graft the colony onto |
| Does the colony become Act 3's home base/faction? | Yes — the Grove *is* that base, purpose-built rather than a Haven branch |
| Does "going dark" pay off? | Yes — the cold open, before any new system unlocks |
| How do Ragnarök beats map onto a final act? | Not decided — see Act 4, below |
| What's Act 3's verb? | Root / Graft |

## Act 4: deliberately still open

Ragnarök surfaced as a strong candidate during this pass — the game's whole
vocabulary (Yggdrasil, Æsir gear as actual items, "the endless") has been
building toward Norse eschatology without naming it, and the colony (souls
carried inside the World-Tree) already parallels the Líf/Lífthrasir survival
myth Act 2's ending re-tells without saying so. But this exploration
explicitly deferred deciding Act 4 — it's a real option worth returning to,
not a settled one. Revisit once Act 3 has more shape (or has shipped), the
same way Act 3 itself waited on Act 2 landing first.

## Alternatives considered (and why they lost)

| For | Option | Why it lost |
|---|---|---|
| Act 3 verb | **Weave** — Norns' fate-loom as the primary system | Too abstract as a whole-act resource without a concrete hook; folded in as Wyrd, a secondary currency, instead of the primary axis |
| Act 3 verb | **Tend** — pure colony-management, no mythic framing | Lost to Root/Graft, which keeps the mythic throughline the game's vocabulary has been building |
| Act 3 verb | **Search** — go find the four pilgrim ships that went dark | Lost to Root/Graft; stays available as flavor/side content rather than the act's spine |
| Cold-open shape | **Answer** — "the silence breaks" as the whole act, not just its opening beat | Resolves one open question cleanly but doesn't answer what the player spends hours doing; demoted to the cold-open beat inside The Rooting |
| Cadence | Wall-clock / decision-dense, extending Act 2's contract | Lost to idle-first — Act 3 swings back toward Act 1 rather than extending Act 2 |
| Growth track | New branch on the existing Haven tree | Lost to a wholly new system (the Grove) — signals new territory over reusing a familiar UI |
| Loom tie-in | Root Zones/Grove plug directly into the existing Loom (28 patterns) | Lost — Loom stays fully untouched, its own island |
| Wyrd's role | Pure narrative-only currency, or stockpiled unspent toward Act 4 | Lost to "a modest permanent lever" — gives Wyrd a power role now rather than staying purely cosmetic or deferred |
| Root Zones shape | Infinite tail, Expanse-style | Lost to a finite, capped band |
| Ramp trigger | A numeric Grove-population or Wyrd-threshold gate | Lost to a narrative beat — softer, more story-driven than every other unlock in the game |
| Enemies | Reskinned existing archetypes (cheaper, Loom-zone-style scaling) | Lost to a purpose-built decay/rot bestiary — reinforces the "defending new growth" framing |

## What this exploration is not

- Not a proposal. No `openspec/changes/` artifact exists for this yet, and
  none should until someone is ready to build it.
- Not a numbers pass. No constants, gates, or currency values are
  specified — that's `/opsx:propose` work once building starts.
- Not final. "Crystallized" here means a concrete, internally-consistent
  direction emerged from working through the options, not that it's locked —
  the alternatives above are worth revisiting if playtesting or further
  design work changes the picture.

## Suggested next step

This direction is concrete enough now to fold a trimmed version into
`world-and-narrative.md`'s "arc across acts" table and "Open narrative
questions" section (replacing "TBD" with "The Rooting — Root/Graft," clearly
marked as unshipped intent, the same way Act 2's own design doc carried
three superseded generations of the Ferryman design before any of them
shipped). Actual mechanical design — the Grove's specific rooms/upgrades,
Wyrd's exact bonus, the Root Zones' capstone and enemy roster, the cold-open
scene's actual text — is `/opsx:propose` work, and should wait until someone
actively wants to start building Act 3. Act 4 stays open for a future pass.

## History

- **2026-07-05, initial pass**: broad brainstorm across four Act 3 verb
  candidates and a Ragnarök-flavored Act 4 sketch, written up as an
  exploration with a recommendation but no decisions made.
- **2026-07-05, interactive pass**: worked through the direction via a
  round of forced-choice questions rather than a single write-up —
  Root/Graft chosen over Weave/Tend/Search; idle-first cadence chosen over
  wall-clock; the three-axis core loop (Grove/Root Zones/Wyrd) chosen over
  a single-system approach; Loom left untouched; Wyrd scoped to a modest
  permanent lever; the Grove made a new system rather than a Haven branch;
  Root Zones made finite and decay/rot-themed; the ramp into Root Zones
  made narrative rather than numeric. Act 4 explicitly left undecided.
  This revision reflects that pass's outcome.
