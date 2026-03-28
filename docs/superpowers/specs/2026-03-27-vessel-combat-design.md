# Vessel Auto-Combat

**Parent spec:** `docs/superpowers/specs/2026-03-27-the-vessel-design.md`
**Sub-project:** 4 of 7
**Depends on:** Sub-project 3 (Room System & Ship Stats)

## Overview

Ship combat uses the same tick-based auto-attack engine as Act 1 hero combat, reskinned as ship volleys. The visual presentation is minimal — a void star field with threat indicators, HP bars, and damage numbers. Enemies scale smoothly with distance, with elite encounters at milestones and Norse mythological bosses at major distance gates.

## Combat Engine

Reuses the existing combat pipeline with ship stats mapped to hero stats:

| Ship Stat | Maps To | Role |
|-----------|---------|------|
| Firepower | Player attack power | Damage dealt per volley |
| Hull (current) | Player HP | Damage absorbed before drift |
| Hull (max) | Player max HP | Total survivability |
| Engines | Evasion (new) | Chance to dodge enemy volleys |
| Sensors | — | No direct combat role (discovery only) |

### Attack Timing

Same tick-based intervals as Act 1:
- **Ship attack interval:** 1.5s (same as player)
- **Enemy attack interval:** varies by type (2.0s normal, 1.5s elite, 1.2s boss)

### Damage Pipeline

```
Ship damage to enemy:
  Firepower → enemy defense → min 1 → crit check → final damage

Enemy damage to ship:
  Enemy attack → ship Hull defense → Engines evasion check → min 1 → final damage
```

Evasion from Engines: `dodge_chance = Engines / (Engines + enemy_accuracy)`, capped at 50%. A dodge negates the entire volley.

### HP and Regen

- **Enemy HP:** determined by distance formula + type modifier
- **Ship Hull:** current hull is the ship's HP. Damaged hull stays damaged until repaired (Workshop room, supplies, or events).
- **No auto-regen between fights.** Hull damage persists. This is the key tension — each fight costs hull that must be actively repaired. Unlike Act 1 where HP regens after kills.

### Kill Rewards

On enemy defeat:
- **Ship XP** — for base stat leveling
- **Salvage** — primary upgrade currency (for room builds/levels)
- **Components** — rare drops, chance scales with enemy tier
- **Fuel/Supplies** — small amounts from scavenging the wreck

## Encounter Frequency

Encounters are **distance-based**. Faster ship = more fights per day.

```
encounters_per_ly = base_rate × distance_modifier
```

- **base_rate:** 2 encounters per ly traveled
- **distance_modifier:** `1.0 + (distance / 5000) × 0.5` — density increases slightly deeper into the void
- At 0.1 ly/day (early): ~0.2 encounters/day (one every ~5 days — sparse, lonely)
- At 1 ly/day (mid): ~2-3 encounters/day
- At 10 ly/day (late): ~25+ encounters/day (constant combat)

Between encounters, the void view shows the peaceful star field.

## Enemy Scaling

### Normal Enemies (formula-based, smooth curve)

```
enemy_hp     = 20 + distance × 2.0
enemy_attack = 5 + distance × 0.8
enemy_defense = 3 + distance × 0.5
enemy_accuracy = 10 + distance × 0.3
```

Stats have ±15% random variance per encounter.

### Enemy Types

**Common — Void Creatures** (70% of encounters):
| Type | Distance | Modifier | Flavor |
|------|----------|----------|--------|
| Void Wisp | 0+ ly | 0.5x stats | Faint, barely hostile |
| Branch Parasite | 200+ ly | 0.8x stats | Feeds on wood-matter |
| Root Worm | 500+ ly | 1.0x stats | Burrowing void dweller |
| Cosmic Stalker | 1,500+ ly | 1.2x stats | Hunts between branches |
| Void Leviathan | 3,000+ ly | 1.5x stats | Massive, slow, devastating |
| Abyss Tendril | 5,000+ ly | 1.8x stats | Reaches from the deep void |
| Entropy Shade | 7,000+ ly | 2.0x stats | Reality itself dissolving |

The highest-tier type available for the current distance is used, with a weighted random roll favoring newer types.

**Elite — Lost Vessels** (20% of encounters):
| Type | Distance | Modifier | Special |
|------|----------|----------|---------|
| Drifting Hulk | 500+ ly | 1.5x stats | Slow attacks, high HP |
| Ghost Frigate | 2,000+ ly | 1.8x stats | Fast attacks, evasive |
| Corrupted Warship | 4,000+ ly | 2.2x stats | Heavy damage, heavy defense |
| Abyssal Dreadnought | 7,000+ ly | 2.8x stats | End-tier elite |

Elites always drop salvage. Higher component drop rate (20% vs 5% for common).

**Bosses — Norse Mythological** (at distance milestones):
| Boss | Distance | Modifier | Drop |
|------|----------|----------|------|
| Níðhöggr's Fang | 1,000 ly | 3x stats | Guaranteed component + room unlock |
| Hræsvelgr's Wake | 2,500 ly | 4x stats | Guaranteed component |
| Jörmungandr Fragment | 5,000 ly | 5x stats | Guaranteed rare component |
| Fenrir's Shadow | 7,500 ly | 7x stats | Guaranteed rare component |
| Surtr's Ember | 9,500 ly | 10x stats | Guaranteed legendary component |

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

Primary currency. Dropped by all enemies:

```
salvage_base = 5 + distance × 0.3
```

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
- Common: 1-3 fuel, 0-1 supplies
- Elite: 5-10 fuel, 2-5 supplies
- Boss: 25 fuel, 15 supplies

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
