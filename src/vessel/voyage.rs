//! The Voyage — Act 2's state machine (sub-project 2).
//!
//! One ship moves through the route graph on wall-clock time. Transitions are
//! computed lazily on tick/load (the Loom's timer pattern — no background
//! scheduling), stepped in whole *game minutes* so that processing N days in
//! one call or in many produces bitwise-identical state (the offline
//! equivalence property).
//!
//! See `docs/superpowers/specs/2026-07-03-vessel-route-waypoints-design.md`.

use super::refits::{RefitId, REFIT_PAIRS};
use super::route::{
    self, Chapter, Feature, Road, RoadId, RumorId, WaypointId, ROUTE_SINK, ROUTE_START,
};
use super::scenes::{self, ColorKey};
use super::souls::{
    self, helm_time_mult, tender_provisions_mult, wind_time_mult, ArcTrigger, SoulId, Station,
    ARC_BEAT_REST_DAYS, BERTHS, FAREWELL_HOPE_COST, LOSS_HOPE_COST,
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
        /// The road that brought us (colors the arrival scene and runs its
        /// threat ledger). Absent in pre-spec-4 saves.
        #[serde(default)]
        arrived_by: Option<RoadId>,
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

/// Where a met soul stands with the Vessel. Unmet souls are simply absent
/// from the roster vec. Every non-`Aboard` state is permanent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoulStatus {
    Aboard,
    /// The ask was refused. The door closed.
    Declined,
    /// Stepped ashore in a farewell (to free a berth). Remembered, not lost.
    Ashore,
    /// Lost — authored scenes only (spec 4). Carved into the hull.
    Lost,
}

/// One met soul's state in the crossing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoulState {
    pub soul: SoulId,
    pub status: SoulStatus,
    /// Standing post. A stationed soul is not resting.
    pub station: Option<Station>,
    /// Beats fired so far (arc has 3 beats + a resolution = 4).
    pub arc_beat: u8,
    /// Rest minutes accumulated toward the next *ready* beat.
    pub rest_minutes: u64,
}

/// An arc beat fired (possibly offline) — queued for the UI to show as a
/// log moment. Serialized so beats that land while away greet the return.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoulEvent {
    pub soul: SoulId,
    /// Index into the soul's arc (0..=3).
    pub beat: u8,
}

/// A resolved scene, ready to read: title, paragraphs (beats + matching
/// color lines + ledger lines), and the payout in small print.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenePlayback {
    pub title: String,
    pub paragraphs: Vec<String>,
    pub payout_note: String,
}

