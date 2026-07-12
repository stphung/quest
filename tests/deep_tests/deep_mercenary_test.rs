//! Integration tests for The Deep — Mercenary System.
//!
//! Covers:
//! - MercQuality bonuses and cost ranges
//! - Archetype base stats and distributions
//! - Recruit pool generation (size, quality distribution by rank, archetype gates)
//! - Starter roster composition and invariants
//! - Stats at level scaling formula
//! - XP curve shape
//! - apply_merc_xp level application and stat proportional scaling
//! - Roster capacity by guild rank
//! - available_mercs filtering
//! - Injury system (injure_merc, check_injury_recovery, severities)
//! - Mark lost and purge pipeline
//! - roll_recruit_cost rounding and ordering by quality
//! - generate_merc_name format
//! - DeepState::on_prestige() — resets prestige, preserves persistent

use chrono::Utc;
use quest::deep::{
    apply_merc_xp, available_mercs, check_injury_recovery, generate_merc_name, generate_mercenary,
    generate_recruit_pool, generate_starter_roster, injure_merc, mark_merc_lost, purge_lost_mercs,
    recruit_pool_size, roll_recruit_cost, roll_recruit_quality, roster_has_capacity,
    stats_at_level, xp_to_next_level, DeepPrestige, DeepState, GuildRank, InjurySeverity,
    MercArchetype, MercQuality, MercStatus, Mercenary,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

fn seeded_rng(seed: u64) -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(seed)
}

fn id_counter() -> impl FnMut() -> u64 {
    let mut n = 0u64;
    move || {
        n += 1;
        n
    }
}

// =============================================================================
// MercQuality — flat_bonus, primary_bonus, cost_range
// =============================================================================

#[test]
fn test_merc_quality_flat_bonus_ordering() {
    assert_eq!(MercQuality::Common.flat_bonus(), 0);
    assert!(MercQuality::Uncommon.flat_bonus() > MercQuality::Common.flat_bonus());
    assert!(MercQuality::Rare.flat_bonus() > MercQuality::Uncommon.flat_bonus());
    assert!(MercQuality::Elite.flat_bonus() > MercQuality::Rare.flat_bonus());
}

#[test]
fn test_merc_quality_primary_bonus_values() {
    assert_eq!(MercQuality::Common.primary_bonus(), 0);
    assert!(MercQuality::Uncommon.primary_bonus() > 0);
    assert!(MercQuality::Rare.primary_bonus() > 0);
    assert!(MercQuality::Elite.primary_bonus() > MercQuality::Uncommon.primary_bonus());
}

#[test]
fn test_merc_quality_cost_range_is_ascending() {
    let (cmin, cmax) = MercQuality::Common.cost_range();
    let (umin, umax) = MercQuality::Uncommon.cost_range();
    let (rmin, rmax) = MercQuality::Rare.cost_range();
    let (emin, emax) = MercQuality::Elite.cost_range();
    assert!(cmax > cmin);
    assert!(umax > umin);
    assert!(rmax > rmin);
    assert!(emax > emin);
    assert!(umin >= cmin, "Uncommon min should be >= Common min");
    assert!(rmin >= umin, "Rare min should be >= Uncommon min");
    assert!(emin >= rmin, "Elite min should be >= Rare min");
}

// =============================================================================
// recruit_pool_size
// =============================================================================

#[test]
fn test_recruit_pool_size_all_ranks() {
    assert_eq!(recruit_pool_size(GuildRank(1)), 3);
    assert_eq!(recruit_pool_size(GuildRank(2)), 4);
    assert_eq!(recruit_pool_size(GuildRank(3)), 4);
    assert_eq!(recruit_pool_size(GuildRank(4)), 5);
    assert_eq!(recruit_pool_size(GuildRank(5)), 5);
}

// =============================================================================
// roll_recruit_quality — distribution correctness
// =============================================================================

#[test]
fn test_roll_recruit_quality_rank1_always_common() {
    let mut rng = seeded_rng(42);
    for _ in 0..100 {
        assert_eq!(
            roll_recruit_quality(GuildRank(1), &mut rng),
            MercQuality::Common
        );
    }
}

#[test]
fn test_roll_recruit_quality_rank2_distribution() {
    let mut rng = seeded_rng(99);
    let n = 1500;
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
    assert!(
        (common_rate - 0.60).abs() < 0.06,
        "Rank 2 Common rate {:.2}% should be ~60%",
        common_rate * 100.0
    );
    assert!(
        (uncommon_rate - 0.40).abs() < 0.06,
        "Rank 2 Uncommon rate {:.2}% should be ~40%",
        uncommon_rate * 100.0
    );
}

#[test]
fn test_roll_recruit_quality_rank3_distribution() {
    let mut rng = seeded_rng(77);
    let n = 1500;
    let mut counts = [0u32; 3];
    for _ in 0..n {
        match roll_recruit_quality(GuildRank(3), &mut rng) {
            MercQuality::Common => counts[0] += 1,
            MercQuality::Uncommon => counts[1] += 1,
            MercQuality::Rare => counts[2] += 1,
            MercQuality::Elite => panic!("Rank 3 should not produce Elite"),
        }
    }
    let rates = counts.map(|c| c as f64 / n as f64);
    assert!(
        (rates[0] - 0.30).abs() < 0.06,
        "Rank 3 Common rate {:.2}% should be ~30%",
        rates[0] * 100.0
    );
    assert!(
        (rates[1] - 0.50).abs() < 0.06,
        "Rank 3 Uncommon rate {:.2}% should be ~50%",
        rates[1] * 100.0
    );
    assert!(
        (rates[2] - 0.20).abs() < 0.06,
        "Rank 3 Rare rate {:.2}% should be ~20%",
        rates[2] * 100.0
    );
}

#[test]
fn test_roll_recruit_quality_rank4_no_common() {
    let mut rng = seeded_rng(55);
    for _ in 0..100 {
        let q = roll_recruit_quality(GuildRank(4), &mut rng);
        assert_ne!(q, MercQuality::Common, "Rank 4 should never produce Common");
    }
}

#[test]
fn test_roll_recruit_quality_rank5_no_common_and_elite_around_30_percent() {
    let mut rng = seeded_rng(7);
    let n = 1500;
    let mut elite = 0u32;
    for _ in 0..n {
        let q = roll_recruit_quality(GuildRank(5), &mut rng);
        assert_ne!(q, MercQuality::Common, "Rank 5 should never produce Common");
        if q == MercQuality::Elite {
            elite += 1;
        }
    }
    let elite_rate = elite as f64 / n as f64;
    assert!(
        (elite_rate - 0.30).abs() < 0.06,
        "Rank 5 Elite rate {:.2}% should be ~30%",
        elite_rate * 100.0
    );
}

// =============================================================================
// MercArchetype base stats invariants
// =============================================================================

