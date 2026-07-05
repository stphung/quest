# Act 1: The Ascent — Design Dossier

> Last refreshed: 2026-07-05 @ 1fce9fc | Sources: `src/core/` (incl.
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
begins doing its thing without being asked: the 100ms tick loop auto-fights
whatever's in front of the hero, one kill every ~2s, banking 200-400 XP per
kill (`src/core/xp.rs:98-107`). Ten kills spawn the subzone boss
(`KILLS_FOR_BOSS = 10`, `src/zones/progression.rs:98-111`); the boss down,
the zone frontier pushes forward. There is no failure state that costs
anything durable — a mob death just retries the same fight (three in a row
triggers a retreat, `DEATH_LOOP_THRESHOLD`), and a boss death retreats to the
highest zone with a defeated boss, never punishing progress already banked
(`src/combat/orchestration.rs:116-200`). The felt shape of the first hour is
pure ascent: level up, gear drops, numbers go up, the frontier zone advances.

Ten zones in, Zone 10's final boss won't take a hit at all — the game flatly
refuses damage until the player has forged the **Stormbreaker**
(`src/zones/gates.rs:14-32`), and that quest reaches sideways into two other
systems the player has been building in parallel: fish to max rank 40, hunt
the Storm Leviathan across ten narratively-paced encounters (5%, 3%, 4%,
5%, 4%, 3%, 2%, 1.5%, 1%, 0.8% — deliberately not a clean monotonic curve,
`src/fishing/CLAUDE.md:125-129`), then build the Storm Forge capstone in
Haven (which itself requires both the War Room and Vault branches complete,
25 PR) before the forge action unlocks. It's the single most cross-system
demand Act 1 makes of the player before its own halfway point.

Past Zone 10 comes **the Expanse** — an infinite eleventh zone that cycles
forever ("The Endless" / boss "Avatar of Infinity") until the player has
somewhere else to go. The first Expanse-cycle boss kill at Prestige Rank 15+
is the deterministic (non-RNG) trigger that unlocks **The Deep**
(`src/core/tick_stages.rs:489-511`) — mercenary expeditions that run on the
wall clock, not the tick, so for the first time part of the game keeps
moving while the terminal is closed. From here the shape of play forks:
**prestige** resets level/attributes/equipment back to zero in exchange for
a permanent, ever-climbing multiplier (`1.0 + 0.5 × rank^0.7`,
`src/character/tiers.rs:64-67`) and unlocks a cascade of gated systems —
Haven (P10+), Soulforge enhancement (P15+), the Deep (P15+, Expanse-gated),
Ascension (Deep-layer gated, then pattern-gated), the Loom (pattern +
Ascension + prestige triple-gated), Fracture zones 12-30 (Deep-layer **and**
prestige dual-gated), and Loom zones 31-50 (all three axes at once). None of
these are single unlocks so much as parallel frontiers that keep receding as
the player invests in them — the Deep's Layers, the Loom's 28 completable
Woven Patterns (plus a 29th, eternal one that never completes, by design),
Ascension's ten tiers, Fracture's six regions, Loom-zone's five chapters.

Around this all sits the meta-layer: 14 challenge minigames discovered
passively (~2hr average, Prestige Rank 1+) provide the game's only genuinely
*active*, skill-based moments against otherwise idle mechanics; 240
achievements and 64 unlockable titles track a huge surface of milestones;
the git-backed Time Vault silently snapshots the save at every major beat so
nothing is ever truly lost. Dungeons (1% per-kill discovery, independent of
prestige) interleave procedurally-generated side content with a boss-key
gate. By the endgame the "climb" reframes twice more — the Deep is a
*descent* through numbered Layers, and clearing Zone 50 (Loom's final
chapter) surfaces a signal from a dying branch of Yggdrasil that becomes Act
2's launch gate: 28 patterns, Ascension X, and 250,000 PR burned in one
action. Act 1 never stops being about *more* — more zones, more gear, more
multipliers stacking on multipliers — right up to the moment it hands the
player something to burn it all on.

## Design Intent

Act 1's own design record is scattered across `docs/decisions.md` rather
than one design doc — it predates OpenSpec and the dossier format, so this
section reconstructs intent from the decisions actually made:

- **"The golden ratio"** (`docs/decisions.md` "Balance Philosophy: Active
  Play ~2-3x Idle, Endgame in Weeks Not Hours") — the single named principle
  behind every other number in this section: active decisions (prestige
  timing, minigames, Haven) should be ~2-3x more efficient than pure idle
  play, no hard walls (progress slows but never stops), and every prestige
  should feel like a genuine power boost. A named milestone-feel table gives
  it teeth — P1 in 30-60min ("I get it now"), Haven at P10 in 8-12h ("New
  system!"), Stormbreaker in 2-4 weeks ("Finally!"), the Expanse cycling
  forever ("One more run") — the same shape the Balance Evidence section
  below measures against. The same decision also codifies **danger zones**
  (`TICK_INTERVAL_MS`, `BASE_XP_PER_TICK`, zone/prestige requirements,
  `MAX_FISHING_RANK` — no edits without simulation) versus safe-to-tune
  levers (fish weights, enemy names, affix ranges, room types, UI).
- **10 zones, not 20.** The original plan was 20 zones across two eras
  ("Planar Journey" at Zones 11-20 with weapon-forging gates per zone). The
  shipped design compresses this to 10 authored zones + the infinite Expanse
  for endless replay without needing 10 more authored zones, plus the
  Stormbreaker quest as a single satisfying endgame gate instead of a
  forging chain repeated per zone (`docs/decisions.md` "Zone Count: 10 vs
  20", "Zone Progression Design: Competing Proposals"). Fracture (12-30) and
  Loom (31-50) zones were added later as post-prestige frontiers, not part
  of this original two-era plan.
- **Sub-linear prestige, deliberately.** Three formulas were compared —
  `1.5^rank` (57.7x by P10, "hyper-exponential, trivializes everything"),
  `1.2^rank` (still runs away by P30), and the shipped `1 + 0.5×rank^0.7`
  (asymptotes ~6-7x) — chosen specifically to preserve a "wall → reset →
  power fantasy" loop at *every* stage rather than letting late prestiges
  become trivially fast (`docs/decisions.md` "Prestige Multiplier Formula").
  This is the same heuristic Act 2's dossier scores against — it originates
  here.
- **Equipment wipes completely on prestige** so each cycle is a genuine
  reset, with the Haven Vault (1/3/5 items at T1/T2/T3) as an earned,
  bounded exception rather than a default (`docs/decisions.md` "Equipment
  Reset on Prestige").
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
`TICK_INTERVAL_MS = 100` (`src/core/constants.rs:2`), gated in `src/main.rs`
so the loop only advances on elapsed ≥100ms with a fixed 0.1s delta
regardless of scheduling drift (`src/main.rs:1405,1456,1486`). Autosave every
30s (`AUTOSAVE_INTERVAL_SECONDS`, `constants.rs:14`). Update checks jitter
10-20 real minutes (`main_helpers/update.rs:38-42`) — a stale inline comment
in `main.rs:1757` still says "~30 minutes."

### Combat pipelines
Player→enemy damage, in exact order (`src/combat/player_attack.rs:46-73`):
base damage → ×(1+Giant's-Might%) → ×(1+Haven damage%) → +flat prestige
damage → ×ascension multiplier → −enemy defense (floor 1) → crit roll (2x)
→ double-strike roll. Enemy→player defense (`src/combat/enemy_attack.rs:
36-46`): (base defense + flat prestige defense) × ascension multiplier →
subtract from enemy damage (floor 1) → Bulwark % DR if active. Attack
intervals: player 1.5s base, normal mob 2.0s, subzone boss 1.8s, zone boss
1.5s, dungeon elite 1.6s, dungeon boss 1.4s (`constants.rs:3,8-12`). Death
handling differs by context: dungeon death exits with no prestige loss and
full HP; overworld boss death retreats to the *highest zone with a defeated
boss* (not literally "subzone 1" as root `CLAUDE.md` says — a known
discrepancy, `openspec/README.md`); overworld mob death retries up to 3
times before retreating; a 60s boss-enrage timer forces defeat and retreats
to the *current* zone's subzone 1. A previously undocumented mechanic: fights
against mobs also auto-retreat after a 30s stalemate (60s in dungeons) with
no death recorded at all (`MOB_FIGHT_TIMEOUT_SECONDS`/
`DUNGEON_FIGHT_TIMEOUT_SECONDS`, `src/combat/orchestration.rs:49-62`) — this
exists in neither `openspec/specs/combat/spec.md` nor root `CLAUDE.md`.

**Frontier Backoff**, a second, higher-order safeguard the mob/boss retreat
rules above create a need for: retreating from a death sends the player
back into the zone they just cleared, where re-beating that zone's boss
auto-advances straight back into the zone that killed them — a death loop
around the death-loop guard itself (`docs/decisions.md`, issue #576).
`record_death_retreat()` (`src/zones/progression.rs:71-85`) tracks this and
`frontier_backoff_blocks()` makes boss-defeat advancement cycle the safe
zone instead of auto-advancing into the recorded death zone, with a
cooldown that grows on repeated retreats, capped at 8 cycles
(`FRONTIER_BACKOFF_MAX_CYCLES`, `constants.rs:108`) and clearing on any
boss kill in the recorded zone or on prestige. Named explicitly in
`src/zones/CLAUDE.md`, this is the game's answer to the exact hardest edge
of the climb — the frontier — and it exists specifically because the
simpler retreat rule alone isn't safe against itself.

**Power Rating**, the single number the game reduces all of the above (and
every other bonus source) to: `compute_power_rating()`
(`src/core/power_rating.rs`) folds equipment, enhancement, prestige, Haven,
god items, sigils, and ascension into one geometric mean, `sqrt(effective
DPS × effective HP)` — effective DPS re-derives the exact damage pipeline
above (crit factor, double-strike factor, hits/second), effective HP
divides max HP by `(1 - damage_reduction%)` so defense is worth more, not
less, in the aggregate. Cached on `GameState.cached_power_rating` and
rendered permanently in the stats panel header (`src/ui/stats_panel.rs:
181`) — this is the closest the game comes to a literal, always-on-screen
expression of "power in one place" (see Design Intent and Fun Assessment).
The item-level power *score* used for auto-equip decisions (see Items,
below) is a related but distinct, narrower mechanism — a common point of
confusion since both are called "power."

### XP & leveling
XP only from kills, `random(200..400 ticks) × passive_rate ×
(1+Haven XP%)` (`src/core/xp.rs:98-107`); passive rate = `1.0 × prestige
multiplier × (1 + WIS_mod×0.05)`. Level curve `xp_for_next_level(level) =
floor(100 × level^1.5)` (`xp.rs:7-9`). +3 attribute points per level, capped
at `20 + 5×rank` (`game_state.rs:229-232`). Offline XP: `(elapsed/5.0) ×
0.25` estimated kills, capped at 7 days (`src/core/offline.rs:33-53`).

### Character & prestige
Six attributes (STR/DEX/CON/INT/WIS/CHA, base 10, mod = `(value-10)/2`)
each drive a distinct combat lever — STR/INT flat damage, CON flat HP,
DEX defense+crit, WIS XP%, CHA prestige-mult bonus
(`src/character/calculation.rs:43-60`). Prestige (`perform_prestige()`,
`src/character/prestige_actions.rs:24-71`) wipes level, XP, attributes,
**all 7 equipment slots**, and active dungeon/fishing/minigame state, while
keeping achievements, Haven, fishing rank, Ascension, Stormglass, the Deep,
the Loom, and Soulforge enhancement — prestige resets the *character*, not
the *account*. Multiplier `1.0 + 0.5×rank^0.7` (P1=1.5x, P10≈3.51x, P30≈
6.41x, `character/tiers.rs:64-67`) applies to XP-gain rate only; separate
flat-bonus formulas add prestige-scaled damage (`floor(5×rank^0.7)`),
defense (`floor(3×rank^0.6)`), crit (`min(rank×0.5,15)%`), and HP
(`floor(15×rank^0.6)`) directly into the combat pipeline
(`constants.rs:188-195`). Prestige *tier names* (Bronze…Celestial…Eternal at
rank 20+) are cosmetic and decouple entirely from the ever-climbing
multiplier — "Eternal" repeats forever from P20 onward even as the game
expects runs into the tens of thousands of ranks by the Loom-zone endgame.

### Zones
50 authored zones (`ZONE_ENEMY_STATS`, `constants.rs:113-179`). Zones 1-10:
subzones, boss every 10 kills, prestige-gated in pairs (P0/P5/P10/P15/P20).
Zone 11, the Expanse, cycles infinitely once unlocked (P25 + `StormsEnd`
achievement) until Fracture zones open. Fracture zones 12-30: **dual-gated**
by Deep Layer (3→Z12-14, 7→Z15-17, 12→Z18-20, 18→Z21-23, 25→Z24-26, 30→
Z27-30) **and** prestige rank (P50/P75/P100/P150/P200/P300 respectively) —
root `CLAUDE.md` documents only the Deep-layer half of this gate, a known
discrepancy (`src/zones/access.rs:30-36`). Stat scaling 1.6x/zone from Zone
11 base (`FRACTURE_ZONE_STAT_MULTIPLIER`). Loom zones 31-50: **triple-gated**
by completed patterns (4/8/16/22/28), Ascension tier (none/VII/VIII/IX/X),
and prestige (P2,000/5,000/15,000/30,000/50,000) across five 4-zone chapters
(`src/zones/access.rs:37-56`). Stat scaling 1.25x/zone from Zone 30 base.

### Items
`ilvl = zone_id × 10`. T0-T9 tier roll is independent of rarity (a Common
can roll T9's 1.00x multiplier just as a Legendary can roll T0's 0.40x) —
cumulative odds T0 38% down to T9 0.1% (`constants.rs:51-74`). Mob drop rate
`min(0.15 + rank×0.01, 0.25) × (1+Haven%)`, capped at Epic, never Legendary;
boss drops are guaranteed and rarity-tabled — normal boss 2% Legendary
ceiling, Zone 10 final boss 5% (`src/items/drops.rs`). 7 equipment slots.
Power score = sum of attributes + affix-value×weight (DamagePercent 2.0
highest, HPBonus 0.5 lowest); auto-equip replaces only on strictly higher
power, and Mythic (God) items are protected from replacement by a hardcoded
rarity check rather than a power-formula floor — God-item power scoring is
itself deferred (tracked as issue #272 per `src/items/CLAUDE.md`).

### Enhancement (Soulforge)
+0 to +10 per slot. Success rates 100% (+1-4), 70/55/40% (+5-7), 30/20/10%
(+8-10); costs 1 PR (+1-4), 2/3/3 PR (+5-7), 4/4 PR (+8-9), 5 PR (+10)
(`src/enhancement/types.rs:53-64`). Failure downgrades by 1 level (+5-9) or
2 (+10), never below +4. Soul Tithe buys a guaranteed success at each of
+5 through +10 for a price explicitly derived from (and unit-tested against)
the expected PR cost of gambling that step repeatedly — the pricing math is
documented directly in the source comment, not just picked round numbers.
Discovery gates at P15+ with the same formula shape as Haven (see cadence
table below).

### Ascension
10 tiers. PR cost I-VI `[35,65,120,200,325,500]`, VII-X
`[1500,4000,8000,15000]`. Multiplier `2^level` for I-VI (2x-64x), `64 ×
1.5^(level-6)` for VII+ (96x/144x/216x/324x). Gates: I-VI need Deep Layers
`[3,7,12,18,25,30]`; VII-X need `[8,16,22,28]` completed Woven Patterns.
The multiplier is recomputed fresh from `state.ascension_level` at three
separate pipeline stages each tick — player damage, enemy defense, and max
HP (`combat/player_attack.rs:57`, `combat/enemy_attack.rs:38`,
`core/tick_stages.rs:800-802`) — a pure function re-evaluated three times
rather than cached once, cheap but worth knowing if any one call site is
ever edited in isolation.

### The Deep
Discovered deterministically (no RNG roll) on the first Expanse-cycle boss
kill at P15+ (`core/tick_stages.rs:489-511`). Layer tiers: Shallows 1-3,
Warrens 4-7, Hollows 8-12, Sunken Reach 13-18, Abyss 19-25, Void 26+. Guild
ranks 1-5 gate roster size and concurrent missions. Missions run on the wall
clock (`DateTime<Utc>`), not simulated per-tick — only checked for pending
check-ins; offline time is resolved on load like the core offline-XP system.
Gateway Expedition (Layer 30 only) is a fixed 72h/259,200s — a stale doc
comment on the enum variant itself still says "24h"
(`src/deep/types.rs:224` vs. `:258`). Deep state (roster, missions, marks,
layer records) **survives prestige** — only a generation counter advances —
despite a stale `mod.rs` doc table claiming otherwise. Clearing Layer 18
unconditionally grants Layer 19 at least 25% familiarity as a hardcoded
tier-transition softener (`src/deep/layers.rs:307-311`).

### Loom
29 total Woven Patterns: 28 completable + 1 permanent "eternal" pattern
deliberately excluded from every completion count — a designed-in sink to
keep the production loop meaningful after the completable content is done,
not a leftover. Shuttle level caps follow the Ascension tier (1 for tiers
0-VI, then 3/5/7/10 for VII-X). WR→PR conversion: `PR/hr = WR × (1+WR/100)`
— self-multiplying, ~1:1 at low rates, rounds to 303 PR/hr at 131 WR/hr (a
stale inline comment still says 302, `src/loom/logic.rs:822` vs. the test at
`:895-897`). Activates only once all 28 completable patterns are done. Loom
zone unlocks are the same triple-gate described above under Zones.

### Power Cores
Six passive PR generators (2/3/5/8/12/18 PR/day), unlocking at Deep Layers
3/7/12/18/25/30. Each core accrues via whole fill-cycles
(`86400 / pr_per_day` seconds per PR) rather than continuous drip, with the
remainder preserved across grants and an identical offline-catchup path. A
freshly-unlocked core deliberately grants zero PR immediately — its first
payout always lands exactly one fill-duration after unlock, live or offline
alike. 48 PR/day is the emergent sum when all six are unlocked, not an
enforced clamp anywhere in code (though the openspec requirement's own
title, "Combined Maximum Passive Output," reads more like a cap than the sum
it actually describes).

### God Items
Three fixed Mythic-rarity Norse artifacts: **Asprika** (Armor, +40 CON/+20
WIS, 30% flat damage reduction), **Sleipnir** (Boots, +40 DEX/+20 WIS, 100%
attack speed plus regen/dungeon/fishing speed bonuses), **Megingjörð**
(belt lore, but occupies the **Ring** slot — there is no belt slot in the
item system — +40 STR/+20 CON, 150% damage). All three carry a flat +40%
XPGain affix. No player-facing acquisition path exists; the only route is a
debug-menu forge action (`--debug` flag), each guarding against
re-forging an already-equipped copy.

### Haven
Account-level (not per-character), 14 rooms across two branches (combat:
Armory→Training Yard/Trophy Hall→Watchtower/Alchemy Lab→War Room; QoL:
Bedroom→Garden/Library→Fishing Dock/Workshop→Vault) plus the Storm Forge
capstone requiring both War Room and Vault, 25 PR. Discovery requires P10+,
chance `0.000014 + (rank-10)×0.000007` per tick. Bonuses are computed once
per tick into a `HavenBonuses` struct and threaded through explicit function
parameters rather than read globally, keeping e.g. `items/drops.rs` free of
any `haven` import (`src/haven/CLAUDE.md:119-125`). War Room grants Double
Strike chance (10/20/35%); Fishing Dock grants Double-Fish chance (25/50/
100%) plus a T4-only +10 max fishing rank; Vault preserves 1/3/5 equipped
items across prestige. Forging the Stormbreaker itself needs both the
Storm Forge built **and** the Storm Leviathan achievement **and** a second,
independent P25 threshold — a triple-lock beyond whatever prestige was
spent building the tree.

### Stormglass
Per-character soft currency from salvaging non-equipped drops, discovered
at P15+. 12 Storm Sigil types (a stale test comment still says "11"),
5 slots unlocked for 25k-400k Stormglass, daily rotation reseeded from the
UTC calendar date (midnight rollover, not local time) via a fixed hash of
the day number. Chrono Surge buys 15min-8h of accelerated ticks on a
`p^1.6` curve (0.85x-3.40x). Storm Lure adds a modest odds bonus to Storm
Leviathan encounters — **not** a guarantee, despite two independent stale
docs (`src/stormglass/CLAUDE.md:30` and `openspec/specs/stormglass/spec.md:
55`) claiming otherwise; the fishing module's own spec describes it
correctly.

### Fishing
Discovered at a flat 5% chance per kill (no prestige gate), suppressed
while already fishing/dungeoneering. 40 ranks across 8 named tiers
(Novice→Transcendent), base cap 30 without Haven, 40 with Fishing Dock T4.
Catch rarity odds shift toward rarer fish every 5 ranks. At max rank,
Legendary catches trigger the 10-encounter Storm Leviathan hunt described
above in Player's Experience — catching it enables Stormbreaker forging.

### Dungeon
Discovered at a flat 1% per kill, independent of prestige (suppresses that
kill's fishing-discovery roll). Procedurally generated via a recursive
backtracker with a 15% extra-loop-connection chance; exactly one Entrance/
Elite/Boss room each, Treasure room count scaling 1→8 by dungeon size. The
Elite guardian's defeat grants a boss key exactly once; the explorer routes
around the boss room until the key is held. Live enemy multipliers are Elite
2.2x HP/1.5x damage/1.6x defense, Boss 3.5x/1.8x/2.0x — a separate,
`#[allow(dead_code)]`-marked helper returning a different (1.5/2.0) pair has
no live callers and is exercised only by its own now-vestigial unit tests.

### Achievements, Challenges, Time Vault
240 achievements across 9 categories, point tiers 5-500, 19 prestige
milestones (a stale CLAUDE.md comment enumerating "18" milestones omits
rank 100's "Eternal" entry from its own count), 64 unlockable titles (a
stale archived design doc and its own tasks file both still say "29" — two
independently-drifted sources agreeing with each other, not one typo).
14 challenge minigames (Chess, Go, Gomoku, Nine Men's Morris, Minesweeper,
Rune Deciphering, Snake, Flappy Bird-style, Jezzball-style, Sigil Surge,
Sigil Matrix/Sudoku, Shard Fusion, Runic Lights, Vault Warden), each with
Novice/Apprentice/Journeyman/Master tiers and the two-step Esc forfeit
pattern. Current discovery weights (Rune 30, Minesweeper 28, Snake 22,
Flappy/Sigil Surge/Shard Fusion/Runic Lights 20 each, Jezzball/Sudoku/Vault
Warden 18 each, Gomoku 15, Morris 12, Chess 8, Go 7) supersede the original
6-game table `docs/decisions.md` documents in detail — but the doc has no
entry at all for the 4 newest games' weights, a real gap in the decision
log. AI: minimax via the `chess-engine` crate for Chess, minimax+alpha-beta
for Morris/Gomoku, MCTS for Go (branching factor too high for a reliable
eval function). Discovery requires P1+, ~2hr average. The git-backed Time
Vault snapshots on every major milestone (zone/dungeon/Leviathan/
achievement/prestige/minigame-win/Haven-build/Soulforge-result/Chrono-Surge)
but never on the 30s autosave; no auto-prune, and even a restored-over
snapshot survives in git's own reflog beneath the app's UI.

### Persistence
JSON saves via `QUEST_DIR` (env var, else `~/.quest`). Character files fail
loudly on parse error (surfaced as "corrupted" in the character list);
**account-level files fail silently** — Haven, achievements, Deep, Loom,
and enhancement state each load via an `unwrap_or_default()` one-liner that
wipes the entire system back to defaults on any parse error, with zero
error surfaced to the player. This is explicitly flagged as a load-bearing
hazard in the regression test's own header comment
(`tests/save_compat_tests.rs:8-13`), which is the only thing standing
between a serde change and a silent account wipe.

### Discovery cadence, compared
| System | Gate | Mechanism |
|---|---|---|
| Haven | P10+ | `0.000014 + (rank-10)×0.000007`/tick, RNG |
| Soulforge | P15+ | same shape as Haven, shifted floor, RNG |
| Challenges | P1+ | flat `0.000014`/tick × (1+Haven%), RNG |
| Fishing | none | flat 5% per kill, RNG |
| Dungeon | none | flat 1% per kill, RNG |
| The Deep | P15+ | **no RNG** — hard-triggered on first Expanse-cycle boss kill |

The `0.000014`/`0.000007` literals are independently declared, numerically
identical constants in three unrelated modules (Haven, Soulforge,
Challenges) with no shared name tying them together — a future balance pass
touching "the" discovery rate has to know to find all three (four, counting
the Deep's different mechanism) by hand.

## Interrelations

```
Combat (kills, 100ms tick) ──► XP/Level ──► Attribute points
        │                                        │
        ├─► Item drops ──► Equip/score ──► combat power
        │                                        │
        └─► Prestige (reset char, +multiplier, +flat combat bonuses)
                    │
        ┌───────────┼──────────────────────────────────────┐
        ▼           ▼              ▼              ▼        ▼
      Haven     Soulforge      Stormglass     Challenges  Fishing
     (P10+)      (P15+)      (P15+, salvage)   (P1+)    (5%/kill)
        │           │                                       │
        │           │                              Storm Leviathan (r40)
        │           └── PR spend ◄── Ascension ◄── Deep patterns ──┐
        │                              │  (Deep layer + pattern    │
        │                              │   gates)                 │
        └── Storm Forge (needs War Room+Vault) ───────────────► Stormbreaker
                                                                    │
                                                        Zone 10 boss gate
                                                                    │
      Zone 11 Expanse (infinite) ──► first cycle-boss kill @P15+ ──► The Deep
                                                                    │
                              Deep Layers ──► Fracture zones (12-30, dual-gated)
                              Deep Layers ──► Power Cores (passive PR)
                                                                    │
                              Loom patterns ──► Ascension VII-X
                              Loom patterns ──► Loom zones (31-50, triple-gated)
                              Loom WR ──► PR (self-multiplying conversion)
                                                                    │
                Zone 50 clear + 28 patterns + Asc X + 250k PR ──► Vessel launch (Act 2)
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
deterministic — root `CLAUDE.md`), so these are conservative floors, not
tight tuning targets. The strategy spread is the more interesting signal:
optimal (which spends heavily on Haven/enhancement) prestiges the *least*
of the three and yet reaches the *highest* pattern count relative to its
prestige spend, while speedrun's raw prestige-count advantage (298 vs. 28)
doesn't translate into proportionally more Loom patterns (10 vs. 8) —
consistent with patterns being Deep/Loom-infrastructure-gated rather than
purely prestige-gated. None of the three built-in profiles reach Loom zone
40+ or Ascension VII+ within 100 real hours from a P300 baseline, which
tracks with the root `CLAUDE.md`'s framing of those as genuine late-game
targets rather than 100-hour ones.

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
| 6 | Anticipation instruments | 2/5 | By the design thesis's own framing (`the-vessel-design.md`, quoted in `act2-pilgrimage.md`): "Act 2's fun is anticipation, choice, people, and consequence — the kinds of fun Act 1 never touched." Act 1 has countdown timers (boss-in-N-kills, mission-in-N-hours) but nothing like Act 2's fuel bar, watch forecasts, or named-arrival scenes. This is an intentional gap, not an oversight — Act 1 specializes elsewhere. |
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
