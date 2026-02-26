// Allow dead_code: economy functions are wired into the game loop incrementally.
#![allow(dead_code)]
//! Warband Marks economy, guild rank costs, mission costs, and reward calculations.
//!
//! This module provides:
//! - Mark earning rates per mission type and layer
//! - Mark spending costs (missions, recruitment, infrastructure, guild upgrades)
//! - Guild rank upgrade eligibility checks
//! - Income modifiers (Supply Cache, Familiarity bonuses, outcome multipliers)
//! - Reward calculations for XP, Stormglass, and prestige rank fragments

use crate::deep::layers::FamiliarityLevel;
use crate::deep::types::{DeepPersistent, DeepPrestige, GuildRank, MissionOutcome, MissionType};

// ── Mark Earning Rates ────────────────────────────────────────────────────────

/// Base Warband Marks earned from a mission on a given 1-based layer, before
/// any modifiers (Supply Cache, familiarity, outcome scaling).
///
/// Values from the balance design §3 table. Void layers (26+) scale linearly.
pub fn base_marks_earned(mission_type: MissionType, layer: u32) -> u32 {
    let layer = layer.max(1);

    if layer >= 26 {
        let extra = layer - 25;
        let (base_26, per_layer) = match mission_type {
            MissionType::SupplyRun => (210, 10u32),
            MissionType::Recon => (295, 15),
            MissionType::Expedition => (730, 35),
            MissionType::Breakthrough | MissionType::GatewayExpedition => (1125, 50),
            MissionType::Construction(_) => (0, 0),
        };
        return base_26 + per_layer * extra;
    }

    // Lookup table for layers 1–25 from the balance design document.
    let (supply, recon, expedition, breakthrough) = match layer {
        1 => (35, 50, 130, 280),
        2 => (40, 55, 140, 300),
        3 => (40, 60, 155, 320),
        4 => (45, 65, 170, 345),
        5 => (50, 70, 185, 370),
        6 => (55, 80, 200, 395),
        7 => (60, 85, 215, 420),
        8 => (65, 95, 235, 450),
        9 => (70, 100, 255, 480),
        10 => (75, 110, 275, 510),
        11 => (80, 115, 295, 540),
        12 => (90, 125, 315, 570),
        13 => (95, 135, 340, 600),
        14 => (100, 145, 365, 635),
        15 => (110, 155, 390, 670),
        16 => (115, 165, 415, 705),
        17 => (125, 175, 440, 740),
        18 => (130, 185, 470, 780),
        19 => (140, 200, 500, 820),
        20 => (150, 210, 530, 860),
        21 => (160, 225, 560, 900),
        22 => (170, 235, 590, 940),
        23 => (180, 250, 625, 985),
        24 => (190, 265, 660, 1030),
        25 => (200, 280, 695, 1075),
        _ => unreachable!(), // handled above
    };

    match mission_type {
        MissionType::SupplyRun => supply,
        MissionType::Recon => recon,
        MissionType::Expedition => expedition,
        MissionType::Breakthrough | MissionType::GatewayExpedition => breakthrough,
        MissionType::Construction(_) => 0, // no Mark reward for construction
    }
}

/// Reward variance factor: actual rewards vary by ±15%.
/// The seed factor should be a value in [0.0, 1.0] derived from seeded RNG;
/// this function maps it to the [0.85, 1.15] multiplier range.
pub fn marks_variance_multiplier(rng_factor: f64) -> f64 {
    0.85 + rng_factor * 0.30
}

/// Outcome multiplier applied to Mark rewards based on mission success level.
pub fn outcome_mark_multiplier(outcome: MissionOutcome) -> f64 {
    match outcome {
        MissionOutcome::Success => 1.0,
        MissionOutcome::PartialSuccess => 0.60,
        MissionOutcome::Failure => 0.20,
    }
}

