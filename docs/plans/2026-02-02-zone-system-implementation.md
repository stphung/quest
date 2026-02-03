# Zone System Implementation Plan (Zones 1-10)

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the zone and subzone progression system for zones 1-10, with prestige gates and boss gates.

**Architecture:** Replace the dormant zone system with a fully integrated progression system. Zones are level-gated with prestige requirements. Subzones have boss gates between them. Zone 10's weapon forging is stubbed (disabled) pending issue #20.

**Tech Stack:** Rust, Serde for serialization, Ratatui for UI

**Related:**
- Design doc: `docs/plans/2026-02-02-zone-system-design.md`
- Future work: GitHub issue #20 (zones 11-20, weapon forging)

---

## Task 1: Update Prestige Multiplier Formula

**Files:**
- Modify: `src/prestige.rs`
- Modify: `src/game_logic.rs` (if prestige_multiplier is duplicated)

**Step 1: Write failing test**

```rust
#[test]
fn test_prestige_multiplier_new_formula() {
    // New formula: 1.2^rank instead of 1.5^rank
    let tier0 = get_prestige_tier(0);
    assert!((tier0.multiplier - 1.0).abs() < 0.001);

    let tier5 = get_prestige_tier(5);
    assert!((tier5.multiplier - 2.488).abs() < 0.01); // 1.2^5 = 2.488

    let tier10 = get_prestige_tier(10);
    assert!((tier10.multiplier - 6.191).abs() < 0.01); // 1.2^10 = 6.191
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_prestige_multiplier_new_formula`
Expected: FAIL (current formula uses 1.5^rank)

**Step 3: Update the multiplier formula**

Change `1.5_f64.powi(rank as i32)` to `1.2_f64.powi(rank as i32)`

**Step 4: Run test to verify it passes**

Run: `cargo test test_prestige_multiplier_new_formula`
Expected: PASS

**Step 5: Update existing tests that check multiplier values**

Fix any tests that assert old 1.5^rank values.

**Step 6: Commit**

```bash
git add src/prestige.rs
git commit -m "feat: change prestige multiplier from 1.5^r to 1.2^r"
```

---

## Task 2: Create Zone Data Structures

**Files:**
- Create: `src/zones/mod.rs`
- Create: `src/zones/data.rs`
- Modify: `src/lib.rs` or `src/main.rs` to add module

**Step 1: Create zones module structure**

```rust
// src/zones/mod.rs
mod data;

pub use data::*;
```

**Step 2: Define zone and subzone structs**

```rust
// src/zones/data.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zone {
    pub id: u32,
    pub name: &'static str,
    pub subzones: Vec<Subzone>,
    pub prestige_requirement: u32,
    pub min_level: u32,
    pub max_level: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subzone {
    pub id: u32,
    pub name: &'static str,
    pub depth: u32,  // 1 = surface, higher = deeper
    pub boss: SubzoneBoss,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubzoneBoss {
    pub name: &'static str,
    pub is_zone_boss: bool,  // true for final subzone boss
}
```

**Step 3: Write test for basic structure**

```rust
#[test]
fn test_zone_structure() {
    let zones = get_all_zones();
    assert_eq!(zones.len(), 10);
    assert_eq!(zones[0].name, "Meadow");
    assert_eq!(zones[0].subzones.len(), 3);
}
```

**Step 4: Commit**

```bash
git add src/zones/
git commit -m "feat: add zone and subzone data structures"
```

---

## Task 3: Define All 10 Zones with Subzones

**Files:**
- Modify: `src/zones/data.rs`

**Step 1: Implement get_all_zones() with full data**

Define all 10 zones with their subzones:

```rust
pub fn get_all_zones() -> Vec<Zone> {
    vec![
        Zone {
            id: 1,
            name: "Meadow",
            prestige_requirement: 0,
            min_level: 1,
            max_level: 10,
            subzones: vec![
                Subzone { id: 1, name: "Sunny Fields", depth: 1, boss: SubzoneBoss { name: "Field Guardian", is_zone_boss: false } },
                Subzone { id: 2, name: "Overgrown Thicket", depth: 2, boss: SubzoneBoss { name: "Thicket Horror", is_zone_boss: false } },
                Subzone { id: 3, name: "Mushroom Caves", depth: 3, boss: SubzoneBoss { name: "Sporeling Queen", is_zone_boss: true } },
            ],
        },
        // ... zones 2-10
    ]
}
```

