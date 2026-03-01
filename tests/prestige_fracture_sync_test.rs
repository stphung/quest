//! Integration tests for prestige reset syncing fracture zone access.
//!
//! Verifies that after prestige, fracture zones (11-30) remain accessible
//! when the account has earned them via StormsEnd achievement, Deep breakthroughs,
//! and sufficient prestige rank.

use quest::achievements::{AchievementId, Achievements};
use quest::character::prestige::perform_prestige;
use quest::core::game_state::GameState;
use quest::zones::sync_account_zone_unlocks;

/// Helper: create a GameState at a given prestige rank with level high enough to prestige again.
fn make_state_ready_to_prestige(current_rank: u32) -> GameState {
    let mut state = GameState::new("Test Hero".to_string(), 0);
    state.prestige_rank = current_rank;
    // Set level high enough to prestige from any rank (220 + rank*10 covers all tiers)
    state.character_level = 220 + (current_rank + 1) * 10;
    state
}

#[test]
fn test_prestige_then_sync_preserves_zone_11() {
    let mut state = make_state_ready_to_prestige(50);
    let mut achievements = Achievements::default();
    achievements.unlock(AchievementId::StormsEnd, None);

    // Before prestige, unlock zone 11
    state.zone_progression.unlock_zone(11);
    assert!(state.zone_progression.is_zone_unlocked(11));

    // Prestige resets zone progression
    perform_prestige(&mut state);

    // Sync should ensure zone 11 is unlocked (P25 required, rank is now 51)
    sync_account_zone_unlocks(
        &mut state.zone_progression,
        achievements.is_unlocked(AchievementId::StormsEnd),
        11,
        state.prestige_rank,
    );

    assert!(state.zone_progression.is_zone_unlocked(11));
}

#[test]
fn test_prestige_then_sync_preserves_fracture_zones_12_through_14() {
    let mut state = make_state_ready_to_prestige(50);
    let mut achievements = Achievements::default();
    achievements.unlock(AchievementId::StormsEnd, None);

    // Unlock fracture zones before prestige
    for z in 11..=14 {
        state.zone_progression.unlock_zone(z);
    }

    perform_prestige(&mut state);

    // Sync with cap 14 should ensure zones 11-14 are unlocked (P50 required, rank is now 51)
    sync_account_zone_unlocks(
        &mut state.zone_progression,
        achievements.is_unlocked(AchievementId::StormsEnd),
        14,
        state.prestige_rank,
    );

    assert!(state.zone_progression.is_zone_unlocked(11));
    assert!(state.zone_progression.is_zone_unlocked(12));
    assert!(state.zone_progression.is_zone_unlocked(13));
    assert!(state.zone_progression.is_zone_unlocked(14));
}

#[test]
fn test_prestige_then_sync_preserves_all_fracture_zones_cap_20() {
    let mut state = make_state_ready_to_prestige(300);
    let mut achievements = Achievements::default();
    achievements.unlock(AchievementId::StormsEnd, None);

    perform_prestige(&mut state);

    sync_account_zone_unlocks(
        &mut state.zone_progression,
        achievements.is_unlocked(AchievementId::StormsEnd),
        20,
        state.prestige_rank,
    );

    for z in 11..=20 {
        assert!(
            state.zone_progression.is_zone_unlocked(z),
            "Zone {} should be unlocked after prestige + sync with cap 20",
            z
        );
    }
}

#[test]
fn test_sync_without_storms_end_does_not_add_zone_11() {
    // Test sync_account_zone_unlocks in isolation (not after prestige)
    // to verify it respects StormsEnd gate
    let mut prog = quest::zones::ZoneProgression::new();
    let achievements = Achievements::default();

    sync_account_zone_unlocks(
        &mut prog,
        achievements.is_unlocked(AchievementId::StormsEnd),
        11,
        100,
    );

    assert!(!prog.is_zone_unlocked(11));
}

