use serde::{Deserialize, Serialize};

/// Represents the four core stat types in the game
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StatType {
    Strength,
    Magic,
    Wisdom,
    Vitality,
}

impl StatType {
    /// Returns an array of all stat types
    pub fn all() -> [StatType; 4] {
        [
            StatType::Strength,
            StatType::Magic,
            StatType::Wisdom,
            StatType::Vitality,
        ]
    }

    /// Returns the abbreviated name of the stat
    pub fn name(&self) -> &str {
        match self {
            StatType::Strength => "STR",
            StatType::Magic => "MAG",
            StatType::Wisdom => "WIS",
            StatType::Vitality => "VIT",
        }
    }
}

/// Represents a single stat with level and experience
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stat {
    pub level: u32,
    pub current_xp: u64,
}

impl Stat {
    /// Creates a new stat at level 1 with 0 XP
    pub fn new() -> Self {
        Self {
            level: 1,
            current_xp: 0,
        }
    }
}

/// Main game state containing all player progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub stats: [Stat; 4],
    pub prestige_rank: u32,
    pub total_prestige_count: u64,
    pub last_save_time: i64,
    pub play_time_seconds: u64,
}

impl GameState {
    /// Creates a new game state with default values
    pub fn new(current_time: i64) -> Self {
        Self {
            stats: [Stat::new(), Stat::new(), Stat::new(), Stat::new()],
            prestige_rank: 0,
            total_prestige_count: 0,
            last_save_time: current_time,
            play_time_seconds: 0,
        }
    }

    /// Gets an immutable reference to a specific stat
    pub fn get_stat(&self, stat_type: StatType) -> &Stat {
        match stat_type {
            StatType::Strength => &self.stats[0],
            StatType::Magic => &self.stats[1],
            StatType::Wisdom => &self.stats[2],
            StatType::Vitality => &self.stats[3],
        }
    }

    /// Gets a mutable reference to a specific stat
    pub fn get_stat_mut(&mut self, stat_type: StatType) -> &mut Stat {
        match stat_type {
            StatType::Strength => &mut self.stats[0],
            StatType::Magic => &mut self.stats[1],
            StatType::Wisdom => &mut self.stats[2],
            StatType::Vitality => &mut self.stats[3],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_stat() {
        let stat = Stat::new();
        assert_eq!(stat.level, 1);
        assert_eq!(stat.current_xp, 0);
    }

    #[test]
    fn test_new_game_state() {
        let current_time = 1234567890;
        let game_state = GameState::new(current_time);

        assert_eq!(game_state.prestige_rank, 0);
        assert_eq!(game_state.total_prestige_count, 0);
        assert_eq!(game_state.last_save_time, current_time);
        assert_eq!(game_state.play_time_seconds, 0);

        // Verify all stats are initialized
        for stat in &game_state.stats {
            assert_eq!(stat.level, 1);
            assert_eq!(stat.current_xp, 0);
        }
    }

    #[test]
    fn test_stat_type_all() {
        let all_stats = StatType::all();
        assert_eq!(all_stats.len(), 4);
        assert_eq!(all_stats[0], StatType::Strength);
        assert_eq!(all_stats[1], StatType::Magic);
        assert_eq!(all_stats[2], StatType::Wisdom);
        assert_eq!(all_stats[3], StatType::Vitality);

        // Verify names
        assert_eq!(StatType::Strength.name(), "STR");
        assert_eq!(StatType::Magic.name(), "MAG");
        assert_eq!(StatType::Wisdom.name(), "WIS");
        assert_eq!(StatType::Vitality.name(), "VIT");
    }

    #[test]
    fn test_get_stat() {
        let mut game_state = GameState::new(0);

        // Test immutable access
        let str_stat = game_state.get_stat(StatType::Strength);
        assert_eq!(str_stat.level, 1);

        let mag_stat = game_state.get_stat(StatType::Magic);
        assert_eq!(mag_stat.level, 1);

        let wis_stat = game_state.get_stat(StatType::Wisdom);
        assert_eq!(wis_stat.level, 1);

        let vit_stat = game_state.get_stat(StatType::Vitality);
        assert_eq!(vit_stat.level, 1);

        // Test mutable access
        let str_stat_mut = game_state.get_stat_mut(StatType::Strength);
        str_stat_mut.level = 10;
        str_stat_mut.current_xp = 500;

        // Verify mutation worked
        let str_stat = game_state.get_stat(StatType::Strength);
        assert_eq!(str_stat.level, 10);
        assert_eq!(str_stat.current_xp, 500);

        // Verify other stats unchanged
        assert_eq!(game_state.get_stat(StatType::Magic).level, 1);
    }
}
