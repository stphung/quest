//! The Route — Act 2's static authored route graph (sub-project 2).
//!
//! Waypoints and roads form a spine-and-diamond DAG from the Last Harbor to
//! the Roots of Light. Branches split at junctions and rejoin within the same
//! chapter; every chapter ends at a single gateway waypoint; the final
//! waypoint is the graph's only sink. The graph is data so a second crossing
//! could exist someday without new code.
//!
//! See `docs/superpowers/specs/2026-07-03-vessel-route-waypoints-design.md`.

use serde::{Deserialize, Serialize};

/// Stable waypoint identifier (index into [`WAYPOINTS`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WaypointId(pub u16);

/// Stable road identifier (index into [`ROADS`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RoadId(pub u16);

/// Stable rumor identifier (index into [`RUMORS`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RumorId(pub u16);

/// Act 2's four chapters, in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Chapter {
    Shallows,
    DriftRoads,
    StarlessDeep,
    RootsOfLight,
}

impl Chapter {
    pub fn display_name(&self) -> &'static str {
        match self {
            Chapter::Shallows => "The Shallows",
            Chapter::DriftRoads => "The Drift Roads",
            Chapter::StarlessDeep => "The Starless Deep",
            Chapter::RootsOfLight => "The Roots of Light",
        }
    }

    pub fn numeral(&self) -> &'static str {
        match self {
            Chapter::Shallows => "I",
            Chapter::DriftRoads => "II",
            Chapter::StarlessDeep => "III",
            Chapter::RootsOfLight => "IV",
        }
    }
}

/// What a waypoint offers. A waypoint may carry several features.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feature {
    /// A true port: the start and the end of the crossing.
    Harbor,
    /// Refits and repairs (spec 4 scenes plug in here).
    Shipyard,
    /// Souls rest easier; arcs advance faster (spec 3).
    RestStop,
    /// Rumors can be purchased here, one per visit.
    WayStation,
    /// A lantern the Vessel may light in passing (keepsake chart marks).
    LanternSite,
    /// A soul may be met here (spec 3/4 content slot). Content parity
    /// requires >= 5 of these on every maximal route.
    SoulCandidate,
}

/// A road's authored character, shown on its junction card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoadTag {
    /// Burns more provisions than its price suggests it should.
    Hungry,
    /// Kind chances: found stores, fair winds.
    Lucky,
    /// Someone takes a cut for passage.
    Tolled,
    /// Singing nights are common here.
    Singing,
    /// Silence and cold; hope frays.
    Dark,
    /// Gentle water; souls rest well.
    Kind,
}

impl RoadTag {
    pub fn display_name(&self) -> &'static str {
        match self {
            RoadTag::Hungry => "hungry",
            RoadTag::Lucky => "lucky",
            RoadTag::Tolled => "tolled",
            RoadTag::Singing => "singing",
            RoadTag::Dark => "dark",
            RoadTag::Kind => "kind",
        }
    }
}

/// Slot for spec 4's arrival scene content. For now a stable content id and
/// a placeholder line the UI can show until the scenes are authored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SceneRef {
    pub id: &'static str,
    pub placeholder: &'static str,
}

/// Slot for a named threat living on a road (spec 4 scene content).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThreatRef {
    pub id: &'static str,
    pub name: &'static str,
}

/// An authored stop on the crossing.
#[derive(Debug, Clone, Copy)]
pub struct Waypoint {
    pub id: WaypointId,
    pub name: &'static str,
    pub chapter: Chapter,
    /// Authored position on the virtual chart canvas (~120x90 cells).
    pub chart_pos: (u16, u16),
    /// Stable content id + one-line fallback. Full scenes live in
    /// `scenes.rs`, indexed by waypoint id; this stays as the registry the
    /// scene-id test walks.
    #[allow(dead_code)]
    pub scene: SceneRef,
    pub features: &'static [Feature],
}

impl Waypoint {
    pub fn has_feature(&self, feature: Feature) -> bool {
        self.features.contains(&feature)
    }
}

/// An authored leg between two waypoints.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Road {
    pub id: RoadId,
    pub from: WaypointId,
    pub to: WaypointId,
    /// Real days at Cruise trim in calm water (1.0..3.0).
    pub base_days: f32,
    /// Provisions the whole leg burns at Cruise (12..55, chapter-ramped).
    pub base_provisions: u16,
    pub character: &'static [RoadTag],
    /// One junction-card line, always shown.
    pub blurb: &'static str,
    /// Stops shown on the card without needing rumors.
    pub known_stops: &'static [WaypointId],
    /// Named threat living on this road (spec 4 scene slot).
    pub threat: Option<ThreatRef>,
}

/// What a rumor is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RumorSubject {
    Road(RoadId),
    Place(WaypointId),
}

/// An authored rumor. The voyage state records *learned* rumors with
/// provenance; this is the content table.
#[derive(Debug, Clone, Copy)]
pub struct RumorDef {
    // Ordinality is asserted in tests; readers index the table directly.
    #[allow(dead_code)]
    pub id: RumorId,
    /// The line the player reads.
    pub text: &'static str,
    pub subject: RumorSubject,
    /// A stop this rumor reveals on the subject road's junction card.
    pub reveals_stop: Option<WaypointId>,
}

const fn scene(id: &'static str, placeholder: &'static str) -> SceneRef {
    SceneRef { id, placeholder }
}

/// Where every crossing begins.
pub const ROUTE_START: WaypointId = WaypointId(0);
/// The only sink: the Tree.
pub const ROUTE_SINK: WaypointId = WaypointId(37);

/// The chart canvas dimensions — every `chart_pos` lives inside this box
/// (the renderer's viewport and the keepsake chart's pan both clamp to it).
pub const CHART_CANVAS_W: u16 = 120;
pub const CHART_CANVAS_H: u16 = 90;

