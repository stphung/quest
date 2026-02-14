//! Fishing logic and game tick processing.
//!
//! Handles fishing session ticks, fish catching, XP rewards, item drops,
//! fishing spot discovery, and rank progression.

#![allow(dead_code)]

use super::generation::{self as fishing_generation, is_storm_leviathan, LeviathanResult};
use super::types::{FishRarity, FishingPhase, FishingState};
use crate::character::prestige::get_prestige_tier;
use crate::core::constants::{
    BASE_MAX_FISHING_RANK, FISHING_DISCOVERY_CHANCE, FISHING_DROP_CHANCE_COMMON,
    FISHING_DROP_CHANCE_EPIC, FISHING_DROP_CHANCE_LEGENDARY, FISHING_DROP_CHANCE_RARE,
    FISHING_DROP_CHANCE_UNCOMMON, MAX_FISHING_RANK,
};
use crate::core::game_state::GameState;
use crate::items::generation as item_generation;
use crate::items::{ilvl_for_zone, roll_random_slot, Rarity};
use rand::{Rng, RngExt};

/// Apply timer reduction from Garden bonus
fn apply_timer_reduction(base_ticks: u32, reduction_percent: f64) -> u32 {
    let reduced = base_ticks as f64 * (1.0 - reduction_percent / 100.0);
    (reduced as u32).max(1) // Minimum 1 tick
}

// Item drop chances by fish rarity are defined in core::constants

/// Haven bonuses that affect fishing
#[derive(Debug, Clone, Default)]
pub struct HavenFishingBonuses {
    /// Garden: -% fishing timers (reduces cast/wait/reel time)
    pub timer_reduction_percent: f64,
    /// Fishing Dock: +% chance to catch double fish
    pub double_fish_chance_percent: f64,
    /// Fishing Dock T4: +max fishing rank (10 at T4)
    pub max_fishing_rank_bonus: u32,
}

/// Result from fishing tick that may include special catches
#[derive(Debug, Clone, Default)]
pub struct FishingTickResult {
    /// Messages to display to the player
    pub messages: Vec<String>,
    /// True if the Storm Leviathan was caught this tick
    pub caught_storm_leviathan: bool,
    /// If set, a Leviathan encounter occurred (it escaped). Value is encounter number (1-10).
    pub leviathan_encounter: Option<u8>,
}

