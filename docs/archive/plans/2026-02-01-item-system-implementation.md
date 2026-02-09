# Item System Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement Diablo-like item system with periodic drops, auto-equip, and equipment visualization

**Architecture:** Four new modules (items, equipment, item_drops, item_names) integrate with existing GameState, derived_stats, and combat systems. Items drop on enemy kill, auto-equip via weighted scoring, and display in stats panel.

**Tech Stack:** Rust, Ratatui, Serde, Rand

---

## Task 1: Core Item Data Structures

**Files:**
- Create: `src/items.rs`
- Modify: `src/main.rs:1` (add mod declaration)

**Step 1: Write failing tests for item structures**

Create `src/items.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Weapon,
    Armor,
    Helmet,
    Gloves,
    Boots,
    Amulet,
    Ring,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Rarity {
    Common = 0,
    Magic = 1,
    Rare = 2,
    Epic = 3,
    Legendary = 4,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttributeBonuses {
    pub str: u32,
    pub dex: u32,
    pub con: u32,
    pub int: u32,
    pub wis: u32,
    pub cha: u32,
}

impl AttributeBonuses {
    pub fn new() -> Self {
        Self {
            str: 0,
            dex: 0,
            con: 0,
            int: 0,
            wis: 0,
            cha: 0,
        }
    }

    pub fn total(&self) -> u32 {
        self.str + self.dex + self.con + self.int + self.wis + self.cha
    }
}

impl Default for AttributeBonuses {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AffixType {
    // Damage-focused
    DamagePercent,
    CritChance,
    CritMultiplier,
    AttackSpeed,
    // Survivability
    HPBonus,
    DamageReduction,
    HPRegen,
    DamageReflection,
    // Progression
    XPGain,
    DropRate,
    PrestigeBonus,
    OfflineRate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Affix {
    pub affix_type: AffixType,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub slot: EquipmentSlot,
    pub rarity: Rarity,
    pub base_name: String,
    pub display_name: String,
    pub attributes: AttributeBonuses,
    pub affixes: Vec<Affix>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attribute_bonuses_total() {
        let attrs = AttributeBonuses {
            str: 5,
            dex: 3,
            con: 2,
            int: 1,
            wis: 0,
            cha: 4,
        };
        assert_eq!(attrs.total(), 15);
    }

    #[test]
    fn test_rarity_ordering() {
        assert!(Rarity::Common < Rarity::Magic);
        assert!(Rarity::Magic < Rarity::Rare);
        assert!(Rarity::Rare < Rarity::Epic);
        assert!(Rarity::Epic < Rarity::Legendary);
    }

    #[test]
    fn test_item_creation() {
        let item = Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Common,
            base_name: "Sword".to_string(),
            display_name: "Fine Sword".to_string(),
            attributes: AttributeBonuses {
                str: 2,
                ..AttributeBonuses::new()
            },
            affixes: vec![],
        };
        assert_eq!(item.slot, EquipmentSlot::Weapon);
        assert_eq!(item.rarity, Rarity::Common);
        assert_eq!(item.attributes.str, 2);
    }
}
```

**Step 2: Run tests to verify they pass**

Run: `cargo test --quiet items::tests`
Expected: 3 passing tests

**Step 3: Add module declaration to main.rs**

In `src/main.rs`, add after other mod declarations (around line 1):

```rust
mod items;
```

**Step 4: Verify compilation**

Run: `cargo build --quiet`
Expected: Successful compilation

**Step 5: Commit**

```bash
git add src/items.rs src/main.rs
git commit -m "feat(items): add core item data structures

- Add EquipmentSlot enum (7 slots)
- Add Rarity enum (5 tiers with ordering)
- Add AttributeBonuses struct with total() helper
- Add AffixType enum (12 types)
- Add Affix and Item structs
- All types derive Serialize/Deserialize for save support"
```

---

## Task 2: Equipment Management

**Files:**
- Create: `src/equipment.rs`
- Modify: `src/main.rs:1` (add mod declaration)

**Step 1: Write tests for equipment storage**

Create `src/equipment.rs`:

```rust
use crate::items::{EquipmentSlot, Item};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Equipment {
    pub weapon: Option<Item>,
    pub armor: Option<Item>,
    pub helmet: Option<Item>,
    pub gloves: Option<Item>,
    pub boots: Option<Item>,
    pub amulet: Option<Item>,
    pub ring: Option<Item>,
}

impl Equipment {
    pub fn new() -> Self {
        Self {
            weapon: None,
            armor: None,
            helmet: None,
            gloves: None,
            boots: None,
            amulet: None,
            ring: None,
        }
    }

    pub fn get(&self, slot: EquipmentSlot) -> &Option<Item> {
        match slot {
            EquipmentSlot::Weapon => &self.weapon,
            EquipmentSlot::Armor => &self.armor,
            EquipmentSlot::Helmet => &self.helmet,
            EquipmentSlot::Gloves => &self.gloves,
            EquipmentSlot::Boots => &self.boots,
            EquipmentSlot::Amulet => &self.amulet,
            EquipmentSlot::Ring => &self.ring,
        }
    }

    pub fn set(&mut self, slot: EquipmentSlot, item: Option<Item>) {
        match slot {
            EquipmentSlot::Weapon => self.weapon = item,
            EquipmentSlot::Armor => self.armor = item,
            EquipmentSlot::Helmet => self.helmet = item,
            EquipmentSlot::Gloves => self.gloves = item,
            EquipmentSlot::Boots => self.boots = item,
            EquipmentSlot::Amulet => self.amulet = item,
            EquipmentSlot::Ring => self.ring = item,
        }
    }

    pub fn iter_equipped(&self) -> impl Iterator<Item = &Item> {
        [
            &self.weapon,
            &self.armor,
            &self.helmet,
            &self.gloves,
            &self.boots,
            &self.amulet,
            &self.ring,
        ]
        .into_iter()
        .filter_map(|item| item.as_ref())
    }
}

impl Default for Equipment {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::{AttributeBonuses, Rarity};

    fn create_test_item(slot: EquipmentSlot) -> Item {
        Item {
            slot,
            rarity: Rarity::Common,
            base_name: "Test".to_string(),
            display_name: "Test Item".to_string(),
            attributes: AttributeBonuses::new(),
            affixes: vec![],
        }
    }

    #[test]
    fn test_equipment_starts_empty() {
        let eq = Equipment::new();
        assert!(eq.weapon.is_none());
        assert!(eq.armor.is_none());
        assert_eq!(eq.iter_equipped().count(), 0);
    }

    #[test]
    fn test_equipment_get_set() {
        let mut eq = Equipment::new();
        let weapon = create_test_item(EquipmentSlot::Weapon);

        eq.set(EquipmentSlot::Weapon, Some(weapon.clone()));
        assert_eq!(eq.get(EquipmentSlot::Weapon), &Some(weapon));
    }

    #[test]
    fn test_iter_equipped() {
        let mut eq = Equipment::new();
        eq.set(EquipmentSlot::Weapon, Some(create_test_item(EquipmentSlot::Weapon)));
        eq.set(EquipmentSlot::Armor, Some(create_test_item(EquipmentSlot::Armor)));

        assert_eq!(eq.iter_equipped().count(), 2);
    }
}
```