#[test]
fn test_archetype_base_stats_all_nonzero() {
    for &arch in MercArchetype::ALL {
        let (p, r, e) = arch.base_stats();
        assert!(p > 0, "{:?} base power must be > 0", arch);
        assert!(r > 0, "{:?} base resilience must be > 0", arch);
        assert!(e > 0, "{:?} base expertise must be > 0", arch);
    }
}

#[test]
fn test_vanguard_base_power_exceeds_expertise() {
    let (p, _r, e) = MercArchetype::Vanguard.base_stats();
    assert!(
        p > e,
        "Vanguard base power ({}) should exceed expertise ({})",
        p,
        e
    );
}

#[test]
fn test_arcanist_base_expertise_exceeds_resilience() {
    let (_p, r, e) = MercArchetype::Arcanist.base_stats();
    assert!(
        e > r,
        "Arcanist base expertise ({}) should exceed resilience ({})",
        e,
        r
    );
}

#[test]
fn test_medic_base_resilience_is_highest_stat() {
    let (p, r, e) = MercArchetype::Medic.base_stats();
    assert!(
        r > p,
        "Medic resilience ({}) should exceed power ({})",
        r,
        p
    );
    assert!(
        r > e,
        "Medic resilience ({}) should exceed expertise ({})",
        r,
        e
    );
}

// =============================================================================
// generate_mercenary — field validation and quality correlation
// =============================================================================

#[test]
fn test_generate_mercenary_starts_available_at_level_1() {
    let mut rng = seeded_rng(1);
    for &arch in MercArchetype::ALL {
        let merc = generate_mercenary(1, arch, MercQuality::Common, &mut rng);
        assert_eq!(
            merc.level,
            Mercenary::BASE_LEVEL,
            "{:?} must start at level 1",
            arch
        );
        assert!(merc.is_available(), "{:?} must start Available", arch);
        assert_eq!(
            merc.missions_completed, 0,
            "{:?} must start with 0 missions_completed",
            arch
        );
    }
}

#[test]
fn test_generate_mercenary_stats_all_nonzero() {
    let mut rng = seeded_rng(2);
    for &arch in MercArchetype::ALL {
        let merc = generate_mercenary(1, arch, MercQuality::Common, &mut rng);
        assert!(merc.power > 0, "{:?} power must be > 0", arch);
        assert!(merc.resilience > 0, "{:?} resilience must be > 0", arch);
        assert!(merc.expertise > 0, "{:?} expertise must be > 0", arch);
    }
}

#[test]
fn test_generate_mercenary_id_set_correctly() {
    let mut rng = seeded_rng(3);
    let merc = generate_mercenary(42, MercArchetype::Scout, MercQuality::Common, &mut rng);
    assert_eq!(merc.id, 42);
}

#[test]
fn test_generate_mercenary_higher_quality_averages_higher_total_stats() {
    let n = 300;
    let mut common_total = 0u64;
    let mut elite_total = 0u64;
    for seed in 0..n {
        let mut rng = seeded_rng(seed);
        let m = generate_mercenary(1, MercArchetype::Vanguard, MercQuality::Common, &mut rng);
        common_total += (m.power + m.resilience + m.expertise) as u64;

        let mut rng = seeded_rng(seed);
        let m = generate_mercenary(1, MercArchetype::Vanguard, MercQuality::Elite, &mut rng);
        elite_total += (m.power + m.resilience + m.expertise) as u64;
    }
    assert!(
        elite_total > common_total,
        "Elite total stats ({}) should exceed Common total ({})",
        elite_total,
        common_total
    );
}

#[test]
fn test_generate_mercenary_vanguard_power_beats_expertise_across_samples() {
    let n = 200;
    let mut power_beats_expertise = 0u32;
    let mut rng = seeded_rng(10);
    for _ in 0..n {
        let m = generate_mercenary(1, MercArchetype::Vanguard, MercQuality::Common, &mut rng);
        if m.power > m.expertise {
            power_beats_expertise += 1;
        }
    }
    // Vanguard base 14 vs 4 — with ±10% variance, power dominates essentially always.
    assert!(
        power_beats_expertise >= 195,
        "Vanguard power should exceed expertise in ~100% of samples, got {}/{}",
        power_beats_expertise,
        n
    );
}

#[test]
fn test_generate_mercenary_arcanist_expertise_beats_resilience_across_samples() {
    let n = 200;
    let mut expertise_beats_resilience = 0u32;
    let mut rng = seeded_rng(20);
    for _ in 0..n {
        let m = generate_mercenary(1, MercArchetype::Arcanist, MercQuality::Common, &mut rng);
        if m.expertise > m.resilience {
            expertise_beats_resilience += 1;
        }
    }
    assert!(
        expertise_beats_resilience >= 195,
        "Arcanist expertise should exceed resilience in ~100% of samples, got {}/{}",
        expertise_beats_resilience,
        n
    );
}

#[test]
fn test_generate_mercenary_medic_resilience_beats_power_across_samples() {
    let n = 200;
    let mut resilience_beats_power = 0u32;
    let mut rng = seeded_rng(30);
    for _ in 0..n {
        let m = generate_mercenary(1, MercArchetype::Medic, MercQuality::Common, &mut rng);
        if m.resilience > m.power {
            resilience_beats_power += 1;
        }
    }
    assert!(
        resilience_beats_power >= 195,
        "Medic resilience should beat power in ~100% of samples, got {}/{}",
        resilience_beats_power,
        n
    );
}

// =============================================================================
// stats_at_level
// =============================================================================

#[test]
fn test_stats_at_level_1_returns_bases_unchanged() {
    for &arch in MercArchetype::ALL {
        let (bp, br, be) = arch.base_stats();
        let (p, r, e) = stats_at_level(arch, bp, br, be, 1);
        assert_eq!(p, bp, "{:?} Level 1 power should equal base", arch);
        assert_eq!(r, br, "{:?} Level 1 resilience should equal base", arch);
        assert_eq!(e, be, "{:?} Level 1 expertise should equal base", arch);
    }
}

#[test]
fn test_stats_at_level_increase_monotonically_all_archetypes() {
    for &arch in MercArchetype::ALL {
        let (bp, br, be) = arch.base_stats();
        let mut prev = (bp, br, be);
        for level in 2..=20u32 {
            let cur = stats_at_level(arch, bp, br, be, level);
            assert!(
                cur.0 >= prev.0,
                "{:?} power should not decrease from level {} to {}",
                arch,
                level - 1,
                level
            );
            assert!(
                cur.1 >= prev.1,
                "{:?} resilience should not decrease from level {} to {}",
                arch,
                level - 1,
                level
            );
            assert!(
                cur.2 >= prev.2,
                "{:?} expertise should not decrease from level {} to {}",
                arch,
                level - 1,
                level
            );
            prev = cur;
        }
    }
}

