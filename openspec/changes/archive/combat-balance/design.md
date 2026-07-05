> Backported design record. Sources: docs/design/combat-balance-architecture.md, docs/design/combat-balance-core.md, docs/design/combat-balance-prestige.md.

## combat-balance-architecture.md

# Combat Balance System Architecture

**Issue:** #123 — Combat is broken because enemies scale dynamically with player stats
**Status:** Design
**Author:** System Architect

---

## 1. Root Cause Analysis

### Problem 1: Dynamic Enemy Scaling Makes Progression Impossible

**Current behavior** (`src/combat/types.rs:58-77`):

```rust
// Enemy HP = player_max_hp * variance(0.8-1.2) * zone_mult * boss_mult
let max_hp = ((player_max_hp as f64 * hp_variance * stat_multiplier) as u32).max(10);

// Enemy damage = (player_max_hp / 7.0) * variance(0.8-1.2) * stat_multiplier
let damage = ((player_max_hp as f64 / 7.0 * damage_variance * stat_multiplier) as u32).max(1);
```

Every enemy stat formula takes `player_max_hp` as the base. As the player levels up and gains CON, their HP rises, and enemies rise in lockstep. Zone/subzone multipliers (10% per zone, 5% per subzone depth) are tiny modifiers on top of the player's own stats — they do not create meaningful difficulty curves between zones.

**Root cause:** The enemy scaling model is relative (percentage of player), not absolute (zone-based static values). Players can never outgear or outlevel content.

### Problem 2: Boss Multipliers Are Too High

**Current behavior** (`src/combat/types.rs:155-171`):

- Subzone boss: 2.0x HP, 1.5x damage (on top of the already player-matched base)
- Zone boss: 3.0x HP, 2.0x damage

Since the base enemy already has 80-120% of the player's HP, a zone boss effectively has **240-360% of the player's HP** and deals damage proportional to **~29% of the player's HP per hit** (player_hp / 7.0 * 2.0). The player's total_damage at base stats is 10 (5 phys + 5 magic), while the boss HP is hundreds. Fights become unwinnable.

**Root cause:** Boss multipliers compound with the already-matched base, creating an unwinnable stat gap.

### Problem 3: Prestige Provides Zero Combat Advantage

**Current behavior** (`src/character/prestige.rs:128-170`):

Prestige resets: level -> 1, XP -> 0, attributes -> base 10, equipment -> cleared.
Prestige provides: XP multiplier (`1.0 + 0.5 * rank^0.7`), attribute cap increase (`20 + 5*rank`).

The XP multiplier only speeds up leveling — it does not make the character stronger in combat. The higher attribute cap only matters if you reach high enough levels to fill it. Since enemies scale with player stats, even reaching those caps provides no advantage.

**Root cause:** Prestige only affects XP speed, not combat power. There is no combat-relevant reward for prestiging.

### Problem 4: Fight Duration Is Wrong (0.3-0.5s vs Design Target 5-10s)

**Current behavior** (`src/combat/logic.rs:147-148`):

```rust
let player_interval = ATTACK_INTERVAL_SECONDS / derived.attack_speed_multiplier; // 1.5s base
let enemy_interval = effective_enemy_attack_interval(state); // 2.0s base
```

Player damage at base stats: `total_damage() = 10` (5 phys + 5 magic).
Enemy HP at base stats: `player_max_hp * 0.8-1.2 = 40-60` (player has 50 HP at base).
Hits to kill: 4-6 hits at 1.5s each = 6-9s. This *looks* correct.

BUT: Enemy damage = `50 / 7.0 * 0.8-1.2 = 5-8`, and player defense = 0 at base.
Player dies in 6-10 hits at 2.0s each = 12-20s. This also looks correct.

The 0.3-0.5s fights reported in the issue likely occur at higher levels where stats scale non-linearly. As attributes grow, player damage outpaces enemy HP because `total_damage()` scales faster than the `player_max_hp` that enemies use as their HP base. This is an artifact of the relative scaling — damage and HP don't scale at the same rate.

**Root cause:** Relative scaling makes fight duration unpredictable and level-dependent rather than zone-dependent.

### Problem 5: 65% of P0 Players Stuck in Zone 1 After 14 Hours

Players cannot beat zone bosses. The zone boss has 3x their HP and 2x their damage. The player's damage is ~10 while the boss has ~150 HP. That is 15 hits at 1.5s = 22.5s to kill the boss, while the boss kills the player in ~7 hits at 1.5s = 10.5s. The player dies first every time unless they get lucky with crits.

**Root cause:** Combination of problems 1 and 2. No amount of leveling helps because enemies scale with you.

### Problem 6: Boss Death Resets Kill Counter

**Current behavior** (`src/combat/logic.rs:381-384`):

```rust
if state.zone_progression.fighting_boss {
    state.zone_progression.fighting_boss = false;
    state.zone_progression.kills_in_subzone = 0;
    // Must kill 10 more mobs before boss spawns again
}
```

Dying to a boss forces the player to kill 10 more mobs, extending the time between boss attempts by ~30-50 seconds (10 fights + regen). For bosses that are mathematically unbeatable, this adds insult to injury.

---

## 2. Proposed Architecture: Zone-Based Static Enemy Scaling

### Core Principle

Replace `player_max_hp`-based enemy generation with **zone-defined base stats** that are independent of the player. The player's power should come from attributes, equipment, and prestige — and should be compared against fixed enemy stats, creating genuine progression.

### 2.1 New Enemy Stat Model

**Replace** the current `generate_enemy_with_multiplier(player_max_hp, stat_multiplier)` with a new function:

```rust
/// Generates an enemy with zone-based static stats.
/// Stats are determined entirely by zone_id, subzone depth, and boss tier.
/// Player stats are NOT used as input.
pub fn generate_zone_enemy_static(zone: &Zone, subzone: &Subzone) -> Enemy {
    let base_hp = zone_base_hp(zone.id);
    let base_damage = zone_base_damage(zone.id);

    // Subzone depth scaling: +15% per depth level beyond 1
    let depth_mult = 1.0 + (subzone.depth as f64 - 1.0) * 0.15;

    let hp = (base_hp as f64 * depth_mult * variance(0.9, 1.1)) as u32;
    let damage = (base_damage as f64 * depth_mult * variance(0.9, 1.1)) as u32;

    Enemy::new(generate_zone_enemy_name(zone.id), hp.max(1), damage.max(1))
}
```

### 2.2 Zone Base Stat Table

Stats are tuned to match the expected player power at each zone's level range, assuming the player has attributes near the zone's `min_level` attribute cap.

| Zone | Name | Prestige | Level Range | Base HP | Base Damage | Subzone Boss HP Mult | Subzone Boss DMG Mult | Zone Boss HP Mult | Zone Boss DMG Mult |
|------|------|----------|-------------|---------|-------------|---------------------|-----------------------|-------------------|--------------------|
| 1 | Meadow | P0 | 1-10 | 30 | 4 | 1.5x | 1.2x | 2.0x | 1.5x |
| 2 | Dark Forest | P0 | 10-25 | 50 | 6 | 1.5x | 1.2x | 2.0x | 1.5x |
| 3 | Mountain Pass | P5 | 25-40 | 80 | 9 | 1.5x | 1.2x | 2.0x | 1.5x |
| 4 | Ancient Ruins | P5 | 40-55 | 110 | 12 | 1.5x | 1.2x | 2.0x | 1.5x |
| 5 | Volcanic Wastes | P10 | 55-70 | 160 | 16 | 1.5x | 1.2x | 2.0x | 1.5x |
| 6 | Frozen Tundra | P10 | 70-85 | 210 | 20 | 1.5x | 1.2x | 2.0x | 1.5x |
| 7 | Crystal Caverns | P15 | 85-100 | 280 | 26 | 1.5x | 1.2x | 2.0x | 1.5x |
| 8 | Sunken Kingdom | P15 | 100-115 | 360 | 32 | 1.5x | 1.2x | 2.0x | 1.5x |
| 9 | Floating Isles | P20 | 115-130 | 460 | 40 | 1.5x | 1.2x | 2.0x | 1.5x |
| 10 | Storm Citadel | P20 | 130-150 | 580 | 50 | 1.5x | 1.2x | 2.5x | 1.8x |
| 11 | The Expanse | Post | 150+ | 700 | 60 | 1.5x | 1.2x | 2.5x | 1.8x |

