use crate::game_state::GameState;

/// Represents a prestige tier with its properties
#[derive(Debug, Clone)]
pub struct PrestigeTier {
    pub rank: u32,
    pub name: &'static str,
    pub required_level: u32,
    pub multiplier: f64,
}

/// Gets the prestige tier for a given rank
///
/// # Arguments
/// * `rank` - The prestige rank
///
/// # Returns
/// The PrestigeTier with name, required level, and multiplier
pub fn get_prestige_tier(rank: u32) -> PrestigeTier {
    match rank {
        0 => PrestigeTier {
            rank: 0,
            name: "None",
            required_level: 0,
            multiplier: 1.0,
        },
        1 => PrestigeTier {
            rank: 1,
            name: "Bronze",
            required_level: 10,
            multiplier: 1.5,
        },
        2 => PrestigeTier {
            rank: 2,
            name: "Silver",
            required_level: 25,
            multiplier: 2.25,
        },
        3 => PrestigeTier {
            rank: 3,
            name: "Gold",
            required_level: 50,
            multiplier: 3.375,
        },
        5 => PrestigeTier {
            rank: 5,
            name: "Platinum",
            required_level: 75,
            multiplier: 7.59375,
        },
        10 => PrestigeTier {
            rank: 10,
            name: "Diamond",
            required_level: 100,
            multiplier: 57.665039,
        },
        15 => PrestigeTier {
            rank: 15,
            name: "Celestial",
            required_level: 150,
            multiplier: 437.893677,
        },
        _ => {
            // For other ranks, interpolate based on the pattern
            let multiplier = 1.5_f64.powi(rank as i32);
            let required_level = if rank < 3 {
                10 + (rank - 1) * 15
            } else if rank < 10 {
                50 + (rank - 3) * 10
            } else {
                100 + (rank - 10) * 25
            };

            PrestigeTier {
                rank,
                name: "Custom",
                required_level,
                multiplier,
            }
        }
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
/// true if all stats meet the required level for next prestige tier
pub fn can_prestige(state: &GameState) -> bool {
    let next_tier = get_next_prestige_tier(state.prestige_rank);

    // Check if all stats are at or above the required level
    for stat in &state.stats {
        if stat.level < next_tier.required_level {
            return false;
        }
    }

    true
}

/// Performs a prestige, resetting stats and incrementing prestige rank
///
/// # Arguments
/// * `state` - The game state to modify
pub fn perform_prestige(state: &mut GameState) {
    // Only prestige if eligible
    if !can_prestige(state) {
        return;
    }

    // Reset all stats to level 1, XP 0
    for stat in &mut state.stats {
        stat.level = 1;
        stat.current_xp = 0;
    }

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

    #[test]
    fn test_get_prestige_tier() {
        // Test defined tiers
        let tier0 = get_prestige_tier(0);
        assert_eq!(tier0.rank, 0);
        assert_eq!(tier0.name, "None");
        assert_eq!(tier0.required_level, 0);
        assert_eq!(tier0.multiplier, 1.0);

        let tier1 = get_prestige_tier(1);
        assert_eq!(tier1.rank, 1);
        assert_eq!(tier1.name, "Bronze");
        assert_eq!(tier1.required_level, 10);
        assert_eq!(tier1.multiplier, 1.5);

        let tier2 = get_prestige_tier(2);
        assert_eq!(tier2.rank, 2);
        assert_eq!(tier2.name, "Silver");
        assert_eq!(tier2.required_level, 25);
        assert_eq!(tier2.multiplier, 2.25);

        let tier3 = get_prestige_tier(3);
        assert_eq!(tier3.rank, 3);
        assert_eq!(tier3.name, "Gold");
        assert_eq!(tier3.required_level, 50);
        assert_eq!(tier3.multiplier, 3.375);

        let tier10 = get_prestige_tier(10);
        assert_eq!(tier10.rank, 10);
        assert_eq!(tier10.name, "Diamond");
        assert_eq!(tier10.required_level, 100);

        // Test interpolated tier
        let tier4 = get_prestige_tier(4);
        assert_eq!(tier4.rank, 4);
        assert_eq!(tier4.name, "Custom");
        assert_eq!(tier4.multiplier, 1.5_f64.powi(4));
    }

    #[test]
    fn test_can_prestige_not_ready() {
        let mut state = GameState::new(0);

        // All stats start at level 1, need level 10 for first prestige
        assert!(!can_prestige(&state));

        // Even if some stats are high enough, all must be
        state.stats[0].level = 10;
        state.stats[1].level = 10;
        state.stats[2].level = 10;
        state.stats[3].level = 9; // One stat below requirement
        assert!(!can_prestige(&state));
    }

    #[test]
    fn test_can_prestige_ready() {
        let mut state = GameState::new(0);

        // Set all stats to level 10 (requirement for first prestige)
        for stat in &mut state.stats {
            stat.level = 10;
        }

        assert!(can_prestige(&state));

        // Should also work if levels are higher
        for stat in &mut state.stats {
            stat.level = 15;
        }

        assert!(can_prestige(&state));
    }

    #[test]
    fn test_perform_prestige() {
        let mut state = GameState::new(0);

        // Set all stats to level 10 and some XP
        for stat in &mut state.stats {
            stat.level = 10;
            stat.current_xp = 500;
        }

        // Prestige should succeed
        perform_prestige(&mut state);

        // Verify prestige rank increased
        assert_eq!(state.prestige_rank, 1);
        assert_eq!(state.total_prestige_count, 1);

        // Verify all stats reset to level 1, XP 0
        for stat in &state.stats {
            assert_eq!(stat.level, 1);
            assert_eq!(stat.current_xp, 0);
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
}
