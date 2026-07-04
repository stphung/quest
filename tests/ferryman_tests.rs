//! Integration tests for The Ferryman (spec 9): the crossing loop, the
//! colony's growth, Drive compounding, and the era ending.

use chrono::{DateTime, Duration, Utc};
use quest::vessel::colony::{ColonyState, District};
use quest::vessel::junction::current_junction_cards;
use quest::vessel::souls::{SoulId, Station};
use quest::vessel::voyage::{SoulStatus, VoyageState};

fn t0() -> DateTime<Utc> {
    "2026-07-03T12:00:00Z".parse().unwrap()
}

/// Sail a whole crossing to the Tree, staffing the crew, and return its
/// day count and the finale.
fn cross(v: &mut VoyageState) -> u64 {
    v.set_station(SoulId(0), Some(Station::Helm));
    v.set_station(SoulId(1), Some(Station::Tender));
    v.set_station(SoulId(2), Some(Station::Watch));
    let mut now = t0();
    let mut guard = 0;
    while !v.arrived() {
        guard += 1;
        assert!(guard < 10_000, "stuck at {:?}", v.phase);
        if v.current_waypoint().is_some() {
            v.play_arrival_scene();
            if v.pending_ask.is_some() && !v.accept_ask() {
                v.decline_ask();
            }
            if v.pending_refit.is_some() {
                v.choose_refit(true);
            }
            let cards = current_junction_cards(v);
            if !cards.is_empty() && !v.arrived() {
                let road = cards.iter().find(|c| c.selectable).unwrap().road.id;
                v.depart(road).unwrap();
            }
        }
        now += Duration::hours(6);
        v.tick(now);
    }
    v.day_index()
}

/// Play a full era: crossing after crossing until the world empties.
/// Returns (crossings, first-crossing days, last-crossing days, delivered).
fn run_era() -> (u32, u64, u64, u64) {
    let mut colony = ColonyState::found("era".to_string());
    let mut crew: Vec<quest::vessel::voyage::SoulState> = Vec::new();
    let mut first_days = 0;
    let mut last_days;
    let mut guard = 0;
    loop {
        guard += 1;
        assert!(guard < 500, "era never ended");
        let n = colony.crossings_completed;
        let mut v = if n == 0 {
            VoyageState::begin("era".to_string(), 7, t0())
        } else {
            VoyageState::begin_ferry(
                "era".to_string(),
                colony.era_seed ^ n as u64,
                t0(),
                &colony,
                crew.clone(),
            )
        };
        v.intro_pending = false;
        let days = cross(&mut v);
        if n == 0 {
            first_days = days;
        }
        last_days = days;
        crew = v
            .souls
            .iter()
            .filter(|s| s.status == SoulStatus::Aboard)
            .cloned()
            .collect();
        let delivered = if v.crossing_number == 1 {
            v.aboard_count() as u32
        } else {
            v.passengers
        };
        colony.deliver_crossing(delivered, days, days * 10, days);
        if colony.era_over() {
            return (
                colony.crossings_completed,
                first_days,
                last_days,
                colony.souls_delivered,
            );
        }
    }
}

#[test]
fn the_first_crossing_founds_the_colony_and_delivers_its_crew() {
    let mut v = VoyageState::begin("found".to_string(), 7, t0());
    v.intro_pending = false;
    cross(&mut v);
    let survivors = v.aboard_count() as u32;
    assert!(survivors >= 3, "the launch trio at least came ashore");

    let mut colony = ColonyState::found("found".to_string());
    colony.deliver_crossing(survivors, v.day_index(), 200, 12);
    assert_eq!(colony.souls_delivered, survivors as u64);
    assert_eq!(colony.crossings_completed, 1);
    assert!(colony.drive > 0, "delivery grows drive");
}

#[test]
fn a_ferry_run_carries_passengers_and_the_rested_crew() {
    let mut colony = ColonyState::found("ferry".to_string());
    // 8,000 delivered: the population term, plus the Quay (500) and
    // Granary (3,500) bonuses — Hearth (10k) not yet.
    colony.souls_delivered = 8_000;
    colony.drive = 8_000;
    let expected_capacity =
        160 + (8 * quest::vessel::colony::EXPEDITION_PER_1000_DELIVERED) as u32 + 110 + 140;
    // A crew with a strained soul (spec 8) — coming home is rest.
    let mut crew: Vec<_> = {
        let mut v = VoyageState::begin("ferry".to_string(), 1, t0());
        v.intro_pending = false;
        v.souls
    };
    crew[2].strain = 2;

    let v = VoyageState::begin_ferry("ferry".to_string(), 5, t0(), &colony, crew);
    assert_eq!(v.crossing_number, 2);
    assert_eq!(
        v.passengers, expected_capacity,
        "base plus every founded district's bonus"
    );
    assert!(v.drive_time_mult < 1.0, "drive sails her faster");
    assert!(
        v.souls.iter().all(|s| s.strain == 0),
        "the crew came home to rest"
    );
    // Ferry runs get no authored recruit asks.
    let mut v = v;
    v.set_station(SoulId(0), Some(Station::Helm));
    v.set_station(SoulId(1), Some(Station::Tender));
    let mut now = t0();
    for _ in 0..300 {
        v.play_arrival_scene();
        assert!(v.pending_ask.is_none(), "no pilgrims board a ferry run");
        if v.pending_refit.is_some() {
            v.choose_refit(true);
        }
        let cards = current_junction_cards(&v);
        if !cards.is_empty() && !v.arrived() {
            let road = cards.iter().find(|c| c.selectable).unwrap().road.id;
            v.depart(road).unwrap();
        }
        now += Duration::hours(12);
        v.tick(now);
        if v.arrived() {
            break;
        }
    }
    assert!(v.arrived(), "the ferry run reaches the Tree");
}

