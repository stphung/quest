# Enhancement System (Blacksmith) Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add an equipment enhancement system (+1 to +10) accessed via a one-time Blacksmith discovery event with dramatic anvil animations.

**Architecture:** Standalone `src/enhancement/` module with account-wide persistence (`~/.quest/enhancement.json`). Follows the Haven pattern: one-time tick-based discovery, permanent `[B]` hotkey access, separate JSON persistence file. Blacksmith UI is a full-screen overlay with animated enhancement sequences.

**Tech Stack:** Rust, Ratatui 0.30 (Crossterm), Serde JSON, rand 0.10

**Design Doc:** `docs/plans/2026-02-14-enhancement-system-design.md`

---

### Task 1: Enhancement Data Types & Constants

Create the core data structures and constants for the enhancement system.

**Files:**
- Create: `src/enhancement/mod.rs`
- Create: `src/enhancement/types.rs`
- Test: `tests/enhancement_test.rs`

**Step 1: Create the module files**

Create `src/enhancement/mod.rs`:
```rust
pub mod types;

pub use types::*;
```

Create `src/enhancement/types.rs`:
```rust
use serde::{Deserialize, Serialize};

/// Account-wide enhancement progress, persisted to ~/.quest/enhancement.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancementProgress {
    pub discovered: bool,
    pub levels: [u8; 7], // Per-slot, 0-10, indexed by EquipmentSlot order
    pub total_attempts: u32,
    pub total_successes: u32,
    pub total_failures: u32,
    pub highest_level_reached: u8,
}

impl Default for EnhancementProgress {
    fn default() -> Self {
        Self::new()
    }
}

impl EnhancementProgress {
    pub fn new() -> Self {
        Self {
            discovered: false,
            levels: [0; 7],
            total_attempts: 0,
            total_successes: 0,
            total_failures: 0,
            highest_level_reached: 0,
        }
    }

    /// Get enhancement level for a slot (by index 0-6)
    pub fn level(&self, slot_index: usize) -> u8 {
        self.levels.get(slot_index).copied().unwrap_or(0)
    }

    /// Set enhancement level for a slot
    pub fn set_level(&mut self, slot_index: usize, level: u8) {
        if let Some(l) = self.levels.get_mut(slot_index) {
            *l = level.min(MAX_ENHANCEMENT_LEVEL);
        }
        self.highest_level_reached = self.highest_level_reached.max(level);
    }
}

/// Maximum enhancement level
pub const MAX_ENHANCEMENT_LEVEL: u8 = 10;

/// Minimum prestige rank to discover the Blacksmith
pub const BLACKSMITH_MIN_PRESTIGE_RANK: u32 = 15;

/// Base discovery chance per tick (~2hr avg at P15)
pub const BLACKSMITH_DISCOVERY_BASE_CHANCE: f64 = 0.000014;

/// Additional discovery chance per prestige rank above minimum
pub const BLACKSMITH_DISCOVERY_RANK_BONUS: f64 = 0.000007;

/// Success rates for each enhancement level (index = target level - 1)
pub const ENHANCEMENT_SUCCESS_RATES: [f64; 10] = [
    1.00, // +1: 100%
    1.00, // +2: 100%
    1.00, // +3: 100%
    1.00, // +4: 100%
    0.70, // +5: 70%
    0.60, // +6: 60%
    0.50, // +7: 50%
    0.30, // +8: 30%
    0.15, // +9: 15%
    0.05, // +10: 5%
];

/// Prestige rank cost for each enhancement level (index = target level - 1)
pub const ENHANCEMENT_COSTS: [u32; 10] = [
    1,  // +1: 1 PR
    1,  // +2: 1 PR
    1,  // +3: 1 PR
    1,  // +4: 1 PR
    3,  // +5: 3 PR
    3,  // +6: 3 PR
    3,  // +7: 3 PR
    5,  // +8: 5 PR
    5,  // +9: 5 PR
    10, // +10: 10 PR
];

/// Failure penalty (level drop) for each enhancement level (index = target level - 1)
/// 0 means no penalty (safe levels). Only applies on failure.
pub const ENHANCEMENT_FAIL_PENALTY: [u8; 10] = [
    0, // +1: safe
    0, // +2: safe
    0, // +3: safe
    0, // +4: safe
    1, // +5: -1
    1, // +6: -1
    1, // +7: -1
    2, // +8: -2
    2, // +9: -2
    2, // +10: -2
];

/// Cumulative stat bonus percentages (index = enhancement level, 0 = no bonus)
pub const ENHANCEMENT_CUMULATIVE_BONUS: [f64; 11] = [
    0.0,  // +0: no bonus
    1.0,  // +1: +1%
    2.0,  // +2: +2%
    4.0,  // +3: +4%
    6.0,  // +4: +6%
    9.0,  // +5: +9%
    13.0, // +6: +13%
    18.0, // +7: +18%
    25.0, // +8: +25%
    35.0, // +9: +35%
    50.0, // +10: +50%
];

/// Get the success rate for enhancing to the given target level
pub fn success_rate(target_level: u8) -> f64 {
    if target_level == 0 || target_level > MAX_ENHANCEMENT_LEVEL {
        return 0.0;
    }
    ENHANCEMENT_SUCCESS_RATES[(target_level - 1) as usize]
}

/// Get the prestige rank cost for enhancing to the given target level
pub fn enhancement_cost(target_level: u8) -> u32 {
    if target_level == 0 || target_level > MAX_ENHANCEMENT_LEVEL {
        return 0;
    }
    ENHANCEMENT_COSTS[(target_level - 1) as usize]
}

/// Get the failure penalty for failing at the given target level
pub fn fail_penalty(target_level: u8) -> u8 {
    if target_level == 0 || target_level > MAX_ENHANCEMENT_LEVEL {
        return 0;
    }
    ENHANCEMENT_FAIL_PENALTY[(target_level - 1) as usize]
}

/// Get the bonus multiplier for a given enhancement level (e.g., 1.09 for +5)
pub fn enhancement_multiplier(level: u8) -> f64 {
    let idx = (level as usize).min(MAX_ENHANCEMENT_LEVEL as usize);
    1.0 + ENHANCEMENT_CUMULATIVE_BONUS[idx] / 100.0
}
```

