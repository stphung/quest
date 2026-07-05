# Exploring: the story arc for Act 3 and Act 4

**Status**: pre-commitment exploration. Nothing here is decided; nothing here
should be implemented. When a direction crystallizes, graduate it into
`docs/dossiers/world-and-narrative.md` (the "arc across acts" table) and then
into an `/opsx:propose` change for whatever ships first.

**Prompted by**: `world-and-narrative.md`'s own "Open narrative questions" list
(Act 3/4 rows are "TBD" / "Not designed"), plus three concrete hooks Act 2
already built for whoever answers them — `vessel_arrived`,
`last_crossing_complete`, and the Sister Verity.

## Where the story actually stands right now

Nothing here is invented — it's what's already shipped (dark) and waiting:

- **`vessel_arrived`** fires at first landfall. The normative spec calls this
  "the durable hook a future Act 3 keys off" — but it's the *shallow* hook.
- **`last_crossing_complete`** fires only once the old world is fully
  emptied and the Ferryman era ends. `colony.rs`'s own module doc calls this
  one outright: *"the next arrival is the Last Crossing: Act 3's gate."* This
  is the deep hook, and the one that actually gates a next act, because it's
  the point where the ferry loop stops being repeatable and the game has
  nothing left to offer but "what's next."
- **The Sister Verity** is a named pilgrim ship, authored to reach the Tree
  and *wait there* rather than going dark like the other four pilgrims. She's
  a face already standing at the harbor, with no scene to spend her on yet.
- **Letters Home** thin out and stop as the Vessel leaves the shore — a
  "going dark" device explicitly flagged in `world-and-narrative.md` as
  reusable, with an open question attached: does it ever pay off?

So the game already has, sitting unused: an empty gate flag, a person waiting
at a door, and a silence that's never been broken. That's a lot of loaded gun
on the mantelpiece for an "undesigned" act.

## The pattern so far, and what's left of it

| Act | Verb | Shape |
|---|---|---|
| 1 — Ascent | **Climb** | up 50 zones |
| (Deep, inside Act 1) | **Descend** | down numbered Layers |
| 2 — Crossing | **Cross / ferry** | outward, across the dark, repeatable until the well runs dry |
| 3 | ? | — |
| 4 | ? | — |

Each act has so far also *deliberately broken* the previous act's design
pattern rather than extending it (Act 2's Fun Assessment dossier section says
this outright: "confirm, don't fix"). Act 1 is idle-first, failure-free,
numbers-only. Act 2 is wall-clock, consequence-bearing, anticipation-first,
one-way. If Act 3 just extends Act 2's shape (another ferry loop, another
Reckoning), it repeats a pattern instead of reframing the hero against a
larger scale the way both prior acts did. The interesting design question
isn't "what's the next resource to manage" — it's **what's the next verb**,
and what does *that* act deliberately keep or break.

Up, down, and across are spent. What's left in that geometry?

```
        UP (Act 1)
         │
         │
  ACROSS ─●─ ACROSS (Act 2, the dark between branches)
         │
         │
        DOWN (the Deep, inside Act 1)
```

The unclaimed vector is **in** — not a direction across the map, but into the
thing the map has been circling since the title screen: Yggdrasil itself.
Every currency name (the Loom *of Worlds*, Ascension, the Storm) has been
gesturing at Norse cosmology without cashing the check. Act 3 is where the
tree stops being a destination on the horizon and becomes the place the
story happens *inside*.

## Candidate Act 3 verbs

### A. Root — descend into Yggdrasil, arrival becomes threshold, not destination

The Tree isn't the end of the crossing, it's the start of a second one — down
into its roots, where (per the actual myth) the Norns sit at the Well of
Urd, spinning fate. This directly answers "what is at the Tree": not a
place to settle, but a deeper structure to enter. It also gives the Loom's
name retroactive weight — the Loom of Worlds was never just a resource
network, it was foreshadowing the actual loom the Norns work at.

