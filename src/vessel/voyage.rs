//! The Voyage — Act 2's state machine (sub-project 2).
//!
//! One ship moves through the route graph on wall-clock time. Transitions are
//! computed lazily on tick/load (the Loom's timer pattern — no background
//! scheduling), stepped in whole *game minutes* so that processing N days in
//! one call or in many produces bitwise-identical state (the offline
//! equivalence property).
//!
//! See `docs/superpowers/specs/2026-07-03-vessel-route-waypoints-design.md`.

use super::route::{
    self, Feature, Road, RoadId, RumorId, SceneRef, WaypointId, ROUTE_SINK, ROUTE_START,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── Constants (first pass; spec 2 table) ────────────────────────────────────

pub const MINUTES_PER_DAY: u64 = 24 * 60;
/// One bar. (150 with the Long Hold refit, a spec-4 shipyard scene.)
pub const PROVISIONS_CAP: f64 = 100.0;
#[allow(dead_code)] // The Long Hold refit lands with spec 4's shipyards.
pub const LONG_HOLD_PROVISIONS_CAP: f64 = 150.0;
/// The hold is full at launch.
pub const LAUNCH_PROVISIONS: f64 = 100.0;
/// Provisions restored when a drift ends — also the affordability floor:
/// every junction's cheapest road costs no more than this (CI-asserted).
pub const DRIFT_RECOVERY_PROVISIONS: u32 = 25;
pub const DRIFT_RECOVERY_HOURS: u64 = 36;
/// Days a played arrival can be held before hope starts to fray.
pub const HOLD_STATION_GRACE_DAYS: u64 = 3;
/// Buying a rumor at a way-station (one per visit).
pub const RUMOR_PRICE: f64 = 6.0;
/// Interim economy: playing an arrival scene provisions the ship. Spec 4's
/// authored scenes replace this uniform gift with real payoffs (markets,
/// gifts, catches); the number is tuned so a Cruise crossing runs the hold
/// close to dry without leaning on drift.
pub const ARRIVAL_PROVISIONS_GIFT: f64 = 20.0;
/// Hope is a small number with a name, never a percentage.
pub const HOPE_MAX: u8 = 10;
/// Hold-station decay never drags hope below "steady" — the eager-souls rule.
pub const HOPE_FLOOR_STEADY: u8 = 5;
pub const LAUNCH_HOPE: u8 = 7;

/// Dev/test wall-clock multiplier (`QUEST_VOYAGE_TIME_SCALE`, default 1.0).
/// At 1440x a voyage "day" passes in one real minute — used by drive-game
/// fixtures and the simulator. Read once.
pub fn time_scale() -> f64 {
    static SCALE: std::sync::OnceLock<f64> = std::sync::OnceLock::new();
    *SCALE.get_or_init(|| {
        std::env::var("QUEST_VOYAGE_TIME_SCALE")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|s| s.is_finite() && *s > 0.0)
            .unwrap_or(1.0)
    })
}

/// Whole game minutes elapsed since launch at wall time `now`.
fn elapsed_game_minutes(launched_at: DateTime<Utc>, now: DateTime<Utc>) -> u64 {
    let real_ms = (now - launched_at).num_milliseconds().max(0) as f64;
    (real_ms * time_scale() / 60_000.0).floor() as u64
}

// ── Trim ────────────────────────────────────────────────────────────────────

/// The ship's one posture dial (Underway spec). Persistent until changed,
/// including offline. Weather interaction lands with sub-project 5; the base
/// multipliers are load-bearing for road pricing today.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Trim {
    Run,
    #[default]
    Cruise,
    Quiet,
    Mourn,
}

impl Trim {
    pub fn time_mult(&self) -> f64 {
        match self {
            Trim::Run => 0.80,
            Trim::Cruise => 1.00,
            Trim::Quiet => 1.20,
            Trim::Mourn => 1.40,
        }
    }