**Step 2: Run tests**

Run: `cargo test --quiet equipment::tests`
Expected: 3 passing tests

**Step 3: Add module declaration**

In `src/main.rs`, add:

```rust
mod equipment;
```

**Step 4: Verify compilation**

Run: `cargo build --quiet`
Expected: Success

**Step 5: Commit**

```bash
git add src/equipment.rs src/main.rs
git commit -m "feat(equipment): add equipment management system

- Add Equipment struct with 7 optional slots
- Add get/set methods for slot access
- Add iter_equipped() for iterating over equipped items
- Tests for empty equipment, get/set, and iteration"
```

---

## Task 3: Item Name Generation

**Files:**
- Create: `src/item_names.rs`
- Modify: `src/main.rs:1` (add mod declaration)

**Step 1: Write tests and implementation for name generation**

Create `src/item_names.rs`:

```rust
use crate::items::{AffixType, EquipmentSlot, Item, Rarity};
use rand::Rng;

pub fn get_base_name(slot: EquipmentSlot) -> Vec<&'static str> {
    match slot {
        EquipmentSlot::Weapon => vec!["Sword", "Axe", "Mace", "Dagger", "Greatsword", "Spear"],
        EquipmentSlot::Armor => vec!["Leather Armor", "Chain Mail", "Plate Mail", "Scale Mail"],
        EquipmentSlot::Helmet => vec!["Cap", "Helm", "Crown", "Coif"],
        EquipmentSlot::Gloves => vec!["Gloves", "Gauntlets", "Mitts", "Handwraps"],
        EquipmentSlot::Boots => vec!["Boots", "Greaves", "Shoes", "Sabatons"],
        EquipmentSlot::Amulet => vec!["Amulet", "Pendant", "Necklace", "Talisman"],
        EquipmentSlot::Ring => vec!["Ring", "Band", "Circle", "Loop"],
    }
}

pub fn get_quality_prefix(rarity: Rarity) -> &'static str {
    match rarity {
        Rarity::Common => "",
        Rarity::Magic => "Fine",
        _ => "", // Rare+ uses procedural names
    }
}

pub fn get_affix_prefix(affix_type: AffixType) -> &'static str {
    match affix_type {
        AffixType::DamagePercent => "Cruel",
        AffixType::CritChance => "Deadly",
        AffixType::CritMultiplier => "Vicious",
        AffixType::AttackSpeed => "Swift",
        AffixType::HPBonus => "Sturdy",
        AffixType::DamageReduction => "Armored",
        AffixType::HPRegen => "Regenerating",
        AffixType::DamageReflection => "Thorned",
        AffixType::XPGain => "Wise",
        AffixType::DropRate => "Lucky",
        AffixType::PrestigeBonus => "Prestigious",
        AffixType::OfflineRate => "Timeless",
    }
}

pub fn get_affix_suffix(affix_type: AffixType) -> &'static str {
    match affix_type {
        AffixType::DamagePercent => "of Power",
        AffixType::CritChance => "of Precision",
        AffixType::CritMultiplier => "of Carnage",
        AffixType::AttackSpeed => "of Haste",
        AffixType::HPBonus => "of Vitality",
        AffixType::DamageReduction => "of Protection",
        AffixType::HPRegen => "of Renewal",
        AffixType::DamageReflection => "of Thorns",
        AffixType::XPGain => "of Learning",
        AffixType::DropRate => "of Fortune",
        AffixType::PrestigeBonus => "of Glory",
        AffixType::OfflineRate => "of Eternity",
    }
}

pub fn generate_display_name(item: &Item) -> String {
    let mut rng = rand::thread_rng();
    let base_names = get_base_name(item.slot);
    let base = base_names[rng.gen_range(0..base_names.len())];

    match item.rarity {
        Rarity::Common => base.to_string(),
        Rarity::Magic => {
            let prefix = get_quality_prefix(item.rarity);
            format!("{} {}", prefix, base)
        }
        Rarity::Rare | Rarity::Epic | Rarity::Legendary => {
            // Use first affix for naming (if any)
            if let Some(first_affix) = item.affixes.first() {
                let use_prefix = rng.gen_bool(0.5);
                if use_prefix {
                    let prefix = get_affix_prefix(first_affix.affix_type);
                    format!("{} {}", prefix, base)
                } else {
                    let suffix = get_affix_suffix(first_affix.affix_type);
                    format!("{} {}", base, suffix)
                }
            } else {
                base.to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::{Affix, AttributeBonuses};

    #[test]
    fn test_common_item_name() {
        let item = Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Common,
            base_name: "Sword".to_string(),
            display_name: String::new(),
            attributes: AttributeBonuses::new(),
            affixes: vec![],
        };
        let name = generate_display_name(&item);
        // Should be just base name
        assert!(!name.is_empty());
        assert!(!name.contains("Fine"));
    }

    #[test]
    fn test_magic_item_name() {
        let item = Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Magic,
            base_name: "Sword".to_string(),
            display_name: String::new(),
            attributes: AttributeBonuses::new(),
            affixes: vec![],
        };
        let name = generate_display_name(&item);
        assert!(name.starts_with("Fine"));
    }

    #[test]
    fn test_rare_item_name_with_affix() {
        let item = Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Rare,
            base_name: "Sword".to_string(),
            display_name: String::new(),
            attributes: AttributeBonuses::new(),
            affixes: vec![Affix {
                affix_type: AffixType::DamagePercent,
                value: 15.0,
            }],
        };
        let name = generate_display_name(&item);
        // Should contain either "Cruel" or "of Power"
        assert!(name.contains("Cruel") || name.contains("of Power"));
    }

    #[test]
    fn test_base_names_exist_for_all_slots() {
        let slots = [
            EquipmentSlot::Weapon,
            EquipmentSlot::Armor,
            EquipmentSlot::Helmet,
            EquipmentSlot::Gloves,
            EquipmentSlot::Boots,
            EquipmentSlot::Amulet,
            EquipmentSlot::Ring,
        ];
        for slot in slots {
            let names = get_base_name(slot);
            assert!(!names.is_empty());
        }
    }
}
```

**Step 2: Run tests**

Run: `cargo test --quiet item_names::tests`
Expected: 4 passing tests

**Step 3: Add module declaration**

In `src/main.rs`:

```rust
mod item_names;
```

**Step 4: Verify compilation**

Run: `cargo build --quiet`
Expected: Success

**Step 5: Commit**

```bash
git add src/item_names.rs src/main.rs
git commit -m "feat(items): add procedural name generation

- Base names for all 7 equipment slots
- Quality prefixes for Magic items (\"Fine\")
- Affix-based prefixes/suffixes for Rare+ items
- generate_display_name() combines rarity + affixes
- Tests for Common, Magic, Rare naming patterns"
```

---

## Task 4: Item Generation Logic

**Files:**
- Create: `src/item_generation.rs`
- Modify: `src/main.rs:1` (add mod declaration)

**Step 1: Write tests for item generation**

Create `src/item_generation.rs`:

