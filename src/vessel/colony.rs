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
//! See `openspec/specs/vessel-act2/spec.md`.

use super::route::{WaypointId, ROUTE_SINK, ROUTE_START, WAYPOINTS};
use chrono::{DateTime, Utc};
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

/// The share of the still-waiting world the dark takes **each day** the
/// crossing is underway. The dark is always eating; a long crossing lets it
/// eat longer. This makes every day cost souls, so *speed saves lives* —
/// Drive (fewer days per crossing) and the Ward (a gentler dark) both cut the
/// toll, and the slow maiden voyage is the costliest crossing there is.
pub const DARK_TAKES_PER_DAY: f64 = 0.0006;

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

/// Real hours to fully charge Riftglass at Drive level 0 (the maiden
/// voyage's first Dock). Higher Drive levels charge proportionally faster
/// (`ColonyState::riftglass_rate_mult`). Validated against the era's
/// pacing envelope by the asserted sweeps in `tests/ferryman_tests.rs`
/// (`strategy_sweep_holds_the_campaign_envelope`,
/// `dock_time_across_charge_policies`).
pub const RIFTGLASS_BASE_HOURS_TO_FULL: f64 = 24.0;

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

/// The old world's decline, watched from the other side of the race:
/// districts mark the *colony's* growth (population, a rising number);
/// these mark the *dying world's* end (how much of it is gone — delivered
/// or lost — a falling one). Like districts, this is a pure function of
/// state, not stored progress: nothing needs to remember which already
/// fired, `world_milestones()` just recomputes from `souls_remaining` every
/// time. The one new mid-era noun of an otherwise flat ferry era — every
/// milestone here can land on any crossing depending on Drive/Ward/Shipwright
/// spend, so it isn't just "district thresholds again" on a different axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldMilestone {
    Tenth,
    Quarter,
    Half,
    ThreeQuarters,
    NinetyPercent,
}

impl WorldMilestone {
    pub const ALL: [WorldMilestone; 5] = [
        WorldMilestone::Tenth,
        WorldMilestone::Quarter,
        WorldMilestone::Half,
        WorldMilestone::ThreeQuarters,
        WorldMilestone::NinetyPercent,
    ];

    /// The fraction of `INITIAL_SOULS` that must be gone (delivered or lost
    /// to the dark) for this milestone to have passed.
    fn threshold_fraction(self) -> f64 {
        match self {
            WorldMilestone::Tenth => 0.10,
            WorldMilestone::Quarter => 0.25,
            WorldMilestone::Half => 0.50,
            WorldMilestone::ThreeQuarters => 0.75,
            WorldMilestone::NinetyPercent => 0.90,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            WorldMilestone::Tenth => "The first tenth",
            WorldMilestone::Quarter => "A quarter gone",
            WorldMilestone::Half => "Halfway",
            WorldMilestone::ThreeQuarters => "Three parts gone",
            WorldMilestone::NinetyPercent => "Nearly done",
        }
    }