/// Parameters for computing the final Mark reward for a completed mission.
pub struct MarkRewardParams {
    pub mission_type: MissionType,
    pub layer: u32,
    /// Raw familiarity percentage (0–100) of this layer.
    pub familiarity: u8,
    /// Whether a Supply Cache is built on this layer.
    pub has_supply_cache: bool,
    pub outcome: MissionOutcome,
    /// Random variance factor in [0.0, 1.0] from seeded RNG.
    pub rng_variance: f64,
}

/// Compute the final Warband Marks reward for a completed mission.
///
/// Applies (in order): base rate → Supply Cache → familiarity bonus → outcome
/// scaling → variance. The result is rounded to the nearest integer.
pub fn compute_mark_reward(params: &MarkRewardParams) -> u32 {
    let base = base_marks_earned(params.mission_type, params.layer) as f64;

    // Supply Cache only applies to Supply Runs.
    let after_cache =
        if params.has_supply_cache && matches!(params.mission_type, MissionType::SupplyRun) {
            base * 1.50 // +50%
        } else {
            base
        };

    // Mastered familiarity grants +15% on any mission type.
    let fam = FamiliarityLevel::from_familiarity(params.familiarity);
    let after_fam = after_cache * fam.mark_yield_multiplier();

    // Scale by mission outcome.
    let after_outcome = after_fam * outcome_mark_multiplier(params.outcome.clone());

    // Apply ±15% variance.
    let variance = marks_variance_multiplier(params.rng_variance);

    (after_outcome * variance).round() as u32
}

// ── Mission Launch Costs ──────────────────────────────────────────────────────

/// Warband Marks cost to launch a mission on a given 1-based layer.
///
/// One free daily Supply Run is handled by the caller (e.g., `DeepPrestige`
/// tracks whether the daily free run has been used). This function always
/// returns the paid cost; callers skip the deduction when the free run applies.
///
/// From balance design §3 spending sinks.
pub fn mission_launch_cost(mission_type: MissionType, layer: u32) -> u32 {
    let layer = layer.max(1);
    match mission_type {
        MissionType::SupplyRun => {
            // 15-25 Marks when not using the free daily slot.
            // Use the midpoint (20) for deterministic cost; variance handled
            // at the supply-run-specific level if desired.
            20
        }
        MissionType::Recon => 30 + layer,
        MissionType::Expedition => 80 + 3 * layer,
        MissionType::Breakthrough | MissionType::GatewayExpedition => 70 + 8 * layer,
        MissionType::Construction(infra) => super::layers::infrastructure_build_cost(infra, layer),
    }
}

// ── Recruitment Costs ─────────────────────────────────────────────────────────

/// Recruit quality tier, corresponding to the pool distribution at each guild rank.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecruitQuality {
    Common,
    Uncommon,
    Rare,
    Elite,
}

impl RecruitQuality {
    /// Warband Marks cost range (min, max) to recruit a candidate of this quality.
    pub fn cost_range(self) -> (u32, u32) {
        match self {
            RecruitQuality::Common => (50, 80),
            RecruitQuality::Uncommon => (80, 130),
            RecruitQuality::Rare => (130, 200),
            RecruitQuality::Elite => (200, 300),
        }
    }

    /// Deterministic cost for a recruit of this quality given a [0.0, 1.0] rng factor.
    pub fn cost(self, rng_factor: f64) -> u32 {
        let (min, max) = self.cost_range();
        let range = (max - min) as f64;
        (min as f64 + rng_factor * range).round() as u32
    }

    /// Base stat bonus applied on top of archetype base stats at Level 1.
    /// Returns (power_bonus, resilience_bonus, expertise_bonus).
    pub fn stat_bonus(self) -> (u32, u32, u32) {
        match self {
            RecruitQuality::Common => (0, 0, 0),
            RecruitQuality::Uncommon => (2, 2, 2),
            RecruitQuality::Rare => (4, 4, 4),
            RecruitQuality::Elite => (8, 8, 8),
        }
    }

