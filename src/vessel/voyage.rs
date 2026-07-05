//! The Voyage — Act 2's state machine (sub-project 2).
//!
//! One ship moves through the route graph on wall-clock time. Transitions are
//! computed lazily on tick/load (the Loom's timer pattern — no background
//! scheduling), stepped in whole *game minutes* so that processing N days in
//! one call or in many produces bitwise-identical state (the offline
//! equivalence property).
//!
//! See `openspec/specs/vessel-act2/spec.md`.

use super::letters::{self, LAST_LETTER_EVENT, LETTER_PARCEL_PROVISIONS, MAIL_FAILS_EVENT};
use super::nights::{self, NightKind, NightOutcome};
use super::pilgrims::{self, PilgrimShip};
use super::refits::{RefitId, REFIT_PAIRS};
use super::route::{
    self, Chapter, Feature, Road, RoadId, RumorId, WaypointId, ROUTE_SINK, ROUTE_START,
};
use super::scenes::{self, ColorKey};
use super::souls::{
    self, helm_time_mult, tender_provisions_mult, ArcTrigger, SoulId, Station, ARC_BEAT_REST_DAYS,
    CREW,
};
use super::weather::{self, WeatherKind, WeatherObj};
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
/// Buying a rumor at a way-station (one per visit).
pub const RUMOR_PRICE: f64 = 6.0;
/// How long the ship waters and takes on mail at a port with no decision
/// to make (one road out, nobody asking to board, no refit door) before
/// she sails herself onward. Decisions — junctions, asks, refit doors,
/// the pier itself — always wait for the ferryman; plain ports do not,
/// or a compressed era would spend itself waiting instead of sailing.
pub const PORT_CALL_GAME_MINUTES: u64 = 6 * 60;

/// Scars on the hull: cap, and what each one eats.
pub const HULL_WEAR_MAX: u8 = 6;
pub const WEAR_BURN_PER_SCAR: f64 = 0.05;

/// The most a wormhole jump committed at 0% Riftglass charge can dock from
/// the next crossing's starting provisions (scaled by `1.0 - charge`) — see
/// `openspec/changes/act2-dock-wormhole-crossing/design.md` Decision 4. A
/// starting value pending `voyage_simulator` validation, not yet final.
pub const MAX_PARTIAL_CHARGE_PROVISIONS_DEFICIT: f64 = 40.0;
/// The most hull wear a 0%-charge jump can pre-apply to the next crossing
/// (scaled by `1.0 - charge`), out of the `HULL_WEAR_MAX` scale.
pub const MAX_PARTIAL_CHARGE_HULL_WEAR: u8 = 3;
/// Extra daily provisions each passenger aboard a ferry run eats (spec 9).
/// Tiny, because a long era's expeditions run into the thousands of souls —
/// a steeper draw would leave a full ship unable to make the crossing at
/// all. Even a 7,000-soul expedition only lifts the daily burn by ~35%.
pub const PROVISIONS_PER_PASSENGER: f64 = 0.00005;

/// How fast time at sea passes relative to the real world — a near-1:1 clock
/// (a day at sea passes in a little under a real day). The pacing is *earned*,
/// not compressed: the maiden voyage sails its ~37 days in about a real month
/// (the slowest crossing of the whole era), and the ramp that follows is the
/// Drive shortening each crossing — not the clock speeding up. Every crossing
/// runs on this one uniform scale; only the earned Drive level changes.
pub const GAME_MINUTES_PER_REAL_MINUTE: f64 = 2.64;

/// Wall-clock multiplier: `QUEST_VOYAGE_TIME_SCALE` (dev/test override; at
/// 1440x a voyage day passes in one real minute — used by drive-game), or
/// the production [`GAME_MINUTES_PER_REAL_MINUTE`]. Read once.
pub fn time_scale() -> f64 {
    static SCALE: std::sync::OnceLock<f64> = std::sync::OnceLock::new();
    *SCALE.get_or_init(|| {
        std::env::var("QUEST_VOYAGE_TIME_SCALE")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .filter(|s| s.is_finite() && *s > 0.0)
            .unwrap_or(GAME_MINUTES_PER_REAL_MINUTE)
    })
}

/// The real (wall-clock) duration in which `game_minutes` of the crossing
/// pass, under the active [`time_scale`]. Fixtures and tests use this to
/// place a voyage "N game-days in" regardless of the scale.
pub fn real_duration_for_game_minutes(game_minutes: i64) -> chrono::Duration {
    let real_secs = (game_minutes as f64) * 60.0 / time_scale();
    chrono::Duration::milliseconds((real_secs * 1000.0).round() as i64)
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
            // Restful is the thriftiest hold there is — its identity now that
            // it no longer raises hope (the hope system is retired).
            Trim::Mourn => 0.80,
        }
    }

    pub fn display_name(&self) -> &'static str {
        // The player-facing Pace ladder (spec 10), in Oregon Trail's
        // register — named for the toll, not the speed. The internal
        // variant names stay (Run/Cruise/Quiet/Mourn) as engine terms.
        match self {
            Trim::Run => "Grueling",
            Trim::Cruise => "Steady",
            Trim::Quiet => "Easy",
            Trim::Mourn => "Restful",
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
    /// Stepped ashore in a farewell (to free a seat). Remembered, not lost.
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
    /// Where a declined or farewelled soul left the story (the manifest
    /// remembers the place). `None` for souls aboard, lost, or from saves
    /// older than spec 7.
    #[serde(default)]
    pub left_at: Option<WaypointId>,
    /// The crossing's price on this soul (spec 8): 0 sound, 1 strained
    /// (affinity stops counting, rest at half pace), 2 worn (cannot hold
    /// a post). Healed one level per RestStop arrival, off-post only.
    #[serde(default)]
    pub strain: u8,
    /// Nights stood in a row; the third strains. Any night off resets it.
    #[serde(default)]
    pub consecutive_watches: u8,
}