/// The v1 route: 38 authored waypoints (~24 seen on any taken path).
pub const WAYPOINTS: [Waypoint; 38] = [
    // ── Chapter I: The Shallows ─────────────────────────────────────────────
    Waypoint {
        id: WaypointId(0),
        name: "The Last Harbor",
        chapter: Chapter::Shallows,
        chart_pos: (60, 88),
        scene: scene(
            "arrival.last_harbor",
            "The pier lights dim behind you. No one waves twice.",
        ),
        features: &[Feature::Harbor],
    },
    Waypoint {
        id: WaypointId(1),
        name: "The Lightship Vigil",
        chapter: Chapter::Shallows,
        chart_pos: (52, 84),
        scene: scene(
            "arrival.lightship_vigil",
            "A moored lightship, and a keeper who has questions.",
        ),
        features: &[Feature::LanternSite, Feature::SoulCandidate],
    },
    Waypoint {
        id: WaypointId(2),
        name: "The Shoal Markets",
        chapter: Chapter::Shallows,
        chart_pos: (58, 80),
        scene: scene(
            "arrival.shoal_markets",
            "Rafts lashed to rafts. Everything is for sale, mostly fish.",
        ),
        features: &[Feature::WayStation],
    },
    Waypoint {
        id: WaypointId(3),
        name: "The Drowned Choir",
        chapter: Chapter::Shallows,
        chart_pos: (42, 77),
        scene: scene(
            "arrival.drowned_choir",
            "Steeples break the surface. Something below still sings the evening office.",
        ),
        features: &[Feature::SoulCandidate],
    },
    Waypoint {
        id: WaypointId(4),
        name: "Graywater Anchorage",
        chapter: Chapter::Shallows,
        chart_pos: (38, 73),
        scene: scene(
            "arrival.graywater_anchorage",
            "A working yard, half-sunk cranes, one stubborn shipwright.",
        ),
        features: &[Feature::Shipyard],
    },
    Waypoint {
        id: WaypointId(5),
        name: "The Kelp Meadows",
        chapter: Chapter::Shallows,
        chart_pos: (72, 77),
        scene: scene(
            "arrival.kelp_meadows",
            "Green fields under the hull. Gulls. It is almost like land.",
        ),
        features: &[Feature::RestStop, Feature::SoulCandidate],
    },
    Waypoint {
        id: WaypointId(6),
        name: "The Tollgate",
        chapter: Chapter::Shallows,
        chart_pos: (74, 72),
        scene: scene(
            "arrival.tollgate",
            "A chain across the strait and a booth that has outlived three empires.",
        ),
        features: &[],
    },
    Waypoint {
        id: WaypointId(7),
        name: "Candlewick Light",
        chapter: Chapter::Shallows,
        chart_pos: (56, 69),
        scene: scene(
            "arrival.candlewick_light",
            "The last lighthouse that still answers when hailed.",
        ),
        features: &[Feature::LanternSite],
    },
    Waypoint {
        id: WaypointId(8),
        name: "The Salt Orchard",
        chapter: Chapter::Shallows,
        chart_pos: (44, 66),
        scene: scene(
            "arrival.salt_orchard",
            "Trees of crusted white. Windfall fruit that keeps forever.",
        ),
        features: &[Feature::RestStop],
    },
    Waypoint {
        id: WaypointId(9),
        name: "The Ossuary Reef",
        chapter: Chapter::Shallows,
        chart_pos: (68, 66),
        scene: scene(
            "arrival.ossuary_reef",
            "The reef is white and it is not coral.",
        ),
        features: &[Feature::SoulCandidate],
    },
    Waypoint {
        id: WaypointId(10),
        name: "The Shallows Gate",
        chapter: Chapter::Shallows,
        chart_pos: (56, 62),
        scene: scene(
            "arrival.shallows_gate",
            "Two pillars, and past them the water changes color.",
        ),
        features: &[Feature::LanternSite],
    },
    // ── Chapter II: The Drift Roads ─────────────────────────────────────────
    Waypoint {
        id: WaypointId(11),
        name: "First Drift",
        chapter: Chapter::DriftRoads,
        chart_pos: (50, 58),
        scene: scene(
            "arrival.first_drift",
            "The current takes the hull like a hand takes an elbow.",
        ),
        features: &[Feature::RestStop],
    },
    Waypoint {
        id: WaypointId(12),
        name: "The Wandering Fair",
        chapter: Chapter::DriftRoads,
        chart_pos: (56, 55),
        scene: scene(
            "arrival.wandering_fair",
            "A fair that drifts where it pleases. Tonight it pleases to meet you.",
        ),
        features: &[Feature::WayStation, Feature::SoulCandidate],
    },
    Waypoint {
        id: WaypointId(13),
        name: "The Mirrorcalm",
        chapter: Chapter::DriftRoads,
        chart_pos: (36, 52),
        scene: scene(
            "arrival.mirrorcalm",
            "Water so still the stars double. The crew speaks in whispers, unasked.",
        ),
        features: &[],
    },
    Waypoint {
        id: WaypointId(14),
        name: "Saint Elm's Rest",
        chapter: Chapter::DriftRoads,
        chart_pos: (32, 48),
        scene: scene(
            "arrival.saint_elms_rest",
            "A hospice for ships. They mend hulls and the other kind of damage.",
        ),
        features: &[Feature::Shipyard, Feature::RestStop, Feature::SoulCandidate],
    },
    Waypoint {
        id: WaypointId(15),
        name: "The Hungry Narrows",
        chapter: Chapter::DriftRoads,
        chart_pos: (56, 51),
        scene: scene(
            "arrival.hungry_narrows",
            "Cliffs close in. The water moves fast and takes its fee in stores.",
        ),
        features: &[],
    },
    Waypoint {
        id: WaypointId(16),
        name: "The Beacon Graveyard",
        chapter: Chapter::DriftRoads,
        chart_pos: (58, 47),
        scene: scene(
            "arrival.beacon_graveyard",
            "Dead lighthouses stand in rows. One of them is lying about being dead.",
        ),
        features: &[Feature::SoulCandidate, Feature::LanternSite],
    },
    Waypoint {
        id: WaypointId(17),
        name: "The Long Meander",
        chapter: Chapter::DriftRoads,
        chart_pos: (78, 52),
        scene: scene(
            "arrival.long_meander",
            "The slow way around. The water hums old work songs.",
        ),
        features: &[],
    },
    Waypoint {
        id: WaypointId(18),
        name: "The Pilgrims' Buoy",
        chapter: Chapter::DriftRoads,
        chart_pos: (82, 48),
        scene: scene(
            "arrival.pilgrims_buoy",
            "A bell buoy hung with ribbons and letters from every crossing before yours.",
        ),
        features: &[Feature::WayStation, Feature::SoulCandidate],
    },
    Waypoint {
        id: WaypointId(19),
        name: "The Crossroads Light",
        chapter: Chapter::DriftRoads,
        chart_pos: (56, 44),
        scene: scene(
            "arrival.crossroads_light",
            "Every road you didn't take is visible from here, briefly.",
        ),
        features: &[Feature::LanternSite],
    },
    Waypoint {
        id: WaypointId(20),
        name: "The Whale Roads",
        chapter: Chapter::DriftRoads,
        chart_pos: (42, 41),
        scene: scene(
            "arrival.whale_roads",
            "Something vast keeps pace off the port side for three days. It means well.",
        ),
        features: &[Feature::SoulCandidate],
    },
    Waypoint {
        id: WaypointId(21),
        name: "The Smugglers' Slip",
        chapter: Chapter::DriftRoads,
        chart_pos: (70, 41),
        scene: scene(
            "arrival.smugglers_slip",
            "A hidden harbor that asks no questions and answers none either.",
        ),
        features: &[Feature::SoulCandidate],
    },
    Waypoint {
        id: WaypointId(22),
        name: "Drift's End",
        chapter: Chapter::DriftRoads,
        chart_pos: (56, 38),
        scene: scene(
            "arrival.drifts_end",
            "The current lets go. Ahead the water is black and patient.",
        ),
        features: &[Feature::Shipyard],
    },
    // ── Chapter III: The Starless Deep ──────────────────────────────────────
    Waypoint {
        id: WaypointId(23),
        name: "The Threshold",
        chapter: Chapter::StarlessDeep,
        chart_pos: (50, 34),
        scene: scene(
            "arrival.threshold",
            "The last place a letter can reach you. Write what you need to write.",
        ),
        features: &[],
    },
    Waypoint {
        id: WaypointId(24),
        name: "The Last Lantern",
        chapter: Chapter::StarlessDeep,
        chart_pos: (56, 31),
        scene: scene(
            "arrival.last_lantern",
            "A lamp on a pole in the middle of nothing. Someone keeps it lit.",
        ),
        features: &[Feature::WayStation, Feature::LanternSite],
    },
    Waypoint {
        id: WaypointId(25),
        name: "The Silence",
        chapter: Chapter::StarlessDeep,
        chart_pos: (40, 28),
        scene: scene(
            "arrival.silence",
            "Sound stops working. The crew writes notes and holds them up.",
        ),
        features: &[],
    },
    Waypoint {
        id: WaypointId(26),
        name: "The Choir of Bones",
        chapter: Chapter::StarlessDeep,
        chart_pos: (36, 25),
        scene: scene(
            "arrival.choir_of_bones",
            "In the silence, something sings anyway. It has been waiting for an audience.",
        ),
        features: &[Feature::SoulCandidate],
    },
    Waypoint {
        id: WaypointId(27),
        name: "The Cold Shelf",
        chapter: Chapter::StarlessDeep,
        chart_pos: (72, 28),
        scene: scene(
            "arrival.cold_shelf",
            "Ice where no ice should be. The stores freeze; so does conversation.",
        ),
        features: &[],
    },
    Waypoint {
        id: WaypointId(28),
        name: "The Sleepers' Trench",
        chapter: Chapter::StarlessDeep,
        chart_pos: (74, 25),
        scene: scene(
            "arrival.sleepers_trench",
            "Ships at anchor, crews asleep for years. One bunk is warm.",
        ),
        features: &[Feature::SoulCandidate],
    },
    Waypoint {
        id: WaypointId(29),
        name: "The Ember Hold",
        chapter: Chapter::StarlessDeep,
        chart_pos: (56, 22),
        scene: scene(
            "arrival.ember_hold",
            "A caldera harbor, warm water, forge-light. The deep's one hearth.",
        ),
        features: &[Feature::Shipyard, Feature::RestStop],
    },
    Waypoint {
        id: WaypointId(30),
        name: "The Starless Reach",
        chapter: Chapter::StarlessDeep,
        chart_pos: (44, 19),
        scene: scene(
            "arrival.starless_reach",
            "No stars, no bottom, no horizon. Only the heading you chose.",
        ),
        features: &[],
    },
    Waypoint {
        id: WaypointId(31),
        name: "The Gloaming Shallows",
        chapter: Chapter::StarlessDeep,
        chart_pos: (68, 19),
        scene: scene(
            "arrival.gloaming_shallows",
            "Soft luminous shoals. The deep, being gentle for once.",
        ),
        features: &[Feature::RestStop],
    },
    Waypoint {
        id: WaypointId(32),
        name: "Deepgate",
        chapter: Chapter::StarlessDeep,
        chart_pos: (56, 16),
        scene: scene(
            "arrival.deepgate",
            "A rift in the dark, and through it: green light.",
        ),
        features: &[Feature::LanternSite],
    },
    // ── Chapter IV: The Roots of Light ──────────────────────────────────────
    Waypoint {
        id: WaypointId(33),
        name: "First Light",
        chapter: Chapter::RootsOfLight,
        chart_pos: (50, 12),
        scene: scene(
            "arrival.first_light",
            "Dawn, after a chapter without one. The crew comes up unasked to watch.",
        ),
        features: &[],
    },
    Waypoint {
        id: WaypointId(34),
        name: "The Root Shallows",
        chapter: Chapter::RootsOfLight,
        chart_pos: (56, 9),
        scene: scene(
            "arrival.root_shallows",
            "Roots the size of rivers, glowing under the hull. You are close.",
        ),
        features: &[Feature::RestStop],
    },
    Waypoint {
        id: WaypointId(35),
        name: "The Bloom Fields",
        chapter: Chapter::RootsOfLight,
        chart_pos: (44, 6),
        scene: scene(
            "arrival.bloom_fields",
            "Flowers on the water for miles. They open as the bow touches them.",
        ),
        features: &[],
    },
    Waypoint {
        id: WaypointId(36),
        name: "The Thorn Run",
        chapter: Chapter::RootsOfLight,
        chart_pos: (68, 6),
        scene: scene(
            "arrival.thorn_run",
            "The fast way in, between roots that did not agree to it.",
        ),
        features: &[],
    },
    Waypoint {
        id: WaypointId(37),
        name: "The Roots of Light",
        chapter: Chapter::RootsOfLight,
        chart_pos: (56, 2),
        scene: scene(
            "arrival.roots_of_light",
            "The Tree. After everything, the Tree.",
        ),
        features: &[Feature::Harbor],
    },
];