```rust
use crate::items::{Affix, AffixType, AttributeBonuses, EquipmentSlot, Item, Rarity};
use crate::item_names::generate_display_name;
use rand::Rng;

pub fn generate_item(slot: EquipmentSlot, rarity: Rarity, _player_level: u32) -> Item {
    let mut rng = rand::thread_rng();

    // Generate attribute bonuses based on rarity
    let attributes = generate_attributes(rarity, &mut rng);

    // Generate affixes based on rarity
    let affixes = generate_affixes(rarity, &mut rng);

    let mut item = Item {
        slot,
        rarity,
        base_name: String::new(), // Will be set by display name
        display_name: String::new(),
        attributes,
        affixes,
    };

    item.display_name = generate_display_name(&item);
    item.base_name = item.display_name.clone();

    item
}

fn generate_attributes(rarity: Rarity, rng: &mut impl Rng) -> AttributeBonuses {
    let (min, max) = match rarity {
        Rarity::Common => (1, 2),
        Rarity::Magic => (2, 4),
        Rarity::Rare => (3, 6),
        Rarity::Epic => (5, 10),
        Rarity::Legendary => (8, 15),
    };

    // Pick a random number of attributes to boost (1-3)
    let num_attrs = rng.gen_range(1..=3);
    let mut attrs = AttributeBonuses::new();

    for _ in 0..num_attrs {
        let value = rng.gen_range(min..=max);
        match rng.gen_range(0..6) {
            0 => attrs.str += value,
            1 => attrs.dex += value,
            2 => attrs.con += value,
            3 => attrs.int += value,
            4 => attrs.wis += value,
            5 => attrs.cha += value,
            _ => unreachable!(),
        }
    }

    attrs
}

fn generate_affixes(rarity: Rarity, rng: &mut impl Rng) -> Vec<Affix> {
    let count = match rarity {
        Rarity::Common => 0,
        Rarity::Magic => 1,
        Rarity::Rare => rng.gen_range(2..=3),
        Rarity::Epic => rng.gen_range(3..=4),
        Rarity::Legendary => rng.gen_range(4..=5),
    };

    let mut affixes = Vec::new();
    let all_affix_types = [
        AffixType::DamagePercent,
        AffixType::CritChance,
        AffixType::CritMultiplier,
        AffixType::AttackSpeed,
        AffixType::HPBonus,
        AffixType::DamageReduction,
        AffixType::HPRegen,
        AffixType::DamageReflection,
        AffixType::XPGain,
        AffixType::DropRate,
        AffixType::PrestigeBonus,
        AffixType::OfflineRate,
    ];

    for _ in 0..count {
        let affix_type = all_affix_types[rng.gen_range(0..all_affix_types.len())];
        let value = generate_affix_value(affix_type, rarity, rng);
        affixes.push(Affix { affix_type, value });
    }

    affixes
}

fn generate_affix_value(affix_type: AffixType, rarity: Rarity, rng: &mut impl Rng) -> f64 {
    let (min, max) = match rarity {
        Rarity::Common => (0.0, 0.0),
        Rarity::Magic => (5.0, 10.0),
        Rarity::Rare => (10.0, 20.0),
        Rarity::Epic => (15.0, 30.0),
        Rarity::Legendary => (25.0, 50.0),
    };

    // Some affixes use different ranges
    match affix_type {
        AffixType::HPBonus => {
            // Flat HP bonus
            let (hp_min, hp_max) = match rarity {
                Rarity::Magic => (10.0, 30.0),
                Rarity::Rare => (30.0, 60.0),
                Rarity::Epic => (50.0, 100.0),
                Rarity::Legendary => (80.0, 150.0),
                _ => (0.0, 0.0),
            };
            rng.gen_range(hp_min..=hp_max)
        }
        _ => rng.gen_range(min..=max),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_common_item() {
        let item = generate_item(EquipmentSlot::Weapon, Rarity::Common, 1);
        assert_eq!(item.rarity, Rarity::Common);
        assert_eq!(item.slot, EquipmentSlot::Weapon);
        assert_eq!(item.affixes.len(), 0);
        assert!(item.attributes.total() > 0);
    }

    #[test]
    fn test_generate_magic_item_has_affix() {
        let item = generate_item(EquipmentSlot::Armor, Rarity::Magic, 5);
        assert_eq!(item.rarity, Rarity::Magic);
        assert_eq!(item.affixes.len(), 1);
    }

    #[test]
    fn test_generate_rare_item_has_multiple_affixes() {
        let item = generate_item(EquipmentSlot::Helmet, Rarity::Rare, 10);
        assert_eq!(item.rarity, Rarity::Rare);
        assert!(item.affixes.len() >= 2 && item.affixes.len() <= 3);
    }

    #[test]
    fn test_generate_legendary_item() {
        let item = generate_item(EquipmentSlot::Weapon, Rarity::Legendary, 20);
        assert_eq!(item.rarity, Rarity::Legendary);
        assert!(item.affixes.len() >= 4 && item.affixes.len() <= 5);
        assert!(item.attributes.total() >= 8);
    }

    #[test]
    fn test_item_has_display_name() {
        let item = generate_item(EquipmentSlot::Weapon, Rarity::Magic, 5);
        assert!(!item.display_name.is_empty());
    }
}
```

**Step 2: Run tests**

Run: `cargo test --quiet item_generation::tests`
Expected: 5 passing tests

**Step 3: Add module declaration**

In `src/main.rs`:

```rust
mod item_generation;
```

**Step 4: Verify compilation**

Run: `cargo build --quiet`
Expected: Success

**Step 5: Commit**

```bash
git add src/item_generation.rs src/main.rs
git commit -m "feat(items): add item generation with rarity-based stats

- generate_item() creates items with rarity-appropriate stats
- Attribute bonuses scale with rarity (1-2 for Common, 8-15 for Legendary)
- Affix counts match rarity (0 for Common, 4-5 for Legendary)
- Affix values scale with rarity (5-10% for Magic, 25-50% for Legendary)
- Tests verify rarity constraints and affix counts"
```

---

## Task 5: Item Drop System

**Files:**
- Create: `src/item_drops.rs`
- Modify: `src/main.rs:1` (add mod declaration)

**Step 1: Write tests for drop system**

Create `src/item_drops.rs`:

```rust
use crate::game_state::GameState;
use crate::item_generation::generate_item;
use crate::items::{EquipmentSlot, Item, Rarity};
use rand::Rng;

pub fn try_drop_item(game_state: &GameState) -> Option<Item> {
    let mut rng = rand::thread_rng();

    // Calculate drop chance: 30% base + 5% per prestige rank
    let drop_chance = 0.30 + (game_state.prestige_rank as f64 * 0.05);

    if rng.gen::<f64>() > drop_chance {
        return None;
    }

    // Roll rarity based on prestige rank
    let rarity = roll_rarity(game_state.prestige_rank, &mut rng);

    // Roll random equipment slot
    let slot = roll_random_slot(&mut rng);

    // Generate item
    Some(generate_item(slot, rarity, game_state.level))
}

fn roll_rarity(prestige_rank: u32, rng: &mut impl Rng) -> Rarity {
    let roll = rng.gen::<f64>();

    match prestige_rank {
        0..=1 => {
            // Bronze: 60% Common, 30% Magic, 10% Rare
            if roll < 0.60 {
                Rarity::Common
            } else if roll < 0.90 {
                Rarity::Magic
            } else {
                Rarity::Rare
            }
        }
        2..=3 => {
            // Silver: 30% Common, 40% Magic, 25% Rare, 5% Epic
            if roll < 0.30 {
                Rarity::Common
            } else if roll < 0.70 {
                Rarity::Magic
            } else if roll < 0.95 {
                Rarity::Rare
            } else {
                Rarity::Epic
            }
        }
        4..=5 => {
            // Gold: 15% Common, 30% Magic, 40% Rare, 13% Epic, 2% Legendary
            if roll < 0.15 {
                Rarity::Common
            } else if roll < 0.45 {
                Rarity::Magic
            } else if roll < 0.85 {
                Rarity::Rare
            } else if roll < 0.98 {
                Rarity::Epic
            } else {
                Rarity::Legendary
            }
        }
        _ => {
            // Platinum+: 10% Common, 20% Magic, 35% Rare, 25% Epic, 10% Legendary
            if roll < 0.10 {
                Rarity::Common
            } else if roll < 0.30 {
                Rarity::Magic
            } else if roll < 0.65 {
                Rarity::Rare
            } else if roll < 0.90 {
                Rarity::Epic
            } else {
                Rarity::Legendary
            }
        }
    }
}

fn roll_random_slot(rng: &mut impl Rng) -> EquipmentSlot {
    match rng.gen_range(0..7) {
        0 => EquipmentSlot::Weapon,
        1 => EquipmentSlot::Armor,
        2 => EquipmentSlot::Helmet,
        3 => EquipmentSlot::Gloves,
        4 => EquipmentSlot::Boots,
        5 => EquipmentSlot::Amulet,
        6 => EquipmentSlot::Ring,
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_roll_rarity_bronze_prestige() {
        // Test Bronze prestige (0-1) distribution
        let mut common = 0;
        let mut magic = 0;
        let mut rare = 0;

        for _ in 0..1000 {
            let mut rng = rand::thread_rng();
            match roll_rarity(0, &mut rng) {
                Rarity::Common => common += 1,
                Rarity::Magic => magic += 1,
                Rarity::Rare => rare += 1,
                _ => {}
            }
        }

        // Rough distribution check (should be ~60%, 30%, 10%)
        assert!(common > 500); // At least 50%
        assert!(magic > 200); // At least 20%
        assert!(rare > 0); // Some rares
    }

    #[test]
    fn test_roll_rarity_platinum_can_drop_legendary() {
        // Test Platinum+ can drop legendary
        let mut found_legendary = false;
        for _ in 0..1000 {
            let mut rng = rand::thread_rng();
            if roll_rarity(6, &mut rng) == Rarity::Legendary {
                found_legendary = true;
                break;
            }
        }
        assert!(found_legendary);
    }

    #[test]
    fn test_roll_random_slot_coverage() {
        let mut slots_seen = std::collections::HashSet::new();
        let mut rng = rand::thread_rng();

        for _ in 0..100 {
            slots_seen.insert(format!("{:?}", roll_random_slot(&mut rng)));
        }

        // Should see most/all slots in 100 rolls
        assert!(slots_seen.len() >= 5);
    }

    #[test]
    fn test_try_drop_item_respects_prestige() {
        let mut game_state = GameState::new(Utc::now().timestamp());

        // With prestige 0, should get some drops
        let mut drops = 0;
        for _ in 0..100 {
            if try_drop_item(&game_state).is_some() {
                drops += 1;
            }
        }
        // ~30% drop rate, so expect 20-40 drops
        assert!(drops > 15 && drops < 50);

        // Higher prestige should increase drops
        game_state.prestige_rank = 4; // 30% + 20% = 50% drop rate
        let mut high_prestige_drops = 0;
        for _ in 0..100 {
            if try_drop_item(&game_state).is_some() {
                high_prestige_drops += 1;
            }
        }
        assert!(high_prestige_drops > drops); // Should be noticeably higher
    }
}
```

**Step 2: Run tests**

Run: `cargo test --quiet item_drops::tests`
Expected: 4 passing tests

**Step 3: Add module declaration**

In `src/main.rs`:

```rust
mod item_drops;
```

**Step 4: Verify compilation**

Run: `cargo build --quiet`
Expected: Success

**Step 5: Commit**

```bash
git add src/item_drops.rs src/main.rs
git commit -m "feat(items): add prestige-scaled drop system

- try_drop_item() rolls for drops with 30% + 5% per prestige
- roll_rarity() distributes rarities by prestige tier
- Bronze: 60% Common, 30% Magic, 10% Rare
- Silver: adds 5% Epic chance
- Gold: adds 2% Legendary chance
- Platinum+: 10% Legendary chance
- Tests verify distribution and prestige scaling"
```

---

## Task 6: Item Scoring and Auto-Equip

**Files:**
- Create: `src/item_scoring.rs`
- Modify: `src/main.rs:1` (add mod declaration)

**Step 1: Write tests for scoring system**

Create `src/item_scoring.rs`:

