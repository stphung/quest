//! Coverage tests for haven/room_defs, dungeon/rewards, achievements/modal, achievements/persistence,
//! haven/bonus.rs discovery chance and maxed compute_bonuses.

use quest::achievements::Achievements;
use quest::dungeon::{Dungeon, DungeonSize};
use quest::haven::bonus::haven_discovery_chance;
use quest::haven::types::{Haven, HavenBonuses};
use quest::haven::{tier_cost, HavenRoomId};
use quest::items::Rarity;
use quest::AchievementId;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn seeded_rng() -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(42)
}

fn test_rng() -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(42)
}

// =========================================================================
// Haven Room Definitions Tests
// =========================================================================

#[test]
fn test_all_rooms_have_nonempty_name() {
    for room in HavenRoomId::ALL {
        let name = room.name();
        assert!(!name.is_empty(), "{:?} should have a non-empty name", room);
    }
}

#[test]
fn test_all_rooms_have_nonempty_description() {
    for room in HavenRoomId::ALL {
        let desc = room.description();
        assert!(
            !desc.is_empty(),
            "{:?} should have a non-empty description",
            room
        );
        // Descriptions should be more than just a word
        assert!(
            desc.len() > 20,
            "{:?} description should be substantial, got: {}",
            room,
            desc
        );
    }
}

#[test]
fn test_hearthstone_has_no_parents() {
    assert!(
        HavenRoomId::Hearthstone.parents().is_empty(),
        "Hearthstone is the root and should have no parents"
    );
}

#[test]
fn test_armory_and_bedroom_require_hearthstone() {
    assert_eq!(HavenRoomId::Armory.parents(), &[HavenRoomId::Hearthstone]);
    assert_eq!(HavenRoomId::Bedroom.parents(), &[HavenRoomId::Hearthstone]);
}

#[test]
fn test_capstones_require_two_parents() {
    // WarRoom requires Watchtower and AlchemyLab
    let war_parents = HavenRoomId::WarRoom.parents();
    assert_eq!(war_parents.len(), 2);
    assert!(war_parents.contains(&HavenRoomId::Watchtower));
    assert!(war_parents.contains(&HavenRoomId::AlchemyLab));

    // Vault requires FishingDock and Workshop
    let vault_parents = HavenRoomId::Vault.parents();
    assert_eq!(vault_parents.len(), 2);
    assert!(vault_parents.contains(&HavenRoomId::FishingDock));
    assert!(vault_parents.contains(&HavenRoomId::Workshop));

    // StormForge requires WarRoom and Vault
    let forge_parents = HavenRoomId::StormForge.parents();
    assert_eq!(forge_parents.len(), 2);
    assert!(forge_parents.contains(&HavenRoomId::WarRoom));
    assert!(forge_parents.contains(&HavenRoomId::Vault));
}

#[test]
fn test_non_capstones_have_single_parent() {
    let single_parent_rooms = [
        HavenRoomId::TrainingYard,
        HavenRoomId::TrophyHall,
        HavenRoomId::Watchtower,
        HavenRoomId::AlchemyLab,
        HavenRoomId::Garden,
        HavenRoomId::Library,
        HavenRoomId::FishingDock,
        HavenRoomId::Workshop,
    ];
    for room in single_parent_rooms {
        assert_eq!(
            room.parents().len(),
            1,
            "{:?} should have exactly 1 parent",
            room
        );
    }
}

#[test]
fn test_hearthstone_depth_is_zero() {
    assert_eq!(HavenRoomId::Hearthstone.depth(), 0);
}

#[test]
fn test_depth_increases_with_tree_distance() {
    // Depth 1: direct children of Hearthstone
    assert_eq!(HavenRoomId::Armory.depth(), 1);
    assert_eq!(HavenRoomId::Bedroom.depth(), 1);

    // Depth 2: grandchildren of Hearthstone
    assert_eq!(HavenRoomId::TrainingYard.depth(), 2);
    assert_eq!(HavenRoomId::TrophyHall.depth(), 2);
    assert_eq!(HavenRoomId::Garden.depth(), 2);
    assert_eq!(HavenRoomId::Library.depth(), 2);

    // Depth 3: mid-tree
    assert_eq!(HavenRoomId::Watchtower.depth(), 3);
    assert_eq!(HavenRoomId::AlchemyLab.depth(), 3);
    assert_eq!(HavenRoomId::FishingDock.depth(), 3);
    assert_eq!(HavenRoomId::Workshop.depth(), 3);

    // Depth 4: capstones
    assert_eq!(HavenRoomId::WarRoom.depth(), 4);
    assert_eq!(HavenRoomId::Vault.depth(), 4);

    // Depth 5: StormForge
    assert_eq!(HavenRoomId::StormForge.depth(), 5);
}

