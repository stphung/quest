//! God Items integration tests.
//!
//! End-to-end tests for the god item system: definitions, progress tracking,
//! auto-equip protection, serialization, and equipped bonus queries.

use quest::god_items::{
    asprika_definition, equipped_god_item_dr, megingjord_definition, sleipnir_definition,
    GodItemId, GodItemProgress, GodItemState,
};
use quest::items::{
    auto_equip_if_better, Affix, AffixType, AttributeBonuses, Equipment, EquipmentSlot, Item,
    Rarity,
};
use quest::GameState;

/// Helper: create a GodItemProgress with Asprika in Discovered state.
fn asprika_discovered_progress() -> GodItemProgress {
    GodItemProgress {
        asprika_state: GodItemState::Discovered,
        ..Default::default()
    }
}

/// Helper: create a GodItemProgress with Sleipnir in Discovered state.
fn sleipnir_discovered_progress() -> GodItemProgress {
    GodItemProgress {
        sleipnir_state: GodItemState::Discovered,
        ..Default::default()
    }
}

// =========================================================================
// Asprika definition and item creation
// =========================================================================

#[test]
fn test_asprika_item_is_mythic_with_correct_fields() {
    let asprika = asprika_definition().to_item();
    assert_eq!(asprika.rarity, Rarity::Mythic);
    assert_eq!(asprika.slot, EquipmentSlot::Armor);
    assert_eq!(asprika.display_name, "Asprika");
    assert_eq!(asprika.god_item_id, Some(GodItemId::Asprika));
    assert_eq!(asprika.ilvl, 100);
    assert!(asprika.attributes.con > 0, "Asprika should have CON");
    assert!(asprika.attributes.wis > 0, "Asprika should have WIS");
}

// =========================================================================
// Auto-equip protection: Mythic items cannot be replaced by non-Mythic
// =========================================================================

#[test]
fn test_asprika_item_is_always_best_in_slot() {
    // Asprika should never be replaced by auto-equip, even by a strong Legendary.
    let asprika = asprika_definition().to_item();
    let legendary = Item {
        slot: EquipmentSlot::Armor,
        rarity: Rarity::Legendary,
        ilvl: 100,
        base_name: "Test".to_string(),
        display_name: "Test".to_string(),
        attributes: AttributeBonuses {
            con: 20,
            dex: 10,
            ..AttributeBonuses::new()
        },
        affixes: vec![
            Affix {
                affix_type: AffixType::DamageReduction,
                value: 15.0,
            },
            Affix {
                affix_type: AffixType::HPBonus,
                value: 100.0,
            },
        ],
        god_item_id: None,
    };

    let mut state = GameState::new("Test".to_string(), 0);
    state.equipment.set(EquipmentSlot::Armor, Some(asprika));

    let equipped = auto_equip_if_better(legendary, &mut state);
    assert!(
        !equipped,
        "Legendary should not replace Mythic Asprika via auto-equip"
    );
    assert_eq!(
        state
            .equipment
            .get(EquipmentSlot::Armor)
            .as_ref()
            .unwrap()
            .rarity,
        Rarity::Mythic,
        "Asprika should still be in the Armor slot"
    );
}

// =========================================================================
// Asprika milestone (expanse_cycle_complete)
// =========================================================================

#[test]
fn test_asprika_milestones_met_with_expanse_cycle() {
    let mut progress = asprika_discovered_progress();

    assert!(!progress.asprika_milestones.all_met());
    progress.sync_asprika_milestone(true);
    assert!(progress.asprika_milestones.all_met());
}

#[test]
fn test_asprika_milestones_not_met_without_expanse_cycle() {
    let progress = asprika_discovered_progress();
    assert!(
        !progress.asprika_milestones.all_met(),
        "Should not be all_met without expanse cycle completion"
    );
}

// =========================================================================
// Sleipnir milestones (master challenges + enhancement)
// =========================================================================

#[test]
fn test_sleipnir_milestones_state_machine() {
    let mut progress = sleipnir_discovered_progress();

    progress.record_master_challenge_win("Chess");
    progress.record_master_challenge_win("Go");
    progress.record_master_challenge_win("Snake");
    progress.sync_enhancement_milestones(&[7, 7, 7, 0, 0, 0, 0]);

    assert!(progress.sleipnir_milestones.all_met());
}

#[test]
fn test_sleipnir_milestones_incomplete_without_enough_challenges() {
    let mut progress = sleipnir_discovered_progress();

    progress.record_master_challenge_win("Chess");
    progress.record_master_challenge_win("Go");
    // Only 2 of 3 required challenge types
    progress.sync_enhancement_milestones(&[7, 7, 7, 0, 0, 0, 0]);

    assert!(
        !progress.sleipnir_milestones.all_met(),
        "Should not be all_met with only 2 challenge types"
    );
}