/// An arc beat fired (possibly offline) — queued for the UI to show as a
/// log moment. Serialized so beats that land while away greet the return.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoulEvent {
    pub soul: SoulId,
    /// Index into the soul's arc (0..=3).
    pub beat: u8,
}

/// One line of the ship's log, stamped with the voyage day it was written.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogEntry {
    pub day: u64,
    pub text: String,
}

/// Why a soul strained (spec 8). Every entry names its cause — a price
/// is only fair if the buyer can read the receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrainCause {
    ThirdWatch,
    SquallAtRun,
    SilenceHelm,
}

impl StrainCause {
    pub fn text(&self) -> &'static str {
        match self {
            StrainCause::ThirdWatch => "three nights on watch, back to back",
            StrainCause::SquallAtRun => "a squall crossed while driven hard, on deck the whole way",
            StrainCause::SilenceHelm => "the helm held through the silence, alone with it",
        }
    }
}

/// Why the hull took a scar (spec 8).
// Wear comes only from choices — Run through squalls, hungry roads —
// never from drifting. Drift is the covenant's gentle failure; letting it
// scar would compound the very state it rescues (probe-verified
// death-spiral, 2026-07-03).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WearCause {
    RunHard,
    SquallAtRun,
    ThreatRoad,
}

impl WearCause {
    pub fn text(&self) -> &'static str {
        match self {
            WearCause::RunHard => "a whole leg driven at the Grueling pace, start to finish",
            WearCause::SquallAtRun => "a squall taken at the Grueling pace",
            WearCause::ThreatRoad => "the road took its price from her skin",
        }
    }
}

/// A price paid on the crossing (spec 8), queued for the UI like arc
/// beats: acquired at deterministic sim-time events, surfaced at the next
/// check-in with the cause named. Serialized so costs taken offline greet
/// the return.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PassageEvent {
    /// A soul strained (level 1) or worn (level 2), and why.
    Strained {
        soul: SoulId,
        level: u8,
        cause: StrainCause,
    },
    /// A soul mended one level at a rest stop.
    HealedAtRest { soul: SoulId },
    /// The hull took a scar (its new count), and why.
    Scarred { wear: u8, cause: WearCause },
}

/// A resolved scene, ready to read: title, paragraphs (beats + matching
/// color lines + ledger lines), and the payout in small print. Serialized
/// when a port was auto-sailed while the ferryman was away — the reading
/// waits; the sailing does not.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
            left_at: None,
            strain: 0,
            consecutive_watches: 0,
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
    /// The crossing's story so far: one line per scene and beat, each
    /// stamped with the voyage day it was written.
    #[serde(default)]
    pub log: Vec<LogEntry>,
    /// The leg that is underway (or just ended) included a drift.
    #[serde(default)]
    pub drifted_this_leg: bool,
    /// The finale playback has been surfaced once (spec 7 owns the rest).
    #[serde(default)]
    pub finale_shown: bool,
    /// Per-night watch overrides: (day index, soul). Editable until the
    /// night begins; persists offline.
    #[serde(default)]
    pub night_assignments: Vec<(u64, SoulId)>,
    /// A strange night already visited this leg (max one per leg).
    #[serde(default)]
    pub strange_seen_this_leg: bool,
    /// Silence-banks already listened into at Easy (one rumor per bank).
    #[serde(default)]
    pub heard_banks: Vec<u32>,
    /// Pilgrim ships already hailed (once per ship).
    #[serde(default)]
    pub hailed: Vec<u8>,
    /// Letters from home delivered so far (index into the sequence).
    #[serde(default)]
    pub letters_received: u8,
    /// The mail has stopped. Permanent; set by the Going-Dark's last beat.
    #[serde(default)]
    pub gone_dark: bool,
    /// Delivered letters waiting to be read (drained by the UI; serialized
    /// so mail that arrived offline greets the return).
    #[serde(default)]
    pub letter_events: Vec<u8>,
    /// Every soul met so far (unmet = absent). Older saves default to the
    /// launch trio.
    #[serde(default = "default_roster")]
    pub souls: Vec<SoulState>,
    /// A boarding ask waiting for an answer. Blocks departure — the ask is
    /// a door, and doors are answered, never slipped past.
    #[serde(default)]
    pub pending_ask: Option<SoulId>,
    /// Arc beats fired since last read (possibly offline) — the return
    /// view's log moments.
    #[serde(default)]
    pub soul_events: Vec<SoulEvent>,
    /// Scars on the hull (0..=HULL_WEAR_MAX): each adds 5% provisions
    /// burn. Mended only at shipyards, in place of a refit (spec 8).
    #[serde(default)]
    pub hull_wear: u8,
    /// Squalls that already took their price at Run (strain + scar), by id.
    #[serde(default)]
    pub priced_squalls: Vec<u32>,
    /// Silence banks that already strained the helm, by id.
    #[serde(default)]
    pub strained_banks: Vec<u32>,
    /// Prices paid, waiting to be read (drained by the UI like soul
    /// events; serialized so costs taken offline greet the return).
    #[serde(default)]
    pub passage_events: Vec<PassageEvent>,
    /// Which crossing of the era this is (spec 9). 1 = the authored
    /// pilgrimage; 2+ are ferry runs carrying passengers by the count.
    #[serde(default = "default_crossing_number")]
    pub crossing_number: u32,
    /// Passengers embarked this crossing (ferry runs only; 0 on the
    /// first). Cargo that eats: the hold burns faster the fuller she is.
    #[serde(default)]
    pub passengers: u32,
    /// The Drive speed multiplier baked in at launch (≤ 1.0). Held
    /// constant for the crossing so offline == live stays bitwise.
    #[serde(default = "default_one", alias = "resonance_time_mult")]
    pub drive_time_mult: f64,
    /// Port scenes the ship sailed through while the ferryman was away
    /// (auto-sail): played by the engine, waiting to be *read*. Drained
    /// oldest-first by the UI when the player returns.
    #[serde(default)]
    pub unread_scenes: Vec<ScenePlayback>,
}