**Step 2: Register the module**

Modify `src/lib.rs` — add after `pub mod dungeon;`:
```rust
pub mod enhancement;
```

Add re-export after existing re-exports:
```rust
pub use enhancement::EnhancementProgress;
```

**Step 3: Write tests**

Create `tests/enhancement_test.rs`:
```rust
use quest::enhancement::*;

#[test]
fn test_enhancement_progress_new() {
    let ep = EnhancementProgress::new();
    assert!(!ep.discovered);
    assert_eq!(ep.levels, [0; 7]);
    assert_eq!(ep.total_attempts, 0);
    assert_eq!(ep.total_successes, 0);
    assert_eq!(ep.total_failures, 0);
    assert_eq!(ep.highest_level_reached, 0);
}

#[test]
fn test_enhancement_level_get_set() {
    let mut ep = EnhancementProgress::new();
    ep.set_level(0, 5); // Weapon to +5
    assert_eq!(ep.level(0), 5);
    assert_eq!(ep.highest_level_reached, 5);

    ep.set_level(0, 3); // Can go down (failure)
    assert_eq!(ep.level(0), 3);
    assert_eq!(ep.highest_level_reached, 5); // Highest doesn't decrease
}

#[test]
fn test_enhancement_level_clamped() {
    let mut ep = EnhancementProgress::new();
    ep.set_level(0, 15); // Over max
    assert_eq!(ep.level(0), MAX_ENHANCEMENT_LEVEL);
}

#[test]
fn test_enhancement_level_out_of_bounds() {
    let ep = EnhancementProgress::new();
    assert_eq!(ep.level(99), 0); // Out of bounds returns 0
}

#[test]
fn test_success_rates() {
    assert_eq!(success_rate(1), 1.0);
    assert_eq!(success_rate(4), 1.0);
    assert_eq!(success_rate(5), 0.70);
    assert_eq!(success_rate(8), 0.30);
    assert_eq!(success_rate(10), 0.05);
    assert_eq!(success_rate(0), 0.0);
    assert_eq!(success_rate(11), 0.0);
}

#[test]
fn test_enhancement_costs() {
    assert_eq!(enhancement_cost(1), 1);
    assert_eq!(enhancement_cost(5), 3);
    assert_eq!(enhancement_cost(8), 5);
    assert_eq!(enhancement_cost(10), 10);
    assert_eq!(enhancement_cost(0), 0);
}

#[test]
fn test_fail_penalties() {
    assert_eq!(fail_penalty(1), 0); // Safe
    assert_eq!(fail_penalty(4), 0); // Safe
    assert_eq!(fail_penalty(5), 1); // -1
    assert_eq!(fail_penalty(7), 1); // -1
    assert_eq!(fail_penalty(8), 2); // -2
    assert_eq!(fail_penalty(10), 2); // -2
}

#[test]
fn test_enhancement_multiplier() {
    assert!((enhancement_multiplier(0) - 1.0).abs() < f64::EPSILON);
    assert!((enhancement_multiplier(5) - 1.09).abs() < f64::EPSILON);
    assert!((enhancement_multiplier(10) - 1.50).abs() < f64::EPSILON);
}

#[test]
fn test_serialization_roundtrip() {
    let mut ep = EnhancementProgress::new();
    ep.discovered = true;
    ep.set_level(0, 7);
    ep.set_level(3, 4);
    ep.total_attempts = 50;
    ep.total_successes = 35;
    ep.total_failures = 15;

    let json = serde_json::to_string(&ep).unwrap();
    let deserialized: EnhancementProgress = serde_json::from_str(&json).unwrap();

    assert!(deserialized.discovered);
    assert_eq!(deserialized.level(0), 7);
    assert_eq!(deserialized.level(3), 4);
    assert_eq!(deserialized.total_attempts, 50);
    assert_eq!(deserialized.highest_level_reached, 7);
}
```

