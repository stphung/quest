//! Zone data integrity tests
//!
//! Verifies structural correctness of all 30 zone definitions, prestige requirements,
//! subzone counts, boss flags, enemy stat scaling, and FractureRegion metadata.

use quest::core::constants::{FRACTURE_ZONE_STAT_MULTIPLIER, ZONE_ENEMY_STATS};
use quest::zones::{get_all_zones, FractureRegion};
use std::collections::HashSet;

// ============================================================================
// Test 1: All 30 zones exist and have sequential IDs 1-30
// ============================================================================

#[test]
fn test_exactly_30_zones_exist() {
    let zones = get_all_zones();
    assert_eq!(zones.len(), 30, "There must be exactly 30 zones");
}

#[test]
fn test_zone_ids_are_sequential_1_to_30() {
    let zones = get_all_zones();
    for (i, zone) in zones.iter().enumerate() {
        let expected_id = (i + 1) as u32;
        assert_eq!(
            zone.id, expected_id,
            "Zone at index {} has id {}, expected {}",
            i, zone.id, expected_id
        );
    }
}

#[test]
fn test_zone_ids_are_unique() {
    let zones = get_all_zones();
    let mut seen = HashSet::new();
    for zone in zones.iter() {
        assert!(seen.insert(zone.id), "Duplicate zone id: {}", zone.id);
    }
}

// ============================================================================
// Test 2: Prestige requirements per tier
// ============================================================================

#[test]
fn test_prestige_requirement_tier1_p0_zones_1_2() {
    let zones = get_all_zones();
    for zone_id in [1u32, 2] {
        let zone = zones.iter().find(|z| z.id == zone_id).unwrap();
        assert_eq!(
            zone.prestige_requirement, 0,
            "Zone {} (Tier 1) should require P0, got P{}",
            zone_id, zone.prestige_requirement
        );
    }
}

#[test]
fn test_prestige_requirement_tier2_p5_zones_3_4() {
    let zones = get_all_zones();
    for zone_id in [3u32, 4] {
        let zone = zones.iter().find(|z| z.id == zone_id).unwrap();
        assert_eq!(
            zone.prestige_requirement, 5,
            "Zone {} (Tier 2) should require P5, got P{}",
            zone_id, zone.prestige_requirement
        );
    }
}

#[test]
fn test_prestige_requirement_tier3_p10_zones_5_6() {
    let zones = get_all_zones();
    for zone_id in [5u32, 6] {
        let zone = zones.iter().find(|z| z.id == zone_id).unwrap();
        assert_eq!(
            zone.prestige_requirement, 10,
            "Zone {} (Tier 3) should require P10, got P{}",
            zone_id, zone.prestige_requirement
        );
    }
}

#[test]
fn test_prestige_requirement_tier4_p15_zones_7_8() {
    let zones = get_all_zones();
    for zone_id in [7u32, 8] {
        let zone = zones.iter().find(|z| z.id == zone_id).unwrap();
        assert_eq!(
            zone.prestige_requirement, 15,
            "Zone {} (Tier 4) should require P15, got P{}",
            zone_id, zone.prestige_requirement
        );
    }
}

#[test]
fn test_prestige_requirement_tier5_p20_zones_9_10() {
    let zones = get_all_zones();
    for zone_id in [9u32, 10] {
        let zone = zones.iter().find(|z| z.id == zone_id).unwrap();
        assert_eq!(
            zone.prestige_requirement, 20,
            "Zone {} (Tier 5) should require P20, got P{}",
            zone_id, zone.prestige_requirement
        );
    }
}

#[test]
fn test_prestige_requirement_expanse_p25_zone_11() {
    let zones = get_all_zones();
    let zone = zones.iter().find(|z| z.id == 11).unwrap();
    assert_eq!(
        zone.prestige_requirement, 25,
        "Zone 11 (The Expanse) should require P25, got P{}",
        zone.prestige_requirement
    );
}

