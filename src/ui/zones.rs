use crate::game_state::{GameState, StatType};
use rand::Rng;

/// Represents a zone in the game world
#[derive(Debug, Clone)]
pub struct Zone {
    pub name: &'static str,
    pub min_level: u32,
    pub max_level: u32,
    pub environment: Vec<&'static str>, // Environment emojis
    pub enemies: Vec<&'static str>,
}

/// Returns all zones in the game
fn get_all_zones() -> Vec<Zone> {
    vec![
        Zone {
            name: "Meadow",
            min_level: 0,
            max_level: 10,
            environment: vec!["🌸", "🌼", "🦋", "🌻", "🌷"],
            enemies: vec!["Slime", "Rabbit", "Ladybug", "Butterfly"],
        },
        Zone {
            name: "Dark Forest",
            min_level: 10,
            max_level: 25,
            environment: vec!["🌲", "🌳", "🍄", "🦇", "🕷️"],
            enemies: vec!["Wolf", "Spider", "Dark Elf", "Bat"],
        },
        Zone {
            name: "Mountain Pass",
            min_level: 25,
            max_level: 50,
            environment: vec!["⛰️", "🏔️", "🪨", "❄️", "☁️"],
            enemies: vec!["Golem", "Yeti", "Mountain Lion", "Eagle"],
        },
        Zone {
            name: "Ancient Ruins",
            min_level: 50,
            max_level: 75,
            environment: vec!["🏛️", "⚱️", "💀", "🗿", "🔮"],
            enemies: vec!["Skeleton", "Ghost", "Ancient Guardian", "Wraith"],
        },
        Zone {
            name: "Volcanic Wastes",
            min_level: 75,
            max_level: 100,
            environment: vec!["🌋", "🔥", "💥", "🌪️", "⚡"],
            enemies: vec!["Fire Elemental", "Lava Beast", "Phoenix", "Dragon"],
        },
    ]
}

/// Gets the current zone based on the player's average level
///
/// # Arguments
/// * `state` - The game state containing player stats
///
/// # Returns
/// The Zone that matches the player's average level
pub fn get_current_zone(state: &GameState) -> Zone {
    // Calculate average level across all stats
    let total_level: u32 = state.stats.iter().map(|s| s.level).sum();
    let avg_level = total_level / state.stats.len() as u32;

    let zones = get_all_zones();

    // Find the zone that contains the average level
    for zone in zones.iter() {
        if avg_level >= zone.min_level && avg_level < zone.max_level {
            return zone.clone();
        }
    }

    // If above all zones, return the highest zone
    zones.last().unwrap().clone()
}

/// Gets a random environment emoji from the zone
///
/// # Arguments
/// * `zone` - The zone to get an environment emoji from
///
/// # Returns
/// A random environment emoji string
pub fn get_random_environment(zone: &Zone) -> &'static str {
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..zone.environment.len());
    zone.environment[index]
}

/// Gets a random enemy from the zone
///
/// # Arguments
/// * `zone` - The zone to get an enemy from
///
/// # Returns
/// A random enemy name string
pub fn get_random_enemy(zone: &Zone) -> &'static str {
    let mut rng = rand::thread_rng();
    let index = rng.gen_range(0..zone.enemies.len());
    zone.enemies[index]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_current_zone_meadow() {
        let mut state = GameState::new(0);

        // Set all stats to level 5 (average 5, should be Meadow 0-10)
        for stat in &mut state.stats {
            stat.level = 5;
        }

        let zone = get_current_zone(&state);
        assert_eq!(zone.name, "Meadow");
        assert_eq!(zone.min_level, 0);
        assert_eq!(zone.max_level, 10);
    }

    #[test]
    fn test_get_current_zone_forest() {
        let mut state = GameState::new(0);

        // Set all stats to level 15 (average 15, should be Dark Forest 10-25)
        for stat in &mut state.stats {
            stat.level = 15;
        }

        let zone = get_current_zone(&state);
        assert_eq!(zone.name, "Dark Forest");
        assert_eq!(zone.min_level, 10);
        assert_eq!(zone.max_level, 25);
    }

    #[test]
    fn test_get_current_zone_volcanic() {
        let mut state = GameState::new(0);

        // Set all stats to level 90 (average 90, should be Volcanic Wastes 75-100)
        for stat in &mut state.stats {
            stat.level = 90;
        }

        let zone = get_current_zone(&state);
        assert_eq!(zone.name, "Volcanic Wastes");
        assert_eq!(zone.min_level, 75);
        assert_eq!(zone.max_level, 100);
    }
}
