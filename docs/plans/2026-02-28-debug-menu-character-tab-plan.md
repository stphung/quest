# Debug Menu Character Tab — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a "Character" tab to the debug menu with fast travel (11 zones), prestige increments (+1/+5/+10), and level increments (+10/+50).

**Architecture:** Extend the existing `DebugAction` enum with parameterized variants (`TravelToZone(u32)`, `GrantPrestige(u32)`, `GrantLevels(u32)`) and add a `DebugCategory::Character` category. Single file change — the UI renders dynamically from category data.

**Tech Stack:** Rust, existing `debug_menu.rs` patterns

---

### Task 1: Add DebugAction variants and CHARACTER_ACTIONS array

**Files:**
- Modify: `src/utils/debug_menu.rs`

**Step 1: Add imports**

At the top of `debug_menu.rs`, add the zones import after existing imports:

```rust
use crate::zones::data::get_all_zones;
```

**Step 2: Add new enum variants**

Add these three variants to the `DebugAction` enum, after `TriggerDeepCompleteActiveMissions`:

```rust
    TravelToZone(u32),
    GrantPrestige(u32),
    GrantLevels(u32),
```

**Step 3: Add CHARACTER_ACTIONS array**

Add after `DEEP_ACTIONS`:

```rust
const CHARACTER_ACTIONS: &[DebugAction] = &[
    DebugAction::TravelToZone(1),
    DebugAction::TravelToZone(2),
    DebugAction::TravelToZone(3),
    DebugAction::TravelToZone(4),
    DebugAction::TravelToZone(5),
    DebugAction::TravelToZone(6),
    DebugAction::TravelToZone(7),
    DebugAction::TravelToZone(8),
    DebugAction::TravelToZone(9),
    DebugAction::TravelToZone(10),
    DebugAction::TravelToZone(11),
    DebugAction::GrantPrestige(1),
    DebugAction::GrantPrestige(5),
    DebugAction::GrantPrestige(10),
    DebugAction::GrantLevels(10),
    DebugAction::GrantLevels(50),
];
```

**Step 4: Append to DEBUG_ACTIONS**

Add the 16 character actions to the end of `DEBUG_ACTIONS`:

```rust
    // Character actions (zone travel, prestige, levels)
    DebugAction::TravelToZone(1),
    DebugAction::TravelToZone(2),
    DebugAction::TravelToZone(3),
    DebugAction::TravelToZone(4),
    DebugAction::TravelToZone(5),
    DebugAction::TravelToZone(6),
    DebugAction::TravelToZone(7),
    DebugAction::TravelToZone(8),
    DebugAction::TravelToZone(9),
    DebugAction::TravelToZone(10),
    DebugAction::TravelToZone(11),
    DebugAction::GrantPrestige(1),
    DebugAction::GrantPrestige(5),
    DebugAction::GrantPrestige(10),
    DebugAction::GrantLevels(10),
    DebugAction::GrantLevels(50),
```

**Step 5: Verify it compiles**

Run: `cargo build 2>&1 | head -20`
Expected: Compile errors for missing match arms (expected — addressed in Task 2)

---

### Task 2: Wire up option_index, label, and run for new variants

**Files:**
- Modify: `src/utils/debug_menu.rs`

**Step 1: Add option_index arms**

In `DebugAction::option_index()`, add after the `TriggerDeepCompleteActiveMissions => 28` arm:

```rust
            Self::TravelToZone(zone_id) => 29 + zone_id as usize - 1,  // 29-39
            Self::GrantPrestige(amount) => match amount {
                1 => 40,
                5 => 41,
                _ => 42, // 10
            },
            Self::GrantLevels(amount) => if amount == 10 { 43 } else { 44 },
```

**Step 2: Add label arms**

In `DebugAction::label()`, add after `TriggerDeepCompleteActiveMissions`:

```rust
            Self::TravelToZone(zone_id) => match zone_id {
                1 => "Travel to Meadow (Zone 1)",
                2 => "Travel to Dark Forest (Zone 2)",
                3 => "Travel to Mountain Pass (Zone 3)",
                4 => "Travel to Ancient Ruins (Zone 4)",
                5 => "Travel to Volcanic Wastes (Zone 5)",
                6 => "Travel to Frozen Tundra (Zone 6)",
                7 => "Travel to Crystal Caverns (Zone 7)",
                8 => "Travel to Sunken Kingdom (Zone 8)",
                9 => "Travel to Floating Isles (Zone 9)",
                10 => "Travel to Storm Citadel (Zone 10)",
                11 => "Travel to The Expanse (Zone 11)",
                _ => "Travel to Unknown Zone",
            },
            Self::GrantPrestige(amount) => match amount {
                1 => "+1 Prestige Rank",
                5 => "+5 Prestige Ranks",
                _ => "+10 Prestige Ranks",
            },
            Self::GrantLevels(amount) => if amount == 10 {
                "+10 Levels"
            } else {
                "+50 Levels"
            },
```

**Step 3: Add run arms**

In `DebugAction::run()`, add after `TriggerDeepCompleteActiveMissions`:

```rust
            Self::TravelToZone(zone_id) => trigger_travel_to_zone(state, enhancement, zone_id),
            Self::GrantPrestige(amount) => trigger_grant_prestige(state, enhancement, amount),
            Self::GrantLevels(amount) => trigger_grant_levels(state, enhancement, amount),
```

**Step 4: Verify it compiles**

Run: `cargo build 2>&1 | head -20`
Expected: Compile errors for missing handler functions (expected — addressed in Task 3)

---

### Task 3: Add DebugCategory::Character and wire into category system

**Files:**
- Modify: `src/utils/debug_menu.rs`

**Step 1: Add category variant**

Add `Character` to `DebugCategory` enum, after `Deep` and before `Borders`:

```rust
pub enum DebugCategory {
    Challenges,
    World,
    Resources,
    Items,
    Deep,
    Character,
    Borders,
}
```

**Step 2: Add category label**

In `DebugCategory::label()`, add:

```rust
            Self::Character => "Character",
```

**Step 3: Add to DEBUG_CATEGORIES**

Insert `DebugCategory::Character` before `DebugCategory::Borders`:

```rust
pub const DEBUG_CATEGORIES: &[DebugCategory] = &[
    DebugCategory::Challenges,
    DebugCategory::World,
    DebugCategory::Resources,
    DebugCategory::Items,
    DebugCategory::Deep,
    DebugCategory::Character,
    DebugCategory::Borders,
];
```

**Step 4: Wire option_count_for_category**

In `option_count_for_category()`, add:

```rust
        DebugCategory::Character => CHARACTER_ACTIONS.len(),
```

**Step 5: Wire global_option_index_for_visible**

In `DebugMenu::global_option_index_for_visible()`, add:

```rust
            DebugCategory::Character => CHARACTER_ACTIONS[visible_index].option_index(),
```

**Step 6: Verify it compiles**

Run: `cargo build 2>&1 | head -20`
Expected: Still needs handler functions (Task 4)

---

### Task 4: Implement handler functions

**Files:**
- Modify: `src/utils/debug_menu.rs`

**Step 1: Add trigger_travel_to_zone**

Add after the existing handler functions (before the `#[cfg(test)]` block):

```rust
fn trigger_travel_to_zone(
    state: &mut GameState,
    enhancement: &EnhancementProgress,
    zone_id: u32,
) -> &'static str {
    let zones = get_all_zones();
    let zone = match zones.iter().find(|z| z.id == zone_id) {
        Some(z) => z,
        None => return "Invalid zone ID!",
    };

    // Clear active content
    state.active_dungeon = None;
    state.active_fishing = None;
    state.combat_state.current_enemy = None;

    // Auto-bump prestige if needed
    if state.prestige_rank < zone.prestige_requirement {
        state.prestige_rank = zone.prestige_requirement;
        state.recalculate_prestige_bonuses();
    }

    // Unlock the target zone (and all zones at or below its prestige tier)
    for z in &zones {
        if z.prestige_requirement <= state.prestige_rank {
            state.zone_progression.unlock_zone(z.id);
        }
    }

    // Travel to subzone 1
    state.zone_progression.current_zone_id = zone_id;
    state.zone_progression.current_subzone_id = 1;
    state.zone_progression.kills_in_subzone = 0;
    state.zone_progression.fighting_boss = false;

    // Recalculate stats
    state.recalculate_derived_stats(&enhancement.levels);

    match zone_id {
        1 => "Traveled to Meadow (Zone 1)",
        2 => "Traveled to Dark Forest (Zone 2)",
        3 => "Traveled to Mountain Pass (Zone 3, P5)",
        4 => "Traveled to Ancient Ruins (Zone 4, P5)",
        5 => "Traveled to Volcanic Wastes (Zone 5, P10)",
        6 => "Traveled to Frozen Tundra (Zone 6, P10)",
        7 => "Traveled to Crystal Caverns (Zone 7, P15)",
        8 => "Traveled to Sunken Kingdom (Zone 8, P15)",
        9 => "Traveled to Floating Isles (Zone 9, P20)",
        10 => "Traveled to Storm Citadel (Zone 10, P20)",
        11 => "Traveled to The Expanse (Zone 11)",
        _ => "Traveled to unknown zone",
    }
}
```