#[test]
fn test_prestige_resets_position_but_sync_keeps_zone_access() {
    let mut state = make_state_ready_to_prestige(50);
    let mut achievements = Achievements::default();
    achievements.unlock(AchievementId::StormsEnd, None);

    // Player was in zone 14 before prestige
    state.zone_progression.current_zone_id = 14;
    state.zone_progression.current_subzone_id = 3;
    for z in 11..=14 {
        state.zone_progression.unlock_zone(z);
    }

    perform_prestige(&mut state);

    // Position should be reset to zone 1
    assert_eq!(state.zone_progression.current_zone_id, 1);
    assert_eq!(state.zone_progression.current_subzone_id, 1);

    // Sync restores access
    sync_account_zone_unlocks(
        &mut state.zone_progression,
        achievements.is_unlocked(AchievementId::StormsEnd),
        14,
        state.prestige_rank,
    );

    // Zones 11-14 should be accessible even though position is zone 1
    assert!(state.zone_progression.is_zone_unlocked(14));
}

#[test]
fn test_multiple_prestiges_preserve_fracture_access() {
    let mut state = make_state_ready_to_prestige(100);
    let mut achievements = Achievements::default();
    achievements.unlock(AchievementId::StormsEnd, None);

    // First prestige + sync
    perform_prestige(&mut state);
    sync_account_zone_unlocks(
        &mut state.zone_progression,
        achievements.is_unlocked(AchievementId::StormsEnd),
        17,
        state.prestige_rank,
    );
    assert!(state.zone_progression.is_zone_unlocked(17));

    // Level up again for second prestige
    state.character_level = 300;

    // Second prestige + sync
    perform_prestige(&mut state);
    sync_account_zone_unlocks(
        &mut state.zone_progression,
        achievements.is_unlocked(AchievementId::StormsEnd),
        17,
        state.prestige_rank,
    );
    assert!(state.zone_progression.is_zone_unlocked(17));
    assert!(state.zone_progression.is_zone_unlocked(11));
}

#[test]
fn test_prestige_sync_with_expanded_cap() {
    let mut state = make_state_ready_to_prestige(100);
    let mut achievements = Achievements::default();
    achievements.unlock(AchievementId::StormsEnd, None);

    // First prestige with cap 14
    perform_prestige(&mut state);
    sync_account_zone_unlocks(
        &mut state.zone_progression,
        achievements.is_unlocked(AchievementId::StormsEnd),
        14,
        state.prestige_rank,
    );
    assert!(state.zone_progression.is_zone_unlocked(14));

    // Level up and prestige again, now cap is 17
    state.character_level = 300;
    perform_prestige(&mut state);
    sync_account_zone_unlocks(
        &mut state.zone_progression,
        achievements.is_unlocked(AchievementId::StormsEnd),
        17,
        state.prestige_rank,
    );

    // Should now have zones up to 17
    assert!(state.zone_progression.is_zone_unlocked(17));
}

#[test]
fn test_sync_respects_prestige_gate_for_fracture_zones() {
    let mut prog = quest::zones::ZoneProgression::new();

    // Cap allows zones 12-20, but prestige only P30 — below P50 for zones 12-14
    sync_account_zone_unlocks(&mut prog, true, 20, 30);
    assert!(prog.is_zone_unlocked(11)); // P25 met
    assert!(!prog.is_zone_unlocked(12)); // P50 not met
    assert!(!prog.is_zone_unlocked(15)); // P75 not met

    // Now with P50 — should unlock 12-14 but not 15+
    sync_account_zone_unlocks(&mut prog, true, 20, 50);
    assert!(prog.is_zone_unlocked(12));
    assert!(prog.is_zone_unlocked(14));
    assert!(!prog.is_zone_unlocked(15));
}

// ============================================================
// Edge Case Tests: Prestige exactly at each boundary
// ============================================================

