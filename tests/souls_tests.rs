//! Integration tests for the Act 2 souls engine (spec 3): the roster
//! invariants, berths, arcs on rest days, the wind, and the covenant.

use chrono::{DateTime, Duration, Utc};
use quest::vessel::route::{self, roads_from, WaypointId, ROUTE_START};
use quest::vessel::souls::{self, SoulId, Station, ARC_BEAT_REST_DAYS, BERTHS, SOULS};
use quest::vessel::voyage::{SoulStatus, VoyagePhase, VoyageState};

fn t0() -> DateTime<Utc> {
    "2026-07-03T12:00:00Z".parse().unwrap()
}

fn started() -> VoyageState {
    let mut v = VoyageState::begin("souls-test".to_string(), 5, t0());
    v.intro_pending = false;
    v.play_arrival_scene();
    v
}

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
fn every_route_meets_every_recruitable_soul() {
    // The per-soul cut: each recruit's sites cover every route family —
    // different scenes, same person. This is content-parity rule 5 with
    // teeth.
    for route in all_routes() {
        for def in SOULS.iter().filter(|s| !s.sites.is_empty()) {
            assert!(
                def.sites.iter().any(|site| route.contains(site)),
                "route {route:?} never meets {}",
                def.name
            );
        }
    }
}

#[test]
fn eight_asks_against_seven_berths_forces_exactly_one_choice() {
    let mut v = started();
    assert_eq!(v.aboard_count(), 3, "the launch trio");

    // Meet the five recruits (engine-level: stage each ask directly).
    let recruits: Vec<SoulId> = SOULS
        .iter()
        .filter(|s| !s.sites.is_empty())
        .map(|s| s.id)
        .collect();
    for (i, id) in recruits.iter().enumerate() {
        v.pending_ask = Some(*id);
        if i < 4 {
            assert!(v.accept_ask(), "berths 4..=7 accept freely");
        } else {
            // The eighth ask: berths are full.
            assert!(!v.accept_ask(), "the 8th ask cannot board a full ship");
            assert!(v.pending_ask.is_some(), "the ask still waits");
            // Depart is blocked until it is answered.
            let road = roads_from(ROUTE_START).next().unwrap();
            assert!(v.depart(road.id).is_err());
            // A farewell frees the berth; the ask then boards.
            assert!(v.farewell(SoulId(0)), "Torvald steps ashore");
            assert!(v.accept_ask());
        }
    }
    assert_eq!(v.aboard_count(), BERTHS);
    assert_eq!(
        v.soul_state(SoulId(0)).unwrap().status,
        SoulStatus::Ashore,
        "farewell is permanent and remembered"
    );
    assert_eq!(v.hope, 6, "a farewell costs one hope");
}

#[test]
fn declining_is_permanent_and_the_ask_never_returns() {
    let mut v = started();
    v.depart(roads_from(ROUTE_START).next().unwrap().id)
        .unwrap();
    v.tick(t0() + Duration::days(2));
    v.play_arrival_scene();
    assert_eq!(v.pending_ask, Some(SoulId(3)), "Maren asks at the Vigil");
    v.decline_ask();
    assert_eq!(
        v.soul_state(SoulId(3)).unwrap().status,
        SoulStatus::Declined
    );
    // Re-arriving anywhere never re-asks a met soul (the door closed).
    assert!(v.pending_ask.is_none());
}

