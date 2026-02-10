use crate::core::constants::{PRESTIGE_MULT_BASE_FACTOR, PRESTIGE_MULT_EXPONENT};
use crate::core::game_state::GameState;

/// Represents a prestige tier with its properties
#[derive(Debug, Clone)]
pub struct PrestigeTier {
    #[allow(dead_code)]
    pub rank: u32,
    pub name: &'static str,
    pub required_level: u32,
    pub multiplier: f64,
}

/// Gets the name for a prestige rank
fn get_prestige_name(rank: u32) -> &'static str {
    match rank {
        0 => "None",
        // Metals (1-4)
        1 => "Bronze",
        2 => "Silver",
        3 => "Gold",
        4 => "Platinum",
        // Gems (5-9)
        5 => "Diamond",
        6 => "Emerald",
        7 => "Sapphire",
        8 => "Ruby",
        9 => "Obsidian",
        // Cosmic (10-14)
        10 => "Celestial",
        11 => "Astral",
        12 => "Cosmic",
        13 => "Stellar",
        14 => "Galactic",
        // Divine (15-19)
        15 => "Transcendent",
        16 => "Divine",
        17 => "Exalted",
        18 => "Mythic",
        19 => "Legendary",
        // Eternal (20+)
        _ => "Eternal",
    }
}

/// Gets the prestige tier for a given rank
///
/// # Arguments
/// * `rank` - The prestige rank
///
/// # Returns
/// The PrestigeTier with name, required level, and multiplier
///
/// # Multiplier Formula
/// Uses diminishing returns: `1 + 0.5 * rank^0.7`
///
/// This provides:
/// - Strong early boost (+50% at P1)
/// - Tapering gains to prevent late-game trivialization
/// - Cycles get progressively longer, creating the "wall" feeling
///
/// See docs/plans/2026-02-03-prestige-multiplier-rebalance.md for details.
pub fn get_prestige_tier(rank: u32) -> PrestigeTier {
    // Diminishing returns formula: 1 + BASE_FACTOR * rank^EXPONENT
    // P1: 1.5x, P5: 2.5x, P10: 3.5x, P20: 5.1x, P30: 6.4x
    let multiplier = 1.0 + PRESTIGE_MULT_BASE_FACTOR * (rank as f64).powf(PRESTIGE_MULT_EXPONENT);

    let required_level = match rank {
        0 => 0,
        1 => 10,
        2 => 25,
        3 => 50,
        4 => 65,
        5 => 80,
        6 => 90,
        7 => 100,
        8 => 110,
        9 => 120,
        10 => 130,
        11 => 140,
        12 => 150,
        13 => 160,
        14 => 170,
        15 => 180,
        16 => 190,
        17 => 200,
        18 => 210,
        19 => 220,
        // 20+: continues at +15 per rank
        _ => 220 + (rank - 19) * 15,
    };

    PrestigeTier {
        rank,
        name: get_prestige_name(rank),
        required_level,
        multiplier,
    }
}

/// Gets the next prestige tier based on current rank
///
/// # Arguments
/// * `current_rank` - The player's current prestige rank
///
/// # Returns
/// The PrestigeTier for the next rank
pub fn get_next_prestige_tier(current_rank: u32) -> PrestigeTier {
    get_prestige_tier(current_rank + 1)
}

/// Checks if the player can prestige
///
/// # Arguments
/// * `state` - The current game state
///
/// # Returns
/// true if character level meets the required level for next prestige tier
pub fn can_prestige(state: &GameState) -> bool {
    let next_tier = get_next_prestige_tier(state.prestige_rank);
    state.character_level >= next_tier.required_level
}

