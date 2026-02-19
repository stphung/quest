//! Fishing logic and game tick processing.
//!
//! Handles fishing session ticks, fish catching, XP rewards, item drops,
//! fishing spot discovery, and rank progression.

#![allow(dead_code)]

use super::drops::try_fishing_item_drop;
use super::generation::{self as fishing_generation, is_storm_leviathan, LeviathanResult};
use super::rank::get_max_fishing_rank;
use super::types::{FishRarity, FishingPhase};
use crate::character::prestige::get_prestige_tier;
use crate::core::game_state::GameState;
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
    god_item_fishing_reduction_percent: f64,
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
                // Apply Garden bonus then god item bonus (multiplicative)
                let after_haven = apply_timer_reduction(base_ticks, haven.timer_reduction_percent);
                session.ticks_remaining =
                    apply_timer_reduction(after_haven, god_item_fishing_reduction_percent);
                result
                    .messages
                    .push("Line cast... waiting for a bite...".to_string());
            }
            FishingPhase::Waiting => {
                // Got a bite! Start reeling
                session.phase = FishingPhase::Reeling;
                let base_ticks = fishing_generation::roll_reeling_ticks(rng);
                // Apply Garden bonus then god item bonus (multiplicative)
                let after_haven = apply_timer_reduction(base_ticks, haven.timer_reduction_percent);
                session.ticks_remaining =
                    apply_timer_reduction(after_haven, god_item_fishing_reduction_percent);
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

                    // Award fishing rank progress (only if below current cap)
                    let max_rank = get_max_fishing_rank(haven.max_fishing_rank_bonus);
                    if state.fishing.rank < max_rank {
                        state.fishing.fish_toward_next_rank += 1;
                    }
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
                // Apply Garden bonus then god item bonus (multiplicative)
                let after_haven = apply_timer_reduction(base_ticks, haven.timer_reduction_percent);
                session.ticks_remaining =
                    apply_timer_reduction(after_haven, god_item_fishing_reduction_percent);
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
    tick_fishing_with_haven_result(state, rng, haven, 0.0).messages
}

/// Legacy function without Haven bonuses (for backwards compatibility)
pub fn tick_fishing(state: &mut GameState, rng: &mut impl Rng) -> Vec<String> {
    tick_fishing_with_haven(state, rng, &HavenFishingBonuses::default())
}

#[cfg(test)]
mod tests {
    use super::super::drops::try_fishing_item_drop;
    use super::super::rank::{check_rank_up, check_rank_up_with_max, get_max_fishing_rank};
    use super::super::types::{FishingSession, FishingState};
    use super::*;
    use crate::core::constants::{BASE_MAX_FISHING_RANK, MAX_FISHING_RANK};
    use crate::fishing::types::FishRarity;
    use crate::items::Rarity;
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

    // =========================================================================
    // STORM LEVIATHAN HUNT PROGRESSION TESTS
    // =========================================================================

    #[test]
    fn test_leviathan_encounter_tracked_in_fishing_tick() {
        // At rank 40 with legendary catches, leviathan encounters should be recorded
        // via tick_fishing_with_haven_result
        let haven = HavenFishingBonuses::default();
        let mut encountered = false;

        for seed in 0u64..5000 {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let mut state = create_test_game_state();
            state.fishing.rank = 40;
            state.fishing.leviathan_encounters = 0;

            let session = FishingSession {
                spot_name: "Deep Sea".to_string(),
                total_fish: 100,
                fish_caught: Vec::new(),
                items_found: Vec::new(),
                ticks_remaining: 1,
                phase: FishingPhase::Reeling,
            };
            state.active_fishing = Some(session);

            let result = tick_fishing_with_haven_result(&mut state, &mut rng, &haven, 0.0);

            if let Some(enc) = result.leviathan_encounter {
                assert_eq!(enc, 1, "First encounter should be number 1");
                assert_eq!(
                    state.fishing.leviathan_encounters, 1,
                    "State should track encounter"
                );
                encountered = true;
                break;
            }
        }

        assert!(
            encountered,
            "Should encounter Leviathan at least once in 5000 seeds at rank 40"
        );
    }