**Step 4: Run tests**

Run: `cargo test --test enhancement_test`
Expected: All pass

**Step 5: Commit**

```bash
git add src/enhancement/ src/lib.rs tests/enhancement_test.rs
git commit -m "feat(enhancement): add core data types and constants"
```

---

### Task 2: Enhancement Logic (Roll, Discovery)

Add the enhancement roll mechanic and Blacksmith discovery function.

**Files:**
- Create: `src/enhancement/logic.rs`
- Modify: `src/enhancement/mod.rs`
- Test: `tests/enhancement_test.rs` (append)

**Step 1: Create logic.rs**

```rust
use rand::Rng;

use super::types::*;

/// Attempt to enhance a slot. Returns true on success, false on failure.
/// Caller must verify prestige_rank >= cost and level < MAX before calling.
pub fn attempt_enhancement<R: Rng>(
    enhancement: &mut EnhancementProgress,
    slot_index: usize,
    rng: &mut R,
) -> bool {
    let current_level = enhancement.level(slot_index);
    if current_level >= MAX_ENHANCEMENT_LEVEL {
        return false;
    }

    let target_level = current_level + 1;
    let rate = success_rate(target_level);

    enhancement.total_attempts += 1;

    if rng.random::<f64>() < rate {
        // Success
        enhancement.set_level(slot_index, target_level);
        enhancement.total_successes += 1;
        true
    } else {
        // Failure — drop level by penalty
        let penalty = fail_penalty(target_level);
        let new_level = current_level.saturating_sub(penalty);
        enhancement.set_level(slot_index, new_level);
        enhancement.total_failures += 1;
        false
    }
}

/// Calculate Blacksmith discovery chance per tick
pub fn blacksmith_discovery_chance(prestige_rank: u32) -> f64 {
    if prestige_rank < BLACKSMITH_MIN_PRESTIGE_RANK {
        return 0.0;
    }
    BLACKSMITH_DISCOVERY_BASE_CHANCE
        + (prestige_rank - BLACKSMITH_MIN_PRESTIGE_RANK) as f64 * BLACKSMITH_DISCOVERY_RANK_BONUS
}

/// Try to discover the Blacksmith. Returns true if discovered this tick.
pub fn try_discover_blacksmith<R: Rng>(
    enhancement: &mut EnhancementProgress,
    prestige_rank: u32,
    rng: &mut R,
) -> bool {
    if enhancement.discovered {
        return false;
    }
    let chance = blacksmith_discovery_chance(prestige_rank);
    if chance <= 0.0 {
        return false;
    }
    if rng.random::<f64>() < chance {
        enhancement.discovered = true;
        return true;
    }
    false
}
```

**Step 2: Update mod.rs**

```rust
pub mod logic;
pub mod types;

pub use logic::*;
pub use types::*;
```

**Step 3: Add tests to enhancement_test.rs**