#[test]
fn test_stats_at_level_vanguard_level10_matches_design_doc() {
    // Vanguard actual base_stats() = (14, 12, 4). Growth: 4.0/3.5/2.0.
    // power: 14 + 4.0*9 = 50
    // resilience: 12 + 3.5*9 = 12+31.5 = 43.5 → 44 (rounded)
    // expertise: 4 + 2.0*9 = 22
    let (bp, br, be) = MercArchetype::Vanguard.base_stats();
    let (p, r, e) = stats_at_level(MercArchetype::Vanguard, bp, br, be, 10);
    assert_eq!(p, 50, "Vanguard L10 power should be 50, got {}", p);
    assert!(
        (r as i32 - 44).abs() <= 1,
        "Vanguard L10 resilience should be ~44, got {}",
        r
    );
    assert_eq!(e, 22, "Vanguard L10 expertise should be 22, got {}", e);
}

#[test]
fn test_stats_at_level_arcanist_expertise_leads_at_level10() {
    let (bp, br, be) = MercArchetype::Arcanist.base_stats();
    let (p, r, e) = stats_at_level(MercArchetype::Arcanist, bp, br, be, 10);
    assert!(
        e > p && e > r,
        "Arcanist L10: expertise ({}) should dominate power ({}) and resilience ({})",
        e,
        p,
        r
    );
}

#[test]
fn test_stats_at_level_saboteur_matches_design_doc() {
    // Saboteur base_stats() = (10, 8, 12). Growth: 3.0/2.5/4.0.
    // power: 10 + 3.0*9 = 37
    // resilience: 8 + 2.5*9 = 30.5 → 31 (rounded)
    // expertise: 12 + 4.0*9 = 48
    let (bp, br, be) = MercArchetype::Saboteur.base_stats();
    let (p, r, e) = stats_at_level(MercArchetype::Saboteur, bp, br, be, 10);
    assert_eq!(p, 37, "Saboteur L10 power should be 37, got {}", p);
    assert!(
        (r as i32 - 31).abs() <= 1,
        "Saboteur L10 resilience should be ~31, got {}",
        r
    );
    assert_eq!(e, 48, "Saboteur L10 expertise should be 48, got {}", e);
}

// =============================================================================
// xp_to_next_level
// =============================================================================

#[test]
fn test_xp_to_next_level_strictly_increasing() {
    let mut prev = 0u32;
    for lvl in 1..=19 {
        let xp = xp_to_next_level(lvl);
        assert!(
            xp > prev,
            "xp_to_next_level({}) = {} should exceed previous {}",
            lvl,
            xp,
            prev
        );
        prev = xp;
    }
}

#[test]
fn test_xp_to_next_level_base_is_200() {
    assert_eq!(xp_to_next_level(1), 200, "Level 1->2 XP should be 200");
}

#[test]
fn test_xp_to_next_level_level2_in_expected_range() {
    // 200 * 2^1.3 ≈ 491.5
    let xp = xp_to_next_level(2);
    assert!(
        xp > 400 && xp < 600,
        "Level 2->3 XP should be ~492, got {}",
        xp
    );
}

// =============================================================================
// apply_merc_xp
// =============================================================================

#[test]
fn test_apply_merc_xp_zero_levels_no_change() {
    let mut rng = seeded_rng(100);
    let mut merc = generate_mercenary(1, MercArchetype::Vanguard, MercQuality::Common, &mut rng);
    let gained = apply_merc_xp(&mut merc, 0);
    assert_eq!(gained, 0, "Applying 0 levels should return 0");
    assert_eq!(merc.level, 1, "Level should remain 1");
}

#[test]
fn test_apply_merc_xp_one_level_increments_level() {
    let mut rng = seeded_rng(101);
    let mut merc = generate_mercenary(1, MercArchetype::Scout, MercQuality::Common, &mut rng);
    let gained = apply_merc_xp(&mut merc, 1);
    assert_eq!(gained, 1);
    assert_eq!(merc.level, 2);
}

#[test]
fn test_apply_merc_xp_multiple_levels_in_one_call() {
    let mut rng = seeded_rng(102);
    let mut merc = generate_mercenary(1, MercArchetype::Medic, MercQuality::Common, &mut rng);
    let gained = apply_merc_xp(&mut merc, 5);
    assert_eq!(gained, 5);
    assert_eq!(merc.level, 6);
}

#[test]
fn test_apply_merc_xp_caps_at_max_level_20() {
    let mut rng = seeded_rng(103);
    let mut merc = generate_mercenary(1, MercArchetype::Arcanist, MercQuality::Common, &mut rng);
    let gained = apply_merc_xp(&mut merc, 99);
    assert_eq!(merc.level, 20, "Level should be capped at 20");
    assert_eq!(gained, 19, "Only 19 actual levels gained (1 to 20)");
}

#[test]
fn test_apply_merc_xp_at_max_level_returns_zero() {
    let mut rng = seeded_rng(104);
    let mut merc = generate_mercenary(1, MercArchetype::Saboteur, MercQuality::Common, &mut rng);
    merc.level = 20;
    let gained = apply_merc_xp(&mut merc, 5);
    assert_eq!(gained, 0, "At max level, apply_merc_xp should return 0");
    assert_eq!(merc.level, 20);
}

#[test]
fn test_apply_merc_xp_stats_increase_after_level_up() {
    let mut rng = seeded_rng(105);
    let mut merc = generate_mercenary(1, MercArchetype::Vanguard, MercQuality::Common, &mut rng);
    let p0 = merc.power;
    let r0 = merc.resilience;
    apply_merc_xp(&mut merc, 5);
    assert!(
        merc.power >= p0,
        "Power should not decrease after level up (was {}, now {})",
        p0,
        merc.power
    );
    assert!(
        merc.resilience >= r0,
        "Resilience should not decrease after level up (was {}, now {})",
        r0,
        merc.resilience
    );
}

// =============================================================================
// generate_recruit_pool
// =============================================================================

#[test]
fn test_generate_recruit_pool_size_matches_rank() {
    let mut rng = seeded_rng(200);
    for rank in 1u8..=5 {
        let mut ids = id_counter();
        let pool = generate_recruit_pool(GuildRank(rank), &mut ids, &mut rng);
        assert_eq!(
            pool.candidates.len(),
            recruit_pool_size(GuildRank(rank)),
            "Rank {} pool size mismatch",
            rank
        );
    }
}

#[test]
fn test_generate_recruit_pool_costs_aligned_with_candidates() {
    let mut rng = seeded_rng(201);
    for rank in 1u8..=5 {
        let mut ids = id_counter();
        let pool = generate_recruit_pool(GuildRank(rank), &mut ids, &mut rng);
        assert_eq!(
            pool.candidates.len(),
            pool.recruit_costs.len(),
            "Candidates and costs must be same length for rank {}",
            rank
        );
    }
}

