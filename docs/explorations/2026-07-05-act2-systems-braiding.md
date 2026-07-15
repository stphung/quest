# Exploring: deepening Act 2 with more of Act 1's systems

**Mode**: `/opsx:explore` | **Status**: direction solid through session 6 —
see "Session 5: correction" for the internal system-of-systems braid and
"Session 6: Dock, Wormhole, Return Crossing" for the concrete loop shape it
lives in. Sessions 2-4 below (legacy-as-flavor) answered a related but
different question and are kept as a parked, optional layer, not the main
thread. Nothing proposed yet.
**Prompted by**: "I loved how Act 1 was a system of systems that came together
to get us to Act 2. I'd like to explore other systems and how they could tie
into deepening Act 2 beyond what it is today."

## What "system of systems" actually meant in Act 1

Act 1 isn't one loop with side quests bolted on — it's a dozen systems that
keep pointing at each other and at two shared currencies (PR, and later WR):

```
   Combat/Zones ──XP──► Level ──► Ascension (PR)
        │                              ▲
        ├──drops──► Items ──► Enhancement (Soulforge)
        │                              ▲
        ├──unlocks──► Deep ──► Power Cores ──PR──┘
        │                │
        │                └──► Fracture Zones (harder combat)
        ├──unlocks──► Loom ──► Woven Patterns ──► Ascension VII-X gate
        ├──unlocks──► Haven (account-wide passive bonuses, forever)
        ├──unlocks──► Challenges, Fishing, God Items
        └──tracked by──► Achievements (cross-cutting, everything)

   All of it braids into ONE gate:
   28 patterns + Ascension X + 250,000 PR + Zone 50 kill ──► LAUNCH
```

That convergence is genuinely the best trick Act 1 does: nothing is an
island, everything is legible to the same two currencies, and the finale
asks you to prove it by spending all of it at once.

## Where Act 2 stands today

Per `docs/dossiers/act2-pilgrimage.md`'s own Fun Assessment (heuristic #4,
"Cross-system braiding: 3/5"):

> "The launch gate braids all of Act 1 into the burn (excellent); the
> voyage itself remains a deliberate island... **Confirmed intentional, not
> a gap.**"

And from Interrelations:

> "During: zero mechanical interaction with Act 1 — deliberate ('one-way
> passage'); Act 1 idles untouched beneath."
>
> "The ferry loop is the act's second major cross-system braid, but an
> internal one... a closed loop with **no Act-1-style external inputs**."

So today's shape is: one big braid (the launch), then total severance for
everything after — both the maiden voyage *and* the recurring Ferryman era
that follows it, which can run 3–5 real months.

```
Act 1 (all systems) ──►[ONE BRAID: the launch burn]──► Act 2 (sealed island)
                                                          │
                                              Voyage, then Ferryman loop,
                                              running for months on its
                                              own internal economy
                                              (provisions/Salvage/yards)
                                              with nothing crossing back in
```

That's a deliberate design choice ("one-way passage," severance from home
as the emotional beat), not an oversight. Worth sitting with before
reaching for a fix: **the thing you're nostalgic for and the thing Act 2
currently *is* are in real tension.** Any deepening here is either honoring
the severance while finding new seams to braid at, or knowingly punching a
hole in a wall the designers built on purpose.

## The actual opportunity, underneath the nostalgia

There's a concrete problem hiding behind the "I want more braiding"
feeling, and it's not just aesthetic: **once a player launches the Vessel,
most of Act 1's growth systems are already maxed.** Ascension X is the
level cap. 28/28 patterns is Loom's completion state. The two hardest gates
in the game are, definitionally, *finished* the moment Act 2 opens.

