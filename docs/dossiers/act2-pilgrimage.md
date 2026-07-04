# Act 2: The Pilgrimage of Souls — Design Dossier

> Last refreshed: 2026-07-04 (post-build) | Sources: `src/vessel/`, `src/vessel/CLAUDE.md`, `docs/superpowers/specs/` (all 15 vessel specs, fully doc-aligned this session), `tests/ferryman_tests.rs`, `src/vessel/colony.rs` unit tests, `src/vessel/transition.rs`, voyage_simulator + ferryman `strategy_sweep` runs, `overlay_snapshot_tests.rs`, played via `QUEST_ACT2=1` fixtures

## Since you last looked (this session's build, same day as the previous refresh)

All three open questions from the last refresh were resolved by the
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
  — spec 4's Farewell/Unweaving/Construction/Launch/Void sequence, static
  full-screen text per beat (per the spec's own allowance), rendered by
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
- **Full spec-tree alignment**: all 15 `docs/superpowers/specs/` vessel
  docs read against current source and annotated with "Doc-alignment note"
  call-outs — the Hope retirement (touched nearly every spec), the
  abandoned combat/crew/rooms-stats specs (confirmed nothing shipped, not
  just "superseded"), the mode-transition spec's abandoned
  continuous-distance shell vs. its now-built 5-beat transition, and a
  scatter of smaller numeric drift (road count, district thresholds,
  era-length estimates, a dead Lantern Mast refit, a stale Mourning Colors
  number). `src/vessel/CLAUDE.md` also picked up two small pre-existing
  drift items found along the way: a stale "two yard tracks" phrase (now
  three) and an inaccurate claim about which persistence path is load-bearing
  (`character/persistence.rs`, not `core/game_state_serde.rs`'s
  `FlatGameState`, which is dead code from an earlier migration).
- All of the above verified: `cargo test` (full suite, 0 failures),
  `cargo clippy --all-targets -- -D warnings` (clean), `cargo fmt --check`
  (clean), `cargo run --release --bin simulator -- --check-progression`
  (passed), `cargo test --release --test ferryman_tests` (passed).

## Since you last looked (delta from the 2026-07-04 @ 4ce9a57 refresh, same day)

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
  This answers old open question #3 by **re-stating intent as a range**
  rather than pulling constants to hit one number — see Balance Evidence.
- **Not touched by this commit** (still open, carried forward): the
  post-maiden discovery drought (#2) and the launch transition still being
  one static screen instead of spec 4's five beats (#4).
- **Housekeeping**: this refresh also fixed a doc-drift item the last
  refresh introduced and then this commit's CLAUDE.md pass missed —
  `src/vessel/CLAUDE.md`'s "Known Invariants" section still described the
  `game_state.rs` 100k/250k PR mismatch as current; it was already fixed in
  source. Re-worded to say there's no drift. See Interrelations for a
  spec-doc drift this commit *did* introduce.

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
doors, read arrival scenes and Letters From Home. Cadence: a decision or
scene every check-in, a junction every few days, a chapter gateway roughly
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
- **Pacing targets, superseded twice over**: spec 9's original body
  (`…/2026-07-03-vessel-ferryman-design.md:315-319`) still describes a
  **two-yard, Hope-bearing** Reckoning tuned to ~19 crossings / ~87% saved.
  A same-file follow-up note (`:310-318`, dated 2026-07-04) updated this to
  the two-yard Drive/Shipwright design. **Neither describes the shipped
  three-yard Ward/no-Hope system** — that redesign exists only in code
  (`colony.rs` doc comments, `CLAUDE.md`) and the commit message, not in a
  spec doc. The authoritative numbers as shipped: **~19–24 crossings
  (balanced spend), up to ~32 leaning on Ward, ~3–5 real months, ~88–94%
  saved with skilled play, C1 ≈ 14–15 real days.**

Note: specs 1–8 describe single-playthrough duration language ("20–200 days")
that spec 9 (the Ferryman) superseded; nothing in the tree says so explicitly.
This is now a two-deep case of spec drift (see above) worth a documentation
pass, though low priority while the act stays dark-shipped.

## Mechanics & Constants

Launch gate (`src/vessel/mod.rs`): `LAUNCH_PR_COST` 250,000 (`mod.rs:116`),
`LAUNCH_REQUIRED_PATTERNS` 28 (`:119`), `LAUNCH_REQUIRED_ASCENSION` X (`:122`),
`WHISPER_INTERVAL_SECONDS` 60 (`:125`). Gate at `mod.rs:147-149`, burn `:158`.

Voyage (`src/vessel/voyage.rs`): `GAME_MINUTES_PER_REAL_MINUTE` 2.64 (`:64`),
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

Route (`src/vessel/route.rs`): 38 waypoints (`:194`), 45 roads (`:620`), 8
rumors (`:1143`), 4 chapters (`:27`), 7 junctions 2/2/2/1 (`:1373-1380`);
spine-and-diamond DAG, single sink = the Tree (`:186`).

Souls (`src/vessel/souls.rs`): 8 authored souls for `CREW` = 7 seats (`:16`) —
Torvald/Cormac (Helm), Eir/Ysolt (Tender), Runa/Maren (Watch), Sefa and
Brother Wren unaffined; 3 aboard at launch, 5 found. Arcs advance on
`ARC_BEAT_REST_DAYS` 2 (`:18`). The old `LOSS_HOPE_COST` /
`FAREWELL_HOPE_COST` fees are gone with Hope — `farewell()` and `mark_lost()`
(`voyage.rs:1749,1768`) no longer cost anything; loss stays authored-scenes
only regardless (`mark_lost()` still has no tick-driven caller).

Colony (`src/vessel/colony.rs`): `INITIAL_SOULS` 100,000 (`:21`);
`DRIVE_DECAY` 0.70 → `DRIVE_FLOOR` 0.05 ≈ 20× (`:36,:39`); `BASE_CAPACITY` 180
× `CAP_GROWTH` 1.36 (`:26,:30`); Salvage = 3 + carried/30 per landfall
(`:44,:46`); `STARTING_SALVAGE` 40 (`:49`); yard costs Drive `4×1.5^L`
(`:52-54`... `drive_cost()`), Shipwright `5×1.42^L`, **Ward `5×1.45^L`**
(between the other two). **New**: `DARK_TAKES_PER_DAY` 0.0006 (`:65`,
compounds per day underway via `dark_toll_for_days()` at `:351`) replaces the
old flat per-crossing toll; `WARD_DECAY` 0.72 / `WARD_TOLL_FLOOR` 0.12
(`:73,:76`) — the Ward buys the daily rate down to a floor that's never zero.
Six districts found at pop. 500 → 66,000 (`:92-99`), each adding +110…+320
expedition size (`:105-113`). **New this session**: five `WorldMilestone`
thresholds (10/25/50/75/90% of `INITIAL_SOULS` gone) fire an authored log
moment each, exactly once — the era's second discovery axis, keyed to the
old world's decline rather than the colony's growth, so it lands on a
different crossing depending on spend policy.

Launch transition (`src/vessel/transition.rs`, new this session):
`BEAT_COUNT` 5 (Farewell/Unweaving/Construction/Launch/Void), gated by the
new persistent `GameState::vessel_transition_played`.

Derived numbers players feel: maiden voyage ≈ two real weeks; ferry-run floor
≈ 3 real days; max hold ≈ 4,500+ souls; era ≈ 3–5 real months depending on
spend policy (longer if leaning hard on the Ward).

## Interrelations

```
Act 1 (everything)          Act 2 (the passage)
  Loom 28 patterns  ─┐
  Ascension X       ─┼─► Launch gate ─► Voyage ─► Colony era
  250,000 PR (burn) ─┘        │            │          │
  Zone 50 kill ─► signal ─────┘            │     Salvage ─► Drive/Shipwright/Ward
                                           │          │
  Deep/Loom/etc. idle beneath ◄── untouched┘     souls delivered ─► districts
                                                       │
                                              vessel_arrived ─► (Act 3 hook)
```

- **In**: the launch gate deliberately braids *all* of Act 1 (Loom, Ascension,
  PR economy, Zone 50) into one burn — the act's strongest cross-system edge.
  Unchanged by this commit.
- **During**: zero mechanical interaction with Act 1 — deliberate ("one-way
  passage"); Act 1 idles untouched beneath. Narrative callbacks only
  (Torvald is the Deep guild's captain).
- **Out**: `vessel_arrived` is the authored Act 3 hook. Salvage/districts are
  Act 2-internal currencies; nothing else consumes them (by design, but note:
  Records and keepsakes have no external surface either).
- **New internal edge**: for the first time, all three Reckoning purchases
  point at the same measurable outcome (% of the world saved) instead of two
  purchases (Drive/Shipwright) plus one inert gauge (Hope) that never
  affected anything a player could feel. This is a meaningfully tighter
  system than the two-yard version the dossier last described.
- **Resolved 2026-07-04**: the Drive/Ward floor dangling-purchase edge is
  fixed. `buy_drive()`/`buy_ward()` (`colony.rs`) now refuse a purchase
  once `drive_maxed()`/`ward_maxed()` is true, and the Reckoning hides the
  cost line entirely for a maxed yard instead of showing an unaffordable
  price. Covered by two new colony tests.
- **Doc drift introduced by this commit**: spec 9
  (`2026-07-03-vessel-ferryman-design.md`) still describes the two-yard,
  Hope-bearing design as current; the three-yard Ward/no-Hope system that
  actually shipped exists only in code comments and the commit message.

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
| Care beats carelessness, wide margin | 70.5%–94.3% spread across policies | ✓, wider than before (was 54%–87%) |
| No stranding ever | unchanged (affordability invariant still asserted in `route.rs`) | ✓ |

**Resolved from last refresh**: the "hope pinned at max, second gauge never
engages" red flag cannot recur — the mechanism is deleted, not just
re-tuned. The dark's daily toll is a **live, checkable number** at every
Reckoning (`voyage_scene.rs`'s `dark_toll_projected()` line), and it visibly
differs across the strategies above — the gauge that used to sit inert now
always engages, in the direction the design intends.

**New watch-item**: leaning hard into the Ward is now the highest-saved
policy (94.3%) but stretches the era to ~5 months and 32 crossings — beyond
even the widened 15–30 test band (the sim run above hit 32, one above the
gate's ceiling). Not a bug — the era test only exercises `balanced_spend` —
but worth flagging: a player who reads "Ward saves the most souls" and leans
all-in may run a noticeably longer era than the stated "~3 real months."
Whether that's an acceptable skill/patience tradeoff or worth a soft cap is
exactly the kind of design call this dossier surfaces, not resolves — see
Open Questions.

## Fun Assessment

*2026-07-04, re-scored against the seven Act 1 benchmarks after the
strategy-sweep evidence above [+ played session — see below]:*

| # | Heuristic | Score | Evidence |
|---|---|---|---|
| 1 | Visible next goal | 4/5 | Gate checklist + fuel bar pre-launch; next-beat timers, watch forecast, district thresholds in-voyage. Still missing an era-level projection ("~N crossings left at this rate") — that gap is unrelated to this session's builds, carried forward. |
| 2 | Wall → reset → power | 5/5 | The ferry loop is a true earned ramp (37→8 days, 180→4,500+ hold). The dark's toll is a per-day, always-visible, always-engaged number on the Reckoning screen, materially diverging by policy (70.5% vs 94.3%). Resistance is legible. |
| 3 | Discovery cadence | **4/5 (was 3)** | World milestones (this session) genuinely fix the flagged gap: the ferry era now reveals *two* independent axes of new content — districts (colony growth) and world milestones (old-world decline) — and because milestones key off `souls_remaining` rather than population, they land on different crossings for different spend policies, so the sequence of "what's new" isn't identical run to run. Held at 4 rather than 5 because both axes are still text-only log moments, not new mechanical levers, and the maiden voyage's much richer discovery density (weather, nights, souls, rumors, refits, letters) isn't matched in kind. |
| 4 | Cross-system braiding | 3/5 | Unchanged: launch gate braids all of Act 1 into the burn (excellent); voyage itself remains a deliberate island. Confirmed intentional, not re-litigated. |
| 5 | Decision density | 3/5 | Maiden voyage unchanged. Ferry runs: still one choice per ~3 real days, but the choice got richer — three options instead of two, each with a live before→after delta shown, not just a level-up button. Density is the same; the *quality* of that single decision improved, which the 1-5 scale doesn't capture well — noted here rather than inflating the score. |
| 6 | Anticipation instruments | 5/5 | Unchanged, still the act's strongest suit. |
| 7 | Stakes and texture | 4/5 | Stakes are less soft: Hope's "pinned at max, nearly no stakes" problem is gone, replaced by a toll that's always live and always differentiates skilled from careless play. The launch transition (this session) also adds a beat of ceremony to the act's single biggest moment that was previously a bare confirmation screen — a small texture win at the *start* of the act, distinct from the ferry-era stakes question. What's still missing: the toll and the milestones are numbers/log lines, not scenes — nobody the player has met is ever named as lost to the dark (that stays authored-only, per the `mark_lost()` covenant, and is a deliberate boundary, not a gap). |

**Where Act 2 deliberately breaks Act 1's patterns** (confirm, don't "fix"):
wall-clock instead of ticks; no failure states; no RNG in outcomes; the
voyage severed from Act 1 systems. All four are stated intent in the specs
and untouched by this commit.

## Open Questions & Decision History

Carried forward, still genuinely open (see `docs/decisions.md` for
outcomes once resolved):
1. ~~Hope gauge never engages — tune, redesign, or demote?~~ **Resolved by
   d39ad67**: retired entirely, replaced by the Ward yard. Logging this
   retroactively in `docs/decisions.md` this refresh since it was never
   logged when shipped.
2. ~~Post-maiden discovery drought — accept, add per-crossing beats, or new
   mid-era noun?~~ **Resolved 2026-07-04, then revisited same day**: first
   answered "accept as intentional, no new noun planned" — then the
   designer asked for actual construction work rather than another round of
   decisions, so world milestones (`WorldMilestone` in `colony.rs`) were
   built after all: a second discovery axis (the old world's decline, not
   the colony's growth) that a spend-policy choice actually moves around in
   the era. See `docs/decisions.md` for both entries, in order.
3. ~~Era length 24 vs intended ~19 crossings — retune or re-state intent?~~
   **Resolved**: intent re-stated as a range (~19–24, test gate 15–30)
   rather than constants pulled to hit one number.
4. ~~Launch transition is one static screen where spec 4 designed 5 beats —
   build or keep?~~ **Resolved 2026-07-04, then revisited same day**: first
   answered "keep the single screen" (low payoff-per-effort) — then built
   after all under the same later build directive as #2 above. See
   `src/vessel/transition.rs` and `docs/decisions.md`'s two entries.
5. ~~The Ward-lean policy is the highest-souls-saved line (94.3%) but runs
   ~32 crossings / ~5 months — beyond the era's own stated "~3 months" and
   the test's 15–30 band. Intended branch, or nudge Ward's cost curve?~~
   **Resolved 2026-07-04**: keep it as an intended "go slower, save more"
   branch — see the restated intent language in `src/vessel/CLAUDE.md`
   (now "~3 real months" for a balanced spend, "~5 real months" as the
   deliberate Ward-heavy line). Skill/patience tradeoff is fine; the margin
   stays wide and legible.

No open design questions remain from this refresh. All five were resolved
2026-07-04 (two of them — #2 and #4 — resolved twice: once by direct
designer answer, then revisited under an explicit "build it" directive the
same session); see `docs/decisions.md` for the full sequence. The next
refresh should re-check the decision retrospective on #3/#5 (era length)
once there's evidence of how the ferry era plays for a real session rather
than only the sim, and on #2/#4 (discovery beat, launch transition) once
there's played evidence of how the newly-built content actually lands.

Held for a later round (not yet asked, unchanged):
- Should Records/keepsakes surface anywhere outside the arrived harbor
  (e.g. title screen), given the era ends and the files remain?
- No player-facing wiki page exists for Act 2 (correct while dark-shipped;
  becomes a launch-checklist item).

**Resolved 2026-07-04 (full spec-alignment pass)**: every one of the 15
`docs/superpowers/specs/` vessel docs was audited against shipped source
and annotated with "Doc-alignment note" call-outs wherever stale — the
Hope retirement (present in nearly all of them), the abandoned
combat/crew/rooms-stats specs (confirmed nothing shipped, not just
"superseded"), the mode-transition spec's continuous-distance shell
(confirmed abandoned) vs. its 5-beat launch transition (built this round —
see below), and a scatter of smaller numeric drift (road count, district
thresholds, era-length estimates). The spec tree is now internally
consistent with `src/vessel/CLAUDE.md` as ground truth; future Act 2
changes should keep appending "Doc-alignment note" or "Follow-up" blocks
to the relevant spec rather than editing history, matching the pattern
this pass established.

Factual drift found and fixed this refresh: `src/vessel/CLAUDE.md`'s "Known
Invariants" section described a `game_state.rs` 100k/250k PR doc-comment
mismatch that was already fixed in source (presumably by the prior
refresh's own fix, or an intervening doc-audit pass) — the CLAUDE.md note
itself just hadn't been updated to say so.