**Design rationale:**
- Zone 1 base HP (30) is beatable by a fresh character with 10 total damage in ~3 hits (4.5s)
- Zone boss HP mult reduced from 3.0x/2.0x to 2.0x/1.5x — still a challenge but not a wall
- Subzone boss mult reduced from 2.0x/1.5x to 1.5x/1.2x — speed bumps, not roadblocks
- Zone 10 boss is intentionally tougher (2.5x/1.8x) as the penultimate challenge
- Each zone's base stats roughly double every 3-4 zones, matching the expected player power curve

### 2.3 Implementation Location

Add new constants to `src/core/constants.rs`:

```rust
/// Zone base enemy stats: (base_hp, base_damage)
/// Index 0 = Zone 1, Index 9 = Zone 10, Index 10 = Zone 11
pub const ZONE_BASE_STATS: [(u32, u32); 11] = [
    (30, 4),    // Zone 1: Meadow
    (50, 6),    // Zone 2: Dark Forest
    (80, 9),    // Zone 3: Mountain Pass
    (110, 12),  // Zone 4: Ancient Ruins
    (160, 16),  // Zone 5: Volcanic Wastes
    (210, 20),  // Zone 6: Frozen Tundra
    (280, 26),  // Zone 7: Crystal Caverns
    (360, 32),  // Zone 8: Sunken Kingdom
    (460, 40),  // Zone 9: Floating Isles
    (580, 50),  // Zone 10: Storm Citadel
    (700, 60),  // Zone 11: The Expanse
];

pub const SUBZONE_DEPTH_SCALING: f64 = 0.15;

pub const SUBZONE_BOSS_HP_MULT: f64 = 1.5;
pub const SUBZONE_BOSS_DMG_MULT: f64 = 1.2;
pub const ZONE_BOSS_HP_MULT: f64 = 2.0;
pub const ZONE_BOSS_DMG_MULT: f64 = 1.5;
pub const FINAL_ZONE_BOSS_HP_MULT: f64 = 2.5;
pub const FINAL_ZONE_BOSS_DMG_MULT: f64 = 1.8;
```

### 2.4 Dungeon Enemy Scaling

Dungeon enemies currently use the same `player_max_hp`-based generation. Change to zone-based:

```rust
// Dungeon enemies use the zone stats where the dungeon was discovered
pub fn generate_dungeon_enemy(zone_id: u32) -> Enemy { ... }
pub fn generate_dungeon_elite(zone_id: u32) -> Enemy { ... }  // 1.5x stats
pub fn generate_dungeon_boss(zone_id: u32) -> Enemy { ... }   // 2.0x stats
```

The dungeon's `zone_id` should be stored on the `Dungeon` struct (already has level/prestige but not zone).

---

## 3. Prestige Combat Benefits

### 3.1 New: Prestige Damage Bonus

Add a flat damage bonus from prestige rank, applied in `update_combat()`:

```rust
/// Prestige provides +2% damage per rank, applied multiplicatively with Haven bonuses.
/// P1: +2%, P5: +10%, P10: +20%, P20: +40%
pub const PRESTIGE_DAMAGE_BONUS_PER_RANK: f64 = 2.0; // percent
```

**Integration point** (`src/combat/logic.rs:168`):

```rust
let prestige_damage_bonus = state.prestige_rank as f64 * PRESTIGE_DAMAGE_BONUS_PER_RANK;
let mut damage = (base_damage as f64
    * (1.0 + haven.damage_percent / 100.0)
    * (1.0 + prestige_damage_bonus / 100.0)) as u32;
```

### 3.2 New: Prestige Defense Bonus

Add a flat defense bonus from prestige rank, applied to `DerivedStats`:

```rust
/// Prestige provides +1 defense per rank.
/// P1: +1, P5: +5, P10: +10, P20: +20
pub const PRESTIGE_DEFENSE_PER_RANK: u32 = 1;
```

**Integration point** (`src/character/derived_stats.rs`): Add `prestige_rank` parameter to `calculate_derived_stats()`, or apply it in combat logic where defense is used.

The simplest integration is in `update_combat()` where enemy damage is calculated:

```rust
let prestige_defense = state.prestige_rank * PRESTIGE_DEFENSE_PER_RANK;
let enemy_damage = enemy.damage.saturating_sub(derived.defense + prestige_defense);
```

### 3.3 New: Prestige HP Bonus

Add a percentage HP bonus from prestige:

```rust
/// Prestige provides +3% max HP per rank.
/// P1: +3%, P5: +15%, P10: +30%, P20: +60%
pub const PRESTIGE_HP_BONUS_PER_RANK: f64 = 3.0; // percent
```

**Integration point** (`src/character/derived_stats.rs`): Apply after base HP calculation. This requires passing `prestige_rank` into `calculate_derived_stats()` or applying it at the callsite in `game_logic.rs` / `tick.rs` where max_hp is synced.

The recommended approach is to add `prestige_rank` to `calculate_derived_stats()`:

```rust
pub fn calculate_derived_stats(
    attrs: &Attributes,
    equipment: &Equipment,
    prestige_rank: u32,
) -> Self {
    // ... existing calculation ...
    let prestige_hp_bonus = prestige_rank as f64 * PRESTIGE_HP_BONUS_PER_RANK / 100.0;
    max_hp = ((max_hp as f64) * (1.0 + prestige_hp_bonus)) as u32;
    // ...
}
```

### 3.4 Summary of Prestige Combat Benefits

| Prestige | Damage Bonus | Defense Bonus | HP Bonus | XP Mult (existing) | Attr Cap (existing) |
|----------|-------------|---------------|----------|---------------------|---------------------|
| P0 | +0% | +0 | +0% | 1.0x | 20 |
| P1 | +2% | +1 | +3% | 1.5x | 25 |
| P5 | +10% | +5 | +15% | ~2.5x | 45 |
| P10 | +20% | +10 | +30% | ~3.5x | 70 |
| P15 | +30% | +15 | +45% | ~4.3x | 95 |
| P20 | +40% | +20 | +60% | ~5.1x | 120 |

---

## 4. Fight Duration Target: 5-10 Seconds

### 4.1 Mathematical Model

With zone-based static stats, fight duration becomes predictable:

**Zone 1 fresh character (P0, level 1, base attributes):**
- Player total damage: 10 (5 phys + 5 magic), defense: 0
- Player max HP: 50, attack interval: 1.5s
- Enemy HP: 30, damage: 4, attack interval: 2.0s

Player hits to kill: `ceil(30 / max(1, 10 - 0)) = 3 hits = 4.5s`
Enemy hits to kill player: `ceil(50 / max(1, 4 - 0)) = 13 hits = 26s`
**Fight duration: ~4.5s** (player wins comfortably)

**Zone 1 subzone boss (P0):**
- Boss HP: 30 * 1.5 = 45, damage: 4 * 1.2 = ~5
- Player hits: ceil(45/10) = 5 hits = 7.5s
- Boss hits to kill: ceil(50/5) = 10 hits = 20s
- **Fight duration: ~7.5s** (player wins with ~60% HP remaining)

**Zone 1 zone boss (P0, player at ~level 8-10):**
- Assume player has ~14 STR, 12 CON by level 10 (+2 STR mod, +1 CON mod)
- Player damage: ~14, HP: ~60
- Boss HP: 30 * 2.0 = 60, damage: 4 * 1.5 = 6
- Player hits: ceil(60/14) = ~5 hits = 7.5s
- Boss hits to kill: ceil(60/6) = 10 hits = 20s
- **Fight duration: ~7.5s** (player wins, tight but doable)

This puts all fights in the 4-10 second range for appropriately-leveled content, which matches the design target.

