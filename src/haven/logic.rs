//! Haven build/upgrade logic and persistence.

use super::types::{haven_discovery_chance, tier_cost, Haven, HavenRoomId};
use crate::core::constants::STORMBREAKER_PRESTIGE_REQUIREMENT;
use rand::Rng;
use std::fs;
use std::io;
use std::path::PathBuf;

/// Check if a character can afford to build/upgrade a room
pub fn can_afford(room: HavenRoomId, haven: &Haven, prestige_rank: u32) -> bool {
    let next = match haven.next_tier(room) {
        Some(t) => t,
        None => return false,
    };
    let cost = tier_cost(room, next);
    prestige_rank >= cost
}

/// Attempt to build/upgrade a room, spending prestige ranks.
/// Returns (new_tier, prestige_spent) on success.
pub fn try_build_room(
    room: HavenRoomId,
    haven: &mut Haven,
    prestige_rank: &mut u32,
) -> Option<(u8, u32)> {
    if !haven.can_build(room) {
        return None;
    }
    let next = haven.next_tier(room)?;
    let cost = tier_cost(room, next);
    if *prestige_rank < cost {
        return None;
    }
    *prestige_rank -= cost;
    haven.build_room(room);
    Some((next, cost))
}

/// Check if the player can forge Stormbreaker.
/// Returns (has_leviathan, has_prestige, can_forge).
pub fn can_forge_stormbreaker(
    achievements: &crate::achievements::Achievements,
    prestige_rank: u32,
) -> (bool, bool, bool) {
    use crate::achievements::AchievementId;
    let has_leviathan = achievements.is_unlocked(AchievementId::StormLeviathan);
    let has_prestige = prestige_rank >= STORMBREAKER_PRESTIGE_REQUIREMENT;
    let can_forge = has_leviathan && has_prestige;
    (has_leviathan, has_prestige, can_forge)
}

/// Try to discover the Haven. Independent roll per tick.
/// Returns true if discovered this tick.
pub fn try_discover_haven<R: Rng>(haven: &mut Haven, prestige_rank: u32, rng: &mut R) -> bool {
    if haven.discovered {
        return false;
    }
    let chance = haven_discovery_chance(prestige_rank);
    if chance <= 0.0 {
        return false;
    }
    if rng.gen::<f64>() < chance {
        haven.discovered = true;
        return true;
    }
    false
}

/// Get the Haven save file path
pub fn haven_save_path() -> io::Result<PathBuf> {
    let home_dir = dirs::home_dir().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Could not determine home directory",
        )
    })?;
    Ok(home_dir.join(".quest").join("haven.json"))
}

/// Load Haven from disk, or return default if not found
pub fn load_haven() -> Haven {
    let path = match haven_save_path() {
        Ok(p) => p,
        Err(_) => return Haven::new(),
    };
    match fs::read_to_string(&path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => Haven::new(),
    }
}

