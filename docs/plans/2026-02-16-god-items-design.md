# God Items Design — Asprika (Phase 1)

## Overview

God Items are a new tier of named, fixed-stat equipment with unique passive abilities and bonuses. They are the first items in Quest with identity — every Asprika is the same Asprika, unlike procedurally-generated gear. Phase 1 ships Asprika only; additional god items (Brynhild, Gungnir, etc.) will be designed and added later.

## God Item Framework

### What Makes God Items Different

| Property | Normal Items | God Items |
|----------|-------------|-----------|
| Generation | Procedural (random stats/affixes) | Fixed (defined in code) |
| Rarity | Common → Legendary | Mythic (new tier above Legendary) |
| Attributes | 1-3 random attributes | 2 attributes: one primary, one supporting |
| Affixes | Random from pool | Fixed set per item |
| Unique passive | None | One per item (new combat mechanic) |
| Unique bonus | None | One per item (non-combat perk) |
| Acquisition | Enemy drops (RNG) | Quest chain + Temple Trial + forge cost |
| Enhancement | Soulforge +0 to +10 | Same — fully compatible with Soulforge |
| Auto-equip | Can be replaced by higher-scoring item | Never auto-replaced (Mythic always wins) |

### Design Principles

- **Always best-in-slot.** A god item should never be outscored by a random Legendary. Mythic rarity ensures auto-equip never replaces them.
- **2-attribute model.** Every god item has one primary stat and one supporting stat. They are specialists, not generalists.
- **Unique mechanics.** Each god item introduces a combat mechanic that doesn't exist on normal gear (evasion, life steal, armor penetration, etc.).
- **High challenge, high reward.** Acquisition requires mastery of multiple game systems plus significant prestige investment.
- **Enhanceable.** God items work with the existing Soulforge system. A +10 god item is the pinnacle of power.

### Mythic Rarity Tier

New rarity above Legendary. Display color: Cyan (or bright white with special treatment). Mythic items:
- Cannot be auto-replaced by lower-rarity items
- Have a distinct visual treatment in the equipment panel
- Show their passive ability and bonus inline with stats

---

## Asprika — Armor of the Æsir

### Identity

The ultimate defensive item. Asprika turns its wearer into an immovable wall that also accelerates progression while offline.

### Stats

- **Slot:** Armor
- **Rarity:** Mythic
- **Primary Attribute:** CON (very high) — massive max HP
- **Supporting Attribute:** WIS (moderate) — XP multiplier boost
- **Affixes:** DamageReduction, HPBonus, HPRegen, DamageReflection
- **Base stats:** ~2x a typical ilvl-100 Legendary (exact values tuned during implementation)

### Unique Passive — "Divine Bulwark"

All incoming damage reduced by **30%**, applied after the defense calculation step in the damage pipeline.

**Damage pipeline with Asprika:**
```
base_enemy_damage
  → subtract player defense
  → apply min 1
  → apply Divine Bulwark (× 0.70)
  → apply crit multiplier (if enemy crits)
  → final damage to player
```

Stacks multiplicatively with the DamageReduction affix on the item itself. If Asprika's DamageReduction affix provides 15% DR, total reduction = `1 - (0.70 × 0.85)` = 40.5%.

### Unique Bonus — "+100% Offline XP"

Doubles the offline XP accumulation rate. Hooks into the offline progression calculation in `game_logic.rs`. Stacks with Haven's offline XP bonuses.

### Acquisition Flow

```
1. Complete a Temple Trial (issue #98, design TBD)
     ↓
2. DISCOVERY: Asprika quest revealed
   - Player sees the item, its stats, passive, bonus
   - Quest requirements shown in Storm Forge → Divine Blueprints
     ↓
3. Complete quest requirements (parallel, any order):
   a. Win 3 different challenge minigames at Master difficulty
   b. Enhance 3 equipment slots to +7 via Soulforge
     ↓
4. Return to Temple Trial and complete it again
     ↓
5. Forge at Storm Forge — costs 50 Prestige Ranks
     ↓
6. Asprika equipped automatically (Mythic auto-equips over any Armor)
```

### Quest Requirements Detail

**Win 3 different challenges at Master difficulty:**
- Must be 3 distinct minigame types (e.g., Chess + Go + Snake)
- Master is the hardest difficulty tier
- Tests genuine player skill across different game types
- Tracked: which challenge types have been won at Master

**Enhance 3 equipment slots to +7:**
- Three separate equipment slots must reach +7 enhancement
- +7 has a 40% success rate with -1 level on failure
- Represents significant prestige rank investment (3 PR per attempt at +5-7)
- Tests commitment to the Soulforge system
- Tracked: count of slots currently at +7 or higher