### 4.2 Variance Band

The 0.9-1.1 variance on enemy stats (reduced from 0.8-1.2) keeps fights feeling different without making any single fight drastically harder. This narrows the band from a 50% swing to a 22% swing.

---

## 5. Boss Kill Counter Reset on Death

### 5.1 Recommendation: Preserve Kill Counter on Boss Death

Change `src/combat/logic.rs:381-384` to:

```rust
if state.zone_progression.fighting_boss {
    state.zone_progression.fighting_boss = false;
    // DO NOT reset kills_in_subzone
    // Player immediately faces the boss again on next spawn cycle
}
```

**Rationale:**
- With static scaling, bosses are now beatable with appropriate gear/level
- Resetting the counter punishes the player for attempting challenging content
- The boss respawns immediately, letting players retry quickly
- If they want to grind more first, they can travel back to an easier zone
- The regen timer (2.5s) already provides a brief pause between attempts

### 5.2 Alternative: Partial Counter Preservation

If full preservation feels too easy, keep half the kills:

```rust
state.zone_progression.kills_in_subzone = state.zone_progression.kills_in_subzone / 2;
```

This requires 5 more kills instead of 10, splitting the difference.

**Recommendation:** Full preservation (option 5.1). The idle RPG genre favors reducing friction.

---

## 6. Backward Compatibility with Existing Saves

### 6.1 Enemy Data (No Migration Needed)

Enemy structs are transient — they are regenerated on each spawn. Existing saves will simply generate new enemies with the new static stats on load. The `Enemy` struct itself (`name, max_hp, current_hp, damage`) is unchanged.

If a player loads a save with an active enemy, that enemy retains its old stats until it dies or the player dies. This is acceptable — it affects at most one fight.

### 6.2 DerivedStats Signature Change

Adding `prestige_rank` to `calculate_derived_stats()` changes its signature. All callsites must be updated:

| File | Usage |
|------|-------|
| `src/combat/logic.rs` (2 places) | Regen and combat damage calculation |
| `src/core/game_logic.rs` (3 places) | Enemy spawning, level-up HP sync |
| `src/core/tick.rs` (1 place) | Stage 3: sync player HP |
| `src/items/scoring.rs` (1 place) | Auto-equip scoring |

All these callsites have access to `GameState` which contains `prestige_rank`.

### 6.3 Dungeon Zone Tracking

Adding `zone_id` to the `Dungeon` struct requires a `#[serde(default)]` annotation for backward compatibility:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dungeon {
    // ... existing fields ...
    /// Zone where dungeon was discovered (for enemy scaling)
    #[serde(default = "default_dungeon_zone")]
    pub zone_id: u32,
}

fn default_dungeon_zone() -> u32 { 1 }
```

Old saves with active dungeons will default to Zone 1 stats. This is conservative but safe.

### 6.4 New Constants

All new constants (`ZONE_BASE_STATS`, `PRESTIGE_DAMAGE_BONUS_PER_RANK`, etc.) are additive to `src/core/constants.rs` — no existing constants are removed or changed.

### 6.5 Combat State

`CombatState` is unchanged. The `player_max_hp` field will now reflect prestige-boosted HP values, but the struct itself doesn't need migration.

---

## 7. Integration Points: Files That Must Change

### Primary Changes (Combat Balance Core)

| File | Change | Description |
|------|--------|-------------|
| `src/core/constants.rs` | Add | Zone base stat table, prestige combat bonus constants, new boss multipliers |
| `src/combat/types.rs` | Modify | Replace `generate_enemy_with_multiplier(player_max_hp, ...)` with `generate_zone_enemy_static(zone, subzone)`. Update `generate_subzone_boss` and `generate_boss_for_current_zone`. Remove `player_max_hp` from all zone enemy generators. Keep `player_max_hp` versions for dungeon (with zone_id). |
| `src/combat/logic.rs` | Modify | Apply prestige damage bonus and prestige defense in `update_combat()`. Change boss death to not reset kill counter. |
| `src/character/derived_stats.rs` | Modify | Add `prestige_rank` parameter. Apply prestige HP bonus. |

### Secondary Changes (Callsite Updates)

| File | Change | Description |
|------|--------|-------------|
| `src/core/game_logic.rs` | Modify | Update `spawn_enemy_if_needed()` to use zone-based generators without `player_max_hp`. Update `calculate_derived_stats` calls with `prestige_rank`. Store `zone_id` on dungeon discovery. |
| `src/core/tick.rs` | Modify | Update `calculate_derived_stats` call with `prestige_rank`. |
| `src/items/scoring.rs` | Modify | Update `calculate_derived_stats` call with `prestige_rank`. |
| `src/dungeon/types.rs` | Modify | Add `zone_id: u32` field to `Dungeon` struct. |
| `src/dungeon/generation.rs` | Modify | Accept and store `zone_id` in dungeon generation. |

### Test Updates

| File | Change | Description |
|------|--------|-------------|
| `src/combat/types.rs` (tests) | Modify | Update enemy generation tests for new API. |
| `src/combat/logic.rs` (tests) | Modify | Update combat tests for prestige bonuses and kill counter behavior. |
| `src/character/derived_stats.rs` (tests) | Modify | Update to pass `prestige_rank`. |
| `tests/` (integration) | Modify | Update any integration tests using enemy generation or derived stats. |

### No Changes Needed

| File | Reason |
|------|--------|
| `src/zones/data.rs` | Zone definitions unchanged |
| `src/zones/progression.rs` | Progression logic unchanged (except kill counter change is in combat/logic.rs) |
| `src/haven/types.rs` | Haven bonuses unchanged, still injected as parameters |
| `src/character/prestige.rs` | Prestige system unchanged; new bonuses are calculated elsewhere |
| `src/items/drops.rs` | Drop system unaffected by combat balance |
| `src/fishing/` | Unrelated system |
| `src/ui/` | UI reads state, doesn't generate enemies |

---

## 8. Migration Path (Implementation Order)

1. **Add constants** (`constants.rs`) — Zone base stats, prestige combat bonuses, new boss multipliers
2. **Modify enemy generation** (`combat/types.rs`) — New static-scaling functions alongside old ones
3. **Update DerivedStats** (`derived_stats.rs`) — Add prestige_rank parameter, apply HP bonus
4. **Update combat logic** (`combat/logic.rs`) — Prestige damage/defense bonuses, kill counter change
5. **Update spawn logic** (`game_logic.rs`) — Use new enemy generators, pass zone_id to dungeons
6. **Update callsites** (`tick.rs`, `scoring.rs`) — Pass prestige_rank to DerivedStats
7. **Add dungeon zone tracking** (`dungeon/types.rs`, `dungeon/generation.rs`) — Store zone_id
8. **Update tests** — All affected test files
9. **Remove old functions** — Delete `generate_enemy_with_multiplier` and `player_max_hp`-based generators once nothing calls them

---

## 9. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Zone base stats need tuning | High | Medium | Values in constant table are easy to adjust. Run headless simulator (`src/simulator/`) to validate. |
| DerivedStats signature change breaks many callsites | Certain | Low | Mechanical refactor — compiler will find all callsites. |
| Old saves with active enemies behave differently | Low | Low | One fight at most; enemy is replaced on next spawn. |
| Prestige bonuses too strong/weak | Medium | Medium | Constants are easily tunable. Start conservative. |
| Dungeon enemy difficulty mismatch | Medium | Medium | Dungeon zone_id fallback to 1 is conservative. |

---

## 10. Validation Criteria

The combat balance changes are successful when:

1. **P0 Zone 1:** Fresh character can clear Zone 1 (all 3 subzones + bosses) within 30-60 minutes
2. **P0 Zone 2:** Player at level ~10-15 can clear Zone 2 within 1-2 hours
3. **Zone Boss Winrate:** Boss success rate > 50% for appropriately-leveled players
4. **Fight Duration:** Normal mobs: 3-6s, subzone bosses: 5-8s, zone bosses: 7-12s
5. **Prestige Impact:** P1 character at level 1 is measurably stronger than P0 character at level 1 in combat (not just XP speed)
6. **Zone Difficulty Curve:** Each zone is harder than the previous, requiring the player to level up within the zone's level range
7. **No Regression:** Existing game features (dungeons, fishing, haven, achievements) continue working correctly

Use the headless simulator (`src/simulator/`) to run automated progression tests across prestige levels 0-20.

## combat-balance-core.md

# Combat Balance Core: Enemy Stats, Boss Multipliers, and Fight Duration

## Status: PROPOSED
## Author: Game Designer #1
## Related: Issue #123 (Combat Balance Overhaul)

---

## Problem Summary

The current combat system has a fundamental design flaw: **enemy stats are derived from the player's stats**, making leveling meaningless for combat power. Specifically:

```
Current formulas (src/combat/types.rs):
  Enemy HP     = player_max_hp * variance(0.8-1.2) * zone_mult * boss_mult
  Enemy Damage = (player_max_hp / 7) * variance(0.8-1.2) * zone_mult * boss_mult
