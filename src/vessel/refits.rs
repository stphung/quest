//! Refits — the three A/B doors (sub-project 4).
//!
//! At the first three distinct shipyards visited, the yard offers one
//! permanent either/or refit. Choosing A closes B forever. Configuration,
//! not consumables: every effect composes into the same integer prices the
//! cards already show.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RefitId {
    StormSail,
    LongHold,
    QuietKeel,
    DeepLarder,
    LanternMast,
    MourningColors,
}

impl RefitId {
    pub fn display_name(&self) -> &'static str {
        match self {
            RefitId::StormSail => "Storm Sail",
            RefitId::LongHold => "Long Hold",
            RefitId::QuietKeel => "Quiet Keel",
            RefitId::DeepLarder => "Deep Larder",
            RefitId::LanternMast => "Lantern Mast",
            RefitId::MourningColors => "Mourning Colors",
        }
    }

    /// One sentence, final effects only — never a multiplier.
    pub fn blurb(&self) -> &'static str {
        match self {
            RefitId::StormSail => "every leg arrives sooner",
            RefitId::LongHold => "the hold carries 150",
            RefitId::QuietKeel => "named threats treat the hull kindly",
            RefitId::DeepLarder => "a drift ends with 40 in the hold",
            RefitId::LanternMast => "the chart names what lies one road further",
            RefitId::MourningColors => "Mourn sails thriftier still",
        }
    }
}

/// An A/B pair as the yard offers it.
#[derive(Debug, Clone, Copy)]
pub struct RefitPair {
    pub a: RefitId,
    pub b: RefitId,
}

/// The authored sequence: route choice decides *which yards*, never
/// *which pairs*.
pub const REFIT_PAIRS: [RefitPair; 3] = [
    RefitPair {
        a: RefitId::StormSail,
        b: RefitId::LongHold,
    },
    RefitPair {
        a: RefitId::QuietKeel,
        b: RefitId::DeepLarder,
    },
    RefitPair {
        a: RefitId::LanternMast,
        b: RefitId::MourningColors,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_pairs_six_distinct_refits() {
        let mut seen = std::collections::HashSet::new();
        for pair in REFIT_PAIRS {
            assert!(seen.insert(pair.a));
            assert!(seen.insert(pair.b));
            assert!(!pair.a.display_name().is_empty());
            assert!(!pair.b.blurb().is_empty());
        }
        assert_eq!(seen.len(), 6);
    }
}