```rust
use quest::enhancement::logic::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[test]
fn test_attempt_enhancement_safe_levels() {
    let mut ep = EnhancementProgress::new();
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    // +1 through +4 are 100% success
    for expected in 1..=4 {
        let result = attempt_enhancement(&mut ep, 0, &mut rng);
        assert!(result);
        assert_eq!(ep.level(0), expected);
    }
    assert_eq!(ep.total_attempts, 4);
    assert_eq!(ep.total_successes, 4);
    assert_eq!(ep.total_failures, 0);
}

#[test]
fn test_attempt_enhancement_max_level_blocked() {
    let mut ep = EnhancementProgress::new();
    ep.set_level(0, MAX_ENHANCEMENT_LEVEL);
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    let result = attempt_enhancement(&mut ep, 0, &mut rng);
    assert!(!result);
    assert_eq!(ep.level(0), MAX_ENHANCEMENT_LEVEL);
    assert_eq!(ep.total_attempts, 0); // No attempt counted
}

#[test]
fn test_attempt_enhancement_failure_penalty_scaling() {
    let mut ep = EnhancementProgress::new();
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    // Set to +7, attempt +8 (30% success, -2 on fail)
    ep.set_level(0, 7);

    // Run many attempts to verify failure penalty
    let mut found_failure = false;
    for _ in 0..100 {
        let mut test_ep = ep.clone();
        let result = attempt_enhancement(&mut test_ep, 0, &mut rng);
        if !result {
            // Failed +8 should drop by 2 (to +5)
            assert_eq!(test_ep.level(0), 5);
            found_failure = true;
            break;
        }
    }
    assert!(found_failure, "Expected at least one failure in 100 attempts at 30%");
}

#[test]
fn test_attempt_enhancement_failure_penalty_small() {
    let mut ep = EnhancementProgress::new();
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    // Set to +4, attempt +5 (70% success, -1 on fail)
    ep.set_level(0, 4);

    let mut found_failure = false;
    for _ in 0..100 {
        let mut test_ep = ep.clone();
        let result = attempt_enhancement(&mut test_ep, 0, &mut rng);
        if !result {
            assert_eq!(test_ep.level(0), 3); // -1 penalty
            found_failure = true;
            break;
        }
    }
    assert!(found_failure, "Expected at least one failure in 100 attempts at 70%");
}

#[test]
fn test_blacksmith_discovery_chance() {
    assert_eq!(blacksmith_discovery_chance(0), 0.0);
    assert_eq!(blacksmith_discovery_chance(14), 0.0);
    assert!(blacksmith_discovery_chance(15) > 0.0);
    assert!(blacksmith_discovery_chance(20) > blacksmith_discovery_chance(15));
}

#[test]
fn test_try_discover_blacksmith() {
    let mut ep = EnhancementProgress::new();
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    // Too low prestige
    assert!(!try_discover_blacksmith(&mut ep, 10, &mut rng));
    assert!(!ep.discovered);

    // Force discovery with high prestige and enough attempts
    let mut discovered = false;
    for _ in 0..100_000 {
        if try_discover_blacksmith(&mut ep, 50, &mut rng) {
            discovered = true;
            break;
        }
    }
    assert!(discovered);
    assert!(ep.discovered);

    // Already discovered — always returns false
    assert!(!try_discover_blacksmith(&mut ep, 50, &mut rng));
}
```

**Step 4: Run tests**

Run: `cargo test --test enhancement_test`
Expected: All pass

**Step 5: Commit**

```bash
git add src/enhancement/logic.rs src/enhancement/mod.rs tests/enhancement_test.rs
git commit -m "feat(enhancement): add enhancement roll and discovery logic"
```

---

### Task 3: Persistence (Save/Load)

Add JSON save/load for enhancement progress.

**Files:**
- Create: `src/enhancement/persistence.rs`
- Modify: `src/enhancement/mod.rs`
- Test: `tests/enhancement_test.rs` (append)

**Step 1: Create persistence.rs**

Follow the exact pattern from `src/haven/logic.rs` lines 71-100:

```rust
use std::fs;
use std::io;
use std::path::PathBuf;

use super::types::EnhancementProgress;

pub fn enhancement_save_path() -> io::Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "Could not determine home directory")
    })?;
    Ok(home_dir.join(".quest").join("enhancement.json"))
}

pub fn load_enhancement() -> EnhancementProgress {
    let path = match enhancement_save_path() {
        Ok(p) => p,
        Err(_) => return EnhancementProgress::new(),
    };
    match fs::read_to_string(&path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => EnhancementProgress::new(),
    }
}

pub fn save_enhancement(enhancement: &EnhancementProgress) -> io::Result<()> {
    let path = enhancement_save_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(enhancement)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, json)?;
    Ok(())
}
```

**Step 2: Update mod.rs**

```rust
pub mod logic;
pub mod persistence;
pub mod types;

pub use logic::*;
pub use persistence::*;
pub use types::*;
```

**Step 3: Commit**

```bash
git add src/enhancement/persistence.rs src/enhancement/mod.rs
git commit -m "feat(enhancement): add JSON persistence"
```

---

### Task 4: Game Lifecycle Integration

Wire enhancement into the game loop: loading, saving, tick discovery, and TickEvent.

**Files:**
- Modify: `src/core/tick.rs` — add BlacksmithDiscovered TickEvent variant, add discovery stage
- Modify: `src/main.rs` — load/save enhancement, handle discovery event, pass to game_tick
- Modify: `src/lib.rs` — re-export

**Step 1: Add TickEvent variant**

In `src/core/tick.rs`, add to the `TickEvent` enum:
```rust
BlacksmithDiscovered,
```

