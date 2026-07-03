//! Integration tests for the Underway layer (spec 5): weather, nights and
//! the watch, pilgrims, and the determinism covenant with all of it live.

use chrono::{DateTime, Duration, Utc};
use quest::vessel::nights::NightKind;
use quest::vessel::pilgrims::{pilgrim_road, PILGRIMS};
use quest::vessel::route::{roads_from, RoadId, ROUTE_START};
use quest::vessel::souls::{SoulId, Station};
use quest::vessel::voyage::{SoulStatus, Trim, VoyagePhase, VoyageState};
use quest::vessel::weather::{self, WeatherKind};

fn t0() -> DateTime<Utc> {
    "2026-07-03T12:00:00Z".parse().unwrap()
}

fn started(seed: u64) -> VoyageState {
    let mut v = VoyageState::begin("underway".to_string(), seed, t0());
    v.intro_pending = false;
    v.play_arrival_scene();
    v
}

/// Stage a voyage mid-leg on a specific road at a specific processed time.
fn traveling_on(seed: u64, road: RoadId, minutes: u64) -> VoyageState {
    let mut v = started(seed);
    v.processed_minutes = minutes;
    v.phase = VoyagePhase::Traveling {
        road,
        departed_at_min: minutes,
        progress_days: 0.0,
    };
    v
}

#[test]
fn nights_resolve_at_sea_and_write_the_log() {
    // Sail the two-day gateway leg with nobody on watch: every typed night
    // costs its unstood price and writes a line.
    let mut v = started(3);
    v.depart(roads_from(ROUTE_START).next().unwrap().id)
        .unwrap();
    let log_before = v.log.len();
    v.tick(t0() + Duration::days(1) + Duration::hours(2));
    // Day 0's night resolved (typed or quiet). Typed nights log; quiet
    // nights stay silent. Either way the sequence was consulted:
    let kind = v.night_kind(0);
    if kind.needs_watch() {
        assert!(v.log.len() > log_before, "a typed night writes the log");
    }
}

#[test]
fn a_watch_affine_stander_is_kinder_than_nobody() {
    // Find a seed whose first night at sea is hungry or cold, then sail it
    // twice: unstood vs stood by Runa (Watch-affine).
    let seed = (0..200u64)
        .find(|s| {
            matches!(
                quest::vessel::nights::raw_night_kind(*s, 0),
                NightKind::Cold | NightKind::Hungry
            )
        })
        .expect("some seed opens cold or hungry");

    let sail = |watch: bool| {
        let mut v = started(seed);
        if watch {
            v.set_station(SoulId(2), Some(Station::Watch));
        }
        v.depart(roads_from(ROUTE_START).next().unwrap().id)
            .unwrap();
        v.tick(t0() + Duration::days(1) + Duration::hours(6));
        v.provisions
    };
    assert!(
        sail(true) > sail(false),
        "Runa on watch spares the stores a typed night costs"
    );
}

#[test]
fn the_strange_cap_holds_per_leg() {
    // With a strange night already seen this leg, a raw-strange day
    // resolves quiet; a new leg resets the cap.
    let strange_day = (0..500u64)
        .find(|d| quest::vessel::nights::raw_night_kind(11, *d) == NightKind::Strange)
        .expect("a strange day exists");
    let mut v = started(11);
    v.strange_seen_this_leg = true;
    assert_eq!(v.night_kind(strange_day), NightKind::Quiet, "capped");
    v.depart(roads_from(ROUTE_START).next().unwrap().id)
        .unwrap();
    assert!(!v.strange_seen_this_leg, "a new leg resets the cap");
    assert_eq!(v.night_kind(strange_day), NightKind::Strange);
}

#[test]
fn a_squall_taxes_the_hold_and_shelter_halves_it() {
    // Find a (seed, hour) with a squall somewhere, then sail that road
    // under it at Cruise vs Quiet.
    let (seed, hour, road) = (0..40u64)
        .flat_map(|seed| (0..1200u64).map(move |h| (seed, h)))
        .find_map(|(seed, hour)| {
            weather::weather_at(seed, hour)
                .into_iter()
                .find(|o| o.kind == WeatherKind::Squall && o.hours_left > 30)
                .map(|o| (seed, hour, o.road))
        })
        .expect("a squall exists somewhere");

    let burn_at = |trim: Trim| {
        let mut v = traveling_on(seed, road, hour * 60);
        v.set_trim(trim);
        let before = v.provisions;
        v.tick(t0() + Duration::minutes((hour * 60 + 12 * 60) as i64));
        before - v.provisions
    };
    let cruise = burn_at(Trim::Cruise);
    let quiet = burn_at(Trim::Quiet);
    assert!(
        cruise > quiet,
        "shelter trims spend less under a squall ({cruise:.1} vs {quiet:.1})"
    );
}

