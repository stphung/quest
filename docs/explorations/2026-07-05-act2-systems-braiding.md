# Exploring: deepening Act 2 with more of Act 1's systems

**Mode**: `/opsx:explore` | **Status**: open thread, nothing proposed yet
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