#[test]
fn test_prestige_requirement_fracture_ch1_p50_zones_12_14() {
    let zones = get_all_zones();
    for zone_id in [12u32, 13, 14] {
        let zone = zones.iter().find(|z| z.id == zone_id).unwrap();
        assert_eq!(
            zone.prestige_requirement, 50,
            "Zone {} (Ch.1 Red Fault) should require P50, got P{}",
            zone_id, zone.prestige_requirement
        );
    }
}

#[test]
fn test_prestige_requirement_fracture_ch2_p75_zones_15_17() {
    let zones = get_all_zones();
    for zone_id in [15u32, 16, 17] {
        let zone = zones.iter().find(|z| z.id == zone_id).unwrap();
        assert_eq!(
            zone.prestige_requirement, 75,
            "Zone {} (Ch.2 Mirror Scar) should require P75, got P{}",
            zone_id, zone.prestige_requirement
        );
    }
}

#[test]
fn test_prestige_requirement_fracture_ch3_p100_zones_18_20() {
    let zones = get_all_zones();
    for zone_id in [18u32, 19, 20] {
        let zone = zones.iter().find(|z| z.id == zone_id).unwrap();
        assert_eq!(
            zone.prestige_requirement, 100,
            "Zone {} (Ch.3 Black Mouth) should require P100, got P{}",
            zone_id, zone.prestige_requirement
        );
    }
}

#[test]
fn test_prestige_requirement_fracture_ch4_p150_zones_21_23() {
    let zones = get_all_zones();
    for zone_id in [21u32, 22, 23] {
        let zone = zones.iter().find(|z| z.id == zone_id).unwrap();
        assert_eq!(
            zone.prestige_requirement, 150,
            "Zone {} (Ch.4 Hollow Throne) should require P150, got P{}",
            zone_id, zone.prestige_requirement
        );
    }
}

#[test]
fn test_prestige_requirement_fracture_ch5_p200_zones_24_26() {
    let zones = get_all_zones();
    for zone_id in [24u32, 25, 26] {
        let zone = zones.iter().find(|z| z.id == zone_id).unwrap();
        assert_eq!(
            zone.prestige_requirement, 200,
            "Zone {} (Ch.5 Wailing Reach) should require P200, got P{}",
            zone_id, zone.prestige_requirement
        );
    }
}

#[test]
fn test_prestige_requirement_fracture_ch6_p300_zones_27_30() {
    let zones = get_all_zones();
    for zone_id in [27u32, 28, 29, 30] {
        let zone = zones.iter().find(|z| z.id == zone_id).unwrap();
        assert_eq!(
            zone.prestige_requirement, 300,
            "Zone {} (Ch.6 Origin Wound) should require P300, got P{}",
            zone_id, zone.prestige_requirement
        );
    }
}

// ============================================================================
// Test 3: Subzone counts per tier
// ============================================================================

#[test]
fn test_subzone_count_3_for_zones_1_to_4() {
    let zones = get_all_zones();
    for zone_id in 1u32..=4 {
        let zone = zones.iter().find(|z| z.id == zone_id).unwrap();
        assert_eq!(
            zone.subzones.len(),
            3,
            "Zone {} should have 3 subzones, has {}",
            zone_id,
            zone.subzones.len()
        );
    }
}

#[test]
fn test_subzone_count_4_for_zones_5_to_11() {
    let zones = get_all_zones();
    for zone_id in 5u32..=11 {
        let zone = zones.iter().find(|z| z.id == zone_id).unwrap();
        assert_eq!(
            zone.subzones.len(),
            4,
            "Zone {} should have 4 subzones, has {}",
            zone_id,
            zone.subzones.len()
        );
    }
}

#[test]
fn test_subzone_count_5_for_fracture_zones_12_to_30() {
    let zones = get_all_zones();
    for zone_id in 12u32..=30 {
        let zone = zones.iter().find(|z| z.id == zone_id).unwrap();
        assert_eq!(
            zone.subzones.len(),
            5,
            "Fracture zone {} should have 5 subzones, has {}",
            zone_id,
            zone.subzones.len()
        );
    }
}

