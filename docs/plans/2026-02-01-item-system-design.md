# Item System Design

**Date:** 2026-02-01
**Status:** Approved
**Author:** Brainstorming session with user

## Overview

A Diablo-like item system with periodic drops, auto-equip, and rich visualization. Items provide attribute bonuses and special affixes that enhance combat effectiveness and progression rate.

## Design Decisions

### Equipment Slots (7 total)
- Weapon
- Armor
- Helmet
- Gloves
- Boots
- Amulet
- Ring

Classic RPG equipment setup provides meaningful progression across multiple item types.

### Stat System: Hybrid
- **Base stats:** All items provide attribute bonuses (STR, DEX, CON, INT, WIS, CHA)
- **Special affixes:** Rare+ items get additional affixes for build diversity
- Attributes affect derived stats through existing modifier system
- Affixes provide percentage-based multipliers and bonuses

### Rarity Tiers (5 levels)

| Rarity | Color | Affixes | Power Level |
|--------|-------|---------|-------------|
| Common | White | 0 | +1-2 attributes |
| Magic | Blue | 1 | +2-4 attributes OR +5-10% |
| Rare | Yellow | 2-3 | +3-6 attributes OR +10-20% |
| Epic | Purple | 3-4 | +5-10 attributes OR +15-30% |
| Legendary | Orange | 4-5 | +8-15 attributes OR +25-50% |

### Drop System
- **Base drop rate:** 30% per kill
- **Prestige scaling:** +5% per prestige rank
- **Examples:**
  - Bronze (rank 0-1): 30% drop chance
  - Silver (rank 2-3): 40% drop chance
  - Gold (rank 4-5): 50% drop chance

### Rarity Distribution by Prestige

**Bronze (0-1):**
- 60% Common, 30% Magic, 10% Rare

**Silver (2-3):**
- 30% Common, 40% Magic, 25% Rare, 5% Epic

**Gold (4-5):**
- 15% Common, 30% Magic, 40% Rare, 13% Epic, 2% Legendary

**Platinum+ (6+):**
- 10% Common, 20% Magic, 35% Rare, 25% Epic, 10% Legendary

### Item Naming
- **Common/Magic:** Simple descriptive (e.g., "Fine Greatsword")
- **Rare+:** Procedural fantasy names (e.g., "Cruel Greatsword of Flame")
- Prefixes/suffixes tied to affixes for flavor

### Auto-Equip: Smart Weighted Scoring
Items automatically replace equipped items if they score higher:
- Attributes weighted by current character build (specialization bonus)
- Affixes weighted by type (damage > utility > QoL)
- Empty slots always equip first item found

### Affix Types

**Damage-focused:**
- Damage % increase
- Critical hit % increase
- Critical damage multiplier
- Attack speed reduction

**Survivability:**
- HP bonus (flat)
- Damage reduction %
- HP regeneration speed
- Damage reflection %

**Progression:**
- XP gain %
- Item drop rate %
- Prestige point multiplier
- Offline progression rate

### UI Layout

```
┌─ CHARACTER ─────────────────────────────────────────────────┐
│ Lvl 25 │ XP: 1,250 / 1,500 │ Prestige: Silver (2)          │
├──────────────────────────────────────────────────────────────┤
│ ┌Stats──────────────┐  ┌Derived────────────────────┐       │
│ │STR 23 DEX 15 CON 23│  │HP 180  DMG 31  DEF 7     │       │
│ │INT 10 WIS 14 CHA 13│  │CRIT 9%  XP +10%          │       │
│ └───────────────────┘  └──────────────────────────┘       │
├──────────────────────────────────────────────────────────────┤
│ ⚔️ Weapon   Flaming Greatsword           [Legendary] ⭐⭐⭐  │
│             +12 STR, +5 DEX, +20% Fire Damage                │
│ 🛡 Armor    Plate Mail of Valor          [Rare] ⭐⭐        │
│             +8 CON, +50 HP                                   │
│ 🪖 Helmet   [Empty]                                          │
│ 🧤 Gloves   Swift Gauntlets              [Magic] ⭐         │
│             +3 DEX                                           │
│ 👢 Boots    [Empty]                                          │
│ 📿 Amulet   [Empty]                                          │
│ 💍 Ring     [Empty]                                          │
└──────────────────────────────────────────────────────────────┘
```

Ultra-compact stats at top, full equipment detail below with two-line entries showing all bonuses.

## Data Structures

```rust
pub struct Item {
    pub slot: EquipmentSlot,
    pub rarity: Rarity,
    pub base_name: String,      // "Greatsword", "Plate Mail"
    pub display_name: String,   // Generated full name
    pub attributes: AttributeBonuses,
    pub affixes: Vec<Affix>,
}

pub enum EquipmentSlot {
    Weapon, Armor, Helmet, Gloves, Boots, Amulet, Ring
}

pub enum Rarity {
    Common,    // White
    Magic,     // Blue, +1 affix
    Rare,      // Yellow, +2-3 affixes
    Epic,      // Purple, +3-4 affixes
    Legendary, // Orange, +4-5 affixes
}

pub struct AttributeBonuses {
    pub str: u32,
    pub dex: u32,
    pub con: u32,
    pub int: u32,
    pub wis: u32,
    pub cha: u32,
}

pub struct Affix {
    pub affix_type: AffixType,
    pub value: f64,
}

pub enum AffixType {
    // Damage-focused
    DamagePercent, CritChance, CritMultiplier, AttackSpeed,
    // Survivability
    HPBonus, DamageReduction, HPRegen, DamageReflection,
    // Progression
    XPGain, DropRate, PrestigeBonus, OfflineRate,
}

pub struct Equipment {
    pub weapon: Option<Item>,
    pub armor: Option<Item>,
    pub helmet: Option<Item>,
    pub gloves: Option<Item>,
    pub boots: Option<Item>,
    pub amulet: Option<Item>,
    pub ring: Option<Item>,
}
```

