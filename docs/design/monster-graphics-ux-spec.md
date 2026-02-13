# Monster Graphics UX Specification

## Executive Summary

This document defines the visual design system for enemy monsters in Quest's terminal-based combat view. It addresses five critical gaps in the current implementation: limited sprite variety (6 templates for 55+ enemy types), monochromatic rendering (all enemies Color::Red), no visual tier differentiation, fragile name-based sprite matching, and no zone identity in monster visuals.

The redesigned system uses **zone-based sprite selection** with **tier-based visual enhancements**, giving each of the 11 zones a distinct color palette and set of monster archetypes that reinforce the zone's theme.

---

## 1. Current State Audit

### What Exists

**6 sprite templates** in `src/ui/enemy_sprites.rs`:
- `SPRITE_ORC` -- Humanoid with helmet
- `SPRITE_TROLL` -- Large humanoid with wide body
- `SPRITE_DRAKE` -- Winged creature with diamond eyes
- `SPRITE_BEAST` -- Pointed-ear quadruped
- `SPRITE_HORROR` -- Multi-eyed amorphous shape with tentacles
- `SPRITE_CRUSHER` -- Armored heavy brute

All sprites are 10 lines tall, 14 chars wide.

**Sprite matching** (`get_sprite_for_enemy`) uses name substring matching:
- "orc" -> ORC, "troll" -> TROLL, "drake" -> DRAKE
- "beast"/"fiend" -> BEAST, "horror"/"terror" -> HORROR
- "crusher"/"render"/"maw" -> CRUSHER
- Everything else -> BEAST (default fallback)

**Rendering** in `combat_3d.rs` line 43: All sprites rendered with `Color::Red` + `Modifier::BOLD`. No variation whatsoever.

### Problems Identified

1. **Sprite coverage gap**: Zone enemies use suffixes like Beetle, Rabbit, Wasp, Boar, Serpent, Wolf, Spider, Bat, Treant, Wisp, Goat, Eagle, Golem, Yeti, Harpy, Skeleton, Mummy, Spirit, Gargoyle, Specter, Salamander, Phoenix, Imp, Elemental, Mammoth, Wendigo, Wraith, Bear, Wyrm, Construct, Guardian, Sprite, Watcher, Kraken, Shark, Naga, Leviathan, Siren, Griffin, Djinn, Sylph, Roc, Wyvern, Titan, Colossus, Lord, King, Champion. Only "Drake" matches an existing sprite. Everything else falls through to BEAST.

2. **No zone identity**: A Meadow Beetle and an Abyssal Kraken look identical (same red BEAST sprite). The player gets zero visual feedback about zone progression.

3. **No tier differentiation**: Normal enemies, Elite enemies (prefixed "Elite "), Boss enemies (prefixed "Boss "), subzone bosses (named like "Sporeling Queen"), and zone bosses (named like "The Undying Storm") all look the same.

4. **Monochromatic rendering**: Everything is `Color::Red`. No use of the rich 16-color terminal palette.

5. **Dungeon enemies use generic names**: `generate_enemy_name()` produces "Grizzled Orc" style names that don't match the zone the dungeon is in. These at least partially match existing sprites, but without zone context.

---

## 2. Design Principles

### P1: Zone Identity First
The player should know which zone they are in from the monster's appearance alone. Color palette is the primary zone identifier; sprite silhouette is secondary.

### P2: Progressive Visual Richness
Early zones (Meadow, Dark Forest) use simple, approachable creatures. Later zones (Storm Citadel, The Expanse) use larger, more visually complex designs. This creates a sense of escalation.

### P3: Clear Tier Hierarchy
Normal < Elite < Subzone Boss < Zone Boss. Each tier step should be immediately recognizable through additive visual indicators (not by replacing the base sprite).

### P4: Terminal Constraints Respected
- Standard 16-color ANSI palette only (no RGB/256-color, which varies by terminal)
- All sprites remain 10 lines tall, 14 chars wide (rendering area constraint)
- Only box-drawing characters, Unicode symbols, and ASCII used (no emojis)

### P5: Deterministic Matching
Sprite selection should be deterministic based on `(zone_id, enemy_suffix)` rather than substring matching. This eliminates fallthrough to default.

---

## 3. Color Palette System

### Zone Color Assignments

Each zone gets a **primary color** (sprite body) and a **secondary color** (eyes/details). The palette leverages the full ANSI-16 color set available through Ratatui's `Color` enum.

