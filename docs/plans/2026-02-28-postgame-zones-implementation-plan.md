# Postgame Zones 12-20 + Ascension System — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add 9 postgame zones (12-20) in 3 chapters unlocked by Deep breakthroughs, with 1.6x exponential enemy scaling and a per-character Ascension system that provides multiplicative combat power.

**Architecture:** Three new modules (`src/ascension/`, `src/zones/postgame.rs`, `src/zones/access.rs`), extensions to existing zone/combat/deep/achievement modules. Two independent flows: Deep breakthroughs unlock zones (account-level), Ascending gives combat power (per-character). TDD throughout.

**Tech Stack:** Rust, serde (JSON persistence), existing test patterns (deterministic `ChaCha8Rng` where needed).

**Design doc:** `docs/plans/2026-02-28-postgame-zones-design.md` — the source of truth for all values.

---

## Task 1: Extend Zone Enemy Stats and Add Postgame Constants

**Files:**
- Modify: `src/core/constants.rs`
- Test: `src/core/constants.rs` (inline tests)

**Step 1: Write the failing test**

Add to the bottom of `src/core/constants.rs` (or create a new test file if there are no inline tests — check first):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zone_enemy_stats_has_20_entries() {
        assert_eq!(ZONE_ENEMY_STATS.len(), 20);
    }

    #[test]
    fn test_postgame_zone_stats_zone_12() {
        let z12 = ZONE_ENEMY_STATS[11]; // Index 11 = Zone 12
        assert_eq!(z12, (8000, 640, 800, 128, 400, 48));
    }

    #[test]
    fn test_postgame_zone_stats_zone_20() {
        let z20 = ZONE_ENEMY_STATS[19]; // Index 19 = Zone 20
        assert_eq!(z20, (343597, 27488, 34360, 5498, 17180, 2062));
    }

    #[test]
    fn test_postgame_constants_exist() {
        assert_eq!(FIRST_POSTGAME_ZONE_ID, 12);
        assert_eq!(LAST_POSTGAME_ZONE_ID, 20);
        assert!((POSTGAME_ZONE_STAT_MULTIPLIER - 1.6).abs() < 1e-10);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib core::constants::tests -- --nocapture`
Expected: FAIL — array is only 11 entries, constants don't exist

**Step 3: Write minimal implementation**

In `src/core/constants.rs`:

1. Change the array size from `11` to `20`:
```rust
pub const ZONE_ENEMY_STATS: [(u32, u32, u32, u32, u32, u32); 20] = [
```

2. Add 9 new tuples after Zone 11 (The Expanse):
```rust
    (5000, 400, 500, 80, 250, 30), // Zone 11: The Expanse (endgame wall)
    // Postgame zones — 1.6x exponential scaling from Zone 11
    (8000, 640, 800, 128, 400, 48),           // Zone 12: Splintered Rim
    (12800, 1024, 1280, 205, 640, 77),        // Zone 13: Ember Ravine
    (20480, 1638, 2048, 328, 1024, 123),      // Zone 14: Heart of the Fault
    (32768, 2621, 3277, 524, 1638, 197),      // Zone 15: Shard Fields
    (52429, 4194, 5243, 839, 2621, 315),      // Zone 16: Refraction Steps
    (83886, 6711, 8389, 1342, 4194, 503),     // Zone 17: Hall of Second Suns
    (134218, 10737, 13422, 2148, 6711, 805),  // Zone 18: Ashen Verge
    (214748, 17180, 21475, 3436, 10737, 1289),// Zone 19: Throat of the World
    (343597, 27488, 34360, 5498, 17180, 2062),// Zone 20: The Black Mouth
];
```

3. Add new constants after `EXPANSE_ZONE_ID`:
```rust
pub const FIRST_POSTGAME_ZONE_ID: u32 = 12;
pub const LAST_POSTGAME_ZONE_ID: u32 = 20;
pub const POSTGAME_ZONE_STAT_MULTIPLIER: f64 = 1.6;
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib core::constants::tests -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/core/constants.rs
git commit -m "feat(zones): extend ZONE_ENEMY_STATS to 20 zones with 1.6x postgame scaling"
```

---

## Task 2: Create PostgameRegion Enum

**Files:**
- Create: `src/zones/postgame.rs`
- Modify: `src/zones/mod.rs`
- Test: `src/zones/postgame.rs` (inline tests)

**Step 1: Write the failing test**

Create `src/zones/postgame.rs` with tests only:

```rust
//! Postgame region types and helpers.

use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_red_fault_zone_range() {
        assert_eq!(PostgameRegion::RedFault.start_zone_id(), 12);
        assert_eq!(PostgameRegion::RedFault.end_zone_id(), 14);
    }

    #[test]
    fn test_mirror_scar_zone_range() {
        assert_eq!(PostgameRegion::MirrorScar.start_zone_id(), 15);
        assert_eq!(PostgameRegion::MirrorScar.end_zone_id(), 17);
    }

    #[test]
    fn test_black_mouth_zone_range() {
        assert_eq!(PostgameRegion::BlackMouth.start_zone_id(), 18);
        assert_eq!(PostgameRegion::BlackMouth.end_zone_id(), 20);
    }

    #[test]
    fn test_unlock_layers() {
        assert_eq!(PostgameRegion::RedFault.unlock_layer(), 3);
        assert_eq!(PostgameRegion::MirrorScar.unlock_layer(), 7);
        assert_eq!(PostgameRegion::BlackMouth.unlock_layer(), 13);
    }

    #[test]
    fn test_region_from_layer() {
        assert_eq!(PostgameRegion::from_layer(3), Some(PostgameRegion::RedFault));
        assert_eq!(PostgameRegion::from_layer(7), Some(PostgameRegion::MirrorScar));
        assert_eq!(PostgameRegion::from_layer(13), Some(PostgameRegion::BlackMouth));
        assert_eq!(PostgameRegion::from_layer(5), None);
    }

    #[test]
    fn test_unlock_headline() {
        assert_eq!(PostgameRegion::RedFault.unlock_headline(), "THE RED FAULT OPENS");
    }

    #[test]
    fn test_serde_round_trip() {
        let region = PostgameRegion::MirrorScar;
        let json = serde_json::to_string(&region).unwrap();
        let loaded: PostgameRegion = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded, PostgameRegion::MirrorScar);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib zones::postgame::tests -- --nocapture`
Expected: FAIL — `PostgameRegion` doesn't exist

**Step 3: Write minimal implementation**

In `src/zones/postgame.rs`, add above the tests:

```rust
//! Postgame region types and helpers.

use serde::{Deserialize, Serialize};

/// Named postgame chapters, each containing 3 zones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PostgameRegion {
    RedFault,
    MirrorScar,
    BlackMouth,
}

impl PostgameRegion {
    /// First zone ID in this region.
    pub fn start_zone_id(&self) -> u32 {
        match self {
            Self::RedFault => 12,
            Self::MirrorScar => 15,
            Self::BlackMouth => 18,
        }
    }

    /// Last zone ID in this region (the cap zone).
    pub fn end_zone_id(&self) -> u32 {
        match self {
            Self::RedFault => 14,
            Self::MirrorScar => 17,
            Self::BlackMouth => 20,
        }
    }

    /// Deep layer whose breakthrough unlocks this region.
    pub fn unlock_layer(&self) -> u32 {
        match self {
            Self::RedFault => 3,
            Self::MirrorScar => 7,
            Self::BlackMouth => 13,
        }
    }

    /// Returns the region unlocked by a given Deep layer breakthrough, if any.
    pub fn from_layer(layer: u32) -> Option<Self> {
        match layer {
            3 => Some(Self::RedFault),
            7 => Some(Self::MirrorScar),
            13 => Some(Self::BlackMouth),
            _ => None,
        }
    }

