//! The Colony — Act 2's incremental spine (sub-project 9, The Ferryman).
//!
//! The crossing is the *run*; the Colony is what persists above it. Every
//! crossing delivers its souls to the living branch, and the number only
//! ever rises — while the old world dims behind you, port by port, on a
//! deterministic schedule. When the last souls are gone (delivered or
//! taken by the dark), the next arrival is the Last Crossing: Act 3's gate.
//!
//! `ColonyState` lives in `colony.json`, keyed by character id like the
//! voyage — but it survives every crossing where the voyage is replaced.
//!
//! See `docs/superpowers/specs/2026-07-03-vessel-ferryman-design.md`.

use super::route::{WaypointId, ROUTE_SINK, ROUTE_START, WAYPOINTS};
use serde::{Deserialize, Serialize};

/// Souls in the dying world at the first launch. The whole era spends
/// this pool down — some carried across, the rest lost to the dark.
pub const INITIAL_SOULS: u64 = 3_000;

/// How many souls the *first* ferry carries, before any district is built
/// (crossing 1 carries the authored cast, not passengers — see
/// `ferry_capacity`). Every founded district adds more berths on top, so
/// the cohorts swell as the colony does — a whole era is a handful of big,
/// deliberate crossings, not a long drip of small ones.
pub const FERRY_BERTHS_AT_LAUNCH: u32 = 160;

/// The quickest a crossing can ever get, as a fraction of its launch time
/// (0.5 = twice as fast). Resonance pulls crossing time down toward this.
pub const FASTEST_CROSSING_TIME_MULT: f64 = 0.5;
/// How much resonance it takes to reach *halfway* to top speed. Set high
/// so a short era's crossings stay weighty — each shortens only a little,
/// never snapping straight to the fastest.
pub const RESONANCE_FOR_HALF_SPEEDUP: f64 = 2_500.0;

/// The share of the still-waiting world the dark takes each crossing —
/// a visible per-crossing toll, not a slow drip. It bites hardest while
/// the world is full (you are losing the race), and eases as it empties
/// (you catch up, and carry the last of them home yourself).
pub const DARK_TAKES_EACH_CROSSING: f64 = 0.065;

/// The colony's districts, unlocked in order by population. Pure growth —
/// every one lands eventually; the choices live on the water.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum District {
    Quay,
    Granary,
    Hearth,
    Shipyard,
    Beacon,
    Charthouse,
}

impl District {
    pub const ALL: [District; 6] = [
        District::Quay,
        District::Granary,
        District::Hearth,
        District::Shipyard,
        District::Beacon,
        District::Charthouse,
    ];

    /// Population at which this district is founded. Spaced so that a
    /// full era founds one district per crossing — six crossings, six
    /// beats, the Charthouse landing on the last of them.
    pub fn founded_at(&self) -> u64 {
        match self {
            District::Quay => 3,
            District::Granary => 260,
            District::Hearth => 620,
            District::Shipyard => 1_080,
            District::Beacon => 1_620,
            District::Charthouse => 2_150,
        }
    }

    /// Passenger berths this district adds to the ferry once founded. The
    /// colony you build is the ship that carries the next crossing — so
    /// each district makes the next cohort larger.
    pub fn added_berths(&self) -> u32 {
        match self {
            District::Quay => 110,
            District::Granary => 140,
            District::Hearth => 170,
            District::Shipyard => 210,
            District::Beacon => 260,
            District::Charthouse => 320,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            District::Quay => "The Quay",
            District::Granary => "The Granary",
            District::Hearth => "The Hearth",
            District::Shipyard => "The Shipyard",
            District::Beacon => "The Beacon",
            District::Charthouse => "The Charthouse",
        }
    }

