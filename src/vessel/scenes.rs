//! Arrival Scenes — Act 2's authored content (sub-project 4).
//!
//! A scene is beats of text plus a payout, recolored deterministically by
//! state (who is aboard, which road brought you, the trim, what you know).
//! Scenes are payoff, never tests: no option menus, no dice. The only
//! interactive moments are doors the design sanctions — asks (spec 3) and
//! refits (engine logic in `voyage.rs`, not scene data).
//!
//! See `docs/superpowers/specs/2026-07-03-vessel-arrival-scenes-design.md`.

use super::route::{Chapter, RoadId, RumorId, WaypointId};
use super::souls::SoulId;
use super::voyage::Trim;

/// A deterministic, state-derived condition for a color line. Pure —
/// the same save always reads the same.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorKey {
    SoulAboard(SoulId),
    ArrivedBy(RoadId),
    TrimIs(Trim),
    KnowsRumor(RumorId),
    HopeAtLeast(u8),
    HopeBelow(u8),
    /// The leg that brought you here included a drift.
    Drifted,
}

/// What completing the scene pays. Applied exactly once, at play time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScenePayout {
    pub provisions: u16,
    pub hope: i8,
    pub rumor: Option<RumorId>,
    /// Keepsakes have no mechanics: mementos for the manifest (spec 7).
    pub keepsake: Option<&'static str>,
}

const fn pay(provisions: u16) -> ScenePayout {
    ScenePayout {
        provisions,
        hope: 0,
        rumor: None,
        keepsake: None,
    }
}

const fn pay_hope(provisions: u16, hope: i8) -> ScenePayout {
    ScenePayout {
        provisions,
        hope,
        rumor: None,
        keepsake: None,
    }
}

/// An authored arrival scene: 1–4 beats, color lines, one payout.
#[derive(Debug, Clone, Copy)]
pub struct SceneDef {
    pub beats: &'static [&'static str],
    /// Matching lines append as extra paragraphs, in authored order.
    pub colors: &'static [(ColorKey, &'static str)],
    pub payout: ScenePayout,
}

/// Scenes indexed by waypoint id (1:1 with `route::WAYPOINTS`).
pub fn scene_def(waypoint: WaypointId) -> &'static SceneDef {
    &SCENES[waypoint.0 as usize]
}

/// The drift-recovery scene, one per chapter: the same event reads
/// differently in the Shallows than the Starless Deep.
pub fn recovery_scene(chapter: Chapter) -> (&'static str, &'static str) {
    match chapter {
        Chapter::Shallows => (
            "The hold, refilled",
            "Two days adrift in known water. Runa's hand-lines and a passing \
             kelp-cutter's charity put food back in the hold. Nobody says \
             'lesson'; everybody thinks it.",
        ),
        Chapter::DriftRoads => (
            "What the current brought",
            "The drift road drifts you. A crate of ship's biscuit, sealed and \
             sound, bumps the hull on the second morning — flotsam from some \
             luckier or unluckier ship. The hold takes it gratefully.",
        ),
        Chapter::StarlessDeep => (
            "The deep provides, coldly",
            "Adrift in the dark. On the second night something vast passes \
             beneath and leaves a wake of stunned pale fish. The crew gathers \
             them in silence and does not discuss the courtesy.",
        ),
        Chapter::RootsOfLight => (
            "Root-fall",
            "Adrift among the roots. Fruit falls from branches that have no \
             business overhanging open water, and keeps falling until the \
             hold stops echoing. The Tree, it seems, feeds pilgrims.",
        ),
    }
}

