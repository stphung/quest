# Item System

Diablo-style procedural item system with 7 equipment slots, 6 rarity tiers (including God/Mythic for god items), attribute bonuses, affixes, and smart auto-equip.

## Module Structure

```
src/items/
├── mod.rs         # Public re-exports
├── types.rs       # Core data structures (Item, EquipmentSlot, Rarity, AffixType, Affix)
├── equipment.rs   # Equipment container with slot management and iteration
├── generation.rs  # Rarity-based item generation (attributes + affixes)
├── drops.rs       # Drop rate calculation and item rolling
├── names.rs       # Procedural name generation with prefixes/suffixes
└── scoring.rs     # Affix power weights and power-based auto-equip
```

## Key Types

### `Item` (`types.rs`)
```rust
pub struct Item {
    pub slot: EquipmentSlot,
    pub rarity: Rarity,
    pub ilvl: u32,                     // Item level (zone_id × 10)
    pub tier: u8,                      // Quality tier (T0-T9), rolled on exponential curve
    pub base_name: String,
    pub display_name: String,
    pub attributes: AttributeBonuses,  // STR, DEX, CON, INT, WIS, CHA
    pub affixes: Vec<Affix>,
    pub god_item_id: Option<GodItemId>, // Set for god items (Asprika, Sleipnir, Megingjord)
}
```

### Enums
- **`EquipmentSlot`**: Weapon, Armor, Helmet, Gloves, Boots, Amulet, Ring
- **`Rarity`**: Common(0), Magic(1), Rare(2), Epic(3), Legendary(4), Mythic(5) — ordered for comparison. Mythic displays as "God" and is used exclusively for god items
- **`AffixType`**: DamagePercent, CritChance, CritMultiplier, AttackSpeed, HPBonus, DamageReduction, HPRegen, DamageReflection, XPGain, Unknown (`#[serde(other)]` fallback for removed variants like DropRate/PrestigeBonus/OfflineRate — ignored at runtime)

## Item Generation Pipeline

Items flow through two separate drop paths:

### Mob Drops (`try_drop_from_mob`)
1. **Drop roll**: 15% base + 1% per prestige rank (capped at 25%), Trophy Hall bonus applied multiplicatively
2. **Rarity roll** (`roll_rarity_for_mob`): 60% Common, 28% Magic, 10% Rare, 2% Epic. **No Legendaries from mobs.** Prestige (+0.5%/rank, max 10%, cap at P20) shifts Common downward. Workshop bonus is multiplicative on non-Common rates (max +25%).
3. **Item generation**: `generate_item(slot, rarity, ilvl)` with ilvl = zone_id × 10
4. **Name generation** and **auto-equip** as below

### Boss Drops (`try_drop_from_boss`)
1. **Always drops** — guaranteed item on boss kill
2. **No Haven/prestige bonuses** — fixed rarity tables
3. **Normal boss**: 40% Magic, 35% Rare, 23% Epic, 2% Legendary
4. **Zone 10 final boss**: 20% Magic, 40% Rare, 35% Epic, 5% Legendary
5. **No Common drops** from bosses

### Shared Steps
- **Item generation** (`generation.rs`): `generate_item(slot, rarity, ilvl)` creates Item with ilvl-scaled and tier-scaled attributes and affixes. Tier is rolled via `roll_tier()` on an exponential drop curve (T0 38% to T9 0.1%)
- **Name generation** (`names.rs`): Procedural name from prefix/suffix tables based on rarity and slot
- **Auto-equip** (`scoring.rs`): `auto_equip_if_better()` compares intrinsic `power()` against current equipment

## Item Level (ilvl) and Tier Scaling

Items scale with two independent multipliers:

**ilvl multiplier** (zone-based): `1.0 + (ilvl - 10) / 30.0`
- ilvl 10 (Zone 1): 1.0x, ilvl 50 (Zone 5): 2.33x, ilvl 100 (Zone 10): 4.0x

**Tier multiplier** (quality roll): T0 = 0.40x through T9 = 1.00x

**Effective multiplier**: `ilvl_multiplier × tier_multiplier`