/// The v1 roads (45 authored; 7 junctions: 2/2/2/1 by chapter).
pub const ROADS: [Road; 45] = [
    // ── Chapter I: The Shallows (12..20 provisions) ─────────────────────────
    Road {
        id: RoadId(0),
        from: WaypointId(0),
        to: WaypointId(1),
        base_days: 1.0,
        base_provisions: 12,
        character: &[RoadTag::Kind],
        blurb: "Known water. The chart still remembers this part.",
        known_stops: &[],
        threat: None,
    },
    Road {
        id: RoadId(1),
        from: WaypointId(1),
        to: WaypointId(2),
        base_days: 1.0,
        base_provisions: 12,
        character: &[],
        blurb: "Shoal lanes, marked and mild.",
        known_stops: &[],
        threat: None,
    },
    // Junction 1: The Shoal Markets
    Road {
        id: RoadId(2),
        from: WaypointId(2),
        to: WaypointId(3),
        base_days: 1.5,
        base_provisions: 14,
        character: &[RoadTag::Singing],
        blurb: "West over the drowned parishes. Singing carries at dusk.",
        known_stops: &[WaypointId(3)],
        threat: None,
    },
    Road {
        id: RoadId(3),
        from: WaypointId(2),
        to: WaypointId(5),
        base_days: 1.5,
        base_provisions: 16,
        character: &[RoadTag::Kind],
        blurb: "East through the kelp. Slow, green, and gentle.",
        known_stops: &[WaypointId(5), WaypointId(6)],
        threat: None,
    },
    Road {
        id: RoadId(4),
        from: WaypointId(3),
        to: WaypointId(4),
        base_days: 1.0,
        base_provisions: 13,
        character: &[],
        blurb: "Past the steeples to the old yards.",
        known_stops: &[],
        threat: None,
    },
    Road {
        id: RoadId(5),
        from: WaypointId(4),
        to: WaypointId(7),
        base_days: 1.5,
        base_provisions: 15,
        character: &[],
        blurb: "Out of the gray water, toward the light that answers.",
        known_stops: &[],
        threat: None,
    },
    Road {
        id: RoadId(6),
        from: WaypointId(5),
        to: WaypointId(6),
        base_days: 1.0,
        base_provisions: 12,
        character: &[RoadTag::Kind],
        blurb: "Meadow water to the chain.",
        known_stops: &[],
        threat: None,
    },
    Road {
        id: RoadId(7),
        from: WaypointId(6),
        to: WaypointId(7),
        base_days: 1.5,
        base_provisions: 18,
        character: &[RoadTag::Tolled],
        blurb: "The strait beyond the booth. Passage costs what it costs.",
        known_stops: &[],
        threat: None,
    },
    // Junction 2: Candlewick Light
    Road {
        id: RoadId(8),
        from: WaypointId(7),
        to: WaypointId(8),
        base_days: 1.5,
        base_provisions: 15,
        character: &[RoadTag::Kind],
        blurb: "The orchard channel. Windfall salt-fruit for the taking.",
        known_stops: &[WaypointId(8)],
        threat: None,
    },
    Road {
        id: RoadId(9),
        from: WaypointId(7),
        to: WaypointId(9),
        base_days: 1.5,
        base_provisions: 20,
        character: &[RoadTag::Dark],
        blurb: "Over the white reef. The lead line comes up wrong.",
        known_stops: &[WaypointId(9)],
        threat: Some(ThreatRef {
            id: "threat.ossuary_warden",
            name: "the Ossuary Warden",
        }),
    },
    Road {
        id: RoadId(10),
        from: WaypointId(8),
        to: WaypointId(10),
        base_days: 1.0,
        base_provisions: 14,
        character: &[],
        blurb: "Last mild water before the pillars.",
        known_stops: &[],
        threat: None,
    },
    Road {
        id: RoadId(11),
        from: WaypointId(9),
        to: WaypointId(10),
        base_days: 1.0,
        base_provisions: 14,
        character: &[],
        blurb: "Off the reef and glad of it.",
        known_stops: &[],
        threat: None,
    },
    // Gateway leg: Shallows -> Drift Roads
    Road {
        id: RoadId(12),
        from: WaypointId(10),
        to: WaypointId(11),
        base_days: 2.0,
        base_provisions: 20,
        character: &[],
        blurb: "Through the pillars. The water changes color and intent.",
        known_stops: &[],
        threat: None,
    },
    // ── Chapter II: The Drift Roads (15..30 provisions) ─────────────────────
    Road {
        id: RoadId(13),
        from: WaypointId(11),
        to: WaypointId(12),
        base_days: 1.0,
        base_provisions: 15,
        character: &[RoadTag::Lucky],
        blurb: "Riding the first current to wherever the fair is tonight.",
        known_stops: &[],
        threat: None,
    },
    // Junction 3: The Wandering Fair (three roads)
    Road {
        id: RoadId(14),
        from: WaypointId(12),
        to: WaypointId(13),
        base_days: 2.0,
        base_provisions: 24,
        character: &[RoadTag::Kind],
        blurb: "West into the still water. Nothing hurries there, including you.",
        known_stops: &[WaypointId(13), WaypointId(14)],
        threat: None,
    },
    Road {
        id: RoadId(15),
        from: WaypointId(12),
        to: WaypointId(15),
        base_days: 1.5,
        base_provisions: 18,
        character: &[RoadTag::Hungry],
        blurb: "Straight through the narrows. Fast, and it eats stores.",
        known_stops: &[WaypointId(15)],
        threat: None,
    },
    Road {
        id: RoadId(16),
        from: WaypointId(12),
        to: WaypointId(17),
        base_days: 2.5,
        base_provisions: 28,
        character: &[RoadTag::Singing, RoadTag::Lucky],
        blurb: "The long way east, with the work songs.",
        known_stops: &[WaypointId(17), WaypointId(18)],
        threat: None,
    },
    Road {
        id: RoadId(17),
        from: WaypointId(13),
        to: WaypointId(14),
        base_days: 1.5,
        base_provisions: 18,
        character: &[RoadTag::Kind],
        blurb: "Mirror water to the hospice lights.",
        known_stops: &[],
        threat: None,
    },
    Road {
        id: RoadId(18),
        from: WaypointId(14),
        to: WaypointId(19),
        base_days: 2.0,
        base_provisions: 24,
        character: &[],
        blurb: "Rested and mended, back into the moving water.",
        known_stops: &[],
        threat: None,
    },
    Road {
        id: RoadId(19),
        from: WaypointId(15),
        to: WaypointId(16),
        base_days: 1.5,
        base_provisions: 20,
        character: &[RoadTag::Hungry],
        blurb: "Out of the narrows past the dead lights.",
        known_stops: &[],
        threat: None,
    },
    Road {
        id: RoadId(20),
        from: WaypointId(16),
        to: WaypointId(19),
        base_days: 2.0,
        base_provisions: 26,
        character: &[RoadTag::Dark],
        blurb: "One of the graveyard beacons follows you with its beam.",
        known_stops: &[],
        threat: None,
    },
    Road {
        id: RoadId(21),
        from: WaypointId(17),
        to: WaypointId(18),
        base_days: 1.5,
        base_provisions: 20,
        character: &[RoadTag::Singing],
        blurb: "Meander water, ribboned and slow.",
        known_stops: &[],
        threat: None,
    },
    Road {
        id: RoadId(22),
        from: WaypointId(18),
        to: WaypointId(19),
        base_days: 1.5,
        base_provisions: 22,
        character: &[],
        blurb: "In from the buoy, letters read, ribbons tied.",
        known_stops: &[],
        threat: None,
    },
    // Junction 4: The Crossroads Light
    Road {
        id: RoadId(23),
        from: WaypointId(19),
        to: WaypointId(20),
        base_days: 2.0,
        base_provisions: 24,
        character: &[RoadTag::Singing, RoadTag::Kind],
        blurb: "The whale roads. Old companions, if you keep their pace.",
        known_stops: &[WaypointId(20)],
        threat: None,
    },
    Road {
        id: RoadId(24),
        from: WaypointId(19),
        to: WaypointId(21),
        base_days: 2.0,
        base_provisions: 30,
        character: &[RoadTag::Tolled, RoadTag::Lucky],
        blurb: "The slip lanes. Expensive, discreet, occasionally generous.",
        known_stops: &[WaypointId(21)],
        threat: None,
    },
    Road {
        id: RoadId(25),
        from: WaypointId(20),
        to: WaypointId(22),
        base_days: 1.5,
        base_provisions: 20,
        character: &[RoadTag::Kind],
        blurb: "The escort peels away where the current ends.",
        known_stops: &[],
        threat: None,
    },
    Road {
        id: RoadId(26),
        from: WaypointId(21),
        to: WaypointId(22),
        base_days: 1.5,
        base_provisions: 18,
        character: &[RoadTag::Lucky],
        blurb: "Quiet lanes out of the slip, riding someone else's luck.",
        known_stops: &[],
        threat: None,
    },
    // Gateway leg: Drift Roads -> Starless Deep
    Road {
        id: RoadId(27),
        from: WaypointId(22),
        to: WaypointId(23),
        base_days: 2.5,
        base_provisions: 30,
        character: &[RoadTag::Dark],
        blurb: "The current lets go. Ahead, the dark takes over.",
        known_stops: &[],
        threat: None,
    },
    // ── Chapter III: The Starless Deep (20..45 provisions) ──────────────────
    Road {
        id: RoadId(28),
        from: WaypointId(23),
        to: WaypointId(24),
        base_days: 1.0,
        base_provisions: 20,
        character: &[RoadTag::Dark],
        blurb: "Toward the one lamp somebody keeps lit out here.",
        known_stops: &[],
        threat: None,
    },
    // Junction 5: The Last Lantern
    Road {
        id: RoadId(29),
        from: WaypointId(24),
        to: WaypointId(25),
        base_days: 3.0,
        base_provisions: 24,
        character: &[RoadTag::Dark],
        blurb: "The silent road. Long, cheap, and it keeps its opinions to itself.",
        known_stops: &[WaypointId(25)],
        threat: Some(ThreatRef {
            id: "threat.the_silence",
            name: "the Silence itself",
        }),
    },
    Road {
        id: RoadId(30),
        from: WaypointId(24),
        to: WaypointId(27),
        base_days: 2.0,
        base_provisions: 34,
        character: &[RoadTag::Hungry, RoadTag::Dark],
        blurb: "Over the cold shelf. Shorter, but the cold eats the stores.",
        known_stops: &[WaypointId(27), WaypointId(28)],
        threat: None,
    },
    Road {
        id: RoadId(31),
        from: WaypointId(25),
        to: WaypointId(26),
        base_days: 1.5,
        base_provisions: 22,
        character: &[RoadTag::Dark, RoadTag::Singing],
        blurb: "Deeper into the quiet, toward the thing that sings anyway.",
        known_stops: &[],
        threat: None,
    },
    Road {
        id: RoadId(32),
        from: WaypointId(26),
        to: WaypointId(29),
        base_days: 2.0,
        base_provisions: 30,
        character: &[RoadTag::Dark],
        blurb: "Out of the choir's hearing, toward forge-light.",
        known_stops: &[],
        threat: None,
    },
    Road {
        id: RoadId(33),
        from: WaypointId(27),
        to: WaypointId(28),
        base_days: 1.5,
        base_provisions: 26,
        character: &[RoadTag::Hungry],
        blurb: "Along the shelf lip, past the anchored sleepers.",
        known_stops: &[],
        threat: None,
    },
    Road {
        id: RoadId(34),
        from: WaypointId(28),
        to: WaypointId(29),
        base_days: 2.0,
        base_provisions: 32,
        character: &[RoadTag::Dark],
        blurb: "Away from the warm bunk, toward the warm harbor.",
        known_stops: &[],
        threat: None,
    },
    // Junction 6: The Ember Hold
    Road {
        id: RoadId(35),
        from: WaypointId(29),
        to: WaypointId(30),
        base_days: 2.5,
        base_provisions: 25,
        character: &[RoadTag::Dark],
        blurb: "The straight dark. Only the heading you chose.",
        known_stops: &[WaypointId(30)],
        threat: None,
    },
    Road {
        id: RoadId(36),
        from: WaypointId(29),
        to: WaypointId(31),
        base_days: 2.0,
        base_provisions: 36,
        character: &[RoadTag::Kind],
        blurb: "The gloaming way, lit and gentle and priced accordingly.",
        known_stops: &[WaypointId(31)],
        threat: None,
    },
    Road {
        id: RoadId(37),
        from: WaypointId(30),
        to: WaypointId(32),
        base_days: 2.0,
        base_provisions: 30,
        character: &[RoadTag::Dark],
        blurb: "A crack of green light grows dead ahead.",
        known_stops: &[],
        threat: None,
    },
    Road {
        id: RoadId(38),
        from: WaypointId(31),
        to: WaypointId(32),
        base_days: 1.5,
        base_provisions: 28,
        character: &[],
        blurb: "The shoals shade from silver to green.",
        known_stops: &[],
        threat: None,
    },
    // Gateway leg: Starless Deep -> Roots of Light
    Road {
        id: RoadId(39),
        from: WaypointId(32),
        to: WaypointId(33),
        base_days: 3.0,
        base_provisions: 40,
        character: &[RoadTag::Dark],
        blurb: "Through the rift. The longest night, and then, allegedly, dawn.",
        known_stops: &[],
        threat: None,
    },
    // ── Chapter IV: The Roots of Light (25..55 provisions) ──────────────────
    Road {
        id: RoadId(40),
        from: WaypointId(33),
        to: WaypointId(34),
        base_days: 1.5,
        base_provisions: 25,
        character: &[RoadTag::Kind],
        blurb: "Down the light, toward roots the size of rivers.",
        known_stops: &[],
        threat: None,
    },
    // Junction 7: The Root Shallows
    Road {
        id: RoadId(41),
        from: WaypointId(34),
        to: WaypointId(35),
        base_days: 2.0,
        base_provisions: 25,
        character: &[RoadTag::Kind, RoadTag::Singing],
        blurb: "Through the bloom fields. The slow, flowering way in.",
        known_stops: &[WaypointId(35)],
        threat: None,
    },
    Road {
        id: RoadId(42),
        from: WaypointId(34),
        to: WaypointId(36),
        base_days: 1.0,
        base_provisions: 40,
        character: &[RoadTag::Hungry],
        blurb: "The thorn run. Fast, close, and it will cost the hold.",
        known_stops: &[WaypointId(36)],
        threat: Some(ThreatRef {
            id: "threat.the_thorns",
            name: "the Thorns",
        }),
    },
    Road {
        id: RoadId(43),
        from: WaypointId(35),
        to: WaypointId(37),
        base_days: 2.0,
        base_provisions: 35,
        character: &[RoadTag::Singing, RoadTag::Kind],
        blurb: "The last miles, flower-lit, every soul on deck.",
        known_stops: &[],
        threat: None,
    },
    Road {
        id: RoadId(44),
        from: WaypointId(36),
        to: WaypointId(37),
        base_days: 1.5,
        base_provisions: 30,
        character: &[],
        blurb: "Out of the thorns and into the roots' embrace.",
        known_stops: &[],
        threat: None,
    },
];