    pub fn body(self) -> &'static str {
        match self {
            WorldMilestone::Tenth => {
                "One soul in ten who waited when you launched is now accounted \
                 for \u{2014} carried home, or lost. The rest still wait, not \
                 knowing which they will be."
            }
            WorldMilestone::Quarter => {
                "A quarter of the old world is gone from the ledger. The \
                 Vessel's wake is the only mark the dark leaves behind."
            }
            WorldMilestone::Half => {
                "Half the world you left behind is gone \u{2014} delivered or \
                 dark, there is no third answer. The ferryman does not slow \
                 to grieve either kind."
            }
            WorldMilestone::ThreeQuarters => {
                "Three in four are gone now. What is left waits closer \
                 together, in ports that have not dimmed yet \u{2014} for now."
            }
            WorldMilestone::NinetyPercent => {
                "Nine in ten, gone. Whatever remains of the old world could \
                 almost fit in a single crossing \u{2014} almost."
            }
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

/// What's newly true after folding a landfall into the colony — the
/// districts and world milestones that crossed their threshold this
/// crossing, for the caller to turn into letters/log moments.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CrossingDelivery {
    pub new_districts: Vec<District>,
    pub new_world_milestones: Vec<WorldMilestone>,
}

/// The Colony's real-time management phase between crossings: Riftglass
/// charges purely from elapsed real time since `docked_at`, at a rate the
/// Drive yard scales (`ColonyState::riftglass_rate_mult`). A pure function
/// of this one anchor timestamp — no incremental accrual to tick, no drift
/// on long absences, mirroring `weather.rs`'s pure-function style.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DockState {
    pub docked_at: DateTime<Utc>,
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
    /// The Ward yard's level: each one multiplies the dark's per-day toll
    /// by `WARD_DECAY`, down to `WARD_TOLL_FLOOR`. Bought with Salvage.
    #[serde(default)]
    pub ward_level: u32,
    /// Sea-days the last crossing took — lets the Reckoning project the dark's
    /// per-day rate into a concrete per-crossing figure.
    #[serde(default)]
    pub days_last_crossing: u64,
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
    /// The active Dock phase, if the Colony is between crossings waiting on
    /// a wormhole jump. `None` once a jump has been committed and the next
    /// crossing has begun.
    #[serde(default)]
    pub dock: Option<DockState>,
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
            days_last_crossing: 0,
            salvage: STARTING_SALVAGE,
            crossings_completed: 0,
            records: CrossingRecords::default(),
            dimmed_ports: Vec::new(),
            era_seed,
            dock: None,
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

    /// Fraction of `INITIAL_SOULS` gone from the old world — delivered or
    /// taken by the dark. 0 at launch, 1 when the era ends. Shared by
    /// `dark_ports()`'s chart-dimming pace and `world_milestones()`.
    fn world_emptied_fraction(&self) -> f64 {
        let gone = INITIAL_SOULS.saturating_sub(self.souls_remaining);
        (gone as f64 / INITIAL_SOULS as f64).clamp(0.0, 1.0)
    }

