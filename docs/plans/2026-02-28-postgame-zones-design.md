# Deep-Unlocked Postgame Zones 12-20 with Ascension System

> **Design doc for issue #423.** Extends the zone ladder to 20 with three named chapters unlocked by Deep breakthroughs, exponential enemy scaling (1.6x per zone), and a per-character Ascension system that provides multiplicative combat power gated by Deep layer milestones.

## Summary

Three postgame chapters, each containing three surface zones with five subzones:

- Deep Layer 3 breakthrough unlocks **The Red Fault** (Zones 12-14)
- Deep Layer 7 breakthrough unlocks **The Mirror Scar** (Zones 15-17)
- Deep Layer 13 breakthrough unlocks **The Black Mouth** (Zones 18-20)

Enemy difficulty scales at **1.6x per zone** (68.72x total from Zone 11 to Zone 20). This steep curve is offset by a new **Ascension system** — a per-character combat power multiplier purchased with prestige ranks, gated by Deep layer milestones. Each Ascension level doubles all combat stats, making it practically required for progression.

Zone 20 (The Black Mouth) becomes the permanent postgame loop cap. Gateway remains reserved for future expansion.

---

## Core Mechanics

### Exponential Difficulty Model

**Multiplier:** 1.6x per zone step, applied uniformly to all six enemy stat fields.

For each zone `z` in `12..=20`:
- `stats[z] = round(stats[z - 1] * 1.6)` for `base_hp`, `hp_step`, `base_dmg`, `dmg_step`, `base_def`, `def_step`

This is a single-multiplier curve. HP, damage, and defense all scale together.

**Shipping postgame stat tuples:**

| Zone | base_hp | hp_step | base_dmg | dmg_step | base_def | def_step |
|------|---------|---------|----------|----------|----------|----------|
| 11 (base) | 5,000 | 400 | 500 | 80 | 250 | 30 |
| 12 | 8,000 | 640 | 800 | 128 | 400 | 48 |
| 13 | 12,800 | 1,024 | 1,280 | 205 | 640 | 77 |
| 14 | 20,480 | 1,638 | 2,048 | 328 | 1,024 | 123 |
| 15 | 32,768 | 2,621 | 3,277 | 524 | 1,638 | 197 |
| 16 | 52,429 | 4,194 | 5,243 | 839 | 2,621 | 315 |
| 17 | 83,886 | 6,711 | 8,389 | 1,342 | 4,194 | 503 |
| 18 | 134,218 | 10,737 | 13,422 | 2,148 | 6,711 | 805 |
| 19 | 214,748 | 17,180 | 21,475 | 3,436 | 10,737 | 1,289 |
| 20 | 343,597 | 27,488 | 34,360 | 5,498 | 17,180 | 2,062 |

**Scope of scaling:** Affects normal zone enemies, subzone bosses, zone bosses, and dungeon enemies that inherit zone stats for Zones 12-20. Does NOT affect item level scaling, loot rarity, XP formulas, prestige formulas, or Deep mission difficulty.

### Ascension System

A per-character combat power multiplier. Each Ascension level doubles all combat stats (damage, defense, HP). Purchased by spending prestige ranks, gated by Deep layer milestones (account-level).

**Ascension table:**

| Level | Deep Gate | PR Cost | Cumulative Mult | Zones it enables (practically) |
|-------|-----------|---------|-----------------|-------------------------------|
| I | Layer 3 (Shallows) | 10 PR | 2x | Z12-13 comfortable |
| II | Layer 7 (Warrens) | 15 PR | 4x | Z14-15 comfortable |
| III | Layer 12 (Hollows) | 25 PR | 8x | Z16-17 comfortable |
| IV | Layer 18 (Sunken Reach) | 35 PR | 16x | Z18-19 comfortable |
| V | Layer 25 (Abyss) | 50 PR | 32x | Z20 beatable |
| VI | Layer 30 (Gateway) | 65 PR | 64x | Z20 farm comfortably |
| VII+ | None (PR only) | 80+ PR (+15 each) | x1.5 each | Z20 trivial, evergreen sink |

**Total PR for I-VI:** 200 PR

**Ascension is a soft gate.** Deep breakthroughs unlock zones (travelable immediately). Ascending gives the combat power to actually beat them. Players CAN enter zones without the "required" Ascension level — enemies are just brutally hard.

