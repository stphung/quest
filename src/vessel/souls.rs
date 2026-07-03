//! The Souls — Act 2's authored roster (sub-project 3).
//!
//! Eight people against seven berths. Each soul is a bundle of hooks into
//! systems that already exist: one **affinity** axis (strengthens the
//! matching station and is what nights read — spec 5), **counsel** lines on
//! junction cards, and an **arc** paid for in rest days. The load-bearing
//! triangle is stations <-> arcs <-> wind; everything else on a soul is
//! voice.
//!
//! See `docs/superpowers/specs/2026-07-03-vessel-souls-design.md`.

use super::route::{Chapter, Feature, RoadId, RumorId, WaypointId};
use serde::{Deserialize, Serialize};

/// Berths aboard the Vessel. 3 launch souls + 5 found = 8 asks against 7.
pub const BERTHS: usize = 7;
/// Rest days a ready arc beat costs before it fires.
pub const ARC_BEAT_REST_DAYS: u64 = 2;
/// Hope cost of losing a soul (authored scenes only, spec 4).
#[allow(dead_code)] // Consumed by `mark_lost`, spec 4's loss API.
pub const LOSS_HOPE_COST: u8 = 3;
/// Hope cost of a farewell (stepping ashore to free a berth).
pub const FAREWELL_HOPE_COST: u8 = 1;

/// The three standing posts — one per system the ship runs on:
/// time (Helm), provisions (Tender), nights (Watch, spec 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Station {
    Helm,
    Tender,
    Watch,
}

impl Station {
    #[allow(dead_code)] // Exercised by tests; the Souls panel cycles posts inline.
    pub const ALL: [Station; 3] = [Station::Helm, Station::Tender, Station::Watch];

    pub fn display_name(&self) -> &'static str {
        match self {
            Station::Helm => "Helm",
            Station::Tender => "Tender",
            Station::Watch => "Watch",
        }
    }
}

/// Leg-time multiplier from the Helm post (composed into progress rate).
pub fn helm_time_mult(staffed: bool, affine: bool) -> f64 {
    match (staffed, affine) {
        (false, _) => 1.00,
        (true, false) => 0.96,
        (true, true) => 0.92,
    }
}

/// Leg-provisions multiplier from the Tender post.
pub fn tender_provisions_mult(staffed: bool, affine: bool) -> f64 {
    match (staffed, affine) {
        (false, _) => 1.00,
        (true, false) => 0.95,
        (true, true) => 0.90,
    }
}

/// Hope is the wind — its one mechanical effect. Bands compose into leg
/// time exactly like trim. At ashen (0) the ship is in the Long Silence.
pub fn wind_time_mult(hope: u8, long_silence: bool) -> f64 {
    if long_silence {
        return 1.40;
    }
    match hope {
        8..=u8::MAX => 0.90,
        5..=7 => 1.00,
        3..=4 => 1.10,
        1..=2 => 1.25,
        0 => 1.40,
    }
}

/// The wind as the panel shows it: an arrow, never a multiplier.
pub fn wind_arrow(hope: u8, long_silence: bool) -> &'static str {
    if long_silence || hope == 0 {
        "\u{2193}\u{2193}" // ↓↓ the Long Silence
    } else if hope >= 8 {
        "\u{2191}" // ↑
    } else if hope >= 5 {
        ""
    } else {
        "\u{2193}" // ↓
    }
}

/// Stable soul identifier (index into [`SOULS`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SoulId(pub u8);

/// What makes an arc beat *ready* (it still costs rest days to fire).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArcTrigger {
    /// Ready from the moment the soul is aboard.
    Aboard,
    /// Ready once the voyage has reached this chapter.
    ReachChapter(Chapter),
    /// Ready once any waypoint with this feature has been visited
    /// (visits before boarding count — the place was seen together or not,
    /// the arc does not relitigate the past).
    VisitFeature(Feature),
    /// Ready once this specific waypoint has been visited.
    VisitWaypoint(WaypointId),
}

/// What a fired beat pays. Small, and always into existing systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArcPayout {
    pub hope: u8,
    /// A rumor the soul shares (granted if not already known).
    pub rumor: Option<RumorId>,
}

/// One beat of a soul's personal thread. Three beats + a resolution each.
#[derive(Debug, Clone, Copy)]
pub struct ArcBeat {
    pub trigger: ArcTrigger,
    /// Log-entry title, e.g. "Sefa sings for the drowned".
    pub title: &'static str,
    /// The beat's text, in the soul's voice (spec 4 scenes may extend it).
    pub text: &'static str,
    pub payout: ArcPayout,
}

