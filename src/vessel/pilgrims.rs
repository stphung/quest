//! Other pilgrims — five authored ships, not a simulation (sub-project 5).
//!
//! Each ship has a name, a silhouette, a one-line character, and a cyclic
//! route script (which roads, on which day phases). Their fates are
//! authored, not simulated: the player's choices don't save or doom them —
//! their story is weather, not consequence, which keeps the authoring
//! bounded. Hailing is once per ship; pilgrim rumors are the only way to
//! hear about roads behind or beside you.

use super::route::RoadId;

#[derive(Debug, Clone, Copy)]
pub struct PilgrimShip {
    pub id: u8,
    pub name: &'static str,
    /// One chart cell.
    pub glyph: char,
    /// Their one-line character (the hail exchange speaks in it).
    pub line: &'static str,
    /// Cyclic script: on day `d`, the ship sails `roads[(d / days_per_leg)
    /// % roads.len()]` — until `dark_after`, when the ship stops appearing.
    pub roads: &'static [RoadId],
    pub days_per_leg: u64,
    /// Authored fate: after this day the ship goes dark (None = sails on;
    /// the Sister Verity stands in the harbor at the Tree for Act 3).
    pub dark_after: Option<u64>,
    /// What they trade when hailed.
    pub hail: &'static str,
}

/// The five. Scripts favor the Drift Roads and the Deep so coincidence
/// with plausible player routes is common but never guaranteed.
pub const PILGRIMS: [PilgrimShip; 5] = [
    PilgrimShip {
        id: 0,
        name: "the Sister Verity",
        glyph: '\u{2600}', // ☀
        line: "A hospice ship out of Saint Elm's, running lamps ahead of the rest.",
        roads: &[RoadId(13), RoadId(14), RoadId(18), RoadId(23), RoadId(25)],
        days_per_leg: 3,
        dark_after: None, // she reaches the Tree; a face for Act 3
        hail: "The Verity's mate trades news like medicine: measured, and \
               exactly what you needed.",
    },
    PilgrimShip {
        id: 1,
        name: "the Grief of Alden",
        glyph: '\u{2604}', // ☄
        line: "A whole village on one hull, sailing on schedule and on grief.",
        roads: &[RoadId(15), RoadId(19), RoadId(20), RoadId(27)],
        days_per_leg: 4,
        dark_after: Some(40), // she goes dark in the middle water
        hail: "Alden's people ask more than they tell, and thank you twice \
               for both.",
    },
    PilgrimShip {
        id: 2,
        name: "the Wager",
        glyph: '\u{263c}', // ☼
        line: "A smuggler's hull full of paying pilgrims. The manifest is honest now. Mostly.",
        roads: &[RoadId(24), RoadId(26), RoadId(21), RoadId(16)],
        days_per_leg: 2,
        dark_after: None,
        hail: "The Wager's captain sells you a rumor and throws in a second \
               free, to establish credit.",
    },
    PilgrimShip {
        id: 3,
        name: "the Psalm",
        glyph: '\u{2669}', // ♩
        line: "They sing the whole way. You hear her before you see her.",
        roads: &[RoadId(2), RoadId(3), RoadId(8), RoadId(12), RoadId(13)],
        days_per_leg: 3,
        dark_after: None,
        hail: "The Psalm's cantor trades verses for news and rounds up in \
               your favor.",
    },
    PilgrimShip {
        id: 4,
        name: "the Held Breath",
        glyph: '\u{25aa}', // ▪
        line: "Runs dark and quiet. Nobody knows her story and she likes it that way.",
        roads: &[RoadId(29), RoadId(31), RoadId(35), RoadId(37), RoadId(30)],
        days_per_leg: 5,
        dark_after: None,
        hail: "A single lantern flash from the Held Breath: coordinates, \
               and nothing else. It is enough.",
    },
];

/// Where a ship sails on `day`, if she still sails.
pub fn pilgrim_road(ship: &PilgrimShip, day: u64) -> Option<RoadId> {
    if ship.dark_after.is_some_and(|d| day > d) {
        return None;
    }
    Some(ship.roads[((day / ship.days_per_leg) % ship.roads.len() as u64) as usize])
}

/// Every ship abroad on `day`, with her road.
pub fn pilgrims_on(day: u64) -> Vec<(&'static PilgrimShip, RoadId)> {
    PILGRIMS
        .iter()
        .filter_map(|s| pilgrim_road(s, day).map(|r| (s, r)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scripts_reference_real_roads_and_cycle() {
        for ship in &PILGRIMS {
            assert!(!ship.roads.is_empty());
            for r in ship.roads {
                assert!((r.0 as usize) < crate::vessel::route::ROADS.len());
            }
            // Deterministic and cyclic.
            assert_eq!(pilgrim_road(ship, 3), pilgrim_road(ship, 3));
        }
    }

    #[test]
    fn the_grief_of_alden_goes_dark_and_the_verity_sails_on() {
        assert!(pilgrim_road(&PILGRIMS[1], 39).is_some());
        assert!(pilgrim_road(&PILGRIMS[1], 41).is_none(), "gone dark");
        assert!(
            pilgrim_road(&PILGRIMS[0], 400).is_some(),
            "the Verity endures"
        );
        assert_eq!(pilgrims_on(41).len(), 4);
    }
}
