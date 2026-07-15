# Act 1: The Ascent — Design Dossier

> Last refreshed: 2026-07-08 @ ee708f3 (doc-audit: fixed stale constants;
> no source changed) | Sources: `src/core/` (incl.
> `power_rating.rs`), `src/combat/`, `src/character/`, `src/zones/`,
> `src/items/`, `src/enhancement/`, `src/ascension/`, `src/deep/`,
> `src/loom/`, `src/power_cores/`, `src/god_items/`, `src/haven/`,
> `src/stormglass/`, `src/fishing/`, `src/dungeon/`, `src/achievements/`,
> `src/challenges/`, `src/history/`, all 20 `openspec/specs/` capabilities,
> `openspec/README.md`'s discrepancy log, `docs/decisions.md`,
> `docs/storyboards/act1-the-ascent.html`, and simulator runs
> (`--check-progression`, three-strategy sweeps at a P300 baseline)

> **Status: new.** This is the first dossier for Act 1 — the whole 50-zone
> climb and every system hanging off it. `openspec/specs/` and the per-module
> `CLAUDE.md` files are the living per-capability truth; this dossier is the
> cross-system, player-eye synthesis the capability specs don't hold on their
> own — how the 20 capabilities feel as one arc, where they brace each other,
> and where they quietly don't. Act 2's experiential design has its own deep
> dossier in [`act2-pilgrimage.md`](act2-pilgrimage.md); the cross-act
> through-line lives in [`world-and-narrative.md`](world-and-narrative.md).

## The Player's Experience

A fresh hero starts at Level 1 in Zone 1 ("Meadow"), Subzone 1, and the game
begins doing its thing without being asked: a fast, steady tick loop
auto-fights whatever's in front of the hero, banking XP with every kill. A
run of kills spawns the subzone boss; the boss down, the zone frontier
pushes forward. There is no failure state that costs anything durable — a
mob death just retries the same fight until it's won or the player retreats
after a few losses in a row, and a boss death retreats to the highest zone
with a defeated boss, never punishing progress already banked. The felt
shape of the first hour is pure ascent: level up, gear drops, numbers go up,
the frontier zone advances.

Ten zones in, Zone 10's final boss won't take a hit at all — the game
flatly refuses damage until the player has forged the **Stormbreaker**, and
that quest reaches sideways into two other systems the player has been
building in parallel: fish to the maximum rank, hunt the Storm Leviathan
across a run of encounters whose odds deliberately rise and fall rather
than cleanly decreasing (a narrative arc, not a clean probability curve),
then build the Storm Forge capstone in Haven — which itself requires both
the War Room and Vault branches complete — before the forge action
unlocks. It's the single most cross-system demand Act 1 makes of the
player before its own halfway point.

Past Zone 10 comes **the Expanse** — an infinite eleventh zone that cycles
forever ("The Endless" / boss "Avatar of Infinity") until the player has
somewhere else to go. The first Expanse-cycle boss kill past a prestige
floor is the deterministic (non-RNG) trigger that unlocks **The Deep** —
mercenary expeditions that run on the wall clock, not the tick, so for the
first time part of the game keeps moving while the terminal is closed.
From here the shape of play forks: **prestige** resets level, attributes,
and equipment back to zero in exchange for a permanent, ever-climbing
multiplier (see Design Intent) and unlocks a cascade of gated systems —
Haven, Soulforge enhancement, the Deep, Ascension (Deep-layer gated, then
pattern-gated), the Loom (pattern + Ascension + prestige triple-gated),
Fracture zones (Deep-layer **and** prestige dual-gated), and Loom zones
(all three axes at once). None of these are single unlocks so much as
parallel frontiers that keep receding as the player invests in them — the
Deep's Layers, the Loom's completable Woven Patterns (plus one eternal one
that never completes, by design), Ascension's ten tiers, Fracture's
regions, Loom-zone's chapters.