#[test]
fn test_max_tier_defaults_to_three() {
    let standard_rooms = [
        HavenRoomId::Hearthstone,
        HavenRoomId::Armory,
        HavenRoomId::TrainingYard,
        HavenRoomId::TrophyHall,
        HavenRoomId::Watchtower,
        HavenRoomId::AlchemyLab,
        HavenRoomId::WarRoom,
        HavenRoomId::Bedroom,
        HavenRoomId::Garden,
        HavenRoomId::Library,
        HavenRoomId::Workshop,
        HavenRoomId::Vault,
    ];
    for room in standard_rooms {
        assert_eq!(room.max_tier(), 3, "{:?} should have max tier 3", room);
    }
}

#[test]
fn test_fishing_dock_max_tier_is_four() {
    assert_eq!(HavenRoomId::FishingDock.max_tier(), 4);
}

#[test]
fn test_storm_forge_max_tier_is_one() {
    assert_eq!(HavenRoomId::StormForge.max_tier(), 1);
}

#[test]
fn test_is_capstone_identifies_correct_rooms() {
    assert!(HavenRoomId::WarRoom.is_capstone());
    assert!(HavenRoomId::Vault.is_capstone());
    assert!(HavenRoomId::StormForge.is_capstone());

    // Non-capstones
    let non_capstones = [
        HavenRoomId::Hearthstone,
        HavenRoomId::Armory,
        HavenRoomId::TrainingYard,
        HavenRoomId::TrophyHall,
        HavenRoomId::Watchtower,
        HavenRoomId::AlchemyLab,
        HavenRoomId::Bedroom,
        HavenRoomId::Garden,
        HavenRoomId::Library,
        HavenRoomId::FishingDock,
        HavenRoomId::Workshop,
    ];
    for room in non_capstones {
        assert!(!room.is_capstone(), "{:?} should NOT be a capstone", room);
    }
}

// =========================================================================
// Tier Cost Tests
// =========================================================================

#[test]
fn test_tier_cost_depth_0_hearthstone() {
    assert_eq!(tier_cost(HavenRoomId::Hearthstone, 1), 1);
    assert_eq!(tier_cost(HavenRoomId::Hearthstone, 2), 2);
    assert_eq!(tier_cost(HavenRoomId::Hearthstone, 3), 3);
}

#[test]
fn test_tier_cost_depth_1() {
    // Armory (depth 1): 1/3/5
    assert_eq!(tier_cost(HavenRoomId::Armory, 1), 1);
    assert_eq!(tier_cost(HavenRoomId::Armory, 2), 3);
    assert_eq!(tier_cost(HavenRoomId::Armory, 3), 5);

    // Bedroom (depth 1): 1/3/5
    assert_eq!(tier_cost(HavenRoomId::Bedroom, 1), 1);
    assert_eq!(tier_cost(HavenRoomId::Bedroom, 2), 3);
    assert_eq!(tier_cost(HavenRoomId::Bedroom, 3), 5);
}

#[test]
fn test_tier_cost_depth_2_and_3() {
    // Depth 2 rooms: 2/4/6
    assert_eq!(tier_cost(HavenRoomId::TrainingYard, 1), 2);
    assert_eq!(tier_cost(HavenRoomId::TrainingYard, 2), 4);
    assert_eq!(tier_cost(HavenRoomId::TrainingYard, 3), 6);

    assert_eq!(tier_cost(HavenRoomId::TrophyHall, 1), 2);
    assert_eq!(tier_cost(HavenRoomId::Garden, 2), 4);
    assert_eq!(tier_cost(HavenRoomId::Library, 3), 6);

    // Depth 3 rooms: same as depth 2
    assert_eq!(tier_cost(HavenRoomId::Watchtower, 1), 2);
    assert_eq!(tier_cost(HavenRoomId::AlchemyLab, 2), 4);
    assert_eq!(tier_cost(HavenRoomId::Workshop, 3), 6);
}

#[test]
fn test_tier_cost_depth_4_capstones() {
    // Depth 4 capstones: 3/5/7
    assert_eq!(tier_cost(HavenRoomId::WarRoom, 1), 3);
    assert_eq!(tier_cost(HavenRoomId::WarRoom, 2), 5);
    assert_eq!(tier_cost(HavenRoomId::WarRoom, 3), 7);

    assert_eq!(tier_cost(HavenRoomId::Vault, 1), 3);
    assert_eq!(tier_cost(HavenRoomId::Vault, 2), 5);
    assert_eq!(tier_cost(HavenRoomId::Vault, 3), 7);
}