#[test]
fn test_generate_recruit_pool_all_candidates_start_available() {
    let mut rng = seeded_rng(202);
    let mut ids = id_counter();
    let pool = generate_recruit_pool(GuildRank(3), &mut ids, &mut rng);
    for merc in &pool.candidates {
        assert!(
            merc.is_available(),
            "Recruit pool candidate {} must be Available",
            merc.name
        );
    }
}

#[test]
fn test_generate_recruit_pool_costs_are_multiples_of_5() {
    let mut rng = seeded_rng(203);
    let mut ids = id_counter();
    let pool = generate_recruit_pool(GuildRank(5), &mut ids, &mut rng);
    for &cost in &pool.recruit_costs {
        assert_eq!(cost % 5, 0, "Cost {} should be a multiple of 5", cost);
    }
}

#[test]
fn test_generate_recruit_pool_costs_within_quality_range() {
    let mut rng = seeded_rng(204);
    for rank in 1u8..=5 {
        let mut ids = id_counter();
        let pool = generate_recruit_pool(GuildRank(rank), &mut ids, &mut rng);
        for &cost in &pool.recruit_costs {
            // Common min=50, Elite max=300. With rounding allow ±5.
            assert!(cost >= 45, "Cost {} is below any quality minimum", cost);
            assert!(cost <= 305, "Cost {} is above any quality maximum", cost);
        }
    }
}

#[test]
fn test_generate_recruit_pool_candidate_ids_unique() {
    let mut rng = seeded_rng(205);
    let mut ids = id_counter();
    let pool = generate_recruit_pool(GuildRank(5), &mut ids, &mut rng);
    let id_set: std::collections::HashSet<u64> = pool.candidates.iter().map(|m| m.id).collect();
    assert_eq!(
        id_set.len(),
        pool.candidates.len(),
        "All candidate ids must be unique"
    );
}

#[test]
fn test_generate_recruit_pool_rank1_only_starter_archetypes() {
    let mut rng = seeded_rng(206);
    for _ in 0..30 {
        let mut ids = id_counter();
        let pool = generate_recruit_pool(GuildRank(1), &mut ids, &mut rng);
        for merc in &pool.candidates {
            assert!(
                matches!(
                    merc.archetype,
                    MercArchetype::Vanguard | MercArchetype::Scout | MercArchetype::Medic
                ),
                "Rank 1 pool should only produce Vanguard/Scout/Medic, got {:?}",
                merc.archetype
            );
        }
    }
}

#[test]
fn test_generate_recruit_pool_rank2_includes_arcanist_no_saboteur() {
    let mut rng = seeded_rng(207);
    let mut seen_arcanist = false;
    for _ in 0..100 {
        let mut ids = id_counter();
        let pool = generate_recruit_pool(GuildRank(2), &mut ids, &mut rng);
        for merc in &pool.candidates {
            assert_ne!(
                merc.archetype,
                MercArchetype::Saboteur,
                "Rank 2 should not produce Saboteur"
            );
            if merc.archetype == MercArchetype::Arcanist {
                seen_arcanist = true;
            }
        }
    }
    assert!(
        seen_arcanist,
        "Over 100 Rank 2 pools, Arcanist should appear at least once"
    );
}

#[test]
fn test_generate_recruit_pool_rank3_plus_can_produce_saboteur() {
    let mut rng = seeded_rng(208);
    let mut seen_saboteur = false;
    for _ in 0..100 {
        let mut ids = id_counter();
        let pool = generate_recruit_pool(GuildRank(3), &mut ids, &mut rng);
        for merc in &pool.candidates {
            if merc.archetype == MercArchetype::Saboteur {
                seen_saboteur = true;
            }
        }
    }
    assert!(
        seen_saboteur,
        "Over 100 Rank 3+ pools, Saboteur should appear at least once"
    );
}

// =============================================================================
// generate_starter_roster
// =============================================================================

#[test]
fn test_generate_starter_roster_exactly_3_mercs() {
    let mut rng = seeded_rng(300);
    let mut ids = id_counter();
    let roster = generate_starter_roster(GuildRank(1), &mut ids, &mut rng);
    assert_eq!(
        roster.len(),
        3,
        "Starter roster must contain exactly 3 mercs"
    );
}

#[test]
fn test_generate_starter_roster_contains_vanguard_scout_medic() {
    let mut rng = seeded_rng(301);
    let mut ids = id_counter();
    let roster = generate_starter_roster(GuildRank(1), &mut ids, &mut rng);
    let archetypes: Vec<_> = roster.iter().map(|m| m.archetype).collect();
    assert!(
        archetypes.contains(&MercArchetype::Vanguard),
        "Starter roster must include Vanguard"
    );
    assert!(
        archetypes.contains(&MercArchetype::Scout),
        "Starter roster must include Scout"
    );
    assert!(
        archetypes.contains(&MercArchetype::Medic),
        "Starter roster must include Medic"
    );
}

#[test]
fn test_generate_starter_roster_no_arcanist_or_saboteur() {
    let mut rng = seeded_rng(302);
    let mut ids = id_counter();
    let roster = generate_starter_roster(GuildRank(1), &mut ids, &mut rng);
    for merc in &roster {
        assert!(
            !matches!(
                merc.archetype,
                MercArchetype::Arcanist | MercArchetype::Saboteur
            ),
            "Starter roster should not include {:?}",
            merc.archetype
        );
    }
}

#[test]
fn test_generate_starter_roster_all_available_at_level_1() {
    let mut rng = seeded_rng(303);
    let mut ids = id_counter();
    let roster = generate_starter_roster(GuildRank(2), &mut ids, &mut rng);
    for merc in &roster {
        assert!(merc.is_available(), "{} should start Available", merc.name);
        assert_eq!(
            merc.level,
            Mercenary::BASE_LEVEL,
            "{} should start at level 1",
            merc.name
        );
        assert_eq!(
            merc.missions_completed, 0,
            "{} missions_completed should be 0",
            merc.name
        );
    }
}

#[test]
fn test_generate_starter_roster_unique_ids() {
    let mut rng = seeded_rng(304);
    let mut ids = id_counter();
    let roster = generate_starter_roster(GuildRank(1), &mut ids, &mut rng);
    let id_set: std::collections::HashSet<u64> = roster.iter().map(|m| m.id).collect();
    assert_eq!(id_set.len(), 3, "Starter roster must have 3 unique ids");
}

#[test]
fn test_generate_starter_roster_composition_same_across_all_ranks() {
    // Starter roster is always Vanguard+Scout+Medic regardless of guild rank.
    for rank in 1u8..=5 {
        let mut rng = seeded_rng(305 + rank as u64);
        let mut ids = id_counter();
        let roster = generate_starter_roster(GuildRank(rank), &mut ids, &mut rng);
        assert_eq!(roster.len(), 3, "Starter always 3 mercs for rank {}", rank);
        let archetypes: Vec<_> = roster.iter().map(|m| m.archetype).collect();
        assert!(archetypes.contains(&MercArchetype::Vanguard));
        assert!(archetypes.contains(&MercArchetype::Scout));
        assert!(archetypes.contains(&MercArchetype::Medic));
    }
}

