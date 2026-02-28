//! Account-level zone access synchronization.

use super::progression::ZoneProgression;
use crate::core::constants::EXPANSE_ZONE_ID;

/// Synchronize zone unlocks from account-level state.
///
/// Called at: character load, prestige reset, StormsEnd, fracture region unlock.
///
/// - If `storms_end_unlocked`, unlocks Zone 11
/// - Unlocks every zone in `12..=fracture_zone_cap`
/// - Never unlocks above cap, never removes earlier unlocks
pub fn sync_account_zone_unlocks(
    prog: &mut ZoneProgression,
    storms_end_unlocked: bool,
    fracture_zone_cap: u32,
) {
    if storms_end_unlocked {
        prog.unlock_zone(EXPANSE_ZONE_ID);
    }
    for zone_id in 12..=fracture_zone_cap {
        prog.unlock_zone(zone_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_unlocks_zone_11_when_storms_end() {
        let mut prog = ZoneProgression::new();
        sync_account_zone_unlocks(&mut prog, true, 11);
        assert!(prog.is_zone_unlocked(11));
    }

    #[test]
    fn test_sync_does_not_unlock_zone_11_without_storms_end() {
        let mut prog = ZoneProgression::new();
        sync_account_zone_unlocks(&mut prog, false, 11);
        assert!(!prog.is_zone_unlocked(11));
    }

    #[test]
    fn test_sync_unlocks_zones_12_through_14_when_cap_14() {
        let mut prog = ZoneProgression::new();
        sync_account_zone_unlocks(&mut prog, true, 14);
        assert!(prog.is_zone_unlocked(11));
        assert!(prog.is_zone_unlocked(12));
        assert!(prog.is_zone_unlocked(13));
        assert!(prog.is_zone_unlocked(14));
        assert!(!prog.is_zone_unlocked(15));
    }

    #[test]
    fn test_sync_unlocks_all_fracture_when_cap_20() {
        let mut prog = ZoneProgression::new();
        sync_account_zone_unlocks(&mut prog, true, 20);
        for z in 11..=20 {
            assert!(prog.is_zone_unlocked(z), "Zone {} should be unlocked", z);
        }
        assert!(!prog.is_zone_unlocked(21));
    }

    #[test]
    fn test_sync_never_removes_earlier_unlocks() {
        let mut prog = ZoneProgression::new();
        prog.unlock_zone(12);
        sync_account_zone_unlocks(&mut prog, true, 11);
        // Zone 12 was manually unlocked, sync should not remove it
        assert!(prog.is_zone_unlocked(12));
    }

    #[test]
    fn test_sync_idempotent() {
        let mut prog = ZoneProgression::new();
        sync_account_zone_unlocks(&mut prog, true, 14);
        sync_account_zone_unlocks(&mut prog, true, 14); // call twice
        assert!(prog.is_zone_unlocked(14));
    }
}