    pub fn provisions_mult(&self) -> f64 {
        match self {
            Trim::Run => 1.30,
            Trim::Cruise => 1.00,
            Trim::Quiet => 0.90,
            Trim::Mourn => 0.90,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Trim::Run => "Run",
            Trim::Cruise => "Cruise",
            Trim::Quiet => "Quiet",
            Trim::Mourn => "Mourn",
        }
    }

    pub const ALL: [Trim; 4] = [Trim::Run, Trim::Cruise, Trim::Quiet, Trim::Mourn];
}

// ── State machine ───────────────────────────────────────────────────────────

/// Whether the arrival scene at the held waypoint has been played.
/// `Waiting` blocks departure: arrivals wait for the player, always.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SceneState {
    Waiting,
    Played,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VoyagePhase {
    Traveling {
        road: RoadId,
        departed_at_min: u64,
        /// Base-days of the road covered so far (trim scales the rate).
        progress_days: f64,
    },
    Drifting {
        road: RoadId,
        progress_days: f64,
        since_min: u64,
    },
    HoldingStation {
        waypoint: WaypointId,
        arrived_at_min: u64,
        scene_state: SceneState,
    },
    Arrived {
        at_min: u64,
    },
}

/// A rumor the player holds, with provenance. Held forever.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LearnedRumor {
    pub rumor: RumorId,
    pub learned_at: WaypointId,
}

fn default_true() -> bool {
    true
}

/// The whole crossing: two gauges, a phase, a chart's worth of memory.
/// Persisted to `voyage.json` (see `vessel::persistence`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoyageState {
    /// Which character this voyage belongs to.
    pub character_id: String,
    /// Seed for the Underway spec's determinism (weather, nights, templates).
    pub voyage_seed: u64,
    pub launched_at: DateTime<Utc>,
    /// Game minutes since launch already simulated (the lazy-tick cursor).
    pub processed_minutes: u64,
    pub phase: VoyagePhase,
    /// Voyage-level posture; persists across legs and offline.
    #[serde(default)]
    pub trim: Trim,
    pub provisions: f64,
    pub hope: u8,
    /// Larger cap once the Long Hold refit exists (spec 4); saved so refits
    /// survive reload.
    #[serde(default = "default_provisions_cap")]
    pub provisions_cap: f64,
    /// Waypoints reached, in order. The keepsake chart's spine.
    pub visited: Vec<WaypointId>,
    /// Roads not taken: grayed forever, names kept, contents never expanded.
    #[serde(default)]
    pub untaken: Vec<RoadId>,
    #[serde(default)]
    pub rumors: Vec<LearnedRumor>,
    /// One rumor purchase per way-station visit.
    #[serde(default)]
    pub rumor_bought_this_visit: bool,
    /// A drift recovery scene waits to be shown (authored per chapter, spec 4).
    #[serde(default)]
    pub pending_recovery_scene: bool,
    /// Whole minutes spent traveling at Mourn since the last hope raise
    /// (integer so day boundaries are exact).
    #[serde(default)]
    pub mourn_minutes: u64,
    /// Hope already decayed during the current hold (idempotent lazy decay).
    #[serde(default)]
    pub hold_decay_applied: u32,
    /// Set false after the first-boot 5-beat transition has played.
    #[serde(default = "default_true")]
    pub intro_pending: bool,
}

fn default_provisions_cap() -> f64 {
    PROVISIONS_CAP
}

/// Why a departure was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepartError {
    /// Not holding station (still traveling, drifting, or arrived).
    NotAtStation,
    /// The arrival scene has not been played; arrivals wait for the player.
    SceneWaiting,
    /// The road does not leave the current waypoint.
    NoSuchRoad,
    /// Locked: not affordable and not the cheapest road out.
    Locked,
}

