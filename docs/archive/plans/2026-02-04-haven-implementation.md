# Haven Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the Haven base building system — an account-level skill tree where players spend prestige and fishing ranks to build rooms that provide permanent passive bonuses.

**Architecture:** Haven state lives in `~/.quest/haven.json`, separate from character saves. Data structures and logic follow the same pattern as fishing/challenges (data module + logic module + UI scene). The Haven integrates with the game loop via an independent discovery roll, with the existing prestige system modified to support the Vault capstone.

**Tech Stack:** Rust, Ratatui 0.26, Serde JSON, Rand

**Design doc:** `docs/plans/2026-02-04-haven-design.md`

---

### Task 1: Haven Data Structures

**Files:**
- Create: `src/haven/mod.rs` (module exports, following the refactored module pattern)
- Create: `src/haven/types.rs` (room definitions, bonuses, Haven state)
- Modify: `src/lib.rs` (add `pub mod haven;`)

**Step 1: Create the haven module structure**

Create `src/haven/mod.rs`:

```rust
//! Haven base building system — account-level skill tree.

pub mod types;

pub use types::*;
```

**Step 2: Write the types module with tests**

Create `src/haven/types.rs` with the data structures:

```rust
//! Haven base building system — account-level skill tree.
//!
//! The Haven persists across all prestige resets and benefits every character.
//! Players spend prestige ranks and fishing ranks to build and upgrade rooms.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Room identifiers in the Haven skill tree
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HavenRoomId {
    // Root
    Hearthstone,
    // Combat branch
    Armory,
    TrainingYard,
    TrophyHall,
    Watchtower,
    AlchemyLab,
    WarRoom,
    // QoL branch
    Bedroom,
    Garden,
    Library,
    FishingDock,
    Workshop,
    Vault,
}

impl HavenRoomId {
    /// All room IDs in tree order
    pub const ALL: [HavenRoomId; 13] = [
        HavenRoomId::Hearthstone,
        HavenRoomId::Armory,
        HavenRoomId::TrainingYard,
        HavenRoomId::TrophyHall,
        HavenRoomId::Watchtower,
        HavenRoomId::AlchemyLab,
        HavenRoomId::WarRoom,
        HavenRoomId::Bedroom,
        HavenRoomId::Garden,
        HavenRoomId::Library,
        HavenRoomId::FishingDock,
        HavenRoomId::Workshop,
        HavenRoomId::Vault,
    ];

    /// Display name for UI
    pub fn name(&self) -> &'static str {
        match self {
            HavenRoomId::Hearthstone => "Hearthstone",
            HavenRoomId::Armory => "Armory",
            HavenRoomId::TrainingYard => "Training Yard",
            HavenRoomId::TrophyHall => "Trophy Hall",
            HavenRoomId::Watchtower => "Watchtower",
            HavenRoomId::AlchemyLab => "Alchemy Lab",
            HavenRoomId::WarRoom => "War Room",
            HavenRoomId::Bedroom => "Bedroom",
            HavenRoomId::Garden => "Garden",
            HavenRoomId::Library => "Library",
            HavenRoomId::FishingDock => "Fishing Dock",
            HavenRoomId::Workshop => "Workshop",
            HavenRoomId::Vault => "Vault",
        }
    }

    /// Flavor description for detail panel
    pub fn description(&self) -> &'static str {
        match self {
            HavenRoomId::Hearthstone => "The warm center of your Haven.",
            HavenRoomId::Armory => "Your weapon collection strengthens all who visit.",
            HavenRoomId::TrainingYard => "Practice dummies and sparring targets.",
            HavenRoomId::TrophyHall => "Trophies from past victories attract fortune.",
            HavenRoomId::Watchtower => "Sharpens your eye for weak points.",
            HavenRoomId::AlchemyLab => "Brews and tonics always simmering.",
            HavenRoomId::WarRoom => "Tactical planning speeds your strikes.",
            HavenRoomId::Bedroom => "Rest well, fight well.",
            HavenRoomId::Garden => "Patience cultivated here carries over.",
            HavenRoomId::Library => "Ancient tomes reveal hidden challenges.",
            HavenRoomId::FishingDock => "A private spot to cast.",
            HavenRoomId::Workshop => "Better tools yield better finds.",
            HavenRoomId::Vault => "Preserves treasured equipment through prestige resets.",
        }
    }

    /// Parent room(s) that must be T1+ to unlock this room.
    /// Returns empty slice for Hearthstone (root).
    /// Capstones require both parents.
    pub fn parents(&self) -> &'static [HavenRoomId] {
        match self {
            HavenRoomId::Hearthstone => &[],
            // Combat branch
            HavenRoomId::Armory => &[HavenRoomId::Hearthstone],
            HavenRoomId::TrainingYard => &[HavenRoomId::Armory],
            HavenRoomId::TrophyHall => &[HavenRoomId::Armory],
            HavenRoomId::Watchtower => &[HavenRoomId::TrainingYard],
            HavenRoomId::AlchemyLab => &[HavenRoomId::TrophyHall],
            HavenRoomId::WarRoom => &[HavenRoomId::Watchtower, HavenRoomId::AlchemyLab],
            // QoL branch
            HavenRoomId::Bedroom => &[HavenRoomId::Hearthstone],
            HavenRoomId::Garden => &[HavenRoomId::Bedroom],
            HavenRoomId::Library => &[HavenRoomId::Bedroom],
            HavenRoomId::FishingDock => &[HavenRoomId::Garden],
            HavenRoomId::Workshop => &[HavenRoomId::Library],
            HavenRoomId::Vault => &[HavenRoomId::FishingDock, HavenRoomId::Workshop],
        }
    }

    /// Child rooms that this room unlocks when built to T1+.
    pub fn children(&self) -> &'static [HavenRoomId] {
        match self {
            HavenRoomId::Hearthstone => &[HavenRoomId::Armory, HavenRoomId::Bedroom],
            HavenRoomId::Armory => &[HavenRoomId::TrainingYard, HavenRoomId::TrophyHall],
            HavenRoomId::TrainingYard => &[HavenRoomId::Watchtower],
            HavenRoomId::TrophyHall => &[HavenRoomId::AlchemyLab],
            HavenRoomId::Watchtower => &[HavenRoomId::WarRoom],
            HavenRoomId::AlchemyLab => &[HavenRoomId::WarRoom],
            HavenRoomId::WarRoom => &[],
            HavenRoomId::Bedroom => &[HavenRoomId::Garden, HavenRoomId::Library],
            HavenRoomId::Garden => &[HavenRoomId::FishingDock],
            HavenRoomId::Library => &[HavenRoomId::Workshop],
            HavenRoomId::FishingDock => &[HavenRoomId::Vault],
            HavenRoomId::Workshop => &[HavenRoomId::Vault],
            HavenRoomId::Vault => &[],
        }
    }

    /// Whether this room is a capstone (requires two parents)
    pub fn is_capstone(&self) -> bool {
        matches!(self, HavenRoomId::WarRoom | HavenRoomId::Vault)
    }
}

/// Cost to build or upgrade a room at a given tier
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HavenCost {
    pub prestige_ranks: u32,
    pub fishing_ranks: u32,
}

/// Get the cost for a specific tier (1, 2, or 3)
pub fn tier_cost(tier: u8) -> HavenCost {
    match tier {
        1 => HavenCost { prestige_ranks: 1, fishing_ranks: 2 },
        2 => HavenCost { prestige_ranks: 3, fishing_ranks: 4 },
        3 => HavenCost { prestige_ranks: 5, fishing_ranks: 6 },
        _ => HavenCost { prestige_ranks: 0, fishing_ranks: 0 },
    }
}

/// Bonus type that a room provides
#[derive(Debug, Clone, Copy)]
pub enum HavenBonusType {
    DamagePercent,
    XpGainPercent,
    DropRatePercent,
    CritChancePercent,
    HpRegenPercent,
    AttackIntervalReduction,
    OfflineXpPercent,
    ChallengeDiscoveryPercent,
    FishingTimerReduction,
    FishingRankXpPercent,
    ItemRarityPercent,
    HpRegenDelayReduction,
    VaultSlots,
}

/// A specific bonus value for a room at a given tier
#[derive(Debug, Clone, Copy)]
pub struct HavenBonus {
    pub bonus_type: HavenBonusType,
    pub values: [f64; 3], // T1, T2, T3
}

impl HavenRoomId {
    /// Get the bonus definition for this room
    pub fn bonus(&self) -> HavenBonus {
        match self {
            HavenRoomId::Hearthstone => HavenBonus {
                bonus_type: HavenBonusType::OfflineXpPercent,
                values: [10.0, 25.0, 40.0],
            },
            HavenRoomId::Armory => HavenBonus {
                bonus_type: HavenBonusType::DamagePercent,
                values: [5.0, 10.0, 18.0],
            },
            HavenRoomId::TrainingYard => HavenBonus {
                bonus_type: HavenBonusType::XpGainPercent,
                values: [5.0, 12.0, 20.0],
            },
            HavenRoomId::TrophyHall => HavenBonus {
                bonus_type: HavenBonusType::DropRatePercent,
                values: [2.0, 4.0, 7.0],
            },
            HavenRoomId::Watchtower => HavenBonus {
                bonus_type: HavenBonusType::CritChancePercent,
                values: [3.0, 6.0, 10.0],
            },
            HavenRoomId::AlchemyLab => HavenBonus {
                bonus_type: HavenBonusType::HpRegenPercent,
                values: [15.0, 30.0, 50.0],
            },
            HavenRoomId::WarRoom => HavenBonus {
                bonus_type: HavenBonusType::AttackIntervalReduction,
                values: [5.0, 10.0, 15.0],
            },
            HavenRoomId::Bedroom => HavenBonus {
                bonus_type: HavenBonusType::HpRegenDelayReduction,
                values: [10.0, 20.0, 35.0],
            },
            HavenRoomId::Garden => HavenBonus {
                bonus_type: HavenBonusType::FishingTimerReduction,
                values: [10.0, 20.0, 30.0],
            },
            HavenRoomId::Library => HavenBonus {
                bonus_type: HavenBonusType::ChallengeDiscoveryPercent,
                values: [20.0, 40.0, 65.0],
            },
            HavenRoomId::FishingDock => HavenBonus {
                bonus_type: HavenBonusType::FishingRankXpPercent,
                values: [15.0, 30.0, 50.0],
            },
            HavenRoomId::Workshop => HavenBonus {
                bonus_type: HavenBonusType::ItemRarityPercent,
                values: [5.0, 10.0, 18.0],
            },
            HavenRoomId::Vault => HavenBonus {
                bonus_type: HavenBonusType::VaultSlots,
                values: [1.0, 2.0, 3.0],
            },
        }
    }

    /// Get the bonus value for a specific tier (0 = unbuilt)
    pub fn bonus_value(&self, tier: u8) -> f64 {
        if tier == 0 || tier > 3 { return 0.0; }
        self.bonus().values[(tier - 1) as usize]
    }

    /// Format the bonus for display (e.g., "+5% DMG", "-10% Attack Interval")
    pub fn format_bonus(&self, tier: u8) -> String {
        if tier == 0 { return String::new(); }
        let value = self.bonus_value(tier);
        match self.bonus().bonus_type {
            HavenBonusType::DamagePercent => format!("+{:.0}% DMG", value),
            HavenBonusType::XpGainPercent => format!("+{:.0}% XP", value),
            HavenBonusType::DropRatePercent => format!("+{:.0}% Drops", value),
            HavenBonusType::CritChancePercent => format!("+{:.0}% Crit", value),
            HavenBonusType::HpRegenPercent => format!("+{:.0}% HP Regen", value),
            HavenBonusType::AttackIntervalReduction => format!("-{:.0}% Attack Interval", value),
            HavenBonusType::OfflineXpPercent => format!("+{:.0}% Offline XP", value),
            HavenBonusType::ChallengeDiscoveryPercent => format!("+{:.0}% Discovery", value),
            HavenBonusType::FishingTimerReduction => format!("-{:.0}% Fishing Timers", value),
            HavenBonusType::FishingRankXpPercent => format!("+{:.0}% Fishing XP", value),
            HavenBonusType::ItemRarityPercent => format!("+{:.0}% Item Rarity", value),
            HavenBonusType::HpRegenDelayReduction => format!("-{:.0}% Regen Delay", value),
            HavenBonusType::VaultSlots => format!("{:.0} item{} preserved", value, if value > 1.0 { "s" } else { "" }),
        }
    }
}

/// Account-level Haven state, saved to ~/.quest/haven.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Haven {
    pub discovered: bool,
    pub rooms: HashMap<HavenRoomId, u8>,
}

impl Default for Haven {
    fn default() -> Self {
        let mut rooms = HashMap::new();
        for room in HavenRoomId::ALL {
            rooms.insert(room, 0);
        }
        Haven {
            discovered: false,
            rooms,
        }
    }
}

impl Haven {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the tier of a room (0 = unbuilt, 1-3)
    pub fn room_tier(&self, room: HavenRoomId) -> u8 {
        *self.rooms.get(&room).unwrap_or(&0)
    }

    /// Check if a room is unlocked (all parents at T1+)
    pub fn is_room_unlocked(&self, room: HavenRoomId) -> bool {
        room.parents().iter().all(|p| self.room_tier(*p) >= 1)
    }

    /// Check if a room can be built or upgraded
    pub fn can_build(&self, room: HavenRoomId) -> bool {
        let tier = self.room_tier(room);
        tier < 3 && self.is_room_unlocked(room)
    }

    /// Get the next tier for a room (current + 1), or None if maxed
    pub fn next_tier(&self, room: HavenRoomId) -> Option<u8> {
        let tier = self.room_tier(room);
        if tier < 3 { Some(tier + 1) } else { None }
    }

    /// Build or upgrade a room. Returns the new tier, or None if not possible.
    pub fn build_room(&mut self, room: HavenRoomId) -> Option<u8> {
        if !self.can_build(room) { return None; }
        let new_tier = self.room_tier(room) + 1;
        self.rooms.insert(room, new_tier);
        Some(new_tier)
    }

    /// Count of rooms built (tier >= 1)
    pub fn rooms_built(&self) -> usize {
        self.rooms.values().filter(|&&t| t >= 1).count()
    }

    /// Total rooms in the tree
    pub fn total_rooms(&self) -> usize {
        HavenRoomId::ALL.len()
    }

    /// Get the vault tier (0 if not built)
    pub fn vault_tier(&self) -> u8 {
        self.room_tier(HavenRoomId::Vault)
    }

    /// Get the bonus value for a specific bonus type (sum if relevant, but each room has a unique type)
    pub fn get_bonus(&self, bonus_type: HavenBonusType) -> f64 {
        HavenRoomId::ALL
            .iter()
            .filter(|r| {
                std::mem::discriminant(&r.bonus().bonus_type) == std::mem::discriminant(&bonus_type)
            })
            .map(|r| r.bonus_value(self.room_tier(*r)))
            .sum()
    }
}

/// Discovery chance per tick. Scales with prestige rank.
/// Base: 0.000014 (~2hr at P10), +0.000007 per rank above 10.
pub fn haven_discovery_chance(prestige_rank: u32) -> f64 {
    if prestige_rank < 10 { return 0.0; }
    0.000014 + (prestige_rank - 10) as f64 * 0.000007
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_haven_all_rooms_unbuilt() {
        let haven = Haven::new();
        assert!(!haven.discovered);
        assert_eq!(haven.rooms_built(), 0);
        assert_eq!(haven.total_rooms(), 13);
        for room in HavenRoomId::ALL {
            assert_eq!(haven.room_tier(room), 0);
        }
    }

    #[test]
    fn test_hearthstone_is_root() {
        assert!(HavenRoomId::Hearthstone.parents().is_empty());
        assert_eq!(HavenRoomId::Hearthstone.children(), &[HavenRoomId::Armory, HavenRoomId::Bedroom]);
    }

    #[test]
    fn test_capstone_requires_two_parents() {
        assert!(HavenRoomId::WarRoom.is_capstone());
        assert_eq!(HavenRoomId::WarRoom.parents(), &[HavenRoomId::Watchtower, HavenRoomId::AlchemyLab]);
        assert!(HavenRoomId::Vault.is_capstone());
        assert_eq!(HavenRoomId::Vault.parents(), &[HavenRoomId::FishingDock, HavenRoomId::Workshop]);
    }

    #[test]
    fn test_hearthstone_unlocked_by_default() {
        let haven = Haven::new();
        assert!(haven.is_room_unlocked(HavenRoomId::Hearthstone));
        assert!(!haven.is_room_unlocked(HavenRoomId::Armory));
        assert!(!haven.is_room_unlocked(HavenRoomId::Bedroom));
    }

    #[test]
    fn test_building_hearthstone_unlocks_children() {
        let mut haven = Haven::new();
        haven.build_room(HavenRoomId::Hearthstone);
        assert_eq!(haven.room_tier(HavenRoomId::Hearthstone), 1);
        assert!(haven.is_room_unlocked(HavenRoomId::Armory));
        assert!(haven.is_room_unlocked(HavenRoomId::Bedroom));
        assert!(!haven.is_room_unlocked(HavenRoomId::TrainingYard));
    }

    #[test]
    fn test_cannot_build_locked_room() {
        let mut haven = Haven::new();
        assert!(!haven.can_build(HavenRoomId::Armory));
        assert!(haven.build_room(HavenRoomId::Armory).is_none());
    }

    #[test]
    fn test_cannot_build_past_tier_3() {
        let mut haven = Haven::new();
        haven.build_room(HavenRoomId::Hearthstone); // T1
        haven.build_room(HavenRoomId::Hearthstone); // T2
        haven.build_room(HavenRoomId::Hearthstone); // T3
        assert!(!haven.can_build(HavenRoomId::Hearthstone));
        assert!(haven.build_room(HavenRoomId::Hearthstone).is_none());
        assert_eq!(haven.room_tier(HavenRoomId::Hearthstone), 3);
    }

    #[test]
    fn test_capstone_requires_both_parents() {
        let mut haven = Haven::new();
        haven.build_room(HavenRoomId::Hearthstone);
        haven.build_room(HavenRoomId::Armory);
        haven.build_room(HavenRoomId::TrainingYard);
        haven.build_room(HavenRoomId::TrophyHall);
        haven.build_room(HavenRoomId::Watchtower);
        // Only one parent built — should NOT unlock War Room
        assert!(!haven.is_room_unlocked(HavenRoomId::WarRoom));
        // Build second parent
        haven.build_room(HavenRoomId::AlchemyLab);
        assert!(haven.is_room_unlocked(HavenRoomId::WarRoom));
    }

    #[test]
    fn test_tier_costs() {
        assert_eq!(tier_cost(1), HavenCost { prestige_ranks: 1, fishing_ranks: 2 });
        assert_eq!(tier_cost(2), HavenCost { prestige_ranks: 3, fishing_ranks: 4 });
        assert_eq!(tier_cost(3), HavenCost { prestige_ranks: 5, fishing_ranks: 6 });
    }

    #[test]
    fn test_bonus_values() {
        assert_eq!(HavenRoomId::Armory.bonus_value(0), 0.0);
        assert_eq!(HavenRoomId::Armory.bonus_value(1), 5.0);
        assert_eq!(HavenRoomId::Armory.bonus_value(2), 10.0);
        assert_eq!(HavenRoomId::Armory.bonus_value(3), 18.0);
    }

    #[test]
    fn test_format_bonus() {
        assert_eq!(HavenRoomId::Armory.format_bonus(1), "+5% DMG");
        assert_eq!(HavenRoomId::WarRoom.format_bonus(3), "-15% Attack Interval");
        assert_eq!(HavenRoomId::Vault.format_bonus(1), "1 item preserved");
        assert_eq!(HavenRoomId::Vault.format_bonus(3), "3 items preserved");
    }

    #[test]
    fn test_rooms_built_count() {
        let mut haven = Haven::new();
        assert_eq!(haven.rooms_built(), 0);
        haven.build_room(HavenRoomId::Hearthstone);
        assert_eq!(haven.rooms_built(), 1);
        haven.build_room(HavenRoomId::Armory);
        assert_eq!(haven.rooms_built(), 2);
        // Upgrading doesn't change count
        haven.build_room(HavenRoomId::Hearthstone); // T2
        assert_eq!(haven.rooms_built(), 2);
    }

    #[test]
    fn test_discovery_chance_below_p10() {
        assert_eq!(haven_discovery_chance(0), 0.0);
        assert_eq!(haven_discovery_chance(9), 0.0);
    }

    #[test]
    fn test_discovery_chance_scales_with_prestige() {
        let p10 = haven_discovery_chance(10);
        let p12 = haven_discovery_chance(12);
        let p20 = haven_discovery_chance(20);
        assert!(p10 > 0.0);
        assert!(p12 > p10);
        assert!(p20 > p12);
        assert!((p10 - 0.000014).abs() < 0.0000001);
    }

    #[test]
    fn test_serde_roundtrip() {
        let mut haven = Haven::new();
        haven.discovered = true;
        haven.build_room(HavenRoomId::Hearthstone);
        haven.build_room(HavenRoomId::Hearthstone); // T2
        haven.build_room(HavenRoomId::Armory);

        let json = serde_json::to_string(&haven).unwrap();
        let loaded: Haven = serde_json::from_str(&json).unwrap();

        assert!(loaded.discovered);
        assert_eq!(loaded.room_tier(HavenRoomId::Hearthstone), 2);
        assert_eq!(loaded.room_tier(HavenRoomId::Armory), 1);
        assert_eq!(loaded.room_tier(HavenRoomId::Bedroom), 0);
    }

    #[test]
    fn test_get_bonus_from_haven() {
        let mut haven = Haven::new();
        assert_eq!(haven.get_bonus(HavenBonusType::DamagePercent), 0.0);
        haven.build_room(HavenRoomId::Hearthstone);
        haven.build_room(HavenRoomId::Armory);
        assert_eq!(haven.get_bonus(HavenBonusType::DamagePercent), 5.0);
    }

    #[test]
    fn test_full_combat_branch_buildable() {
        let mut haven = Haven::new();
        // Build full combat branch
        assert!(haven.build_room(HavenRoomId::Hearthstone).is_some());
        assert!(haven.build_room(HavenRoomId::Armory).is_some());
        assert!(haven.build_room(HavenRoomId::TrainingYard).is_some());
        assert!(haven.build_room(HavenRoomId::TrophyHall).is_some());
        assert!(haven.build_room(HavenRoomId::Watchtower).is_some());
        assert!(haven.build_room(HavenRoomId::AlchemyLab).is_some());
        assert!(haven.build_room(HavenRoomId::WarRoom).is_some());
        assert_eq!(haven.rooms_built(), 7);
    }
}
```