#[test]
fn test_generate_starter_roster_names_vary_across_seeds() {
    let mut names_seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for seed in 0..20u64 {
        let mut rng = seeded_rng(seed + 400);
        let mut ids = id_counter();
        let roster = generate_starter_roster(GuildRank(1), &mut ids, &mut rng);
        for merc in &roster {
            names_seen.insert(merc.name.clone());
        }
    }
    assert!(
        names_seen.len() > 5,
        "Expect name variety across seeds, got {} unique names",
        names_seen.len()
    );
}

// =============================================================================
// roster_has_capacity
// =============================================================================

#[test]
fn test_roster_has_capacity_empty_roster_always_true() {
    let roster: std::collections::HashMap<u64, Mercenary> = std::collections::HashMap::new();
    for rank in 1u8..=5 {
        assert!(
            roster_has_capacity(&roster, GuildRank(rank)),
            "Empty roster should always have capacity at rank {}",
            rank
        );
    }
}

#[test]
fn test_roster_has_capacity_at_all_rank_maximums() {
    // Rank maxima: 5, 7, 9, 12, 15
    let maxima = [(1u8, 5usize), (2, 7), (3, 9), (4, 12), (5, 15)];
    let mut rng = seeded_rng(500);
    for (rank, max) in maxima {
        let roster: std::collections::HashMap<u64, Mercenary> = (0..max as u64)
            .map(|id| {
                let m =
                    generate_mercenary(id, MercArchetype::Vanguard, MercQuality::Common, &mut rng);
                (m.id, m)
            })
            .collect();
        assert!(
            !roster_has_capacity(&roster, GuildRank(rank)),
            "Rank {} full at {} mercs should have no capacity",
            rank,
            max
        );
        // Build a smaller roster with one fewer merc
        let one_below: std::collections::HashMap<u64, Mercenary> = roster
            .iter()
            .take(max - 1)
            .map(|(&k, v)| (k, v.clone()))
            .collect();
        assert!(
            roster_has_capacity(&one_below, GuildRank(rank)),
            "Rank {} at {} mercs should still have capacity",
            rank,
            max - 1
        );
    }
}

#[test]
fn test_roster_has_capacity_rank1_full_has_capacity_at_rank2() {
    let mut rng = seeded_rng(501);
    let roster: std::collections::HashMap<u64, Mercenary> = (0..5)
        .map(|id| {
            let m = generate_mercenary(id, MercArchetype::Vanguard, MercQuality::Common, &mut rng);
            (m.id, m)
        })
        .collect();
    assert!(
        !roster_has_capacity(&roster, GuildRank(1)),
        "5/5 should be full at rank 1"
    );
    assert!(
        roster_has_capacity(&roster, GuildRank(2)),
        "5/7 should have capacity at rank 2"
    );
}

// =============================================================================
// available_mercs
// =============================================================================

#[test]
fn test_available_mercs_empty_roster() {
    let roster: std::collections::HashMap<u64, Mercenary> = std::collections::HashMap::new();
    assert!(available_mercs(&roster).is_empty());
}

#[test]
fn test_available_mercs_filters_all_status_variants() {
    let mut rng = seeded_rng(600);
    let mut m1 = generate_mercenary(1, MercArchetype::Vanguard, MercQuality::Common, &mut rng);
    let mut m2 = generate_mercenary(2, MercArchetype::Scout, MercQuality::Common, &mut rng);
    let mut m3 = generate_mercenary(3, MercArchetype::Medic, MercQuality::Common, &mut rng);
    let m4 = generate_mercenary(4, MercArchetype::Arcanist, MercQuality::Common, &mut rng);
    m1.status = MercStatus::OnMission(1);
    m2.status = MercStatus::Injured {
        recover_at: chrono::Utc::now() + chrono::Duration::hours(6),
    };
    m3.status = MercStatus::Lost;
    // m4 remains Available.
    let roster: std::collections::HashMap<u64, Mercenary> =
        vec![(1, m1), (2, m2), (3, m3), (4, m4)]
            .into_iter()
            .collect();
    let avail = available_mercs(&roster);
    assert_eq!(avail.len(), 1, "Only the Available merc should appear");
    assert_eq!(avail[0].id, 4, "Available merc should be id=4");
}

#[test]
fn test_available_mercs_all_available() {
    let mut rng = seeded_rng(601);
    let roster: std::collections::HashMap<u64, Mercenary> = (1..=5)
        .map(|id| {
            let m = generate_mercenary(id, MercArchetype::Vanguard, MercQuality::Common, &mut rng);
            (m.id, m)
        })
        .collect();
    assert_eq!(
        available_mercs(&roster).len(),
        5,
        "All Available mercs should appear"
    );
}

#[test]
fn test_available_mercs_all_on_mission_returns_empty() {
    let mut rng = seeded_rng(602);
    let mut m1 = generate_mercenary(1, MercArchetype::Vanguard, MercQuality::Common, &mut rng);
    let mut m2 = generate_mercenary(2, MercArchetype::Scout, MercQuality::Common, &mut rng);
    m1.status = MercStatus::OnMission(10);
    m2.status = MercStatus::OnMission(11);
    let roster: std::collections::HashMap<u64, Mercenary> =
        vec![(1, m1), (2, m2)].into_iter().collect();
    assert!(available_mercs(&roster).is_empty());
}

// =============================================================================
// Injury system — injure_merc, check_injury_recovery (wall-clock)
// =============================================================================

#[test]
fn test_injure_merc_light_sets_injured_status() {
    let mut rng = seeded_rng(700);
    let now = Utc::now();
    let mut merc = generate_mercenary(1, MercArchetype::Vanguard, MercQuality::Common, &mut rng);
    injure_merc(&mut merc, InjurySeverity::Light, now, &mut rng);
    assert!(
        matches!(merc.status, MercStatus::Injured { recover_at } if recover_at > now),
        "Light injury should set Injured status with a future recovery time"
    );
    assert!(!merc.is_available(), "Injured merc should not be available");
}

#[test]
fn test_injure_merc_all_severities_produce_injured_status() {
    for severity in [
        InjurySeverity::Light,
        InjurySeverity::Moderate,
        InjurySeverity::Severe,
    ] {
        let mut rng = seeded_rng(701);
        let mut merc = generate_mercenary(1, MercArchetype::Scout, MercQuality::Common, &mut rng);
        injure_merc(&mut merc, severity, Utc::now(), &mut rng);
        assert!(
            matches!(merc.status, MercStatus::Injured { .. }),
            "{:?} injury should set Injured status",
            severity
        );
    }
}

