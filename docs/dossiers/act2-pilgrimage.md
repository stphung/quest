# Act 2: The Pilgrimage of Souls — Design Dossier

> Last refreshed: 2026-07-05 @ 04edecd | Sources: `src/vessel/`, `src/main.rs` (vessel wiring), `src/vessel/CLAUDE.md`, `openspec/changes/archive/the-vessel-act2/design.md` (the 15 backported vessel specs, now consolidated into one file), `openspec/specs/vessel-act2/spec.md`, `tests/ferryman_tests.rs`, `src/vessel/colony.rs` unit tests, `src/vessel/transition.rs`, voyage_simulator + ferryman `strategy_sweep` runs, `overlay_snapshot_tests.rs`, played via `QUEST_ACT2=1` fixtures

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
the signal; from then on a whisper crosses the ticker about once a minute and
a `[V]` overlay shows a four-gate checklist — complete the Loom (28 patterns),
reach Ascension X, and accumulate 250,000 PR to burn in one all-or-nothing
action. The player watches a fuel bar fill over weeks. The burn is the act
break: everything Act 1 accumulated becomes the hull.

Then the game changes shape entirely. The Voyage runs on the wall clock
(2.64 game-minutes per real minute; a sea-day ≈ 9 real hours), not the tick
loop. The **maiden voyage** is the decision-rich crossing: ~26 named ports
over ~40 sea-days (≈ two real weeks), checking in a few times a day to chart
courses at junctions (committing closes the other roads for good), set Pace
(the old "trim" dial), stand the watch, answer recruit asks, choose refit
doors, read arrival scenes and Letters From Home. Three roads carry a named,
authored hazard instead of a random encounter — the Ossuary Warden, the
Silence itself, and the Thorns (the act's only permanent loss) — each
resolved deterministically by who's standing where and what pace is set,
never by a die roll. Five other ships share the dark with the player,
each with their own route and one line of character; hailing one (once
each) trades news for a fragment of their story, and one of the five, the
Sister Verity, is written to reach the Tree and wait there rather than
eventually going dark like the other four. Cadence: a decision or scene
every check-in, a junction every few days, a chapter gateway roughly
weekly, the Going-Dark once — the night the mail stops.

Arrival opens three quiet rooms (manifest, keepsake chart, record) — and then
the **Reckoning** reframes the act: 100,000 souls wait in the dying world, the
ship sails back, and every landfall pays Salvage to spend on **three** yards
(Drive = speed, Shipwright = hold, Ward = dampens the dark's daily bite).
Ferry runs (crossing 2+) are fully hands-off — the ship sails herself in
Drive-scaled time while the player's choice compresses to one three-way pick
at each Reckoning, each shown with a concrete "you'd go from X to Y" number.
The era runs ~19–32 crossings depending on how the player spends (see Balance
Evidence), over roughly 3–5 real months, the ramp going 37 → 8 sea-days while
loads climb from 7 crew to ~4,500+ souls, the dark now biting **every day** a
crossing is underway rather than once at its end — so a slow crossing bleeds
longer, and for the first time all three yards visibly answer to that one
pressure.

The design's own name for this loop is **the Ferryman** — `colony.rs`'s own
module doc files itself under "Act 2's incremental spine (sub-project 9,
The Ferryman)." It's never printed on screen as a title (no "you are the
Ferryman" banner anywhere in `src/ui/`), but it's the shape of everything
from arrival onward: the player becomes the one who keeps going back
across the dark. The loop ends rather than cycling in place — once
`souls_remaining` hits zero, the next arrival is **the Last Crossing**:
`era_over()` refuses any further "Sail Again," a second persistent flag
(`GameState::last_crossing_complete`, distinct from `vessel_arrived`, which
was already set at the very first landfall) is raised, and an authored
scene closes the era ("the old world is empty... the door, at last, is
ajar").

## Design Intent

The de-facto design bible is scattered but real (no single consolidated doc):

- **Thesis** (`openspec/changes/archive/the-vessel-act2/design.md`, section
  `2026-03-27-the-vessel-design.md`): Act 1 is "power in one place"; Act 2 is
  *passage*. "Act 2's fun is anticipation, choice, people, and consequence —
  the kinds of fun Act 1 never touched… what accumulates while you're away
  is not numbers. It is *arrival*."
- **Direction choice** (same file, section
  `2026-07-02-act2-voyage-experience-exploration.md`): route/place as spine
  + souls/loss as heart; each check-in should be "an event with a face on
  it, not a status glance."
- **Per-slice intent** in the specs-2–8 sections: arrivals are payoff
  scenes, never tests; no dice anywhere; loss is authored-only; "traveling
  is not dead time."
- **Pacing targets, superseded twice over, both generations preserved in
  the same file**: the `2026-07-03-vessel-ferryman-design.md` section's
  original body still describes a **two-yard, Hope-bearing** Reckoning
  tuned to ~19 crossings / ~87% saved. Its own first follow-up block
  ("the two yards — Drive & Shipwright, earned not compressed") updated
  this to a two-yard Drive/Shipwright design. **A second follow-up block in
  the same section** ("the third yard — Ward, and per-day attrition",
  dated 2026-07-04, same day) documents the shipped three-yard Ward/no-Hope
  system in full — so this *is* described in a spec doc after all, just not
  in the section's original body; a prior version of this dossier claimed
  otherwise and was wrong, not stale (see Refresh History, 2026-07-05).
  The authoritative numbers as shipped: **~19–24 crossings (balanced
  spend), up to ~32 leaning on Ward, ~3–5 real months, ~88–94% saved with
  skilled play, C1 ≈ 14–15 real days.**
- **The Ferryman's "Elevation"** (same file, section
  `2026-07-03-vessel-ferryman-design.md`): explicitly *amends* Act 2's own
  anti-goal — "no resettable loop" — rather than breaking it: "the crossing
  loop *is* Act 2's identity now... it ends (Act 3), it does not cycle in
  place." Chart knowledge and "doors close" both persist forever across
  crossings; only weather, provisions, and per-run road closures reset each
  one. The section's original engine proposal (a "Resonance" multiplier)
  was retired entirely and rebuilt twice over into the shipped
  Drive/Shipwright/Ward yards (see the two Follow-up blocks in the same
  section) — but the *structural* pillar it amends (repeatable crossing →
  permanent colony → an ending, not a reset) is exactly what's live in
  `colony.rs` today, including the "Last Crossing" ending the era once
  `souls_remaining` hits zero (see Interrelations).

Note: specs 1–8 describe single-playthrough duration language ("20–200 days")
that spec 9 (the Ferryman) superseded; nothing in the tree says so explicitly.
This is now a two-deep case of spec drift (see above) worth a documentation
pass, though low priority while the act stays dark-shipped.

## Mechanics & Constants

### Launch gate
`src/vessel/mod.rs`: `LAUNCH_PR_COST` 250,000 (`mod.rs:116`),
`LAUNCH_REQUIRED_PATTERNS` 28 (`:119`), `LAUNCH_REQUIRED_ASCENSION` X (`:122`),
`WHISPER_INTERVAL_SECONDS` 60 (`:125`). Gate at `mod.rs:147-149`, burn `:158`.

### Voyage
`src/vessel/voyage.rs`: `GAME_MINUTES_PER_REAL_MINUTE` 2.64 (`:64`),
`PROVISIONS_CAP` 100 (`:31`, `LONG_HOLD_PROVISIONS_CAP` 150 `:33`),
`DRIFT_RECOVERY_PROVISIONS` 25 (`:38`) / `DRIFT_RECOVERY_HOURS` 36 (`:39`) —
also the affordability floor asserted at `route.rs:1384-1391`, so a dry
hold always means drifting, never stranding. `PORT_CALL_GAME_MINUTES` 360
(`:47`). **Hope's constants (`HOPE_MAX`, `LAUNCH_HOPE`, `HOPE_FLOOR_STEADY`,
`HOPE_SPEND_FLOOR`, `PRESS_*`, `HARD_RATIONS_BURN_MULT`, and
`HOLD_STATION_GRACE_DAYS`) are all gone** — Mourn Pace (was "Restful" trim)
is now identified purely by being the thriftiest hold burn (0.80×), not by
pressing hope, and holding station indefinitely no longer has any soft
pressure attached to it (a small, low-priority drift the mode-transition/
route-waypoints specs also flag).