```

This causes:
1. **No power growth**: A level 1 player and a level 100 player face proportionally identical challenges
2. **Boss death spiral**: Zone/boss multipliers stack on top of already player-scaled values, creating unkillable bosses (~0.1% success rate)
3. **Uniform fight duration**: Every fight feels the same regardless of zone or level
4. **65% of P0 players stuck in Zone 1**: Leveling provides no combat advantage, only boss multipliers make bosses harder

---

## Design Philosophy

### Core Principle: Zone-Based Static Scaling

Enemies should have **fixed stats determined by their zone and subzone**, independent of the player. This creates a natural difficulty curve where:

- **Under-leveled**: Content is genuinely hard (longer fights, risk of death)
- **At-level**: Content is appropriately challenging (5-10s fights, bosses require some luck)
- **Over-leveled**: Content becomes easy (quick kills, trivial bosses)

This is the standard idle RPG approach and is critical for the prestige loop to feel rewarding.

### Design Targets

| Encounter Type | Fight Duration (at-level) | Player Win Rate (at-level) |
|---|---|---|
| Normal mob | 5-8 seconds (3-5 exchanges) | ~95% |
| Subzone boss | 10-15 seconds (6-10 exchanges) | ~60-70% |
| Zone boss | 15-25 seconds (10-16 exchanges) | ~30-40% |
| Dungeon elite | 8-12 seconds | ~80% |
| Dungeon boss | 12-20 seconds | ~50-60% |

"At-level" means the player's level is at the zone's `min_level` with average attribute distribution and no equipment bonuses.

---

## Player Power Reference

Before defining enemy stats, we need to model what players look like at each zone boundary.

### Attribute Growth Model (P0, no equipment)

At P0, attribute cap = 20. Level-ups give +3 random points across 6 attributes.

| Level | Avg Attribute | CON Modifier | STR Modifier | Max HP | Phys Dmg | Total Dmg | Defense |
|---|---|---|---|---|---|---|---|
| 1 | 10 | 0 | 0 | 50 | 5 | 10 | 0 |
| 5 | 12 | +1 | +1 | 60 | 7 | 14 | 1 |
| 10 | 14 | +2 | +2 | 70 | 9 | 18 | 2 |
| 15 | 16 | +3 | +3 | 80 | 11 | 22 | 3 |
| 20 | 18 | +4 | +4 | 90 | 13 | 26 | 4 |
| 25 | 20 (cap) | +5 | +5 | 100 | 15 | 30 | 5 |

Note: At P0 cap (attr 20), all characters converge to the same stats regardless of level beyond 25. This is intentional -- it creates a "soft ceiling" that encourages prestige.

### Prestige Power Scaling

Each prestige rank adds +5 to attribute cap. By P5 (cap=45), a maxed character has:
- CON 45, mod +17: HP = 50 + 170 = 220
- STR 45, mod +17: Phys = 5 + 34 = 39
- Total damage: ~78 (phys + magic)
- DEX 45, mod +17: Defense = 17

This means prestige players are dramatically more powerful -- by design. They should trivialize earlier zones and only struggle with prestige-gated content.

### Reference Player Stats by Zone Entry Level

These are the expected player stats when first entering each zone (P0 for zones 1-2, appropriate prestige for later zones). Equipment bonuses add roughly +20-50% to base stats.

| Zone | Entry Level | Prestige | HP (base) | Total Dmg (base) | Defense (base) |
|---|---|---|---|---|---|
| 1 Meadow | 1 | P0 | 50 | 10 | 0 |
| 2 Dark Forest | 10 | P0 | 70 | 18 | 2 |
| 3 Mountain Pass | 25 | P5 | 130 | 38 | 8 |
| 4 Ancient Ruins | 40 | P5 | 160 | 48 | 12 |
| 5 Volcanic Wastes | 55 | P10 | 220 | 64 | 18 |
| 6 Frozen Tundra | 70 | P10 | 260 | 76 | 22 |
| 7 Crystal Caverns | 85 | P15 | 320 | 92 | 28 |
| 8 Sunken Kingdom | 100 | P15 | 370 | 106 | 32 |
| 9 Floating Isles | 115 | P20 | 440 | 124 | 38 |
| 10 Storm Citadel | 130 | P20 | 500 | 140 | 44 |
| 11 The Expanse | 150 | P20+ | 560+ | 156+ | 50+ |

---

## New Enemy Stat Formulas

### Core Formula: Zone-Based Static Stats

Replace the current `generate_enemy_with_multiplier(player_max_hp, ...)` with a formula based purely on zone and subzone:

```
Enemy Base HP     = ZONE_BASE_HP[zone_id] + (subzone_depth - 1) * ZONE_HP_STEP[zone_id]
Enemy Base Damage = ZONE_BASE_DMG[zone_id] + (subzone_depth - 1) * ZONE_DMG_STEP[zone_id]
Enemy Base Defense = ZONE_BASE_DEF[zone_id] + (subzone_depth - 1) * ZONE_DEF_STEP[zone_id]