> **⚠️ CORRECTION (2026-07-14) — the paragraph below is factually wrong. Do
> not build on it.**
>
> Act 1 does **not** keep running during Act 2. `main.rs:555` takes the voyage
> branch on `vessel_launched && act2_enabled()` and `main.rs:1018` `continue`s,
> while both `game_tick_with_context` call sites sit at `main.rs:1253` / `:1608`
> — *after* it. Zones don't tick, Power Cores don't generate, Haven doesn't
> build, nothing is gettable. Act 1 is **frozen**, and permanently so
> (`vessel_launched` is only ever set `true`). Confirmed intended design,
> 2026-07-14.
>
> **What this voids:** there is no stranded output to redirect, so the "second
> outlet" framing below — and the "Act 1 bridges" Phase 2 item it justified
> (#734) — has no premise. Any such bridge is *new content*, not reuse, and
> must be re-argued from scratch on its own merits.
>
> **What survives:** the opening observation still holds — Ascension X and 28/28
> patterns *are* maxed at launch. But they're not "stranded and still
> producing"; they're switched off. The real question this exploration should
> have asked: **is it right that the hero a player built over hundreds of hours
> goes inert for a 3–7 month era?** That is a live design question, unanswered.
>
> Preserved verbatim below as the record of how the item came to be proposed.

So a player deep in a 3–5-month Ferryman era who keeps playing Act 1 in
parallel (zones still tick, Power Cores still generate PR, Haven rooms
still build, Fishing/Challenges/Achievements are all still gettable) is
generating output that has **nowhere left to go.** Act 1 isn't idle by
design so much as *stranded* — it kept running, but its only outlet
(the launch gate) already closed behind the player. That's the gap worth
naming distinctly from "I miss the vibes": **Act 1 systems have no second
outlet once their first one is spent.**

That reframes the question from "how do we add more crossover" to "what's
the second thing all these systems get to point at, now that the first
thing is done?"

## Two shapes this could take

```
                       ┌─────────────────────────────────┐
                       │   A. SEAM-ONLY BRAIDING          │
                       │   (respect the island)           │
                       ├─────────────────────────────────┤
                       │  Braid harder BEFORE launch and  │
                       │  AFTER the era ends. Voyage/     │
                       │  Ferryman interior stays sealed,  │
                       │  exactly as designed.             │
                       └─────────────────────────────────┘

                       ┌─────────────────────────────────┐
                       │   B. PUNCTURE THE FERRYMAN ERA    │
                       │   (extend the braid)              │
                       ├─────────────────────────────────┤
                       │  The Ferryman loop (not the       │
                       │  maiden voyage) becomes a second   │
                       │  convergence point Act 1 systems  │
                       │  keep feeding, the way Power Cores │
                       │  feed PR feeds Ascension today.    │
                       └─────────────────────────────────┘
```

Both are legitimate; they're not really in competition, they answer
different halves of the prompt. A: keeps every current pillar untouched
and gives the parts of Act 1 that go stale post-launch somewhere to still
matter. B: is the more literal answer to "I want that system-of-systems
feeling to keep going," but it's a considered reversal of the "voyage
severed from Act 1" pillar, not a free lunch — it needs a deliberate design
call, not a quiet code change.

> **Correction (session 5)**: sessions 2 through 4 below pursued
> "legacy-as-flavor" — Act 1 stats read-only-flavoring Act 2's new
> dimensions. That was a misread of the prompt. The actual ask was **Act 2
> systems feeding Act 2 systems**, self-contained, the way Act 1's systems
> fed each other — not Act 1 reaching into Act 2 at all, even softly. See
> "Session 5: correction" at the end of this document for the corrected
> direction. Sessions 2-4 are kept below, unedited, as a record of the
> wrong turn and because the two dimensions they named (ship leveling,
> districts) turned out to still be the right *nouns* — just braided to
> each other now, not to Act 1.

## Where the conversation landed (session 2)

Live discussion narrowed the two shapes above considerably. Two clarifying
answers reframed the whole question:

1. **"I think it's cool if an act has more dimensions than just one loop."**
   Looking back at the Ferryman era with that lens: it isn't really a system
   of systems today, it's **one currency, three sinks** —

   ```
   sail ──► Salvage ──► spend on { Drive | Shipwright | Ward } ──► repeat
   ```

   Districts and world milestones are discovery axes (things that *unlock*),
   not systems with their own internal choices. So the honest gap isn't
   "Act 1 doesn't reach into Act 2" — it's that **Act 2's own recurring era
   only has one dimension to grow**, where Act 1 had several running in
   parallel (combat, items, Deep, Loom, Haven, Ascension all leveling
   independently, occasionally gating each other).

2. **"Continued progress" means a sense that what you did before is carrying
   over — it doesn't have to come over 1-1 systemwise.** Not an ongoing
   resource pipe (Shape B, Power Cores literally feeding Salvage forever).
   Something closer to what the *launch gate itself* already does — Act 1
   converges once into the burn — but expressed as **flavor and starting
   conditions** across new dimensions, rather than one lump PR number.

Put together: the fix isn't "reopen Act 1 as an input," it's **give the
Ferryman era a second and third dimension of its own**, and let a player's
Act 1 history shape *how those new dimensions start*, not feed them forever.

```
                    ONE-TIME REFLECTION (not an ongoing feed)
                    ═══════════════════════════════════════

  Act 1 legacy                          Act 2's two new dimensions
  ───────────────                       ───────────────────────────

  Soulforge enhancement ─┐
  God Items owned ───────┼──flavors──►  THE SHIP LEVELS UP
  Haven tier/rooms ──────┘              (Hold/Ward/Drive become slots with
                                          their own tiers/mastery, not just
                                          Salvage sinks — starting tier or
                                          a named variant reflects what you
                                          brought to the burn)

  Haven room mix ────────┐
  Fishing rank ──────────┼──flavors──►  DISTRICTS AS A BUILD TREE
  God Items owned ───────┘              (each founded district offers 2-3
                                          named passive choices instead of
                                          unlocking automatically — which
                                          options are on offer can reflect
                                          Act 1 history)

                    Ongoing growth (Salvage buying further levels/choices)
                    stays 100% Act-2-internal — the "sealed era" pillar for
                    ongoing mechanics is untouched; only the *opening state*
                    of these two new dimensions carries a legacy imprint.
```

Of the candidate new dimensions raised in this session, **the ship leveling
up** and **districts as a build-out tree** were picked as the ones worth
fleshing out first (over a refinement chain and per-soul veteran ranks,
which remain on the table for later). Both are Shape-A-compatible — no
change to what happens *during* a crossing — and both give the ferry era a
second and third thing to grow besides the Salvage/3-yards economy.

### The four legacy signals (session 3)

Filtered against a three-part test — a signal must (1) actually **vary**
among players who already cleared the same launch gate (anything that's a
floor everyone shares, like Ascension X or 28/28 patterns, is the gate
itself, not a legacy signal), (2) be **legible at a glance** (a count, an
owned/not-owned flag, a title — not a buried formula), and (3) read as
**something chosen**, not an incidental byproduct of grinding to the gate.
That test eliminates Ascension level, Loom patterns, prestige rank,
Stormbreaker ownership, and Zone reached outright (all launch-gate floors),
and holds Deep-layer-reached and Fishing rank/Storm-Leviathan out of the
*core* set for now — Deep already has its own narrative thread into Act 2
(Torvald), and Fishing is real but thin, better as a later light-touch
flavor option than a load-bearing signal. What survives, each mapped to
exactly one destination so nothing competes for two slots:

```
Soulforge total enhancement (0-70) ──► Ship: starting Ward tier
God Items owned (0-3)              ──► Ship: named passive per yard
                                         (Sleipnir→Drive, Megingjörð→Ward,
                                         Asprika→Hold)
Haven room count/tier              ──► Districts: extra option(s) on offer
Achievement score/title            ──► Districts: one rare "Founder's"
                                         district, gated by a specific
                                         achievement/title (not an
                                         aggregate score threshold — see
                                         session 4)
```

**Design principle for all four: read-only, not converted.** None of these
should be *spent* or consumed by Act 2 — Act 2 only ever queries "how much
Soulforge enhancement does this character have," "which God Items does
this account own," etc., the same stats exactly as they exist for Act 1's
own purposes. That keeps the door open for Act 3 (or any future act) to
independently reference the same untouched stats for its own flavor later,
rather than finding them already used up by Act 2's read of them —
consistent with `world-and-narrative.md`'s open question about whether the
colony becomes Act 3's home base at all.

### Concrete sketches (session 4)

A worked example of the Ward yard as named tiers over the same underlying
cost curve — a skin, not a rebalance:

| Tier | Name | Cost curve | Vignette (fires once, on tier-up) |
|---|---|---|---|
| 0 | Patched Hull | starting (or 1 free tier via Soulforge legacy, below) | — |
| 1 | Warded Hull | today's Ward level 1 (`WARD_DECAY = 0.72`) | "The seams hold. For now." |
| 2 | Bound Hull | today's level 2 | "Something in the wood remembers being whole." |
| 3 | Blessed Hull | today's level 3 | "The dark's bite glances, doesn't land." |
| 4 (cap) | Unbreakable | today's level 4/max | "Nothing left to fear from the crossing itself." |

The Soulforge legacy signal (total enhancement across 7 slots past a
midpoint threshold, e.g. 35/70) grants exactly one free starting tier —
launch at "Warded Hull" instead of "Patched Hull" — small and legible,
doesn't touch the tuned strategy-sweep numbers past that first step.

God Item passives, sketched as small flat modifiers layered on a yard
(shown as a named badge on the Reckoning row, not a hidden number) rather
than new tiers of their own:

| Item | Yard | Passive (rough magnitude) |
|---|---|---|
| Sleipnir | Drive | −5% days per leg |
| Megingjörð | Ward | −5% dark toll rate |
| Asprika | Shipwright (Hold) | +5% hold capacity |

District example (District 2, illustrative names):

```
District founded ──► choose one:
  "Warders' Court"  (small Ward efficiency, permanent)
  "Shipwrights' Yard"  (small Hold efficiency, permanent)
  "Founder's Hearth"  (only offered if a specific Act 1 title/achievement
                        is held — not an aggregate score threshold, see
                        below — unique passive + flavor text naming the
                        player's own Act 1 legend)
```

**Founder's Hearth gating, reconsidered**: not an aggregate Achievement
*score* threshold (too abstract, another buried formula, fails the
legibility test the four signals were held to). Should instead key off one
or a small named set of specific existing achievements/titles — e.g. a
specific top-tier title already granted by the Achievements system — so
the option's flavor text can name the exact thing the player did, not "you
scored high enough." **Which specific title(s) qualify is still open** —
needs a look at what titles the Achievements system actually grants today
before picking one, rather than inventing a new threshold from scratch.

**Still open, for a future session**: which specific Achievement title(s)
gate Founder's Hearth, the full Haven-room → district-option mapping,
whether district picks should be permanent (mirroring the Voyage's
refit-door pattern) or revisable, and named tiers for Drive/Shipwright to
match Ward's worked example above.

## Per-system sketches

| Act 1 system | Current post-launch fate | Braid idea | Shape |
|---|---|---|---|
| **Power Cores** | Keeps generating PR, which has no further use once the 250k burn is behind you | A fraction of Power Core output converts to Salvage during the ferry era — mirrors the existing WR→PR conversion pattern in Loom, gives a "done" system a second currency to feed | B |
| **Haven** | Account-level, permanent, keeps building | Specific rooms grant small passive Voyage/Reckoning bonuses (cheaper provisions burn, faster Drive) — "everything I built at home makes the ship faster," the exact callback the launch gate already does once | A or B |
| **Soulforge / Enhancement** | Equipment enhancement, nothing left to enhance once you've capped your combat kit | Reuse the same +0→+10 mechanic on the *ship's* components (Hold/Ward/Drive slots) instead of armor slots — new noun, same loved mechanic, and it already has a narrative peg (hull wear, the hidden "mend the hull" refit option) | B |
| **Deep** | Its own generational, wall-clock, away-safe mission system — structurally the closest cousin to the Voyage already, and Torvald is already a narrative callback | Deep missions framed as supply runs that feed Salvage/provisions into the active crossing — two wall-clock systems that already resemble each other literally feeding one another | B |
| **Loom** | Caps at 28/28 patterns for the Ascension gate; nothing left to weave | New patterns unlockable *only* during the ferry era, feeding Colony yards or new refit unlocks directly — gives Loom a reason to keep growing past "done" | B |
| **God Items** | Three Norse-mythology artifacts with unique passives; already thematically primed (Sleipnir literally crosses between worlds) | Each God Item unlocks a specific Voyage effect — Sleipnir shaves Drive time, Megingjörð resists hull wear, Asprika does something to provisions or Ward. Best thematic fit on this whole list | A or B |
| **Achievements** | Tracks combat/leveling/prestige/zones/challenges/fishing/dungeons/Haven — **not Vessel at all today** | Straightforward, low-risk: add Vessel/Colony milestones (crossings completed, % world saved, Last Crossing reached) to the existing cross-cutting tracker. This is a real integration gap, not a design question | A (closes a gap either way) |
| **Challenges** | Player-triggered skill minigames | The three Threats (Ossuary Warden/Silence/Thorns) are currently resolved by crew placement + pace only, no skill test — an optional Challenge-style minigame at a Threat road could let a player *beat* the toll through skill rather than loadout alone, without adding dice (still deterministic pass/fail on player performance) | B, careful — touches the "no dice" pillar, must stay skill-not-chance |
| **Fishing** | 40-rank separate progression track, thematically already "the sea" | Minor "fishing off the stern" during a crossing yields trivial provisions/Salvage trickle — cheap flavor, low mechanical weight, honors "traveling is not dead time" | B, light-touch |
| **History / Time Vault** | Git-commits every meaningful `SaveEvent`; **zero Vessel-specific event variants exist today** | Add `SaveEvent` variants for launch, arrival, world milestones, Last Crossing — lets players browse/restore/fork their Act 2 journey the same way they can for everything else. Pure integration completeness, no new mechanic, no pillar conflict | A (closes a gap either way) |
| **Stormglass** | Character-level currency, persists across prestige, already the game's throughline Storm motif (Sigils, Leviathan, Stormbreaker) | Spend Stormglass currency to ward off a squall during the Voyage — echoes Stormbreaker already being a hard Zone 10 gate; the Voyage's Weather system is pure-function-of-seed today, so this would need a currency-buys-a-mitigation hook, not raw RNG-fighting | B |

## Two sketches worth going deeper on

**Achievements + History are the free wins.** Both are pure integration gaps
(confirmed by the research pass: Achievements doesn't track Vessel at all,
`SaveEvent` has no Vessel variants), not design questions. Nothing about
"the voyage is an island" argues against tracking milestones or committing
save history for it — those are meta-systems that already reach into every
other system without touching gameplay mechanics. These could graduate to
an `/opsx:propose` almost immediately, independent of resolving the A vs. B
question above.

**God Items are the best-fitting Shape-A idea.** They're the one system on
this list that's *already* thematically Norse-endgame-artifact shaped, and
"an artifact matters aboard the Vessel" doesn't require puncturing the
severance pillar at all — it's just three more inputs alongside the
existing launch-gate braid, or three small always-on passive bonuses that
read as "what you earned in Act 1 rides with you," which is different from
"Act 1 keeps mechanically feeding Act 2 forever." Low risk, high thematic
payoff, cheapest place to start if the goal is "make the launch feel like
an even bigger braid" without opening the harder Shape-B question.

**Power Cores + Deep are the most interesting Shape-B idea**, because they
don't just add a bonus — they answer the "stranded output" problem
directly. Both systems generate ongoing output (PR, expedition loot) that
currently has nothing left to buy. Routing a fraction of that output into
Salvage during the ferry era is the closest mechanical analog to how Act 1
itself works (many sources, one shared sink) and would be the single
change that most literally recreates the "system of systems" feeling for
the post-launch game, at the cost of revisiting a stated design pillar.

## Open questions (not resolved, this is still exploration)

1. **Is "the voyage is severed from Act 1" a pillar worth keeping absolute,
   or was it right for the *maiden* voyage specifically and over-applied to
   the recurring Ferryman era too?** The dossier states both are islands
   today; splitting them (maiden voyage stays sealed, Ferryman era gets a
   trickle-in from Act 1) might resolve the tension without touching the
   act's single best "severance" beat — the first crossing.
2. **If Shape B is chosen, does that make the endgame feel busier without
   feeling deeper?** Act 2's Fun Assessment already flags decision density
   as the era's weak spot (ferry runs are "one choice per ~3 real days");
   piping in more currencies doesn't obviously fix that unless it creates
   new decisions, not just new number-go-up.
3. **Which of these should be additive-only vs. gated?** E.g., should God
   Item Voyage bonuses be automatic once owned, or should equipping them
   for the crossing cost something (mirroring the Vessel's other
   all-or-nothing choices)?
4. **Scope for Act 3.** `world-and-narrative.md` already flags "does the
   colony become Act 3's home base?" as an open narrative question — any
   Shape-B braid built into the Ferryman era should be checked against
   whatever Act 3 eventually wants that resource loop to become, so this
   doesn't get re-litigated twice.

## Where this could go next

Nothing here is decided. If a direction resonates, the natural next steps:

- **Achievements + History (Time Vault) integration** — small enough to
  `/opsx:propose` directly, no open design question blocking it.
- **God Items → Voyage passives** — needs a short design pass (what exactly
  does each of the 3 items do?) but doesn't require resolving the Shape A/B
  question first.
- **Power Cores / Deep → Salvage bridge** — the highest-payoff, highest-risk
  idea; worth a dedicated `/opsx:explore` session with the designer to
  settle the severance-pillar question before it becomes a proposal.

## Session 5: correction — Act 2 feeding Act 2, not Act 1 feeding Act 2

**"I didn't mean that act 1 systems feed act 2. I meant there are act 2
systems that feed act 2 similar to act 1."**

Everything above from session 2 onward pursued the wrong shape. The actual
ask is structural, not thematic: Act 1's braid isn't "Act 1 feeds Act 2,"
it's that *combat feeds items feeds Soulforge, Deep feeds Power Cores feeds
PR feeds Ascension, Loom feeds Ascension's top tiers, Haven bonuses touch
everything* — many systems, entirely internal to Act 1, gating and feeding
each other, converging only once (the launch burn) as a side effect of all
that internal activity. Act 2's Ferryman era has no internal braid at all
today — it's one pipe (crossings → Salvage → three sinks that don't touch
each other). The fix is to give Act 2 that same internal structure,
self-contained, using no Act 1 stat as an input.

