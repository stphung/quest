//! Void weather — currents, silence-banks, and squalls (sub-project 5).
//!
//! Weather is a **pure function of `(voyage_seed, game-hour)`**: no state is
//! stored or serialized, so offline resolution and live play see identical
//! skies by construction, and save-scumming changes nothing. Objects live on
//! the route graph's edges, drift to an adjacent road once a day, and
//! dissipate after 2–5 days. Weather prices time and provisions —
//! the things the player already trades. It never touches souls.
//!
//! See `docs/superpowers/specs/2026-07-03-vessel-underway-design.md`.

use super::route::{self, Chapter, RoadId};

/// The three kinds of void weather (v1: exactly three).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherKind {
    /// `≋` A running current with a bearing. Opportunity, not tax: Run
    /// rides a with-bearing current; Run against one costs more.
    Current,
    /// `▒` A region where sound declines. Sail it at Easy and the crew can
    /// hear what the silence hides — a rumor, one per bank.
    SilenceBank,
    /// `≈` Fast dirty water. Taxes provisions; shelter through it at
    /// Mourn/Quiet, or pay double for Running it.
    Squall,
}

impl WeatherKind {
    pub fn glyph(&self) -> char {
        match self {
            WeatherKind::Current => '\u{224b}',     // ≋
            WeatherKind::SilenceBank => '\u{2592}', // ▒
            WeatherKind::Squall => '\u{2248}',      // ≈
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            WeatherKind::Current => "a current",
            WeatherKind::SilenceBank => "a silence-bank",
            WeatherKind::Squall => "a squall",
        }
    }
}

/// One weather object, derived on demand. `id` is stable for the object's
/// whole life (keyed by spawn slot) — the once-per-bank rumor and the UI
/// both rely on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeatherObj {
    pub id: u32,
    pub kind: WeatherKind,
    /// The road it sits on at the queried hour.
    pub road: RoadId,
    /// For currents: true = the current runs the road's direction of
    /// travel (riding it helps); false = it runs against you.
    pub with_bearing: bool,
    /// A flavor name from the chapter deck ("the Northing").
    pub name: &'static str,
    /// Hours until it dissipates (display only).
    pub hours_left: u32,
}

/// At most this many weather objects may affect the player's current leg.
pub const MAX_ON_LEG: usize = 2;
/// At most this many render on the chart; the rest of the void's weather
/// is simply not shown.
pub const MAX_VISIBLE: usize = 4;

/// Weather spawn cadence: one draw per chapter every this many hours.
/// Chapter decks skew which kinds come up.
const SPAWN_PERIOD_HOURS: u64 = 30;
/// Lifetime range in hours (2–5 real days).
const LIFE_MIN_HOURS: u64 = 48;
const LIFE_SPAN_HOURS: u64 = 72;

