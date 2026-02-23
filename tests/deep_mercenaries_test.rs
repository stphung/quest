//! Integration tests for The Deep — mercenary generation, recruitment, and lifecycle.

use quest::deep::mercenaries::{
    apply_medic_downgrade, generate_merc_name, generate_mercenary, generate_recruitment_pool,
    generate_starting_roster, injure_mercenary, recover_mercenaries, recruit_mercenary,
    RECRUITMENT_COST,
};
use quest::deep::{GuildRank, MercArchetype, MercStatus, Mercenary, TheDeepState};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

// =========================================================================
// Helpers
// =========================================================================

fn seeded_rng() -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(42)
}

fn all_guild_ranks() -> Vec<GuildRank> {
    vec![
        GuildRank::Freelancers,
        GuildRank::Sellswords,
        GuildRank::Company,
        GuildRank::Battalion,
        GuildRank::Legion,
    ]
}

// =========================================================================
// generate_merc_name
// =========================================================================

#[test]
fn test_generate_merc_name_not_empty() {
    let mut rng = seeded_rng();
    let name = generate_merc_name(&mut rng);
    assert!(!name.is_empty(), "merc name should not be empty");
}

#[test]
fn test_generate_merc_name_has_two_parts() {
    let mut rng = seeded_rng();
    for _ in 0..20 {
        let name = generate_merc_name(&mut rng);
        let parts: Vec<&str> = name.split_whitespace().collect();
        assert_eq!(
            parts.len(),
            2,
            "merc name '{}' should have exactly first name + surname",
            name
        );
    }
}

#[test]
fn test_generate_merc_name_varies() {
    // With seed 42, produce several names and verify they're not all identical
    let mut rng = seeded_rng();
    let names: Vec<String> = (0..10).map(|_| generate_merc_name(&mut rng)).collect();
    let unique: std::collections::HashSet<&str> = names.iter().map(|s| s.as_str()).collect();
    assert!(
        unique.len() > 1,
        "generated names should not all be identical: {:?}",
        names
    );
}

// =========================================================================
// generate_mercenary — stat ranges by guild rank
// =========================================================================

#[test]
fn test_generate_mercenary_freelancers_stat_range() {
    let mut rng = seeded_rng();
    for i in 0..20 {
        let merc = generate_mercenary(MercArchetype::Scout, GuildRank::Freelancers, i, &mut rng);
        // Scout has 1.0/1.0 bias — stats should be close to the raw range (15-25)
        // Allow some headroom for rounding
        assert!(
            merc.power >= 1 && merc.power <= 35,
            "Freelancer power {} out of reasonable range",
            merc.power
        );
        assert!(
            merc.resilience >= 1 && merc.resilience <= 35,
            "Freelancer resilience {} out of reasonable range",
            merc.resilience
        );
    }
}

#[test]
fn test_generate_mercenary_legion_higher_stats_than_freelancers() {
    let mut rng_f = ChaCha8Rng::seed_from_u64(42);
    let mut rng_l = ChaCha8Rng::seed_from_u64(42);

    let samples = 50;
    let total_freelancer_power: u32 = (0..samples)
        .map(|i| {
            generate_mercenary(MercArchetype::Scout, GuildRank::Freelancers, i, &mut rng_f).power
        })
        .sum();
    let total_legion_power: u32 = (0..samples)
        .map(|i| generate_mercenary(MercArchetype::Scout, GuildRank::Legion, i, &mut rng_l).power)
        .sum();

    assert!(
        total_legion_power > total_freelancer_power,
        "Legion mercs ({}) should have higher total power than Freelancers ({})",
        total_legion_power,
        total_freelancer_power
    );
}

#[test]
fn test_generate_mercenary_level_starts_at_one() {
    let mut rng = seeded_rng();
    for rank in all_guild_ranks() {
        let merc = generate_mercenary(MercArchetype::Vanguard, rank, 1, &mut rng);
        assert_eq!(merc.level, 1, "new merc should start at level 1");
    }
}

#[test]
fn test_generate_mercenary_status_is_ready() {
    let mut rng = seeded_rng();
    let merc = generate_mercenary(MercArchetype::Medic, GuildRank::Company, 1, &mut rng);
    assert_eq!(merc.status, MercStatus::Ready);
}

#[test]
fn test_generate_mercenary_missions_completed_zero() {
    let mut rng = seeded_rng();
    let merc = generate_mercenary(MercArchetype::Saboteur, GuildRank::Sellswords, 1, &mut rng);
    assert_eq!(merc.missions_completed, 0);
}

