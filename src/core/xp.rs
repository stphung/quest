use super::constants::*;
use super::game_state::GameState;
use crate::character::attributes::AttributeType;
use rand::RngExt;

/// Calculates the XP required to reach the next level
pub fn xp_for_next_level(level: u32) -> u64 {
    (XP_CURVE_BASE * f64::powf(level as f64, XP_CURVE_EXPONENT)) as u64
}

/// Calculates the prestige multiplier for XP gains including CHA bonus
pub fn prestige_multiplier(rank: u32, cha_modifier: i32) -> f64 {
    let base = crate::character::prestige::get_prestige_tier(rank).multiplier;
    base + (cha_modifier as f64 * PRESTIGE_MULT_PER_CHA_MODIFIER)
}

/// Calculates the XP gained per tick based on prestige rank and WIS
pub fn xp_gain_per_tick(prestige_rank: u32, wis_modifier: i32, cha_modifier: i32) -> f64 {
    let prestige_mult = prestige_multiplier(prestige_rank, cha_modifier);
    let wis_mult = 1.0 + (wis_modifier as f64 * XP_MULT_PER_WIS_MODIFIER);
    BASE_XP_PER_TICK * prestige_mult * wis_mult
}

/// Distributes 3 attribute points randomly among non-capped attributes
pub fn distribute_level_up_points(state: &mut GameState) -> Vec<AttributeType> {
    let mut rng = rand::rng();
    let cap = state.get_attribute_cap();
    let mut increased = Vec::new();

    let mut points = LEVEL_UP_ATTRIBUTE_POINTS;
    let mut attempts = 0;
    let max_attempts = LEVEL_UP_MAX_DISTRIBUTION_ATTEMPTS;

    while points > 0 && attempts < max_attempts {
        let attr_index = rng.random_range(0..NUM_ATTRIBUTES);
        let attr = AttributeType::all()[attr_index];

        if state.attributes.get(attr) < cap {
            state.attributes.increment(attr);
            increased.push(attr);
            points -= 1;
        }

        attempts += 1;
    }

    increased
}

/// Applies XP to the character and processes any level-ups
/// Returns (number of level-ups, attributes increased)
pub fn apply_tick_xp(state: &mut GameState, xp_gain: f64) -> (u32, Vec<AttributeType>) {
    state.xp_this_second += xp_gain as u64;
    state.character_xp += xp_gain as u64;
    state.combat_seconds_this_tick = true;

    let mut levelups = 0;
    let mut all_increased = Vec::new();

    loop {
        let xp_needed = xp_for_next_level(state.character_level);

        if state.character_xp >= xp_needed {
            state.character_xp -= xp_needed;
            state.character_level += 1;
            levelups += 1;

            let increased = distribute_level_up_points(state);
            all_increased.extend(increased);

            // Mark derived stats as needing recalculation on next tick
            state.invalidate_derived_stats();
        } else {
            break;
        }
    }

    (levelups, all_increased)
}

