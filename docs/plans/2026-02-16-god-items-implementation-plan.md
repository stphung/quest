# God Items Implementation Plan (Phase 1 — Asprika)

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the god item framework and the first god item (Asprika — Armor of the Æsir) with Mythic rarity, a unique passive (30% DR), +100% offline XP, milestone tracking, and Storm Forge UI.

**Architecture:** God items are a new module (`src/god_items/`) following the same persistence pattern as achievements and enhancement. The `Item` struct gains an optional `god_item_id` field. Divine Bulwark integrates into the existing damage pipeline in `combat/logic.rs`. Offline XP bonus integrates into `main.rs` where `apply_offline_xp` is called.

**Tech Stack:** Rust, Serde (JSON persistence), Ratatui (UI)

**Design doc:** `docs/plans/2026-02-16-god-items-design.md`

---

### Task 1: Add Mythic Rarity Tier

**Files:**
- Modify: `src/items/types.rs:48-68` (Rarity enum + name())
- Modify: `src/items/scoring.rs:62-77` (auto_equip_if_better)
- Test: `src/items/types.rs` (inline tests)

**Step 1: Write the failing test**

In `src/items/types.rs`, add to the `tests` module:

```rust
#[test]
fn test_mythic_rarity_above_legendary() {
    assert!(Rarity::Legendary < Rarity::Mythic);
    assert!(Rarity::Mythic > Rarity::Epic);
}

#[test]
fn test_mythic_rarity_name() {
    assert_eq!(Rarity::Mythic.name(), "Mythic");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib items::types::tests::test_mythic_rarity_above_legendary -- --nocapture`
Expected: FAIL — no variant `Mythic` in `Rarity`

**Step 3: Implement Mythic variant**

In `src/items/types.rs`, add `Mythic = 5` after `Legendary = 4` in the Rarity enum. Add `Rarity::Mythic => "Mythic"` to the `name()` match.

**Step 4: Run tests to verify they pass**

Run: `cargo test --lib items::types -- --nocapture`
Expected: PASS (all existing + new tests)

**Step 5: Write auto-equip protection test**

In `src/items/scoring.rs` tests, add:

```rust
#[test]
fn test_mythic_item_never_auto_replaced() {
    let mut game_state = GameState::new("Test Hero".to_string(), Utc::now().timestamp());

    // Equip a Mythic item (weak stats)
    let mythic = Item {
        slot: EquipmentSlot::Armor,
        rarity: Rarity::Mythic,
        ilvl: 100,
        base_name: "Asprika".to_string(),
        display_name: "Asprika".to_string(),
        attributes: AttributeBonuses { con: 1, ..AttributeBonuses::new() },
        affixes: vec![],
    };
    game_state.equipment.set(EquipmentSlot::Armor, Some(mythic));

    // Try to equip a Legendary with higher raw score
    let legendary = Item {
        slot: EquipmentSlot::Armor,
        rarity: Rarity::Legendary,
        ilvl: 100,
        base_name: "Test".to_string(),
        display_name: "Test".to_string(),
        attributes: AttributeBonuses { con: 50, str: 50, ..AttributeBonuses::new() },
        affixes: vec![
            Affix { affix_type: AffixType::DamagePercent, value: 100.0 },
        ],
    };
    let equipped = auto_equip_if_better(legendary, &mut game_state);
    assert!(!equipped, "Mythic item should never be auto-replaced by a Legendary");
    assert_eq!(
        game_state.equipment.get(EquipmentSlot::Armor).as_ref().unwrap().rarity,
        Rarity::Mythic,
    );
}
```

**Step 6: Implement auto-equip protection**

In `src/items/scoring.rs` `auto_equip_if_better()`, add a guard at the top of the score comparison:

```rust
// Never auto-replace a Mythic (god) item
if let Some(current) = game_state.equipment.get(item.slot).as_ref() {
    if current.rarity == Rarity::Mythic && item.rarity != Rarity::Mythic {
        return false;
    }
}
```

**Step 7: Run all tests**

Run: `cargo test --lib items -- --nocapture`
Expected: ALL PASS

**Step 8: Commit**

