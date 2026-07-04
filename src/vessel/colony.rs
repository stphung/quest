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
/// this pool down — some carried across, the rest lost to the dark. A big
/// pool so the campaign is long: dozens of crossings, the headline number
/// climbing into the tens of thousands.
pub const INITIAL_SOULS: u64 = 100_000;

/// How many souls the *first* ferry carries, before the colony has grown it
/// (crossing 1 carries the authored cast, not passengers — see
/// `expedition_size`).
pub const EXPEDITION_AT_LAUNCH: u32 = 160;

/// Berths the ferry gains for every 1,000 souls already delivered — the
/// colony you build is the ship that carries the next crossing, so the
/// expedition grows on *every* run, not just when a district is founded.
/// This is what keeps a long era feeling like progress the whole way.
pub const EXPEDITION_PER_1000_DELIVERED: u64 = 75;

/// The quickest a crossing can ever get, as a fraction of its launch time
/// (0.5 = twice as fast). Drive pulls crossing time down toward this.
pub const FASTEST_CROSSING_TIME_MULT: f64 = 0.5;
/// How much drive it takes to reach *halfway* to top speed. Set high
/// so a short era's crossings stay weighty — each shortens only a little,
/// never snapping straight to the fastest.
pub const DRIVE_FOR_HALF_SPEEDUP: f64 = 2_500.0;

/// The share of the still-waiting world the dark takes each crossing —
/// a visible per-crossing toll, not a slow drip. Small, because a long era
/// gives it many crossings to compound over: at this rate you still carry
/// ~80% of the world home, and lose the other fifth to the dark.
pub const DARK_TAKES_EACH_CROSSING: f64 = 0.007;

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

    /// Population at which this district is founded. Spaced across the whole
    /// long era so a milestone lands roughly every several crossings — the
    /// Quay early, the Charthouse near the finale.
    pub fn founded_at(&self) -> u64 {
        match self {
            District::Quay => 500,
            District::Granary => 3_500,
            District::Hearth => 10_000,
            District::Shipyard => 22_000,
            District::Beacon => 42_000,
            District::Charthouse => 66_000,
        }
    }

    /// Souls this district adds to each expedition once founded. The
    /// colony you build is the ship that carries the next crossing — so
    /// each district makes the next expedition larger.
    pub fn expedition_bonus(&self) -> u32 {
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
            District::Quay => "room at last for a proper expedition",
            District::Granary => "stores to victual a fuller ship",
            District::Hearth => "warmth enough to carry more, and rest heals fully",
            District::Shipyard => {
                "the great slips; the largest expeditions, and the hull mends at the Tree"
            }
            District::Beacon => "a light the far shore steers by; each crossing builds more drive",
            District::Charthouse => "the whole colony behind each sailing",
        }
    }
}

/// The trophy shelf — records, not the engine.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossingRecords {
    pub fastest_days: u64,
    pub most_carried: u32,
    #[serde(alias = "total_leagues")]
    pub total_lightyears: u64,
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
    #[serde(alias = "resonance")]
    pub drive: u64,
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
            drive: 0,
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

    /// Souls a ferry run carries: the launch base, plus a share that grows
    /// with the colony's population, plus every founded district's bonus.
    /// The population term is the main engine — it climbs on every crossing,
    /// so the expedition never stops growing across a long era; the districts
    /// are milestone jumps on top of it.
    pub fn expedition_size(&self) -> u32 {
        let from_population = (self.souls_delivered * EXPEDITION_PER_1000_DELIVERED / 1_000) as u32;
        let from_districts = self
            .districts()
            .iter()
            .map(|d| d.expedition_bonus())
            .sum::<u32>();
        EXPEDITION_AT_LAUNCH + from_population + from_districts
    }

    /// How many souls the next ferry run embarks: capacity, or whatever
    /// the world has left, whichever is smaller.
    pub fn next_expedition(&self) -> u32 {
        (self.expedition_size() as u64).min(self.souls_remaining) as u32
    }

    /// The Drive time multiplier (≤ 1.0): the Vessel sails faster the
    /// more of the old world she has carried. Soft-capped at the floor.
    pub fn drive_time_mult(&self) -> f64 {
        let r = self.drive as f64;
        let speedup = (1.0 - FASTEST_CROSSING_TIME_MULT) * (r / (r + DRIVE_FOR_HALF_SPEEDUP));
        1.0 - speedup
    }

    /// A human "×N.N her old self" for the Reckoning.
    pub fn drive_speed_factor(&self) -> f64 {
        1.0 / self.drive_time_mult()
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
    /// grow drive and population, spend the dark's toll, keep records.
    /// Returns the districts newly founded (for the letters that greet them).
    pub fn deliver_crossing(
        &mut self,
        passengers: u32,
        days: u64,
        lightyears: u64,
        nights: u64,
    ) -> Vec<District> {
        let before = self.districts();

        let carried = (passengers as u64).min(self.souls_remaining);
        self.souls_delivered += carried;
        self.souls_remaining -= carried;
        self.drive += carried;

        // The dark took its share of whoever was still waiting.
        let toll = self.dark_toll().min(self.souls_remaining);
        self.souls_remaining -= toll;

        self.crossings_completed += 1;
        self.records.total_lightyears += lightyears;
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
        c.souls_delivered = 600; // past the Quay (500), short of the Granary
        assert_eq!(c.districts(), vec![District::Quay]);
        c.souls_delivered = 66_000; // the Charthouse threshold — all founded
        assert_eq!(c.districts().len(), 6, "the whole colony is founded");
    }

    #[test]
    fn resonance_speeds_her_up_toward_the_floor() {
        let mut c = ColonyState::found("t".into());
        assert!((c.drive_time_mult() - 1.0).abs() < 1e-9, "launch: no bonus");
        c.drive = DRIVE_FOR_HALF_SPEEDUP as u64;
        assert!((c.drive_time_mult() - 0.75).abs() < 0.01, "half point");
        c.drive = 1_000_000;
        assert!(c.drive_time_mult() > FASTEST_CROSSING_TIME_MULT - 0.01);
        assert!(c.drive_time_mult() < FASTEST_CROSSING_TIME_MULT + 0.02);
    }

    #[test]
    fn the_expedition_grows_with_population_and_districts() {
        let mut c = ColonyState::found("t".into());
        assert_eq!(
            c.expedition_size(),
            EXPEDITION_AT_LAUNCH,
            "launch: base only"
        );
        // Population alone grows the ferry: 1,000 delivered adds 75 berths,
        // and 1,000 is past the Quay (500) so its bonus (110) stacks too.
        c.souls_delivered = 1_000;
        assert_eq!(c.expedition_size(), 160 + 75 + 110);
        // The whole colony founded, deep into the era: the population term
        // dominates, every district's bonus on top.
        c.souls_delivered = 80_000;
        let bonuses = 110 + 140 + 170 + 210 + 260 + 320;
        assert_eq!(c.expedition_size(), 160 + (80_000 * 75 / 1_000) + bonuses);
    }

    #[test]
    fn delivering_grows_the_colony_and_the_dark_takes_its_share() {
        let mut c = ColonyState::found("t".into());
        let new = c.deliver_crossing(600, 35, 200, 12);
        assert_eq!(c.souls_delivered, 600);
        assert_eq!(c.crossings_completed, 1);
        assert_eq!(c.records.most_carried, 600);
        assert_eq!(c.records.fastest_days, 35);
        assert!(
            new.contains(&District::Quay),
            "600 delivered founds the Quay"
        );
        // The pool fell by more than the 600 carried — the dark took some.
        assert!(c.souls_remaining < INITIAL_SOULS - 600);
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