/// Processes a fishing session tick with phase-based timing.
///
/// # Fishing Phases (average ~5s per fish)
/// 1. **Casting** (1s) - Line is being cast
/// 2. **Waiting** (2-4s) - Waiting for a bite
/// 3. **Reeling** (1-2s) - Fish is biting, reeling in
///
/// `haven` contains Haven bonuses for fishing
///
/// Returns a `FishingTickResult` with messages and special catch flags.
pub fn tick_fishing_with_haven_result(
    state: &mut GameState,
    rng: &mut impl Rng,
    haven: &HavenFishingBonuses,
) -> FishingTickResult {
    let mut result = FishingTickResult::default();

    // Take ownership of active_fishing to work with it
    let session = match state.active_fishing.take() {
        Some(s) => s,
        None => return result,
    };

    let mut session = session;

    // Decrement tick counter
    if session.ticks_remaining > 0 {
        session.ticks_remaining -= 1;
    }

    // Process phase transitions when timer reaches 0
    if session.ticks_remaining == 0 {
        match session.phase {
            FishingPhase::Casting => {
                // Casting complete, start waiting for bite
                session.phase = FishingPhase::Waiting;
                let base_ticks = fishing_generation::roll_waiting_ticks(rng);
                // Apply Garden bonus: reduce timers
                session.ticks_remaining =
                    apply_timer_reduction(base_ticks, haven.timer_reduction_percent);
                result
                    .messages
                    .push("Line cast... waiting for a bite...".to_string());
            }
            FishingPhase::Waiting => {
                // Got a bite! Start reeling
                session.phase = FishingPhase::Reeling;
                let base_ticks = fishing_generation::roll_reeling_ticks(rng);
                // Apply Garden bonus: reduce timers
                session.ticks_remaining =
                    apply_timer_reduction(base_ticks, haven.timer_reduction_percent);
                result
                    .messages
                    .push("🐟 Got a bite! Reeling in...".to_string());
            }
            FishingPhase::Reeling => {
                // Catch the fish!
                // Check for double fish (Fishing Dock bonus)
                let double_fish_roll = rng.random::<f64>() * 100.0;
                let fish_count = if double_fish_roll < haven.double_fish_chance_percent {
                    2
                } else {
                    1
                };

                for fish_num in 0..fish_count {
                    let rarity = fishing_generation::roll_fish_rarity(state.fishing.rank, rng);
                    // Use rank-aware fish generation for Storm Leviathan hunt
                    let (fish, leviathan_result) = fishing_generation::generate_fish_with_rank(
                        rarity,
                        state.fishing.rank,
                        state.fishing.leviathan_encounters,
                        rng,
                    );

                    match leviathan_result {
                        LeviathanResult::Caught => {
                            result.caught_storm_leviathan = true;
                        }
                        LeviathanResult::Escaped { encounter_number } => {
                            // Increment encounters and signal modal should show
                            state.fishing.leviathan_encounters = encounter_number;
                            result.leviathan_encounter = Some(encounter_number);
                        }
                        LeviathanResult::None => {}
                    }

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
                    let double_msg = if fish_count == 2 && fish_num == 1 {
                        " (DOUBLE!)"
                    } else {
                        ""
                    };

                    // Special message for Storm Leviathan
                    if is_storm_leviathan(&fish) {
                        result.messages.push(format!(
                            "⚡🐉 YOU CAUGHT THE STORM LEVIATHAN! [{}] +{} XP{}",
                            rarity_name, xp_gained, double_msg
                        ));
                        result.messages.push(
                            "The legendary beast! You can now forge the Stormbreaker at the Storm Forge!".to_string()
                        );
                    } else {
                        result.messages.push(format!(
                            "🎣 Caught {} [{}]! +{} XP{}",
                            fish.name, rarity_name, xp_gained, double_msg
                        ));
                    }

                    // Check for item drop (use zone for ilvl)
                    let zone_id = state.zone_progression.current_zone_id as usize;
                    if let Some(item) = try_fishing_item_drop(rarity, zone_id, rng) {
                        result
                            .messages
                            .push(format!("📦 Found item: {}!", item.display_name));
                        session.items_found.push(item);
                    }

                    // Add fish to session
                    session.fish_caught.push(fish);
                }

                // Check if session is complete
                if session.fish_caught.len() >= session.total_fish as usize {
                    result.messages.push(format!(
                        "Fishing spot depleted! Caught {} fish at {}.",
                        session.fish_caught.len(),
                        session.spot_name
                    ));
                    // Don't put session back - it ends
                    return result;
                }

                // Start casting again for next fish
                session.phase = FishingPhase::Casting;
                let base_ticks = fishing_generation::roll_casting_ticks(rng);
                session.ticks_remaining =
                    apply_timer_reduction(base_ticks, haven.timer_reduction_percent);
            }
        }
    }

    // Put session back
    state.active_fishing = Some(session);

    result
}

/// Processes a fishing session tick with phase-based timing.
///
/// # Fishing Phases (average ~5s per fish)
/// 1. **Casting** (1s) - Line is being cast
/// 2. **Waiting** (2-4s) - Waiting for a bite
/// 3. **Reeling** (1-2s) - Fish is biting, reeling in
///
/// `haven` contains Haven bonuses for fishing
#[allow(dead_code)]
pub fn tick_fishing_with_haven(
    state: &mut GameState,
    rng: &mut impl Rng,
    haven: &HavenFishingBonuses,
) -> Vec<String> {
    tick_fishing_with_haven_result(state, rng, haven).messages
}

