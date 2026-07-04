//! Integration tests for The Price of Passage (spec 8): hope sinks,
//! the strain ledger, hull wear, and the yard's third door.

use chrono::{DateTime, Duration, Utc};
use quest::vessel::route::{roads_from, RoadId, WaypointId, ROUTE_START};
use quest::vessel::souls::{SoulId, Station};
use quest::vessel::voyage::{
    PassageEvent, SceneState, SoulStatus, StrainCause, Trim, VoyagePhase, VoyageState,
    HOPE_SPEND_FLOOR, PRESS_HOPE_COST,
};

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

// ── Press the Helm ──────────────────────────────────────────────────────────

#[test]
fn pressing_spends_hope_and_shortens_the_leg_once() {
    let mut v = underway(5);
    v.tick(t0() + gh(12));
    let hope0 = v.hope;
    let eta0 = v.eta_minutes().unwrap();

    assert!(v.can_press());
    assert!(v.press_helm());
    assert_eq!(v.hope, hope0 - PRESS_HOPE_COST);
    let eta1 = v.eta_minutes().unwrap();
    assert!(
        (eta1 as f64) < eta0 as f64 * 0.90,
        "the remaining leg shortens ~15% ({eta0} -> {eta1})"
    );
    // Once per leg, and the hint gate agrees.
    assert!(!v.can_press());
    assert!(!v.press_helm());
}

#[test]
fn pressing_needs_hope_to_spend() {
    let mut v = underway(5);
    v.tick(t0() + gh(6));
    v.hope = HOPE_SPEND_FLOOR - 1;
    assert!(!v.can_press());
    assert!(!v.press_helm(), "you cannot buy your way into the silence");
}

#[test]
fn the_second_press_in_a_chapter_strains_the_helm() {
    let mut v = underway(5);
    v.set_station(SoulId(0), Some(Station::Helm));
    v.tick(t0() + gh(6));
    assert!(v.press_helm());
    assert_eq!(
        v.soul_state(SoulId(0)).unwrap().strain,
        0,
        "one press is free"
    );

    // Arrive, depart the next leg (same chapter), press again.
    let mut now = t0() + gh(6);
    while v.current_waypoint().is_none() {
        now += gh(6);
        v.tick(now);
    }
    v.play_arrival_scene();
    if v.pending_ask.is_some() {
        v.decline_ask();
    }
    let next = roads_from(v.current_waypoint().unwrap()).next().unwrap();
    v.depart(next.id).unwrap();
    now += gh(6);
    v.tick(now);
    assert!(v.press_helm(), "a new leg allows a new press");
    assert_eq!(
        v.soul_state(SoulId(0)).unwrap().strain,
        1,
        "the second press in one chapter wears on the wheel-hand"
    );
    assert!(v.take_passage_events().iter().any(|e| matches!(
        e,
        PassageEvent::Strained {
            cause: StrainCause::PressedHard,
            ..
        }
    )));
}

// ── Hard rations ────────────────────────────────────────────────────────────

#[test]
fn hard_rations_stretch_the_hold_and_charge_hope_daily() {
    // Two identical ships, one on hard rations: compare a day's burn.
    let run = |rations: bool| {
        let mut v = underway(9);
        v.set_station(SoulId(1), Some(Station::Tender));
        if rations {
            assert!(v.set_hard_rations(true));
        }
        let p0 = v.provisions;
        let h0 = v.hope;
        v.tick(t0() + gh(26)); // past one full day underway
        (p0 - v.provisions, h0 as i32 - v.hope as i32)
    };
    let (burn_full, hope_full) = run(false);
    let (burn_hard, hope_hard) = run(true);
    assert!(
        burn_hard < burn_full * 0.80,
        "hard rations burn ~25% less ({burn_full:.2} -> {burn_hard:.2})"
    );
    // Nights can move hope on both ships (a singing night lifts it);
    // rations' price is the difference between the twins.
    assert_eq!(
        hope_hard - hope_full,
        1,
        "hard rations cost a point a day over full rations"
    );
}

#[test]
fn rations_cannot_be_asked_of_the_hopeless() {
    let mut v = underway(9);
    v.hope = HOPE_SPEND_FLOOR - 1;
    assert!(!v.set_hard_rations(true));
    v.hope = HOPE_SPEND_FLOOR;
    assert!(v.set_hard_rations(true));
    v.hope = 0;
    assert!(v.set_hard_rations(false), "turning it off is always free");
    assert!(!v.hard_rations);
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
fn old_saves_load_sound_unworn_and_fully_rationed() {
    let v = VoyageState::begin("compat".to_string(), 1, t0());
    let mut json: serde_json::Value = serde_json::to_value(&v).unwrap();
    let obj = json.as_object_mut().unwrap();
    for key in [
        "hull_wear",
        "hard_rations",
        "rations_minutes",
        "pressed_this_leg",
        "presses_this_chapter",
        "priced_squalls",
        "strained_banks",
        "passage_events",
    ] {
        obj.remove(key);
    }
    for soul in json["souls"].as_array_mut().unwrap() {
        let s = soul.as_object_mut().unwrap();
        s.remove("strain");
        s.remove("consecutive_watches");
    }
    let loaded: VoyageState = serde_json::from_value(json).unwrap();
    assert_eq!(loaded.hull_wear, 0);
    assert!(!loaded.hard_rations);
    assert!(loaded.souls.iter().all(|s| s.strain == 0));
}

#[test]
fn offline_equivalence_holds_with_the_prices_in_play() {
    let build = || {
        let mut v = underway(29);
        v.set_station(SoulId(0), Some(Station::Helm));
        v.set_hard_rations(true);
        assert!(v.press_helm() || v.hope < HOPE_SPEND_FLOOR);
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
    assert_eq!(live.hope, offline.hope);
    assert_eq!(live.hull_wear, offline.hull_wear);
    assert_eq!(live.passage_events, offline.passage_events);
    assert_eq!(
        live.souls.iter().map(|s| s.strain).collect::<Vec<_>>(),
        offline.souls.iter().map(|s| s.strain).collect::<Vec<_>>()
    );
}
