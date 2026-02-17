# God Items Design

## Overview

God Items are a new tier of named, fixed-stat equipment with unique passive abilities and bonuses. They are the first items in Quest with identity — every Asprika is the same Asprika, unlike procedurally-generated gear. Three god items are implemented: Asprika, Sleipnir, and Megingjord.

## God Item Framework

### What Makes God Items Different

| Property | Normal Items | God Items |
|----------|-------------|-----------|
| Generation | Procedural (random stats/affixes) | Fixed (defined in code) |
| Rarity | Common → Legendary | Mythic (new tier above Legendary) |
| Attributes | 1-3 random attributes | 2 attributes: one primary, one supporting |
| Affixes | Random from pool | Fixed: +40% XP Gain on all god items |
| Unique passive | None | One per item (combat mechanic) |
| Unique bonuses | None | 1-3 per item (non-combat perks) |
| Acquisition | Enemy drops (RNG) | Discovery → milestones → forge (50 PR) |
| Enhancement | Soulforge +0 to +10 | Same — fully compatible with Soulforge |
| Auto-equip | Can be replaced by higher-scoring item | Never auto-replaced (Mythic always wins) |

### Design Principles

- **Always best-in-slot.** A god item should never be outscored by a random Legendary. Mythic rarity ensures auto-equip never replaces them.
- **2-attribute model.** Every god item has one primary stat (40) and one supporting stat (20).
- **Unique mechanics.** Each god item introduces combat mechanics that don't exist on normal gear.
- **High challenge, high reward.** Acquisition requires mastery of multiple game systems plus 50 prestige ranks.
- **Enhanceable.** God items work with the existing Soulforge system. A +10 god item is the pinnacle of power.

### Mythic Rarity Tier

New rarity above Legendary. Mythic items:
- Cannot be auto-replaced by lower-rarity items
- Have a distinct visual treatment in the equipment panel
- Show their passive ability and bonuses inline with stats

---

## God Items

### Asprika — Armor of the Aesir

The ultimate defensive item. Turns its wearer into an immovable wall that accelerates offline progression.

| Property | Value |
|----------|-------|
| **Slot** | Armor |
| **Stats** | CON 40 / WIS 20 |
| **Affix** | +40% XP Gain |
| **Passive** | Divine Bulwark — 30% damage reduction (after defense calc) |
| **Bonus** | +100% Offline XP (stacks with Haven) |
| **Requirement** | Beyond Infinity I (complete 1 Expanse cycle) |
| **Forge Cost** | 50 Prestige Ranks |

**Damage pipeline with Asprika:**
```
base_enemy_damage → subtract defense → min 1 → Divine Bulwark (x 0.70) → final damage
```

### Sleipnir — Boots of the Eight-Legged

The speed god item. Makes everything faster — combat, dungeons, fishing, and recovery.

| Property | Value |
|----------|-------|
| **Slot** | Boots |
| **Stats** | DEX 40 / WIS 20 |
| **Affix** | +40% XP Gain |
| **Passive** | Windborne — 100% Attack Speed (halves attack interval) |
| **Bonus 1** | Swiftstrider — 50% HP regen delay reduction (multiplicative with Haven) |
| **Bonus 2** | Swiftfoot — 50% dungeon movement speed (explore 2.5s→1.25s, travel 0.8s→0.4s) |
| **Bonus 3** | Nimble Hands — 50% fishing timer reduction (multiplicative with Haven Garden) |
| **Requirement** | 3 Master challenge wins + Soulforge Savant (+7) |
| **Forge Cost** | 50 Prestige Ranks |

**Attack speed integration:**
```
player_interval = 1.5s / (derived.attack_speed_multiplier + 1.0) = 0.75s
```

### Megingjord — Belt of Giant Strength

The raw power god item. Massive damage boost with accelerated prestige progression.

| Property | Value |
|----------|-------|
| **Slot** | Ring |
| **Stats** | STR 40 / CON 20 |
| **Affix** | +40% XP Gain |
| **Passive** | Giant's Might — 150% Damage (applied before Haven multiplier) |
| **Bonus** | Prestige Mastery — 2x prestige XP multiplier (online + offline) |
| **Requirement** | Soulforge Grandmaster (+9) |
| **Forge Cost** | 50 Prestige Ranks |

**Damage pipeline with Megingjord:**
```
base_damage → Giant's Might (x 2.5) → Haven Armory (%) → prestige flat → defense → crit
```

---

## State Machine

All god items follow the same acquisition flow:

```
Undiscovered → Discovered → ReadyToForge → Forged (50 PR)
```

- **Undiscovered**: Item not yet revealed to the player
- **Discovered**: Player sees the item and its milestone requirements
- **ReadyToForge**: All milestones complete, player can forge at Soulforge
- **Forged**: Item created and auto-equipped (cannot be unequipped or replaced)

---

## System Architecture

### Files

| File | Purpose |
|------|---------|
| `src/god_items/types.rs` | GodItemId, GodItemPassive, GodItemBonus enums, definitions, helpers, milestones |
| `src/god_items/persistence.rs` | Save/load GodItemProgress from `~/.quest/god_items.json` |
| `src/combat/logic.rs` | GodItemCombatBonuses struct, wired into damage pipeline |
| `src/core/offline.rs` | Prestige XP multiplier + offline XP bonus parameters |
| `src/dungeon/logic.rs` | Dungeon speed percent parameter on update_dungeon() |
| `src/fishing/logic.rs` | Fishing reduction percent parameter on tick_fishing_with_haven_result() |
| `src/core/tick.rs` | Builds bonuses from equipped items, passes to combat/dungeon/fishing |
| `src/items/types.rs` | Mythic rarity tier |
| `src/utils/debug_menu.rs` | Debug shortcuts for discover/complete/forge each item |

### Integration

```
god_items/types.rs (definitions + helpers)
        │
        ├──→ combat/logic.rs     (GodItemCombatBonuses: DR, atk speed, regen, damage)
        ├──→ core/offline.rs     (prestige XP mult, offline XP bonus)
        ├──→ dungeon/logic.rs    (dungeon speed percent)
        ├──→ fishing/logic.rs    (fishing timer reduction)
        │
        └──→ core/tick.rs        (orchestrates all of the above per tick)
```

### AttackSpeed Affix Nerf

Random AttackSpeed affixes were nerfed to 1/3 base ranges so Sleipnir's Windborne (100%) stands out:
- Legendary at ilvl 100: 8-13% (was 24-40%)

---

## Dependencies

- **Temple Trials (issue #98)**: Player-facing discovery mechanism. Currently only accessible via debug menu.
- **Forge UI**: No player-facing forge screen yet. `GOD_ITEM_FORGE_COST = 50` constant is defined and ready.

## Design Docs

- `docs/plans/2026-02-16-sleipnir-speed-bonuses-design.md` — Swiftfoot + Nimble Hands design