Add to `TickResult`:
```rust
pub enhancement_changed: bool,
```

Initialize it to `false` in the TickResult construction.

**Step 2: Add discovery stage in game_tick()**

Update `game_tick()` signature to accept `enhancement: &mut EnhancementProgress`.

After Stage 10 (Haven discovery, ~line 753), add Stage 11 (renumber existing 11 to 12):

```rust
// ── 11. Blacksmith discovery check ────────────────────────────
if !enhancement.discovered
    && state.prestige_rank >= crate::enhancement::BLACKSMITH_MIN_PRESTIGE_RANK
    && state.active_dungeon.is_none()
    && state.active_fishing.is_none()
    && state.active_minigame.is_none()
    && crate::enhancement::try_discover_blacksmith(enhancement, state.prestige_rank, rng)
{
    achievements.on_blacksmith_discovered(Some(&state.character_name));
    result.events.push(TickEvent::BlacksmithDiscovered);
    result.enhancement_changed = true;
    if !debug_mode {
        result.achievements_changed = true;
    }
}
```

**Step 3: Update main.rs**

Near haven loading (~line 391), add:
```rust
let mut enhancement = enhancement::load_enhancement();
```

Pass `&mut enhancement` to `game_tick()` calls.

In the save paths (NeedsSaveAll, autosave, quit), add:
```rust
if enhancement.discovered {
    enhancement::save_enhancement(&enhancement).ok();
}
```

Handle `BlacksmithDiscovered` event — set a flag to show the discovery modal.

**Step 4: Update all game_tick() call sites**

Both in `main.rs` and `src/bin/simulator.rs`, update calls to include the `enhancement` parameter.

**Step 5: Run tests**

Run: `cargo test`
Expected: All pass (existing tests updated with new parameter)

**Step 6: Commit**

```bash
git add src/core/tick.rs src/main.rs src/lib.rs src/bin/simulator.rs
git commit -m "feat(enhancement): wire into game lifecycle and tick pipeline"
```

---

### Task 5: Combat Integration

Apply enhancement multipliers to derived stats calculation.

**Files:**
- Modify: `src/character/derived_stats.rs` — accept enhancement levels, multiply item contributions
- Modify: callers of `calculate_derived_stats()` to pass enhancement levels
- Test: `tests/enhancement_test.rs` (append)

**Step 1: Update calculate_derived_stats signature**

Add `enhancement_levels: &[u8; 7]` parameter. In the equipment iteration loop, apply the enhancement multiplier to each item's attribute contributions:

```rust
// In the attribute accumulation section:
for (idx, item) in equipment.iter_equipped_with_index() {
    let mult = crate::enhancement::enhancement_multiplier(enhancement_levels[idx]);
    let mut item_attrs = item.attributes.to_attributes();
    item_attrs.scale(mult);
    total_attrs.add(&item_attrs);
}
```

Note: `iter_equipped_with_index()` may need to be added to `Equipment` to yield `(slot_index, &Item)` pairs. If not, iterate by slot order.

Also apply the multiplier to affix values in the affix loop.

**Step 2: Update all callers**

Search for all calls to `calculate_derived_stats()` and pass the enhancement levels array. In places where enhancement is not available (e.g., tests), pass `&[0; 7]`.

**Step 3: Add tests**

```rust
#[test]
fn test_enhancement_multiplier_affects_derived_stats() {
    // Create a character with equipment
    // Calculate derived stats with [0;7] vs [5,0,0,0,0,0,0]
    // Verify weapon slot at +5 increases damage by ~9%
}
```

**Step 4: Run tests**

Run: `cargo test`
Expected: All pass

**Step 5: Commit**

```bash
git add src/character/derived_stats.rs src/items/equipment.rs
git commit -m "feat(enhancement): apply enhancement multipliers to derived stats"
```

---

### Task 6: Item Display Enhancement Prefix

Show enhancement level in item names throughout the UI.

**Files:**
- Modify: `src/ui/stats_panel.rs` — add +N prefix to item names
- Modify: `src/ui/info_panel.rs` — add +N prefix in loot log
- Test: Visual verification

**Step 1: Create a helper function**

Add to `src/enhancement/types.rs`:
```rust
/// Format an enhancement prefix for display (e.g., "+5 " or "" for +0)
pub fn enhancement_prefix(level: u8) -> String {
    if level == 0 {
        String::new()
    } else {
        format!("+{} ", level)
    }
}

/// Get the display color for an enhancement level
/// Returns a color tier: 0 = none, 1 = white (+1-4), 2 = yellow (+5-7), 3 = magenta (+8-9), 4 = gold (+10)
pub fn enhancement_color_tier(level: u8) -> u8 {
    match level {
        0 => 0,
        1..=4 => 1,
        5..=7 => 2,
        8..=9 => 3,
        _ => 4,
    }
}
```