impl VoyageState {
    /// `voyage::begin` — a new crossing at the Last Harbor with a full hold,
    /// three souls' worth of hope, and everything ahead.
    pub fn begin(character_id: String, voyage_seed: u64, now: DateTime<Utc>) -> Self {
        VoyageState {
            character_id,
            voyage_seed,
            launched_at: now,
            processed_minutes: 0,
            phase: VoyagePhase::HoldingStation {
                waypoint: ROUTE_START,
                arrived_at_min: 0,
                scene_state: SceneState::Waiting,
            },
            trim: Trim::Cruise,
            provisions: LAUNCH_PROVISIONS,
            hope: LAUNCH_HOPE,
            provisions_cap: PROVISIONS_CAP,
            visited: vec![ROUTE_START],
            untaken: Vec::new(),
            rumors: Vec::new(),
            rumor_bought_this_visit: false,
            pending_recovery_scene: false,
            mourn_minutes: 0,
            hold_decay_applied: 0,
            intro_pending: true,
        }
    }

    // ── Lazy tick ───────────────────────────────────────────────────────────

    /// Advance the simulation to `now`, one whole game minute at a time.
    /// Idempotent and chunking-invariant: `tick(t2)` equals
    /// `tick(t1); tick(t2)` exactly, which is the offline covenant's
    /// load-bearing property.
    pub fn tick(&mut self, now: DateTime<Utc>) {
        let target = elapsed_game_minutes(self.launched_at, now);
        while self.processed_minutes < target {
            self.step_minute();
            self.processed_minutes += 1;
        }
    }

    /// One game minute of world. Small, total, and allocation-free.
    fn step_minute(&mut self) {
        match self.phase {
            VoyagePhase::Traveling {
                road: road_id,
                departed_at_min,
                progress_days,
            } => {
                let road = route::road(road_id);
                let dp = (1.0 / MINUTES_PER_DAY as f64) / self.trim.time_mult();
                let burn = dp
                    * (f64::from(road.base_provisions) / f64::from(road.base_days))
                    * self.trim.provisions_mult();

                if self.provisions < burn {
                    // The hold runs dry mid-road: drift where we stand.
                    self.provisions = 0.0;
                    self.phase = VoyagePhase::Drifting {
                        road: road_id,
                        progress_days,
                        since_min: self.processed_minutes,
                    };
                    return;
                }

                self.provisions -= burn;
                if self.trim == Trim::Mourn {
                    self.mourn_minutes += 1;
                    if self.mourn_minutes >= MINUTES_PER_DAY {
                        self.mourn_minutes -= MINUTES_PER_DAY;
                        self.hope = (self.hope + 1).min(HOPE_MAX);
                    }
                }

                let new_progress = progress_days + dp;
                if new_progress >= f64::from(road.base_days) {
                    self.arrive_at(road.to);
                } else {
                    self.phase = VoyagePhase::Traveling {
                        road: road_id,
                        departed_at_min,
                        progress_days: new_progress,
                    };
                }
            }
            VoyagePhase::Drifting {
                road,
                progress_days,
                since_min,
            } => {
                if self.processed_minutes - since_min >= DRIFT_RECOVERY_HOURS * 60 {
                    // Something small is caught, mended, shared. The covenant:
                    // drift prices time and pride, never souls.
                    self.provisions = f64::from(DRIFT_RECOVERY_PROVISIONS);
                    self.pending_recovery_scene = true;
                    self.phase = VoyagePhase::Traveling {
                        road,
                        departed_at_min: self.processed_minutes,
                        progress_days,
                    };
                }
            }
            VoyagePhase::HoldingStation { arrived_at_min, .. } => {
                let days_held = (self.processed_minutes - arrived_at_min) / MINUTES_PER_DAY;
                let expected = days_held.saturating_sub(HOLD_STATION_GRACE_DAYS) as u32;
                while self.hold_decay_applied < expected {
                    self.hold_decay_applied += 1;
                    if self.hope > HOPE_FLOOR_STEADY {
                        self.hope -= 1;
                    }
                }
            }
            VoyagePhase::Arrived { .. } => {}
        }
    }