```bash
git add src/items/types.rs src/items/scoring.rs
git commit -m "feat: add Mythic rarity tier above Legendary with auto-equip protection"
```

---

### Task 2: Create God Item Data Model

**Files:**
- Create: `src/god_items/mod.rs`
- Create: `src/god_items/types.rs`
- Modify: `src/lib.rs` (add module)
- Modify: `src/items/types.rs` (add `god_item_id` field to `Item`)
- Test: `src/god_items/types.rs` (inline tests)

**Step 1: Write the types module with tests**

Create `src/god_items/types.rs`:

```rust
use serde::{Deserialize, Serialize};
use crate::items::types::{
    Affix, AffixType, AttributeBonuses, EquipmentSlot, Item, Rarity,
};

/// Unique identifier for each god item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GodItemId {
    Asprika,
}

/// Passive ability unique to a god item.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GodItemPassive {
    /// Reduces all incoming damage by the given percentage (applied after defense).
    DivineBulwark { damage_reduction_percent: f64 },
}

/// Non-combat bonus unique to a god item.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GodItemBonus {
    /// Multiplies offline XP rate (1.0 = +100%).
    OfflineXpMultiplier { multiplier: f64 },
}

/// Static definition of a god item.
pub struct GodItemDefinition {
    pub id: GodItemId,
    pub name: &'static str,
    pub title: &'static str,
    pub slot: EquipmentSlot,
    pub attributes: AttributeBonuses,
    pub affixes: Vec<Affix>,
    pub passive: GodItemPassive,
    pub bonus: GodItemBonus,
}

impl GodItemDefinition {
    /// Creates the Item struct for this god item definition.
    pub fn to_item(&self) -> Item {
        Item {
            slot: self.slot,
            rarity: Rarity::Mythic,
            ilvl: 100,
            base_name: self.name.to_string(),
            display_name: self.name.to_string(),
            attributes: self.attributes.clone(),
            affixes: self.affixes.clone(),
            god_item_id: Some(self.id),
        }
    }
}

/// Returns the definition for Asprika.
pub fn asprika_definition() -> GodItemDefinition {
    GodItemDefinition {
        id: GodItemId::Asprika,
        name: "Asprika",
        title: "Armor of the Æsir",
        slot: EquipmentSlot::Armor,
        attributes: AttributeBonuses {
            str: 0,
            dex: 0,
            con: 40,  // Primary
            int: 0,
            wis: 20,  // Supporting
            cha: 0,
        },
        affixes: vec![
            Affix { affix_type: AffixType::DamageReduction, value: 20.0 },
            Affix { affix_type: AffixType::HPBonus, value: 200.0 },
            Affix { affix_type: AffixType::HPRegen, value: 30.0 },
            Affix { affix_type: AffixType::DamageReflection, value: 15.0 },
        ],
        passive: GodItemPassive::DivineBulwark { damage_reduction_percent: 30.0 },
        bonus: GodItemBonus::OfflineXpMultiplier { multiplier: 1.0 },
    }
}

/// Look up a god item definition by ID.
pub fn get_god_item_definition(id: GodItemId) -> GodItemDefinition {
    match id {
        GodItemId::Asprika => asprika_definition(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asprika_definition_is_mythic_armor() {
        let def = asprika_definition();
        assert_eq!(def.id, GodItemId::Asprika);
        assert_eq!(def.slot, EquipmentSlot::Armor);
        assert_eq!(def.name, "Asprika");
    }

    #[test]
    fn test_asprika_has_divine_bulwark_passive() {
        let def = asprika_definition();
        match def.passive {
            GodItemPassive::DivineBulwark { damage_reduction_percent } => {
                assert!((damage_reduction_percent - 30.0).abs() < f64::EPSILON);
            }
        }
    }

    #[test]
    fn test_asprika_has_offline_xp_bonus() {
        let def = asprika_definition();
        match def.bonus {
            GodItemBonus::OfflineXpMultiplier { multiplier } => {
                assert!((multiplier - 1.0).abs() < f64::EPSILON);
            }
        }
    }

    #[test]
    fn test_asprika_has_con_primary_wis_supporting() {
        let def = asprika_definition();
        assert!(def.attributes.con > 0, "CON should be primary");
        assert!(def.attributes.wis > 0, "WIS should be supporting");
        assert_eq!(def.attributes.str, 0);
        assert_eq!(def.attributes.dex, 0);
        assert_eq!(def.attributes.int, 0);
        assert_eq!(def.attributes.cha, 0);
    }

    #[test]
    fn test_to_item_creates_mythic_item() {
        let def = asprika_definition();
        let item = def.to_item();
        assert_eq!(item.rarity, Rarity::Mythic);
        assert_eq!(item.slot, EquipmentSlot::Armor);
        assert_eq!(item.display_name, "Asprika");
        assert_eq!(item.god_item_id, Some(GodItemId::Asprika));
    }

    #[test]
    fn test_get_god_item_definition() {
        let def = get_god_item_definition(GodItemId::Asprika);
        assert_eq!(def.id, GodItemId::Asprika);
    }
}
```

