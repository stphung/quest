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
