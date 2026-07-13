# Act 2: The Pilgrimage of Souls — Design Dossier

> Last refreshed: 2026-07-05 (the "Ignition" animated launch transition
> shipped and Dock/Wormhole's balance evidence corrected against its own
> shipped validation, see Refresh History) | Sources: `src/vessel/`, `src/main.rs` (vessel wiring), `src/vessel/CLAUDE.md`, `openspec/changes/archive/the-vessel-act2/design.md` (the 15 backported vessel specs, now consolidated into one file), `openspec/changes/archive/2026-07-05-act2-dock-wormhole-crossing/` (Dock/Wormhole, including its `tasks.md` balance validation), `openspec/specs/vessel-act2/spec.md`, `src/ui/vessel_transition_fx.rs` (the Ignition transition renderer), `docs/explorations/2026-07-05-act2-systems-braiding.md` (Dock/Wormhole's originating exploration and the deferred Session 5 braid), `tests/ferryman_tests.rs`, `src/vessel/colony.rs` unit tests, `src/vessel/transition.rs`, voyage_simulator + ferryman `strategy_sweep`/`dock_time_across_charge_policies` runs, `overlay_snapshot_tests.rs`, played via `QUEST_ACT2=1` fixtures

> **Status: living, deep-refreshed across several sessions.** This dossier
> holds Act 2's cross-system, player-eye synthesis — how the launch gate,
> the Voyage, and the ferry-era Reckoning feel as one act, and where the
> shipped mechanics diverge from the archived spec tree.
> `openspec/specs/vessel-act2/spec.md` and `src/vessel/CLAUDE.md` are the
> living per-capability truth this dossier complements, not replaces;
> `docs/decisions.md` holds resolved design calls with rationale. Act 1's
> mirror dossier is [`act1-ascent.md`](act1-ascent.md); the cross-act
> through-line is [`world-and-narrative.md`](world-and-narrative.md).
> Session-by-session refresh history lives at the bottom, under
> [Refresh History](#refresh-history), so the sections below read as one
> current snapshot rather than a change log.

## The Player's Experience

Act 2 begins as a rumor inside Act 1. The first Zone 50 final-boss kill sets
the signal; from then on a whisper crosses the ticker occasionally and a
`[V]` overlay shows a four-gate checklist — complete the Loom (28
patterns), reach Ascension X, and accumulate 250,000 PR to burn in one
all-or-nothing action. The player watches a fuel bar fill over weeks. The
burn is the act break: everything Act 1 accumulated becomes the hull. A
five-beat cinematic — pulsing glow, expanding sonar rings, a warp-speed
rush climaxing in a white flash — carries the player from that burn into
the Void, where the ship's silhouette resolves out of the dark; it is the
only place in the whole act the screen stops being a UI and becomes pure
spectacle.

Then the game changes shape entirely. The Voyage runs on a compressed wall
clock, not the tick loop. The **maiden voyage** is the decision-rich
crossing — a couple of real weeks of checking in a few times a day to
chart courses at junctions (committing closes the other roads for good),
set Pace (the old "trim" dial), stand the watch, answer recruit asks,
choose refit doors, read arrival scenes and Letters From Home. Three roads
carry a named, authored hazard instead of a random encounter — the Ossuary
Warden, the Silence itself, and the Thorns (the act's only permanent loss)
— each resolved deterministically by who's standing where and what pace is
set, never by a die roll. Five other ships share the dark with the player,
each with their own route and one line of character; hailing one (once
each) trades news for a fragment of their story, and one of the five, the
Sister Verity, is written to reach the Tree and wait there (of the five,
only the Grief of Alden ever goes dark, on her authored day). Cadence: a decision or scene
every check-in, a junction every few days, a chapter gateway roughly
weekly, the Going-Dark once — the night the mail stops.

Arrival opens three quiet rooms (manifest, keepsake chart, record) — and
then the **Reckoning** reframes the act: a vast number of souls wait in
the dying world, the ship sails back, and every landfall pays Salvage to
spend on **three** yards (Drive = speed, Shipwright = hold, Ward = dampens
the dark's daily bite). Every arrival — from the maiden voyage's onward —
now opens a **Dock** phase instead of an instant next crossing (spec 9
addendum, 2026-07-05): a new resource, Riftglass, charges from a
Drive-scaled real-time clock, and the player commits a one-way wormhole
jump whenever they choose — full charge is the safe, patient start; an
early jump trades Dock time for a deterministic deficit on the crossing to
come. Once underway, the crossing itself remains fully hands-off — the
ship sails herself in Drive-scaled time — while the player's choice
compresses to the three-way yard pick plus the jump-timing call, each
shown with a concrete "you'd go from X to Y" number. The era runs a good
number of crossings depending on how the player spends (see Balance Evidence), over
a few real months, the pace quickening steadily while loads climb from a
handful of crew into the thousands of souls — the dark now biting **every
day** a crossing is underway rather than once at its end, so a slow
crossing bleeds longer, and for the first time all three yards visibly
answer to that one pressure.

The design's own name for this loop is **the Ferryman**. It's never
printed on screen as a title — there's no "you are the Ferryman" banner
anywhere — but it's the shape of everything from arrival onward: the
player becomes the one who keeps going back across the dark. The loop
ends rather than cycling in place:

- Once the old world is fully emptied, the next arrival is **the Last
  Crossing** — no further Dock/jump is offered.
- A second, deeper flag is raised at that point, distinct from the one
  set at the very first landfall.
- An authored scene closes the era ("the old world is empty... the door,
  at last, is ajar").

## Design Intent

The de-facto design bible is scattered but real (no single consolidated doc
— see Sources for where all of this lives):

- **The original pitch**: Act 1 is "power in one place"; Act 2 is
  *passage*. "Act 2's fun is anticipation, choice, people, and consequence —
  the kinds of fun Act 1 never touched… what accumulates while you're away
  is not numbers. It is *arrival*."
- **The direction-choice exploration** that followed: route/place as spine
  + souls/loss as heart; each check-in should be "an event with a face on
  it, not a status glance."
- **Per-slice intent** across the early design work: arrivals are payoff
  scenes, never tests; no dice anywhere; loss is authored-only; "traveling
  is not dead time."
- **Pacing targets, superseded twice over, all three generations preserved
  in the same design document**: the original Ferryman design described a
  **two-yard, Hope-bearing** Reckoning tuned to ~19 crossings / ~87% saved.
  A first follow-up updated this to a two-yard Drive/Shipwright design. A
  **second follow-up**, written the same day, documents the shipped
  three-yard Ward/no-Hope system in full — so the current system *is*
  described in the design doc after all, just not in its original body; a
  prior version of this dossier claimed otherwise and was wrong, not stale
  (see Refresh History, 2026-07-05). The authoritative numbers as shipped,
  dated 2026-07-04, pre-dating Dock/Wormhole later the same day:
  **~19–24 crossings (balanced spend), up to ~32 leaning on Ward, ~3–5 real
  months, ~88–94% saved with skilled play, C1 ≈ 14–15 real days.** Dock/
  Wormhole's own balance validation moved the measured figures further
  (balanced now 29 crossings/87.5%, Ward-lean 49/93.9% — see Balance
  Evidence); this design-doc line was never updated to match; `src/vessel/
  CLAUDE.md`'s mirror of it was fixed 2026-07-12 (Open Questions #7,
  resolved).
- **The Ferryman's "Elevation"**: the design explicitly *amends* Act 2's own
  anti-goal — "no resettable loop" — rather than breaking it: "the crossing
  loop *is* Act 2's identity now... it ends (Act 3), it does not cycle in
  place." Chart knowledge and "doors close" both persist forever across
  crossings; only weather, provisions, and per-run road closures reset each
  one. The design's original engine proposal (a "Resonance" multiplier) was
  retired entirely and rebuilt twice over into the shipped
  Drive/Shipwright/Ward yards — but the *structural* pillar it amends
  (repeatable crossing → permanent colony → an ending, not a reset) is
  exactly what's live today, including the "Last Crossing" ending the era
  once the old world is fully emptied (see Interrelations).

Note: the earlier design slices describe single-playthrough duration
language ("20–200 days") that the Ferryman slice superseded; nothing in
the design doc says so explicitly. This is now a two-deep case of spec
drift worth a documentation pass, though low priority while the act stays
dark-shipped.

## Mechanics & Constants

### Launch gate
Burns 250,000 Prestige Ranks in one all-or-nothing action once every gate
clears — the signal discovered at Zone 50, the Loom's 28 patterns
complete, Ascension X reached. A whisper crosses the ticker periodically
well before the gate is actually reachable, building anticipation ahead of
the fuel bar itself.

### Voyage
Runs on a compressed wall clock rather than the tick loop, so the maiden
voyage plays out over about two real weeks.

- Provisions are a burnable, replenishable gauge with headroom for a
  longer hold; running dry always means drifting in place until
  recovered, never getting stranded outright — an affordability floor the
  game enforces directly, not just hopes for.
- Hope — the act's original second gauge — is gone entirely. The
  thriftiest pace is now identified purely by being the cheapest
  provisions burn, not by pressing a second resource, and holding station
  indefinitely carries no soft pressure at all anymore (a small,
  low-priority drift a couple of the archived design slices still
  describe otherwise).

### Route
A fixed, one-way route graph shaped like a spine with diamond-shaped
branches that split and rejoin within a chapter, ending at a single
destination — the Tree. Junctions offer a small number of roads;
committing to one closes the others for that crossing, permanently.

### Souls
A small, fully authored crew competing for a handful of duty stations —
some already aboard at launch, the rest found along the way. Each has a
slow-advancing arc. The old per-loss and per-farewell costs tied to Hope
are gone entirely; losing a soul or bidding one farewell costs nothing
mechanically — loss stays an authored-scenes-only event regardless (see
the Threats, below).

### Strain, hull wear & the Threats
Two linked texture mechanics from "The Price of Passage" persist alongside
the Reckoning economy, neither mentioned elsewhere in this dossier until
now.

- **Strain** accrues on a soul from one of three named causes: several
  nights on watch back to back, a squall crossed while driven hard, or
  holding the helm alone through the Silence threat below. A strained
  soul pauses their arc and loses station affinity until rested at port.
- **Hull wear** scars from a parallel set of causes: a whole leg run at
  the hardest pace, a squall taken at that pace, or a threat road's
  price. Each scar compounds the ship's provisions burn.
- Every shipyard refit door secretly carries a **third option**: forgo
  both authored refits at that yard forever in exchange for zeroing the
  scars outright — a real, if expensive, escape valve this dossier's
  earlier refit-door framing didn't mention.

Wear's other source, and the game's only place loss lives, is **the
Threats** — three specific, named roads resolved by a deterministic
ledger, never a die roll:

- **The Ossuary Warden** (over the reef) — a specific soul aboard sings
  safe passage for free; a slower pace pays a toll; anything faster pays
  the same toll *and* scars the hull.
- **The Silence itself** — an unstaffed station or a specific refit
  absorbs it for free; a fully staffed ship pays a toll and loses that
  leg's log entirely.
- **The Thorns** — called out in the design as "the game's only loss";
  one specific soul at the Helm reads it clean, any other configuration
  can cost a soul.

Each outcome is fully determined by crew placement, pace, and refits
chosen beforehand — consistent with the act's "no dice anywhere" pillar
(see Design Intent) even at its single point of genuine risk.

### The Other Pilgrims
Five authored ships share the dark with the player, "not a simulation" by
design — each has a name, a one-line character, and a fixed route script,
so their fates don't depend on the player's choices at all, deliberately,
to keep the authoring bounded. Hailing one, once each, trades a line of
news for a fragment of their story; pilgrim rumors are the only way to
learn about roads behind or beside the player's own. One of the five,
**the Sister Verity**, is written to reach the Tree and wait there — a
second, softer Act 3 thread; of the five, only the Grief of Alden is
authored to go dark (after day 40, "in the middle water"), the other
three sail on alongside the two hard gate flags in Interrelations.

### Colony
Once the maiden voyage lands, every arrival pays out **Salvage** to spend
across three yards, each priced on its own compounding curve:

- **Drive** — speed.
- **Shipwright** — hold.
- **Ward** — softens the dark's daily bite; its cost sits between the
  other two.

The dark's toll is a per-day rate rather than a flat per-crossing tax, so
it compounds over however long a crossing takes — meaning Drive and
Shipwright both cut the toll too, not just Ward; all three yards point at
the same outcome for the first time. A handful of districts unlock as the
colony's population grows, each adding standing capacity; a separate,
parallel set of world milestones fires as the *old* world empties
instead — a second discovery axis keyed to decline rather than growth,
landing on a different crossing depending on how the player spends.

**Era end**: once the old world is fully emptied, `ColonyState::era_over()`
(`souls_remaining == 0`) refuses any further Dock/jump. The arrival that
empties the world sets `last_crossing_complete`, a second persistent flag
distinct from `vessel_arrived` (set at first landfall) — the design's
"Last Crossing," called "Act 3's gate" in the design's own words. That
same arrival plays a five-beat era-end epilogue exactly once
(`ColonyState::take_era_end_playback()`, one-shot via the
colony-persisted `era_end_shown`) from `main.rs`'s clear-screen drain;
the Dock view afterward renders a dedicated quiet-harbor state instead
of offering a jump. The flag is now specced
(`openspec/specs/vessel-act2/spec.md`, "The Last Crossing Ends The Era,"
closed 2026-07-12 — see Open Questions #6) and the epilogue's
one-shot/persistence behavior has its own unit tests in `colony.rs`,
beyond the save-compat fixture corpus. Not yet exercised by any Act 3
content.

### Launch transition
A five-beat authored sequence (Farewell, Unweaving, Construction, Launch,
Void) plays once, gated by its own persistent flag, right before the
Voyage takes the screen. Originally shipped as static per-beat text, it now
plays as a fully animated set-piece — "Ignition" — the single moment in the
whole act where the screen stops being a UI and becomes pure spectacle:
breathing, pulsing text; outward sonar-like rings; the final two beats add
warp-speed streaks radiating from center that climax in a white flash
dissolving into the Void, where the ship's authored silhouette resolves out
of the starfield. Two other visual treatments were built and compared
side by side before this one was chosen as the shipped version.

### Derived numbers players feel
- The maiden voyage plays out over about two real weeks.
- Ferry runs shrink toward just a few real days each as Drive levels climb.
- The ship's hold grows from a handful of crew into the thousands of souls.
- Docking to full Riftglass charge takes about a real day at Drive level 0,
  faster at higher Drive levels — small next to the crossing itself
  (adds roughly a few real days across the whole era).
- The whole ferry era runs a few real months depending on how the player
  spends, longer if leaning hard on the Ward (see Balance Evidence for the
  measured range).

## Interrelations

```
Act 1 (everything)                Act 2: launch → crossing 1 → the Ferryman loop
  Loom's patterns    ─┐
  Ascension's top tier─┼─► Launch gate ─► Voyage (crossing 1) ─► first landfall
  250,000 PR (burn)   ─┘        │               │             (Act 3 hook #1 —
  Zone 50 kill ─► signal ───────┘               │              spec-normative)
                                                ▼                     │
  Deep/Loom/etc. idle beneath ◄── untouched   Colony founded ◄────────┘
                                                │
                          ┌───────────────────► Reckoning: Salvage
                          │                     ─► Drive / Shipwright / Ward
                          │                            │
                     Dock (Riftglass charges) ◄────────┘
                          │
                   wormhole jump ─► ferry run, hands-off ──┐
                   (loops until the old world is emptied)  │
                          │◄───────────────────────────────┘
              souls delivered ─► districts + world milestones
              old world emptied ─► Dock/jump stops being offered
                          │
                          ▼
              world fully emptied (Act 3 hook #2, "the Last Crossing" —
              named in the design's own words; specced 2026-07-12)
```

- **In**: the launch gate deliberately braids *all* of Act 1 (Loom, Ascension,
  PR economy, Zone 50) into one burn — the act's strongest cross-system edge.
- **During**: zero mechanical interaction with Act 1 — deliberate ("one-way
  passage"); Act 1 idles untouched beneath. Narrative callbacks only
  (Torvald is the Deep guild's captain).
- **The ferry loop is the act's second major cross-system braid**, but an
  internal one: Reckoning spend (Drive/Shipwright/Ward) feeds directly back
  into how much of the old world the next crossing can rescue before the
  era ends — a closed loop with no Act-1-style external inputs, consistent
  with "During" above. This is the Ferryman design's "Elevation" pillar
  (see Design Intent) made mechanical.
- **Out, in two stages, not one**: the first landfall is the Act 3 hook the
  normative capability spec actually documents ("the durable hook a future
  Act 3 keys off"). But the Ferryman design's own intent — and the shipped
  game — go one step further: a second, deeper flag fires only once the
  old world is fully emptied and the era's final arrival lands, matching
  the design's own words verbatim ("the next arrival is the Last Crossing:
  Act 3's gate"). **The normative spec's silence on this
  second, deeper hook was a real spec gap** — closed 2026-07-12 by the
  `act2-release-hardening` change ("The Last Crossing Ends The Era"; see
  Open Questions #6).
  Salvage and districts are Act 2-internal currencies; nothing else
  consumes them (by design, but note: Records and keepsakes have no
  external surface either).
- **A third, softer Act 3 thread**: alongside the two boolean gates above,
  the Sister Verity (see Mechanics & Constants > The Other Pilgrims) is a
  named pilgrim ship authored to reach the Tree and wait there (only the
  Grief of Alden ever goes dark) — a face already staged for whatever
  Act 3 becomes, independent of either flag.
- **All three Reckoning purchases point at the same measurable outcome** (%
  of the world saved), a meaningfully tighter system than the earlier
  two-yard-plus-one-inert-gauge (Hope) design — see Refresh History.
- **Resolved**: the Drive/Ward floor dangling-purchase edge is fixed — both
  yards now refuse a purchase once it would buy zero further gain, and the
  Reckoning hides the cost line entirely for a maxed yard instead of
  showing an unaffordable price. Covered by two tests.

## Balance Evidence

*2026-07-12 (3-month retune, `act2-era-pacing-3mo`): `CAP_GROWTH`
1.36 → 1.46 and `DARK_TAKES_PER_DAY` 0.0006 → 0.0007 pulled the balanced
campaign back to the original ~3-month target; the sweeps below are now
COMMITTED, ASSERTED CI gates (`strategy_sweep_holds_the_campaign_envelope`,
including a committed ward-lean policy), no longer `#[ignore]`d or local-only.
Current measured (deterministic):*

| Policy | Crossings | Era length | Souls saved |
|---|---|---|---|
| Drive-only (reckless) | 99 | 11.1 mo | 67.1% |
| Shipwright-only | 10 | 6.3 mo | 78.5% |
| Balanced (Drive+Shipwright, parity) | 22 | 3.1 mo | 88.6% |
| Cap-lean / souls-first | 12 | 2.2 mo | 90.1% |
| Ward-lean (committed policy) | 44 | 7.2 mo | 93.2% |

C1 stays ≈15 real days (voyage clock untouched); patient full-charge
jumping still beats always-jumping-at-0% on souls saved. Asserted bands:
balanced 15–30 crossings / 2.5–4.5 mo / ≥84%; drive-only ≤74%; ward-lean
≥90% and longer than balanced.

*Historical (2026-07-05, pre-retune constants), kept for the record — the
prior session's re-run against then-HEAD:
`cargo test --release --test ferryman_tests -- --ignored --nocapture
strategy_sweep`, `dock_time_across_charge_policies`, and the committed CI
gate `an_era_ferries_most_of_the_world_across_a_ramping_run_of_crossings`;
plus a locally-added "ward-lean" policy (blends Ward with Drive/Shipwright,
not committed — same approach the prior refresh used) to check the shipped
docs' "~94%" claim, which the committed test suite does not itself cover:*

| Policy | Crossings | Era length | Souls saved |
|---|---|---|---|
| Drive-only (reckless) | 101 | 11.4 mo | 70.5% |
| Shipwright-only | 15 | 9.6 mo | 74.2% |
| Balanced (Drive+Shipwright, parity) | 29 | 4.0 mo | 87.5% |
| Cap-lean / souls-first | 18 | 3.2 mo | 88.8% |
| Ward-lean (blended with Drive+Shipwright) | 49 | 7.1 mo | 93.9% |

| Intent (as shipped) | Measured | Verdict |
|---|---|---|
| C1 ≈ 14 real days | 15 real days (all policies — Drive level 0 is fixed) | ✓ |
| ~19–24 crossings, ~3 real months, ~88% saved, skilled | balanced now 29 crossings / 4.0 mo / 87.5% (validated at ship time, see watch-item); cap-lean 18 crossings / 3.2 mo / 88.8% still matches | ~ — stated intent is stale, not the mechanic |
| Reckless traps ~70–74% | 70.5% (drive-only), 74.2% (cap-only) | ✓ |
| Leaning on Ward pushes toward ~94%, costlier | 93.9% at 49 crossings / 7.1 mo | ✓ — already the accepted figure, just not yet in this table |
| Care beats carelessness, wide margin | 70.5%–93.9% spread across policies | ✓ |
| No stranding ever | unchanged (the affordability floor still holds) | ✓ |
| Dock/jump timing tension: patient (full charge) beats rushed (jump at 0%) overall | balanced spend: full charge 29 crossings / 4.0 mo total vs. jump-at-0% 28 crossings / 4.9 mo total, 87.5% vs. 84.5% saved | ✓ — rushing costs more real time and saves fewer souls despite one fewer crossing |

The "hope pinned at max, second gauge never engages" red flag from an
earlier refresh cannot recur — the mechanism is deleted, not just re-tuned.
The dark's daily toll is a **live, checkable number** at every Reckoning,
and it visibly differs across the strategies above — the gauge that used
to sit inert now always engages, in the direction the design intends.

**Watch-item, corrected this session**: every policy line has moved from
the prior refresh's numbers — most notably balanced play, from 24 to 29
crossings (88.1% → 87.5% saved, 3.4 → 4.0 months), and Ward-lean, from 32
to 49 crossings (94.3% → 93.9%, 4.9 → 7.1 months). This is not new drift
this session uncovered — it's the Dock/Wormhole change's own effect,
already measured and accepted at the moment that change shipped: its
archived `tasks.md` (task 6.2) records this exact result ("87.5% → 84.5%
saved, 3.9 → 4.9 months sailing... the intended risk/reward tension holds.
No constant adjustment needed"). The gap is that neither the shipped
change's own doc updates (task 7.1/7.2) nor the dossier's prior refresh
propagated that validated number into this table or into
`src/vessel/CLAUDE.md`'s own plain-English era-length line, which still
reads "~19–24 crossings, ~3 real months, ~88% saved" — a claim that file's
own commit already knew, from its own balance validation, no longer
matched its reference policy (29 crossings, 87.5%). The committed CI gate
itself is unaffected (29 crossings sits inside its 15–30 band, 87,533
delivered clears its ≥78,000 floor) — this is a documentation-propagation
gap, not an unvalidated design regression. See Open Questions for the
`CLAUDE.md` line specifically.

## Fun Assessment

*Scored against the seven heuristics this dossier and `act1-ascent.md`
both use (these originate from Act 1's own benchmarks, per
`world-and-narrative.md`'s "Design guardrails" section):*

| # | Heuristic | Score | Evidence |
|---|---|---|---|
| 1 | Visible next goal | 4/5 | Gate checklist + fuel bar pre-launch; next-beat timers, watch forecast, district thresholds in-voyage. Still missing an era-level projection ("~N crossings left at this rate"). |
| 2 | Wall → reset → power | 5/5 | The ferry loop is a true earned ramp (40→8 in-game days per crossing, 180→4,500+ hold). The dark's toll is a per-day, always-visible, always-engaged number on the Reckoning screen, materially diverging by policy (70.5% vs 93.9%). Resistance is legible. |
| 3 | Discovery cadence | 4/5 | World milestones fix the flagged gap: the ferry era now reveals *two* independent axes of new content — districts (colony growth) and world milestones (old-world decline) — and because milestones key off how much of the old world remains rather than population, they land on different crossings for different spend policies, so the sequence of "what's new" isn't identical run to run. Held at 4 rather than 5 because both axes are still text-only log moments, not new mechanical levers, and the maiden voyage's much richer discovery density (weather, nights, souls, rumors, refits, letters) isn't matched in kind. |
| 4 | Cross-system braiding | 3/5 | The launch gate braids all of Act 1 into the burn (excellent); the voyage itself remains a deliberate island. Confirmed intentional, not a gap. |
| 5 | Decision density | 4/5 | Maiden voyage is decision-rich. Ferry runs gained a second dimension (spec 9 addendum, 2026-07-05): arrival now opens a real-time Dock phase where Riftglass charges from a new Drive-scaled clock, and the player chooses *when* to commit the one-way wormhole jump — full charge is safe and patient, an early jump trades Dock time for a deterministic provisions/hull-wear deficit on the crossing to come. That's a genuine timing decision layered on top of the existing three-way yard spend, not just a faster "level-up" button. Held at 4 rather than 5 because Dock time itself is small relative to sailing time (~0.1–0.5 real months added across a whole era) and the jump-timing choice, while real, is a single dial rather than several interacting levers. |
| 6 | Anticipation instruments | 5/5 | The act's strongest suit — fuel bars, watch forecasts, chapter gateways, Letters From Home, the Going-Dark. |
| 7 | Stakes and texture | 4/5 | Stakes are no longer soft: Hope's "pinned at max, nearly no stakes" problem is gone, replaced by a toll that's always live and always differentiates skilled from careless play. The launch transition — now a fully animated set-piece ("Ignition"), not just static text — adds a beat of ceremony to the act's single biggest moment that was previously a bare confirmation screen. What's still missing: the toll and the milestones are numbers/log lines, not scenes — nobody the player has met is ever named as lost to the dark (that stays authored-only by design, and is a deliberate boundary, not a gap). |

**Where Act 2 deliberately breaks Act 1's patterns** (confirm, don't "fix"):
wall-clock instead of ticks; no failure states; no RNG in outcomes; the
voyage severed from Act 1 systems. All four are stated design intent, not
gaps — see `act1-ascent.md`'s Fun Assessment for the mirrored read of Act 1
against these same seven heuristics.

## Open Questions & Decision History

Eight of nine questions raised across this dossier's refreshes are resolved
(see `docs/decisions.md` for full rationale on each); one is still open
(#9). #6–#8 were closed by the `act2-release-hardening` change
(2026-07-12):

1. ~~Hope gauge never engages — tune, redesign, or demote?~~ **Resolved**:
   retired entirely, replaced by the Ward yard (commit d39ad67).
2. ~~Post-maiden discovery drought — accept, add per-crossing beats, or new
   mid-era noun?~~ **Resolved, then revisited**: first answered "accept as
   intentional" — then world milestones (`WorldMilestone` in `colony.rs`)
   were built after all, a second discovery axis keyed to the old world's
   decline rather than the colony's growth.
3. ~~Era length 24 vs intended ~19 crossings — retune or re-state intent?~~
   **Resolved**: intent re-stated as a range (~19–24, test gate 15–30)
   rather than constants pulled to hit one number.
4. ~~Launch transition is one static screen where the design called for 5
   beats — build or keep?~~ **Resolved, then revisited**: first "keep the
   single screen" — then built the 5-beat sequence (`src/vessel/
   transition.rs`) under the same later build directive as #2.
5. ~~The Ward-lean policy is the highest-souls-saved line (94.3%) but runs
   ~32 crossings / ~5 months — beyond the era's own stated "~3 months" and
   the test's 15–30 band. Intended branch, or nudge Ward's cost curve?~~
   **Resolved**: keep it as an intended "go slower, save more" branch — era
   length is now stated as "~3–5 real months" depending on spend. Skill/
   patience tradeoff is fine; the margin stays wide and legible.
6. ~~`last_crossing_complete` (the Last Crossing / true Act 3 gate) has no
   openspec coverage.~~ **Resolved (2026-07-12)**: the
   `act2-release-hardening` change adds "The Last Crossing Ends The Era"
   to the vessel-act2 spec (plus a "never docks" carve-out on Dock Phase
   Entry) and names the flag in `src/vessel/CLAUDE.md`'s GameState-fields
   list. Original finding, kept for history: it's implemented (`main.rs:746-747,834`, `colony.rs:571-572`)
   and named outright in `colony.rs`'s own module doc ("the next arrival is
   the Last Crossing: Act 3's gate"), but `openspec/specs/vessel-act2/
   spec.md:119` still describes only `vessel_arrived` as "the durable hook
   a future Act 3 keys off," and `src/vessel/CLAUDE.md` doesn't mention the
   flag at all. Not urgent while the act stays dark-shipped and Act 3 is
   undesigned, but whoever eventually designs Act 3 should know the deeper
   hook exists before assuming `vessel_arrived` is the only one. A
   documentation sync (spec + CLAUDE.md), not a code change. Slightly more
   pressing than before: a pre-commitment exploration,
   `docs/explorations/2026-07-05-act-3-4-story-arc.md`, now names
   `last_crossing_complete` outright as the entire gate for a concrete (if
   unshipped) Act 3 direction — nothing here is decided or built, but it's
   a second, independent signal that the spec gap should close before
   anyone starts building against the flag.
7. ~~`src/vessel/CLAUDE.md`'s own era-length intent line contradicts its
   own commit's balance validation.~~ **Resolved (2026-07-12)**: the
   `act2-release-hardening` change rewrote the prose line to the measured
   figures (balanced 29 crossings / ~4.0 mo / 87.5%, ward-lean ~57 / ~9 mo
   / ~93.5%), now CI-asserted by the promoted
   `strategy_sweep_holds_the_campaign_envelope` gate. Original finding: The Dock/Wormhole
   change's archived `tasks.md` (task 6.2) measured and explicitly accepted
   balanced spend at 29 crossings / 87.5% saved / 4.0 months ("the intended
   risk/reward tension holds. No constant adjustment needed") — but that
   same commit left `src/vessel/CLAUDE.md`'s narrative "How It Works" line
   reading "~19–24 crossings, ~3 real months, ~88% saved with a balanced
   spend" unchanged, even though its own validation already knew that
   figure was stale. This dossier's Balance Evidence table is now caught up
   to the accepted numbers (see above); `CLAUDE.md`'s prose line is the one
   place this still needs a documentation-only fix, following the same
   "re-state the range" precedent as Open Questions #3 and #5. Not urgent
   while the CI gate (15–30 crossings) is unaffected.
8. ~~`src/vessel/CLAUDE.md`'s Launch Transition section is stale.~~
   **Resolved (2026-07-12)**: rewritten by the `act2-release-hardening`
   change to describe the animated "Ignition" sequence
   (`ui/vessel_transition_fx.rs`). Original finding: It still reads "no animation... static text screens per
   beat are sufficient" (written when the transition first shipped) and
   its constants table doesn't mention `ui/vessel_transition_fx.rs` at all,
   but the transition has since been rebuilt as a fully animated sequence
   ("Ignition," PR #684) — see Mechanics & Constants and Refresh History.
   A documentation fix to that CLAUDE.md, not a dossier-only correction
   (per this skill's own anti-pattern guidance, not silently fixed here).
9. **Session 5 of the Dock/Wormhole's own originating exploration — a
   ship-tier/district mutual-gating "system of systems" braid, complete
   with veteran souls and a Refinement production chain — remains
   deliberately unbuilt.** `docs/explorations/2026-07-05-act2-systems-
   braiding.md` is where the shipped Dock/Wormhole idea (its "Session 6")
   originated, but that same document's Session 5 sketches a much larger,
   still-unshipped direction for the Ferryman era's own internal braiding
   (raw Salvage capping ship tiers, a Refinery district unlocking refined
   materials, veteran crew ranking up at a station over many crossings) —
   explicitly scoped out of the Dock/Wormhole change as a Non-Goal, not
   rejected. Nothing here is decided or built; worth knowing about before
   proposing further Ferryman-era depth, since it's the designer's own
   already-considered answer to Fun Assessment heuristic 4's "the voyage
   remains a deliberate island" note.

Carried forward from prior refreshes (re-verified current, not
re-litigated): the full 15-doc spec-alignment pass that annotated every
archived vessel spec with "Doc-alignment note" call-outs — the Hope
retirement (present in nearly all of them), the abandoned combat/crew/
rooms-stats specs (confirmed nothing shipped, not just "superseded"), the
mode-transition spec's abandoned continuous-distance shell vs. the built
5-beat transition, and a scatter of smaller numeric drift (road count,
district thresholds, era-length estimates) — is still valid after the
OpenSpec migration moved the whole tree into
`openspec/changes/archive/the-vessel-act2/design.md` (see Refresh History,
2026-07-05). A stale `src/vessel/CLAUDE.md` "Known Invariants" doc-drift
item (a `game_state.rs` 100k/250k PR mismatch) was fixed and re-confirmed
absent.

Held for a later round (not yet asked, unchanged):
- Should Records/keepsakes surface anywhere outside the arrived harbor
  (e.g. title screen), given the era ends and the files remain?
- No player-facing wiki page exists for Act 2 (correct while dark-shipped;
  becomes a launch-checklist item).

## Refresh History

Session-by-session log of what changed at each refresh, most recent first.
The sections above always describe the *current* state only — read this
section for how it got there.

### 2026-07-13 — the era's ending is authored (`act2-era-epilogue`)

The Last Crossing dead-end is closed: a five-beat, state-conditioned
epilogue (the settled account — delivered, taken, crossings, days at sea,
districts — closing on Sister Verity and the door ajar) now plays exactly
once, colony-persisted (`era_end_shown`), reload-safe, replacing the old
one-line modal. The post-era Dock renders a quiet harbor instead of a 0%
charge bar with a no-op jump preview (a real affordance bug, fixed); the
Record keeps a permanent era block; the Reckoning drops its next-crossing
projection once the era is over. The Last Crossing spec requirement gained
its missing automated coverage (one-shot/persistence tests + era-over
Dock/Record snapshots + a live e2e pass). Held-for-later question
(era record outside the harbor, e.g. title screen) unchanged.

### 2026-07-12 (later) — the 3-month retune (`act2-era-pacing-3mo`)

Per direction, the balanced campaign was tuned from the accepted-by-docs
4.0 months back to the original ~3-month target: `CAP_GROWTH` 1.36 → 1.46
(fewer, fuller crossings — the dominant lever, per measured single-knob
sensitivity) with `DARK_TAKES_PER_DAY` 0.0006 → 0.0007 compensating so the
naive extremes stay traps. Landed at balanced 22 crossings / 3.1 mo /
88.6% saved; ward-lean 44 / 7.2 / 93.2%; drive-only 67.1%. The envelope
gates tightened to enforce it (2.5–4.5 mo). Balance Evidence table above
updated; the pre-retune table kept as historical record.

### 2026-07-12 — release hardening lands; Open Questions #6–#8 resolved

The `act2-release-hardening` change (release-readiness follow-through, not
a dossier refresh) closed three of the four open questions: the Last
Crossing is now specced in `openspec/specs/vessel-act2` (#6), `src/vessel/
CLAUDE.md`'s era-length prose matches the measured, now-CI-asserted
figures (#7), and its Launch Transition section describes Ignition (#8).
The same change promoted the `#[ignore]`d ferryman sweeps into asserted
balance gates (adding the previously-missing ward-lean policy: measured
~57 crossings / ~9 mo / ~93.5% saved), wired `voyage_simulator` and a
`QUEST_ACT2=1` flag-ON test step into CI, added `voyage.json`/
`colony.json` to the save-compat corpus, and reconciled this dossier's
"going dark like the other four" pilgrim phrasing to the authored code
(only the Grief of Alden darkens; the test
`the_grief_of_alden_goes_dark_and_the_verity_sails_on` pins it). #9 (the
Session 5 systems braid) remains the one open question.

### 2026-07-05 — Ignition transition; balance evidence corrected against its own shipped validation; Session 5 exploration surfaced

Diffed the dossier's Sources paths against HEAD. First pass found the
launch transition and a stale Balance Evidence table; a follow-up pass,
prompted by a reviewer flagging that the Dock/Wormhole change itself
deserved a closer read (not just its headline numbers), traced the actual
root cause and surfaced a real, previously-uncited exploration document:

- **The launch transition shipped its final visual form.** PR #684 rebuilt
  the previously-static 5-beat launch transition (`transition.rs`) as
  "Ignition" (`src/ui/vessel_transition_fx.rs`, new): pulsing text, sonar
  rings, warp-speed streaks climaxing in a white flash into the Void, where
  the ship's authored art now resolves as one aligned block instead of
  drifting line-by-line (a real alignment bug the build's two rejected
  comparison variants surfaced and fixed). Added to Player's Experience and
  Mechanics & Constants; Fun Assessment heuristic 7's evidence updated.
  Flagged as a new Open Question (#8) that `src/vessel/CLAUDE.md`'s Launch
  Transition section still describes the old static version.
- **Balance Evidence's drift is the Dock/Wormhole change's own already-
  accepted result, not new or unexplained.** The prior refresh's table (24
  crossings/88.1% balanced, 32/94.3% Ward-lean) predates that change's own
  `tests/ferryman_tests.rs` rewrite. Re-ran `strategy_sweep`,
  `dock_time_across_charge_policies`, the committed CI era gate, and a
  fresh local (uncommitted) ward-lean policy: balanced spend now measures
  29 crossings/87.5%/4.0 months, Ward-lean 49/93.9%/7.1 months. Tracing
  this back to the archived `openspec/changes/archive/2026-07-05-act2-
  dock-wormhole-crossing/tasks.md` (task 6.2) shows these exact figures
  (87.5%→84.5%, 3.9→4.9 months for the charge-policy comparison) were
  already measured and explicitly accepted *at the moment Dock/Wormhole
  shipped* — "the intended risk/reward tension holds. No constant
  adjustment needed." What was actually missing was propagation: neither
  that change's own doc tasks (7.1/7.2) nor the dossier's prior refresh
  carried the validated number into this table or into `src/vessel/
  CLAUDE.md`'s own "~19–24 crossings, ~3 real months, ~88% saved" line,
  which the same commit's own validation had already outdated. Reframed
  from "new drift, worth investigating" to "known and accepted, just
  never propagated" — Open Question #7 rewritten accordingly, and Design
  Intent's quoted design-doc figures annotated as pre-dating Dock/
  Wormhole. The committed CI gate itself was never at risk (29 sits inside
  its 15–30 band).
- **Surfaced `docs/explorations/2026-07-05-act2-systems-braiding.md`** (PR
  #682, not previously cited by this dossier) — the exploration Session
  6 of which is literally where the shipped Dock/Wormhole idea originated,
  confirming the mechanism this dossier already described. More
  importantly, that same document's Session 5 sketches a much larger,
  still-unshipped "system of systems" direction for the Ferryman era
  itself (ship tiers and districts mutually gating each other, a
  Refinement production chain, veteran souls ranking up over many
  crossings) — explicitly scoped out of the Dock/Wormhole change as a
  Non-Goal, not rejected. Added as new Open Question #9: a completeness-
  audit catch (a named, real, load-bearing-for-the-future design direction
  this dossier hadn't reflected under its own name).
- **Verified the Act 3 hook gap (Open Question #6) is still open**, and
  noted a second, independent signal for it: a same-day pre-commitment
  exploration doc, `docs/explorations/2026-07-05-act-3-4-story-arc.md`,
  now names `last_crossing_complete` as the entire gate for a concrete (if
  unshipped) Act 3 direction.
- Confirmed via `cargo build --release --test ferryman_tests` (clean) and
  the test runs cited above; no source or spec files were edited this pass
  — dossier-only refresh.

### 2026-07-05 — Dock/Wormhole/Riftglass shipped (spec 9 addendum)

Implemented `openspec/changes/act2-dock-wormhole-crossing/`, the
`docs/explorations/2026-07-05-act2-systems-braiding.md` Session 6 idea:
arrival no longer auto-starts the next crossing. A real-time Dock phase
opens instead, charging a new resource (Riftglass, Drive-scaled rate) that
gates a one-way wormhole jump — full charge is the safe/patient start, a
partial charge trades Dock time for a deterministic provisions/hull-wear
deficit on the crossing to come. Updated: the Player's Experience section's
"Ferry runs are fully hands-off" claim (now qualified — the *crossing* is
hands-off once underway, but the Dock/jump timing is a new player choice);
the Interrelations diagram and "Sail Again" terminology throughout (renamed
to Dock/jump, matching the shipped `VoyageInputResult::Jump`); Fun
Assessment heuristic 5 (Decision density), raised 3/5 → 4/5. Balance
evidence: `ferryman_tests::dock_time_across_charge_policies` (run with
`--ignored --nocapture`) shows Dock time adds only ~0.1–0.5 real months
across a ~4–11 month era at the shipped `RIFTGLASS_BASE_HOURS_TO_FULL =
24.0`, and that rushing every jump (0% charge) costs *more* overall
(worse crossings) than patiently waiting for full charge — the intended
risk/reward tension holds without further tuning. Historical Refresh
History entries below that reference the pre-Dock `SailAgain` mechanic are
left as-written — they're a record of what was true at the time, not
current-state prose.

### 2026-07-05 — code references stripped from prose, bulleted for structure

Feedback: referencing functions and files isn't necessary in prose, and
dense paragraphs should use more bullet/structure. Applied throughout
Player's Experience, Design Intent, Mechanics & Constants, and
Interrelations — inline `file.rs`, function-call, and struct-field
citations removed (the header and closing Sources sections already point
a verifier at the right module), and passages enumerating three or more
related things (named causes, gates, yards, steps) converted from dense
paragraphs into bullet lists. Open Questions keeps its file/line citations
— there, a specific location in the code is the actionable substance of
the finding, not decoration. No mechanics, balance evidence, or
fun-assessment content changed.

### 2026-07-05 — rewritten concept-first

The dossier had drifted into reading like a second `CLAUDE.md` — every
mechanic accompanied by its exact formula, cost curve, or percentage
table. Rewritten per an updated `write-dossier` skill: Mechanics &
Constants now leads with what each system *is* and how it *relates* to the
rest of the design, keeping a number only when it's a structural
identifier already used elsewhere (250,000 PR, 28 patterns, Ascension X),
the exact magnitude is itself the point (Fracture vs. Loom's stat scaling,
in the Act 1 dossier), or it feeds Balance Evidence directly — which stays
fully numeric, unchanged, since measured results are exactly where precise
figures belong. Player's Experience and a couple of Design Intent bullets
got the same light trim. No mechanics, balance evidence, or fun-assessment
content changed — this is a legibility pass, not a factual one.

### 2026-07-05 — Strain, hull wear, the Threats, and the Other Pilgrims added

A `/goal` to verify both act dossiers reflect the current design prompted a
dedicated completeness audit against the archived design.md, `src/vessel/
CLAUDE.md`, and `docs/decisions.md` — specifically hunting for other named
concepts missing the way the Ferryman was. Found three, all still live in
`src/vessel/voyage.rs` and `src/vessel/pilgrims.rs` and none previously
mentioned anywhere in this dossier: **Strain & hull wear** (`StrainCause`/
`WearCause`, the shipyard's hidden third "mend the hull" refit option),
**the Threats** (three named, ledger-resolved hazard roads — the Ossuary
Warden, the Silence itself, and the Thorns, "the game's only loss," none
of them RNG), and **the Other Pilgrims** (five authored ships with fixed
routes, hailable once each; the Sister Verity is written to reach the Tree
and wait for Act 3; only the Grief of Alden goes dark). Added to
Player's Experience, two new Mechanics & Constants subsections, and a new
Interrelations bullet naming the Verity as a third, narrative Act 3 thread
alongside the two boolean gate flags. Content addition, not a correction —
nothing previously stated was wrong.

### 2026-07-05 — the Ferryman loop and the Last Crossing gate added

A reviewer read the previous restructuring pass and asked whether "the
Ferryman" idea was represented — it wasn't, in the Interrelations section
or anywhere else. Checked against source rather than assumed: the ferry
loop (repeatable crossings, `souls_remaining` racing toward zero) and its
end-state (`ColonyState::era_over()` blocking `SailAgain`, and a second
persistent flag, `GameState::last_crossing_complete`, distinct from
`vessel_arrived`) are fully implemented (`colony.rs:571-572`,
`main.rs:746-747,834`) and named outright in `colony.rs`'s own module doc
("sub-project 9, The Ferryman" / "the next arrival is the Last Crossing:
Act 3's gate") — but were absent from this dossier, from
`openspec/specs/vessel-act2/spec.md`, and from `src/vessel/CLAUDE.md`
alike. Added to Player's Experience (naming the Ferryman and the Last
Crossing), Design Intent (the Ferryman design's "Elevation" pillar
amendment), Mechanics & Constants (era-end mechanics under Colony),
Interrelations (the diagram now shows the full loop and both Act 3 hooks),
and a new open question (#6) flagging that the openspec spec still only
documents the first hook. This is a content addition, not a correction —
nothing previously stated was wrong, it was incomplete.

### 2026-07-05 — path housekeeping only

Commit 974bdbb ("Adopt OpenSpec as source of truth; archive the old design
docs") removed `docs/superpowers/specs/` entirely between the prior refresh
and this one — the 15 vessel specs it cited by path now live concatenated
(verbatim, each under its own `## <original-filename>.md` heading) in
`openspec/changes/archive/the-vessel-act2/design.md`, and the capability
itself is now normatively described in `openspec/specs/vessel-act2/spec.md`.
No mechanics changed; this pass repointed every stale path and corrected
one claim that no longer held now that the full file is readable in one
place: Design Intent previously said the shipped three-yard Ward/no-Hope
system "exists only in code... not in a spec doc" — the archived
design.md's `2026-07-03-vessel-ferryman-design.md` section in fact carries
a same-day "Follow-up (2026-07-04, later the same day): the third yard —
Ward" block describing it in full. That block was already there at the
prior refresh; the dossier's own claim was simply wrong, not something that
drifted afterward. This pass also restructured the whole dossier to mirror
`act1-ascent.md`'s section layout (Status blockquote, per-subsystem
Mechanics & Constants headers, a closing Sources list, and this Refresh
History section instead of change-log entries stacked at the top).

### Same-day build session (world milestones, launch transition, full spec alignment)

All three open questions from the prior refresh were resolved by the
designer (keep Ward's long line as intended, accept the discovery drought,
keep the single-screen launch confirmation) — logged in `docs/decisions.md`.
The designer then asked for actual construction work, not just more
decisions, so this pass built three things and did a full spec-tree
alignment:

- **Fixed the Drive/Ward floor dangling-purchase edge.** `buy_drive()`/
  `buy_ward()` (`colony.rs`) now refuse a purchase via new
  `drive_maxed()`/`ward_maxed()` checks once further Salvage would buy zero
  gain; the Reckoning UI hides the cost line entirely for a maxed yard
  instead of showing an unaffordable price. Two new colony tests; one
  snapshot re-blessed (`voyage_reckoning_xl_160x45`).
- **Built the 5-beat launch transition** (`src/vessel/transition.rs`, new)
  — the design's Farewell/Unweaving/Construction/Launch/Void sequence, static
  full-screen text per beat, rendered by
  `ui::vessel_scene::render_launch_transition()`, gated by a new persistent
  `GameState::vessel_transition_played` flag, wired into `main.rs` right
  before the Voyage takes the screen. Two new overlay snapshots, two unit
  tests in `transition.rs`.
- **Added world milestones** (`WorldMilestone` in `colony.rs`) — a second,
  genuinely new mid-era discovery axis for the ferry era: not the colony's
  growth (districts, already covered) but the *dying world's* decline (10%
  / 25% / 50% / 75% / 90% of `INITIAL_SOULS` gone), firing an authored log
  moment each. `deliver_crossing()` now returns a `CrossingDelivery {
  new_districts, new_world_milestones }` instead of a bare `Vec<District>`.
  Because it's keyed to how much of the *old* world is gone rather than how
  large the colony has grown, a milestone lands on a different crossing
  depending on spend policy (Ward-heavy vs. Shipwright-heavy) — a genuinely
  new noun, not districts on a second axis. Four new tests (two unit, two
  integration) confirm all five fire exactly once, in order, spread across
  the era.
- **Full spec-tree alignment**: all 15 vessel docs (then under
  `docs/superpowers/specs/`, later consolidated into
  `openspec/changes/archive/the-vessel-act2/design.md`) read against
  current source and annotated with "Doc-alignment note" call-outs — the
  Hope retirement (touched nearly every spec), the abandoned
  combat/crew/rooms-stats specs (confirmed nothing shipped, not just
  "superseded"), the mode-transition spec's abandoned continuous-distance
  shell vs. its now-built 5-beat transition, and a scatter of smaller
  numeric drift (road count, district thresholds, era-length estimates, a
  dead Lantern Mast refit, a stale Mourning Colors number). `src/vessel/
  CLAUDE.md` also picked up two small pre-existing drift items found along
  the way: a stale "two yard tracks" phrase (now three) and an inaccurate
  claim about which persistence path is load-bearing (`character/
  persistence.rs`, not `core/game_state_serde.rs`'s `FlatGameState`, which
  is dead code from an earlier migration).
- All of the above verified: `cargo test` (full suite, 0 failures),
  `cargo clippy --all-targets -- -D warnings` (clean), `cargo fmt --check`
  (clean), `cargo run --release --bin simulator -- --check-progression`
  (passed), `cargo test --release --test ferryman_tests` (passed).

### 2026-07-04 @ 4ce9a57 — Ward retires Hope

One commit landed between refreshes and it resolves the dossier's #1 open
question outright: **d39ad67 "retire Hope into a three-yard Reckoning"**.

- **Hope is gone.** The old second gauge (`HOPE_MAX`, `LAUNCH_HOPE`,
  `HOPE_FLOOR_STEADY`, Press-the-helm, Hard Rations) is deleted entirely —
  every constant, field, and Trim behavior tied to it. The flagged red flag
  ("hope ended 10/10 with minimum 7 in 24 of 25 runs — the gauge never
  engages") can't recur because the mechanism it describes no longer exists.
- **A third yard, the Ward, replaces it as the attrition lever.** The
  Reckoning (`[L]` view) now shows three purchases side by side —
  Drive `[D]` (speed), Shipwright `[C]` (hold), Ward `[W]` (dampens the
  dark's bite) — each with a live before→after number, not just a level.
- **The dark's toll changed shape**: from a flat per-crossing tax
  (`DARK_TAKES_EACH_CROSSING`, retired) to a **per-day** rate
  (`DARK_TAKES_PER_DAY` = 0.0006, compounding over the crossing's days,
  `colony.rs:64,351`). This is the mechanically important change: it means
  Drive (fewer days) and Shipwright (fewer crossings) both now cut the toll
  too, not just the new Ward yard — all three purchases point at the same
  outcome for the first time.
- **Era-length intent was widened, not re-tuned.** The old target was a
  single number (~19 crossings); the shipped intent (CLAUDE.md, era test
  gate) now reads "~19–24 crossings" and the test asserts a `15..=30` band.
- **Housekeeping**: this refresh also fixed a doc-drift item the last
  refresh introduced and then this commit's CLAUDE.md pass missed —
  `src/vessel/CLAUDE.md`'s "Known Invariants" section still described the
  `game_state.rs` 100k/250k PR mismatch as current; it was already fixed in
  source. Re-worded to say there's no drift.

## Sources

- Act 2 capability spec: `openspec/specs/vessel-act2/spec.md`
- Backported per-feature design intent: `openspec/changes/archive/
  the-vessel-act2/design.md` (all 15 original vessel specs, consolidated),
  `openspec/changes/archive/2026-07-05-act2-dock-wormhole-crossing/`
  (Dock/Wormhole)
- Implementation docs: `src/vessel/CLAUDE.md`
- Launch transition animation: `src/ui/vessel_transition_fx.rs`
- Design rationale: `docs/decisions.md`
- Dock/Wormhole's originating exploration (Session 6), and the
  still-unshipped Ferryman-era "system of systems" braid it deferred
  (Session 5): `docs/explorations/2026-07-05-act2-systems-braiding.md`
- Forward-looking, unshipped Act 3/4 context:
  `docs/explorations/2026-07-05-act-3-4-story-arc.md`
- Cross-act framing: [`world-and-narrative.md`](world-and-narrative.md)
- Act 1's mirror dossier: [`act1-ascent.md`](act1-ascent.md)
- Balance evidence: `cargo test --release --test ferryman_tests --
  --ignored --nocapture strategy_sweep`, `dock_time_across_charge_policies`,
  `an_era_ferries_most_of_the_world_across_a_ramping_run_of_crossings`,
  `voyage_simulator`, `overlay_snapshot_tests.rs`