    #[test]
    fn test_leviathan_no_encounter_below_rank_40() {
        let haven = HavenFishingBonuses::default();

        for seed in 0u64..1000 {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let mut state = create_test_game_state();
            state.fishing.rank = 30; // Below 40
            state.fishing.leviathan_encounters = 0;

            let session = FishingSession {
                spot_name: "Lake".to_string(),
                total_fish: 100,
                fish_caught: Vec::new(),
                items_found: Vec::new(),
                ticks_remaining: 1,
                phase: FishingPhase::Reeling,
            };
            state.active_fishing = Some(session);

            let result = tick_fishing_with_haven_result(&mut state, &mut rng, &haven, 0.0);

            assert!(
                result.leviathan_encounter.is_none(),
                "Should never encounter Leviathan below rank 40 (seed {})",
                seed
            );
            assert!(
                !result.caught_storm_leviathan,
                "Should never catch Leviathan below rank 40"
            );
        }
    }

    #[test]
    fn test_leviathan_caught_via_fishing_tick() {
        let haven = HavenFishingBonuses::default();
        let mut caught = false;

        for seed in 0u64..10000 {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let mut state = create_test_game_state();
            state.fishing.rank = 40;
            state.fishing.leviathan_encounters = 10; // All encounters done

            let session = FishingSession {
                spot_name: "Deep Sea".to_string(),
                total_fish: 100,
                fish_caught: Vec::new(),
                items_found: Vec::new(),
                ticks_remaining: 1,
                phase: FishingPhase::Reeling,
            };
            state.active_fishing = Some(session);

            let result = tick_fishing_with_haven_result(&mut state, &mut rng, &haven, 0.0);

            if result.caught_storm_leviathan {
                assert!(
                    result
                        .messages
                        .iter()
                        .any(|m| m.contains("STORM LEVIATHAN")),
                    "Catch message should mention Storm Leviathan"
                );
                caught = true;
                break;
            }
        }

        assert!(
            caught,
            "Should catch Leviathan eventually at rank 40 with 10 encounters (25% chance)"
        );
    }

    // =========================================================================
    // ITEM DROP FROM FISHING TESTS
    // =========================================================================

    #[test]
    fn test_fishing_item_drop_rates_by_rarity() {
        let trials = 5000;

        for (rarity, expected_rate, tolerance) in [
            (FishRarity::Common, 0.05, 0.02),
            (FishRarity::Uncommon, 0.05, 0.02),
            (FishRarity::Rare, 0.15, 0.03),
            (FishRarity::Epic, 0.35, 0.04),
            (FishRarity::Legendary, 0.75, 0.04),
        ] {
            let mut drops = 0;
            for seed in 0..trials {
                let mut rng = ChaCha8Rng::seed_from_u64(seed);
                if try_fishing_item_drop(rarity, 1, &mut rng).is_some() {
                    drops += 1;
                }
            }
            let actual_rate = drops as f64 / trials as f64;
            assert!(
                (actual_rate - expected_rate).abs() < tolerance,
                "{:?} drop rate {:.3} should be near {:.3} (tolerance {:.3})",
                rarity,
                actual_rate,
                expected_rate,
                tolerance
            );
        }
    }

    #[test]
    fn test_fishing_item_drop_rarity_matches_fish_rarity() {
        // Verify that the item rarity mapping is correct
        for (fish_rarity, expected_item_rarity) in [
            (FishRarity::Common, Rarity::Common),
            (FishRarity::Uncommon, Rarity::Magic),
            (FishRarity::Rare, Rarity::Rare),
            (FishRarity::Epic, Rarity::Epic),
            (FishRarity::Legendary, Rarity::Legendary),
        ] {
            // Try many seeds until we get a drop, then check its rarity
            for seed in 0u64..10000 {
                let mut rng = ChaCha8Rng::seed_from_u64(seed);
                if let Some(item) = try_fishing_item_drop(fish_rarity, 5, &mut rng) {
                    assert_eq!(
                        item.rarity, expected_item_rarity,
                        "{:?} fish should produce {:?} items, got {:?}",
                        fish_rarity, expected_item_rarity, item.rarity
                    );
                    break;
                }
            }
        }
    }

    #[test]
    fn test_fishing_item_drop_uses_zone_ilvl() {
        // Items from fishing should use ilvl_for_zone(zone_id)
        for zone_id in [1, 5, 10] {
            let expected_ilvl = crate::items::ilvl_for_zone(zone_id);
            // Use legendary rarity for high drop chance (75%)
            for seed in 0u64..100 {
                let mut rng = ChaCha8Rng::seed_from_u64(seed);
                if let Some(item) = try_fishing_item_drop(FishRarity::Legendary, zone_id, &mut rng)
                {
                    assert_eq!(
                        item.ilvl, expected_ilvl,
                        "Item ilvl should match zone {} ilvl {}",
                        zone_id, expected_ilvl
                    );
                    break;
                }
            }
        }
    }