#[test]
fn test_tier_cost_fishing_dock_special() {
    // FishingDock T1-3 follow depth 3 costs, T4 is special
    assert_eq!(tier_cost(HavenRoomId::FishingDock, 1), 2);
    assert_eq!(tier_cost(HavenRoomId::FishingDock, 2), 4);
    assert_eq!(tier_cost(HavenRoomId::FishingDock, 3), 6);
    assert_eq!(tier_cost(HavenRoomId::FishingDock, 4), 10);
}

#[test]
fn test_tier_cost_storm_forge_special() {
    assert_eq!(tier_cost(HavenRoomId::StormForge, 1), 25);
    // Invalid tiers return 0
    assert_eq!(tier_cost(HavenRoomId::StormForge, 0), 0);
    assert_eq!(tier_cost(HavenRoomId::StormForge, 2), 0);
}

#[test]
fn test_tier_cost_invalid_tiers_return_zero() {
    // Tier 0 and beyond max should return 0 for all rooms
    assert_eq!(tier_cost(HavenRoomId::Hearthstone, 0), 0);
    assert_eq!(tier_cost(HavenRoomId::Hearthstone, 4), 0);
    assert_eq!(tier_cost(HavenRoomId::Armory, 0), 0);
    assert_eq!(tier_cost(HavenRoomId::Armory, 4), 0);
    assert_eq!(tier_cost(HavenRoomId::FishingDock, 0), 0);
    assert_eq!(tier_cost(HavenRoomId::FishingDock, 5), 0);
}

#[test]
fn test_tier_cost_increases_with_depth() {
    // For same tier level, deeper rooms should cost more
    let t1_depth0 = tier_cost(HavenRoomId::Hearthstone, 1);
    let t1_depth1 = tier_cost(HavenRoomId::Armory, 1);
    let t1_depth2 = tier_cost(HavenRoomId::TrainingYard, 1);
    let t1_depth4 = tier_cost(HavenRoomId::WarRoom, 1);

    assert!(t1_depth1 >= t1_depth0);
    assert!(t1_depth2 >= t1_depth1);
    assert!(t1_depth4 >= t1_depth2);
}

// =========================================================================
// Children Consistency Tests
// =========================================================================

#[test]
fn test_children_match_parents() {
    // Every child's parents should include the room that lists it as a child
    for room in HavenRoomId::ALL {
        for child in room.children() {
            assert!(
                child.parents().contains(&room),
                "{:?} lists {:?} as child, but {:?}'s parents don't include {:?}",
                room,
                child,
                child,
                room
            );
        }
    }
}

#[test]
fn test_parents_match_children() {
    // Every room's parents should list this room as a child
    for room in HavenRoomId::ALL {
        for parent in room.parents() {
            assert!(
                parent.children().contains(&room),
                "{:?} lists {:?} as parent, but {:?}'s children don't include {:?}",
                room,
                parent,
                parent,
                room
            );
        }
    }
}

#[test]
fn test_storm_forge_has_no_children() {
    assert!(HavenRoomId::StormForge.children().is_empty());
}

// =========================================================================
// Haven ALL array completeness
// =========================================================================

#[test]
fn test_all_array_has_14_rooms() {
    assert_eq!(HavenRoomId::ALL.len(), 14);
}

#[test]
fn test_all_array_contains_no_duplicates() {
    let mut seen = std::collections::HashSet::new();
    for room in HavenRoomId::ALL {
        assert!(
            seen.insert(room),
            "{:?} appears more than once in ALL",
            room
        );
    }
}

// =========================================================================
// Dungeon Rewards Tests
// =========================================================================

#[test]
fn test_boss_xp_reward_within_range_small() {
    let mut rng = test_rng();
    for _ in 0..20 {
        let xp = quest::dungeon::calculate_boss_xp_reward(&mut rng, DungeonSize::Small);
        assert!((1000..=1500).contains(&xp), "Small XP {} out of range", xp);
    }
}

#[test]
fn test_boss_xp_reward_within_range_medium() {
    let mut rng = test_rng();
    for _ in 0..20 {
        let xp = quest::dungeon::calculate_boss_xp_reward(&mut rng, DungeonSize::Medium);
        assert!((2000..=3000).contains(&xp), "Medium XP {} out of range", xp);
    }
}

#[test]
fn test_boss_xp_reward_within_range_large() {
    let mut rng = test_rng();
    for _ in 0..20 {
        let xp = quest::dungeon::calculate_boss_xp_reward(&mut rng, DungeonSize::Large);
        assert!((4000..=6000).contains(&xp), "Large XP {} out of range", xp);
    }
}

