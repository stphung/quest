//! Fishing logic and game tick processing.
//!
//! Handles fishing session ticks, fish catching, XP rewards, item drops,
//! fishing spot discovery, and rank progression.

#![allow(dead_code)]

use crate::fishing::{FishRarity, FishingState};
use crate::fishing_generation::{self, INITIAL_CATCH_TICKS};
use crate::game_state::GameState;
use crate::item_generation;
use crate::items::{EquipmentSlot, Rarity};
use crate::prestige::get_prestige_tier;
use rand::Rng;

/// Discovery chance for finding a fishing spot (5%)
const FISHING_DISCOVERY_CHANCE: f64 = 0.05;

/// Item drop chances by fish rarity (percentage)
const DROP_CHANCE_COMMON: f64 = 0.05; // 5%
const DROP_CHANCE_UNCOMMON: f64 = 0.05; // 5% (same as common)
const DROP_CHANCE_RARE: f64 = 0.15; // 15%
const DROP_CHANCE_EPIC: f64 = 0.35; // 35%
const DROP_CHANCE_LEGENDARY: f64 = 0.75; // 75%

/// Processes a fishing session tick.
///
/// Returns a vector of messages to display (catch notifications, session end).
///
/// # Processing Flow
/// 1. Decrements ticks_until_catch counter
/// 2. When counter reaches 0:
///    - Rolls fish rarity based on fishing rank
///    - Generates fish with name and XP reward
///    - Awards character XP (with prestige multiplier)
///    - Awards fishing rank progress
///    - Checks for item drop based on fish rarity
///    - Tracks legendary catches
///    - Resets catch timer
/// 3. Ends session when all fish are caught
pub fn tick_fishing(state: &mut GameState, rng: &mut impl Rng) -> Vec<String> {
    let mut messages = Vec::new();

    // Take ownership of active_fishing to work with it
    let session = match state.active_fishing.take() {
        Some(s) => s,
        None => return messages,
    };

    // Create mutable copy to modify
    let mut session = session;

    // Decrement tick counter
    if session.ticks_until_catch > 0 {
        session.ticks_until_catch -= 1;
    }

    // Check if it's time to catch a fish
    if session.ticks_until_catch == 0 {
        // Roll rarity and generate fish
        let rarity = fishing_generation::roll_fish_rarity(state.fishing.rank, rng);
        let fish = fishing_generation::generate_fish(rarity, rng);

        // Calculate XP with prestige multiplier
        let prestige_multiplier = get_prestige_tier(state.prestige_rank).multiplier;
        let xp_gained = (fish.xp_reward as f64 * prestige_multiplier) as u64;

        // Award character XP
        state.character_xp += xp_gained;

        // Award fishing rank progress
        state.fishing.fish_toward_next_rank += 1;
        state.fishing.total_fish_caught += 1;

        // Track legendary catches
        if rarity == FishRarity::Legendary {
            state.fishing.legendary_catches += 1;
        }

        // Generate catch message
        let rarity_name = match rarity {
            FishRarity::Common => "Common",
            FishRarity::Uncommon => "Uncommon",
            FishRarity::Rare => "Rare",
            FishRarity::Epic => "Epic",
            FishRarity::Legendary => "Legendary",
        };
        messages.push(format!(
            "Caught {} [{}]! +{} XP",
            fish.name, rarity_name, xp_gained
        ));

        // Check for item drop
        if let Some(item) = try_fishing_item_drop(rarity, state.character_level, rng) {
            messages.push(format!("Found item: {}!", item.display_name));
            session.items_found.push(item);
        }

        // Add fish to session
        session.fish_caught.push(fish);

        // Reset catch timer or end session
        if session.fish_caught.len() >= session.total_fish as usize {
            // Session complete
            messages.push(format!(
                "Fishing session at {} complete! Caught {} fish.",
                session.spot_name,
                session.fish_caught.len()
            ));
            // Don't put session back - it ends
            return messages;
        } else {
            // Reset timer for next catch
            session.ticks_until_catch = INITIAL_CATCH_TICKS;
        }
    }

    // Put session back
    state.active_fishing = Some(session);

    messages
}