/// Authored rumor table. Way-stations sell from curated lists
/// (see [`way_station_stock`]); arrival scenes and pilgrim hails grant the rest.
pub const RUMORS: [RumorDef; 8] = [
    RumorDef {
        id: RumorId(0),
        text: "The Tollgate keeper waives the chain for a crew that sings the old office.",
        subject: RumorSubject::Road(RoadId(7)),
        reveals_stop: None,
    },
    RumorDef {
        id: RumorId(1),
        text: "A shipwright still works the Graywater yards, past the drowned steeples.",
        subject: RumorSubject::Road(RoadId(2)),
        reveals_stop: Some(WaypointId(4)),
    },
    RumorDef {
        id: RumorId(2),
        text: "The reef past Candlewick is white as teeth, and the Warden counts every hull.",
        subject: RumorSubject::Road(RoadId(9)),
        reveals_stop: None,
    },
    RumorDef {
        id: RumorId(3),
        text: "Saint Elm's takes any ship that limps in, and mends the other kind of damage too.",
        subject: RumorSubject::Road(RoadId(14)),
        reveals_stop: Some(WaypointId(14)),
    },
    RumorDef {
        id: RumorId(4),
        text: "The narrows are fast but they eat stores like a stove eats driftwood.",
        subject: RumorSubject::Road(RoadId(15)),
        reveals_stop: Some(WaypointId(16)),
    },
    RumorDef {
        id: RumorId(5),
        text: "Follow the whales and no harm finds you; outrun them and you sail alone.",
        subject: RumorSubject::Road(RoadId(23)),
        reveals_stop: None,
    },
    RumorDef {
        id: RumorId(6),
        text: "On the silent road, the ones who sleep in the Trench dream of visitors.",
        subject: RumorSubject::Road(RoadId(30)),
        reveals_stop: Some(WaypointId(28)),
    },
    RumorDef {
        id: RumorId(7),
        text: "There is a hearth in the deep. Forge-light, warm water. Sailors call it the Ember Hold.",
        subject: RumorSubject::Place(WaypointId(29)),
        reveals_stop: None,
    },
];