    /// Additional bonus to the archetype's primary stats for rare/elite quality.
    /// Primary-stat bonus is added on top of `stat_bonus`.
    pub fn primary_stat_bonus(self) -> u32 {
        match self {
            RecruitQuality::Common => 0,
            RecruitQuality::Uncommon => 2,
            RecruitQuality::Rare => 4,
            RecruitQuality::Elite => 8,
        }
    }
}

/// Probability distribution of recruit qualities at each guild rank.
///
/// Returns a weighted list of (quality, weight) pairs.  The caller uses these
/// weights with a seeded RNG to select a quality tier.
pub fn recruit_quality_distribution(rank: GuildRank) -> &'static [(RecruitQuality, u32)] {
    // Weights do not need to sum to 100; they are relative.
    match rank.0 {
        1 => &[(RecruitQuality::Common, 100)],
        2 => &[(RecruitQuality::Common, 60), (RecruitQuality::Uncommon, 40)],
        3 => &[
            (RecruitQuality::Common, 30),
            (RecruitQuality::Uncommon, 50),
            (RecruitQuality::Rare, 20),
        ],
        4 => &[
            (RecruitQuality::Uncommon, 40),
            (RecruitQuality::Rare, 50),
            (RecruitQuality::Elite, 10),
        ],
        5 => &[
            (RecruitQuality::Uncommon, 20),
            (RecruitQuality::Rare, 50),
            (RecruitQuality::Elite, 30),
        ],
        _ => &[(RecruitQuality::Common, 100)],
    }
}

// ── Guild Rank Economy ────────────────────────────────────────────────────────

/// Warband Marks cost to upgrade from the given guild rank to the next.
///
/// Returns `None` if already at max rank.
/// Costs from balance design §7.
pub fn guild_upgrade_cost(current_rank: GuildRank) -> Option<u32> {
    match current_rank.0 {
        1 => Some(200),
        2 => Some(500),
        3 => Some(1200),
        4 => Some(3000),
        _ => None, // rank 5 is the max
    }
}

/// Error variants returned when a guild rank upgrade is attempted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuildUpgradeError {
    /// Already at the maximum rank.
    AlreadyMaxRank,
    /// Layer breakthrough requirement not met.
    LayerRequirementNotMet { required_layer: u32 },
    /// Insufficient Warband Marks.
    InsufficientMarks { required: u32, available: u32 },
}

/// Check whether the player can upgrade their guild rank, and if so, perform
/// the upgrade by spending Marks from `prestige`.
///
/// The upgrade is atomic: marks are only deducted if all checks pass.
pub fn try_upgrade_guild_rank(
    persistent: &mut DeepPersistent,
    prestige: &mut DeepPrestige,
) -> Result<GuildRank, GuildUpgradeError> {
    let current = persistent.guild_rank;

    // 1. Check max rank.
    let next = current.next().ok_or(GuildUpgradeError::AlreadyMaxRank)?;

    // 2. Check layer breakthrough requirement.
    if let Some(required_layer) = next.required_breakthrough_layer() {
        let cleared = persistent
            .layer_record(required_layer)
            .map(|r| r.cleared)
            .unwrap_or(false);
        if !cleared {
            return Err(GuildUpgradeError::LayerRequirementNotMet { required_layer });
        }
    }

    // 3. Check and spend marks.
    let cost = guild_upgrade_cost(current).expect("next rank exists so cost must be Some");
    if prestige.warband_marks < cost {
        return Err(GuildUpgradeError::InsufficientMarks {
            required: cost,
            available: prestige.warband_marks,
        });
    }
    prestige.warband_marks -= cost;

    // 4. Apply upgrade.
    persistent.guild_rank = next;

    Ok(next)
}

// ── XP Rewards ───────────────────────────────────────────────────────────────

/// Character XP awarded when a mission completes (feeds into the main XP pool).
///
/// From balance design §9. These are base values before outcome scaling.
pub fn base_xp_reward(mission_type: MissionType, layer: u32) -> u32 {
    let layer = layer.max(1);
    match mission_type {
        MissionType::SupplyRun => 150 + 20 * layer,
        MissionType::Recon => 300 + 35 * layer,
        MissionType::Expedition => 600 + 60 * layer,
        MissionType::Breakthrough | MissionType::GatewayExpedition => 1200 + 120 * layer,
        MissionType::Construction(_) => 0,
    }
}