**The action is "Ascend."** Not a purchase. A ritual transformation of combat power. The Ascend confirmation screen mirrors the prestige confirmation: shows what you sacrifice (PR), what you gain (multiplier), requires explicit confirmation.

### Wall-Breakthrough Progression Pattern

Each chapter has a "wall zone" (the cap) that is practically impassable without the next Ascension level:

| Zone | With current Asc | Effective Difficulty | Feel |
|------|-----------------|---------------------|------|
| Z12 | Asc I (2x) | 0.8x | Easier than Z11 |
| Z13 | Asc I (2x) | 1.28x | Comfortable |
| **Z14** | **Asc I (2x)** | **2.05x** | **WALL** |
| Z14 | Asc II (4x) | 1.03x | Breakthrough |
| Z15 | Asc II (4x) | 1.64x | Comfortable |
| Z16 | Asc II (4x) | 2.62x | Getting hard |
| **Z17** | **Asc II (4x)** | **4.20x** | **WALL** |
| Z17 | Asc III (8x) | 2.10x | Breakthrough |
| Z18 | Asc III (8x) | 3.36x | Hard |
| Z19 | Asc IV (16x) | 2.68x | After Deep push |
| **Z20** | **Asc IV (16x)** | **4.30x** | **WALL** |
| Z20 | Asc V (32x) | 2.15x | Beatable |
| Z20 | Asc VI (64x) | 1.07x | Farm comfortably |

The typical player loop: unlock chapter via Deep → farm first 2 zones → hit wall at cap zone → push Deep further → Ascend → beat cap zone → repeat.

---

## Product Behavior

### Unlock Cadence

1. Deep Layer 3 breakthrough → "THE RED FAULT OPENS" world event → zones 12-14 travelable, Ascension I available
2. Deep Layer 7 breakthrough → "THE MIRROR SCAR AWAKES" world event → zones 15-17 travelable, Ascension II available
3. Deep Layer 13 breakthrough → "THE BLACK MOUTH UNSEALS" world event → zones 18-20 travelable, Ascension III available
4. Deep Layers 18/25/30 → Ascension IV/V/VI available (power only, no new zones)
5. After Ascension VI → uncapped Ascension VII+ available with PR only (no Deep gate)

### Progression Semantics

1. Zone 10 still unlocks Zone 11 (The Expanse) through StormsEnd.
2. If no chapter has been unlocked, Zone 11 loops exactly as today.
3. When cap is 14, route is 11 → 12 → 13 → 14, and Zone 14 loops.
4. When cap is 17, route continues 14 → 15 → 16 → 17, and Zone 17 loops.
5. When cap is 20, route continues 17 → 18 → 19 → 20, and Zone 20 loops.
6. Only the current cap zone loops. All other zones advance forward when their boss is defeated.
7. The Expanse remains travelable but stops being the repeatable cap once a later chapter exists.
8. Manual travel to an older postgame zone never creates a local loop — clearing it advances forward.

### Ascension Flow

1. Deep layer milestone reached → notification: "Ascension [N] available" in stats panel
2. Player opens Ascend confirmation (keybind or from stats panel, mirrors prestige flow)
3. Confirmation shows: PR cost, current → new multiplier, zones now practical
4. Player confirms → PR deducted, `ascension_level` incremented on character
5. For chapter-unlock Ascensions (I/II/III): world-event modal fires if not yet shown
6. For power-only Ascensions (IV-VI): simpler power-up notification
7. For uncapped Ascensions (VII+): always available, same Ascend flow

### Novelty Requirements (per chapter unlock)

Each chapter unlock must:
- Show a full world-event modal
- Add one combat log line
- Add one ticker announcement
- Update the travelable zone list immediately
- Feature distinct zone backgrounds and palettes
- Use explicit per-zone enemy naming pools
- Have 5 subzones per zone (bigger than early game zones)
- End on a landmark boss zone

### What Does Not Change in V1

- No route picker
- No Expanse remix system
- No zone-specific combat-rule subsystem
- No new rarity-floor loot system
- No Zone 21+
- No Gateway-based postgame zone unlock
- Loot stays linear: `ilvl = zone_id * 10`

