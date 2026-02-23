//! Tests for history types: SaveEvent descriptions, suffix formatting, and
//! commit message composition.

use quest::history::{format_suffix, SaveEvent};

// ── SaveEvent::description() ─────────────────────────────────────────────

#[test]
fn level_up_description() {
    assert_eq!(SaveEvent::LevelUp(15).description(), "Level up to 15");
}

#[test]
fn level_up_description_high() {
    assert_eq!(SaveEvent::LevelUp(100).description(), "Level up to 100");
}

#[test]
fn prestige_rank_description() {
    assert_eq!(
        SaveEvent::PrestigeRank(5).description(),
        "Prestige to rank 5"
    );
}

#[test]
fn prestige_rank_zero() {
    assert_eq!(
        SaveEvent::PrestigeRank(0).description(),
        "Prestige to rank 0"
    );
}

#[test]
fn zone_boss_defeated_description() {
    assert_eq!(
        SaveEvent::ZoneBossDefeated("Dark Forest".to_string()).description(),
        "Defeated Dark Forest boss"
    );
}

#[test]
fn zone_unlocked_description() {
    assert_eq!(
        SaveEvent::ZoneUnlocked("Mountain Pass".to_string()).description(),
        "Unlocked Mountain Pass"
    );
}

#[test]
fn dungeon_completed_description() {
    assert_eq!(
        SaveEvent::DungeonCompleted("Medium".to_string()).description(),
        "Completed Medium dungeon"
    );
}

#[test]
fn fishing_rank_up_description() {
    assert_eq!(
        SaveEvent::FishingRankUp(12).description(),
        "Fishing rank up to 12"
    );
}

#[test]
fn storm_leviathan_caught_description() {
    assert_eq!(
        SaveEvent::StormLeviathanCaught.description(),
        "Caught the Storm Leviathan"
    );
}

#[test]
fn haven_room_built_description() {
    assert_eq!(
        SaveEvent::HavenRoomBuilt("Armory".to_string()).description(),
        "Built Armory in Haven"
    );
}

#[test]
fn haven_room_upgraded_description() {
    assert_eq!(
        SaveEvent::HavenRoomUpgraded("Armory".to_string(), 2).description(),
        "Upgraded Armory to T2"
    );
}

#[test]
fn soulforge_enhanced_description() {
    assert_eq!(
        SaveEvent::SoulforgeEnhanced("Weapon".to_string(), 7).description(),
        "Enhanced Weapon to +7"
    );
}

#[test]
fn challenge_won_description() {
    assert_eq!(
        SaveEvent::ChallengeWon("Chess".to_string(), "Master".to_string()).description(),
        "Won Chess at Master"
    );
}

#[test]
fn god_item_forged_description() {
    assert_eq!(
        SaveEvent::GodItemForged("Asprika".to_string()).description(),
        "Forged Asprika"
    );
}

#[test]
fn character_created_description() {
    assert_eq!(
        SaveEvent::CharacterCreated("Odin".to_string()).description(),
        "Created character Odin"
    );
}

#[test]
fn character_deleted_description() {
    assert_eq!(
        SaveEvent::CharacterDeleted("Loki".to_string()).description(),
        "Deleted character Loki"
    );
}

#[test]
fn equipment_upgrade_description() {
    assert_eq!(
        SaveEvent::EquipmentUpgrade("Legendary Sword".to_string()).description(),
        "Equipped Legendary Sword"
    );
}

#[test]
fn storm_sigil_activated_description() {
    assert_eq!(
        SaveEvent::StormSigilActivated("Battle Fury".to_string()).description(),
        "Activated Storm Sigil: Battle Fury"
    );
}

#[test]
fn manual_save_description() {
    assert_eq!(SaveEvent::ManualSave.description(), "Manual save");
}

// ── format_suffix() ──────────────────────────────────────────────────────

#[test]
fn format_suffix_basic() {
    // 2h15m = 135 minutes = 8100 seconds
    assert_eq!(
        format_suffix(18, 0, 2, 3, 8100, "Hero"),
        "Lv18 P0 Z2-3 2h15m @Hero"
    );
}

#[test]
fn format_suffix_large_values() {
    // 45h10m = 45*60 + 10 = 2710 minutes = 162600 seconds
    assert_eq!(
        format_suffix(50, 20, 8, 4, 162600, "Hero"),
        "Lv50 P20 Z8-4 45h10m @Hero"
    );
}

#[test]
fn format_suffix_zero_playtime() {
    assert_eq!(
        format_suffix(1, 0, 1, 1, 0, "Hero"),
        "Lv1 P0 Z1-1 0h00m @Hero"
    );
}