```rust
use crate::equipment::Equipment;
use crate::game_state::GameState;
use crate::items::{AffixType, AttributeBonuses, Item};

pub fn score_item(item: &Item, game_state: &GameState) -> f64 {
    let mut score = 0.0;

    // Calculate attribute weights based on current character build
    let weights = calculate_attribute_weights(game_state);

    // Score attributes
    score += item.attributes.str as f64 * weights.str as f64;
    score += item.attributes.dex as f64 * weights.dex as f64;
    score += item.attributes.con as f64 * weights.con as f64;
    score += item.attributes.int as f64 * weights.int as f64;
    score += item.attributes.wis as f64 * weights.wis as f64;
    score += item.attributes.cha as f64 * weights.cha as f64;

    // Score affixes with different weights
    for affix in &item.affixes {
        let affix_score = match affix.affix_type {
            AffixType::DamagePercent => affix.value * 2.0,
            AffixType::CritChance => affix.value * 1.5,
            AffixType::CritMultiplier => affix.value * 1.5,
            AffixType::AttackSpeed => affix.value * 1.2,
            AffixType::HPBonus => affix.value * 0.5, // Flat HP less valuable
            AffixType::DamageReduction => affix.value * 1.3,
            AffixType::HPRegen => affix.value * 1.0,
            AffixType::DamageReflection => affix.value * 0.8,
            AffixType::XPGain => affix.value * 1.0,
            AffixType::DropRate => affix.value * 0.5,
            AffixType::PrestigeBonus => affix.value * 0.8,
            AffixType::OfflineRate => affix.value * 0.5,
        };
        score += affix_score;
    }

    score
}

fn calculate_attribute_weights(game_state: &GameState) -> AttributeBonuses {
    // Weight attributes based on current values (specialization bonus)
    // Higher existing attributes get higher weights
    let attrs = &game_state.attributes;

    let total = (attrs.str + attrs.dex + attrs.con + attrs.int + attrs.wis + attrs.cha).max(1);

    AttributeBonuses {
        str: 1 + (attrs.str * 100 / total),
        dex: 1 + (attrs.dex * 100 / total),
        con: 1 + (attrs.con * 100 / total),
        int: 1 + (attrs.int * 100 / total),
        wis: 1 + (attrs.wis * 100 / total),
        cha: 1 + (attrs.cha * 100 / total),
    }
}

pub fn auto_equip_if_better(item: Item, game_state: &mut GameState) -> bool {
    let new_score = score_item(&item, game_state);

    let should_equip = if let Some(current) = game_state.equipment.get(item.slot) {
        let current_score = score_item(current, game_state);
        new_score > current_score
    } else {
        // Empty slot, always equip
        true
    };

    if should_equip {
        game_state.equipment.set(item.slot, Some(item));
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::{Affix, EquipmentSlot, Rarity};
    use chrono::Utc;

    fn create_test_item(slot: EquipmentSlot, rarity: Rarity, str_bonus: u32) -> Item {
        Item {
            slot,
            rarity,
            base_name: "Test".to_string(),
            display_name: "Test Item".to_string(),
            attributes: AttributeBonuses {
                str: str_bonus,
                ..AttributeBonuses::new()
            },
            affixes: vec![],
        }
    }

    #[test]
    fn test_score_item_with_attributes() {
        let game_state = GameState::new(Utc::now().timestamp());
        let item = create_test_item(EquipmentSlot::Weapon, Rarity::Common, 5);

        let score = score_item(&item, &game_state);
        assert!(score > 0.0);
    }

    #[test]
    fn test_score_item_with_affixes() {
        let game_state = GameState::new(Utc::now().timestamp());
        let item = Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Magic,
            base_name: "Test".to_string(),
            display_name: "Test Item".to_string(),
            attributes: AttributeBonuses::new(),
            affixes: vec![Affix {
                affix_type: AffixType::DamagePercent,
                value: 15.0,
            }],
        };

        let score = score_item(&item, &game_state);
        // DamagePercent has 2.0 weight, so 15 * 2 = 30
        assert!(score >= 30.0);
    }

    #[test]
    fn test_auto_equip_empty_slot() {
        let mut game_state = GameState::new(Utc::now().timestamp());
        let item = create_test_item(EquipmentSlot::Weapon, Rarity::Common, 2);

        let equipped = auto_equip_if_better(item, &mut game_state);
        assert!(equipped);
        assert!(game_state.equipment.get(EquipmentSlot::Weapon).is_some());
    }

    #[test]
    fn test_auto_equip_better_item() {
        let mut game_state = GameState::new(Utc::now().timestamp());

        // Equip weak item
        let weak = create_test_item(EquipmentSlot::Weapon, Rarity::Common, 1);
        auto_equip_if_better(weak, &mut game_state);

        // Try stronger item
        let strong = create_test_item(EquipmentSlot::Weapon, Rarity::Rare, 10);
        let equipped = auto_equip_if_better(strong.clone(), &mut game_state);

        assert!(equipped);
        assert_eq!(
            game_state
                .equipment
                .get(EquipmentSlot::Weapon)
                .as_ref()
                .unwrap()
                .attributes
                .str,
            10
        );
    }

    #[test]
    fn test_auto_equip_rejects_worse_item() {
        let mut game_state = GameState::new(Utc::now().timestamp());

        // Equip strong item
        let strong = create_test_item(EquipmentSlot::Weapon, Rarity::Rare, 10);
        auto_equip_if_better(strong, &mut game_state);

        // Try weaker item
        let weak = create_test_item(EquipmentSlot::Weapon, Rarity::Common, 1);
        let equipped = auto_equip_if_better(weak, &mut game_state);

        assert!(!equipped);
        assert_eq!(
            game_state
                .equipment
                .get(EquipmentSlot::Weapon)
                .as_ref()
                .unwrap()
                .attributes
                .str,
            10
        );
    }
}
```

**Step 2: Run tests**

Run: `cargo test --quiet item_scoring::tests`
Expected: 5 passing tests

**Step 3: Add module declaration**

In `src/main.rs`:

```rust
mod item_scoring;
```

**Step 4: Verify compilation**

Run: `cargo build --quiet`
Expected: Success

**Step 5: Commit**

```bash
git add src/item_scoring.rs src/main.rs
git commit -m "feat(items): add smart weighted scoring and auto-equip

- score_item() calculates item value based on attributes + affixes
- Attributes weighted by character build (specialization bonus)
- Affixes weighted by type (damage 2.0x, XP 1.0x, drop rate 0.5x)
- auto_equip_if_better() replaces items only if score improves
- Empty slots always equip first item
- Tests verify scoring, equip, upgrade, and rejection logic"
```

---

## Task 7: Integrate Equipment into GameState

**Files:**
- Modify: `src/game_state.rs:1-50` (add equipment field)

**Step 1: Add equipment field to GameState**

In `src/game_state.rs`, add import at top:

```rust
use crate::equipment::Equipment;
```

Then add field to `GameState` struct (around line 15):

```rust
pub equipment: Equipment,
```

In `GameState::new()` method (around line 30), add:

```rust
equipment: Equipment::new(),
```

**Step 2: Run tests to verify**

Run: `cargo test --quiet game_state::tests`
Expected: All existing tests pass

**Step 3: Verify full test suite**

Run: `cargo test --quiet`
Expected: All tests pass (including new item system tests)

**Step 4: Commit**

```bash
git add src/game_state.rs
git commit -m "feat(game): integrate equipment into GameState

- Add equipment: Equipment field to GameState
- Initialize with Equipment::new() in constructor
- Equipment persists with save/load via Serialize/Deserialize"
```

---

## Task 8: Update Derived Stats with Equipment Bonuses

**Files:**
- Modify: `src/derived_stats.rs:30-80` (add equipment parameter and calculations)

**Step 1: Write tests for equipment-modified derived stats**

In `src/derived_stats.rs`, update the function signature and add equipment integration.

First, add the test at the end of the file (in the `#[cfg(test)]` section):

```rust
#[test]
fn test_derived_stats_with_equipment() {
    use crate::equipment::Equipment;
    use crate::items::{AttributeBonuses, EquipmentSlot, Item, Rarity};

    let attrs = Attributes {
        str: 10,
        dex: 10,
        con: 10,
        int: 10,
        wis: 10,
        cha: 10,
    };

    // No equipment
    let stats_no_eq = calculate_derived_stats(&attrs, 0, &Equipment::new());
    let base_damage = stats_no_eq.damage;

    // With +5 STR weapon
    let mut equipment = Equipment::new();
    let weapon = Item {
        slot: EquipmentSlot::Weapon,
        rarity: Rarity::Common,
        base_name: "Sword".to_string(),
        display_name: "Test Sword".to_string(),
        attributes: AttributeBonuses {
            str: 5,
            ..AttributeBonuses::new()
        },
        affixes: vec![],
    };
    equipment.set(EquipmentSlot::Weapon, Some(weapon));

    let stats_with_eq = calculate_derived_stats(&attrs, 0, &equipment);
    // STR 10 -> 15 means modifier goes from 0 to +2, so +2 damage
    assert!(stats_with_eq.damage > base_damage);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --quiet derived_stats::tests::test_derived_stats_with_equipment`