#[test]
fn quiet_trim_hears_a_silence_bank_once() {
    let (seed, hour, road) = (0..60u64)
        .flat_map(|seed| (0..1600u64).map(move |h| (seed, h)))
        .find_map(|(seed, hour)| {
            weather::weather_at(seed, hour)
                .into_iter()
                .find(|o| o.kind == WeatherKind::SilenceBank && o.hours_left > 30)
                .map(|o| (seed, hour, o.road))
        })
        .expect("a silence-bank exists somewhere");

    let mut v = traveling_on(seed, road, hour * 60);
    v.set_trim(Trim::Quiet);
    let rumors_before = v.rumors.len();
    v.tick(t0() + Duration::minutes((hour * 60 + 120) as i64));
    assert!(!v.heard_banks.is_empty(), "Quiet heard the bank");
    assert!(v.rumors.len() > rumors_before, "and it paid in knowledge");
    // Un-Quiet, the same bank frays hope instead.
    let mut w = traveling_on(seed, road, hour * 60);
    w.set_trim(Trim::Cruise);
    w.hope = 7;
    w.tick(t0() + Duration::minutes((hour * 60 + 30 * 60) as i64));
    assert!(
        w.silence_minutes > 0 || w.hope < 7,
        "the silence pressed in"
    );
}

#[test]
fn hailing_a_pilgrim_happens_once_and_pays_in_rumors() {
    // Day 0: the Sister Verity sails her first scripted road.
    let verity_road = pilgrim_road(&PILGRIMS[0], 0).unwrap();
    let mut v = traveling_on(5, verity_road, 30);
    let ship = v.hailable().expect("the Verity shares this road");
    assert_eq!(ship.id, 0);
    let rumors_before = v.rumors.len();
    assert!(v.hail().is_some());
    assert!(v.rumors.len() > rumors_before);
    assert!(v.hailable().is_none(), "once per ship");
    assert!(v.hail().is_none());
    assert!(v.log.iter().any(|l| l.text().contains("Sister Verity")));
}

#[test]
fn offline_equivalence_holds_with_the_whole_underway_layer() {
    // The load-bearing property, now with weather, nights, watch overrides,
    // and pilgrims in play: chunked live ticks == one offline tick, bitwise.
    let build = || {
        let mut v = started(17);
        v.set_station(SoulId(0), Some(Station::Helm));
        v.set_station(SoulId(2), Some(Station::Watch));
        v.set_trim(Trim::Quiet);
        v.depart(roads_from(ROUTE_START).next().unwrap().id)
            .unwrap();
        v.assign_night(1, Some(SoulId(1)));
        v
    };
    let horizon = t0() + Duration::days(11);

    let mut live = build();
    let mut now = t0();
    while now < horizon {
        now += Duration::minutes(173);
        live.tick(now.min(horizon));
    }
    let mut offline = build();
    offline.tick(horizon);

    assert_eq!(live.phase, offline.phase);
    assert_eq!(live.provisions.to_bits(), offline.provisions.to_bits());
    assert_eq!(live.hope, offline.hope);
    assert_eq!(live.log, offline.log);
    assert_eq!(live.rumors, offline.rumors);
    assert_eq!(live.heard_banks, offline.heard_banks);
    assert_eq!(live.souls, offline.souls);
}

#[test]
fn the_covenant_extends_to_travel() {
    // Weather, nights, silence, drift — 90 days of it, unattended. No
    // sequence of offline world ever injures, removes, or advances-to-loss
    // a soul; the worst is priced provisions/hope.
    let mut v = started(23);
    v.provisions = 8.0; // guarantee drifts along the way
    let roster_before: Vec<_> = v.souls.iter().map(|s| (s.soul, s.status)).collect();
    v.depart(roads_from(ROUTE_START).next().unwrap().id)
        .unwrap();
    v.tick(t0() + Duration::days(90));
    let roster_after: Vec<_> = v.souls.iter().map(|s| (s.soul, s.status)).collect();
    assert_eq!(roster_before, roster_after);
    assert!(v.carved_names().is_empty());
    assert!(
        v.souls.iter().all(|s| s.status == SoulStatus::Aboard),
        "everyone is still aboard"
    );
}