/// P25 is the exact boundary for Zone 11 (meets requirement).
/// Zone 12 requires P50, so it should NOT unlock at P25.
#[test]
fn test_exact_p25_boundary_unlocks_zone11_not_zone12() {
    let mut prog = quest::zones::ZoneProgression::new();
    // Cap includes 12 so we can test it's NOT unlocked due to prestige
    sync_account_zone_unlocks(&mut prog, true, 14, 25);
    assert!(
        prog.is_zone_unlocked(11),
        "Zone 11 requires P25, should unlock at exactly P25"
    );
    assert!(
        !prog.is_zone_unlocked(12),
        "Zone 12 requires P50, should NOT unlock at P25"
    );
}

/// P24 is one below the boundary for Zone 11 — neither Zone 11 nor 12 should unlock.
#[test]
fn test_p24_does_not_unlock_zone11() {
    let mut prog = quest::zones::ZoneProgression::new();
    sync_account_zone_unlocks(&mut prog, true, 14, 24);
    assert!(
        !prog.is_zone_unlocked(11),
        "Zone 11 requires P25, should NOT unlock at P24"
    );
    assert!(
        !prog.is_zone_unlocked(12),
        "Zone 12 requires P50, should NOT unlock at P24"
    );
}

/// P50 is the exact boundary for Zones 12-14. Zone 15 requires P75.
#[test]
fn test_exact_p50_boundary_unlocks_zones_12_14_not_zone15() {
    let mut prog = quest::zones::ZoneProgression::new();
    sync_account_zone_unlocks(&mut prog, true, 17, 50);
    assert!(
        prog.is_zone_unlocked(11),
        "Zone 11 at P25 should unlock at P50"
    );
    assert!(
        prog.is_zone_unlocked(12),
        "Zone 12 requires P50, should unlock at exactly P50"
    );
    assert!(
        prog.is_zone_unlocked(13),
        "Zone 13 requires P50, should unlock at exactly P50"
    );
    assert!(
        prog.is_zone_unlocked(14),
        "Zone 14 requires P50, should unlock at exactly P50"
    );
    assert!(
        !prog.is_zone_unlocked(15),
        "Zone 15 requires P75, should NOT unlock at P50"
    );
}

/// P49 is one below the boundary for Zones 12-14. Zone 11 can still unlock (P25 met).
#[test]
fn test_p49_does_not_unlock_zone12() {
    let mut prog = quest::zones::ZoneProgression::new();
    sync_account_zone_unlocks(&mut prog, true, 14, 49);
    assert!(
        prog.is_zone_unlocked(11),
        "Zone 11 requires P25, should unlock at P49"
    );
    assert!(
        !prog.is_zone_unlocked(12),
        "Zone 12 requires P50, should NOT unlock at P49"
    );
}

/// P75 is the exact boundary for Zones 15-17. Zone 18 requires P100.
#[test]
fn test_exact_p75_boundary_unlocks_zones_15_17_not_zone18() {
    let mut prog = quest::zones::ZoneProgression::new();
    sync_account_zone_unlocks(&mut prog, true, 20, 75);
    assert!(
        prog.is_zone_unlocked(15),
        "Zone 15 requires P75, should unlock at exactly P75"
    );
    assert!(
        prog.is_zone_unlocked(16),
        "Zone 16 requires P75, should unlock at exactly P75"
    );
    assert!(
        prog.is_zone_unlocked(17),
        "Zone 17 requires P75, should unlock at exactly P75"
    );
    assert!(
        !prog.is_zone_unlocked(18),
        "Zone 18 requires P100, should NOT unlock at P75"
    );
}