Expected: Compilation error (function signature doesn't match)

**Step 3: Update calculate_derived_stats signature and implementation**

In `src/derived_stats.rs`, update the function (around line 30):

```rust
pub fn calculate_derived_stats(
    attributes: &Attributes,
    prestige_rank: u32,
    equipment: &Equipment,
) -> DerivedStats {
    // Add equipment attribute bonuses to base
    let mut total_attrs = *attributes;
    for item in equipment.iter_equipped() {
        total_attrs.str += item.attributes.str;
        total_attrs.dex += item.attributes.dex;
        total_attrs.con += item.attributes.con;
        total_attrs.int += item.attributes.int;
        total_attrs.wis += item.attributes.wis;
        total_attrs.cha += item.attributes.cha;
    }

    // Calculate modifiers from total attributes
    let str_mod = calculate_modifier(total_attrs.str);
    let dex_mod = calculate_modifier(total_attrs.dex);
    let con_mod = calculate_modifier(total_attrs.con);
    let int_mod = calculate_modifier(total_attrs.int);
    let wis_mod = calculate_modifier(total_attrs.wis);
    let cha_mod = calculate_modifier(total_attrs.cha);

    // ... rest of existing calculation logic stays the same ...
    // (keep the existing max_hp, damage, defense, crit_chance, xp_multiplier calculations)
```

Add import at top if not present:

```rust
use crate::equipment::Equipment;
```

**Step 4: Fix all call sites**

Update all calls to `calculate_derived_stats` to pass equipment. Find them with:

Run: `grep -rn "calculate_derived_stats" src/ --include="*.rs"`

Update each call site to pass `&game_state.equipment` as the third parameter.

**Step 5: Run tests**

Run: `cargo test --quiet`
Expected: All tests pass

**Step 6: Commit**

```bash
git add src/derived_stats.rs
git commit -m "feat(stats): include equipment bonuses in derived stats

- Add equipment parameter to calculate_derived_stats()
- Sum equipment attribute bonuses before calculating modifiers
- Equipment bonuses affect HP, damage, defense, crit, XP
- Update all call sites to pass equipment
- Test verifies equipment increases damage"
```

---

## Task 9: Apply Equipment Affixes to Derived Stats

**Files:**
- Modify: `src/derived_stats.rs:60-100` (add affix application)

**Step 1: Write test for affix effects**

In `src/derived_stats.rs` tests section, add:

```rust
#[test]
fn test_derived_stats_with_affixes() {
    use crate::equipment::Equipment;
    use crate::items::{Affix, AffixType, AttributeBonuses, EquipmentSlot, Item, Rarity};

    let attrs = Attributes {
        str: 15,
        dex: 10,
        con: 10,
        int: 10,
        wis: 10,
        cha: 10,
    };

    let mut equipment = Equipment::new();
    let weapon = Item {
        slot: EquipmentSlot::Weapon,
        rarity: Rarity::Rare,
        base_name: "Sword".to_string(),
        display_name: "Cruel Sword".to_string(),
        attributes: AttributeBonuses::new(),
        affixes: vec![Affix {
            affix_type: AffixType::DamagePercent,
            value: 20.0, // +20% damage
        }],
    };
    equipment.set(EquipmentSlot::Weapon, Some(weapon));

    let stats_no_affix = calculate_derived_stats(&attrs, 0, &Equipment::new());
    let stats_with_affix = calculate_derived_stats(&attrs, 0, &equipment);

    // Damage should be ~20% higher
    let expected_damage = (stats_no_affix.damage as f64 * 1.20) as u32;
    assert_eq!(stats_with_affix.damage, expected_damage);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --quiet derived_stats::tests::test_derived_stats_with_affixes`
Expected: FAIL (affixes not applied yet)

**Step 3: Implement affix application**

In `src/derived_stats.rs`, after calculating base stats but before returning, add:

```rust
    // Apply equipment affixes as multipliers/bonuses
    let mut hp_bonus: f64 = 0.0;
    let mut damage_mult: f64 = 1.0;
    let mut defense_mult: f64 = 1.0;
    let mut crit_bonus: f64 = 0.0;
    let mut xp_mult: f64 = 1.0;

    for item in equipment.iter_equipped() {
        for affix in &item.affixes {
            use crate::items::AffixType;
            match affix.affix_type {
                AffixType::DamagePercent => damage_mult *= 1.0 + (affix.value / 100.0),
                AffixType::CritChance => crit_bonus += affix.value,
                AffixType::HPBonus => hp_bonus += affix.value,
                AffixType::DamageReduction => defense_mult *= 1.0 + (affix.value / 100.0),
                AffixType::XPGain => xp_mult *= 1.0 + (affix.value / 100.0),
                // Other affixes don't affect derived stats directly
                _ => {}
            }
        }
    }

    // Apply multipliers to stats
    max_hp = ((max_hp as f64 + hp_bonus) as u32).max(1);
    damage = ((damage as f64 * damage_mult) as u32).max(1);
    defense = ((defense as f64 * defense_mult) as u32).max(0);
    crit_chance += crit_bonus;
    xp_multiplier *= xp_mult;

    DerivedStats {
        max_hp,
        damage,
        defense,
        crit_chance,
        xp_multiplier,
    }
```

**Step 4: Run tests**

Run: `cargo test --quiet derived_stats::tests`
Expected: All tests pass

**Step 5: Commit**

```bash
git add src/derived_stats.rs
git commit -m "feat(stats): apply equipment affixes to derived stats

- DamagePercent affix multiplies damage
- CritChance affix adds to crit chance
- HPBonus affix adds flat HP
- DamageReduction affix increases defense
- XPGain affix multiplies XP rate
- Test verifies +20% damage affix works correctly"
```

---

## Task 10: Integrate Item Drops into Combat

**Files:**
- Modify: `src/main.rs:185-203` (EnemyDied event handler)

**Step 1: Add item drop to enemy death handler**

In `src/main.rs`, find the `CombatEvent::EnemyDied` handler (around line 185) and update it:

```rust
CombatEvent::EnemyDied { xp_gained } => {
    // Add to combat log
    if let Some(enemy) = &game_state.combat_state.current_enemy {
        let message = format!("✨ {} defeated! +{} XP", enemy.name, xp_gained);
        game_state.combat_state.add_log_entry(message, false, true);
    }
    apply_tick_xp(game_state, xp_gained as f64);

    // Try to drop item
    use item_drops::try_drop_item;
    use item_scoring::auto_equip_if_better;

    if let Some(item) = try_drop_item(game_state) {
        let item_name = item.display_name.clone();
        let rarity = item.rarity;
        let equipped = auto_equip_if_better(item, game_state);

        let rarity_name = match rarity {
            items::Rarity::Common => "Common",
            items::Rarity::Magic => "Magic",
            items::Rarity::Rare => "Rare",
            items::Rarity::Epic => "Epic",
            items::Rarity::Legendary => "Legendary",
        };

        let stars = "⭐".repeat(rarity as usize + 1);
        let equipped_text = if equipped { " (equipped!)" } else { "" };

        let message = format!(
            "🎁 Found: {} [{}] {}{}",
            item_name, rarity_name, stars, equipped_text
        );
        game_state.combat_state.add_log_entry(message, false, true);
    }
}
```

**Step 2: Run game to test**

Run: `cargo run`
Expected: Game runs, items drop after kills and appear in combat log

**Step 3: Verify compilation**

Run: `cargo build --quiet`
Expected: Success

**Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat(combat): integrate item drops on enemy death

- Call try_drop_item() when enemy dies
- Auto-equip if item is better than current
- Add combat log entry showing item name, rarity, and equip status
- Rarity shown with star rating (⭐ to ⭐⭐⭐⭐⭐)"
```

---

## Task 11: Display Equipment in Stats Panel

**Files:**
- Modify: `src/ui/stats_panel.rs:50-150` (add equipment section)

**Step 1: Add equipment rendering function**

In `src/ui/stats_panel.rs`, add at the end of the file (before closing brace):

```rust
fn render_equipment_section(game_state: &GameState) -> Vec<Line<'static>> {
    use crate::items::EquipmentSlot;

    let mut lines = vec![];

    // Section header
    lines.push(Line::from(vec![Span::styled(
        "─── Equipment ───",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(""));

    // Render each slot
    let slots = [
        (EquipmentSlot::Weapon, "⚔️", "Weapon"),
        (EquipmentSlot::Armor, "🛡", "Armor"),
        (EquipmentSlot::Helmet, "🪖", "Helmet"),
        (EquipmentSlot::Gloves, "🧤", "Gloves"),
        (EquipmentSlot::Boots, "👢", "Boots"),
        (EquipmentSlot::Amulet, "📿", "Amulet"),
        (EquipmentSlot::Ring, "💍", "Ring"),
    ];

    for (slot, icon, name) in slots {
        if let Some(item) = game_state.equipment.get(slot) {
            // Line 1: Icon, slot name, item name, rarity
            let rarity_color = match item.rarity {
                crate::items::Rarity::Common => Color::White,
                crate::items::Rarity::Magic => Color::Blue,
                crate::items::Rarity::Rare => Color::Yellow,
                crate::items::Rarity::Epic => Color::Magenta,
                crate::items::Rarity::Legendary => Color::LightRed,
            };

            let rarity_name = match item.rarity {
                crate::items::Rarity::Common => "Common",
                crate::items::Rarity::Magic => "Magic",
                crate::items::Rarity::Rare => "Rare",
                crate::items::Rarity::Epic => "Epic",
                crate::items::Rarity::Legendary => "Legendary",
            };

            let stars = "⭐".repeat(item.rarity as usize + 1);

            lines.push(Line::from(vec![
                Span::raw(format!("{} {:8} ", icon, name)),
                Span::styled(
                    format!("{:30}", item.display_name),
                    Style::default().fg(rarity_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("[{}] {}", rarity_name, stars),
                    Style::default().fg(rarity_color),
                ),
            ]));

            // Line 2: Bonuses (indented)
            let mut bonuses = vec![];

            // Attributes
            if item.attributes.str > 0 {
                bonuses.push(format!("+{} STR", item.attributes.str));
            }
            if item.attributes.dex > 0 {
                bonuses.push(format!("+{} DEX", item.attributes.dex));
            }
            if item.attributes.con > 0 {
                bonuses.push(format!("+{} CON", item.attributes.con));
            }
            if item.attributes.int > 0 {
                bonuses.push(format!("+{} INT", item.attributes.int));
            }
            if item.attributes.wis > 0 {
                bonuses.push(format!("+{} WIS", item.attributes.wis));
            }
            if item.attributes.cha > 0 {
                bonuses.push(format!("+{} CHA", item.attributes.cha));
            }

            // Affixes
            for affix in &item.affixes {
                use crate::items::AffixType;
                let affix_text = match affix.affix_type {
                    AffixType::DamagePercent => format!("+{:.0}% Damage", affix.value),
                    AffixType::CritChance => format!("+{:.0}% Crit", affix.value),
                    AffixType::CritMultiplier => format!("+{:.0}% Crit Dmg", affix.value),
                    AffixType::AttackSpeed => format!("+{:.0}% Attack Speed", affix.value),
                    AffixType::HPBonus => format!("+{:.0} HP", affix.value),
                    AffixType::DamageReduction => format!("+{:.0}% Defense", affix.value),
                    AffixType::HPRegen => format!("+{:.0}% HP Regen", affix.value),
                    AffixType::DamageReflection => format!("+{:.0}% Reflect", affix.value),
                    AffixType::XPGain => format!("+{:.0}% XP", affix.value),
                    AffixType::DropRate => format!("+{:.0}% Drop Rate", affix.value),
                    AffixType::PrestigeBonus => format!("+{:.0}% Prestige", affix.value),
                    AffixType::OfflineRate => format!("+{:.0}% Offline", affix.value),
                };
                bonuses.push(affix_text);
            }

            if !bonuses.is_empty() {
                lines.push(Line::from(vec![Span::styled(
                    format!("            {}", bonuses.join(", ")),
                    Style::default().fg(Color::Gray),
                )]));
            }
        } else {
            // Empty slot
            lines.push(Line::from(vec![
                Span::raw(format!("{} {:8} ", icon, name)),
                Span::styled("[Empty]", Style::default().fg(Color::DarkGray)),
            ]));
        }
    }

    lines
}
```

**Step 2: Integrate into draw_stats_panel**

In `src/ui/stats_panel.rs`, find `draw_stats_panel` function and add equipment section. Update the constraints and add rendering:

Before the existing code, update layout constraints to make room:

```rust
let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(3),  // Title
        Constraint::Length(8),  // Attributes + Derived
        Constraint::Min(20),    // Equipment (NEW)
    ])
    .split(inner);
```

Then add after rendering derived stats:

```rust
// Render equipment
let equipment_lines = render_equipment_section(game_state);
let equipment_paragraph = Paragraph::new(equipment_lines);
frame.render_widget(equipment_paragraph, chunks[2]);
```

**Step 3: Run game to test UI**

Run: `cargo run`
Expected: Equipment shows in stats panel, updates when items equipped

**Step 4: Verify compilation**

Run: `cargo build --quiet`
Expected: Success

**Step 5: Commit**

```bash
git add src/ui/stats_panel.rs
git commit -m "feat(ui): display equipment in stats panel

- Add render_equipment_section() showing all 7 slots
- Each item shows: icon, name, rarity, star rating
- Second line shows attribute bonuses and affixes
- Empty slots show [Empty] in dark gray
- Rarity color-coded: White/Blue/Yellow/Magenta/Red
- Equipment section integrated into stats panel layout"
```

---

## Task 12: Equipment Persists Through Prestige

**Files:**
- Modify: `src/prestige.rs:50-80` (ensure equipment isn't reset)

**Step 1: Verify equipment survives prestige**

In `src/prestige.rs`, find `perform_prestige` function and verify it does NOT touch `game_state.equipment`.

Add test at end of file:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::equipment::Equipment;
    use crate::items::{AttributeBonuses, EquipmentSlot, Item, Rarity};
    use chrono::Utc;

    #[test]
    fn test_equipment_survives_prestige() {
        let mut game_state = crate::game_state::GameState::new(Utc::now().timestamp());

        // Equip an item
        let weapon = Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Rare,
            base_name: "Sword".to_string(),
            display_name: "Test Sword".to_string(),
            attributes: AttributeBonuses {
                str: 10,
                ..AttributeBonuses::new()
            },
            affixes: vec![],
        };
        game_state.equipment.set(EquipmentSlot::Weapon, Some(weapon.clone()));

        // Level up enough to prestige
        game_state.level = 10;

        // Perform prestige
        perform_prestige(&mut game_state);

        // Equipment should still be there
        assert!(game_state.equipment.get(EquipmentSlot::Weapon).is_some());
        assert_eq!(
            game_state
                .equipment
                .get(EquipmentSlot::Weapon)
                .as_ref()
                .unwrap()
                .attributes
                .str,
            10
        );
    }
}
```

**Step 2: Run test**

Run: `cargo test --quiet prestige::tests::test_equipment_survives_prestige`
Expected: PASS (equipment already persists, prestige doesn't touch it)

**Step 3: Commit**

```bash
git add src/prestige.rs
git commit -m "test(prestige): verify equipment survives prestige

- Add test confirming equipment persists through prestige
- Equipment is intentionally NOT reset (unlike attributes)
- Provides permanent progression across prestige cycles"
```

---

## Task 13: Save/Load Compatibility

**Files:**
- Test: Save/load with equipment works

**Step 1: Add integration test**

In `src/save_manager.rs`, add test:

```rust
#[test]
fn test_save_load_with_equipment() {
    use crate::equipment::Equipment;
    use crate::items::{AttributeBonuses, EquipmentSlot, Item, Rarity};

    let save_mgr = SaveManager::new_for_test().unwrap();

    let mut game_state = crate::game_state::GameState::new(chrono::Utc::now().timestamp());

    // Equip items
    let weapon = Item {
        slot: EquipmentSlot::Weapon,
        rarity: Rarity::Legendary,
        base_name: "Greatsword".to_string(),
        display_name: "Flaming Greatsword".to_string(),
        attributes: AttributeBonuses {
            str: 12,
            dex: 5,
            ..AttributeBonuses::new()
        },
        affixes: vec![],
    };
    game_state.equipment.set(EquipmentSlot::Weapon, Some(weapon));

    // Save
    save_mgr.save(&game_state).unwrap();

    // Load
    let loaded = save_mgr.load().unwrap();

    // Verify equipment loaded correctly
    assert!(loaded.equipment.get(EquipmentSlot::Weapon).is_some());
    let loaded_weapon = loaded.equipment.get(EquipmentSlot::Weapon).as_ref().unwrap();
    assert_eq!(loaded_weapon.display_name, "Flaming Greatsword");
    assert_eq!(loaded_weapon.attributes.str, 12);
    assert_eq!(loaded_weapon.rarity, Rarity::Legendary);
}
```

**Step 2: Run test**

Run: `cargo test --quiet save_manager::tests::test_save_load_with_equipment`
Expected: PASS (Equipment derives Serialize/Deserialize, so it works automatically)

**Step 3: Commit**

```bash
git add src/save_manager.rs
git commit -m "test(save): verify equipment saves and loads correctly

- Add test confirming equipment persists across save/load
- Equipment automatically serializes via Serde
- Loaded equipment matches saved state (rarity, stats, affixes)"
```

---

## Task 14: Run Full Test Suite and Format

**Files:**
- All

**Step 1: Run all tests**

Run: `cargo test --quiet`
Expected: All tests pass

**Step 2: Format code**

Run: `cargo fmt`
Expected: Code formatted

**Step 3: Run clippy**

Run: `cargo clippy --quiet -- -D warnings`
Expected: No warnings

**Step 4: Commit if any formatting changes**

```bash
git add -A
git commit -m "chore: format code and fix clippy warnings"
```

---

## Task 15: Update CLAUDE.md Documentation

**Files:**
- Modify: `CLAUDE.md:1-100` (add item system documentation)

**Step 1: Add item system section to CLAUDE.md**

In `CLAUDE.md`, add after the "Core Modules" section:

```markdown
### Item System (`src/items.rs`, `src/equipment.rs`, etc.)

- `items.rs` — Core item data structures (EquipmentSlot, Rarity, AttributeBonuses, Affix types, Item)
- `equipment.rs` — Equipment storage with 7 slots, get/set/iter methods
- `item_names.rs` — Procedural name generation (quality prefixes, affix-based names)
- `item_generation.rs` — Item generation with rarity-scaled stats and affixes
- `item_drops.rs` — Prestige-scaled drop system (30% + 5% per rank), rarity distribution
- `item_scoring.rs` — Smart weighted scoring for auto-equip (attributes + affixes)

**Item Rarities:**
- Common (0 affixes), Magic (1), Rare (2-3), Epic (3-4), Legendary (4-5)

**Equipment Slots:**
- Weapon, Armor, Helmet, Gloves, Boots, Amulet, Ring

**Auto-Equip:**
- Items scored by weighted attributes (favors current build specialization) + affixes
- Automatically replaces equipped item if new item scores higher
- Empty slots always equip first item
```

**Step 2: Update "Key Constants" section**

Add to constants:

```markdown
- Item drop rate: 30% base + 5% per prestige rank
- Affix weights: Damage (2.0x), Crit (1.5x), XP (1.0x), Drop Rate (0.5x)
```

**Step 3: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: add item system to CLAUDE.md

- Document item modules and their responsibilities
- List rarity tiers and equipment slots
- Explain auto-equip scoring system
- Add drop rate constants"
```

---

## Final Verification

**Step 1: Run full CI checks**

Run: `./scripts/ci-checks.sh`
Expected: All checks pass (format, clippy, tests, build, audit)

**Step 2: Play-test the game**

Run: `cargo run`

Test:
- Kill enemies and watch for item drops (~30-40% rate)
- Verify items appear in combat log
- Check equipment section in stats panel
- Verify items auto-equip
- Check that better items replace worse ones
- Save and reload, verify equipment persists
- Prestige and verify equipment persists

**Step 3: Final commit**

```bash
git add -A
git commit -m "feat: complete item system implementation

Full Diablo-like item system with:
- 7 equipment slots (Weapon, Armor, Helmet, Gloves, Boots, Amulet, Ring)
- 5 rarity tiers (Common → Legendary) with scaling stats
- Prestige-scaled drops (30% + 5% per rank)
- Smart weighted auto-equip favoring build specialization
- Equipment bonuses integrate with derived stats
- Equipment persists through save/load and prestige
- Full UI visualization in stats panel

Closes design doc: docs/plans/2026-02-01-item-system-design.md"
```

---

## Success Criteria Checklist

- [ ] Items drop after ~30-40% of kills (scales with prestige)
- [ ] Higher prestige = better rarity distribution
- [ ] Items automatically equip if better than current
- [ ] Equipment visible in stats panel with full details
- [ ] All bonuses correctly apply to combat
- [ ] System survives save/load
- [ ] System survives prestige
- [ ] UI remains readable and uncluttered
- [ ] All tests pass
- [ ] No clippy warnings
- [ ] Code formatted
- [ ] CLAUDE.md updated
