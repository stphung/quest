//! Power Cores data model: per-layer passive PR generation system.
//!
//! Each Power Core is unlocked by clearing a specific Deep layer milestone achievement.
//! Once unlocked, a core passively generates prestige ranks over time at a fixed rate.
//! State is persisted to `~/.quest/passives.json` as part of the generic [`PassivesState`].

use crate::achievements::{AchievementId, Achievements};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single passive generator's timer state.
///
/// Stored as a struct (rather than a bare `i64`) so that future fields can be
/// added with `#[serde(default)]` without a breaking schema change.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GeneratorTimer {
    /// Unix timestamp (seconds) when this generator last granted its resource.
    /// A value of `0` means "never granted" (the generator was just initialised).
    pub last_granted_at: i64,
}

/// Top-level persistent state for all passive generators.
///
/// Saved to `~/.quest/passives.json`. Each entry in [`generators`](Self::generators)
/// is keyed by a stable string identifier (e.g. `"power_core_1"`) that maps to
/// a static definition elsewhere in code.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PassivesState {
    /// Timer state per generator, keyed by stable string ID.
    #[serde(default)]
    pub generators: HashMap<String, GeneratorTimer>,
}

/// Static definition of a Power Core.
#[derive(Debug, Clone)]
pub struct PowerCoreDef {
    /// Stable string key used in [`PassivesState::generators`].
    pub key: &'static str,
    /// The achievement that unlocks this core.
    pub achievement_id: AchievementId,
    /// Display name of the core.
    pub name: &'static str,
    /// Prestige ranks granted per day (86400 seconds).
    pub pr_per_day: u32,
    /// Deep layer required to unlock this core.
    pub required_layer: u32,
}

/// All Power Core definitions, indexed by Power Core achievement milestones.
pub const ALL_POWER_CORES: &[PowerCoreDef] = &[
    PowerCoreDef {
        key: "power_core_1",
        achievement_id: AchievementId::PowerCoreI,
        name: "Red Fault",
        pr_per_day: 2,
        required_layer: 3,
    },
    PowerCoreDef {
        key: "power_core_2",
        achievement_id: AchievementId::PowerCoreII,
        name: "Mirror Scar",
        pr_per_day: 3,
        required_layer: 7,
    },
    PowerCoreDef {
        key: "power_core_3",
        achievement_id: AchievementId::PowerCoreIII,
        name: "Black Mouth",
        pr_per_day: 5,
        required_layer: 12,
    },
    PowerCoreDef {
        key: "power_core_4",
        achievement_id: AchievementId::PowerCoreIV,
        name: "Hollow Throne",
        pr_per_day: 8,
        required_layer: 18,
    },
    PowerCoreDef {
        key: "power_core_5",
        achievement_id: AchievementId::PowerCoreV,
        name: "Wailing Reach",
        pr_per_day: 12,
        required_layer: 25,
    },
    PowerCoreDef {
        key: "power_core_6",
        achievement_id: AchievementId::PowerCoreVI,
        name: "Origin Wound",
        pr_per_day: 18,
        required_layer: 30,
    },
];

/// Look up the static definition for a given achievement ID.
///
/// Returns `None` if `id` does not correspond to any Power Core.
pub fn get_power_core_def(id: AchievementId) -> Option<&'static PowerCoreDef> {
    ALL_POWER_CORES.iter().find(|def| def.achievement_id == id)
}

/// Return all Power Core definitions whose unlock achievement has been earned.
pub fn get_unlocked_cores(achievements: &Achievements) -> Vec<&'static PowerCoreDef> {
    ALL_POWER_CORES
        .iter()
        .filter(|def| achievements.is_unlocked(def.achievement_id))
        .collect()
}

/// Seconds required for a core to fill once and grant 1 PR.
///
/// `fill_duration_secs = 86400 / pr_per_day`
pub fn fill_duration_secs(pr_per_day: u32) -> i64 {
    86400 / pr_per_day as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_power_cores_have_nonzero_rate() {
        for def in ALL_POWER_CORES {
            assert!(
                def.pr_per_day > 0,
                "Power Core '{}' has pr_per_day == 0",
                def.name
            );
        }
    }

    #[test]
    fn all_power_cores_have_nonempty_name() {
        for def in ALL_POWER_CORES {
            assert!(
                !def.name.is_empty(),
                "Power Core for {:?} has empty name",
                def.achievement_id
            );
        }
    }

    #[test]
    fn six_power_cores_defined() {
        assert_eq!(ALL_POWER_CORES.len(), 6);
    }

    #[test]
    fn get_power_core_def_finds_known_id() {
        let def = get_power_core_def(AchievementId::PowerCoreI);
        assert!(def.is_some());
        let def = def.unwrap();
        assert_eq!(def.name, "Red Fault");
        assert_eq!(def.pr_per_day, 2);
        assert_eq!(def.key, "power_core_1");
    }

    #[test]
    fn get_power_core_def_returns_none_for_unrelated_id() {
        let def = get_power_core_def(AchievementId::SlayerI);
        assert!(def.is_none());
    }

    #[test]
    fn fill_duration_secs_correct_for_one_pr_per_day() {
        assert_eq!(fill_duration_secs(1), 86400);
    }

    #[test]
    fn fill_duration_secs_correct_for_six_pr_per_day() {
        assert_eq!(fill_duration_secs(6), 14400); // 4h
    }

    #[test]
    fn all_cores_unique_achievement_ids() {
        let mut seen = std::collections::HashSet::new();
        for def in ALL_POWER_CORES {
            assert!(
                seen.insert(def.achievement_id),
                "Duplicate achievement_id {:?} in ALL_POWER_CORES",
                def.achievement_id
            );
        }
    }

    #[test]
    fn all_cores_unique_keys() {
        let mut seen = std::collections::HashSet::new();
        for def in ALL_POWER_CORES {
            assert!(
                seen.insert(def.key),
                "Duplicate key '{}' in ALL_POWER_CORES",
                def.key
            );
        }
    }

    #[test]
    fn passives_state_default_is_empty() {
        let state = PassivesState::default();
        assert!(state.generators.is_empty());
    }

    #[test]
    fn generator_timer_default_is_zero() {
        let timer = GeneratorTimer::default();
        assert_eq!(timer.last_granted_at, 0);
    }

    #[test]
    fn get_unlocked_cores_returns_empty_with_no_achievements() {
        let achievements = Achievements::default();
        let cores = get_unlocked_cores(&achievements);
        assert!(cores.is_empty());
    }

    #[test]
    fn core_definitions_ascending_pr_rate() {
        for window in ALL_POWER_CORES.windows(2) {
            assert!(
                window[1].pr_per_day >= window[0].pr_per_day,
                "Core '{}' ({} PR/day) is not >= preceding core '{}' ({} PR/day)",
                window[1].name,
                window[1].pr_per_day,
                window[0].name,
                window[0].pr_per_day,
            );
        }
    }
}