/// P100 is the exact boundary for Zones 18-20. Zone 21 requires P150.
#[test]
fn test_exact_p100_boundary_unlocks_zones_18_20_not_zone21() {
    let mut prog = quest::zones::ZoneProgression::new();
    sync_account_zone_unlocks(&mut prog, true, 23, 100);
    assert!(
        prog.is_zone_unlocked(18),
        "Zone 18 requires P100, should unlock at exactly P100"
    );
    assert!(
        prog.is_zone_unlocked(19),
        "Zone 19 requires P100, should unlock at exactly P100"
    );
    assert!(
        prog.is_zone_unlocked(20),
        "Zone 20 requires P100, should unlock at exactly P100"
    );
    assert!(
        !prog.is_zone_unlocked(21),
        "Zone 21 requires P150, should NOT unlock at P100"
    );
}

/// P150 is the exact boundary for Zones 21-23. Zone 24 requires P200.
#[test]
fn test_exact_p150_boundary_unlocks_zones_21_23_not_zone24() {
    let mut prog = quest::zones::ZoneProgression::new();
    sync_account_zone_unlocks(&mut prog, true, 26, 150);
    assert!(
        prog.is_zone_unlocked(21),
        "Zone 21 requires P150, should unlock at exactly P150"
    );
    assert!(
        prog.is_zone_unlocked(22),
        "Zone 22 requires P150, should unlock at exactly P150"
    );
    assert!(
        prog.is_zone_unlocked(23),
        "Zone 23 requires P150, should unlock at exactly P150"
    );
    assert!(
        !prog.is_zone_unlocked(24),
        "Zone 24 requires P200, should NOT unlock at P150"
    );
}

/// P200 is the exact boundary for Zones 24-26. Zone 27 requires P300.
#[test]
fn test_exact_p200_boundary_unlocks_zones_24_26_not_zone27() {
    let mut prog = quest::zones::ZoneProgression::new();
    sync_account_zone_unlocks(&mut prog, true, 30, 200);
    assert!(
        prog.is_zone_unlocked(24),
        "Zone 24 requires P200, should unlock at exactly P200"
    );
    assert!(
        prog.is_zone_unlocked(25),
        "Zone 25 requires P200, should unlock at exactly P200"
    );
    assert!(
        prog.is_zone_unlocked(26),
        "Zone 26 requires P200, should unlock at exactly P200"
    );
    assert!(
        !prog.is_zone_unlocked(27),
        "Zone 27 requires P300, should NOT unlock at P200"
    );
}

/// P300 is the exact boundary for Zones 27-30. All fracture zones should unlock.
#[test]
fn test_exact_p300_boundary_unlocks_all_fracture_zones() {
    let mut prog = quest::zones::ZoneProgression::new();
    sync_account_zone_unlocks(&mut prog, true, 30, 300);
    for z in 11..=30 {
        assert!(
            prog.is_zone_unlocked(z),
            "Zone {} should be unlocked at P300 with cap=30",
            z
        );
    }
}

// ============================================================
// Edge Case Tests: Cap at each chapter boundary
// ============================================================

/// cap=14 with P300: only zones 12-14 should unlock (cap hard-blocks 15+).
#[test]
fn test_cap14_with_p300_only_unlocks_zones_12_14() {
    let mut prog = quest::zones::ZoneProgression::new();
    sync_account_zone_unlocks(&mut prog, true, 14, 300);
    assert!(
        prog.is_zone_unlocked(12),
        "Zone 12 should unlock with cap=14 and P300"
    );
    assert!(
        prog.is_zone_unlocked(13),
        "Zone 13 should unlock with cap=14 and P300"
    );
    assert!(
        prog.is_zone_unlocked(14),
        "Zone 14 should unlock with cap=14 and P300"
    );
    assert!(
        !prog.is_zone_unlocked(15),
        "Zone 15 should NOT unlock when cap=14"
    );
    assert!(
        !prog.is_zone_unlocked(16),
        "Zone 16 should NOT unlock when cap=14"
    );
    assert!(
        !prog.is_zone_unlocked(30),
        "Zone 30 should NOT unlock when cap=14"
    );
}

