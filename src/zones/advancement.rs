//! Zone and subzone advancement and travel logic.

#![allow(dead_code)]

use super::data::get_all_zones;
use super::progression::ZoneProgression;

impl ZoneProgression {
    /// Advances to the next subzone within the current zone.
    /// Returns true if successful.
    pub fn advance_to_next_subzone(&mut self) -> bool {
        let zones = get_all_zones();
        let zone = zones.iter().find(|z| z.id == self.current_zone_id);

        if let Some(zone) = zone {
            let max_subzone = zone.subzones.len() as u32;
            if self.current_subzone_id < max_subzone {
                // Check if current boss is defeated
                if self.is_boss_defeated(self.current_zone_id, self.current_subzone_id) {
                    self.current_subzone_id += 1;
                    return true;
                }
            }
        }
        false
    }

    /// Advances to the next zone.
    /// Returns true if successful.
    pub fn advance_to_next_zone(&mut self, prestige_rank: u32) -> bool {
        let zones = get_all_zones();
        let next_zone_id = self.current_zone_id + 1;

        if let Some(next_zone) = zones.iter().find(|z| z.id == next_zone_id) {
            if self.can_unlock_zone(next_zone, prestige_rank) {
                self.unlock_zone(next_zone_id);
                self.current_zone_id = next_zone_id;
                self.current_subzone_id = 1;
                return true;
            }
        }
        false
    }

    /// Sets the current zone and subzone directly.
    /// Used when traveling to previously unlocked areas.
    pub fn travel_to(&mut self, zone_id: u32, subzone_id: u32) -> bool {
        if self.can_enter_subzone(zone_id, subzone_id) {
            self.current_zone_id = zone_id;
            self.current_subzone_id = subzone_id;
            return true;
        }
        false
    }

    /// Resets progression for a new prestige cycle.
    /// Keeps zones unlocked based on the new prestige rank.
    pub fn reset_for_prestige(&mut self, new_prestige_rank: u32) {
        // Reset position to start
        self.current_zone_id = 1;
        self.current_subzone_id = 1;

        // Reset kill tracking
        self.kills_in_subzone = 0;
        self.fighting_boss = false;

        // Clear defeated bosses
        self.defeated_bosses.clear();

        // Recalculate unlocked zones based on new prestige rank
        let zones = get_all_zones();
        self.unlocked_zones = zones
            .iter()
            .filter(|z| z.prestige_requirement <= new_prestige_rank)
            .map(|z| z.id)
            .collect();
        self.unlocked_zones.sort();
    }
}

#[cfg(test)]
mod tests {
    use crate::core::constants::EXPANSE_ZONE_ID;
    use crate::zones::ZoneProgression;

    #[test]
    fn test_advance_subzone() {
        let mut prog = ZoneProgression::new();

        // Cannot advance without defeating boss
        assert!(!prog.advance_to_next_subzone());
        assert_eq!(prog.current_subzone_id, 1);

        // Defeat boss and advance
        prog.defeat_boss(1, 1);
        assert!(prog.advance_to_next_subzone());
        assert_eq!(prog.current_subzone_id, 2);
    }

    #[test]
    fn test_advance_zone() {
        let mut prog = ZoneProgression::new();

        // Defeat all zone 1 bosses
        prog.defeat_boss(1, 1);
        prog.defeat_boss(1, 2);
        prog.defeat_boss(1, 3);

        // Should advance to zone 2 (P0 requirement met)
        assert!(prog.advance_to_next_zone(0));
        assert_eq!(prog.current_zone_id, 2);
        assert_eq!(prog.current_subzone_id, 1);
    }

    #[test]
    fn test_advance_zone_prestige_blocked() {
        let mut prog = ZoneProgression::new();

        // Progress through zones 1 and 2
        for subzone in 1..=3 {
            prog.defeat_boss(1, subzone);
        }
        prog.advance_to_next_zone(0);

        for subzone in 1..=3 {
            prog.defeat_boss(2, subzone);
        }

        // Try to advance to zone 3 with P0 (needs P5)
        assert!(!prog.advance_to_next_zone(0));
        assert_eq!(prog.current_zone_id, 2);

        // With P5, should work
        assert!(prog.advance_to_next_zone(5));
        assert_eq!(prog.current_zone_id, 3);
    }

    #[test]
    fn test_travel_to() {
        let mut prog = ZoneProgression::new();

        // Can travel to unlocked zone's first subzone
        assert!(prog.travel_to(2, 1));
        assert_eq!(prog.current_zone_id, 2);
        assert_eq!(prog.current_subzone_id, 1);

        // Cannot travel to locked zone
        assert!(!prog.travel_to(3, 1));

        // Cannot travel to locked subzone
        assert!(!prog.travel_to(2, 2));

        // Defeat boss, then can travel
        prog.defeat_boss(2, 1);
        assert!(prog.travel_to(2, 2));
    }

    #[test]
    fn test_reset_for_prestige() {
        let mut prog = ZoneProgression::new();

        // Make some progress
        prog.current_zone_id = 3;
        prog.current_subzone_id = 2;
        prog.defeat_boss(1, 1);
        prog.defeat_boss(1, 2);
        prog.unlock_zone(3);

        // Reset with P5
        prog.reset_for_prestige(5);

        // Should be back at start
        assert_eq!(prog.current_zone_id, 1);
        assert_eq!(prog.current_subzone_id, 1);
        assert!(prog.defeated_bosses.is_empty());

        // Should have zones 1-4 unlocked (P0 and P5 zones)
        assert!(prog.is_zone_unlocked(1));
        assert!(prog.is_zone_unlocked(2));
        assert!(prog.is_zone_unlocked(3));
        assert!(prog.is_zone_unlocked(4));
        assert!(!prog.is_zone_unlocked(5)); // Needs P10
    }