---

## Content Spec

### Chapter 1: The Red Fault
Deep trigger: Layer 3
Unlock title: THE RED FAULT OPENS
Zones: 12-14
Cap after unlock: 14

Regional identity: A continental wound has split the land. Heat, ash, and red arterial light pour upward through a world-scale fracture.

**Zone 12: Splintered Rim** (level range 165-180)
Subzones: Rimwatch Scree, Ashsplit Road, Broken Survey, Bloodglass Shelf, The First Fissure
Bosses: Rimclaw Stalker, Cinder Hound, Searback Ram, Shard-Tusk Brute, Fault Stalker

**Zone 13: Ember Ravine** (level range 180-195)
Subzones: Coalwind Narrows, Smelter Steps, Cauterized Span, Scarforge Hollow, Ember Ravine
Bosses: Maw of Soot, Crucible Knight, Rift Colossus, The Scarbound, Cinder Maw

**Zone 14: Heart of the Fault** (level range 195-210)
Subzones: Artery Bridge, Magma Choir, Severed Descent, Coreglass Throne, Heart of the Fault
Bosses: Veinbreaker, Ash Cantor, Pyre Warden, Rupture Regent, The Red Tyrant

### Chapter 2: The Mirror Scar
Deep trigger: Layer 7
Unlock title: THE MIRROR SCAR AWAKES
Zones: 15-17
Cap after unlock: 17

Regional identity: Reality has been cut and folded back on itself. Reflection becomes geography.

**Zone 15: Shard Fields** (level range 210-225)
Subzones: Split Horizon, Shard Drift, Mirror Furrows, White Fracture, Shard Fields
Bosses: Glass Hound, Prism Jackal, Sky Echo, The Faceted, Prism Widow

**Zone 16: Refraction Steps** (level range 225-240)
Subzones: Bent Causeway, Parallax Gate, Reflected Climb, Lightfall Court, Refraction Steps
Bosses: Angle Serpent, Twin Sight, The Repeater, Shard Marshal, The Reflection Engine

**Zone 17: Hall of Second Suns** (level range 240-255)
Subzones: Solar Debris, False Noon, Mirror Nave, Witness Gallery, Hall of Second Suns
Bosses: Helio Wraith, The Doubled King, Sunshard Titan, The Faceless Chorus, The Many-Faced Witness

### Chapter 3: The Black Mouth
Deep trigger: Layer 13
Unlock title: THE BLACK MOUTH UNSEALS
Zones: 18-20
Cap after unlock: 20

Regional identity: The surface has stopped merely breaking. It has opened wide enough to show hunger.

**Zone 18: Ashen Verge** (level range 255-270)
Subzones: Charline Flats, Gloam Ditch, Burnt Procession, Veil of Cinders, Ashen Verge
Bosses: Gravewing, Ash Revenant, Night Forger, The Last Pyre, Hollow Giant

**Zone 19: Throat of the World** (level range 270-285)
Subzones: Mawgate Steps, Windpipe Hollow, Devourer's Span, Gullet Court, Throat of the World
Bosses: Toothwarden, Black-Lung Behemoth, The Sable Herd, The Horizon Eater, Night Sovereign

**Zone 20: The Black Mouth** (level range 285-u32::MAX)
Subzones: Lip of Unmaking, Void Saliva Falls, Jawbone Causeway, Mouth Chapel, The Black Mouth
Bosses: The First Hunger, Crawlfather, The Unlit Colossus, The Choir Below, The Mouth Unending

*Note: Zone and enemy names are refineable during implementation. Structure is locked.*

---

## Presentation Spec

### Unlock Modal Copy

**Red Fault:**
Title: THE RED FAULT OPENS
Atmospheric: "The surface has split, and the wound is burning."
Mechanical: "Zones 12-14 are now reachable beyond the current frontier."
Progression: "Defeat the current endpoint to advance."
Ticker: "Red Fault available"
Combat log: "The Red Fault has opened beyond the Expanse."

**Mirror Scar:**
Title: THE MIRROR SCAR AWAKES
Atmospheric: "The horizon has cracked. Reflection now bleeds into the world."
Mechanical: "Zones 15-17 are now reachable beyond the current frontier."
Progression: "Defeat the current endpoint to advance."
Ticker: "Mirror Scar available"
Combat log: "The Mirror Scar has awakened beyond the frontier."