/// Legacy function without Haven bonuses (for backwards compatibility)
pub fn tick_fishing(state: &mut GameState, rng: &mut impl Rng) -> Vec<String> {
    tick_fishing_with_haven(state, rng, &HavenFishingBonuses::default())
}

/// Attempts to drop an item based on fish rarity.
///
/// Drop chances:
/// - Common: 5%
/// - Uncommon: 5%
/// - Rare: 15%
/// - Epic: 35%
/// - Legendary: 75%
///
/// Item level is based on zone_id (ilvl = zone_id * 10).
fn try_fishing_item_drop(
    rarity: FishRarity,
    zone_id: usize,
    rng: &mut impl Rng,
) -> Option<crate::items::Item> {
    let drop_chance = match rarity {
        FishRarity::Common => FISHING_DROP_CHANCE_COMMON,
        FishRarity::Uncommon => FISHING_DROP_CHANCE_UNCOMMON,
        FishRarity::Rare => FISHING_DROP_CHANCE_RARE,
        FishRarity::Epic => FISHING_DROP_CHANCE_EPIC,
        FishRarity::Legendary => FISHING_DROP_CHANCE_LEGENDARY,
    };

    if rng.random::<f64>() < drop_chance {
        // Generate item with rarity matching fish rarity
        let item_rarity = match rarity {
            FishRarity::Common => Rarity::Common,
            FishRarity::Uncommon => Rarity::Magic,
            FishRarity::Rare => Rarity::Rare,
            FishRarity::Epic => Rarity::Epic,
            FishRarity::Legendary => Rarity::Legendary,
        };

        // Random equipment slot
        let slot = roll_random_slot(rng);

        // Item level based on zone
        let ilvl = ilvl_for_zone(zone_id);

        Some(item_generation::generate_item(slot, item_rarity, ilvl))
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
    if rng.random::<f64>() >= FISHING_DISCOVERY_CHANCE {
        return None;
    }

    // Generate new fishing session
    let session = fishing_generation::generate_fishing_session(rng);
    let spot_name = session.spot_name.clone();

    state.active_fishing = Some(session);

    Some(format!("Discovered fishing spot: {}!", spot_name))
}

// BASE_MAX_FISHING_RANK and MAX_FISHING_RANK are imported from core::constants

/// Returns the effective maximum fishing rank based on Haven bonus.
///
/// Base max is 30, but FishingDock T4 adds +10 for a total of 40.
pub fn get_max_fishing_rank(fishing_rank_bonus: u32) -> u32 {
    (BASE_MAX_FISHING_RANK + fishing_rank_bonus).min(MAX_FISHING_RANK)
}

/// Checks if the player should rank up in fishing.
///
/// Returns a rank up message if the threshold is reached.
///
/// # Arguments
/// - `fishing_state`: The player's fishing state
/// - `max_rank`: The effective maximum rank (base 30 + Haven bonus)
///
/// # Rank Up Mechanics
/// - Each rank requires a certain number of fish to catch
/// - Fish requirement increases with rank tier
/// - Excess fish count carries over to next rank
/// - Rank is capped at the effective max rank
pub fn check_rank_up_with_max(fishing_state: &mut FishingState, max_rank: u32) -> Option<String> {
    // Already at max rank - no further progression
    if fishing_state.rank >= max_rank {
        return None;
    }

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

/// Checks if the player should rank up in fishing (legacy, uses absolute max).
///
/// Returns a rank up message if the threshold is reached.
///
/// # Rank Up Mechanics
/// - Each rank requires a certain number of fish to catch
/// - Fish requirement increases with rank tier
/// - Excess fish count carries over to next rank
/// - Rank is capped at MAX_FISHING_RANK (30)
pub fn check_rank_up(fishing_state: &mut FishingState) -> Option<String> {
    check_rank_up_with_max(fishing_state, MAX_FISHING_RANK)
}

#[cfg(test)]
mod tests {
    use super::super::types::FishingSession;
    use super::*;
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

        // Create a fishing session in Reeling phase with 1 tick remaining
        let session = FishingSession {
            spot_name: "Test Lake".to_string(),
            total_fish: 5,
            fish_caught: Vec::new(),
            items_found: Vec::new(),
            ticks_remaining: 1,
            phase: FishingPhase::Reeling,
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
        assert!(
            session.ticks_remaining >= fishing_generation::CASTING_TICKS_MIN
                && session.ticks_remaining <= fishing_generation::CASTING_TICKS_MAX,
            "Timer should be reset to casting ticks range"
        );
        assert_eq!(
            session.phase,
            FishingPhase::Casting,
            "Should be back to casting"
        );
    }

    #[test]
    fn test_tick_fishing_decrements_timer() {
        let mut rng = create_test_rng();
        let mut state = create_test_game_state();

        // Create a fishing session in Waiting phase with multiple ticks remaining
        let session = FishingSession {
            spot_name: "Test Lake".to_string(),
            total_fish: 5,
            fish_caught: Vec::new(),
            items_found: Vec::new(),
            ticks_remaining: 10,
            phase: FishingPhase::Waiting,
        };
        state.active_fishing = Some(session);

        let messages = tick_fishing(&mut state, &mut rng);

        // No catch yet - still waiting
        assert!(
            messages.is_empty(),
            "Should not have messages when timer > 0"
        );

        // Timer should have decremented
        let session = state.active_fishing.as_ref().unwrap();
        assert_eq!(session.ticks_remaining, 9, "Timer should decrement by 1");
    }

    #[test]
    fn test_session_ends_when_all_fish_caught() {
        let mut rng = create_test_rng();
        let mut state = create_test_game_state();

        // Create a session with 1 fish total in Reeling phase
        let session = FishingSession {
            spot_name: "Small Pond".to_string(),
            total_fish: 1,
            fish_caught: Vec::new(),
            items_found: Vec::new(),
            ticks_remaining: 1,
            phase: FishingPhase::Reeling,
        };
        state.active_fishing = Some(session);

        let messages = tick_fishing(&mut state, &mut rng);

        // Should have catch message and completion message
        assert!(
            messages.len() >= 2,
            "Should have catch and completion messages"
        );
        assert!(
            messages.iter().any(|m| m.contains("depleted")),
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
            ticks_remaining: 15,
            phase: FishingPhase::Waiting,
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
            leviathan_encounters: 0,
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
            leviathan_encounters: 0,
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
            leviathan_encounters: 0,
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
    fn test_check_rank_up_capped_at_max() {
        let mut fishing_state = FishingState {
            rank: MAX_FISHING_RANK, // Already at max (30)
            total_fish_caught: 50000,
            fish_toward_next_rank: 5000, // Way more than enough to rank up
            legendary_catches: 100,
            leviathan_encounters: 0,
        };

        let result = check_rank_up(&mut fishing_state);

        assert!(result.is_none(), "Should not rank up past max rank");
        assert_eq!(
            fishing_state.rank, MAX_FISHING_RANK,
            "Rank should remain at max (30)"
        );
        assert_eq!(
            fishing_state.fish_toward_next_rank, 5000,
            "Progress should not be consumed at max rank"
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
                ticks_remaining: 1,
                phase: FishingPhase::Reeling,
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
            ticks_remaining: 1,
            phase: FishingPhase::Reeling,
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
            ticks_remaining: 1,
            phase: FishingPhase::Reeling,
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

    // =========================================================================
    // PHASE TRANSITION STATE MACHINE TESTS
    // =========================================================================

    #[test]
    fn test_phase_transitions_casting_to_waiting() {
        let mut rng = create_test_rng();
        let mut state = create_test_game_state();

        // Start in Casting phase with 1 tick remaining
        let session = FishingSession {
            spot_name: "Test Lake".to_string(),
            total_fish: 5,
            fish_caught: Vec::new(),
            items_found: Vec::new(),
            ticks_remaining: 1,
            phase: FishingPhase::Casting,
        };
        state.active_fishing = Some(session);

        let messages = tick_fishing(&mut state, &mut rng);

        // Should transition to Waiting
        let session = state.active_fishing.as_ref().unwrap();
        assert_eq!(
            session.phase,
            FishingPhase::Waiting,
            "Should transition from Casting to Waiting"
        );
        assert!(
            messages.iter().any(|m| m.contains("waiting")),
            "Should have waiting message"
        );
    }

    #[test]
    fn test_phase_transitions_waiting_to_reeling() {
        let mut rng = create_test_rng();
        let mut state = create_test_game_state();

        // Start in Waiting phase with 1 tick remaining
        let session = FishingSession {
            spot_name: "Test Lake".to_string(),
            total_fish: 5,
            fish_caught: Vec::new(),
            items_found: Vec::new(),
            ticks_remaining: 1,
            phase: FishingPhase::Waiting,
        };
        state.active_fishing = Some(session);

        let messages = tick_fishing(&mut state, &mut rng);

        // Should transition to Reeling
        let session = state.active_fishing.as_ref().unwrap();
        assert_eq!(
            session.phase,
            FishingPhase::Reeling,
            "Should transition from Waiting to Reeling"
        );
        assert!(
            messages
                .iter()
                .any(|m| m.contains("bite") || m.contains("Reeling")),
            "Should have bite/reeling message"
        );
    }

    #[test]
    fn test_phase_transitions_reeling_to_casting() {
        let mut rng = create_test_rng();
        let mut state = create_test_game_state();

        // Start in Reeling phase with 1 tick remaining (fish still to catch)
        let session = FishingSession {
            spot_name: "Test Lake".to_string(),
            total_fish: 5, // More fish to catch
            fish_caught: Vec::new(),
            items_found: Vec::new(),
            ticks_remaining: 1,
            phase: FishingPhase::Reeling,
        };
        state.active_fishing = Some(session);

        let messages = tick_fishing(&mut state, &mut rng);

        // Should catch fish and transition back to Casting
        let session = state.active_fishing.as_ref().unwrap();
        assert_eq!(
            session.phase,
            FishingPhase::Casting,
            "Should transition from Reeling back to Casting after catch"
        );
        assert_eq!(session.fish_caught.len(), 1, "Should have caught 1 fish");
        assert!(
            messages.iter().any(|m| m.contains("Caught")),
            "Should have catch message"
        );
    }

    #[test]
    fn test_full_fishing_cycle_casting_waiting_reeling_catch() {
        let mut rng = create_test_rng();
        let mut state = create_test_game_state();

        // Start fresh session in Casting
        let session = FishingSession {
            spot_name: "Test Lake".to_string(),
            total_fish: 3,
            fish_caught: Vec::new(),
            items_found: Vec::new(),
            ticks_remaining: 1,
            phase: FishingPhase::Casting,
        };
        state.active_fishing = Some(session);

        // Phase 1: Casting → Waiting
        tick_fishing(&mut state, &mut rng);
        assert_eq!(
            state.active_fishing.as_ref().unwrap().phase,
            FishingPhase::Waiting
        );

        // Drain the waiting timer
        loop {
            let session = state.active_fishing.as_ref().unwrap();
            if session.ticks_remaining == 1 {
                break;
            }
            tick_fishing(&mut state, &mut rng);
        }

        // Phase 2: Waiting → Reeling
        tick_fishing(&mut state, &mut rng);
        assert_eq!(
            state.active_fishing.as_ref().unwrap().phase,
            FishingPhase::Reeling
        );

        // Drain the reeling timer
        loop {
            let session = state.active_fishing.as_ref().unwrap();
            if session.ticks_remaining == 1 {
                break;
            }
            tick_fishing(&mut state, &mut rng);
        }

        // Phase 3: Reeling → Catch → Casting
        let fish_before = state.fishing.total_fish_caught;
        tick_fishing(&mut state, &mut rng);

        // Verify fish was caught
        assert_eq!(
            state.fishing.total_fish_caught,
            fish_before + 1,
            "Should have caught a fish"
        );

        // Should be back to Casting for next fish
        assert_eq!(
            state.active_fishing.as_ref().unwrap().phase,
            FishingPhase::Casting,
            "Should return to Casting after catch"
        );
    }

    // =========================================================================
    // Haven Fishing Bonus Tests
    // =========================================================================

    #[test]
    fn test_apply_timer_reduction() {
        // 0% reduction should not change ticks
        assert_eq!(apply_timer_reduction(100, 0.0), 100);

        // 50% reduction should halve ticks
        assert_eq!(apply_timer_reduction(100, 50.0), 50);

        // 40% reduction (Garden T3) on 10 ticks
        assert_eq!(apply_timer_reduction(10, 40.0), 6);

        // Minimum 1 tick even with 100% reduction
        assert_eq!(apply_timer_reduction(10, 100.0), 1);
    }

    #[test]
    fn test_haven_timer_reduction() {
        let mut rng = create_test_rng();
        let mut state = create_test_game_state();

        // Create a fishing session in Casting phase
        let session = FishingSession {
            spot_name: "Test Lake".to_string(),
            total_fish: 5,
            fish_caught: Vec::new(),
            items_found: Vec::new(),
            ticks_remaining: 1,
            phase: FishingPhase::Casting,
        };
        state.active_fishing = Some(session);

        // Transition with 40% timer reduction (Garden T3)
        let haven = HavenFishingBonuses {
            timer_reduction_percent: 40.0,
            double_fish_chance_percent: 0.0,
            max_fishing_rank_bonus: 0,
        };
        tick_fishing_with_haven(&mut state, &mut rng, &haven);

        // Should be in Waiting phase with reduced ticks
        let session = state.active_fishing.as_ref().unwrap();
        assert_eq!(session.phase, FishingPhase::Waiting);

        // Waiting ticks should be reduced (base is 10-80, reduced by 40%)
        let max_reduced_ticks = (fishing_generation::WAITING_TICKS_MAX as f64 * 0.6) as u32;
        assert!(
            session.ticks_remaining <= max_reduced_ticks,
            "Ticks {} should be <= {} (40% reduction)",
            session.ticks_remaining,
            max_reduced_ticks
        );
    }

    #[test]
    fn test_haven_double_fish() {
        let mut state = create_test_game_state();
        let mut double_fish_count = 0;
        let trials = 1000;

        for seed in 0..trials {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);

            // Create a fishing session in Reeling phase with 1 tick remaining
            let session = FishingSession {
                spot_name: "Test Lake".to_string(),
                total_fish: 100, // Lots of fish so we don't run out
                fish_caught: Vec::new(),
                items_found: Vec::new(),
                ticks_remaining: 1,
                phase: FishingPhase::Reeling,
            };
            state.active_fishing = Some(session);

            let initial_fish = state.fishing.total_fish_caught;

            // 50% double fish chance (Fishing Dock T2)
            let haven = HavenFishingBonuses {
                timer_reduction_percent: 0.0,
                double_fish_chance_percent: 50.0,
                max_fishing_rank_bonus: 0,
            };
            tick_fishing_with_haven(&mut state, &mut rng, &haven);

            let fish_caught = state.fishing.total_fish_caught - initial_fish;
            if fish_caught == 2 {
                double_fish_count += 1;
            }

            // Reset for next trial
            state.fishing.total_fish_caught = 0;
        }

        // With 50% chance, expect ~500 double catches in 1000 trials
        assert!(
            (400..=600).contains(&double_fish_count),
            "Expected ~500 double fish (50%), got {}",
            double_fish_count
        );
    }
}