Actual HP     = Enemy Base HP * variance(0.9, 1.1)
Actual Damage = Enemy Base Damage * variance(0.9, 1.1)
Actual Defense = Enemy Base Defense (no variance, keep it predictable)
```

Variance is reduced from 0.8-1.2 to 0.9-1.1 to make combat more predictable and balance-testable.

### Zone Enemy Stat Table

These values are tuned so that an at-level player with no equipment kills a normal mob in 4-6 player attacks (6-9 seconds at 1.5s interval) while surviving 4-7 enemy attacks (8-14 seconds at 2.0s interval).

**Design constraint**: Player should always be able to survive long enough to kill the enemy at-level. Enemy effective DPS < Player effective DPS for normal mobs.

| Zone | Base HP | HP Step | Base Dmg | Dmg Step | Base Def | Def Step |
|---|---|---|---|---|---|---|
| 1 Meadow | 30 | 5 | 5 | 1 | 0 | 0 |
| 2 Dark Forest | 50 | 8 | 9 | 2 | 1 | 1 |
| 3 Mountain Pass | 90 | 12 | 16 | 3 | 4 | 1 |
| 4 Ancient Ruins | 120 | 15 | 22 | 4 | 7 | 2 |
| 5 Volcanic Wastes | 170 | 18 | 30 | 5 | 11 | 2 |
| 6 Frozen Tundra | 210 | 22 | 38 | 6 | 15 | 3 |
| 7 Crystal Caverns | 270 | 25 | 48 | 7 | 20 | 3 |
| 8 Sunken Kingdom | 320 | 30 | 56 | 8 | 24 | 4 |
| 9 Floating Isles | 380 | 35 | 66 | 9 | 30 | 4 |
| 10 Storm Citadel | 450 | 40 | 78 | 10 | 36 | 5 |
| 11 The Expanse | 520 | 45 | 88 | 12 | 42 | 5 |

### Worked Example: Zone 1, Subzone 1 (Sunny Fields)

At-level player (Level 1, P0, no equipment):
- Player: HP=50, Dmg=10, Def=0, Atk interval=1.5s
- Enemy: HP=30(+/-10%), Dmg=5(+/-10%), Def=0, Atk interval=2.0s

Player DPS: 10 / 1.5 = 6.67/s. Time to kill enemy: 30 / 6.67 = **4.5 seconds** (3 attacks)
Enemy DPS: 5 / 2.0 = 2.5/s. Damage taken in 4.5s: ~11 HP. Player survives easily with 39 HP remaining.

### Worked Example: Zone 1, Subzone 3 (Mushroom Caves) - Normal Mob

At-level player (Level ~7, P0):
- Player: HP=60, Dmg=14, Def=1, Atk interval=1.5s
- Enemy: HP=40(+/-10%), Dmg=7(+/-10%), Def=0

Player effective DPS: 14 / 1.5 = 9.3/s. Time to kill: 40 / 9.3 = **4.3 seconds** (3 attacks)
Enemy effective DPS: (7-1) / 2.0 = 3.0/s. Damage taken: ~13 HP. Player at 47 HP. Comfortable.

### Worked Example: Zone 2, Subzone 1 (Forest Edge) - Normal Mob

At-level player (Level 10, P0):
- Player: HP=70, Dmg=18, Def=2
- Enemy: HP=50, Dmg=9, Def=1

Player effective DPS: (18-1) / 1.5 = 11.3/s. Time to kill: 50 / 11.3 = **4.4s** (3 attacks)
Enemy effective DPS: (9-2) / 2.0 = 3.5/s. Damage taken: ~15 HP. Player at 55 HP. Good.

### Worked Example: Over-leveled (Level 20 in Zone 1)

Player (Level 20, P0):
- Player: HP=90, Dmg=26, Def=4
- Zone 1 Subzone 1 Enemy: HP=30, Dmg=5, Def=0

Player effective DPS: 26 / 1.5 = 17.3/s. Time to kill: 30 / 17.3 = **1.7s** (2 attacks)
Enemy effective DPS: (5-4) / 2.0 = 0.5/s. Damage taken: ~1 HP. Trivial, as intended.

---

## Boss Multipliers

### Subzone Bosses

Subzone bosses guard the transition between subzones. They should be a meaningful check that requires the player to be close to the zone's level range, but not a hard wall.

```
Subzone Boss HP     = Normal Enemy HP * 2.5
Subzone Boss Damage = Normal Enemy Damage * 1.3
Subzone Boss Defense = Normal Enemy Defense * 1.5
Subzone Boss Attack Interval = 1.8s (unchanged from current)
```

### Zone Bosses

Zone bosses guard the transition to the next zone. They should be a hard check that requires the player to be at or slightly above the zone's max level.

```
Zone Boss HP     = Normal Enemy HP * 4.0
Zone Boss Damage = Normal Enemy Damage * 1.6
Zone Boss Defense = Normal Enemy Defense * 2.0
Zone Boss Attack Interval = 1.5s (unchanged from current)
```

### Worked Example: Zone 1 Subzone Boss (Field Guardian)

At-level player (Level ~5, P0):
- Player: HP=60, Dmg=14, Def=1, Atk=1.5s
- Subzone 1 normal mob: HP=30, Dmg=5, Def=0
- Field Guardian: HP=75, Dmg=6, Def=0, Atk=1.8s

Player DPS: 14 / 1.5 = 9.3/s. Time to kill boss: 75 / 9.3 = **8.1 seconds** (6 attacks)
Boss DPS: (6-1) / 1.8 = 2.8/s. Damage taken in 8.1s: ~23 HP. Player at 37 HP. **Win rate: ~75-85%** (variance may kill lower rolls).

### Worked Example: Zone 1 Zone Boss (Sporeling Queen)

At-level player (Level ~10, P0):
- Player: HP=70, Dmg=18, Def=2, Atk=1.5s
- Subzone 3 normal mob: HP=40, Dmg=7, Def=0
- Sporeling Queen: HP=160, Dmg=11, Def=0, Atk=1.5s

Player DPS: 18 / 1.5 = 12.0/s. Time to kill: 160 / 12.0 = **13.3 seconds** (9 attacks)
Boss DPS: (11-2) / 1.5 = 6.0/s. Damage taken in 13.3s: ~80 HP. Player HP=70. **Player dies by default.**

But consider: at level 10, the player should have some equipment from Zone 1 drops. With +20% stats from gear:
- Player: HP=84, Dmg=22, Def=3
- Player DPS: 22 / 1.5 = 14.7/s. Time to kill: 160 / 14.7 = 10.9s (8 attacks)
- Boss effective DPS: (11-3) / 1.5 = 5.3/s. Damage taken: 58 HP. Player at 26 HP. **Win rate: ~35-45%.** Crit luck matters. This is correct for a zone boss.

The player can also over-level (level 12-15) to improve odds significantly. This is the intended path for most players.

### Worked Example: Zone 5 Subzone Boss (Ash Walker Chief)

At-level player (Level 55, P10):
- Player: HP=220, Dmg=64, Def=18
- Subzone 1 normal: HP=170, Dmg=30, Def=11
- Boss: HP=425, Dmg=39, Def=16, Atk=1.8s

Player DPS: (64-16) / 1.5 = 32/s. Time to kill: 425 / 32 = **13.3s** (9 attacks)
Boss effective DPS: (39-18) / 1.8 = 11.7/s. Damage taken: 156 HP. Player at 64 HP. **Win rate: ~60-65%.** Good.

### Boss Multiplier Summary

| Boss Type | HP Mult | Dmg Mult | Def Mult | Atk Interval |
|---|---|---|---|---|
| Subzone Boss | 2.5x | 1.3x | 1.5x | 1.8s |
| Zone Boss | 4.0x | 1.6x | 2.0x | 1.5s |
| Dungeon Elite | 1.5x | 1.2x | 1.3x | 1.6s |
| Dungeon Boss | 2.5x | 1.4x | 1.5x | 1.4s |

Note: Dungeon enemies use the same zone-based static stats as their base, with dungeon-specific multipliers on top.

---

## Defense in the Damage Formula

### Current Formula

```rust
let enemy_damage = enemy.damage.saturating_sub(derived.defense);
```

This is a flat subtraction model. This is fine for the game's design but needs a minimum damage floor to prevent defense from completely negating damage:

```
Effective Damage = max(1, attacker_damage - defender_defense)
```

The current code already has `saturating_sub` which goes to 0, but the minimum should be 1 to prevent complete damage immunity. This needs a code change.

For player attacks vs enemy defense:
```
Player Effective Damage = max(1, player_total_damage - enemy_defense)
```

This matters in later zones where enemies have meaningful defense values.

---

## Fight Duration Analysis by Zone

Using the stat tables above, here are expected fight durations for an at-level player with no equipment:

| Zone | Normal Mob (s) | Subzone Boss (s) | Zone Boss (s) |
|---|---|---|---|
| 1 Meadow | 4-6 | 8-11 | 13-18 |
| 2 Dark Forest | 4-6 | 8-12 | 14-19 |
| 3 Mountain Pass | 5-7 | 9-13 | 15-20 |
| 4 Ancient Ruins | 5-7 | 10-14 | 16-22 |
| 5 Volcanic Wastes | 5-8 | 10-14 | 16-22 |
| 6 Frozen Tundra | 5-8 | 10-15 | 17-23 |
| 7 Crystal Caverns | 5-8 | 11-15 | 18-24 |
| 8 Sunken Kingdom | 6-8 | 11-15 | 18-24 |
| 9 Floating Isles | 6-9 | 12-16 | 19-25 |
| 10 Storm Citadel | 6-9 | 12-16 | 19-25 |

When over-leveled by 5+ levels, normal mob duration drops to 2-3 seconds. When over-leveled by 10+, it drops to 1-2 seconds. This is the core reward loop: grinding levels makes content faster.

---

## Death Penalty

### Current Behavior
- Death to boss: resets `fighting_boss=false` and `kills_in_subzone=0`
- Must kill 10 more mobs to trigger boss again

### Recommendation: Keep with Modification

The kill counter reset is good -- it creates a meaningful cost for death without being punishing. However, 10 kills when each fight takes 5-8s (plus 2.5s regen) means 75-105 seconds to get back to the boss. This is appropriate.

**Change**: After dying to a boss, reduce the required kills to trigger the boss again from 10 to 5 for that specific attempt. This prevents the frustration of a long grind-back after a close loss while still maintaining some cost. Store this as a `boss_retry_kills` field on `ZoneProgression` that resets to the full 10 after defeating the boss or changing zones.

This is a softer penalty that keeps players engaged rather than frustrated, especially important for zone bosses where the win rate is 30-40%.

---

## Over-leveling and Power Curve

### How Leveling Creates Power

Each level gives +3 random attribute points. For combat:
- STR/INT increase damage (+2 per modifier point, i.e., per 2 attribute points)
- CON increases HP (+10 per modifier point)
- DEX increases defense (+1 per modifier) and crit chance (+1% per modifier)

Average per level: +0.5 to each attribute = +0.25 modifier = +0.5 damage, +2.5 HP, +0.25 defense

This means being 5 levels above content gives roughly:
- +2.5 damage, +12.5 HP, +1.25 defense

Against Zone 1 enemies (HP=30, Dmg=5, Def=0), these are significant improvements.

### Equipment Amplification

Equipment drops from killed enemies. Higher-zone equipment has higher ilvl and better stats. A player farming Zone 1 will accumulate ilvl 10 gear that provides an additional ~15-30% boost to base stats. This compounds with level advantages.

### Prestige Power Spike

Prestiging raises the attribute cap by 5, allowing each attribute to grow further. A P1 character maxing out attributes reaches much higher stats than a P0 character. This creates a dramatic power spike for prestige-gated zones.

---

## Implementation Checklist for Task #4

1. **Add zone stat lookup tables** to `src/core/constants.rs` (or a new `src/combat/balance.rs`)
2. **Replace `generate_enemy_with_multiplier`** to use zone-based static stats instead of `player_max_hp`
3. **Update `generate_zone_enemy`** to use new static tables
4. **Update `generate_subzone_boss`** to use new boss multipliers
5. **Add enemy defense field** -- the `Enemy` struct currently has no `defense` field; add it and use in damage calculations
6. **Add minimum damage floor** of 1 to both player and enemy damage calculations
7. **Update dungeon enemy generation** to use zone-based stats (dungeon in zone N uses zone N stats)
8. **Add boss retry mechanic** (5 kills instead of 10 after boss death) -- optional, lower priority
9. **Update all tests** that depend on current enemy stat formulas

### Constants to Add

```rust
// Zone enemy base stats: (base_hp, hp_step, base_dmg, dmg_step, base_def, def_step)
pub const ZONE_ENEMY_STATS: [(u32, u32, u32, u32, u32, u32); 11] = [
    (30, 5, 5, 1, 0, 0),     // Zone 1: Meadow
    (50, 8, 9, 2, 1, 1),     // Zone 2: Dark Forest
    (90, 12, 16, 3, 4, 1),   // Zone 3: Mountain Pass
    (120, 15, 22, 4, 7, 2),  // Zone 4: Ancient Ruins
    (170, 18, 30, 5, 11, 2), // Zone 5: Volcanic Wastes
    (210, 22, 38, 6, 15, 3), // Zone 6: Frozen Tundra
    (270, 25, 48, 7, 20, 3), // Zone 7: Crystal Caverns
    (320, 30, 56, 8, 24, 4), // Zone 8: Sunken Kingdom
    (380, 35, 66, 9, 30, 4), // Zone 9: Floating Isles
    (450, 40, 78, 10, 36, 5), // Zone 10: Storm Citadel
    (520, 45, 88, 12, 42, 5), // Zone 11: The Expanse
];

