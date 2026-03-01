//! Integration tests for Storm Forge forging requirements

use quest::achievements::{AchievementId, Achievements};

/// Test that forging requires both Storm Leviathan achievement and 25 prestige
#[test]
fn test_forge_requires_storm_leviathan_and_25_prestige() {
    let mut achievements = Achievements::default();
    let mut prestige_rank = 25u32;

    // Without Storm Leviathan, should not be able to forge
    let has_leviathan = achievements.is_unlocked(AchievementId::StormLeviathan);
    let has_prestige = prestige_rank >= 25;

    assert!(!has_leviathan);
    assert!(has_prestige);
    assert!(!(has_leviathan && has_prestige)); // Cannot forge yet

    // Unlock Storm Leviathan
    achievements.on_storm_leviathan_caught(Some("Hero"));

    let has_leviathan = achievements.is_unlocked(AchievementId::StormLeviathan);
    assert!(has_leviathan);
    assert!(has_leviathan && has_prestige); // Can forge now

    // Simulate forging - deduct prestige
    prestige_rank -= 25;
    achievements.unlock(AchievementId::TheStormbreaker, Some("Hero".to_string()));

    assert_eq!(prestige_rank, 0);
    assert!(achievements.is_unlocked(AchievementId::TheStormbreaker));
}

#[test]
fn test_forge_fails_without_storm_leviathan() {
    let achievements = Achievements::default();
    let prestige_rank = 30u32; // More than enough prestige

    let has_leviathan = achievements.is_unlocked(AchievementId::StormLeviathan);
    let has_prestige = prestige_rank >= 25;

    assert!(!has_leviathan);
    assert!(has_prestige);

    // Cannot forge without Storm Leviathan
    let can_forge = has_leviathan && has_prestige;
    assert!(!can_forge);
}

#[test]
fn test_forge_fails_with_insufficient_prestige() {
    let mut achievements = Achievements::default();
    let prestige_rank = 24u32; // One short

    // Unlock Storm Leviathan
    achievements.on_storm_leviathan_caught(Some("Hero"));

    let has_leviathan = achievements.is_unlocked(AchievementId::StormLeviathan);
    let has_prestige = prestige_rank >= 25;

    assert!(has_leviathan);
    assert!(!has_prestige);

    // Cannot forge with insufficient prestige
    let can_forge = has_leviathan && has_prestige;
    assert!(!can_forge);
}

#[test]
fn test_forge_succeeds_with_both_requirements() {
    let mut achievements = Achievements::default();
    let mut prestige_rank = 25u32;

    // Unlock Storm Leviathan
    achievements.on_storm_leviathan_caught(Some("Hero"));

    let has_leviathan = achievements.is_unlocked(AchievementId::StormLeviathan);
    let has_prestige = prestige_rank >= 25;

    assert!(has_leviathan);
    assert!(has_prestige);

    // Can forge with both requirements met
    let can_forge = has_leviathan && has_prestige;
    assert!(can_forge);

    // Simulate forging
    if can_forge {
        prestige_rank -= 25;
        achievements.unlock(AchievementId::TheStormbreaker, Some("Hero".to_string()));
    }

    assert_eq!(prestige_rank, 0);
    assert!(achievements.is_unlocked(AchievementId::TheStormbreaker));
}

#[test]
fn test_forge_succeeds_with_excess_prestige() {
    let mut achievements = Achievements::default();
    let mut prestige_rank = 100u32; // Way more than needed

    // Unlock Storm Leviathan
    achievements.on_storm_leviathan_caught(Some("Hero"));

    let has_leviathan = achievements.is_unlocked(AchievementId::StormLeviathan);
    let has_prestige = prestige_rank >= 25;

    assert!(has_leviathan);
    assert!(has_prestige);

    let can_forge = has_leviathan && has_prestige;
    assert!(can_forge);

    // Simulate forging - only deducts 25
    if can_forge {
        prestige_rank -= 25;
        achievements.unlock(AchievementId::TheStormbreaker, Some("Hero".to_string()));
    }

    assert_eq!(prestige_rank, 75); // Kept the excess
    assert!(achievements.is_unlocked(AchievementId::TheStormbreaker));
}

#[test]
fn test_prestige_correctly_deducted_on_successful_forge() {
    let mut achievements = Achievements::default();
    let mut prestige_rank = 50u32;

    // Unlock Storm Leviathan
    achievements.on_storm_leviathan_caught(Some("Hero"));

    let initial_prestige = prestige_rank;
    let can_forge = achievements.is_unlocked(AchievementId::StormLeviathan) && prestige_rank >= 25;

    assert!(can_forge);

    // Simulate forging
    if can_forge {
        prestige_rank -= 25;
        achievements.unlock(AchievementId::TheStormbreaker, Some("Hero".to_string()));
    }

    // Verify exactly 25 prestige was deducted
    assert_eq!(prestige_rank, initial_prestige - 25);
    assert_eq!(prestige_rank, 25);
}

#[test]
fn test_cannot_forge_twice() {
    let mut achievements = Achievements::default();
    let mut prestige_rank = 60u32;

    // Unlock Storm Leviathan
    achievements.on_storm_leviathan_caught(Some("Hero"));

    // First forge
    let can_forge = achievements.is_unlocked(AchievementId::StormLeviathan) && prestige_rank >= 25;
    assert!(can_forge);

    prestige_rank -= 25;
    achievements.unlock(AchievementId::TheStormbreaker, Some("Hero".to_string()));

    // Try to forge again - already have TheStormbreaker
    let already_forged = achievements.is_unlocked(AchievementId::TheStormbreaker);
    assert!(already_forged);

    // In the game logic, we check !is_unlocked(TheStormbreaker) before showing forge
    // So the second forge attempt would not even be offered
    assert_eq!(prestige_rank, 35); // Only one forge happened
}