#[test]
fn test_generate_mercenary_injury_cooldown_zero() {
    let mut rng = seeded_rng();
    let merc = generate_mercenary(MercArchetype::Arcanist, GuildRank::Battalion, 1, &mut rng);
    assert_eq!(merc.injury_cooldown, 0);
}

#[test]
fn test_generate_mercenary_archetype_preserved() {
    let mut rng = seeded_rng();
    for archetype in [
        MercArchetype::Vanguard,
        MercArchetype::Scout,
        MercArchetype::Medic,
        MercArchetype::Saboteur,
        MercArchetype::Arcanist,
    ] {
        let merc = generate_mercenary(archetype, GuildRank::Freelancers, 1, &mut rng);
        assert_eq!(merc.archetype, archetype);
    }
}

#[test]
fn test_generate_mercenary_vanguard_has_higher_resilience_than_arcanist_on_average() {
    // Vanguard bias: (0.85, 1.20), Arcanist bias: (1.25, 0.80)
    // Over many samples vanguard resilience should exceed arcanist resilience
    let mut rng_v = ChaCha8Rng::seed_from_u64(99);
    let mut rng_a = ChaCha8Rng::seed_from_u64(99);
    let samples = 100;

    let vanguard_res: u32 = (0..samples)
        .map(|i| {
            generate_mercenary(MercArchetype::Vanguard, GuildRank::Company, i, &mut rng_v)
                .resilience
        })
        .sum();
    let arcanist_res: u32 = (0..samples)
        .map(|i| {
            generate_mercenary(MercArchetype::Arcanist, GuildRank::Company, i, &mut rng_a)
                .resilience
        })
        .sum();

    assert!(
        vanguard_res > arcanist_res,
        "Vanguard avg resilience ({}) should exceed Arcanist ({})",
        vanguard_res,
        arcanist_res
    );
}

#[test]
fn test_generate_mercenary_arcanist_has_higher_power_than_vanguard_on_average() {
    let mut rng_v = ChaCha8Rng::seed_from_u64(99);
    let mut rng_a = ChaCha8Rng::seed_from_u64(99);
    let samples = 100;

    let vanguard_pow: u32 = (0..samples)
        .map(|i| {
            generate_mercenary(MercArchetype::Vanguard, GuildRank::Company, i, &mut rng_v).power
        })
        .sum();
    let arcanist_pow: u32 = (0..samples)
        .map(|i| {
            generate_mercenary(MercArchetype::Arcanist, GuildRank::Company, i, &mut rng_a).power
        })
        .sum();

    assert!(
        arcanist_pow > vanguard_pow,
        "Arcanist avg power ({}) should exceed Vanguard ({})",
        arcanist_pow,
        vanguard_pow
    );
}

// =========================================================================
// generate_starting_roster
// =========================================================================

#[test]
fn test_generate_starting_roster_freelancers_has_five_mercs() {
    let mut rng = seeded_rng();
    let roster = generate_starting_roster(GuildRank::Freelancers, &mut rng);
    // min(5, 5) = 5
    assert_eq!(roster.len(), 5);
}

#[test]
fn test_generate_starting_roster_has_all_five_archetypes() {
    let mut rng = seeded_rng();
    let roster = generate_starting_roster(GuildRank::Freelancers, &mut rng);
    let archetypes: std::collections::HashSet<String> =
        roster.iter().map(|m| format!("{}", m.archetype)).collect();
    assert!(
        archetypes.contains("Vanguard"),
        "Roster missing Vanguard: {:?}",
        archetypes
    );
    assert!(
        archetypes.contains("Scout"),
        "Roster missing Scout: {:?}",
        archetypes
    );
    assert!(
        archetypes.contains("Medic"),
        "Roster missing Medic: {:?}",
        archetypes
    );
    assert!(
        archetypes.contains("Saboteur"),
        "Roster missing Saboteur: {:?}",
        archetypes
    );
    assert!(
        archetypes.contains("Arcanist"),
        "Roster missing Arcanist: {:?}",
        archetypes
    );
}

#[test]
fn test_generate_starting_roster_all_mercs_ready() {
    let mut rng = seeded_rng();
    let roster = generate_starting_roster(GuildRank::Freelancers, &mut rng);
    for merc in &roster {
        assert_eq!(
            merc.status,
            MercStatus::Ready,
            "Starting merc {} should be Ready",
            merc.name
        );
    }
}

