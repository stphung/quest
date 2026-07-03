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

/// A ferry run's base passenger berths (crossing 1 carries the authored
/// cast, not passengers — see `ferry_capacity`). The Shipyard district
/// grows this.
pub const FERRY_CAPACITY_BASE: u32 = 40;

/// Resonance curve: the time multiplier shrinks from 1.0 toward this
/// floor as resonance grows (0.5 = twice her launch speed at the cap).
pub const RESONANCE_SPEED_FLOOR: f64 = 0.5;
/// Half-speedup point: at this much resonance she sails at the midpoint
/// between 1.0 and the floor.
pub const RESONANCE_HALF: f64 = 500.0;

/// Souls the dark takes per game-day underway, at the era's start. The
/// rate climbs as the world empties (see [`ColonyState::dimming_loss`]).
pub const DIMMING_BASE_PER_DAY: f64 = 1.5;

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

    /// Population at which this district is founded.
    pub fn threshold(&self) -> u64 {
        match self {
            District::Quay => 25,
            District::Granary => 60,
            District::Hearth => 150,
            District::Shipyard => 400,
            District::Beacon => 1_000,
            District::Charthouse => 2_500,
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
            District::Quay => "the running back is quicker",
            District::Granary => "the hold carries more; way-stations pay better",
            District::Hearth => "the ship launches brighter; rest heals fully",
            District::Shipyard => "half again the berths; the hull mends at the Tree",
            District::Beacon => "the crossings resonate louder",
            District::Charthouse => "the chart's knowledge keeps between crossings",
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
            .filter(|d| self.population() >= d.threshold())
            .collect()
    }

    pub fn has_district(&self, d: District) -> bool {
        self.population() >= d.threshold()
    }

    /// Passenger berths a ferry run carries — base, grown by the Shipyard.
    pub fn ferry_capacity(&self) -> u32 {
        if self.has_district(District::Shipyard) {
            (FERRY_CAPACITY_BASE as f64 * 1.5) as u32
        } else {
            FERRY_CAPACITY_BASE
        }
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
        let speedup = (1.0 - RESONANCE_SPEED_FLOOR) * (r / (r + RESONANCE_HALF));
        1.0 - speedup
    }

    /// A human "×N.N her old self" for the Reckoning.
    pub fn resonance_speed_factor(&self) -> f64 {
        1.0 / self.resonance_time_mult()
    }

    /// The dark's toll over a span of game-days underway — a pure,
    /// accelerating function of how far into the era those days fall.
    /// The world empties faster the emptier it already is.
    pub fn dimming_loss(&self, day_from: u64, day_to: u64) -> u64 {
        let mut lost = 0.0;
        for day in day_from..day_to {
            // Accelerates: +1% of the base per day elapsed in the era.
            lost += DIMMING_BASE_PER_DAY * (1.0 + day as f64 * 0.01);
        }
        lost.round() as u64
    }

    /// The game-day (since the era began) a port goes dark. Deterministic
    /// per (era_seed, port) — the chart dims in *this* world's order.
    pub fn port_dim_day(&self, port: WaypointId) -> u64 {
        // Spread across a long era; earlier ports (nearer home) tend to
        // dim sooner, so the dark rolls outward from the old world.
        let base = mix64(self.era_seed ^ (port.0 as u64).wrapping_mul(0x9E37));
        let spread = 30 + (base % 120); // 30–150 game-days
        let nearness = (port.0 as u64).min(37);
        spread.saturating_sub(nearness / 2)
    }

    /// Ports dark as of `day` (drives the chart's dimming). Never the
    /// start or the Tree — home pier and destination always stand.
    pub fn dimmed_as_of(&self, day: u64) -> Vec<WaypointId> {
        WAYPOINTS
            .iter()
            .map(|w| w.id)
            .filter(|id| *id != ROUTE_START && *id != ROUTE_SINK)
            .filter(|id| self.port_dim_day(*id) <= day)
            .collect()
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

        // The dark took its share of the rest while we sailed.
        let toll = self.dimming_loss(0, days).min(self.souls_remaining);
        self.souls_remaining -= toll;

        self.crossings_completed += 1;
        self.records.total_leagues += leagues;
        self.records.total_nights += nights;
        self.records.most_carried = self.records.most_carried.max(passengers);
        if days > 0 && (self.records.fastest_days == 0 || days < self.records.fastest_days) {
            self.records.fastest_days = days;
        }
        self.dimmed_ports = self.dimmed_as_of(self.crossings_completed as u64 * 35);

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
        c.resonance = RESONANCE_HALF as u64;
        assert!((c.resonance_time_mult() - 0.75).abs() < 0.01, "half point");
        c.resonance = 1_000_000;
        assert!(c.resonance_time_mult() > RESONANCE_SPEED_FLOOR - 0.01);
        assert!(c.resonance_time_mult() < RESONANCE_SPEED_FLOOR + 0.02);
    }

    #[test]
    fn capacity_grows_with_the_shipyard() {
        let mut c = ColonyState::found("t".into());
        assert_eq!(c.ferry_capacity(), FERRY_CAPACITY_BASE);
        c.souls_delivered = 400;
        assert_eq!(c.ferry_capacity(), 60);
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