#[test]
fn test_injure_merc_severe_averages_longer_recovery_than_light() {
    let n = 100;
    let now = Utc::now();
    let mut light_sum = 0i64;
    let mut severe_sum = 0i64;
    for seed in 0..n {
        let mut rng = seeded_rng(seed + 702);
        let mut merc1 = generate_mercenary(1, MercArchetype::Medic, MercQuality::Common, &mut rng);
        injure_merc(&mut merc1, InjurySeverity::Light, now, &mut rng);
        if let MercStatus::Injured { recover_at } = merc1.status {
            light_sum += (recover_at - now).num_seconds();
        }

        let mut rng = seeded_rng(seed + 702);
        let mut merc2 = generate_mercenary(1, MercArchetype::Medic, MercQuality::Common, &mut rng);
        injure_merc(&mut merc2, InjurySeverity::Severe, now, &mut rng);
        if let MercStatus::Injured { recover_at } = merc2.status {
            severe_sum += (recover_at - now).num_seconds();
        }
    }
    assert!(
        severe_sum >= light_sum,
        "Severe recovery sum ({}) should be >= Light recovery sum ({})",
        severe_sum,
        light_sum
    );
}

#[test]
fn test_injure_merc_recovery_within_severity_range() {
    let now = Utc::now();
    for severity in [
        InjurySeverity::Light,
        InjurySeverity::Moderate,
        InjurySeverity::Severe,
    ] {
        let (min, max) = severity.recovery_secs();
        for seed in 0..50 {
            let mut rng = seeded_rng(seed + 750);
            let mut merc =
                generate_mercenary(1, MercArchetype::Scout, MercQuality::Common, &mut rng);
            injure_merc(&mut merc, severity, now, &mut rng);
            let MercStatus::Injured { recover_at } = merc.status else {
                panic!("Merc should be Injured");
            };
            let secs = (recover_at - now).num_seconds();
            assert!(
                (min as i64..=max as i64).contains(&secs),
                "{:?} recovery {}s should be in [{}, {}]",
                severity,
                secs,
                min,
                max
            );
        }
    }
}

#[test]
fn test_check_injury_recovery_heals_when_time_elapsed() {
    let mut rng = seeded_rng(800);
    let now = Utc::now();
    let mut merc = generate_mercenary(1, MercArchetype::Scout, MercQuality::Common, &mut rng);
    merc.status = MercStatus::Injured {
        recover_at: now - chrono::Duration::seconds(1),
    };
    let mut roster: std::collections::HashMap<u64, Mercenary> =
        vec![(1, merc)].into_iter().collect();
    let recovered = check_injury_recovery(&mut roster, now);
    assert_eq!(recovered, 1, "Merc past recover_at should heal");
    assert!(
        roster[&1].is_available(),
        "Recovered merc should be Available"
    );
}

#[test]
fn test_check_injury_recovery_heals_at_exact_recovery_time() {
    let mut rng = seeded_rng(801);
    let now = Utc::now();
    let mut merc = generate_mercenary(1, MercArchetype::Scout, MercQuality::Common, &mut rng);
    merc.status = MercStatus::Injured { recover_at: now };
    let mut roster: std::collections::HashMap<u64, Mercenary> =
        vec![(1, merc)].into_iter().collect();
    let recovered = check_injury_recovery(&mut roster, now);
    assert_eq!(recovered, 1, "Merc should heal exactly at recover_at");
    assert!(roster[&1].is_available());
}

#[test]
fn test_check_injury_recovery_leaves_pending_injuries() {
    let mut rng = seeded_rng(802);
    let now = Utc::now();
    let recover_at = now + chrono::Duration::hours(3);
    let mut merc = generate_mercenary(1, MercArchetype::Medic, MercQuality::Common, &mut rng);
    merc.status = MercStatus::Injured { recover_at };
    let mut roster: std::collections::HashMap<u64, Mercenary> =
        vec![(1, merc)].into_iter().collect();
    let recovered = check_injury_recovery(&mut roster, now);
    assert_eq!(recovered, 0, "Merc with time remaining should stay injured");
    assert!(
        matches!(roster[&1].status, MercStatus::Injured { recover_at: t } if t == recover_at),
        "Pending injury must remain untouched"
    );
}

#[test]
fn test_check_injury_recovery_heals_multiple_and_no_missions_needed() {
    // Soft-lock regression (issue #462): recovery must not depend on missions.
    let mut rng = seeded_rng(803);
    let now = Utc::now();
    let mut m1 = generate_mercenary(1, MercArchetype::Vanguard, MercQuality::Common, &mut rng);
    let mut m2 = generate_mercenary(2, MercArchetype::Scout, MercQuality::Common, &mut rng);
    let mut m3 = generate_mercenary(3, MercArchetype::Medic, MercQuality::Common, &mut rng);
    m1.status = MercStatus::Injured {
        recover_at: now - chrono::Duration::hours(1),
    };
    m2.status = MercStatus::Injured {
        recover_at: now - chrono::Duration::minutes(5),
    };
    m3.status = MercStatus::Injured {
        recover_at: now + chrono::Duration::hours(8),
    };
    let mut roster: std::collections::HashMap<u64, Mercenary> =
        vec![(1, m1), (2, m2), (3, m3)].into_iter().collect();
    let recovered = check_injury_recovery(&mut roster, now);
    assert_eq!(recovered, 2, "Both elapsed injuries should heal");
    assert!(roster[&1].is_available());
    assert!(roster[&2].is_available());
    assert!(!roster[&3].is_available());
}

#[test]
fn test_check_injury_recovery_noop_on_available_and_lost() {
    let mut rng = seeded_rng(804);
    let now = Utc::now();
    let m1 = generate_mercenary(1, MercArchetype::Vanguard, MercQuality::Common, &mut rng);
    let mut m2 = generate_mercenary(2, MercArchetype::Saboteur, MercQuality::Common, &mut rng);
    m2.status = MercStatus::Lost;
    let mut roster: std::collections::HashMap<u64, Mercenary> =
        vec![(1, m1), (2, m2)].into_iter().collect();
    let recovered = check_injury_recovery(&mut roster, now);
    assert_eq!(recovered, 0);
    assert!(roster[&1].is_available(), "Available merc stays Available");
    assert!(
        matches!(roster[&2].status, MercStatus::Lost),
        "Lost merc stays Lost"
    );
}

// =============================================================================
// mark_merc_lost and purge_lost_mercs
// =============================================================================

#[test]
fn test_mark_merc_lost_sets_lost_status() {
    let mut rng = seeded_rng(900);
    let mut merc = generate_mercenary(1, MercArchetype::Vanguard, MercQuality::Common, &mut rng);
    assert!(merc.is_available());
    mark_merc_lost(&mut merc);
    assert!(matches!(merc.status, MercStatus::Lost));
    assert!(!merc.is_available());
}