**Step 2: Create the mod.rs**

Create `src/god_items/mod.rs`:

```rust
pub mod types;
pub use types::*;
```

**Step 3: Add `god_item_id` field to Item**

In `src/items/types.rs`, add to the `Item` struct:

```rust
#[serde(default)]
pub god_item_id: Option<crate::god_items::GodItemId>,
```

**Step 4: Register module in lib.rs**

In `src/lib.rs`, add `pub mod god_items;` after `pub mod fishing;`.

**Step 5: Run tests**

Run: `cargo test --lib god_items -- --nocapture`
Expected: ALL PASS

Run: `cargo test --lib items -- --nocapture`
Expected: ALL PASS (existing tests should pass with `god_item_id` defaulting to `None`)

**Step 6: Commit**

```bash
git add src/god_items/ src/items/types.rs src/lib.rs
git commit -m "feat: add god item data model with Asprika definition"
```

---

### Task 3: Add God Item Progress Persistence

**Files:**
- Create: `src/god_items/persistence.rs`
- Modify: `src/god_items/mod.rs`
- Modify: `src/god_items/types.rs` (add progress types)
- Test: `src/god_items/types.rs`, `src/god_items/persistence.rs` (inline tests)

**Step 1: Add progress types to types.rs**

Add to `src/god_items/types.rs`:

```rust
/// State machine for a god item quest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GodItemState {
    /// Not yet discovered (Temple Trial not completed).
    Undiscovered,
    /// Discovered — quest requirements visible, working on milestones.
    Discovered,
    /// All milestones met, Temple Trial return completed, ready to forge.
    ReadyToForge,
    /// Forged and equipped.
    Forged,
}

impl Default for GodItemState {
    fn default() -> Self {
        Self::Undiscovered
    }
}

/// Progress tracking for Asprika's milestones.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsprikaMilestones {
    /// Number of distinct challenge types won at Master difficulty.
    pub master_challenge_types_won: Vec<String>,
    /// Number of equipment slots currently at +7 or higher.
    pub slots_at_plus_7: u8,
    /// Whether the Temple Trial return visit is complete.
    pub temple_trial_return_complete: bool,
}

impl Default for AsprikaMilestones {
    fn default() -> Self {
        Self {
            master_challenge_types_won: Vec::new(),
            temple_trial_return_complete: false,
            slots_at_plus_7: 0,
        }
    }
}

impl AsprikaMilestones {
    pub fn master_challenges_met(&self) -> bool {
        self.master_challenge_types_won.len() >= 3
    }

    pub fn slots_at_7_met(&self) -> bool {
        self.slots_at_plus_7 >= 3
    }

    pub fn all_met(&self) -> bool {
        self.master_challenges_met()
            && self.slots_at_7_met()
            && self.temple_trial_return_complete
    }
}

/// Account-wide god item progress.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GodItemProgress {
    pub asprika_state: GodItemState,
    pub asprika_milestones: AsprikaMilestones,
}

impl Default for GodItemProgress {
    fn default() -> Self {
        Self {
            asprika_state: GodItemState::Undiscovered,
            asprika_milestones: AsprikaMilestones::default(),
        }
    }
}
```

