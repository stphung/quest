//! Integration tests for the Act 2 voyage engine (spec 2): whole crossings
//! through the real route graph, and the offline-equivalence covenant across
//! a save/load boundary.

use chrono::{DateTime, Duration, Utc};
use quest::vessel::junction::current_junction_cards;
use quest::vessel::persistence::{load_voyage_from_path, save_voyage_to_path};
use quest::vessel::route::{self, ROUTE_SINK, ROUTE_START};
use quest::vessel::voyage::{Trim, VoyagePhase, VoyageState};

fn t0() -> DateTime<Utc> {
    "2026-07-03T12:00:00Z".parse().unwrap()
}

/// Play the crossing with a given road-picking strategy: always play the
/// arrival scene, depart immediately, check in every 6 hours. Returns the
/// finished state and the days it took.
fn cross(mut pick: impl FnMut(&VoyageState) -> usize, trim: Trim) -> (VoyageState, u64) {
    let mut v = VoyageState::begin("crosser".to_string(), 99, t0());
    v.set_trim(trim);
    let mut now = t0();
    let mut checkins = 0u32;
    while !v.arrived() {
        checkins += 1;
        assert!(
            checkins < 10_000,
            "crossing did not finish; stuck in {:?}",
            v.phase
        );
        if v.play_arrival_scene().is_some() || v.current_waypoint().is_some() {
            if v.pending_refit.is_some() {
                v.choose_refit(true);
            }
            // Answer any boarding ask: yes while a berth is free, no after
            // (asks block departure until answered).
            if v.pending_ask.is_some() && !v.accept_ask() {
                v.decline_ask();
            }
            let cards = current_junction_cards(&v);
            if !cards.is_empty() && !v.arrived() {
                let selectable: Vec<_> = cards.iter().filter(|c| c.selectable).collect();
                assert!(
                    !selectable.is_empty(),
                    "no selectable road at {:?} with {} provisions — the \
                     affordability invariant failed",
                    v.current_waypoint(),
                    v.provisions_display()
                );
                let choice = pick(&v).min(selectable.len() - 1);
                v.depart(selectable[choice].road.id).unwrap();
            }
        }
        now += Duration::hours(6);
        v.tick(now);
    }
    (v.clone(), v.day_index())
}

#[test]
fn cheapest_road_crossing_always_reaches_the_tree() {
    let (v, days) = cross(|_| 0, Trim::Cruise);
    assert!(v.arrived());
    assert_eq!(*v.visited.last().unwrap(), ROUTE_SINK);
    // ~24 waypoints of 1.0-3.0 day legs: an attended crossing lands well
    // inside the pacing envelope.
    assert!(
        (25..=120).contains(&days),
        "crossing took {days} days, outside the envelope"
    );
    // Doors closed behind us at every junction.
    assert!(!v.untaken.is_empty());
    // The keepsake spine is a real route.
    assert_eq!(v.visited[0], ROUTE_START);
    for pair in v.visited.windows(2) {
        assert!(
            route::roads_from(pair[0]).any(|r| r.to == pair[1]),
            "visited spine skips from {:?} to {:?}",
            pair[0],
            pair[1]
        );
    }
}

#[test]
fn priciest_road_crossing_survives_by_drifting() {
    // Always taking the most expensive selectable road runs the hold down;
    // drift recovery must carry the crossing anyway. "You cannot lose the
    // crossing" is structural, not luck.
    let (v, days) = cross(
        |s| current_junction_cards(s).len().saturating_sub(1),
        Trim::Run,
    );
    assert!(v.arrived());
    assert!(
        (20..=160).contains(&days),
        "run-hard crossing took {days} days"
    );
}

#[test]
fn mourn_crossing_is_slow_and_reaches_the_tree() {
    let (v, days) = cross(|_| 0, Trim::Mourn);
    assert!(v.arrived());
    // Restful is the slowest pace (x1.40) — its identity is thrift, not speed.
    assert!(
        days >= 30,
        "mourn crossing was implausibly fast: {days} days"
    );
}

#[test]
fn offline_equivalence_across_a_save_load_boundary() {
    // The spec's load-bearing property: N days lazy-ticked on load equals the
    // same N days ticked live — including through save/load, bitwise.
    let build = || {
        let mut v = VoyageState::begin("offline".to_string(), 7, t0());
        v.set_trim(Trim::Quiet);
        v.play_arrival_scene();
        v.depart(route::roads_from(ROUTE_START).next().unwrap().id)
            .unwrap();
        v
    };
    let horizon = t0() + Duration::days(7);

    // Live: ticked every 100 "game minutes" for a week.
    let mut live = build();
    let mut now = t0();
    while now < horizon {
        now += Duration::minutes(100);
        live.tick(now.min(horizon));
    }

    // Offline: saved untouched, loaded a week later, ticked once.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("voyage.json");
    let offline = build();
    save_voyage_to_path(&offline, &path).unwrap();
    let mut offline = load_voyage_from_path(&path, "offline").unwrap();
    offline.tick(horizon);

    assert_eq!(live.phase, offline.phase);
    assert_eq!(live.provisions.to_bits(), offline.provisions.to_bits());
    assert_eq!(live.visited, offline.visited);
    assert_eq!(live.processed_minutes, offline.processed_minutes);
}

#[test]
fn a_week_away_never_harms_the_voyage_beyond_its_prices() {
    // The covenant: holding station costs nothing — a month at anchor moves
    // nothing, and the ship still departs fine afterward.
    let mut v = VoyageState::begin("away".to_string(), 3, t0());
    v.play_arrival_scene();
    v.tick(t0() + Duration::days(30));
    assert_eq!(
        v.provisions_display(),
        100,
        "holding never burns provisions"
    );
    assert!(matches!(v.phase, VoyagePhase::HoldingStation { .. }));
    let road = route::roads_from(ROUTE_START).next().unwrap();
    assert!(v.depart(road.id).is_ok());
}