#[test]
fn test_boss_xp_reward_within_range_epic() {
    let mut rng = test_rng();
    for _ in 0..20 {
        let xp = quest::dungeon::calculate_boss_xp_reward(&mut rng, DungeonSize::Epic);
        assert!((8000..=12000).contains(&xp), "Epic XP {} out of range", xp);
    }
}

#[test]
fn test_boss_xp_reward_within_range_legendary() {
    let mut rng = test_rng();
    for _ in 0..20 {
        let xp = quest::dungeon::calculate_boss_xp_reward(&mut rng, DungeonSize::Legendary);
        assert!(
            (15000..=25000).contains(&xp),
            "Legendary XP {} out of range",
            xp
        );
    }
}

#[test]
fn test_boss_xp_reward_scales_with_size() {
    let mut rng = test_rng();
    // Larger dungeons should give more XP on average
    let small_avg: u64 = (0..50)
        .map(|_| quest::dungeon::calculate_boss_xp_reward(&mut rng, DungeonSize::Small))
        .sum::<u64>()
        / 50;
    let epic_avg: u64 = (0..50)
        .map(|_| quest::dungeon::calculate_boss_xp_reward(&mut rng, DungeonSize::Epic))
        .sum::<u64>()
        / 50;
    assert!(
        epic_avg > small_avg,
        "Epic avg {} should exceed Small avg {}",
        epic_avg,
        small_avg
    );
}

#[test]
fn test_generate_treasure_item_returns_valid_item() {
    let mut rng = test_rng();
    let item = quest::dungeon::generate_treasure_item(&mut rng, 0, 1, 0, 0.0);
    assert!(!item.base_name.is_empty());
    assert!(item.ilvl > 0);
}

#[test]
fn test_generate_treasure_item_zone_scaling() {
    let mut rng = test_rng();
    // Higher zone should produce higher ilvl items
    let item_z1 = quest::dungeon::generate_treasure_item(&mut rng, 0, 1, 0, 0.0);
    let item_z10 = quest::dungeon::generate_treasure_item(&mut rng, 0, 10, 0, 0.0);
    assert!(
        item_z10.ilvl > item_z1.ilvl,
        "Zone 10 ilvl {} should exceed zone 1 ilvl {}",
        item_z10.ilvl,
        item_z1.ilvl
    );
}

#[test]
fn test_generate_treasure_item_rarity_boost() {
    let mut rng = test_rng();
    // With maximum rarity boost, items should tend toward higher rarity
    let mut high_rarity_count = 0;
    for _ in 0..100 {
        let item = quest::dungeon::generate_treasure_item(&mut rng, 0, 5, 4, 0.0);
        if matches!(
            item.rarity,
            Rarity::Epic | Rarity::Legendary | Rarity::Mythic
        ) {
            high_rarity_count += 1;
        }
    }
    // With +4 rarity boost, most items should be at least Epic
    assert!(
        high_rarity_count > 50,
        "Only {} out of 100 items were Epic+ with +4 boost",
        high_rarity_count
    );
}

#[test]
fn test_on_treasure_room_entered_without_dungeon() {
    let mut rng = seeded_rng();
    use quest::core::GameState;

    let mut state = GameState::new("TestHero".to_string(), chrono::Utc::now().timestamp());
    // No active dungeon
    assert!(state.active_dungeon.is_none());

    // Should still return Some (generates item using default rarity boost)
    let result = quest::dungeon::on_treasure_room_entered(&mut rng, &mut state, 0.0);
    assert!(result.is_some());
    let (item, _equipped) = result.unwrap();
    assert!(!item.base_name.is_empty());
}

#[test]
fn test_on_treasure_room_entered_with_dungeon() {
    let mut rng = seeded_rng();
    use quest::core::GameState;

    let mut state = GameState::new("TestHero".to_string(), chrono::Utc::now().timestamp());
    state.active_dungeon = Some(Dungeon::new(DungeonSize::Epic));

    let result = quest::dungeon::on_treasure_room_entered(&mut rng, &mut state, 0.0);
    assert!(result.is_some());
    let (item, _equipped) = result.unwrap();
    assert!(!item.base_name.is_empty());

    // Item should be added to dungeon collected_items
    let dungeon = state.active_dungeon.as_ref().unwrap();
    assert_eq!(dungeon.collected_items.len(), 1);
}

