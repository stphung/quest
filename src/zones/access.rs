//! Account-level zone access synchronization.

use super::progression::ZoneProgression;
use crate::core::constants::EXPANSE_ZONE_ID;

/// Synchronize zone unlocks from account-level state.
///
/// Called at: character load, prestige reset, StormsEnd, fracture region unlock,
/// Loom pattern completion.
///
/// - If `storms_end_unlocked` and prestige >= 25, unlocks Zone 11
/// - Unlocks every zone in `12..=fracture_zone_cap` whose prestige requirement is met
/// - Unlocks every zone in `31..=loom_zone_cap` whose prestige requirement is met
/// - Never unlocks above cap, never removes earlier unlocks
pub fn sync_account_zone_unlocks(
    prog: &mut ZoneProgression,
    storms_end_unlocked: bool,
    fracture_zone_cap: u32,
    prestige_rank: u32,
    loom_zone_cap: u32,
    ascension_level: u32,
) {
    if storms_end_unlocked {
        if let Some(zone) = crate::zones::data::get_zone(EXPANSE_ZONE_ID) {
            if prestige_rank >= zone.prestige_requirement {
                prog.unlock_zone(EXPANSE_ZONE_ID);
            }
        }
    }
    let zones = crate::zones::data::get_all_zones();
    for zone_id in 12..=fracture_zone_cap {
        if let Some(zone) = zones.iter().find(|z| z.id == zone_id) {
            if prestige_rank >= zone.prestige_requirement {
                prog.unlock_zone(zone_id);
            }
        }
    }
    // Loom zones 31..=loom_zone_cap
    for zone_id in 31..=loom_zone_cap {
        // Determine required ascension level for this zone
        let required_ascension = match zone_id {
            31..=34 => 0,  // No ascension required for Ch.7
            35..=38 => 7,  // Ascension VII
            39..=42 => 8,  // Ascension VIII
            43..=46 => 9,  // Ascension IX
            47..=50 => 10, // Ascension X
            _ => 0,
        };
        if ascension_level < required_ascension {
            continue;
        }
        if let Some(zone) = zones.iter().find(|z| z.id == zone_id) {
            if prestige_rank >= zone.prestige_requirement {
                prog.unlock_zone(zone_id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_unlocks_zone_11_when_storms_end() {
        let mut prog = ZoneProgression::new();
        sync_account_zone_unlocks(&mut prog, true, 11, 25, 30, 0);
        assert!(prog.is_zone_unlocked(11));
    }

    #[test]
    fn test_sync_does_not_unlock_zone_11_without_storms_end() {
        let mut prog = ZoneProgression::new();
        sync_account_zone_unlocks(&mut prog, false, 11, 25, 30, 0);
        assert!(!prog.is_zone_unlocked(11));
    }

    #[test]
    fn test_sync_does_not_unlock_zone_11_without_prestige() {
        let mut prog = ZoneProgression::new();
        sync_account_zone_unlocks(&mut prog, true, 11, 10, 30, 0);
        assert!(!prog.is_zone_unlocked(11));
    }

    #[test]
    fn test_sync_unlocks_zones_12_through_14_when_cap_14() {
        let mut prog = ZoneProgression::new();
        sync_account_zone_unlocks(&mut prog, true, 14, 50, 30, 0);
        assert!(prog.is_zone_unlocked(11));
        assert!(prog.is_zone_unlocked(12));
        assert!(prog.is_zone_unlocked(13));
        assert!(prog.is_zone_unlocked(14));
        assert!(!prog.is_zone_unlocked(15));
    }

    #[test]
    fn test_sync_unlocks_all_fracture_when_cap_20() {
        let mut prog = ZoneProgression::new();
        sync_account_zone_unlocks(&mut prog, true, 20, 300, 30, 0);
        for z in 11..=20 {
            assert!(prog.is_zone_unlocked(z), "Zone {} should be unlocked", z);
        }
        assert!(!prog.is_zone_unlocked(21));
    }

    #[test]
    fn test_sync_respects_prestige_for_fracture_zones() {
        let mut prog = ZoneProgression::new();
        // Cap is 20, but prestige only 50 — should unlock 11-14 (P25/P50) but not 15+ (P75)
        sync_account_zone_unlocks(&mut prog, true, 20, 50, 30, 0);
        assert!(prog.is_zone_unlocked(11));
        assert!(prog.is_zone_unlocked(12));
        assert!(prog.is_zone_unlocked(14));
        assert!(!prog.is_zone_unlocked(15));
    }

    #[test]
    fn test_sync_never_removes_earlier_unlocks() {
        let mut prog = ZoneProgression::new();
        prog.unlock_zone(12);
        sync_account_zone_unlocks(&mut prog, true, 11, 25, 30, 0);
        // Zone 12 was manually unlocked, sync should not remove it
        assert!(prog.is_zone_unlocked(12));
    }

    #[test]
    fn test_sync_idempotent() {
        let mut prog = ZoneProgression::new();
        sync_account_zone_unlocks(&mut prog, true, 14, 50, 30, 0);
        sync_account_zone_unlocks(&mut prog, true, 14, 50, 30, 0); // call twice
        assert!(prog.is_zone_unlocked(14));
    }

    #[test]
    fn test_sync_unlocks_loom_zones_31_to_34() {
        let mut prog = ZoneProgression::new();
        sync_account_zone_unlocks(&mut prog, true, 30, 2000, 34, 0);
        assert!(prog.is_zone_unlocked(31));
        assert!(prog.is_zone_unlocked(34));
        assert!(!prog.is_zone_unlocked(35));
    }

    #[test]
    fn test_sync_does_not_unlock_loom_zones_beyond_cap() {
        let mut prog = ZoneProgression::new();
        sync_account_zone_unlocks(&mut prog, true, 30, 50000, 34, 10);
        assert!(prog.is_zone_unlocked(34));
        assert!(!prog.is_zone_unlocked(35));
    }

    #[test]
    fn test_sync_loom_zones_respects_prestige() {
        let mut prog = ZoneProgression::new();
        // Cap is 50, but prestige only 2000 — should unlock Z31-34 (P2000) but not Z35+ (P5000)
        sync_account_zone_unlocks(&mut prog, true, 30, 2000, 50, 10);
        assert!(prog.is_zone_unlocked(34));
        assert!(!prog.is_zone_unlocked(35));
    }

    #[test]
    fn test_loom_zone_ascension_gate() {
        let mut prog = ZoneProgression::new();
        // With high prestige but low ascension, Z35+ should not unlock
        sync_account_zone_unlocks(&mut prog, true, 30, 50000, 50, 6); // Asc VI
        assert!(prog.is_zone_unlocked(34)); // Ch.7 unlocked (no asc requirement)
        assert!(!prog.is_zone_unlocked(35)); // Ch.8 NOT unlocked (needs Asc VII)

        // With Asc VII, Z35-38 should unlock
        sync_account_zone_unlocks(&mut prog, true, 30, 50000, 50, 7);
        assert!(prog.is_zone_unlocked(38)); // Ch.8 unlocked
        assert!(!prog.is_zone_unlocked(39)); // Ch.9 NOT unlocked (needs Asc VIII)
    }
}