// ============================================================================
// Test 4: Every subzone has a non-empty named boss
// ============================================================================

#[test]
fn test_every_subzone_has_named_boss() {
    let zones = get_all_zones();
    for zone in zones.iter() {
        for subzone in zone.subzones.iter() {
            assert!(
                !subzone.boss.name.is_empty(),
                "Zone {} subzone {} has an empty boss name",
                zone.id,
                subzone.id
            );
        }
    }
}

#[test]
fn test_every_subzone_boss_name_not_whitespace_only() {
    let zones = get_all_zones();
    for zone in zones.iter() {
        for subzone in zone.subzones.iter() {
            assert!(
                !subzone.boss.name.trim().is_empty(),
                "Zone {} subzone {} has a whitespace-only boss name",
                zone.id,
                subzone.id
            );
        }
    }
}

// ============================================================================
// Test 5: Every zone's final subzone boss has is_zone_boss=true
// ============================================================================

#[test]
fn test_final_subzone_boss_is_zone_boss_true() {
    let zones = get_all_zones();
    for zone in zones.iter() {
        let last = zone.subzones.last().expect("Zone has no subzones");
        assert!(
            last.boss.is_zone_boss,
            "Zone {} final subzone {} boss '{}' should have is_zone_boss=true",
            zone.id, last.id, last.boss.name
        );
    }
}

#[test]
fn test_non_final_subzone_boss_is_zone_boss_false() {
    let zones = get_all_zones();
    for zone in zones.iter() {
        let len = zone.subzones.len();
        for (i, subzone) in zone.subzones.iter().enumerate() {
            let is_last = i == len - 1;
            if !is_last {
                assert!(
                    !subzone.boss.is_zone_boss,
                    "Zone {} subzone {} is not the last but has is_zone_boss=true",
                    zone.id, subzone.id
                );
            }
        }
    }
}

// ============================================================================
// Test 6: Zone enemy stats scaling for fracture zones (Z12-30 at 1.6x from Z11)
// ============================================================================

#[test]
fn test_zone_enemy_stats_table_has_30_entries() {
    assert_eq!(
        ZONE_ENEMY_STATS.len(),
        30,
        "ZONE_ENEMY_STATS must have 30 entries"
    );
}

#[test]
fn test_fracture_zone_stat_multiplier_constant_is_1_6() {
    assert!(
        (FRACTURE_ZONE_STAT_MULTIPLIER - 1.6).abs() < 1e-10,
        "FRACTURE_ZONE_STAT_MULTIPLIER must be 1.6, got {}",
        FRACTURE_ZONE_STAT_MULTIPLIER
    );
}

#[test]
fn test_fracture_zone_base_hp_scales_1_6x_from_z11() {
    // Zone 11 (index 10): base_hp = 5000
    // Zone 12 (index 11): base_hp should be ~8000 (5000 * 1.6)
    let z11 = ZONE_ENEMY_STATS[10];
    let z12 = ZONE_ENEMY_STATS[11];
    let ratio = z12.0 as f64 / z11.0 as f64;
    assert!(
        (ratio - 1.6).abs() < 0.01,
        "Z12 base_hp ratio to Z11 should be ~1.6, got {:.4}",
        ratio
    );
}

#[test]
fn test_fracture_zone_base_dmg_scales_1_6x_from_z11() {
    let z11 = ZONE_ENEMY_STATS[10];
    let z12 = ZONE_ENEMY_STATS[11];
    let ratio = z12.2 as f64 / z11.2 as f64;
    assert!(
        (ratio - 1.6).abs() < 0.01,
        "Z12 base_dmg ratio to Z11 should be ~1.6, got {:.4}",
        ratio
    );
}

#[test]
fn test_fracture_zone_base_def_scales_1_6x_from_z11() {
    let z11 = ZONE_ENEMY_STATS[10];
    let z12 = ZONE_ENEMY_STATS[11];
    let ratio = z12.4 as f64 / z11.4 as f64;
    assert!(
        (ratio - 1.6).abs() < 0.01,
        "Z12 base_def ratio to Z11 should be ~1.6, got {:.4}",
        ratio
    );
}