/// cap=17 with P300: zones 12-17 unlock, zones 18+ do not.
#[test]
fn test_cap17_with_p300_only_unlocks_zones_12_17() {
    let mut prog = quest::zones::ZoneProgression::new();
    sync_account_zone_unlocks(&mut prog, true, 17, 300);
    for z in 12..=17 {
        assert!(
            prog.is_zone_unlocked(z),
            "Zone {} should unlock with cap=17 and P300",
            z
        );
    }
    assert!(
        !prog.is_zone_unlocked(18),
        "Zone 18 should NOT unlock when cap=17"
    );
    assert!(
        !prog.is_zone_unlocked(30),
        "Zone 30 should NOT unlock when cap=17"
    );
}

/// cap=20 with P300: zones 12-20 unlock, zones 21+ do not.
#[test]
fn test_cap20_with_p300_only_unlocks_zones_12_20() {
    let mut prog = quest::zones::ZoneProgression::new();
    sync_account_zone_unlocks(&mut prog, true, 20, 300);
    for z in 12..=20 {
        assert!(
            prog.is_zone_unlocked(z),
            "Zone {} should unlock with cap=20 and P300",
            z
        );
    }
    assert!(
        !prog.is_zone_unlocked(21),
        "Zone 21 should NOT unlock when cap=20"
    );
}

/// cap=23 with P300: zones 12-23 unlock, zones 24+ do not.
#[test]
fn test_cap23_with_p300_only_unlocks_zones_12_23() {
    let mut prog = quest::zones::ZoneProgression::new();
    sync_account_zone_unlocks(&mut prog, true, 23, 300);
    for z in 12..=23 {
        assert!(
            prog.is_zone_unlocked(z),
            "Zone {} should unlock with cap=23 and P300",
            z
        );
    }
    assert!(
        !prog.is_zone_unlocked(24),
        "Zone 24 should NOT unlock when cap=23"
    );
}

/// cap=26 with P300: zones 12-26 unlock, zones 27+ do not.
#[test]
fn test_cap26_with_p300_only_unlocks_zones_12_26() {
    let mut prog = quest::zones::ZoneProgression::new();
    sync_account_zone_unlocks(&mut prog, true, 26, 300);
    for z in 12..=26 {
        assert!(
            prog.is_zone_unlocked(z),
            "Zone {} should unlock with cap=26 and P300",
            z
        );
    }
    assert!(
        !prog.is_zone_unlocked(27),
        "Zone 27 should NOT unlock when cap=26"
    );
}

/// cap=30 with P300: all zones 12-30 unlock.
#[test]
fn test_cap30_with_p300_unlocks_all_fracture_zones() {
    let mut prog = quest::zones::ZoneProgression::new();
    sync_account_zone_unlocks(&mut prog, true, 30, 300);
    for z in 12..=30 {
        assert!(
            prog.is_zone_unlocked(z),
            "Zone {} should unlock with cap=30 and P300",
            z
        );
    }
}

// ============================================================
// Edge Case Tests: storms_end interactions
// ============================================================

/// storms_end=false with high prestige and high cap: Zone 11 should NOT unlock,
/// but fracture zones 12+ should unlock normally (they are NOT gated by StormsEnd).
#[test]
fn test_no_storms_end_high_prestige_high_cap_unlocks_fracture_not_zone11() {
    let mut prog = quest::zones::ZoneProgression::new();
    // High prestige and cap but StormsEnd not unlocked
    sync_account_zone_unlocks(&mut prog, false, 30, 300);
    assert!(
        !prog.is_zone_unlocked(11),
        "Zone 11 should NOT unlock without StormsEnd, even with high prestige"
    );
    // Fracture zones 12+ are only gated by prestige + cap, not StormsEnd
    assert!(
        prog.is_zone_unlocked(12),
        "Zone 12 should unlock with high prestige and cap, even without StormsEnd"
    );
    assert!(
        prog.is_zone_unlocked(30),
        "Zone 30 should unlock with P300 cap=30, even without StormsEnd"
    );
}

