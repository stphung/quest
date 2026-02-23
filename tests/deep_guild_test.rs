//! Integration tests for The Deep — guild upgrade system.

use quest::deep::guild::{can_upgrade_guild, guild_upgrade_progress, upgrade_guild};
use quest::deep::{DeepAccountState, DeepRunState, GuildRank, LayerState, TheDeepState};

// =========================================================================
// Helpers
// =========================================================================

fn cleared_layer(layer_number: u8) -> LayerState {
    LayerState {
        layer_number,
        familiarity: 1.0,
        cleared: true,
        infrastructure: Vec::new(),
    }
}

fn state_with_rank_and_marks(rank: GuildRank, marks: u32) -> TheDeepState {
    TheDeepState {
        discovered: true,
        account: DeepAccountState {
            guild_rank: rank,
            layers: Vec::new(),
            total_marks_earned: 0,
        },
        run: DeepRunState {
            warband_marks: marks,
            ..DeepRunState::new()
        },
    }
}

/// Build a state that satisfies all upgrade conditions for `rank`.
fn state_ready_to_upgrade(rank: GuildRank) -> TheDeepState {
    let next = rank.next().expect("rank must not be Legion");
    let cost = rank.upgrade_cost().expect("rank must not be Legion");
    let mut state = state_with_rank_and_marks(rank, cost);

    if let Some(required_layer) = next.required_layer() {
        state.account.layers.push(cleared_layer(required_layer));
    }

    state
}

// =========================================================================
// can_upgrade_guild
// =========================================================================

#[test]
fn can_upgrade_guild_fails_at_max_rank() {
    let state = state_with_rank_and_marks(GuildRank::Legion, 999_999);
    let result = can_upgrade_guild(&state);
    assert_eq!(result, Err("Already at maximum guild rank (Legion)"));
}

#[test]
fn can_upgrade_guild_fails_without_enough_marks() {
    // Freelancers → Sellswords costs 500; give only 499
    let mut state = state_with_rank_and_marks(GuildRank::Freelancers, 499);
    // No layer requirement for Sellswords (requires layer 3 — actually it does)
    state.account.layers.push(cleared_layer(3));

    let result = can_upgrade_guild(&state);
    assert_eq!(result, Err("Insufficient Warband Marks for guild upgrade"));
}

#[test]
fn can_upgrade_guild_fails_without_required_layer_cleared() {
    // Freelancers → Sellswords requires layer 3 cleared
    let cost = GuildRank::Freelancers.upgrade_cost().unwrap();
    let state = state_with_rank_and_marks(GuildRank::Freelancers, cost);
    // No layers added

    let result = can_upgrade_guild(&state);
    assert_eq!(result, Err("Required layer has not been cleared"));
}

#[test]
fn can_upgrade_guild_fails_when_required_layer_exists_but_not_cleared() {
    let cost = GuildRank::Freelancers.upgrade_cost().unwrap();
    let mut state = state_with_rank_and_marks(GuildRank::Freelancers, cost);
    state.account.layers.push(LayerState {
        layer_number: 3,
        familiarity: 0.5,
        cleared: false,
        infrastructure: Vec::new(),
    });

    let result = can_upgrade_guild(&state);
    assert_eq!(result, Err("Required layer has not been cleared"));
}

#[test]
fn can_upgrade_guild_succeeds_with_all_conditions_met() {
    let state = state_ready_to_upgrade(GuildRank::Freelancers);
    assert!(can_upgrade_guild(&state).is_ok());
}

#[test]
fn can_upgrade_guild_succeeds_for_each_upgradable_rank() {
    let upgradable = [
        GuildRank::Freelancers,
        GuildRank::Sellswords,
        GuildRank::Company,
        GuildRank::Battalion,
    ];
    for rank in upgradable {
        let state = state_ready_to_upgrade(rank);
        assert!(
            can_upgrade_guild(&state).is_ok(),
            "Expected Ok for {:?}",
            rank
        );
    }
}

// =========================================================================
// upgrade_guild
// =========================================================================

#[test]
fn upgrade_guild_deducts_marks_and_advances_rank() {
    let mut state = state_ready_to_upgrade(GuildRank::Freelancers);
    let cost = GuildRank::Freelancers.upgrade_cost().unwrap();
    let before_marks = state.run.warband_marks;

    let new_rank = upgrade_guild(&mut state).unwrap();

    assert_eq!(new_rank, GuildRank::Sellswords);
    assert_eq!(state.account.guild_rank, GuildRank::Sellswords);
    assert_eq!(state.run.warband_marks, before_marks - cost);
}