#[test]
fn test_fracture_zone_base_hp_scales_1_6x_each_step_z12_to_z30() {
    // Each fracture zone's base_hp should be ~1.6x the previous zone's base_hp
    for zone_id in 13u32..=30 {
        let prev = ZONE_ENEMY_STATS[(zone_id - 2) as usize]; // previous zone index
        let curr = ZONE_ENEMY_STATS[(zone_id - 1) as usize]; // current zone index
        let ratio = curr.0 as f64 / prev.0 as f64;
        assert!(
            (ratio - 1.6).abs() < 0.02,
            "Zone {} base_hp ratio to Zone {} should be ~1.6, got {:.4}",
            zone_id,
            zone_id - 1,
            ratio
        );
    }
}

#[test]
fn test_fracture_zone_base_dmg_scales_1_6x_each_step_z12_to_z30() {
    for zone_id in 13u32..=30 {
        let prev = ZONE_ENEMY_STATS[(zone_id - 2) as usize];
        let curr = ZONE_ENEMY_STATS[(zone_id - 1) as usize];
        let ratio = curr.2 as f64 / prev.2 as f64;
        assert!(
            (ratio - 1.6).abs() < 0.02,
            "Zone {} base_dmg ratio to Zone {} should be ~1.6, got {:.4}",
            zone_id,
            zone_id - 1,
            ratio
        );
    }
}

#[test]
fn test_fracture_zone_base_def_scales_1_6x_each_step_z12_to_z30() {
    for zone_id in 13u32..=30 {
        let prev = ZONE_ENEMY_STATS[(zone_id - 2) as usize];
        let curr = ZONE_ENEMY_STATS[(zone_id - 1) as usize];
        let ratio = curr.4 as f64 / prev.4 as f64;
        assert!(
            (ratio - 1.6).abs() < 0.02,
            "Zone {} base_def ratio to Zone {} should be ~1.6, got {:.4}",
            zone_id,
            zone_id - 1,
            ratio
        );
    }
}

#[test]
fn test_fracture_zone_stats_are_monotonically_increasing_hp() {
    // Each zone's base_hp should be strictly greater than the previous zone
    for zone_id in 12u32..=30 {
        let prev = ZONE_ENEMY_STATS[(zone_id - 2) as usize];
        let curr = ZONE_ENEMY_STATS[(zone_id - 1) as usize];
        assert!(
            curr.0 > prev.0,
            "Zone {} base_hp ({}) should exceed Zone {} base_hp ({})",
            zone_id,
            curr.0,
            zone_id - 1,
            prev.0
        );
    }
}

#[test]
fn test_non_fracture_zone_stats_are_not_zero() {
    // Zones 1-11 should all have non-zero base stats
    for zone_id in 1u32..=11 {
        let stats = ZONE_ENEMY_STATS[(zone_id - 1) as usize];
        assert!(stats.0 > 0, "Zone {} base_hp should not be zero", zone_id);
        assert!(stats.2 > 0, "Zone {} base_dmg should not be zero", zone_id);
    }
}

// ============================================================================
// Test 7: No duplicate zone names, no empty subzone names
// ============================================================================

#[test]
fn test_no_duplicate_zone_names() {
    let zones = get_all_zones();
    let mut names = HashSet::new();
    for zone in zones.iter() {
        assert!(
            names.insert(zone.name),
            "Duplicate zone name: '{}' (zone {})",
            zone.name,
            zone.id
        );
    }
}

#[test]
fn test_no_empty_zone_names() {
    let zones = get_all_zones();
    for zone in zones.iter() {
        assert!(!zone.name.is_empty(), "Zone {} has an empty name", zone.id);
        assert!(
            !zone.name.trim().is_empty(),
            "Zone {} has a whitespace-only name",
            zone.id
        );
    }
}