fn default_roster() -> Vec<SoulState> {
    souls::launch_souls()
        .map(|def| SoulState {
            soul: def.id,
            status: SoulStatus::Aboard,
            station: None,
            arc_beat: 0,
            rest_minutes: 0,
        })
        .collect()
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
    /// Refits chosen (permanent A/B doors at shipyards).
    #[serde(default)]
    pub refits: Vec<RefitId>,
    /// How many refit doors have been offered (the sequence cursor).
    #[serde(default)]
    pub refit_doors_seen: u8,
    /// A yard's offer waiting to be answered; blocks departure like an ask.
    #[serde(default)]
    pub pending_refit: Option<u8>,
    /// Mementos. No mechanics; the manifest remembers (spec 7).
    #[serde(default)]
    pub keepsakes: Vec<String>,
    /// The crossing's story so far: one title per scene and beat.
    #[serde(default)]
    pub log: Vec<String>,
    /// The leg that is underway (or just ended) included a drift.
    #[serde(default)]
    pub drifted_this_leg: bool,
    /// The finale playback has been surfaced once (spec 7 owns the rest).
    #[serde(default)]
    pub finale_shown: bool,
    /// Every soul met so far (unmet = absent). Older saves default to the
    /// launch trio.
    #[serde(default = "default_roster")]
    pub souls: Vec<SoulState>,
    /// A boarding ask waiting for an answer. Blocks departure — the ask is
    /// a door, and doors are answered, never slipped past.
    #[serde(default)]
    pub pending_ask: Option<SoulId>,
    /// Hope hit ashen: legs crawl and arcs pause until a rest stop.
    #[serde(default)]
    pub long_silence: bool,
    /// Arc beats fired since last read (possibly offline) — the return
    /// view's log moments.
    #[serde(default)]
    pub soul_events: Vec<SoulEvent>,
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
    /// A soul is asking to board; answer before the ship leaves.
    AskPending,
    /// The yard's refit offer waits; answer before the ship leaves.
    RefitPending,
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
                arrived_by: None,
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
            souls: default_roster(),
            pending_ask: None,
            long_silence: false,
            soul_events: Vec::new(),
            refits: Vec::new(),
            refit_doors_seen: 0,
            pending_refit: None,
            keepsakes: Vec::new(),
            log: Vec::new(),
            drifted_this_leg: false,
            finale_shown: false,
        }
    }

    pub fn has_refit(&self, refit: RefitId) -> bool {
        self.refits.contains(&refit)
    }

    // ── Multiplier composition ──────────────────────────────────────────────
    // Fixed order, documented once: time = base × trim × wind × helm;
    // provisions = base × trim × tender. The UI only ever shows the composed
    // integers these produce.

    /// Leg-time multiplier at a hypothetical trim (the trim panel previews
    /// all four). Wind and helm ride along.
    pub fn time_mult_with(&self, trim: Trim) -> f64 {
        let helm = self.station_soul(Station::Helm);
        let storm_sail = if self.has_refit(RefitId::StormSail) {
            0.90
        } else {
            1.00
        };
        trim.time_mult()
            * wind_time_mult(self.hope, self.long_silence)
            * helm_time_mult(
                helm.is_some(),
                helm.is_some_and(|id| souls::soul(id).affinity == Some(Station::Helm)),
            )
            * storm_sail
    }

    /// Leg-provisions multiplier at a hypothetical trim.
    pub fn provisions_mult_with(&self, trim: Trim) -> f64 {
        let tender = self.station_soul(Station::Tender);
        // Mourning Colors deepens Mourn's thrift (0.80 instead of 0.90).
        let trim_mult = if trim == Trim::Mourn && self.has_refit(RefitId::MourningColors) {
            0.80
        } else {
            trim.provisions_mult()
        };
        trim_mult
            * tender_provisions_mult(
                tender.is_some(),
                tender.is_some_and(|id| souls::soul(id).affinity == Some(Station::Tender)),
            )
    }

    pub fn time_mult(&self) -> f64 {
        self.time_mult_with(self.trim)
    }

    pub fn provisions_mult(&self) -> f64 {
        self.provisions_mult_with(self.trim)
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
        self.step_arcs_minute();
        match self.phase {
            VoyagePhase::Traveling {
                road: road_id,
                departed_at_min,
                progress_days,
            } => {
                let road = route::road(road_id);
                let dp = (1.0 / MINUTES_PER_DAY as f64) / self.time_mult();
                let burn = dp
                    * (f64::from(road.base_provisions) / f64::from(road.base_days))
                    * self.provisions_mult();

                if self.provisions < burn {
                    // The hold runs dry mid-road: drift where we stand.
                    self.provisions = 0.0;
                    self.drifted_this_leg = true;
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
                    self.arrive_at(road.to, Some(road_id));
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
                    self.provisions = if self.has_refit(RefitId::DeepLarder) {
                        40.0
                    } else {
                        f64::from(DRIFT_RECOVERY_PROVISIONS)
                    };
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

    fn arrive_at(&mut self, waypoint: WaypointId, arrived_by: Option<RoadId>) {
        self.visited.push(waypoint);
        self.rumor_bought_this_visit = false;
        self.hold_decay_applied = 0;

        // A hearth breaks the Long Silence: the fire is relit, hope
        // returns to "low", and the ship can breathe again.
        if self.long_silence && route::waypoint(waypoint).has_feature(Feature::RestStop) {
            self.long_silence = false;
            self.hope = self.hope.max(3);
        }

        // A soul may be waiting here. The ask blocks departure until
        // answered — doors are answered, never slipped past.
        if let Some(def) = souls::recruit_at(waypoint) {
            if !self.met(def.id) {
                self.pending_ask = Some(def.id);
            }
        }

        // The first three distinct shipyards each open one refit door.
        if route::waypoint(waypoint).has_feature(Feature::Shipyard)
            && (self.refit_doors_seen as usize) < REFIT_PAIRS.len()
            && self.pending_refit.is_none()
        {
            self.pending_refit = Some(self.refit_doors_seen);
        }

        // Entering Chapter IV: unresolved arcs skip straight to their
        // resolution beat (skipped beats pay nothing) — nobody's story is
        // left dangling at the finale.
        if route::waypoint(waypoint).chapter == Chapter::RootsOfLight {
            for i in 0..self.souls.len() {
                if self.souls[i].status == SoulStatus::Aboard && self.souls[i].arc_beat < 3 {
                    self.souls[i].arc_beat = 3;
                    self.souls[i].rest_minutes = 0;
                }
            }
        }

        if waypoint == ROUTE_SINK {
            // The crossing ends with every aboard story resolved: any
            // resolution still unfired fires now, rest debt forgiven.
            for i in 0..self.souls.len() {
                if self.souls[i].status == SoulStatus::Aboard && self.souls[i].arc_beat < 4 {
                    self.fire_beat(i);
                }
            }
            self.phase = VoyagePhase::Arrived {
                at_min: self.processed_minutes,
            };
        } else {
            self.phase = VoyagePhase::HoldingStation {
                waypoint,
                arrived_at_min: self.processed_minutes,
                scene_state: SceneState::Waiting,
                arrived_by,
            };
        }
    }

    // ── Arcs ────────────────────────────────────────────────────────────────

    /// One minute of arc time: souls aboard and off-station rest; a soul
    /// whose next beat is ready converts rest into story. The Long Silence
    /// pauses all of it.
    fn step_arcs_minute(&mut self) {
        if self.long_silence {
            return;
        }
        for i in 0..self.souls.len() {
            let s = self.souls[i];
            if s.status != SoulStatus::Aboard || s.station.is_some() {
                continue;
            }
            let def = souls::soul(s.soul);
            let Some(beat) = def.arc.get(s.arc_beat as usize) else {
                continue; // arc complete
            };
            if !self.trigger_met(beat.trigger) {
                continue;
            }
            self.souls[i].rest_minutes += 1;
            if self.souls[i].rest_minutes >= ARC_BEAT_REST_DAYS * MINUTES_PER_DAY {
                self.fire_beat(i);
            }
        }
    }

    fn trigger_met(&self, trigger: ArcTrigger) -> bool {
        match trigger {
            ArcTrigger::Aboard => true,
            ArcTrigger::ReachChapter(chapter) => self
                .visited
                .last()
                .is_some_and(|w| route::waypoint(*w).chapter >= chapter),
            ArcTrigger::VisitFeature(feature) => self
                .visited
                .iter()
                .any(|w| route::waypoint(*w).has_feature(feature)),
            ArcTrigger::VisitWaypoint(waypoint) => self.visited.contains(&waypoint),
        }
    }

    /// Fire the roster[i] soul's next beat: payout lands, the moment is
    /// queued for the log, the rest counter resets.
    fn fire_beat(&mut self, i: usize) {
        let s = self.souls[i];
        let def = souls::soul(s.soul);
        let Some(beat) = def.arc.get(s.arc_beat as usize) else {
            return;
        };
        self.hope = (self.hope + beat.payout.hope).min(HOPE_MAX);
        if let Some(rumor) = beat.payout.rumor {
            if !self.knows_rumor(rumor) {
                let learned_at = self.visited.last().copied().unwrap_or(ROUTE_START);
                self.rumors.push(LearnedRumor { rumor, learned_at });
            }
        }
        self.soul_events.push(SoulEvent {
            soul: s.soul,
            beat: s.arc_beat,
        });
        self.souls[i].arc_beat += 1;
        self.souls[i].rest_minutes = 0;
    }

    // ── Player actions ──────────────────────────────────────────────────────

    /// Play the arrival scene at the held waypoint: resolve its beats and
    /// color lines, run the road's threat ledger, apply the payout — all
    /// exactly once. Returns the playback for the UI, `None` if nothing
    /// waits to play.
    pub fn play_arrival_scene(&mut self) -> Option<ScenePlayback> {
        let VoyagePhase::HoldingStation {
            waypoint,
            scene_state: scene_state @ SceneState::Waiting,
            arrived_by,
            ..
        } = &mut self.phase
        else {
            return None;
        };
        *scene_state = SceneState::Played;
        let waypoint = *waypoint;
        let arrived_by = *arrived_by;

        let def = scenes::scene_def(waypoint);
        let mut paragraphs: Vec<String> = def.beats.iter().map(|b| b.to_string()).collect();
        for (key, line) in def.colors {
            if self.color_matches(*key, arrived_by) {
                paragraphs.push(line.to_string());
            }
        }

        // The road's threat, if it carried one, speaks now — a ledger of
        // prior choices, never a roll.
        let mut payout_notes: Vec<String> = Vec::new();
        if let Some(road_id) = arrived_by {
            if route::road(road_id).threat.is_some() {
                let (lines, notes) = self.threat_ledger(road_id);
                paragraphs.extend(lines);
                payout_notes.extend(notes);
            }
        }

        // The authored payout, applied once.
        let payout = def.payout;
        if payout.provisions > 0 {
            self.provisions =
                (self.provisions + f64::from(payout.provisions)).min(self.provisions_cap);
            payout_notes.push(format!("the hold gains {}", payout.provisions));
        }
        match payout.hope.cmp(&0) {
            std::cmp::Ordering::Greater => {
                self.hope = (self.hope + payout.hope as u8).min(HOPE_MAX);
                payout_notes.push("hope rises".to_string());
            }
            std::cmp::Ordering::Less => {
                self.lower_hope(payout.hope.unsigned_abs());
                payout_notes.push("hope dims".to_string());
            }
            std::cmp::Ordering::Equal => {}
        }
        if let Some(rumor) = payout.rumor {
            if !self.knows_rumor(rumor) {
                self.rumors.push(LearnedRumor {
                    rumor,
                    learned_at: waypoint,
                });
                payout_notes.push("a rumor learned".to_string());
            }
        }
        if let Some(keepsake) = payout.keepsake {
            self.keepsakes.push(keepsake.to_string());
            payout_notes.push(format!("kept: {keepsake}"));
        }

        let title = route::waypoint(waypoint).name.to_string();
        self.log.push(title.clone());
        self.drifted_this_leg = false;

        Some(ScenePlayback {
            title,
            paragraphs,
            payout_note: payout_notes.join(" \u{00b7} "),
        })
    }

    fn color_matches(&self, key: ColorKey, arrived_by: Option<RoadId>) -> bool {
        match key {
            ColorKey::SoulAboard(id) => self
                .soul_state(id)
                .is_some_and(|s| s.status == SoulStatus::Aboard),
            ColorKey::ArrivedBy(road) => arrived_by == Some(road),
            ColorKey::TrimIs(trim) => self.trim == trim,
            ColorKey::KnowsRumor(rumor) => self.knows_rumor(rumor),
            ColorKey::HopeAtLeast(n) => self.hope >= n,
            ColorKey::HopeBelow(n) => self.hope < n,
            ColorKey::Drifted => self.drifted_this_leg,
        }
    }

    /// The threat ledgers: outcome rows checked in order, every row a
    /// consequence of choices the junction card priced. Returns the scene
    /// lines and the payout notes.
    fn threat_ledger(&mut self, road_id: RoadId) -> (Vec<String>, Vec<String>) {
        let quiet_keel = self.has_refit(RefitId::QuietKeel);
        match road_id.0 {
            // The Ossuary Warden, over the reef.
            9 => {
                let sefa_aboard = self
                    .soul_state(SoulId(4))
                    .is_some_and(|s| s.status == SoulStatus::Aboard);
                if sefa_aboard {
                    self.keepsakes
                        .push("the Warden's token, white and warm".to_string());
                    (
                        vec!["The Warden rises to count you — and Sefa sings the office \
                             for the unburied. The white water goes still, and \
                             something like a tithe-mark is pressed into the rail."
                            .to_string()],
                        vec!["kept: the Warden's token".to_string()],
                    )
                } else if quiet_keel || self.trim == Trim::Quiet || self.trim == Trim::Mourn {
                    self.provisions = (self.provisions - 15.0).max(0.0);
                    (
                        vec!["The Warden takes its toll from the hold, slowly, while \
                             the crew stands silent and lets it. Slow and respectful \
                             buys passage. It does not buy exemption."
                            .to_string()],
                        vec!["the Warden takes 15".to_string()],
                    )
                } else {
                    self.provisions = (self.provisions - 15.0).max(0.0);
                    self.lower_hope(2);
                    (
                        vec!["You hurry the reef, and the Warden hurries with you. It \
                             takes its toll from the hold and something less \
                             replaceable from the crew's sleep."
                            .to_string()],
                        vec!["the Warden takes 15 \u{00b7} hope dims".to_string()],
                    )
                }
            }
            // The Silence itself, on the silent road.
            29 => {
                let anchored = self.aboard().any(|s| s.station.is_none());
                if anchored || quiet_keel {
                    (
                        vec!["The Silence leans on the ship for three days. The souls \
                             at rest hold the crew's voice for them — a hand on a \
                             shoulder, a note passed, a meal made loudly. It passes."
                            .to_string()],
                        vec![],
                    )
                } else {
                    self.lower_hope(2);
                    (
                        vec!["Every soul stood a post and nobody held the middle of \
                             the ship. The Silence sat there instead, and the leg's \
                             log is three blank pages."
                            .to_string()],
                        vec!["hope dims".to_string()],
                    )
                }
            }
            // The Thorns — the game's only loss.
            42 => {
                let cormac_at_helm = self
                    .soul_state(SoulId(6))
                    .is_some_and(|s| s.status == SoulStatus::Aboard)
                    && self.station_soul(Station::Helm) == Some(SoulId(6));
                if cormac_at_helm {
                    self.keepsakes
                        .push("a thorn spar, cut clean at the tip".to_string());
                    (
                        vec!["Cormac takes the run in one line, reading the thorns \
                             like a sentence he wrote. The hull never touches. At \
                             the far end he hands back the wheel and says, 'Now it's \
                             a road.'"
                            .to_string()],
                        vec!["kept: a thorn spar".to_string()],
                    )
                } else if quiet_keel {
                    self.provisions = (self.provisions - 10.0).max(0.0);
                    (
                        vec!["The thorns close on the hull and the quiet keel holds — \
                             barely, loudly, expensively. The crew spends the last \
                             mile listening to the wood argue and win."
                            .to_string()],
                        vec!["the thorns take 10".to_string()],
                    )
                } else {
                    // The exposure is the post: helm first, then tender,
                    // then watch. Resting souls are below, and safe.
                    let exposed = [Station::Helm, Station::Tender, Station::Watch]
                        .into_iter()
                        .find_map(|post| self.station_soul(post));
                    if let Some(lost) = exposed {
                        let name = souls::soul(lost).name;
                        self.mark_lost(lost);
                        (
                            vec![format!(
                                "The thorns take the ship the way weather takes \
                                     a coastline. When the run opens out and the \
                                     count is called, {name} does not answer it. The \
                                     post stands empty. The hull carries a new name."
                            )],
                            vec![format!("{name} is lost \u{00b7} hope falls")],
                        )
                    } else {
                        self.provisions = (self.provisions - 20.0).max(0.0);
                        self.lower_hope(2);
                        (
                            vec!["With every soul below, the ship takes the thorns on \
                                 her own skin. She holds. It costs the hold and the \
                                 crew's certainty, and the scars stay."
                                .to_string()],
                            vec!["the thorns take 20 \u{00b7} hope dims".to_string()],
                        )
                    }
                }
            }
            _ => (Vec::new(), Vec::new()),
        }
    }

    /// The refit door being offered, if any.
    pub fn pending_refit_pair(&self) -> Option<crate::vessel::refits::RefitPair> {
        self.pending_refit
            .map(|i| REFIT_PAIRS[(i as usize).min(REFIT_PAIRS.len() - 1)])
    }

    /// Answer the yard: take A or B. The other closes forever.
    pub fn choose_refit(&mut self, pick_a: bool) -> Option<RefitId> {
        let pair = self.pending_refit_pair()?;
        let chosen = if pick_a { pair.a } else { pair.b };
        self.refits.push(chosen);
        if chosen == RefitId::LongHold {
            self.provisions_cap = LONG_HOLD_PROVISIONS_CAP;
        }
        self.refit_doors_seen += 1;
        self.pending_refit = None;
        self.log.push(format!("Refit: {}", chosen.display_name()));
        Some(chosen)
    }

    /// The finale scene at the Tree, surfaced once (spec 7 owns the rest).
    pub fn take_finale_playback(&mut self) -> Option<ScenePlayback> {
        if !self.arrived() || self.finale_shown {
            return None;
        }
        self.finale_shown = true;
        let def = scenes::scene_def(ROUTE_SINK);
        self.hope = (self.hope + def.payout.hope.max(0) as u8).min(HOPE_MAX);
        let title = route::waypoint(ROUTE_SINK).name.to_string();
        self.log.push(title.clone());
        Some(ScenePlayback {
            title,
            paragraphs: def.beats.iter().map(|b| b.to_string()).collect(),
            payout_note: "the crossing is over".to_string(),
        })
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

    /// Whole-leg price as the card shows it: trim and tender composed.
    pub fn road_price(&self, road: &Road) -> u32 {
        (f64::from(road.base_provisions) * self.provisions_mult()).round() as u32
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
        if self.pending_ask.is_some() {
            return Err(DepartError::AskPending);
        }
        if self.pending_refit.is_some() {
            return Err(DepartError::RefitPending);
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

    // ── The roster ──────────────────────────────────────────────────────────

    pub fn met(&self, id: SoulId) -> bool {
        self.souls.iter().any(|s| s.soul == id)
    }

    pub fn soul_state(&self, id: SoulId) -> Option<&SoulState> {
        self.souls.iter().find(|s| s.soul == id)
    }

    pub fn aboard(&self) -> impl Iterator<Item = &SoulState> {
        self.souls.iter().filter(|s| s.status == SoulStatus::Aboard)
    }

    pub fn aboard_count(&self) -> usize {
        self.aboard().count()
    }

    /// Who holds a post, if anyone.
    pub fn station_soul(&self, station: Station) -> Option<SoulId> {
        self.aboard()
            .find(|s| s.station == Some(station))
            .map(|s| s.soul)
    }

    /// Assign a soul to a post (bumping whoever held it) or relieve them
    /// (`None`) so their arc can move again.
    pub fn set_station(&mut self, id: SoulId, station: Option<Station>) -> bool {
        if !self
            .soul_state(id)
            .is_some_and(|s| s.status == SoulStatus::Aboard)
        {
            return false;
        }
        if let Some(post) = station {
            for s in &mut self.souls {
                if s.station == Some(post) {
                    s.station = None;
                }
            }
        }
        for s in &mut self.souls {
            if s.soul == id {
                s.station = station;
            }
        }
        true
    }

    /// Say yes to the pending ask. Fails (returning `false`) when the
    /// berths are full — free one with [`Self::farewell`] first, or decline.
    pub fn accept_ask(&mut self) -> bool {
        let Some(id) = self.pending_ask else {
            return false;
        };
        if self.aboard_count() >= BERTHS {
            return false;
        }
        self.pending_ask = None;
        self.souls.push(SoulState {
            soul: id,
            status: SoulStatus::Aboard,
            station: None,
            arc_beat: 0,
            rest_minutes: 0,
        });
        true
    }

    /// Say no. Permanent: the door closes, the name stays on the chart.
    pub fn decline_ask(&mut self) -> Option<SoulId> {
        let id = self.pending_ask.take()?;
        self.souls.push(SoulState {
            soul: id,
            status: SoulStatus::Declined,
            station: None,
            arc_beat: 0,
            rest_minutes: 0,
        });
        Some(id)
    }

    /// A soul steps ashore to free a berth. Remembered in the manifest,
    /// never carved into the hull. Costs a little hope.
    pub fn farewell(&mut self, id: SoulId) -> bool {
        let Some(s) = self
            .souls
            .iter_mut()
            .find(|s| s.soul == id && s.status == SoulStatus::Aboard)
        else {
            return false;
        };
        s.status = SoulStatus::Ashore;
        s.station = None;
        self.lower_hope(FAREWELL_HOPE_COST);
        true
    }

    /// A soul is lost. Reachable ONLY from authored scenes (spec 4) — no
    /// tick-driven path calls this, which is the covenant in one sentence.
    /// The name is carved into the hull for the rest of the game.
    #[allow(dead_code)] // Spec 4's loss API; covenant-tested today.
    pub fn mark_lost(&mut self, id: SoulId) -> bool {
        let Some(s) = self
            .souls
            .iter_mut()
            .find(|s| s.soul == id && s.status == SoulStatus::Aboard)
        else {
            return false;
        };
        s.status = SoulStatus::Lost;
        s.station = None;
        self.lower_hope(LOSS_HOPE_COST);
        true
    }

    /// Names carved into the hull, in the order they were lost.
    pub fn carved_names(&self) -> Vec<&'static str> {
        self.souls
            .iter()
            .filter(|s| s.status == SoulStatus::Lost)
            .map(|s| souls::soul(s.soul).name)
            .collect()
    }

    /// Drain arc moments queued for the log (possibly fired offline).
    pub fn take_soul_events(&mut self) -> Vec<SoulEvent> {
        std::mem::take(&mut self.soul_events)
    }

    /// Lower hope, entering the Long Silence at ashen.
    fn lower_hope(&mut self, by: u8) {
        self.hope = self.hope.saturating_sub(by);
        if self.hope == 0 {
            self.long_silence = true;
        }
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
            let remaining = (base_days - progress_days).max(0.0) * self.time_mult();
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
        // W1 is Maren's recruit site; the ask blocks departure until
        // answered. Take her aboard.
        assert!(v.accept_ask(), "Maren's ask should be pending at W1");
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
