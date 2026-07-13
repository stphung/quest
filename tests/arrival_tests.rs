//! Integration tests for The Arrival (spec 7): the staged finale, the
//! manifest's memory, and the Act 3 gate.

use chrono::{DateTime, Duration, Utc};
use quest::vessel::junction::current_junction_cards;
use quest::vessel::route::{roads_from, WaypointId, ROUTE_SINK, ROUTE_START};
use quest::vessel::scenes::{finale_carved_beat, scene_def, FINALE_HARBOR, FINALE_LAMP};
use quest::vessel::souls::{soul, SoulId};
use quest::vessel::voyage::{SoulStatus, VoyageState};

fn t0() -> DateTime<Utc> {
    "2026-07-03T12:00:00Z".parse().unwrap()
}

/// Sail the cheapest selectable road at every junction, answering every
/// door, until the Tree. Nobody is stationed, so the Thorns (if crossed)
/// cost stores, never a soul — the crossing is lossless by construction.
fn cross() -> VoyageState {
    let mut v = VoyageState::begin("arrival".to_string(), 7, t0());
    v.intro_pending = false;
    let mut now = t0();
    let mut guard = 0;
    while !v.arrived() {
        guard += 1;
        assert!(guard < 10_000, "stuck in {:?}", v.phase);
        if v.current_waypoint().is_some() {
            v.play_arrival_scene();
            v.take_letter_events();
            if v.pending_ask.is_some() && !v.accept_ask() {
                v.decline_ask();
            }
            if v.pending_refit.is_some() {
                v.choose_refit(true);
            }
            let cards = current_junction_cards(&v);
            if !cards.is_empty() && !v.arrived() {
                let road = cards.iter().find(|c| c.selectable).unwrap().road.id;
                v.depart(road).unwrap();
            }
        }
        now += Duration::hours(6);
        v.tick(now);
    }
    v
}

#[test]
fn the_finale_reads_the_whole_crossing() {
    let mut v = cross();
    let play = v.take_finale_playback().expect("the finale plays");

    // W37's authored beats read first, unchanged.
    let w37 = scene_def(ROUTE_SINK);
    for (i, beat) in w37.beats.iter().enumerate() {
        assert_eq!(play.paragraphs[i], *beat, "beat {i} is W37's");
    }

    // The final chapter's close beat leads the ceremony (release polish).
    assert_eq!(
        play.paragraphs[w37.beats.len()],
        quest::vessel::scenes::chapter_close_beat(quest::vessel::route::Chapter::RootsOfLight),
        "the Roots of Light close beat follows W37's own beats"
    );

    // The rail: one line per soul aboard, in boarding (roster) order.
    let aboard: Vec<SoulId> = v
        .souls
        .iter()
        .filter(|s| s.status == SoulStatus::Aboard)
        .map(|s| s.soul)
        .collect();
    assert!(aboard.len() >= 3, "the launch trio crossed");
    let mut cursor = w37.beats.len() + 1;
    for id in &aboard {
        assert_eq!(
            play.paragraphs[cursor],
            soul(*id).rail_line,
            "{} speaks at the rail in boarding order",
            soul(*id).name
        );
        cursor += 1;
    }

    // A lossless crossing reads no carved-names beat — absence of grief
    // is a shorter scene, not an announcement.
    assert!(v.carved_names().is_empty());
    assert!(
        !play
            .paragraphs
            .iter()
            .any(|p| p.contains("names are carved")),
        "no memorial on a lossless crossing"
    );

    // The harbor, then the lamp, and nothing after the lamp.
    assert_eq!(play.paragraphs[cursor], FINALE_HARBOR);
    assert_eq!(play.paragraphs[cursor + 1], FINALE_LAMP);
    assert_eq!(play.paragraphs.len(), cursor + 2);

    // The latch holds, and holds through save/load.
    assert!(v.take_finale_playback().is_none(), "shown exactly once");
    let mut loaded: VoyageState =
        serde_json::from_str(&serde_json::to_string(&v).unwrap()).unwrap();
    assert!(loaded.finale_shown);
    assert!(loaded.take_finale_playback().is_none());
    assert!(loaded.arrived(), "the harbor keeps");
}