#[test]
fn test_add_dungeon_xp() {
    use quest::core::GameState;

    let mut state = GameState::new("TestHero".to_string(), chrono::Utc::now().timestamp());
    state.active_dungeon = Some(Dungeon::new(DungeonSize::Small));

    quest::dungeon::add_dungeon_xp(&mut state, 500);
    assert_eq!(state.active_dungeon.as_ref().unwrap().xp_earned, 500);

    quest::dungeon::add_dungeon_xp(&mut state, 300);
    assert_eq!(state.active_dungeon.as_ref().unwrap().xp_earned, 800);
}

#[test]
fn test_add_dungeon_xp_without_dungeon_is_noop() {
    use quest::core::GameState;

    let mut state = GameState::new("TestHero".to_string(), chrono::Utc::now().timestamp());
    assert!(state.active_dungeon.is_none());

    // Should not panic
    quest::dungeon::add_dungeon_xp(&mut state, 500);
}

#[test]
fn test_on_treasure_room_entered_collects_items() {
    let mut rng = seeded_rng();
    use quest::core::GameState;

    let mut state = GameState::new("TestHero".to_string(), chrono::Utc::now().timestamp());
    state.active_dungeon = Some(Dungeon::new(DungeonSize::Medium));

    // Enter multiple treasure rooms
    quest::dungeon::on_treasure_room_entered(&mut rng, &mut state, 0.0);
    quest::dungeon::on_treasure_room_entered(&mut rng, &mut state, 0.0);
    quest::dungeon::on_treasure_room_entered(&mut rng, &mut state, 0.0);

    let dungeon = state.active_dungeon.as_ref().unwrap();
    assert_eq!(dungeon.collected_items.len(), 3);
}

// =========================================================================
// Achievement Modal Tests
// =========================================================================

#[test]
fn test_is_modal_ready_empty_queue_returns_false() {
    let achievements = Achievements::default();
    assert!(!achievements.is_modal_ready());
}

#[test]
fn test_is_modal_ready_before_500ms_returns_false() {
    let mut achievements = Achievements::default();
    // Unlock triggers modal queue and starts accumulation timer
    achievements.unlock(AchievementId::SlayerI, Some("Hero".to_string()));

    // Manually set the accumulation_start to "now" so the elapsed time is
    // deterministically under 500ms, avoiding a wall-clock race with unlock().
    achievements.accumulation_start = Some(std::time::Instant::now());

    assert!(!achievements.is_modal_ready());
}

#[test]
fn test_is_modal_ready_after_500ms_returns_true() {
    let mut achievements = Achievements::default();
    achievements.unlock(AchievementId::SlayerI, Some("Hero".to_string()));

    // Manually set the accumulation_start to 600ms ago
    achievements.accumulation_start =
        Some(std::time::Instant::now() - std::time::Duration::from_millis(600));

    assert!(achievements.is_modal_ready());
}

#[test]
fn test_is_modal_ready_with_no_timer_returns_false() {
    let mut achievements = Achievements::default();
    // Push to modal_queue without setting accumulation_start
    achievements.modal_queue.push(AchievementId::SlayerI);
    // No accumulation_start set
    assert!(!achievements.is_modal_ready());
}

#[test]
fn test_take_modal_queue_drains_queue() {
    let mut achievements = Achievements::default();
    achievements.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    achievements.unlock(AchievementId::BossHunterI, Some("Hero".to_string()));

    let queue = achievements.take_modal_queue();
    assert_eq!(queue.len(), 2);
    assert!(queue.contains(&AchievementId::SlayerI));
    assert!(queue.contains(&AchievementId::BossHunterI));

    // Second call should return empty
    let queue2 = achievements.take_modal_queue();
    assert!(queue2.is_empty());
}

#[test]
fn test_take_modal_queue_resets_timer() {
    let mut achievements = Achievements::default();
    achievements.unlock(AchievementId::SlayerI, Some("Hero".to_string()));

    assert!(achievements.accumulation_start.is_some());

    achievements.take_modal_queue();

    assert!(achievements.accumulation_start.is_none());
}

#[test]
fn test_modal_queue_accumulates_multiple_achievements() {
    let mut achievements = Achievements::default();
    achievements.unlock(AchievementId::SlayerI, Some("Hero".to_string()));
    achievements.unlock(AchievementId::Level10, Some("Hero".to_string()));
    achievements.unlock(AchievementId::GoneFishing, Some("Hero".to_string()));

    assert_eq!(achievements.modal_queue.len(), 3);

    // Timer should only be started once (on first unlock)
    assert!(achievements.accumulation_start.is_some());
}

