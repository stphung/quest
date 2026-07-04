#![allow(dead_code)] // Functions wired into the game loop incrementally
//! Mercenary generation, recruitment, and archetype system for The Deep.
//!
//! Covers:
//! - Archetype stat distributions (Power, Resilience, Expertise)
//! - Merc generation with guild-rank quality scaling and random variance
//! - Recruitment pool generation (3-5 daily candidates by guild rank)
//! - Starter roster (3 free mercs on discovery/prestige)
//! - Level-up stat scaling per archetype
//! - Recruit cost calculation by quality tier
//!
//! All functions take `rng: &mut impl Rng` — no local RNG is created internally.
//! Follows the Haven bonus injection pattern: no global state, all context passed as params.

use super::types::{DeepPrestige, GuildRank, MercArchetype, MercStatus, Mercenary, RecruitPool};
use chrono::Utc;
use rand::{Rng, RngExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Quality Tiers ─────────────────────────────────────────────────────────────

/// Internal quality tier for a generated mercenary.
/// Determines stat bonuses above the archetype base and recruit cost range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MercQuality {
    #[default]
    /// Basic recruit. Available at all guild ranks.
    Common,
    /// Above-average recruit. Available at Rank 2+.
    Uncommon,
    /// Elite recruit. Available at Rank 3+.
    Rare,
    /// Exceptional recruit. Available at Rank 5 only.
    Elite,
}

impl MercQuality {
    /// Flat stat bonus applied to all three stats (on top of archetype base).
    /// Primary stats receive an additional `primary_bonus()`.
    pub fn flat_bonus(self) -> u32 {
        match self {
            MercQuality::Common => 0,
            MercQuality::Uncommon => 2,
            MercQuality::Rare => 4,
            MercQuality::Elite => 8,
        }
    }

    /// Extra bonus applied to the archetype's primary stats only.
    pub fn primary_bonus(self) -> u32 {
        match self {
            MercQuality::Common => 0,
            MercQuality::Uncommon => 2,
            MercQuality::Rare => 2,
            MercQuality::Elite => 4,
        }
    }

    /// Warband Marks cost range (min, max) to recruit this quality.
    pub fn cost_range(self) -> (u32, u32) {
        match self {
            MercQuality::Common => (50, 80),
            MercQuality::Uncommon => (80, 130),
            MercQuality::Rare => (130, 200),
            MercQuality::Elite => (200, 300),
        }
    }

    /// Returns the next quality tier, or `None` if already Elite.
    pub fn next(self) -> Option<MercQuality> {
        match self {
            MercQuality::Common => Some(MercQuality::Uncommon),
            MercQuality::Uncommon => Some(MercQuality::Rare),
            MercQuality::Rare => Some(MercQuality::Elite),
            MercQuality::Elite => None,
        }
    }
}

// ── Promotion ────────────────────────────────────────────────────────────────

/// Missions a merc must have completed before being eligible for promotion
/// from the given quality tier.
pub fn promotion_missions_required(from: MercQuality) -> u32 {
    match from {
        MercQuality::Common => 3,
        MercQuality::Uncommon => 6,
        MercQuality::Rare => 12,
        MercQuality::Elite => u32::MAX, // can't promote from Elite
    }
}

/// Guild rank required to promote *to* the given quality tier.
pub fn promotion_guild_rank_required(to: MercQuality) -> u8 {
    match to {
        MercQuality::Common => 1, // unreachable in practice
        MercQuality::Uncommon => 2,
        MercQuality::Rare => 3,
        MercQuality::Elite => 4,
    }
}

/// Deterministic promotion cost for a merc (based on id) to the given tier.
/// Returns Warband Marks cost, rounded to nearest 5.
pub fn promotion_cost(merc_id: u64, to: MercQuality) -> u32 {
    let (min, max) = match to {
        MercQuality::Common => (0, 0), // unreachable
        MercQuality::Uncommon => (160, 260),
        MercQuality::Rare => (260, 400),
        MercQuality::Elite => (400, 600),
    };
    let range = max - min;
    if range == 0 {
        return 0;
    }
    // Simple hash: spread merc ids across the cost range
    let hash = (merc_id.wrapping_mul(2654435761) >> 16) as u32;
    let raw = min + hash % (range + 1);
    // Round to nearest 5
    ((raw + 2) / 5) * 5
}

// ── Promotion Eligibility & Application ──────────────────────────────────────

/// Error type for merc promotion attempts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromotionError {
    AlreadyElite,
    InsufficientMissions { have: u32, need: u32 },
    InsufficientRank { have: u8, need: u8 },
    InsufficientMarks { have: u32, need: u32 },
}

/// Check whether a merc can be promoted. Returns (target quality, mark cost) on success.
pub fn can_promote(
    merc: &Mercenary,
    guild_rank: GuildRank,
    available_marks: u32,
) -> Result<(MercQuality, u32), PromotionError> {
    let target = merc.quality.next().ok_or(PromotionError::AlreadyElite)?;

    let missions_needed = promotion_missions_required(merc.quality);
    if merc.missions_completed < missions_needed {
        return Err(PromotionError::InsufficientMissions {
            have: merc.missions_completed,
            need: missions_needed,
        });
    }

    let rank_needed = promotion_guild_rank_required(target);
    if guild_rank.0 < rank_needed {
        return Err(PromotionError::InsufficientRank {
            have: guild_rank.0,
            need: rank_needed,
        });
    }

    let cost = promotion_cost(merc.id, target);
    if available_marks < cost {
        return Err(PromotionError::InsufficientMarks {
            have: available_marks,
            need: cost,
        });
    }

    Ok((target, cost))
}

/// Apply promotion stat deltas to a merc and update its quality tier.
fn apply_promotion_stats(merc: &mut Mercenary, target: MercQuality) {
    let flat_delta = target.flat_bonus() - merc.quality.flat_bonus();
    let primary_delta = target.primary_bonus() - merc.quality.primary_bonus();
    let (p_primary, r_primary, e_primary) = archetype_primary_flags(merc.archetype);

    merc.power += flat_delta + if p_primary { primary_delta } else { 0 };
    merc.resilience += flat_delta + if r_primary { primary_delta } else { 0 };
    merc.expertise += flat_delta + if e_primary { primary_delta } else { 0 };

    merc.quality = target;
}

/// Promote a merc to the next quality tier.
///
/// Validates eligibility, deducts marks, applies stat deltas, and updates quality.
/// Returns the mark cost on success.
pub fn promote_mercenary(
    merc: &mut Mercenary,
    prestige: &mut DeepPrestige,
    guild_rank: GuildRank,
) -> Result<u32, PromotionError> {
    let (target, cost) = can_promote(merc, guild_rank, prestige.warband_marks)?;
    prestige.spend_marks(cost);
    apply_promotion_stats(merc, target);
    Ok(cost)
}

/// Promote a merc by id, working around the borrow-checker constraint where
/// the merc lives inside `prestige.roster`.
///
/// Returns `(merc_name, quality_name, cost)` on success for UI display.
pub fn promote_merc_by_id(
    merc_id: u64,
    prestige: &mut DeepPrestige,
    guild_rank: GuildRank,
) -> Result<(String, &'static str, u32), PromotionError> {
    // Validate with immutable borrow first
    let (target, cost) = {
        let merc = prestige
            .find_merc(merc_id)
            .ok_or(PromotionError::AlreadyElite)?;
        can_promote(merc, guild_rank, prestige.warband_marks)?
    };

    // Now mutate
    prestige.spend_marks(cost);
    let merc = prestige.find_merc_mut(merc_id).unwrap();
    apply_promotion_stats(merc, target);

    let name = merc.name.clone();
    let quality_name = match merc.quality {
        MercQuality::Common => "Common",
        MercQuality::Uncommon => "Uncommon",
        MercQuality::Rare => "Rare",
        MercQuality::Elite => "Elite",
    };
    Ok((name, quality_name, cost))
}

// ── Guild-Rank Quality Tables ─────────────────────────────────────────────────

/// Returns the pool-size (number of candidates) for the given guild rank.
/// Rank 1: 3 candidates, Rank 2-3: 4, Rank 4-5: 5.
pub fn recruit_pool_size(guild_rank: GuildRank) -> usize {
    match guild_rank.0 {
        1 => 3,
        2 | 3 => 4,
        _ => 5,
    }
}