/// Attempts to drop an item based on fish rarity.
///
/// Drop chances:
/// - Common: 5%
/// - Uncommon: 5%
/// - Rare: 15%
/// - Epic: 35%
/// - Legendary: 75%
fn try_fishing_item_drop(
    rarity: FishRarity,
    player_level: u32,
    rng: &mut impl Rng,
) -> Option<crate::items::Item> {
    let drop_chance = match rarity {
        FishRarity::Common => DROP_CHANCE_COMMON,
        FishRarity::Uncommon => DROP_CHANCE_UNCOMMON,
        FishRarity::Rare => DROP_CHANCE_RARE,
        FishRarity::Epic => DROP_CHANCE_EPIC,
        FishRarity::Legendary => DROP_CHANCE_LEGENDARY,
    };

    if rng.gen::<f64>() < drop_chance {
        // Generate item with rarity matching fish rarity
        let item_rarity = match rarity {
            FishRarity::Common => Rarity::Common,
            FishRarity::Uncommon => Rarity::Magic,
            FishRarity::Rare => Rarity::Rare,
            FishRarity::Epic => Rarity::Epic,
            FishRarity::Legendary => Rarity::Legendary,
        };

        // Random equipment slot
        let slots = [
            EquipmentSlot::Weapon,
            EquipmentSlot::Armor,
            EquipmentSlot::Helmet,
            EquipmentSlot::Gloves,
            EquipmentSlot::Boots,
            EquipmentSlot::Amulet,
            EquipmentSlot::Ring,
        ];
        let slot = slots[rng.gen_range(0..slots.len())];

        Some(item_generation::generate_item(
            slot,
            item_rarity,
            player_level,
        ))
    } else {
        None
    }
}

/// Attempts to discover a fishing spot.
///
/// Returns a discovery message if a spot is found.
///
/// # Conditions
/// - 5% chance per call
/// - Only if no active fishing session
/// - Only if not in a dungeon
pub fn try_discover_fishing(state: &mut GameState, rng: &mut impl Rng) -> Option<String> {
    // Check preconditions
    if state.active_fishing.is_some() {
        return None;
    }
    if state.active_dungeon.is_some() {
        return None;
    }

    // 5% discovery chance
    if rng.gen::<f64>() >= FISHING_DISCOVERY_CHANCE {
        return None;
    }

    // Generate new fishing session
    let session = fishing_generation::generate_fishing_session(rng);
    let spot_name = session.spot_name.clone();

    state.active_fishing = Some(session);

    Some(format!("Discovered fishing spot: {}!", spot_name))
}