**Step 3: Run test to verify it compiles and passes**

Run: `cargo test haven::types::tests -- --nocapture`

Expected: All tests PASS.

**Step 4: Add module declaration**

Add `pub mod haven;` to `src/lib.rs` (alphabetically, after `pub mod fishing;`).

**Step 5: Run all tests**

Run: `cargo test`

Expected: All existing tests + new haven tests pass.

**Step 6: Commit**

```bash
git add src/haven/ src/lib.rs
git commit -m "feat(haven): add data structures, room definitions, and skill tree"
```

---

### Task 2: Haven Logic — Build Validation & Persistence

**Files:**
- Create: `src/haven/logic.rs`
- Modify: `src/haven/mod.rs` (add `pub mod logic;`)

**Step 1: Write the logic module**

Create `src/haven/logic.rs`:

```rust
//! Haven build/upgrade logic and persistence.

use super::types::{Haven, HavenRoomId, haven_discovery_chance, tier_cost};
use rand::Rng;
use std::fs;
use std::io;
use std::path::PathBuf;

/// Check if a character can afford to build/upgrade a room
pub fn can_afford(
    room: HavenRoomId,
    haven: &Haven,
    prestige_rank: u32,
    fishing_rank: u32,
) -> bool {
    let next = match haven.next_tier(room) {
        Some(t) => t,
        None => return false,
    };
    let cost = tier_cost(next);
    prestige_rank >= cost.prestige_ranks && fishing_rank >= cost.fishing_ranks
}

/// Attempt to build/upgrade a room, spending character ranks.
/// Returns (new_tier, prestige_spent, fishing_spent) on success.
pub fn try_build_room(
    room: HavenRoomId,
    haven: &mut Haven,
    prestige_rank: &mut u32,
    fishing_rank: &mut u32,
) -> Option<(u8, u32, u32)> {
    if !haven.can_build(room) {
        return None;
    }
    let next = haven.next_tier(room)?;
    let cost = tier_cost(next);
    if *prestige_rank < cost.prestige_ranks || *fishing_rank < cost.fishing_ranks {
        return None;
    }
    *prestige_rank -= cost.prestige_ranks;
    *fishing_rank -= cost.fishing_ranks;
    haven.build_room(room);
    Some((next, cost.prestige_ranks, cost.fishing_ranks))
}

/// Try to discover the Haven. Independent roll per tick.
/// Returns true if discovered this tick.
pub fn try_discover_haven<R: Rng>(
    haven: &mut Haven,
    prestige_rank: u32,
    rng: &mut R,
) -> bool {
    if haven.discovered {
        return false;
    }
    let chance = haven_discovery_chance(prestige_rank);
    if chance <= 0.0 {
        return false;
    }
    if rng.gen::<f64>() < chance {
        haven.discovered = true;
        return true;
    }
    false
}

/// Get the Haven save file path
pub fn haven_save_path() -> io::Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Could not determine home directory",
        )
    })?;
    Ok(home_dir.join(".quest").join("haven.json"))
}

/// Load Haven from disk, or return default if not found
pub fn load_haven() -> Haven {
    let path = match haven_save_path() {
        Ok(p) => p,
        Err(_) => return Haven::new(),
    };
    match fs::read_to_string(&path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => Haven::new(),
    }
}

/// Save Haven to disk
pub fn save_haven(haven: &Haven) -> io::Result<()> {
    let path = haven_save_path()?;
    let json = serde_json::to_string_pretty(haven)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_can_afford_basic() {
        let haven = Haven::new();
        // Hearthstone T1 costs 1 prestige, 2 fishing
        assert!(can_afford(HavenRoomId::Hearthstone, &haven, 1, 2));
        assert!(!can_afford(HavenRoomId::Hearthstone, &haven, 0, 2));
        assert!(!can_afford(HavenRoomId::Hearthstone, &haven, 1, 1));
    }

    #[test]
    fn test_can_afford_tier_2() {
        let mut haven = Haven::new();
        haven.build_room(HavenRoomId::Hearthstone); // T1
        // T2 costs 3 prestige, 4 fishing
        assert!(can_afford(HavenRoomId::Hearthstone, &haven, 3, 4));
        assert!(!can_afford(HavenRoomId::Hearthstone, &haven, 2, 4));
    }

    #[test]
    fn test_can_afford_maxed_room() {
        let mut haven = Haven::new();
        haven.build_room(HavenRoomId::Hearthstone);
        haven.build_room(HavenRoomId::Hearthstone);
        haven.build_room(HavenRoomId::Hearthstone); // T3
        assert!(!can_afford(HavenRoomId::Hearthstone, &haven, 100, 100));
    }

    #[test]
    fn test_try_build_room_success() {
        let mut haven = Haven::new();
        let mut prestige = 10u32;
        let mut fishing = 10u32;
        let result = try_build_room(HavenRoomId::Hearthstone, &mut haven, &mut prestige, &mut fishing);
        assert_eq!(result, Some((1, 1, 2)));
        assert_eq!(prestige, 9);
        assert_eq!(fishing, 8);
        assert_eq!(haven.room_tier(HavenRoomId::Hearthstone), 1);
    }

    #[test]
    fn test_try_build_room_insufficient_funds() {
        let mut haven = Haven::new();
        let mut prestige = 0u32;
        let mut fishing = 0u32;
        let result = try_build_room(HavenRoomId::Hearthstone, &mut haven, &mut prestige, &mut fishing);
        assert!(result.is_none());
        assert_eq!(haven.room_tier(HavenRoomId::Hearthstone), 0);
    }

    #[test]
    fn test_try_build_room_locked() {
        let mut haven = Haven::new();
        let mut prestige = 100u32;
        let mut fishing = 100u32;
        // Armory is locked (Hearthstone not built)
        let result = try_build_room(HavenRoomId::Armory, &mut haven, &mut prestige, &mut fishing);
        assert!(result.is_none());
        assert_eq!(prestige, 100); // Not spent
    }

    #[test]
    fn test_try_discover_haven_below_p10() {
        let mut haven = Haven::new();
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        // Below P10, should never discover
        for _ in 0..100_000 {
            assert!(!try_discover_haven(&mut haven, 9, &mut rng));
        }
    }

    #[test]
    fn test_try_discover_haven_already_discovered() {
        let mut haven = Haven::new();
        haven.discovered = true;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        assert!(!try_discover_haven(&mut haven, 20, &mut rng));
    }

    #[test]
    fn test_try_discover_haven_eventually_succeeds() {
        let mut haven = Haven::new();
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut discovered = false;
        for _ in 0..1_000_000 {
            if try_discover_haven(&mut haven, 10, &mut rng) {
                discovered = true;
                break;
            }
        }
        assert!(discovered, "Should discover haven within 1M ticks at P10");
        assert!(haven.discovered);
    }

    #[test]
    fn test_build_full_branch_costs() {
        let mut haven = Haven::new();
        let mut prestige = 200u32;
        let mut fishing = 200u32;
        let initial_p = prestige;
        let initial_f = fishing;

        // Build full combat branch at T1
        try_build_room(HavenRoomId::Hearthstone, &mut haven, &mut prestige, &mut fishing);
        try_build_room(HavenRoomId::Armory, &mut haven, &mut prestige, &mut fishing);
        try_build_room(HavenRoomId::TrainingYard, &mut haven, &mut prestige, &mut fishing);
        try_build_room(HavenRoomId::TrophyHall, &mut haven, &mut prestige, &mut fishing);
        try_build_room(HavenRoomId::Watchtower, &mut haven, &mut prestige, &mut fishing);
        try_build_room(HavenRoomId::AlchemyLab, &mut haven, &mut prestige, &mut fishing);
        try_build_room(HavenRoomId::WarRoom, &mut haven, &mut prestige, &mut fishing);

        // 7 rooms at T1 = 7 prestige, 14 fishing
        assert_eq!(initial_p - prestige, 7);
        assert_eq!(initial_f - fishing, 14);
    }
}
```