/// Roll a quality tier for a recruit based on guild rank.
///
/// Quality distribution by rank:
/// - Rank 1: 100% Common
/// - Rank 2: 60% Common, 40% Uncommon
/// - Rank 3: 30% Common, 50% Uncommon, 20% Rare
/// - Rank 4: 0% Common, 40% Uncommon, 50% Rare, 10% Elite
/// - Rank 5: 0% Common, 20% Uncommon, 50% Rare, 30% Elite
pub fn roll_recruit_quality(guild_rank: GuildRank, rng: &mut impl Rng) -> MercQuality {
    let roll: f64 = rng.random_range(0.0..100.0);
    match guild_rank.0 {
        1 => MercQuality::Common,
        2 => {
            if roll < 60.0 {
                MercQuality::Common
            } else {
                MercQuality::Uncommon
            }
        }
        3 => {
            if roll < 30.0 {
                MercQuality::Common
            } else if roll < 80.0 {
                MercQuality::Uncommon
            } else {
                MercQuality::Rare
            }
        }
        4 => {
            if roll < 40.0 {
                MercQuality::Uncommon
            } else if roll < 90.0 {
                MercQuality::Rare
            } else {
                MercQuality::Elite
            }
        }
        _ => {
            // Rank 5
            if roll < 20.0 {
                MercQuality::Uncommon
            } else if roll < 70.0 {
                MercQuality::Rare
            } else {
                MercQuality::Elite
            }
        }
    }
}

/// Archetypes available in the recruit pool at each guild rank.
///
/// - Rank 1: Vanguard, Scout, Medic only (introductory archetypes)
/// - Rank 2: + Arcanist
/// - Rank 3+: All archetypes including Saboteur
const RANK_1_ARCHETYPES: &[MercArchetype] = &[
    MercArchetype::Vanguard,
    MercArchetype::Scout,
    MercArchetype::Medic,
];

const RANK_2_ARCHETYPES: &[MercArchetype] = &[
    MercArchetype::Vanguard,
    MercArchetype::Scout,
    MercArchetype::Medic,
    MercArchetype::Arcanist,
];

const ALL_ARCHETYPES: &[MercArchetype] = MercArchetype::ALL;

fn available_archetypes(guild_rank: GuildRank) -> &'static [MercArchetype] {
    match guild_rank.0 {
        1 => RANK_1_ARCHETYPES,
        2 => RANK_2_ARCHETYPES,
        _ => ALL_ARCHETYPES,
    }
}

// ── Archetype Primary Stat Flags ──────────────────────────────────────────────

/// Returns (power_is_primary, resilience_is_primary, expertise_is_primary).
/// Used to apply `MercQuality::primary_bonus()` to the right stats.
pub fn archetype_primary_flags(archetype: MercArchetype) -> (bool, bool, bool) {
    match archetype {
        // Vanguard: STR/CON → high Power + Resilience
        MercArchetype::Vanguard => (true, true, false),
        // Scout: DEX/WIS → high Expertise + moderate Resilience
        MercArchetype::Scout => (false, true, true),
        // Arcanist: INT → highest Expertise
        MercArchetype::Arcanist => (false, false, true),
        // Medic: WIS/CON → highest Resilience
        MercArchetype::Medic => (false, true, true),
        // Saboteur: DEX/INT → high Expertise
        MercArchetype::Saboteur => (false, false, true),
    }
}

// ── Per-Level Stat Growth ─────────────────────────────────────────────────────

/// Growth per level for (Power, Resilience, Expertise), stored as fixed-point
/// (multiply by 10 to avoid f64 in save data).  Divide by 10.0 to get the
/// per-level float increment.
///
/// | Archetype | Power/Lvl | Resilience/Lvl | Expertise/Lvl |
/// |-----------|-----------|----------------|---------------|
/// | Vanguard  | 4.0       | 3.5            | 2.0           |
/// | Scout     | 3.0       | 3.0            | 3.5           |
/// | Arcanist  | 3.5       | 2.0            | 4.0           |
/// | Medic     | 2.0       | 3.5            | 3.0           |
/// | Saboteur  | 3.0       | 2.5            | 4.0           |
fn archetype_growth_per_level(archetype: MercArchetype) -> (f64, f64, f64) {
    match archetype {
        MercArchetype::Vanguard => (4.0, 3.5, 2.0),
        MercArchetype::Scout => (3.0, 3.0, 3.5),
        MercArchetype::Arcanist => (3.5, 2.0, 4.0),
        MercArchetype::Medic => (2.0, 3.5, 3.0),
        MercArchetype::Saboteur => (3.0, 2.5, 4.0),
    }
}

/// Compute stats for a mercenary at a given level.
///
/// Formula (from balance doc): `stat_at_level = base + growth_per_level * (level - 1)`
///
/// Returns (power, resilience, expertise).
pub fn stats_at_level(
    archetype: MercArchetype,
    base_power: u32,
    base_resilience: u32,
    base_expertise: u32,
    level: u32,
) -> (u32, u32, u32) {
    if level <= 1 {
        return (base_power, base_resilience, base_expertise);
    }
    let (pg, rg, eg) = archetype_growth_per_level(archetype);
    let levels_gained = (level - 1) as f64;
    let power = base_power + (pg * levels_gained).round() as u32;
    let resilience = base_resilience + (rg * levels_gained).round() as u32;
    let expertise = base_expertise + (eg * levels_gained).round() as u32;
    (power, resilience, expertise)
}

// ── Variance ──────────────────────────────────────────────────────────────────

/// Random variance range applied to base stats: ±10%.
const STAT_VARIANCE_MIN: f64 = 0.90;
const STAT_VARIANCE_MAX: f64 = 1.10;

fn apply_variance(value: u32, rng: &mut impl Rng) -> u32 {
    let factor: f64 = rng.random_range(STAT_VARIANCE_MIN..=STAT_VARIANCE_MAX);
    ((value as f64) * factor).round().max(1.0) as u32
}

// ── Name Generation ───────────────────────────────────────────────────────────

const MERC_FIRST_NAMES: &[&str] = &[
    "Aldric", "Brynn", "Calder", "Dagna", "Edric", "Freya", "Gareth", "Hilde", "Invar", "Jora",
    "Kael", "Lyra", "Maren", "Nessa", "Osric", "Petra", "Quill", "Runa", "Sven", "Tilda", "Ulric",
    "Vera", "Wulf", "Xena", "Yrsa", "Zora", "Bram", "Cira", "Dorn", "Elsa", "Finn", "Groa", "Holt",
    "Isla", "Jak", "Kira", "Leif", "Mora", "Njord", "Odda",
];

const VANGUARD_EPITHETS: &[&str] = &[
    "Ironwall",
    "Stonebreaker",
    "the Unyielding",
    "Shieldborn",
    "the Immovable",
    "the Bulwark",
    "Rampart",
    "Forgeborn",
    "Steadfast",
    "the Unbroken",
];

const SCOUT_EPITHETS: &[&str] = &[
    "Shadowfoot",
    "the Far-Eyed",
    "Duskwalker",
    "the Silent",
    "Swiftpath",
    "the Unseen",
    "Lightfoot",
    "Voidtracer",
    "the Keen",
    "Nightcrawler",
];

const ARCANIST_EPITHETS: &[&str] = &[
    "the Learned",
    "Runeweaver",
    "Voidcaller",
    "Spellwright",
    "the Arcane",
    "Deepreader",
    "the Knowing",
    "Glyphbinder",
    "Elementborn",
    "the Wise",
];

const MEDIC_EPITHETS: &[&str] = &[
    "the Mender",
    "Lifeguard",
    "Soulbinder",
    "the Tender",
    "Ironhands",
    "Bloodstopper",
    "the Steadying",
    "Warmhearted",
    "the Careful",
    "Vitalkeeper",
];

const SABOTEUR_EPITHETS: &[&str] = &[
    "the Cunning",
    "Trapbane",
    "Voidpicker",
    "the Slippery",
    "Ironfingers",
    "the Clever",
    "Lockpick",
    "Shadowcraft",
    "the Devious",
    "Wrenchborn",
];

fn archetype_epithets(archetype: MercArchetype) -> &'static [&'static str] {
    match archetype {
        MercArchetype::Vanguard => VANGUARD_EPITHETS,
        MercArchetype::Scout => SCOUT_EPITHETS,
        MercArchetype::Arcanist => ARCANIST_EPITHETS,
        MercArchetype::Medic => MEDIC_EPITHETS,
        MercArchetype::Saboteur => SABOTEUR_EPITHETS,
    }
}

/// Generate a thematic name for a mercenary.
/// Format: `"<FirstName> <Epithet>"` (e.g., "Aldric Ironwall", "Brynn the Silent").
pub fn generate_merc_name(archetype: MercArchetype, rng: &mut impl Rng) -> String {
    let first = MERC_FIRST_NAMES[rng.random_range(0..MERC_FIRST_NAMES.len())];
    let epithets = archetype_epithets(archetype);
    let epithet = epithets[rng.random_range(0..epithets.len())];
    format!("{} {}", first, epithet)
}

// ── Core Generation ───────────────────────────────────────────────────────────