**Black Mouth:**
Title: THE BLACK MOUTH UNSEALS
Atmospheric: "The final wound has opened wide enough to hunger."
Mechanical: "Zones 18-20 are now reachable beyond the current frontier."
Progression: "Defeat the current endpoint to advance."
Ticker: "Black Mouth available"
Combat log: "The Black Mouth has unsealed beyond the world's wound."

### Zone Backgrounds (visual direction)

- Zone 12: ashfall, cracked rim silhouette, deep red glow
- Zone 13: lava canyon, heat shimmer, rising ember plume
- Zone 14: arterial abyss, coreglass structures, red-black cathedral scale
- Zone 15: glass plain, hard white horizon, reflective shards
- Zone 16: impossible mirrored stair geometry, cyan light cuts
- Zone 17: mirrored sanctuary, twin suns, high-prism glare
- Zone 18: dead ash plains, dim crimson edge light, tall silhouettes
- Zone 19: inward-sloping maw terrain, breathing shadow, dark red slit-light
- Zone 20: monumental void-jaw geometry, sparse crimson illumination, terminal-scale depth

### Enemy Naming Pools (per zone)

Zone 12 prefixes: Rim, Ash, Fault, Ember, Bloodglass
Zone 12 suffixes: Stalker, Hound, Ram, Brute, Crawler

Zone 13 prefixes: Coalwind, Soot, Crucible, Scarforge, Rift
Zone 13 suffixes: Maw, Knight, Colossus, Warden, Fiend

Zone 14 prefixes: Vein, Pyre, Coreglass, Magma, Rupture
Zone 14 suffixes: Breaker, Cantor, Regent, Tyrant, Revenant

Zone 15 prefixes: Shard, Prism, Mirror, White, Glass
Zone 15 suffixes: Hound, Jackal, Widow, Watcher, Echo

Zone 16 prefixes: Bent, Parallax, Reflected, Lightfall, Angle
Zone 16 suffixes: Serpent, Marshal, Repeater, Sentinel, Engine

Zone 17 prefixes: Solar, False, Sunshard, Witness, Second
Zone 17 suffixes: Wraith, King, Titan, Chorus, Herald

Zone 18 prefixes: Char, Gloam, Ashen, Cinder, Veil
Zone 18 suffixes: Wing, Revenant, Forger, Giant, Shade

Zone 19 prefixes: Maw, Tooth, Sable, Gullet, Windpipe
Zone 19 suffixes: Warden, Behemoth, Herd, Devourer, Judge

Zone 20 prefixes: Void, Jawbone, Unlit, First, Mouth
Zone 20 suffixes: Hunger, Colossus, Choir, Crawler, Remnant

No postgame zone may fall back to Expanse or generic naming.

### Enemy Palette Defaults

Zone 12: Red / Yellow. Zone 13: LightRed / Yellow. Zone 14: LightRed / Magenta.
Zone 15: Gray / Cyan. Zone 16: White / Cyan. Zone 17: LightYellow / LightMagenta.
Zone 18: DarkGray / LightRed. Zone 19: Red / DarkGray. Zone 20: White / Red.

No postgame zone may fall back to Expanse palette logic.

### Stats Panel

1. Keep existing main row for zones 1-11
2. Add second row labeled POST for zones 12-20
3. Only render POST row once Zone 12 is unlocked or current zone >= 12
4. Use same current/unlocked/completed/locked visual language as main row
5. Cap zone gets a distinct cap indicator
6. Show Ascension level alongside prestige info (e.g., "P75 | Asc III")

---

## Architecture

### Persistence Model

| Data | Location | Scope |
|------|----------|-------|
| `ascension_level: u32` | `GameState` (per-character save) | Per-character |
| `postgame_zone_cap: u32` | `DeepPersistent` (deep.json) | Account-level |
| `pending_postgame_region_unlock: Option<PostgameRegion>` | `DeepPersistent` | Account-level |

Serde defaults:
- `postgame_zone_cap` defaults to 11 (`#[serde(default = "default_postgame_zone_cap")]`)
- `pending_postgame_region_unlock` defaults to None (`#[serde(default)]`)
- `ascension_level` defaults to 0 (`#[serde(default)]`)