/// storms_end=true with prestige=0: no zones unlock at all.
/// Zone 11 needs P25, fracture zones need P50+.
#[test]
fn test_storms_end_true_but_prestige_zero_no_zones_unlock() {
    let mut prog = quest::zones::ZoneProgression::new();
    sync_account_zone_unlocks(&mut prog, true, 30, 0);
    assert!(
        !prog.is_zone_unlocked(11),
        "Zone 11 requires P25, should NOT unlock at P0 even with StormsEnd"
    );
    assert!(
        !prog.is_zone_unlocked(12),
        "Zone 12 requires P50, should NOT unlock at P0"
    );
    assert!(
        !prog.is_zone_unlocked(30),
        "Zone 30 requires P300, should NOT unlock at P0"
    );
}

/// storms_end=false with prestige=0: no zones unlock.
#[test]
fn test_no_storms_end_prestige_zero_no_zones_unlock() {
    let mut prog = quest::zones::ZoneProgression::new();
    sync_account_zone_unlocks(&mut prog, false, 30, 0);
    for z in 11..=30 {
        assert!(
            !prog.is_zone_unlocked(z),
            "Zone {} should NOT unlock at P0",
            z
        );
    }
}

// ============================================================
// Edge Case Tests: Multiple sequential syncs
// ============================================================

/// Sequential syncs with increasing prestige should only accumulate unlocks,
/// never remove previously unlocked zones.
#[test]
fn test_sequential_syncs_with_increasing_prestige_only_accumulate() {
    let mut prog = quest::zones::ZoneProgression::new();

    // First sync: P25 — unlocks zone 11 only
    sync_account_zone_unlocks(&mut prog, true, 30, 25);
    assert!(prog.is_zone_unlocked(11));
    assert!(!prog.is_zone_unlocked(12));

    // Second sync: P50 — unlocks zones 12-14, zone 11 stays unlocked
    sync_account_zone_unlocks(&mut prog, true, 30, 50);
    assert!(
        prog.is_zone_unlocked(11),
        "Zone 11 should remain unlocked after second sync"
    );
    assert!(prog.is_zone_unlocked(12));
    assert!(prog.is_zone_unlocked(14));
    assert!(!prog.is_zone_unlocked(15));

    // Third sync: P75 — unlocks zones 15-17, earlier zones stay
    sync_account_zone_unlocks(&mut prog, true, 30, 75);
    assert!(prog.is_zone_unlocked(11));
    assert!(prog.is_zone_unlocked(14));
    assert!(prog.is_zone_unlocked(15));
    assert!(prog.is_zone_unlocked(17));
    assert!(!prog.is_zone_unlocked(18));

    // Fourth sync: P100 — unlocks zones 18-20
    sync_account_zone_unlocks(&mut prog, true, 30, 100);
    assert!(prog.is_zone_unlocked(18));
    assert!(prog.is_zone_unlocked(20));
    assert!(!prog.is_zone_unlocked(21));

    // Fifth sync: P150 — unlocks zones 21-23
    sync_account_zone_unlocks(&mut prog, true, 30, 150);
    assert!(prog.is_zone_unlocked(21));
    assert!(prog.is_zone_unlocked(23));
    assert!(!prog.is_zone_unlocked(24));

    // Sixth sync: P200 — unlocks zones 24-26
    sync_account_zone_unlocks(&mut prog, true, 30, 200);
    assert!(prog.is_zone_unlocked(24));
    assert!(prog.is_zone_unlocked(26));
    assert!(!prog.is_zone_unlocked(27));

    // Seventh sync: P300 — unlocks all remaining zones
    sync_account_zone_unlocks(&mut prog, true, 30, 300);
    for z in 11..=30 {
        assert!(
            prog.is_zone_unlocked(z),
            "Zone {} should be unlocked after full progression",
            z
        );
    }
}

