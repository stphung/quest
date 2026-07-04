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

/// The "balanced" line the campaign is tuned around: spend the Salvage in
/// hand keeping the two yards in step — Drive when it's behind or level,
/// otherwise the hold. Mirrors how a player steering the middle would spend.
fn balanced_spend(c: &mut ColonyState) {
    loop {
        let bought = if c.drive_level <= c.cap_level && c.salvage >= c.drive_cost() {
            c.buy_drive()
        } else if c.salvage >= c.cap_cost() {
            c.buy_capacity()
        } else if c.salvage >= c.drive_cost() {
            c.buy_drive()
        } else {
            false
        };
        if !bought {
            break;
        }
    }
}

#[allow(dead_code)]
fn drive_only_spend(c: &mut ColonyState) {
    while c.buy_drive() {}
}
#[allow(dead_code)]
fn cap_only_spend(c: &mut ColonyState) {
    while c.buy_capacity() {}
}
/// Souls-first optimal line: empty the world in the fewest crossings (each
/// crossing is a dark toll), so lean into the hold — but keep just enough
/// Drive that the crossings still turn around fast. Two hold levels per Drive.
#[allow(dead_code)]
fn cap_lean_spend(c: &mut ColonyState) {
    loop {
        let bought = if c.cap_level <= c.drive_level * 2 && c.salvage >= c.cap_cost() {
            c.buy_capacity()
        } else if c.salvage >= c.drive_cost() {
            c.buy_drive()
        } else if c.salvage >= c.cap_cost() {
            c.buy_capacity()
        } else {
            false
        };
        if !bought {
            break;
        }
    }
}

#[test]
#[ignore = "tuning sweep — run with --ignored --nocapture"]
fn strategy_sweep() {
    let scale = quest::vessel::voyage::time_scale();
    for (name, spend) in [
        ("drive-only", drive_only_spend as fn(&mut ColonyState)),
        ("cap-only", cap_only_spend),
        ("balanced", balanced_spend),
        ("cap-lean (souls-first)", cap_lean_spend),
    ] {
        let (crossings, first, _last, delivered, total) = run_era_with(spend);
        eprintln!(
            "{name:>24}: {crossings:>3} crossings  {:>4.1} mo  {:>4.1}% saved  (C1 {:.0}d)",
            (total as f64 / scale) / 30.0,
            delivered as f64 / 1000.0,
            first as f64 / scale,
        );
    }
}

