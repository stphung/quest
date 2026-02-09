# Zone System Design

> **Status:** Design complete, pending implementation

## Overview

A zone and subzone system that creates a sense of traveling from easy to difficult areas with themed environments. Features boss gates between subzones, prestige gates between zone tiers, and an endgame weapon-forging mechanic.

## Design Goals

1. Create a sense of progression and travel through themed areas
2. Give prestige tangible rewards (unlock new zones, not just multipliers)
3. Introduce an endgame "prestige sink" with weapon forging
4. Support endless play after Zone 20

---

## Zone Structure

### 20 Unique Zones

| Tier | Prestige Req | Zones | Subzones Each | Total Subzones |
|------|--------------|-------|---------------|----------------|
| 1 | P0 | 1-2 | 3 | 6 |
| 2 | P5 | 3-4 | 3 | 6 |
| 3 | P10 | 5-6 | 4 | 8 |
| 4 | P15 | 7-8 | 4 | 8 |
| 5 | P20 | 9-10 | 4 | 8 |
| 6+ | Weapon+Boss | 11-20 | 4 | 40 |

**Totals:** 20 zones, 76 subzones

### Subzone Depth Pattern

Each zone has subzones representing going "deeper" into danger:

```
Zone Example: Dark Forest
├── Subzone 1: Forest Edge (surface, easiest)
├── Subzone 2: Twisted Woods (middle)
└── Subzone 3: Spider's Hollow (deep, hardest)
    → Boss Gate → Next Zone
```

Tiers 1-2 have 3 subzones per zone (Surface → Middle → Depths)
Tiers 3+ have 4 subzones per zone (Entry → Outer → Inner → Heart)

---

## Progression System

### Era 1: Tutorial Progression (Zones 1-10)

Zones unlock via **prestige rank gates only**:

```
Prestige 0  → Zones 1-2 unlock
Prestige 5  → Zones 3-4 unlock
Prestige 10 → Zones 5-6 unlock
Prestige 15 → Zones 7-8 unlock
Prestige 20 → Zones 9-10 unlock
```

**Estimated time to Zone 10:** ~12 hours

### Era 2: Weapon Forging (Zones 11-20)

Starting at Zone 10, progression requires **building a legendary weapon** and **defeating a multi-phase boss**:

```
Zone 10 (infinite grind)
    ↓ Build Weapon (4 components)
    ↓ Defeat Boss (5 phases)
Zone 11 unlocks
    ↓ Build Weapon + Defeat Boss
Zone 12 unlocks
    ...
Zone 15 requires P30 + Weapon + Boss
    ...
Zone 20 requires P40 + Weapon + Boss
Zone 20 = Infinite endgame (future expansion TBD)
```

---

## Prestige System Changes

### Multiplier Scaling (CHANGED)

**Old:** `1.5^rank` (too fast, trivializes content)
**New:** `1.2^rank` (gentler curve)

| Prestige | Old (1.5^r) | New (1.2^r) |
|----------|-------------|-------------|
| 5 | 7.59x | 2.49x |
| 10 | 57.7x | 6.19x |
| 15 | 437x | 15.4x |
| 20 | 3,325x | 38.3x |

### Prestige Currency (NEW)

Each prestige earns **prestige points** based on level reached:

```
Prestige Points = Level reached when prestiging
```

Examples:
- Prestige at level 80 → earn 80 points
- Prestige at level 150 → earn 150 points

Points are spent on weapon components and boss phase attempts.

---

## Weapon Forging System

### Overview

Each zone (11+) has a **Legendary Weapon** that must be forged to defeat the zone's **Immortal Boss**.

### Weapon Components

Each weapon has 4 components with escalating costs:

| Component | Base Cost (Zone 10) |
|-----------|---------------------|
| Blade/Core | 1,000 points |
| Hilt/Frame | 2,500 points |
| Runes/Power | 5,000 points |
| Soul Binding | 10,000 points |
| **Total** | **18,500 points** |

### Boss Phases

Once the weapon is complete, the Immortal Boss awakens with 5 phases:

| Phase | Cost to Attempt | Notes |
|-------|-----------------|-------|
| 1 | 500 points | Unlocks permanently once beaten |
| 2 | 1,000 points | Unlocks permanently once beaten |
| 3 | 2,500 points | Unlocks permanently once beaten |
| 4 | 5,000 points | Unlocks permanently once beaten |
| 5 | 10,000 points | Defeating opens next zone |
| **Total** | **19,000 points** |