Around this all sits the meta-layer: a wide roster of challenge minigames
discovered passively provides the game's only genuinely *active*,
skill-based moments against otherwise idle mechanics; a large slate of
achievements and unlockable titles track milestones across the whole
account; the git-backed Time Vault silently snapshots the save at every
major beat so nothing is ever truly lost. Dungeons, discovered independently
of prestige, interleave procedurally-generated side content with a boss-key
gate. By the endgame the "climb" reframes twice more — the Deep is a
*descent* through numbered Layers, and clearing Zone 50 (Loom's final
chapter) surfaces a signal from a dying branch of Yggdrasil that becomes Act
2's launch gate: the Loom's patterns, Ascension's top tier, and a huge
one-time burn of Prestige Ranks, spent in a single action. Act 1 never stops
being about *more* — more zones, more gear, more multipliers stacking on
multipliers — right up to the moment it hands the player something to burn
it all on.

## Design Intent

Act 1's own design record is scattered across `docs/decisions.md` rather
than one design doc — it predates OpenSpec and the dossier format, so this
section reconstructs intent from the decisions actually made:

- **"The golden ratio"** (`docs/decisions.md` "Balance Philosophy: Active
  Play ~2-3x Idle, Endgame in Weeks Not Hours") — the single named principle
  behind every other pacing decision in this section: active decisions
  (prestige timing, minigames, Haven) should meaningfully outpace pure idle
  play, no hard walls (progress slows but never stops), and every prestige
  should feel like a genuine power boost. A named milestone-feel table gives
  it teeth, tuned so each early milestone lands within a session and later
  ones stretch to weeks — "I get it now" at first prestige, "New system!"
  at Haven, "Finally!" at Stormbreaker, "One more run" once the Expanse
  cycles forever — the same shape the Balance Evidence section below
  measures against. The same decision also codifies **danger zones** (the
  core tick/XP/level constants and every progression gate — no edits
  without simulation) versus safe-to-tune levers (fish weights, enemy
  names, affix ranges, room types, UI).
- **10 zones, not 20.** The original plan was 20 zones across two eras
  ("Planar Journey" at Zones 11-20 with weapon-forging gates per zone). The
  shipped design compresses this to 10 authored zones + the infinite Expanse
  for endless replay without needing 10 more authored zones, plus the
  Stormbreaker quest as a single satisfying endgame gate instead of a
  forging chain repeated per zone (`docs/decisions.md` "Zone Count: 10 vs
  20", "Zone Progression Design: Competing Proposals"). Fracture (12-30) and
  Loom (31-50) zones were added later as post-prestige frontiers, not part
  of this original two-era plan.
- **Sub-linear prestige, deliberately.** Several formulas were compared,
  ranging from one that "trivializes everything" within the first few
  prestiges to one that "still runs away" further out; the shipped curve
  was chosen specifically because it asymptotes toward a modest ceiling
  instead, preserving a "wall → reset → power fantasy" loop at *every*
  stage rather than letting late prestiges become trivially fast
  (`docs/decisions.md` "Prestige Multiplier Formula"). This is the same
  heuristic Act 2's dossier scores against — it originates here.
- **Equipment wipes completely on prestige** so each cycle is a genuine
  reset, with the Haven Vault room as an earned, bounded exception rather
  than a default (`docs/decisions.md` "Equipment Reset on Prestige").
- **Stormbreaker as a quest chain, not a drop.** A pure-RNG legendary drop
  for a hard progression gate "feels bad (no agency)"; the shipped
  fishing→Haven→forge chain braids three systems together and gives the
  player something to plan toward over its intended ~month-scale pace
  (`docs/decisions.md` "Stormbreaker: Drop vs Forge").
- **Haven bonuses changed shape from design to implementation** more than
  once — War Room became Double Strike instead of attack-interval reduction
  specifically to avoid touching tick timing; the Fishing Dock became
  Double-Fish-chance + max-rank increase instead of a flat XP boost, to
  extend the fishing system's own depth (ranks 31-40) rather than just
  accelerating it (`docs/decisions.md` "Haven Bonus Types: Design vs
  Implementation").
- **JSON over binary saves**, chosen for debuggability and because save
  files are small enough that binary encoding buys nothing — serde's
  structural validation is treated as sufficient, though this is also the
  root of the silent-wipe hazard flagged below (`docs/decisions.md` "Save
  Format: Binary vs JSON").
- **Challenge discovery weights and AI choice per game** were tuned
  explicitly for accessibility ordering (Rune/Minesweeper most common, Go/
  Chess rarest) and per-game algorithm fit (MCTS for Go's high branching
  factor and absent eval function, minimax+alpha-beta elsewhere) —
  `docs/decisions.md` "Challenge Discovery Weights", "AI Algorithms Per
  Game". The weight table has been revised twice since (6→10→14 games; see
  Open Questions).
- **The recurring motifs** `world-and-narrative.md` names for the
  cross-act arc — rebirth cycles, the Storm as threshold/reward, "the
  endless" as both engine and dread — are Act 1's own vocabulary first; Act
  2 inherited and re-purposed them, not the other way around.

## Mechanics & Constants

### Tick loop & time
A fixed, short tick — not a wall-clock delta — drives everything, so play
speed never drifts with framerate or scheduling hiccups.

- Autosave and the update-checker each run on their own independent timers.
- The update-checker's timer is randomly jittered so many clients don't
  all check in at once.
- A stale comment describing that jitter overstates how long it actually is.

### Combat pipelines
Player damage builds in a fixed order, and incoming damage runs the mirror
version on the way in:

- Base hit → percentage bonuses (equipment, Haven) → a flat prestige bonus
  → the Ascension multiplier → the enemy's defense subtracts → a crit,
  and occasionally a second "double-strike" hit, can land on top.
- Different enemy roles attack at different speeds; the player is quicker
  than most of them — the zone boss matches its pace, and the dungeon boss
  just edges it out.

Death is handled differently depending on where it happens:

- A dungeon death costs nothing and exits cleanly.
- Dying to a mob just retries the same fight until it's won or the player
  retreats after a few losses in a row.
- Dying to a boss retreats to the last zone with a defeated boss rather
  than the current one — described elsewhere as simply "resets to the
  start," which isn't quite what happens (a known documentation gap).
- A boss fight that drags on too long without resolving forces a loss on
  an enrage timer.
- A previously undocumented mechanic: a mob fight that stalls out
  entirely auto-retreats on its own, with no death recorded at all — this
  exists in neither the combat spec nor the root project docs.

**Frontier Backoff** is a second-order fix for a problem the retreat rule
above creates on its own: retreating from a death sends the player back
into the zone they just cleared, and re-beating that zone's boss
auto-advances straight back into the zone that killed them — a death loop
wrapped around the death-loop guard. Frontier Backoff detects the pattern
and makes boss-defeat advancement cycle the safe zone instead of walking
straight back into danger, with a cooldown that grows the more it happens
and clears on the next real win. It's the game's answer to the single
hardest edge of the climb — the frontier — existing specifically because
the simpler rule isn't safe against itself.

**Power Rating** is the single number the game reduces all of the above
to: a geometric mean of effective damage-per-second and effective HP,
folding in every bonus source at once — equipment, enhancement, prestige,
Haven, god items, sigils, Ascension. It's cached and rendered permanently
in the stats panel header — the closest thing in the game to a literal,
always-on-screen expression of "power in one place" (see Design Intent
and Fun Assessment). The item-level power *score* used for auto-equip
decisions (see Items, below) is a related but distinct, narrower number —
a common point of confusion since both get called "power."

### XP & leveling
- Experience comes only from kills, in a wide-enough random range per
  kill that grinding doesn't feel metronomic.
- Scaled by the character's passive rate (the prestige multiplier plus a
  small Wisdom bonus) and any Haven XP bonus.
- The level curve gets steeper than linear, so later levels take
  meaningfully longer than early ones.
- Each level grants a few attribute points, and the cap on total
  attributes rises with prestige rank rather than staying fixed.
- Offline time still earns XP, at a reduced rate and with a hard cap.

### Character & prestige
Six attributes each drive one distinct combat lever:

- Strength and Intelligence add flat damage.
- Constitution adds HP.
- Dexterity adds defense and crit.
- Wisdom adds XP gain.
- Charisma sweetens the prestige multiplier a little further.

Prestige wipes the character back to a fresh start — level, XP,
attributes, and every equipped item — while leaving account-level
progress alone: achievements, Haven, fishing rank, Ascension, Stormglass,
the Deep, the Loom, and Soulforge enhancement all survive. It resets the
*character*, not the *account*.

The prestige multiplier follows the sub-linear curve described in Design
Intent, applied to XP gain; separate, smaller bonuses tied to prestige
rank layer flat damage, defense, crit, and HP directly into the combat
pipeline on top of it. One quirk worth knowing: the cosmetic *tier names*
attached to prestige rank (Bronze, Silver, ... Eternal) stop advancing
well before the multiplier does — "Eternal" is the name for every rank
from the mid-20s onward, even though the game's own late-game content
expects ranks in the tens of thousands.

### Zones
Fifty authored zones fall into three bands with three different gating
philosophies:

- **Zones 1-10, the core climb** — subzones, a boss every so many kills,
  unlocked a couple at a time as prestige rank rises.
- **Zone 11, the Expanse** — an infinite pressure-release valve that
  cycles forever once unlocked, standing between the core climb and
  everything after it.
- **Fracture zones (12-30), dual-gated** — a Deep-Layer threshold and a
  separate prestige-rank floor both have to clear before a band opens
  (only the Deep-Layer half of that gate is documented elsewhere — a
  known gap).
- **Loom zones (31-50), triple-gated** — completed Woven Patterns, an
  Ascension tier, and a prestige floor all have to clear together, across
  five chapters.

Enemies scale noticeably harder per zone in the Fracture band than in the
Loom band (1.6x per zone against Loom's 1.25x) — the Fracture climb is
the steeper of the two post-prestige frontiers.

### Items
- Item level tracks the zone it dropped in directly, so it's always
  readable at a glance without doing math.
- A separate quality roll (tier) is fully independent of rarity — a
  Common item can roll the same top tier a Legendary can, and vice versa,
  so a "lucky" common drop can genuinely rival an "unlucky" epic one.
- Mob drop chance rises slowly with prestige rank and caps out well short
  of a coin flip.
- Only bosses guarantee a drop, and only bosses can roll the very rarest
  tier at all.
- Equipment fills 7 slots.
- Auto-equip compares a computed power score — attributes plus weighted
  affixes — and only swaps gear on a strict improvement.
- God items are protected from being swapped out by a rarity check rather
  than by that power score, because God-item power scoring itself is
  still an open gap.

### Enhancement (Soulforge)
- Equipment enhances from +0 up to a hard cap, one slot at a time.
- Success chance holds at 100% for the early levels, then drops in two
  more steps as the level rises.
- A failed roll at the higher levels can downgrade the enhancement
  instead of just doing nothing.
- A parallel "Soul Tithe" path buys a guaranteed success at the higher
  levels instead of gambling — priced, by the design's own admission,
  directly against the expected cost of gambling that step repeatedly,
  not a round number picked by feel.
- Discovery gates a little later than Haven's, on the same
  discovery-chance shape.

### Ascension
- Ten tiers of permanent combat multiplier, paid for in Prestige Ranks.
- The first six tiers gate on Deep-Layer progress and roughly double the
  multiplier each time.
- The last four re-gate on completed Woven Patterns instead and shift to
  a shallower, slower-compounding curve — Ascension is the hinge where
  Deep-driven power gives way to Loom-driven power.
- The multiplier is recomputed fresh at three separate points in the
  combat pipeline each tick (damage, defense, max HP) rather than cached
  once — cheap, but worth knowing if only one of those three points is
  ever touched in isolation.

### The Deep
- The only discovery in the game with **no RNG roll at all** — it
  unlocks deterministically the first time the player clears the
  Expanse's cycling boss past a prestige floor.
- Layers deepen through named tiers (Shallows through Void); guild rank
  gates roster size and how many missions can run at once.
- Missions run on the wall clock rather than the tick — the game only
  checks for missions that have finished, the same way it resolves
  offline time.
- Deep state — roster, missions, marks, layer records — survives
  prestige entirely; only a generation counter advances, despite a stale
  doc elsewhere claiming otherwise.
- The single longest mission, unlocked only at the deepest layer, runs
  for several real days.

### Loom
- A production network with 28 completable "Woven Patterns" plus one
  permanent, uncompletable "eternal" pattern — a deliberate design choice
  to keep the network meaningful after the completable content runs out,
  not a leftover.
- Output converts into Prestige Ranks through a self-multiplying formula
  (the more you're producing, the more each unit is worth), but only once
  every completable pattern is done.
- Loom zone unlocks share the same triple-gate described above under Zones.

### Power Cores
- Six passive Prestige Rank generators, unlocking one per Deep Layer
  milestone.
- Each pays out on its own fixed cycle rather than a continuous drip — a
  freshly-unlocked core deliberately pays nothing until its first full
  cycle completes, live or offline alike.
- Their combined total is simply the sum of whichever cores are
  unlocked, not an enforced cap, even though the normative spec's own
  requirement title reads more like one.

### God Items
- Three fixed, top-rarity Norse-themed artifacts, each in a specific
  equipment slot and built around one strong passive: armor that shrugs
  off a flat share of damage, boots that double attack speed, a ring
  that meaningfully boosts damage.
- Deliberately no player-facing way to earn one; the only path is a
  debug action.
- A quirk worth knowing: the item that reads as a belt in its own lore
  actually occupies the Ring slot, since the item system has no belt
  slot at all.

### Haven
- An account-level (not per-character) home base with two branches — one
  combat-flavored, one quality-of-life — plus a capstone that needs both
  branches finished.
- Discovery unlocks around mid-prestige, on a chance that climbs slowly
  with rank.
- Bonuses are computed once per tick and threaded explicitly through
  function parameters rather than read from global state, deliberately
  keeping unrelated systems from ever needing to reach into Haven directly.
- Its rooms cover a wide spread — a chance at a second hit per swing,
  better fishing odds and a higher rank cap, and a small number of
  equipped items preserved across prestige.
- Forging the Stormbreaker itself needs the capstone room built *and*
  the Storm Leviathan caught *and* a second, independent prestige floor
  — a triple-lock beyond whatever it cost to build the tree in the first
  place.

### Stormglass
- A per-character soft currency earned by salvaging drops instead of
  equipping them, unlocked a little later than Haven.
- Funds a small set of equippable "Sigils" that rotate daily on a fixed,
  seeded schedule (keyed to the real calendar date, not local time).
- A separate consumable temporarily accelerates the tick rate.
- A separate item nudges the odds of a rare fishing encounter upward —
  it is **not** a guarantee, despite two independently-drifted docs
  (one of them the normative spec) claiming otherwise; the fishing
  system's own spec has it right.

### Fishing
- Discovered on a flat per-kill chance with no prestige gate.
- Forty ranks across eight named tiers, with a base cap that Haven's
  Fishing Dock room raises further.
- Catch rarity odds shift toward rarer fish as rank climbs.
- At the maximum rank, legendary catches trigger the ten-encounter Storm
  Leviathan hunt described in Player's Experience — landing it is what
  unlocks Stormbreaker forging.

### Dungeon
- Discovered on its own flat per-kill chance, independent of prestige,
  and mutually exclusive with that same kill's fishing-discovery roll.
- Layouts are procedurally generated with the occasional extra loop for
  variety; exactly one entrance, one elite, and one boss room each, plus
  a scaling number of treasure rooms by dungeon size.
- Beating the elite grants a boss key exactly once, and the explorer
  routes around the boss room until it's held.
- Elite and boss enemies both hit noticeably harder than a standard
  encounter — a separate, unused helper still defines a different,
  weaker pair of multipliers that no live code path actually calls.

### Achievements, Challenges, Time Vault
- Achievements span many categories and point tiers, with prestige-rank
  and level milestones layered on top, plus a large set of unlockable
  titles — a couple of stale figures survive in older docs for both the
  title count and the milestone count.
- Fourteen challenge minigames — from classic board games (Chess, Go,
  Gomoku, Nine Men's Morris) to original puzzle and action games — each
  with four difficulty tiers (Novice through Master) and the same
  two-step-Esc forfeit pattern throughout.
- Discovery weights favor the more accessible games heavily; Chess and
  Go are the rarest by design.
- Each game uses whatever AI approach actually fits it — minimax-family
  search for the classic board games, Monte Carlo tree search for Go
  specifically, since its branching factor defeats a simple evaluation
  function.
- The git-backed Time Vault snapshots the save at every major milestone
  (never on the routine autosave), never auto-prunes, and even a
  restored-over snapshot is still recoverable underneath the game's own UI.

### Persistence
- Saves are plain JSON, resolved through an overridable directory.
- Character files fail loudly on a parse error, surfacing as a visibly
  "corrupted" entry rather than silently vanishing.
- **Account-level files do the opposite** — Haven, achievements, the
  Deep, the Loom, and enhancement state each fall back to an empty
  default on any parse error, with nothing surfaced to the player.
- This is called out explicitly as a load-bearing hazard by the
  regression test that guards it — the only thing standing between a
  save-format change and a silent, invisible account wipe.

### Discovery cadence, compared
| System | How it unlocks |
|---|---|
| Haven | rank-gated, per-tick chance |
| Soulforge | same shape as Haven, unlocks a bit later |
| Challenges | flat per-tick chance, boosted by Haven |
| Fishing | flat chance per kill, no prestige gate |
| Dungeon | flat chance per kill, no prestige gate |
| The Deep | no roll — a deterministic trigger |

Interestingly, Haven's, Soulforge's, and Challenges' base rates are the
exact same literal value, independently declared in three unrelated
places rather than one shared constant — a future balance pass touching
"the" discovery rate would need to know to find all three by hand.

## Interrelations

```
Combat (kills, tick loop) ──► XP/Level ──► Attribute points
        │                                        │
        ├─► Item drops ──► Equip/score ──► combat power
        │                                        │
        └─► Prestige (reset char, +multiplier, +flat combat bonuses)
                    │
        ┌───────────┼──────────────────────────────────────┐
        ▼           ▼              ▼              ▼        ▼
      Haven     Soulforge      Stormglass     Challenges  Fishing
   (rank-gated) (rank-gated) (rank-gated, salvage) (rank-gated) (per kill)
        │           │                                       │
        │           │                              Storm Leviathan (max rank)
        │           └── PR spend ◄── Ascension ◄── Deep patterns ──┐
        │                              │  (Deep layer + pattern    │
        │                              │   gates)                 │
        └── Storm Forge (needs War Room+Vault) ───────────────► Stormbreaker
                                                                    │
                                                        Zone 10 boss gate
                                                                    │
       Zone 11 Expanse (infinite) ──► first cycle-boss kill ──► The Deep
                                                                    │
                              Deep Layers ──► Fracture zones (dual-gated)
                              Deep Layers ──► Power Cores (passive PR)
                                                                    │
                              Loom patterns ──► Ascension VII-X
                              Loom patterns ──► Loom zones (triple-gated)
                              Loom WR ──► PR (self-multiplying conversion)
                                                                    │
        Zone 50 clear + 28 patterns + Ascension X + 250k PR ──► Vessel launch (Act 2)
```

- **The tightest loop**: prestige is the hub every other system either feeds
  (kills→XP→level→prestige) or spends from (PR funds Haven, Soulforge,
  Ascension, and — via Loom's WR→PR conversion — indirectly funds itself).
  No other system in the game has this many inbound and outbound edges.
- **Stormbreaker is the strongest *authored* cross-system braid**: it's the
  only place the game deliberately requires three unrelated systems
  (fishing, Haven, prestige spend) to converge on one gate, rather than
  letting the natural prestige hub handle it.
- **The Deep sits at a genuine fork**: it's the only discovery with no RNG
  roll at all, and its Layers feed *two* independent downstream systems
  (Fracture zone unlocks and Power Core PR generation) that don't otherwise
  interact with each other.
- **God Items are a dead end by design**: they have combat effects but no
  acquisition path, no power-formula integration, and no other system reads
  their state — confirmed intentional (spec: "no procedural drop, discovery
  roll, or player-facing unlock gate produces a God Item"), but worth
  naming as the one system in this diagram with no outbound or inbound
  edge to anything else.
- **Out**: the Vessel launch gate is Act 1's single outbound edge to Act 2 —
  see `act2-pilgrimage.md`'s Interrelations section for the other side of
  that braid.

## Balance Evidence

*2026-07-05, `cargo run --release --bin simulator -- --check-progression`
(passes; three scenarios × multiple seeds each) plus three ad-hoc 100-real-
hour sweeps (`--ticks 3600000 --prestige 300 --stormbreaker`, seed 1) across
the three built-in strategy profiles, all at HEAD (2cf51d6):*

| Scenario (from `--check-progression`) | Gate | Result |
|---|---|---|
| early-game, 2h, casual, 5 seeds | Zone 2 by 45min, Level 20 by 1h, 300+ kills, 15+ boss kills, 50+ drops, ≤20 deaths, 5+ achievements | All pass with comfortable headroom (e.g. Level 20 actual ~13-15min against a 1h gate) |
| prestige-economy, 6h, optimal, 3 seeds | 3+ prestiges, 15+ PR earned, PR income covers spend, 8+ challenges won, 6+ Haven rooms, 12+ achievements | All pass (14-15 prestiges, 28-31 PR, 15-17 challenges) |
| endgame-systems, 30h, P200 baseline, speedrun, 2 seeds | Deep L25+, Fracture Z27+, 4+ patterns, Loom Z31+, Ascension I+, Level 100+ | All pass (Deep L30, Fracture Z30, 6 patterns, Loom Z34, Asc II, Level ~1050-1090) |

| Strategy (100h, P300 baseline) | Level | Prestige count | Fracture cap | Patterns | Loom cap | Ascension | Challenges won |
|---|---|---|---|---|---|---|---|
| Casual | 1,526 | 113 | Z20 | 6/28 | Z34 | II | 99 |
| Optimal | 347 | 28 | Z26 | 8/28 | Z38 | IV | 299 |
| Speedrun | 3,509 | 298 | Z30 (cap) | 10/28 | Z38 | IV | 599 |

Thresholds carry ~2x headroom by design (the tick loop isn't perfectly
deterministic), so these are conservative floors, not tight tuning
targets. The strategy spread is the more interesting signal: optimal
(which spends heavily on Haven/enhancement) prestiges the *least* of the
three and yet reaches the *highest* pattern count relative to its
prestige spend, while speedrun's raw prestige-count advantage (298 vs. 28)
doesn't translate into proportionally more Loom patterns (10 vs. 8) —
consistent with patterns being Deep/Loom-infrastructure-gated rather than
purely prestige-gated. None of the three built-in profiles reach Loom zone
40+ or Ascension VII+ within 100 real hours from a P300 baseline, which
matches the project's own framing of those as genuine late-game targets
rather than 100-hour ones.

## Fun Assessment

*2026-07-05, scored against the same seven heuristics `act2-pilgrimage.md`
uses (these originate from Act 1's own benchmarks, per
`world-and-narrative.md`'s "Design guardrails" section) — scored honestly
against Act 1 itself, not retrospectively through Act 2's lens:*

| # | Heuristic | Score | Evidence |
|---|---|---|---|
| 1 | Visible next goal | 4/5 | Zone/subzone frontier, boss-kill countdown (10 kills), level bar, and Deep/Loom layer and pattern counters are all live, numeric, and always on screen. Docked one point because most *discovery* gates (Haven, Soulforge, Challenges, Fishing, Dungeon) are invisible per-tick RNG rolls with no meter — the player has no signal they're "close" to a Haven discovery the way Act 2 shows a fuel bar for its own gate. |
| 2 | Wall → reset → power | 5/5 | This is the heuristic Act 1's own prestige design was built around (see Design Intent) — the sub-linear formula was chosen specifically to keep this loop honest at every scale, and the simulator evidence above shows it still holds 100+ hours in (casual reaches 113 prestiges without the loop going slack). |
| 3 | Discovery cadence | 5/5 | Six largely-independent discovery axes (Haven, Soulforge, Challenges, Fishing, Dungeon, the Deep) each unlock a genuinely different kind of content, several running concurrently at different prestige floors — more simultaneous discovery axes than any other system in the game, including Act 2. |
| 4 | Cross-system braiding | 5/5 | The strongest in the game by a wide margin (see Interrelations) — prestige is a hub with edges to nearly every other system, and Stormbreaker is a deliberately-authored three-system quest chain. This is the one heuristic Act 1 was explicitly built around and Act 2 was explicitly built to feel *different* from. |
| 5 | Decision density | 2/5 | The core loop is idle-first by design (see Design Intent's guardrail) — combat, retreats, and most discoveries happen with zero player input. Real decisions cluster in three places: spending PR (Haven room, Ascension tier, Enhancement roll, or bank it), the 14 active challenge minigames, and equipment/Haven build order — meaningful, but sparse relative to the hours of pure auto-battle between them. |
| 6 | Anticipation instruments | 2/5 | By Act 2's own design thesis, quoted in its dossier: "Act 2's fun is anticipation, choice, people, and consequence — the kinds of fun Act 1 never touched." Act 1 has countdown timers (boss-in-N-kills, mission-in-N-hours) but nothing like Act 2's fuel bar, watch forecasts, or named-arrival scenes. This is an intentional gap, not an oversight — Act 1 specializes elsewhere. |
| 7 | Stakes and texture | 2/5 | Deaths cost almost nothing (a mob death just retries; a boss death retreats without XP/item loss) and prestige is a *chosen* reset, not an imposed one — there is no permanent-loss mechanic anywhere in Act 1 comparable to Act 2's "doors close, souls lost stay lost." Texture-wise the world is richly named (Storm Citadel, the Black Mouth, Threadbare Wastes) but that richness is almost entirely environmental flavor text, not consequence. |

**Where Act 1 and Act 2 deliberately diverge** (confirm, don't "fix"): Act 1
is explicitly "power in one place" — unlimited growth, engines, loot, zero
failure states — while Act 2 is explicitly "passage" — bounded, consequence-
bearing, anticipation-first. The low scores on heuristics 5-7 above are not
gaps to close; they are the exact contrast the design thesis states Act 2
exists to provide. Reading the two dossiers side by side is the intended
comparison.

## Open Questions & Decision History

Surfaced during this dossier's research pass, not yet in `docs/decisions.md`:

1. **The 30s/60s mob-fight stalemate timeout has no spec coverage.**
   `MOB_FIGHT_TIMEOUT_SECONDS`/`DUNGEON_FIGHT_TIMEOUT_SECONDS`
   (`src/combat/orchestration.rs:49-62`) is a real, functioning mechanic
   (auto-retreat from an unwinnable fight without counting as a death) that
   appears in neither `openspec/specs/combat/spec.md` nor root `CLAUDE.md`.
   Worth a documentation pass; not a code bug.
2. **Storm Lure's "guarantee" error is duplicated, not singular.** The known
   discrepancy log only names `src/fishing/CLAUDE.md`; this pass found the
   identical wrong claim independently repeated in
   `src/stormglass/CLAUDE.md:30` and in `openspec/specs/stormglass/spec.md:
   55` itself — meaning the openspec baseline disagrees with its own sibling
   spec (`fishing/spec.md`, which has it right). Should be fixed in the same
   pass as the known fishing-doc item.
3. **Challenge discovery weights for the 4 newest minigames (Sudoku/Sigil
   Matrix, Shard Fusion, Runic Lights, Vault Warden) have no entry in
   `docs/decisions.md`.** The doc carefully documents the 6→10-game
   rebalance but stops there; the current 14-game table
   (`src/challenges/menu.rs:431-484`) has four weights with no logged
   rationale.
4. **God Item power scoring remains deferred** (tracked as issue #272 per
   `src/items/CLAUDE.md`) — the only thing preventing a God item from being
   auto-replaced by a higher-rolled non-God item is a hardcoded rarity
   check, not a power-formula floor. Low urgency while acquisition stays
   debug-only, but worth flagging before any player-facing acquisition path
   ships.
5. **Prestige tier *names* are cosmetically decoupled from the multiplier
   past rank 20** ("Eternal" repeating for thousands of ranks). Not a bug —
   just worth a deliberate decision on whether that's intended forever or
   a future title/flavor pass should add more rank-name texture at the very
   high end the Loom-zone endgame now reaches.

Carried forward from `openspec/README.md`'s existing discrepancy log
(re-verified current during this pass, not re-litigated): the boss-death
retreat description, the Fracture dual-gate omission in root `CLAUDE.md`,
the Loom "302 vs 303" PR/hr comment, the Deep's "24h" Gateway comment, the
achievements "29 titles"/"18 milestones" stale figures, and the God Item
belt-lore/Ring-slot naming mismatch. None of these need re-deciding here —
see `openspec/README.md` for the full list and `docs/decisions.md` for
anything with a resolved rationale.

## Sources

- Core systems: `openspec/specs/{game-loop,combat,character-progression,
  zones,items,enhancement,ascension,deep,loom,power-cores,fishing,dungeon,
  haven,stormglass,achievements,god-items,time-vault,persistence}/spec.md`
- Per-module implementation docs: `src/*/CLAUDE.md`
- Design rationale: `docs/decisions.md`
- Cross-act framing: [`world-and-narrative.md`](world-and-narrative.md)
- Known code-vs-docs drift: `openspec/README.md`
- Balance evidence: `cargo run --release --bin simulator --
  --check-progression`, ad-hoc strategy sweeps at this dossier's refresh sha