**Step 2: Write tests for zone data**

```rust
#[test]
fn test_zone_prestige_requirements() {
    let zones = get_all_zones();
    assert_eq!(zones[0].prestige_requirement, 0);  // Meadow
    assert_eq!(zones[1].prestige_requirement, 0);  // Dark Forest
    assert_eq!(zones[2].prestige_requirement, 5);  // Mountain Pass
    assert_eq!(zones[3].prestige_requirement, 5);  // Ancient Ruins
    assert_eq!(zones[4].prestige_requirement, 10); // Volcanic Wastes
    assert_eq!(zones[5].prestige_requirement, 10); // Frozen Tundra
    assert_eq!(zones[6].prestige_requirement, 15); // Crystal Caverns
    assert_eq!(zones[7].prestige_requirement, 15); // Sunken Kingdom
    assert_eq!(zones[8].prestige_requirement, 20); // Floating Isles
    assert_eq!(zones[9].prestige_requirement, 20); // Storm Citadel
}

#[test]
fn test_subzone_counts() {
    let zones = get_all_zones();
    // Tiers 1-2: 3 subzones each
    assert_eq!(zones[0].subzones.len(), 3);
    assert_eq!(zones[1].subzones.len(), 3);
    assert_eq!(zones[2].subzones.len(), 3);
    assert_eq!(zones[3].subzones.len(), 3);
    // Tiers 3-5: 4 subzones each
    assert_eq!(zones[4].subzones.len(), 4);
    assert_eq!(zones[5].subzones.len(), 4);
    assert_eq!(zones[6].subzones.len(), 4);
    assert_eq!(zones[7].subzones.len(), 4);
    assert_eq!(zones[8].subzones.len(), 4);
    assert_eq!(zones[9].subzones.len(), 4);
}
```

**Step 3: Commit**

```bash
git add src/zones/data.rs
git commit -m "feat: define all 10 zones with subzones and bosses"
```

---

## Task 4: Add Zone Progression State to GameState

**Files:**
- Modify: `src/game_state.rs`
- Create: `src/zones/progression.rs`

**Step 1: Create zone progression state struct**

```rust
// src/zones/progression.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ZoneProgression {
    pub current_zone_id: u32,
    pub current_subzone_id: u32,
    pub defeated_bosses: Vec<(u32, u32)>,  // (zone_id, subzone_id) pairs
    pub unlocked_zones: Vec<u32>,
}

impl ZoneProgression {
    pub fn new() -> Self {
        Self {
            current_zone_id: 1,
            current_subzone_id: 1,
            defeated_bosses: vec![],
            unlocked_zones: vec![1, 2],  // Start with zones 1-2 unlocked
        }
    }

    pub fn is_boss_defeated(&self, zone_id: u32, subzone_id: u32) -> bool {
        self.defeated_bosses.contains(&(zone_id, subzone_id))
    }

    pub fn is_zone_unlocked(&self, zone_id: u32) -> bool {
        self.unlocked_zones.contains(&zone_id)
    }
}
```

**Step 2: Add to GameState**

```rust
// In game_state.rs
#[serde(default)]
pub zone_progression: ZoneProgression,
```

**Step 3: Write tests**

```rust
#[test]
fn test_zone_progression_default() {
    let prog = ZoneProgression::new();
    assert_eq!(prog.current_zone_id, 1);
    assert_eq!(prog.current_subzone_id, 1);
    assert!(prog.is_zone_unlocked(1));
    assert!(prog.is_zone_unlocked(2));
    assert!(!prog.is_zone_unlocked(3));
}
```

**Step 4: Commit**

```bash
git add src/zones/progression.rs src/game_state.rs
git commit -m "feat: add zone progression state to GameState"
```

---

