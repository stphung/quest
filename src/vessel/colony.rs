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

/// The ferry's hold at Shipwright level 0 — how many souls a fresh ferry
/// carries before you spend any Salvage widening her. The Shipwright yard
/// grows this; the districts add standing bonuses on top.
pub const BASE_CAPACITY: u32 = 180;

/// Each Shipwright level multiplies the hold by this — a compounding widen,
/// so late loads climb into the tens of thousands. Paid for with Salvage.
pub const CAP_GROWTH: f64 = 1.36;

/// Each Drive level multiplies the crossing's sail-time by this (0.70 =
/// 30% faster per level), compounding down toward the floor. The ramp is
/// *earned*: the maiden voyage launches at Drive 0 (full time), and every
/// level you buy shortens every crossing that follows.
pub const DRIVE_DECAY: f64 = 0.70;
/// The quickest a crossing can ever get, as a fraction of its launch time
/// (0.05 = twenty times as fast). Drive levels approach but never pass it.
pub const DRIVE_FLOOR: f64 = 0.05;

/// Salvage — the currency of the yards — earned on every landfall: a flat
/// base plus a share of the souls carried, so a fuller hold funds faster
/// upgrades. This is the loop's economy: carry more → build more.
pub const SALVAGE_AT_LANDFALL: u64 = 3;
/// One Salvage for every this-many souls delivered in a crossing.
pub const SOULS_PER_SALVAGE: u64 = 30;
/// The founding grant — enough Salvage after the maiden voyage that the early
/// yard choices bite at once and the ramp takes hold from the second crossing.
pub const STARTING_SALVAGE: u64 = 40;

/// The Drive yard's price ladder: level `L` costs `4 × 1.5^L` Salvage,
/// rounded — cheap early, steep late, so Salvage is always scarce and the
/// Drive-vs-hold choice always bites.
pub const DRIVE_COST_BASE: f64 = 4.0;
pub const DRIVE_COST_GROWTH: f64 = 1.5;
/// The Shipwright's price ladder: level `L` costs `5 × 1.42^L` Salvage.
pub const CAP_COST_BASE: f64 = 5.0;
pub const CAP_COST_GROWTH: f64 = 1.42;

/// The share of the still-waiting world the dark takes each crossing —
/// a visible per-crossing toll, not a slow drip. Small, but it makes pure
/// speed a trap: a Drive-only build runs many short crossings and the dark
/// bites on every one, saving fewer souls than a balanced hand.
pub const DARK_TAKES_EACH_CROSSING: f64 = 0.011;