/// Generate a single mercenary with the given archetype, quality, and unique id.
///
/// Stats are:
/// 1. Start from `archetype.base_stats()` (defined in types.rs)
/// 2. Add guild-rank flat bonus to all stats
/// 3. Add quality primary bonus to primary stats only
/// 4. Apply ±10% random variance per stat
///
/// The merc starts at Level 1 with `MercStatus::Available`.
pub fn generate_mercenary(
    id: u64,
    archetype: MercArchetype,
    quality: MercQuality,
    rng: &mut impl Rng,
) -> Mercenary {
    let (base_power, base_resilience, base_expertise) = archetype.base_stats();
    let flat = quality.flat_bonus();
    let primary = quality.primary_bonus();
    let (power_primary, resilience_primary, expertise_primary) = archetype_primary_flags(archetype);

    let raw_power = base_power + flat + if power_primary { primary } else { 0 };
    let raw_resilience = base_resilience + flat + if resilience_primary { primary } else { 0 };
    let raw_expertise = base_expertise + flat + if expertise_primary { primary } else { 0 };

    let power = apply_variance(raw_power, rng);
    let resilience = apply_variance(raw_resilience, rng);
    let expertise = apply_variance(raw_expertise, rng);

    let name = generate_merc_name(archetype, rng);

    Mercenary {
        id,
        name,
        archetype,
        power,
        resilience,
        expertise,
        level: Mercenary::BASE_LEVEL,
        missions_completed: 0,
        quality,
        status: MercStatus::Available,
    }
}

// ── Recruitment Pool ──────────────────────────────────────────────────────────

/// Generate a fresh recruitment pool for the given guild rank.
///
/// Pool size: 3 candidates at Rank 1, 4 at Rank 2-3, 5 at Rank 4-5.
/// Each candidate's archetype is sampled from the rank-appropriate archetype table.
/// Quality is rolled per-candidate using `roll_recruit_quality`.
/// Recruit costs are rolled once per candidate within the quality's cost range.
///
/// `next_id` is a closure that returns the next unique merc id (increments the
/// `DeepPersistent::merc_id_counter`).
pub fn generate_recruit_pool(
    guild_rank: GuildRank,
    mut next_id: impl FnMut() -> u64,
    rng: &mut impl Rng,
) -> RecruitPool {
    let pool_size = recruit_pool_size(guild_rank);
    let archetypes = available_archetypes(guild_rank);

    let mut candidates = Vec::with_capacity(pool_size);
    let mut recruit_costs = Vec::with_capacity(pool_size);

    for _ in 0..pool_size {
        let archetype = archetypes[rng.random_range(0..archetypes.len())];
        let quality = roll_recruit_quality(guild_rank, rng);
        let id = next_id();
        let merc = generate_mercenary(id, archetype, quality, rng);
        let (cost_min, cost_max) = quality.cost_range();
        // Round cost to nearest 5 for cleaner display
        let raw_cost = rng.random_range(cost_min..=cost_max);
        let cost = ((raw_cost as f64 / 5.0).round() as u32) * 5;
        candidates.push(merc);
        recruit_costs.push(cost);
    }

    RecruitPool {
        candidates,
        refreshed_at: Utc::now(),
        recruit_costs,
    }
}

// ── Starter Roster ────────────────────────────────────────────────────────────

/// Generate the 3 free starter mercenaries awarded on Deep discovery or prestige reset.
///
/// Composition: 1 Vanguard + 1 Scout + 1 Medic (the three introductory archetypes).
/// Quality matches the current guild rank (same as a Common-quality recruit for that rank).
/// Variance is still applied so each run feels different.
///
/// Returns a `Vec<Mercenary>` of exactly 3 mercs.
pub fn generate_starter_roster(
    guild_rank: GuildRank,
    mut next_id: impl FnMut() -> u64,
    rng: &mut impl Rng,
) -> Vec<Mercenary> {
    // Starter quality scales with guild rank so returning veterans get better recruits.
    let quality = match guild_rank.0 {
        1 | 2 => MercQuality::Common,
        3 => MercQuality::Uncommon,
        _ => MercQuality::Rare, // Rank 4-5
    };

    let starter_archetypes = [
        MercArchetype::Vanguard,
        MercArchetype::Scout,
        MercArchetype::Medic,
    ];

    starter_archetypes
        .iter()
        .map(|&archetype| {
            let id = next_id();
            generate_mercenary(id, archetype, quality, rng)
        })
        .collect()
}

// ── Level-Up Application ──────────────────────────────────────────────────────

/// XP required to advance from `level` to `level + 1`.
///
/// Formula (balance doc): `200 * level^1.3`
pub fn xp_to_next_level(level: u32) -> u32 {
    (200.0 * (level as f64).powf(1.3)).round() as u32
}

/// Apply mission XP to a mercenary, returning the number of levels gained.
///
/// Levels gained are applied immediately (stats scaled via `stats_at_level`).
/// The merc's `level` field is incremented; base stats (power/resilience/expertise)
/// are updated to reflect the new level.
///
/// Returns the count of levels gained (0 if none).
pub fn apply_merc_xp(merc: &mut Mercenary, xp_gained: u32) -> u32 {
    const MAX_LEVEL: u32 = 20;
    if merc.level >= MAX_LEVEL {
        return 0;
    }

    // We track xp_gained by counting how many level thresholds it crosses.
    // Since Mercenary has no persistent XP field, we reconstruct XP from missions_completed
    // as an approximation. However, the caller is responsible for tracking total XP
    // externally — this function uses a simple greedy approach: check if xp_gained
    // meets the threshold for current level.
    //
    // NOTE: The mission system will track per-merc XP in MissionResult::merc_level_ups.
    // This function just applies pre-computed level-ups; for XP accumulation see the
    // mission logic module.

    let levels_gained = xp_gained; // Caller passes levels, not raw XP, for simplicity.
    let old_level = merc.level;
    merc.level = (merc.level + levels_gained).min(MAX_LEVEL);
    let actual_levels = merc.level - old_level;

    if actual_levels > 0 {
        // Update base stats to reflect the new level using the growth formula.
        let (new_power, new_resilience, new_expertise) = stats_at_level(
            merc.archetype,
            archetype_base_power(merc.archetype),
            archetype_base_resilience(merc.archetype),
            archetype_base_expertise(merc.archetype),
            merc.level,
        );
        // Scale the merc's actual stats proportionally, preserving quality variance.
        // We compute the ratio between old and new archetype baseline stats, then
        // apply that scaling to the merc's actual (variance-adjusted) stats.
        let (old_base_p, old_base_r, old_base_e) = stats_at_level(
            merc.archetype,
            archetype_base_power(merc.archetype),
            archetype_base_resilience(merc.archetype),
            archetype_base_expertise(merc.archetype),
            old_level,
        );

        merc.power = scale_stat(merc.power, old_base_p, new_power);
        merc.resilience = scale_stat(merc.resilience, old_base_r, new_resilience);
        merc.expertise = scale_stat(merc.expertise, old_base_e, new_expertise);
    }

    actual_levels
}

/// Scale `current_stat` by the ratio of `new_base / old_base`, preserving variance.
fn scale_stat(current: u32, old_base: u32, new_base: u32) -> u32 {
    if old_base == 0 {
        return new_base;
    }
    let ratio = new_base as f64 / old_base as f64;
    ((current as f64 * ratio).round() as u32).max(1)
}

/// Raw base Power for the archetype at level 1 (from types.rs `base_stats()`).
fn archetype_base_power(archetype: MercArchetype) -> u32 {
    archetype.base_stats().0
}

fn archetype_base_resilience(archetype: MercArchetype) -> u32 {
    archetype.base_stats().1
}

fn archetype_base_expertise(archetype: MercArchetype) -> u32 {
    archetype.base_stats().2
}

// ── Roster Management ─────────────────────────────────────────────────────────

/// Whether the roster has room for another mercenary.
pub fn roster_has_capacity(roster: &HashMap<u64, Mercenary>, guild_rank: GuildRank) -> bool {
    roster.len() < guild_rank.max_roster() as usize
}

/// Available (non-injured, non-on-mission) mercenaries in the roster.
pub fn available_mercs(roster: &HashMap<u64, Mercenary>) -> Vec<&Mercenary> {
    roster.values().filter(|m| m.is_available()).collect()
}

/// Remove permanently-lost mercs from the roster (after death notification acknowledged).
/// Returns the number removed.
pub fn purge_lost_mercs(roster: &mut HashMap<u64, Mercenary>) -> u32 {
    let before = roster.len();
    roster.retain(|_, m| !matches!(m.status, MercStatus::Lost));
    (before - roster.len()) as u32
}

// ── Injury Resolution ─────────────────────────────────────────────────────────

/// Injury severity, used when generating injuries during mission resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InjurySeverity {
    /// 4-8 hours recovery.
    Light,
    /// 8-12 hours recovery.
    Moderate,
    /// 12-16 hours recovery (Medic-prevented loss).
    Severe,
}

impl InjurySeverity {
    /// Recovery duration range in seconds (min, max).
    pub fn recovery_secs(self) -> (u64, u64) {
        match self {
            InjurySeverity::Light => (4 * 3600, 8 * 3600),
            InjurySeverity::Moderate => (8 * 3600, 12 * 3600),
            InjurySeverity::Severe => (12 * 3600, 16 * 3600),
        }
    }
}