/// Save Haven to disk
pub fn save_haven(haven: &Haven) -> io::Result<()> {
    let path = haven_save_path()?;
    let json = serde_json::to_string_pretty(haven)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_afford_basic() {
        let haven = Haven::new();
        // Hearthstone T1 costs 1 prestige rank
        assert!(can_afford(HavenRoomId::Hearthstone, &haven, 1));
        assert!(!can_afford(HavenRoomId::Hearthstone, &haven, 0));
    }

    #[test]
    fn test_can_afford_tier_2() {
        let mut haven = Haven::new();
        haven.build_room(HavenRoomId::Hearthstone); // T1
                                                    // Hearthstone T2 costs 2 prestige ranks (depth 0: 1/2/3)
        assert!(can_afford(HavenRoomId::Hearthstone, &haven, 2));
        assert!(!can_afford(HavenRoomId::Hearthstone, &haven, 1));
    }

    #[test]
    fn test_can_afford_maxed_room() {
        let mut haven = Haven::new();
        haven.build_room(HavenRoomId::Hearthstone);
        haven.build_room(HavenRoomId::Hearthstone);
        haven.build_room(HavenRoomId::Hearthstone); // T3
        assert!(!can_afford(HavenRoomId::Hearthstone, &haven, 100));
    }

    #[test]
    fn test_try_build_room_success() {
        let mut haven = Haven::new();
        let mut prestige = 10u32;
        let result = try_build_room(HavenRoomId::Hearthstone, &mut haven, &mut prestige);
        assert_eq!(result, Some((1, 1)));
        assert_eq!(prestige, 9);
        assert_eq!(haven.room_tier(HavenRoomId::Hearthstone), 1);
    }

    #[test]
    fn test_try_build_room_insufficient_funds() {
        let mut haven = Haven::new();
        let mut prestige = 0u32;
        let result = try_build_room(HavenRoomId::Hearthstone, &mut haven, &mut prestige);
        assert!(result.is_none());
        assert_eq!(haven.room_tier(HavenRoomId::Hearthstone), 0);
    }

    #[test]
    fn test_try_build_room_locked() {
        let mut haven = Haven::new();
        let mut prestige = 100u32;
        // Armory is locked (Hearthstone not built)
        let result = try_build_room(HavenRoomId::Armory, &mut haven, &mut prestige);
        assert!(result.is_none());
        assert_eq!(prestige, 100); // Not spent
    }

    #[test]
    fn test_try_discover_haven_below_p10() {
        let mut haven = Haven::new();
        let mut rng = rand::thread_rng();
        // Below P10, should never discover
        for _ in 0..100_000 {
            assert!(!try_discover_haven(&mut haven, 9, &mut rng));
        }
    }

    #[test]
    fn test_try_discover_haven_already_discovered() {
        let mut haven = Haven::new();
        haven.discovered = true;
        let mut rng = rand::thread_rng();
        assert!(!try_discover_haven(&mut haven, 20, &mut rng));
    }

    #[test]
    fn test_try_discover_haven_eventually_succeeds() {
        let mut haven = Haven::new();
        let mut rng = rand::thread_rng();
        let mut discovered = false;
        for _ in 0..1_000_000 {
            if try_discover_haven(&mut haven, 10, &mut rng) {
                discovered = true;
                break;
            }
        }
        assert!(discovered, "Should discover haven within 1M ticks at P10");
        assert!(haven.discovered);
    }

    #[test]
    fn test_build_full_branch_costs() {
        let mut haven = Haven::new();
        let mut prestige = 200u32;
        let initial_p = prestige;

        // Build full combat branch at T1
        // Costs: Hearthstone(1) + Armory(1) + TrainingYard(2) + TrophyHall(2)
        //        + Watchtower(2) + AlchemyLab(2) + WarRoom(3) = 13
        try_build_room(HavenRoomId::Hearthstone, &mut haven, &mut prestige);
        try_build_room(HavenRoomId::Armory, &mut haven, &mut prestige);
        try_build_room(HavenRoomId::TrainingYard, &mut haven, &mut prestige);
        try_build_room(HavenRoomId::TrophyHall, &mut haven, &mut prestige);
        try_build_room(HavenRoomId::Watchtower, &mut haven, &mut prestige);
        try_build_room(HavenRoomId::AlchemyLab, &mut haven, &mut prestige);
        try_build_room(HavenRoomId::WarRoom, &mut haven, &mut prestige);

        assert_eq!(initial_p - prestige, 13);
    }

    // =========================================================================
    // Prestige Token Economy Tests
    // =========================================================================

    #[test]
    fn test_tokens_deducted_on_successful_build() {
        let mut haven = Haven::new();
        let mut prestige = 10u32;

        // Build Hearthstone T1 costs 1 token
        let result = try_build_room(HavenRoomId::Hearthstone, &mut haven, &mut prestige);
        assert!(result.is_some());
        assert_eq!(prestige, 9);

        // Build Hearthstone T2 costs 2 tokens
        let result = try_build_room(HavenRoomId::Hearthstone, &mut haven, &mut prestige);
        assert!(result.is_some());
        assert_eq!(prestige, 7);

        // Build Hearthstone T3 costs 3 tokens
        let result = try_build_room(HavenRoomId::Hearthstone, &mut haven, &mut prestige);
        assert!(result.is_some());
        assert_eq!(prestige, 4);
    }

    #[test]
    fn test_tokens_not_deducted_on_failed_build() {
        let mut haven = Haven::new();
        let mut prestige = 5u32;

        // Try to build locked room - should fail, no deduction
        let result = try_build_room(HavenRoomId::Armory, &mut haven, &mut prestige);
        assert!(result.is_none());
        assert_eq!(prestige, 5);

        // Build Hearthstone to max
        haven.build_room(HavenRoomId::Hearthstone);
        haven.build_room(HavenRoomId::Hearthstone);
        haven.build_room(HavenRoomId::Hearthstone);

        // Try to build maxed room - should fail, no deduction
        let result = try_build_room(HavenRoomId::Hearthstone, &mut haven, &mut prestige);
        assert!(result.is_none());
        assert_eq!(prestige, 5);
    }

    #[test]
    fn test_insufficient_tokens_prevents_build() {
        let mut haven = Haven::new();
        let mut prestige = 0u32;

        // Can't afford anything with 0 tokens
        assert!(!can_afford(HavenRoomId::Hearthstone, &haven, prestige));
        let result = try_build_room(HavenRoomId::Hearthstone, &mut haven, &mut prestige);
        assert!(result.is_none());
        assert_eq!(haven.room_tier(HavenRoomId::Hearthstone), 0);
    }

    #[test]
    fn test_exact_token_amount_allows_build() {
        let mut haven = Haven::new();
        let mut prestige = 1u32; // Exactly enough for T1 Hearthstone

        assert!(can_afford(HavenRoomId::Hearthstone, &haven, prestige));
        let result = try_build_room(HavenRoomId::Hearthstone, &mut haven, &mut prestige);
        assert!(result.is_some());
        assert_eq!(prestige, 0);
        assert_eq!(haven.room_tier(HavenRoomId::Hearthstone), 1);
    }

    #[test]
    fn test_build_fishing_branch_costs() {
        let mut haven = Haven::new();
        let mut prestige = 200u32;
        let initial_p = prestige;

        // Build full fishing branch at T1
        // Costs: Hearthstone(1) + Bedroom(1) + Garden(2) + Library(2)
        //        + FishingDock(2) + Workshop(2) + Vault(3) = 13
        try_build_room(HavenRoomId::Hearthstone, &mut haven, &mut prestige);
        try_build_room(HavenRoomId::Bedroom, &mut haven, &mut prestige);
        try_build_room(HavenRoomId::Garden, &mut haven, &mut prestige);
        try_build_room(HavenRoomId::Library, &mut haven, &mut prestige);
        try_build_room(HavenRoomId::FishingDock, &mut haven, &mut prestige);
        try_build_room(HavenRoomId::Workshop, &mut haven, &mut prestige);
        try_build_room(HavenRoomId::Vault, &mut haven, &mut prestige);

        assert_eq!(initial_p - prestige, 13);
    }

    #[test]
    fn test_upgrade_costs_more_than_initial_build() {
        let mut haven = Haven::new();

        // T1 costs less than T2 costs less than T3
        let t1_cost = super::tier_cost(HavenRoomId::Hearthstone, 1);
        let t2_cost = super::tier_cost(HavenRoomId::Hearthstone, 2);
        let t3_cost = super::tier_cost(HavenRoomId::Hearthstone, 3);

        assert!(t1_cost < t2_cost);
        assert!(t2_cost < t3_cost);

        // Verify with actual building
        let mut prestige = 100u32;
        try_build_room(HavenRoomId::Hearthstone, &mut haven, &mut prestige);
        assert_eq!(100 - prestige, t1_cost);

        let before_t2 = prestige;
        try_build_room(HavenRoomId::Hearthstone, &mut haven, &mut prestige);
        assert_eq!(before_t2 - prestige, t2_cost);

        let before_t3 = prestige;
        try_build_room(HavenRoomId::Hearthstone, &mut haven, &mut prestige);
        assert_eq!(before_t3 - prestige, t3_cost);
    }

    #[test]
    fn test_deeper_rooms_cost_more() {
        // Depth 0 (root) is cheapest
        let root_cost = super::tier_cost(HavenRoomId::Hearthstone, 1);
        // Depth 1 costs more
        let d1_cost = super::tier_cost(HavenRoomId::Armory, 1);
        // Depth 2-3 costs more
        let d2_cost = super::tier_cost(HavenRoomId::TrainingYard, 1);
        // Depth 4 (capstone) costs most
        let d4_cost = super::tier_cost(HavenRoomId::WarRoom, 1);

        assert!(root_cost <= d1_cost);
        assert!(d1_cost <= d2_cost);
        assert!(d2_cost <= d4_cost);
    }

    #[test]
    fn test_multi_prestige_token_accumulation() {
        // Simulate earning tokens across multiple prestiges
        let mut total_earned = 0u32;
        let mut haven = Haven::new();
        haven.discovered = true;

        // Each prestige earns 1 token
        for prestige_count in 1..=20 {
            total_earned += 1; // Earn 1 token per prestige

            // After 10 prestiges, should be able to build a basic Haven
            if prestige_count == 10 {
                // Can afford: Hearthstone T1(1) + Armory T1(1) + TrainingYard T1(2) = 4 tokens min
                assert!(total_earned >= 4);
            }
        }

        // After 20 prestiges, should have enough for significant progress
        assert_eq!(total_earned, 20);
    }

    #[test]
    fn test_can_afford_respects_current_tier() {
        let mut haven = Haven::new();
        let prestige = 2u32;

        // Can afford T1 (costs 1)
        assert!(can_afford(HavenRoomId::Hearthstone, &haven, prestige));

        // Build T1
        haven.build_room(HavenRoomId::Hearthstone);

        // Can afford T2 (costs 2)
        assert!(can_afford(HavenRoomId::Hearthstone, &haven, prestige));

        // Build T2
        haven.build_room(HavenRoomId::Hearthstone);

        // Cannot afford T3 (costs 3, only have 2)
        assert!(!can_afford(HavenRoomId::Hearthstone, &haven, prestige));
    }

    // =========================================================================
    // Edge Cases
    // =========================================================================

    #[test]
    fn test_discovery_requires_p10() {
        let mut haven = Haven::new();
        let mut rng = rand::thread_rng();

        // P0-P9 cannot discover Haven
        for p in 0..10 {
            for _ in 0..1000 {
                if try_discover_haven(&mut haven, p, &mut rng) {
                    panic!("Should not discover Haven at P{}", p);
                }
            }
        }
    }

    #[test]
    fn test_building_order_matters() {
        let mut haven = Haven::new();
        let mut prestige = 100u32;

        // Cannot build Watchtower before its prerequisites
        // Watchtower requires: Hearthstone -> Armory -> TrainingYard -> Watchtower
        assert!(try_build_room(HavenRoomId::Watchtower, &mut haven, &mut prestige).is_none());

        // Build prerequisites in order
        try_build_room(HavenRoomId::Hearthstone, &mut haven, &mut prestige);
        assert!(try_build_room(HavenRoomId::Watchtower, &mut haven, &mut prestige).is_none()); // Still locked

        try_build_room(HavenRoomId::Armory, &mut haven, &mut prestige);
        assert!(try_build_room(HavenRoomId::Watchtower, &mut haven, &mut prestige).is_none()); // Still locked

        try_build_room(HavenRoomId::TrainingYard, &mut haven, &mut prestige);
        assert!(try_build_room(HavenRoomId::Watchtower, &mut haven, &mut prestige).is_some());
        // Now unlocked!
    }

    #[test]
    fn test_capstone_building_order_matters() {
        let mut haven = Haven::new();
        let mut prestige = 100u32;

        // Cannot build WarRoom (capstone) before BOTH parents
        // WarRoom requires: Watchtower AND AlchemyLab
        assert!(try_build_room(HavenRoomId::WarRoom, &mut haven, &mut prestige).is_none());

        // Build path to Watchtower
        try_build_room(HavenRoomId::Hearthstone, &mut haven, &mut prestige);
        try_build_room(HavenRoomId::Armory, &mut haven, &mut prestige);
        try_build_room(HavenRoomId::TrainingYard, &mut haven, &mut prestige);
        try_build_room(HavenRoomId::Watchtower, &mut haven, &mut prestige);

        // Still can't build WarRoom - need AlchemyLab too
        assert!(try_build_room(HavenRoomId::WarRoom, &mut haven, &mut prestige).is_none());

        // Build path to AlchemyLab (shares Armory with Watchtower path)
        try_build_room(HavenRoomId::TrophyHall, &mut haven, &mut prestige);
        try_build_room(HavenRoomId::AlchemyLab, &mut haven, &mut prestige);

        // Now WarRoom should be buildable
        assert!(try_build_room(HavenRoomId::WarRoom, &mut haven, &mut prestige).is_some());
    }

    // =========================================================================
    // Storm Forge Building Tests
    // =========================================================================

    #[test]
    fn test_storm_forge_costs_25_prestige() {
        // StormForge T1 costs 25 prestige ranks
        assert_eq!(super::tier_cost(HavenRoomId::StormForge, 1), 25);
    }

    #[test]
    fn test_storm_forge_only_has_one_tier() {
        // StormForge max tier is 1
        assert_eq!(HavenRoomId::StormForge.max_tier(), 1);

        // Tier 2 cost should be 0 (invalid)
        assert_eq!(super::tier_cost(HavenRoomId::StormForge, 2), 0);
        assert_eq!(super::tier_cost(HavenRoomId::StormForge, 3), 0);
    }

    #[test]
    fn test_storm_forge_requires_both_capstones() {
        let mut haven = Haven::new();

        // Build complete tree to WarRoom
        haven.build_room(HavenRoomId::Hearthstone);
        haven.build_room(HavenRoomId::Armory);
        haven.build_room(HavenRoomId::TrainingYard);
        haven.build_room(HavenRoomId::TrophyHall);
        haven.build_room(HavenRoomId::Watchtower);
        haven.build_room(HavenRoomId::AlchemyLab);
        haven.build_room(HavenRoomId::WarRoom);

        // Should NOT be able to build StormForge yet (needs Vault too)
        assert!(!haven.is_room_unlocked(HavenRoomId::StormForge));

        // Build path to Vault
        haven.build_room(HavenRoomId::Bedroom);
        haven.build_room(HavenRoomId::Garden);
        haven.build_room(HavenRoomId::Library);
        haven.build_room(HavenRoomId::FishingDock);
        haven.build_room(HavenRoomId::Workshop);
        haven.build_room(HavenRoomId::Vault);

        // Now StormForge should be unlocked
        assert!(haven.is_room_unlocked(HavenRoomId::StormForge));
    }

    #[test]
    fn test_storm_forge_cannot_build_without_25_prestige() {
        let mut haven = Haven::new();
        let mut prestige = 24u32; // One short of required

        // Build full tree to unlock StormForge
        build_full_tree_to_capstones(&mut haven);

        // Cannot build with insufficient prestige
        assert!(!can_afford(HavenRoomId::StormForge, &haven, prestige));
        assert!(try_build_room(HavenRoomId::StormForge, &mut haven, &mut prestige).is_none());
        assert_eq!(prestige, 24); // No prestige spent
    }

    #[test]
    fn test_storm_forge_can_build_with_exactly_25_prestige() {
        let mut haven = Haven::new();
        let mut prestige = 25u32; // Exactly the required amount

        // Build full tree to unlock StormForge
        build_full_tree_to_capstones(&mut haven);

        // Can build with exact amount
        assert!(can_afford(HavenRoomId::StormForge, &haven, prestige));
        let result = try_build_room(HavenRoomId::StormForge, &mut haven, &mut prestige);
        assert_eq!(result, Some((1, 25)));
        assert_eq!(prestige, 0); // All prestige spent
        assert_eq!(haven.room_tier(HavenRoomId::StormForge), 1);
    }

    #[test]
    fn test_storm_forge_built_grants_access() {
        let mut haven = Haven::new();

        assert!(!haven.has_storm_forge());

        // Build full tree including StormForge
        build_full_tree_to_capstones(&mut haven);
        haven.build_room(HavenRoomId::StormForge);

        assert!(haven.has_storm_forge());
    }

    #[test]
    fn test_storm_forge_cannot_upgrade_past_tier_1() {
        let mut haven = Haven::new();

        // Build StormForge
        build_full_tree_to_capstones(&mut haven);
        haven.build_room(HavenRoomId::StormForge);

        assert_eq!(haven.room_tier(HavenRoomId::StormForge), 1);

        // Cannot build again (already at max tier)
        assert!(!haven.can_build(HavenRoomId::StormForge));
        assert!(haven.next_tier(HavenRoomId::StormForge).is_none());
    }

    // Helper function to build full tree to both capstones (WarRoom and Vault)
    fn build_full_tree_to_capstones(haven: &mut Haven) {
        // Root
        haven.build_room(HavenRoomId::Hearthstone);

        // Combat branch to WarRoom
        haven.build_room(HavenRoomId::Armory);
        haven.build_room(HavenRoomId::TrainingYard);
        haven.build_room(HavenRoomId::TrophyHall);
        haven.build_room(HavenRoomId::Watchtower);
        haven.build_room(HavenRoomId::AlchemyLab);
        haven.build_room(HavenRoomId::WarRoom);

        // QoL branch to Vault
        haven.build_room(HavenRoomId::Bedroom);
        haven.build_room(HavenRoomId::Garden);
        haven.build_room(HavenRoomId::Library);
        haven.build_room(HavenRoomId::FishingDock);
        haven.build_room(HavenRoomId::Workshop);
        haven.build_room(HavenRoomId::Vault);
    }
}
