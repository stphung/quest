# Item System

Diablo-style procedural item system with 7 equipment slots, 5 rarity tiers, attribute bonuses, affixes, and smart auto-equip.

## Module Structure

```
src/items/
├── mod.rs         # Public re-exports
├── types.rs       # Core data structures (Item, EquipmentSlot, Rarity, AffixType, Affix)
├── equipment.rs   # Equipment container with slot management and iteration
├── generation.rs  # Rarity-based item generation (attributes + affixes)
├── drops.rs       # Drop rate calculation and item rolling
├── names.rs       # Procedural name generation with prefixes/suffixes
└── scoring.rs     # Weighted auto-equip scoring with attribute specialization
```

## Key Types

### `Item` (`types.rs`)
```rust
pub struct Item {
    pub name: String,
    pub slot: EquipmentSlot,
    pub rarity: Rarity,
    pub level: u32,
    pub attributes: AttributeBonuses,  // STR, DEX, CON, INT, WIS, CHA
    pub affixes: Vec<Affix>,
}
```

### Enums
- **`EquipmentSlot`**: Weapon, Armor, Helmet, Gloves, Boots, Amulet, Ring
- **`Rarity`**: Common(0), Magic(1), Rare(2), Epic(3), Legendary(4) — ordered for comparison
- **`AffixType`**: DamagePercent, CritChance, CritMultiplier, AttackSpeed, HPBonus, DamageReduction, HPRegen, DamageReflection, XPGain

## Item Generation Pipeline

Items flow through this pipeline:

1. **Drop roll** (`drops.rs`): `try_drop_item_with_haven()` checks drop chance → rolls rarity → rolls slot → calls generation
2. **Rarity roll** (`drops.rs`): Base distribution (55% Common, 30% Magic, 12% Rare, 2.5% Epic, 0.5% Legendary) shifted by prestige (+1%/rank, max 10%) and Haven Workshop bonus (max 25%)
3. **Item generation** (`generation.rs`): `generate_item(slot, rarity, level)` creates Item with rarity-scaled attributes and affixes
4. **Name generation** (`names.rs`): Procedural name from prefix/suffix tables based on rarity and slot
5. **Auto-equip** (`scoring.rs`): `auto_equip_if_better()` compares weighted score against current equipment

## Generation Rules by Rarity

| Rarity    | Attribute Points | Affixes | Distribution |
|-----------|-----------------|---------|-------------|
| Common    | +1-2            | 0       | 1-2 attrs randomly |
| Magic     | +2-4            | 1       | Focused on 2-3 attrs |
| Rare      | +4-7            | 2       | Spread across attrs |
| Epic      | +6-10           | 3       | Generous spread |
| Legendary | +8-15           | 4-5     | High values everywhere |

## Auto-Equip Scoring (`scoring.rs`)

The scoring system uses **attribute specialization**: attributes that the character already has high values in get weighted more heavily. This reinforces the character's natural build.

```
score = sum(item_attr * weight) + sum(affix_value * affix_weight)
weight = 1 + (current_attr_value * 100 / total_attr_points)
```

Affix weights (from `score_item`):
- DamagePercent: 2.0x (highest)
- CritChance, CritMultiplier: 1.5x
- DamageReduction: 1.3x
- AttackSpeed: 1.2x
- HPRegen, XPGain: 1.0x
- DamageReflection: 0.8x
- HPBonus: 0.5x (lowest — flat HP less valuable at scale)

## Drop Rate Formula

```
base_chance = ITEM_DROP_BASE_CHANCE + (prestige_rank * ITEM_DROP_PRESTIGE_BONUS)
drop_chance = min(base_chance * (1.0 + haven_drop_bonus/100), ITEM_DROP_MAX_CHANCE)
```

Constants from `core/constants.rs`: 15% base, +1% per prestige rank, capped at 25%.

## Haven Integration

Two Haven rooms affect items:
- **Trophy Hall**: Increases drop rate percentage (applied multiplicatively to base chance)
- **Workshop**: Shifts rarity distribution toward higher tiers (max 25% bonus)

Both bonuses are passed as parameters to `try_drop_item_with_haven()`.

## Adding a New Affix Type

1. Add variant to `AffixType` enum in `types.rs`
2. Add generation rules in `generation.rs` (value ranges per rarity)
3. Add scoring weight in `scoring.rs` `score_item()` match
4. Add display name/formatting in `names.rs` if it affects item names
5. Apply the affix effect in `combat/logic.rs` or `character/derived_stats.rs`

## Adding a New Equipment Slot

1. Add variant to `EquipmentSlot` enum in `types.rs`
2. Add slot field and accessor in `equipment.rs` `Equipment` struct
3. Update `equipment.rs` iteration to include the new slot
4. Add name generation tables for the slot in `names.rs`
5. Update `ui/stats_panel.rs` to display the new slot
6. Update serialization (Serde handles enum variants automatically)