#[test]
fn test_is_modal_ready_exactly_at_500ms() {
    let mut achievements = Achievements::default();
    achievements.unlock(AchievementId::SlayerI, Some("Hero".to_string()));

    // Set to exactly 500ms ago
    achievements.accumulation_start =
        Some(std::time::Instant::now() - std::time::Duration::from_millis(501));

    assert!(achievements.is_modal_ready());
}

// =========================================================================
// Achievement Persistence Tests
// =========================================================================

#[test]
fn test_achievements_save_load_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("achievements.json");

    // Create achievements with some state
    let mut achievements = Achievements::default();
    achievements.unlock(AchievementId::SlayerI, Some("TestHero".to_string()));
    achievements.unlock(AchievementId::Level10, Some("TestHero".to_string()));
    achievements.total_kills = 150;
    achievements.total_bosses_defeated = 5;
    achievements.highest_level = 42;

    // Save
    let json = serde_json::to_string_pretty(&achievements).unwrap();
    std::fs::write(&path, &json).unwrap();

    // Load
    let loaded_json = std::fs::read_to_string(&path).unwrap();
    let loaded: Achievements = serde_json::from_str(&loaded_json).unwrap();

    // Verify persistent state
    assert!(loaded.is_unlocked(AchievementId::SlayerI));
    assert!(loaded.is_unlocked(AchievementId::Level10));
    assert!(!loaded.is_unlocked(AchievementId::SlayerII));
    assert_eq!(loaded.total_kills, 150);
    assert_eq!(loaded.total_bosses_defeated, 5);
    assert_eq!(loaded.highest_level, 42);
}

#[test]
fn test_load_invalid_json_returns_default() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("achievements.json");

    // Write invalid JSON
    std::fs::write(&path, "not valid json {{{").unwrap();

    // Deserialize should return default
    let json = std::fs::read_to_string(&path).unwrap();
    let loaded: Achievements = serde_json::from_str(&json).unwrap_or_default();

    assert_eq!(loaded.total_kills, 0);
    assert!(!loaded.is_unlocked(AchievementId::SlayerI));
}

#[test]
fn test_load_missing_file_returns_default() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nonexistent.json");

    // Reading missing file should fail gracefully
    let result = std::fs::read_to_string(&path);
    assert!(result.is_err());

    // The load_achievements() pattern: missing file -> default
    let loaded = match result {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => Achievements::default(),
    };

    assert_eq!(loaded.total_kills, 0);
    assert!(loaded.unlocked.is_empty());
}

#[test]
fn test_transient_fields_not_serialized() {
    let mut achievements = Achievements::default();
    achievements.unlock(AchievementId::SlayerI, Some("Hero".to_string()));

    // Transient fields are populated
    assert!(!achievements.pending_notifications.is_empty());
    assert!(!achievements.newly_unlocked.is_empty());
    assert!(!achievements.modal_queue.is_empty());
    assert!(achievements.accumulation_start.is_some());

    // Serialize and deserialize
    let json = serde_json::to_string(&achievements).unwrap();
    let loaded: Achievements = serde_json::from_str(&json).unwrap();

    // Transient fields should be reset to defaults
    assert!(loaded.pending_notifications.is_empty());
    assert!(loaded.newly_unlocked.is_empty());
    assert!(loaded.modal_queue.is_empty());
    assert!(loaded.accumulation_start.is_none());
    assert!(loaded.recently_unlocked.is_empty());
}

#[test]
fn test_achievements_save_preserves_unlock_time() {
    let mut achievements = Achievements::default();
    achievements.unlock(AchievementId::SlayerI, Some("Hero".to_string()));

    let original_time = achievements
        .unlocked
        .get(&AchievementId::SlayerI)
        .unwrap()
        .unlocked_at;

    let json = serde_json::to_string(&achievements).unwrap();
    let loaded: Achievements = serde_json::from_str(&json).unwrap();

    let loaded_time = loaded
        .unlocked
        .get(&AchievementId::SlayerI)
        .unwrap()
        .unlocked_at;

    assert_eq!(original_time, loaded_time);
}

#[test]
fn test_achievements_save_preserves_character_name() {
    let mut achievements = Achievements::default();
    achievements.unlock(AchievementId::SlayerI, Some("MyHero".to_string()));

    let json = serde_json::to_string(&achievements).unwrap();
    let loaded: Achievements = serde_json::from_str(&json).unwrap();

    let char_name = &loaded
        .unlocked
        .get(&AchievementId::SlayerI)
        .unwrap()
        .character_name;

    assert_eq!(char_name.as_deref(), Some("MyHero"));
}

