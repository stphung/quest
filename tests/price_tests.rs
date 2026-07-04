//! Integration tests for The Price of Passage (spec 8): the strain ledger,
//! hull wear, and the yard's third door.

use chrono::{DateTime, Duration, Utc};
use quest::vessel::route::{roads_from, RoadId, WaypointId, ROUTE_START};
use quest::vessel::souls::{SoulId, Station};
use quest::vessel::voyage::{PassageEvent, SceneState, SoulStatus, Trim, VoyagePhase, VoyageState};

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

fn underway(seed: u64) -> VoyageState {
    let mut v = VoyageState::begin(format!("price-{seed}"), seed, t0());
    v.intro_pending = false;
    v.play_arrival_scene();
    v.depart(roads_from(ROUTE_START).next().unwrap().id)
        .unwrap();
    v
}

// ── The strain ledger ───────────────────────────────────────────────────────

#[test]
fn worn_souls_cannot_hold_a_post_and_rest_stops_mend_the_off_post() {
    let mut v = VoyageState::begin("mend".to_string(), 3, t0());
    v.intro_pending = false;
    // Runa is worn before the ship even sails (staged directly; the
    // acquisition paths are covered below and by the simulator).
    v.souls[2].strain = 2;
    assert!(
        !v.set_station(SoulId(2), Some(Station::Watch)),
        "a worn soul cannot stand a post"
    );

    // Sail the cheapest route until a rest stop is reached: the off-post
    // heal fires on arrival (content parity guarantees rest stops).
    use quest::vessel::junction::current_junction_cards;
    let mut now = t0();
    let mut guard = 0;
    while v.soul_state(SoulId(2)).unwrap().strain == 2 {
        guard += 1;
        assert!(guard < 2_000, "no rest stop ever mended Runa");
        if v.current_waypoint().is_some() {
            v.play_arrival_scene();
            if v.pending_ask.is_some() {
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
    assert_eq!(
        v.soul_state(SoulId(2)).unwrap().strain,
        1,
        "one level per stop"
    );
    assert!(v
        .take_passage_events()
        .iter()
        .any(|e| matches!(e, PassageEvent::HealedAtRest { soul } if *soul == SoulId(2))));
    // And mended hands can post again once sound.
    v.souls[2].strain = 0;
    assert!(v.set_station(SoulId(2), Some(Station::Watch)));
}

#[test]
fn a_strained_tender_loses_the_affine_edge() {
    let mut v = VoyageState::begin("affine".to_string(), 3, t0());
    v.intro_pending = false;
    v.set_station(SoulId(1), Some(Station::Tender));
    let sound = v.provisions_mult_with(Trim::Cruise);
    v.souls[1].strain = 1;
    let strained = v.provisions_mult_with(Trim::Cruise);
    assert!(
        strained > sound,
        "Eir strained: the tender bonus falls back to unaffine ({sound:.3} -> {strained:.3})"
    );
}

// ── Hull wear ───────────────────────────────────────────────────────────────

#[test]
fn a_leg_driven_at_run_scars_the_hull_and_scars_make_her_eat() {
    let mut v = underway(7);
    v.set_trim(Trim::Run);
    let mult0 = v.provisions_mult_with(Trim::Cruise);
    let mut now = t0();
    while v.current_waypoint().is_none() {
        now += gh(6);
        v.tick(now);
    }
    assert_eq!(v.hull_wear, 1, "the leg made good at Run leaves a mark");
    assert!(v
        .take_passage_events()
        .iter()
        .any(|e| matches!(e, PassageEvent::Scarred { wear: 1, .. })));
    let mult1 = v.provisions_mult_with(Trim::Cruise);
    assert!(
        (mult1 / mult0 - 1.05).abs() < 1e-9,
        "each scar adds 5% burn ({mult0:.3} -> {mult1:.3})"
    );
}

#[test]
fn the_yard_mends_instead_of_refitting_and_the_shelf_closes() {
    let mut v = quest::fixtures::voyage_holding_at(
        "mend-yard".to_string(),
        WaypointId(4), // Graywater Anchorage — a shipyard
        8,
        80.0,
        t0(),
    );
    assert!(v.pending_refit.is_some(), "the yard makes its offer");
    v.hull_wear = 3;
    assert!(v.choose_mend());
    assert_eq!(v.hull_wear, 0, "the scars planed away");
    assert!(v.pending_refit.is_none());
    assert_eq!(v.refit_doors_seen, 1, "that yard's refit pair is spent");
    assert!(v.refits.is_empty(), "no refit was taken");
}

// ── The threat ledger reads strain ──────────────────────────────────────────

#[test]
fn the_thorns_take_the_most_strained_stationed_soul_first() {
    let stage = || {
        let mut v = VoyageState::begin("thorns".to_string(), 11, t0());
        v.intro_pending = false;
        v.phase = VoyagePhase::HoldingStation {
            waypoint: WaypointId(36),
            arrived_at_min: v.processed_minutes,
            scene_state: SceneState::Waiting,
            arrived_by: Some(RoadId(42)),
        };
        v.set_station(SoulId(0), Some(Station::Helm));
        v.set_station(SoulId(1), Some(Station::Tender));
        v
    };

    // All sound: exposure is the post order — the helm.
    let mut v = stage();
    v.play_arrival_scene().unwrap();
    assert_eq!(v.carved_names(), vec!["Torvald"]);

    // The tender strained: the thorns find the weakest hand first.
    let mut v = stage();
    v.souls[1].strain = 1;
    v.play_arrival_scene().unwrap();
    assert_eq!(
        v.carved_names(),
        vec!["Eir"],
        "the most strained stationed soul is the exposure"
    );
    assert_eq!(
        v.soul_state(SoulId(0)).unwrap().status,
        SoulStatus::Aboard,
        "the sound helm is spared"
    );
}

// ── Compat and the covenant ─────────────────────────────────────────────────

#[test]
fn old_saves_load_sound_and_unworn() {
    let v = VoyageState::begin("compat".to_string(), 1, t0());
    let mut json: serde_json::Value = serde_json::to_value(&v).unwrap();
    let obj = json.as_object_mut().unwrap();
    // Spec-8 fields absent from an older save default cleanly. Retired-hope
    // fields (hope, long_silence, hard_rations, …) are handled the other way:
    // serde ignores them if a legacy save still carries them.
    for key in [
        "hull_wear",
        "priced_squalls",
        "strained_banks",
        "passage_events",
    ] {
        obj.remove(key);
    }
    // A legacy save may still carry the now-retired hope fields; serde must
    // load past them (unknown fields are ignored, not rejected).
    obj.insert("hope".to_string(), serde_json::json!(7));
    obj.insert("long_silence".to_string(), serde_json::json!(false));
    obj.insert("hard_rations".to_string(), serde_json::json!(true));
    for soul in json["souls"].as_array_mut().unwrap() {
        let s = soul.as_object_mut().unwrap();
        s.remove("strain");
        s.remove("consecutive_watches");
    }
    let loaded: VoyageState = serde_json::from_value(json).unwrap();
    assert_eq!(loaded.hull_wear, 0);
    assert!(loaded.souls.iter().all(|s| s.strain == 0));
}

#[test]
fn offline_equivalence_holds_with_the_prices_in_play() {
    let build = || {
        let mut v = underway(29);
        v.set_station(SoulId(0), Some(Station::Helm));
        v
    };
    let horizon = t0() + gd(9);
    let mut live = build();
    let mut now = t0();
    while now < horizon {
        now += gm(199);
        live.tick(now.min(horizon));
    }
    let mut offline = build();
    offline.tick(horizon);

    assert_eq!(live.phase, offline.phase);
    assert_eq!(live.provisions.to_bits(), offline.provisions.to_bits());
    assert_eq!(live.hull_wear, offline.hull_wear);
    assert_eq!(live.passage_events, offline.passage_events);
    assert_eq!(
        live.souls.iter().map(|s| s.strain).collect::<Vec<_>>(),
        offline.souls.iter().map(|s| s.strain).collect::<Vec<_>>()
    );
}