Add tests:

```rust
#[test]
fn test_asprika_milestones_default_not_met() {
    let m = AsprikaMilestones::default();
    assert!(!m.master_challenges_met());
    assert!(!m.slots_at_7_met());
    assert!(!m.all_met());
}

#[test]
fn test_asprika_milestones_all_met() {
    let m = AsprikaMilestones {
        master_challenge_types_won: vec![
            "Chess".to_string(),
            "Go".to_string(),
            "Snake".to_string(),
        ],
        slots_at_plus_7: 3,
        temple_trial_return_complete: true,
    };
    assert!(m.master_challenges_met());
    assert!(m.slots_at_7_met());
    assert!(m.all_met());
}

#[test]
fn test_god_item_state_default_undiscovered() {
    let p = GodItemProgress::default();
    assert_eq!(p.asprika_state, GodItemState::Undiscovered);
}
```

**Step 2: Create persistence module**

Create `src/god_items/persistence.rs` following the achievements persistence pattern:

```rust
use super::types::GodItemProgress;
use std::fs;
use std::io;
use std::path::PathBuf;

pub fn god_items_save_path() -> io::Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "Could not determine home directory")
    })?;
    Ok(home_dir.join(".quest").join("god_items.json"))
}

pub fn load_god_item_progress() -> GodItemProgress {
    let path = match god_items_save_path() {
        Ok(p) => p,
        Err(_) => return GodItemProgress::default(),
    };
    match fs::read_to_string(&path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => GodItemProgress::default(),
    }
}

pub fn save_god_item_progress(progress: &GodItemProgress) -> io::Result<()> {
    let path = god_items_save_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(progress)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_god_item_progress_serialization_roundtrip() {
        let progress = GodItemProgress::default();
        let json = serde_json::to_string_pretty(&progress).unwrap();
        let loaded: GodItemProgress = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.asprika_state, progress.asprika_state);
    }

    #[test]
    fn test_god_items_save_path() {
        let result = god_items_save_path();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("god_items.json"));
    }
}
```

**Step 3: Update mod.rs**

```rust
pub mod persistence;
pub mod types;
pub use persistence::*;
pub use types::*;
```

**Step 4: Run tests**

Run: `cargo test --lib god_items -- --nocapture`
Expected: ALL PASS

**Step 5: Commit**

```bash
git add src/god_items/
git commit -m "feat: add god item progress persistence (account-level, ~/.quest/god_items.json)"
```

---

### Task 4: Integrate Divine Bulwark into Combat Damage Pipeline

**Files:**
- Modify: `src/combat/logic.rs:318-342` (enemy damage calculation)
- Modify: `src/combat/logic.rs` (update_combat signature or pass god item info)
- Test: `src/combat/logic.rs` (inline tests)

**Step 1: Write the failing test**

In `src/combat/logic.rs` tests, add:

```rust
#[test]
fn test_divine_bulwark_reduces_enemy_damage() {
    let mut state = GameState::new("Test Hero".to_string(), 0);
    let mut achievements = Achievements::default();
    state.combat_state = CombatState::Fighting;
    state.combat_state.player_current_hp = 100;
    state.combat_state.player_max_hp = 100;
    state.combat_state.enemy_attack_timer = 100.0; // Force enemy attack
    state.combat_state.current_enemy = Some(Enemy::new("Test".to_string(), 100, 20));

    let haven = HavenCombatBonuses::default();
    let prestige = PrestigeCombatBonuses::default();

    // Without Divine Bulwark: damage = 20 - 0 defense = 20
    let events = update_combat(&mut state, 0.1, &haven, &prestige, &mut achievements, 0.0);
    let damage_taken = events.iter().find_map(|e| match e {
        CombatEvent::EnemyAttack { damage } => Some(*damage),
        _ => None,
    }).unwrap();
    assert_eq!(damage_taken, 20, "Without bulwark, full damage");

    // Reset for Divine Bulwark test
    state.combat_state.player_current_hp = 100;
    state.combat_state.enemy_attack_timer = 100.0;
    state.combat_state.current_enemy = Some(Enemy::new("Test".to_string(), 100, 20));

    // With 30% Divine Bulwark: damage = floor(20 * 0.70) = 14
    let events = update_combat(&mut state, 0.1, &haven, &prestige, &mut achievements, 30.0);
    let damage_taken = events.iter().find_map(|e| match e {
        CombatEvent::EnemyAttack { damage } => Some(*damage),
        _ => None,
    }).unwrap();
    assert_eq!(damage_taken, 14, "With 30% bulwark, damage should be 14");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib combat::logic::tests::test_divine_bulwark_reduces_enemy_damage -- --nocapture`