| Zone | Name | Primary Color | Secondary Color | Rationale |
|------|------|--------------|-----------------|-----------|
| 1 | Meadow | `Color::Green` | `Color::Yellow` | Nature, sunlight, pastoral |
| 2 | Dark Forest | `Color::DarkGray` | `Color::Green` | Shadow, with eerie green eyes |
| 3 | Mountain Pass | `Color::Gray` | `Color::White` | Stone, snow, cold |
| 4 | Ancient Ruins | `Color::Magenta` | `Color::LightRed` | Cursed/arcane energy |
| 5 | Volcanic Wastes | `Color::LightRed` | `Color::Yellow` | Fire, lava, ember glow |
| 6 | Frozen Tundra | `Color::Cyan` | `Color::White` | Ice, frost |
| 7 | Crystal Caverns | `Color::LightMagenta` | `Color::Cyan` | Prismatic, luminous |
| 8 | Sunken Kingdom | `Color::Blue` | `Color::Cyan` | Deep ocean, bioluminescence |
| 9 | Floating Isles | `Color::White` | `Color::Yellow` | Clouds, wind, bright sky |
| 10 | Storm Citadel | `Color::Yellow` | `Color::White` | Lightning, electric energy |
| 11 | The Expanse | `Color::LightRed` | `Color::Magenta` | Void, eldritch, cosmic horror |

### Dungeon Enemy Colors
Dungeon enemies inherit the color palette of the zone the dungeon was discovered in (dungeons are zone-scaled via `zone_id`).

---

## 4. Zone Monster Archetypes

Each zone has 5 enemy suffixes. These should map to 5 distinct sprite archetypes per zone. Rather than creating 55 unique sprites, we define **8 base sprite archetypes** and assign them per zone based on the suffix's creature type.

### Base Sprite Archetypes (8 total)

| Archetype | Silhouette Description | Best For |
|-----------|----------------------|----------|
| **INSECT** | Small body, antennae, segmented legs | Beetles, wasps, spiders, constructs |
| **QUADRUPED** | Four-legged animal, tail, ears/horns | Wolves, boars, bears, mammoths, goats |
| **SERPENT** | Long sinuous body, no legs, fangs | Serpents, wyrms, salamanders, nagas |
| **HUMANOID** | Standing figure, arms, possibly weapons | Skeletons, mummies, harpies, nagas, sirens |
| **AVIAN** | Wings spread, talons, beak | Eagles, bats, phoenixes, griffins, rocs, wyverns |
| **ELEMENTAL** | Amorphous, floating, geometric patterns | Elementals, wisps, sprites, djinn, spirits, golems |
| **TITAN** | Very wide/tall, massive build, heavy limbs | Treants, yetis, golems, titans, colossi, krakens |
| **HORROR** | Asymmetric, tentacles, multiple eyes | Horrors, spectres, wraiths, leviathans, void creatures |

### Zone-to-Archetype Mapping

#### Zone 1: Meadow
| Suffix | Archetype | Notes |
|--------|-----------|-------|
| Beetle | INSECT | Small chitinous creature |
| Rabbit | QUADRUPED | Quick, low-threat |
| Wasp | INSECT | Flying variant (same base, could add wing detail) |
| Boar | QUADRUPED | Stockier variant |
| Serpent | SERPENT | Coiled, simple |

#### Zone 2: Dark Forest
| Suffix | Archetype | Notes |
|--------|-----------|-------|
| Wolf | QUADRUPED | Lean predator |
| Spider | INSECT | Eight-legged, web detail |
| Bat | AVIAN | Smaller wing profile |
| Treant | TITAN | Tree-shaped, massive |
| Wisp | ELEMENTAL | Floating orb of light |

#### Zone 3: Mountain Pass
| Suffix | Archetype | Notes |
|--------|-----------|-------|
| Goat | QUADRUPED | Horned variant |
| Eagle | AVIAN | Large wingspan |
| Golem | TITAN | Stone construct, blocky |
| Yeti | TITAN | Hulking fur-covered |
| Harpy | HUMANOID | Winged humanoid |

#### Zone 4: Ancient Ruins
| Suffix | Archetype | Notes |
|--------|-----------|-------|
| Skeleton | HUMANOID | Bone structure visible |
| Mummy | HUMANOID | Wrapped bandages |
| Spirit | ELEMENTAL | Translucent, floating |
| Gargoyle | TITAN | Stone-winged beast |
| Specter | HORROR | Ghostly, shifting form |