/// An authored person. The table is the whole cast — no procedural souls.
#[derive(Debug, Clone, Copy)]
pub struct SoulDef {
    pub id: SoulId,
    pub name: &'static str,
    /// The one-line personality that colors log moments and scenes.
    pub voice: &'static str,
    pub origin: &'static str,
    /// One aptitude axis: strengthens the matching station AND is what
    /// nights read (spec 5). `None` means their value is counsel and arc.
    pub affinity: Option<Station>,
    /// Recruit sites, one per branch arm (empty = boards at launch).
    pub sites: &'static [WaypointId],
    /// Three beats + a resolution (the last entry).
    pub arc: &'static [ArcBeat],
    /// Junction-card counsel, authored per road, in their voice.
    pub counsel: &'static [(RoadId, &'static str)],
}

const fn beat(
    trigger: ArcTrigger,
    title: &'static str,
    text: &'static str,
    hope: u8,
    rumor: Option<RumorId>,
) -> ArcBeat {
    ArcBeat {
        trigger,
        title,
        text,
        payout: ArcPayout { hope, rumor },
    }
}

/// The cast: three launch souls, five found along the route.
pub const SOULS: [SoulDef; 8] = [
    SoulDef {
        id: SoulId(0),
        name: "Torvald",
        voice: "I've been lower than dark. It blinks first.",
        origin: "captain of the Deep's guild",
        affinity: Some(Station::Helm),
        sites: &[],
        arc: &[
            beat(
                ArcTrigger::Aboard,
                "Torvald takes the wheel",
                "He holds the wheel like an argument he has already won. \
                 \"Point her anywhere. I've come back from worse.\"",
                1,
                None,
            ),
            beat(
                ArcTrigger::ReachChapter(Chapter::DriftRoads),
                "Torvald names the currents",
                "\"The Deep had rivers too. You learn them or you feed them.\" \
                 He marks three currents on the chart from memory.",
                1,
                None,
            ),
            beat(
                ArcTrigger::ReachChapter(Chapter::StarlessDeep),
                "Torvald goes quiet",
                "The dark outside is the dark he retired from. He stands at \
                 the rail all night and thanks nobody for asking.",
                0,
                Some(RumorId(6)),
            ),
            beat(
                ArcTrigger::ReachChapter(Chapter::RootsOfLight),
                "Torvald, unclenched",
                "In the root-light he finally sits down. \"Huh,\" he says. \
                 \"So there was a bottom to get to after all.\"",
                2,
                None,
            ),
        ],
        counsel: &[
            (RoadId(15), "The narrows are honest. Hungry, but honest."),
            (
                RoadId(29),
                "Long and silent and cheap. That's the Deep's kind of bargain — take it.",
            ),
            (
                RoadId(35),
                "No stars means nothing to steer by but the wheel. I can be the wheel.",
            ),
        ],
    },
    SoulDef {
        id: SoulId(1),
        name: "Eir",
        voice: "A ship is a house that argues with the sea.",
        origin: "warden of the Haven",
        affinity: Some(Station::Tender),
        sites: &[],
        arc: &[
            beat(
                ArcTrigger::Aboard,
                "Eir takes inventory",
                "She counts the hold twice and labels everything, including \
                 the crew. \"You. Morale. Stand there.\"",
                1,
                None,
            ),
            beat(
                ArcTrigger::VisitFeature(Feature::Shipyard),
                "Eir and the shipwrights",
                "She argues with the yard about the hull for an hour and comes \
                 back satisfied. \"They knew their business. I checked.\"",
                1,
                None,
            ),
            beat(
                ArcTrigger::ReachChapter(Chapter::StarlessDeep),
                "Eir rations the warmth",
                "Blankets appear where they are needed before anyone asks. \
                 The Haven trained her to hear a cold room through a wall.",
                1,
                None,
            ),
            beat(
                ArcTrigger::ReachChapter(Chapter::RootsOfLight),
                "Eir plants something",
                "She has been carrying seeds the whole way. Of course she has. \
                 \"A house argues with the sea. A garden just wins.\"",
                2,
                None,
            ),
        ],
        counsel: &[
            (
                RoadId(3),
                "Green water, slow miles. The hold barely notices roads like this.",
            ),
            (
                RoadId(30),
                "The cold eats stores faster than the card says. Trust me on shelves and cold.",
            ),
            (
                RoadId(36),
                "Pricey and gentle. Sometimes the hold buys the crew, not the miles.",
            ),
        ],
    },
    SoulDef {
        id: SoulId(2),
        name: "Runa",
        voice: "Everything worth catching sings first.",
        origin: "the fisher",
        affinity: Some(Station::Watch),
        sites: &[],
        arc: &[
            beat(
                ArcTrigger::Aboard,
                "Runa reads the water",
                "She trails a hand-line off the stern within the hour. \"Not \
                 for the fish,\" she says. \"For the news.\"",
                1,
                None,
            ),
            beat(
                ArcTrigger::VisitFeature(Feature::LanternSite),
                "Runa and the lights",
                "She knows the keepers' knock. Every lantern on this water has \
                 sold her bait or bought her catch.",
                1,
                Some(RumorId(0)),
            ),
            beat(
                ArcTrigger::ReachChapter(Chapter::StarlessDeep),
                "Runa fishes the dark",
                "The line goes down and down and comes back wet with \
                 something that isn't water. She grins. \"Still news.\"",
                1,
                None,
            ),
            beat(
                ArcTrigger::ReachChapter(Chapter::RootsOfLight),
                "Runa's last catch",
                "She catches one glowing root-fish, looks at it a long time, \
                 and lets it go. \"Season's closed,\" she says, content.",
                2,
                None,
            ),
        ],
        counsel: &[
            (
                RoadId(2),
                "Singing water west. Whatever's in those steeples, it likes an audience.",
            ),
            (
                RoadId(23),
                "Follow the whales. Anything that big that's still kind is worth trusting.",
            ),
        ],
    },
    SoulDef {
        id: SoulId(3),
        name: "Maren",
        voice: "A light is a promise somebody has to keep.",
        origin: "the last lightship keeper",
        affinity: Some(Station::Watch),
        sites: &[WaypointId(1)],
        arc: &[
            beat(
                ArcTrigger::Aboard,
                "Maren douses her lamp",
                "She is the last thing aboard; the lightship goes dark behind \
                 her. \"There's nobody left to warn,\" she says. \"Except us.\"",
                1,
                None,
            ),
            beat(
                ArcTrigger::VisitFeature(Feature::LanternSite),
                "Maren lights what she finds",
                "Wherever there is a lantern, she fills and trims and lights \
                 it, whether or not anyone will ever pass again.",
                1,
                None,
            ),
            beat(
                ArcTrigger::ReachChapter(Chapter::StarlessDeep),
                "Maren against the dark",
                "In the starless reach she hangs her old lamp at the bow. \
                 It is nothing against the deep. She keeps it lit anyway.",
                1,
                None,
            ),
            beat(
                ArcTrigger::ReachChapter(Chapter::RootsOfLight),
                "Maren's promise, kept",
                "\"The Tree is lit,\" she says, off-hand, as if reporting a \
                 lighthouse back in service. Then she cries, briefly, on duty.",
                2,
                None,
            ),
        ],
        counsel: &[
            (
                RoadId(9),
                "The reef takes hulls that hurry. Warden or no warden, go slow and lit.",
            ),
            (
                RoadId(24),
                "Smugglers keep their lanes dark. Dark lanes are where promises get broken.",
            ),
        ],
    },
    SoulDef {
        id: SoulId(4),
        name: "Sefa",
        voice: "The dead don't need the song. The living do.",
        origin: "the last cantor of the drowned parishes",
        affinity: None,
        sites: &[WaypointId(3), WaypointId(5)],
        arc: &[
            beat(
                ArcTrigger::Aboard,
                "Sefa boards singing",
                "She comes over the rail mid-verse, as though the ship were \
                 simply the next line of the lament.",
                1,
                None,
            ),
            beat(
                ArcTrigger::VisitWaypoint(WaypointId(9)),
                "Sefa sings for the reef",
                "At the white reef she sings the office for the unburied. \
                 The Warden, whatever it is, keeps its distance all night.",
                1,
                Some(RumorId(2)),
            ),
            beat(
                ArcTrigger::ReachChapter(Chapter::StarlessDeep),
                "Sefa and the silence",
                "\"Silence is just a song that lost its nerve.\" She hums \
                 into the dark until the dark hums back.",
                1,
                None,
            ),
            beat(
                ArcTrigger::ReachChapter(Chapter::RootsOfLight),
                "Sefa's evening office",
                "She teaches the whole crew the office, voice by voice. The \
                 parishes are drowned; the parish is here.",
                2,
                None,
            ),
        ],
        counsel: &[
            (
                RoadId(2),
                "I know that singing. It's grief, not hunger. Grief can be visited.",
            ),
            (
                RoadId(31),
                "Where something sings anyway, go. That is the bravest thing in the dark.",
            ),
        ],
    },
    SoulDef {
        id: SoulId(5),
        name: "Ysolt",
        voice: "Hulls and hearts. Same joinery, mostly.",
        origin: "a mender from Saint Elm's",
        affinity: Some(Station::Tender),
        sites: &[WaypointId(14), WaypointId(16), WaypointId(18)],
        arc: &[
            beat(
                ArcTrigger::Aboard,
                "Ysolt finds the creak",
                "Within a day she has found and fixed a creak the crew had \
                 stopped hearing years ago. The ship sounds younger.",
                1,
                None,
            ),
            beat(
                ArcTrigger::VisitFeature(Feature::RestStop),
                "Ysolt makes the rounds",
                "At rest she mends quietly: a seam, a sleeve, an argument \
                 between two of the crew. Same joinery, mostly.",
                1,
                None,
            ),
            beat(
                ArcTrigger::ReachChapter(Chapter::StarlessDeep),
                "Ysolt shores the hull",
                "\"The deep leans on a hull. You brace it before it asks.\" \
                 She spends two days below and comes up certain.",
                1,
                None,
            ),
            beat(
                ArcTrigger::ReachChapter(Chapter::RootsOfLight),
                "Ysolt signs her work",
                "She carves her maker's mark inside the hull, where only the \
                 ship can see it. \"Done,\" she says. \"She'll outlive us.\"",
                2,
                None,
            ),
        ],
        counsel: &[
            (
                RoadId(14),
                "Saint Elm's takes any ship that limps. Nobody has to limp alone out here.",
            ),
            (
                RoadId(30),
                "Cold cracks what hurry strained. If we go over the shelf, go rested.",
            ),
        ],
    },
    SoulDef {
        id: SoulId(6),
        name: "Cormac",
        voice: "Charts end. Roads don't.",
        origin: "a pilot of the uncharted lanes",
        affinity: Some(Station::Helm),
        sites: &[WaypointId(20), WaypointId(21)],
        arc: &[
            beat(
                ArcTrigger::Aboard,
                "Cormac corrects the chart",
                "He looks at your chart for a long minute, then adds two \
                 hazards and crosses out one island. \"Never existed.\"",
                1,
                None,
            ),
            beat(
                ArcTrigger::ReachChapter(Chapter::StarlessDeep),
                "Cormac flies blind",
                "\"Down here the road is a feeling in the keel.\" He steers \
                 an hour with his eyes shut and gains back half a day.",
                1,
                None,
            ),
            beat(
                ArcTrigger::VisitFeature(Feature::WayStation),
                "Cormac calls in a debt",
                "At the way-station a stranger owes him. The debt is paid in \
                 quiet news of the roads ahead.",
                1,
                Some(RumorId(5)),
            ),
            beat(
                ArcTrigger::ReachChapter(Chapter::RootsOfLight),
                "Cormac, charted",
                "He draws the route you actually sailed, signs it, and pins \
                 it up. \"There. Now it's a road. Now it exists.\"",
                2,
                None,
            ),
        ],
        counsel: &[
            (
                RoadId(24),
                "I know the slip lanes. Expensive is just a toll on people who can't pilot.",
            ),
            (
                RoadId(42),
                "The thorn run is one clean line if your helm is awake. It's the miles I'd buy.",
            ),
        ],
    },
    SoulDef {
        id: SoulId(7),
        name: "Brother Wren",
        voice: "I slept in the deep. It dreams, you know.",
        origin: "woken from the Sleepers' Trench",
        affinity: None,
        sites: &[WaypointId(26), WaypointId(28)],
        arc: &[
            beat(
                ArcTrigger::Aboard,
                "Wren wakes fully",
                "He surfaces from years of sleep like a diver, slow and \
                 careful. \"How long — no. Doesn't matter. We're going up.\"",
                1,
                None,
            ),
            beat(
                ArcTrigger::VisitFeature(Feature::RestStop),
                "Wren learns to rest",
                "He is afraid to sleep and won't say so. At the hearth he \
                 finally lets himself, and wakes up laughing at nothing.",
                1,
                None,
            ),
            beat(
                ArcTrigger::VisitWaypoint(WaypointId(32)),
                "Wren at the gate",
                "At Deepgate he stares into the rift a long time. \"The deep \
                 dreamed this door,\" he says. \"I remember dreaming it too.\"",
                1,
                None,
            ),
            beat(
                ArcTrigger::ReachChapter(Chapter::RootsOfLight),
                "Wren, awake",
                "In the root-light he stops flinching at sleep. \"If I dream \
                 now,\" he says, \"I'll dream of somewhere I've actually been.\"",
                2,
                None,
            ),
        ],
        counsel: &[
            (
                RoadId(35),
                "The straight dark is empty. I slept under it; nothing there but patience.",
            ),
            (
                RoadId(39),
                "The rift is a door the deep dreamed. Doors want going through.",
            ),
        ],
    },
];

