//! Integration tests for the Act 2 souls engine (spec 3): the roster
//! invariants, crew seats, arcs on rest days, and the covenant.

use chrono::{DateTime, Duration, Utc};
use quest::vessel::route::{self, roads_from, WaypointId, ROUTE_START};
use quest::vessel::souls::{self, SoulId, Station, ARC_BEAT_REST_DAYS, CREW, SOULS};
use quest::vessel::voyage::{SoulStatus, Trim, VoyageState};

fn t0() -> DateTime<Utc> {
    "2026-07-03T12:00:00Z".parse().unwrap()
}

/// Real durations in which N game-days / -minutes pass under the voyage time
/// scale — these tests assert exact points in *game* time.
fn gd(d: i64) -> Duration {
    quest::vessel::voyage::real_duration_for_game_minutes(d * 1440)
}
fn gm(m: i64) -> Duration {
    quest::vessel::voyage::real_duration_for_game_minutes(m)
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
fn eight_asks_against_seven_crew_seats_forces_exactly_one_choice() {
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
            assert!(v.accept_ask(), "seats 4..=7 accept freely");
        } else {
            // The eighth ask: the crew is full.
            assert!(!v.accept_ask(), "the 8th ask cannot board a full ship");
            assert!(v.pending_ask.is_some(), "the ask still waits");
            // Depart is blocked until it is answered.
            let road = roads_from(ROUTE_START).next().unwrap();
            assert!(v.depart(road.id).is_err());
            // A farewell frees a seat; the ask then boards.
            assert!(v.farewell(SoulId(0)), "Torvald steps ashore");
            assert!(v.accept_ask());
        }
    }
    assert_eq!(v.aboard_count(), CREW);
    assert_eq!(
        v.soul_state(SoulId(0)).unwrap().status,
        SoulStatus::Ashore,
        "farewell is permanent and remembered"
    );
}