/// Performs a prestige, resetting character progress and incrementing prestige rank
///
/// # Arguments
/// * `state` - The game state to modify
pub fn perform_prestige(state: &mut GameState) {
    use super::attributes::Attributes;
    use crate::combat::CombatState;
    use crate::items::Equipment;

    // Only prestige if eligible
    if !can_prestige(state) {
        return;
    }

    // Reset character to level 1, XP 0
    state.character_level = 1;
    state.character_xp = 0;

    // Reset attributes to base 10
    state.attributes = Attributes::new();

    // Reset equipment (complete wipe)
    state.equipment = Equipment::new();

    // Reset active dungeon
    state.active_dungeon = None;

    // Clear active fishing session (transient state)
    // Note: Fishing rank and progression (total_fish_caught, legendary_catches, etc.)
    // are intentionally preserved across prestige as a separate progression track
    state.active_fishing = None;

    // Clear any active minigame session
    state.active_minigame = None;

    // Reset combat state with base HP (50 for fresh attributes)
    state.combat_state = CombatState::new(50);

    // Increment prestige rank and total prestige count
    state.prestige_rank += 1;
    state.total_prestige_count += 1;

    // Reset zone progression but keep unlocks based on new prestige rank
    state
        .zone_progression
        .reset_for_prestige(state.prestige_rank);
}