**Complete Temple Trial (return visit):**
- Must complete the same Temple Trial that triggered discovery
- Proves the player can still handle the trial after progressing
- Design details depend on Temple Trial system (issue #98)

**Forge cost — 50 Prestige Ranks:**
- Spent at the Storm Forge after all requirements are met
- At P20 discovery, this means the player needs substantial additional prestige grinding
- Asprika is realistically forged around P50+ depending on other prestige sinks (Haven, Soulforge)

---

## System Architecture

### What to Build Now (Phase 1)

1. **God Item data model** (`src/items/god_items.rs` or similar)
   - `GodItemId` enum (starts with `Asprika`, expandable)
   - `GodItemDefinition` struct: slot, attributes, affixes, passive, bonus
   - `GodItemPassive` enum: `DivineBulwark { damage_reduction_percent: f64 }`
   - `GodItemBonus` enum: `OfflineXpMultiplier { multiplier: f64 }`
   - Static definitions (not generated)

2. **Mythic rarity tier** (`src/items/types.rs`)
   - Add `Mythic` variant to `Rarity` enum (above Legendary)
   - Update display color, ordering, and auto-equip logic
   - Mythic items are never auto-replaced by lower rarity

3. **God Item persistence** (`src/god_items/persistence.rs` or similar)
   - `GodItemProgress` struct per item: discovered, milestone progress, forged
   - Save/load from `~/.quest/god_items.json`
   - Account-level (persists across characters, like achievements)

4. **Passive integration** (`src/combat/logic.rs`, `src/character/derived_stats.rs`)
   - Divine Bulwark: 30% DR applied in damage pipeline
   - Check equipped god items for active passives during combat

5. **Bonus integration** (`src/core/game_logic.rs`)
   - Offline XP bonus: check equipped god items, apply multiplier

6. **Storm Forge UI** (`src/ui/soulforge_scene.rs` or new file)
   - "Divine Blueprints" section showing Asprika blueprint
   - Milestone progress display with checkmarks
   - Forge button (enabled when all requirements met + 50 PR available)

7. **Forge notification** (uses existing achievement modal system)
   - Special Mythic-themed modal on forge completion

### What to Build Later (Blocked on Dependencies)

- **Temple Trial discovery** — blocked on issue #98 (Temple Trials design)
- **Quest tracking UI** — blocked on quest system design (TBD)
- **Additional god items** — Brynhild, Gungnir, etc. designed separately

### Testing Strategy

- Unit tests for god item passive calculations (30% DR applied correctly)
- Unit tests for offline XP bonus integration
- Unit tests for Mythic rarity ordering and auto-equip behavior
- Integration tests for milestone tracking persistence
- Behavior-lock tests for damage pipeline with Divine Bulwark

---

## UI/UX

### Storm Forge — Divine Blueprints

```
╔═══════════════════════════════════════════════════════════╗
║              DIVINE BLUEPRINTS — STORM FORGE              ║
╠═══════════════════════════════════════════════════════════╣
║                                                           ║
║  ┌─ ASPRIKA — Armor of the Æsir ─────────────────────┐  ║
║  │  Rarity: ★ MYTHIC ★                               │  ║
║  │                                                     │  ║
║  │  ◆ Divine Bulwark: -30% incoming damage             │  ║
║  │  ◆ +100% Offline XP                                │  ║
║  │  ◆ CON ████████████ (primary)                      │  ║
║  │  ◆ WIS ██████ (supporting)                         │  ║
║  │                                                     │  ║
║  │  REQUIREMENTS:                                      │  ║
║  │  [✓] Win 3 Master challenges        3/3            │  ║
║  │  [✗] Enhance 3 slots to +7          1/3            │  ║
║  │  [✗] Complete Temple Trial (return)                 │  ║
║  │                                                     │  ║
║  │  Forge Cost: 50 Prestige Ranks                      │  ║
║  │  Status: Requirements incomplete                    │  ║
║  └─────────────────────────────────────────────────────┘  ║
║                                                           ║
║  [ESC] Back                                               ║
╚═══════════════════════════════════════════════════════════╝
```

### Equipment Panel Display

```
⚔ Asprika +3 ★MYTHIC★
  Divine Bulwark: -30% DMG taken
  +100% Offline XP
  CON +48  WIS +24
  DmgRed +18%  HP +150  HPRegen +12%
```

### Combat Log

```
Enemy attacks for 45 → 32 (Divine Bulwark)
```

### Forge Notification (Modal)

```
★ MYTHIC ITEM FORGED ★
Asprika — Armor of the Æsir
Divine Bulwark: -30% incoming damage
```

---

## Future God Items (Phase 2+)

Designed later. Candidates discussed during brainstorming:

| Item | Slot | Gate | Passive Concept | Status |
|------|------|------|----------------|--------|
| Asprika | Armor | P20 | Divine Bulwark (30% DR) | **Phase 1 — this doc** |
| Brynhild | Ring | ~P35 | Life steal (TBD) | Design pending |
| Gungnir | Weapon | ~P50 | Armor penetration (TBD) | Design pending |

All god items follow the same framework: Norse mythology-inspired names, 2-attribute model (primary + supporting), unique passive + unique bonus, Temple Trial discovery, mastery-gate milestones, prestige rank forge cost.

---

## Dependencies

- **Issue #98 — Temple Trials:** Discovery mechanism for god items. Asprika quest is revealed on first Temple Trial completion. Must complete trial again to acquire item.
- **Quest tracking system (TBD):** Minimal for Phase 1 — just milestone counters and a state enum (undiscovered → discovered → requirements_met → forged). May be designed as part of #98 or as a standalone system.

## Open Questions

- Exact stat values for Asprika (attribute amounts, affix percentages) — to be tuned during implementation
- Should the 30% DR from Divine Bulwark be visible as a separate line in the stats panel, or folded into the total damage reduction stat?
- Should Asprika's +100% offline XP show in the stats panel alongside the Haven offline XP bonus?