/// Calculates XP bonus from killing an enemy
/// `haven_xp_gain_percent` is the Training Yard bonus (0.0 if not built)
pub fn combat_kill_xp(passive_xp_rate: f64, haven_xp_gain_percent: f64) -> u64 {
    let ticks = rand::rng().random_range(COMBAT_XP_MIN_TICKS..=COMBAT_XP_MAX_TICKS);
    let base_xp = passive_xp_rate * ticks as f64;
    // Apply Haven Training Yard bonus
    (base_xp * (1.0 + haven_xp_gain_percent / 100.0)) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xp_for_next_level() {
        assert_eq!(xp_for_next_level(1), 100);
        assert_eq!(xp_for_next_level(2), 282);
        assert_eq!(xp_for_next_level(10), 3162);
    }

    #[test]
    fn test_prestige_multiplier() {
        // Rank 0, CHA 10 (+0): 1.0 + 0 = 1.0
        assert_eq!(prestige_multiplier(0, 0), 1.0);

        // Rank 1, CHA 10 (+0): 1.5 + 0 = 1.5
        assert_eq!(prestige_multiplier(1, 0), 1.5);

        // Rank 1, CHA 16 (+3): 1.5 + 0.3 = 1.8
        assert_eq!(prestige_multiplier(1, 3), 1.8);
    }

    #[test]
    fn test_xp_gain_per_tick() {
        // Rank 0, WIS 10 (+0), CHA 10 (+0): 1.0 * 1.0 * 1.0 = 1.0
        assert_eq!(xp_gain_per_tick(0, 0, 0), 1.0);

        // Rank 1, WIS 20 (+5), CHA 16 (+3): 1.8 * 1.25 = 2.25
        assert_eq!(xp_gain_per_tick(1, 5, 3), 2.25);
    }

    #[test]
    fn test_distribute_level_up_points() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let increased = distribute_level_up_points(&mut state);

        // Should distribute 3 points
        assert_eq!(increased.len(), 3);

        // Total attribute sum should be 60 + 3 = 63
        let mut sum = 0;
        for attr in AttributeType::all() {
            sum += state.attributes.get(attr);
        }
        assert_eq!(sum, 63);
    }

    #[test]
    fn test_distribute_respects_caps() {
        let mut state = GameState::new("Test Hero".to_string(), 0);

        // Set all attributes to cap - 1 (prestige 0 = cap 20)
        for attr in AttributeType::all() {
            state.attributes.set(attr, 19);
        }

        let increased = distribute_level_up_points(&mut state);
        assert_eq!(increased.len(), 3);

        // All should be at cap now (20)
        for attr in increased {
            assert!(state.attributes.get(attr) <= 20);
        }
    }

    #[test]
    fn test_apply_tick_xp_no_levelup() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let (levelups, increased) = apply_tick_xp(&mut state, 50.0);

        assert_eq!(levelups, 0);
        assert_eq!(increased.len(), 0);
        assert_eq!(state.character_level, 1);
        assert_eq!(state.character_xp, 50);
    }

    #[test]
    fn test_apply_tick_xp_single_levelup() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let (levelups, increased) = apply_tick_xp(&mut state, 100.0);

        assert_eq!(levelups, 1);
        assert_eq!(increased.len(), 3);
        assert_eq!(state.character_level, 2);
        assert_eq!(state.character_xp, 0);
    }

    #[test]
    fn test_apply_tick_xp_multiple_levelups() {
        let mut state = GameState::new("Test Hero".to_string(), 0);

        // Give enough XP for multiple level ups
        // Level 1->2: 100, Level 2->3: 282, Total: 382
        let (levelups, increased) = apply_tick_xp(&mut state, 400.0);

        assert_eq!(levelups, 2);
        assert_eq!(increased.len(), 6); // 3 points per level * 2 levels
        assert_eq!(state.character_level, 3);
    }

    #[test]
    fn test_combat_kill_xp() {
        let xp = combat_kill_xp(1.0, 0.0);
        assert!((200..=400).contains(&xp));
    }

    #[test]
    fn test_combat_kill_xp_with_haven_bonus() {
        let mut total_no_bonus = 0u64;
        let mut total_with_bonus = 0u64;
        let trials = 1000;

        for _ in 0..trials {
            total_no_bonus += combat_kill_xp(1.0, 0.0);
            total_with_bonus += combat_kill_xp(1.0, 30.0);
        }

        let avg_no_bonus = total_no_bonus as f64 / trials as f64;
        let avg_with_bonus = total_with_bonus as f64 / trials as f64;
        let ratio = avg_with_bonus / avg_no_bonus;

        assert!(
            (1.25..=1.35).contains(&ratio),
            "Haven +30% XP should increase average XP by ~30%, got {:.2}x",
            ratio
        );
    }

    #[test]
    fn test_xp_for_next_level_scaling() {
        let xp_1 = xp_for_next_level(1);
        let xp_5 = xp_for_next_level(5);
        let xp_10 = xp_for_next_level(10);
        let xp_50 = xp_for_next_level(50);

        assert!(xp_1 < xp_5);
        assert!(xp_5 < xp_10);
        assert!(xp_10 < xp_50);
    }

    #[test]
    fn test_prestige_multiplier_negative_charisma() {
        let mult = prestige_multiplier(1, -2);
        assert_eq!(mult, 1.3);
    }

    #[test]
    fn test_distribute_when_all_at_cap() {
        let mut state = GameState::new("Test Hero".to_string(), 0);

        for attr in AttributeType::all() {
            state.attributes.set(attr, 20);
        }

        let increased = distribute_level_up_points(&mut state);
        assert!(increased.len() < 3);
    }

    #[test]
    fn test_xp_for_next_level_at_level_100() {
        let xp = xp_for_next_level(100);
        assert_eq!(xp, 100_000);
    }

    #[test]
    fn test_xp_for_next_level_at_level_500() {
        let xp = xp_for_next_level(500);
        let expected = (100.0 * 500.0_f64.powf(1.5)) as u64;
        assert_eq!(xp, expected);
        assert!(xp > 1_000_000);
    }

    #[test]
    fn test_xp_for_next_level_at_level_1000() {
        let xp = xp_for_next_level(1000);
        let expected = (100.0 * 1000.0_f64.powf(1.5)) as u64;
        assert_eq!(xp, expected);
        assert!(xp > 3_000_000);
    }

    #[test]
    fn test_xp_curve_monotonically_increasing() {
        let mut prev_xp = 0u64;
        for level in 1..=500 {
            let xp = xp_for_next_level(level);
            assert!(
                xp > prev_xp,
                "XP at level {level} ({xp}) must exceed level {} ({prev_xp})",
                level - 1
            );
            prev_xp = xp;
        }
    }

    #[test]
    fn test_xp_at_level_1_is_base_value() {
        assert_eq!(xp_for_next_level(1), XP_CURVE_BASE as u64);
    }

    #[test]
    fn test_prestige_multiplier_at_p50() {
        let mult = prestige_multiplier(50, 0);
        let expected = 1.0 + 0.5 * 50.0_f64.powf(0.7);
        assert!((mult - expected).abs() < 0.01);
    }

    #[test]
    fn test_prestige_multiplier_at_p100() {
        let mult = prestige_multiplier(100, 0);
        let expected = 1.0 + 0.5 * 100.0_f64.powf(0.7);
        assert!((mult - expected).abs() < 0.01);
        assert!(mult > 10.0);
    }

    #[test]
    fn test_prestige_multiplier_with_very_low_cha() {
        let mult = prestige_multiplier(1, -4);
        assert!((mult - 1.1).abs() < 0.01);
    }

    #[test]
    fn test_prestige_multiplier_with_very_high_cha() {
        let mult = prestige_multiplier(1, 10);
        assert!((mult - 2.5).abs() < 0.01);
    }

    #[test]
    fn test_prestige_multiplier_always_at_least_1() {
        let mult_p0 = prestige_multiplier(0, -4);
        assert!((mult_p0 - 0.6).abs() < 0.01);

        let mult_p10 = prestige_multiplier(10, -4);
        assert!(mult_p10 > 1.0);
    }

    #[test]
    fn test_distribute_when_all_at_cap_returns_empty() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let cap = state.get_attribute_cap();

        for attr in AttributeType::all() {
            state.attributes.set(attr, cap);
        }

        let increased = distribute_level_up_points(&mut state);
        assert!(
            increased.is_empty(),
            "Should distribute zero points when all attributes at cap"
        );

        for attr in AttributeType::all() {
            assert_eq!(state.attributes.get(attr), cap);
        }
    }

    #[test]
    fn test_distribute_when_only_one_below_cap() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let cap = state.get_attribute_cap();

        for attr in AttributeType::all() {
            state.attributes.set(attr, cap);
        }
        state.attributes.set(AttributeType::Strength, cap - 3);

        let increased = distribute_level_up_points(&mut state);
        assert_eq!(increased.len(), 3);

        for attr in &increased {
            assert_eq!(*attr, AttributeType::Strength);
        }
        assert_eq!(state.attributes.get(AttributeType::Strength), cap);
    }

    #[test]
    fn test_distribute_with_high_prestige_cap() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        state.prestige_rank = 10;
        let cap = state.get_attribute_cap();
        assert_eq!(cap, 70);

        for attr in AttributeType::all() {
            state.attributes.set(attr, 69);
        }

        let increased = distribute_level_up_points(&mut state);
        assert_eq!(increased.len(), 3);

        for attr in &increased {
            assert_eq!(state.attributes.get(*attr), 70);
        }
    }

    #[test]
    fn test_distribute_with_p20_cap() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        state.prestige_rank = 20;
        assert_eq!(state.get_attribute_cap(), 120);

        let increased = distribute_level_up_points(&mut state);
        assert_eq!(increased.len(), 3);
    }
}