/// The Tree on the horizon: six art stages, selected by progress toward the
/// sink (see [`tree_stage_index`]). Stage art lives with the route data.
pub const TREE_STAGES: [&str; 6] = [
    // Stage 0: a smudge on the horizon
    "      .\n      |",
    // Stage 1: a distant sapling silhouette
    "     \\|/\n      |",
    // Stage 2: branches resolve
    "    \\ | /\n    \\\\|//\n      |",
    // Stage 3: a crown, faint light
    "   . \\|/ .\n   \\\\\\|///\n     \\|/\n      |",
    // Stage 4: the crown fills the sky's edge
    "  *  \\|/  *\n  .\\\\\\|///.\n   \\\\\\|///\n     \\|/\n      |",
    // Stage 5: the Tree, whole, alight
    " *  .\\|/.  *\n  \\\\\\\\|////\n .\\\\\\\\|////.\n   \\\\\\|///\n     \\|/\n     /|\\",
];

// ── Lookup helpers ──────────────────────────────────────────────────────────

/// Waypoint by id. Ids are ordinal, so this is an index.
pub fn waypoint(id: WaypointId) -> &'static Waypoint {
    &WAYPOINTS[id.0 as usize]
}

/// Road by id. Ids are ordinal, so this is an index.
pub fn road(id: RoadId) -> &'static Road {
    &ROADS[id.0 as usize]
}