#[test]
fn declining_is_permanent_and_the_ask_never_returns() {
    let mut v = started();
    v.depart(roads_from(ROUTE_START).next().unwrap().id)
        .unwrap();
    v.tick(t0() + gd(2));
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
    v.tick(t0() + gd(3));
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
    v.tick(t0() + gd(3 + ARC_BEAT_REST_DAYS as i64));
    assert_eq!(v.soul_state(SoulId(0)).unwrap().arc_beat, 1);
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
fn the_covenant_no_offline_stretch_touches_the_roster() {
    // Hold, travel, and drift for 60 days without a single player action:
    // the roster count and statuses never change, and no loss fires.
    let mut v = started();
    v.provisions = 5.0; // guarantees a drift mid-leg
    let statuses: Vec<_> = v.souls.iter().map(|s| (s.soul, s.status)).collect();
    v.depart(roads_from(ROUTE_START).next().unwrap().id)
        .unwrap();
    v.tick(t0() + gd(60));
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
fn offline_equivalence_holds_with_arcs_in_play() {
    let build = || {
        let mut v = started();
        v.set_station(SoulId(0), Some(Station::Helm));
        v.depart(roads_from(ROUTE_START).next().unwrap().id)
            .unwrap();
        v
    };
    let horizon = t0() + gd(9);

    let mut live = build();
    let mut now = t0();
    while now < horizon {
        now += gm(137);
        live.tick(now.min(horizon));
    }
    let mut offline = build();
    offline.tick(horizon);

    assert_eq!(live.phase, offline.phase);
    assert_eq!(live.provisions.to_bits(), offline.provisions.to_bits());
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
    obj.remove("soul_events");
    let loaded: VoyageState = serde_json::from_value(json).unwrap();
    assert_eq!(loaded.aboard_count(), 3);
    assert!(loaded.pending_ask.is_none());
    let names: Vec<_> = loaded.aboard().map(|s| souls::soul(s.soul).name).collect();
    assert_eq!(names, vec!["Torvald", "Eir", "Runa"]);
}

#[test]
fn threat_ledgers_are_functions_of_prior_choices() {
    use quest::vessel::route::RoadId;
    use quest::vessel::voyage::VoyagePhase;

    // Stage an arrival over the Ossuary Reef (R9) three ways.
    let stage = |via: RoadId, at: quest::vessel::route::WaypointId| {
        let mut v = started();
        v.phase = VoyagePhase::HoldingStation {
            waypoint: at,
            arrived_at_min: v.processed_minutes,
            scene_state: quest::vessel::voyage::SceneState::Waiting,
            arrived_by: Some(via),
        };
        v
    };
    let reef = quest::vessel::route::WaypointId(9);

    // Hurried and unsung: the Warden takes provisions AND scars the hull.
    let mut v = stage(RoadId(9), reef);
    v.set_trim(Trim::Run);
    let p0 = v.provisions;
    v.play_arrival_scene().unwrap();
    assert!(
        v.provisions < p0 && v.hull_wear > 0,
        "hurried: hold and hull"
    );

    // Slow and respectful: the hold pays, but the hull is spared.
    let mut v = stage(RoadId(9), reef);
    v.set_trim(Trim::Quiet);
    let p0 = v.provisions;
    v.play_arrival_scene().unwrap();
    assert!(
        v.provisions < p0 && v.hull_wear == 0,
        "respectful spares the hull"
    );

    // Sefa aboard: safe, and a keepsake.
    let mut v = stage(RoadId(9), reef);
    v.pending_ask = Some(SoulId(4));
    assert!(v.accept_ask());
    let p0 = v.provisions;
    v.play_arrival_scene().unwrap();
    assert_eq!(v.provisions, p0, "the office buys passage");
    assert!(v.keepsakes.iter().any(|k| k.contains("Warden")));
}

#[test]
fn the_thorns_take_a_stationed_soul_and_only_a_stationed_soul() {
    use quest::vessel::route::{RoadId, WaypointId};
    use quest::vessel::voyage::{SceneState, VoyagePhase};

    let thorn_run = WaypointId(36);
    let stage = || {
        let mut v = started();
        v.phase = VoyagePhase::HoldingStation {
            waypoint: thorn_run,
            arrived_at_min: v.processed_minutes,
            scene_state: SceneState::Waiting,
            arrived_by: Some(RoadId(42)),
        };
        v
    };

    // A stationed soul is exposed: helm first.
    let mut v = stage();
    v.set_station(SoulId(0), Some(Station::Helm));
    v.set_station(SoulId(1), Some(Station::Tender));
    v.play_arrival_scene().unwrap();
    assert_eq!(
        v.carved_names(),
        vec!["Torvald"],
        "the helm was the exposure"
    );
    assert_eq!(
        v.soul_state(SoulId(1)).unwrap().status,
        SoulStatus::Aboard,
        "one loss, not a massacre"
    );

    // Everyone below: the ship pays instead; nobody is taken.
    let mut v = stage();
    let p0 = v.provisions;
    v.play_arrival_scene().unwrap();
    assert!(v.carved_names().is_empty());
    assert!(v.provisions < p0);

    // Cormac at the helm: one clean line, keepsake, no loss.
    let mut v = stage();
    v.pending_ask = Some(SoulId(6));
    assert!(v.accept_ask());
    v.set_station(SoulId(6), Some(Station::Helm));
    v.play_arrival_scene().unwrap();
    assert!(v.carved_names().is_empty());
    assert!(v.keepsakes.iter().any(|k| k.contains("thorn spar")));

    // The Quiet Keel refit: the hull pays, nobody is taken.
    let mut v = stage();
    v.set_station(SoulId(0), Some(Station::Helm));
    v.refits.push(quest::vessel::refits::RefitId::QuietKeel);
    v.play_arrival_scene().unwrap();
    assert!(v.carved_names().is_empty(), "the keel held");
}

#[test]
fn refit_doors_open_at_the_first_three_shipyards_and_close_behind() {
    use quest::vessel::refits::RefitId;

    let mut v = started();
    assert!(v.pending_refit.is_none());
    // Stage three yard visits.
    for (i, pick_a) in [(0u8, true), (1, false), (2, true)] {
        v.pending_refit = Some(i);
        let pair = v.pending_refit_pair().unwrap();
        let chosen = v.choose_refit(pick_a).unwrap();
        assert_eq!(chosen, if pick_a { pair.a } else { pair.b });
        assert!(v.pending_refit.is_none());
    }
    assert_eq!(v.refits.len(), 3);
    assert_eq!(v.refit_doors_seen, 3);
    assert!(v.has_refit(RefitId::StormSail));
    assert!(v.has_refit(RefitId::DeepLarder));
    assert!(v.has_refit(RefitId::LanternMast));
    // Long Hold not taken: cap unchanged. Deep Larder taken instead.
    assert_eq!(v.provisions_cap, 100.0);
    // Storm Sail composes into leg time.
    assert!(v.time_mult() < 1.0);
}

#[test]
fn scene_payouts_apply_exactly_once() {
    let mut v = started(); // W0 scene played by started()
    let p_after_first = v.provisions;
    assert!(v.play_arrival_scene().is_none(), "scenes play once");
    assert_eq!(v.provisions, p_after_first);
    assert_eq!(v.log.len(), 1, "one log line per scene");
}