#### Zone 5: Volcanic Wastes
| Suffix | Archetype | Notes |
|--------|-----------|-------|
| Salamander | SERPENT | Fire lizard |
| Phoenix | AVIAN | Flame wings |
| Imp | HUMANOID | Small, horned |
| Drake | AVIAN | Existing drake sprite (reuse) |
| Elemental | ELEMENTAL | Fire/lava form |

#### Zone 6: Frozen Tundra
| Suffix | Archetype | Notes |
|--------|-----------|-------|
| Mammoth | TITAN | Massive, tusked |
| Wendigo | HORROR | Gaunt, antlered |
| Wraith | HORROR | Spectral ice |
| Bear | QUADRUPED | Large, ice-furred |
| Wyrm | SERPENT | Frost serpent |

#### Zone 7: Crystal Caverns
| Suffix | Archetype | Notes |
|--------|-----------|-------|
| Construct | TITAN | Crystalline golem |
| Guardian | HUMANOID | Crystal-armored sentinel |
| Sprite | ELEMENTAL | Tiny light being |
| Watcher | HORROR | All-seeing eye creature |
| Golem | TITAN | Gem-encrusted |

#### Zone 8: Sunken Kingdom
| Suffix | Archetype | Notes |
|--------|-----------|-------|
| Kraken | HORROR | Tentacled, massive |
| Shark | QUADRUPED | Fin profile (adapted) |
| Naga | SERPENT | Serpent-bodied humanoid |
| Leviathan | TITAN | Enormous sea beast |
| Siren | HUMANOID | Aquatic humanoid |

#### Zone 9: Floating Isles
| Suffix | Archetype | Notes |
|--------|-----------|-------|
| Griffin | AVIAN | Lion-eagle hybrid |
| Djinn | ELEMENTAL | Smoke/wind form |
| Sylph | ELEMENTAL | Delicate wind sprite |
| Roc | AVIAN | Enormous bird |
| Wyvern | AVIAN | Two-legged dragon |

#### Zone 10: Storm Citadel
| Suffix | Archetype | Notes |
|--------|-----------|-------|
| Titan | TITAN | Massive armored figure |
| Colossus | TITAN | Even bigger variant |
| Lord | HUMANOID | Armored commander |
| King | HUMANOID | Crown, regal bearing |
| Champion | HUMANOID | Elite warrior |

#### Zone 11: The Expanse (uses default/fallback suffixes)
| Suffix | Archetype | Notes |
|--------|-----------|-------|
| Beast | QUADRUPED | Void-warped animal |
| Horror | HORROR | Eldritch nightmare |
| Fiend | HUMANOID | Demonic entity |
| Terror | HORROR | Shapeless fear |
| Monster | TITAN | Unclassifiable abomination |

---

## 5. Visual Tier Hierarchy

### Tier Definitions

The game has 5 enemy tiers with increasing threat:

| Tier | Source | Name Pattern | Stat Multiplier |
|------|--------|-------------|----------------|
| Normal | Zone mobs | "Meadow Beetle" | 1.0x |
| Dungeon Elite | Dungeon rooms | "Elite Grizzled Orc" | 2.2x HP |
| Subzone Boss | Kill 10 mobs | "Field Guardian" | 3.0x HP |
| Dungeon Boss | Dungeon boss room | "Boss Darken Horror" | 3.5x HP |
| Zone Boss | Final subzone | "Sporeling Queen" | 5.0x HP |

### Visual Differentiation Strategy

Each tier adds visual indicators around the base sprite. These are **additive** -- higher tiers include all lower-tier indicators plus new ones.

#### Tier 1: Normal Enemies
- Base sprite rendered in zone primary color
- Eyes rendered in zone secondary color
- No additional decoration
- Enemy name displayed below in `Color::Yellow`

Example rendering:
```
     [sprite in zone color]

     Meadow Beetle
```

#### Tier 2: Dungeon Elite
- Base sprite in zone primary color
- Name prefix "Elite" already present
- Add **side markers**: vertical bars on left and right of sprite
- Name displayed in `Color::LightRed`

Example rendering:
```
  |      [sprite]       |
  |                     |

     Elite Meadow Beetle
```

Implementation: Add `|` characters in `Color::Yellow` at the leftmost and rightmost columns of the sprite area.

#### Tier 3: Subzone Boss
- Base sprite in zone primary color, but with `Modifier::BOLD`
- Add **crown symbol** above the sprite: `--- * ---`
- Name displayed in `Color::LightRed` with `Modifier::BOLD`