/// Scale base XP by outcome.
pub fn xp_reward(mission_type: MissionType, layer: u32, outcome: MissionOutcome) -> u32 {
    let base = base_xp_reward(mission_type, layer) as f64;
    let scaled = base * outcome_mark_multiplier(outcome); // reuse same outcome ratios
    scaled.round() as u32
}

// ── Stormglass Rewards ────────────────────────────────────────────────────────

/// Stormglass earned from a completed mission.
///
/// Only Expeditions and Breakthroughs earn Stormglass.
/// From balance design §9.
pub fn stormglass_reward(mission_type: MissionType, layer: u32) -> u64 {
    let layer = layer.max(1) as u64;
    match mission_type {
        MissionType::Expedition => 5 + layer / 3,
        MissionType::Breakthrough => 10 + layer / 2,
        _ => 0,
    }
}

// ── Prestige Rank Fragments ───────────────────────────────────────────────────

/// Prestige rank fragment (in hundredths of a rank) awarded from a Breakthrough.
///
/// Accumulates across Breakthroughs; callers track the running total and award
/// whole prestige ranks when the total reaches 100.
///
/// From balance design §9:
/// - Layers 1-7:  0 PR
/// - Layers 8-12: 0.25 PR  (25 hundredths)
/// - Layers 13-18: 0.50 PR (50 hundredths)
/// - Layers 19-25: 0.75 PR (75 hundredths)
/// - Layers 26+:  1.00 PR  (100 hundredths)
pub fn prestige_fragment_hundredths(mission_type: MissionType, layer: u32) -> u32 {
    if !matches!(mission_type, MissionType::Breakthrough) {
        return 0;
    }
    match layer {
        1..=7 => 0,
        8..=12 => 25,
        13..=18 => 50,
        19..=25 => 75,
        _ => 100,
    }
}

// ── Merc XP (per-merc leveling) ───────────────────────────────────────────────

/// XP gained by each squad member from completing a mission.
///
/// From balance design §4.
pub fn merc_xp_per_mission(mission_type: MissionType, layer: u32) -> u32 {
    let layer = layer.max(1);
    match mission_type {
        MissionType::SupplyRun => 100 + 10 * layer,
        MissionType::Recon => 200 + 20 * layer,
        MissionType::Expedition => 400 + 40 * layer,
        MissionType::Breakthrough | MissionType::GatewayExpedition => 800 + 80 * layer,
        MissionType::Construction(_) => 50,
    }
}