**Total to unlock next zone:** ~37,500 prestige points

### Scaling for Later Zones

Zone costs scale with progression:

| Zone | Weapon Cost | Boss Cost | Total | Additional Gate |
|------|-------------|-----------|-------|-----------------|
| 10→11 | 18,500 | 19,000 | 37,500 | - |
| 11→12 | ~22,000 | ~23,000 | ~45,000 | - |
| 12→13 | ~27,000 | ~28,000 | ~55,000 | - |
| 13→14 | ~32,000 | ~33,000 | ~65,000 | - |
| 14→15 | ~38,000 | ~40,000 | ~78,000 | **Requires P30** |
| ... | ... | ... | ... | |
| 19→20 | ~75,000 | ~80,000 | ~155,000 | **Requires P40** |

---

## Zone Themes

### Thematic Arc

```
Era 1 - Mortal Realm (Zones 1-10)
├── Tier 1: Nature's Edge (Meadow → Dark Forest)
├── Tier 2: Civilization's Remnants (Mountain Pass → Ancient Ruins)
├── Tier 3: Elemental Forces (Volcanic Wastes → Frozen Tundra)
├── Tier 4: Hidden Depths (Crystal Caverns → Sunken Kingdom)
└── Tier 5: Ascending (Floating Isles → Storm Citadel)

Era 2 - Planar Journey (Zones 11-20)
├── Shadow & Fey (Shadow Realm → Fey Wilds)
├── Order & Chaos (Clockwork Spire → Abyssal Depths)
├── Divine Ascent (Celestial Gardens → Astral Sea)
├── Mind & Elements (Nightmare Realm → Elemental Nexus)
└── Ultimate End (The Void → Throne of Eternity)
```

---

### Tier 1: Nature's Edge (P0) - 3 Subzones Each

**Zone 1: Meadow**
| Subzone | Theme | Enemies |
|---------|-------|---------|
| Sunny Fields | Gentle intro | Slimes, rabbits |
| Overgrown Thicket | Denser, aggressive | Boars, wolves |
| Mushroom Caves | Underground fungal | Sporelings, beetles |
| **Boss Gate** | Sporeling Queen | |

**Zone 2: Dark Forest**
| Subzone | Theme | Enemies |
|---------|-------|---------|
| Forest Edge | Twilight woods | Wolves, bandits |
| Twisted Woods | Corrupted trees | Dark elves, treants |
| Spider's Hollow | Web-filled caverns | Giant spiders, cocoon horrors |
| **Boss Gate** | Broodmother Arachne | |

---

### Tier 2: Civilization's Remnants (P5) - 3 Subzones Each

**Zone 3: Mountain Pass**
| Subzone | Theme | Enemies |
|---------|-------|---------|
| Rocky Foothills | Mountain trails | Mountain lions, bandits |
| Frozen Peaks | Alpine heights | Yetis, ice elementals |
| Dragon's Perch | Ancient territory | Drakes, wyverns |
| **Boss Gate** | Frost Wyrm | |

**Zone 4: Ancient Ruins**
| Subzone | Theme | Enemies |
|---------|-------|---------|
| Outer Sanctum | Crumbling walls | Skeletons, cultists |
| Sunken Temple | Flooded chambers | Ghosts, drowned dead |
| Sealed Catacombs | Deepest tombs | Wraiths, mummies |
| **Boss Gate** | Lich King's Shade | |

---

### Tier 3: Elemental Forces (P10) - 4 Subzones Each

**Zone 5: Volcanic Wastes**
| Subzone | Theme | Enemies |
|---------|-------|---------|
| Scorched Badlands | Ash fields | Fire imps, ash walkers |
| Lava Rivers | Molten streams | Salamanders, magma elementals |
| Obsidian Fortress | Volcanic glass | Fire giants, lava hounds |
| Magma Core | Heart of volcano | Core guardians, flame wraiths |
| **Boss Gate** | Infernal Titan | |

**Zone 6: Frozen Tundra**
| Subzone | Theme | Enemies |
|---------|-------|---------|
| Snowbound Plains | Blizzards | Dire wolves, frost trolls |
| Glacier Maze | Ice labyrinth | Ice wraiths, frozen dead |
| Frozen Lake | Beneath the ice | Lake horrors, frost serpents |
| Permafrost Tomb | Ancient preserved | Frozen giants, ice liches |
| **Boss Gate** | The Frozen One | |

