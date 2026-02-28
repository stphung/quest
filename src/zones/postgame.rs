//! Postgame region types and helpers.

use serde::{Deserialize, Serialize};

/// Named postgame chapters, each containing 3 zones.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PostgameRegion {
    RedFault,
    MirrorScar,
    BlackMouth,
}

impl PostgameRegion {
    /// First zone ID in this region.
    pub fn start_zone_id(&self) -> u32 {
        match self {
            Self::RedFault => 12,
            Self::MirrorScar => 15,
            Self::BlackMouth => 18,
        }
    }

    /// Last zone ID in this region (the cap zone).
    pub fn end_zone_id(&self) -> u32 {
        match self {
            Self::RedFault => 14,
            Self::MirrorScar => 17,
            Self::BlackMouth => 20,
        }
    }

    /// Deep layer whose breakthrough unlocks this region.
    pub fn unlock_layer(&self) -> u32 {
        match self {
            Self::RedFault => 3,
            Self::MirrorScar => 7,
            Self::BlackMouth => 13,
        }
    }

    /// Returns the region unlocked by a given Deep layer breakthrough, if any.
    pub fn from_layer(layer: u32) -> Option<Self> {
        match layer {
            3 => Some(Self::RedFault),
            7 => Some(Self::MirrorScar),
            13 => Some(Self::BlackMouth),
            _ => None,
        }
    }

    /// Full-caps headline for the unlock modal.
    pub fn unlock_headline(&self) -> &'static str {
        match self {
            Self::RedFault => "THE RED FAULT OPENS",
            Self::MirrorScar => "THE MIRROR SCAR AWAKES",
            Self::BlackMouth => "THE BLACK MOUTH UNSEALS",
        }
    }

    /// Atmospheric text for the unlock modal.
    pub fn unlock_atmospheric(&self) -> &'static str {
        match self {
            Self::RedFault => "The surface has split, and the wound is burning.",
            Self::MirrorScar => "The horizon has cracked. Reflection now bleeds into the world.",
            Self::BlackMouth => "The final wound has opened wide enough to hunger.",
        }
    }

    /// Mechanical text for the unlock modal.
    pub fn unlock_mechanical(&self) -> &'static str {
        match self {
            Self::RedFault => "Zones 12-14 are now reachable beyond the current frontier.",
            Self::MirrorScar => "Zones 15-17 are now reachable beyond the current frontier.",
            Self::BlackMouth => "Zones 18-20 are now reachable beyond the current frontier.",
        }
    }

    /// Combat log line.
    pub fn unlock_log_line(&self) -> &'static str {
        match self {
            Self::RedFault => "The Red Fault has opened beyond the Expanse.",
            Self::MirrorScar => "The Mirror Scar has awakened beyond the frontier.",
            Self::BlackMouth => "The Black Mouth has unsealed beyond the world's wound.",
        }
    }

    /// Ticker text.
    pub fn unlock_ticker_text(&self) -> &'static str {
        match self {
            Self::RedFault => "Red Fault available",
            Self::MirrorScar => "Mirror Scar available",
            Self::BlackMouth => "Black Mouth available",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_red_fault_zone_range() {
        assert_eq!(PostgameRegion::RedFault.start_zone_id(), 12);
        assert_eq!(PostgameRegion::RedFault.end_zone_id(), 14);
    }

    #[test]
    fn test_mirror_scar_zone_range() {
        assert_eq!(PostgameRegion::MirrorScar.start_zone_id(), 15);
        assert_eq!(PostgameRegion::MirrorScar.end_zone_id(), 17);
    }

    #[test]
    fn test_black_mouth_zone_range() {
        assert_eq!(PostgameRegion::BlackMouth.start_zone_id(), 18);
        assert_eq!(PostgameRegion::BlackMouth.end_zone_id(), 20);
    }

    #[test]
    fn test_unlock_layers() {
        assert_eq!(PostgameRegion::RedFault.unlock_layer(), 3);
        assert_eq!(PostgameRegion::MirrorScar.unlock_layer(), 7);
        assert_eq!(PostgameRegion::BlackMouth.unlock_layer(), 13);
    }

    #[test]
    fn test_region_from_layer() {
        assert_eq!(
            PostgameRegion::from_layer(3),
            Some(PostgameRegion::RedFault)
        );
        assert_eq!(
            PostgameRegion::from_layer(7),
            Some(PostgameRegion::MirrorScar)
        );
        assert_eq!(
            PostgameRegion::from_layer(13),
            Some(PostgameRegion::BlackMouth)
        );
        assert_eq!(PostgameRegion::from_layer(5), None);
    }

    #[test]
    fn test_unlock_headline() {
        assert_eq!(
            PostgameRegion::RedFault.unlock_headline(),
            "THE RED FAULT OPENS"
        );
    }

    #[test]
    fn test_serde_round_trip() {
        let region = PostgameRegion::MirrorScar;
        let json = serde_json::to_string(&region).unwrap();
        let loaded: PostgameRegion = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded, PostgameRegion::MirrorScar);
    }
}