/// XP required for a merc to advance from `level` to `level + 1`.
///
/// Formula: `200 * level^1.3` from balance design §4.
pub fn merc_xp_to_next_level(level: u32) -> u32 {
    if level == 0 {
        return 200;
    }
    let base = (level as f64).powf(1.3);
    (200.0 * base).round() as u32
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deep::types::{DeepPersistent, DeepPrestige, Infrastructure};

    // ── base_marks_earned ─────────────────────────────────────────────────────

    #[test]
    fn test_base_marks_layer_1() {
        assert_eq!(base_marks_earned(MissionType::SupplyRun, 1), 35);
        assert_eq!(base_marks_earned(MissionType::Recon, 1), 50);
        assert_eq!(base_marks_earned(MissionType::Expedition, 1), 130);
        assert_eq!(base_marks_earned(MissionType::Breakthrough, 1), 280);
    }

    #[test]
    fn test_base_marks_layer_25() {
        assert_eq!(base_marks_earned(MissionType::SupplyRun, 25), 200);
        assert_eq!(base_marks_earned(MissionType::Recon, 25), 280);
        assert_eq!(base_marks_earned(MissionType::Expedition, 25), 695);
        assert_eq!(base_marks_earned(MissionType::Breakthrough, 25), 1075);
    }

    #[test]
    fn test_base_marks_void_layer_26() {
        // Void: base_26 + per_layer * (26-25)
        assert_eq!(base_marks_earned(MissionType::SupplyRun, 26), 210 + 10);
        assert_eq!(base_marks_earned(MissionType::Recon, 26), 295 + 15);
        assert_eq!(base_marks_earned(MissionType::Expedition, 26), 730 + 35);
        assert_eq!(base_marks_earned(MissionType::Breakthrough, 26), 1125 + 50);
    }

    #[test]
    fn test_construction_earns_no_marks() {
        assert_eq!(
            base_marks_earned(MissionType::Construction(Infrastructure::Outpost), 5),
            0
        );
    }

    #[test]
    fn test_marks_increase_with_depth() {
        for layer in 1..25 {
            let a = base_marks_earned(MissionType::Expedition, layer);
            let b = base_marks_earned(MissionType::Expedition, layer + 1);
            assert!(
                b >= a,
                "Expedition marks should not decrease layer {} -> {}",
                layer,
                layer + 1
            );
        }
    }

    // ── outcome_mark_multiplier ───────────────────────────────────────────────

    #[test]
    fn test_outcome_mark_multipliers() {
        assert_eq!(outcome_mark_multiplier(MissionOutcome::Success), 1.0);
        assert_eq!(
            outcome_mark_multiplier(MissionOutcome::PartialSuccess),
            0.60
        );
        assert_eq!(outcome_mark_multiplier(MissionOutcome::Failure), 0.20);
    }

    // ── compute_mark_reward ───────────────────────────────────────────────────

    #[test]
    fn test_compute_mark_reward_full_success_no_modifiers() {
        let params = MarkRewardParams {
            mission_type: MissionType::SupplyRun,
            layer: 1,
            familiarity: 0,
            has_supply_cache: false,
            outcome: MissionOutcome::Success,
            rng_variance: 0.5, // midpoint = 1.0 multiplier
        };
        let base = 35.0_f64;
        let expected = (base * 1.0_f64).round() as u32; // variance at midpoint
        let result = compute_mark_reward(&params);
        // Allow ±1 due to float rounding.
        assert!((result as i32 - expected as i32).abs() <= 1);
    }

    #[test]
    fn test_compute_mark_reward_supply_cache_bonus() {
        let params = MarkRewardParams {
            mission_type: MissionType::SupplyRun,
            layer: 1,
            familiarity: 0,
            has_supply_cache: true,
            outcome: MissionOutcome::Success,
            rng_variance: 0.5,
        };
        let no_cache_params = MarkRewardParams {
            has_supply_cache: false,
            ..MarkRewardParams {
                mission_type: MissionType::SupplyRun,
                layer: 1,
                familiarity: 0,
                has_supply_cache: false,
                outcome: MissionOutcome::Success,
                rng_variance: 0.5,
            }
        };
        let with_cache = compute_mark_reward(&params);
        let without_cache = compute_mark_reward(&no_cache_params);
        assert!(
            with_cache > without_cache,
            "Supply Cache should increase marks"
        );
    }

    #[test]
    fn test_compute_mark_reward_supply_cache_does_not_apply_to_expedition() {
        let with_cache = compute_mark_reward(&MarkRewardParams {
            mission_type: MissionType::Expedition,
            layer: 1,
            familiarity: 0,
            has_supply_cache: true,
            outcome: MissionOutcome::Success,
            rng_variance: 0.5,
        });
        let without_cache = compute_mark_reward(&MarkRewardParams {
            mission_type: MissionType::Expedition,
            layer: 1,
            familiarity: 0,
            has_supply_cache: false,
            outcome: MissionOutcome::Success,
            rng_variance: 0.5,
        });
        assert_eq!(
            with_cache, without_cache,
            "Supply Cache should not boost Expeditions"
        );
    }

    #[test]
    fn test_compute_mark_reward_mastered_familiarity_bonus() {
        let mastered = compute_mark_reward(&MarkRewardParams {
            mission_type: MissionType::Expedition,
            layer: 5,
            familiarity: 80, // Mastered
            has_supply_cache: false,
            outcome: MissionOutcome::Success,
            rng_variance: 0.5,
        });
        let unknown = compute_mark_reward(&MarkRewardParams {
            mission_type: MissionType::Expedition,
            layer: 5,
            familiarity: 0, // Unknown
            has_supply_cache: false,
            outcome: MissionOutcome::Success,
            rng_variance: 0.5,
        });
        assert!(mastered > unknown, "Mastered should grant more marks");
    }

    #[test]
    fn test_compute_mark_reward_failure_reduces_to_20_percent() {
        let success = compute_mark_reward(&MarkRewardParams {
            mission_type: MissionType::Expedition,
            layer: 1,
            familiarity: 0,
            has_supply_cache: false,
            outcome: MissionOutcome::Success,
            rng_variance: 0.5,
        });
        let failure = compute_mark_reward(&MarkRewardParams {
            mission_type: MissionType::Expedition,
            layer: 1,
            familiarity: 0,
            has_supply_cache: false,
            outcome: MissionOutcome::Failure,
            rng_variance: 0.5,
        });
        // Failure should be ~20% of success.
        let ratio = failure as f64 / success as f64;
        assert!(
            (ratio - 0.20).abs() < 0.05,
            "Failure marks ratio was {}",
            ratio
        );
    }

    // ── mission_launch_cost ───────────────────────────────────────────────────

    #[test]
    fn test_mission_launch_costs_are_positive_except_construction() {
        for mission in [
            MissionType::SupplyRun,
            MissionType::Recon,
            MissionType::Expedition,
            MissionType::Breakthrough,
        ] {
            assert!(mission_launch_cost(mission, 1) > 0);
        }
        // Construction missions cost the infrastructure build cost.
        // Bridge on layer 1 = 140 + 7*1 = 147
        assert_eq!(
            mission_launch_cost(MissionType::Construction(Infrastructure::Bridge), 1),
            147
        );
    }

    #[test]
    fn test_mission_launch_costs_scale_with_depth() {
        assert!(
            mission_launch_cost(MissionType::Recon, 10)
                > mission_launch_cost(MissionType::Recon, 1)
        );
        assert!(
            mission_launch_cost(MissionType::Expedition, 10)
                > mission_launch_cost(MissionType::Expedition, 1)
        );
        assert!(
            mission_launch_cost(MissionType::Breakthrough, 10)
                > mission_launch_cost(MissionType::Breakthrough, 1)
        );
    }

    // ── guild_upgrade_cost ────────────────────────────────────────────────────

    #[test]
    fn test_guild_upgrade_costs() {
        assert_eq!(guild_upgrade_cost(GuildRank(1)), Some(200));
        assert_eq!(guild_upgrade_cost(GuildRank(2)), Some(500));
        assert_eq!(guild_upgrade_cost(GuildRank(3)), Some(1200));
        assert_eq!(guild_upgrade_cost(GuildRank(4)), Some(3000));
        assert_eq!(guild_upgrade_cost(GuildRank(5)), None);
    }

    // ── try_upgrade_guild_rank ────────────────────────────────────────────────

    fn persistent_with_rank(rank: u8) -> DeepPersistent {
        let mut p = DeepPersistent::new();
        p.guild_rank = GuildRank(rank);
        p
    }

    #[test]
    fn test_upgrade_guild_rank_success() {
        let mut persistent = persistent_with_rank(1);
        // Clear layer 3 (required for rank 2).
        persistent.layer_record_mut(3).cleared = true;
        let mut prestige = DeepPrestige::new();
        prestige.warband_marks = 500;

        let result = try_upgrade_guild_rank(&mut persistent, &mut prestige);
        assert_eq!(result, Ok(GuildRank(2)));
        assert_eq!(persistent.guild_rank, GuildRank(2));
        assert_eq!(prestige.warband_marks, 300); // 500 - 200
    }

    #[test]
    fn test_upgrade_guild_rank_fails_at_max() {
        let mut persistent = persistent_with_rank(5);
        let mut prestige = DeepPrestige::new();
        prestige.warband_marks = 99999;

        let result = try_upgrade_guild_rank(&mut persistent, &mut prestige);
        assert_eq!(result, Err(GuildUpgradeError::AlreadyMaxRank));
    }

    #[test]
    fn test_upgrade_guild_rank_fails_without_layer_cleared() {
        let mut persistent = persistent_with_rank(1);
        // Layer 3 NOT cleared.
        let mut prestige = DeepPrestige::new();
        prestige.warband_marks = 500;

        let result = try_upgrade_guild_rank(&mut persistent, &mut prestige);
        assert_eq!(
            result,
            Err(GuildUpgradeError::LayerRequirementNotMet { required_layer: 3 })
        );
        // Marks should be unchanged.
        assert_eq!(prestige.warband_marks, 500);
    }

    #[test]
    fn test_upgrade_guild_rank_fails_with_insufficient_marks() {
        let mut persistent = persistent_with_rank(1);
        persistent.layer_record_mut(3).cleared = true;
        let mut prestige = DeepPrestige::new();
        prestige.warband_marks = 100; // Need 200

        let result = try_upgrade_guild_rank(&mut persistent, &mut prestige);
        assert_eq!(
            result,
            Err(GuildUpgradeError::InsufficientMarks {
                required: 200,
                available: 100,
            })
        );
        // Marks should be unchanged.
        assert_eq!(prestige.warband_marks, 100);
        // Rank should be unchanged.
        assert_eq!(persistent.guild_rank, GuildRank(1));
    }

    #[test]
    fn test_upgrade_guild_rank_marks_not_deducted_on_failure() {
        let mut persistent = persistent_with_rank(2);
        // Layer 7 required for rank 3 — not cleared.
        let mut prestige = DeepPrestige::new();
        prestige.warband_marks = 1000;

        let _ = try_upgrade_guild_rank(&mut persistent, &mut prestige);
        assert_eq!(
            prestige.warband_marks, 1000,
            "Marks should not be deducted on failure"
        );
    }

    // ── recruit_quality_distribution ─────────────────────────────────────────

    #[test]
    fn test_recruit_quality_distribution_rank_1_only_common() {
        let dist = recruit_quality_distribution(GuildRank(1));
        assert_eq!(dist.len(), 1);
        assert_eq!(dist[0].0, RecruitQuality::Common);
    }

    #[test]
    fn test_recruit_quality_distribution_rank_5_includes_elite() {
        let dist = recruit_quality_distribution(GuildRank(5));
        assert!(
            dist.iter().any(|(q, _)| *q == RecruitQuality::Elite),
            "Rank 5 pool should include Elite quality"
        );
    }

    // ── XP and Stormglass Rewards ─────────────────────────────────────────────

    #[test]
    fn test_base_xp_reward_layer_1() {
        assert_eq!(base_xp_reward(MissionType::SupplyRun, 1), 170); // 150 + 20
        assert_eq!(base_xp_reward(MissionType::Expedition, 1), 660); // 600 + 60
        assert_eq!(base_xp_reward(MissionType::Breakthrough, 1), 1320); // 1200 + 120
    }

    #[test]
    fn test_base_xp_reward_layer_10() {
        assert_eq!(base_xp_reward(MissionType::SupplyRun, 10), 350); // 150 + 200
        assert_eq!(base_xp_reward(MissionType::Expedition, 10), 1200); // 600 + 600
    }

    #[test]
    fn test_xp_reward_scales_with_outcome() {
        let success = xp_reward(MissionType::Expedition, 5, MissionOutcome::Success);
        let partial = xp_reward(MissionType::Expedition, 5, MissionOutcome::PartialSuccess);
        let failure = xp_reward(MissionType::Expedition, 5, MissionOutcome::Failure);
        assert!(success > partial);
        assert!(partial > failure);
    }

    #[test]
    fn test_stormglass_reward_only_from_expeditions_and_breakthroughs() {
        assert_eq!(stormglass_reward(MissionType::SupplyRun, 10), 0);
        assert_eq!(stormglass_reward(MissionType::Recon, 10), 0);
        assert_eq!(
            stormglass_reward(MissionType::Construction(Infrastructure::Bridge), 10),
            0
        );
        assert!(stormglass_reward(MissionType::Expedition, 1) > 0);
        assert!(stormglass_reward(MissionType::Breakthrough, 1) > 0);
    }

    #[test]
    fn test_stormglass_reward_scales_with_depth() {
        let l1 = stormglass_reward(MissionType::Expedition, 1);
        let l10 = stormglass_reward(MissionType::Expedition, 10);
        let l20 = stormglass_reward(MissionType::Expedition, 20);
        assert!(l10 >= l1);
        assert!(l20 >= l10);
    }

    // ── Prestige Fragments ────────────────────────────────────────────────────

    #[test]
    fn test_prestige_fragment_only_from_breakthrough() {
        assert_eq!(prestige_fragment_hundredths(MissionType::SupplyRun, 10), 0);
        assert_eq!(prestige_fragment_hundredths(MissionType::Expedition, 10), 0);
    }

    #[test]
    fn test_prestige_fragment_thresholds() {
        // Layers 1-7: no fragment.
        assert_eq!(
            prestige_fragment_hundredths(MissionType::Breakthrough, 7),
            0
        );
        // Layers 8-12: 25 hundredths (0.25 PR).
        assert_eq!(
            prestige_fragment_hundredths(MissionType::Breakthrough, 8),
            25
        );
        assert_eq!(
            prestige_fragment_hundredths(MissionType::Breakthrough, 12),
            25
        );
        // Layers 13-18: 50 hundredths (0.50 PR).
        assert_eq!(
            prestige_fragment_hundredths(MissionType::Breakthrough, 13),
            50
        );
        assert_eq!(
            prestige_fragment_hundredths(MissionType::Breakthrough, 18),
            50
        );
        // Layers 19-25: 75 hundredths.
        assert_eq!(
            prestige_fragment_hundredths(MissionType::Breakthrough, 19),
            75
        );
        assert_eq!(
            prestige_fragment_hundredths(MissionType::Breakthrough, 25),
            75
        );
        // Void (26+): 100 hundredths (1 full PR).
        assert_eq!(
            prestige_fragment_hundredths(MissionType::Breakthrough, 26),
            100
        );
        assert_eq!(
            prestige_fragment_hundredths(MissionType::Breakthrough, 50),
            100
        );
    }

    // ── Merc XP ───────────────────────────────────────────────────────────────

    #[test]
    fn test_merc_xp_per_mission_layer_1() {
        assert_eq!(merc_xp_per_mission(MissionType::SupplyRun, 1), 110);
        assert_eq!(merc_xp_per_mission(MissionType::Recon, 1), 220);
        assert_eq!(merc_xp_per_mission(MissionType::Expedition, 1), 440);
        assert_eq!(merc_xp_per_mission(MissionType::Breakthrough, 1), 880);
        assert_eq!(
            merc_xp_per_mission(MissionType::Construction(Infrastructure::Watchtower), 5),
            50
        );
    }

    #[test]
    fn test_merc_xp_to_next_level_increases() {
        let mut prev = 0u32;
        for level in 1..=20 {
            let xp = merc_xp_to_next_level(level);
            assert!(
                xp > prev,
                "XP to level {} should exceed XP to level {}",
                level + 1,
                level
            );
            prev = xp;
        }
    }

    #[test]
    fn test_merc_xp_to_next_level_formula() {
        // Level 1: 200 * 1^1.3 = 200
        assert_eq!(merc_xp_to_next_level(1), 200);
        // Level 2: 200 * 2^1.3 ≈ 200 * 2.462 ≈ 492
        let l2 = merc_xp_to_next_level(2);
        assert!((l2 as i32 - 492).abs() <= 2, "Level 2 XP was {}", l2);
    }
}