#[test]
fn test_generate_starting_roster_all_mercs_level_one() {
    let mut rng = seeded_rng();
    let roster = generate_starting_roster(GuildRank::Sellswords, &mut rng);
    for merc in &roster {
        assert_eq!(
            merc.level, 1,
            "Starting merc {} should be level 1",
            merc.name
        );
    }
}

// The starting roster cap is min(5, roster_cap()), so for all ranks it's 5
// since the minimum roster_cap is 5 (Freelancers).
#[test]
fn test_generate_starting_roster_cap_is_min_five_for_all_ranks() {
    let mut rng = seeded_rng();
    for rank in all_guild_ranks() {
        let mut rng2 = ChaCha8Rng::seed_from_u64(42);
        let roster = generate_starting_roster(rank, &mut rng2);
        let expected = rank.roster_cap().min(5);
        assert_eq!(
            roster.len(),
            expected,
            "Roster for {:?} should have {} mercs",
            rank,
            expected
        );
        let _ = &mut rng; // suppress unused warning
    }
}

// =========================================================================
// generate_recruitment_pool
// =========================================================================

#[test]
fn test_generate_recruitment_pool_has_three_to_four_candidates() {
    let mut rng = seeded_rng();
    // Run several times to see both sizes
    let mut sizes_seen = std::collections::HashSet::new();
    for seed in 0..50u64 {
        let mut r = ChaCha8Rng::seed_from_u64(seed);
        let pool = generate_recruitment_pool(GuildRank::Freelancers, &mut r);
        assert!(
            pool.len() == 3 || pool.len() == 4,
            "Pool size {} is not 3 or 4",
            pool.len()
        );
        sizes_seen.insert(pool.len());
    }
    assert!(
        sizes_seen.contains(&3) && sizes_seen.contains(&4),
        "Pool should produce both sizes 3 and 4 across different seeds, got: {:?}",
        sizes_seen
    );
    let _ = &mut rng;
}

#[test]
fn test_generate_recruitment_pool_mercs_are_ready() {
    let mut rng = seeded_rng();
    let pool = generate_recruitment_pool(GuildRank::Company, &mut rng);
    for merc in &pool {
        assert_eq!(
            merc.status,
            MercStatus::Ready,
            "Pool merc {} should be Ready",
            merc.name
        );
    }
}

#[test]
fn test_generate_recruitment_pool_mercs_level_one() {
    let mut rng = seeded_rng();
    let pool = generate_recruitment_pool(GuildRank::Legion, &mut rng);
    for merc in &pool {
        assert_eq!(merc.level, 1);
    }
}

// =========================================================================
// recruit_mercenary — happy path
// =========================================================================

fn state_with_marks_and_pool(marks: u32, rank: GuildRank) -> TheDeepState {
    let mut rng = seeded_rng();
    let mut state = TheDeepState::new();
    state.account.guild_rank = rank;
    state.run.warband_marks = marks;
    state.run.recruitment_pool = generate_recruitment_pool(rank, &mut rng);
    state
}

#[test]
fn test_recruit_mercenary_deducts_marks() {
    let mut state = state_with_marks_and_pool(100, GuildRank::Freelancers);
    let before = state.run.warband_marks;
    recruit_mercenary(&mut state, 0).expect("recruitment should succeed");
    assert_eq!(
        state.run.warband_marks,
        before - RECRUITMENT_COST,
        "Marks should decrease by RECRUITMENT_COST"
    );
}

#[test]
fn test_recruit_mercenary_moves_merc_to_roster() {
    let mut state = state_with_marks_and_pool(100, GuildRank::Freelancers);
    let pool_len_before = state.run.recruitment_pool.len();
    recruit_mercenary(&mut state, 0).expect("recruitment should succeed");
    assert_eq!(state.run.mercenaries.len(), 1, "Roster should have 1 merc");
    assert_eq!(
        state.run.recruitment_pool.len(),
        pool_len_before - 1,
        "Pool should shrink by 1"
    );
}

#[test]
fn test_recruit_mercenary_selects_correct_pool_index() {
    let mut state = state_with_marks_and_pool(200, GuildRank::Company);
    let expected_name = state.run.recruitment_pool[1].name.clone();
    recruit_mercenary(&mut state, 1).expect("recruitment should succeed");
    assert_eq!(
        state.run.mercenaries[0].name, expected_name,
        "Recruited merc should match the pool entry at index 1"
    );
}