// Boss multipliers: (hp_mult, dmg_mult, def_mult)
pub const SUBZONE_BOSS_MULTIPLIERS: (f64, f64, f64) = (2.5, 1.3, 1.5);
pub const ZONE_BOSS_MULTIPLIERS: (f64, f64, f64) = (4.0, 1.6, 2.0);
pub const DUNGEON_ELITE_MULTIPLIERS: (f64, f64, f64) = (1.5, 1.2, 1.3);
pub const DUNGEON_BOSS_MULTIPLIERS: (f64, f64, f64) = (2.5, 1.4, 1.5);
```

---

## Compatibility Notes

### Save File Compatibility

The `Enemy` struct is serialized in `CombatState`. Adding a `defense` field requires `#[serde(default)]` to maintain backward compatibility with old saves. An enemy loaded from an old save will have defense=0, which is safe.

### Dungeon Enemies

Dungeon enemies currently use generic `generate_enemy(player_max_hp)`, `generate_elite_enemy(player_max_hp)`, and `generate_boss_enemy(player_max_hp)`. These should be updated to use zone-based stats from the zone the dungeon was discovered in. The dungeon should store the `zone_id` it was generated in (it may already have this info via dungeon level).

### Zone 11 (The Expanse)

The Expanse cycles infinitely. Its stats are intentionally higher than Zone 10 to serve as an endless challenge. Since it cycles, players will naturally over-level its content through repeated clears and prestige.

---

## Validation Criteria

The implementation should be validated against these scenarios:

1. **Fresh P0 player in Zone 1**: Can kill normal mobs in 4-6s. Reaches boss after ~2 minutes of farming. Has ~35-45% boss win rate by level 10 with gear.
2. **P0 player grinding Zone 1 at level 15**: Normal mobs die in 2-3s. Zone boss beatable with ~70%+ win rate.
3. **P5 player entering Zone 3**: Normal mobs take 5-7s. Subzone bosses are challenging but beatable at 60-70%.
4. **Over-leveled player in earlier zone**: Previous zone content is trivially easy (1-2s kills).
5. **Boss death recovery**: Takes 40-60 seconds (5 kills at 5-8s + regen) to retry boss.

## combat-balance-prestige.md

# Prestige Combat Benefits Design

Issue: #123 — Prestige provides no direct combat benefit; 65% of P0 players stuck in Zone 1.

## 1. Analysis of Current Prestige Benefits Gap

### What Prestige Currently Provides

| Benefit | Formula | Combat Impact |
|---------|---------|---------------|
| XP multiplier | `1.0 + 0.5 * rank^0.7` | Indirect — faster leveling, not combat power |
| Attribute cap | `20 + 5 * rank` | **Negated** — enemies scale with `player_max_hp` |
| Item drop rate | `+1% per rank (cap 25%)` | Indirect — more gear, not combat power |
| Mob rarity bonus | `+0.5% per rank (cap 10%)` | Indirect — better gear quality |

### The Core Problem: Enemy HP Scaling Cancels Attribute Caps

Enemy generation in `combat/types.rs:63-77` uses `player_max_hp` as the base:

```
Enemy HP   = player_max_hp * random(0.8..1.2) * zone_multiplier
Enemy DMG  = player_max_hp / 7.0 * random(0.8..1.2) * zone_multiplier
```

When a P5 player has CON 35 (cap 45), their max HP is 175. But enemies are generated at 80-120% of 175, keeping fights at the same relative difficulty as a P0 player with CON 20 and max HP 100. **Higher attribute caps give bigger numbers but identical fight outcomes.**

This means prestige rank 1-9 provides:
- Faster XP gain (useful)
- Higher numbers that are cosmetically larger (not useful)
- No ability to kill enemies faster or survive longer in relative terms

### Haven Cannot Fill the Gap

Haven provides real combat bonuses (Armory: +25% damage at T3, Watchtower: +20% crit, War Room: +35% double strike). However:
- Haven requires P10+ just to discover
- Building Haven rooms costs prestige ranks (competing with the ranks needed to advance)
- P0-P9 players have zero access to these combat bonuses
- This creates a dead zone where prestige 1-9 feels unrewarding in combat

### Player Impact