No save version bump required.

### New Types

**`PostgameRegion` enum** in `src/zones/postgame.rs`:
- `RedFault`, `MirrorScar`, `BlackMouth`
- Methods: `start_zone_id()`, `end_zone_id()`, `unlock_layer()`, `title()`, `unlock_headline()`, `unlock_log_line()`, `unlock_ticker_text()`, `unlock_mechanical_line()`, `unlock_atmospheric_line()`

**`src/ascension/` module:**
- `types.rs` — constants (cost table, multiplier per level), helper functions
- `logic.rs` — `can_ascend(ascension_level, prestige_rank, deepest_layer) -> bool`, `ascend(state: &mut GameState) -> AscendResult`, `ascension_combat_multiplier(level: u32) -> f64`

**Ascension multiplier formula:**
- Levels 1-6: `2.0^level` (2, 4, 8, 16, 32, 64)
- Levels 7+: `64.0 * 1.5^(level - 6)`

**Ascension cost formula:**
- Levels 1-6: lookup table `[10, 15, 25, 35, 50, 65]`
- Levels 7+: `65 + 15 * (level - 6)`

**Ascension Deep gate:**
- Levels 1-6: lookup table `[3, 7, 12, 18, 25, 30]`
- Levels 7+: no Deep gate (PR only)

### Modified Types

**`CombatBonuses`:**
- Add `ascension_multiplier: f64` field
- Applied as final multiplier on player damage, defense, and HP in combat pipeline

**`BossDefeatResult`:**
- Keep `ExpanseCycle` for Zone 11 only
- Add `PostgameCycle { zone_id: u32 }`

**`TickEvent`:**
- Add `PostgameRegionUnlocked { region: PostgameRegion, message: String }`
- Add `Ascended { level: u32, message: String }`

**`TickEventFlags`:**
- Add `postgame_region_unlocked: Option<PostgameRegion>`

**`GameOverlay`:**
- Add `PostgameRegionUnlock { region: PostgameRegion }`

### Two Independent Flows

**Flow 1: Deep breakthrough → zone unlock**
1. Deep mission resolves a breakthrough at L3/L7/L13
2. `maybe_unlock_postgame_region()` raises `postgame_zone_cap` and sets `pending_postgame_region_unlock`
3. Next tick emits `TickEvent::PostgameRegionUnlocked`
4. Main loop opens world-event modal
5. `sync_account_zone_unlocks()` makes new zones travelable

**Flow 2: Player Ascends → combat power**
1. Player opens Ascend confirmation
2. Validates: Deep gate met (account-level), enough PR (character-level)
3. Deducts PR, increments `ascension_level` on GameState
4. Recalculates combat bonuses with new multiplier
5. Emits Ascended event for combat log/ticker

### Zone Progression Logic

**`on_boss_defeated()` changes:**
1. Zones 1-9: unchanged
2. Zone 10: StormsEnd, advance to Zone 11 (unchanged)
3. Zone 11 with `postgame_zone_cap == 11`: `ExpanseCycle` (unchanged)
4. Zone 11 with `postgame_zone_cap > 11`: advance to Zone 12
5. Zone `z` where `z < postgame_zone_cap`: advance to `z + 1`
6. Zone `z` where `z == postgame_zone_cap` and `z == 11`: `ExpanseCycle`
7. Zone `z` where `z == postgame_zone_cap` and `z > 11`: `PostgameCycle { zone_id: z }`

### Zone Access

**`sync_account_zone_unlocks(prog, storms_end_unlocked, postgame_zone_cap)`:**
Called at: character load, prestige reset, StormsEnd, postgame region unlock.
1. If `storms_end_unlocked`, unlock Zone 11
2. Unlock every zone in `12..=postgame_zone_cap`
3. Never unlock above cap, never remove earlier unlocks

**Zone data for Z12-20:**
- `prestige_requirement = 0` (access is managed by sync function, not prestige checks)
- Extend `ZONE_ENEMY_STATS` from 11 to 20 rows
- Each zone has 5 subzones

### Constants