## Task 5: Implement Zone Unlock Logic

**Files:**
- Modify: `src/zones/progression.rs`

**Step 1: Add unlock check function**

```rust
impl ZoneProgression {
    pub fn can_unlock_zone(&self, zone: &Zone, prestige_rank: u32) -> bool {
        // Check prestige requirement
        if prestige_rank < zone.prestige_requirement {
            return false;
        }

        // Check if previous zone's final boss is defeated
        if zone.id > 1 {
            let prev_zone_id = zone.id - 1;
            // Need to find max subzone id for previous zone
            // For simplicity, check if any boss from prev zone's last subzone is beaten
            // This will be refined when we have zone data available
        }

        true
    }

    pub fn unlock_zone(&mut self, zone_id: u32) {
        if !self.unlocked_zones.contains(&zone_id) {
            self.unlocked_zones.push(zone_id);
        }
    }

    pub fn defeat_boss(&mut self, zone_id: u32, subzone_id: u32) {
        if !self.is_boss_defeated(zone_id, subzone_id) {
            self.defeated_bosses.push((zone_id, subzone_id));
        }
    }
}
```

**Step 2: Write tests**

```rust
#[test]
fn test_zone_unlock_prestige_gate() {
    let zones = get_all_zones();
    let mut prog = ZoneProgression::new();

    // Zone 3 requires prestige 5
    assert!(!prog.can_unlock_zone(&zones[2], 0));
    assert!(!prog.can_unlock_zone(&zones[2], 4));
    assert!(prog.can_unlock_zone(&zones[2], 5));
    assert!(prog.can_unlock_zone(&zones[2], 10));
}
```

**Step 3: Commit**

```bash
git add src/zones/progression.rs
git commit -m "feat: implement zone unlock logic with prestige gates"
```

---

## Task 6: Implement Subzone Boss Gates

**Files:**
- Modify: `src/zones/progression.rs`

**Step 1: Add subzone progression logic**

```rust
impl ZoneProgression {
    pub fn can_enter_subzone(&self, zone_id: u32, subzone_id: u32) -> bool {
        // First subzone is always accessible if zone is unlocked
        if subzone_id == 1 {
            return self.is_zone_unlocked(zone_id);
        }

        // Need previous subzone's boss defeated
        self.is_boss_defeated(zone_id, subzone_id - 1)
    }

    pub fn advance_to_next_subzone(&mut self, zones: &[Zone]) -> bool {
        let zone = zones.iter().find(|z| z.id == self.current_zone_id);
        if let Some(zone) = zone {
            let max_subzone = zone.subzones.len() as u32;
            if self.current_subzone_id < max_subzone {
                self.current_subzone_id += 1;
                return true;
            }
        }
        false
    }

    pub fn advance_to_next_zone(&mut self, zones: &[Zone], prestige_rank: u32) -> bool {
        let next_zone_id = self.current_zone_id + 1;
        if let Some(next_zone) = zones.iter().find(|z| z.id == next_zone_id) {
            if self.can_unlock_zone(next_zone, prestige_rank) {
                self.unlock_zone(next_zone_id);
                self.current_zone_id = next_zone_id;
                self.current_subzone_id = 1;
                return true;
            }
        }
        false
    }
}
```

**Step 2: Write tests**

```rust
#[test]
fn test_subzone_boss_gate() {
    let mut prog = ZoneProgression::new();

    // Can enter first subzone
    assert!(prog.can_enter_subzone(1, 1));

    // Cannot enter second subzone without defeating first boss
    assert!(!prog.can_enter_subzone(1, 2));

    // Defeat first boss
    prog.defeat_boss(1, 1);

    // Now can enter second subzone
    assert!(prog.can_enter_subzone(1, 2));
}
```

**Step 3: Commit**

```bash
git add src/zones/progression.rs
git commit -m "feat: implement subzone boss gates"
```

---

## Task 7: Integrate Zones with Combat System

**Files:**
- Modify: `src/combat.rs`
- Modify: `src/game_logic.rs`

**Step 1: Update enemy generation to use current zone/subzone**