#[test]
fn test_no_empty_subzone_names() {
    let zones = get_all_zones();
    for zone in zones.iter() {
        for subzone in zone.subzones.iter() {
            assert!(
                !subzone.name.is_empty(),
                "Zone {} subzone {} has an empty name",
                zone.id,
                subzone.id
            );
            assert!(
                !subzone.name.trim().is_empty(),
                "Zone {} subzone {} has a whitespace-only name",
                zone.id,
                subzone.id
            );
        }
    }
}

#[test]
fn test_subzone_ids_are_sequential_within_each_zone() {
    let zones = get_all_zones();
    for zone in zones.iter() {
        for (i, subzone) in zone.subzones.iter().enumerate() {
            let expected_id = (i + 1) as u32;
            assert_eq!(
                subzone.id, expected_id,
                "Zone {} subzone at index {} has id {}, expected {}",
                zone.id, i, subzone.id, expected_id
            );
        }
    }
}

#[test]
fn test_subzone_depth_matches_id() {
    // depth should equal subzone id (1-based)
    let zones = get_all_zones();
    for zone in zones.iter() {
        for subzone in zone.subzones.iter() {
            assert_eq!(
                subzone.depth, subzone.id,
                "Zone {} subzone {} depth ({}) should equal id ({})",
                zone.id, subzone.id, subzone.depth, subzone.id
            );
        }
    }
}

// ============================================================================
// Test 8: FractureRegion start/end zone IDs match zone definitions
// ============================================================================

#[test]
fn test_fracture_region_red_fault_zones_exist_in_data() {
    let zones = get_all_zones();
    let region = FractureRegion::RedFault;
    for zone_id in region.start_zone_id()..=region.end_zone_id() {
        assert!(
            zones.iter().any(|z| z.id == zone_id),
            "FractureRegion::RedFault claims zone {} but it does not exist",
            zone_id
        );
    }
}

#[test]
fn test_fracture_region_mirror_scar_zones_exist_in_data() {
    let zones = get_all_zones();
    let region = FractureRegion::MirrorScar;
    for zone_id in region.start_zone_id()..=region.end_zone_id() {
        assert!(
            zones.iter().any(|z| z.id == zone_id),
            "FractureRegion::MirrorScar claims zone {} but it does not exist",
            zone_id
        );
    }
}

#[test]
fn test_fracture_region_black_mouth_zones_exist_in_data() {
    let zones = get_all_zones();
    let region = FractureRegion::BlackMouth;
    for zone_id in region.start_zone_id()..=region.end_zone_id() {
        assert!(
            zones.iter().any(|z| z.id == zone_id),
            "FractureRegion::BlackMouth claims zone {} but it does not exist",
            zone_id
        );
    }
}

#[test]
fn test_fracture_region_hollow_throne_zones_exist_in_data() {
    let zones = get_all_zones();
    let region = FractureRegion::HollowThrone;
    for zone_id in region.start_zone_id()..=region.end_zone_id() {
        assert!(
            zones.iter().any(|z| z.id == zone_id),
            "FractureRegion::HollowThrone claims zone {} but it does not exist",
            zone_id
        );
    }
}

#[test]
fn test_fracture_region_wailing_reach_zones_exist_in_data() {
    let zones = get_all_zones();
    let region = FractureRegion::WailingReach;
    for zone_id in region.start_zone_id()..=region.end_zone_id() {
        assert!(
            zones.iter().any(|z| z.id == zone_id),
            "FractureRegion::WailingReach claims zone {} but it does not exist",
            zone_id
        );
    }
}

#[test]
fn test_fracture_region_origin_wound_zones_exist_in_data() {
    let zones = get_all_zones();
    let region = FractureRegion::OriginWound;
    for zone_id in region.start_zone_id()..=region.end_zone_id() {
        assert!(
            zones.iter().any(|z| z.id == zone_id),
            "FractureRegion::OriginWound claims zone {} but it does not exist",
            zone_id
        );
    }
}