/// Sail a whole crossing to the Tree, staffing the crew, and return its
/// day count and the finale.
fn cross(v: &mut VoyageState) -> u64 {
    use quest::vessel::voyage::real_duration_for_game_minutes;
    v.set_station(SoulId(0), Some(Station::Helm));
    v.set_station(SoulId(1), Some(Station::Tender));
    v.set_station(SoulId(2), Some(Station::Watch));
    // The maiden voyage (crossing 1) is navigated by the ferryman; ferry runs
    // (crossing 2+) navigate themselves, so we only make the choices on the
    // first. Step in fine game-minute increments so the measured day count is
    // the real one, not a coarse-stepping artifact.
    let maiden = v.crossing_number == 1;
    let mut now = t0();
    let mut guard = 0;
    while !v.arrived() {
        guard += 1;
        assert!(guard < 200_000, "stuck at {:?}", v.phase);
        if maiden && v.current_waypoint().is_some() {
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
        now += real_duration_for_game_minutes(60);
        v.tick(now);
    }
    v.day_index()
}

/// Play a full era: crossing after crossing until the world empties.
/// Returns (crossings, first-crossing days, last-crossing days, delivered).
fn run_era() -> (u32, u64, u64, u64) {
    let (c, f, l, d, _) = run_era_with(balanced_spend);
    (c, f, l, d)
}

/// Play a full era under an arbitrary spend policy. Returns
/// (crossings, first-days, last-days, delivered, total-game-days).
fn run_era_with(spend: fn(&mut ColonyState)) -> (u32, u64, u64, u64, u64) {
    let mut colony = ColonyState::found("era".to_string());
    let mut crew: Vec<quest::vessel::voyage::SoulState> = Vec::new();
    let mut first_days = 0;
    let mut last_days;
    let mut total_days = 0u64;
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
        total_days += days;
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
        if std::env::var("RAMPDBG").is_ok() {
            eprintln!(
                "  C{:>2}: {:>4.1} real-days  (drive Lv{} cap Lv{})  carried {}",
                colony.crossings_completed + 1,
                days as f64 / quest::vessel::voyage::time_scale(),
                colony.drive_level,
                colony.cap_level,
                delivered
            );
        }
        colony.deliver_crossing(delivered, days, days * 10, days);
        // On arrival, spend the crossing's Salvage in the yards.
        spend(&mut colony);
        if colony.era_over() {
            return (
                colony.crossings_completed,
                first_days,
                last_days,
                colony.souls_delivered,
                total_days,
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
    assert!(
        colony.salvage > quest::vessel::colony::STARTING_SALVAGE,
        "the maiden landfall pays out Salvage on top of the founding grant"
    );
}

#[test]
fn a_ferry_run_carries_passengers_and_the_rested_crew() {
    let mut colony = ColonyState::found("ferry".to_string());
    // 8,000 delivered founds the Quay (500) and Granary (3,500), not yet the
    // Hearth (10k). Three Shipwright levels widen the base hold; the two
    // founded districts add their standing bonuses on top. Some Drive too.
    colony.souls_delivered = 8_000;
    colony.cap_level = 3;
    colony.drive_level = 2;
    let widened = (f64::from(quest::vessel::colony::BASE_CAPACITY)
        * quest::vessel::colony::CAP_GROWTH.powi(3))
    .round() as u32;
    let expected_capacity = widened + 110 + 140;
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
fn an_era_ferries_most_of_the_world_across_a_ramping_run_of_crossings() {
    let (crossings, first_days, last_days, delivered) = run_era();
    eprintln!("era: {crossings} crossings, {first_days}d -> {last_days}d, {delivered} delivered");
    // The tuned campaign: a long run of crossings you feel the ramp in — a
    // two-week maiden voyage that builds up over the first handful, then a fast
    // fun stretch of quick turnarounds while the loads climb.
    assert!(
        (15..=30).contains(&crossings),
        "the era is a long, felt run of crossings ({crossings})"
    );
    assert!(
        delivered >= 78_000,
        "the balanced line still carries most of the 100k world ({delivered} delivered)"
    );
    assert!(
        last_days < first_days,
        "drive trims each crossing — the maiden voyage is the slowest ({first_days}d -> {last_days}d)"
    );

    // Prove the loop feels like progress the whole way: playing the balanced
    // line (Salvage spent on both yards each arrival), the hold grows across
    // the era, all six districts are founded, and they land spread out.
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
        balanced_spend(&mut colony);
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
    // Districts are spread out, not bunched — at least three distinct crossings.
    let distinct: std::collections::HashSet<u32> = founded_on.iter().map(|(c, _)| *c).collect();
    assert!(
        distinct.len() >= 3,
        "milestones are spread across the era: {founded_on:?}"
    );
    // The hold never shrinks — the Shipwright only ever widens her — and grows
    // dramatically from launch to finale as Salvage is spent.
    assert!(
        sizes.windows(2).all(|w| w[1] >= w[0]),
        "the hold grows (or holds) every crossing: {sizes:?}"
    );
    assert!(
        *sizes.last().unwrap() > sizes[1] * 5,
        "the ferry ends far larger than it began ({} -> {})",
        sizes[1],
        sizes.last().unwrap()
    );
}

#[test]
fn skilled_play_saves_far_more_souls_than_reckless_play() {
    // The design intent, as a gate: because the dark takes a toll every
    // crossing, leaning into the hold (the souls-first line) saves most of the
    // world, while chasing pure speed runs dozens of near-empty crossings and
    // the dark bleeds it white. The margin is meant to be *wide* — skill is
    // rewarded, not marginal.
    let (_, _, _, souls_first, _) = run_era_with(cap_lean_spend);
    let (_, _, _, reckless, _) = run_era_with(drive_only_spend);
    eprintln!("souls-first {souls_first} vs reckless {reckless}");
    assert!(
        souls_first >= 82_000,
        "the souls-first line carries most of the world home ({souls_first})"
    );
    assert!(
        reckless <= 68_000,
        "the reckless drive-only line bleeds the world to the dark ({reckless})"
    );
    assert!(
        souls_first >= reckless + 18_000,
        "skilled play saves far more — a wide margin ({souls_first} vs {reckless})"
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
        c.drive_level = 2;
        c.cap_level = 1;
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