#[test]
fn upgrade_guild_advances_through_all_ranks() {
    let progression = [
        (GuildRank::Freelancers, GuildRank::Sellswords),
        (GuildRank::Sellswords, GuildRank::Company),
        (GuildRank::Company, GuildRank::Battalion),
        (GuildRank::Battalion, GuildRank::Legion),
    ];

    for (from, expected_to) in progression {
        let mut state = state_ready_to_upgrade(from);
        let new_rank = upgrade_guild(&mut state).unwrap();
        assert_eq!(
            new_rank, expected_to,
            "Expected {:?} after upgrading from {:?}",
            expected_to, from
        );
        assert_eq!(state.account.guild_rank, expected_to);
    }
}

#[test]
fn upgrade_guild_fails_at_max_rank() {
    let mut state = state_with_rank_and_marks(GuildRank::Legion, 999_999);
    let result = upgrade_guild(&mut state);
    assert_eq!(result, Err("Already at maximum guild rank (Legion)"));
    assert_eq!(state.account.guild_rank, GuildRank::Legion);
    assert_eq!(state.run.warband_marks, 999_999, "Marks should not change");
}

#[test]
fn upgrade_guild_fails_without_enough_marks() {
    let cost = GuildRank::Freelancers.upgrade_cost().unwrap();
    let mut state = state_with_rank_and_marks(GuildRank::Freelancers, cost - 1);
    state.account.layers.push(cleared_layer(3));

    let result = upgrade_guild(&mut state);
    assert_eq!(result, Err("Insufficient Warband Marks for guild upgrade"));
    assert_eq!(state.account.guild_rank, GuildRank::Freelancers);
    assert_eq!(state.run.warband_marks, cost - 1);
}

#[test]
fn upgrade_guild_fails_without_required_layer_cleared() {
    let cost = GuildRank::Freelancers.upgrade_cost().unwrap();
    let mut state = state_with_rank_and_marks(GuildRank::Freelancers, cost);
    // Required layer 3 not added

    let result = upgrade_guild(&mut state);
    assert_eq!(result, Err("Required layer has not been cleared"));
    assert_eq!(state.account.guild_rank, GuildRank::Freelancers);
}

// =========================================================================
// guild_upgrade_progress
// =========================================================================

#[test]
fn guild_upgrade_progress_at_freelancers() {
    let cost = GuildRank::Freelancers.upgrade_cost().unwrap();
    let mut state = state_with_rank_and_marks(GuildRank::Freelancers, cost);
    state.account.layers.push(cleared_layer(3));

    let progress = guild_upgrade_progress(&state);

    assert_eq!(progress.current_rank, GuildRank::Freelancers);
    assert_eq!(progress.next_rank, Some(GuildRank::Sellswords));
    assert_eq!(progress.marks_needed, Some(cost));
    assert_eq!(progress.layer_needed, Some(3));
    assert!(progress.layer_cleared);
    assert_eq!(progress.marks_available, cost);
}

#[test]
fn guild_upgrade_progress_layer_not_cleared() {
    let cost = GuildRank::Freelancers.upgrade_cost().unwrap();
    let state = state_with_rank_and_marks(GuildRank::Freelancers, cost);
    // layer 3 not cleared

    let progress = guild_upgrade_progress(&state);

    assert_eq!(progress.layer_needed, Some(3));
    assert!(!progress.layer_cleared);
}

#[test]
fn guild_upgrade_progress_at_legion_max_rank() {
    let state = state_with_rank_and_marks(GuildRank::Legion, 50_000);

    let progress = guild_upgrade_progress(&state);

    assert_eq!(progress.current_rank, GuildRank::Legion);
    assert_eq!(progress.next_rank, None);
    assert_eq!(progress.marks_needed, None);
    assert_eq!(progress.layer_needed, None);
    assert!(
        !progress.layer_cleared,
        "layer_cleared is false when no layer needed"
    );
    assert_eq!(progress.marks_available, 50_000);
}

#[test]
fn guild_upgrade_progress_shows_correct_marks_available() {
    let state = state_with_rank_and_marks(GuildRank::Company, 3_000);

    let progress = guild_upgrade_progress(&state);

    assert_eq!(progress.marks_available, 3_000);
}

#[test]
fn guild_upgrade_progress_at_each_upgradable_rank() {
    let ranks = [
        (GuildRank::Freelancers, GuildRank::Sellswords),
        (GuildRank::Sellswords, GuildRank::Company),
        (GuildRank::Company, GuildRank::Battalion),
        (GuildRank::Battalion, GuildRank::Legion),
    ];

    for (current, expected_next) in ranks {
        let state = state_with_rank_and_marks(current, 0);
        let progress = guild_upgrade_progress(&state);
        assert_eq!(
            progress.next_rank,
            Some(expected_next),
            "Wrong next rank for {:?}",
            current
        );
        assert_eq!(
            progress.marks_needed,
            current.upgrade_cost(),
            "Wrong marks_needed for {:?}",
            current
        );
    }
}