/// Decreasing prestige in sequential syncs should NOT remove previously unlocked zones.
#[test]
fn test_sequential_syncs_decreasing_prestige_never_lock_zones() {
    let mut prog = quest::zones::ZoneProgression::new();

    // Unlock zones 12-14 with P50
    sync_account_zone_unlocks(&mut prog, true, 14, 50);
    assert!(prog.is_zone_unlocked(12));

    // Sync again with lower prestige (P25) — zone 12 must remain unlocked
    sync_account_zone_unlocks(&mut prog, true, 14, 25);
    assert!(
        prog.is_zone_unlocked(12),
        "Zone 12 should remain unlocked even if new sync has lower prestige"
    );
}

// ============================================================
// Edge Case Tests: Cap at/below zone 11
// ============================================================

/// cap=11 means no fracture zones (12+) should unlock even with high prestige.
#[test]
fn test_cap11_no_fracture_zones_unlock() {
    let mut prog = quest::zones::ZoneProgression::new();
    sync_account_zone_unlocks(&mut prog, true, 11, 300);
    // Zone 11 itself unlocks (via StormsEnd path, not fracture loop)
    assert!(
        prog.is_zone_unlocked(11),
        "Zone 11 should still unlock via StormsEnd path"
    );
    // No fracture zones (12+) should unlock
    for z in 12..=30 {
        assert!(
            !prog.is_zone_unlocked(z),
            "Zone {} should NOT unlock when cap=11",
            z
        );
    }
}

/// cap=0 means no fracture zones (12+) should unlock.
#[test]
fn test_cap0_no_fracture_zones_unlock() {
    let mut prog = quest::zones::ZoneProgression::new();
    sync_account_zone_unlocks(&mut prog, true, 0, 300);
    // Zone 11 still unlocks (via StormsEnd, not fracture loop)
    assert!(
        prog.is_zone_unlocked(11),
        "Zone 11 should still unlock via StormsEnd path"
    );
    // No fracture zones should unlock
    for z in 12..=30 {
        assert!(
            !prog.is_zone_unlocked(z),
            "Zone {} should NOT unlock when cap=0",
            z
        );
    }
}

/// cap=1 (below zone 12): no fracture zones should unlock.
#[test]
fn test_cap1_no_fracture_zones_unlock() {
    let mut prog = quest::zones::ZoneProgression::new();
    sync_account_zone_unlocks(&mut prog, false, 1, 300);
    for z in 12..=30 {
        assert!(
            !prog.is_zone_unlocked(z),
            "Zone {} should NOT unlock when cap=1",
            z
        );
    }
}

// ============================================================
// Edge Case Tests: Prestige between tier boundaries
// ============================================================

/// P60 with cap=30: unlocks Z11 (P25), Z12-14 (P50), but NOT Z15-17 (P75).
#[test]
fn test_p60_cap30_unlocks_z11_z12_14_not_z15_plus() {
    let mut prog = quest::zones::ZoneProgression::new();
    sync_account_zone_unlocks(&mut prog, true, 30, 60);
    assert!(
        prog.is_zone_unlocked(11),
        "Zone 11 at P25, should unlock at P60"
    );
    assert!(
        prog.is_zone_unlocked(12),
        "Zone 12 at P50, should unlock at P60"
    );
    assert!(
        prog.is_zone_unlocked(13),
        "Zone 13 at P50, should unlock at P60"
    );
    assert!(
        prog.is_zone_unlocked(14),
        "Zone 14 at P50, should unlock at P60"
    );
    assert!(
        !prog.is_zone_unlocked(15),
        "Zone 15 at P75, should NOT unlock at P60"
    );
    assert!(
        !prog.is_zone_unlocked(16),
        "Zone 16 at P75, should NOT unlock at P60"
    );
    assert!(
        !prog.is_zone_unlocked(17),
        "Zone 17 at P75, should NOT unlock at P60"
    );
    assert!(
        !prog.is_zone_unlocked(18),
        "Zone 18 at P100, should NOT unlock at P60"
    );
}