    fn arrive_at(&mut self, waypoint: WaypointId) {
        self.visited.push(waypoint);
        self.rumor_bought_this_visit = false;
        self.hold_decay_applied = 0;
        if waypoint == ROUTE_SINK {
            self.phase = VoyagePhase::Arrived {
                at_min: self.processed_minutes,
            };
        } else {
            self.phase = VoyagePhase::HoldingStation {
                waypoint,
                arrived_at_min: self.processed_minutes,
                scene_state: SceneState::Waiting,
            };
        }
    }

    // ── Player actions ──────────────────────────────────────────────────────

    /// Play the arrival scene at the held waypoint. Returns the scene slot
    /// (spec 4 content) — `None` if there is nothing waiting to play.
    pub fn play_arrival_scene(&mut self) -> Option<&'static SceneRef> {
        if let VoyagePhase::HoldingStation {
            waypoint,
            scene_state: scene_state @ SceneState::Waiting,
            ..
        } = &mut self.phase
        {
            *scene_state = SceneState::Played;
            let waypoint = *waypoint;
            self.provisions = (self.provisions + ARRIVAL_PROVISIONS_GIFT).min(self.provisions_cap);
            Some(&route::waypoint(waypoint).scene)
        } else {
            None
        }
    }

    /// Set the ship's posture. Takes effect from the next unprocessed minute.
    pub fn set_trim(&mut self, trim: Trim) {
        if self.trim != trim {
            self.trim = trim;
            self.mourn_minutes = 0;
        }
    }

    /// A road can always be sailed if it is affordable — or if it is the
    /// cheapest way out (running the hold empty means drifting, not
    /// stranding; with the affordability invariant this is what makes the
    /// crossing unlosable).
    pub fn road_selectable(&self, road: &Road) -> bool {
        self.road_affordable(road)
            || route::cheapest_road_from(road.from).is_some_and(|c| c.id == road.id)
    }

    /// Whole-leg price at the current trim, as the card shows it.
    pub fn road_price(&self, road: &Road) -> u32 {
        (f64::from(road.base_provisions) * self.trim.provisions_mult()).round() as u32
    }

    pub fn road_affordable(&self, road: &Road) -> bool {
        f64::from(self.road_price(road)) <= self.provisions
    }

    /// Commit to a road. Sibling roads gray out permanently; the doors-close
    /// pillar lives here.
    pub fn depart(&mut self, road_id: RoadId) -> Result<(), DepartError> {
        let VoyagePhase::HoldingStation {
            waypoint,
            scene_state,
            ..
        } = self.phase
        else {
            return Err(DepartError::NotAtStation);
        };
        if scene_state == SceneState::Waiting {
            return Err(DepartError::SceneWaiting);
        }
        let road = route::roads_from(waypoint)
            .find(|r| r.id == road_id)
            .ok_or(DepartError::NoSuchRoad)?;
        if !self.road_selectable(road) {
            return Err(DepartError::Locked);
        }
        for sibling in route::roads_from(waypoint) {
            if sibling.id != road_id && !self.untaken.contains(&sibling.id) {
                self.untaken.push(sibling.id);
            }
        }
        self.phase = VoyagePhase::Traveling {
            road: road_id,
            departed_at_min: self.processed_minutes,
            progress_days: 0.0,
        };
        Ok(())
    }

    /// Buy the next rumor a way-station holds: flat price, one per visit.
    pub fn buy_rumor(&mut self) -> Option<RumorId> {
        let VoyagePhase::HoldingStation { waypoint, .. } = self.phase else {
            return None;
        };
        if !route::waypoint(waypoint).has_feature(Feature::WayStation)
            || self.rumor_bought_this_visit
            || self.provisions < RUMOR_PRICE
        {
            return None;
        }
        let next = route::way_station_stock(waypoint)
            .iter()
            .find(|id| !self.knows_rumor(**id))
            .copied()?;
        self.provisions -= RUMOR_PRICE;
        self.rumor_bought_this_visit = true;
        self.rumors.push(LearnedRumor {
            rumor: next,
            learned_at: waypoint,
        });
        Some(next)
    }

    pub fn knows_rumor(&self, id: RumorId) -> bool {
        self.rumors.iter().any(|r| r.rumor == id)
    }

    /// Consume the pending drift-recovery scene flag (UI shows it once).
    pub fn take_pending_recovery_scene(&mut self) -> bool {
        std::mem::take(&mut self.pending_recovery_scene)
    }

    // ── Read model ──────────────────────────────────────────────────────────

    /// The waypoint the ship is at, if holding station.
    pub fn current_waypoint(&self) -> Option<WaypointId> {
        match self.phase {
            VoyagePhase::HoldingStation { waypoint, .. } => Some(waypoint),
            VoyagePhase::Arrived { .. } => Some(ROUTE_SINK),
            _ => None,
        }
    }

    /// The road underway, if traveling or drifting.
    pub fn current_road(&self) -> Option<&'static Road> {
        match self.phase {
            VoyagePhase::Traveling { road, .. } | VoyagePhase::Drifting { road, .. } => {
                Some(route::road(road))
            }
            _ => None,
        }
    }

    /// Days since launch (the seed input for Underway determinism).
    pub fn day_index(&self) -> u64 {
        self.processed_minutes / MINUTES_PER_DAY
    }

    /// Game minutes until arrival at the current trim, if traveling.
    pub fn eta_minutes(&self) -> Option<u64> {
        if let VoyagePhase::Traveling {
            road,
            progress_days,
            ..
        } = self.phase
        {
            let base_days = f64::from(route::road(road).base_days);
            let remaining = (base_days - progress_days).max(0.0) * self.trim.time_mult();
            Some((remaining * MINUTES_PER_DAY as f64).ceil() as u64)
        } else {
            None
        }
    }

    #[allow(dead_code)] // Simulator/tests; the finale (spec 7) reads it in-game.
    pub fn arrived(&self) -> bool {
        matches!(self.phase, VoyagePhase::Arrived { .. })
    }

    /// Provisions as the bar shows them: a small whole number.
    pub fn provisions_display(&self) -> u32 {
        self.provisions.round().max(0.0) as u32
    }
}

