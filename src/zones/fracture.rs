//! Fracture region types and helpers.

use serde::{Deserialize, Serialize};

/// Named fracture regions unlocked by Deep layer breakthroughs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FractureRegion {
    RedFault,
    MirrorScar,
    BlackMouth,
    HollowThrone,
    WailingReach,
    OriginWound,
}

#[allow(dead_code)]
impl FractureRegion {
    /// First zone ID in this region.
    pub fn start_zone_id(&self) -> u32 {
        match self {
            Self::RedFault => 12,
            Self::MirrorScar => 15,
            Self::BlackMouth => 18,
            Self::HollowThrone => 21,
            Self::WailingReach => 24,
            Self::OriginWound => 27,
        }
    }

    /// Last zone ID in this region (the cap zone).
    pub fn end_zone_id(&self) -> u32 {
        match self {
            Self::RedFault => 14,
            Self::MirrorScar => 17,
            Self::BlackMouth => 20,
            Self::HollowThrone => 23,
            Self::WailingReach => 26,
            Self::OriginWound => 30, // 4 zones (Z27-30), finale chapter
        }
    }

    /// Deep layer whose breakthrough unlocks this region.
    pub fn unlock_layer(&self) -> u32 {
        match self {
            Self::RedFault => 3,
            Self::MirrorScar => 7,
            Self::BlackMouth => 12,
            Self::HollowThrone => 18,
            Self::WailingReach => 25,
            Self::OriginWound => 30,
        }
    }

    /// Returns the region unlocked by a given Deep layer breakthrough, if any.
    pub fn from_layer(layer: u32) -> Option<Self> {
        match layer {
            3 => Some(Self::RedFault),
            7 => Some(Self::MirrorScar),
            12 => Some(Self::BlackMouth),
            18 => Some(Self::HollowThrone),
            25 => Some(Self::WailingReach),
            30 => Some(Self::OriginWound),
            _ => None,
        }
    }

