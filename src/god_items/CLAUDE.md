# God Items Module

Three Norse mythology-themed endgame items with unique combat passives and non-combat bonuses.

## Files

| File | Purpose |
|------|---------|
| `mod.rs` | Re-exports all types from `types.rs` |
| `types.rs` | `GodItemId` enum, `GodItemPassive` enum, `GodItemBonus` enum, `GodItemDefinition` struct, item definitions (Asprika, Sleipnir, Megingjord), and equipped-item query helpers |

## Key Types

- **`GodItemId`**: Enum with three variants -- `Asprika`, `Sleipnir`, `Megingjord`. Serializable. Stored in the `god_item_id: Option<GodItemId>` field on `Item`.
- **`GodItemPassive`**: Combat passive abilities -- `DivineBulwark { damage_reduction_percent }`, `Windborne { attack_speed_percent }`, `GiantsMight { damage_percent }`.
- **`GodItemBonus`**: Non-combat bonuses -- `Swiftstrider { regen_reduction_percent }`, `Swiftfoot { dungeon_speed_percent }`, `NimbleHands { fishing_reduction_percent }`.
- **`GodItemDefinition`**: Static definition containing `id`, `name`, `title`, `slot`, `attributes`, `affixes`, `passive`, and `bonuses`. Has a `to_item()` method that creates a Rarity::Mythic Item with ilvl 100, tier 9.
- **`CachedGodItemBonuses`**: Aggregated bonus values (DR%, attack speed%, damage%, regen/dungeon/fishing reductions) computed once via `compute(equipment)` in a single pass over equipped items, avoiding per-tick equipment scans.

## Item Definitions

| Item | Slot | Attributes | Passive | Non-Combat Bonuses | XP |
|------|------|-----------|---------|-------------------|-----|
| Asprika | Armor | +40 CON, +20 WIS | Divine Bulwark: 30% damage reduction (post-defense) | -- | +40% |
| Sleipnir | Boots | +40 DEX, +20 WIS | Windborne: 100% attack speed | Swiftstrider (50% regen reduction), Swiftfoot (50% dungeon speed), NimbleHands (50% fishing speed) | +40% |
| Megingjord | Ring | +40 STR, +20 CON | Giant's Might: 150% damage | -- | +40% |

All three items have a single XPGain affix (+40%) and are created as Rarity::Mythic, ilvl 100, tier 9.

## How It Works

God items are created via the debug menu (discovery/forging system not yet designed). Each item's `to_item()` method produces a standard `Item` struct with `god_item_id` set to `Some(id)`.

**Auto-equip protection**: The item scoring system in `items/scoring.rs` never replaces a god item (Rarity::Mythic) with a lower rarity item.

**Equipped-item query helpers**: Six public functions scan the equipment for god items and return their bonus values (or 0.0 if not equipped):
- `equipped_god_item_dr()` -- Divine Bulwark damage reduction %
- `equipped_god_item_attack_speed_percent()` -- Windborne attack speed %
- `equipped_god_item_damage_percent()` -- Giant's Might damage %
- `equipped_god_item_regen_reduction_percent()` -- Swiftstrider regen reduction %
- `equipped_god_item_dungeon_speed_percent()` -- Swiftfoot dungeon speed %
- `equipped_god_item_fishing_reduction_percent()` -- NimbleHands fishing reduction %

## Integration Points

- **Combat pipeline** (`combat/player_attack.rs`): Giant's Might damage bonus applied early in damage calculation
- **Combat pipeline** (`combat/enemy_attack.rs`): Divine Bulwark DR applied after defense
- **Attack speed** (`combat/orchestration.rs`, `core/power_rating.rs`): Player's `attack_speed_percent` (carrying Windborne) reduces the effective attack interval; `combat/attacks.rs` only computes the enemy-side `effective_enemy_attack_interval()`
- **Derived stats** (`character/derived_stats.rs`): XP gain affix contributes to XP multiplier
- **HP regen** (`combat/regen.rs`): Swiftstrider reduces regen delay between encounters via `bonuses.regen_reduction_percent`
- **Dungeon movement** (`dungeon/logic.rs`): Swiftfoot reduces room movement timers
- **Fishing timers** (`fishing/logic.rs`): NimbleHands reduces fishing phase durations
- **Debug menu** (`utils/debug_menu.rs`): Forge actions create and equip god items
- **UI** (`ui/stats_equipment.rs`): God items displayed with Mythic (God) rarity styling
