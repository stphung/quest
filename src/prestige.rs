use crate::game_state::GameState;

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
pub fn get_prestige_tier(rank: u32) -> PrestigeTier {
    let multiplier = 1.5_f64.powi(rank as i32);

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
    use crate::attributes::Attributes;
    use crate::combat::CombatState;
    use crate::equipment::Equipment;

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

    // Reset combat state with base HP (50 for fresh attributes)
    state.combat_state = CombatState::new(50);

    // Increment prestige rank and total prestige count
    state.prestige_rank += 1;
    state.total_prestige_count += 1;
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
    use crate::attributes::AttributeType;

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

        // Verify multiplier formula: 1.5^rank
        assert_eq!(get_prestige_tier(4).multiplier, 1.5_f64.powi(4));
        assert_eq!(get_prestige_tier(10).multiplier, 1.5_f64.powi(10));
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

        let mut game_state =
            crate::game_state::GameState::new("Test Hero".to_string(), Utc::now().timestamp());

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
    fn test_dungeon_cleared_on_prestige() {
        use crate::dungeon::{Dungeon, DungeonSize};
        use chrono::Utc;

        let mut game_state =
            crate::game_state::GameState::new("Test Hero".to_string(), Utc::now().timestamp());

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

        let mut game_state =
            crate::game_state::GameState::new("Test Hero".to_string(), Utc::now().timestamp());

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
}