/// Checks if the player should rank up in fishing.
///
/// Returns a rank up message if the threshold is reached.
///
/// # Rank Up Mechanics
/// - Each rank requires a certain number of fish to catch
/// - Fish requirement increases with rank tier
/// - Excess fish count carries over to next rank
pub fn check_rank_up(fishing_state: &mut FishingState) -> Option<String> {
    let required = FishingState::fish_required_for_rank(fishing_state.rank);

    if fishing_state.fish_toward_next_rank >= required {
        // Rank up
        fishing_state.fish_toward_next_rank -= required;
        fishing_state.rank += 1;

        let new_rank_name = fishing_state.rank_name();
        Some(format!(
            "Fishing rank up! Now rank {}: {}",
            fishing_state.rank, new_rank_name
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fishing::FishingSession;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn create_test_rng() -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(12345)
    }

    fn create_test_game_state() -> GameState {
        GameState::new("Test Fisher".to_string(), 0)
    }

    #[test]
    fn test_tick_fishing_catches_fish_and_awards_xp() {
        let mut rng = create_test_rng();
        let mut state = create_test_game_state();

        // Create a fishing session with 1 tick remaining
        let session = FishingSession {
            spot_name: "Test Lake".to_string(),
            total_fish: 5,
            fish_caught: Vec::new(),
            items_found: Vec::new(),
            ticks_until_catch: 1,
        };
        state.active_fishing = Some(session);

        let initial_xp = state.character_xp;
        let initial_fish_count = state.fishing.total_fish_caught;

        let messages = tick_fishing(&mut state, &mut rng);

        // Should have caught a fish
        assert!(
            !messages.is_empty(),
            "Should have catch message when timer reaches 0"
        );
        assert!(
            messages[0].contains("Caught"),
            "Message should mention catching"
        );

        // XP should have increased
        assert!(
            state.character_xp > initial_xp,
            "XP should increase after catch"
        );

        // Fish count should have increased
        assert_eq!(
            state.fishing.total_fish_caught,
            initial_fish_count + 1,
            "Total fish caught should increase"
        );
        assert_eq!(
            state.fishing.fish_toward_next_rank, 1,
            "Fish toward next rank should increase"
        );

        // Session should still be active (didn't catch all fish yet)
        assert!(
            state.active_fishing.is_some(),
            "Session should still be active"
        );
        let session = state.active_fishing.as_ref().unwrap();
        assert_eq!(session.fish_caught.len(), 1, "Should have 1 fish caught");
        assert_eq!(
            session.ticks_until_catch, INITIAL_CATCH_TICKS,
            "Timer should be reset"
        );
    }

    #[test]
    fn test_tick_fishing_decrements_timer() {
        let mut rng = create_test_rng();
        let mut state = create_test_game_state();

        // Create a fishing session with multiple ticks remaining
        let session = FishingSession {
            spot_name: "Test Lake".to_string(),
            total_fish: 5,
            fish_caught: Vec::new(),
            items_found: Vec::new(),
            ticks_until_catch: 10,
        };
        state.active_fishing = Some(session);

        let messages = tick_fishing(&mut state, &mut rng);

        // No catch yet
        assert!(
            messages.is_empty(),
            "Should not have messages when timer > 0"
        );

        // Timer should have decremented
        let session = state.active_fishing.as_ref().unwrap();
        assert_eq!(session.ticks_until_catch, 9, "Timer should decrement by 1");
    }

    #[test]
    fn test_session_ends_when_all_fish_caught() {
        let mut rng = create_test_rng();
        let mut state = create_test_game_state();

        // Create a session with 1 fish total and 1 tick remaining
        let session = FishingSession {
            spot_name: "Small Pond".to_string(),
            total_fish: 1,
            fish_caught: Vec::new(),
            items_found: Vec::new(),
            ticks_until_catch: 1,
        };
        state.active_fishing = Some(session);

        let messages = tick_fishing(&mut state, &mut rng);

        // Should have catch message and completion message
        assert!(
            messages.len() >= 2,
            "Should have catch and completion messages"
        );
        assert!(
            messages.iter().any(|m| m.contains("complete")),
            "Should have completion message"
        );

        // Session should be cleared
        assert!(
            state.active_fishing.is_none(),
            "Session should be cleared after catching all fish"
        );
    }

    #[test]
    fn test_try_discover_fishing_respects_conditions() {
        let mut rng = create_test_rng();
        let mut state = create_test_game_state();

        // With active fishing, should not discover
        state.active_fishing = Some(FishingSession {
            spot_name: "Existing".to_string(),
            total_fish: 5,
            fish_caught: Vec::new(),
            items_found: Vec::new(),
            ticks_until_catch: 15,
        });

        // Try many times - should never discover when already fishing
        for _ in 0..100 {
            let result = try_discover_fishing(&mut state, &mut rng);
            assert!(result.is_none(), "Should not discover when already fishing");
        }

        // Clear fishing session
        state.active_fishing = None;

        // With active dungeon, should not discover
        state.active_dungeon = Some(crate::dungeon::Dungeon::new(
            crate::dungeon::DungeonSize::Small,
        ));

        for _ in 0..100 {
            let result = try_discover_fishing(&mut state, &mut rng);
            assert!(result.is_none(), "Should not discover when in dungeon");
        }
    }

    #[test]
    fn test_try_discover_fishing_has_5_percent_chance() {
        let mut state = create_test_game_state();

        // Run many trials to verify approximately 5% discovery rate
        let trials = 10000;
        let mut discoveries = 0;

        for seed in 0..trials {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);

            // Reset state
            state.active_fishing = None;
            state.active_dungeon = None;

            if try_discover_fishing(&mut state, &mut rng).is_some() {
                discoveries += 1;
                // Clear for next trial
                state.active_fishing = None;
            }
        }

        let rate = discoveries as f64 / trials as f64;
        // Allow 1% tolerance (4-6% range)
        assert!(
            (0.04..=0.06).contains(&rate),
            "Discovery rate {} should be approximately 5%",
            rate
        );
    }

    #[test]
    fn test_check_rank_up_at_threshold() {
        let mut fishing_state = FishingState {
            rank: 1,
            total_fish_caught: 100,
            fish_toward_next_rank: 100, // Exactly at threshold for rank 1 (requires 100)
            legendary_catches: 0,
        };

        let result = check_rank_up(&mut fishing_state);

        assert!(result.is_some(), "Should rank up at threshold");
        assert_eq!(fishing_state.rank, 2, "Rank should increase to 2");
        assert_eq!(
            fishing_state.fish_toward_next_rank, 0,
            "Progress should reset"
        );
    }

    #[test]
    fn test_check_rank_up_with_excess() {
        let mut fishing_state = FishingState {
            rank: 1,
            total_fish_caught: 120,
            fish_toward_next_rank: 120, // 20 excess
            legendary_catches: 0,
        };

        let result = check_rank_up(&mut fishing_state);

        assert!(result.is_some(), "Should rank up");
        assert_eq!(fishing_state.rank, 2);
        assert_eq!(
            fishing_state.fish_toward_next_rank, 20,
            "Excess should carry over"
        );
    }

    #[test]
    fn test_check_rank_up_not_ready() {
        let mut fishing_state = FishingState {
            rank: 1,
            total_fish_caught: 50,
            fish_toward_next_rank: 50, // Only halfway to 100
            legendary_catches: 0,
        };

        let result = check_rank_up(&mut fishing_state);

        assert!(result.is_none(), "Should not rank up before threshold");
        assert_eq!(fishing_state.rank, 1, "Rank should remain 1");
        assert_eq!(
            fishing_state.fish_toward_next_rank, 50,
            "Progress should remain"
        );
    }

    #[test]
    fn test_legendary_fish_tracked() {
        let mut rng = create_test_rng();
        let mut state = create_test_game_state();

        // We need to catch a legendary fish - set up high rank for better odds
        state.fishing.rank = 30; // Max rank for best legendary chance

        let initial_legendary = state.fishing.legendary_catches;

        // Run many fishing attempts to catch a legendary
        let mut caught_legendary = false;
        for _ in 0..1000 {
            let session = FishingSession {
                spot_name: "Test".to_string(),
                total_fish: 100,
                fish_caught: Vec::new(),
                items_found: Vec::new(),
                ticks_until_catch: 1,
            };
            state.active_fishing = Some(session);

            tick_fishing(&mut state, &mut rng);

            if state.fishing.legendary_catches > initial_legendary {
                caught_legendary = true;
                break;
            }
        }

        assert!(
            caught_legendary,
            "Should eventually catch a legendary fish at max rank"
        );
    }

    #[test]
    fn test_prestige_multiplier_affects_xp() {
        let mut rng = ChaCha8Rng::seed_from_u64(99999); // Fixed seed for reproducibility
        let mut state = create_test_game_state();

        // First catch without prestige
        let session = FishingSession {
            spot_name: "Test".to_string(),
            total_fish: 5,
            fish_caught: Vec::new(),
            items_found: Vec::new(),
            ticks_until_catch: 1,
        };
        state.active_fishing = Some(session);
        state.prestige_rank = 0;

        let initial_xp = state.character_xp;
        tick_fishing(&mut state, &mut rng);
        let xp_gain_no_prestige = state.character_xp - initial_xp;

        // Now with prestige rank 2 (1.5^2 = 2.25x multiplier)
        let mut rng2 = ChaCha8Rng::seed_from_u64(99999); // Same seed for same fish
        let mut state2 = create_test_game_state();

        let session2 = FishingSession {
            spot_name: "Test".to_string(),
            total_fish: 5,
            fish_caught: Vec::new(),
            items_found: Vec::new(),
            ticks_until_catch: 1,
        };
        state2.active_fishing = Some(session2);
        state2.prestige_rank = 2;

        let initial_xp2 = state2.character_xp;
        tick_fishing(&mut state2, &mut rng2);
        let xp_gain_with_prestige = state2.character_xp - initial_xp2;

        // XP with prestige should be higher (accounting for integer truncation)
        assert!(
            xp_gain_with_prestige > xp_gain_no_prestige,
            "XP with prestige ({}) should be greater than without ({})",
            xp_gain_with_prestige,
            xp_gain_no_prestige
        );
    }

    #[test]
    fn test_tick_fishing_no_session() {
        let mut rng = create_test_rng();
        let mut state = create_test_game_state();

        // No active fishing session
        state.active_fishing = None;

        let messages = tick_fishing(&mut state, &mut rng);

        assert!(messages.is_empty(), "Should return empty when no session");
        assert!(
            state.active_fishing.is_none(),
            "Should remain with no session"
        );
    }
}
