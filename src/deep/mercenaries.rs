//! Mercenary generation, recruitment, and lifecycle management for The Deep.
//!
//! This module handles all mercenary operations: generating mercs with archetype-
//! appropriate stat distributions, building rosters and recruitment pools, and
//! managing the injury/recovery lifecycle.

use super::types::{GuildRank, MercArchetype, MercStatus, Mercenary, TheDeepState};
use rand::{Rng, RngExt};

// =========================================================================
// Name generation
// =========================================================================

const FIRST_NAMES: &[&str] = &[
    "Kael", "Lyra", "Thorne", "Rook", "Maren", "Yssa", "Torvik", "Drev", "Pell", "Kal", "Sable",
    "Fenwick", "Ashara", "Brin", "Corva", "Gavin", "Hestia", "Jax", "Naia", "Voss",
];

const SURNAMES: &[&str] = &[
    "Dawnwhisper",
    "Ashveil",
    "Ironbrow",
    "Coldmere",
    "Brightmantle",
    "Dusk",
    "Vorn",
    "Stormborn",
    "Deephollow",
    "Nightforge",
    "Oakenshield",
    "Ravencrest",
    "Silverbane",
    "Thornwick",
    "Windhaven",
];

/// Generate a procedural fantasy name for a mercenary.
pub fn generate_merc_name<R: Rng>(rng: &mut R) -> String {
    let first = FIRST_NAMES[rng.random_range(0..FIRST_NAMES.len())];
    let surname = SURNAMES[rng.random_range(0..SURNAMES.len())];
    format!("{} {}", first, surname)
}

// =========================================================================
// Stat ranges by guild rank
// =========================================================================

/// Returns the (min, max) base stat range for a given guild rank.
fn stat_range(guild_rank: GuildRank) -> (u32, u32) {
    match guild_rank {
        GuildRank::Freelancers => (15, 25),
        GuildRank::Sellswords => (20, 30),
        GuildRank::Company => (25, 40),
        GuildRank::Battalion => (35, 50),
        GuildRank::Legion => (45, 65),
    }
}

// =========================================================================
// Archetype stat distribution
// =========================================================================

/// Returns a (power_bias, resilience_bias) float pair.
///
/// A bias > 1.0 means the stat is scaled up from the rolled base.
/// A bias < 1.0 means it is scaled down.
/// Values are applied after both stats are independently rolled.
fn archetype_bias(archetype: MercArchetype) -> (f32, f32) {
    match archetype {
        // Vanguard: tanky — high resilience, average power
        MercArchetype::Vanguard => (0.85, 1.20),
        // Scout: balanced — equal in both
        MercArchetype::Scout => (1.00, 1.00),
        // Medic: support — lower direct stats, resilience slightly above average
        MercArchetype::Medic => (0.80, 1.10),
        // Saboteur: utility attacker — slightly higher power, average resilience
        MercArchetype::Saboteur => (1.15, 0.90),
        // Arcanist: glass cannon — higher power, lower resilience
        MercArchetype::Arcanist => (1.25, 0.80),
    }
}

// =========================================================================
// Core generation
// =========================================================================

/// Global counter seed for unique IDs — using a simple approach based on roster size.
/// IDs are assigned by the caller context; here we accept an `id` parameter.
pub fn generate_mercenary<R: Rng>(
    archetype: MercArchetype,
    guild_rank: GuildRank,
    id: u32,
    rng: &mut R,
) -> Mercenary {
    let (min, max) = stat_range(guild_rank);
    let (power_bias, resilience_bias) = archetype_bias(archetype);

    let base_power = rng.random_range(min..=max);
    let base_resilience = rng.random_range(min..=max);

    // Apply archetype bias, rounding to nearest u32 and clamping to minimum 1
    let power = ((base_power as f32 * power_bias).round() as u32).max(1);
    let resilience = ((base_resilience as f32 * resilience_bias).round() as u32).max(1);

    Mercenary {
        id,
        name: generate_merc_name(rng),
        archetype,
        level: 1,
        power,
        resilience,
        status: MercStatus::Ready,
        missions_completed: 0,
        injury_cooldown: 0,
    }
}

// =========================================================================
// Roster and pool generation
// =========================================================================