### Route
`src/vessel/route.rs`: 38 waypoints (`:194`), 45 roads (`:620`), 8
rumors (`:1143`), 4 chapters (`:27`), 7 junctions 2/2/2/1 (`:1373-1380`);
spine-and-diamond DAG, single sink = the Tree (`:186`).

### Souls
`src/vessel/souls.rs`: 8 authored souls for `CREW` = 7 seats (`:16`) —
Torvald/Cormac (Helm), Eir/Ysolt (Tender), Runa/Maren (Watch), Sefa and
Brother Wren unaffined; 3 aboard at launch, 5 found. Arcs advance on
`ARC_BEAT_REST_DAYS` 2 (`:18`). The old `LOSS_HOPE_COST` /
`FAREWELL_HOPE_COST` fees are gone with Hope — `farewell()` and `mark_lost()`
(`voyage.rs:1749,1768`) no longer cost anything; loss stays authored-scenes
only regardless (`mark_lost()` still has no tick-driven caller).

### Strain, hull wear & the Threats
Two linked texture mechanics from "The Price of Passage" that persist
alongside the Reckoning economy — neither is mentioned by name anywhere
else in this dossier's Mechanics section, so both are captured here in
full. **Strain** (`SoulState.strain: u8`, `voyage.rs:221`) accrues on a
soul from one of three named causes (`StrainCause`, `voyage.rs:246-257`):
`ThirdWatch` (three nights on watch back to back), `SquallAtRun` (a squall
crossed while driven hard), `SilenceHelm` (the helm held alone through the
Silence threat, below) — a strained soul pauses their arc and loses
station affinity until rested at port. **Hull wear** (`hull_wear: u8`,
capped at `HULL_WEAR_MAX` 6, `voyage.rs:50`) scars from three causes
(`WearCause`, `:268-279`): a whole leg run at Grueling pace, a squall taken
at Grueling pace, or a threat road's price — each scar adds 5%
(`WEAR_BURN_PER_SCAR` 0.05) to provisions burn, compounding. Every
shipyard refit door secretly carries a **third option**, `choose_mend()`
(`:633-641`): forgo both authored refits at that yard forever in exchange
for zeroing the scars outright — a real, if expensive, escape valve the
dossier's earlier "3 refit door pairs" framing didn't mention.

