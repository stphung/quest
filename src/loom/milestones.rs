//! Pattern milestone types and helpers for key pattern completion modals.

use serde::{Deserialize, Serialize};

/// Key pattern completion milestones that trigger celebration modals.
/// Similar to `FractureRegion` for Deep layer breakthroughs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PatternMilestone {
    /// 4 patterns — Thread Wilds (Z31-34)
    ThreadWilds,
    /// 8 patterns — Woven Frontier (Z35-38) + Ascension VII
    WovenFrontier,
    /// 16 patterns — The Unraveling (Z39-42) + Ascension VIII
    TheUnraveling,
    /// 22 patterns — Grand Design (Z43-46) + Ascension IX
    GrandDesign,
    /// 28 patterns — Final Weave (Z47-50) + Ascension X + WR→PR
    FinalWeave,
}

#[allow(dead_code)]
impl PatternMilestone {
    /// Returns the milestone triggered by the given completed pattern count, if any.
    pub fn from_count(completed: usize) -> Option<Self> {
        match completed {
            4 => Some(Self::ThreadWilds),
            8 => Some(Self::WovenFrontier),
            16 => Some(Self::TheUnraveling),
            22 => Some(Self::GrandDesign),
            28 => Some(Self::FinalWeave),
            _ => None,
        }
    }

    /// Number of completed patterns at this milestone.
    pub fn pattern_count(&self) -> usize {
        match self {
            Self::ThreadWilds => 4,
            Self::WovenFrontier => 8,
            Self::TheUnraveling => 16,
            Self::GrandDesign => 22,
            Self::FinalWeave => 28,
        }
    }

    /// First Loom zone ID unlocked at this milestone.
    pub fn start_zone_id(&self) -> u32 {
        match self {
            Self::ThreadWilds => 31,
            Self::WovenFrontier => 35,
            Self::TheUnraveling => 39,
            Self::GrandDesign => 43,
            Self::FinalWeave => 47,
        }
    }

    /// Last Loom zone ID unlocked at this milestone.
    pub fn end_zone_id(&self) -> u32 {
        match self {
            Self::ThreadWilds => 34,
            Self::WovenFrontier => 38,
            Self::TheUnraveling => 42,
            Self::GrandDesign => 46,
            Self::FinalWeave => 50,
        }
    }