---

### Tier 4: Hidden Depths (P15) - 4 Subzones Each

**Zone 7: Crystal Caverns**
| Subzone | Theme | Enemies |
|---------|-------|---------|
| Glittering Tunnels | Gem passages | Crystal bats, gem golems |
| Prismatic Halls | Light-bending | Prism elementals, light wisps |
| Resonance Depths | Singing crystals | Sound demons, echo wraiths |
| Heart Crystal | Central formation | Crystal guardians, gem titans |
| **Boss Gate** | Crystal Colossus | |

**Zone 8: Sunken Kingdom**
| Subzone | Theme | Enemies |
|---------|-------|---------|
| Coral Gardens | Reef territory | Merfolk, reef sharks |
| Drowned Streets | Underwater city | Drowned citizens, sea ghosts |
| Abyssal Palace | Deep pressure | Anglerfish horrors, pressure beasts |
| Throne of Tides | Old king's seat | Royal guard, tide elementals |
| **Boss Gate** | The Drowned King | |

---

### Tier 5: Ascending (P20) - 4 Subzones Each

**Zone 9: Floating Isles**
| Subzone | Theme | Enemies |
|---------|-------|---------|
| Cloud Docks | Landing platforms | Harpies, cloud sprites |
| Sky Bridges | Connecting paths | Wind elementals, sky pirates |
| Stormfront | Thunder territory | Storm drakes, lightning birds |
| Eye of the Storm | Calm center | Storm guardians, tempest knights |
| **Boss Gate** | Tempest Lord | |

**Zone 10: Storm Citadel** ⚡ FIRST WEAPON ZONE
| Subzone | Theme | Enemies |
|---------|-------|---------|
| Lightning Fields | Charged ground | Spark elementals, volt hounds |
| Thunder Halls | Fortress corridors | Storm knights, thunder golems |
| Generator Core | Power source | Core wardens, energy beings |
| Apex Spire | Highest point | Elite guardians, storm lords |
| **Weapon** | **Stormbreaker** | |
| **Boss** | **The Undying Storm** | |

---

### Era 2: Planar Journey (Zones 11-20)

All zones have 4 subzones + Legendary Weapon + Immortal Boss.

**Zone 11: Shadow Realm**
| Subzone | Theme |
|---------|-------|
| Twilight Border | Where light fades |
| Umbral Forest | Trees of pure shadow |
| Dark Mirror | Reflections fight back |
| Void Heart | Absolute darkness |
| **Weapon** | **Shadowrend** |
| **Boss** | **The Lightless One** |

**Zone 12: Fey Wilds**
| Subzone | Theme |
|---------|-------|
| Enchanted Glade | Beautiful but deadly |
| Trickster's Maze | Ever-shifting paths |
| Court of Thorns | Fey nobility |
| World Tree Roots | Ancient power |
| **Weapon** | **Faebloom** |
| **Boss** | **The Erlking** |

**Zone 13: Clockwork Spire**
| Subzone | Theme |
|---------|-------|
| Gear Gardens | Mechanical flora |
| Assembly Lines | Construct factories |
| Logic Engine | Computing core |
| Grand Orrery | Universe model |
| **Weapon** | **Cogbreaker** |
| **Boss** | **The Prime Architect** |

**Zone 14: Abyssal Depths**
| Subzone | Theme |
|---------|-------|
| Pressure Zone | Crushing depths |
| Biolume Caves | Glowing creatures |
| Leviathan Graveyard | Bones of giants |
| The Maw | Ancient hunger |
| **Weapon** | **Depthcaller** |
| **Boss** | **The Abyssal Maw** |

**Zone 15: Celestial Gardens** 🚪 P30 GATE
| Subzone | Theme |
|---------|-------|
| Golden Gates | Divine entrance |
| Seraph Orchards | Angel-tended |
| Hall of Virtues | Tests of worth |
| Throne Approach | Final ascent |
| **Weapon** | **Divinebane** |
| **Boss** | **The Fallen Seraph** |

**Zone 16: Astral Sea**
| Subzone | Theme |
|---------|-------|
| Star Docks | Cosmic harbors |
| Nebula Shoals | Gas clouds |
| Constellation Path | Among stars |
| Galactic Core | Center of all |
| **Weapon** | **Starshatter** |
| **Boss** | **The Cosmic Warden** |