    pub fn bonus(&self) -> &'static str {
        match self {
            District::Quay => "proper berths at last — the first real cohort",
            District::Granary => "stores to victual a fuller ship",
            District::Hearth => "warmth enough to carry more, and rest heals fully",
            District::Shipyard => {
                "the great slips; the largest cohorts, and the hull mends at the Tree"
            }
            District::Beacon => "a light the far shore steers by; the crossings resonate louder",
            District::Charthouse => "the whole colony behind each sailing",
        }
    }
}

/// The trophy shelf — records, not the engine.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossingRecords {
    pub fastest_days: u64,
    pub most_carried: u32,
    pub total_leagues: u64,
    pub total_nights: u64,
}

/// The persistent side of Act 2's loop. Population is exactly
/// `souls_delivered` — the colony keeps everyone it receives.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColonyState {
    pub character_id: String,
    /// The headline number: souls carried out of the dark, lifetime.
    pub souls_delivered: u64,
    /// The pressure: souls still in the dying world (delivery and the
    /// dimming both spend it down).
    pub souls_remaining: u64,
    /// The engine: rises with every delivery, multiplies every crossing.
    pub resonance: u64,
    pub crossings_completed: u32,
    #[serde(default)]
    pub records: CrossingRecords,
    /// Ports the dark has already taken (cosmetic on the chart; the
    /// souls they held are already out of `souls_remaining`).
    #[serde(default)]
    pub dimmed_ports: Vec<WaypointId>,
    /// The era's seed — one per account, so the dimming order is *this*
    /// world's story (derived from the character id at founding).
    pub era_seed: u64,
}

impl ColonyState {
    /// Found the colony the moment the first crossing arrives.
    pub fn found(character_id: String) -> Self {
        let era_seed = mix64(hash_id(&character_id));
        ColonyState {
            character_id,
            souls_delivered: 0,
            souls_remaining: INITIAL_SOULS,
            resonance: 0,
            crossings_completed: 0,
            records: CrossingRecords::default(),
            dimmed_ports: Vec::new(),
            era_seed,
        }
    }

    /// Population = souls delivered.
    pub fn population(&self) -> u64 {
        self.souls_delivered
    }

    /// Districts founded so far.
    pub fn districts(&self) -> Vec<District> {
        District::ALL
            .into_iter()
            .filter(|d| self.population() >= d.founded_at())
            .collect()
    }

    pub fn has_district(&self, d: District) -> bool {
        self.population() >= d.founded_at()
    }

    /// Passenger berths a ferry run carries: the launch base plus every
    /// founded district's berths. The colony grows the ship, so the
    /// cohorts grow with it.
    pub fn ferry_capacity(&self) -> u32 {
        FERRY_BERTHS_AT_LAUNCH
            + self
                .districts()
                .iter()
                .map(|d| d.added_berths())
                .sum::<u32>()
    }

    /// How many souls the next ferry run embarks: capacity, or whatever
    /// the world has left, whichever is smaller.
    pub fn next_passengers(&self) -> u32 {
        (self.ferry_capacity() as u64).min(self.souls_remaining) as u32
    }

    /// The Resonance time multiplier (≤ 1.0): the Vessel sails faster the
    /// more of the old world she has carried. Soft-capped at the floor.
    pub fn resonance_time_mult(&self) -> f64 {
        let r = self.resonance as f64;
        let speedup = (1.0 - FASTEST_CROSSING_TIME_MULT) * (r / (r + RESONANCE_FOR_HALF_SPEEDUP));
        1.0 - speedup
    }

    /// A human "×N.N her old self" for the Reckoning.
    pub fn resonance_speed_factor(&self) -> f64 {
        1.0 / self.resonance_time_mult()
    }

    /// Souls the dark takes this crossing: a fixed share of whoever is
    /// still waiting. A big, visible bite while the world is full; a small
    /// one once it has emptied — so late crossings are yours to finish.
    pub fn dark_toll(&self) -> u64 {
        (self.souls_remaining as f64 * DARK_TAKES_EACH_CROSSING).round() as u64
    }