Wear's other source, and the game's only place loss lives, is
**the Threats** — three specific, named roads resolved by a deterministic
ledger (`threat_ledger()`, `voyage.rs:1168-1230+`), never a die roll:
**the Ossuary Warden** (road 9, over the reef — Sefa aboard sings safe
passage for free; a Quiet/Mourn pace pays 15 provisions; anything faster
pays the same 15 *and* scars the hull), **the Silence itself** (road 29 —
an unstaffed station or the Quiet Keel refit absorbs it for free; a fully
staffed ship pays 10 provisions and loses the leg's log entirely), and
**the Thorns** (road 42 — explicitly commented in source as "the game's
only loss"; Cormac at the Helm reads it clean, any other configuration
can cost a soul). Each outcome is fully determined by crew placement,
pace, and refits chosen beforehand — consistent with the act's "no dice
anywhere" pillar (see Design Intent) even at its single point of genuine
risk.

### The Other Pilgrims
`src/vessel/pilgrims.rs`: five authored ships sharing the dark with the
player, "not a simulation" per the module's own doc comment (`:1-8`) — each
has a name, a one-line character, and a fixed cyclic route script, so their
fates don't depend on the player's choices at all (deliberately, to keep
the authoring bounded). The five (`PILGRIMS`, `:29-`): **the Sister
Verity** (a hospice ship, `dark_after: None` — she alone reaches the Tree
and is explicitly commented as "a face for Act 3"), **the Grief of Alden**
(goes dark mid-crossing), **the Wager**, **the Psalm**, and **the Held
Breath**. Hailing a ship (`hail()`, `voyage.rs:1645`, once per ship) trades
a line of news for a fragment of their story; pilgrim rumors are the only
way to learn about roads behind or beside the player's own. The Sister
Verity is a second, softer Act 3 thread alongside the two hard gate flags
in Interrelations — a named face already written to be waiting at the
Tree, not just a boolean.

### Colony
`src/vessel/colony.rs`: `INITIAL_SOULS` 100,000 (`:21`);
`DRIVE_DECAY` 0.70 → `DRIVE_FLOOR` 0.05 ≈ 20× (`:36,:39`); `BASE_CAPACITY` 180
× `CAP_GROWTH` 1.36 (`:26,:30`); Salvage = 3 + carried/30 per landfall
(`:44,:46`); `STARTING_SALVAGE` 40 (`:49`); yard costs Drive `4×1.5^L`
(`:52-54`... `drive_cost()`), Shipwright `5×1.42^L`, **Ward `5×1.45^L`**
(between the other two). `DARK_TAKES_PER_DAY` 0.0006 (`:65`,
compounds per day underway via `dark_toll_for_days()` at `:351`) replaces the
old flat per-crossing toll; `WARD_DECAY` 0.72 / `WARD_TOLL_FLOOR` 0.12
(`:73,:76`) — the Ward buys the daily rate down to a floor that's never zero.
Six districts found at pop. 500 → 66,000 (`:92-99`), each adding +110…+320
expedition size (`:105-113`). Five `WorldMilestone` thresholds (10/25/50/75/90%
of `INITIAL_SOULS` gone) fire an authored log moment each, exactly once — the
era's second discovery axis, keyed to the old world's decline rather than the
colony's growth, so it lands on a different crossing depending on spend
policy.

**Era end**: once `souls_remaining` reaches 0, `ColonyState::era_over()`
(`colony.rs:571-572`) returns true; `main.rs:834` then refuses `SailAgain` —
no further ferry crossings — and the arrival that emptied the world sets a
second persistent flag, `GameState::last_crossing_complete`
(`main.rs:746-747`, distinct from `vessel_arrived`) — the design's "Last
Crossing," which `colony.rs`'s own module doc calls "Act 3's gate" in as
many words. The flag isn't yet exercised by any Act 3 content, and its
state transition isn't covered by an automated test beyond the save-compat
fixture corpus (which only pins its default `false`) — see Open Questions.

### Launch transition
`src/vessel/transition.rs`: `BEAT_COUNT` 5 (Farewell/Unweaving/Construction/
Launch/Void), gated by the persistent `GameState::vessel_transition_played`.

### Derived numbers players feel
Maiden voyage ≈ two real weeks; ferry-run floor ≈ 3 real days; max hold ≈
4,500+ souls; era ≈ 3–5 real months depending on spend policy (longer if
leaning hard on the Ward).

## Interrelations

```
Act 1 (everything)                Act 2: launch → crossing 1 → the Ferryman loop
  Loom 28 patterns  ─┐
  Ascension X       ─┼─► Launch gate ─► Voyage (crossing 1) ─► vessel_arrived
  250,000 PR (burn) ─┘        │               │             (Act 3 hook #1 —
  Zone 50 kill ─► signal ─────┘               │              spec-normative)
                                              ▼                     │
  Deep/Loom/etc. idle beneath ◄── untouched   Colony founded ◄──────┘
                                              │
                          ┌─────────────────► Reckoning: Salvage
                          │                   ─► Drive / Shipwright / Ward
                          │                          │
                    "Sail Again" ◄─── ferry run, hands-off ──┘
                    (loops until era_over())
                          │
              souls delivered ─► districts + world milestones
              souls_remaining → 0 ─► era_over() blocks "Sail Again"
                          │
                          ▼
              last_crossing_complete (Act 3 hook #2, "the Last Crossing" —
              named in colony.rs's own doc; absent from the openspec spec)
```

- **In**: the launch gate deliberately braids *all* of Act 1 (Loom, Ascension,
  PR economy, Zone 50) into one burn — the act's strongest cross-system edge.
- **During**: zero mechanical interaction with Act 1 — deliberate ("one-way
  passage"); Act 1 idles untouched beneath. Narrative callbacks only
  (Torvald is the Deep guild's captain).
- **The ferry loop is the act's second major cross-system braid**, but an
  internal one: Reckoning spend (Drive/Shipwright/Ward) feeds directly back
  into how much of `souls_remaining` the next crossing can rescue before
  `era_over()` ends it — a closed loop with no Act-1-style external inputs,
  consistent with "During" above. This is the Ferryman design's "Elevation"
  pillar (see Design Intent) made mechanical.
- **Out, in two stages, not one**: `vessel_arrived` fires at the very first
  landfall and is the Act 3 hook `openspec/specs/vessel-act2/spec.md:119`
  actually documents ("the durable hook a future Act 3 keys off"). But the
  Ferryman design's own intent — and the shipped code — go one step
  further: `last_crossing_complete` (`main.rs:746-747`) fires only once
  `souls_remaining` hits zero and the era's final arrival lands, matching
  `colony.rs`'s own module doc verbatim ("the next arrival is the Last
  Crossing: Act 3's gate"). **The openspec capability spec never mentions
  this second, deeper hook** — a real spec gap, not a dossier error (see
  Open Questions). Salvage/districts are Act 2-internal currencies; nothing
  else consumes them (by design, but note: Records and keepsakes have no
  external surface either).
- **A third, softer Act 3 thread**: alongside the two boolean gates above,
  the Sister Verity (see Mechanics & Constants > The Other Pilgrims) is a
  named pilgrim ship authored to reach the Tree and wait there rather than
  going dark like the other four — a face already staged for whatever
  Act 3 becomes, independent of either flag.
- **All three Reckoning purchases point at the same measurable outcome** (%
  of the world saved), a meaningfully tighter system than the earlier
  two-yard-plus-one-inert-gauge (Hope) design — see Refresh History.
- **Resolved**: the Drive/Ward floor dangling-purchase edge is fixed.
  `buy_drive()`/`buy_ward()` (`colony.rs`) now refuse a purchase once
  `drive_maxed()`/`ward_maxed()` is true, and the Reckoning hides the cost
  line entirely for a maxed yard instead of showing an unaffordable price.
  Covered by two colony tests.

## Balance Evidence

*2026-07-04, `cargo test --release --test ferryman_tests -- --ignored
--nocapture strategy_sweep` at d39ad67, plus a locally-added `ward-lean`
policy (blends Ward with Drive/Shipwright, not committed — reproduce via a
spend closure that treats Ward like a third parity-seeking yard) to check the
CLAUDE.md's "~94%" claim, which the committed test suite does not itself
cover:*

| Policy | Crossings | Era length | Souls saved |
|---|---|---|---|
| Drive-only (reckless) | 101 | 11.1 mo | 70.5% |
| Shipwright-only | 15 | 9.1 mo | 74.2% |
| Balanced (Drive+Shipwright, parity) | 24 | 3.4 mo | 88.1% |
| Cap-lean / souls-first | 19 | 3.2 mo | 88.7% |
| Ward-lean (blended with Drive+Shipwright) | 32 | 4.9 mo | 94.3% |

| Intent (CLAUDE.md, as shipped) | Measured | Verdict |
|---|---|---|
| C1 ≈ 14 real days | 15 real days (all policies — Drive level 0 is fixed) | ✓ |
| ~19–24 crossings, ~3 real months, ~88% saved, skilled | 19–24 crossings, 3.2–3.4 mo, 88.1–88.7% (balanced/cap-lean) | ✓ |
| Reckless traps ~70–74% | 70.5% (drive-only), 74.2% (cap-only) | ✓ |
| Leaning on Ward pushes toward ~94%, costlier | 94.3% at 32 crossings / 4.9 mo | ✓ — but almost double the intended era length |
| Care beats carelessness, wide margin | 70.5%–94.3% spread across policies | ✓ |
| No stranding ever | unchanged (affordability invariant still asserted in `route.rs`) | ✓ |

The "hope pinned at max, second gauge never engages" red flag from an
earlier refresh cannot recur — the mechanism is deleted, not just re-tuned.
The dark's daily toll is a **live, checkable number** at every Reckoning
(`voyage_scene.rs`'s `dark_toll_projected()` line), and it visibly differs
across the strategies above — the gauge that used to sit inert now always
engages, in the direction the design intends.

**Watch-item**: leaning hard into the Ward is now the highest-saved policy
(94.3%) but stretches the era to ~5 months and 32 crossings — beyond even the
widened 15–30 test band (the sim run above hit 32, one above the gate's
ceiling). Not a bug — the era test only exercises `balanced_spend` — but
worth flagging: a player who reads "Ward saves the most souls" and leans
all-in may run a noticeably longer era than the stated "~3 real months."
Whether that's an acceptable skill/patience tradeoff or worth a soft cap was
weighed and resolved — see Open Questions.

## Fun Assessment

*Scored against the seven heuristics this dossier and `act1-ascent.md`
both use (these originate from Act 1's own benchmarks, per
`world-and-narrative.md`'s "Design guardrails" section):*

| # | Heuristic | Score | Evidence |
|---|---|---|---|
| 1 | Visible next goal | 4/5 | Gate checklist + fuel bar pre-launch; next-beat timers, watch forecast, district thresholds in-voyage. Still missing an era-level projection ("~N crossings left at this rate"). |
| 2 | Wall → reset → power | 5/5 | The ferry loop is a true earned ramp (37→8 days, 180→4,500+ hold). The dark's toll is a per-day, always-visible, always-engaged number on the Reckoning screen, materially diverging by policy (70.5% vs 94.3%). Resistance is legible. |
| 3 | Discovery cadence | 4/5 | World milestones fix the flagged gap: the ferry era now reveals *two* independent axes of new content — districts (colony growth) and world milestones (old-world decline) — and because milestones key off `souls_remaining` rather than population, they land on different crossings for different spend policies, so the sequence of "what's new" isn't identical run to run. Held at 4 rather than 5 because both axes are still text-only log moments, not new mechanical levers, and the maiden voyage's much richer discovery density (weather, nights, souls, rumors, refits, letters) isn't matched in kind. |
| 4 | Cross-system braiding | 3/5 | The launch gate braids all of Act 1 into the burn (excellent); the voyage itself remains a deliberate island. Confirmed intentional, not a gap. |
| 5 | Decision density | 3/5 | Maiden voyage is decision-rich. Ferry runs: one choice per ~3 real days, but a rich one — three options, each with a live before→after delta shown, not just a level-up button. |
| 6 | Anticipation instruments | 5/5 | The act's strongest suit — fuel bars, watch forecasts, chapter gateways, Letters From Home, the Going-Dark. |
| 7 | Stakes and texture | 4/5 | Stakes are no longer soft: Hope's "pinned at max, nearly no stakes" problem is gone, replaced by a toll that's always live and always differentiates skilled from careless play. The launch transition adds a beat of ceremony to the act's single biggest moment that was previously a bare confirmation screen. What's still missing: the toll and the milestones are numbers/log lines, not scenes — nobody the player has met is ever named as lost to the dark (that stays authored-only, per the `mark_lost()` covenant, and is a deliberate boundary, not a gap). |

**Where Act 2 deliberately breaks Act 1's patterns** (confirm, don't "fix"):
wall-clock instead of ticks; no failure states; no RNG in outcomes; the
voyage severed from Act 1 systems. All four are stated design intent, not
gaps — see `act1-ascent.md`'s Fun Assessment for the mirrored read of Act 1
against these same seven heuristics.

## Open Questions & Decision History

Five of six questions raised across this dossier's refreshes are resolved
(see `docs/decisions.md` for full rationale on each); one new one (#6)
surfaces from this pass and is still open:

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
6. **`last_crossing_complete` (the Last Crossing / true Act 3 gate) has no
   openspec coverage — still open, newly surfaced this pass.** It's
   implemented (`main.rs:746-747,834`, `colony.rs:571-572`) and named
   outright in `colony.rs`'s own module doc ("the next arrival is the Last
   Crossing: Act 3's gate"), but `openspec/specs/vessel-act2/spec.md:119`
   still describes only `vessel_arrived` as "the durable hook a future
   Act 3 keys off," and `src/vessel/CLAUDE.md` doesn't mention the flag at
   all. Not urgent while the act stays dark-shipped and Act 3 is
   undesigned, but whoever eventually designs Act 3 should know the deeper
   hook exists before assuming `vessel_arrived` is the only one. A
   documentation sync (spec + CLAUDE.md), not a code change.

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
and wait for Act 3 rather than going dark like the other four). Added to
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
  the-vessel-act2/design.md` (all 15 original vessel specs, consolidated)
- Implementation docs: `src/vessel/CLAUDE.md`
- Design rationale: `docs/decisions.md`
- Cross-act framing: [`world-and-narrative.md`](world-and-narrative.md)
- Act 1's mirror dossier: [`act1-ascent.md`](act1-ascent.md)
- Balance evidence: `cargo test --release --test ferryman_tests --
  --ignored --nocapture strategy_sweep`, `voyage_simulator`,
  `overlay_snapshot_tests.rs`