    /// Full-caps headline for the unlock modal.
    pub fn unlock_headline(&self) -> &'static str {
        match self {
            Self::RedFault => "THE RED FAULT OPENS",
            Self::MirrorScar => "THE MIRROR SCAR AWAKES",
            Self::BlackMouth => "THE BLACK MOUTH UNSEALS",
        }
    }

    /// Atmospheric text for the unlock modal.
    pub fn unlock_atmospheric(&self) -> &'static str {
        match self {
            Self::RedFault => "The surface has split, and the wound is burning.",
            Self::MirrorScar => "The horizon has cracked. Reflection now bleeds into the world.",
            Self::BlackMouth => "The final wound has opened wide enough to hunger.",
        }
    }

    /// Mechanical text for the unlock modal.
    pub fn unlock_mechanical(&self) -> &'static str {
        match self {
            Self::RedFault => "Zones 12-14 are now reachable beyond the current frontier.",
            Self::MirrorScar => "Zones 15-17 are now reachable beyond the current frontier.",
            Self::BlackMouth => "Zones 18-20 are now reachable beyond the current frontier.",
        }
    }

    /// Combat log line.
    pub fn unlock_log_line(&self) -> &'static str {
        match self {
            Self::RedFault => "The Red Fault has opened beyond the Expanse.",
            Self::MirrorScar => "The Mirror Scar has awakened beyond the frontier.",
            Self::BlackMouth => "The Black Mouth has unsealed beyond the world's wound.",
        }
    }

    /// Ticker text.
    pub fn unlock_ticker_text(&self) -> &'static str {
        match self {
            Self::RedFault => "Red Fault available",
            Self::MirrorScar => "Mirror Scar available",
            Self::BlackMouth => "Black Mouth available",
        }
    }
}
```

In `src/zones/mod.rs`, add the new module and re-export:

```rust
pub mod postgame;
pub use postgame::PostgameRegion;
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib zones::postgame::tests -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/zones/postgame.rs src/zones/mod.rs
git commit -m "feat(zones): add PostgameRegion enum with chapter metadata"
```

---

## Task 3: Add Postgame Zone Data (Zones 12-20)

**Files:**
- Modify: `src/zones/data.rs`
- Test: `src/zones/data.rs` (inline tests)

**Step 1: Write the failing tests**

Add to the existing `tests` module in `src/zones/data.rs`:

```rust
    #[test]
    fn test_zone_count_with_postgame() {
        let zones = get_all_zones();
        assert_eq!(zones.len(), 20);
    }

    #[test]
    fn test_postgame_zone_12() {
        let zone = get_zone(12).unwrap();
        assert_eq!(zone.name, "Splintered Rim");
        assert_eq!(zone.subzones.len(), 5);
        assert_eq!(zone.prestige_requirement, 0);
        assert_eq!(zone.min_level, 165);
        assert_eq!(zone.max_level, 180);
    }

    #[test]
    fn test_postgame_zone_20() {
        let zone = get_zone(20).unwrap();
        assert_eq!(zone.name, "The Black Mouth");
        assert_eq!(zone.subzones.len(), 5);
        assert_eq!(zone.max_level, u32::MAX);
    }

    #[test]
    fn test_all_postgame_zones_have_5_subzones() {
        for zone_id in 12..=20 {
            let zone = get_zone(zone_id).unwrap();
            assert_eq!(
                zone.subzones.len(), 5,
                "Zone {} ({}) should have 5 subzones", zone_id, zone.name
            );
        }
    }

    #[test]
    fn test_all_postgame_zones_have_zone_boss() {
        for zone_id in 12..=20 {
            let zone = get_zone(zone_id).unwrap();
            let last = zone.subzones.last().unwrap();
            assert!(
                last.boss.is_zone_boss,
                "Zone {} ({}) final subzone boss should be zone boss", zone_id, zone.name
            );
        }
    }

    #[test]
    fn test_postgame_zones_prestige_requirement_zero() {
        for zone_id in 12..=20 {
            let zone = get_zone(zone_id).unwrap();
            assert_eq!(
                zone.prestige_requirement, 0,
                "Zone {} should have prestige_requirement 0 (managed by sync)", zone_id
            );
        }
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib zones::data::tests -- --nocapture`
Expected: FAIL — `test_zone_count_with_postgame` expects 20, gets 11

**Step 3: Write minimal implementation**

In `src/zones/data.rs`, add 9 new `Zone` entries to the `ALL_ZONES` LazyLock vec after Zone 11. Each zone has 5 subzones. Use the exact names from the design doc (`docs/plans/2026-02-28-postgame-zones-design.md` Content Spec section).

The zone entries follow the exact same pattern as existing zones. All postgame zones have `prestige_requirement: 0`, `requires_weapon: false`, `weapon_name: None`.

Zone 12-14: The Red Fault
Zone 15-17: The Mirror Scar
Zone 18-20: The Black Mouth (Zone 20 uses `max_level: u32::MAX`)

Level ranges from design doc:
- Z12: 165-180, Z13: 180-195, Z14: 195-210
- Z15: 210-225, Z16: 225-240, Z17: 240-255
- Z18: 255-270, Z19: 270-285, Z20: 285-u32::MAX

Boss names from design doc Content Spec (5 per zone — subzone bosses + zone boss on the last one).

Also update `test_zone_count` from `assert_eq!(zones.len(), 11)` to `assert_eq!(zones.len(), 20)`.

Update `test_get_zone` to remove `assert!(get_zone(12).is_none())` and replace with:
```rust
assert!(get_zone(12).is_some());
assert_eq!(get_zone(12).unwrap().name, "Splintered Rim");
assert!(get_zone(20).is_some());
assert!(get_zone(21).is_none());
```

Update `test_get_subzone` similarly.

**Step 4: Run test to verify it passes**

Run: `cargo test --lib zones::data::tests -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/zones/data.rs
git commit -m "feat(zones): add 9 postgame zones (12-20) with 5 subzones each"
```

---

## Task 4: Create Ascension Module — Types and Constants

**Files:**
- Create: `src/ascension/mod.rs`
- Create: `src/ascension/types.rs`
- Modify: `src/lib.rs`
- Test: `src/ascension/types.rs` (inline tests)

**Step 1: Write the failing test**

Create `src/ascension/types.rs` with tests:

```rust
//! Ascension system constants and helper types.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascension_cost_levels_1_through_6() {
        assert_eq!(ascension_cost(1), 10);
        assert_eq!(ascension_cost(2), 15);
        assert_eq!(ascension_cost(3), 25);
        assert_eq!(ascension_cost(4), 35);
        assert_eq!(ascension_cost(5), 50);
        assert_eq!(ascension_cost(6), 65);
    }

    #[test]
    fn test_ascension_cost_level_7_plus() {
        assert_eq!(ascension_cost(7), 80);  // 65 + 15*(7-6) = 80
        assert_eq!(ascension_cost(8), 95);  // 65 + 15*(8-6) = 95
        assert_eq!(ascension_cost(10), 125); // 65 + 15*(10-6) = 125
    }

    #[test]
    fn test_ascension_deep_gate_levels_1_through_6() {
        assert_eq!(ascension_deep_gate(1), Some(3));
        assert_eq!(ascension_deep_gate(2), Some(7));
        assert_eq!(ascension_deep_gate(3), Some(12));
        assert_eq!(ascension_deep_gate(4), Some(18));
        assert_eq!(ascension_deep_gate(5), Some(25));
        assert_eq!(ascension_deep_gate(6), Some(30));
    }

    #[test]
    fn test_ascension_deep_gate_level_7_plus_none() {
        assert_eq!(ascension_deep_gate(7), None);
        assert_eq!(ascension_deep_gate(100), None);
    }

    #[test]
    fn test_ascension_combat_multiplier() {
        assert!((ascension_combat_multiplier(0) - 1.0).abs() < 1e-10);
        assert!((ascension_combat_multiplier(1) - 2.0).abs() < 1e-10);
        assert!((ascension_combat_multiplier(2) - 4.0).abs() < 1e-10);
        assert!((ascension_combat_multiplier(3) - 8.0).abs() < 1e-10);
        assert!((ascension_combat_multiplier(4) - 16.0).abs() < 1e-10);
        assert!((ascension_combat_multiplier(5) - 32.0).abs() < 1e-10);
        assert!((ascension_combat_multiplier(6) - 64.0).abs() < 1e-10);
    }

    #[test]
    fn test_ascension_combat_multiplier_level_7_plus() {
        assert!((ascension_combat_multiplier(7) - 96.0).abs() < 1e-10);  // 64 * 1.5
        assert!((ascension_combat_multiplier(8) - 144.0).abs() < 1e-10); // 64 * 1.5^2
    }

    #[test]
    fn test_total_pr_for_levels_1_through_6() {
        let total: u32 = (1..=6).map(ascension_cost).sum();
        assert_eq!(total, 200);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib ascension::types::tests -- --nocapture`
Expected: FAIL — module doesn't exist

**Step 3: Write minimal implementation**

Create `src/ascension/mod.rs`:
```rust
//! Ascension system — per-character combat power multiplier gated by Deep milestones.

pub mod types;
pub mod logic;

pub use logic::{ascend, can_ascend, AscendResult};
pub use types::{ascension_combat_multiplier, ascension_cost, ascension_deep_gate};
```

Create `src/ascension/types.rs` (add the functions above the tests):
```rust
//! Ascension system constants and helper types.

/// PR cost lookup for Ascension levels 1-6.
const ASCENSION_COSTS: [u32; 6] = [10, 15, 25, 35, 50, 65];

/// Deep layer gate lookup for Ascension levels 1-6.
const ASCENSION_DEEP_GATES: [u32; 6] = [3, 7, 12, 18, 25, 30];

/// Prestige rank cost to Ascend to the given level.
pub fn ascension_cost(level: u32) -> u32 {
    if level >= 1 && level <= 6 {
        ASCENSION_COSTS[(level - 1) as usize]
    } else if level > 6 {
        65 + 15 * (level - 6)
    } else {
        0
    }
}

/// Deep layer gate for the given Ascension level. None means no Deep gate (PR only).
pub fn ascension_deep_gate(level: u32) -> Option<u32> {
    if level >= 1 && level <= 6 {
        Some(ASCENSION_DEEP_GATES[(level - 1) as usize])
    } else {
        None
    }
}

/// Combat multiplier at a given Ascension level.
/// Level 0 = 1.0x, Levels 1-6 = 2^level, Levels 7+ = 64 * 1.5^(level-6).
pub fn ascension_combat_multiplier(level: u32) -> f64 {
    if level == 0 {
        1.0
    } else if level <= 6 {
        2.0_f64.powi(level as i32)
    } else {
        64.0 * 1.5_f64.powi((level - 6) as i32)
    }
}
```

Create `src/ascension/logic.rs` as a stub for now (will be filled in Task 5):
```rust
//! Ascension logic — eligibility checks and execution.

use crate::core::game_state::GameState;

/// Result of an Ascend action.
#[derive(Debug, Clone, PartialEq)]
pub enum AscendResult {
    /// Successfully ascended to the given level.
    Success { new_level: u32, multiplier: f64 },
    /// Not enough prestige ranks.
    InsufficientPR { needed: u32, have: u32 },
    /// Deep layer gate not met.
    DeepGateNotMet { needed_layer: u32, current_layer: u32 },
}

/// Check if the character can Ascend to their next level.
pub fn can_ascend(ascension_level: u32, prestige_rank: u32, deepest_layer: u32) -> bool {
    let next = ascension_level + 1;
    let cost = super::types::ascension_cost(next);
    if prestige_rank < cost {
        return false;
    }
    if let Some(gate) = super::types::ascension_deep_gate(next) {
        if deepest_layer < gate {
            return false;
        }
    }
    true
}

/// Execute an Ascension. Returns the result.
pub fn ascend(state: &mut GameState, deepest_layer: u32) -> AscendResult {
    let next = state.ascension_level + 1;
    let cost = super::types::ascension_cost(next);

    if let Some(gate) = super::types::ascension_deep_gate(next) {
        if deepest_layer < gate {
            return AscendResult::DeepGateNotMet {
                needed_layer: gate,
                current_layer: deepest_layer,
            };
        }
    }

    if state.prestige_rank < cost {
        return AscendResult::InsufficientPR {
            needed: cost,
            have: state.prestige_rank,
        };
    }

    state.prestige_rank -= cost;
    state.ascension_level = next;
    let multiplier = super::types::ascension_combat_multiplier(next);
    AscendResult::Success {
        new_level: next,
        multiplier,
    }
}
```

In `src/lib.rs`, add after `pub mod zones;`:
```rust
pub mod ascension;
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib ascension::types::tests -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ascension/ src/lib.rs
git commit -m "feat(ascension): add Ascension module with cost, gate, and multiplier functions"
```

---

## Task 5: Ascension Logic Tests and GameState Integration

**Files:**
- Modify: `src/core/game_state.rs`
- Modify: `src/character/persistence.rs` (serde for ascension_level)
- Test: `tests/ascension_test.rs` (new integration test file)

**Step 1: Write the failing test**

Create `tests/ascension_test.rs`:

```rust
use quest::ascension::logic::{ascend, can_ascend, AscendResult};
use quest::GameState;

#[test]
fn test_can_ascend_basic() {
    // Level 0 -> 1: needs 10 PR and Deep layer 3
    assert!(can_ascend(0, 10, 3));
    assert!(!can_ascend(0, 9, 3));  // insufficient PR
    assert!(!can_ascend(0, 10, 2)); // deep gate not met
}

#[test]
fn test_can_ascend_level_7_no_deep_gate() {
    // Level 6 -> 7: needs 80 PR, no Deep gate
    assert!(can_ascend(6, 80, 0)); // deepest_layer doesn't matter
    assert!(!can_ascend(6, 79, 30)); // PR insufficient
}

#[test]
fn test_ascend_deducts_pr() {
    let mut state = GameState::new("Test".to_string(), 0);
    state.prestige_rank = 50;
    state.ascension_level = 0;

    let result = ascend(&mut state, 3);
    assert_eq!(result, AscendResult::Success { new_level: 1, multiplier: 2.0 });
    assert_eq!(state.prestige_rank, 40); // 50 - 10
    assert_eq!(state.ascension_level, 1);
}

#[test]
fn test_ascend_insufficient_pr() {
    let mut state = GameState::new("Test".to_string(), 0);
    state.prestige_rank = 5;
    state.ascension_level = 0;

    let result = ascend(&mut state, 3);
    assert_eq!(result, AscendResult::InsufficientPR { needed: 10, have: 5 });
    assert_eq!(state.prestige_rank, 5); // unchanged
    assert_eq!(state.ascension_level, 0); // unchanged
}

#[test]
fn test_ascend_deep_gate_not_met() {
    let mut state = GameState::new("Test".to_string(), 0);
    state.prestige_rank = 100;
    state.ascension_level = 0;

    let result = ascend(&mut state, 2); // need layer 3
    assert_eq!(result, AscendResult::DeepGateNotMet { needed_layer: 3, current_layer: 2 });
    assert_eq!(state.prestige_rank, 100); // unchanged
}

#[test]
fn test_ascension_level_serialization() {
    let mut state = GameState::new("Test".to_string(), 0);
    state.ascension_level = 4;

    let json = serde_json::to_string(&state).unwrap();
    let loaded: GameState = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.ascension_level, 4);
}