    // =========================================================================
    // RANK SYSTEM EDGE CASE TESTS
    // =========================================================================

    #[test]
    fn test_get_max_fishing_rank_without_haven() {
        assert_eq!(
            get_max_fishing_rank(0),
            BASE_MAX_FISHING_RANK,
            "Without Haven bonus, max rank should be 30"
        );
    }

    #[test]
    fn test_get_max_fishing_rank_with_haven_t4() {
        assert_eq!(
            get_max_fishing_rank(10),
            MAX_FISHING_RANK,
            "With Fishing Dock T4 (+10), max rank should be 40"
        );
    }

    #[test]
    fn test_get_max_fishing_rank_capped_at_40() {
        assert_eq!(
            get_max_fishing_rank(100),
            MAX_FISHING_RANK,
            "Max rank should be capped at 40 even with large bonus"
        );
    }

    #[test]
    fn test_check_rank_up_with_max_limits_progression() {
        let mut fishing_state = FishingState {
            rank: 30,
            total_fish_caught: 50000,
            fish_toward_next_rank: 4000, // Enough for rank 31 (requires 4000)
            legendary_catches: 100,
            leviathan_encounters: 0,
        };

        // With max_rank=30, should NOT rank up past 30
        let result = check_rank_up_with_max(&mut fishing_state, 30);
        assert!(result.is_none(), "Should not rank up past max_rank=30");
        assert_eq!(fishing_state.rank, 30);

        // With max_rank=40 (Haven bonus), should rank up
        let result = check_rank_up_with_max(&mut fishing_state, 40);
        assert!(result.is_some(), "Should rank up to 31 with max_rank=40");
        assert_eq!(fishing_state.rank, 31);
    }

    #[test]
    fn test_rank_up_at_all_tier_boundaries() {
        // Test that rank-up works correctly at each tier boundary
        let tier_boundaries = [
            (5, 6, 100),    // Novice -> Apprentice
            (10, 11, 200),  // Apprentice -> Journeyman
            (15, 16, 400),  // Journeyman -> Expert
            (20, 21, 800),  // Expert -> Master
            (25, 26, 1500), // Master -> Grandmaster
        ];

        for (from_rank, to_rank, required) in tier_boundaries {
            let mut fishing_state = FishingState {
                rank: from_rank,
                total_fish_caught: 100000,
                fish_toward_next_rank: required,
                legendary_catches: 0,
                leviathan_encounters: 0,
            };

            let result = check_rank_up_with_max(&mut fishing_state, 40);
            assert!(
                result.is_some(),
                "Should rank up from {} to {} with {} fish",
                from_rank,
                to_rank,
                required
            );
            assert_eq!(
                fishing_state.rank, to_rank,
                "Rank should advance from {} to {}",
                from_rank, to_rank
            );
            assert_eq!(
                fishing_state.fish_toward_next_rank, 0,
                "Progress should reset exactly at threshold"
            );
        }
    }

    #[test]
    fn test_fish_progress_does_not_accumulate_at_cap() {
        // When at rank 30 without T4, fish_toward_next_rank should NOT increase
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut state = create_test_game_state();
        state.fishing.rank = 30;
        state.fishing.fish_toward_next_rank = 0;
        state.fishing.total_fish_caught = 25000;

        // Set up a session about to complete a catch
        let session = FishingSession {
            spot_name: "Small Pond".to_string(),
            total_fish: 5,
            fish_caught: Vec::new(),
            items_found: Vec::new(),
            ticks_remaining: 1,
            phase: FishingPhase::Reeling,
        };
        state.active_fishing = Some(session);

        let haven = HavenFishingBonuses {
            timer_reduction_percent: 0.0,
            double_fish_chance_percent: 0.0,
            max_fishing_rank_bonus: 0, // No T4 — cap is 30
        };

        let _result = tick_fishing_with_haven_result(&mut state, &mut rng, &haven, 0.0);

        // total_fish_caught should increase (it always counts)
        assert!(
            state.fishing.total_fish_caught > 25000,
            "total_fish_caught should still increase"
        );
        // But fish_toward_next_rank should NOT increase at cap
        assert_eq!(
            state.fishing.fish_toward_next_rank, 0,
            "fish_toward_next_rank should not accumulate when at rank cap"
        );
    }