**Step 2: Update stats_panel.rs**

In the equipment display section, prepend the enhancement prefix to item names. The enhancement levels array needs to be passed to the render function (from the `EnhancementProgress` loaded in main.rs).

For +5-7: prefix in Yellow. For +8-9: prefix in Magenta. For +10: prefix in Rgb(255,215,0) + BOLD.

**Step 3: Update info_panel.rs**

Same prefix treatment for items in the loot log.

**Step 4: Commit**

```bash
git add src/enhancement/types.rs src/ui/stats_panel.rs src/ui/info_panel.rs
git commit -m "feat(enhancement): show +N prefix on enhanced items"
```

---

### Task 7: Blacksmith UI State & Input Handling

Add the Blacksmith UI state, `[B]` hotkey, and input routing.

**Files:**
- Modify: `src/input.rs` — add BlacksmithUiState, [B] hotkey, handle_blacksmith()
- Modify: `src/main.rs` — pass blacksmith state to input handler, add to overlay rendering

**Step 1: Define BlacksmithUiState**

Add to `src/input.rs` (or a new file, but input.rs follows the HavenUiState pattern):

```rust
pub struct BlacksmithUiState {
    pub open: bool,
    pub selected_slot: usize,
    pub phase: BlacksmithPhase,
    pub animation_tick: u8,
    pub last_result: Option<EnhancementResult>,
}

pub enum BlacksmithPhase {
    Menu,
    Confirming,
    Hammering,
    ResultSuccess { old_level: u8, new_level: u8 },
    ResultFailure { old_level: u8, new_level: u8 },
}

pub struct EnhancementResult {
    pub slot_index: usize,
    pub success: bool,
    pub old_level: u8,
    pub new_level: u8,
}
```

**Step 2: Add [B] hotkey**

In `handle_base_game()`, add near the `[H]` haven handler:
```rust
KeyCode::Char('b') | KeyCode::Char('B') => {
    if enhancement.discovered {
        blacksmith_ui.open = true;
        blacksmith_ui.phase = BlacksmithPhase::Menu;
    }
    InputResult::Continue
}
```

**Step 3: Add handle_blacksmith()**

Create `handle_blacksmith()` function with:
- **Menu phase**: Up/Down to navigate slots, Enter to confirm, Esc to close
- **Confirming phase**: Enter to pay and start hammering, Esc to cancel
- **Hammering phase**: No input (animation plays via tick counter)
- **Result phases**: Any key returns to Menu

On Enter in Confirming:
1. Deduct prestige rank by `enhancement_cost(target_level)`
2. Call `attempt_enhancement()`
3. Transition to Hammering phase (animation_tick = 0)
4. Save enhancement progress

**Step 4: Add to input priority chain**

Insert blacksmith handling after haven (step 2) and before vault (step 3):
```rust
// 2.5. Blacksmith overlay
if blacksmith_ui.open {
    return handle_blacksmith(key, blacksmith_ui, enhancement, &mut state.prestige_rank);
}
```

**Step 5: Commit**

```bash
git add src/input.rs src/main.rs
git commit -m "feat(enhancement): add Blacksmith UI state and input handling"
```

---

### Task 8: Blacksmith Menu Scene

Create the Blacksmith menu overlay UI.

**Files:**
- Create: `src/ui/blacksmith_scene.rs`
- Modify: `src/ui/mod.rs` — add module, call from render pipeline

**Step 1: Create blacksmith_scene.rs**

Implement `render_blacksmith()` function with:
- Centered full-screen overlay (like Haven)
- Title: "⚒ THE BLACKSMITH" with prestige rank display
- 7 equipment slots listed with: emoji, name, current level → next level, success %, cost
- +10 slots show "MAX" in green
- Insufficient PR: cost in red
- Lifetime stats footer
- Navigation help bar

Follow the Haven scene pattern: `Clear` widget, bordered `Block`, inner layout with `Constraint`s.

**Step 2: Wire into mod.rs**

Add `pub mod blacksmith_scene;` and call `render_blacksmith()` from the main render dispatch when `blacksmith_ui.open` is true.

**Step 3: Commit**

```bash
git add src/ui/blacksmith_scene.rs src/ui/mod.rs
git commit -m "feat(enhancement): add Blacksmith menu scene"
```

---

### Task 9: Blacksmith Animations

Implement the anvil hammering, success sparkle, and failure shake animations.