```rust
pub fn generate_enemy_for_zone(
    zone: &Zone,
    subzone: &Subzone,
    player_max_hp: i32,
    player_damage: i32,
) -> Enemy {
    // Scale enemy based on zone depth and subzone depth
    let zone_multiplier = 1.0 + (zone.id as f64 * 0.1);
    let subzone_multiplier = 1.0 + (subzone.depth as f64 * 0.05);

    // ... generate enemy with scaled stats
}
```

**Step 2: Add boss generation**

```rust
pub fn generate_subzone_boss(
    zone: &Zone,
    subzone: &Subzone,
    player_max_hp: i32,
    player_damage: i32,
) -> Enemy {
    let base_enemy = generate_enemy_for_zone(zone, subzone, player_max_hp, player_damage);

    let (hp_mult, dmg_mult) = if subzone.boss.is_zone_boss {
        (3.0, 2.0)  // Zone boss: 3x HP, 2x damage
    } else {
        (2.0, 1.5)  // Subzone boss: 2x HP, 1.5x damage
    };

    Enemy {
        name: subzone.boss.name.to_string(),
        max_hp: (base_enemy.max_hp as f64 * hp_mult) as i32,
        current_hp: (base_enemy.max_hp as f64 * hp_mult) as i32,
        damage: (base_enemy.damage as f64 * dmg_mult) as i32,
        is_boss: true,
    }
}
```

**Step 3: Write tests**

```rust
#[test]
fn test_zone_boss_scaling() {
    let zones = get_all_zones();
    let boss = generate_subzone_boss(&zones[0], &zones[0].subzones[2], 100, 10);

    assert_eq!(boss.name, "Sporeling Queen");
    assert!(boss.is_boss);
    // Zone boss should have 3x multiplier
}
```

**Step 4: Commit**

```bash
git add src/combat.rs src/game_logic.rs
git commit -m "feat: integrate zone-based enemy and boss generation"
```

---

## Task 8: Update UI to Display Current Zone/Subzone

**Files:**
- Modify: `src/ui/stats_panel.rs` or create `src/ui/zone_display.rs`
- Modify: `src/ui/mod.rs`

**Step 1: Add zone display to stats panel**

Add a section showing:
- Current zone name
- Current subzone name
- Progress indicator (subzone X/Y)
- Next zone unlock requirement (if gated)

**Step 2: Style zone names by tier**

```rust
fn zone_tier_color(zone_id: u32) -> Color {
    match zone_id {
        1..=2 => Color::Green,    // Tier 1
        3..=4 => Color::Yellow,   // Tier 2
        5..=6 => Color::Red,      // Tier 3
        7..=8 => Color::Magenta,  // Tier 4
        9..=10 => Color::Cyan,    // Tier 5
        _ => Color::White,
    }
}
```

**Step 3: Commit**

```bash
git add src/ui/
git commit -m "feat: display current zone and subzone in UI"
```

---

## Task 9: Add Zone Progression to Prestige Reset

**Files:**
- Modify: `src/prestige.rs`

**Step 1: Reset zone progression on prestige (with unlocks preserved)**

```rust
pub fn perform_prestige(state: &mut GameState) {
    // ... existing prestige logic ...

    // Reset zone progression but keep unlocks based on new prestige rank
    state.zone_progression.current_zone_id = 1;
    state.zone_progression.current_subzone_id = 1;
    state.zone_progression.defeated_bosses.clear();

    // Recalculate unlocked zones based on new prestige rank
    let zones = get_all_zones();
    state.zone_progression.unlocked_zones = zones
        .iter()
        .filter(|z| z.prestige_requirement <= state.prestige_rank)
        .map(|z| z.id)
        .collect();
}
```

**Step 2: Write tests**

```rust
#[test]
fn test_prestige_preserves_zone_unlocks() {
    let mut state = GameState::new("Test".to_string(), 0);
    state.prestige_rank = 5;
    state.character_level = 25;  // Enough for next prestige

    perform_prestige(&mut state);

    // Should have zones 1-4 unlocked (P0 and P5 zones)
    assert!(state.zone_progression.is_zone_unlocked(1));
    assert!(state.zone_progression.is_zone_unlocked(4));
    assert!(!state.zone_progression.is_zone_unlocked(5));  // Needs P10
}
```