#[test]
fn test_sleipnir_milestones_incomplete_without_enhancement() {
    let mut progress = sleipnir_discovered_progress();

    progress.record_master_challenge_win("Chess");
    progress.record_master_challenge_win("Go");
    progress.record_master_challenge_win("Snake");
    progress.sync_enhancement_milestones(&[6, 6, 6, 0, 0, 0, 0]); // below +7

    assert!(
        !progress.sleipnir_milestones.all_met(),
        "Should not be all_met without highest enhancement >= 7"
    );
}

#[test]
fn test_record_master_challenge_win_ignored_when_undiscovered() {
    let mut progress = GodItemProgress::default();
    assert_eq!(progress.sleipnir_state, GodItemState::Undiscovered);

    let recorded = progress.record_master_challenge_win("Chess");
    assert!(
        !recorded,
        "Should not record wins when Sleipnir undiscovered"
    );
    assert!(
        progress
            .sleipnir_milestones
            .master_challenge_types_won
            .is_empty(),
        "No wins should be tracked when undiscovered"
    );
}

#[test]
fn test_record_master_challenge_win_deduplicates() {
    let mut progress = sleipnir_discovered_progress();

    assert!(progress.record_master_challenge_win("Chess"));
    assert!(
        !progress.record_master_challenge_win("Chess"),
        "Duplicate win type should return false"
    );
    assert_eq!(
        progress
            .sleipnir_milestones
            .master_challenge_types_won
            .len(),
        1,
        "Should only have one entry for Chess"
    );
}

// =========================================================================
// Megingjord milestones (enhancement level >= 9)
// =========================================================================

#[test]
fn test_megingjord_milestones_met() {
    let mut progress = GodItemProgress {
        megingjord_state: GodItemState::Discovered,
        ..Default::default()
    };

    progress.sync_enhancement_milestones(&[9, 0, 0, 0, 0, 0, 0]);
    assert!(progress.megingjord_milestones.all_met());
}

#[test]
fn test_megingjord_milestones_not_met() {
    let mut progress = GodItemProgress {
        megingjord_state: GodItemState::Discovered,
        ..Default::default()
    };

    progress.sync_enhancement_milestones(&[8, 0, 0, 0, 0, 0, 0]);
    assert!(!progress.megingjord_milestones.all_met());
}

// =========================================================================
// Serialization roundtrips
// =========================================================================

#[test]
fn test_mythic_item_serialization_roundtrip() {
    let asprika = asprika_definition().to_item();
    let json = serde_json::to_string(&asprika).unwrap();
    let loaded: Item = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.rarity, Rarity::Mythic);
    assert_eq!(loaded.god_item_id, Some(GodItemId::Asprika));
    assert_eq!(loaded.display_name, "Asprika");
}

#[test]
fn test_god_item_progress_serialization_roundtrip() {
    let mut progress = sleipnir_discovered_progress();
    progress.asprika_state = GodItemState::Discovered;

    progress.record_master_challenge_win("Chess");
    progress.sync_enhancement_milestones(&[0, 7, 0, 8, 0, 0, 7]);
    progress.sync_asprika_milestone(true);

    let json = serde_json::to_string_pretty(&progress).unwrap();
    let loaded: GodItemProgress = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.asprika_state, GodItemState::Discovered);
    assert!(loaded.asprika_milestones.expanse_cycle_complete);
    assert_eq!(loaded.sleipnir_state, GodItemState::Discovered);
    assert_eq!(
        loaded.sleipnir_milestones.master_challenge_types_won.len(),
        1
    );
    assert_eq!(loaded.sleipnir_milestones.highest_enhancement_level, 8);
    assert_eq!(loaded.megingjord_milestones.highest_enhancement_level, 8);
}

#[test]
fn test_god_item_progress_default_serialization() {
    let progress = GodItemProgress::default();
    let json = serde_json::to_string(&progress).unwrap();
    let loaded: GodItemProgress = serde_json::from_str(&json).unwrap();
    assert_eq!(loaded.asprika_state, GodItemState::Undiscovered);
    assert!(!loaded.asprika_milestones.expanse_cycle_complete);
    assert_eq!(loaded.sleipnir_state, GodItemState::Undiscovered);
    assert!(loaded
        .sleipnir_milestones
        .master_challenge_types_won
        .is_empty());
    assert_eq!(loaded.megingjord_state, GodItemState::Undiscovered);
}

// =========================================================================
// Equipped god item bonus queries
// =========================================================================