**Step 2: Run test to verify**

Run: `cargo test haven::logic::tests -- --nocapture`

Expected: All tests PASS (except discovery test needs `rand_chacha` — check if it's already a dep, otherwise use `rand::rngs::StdRng`).

**Step 3: Add module declaration**

Add `pub mod logic;` and `pub use logic::*;` to `src/haven/mod.rs`.

**Step 4: Run all tests**

Run: `cargo test`

Expected: All pass.

**Step 5: Commit**

```bash
git add src/haven/logic.rs src/haven/mod.rs
git commit -m "feat(haven): add build/upgrade logic, discovery, and persistence"
```

---

### Task 3: Add Debug Menu Trigger for Haven

**Files:**
- Modify: `src/utils/debug_menu.rs`

**Context:** The debug menu (activated with `--debug` flag, toggled with backtick) allows testing chance-based discoveries. Add a "Trigger Haven Discovery" option for testing.

**Step 1: Add Haven to DEBUG_OPTIONS**

In `src/utils/debug_menu.rs`, add to the `DEBUG_OPTIONS` array:

```rust
pub const DEBUG_OPTIONS: &[&str] = &[
    "Trigger Dungeon",
    "Trigger Fishing",
    "Trigger Chess Challenge",
    "Trigger Morris Challenge",
    "Trigger Gomoku Challenge",
    "Trigger Minesweeper Challenge",
    "Trigger Rune Challenge",
    "Trigger Haven Discovery",  // Add this
];
```

**Step 2: Add trigger function**

Add the trigger function:

```rust
fn trigger_haven_discovery(haven: &mut Haven) -> &'static str {
    if haven.discovered {
        return "Haven already discovered!";
    }
    haven.discovered = true;
    "Haven discovered!"
}
```

**Step 3: Update trigger_selected**

The `trigger_selected` method needs access to Haven. Update its signature to accept `&mut Haven` and add the match arm:

```rust
pub fn trigger_selected(&mut self, state: &mut GameState, haven: &mut Haven) -> &'static str {
    let msg = match self.selected_index {
        // ... existing arms ...
        7 => trigger_haven_discovery(haven),
        _ => "Unknown option",
    };
    self.close();
    msg
}
```

**Step 4: Update imports**

Add to imports in `debug_menu.rs`:

```rust
use crate::haven::Haven;
```

**Step 5: Update callers in main.rs**

Update the call to `trigger_selected` in `main.rs` to pass `&mut haven`.

**Step 6: Run tests and check**

Run: `cargo test utils::debug_menu::tests -- --nocapture`

Expected: All tests pass (may need to update test to pass a Haven).

**Step 7: Commit**

```bash
git add src/utils/debug_menu.rs src/main.rs
git commit -m "feat(haven): add debug menu trigger for Haven discovery"
```

---

### Task 4: Integrate Haven Discovery into Game Loop

**Files:**
- Modify: `src/main.rs`

**Context:** The game loop in `main.rs` calls `game_tick()` every 100ms. Haven discovery needs its own independent RNG roll, similar to challenge discovery. The Haven must be loaded at startup and passed through the game loop.

**Step 1: Load Haven at startup**

In `main.rs`, after `CharacterManager::new()`, add Haven loading:

```rust
use crate::haven::{load_haven, save_haven, try_discover_haven, Haven};

let mut haven = load_haven();
```

**Step 2: Add Haven discovery to game_tick**

The `game_tick` function needs access to the Haven. Two approaches:
- Pass `&mut Haven` as a parameter to `game_tick`
- Or add discovery as a separate call after `game_tick` in the main loop

Preferred: add it after `game_tick` in the main loop (around line 1135), keeping `game_tick` signature unchanged:

```rust
// After game_tick call, add Haven discovery
if !haven.discovered
    && state.prestige_rank >= 10
    && state.active_dungeon.is_none()
    && state.active_fishing.is_none()
    && state.active_chess.is_none()
    && state.active_morris.is_none()
    && state.active_gomoku.is_none()
    && state.active_minesweeper.is_none()
{
    let mut rng = rand::thread_rng();
    if try_discover_haven(&mut haven, state.prestige_rank, &mut rng) {
        save_haven(&haven).ok();
        showing_haven_discovery = true;
    }
}
```

**Step 3: Add discovery modal state variable**

Near the other state variables (around line 84):

```rust
let mut showing_haven_discovery = false;
```

**Step 4: Handle discovery modal input**

In the input handling section, before other key handlers (similar to prestige confirm at lines 639-665):

```rust
if showing_haven_discovery {
    if let KeyCode::Enter = key_event.code {
        showing_haven_discovery = false;
    }
    continue; // Block other input while modal is showing
}
```

**Step 5: Run `make check`**

Run: `make check`

Expected: Build succeeds, all tests pass. (UI rendering of the modal is Task 6.)

**Step 6: Commit**

```bash
git add src/main.rs
git commit -m "feat(haven): integrate discovery RNG into game loop"
```

---

### Task 5: Add Haven Key Binding & Screen State

**Files:**
- Modify: `src/main.rs`

**Step 1: Add Haven UI state variables**

Near other state variables:

```rust
let mut showing_haven = false;
let mut haven_selected_room: usize = 0; // Index into HavenRoomId::ALL
let mut haven_confirming_build = false;
```

**Step 2: Add `[H]` key handler in gameplay**

In the main game key handler section (around lines 1101-1127), add:

```rust
KeyCode::Char('h') | KeyCode::Char('H') => {
    if haven.discovered {
        showing_haven = true;
        haven_selected_room = 0;
        haven_confirming_build = false;
    }
}
```

**Step 3: Add Haven input handling**

Before the main game key handler, add Haven-specific input handling (when `showing_haven` is true):

```rust
if showing_haven {
    if haven_confirming_build {
        match key_event.code {
            KeyCode::Enter => {
                // Attempt build
                let room = crate::haven::HavenRoomId::ALL[haven_selected_room];
                if let Some((_tier, _p, _f)) = crate::haven::try_build_room(
                    room,
                    &mut haven,
                    &mut state.prestige_rank,
                    &mut state.fishing.rank,
                ) {
                    save_haven(&haven).ok();
                    character_manager.save_character(&state).ok();
                }
                haven_confirming_build = false;
            }
            KeyCode::Esc => {
                haven_confirming_build = false;
            }
            _ => {}
        }
    } else {
        match key_event.code {
            KeyCode::Up => {
                if haven_selected_room > 0 {
                    haven_selected_room -= 1;
                }
            }
            KeyCode::Down => {
                if haven_selected_room + 1 < haven::HavenRoomId::ALL.len() {
                    haven_selected_room += 1;
                }
            }
            KeyCode::Enter => {
                let room = crate::haven::HavenRoomId::ALL[haven_selected_room];
                if haven.can_build(room) && crate::haven::can_afford(
                    room, &haven, state.prestige_rank, state.fishing.rank
                ) {
                    haven_confirming_build = true;
                }
            }
            KeyCode::Esc => {
                showing_haven = false;
            }
            _ => {}
        }
    }
    continue; // Block other input while Haven is open
}
```

**Step 4: Add `[H]` to character select screen**

In the character select input handling (around lines 430-450), add:

```rust
KeyCode::Char('h') | KeyCode::Char('H') => {
    if haven.discovered {
        showing_haven = true;
        haven_selected_room = 0;
        haven_confirming_build = false;
    }
}
```

**Step 5: Run `make check`**

Run: `make check`

Expected: Builds and tests pass. Haven can be opened/closed but nothing renders yet.

**Step 6: Commit**

```bash
git add src/main.rs
git commit -m "feat(haven): add [H] key binding and Haven screen state"
```

---

### Task 6: Integrate Vault with Prestige

**Files:**
- Modify: `src/character/prestige.rs`
- Modify: `src/main.rs`

**Context:** The Vault capstone preserves equipped items through prestige. The prestige flow needs: (1) check Vault tier, (2) if > 0, show item selection screen, (3) preserve selected items.

**Step 1: Write test for Vault preservation**

Add to `src/character/prestige.rs` tests:

```rust
#[test]
fn test_perform_prestige_with_vault_items() {
    use crate::items::{AttributeBonuses, EquipmentSlot, Item, Rarity};
    use chrono::Utc;

    let mut state = GameState::new("Test Hero".to_string(), Utc::now().timestamp());

    // Equip a weapon
    let weapon = Item {
        slot: EquipmentSlot::Weapon,
        rarity: Rarity::Legendary,
        base_name: "Sword".to_string(),
        display_name: "Stormbreaker".to_string(),
        attributes: AttributeBonuses { str: 15, ..AttributeBonuses::new() },
        affixes: vec![],
    };
    state.equipment.set(EquipmentSlot::Weapon, Some(weapon));

    state.character_level = 10;

    // Prestige with 1 vault slot, preserving weapon
    let preserved = vec![EquipmentSlot::Weapon];
    perform_prestige_with_vault(&mut state, &preserved);

    assert_eq!(state.prestige_rank, 1);
    assert_eq!(state.character_level, 1);
    // Weapon should be preserved
    assert!(state.equipment.get(EquipmentSlot::Weapon).is_some());
    assert_eq!(state.equipment.get(EquipmentSlot::Weapon).unwrap().display_name, "Stormbreaker");
    // Other slots should be empty
    assert!(state.equipment.get(EquipmentSlot::Armor).is_none());
}
```

**Step 2: Implement `perform_prestige_with_vault`**

Add to `src/character/prestige.rs`:

```rust
/// Performs prestige with Vault item preservation.
/// `preserved_slots` contains the equipment slots to keep (limited by Vault tier externally).
pub fn perform_prestige_with_vault(state: &mut GameState, preserved_slots: &[crate::items::EquipmentSlot]) {
    use crate::equipment::Equipment;
    use crate::items::EquipmentSlot;

    if !can_prestige(state) {
        return;
    }

    // Save items from preserved slots before reset
    let mut saved_items: Vec<(EquipmentSlot, crate::items::Item)> = Vec::new();
    for slot in preserved_slots {
        if let Some(item) = state.equipment.get(*slot) {
            saved_items.push((*slot, item.clone()));
        }
    }

    // Normal prestige reset
    perform_prestige(state);

    // Restore preserved items
    for (slot, item) in saved_items {
        state.equipment.set(slot, Some(item));
    }
}
```

**Step 3: Run tests**

Run: `cargo test character::prestige::tests -- --nocapture`

Expected: All prestige tests pass including new vault test.

**Step 4: Wire up Vault in main.rs prestige flow**

In `main.rs`, modify the prestige confirmation handler. When the player confirms prestige, check `haven.vault_tier()`:

- If 0: call `perform_prestige(&mut state)` as before
- If > 0: set `showing_vault_selection = true` with the number of slots, let the player pick items, then call `perform_prestige_with_vault(&mut state, &selected_slots)`

This requires additional state variables and a vault selection UI (deferred to Task 6 for rendering).

**Step 5: Commit**

```bash
git add src/character/prestige.rs src/main.rs
git commit -m "feat(haven): add Vault item preservation on prestige"
```

---

### Task 7: Haven UI — Skill Tree Scene

**Files:**
- Create: `src/ui/haven_scene.rs`
- Modify: `src/ui/mod.rs`

**Context:** Follow the pattern of `challenge_menu_scene.rs` — a rendering function called from `main.rs` during the draw phase. The scene shows the skill tree on the left and room detail on the right.

**Step 1: Create `src/ui/haven_scene.rs`**

Implement three rendering functions:

1. `render_haven_tree(frame, area, haven, selected_room, game_state)` — main Haven screen with tree + detail panel
2. `render_haven_discovery_modal(frame, area)` — the discovery notification overlay
3. `render_build_confirmation(frame, area, room, haven, prestige_rank, fishing_rank)` — build/upgrade confirmation overlay

Reference `challenge_menu_scene.rs` for layout patterns (Block, Borders, List, Paragraph, Color/Style). Reference `dungeon_map.rs` for grid-based rendering if helpful.

Key rendering details from the design doc:
- Left panel: tree with `★··` tier indicators, `🔒` for locked, `· · ·` for available
- Right panel: room name, description, current/next bonus, cost, affordability
- Summary bar at top: "Active bonuses (X/13 rooms): +5% DMG +10% XP..."
- Selected room highlighted with `▶` cursor

**Step 2: Export from `src/ui/mod.rs`**

Add `pub mod haven_scene;` to `src/ui/mod.rs`.

**Step 3: Wire rendering into main.rs draw function**

In the `terminal.draw()` closure, add conditions:

```rust
if showing_haven_discovery {
    haven_scene::render_haven_discovery_modal(frame, frame.area());
} else if showing_haven {
    haven_scene::render_haven_tree(frame, frame.area(), &haven, haven_selected_room, &state);
    if haven_confirming_build {
        let room = haven::HavenRoomId::ALL[haven_selected_room];
        haven_scene::render_build_confirmation(frame, frame.area(), room, &haven, state.prestige_rank, state.fishing.rank);
    }
}
```

**Step 4: Run `make check`**

Run: `make check`

Expected: Builds, renders, all tests pass.

**Step 5: Commit**

```bash
git add src/ui/haven_scene.rs src/ui/mod.rs src/main.rs
git commit -m "feat(haven): add skill tree UI, discovery modal, and build confirmation"
```

---

### Task 8: Haven UI — Vault Item Selection Screen

**Files:**
- Modify: `src/ui/haven_scene.rs`
- Modify: `src/main.rs`

**Step 1: Add vault selection rendering**

Add to `haven_scene.rs`:

```rust
pub fn render_vault_selection(
    frame: &mut Frame,
    area: Rect,
    game_state: &GameState,
    vault_slots: u8,
    selected_index: usize,
    selected_items: &[EquipmentSlot],
)
```

Shows equipped items as a list. Player toggles selection with Enter. Shows "Selected: X/Y" counter. Space to confirm.

**Step 2: Add vault selection state to main.rs**

```rust
let mut showing_vault_selection = false;
let mut vault_selected_index: usize = 0;
let mut vault_selected_slots: Vec<items::EquipmentSlot> = Vec::new();
```

**Step 3: Wire vault selection input and rendering**

In prestige confirmation: if Vault tier > 0, set `showing_vault_selection = true` instead of immediately prestiging.

Handle vault selection input: Up/Down to navigate, Enter to toggle, Space to confirm and perform prestige.

**Step 4: Run `make check`**

Run: `make check`

Expected: All pass.

**Step 5: Commit**

```bash
git add src/ui/haven_scene.rs src/main.rs
git commit -m "feat(haven): add Vault item selection screen on prestige"
```

---

### Task 9: Apply Haven Bonuses to Game Systems

**Files:**
- Modify: `src/main.rs` (pass Haven bonuses into relevant systems)
- Possibly modify: `src/combat/logic.rs`, `src/core/game_logic.rs`, `src/fishing/logic.rs`, `src/items/drops.rs`

**Context:** Haven bonuses are percentages of base values, computed once on character load. Each system needs to read the relevant bonus. The cleanest approach is to compute a `HavenBonuses` struct on load and pass it where needed. Alternatively, pass `&Haven` to each system.

**Step 1: Create a computed bonuses helper**

Add to `src/haven/types.rs`:

```rust
/// Pre-computed Haven bonuses for efficient access during gameplay
#[derive(Debug, Clone, Default)]
pub struct HavenBonuses {
    pub damage_percent: f64,
    pub xp_gain_percent: f64,
    pub drop_rate_percent: f64,
    pub crit_chance_percent: f64,
    pub hp_regen_percent: f64,
    pub attack_interval_reduction: f64,
    pub offline_xp_percent: f64,
    pub challenge_discovery_percent: f64,
    pub fishing_timer_reduction: f64,
    pub fishing_rank_xp_percent: f64,
    pub item_rarity_percent: f64,
    pub hp_regen_delay_reduction: f64,
    pub vault_slots: u8,
}

impl Haven {
    pub fn compute_bonuses(&self) -> HavenBonuses {
        HavenBonuses {
            damage_percent: self.get_bonus(HavenBonusType::DamagePercent),
            xp_gain_percent: self.get_bonus(HavenBonusType::XpGainPercent),
            // ... etc for each field
            vault_slots: self.vault_tier(),
        }
    }
}
```

**Step 2: Apply bonuses to each system**

This is the most invasive task. For each bonus, find where the base value is used and apply the Haven modifier. Examples:

- **Damage:** In `src/combat/logic.rs` where physical/magic damage is calculated, multiply base by `(1.0 + haven_bonuses.damage_percent / 100.0)`
- **XP gain:** In `src/core/game_logic.rs` where XP is applied, multiply by `(1.0 + haven_bonuses.xp_gain_percent / 100.0)`
- **Drop rate:** In `src/items/drops.rs` where drop chance is calculated, add `haven_bonuses.drop_rate_percent / 100.0`
- **Offline XP:** In `src/core/game_logic.rs` offline progression calculation, multiply rate by Haven bonus
- **Challenge discovery:** In `src/challenges/menu.rs` `CHALLENGE_DISCOVERY_CHANCE`, multiply by Haven bonus

Each integration point should be minimal — a single multiplication or addition. The Haven struct is read-only during gameplay.

**Step 3: Write tests for bonus application**

Test that bonuses modify the expected values. For example, test that damage with Haven Armory T1 is 5% higher than without.

**Step 4: Run `make check`**

Run: `make check`

Expected: All pass.

**Step 5: Commit**

```bash
git add -A
git commit -m "feat(haven): apply Haven bonuses to combat, XP, drops, fishing, and discovery"
```

---

### Task 10: Character Select Haven Integration

**Files:**
- Modify: `src/ui/character_select.rs`

**Step 1: Add Haven summary to character details**

In `draw_character_details()` (around line 210), add a line showing Haven progress if discovered:

```rust
if haven.discovered {
    lines.push(Line::from(format!("Haven: {}/{} rooms built", haven.rooms_built(), haven.total_rooms())));
}
```

**Step 2: Add `[H] Haven` to controls bar**

In the controls section (around line 65), conditionally add `[H] Haven`:

```rust
let haven_text = if haven.discovered { "  [H] Haven" } else { "" };
```

Append to the controls line.

**Step 3: Run `make check`**

Run: `make check`

Expected: All pass.

**Step 4: Commit**

```bash
git add src/ui/character_select.rs src/main.rs
git commit -m "feat(haven): show Haven progress on character select screen"
```

---

### Task 11: Final Integration & Polish

**Files:**
- Modify: `src/main.rs` (ensure Haven is saved on autosave)
- Modify: Various files for edge cases

**Step 1: Save Haven on autosave**

In the autosave block (around line 1137), add:

```rust
save_haven(&haven).ok();
```

**Step 2: Add `[H]` to gameplay footer**

In `src/ui/stats_panel.rs` `draw_footer()`, add Haven indicator when discovered:

```rust
// Add Haven key hint if discovered
if haven.discovered {
    // Add "[H] Haven" to controls
}
```

This requires passing `&Haven` (or just `haven_discovered: bool`) to `draw_footer`.

**Step 3: Edge case testing**

- Test that Haven persists when game is closed and reopened (manual test)
- Test that building from character select vs gameplay both work
- Test that Vault selection works with 0, 1, 2, 3 equipped items
- Test that a character with insufficient ranks sees the "Cannot build" state

**Step 4: Run full CI checks**

Run: `make check`

Expected: All checks pass (format, clippy, tests, build, audit).

**Step 5: Commit**

```bash
git add -A
git commit -m "feat(haven): final integration, autosave, footer hint, and polish"
```

---

## Summary

| Task | Description | Key Files |
|------|-------------|-----------|
| 1 | Data structures & room definitions | `haven/mod.rs`, `haven/types.rs`, `lib.rs` |
| 2 | Build logic & persistence | `haven/logic.rs`, `haven/mod.rs` |
| 3 | Debug menu trigger | `utils/debug_menu.rs`, `main.rs` |
| 4 | Discovery in game loop | `main.rs` |
| 5 | Key binding & screen state | `main.rs` |
| 6 | Vault + prestige integration | `character/prestige.rs`, `main.rs` |
| 7 | Skill tree UI scene | `ui/haven_scene.rs`, `ui/mod.rs` |
| 8 | Vault item selection UI | `ui/haven_scene.rs`, `main.rs` |
| 9 | Apply bonuses to game systems | `combat/logic.rs`, `core/game_logic.rs`, etc. |
| 10 | Character select integration | `ui/character_select.rs` |
| 11 | Polish, autosave, edge cases | Various |
