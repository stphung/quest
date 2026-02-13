# Balance Tuning Architecture: Beta Test Findings Response

**Status:** ACTIVE
**Author:** System Architect
**Blocked by:** Beta test reports (completed)
**Blocks:** Tasks #3, #4 (implementation)

---

## 1. Executive Summary

Beta testing revealed five critical balance issues with the current zone-based static enemy system. While the previous refactor (replacing player-HP-scaled enemies with zone-based static stats) was architecturally correct, the **specific numeric values** need tuning. This document provides root cause analysis for each finding, proposes exact constant changes, specifies the implementation plan, and assesses risk.

**Key insight:** All five issues trace to the same root cause -- the zone stat table and prestige bonus formulas were designed around theoretical "at-level" player models that overestimated fight difficulty. In practice, players accumulate equipment bonuses and prestige bonuses faster than the model predicted, making content trivially easy.

---

## 2. Root Cause Analysis

### Issue 1: Mob fights too fast (2.2s avg vs 5-10s target)

**Observed:** Zone 1 mobs die in 2-3 hits, averaging 2.2s per fight.

**Root cause:** Zone 1 base HP of 30 is too low relative to player damage.

At level 1 with base attributes (STR 10, INT 10), player total damage = 10. Against 30 HP enemies with 0 defense, that is 3 hits = 4.5s at the 1.5s attack interval. This is already at the low end of the target.

But in practice, by the time players have even a few levels (level 3-5), they gain +1-2 STR/INT modifier, pushing damage to 12-14. With equipment from early drops, damage reaches 16-18. Against 30 HP:
- 18 damage / 0 defense = 2 hits = 3.0s
- With a crit on hit 1, enemy dies in 1 hit = 1.5s

**Fix:** Increase Zone 1 base HP from 30 to 60. Scale all subsequent zones proportionally. This targets 4-6 hits for an at-level player, which is 6-9s (matching 5-10s target). Over-leveled players will still clear in 2-3 hits (3-4.5s), which is the intended reward for power growth.

### Issue 2: Zero deaths in Zones 1-8

**Observed:** Players never die to any mob or boss in Zones 1-8 across all prestige tiers tested.

**Root cause:** Enemy damage values are too low relative to player HP. Zone 1 mobs do 5 damage vs player's 50+ HP. That requires 10+ hits to kill the player, and the player kills mobs in 2-3 hits. The player never faces meaningful danger.

The base damage values in `ZONE_ENEMY_STATS` were set conservatively to avoid the previous "unbeatable boss" problem. The pendulum swung too far toward safety.

**Fix:** Increase base enemy damage across all zones by approximately 1.5-2x. Zone 1 base damage should be 8-10 (from 5), meaning mobs kill players in 5-6 hits. Since the player attacks every 1.5s and the enemy attacks every 2.0s, this creates:
- Player time-to-kill: 6-9s (with new HP values)
- Enemy time-to-kill player: 10-12s
- Win rate: ~85-90% for normal mobs (correct target)
- Subzone boss damage at 1.3x: ~13 damage, kills in 4 hits = 8s. Combined with boss HP, creates real danger.

### Issue 3: Dungeons 0% failure rate

**Observed:** Elite (1.5x) and Boss (2.5x) dungeon multipliers produce trivially easy encounters at all prestige levels.