Expected: FAIL — `update_combat` doesn't accept the DR parameter yet

**Step 3: Add `god_item_dr_percent` parameter to `update_combat`**

Add a new `god_item_dr_percent: f64` parameter to `update_combat()`. In the enemy damage calculation (around line 324), apply it after defense subtraction:

```rust
let total_defense = derived.defense + prestige_bonuses.flat_defense;
let base_damage = enemy.damage.saturating_sub(total_defense).max(1);
// Apply Divine Bulwark (god item damage reduction)
let enemy_damage = if god_item_dr_percent > 0.0 {
    ((base_damage as f64) * (1.0 - god_item_dr_percent / 100.0)) as u32
} else {
    base_damage
}.max(1);
```

**Step 4: Update all callers of `update_combat`**

In `src/core/tick.rs`, pass `0.0` for now (we'll wire it up properly in Task 6):

```rust
let combat_events = update_combat(state, delta_time, &haven_combat, prestige_bonuses, achievements, 0.0);
```

**Step 5: Run tests**

Run: `cargo test --lib combat -- --nocapture`
Expected: ALL PASS (including new test)

**Step 6: Commit**

```bash
git add src/combat/logic.rs src/core/tick.rs
git commit -m "feat: integrate Divine Bulwark (god item DR) into combat damage pipeline"
```

---

### Task 5: Integrate Offline XP Bonus

**Files:**
- Modify: `src/core/offline.rs:30-50` (calculate_offline_xp signature)
- Modify: `src/main.rs` (apply_offline_xp passes god item bonus)
- Test: `src/core/offline.rs` (inline tests)

**Step 1: Write the failing test**

In `src/core/offline.rs` tests, add:

```rust
#[test]
fn test_offline_xp_with_god_item_bonus() {
    let base_xp = calculate_offline_xp(3600, 0, 0, 0, 0.0, 0.0);
    let god_item_xp = calculate_offline_xp(3600, 0, 0, 0, 0.0, 100.0);

    // +100% god item bonus should double XP
    let ratio = god_item_xp / base_xp;
    assert!(
        (ratio - 2.0).abs() < 0.01,
        "God item +100% offline XP should double base, got {:.3}x",
        ratio
    );
}

#[test]
fn test_offline_xp_god_item_stacks_with_haven() {
    let base_xp = calculate_offline_xp(3600, 0, 0, 0, 0.0, 0.0);
    // Haven +100% and god item +100% should compose: (1 + 1.0) * (1 + 1.0) = 4x
    let stacked_xp = calculate_offline_xp(3600, 0, 0, 0, 100.0, 100.0);

    let ratio = stacked_xp / base_xp;
    assert!(
        (ratio - 4.0).abs() < 0.01,
        "Haven + god item stacked should give 4x, got {:.3}x",
        ratio
    );
}
```

**Step 2: Run test to verify it fails**

Expected: FAIL — `calculate_offline_xp` doesn't accept the god item parameter yet

**Step 3: Add `god_item_offline_xp_percent` parameter**

In `src/core/offline.rs`, add a new parameter to `calculate_offline_xp`:

```rust
pub fn calculate_offline_xp(
    elapsed_seconds: i64,
    prestige_rank: u32,
    wis_modifier: i32,
    cha_modifier: i32,
    haven_offline_xp_percent: f64,
    god_item_offline_xp_percent: f64,
) -> f64 {
    // ... existing code ...
    // Apply Haven Hearthstone bonus
    let base_xp = estimated_kills * xp_per_kill;
    let haven_mult = 1.0 + haven_offline_xp_percent / 100.0;
    let god_item_mult = 1.0 + god_item_offline_xp_percent / 100.0;
    base_xp * haven_mult * god_item_mult
}
```

**Step 4: Update all callers**

- `process_offline_progression` in `offline.rs`: add `god_item_offline_xp_percent: f64` parameter, pass through
- `apply_offline_xp` in `main.rs`: calculate god item bonus from equipped armor, pass to `process_offline_progression`
- All existing tests: add `0.0` for the new parameter

**Step 5: Run tests**

Run: `cargo test --lib core::offline -- --nocapture`
Expected: ALL PASS

**Step 6: Commit**

```bash
git add src/core/offline.rs src/core/game_logic.rs src/main.rs
git commit -m "feat: integrate god item offline XP bonus into offline progression"
```

---

### Task 6: Wire God Item State into Game Loop

**Files:**
- Modify: `src/main.rs` (load/save GodItemProgress, compute DR for tick, compute offline XP bonus)
- Modify: `src/core/tick.rs` (pass god item DR to update_combat, add `god_items_changed` to TickResult)
- Test: integration test

**Step 1: Add `god_items_changed` to TickResult**

In `src/core/tick.rs`, add `pub god_items_changed: bool` to `TickResult`.

**Step 2: Load/save in main.rs**

In `main.rs`, following the pattern for achievements and enhancement:
- Load: `let mut god_item_progress = god_items::load_god_item_progress();`
- Save: when `tick_result.god_items_changed` is true, call `god_items::save_god_item_progress(&god_item_progress)`
- Also save on quit and prestige

**Step 3: Compute god item DR for combat**

In `main.rs` or `tick.rs`, before calling `update_combat`, check if the equipped Armor has `god_item_id == Some(GodItemId::Asprika)` and if so, look up the passive and pass `30.0` as the DR percent.

Helper function in `god_items/types.rs`:

```rust
/// Returns the god item damage reduction percent from equipped items, if any.
pub fn equipped_god_item_dr(equipment: &crate::items::Equipment) -> f64 {
    for item in equipment.iter_equipped() {
        if let Some(id) = item.god_item_id {
            let def = get_god_item_definition(id);
            if let GodItemPassive::DivineBulwark { damage_reduction_percent } = def.passive {
                return damage_reduction_percent;
            }
        }
    }
    0.0
}
```

**Step 4: Compute offline XP bonus from god items**

Helper function in `god_items/types.rs`:

```rust
/// Returns the god item offline XP bonus percent from equipped items, if any.
pub fn equipped_god_item_offline_xp_percent(equipment: &crate::items::Equipment) -> f64 {
    for item in equipment.iter_equipped() {
        if let Some(id) = item.god_item_id {
            let def = get_god_item_definition(id);
            if let GodItemBonus::OfflineXpMultiplier { multiplier } = def.bonus {
                return multiplier * 100.0;
            }
        }
    }
    0.0
}
```

**Step 5: Wire into tick.rs**

In `game_tick()`, compute the DR and pass it to `update_combat`:

```rust
let god_item_dr = crate::god_items::equipped_god_item_dr(&state.equipment);
let combat_events = update_combat(state, delta_time, &haven_combat, prestige_bonuses, achievements, god_item_dr);
```

**Step 6: Wire into main.rs offline XP**

In `apply_offline_xp`, compute the god item bonus:

```rust
let god_item_offline_bonus = god_items::equipped_god_item_offline_xp_percent(&state.equipment);
let report = process_offline_progression(state, haven_offline_bonus, god_item_offline_bonus);
```

**Step 7: Run all tests**

Run: `cargo test -- --nocapture`
Expected: ALL PASS

**Step 8: Commit**

```bash
git add src/main.rs src/core/tick.rs src/god_items/types.rs
git commit -m "feat: wire god item state into game loop (DR in combat, offline XP bonus)"
```

---

### Task 7: Milestone Tracking Integration

**Files:**
- Modify: `src/god_items/types.rs` (add milestone sync methods)
- Modify: `src/main.rs` (sync milestones on challenge win and enhancement)
- Test: `src/god_items/types.rs` (inline tests)

**Step 1: Add milestone sync methods**

In `src/god_items/types.rs`, add:

```rust
impl GodItemProgress {
    /// Sync the slots_at_plus_7 count from current enhancement levels.
    pub fn sync_enhancement_milestones(&mut self, enhancement_levels: &[u8; 7]) {
        self.asprika_milestones.slots_at_plus_7 =
            enhancement_levels.iter().filter(|&&l| l >= 7).count() as u8;
    }

    /// Record a Master difficulty challenge win for a given game type.
    /// Returns true if this was a new type (milestone progressed).
    pub fn record_master_challenge_win(&mut self, challenge_type: &str) -> bool {
        if self.asprika_state == GodItemState::Undiscovered {
            return false;
        }
        let type_str = challenge_type.to_string();
        if !self.asprika_milestones.master_challenge_types_won.contains(&type_str) {
            self.asprika_milestones.master_challenge_types_won.push(type_str);
            true
        } else {
            false
        }
    }
}
```

**Step 2: Write tests**

```rust
#[test]
fn test_sync_enhancement_milestones() {
    let mut progress = GodItemProgress::default();
    progress.asprika_state = GodItemState::Discovered;
    let levels = [0, 7, 3, 8, 7, 0, 0];
    progress.sync_enhancement_milestones(&levels);
    assert_eq!(progress.asprika_milestones.slots_at_plus_7, 3);
}

#[test]
fn test_record_master_challenge_win() {
    let mut progress = GodItemProgress::default();
    progress.asprika_state = GodItemState::Discovered;
    assert!(progress.record_master_challenge_win("Chess"));
    assert!(progress.record_master_challenge_win("Go"));
    assert!(!progress.record_master_challenge_win("Chess")); // duplicate
    assert_eq!(progress.asprika_milestones.master_challenge_types_won.len(), 2);
}

#[test]
fn test_record_master_challenge_win_ignored_when_undiscovered() {
    let mut progress = GodItemProgress::default();
    assert!(!progress.record_master_challenge_win("Chess"));
    assert!(progress.asprika_milestones.master_challenge_types_won.is_empty());
}
```

**Step 3: Integrate into main.rs**

After a challenge minigame win at Master difficulty, call `god_item_progress.record_master_challenge_win(type_name)`.

After each enhancement attempt, call `god_item_progress.sync_enhancement_milestones(&enhancement.levels)`.

**Step 4: Run tests**

Run: `cargo test --lib god_items -- --nocapture`
Expected: ALL PASS

**Step 5: Commit**

```bash
git add src/god_items/types.rs src/main.rs
git commit -m "feat: add milestone tracking for god item quest progress"
```

---

### Task 8: Storm Forge — Divine Blueprints UI

**Files:**
- Modify: `src/ui/soulforge_scene.rs` (add Divine Blueprints section)
- Modify: `src/main.rs` (pass god item progress to soulforge scene, handle forge input)
- Test: manual visual testing

**Step 1: Add Divine Blueprints rendering**

In the Soulforge scene, add a new section when god item progress shows `Discovered` or later states. Display:
- Asprika name and title
- Passive and bonus descriptions
- Milestone progress with checkmarks
- Forge button (enabled when all met + 50 PR available)

**Step 2: Handle forge input**

When user confirms forge:
1. Deduct 50 prestige ranks from `state.prestige_rank`
2. Create the Asprika item via `asprika_definition().to_item()`
3. Equip it via `state.equipment.set(EquipmentSlot::Armor, Some(item))`
4. Set `god_item_progress.asprika_state = GodItemState::Forged`
5. Show forge notification modal

**Step 3: Run `make check`**

Run: `make check`
Expected: ALL PASS

**Step 4: Commit**

```bash
git add src/ui/soulforge_scene.rs src/main.rs
git commit -m "feat: add Divine Blueprints UI to Storm Forge for god item forging"
```

---

### Task 9: Debug Menu Integration

**Files:**
- Modify: `src/utils/debug_menu.rs` (add god item discovery/forge shortcuts)
- Modify: `src/main.rs` (handle debug menu god item actions)

**Step 1: Add debug menu options**

Add options to the debug menu:
- "Discover Asprika" — sets `asprika_state = Discovered`
- "Complete Asprika Milestones" — fills all milestones
- "Forge Asprika" — creates and equips the item

**Step 2: Run `make check`**

Run: `make check`
Expected: ALL PASS

**Step 3: Commit**

```bash
git add src/utils/debug_menu.rs src/main.rs
git commit -m "feat: add god item debug menu shortcuts"
```

---

### Task 10: Final Integration Test and Cleanup

**Files:**
- Create: `tests/god_items_test.rs`
- Verify: `make check` passes

**Step 1: Write integration tests**

Create `tests/god_items_test.rs`:

```rust
use quest::god_items::*;
use quest::items::*;

#[test]
fn test_asprika_item_is_always_best_in_slot() {
    // Asprika should outscore any Legendary item
    let asprika = asprika_definition().to_item();
    let legendary = Item {
        slot: EquipmentSlot::Armor,
        rarity: Rarity::Legendary,
        ilvl: 100,
        base_name: "Test".to_string(),
        display_name: "Test".to_string(),
        attributes: AttributeBonuses { con: 20, dex: 10, ..AttributeBonuses::new() },
        affixes: vec![
            Affix { affix_type: AffixType::DamageReduction, value: 15.0 },
            Affix { affix_type: AffixType::HPBonus, value: 100.0 },
        ],
        god_item_id: None,
    };

    let state = quest::GameState::new("Test".to_string(), 0);
    let asprika_score = quest::items::score_item(&asprika, &state);
    let legendary_score = quest::items::score_item(&legendary, &state);
    assert!(asprika_score > legendary_score,
        "Asprika ({}) should outscore a strong Legendary ({})", asprika_score, legendary_score);
}

#[test]
fn test_god_item_progress_state_machine() {
    let mut progress = GodItemProgress::default();
    assert_eq!(progress.asprika_state, GodItemState::Undiscovered);

    progress.asprika_state = GodItemState::Discovered;
    progress.record_master_challenge_win("Chess");
    progress.record_master_challenge_win("Go");
    progress.record_master_challenge_win("Snake");
    progress.sync_enhancement_milestones(&[7, 7, 7, 0, 0, 0, 0]);
    progress.asprika_milestones.temple_trial_return_complete = true;

    assert!(progress.asprika_milestones.all_met());
}

#[test]
fn test_mythic_item_serialization_roundtrip() {
    let asprika = asprika_definition().to_item();
    let json = serde_json::to_string(&asprika).unwrap();
    let loaded: Item = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.rarity, Rarity::Mythic);
    assert_eq!(loaded.god_item_id, Some(GodItemId::Asprika));
}
```

**Step 2: Run full CI checks**

Run: `make check`
Expected: ALL PASS (format, clippy, tests, build, audit)

**Step 3: Commit**

```bash
git add tests/god_items_test.rs
git commit -m "test: add god items integration tests"
```

---

## Task Dependency Graph

```
Task 1 (Mythic rarity)
  └─► Task 2 (God item data model) ─── depends on Mythic variant
        └─► Task 3 (Persistence)
        └─► Task 4 (Divine Bulwark combat) ─── needs god_item_id on Item
        └─► Task 5 (Offline XP bonus)
              └─► Task 6 (Wire into game loop) ─── depends on 4 + 5
                    └─► Task 7 (Milestone tracking)
                          └─► Task 8 (Storm Forge UI)
                                └─► Task 9 (Debug menu)
                                      └─► Task 10 (Integration tests)
```

## Notes

- **Temple Trial integration (issue #98)** is out of scope. For now, discovery can be triggered via the debug menu. The `GodItemState::Discovered` transition will be wired to Temple Trials when that system is designed.
- **Exact stat values** for Asprika (CON 40, WIS 20, affixes) are initial values subject to balance tuning. The stat structure supports easy adjustment.
- **UI work (Task 8)** is less prescriptive than other tasks because it involves Ratatui layout work that's hard to fully specify in advance. Follow existing patterns in `soulforge_scene.rs`.
