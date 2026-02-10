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
    pub slot: EquipmentSlot,
    pub rarity: Rarity,
    pub ilvl: u32,                     // Item level (zone_id × 10)
    pub base_name: String,
    pub display_name: String,
    pub attributes: AttributeBonuses,  // STR, DEX, CON, INT, WIS, CHA
    pub affixes: Vec<Affix>,
}
```

### Enums
- **`EquipmentSlot`**: Weapon, Armor, Helmet, Gloves, Boots, Amulet, Ring
- **`Rarity`**: Common(0), Magic(1), Rare(2), Epic(3), Legendary(4) — ordered for comparison
- **`AffixType`**: DamagePercent, CritChance, CritMultiplier, AttackSpeed, HPBonus, DamageReduction, HPRegen, DamageReflection, XPGain

## Item Generation Pipeline

Items flow through two separate drop paths:

### Mob Drops (`try_drop_from_mob`)
1. **Drop roll**: 15% base + 1% per prestige rank (capped at 25%), Trophy Hall bonus applied multiplicatively
2. **Rarity roll** (`roll_rarity_for_mob`): 60% Common, 28% Magic, 10% Rare, 2% Epic. **No Legendaries from mobs.** Prestige (+1%/rank, max 10%) and Workshop bonus (max 25%) shift Common downward.
3. **Item generation**: `generate_item(slot, rarity, ilvl)` with ilvl = zone_id × 10
4. **Name generation** and **auto-equip** as below

### Boss Drops (`try_drop_from_boss`)
1. **Always drops** — guaranteed item on boss kill
2. **No Haven/prestige bonuses** — fixed rarity tables
3. **Normal boss**: 40% Magic, 35% Rare, 20% Epic, 5% Legendary
4. **Zone 10 final boss**: 20% Magic, 40% Rare, 30% Epic, 10% Legendary
5. **No Common drops** from bosses

### Shared Steps
- **Item generation** (`generation.rs`): `generate_item(slot, rarity, ilvl)` creates Item with ilvl-scaled attributes and affixes
- **Name generation** (`names.rs`): Procedural name from prefix/suffix tables based on rarity and slot
- **Auto-equip** (`scoring.rs`): `auto_equip_if_better()` compares weighted score against current equipment

## Item Level (ilvl) Scaling

Items scale with zone progression via `ilvl = zone_id × 10`:
- **ilvl multiplier**: `1.0 + (ilvl - 10) / 30.0`
- ilvl 10 (Zone 1): 1.0x stats
- ilvl 50 (Zone 5): 2.33x stats
- ilvl 100 (Zone 10): 4.0x stats

Both attribute values and affix values are multiplied by the ilvl multiplier.

## Generation Rules by Rarity

Base attribute ranges at ilvl 10 (scaled by ilvl multiplier), 1-3 random attributes:

| Rarity    | Base Attr Range | Affixes | At ilvl 10 | At ilvl 100 (4.0x) |
|-----------|----------------|---------|------------|---------------------|
| Common    | 1              | 0       | 1-3 total  | 4-12 total          |
| Magic     | 1-2            | 1       | 1-6 total  | 4-24 total          |
| Rare      | 2-3            | 2-3     | 2-9 total  | 8-36 total          |
| Epic      | 3-4            | 3-4     | 3-12 total | 12-48 total         |
| Legendary | 4-6            | 4-5     | 4-18 total | 16-72 total         |

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

## Mob Drop Rate Formula

```
base_chance = ITEM_DROP_BASE_CHANCE + (prestige_rank * ITEM_DROP_PRESTIGE_BONUS)
drop_chance = min(base_chance * (1.0 + haven_drop_bonus/100), ITEM_DROP_MAX_CHANCE)
```

Constants from `core/constants.rs`: 15% base, +1% per prestige rank, capped at 25%.

## Haven Integration

Two Haven rooms affect mob drops (boss drops are not affected):
- **Trophy Hall**: Increases drop rate percentage (applied multiplicatively to base chance)
- **Workshop**: Shifts rarity distribution toward higher tiers (max 25% bonus)

Both bonuses are passed as parameters to `try_drop_from_mob()`.

## Fishing Item Drops

Fish catches can also drop items based on fish rarity:
- Common/Uncommon: 5% drop chance
- Rare: 15% drop chance
- Epic: 35% drop chance
- Legendary: 75% drop chance

Item rarity matches the fish rarity. Item ilvl is based on the current zone.

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