    /// The order a port goes dark — deterministic per (era_seed, port), so
    /// the chart dims in *this* world's story. Lower goes first; ports nearer
    /// home (lower id) carry a small bias to go first, so the dark rolls
    /// outward from the old world. Only a sort key, not an absolute day —
    /// the *pace* of the blackout is set by how empty the world is
    /// (see [`dark_ports`](Self::dark_ports)).
    pub fn port_dim_order(&self, port: WaypointId) -> u64 {
        let base = mix64(self.era_seed ^ (port.0 as u64).wrapping_mul(0x9E37));
        let spread = 30 + (base % 120);
        let nearness = (port.0 as u64).min(37);
        spread.saturating_sub(nearness / 2)
    }

    /// The ports the dark has taken so far, paced to how empty the world is:
    /// the blackout keeps `port_dim_order` (this world's story) but spreads
    /// across the *whole* era, so the chart empties in step with the manifest
    /// rather than all at once in the first few crossings. Never the start or
    /// the Tree — home pier and destination always stand. Drives the chart's
    /// dimming.
    pub fn dark_ports(&self) -> Vec<WaypointId> {
        let mut dimmable: Vec<WaypointId> = WAYPOINTS
            .iter()
            .map(|w| w.id)
            .filter(|id| *id != ROUTE_START && *id != ROUTE_SINK)
            .collect();
        // Fraction of the old world no longer out there (delivered or taken
        // by the dark): 0 at launch, 1 when the era ends.
        let gone = INITIAL_SOULS.saturating_sub(self.souls_remaining);
        let frac = (gone as f64 / INITIAL_SOULS as f64).clamp(0.0, 1.0);
        let count = (frac * dimmable.len() as f64).round() as usize;
        dimmable.sort_by_key(|id| (self.port_dim_order(*id), id.0));
        dimmable.truncate(count);
        dimmable
    }

    /// Fold a completed crossing into the colony: deliver its passengers,
    /// grow resonance and population, spend the dark's toll, keep records.
    /// Returns the districts newly founded (for the letters that greet them).
    pub fn deliver_crossing(
        &mut self,
        passengers: u32,
        days: u64,
        leagues: u64,
        nights: u64,
    ) -> Vec<District> {
        let before = self.districts();

        let carried = (passengers as u64).min(self.souls_remaining);
        self.souls_delivered += carried;
        self.souls_remaining -= carried;
        self.resonance += carried;

        // The dark took its share of whoever was still waiting.
        let toll = self.dark_toll().min(self.souls_remaining);
        self.souls_remaining -= toll;

        self.crossings_completed += 1;
        self.records.total_leagues += leagues;
        self.records.total_nights += nights;
        self.records.most_carried = self.records.most_carried.max(passengers);
        if days > 0 && (self.records.fastest_days == 0 || days < self.records.fastest_days) {
            self.records.fastest_days = days;
        }
        self.dimmed_ports = self.dark_ports();

        District::ALL
            .into_iter()
            .filter(|d| !before.contains(d) && self.districts().contains(d))
            .collect()
    }

    /// True once the old world is empty — the next arrival is the end.
    pub fn era_over(&self) -> bool {
        self.souls_remaining == 0
    }
}

/// SplitMix64 — a tiny deterministic mixer (same family as weather's).
fn mix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E3779B97F4A7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
    x ^ (x >> 31)
}

