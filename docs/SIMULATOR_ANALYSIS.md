# Simulator vs Game Analysis

## Current State

The simulator duplicates some game logic instead of sharing it. This doc identifies what's shared vs duplicated.

## ✅ Already Shared (Good!)

| Component | File | Notes |
|-----------|------|-------|
| Derived Stats | `character/derived_stats.rs` | `DerivedStats::calculate_derived_stats()` |
| Enemy Generation | `combat/types.rs` | `generate_enemy_for_current_zone()`, `generate_subzone_boss()` |
| Zone Data | `zones/data.rs` | `get_zone()`, `get_all_zones()` |
| Balance Constants | `core/balance.rs` | HP, damage, XP formulas |
| Attribute Distribution | `core/balance.rs` | `distribute_level_up_points()`, `attributes_at_level()` |
| Item Generation | `items/generation.rs` | `generate_item()` |
| Item Scoring | `items/scoring.rs` | `auto_equip_if_better()` |
| Drop Rates | `items/drops.rs` | `drop_chance_for_prestige()`, `ilvl_for_zone()` |

## ❌ Duplicated (Should Refactor)

### 1. Combat Loop
| Sim | Game | Difference |
|-----|------|------------|
| `simulate_combat()` in `combat_sim.rs` | `update_combat()` in `combat/logic.rs` | Sim is tick-based, game is time-based |

**Solution:** Extract core combat math to shared function:
```rust
// New: core/combat.rs
pub struct CombatRound {
    player_damage: u32,
    player_crit: bool,
    enemy_damage: u32,
    player_hp_after: u32,
    enemy_hp_after: u32,
}

pub fn calculate_combat_round(
    player: &DerivedStats,
    enemy_hp: u32,
    enemy_damage: u32,
    player_current_hp: u32,
    rng: &mut impl Rng,
) -> CombatRound
```

### 2. Prestige Requirements
| Sim | Game |
|-----|------|
| Hardcoded in `progression_sim.rs` | `get_prestige_tier()` in `character/prestige.rs` |

**Solution:** Sim should call `get_prestige_tier(rank).required_level`

### 3. XP for Level
| Sim | Game |
|-----|------|
| `xp_for_level()` in `progression_sim.rs` | `xp_for_next_level()` in `core/game_logic.rs` |

Both use same formula now (via `balance.rs`), but sim has its own function.

**Solution:** Remove sim's `xp_for_level()`, use `core::balance::xp_required_for_level()`

### 4. Zone Progression State
| Sim | Game |
|-----|------|
| `SimProgression` (259 lines) | `ZoneProgression` + `GameState` |

Sim tracks: zone, subzone, kills, prestige, xp, level
Game tracks: same + achievements, equipment, etc.

**Solution:** Create lightweight `ProgressionState` trait both can implement.

### 5. Boss Spawn Logic
| Sim | Game |
|-----|------|
| `should_spawn_boss()` checks `kills >= 10` | `should_spawn_boss()` checks `kills >= KILLS_FOR_BOSS` |

**Solution:** Both should use `KILLS_PER_BOSS` from `balance.rs` (sim already does, verify game does too)

### 6. Death Penalty
| Sim | Game |
|-----|------|
| `record_death(was_boss)` resets kills if boss | `combat/logic.rs` resets kills if boss |

Same logic but duplicated.

### 7. Loot Rolling
| Sim | Game |
|-----|------|
| `roll_mob_drop_real()` | `try_drop_from_mob()` |
| `roll_boss_drop_real()` | `try_drop_from_boss()` |

Sim wraps game functions but adds its own RNG handling.

## Proposed Refactor

### Phase 1: Extract Shared Traits
```rust
// core/progression.rs
pub trait Progression {
    fn current_zone(&self) -> u32;
    fn current_subzone(&self) -> u32;
    fn kills_in_subzone(&self) -> u32;
    fn should_spawn_boss(&self) -> bool;
    fn record_kill(&mut self, was_boss: bool);
    fn record_death(&mut self, was_boss: bool);
    fn advance_after_boss(&mut self);
}
```

### Phase 2: Shared Combat Math
```rust
// core/combat_math.rs
pub fn calculate_player_attack(stats: &DerivedStats, rng: &mut impl Rng) -> (u32, bool);
pub fn calculate_damage_taken(enemy_dmg: u32, player_defense: u32) -> u32;
pub fn is_player_alive(current_hp: u32) -> bool;
```

### Phase 3: Shared Prestige Logic
```rust
// character/prestige.rs (add)
pub fn required_level_for_prestige(rank: u32) -> u32;
pub fn max_zone_for_prestige(rank: u32) -> u32;
```

## Files to Modify

1. **Create:** `src/core/combat_math.rs` - Pure combat calculations
2. **Create:** `src/core/progression_trait.rs` - Shared progression interface
3. **Modify:** `src/character/prestige.rs` - Export level requirements
4. **Modify:** `src/simulator/combat_sim.rs` - Use shared combat math
5. **Modify:** `src/simulator/progression_sim.rs` - Use shared prestige logic
6. **Delete:** Duplicated XP/prestige calculations from sim

## Completed Refactoring

### Phase 1: Shared Progression Trait ✅
- Created `core/progression.rs` with `Progression` trait
- `SimProgression` implements the trait
- Shared functions: `max_zone_for_prestige()`, `can_access_zone()`

### Phase 2: Shared Combat Math ✅
- Created `core/combat_math.rs` with pure functions
- Functions: `calculate_player_attack()`, `calculate_damage_taken()`, etc.
- `SimPlayer` uses shared functions

### Phase 3: Game Uses Shared Code ✅
- `combat/logic.rs` now uses `combat_math` functions
- Player attack, enemy damage, damage reflection all use shared code

## Remaining Work

- [ ] `ZoneProgression` could implement `Progression` trait (lower priority)
  - Requires handling prestige_rank which is stored in GameState, not ZoneProgression
  - Would need a wrapper struct or parameter passing

## Summary

**Lines of shared code:** ~360 lines
**Duplicated code eliminated:** ~80 lines
**All tests passing:** 962