#[test]
fn the_carved_names_read_at_the_rail() {
    let mut v = cross();
    // A soul was lost on this crossing (staged directly: the engine's only
    // real path is the Thorns ledger, covenant-tested in souls_tests).
    let lost = v
        .souls
        .iter()
        .find(|s| s.status == SoulStatus::Aboard)
        .map(|s| s.soul)
        .unwrap();
    assert!(v.mark_lost(lost));

    let play = v.take_finale_playback().expect("the finale plays");
    let carved = finale_carved_beat(&[soul(lost).name]);
    assert!(
        play.paragraphs.contains(&carved),
        "the memorial reads within sight of the Tree"
    );
    assert!(
        !play.paragraphs.contains(&soul(lost).rail_line.to_string()),
        "the lost do not speak at the rail"
    );
    // The memorial reads after the rail and before the harbor.
    let carved_at = play.paragraphs.iter().position(|p| *p == carved).unwrap();
    let harbor_at = play
        .paragraphs
        .iter()
        .position(|p| *p == FINALE_HARBOR)
        .unwrap();
    assert!(carved_at < harbor_at);
    assert_eq!(
        play.paragraphs.last().map(String::as_str),
        Some(FINALE_LAMP),
        "the lamp closes the act"
    );
}

#[test]
fn the_manifest_remembers_where_souls_left() {
    // A decline records the waypoint the door closed at.
    let mut v = VoyageState::begin("left-at".to_string(), 5, t0());
    v.intro_pending = false;
    v.play_arrival_scene();
    v.depart(roads_from(ROUTE_START).next().unwrap().id)
        .unwrap();
    v.tick(t0() + Duration::days(2)); // arrive W1 — Maren asks
    v.play_arrival_scene();
    let declined = v.decline_ask().expect("Maren's ask waits at W1");
    let s = v.soul_state(declined).unwrap();
    assert_eq!(s.status, SoulStatus::Declined);
    assert_eq!(s.left_at, Some(WaypointId(1)));

    // A farewell records where the soul stepped ashore.
    assert!(v.farewell(SoulId(0)));
    let s = v.soul_state(SoulId(0)).unwrap();
    assert_eq!(s.status, SoulStatus::Ashore);
    assert_eq!(s.left_at, Some(WaypointId(1)));
}

#[test]
fn old_voyage_saves_load_without_leaving_places() {
    let v = VoyageState::begin("compat".to_string(), 1, t0());
    let mut json: serde_json::Value = serde_json::to_value(&v).unwrap();
    for soul in json["souls"].as_array_mut().unwrap() {
        soul.as_object_mut().unwrap().remove("left_at");
    }
    let loaded: VoyageState = serde_json::from_value(json).unwrap();
    assert!(loaded.souls.iter().all(|s| s.left_at.is_none()));
}

#[test]
fn the_act_three_gate_defaults_closed_and_persists() {
    // Old saves (no field) load with the gate closed.
    let state = quest::core::GameState::new("gate".to_string(), 0);
    let mut json = serde_json::to_value(&state).unwrap();
    json.as_object_mut().unwrap().remove("vessel_arrived");
    let loaded: quest::core::GameState = serde_json::from_value(json).unwrap();
    assert!(!loaded.vessel_arrived);

    // Once open, it stays open through save/load.
    let mut state = quest::core::GameState::new("gate".to_string(), 0);
    state.vessel_arrived = true;
    let loaded: quest::core::GameState =
        serde_json::from_value(serde_json::to_value(&state).unwrap()).unwrap();
    assert!(loaded.vessel_arrived);
}

#[test]
fn the_log_keeps_time() {
    let v = cross();
    // Every line the crossing writes knows its day, and the days never
    // run backwards.
    assert!(!v.log.is_empty());
    let mut last_day = 0;
    for entry in &v.log {
        assert!(entry.day >= last_day, "the log reads in order");
        last_day = entry.day;
    }
    assert_eq!(v.log.first().unwrap().day, 0, "the pier is day 0");
    assert!(last_day >= 20, "the crossing took real time");
    // And the days survive save/load.
    let loaded: VoyageState = serde_json::from_str(&serde_json::to_string(&v).unwrap()).unwrap();
    assert_eq!(loaded.log, v.log);
}

#[test]
fn every_soul_has_a_rail_line() {
    for def in &quest::vessel::souls::SOULS {
        assert!(
            !def.rail_line.is_empty(),
            "{} has nothing to say at the rail",
            def.name
        );
        assert!(
            def.rail_line.contains(def.name),
            "{}'s rail line names them",
            def.name
        );
    }
}