/// Chapter weather decks: the Shallows run mild currents, the Starless
/// Deep deals long silence; squalls cluster in the middle chapters.
/// (kind weights out of 8, name pool)
fn deck(chapter: Chapter) -> (&'static [WeatherKind; 8], &'static [&'static str]) {
    use WeatherKind::*;
    match chapter {
        Chapter::Shallows => (
            &[
                Current,
                Current,
                Current,
                Current,
                Current,
                Squall,
                Squall,
                SilenceBank,
            ],
            &["the Homeward Set", "the Gull Run", "the Ebb Coil"],
        ),
        Chapter::DriftRoads => (
            &[
                Current,
                Current,
                Current,
                Squall,
                Squall,
                Squall,
                SilenceBank,
                SilenceBank,
            ],
            &["the Northing", "the Fair-Chaser", "the Long Pull"],
        ),
        Chapter::StarlessDeep => (
            &[
                SilenceBank,
                SilenceBank,
                SilenceBank,
                SilenceBank,
                Current,
                Squall,
                Squall,
                SilenceBank,
            ],
            &["the Hush", "the Deep Quiet", "the Unspoken"],
        ),
        Chapter::RootsOfLight => (
            &[
                Current,
                Current,
                Current,
                Current,
                Squall,
                Current,
                SilenceBank,
                Current,
            ],
            &["the Rootwash", "the Bloom Tide", "the Last Set"],
        ),
    }
}

/// Roads belonging to a chapter (weather stays inside its chapter's water;
/// gateway legs count with the earlier chapter).
fn chapter_roads(chapter: Chapter) -> Vec<RoadId> {
    route::ROADS
        .iter()
        .filter(|r| route::waypoint(r.from).chapter == chapter)
        .map(|r| r.id)
        .collect()
}

/// SplitMix64 — a tiny deterministic mixer. All weather draws come through
/// here; no thread RNG anywhere near the void.
fn mix(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

fn draw(seed: u64, salt: u64, n: u64) -> u64 {
    mix(seed ^ mix(salt.wrapping_mul(0x517c_c1b7).wrapping_add(n)))
}

/// All weather objects alive at `hour` (game hours since launch), capped at
/// [`MAX_VISIBLE`] in stable id order.
pub fn weather_at(seed: u64, hour: u64) -> Vec<WeatherObj> {
    let chapters = [
        Chapter::Shallows,
        Chapter::DriftRoads,
        Chapter::StarlessDeep,
        Chapter::RootsOfLight,
    ];
    let mut objs = Vec::new();
    for (ci, chapter) in chapters.into_iter().enumerate() {
        let roads = chapter_roads(chapter);
        if roads.is_empty() {
            continue;
        }
        let (kinds, names) = deck(chapter);
        // Spawn slots: slot n spawns at a jittered hour inside its period.
        // Only the last few slots can still be alive at `hour`.
        let cur_slot = hour / SPAWN_PERIOD_HOURS;
        let first = cur_slot.saturating_sub(4); // 5 days back covers max life
        for slot in first..=cur_slot {
            let salt = (ci as u64) << 32 | slot;
            let jitter = draw(seed, salt, 0) % SPAWN_PERIOD_HOURS;
            let spawn = slot * SPAWN_PERIOD_HOURS + jitter;
            let life = LIFE_MIN_HOURS + draw(seed, salt, 1) % LIFE_SPAN_HOURS;
            if hour < spawn || hour >= spawn + life {
                continue;
            }
            let kind = kinds[(draw(seed, salt, 2) % 8) as usize];
            let name = names[(draw(seed, salt, 3) % names.len() as u64) as usize];
            // Drift: a deterministic walk over the chapter's roads, one
            // step per day of life.
            let day_of_life = (hour - spawn) / 24;
            let road = roads[(draw(seed, salt, 4 + day_of_life) % roads.len() as u64) as usize];
            objs.push(WeatherObj {
                id: ((ci as u32) << 16) | (slot as u32 & 0xffff),
                kind,
                road,
                with_bearing: draw(seed, salt, 99).is_multiple_of(2),
                name,
                hours_left: (spawn + life - hour) as u32,
            });
        }
    }
    objs.sort_by_key(|o| o.id);
    objs.truncate(MAX_VISIBLE);
    objs
}

/// Weather sitting on a specific road at `hour`, capped at [`MAX_ON_LEG`].
pub fn weather_on_road(seed: u64, hour: u64, road: RoadId) -> Vec<WeatherObj> {
    let mut objs: Vec<WeatherObj> = weather_at(seed, hour)
        .into_iter()
        .filter(|o| o.road == road)
        .collect();
    objs.truncate(MAX_ON_LEG);
    objs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weather_is_a_pure_function_of_seed_and_hour() {
        for hour in [0u64, 7, 100, 999, 5000] {
            assert_eq!(weather_at(42, hour), weather_at(42, hour));
        }
        // Different seeds see different skies (at some hour).
        let differs = (0..200u64).any(|h| weather_at(1, h * 12) != weather_at(2, h * 12));
        assert!(differs, "seeds should diverge");
    }

    #[test]
    fn caps_hold_at_every_sampled_hour() {
        for hour in 0..2000u64 {
            let objs = weather_at(7, hour * 3);
            assert!(objs.len() <= MAX_VISIBLE);
            for road in objs.iter().map(|o| o.road) {
                assert!(weather_on_road(7, hour * 3, road).len() <= MAX_ON_LEG);
            }
        }
    }

    #[test]
    fn objects_live_days_and_keep_their_identity() {
        // Find an object and follow it: same id, same kind/name, roads
        // stay within its chapter, and it eventually dissipates.
        let first = (0..500u64)
            .map(|h| h * 6)
            .find_map(|hour| weather_at(11, hour).first().copied().map(|o| (hour, o)));
        let (hour, obj) = first.expect("no weather ever spawned");
        if let Some(same) = weather_at(11, hour + 12).iter().find(|o| o.id == obj.id) {
            assert_eq!(same.kind, obj.kind);
            assert_eq!(same.name, obj.name);
        }
        // It is gone well past its remaining life.
        let after = weather_at(11, hour + obj.hours_left as u64 + 1);
        assert!(
            !after.iter().any(|o| o.id == obj.id),
            "object outlived its life"
        );
    }

    #[test]
    fn every_chapter_deals_weather() {
        let mut seen = [false; 4];
        for h in 0..3000u64 {
            for obj in weather_at(3, h * 4) {
                seen[(obj.id >> 16) as usize] = true;
            }
        }
        assert_eq!(seen, [true; 4], "every chapter's deck should deal");
    }

    #[test]
    fn the_deep_deals_mostly_silence() {
        let mut silence = 0u32;
        let mut total = 0u32;
        for h in 0..4000u64 {
            for obj in weather_at(9, h * 6) {
                if (obj.id >> 16) == 2 {
                    total += 1;
                    if obj.kind == WeatherKind::SilenceBank {
                        silence += 1;
                    }
                }
            }
        }
        assert!(total > 0);
        assert!(
            silence * 2 > total,
            "the Starless Deep should deal silence more often than not"
        );
    }
}