**Files:**
- Modify: `src/ui/blacksmith_scene.rs` — add animation rendering functions
- Modify: `src/input.rs` or `src/main.rs` — advance animation_tick each game tick

**Step 1: Implement hammering animation**

`render_hammering()` function:
- 25 ticks total (~2.5 seconds)
- ASCII anvil centered in overlay
- Hammer position based on tick (rises, strikes at ticks 7-8, 15-16, 23-24)
- Spark characters (✦ ✧ * ·) in Yellow/Orange at random positions around anvil on strike ticks
- Item name + current level displayed on the anvil
- Use `tick % N` for frame cycling

**Step 2: Implement success result**

`render_success()` function:
- 20 ticks total (~2 seconds)
- "SUCCESS!" text pulsing Yellow ↔ Rgb(255,215,0) via `sin(tick * 0.5)`
- Sparkle border: ✦ ✧ * characters cycling positions each tick
- Item name in Green + BOLD with new level
- Bonus percentage displayed
- Border flashes Yellow on tick 0

**Step 3: Implement failure result**

`render_failure()` function:
- 15 ticks total (~1.5 seconds)
- "FAILED!" in Red + BOLD
- Text shakes: offset by ±1 char for ticks 0-5 using `tick % 2`
- ╳ crack characters around item
- Level drop shown: old level (DarkGray) → new level (Red)
- Border flashes Red on tick 0

**Step 4: Advance animation tick in game loop**

In `main.rs`, during each game tick, if `blacksmith_ui.phase` is Hammering/ResultSuccess/ResultFailure, increment `animation_tick`. When tick exceeds max for phase, transition:
- Hammering → ResultSuccess or ResultFailure (based on stored result)
- ResultSuccess/ResultFailure: wait for keypress (handled in input)

**Step 5: Commit**

```bash
git add src/ui/blacksmith_scene.rs src/main.rs
git commit -m "feat(enhancement): add anvil hammering and result animations"
```

---

### Task 10: Debug Menu & Discovery Modal

Add Blacksmith discovery trigger to debug menu and the discovery notification modal.

**Files:**
- Modify: `src/utils/debug_menu.rs` — add "Trigger Blacksmith Discovery" option
- Modify: `src/ui/blacksmith_scene.rs` — add discovery modal
- Modify: `src/main.rs` — handle BlacksmithDiscovered event, show modal

**Step 1: Add debug option**

In `DEBUG_OPTIONS` array, add:
```rust
"Trigger Blacksmith Discovery", // Index 12
```

In `trigger_selected()`, add match arm:
```rust
12 => trigger_blacksmith_discovery(enhancement),
```

Add function:
```rust
fn trigger_blacksmith_discovery(enhancement: &mut EnhancementProgress) -> &'static str {
    if enhancement.discovered {
        return "Blacksmith already discovered!";
    }
    enhancement.discovered = true;
    "Blacksmith discovered!"
}
```

Update `trigger_selected()` signature to accept `enhancement: &mut EnhancementProgress`.

**Step 2: Add discovery modal**

`render_blacksmith_discovery_modal()` in `blacksmith_scene.rs`:
- Centered modal (50×7)
- "⚒ Discovery!" title in Yellow
- "You found a wandering Blacksmith!" message
- "Press [B] to visit. [Enter] to continue"

**Step 3: Handle event in main.rs**

When `TickEvent::BlacksmithDiscovered` is received, set a flag to display the discovery modal. Modal dismissed by Enter key.

**Step 4: Commit**

```bash
git add src/utils/debug_menu.rs src/ui/blacksmith_scene.rs src/main.rs
git commit -m "feat(enhancement): add debug trigger and discovery modal"
```

---

### Task 11: Achievements

Add enhancement-related achievements.

**Files:**
- Modify: `src/achievements/types.rs` — add AchievementId variants, on_enhancement_* methods
- Modify: `src/achievements/data.rs` — register achievement definitions
- Test: `tests/enhancement_test.rs` (append)

**Step 1: Add AchievementId variants**

```rust
// Enhancement
ApprenticeSmith,      // Reach +1 on any slot
JourneymanSmith,      // Reach +5 on any slot
MasterSmith,          // Reach +10 on any slot
FullyEnhanced,        // Reach +10 on all 7 slots
PersistentHammering,  // 100 total enhancement attempts
BlacksmithDiscovered, // Discover the Blacksmith
```

**Step 2: Add on_* methods to Achievements**