/// P80 with cap=30: unlocks through Z15-17 (P75) but NOT Z18-20 (P100).
#[test]
fn test_p80_cap30_unlocks_through_z17_not_z18_plus() {
    let mut prog = quest::zones::ZoneProgression::new();
    sync_account_zone_unlocks(&mut prog, true, 30, 80);
    assert!(
        prog.is_zone_unlocked(17),
        "Zone 17 at P75, should unlock at P80"
    );
    assert!(
        !prog.is_zone_unlocked(18),
        "Zone 18 at P100, should NOT unlock at P80"
    );
}

/// P125 with cap=30: unlocks through Z18-20 (P100) but NOT Z21-23 (P150).
#[test]
fn test_p125_cap30_unlocks_through_z20_not_z21_plus() {
    let mut prog = quest::zones::ZoneProgression::new();
    sync_account_zone_unlocks(&mut prog, true, 30, 125);
    assert!(
        prog.is_zone_unlocked(20),
        "Zone 20 at P100, should unlock at P125"
    );
    assert!(
        !prog.is_zone_unlocked(21),
        "Zone 21 at P150, should NOT unlock at P125"
    );
}

/// P175 with cap=30: unlocks through Z21-23 (P150) but NOT Z24-26 (P200).
#[test]
fn test_p175_cap30_unlocks_through_z23_not_z24_plus() {
    let mut prog = quest::zones::ZoneProgression::new();
    sync_account_zone_unlocks(&mut prog, true, 30, 175);
    assert!(
        prog.is_zone_unlocked(23),
        "Zone 23 at P150, should unlock at P175"
    );
    assert!(
        !prog.is_zone_unlocked(24),
        "Zone 24 at P200, should NOT unlock at P175"
    );
}

/// P250 with cap=30: unlocks through Z24-26 (P200) but NOT Z27-30 (P300).
#[test]
fn test_p250_cap30_unlocks_through_z26_not_z27_plus() {
    let mut prog = quest::zones::ZoneProgression::new();
    sync_account_zone_unlocks(&mut prog, true, 30, 250);
    assert!(
        prog.is_zone_unlocked(26),
        "Zone 26 at P200, should unlock at P250"
    );
    assert!(
        !prog.is_zone_unlocked(27),
        "Zone 27 at P300, should NOT unlock at P250"
    );
}

// ============================================================
// Edge Case Tests: Cap and prestige interaction together
// ============================================================

/// cap=14 with P49: prestige gates prevent zones 12-14 from unlocking even though cap=14.
#[test]
fn test_cap14_p49_no_fracture_zones_unlock() {
    let mut prog = quest::zones::ZoneProgression::new();
    sync_account_zone_unlocks(&mut prog, true, 14, 49);
    assert!(
        prog.is_zone_unlocked(11),
        "Zone 11 at P25, should unlock at P49"
    );
    assert!(
        !prog.is_zone_unlocked(12),
        "Zone 12 at P50, should NOT unlock at P49 even with cap=14"
    );
    assert!(
        !prog.is_zone_unlocked(13),
        "Zone 13 at P50, should NOT unlock at P49 even with cap=14"
    );
    assert!(
        !prog.is_zone_unlocked(14),
        "Zone 14 at P50, should NOT unlock at P49 even with cap=14"
    );
}

/// cap=30 with P74: cap is high but prestige gates still apply for Z15+ (P75 needed).
#[test]
fn test_cap30_p74_no_z15_plus_unlock() {
    let mut prog = quest::zones::ZoneProgression::new();
    sync_account_zone_unlocks(&mut prog, true, 30, 74);
    assert!(
        prog.is_zone_unlocked(14),
        "Zone 14 at P50, should unlock at P74"
    );
    assert!(
        !prog.is_zone_unlocked(15),
        "Zone 15 at P75, should NOT unlock at P74 even with cap=30"
    );
    assert!(
        !prog.is_zone_unlocked(30),
        "Zone 30 at P300, should NOT unlock at P74"
    );
}