- **P0 players (65% stuck in Zone 1)**: No combat tools beyond raw attribute points, which enemies match
- **P1-P4 players**: XP multiplier helps them level faster, but fights against Zone 2 bosses are the same difficulty
- **P5-P9 players**: Unlocked Zones 3-4 by rank requirement, but combat scaling still neutralizes their stats
- **P10+ players**: First real combat boost when Haven is discovered, but it takes significant prestige rank investment

## 2. Proposed Prestige Combat Bonuses

### Design Principles

1. **Prestige should break the HP-scaling treadmill**: Bonuses that are not derived from attributes and thus not reflected in `player_max_hp`
2. **Additive with Haven, not multiplicative**: Prestige bonuses and Haven bonuses stack additively to prevent runaway scaling
3. **Diminishing returns**: Same formula shape as XP multiplier to prevent late-game trivialization
4. **Immediately tangible**: Even P1 should feel a combat difference

### Proposed Bonuses Per Prestige Rank

We introduce **four** prestige combat bonuses, stored in a new `PrestigeCombatBonuses` struct. These are computed from prestige rank alone, not from attributes, so they bypass enemy HP scaling.

#### A. Prestige Damage Bonus (flat)

- **Formula**: `floor(2.0 * rank^0.6)` flat damage added after all multipliers
- **Rationale**: Flat damage is not part of `DerivedStats` and not reflected in `player_max_hp`, so enemies cannot scale against it. It represents "veteran battle instinct."
- **Values**:

| Rank | Flat Damage | Notes |
|------|-------------|-------|
| P0 | +0 | Baseline |
| P1 | +2 | Noticeable on 10-damage base hits |
| P2 | +3 | |
| P3 | +3 | |
| P5 | +4 | |
| P10 | +7 | Significant vs. Zone 5-6 enemies |
| P15 | +9 | |
| P20 | +11 | Meaningful but not dominant |

This bonus is applied as a final additive step in damage calculation, after Haven's `damage_percent` multiplier.

#### B. Prestige Defense Bonus (flat)

- **Formula**: `floor(1.0 * rank^0.55)` flat defense added after derived defense
- **Rationale**: Reduces incoming damage by a fixed amount that is independent of `player_max_hp`. Represents "hardened veteran resilience."
- **Values**:

| Rank | Flat Defense | Notes |
|------|-------------|-------|
| P0 | +0 | Baseline |
| P1 | +1 | Reduces mob damage by 1 |
| P2 | +1 | |
| P3 | +1 | |
| P5 | +2 | |
| P10 | +3 | Blocks ~20% of early zone mob damage |
| P15 | +4 | |
| P20 | +5 | |

Applied after DEX-based defense in `update_combat()`, subtracted from enemy damage alongside existing defense.

#### C. Prestige Crit Chance Bonus (percentage)

- **Formula**: `min(rank * 0.5, 10.0)` percentage points of crit chance
- **Rationale**: Percentage-based but capped at 10% to not overshadow Haven Watchtower (+20% at T3). Represents "experienced combatant precision."
- **Values**:

| Rank | Crit Bonus | Total with Haven T3 |
|------|------------|---------------------|
| P0 | +0% | 0% bonus |
| P1 | +0.5% | +0.5% |
| P5 | +2.5% | +2.5% |
| P10 | +5% | +25% (with Watchtower T3) |
| P15 | +7.5% | +27.5% |
| P20 | +10% (cap) | +30% |

Stacks additively with base crit (from DEX), equipment affixes, and Haven Watchtower.

#### D. Prestige HP Bonus (flat)

- **Formula**: `floor(5.0 * rank^0.5)` flat HP added to `max_hp`
- **Rationale**: Extra HP that is added AFTER enemies are generated (applied during combat, not during `DerivedStats` calculation). This is a survivability cushion that enemies do not scale against.
- **Important implementation note**: This bonus must NOT be included in `DerivedStats.max_hp` because that is what enemy generation reads. Instead, it is added as a separate bonus in the combat system, similar to how Haven bonuses are injected.
- **Values**:

| Rank | Flat HP | Notes |
|------|---------|-------|
| P0 | +0 | Baseline |
| P1 | +5 | One extra hit vs. early enemies |
| P2 | +7 | |
| P5 | +11 | |
| P10 | +15 | ~15% extra for P0-cap characters |
| P15 | +19 | |
| P20 | +22 | Meaningful buffer, not dominant |

**Critical constraint**: This HP bonus must be applied in combat only, not in `DerivedStats::calculate_derived_stats()`, to prevent enemies from scaling against it. The UI should still display the combined HP total.

## 3. Interaction with Haven Bonuses

### Stacking Rules

All prestige combat bonuses stack **additively** with Haven bonuses — never multiplicatively. This prevents exponential power curves.

| Stat | Prestige Source | Haven Source | Stacking |
|------|----------------|-------------|----------|
| Damage | Flat bonus (post-multiplier) | Armory % multiplier | Prestige flat added after Haven % applied to base |
| Defense | Flat bonus | None (Haven has no defense) | No overlap |
| Crit Chance | Flat % | Watchtower flat % | Sum of both added to base crit |
| HP | Flat bonus (combat-only) | None (Haven has no HP bonus) | No overlap |
| Double Strike | None | War Room % | No overlap |
| HP Regen | None | Alchemy Lab %, Bedroom delay | No overlap |

### Damage Calculation Order (Updated)

```
1. Calculate base damage from DerivedStats (STR/INT + equipment)
2. Apply Haven Armory multiplier: damage *= (1 + armory_percent / 100)
3. Apply prestige flat damage: damage += prestige_damage_bonus
4. Roll crit: crit_chance = base_crit + haven_crit + prestige_crit
5. If crit: damage *= crit_multiplier
6. Apply to enemy
```

Note: Prestige flat damage is added BEFORE crit multiplier so crits amplify it. This is intentional — it makes crits feel more impactful for prestiged players.

### Defense Calculation Order (Updated)

```
1. Calculate base defense from DerivedStats (DEX + equipment)
2. Add prestige flat defense: total_defense = base_defense + prestige_defense
3. Enemy damage after defense: max(1, enemy.damage - total_defense)
```

### No Double-Dipping Guarantee

- Haven bonuses are passed via `HavenCombatBonuses` struct (percentage-based)
- Prestige bonuses are passed via a new `PrestigeCombatBonuses` struct (flat values)
- They operate on different stages of the damage pipeline
- A player cannot get the same bonus from both sources

## 4. Expected Combat Effectiveness by Prestige Tier

### Baseline Scenario: P0, Level 10, Base Attributes (all 10), No Equipment

- Player max HP: 50
- Player damage: 10 (5 phys + 5 magic)
- Player defense: 0
- Player crit: 5%
- Enemy HP: ~40-60 (80-120% of 50)
- Enemy damage: ~6-8 (50/7 * variance)
- **Fight duration**: ~6-8 attacks each, ~9-12 seconds
- **Win rate**: ~60-70% (player often dies to Zone 1 subzone 3 boss)

### P1 (Bronze): +2 DMG, +1 DEF, +0.5% Crit, +5 HP

- Player effective damage: 12 (+20% vs baseline, significant)
- Player effective HP: 55 (enemies still see 50, so generated at P0 levels)
- Player defense: 1 (blocks ~15% of mob damage)
- **Fight duration**: ~5-7 attacks, ~7-10 seconds
- **Win rate**: ~80-85% vs Zone 1 (comfortable progression)
- **Impact**: P1 players can reliably clear Zone 1 bosses

### P5 (Diamond): +4 DMG, +2 DEF, +2.5% Crit, +11 HP

At P5, attributes can reach cap 45, but enemies scale to match. The flat bonuses provide the actual edge:

- Player effective damage: base ~39 + 4 flat + Haven Armory if available = ~43
- Player effective defense: base ~17 + 2 flat = 19
- Additional 11 HP hidden from enemy generation
- **Fight efficiency**: ~15-20% faster kills vs. equal-level content
- **Impact**: Zones 1-4 are comfortable; Zone 5+ (P10-gated) is still challenging