#[test]
fn a_long_era_ferries_most_of_a_huge_world_across_growing_crossings() {
    let (crossings, first_days, last_days, delivered) = run_era();
    eprintln!("era: {crossings} crossings, {first_days}d -> {last_days}d, {delivered} delivered");
    // A long campaign of many crossings — you feel the loop — carrying most
    // of a huge world home, the expedition growing the whole way.
    assert!(
        (25..=60).contains(&crossings),
        "the era is a long run of many crossings ({crossings})"
    );
    assert!(
        delivered >= 75_000,
        "the colony saves ~80% of the 100k world ({delivered} delivered)"
    );
    assert!(
        last_days < first_days,
        "drive still trims each crossing ({first_days}d -> {last_days}d)"
    );

    // Prove the loop feels like progress the whole way: the expedition grows
    // on essentially every crossing (the population engine), all six districts
    // are founded, and they land spread across the era — not all at once.
    let mut colony = ColonyState::found("districts".to_string());
    let mut founded_on = Vec::new();
    let mut sizes = Vec::new();
    let mut crossing = 0;
    while !colony.era_over() {
        crossing += 1;
        let carried = if crossing == 1 {
            6
        } else {
            colony.next_expedition()
        };
        sizes.push(colony.expedition_size());
        let new = colony.deliver_crossing(carried, 32, 320, 10);
        for d in new {
            founded_on.push((colony.crossings_completed, d));
        }
    }
    assert_eq!(
        founded_on.len(),
        6,
        "all six districts founded: {founded_on:?}"
    );
    assert!(
        colony.has_district(District::Charthouse),
        "the Charthouse (pop 66k) is reached near the finale"
    );
    // Districts are spread out, not bunched — at least four distinct crossings.
    let distinct: std::collections::HashSet<u32> = founded_on.iter().map(|(c, _)| *c).collect();
    assert!(
        distinct.len() >= 4,
        "milestones are spread across the era: {founded_on:?}"
    );
    // The expedition never shrinks — every crossing is at least as big as the
    // last — and grows dramatically from launch to finale.
    assert!(
        sizes.windows(2).all(|w| w[1] >= w[0]),
        "the expedition grows (or holds) every crossing"
    );
    assert!(
        *sizes.last().unwrap() > sizes[1] * 10,
        "the ferry ends far larger than it began ({} -> {})",
        sizes[1],
        sizes.last().unwrap()
    );
}

#[test]
fn the_colony_grows_through_its_districts() {
    let mut colony = ColonyState::found("grow".to_string());
    assert!(colony.districts().is_empty());
    // Deliver until the Hearth (pop 10,000).
    while !colony.has_district(District::Hearth) {
        colony.souls_remaining = colony.souls_remaining.max(600);
        colony.deliver_crossing(600, 30, 300, 10);
    }
    assert!(colony.has_district(District::Quay));
    assert!(colony.has_district(District::Granary));
    assert!(colony.has_district(District::Hearth));
}

#[test]
fn old_saves_have_no_colony_and_resume_on_crossing_one() {
    // A pre-ferryman voyage has crossing_number defaulting to 1.
    let v = VoyageState::begin("compat".to_string(), 1, t0());
    let mut json: serde_json::Value = serde_json::to_value(&v).unwrap();
    let obj = json.as_object_mut().unwrap();
    obj.remove("crossing_number");
    obj.remove("passengers");
    obj.remove("drive_time_mult");
    let loaded: VoyageState = serde_json::from_value(json).unwrap();
    assert_eq!(loaded.crossing_number, 1);
    assert_eq!(loaded.passengers, 0);
    assert_eq!(loaded.drive_time_mult, 1.0);
}

#[test]
fn the_ferry_loop_is_offline_equivalent() {
    let colony = {
        let mut c = ColonyState::found("eq".to_string());
        c.souls_delivered = 500;
        c.drive = 600;
        c
    };
    let crew = {
        let mut v = VoyageState::begin("eq".to_string(), 1, t0());
        v.intro_pending = false;
        v.souls
    };
    let build = || {
        let mut v = VoyageState::begin_ferry("eq".to_string(), 9, t0(), &colony, crew.clone());
        v.set_station(SoulId(0), Some(Station::Helm));
        v.play_arrival_scene();
        v.depart(
            quest::vessel::route::roads_from(quest::vessel::route::ROUTE_START)
                .next()
                .unwrap()
                .id,
        )
        .unwrap();
        v
    };
    let horizon = t0() + Duration::days(8);
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
}