The two dimensions picked in session 2 — **the ship levels up** and
**districts as a build-out tree** — are still the right nouns. What was
missing is a third and fourth piece, and real gating between all of them
instead of each just being a second/third Salvage sink:

```
                    ┌──────────────────────────────────────┐
                    │         THE CORE LOOP (crossings)      │
                    │   sail ──► raw Salvage + Soul time-    │
                    │   in-station                            │
                    └───────────────┬────────────────────────┘
                                    │
                 ┌──────────────────┼──────────────────┐
                 ▼                  ▼                  ▼
          VETERAN SOULS      RAW SALVAGE          (station time
          rank up at a       spendable now,        accrues toward
          station over       as today, on          veteran rank)
          many crossings     basic yard tiers
                 │                  │
                 │                  ▼
                 │           REFINEMENT
                 │           (needs a Refinery district built)
                 │           raw Salvage ──► Star-Cord (Drive) /
                 │                           Ironbound Timber (Ward) /
                 │                           Broadwood (Hold)
                 │                  │
                 ▼                  ▼
         unlock a unique    refined materials required for
         named ship tier    the TOP ship tiers — raw Salvage
         or district        alone hard-caps mid-tier (mirrors
         option tied to     Loom feeding Ascension VII-X exactly)
         that soul's arc            │
                 │                  ▼
                 └────────────►  SHIP LEVELS UP  ◄─────────┐
                                    │                        │
                                    ▼                        │
                          faster/safer crossings ────────────┘
                          (more Salvage + soul-time per
                          real-week — feeds the whole loop
                          again, Act 1's "wall → reset →
                          power" shape)

         Meanwhile: DISTRICTS need both raw Salvage AND a
         minimum Ship tier to found — and Refineries (one
         district type) are the only way to unlock refinement
         at all — so Ship and Districts gate each other in
         both directions, not just both drink from Salvage.
```

