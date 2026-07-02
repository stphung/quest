# Vessel Auto-Combat

**Parent spec:** `docs/superpowers/specs/2026-03-27-the-vessel-design.md`
**Sub-project:** 4 of 7
**Depends on:** Sub-project 3 (Room System & Ship Stats)

## Overview

Ship combat uses the same tick-based auto-attack engine as Act 1 hero combat, reskinned as ship volleys. The visual presentation is minimal — a void star field with threat indicators, HP bars, and damage numbers. Enemies scale smoothly with distance, with elite encounters at milestones and Norse mythological bosses at major distance gates.

## Combat Engine

Reuses the existing combat pipeline with ship stats mapped to hero stats. Note: the Act 1 combat stat pipeline was migrated from `u32` to `u64` (#619) — vessel combat stats should be `u64` from the start.

| Ship Stat | Maps To | Role |
|-----------|---------|------|
| Firepower | Player attack power | Damage dealt per volley |
| Hull (current) | Player HP | Damage absorbed before drift |
| Hull (max) | Player max HP | Total survivability |
| Engines | Evasion | Chance to dodge enemy volleys |
| Sensors | Crit chance | Higher Sensors = more crits |

### Attack Timing

Same tick-based intervals as Act 1:
- **Ship attack interval:** 1.5s (same as player)
- **Enemy attack interval:** varies by type (2.0s normal, 1.5s elite, 1.2s boss)

### Damage Pipeline

Full pipeline defined in the Room System & Ship Stats spec (`vessel-rooms-stats-design.md`). Summary:

```
Ship damage to enemy:
  Final Firepower + Rune Array flat bonus (optional) → - enemy defense → min 1 → crit check (Sensors-based) → final damage

Enemy damage to ship:
  Enemy attack → - Hull defense → min 1 → Engines evasion check → final damage to Hull HP
```

**Crit:** `chance = 5% + (Sensors / (Sensors + enemy_stealth)) × 25%`, capped at 30%. Base crit multiplier: 2.0x (components can increase).

**Evasion:** `dodge_chance = Engines / (Engines + enemy_accuracy)`, capped at 50%. A dodge negates the entire volley.

**Rune Array:** If the Rune Array room is built, old-world Transmissions convert into a flat Firepower bonus added before enemy defense. This is optional — costs a room slot and power budget.

### HP and Regen

- **Enemy HP:** determined by distance formula + type modifier
- **Ship Hull:** current hull is the ship's HP. Damaged hull stays damaged until repaired (Workshop room, supplies, or events).
- **No auto-regen between fights.** Hull damage persists. This is the key tension — each fight costs hull that must be actively repaired. Unlike Act 1 where HP regens after kills.

### Kill Rewards

On enemy defeat:
- **Ship XP** — `xp = 50 × (1 + distance/50)^0.5` × type modifier (common 1x, elite 2.5x, boss 10x). At the plateau (~48 kills/day at 9,000 ly ≈ 670 XP each) the ship gains a level every ~2-3 days against the `500 × level^1.3` curve.
- **Salvage** — primary upgrade currency (for room builds/levels)
- **Components** — rare drops, chance scales with enemy tier
- **Fuel/Supplies** — small amounts from scavenging the wreck

## Encounter Frequency

Two components: an **ambient wall-clock rate** (the void is never fully empty) plus a **speed-driven rate** (faster ship = more fights per day):

```
encounters_per_day = 3 + 1.5 × speed_ly_per_day     (capped at 48/day — one per 30 min)
```

- At launch (0.1 ly/day): ~3/day — a fight every ~8 hours. Sparse and lonely, but alive, and enough salvage income to bootstrap the first room builds.
- Mid voyage (10 ly/day): ~18/day.
- Cruise plateau (60-100 ly/day): capped at 48/day — constant combat.

**The ambient component continues while the ship is halted** — blocked at a boss milestone or recovering in drift, the void comes to you. This guarantees salvage income even when progress is stopped, so a too-weak ship grinds up to a boss rather than soft-locking.

Between encounters, the void view shows the peaceful star field.

## Enemy Scaling

### Normal Enemies (formula-based, smooth power curves)

Distance spans three orders of magnitude, so scaling is **sublinear** — linear scaling would produce four-digit attack values no ship stat budget can match. First-pass formulas (simulator-validated before implementation):

```
scale = 1 + distance / 50

enemy_hp       = 20 × scale^0.90
enemy_attack   =  5 × scale^0.70
enemy_defense  =  3 × scale^0.65
enemy_accuracy = 10 × scale^0.50
enemy_stealth  = 10 × scale^0.50
```

Anchors (common enemies, before type modifiers):

| Distance | HP | Attack | Defense | Accuracy |
|----------|-----|--------|---------|----------|
| 0 ly | 20 | 5 | 3 | 10 |
| 100 ly | 54 | 11 | 6 | 17 |
| 1,000 ly | 310 | 42 | 22 | 46 |
| 9,000 ly | 2,150 | 190 | 88 | 135 |

The exponents are tuned so late-voyage ship stats (final stats in the hundreds after all four layers) stay in the same magnitude band as enemy stats — fights get harder but the subtractive damage pipeline never degenerates into always-min-1 or one-shots.

Stats have ±15% random variance per encounter.

### Enemy Types

Type thresholds sit on the same geometric ladder as room slots, so new enemies appear at a steady wall-clock cadence.

**Common — Void Creatures** (70% of encounters):
| Type | Distance | Modifier | Flavor |
|------|----------|----------|--------|
| Void Wisp | 0+ ly | 0.5x stats | Faint, barely hostile |
| Branch Parasite | 25+ ly | 0.8x stats | Feeds on wood-matter |
| Root Worm | 100+ ly | 1.0x stats | Burrowing void dweller |
| Cosmic Stalker | 400+ ly | 1.2x stats | Hunts between branches |
| Void Leviathan | 1,600+ ly | 1.5x stats | Massive, slow, devastating |
| Abyss Tendril | 4,000+ ly | 1.8x stats | Reaches from the deep void |
| Entropy Shade | 6,400+ ly | 2.0x stats | Reality itself dissolving |

The highest-tier type available for the current distance is used, with a weighted random roll favoring newer types.

**Elite — Lost Vessels** (20% of encounters):
| Type | Distance | Modifier | Special |
|------|----------|----------|---------|
| Drifting Hulk | 100+ ly | 1.5x stats | Slow attacks, high HP |
| Ghost Frigate | 640+ ly | 1.8x stats | Fast attacks, evasive |
| Corrupted Warship | 2,500+ ly | 2.2x stats | Heavy damage, heavy defense |
| Abyssal Dreadnought | 6,400+ ly | 2.8x stats | End-tier elite |

Elites always drop salvage. Higher component drop rate (20% vs 5% for common).

**Bosses — Norse Mythological** (at distance milestones):
| Boss | Distance | Modifier | Drop |
|------|----------|----------|------|
| Níðhöggr's Fang | 50 ly | 3x stats | Guaranteed component + room unlock |
| Hræsvelgr's Wake | 400 ly | 4x stats | Guaranteed component |
| Jörmungandr Fragment | 1,600 ly | 5x stats | Guaranteed rare component |
| Fenrir's Shadow | 4,500 ly | 7x stats | Guaranteed rare component |
| Surtr's Ember | 9,200 ly | 10x stats | Guaranteed legendary component |

Under the intended speed trajectory these land roughly at months ~2.5, 4, 5, 6, and 7.5 of the voyage — one boss per phase of the ship's growth.

Bosses are one-time encounters. They block passage — you must defeat them to continue past their distance milestone. Attack interval: 1.2s. After defeat, a narrative moment plays.

## Combat Visuals

Minimal HUD overlay on the void star field. No detailed ship sprites fighting.

```
┌─ The Void ──────────────────────────────┐
│                                         │
│  · ·    ·        ·    ·                 │
│     ·        ╱═══╲        ·    ·        │
│  ·          ╱ ◆◆◆ ╲                    │
│            ═══════════    ·             │
│  ·    ·       ║║║║      ·          ·   │
│         ·          ·         ·          │
│                                         │
│  ── THREAT ──────────────────────       │
│  Root Worm                ★ Common      │
│  HP: ██████████░░░░  68%                │
│                                         │
│  Ship Hull: █████████░  92%             │
│                                         │
│  -12 ↑  -8 ↓    -15 ↑   DODGE          │
│                                         │
├─────────────────────────────────────────┤
│  ⚔ Root Worm takes 12 damage            │
│  ↓ Ship hull -8 (92%)                   │
│  ⚔ Root Worm takes 15 damage            │
│  ✦ Root Worm evaded! (DODGE)            │
└─────────────────────────────────────────┘
```

- Damage numbers float briefly above/below the ship art
- "DODGE" flashes when Engines evasion triggers
- Enemy name and type shown with a tier indicator (★ Common, ★★ Elite, ★★★ Boss)
- HP bars for both enemy and ship hull
- Combat log scrolls at the bottom of the void view

When no encounter is active, the threat area is empty and the void feels peaceful.

## Death Handling

If hull reaches 0 during combat:
- Combat ends immediately (enemy doesn't finish you off)
- Ship enters **drift state** (defined in sub-project 2)
- Current enemy despawns (not defeated, no loot)
- No distance is lost from the combat itself

Boss encounters that reduce hull to 0: boss resets to full HP. Player must recover from drift and try again.

## Loot System

### Salvage

Primary currency. Dropped by all enemies (sublinear, same rationale as enemy scaling — room costs are static, so linear salvage would trivialize late-game upgrades):

```
salvage_base = 5 × (1 + distance/50)^0.6
```

Anchors: ~5 at launch, ~31 at 1,000 ly, ~113 at 9,000 ly.

- Common enemies: 1.0x salvage
- Elite enemies: 2.5x salvage
- Bosses: 10x salvage

### Components

Dropped by enemies, installed in room component slots:

| Source | Drop Rate | Quality |
|--------|-----------|---------|
| Common enemy | 5% | Common component |
| Elite enemy | 20% | Uncommon+ component |
| Boss | 100% | Rare+ component |

Component quality tiers (like item rarity): Common, Uncommon, Rare, Legendary. Higher quality = bigger stat bonuses or more unique effects.

### Fuel & Supplies

Small amounts scavenged from defeated enemies:
- Common: 0-1 fuel, 0-1 supplies
- Elite: 2-4 fuel, 1-2 supplies
- Boss: 25 fuel, 15 supplies

Deliberately lean: scavenging covers most fuel drain through the mid voyage but runs a deficit at the cruise plateau, where harvesting + Refinery take over (see mode-transition spec, Fuel economy design intent).

## Files

| File | Change |
|------|--------|
| `src/vessel/combat.rs` | New: encounter generation, damage pipeline, evasion, loot drops |
| `src/vessel/enemies.rs` | New: enemy types, scaling formulas, boss definitions |
| `src/vessel/types.rs` | Modify: add combat state, encounter tracking to VesselState |
| `src/vessel/tick.rs` | Modify: integrate combat ticks into voyage tick |
| `src/ui/vessel_scene.rs` | Modify: add threat HUD, combat log, damage numbers |

## Testing

- Unit test: enemy stat scaling formula at various distances
- Unit test: encounter frequency scales with speed and distance
- Unit test: evasion calculation from Engines stat
- Unit test: damage pipeline (Firepower → defense → min 1)
- Unit test: hull damage persists between encounters
- Unit test: drift triggers on hull reaching 0
- Unit test: boss blocks passage (can't pass milestone without defeating)
- Unit test: loot drops scale with distance and enemy type
- Unit test: boss encounters are one-time (don't repeat after defeat)
- Unit test: elite/common/boss spawn rates match expected distribution