### P10 (Celestial): +7 DMG, +3 DEF, +5% Crit, +15 HP + Haven Access

P10 is the breakpoint where Haven becomes available, layering percentage bonuses on top:

| Stat | Prestige Alone | + Haven T1 | + Haven T3 |
|------|---------------|------------|------------|
| Damage bonus | +7 flat | +7 flat, +5% base | +7 flat, +25% base |
| Crit chance | +5% | +10% | +25% |
| Double strike | 0% | +10% | +35% |
| Defense | +3 flat | +3 flat | +3 flat |
| HP bonus | +15 flat | +15 flat | +15 flat |

- **Without Haven**: 15-20% combat advantage from prestige alone
- **With Haven T1**: 25-30% combat advantage
- **With Haven T3**: 50-70% combat advantage (the intended "fully invested" power level)

### P15 (Transcendent): +9 DMG, +4 DEF, +7.5% Crit, +19 HP

- Prestige combat bonuses provide a steady ~20% advantage at any gear/attribute level
- Haven bonuses (if invested) add another 50-70%
- Zones 7-8 are the target content here
- Total combat effectiveness: ~80-100% above what attribute scaling alone provides

### P20 (Eternal): +11 DMG, +5 DEF, +10% Crit (cap), +22 HP

- Prestige bonuses are approaching their diminishing returns ceiling
- The flat bonuses become proportionally less impactful as base stats grow
- Haven bonuses are the dominant factor at this level
- Focus shifts to Zone 9-10 content and Stormbreaker path
- Total combat effectiveness: ~90-120% above base (mostly from Haven)

## 5. How This Changes the "Stuck in Zone 1" Problem

### Current State (P0, No Combat Benefits)

```
Zone 1 Subzone 1: Win ~80% of fights (manageable)
Zone 1 Subzone 2: Win ~65% of fights (struggling)
Zone 1 Subzone 3 Boss: Win ~40% of fights (frequently stuck)
-> 65% of P0 players fail to progress past Zone 1
```

### With Prestige Combat Benefits (P1)

```
Zone 1 Subzone 1: Win ~95% of fights (+2 flat DMG, +1 DEF, +5 HP)
Zone 1 Subzone 2: Win ~85% of fights
Zone 1 Subzone 3 Boss: Win ~65% of fights (still challenging, not trivial)
Zone 2 Subzone 1: Win ~75% of fights (healthy progression)
-> Estimated 80%+ of P1 players clear Zone 1
```

### Key Insight: P0 Remains the "Tutorial"

P0 is intentionally harder. The game communicates through difficulty that prestige is the intended progression mechanic. Players who are stuck should prestige (requires level 10, which is achievable even with losses). The first prestige gives:

1. +50% XP multiplier (reach level 10 much faster on the next cycle)
2. **+2 flat damage** (10-20% more effective immediately)
3. **+1 flat defense** (blocks ~15% of mob damage)
4. **+5 flat HP** (one extra hit of survivability enemies cannot see)
5. Attribute cap 25 (minor benefit but reaches higher stats faster)

This creates a clear "aha moment" — after first prestige, Zone 1 feels noticeably easier.

### Projected Progression After Fix

| Prestige | Expected Zone Progress | Bottleneck |
|----------|----------------------|------------|
| P0 | Zone 1 (tutorial grind) | Boss difficulty, intentional |
| P1-2 | Zone 1-2 comfortably | XP curve for Zone 2 bosses |
| P3-4 | Zone 1-2 cleared, waiting for P5 | Prestige gate to Zone 3 |
| P5-9 | Zones 3-4 | Haven discovery wait (P10) |
| P10-14 | Zones 5-6, Haven building | PR investment vs. zone gates |
| P15-19 | Zones 7-8, Haven near-max | Level requirements get steep |
| P20+ | Zones 9-10, Stormbreaker path | End-game content gating |

## 6. Prestige Tier Unlock Flow and Power Curve

### Power Curve Visualization

```
Combat
Power
  ^
  |                                              P20 + Haven T3
  |                                         ****
  |                                     ****
  |                                 ****     <- Haven bonuses dominate
  |                            ****
  |                       ****               P10 + Haven discovered
  |                  *****
  |             *****                        <- Prestige flat bonuses
  |        *****                               provide the edge
  |   *****
  |***                                       P1 breakpoint
  |*                                         P0 baseline
  +----------------------------------------> Prestige Rank
  0    5    10    15    20
```

The curve has three phases:
1. **P0-P1**: Steep jump — first prestige breaks the "stuck" state
2. **P1-P10**: Gradual improvement — each prestige feels incrementally better
3. **P10-P20**: Accelerating returns — Haven bonuses layer on top of prestige bonuses

### Implementation Struct

```rust
/// Combat bonuses from prestige rank (independent of attributes/Haven)
pub struct PrestigeCombatBonuses {
    pub flat_damage: u64,       // Added after Haven % multiplier, before crit
    pub flat_defense: u64,      // Added to DEX-based defense
    pub crit_chance: f64,       // Percentage points added to crit chance
    pub flat_hp: u64,           // Added to combat HP, NOT to DerivedStats.max_hp
}

impl PrestigeCombatBonuses {
    pub fn from_rank(rank: u32) -> Self {
        Self {
            flat_damage: (2.0 * (rank as f64).powf(0.6)).floor() as u64,
            flat_defense: (1.0 * (rank as f64).powf(0.55)).floor() as u64,
            crit_chance: (rank as f64 * 0.5).min(10.0),
            flat_hp: (5.0 * (rank as f64).powf(0.5)).floor() as u64,
        }
    }
}
```

### Integration Points

1. **`combat/logic.rs`**: `update_combat()` accepts `PrestigeCombatBonuses` alongside existing `HavenCombatBonuses`
2. **`character/prestige.rs`**: New `prestige_combat_bonuses(rank) -> PrestigeCombatBonuses` function
3. **`core/tick.rs`**: Compute prestige bonuses at start of tick, pass to combat
4. **`combat/types.rs`**: Enemy generation remains unchanged (uses `DerivedStats.max_hp` only)
5. **`ui/stats_panel.rs`**: Display prestige bonuses separately (e.g., "+7 DMG from prestige")
6. **`ui/prestige_confirm.rs`**: Show the combat bonuses gained from next prestige

### Constants to Add (`core/constants.rs`)

```rust
// Prestige combat bonus formulas
pub const PRESTIGE_FLAT_DAMAGE_FACTOR: f64 = 2.0;
pub const PRESTIGE_FLAT_DAMAGE_EXPONENT: f64 = 0.6;
pub const PRESTIGE_FLAT_DEFENSE_FACTOR: f64 = 1.0;
pub const PRESTIGE_FLAT_DEFENSE_EXPONENT: f64 = 0.55;
pub const PRESTIGE_CRIT_PER_RANK: f64 = 0.5;
pub const PRESTIGE_CRIT_CAP: f64 = 10.0;
pub const PRESTIGE_FLAT_HP_FACTOR: f64 = 5.0;
pub const PRESTIGE_FLAT_HP_EXPONENT: f64 = 0.5;
```

## Summary of Changes

| Change | Files Affected | Risk |
|--------|---------------|------|
| New `PrestigeCombatBonuses` struct | `character/prestige.rs` | Low — new code |
| New constants | `core/constants.rs` | Low — additive |
| Pass bonuses to `update_combat()` | `combat/logic.rs`, `core/tick.rs` | Medium — modifies combat pipeline |
| Apply flat damage in attack phase | `combat/logic.rs` | Medium — changes damage numbers |
| Apply flat defense in defense phase | `combat/logic.rs` | Medium — changes incoming damage |
| Apply flat HP in combat state | `combat/logic.rs` or `core/tick.rs` | High — must NOT leak into enemy gen |
| Apply crit bonus | `combat/logic.rs` | Low — already sums crit sources |
| UI display | `ui/stats_panel.rs`, `ui/prestige_confirm.rs` | Low — display only |
| No change to enemy generation | `combat/types.rs` | None — intentionally unchanged |
