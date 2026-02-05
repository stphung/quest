//! Haven build/upgrade logic and persistence.

use super::types::{haven_discovery_chance, tier_cost, Haven, HavenRoomId};
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
    let cost = tier_cost(next);
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
    let cost = tier_cost(next);
    if *prestige_rank < cost {
        return None;
    }
    *prestige_rank -= cost;
    haven.build_room(room);
    Some((next, cost))
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
                                                    // T2 costs 3 prestige ranks
        assert!(can_afford(HavenRoomId::Hearthstone, &haven, 3));
        assert!(!can_afford(HavenRoomId::Hearthstone, &haven, 2));
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
        try_build_room(HavenRoomId::Hearthstone, &mut haven, &mut prestige);
        try_build_room(HavenRoomId::Armory, &mut haven, &mut prestige);
        try_build_room(HavenRoomId::TrainingYard, &mut haven, &mut prestige);
        try_build_room(HavenRoomId::TrophyHall, &mut haven, &mut prestige);
        try_build_room(HavenRoomId::Watchtower, &mut haven, &mut prestige);
        try_build_room(HavenRoomId::AlchemyLab, &mut haven, &mut prestige);
        try_build_room(HavenRoomId::WarRoom, &mut haven, &mut prestige);

        // 7 rooms at T1 = 7 prestige ranks
        assert_eq!(initial_p - prestige, 7);
    }
}