Example rendering:
```
        --- * ---
     [sprite, bold]

     Field Guardian
```

Implementation: Insert a centered crown line above the sprite. The `*` uses `Color::Yellow`, the dashes use `Color::DarkGray`.

#### Tier 4: Dungeon Boss
- Base sprite in zone primary color with `Modifier::BOLD`
- Add **crown symbol** above (same as subzone boss)
- Add **corner brackets** forming a frame
- Name displayed in `Color::LightRed` with `Modifier::BOLD`

Example rendering:
```
  +--- * ---+
  |  [sprite, bold]  |
  |                   |
  +-------------------+
     Boss Darken Horror
```

Implementation: Render a box-drawing frame (`+`, `-`, `|`) around the sprite area in `Color::DarkGray`.

#### Tier 5: Zone Boss
- Sprite rendered in `Color::LightRed` (overrides zone color) to signal ultimate threat
- `Modifier::BOLD` applied
- Add **double crown**: `=== * ===`
- Add **decorative frame** using double box-drawing characters
- Name displayed in `Color::LightRed` with `Modifier::BOLD | Modifier::UNDERLINED`

Example rendering:
```
  +=== * ===+
  |  [sprite, bold, red] |
  |                       |
  +======================+
     The Undying Storm
```

Implementation: Double-line frame using `=` and `|`. Crown uses `=` instead of `-`. Star `*` in `Color::Yellow`.

### Tier Detection Logic

The tier can be determined from available context at render time:

1. **Zone Boss**: The enemy name matches a known zone boss name from `zones/data.rs` (the `SubzoneBoss` with `is_zone_boss: true`). Alternatively, check `game_state.zone_progression.fighting_boss` combined with the subzone being the last in its zone.

2. **Subzone Boss**: `game_state.zone_progression.fighting_boss` is true but the enemy is not a zone boss.

3. **Dungeon Boss**: Enemy name starts with "Boss " and player is in a dungeon (`game_state.active_dungeon.is_some()`).

4. **Dungeon Elite**: Enemy name starts with "Elite " and player is in a dungeon.

5. **Normal**: Everything else.

This detection should be implemented as a function that takes `(&GameState)` and returns an `EnemyTier` enum, keeping the logic centralized.

---

## 6. Sprite Matching Architecture

### Proposed: Zone-Based Matching with Suffix Lookup

Replace the current `get_sprite_for_enemy(enemy_name: &str)` with:

```rust
pub fn get_sprite_for_enemy(enemy_name: &str, zone_id: u32) -> &'static EnemySprite
```

The matching algorithm:

1. Extract the enemy suffix (last word of the name, e.g., "Beetle" from "Meadow Beetle")
2. Look up `(zone_id, suffix)` in a static mapping table
3. Return the corresponding archetype sprite
4. **Fallback**: If no match (e.g., dungeon enemies with old-style names, or boss names like "Sporeling Queen"), use a zone-default archetype:

| Zone | Default Archetype |
|------|-------------------|
| 1 | QUADRUPED |
| 2 | QUADRUPED |
| 3 | TITAN |
| 4 | HUMANOID |
| 5 | ELEMENTAL |
| 6 | QUADRUPED |
| 7 | ELEMENTAL |
| 8 | SERPENT |
| 9 | AVIAN |
| 10 | HUMANOID |
| 11 | HORROR |

### Boss Sprite Selection

Named bosses (subzone/zone bosses) should have their own sprite logic:
- Use keyword matching on the boss name for archetype selection
- "Wolf" -> QUADRUPED, "Treant" -> TITAN, "Wyrm"/"Serpent" -> SERPENT, etc.
- If no keyword matches, use the zone default archetype
- Bosses always get tier decorations regardless of sprite

### Dungeon Enemy Matching

Dungeon enemies use `generate_enemy_name()` which produces generic names ("Grizzled Orc", "Darken Beast"). For these:
- If the dungeon's zone_id is known, use zone color palette
- Match the suffix word against the generic suffix list: "Orc"->HUMANOID, "Troll"->TITAN, "Drake"->AVIAN, "Crusher"->TITAN, "Beast"/"Fiend"->QUADRUPED, "Horror"/"Terror"->HORROR, "Render"/"Maw"->HORROR
- This preserves backward compatibility with existing dungeon name generation

---

## 7. Rendering Changes

### combat_3d.rs Modifications

The `render_simple_sprite` function needs to:

1. Accept zone_id (from `game_state.zone_progression.current_zone_id`)
2. Determine enemy tier from game state
3. Select sprite via new zone-based matching
4. Apply zone color palette (primary for body, secondary for eye characters)
5. Apply tier decorations (crown, frame, name styling)

### Eye Character Coloring

The sprites use specific characters for eyes: `●` (BEAST, ORC, TROLL, CRUSHER, HORROR), `◆` (DRAKE, HORROR). The secondary zone color should be applied specifically to these eye characters. Implementation approach:

- When rendering each sprite line, scan for eye characters (`●`, `◆`)
- Render eye characters in the secondary color
- Render all other characters in the primary color

This adds a subtle but effective two-tone effect without increasing sprite complexity.

### Name Color by Tier

| Tier | Name Color | Modifier |
|------|-----------|----------|
| Normal | `Color::Yellow` | none |
| Dungeon Elite | `Color::LightRed` | none |
| Subzone Boss | `Color::LightRed` | BOLD |
| Dungeon Boss | `Color::LightRed` | BOLD |
| Zone Boss | `Color::LightRed` | BOLD + UNDERLINED |

---

## 8. HP Bar Color Theming

Currently the enemy HP bar is always `Color::Red` (combat_scene.rs line 76). This should also reflect the zone:

- Normal enemies: HP bar uses zone primary color
- Boss enemies (all tiers): HP bar uses `Color::LightRed` (danger signal)

This provides additional zone identity in the UI without being overwhelming.

---

## 9. Implementation Data Structures

### EnemyTier Enum

```rust
pub enum EnemyTier {
    Normal,
    DungeonElite,
    SubzoneBoss,
    DungeonBoss,
    ZoneBoss,
}
```

### ZoneColorPalette

```rust
pub struct ZoneColorPalette {
    pub primary: Color,
    pub secondary: Color,
}

pub fn zone_palette(zone_id: u32) -> ZoneColorPalette { ... }
```

### Sprite Archetype Enum

```rust
pub enum SpriteArchetype {
    Insect,
    Quadruped,
    Serpent,
    Humanoid,
    Avian,
    Elemental,
    Titan,
    Horror,
}
```

### Zone Suffix Mapping

A static lookup table: `fn archetype_for_suffix(zone_id: u32, suffix: &str) -> SpriteArchetype`

This maps every known (zone_id, suffix) pair to an archetype. Unknown suffixes fall back to the zone default.

---

## 10. Migration Strategy

### Phase 1: Color System (Low risk, high impact)
- Add `zone_palette()` function
- Modify `combat_3d.rs` to use zone colors instead of hardcoded `Color::Red`
- Modify `combat_scene.rs` HP bar to use zone colors
- No sprite changes needed

### Phase 2: Tier Detection + Decorations (Medium risk)
- Add `EnemyTier` enum and detection function
- Add crown/frame rendering above/around sprites
- Add tier-based name styling

### Phase 3: New Sprite Archetypes (Highest effort)
- Design and add INSECT, QUADRUPED, SERPENT, AVIAN, ELEMENTAL, TITAN sprites
- Refactor existing HUMANOID sprite from current ORC/TROLL
- Refactor existing HORROR sprite from current HORROR
- Add zone-based sprite matching
- Add two-tone eye coloring

### Backward Compatibility
- Keep `get_sprite_for_enemy(name)` as a deprecated wrapper that calls the new function with `zone_id = 0` (using fallback behavior)
- Existing tests continue to work
- Dungeon enemies with generic names still match via suffix keyword

---

## 11. Summary of Deliverables

| Deliverable | Files Affected | Priority |
|-------------|---------------|----------|
| Zone color palette system | `enemy_sprites.rs` (new), `combat_3d.rs` | P0 |
| Enemy tier detection | `enemy_sprites.rs` or new `enemy_tier.rs` | P0 |
| Tier visual decorations (crown, frame) | `combat_3d.rs` | P1 |
| 8 base sprite archetypes (ASCII art) | `enemy_sprites.rs` | P1 |
| Zone-to-archetype suffix mapping | `enemy_sprites.rs` | P1 |
| Two-tone eye coloring | `combat_3d.rs` | P2 |
| HP bar zone coloring | `combat_scene.rs` | P2 |
| Boss-specific sprite keywords | `enemy_sprites.rs` | P2 |

---

## Appendix A: Full Zone Enemy Name Matrix

For reference, every possible zone enemy name is `{prefix} {suffix}`:

| Zone | Prefixes | Suffixes |
|------|----------|----------|
| 1 Meadow | Meadow, Field, Flower, Grass, Sunny | Beetle, Rabbit, Wasp, Boar, Serpent |
| 2 Dark Forest | Forest, Shadow, Dark, Thorn, Wild | Wolf, Spider, Bat, Treant, Wisp |
| 3 Mountain Pass | Mountain, Rock, Stone, Peak, Cliff | Goat, Eagle, Golem, Yeti, Harpy |
| 4 Ancient Ruins | Ancient, Ruin, Temple, Cursed, Forgotten | Skeleton, Mummy, Spirit, Gargoyle, Specter |
| 5 Volcanic Wastes | Volcanic, Flame, Ash, Molten, Ember | Salamander, Phoenix, Imp, Drake, Elemental |
| 6 Frozen Tundra | Frozen, Ice, Frost, Snow, Glacial | Mammoth, Wendigo, Wraith, Bear, Wyrm |
| 7 Crystal Caverns | Crystal, Gem, Prismatic, Shard, Luminous | Construct, Guardian, Sprite, Watcher, Golem |
| 8 Sunken Kingdom | Sunken, Deep, Coral, Tidal, Abyssal | Kraken, Shark, Naga, Leviathan, Siren |
| 9 Floating Isles | Sky, Cloud, Wind, Storm, Floating | Griffin, Djinn, Sylph, Roc, Wyvern |
| 10 Storm Citadel | Thunder, Lightning, Tempest, Storm, Eternal | Titan, Colossus, Lord, King, Champion |
| 11 The Expanse | (uses default fallback) | Beast, Horror, Fiend, Terror, Monster |

## Appendix B: All Boss Names by Zone

| Zone | Boss | Type |
|------|------|------|
| 1 | Field Guardian | Subzone |
| 1 | Thicket Horror | Subzone |
| 1 | Sporeling Queen | Zone Boss |
| 2 | Alpha Wolf | Subzone |
| 2 | Corrupted Treant | Subzone |
| 2 | Broodmother Arachne | Zone Boss |
| 3 | Bandit King | Subzone |
| 3 | Ice Giant | Subzone |
| 3 | Frost Wyrm | Zone Boss |
| 4 | Skeleton Lord | Subzone |
| 4 | Spectral Guardian | Subzone |
| 4 | Lich King's Shade | Zone Boss |
| 5 | Ash Walker Chief | Subzone |
| 5 | Magma Serpent | Subzone |
| 5 | Fire Giant Warlord | Subzone |
| 5 | Infernal Titan | Zone Boss |
| 6 | Dire Wolf Alpha | Subzone |
| 6 | Ice Wraith Lord | Subzone |
| 6 | Lake Horror | Subzone |
| 6 | The Frozen One | Zone Boss |
| 7 | Gem Golem | Subzone |
| 7 | Prism Elemental | Subzone |
| 7 | Echo Wraith | Subzone |
| 7 | Crystal Colossus | Zone Boss |
| 8 | Merfolk Warlord | Subzone |
| 8 | Drowned Admiral | Subzone |
| 8 | Pressure Beast | Subzone |
| 8 | The Drowned King | Zone Boss |
| 9 | Harpy Matriarch | Subzone |
| 9 | Wind Elemental Lord | Subzone |
| 9 | Storm Drake | Subzone |
| 9 | Tempest Lord | Zone Boss |
| 10 | Spark Colossus | Subzone |
| 10 | Storm Knight Commander | Subzone |
| 10 | Core Warden | Subzone |
| 10 | The Undying Storm | Zone Boss |
| 11 | Void Sentinel | Subzone |
| 11 | Tempest Incarnate | Subzone |
| 11 | Rift Behemoth | Subzone |
| 11 | Avatar of Infinity | Zone Boss |

## Appendix C: Terminal Color Reference (Ratatui Color enum)

Available colors used in this spec:
- `Color::Black`, `Color::Red`, `Color::Green`, `Color::Yellow`
- `Color::Blue`, `Color::Magenta`, `Color::Cyan`, `Color::Gray`
- `Color::DarkGray`, `Color::LightRed`, `Color::LightGreen`, `Color::LightYellow`
- `Color::LightBlue`, `Color::LightMagenta`, `Color::LightCyan`, `Color::White`

Available modifiers: `BOLD`, `DIM`, `ITALIC`, `UNDERLINED`

Note: `Color::Rgb(r, g, b)` and `Color::Indexed(n)` are available in Ratatui but depend on terminal support. This spec uses only the 16 named ANSI colors for maximum compatibility.