/// Hope's name. Never a number on screen without its word.
pub fn hope_label(hope: u8) -> &'static str {
    match hope {
        0 => "ashen",
        1 => "guttering",
        2 => "failing",
        3 => "low",
        4 => "uneasy",
        5 => "steady",
        6 => "warm",
        7 => "bright",
        8 => "high",
        9 => "singing",
        _ => "radiant",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn t0() -> DateTime<Utc> {
        "2026-07-03T12:00:00Z".parse().unwrap()
    }

    fn started() -> VoyageState {
        let mut v = VoyageState::begin("test-char".to_string(), 7, t0());
        v.play_arrival_scene();
        v
    }

    /// Sail the spine to the first junction (W0 -> W1 -> W2, the Shoal
    /// Markets), scenes played, ready to choose. Returns the state and the
    /// wall time it was last ticked at.
    fn at_first_junction() -> (VoyageState, DateTime<Utc>) {
        let mut v = started();
        v.depart(route::roads_from(ROUTE_START).next().unwrap().id)
            .unwrap();
        let mut now = t0() + Duration::days(2);
        v.tick(now);
        v.play_arrival_scene().expect("arrived at W1");
        v.depart(route::roads_from(WaypointId(1)).next().unwrap().id)
            .unwrap();
        now += Duration::days(2);
        v.tick(now);
        v.play_arrival_scene().expect("arrived at W2");
        assert_eq!(v.current_waypoint(), Some(WaypointId(2)));
        (v, now)
    }

    #[test]
    fn begins_holding_station_at_the_last_harbor_with_full_gauges() {
        let v = VoyageState::begin("c".into(), 1, t0());
        assert_eq!(v.current_waypoint(), Some(ROUTE_START));
        assert_eq!(v.provisions, LAUNCH_PROVISIONS);
        assert_eq!(v.hope, LAUNCH_HOPE);
        assert_eq!(hope_label(v.hope), "bright");
        assert_eq!(v.visited, vec![ROUTE_START]);
    }

    #[test]
    fn scene_waiting_blocks_departure() {
        let mut v = VoyageState::begin("c".into(), 1, t0());
        let road = route::roads_from(ROUTE_START).next().unwrap();
        assert_eq!(v.depart(road.id), Err(DepartError::SceneWaiting));
        assert!(v.play_arrival_scene().is_some());
        assert!(v.play_arrival_scene().is_none(), "scene plays once");
        assert_eq!(v.depart(road.id), Ok(()));
    }

    #[test]
    fn a_leg_completes_on_wall_clock_and_arrives_waiting() {
        let mut v = started();
        let road = route::roads_from(ROUTE_START).next().unwrap();
        v.depart(road.id).unwrap();

        // Road 0 is 1.0 base days at Cruise: not there at 23 hours...
        v.tick(t0() + Duration::hours(23));
        assert!(matches!(v.phase, VoyagePhase::Traveling { .. }));
        // ...there within the next two.
        v.tick(t0() + Duration::hours(25));
        assert_eq!(v.current_waypoint(), Some(road.to));
        assert!(matches!(
            v.phase,
            VoyagePhase::HoldingStation {
                scene_state: SceneState::Waiting,
                ..
            }
        ));
        // The leg burned about its base price.
        let burned = LAUNCH_PROVISIONS - v.provisions;
        assert!(
            (burned - f64::from(road.base_provisions)).abs() < 1.0,
            "burned {burned}, expected ~{}",
            road.base_provisions
        );
    }

    #[test]
    fn run_trim_arrives_earlier_and_burns_more() {
        let mut cruise = started();
        let mut run = started();
        run.set_trim(Trim::Run);
        let road = route::roads_from(ROUTE_START).next().unwrap();
        cruise.depart(road.id).unwrap();
        run.depart(road.id).unwrap();

        // At 20 hours: Run (0.8 days = 19.2h) has arrived, Cruise has not.
        let at = t0() + Duration::hours(20);
        cruise.tick(at);
        run.tick(at);
        assert!(matches!(cruise.phase, VoyagePhase::Traveling { .. }));
        assert!(matches!(run.phase, VoyagePhase::HoldingStation { .. }));
        let run_burn = LAUNCH_PROVISIONS - run.provisions;
        assert!(
            (run_burn - f64::from(road.base_provisions) * 1.30).abs() < 1.0,
            "run should burn ~130%, burned {run_burn}"
        );
    }

    #[test]
    fn mourn_raises_hope_one_per_day_at_sea() {
        let mut v = started();
        v.hope = 5;
        v.set_trim(Trim::Mourn);
        // Road 12 is the longest early road; use a long leg instead:
        // sail road 0 (1 day base = 1.4 days at Mourn).
        let road = route::roads_from(ROUTE_START).next().unwrap();
        v.depart(road.id).unwrap();
        v.tick(t0() + Duration::hours(25)); // past one full day, still at sea
        assert!(matches!(v.phase, VoyagePhase::Traveling { .. }));
        assert_eq!(v.hope, 6, "one full Mourn day raises hope once");
    }

    #[test]
    fn empty_hold_drifts_then_recovers_and_resumes() {
        let mut v = started();
        v.provisions = 5.0; // nowhere near road 0's 12
        let road = route::roads_from(ROUTE_START).next().unwrap();
        // Road 0 is the only (hence cheapest) road out: selectable even
        // though unaffordable.
        assert!(!v.road_affordable(road));
        assert!(v.road_selectable(road));
        v.depart(road.id).unwrap();

        // 5/12ths of the day in, the hold runs dry and the ship drifts.
        v.tick(t0() + Duration::hours(12));
        let VoyagePhase::Drifting { progress_days, .. } = v.phase else {
            panic!("expected drift, got {:?}", v.phase);
        };
        assert!(progress_days > 0.3 && progress_days < 0.5);
        assert_eq!(v.provisions, 0.0);

        // 36 hours later: recovered, resumed at the same progress, +25.
        v.tick(t0() + Duration::hours(12 + 36 + 1));
        assert!(matches!(v.phase, VoyagePhase::Traveling { .. }));
        assert!(v.provisions > 20.0 && v.provisions <= 25.0);
        assert!(v.take_pending_recovery_scene());
        assert!(!v.take_pending_recovery_scene(), "shown once");

        // And the leg still finishes.
        v.tick(t0() + Duration::days(4));
        assert_eq!(v.current_waypoint(), Some(road.to));
    }

    #[test]
    fn holding_past_grace_frays_hope_to_steady_and_no_further() {
        let mut v = started();
        v.hope = 8;
        // Hold at the start (scene already played) for 10 days.
        v.tick(t0() + Duration::days(10));
        assert_eq!(v.hope, HOPE_FLOOR_STEADY, "8 -> floor after 3-day grace");
        v.tick(t0() + Duration::days(30));
        assert_eq!(v.hope, HOPE_FLOOR_STEADY, "never below steady from holding");
    }

    #[test]
    fn committing_grays_siblings_permanently() {
        let (mut v, _) = at_first_junction();

        let roads: Vec<_> = route::roads_from(WaypointId(2)).collect();
        assert_eq!(roads.len(), 2, "W2 is a junction");
        v.depart(roads[0].id).unwrap();
        assert!(v.untaken.contains(&roads[1].id));
        assert!(!v.untaken.contains(&roads[0].id));
    }

    #[test]
    fn locked_roads_refuse_departure() {
        let (mut v, _) = at_first_junction();

        // At W2 the cheapest road is R2 (14); make R3 (16) unaffordable.
        v.provisions = 15.0;
        let roads: Vec<_> = route::roads_from(WaypointId(2)).collect();
        let cheapest = route::cheapest_road_from(WaypointId(2)).unwrap();
        let pricier = roads.iter().find(|r| r.id != cheapest.id).unwrap();
        assert_eq!(v.depart(pricier.id), Err(DepartError::Locked));
        assert_eq!(v.depart(cheapest.id), Ok(()));
    }

    #[test]
    fn way_station_sells_one_rumor_per_visit() {
        let (mut v, _) = at_first_junction();

        let before = v.provisions;
        let bought = v.buy_rumor().expect("station has stock");
        assert!(v.knows_rumor(bought));
        assert_eq!(v.provisions, before - RUMOR_PRICE);
        assert!(v.buy_rumor().is_none(), "one per visit");
    }

    #[test]
    fn no_rumor_purchases_off_station() {
        let mut v = started(); // W0 is a Harbor, not a WayStation
        assert!(v.buy_rumor().is_none());
    }

    #[test]
    fn tick_is_chunking_invariant() {
        // The offline-equivalence property at unit scale: many small ticks
        // land in exactly the same state as one big tick.
        let build = || {
            let mut v = started();
            v.set_trim(Trim::Quiet);
            v.depart(route::roads_from(ROUTE_START).next().unwrap().id)
                .unwrap();
            v
        };
        let mut live = build();
        let mut offline = build();

        for hour in 1..=72 {
            live.tick(t0() + Duration::hours(hour));
        }
        offline.tick(t0() + Duration::hours(72));

        assert_eq!(live.phase, offline.phase);
        assert_eq!(live.provisions.to_bits(), offline.provisions.to_bits());
        assert_eq!(live.hope, offline.hope);
        assert_eq!(live.processed_minutes, offline.processed_minutes);
    }

    #[test]
    fn hope_labels_cover_the_scale() {
        for h in 0..=HOPE_MAX {
            assert!(!hope_label(h).is_empty());
        }
        assert_eq!(hope_label(HOPE_FLOOR_STEADY), "steady");
    }
}