#[test]
fn test_purge_lost_mercs_removes_only_lost() {
    let mut rng = seeded_rng(901);
    let mut m1 = generate_mercenary(1, MercArchetype::Vanguard, MercQuality::Common, &mut rng);
    let m2 = generate_mercenary(2, MercArchetype::Scout, MercQuality::Common, &mut rng);
    let mut m3 = generate_mercenary(3, MercArchetype::Medic, MercQuality::Common, &mut rng);
    mark_merc_lost(&mut m1);
    mark_merc_lost(&mut m3);
    let mut roster: std::collections::HashMap<u64, Mercenary> =
        vec![(1, m1), (2, m2), (3, m3)].into_iter().collect();
    let purged = purge_lost_mercs(&mut roster);
    assert_eq!(purged, 2, "Two mercs were marked lost");
    assert_eq!(roster.len(), 1, "Only one merc should remain");
    assert!(roster.contains_key(&2), "Surviving merc should have id=2");
}

#[test]
fn test_purge_lost_mercs_returns_count() {
    let mut rng = seeded_rng(902);
    let mercs: Vec<_> = (1..=5)
        .map(|id| generate_mercenary(id, MercArchetype::Vanguard, MercQuality::Common, &mut rng))
        .collect();
    let mut roster: std::collections::HashMap<u64, Mercenary> =
        mercs.into_iter().map(|m| (m.id, m)).collect();
    // Mark mercs with ids 1, 3, 5 as lost
    mark_merc_lost(roster.get_mut(&1).unwrap());
    mark_merc_lost(roster.get_mut(&3).unwrap());
    mark_merc_lost(roster.get_mut(&5).unwrap());
    let purged = purge_lost_mercs(&mut roster);
    assert_eq!(purged, 3);
    assert_eq!(roster.len(), 2);
}

#[test]
fn test_purge_lost_mercs_on_empty_roster() {
    let mut roster: std::collections::HashMap<u64, Mercenary> = std::collections::HashMap::new();
    let purged = purge_lost_mercs(&mut roster);
    assert_eq!(purged, 0);
}

#[test]
fn test_purge_lost_mercs_noop_when_none_lost() {
    let mut rng = seeded_rng(903);
    let mut roster: std::collections::HashMap<u64, Mercenary> = (1..=3)
        .map(|id| {
            let m = generate_mercenary(id, MercArchetype::Scout, MercQuality::Common, &mut rng);
            (m.id, m)
        })
        .collect();
    let len_before = roster.len();
    let purged = purge_lost_mercs(&mut roster);
    assert_eq!(purged, 0, "Nothing purged if no one is Lost");
    assert_eq!(roster.len(), len_before);
}

// =============================================================================
// roll_recruit_cost
// =============================================================================

#[test]
fn test_roll_recruit_cost_multiple_of_5() {
    let mut rng = seeded_rng(1000);
    for quality in [
        MercQuality::Common,
        MercQuality::Uncommon,
        MercQuality::Rare,
        MercQuality::Elite,
    ] {
        for _ in 0..100 {
            let cost = roll_recruit_cost(quality, &mut rng);
            assert_eq!(
                cost % 5,
                0,
                "{:?} cost {} must be a multiple of 5",
                quality,
                cost
            );
        }
    }
}

#[test]
fn test_roll_recruit_cost_within_quality_range() {
    let mut rng = seeded_rng(1001);
    for quality in [
        MercQuality::Common,
        MercQuality::Uncommon,
        MercQuality::Rare,
        MercQuality::Elite,
    ] {
        let (min, max) = quality.cost_range();
        let rounded_min = ((min as f64 / 5.0).floor() as u32) * 5;
        let rounded_max = ((max as f64 / 5.0).ceil() as u32) * 5;
        for _ in 0..100 {
            let cost = roll_recruit_cost(quality, &mut rng);
            assert!(
                cost >= rounded_min && cost <= rounded_max,
                "{:?} cost {} out of range [{}, {}]",
                quality,
                cost,
                rounded_min,
                rounded_max
            );
        }
    }
}

#[test]
fn test_roll_recruit_cost_elite_averages_more_than_common() {
    // Common's cost range (50-80) and Elite's (200-300) never overlap, so
    // this holds true by construction — a handful of samples is enough.
    let mut rng = seeded_rng(1002);
    let common: u64 = (0..5)
        .map(|_| roll_recruit_cost(MercQuality::Common, &mut rng) as u64)
        .sum();
    let elite: u64 = (0..5)
        .map(|_| roll_recruit_cost(MercQuality::Elite, &mut rng) as u64)
        .sum();
    assert!(
        elite > common,
        "Elite average cost should exceed Common ({:.1} vs {:.1})",
        elite as f64 / 5.0,
        common as f64 / 5.0
    );
}

// =============================================================================
// generate_merc_name
// =============================================================================

#[test]
fn test_generate_merc_name_has_space_separator() {
    let mut rng = seeded_rng(1100);
    for &arch in MercArchetype::ALL {
        let name = generate_merc_name(arch, &mut rng);
        assert!(!name.is_empty(), "{:?} name should not be empty", arch);
        assert!(
            name.contains(' '),
            "{:?} name should contain a space, got '{}'",
            arch,
            name
        );
    }
}

#[test]
fn test_generate_merc_name_variety_across_seeds() {
    let mut names: std::collections::HashSet<String> = std::collections::HashSet::new();
    for seed in 0..50u64 {
        let mut rng = seeded_rng(seed + 1101);
        names.insert(generate_merc_name(MercArchetype::Vanguard, &mut rng));
    }
    assert!(
        names.len() > 10,
        "Expected high name variety, got only {} unique names over 50 seeds",
        names.len()
    );
}

#[test]
fn test_generate_merc_name_uses_archetype_appropriate_epithets() {
    let vanguard_markers = ["Ironwall", "Bulwark", "Shieldborn", "Rampart", "Steadfast"];
    let scout_markers = [
        "Shadowfoot",
        "Duskwalker",
        "Swiftpath",
        "Lightfoot",
        "Nightcrawler",
    ];
    let mut rng = seeded_rng(1102);
    let mut found_vanguard = false;
    let mut found_scout = false;
    for _ in 0..200 {
        let vname = generate_merc_name(MercArchetype::Vanguard, &mut rng);
        let sname = generate_merc_name(MercArchetype::Scout, &mut rng);
        if vanguard_markers.iter().any(|m| vname.contains(m)) {
            found_vanguard = true;
        }
        if scout_markers.iter().any(|m| sname.contains(m)) {
            found_scout = true;
        }
    }
    assert!(
        found_vanguard,
        "Vanguard names should use Vanguard epithets"
    );
    assert!(found_scout, "Scout names should use Scout epithets");
}