/// The Ward yard — the third Salvage track (speed / capacity / **attrition**).
/// Each level multiplies the dark's per-crossing toll by `WARD_DECAY`,
/// compounding down toward `WARD_TOLL_FLOOR` (never to zero — the dark always
/// keeps a little). It buys down the very toll that makes crossing-count
/// matter, so it is the souls-first hand's answer to a long era. A punchy
/// per-level cut (each level takes ~28% off the toll) so the choice reads.
pub const WARD_DECAY: f64 = 0.72;
/// The most the Ward can ever blunt the toll, as a fraction of its base rate
/// (0.12 = at most an 88% cut). A residual bite always remains.
pub const WARD_TOLL_FLOOR: f64 = 0.12;
/// The Ward's price ladder: level `L` costs `5 × 1.45^L` Salvage — priced
/// between Drive and the Shipwright, an accessible trade against carrying
/// more or sailing faster.
pub const WARD_COST_BASE: f64 = 5.0;
pub const WARD_COST_GROWTH: f64 = 1.45;

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
    /// The Drive yard's level: each one multiplies every future crossing's
    /// sail-time by `DRIVE_DECAY`, down to `DRIVE_FLOOR`. Bought with Salvage.
    #[serde(default)]
    pub drive_level: u32,
    /// The Shipwright's level: each one multiplies the hold by `CAP_GROWTH`.
    /// Bought with Salvage.
    #[serde(default)]
    pub cap_level: u32,
    /// The Ward yard's level: each one multiplies the dark's per-crossing toll
    /// by `WARD_DECAY`, down to `WARD_TOLL_FLOOR`. Bought with Salvage.
    #[serde(default)]
    pub ward_level: u32,
    /// Salvage in hand — the yards' currency, earned on every landfall and
    /// spent on Drive or hold.
    #[serde(default)]
    pub salvage: u64,
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
            drive_level: 0,
            cap_level: 0,
            ward_level: 0,
            salvage: STARTING_SALVAGE,
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

    /// Souls a ferry run carries: the base hold widened by every Shipwright
    /// level you have bought, plus every founded district's standing bonus.
    /// The hold is the Shipwright track — it only grows when you spend Salvage
    /// on it; the districts are passive milestone jumps on top.
    pub fn expedition_size(&self) -> u32 {
        let widened = (f64::from(BASE_CAPACITY) * CAP_GROWTH.powi(self.cap_level as i32)).round();
        let from_districts = self
            .districts()
            .iter()
            .map(|d| d.expedition_bonus())
            .sum::<u32>();
        widened as u32 + from_districts
    }

    /// How many souls the next ferry run embarks: capacity, or whatever
    /// the world has left, whichever is smaller.
    pub fn next_expedition(&self) -> u32 {
        (self.expedition_size() as u64).min(self.souls_remaining) as u32
    }

    /// The Drive time multiplier (≤ 1.0): every Drive level shortens the
    /// crossing by `DRIVE_DECAY`, compounding down to `DRIVE_FLOOR`. At level
    /// 0 (the maiden voyage) it is exactly 1.0 — the slowest crossing there is.
    pub fn drive_time_mult(&self) -> f64 {
        DRIVE_DECAY.powi(self.drive_level as i32).max(DRIVE_FLOOR)
    }

    /// A human "×N.N her old self" for the Reckoning.
    pub fn drive_speed_factor(&self) -> f64 {
        1.0 / self.drive_time_mult()
    }

    /// Salvage the Drive yard charges to reach the next level.
    pub fn drive_cost(&self) -> u64 {
        (DRIVE_COST_BASE * DRIVE_COST_GROWTH.powi(self.drive_level as i32)).round() as u64
    }

    /// Salvage the Shipwright charges to reach the next hold level.
    pub fn cap_cost(&self) -> u64 {
        (CAP_COST_BASE * CAP_COST_GROWTH.powi(self.cap_level as i32)).round() as u64
    }

    /// Spend Salvage to raise the Drive one level. Returns false (no change)
    /// if there isn't enough in hand.
    pub fn buy_drive(&mut self) -> bool {
        let cost = self.drive_cost();
        if self.salvage < cost {
            return false;
        }
        self.salvage -= cost;
        self.drive_level += 1;
        true
    }

    /// Spend Salvage to widen the hold one level. Returns false if short.
    pub fn buy_capacity(&mut self) -> bool {
        let cost = self.cap_cost();
        if self.salvage < cost {
            return false;
        }
        self.salvage -= cost;
        self.cap_level += 1;
        true
    }

    /// The Ward's toll multiplier (≤ 1.0): every level blunts the dark by
    /// `WARD_DECAY`, compounding down to `WARD_TOLL_FLOOR`. Level 0 is 1.0 —
    /// the full toll, until you spend Salvage warding it.
    pub fn ward_toll_mult(&self) -> f64 {
        WARD_DECAY.powi(self.ward_level as i32).max(WARD_TOLL_FLOOR)
    }

    /// The dark's effective per-crossing rate after the Ward, as a fraction —
    /// the number the Reckoning shows as a percentage.
    pub fn dark_toll_rate(&self) -> f64 {
        DARK_TAKES_EACH_CROSSING * self.ward_toll_mult()
    }

    /// Salvage the Ward charges to reach the next level.
    pub fn ward_cost(&self) -> u64 {
        (WARD_COST_BASE * WARD_COST_GROWTH.powi(self.ward_level as i32)).round() as u64
    }

    /// Spend Salvage to raise the Ward one level. Returns false if short.
    pub fn buy_ward(&mut self) -> bool {
        let cost = self.ward_cost();
        if self.salvage < cost {
            return false;
        }
        self.salvage -= cost;
        self.ward_level += 1;
        true
    }

    /// Salvage a crossing yields at landfall: a flat base plus a share of the
    /// souls it carried — a fuller hold funds faster upgrades.
    pub fn salvage_income(carried: u64) -> u64 {
        SALVAGE_AT_LANDFALL + carried / SOULS_PER_SALVAGE
    }

    /// Souls the dark takes this crossing: a fixed share of whoever is
    /// still waiting. A big, visible bite while the world is full; a small
    /// one once it has emptied — so late crossings are yours to finish.
    pub fn dark_toll(&self) -> u64 {
        (self.souls_remaining as f64 * self.dark_toll_rate()).round() as u64
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
        // The crossing pays out in Salvage for the yards.
        self.salvage += Self::salvage_income(carried);

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
    fn drive_levels_speed_her_up_toward_the_floor() {
        let mut c = ColonyState::found("t".into());
        assert!(
            (c.drive_time_mult() - 1.0).abs() < 1e-9,
            "launch (level 0): the slowest crossing there is"
        );
        // Each level compounds the decay.
        c.drive_level = 1;
        assert!(
            (c.drive_time_mult() - DRIVE_DECAY).abs() < 1e-9,
            "one level"
        );
        c.drive_level = 3;
        assert!(
            (c.drive_time_mult() - DRIVE_DECAY.powi(3)).abs() < 1e-9,
            "three levels compound"
        );
        // Far past the floor, she never goes quicker than the floor.
        c.drive_level = 100;
        assert!((c.drive_time_mult() - DRIVE_FLOOR).abs() < 1e-9, "floor");
    }

    #[test]
    fn the_hold_grows_with_the_shipwright_and_districts() {
        let mut c = ColonyState::found("t".into());
        assert_eq!(
            c.expedition_size(),
            BASE_CAPACITY,
            "level 0, no districts: the base hold only"
        );
        // A Shipwright level widens the hold by CAP_GROWTH — nothing else does.
        c.cap_level = 1;
        let widened = (f64::from(BASE_CAPACITY) * CAP_GROWTH).round() as u32;
        assert_eq!(c.expedition_size(), widened, "one Shipwright level");
        // Districts (founded by population) stack their standing bonuses on top.
        c.souls_delivered = 66_000; // all six districts founded
        let bonuses = 110 + 140 + 170 + 210 + 260 + 320;
        assert_eq!(c.expedition_size(), widened + bonuses);
    }

    #[test]
    fn the_yards_spend_salvage_on_drive_and_hold() {
        let mut c = ColonyState::found("t".into());
        let start = c.salvage;
        // The first Drive level is affordable out of the founding grant.
        let cost0 = c.drive_cost();
        assert!(
            c.buy_drive(),
            "the founding grant buys the first Drive level"
        );
        assert_eq!(c.drive_level, 1);
        assert_eq!(c.salvage, start - cost0);
        // Costs climb with the level.
        assert!(c.drive_cost() > cost0, "the ladder steepens");
        // Drained of Salvage, the yard refuses.
        c.salvage = 0;
        assert!(!c.buy_drive(), "no Salvage, no upgrade");
        assert!(!c.buy_capacity());
        assert_eq!(c.drive_level, 1, "a refused buy changes nothing");
    }

    #[test]
    fn delivering_grows_the_colony_pays_salvage_and_the_dark_takes_its_share() {
        let mut c = ColonyState::found("t".into());
        let salvage_before = c.salvage;
        let new = c.deliver_crossing(600, 35, 200, 12);
        assert_eq!(c.souls_delivered, 600);
        assert_eq!(c.crossings_completed, 1);
        assert_eq!(c.records.most_carried, 600);
        assert_eq!(c.records.fastest_days, 35);
        // The crossing paid out Salvage: base plus a share of the 600 carried.
        assert_eq!(c.salvage, salvage_before + ColonyState::salvage_income(600));
        assert!(
            new.contains(&District::Quay),
            "600 delivered founds the Quay"
        );
        // The pool fell by more than the 600 carried — the dark took some.
        assert!(c.souls_remaining < INITIAL_SOULS - 600);
    }

    #[test]
    fn the_ward_buys_the_dark_toll_down_toward_a_floor() {
        let mut c = ColonyState::found("t".into());
        c.souls_remaining = 10_000;
        let base_toll = c.dark_toll();
        assert!(base_toll > 0, "the dark bites at Ward 0");

        // Each level blunts the toll, compounding.
        c.ward_level = 1;
        assert!(
            (c.ward_toll_mult() - WARD_DECAY).abs() < 1e-9,
            "one level = one decay step"
        );
        assert!(c.dark_toll() < base_toll, "a warded toll is smaller");

        // It floors — the dark never fully stops.
        c.ward_level = 100;
        assert!(
            (c.ward_toll_mult() - WARD_TOLL_FLOOR).abs() < 1e-9,
            "the Ward can only blunt, never negate"
        );
        assert!(c.dark_toll() > 0, "a residual bite always remains");
    }

    #[test]
    fn the_ward_yard_spends_salvage_like_the_others() {
        let mut c = ColonyState::found("t".into());
        let start = c.salvage;
        let cost0 = c.ward_cost();
        assert!(c.buy_ward(), "the founding grant buys the first Ward level");
        assert_eq!(c.ward_level, 1);
        assert_eq!(c.salvage, start - cost0);
        assert!(c.ward_cost() > cost0, "the ladder steepens");
        c.salvage = 0;
        assert!(!c.buy_ward(), "no Salvage, no upgrade");
        assert_eq!(c.ward_level, 1, "a refused buy changes nothing");
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