#[test]
fn arcs_advance_on_rest_days_and_pause_on_post() {
    let mut v = started();
    // Torvald stands the helm; Eir and Runa rest.
    assert!(v.set_station(SoulId(0), Some(Station::Helm)));

    // Hold station for 3 days: the resting souls' first beats (ready from
    // boarding) fire after 2 rest days; Torvald's does not.
    v.tick(t0() + Duration::days(3));
    assert_eq!(
        v.soul_state(SoulId(0)).unwrap().arc_beat,
        0,
        "a soul on post is not resting"
    );
    assert_eq!(
        v.soul_state(SoulId(1)).unwrap().arc_beat,
        1,
        "Eir's beat fired"
    );
    assert_eq!(
        v.soul_state(SoulId(2)).unwrap().arc_beat,
        1,
        "Runa's beat fired"
    );
    let events = v.take_soul_events();
    assert_eq!(events.len(), 2, "both beats queued as log moments");

    // Relieve Torvald: his beat fires after two further rest days.
    assert!(v.set_station(SoulId(0), None));
    v.tick(t0() + Duration::days(3 + ARC_BEAT_REST_DAYS as i64));
    assert_eq!(v.soul_state(SoulId(0)).unwrap().arc_beat, 1);

    // Beats pay hope: 7 -> capped rises by the fired payouts.
    assert!(v.hope > 7, "beats raise hope (got {})", v.hope);
}

#[test]
fn one_soul_per_post_and_posts_swap_cleanly() {
    let mut v = started();
    assert!(v.set_station(SoulId(0), Some(Station::Helm)));
    assert!(
        v.set_station(SoulId(1), Some(Station::Helm)),
        "Eir bumps Torvald"
    );
    assert_eq!(v.station_soul(Station::Helm), Some(SoulId(1)));
    assert_eq!(
        v.soul_state(SoulId(0)).unwrap().station,
        None,
        "bumped back to rest"
    );
}

#[test]
fn stations_change_the_composed_prices() {
    let mut v = started();
    let road = route::road(quest::vessel::route::RoadId(3)); // 16 provisions, 1.5 days
    let base_price = v.road_price(road);
    assert_eq!(base_price, 16);

    // Eir (Tender-affine) at the Tender: -10%.
    assert!(v.set_station(SoulId(1), Some(Station::Tender)));
    assert_eq!(v.road_price(road), 14, "16 x 0.90 rounds to 14");

    // Torvald (Helm-affine) at the Helm: legs run faster.
    assert!(v.set_station(SoulId(0), Some(Station::Helm)));
    let eta_helm = {
        let mut w = v.clone();
        w.depart(route::roads_from(ROUTE_START).next().unwrap().id)
            .unwrap();
        w.eta_minutes().unwrap()
    };
    let eta_bare = {
        let mut w = v.clone();
        w.set_station(SoulId(0), None);
        w.depart(route::roads_from(ROUTE_START).next().unwrap().id)
            .unwrap();
        w.eta_minutes().unwrap()
    };
    assert!(eta_helm < eta_bare, "an affine helm shortens the leg");
}

#[test]
fn hope_is_the_wind_and_the_long_silence_breaks_at_a_hearth() {
    // Two identical ships, different hope: the hopeful one arrives first.
    let leg = |hope: u8| {
        let mut v = started();
        v.hope = hope;
        v.depart(roads_from(ROUTE_START).next().unwrap().id)
            .unwrap();
        v.tick(t0() + Duration::hours(26));
        matches!(v.phase, VoyagePhase::HoldingStation { .. })
    };
    assert!(
        leg(9),
        "high hope: the 1.0-day leg is done inside 26h (0.9x)"
    );
    assert!(!leg(2), "guttering hope: the same leg drags (1.25x)");

    // The Long Silence: hope hits ashen, legs crawl, arcs pause — until a
    // rest-stop hearth breaks it.
    let mut v = started();
    v.hope = 1;
    // A farewell at hope 1 lands on ashen and the silence falls.
    assert!(v.farewell(SoulId(2)));
    assert_eq!(v.hope, 0);
    assert!(v.long_silence);

    // Arcs pause in the silence.
    v.tick(t0() + Duration::days(4));
    assert_eq!(v.soul_state(SoulId(0)).unwrap().arc_beat, 0, "arcs paused");

    // Sail to the Kelp Meadows (W2 -> W5 is a RestStop) — via W1/W2.
    v.depart(roads_from(ROUTE_START).next().unwrap().id)
        .unwrap();
    v.tick(t0() + Duration::days(7)); // slow legs under the silence
    v.play_arrival_scene();
    if v.pending_ask.is_some() {
        v.decline_ask(); // Maren asks at W1; the silence declines
    }
    v.depart(roads_from(WaypointId(1)).next().unwrap().id)
        .unwrap();
    v.tick(t0() + Duration::days(10));
    v.play_arrival_scene();
    assert_eq!(v.current_waypoint(), Some(WaypointId(2)));
    // W2 -> W5 (the Kelp Meadows, a RestStop).
    v.depart(quest::vessel::route::RoadId(3)).unwrap();
    v.tick(t0() + Duration::days(16));
    assert_eq!(v.current_waypoint(), Some(WaypointId(5)));
    assert!(!v.long_silence, "the hearth breaks the silence");
    // The break restores hope to "low" (3); arcs resume immediately and any
    // beats that fire in the same stretch raise it further.
    assert!(v.hope >= 3, "hope came back (got {})", v.hope);
    assert!(
        v.souls.iter().any(|s| s.arc_beat > 0),
        "arcs resumed after the silence"
    );
}