/// Roll a recovery duration in seconds for a given injury severity.
pub fn roll_injury_recovery_secs(severity: InjurySeverity, rng: &mut impl Rng) -> u64 {
    let (min, max) = severity.recovery_secs();
    rng.random_range(min..=max)
}

/// Apply an injury of the given severity to a mercenary.
/// Sets status to `MercStatus::Injured { recover_at }` using wall-clock time,
/// so recovery progresses even when no missions can be launched.
pub fn injure_merc(
    merc: &mut Mercenary,
    severity: InjurySeverity,
    now: chrono::DateTime<Utc>,
    rng: &mut impl Rng,
) {
    let recovery_secs = roll_injury_recovery_secs(severity, rng);
    merc.status = MercStatus::Injured {
        recover_at: now + chrono::Duration::seconds(recovery_secs as i64),
    };
}

/// Mark a mercenary as permanently lost.
/// The merc remains in the roster until `purge_lost_mercs` is called (after acknowledgement).
pub fn mark_merc_lost(merc: &mut Mercenary) {
    merc.status = MercStatus::Lost;
}

/// Heal every injured merc whose wall-clock recovery time has elapsed.
/// Call each game tick and on load (offline catch-up).
/// Returns the number of mercs that recovered.
pub fn check_injury_recovery(
    roster: &mut HashMap<u64, Mercenary>,
    now: chrono::DateTime<Utc>,
) -> u32 {
    let mut recovered = 0;
    for merc in roster.values_mut() {
        if let MercStatus::Injured { recover_at } = merc.status {
            if now >= recover_at {
                merc.status = MercStatus::Available;
                recovered += 1;
            }
        }
    }
    recovered
}

// ── Recruit Cost ──────────────────────────────────────────────────────────────