    /// Full-caps headline for the unlock modal.
    pub fn unlock_headline(&self) -> &'static str {
        match self {
            Self::RedFault => "THE RED FAULT OPENS",
            Self::MirrorScar => "THE MIRROR SCAR AWAKES",
            Self::BlackMouth => "THE BLACK MOUTH UNSEALS",
            Self::HollowThrone => "THE HOLLOW THRONE REVEALS",
            Self::WailingReach => "THE WAILING REACH CALLS",
            Self::OriginWound => "THE ORIGIN WOUND OPENS",
        }
    }

    /// Atmospheric text for the unlock modal.
    pub fn unlock_atmospheric(&self) -> &'static str {
        match self {
            Self::RedFault => "The surface has split, and the wound is burning.",
            Self::MirrorScar => "The horizon has cracked. Reflection now bleeds into the world.",
            Self::BlackMouth => "The final wound has opened wide enough to hunger.",
            Self::HollowThrone => "Beyond the wound, a kingdom older than the world still waits.",
            Self::WailingReach => "Reality forgets itself here. Sound has learned to weep.",
            Self::OriginWound => {
                "The first fracture. The wound that was here before the world it broke."
            }
        }
    }

    /// Narrative bridge text tying the fracture to its Power Core.
    pub fn power_core_narrative(&self) -> &'static str {
        match self {
            Self::RedFault => {
                "Deep within, a core of power ignites \u{2014} the Red Fault now feeds your ascent."
            }
            Self::MirrorScar => "A second core awakens in the Mirror Scar.",
            Self::BlackMouth => "A core pulses in the Black Mouth, feeding on the dark.",
            Self::HollowThrone => "The Hollow Throne yields an ancient core.",
            Self::WailingReach => "The Wailing Reach resonates with a core beyond hearing.",
            Self::OriginWound => "Its final core beats at the origin.",
        }
    }

    /// Mechanical text for the unlock modal.
    pub fn unlock_mechanical(&self) -> &'static str {
        match self {
            Self::RedFault => "Zones 12-14 are now reachable beyond the current frontier.",
            Self::MirrorScar => "Zones 15-17 are now reachable beyond the current frontier.",
            Self::BlackMouth => "Zones 18-20 are now reachable beyond the current frontier.",
            Self::HollowThrone => "Zones 21-23 are now reachable beyond the current frontier.",
            Self::WailingReach => "Zones 24-26 are now reachable beyond the current frontier.",
            Self::OriginWound => "Zones 27-30 are now reachable beyond the current frontier.",
        }
    }

    /// Narrative bridge text tying the fracture to Ascension.
    pub fn ascension_narrative(&self) -> &'static str {
        match self {
            Self::RedFault => {
                "The fracture has tested you. Your strength can now transcend mortal limits."
            }
            Self::MirrorScar => "The rift deepens. Greater power stirs within you.",
            Self::BlackMouth => {
                "The abyss answers. Unimaginable power awaits those who dare claim it."
            }
            Self::HollowThrone => {
                "The throne's guardians have fallen. Power beyond reckoning yields to your will."
            }
            Self::WailingReach => {
                "The reach has acknowledged you. Even silence bows before your strength."
            }
            Self::OriginWound => {
                "You stand at the source of all breaking. Nothing remains that can challenge you."
            }
        }
    }

    /// The Ascension level that becomes newly available at this region's unlock.
    /// RedFault (Layer 3) -> Ascension I, MirrorScar (Layer 7) -> II, BlackMouth (Layer 12) -> III,
    /// HollowThrone (Layer 18) -> IV, WailingReach (Layer 25) -> V, OriginWound (Layer 30) -> VI.
    pub fn ascension_level_unlocked(&self) -> u32 {
        match self {
            Self::RedFault => 1,
            Self::MirrorScar => 2,
            Self::BlackMouth => 3,
            Self::HollowThrone => 4,
            Self::WailingReach => 5,
            Self::OriginWound => 6,
        }
    }

    /// Combat log line.
    pub fn unlock_log_line(&self) -> &'static str {
        match self {
            Self::RedFault => "The Red Fault has opened beyond the Expanse.",
            Self::MirrorScar => "The Mirror Scar has awakened beyond the frontier.",
            Self::BlackMouth => "The Black Mouth has unsealed beyond the world's wound.",
            Self::HollowThrone => "The Hollow Throne has revealed itself beneath the wound.",
            Self::WailingReach => "The Wailing Reach calls from the boundary of existence.",
            Self::OriginWound => "The Origin Wound has opened at the source of all fractures.",
        }
    }

    /// Ticker text.
    pub fn unlock_ticker_text(&self) -> &'static str {
        match self {
            Self::RedFault => "Red Fault available",
            Self::MirrorScar => "Mirror Scar available",
            Self::BlackMouth => "Black Mouth available",
            Self::HollowThrone => "Hollow Throne available",
            Self::WailingReach => "Wailing Reach available",
            Self::OriginWound => "Origin Wound available",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_red_fault_zone_range() {
        assert_eq!(FractureRegion::RedFault.start_zone_id(), 12);
        assert_eq!(FractureRegion::RedFault.end_zone_id(), 14);
    }

    #[test]
    fn test_mirror_scar_zone_range() {
        assert_eq!(FractureRegion::MirrorScar.start_zone_id(), 15);
        assert_eq!(FractureRegion::MirrorScar.end_zone_id(), 17);
    }

    #[test]
    fn test_black_mouth_zone_range() {
        assert_eq!(FractureRegion::BlackMouth.start_zone_id(), 18);
        assert_eq!(FractureRegion::BlackMouth.end_zone_id(), 20);
    }

    #[test]
    fn test_unlock_layers() {
        assert_eq!(FractureRegion::RedFault.unlock_layer(), 3);
        assert_eq!(FractureRegion::MirrorScar.unlock_layer(), 7);
        assert_eq!(FractureRegion::BlackMouth.unlock_layer(), 12);
    }

    #[test]
    fn test_region_from_layer() {
        assert_eq!(
            FractureRegion::from_layer(3),
            Some(FractureRegion::RedFault)
        );
        assert_eq!(
            FractureRegion::from_layer(7),
            Some(FractureRegion::MirrorScar)
        );
        assert_eq!(
            FractureRegion::from_layer(12),
            Some(FractureRegion::BlackMouth)
        );
        assert_eq!(FractureRegion::from_layer(13), None);
        assert_eq!(FractureRegion::from_layer(5), None);
    }

    #[test]
    fn test_unlock_headline() {
        assert_eq!(
            FractureRegion::RedFault.unlock_headline(),
            "THE RED FAULT OPENS"
        );
    }

    #[test]
    fn test_unlock_headline_all_variants() {
        assert_eq!(
            FractureRegion::MirrorScar.unlock_headline(),
            "THE MIRROR SCAR AWAKES"
        );
        assert_eq!(
            FractureRegion::BlackMouth.unlock_headline(),
            "THE BLACK MOUTH UNSEALS"
        );
        assert_eq!(
            FractureRegion::HollowThrone.unlock_headline(),
            "THE HOLLOW THRONE REVEALS"
        );
        assert_eq!(
            FractureRegion::WailingReach.unlock_headline(),
            "THE WAILING REACH CALLS"
        );
        assert_eq!(
            FractureRegion::OriginWound.unlock_headline(),
            "THE ORIGIN WOUND OPENS"
        );
    }

    const ALL_REGIONS: [FractureRegion; 6] = [
        FractureRegion::RedFault,
        FractureRegion::MirrorScar,
        FractureRegion::BlackMouth,
        FractureRegion::HollowThrone,
        FractureRegion::WailingReach,
        FractureRegion::OriginWound,
    ];

    #[test]
    fn test_unlock_atmospheric_all_variants() {
        assert_eq!(
            FractureRegion::RedFault.unlock_atmospheric(),
            "The surface has split, and the wound is burning."
        );
        assert_eq!(
            FractureRegion::MirrorScar.unlock_atmospheric(),
            "The horizon has cracked. Reflection now bleeds into the world."
        );
        assert_eq!(
            FractureRegion::BlackMouth.unlock_atmospheric(),
            "The final wound has opened wide enough to hunger."
        );
        assert_eq!(
            FractureRegion::HollowThrone.unlock_atmospheric(),
            "Beyond the wound, a kingdom older than the world still waits."
        );
        assert_eq!(
            FractureRegion::WailingReach.unlock_atmospheric(),
            "Reality forgets itself here. Sound has learned to weep."
        );
        assert_eq!(
            FractureRegion::OriginWound.unlock_atmospheric(),
            "The first fracture. The wound that was here before the world it broke."
        );
    }

    #[test]
    fn test_power_core_narrative_all_variants() {
        assert!(FractureRegion::RedFault
            .power_core_narrative()
            .contains("Red Fault"));
        assert_eq!(
            FractureRegion::MirrorScar.power_core_narrative(),
            "A second core awakens in the Mirror Scar."
        );
        assert_eq!(
            FractureRegion::BlackMouth.power_core_narrative(),
            "A core pulses in the Black Mouth, feeding on the dark."
        );
        assert_eq!(
            FractureRegion::HollowThrone.power_core_narrative(),
            "The Hollow Throne yields an ancient core."
        );
        assert_eq!(
            FractureRegion::WailingReach.power_core_narrative(),
            "The Wailing Reach resonates with a core beyond hearing."
        );
        assert_eq!(
            FractureRegion::OriginWound.power_core_narrative(),
            "Its final core beats at the origin."
        );
    }

    #[test]
    fn test_unlock_mechanical_all_variants() {
        assert_eq!(
            FractureRegion::RedFault.unlock_mechanical(),
            "Zones 12-14 are now reachable beyond the current frontier."
        );
        assert_eq!(
            FractureRegion::MirrorScar.unlock_mechanical(),
            "Zones 15-17 are now reachable beyond the current frontier."
        );
        assert_eq!(
            FractureRegion::BlackMouth.unlock_mechanical(),
            "Zones 18-20 are now reachable beyond the current frontier."
        );
        assert_eq!(
            FractureRegion::HollowThrone.unlock_mechanical(),
            "Zones 21-23 are now reachable beyond the current frontier."
        );
        assert_eq!(
            FractureRegion::WailingReach.unlock_mechanical(),
            "Zones 24-26 are now reachable beyond the current frontier."
        );
        assert_eq!(
            FractureRegion::OriginWound.unlock_mechanical(),
            "Zones 27-30 are now reachable beyond the current frontier."
        );
    }

    #[test]
    fn test_ascension_narrative_all_variants() {
        assert_eq!(
            FractureRegion::RedFault.ascension_narrative(),
            "The fracture has tested you. Your strength can now transcend mortal limits."
        );
        assert_eq!(
            FractureRegion::MirrorScar.ascension_narrative(),
            "The rift deepens. Greater power stirs within you."
        );
        assert_eq!(
            FractureRegion::BlackMouth.ascension_narrative(),
            "The abyss answers. Unimaginable power awaits those who dare claim it."
        );
        assert_eq!(
            FractureRegion::HollowThrone.ascension_narrative(),
            "The throne's guardians have fallen. Power beyond reckoning yields to your will."
        );
        assert_eq!(
            FractureRegion::WailingReach.ascension_narrative(),
            "The reach has acknowledged you. Even silence bows before your strength."
        );
        assert_eq!(
            FractureRegion::OriginWound.ascension_narrative(),
            "You stand at the source of all breaking. Nothing remains that can challenge you."
        );
    }

    #[test]
    fn test_ascension_level_unlocked_all_variants() {
        assert_eq!(FractureRegion::RedFault.ascension_level_unlocked(), 1);
        assert_eq!(FractureRegion::MirrorScar.ascension_level_unlocked(), 2);
        assert_eq!(FractureRegion::BlackMouth.ascension_level_unlocked(), 3);
        assert_eq!(FractureRegion::HollowThrone.ascension_level_unlocked(), 4);
        assert_eq!(FractureRegion::WailingReach.ascension_level_unlocked(), 5);
        assert_eq!(FractureRegion::OriginWound.ascension_level_unlocked(), 6);
    }

    #[test]
    fn test_unlock_log_line_all_variants() {
        assert_eq!(
            FractureRegion::RedFault.unlock_log_line(),
            "The Red Fault has opened beyond the Expanse."
        );
        assert_eq!(
            FractureRegion::MirrorScar.unlock_log_line(),
            "The Mirror Scar has awakened beyond the frontier."
        );
        assert_eq!(
            FractureRegion::BlackMouth.unlock_log_line(),
            "The Black Mouth has unsealed beyond the world's wound."
        );
        assert_eq!(
            FractureRegion::HollowThrone.unlock_log_line(),
            "The Hollow Throne has revealed itself beneath the wound."
        );
        assert_eq!(
            FractureRegion::WailingReach.unlock_log_line(),
            "The Wailing Reach calls from the boundary of existence."
        );
        assert_eq!(
            FractureRegion::OriginWound.unlock_log_line(),
            "The Origin Wound has opened at the source of all fractures."
        );
    }

    #[test]
    fn test_unlock_ticker_text_all_variants() {
        assert_eq!(
            FractureRegion::RedFault.unlock_ticker_text(),
            "Red Fault available"
        );
        assert_eq!(
            FractureRegion::MirrorScar.unlock_ticker_text(),
            "Mirror Scar available"
        );
        assert_eq!(
            FractureRegion::BlackMouth.unlock_ticker_text(),
            "Black Mouth available"
        );
        assert_eq!(
            FractureRegion::HollowThrone.unlock_ticker_text(),
            "Hollow Throne available"
        );
        assert_eq!(
            FractureRegion::WailingReach.unlock_ticker_text(),
            "Wailing Reach available"
        );
        assert_eq!(
            FractureRegion::OriginWound.unlock_ticker_text(),
            "Origin Wound available"
        );
    }

    #[test]
    fn test_all_regions_have_non_empty_narrative_text() {
        for region in ALL_REGIONS {
            assert!(!region.unlock_headline().is_empty());
            assert!(!region.unlock_atmospheric().is_empty());
            assert!(!region.power_core_narrative().is_empty());
            assert!(!region.unlock_mechanical().is_empty());
            assert!(!region.ascension_narrative().is_empty());
            assert!(!region.unlock_log_line().is_empty());
            assert!(!region.unlock_ticker_text().is_empty());
        }
    }

    #[test]
    fn test_all_layer_gates_round_trip_to_matching_region() {
        for region in ALL_REGIONS {
            assert_eq!(
                FractureRegion::from_layer(region.unlock_layer()),
                Some(region)
            );
        }
    }

    #[test]
    fn test_end_zone_id_is_start_plus_two_except_origin_wound() {
        for region in ALL_REGIONS {
            if region == FractureRegion::OriginWound {
                // Origin Wound has 4 zones (finale chapter), not 3.
                assert_eq!(region.end_zone_id(), region.start_zone_id() + 3);
            } else {
                assert_eq!(region.end_zone_id(), region.start_zone_id() + 2);
            }
        }
    }

    #[test]
    fn test_serde_round_trip() {
        let region = FractureRegion::MirrorScar;
        let json = serde_json::to_string(&region).unwrap();
        let loaded: FractureRegion = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded, FractureRegion::MirrorScar);
    }

    #[test]
    fn test_hollow_throne_zone_range() {
        assert_eq!(FractureRegion::HollowThrone.start_zone_id(), 21);
        assert_eq!(FractureRegion::HollowThrone.end_zone_id(), 23);
    }

    #[test]
    fn test_wailing_reach_zone_range() {
        assert_eq!(FractureRegion::WailingReach.start_zone_id(), 24);
        assert_eq!(FractureRegion::WailingReach.end_zone_id(), 26);
    }

    #[test]
    fn test_origin_wound_zone_range() {
        assert_eq!(FractureRegion::OriginWound.start_zone_id(), 27);
        assert_eq!(FractureRegion::OriginWound.end_zone_id(), 30);
    }

    #[test]
    fn test_new_unlock_layers() {
        assert_eq!(FractureRegion::HollowThrone.unlock_layer(), 18);
        assert_eq!(FractureRegion::WailingReach.unlock_layer(), 25);
        assert_eq!(FractureRegion::OriginWound.unlock_layer(), 30);
    }

    #[test]
    fn test_new_region_from_layer() {
        assert_eq!(
            FractureRegion::from_layer(18),
            Some(FractureRegion::HollowThrone)
        );
        assert_eq!(
            FractureRegion::from_layer(25),
            Some(FractureRegion::WailingReach)
        );
        assert_eq!(
            FractureRegion::from_layer(30),
            Some(FractureRegion::OriginWound)
        );
    }

    #[test]
    fn test_serde_round_trip_hollow_throne() {
        let region = FractureRegion::HollowThrone;
        let json = serde_json::to_string(&region).unwrap();
        let loaded: FractureRegion = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded, FractureRegion::HollowThrone);
    }

    #[test]
    fn test_serde_round_trip_wailing_reach() {
        let region = FractureRegion::WailingReach;
        let json = serde_json::to_string(&region).unwrap();
        let loaded: FractureRegion = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded, FractureRegion::WailingReach);
    }

    #[test]
    fn test_serde_round_trip_origin_wound() {
        let region = FractureRegion::OriginWound;
        let json = serde_json::to_string(&region).unwrap();
        let loaded: FractureRegion = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded, FractureRegion::OriginWound);
    }
}