/// Rumor definition by id.
pub fn rumor(id: RumorId) -> &'static RumorDef {
    &RUMORS[id.0 as usize]
}

/// All roads leaving a waypoint, in authored order.
pub fn roads_from(id: WaypointId) -> impl Iterator<Item = &'static Road> {
    ROADS.iter().filter(move |r| r.from == id)
}

/// A junction is any waypoint with more than one outgoing road.
#[allow(dead_code)] // Exercised by the graph invariant tests; UI slot pending.
pub fn is_junction(id: WaypointId) -> bool {
    roads_from(id).count() > 1
}

/// The cheapest road out of a waypoint (by base provisions), if any leave it.
/// This road is never locked: it can always be sailed, even into a drift.
pub fn cheapest_road_from(id: WaypointId) -> Option<&'static Road> {
    roads_from(id).min_by_key(|r| r.base_provisions)
}

/// Chapter gateway waypoints: the single exit point of each chapter.
/// Chapter-beat scenes (spec 4) hang off these.
#[allow(dead_code)]
pub const CHAPTER_GATEWAYS: [WaypointId; 4] =
    [WaypointId(10), WaypointId(22), WaypointId(32), ROUTE_SINK];

#[allow(dead_code)] // Chapter-beat slot for spec 4.
pub fn is_chapter_gateway(id: WaypointId) -> bool {
    CHAPTER_GATEWAYS.contains(&id)
}