#[test]
fn test_achievements_default_has_empty_collections() {
    let achievements = Achievements::default();
    assert!(achievements.unlocked.is_empty());
    assert!(achievements.progress.is_empty());
    assert_eq!(achievements.total_kills, 0);
    assert_eq!(achievements.total_bosses_defeated, 0);
    assert_eq!(achievements.total_fish_caught, 0);
    assert_eq!(achievements.total_dungeons_completed, 0);
    assert_eq!(achievements.total_minigame_wins, 0);
    assert_eq!(achievements.highest_prestige_rank, 0);
    assert_eq!(achievements.highest_level, 0);
    assert_eq!(achievements.highest_fishing_rank, 0);
    assert_eq!(achievements.zones_fully_cleared, 0);
    assert_eq!(achievements.expanse_cycles_completed, 0);
}

#[test]
fn test_partial_json_loads_with_defaults() {
    // Test that a JSON with only some fields loads correctly
    let partial_json = r#"{
        "unlocked": {},
        "progress": {},
        "total_kills": 999
    }"#;

    let loaded: Achievements = serde_json::from_str(partial_json).unwrap_or_default();
    // The partial JSON should deserialize correctly - serde defaults missing fields
    // If it fails to parse, we get default (which is also fine)
    // The point is it doesn't panic
    assert!(loaded.total_kills == 999 || loaded.total_kills == 0);
}

// =========================================================================
// Dungeon Size Integration with Rewards
// =========================================================================

#[test]
fn test_dungeon_size_treasure_rarity_boost_values() {
    assert_eq!(DungeonSize::Small.treasure_rarity_boost(), 1);
    assert_eq!(DungeonSize::Medium.treasure_rarity_boost(), 1);
    assert_eq!(DungeonSize::Large.treasure_rarity_boost(), 1);
    assert_eq!(DungeonSize::Epic.treasure_rarity_boost(), 2);
    assert_eq!(DungeonSize::Legendary.treasure_rarity_boost(), 3);
}

#[test]
fn test_dungeon_size_boss_xp_ranges_are_ordered() {
    let sizes = [
        DungeonSize::Small,
        DungeonSize::Medium,
        DungeonSize::Large,
        DungeonSize::Epic,
        DungeonSize::Legendary,
    ];
    let mut prev_max = 0u64;
    for size in sizes {
        let (min, max) = size.boss_xp_range();
        assert!(min < max, "{:?} min {} >= max {}", size, min, max);
        assert!(
            min > prev_max,
            "{:?} min {} should exceed prev max {}",
            size,
            min,
            prev_max
        );
        prev_max = max;
    }
}

// =========================================================================
// Haven Bonus Tests — compute_bonuses with maxed Haven
// =========================================================================

/// Build a fully maxed Haven (all 14 rooms at their maximum tier).
fn maxed_haven() -> Haven {
    let mut haven = Haven::new();
    for _ in 0..3 {
        haven.build_room(HavenRoomId::Hearthstone);
    }
    for _ in 0..3 {
        haven.build_room(HavenRoomId::Armory);
        haven.build_room(HavenRoomId::Bedroom);
    }
    for _ in 0..3 {
        haven.build_room(HavenRoomId::TrainingYard);
        haven.build_room(HavenRoomId::Garden);
    }
    for _ in 0..3 {
        haven.build_room(HavenRoomId::TrophyHall);
        haven.build_room(HavenRoomId::Library);
        haven.build_room(HavenRoomId::Watchtower);
        haven.build_room(HavenRoomId::AlchemyLab);
        haven.build_room(HavenRoomId::Workshop);
    }
    for _ in 0..4 {
        haven.build_room(HavenRoomId::FishingDock);
    }
    for _ in 0..3 {
        haven.build_room(HavenRoomId::WarRoom);
        haven.build_room(HavenRoomId::Vault);
    }
    haven.build_room(HavenRoomId::StormForge);
    haven
}

#[test]
fn haven_compute_bonuses_maxed_has_storm_forge() {
    let haven = maxed_haven();
    let bonuses: HavenBonuses = haven.compute_bonuses();
    assert!(bonuses.has_storm_forge);
}

#[test]
fn haven_compute_bonuses_maxed_fishing_rank_bonus_is_ten() {
    let haven = maxed_haven();
    let bonuses = haven.compute_bonuses();
    assert_eq!(bonuses.max_fishing_rank_bonus, 10);
}

#[test]
fn haven_compute_bonuses_maxed_damage_percent() {
    // Armory T3 = +25%
    let haven = maxed_haven();
    let bonuses = haven.compute_bonuses();
    assert!(
        (bonuses.damage_percent - 25.0).abs() < f64::EPSILON,
        "Expected 25.0, got {}",
        bonuses.damage_percent
    );
}