#[test]
fn the_covenant_no_offline_stretch_touches_the_roster() {
    // Hold, travel, and drift for 60 days without a single player action:
    // the roster count and statuses never change, and no loss fires.
    let mut v = started();
    v.provisions = 5.0; // guarantees a drift mid-leg
    let statuses: Vec<_> = v.souls.iter().map(|s| (s.soul, s.status)).collect();
    v.depart(roads_from(ROUTE_START).next().unwrap().id)
        .unwrap();
    v.tick(t0() + Duration::days(60));
    let after: Vec<_> = v.souls.iter().map(|s| (s.soul, s.status)).collect();
    // Maren's ask may be pending (that is a door, not a change) — but the
    // met roster is byte-identical.
    assert_eq!(statuses, after, "offline time must never touch the roster");
    assert!(
        v.carved_names().is_empty(),
        "no loss without an authored scene"
    );
}

#[test]
fn offline_equivalence_holds_with_arcs_and_wind_in_play() {
    let build = || {
        let mut v = started();
        v.set_station(SoulId(0), Some(Station::Helm));
        v.depart(roads_from(ROUTE_START).next().unwrap().id)
            .unwrap();
        v
    };
    let horizon = t0() + Duration::days(9);

    let mut live = build();
    let mut now = t0();
    while now < horizon {
        now += Duration::minutes(137);
        live.tick(now.min(horizon));
    }
    let mut offline = build();
    offline.tick(horizon);

    assert_eq!(live.phase, offline.phase);
    assert_eq!(live.provisions.to_bits(), offline.provisions.to_bits());
    assert_eq!(live.hope, offline.hope);
    assert_eq!(live.souls, offline.souls);
    assert_eq!(live.soul_events, offline.soul_events);
}

#[test]
fn old_voyage_saves_load_with_the_launch_trio() {
    // A pre-souls voyage.json has none of the new fields: it must load with
    // the launch roster and no pending state — existing crossings continue.
    let v = started();
    let mut json: serde_json::Value = serde_json::to_value(&v).unwrap();
    let obj = json.as_object_mut().unwrap();
    obj.remove("souls");
    obj.remove("pending_ask");
    obj.remove("long_silence");
    obj.remove("soul_events");
    let loaded: VoyageState = serde_json::from_value(json).unwrap();
    assert_eq!(loaded.aboard_count(), 3);
    assert!(loaded.pending_ask.is_none());
    assert!(!loaded.long_silence);
    let names: Vec<_> = loaded.aboard().map(|s| souls::soul(s.soul).name).collect();
    assert_eq!(names, vec!["Torvald", "Eir", "Runa"]);
}