#[test]
fn format_suffix_exact_hour() {
    // 1 hour = 3600 seconds
    assert_eq!(
        format_suffix(10, 1, 3, 2, 3600, "Hero"),
        "Lv10 P1 Z3-2 1h00m @Hero"
    );
}

#[test]
fn format_suffix_minutes_only() {
    // 30 minutes = 1800 seconds
    assert_eq!(
        format_suffix(5, 0, 1, 1, 1800, "Hero"),
        "Lv5 P0 Z1-1 0h30m @Hero"
    );
}

#[test]
fn format_suffix_ignores_leftover_seconds() {
    // 3661 seconds = 1h01m (with 1 leftover second ignored)
    assert_eq!(
        format_suffix(10, 0, 1, 1, 3661, "Hero"),
        "Lv10 P0 Z1-1 1h01m @Hero"
    );
}

// ── commit_message() ─────────────────────────────────────────────────────

#[test]
fn commit_message_zone_boss() {
    let event = SaveEvent::ZoneBossDefeated("Dark Forest".to_string());
    let msg = event.commit_message(18, 0, 2, 3, 8100, "Hero");
    assert_eq!(msg, "Defeated Dark Forest boss | Lv18 P0 Z2-3 2h15m @Hero");
}

#[test]
fn commit_message_manual_save() {
    let msg = SaveEvent::ManualSave.commit_message(50, 20, 8, 4, 162600, "Hero");
    assert_eq!(msg, "Manual save | Lv50 P20 Z8-4 45h10m @Hero");
}

#[test]
fn commit_message_level_up() {
    let msg = SaveEvent::LevelUp(25).commit_message(25, 3, 4, 2, 18000, "Hero");
    assert_eq!(msg, "Level up to 25 | Lv25 P3 Z4-2 5h00m @Hero");
}

#[test]
fn commit_message_prestige() {
    let msg = SaveEvent::PrestigeRank(10).commit_message(1, 10, 1, 1, 72000, "Hero");
    assert_eq!(msg, "Prestige to rank 10 | Lv1 P10 Z1-1 20h00m @Hero");
}

#[test]
fn commit_message_challenge_won() {
    let msg = SaveEvent::ChallengeWon("Go".to_string(), "Journeyman".to_string())
        .commit_message(30, 5, 5, 3, 36000, "Hero");
    assert_eq!(msg, "Won Go at Journeyman | Lv30 P5 Z5-3 10h00m @Hero");
}

// ── Clone / PartialEq ───────────────────────────────────────────────────

#[test]
fn save_event_clone_and_eq() {
    let a = SaveEvent::LevelUp(10);
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn save_event_ne() {
    assert_ne!(SaveEvent::LevelUp(1), SaveEvent::LevelUp(2));
    assert_ne!(SaveEvent::ManualSave, SaveEvent::StormLeviathanCaught);
}

// ── CommitInfo / TimelineInfo basic construction ─────────────────────────

#[test]
fn commit_info_construction() {
    use quest::history::CommitInfo;
    let info = CommitInfo {
        id: "abc1234".to_string(),
        message: "Level up to 10 | Lv10 P0 Z1-2 1h30m".to_string(),
        timestamp: 1700000000,
        level: 10,
        prestige: 0,
        zone: 1,
        playtime: 5400,
    };
    assert_eq!(info.id, "abc1234");
    assert_eq!(info.level, 10);
    assert_eq!(info.prestige, 0);
    assert_eq!(info.zone, 1);
    assert_eq!(info.playtime, 5400);
}

#[test]
fn timeline_info_construction() {
    use quest::history::TimelineInfo;
    let info = TimelineInfo {
        name: "main".to_string(),
        is_active: true,
        head_commit: None,
    };
    assert_eq!(info.name, "main");
    assert!(info.is_active);
    assert!(info.head_commit.is_none());
}

#[test]
fn timeline_info_with_head_commit() {
    use quest::history::{CommitInfo, TimelineInfo};
    let commit = CommitInfo {
        id: "def5678".to_string(),
        message: "Manual save | Lv5 P0 Z1-1 0h10m".to_string(),
        timestamp: 1700000100,
        level: 5,
        prestige: 0,
        zone: 1,
        playtime: 600,
    };
    let info = TimelineInfo {
        name: "branch-1".to_string(),
        is_active: false,
        head_commit: Some(commit),
    };
    assert!(!info.is_active);
    assert!(info.head_commit.is_some());
    assert_eq!(info.head_commit.unwrap().level, 5);
}