#[test]
fn test_fracture_region_start_zones_are_correct() {
    assert_eq!(FractureRegion::RedFault.start_zone_id(), 12);
    assert_eq!(FractureRegion::MirrorScar.start_zone_id(), 15);
    assert_eq!(FractureRegion::BlackMouth.start_zone_id(), 18);
    assert_eq!(FractureRegion::HollowThrone.start_zone_id(), 21);
    assert_eq!(FractureRegion::WailingReach.start_zone_id(), 24);
    assert_eq!(FractureRegion::OriginWound.start_zone_id(), 27);
}

#[test]
fn test_fracture_region_end_zones_are_correct() {
    assert_eq!(FractureRegion::RedFault.end_zone_id(), 14);
    assert_eq!(FractureRegion::MirrorScar.end_zone_id(), 17);
    assert_eq!(FractureRegion::BlackMouth.end_zone_id(), 20);
    assert_eq!(FractureRegion::HollowThrone.end_zone_id(), 23);
    assert_eq!(FractureRegion::WailingReach.end_zone_id(), 26);
    assert_eq!(FractureRegion::OriginWound.end_zone_id(), 30);
}

#[test]
fn test_fracture_regions_cover_all_fracture_zones_without_gaps() {
    // Fracture zones 12-30 should be fully covered by the six regions without gaps
    let all_regions = [
        FractureRegion::RedFault,
        FractureRegion::MirrorScar,
        FractureRegion::BlackMouth,
        FractureRegion::HollowThrone,
        FractureRegion::WailingReach,
        FractureRegion::OriginWound,
    ];

    let mut covered: HashSet<u32> = HashSet::new();
    for region in &all_regions {
        for zone_id in region.start_zone_id()..=region.end_zone_id() {
            assert!(
                covered.insert(zone_id),
                "Zone {} is covered by multiple fracture regions",
                zone_id
            );
        }
    }

    // Check all fracture zones 12-30 are covered
    for zone_id in 12u32..=30 {
        assert!(
            covered.contains(&zone_id),
            "Fracture zone {} is not covered by any FractureRegion",
            zone_id
        );
    }

    // Check no non-fracture zones are covered
    assert!(
        !covered.contains(&11),
        "Zone 11 (Expanse) should not be in a fracture region"
    );
    assert!(
        !covered.contains(&31),
        "Zone 31 does not exist and should not appear in coverage"
    );
}

#[test]
fn test_fracture_regions_are_contiguous_no_overlap() {
    // Verify regions are adjacent: end of one = start of next - 1
    let pairs = [
        (FractureRegion::RedFault, FractureRegion::MirrorScar),
        (FractureRegion::MirrorScar, FractureRegion::BlackMouth),
        (FractureRegion::BlackMouth, FractureRegion::HollowThrone),
        (FractureRegion::HollowThrone, FractureRegion::WailingReach),
        (FractureRegion::WailingReach, FractureRegion::OriginWound),
    ];
    for (first, second) in &pairs {
        assert_eq!(
            first.end_zone_id() + 1,
            second.start_zone_id(),
            "{:?} ends at zone {} but {:?} starts at zone {} (expected {})",
            first,
            first.end_zone_id(),
            second,
            second.start_zone_id(),
            first.end_zone_id() + 1
        );
    }
}

#[test]
fn test_fracture_region_from_layer_roundtrip() {
    let all_regions = [
        FractureRegion::RedFault,
        FractureRegion::MirrorScar,
        FractureRegion::BlackMouth,
        FractureRegion::HollowThrone,
        FractureRegion::WailingReach,
        FractureRegion::OriginWound,
    ];
    for region in &all_regions {
        let layer = region.unlock_layer();
        let recovered = FractureRegion::from_layer(layer);
        assert_eq!(
            recovered,
            Some(*region),
            "FractureRegion::from_layer({}) should return {:?}",
            layer,
            region
        );
    }
}

#[test]
fn test_fracture_region_from_layer_returns_none_for_non_unlock_layers() {
    // Layers that do not trigger any unlock should return None
    let non_unlock_layers = [0u32, 1, 2, 4, 5, 6, 8, 9, 10, 11, 13, 15, 20, 24, 26, 100];
    for layer in non_unlock_layers {
        assert_eq!(
            FractureRegion::from_layer(layer),
            None,
            "Layer {} should not unlock any fracture region",
            layer
        );
    }
}