```rust
pub fn on_blacksmith_discovered(&mut self, character_name: Option<&str>) {
    self.unlock_with_name(AchievementId::BlacksmithDiscovered, character_name);
}

pub fn on_enhancement_upgraded(&mut self, new_level: u8, all_levels: &[u8; 7], total_attempts: u32, character_name: Option<&str>) {
    // Milestone checks
    if new_level >= 1 { self.unlock_with_name(AchievementId::ApprenticeSmith, character_name); }
    if new_level >= 5 { self.unlock_with_name(AchievementId::JourneymanSmith, character_name); }
    if new_level >= 10 { self.unlock_with_name(AchievementId::MasterSmith, character_name); }
    if all_levels.iter().all(|&l| l >= 10) { self.unlock_with_name(AchievementId::FullyEnhanced, character_name); }
    if total_attempts >= 100 { self.unlock_with_name(AchievementId::PersistentHammering, character_name); }
}
```

**Step 3: Register in data.rs**

Add definitions with descriptions, icons, and categories (Progression).

**Step 4: Add tests**

```rust
#[test]
fn test_enhancement_achievements() {
    // Verify on_enhancement_upgraded triggers correct achievements at each threshold
}
```

**Step 5: Commit**

```bash
git add src/achievements/types.rs src/achievements/data.rs tests/enhancement_test.rs
git commit -m "feat(enhancement): add Blacksmith achievements"
```

---

### Task 12: Stats Tab Integration

Add enhancement data to the stats tab in the achievement browser.

**Files:**
- Modify: `src/ui/achievement_browser_scene.rs` — add enhancement sections to left and right columns

**Step 1: Add to left column**

In `build_stats_left_lines()`, after the DUNGEONS & CHALLENGES section, add:

```rust
// ── ENHANCEMENT ──────────────
lines.push(Line::from(Span::styled("── ENHANCEMENT ──", header_style)));
lines.push(stat_line("Attempts", &format_number(enhancement.total_attempts as u64), label_style, value_style, width));
lines.push(stat_line("Successes", &format_number(enhancement.total_successes as u64), label_style, value_style, width));
lines.push(stat_line("Failures", &format_number(enhancement.total_failures as u64), label_style, value_style, width));
lines.push(stat_line("Highest Level", &format!("+{}", enhancement.highest_level_reached), label_style, value_style, width));
```

**Step 2: Add to right column**

In `build_stats_right_lines()`, before the ACHIEVEMENTS summary section, add an ENHANCEMENT grid showing per-slot levels with color coding:

```
── ENHANCEMENT ──────────────
Weapon  +7  Armor   +5
Helmet  +3  Gloves  +4
Boots   +2  Amulet  +8
Ring    +1
```

Level colors: +0 DarkGray, +1-4 White, +5-7 Yellow, +8-9 Magenta, +10 Rgb(255,215,0).

**Step 3: Update function signatures**

Both functions need `enhancement: &EnhancementProgress` parameter. Update `render_stats_view()` to pass it through.

**Step 4: Commit**

```bash
git add src/ui/achievement_browser_scene.rs
git commit -m "feat(enhancement): add enhancement data to stats tab"
```

---

### Task 13: Character Select Blacksmith Access

Add `[B]` hotkey to the character select screen (like `[H]` for Haven).

**Files:**
- Modify: `src/ui/character_select.rs` — add [B] Blacksmith to controls
- Modify: `src/character/input.rs` — handle B key on character select

**Step 1: Add to controls display**

In `draw_controls()`, add Blacksmith button alongside Haven:
```rust
if enhancement.discovered {
    second_row_spans.push(Span::raw("  "));
    second_row_spans.push(Span::styled(
        "[B] Blacksmith",
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    ));
}
```

**Step 2: Handle input**

In character select input handler, add B key to open blacksmith overlay.

**Step 3: Commit**

```bash
git add src/ui/character_select.rs src/character/input.rs
git commit -m "feat(enhancement): add Blacksmith access from character select"
```

---

### Task 14: Final Integration Tests & Polish

Comprehensive integration tests and final cleanup.

**Files:**
- Test: `tests/enhancement_test.rs` (finalize)
- Modify: any files needing cleanup

**Step 1: Add integration tests**

```rust
#[test]
fn test_full_enhancement_flow() {
    // Create enhancement progress
    // Discover blacksmith
    // Enhance weapon from +0 to +4 (all safe)
    // Verify levels, costs deducted, stats updated
}

#[test]
fn test_enhancement_persistence_roundtrip() {
    // Create, modify, save, load, verify all fields preserved
}

#[test]
fn test_enhancement_display_prefix() {
    // Verify enhancement_prefix() and enhancement_color_tier()
}
```

**Step 2: Run full test suite**

Run: `cargo test`
Run: `make check` (full CI pipeline)
Expected: All pass

**Step 3: Commit**

```bash
git add tests/enhancement_test.rs
git commit -m "test(enhancement): add comprehensive integration tests"
```