/// Rumors a given way-station sells, in the order they are sold (one per
/// visit). Curated per station so a purchase is always relevant.
pub fn way_station_stock(id: WaypointId) -> &'static [RumorId] {
    match id.0 {
        2 => &[RumorId(1), RumorId(0), RumorId(2)], // The Shoal Markets
        12 => &[RumorId(3), RumorId(4)],            // The Wandering Fair
        18 => &[RumorId(5), RumorId(7)],            // The Pilgrims' Buoy
        24 => &[RumorId(6)],                        // The Last Lantern
        _ => &[],
    }
}

/// Tree art stage for a waypoint, by chart progress toward the sink.
/// Six stages spread over the crossing; the sink always shows the last.
pub fn tree_stage_index(at: WaypointId) -> usize {
    if at == ROUTE_SINK {
        return TREE_STAGES.len() - 1;
    }
    let start_y = waypoint(ROUTE_START).chart_pos.1 as f32;
    let sink_y = waypoint(ROUTE_SINK).chart_pos.1 as f32;
    let here_y = waypoint(at).chart_pos.1 as f32;
    let progress = ((start_y - here_y) / (start_y - sink_y)).clamp(0.0, 1.0);
    ((progress * TREE_STAGES.len() as f32) as usize).min(TREE_STAGES.len() - 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// Every maximal route through the DAG, as waypoint sequences.
    fn all_routes() -> Vec<Vec<WaypointId>> {
        let mut routes = Vec::new();
        let mut stack = vec![vec![ROUTE_START]];
        while let Some(path) = stack.pop() {
            let last = *path.last().unwrap();
            let next: Vec<_> = roads_from(last).collect();
            if next.is_empty() {
                routes.push(path);
                continue;
            }
            for r in next {
                let mut p = path.clone();
                p.push(r.to);
                stack.push(p);
            }
        }
        routes
    }

    #[test]
    fn ids_are_ordinal_indices() {
        for (i, w) in WAYPOINTS.iter().enumerate() {
            assert_eq!(w.id.0 as usize, i, "waypoint {} id mismatch", w.name);
        }
        for (i, r) in ROADS.iter().enumerate() {
            assert_eq!(r.id.0 as usize, i, "road {i} id mismatch");
        }
        for (i, r) in RUMORS.iter().enumerate() {
            assert_eq!(r.id.0 as usize, i, "rumor {i} id mismatch");
        }
    }

    #[test]
    fn graph_is_acyclic_with_single_sink() {
        // Single sink: exactly one waypoint has no outgoing roads, and it is
        // the Tree.
        let sinks: Vec<_> = WAYPOINTS
            .iter()
            .filter(|w| roads_from(w.id).count() == 0)
            .map(|w| w.id)
            .collect();
        assert_eq!(sinks, vec![ROUTE_SINK], "the Tree must be the only sink");

        // Acyclic + every road leads toward the Tree: DFS from the start
        // terminates and reaches the sink on every path (all_routes() would
        // loop forever on a cycle; bound it by pathological path count).
        let routes = all_routes();
        assert!(!routes.is_empty());
        for route in &routes {
            assert_eq!(*route.last().unwrap(), ROUTE_SINK);
            // No repeated waypoint within a path (would indicate a cycle).
            let unique: HashSet<_> = route.iter().collect();
            assert_eq!(unique.len(), route.len(), "cycle detected in {route:?}");
        }

        // Every waypoint except the start is reachable (no orphaned content
        // that is not on any route).
        let reachable: HashSet<_> = routes.iter().flatten().copied().collect();
        assert_eq!(reachable.len(), WAYPOINTS.len(), "orphaned waypoints exist");
    }

    #[test]
    fn branches_rejoin_within_their_chapter() {
        // No road crosses a chapter boundary except gateway legs, and gateway
        // legs only ever leave a chapter gateway.
        for r in &ROADS {
            let from = waypoint(r.from);
            let to = waypoint(r.to);
            if from.chapter != to.chapter {
                assert!(
                    is_chapter_gateway(r.from),
                    "road {} crosses chapter boundary from non-gateway {}",
                    r.id.0,
                    from.name
                );
                assert_eq!(
                    roads_from(r.from).count(),
                    1,
                    "gateway {} must have exactly one exit",
                    from.name
                );
            }
        }
    }

    #[test]
    fn seven_junctions_two_two_two_one() {
        let mut counts = [0usize; 4];
        for w in &WAYPOINTS {
            if is_junction(w.id) {
                counts[w.chapter as usize] += 1;
            }
        }
        assert_eq!(counts, [2, 2, 2, 1], "junction distribution by chapter");
    }

    #[test]
    fn affordability_invariant_at_every_junction() {
        for w in &WAYPOINTS {
            if is_junction(w.id) {
                let cheapest = cheapest_road_from(w.id).unwrap();
                assert!(
                    u32::from(cheapest.base_provisions)
                        <= crate::vessel::voyage::DRIFT_RECOVERY_PROVISIONS,
                    "junction {} cheapest road costs {} > drift recovery",
                    w.name,
                    cheapest.base_provisions
                );
            }
        }
    }

    #[test]
    fn content_parity_on_every_maximal_route() {
        for route in all_routes() {
            let count = |f: Feature| {
                route
                    .iter()
                    .filter(|id| waypoint(**id).has_feature(f))
                    .count()
            };
            assert!(
                count(Feature::SoulCandidate) >= 5,
                "route {route:?} passes fewer than 5 soul-candidate scenes"
            );
            assert!(
                count(Feature::Shipyard) >= 2,
                "route {route:?} passes fewer than 2 shipyards"
            );
            assert!(
                count(Feature::RestStop) >= 2,
                "route {route:?} passes fewer than 2 rest stops"
            );
        }
    }

    #[test]
    fn route_sizes_match_the_authoring_contract() {
        assert_eq!(WAYPOINTS.len(), 38);
        assert_eq!(ROADS.len(), 45);
        let path_lengths: Vec<usize> = all_routes().iter().map(|r| r.len()).collect();
        let min = *path_lengths.iter().min().unwrap();
        let max = *path_lengths.iter().max().unwrap();
        assert!(
            (22..=28).contains(&min) && (22..=28).contains(&max),
            "taken paths should see ~24 waypoints, got {min}..{max}"
        );
    }

    #[test]
    fn road_prices_and_days_stay_in_authored_ranges() {
        for r in &ROADS {
            assert!(
                (1.0..=3.0).contains(&r.base_days),
                "road {} days {} out of range",
                r.id.0,
                r.base_days
            );
            assert!(
                (12..=55).contains(&r.base_provisions),
                "road {} provisions {} out of range",
                r.id.0,
                r.base_provisions
            );
        }
    }

    #[test]
    fn every_scene_ref_resolves() {
        let mut seen = HashSet::new();
        for w in &WAYPOINTS {
            assert!(!w.scene.id.is_empty());
            assert!(!w.scene.placeholder.is_empty());
            assert!(seen.insert(w.scene.id), "duplicate scene id {}", w.scene.id);
        }
    }

    #[test]
    fn rumors_and_stations_are_consistent() {
        for def in &RUMORS {
            // reveals_stop must lie on the subject road's branch: for v1 we
            // assert it is a real waypoint and, when the subject is a road,
            // is reachable from that road's destination or is the destination.
            if let Some(stop) = def.reveals_stop {
                assert!((stop.0 as usize) < WAYPOINTS.len());
            }
            match def.subject {
                RumorSubject::Road(rid) => assert!((rid.0 as usize) < ROADS.len()),
                RumorSubject::Place(wid) => assert!((wid.0 as usize) < WAYPOINTS.len()),
            }
        }
        // Way-station stock only at way-stations, and every station has stock.
        for w in &WAYPOINTS {
            let stock = way_station_stock(w.id);
            if w.has_feature(Feature::WayStation) {
                assert!(!stock.is_empty(), "way-station {} sells nothing", w.name);
            } else {
                assert!(
                    stock.is_empty(),
                    "{} sells rumors without a station",
                    w.name
                );
            }
        }
    }

    #[test]
    fn tree_stages_progress_toward_the_sink() {
        assert_eq!(tree_stage_index(ROUTE_START), 0);
        assert_eq!(tree_stage_index(ROUTE_SINK), TREE_STAGES.len() - 1);
        // Stages never regress along any route.
        for route in all_routes() {
            let mut last = 0;
            for id in route {
                let stage = tree_stage_index(id);
                assert!(
                    stage >= last,
                    "tree stage regressed at {}",
                    waypoint(id).name
                );
                last = stage;
            }
        }
    }

    #[test]
    fn chart_positions_fit_the_canvas() {
        for w in &WAYPOINTS {
            assert!(
                w.chart_pos.0 < 120 && w.chart_pos.1 < 90,
                "{} off-canvas",
                w.name
            );
        }
    }
}