/// Generate a recruit cost for a merc of the given quality, rounded to nearest 5.
pub fn roll_recruit_cost(quality: MercQuality, rng: &mut impl Rng) -> u32 {
    let (min, max) = quality.cost_range();
    let raw = rng.random_range(min..=max);
    ((raw as f64 / 5.0).round() as u32) * 5
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn seeded_rng(seed: u64) -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(seed)
    }

    // ── Name Generation ───────────────────────────────────────────────────────

    #[test]
    fn test_generate_merc_name_is_non_empty() {
        let mut rng = seeded_rng(1);
        for &archetype in MercArchetype::ALL {
            let name = generate_merc_name(archetype, &mut rng);
            assert!(!name.is_empty(), "{:?} name should not be empty", archetype);
            assert!(
                name.contains(' '),
                "{:?} name should have a space: '{}'",
                archetype,
                name
            );
        }
    }

    #[test]
    fn test_generate_merc_name_archetype_epithets_are_distinct() {
        // Vanguard and Scout should never share an epithet (just verify tables differ)
        let vanguard_names: std::collections::HashSet<_> = VANGUARD_EPITHETS.iter().collect();
        let scout_names: std::collections::HashSet<_> = SCOUT_EPITHETS.iter().collect();
        let overlap: Vec<_> = vanguard_names.intersection(&scout_names).collect();
        assert!(
            overlap.is_empty(),
            "Vanguard and Scout epithets should not overlap: {:?}",
            overlap
        );
    }

    // ── Quality Distribution ──────────────────────────────────────────────────

    #[test]
    fn test_roll_recruit_quality_rank1_always_common() {
        let mut rng = seeded_rng(42);
        for _ in 0..1000 {
            assert_eq!(
                roll_recruit_quality(GuildRank(1), &mut rng),
                MercQuality::Common
            );
        }
    }

    #[test]
    fn test_roll_recruit_quality_rank2_distribution() {
        let mut rng = seeded_rng(99);
        let n = 10_000;
        let mut common = 0u32;
        let mut uncommon = 0u32;
        for _ in 0..n {
            match roll_recruit_quality(GuildRank(2), &mut rng) {
                MercQuality::Common => common += 1,
                MercQuality::Uncommon => uncommon += 1,
                q => panic!("Rank 2 should not produce {:?}", q),
            }
        }
        let common_rate = common as f64 / n as f64;
        let uncommon_rate = uncommon as f64 / n as f64;
        // Expected: 60% Common, 40% Uncommon (±3% tolerance)
        assert!(
            (common_rate - 0.60).abs() < 0.03,
            "Rank 2 Common rate: {:.2}%",
            common_rate * 100.0
        );
        assert!(
            (uncommon_rate - 0.40).abs() < 0.03,
            "Rank 2 Uncommon rate: {:.2}%",
            uncommon_rate * 100.0
        );
    }

    #[test]
    fn test_roll_recruit_quality_rank5_no_common() {
        let mut rng = seeded_rng(7);
        for _ in 0..1000 {
            let q = roll_recruit_quality(GuildRank(5), &mut rng);
            assert_ne!(
                q,
                MercQuality::Common,
                "Rank 5 should never produce Common quality"
            );
        }
    }

    // ── Mercenary Generation ──────────────────────────────────────────────────

    #[test]
    fn test_generate_mercenary_all_stats_nonzero() {
        let mut rng = seeded_rng(123);
        for &archetype in MercArchetype::ALL {
            let merc = generate_mercenary(1, archetype, MercQuality::Common, &mut rng);
            assert!(merc.power > 0, "{:?} power must be > 0", archetype);
            assert!(
                merc.resilience > 0,
                "{:?} resilience must be > 0",
                archetype
            );
            assert!(merc.expertise > 0, "{:?} expertise must be > 0", archetype);
        }
    }

    #[test]
    fn test_generate_mercenary_starts_at_level_1_available() {
        let mut rng = seeded_rng(5);
        let merc = generate_mercenary(1, MercArchetype::Vanguard, MercQuality::Common, &mut rng);
        assert_eq!(merc.level, Mercenary::BASE_LEVEL);
        assert!(merc.is_available());
        assert_eq!(merc.missions_completed, 0);
    }

    #[test]
    fn test_generate_mercenary_higher_quality_has_higher_stats() {
        // Average over many samples: Rare should beat Common for total stats
        let n = 200;
        let mut common_total = 0u64;
        let mut rare_total = 0u64;
        for seed in 0..n {
            let mut rng = seeded_rng(seed);
            let m_common =
                generate_mercenary(1, MercArchetype::Vanguard, MercQuality::Common, &mut rng);
            let mut rng = seeded_rng(seed);
            let m_rare =
                generate_mercenary(1, MercArchetype::Vanguard, MercQuality::Rare, &mut rng);
            common_total += (m_common.power + m_common.resilience + m_common.expertise) as u64;
            rare_total += (m_rare.power + m_rare.resilience + m_rare.expertise) as u64;
        }
        assert!(
            rare_total > common_total,
            "Rare total ({}) should exceed Common total ({})",
            rare_total,
            common_total
        );
    }

    #[test]
    fn test_generate_mercenary_vanguard_has_high_power_and_resilience() {
        let n = 100;
        let mut rng = seeded_rng(11);
        for _ in 0..n {
            let v = generate_mercenary(1, MercArchetype::Vanguard, MercQuality::Common, &mut rng);
            // Vanguard base: 14 power, 12 resilience, 4 expertise — power should lead
            assert!(
                v.power > v.expertise,
                "Vanguard power ({}) should exceed expertise ({})",
                v.power,
                v.expertise
            );
        }
    }

    #[test]
    fn test_generate_mercenary_arcanist_has_highest_expertise() {
        let mut rng = seeded_rng(22);
        let a = generate_mercenary(1, MercArchetype::Arcanist, MercQuality::Common, &mut rng);
        // Arcanist base expertise: 14 — should always exceed power (10) and resilience (6)
        assert!(
            a.expertise >= a.power,
            "Arcanist expertise ({}) should be >= power ({})",
            a.expertise,
            a.power
        );
    }

    #[test]
    fn test_generate_mercenary_medic_has_highest_resilience() {
        let n = 100;
        let mut rng = seeded_rng(33);
        let mut medic_resilience_beats_power = 0u32;
        for _ in 0..n {
            let m = generate_mercenary(1, MercArchetype::Medic, MercQuality::Common, &mut rng);
            if m.resilience >= m.power {
                medic_resilience_beats_power += 1;
            }
        }
        // With variance, Medic resilience (base 14) should exceed power (base 6) nearly always
        assert!(
            medic_resilience_beats_power > 90,
            "Medic resilience should beat power in > 90% of cases, got {}%",
            medic_resilience_beats_power
        );
    }

    // ── Recruit Pool ──────────────────────────────────────────────────────────

    #[test]
    fn test_recruit_pool_size_by_rank() {
        assert_eq!(recruit_pool_size(GuildRank(1)), 3);
        assert_eq!(recruit_pool_size(GuildRank(2)), 4);
        assert_eq!(recruit_pool_size(GuildRank(3)), 4);
        assert_eq!(recruit_pool_size(GuildRank(4)), 5);
        assert_eq!(recruit_pool_size(GuildRank(5)), 5);
    }

    #[test]
    fn test_generate_recruit_pool_correct_size() {
        let mut rng = seeded_rng(77);
        let mut id_counter = 0u64;
        for rank in 1..=5 {
            let pool = generate_recruit_pool(
                GuildRank(rank),
                || {
                    id_counter += 1;
                    id_counter
                },
                &mut rng,
            );
            assert_eq!(
                pool.candidates.len(),
                recruit_pool_size(GuildRank(rank)),
                "Rank {} pool should have {} candidates",
                rank,
                recruit_pool_size(GuildRank(rank))
            );
            assert_eq!(
                pool.candidates.len(),
                pool.recruit_costs.len(),
                "Candidates and costs must be aligned"
            );
        }
    }

    #[test]
    fn test_generate_recruit_pool_rank1_only_starter_archetypes() {
        let mut rng = seeded_rng(88);
        let mut id_counter = 0u64;
        // Run many pools to get statistical coverage
        for _ in 0..50 {
            let pool = generate_recruit_pool(
                GuildRank(1),
                || {
                    id_counter += 1;
                    id_counter
                },
                &mut rng,
            );
            for merc in &pool.candidates {
                assert!(
                    matches!(
                        merc.archetype,
                        MercArchetype::Vanguard | MercArchetype::Scout | MercArchetype::Medic
                    ),
                    "Rank 1 should only produce Vanguard/Scout/Medic, got {:?}",
                    merc.archetype
                );
            }
        }
    }

    #[test]
    fn test_generate_recruit_pool_costs_are_reasonable() {
        let mut rng = seeded_rng(55);
        let mut id_counter = 0u64;
        let pool = generate_recruit_pool(
            GuildRank(3),
            || {
                id_counter += 1;
                id_counter
            },
            &mut rng,
        );
        for &cost in &pool.recruit_costs {
            assert!(cost >= 25, "Recruit cost {} is too low", cost);
            assert!(cost <= 200, "Recruit cost {} is too high", cost);
            // Must be a multiple of 5
            assert_eq!(cost % 5, 0, "Cost {} should be multiple of 5", cost);
        }
    }

    #[test]
    fn test_generate_recruit_pool_ids_are_unique() {
        let mut rng = seeded_rng(66);
        let mut id_counter = 0u64;
        let pool = generate_recruit_pool(
            GuildRank(5),
            || {
                id_counter += 1;
                id_counter
            },
            &mut rng,
        );
        let ids: std::collections::HashSet<_> = pool.candidates.iter().map(|m| m.id).collect();
        assert_eq!(
            ids.len(),
            pool.candidates.len(),
            "All candidate ids must be unique"
        );
    }

    // ── Starter Roster ────────────────────────────────────────────────────────

    #[test]
    fn test_generate_starter_roster_is_three_mercs() {
        let mut rng = seeded_rng(100);
        let mut id_counter = 0u64;
        let roster = generate_starter_roster(
            GuildRank(1),
            || {
                id_counter += 1;
                id_counter
            },
            &mut rng,
        );
        assert_eq!(roster.len(), 3);
    }

    #[test]
    fn test_generate_starter_roster_contains_correct_archetypes() {
        let mut rng = seeded_rng(101);
        let mut id_counter = 0u64;
        let roster = generate_starter_roster(
            GuildRank(1),
            || {
                id_counter += 1;
                id_counter
            },
            &mut rng,
        );
        let archetypes: Vec<_> = roster.iter().map(|m| m.archetype).collect();
        assert!(
            archetypes.contains(&MercArchetype::Vanguard),
            "Starter roster must contain Vanguard"
        );
        assert!(
            archetypes.contains(&MercArchetype::Scout),
            "Starter roster must contain Scout"
        );
        assert!(
            archetypes.contains(&MercArchetype::Medic),
            "Starter roster must contain Medic"
        );
    }

    #[test]
    fn test_generate_starter_roster_all_available() {
        let mut rng = seeded_rng(102);
        let mut id_counter = 0u64;
        let roster = generate_starter_roster(
            GuildRank(2),
            || {
                id_counter += 1;
                id_counter
            },
            &mut rng,
        );
        for merc in &roster {
            assert!(merc.is_available(), "{} should start Available", merc.name);
        }
    }

    #[test]
    fn test_generate_starter_roster_ids_are_unique() {
        let mut rng = seeded_rng(103);
        let mut id_counter = 0u64;
        let roster = generate_starter_roster(
            GuildRank(1),
            || {
                id_counter += 1;
                id_counter
            },
            &mut rng,
        );
        let ids: std::collections::HashSet<_> = roster.iter().map(|m| m.id).collect();
        assert_eq!(ids.len(), 3, "Starter roster ids must all be unique");
    }

    // ── XP / Level Scaling ────────────────────────────────────────────────────

    #[test]
    fn test_xp_to_next_level_increases_with_level() {
        let mut prev = 0;
        for lvl in 1..=19 {
            let xp = xp_to_next_level(lvl);
            assert!(
                xp > prev,
                "Level {} XP ({}) should exceed level {} XP ({})",
                lvl + 1,
                xp,
                lvl,
                prev
            );
            prev = xp;
        }
    }

    #[test]
    fn test_xp_to_next_level_matches_design_doc() {
        // Level 1->2: 200, Level 2->3: ~492 (200 * 2^1.3 ≈ 491.5)
        assert!((xp_to_next_level(1) as i32 - 200).abs() < 5);
        let l2 = xp_to_next_level(2);
        assert!(l2 > 400 && l2 < 600, "Level 2->3 XP {} should be ~492", l2);
    }

    #[test]
    fn test_stats_at_level_vanguard_matches_design_doc() {
        // Balance doc: Rank 1 Vanguard Level 10 should have Power ~48, Resilience ~46, Expertise ~26
        let (p, r, e) = stats_at_level(MercArchetype::Vanguard, 12, 14, 8, 10);
        // Power: 12 + 4.0*9 = 12+36 = 48
        assert_eq!(p, 48, "Vanguard Level 10 Power should be 48, got {}", p);
        // Resilience: 14 + 3.5*9 = 14+31.5 = 46 (rounded)
        assert!(
            (r as i32 - 46).abs() <= 1,
            "Vanguard Level 10 Resilience should be ~46, got {}",
            r
        );
        // Expertise: 8 + 2.0*9 = 8+18 = 26
        assert_eq!(e, 26, "Vanguard Level 10 Expertise should be 26, got {}", e);
    }

    #[test]
    fn test_stats_at_level_level_1_returns_base() {
        let (p, r, e) = stats_at_level(MercArchetype::Scout, 8, 10, 12, 1);
        assert_eq!(p, 8);
        assert_eq!(r, 10);
        assert_eq!(e, 12);
    }

    // ── Injury System ─────────────────────────────────────────────────────────

    #[test]
    fn test_roll_injury_recovery_within_range() {
        let mut rng = seeded_rng(200);
        for severity in [
            InjurySeverity::Light,
            InjurySeverity::Moderate,
            InjurySeverity::Severe,
        ] {
            let (min, max) = severity.recovery_secs();
            for _ in 0..50 {
                let secs = roll_injury_recovery_secs(severity, &mut rng);
                assert!(
                    secs >= min && secs <= max,
                    "{:?} recovery {} should be in [{}, {}]",
                    severity,
                    secs,
                    min,
                    max
                );
            }
        }
    }

    #[test]
    fn test_injure_merc_sets_injured_status() {
        let mut rng = seeded_rng(201);
        let now = Utc::now();
        let mut merc =
            generate_mercenary(1, MercArchetype::Vanguard, MercQuality::Common, &mut rng);
        injure_merc(&mut merc, InjurySeverity::Moderate, now, &mut rng);
        assert!(
            matches!(merc.status, MercStatus::Injured { recover_at } if recover_at > now),
            "Merc should be Injured with a future recovery time after injure_merc()"
        );
        assert!(!merc.is_available());
    }

    #[test]
    fn test_injure_merc_recovery_within_severity_range() {
        let mut rng = seeded_rng(205);
        let now = Utc::now();
        let mut merc = generate_mercenary(1, MercArchetype::Scout, MercQuality::Common, &mut rng);
        injure_merc(&mut merc, InjurySeverity::Severe, now, &mut rng);
        let MercStatus::Injured { recover_at } = merc.status else {
            panic!("Merc should be Injured");
        };
        let (min, max) = InjurySeverity::Severe.recovery_secs();
        let secs = (recover_at - now).num_seconds();
        assert!(
            (min as i64..=max as i64).contains(&secs),
            "Severe recovery {}s should be in [{}, {}]",
            secs,
            min,
            max
        );
    }

    #[test]
    fn test_check_injury_recovery_heals_elapsed_injuries() {
        let mut rng = seeded_rng(202);
        let now = Utc::now();
        let mut merc = generate_mercenary(1, MercArchetype::Medic, MercQuality::Common, &mut rng);
        merc.status = MercStatus::Injured {
            recover_at: now - chrono::Duration::seconds(1),
        };
        let mut roster: HashMap<u64, Mercenary> = vec![(1, merc)].into_iter().collect();
        let recovered = check_injury_recovery(&mut roster, now);
        assert_eq!(recovered, 1, "Merc past recover_at should heal");
        assert!(roster[&1].is_available());
    }

    #[test]
    fn test_check_injury_recovery_leaves_pending_injuries() {
        let mut rng = seeded_rng(203);
        let now = Utc::now();
        let mut merc = generate_mercenary(1, MercArchetype::Scout, MercQuality::Common, &mut rng);
        let recover_at = now + chrono::Duration::hours(3);
        merc.status = MercStatus::Injured { recover_at };
        let mut roster: HashMap<u64, Mercenary> = vec![(1, merc)].into_iter().collect();
        let recovered = check_injury_recovery(&mut roster, now);
        assert_eq!(recovered, 0);
        assert!(
            matches!(roster[&1].status, MercStatus::Injured { recover_at: t } if t == recover_at),
            "Injury with time remaining should be untouched"
        );
    }

    #[test]
    fn test_mark_merc_lost_and_purge() {
        let mut rng = seeded_rng(204);
        let m1 = generate_mercenary(1, MercArchetype::Vanguard, MercQuality::Common, &mut rng);
        let m2 = generate_mercenary(2, MercArchetype::Scout, MercQuality::Common, &mut rng);
        let mut roster: HashMap<u64, Mercenary> = vec![(1, m1), (2, m2)].into_iter().collect();
        mark_merc_lost(roster.get_mut(&1).unwrap());
        assert!(matches!(roster[&1].status, MercStatus::Lost));
        let purged = purge_lost_mercs(&mut roster);
        assert_eq!(purged, 1);
        assert_eq!(roster.len(), 1);
        assert!(roster.contains_key(&2));
    }

    // ── Roster Management ─────────────────────────────────────────────────────

    #[test]
    fn test_roster_has_capacity() {
        let mut rng = seeded_rng(300);
        let mut roster = HashMap::new();
        assert!(roster_has_capacity(&roster, GuildRank(1))); // 0/5
        for i in 0..5 {
            let merc =
                generate_mercenary(i, MercArchetype::Vanguard, MercQuality::Common, &mut rng);
            roster.insert(i, merc);
        }
        assert!(!roster_has_capacity(&roster, GuildRank(1))); // 5/5 full
        assert!(roster_has_capacity(&roster, GuildRank(2))); // 5/7 ok
    }

    #[test]
    fn test_available_mercs_filters_correctly() {
        let mut rng = seeded_rng(301);
        let mut merc1 =
            generate_mercenary(1, MercArchetype::Vanguard, MercQuality::Common, &mut rng);
        let mut merc2 = generate_mercenary(2, MercArchetype::Scout, MercQuality::Common, &mut rng);
        let merc3 = generate_mercenary(3, MercArchetype::Medic, MercQuality::Common, &mut rng);
        merc1.status = MercStatus::OnMission(42);
        merc2.status = MercStatus::Injured {
            recover_at: Utc::now() + chrono::Duration::hours(6),
        };
        let roster: HashMap<u64, Mercenary> = vec![(1, merc1), (2, merc2), (3, merc3)]
            .into_iter()
            .collect();
        let available = available_mercs(&roster);
        assert_eq!(available.len(), 1);
        assert_eq!(available[0].id, 3);
    }

    // ── Recruit Cost ──────────────────────────────────────────────────────────

    #[test]
    fn test_roll_recruit_cost_within_range_and_multiple_of_5() {
        let mut rng = seeded_rng(400);
        for quality in [
            MercQuality::Common,
            MercQuality::Uncommon,
            MercQuality::Rare,
            MercQuality::Elite,
        ] {
            let (min, max) = quality.cost_range();
            let rounded_min = ((min as f64 / 5.0).round() as u32) * 5;
            let rounded_max = ((max as f64 / 5.0).round() as u32) * 5;
            for _ in 0..50 {
                let cost = roll_recruit_cost(quality, &mut rng);
                assert_eq!(cost % 5, 0, "Cost {} must be multiple of 5", cost);
                assert!(
                    cost >= rounded_min && cost <= rounded_max + 5,
                    "{:?} cost {} should be in [{}, {}]",
                    quality,
                    cost,
                    rounded_min,
                    rounded_max
                );
            }
        }
    }

    #[test]
    fn test_recruit_cost_elite_more_expensive_than_common() {
        let mut rng = seeded_rng(401);
        let common_costs: Vec<u32> = (0..50)
            .map(|_| roll_recruit_cost(MercQuality::Common, &mut rng))
            .collect();
        let elite_costs: Vec<u32> = (0..50)
            .map(|_| roll_recruit_cost(MercQuality::Elite, &mut rng))
            .collect();
        let common_avg = common_costs.iter().sum::<u32>() as f64 / 50.0;
        let elite_avg = elite_costs.iter().sum::<u32>() as f64 / 50.0;
        assert!(
            elite_avg > common_avg,
            "Elite avg cost ({:.1}) should exceed Common avg cost ({:.1})",
            elite_avg,
            common_avg
        );
    }

    #[test]
    fn test_merc_quality_serde_default() {
        // Simulate a legacy save without quality field
        let json = r#"{
            "id": 1,
            "name": "Test",
            "archetype": "Vanguard",
            "power": 14,
            "resilience": 12,
            "expertise": 4,
            "level": 1,
            "missions_completed": 0,
            "status": "Available"
        }"#;
        let merc: Mercenary = serde_json::from_str(json).unwrap();
        assert_eq!(merc.quality, MercQuality::Common);
    }

    #[test]
    fn test_merc_quality_stored_on_generation() {
        let mut rng = seeded_rng(42);
        let merc = generate_mercenary(1, MercArchetype::Vanguard, MercQuality::Rare, &mut rng);
        assert_eq!(merc.quality, MercQuality::Rare);
    }

    #[test]
    fn test_merc_quality_next() {
        assert_eq!(MercQuality::Common.next(), Some(MercQuality::Uncommon));
        assert_eq!(MercQuality::Uncommon.next(), Some(MercQuality::Rare));
        assert_eq!(MercQuality::Rare.next(), Some(MercQuality::Elite));
        assert_eq!(MercQuality::Elite.next(), None);
    }

    #[test]
    fn test_promotion_missions_required() {
        assert_eq!(promotion_missions_required(MercQuality::Common), 3);
        assert_eq!(promotion_missions_required(MercQuality::Uncommon), 6);
        assert_eq!(promotion_missions_required(MercQuality::Rare), 12);
    }

    #[test]
    fn test_promotion_guild_rank_required() {
        assert_eq!(promotion_guild_rank_required(MercQuality::Uncommon), 2);
        assert_eq!(promotion_guild_rank_required(MercQuality::Rare), 3);
        assert_eq!(promotion_guild_rank_required(MercQuality::Elite), 4);
    }

    #[test]
    fn test_promotion_cost_deterministic() {
        let cost1 = promotion_cost(42, MercQuality::Uncommon);
        let cost2 = promotion_cost(42, MercQuality::Uncommon);
        assert_eq!(
            cost1, cost2,
            "Same merc id + tier should always give same cost"
        );
        assert!(
            (160..=260).contains(&cost1),
            "Uncommon cost out of range: {}",
            cost1
        );
        assert_eq!(cost1 % 5, 0, "Cost should be rounded to nearest 5");
    }

    #[test]
    fn test_promotion_cost_ranges() {
        for id in 0..100u64 {
            let u = promotion_cost(id, MercQuality::Uncommon);
            assert!(
                (160..=260).contains(&u),
                "Uncommon cost {} out of range for id {}",
                u,
                id
            );
            let r = promotion_cost(id, MercQuality::Rare);
            assert!(
                (260..=400).contains(&r),
                "Rare cost {} out of range for id {}",
                r,
                id
            );
            let e = promotion_cost(id, MercQuality::Elite);
            assert!(
                (400..=600).contains(&e),
                "Elite cost {} out of range for id {}",
                e,
                id
            );
        }
    }

    // ── Promotion Eligibility ─────────────────────────────────────────────────

    #[test]
    fn test_can_promote_common_merc() {
        let mut rng = seeded_rng(1);
        let mut merc =
            generate_mercenary(1, MercArchetype::Vanguard, MercQuality::Common, &mut rng);
        merc.missions_completed = 5;
        let cost = promotion_cost(1, MercQuality::Uncommon);
        let result = can_promote(&merc, GuildRank(2), cost + 100);
        assert!(result.is_ok());
        let (target, mark_cost) = result.unwrap();
        assert_eq!(target, MercQuality::Uncommon);
        assert_eq!(mark_cost, cost);
    }

    #[test]
    fn test_can_promote_blocked_by_missions() {
        let mut rng = seeded_rng(1);
        let merc = generate_mercenary(1, MercArchetype::Vanguard, MercQuality::Common, &mut rng);
        let result = can_promote(&merc, GuildRank(2), 9999);
        assert!(matches!(
            result,
            Err(PromotionError::InsufficientMissions { .. })
        ));
    }

    #[test]
    fn test_can_promote_blocked_by_rank() {
        let mut rng = seeded_rng(1);
        let mut merc =
            generate_mercenary(1, MercArchetype::Vanguard, MercQuality::Common, &mut rng);
        merc.missions_completed = 5;
        let result = can_promote(&merc, GuildRank(1), 9999);
        assert!(matches!(
            result,
            Err(PromotionError::InsufficientRank { .. })
        ));
    }

    #[test]
    fn test_can_promote_blocked_by_marks() {
        let mut rng = seeded_rng(1);
        let mut merc =
            generate_mercenary(1, MercArchetype::Vanguard, MercQuality::Common, &mut rng);
        merc.missions_completed = 5;
        let result = can_promote(&merc, GuildRank(2), 0);
        assert!(matches!(
            result,
            Err(PromotionError::InsufficientMarks { .. })
        ));
    }

    #[test]
    fn test_can_promote_blocked_for_elite() {
        let mut rng = seeded_rng(1);
        let mut merc = generate_mercenary(1, MercArchetype::Vanguard, MercQuality::Elite, &mut rng);
        merc.missions_completed = 100;
        let result = can_promote(&merc, GuildRank(5), 9999);
        assert!(matches!(result, Err(PromotionError::AlreadyElite)));
    }

    #[test]
    fn test_promote_mercenary_applies_stat_deltas() {
        use crate::deep::types::DeepPrestige;
        let mut rng = seeded_rng(1);
        let mut merc =
            generate_mercenary(1, MercArchetype::Vanguard, MercQuality::Common, &mut rng);
        merc.missions_completed = 5;
        let pre_power = merc.power;
        let pre_resilience = merc.resilience;
        let pre_expertise = merc.expertise;

        let mut prestige = DeepPrestige {
            warband_marks: 1000,
            ..Default::default()
        };

        let result = promote_mercenary(&mut merc, &mut prestige, GuildRank(2));
        assert!(result.is_ok());
        assert_eq!(merc.quality, MercQuality::Uncommon);

        // Vanguard primaries: power + resilience
        // Common→Uncommon: flat +2, primary +2
        assert_eq!(merc.power, pre_power + 2 + 2); // flat + primary
        assert_eq!(merc.resilience, pre_resilience + 2 + 2); // flat + primary
        assert_eq!(merc.expertise, pre_expertise + 2); // flat only

        // Marks deducted
        let cost = promotion_cost(1, MercQuality::Uncommon);
        assert_eq!(prestige.warband_marks, 1000 - cost);
    }

    #[test]
    fn test_promote_mercenary_rare_to_elite() {
        use crate::deep::types::DeepPrestige;
        let mut rng = seeded_rng(1);
        let mut merc = generate_mercenary(1, MercArchetype::Arcanist, MercQuality::Rare, &mut rng);
        merc.missions_completed = 15;
        let pre_power = merc.power;
        let pre_resilience = merc.resilience;
        let pre_expertise = merc.expertise;

        let mut prestige = DeepPrestige {
            warband_marks: 1000,
            ..Default::default()
        };

        let result = promote_mercenary(&mut merc, &mut prestige, GuildRank(4));
        assert!(result.is_ok());
        assert_eq!(merc.quality, MercQuality::Elite);

        // Arcanist primary: expertise only
        // Rare→Elite: flat_delta = 8-4 = 4, primary_delta = 4-2 = 2
        assert_eq!(merc.power, pre_power + 4); // flat only
        assert_eq!(merc.resilience, pre_resilience + 4); // flat only
        assert_eq!(merc.expertise, pre_expertise + 4 + 2); // flat + primary
    }

    // ── Recruit Quality Distribution (Ranks 3-5) ─────────────────────────────

    #[test]
    fn test_roll_recruit_quality_rank3_distribution() {
        let mut rng = seeded_rng(909);
        let n = 10_000;
        let mut common = 0u32;
        let mut uncommon = 0u32;
        let mut rare = 0u32;
        for _ in 0..n {
            match roll_recruit_quality(GuildRank(3), &mut rng) {
                MercQuality::Common => common += 1,
                MercQuality::Uncommon => uncommon += 1,
                MercQuality::Rare => rare += 1,
                q => panic!("Rank 3 should not produce {:?}", q),
            }
        }
        let common_rate = common as f64 / n as f64;
        let uncommon_rate = uncommon as f64 / n as f64;
        let rare_rate = rare as f64 / n as f64;
        // Expected: 30% Common, 50% Uncommon, 20% Rare (±3% tolerance)
        assert!(
            (common_rate - 0.30).abs() < 0.03,
            "Rank 3 Common rate: {:.2}%",
            common_rate * 100.0
        );
        assert!(
            (uncommon_rate - 0.50).abs() < 0.03,
            "Rank 3 Uncommon rate: {:.2}%",
            uncommon_rate * 100.0
        );
        assert!(
            (rare_rate - 0.20).abs() < 0.03,
            "Rank 3 Rare rate: {:.2}%",
            rare_rate * 100.0
        );
    }

    #[test]
    fn test_roll_recruit_quality_rank4_distribution() {
        let mut rng = seeded_rng(910);
        let n = 10_000;
        let mut uncommon = 0u32;
        let mut rare = 0u32;
        let mut elite = 0u32;
        for _ in 0..n {
            match roll_recruit_quality(GuildRank(4), &mut rng) {
                MercQuality::Uncommon => uncommon += 1,
                MercQuality::Rare => rare += 1,
                MercQuality::Elite => elite += 1,
                q => panic!("Rank 4 should never produce {:?}", q),
            }
        }
        let uncommon_rate = uncommon as f64 / n as f64;
        let rare_rate = rare as f64 / n as f64;
        let elite_rate = elite as f64 / n as f64;
        // Expected: 40% Uncommon, 50% Rare, 10% Elite (±3% tolerance)
        assert!(
            (uncommon_rate - 0.40).abs() < 0.03,
            "Rank 4 Uncommon rate: {:.2}%",
            uncommon_rate * 100.0
        );
        assert!(
            (rare_rate - 0.50).abs() < 0.03,
            "Rank 4 Rare rate: {:.2}%",
            rare_rate * 100.0
        );
        assert!(
            (elite_rate - 0.10).abs() < 0.03,
            "Rank 4 Elite rate: {:.2}%",
            elite_rate * 100.0
        );
    }

    #[test]
    fn test_roll_recruit_quality_rank5_distribution() {
        let mut rng = seeded_rng(911);
        let n = 10_000;
        let mut uncommon = 0u32;
        let mut rare = 0u32;
        let mut elite = 0u32;
        for _ in 0..n {
            match roll_recruit_quality(GuildRank(5), &mut rng) {
                MercQuality::Uncommon => uncommon += 1,
                MercQuality::Rare => rare += 1,
                MercQuality::Elite => elite += 1,
                q => panic!("Rank 5 should never produce {:?}", q),
            }
        }
        let uncommon_rate = uncommon as f64 / n as f64;
        let rare_rate = rare as f64 / n as f64;
        let elite_rate = elite as f64 / n as f64;
        // Expected: 20% Uncommon, 50% Rare, 30% Elite (±3% tolerance)
        assert!(
            (uncommon_rate - 0.20).abs() < 0.03,
            "Rank 5 Uncommon rate: {:.2}%",
            uncommon_rate * 100.0
        );
        assert!(
            (rare_rate - 0.50).abs() < 0.03,
            "Rank 5 Rare rate: {:.2}%",
            rare_rate * 100.0
        );
        assert!(
            (elite_rate - 0.30).abs() < 0.03,
            "Rank 5 Elite rate: {:.2}%",
            elite_rate * 100.0
        );
    }

    // ── Archetype Gating by Rank ──────────────────────────────────────────────

    #[test]
    fn test_recruit_pool_rank2_includes_arcanist_but_not_saboteur() {
        let mut rng = seeded_rng(912);
        let mut id_counter = 0u64;
        let mut saw_arcanist = false;
        for _ in 0..100 {
            let pool = generate_recruit_pool(
                GuildRank(2),
                || {
                    id_counter += 1;
                    id_counter
                },
                &mut rng,
            );
            for merc in &pool.candidates {
                assert_ne!(
                    merc.archetype,
                    MercArchetype::Saboteur,
                    "Rank 2 should never produce Saboteur"
                );
                if merc.archetype == MercArchetype::Arcanist {
                    saw_arcanist = true;
                }
            }
        }
        assert!(saw_arcanist, "Rank 2 should be able to produce Arcanist");
    }

    #[test]
    fn test_recruit_pool_rank3_includes_saboteur() {
        let mut rng = seeded_rng(913);
        let mut id_counter = 0u64;
        let mut saw_saboteur = false;
        for _ in 0..100 {
            let pool = generate_recruit_pool(
                GuildRank(3),
                || {
                    id_counter += 1;
                    id_counter
                },
                &mut rng,
            );
            for merc in &pool.candidates {
                if merc.archetype == MercArchetype::Saboteur {
                    saw_saboteur = true;
                }
            }
        }
        assert!(saw_saboteur, "Rank 3+ should be able to produce Saboteur");
    }

    // ── Archetype Primary Flags ──────────────────────────────────────────────

    #[test]
    fn test_archetype_primary_flags_all_variants() {
        assert_eq!(
            archetype_primary_flags(MercArchetype::Vanguard),
            (true, true, false)
        );
        assert_eq!(
            archetype_primary_flags(MercArchetype::Scout),
            (false, true, true)
        );
        assert_eq!(
            archetype_primary_flags(MercArchetype::Arcanist),
            (false, false, true)
        );
        assert_eq!(
            archetype_primary_flags(MercArchetype::Medic),
            (false, true, true)
        );
        assert_eq!(
            archetype_primary_flags(MercArchetype::Saboteur),
            (false, false, true)
        );
    }

    // ── Starter Roster Quality Scaling ────────────────────────────────────────

    #[test]
    fn test_generate_starter_roster_quality_scales_with_rank() {
        let mut rng = seeded_rng(914);
        let mut id_counter = 0u64;
        let mut next_id = || {
            id_counter += 1;
            id_counter
        };

        let r1 = generate_starter_roster(GuildRank(1), &mut next_id, &mut rng);
        assert!(r1.iter().all(|m| m.quality == MercQuality::Common));

        let r2 = generate_starter_roster(GuildRank(2), &mut next_id, &mut rng);
        assert!(r2.iter().all(|m| m.quality == MercQuality::Common));

        let r3 = generate_starter_roster(GuildRank(3), &mut next_id, &mut rng);
        assert!(r3.iter().all(|m| m.quality == MercQuality::Uncommon));

        let r4 = generate_starter_roster(GuildRank(4), &mut next_id, &mut rng);
        assert!(r4.iter().all(|m| m.quality == MercQuality::Rare));

        let r5 = generate_starter_roster(GuildRank(5), &mut next_id, &mut rng);
        assert!(r5.iter().all(|m| m.quality == MercQuality::Rare));
    }

    // ── apply_merc_xp ─────────────────────────────────────────────────────────

    #[test]
    fn test_apply_merc_xp_gains_levels_and_scales_stats_up() {
        let mut rng = seeded_rng(1);
        let mut merc =
            generate_mercenary(1, MercArchetype::Vanguard, MercQuality::Common, &mut rng);
        let pre_power = merc.power;
        let gained = apply_merc_xp(&mut merc, 3);
        assert_eq!(gained, 3);
        assert_eq!(merc.level, 4);
        assert!(
            merc.power > pre_power,
            "Power should increase after leveling up (before {}, after {})",
            pre_power,
            merc.power
        );
    }

    #[test]
    fn test_apply_merc_xp_zero_levels_gained_is_noop() {
        let mut rng = seeded_rng(2);
        let mut merc = generate_mercenary(1, MercArchetype::Scout, MercQuality::Common, &mut rng);
        let pre_power = merc.power;
        let gained = apply_merc_xp(&mut merc, 0);
        assert_eq!(gained, 0);
        assert_eq!(merc.level, 1);
        assert_eq!(merc.power, pre_power);
    }

    #[test]
    fn test_apply_merc_xp_caps_at_max_level_20() {
        let mut rng = seeded_rng(3);
        let mut merc =
            generate_mercenary(1, MercArchetype::Arcanist, MercQuality::Common, &mut rng);
        let gained = apply_merc_xp(&mut merc, 25); // Try to gain more than the cap allows
        assert_eq!(gained, 19); // 1 -> 20 is 19 levels
        assert_eq!(merc.level, 20);

        // Once at level 20, further XP grants nothing.
        let gained_again = apply_merc_xp(&mut merc, 5);
        assert_eq!(gained_again, 0);
        assert_eq!(merc.level, 20);
    }

    // ── promotion_cost edge cases ─────────────────────────────────────────────

    #[test]
    fn test_promotion_cost_common_is_always_zero() {
        // Common is unreachable as a promotion target (min == max == 0).
        for id in [0u64, 1, 42, 999] {
            assert_eq!(promotion_cost(id, MercQuality::Common), 0);
        }
    }

    // ── promote_merc_by_id ────────────────────────────────────────────────────

    #[test]
    fn test_promote_merc_by_id_success() {
        use crate::deep::types::DeepPrestige;
        let mut rng = seeded_rng(1);
        let mut merc = generate_mercenary(10, MercArchetype::Medic, MercQuality::Common, &mut rng);
        merc.missions_completed = 5;
        let merc_name = merc.name.clone();

        let mut prestige = DeepPrestige {
            warband_marks: 1000,
            ..Default::default()
        };
        prestige.roster.insert(merc.id, merc);

        let result = promote_merc_by_id(10, &mut prestige, GuildRank(2));
        assert!(result.is_ok());
        let (name, quality_name, cost) = result.unwrap();
        assert_eq!(name, merc_name);
        assert_eq!(quality_name, "Uncommon");
        assert_eq!(cost, promotion_cost(10, MercQuality::Uncommon));
        assert_eq!(
            prestige.find_merc(10).unwrap().quality,
            MercQuality::Uncommon
        );
    }

    #[test]
    fn test_promote_merc_by_id_not_found() {
        use crate::deep::types::DeepPrestige;
        let mut prestige = DeepPrestige {
            warband_marks: 1000,
            ..Default::default()
        };
        let result = promote_merc_by_id(999, &mut prestige, GuildRank(2));
        assert!(result.is_err());
    }

    #[test]
    fn test_promote_merc_by_id_insufficient_marks() {
        use crate::deep::types::DeepPrestige;
        let mut rng = seeded_rng(1);
        let mut merc =
            generate_mercenary(11, MercArchetype::Vanguard, MercQuality::Common, &mut rng);
        merc.missions_completed = 5;

        let mut prestige = DeepPrestige {
            warband_marks: 0,
            ..Default::default()
        };
        prestige.roster.insert(merc.id, merc);

        let result = promote_merc_by_id(11, &mut prestige, GuildRank(2));
        assert!(matches!(
            result,
            Err(PromotionError::InsufficientMarks { .. })
        ));
        // Roster/marks should be unchanged on failure.
        assert_eq!(prestige.warband_marks, 0);
        assert_eq!(prestige.find_merc(11).unwrap().quality, MercQuality::Common);
    }

    // ── Injury Severity (Light) ───────────────────────────────────────────────

    #[test]
    fn test_injure_merc_light_severity() {
        let mut rng = seeded_rng(500);
        let now = Utc::now();
        let mut merc =
            generate_mercenary(1, MercArchetype::Saboteur, MercQuality::Common, &mut rng);
        injure_merc(&mut merc, InjurySeverity::Light, now, &mut rng);
        let MercStatus::Injured { recover_at } = merc.status else {
            panic!("Merc should be Injured");
        };
        let (min, max) = InjurySeverity::Light.recovery_secs();
        let secs = (recover_at - now).num_seconds();
        assert!(
            (min as i64..=max as i64).contains(&secs),
            "Light recovery {}s should be in [{}, {}]",
            secs,
            min,
            max
        );
    }

    #[test]
    fn test_promote_mercenary_uncommon_to_rare() {
        use crate::deep::types::DeepPrestige;
        let mut rng = seeded_rng(1);
        let mut merc = generate_mercenary(1, MercArchetype::Scout, MercQuality::Uncommon, &mut rng);
        merc.missions_completed = 10;
        let pre_power = merc.power;
        let pre_resilience = merc.resilience;
        let pre_expertise = merc.expertise;

        let mut prestige = DeepPrestige {
            warband_marks: 1000,
            ..Default::default()
        };

        let result = promote_mercenary(&mut merc, &mut prestige, GuildRank(3));
        assert!(result.is_ok());
        assert_eq!(merc.quality, MercQuality::Rare);

        // Scout primaries: resilience + expertise
        // Uncommon→Rare: flat_delta = 4-2 = 2, primary_delta = 2-2 = 0
        assert_eq!(merc.power, pre_power + 2); // flat only
        assert_eq!(merc.resilience, pre_resilience + 2); // flat only (primary delta is 0)
        assert_eq!(merc.expertise, pre_expertise + 2); // flat only (primary delta is 0)
    }
}