#[test]
fn test_fracture_region_unlock_layers_are_monotonically_increasing() {
    let ordered_regions = [
        FractureRegion::RedFault,
        FractureRegion::MirrorScar,
        FractureRegion::BlackMouth,
        FractureRegion::HollowThrone,
        FractureRegion::WailingReach,
        FractureRegion::OriginWound,
    ];
    for i in 1..ordered_regions.len() {
        let prev_layer = ordered_regions[i - 1].unlock_layer();
        let curr_layer = ordered_regions[i].unlock_layer();
        assert!(
            curr_layer > prev_layer,
            "{:?} unlock_layer ({}) should be greater than {:?} unlock_layer ({})",
            ordered_regions[i],
            curr_layer,
            ordered_regions[i - 1],
            prev_layer
        );
    }
}

// ============================================================================
// Additional integrity checks
// ============================================================================

#[test]
fn test_zone_10_is_only_weapon_gated_zone() {
    let zones = get_all_zones();
    for zone in zones.iter() {
        if zone.id == 10 {
            assert!(zone.requires_weapon, "Zone 10 should require a weapon");
            assert_eq!(
                zone.weapon_name,
                Some("Stormbreaker"),
                "Zone 10 weapon should be Stormbreaker"
            );
        } else {
            assert!(
                !zone.requires_weapon,
                "Zone {} should not require a weapon",
                zone.id
            );
        }
    }
}

#[test]
fn test_all_fracture_zones_do_not_require_weapon() {
    let zones = get_all_zones();
    for zone_id in 12u32..=30 {
        let zone = zones.iter().find(|z| z.id == zone_id).unwrap();
        assert!(
            !zone.requires_weapon,
            "Fracture zone {} should not require a weapon",
            zone_id
        );
    }
}

#[test]
fn test_fracture_zones_prestige_req_increases_by_chapter() {
    // Ch.1 (12-14): P50, Ch.2 (15-17): P75, Ch.3 (18-20): P100,
    // Ch.4 (21-23): P150, Ch.5 (24-26): P200, Ch.6 (27-30): P300
    let chapters: &[(u32, u32, u32)] = &[
        (12, 14, 50),
        (15, 17, 75),
        (18, 20, 100),
        (21, 23, 150),
        (24, 26, 200),
        (27, 30, 300),
    ];
    let zones = get_all_zones();
    for &(start, end, prestige_req) in chapters {
        for zone_id in start..=end {
            let zone = zones.iter().find(|z| z.id == zone_id).unwrap();
            assert_eq!(
                zone.prestige_requirement, prestige_req,
                "Zone {} should require P{}, got P{}",
                zone_id, prestige_req, zone.prestige_requirement
            );
        }
    }
}

#[test]
fn test_zone_11_expanse_has_max_level_unlimited() {
    let zones = get_all_zones();
    let expanse = zones.iter().find(|z| z.id == 11).unwrap();
    assert_eq!(
        expanse.max_level,
        u32::MAX,
        "Zone 11 (The Expanse) should have max_level = u32::MAX for unbounded scaling"
    );
}

#[test]
fn test_all_zones_have_valid_level_ranges() {
    let zones = get_all_zones();
    for zone in zones.iter() {
        // Zone 11 special case: max_level = u32::MAX
        if zone.id == 11 {
            continue;
        }
        assert!(
            zone.min_level < zone.max_level,
            "Zone {} min_level ({}) should be less than max_level ({})",
            zone.id,
            zone.min_level,
            zone.max_level
        );
    }
}

#[test]
fn test_zone_descriptions_are_not_empty() {
    let zones = get_all_zones();
    for zone in zones.iter() {
        assert!(
            !zone.description.trim().is_empty(),
            "Zone {} has an empty or whitespace-only description",
            zone.id
        );
    }
}