**Step 2: Add trigger_grant_prestige**

```rust
fn trigger_grant_prestige(
    state: &mut GameState,
    enhancement: &EnhancementProgress,
    amount: u32,
) -> &'static str {
    state.prestige_rank += amount;
    state.recalculate_prestige_bonuses();

    // Unlock zones accessible at new prestige rank
    let zones = get_all_zones();
    for z in &zones {
        if z.prestige_requirement <= state.prestige_rank {
            state.zone_progression.unlock_zone(z.id);
        }
    }

    // Recalculate stats
    state.recalculate_derived_stats(&enhancement.levels);

    match amount {
        1 => "Granted +1 Prestige Rank!",
        5 => "Granted +5 Prestige Ranks!",
        _ => "Granted +10 Prestige Ranks!",
    }
}
```

**Step 3: Add trigger_grant_levels**

```rust
fn trigger_grant_levels(
    state: &mut GameState,
    enhancement: &EnhancementProgress,
    count: u32,
) -> &'static str {
    let mut rng = rand::rng();
    for _ in 0..count {
        state.character_level += 1;
        crate::core::xp::distribute_level_up_points(&mut rng, state);
    }
    state.character_xp = 0; // Reset partial XP to avoid confusion
    state.recalculate_derived_stats(&enhancement.levels);

    if count == 10 {
        "Granted +10 Levels!"
    } else {
        "Granted +50 Levels!"
    }
}
```

**Step 4: Verify it compiles and all existing tests pass**

Run: `cargo build 2>&1 | tail -5`
Expected: Successful build

Run: `cargo test --lib debug_menu 2>&1 | tail -10`
Expected: All existing tests pass

---

### Task 5: Fix existing tests affected by new category

**Files:**
- Modify: `src/utils/debug_menu.rs` (test section)

The existing `test_category_navigation_resets_selection` test navigates categories by count. Adding a new category shifts Borders from position 5 to position 6.

**Step 1: Run existing tests to identify failures**

Run: `cargo test --lib debug_menu -- --nocapture 2>&1 | tail -30`
Expected: Some tests may fail due to shifted category positions

**Step 2: Fix any affected tests**

Update category navigation tests to account for 7 categories instead of 6. The `test_border_preview_does_not_close_menu` test navigates to Borders using `navigate_next_category()` 5 times — this now needs 6 times since Character is between Deep and Borders.

**Step 3: Verify all existing tests pass**

Run: `cargo test --lib debug_menu 2>&1 | tail -10`
Expected: All tests pass

---

### Task 6: Add tests for new Character tab actions

**Files:**
- Modify: `src/utils/debug_menu.rs` (test section)

**Step 1: Add travel tests**

```rust
    #[test]
    fn test_trigger_travel_to_zone_basic() {
        let mut state = GameState::new("Test".to_string(), 0);
        let enhancement = EnhancementProgress::new();
        assert_eq!(state.zone_progression.current_zone_id, 1);

        let msg = trigger_travel_to_zone(&mut state, &enhancement, 5);
        assert_eq!(msg, "Traveled to Volcanic Wastes (Zone 5, P10)");
        assert_eq!(state.zone_progression.current_zone_id, 5);
        assert_eq!(state.zone_progression.current_subzone_id, 1);
        // Prestige auto-bumped to P10
        assert_eq!(state.prestige_rank, 10);
    }

    #[test]
    fn test_trigger_travel_clears_active_content() {
        let mut state = GameState::new("Test".to_string(), 0);
        let enhancement = EnhancementProgress::new();
        state.active_dungeon = Some(generate_dungeon(1, 0, 1));

        trigger_travel_to_zone(&mut state, &enhancement, 1);
        assert!(state.active_dungeon.is_none());
    }

    #[test]
    fn test_trigger_travel_no_prestige_downgrade() {
        let mut state = GameState::new("Test".to_string(), 0);
        let enhancement = EnhancementProgress::new();
        state.prestige_rank = 20;

        trigger_travel_to_zone(&mut state, &enhancement, 1);
        // Traveling to P0 zone should not lower prestige
        assert_eq!(state.prestige_rank, 20);
    }

    #[test]
    fn test_trigger_travel_unlocks_intermediate_zones() {
        let mut state = GameState::new("Test".to_string(), 0);
        let enhancement = EnhancementProgress::new();

        trigger_travel_to_zone(&mut state, &enhancement, 7);
        // Should unlock zones 1-8 (all P0, P5, P10, P15 zones)
        for zone_id in 1..=8 {
            assert!(
                state.zone_progression.is_zone_unlocked(zone_id),
                "Zone {zone_id} should be unlocked after traveling to Zone 7 (P15)"
            );
        }
    }
```

