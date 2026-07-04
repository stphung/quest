//! Integration tests for Letters From Home & the Going-Dark (spec 6).

use chrono::{DateTime, Duration, Utc};
use quest::vessel::junction::current_junction_cards;
use quest::vessel::letters::{LAST_LETTER_EVENT, MAIL_FAILS_EVENT};
use quest::vessel::route::{roads_from, Chapter, WaypointId, ROUTE_START};
use quest::vessel::voyage::{Trim, VoyageState};

fn t0() -> DateTime<Utc> {
    "2026-07-03T12:00:00Z".parse().unwrap()
}

/// Real durations in which N game-days / -hours / -minutes pass under the
/// voyage time scale — these tests assert exact points in *game* time.
fn gd(d: i64) -> Duration {
    quest::vessel::voyage::real_duration_for_game_minutes(d * 1440)
}
fn gh(h: i64) -> Duration {
    quest::vessel::voyage::real_duration_for_game_minutes(h * 60)
}
fn gm(m: i64) -> Duration {
    quest::vessel::voyage::real_duration_for_game_minutes(m)
}

/// Walk the cheapest route to the Tree, answering every door, collecting
/// the letter events as they surface.
fn cross_collecting() -> (VoyageState, Vec<(WaypointId, Vec<u8>)>) {
    let mut v = VoyageState::begin("letters".to_string(), 13, t0());
    v.intro_pending = false;
    let mut now = t0();
    let mut deliveries = Vec::new();
    let mut guard = 0;
    while !v.arrived() {
        guard += 1;
        assert!(guard < 10_000, "stuck in {:?}", v.phase);
        if let Some(at) = v.current_waypoint() {
            v.play_arrival_scene();
            let events = v.take_letter_events();
            if !events.is_empty() {
                deliveries.push((at, events));
            }
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
        now += gh(6);
        v.tick(now);
    }
    (v, deliveries)
}

#[test]
fn the_mail_flows_then_stops_forever() {
    let (v, deliveries) = cross_collecting();

    // Letters at Chapter I/II arrivals, in sequence from letter 0.
    let mut expected_index = 0u8;
    let mut saw_last_letter = false;
    let mut saw_mail_fails = false;
    for (at, events) in &deliveries {
        let chapter = quest::vessel::route::waypoint(*at).chapter;
        for e in events {
            match *e {
                LAST_LETTER_EVENT => {
                    assert_eq!(*at, WaypointId(23), "the Last Letter is the Threshold's");
                    saw_last_letter = true;
                }
                MAIL_FAILS_EVENT => {
                    assert_eq!(chapter, Chapter::StarlessDeep);
                    assert!(saw_last_letter, "the mail fails after the Last Letter");
                    assert!(!saw_mail_fails, "the beat fires exactly once");
                    saw_mail_fails = true;
                }
                i => {
                    assert!(
                        chapter <= Chapter::DriftRoads,
                        "ordinary mail stops at Ch II"
                    );
                    assert_eq!(i, expected_index, "letters arrive in sequence");
                    expected_index += 1;
                }
            }
        }
        if saw_mail_fails {
            assert!(
                chapter >= Chapter::StarlessDeep,
                "nothing delivers after the dark"
            );
        }
    }
    assert!(expected_index >= 5, "the safe chapters carry real mail");
    assert!(saw_last_letter, "the Threshold hands over the Last Letter");
    assert!(saw_mail_fails, "the night the mail does not come, comes");
    assert!(v.gone_dark, "and it is permanent");
    // No letters of any kind delivered in Chapter IV.
    for (at, events) in &deliveries {
        if quest::vessel::route::waypoint(*at).chapter == Chapter::RootsOfLight {
            panic!("mail delivered at {at:?} past the dark: {events:?}");
        }
    }
}

#[test]
fn letters_pay_their_parcel_and_hope_exactly_once() {
    // The mail catches up at ports of call, not at the launch pier: the
    // first letter waits at the first arrival.
    let mut v = VoyageState::begin("parcel".to_string(), 5, t0());
    v.intro_pending = false;
    v.play_arrival_scene();
    assert!(v.take_letter_events().is_empty(), "no mail at the pier");
    // Post the whole trio so no arc beats fire mid-leg: the only hope
    // movement left is the letter's.
    use quest::vessel::souls::{SoulId, Station};
    v.set_station(SoulId(0), Some(Station::Helm));
    v.set_station(SoulId(1), Some(Station::Tender));
    v.set_station(SoulId(2), Some(Station::Watch));
    v.depart(roads_from(ROUTE_START).next().unwrap().id)
        .unwrap();
    v.hope = 7;
    v.tick(t0() + gd(2)); // arrive W1
    assert_eq!(v.hope, 8, "letter 0 lifts hope on arrival");
    assert_eq!(v.letters_received, 1);
    let events = v.take_letter_events();
    assert_eq!(events, vec![0]);
    assert!(v.take_letter_events().is_empty(), "drained once");
    assert!(v.log.iter().any(|l| l.text.contains("A letter from")));
}

#[test]
fn the_mail_thins_past_twelve() {
    let mut v = VoyageState::begin("thin".to_string(), 5, t0());
    v.intro_pending = false;
    v.letters_received = 12;
    v.play_arrival_scene();
    v.depart(roads_from(ROUTE_START).next().unwrap().id)
        .unwrap();
    v.tick(t0() + gd(2)); // arrive W1
    assert!(
        v.take_letter_events().is_empty(),
        "arrivals past twelve get nothing — the mail is thinning"
    );
    assert_eq!(v.letters_received, 13, "the count still advances");
}

#[test]
fn gone_dark_survives_reload_and_the_mail_never_resumes() {
    let (v, _) = cross_collecting();
    let json = serde_json::to_string(&v).unwrap();
    let loaded: VoyageState = serde_json::from_str(&json).unwrap();
    assert!(loaded.gone_dark);
    assert_eq!(loaded.letters_received, v.letters_received);
}

#[test]
fn offline_equivalence_holds_with_the_mail_in_play() {
    let build = || {
        let mut v = VoyageState::begin("mail-eq".to_string(), 29, t0());
        v.intro_pending = false;
        v.play_arrival_scene();
        v.set_trim(Trim::Quiet);
        v.depart(roads_from(ROUTE_START).next().unwrap().id)
            .unwrap();
        v
    };
    let horizon = t0() + gd(8);
    let mut live = build();
    let mut now = t0();
    while now < horizon {
        now += gm(211);
        live.tick(now.min(horizon));
    }
    let mut offline = build();
    offline.tick(horizon);

    assert_eq!(live.phase, offline.phase);
    assert_eq!(live.provisions.to_bits(), offline.provisions.to_bits());
    assert_eq!(live.hope, offline.hope);
    assert_eq!(live.letters_received, offline.letters_received);
    assert_eq!(live.letter_events, offline.letter_events);
    assert_eq!(live.log, offline.log);
}

#[test]
fn old_saves_load_with_the_mail_unstarted() {
    let v = VoyageState::begin("compat".to_string(), 1, t0());
    let mut json: serde_json::Value = serde_json::to_value(&v).unwrap();
    let obj = json.as_object_mut().unwrap();
    obj.remove("letters_received");
    obj.remove("gone_dark");
    obj.remove("letter_events");
    let loaded: VoyageState = serde_json::from_value(json).unwrap();
    assert_eq!(loaded.letters_received, 0);
    assert!(!loaded.gone_dark);
    assert!(loaded.letter_events.is_empty());
}