    #[test]
    fn test_fish_progress_accumulates_with_t4() {
        // When at rank 30 WITH T4, fish_toward_next_rank SHOULD increase
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut state = create_test_game_state();
        state.fishing.rank = 30;
        state.fishing.fish_toward_next_rank = 0;
        state.fishing.total_fish_caught = 25000;

        let session = FishingSession {
            spot_name: "Small Pond".to_string(),
            total_fish: 5,
            fish_caught: Vec::new(),
            items_found: Vec::new(),
            ticks_remaining: 1,
            phase: FishingPhase::Reeling,
        };
        state.active_fishing = Some(session);

        let haven = HavenFishingBonuses {
            timer_reduction_percent: 0.0,
            double_fish_chance_percent: 0.0,
            max_fishing_rank_bonus: 10, // T4 unlocked — cap is 40
        };

        let _result = tick_fishing_with_haven_result(&mut state, &mut rng, &haven, 0.0);

        assert!(
            state.fishing.fish_toward_next_rank > 0,
            "fish_toward_next_rank should accumulate when below rank cap with T4"
        );
    }

    // =========================================================================
    // SESSION EDGE CASE TESTS
    // =========================================================================

    #[test]
    fn test_double_fish_can_complete_session() {
        // If double fish is triggered on the last fish, session should end correctly
        let mut caught_double_on_last = false;

        for seed in 0u64..2000 {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let mut state = create_test_game_state();

            // Session with 2 total fish and 1 already caught — double fish should finish it
            let session = FishingSession {
                spot_name: "Small Pond".to_string(),
                total_fish: 2,
                fish_caught: Vec::new(), // 0 caught, need 2
                items_found: Vec::new(),
                ticks_remaining: 1,
                phase: FishingPhase::Reeling,
            };
            state.active_fishing = Some(session);

            let haven = HavenFishingBonuses {
                timer_reduction_percent: 0.0,
                double_fish_chance_percent: 100.0, // Guarantee double fish
                max_fishing_rank_bonus: 0,
            };

            let result = tick_fishing_with_haven_result(&mut state, &mut rng, &haven, 0.0);

            // With 100% double fish and 2 total needed, should complete in one catch
            if state.active_fishing.is_none() {
                assert!(
                    result.messages.iter().any(|m| m.contains("depleted")),
                    "Should have completion message"
                );
                assert!(
                    result.messages.iter().any(|m| m.contains("DOUBLE")),
                    "Should have double fish marker"
                );
                caught_double_on_last = true;
                break;
            }
        }

        assert!(
            caught_double_on_last,
            "Double fish should be able to complete a session"
        );
    }

    #[test]
    fn test_fishing_tick_result_messages_contain_rarity() {
        let haven = HavenFishingBonuses::default();
        let mut rng = create_test_rng();
        let mut state = create_test_game_state();

        let session = FishingSession {
            spot_name: "Lake".to_string(),
            total_fish: 100,
            fish_caught: Vec::new(),
            items_found: Vec::new(),
            ticks_remaining: 1,
            phase: FishingPhase::Reeling,
        };
        state.active_fishing = Some(session);

        let result = tick_fishing_with_haven_result(&mut state, &mut rng, &haven, 0.0);

        // Catch message should contain one of the rarity names
        let catch_msg = result
            .messages
            .iter()
            .find(|m| m.contains("Caught"))
            .expect("Should have a catch message");
        let has_rarity = catch_msg.contains("Common")
            || catch_msg.contains("Uncommon")
            || catch_msg.contains("Rare")
            || catch_msg.contains("Epic")
            || catch_msg.contains("Legendary");
        assert!(
            has_rarity,
            "Catch message should contain rarity: {}",
            catch_msg
        );
    }

    #[test]
    fn test_god_item_fishing_reduction_stacks_with_haven() {
        let base_ticks = 100;
        let after_haven = apply_timer_reduction(base_ticks, 40.0); // 60 ticks
        assert_eq!(after_haven, 60);
        let after_god_item = apply_timer_reduction(after_haven, 50.0); // 30 ticks
        assert_eq!(after_god_item, 30);
        // Total reduction: 70% (multiplicative, not additive)
    }
}