**Step 2: Add prestige tests**

```rust
    #[test]
    fn test_trigger_grant_prestige() {
        let mut state = GameState::new("Test".to_string(), 0);
        let enhancement = EnhancementProgress::new();

        let msg = trigger_grant_prestige(&mut state, &enhancement, 5);
        assert_eq!(msg, "Granted +5 Prestige Ranks!");
        assert_eq!(state.prestige_rank, 5);

        // Zones 3-4 (P5) should now be unlocked
        assert!(state.zone_progression.is_zone_unlocked(3));
        assert!(state.zone_progression.is_zone_unlocked(4));
    }

    #[test]
    fn test_trigger_grant_prestige_stacks() {
        let mut state = GameState::new("Test".to_string(), 0);
        let enhancement = EnhancementProgress::new();

        trigger_grant_prestige(&mut state, &enhancement, 5);
        trigger_grant_prestige(&mut state, &enhancement, 10);
        assert_eq!(state.prestige_rank, 15);
    }
```

**Step 3: Add level tests**

```rust
    #[test]
    fn test_trigger_grant_levels() {
        let mut state = GameState::new("Test".to_string(), 0);
        let enhancement = EnhancementProgress::new();
        assert_eq!(state.character_level, 1);

        let msg = trigger_grant_levels(&mut state, &enhancement, 10);
        assert_eq!(msg, "Granted +10 Levels!");
        assert_eq!(state.character_level, 11);
    }

    #[test]
    fn test_trigger_grant_levels_distributes_attributes() {
        let mut state = GameState::new("Test".to_string(), 0);
        let enhancement = EnhancementProgress::new();
        let initial_sum: u32 = crate::character::attributes::AttributeType::all()
            .iter()
            .map(|a| state.attributes.get(*a))
            .sum();

        trigger_grant_levels(&mut state, &enhancement, 10);

        let final_sum: u32 = crate::character::attributes::AttributeType::all()
            .iter()
            .map(|a| state.attributes.get(*a))
            .sum();
        // 10 levels * 3 points = 30 attribute points gained
        assert_eq!(final_sum, initial_sum + 30);
    }
```

**Step 4: Add Character category test**

```rust
    #[test]
    fn test_character_category_has_16_options() {
        assert_eq!(
            option_count_for_category(DebugCategory::Character),
            16
        );
    }
```

**Step 5: Run all tests**

Run: `cargo test --lib debug_menu 2>&1 | tail -15`
Expected: All tests pass

---

### Task 7: Run full CI checks and commit

**Files:**
- Verify: all files

**Step 1: Run clippy**

Run: `cargo clippy --all-targets -- -D warnings 2>&1 | tail -10`
Expected: No warnings

**Step 2: Run fmt check**

Run: `cargo fmt --check`
Expected: No formatting issues (run `cargo fmt` if needed)

**Step 3: Run full test suite**

Run: `cargo test 2>&1 | grep "test result" | tail -5`
Expected: All tests pass, 0 failures

**Step 4: Commit**

```bash
git add src/utils/debug_menu.rs
git commit -m "feat: add Character tab to debug menu with fast travel and level/prestige grants

Adds 16 new debug actions in a 'Character' category tab:
- Travel to any of 11 zones (auto-adjusts prestige to meet requirements)
- Prestige increments: +1, +5, +10
- Level increments: +10, +50"
```