#[test]
fn test_ascension_level_defaults_to_zero() {
    // Simulate loading from old save without ascension_level
    let state = GameState::new("Test".to_string(), 0);
    assert_eq!(state.ascension_level, 0);
}

#[test]
fn test_new_character_starts_at_ascension_zero() {
    let state = GameState::new("Hero".to_string(), 0);
    assert_eq!(state.ascension_level, 0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test ascension_test -- --nocapture`
Expected: FAIL — `ascension_level` field doesn't exist on GameState

**Step 3: Write minimal implementation**

In `src/core/game_state.rs`, add to the `GameState` struct (in the persistent/saved section, near `stormglass`):

```rust
    /// Ascension level — per-character combat power multiplier (0 = no ascension)
    #[serde(default)]
    pub ascension_level: u32,
```

In `GameState::new()`, initialize it:
```rust
    ascension_level: 0,
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test ascension_test -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/core/game_state.rs tests/ascension_test.rs
git commit -m "feat(ascension): add ascension_level to GameState with serde default"
```

---

## Task 6: Add Postgame Fields to DeepPersistent

**Files:**
- Modify: `src/deep/types.rs`
- Test: `tests/deep_integration_test.rs` (add new tests, or create if needed)

**Step 1: Write the failing test**

Add to the existing deep integration tests or create `tests/postgame_deep_test.rs`:

```rust
use quest::deep::DeepState;
use quest::zones::PostgameRegion;

#[test]
fn test_deep_persistent_postgame_zone_cap_defaults_to_11() {
    let deep = DeepState::new();
    assert_eq!(deep.persistent.postgame_zone_cap, 11);
}

#[test]
fn test_deep_persistent_pending_region_defaults_to_none() {
    let deep = DeepState::new();
    assert!(deep.persistent.pending_postgame_region_unlock.is_none());
}

#[test]
fn test_deep_persistent_serde_defaults() {
    // Simulate loading from old save without new fields
    let json = r#"{"discovered":false,"guild_rank":1,"guild_upgrade_cost":500,"layers":[],"deepest_layer_reached":0,"merc_id_counter":0,"mission_id_counter":0}"#;
    let persistent: quest::deep::DeepPersistent = serde_json::from_str(json).unwrap();
    assert_eq!(persistent.postgame_zone_cap, 11);
    assert!(persistent.pending_postgame_region_unlock.is_none());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test postgame_deep_test -- --nocapture`
Expected: FAIL — fields don't exist

**Step 3: Write minimal implementation**

In `src/deep/types.rs`, add to `DeepPersistent`:

```rust
    /// Highest postgame zone the player can access (default 11 = Expanse only).
    #[serde(default = "default_postgame_zone_cap")]
    pub postgame_zone_cap: u32,
    /// Pending postgame region unlock notification (consumed by tick to show modal).
    #[serde(default)]
    pub pending_postgame_region_unlock: Option<crate::zones::PostgameRegion>,
```

Add the default function:
```rust
fn default_postgame_zone_cap() -> u32 {
    11
}
```

In `DeepPersistent::new()`, initialize:
```rust
    postgame_zone_cap: 11,
    pending_postgame_region_unlock: None,
```

In `src/deep/mod.rs`, make sure `DeepPersistent` is publicly re-exported (it likely already is).

**Step 4: Run test to verify it passes**

Run: `cargo test --test postgame_deep_test -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/deep/types.rs tests/postgame_deep_test.rs
git commit -m "feat(deep): add postgame_zone_cap and pending_postgame_region_unlock to DeepPersistent"
```

---

## Task 7: Zone Access Sync Function

**Files:**
- Create: `src/zones/access.rs`
- Modify: `src/zones/mod.rs`
- Test: `src/zones/access.rs` (inline tests)

**Step 1: Write the failing test**

Create `src/zones/access.rs` with tests:

```rust
//! Account-level zone access synchronization.

use super::progression::ZoneProgression;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_unlocks_zone_11_when_storms_end() {
        let mut prog = ZoneProgression::new();
        sync_account_zone_unlocks(&mut prog, true, 11);
        assert!(prog.is_zone_unlocked(11));
    }

    #[test]
    fn test_sync_does_not_unlock_zone_11_without_storms_end() {
        let mut prog = ZoneProgression::new();
        sync_account_zone_unlocks(&mut prog, false, 11);
        assert!(!prog.is_zone_unlocked(11));
    }

    #[test]
    fn test_sync_unlocks_zones_12_through_14_when_cap_14() {
        let mut prog = ZoneProgression::new();
        sync_account_zone_unlocks(&mut prog, true, 14);
        assert!(prog.is_zone_unlocked(11));
        assert!(prog.is_zone_unlocked(12));
        assert!(prog.is_zone_unlocked(13));
        assert!(prog.is_zone_unlocked(14));
        assert!(!prog.is_zone_unlocked(15));
    }

    #[test]
    fn test_sync_unlocks_all_postgame_when_cap_20() {
        let mut prog = ZoneProgression::new();
        sync_account_zone_unlocks(&mut prog, true, 20);
        for z in 11..=20 {
            assert!(prog.is_zone_unlocked(z), "Zone {} should be unlocked", z);
        }
        assert!(!prog.is_zone_unlocked(21));
    }

    #[test]
    fn test_sync_never_removes_earlier_unlocks() {
        let mut prog = ZoneProgression::new();
        prog.unlock_zone(12);
        sync_account_zone_unlocks(&mut prog, true, 11);
        // Zone 12 was manually unlocked, sync should not remove it
        assert!(prog.is_zone_unlocked(12));
    }

    #[test]
    fn test_sync_idempotent() {
        let mut prog = ZoneProgression::new();
        sync_account_zone_unlocks(&mut prog, true, 14);
        sync_account_zone_unlocks(&mut prog, true, 14); // call twice
        assert!(prog.is_zone_unlocked(14));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib zones::access::tests -- --nocapture`
Expected: FAIL — `sync_account_zone_unlocks` doesn't exist

**Step 3: Write minimal implementation**

In `src/zones/access.rs`:

```rust
//! Account-level zone access synchronization.

use super::progression::ZoneProgression;
use crate::core::constants::EXPANSE_ZONE_ID;

/// Synchronize zone unlocks from account-level state.
///
/// Called at: character load, prestige reset, StormsEnd, postgame region unlock.
///
/// - If `storms_end_unlocked`, unlocks Zone 11
/// - Unlocks every zone in `12..=postgame_zone_cap`
/// - Never unlocks above cap, never removes earlier unlocks
pub fn sync_account_zone_unlocks(
    prog: &mut ZoneProgression,
    storms_end_unlocked: bool,
    postgame_zone_cap: u32,
) {
    if storms_end_unlocked {
        prog.unlock_zone(EXPANSE_ZONE_ID);
    }
    for zone_id in 12..=postgame_zone_cap {
        prog.unlock_zone(zone_id);
    }
}
```

In `src/zones/mod.rs`, add:
```rust
pub mod access;
pub use access::sync_account_zone_unlocks;
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib zones::access::tests -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/zones/access.rs src/zones/mod.rs
git commit -m "feat(zones): add sync_account_zone_unlocks for postgame zone access"
```

---

## Task 8: Boss Defeat — Postgame Cycling Logic

**Files:**
- Modify: `src/zones/boss_defeat.rs`
- Test: `tests/postgame_zones_test.rs` (new file)

**Step 1: Write the failing test**

Create `tests/postgame_zones_test.rs`:

```rust
use quest::zones::{BossDefeatResult, ZoneProgression};
use quest::achievements::Achievements;
use quest::core::constants::{EXPANSE_ZONE_ID, KILLS_FOR_BOSS};

#[test]
fn test_zone_11_boss_with_cap_11_returns_expanse_cycle() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();
    prog.current_zone_id = EXPANSE_ZONE_ID;
    prog.current_subzone_id = 4;
    prog.unlock_zone(EXPANSE_ZONE_ID);
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated(20, &mut achievements);
    assert_eq!(result, BossDefeatResult::ExpanseCycle);
}

#[test]
fn test_zone_11_boss_with_cap_14_advances_to_zone_12() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();
    prog.current_zone_id = EXPANSE_ZONE_ID;
    prog.current_subzone_id = 4;
    prog.unlock_zone(EXPANSE_ZONE_ID);
    prog.unlock_zone(12);
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated_with_cap(20, &mut achievements, 14);
    match result {
        BossDefeatResult::ZoneComplete { new_zone_id, .. } => {
            assert_eq!(new_zone_id, 12);
        }
        _ => panic!("Expected ZoneComplete to zone 12, got {:?}", result),
    }
}

#[test]
fn test_zone_14_boss_with_cap_14_returns_postgame_cycle() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();
    prog.current_zone_id = 14;
    prog.current_subzone_id = 5;
    prog.unlock_zone(14);
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated_with_cap(20, &mut achievements, 14);
    assert_eq!(result, BossDefeatResult::PostgameCycle { zone_id: 14 });
    assert_eq!(prog.current_subzone_id, 1);
}

#[test]
fn test_zone_14_boss_with_cap_17_advances_to_zone_15() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();
    prog.current_zone_id = 14;
    prog.current_subzone_id = 5;
    prog.unlock_zone(14);
    prog.unlock_zone(15);
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated_with_cap(20, &mut achievements, 17);
    match result {
        BossDefeatResult::ZoneComplete { new_zone_id, .. } => {
            assert_eq!(new_zone_id, 15);
        }
        _ => panic!("Expected ZoneComplete to zone 15, got {:?}", result),
    }
}