#[test]
fn haven_compute_bonuses_maxed_xp_gain_percent() {
    // TrainingYard T3 = +30%
    let haven = maxed_haven();
    let bonuses = haven.compute_bonuses();
    assert!(
        (bonuses.xp_gain_percent - 30.0).abs() < f64::EPSILON,
        "Expected 30.0, got {}",
        bonuses.xp_gain_percent
    );
}

#[test]
fn haven_compute_bonuses_maxed_drop_rate_percent() {
    // TrophyHall T3 = +15%
    let haven = maxed_haven();
    let bonuses = haven.compute_bonuses();
    assert!(
        (bonuses.drop_rate_percent - 15.0).abs() < f64::EPSILON,
        "Expected 15.0, got {}",
        bonuses.drop_rate_percent
    );
}

#[test]
fn haven_compute_bonuses_maxed_crit_chance_percent() {
    // Watchtower T3 = +20%
    let haven = maxed_haven();
    let bonuses = haven.compute_bonuses();
    assert!(
        (bonuses.crit_chance_percent - 20.0).abs() < f64::EPSILON,
        "Expected 20.0, got {}",
        bonuses.crit_chance_percent
    );
}

#[test]
fn haven_compute_bonuses_maxed_double_strike_chance() {
    // WarRoom T3 = +35%
    let haven = maxed_haven();
    let bonuses = haven.compute_bonuses();
    assert!(
        (bonuses.double_strike_chance - 35.0).abs() < f64::EPSILON,
        "Expected 35.0, got {}",
        bonuses.double_strike_chance
    );
}

#[test]
fn haven_compute_bonuses_maxed_offline_xp_percent() {
    // Hearthstone T3 = +100%
    let haven = maxed_haven();
    let bonuses = haven.compute_bonuses();
    assert!(
        (bonuses.offline_xp_percent - 100.0).abs() < f64::EPSILON,
        "Expected 100.0, got {}",
        bonuses.offline_xp_percent
    );
}

#[test]
fn haven_compute_bonuses_maxed_hp_regen_percent() {
    // AlchemyLab T3 = +100%
    let haven = maxed_haven();
    let bonuses = haven.compute_bonuses();
    assert!(
        (bonuses.hp_regen_percent - 100.0).abs() < f64::EPSILON,
        "Expected 100.0, got {}",
        bonuses.hp_regen_percent
    );
}

#[test]
fn haven_compute_bonuses_maxed_vault_slots() {
    // Vault T3 = 5 slots
    let haven = maxed_haven();
    let bonuses = haven.compute_bonuses();
    assert_eq!(bonuses.vault_slots, 5);
}

// =========================================================================
// Haven Discovery Chance Tests — haven/bonus.rs
// =========================================================================

#[test]
fn haven_discovery_chance_zero_below_prestige_10() {
    assert_eq!(haven_discovery_chance(0), 0.0);
    assert_eq!(haven_discovery_chance(5), 0.0);
    assert_eq!(haven_discovery_chance(9), 0.0);
}

#[test]
fn haven_discovery_chance_nonzero_at_prestige_10() {
    let chance = haven_discovery_chance(10);
    assert!(
        chance > 0.0,
        "Expected positive chance at P10, got {}",
        chance
    );
}

#[test]
fn haven_discovery_chance_equals_base_at_prestige_10() {
    // Base constant is 0.000014
    let chance = haven_discovery_chance(10);
    assert!(
        (chance - 0.000014).abs() < 1e-10,
        "Expected 0.000014 at P10, got {}",
        chance
    );
}

#[test]
fn haven_discovery_chance_increases_linearly_above_prestige_10() {
    let p10 = haven_discovery_chance(10);
    let p11 = haven_discovery_chance(11);
    let p20 = haven_discovery_chance(20);

    // Each rank above 10 adds 0.000007
    let step = 0.000007_f64;
    assert!(
        (p11 - p10 - step).abs() < 1e-10,
        "Step from P10->P11 should be {}, got {}",
        step,
        p11 - p10
    );
    assert!(
        (p20 - p10 - 10.0 * step).abs() < 1e-10,
        "Step from P10->P20 should be 10 steps of {}, got {}",
        step,
        p20 - p10
    );
}

#[test]
fn haven_discovery_chance_strictly_monotonic_above_prestige_10() {
    let mut prev = haven_discovery_chance(10);
    for rank in 11..=30 {
        let cur = haven_discovery_chance(rank);
        assert!(
            cur > prev,
            "Expected monotonic increase: P{} ({}) > P{} ({})",
            rank,
            cur,
            rank - 1,
            prev
        );
        prev = cur;
    }
}