fn hash_id(id: &str) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for b in id.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn districts_unlock_in_order_by_population() {
        let mut c = ColonyState::found("t".into());
        assert!(c.districts().is_empty());
        c.souls_delivered = 30;
        assert_eq!(c.districts(), vec![District::Quay]);
        c.souls_delivered = 2_500;
        assert_eq!(c.districts().len(), 6, "the whole colony is founded");
    }

    #[test]
    fn resonance_speeds_her_up_toward_the_floor() {
        let mut c = ColonyState::found("t".into());
        assert!(
            (c.resonance_time_mult() - 1.0).abs() < 1e-9,
            "launch: no bonus"
        );
        c.resonance = RESONANCE_FOR_HALF_SPEEDUP as u64;
        assert!((c.resonance_time_mult() - 0.75).abs() < 0.01, "half point");
        c.resonance = 1_000_000;
        assert!(c.resonance_time_mult() > FASTEST_CROSSING_TIME_MULT - 0.01);
        assert!(c.resonance_time_mult() < FASTEST_CROSSING_TIME_MULT + 0.02);
    }

    #[test]
    fn capacity_grows_with_every_district() {
        let mut c = ColonyState::found("t".into());
        assert_eq!(
            c.ferry_capacity(),
            FERRY_BERTHS_AT_LAUNCH,
            "launch: base only"
        );
        // Quay + Granary founded → base plus both their berths.
        c.souls_delivered = 300;
        assert_eq!(c.ferry_capacity(), 160 + 110 + 140);
        // The whole colony founded → every district's berths stack.
        c.souls_delivered = 2_200;
        assert_eq!(c.ferry_capacity(), 160 + 110 + 140 + 170 + 210 + 260 + 320);
    }

    #[test]
    fn delivering_grows_the_colony_and_the_dark_takes_its_share() {
        let mut c = ColonyState::found("t".into());
        let new = c.deliver_crossing(40, 35, 200, 12);
        assert_eq!(c.souls_delivered, 40);
        assert_eq!(c.crossings_completed, 1);
        assert_eq!(c.records.most_carried, 40);
        assert_eq!(c.records.fastest_days, 35);
        assert!(
            new.contains(&District::Quay),
            "40 delivered founds the Quay"
        );
        // The pool fell by more than the 40 carried — the dimming took some.
        assert!(c.souls_remaining < INITIAL_SOULS - 40);
    }

    #[test]
    fn the_dark_spreads_in_step_with_the_emptying_world() {
        let mut c = ColonyState::found("t".into());
        let dimmable = WAYPOINTS
            .iter()
            .filter(|w| w.id != ROUTE_START && w.id != ROUTE_SINK)
            .count();

        // At launch the whole world is lit; the era's end leaves it all dark.
        assert!(c.dark_ports().is_empty(), "launch: nothing has gone dark");
        c.souls_remaining = 0;
        assert_eq!(c.dark_ports().len(), dimmable, "era's end: all dark");

        // Halfway emptied dims about half the ports, and the blackout only
        // ever grows — the dark is never un-taken.
        c.souls_remaining = INITIAL_SOULS / 2;
        let half = c.dark_ports();
        assert!(
            (half.len() as i64 - (dimmable as i64 / 2)).abs() <= 1,
            "half emptied ⇒ ~half dark, got {}",
            half.len()
        );
        c.souls_remaining = INITIAL_SOULS / 4;
        let later = c.dark_ports();
        assert!(later.len() > half.len(), "the dark only spreads");
        let half_set: std::collections::HashSet<_> = half.iter().collect();
        assert!(
            half_set.iter().all(|id| later.contains(id)),
            "a port that went dark stays dark"
        );
        // The home pier and the Tree always stand.
        assert!(!later.contains(&ROUTE_START) && !later.contains(&ROUTE_SINK));
    }

    #[test]
    fn the_era_ends_when_the_world_empties() {
        let mut c = ColonyState::found("t".into());
        c.souls_remaining = 30;
        assert!(!c.era_over());
        c.deliver_crossing(60, 35, 100, 10); // carries 30, dark takes 0 more
        assert!(c.era_over(), "the last souls carried out ends the era");
    }

    #[test]
    fn founding_is_deterministic_per_character() {
        assert_eq!(
            ColonyState::found("abc".into()).era_seed,
            ColonyState::found("abc".into()).era_seed
        );
        assert_ne!(
            ColonyState::found("abc".into()).era_seed,
            ColonyState::found("xyz".into()).era_seed
        );
    }
}