// =========================================================================
// recruit_mercenary — failure cases
// =========================================================================

#[test]
fn test_recruit_mercenary_fails_invalid_index() {
    let mut state = state_with_marks_and_pool(100, GuildRank::Freelancers);
    let pool_len = state.run.recruitment_pool.len();
    let result = recruit_mercenary(&mut state, pool_len); // one past end
    assert!(result.is_err(), "Out-of-bounds index should fail");
}

#[test]
fn test_recruit_mercenary_fails_when_roster_full() {
    let mut state = state_with_marks_and_pool(500, GuildRank::Freelancers);
    let cap = state.account.guild_rank.roster_cap();

    // Fill the roster to the cap
    let mut rng = seeded_rng();
    for i in 0..cap {
        state.run.mercenaries.push(Mercenary {
            id: i as u32,
            name: format!("Merc {}", i),
            archetype: MercArchetype::Scout,
            level: 1,
            power: 20,
            resilience: 20,
            status: MercStatus::Ready,
            missions_completed: 0,
            injury_cooldown: 0,
        });
        let _ = &mut rng;
    }

    let result = recruit_mercenary(&mut state, 0);
    assert!(result.is_err(), "Recruiting into a full roster should fail");
    // Marks and pool should be unchanged
    assert_eq!(state.run.warband_marks, 500);
}

#[test]
fn test_recruit_mercenary_fails_insufficient_marks() {
    let mut state = state_with_marks_and_pool(RECRUITMENT_COST - 1, GuildRank::Freelancers);
    let result = recruit_mercenary(&mut state, 0);
    assert!(result.is_err(), "Insufficient marks should fail");
    // Pool should be unchanged
    assert!(!state.run.recruitment_pool.is_empty());
    // Marks should be unchanged
    assert_eq!(state.run.warband_marks, RECRUITMENT_COST - 1);
}

// =========================================================================
// injure_mercenary / recover_mercenaries lifecycle
// =========================================================================

fn make_merc(id: u32, archetype: MercArchetype) -> Mercenary {
    Mercenary {
        id,
        name: format!("Test Merc {}", id),
        archetype,
        level: 1,
        power: 20,
        resilience: 20,
        status: MercStatus::Ready,
        missions_completed: 0,
        injury_cooldown: 0,
    }
}

#[test]
fn test_injure_mercenary_sets_status_and_cooldown() {
    let mut merc = make_merc(1, MercArchetype::Vanguard);
    injure_mercenary(&mut merc, 2);
    assert_eq!(merc.status, MercStatus::Injured);
    assert_eq!(merc.injury_cooldown, 2);
}

#[test]
fn test_recover_mercenaries_decrements_cooldown() {
    let mut state = TheDeepState::new();
    let mut merc = make_merc(1, MercArchetype::Scout);
    injure_mercenary(&mut merc, 2);
    state.run.mercenaries.push(merc);

    recover_mercenaries(&mut state);

    assert_eq!(state.run.mercenaries[0].injury_cooldown, 1);
    assert_eq!(state.run.mercenaries[0].status, MercStatus::Injured);
}

#[test]
fn test_recover_mercenaries_restores_ready_at_zero() {
    let mut state = TheDeepState::new();
    let mut merc = make_merc(1, MercArchetype::Medic);
    injure_mercenary(&mut merc, 1);
    state.run.mercenaries.push(merc);

    recover_mercenaries(&mut state);

    assert_eq!(state.run.mercenaries[0].injury_cooldown, 0);
    assert_eq!(state.run.mercenaries[0].status, MercStatus::Ready);
}

#[test]
fn test_recover_mercenaries_two_mission_injury_takes_two_recoveries() {
    let mut state = TheDeepState::new();
    let mut merc = make_merc(1, MercArchetype::Saboteur);
    injure_mercenary(&mut merc, 2);
    state.run.mercenaries.push(merc);

    recover_mercenaries(&mut state);
    assert_eq!(state.run.mercenaries[0].status, MercStatus::Injured);
    assert_eq!(state.run.mercenaries[0].injury_cooldown, 1);

    recover_mercenaries(&mut state);
    assert_eq!(state.run.mercenaries[0].status, MercStatus::Ready);
    assert_eq!(state.run.mercenaries[0].injury_cooldown, 0);
}