    /// World milestones passed so far, in order.
    pub fn world_milestones(&self) -> Vec<WorldMilestone> {
        let frac = self.world_emptied_fraction();
        WorldMilestone::ALL
            .into_iter()
            .filter(|m| frac >= m.threshold_fraction())
            .collect()
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

    /// How much faster than the base rate Riftglass charges — reuses the
    /// Drive yard's own speed factor verbatim (design.md Decision 3): the
    /// yard that makes the ship faster also punches the rift faster, no new
    /// content needed.
    pub fn riftglass_rate_mult(&self) -> f64 {
        self.drive_speed_factor()
    }

    /// Real hours to reach full Riftglass charge at the current Drive level.
    pub fn riftglass_hours_to_full(&self) -> f64 {
        RIFTGLASS_BASE_HOURS_TO_FULL / self.riftglass_rate_mult()
    }

    /// Riftglass charge, 0.0 (just docked) to 1.0 (fully charged, capped —
    /// no overcharge decay). A pure function of elapsed real time since
    /// `dock.docked_at`, identical whether queried once after a long
    /// absence or repeatedly across many short intervals. `0.0` if not
    /// currently docked.
    pub fn riftglass_charge(&self, now: DateTime<Utc>) -> f64 {
        let Some(dock) = &self.dock else {
            return 0.0;
        };
        let elapsed_hours = (now - dock.docked_at).num_seconds() as f64 / 3600.0;
        (elapsed_hours / self.riftglass_hours_to_full()).clamp(0.0, 1.0)
    }

    /// Enter the Dock phase — called the moment a crossing's arrival
    /// delivers to the Colony.
    pub fn dock(&mut self, now: DateTime<Utc>) {
        self.dock = Some(DockState { docked_at: now });
    }

    /// Leave the Dock phase — called when a wormhole jump is committed.
    pub fn undock(&mut self) {
        self.dock = None;
    }

    /// Salvage the Drive yard charges to reach the next level.
    pub fn drive_cost(&self) -> u64 {
        (DRIVE_COST_BASE * DRIVE_COST_GROWTH.powi(self.drive_level as i32)).round() as u64
    }

    /// Salvage the Shipwright charges to reach the next hold level.
    pub fn cap_cost(&self) -> u64 {
        (CAP_COST_BASE * CAP_COST_GROWTH.powi(self.cap_level as i32)).round() as u64
    }

    /// True once the next Drive level would buy no more speed — she's
    /// already at `DRIVE_FLOOR`. The yard should stop offering it here.
    pub fn drive_maxed(&self) -> bool {
        let next = DRIVE_DECAY
            .powi((self.drive_level + 1) as i32)
            .max(DRIVE_FLOOR);
        (self.drive_time_mult() - next).abs() < 1e-9
    }

    /// Spend Salvage to raise the Drive one level. Returns false (no change)
    /// if there isn't enough in hand, or if she's already at her limit — a
    /// level that buys zero speed is never worth escalating Salvage for.
    pub fn buy_drive(&mut self) -> bool {
        let cost = self.drive_cost();
        if self.salvage < cost || self.drive_maxed() {
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

    /// The dark's effective **per-day** rate after the Ward, as a fraction of
    /// the still-waiting world — the number the Reckoning shows as a percentage.
    pub fn dark_daily_rate(&self) -> f64 {
        DARK_TAKES_PER_DAY * self.ward_toll_mult()
    }

    /// Salvage the Ward charges to reach the next level.
    pub fn ward_cost(&self) -> u64 {
        (WARD_COST_BASE * WARD_COST_GROWTH.powi(self.ward_level as i32)).round() as u64
    }

    /// True once the next Ward level would blunt the dark no further — the
    /// toll is already down at `WARD_TOLL_FLOOR`.
    pub fn ward_maxed(&self) -> bool {
        let next = WARD_DECAY
            .powi((self.ward_level + 1) as i32)
            .max(WARD_TOLL_FLOOR);
        (self.ward_toll_mult() - next).abs() < 1e-9
    }

    /// Spend Salvage to raise the Ward one level. Returns false if short, or
    /// if she's already at the floor — same reasoning as `buy_drive`.
    pub fn buy_ward(&mut self) -> bool {
        let cost = self.ward_cost();
        if self.salvage < cost || self.ward_maxed() {
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

    /// Souls the dark takes over a crossing of `days` sea-days: it eats a
    /// `dark_daily_rate()` share of the still-waiting world every day, so a
    /// long crossing compounds into a larger bite. A big share while the world
    /// is full; a small one once it has emptied — late crossings are yours.
    pub fn dark_toll_for_days(&self, days: u64) -> u64 {
        let frac = 1.0 - (1.0 - self.dark_daily_rate()).powf(days as f64);
        (self.souls_remaining as f64 * frac).round() as u64
    }

    /// The dark's bite over a crossing like the last one — for the Reckoning's
    /// concrete "≈ N a crossing" projection (defaults to a nominal crossing
    /// before the first landfall records a real day count).
    pub fn dark_toll_projected(&self) -> u64 {
        let days = if self.days_last_crossing == 0 {
            10
        } else {
            self.days_last_crossing
        };
        self.dark_toll_for_days(days)
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
        let frac = self.world_emptied_fraction();
        let count = (frac * dimmable.len() as f64).round() as usize;
        dimmable.sort_by_key(|id| (self.port_dim_order(*id), id.0));
        dimmable.truncate(count);
        dimmable
    }

    /// Fold a completed crossing into the colony: deliver its passengers,
    /// grow drive and population, spend the dark's toll, keep records.
    /// Returns what's newly true this landfall (for the letters/log moments
    /// that greet it).
    pub fn deliver_crossing(
        &mut self,
        passengers: u32,
        days: u64,
        lightyears: u64,
        nights: u64,
    ) -> CrossingDelivery {
        let districts_before = self.districts();
        let milestones_before = self.world_milestones();

        let carried = (passengers as u64).min(self.souls_remaining);
        self.souls_delivered += carried;
        self.souls_remaining -= carried;
        // The crossing pays out in Salvage for the yards.
        self.salvage += Self::salvage_income(carried);

        // The dark took its share of whoever was still waiting — a day at a
        // time, so a long crossing costs more than a short one.
        self.days_last_crossing = days;
        let toll = self.dark_toll_for_days(days).min(self.souls_remaining);
        self.souls_remaining -= toll;

        self.crossings_completed += 1;
        self.records.total_lightyears += lightyears;
        self.records.total_nights += nights;
        self.records.most_carried = self.records.most_carried.max(passengers);
        if days > 0 && (self.records.fastest_days == 0 || days < self.records.fastest_days) {
            self.records.fastest_days = days;
        }
        self.dimmed_ports = self.dark_ports();

        CrossingDelivery {
            new_districts: District::ALL
                .into_iter()
                .filter(|d| !districts_before.contains(d) && self.districts().contains(d))
                .collect(),
            new_world_milestones: WorldMilestone::ALL
                .into_iter()
                .filter(|m| !milestones_before.contains(m) && self.world_milestones().contains(m))
                .collect(),
        }
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
    fn the_drive_yard_refuses_a_purchase_at_the_floor() {
        let mut c = ColonyState::found("t".into());
        c.drive_level = 100; // far past DRIVE_FLOOR
        c.salvage = 1_000_000_000;
        assert!(c.drive_maxed(), "100 levels is well past the floor");
        assert!(
            !c.buy_drive(),
            "a level that buys zero speed should never escalate Salvage"
        );
        assert_eq!(c.drive_level, 100, "the refused buy changes nothing");
        assert_eq!(c.salvage, 1_000_000_000, "and spends nothing");
    }

    #[test]
    fn the_ward_yard_refuses_a_purchase_at_the_floor() {
        let mut c = ColonyState::found("t".into());
        c.ward_level = 100; // far past WARD_TOLL_FLOOR
        c.salvage = 1_000_000_000;
        assert!(c.ward_maxed(), "100 levels is well past the floor");
        assert!(
            !c.buy_ward(),
            "a level that blunts the toll no further should never escalate Salvage"
        );
        assert_eq!(c.ward_level, 100, "the refused buy changes nothing");
        assert_eq!(c.salvage, 1_000_000_000, "and spends nothing");
    }

    #[test]
    fn delivering_grows_the_colony_pays_salvage_and_the_dark_takes_its_share() {
        let mut c = ColonyState::found("t".into());
        let salvage_before = c.salvage;
        let delivery = c.deliver_crossing(600, 35, 200, 12);
        assert_eq!(c.souls_delivered, 600);
        assert_eq!(c.crossings_completed, 1);
        assert_eq!(c.records.most_carried, 600);
        assert_eq!(c.records.fastest_days, 35);
        // The crossing paid out Salvage: base plus a share of the 600 carried.
        assert_eq!(c.salvage, salvage_before + ColonyState::salvage_income(600));
        assert!(
            delivery.new_districts.contains(&District::Quay),
            "600 delivered founds the Quay"
        );
        // The pool fell by more than the 600 carried — the dark took some.
        assert!(c.souls_remaining < INITIAL_SOULS - 600);
    }

    #[test]
    fn the_ward_buys_the_dark_toll_down_toward_a_floor() {
        let mut c = ColonyState::found("t".into());
        c.souls_remaining = 10_000;
        let base_toll = c.dark_toll_for_days(10);
        assert!(base_toll > 0, "the dark bites at Ward 0");

        // Each level blunts the toll, compounding.
        c.ward_level = 1;
        assert!(
            (c.ward_toll_mult() - WARD_DECAY).abs() < 1e-9,
            "one level = one decay step"
        );
        assert!(
            c.dark_toll_for_days(10) < base_toll,
            "a warded toll is smaller"
        );

        // A longer crossing costs more than a short one.
        assert!(
            c.dark_toll_for_days(20) > c.dark_toll_for_days(5),
            "more days at sea, more souls to the dark"
        );

        // It floors — the dark never fully stops.
        c.ward_level = 100;
        assert!(
            (c.ward_toll_mult() - WARD_TOLL_FLOOR).abs() < 1e-9,
            "the Ward can only blunt, never negate"
        );
        assert!(
            c.dark_toll_for_days(10) > 0,
            "a residual bite always remains"
        );
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
    fn world_milestones_pass_in_order_as_the_world_empties() {
        let mut c = ColonyState::found("t".into());
        assert!(
            c.world_milestones().is_empty(),
            "launch: none of the old world is gone yet"
        );

        c.souls_remaining = INITIAL_SOULS - INITIAL_SOULS / 10; // 10% gone
        assert_eq!(
            c.world_milestones(),
            vec![WorldMilestone::Tenth],
            "exactly the first milestone passes at 10% gone"
        );

        c.souls_remaining = INITIAL_SOULS / 2; // 50% gone
        assert_eq!(
            c.world_milestones(),
            vec![
                WorldMilestone::Tenth,
                WorldMilestone::Quarter,
                WorldMilestone::Half,
            ],
            "milestones accumulate, they never un-pass"
        );

        c.souls_remaining = 0; // the whole world gone
        assert_eq!(
            c.world_milestones().len(),
            WorldMilestone::ALL.len(),
            "every milestone has passed once the era is over"
        );
    }

    #[test]
    fn delivering_a_crossing_reports_newly_passed_world_milestones() {
        let mut c = ColonyState::found("t".into());
        // Just short of 10% gone — a couple more souls accounted for tips it over.
        c.souls_remaining = INITIAL_SOULS - INITIAL_SOULS / 10 + 5;
        let delivery = c.deliver_crossing(10, 0, 100, 0); // 0 days: no dark toll, just delivery
        assert_eq!(
            delivery.new_world_milestones,
            vec![WorldMilestone::Tenth],
            "crossing the threshold this landfall reports it once"
        );
        // The next crossing, still short of the next threshold, reports none.
        let delivery = c.deliver_crossing(1, 0, 100, 0);
        assert!(
            delivery.new_world_milestones.is_empty(),
            "no new milestone until the next threshold is actually crossed"
        );
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
    fn riftglass_charge_is_zero_when_not_docked() {
        let c = ColonyState::found("t".into());
        assert_eq!(c.riftglass_charge(chrono::Utc::now()), 0.0);
    }

    #[test]
    fn riftglass_charges_linearly_from_the_dock_anchor() {
        let mut c = ColonyState::found("t".into());
        let t0 = chrono::DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        c.dock(t0);
        assert_eq!(c.riftglass_charge(t0), 0.0, "no time has passed yet");

        let half = t0 + chrono::Duration::hours((RIFTGLASS_BASE_HOURS_TO_FULL / 2.0) as i64);
        assert!(
            (c.riftglass_charge(half) - 0.5).abs() < 1e-6,
            "half the base hours ⇒ half charge at Drive 0"
        );

        let full = t0 + chrono::Duration::hours(RIFTGLASS_BASE_HOURS_TO_FULL as i64);
        assert!(
            (c.riftglass_charge(full) - 1.0).abs() < 1e-6,
            "the base hours ⇒ full charge at Drive 0"
        );

        let overshot = t0 + chrono::Duration::hours((RIFTGLASS_BASE_HOURS_TO_FULL * 3.0) as i64);
        assert_eq!(
            c.riftglass_charge(overshot),
            1.0,
            "charge caps at 1.0, never decays past full"
        );
    }

    #[test]
    fn riftglass_charge_is_chunking_invariant() {
        // Queried once after a long gap, or repeatedly across short gaps,
        // the charge at any given `now` must be identical — it's a pure
        // function of the anchor and `now`, nothing is ticked/mutated.
        let mut c = ColonyState::found("t".into());
        let t0 = chrono::DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        c.dock(t0);
        let later = t0 + chrono::Duration::hours(10);
        let long_gap = c.riftglass_charge(later);
        for hours in 0..10 {
            let _ = c.riftglass_charge(t0 + chrono::Duration::hours(hours));
        }
        let after_many_short_queries = c.riftglass_charge(later);
        assert_eq!(long_gap, after_many_short_queries);
    }

    #[test]
    fn drive_level_speeds_riftglass_accrual() {
        let mut slow = ColonyState::found("t".into());
        let mut fast = ColonyState::found("t".into());
        fast.drive_level = 3;
        let t0 = chrono::DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        slow.dock(t0);
        fast.dock(t0);
        let later = t0 + chrono::Duration::hours(6);
        assert!(
            fast.riftglass_charge(later) > slow.riftglass_charge(later),
            "a higher Drive level charges Riftglass faster for the same elapsed time"
        );
    }

    #[test]
    fn dock_and_undock_toggle_the_dock_phase() {
        let mut c = ColonyState::found("t".into());
        assert!(c.dock.is_none());
        c.dock(chrono::Utc::now());
        assert!(c.dock.is_some());
        c.undock();
        assert!(c.dock.is_none());
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