Four pieces, each doing a distinct job (mirroring Act 1's shape of several
systems with distinct roles, not several skins on one economy):

1. **The core loop (crossings)** — unchanged from today, produces raw
   Salvage and (new) soul time-in-station.
2. **Veteran souls** — crew stationed at Helm/Tender/Watch across *many*
   crossings (distinct from the maiden voyage's one-time 3-beat arcs)
   accrue a veteran rank; reaching top rank at a station unlocks a unique
   named ship tier or district option tied to that soul specifically — a
   per-character growth axis feeding the shared systems, the way Ascension
   is per-character but feeds access to shared Deep/Loom content in Act 1.
3. **Refinement** — a production step, not a system unto itself: raw
   Salvage becomes one of three named materials (Star-Cord, Ironbound
   Timber, Broadwood — placeholder names), but only once a Refinery
   district exists. This is deliberately Loom-shaped (a small conversion
   chain) rather than Soulforge-shaped (a direct upgrade).
4. **Ship levels up** and **Districts** — as named in session 2, but now
   mutually gating: raw Salvage alone caps ship tiers partway (a **hard
   gate**, confirmed in this session — mirrors Loom→Ascension VII-X
   exactly rather than being a soft efficiency bonus); districts require a
   minimum ship tier to found at all (a slower ship can't hold the site);
   only a Refinery district unlocks the refined materials the top ship
   tiers require.

**Resolved this session**: the Refinery gate is a hard requirement for top
ship tiers, not a soft accelerant — the stronger, more Act-1-shaped braid,
accepted with the tradeoff that it puts more pressure on the tuned
strategy-sweep balance (see Balance Evidence in `act2-pilgrimage.md`) and
will need re-validation once numbers exist.

**The session 2-4 "legacy-as-flavor" thread is not discarded, just
demoted**: God Items as Voyage artifacts, and Achievements/History tracking
Vessel milestones, are both still perfectly good ideas — they just answer
"does Act 1 get *recognized* in Act 2," a smaller and separate question
from "does Act 2 have its own internal engine." Worth revisiting as an
additive layer once this braid has a real shape, not as competition for it.

**Open questions, this session**:
- Does Refinement need all three named materials from the start, or does
  the Refinery unlock them one at a time (its own small progression)?
- Should veteran-soul rank-up unlock a ship tier, a district option, or
  either depending on which station the soul held — i.e. does a Helm
  veteran always feed the ship and a Tender veteran always feed districts,
  or is it soul-specific and authored per character?
- The maiden voyage's souls (8 named, 7 berths) are fully authored,
  one-loss-only content — does "veteran rank" apply to those same 8 souls
  across the whole ferry era, or does the era need new, ferry-only crew
  who were never part of the maiden voyage's authored arcs?
- Numeric shape of the hard gate (how far does raw Salvage get you before
  it caps, and how much does a Refinery need to produce to close the gap)
  is unset — needs an actual pass against the strategy-sweep methodology
  before this could become a proposal.

## Session 6: Dock, Wormhole, Return Crossing — a concrete loop shape

**"I would like to build in an idea where we need time at destination
refueling to then travel through a one way wormhole to start the next
voyage. That would give some opportunities for game systems."**

This gives session 5's abstract braid an actual place to live in the game
loop, and explains structurally why the transit should be one-way rather
than round-trip: the ship punches through a wormhole *fast* to the far
side, then sails the *real*, meaningful distance back using all of the
Voyage's existing machinery (route, phases, pace, weather, threats) — only
the return leg needs to feel like a journey; the outbound hop is the
commit moment.

```
     DOCK (at the Colony)              WORMHOLE                RETURN CROSSING
     ══════════════════               ══════════               (existing Voyage
                                                                  machinery, as-is)
  refuel + build, real time    ──►   one-way, committed,  ──►  sail back through
  passes, active management           no undo once you           the dark, generate
                                       jump                       Salvage, face the
  • repair hull wear /                                           dark's toll, arrive
    clear soul Strain                                             back at the Colony
  • found/choose Districts
  • Refinery converts raw                    │
    Salvage → materials                      │
  • spend materials/Salvage                  ▼
    on Ship tiers                    arrive at the far
  • rotate/level veteran            side (old world/
    crew                             frontier)
       │                                     │
       └─────────────◄───────────────────────┘
              (arrival back at Colony reopens Dock)
```

**Scope, resolved this session**: applies only from the first arrival
onward (ferry runs 2+). The maiden voyage's outbound leg — the rich,
decision-dense, one-time crossing with souls, refit doors, threats, and
letters — is untouched. Dock/Wormhole is specifically the answer to what
"Sail Again" currently glosses over as hands-off automation; it doesn't
touch the act's most-tested, most-authored content.

**Riftglass** (placeholder name) — a new resource, distinct from the
session-5 ship-building materials (Star-Cord/Ironbound Timber/Broadwood),
that charges the wormhole. It accumulates purely from *time spent docked*,
the same shape as provisions accumulating from time spent sailing — not a
recipe, not gated behind a specific district on its own. Districts and Ship
tiers can modify the *rate* (a district built for it, or a high Drive tier,
charges the rift faster), which is what makes Dock-phase investment matter
for pacing rather than Dock time being pure downtime with a wait bar
attached.

**Resolved this session: charging is a partial-charge tradeoff, not a
simple wait-until-full gauge.** Leaving on a partial charge should mean a
genuinely riskier or costlier return crossing, not just "the same crossing,
started sooner" — candidate shapes for what "riskier" means (none decided,
first pass at options):
- Landing off-course — the return crossing starts further back on the
  route DAG, adding real days.
- Landing with a provisions deficit, or with hull wear already applied
  before the first league is sailed.
- A guaranteed minor threat or worse starting weather roll on arrival.

Full charge lands cleanly at the standard frontier point in full order —
the safe, patient option; partial charge trades real Dock time now for a
harder return crossing later. This is the piece meant to directly answer
the Fun Assessment's flagged weak spot ("ferry runs: one choice per ~3 real
days") — Dock time stops being a shopping screen and becomes an actual
management phase with a real timing decision at the end of it.

**Open questions, this session**:
- Which of the "riskier jump" candidates above (or a combination) actually
  gets built — needs to stay consistent with "no dice anywhere": whatever
  a partial charge produces should be a deterministic function of charge
  level, not a die roll dressed up as risk.
- Does Riftglass decay if you overcharge and wait past full (a soft
  pressure not to over-invest in Dock time), or does it simply cap?
- Should the *rate* modifiers (which Districts/Ship tiers speed up
  charging) be the same infrastructure as session 5's ship/district
  mutual-gating braid, or a separate axis layered on top? Likely the
  former — reusing one braid rather than building two parallel ones — but
  not yet checked against the numeric shape of that gate.
- Whether the old-world-decline world-milestones axis should affect
  wormhole stability/cost as the era progresses (an easier or harder jump
  as the dying world empties) — an interesting tie between two existing
  discovery axes, not yet explored.