## Implementation Flow

### 1. Item Generation
```rust
pub fn try_drop_item(game_state: &GameState) -> Option<Item> {
    // Roll for drop
    let drop_chance = 0.30 + (game_state.prestige_rank as f64 * 0.05);
    if rand::random::<f64>() > drop_chance {
        return None;
    }

    // Determine rarity based on prestige
    let rarity = roll_rarity(game_state.prestige_rank);
    let slot = roll_random_slot();

    // Generate item with random stats
    Some(generate_item(slot, rarity, game_state.level))
}
```

### 2. Auto-Equip Logic
```rust
pub fn score_item(item: &Item, game_state: &GameState) -> f64 {
    let mut score = 0.0;

    // Attribute scoring: weight by current character focus
    let weights = calculate_attribute_weights(game_state);
    score += item.attributes.str as f64 * weights.str;
    // ... etc

    // Affix scoring: different weights
    for affix in &item.affixes {
        score += match affix.affix_type {
            AffixType::DamagePercent => affix.value * 2.0,
            AffixType::CritChance => affix.value * 1.5,
            AffixType::XPGain => affix.value * 1.0,
            // ... etc
        };
    }

    score
}

pub fn auto_equip_if_better(item: Item, game_state: &mut GameState) {
    let new_score = score_item(&item, game_state);

    if let Some(current) = &game_state.equipment[item.slot] {
        if score_item(current, game_state) < new_score {
            game_state.equipment[item.slot] = Some(item);
        }
    } else {
        game_state.equipment[item.slot] = Some(item);
    }
}
```

### 3. Integration with Derived Stats
```rust
pub fn calculate_derived_stats(
    attributes: &Attributes,
    prestige_rank: u32,
    equipment: &Equipment,
) -> DerivedStats {
    // Add equipment attribute bonuses to base
    let mut total_attrs = attributes.clone();
    for item in equipment.iter_equipped() {
        total_attrs.str += item.attributes.str;
        total_attrs.dex += item.attributes.dex;
        // ... etc
    }

    // Calculate derived stats from totals
    let mut stats = calculate_base_derived_stats(&total_attrs, prestige_rank);

    // Apply equipment affixes as multipliers
    for item in equipment.iter_equipped() {
        for affix in &item.affixes {
            apply_affix_to_stats(&mut stats, affix);
        }
    }

    stats
}
```

### 4. Combat Integration
After enemy death in `src/main.rs`:
```rust
CombatEvent::EnemyDied { xp_gained } => {
    // Existing XP logic...

    // Try to drop item
    if let Some(item) = try_drop_item(game_state) {
        let equipped = auto_equip_if_better(item.clone(), game_state);
        let msg = format!(
            "✨ Found: {} [{}] {}{}",
            item.display_name,
            rarity_name(&item.rarity),
            "⭐".repeat(item.rarity as usize),
            if equipped { " (equipped!)" } else { "" }
        );
        game_state.combat_state.add_log_entry(msg, false, true);
    }
}
```

## Testing Strategy

### Unit Tests
- Item generation respects rarity constraints
- Affix counts match rarity tier
- Drop rates scale with prestige correctly
- Scoring prefers higher rarity items
- Auto-equip logic works correctly
- Empty slots always equip first item

### Integration Tests
- Items persist through save/load
- Equipment survives prestige
- Derived stats correctly include equipment bonuses
- UI displays all equipment correctly

### Edge Cases
- Empty equipment slots
- Very long item names (truncation)
- Maximum attribute bonuses (overflow prevention)
- Backward compatibility with old saves

## Files to Create/Modify

### New Files
- `src/items.rs` - Item structs and generation
- `src/item_drops.rs` - Drop logic and rarity rolling
- `src/item_names.rs` - Name generation (prefixes/suffixes)
- `src/equipment.rs` - Equipment management and scoring

### Modified Files
- `src/game_state.rs` - Add `equipment: Equipment` field
- `src/derived_stats.rs` - Include equipment in calculations
- `src/main.rs` - Call `try_drop_item()` on enemy death
- `src/ui/stats_panel.rs` - Render equipment section
- `src/save_manager.rs` - Ensure equipment serializes correctly

## Success Criteria

1. Items drop after ~30-40% of kills (scales with prestige)
2. Higher prestige = better rarity distribution
3. Items automatically equip if better than current
4. Equipment visible in stats panel with full details
5. All bonuses correctly apply to combat
6. System survives save/load and prestige
7. UI remains readable and uncluttered