```
        Act 2 ends: colony founded, thousands of souls, old world emptied
                              │
                              ▼
                    vessel_arrived (shallow gate)
                              │
                    last_crossing_complete (deep gate) ── Act 3 opens
                              │
                              ▼
              ┌───────────────────────────────┐
              │   ROOT: descend into the Tree  │
              │   the colony needs grafting     │
              │   onto a living branch to live   │
              └───────────────────────────────┘
```

- **Verb**: Root / Graft. The colony (Act 2's payoff) isn't a place, it's
  cargo still in transit — it needs a living branch to take hold on, and Act
  3 is finding and earning one.
- **Home base**: yes to the open question — the colony *becomes* Act 3's
  faction, but its stakes change from "get everyone here" (Act 2) to "make
  the graft take" (Act 3). Same population, new pressure.
- **Braid opportunity**: this is the act that could reconnect to Act 1's
  systems after Act 2 was deliberately an island. A living branch of
  Yggdrasil could literally *be* a reframed Zone map — new zones grown out of
  the graft, gated by how much of the colony has taken root, rather than by
  prestige rank. That would make Act 3 the first act to braid backward into
  Act 1's climb *and* forward into whatever Act 4 is.

### B. Weave — the Norns' Loom, fate as the resource

A variant of Root that leans harder into "fate" as the currency rather than
"land." The player doesn't just graft a branch, they sit in on the actual
weaving of what happens next — to the colony, to the dying world left behind,
maybe to the other pilgrim ships still out there. This is where the Sister
Verity earns her scene: she's not just flavor, she's a second thread already
spun, waiting to be rejoined to the player's.

- **Verb**: Weave / Thread.
- **Risk**: "fate as a resource" is more abstract than Act 1's tangible zones
  or Act 2's tangible souls-and-salvage — needs a concrete mechanical hook or
  it stays a vibe. (Compare: Act 2 avoided this trap by making "passage"
  concrete — provisions, Salvage, a route graph you can literally see.)

### C. Answer — the silence breaks, contact is the whole act

A smaller, gentler alternative: Act 3 is not a new physical geography at all,
it's the payoff of "going dark." The Letters Home thread finally gets a
reply — from the dying world behind, from the four pilgrim ships that went
dark, or from further up the Tree. The act is entirely about what comes back
across a silence the player assumed was permanent.

- **Verb**: Answer / Receive.
- This plays *against* type for an act (Act 1 and 2 both open with the hero
  going somewhere) — Act 3 could instead be the first act built around
  something arriving *at* the hero. That's a real tonal option, not just a
  smaller version of A/B.
- Best treated as a strong **beat inside** Act 3 (the reply itself, maybe the
  one that opens or closes the act) rather than the whole act's shape — it
  resolves one open question cleanly but doesn't answer "what does the
  player spend hours doing."

**Recommendation for Act 3**: **A (Root/Graft)**, with **C (the silence
breaking)** as the beat that opens it and **B's Norns framing** folded in as
color/why rather than the primary resource. Root gives Act 3 a concrete verb,
a mechanical hook (a branch to grow, gated content to unlock as it takes),
and a clean answer to three of the five open questions at once (Tree as
threshold, colony as faction, going-dark payoff as the cold open).

## Candidate Act 4: Ragnarök, played straight

This is less speculative than Act 3 — the whole game's vocabulary (Yggdrasil,
Asprika/Sleipnir/Megingjörð as *actual* Æsir gear, the Storm, "the endless")
has been building toward Norse eschatology without saying the word. Act 4 is
where that becomes text instead of subtext.

The myth itself hands the act a structure almost too neatly:

- **Ragnarök** — the gods' final battle, the world burned by Surtr, ends in
  the world's death.
- **Líf and Lífthrasir** — the two who survive, hidden inside Yggdrasil's
  wood, who reseed humanity in the world that comes after.