| Tier | Drop Rate | Multiplier |
|------|-----------|-----------|
| T0 | 38.0% | 0.40x |
| T1 | 24.0% | 0.47x |
| T2 | 15.0% | 0.54x |
| T3 | 10.0% | 0.61x |
| T4 | 6.0% | 0.68x |
| T5 | 3.5% | 0.74x |
| T6 | 2.0% | 0.80x |
| T7 | 1.0% | 0.86x |
| T8 | 0.4% | 0.93x |
| T9 | 0.1% | 1.00x |

Both attribute values and affix values are multiplied by the combined multiplier. God items always receive T9. Legacy saves default to T1.

## Generation Rules by Rarity

Base attribute ranges at ilvl 10 (scaled by `ilvl_multiplier × tier_multiplier`):

| Rarity    | Base Attr Range | Affixes | At ilvl 10 T9 | At ilvl 100 T9 (4.0x) | At ilvl 100 T0 (1.6x) |
|-----------|----------------|---------|--------------|----------------------|----------------------|
| Common    | 1              | 0       | 1-3 total    | 4-12 total           | 2-6 total            |
| Magic     | 1-2            | 1       | 1-6 total    | 4-24 total           | 1-10 total           |
| Rare      | 2-3            | 2-3     | 2-9 total    | 8-36 total           | 3-14 total           |
| Epic      | 3-4            | 3-4     | 3-12 total   | 12-48 total          | 5-19 total           |
| Legendary | 4-6            | 4-5     | 4-18 total   | 16-72 total          | 6-29 total           |

## Intrinsic Power Score (`types.rs` + `scoring.rs`)

Every item has a `power()` method that returns an intrinsic power score (displayed as ⚡ in cyan). This score is **character-independent** — the same item always produces the same power number regardless of who equips it.

```
power = sum(all_attribute_values) + sum(affix_value × affix_power_weight)
```

Affix power weights (from `affix_power_weight()`):
- DamagePercent: 2.0x (highest)
- CritChance, CritMultiplier: 1.5x
- DamageReduction: 1.3x
- AttackSpeed: 1.2x
- HPRegen, XPGain: 1.0x
- DamageReflection: 0.8x
- HPBonus: 0.5x (lowest)

Power score is shown in the equipment panel and scrolling loot ticker. God items intentionally do not score above T9 legendaries on the power formula — several options (scoring passives as affixes, a flat bonus, hand-set power, buffed stats) were considered and declined; see #272 (closed, not planned).

## Auto-Equip (`scoring.rs`)

Auto-equip uses the intrinsic `power()` score to decide whether a new item replaces the current one. If the new item has strictly higher power, it replaces the equipped item. Equal power keeps the incumbent.

**God item protection**: God (Mythic) items are never auto-replaced by lower rarity items.

## Mob Drop Rate Formula

```
base_chance = ITEM_DROP_BASE_CHANCE + (prestige_rank * ITEM_DROP_PRESTIGE_BONUS)
drop_chance = min(base_chance * (1.0 + haven_drop_bonus/100), ITEM_DROP_MAX_CHANCE)
```

Constants from `core/constants.rs`: 15% base, +1% per prestige rank, capped at 25%.

## Haven Integration

Two Haven rooms affect mob drops (boss drops are not affected):
- **Trophy Hall**: Increases drop rate percentage (applied multiplicatively to base chance)
- **Workshop**: Multiplicative bonus on non-Common rarity rates (e.g. T3 = ×1.25 on Magic/Rare/Epic). Max 25% bonus

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
3. Add weight in `scoring.rs` `affix_power_weight()` match
4. Add display name/formatting in `names.rs` if it affects item names
5. Apply the affix effect in `combat/logic.rs` or `character/derived_stats.rs`

## Adding a New Equipment Slot

1. Add variant to `EquipmentSlot` enum in `types.rs`
2. Add slot field and accessor in `equipment.rs` `Equipment` struct
3. Update `equipment.rs` iteration to include the new slot
4. Add name generation tables for the slot in `names.rs`
5. Update `ui/stats_equipment.rs` to display the new slot
6. Update serialization (Serde handles enum variants automatically)
