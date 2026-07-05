> Backported design record. Sources: docs/plans/2026-02-17-item-tier-system-design.md.

## 2026-02-17-item-tier-system-design.md

# Item Tier System Design

## Problem

At high prestige, most item drops converge to Epic/Legendary and all feel the same. There's no diversity within a rarity — a Legendary is a Legendary regardless of how well it rolled.

## Solution

Add a **Tier** dimension (T0-T9) to every item. Tier is an independent quality roll that multiplies stat values. Combined with rarity and ilvl, this creates 550 distinct item profiles (5 rarities x 10 tiers x 11 zones).

## Three Axes of Item Power

```
         Epic       T7        70
         ────       ──        ──
         Rarity     Tier      ilvl
         (affixes)  (quality) (zone)
```

- **Rarity** (Common/Magic/Rare/Epic/Legendary): Determines affix count. Rolled from existing drop tables (unchanged).
- **Tier** (T0-T9): Quality multiplier on stat values. Rolled independently with an exponential curve. **New.**
- **ilvl** (10-110): Zone-based multiplier on stat values. Fixed per zone (unchanged).

**Power formula:**
```
stat_value = base_range × ilvl_multiplier × tier_multiplier
```

## Tier Multipliers and Drop Rates

Exponential curve heavily favoring low tiers. T9 is a 1-in-1,000 roll.

| Tier | Multiplier | Drop Rate | Odds |
|------|-----------|-----------|------|
| T0 | 0.40x | 38.0% | 1 in 3 |
| T1 | 0.47x | 24.0% | 1 in 4 |
| T2 | 0.54x | 15.0% | 1 in 7 |
| T3 | 0.61x | 10.0% | 1 in 10 |
| T4 | 0.68x | 6.0% | 1 in 17 |
| T5 | 0.74x | 3.5% | 1 in 29 |
| T6 | 0.80x | 2.0% | 1 in 50 |
| T7 | 0.86x | 1.0% | 1 in 100 |
| T8 | 0.93x | 0.4% | 1 in 250 |
| T9 | 1.00x | 0.1% | 1 in 1,000 |

**Distribution summary:**
- T0-T3: 87% of drops (trash to decent)
- T4-T6: 11.5% of drops (good)
- T7-T9: 1.5% of drops (exceptional to legendary quality)

**Chase item rarity:** Legendary T9 from a Zone 10 boss = 5% Legendary × 0.1% T9 = 1 in 200,000 boss kills.

## Power Overlap Between Tiers

A high-tier lower-rarity item can beat a low-tier higher-rarity item. At the same ilvl:

```
Zone 7 (ilvl 70), ilvl_multiplier = 3.0x

                          combined    attrs/stat    affix values
Legendary T9  (3.0×1.00)  3.00x      12-18         18-30
Epic T9       (3.0×1.00)  3.00x       9-12         12-18
Legendary T0  (3.0×0.40)  1.20x       5-7           7-12
Rare T9       (3.0×1.00)  3.00x       6-9           6-12
Epic T0       (3.0×0.40)  1.20x       4-5           5-7
```

A `Rare T9` beats an `Epic T0` through `T4`. About 5 sub-tiers of overlap.

## Generation Pipeline (Updated)

```
1. Drop roll       → same as today (15-25% mobs, 100% bosses)
2. Rarity roll     → same as today (mob/boss tables)
3. Slot roll       → same as today (uniform across 7 slots)
4. ilvl            → same as today (zone_id × 10)
5. Tier roll       → NEW: exponential curve T0-T9
6. Generate attrs  → base from rarity × ilvl_mult × tier_mult (CHANGED)
7. Generate affixes→ count from rarity, values × ilvl_mult × tier_mult (CHANGED)
8. Generate name   → same as today
9. Auto-equip      → same as today (reads real stat values)
```

### Tier Roll Implementation

```rust
fn roll_tier(rng: &mut impl Rng) -> u8 {
    let roll: f64 = rng.random();
    if roll < 0.380 { 0 }       // 38.0%
    else if roll < 0.620 { 1 }  // 24.0%
    else if roll < 0.770 { 2 }  // 15.0%
    else if roll < 0.870 { 3 }  // 10.0%
    else if roll < 0.930 { 4 }  //  6.0%
    else if roll < 0.965 { 5 }  //  3.5%
    else if roll < 0.985 { 6 }  //  2.0%
    else if roll < 0.995 { 7 }  //  1.0%
    else if roll < 0.999 { 8 }  //  0.4%
    else { 9 }                   //  0.1%
}

fn tier_multiplier(tier: u8) -> f64 {
    match tier {
        0 => 0.40,
        1 => 0.47,
        2 => 0.54,
        3 => 0.61,
        4 => 0.68,
        5 => 0.74,
        6 => 0.80,
        7 => 0.86,
        8 => 0.93,
        9 => 1.00,
        _ => 0.74, // fallback to T5
    }
}
```

### Updated `generate_item()`

```rust
pub fn generate_item(slot: EquipmentSlot, rarity: Rarity, ilvl: u32) -> Item {
    let mut rng = rand::rng();
    let tier = roll_tier(&mut rng);
    let attributes = generate_attributes(rarity, ilvl, tier, &mut rng);
    let affixes = generate_affixes(rarity, ilvl, tier, &mut rng);
    // ...rest unchanged...
}
```

### Updated `generate_attributes()`

The tier multiplier is applied alongside the existing ilvl multiplier:

```rust
fn generate_attributes(rarity: Rarity, ilvl: u32, tier: u8, rng: &mut impl Rng) -> AttributeBonuses {
    let (base_min, base_max) = match rarity { /* unchanged */ };
    let multiplier = ilvl_multiplier(ilvl) * tier_multiplier(tier);
    // ...rest unchanged, uses combined multiplier...
}
```

Same change in `generate_affixes()` and `generate_affix_value()`.

## No Power Creep

T9 = 1.00x multiplier = the current system's maximum possible roll. The power ceiling does not change. Most drops become weaker than today's average, making high-tier drops feel special.

## Backwards Compatibility

### Item Struct Change

```rust
pub struct Item {
    pub slot: EquipmentSlot,
    pub rarity: Rarity,
    pub ilvl: u32,
    #[serde(default = "default_tier")]
    pub tier: u8,                       // NEW FIELD
    pub base_name: String,
    pub display_name: String,
    pub attributes: AttributeBonuses,
    pub affixes: Vec<Affix>,
    pub god_item_id: Option<GodItemId>,
}

fn default_tier() -> u8 { 5 }  // T5 = 0.74x
```

- Existing items missing the `tier` field deserialize as T5
- Their stats are unchanged — the T5 label is approximate but reasonable
- No migration, no stat recalculation, no risk of breaking saves

### God Items (Mythic)

God items always get `tier: 9`. They are hand-crafted with fixed stats, so the tier multiplier is not applied during generation — T9 is purely a display label indicating maximum quality.

## UI Changes

### Stats Panel — Equipment List

```
┌─ Equipment ──────────────────────────────────────────────┐
│ Weapon  Jade Blade of the Eternal   Legendary T9     100 │  ← LightRed
│ Armor   Iron Plate of Wrath        Epic T7           70 │  ← Magenta
│ Helm    Burning Crown              Rare T9           80 │  ← Yellow
│ Gloves  Shadow Gauntlets           Epic T1           70 │  ← Magenta
│ Boots   Fine Greaves               Magic T5          90 │  ← Blue
│ Amulet  Sturdy Pendant             Rare T3           40 │  ← Yellow
│ Ring    Copper Band                Common T0          10 │  ← White
└──────────────────────────────────────────────────────────┘
```

Format: `{name}  {Rarity} T{tier} {ilvl}` — right-aligned, in rarity color.

### Loot Ticker

```
⚔ Legendary T9 Jade Blade 🔨  ⚔ Epic T1 Gloves  ⚔ Rare T3 Pendant
```

Format: `{Rarity} T{tier} {name}` — replaces the current `[R]` initial with full rarity name and tier.

### Ticker Entry Changes

Current format:
```rust
let text = format!("[{}] {}{}", rarity_initial, item_name, equip_tag);
```

New format:
```rust
let text = format!("{} T{} {}{}", rarity.name(), tier, item_name, equip_tag);
```

The `rarity_initial` match blocks (C/M/R/E/L/G) are removed. Full rarity name used instead.

### Bold/Emphasis Rules

Current: bold for Epic/Legendary/Mythic.
Updated: bold for T7+ at any rarity OR Epic+. A `Rare T8` deserves emphasis.

```rust
bold: tier >= 7 || matches!(rarity, Rarity::Epic | Rarity::Legendary | Rarity::Mythic),
```

## Files Changed

| File | Change | Scope |
|------|--------|-------|
| `items/types.rs` | Add `tier: u8` field with serde default | Small |
| `items/generation.rs` | Add `roll_tier()`, `tier_multiplier()`, thread tier through generation | Medium |
| `core/constants.rs` | Add tier drop rate thresholds and multiplier constants | Small |
| `tick_events.rs` | Update ticker format from `[R]` to `Rare T{tier}`, update bold rule | Small |
| `ui/stats_panel.rs` | Display tier and ilvl in equipment panel | Small |
| `ui/haven_scene.rs` | Display tier in prestige item preservation list | Small |
| `god_items/types.rs` | Set `tier: 9` on all god items | Small |
| `items/scoring.rs` | No changes — scoring reads actual stat values | None |
| `items/drops.rs` | No changes — tier is rolled inside `generate_item()` | None |
| `items/names.rs` | No changes — names are independent of tier | None |

## What Does NOT Change

- Rarity drop tables (mob/boss distributions)
- Affix count per rarity (0/1/2-3/3-4/4-5)
- ilvl formula (zone_id × 10)
- ilvl multiplier formula (1.0 + (ilvl-10)/30)
- Auto-equip scoring (reads real stat values, naturally prefers higher tiers)
- Dungeon rarity boosting (`boost_rarity()` boosts rarity, not tier — tier rolls independently)
- Haven drop rate and rarity bonuses (affect rarity, not tier)
- Fishing item drops (tier rolls inside `generate_item()`)
- God item stats (hand-crafted, tier is display-only)

## Testing

- Verify `roll_tier()` distribution over 100k samples matches expected percentages within tolerance
- Verify `tier_multiplier()` returns correct values for all tiers 0-9
- Verify `generate_item()` produces stat values scaled by tier: T9 items at ilvl X should match current items at ilvl X
- Verify T0 items are ~40% of T9 power at same rarity/ilvl
- Verify existing items deserialize with tier=5
- Verify god items have tier=9
- Verify auto-equip correctly prefers T9 over T0 at same rarity/ilvl
- Verify display format shows `{Rarity} T{tier} {ilvl}` in stats panel and ticker