/// Performs prestige with Vault item preservation.
/// `preserved_slots` contains the equipment slots to keep (limited by Vault tier externally).
pub fn perform_prestige_with_vault(
    state: &mut GameState,
    preserved_slots: &[crate::items::EquipmentSlot],
) {
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

/// Gets the adventurer rank based on average level
///
/// # Arguments
/// * `avg_level` - The average level across all stats
///
/// # Returns
/// A string describing the adventurer's rank
pub fn get_adventurer_rank(avg_level: u32) -> &'static str {
    match avg_level {
        0..=9 => "Novice",
        10..=24 => "Adept",
        25..=49 => "Master",
        50..=74 => "Grand Master",
        75..=99 => "Legend",
        _ => "Mythic",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::attributes::AttributeType;

    #[test]
    fn test_get_prestige_tier() {
        // Test rank 0 (None)
        let tier0 = get_prestige_tier(0);
        assert_eq!(tier0.rank, 0);
        assert_eq!(tier0.name, "None");
        assert_eq!(tier0.required_level, 0);
        assert_eq!(tier0.multiplier, 1.0);

        // Test metals tier (1-4)
        let tier1 = get_prestige_tier(1);
        assert_eq!(tier1.name, "Bronze");
        assert_eq!(tier1.required_level, 10);

        let tier2 = get_prestige_tier(2);
        assert_eq!(tier2.name, "Silver");
        assert_eq!(tier2.required_level, 25);

        let tier3 = get_prestige_tier(3);
        assert_eq!(tier3.name, "Gold");
        assert_eq!(tier3.required_level, 50);

        let tier4 = get_prestige_tier(4);
        assert_eq!(tier4.name, "Platinum");
        assert_eq!(tier4.required_level, 65);

        // Test gems tier (5-9)
        let tier5 = get_prestige_tier(5);
        assert_eq!(tier5.name, "Diamond");
        assert_eq!(tier5.required_level, 80);

        let tier6 = get_prestige_tier(6);
        assert_eq!(tier6.name, "Emerald");

        let tier9 = get_prestige_tier(9);
        assert_eq!(tier9.name, "Obsidian");

        // Test cosmic tier (10-14)
        let tier10 = get_prestige_tier(10);
        assert_eq!(tier10.name, "Celestial");
        assert_eq!(tier10.required_level, 130);

        let tier14 = get_prestige_tier(14);
        assert_eq!(tier14.name, "Galactic");

        // Test divine tier (15-19)
        let tier15 = get_prestige_tier(15);
        assert_eq!(tier15.name, "Transcendent");

        let tier19 = get_prestige_tier(19);
        assert_eq!(tier19.name, "Legendary");

        // Test eternal tier (20+)
        let tier20 = get_prestige_tier(20);
        assert_eq!(tier20.name, "Eternal");

        let tier100 = get_prestige_tier(100);
        assert_eq!(tier100.name, "Eternal");

        // Verify multiplier formula: 1 + 0.5 * rank^0.7 (diminishing returns)
        let expected_p4 = 1.0 + 0.5 * 4.0_f64.powf(0.7);
        let expected_p10 = 1.0 + 0.5 * 10.0_f64.powf(0.7);
        assert!((get_prestige_tier(4).multiplier - expected_p4).abs() < 0.001);
        assert!((get_prestige_tier(10).multiplier - expected_p10).abs() < 0.001);
    }

    #[test]
    fn test_multiplier_diminishing_returns() {
        // Verify multiplier grows with diminishing returns
        // Each subsequent prestige should give less % gain than the previous

        let mut prev_mult = get_prestige_tier(0).multiplier;
        let mut prev_gain = f64::MAX;

        for rank in 1..=30 {
            let mult = get_prestige_tier(rank).multiplier;
            let gain = mult - prev_mult;

            // Multiplier should always increase
            assert!(
                mult > prev_mult,
                "Multiplier should increase: P{} ({}) > P{} ({})",
                rank,
                mult,
                rank - 1,
                prev_mult
            );

            // Gain should decrease (diminishing returns) after P1
            if rank > 1 {
                assert!(
                    gain < prev_gain,
                    "Gain should diminish: P{} gain ({:.3}) < P{} gain ({:.3})",
                    rank,
                    gain,
                    rank - 1,
                    prev_gain
                );
            }

            prev_mult = mult;
            prev_gain = gain;
        }
    }

    #[test]
    fn test_multiplier_expected_values() {
        // Verify key multiplier values match design doc
        // Formula: 1 + 0.5 * rank^0.7

        let cases = [
            (0, 1.0),   // No prestige
            (1, 1.5),   // +50% at P1
            (5, 2.54),  // ~2.5x at P5
            (10, 3.51), // ~3.5x at P10
            (20, 5.07), // ~5x at P20
            (30, 6.41), // ~6.4x at P30
        ];

        for (rank, expected) in cases {
            let actual = get_prestige_tier(rank).multiplier;
            assert!(
                (actual - expected).abs() < 0.1,
                "P{}: expected ~{}, got {}",
                rank,
                expected,
                actual
            );
        }
    }

    #[test]
    fn test_can_prestige_not_ready() {
        let state = GameState::new("Test Hero".to_string(), 0);

        // Character starts at level 1, need level 10 for first prestige
        assert!(!can_prestige(&state));
    }

    #[test]
    fn test_can_prestige_ready() {
        let mut state = GameState::new("Test Hero".to_string(), 0);

        // Set character level to 10 (requirement for first prestige)
        state.character_level = 10;
        assert!(can_prestige(&state));

        // Should also work if level is higher
        state.character_level = 15;
        assert!(can_prestige(&state));
    }

    #[test]
    fn test_perform_prestige() {
        let mut state = GameState::new("Test Hero".to_string(), 0);

        // Set character level to 10 and some XP
        state.character_level = 10;
        state.character_xp = 5000;

        // Increase some attributes
        state.attributes.set(AttributeType::Strength, 15);
        state.attributes.set(AttributeType::Wisdom, 12);

        // Prestige should succeed
        perform_prestige(&mut state);

        // Verify prestige rank increased
        assert_eq!(state.prestige_rank, 1);
        assert_eq!(state.total_prestige_count, 1);

        // Verify character reset to level 1, XP 0
        assert_eq!(state.character_level, 1);
        assert_eq!(state.character_xp, 0);

        // Verify all attributes reset to 10
        for attr in AttributeType::all() {
            assert_eq!(state.attributes.get(attr), 10);
        }

        // Try to prestige again when not ready
        let old_rank = state.prestige_rank;
        perform_prestige(&mut state);

        // Should not have changed
        assert_eq!(state.prestige_rank, old_rank);
    }

    #[test]
    fn test_get_adventurer_rank() {
        assert_eq!(get_adventurer_rank(0), "Novice");
        assert_eq!(get_adventurer_rank(5), "Novice");
        assert_eq!(get_adventurer_rank(9), "Novice");
        assert_eq!(get_adventurer_rank(10), "Adept");
        assert_eq!(get_adventurer_rank(15), "Adept");
        assert_eq!(get_adventurer_rank(24), "Adept");
        assert_eq!(get_adventurer_rank(25), "Master");
        assert_eq!(get_adventurer_rank(40), "Master");
        assert_eq!(get_adventurer_rank(49), "Master");
        assert_eq!(get_adventurer_rank(50), "Grand Master");
        assert_eq!(get_adventurer_rank(60), "Grand Master");
        assert_eq!(get_adventurer_rank(74), "Grand Master");
        assert_eq!(get_adventurer_rank(75), "Legend");
        assert_eq!(get_adventurer_rank(85), "Legend");
        assert_eq!(get_adventurer_rank(99), "Legend");
        assert_eq!(get_adventurer_rank(100), "Mythic");
        assert_eq!(get_adventurer_rank(150), "Mythic");
    }

    #[test]
    fn test_equipment_cleared_on_prestige() {
        use crate::items::{AttributeBonuses, EquipmentSlot, Item, Rarity};
        use chrono::Utc;

        let mut game_state = crate::core::game_state::GameState::new(
            "Test Hero".to_string(),
            Utc::now().timestamp(),
        );

        // Equip an item
        let weapon = Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Rare,
            ilvl: 10,
            base_name: "Sword".to_string(),
            display_name: "Test Sword".to_string(),
            attributes: AttributeBonuses {
                str: 10,
                ..AttributeBonuses::new()
            },
            affixes: vec![],
        };
        game_state
            .equipment
            .set(EquipmentSlot::Weapon, Some(weapon.clone()));

        // Level up enough to prestige
        game_state.character_level = 10;

        // Perform prestige
        perform_prestige(&mut game_state);

        // Equipment should be cleared
        assert!(game_state.equipment.get(EquipmentSlot::Weapon).is_none());
        assert!(game_state.equipment.get(EquipmentSlot::Armor).is_none());
        assert!(game_state.equipment.get(EquipmentSlot::Helmet).is_none());
        assert!(game_state.equipment.get(EquipmentSlot::Gloves).is_none());
        assert!(game_state.equipment.get(EquipmentSlot::Boots).is_none());
        assert!(game_state.equipment.get(EquipmentSlot::Amulet).is_none());
        assert!(game_state.equipment.get(EquipmentSlot::Ring).is_none());
    }

    #[test]
    fn test_prestige_with_vault_preserves_items() {
        use crate::items::{AttributeBonuses, EquipmentSlot, Item, Rarity};
        use chrono::Utc;

        let mut game_state = crate::core::game_state::GameState::new(
            "Test Hero".to_string(),
            Utc::now().timestamp(),
        );

        // Equip a weapon and armor
        let weapon = Item {
            slot: EquipmentSlot::Weapon,
            rarity: Rarity::Legendary,
            ilvl: 10,
            base_name: "Sword".to_string(),
            display_name: "Stormbreaker".to_string(),
            attributes: AttributeBonuses {
                str: 15,
                ..AttributeBonuses::new()
            },
            affixes: vec![],
        };
        let armor = Item {
            slot: EquipmentSlot::Armor,
            rarity: Rarity::Rare,
            ilvl: 10,
            base_name: "Plate".to_string(),
            display_name: "Dragon Plate".to_string(),
            attributes: AttributeBonuses {
                con: 10,
                ..AttributeBonuses::new()
            },
            affixes: vec![],
        };
        game_state
            .equipment
            .set(EquipmentSlot::Weapon, Some(weapon));
        game_state.equipment.set(EquipmentSlot::Armor, Some(armor));

        // Level up enough to prestige
        game_state.character_level = 10;

        // Prestige with 1 vault slot, preserving weapon only
        let preserved = vec![EquipmentSlot::Weapon];
        perform_prestige_with_vault(&mut game_state, &preserved);

        // Prestige should have happened
        assert_eq!(game_state.prestige_rank, 1);
        assert_eq!(game_state.character_level, 1);

        // Weapon should be preserved
        let weapon = game_state.equipment.get(EquipmentSlot::Weapon);
        assert!(weapon.is_some());
        assert_eq!(weapon.as_ref().unwrap().display_name, "Stormbreaker");

        // Armor should be cleared (not preserved)
        assert!(game_state.equipment.get(EquipmentSlot::Armor).is_none());
    }

    #[test]
    fn test_dungeon_cleared_on_prestige() {
        use crate::dungeon::{Dungeon, DungeonSize};
        use chrono::Utc;

        let mut game_state = crate::core::game_state::GameState::new(
            "Test Hero".to_string(),
            Utc::now().timestamp(),
        );

        // Set up an active dungeon
        game_state.active_dungeon = Some(Dungeon::new(DungeonSize::Small));

        // Level up enough to prestige
        game_state.character_level = 10;

        // Perform prestige
        perform_prestige(&mut game_state);

        // Dungeon should be cleared
        assert!(game_state.active_dungeon.is_none());
    }

    #[test]
    fn test_combat_state_reset_on_prestige() {
        use chrono::Utc;

        let mut game_state = crate::core::game_state::GameState::new(
            "Test Hero".to_string(),
            Utc::now().timestamp(),
        );

        // Modify combat state
        game_state.combat_state.player_current_hp = 10; // Damaged
        game_state.combat_state.is_regenerating = true;

        // Level up enough to prestige
        game_state.character_level = 10;

        // Perform prestige
        perform_prestige(&mut game_state);

        // Combat state should be fresh
        assert_eq!(game_state.combat_state.player_current_hp, 50); // Base HP
        assert_eq!(game_state.combat_state.player_max_hp, 50);
        assert!(!game_state.combat_state.is_regenerating);
        assert!(game_state.combat_state.current_enemy.is_none());
    }

    #[test]
    fn test_fishing_preserved_on_prestige() {
        use crate::fishing::{CaughtFish, FishRarity, FishingPhase, FishingSession};
        use chrono::Utc;

        let mut state = crate::core::game_state::GameState::new(
            "Test Hero".to_string(),
            Utc::now().timestamp(),
        );

        // Set up fishing progress
        state.fishing.rank = 10;
        state.fishing.total_fish_caught = 500;
        state.fishing.fish_toward_next_rank = 50;
        state.fishing.legendary_catches = 5;

        // Set up an active fishing session
        state.active_fishing = Some(FishingSession {
            spot_name: "Test Lake".to_string(),
            total_fish: 10,
            fish_caught: vec![CaughtFish {
                name: "Test Fish".to_string(),
                rarity: FishRarity::Common,
                xp_reward: 10,
            }],
            items_found: vec![],
            ticks_remaining: 15,
            phase: FishingPhase::Waiting,
        });

        // Level up enough to prestige
        state.character_level = 10;

        // Perform prestige
        perform_prestige(&mut state);

        // Fishing rank and progression should be preserved
        assert_eq!(state.fishing.rank, 10);
        assert_eq!(state.fishing.total_fish_caught, 500);
        assert_eq!(state.fishing.fish_toward_next_rank, 50);
        assert_eq!(state.fishing.legendary_catches, 5);

        // Active fishing session should be cleared (transient state)
        assert!(state.active_fishing.is_none());
    }

    #[test]
    fn test_zone_progression_reset_on_prestige() {
        use chrono::Utc;

        let mut state = crate::core::game_state::GameState::new(
            "Test Hero".to_string(),
            Utc::now().timestamp(),
        );

        // Advance zone progression
        state.zone_progression.current_zone_id = 3;
        state.zone_progression.current_subzone_id = 2;
        state.zone_progression.defeat_boss(1, 1);
        state.zone_progression.defeat_boss(2, 1);

        // Level up enough to prestige
        state.character_level = 10;

        // Perform prestige
        perform_prestige(&mut state);

        // Zone progression should be reset to start
        assert_eq!(state.zone_progression.current_zone_id, 1);
        assert_eq!(state.zone_progression.current_subzone_id, 1);

        // Defeated bosses should be cleared
        assert!(!state.zone_progression.is_boss_defeated(1, 1));
        assert!(!state.zone_progression.is_boss_defeated(2, 1));
    }

    #[test]
    fn test_multiple_prestiges() {
        let mut state = GameState::new("Test Hero".to_string(), 0);

        // First prestige at level 10
        state.character_level = 10;
        perform_prestige(&mut state);
        assert_eq!(state.prestige_rank, 1);
        assert_eq!(state.total_prestige_count, 1);

        // Second prestige requires level 25
        state.character_level = 25;
        perform_prestige(&mut state);
        assert_eq!(state.prestige_rank, 2);
        assert_eq!(state.total_prestige_count, 2);

        // Third prestige requires level 50
        state.character_level = 50;
        perform_prestige(&mut state);
        assert_eq!(state.prestige_rank, 3);
        assert_eq!(state.total_prestige_count, 3);

        // Character should be reset each time
        assert_eq!(state.character_level, 1);
        assert_eq!(state.character_xp, 0);
    }

    #[test]
    fn test_attribute_cap_increases_with_prestige() {
        let mut state = GameState::new("Test Hero".to_string(), 0);

        // Base cap is 20
        assert_eq!(state.get_attribute_cap(), 20);

        // Prestige increases cap by 5
        state.character_level = 10;
        perform_prestige(&mut state);
        assert_eq!(state.get_attribute_cap(), 25);

        state.character_level = 25;
        perform_prestige(&mut state);
        assert_eq!(state.get_attribute_cap(), 30);
    }

    #[test]
    fn test_get_next_prestige_tier() {
        // From rank 0, next is rank 1 (Bronze)
        let next = get_next_prestige_tier(0);
        assert_eq!(next.rank, 1);
        assert_eq!(next.name, "Bronze");
        assert_eq!(next.required_level, 10);

        // From rank 4, next is rank 5 (Diamond)
        let next = get_next_prestige_tier(4);
        assert_eq!(next.rank, 5);
        assert_eq!(next.name, "Diamond");
        assert_eq!(next.required_level, 80);
    }

    #[test]
    fn test_high_prestige_ranks() {
        // Test rank 20+ (Eternal tier)
        let tier20 = get_prestige_tier(20);
        assert_eq!(tier20.name, "Eternal");
        assert_eq!(tier20.required_level, 235); // 220 + (20-19)*15

        let tier25 = get_prestige_tier(25);
        assert_eq!(tier25.name, "Eternal");
        assert_eq!(tier25.required_level, 310); // 220 + (25-19)*15

        // Multiplier should continue scaling
        assert!(tier25.multiplier > tier20.multiplier);
    }

    #[test]
    fn test_prestige_not_possible_when_ineligible() {
        let mut state = GameState::new("Test Hero".to_string(), 0);

        // At rank 1, need level 25 for next prestige
        state.prestige_rank = 1;
        state.character_level = 20; // Not enough

        assert!(!can_prestige(&state));

        // Try to prestige anyway
        perform_prestige(&mut state);

        // Should not have changed
        assert_eq!(state.prestige_rank, 1);
    }

    #[test]
    fn test_prestige_exactly_at_required_level() {
        let mut state = GameState::new("Test Hero".to_string(), 0);

        // Exactly at required level for first prestige
        state.character_level = 10;
        assert!(can_prestige(&state));

        // Prestige to rank 1, now need exactly level 25
        perform_prestige(&mut state);
        state.character_level = 25;
        assert!(can_prestige(&state));
    }
}
