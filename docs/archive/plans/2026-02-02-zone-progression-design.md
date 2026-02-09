# Zone Progression System Design

## Overview

A comprehensive zone progression system that gives players a sense of advancing through distinct areas, with prestige gating to ensure long-term engagement. The system includes visual theming, zone-specific enemies, and a rebalanced prestige system with diminishing returns.

## Goals

- Players feel progression through distinct themed zones
- Prestige is required to reach endgame content (not skippable)
- Game lasts months of play, not days
- Infinite prestige ladder for hardcore players

## Zone Structure

### 8 Zones Total

| Zone | Levels | Prestige Req | Theme |
|------|--------|--------------|-------|
| 1. Meadow | 0-20 | - | Peaceful starting area |
| 2. Dark Forest | 20-40 | - | Ominous woods |
| 3. Mountain Pass | 40-65 | - | Harsh alpine |
| 4. Ancient Ruins | 65-95 | - | Crumbling temple |
| 5. Volcanic Wastes | 95-130 | - | Fire and brimstone |
| 6. Crystal Caverns | 130-170 | Prestige 5 | Glittering underground |
| 7. Ethereal Plane | 170-220 | Prestige 10 | Spirit dimension |
| 8. Realm of Gods | 220+ | Prestige 15 | Divine endgame |

### Zone Enemies

| Zone | Enemies |
|------|---------|
| Meadow | Slime, Rabbit, Ladybug, Butterfly |
| Dark Forest | Wolf, Spider, Dark Elf, Bat |
| Mountain Pass | Golem, Yeti, Mountain Lion, Eagle |
| Ancient Ruins | Skeleton, Ghost, Ancient Guardian, Wraith |
| Volcanic Wastes | Fire Elemental, Lava Beast, Phoenix, Dragon |
| Crystal Caverns | Crystal Golem, Gem Serpent, Cave Wyrm, Living Shard |
| Ethereal Plane | Phantom, Soul Eater, Ethereal Knight, Banshee |
| Realm of Gods | Celestial, Seraph, Titan, Demigod |

### Zone Themes

| Zone | Wall/Floor Colors | Decorations |
|------|-------------------|-------------|
| Meadow | Green / Light green | 🌸 🌼 🦋 🌻 🌷 |
| Dark Forest | Dark gray / Brown | 🌲 🌳 🍄 🦇 🕷️ |
| Mountain Pass | White / Gray | ⛰️ 🏔️ 🪨 ❄️ ☁️ |
| Ancient Ruins | Dark yellow / Stone | 🏛️ ⚱️ 💀 🗿 🔮 |
| Volcanic Wastes | Red / Dark red | 🌋 🔥 💥 🌪️ ⚡ |
| Crystal Caverns | Cyan / Dark blue | 💎 🔷 ✨ 🪻 💠 |
| Ethereal Plane | Magenta / Purple | 👻 🌀 ☁️ 💫 🌌 |
| Realm of Gods | Gold / White | ⚜️ 👑 ✨ 🌟 ☀️ |

### Zone Flavor Text

| Zone | Entry Message |
|------|---------------|
| Meadow | "A peaceful beginning..." |
| Dark Forest | "Shadows stir between the trees..." |
| Mountain Pass | "The wind howls through the peaks..." |
| Ancient Ruins | "Echoes of a forgotten age..." |
| Volcanic Wastes | "The air burns with heat..." |
| Crystal Caverns | "Light refracts endlessly..." |
| Ethereal Plane | "Reality fades at the edges..." |
| Realm of Gods | "You stand among immortals..." |

## Prestige System Changes

### Triple Gate System

Players are gated from endgame content through three reinforcing mechanisms:

1. **Level Cap**: `max_level = 20 + (prestige × 15)`
2. **Attribute Cap**: `attr_cap = 20 + (prestige × 5)`
3. **Zone Requirements**: Endgame zones require minimum prestige

### New Multiplier Formula (Diminishing Returns)

```rust
pub fn prestige_multiplier(rank: u32) -> f64 {
    1.0 + (rank as f64 * 0.3) / (1.0 + rank as f64 * 0.05)
}
```

| Rank | Multiplier | Gain |
|------|------------|------|
| 0 | 1.0x | - |
| 1 | 1.29x | +29% |
| 5 | 2.2x | +8% |
| 10 | 3.0x | +5% |
| 15 | 3.5x | +3% |
| 20 | 3.9x | +2% |
| 50 | 5.0x | +1% |
| 100 | 5.6x | <1% |

Asymptotes around 6x. Each prestige adds value but never breaks the game.

### Prestige Rank Names