In `src/core/constants.rs`:
- Keep `FINAL_ZONE_ID = 10`, `EXPANSE_ZONE_ID = 11`
- Add `FIRST_POSTGAME_ZONE_ID = 12`, `LAST_POSTGAME_ZONE_ID = 20`
- Add `POSTGAME_ZONE_STAT_MULTIPLIER: f64 = 1.6`
- Extend `ZONE_ENEMY_STATS` to 20 rows with exact values from this doc

---

## Achievements

12 new achievements in V1:

### Zone Completion (9)

| ID | Name | Trigger | Category | Points |
|----|------|---------|----------|--------|
| PostgameZone12 | Rimbreaker | Defeat Z12 boss | Combat | 25 |
| PostgameZone13 | Cinderfall | Defeat Z13 boss | Combat | 25 |
| PostgameZone14 | Heart Piercer | Defeat Z14 boss | Combat | 50 |
| PostgameZone15 | Shard Breaker | Defeat Z15 boss | Combat | 50 |
| PostgameZone16 | Light Bender | Defeat Z16 boss | Combat | 50 |
| PostgameZone17 | Sunslayer | Defeat Z17 boss | Combat | 100 |
| PostgameZone18 | Ashen Sentinel | Defeat Z18 boss | Combat | 100 |
| PostgameZone19 | Throat Runner | Defeat Z19 boss | Combat | 100 |
| PostgameZone20 | Maw Closer | Defeat Z20 boss | Combat | 250 |

### Ascension Milestones (3)

| ID | Name | Trigger | Category | Points |
|----|------|---------|----------|--------|
| AscensionI | First Ascension | Reach Ascension I | Progression | 25 |
| AscensionIII | Deepborn | Reach Ascension III | Progression | 50 |
| AscensionVI | Transcendent | Reach Ascension VI | Progression | 250 |

Achievement names refineable during implementation.

---

## Test Cases

### Deep unlock and persistence

- Layer 3 breakthrough raises `postgame_zone_cap` from 11 to 14
- Layer 7 breakthrough raises cap from 14 to 17
- Layer 13 breakthrough raises cap from 17 to 20
- Repeated breakthroughs do not duplicate or downgrade unlocks
- `pending_postgame_region_unlock` serializes and deserializes cleanly
- Chapter unlocks survive prestige
- Chapter unlocks survive app restart

### Ascension

- `can_ascend()` returns false when Deep gate not met (account-level check)
- `can_ascend()` returns false when PR insufficient (character-level check)
- `ascend()` deducts correct PR from character
- `ascension_combat_multiplier(1)` returns 2.0
- `ascension_combat_multiplier(6)` returns 64.0
- `ascension_combat_multiplier(7)` returns 96.0 (64 * 1.5)
- Ascension level persists through save/load
- Ascension level survives prestige (stays on character, PR is gone but level remains)
- New character starts at Ascension 0

### Boss-defeat progression

- Zone 11 boss with cap 11 returns `ExpanseCycle`
- Zone 11 boss with cap 14 advances to Zone 12
- Zone 12 boss with cap 14 advances to Zone 13
- Zone 14 boss with cap 14 returns `PostgameCycle { zone_id: 14 }`
- Zone 14 boss with cap 17 advances to Zone 15
- Zone 17 boss with cap 17 returns `PostgameCycle { zone_id: 17 }`
- Zone 20 boss with cap 20 returns `PostgameCycle { zone_id: 20 }`

### Manual travel

- With cap 14, travel to Zone 11, clear it, advance to Zone 12
- With cap 17, travel to Zone 12, clear it, advance to Zone 13
- With cap 20, travel to Zone 17, clear it, advance to Zone 18
- Older postgame zones never locally loop when a higher cap exists

### Content integrity

- `get_all_zones()` returns 20
- Zones 12-20 each have exactly 5 subzones
- Every zone name, subzone name, and boss name matches this plan
- ZONE_ENEMY_STATS.len() == 20

### Exponential scaling

- Zone 12 tuple matches (8000, 640, 800, 128, 400, 48)
- Zone 20 tuple matches (343597, 27488, 34360, 5498, 17180, 2062)
- Each postgame zone row = round(previous * 1.6)
- Postgame zones do not reuse Zone 11 fallback stats

### Combat with Ascension