**Root cause:** The dungeon multipliers were designed for the old player-scaled enemy system where base enemies already matched the player. With zone-based static stats, the base enemy is typically weaker than the player (since players are usually over-leveled for the zone they're farming). The multipliers need to be higher to compensate.

Additionally, dungeon enemies currently use `subzone_depth=1` (the weakest enemies in the zone), ignoring the player's actual subzone position. Dungeons discovered in the last subzone of a zone should be harder than those in the first.

**Fix:**
- Increase `DUNGEON_ELITE_MULTIPLIERS` HP from 1.5x to 2.5x, damage from 1.2x to 1.6x
- Increase `DUNGEON_BOSS_MULTIPLIERS` HP from 2.5x to 4.0x, damage from 1.4x to 2.0x
- These new values mirror the overworld boss multipliers, creating parity between overworld and dungeon challenge

### Issue 4: Prestige bonuses negligible

**Observed:** Prestige flat_damage ranges from +5 (P5) to +12 (P20), while player base damage at those levels is 50-140+. The prestige bonus is <10% of total damage, imperceptible to the player.

**Root cause:** The prestige bonus formulas use a low factor and exponent:
```
flat_damage = floor(2.0 * rank^0.6)  -- P5: +5, P10: +7, P20: +11
flat_defense = floor(1.0 * rank^0.55) -- P5: +2, P10: +3, P20: +5
flat_hp = floor(5.0 * rank^0.5)       -- P5: +11, P10: +15, P20: +22
```

These bonuses were designed to be "not dominant" but ended up being irrelevant. The design doc stated P20 flat_damage of +11 is "meaningful but not dominant" -- but against enemies with 300+ HP and player damage of 100+, +11 flat is a rounding error.

**Fix:** Increase all prestige bonus factors by 3-5x while keeping the diminishing returns exponents:
```
flat_damage = floor(6.0 * rank^0.8)   -- P5: +24, P10: +38, P20: +66
flat_defense = floor(3.0 * rank^0.7)  -- P5: +10, P10: +15, P20: +24
flat_hp = floor(15.0 * rank^0.7)      -- P5: +50, P10: +75, P20: +122
```

At P10, +38 flat damage on a base of 64 is +59% -- genuinely meaningful. At P20, +66 flat damage on a base of 140 is +47% -- significant but not triviliazing thanks to the higher zone HP values.

### Issue 5: P15 more productive than P20

**Observed:** P15 characters farming Zone 8 (which they trivialize) get 13x more kills than P20 characters struggling in Zone 10. This makes P20 feel strictly worse than staying at P15.

**Root cause:** The stat gap between Zone 8 and Zone 10 is too large. Zone 8 base HP is 320, Zone 10 is 450 -- a 40% increase. But Zone 8 damage is 56 and Zone 10 is 78 -- also 40% increase. The compound effect of both HP and damage increasing means Zone 10 is approximately 2x harder than Zone 8.

Meanwhile, the prestige bonus increase from P15 to P20 only adds:
- Old: +2 flat damage, +1 flat defense, +1.25% crit, +3 flat HP
- These bonuses are trivial compared to the zone stat jump

**Fix:** Two-part fix:
1. **Compress the zone stat curve for Zones 7-10** to reduce the gap between consecutive zones
2. **Increase prestige bonuses (Issue 4 fix)** so P20 provides meaningful combat advantages

The Zone 7-10 compression specifically means:
- Reduce Zone 9 and 10 HP/damage by ~15-20% from current values
- Increase Zone 7-8 HP/damage by ~5-10% to maintain the difficulty floor
- Net effect: the "wall" between Zone 8 and Zone 10 narrows from 2x to ~1.4x

---

## 3. Proposed Constant Changes

### 3.1 Zone Enemy Stats (`ZONE_ENEMY_STATS` in `src/core/constants.rs`)

Current values -> New values. Changes are: HP roughly doubled for early zones, enemy damage increased 1.5-2x across the board, zone 7-10 curve compressed.

```rust
// CURRENT:
pub const ZONE_ENEMY_STATS: [(u32, u32, u32, u32, u32, u32); 11] = [
    (30, 5, 5, 1, 0, 0),      // Zone 1: Meadow
    (50, 8, 9, 2, 1, 1),      // Zone 2: Dark Forest
    (90, 12, 16, 3, 4, 1),    // Zone 3: Mountain Pass
    (120, 15, 22, 4, 7, 2),   // Zone 4: Ancient Ruins
    (170, 18, 30, 5, 11, 2),  // Zone 5: Volcanic Wastes
    (210, 22, 38, 6, 15, 3),  // Zone 6: Frozen Tundra
    (270, 25, 48, 7, 20, 3),  // Zone 7: Crystal Caverns
    (320, 30, 56, 8, 24, 4),  // Zone 8: Sunken Kingdom
    (380, 35, 66, 9, 30, 4),  // Zone 9: Floating Isles
    (450, 40, 78, 10, 36, 5), // Zone 10: Storm Citadel
    (520, 45, 88, 12, 42, 5), // Zone 11: The Expanse
];

// NEW:
pub const ZONE_ENEMY_STATS: [(u32, u32, u32, u32, u32, u32); 11] = [
    (60, 8, 10, 2, 0, 0),      // Zone 1: Meadow (+100% HP, +100% dmg)
    (95, 12, 16, 3, 2, 1),     // Zone 2: Dark Forest (+90% HP, +78% dmg)
    (150, 18, 28, 5, 6, 2),    // Zone 3: Mountain Pass (+67% HP, +75% dmg)
    (200, 22, 38, 6, 10, 2),   // Zone 4: Ancient Ruins (+67% HP, +73% dmg)
    (260, 26, 48, 7, 15, 3),   // Zone 5: Volcanic Wastes (+53% HP, +60% dmg)
    (320, 30, 58, 8, 20, 3),   // Zone 6: Frozen Tundra (+52% HP, +53% dmg)
    (380, 32, 68, 9, 25, 4),   // Zone 7: Crystal Caverns (+41% HP, +42% dmg)
    (430, 35, 76, 10, 30, 4),  // Zone 8: Sunken Kingdom (+34% HP, +36% dmg)
    (480, 38, 84, 10, 34, 4),  // Zone 9: Floating Isles (+26% HP, +27% dmg)
    (530, 40, 92, 11, 38, 5),  // Zone 10: Storm Citadel (+18% HP, +18% dmg)
    (600, 45, 102, 12, 44, 5), // Zone 11: The Expanse (+15% HP, +16% dmg)
];
```

**Design rationale for the new curve:**

Zone 1 worked example (P0, Level 1, no equipment):
- Player: HP=50, Dmg=10, Def=0, Atk=1.5s
- Enemy: HP=60, Dmg=10, Def=0, Atk=2.0s
- Player hits to kill: ceil(60/10) = 6 hits = 9.0s (in target range)
- Enemy hits to kill player: ceil(50/10) = 5 hits = 10.0s
- **Win rate: ~80-85%** (player attacks faster, wins most fights but takes real damage)

Zone 1 worked example (P0, Level 5, minor equipment):
- Player: HP=60, Dmg=14, Def=1, Atk=1.5s
- Enemy: HP=60, Dmg=10, Def=0
- Player hits to kill: ceil(60/14) = 5 hits = 7.5s
- Enemy effective DPS: (10-1)/2.0 = 4.5/s. Damage in 7.5s: ~34. Player at 26 HP.
- **Win rate: ~90-95%.** Correct for over-leveled.

Zone 10 worked example (P20, Level 130, good equipment):
- Player: HP=500+122 prestige = 622, Dmg=140+66 prestige = 206, Def=44+24 prestige = 68
- Enemy: HP=530, Dmg=92, Def=38
- Player effective DPS: (206-38)/1.5 = 112/s. Time to kill: 530/112 = 4.7s
- Enemy effective DPS: (92-68)/2.0 = 12/s. Damage in 4.7s: ~56. Player at 566 HP.
- **Win rate: ~95%.** P20 with full prestige bonuses handles Zone 10 normal mobs well.

The Zone 9-10 gap is now 480->530 HP (10% increase) vs the old 380->450 (18% increase). This makes the P15->P20 transition smoother.

### 3.2 Dungeon Multipliers (`src/core/constants.rs`)

```rust
// CURRENT:
pub const DUNGEON_ELITE_MULTIPLIERS: (f64, f64, f64) = (1.5, 1.2, 1.3);
pub const DUNGEON_BOSS_MULTIPLIERS: (f64, f64, f64) = (2.5, 1.4, 1.5);

// NEW:
pub const DUNGEON_ELITE_MULTIPLIERS: (f64, f64, f64) = (2.5, 1.6, 1.5);
pub const DUNGEON_BOSS_MULTIPLIERS: (f64, f64, f64) = (4.0, 2.0, 2.0);
```

The new elite multipliers match the old subzone boss multipliers, and the new dungeon boss multipliers match the zone boss multipliers. This ensures dungeons present boss-level challenge, which is appropriate since dungeon failure only exits the dungeon (no prestige loss).

### 3.3 Prestige Combat Bonus Formulas (`src/core/constants.rs`)

```rust
// CURRENT:
pub const PRESTIGE_FLAT_DAMAGE_FACTOR: f64 = 2.0;
pub const PRESTIGE_FLAT_DAMAGE_EXPONENT: f64 = 0.6;
pub const PRESTIGE_FLAT_DEFENSE_FACTOR: f64 = 1.0;
pub const PRESTIGE_FLAT_DEFENSE_EXPONENT: f64 = 0.55;
pub const PRESTIGE_CRIT_PER_RANK: f64 = 0.5;
pub const PRESTIGE_CRIT_CAP: f64 = 10.0;
pub const PRESTIGE_FLAT_HP_FACTOR: f64 = 5.0;
pub const PRESTIGE_FLAT_HP_EXPONENT: f64 = 0.5;

// NEW:
pub const PRESTIGE_FLAT_DAMAGE_FACTOR: f64 = 6.0;
pub const PRESTIGE_FLAT_DAMAGE_EXPONENT: f64 = 0.8;
pub const PRESTIGE_FLAT_DEFENSE_FACTOR: f64 = 3.0;
pub const PRESTIGE_FLAT_DEFENSE_EXPONENT: f64 = 0.7;
pub const PRESTIGE_CRIT_PER_RANK: f64 = 0.75;
pub const PRESTIGE_CRIT_CAP: f64 = 15.0;
pub const PRESTIGE_FLAT_HP_FACTOR: f64 = 15.0;
pub const PRESTIGE_FLAT_HP_EXPONENT: f64 = 0.7;
```

**New prestige bonus values at key ranks:**

| Rank | Flat Damage (old/new) | Flat Defense (old/new) | Crit% (old/new) | Flat HP (old/new) |
|------|----------------------|----------------------|-----------------|-------------------|
| P1   | +2 / +6   | +1 / +3   | 0.5% / 0.75%  | +5 / +15   |
| P5   | +5 / +24  | +2 / +10  | 2.5% / 3.75%  | +11 / +50  |
| P10  | +7 / +38  | +3 / +15  | 5.0% / 7.5%   | +15 / +75  |
| P15  | +9 / +50  | +4 / +20  | 7.5% / 11.25% | +19 / +97  |
| P20  | +11 / +66 | +5 / +24  | 10% / 15%(cap) | +22 / +122 |

**Impact analysis at P10 (target zone: 5-6):**
- Old: +7 dmg on 64 base = +10.9%. New: +38 dmg on 64 base = +59.4%
- Old: +3 def on 18 base = +16.7%. New: +15 def on 18 base = +83.3%
- Old: +15 HP on 220 base = +6.8%. New: +75 HP on 220 base = +34.1%
- These are now genuinely meaningful bonuses that make prestige feel rewarding

**Impact analysis at P20 (target zone: 9-10):**
- Old: +11 dmg on 140 base = +7.9%. New: +66 dmg on 140 base = +47.1%
- Old: +5 def on 44 base = +11.4%. New: +24 def on 44 base = +54.5%
- Old: +22 HP on 500 base = +4.4%. New: +122 HP on 500 base = +24.4%
- P20 players now have a substantial edge over the zone content they're meant to farm

### 3.4 Simulator --stormbreaker Flag

Add a `--stormbreaker` CLI flag to the headless simulator (`src/bin/simulator.rs`) that forces the `TheStormbreaker` achievement to be unlocked at simulation start. This allows testing Zone 10 final boss encounters without having to simulate the full Stormbreaker acquisition path.

**Implementation:** After creating the `Achievements` struct in the simulator `run_simulation()` function, if the flag is set, call `achievements.force_unlock(AchievementId::TheStormbreaker)` (need to add this method if it does not exist, or directly set the achievement as unlocked).

---

## 4. Implementation Plan

### Phase 1: Constant Changes (Task #3)

**Files modified:**
- `src/core/constants.rs` -- Update `ZONE_ENEMY_STATS` array (11 tuples), `DUNGEON_ELITE_MULTIPLIERS`, `DUNGEON_BOSS_MULTIPLIERS`

**No code logic changes.** These are pure numeric constant updates. The enemy generation code (`src/combat/types.rs`) already reads from these constants. Changing the constants is sufficient.

**Test impact:** Tests in `src/combat/types.rs` and `src/combat/logic.rs` that assert specific enemy HP/damage values will need updating to match new constants. Tests that use relative assertions (e.g., "boss HP > normal HP") should pass without changes.

### Phase 2: Prestige Bonus Scaling (Task #4)

**Files modified:**
- `src/core/constants.rs` -- Update 8 prestige bonus constants

**No code logic changes.** The `PrestigeCombatBonuses::from_rank()` method in `src/character/prestige.rs` already reads these constants and applies the formula. Changing the constants changes the output.

**Test impact:** Tests in `src/character/prestige.rs` that assert specific prestige bonus values (if any) will need updating. The `test_multiplier_expected_values` test tests XP multiplier, not combat bonuses, so it should be unaffected.

### Phase 3: Simulator Enhancement (Task #4)

**Files modified:**
- `src/bin/simulator.rs` -- Add `--stormbreaker` CLI flag parsing and achievement forcing

**Low risk.** The simulator is a standalone binary that does not affect the game.

### Implementation Order

1. **Task #3 first:** Zone stats and dungeon multipliers -- these are the most impactful changes and affect the widest range of gameplay scenarios.
2. **Task #4 second:** Prestige bonuses and simulator flag -- these layer on top of the zone stat changes.

Both tasks can proceed in parallel since they modify different constants in the same file. However, they should NOT be merged simultaneously -- Task #3 should land first so beta testers can validate zone balance before prestige bonuses are added.

---

## 5. File Change List

### Task #3: Mob HP/Damage Tuning and Dungeon Multipliers

| File | Change Type | Description |
|------|-------------|-------------|
| `src/core/constants.rs` | Modify | Update `ZONE_ENEMY_STATS` (11 entries), `DUNGEON_ELITE_MULTIPLIERS`, `DUNGEON_BOSS_MULTIPLIERS` |
| `src/combat/logic.rs` (tests) | Modify | Update any tests with hardcoded enemy stat expectations |
| `src/combat/types.rs` (tests) | Modify | Update enemy generation tests for new stat values |
| `tests/` | Modify | Update integration tests with hardcoded stat expectations |

### Task #4: Prestige Bonus Scaling and Simulator Flag

| File | Change Type | Description |
|------|-------------|-------------|
| `src/core/constants.rs` | Modify | Update 8 `PRESTIGE_FLAT_*` constants |
| `src/bin/simulator.rs` | Modify | Add `--stormbreaker` flag, force achievement on startup |
| `src/character/prestige.rs` (tests) | Modify | Update tests that assert specific prestige bonus values |

### Files NOT Changed

| File | Reason |
|------|--------|
| `src/combat/types.rs` (code) | Enemy generation logic is unchanged; it reads from constants |
| `src/combat/logic.rs` (code) | Combat logic is unchanged; prestige bonuses already applied correctly |
| `src/character/prestige.rs` (code) | `PrestigeCombatBonuses::from_rank()` reads from constants, no logic change |
| `src/character/derived_stats.rs` | No changes needed |
| `src/core/game_logic.rs` | No changes needed |
| `src/core/tick.rs` | No changes needed |
| `src/zones/*` | Zone definitions unchanged |
| `src/dungeon/*` | Dungeon logic unchanged |
| `src/haven/*` | Haven unchanged |
| `src/items/*` | Item system unchanged |
| `src/ui/*` | UI unchanged |

---

## 6. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Zone stat increases make P0 fresh start too hard | Medium | High | Zone 1 damage=10 vs player HP=50 still gives 5 hits to die. Win rate ~80-85%. If too hard, reduce Zone 1 damage to 8. |
| Prestige bonus 3x increase is too strong for late game | Medium | Medium | Diminishing returns exponents (0.7-0.8) prevent runaway. P20 bonuses are ~47% of base damage, significant but not trivializing. |
| Dungeon boss multiplier increase (4.0x HP) makes dungeons too hard | Low | Medium | 4.0x matches zone bosses. Dungeon death is safe (no prestige loss). Players can choose to avoid dungeons. |
| Compressed Zone 7-10 curve makes late zones too similar | Low | Low | HP increases are 12-15% between zones, still noticeable. Damage scaling and defense provide additional differentiation. |
| Existing tests break | Certain | Low | Tests only fail on hardcoded stat values. Fix by updating expected values. No logic changes, so no new bugs from code. |
| Save file incompatibility | None | None | All changes are in constants. Enemy structs are regenerated on spawn. Prestige bonuses are recalculated from rank. No migration needed. |

### Worst-Case Scenario

If the new values are still wrong after beta testing, the fix is purely numeric: change constant values in `src/core/constants.rs`. No code logic needs to change. The architecture is already correct; only the tuning parameters need adjustment.

---

## 7. Validation Criteria

The balance tuning is successful when the simulator confirms:

### P0 Fresh Start (Zones 1-2)
- Normal mob fight duration: 6-9s (Zone 1 subzone 1)
- Player death rate: 5-15% against normal mobs
- Zone 1 boss win rate: 30-50% at-level (level ~10)
- Total time to clear Zone 1: 20-40 minutes

### P5 Mid-Early (Zones 3-4)
- Normal mob fight duration: 5-8s
- Player death rate: 5-10% against normal mobs
- Prestige bonuses visibly reduce fight duration vs P0 at same level

### P10 Mid-Game (Zones 5-6)
- Normal mob fight duration: 5-8s
- Dungeon elite fight duration: 8-14s
- Dungeon failure rate: 10-25%

### P15 Late-Game (Zones 7-8)
- Zone 8 normal mobs: 5-8s fight duration
- Zone 8 is not trivially farmable (occasional deaths)

### P20 End-Game (Zones 9-10)
- Zone 10 normal mobs: 5-8s fight duration
- P20 is more productive than P15 (kills/hour is higher, not lower)
- Zone 10 boss requires engagement (not auto-win)
- Stormbreaker gate functions correctly (simulator --stormbreaker flag)

---

## 8. Backward Compatibility

**No breaking changes.** All modifications are to numeric constants:
- `ZONE_ENEMY_STATS` array values change, but the array structure is identical
- `DUNGEON_*_MULTIPLIERS` tuple values change, but the tuple structure is identical
- `PRESTIGE_FLAT_*` constant values change
- Enemy generation functions remain unchanged
- Combat logic remains unchanged
- Save format is unchanged
- No new structs, fields, or function signatures

Players loading old saves will immediately see new enemy stats on the next spawn. Current in-progress enemy fights use the old stats until that enemy dies (at most one fight). This is acceptable and requires no migration.