| Rank | Name | Required Level |
|------|------|----------------|
| 1 | Bronze | 15 |
| 2 | Silver | 30 |
| 3 | Gold | 50 |
| 4 | Platinum | 70 |
| 5 | Diamond | 95 |
| 6 | Emerald | 120 |
| 7 | Sapphire | 145 |
| 8 | Ruby | 170 |
| 9 | Obsidian | 195 |
| 10 | Celestial | 170 |
| 11 | Astral | 195 |
| 12 | Cosmic | 220 |
| 13 | Stellar | 245 |
| 14 | Galactic | 270 |
| 15 | Transcendent | 245 |
| 16 | Divine | 275 |
| 17 | Exalted | 305 |
| 18 | Mythic | 335 |
| 19 | Legendary | 365 |
| 20+ | Eternal I, II, III... | +30 each |

### Progression Gates

| Prestige | Level Cap | Attr Cap | Zones Accessible |
|----------|-----------|----------|------------------|
| 0 | 20 | 20 | 1-2 |
| 1 | 35 | 25 | 1-2 |
| 3 | 65 | 35 | 1-4 |
| 5 | 95 | 45 | 1-6 |
| 7 | 125 | 55 | 1-6 |
| 10 | 170 | 70 | 1-7 |
| 15 | 245 | 95 | 1-8 |

### Projected Playtime

| Milestone | Active Play | Real Time (1hr/day) |
|-----------|-------------|---------------------|
| Prestige 5 | ~15 hrs | 2 weeks |
| Prestige 10 | ~50 hrs | 7 weeks |
| Prestige 15 | ~100 hrs | 3+ months |
| Level 200+ zone | ~120 hrs | 4 months |

## Visual Implementation

### Zone Header

```
═══════════ 💎 Crystal Caverns (Lv 130-170) ✨ ═══════════
            ⚠ Requires Prestige 5 ⚠
```

- Centered above 3D dungeon view
- Decorations from zone's theme on each side
- Prestige warning if requirement not met

### Scene Decorations

```
═══════════ 🌲 Dark Forest (Lv 20-40) 🍄 ═══════════

        ┌───────────────────┐
   🦇   │                   │
        │      WOLF         │   🕷️
        │    ████████░░     │
        │     HP: 80%       │
        │                   │
   🍄   └───────────────────┘
              🌳
```

- 1-2 decorations on floor area per scene
- Positioned in corners/edges, not overlapping combat
- Refresh when new enemy spawns

### Zone Transition Banner

```
┌──────────────────────────────────────┐
│   🌋 Entering Volcanic Wastes 🔥     │
│      The air burns with heat...      │
└──────────────────────────────────────┘
```

- Brief overlay (1-2 seconds) or combat log entry
- Shows zone decorations and flavor text

### Color Theming

Each zone defines colors applied to:
- Wall characters (`│`, `─`, `█`, `┌`, `┐`, `└`, `┘`)
- Floor patterns (`.`, `,`, `'`)
- Accent on HP bars and damage numbers

## Implementation

### Files to Modify

| File | Changes |
|------|---------|
| `src/ui/zones.rs` | Add 3 new zones, prestige requirements, flavor text, color themes |
| `src/prestige.rs` | New multiplier formula, new rank names, level cap function |
| `src/game_state.rs` | Add `level_cap()` check, update `get_attribute_cap()` |
| `src/game_logic.rs` | Enforce level cap on XP gain, zone transition detection |
| `src/combat.rs` | Use zone-specific enemy generation |
| `src/ui/combat_scene.rs` | Add zone header rendering |
| `src/ui/combat_3d.rs` | Apply zone color themes to walls/floor |
| `src/ui/perspective.rs` | Support color theming |
| `src/ui/enemy_sprites.rs` | Add 12 new enemy sprites |

### New Structs/Functions

```rust
// zones.rs
pub struct ZoneTheme {
    pub wall_color: Color,
    pub floor_color: Color,
    pub accent_color: Color,
    pub decorations: Vec<&'static str>,
    pub flavor_text: &'static str,
}

pub fn get_zone_for_level(level: u32) -> &Zone;
pub fn can_enter_zone(zone: &Zone, prestige: u32) -> bool;
pub fn on_zone_transition(old: &Zone, new: &Zone) -> ZoneTransitionEvent;

// prestige.rs
pub fn prestige_multiplier(rank: u32) -> f64;  // Updated formula
pub fn level_cap(rank: u32) -> u32;            // New
pub fn get_prestige_name(rank: u32) -> &str;   // Updated with all names
```

### Estimated Effort

| Component | Effort |
|-----------|--------|
| Zone data & theming | Small |
| Prestige formula changes | Small |
| Level cap enforcement | Small |
| Zone header UI | Medium |
| Color theming in renderer | Medium |
| 12 new enemy sprites | Medium |
| Zone transition notifications | Small |
| Testing & balance tuning | Medium |