That second beat is *already the shape of Act 2's ending* — a colony carried
inside the Tree, the sole survivors of a dying world. Act 2 re-told this myth
without naming it. Act 4 could be the act that closes the loop: the colony
the player has spent two acts protecting turns out to *be* Líf and
Lífthrasir, and the "final battle" isn't fought to win in the conventional
sense — it's survived, the way the myth says it's survived, by a branch that
was rooted (Act 3) deep enough to outlast the fire.

- **Verb**: Burn / Endure. Where Acts 1-3 are all about reaching or building
  something, Act 4 is about holding a line while something ends around you.
- **A clean bookend**: if Act 4 ends with reseeding, the honest closing image
  is a *new* Zone 1 — not literally restarting the save, but the game's own
  vocabulary (prestige is already "death and return," per
  `world-and-narrative.md`'s own recurring-motifs list) gets to cash out as
  fiction instead of just mechanic. The hero's own rebirth cycle across
  hundreds of prestiges was foreshadowing this the entire time.
- **Open risk**: Ragnarök "played straight" risks being over-literal fan
  service if it's not grounded in *this* story's specific stakes (the
  colony, the Verity, whichever souls were lost on the Thorns in Act 2). The
  myth should supply structure, not replace the player's actual save-specific
  history.

## What this does to the open questions

| Question (from `world-and-narrative.md`) | Answer under Root → Ragnarök |
|---|---|
| What is at the Tree? | A threshold, not a destination — the roots, the Norns, a branch to graft onto |
| Does the colony become Act 3's home base/faction? | Yes — same population, reframed stakes (arrival → survival of the graft) |
| Does "going dark" pay off? | Yes, as Act 3's cold open — the silence breaks before anything else happens |
| How do Ragnarök beats map onto a final act? | Directly — Act 4 *is* Ragnarök, with the colony as the Líf/Lífthrasir survivors Act 2 already wrote without naming |
| What's Act 3's verb? | Root / Graft (in, not up/down/across) |

## What Act 3 should deliberately keep or break

Following the pattern both prior acts set (each act contrasts, not extends,
the one before):

- **Keep from Act 2**: authored-only consequence, no dice on anything that
  matters, "numbers are narrative" (a graft-percentage gate should feel as
  momentous as 250,000 PR or 28 patterns did).
- **Break from Act 2**: Act 2 was proudly an island (zero mechanical
  interaction with Act 1 systems). Act 3 rejoining Act 1's systems (new
  zones grown from the graft, gated by colony health rather than prestige)
  would be the first act to braid *backward*, which is itself a new pattern
  worth calling out as deliberate if it's the direction taken.
- **Undecided, worth flagging early**: does Act 3 stay idle-first (Act 1's
  contract) or wall-clock/decision-dense (Act 2's contract)? Root's "graft
  taking hold" framing could support either — a slow tick-driven growth meter
  (idle-first) or a check-in cadence like the Voyage (wall-clock). This is
  probably the single highest-leverage decision left unmade, since it decides
  whether Act 3 feels more like Act 1 or Act 2 to play.

## What this exploration is not

- Not a proposal. No `openspec/changes/` artifact exists for this yet, and
  none should until a direction here is chosen.
- Not a numbers pass. No constants, gates, or currencies are specified —
  that's `/opsx:propose` work once a verb is chosen.
- Not exhaustive. Other Act 3 verbs are surely possible (a "Tend" verb built
  entirely around the colony as a management sim; a "Search" verb built
  around the four pilgrim ships that *did* go dark). Root/Ragnarök is a
  recommendation grounded in what the game's vocabulary already implies, not
  the only coherent answer.

## Suggested next step

If this direction resonates: fold a trimmed version of "Root → Ragnarök"
into `world-and-narrative.md`'s "arc across acts" table and "Open narrative
questions" section (replacing "TBD"/open questions with the working
hypothesis, clearly marked as unshipped intent, the same way Act 2's own
design doc carried three superseded generations of the Ferryman design
before any of them shipped). Actual mechanical design — what "graft
percentage" even is, whether Act 3 is idle or wall-clock, what a Norns/Loom
tie-in looks like as a system — is `/opsx:propose` work, and should wait
until someone actively wants to start building Act 3.