    /// Full-caps headline for the unlock modal.
    pub fn unlock_headline(&self) -> &'static str {
        match self {
            Self::ThreadWilds => "THE THREAD WILDS AWAKEN",
            Self::WovenFrontier => "THE WOVEN FRONTIER UNFOLDS",
            Self::TheUnraveling => "THE UNRAVELING BEGINS",
            Self::GrandDesign => "THE GRAND DESIGN REVEALS",
            Self::FinalWeave => "THE FINAL WEAVE COMPLETES",
        }
    }

    /// Atmospheric text for the unlock modal.
    pub fn unlock_atmospheric(&self) -> &'static str {
        match self {
            Self::ThreadWilds => {
                "Wild threads pulse with untamed energy. A new frontier of reality beckons."
            }
            Self::WovenFrontier => {
                "The loom hums with deeper resonance. Threads weave into patterns never seen before."
            }
            Self::TheUnraveling => {
                "The fabric strains. What was woven must be unwoven to weave anew."
            }
            Self::GrandDesign => {
                "The pattern of all patterns emerges. Creation's blueprint lies bare."
            }
            Self::FinalWeave => {
                "Every thread finds its place. The Loom sings with a voice older than worlds."
            }
        }
    }

    /// Mechanical text for the unlock modal.
    pub fn unlock_mechanical(&self) -> &'static str {
        match self {
            Self::ThreadWilds => "Zones 31-34 are now reachable beyond the fracture frontier.",
            Self::WovenFrontier => "Zones 35-38 are now reachable beyond the thread wilds.",
            Self::TheUnraveling => "Zones 39-42 are now reachable beyond the woven frontier.",
            Self::GrandDesign => "Zones 43-46 are now reachable beyond the unraveling.",
            Self::FinalWeave => "Zones 47-50 are now reachable. The Loom is complete.",
        }
    }

    /// Ascension level unlocked at this milestone (None if no new Ascension).
    pub fn ascension_level_unlocked(&self) -> Option<u32> {
        match self {
            Self::ThreadWilds => None,
            Self::WovenFrontier => Some(7),
            Self::TheUnraveling => Some(8),
            Self::GrandDesign => Some(9),
            Self::FinalWeave => Some(10),
        }
    }

    /// Narrative bridge text for Ascension.
    pub fn ascension_narrative(&self) -> &'static str {
        match self {
            Self::ThreadWilds => "",
            Self::WovenFrontier => {
                "The patterns have strengthened you. A new tier of Ascension awaits."
            }
            Self::TheUnraveling => "The unraveling has forged you anew. Greater Ascension beckons.",
            Self::GrandDesign => {
                "The grand design demands grand power. Ascend beyond mortal reckoning."
            }
            Self::FinalWeave => {
                "The Loom is whole. Ultimate Ascension awaits those who dare claim it."
            }
        }
    }

    /// Combat log line.
    pub fn unlock_log_line(&self) -> &'static str {
        match self {
            Self::ThreadWilds => "The Thread Wilds have awakened beyond the fracture frontier.",
            Self::WovenFrontier => "The Woven Frontier has unfolded beyond the thread wilds.",
            Self::TheUnraveling => "The Unraveling has begun beyond the woven frontier.",
            Self::GrandDesign => "The Grand Design has revealed itself.",
            Self::FinalWeave => "The Final Weave is complete. The Loom sings.",
        }
    }

    /// Ticker text.
    pub fn unlock_ticker_text(&self) -> &'static str {
        match self {
            Self::ThreadWilds => "Thread Wilds open",
            Self::WovenFrontier => "Woven Frontier open",
            Self::TheUnraveling => "Unraveling begins",
            Self::GrandDesign => "Grand Design revealed",
            Self::FinalWeave => "Final Weave complete",
        }
    }

    /// Extra narrative for WR→PR conversion (only for FinalWeave).
    pub fn wr_pr_narrative(&self) -> Option<&'static str> {
        match self {
            Self::FinalWeave => {
                Some("The completed Loom now converts Woven Reality into Prestige Ranks.")
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_milestone_from_count() {
        assert_eq!(PatternMilestone::from_count(0), None);
        assert_eq!(PatternMilestone::from_count(3), None);
        assert_eq!(
            PatternMilestone::from_count(4),
            Some(PatternMilestone::ThreadWilds)
        );
        assert_eq!(
            PatternMilestone::from_count(8),
            Some(PatternMilestone::WovenFrontier)
        );
        assert_eq!(PatternMilestone::from_count(10), None);
        assert_eq!(
            PatternMilestone::from_count(16),
            Some(PatternMilestone::TheUnraveling)
        );
        assert_eq!(
            PatternMilestone::from_count(22),
            Some(PatternMilestone::GrandDesign)
        );
        assert_eq!(
            PatternMilestone::from_count(28),
            Some(PatternMilestone::FinalWeave)
        );
    }

    #[test]
    fn test_milestone_zone_ranges() {
        assert_eq!(PatternMilestone::ThreadWilds.start_zone_id(), 31);
        assert_eq!(PatternMilestone::ThreadWilds.end_zone_id(), 34);
        assert_eq!(PatternMilestone::FinalWeave.start_zone_id(), 47);
        assert_eq!(PatternMilestone::FinalWeave.end_zone_id(), 50);
    }

    #[test]
    fn test_milestone_pattern_counts() {
        assert_eq!(PatternMilestone::ThreadWilds.pattern_count(), 4);
        assert_eq!(PatternMilestone::WovenFrontier.pattern_count(), 8);
        assert_eq!(PatternMilestone::TheUnraveling.pattern_count(), 16);
        assert_eq!(PatternMilestone::GrandDesign.pattern_count(), 22);
        assert_eq!(PatternMilestone::FinalWeave.pattern_count(), 28);
    }

    #[test]
    fn test_milestone_ascension_levels() {
        assert_eq!(
            PatternMilestone::ThreadWilds.ascension_level_unlocked(),
            None
        );
        assert_eq!(
            PatternMilestone::WovenFrontier.ascension_level_unlocked(),
            Some(7)
        );
        assert_eq!(
            PatternMilestone::FinalWeave.ascension_level_unlocked(),
            Some(10)
        );
    }

    #[test]
    fn test_serde_round_trip() {
        let milestone = PatternMilestone::GrandDesign;
        let json = serde_json::to_string(&milestone).unwrap();
        let loaded: PatternMilestone = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded, PatternMilestone::GrandDesign);
    }
}