/// Soul definition by id. Ids are ordinal, so this is an index.
pub fn soul(id: SoulId) -> &'static SoulDef {
    &SOULS[id.0 as usize]
}

/// The souls that board at launch (empty site lists).
pub fn launch_souls() -> impl Iterator<Item = &'static SoulDef> {
    SOULS.iter().filter(|s| s.sites.is_empty())
}

/// The soul (if any) whose recruit site is this waypoint.
pub fn recruit_at(waypoint: WaypointId) -> Option<&'static SoulDef> {
    SOULS.iter().find(|s| s.sites.contains(&waypoint))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_ordinal_and_the_cast_is_eight() {
        assert_eq!(SOULS.len(), 8, "eight asks against seven berths");
        for (i, s) in SOULS.iter().enumerate() {
            assert_eq!(s.id.0 as usize, i, "soul {} id mismatch", s.name);
        }
    }

    #[test]
    fn three_launch_souls_and_five_recruits() {
        assert_eq!(launch_souls().count(), 3);
        assert_eq!(SOULS.iter().filter(|s| !s.sites.is_empty()).count(), 5);
    }

    #[test]
    fn every_station_has_exactly_two_affine_souls() {
        for station in Station::ALL {
            let affine = SOULS.iter().filter(|s| s.affinity == Some(station)).count();
            assert_eq!(
                affine,
                2,
                "{} should have two affine souls",
                station.display_name()
            );
        }
        // And two souls carry no affinity at all: counsel and arc alone
        // justify a berth.
        assert_eq!(SOULS.iter().filter(|s| s.affinity.is_none()).count(), 2);
    }

    #[test]
    fn every_arc_is_three_beats_and_a_resolution() {
        for s in &SOULS {
            assert_eq!(
                s.arc.len(),
                4,
                "{}'s arc should be 3 beats + resolution",
                s.name
            );
            assert_eq!(
                s.arc[0].trigger,
                ArcTrigger::Aboard,
                "{}'s first beat should fire from boarding",
                s.name
            );
            assert_eq!(
                s.arc[3].trigger,
                ArcTrigger::ReachChapter(Chapter::RootsOfLight),
                "{}'s resolution should land in Chapter IV",
                s.name
            );
            assert_eq!(s.arc[3].payout.hope, 2, "resolutions pay hope +2");
        }
    }

    #[test]
    fn recruit_sites_are_soul_candidate_waypoints_and_unique() {
        use crate::vessel::route::{waypoint, Feature};
        let mut seen = std::collections::HashSet::new();
        for s in &SOULS {
            for site in s.sites {
                assert!(
                    waypoint(*site).has_feature(Feature::SoulCandidate),
                    "{}'s site {:?} is not a SoulCandidate waypoint",
                    s.name,
                    site
                );
                assert!(seen.insert(*site), "site {site:?} claimed twice");
            }
        }
    }

    #[test]
    fn counsel_references_real_roads() {
        for s in &SOULS {
            for (road, line) in s.counsel {
                assert!((road.0 as usize) < crate::vessel::route::ROADS.len());
                assert!(!line.is_empty());
            }
        }
    }

    #[test]
    fn wind_bands_cover_the_scale() {
        assert_eq!(wind_time_mult(10, false), 0.90);
        assert_eq!(wind_time_mult(7, false), 1.00);
        assert_eq!(wind_time_mult(4, false), 1.10);
        assert_eq!(wind_time_mult(2, false), 1.25);
        assert_eq!(wind_time_mult(0, false), 1.40);
        assert_eq!(wind_time_mult(9, true), 1.40, "silence overrides the band");
        assert_eq!(wind_arrow(9, false), "\u{2191}");
        assert_eq!(wind_arrow(6, false), "");
        assert_eq!(wind_arrow(3, false), "\u{2193}");
        assert_eq!(wind_arrow(5, true), "\u{2193}\u{2193}");
    }
}