- Ascension multiplier is applied to player damage, defense, and HP
- At Ascension 0, no multiplier (1.0x)
- At Ascension I (2x), player damage doubles
- CombatBonuses correctly includes ascension_multiplier

### UI and announcements

- Each chapter unlock queues exactly one world-event modal
- Modal appears even if player is not in Deep overlay
- Ticker and combat log fire once per chapter unlock
- POST row renders once Zone 12 is unlocked
- POST row shows cap zone indicator
- Ascension level shown in stats panel
- Postgame backgrounds and palettes do not fall back to Expanse defaults

### Reward continuity

- `ilvl_for_zone(12)` == 120
- `ilvl_for_zone(20)` == 200
- Loot remains linear while enemy stats scale exponentially

### Achievement safety

- Zone 11 cycle still increments BeyondInfinity
- Zone 12-20 cycles do not increment BeyondInfinity
- Existing Zone 1-10 achievements unchanged
- StormsEnd behavior unchanged
- New zone completion achievements fire correctly
- Ascension milestones fire correctly

---

## Files Expected to Change

**New files:**
- `src/ascension/mod.rs`
- `src/ascension/types.rs`
- `src/ascension/logic.rs`
- `src/zones/postgame.rs`
- `src/zones/access.rs`

**Core and Deep:**
- `src/deep/types.rs` — add `postgame_zone_cap`, `pending_postgame_region_unlock` to DeepPersistent
- `src/deep/mod.rs` — re-exports
- `src/deep/missions.rs` — call `maybe_unlock_postgame_region` after breakthrough
- `src/core/constants.rs` — extend ZONE_ENEMY_STATS, add postgame constants
- `src/core/game_state.rs` — add `ascension_level` field
- `src/core/tick.rs` — emit postgame region unlock events
- `src/core/tick_types.rs` — add PostgameRegionUnlocked, Ascended, PostgameCycle variants
- `src/tick_events.rs` — map new events to combat log
- `src/main.rs` — Ascend keybind, overlay routing
- `src/input/types.rs` — new overlay variant
- `src/input/mod.rs` — dismiss PostgameRegionUnlock modal

**Zones:**
- `src/zones/data.rs` — extend to 20 zones with 5 subzones each
- `src/zones/mod.rs` — re-exports
- `src/zones/boss_defeat.rs` — PostgameCycle logic
- `src/zones/advancement.rs` — call sync_account_zone_unlocks after prestige reset

**Combat and UI:**
- `src/combat/events.rs` — add ascension_multiplier to CombatBonuses
- `src/combat/player_attack.rs` — apply ascension multiplier in damage pipeline
- `src/combat/enemy_attack.rs` — apply ascension multiplier to defense/HP
- `src/combat/enemy_generation.rs` — postgame enemy naming pools
- `src/ui/zone_bg.rs` — 9 new zone backgrounds
- `src/ui/enemy_sprites.rs` — postgame palettes
- `src/ui/enemy_sprite_data.rs` — postgame sprite data
- `src/ui/stats_panel.rs` — POST row, Ascension display
- `src/main_helpers/overlay.rs` — render chapter-unlock modal

**Achievements:**
- `src/achievements/types.rs` — 12 new AchievementId variants
- `src/achievements/data.rs` — achievement definitions
- `src/achievements/handlers.rs` — on_postgame_boss_defeated, on_ascended

**Tests:**
- `tests/zone_progression_test.rs`
- `tests/game_tick_behavior_test.rs`
- `tests/deep_integration_test.rs`
- New: `tests/ascension_test.rs`
- New: `tests/postgame_zones_test.rs`

**Docs:**
- `src/zones/CLAUDE.md`
- `src/deep/CLAUDE.md`
- New: `src/ascension/CLAUDE.md`

---

## Assumptions and Defaults

- Each Deep tier milestone unlocks one Ascension level and (for L3/L7/L13) one chapter
- Ascension is per-character, stored on GameState
- Zone unlocks are account-level, stored on DeepPersistent
- The 1.6x multiplier is the shipping value; balance tuning may adjust pre-release
- Zone content names are provisional and refineable during implementation
- Ascension VII+ uses 1.5x multiplier per level (diminishing vs the 2.0x of I-VI)
- No save version bump needed (all new fields have serde defaults)
- Gateway remains untouched for next expansion
