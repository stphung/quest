use crate::attributes::Attributes;
use crate::combat::CombatState;
use crate::equipment::Equipment;
use serde::{Deserialize, Serialize};

/// Main game state containing all player progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub character_id: String,
    pub character_name: String,
    pub character_level: u32,
    pub character_xp: u64,
    pub attributes: Attributes,
    pub prestige_rank: u32,
    pub total_prestige_count: u64,
    pub last_save_time: i64,
    pub play_time_seconds: u64,
    pub combat_state: CombatState,
    pub equipment: Equipment,
}

impl GameState {
    /// Creates a new game state with default values
    pub fn new(character_name: String, current_time: i64) -> Self {
        use uuid::Uuid;

        let attributes = Attributes::new();
        let combat_state = CombatState::new(50); // Base HP
        let equipment = Equipment::new();

        Self {
            character_id: Uuid::new_v4().to_string(),
            character_name,
            character_level: 1,
            character_xp: 0,
            attributes,
            prestige_rank: 0,
            total_prestige_count: 0,
            last_save_time: current_time,
            play_time_seconds: 0,
            combat_state,
            equipment,
        }
    }

    pub fn get_attribute_cap(&self) -> u32 {
        20 + (self.prestige_rank * 5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attributes::AttributeType;

    #[test]
    fn test_new_game_state() {
        let current_time = 1234567890;
        let game_state = GameState::new("Test Hero".to_string(), current_time);

        assert_eq!(game_state.character_level, 1);
        assert_eq!(game_state.character_xp, 0);
        assert_eq!(game_state.prestige_rank, 0);
        assert_eq!(game_state.total_prestige_count, 0);
        assert_eq!(game_state.last_save_time, current_time);
        assert_eq!(game_state.play_time_seconds, 0);

        // Verify all attributes start at 10
        for attr in AttributeType::all() {
            assert_eq!(game_state.attributes.get(attr), 10);
        }
    }

    #[test]
    fn test_attribute_cap() {
        let mut game_state = GameState::new("Test Hero".to_string(), 0);

        // Prestige 0: cap 20
        assert_eq!(game_state.get_attribute_cap(), 20);

        // Prestige 1: cap 25
        game_state.prestige_rank = 1;
        assert_eq!(game_state.get_attribute_cap(), 25);

        // Prestige 2: cap 30
        game_state.prestige_rank = 2;
        assert_eq!(game_state.get_attribute_cap(), 30);
    }
}