#[test]
fn test_equipped_god_item_dr_integration() {
    let mut equipment = Equipment::new();

    // No god item — 0% DR
    assert_eq!(equipped_god_item_dr(&equipment), 0.0);

    // Equip Asprika — 30% DR
    let asprika = asprika_definition().to_item();
    equipment.set(EquipmentSlot::Armor, Some(asprika));
    assert!((equipped_god_item_dr(&equipment) - 30.0).abs() < f64::EPSILON);
}

#[test]
fn test_equipped_god_item_dr_zero_with_non_god_items() {
    let mut equipment = Equipment::new();
    let normal_item = Item {
        slot: EquipmentSlot::Armor,
        rarity: Rarity::Legendary,
        ilvl: 100,
        base_name: "Plate".to_string(),
        display_name: "Plate".to_string(),
        attributes: AttributeBonuses {
            con: 15,
            ..AttributeBonuses::new()
        },
        affixes: vec![Affix {
            affix_type: AffixType::DamageReduction,
            value: 10.0,
        }],
        god_item_id: None,
    };
    equipment.set(EquipmentSlot::Armor, Some(normal_item));

    assert_eq!(
        equipped_god_item_dr(&equipment),
        0.0,
        "Non-god items should contribute 0% god item DR"
    );
}

// =========================================================================
// God item state transitions
// =========================================================================

#[test]
fn test_god_item_state_transitions() {
    let mut progress = GodItemProgress::default();

    // Undiscovered -> Discovered
    assert_eq!(progress.asprika_state, GodItemState::Undiscovered);
    progress.asprika_state = GodItemState::Discovered;
    assert_eq!(progress.asprika_state, GodItemState::Discovered);

    // Discovered -> ReadyToForge (after milestones)
    progress.asprika_state = GodItemState::ReadyToForge;
    assert_eq!(progress.asprika_state, GodItemState::ReadyToForge);

    // ReadyToForge -> Forged
    progress.asprika_state = GodItemState::Forged;
    assert_eq!(progress.asprika_state, GodItemState::Forged);
}

// =========================================================================
// Sync enhancement milestones edge cases
// =========================================================================

#[test]
fn test_sync_enhancement_milestones_updates_sleipnir_and_megingjord() {
    let mut progress = GodItemProgress::default();

    // All zeros
    progress.sync_enhancement_milestones(&[0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(progress.sleipnir_milestones.highest_enhancement_level, 0);
    assert_eq!(progress.megingjord_milestones.highest_enhancement_level, 0);

    // Highest is 7
    progress.sync_enhancement_milestones(&[7, 3, 5, 0, 0, 0, 0]);
    assert_eq!(progress.sleipnir_milestones.highest_enhancement_level, 7);
    assert_eq!(progress.megingjord_milestones.highest_enhancement_level, 7);

    // Highest is 10
    progress.sync_enhancement_milestones(&[10, 8, 9, 7, 0, 0, 0]);
    assert_eq!(progress.sleipnir_milestones.highest_enhancement_level, 10);
    assert_eq!(progress.megingjord_milestones.highest_enhancement_level, 10);

    // Below threshold
    progress.sync_enhancement_milestones(&[6, 6, 6, 6, 6, 6, 6]);
    assert_eq!(progress.sleipnir_milestones.highest_enhancement_level, 6);
    assert_eq!(progress.megingjord_milestones.highest_enhancement_level, 6);
}

// =========================================================================
// Sleipnir and Megingjord definitions
// =========================================================================

#[test]
fn test_sleipnir_item_is_mythic_with_correct_fields() {
    let sleipnir = sleipnir_definition().to_item();
    assert_eq!(sleipnir.rarity, Rarity::Mythic);
    assert_eq!(sleipnir.slot, EquipmentSlot::Boots);
    assert_eq!(sleipnir.display_name, "Sleipnir");
    assert_eq!(sleipnir.god_item_id, Some(GodItemId::Sleipnir));
    assert_eq!(sleipnir.ilvl, 100);
    assert!(sleipnir.attributes.dex > 0, "Sleipnir should have DEX");
}

#[test]
fn test_megingjord_item_is_mythic_with_correct_fields() {
    let megingjord = megingjord_definition().to_item();
    assert_eq!(megingjord.rarity, Rarity::Mythic);
    assert_eq!(megingjord.slot, EquipmentSlot::Ring);
    assert_eq!(megingjord.display_name, "Megingjord");
    assert_eq!(megingjord.god_item_id, Some(GodItemId::Megingjord));
    assert_eq!(megingjord.ilvl, 100);
    assert!(megingjord.attributes.str > 0, "Megingjord should have STR");
}