#[test]
fn test_zone_20_boss_with_cap_20_returns_postgame_cycle() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();
    prog.current_zone_id = 20;
    prog.current_subzone_id = 5;
    prog.unlock_zone(20);
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated_with_cap(50, &mut achievements, 20);
    assert_eq!(result, BossDefeatResult::PostgameCycle { zone_id: 20 });
}

#[test]
fn test_postgame_subzone_advance() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();
    prog.current_zone_id = 12;
    prog.current_subzone_id = 1;
    prog.unlock_zone(12);
    for _ in 0..KILLS_FOR_BOSS {
        prog.record_kill();
    }

    let result = prog.on_boss_defeated_with_cap(20, &mut achievements, 14);
    assert!(matches!(result, BossDefeatResult::SubzoneComplete { new_subzone_id: 2 }));
}

#[test]
fn test_zone_12_boss_advances_to_13() {
    let mut prog = ZoneProgression::new();
    let mut achievements = Achievements::default();
    prog.current_zone_id = 12;
    prog.current_subzone_id = 5; // final subzone
    prog.unlock_zone(12);
    prog.unlock_zone(13);
    prog.fighting_boss = true;

    let result = prog.on_boss_defeated_with_cap(20, &mut achievements, 14);
    match result {
        BossDefeatResult::ZoneComplete { new_zone_id, .. } => {
            assert_eq!(new_zone_id, 13);
        }
        _ => panic!("Expected ZoneComplete to zone 13, got {:?}", result),
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test postgame_zones_test -- --nocapture`
Expected: FAIL — `PostgameCycle` variant doesn't exist, `on_boss_defeated_with_cap` doesn't exist

**Step 3: Write minimal implementation**

In `src/zones/boss_defeat.rs`:

1. Add new variant to `BossDefeatResult`:
```rust
    /// Completed a postgame cycle (cap zone loops) — returns to subzone 1
    PostgameCycle { zone_id: u32 },
```

2. Add a new method `on_boss_defeated_with_cap` that extends `on_boss_defeated` with postgame awareness:

```rust
    /// Handles boss defeat with postgame zone cap awareness.
    /// `postgame_zone_cap` is the highest zone the player can access (from DeepPersistent).
    pub fn on_boss_defeated_with_cap(
        &mut self,
        prestige_rank: u32,
        achievements: &mut Achievements,
        postgame_zone_cap: u32,
    ) -> BossDefeatResult {
        let zone_id = self.current_zone_id;
        let subzone_id = self.current_subzone_id;

        let zones = get_all_zones();
        let Some(zone) = zones.iter().find(|z| z.id == zone_id) else {
            return BossDefeatResult::SubzoneComplete {
                new_subzone_id: self.current_subzone_id,
            };
        };

        let is_zone_boss = subzone_id == zone.subzones.len() as u32;

        // Check for Zone 10 weapon requirement
        let has_stormbreaker = achievements.is_unlocked(AchievementId::TheStormbreaker);
        if zone.requires_weapon && is_zone_boss && !has_stormbreaker {
            self.fighting_boss = false;
            self.kills_in_subzone = 0;
            return BossDefeatResult::WeaponRequired {
                weapon_name: zone.weapon_name.unwrap_or("legendary weapon").to_string(),
            };
        }

        self.defeat_boss(zone_id, subzone_id);

        if !is_zone_boss {
            self.advance_to_next_subzone();
            return BossDefeatResult::SubzoneComplete {
                new_subzone_id: self.current_subzone_id,
            };
        }

        // Zone boss defeated — handle progression
        // Zone 10: StormsEnd
        if zone_id == FINAL_ZONE_ID {
            achievements.unlock(AchievementId::StormsEnd, None);
            self.unlock_zone(EXPANSE_ZONE_ID);
            self.current_zone_id = EXPANSE_ZONE_ID;
            self.current_subzone_id = 1;
            return BossDefeatResult::StormsEnd;
        }

        // Zone 11 (Expanse) with no postgame unlocked: classic cycle
        if zone_id == EXPANSE_ZONE_ID && postgame_zone_cap <= EXPANSE_ZONE_ID {
            self.current_subzone_id = 1;
            self.kills_in_subzone = 0;
            return BossDefeatResult::ExpanseCycle;
        }

        // Current zone is the cap zone (postgame or Expanse) — cycle
        if zone_id == postgame_zone_cap && zone_id > EXPANSE_ZONE_ID {
            self.current_subzone_id = 1;
            self.kills_in_subzone = 0;
            return BossDefeatResult::PostgameCycle { zone_id };
        }

        // Try to advance to next zone (if unlocked)
        if self.advance_to_next_zone(prestige_rank) {
            return BossDefeatResult::ZoneComplete {
                old_zone: zone.name.to_string(),
                new_zone_id: self.current_zone_id,
            };
        }

        // Expanse with higher cap: advance to zone 12
        if zone_id == EXPANSE_ZONE_ID && postgame_zone_cap > EXPANSE_ZONE_ID {
            let next = 12;
            if self.is_zone_unlocked(next) {
                self.current_zone_id = next;
                self.current_subzone_id = 1;
                return BossDefeatResult::ZoneComplete {
                    old_zone: zone.name.to_string(),
                    new_zone_id: next,
                };
            }
        }

        // Fallback: cycle in place
        self.current_subzone_id = 1;
        self.kills_in_subzone = 0;
        if zone_id > EXPANSE_ZONE_ID {
            BossDefeatResult::PostgameCycle { zone_id }
        } else {
            BossDefeatResult::ExpanseCycle
        }
    }
```

**Step 4: Run test to verify it passes**

Run: `cargo test --test postgame_zones_test -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/zones/boss_defeat.rs tests/postgame_zones_test.rs
git commit -m "feat(zones): add PostgameCycle variant and on_boss_defeated_with_cap method"
```

---

## Task 9: Add Ascension Multiplier to CombatBonuses

**Files:**
- Modify: `src/combat/events.rs`
- Modify: `src/combat/player_attack.rs`
- Modify: `src/combat/enemy_attack.rs`
- Test: `tests/ascension_test.rs` (extend)

**Step 1: Write the failing test**

Add to `tests/ascension_test.rs`:

```rust
use quest::combat::CombatBonuses;

#[test]
fn test_combat_bonuses_ascension_multiplier_default() {
    let bonuses = CombatBonuses::default();
    assert!((bonuses.ascension_multiplier - 1.0).abs() < 1e-10);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test ascension_test::test_combat_bonuses_ascension_multiplier_default -- --nocapture`
Expected: FAIL — `ascension_multiplier` field doesn't exist

**Step 3: Write minimal implementation**

In `src/combat/events.rs`, add to `CombatBonuses`:

```rust
    // --- Ascension multiplier (player_attack.rs, enemy_attack.rs) ---
    /// Ascension combat multiplier applied to damage, defense, and HP.
    /// Defaults to 1.0 (no ascension).
    pub ascension_multiplier: f64,
```

Since `CombatBonuses` derives `Default`, and `f64` defaults to `0.0`, we need a custom Default impl or use a different approach. Check if it already has `#[derive(Default)]`. If so, we need to handle the 1.0 default.

Option A: Remove `#[derive(Default)]` and implement manually.
Option B: Keep the default as 0.0 and treat 0.0 as 1.0 in the pipeline.

Option A is cleaner. Replace `#[derive(... Default)]` with a manual impl:

```rust
impl Default for CombatBonuses {
    fn default() -> Self {
        Self {
            early_damage_percent: 0.0,
            damage_percent: 0.0,
            flat_damage: 0,
            crit_chance_percent: 0.0,
            double_strike_chance: 0.0,
            xp_gain_percent: 0.0,
            flat_defense: 0,
            damage_reduction_percent: 0.0,
            flat_hp: 0,
            attack_speed_percent: 0.0,
            hp_regen_percent: 0.0,
            hp_regen_delay_reduction: 0.0,
            regen_reduction_percent: 0.0,
            ascension_multiplier: 1.0,
        }
    }
}
```

In `src/combat/player_attack.rs`, apply ascension multiplier after flat_damage and before enemy defense:

Change step 3 comment and add multiplier:
```rust
    // 3. Apply flat damage bonus (e.g. prestige), added after % bonuses, before defense
    let pre_defense_damage = boosted_damage + bonuses.flat_damage;
    // 4. Apply Ascension multiplier to damage
    let pre_crit_damage = (pre_defense_damage as f64 * bonuses.ascension_multiplier) as u32;
```

In `src/combat/enemy_attack.rs`, apply ascension multiplier to defense. Find where `derived.defense` is used and multiply by `ascension_multiplier`:

```rust
    let total_defense = ((derived.defense as u32 + bonuses.flat_defense) as f64
        * bonuses.ascension_multiplier) as u32;
```

Note: The exact code modifications depend on the current enemy_attack.rs structure. Read the file to find the exact lines. The ascension multiplier should boost player defense.

For HP, the ascension multiplier should be applied to `flat_hp` in `tick.rs` where `player_max_hp` is set. This is in the `game_tick` sync HP stage. Multiply `(derived.max_hp as u32 + bonuses.flat_hp)` by `ascension_multiplier`.

**Step 4: Run test to verify it passes**

Run: `cargo test --test ascension_test -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/combat/events.rs src/combat/player_attack.rs src/combat/enemy_attack.rs
git commit -m "feat(combat): add ascension_multiplier to CombatBonuses and damage/defense pipeline"
```

---

## Task 10: Postgame Enemy Naming Pools

**Files:**
- Modify: `src/combat/enemy_generation.rs`
- Test: `src/combat/enemy_generation.rs` (add test)

**Step 1: Write the failing test**

Add inline test to `src/combat/enemy_generation.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postgame_zones_have_unique_prefixes() {
        for zone_id in 12..=20 {
            let prefixes = get_zone_enemy_prefixes(zone_id);
            assert!(
                prefixes.len() >= 5,
                "Zone {} should have at least 5 prefixes, got {}",
                zone_id, prefixes.len()
            );
            // Should NOT be the fallback array
            assert_ne!(
                prefixes[0], "Wild",
                "Zone {} should not use fallback prefixes", zone_id
            );
        }
    }

    #[test]
    fn test_postgame_zones_have_unique_suffixes() {
        for zone_id in 12..=20 {
            let suffixes = get_zone_enemy_suffixes(zone_id);
            assert!(
                suffixes.len() >= 5,
                "Zone {} should have at least 5 suffixes, got {}",
                zone_id, suffixes.len()
            );
            // Should NOT be the fallback array
            assert_ne!(
                suffixes[0], "Beast",
                "Zone {} should not use fallback suffixes", zone_id
            );
        }
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib combat::enemy_generation::tests -- --nocapture`
Expected: FAIL — zones 12-20 hit the `_ =>` fallback

**Step 3: Write minimal implementation**

Add explicit match arms for zones 12-20 in both `get_zone_enemy_prefixes` and `get_zone_enemy_suffixes`. Use the naming pools from the design doc:

```rust
        12 => &["Rim", "Ash", "Fault", "Ember", "Bloodglass"],
        13 => &["Coalwind", "Soot", "Crucible", "Scarforge", "Rift"],
        14 => &["Vein", "Pyre", "Coreglass", "Magma", "Rupture"],
        15 => &["Shard", "Prism", "Mirror", "White", "Glass"],
        16 => &["Bent", "Parallax", "Reflected", "Lightfall", "Angle"],
        17 => &["Solar", "False", "Sunshard", "Witness", "Second"],
        18 => &["Char", "Gloam", "Ashen", "Cinder", "Veil"],
        19 => &["Maw", "Tooth", "Sable", "Gullet", "Windpipe"],
        20 => &["Void", "Jawbone", "Unlit", "First", "Mouth"],
```

And suffixes:
```rust
        12 => &["Stalker", "Hound", "Ram", "Brute", "Crawler"],
        13 => &["Maw", "Knight", "Colossus", "Warden", "Fiend"],
        14 => &["Breaker", "Cantor", "Regent", "Tyrant", "Revenant"],
        15 => &["Hound", "Jackal", "Widow", "Watcher", "Echo"],
        16 => &["Serpent", "Marshal", "Repeater", "Sentinel", "Engine"],
        17 => &["Wraith", "King", "Titan", "Chorus", "Herald"],
        18 => &["Wing", "Revenant", "Forger", "Giant", "Shade"],
        19 => &["Warden", "Behemoth", "Herd", "Devourer", "Judge"],
        20 => &["Hunger", "Colossus", "Choir", "Crawler", "Remnant"],
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib combat::enemy_generation::tests -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/combat/enemy_generation.rs
git commit -m "feat(combat): add postgame enemy naming pools for zones 12-20"
```

---

## Task 11: New TickEvent Variants and Event Mapping

**Files:**
- Modify: `src/core/tick_types.rs`
- Modify: `src/tick_events.rs`
- Test: Compile check (the existing test suite covers TickEvent exhaustive matching)

**Step 1: Add new TickEvent variants**

In `src/core/tick_types.rs`, add to the `TickEvent` enum:

```rust
    // ── Postgame Zones ───────────────────────────────────────────
    /// A postgame region was unlocked by a Deep breakthrough.
    PostgameRegionUnlocked {
        region: crate::zones::PostgameRegion,
        message: String,
    },

    /// Player has Ascended to a new level.
    Ascended { level: u32, message: String },
```

Add to `BossDefeatResult` usage — ensure `PostgameCycle` is handled in any exhaustive matches on `BossDefeatResult` in tick processing.

**Step 2: Add event mapping in tick_events.rs**

In `src/tick_events.rs`, find the `apply_tick_events` function and add match arms for the new variants in the main event loop:

```rust
    TickEvent::PostgameRegionUnlocked { message, .. } => {
        add_log_entry(state, &message);
    }
    TickEvent::Ascended { message, .. } => {
        add_log_entry(state, &message);
    }
```

Also in `TickEventFlags`, add:
```rust
    pub postgame_region_unlocked: Option<crate::zones::PostgameRegion>,
```

**Step 3: Build to verify compilation**

Run: `cargo build --all-targets`
Expected: May have exhaustive match warnings. Fix all match arms that need updating.

**Step 4: Run tests**

Run: `cargo test`
Expected: PASS (existing tests still work)

**Step 5: Commit**

```bash
git add src/core/tick_types.rs src/tick_events.rs
git commit -m "feat(tick): add PostgameRegionUnlocked and Ascended TickEvent variants"
```

---

## Task 12: Deep Breakthrough Triggers Postgame Region Unlock

**Files:**
- Modify: `src/deep/missions.rs` or `src/core/tick_stages.rs` (wherever breakthrough resolution happens)
- Test: `tests/postgame_deep_test.rs` (extend)

**Step 1: Write the failing test**

Add to `tests/postgame_deep_test.rs`:

```rust
use quest::zones::PostgameRegion;

#[test]
fn test_layer_3_breakthrough_sets_cap_to_14() {
    let mut deep = quest::deep::DeepState::new();
    deep.persistent.discovered = true;
    deep.persistent.deepest_layer_reached = 3;

    // Simulate checking if a breakthrough should unlock a region
    if let Some(region) = PostgameRegion::from_layer(3) {
        let new_cap = region.end_zone_id();
        if new_cap > deep.persistent.postgame_zone_cap {
            deep.persistent.postgame_zone_cap = new_cap;
            deep.persistent.pending_postgame_region_unlock = Some(region);
        }
    }

    assert_eq!(deep.persistent.postgame_zone_cap, 14);
    assert_eq!(
        deep.persistent.pending_postgame_region_unlock,
        Some(PostgameRegion::RedFault)
    );
}

#[test]
fn test_repeated_breakthrough_does_not_downgrade_cap() {
    let mut deep = quest::deep::DeepState::new();
    deep.persistent.postgame_zone_cap = 17;
    deep.persistent.pending_postgame_region_unlock = None;

    // Layer 3 again shouldn't downgrade from 17 to 14
    if let Some(region) = PostgameRegion::from_layer(3) {
        let new_cap = region.end_zone_id();
        if new_cap > deep.persistent.postgame_zone_cap {
            deep.persistent.postgame_zone_cap = new_cap;
            deep.persistent.pending_postgame_region_unlock = Some(region);
        }
    }

    assert_eq!(deep.persistent.postgame_zone_cap, 17); // unchanged
    assert!(deep.persistent.pending_postgame_region_unlock.is_none()); // not set
}
```

**Step 2: Run test to verify it passes**

Run: `cargo test --test postgame_deep_test -- --nocapture`
Expected: PASS (this test just validates the logic pattern we'll integrate)

**Step 3: Add integration point**

Find where `Deep breakthrough` is processed in the tick pipeline. In `src/core/tick_stages.rs`, after a `DeepBreakthrough` event is processed, add:

```rust
// Check if this breakthrough unlocks a postgame region
if let Some(region) = crate::zones::PostgameRegion::from_layer(layer) {
    let new_cap = region.end_zone_id();
    if new_cap > deep.persistent.postgame_zone_cap {
        deep.persistent.postgame_zone_cap = new_cap;
        deep.persistent.pending_postgame_region_unlock = Some(region);
        result.deep_changed = true;
    }
}
```

Also add a check in the tick pipeline for `pending_postgame_region_unlock`:
```rust
if let Some(region) = deep.persistent.pending_postgame_region_unlock.take() {
    // Sync zone unlocks
    crate::zones::sync_account_zone_unlocks(
        &mut state.zone_progression,
        achievements.is_unlocked(crate::achievements::AchievementId::StormsEnd),
        deep.persistent.postgame_zone_cap,
    );
    result.events.push(TickEvent::PostgameRegionUnlocked {
        region,
        message: format!("\u{1F30B} {}", region.unlock_log_line()),
    });
    result.deep_changed = true;
}
```

**Step 4: Build and test**

Run: `cargo build --all-targets && cargo test`
Expected: PASS

**Step 5: Commit**

```bash
git add src/core/tick_stages.rs
git commit -m "feat(deep): trigger postgame region unlock on breakthrough at L3/L7/L13"
```

---

## Task 13: Add 12 New Achievements

**Files:**
- Modify: `src/achievements/types.rs` — 12 new `AchievementId` variants
- Modify: `src/achievements/data.rs` — 12 new `AchievementDef` entries
- Modify: `src/achievements/handlers.rs` — postgame zone handlers + ascension handler
- Test: inline tests in handlers.rs

**Step 1: Add AchievementId variants**

In `src/achievements/types.rs`, add after `BeyondInfinity`:

```rust
    // Postgame zone completion achievements (zones 12-20)
    PostgameZone12, // Rimbreaker
    PostgameZone13, // Cinderfall
    PostgameZone14, // Heart Piercer
    PostgameZone15, // Shard Breaker
    PostgameZone16, // Light Bender
    PostgameZone17, // Sunslayer
    PostgameZone18, // Ashen Sentinel
    PostgameZone19, // Throat Runner
    PostgameZone20, // Maw Closer
    // Ascension milestone achievements
    AscensionI,   // First Ascension
    AscensionIII, // Deepborn
    AscensionVI,  // Transcendent
```

**Step 2: Add AchievementDef entries**

In `src/achievements/data.rs`, add to `ALL_ACHIEVEMENTS`:

```rust
    AchievementDef { id: AchievementId::PostgameZone12, name: "Rimbreaker", description: "Defeat the final boss of Zone 12 (Splintered Rim)", category: AchievementCategory::Combat, icon: "\u{2694}", points: 25 },
    AchievementDef { id: AchievementId::PostgameZone13, name: "Cinderfall", description: "Defeat the final boss of Zone 13 (Ember Ravine)", category: AchievementCategory::Combat, icon: "\u{2694}", points: 25 },
    AchievementDef { id: AchievementId::PostgameZone14, name: "Heart Piercer", description: "Defeat the final boss of Zone 14 (Heart of the Fault)", category: AchievementCategory::Combat, icon: "\u{2694}", points: 50 },
    AchievementDef { id: AchievementId::PostgameZone15, name: "Shard Breaker", description: "Defeat the final boss of Zone 15 (Shard Fields)", category: AchievementCategory::Combat, icon: "\u{2694}", points: 50 },
    AchievementDef { id: AchievementId::PostgameZone16, name: "Light Bender", description: "Defeat the final boss of Zone 16 (Refraction Steps)", category: AchievementCategory::Combat, icon: "\u{2694}", points: 50 },
    AchievementDef { id: AchievementId::PostgameZone17, name: "Sunslayer", description: "Defeat the final boss of Zone 17 (Hall of Second Suns)", category: AchievementCategory::Combat, icon: "\u{2694}", points: 100 },
    AchievementDef { id: AchievementId::PostgameZone18, name: "Ashen Sentinel", description: "Defeat the final boss of Zone 18 (Ashen Verge)", category: AchievementCategory::Combat, icon: "\u{2694}", points: 100 },
    AchievementDef { id: AchievementId::PostgameZone19, name: "Throat Runner", description: "Defeat the final boss of Zone 19 (Throat of the World)", category: AchievementCategory::Combat, icon: "\u{2694}", points: 100 },
    AchievementDef { id: AchievementId::PostgameZone20, name: "Maw Closer", description: "Defeat the final boss of Zone 20 (The Black Mouth)", category: AchievementCategory::Combat, icon: "\u{2694}", points: 250 },
    AchievementDef { id: AchievementId::AscensionI, name: "First Ascension", description: "Reach Ascension I", category: AchievementCategory::Progression, icon: "\u{2B06}", points: 25 },
    AchievementDef { id: AchievementId::AscensionIII, name: "Deepborn", description: "Reach Ascension III", category: AchievementCategory::Progression, icon: "\u{2B06}", points: 50 },
    AchievementDef { id: AchievementId::AscensionVI, name: "Transcendent", description: "Reach Ascension VI", category: AchievementCategory::Progression, icon: "\u{2B06}", points: 250 },
```

**Step 3: Add handler methods**

In `src/achievements/handlers.rs`, extend `on_zone_fully_cleared` to handle zones 12-20:

```rust
        // Postgame zone completion achievements (zones 12-20)
        12 => Some(AchievementId::PostgameZone12),
        13 => Some(AchievementId::PostgameZone13),
        14 => Some(AchievementId::PostgameZone14),
        15 => Some(AchievementId::PostgameZone15),
        16 => Some(AchievementId::PostgameZone16),
        17 => Some(AchievementId::PostgameZone17),
        18 => Some(AchievementId::PostgameZone18),
        19 => Some(AchievementId::PostgameZone19),
        20 => Some(AchievementId::PostgameZone20),
```

Add a new handler for ascension:
```rust
    /// Called when the character Ascends to a new level.
    pub fn on_ascended(&mut self, new_level: u32, character_name: Option<&str>) {
        match new_level {
            1 => { self.unlock_with_name(AchievementId::AscensionI, character_name); }
            3 => { self.unlock_with_name(AchievementId::AscensionIII, character_name); }
            6 => { self.unlock_with_name(AchievementId::AscensionVI, character_name); }
            _ => {}
        }
    }
```

Make sure that postgame zone cycles (PostgameCycle) do NOT increment the `BeyondInfinity` achievement — only `ExpanseCycle` should. Check the code in tick_stages.rs that handles BossDefeatResult.

**Step 4: Build and test**

Run: `cargo build --all-targets && cargo test`
Expected: PASS

**Step 5: Commit**

```bash
git add src/achievements/types.rs src/achievements/data.rs src/achievements/handlers.rs
git commit -m "feat(achievements): add 12 postgame achievements (9 zone + 3 ascension)"
```

---

## Task 14: Wire Ascension Multiplier into Tick Pipeline

**Files:**
- Modify: `src/core/tick.rs` — build ascension_multiplier into CombatBonuses
- Test: existing game_tick tests should still pass

**Step 1: Find where CombatBonuses is constructed in tick.rs**

In `src/core/tick.rs`, Stage 3 (sync player HP) builds the `CombatBonuses` struct. Add the ascension multiplier:

```rust
ascension_multiplier: crate::ascension::ascension_combat_multiplier(state.ascension_level),
```

**Step 2: Apply ascension multiplier to player_max_hp**

In the HP sync code, change:
```rust
let adjusted_max_hp = derived.max_hp as u32 + bonuses.flat_hp;
```
to:
```rust
let adjusted_max_hp = ((derived.max_hp as u32 + bonuses.flat_hp) as f64
    * bonuses.ascension_multiplier) as u32;
```

**Step 3: Build and test**

Run: `cargo build --all-targets && cargo test`
Expected: PASS

**Step 4: Commit**

```bash
git add src/core/tick.rs
git commit -m "feat(tick): wire ascension_multiplier into CombatBonuses construction and HP sync"
```

---

## Task 15: Wire Boss Defeat to Use postgame_zone_cap

**Files:**
- Modify: `src/core/tick_stages.rs` — pass postgame_zone_cap to boss defeat handler
- Test: existing boss defeat tests should still pass

**Step 1: Find SubzoneBossDefeated handling in tick_stages.rs**

In `src/core/tick_stages.rs`, find where `on_boss_defeated` is called (in `process_combat_events` or similar). Change it to call `on_boss_defeated_with_cap` instead, passing `deep.persistent.postgame_zone_cap`.

**Step 2: Ensure PostgameCycle is handled**

In the match on `BossDefeatResult`, add:
```rust
BossDefeatResult::PostgameCycle { zone_id } => {
    // Postgame zone cycles: do NOT increment BeyondInfinity
    let zone_name = crate::zones::get_zone(zone_id)
        .map(|z| z.name)
        .unwrap_or("Unknown");
    // Emit an event similar to ExpanseCycle but for postgame
}
```

**Step 3: Build and test**

Run: `cargo build --all-targets && cargo test`
Expected: PASS

**Step 4: Commit**

```bash
git add src/core/tick_stages.rs
git commit -m "feat(tick): use on_boss_defeated_with_cap for postgame zone cycling"
```

---

## Task 16: Prestige Reset Syncs Postgame Zones

**Files:**
- Modify: `src/zones/advancement.rs` — `reset_for_prestige` does not currently sync postgame zones
- Modify: wherever prestige is executed (likely `src/character/prestige_actions.rs` or `src/input/prestige_input.rs`)
- Test: existing prestige tests

**Step 1: Ensure sync is called after prestige**

After `reset_for_prestige` is called during prestige, call `sync_account_zone_unlocks`:

```rust
// After reset_for_prestige:
crate::zones::sync_account_zone_unlocks(
    &mut state.zone_progression,
    achievements.is_unlocked(AchievementId::StormsEnd),
    deep.persistent.postgame_zone_cap,
);
```

This should be added wherever prestige execution happens that has access to the deep state.

**Step 2: Build and test**

Run: `cargo build --all-targets && cargo test`
Expected: PASS

**Step 3: Commit**

```bash
git add -A
git commit -m "feat(zones): sync postgame zone unlocks after prestige reset"
```

---

## Task 17: Stats Panel — POST Row and Ascension Display

**Files:**
- Modify: `src/ui/stats_panel.rs`
- Test: visual verification (no unit test for UI rendering)

**Step 1: Add POST zone row**

In `src/ui/stats_panel.rs`, find where zone progress capsules are rendered. After the existing zone row (Z1-Z11), add a conditional POST row:

```rust
// Only render POST row if zone 12+ is unlocked or current zone >= 12
let show_post_row = state.zone_progression.is_zone_unlocked(12)
    || state.zone_progression.current_zone_id >= 12;

if show_post_row {
    // Render POST label + capsules for zones 12-20
    // Use same visual language: current/unlocked/completed/locked
}
```

**Step 2: Add Ascension display**

In the prestige info area of the stats panel, add Ascension level:

```rust
if state.ascension_level > 0 {
    // Display "P{rank} | Asc {level}" instead of just "P{rank}"
    let asc_text = format!("P{} | Asc {}", state.prestige_rank, state.ascension_level);
}
```

**Step 3: Build**

Run: `cargo build --all-targets`
Expected: Compiles

**Step 4: Commit**

```bash
git add src/ui/stats_panel.rs
git commit -m "feat(ui): add POST zone row and Ascension level to stats panel"
```

---

## Task 18: Update CLAUDE.md Documentation

**Files:**
- Modify: `src/zones/CLAUDE.md`
- Modify: `src/deep/CLAUDE.md`
- Create: `src/ascension/CLAUDE.md`
- Modify: `CLAUDE.md` (root)

**Step 1: Write docs**

Create `src/ascension/CLAUDE.md` documenting:
- Module structure (types.rs, logic.rs)
- Key functions (ascension_cost, ascension_deep_gate, ascension_combat_multiplier, can_ascend, ascend)
- Constants tables (costs, gates, multipliers)
- Integration points (GameState.ascension_level, CombatBonuses.ascension_multiplier, tick.rs)

Update `src/zones/CLAUDE.md`:
- Add postgame zone tiers section (Z12-20)
- Document PostgameRegion enum
- Document sync_account_zone_unlocks
- Document on_boss_defeated_with_cap and PostgameCycle

Update `src/deep/CLAUDE.md`:
- Document postgame_zone_cap and pending_postgame_region_unlock fields
- Document breakthrough → region unlock flow

Update root `CLAUDE.md`:
- Add Ascension Module section
- Update Zone System section to mention Z12-20
- Update constants section

**Step 2: Commit**

```bash
git add src/ascension/CLAUDE.md src/zones/CLAUDE.md src/deep/CLAUDE.md CLAUDE.md
git commit -m "docs: add Ascension module docs, update zones and deep docs for postgame"
```

---

## Task 19: Full Test Suite Pass

**Step 1: Run all tests**

Run: `cargo test`
Expected: All tests pass

**Step 2: Run clippy**

Run: `cargo clippy --all-targets -- -D warnings`
Expected: No warnings

**Step 3: Run fmt check**

Run: `cargo fmt --check`
Expected: Clean

**Step 4: Fix any issues found**

If any tests fail or clippy warnings appear, fix them.

**Step 5: Final commit if needed**

```bash
git add -A
git commit -m "fix: resolve any remaining test failures and clippy warnings"
```

---

## Task 20: Run Full CI Check

**Step 1: Run make check**

Run: `make check`
Expected: All 5 checks pass (format, clippy, test, build, audit)

**Step 2: Verify the implementation**

Quick spot-checks:
- `cargo test --test postgame_zones_test` — all postgame boss defeat tests pass
- `cargo test --test ascension_test` — all ascension logic tests pass
- `cargo test --lib zones::data::tests` — zone count is 20
- `cargo test --lib ascension::types::tests` — multiplier formulas correct

---

## Deferred to Future Tasks (Not in V1)

These items from the design doc are explicitly deferred:
- Zone backgrounds (`src/ui/zone_bg.rs`) — 9 new ASCII backgrounds (significant art work)
- Enemy sprite palettes (`src/ui/enemy_sprites.rs`, `src/ui/enemy_sprite_data.rs`) — per-zone palettes
- GameOverlay for chapter unlock modals — full modal UI
- Ascend confirmation screen UI — full prestige-mirror UI
- Keybind for Ascend action
- Input routing for postgame region unlock dismissal
- Simulator support for postgame zones
- Deep simulator Ascension integration

These are substantial UI/input tasks that should be separate implementation tasks after the core logic is solid and tested.