**Step 3: Commit**

```bash
git add src/prestige.rs
git commit -m "feat: integrate zone progression with prestige system"
```

---

## Task 10: Remove Old Zone System

**Files:**
- Delete or refactor: `src/ui/zones.rs`
- Update any imports

**Step 1: Remove old zones.rs or merge useful parts**

The old `src/ui/zones.rs` has some useful helpers like `get_random_environment()`. Either:
- Delete entirely and reimplement in new system
- Keep as utility functions if needed

**Step 2: Update imports**

Ensure all zone-related imports point to new `src/zones/` module.

**Step 3: Run full test suite**

```bash
cargo test
```

**Step 4: Commit**

```bash
git add -A
git commit -m "refactor: remove legacy zone system, use new zones module"
```

---

## Task 11: Add Zone 10 Weapon Placeholder

**Files:**
- Modify: `src/zones/data.rs`

**Step 1: Add placeholder for weapon system**

Zone 10 should indicate it has a weapon, but the system is disabled:

```rust
// In Zone 10 definition
Zone {
    id: 10,
    name: "Storm Citadel",
    // ...
    weapon_available: false,  // Disabled until issue #20
    weapon_name: Some("Stormbreaker"),  // For future
}
```

**Step 2: Add UI indicator**

When reaching Zone 10's final subzone, show message:
"The path forward is shrouded... A legendary weapon must be forged. (Coming soon)"

**Step 3: Commit**

```bash
git add src/zones/
git commit -m "feat: add Zone 10 weapon placeholder (disabled, see issue #20)"
```

---

## Task 12: Final Integration Test

**Files:**
- Create: `tests/zone_integration_test.rs` or add to existing test file

**Step 1: Write end-to-end zone progression test**

```rust
#[test]
fn test_full_zone_progression_flow() {
    let mut state = GameState::new("Test Hero".to_string(), 0);

    // Start in Zone 1, Subzone 1
    assert_eq!(state.zone_progression.current_zone_id, 1);
    assert_eq!(state.zone_progression.current_subzone_id, 1);

    // Defeat all bosses in Zone 1
    state.zone_progression.defeat_boss(1, 1);
    state.zone_progression.defeat_boss(1, 2);
    state.zone_progression.defeat_boss(1, 3);

    // Should be able to advance to Zone 2
    let zones = get_all_zones();
    assert!(state.zone_progression.advance_to_next_zone(&zones, 0));
    assert_eq!(state.zone_progression.current_zone_id, 2);

    // Zone 3 should be locked (needs P5)
    state.zone_progression.defeat_boss(2, 1);
    state.zone_progression.defeat_boss(2, 2);
    state.zone_progression.defeat_boss(2, 3);
    assert!(!state.zone_progression.advance_to_next_zone(&zones, 0));

    // With P5, should unlock
    assert!(state.zone_progression.advance_to_next_zone(&zones, 5));
    assert_eq!(state.zone_progression.current_zone_id, 3);
}
```

**Step 2: Run all tests**

```bash
cargo test
make check
```

**Step 3: Final commit**

```bash
git add -A
git commit -m "test: add zone progression integration tests"
```

---

## Summary

| Task | Description | Estimated Complexity |
|------|-------------|---------------------|
| 1 | Update prestige multiplier | Simple |
| 2 | Create zone data structures | Medium |
| 3 | Define all 10 zones | Medium |
| 4 | Add zone progression state | Medium |
| 5 | Implement zone unlock logic | Medium |
| 6 | Implement subzone boss gates | Medium |
| 7 | Integrate with combat system | Complex |
| 8 | Update UI | Medium |
| 9 | Integrate with prestige | Medium |
| 10 | Remove old zone system | Simple |
| 11 | Add Zone 10 placeholder | Simple |
| 12 | Integration tests | Medium |

**Total tasks:** 12
**Dependencies:** Tasks should be done roughly in order, though 2-6 can be batched.