// =============================================================================
// DeepState::on_prestige — preserves operational state, advances generation
// =============================================================================

#[test]
fn test_on_prestige_preserves_roster() {
    let mut rng = seeded_rng(2000);
    let mut state = DeepState::new();
    state.persistent.discovered = true;
    let mut ids = id_counter();
    let starters = generate_starter_roster(GuildRank(1), &mut ids, &mut rng);
    let count = starters.len();
    state.prestige.roster = starters.into_iter().map(|m| (m.id, m)).collect();
    assert!(!state.prestige.roster.is_empty());
    state.on_prestige();
    assert_eq!(
        state.prestige.roster.len(),
        count,
        "on_prestige() should preserve the roster"
    );
}

#[test]
fn test_on_prestige_preserves_warband_marks() {
    let mut state = DeepState::new();
    state.prestige.warband_marks = 500;
    state.on_prestige();
    assert_eq!(
        state.prestige.warband_marks, 500,
        "on_prestige() should preserve warband_marks"
    );
}

#[test]
fn test_on_prestige_preserves_active_missions() {
    let mut state = DeepState::new();
    state.on_prestige();
    assert!(
        state.prestige.active_missions.is_empty(),
        "on_prestige() should preserve active_missions (empty to start)"
    );
}

#[test]
fn test_on_prestige_preserves_guild_rank() {
    let mut state = DeepState::new();
    state.persistent.guild_rank = GuildRank(3);
    state.on_prestige();
    assert_eq!(
        state.persistent.guild_rank,
        GuildRank(3),
        "Guild rank must persist across prestige"
    );
}

#[test]
fn test_on_prestige_preserves_discovered_flag() {
    let mut state = DeepState::new();
    state.persistent.discovered = true;
    state.on_prestige();
    assert!(
        state.persistent.discovered,
        "discovered flag must persist across prestige"
    );
}

#[test]
fn test_on_prestige_preserves_deepest_layer_reached() {
    let mut state = DeepState::new();
    state.persistent.deepest_layer_reached = 7;
    state.on_prestige();
    assert_eq!(
        state.persistent.deepest_layer_reached, 7,
        "deepest_layer_reached must persist across prestige"
    );
}

#[test]
fn test_on_prestige_preserves_merc_id_counter() {
    let mut rng = seeded_rng(2001);
    let mut state = DeepState::new();
    state.persistent.discovered = true;
    // Consume IDs for starters.
    let starters =
        generate_starter_roster(GuildRank(1), || state.persistent.next_merc_id(), &mut rng);
    state.prestige.roster = starters.into_iter().map(|m| (m.id, m)).collect();
    let counter_before = state.persistent.merc_id_counter;
    assert_eq!(counter_before, 3, "Three IDs consumed");
    state.on_prestige();
    assert_eq!(
        state.persistent.merc_id_counter, counter_before,
        "merc_id_counter should survive prestige"
    );
}

// =============================================================================
// DeepPrestige helper methods
// =============================================================================

#[test]
fn test_deep_prestige_spend_marks_success_and_failure() {
    let mut prestige = DeepPrestige::new();
    prestige.warband_marks = 100;
    assert!(
        prestige.spend_marks(50),
        "Should succeed with sufficient marks"
    );
    assert_eq!(prestige.warband_marks, 50);
    assert!(
        !prestige.spend_marks(100),
        "Should fail with insufficient marks"
    );
    assert_eq!(
        prestige.warband_marks, 50,
        "Balance unchanged on failed spend"
    );
}

#[test]
fn test_deep_prestige_available_merc_count() {
    let mut rng = seeded_rng(3000);
    let mut prestige = DeepPrestige::new();
    let mut m1 = generate_mercenary(1, MercArchetype::Vanguard, MercQuality::Common, &mut rng);
    let m2 = generate_mercenary(2, MercArchetype::Scout, MercQuality::Common, &mut rng);
    m1.status = MercStatus::Injured {
        recover_at: chrono::Utc::now() + chrono::Duration::hours(6),
    };
    prestige.roster = vec![(m1.id, m1), (m2.id, m2)].into_iter().collect();
    assert_eq!(prestige.available_merc_count(), 1);
}

#[test]
fn test_deep_prestige_find_merc_found_and_not_found() {
    let mut rng = seeded_rng(3001);
    let mut prestige = DeepPrestige::new();
    let merc = generate_mercenary(7, MercArchetype::Medic, MercQuality::Common, &mut rng);
    prestige.roster = vec![(merc.id, merc)].into_iter().collect();
    assert!(
        prestige.find_merc(7).is_some(),
        "find_merc should find existing id"
    );
    assert!(
        prestige.find_merc(99).is_none(),
        "find_merc should return None for unknown id"
    );
}

#[test]
fn test_deep_prestige_find_merc_mut_modifies_in_place() {
    let mut rng = seeded_rng(3002);
    let mut prestige = DeepPrestige::new();
    let merc = generate_mercenary(7, MercArchetype::Medic, MercQuality::Common, &mut rng);
    prestige.roster = vec![(merc.id, merc)].into_iter().collect();
    let found = prestige.find_merc_mut(7).unwrap();
    found.missions_completed = 5;
    assert_eq!(prestige.roster[&7].missions_completed, 5);
}

// =============================================================================
// GuildRank helper methods
// =============================================================================

#[test]
fn test_guild_rank_max_roster_values() {
    assert_eq!(GuildRank(1).max_roster(), 5);
    assert_eq!(GuildRank(2).max_roster(), 7);
    assert_eq!(GuildRank(3).max_roster(), 9);
    assert_eq!(GuildRank(4).max_roster(), 12);
    assert_eq!(GuildRank(5).max_roster(), 15);
}

#[test]
fn test_guild_rank_concurrent_missions() {
    assert_eq!(GuildRank(1).concurrent_missions(), 1);
    assert_eq!(GuildRank(2).concurrent_missions(), 2);
    assert_eq!(GuildRank(3).concurrent_missions(), 2);
    assert_eq!(GuildRank(4).concurrent_missions(), 3);
    assert_eq!(GuildRank(5).concurrent_missions(), 4);
}

#[test]
fn test_guild_rank_next_and_can_advance() {
    assert!(GuildRank(1).can_advance());
    assert_eq!(GuildRank(1).next(), Some(GuildRank(2)));
    assert!(!GuildRank(5).can_advance());
    assert_eq!(GuildRank(5).next(), None);
}

#[test]
fn test_guild_rank_display_names_all_defined() {
    let names = ["Freelancers", "Company", "Battalion", "Legion", "Vanguard"];
    for (i, &expected) in names.iter().enumerate() {
        assert_eq!(
            GuildRank(i as u8 + 1).display_name(),
            expected,
            "Rank {} display name mismatch",
            i + 1
        );
    }
}