fn default_provisions_cap() -> f64 {
    PROVISIONS_CAP
}

fn default_crossing_number() -> u32 {
    1
}

fn default_one() -> f64 {
    1.0
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
    /// three souls aboard, and everything ahead.
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
            provisions_cap: PROVISIONS_CAP,
            visited: vec![ROUTE_START],
            untaken: Vec::new(),
            rumors: Vec::new(),
            rumor_bought_this_visit: false,
            pending_recovery_scene: false,
            intro_pending: true,
            souls: default_roster(),
            pending_ask: None,
            soul_events: Vec::new(),
            refits: Vec::new(),
            refit_doors_seen: 0,
            pending_refit: None,
            keepsakes: Vec::new(),
            log: Vec::new(),
            drifted_this_leg: false,
            finale_shown: false,
            night_assignments: Vec::new(),
            strange_seen_this_leg: false,
            heard_banks: Vec::new(),
            hailed: Vec::new(),
            letters_received: 0,
            gone_dark: false,
            letter_events: Vec::new(),
            hull_wear: 0,
            priced_squalls: Vec::new(),
            strained_banks: Vec::new(),
            passage_events: Vec::new(),
            crossing_number: 1,
            passengers: 0,
            drive_time_mult: 1.0,
            unread_scenes: Vec::new(),
        }
    }

    /// A ferry run (crossing 2+, spec 9): the surviving crew sail again,
    /// rested; the hold carries passengers by the count; Drive and the
    /// Colony's districts fold their bonuses into a fresh crossing. The
    /// intro and the authored recruit asks are behind us — passengers, not
    /// pilgrims, ride now.
    ///
    /// `charge` (0.0..=1.0) is the Riftglass charge the wormhole jump was
    /// committed at (Dock/Wormhole, spec 9 addendum): `1.0` (full charge)
    /// begins the crossing exactly as before; anything less pre-applies a
    /// deterministic provisions/hull-wear deficit via
    /// `apply_partial_charge_penalty` (design.md Decision 4).
    pub fn begin_ferry(
        character_id: String,
        voyage_seed: u64,
        now: DateTime<Utc>,
        colony: &crate::vessel::colony::ColonyState,
        crew: Vec<SoulState>,
        charge: f64,
    ) -> Self {
        use crate::vessel::colony::District;
        let mut v = VoyageState::begin(character_id, voyage_seed, now);
        v.intro_pending = false;
        v.crossing_number = colony.crossings_completed + 2;
        v.passengers = colony.next_expedition();
        v.drive_time_mult = colony.drive_time_mult();

        // Coming home was rest: the crew sail sound, their stations kept
        // where they can be, arcs already told.
        v.souls = crew
            .into_iter()
            .filter(|s| s.status == SoulStatus::Aboard)
            .map(|mut s| {
                s.strain = 0;
                s.consecutive_watches = 0;
                s.rest_minutes = 0;
                s
            })
            .collect();

        // The colony's standing bonuses (districts derive from population).
        // A ferry run does not manage rations — the crossing's length answers
        // to Drive, not to how full the hold is — so the passenger load never
        // deepens her burn (see `provisions_mult_with`), and the base hold
        // (plus the Granary) always victuals her.
        if colony.has_district(District::Granary) {
            v.provisions_cap = PROVISIONS_CAP + 25.0;
            v.provisions = v.provisions_cap;
        }
        v.apply_partial_charge_penalty(charge);
        v
    }

    /// A wormhole jump committed at less than full Riftglass charge costs
    /// the new crossing a deterministic provisions/hull-wear deficit,
    /// scaled by how far short of full the charge was — no randomness, per
    /// the "no dice anywhere" pillar. A no-op at `charge >= 1.0`.
    fn apply_partial_charge_penalty(&mut self, charge: f64) {
        let deficit = 1.0 - charge.clamp(0.0, 1.0);
        if deficit <= 0.0 {
            return;
        }
        self.provisions =
            (self.provisions - MAX_PARTIAL_CHARGE_PROVISIONS_DEFICIT * deficit).max(0.0);
        self.hull_wear = (f64::from(MAX_PARTIAL_CHARGE_HULL_WEAR) * deficit).round() as u8;
    }

    pub fn has_refit(&self, refit: RefitId) -> bool {
        self.refits.contains(&refit)
    }

    // ── The price of passage (spec 8) ───────────────────────────────────────

    /// A soul takes a level of strain, with the cause on the receipt.
    /// Becoming worn clears their post — a worn soul cannot hold one.
    fn strain_soul(&mut self, id: SoulId, cause: StrainCause) {
        let Some(s) = self
            .souls
            .iter_mut()
            .find(|s| s.soul == id && s.status == SoulStatus::Aboard)
        else {
            return;
        };
        if s.strain >= 2 {
            return;
        }
        s.strain += 1;
        let level = s.strain;
        if level == 2 {
            s.station = None;
        }
        self.passage_events.push(PassageEvent::Strained {
            soul: id,
            level,
            cause,
        });
    }

    /// The hull takes a scar (capped): each one makes her eat 5% more.
    fn scar_hull(&mut self, cause: WearCause) {
        if self.hull_wear >= HULL_WEAR_MAX {
            return;
        }
        self.hull_wear += 1;
        self.passage_events.push(PassageEvent::Scarred {
            wear: self.hull_wear,
            cause,
        });
    }

    /// Drain the prices paid since last read (possibly offline).
    pub fn take_passage_events(&mut self) -> Vec<PassageEvent> {
        std::mem::take(&mut self.passage_events)
    }

    /// The yard's third door: mend the hull instead of refitting. Zeroes
    /// the scars; that yard's refit pair closes forever, unchosen.
    pub fn choose_mend(&mut self) -> bool {
        if self.pending_refit.is_none() {
            return false;
        }
        self.hull_wear = 0;
        self.refit_doors_seen += 1;
        self.pending_refit = None;
        self.push_log("Mended at the yard. The scars planed away.".to_string());
        true
    }

    // ── Multiplier composition ──────────────────────────────────────────────
    // Fixed order, documented once: time = base × trim × helm;
    // provisions = base × trim × tender. The UI only ever shows the composed
    // integers these produce.

    /// Leg-time multiplier at a hypothetical trim (the trim panel previews
    /// all four). The helm rides along.
    pub fn time_mult_with(&self, trim: Trim) -> f64 {
        let helm = self.station_soul(Station::Helm);
        let storm_sail = if self.has_refit(RefitId::StormSail) {
            0.90
        } else {
            1.00
        };
        // Drive (spec 9) sails her faster the more of the old world
        // she has carried — a constant baked in at launch.
        self.drive_time_mult
            * trim.time_mult()
            * helm_time_mult(
                helm.is_some(),
                // A strained hand loses the affine edge (spec 8).
                helm.is_some_and(|id| {
                    souls::soul(id).affinity == Some(Station::Helm) && self.soul_sound(id)
                }),
            )
            * storm_sail
    }

    /// True when the soul carries no strain (spec 8's affinity gate).
    fn soul_sound(&self, id: SoulId) -> bool {
        self.soul_state(id).is_some_and(|s| s.strain == 0)
    }

    /// Leg-provisions multiplier at a hypothetical trim.
    pub fn provisions_mult_with(&self, trim: Trim) -> f64 {
        let tender = self.station_soul(Station::Tender);
        // Mourning Colors deepens Restful's thrift (0.70 instead of 0.80).
        let trim_mult = if trim == Trim::Mourn && self.has_refit(RefitId::MourningColors) {
            0.70
        } else {
            trim.provisions_mult()
        };
        // Spec 8: scars make her eat.
        let wear = 1.0 + WEAR_BURN_PER_SCAR * f64::from(self.hull_wear);
        // Passengers are cargo that eats (spec 9): every passenger adds a
        // little to the daily burn, so scarcity becomes load management — but
        // only on the maiden voyage, where the hold is managed. A ferry run is
        // hands-off: no one meters the rations, the colony victuals the
        // expedition, and her length answers to Drive alone, not the headcount.
        let load = if self.crossing_number > 1 {
            1.0
        } else {
            1.0 + PROVISIONS_PER_PASSENGER * f64::from(self.passengers)
        };
        load * trim_mult
            * tender_provisions_mult(
                tender.is_some(),
                tender.is_some_and(|id| {
                    souls::soul(id).affinity == Some(Station::Tender) && self.soul_sound(id)
                }),
            )
            * wear
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
                let mut dp = (1.0 / MINUTES_PER_DAY as f64) / self.time_mult();
                let mut burn = dp
                    * (f64::from(road.base_provisions) / f64::from(road.base_days))
                    * self.provisions_mult();

                // The void's weather on this road, this hour. Derived, not
                // stored — offline sees the same sky by construction.
                let hour = self.processed_minutes / 60;
                for wx in weather::weather_on_road(self.voyage_seed, hour, road_id) {
                    match wx.kind {
                        WeatherKind::Current => {
                            if self.trim == Trim::Run {
                                if wx.with_bearing {
                                    // Riding it: big time gain, no extra burn.
                                    dp /= 0.85;
                                } else {
                                    // Running against it costs stores.
                                    burn *= 1.15;
                                }
                            }
                        }
                        WeatherKind::SilenceBank => {
                            if self.trim == Trim::Quiet {
                                // Quiet hears what the silence hides —
                                // once per bank.
                                if !self.heard_banks.contains(&wx.id) {
                                    self.heard_banks.push(wx.id);
                                    if self.grant_next_rumor(road.from) {
                                        self.push_log(format!(
                                            "In {}, held at Easy, the crew heard \
                                             what the silence was hiding.",
                                            wx.name
                                        ));
                                    }
                                }
                            } else {
                                // Not at Easy: the helm holds the silence
                                // alone (spec 8) — once per bank, the
                                // wheel-hand strains.
                                if !self.strained_banks.contains(&wx.id) {
                                    if let Some(helm) = self.station_soul(Station::Helm) {
                                        self.strained_banks.push(wx.id);
                                        self.strain_soul(helm, StrainCause::SilenceHelm);
                                    }
                                }
                            }
                        }
                        WeatherKind::Squall => {
                            let per_day = match self.trim {
                                Trim::Run => 4.0,
                                Trim::Quiet | Trim::Mourn => 1.0,
                                Trim::Cruise => 2.0,
                            };
                            burn += per_day / MINUTES_PER_DAY as f64;
                            // Run takes a squall on deck and on the skin
                            // (spec 8): once per squall, the first posted
                            // soul strains and the hull scars.
                            if self.trim == Trim::Run && !self.priced_squalls.contains(&wx.id) {
                                self.priced_squalls.push(wx.id);
                                self.scar_hull(WearCause::SquallAtRun);
                                let posted = [Station::Helm, Station::Tender, Station::Watch]
                                    .into_iter()
                                    .find_map(|p| self.station_soul(p));
                                if let Some(soul) = posted {
                                    self.strain_soul(soul, StrainCause::SquallAtRun);
                                }
                            }
                        }
                    }
                }

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

                let new_progress = progress_days + dp;
                if new_progress >= f64::from(road.base_days) {
                    self.arrive_at(road.to, Some(road_id));
                } else {
                    self.phase = VoyagePhase::Traveling {
                        road: road_id,
                        departed_at_min,
                        progress_days: new_progress,
                    };
                    // Midnight at sea: the day's night resolves.
                    if (self.processed_minutes + 1).is_multiple_of(MINUTES_PER_DAY) {
                        self.resolve_night(self.processed_minutes / MINUTES_PER_DAY, road_id);
                    }
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
            VoyagePhase::HoldingStation {
                waypoint,
                arrived_at_min,
                arrived_by,
                ..
            } => {
                // Auto-sail (pacing): a mid-crossing port with no decision —
                // exactly one road out, no ask, no refit — gets a brief port
                // call, then she sails herself; the scene is kept to read on
                // return. On a *ferry run* (crossing 2+) she also navigates
                // junctions herself, taking the first road on: the return
                // crossings are hands-off, and the crossing's length answers
                // to Drive, not to when the ferryman next looks in. The maiden
                // voyage (crossing 1) still holds at junctions for the choice.
                // `arrived_by: None` marks the pier (launch / Sail again):
                // those departures are always the ferryman's.
                let ferry_run = self.crossing_number > 1;
                // No hands work a ferry run's yard — skip the refit so she
                // does not stall on a decision no one is there to make.
                if ferry_run && self.pending_refit.is_some() {
                    self.pending_refit = None;
                }
                let mut roads = route::roads_from(waypoint);
                let auto_road = match (roads.next(), roads.next()) {
                    (Some(road), None) => Some(road.id),
                    (Some(road), Some(_)) if ferry_run => Some(road.id),
                    _ => None,
                };
                // A ferry run launches herself from the pier too (arrived_by
                // None); the maiden voyage waits there for the ferryman.
                let launch_ok = arrived_by.is_some() || ferry_run;
                if let (Some(road_id), true) = (auto_road, launch_ok) {
                    if self.processed_minutes - arrived_at_min >= PORT_CALL_GAME_MINUTES
                        && self.pending_ask.is_none()
                        && self.pending_refit.is_none()
                    {
                        if let Some(playback) = self.play_arrival_scene() {
                            self.unread_scenes.push(playback);
                        }
                        // Guards re-checked inside; a refused departure
                        // simply keeps holding.
                        let _ = self.depart(road_id);
                    }
                }
            }
            VoyagePhase::Arrived { .. } => {}
        }
    }

    fn arrive_at(&mut self, waypoint: WaypointId, arrived_by: Option<RoadId>) {
        // A leg made good at Run leaves a mark (spec 8): driving her hard
        // is the choice that wears her. Squalls and threat roads add more.
        if arrived_by.is_some() && self.trim == Trim::Run {
            self.scar_hull(WearCause::RunHard);
        }
        self.visited.push(waypoint);
        self.rumor_bought_this_visit = false;

        // Rest stops mend the off-post by one level (spec 8) — posts are
        // not rest; relieving someone is the decision.
        if route::waypoint(waypoint).has_feature(Feature::RestStop) {
            let mut healed = Vec::new();
            for s in &mut self.souls {
                if s.status == SoulStatus::Aboard && s.station.is_none() && s.strain > 0 {
                    s.strain -= 1;
                    healed.push(s.soul);
                }
            }
            for soul in healed {
                self.passage_events
                    .push(PassageEvent::HealedAtRest { soul });
            }
        }

        // The mail. Letters catch up to the Vessel at ports while home
        // still writes; the Threshold hands over the last one; and the
        // first arrival past the Last Lantern is the night the mail does
        // not come. Location-triggered, never timed (the covenant).
        let chapter = route::waypoint(waypoint).chapter;
        // Only the first crossing hears from home — the old world went
        // dark, permanently (spec 6). Ferry runs get colony letters
        // instead, driven from the loop (spec 9).
        if self.crossing_number == 1 && !self.gone_dark {
            if waypoint == WaypointId(23) {
                self.deliver_letter(LAST_LETTER_EVENT);
            } else if chapter <= Chapter::DriftRoads {
                let index = self.letters_received;
                if letters::letter_by_event(index).is_some() {
                    self.deliver_letter(index);
                }
                // Past twelve, the mail simply thins: arrivals get nothing,
                // which is its own foreshadowing.
                self.letters_received = self.letters_received.saturating_add(1);
            } else if self.visited.contains(&WaypointId(24)) && waypoint != WaypointId(24) {
                // The crew gathers at mail-hour, and nothing comes.
                self.gone_dark = true;
                self.push_log(letters::MAIL_FAILS_LOG.to_string());
                self.letter_events.push(MAIL_FAILS_EVENT);
            }
        }

        // A soul may be waiting here — but only the authored pilgrimage
        // recruits (crossing 1). Ferry runs embark passengers by the
        // count at launch, not named souls at ports.
        if self.crossing_number == 1 {
            if let Some(def) = souls::recruit_at(waypoint) {
                if !self.met(def.id) {
                    self.pending_ask = Some(def.id);
                }
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
    /// whose next beat is ready converts rest into story.
    fn step_arcs_minute(&mut self) {
        for i in 0..self.souls.len() {
            let s = self.souls[i];
            if s.status != SoulStatus::Aboard || s.station.is_some() {
                continue;
            }
            // Spec 8: a worn soul's story pauses; a strained soul rests
            // at half pace (a hurt person heals before they tell tales).
            if s.strain >= 2 {
                continue;
            }
            if s.strain == 1 && !self.processed_minutes.is_multiple_of(2) {
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
        self.push_log(title.clone());
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
                    self.scar_hull(WearCause::ThreatRoad);
                    (
                        vec!["You hurry the reef, and the Warden hurries with you. It \
                             takes its toll from the hold and something less \
                             replaceable from the crew's sleep."
                            .to_string()],
                        vec!["the Warden takes 15 \u{00b7} the hull scars".to_string()],
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
                    self.provisions = (self.provisions - 10.0).max(0.0);
                    (
                        vec!["Every soul stood a post and nobody held the middle of \
                             the ship. The Silence sat there instead, and the leg's \
                             log is three blank pages."
                            .to_string()],
                        vec!["the Silence takes 10".to_string()],
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
                    self.scar_hull(WearCause::ThreatRoad);
                    (
                        vec!["The thorns close on the hull and the quiet keel holds — \
                             barely, loudly, expensively. The crew spends the last \
                             mile listening to the wood argue and win."
                            .to_string()],
                        vec!["the thorns take 10".to_string()],
                    )
                } else {
                    // The exposure is the post, weakest first (spec 8):
                    // the most strained stationed soul, ties broken helm,
                    // tender, watch. Resting souls are below, and safe.
                    let exposed = [Station::Watch, Station::Tender, Station::Helm]
                        .into_iter()
                        .filter_map(|post| self.station_soul(post))
                        .max_by_key(|id| self.soul_state(*id).map(|s| s.strain).unwrap_or(0));
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
                            vec![format!("{name} is lost")],
                        )
                    } else {
                        self.provisions = (self.provisions - 20.0).max(0.0);
                        self.scar_hull(WearCause::ThreatRoad);
                        (
                            vec!["With every soul below, the ship takes the thorns on \
                                 her own skin. She holds. It costs the hold, and the \
                                 scars stay."
                                .to_string()],
                            vec!["the thorns take 20 \u{00b7} the hull scars".to_string()],
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
        self.push_log(format!("Refit: {}", chosen.display_name()));
        Some(chosen)
    }

    /// The finale at the Tree, surfaced once: W37's authored beats, then
    /// the rail (one line per soul aboard, boarding order), the carved
    /// names if any were lost, the Sister Verity in the harbor, and the
    /// closed door with the lamp. A pure function of the save — the same
    /// crossing always reads the same finale.
    pub fn take_finale_playback(&mut self) -> Option<ScenePlayback> {
        if !self.arrived() || self.finale_shown {
            return None;
        }
        self.finale_shown = true;
        let def = scenes::scene_def(ROUTE_SINK);
        let title = route::waypoint(ROUTE_SINK).name.to_string();
        self.push_log(title.clone());

        let mut paragraphs: Vec<String> = def.beats.iter().map(|b| b.to_string()).collect();
        // The rail: souls who stepped ashore earlier said their goodbyes
        // in the Log; the rail belongs to the ones who crossed.
        for s in &self.souls {
            if s.status == SoulStatus::Aboard {
                paragraphs.push(souls::soul(s.soul).rail_line.to_string());
            }
        }
        let carved = self.carved_names();
        if !carved.is_empty() {
            paragraphs.push(scenes::finale_carved_beat(&carved));
        }
        paragraphs.push(scenes::FINALE_HARBOR.to_string());
        paragraphs.push(scenes::FINALE_LAMP.to_string());

        Some(ScenePlayback {
            title,
            paragraphs,
            payout_note: "the crossing is over".to_string(),
        })
    }

    /// Set the ship's posture. Takes effect from the next unprocessed minute.
    pub fn set_trim(&mut self, trim: Trim) {
        self.trim = trim;
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
        // A new leg: the strange-night cap resets, and stale watch
        // overrides (nights already past) are pruned.
        self.strange_seen_this_leg = false;
        let today = self.processed_minutes / MINUTES_PER_DAY;
        self.night_assignments.retain(|(d, _)| *d >= today);
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

    /// The oldest port scene the ship sailed through unattended (auto-sail),
    /// if any — drained one at a time so the ferryman reads the ports in
    /// the order she made them.
    pub fn take_next_unread_scene(&mut self) -> Option<ScenePlayback> {
        if self.unread_scenes.is_empty() {
            None
        } else {
            Some(self.unread_scenes.remove(0))
        }
    }

    /// Deliver one letter: the parcel, the log, the queue.
    fn deliver_letter(&mut self, event: u8) {
        let Some(def) = letters::letter_by_event(event) else {
            return;
        };
        self.provisions = (self.provisions + LETTER_PARCEL_PROVISIONS).min(self.provisions_cap);
        self.push_log(format!("A letter from {}", def.sender));
        self.letter_events.push(event);
    }

    /// Drain letters waiting to be read (possibly delivered offline).
    pub fn take_letter_events(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.letter_events)
    }

    // ── Nights and the watch ────────────────────────────────────────────────

    /// The night's type on `day`, with the per-leg strange cap applied.
    pub fn night_kind(&self, day: u64) -> NightKind {
        let raw = nights::raw_night_kind(self.voyage_seed, day);
        if raw == NightKind::Strange && self.strange_seen_this_leg {
            NightKind::Quiet
        } else {
            raw
        }
    }

    /// Who stands `day`'s night if nothing changes: the per-night override,
    /// else the Watch-station soul.
    pub fn night_stander(&self, day: u64) -> Option<SoulId> {
        self.night_assignments
            .iter()
            .find(|(d, _)| *d == day)
            .map(|(_, s)| *s)
            .or_else(|| self.station_soul(Station::Watch))
    }

    /// Assign (or clear) a soul to a coming night. Editable until the
    /// night begins; standing it costs a resting soul a rest day.
    pub fn assign_night(&mut self, day: u64, soul: Option<SoulId>) {
        self.night_assignments.retain(|(d, _)| *d != day);
        if let Some(soul) = soul {
            if self
                .soul_state(soul)
                .is_some_and(|s| s.status == SoulStatus::Aboard)
            {
                self.night_assignments.push((day, soul));
            }
        }
    }

    /// The three-night forecast: (day, kind, who stands it).
    pub fn night_forecast(&self) -> Vec<(u64, NightKind, Option<SoulId>)> {
        let today = self.processed_minutes / MINUTES_PER_DAY;
        (0..3)
            .map(|i| {
                let day = today + i;
                (day, self.night_kind(day), self.night_stander(day))
            })
            .collect()
    }

    fn resolve_night(&mut self, day: u64, road: RoadId) {
        let kind = self.night_kind(day);
        if kind == NightKind::Strange {
            self.strange_seen_this_leg = true;
        }
        if !kind.needs_watch() {
            // A night that asks nothing is a night off: fatigue resets.
            for s in &mut self.souls {
                s.consecutive_watches = 0;
            }
            return; // quiet nights ask nothing and write nothing
        }

        // Who stands it: the assignment/watch soul; else a matched-course
        // pilgrim crew (their roads coincide and you aren't Running); else
        // nobody.
        let stander = self.night_stander(day);
        let (outcome, name) = if let Some(soul_id) = stander {
            let def = souls::soul(soul_id);
            // A resting soul standing a night is not resting that night.
            if let Some(s) = self.souls.iter_mut().find(|s| s.soul == soul_id) {
                if s.station.is_none() {
                    s.rest_minutes = s.rest_minutes.saturating_sub(MINUTES_PER_DAY);
                }
            }
            if def.affinity == Some(Station::Watch) && self.soul_sound(soul_id) {
                (NightOutcome::StoodAffine, Some(def.name))
            } else {
                (NightOutcome::Stood, Some(def.name))
            }
        } else if self.trim != Trim::Run
            && pilgrims::pilgrims_on(day).iter().any(|(_, r)| *r == road)
        {
            (NightOutcome::StoodByPilgrim, None)
        } else {
            (NightOutcome::Unstood, None)
        };

        // Watch fatigue (spec 8): the stander's count rises, everyone
        // else's night off resets theirs; the third night in a row strains.
        for s in &mut self.souls {
            if s.status != SoulStatus::Aboard {
                continue;
            }
            if Some(s.soul) == stander {
                s.consecutive_watches = s.consecutive_watches.saturating_add(1);
            } else {
                s.consecutive_watches = 0;
            }
        }
        if let Some(id) = stander {
            if self
                .soul_state(id)
                .is_some_and(|s| s.consecutive_watches >= 3)
            {
                if let Some(s) = self.souls.iter_mut().find(|s| s.soul == id) {
                    s.consecutive_watches = 0;
                }
                self.strain_soul(id, StrainCause::ThirdWatch);
            }
        }

        let effect = nights::night_effect(kind, outcome);
        if effect.provisions < 0.0 {
            self.provisions = (self.provisions + effect.provisions).max(0.0);
        }
        if effect.rumor {
            // A singing/strange night stood by the right soul yields a rumor
            // (a no-op once every rumor is already known).
            self.grant_next_rumor(route::road(road).from);
        }
        self.push_log(nights::log_line(kind, outcome, name));
    }

    /// Learn the next authored rumor not yet held (weather, nights, and
    /// pilgrims all pay through here). Returns false when all are known.
    fn grant_next_rumor(&mut self, learned_at: WaypointId) -> bool {
        let next = route::RUMORS.iter().find(|r| !self.knows_rumor(r.id));
        if let Some(def) = next {
            self.rumors.push(LearnedRumor {
                rumor: def.id,
                learned_at,
            });
            true
        } else {
            false
        }
    }

    // ── Weather and pilgrims (read model + hail) ────────────────────────────

    /// Weather visible on the chart right now.
    pub fn weather_visible(&self) -> Vec<WeatherObj> {
        weather::weather_at(self.voyage_seed, self.processed_minutes / 60)
    }

    /// Weather on the road underway, if traveling.
    pub fn weather_on_leg(&self) -> Vec<WeatherObj> {
        match self.phase {
            VoyagePhase::Traveling { road, .. } | VoyagePhase::Drifting { road, .. } => {
                weather::weather_on_road(self.voyage_seed, self.processed_minutes / 60, road)
            }
            _ => Vec::new(),
        }
    }

    /// Pilgrim ships abroad today, with their roads.
    pub fn pilgrims_today(&self) -> Vec<(&'static PilgrimShip, RoadId)> {
        pilgrims::pilgrims_on(self.processed_minutes / MINUTES_PER_DAY)
    }

    /// A ship in hailing range: sharing your road, not yet hailed.
    pub fn hailable(&self) -> Option<&'static PilgrimShip> {
        let road = self.current_road()?.id;
        self.pilgrims_today()
            .into_iter()
            .find(|(s, r)| *r == road && !self.hailed.contains(&s.id))
            .map(|(s, _)| s)
    }

    /// Hail her. Once per ship for the whole crossing: an exchange of
    /// news, and their rumor becomes your chart annotation.
    pub fn hail(&mut self) -> Option<&'static PilgrimShip> {
        let ship = self.hailable()?;
        self.hailed.push(ship.id);
        let at = self.current_road().map(|r| r.from).unwrap_or(ROUTE_START);
        self.grant_next_rumor(at);
        self.push_log(format!("Hailed {}. {}", ship.name, ship.hail));
        Some(ship)
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
        // A worn soul cannot hold a post (spec 8).
        if station.is_some() && self.soul_state(id).is_some_and(|s| s.strain >= 2) {
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
    /// the crew is full — free a seat with [`Self::farewell`] first, or decline.
    pub fn accept_ask(&mut self) -> bool {
        let Some(id) = self.pending_ask else {
            return false;
        };
        if self.aboard_count() >= CREW {
            return false;
        }
        self.pending_ask = None;
        self.souls.push(SoulState {
            soul: id,
            status: SoulStatus::Aboard,
            station: None,
            arc_beat: 0,
            rest_minutes: 0,
            left_at: None,
            strain: 0,
            consecutive_watches: 0,
        });
        true
    }

    /// Say no. Permanent: the door closes, the name stays on the chart.
    pub fn decline_ask(&mut self) -> Option<SoulId> {
        let id = self.pending_ask.take()?;
        let here = self.current_waypoint();
        self.souls.push(SoulState {
            soul: id,
            status: SoulStatus::Declined,
            station: None,
            arc_beat: 0,
            rest_minutes: 0,
            left_at: here,
            strain: 0,
            consecutive_watches: 0,
        });
        Some(id)
    }

    /// A soul steps ashore to free a seat. Remembered in the manifest,
    /// never carved into the hull.
    pub fn farewell(&mut self, id: SoulId) -> bool {
        let here = self.current_waypoint();
        let Some(s) = self
            .souls
            .iter_mut()
            .find(|s| s.soul == id && s.status == SoulStatus::Aboard)
        else {
            return false;
        };
        s.status = SoulStatus::Ashore;
        s.station = None;
        s.left_at = here;
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

    /// Write a line into the ship's log, stamped with today.
    fn push_log(&mut self, text: String) {
        let day = self.day_index();
        self.log.push(LogEntry { day, text });
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

    pub fn arrived(&self) -> bool {
        matches!(self.phase, VoyagePhase::Arrived { .. })
    }

    /// Provisions as the bar shows them: a small whole number.
    pub fn provisions_display(&self) -> u32 {
        self.provisions.round().max(0.0) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn t0() -> DateTime<Utc> {
        "2026-07-03T12:00:00Z".parse().unwrap()
    }

    /// Real duration in which `h` *game*-hours pass (scale-aware) — for
    /// tests that assert exact points in game time.
    fn gh(h: i64) -> Duration {
        real_duration_for_game_minutes(h * 60)
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

        // Road 0 is 1.0 base days at Cruise: not there at 23 game-hours...
        v.tick(t0() + gh(23));
        assert!(matches!(v.phase, VoyagePhase::Traveling { .. }));
        // ...there within the next two.
        v.tick(t0() + gh(25));
        assert_eq!(v.current_waypoint(), Some(road.to));
        assert!(matches!(
            v.phase,
            VoyagePhase::HoldingStation {
                scene_state: SceneState::Waiting,
                ..
            }
        ));
        // The leg burned about its base price — at least the base, plus
        // whatever weather it met (net of the letter parcel that was waiting
        // at the arrival — spec 6).
        let burned =
            LAUNCH_PROVISIONS - v.provisions + crate::vessel::letters::LETTER_PARCEL_PROVISIONS;
        let base = f64::from(road.base_provisions);
        assert!(
            (base - 1.0..=base + 6.0).contains(&burned),
            "burned {burned}, expected ~{base} (+ any weather)"
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

        // At 20 game-hours: Run (0.8 days = 19.2h) has arrived, Cruise has not.
        let at = t0() + gh(20);
        cruise.tick(at);
        run.tick(at);
        assert!(matches!(cruise.phase, VoyagePhase::Traveling { .. }));
        assert!(matches!(run.phase, VoyagePhase::HoldingStation { .. }));
        let run_burn =
            LAUNCH_PROVISIONS - run.provisions + crate::vessel::letters::LETTER_PARCEL_PROVISIONS;
        assert!(
            (run_burn - f64::from(road.base_provisions) * 1.30).abs() < 1.0,
            "run should burn ~130%, burned {run_burn}"
        );
    }

    #[test]
    fn restful_is_the_thriftiest_hold() {
        // Restful (Mourn) now earns its identity through thrift, not hope:
        // it burns less per leg than every other pace.
        let v = started();
        let restful = v.provisions_mult_with(Trim::Mourn);
        assert!(restful < v.provisions_mult_with(Trim::Quiet));
        assert!(restful < v.provisions_mult_with(Trim::Cruise));
        assert!(restful < v.provisions_mult_with(Trim::Run));
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
        v.tick(t0() + gh(12));
        let VoyagePhase::Drifting { progress_days, .. } = v.phase else {
            panic!("expected drift, got {:?}", v.phase);
        };
        assert!(progress_days > 0.3 && progress_days < 0.5);
        assert_eq!(v.provisions, 0.0);

        // 36 hours later: recovered, resumed at the same progress, +25 —
        // minus whatever the nights and weather priced in the interim.
        v.tick(t0() + gh(12 + 36 + 1));
        assert!(matches!(v.phase, VoyagePhase::Traveling { .. }));
        assert!(v.provisions > 10.0 && v.provisions <= 25.0);
        assert!(v.take_pending_recovery_scene());
        assert!(!v.take_pending_recovery_scene(), "shown once");

        // And the leg still finishes.
        v.tick(t0() + gh(4 * 24));
        assert_eq!(v.current_waypoint(), Some(road.to));
    }

    #[test]
    fn plain_ports_auto_sail_after_the_port_call_and_keep_the_scene() {
        // W0 -> W1 (the Lightship Vigil): one road onward, but Maren asks
        // to board there — the ask holds the ship until it is answered.
        let mut v = started();
        let road = route::roads_from(ROUTE_START).next().unwrap();
        v.depart(road.id).unwrap();
        v.tick(t0() + gh(24 + 12));
        assert!(
            v.pending_ask.is_some(),
            "an ask is a decision: the ship waits"
        );
        assert!(matches!(v.phase, VoyagePhase::HoldingStation { .. }));

        // Decline her; the next step_minute makes the port call and sails.
        v.decline_ask();
        v.tick(t0() + gh(24 + 13));
        assert!(
            matches!(v.phase, VoyagePhase::Traveling { .. }),
            "no decision left: she sails herself, got {:?}",
            v.phase
        );
        // The scene was played by the engine and kept for the ferryman.
        assert_eq!(v.unread_scenes.len(), 1);
        assert_eq!(
            v.unread_scenes[0].title,
            route::waypoint(road.to).name,
            "the auto-sailed port's scene waits to be read"
        );

        // The pier never auto-sails: a fresh voyage holds at the start
        // (arrived_by: None) for as long as the ferryman needs.
        let mut fresh = started();
        fresh.tick(t0() + gh(24 * 30));
        assert!(
            matches!(
                fresh.phase,
                VoyagePhase::HoldingStation {
                    waypoint: ROUTE_START,
                    ..
                }
            ),
            "launch is the ferryman's act, never the engine's"
        );
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
        assert_eq!(live.processed_minutes, offline.processed_minutes);
    }
}