/// All five archetypes in a fixed order for guaranteed-diversity logic.
const ALL_ARCHETYPES: [MercArchetype; 5] = [
    MercArchetype::Vanguard,
    MercArchetype::Scout,
    MercArchetype::Medic,
    MercArchetype::Saboteur,
    MercArchetype::Arcanist,
];

/// Generate the starting roster for a new run.
///
/// Produces `min(5, guild_rank.roster_cap())` mercenaries. The first five slots
/// are guaranteed to contain one of each archetype; any additional slots (when
/// the cap exceeds 5) fill with random archetypes.
pub fn generate_starting_roster<R: Rng>(guild_rank: GuildRank, rng: &mut R) -> Vec<Mercenary> {
    let count = guild_rank.roster_cap().min(5);
    let mut roster = Vec::with_capacity(count);

    // One of each archetype for the first 5
    for (i, &archetype) in ALL_ARCHETYPES.iter().take(count).enumerate() {
        let merc = generate_mercenary(archetype, guild_rank, (i + 1) as u32, rng);
        roster.push(merc);
    }

    roster
}

/// Generate a recruitment pool of 3-4 random candidates.
pub fn generate_recruitment_pool<R: Rng>(guild_rank: GuildRank, rng: &mut R) -> Vec<Mercenary> {
    let count = rng.random_range(3..=4);
    let mut pool = Vec::with_capacity(count);

    for i in 0..count {
        let archetype = ALL_ARCHETYPES[rng.random_range(0..ALL_ARCHETYPES.len())];
        // IDs in pool use a high offset to avoid collision with roster IDs
        let merc = generate_mercenary(archetype, guild_rank, (100 + i) as u32, rng);
        pool.push(merc);
    }

    pool
}

// =========================================================================
// Recruitment
// =========================================================================

/// Cost in Warband Marks to recruit a mercenary from the pool.
pub const RECRUITMENT_COST: u32 = 40;

/// Recruit a mercenary from the pool into the active roster.
///
/// Validates that:
/// - `pool_index` is valid
/// - The roster is not already at the guild rank cap
/// - The player has enough Warband Marks
///
/// On success, deducts the cost and moves the merc from pool to roster.
pub fn recruit_mercenary(state: &mut TheDeepState, pool_index: usize) -> Result<(), &'static str> {
    if pool_index >= state.run.recruitment_pool.len() {
        return Err("invalid pool index");
    }
    let cap = state.account.guild_rank.roster_cap();
    if state.run.mercenaries.len() >= cap {
        return Err("roster is full");
    }
    if state.run.warband_marks < RECRUITMENT_COST {
        return Err("not enough Warband Marks");
    }

    state.run.warband_marks -= RECRUITMENT_COST;
    let merc = state.run.recruitment_pool.remove(pool_index);
    state.run.mercenaries.push(merc);
    Ok(())
}

// =========================================================================
// Injury and recovery
// =========================================================================

/// Set a mercenary's injury cooldown, marking them as Injured.
///
/// `missions_remaining` is the number of future missions the merc must sit out.
pub fn injure_mercenary(merc: &mut Mercenary, missions_remaining: u8) {
    merc.status = MercStatus::Injured;
    merc.injury_cooldown = missions_remaining;
}

/// Decrement injury cooldowns for all mercs after a mission completes.
///
/// Mercs whose cooldown reaches 0 become Ready again.
pub fn recover_mercenaries(state: &mut TheDeepState) {
    for merc in &mut state.run.mercenaries {
        if merc.status == MercStatus::Injured && merc.injury_cooldown > 0 {
            merc.injury_cooldown -= 1;
            if merc.injury_cooldown == 0 {
                merc.status = MercStatus::Ready;
            }
        }
    }
}

// =========================================================================
// Medic downgrade
// =========================================================================

/// Downgrade injury severity if a Medic is present in the squad.
///
/// Severity levels (caller-defined convention):
/// - 0 = scratch (no effect)
/// - 1 = injury (merc out for 1-2 missions)
/// - 2 = loss (merc permanently gone)
///
/// With a Medic: loss(2) → injury(1), injury(1) → scratch(0), scratch stays 0.
pub fn apply_medic_downgrade(mercs: &[&Mercenary], injury_severity: u8) -> u8 {
    let has_medic = mercs
        .iter()
        .any(|m| m.archetype == MercArchetype::Medic && m.status != MercStatus::Lost);

    if has_medic && injury_severity > 0 {
        injury_severity - 1
    } else {
        injury_severity
    }
}
