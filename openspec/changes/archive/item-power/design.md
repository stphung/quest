> Backported design record. Sources: docs/plans/2026-02-18-item-power-design.md.

## 2026-02-18-item-power-design.md

# Item Power (⚡) Design

## Summary

Add an intrinsic power number to items, displayed as ⚡ followed by an integer. The number reflects an item's total stat budget — attributes plus weighted affixes — independent of the player's character build.

## Formula

```
power = sum(all attribute values) + sum(affix_value × affix_weight)
```

Computed on-the-fly via `Item::power() -> u32`. No storage, no save migration.

- **Attribute weights**: all 1.0 (equal, no character dependency)
- **Affix weights**: reused from `scoring.rs` auto-equip logic:
  - DamagePercent: 2.0
  - CritChance, CritMultiplier: 1.5
  - DamageReduction: 1.3
  - AttackSpeed: 1.2
  - HPRegen, XPGain: 1.0
  - DamageReflection: 0.8
  - HPBonus: 0.5
  - Unknown: 0.0

Result rounded to nearest `u32`.

## Expected Power Ranges

| Zone | Rarity | Tier | ⚡ Power |
|------|--------|------|---------|
| 1 | Common | T0 | 1-3 |
| 1 | Magic | T3 | 3-9 |
| 5 | Rare | T5 | 40-80 |
| 7 | Epic | T6 | 60-100 |
| 10 | Legendary | T7 | 250-350 |
| 10 | Legendary | T9 | 300-400 |
| — | God items | T9 | ~100 (see #272) |

God items score low because their passives (150% damage, 100% attack speed, 30% DR) are not captured by this formula. Issue #272 tracks resolving this.

## Display

### Equipment Panel (`stats_panel.rs`)

Appended after ilvl:
```
Weapon  +3 Thunderstrike    Legendary  T7  i70  ⚡326
```
Color: Cyan. Follows existing bold rules.

### Loot Ticker (`tick_events.rs`)

New segment after ilvl:
```
⚔ Legendary T7 Thunderstrike i70 ⚡326 🔨
```
Color: Cyan.

## Implementation

1. `src/items/scoring.rs` — extract affix weights to `pub const AFFIX_POWER_WEIGHTS`
2. `src/items/types.rs` — add `Item::power() -> u32` method using shared weights
3. `src/core/tick.rs` — add `power: u32` field to `TickEvent::ItemDropped`
4. `src/tick_events.rs` — add ⚡ segment to ticker entry
5. `src/ui/stats_panel.rs` — add ⚡ column to equipment display