/// The whole route's scenes. Compact on purpose: base beats carry the
/// place; color lines carry the crossing's particulars.
pub const SCENES: [SceneDef; 38] = [
    // W0 — The Last Harbor
    SceneDef {
        beats: &[
            "The pier lights dim behind you, one window at a time, as the \
             keepers walk the shore road home. Nobody waves twice. There is \
             nothing left to wave at: the Vessel is the last thing this \
             harbor will ever launch.",
            "The harbormaster's ledger lies open on the bollard, a pen \
             across it. The last entry is you. The hold is filled to the \
             chalk line without a word about payment.",
        ],
        colors: &[(
            ColorKey::HopeAtLeast(8),
            "Someone on the pier starts a song, and the crew — quietly, so \
             as not to argue with it — sings the harbor out of sight.",
        )],
        payout: pay(0), // launches full; the pier gave everything already
    },
    // W1 — The Lightship Vigil
    SceneDef {
        beats: &[
            "The lightship rides its ancient mooring, lamp turning for \
             traffic that stopped coming years ago. Its keeper takes your \
             lines with the ease of someone who has decided not to be \
             surprised by anything again.",
            "She trades stores for news and finds your news wanting. 'A \
             tree,' she says, tasting it. 'Well. Somebody should see about \
             that.'",
        ],
        colors: &[(
            ColorKey::SoulAboard(SoulId(2)),
            "Runa knows the keeper's knock. They talk bait and weather for \
             an hour, the way old friends discuss everything else.",
        )],
        payout: ScenePayout {
            provisions: 12,
            hope: 0,
            rumor: None,
            keepsake: Some("a spare wick from the last lightship"),
        },
    },
    // W2 — The Shoal Markets
    SceneDef {
        beats: &["Rafts lashed to rafts lashed to a drowned customs house. \
             Everything is for sale, mostly fish, some of it honestly. The \
             market treats the end of the world as a supply problem, which, \
             it observes, it always was."],
        colors: &[(
            ColorKey::TrimIs(Trim::Run),
            "Word of your pace precedes you; a fishwife charges you the \
             hurry tax and does not pretend otherwise.",
        )],
        payout: pay(28),
    },
    // W3 — The Drowned Choir
    SceneDef {
        beats: &[
            "Steeples break the surface like reeds. At dusk the evening \
             office rises from below, note-perfect, patient — the parishes \
             kept their appointment with God and merely changed address.",
            "The crew doesn't fish these waters. It would feel like \
             stealing from a collection plate.",
        ],
        colors: &[(
            ColorKey::SoulAboard(SoulId(4)),
            "Sefa answers the office from the rail, the descant line. Below, \
             the singing pauses — then makes room for her.",
        )],
        payout: pay(14),
    },
    // W4 — Graywater Anchorage
    SceneDef {
        beats: &[
            "Half-sunk cranes, one working slip, and a shipwright who looks \
             at the Vessel the way a doctor looks at an interesting X-ray. \
             'Loom-woven,' she says. 'I've heard of these. Hold still.'",
        ],
        colors: &[],
        payout: pay(18),
    },
    // W5 — The Kelp Meadows
    SceneDef {
        beats: &[
            "Green fields under the hull, combed by the current. Gulls, \
             actual gulls. For an afternoon it is almost like land, and the \
             crew lies on deck pretending it is.",
        ],
        colors: &[(
            ColorKey::Drifted,
            "After the empty days, the meadows read as a promise kept: the \
             world still has pantries.",
        )],
        payout: pay(20),
    },
    // W6 — The Tollgate
    SceneDef {
        beats: &[
            "A chain across the strait, and a booth that has outlived three \
             empires and intends to outlive the sea. The keeper takes \
             payment in stores, stories, or song — the rate card, carved in \
             slate, predates money twice.",
        ],
        colors: &[(
            ColorKey::KnowsRumor(RumorId(0)),
            "You knew about the singing. The chain drops before the coin \
             purse opens, and the keeper looks almost disappointed.",
        )],
        payout: pay(12),
    },
    // W7 — Candlewick Light
    SceneDef {
        beats: &[
            "The last lighthouse that still answers when hailed. Its lamp \
             room smells of hot brass and stubbornness. The keeper signs \
             the visitors' book for you — the previous entry is nine years \
             old.",
        ],
        colors: &[
            (
                ColorKey::SoulAboard(SoulId(3)),
                "Maren and the keeper speak the lamp-trade's shorthand late \
                 into the night. In the morning the light burns whiter.",
            ),
            (
                ColorKey::ArrivedBy(RoadId(7)),
                "The keeper heard the Tollgate chain drop this morning and \
                 has been expecting you since. News still travels the old \
                 strait faster than hulls do.",
            ),
        ],
        payout: pay(18),
    },
    // W8 — The Salt Orchard
    SceneDef {
        beats: &[
            "Trees of crusted white stand in tide-rows, heavy with fruit \
             that keeps forever and tastes like the memory of apricots. \
             Windfall is free; picking is stealing. The orchard makes the \
             distinction clear without a single sign.",
        ],
        colors: &[],
        payout: ScenePayout {
            provisions: 20,
            hope: 0,
            rumor: None,
            keepsake: Some("a jar of salt-orchard preserve"),
        },
    },
    // W9 — The Ossuary Reef
    SceneDef {
        beats: &[
            "The reef is white and it is not coral. The lead line comes up \
             wrong twice, then right in a way that is worse. Passage is \
             single-file, slow, over the drowned dead of a war nobody won.",
        ],
        colors: &[],
        payout: pay(0), // the reef gives nothing freely; the ledger speaks
    },
    // W10 — The Shallows Gate
    SceneDef {
        beats: &[
            "Two pillars, older than the charts that mark them. Between \
             them the water changes color and, the crew agrees without \
             discussing it, intent.",
            "Behind you: every light you have ever known. Ahead: the Drift \
             Roads. The Vessel pauses at the line like a swimmer at the \
             cold end of the pool, and then goes.",
        ],
        colors: &[(
            ColorKey::HopeBelow(5),
            "Nobody says 'we could still turn around.' The not-saying takes \
             most of the evening.",
        )],
        payout: pay_hope(16, 1),
    },
    // W11 — First Drift
    SceneDef {
        beats: &["The current takes the hull like a hand takes an elbow — \
             polite, unrefusable. Steering becomes negotiation. The crew \
             learns the Drift Roads' first law: you don't sail them, you \
             are walked by them."],
        colors: &[],
        payout: pay(18),
    },
    // W12 — The Wandering Fair
    SceneDef {
        beats: &[
            "A fair that drifts where it pleases; tonight it pleases to \
             meet you. Lanterns, frying oil, a wheel of fortune with the \
             fortune painted out. Nobody asks where you're going — bad form \
             at the end of the world.",
            "Stores are cheap. Gossip is free and worth exactly that, \
             except when it isn't.",
        ],
        colors: &[(
            ColorKey::HopeAtLeast(8),
            "The crew wins the ring-toss so many times the barker retires \
             the prize shelf and joins them for a drink instead.",
        )],
        payout: ScenePayout {
            provisions: 28,
            hope: 0,
            rumor: None,
            keepsake: Some("a fair ribbon, unfaded"),
        },
    },
    // W13 — The Mirrorcalm
    SceneDef {
        beats: &[
            "Water so still the stars double, and the Vessel hangs between \
             two skies. The crew speaks in whispers, unasked. Even the hull \
             creaks apologetically.",
        ],
        colors: &[(
            ColorKey::TrimIs(Trim::Mourn),
            "In the doubled starlight, the names you sail for feel briefly \
             present, standing just behind each shoulder, approving.",
        )],
        payout: pay(18),
    },
    // W14 — Saint Elm's Rest
    SceneDef {
        beats: &[
            "A hospice for ships. They ask no questions except 'where does \
             it hurt,' and they ask it of hulls and crews alike. The yard \
             mends your seams; the kitchen mends the rest.",
        ],
        colors: &[],
        payout: pay_hope(20, 1),
    },
    // W15 — The Hungry Narrows
    SceneDef {
        beats: &[
            "Cliffs close in until the sky is a river overhead. The water \
             runs fast and takes its fee in stores — a barrel snatched by a \
             standing wave, flour damp beyond saving. The narrows are \
             honest. They are just not kind.",
        ],
        colors: &[],
        payout: pay(12),
    },
    // W16 — The Beacon Graveyard
    SceneDef {
        beats: &[
            "Dead lighthouses stand in rows on their drowned headlands, \
             lenses cataracted with salt. One of them is lying about being \
             dead: as you pass, a beam sweeps once — deliberate, tracking — \
             and goes dark again.",
        ],
        colors: &[(
            ColorKey::SoulAboard(SoulId(3)),
            "Maren logs its bearing without comment. Whatever keeps that \
             lamp, she intends it to know somebody noticed.",
        )],
        payout: pay(18),
    },
    // W17 — The Long Meander
    SceneDef {
        beats: &["The slow way around, and the water knows work songs older \
             than rope. The current hums them through the hull at night, \
             4/4, patient, the tempo of oars long gone."],
        colors: &[],
        payout: pay(20),
    },
    // W18 — The Pilgrims' Buoy
    SceneDef {
        beats: &[
            "A bell buoy hung with ribbons, tokens, and letters in oilskin \
             — every crossing that came before yours, leaving word for \
             every crossing after. The crew reads for hours and adds their \
             own.",
        ],
        colors: &[(
            ColorKey::HopeBelow(5),
            "One letter, water-stained, says only: 'It is further than they \
             told us and it is worth it.' It gets passed around all night.",
        )],
        payout: pay_hope(25, 1),
    },
    // W19 — The Crossroads Light
    SceneDef {
        beats: &["From the lantern gallery, in clear air, you can see every \
             road you didn't take — briefly, before the haze closes. The \
             keeper says everyone looks. The keeper says nobody looks \
             twice."],
        colors: &[],
        payout: pay(18),
    },
    // W20 — The Whale Roads
    SceneDef {
        beats: &[
            "Something vast keeps pace off the port side for three days. \
             It surfaces at dawn and dusk like a courtesy, and its eye, \
             when it rolls your way, holds the specific patience of a \
             grandmother watching a child carry something breakable.",
        ],
        colors: &[],
        payout: ScenePayout {
            provisions: 18,
            hope: 0,
            rumor: None,
            keepsake: Some("a knot of whale-road baleen"),
        },
    },
    // W21 — The Smugglers' Slip
    SceneDef {
        beats: &[
            "A harbor that isn't on your chart because it pays not to be. \
             Nobody asks your business; a price list is slid across the \
             counter face-down. The stores are excellent. The receipts do \
             not exist.",
        ],
        colors: &[],
        payout: pay(26),
    },
    // W22 — Drift's End
    SceneDef {
        beats: &[
            "The current sets you down like a valet and withdraws. The \
             sudden stillness rings. Ahead the water is black, unmoving, \
             and — the crew stops pretending otherwise — listening.",
            "The yard here doesn't ask where you're bound. They just check \
             every seam twice.",
        ],
        colors: &[],
        payout: pay(20),
    },
    // W23 — The Threshold
    SceneDef {
        beats: &[
            "The last place a letter can reach you, or leave you. A \
             postmaster with a candle-lit desk takes the crew's envelopes \
             one by one, stamping each like an oath.",
            "Write what you need to write. The dark ahead doesn't forward \
             mail.",
        ],
        colors: &[(
            ColorKey::TrimIs(Trim::Mourn),
            "You post the letters you have been carrying since launch — \
             the ones addressed to people who will never read them. The \
             postmaster stamps those too, without being asked.",
        )],
        payout: pay(22),
    },
    // W24 — The Last Lantern
    SceneDef {
        beats: &[
            "A lamp on a pole in the middle of nothing. Someone keeps it \
             lit: there is a rowboat, a tent on a raft, a woman who nods \
             at you as if you were the evening ferry, dead on time.",
            "She sells oil, stories, and the last stores this side of the \
             dark. She does not sell the answer to 'why do you stay.'",
        ],
        colors: &[],
        payout: pay(28),
    },
    // W25 — The Silence
    SceneDef {
        beats: &[
            "Sound stops working. Not muffled — declined. The crew writes \
             notes and holds them up; the hull's creak arrives a half \
             second late, like a translation. Whatever lives here is not \
             hostile. It is grading you on comportment.",
        ],
        colors: &[],
        payout: pay(14),
    },
    // W26 — The Choir of Bones
    SceneDef {
        beats: &[
            "In the silence, something sings anyway — a structure of white \
             spires that takes the wind (there is no wind) and turns it \
             into the alto line of a hymn nobody aboard learned and \
             everybody aboard knows.",
        ],
        colors: &[(
            ColorKey::SoulAboard(SoulId(4)),
            "Sefa listens for a long time and then says, wet-eyed, 'That's \
             ours. That's the office. How does it know the office?'",
        )],
        payout: pay(18),
    },
    // W27 — The Cold Shelf
    SceneDef {
        beats: &["Ice where no ice should be, growing in shelves from water \
             that refuses to explain itself. The stores frost inside the \
             hold. Conversation frosts too — the cold gets into sentences \
             and shortens them."],
        colors: &[],
        payout: pay(15),
    },
    // W28 — The Sleepers' Trench
    SceneDef {
        beats: &[
            "Ships at anchor in the dark, crews asleep in their bunks for \
             years, breathing slow as tides. Your lamplight crosses their \
             faces and they smile, all of them, at whatever they are \
             sailing in there.",
            "One bunk is warm and freshly empty.",
        ],
        colors: &[],
        payout: pay(18),
    },
    // W29 — The Ember Hold
    SceneDef {
        beats: &["A caldera harbor, warm as bathwater, forge-light on the \
             cliffs. The deep's one hearth. Smiths who talk to hulls, \
             kitchens that refuse to hear about rationing, beds that are \
             beds."],
        colors: &[
            (
                ColorKey::Drifted,
                "After the dark and the empty hold, the crew stands in the \
                 forge-light a long time before anyone remembers to tie up.",
            ),
            (
                ColorKey::ArrivedBy(RoadId(34)),
                "The smiths ask, carefully, whether the sleepers still \
                 smile. When you say yes, the whole yard relaxes: the warm \
                 bunk found its way home after all.",
            ),
        ],
        payout: ScenePayout {
            provisions: 25,
            hope: 1,
            rumor: None,
            keepsake: Some("an ember-hold coal that never quite cools"),
        },
    },
    // W30 — The Starless Reach
    SceneDef {
        beats: &[
            "No stars, no bottom, no horizon. Only the heading you chose, \
             and the wake proving you ever moved at all. The crew invents \
             constellations on the cabin ceiling and navigates by those.",
        ],
        colors: &[(
            ColorKey::SoulAboard(SoulId(0)),
            "Torvald stands the dark watches like a man keeping an old \
             appointment. 'Told you,' he says to it, quietly. 'Blinked.'",
        )],
        payout: pay(15),
    },
    // W31 — The Gloaming Shallows
    SceneDef {
        beats: &["Soft luminous shoals, blue-green light rising through the \
             water like a held breath released. Fish drift the glow in \
             slow congregations. The deep, being gentle — once, on \
             purpose, for you."],
        colors: &[],
        payout: pay_hope(20, 1),
    },
    // W32 — Deepgate
    SceneDef {
        beats: &[
            "A rift in the dark, tall as weather, and through it: green \
             light. Actual green. Leaf green. The crew lines the rail in \
             silence while the Vessel threads the gate, and the dark lets \
             you go with something that might be a bow.",
        ],
        colors: &[],
        payout: pay_hope(18, 1),
    },
    // W33 — First Light
    SceneDef {
        beats: &[
            "Dawn, after a chapter without one. It comes up gold through \
             water gone clear as air, and the crew — every soul, unasked, \
             off-watch or not — comes up on deck to watch it happen.",
        ],
        colors: &[(
            ColorKey::HopeBelow(5),
            "Somewhere in the light, the weight of the dark crossing slides \
             off the ship like water. Not all of it. Enough.",
        )],
        payout: pay_hope(20, 1),
    },
    // W34 — The Root Shallows
    SceneDef {
        beats: &[
            "Roots the size of rivers glow beneath the hull, running with \
             slow gold light, all of them going the same direction you \
             are. You are close. The water tastes faintly of orchards.",
        ],
        colors: &[],
        payout: pay(20),
    },
    // W35 — The Bloom Fields
    SceneDef {
        beats: &["Flowers on the water for miles, closed like fists — until \
             the bow touches the first rank and they open, row on row, \
             ahead of you, a wake in reverse. The Vessel arrives \
             everywhere here already announced."],
        colors: &[],
        payout: ScenePayout {
            provisions: 18,
            hope: 1,
            rumor: None,
            keepsake: Some("a bloom-field flower, pressed and still faintly warm"),
        },
    },
    // W36 — The Thorn Run
    SceneDef {
        beats: &[
            "The fast way in: a corridor of root-thorns, each the size of \
             a mast, close enough to read the grain. The water runs quick \
             and the margins run out.",
        ],
        colors: &[],
        payout: pay(5), // the run gives nothing; the ledger speaks
    },
    // W37 — The Roots of Light
    SceneDef {
        beats: &[
            "The Tree.",
            "After everything — the drowned parishes, the fair, the \
             silence, the dark and the dawn after it — the Tree, whole and \
             green and impossibly patient, roots wide as the harbor that \
             the roots themselves have grown into.",
            "Lines are thrown. Hands catch them. There are hands to catch \
             them: lit windows in the root-walls, smoke from galley fires, \
             a bell. The crossing is over. The arrival has just begun.",
        ],
        colors: &[],
        payout: pay_hope(0, 2),
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vessel::route::{Feature, WAYPOINTS};

    #[test]
    fn every_waypoint_has_an_authored_scene() {
        assert_eq!(SCENES.len(), WAYPOINTS.len());
        for (i, scene) in SCENES.iter().enumerate() {
            assert!(!scene.beats.is_empty(), "waypoint {i} has no beats");
            for beat in scene.beats {
                assert!(!beat.is_empty());
            }
        }
    }

    #[test]
    fn payouts_follow_the_economy_table_shape() {
        for w in &WAYPOINTS {
            let payout = scene_def(w.id).payout;
            if w.has_feature(Feature::WayStation) {
                assert!(
                    payout.provisions >= 25,
                    "{} is a way-station and should provision well",
                    w.name
                );
            }
            if w.has_feature(Feature::RestStop) {
                assert!(
                    payout.provisions >= 15,
                    "{} is a rest stop and should provision",
                    w.name
                );
            }
            assert!(payout.provisions <= 30, "{} over-provisions", w.name);
            assert!(payout.hope.abs() <= 2, "{} moves hope too hard", w.name);
        }
    }

    #[test]
    fn threat_waypoints_give_nearly_nothing() {
        // The reef (W9) and the thorn run (W36): the ledger speaks, not
        // the pantry.
        for id in [9u16, 36] {
            assert!(scene_def(WaypointId(id)).payout.provisions <= 5);
        }
    }

    #[test]
    fn recovery_scenes_exist_for_all_chapters() {
        for chapter in [
            Chapter::Shallows,
            Chapter::DriftRoads,
            Chapter::StarlessDeep,
            Chapter::RootsOfLight,
        ] {
            let (title, body) = recovery_scene(chapter);
            assert!(!title.is_empty() && !body.is_empty());
        }
    }

    #[test]
    fn keepsakes_are_authored_and_bounded() {
        let count = SCENES
            .iter()
            .filter(|s| s.payout.keepsake.is_some())
            .count();
        assert!((5..=12).contains(&count), "got {count} keepsakes");
    }
}
