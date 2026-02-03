//! Zone progression state and logic.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use super::data::{get_all_zones, Zone};

/// Tracks the player's progression through zones and subzones.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneProgression {
    /// Current zone ID (1-10)
    pub current_zone_id: u32,
    /// Current subzone ID within the zone
    pub current_subzone_id: u32,
    /// List of defeated bosses as (zone_id, subzone_id) pairs
    pub defeated_bosses: Vec<(u32, u32)>,
    /// List of unlocked zone IDs
    pub unlocked_zones: Vec<u32>,
}

impl Default for ZoneProgression {
    fn default() -> Self {
        Self::new()
    }
}

impl ZoneProgression {
    /// Creates a new zone progression starting in Zone 1, Subzone 1.
    pub fn new() -> Self {
        Self {
            current_zone_id: 1,
            current_subzone_id: 1,
            defeated_bosses: vec![],
            unlocked_zones: vec![1, 2], // Start with zones 1-2 unlocked (P0 zones)
        }
    }

    /// Checks if a boss has been defeated.
    pub fn is_boss_defeated(&self, zone_id: u32, subzone_id: u32) -> bool {
        self.defeated_bosses.contains(&(zone_id, subzone_id))
    }

    /// Checks if a zone is unlocked.
    pub fn is_zone_unlocked(&self, zone_id: u32) -> bool {
        self.unlocked_zones.contains(&zone_id)
    }

    /// Checks if a zone can be unlocked based on prestige rank.
    pub fn can_unlock_zone(&self, zone: &Zone, prestige_rank: u32) -> bool {
        // Check prestige requirement
        if prestige_rank < zone.prestige_requirement {
            return false;
        }

        // Check if previous zone's final boss is defeated (if not first zone)
        if zone.id > 1 {
            let prev_zone_id = zone.id - 1;
            if let Some(prev_zone) = get_all_zones().into_iter().find(|z| z.id == prev_zone_id) {
                let last_subzone_id = prev_zone.subzones.len() as u32;
                if !self.is_boss_defeated(prev_zone_id, last_subzone_id) {
                    return false;
                }
            }
        }

        true
    }

    /// Unlocks a zone.
    pub fn unlock_zone(&mut self, zone_id: u32) {
        if !self.unlocked_zones.contains(&zone_id) {
            self.unlocked_zones.push(zone_id);
            self.unlocked_zones.sort();
        }
    }

    /// Records a boss defeat.
    pub fn defeat_boss(&mut self, zone_id: u32, subzone_id: u32) {
        if !self.is_boss_defeated(zone_id, subzone_id) {
            self.defeated_bosses.push((zone_id, subzone_id));
        }
    }

    /// Checks if the player can enter a specific subzone.
    pub fn can_enter_subzone(&self, zone_id: u32, subzone_id: u32) -> bool {
        // Zone must be unlocked
        if !self.is_zone_unlocked(zone_id) {
            return false;
        }

        // First subzone is always accessible if zone is unlocked
        if subzone_id == 1 {
            return true;
        }

        // Need previous subzone's boss defeated
        self.is_boss_defeated(zone_id, subzone_id - 1)
    }

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

    /// Gets the current zone and subzone names.
    pub fn current_location_names(&self) -> (String, String) {
        let zones = get_all_zones();
        if let Some(zone) = zones.iter().find(|z| z.id == self.current_zone_id) {
            if let Some(subzone) = zone
                .subzones
                .iter()
                .find(|s| s.id == self.current_subzone_id)
            {
                return (zone.name.to_string(), subzone.name.to_string());
            }
        }
        ("Unknown".to_string(), "Unknown".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zone_progression_default() {
        let prog = ZoneProgression::new();
        assert_eq!(prog.current_zone_id, 1);
        assert_eq!(prog.current_subzone_id, 1);
        assert!(prog.is_zone_unlocked(1));
        assert!(prog.is_zone_unlocked(2));
        assert!(!prog.is_zone_unlocked(3));
    }

    #[test]
    fn test_boss_defeat_tracking() {
        let mut prog = ZoneProgression::new();

        assert!(!prog.is_boss_defeated(1, 1));
        prog.defeat_boss(1, 1);
        assert!(prog.is_boss_defeated(1, 1));

        // Defeating same boss again should not duplicate
        prog.defeat_boss(1, 1);
        assert_eq!(
            prog.defeated_bosses
                .iter()
                .filter(|&&b| b == (1, 1))
                .count(),
            1
        );
    }

    #[test]
    fn test_subzone_access() {
        let mut prog = ZoneProgression::new();

        // Can enter first subzone
        assert!(prog.can_enter_subzone(1, 1));

        // Cannot enter second subzone without defeating first boss
        assert!(!prog.can_enter_subzone(1, 2));

        // Defeat first boss
        prog.defeat_boss(1, 1);

        // Now can enter second subzone
        assert!(prog.can_enter_subzone(1, 2));
    }

    #[test]
    fn test_zone_unlock_prestige_gate() {
        let prog = ZoneProgression::new();
        let zones = get_all_zones();

        // Zone 3 requires prestige 5
        assert!(!prog.can_unlock_zone(&zones[2], 0));
        assert!(!prog.can_unlock_zone(&zones[2], 4));
        // Note: Also needs zone 2's boss defeated
    }

    #[test]
    fn test_zone_unlock_boss_gate() {
        let mut prog = ZoneProgression::new();
        let zones = get_all_zones();

        // Zone 3 requires P5 AND zone 2's final boss defeated
        // With P5 but no boss defeated
        assert!(!prog.can_unlock_zone(&zones[2], 5));

        // Defeat zone 2's bosses
        prog.defeat_boss(2, 1);
        prog.defeat_boss(2, 2);
        prog.defeat_boss(2, 3);

        // Now should be able to unlock
        assert!(prog.can_unlock_zone(&zones[2], 5));
    }

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
    fn test_current_location_names() {
        let prog = ZoneProgression::new();
        let (zone, subzone) = prog.current_location_names();
        assert_eq!(zone, "Meadow");
        assert_eq!(subzone, "Sunny Fields");
    }
}