    #[test]
    fn test_reset_for_prestige_clears_kill_tracking() {
        let mut prog = ZoneProgression::new();

        // Accumulate some kills and trigger boss
        for _ in 0..crate::core::constants::KILLS_FOR_BOSS {
            prog.record_kill();
        }
        assert!(prog.fighting_boss);

        // Prestige reset
        prog.reset_for_prestige(1);

        assert_eq!(prog.kills_in_subzone, 0);
        assert!(!prog.fighting_boss);
        assert_eq!(prog.current_zone_id, 1);
        assert_eq!(prog.current_subzone_id, 1);
        assert!(prog.defeated_bosses.is_empty());
    }

    #[test]
    fn test_prestige_reset_during_zone_11() {
        let mut prog = ZoneProgression::new();

        prog.current_zone_id = EXPANSE_ZONE_ID;
        prog.current_subzone_id = 3;
        prog.unlock_zone(EXPANSE_ZONE_ID);
        prog.kills_in_subzone = 5;
        prog.fighting_boss = false;

        prog.reset_for_prestige(20);

        assert_eq!(prog.current_zone_id, 1);
        assert_eq!(prog.current_subzone_id, 1);
        assert_eq!(prog.kills_in_subzone, 0);
        assert!(prog.defeated_bosses.is_empty());
    }

    #[test]
    fn test_reset_for_prestige_p0_only_zones_1_2() {
        let mut prog = ZoneProgression::new();
        prog.current_zone_id = 3;
        prog.defeat_boss(1, 1);
        prog.unlock_zone(3);

        prog.reset_for_prestige(0);

        assert!(prog.is_zone_unlocked(1));
        assert!(prog.is_zone_unlocked(2));
        assert!(!prog.is_zone_unlocked(3));
        assert!(!prog.is_zone_unlocked(5));
    }

    #[test]
    fn test_reset_for_prestige_high_p25() {
        let mut prog = ZoneProgression::new();
        prog.reset_for_prestige(25);

        for zone_id in 1..=10 {
            assert!(
                prog.is_zone_unlocked(zone_id),
                "Zone {zone_id} should be unlocked at P25"
            );
        }
        assert!(prog.is_zone_unlocked(EXPANSE_ZONE_ID));
    }

    #[test]
    fn test_reset_always_preserves_zone_1() {
        let mut prog = ZoneProgression::new();

        for prestige in [0, 1, 5, 10, 20, 50] {
            prog.reset_for_prestige(prestige);
            assert!(
                prog.is_zone_unlocked(1),
                "Zone 1 must always be unlocked at P{prestige}"
            );
            assert_eq!(prog.current_zone_id, 1);
            assert_eq!(prog.current_subzone_id, 1);
        }
    }

    #[test]
    fn test_reset_from_mid_zone_clears_progress() {
        let mut prog = ZoneProgression::new();

        prog.current_zone_id = 5;
        prog.current_subzone_id = 3;
        prog.kills_in_subzone = 7;
        prog.fighting_boss = true;
        prog.defeat_boss(5, 1);
        prog.defeat_boss(5, 2);

        prog.reset_for_prestige(10);

        assert_eq!(prog.current_zone_id, 1);
        assert_eq!(prog.current_subzone_id, 1);
        assert_eq!(prog.kills_in_subzone, 0);
        assert!(!prog.fighting_boss);
        assert!(prog.defeated_bosses.is_empty());
    }

    #[test]
    fn test_cant_advance_past_last_subzone_without_boss_defeat() {
        let mut prog = ZoneProgression::new();

        // Zone 1 has 3 subzones. Set to subzone 3.
        prog.current_subzone_id = 3;

        // Without defeating the boss, can't advance
        let advanced = prog.advance_to_next_subzone();
        assert!(!advanced);
        assert_eq!(prog.current_subzone_id, 3);
    }

    #[test]
    fn test_travel_to_invalid_subzone() {
        let mut prog = ZoneProgression::new();

        let result = prog.travel_to(1, 5);
        assert!(!result);
    }

    #[test]
    fn test_travel_to_locked_zone() {
        let mut prog = ZoneProgression::new();

        let result = prog.travel_to(5, 1);
        assert!(!result);
    }

    #[test]
    fn test_advance_to_next_zone_beyond_last_zone() {
        let mut prog = ZoneProgression::new();

        // Set to zone 11 (The Expanse), which is the last zone
        prog.current_zone_id = crate::core::constants::EXPANSE_ZONE_ID;
        prog.unlock_zone(crate::core::constants::EXPANSE_ZONE_ID);

        // There is no zone 12, so advance should fail
        assert!(!prog.advance_to_next_zone(50));
        assert_eq!(
            prog.current_zone_id,
            crate::core::constants::EXPANSE_ZONE_ID
        );
    }

    #[test]
    fn test_advance_subzone_at_max_subzone() {
        let mut prog = ZoneProgression::new();

        // Zone 1 has 3 subzones; set to the last one
        prog.current_subzone_id = 3;
        prog.defeat_boss(1, 3);

        // Can't advance past the last subzone
        assert!(!prog.advance_to_next_subzone());
        assert_eq!(prog.current_subzone_id, 3);
    }

    #[test]
    fn test_travel_to_current_location() {
        let mut prog = ZoneProgression::new();

        // Can travel to where we already are
        assert!(prog.travel_to(1, 1));
        assert_eq!(prog.current_zone_id, 1);
        assert_eq!(prog.current_subzone_id, 1);
    }
}