**Zone 17: Nightmare Realm**
| Subzone | Theme |
|---------|-------|
| Fever Dreams | Distorted reality |
| Phobia Halls | Fears manifest |
| Memory Corruption | Past turned hostile |
| Sleeper's Prison | Nightmare source |
| **Weapon** | **Dreamrender** |
| **Boss** | **The Sleepless Horror** |

**Zone 18: Elemental Nexus**
| Subzone | Theme |
|---------|-------|
| Convergence Point | Elements collide |
| Primal Storms | Chaotic weather |
| Balance Chamber | Harmony attempted |
| Origin Flame | Source of elements |
| **Weapon** | **Primordial Edge** |
| **Boss** | **The Elemental Chaos** |

**Zone 19: The Void**
| Subzone | Theme |
|---------|-------|
| Edge of Nothing | Reality crumbles |
| Entropy Fields | Unmaking |
| Forgotten Space | Lost existence |
| The Unmaking | Cessation |
| **Weapon** | **Voidheart** |
| **Boss** | **The Unmaker** |

**Zone 20: Throne of Eternity** 🚪 P40 GATE ♾️ INFINITE ENDGAME
| Subzone | Theme |
|---------|-------|
| Eternal Gates | Final threshold |
| Timeless Halls | Time merges |
| Infinity Garden | Endless scaling |
| The Throne | Ultimate power |
| **Weapon** | **Eternity's End** |
| **Boss** | **The Eternal One** (infinite phases) |

---

## Time Estimates

| Milestone | Prestige | Estimated Time |
|-----------|----------|----------------|
| Zone 3-4 unlock | P5 | ~2 hours |
| Zone 5-6 unlock | P10 | ~5 hours |
| Zone 7-8 unlock | P15 | ~8 hours |
| Zone 9-10 unlock | P20 | ~12 hours |
| Zone 11 unlock | P20 + currency | ~18 hours |
| Zone 15 unlock | P30 + currency | ~35 hours |
| Zone 20 unlock | P40 + currency | ~60 hours |

---

## Implementation Notes

### Files to Modify

- `src/ui/zones.rs` - Replace current zone system
- `src/prestige.rs` - Update multiplier formula to `1.2^rank`
- `src/game_state.rs` - Add prestige currency, weapon progress, boss progress
- `src/game_logic.rs` - Add prestige point earning on prestige

### New Files Needed

- `src/zones/mod.rs` - Zone definitions and subzones
- `src/zones/progression.rs` - Zone unlock logic
- `src/weapon_forge.rs` - Weapon component system
- `src/bosses.rs` - Multi-phase boss system
- `src/ui/zone_map.rs` - Zone/subzone display UI
- `src/ui/weapon_forge.rs` - Forging progress UI

### Data Structures

```rust
struct Zone {
    id: u32,
    name: String,
    subzones: Vec<Subzone>,
    prestige_requirement: Option<u32>,
    weapon: Option<LegendaryWeapon>,
    boss: Option<ImmortalBoss>,
}

struct Subzone {
    name: String,
    depth: u32,  // 1 = surface, higher = deeper
    boss_gate: Option<BossGate>,
}

struct LegendaryWeapon {
    name: String,
    components: [WeaponComponent; 4],
}

struct WeaponComponent {
    name: String,
    cost: u32,
    completed: bool,
}

struct ImmortalBoss {
    name: String,
    phases: [BossPhase; 5],
}

struct BossPhase {
    phase_num: u32,
    cost: u32,
    defeated: bool,
}
```

---

## Open Questions

1. ~~**Zone 6-20 themes**~~ - ✅ Complete
2. ~~**Weapon/Boss names**~~ - ✅ Complete
3. **Post-Zone 20** - Future expansion system TBD
4. **Boss combat mechanics** - How do multi-phase bosses differ from normal combat?
5. **Subzone boss gates** - What are the mini-bosses between subzones?
6. **Weapon components** - Need 4 component names per weapon (Blade, Hilt, Runes, Soul Binding equivalents)
7. **Enemy lists** - Era 2 zones need enemy type definitions

---

## Changelog

- 2026-02-02: Initial design complete
- 2026-02-02: Added all 20 zone themes, subzones, weapons, and bosses