#[test]
fn test_recover_mercenaries_does_not_affect_ready_mercs() {
    let mut state = TheDeepState::new();
    let merc = make_merc(1, MercArchetype::Arcanist);
    // merc is Ready with cooldown 0
    state.run.mercenaries.push(merc);

    recover_mercenaries(&mut state);

    assert_eq!(state.run.mercenaries[0].status, MercStatus::Ready);
    assert_eq!(state.run.mercenaries[0].injury_cooldown, 0);
}

#[test]
fn test_recover_mercenaries_does_not_affect_lost_mercs() {
    let mut state = TheDeepState::new();
    let mut merc = make_merc(1, MercArchetype::Vanguard);
    merc.status = MercStatus::Lost;
    merc.injury_cooldown = 0;
    state.run.mercenaries.push(merc);

    recover_mercenaries(&mut state);

    assert_eq!(state.run.mercenaries[0].status, MercStatus::Lost);
}

#[test]
fn test_recover_mercenaries_handles_mixed_roster() {
    let mut state = TheDeepState::new();

    let ready_merc = make_merc(1, MercArchetype::Scout);
    let mut injured_merc = make_merc(2, MercArchetype::Medic);
    let mut lost_merc = make_merc(3, MercArchetype::Arcanist);

    injure_mercenary(&mut injured_merc, 2);
    lost_merc.status = MercStatus::Lost;

    state.run.mercenaries.push(ready_merc);
    state.run.mercenaries.push(injured_merc);
    state.run.mercenaries.push(lost_merc);

    recover_mercenaries(&mut state);

    assert_eq!(state.run.mercenaries[0].status, MercStatus::Ready); // unchanged
    assert_eq!(state.run.mercenaries[1].status, MercStatus::Injured); // still injured, cooldown now 1
    assert_eq!(state.run.mercenaries[1].injury_cooldown, 1);
    assert_eq!(state.run.mercenaries[2].status, MercStatus::Lost); // unchanged
}

// =========================================================================
// apply_medic_downgrade
// =========================================================================

#[test]
fn test_medic_downgrade_loss_to_injury() {
    let medic = make_merc(1, MercArchetype::Medic);
    let squad = vec![&medic];
    let result = apply_medic_downgrade(&squad, 2);
    assert_eq!(
        result, 1,
        "Loss (2) should downgrade to injury (1) with Medic"
    );
}

#[test]
fn test_medic_downgrade_injury_to_scratch() {
    let medic = make_merc(1, MercArchetype::Medic);
    let squad = vec![&medic];
    let result = apply_medic_downgrade(&squad, 1);
    assert_eq!(
        result, 0,
        "Injury (1) should downgrade to scratch (0) with Medic"
    );
}

#[test]
fn test_medic_downgrade_scratch_stays_zero() {
    let medic = make_merc(1, MercArchetype::Medic);
    let squad = vec![&medic];
    let result = apply_medic_downgrade(&squad, 0);
    assert_eq!(result, 0, "Scratch (0) should stay 0 even with Medic");
}

#[test]
fn test_no_medic_no_downgrade() {
    let vanguard = make_merc(1, MercArchetype::Vanguard);
    let scout = make_merc(2, MercArchetype::Scout);
    let squad = vec![&vanguard, &scout];
    assert_eq!(apply_medic_downgrade(&squad, 2), 2);
    assert_eq!(apply_medic_downgrade(&squad, 1), 1);
    assert_eq!(apply_medic_downgrade(&squad, 0), 0);
}

#[test]
fn test_lost_medic_does_not_downgrade() {
    // A Lost medic cannot provide the downgrade
    let mut medic = make_merc(1, MercArchetype::Medic);
    medic.status = MercStatus::Lost;
    let squad = vec![&medic];
    let result = apply_medic_downgrade(&squad, 2);
    assert_eq!(result, 2, "Lost Medic should not provide downgrade benefit");
}

#[test]
fn test_medic_in_mixed_squad_still_downgrades() {
    let vanguard = make_merc(1, MercArchetype::Vanguard);
    let medic = make_merc(2, MercArchetype::Medic);
    let arcanist = make_merc(3, MercArchetype::Arcanist);
    let squad = vec![&vanguard, &medic, &arcanist];
    let result = apply_medic_downgrade(&squad, 2);
    assert_eq!(
        result, 1,
        "Medic in squad should downgrade regardless of other members"
    );
}

#[test]
fn test_apply_medic_downgrade_empty_squad_no_panic() {
    let squad: Vec<&Mercenary> = vec![];
    let result = apply_medic_downgrade(&squad, 2);
    assert_eq!(
        result, 2,
        "Empty squad should not crash and should not downgrade"
    );
}
