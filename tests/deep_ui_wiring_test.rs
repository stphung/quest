//! QA-3: Integration tests for UI-related type changes, concurrent slot wiring,
//! and duration modifier computation.
//!
//! Tests cover:
//! - GuildRank::concurrent_missions() raw table values unchanged
//! - effective_concurrent_missions() bonus at Rank 1 with L3 breakthrough
//! - DurationModifiers computation for T1-1 effective duration display
//! - DeepUiState lifecycle (open, close, defaults, visit counters)
//! - DeepView tab navigation (next, prev, wrapping, EventResponse exclusion)

use quest::deep::{
    apply_duration_modifiers, effective_concurrent_missions, DurationModifiers, GuildRank,
    MIN_MISSION_DURATION_SECS,
};

// =========================================================================
// Concurrent Mission Wiring — raw GuildRank method unchanged
// =========================================================================

#[test]
fn test_guild_rank_concurrent_missions_table_values() {
    // The raw GuildRank::concurrent_missions() method should return
    // the base table values from GUILD_RANK_STATS, unmodified by any bonus.
    assert_eq!(GuildRank(1).concurrent_missions(), 1);
    assert_eq!(GuildRank(2).concurrent_missions(), 2);
    assert_eq!(GuildRank(3).concurrent_missions(), 2);
    assert_eq!(GuildRank(4).concurrent_missions(), 3);
    assert_eq!(GuildRank(5).concurrent_missions(), 4);
}

#[test]
fn test_effective_concurrent_rank1_no_breakthrough() {
    // Before L3 breakthrough, Rank 1 gets base 1 concurrent slot.
    assert_eq!(effective_concurrent_missions(GuildRank(1), 0), 1);
    assert_eq!(effective_concurrent_missions(GuildRank(1), 1), 1);
    assert_eq!(effective_concurrent_missions(GuildRank(1), 2), 1);
}

#[test]
fn test_effective_concurrent_rank1_with_l3_breakthrough() {
    // After L3 breakthrough, Rank 1 gets +1 bonus => 2 concurrent slots.
    assert_eq!(effective_concurrent_missions(GuildRank(1), 3), 2);
    assert_eq!(effective_concurrent_missions(GuildRank(1), 5), 2);
    assert_eq!(effective_concurrent_missions(GuildRank(1), 10), 2);
}

#[test]
fn test_effective_concurrent_higher_ranks_unaffected() {
    // Higher ranks should return their base concurrent mission count,
    // not affected by the L3 breakthrough bonus.
    assert_eq!(effective_concurrent_missions(GuildRank(2), 0), 2);
    assert_eq!(effective_concurrent_missions(GuildRank(2), 3), 2);
    assert_eq!(effective_concurrent_missions(GuildRank(2), 10), 2);
    assert_eq!(effective_concurrent_missions(GuildRank(3), 7), 2);
    assert_eq!(effective_concurrent_missions(GuildRank(4), 13), 3);
    assert_eq!(effective_concurrent_missions(GuildRank(5), 25), 4);
}

#[test]
fn test_effective_concurrent_rank1_boundary_layer2_vs_3() {
    // Layer 2 = no bonus, Layer 3 = bonus (exact boundary)
    assert_eq!(effective_concurrent_missions(GuildRank(1), 2), 1);
    assert_eq!(effective_concurrent_missions(GuildRank(1), 3), 2);
}

// =========================================================================
// Duration Modifier Computation (T1-1 display support)
// =========================================================================

#[test]
fn test_duration_mods_outpost_only() {
    let base = 7200u64; // 2 hours
    let mods = DurationModifiers {
        has_outpost: true,
        familiarity: 0,
        has_saboteur: false,
        saboteur_is_veteran: false,
        is_overpowered: false,
        bridge_layers: 0,
    };
    let effective = apply_duration_modifiers(base, &mods);
    // Outpost: *0.75 => 7200 * 0.75 = 5400
    assert_eq!(effective, 5400);
}

#[test]
fn test_duration_mods_outpost_and_mastered_familiarity() {
    let base = 7200u64; // 2 hours
    let mods = DurationModifiers {
        has_outpost: true,
        familiarity: 75, // Mastered => *0.70
        has_saboteur: false,
        saboteur_is_veteran: false,
        is_overpowered: false,
        bridge_layers: 0,
    };
    let effective = apply_duration_modifiers(base, &mods);
    // Outpost: *0.75, Mastered: *0.70 => 7200 * 0.75 * 0.70 = 3779 (f64 truncation)
    let expected = (7200.0_f64 * 0.75 * 0.70) as u64;
    assert_eq!(effective, expected);
}

#[test]
fn test_duration_mods_saboteur_base() {
    let base = 10000u64;
    let mods = DurationModifiers {
        has_outpost: false,
        familiarity: 0,
        has_saboteur: true,
        saboteur_is_veteran: false,
        is_overpowered: false,
        bridge_layers: 0,
    };
    let effective = apply_duration_modifiers(base, &mods);
    // Saboteur base: *0.90 => 10000 * 0.90 = 9000
    assert_eq!(effective, 9000);
}

#[test]
fn test_duration_mods_saboteur_veteran() {
    let base = 10000u64;
    let mods = DurationModifiers {
        has_outpost: false,
        familiarity: 0,
        has_saboteur: true,
        saboteur_is_veteran: true,
        is_overpowered: false,
        bridge_layers: 0,
    };
    let effective = apply_duration_modifiers(base, &mods);
    // Saboteur veteran: *0.85 => 10000 * 0.85 = 8500
    assert_eq!(effective, 8500);
}

#[test]
fn test_duration_mods_all_active_stacks_multiplicatively() {
    let base = 14400u64; // 4 hours
    let mods = DurationModifiers {
        has_outpost: true,  // *0.75
        familiarity: 100,   // Mastered *0.70
        has_saboteur: true, // *0.85 (veteran)
        saboteur_is_veteran: true,
        is_overpowered: true, // *0.90
        bridge_layers: 2,     // *0.80 (2 bridges at 10% each)
    };
    let effective = apply_duration_modifiers(base, &mods);
    let expected = (14400.0_f64 * 0.75 * 0.70 * 0.85 * 0.90 * 0.80) as u64;
    assert_eq!(effective, expected);
    assert!(effective < base);
    assert!(
        effective >= MIN_MISSION_DURATION_SECS,
        "Should respect MIN_MISSION_DURATION_SECS floor"
    );
}

#[test]
fn test_duration_mods_respects_minimum_floor() {
    let base = MIN_MISSION_DURATION_SECS;
    let mods = DurationModifiers {
        has_outpost: true,
        familiarity: 100,
        has_saboteur: true,
        saboteur_is_veteran: true,
        is_overpowered: true,
        bridge_layers: 5,
    };
    let effective = apply_duration_modifiers(base, &mods);
    assert_eq!(
        effective, MIN_MISSION_DURATION_SECS,
        "Duration must never go below MIN_MISSION_DURATION_SECS"
    );
}

#[test]
fn test_duration_mods_no_modifiers_returns_base() {
    let base = 21600u64; // 6 hours
    let mods = DurationModifiers {
        has_outpost: false,
        familiarity: 0,
        has_saboteur: false,
        saboteur_is_veteran: false,
        is_overpowered: false,
        bridge_layers: 0,
    };
    let effective = apply_duration_modifiers(base, &mods);
    assert_eq!(effective, base, "No modifiers should return base duration");
}

#[test]
fn test_duration_mods_bridge_cap_at_50_percent() {
    let base = 28800u64; // 8 hours
                         // 6 bridge layers would reduce by 60% without cap, but cap is 50%.
    let mods = DurationModifiers {
        has_outpost: false,
        familiarity: 0,
        has_saboteur: false,
        saboteur_is_veteran: false,
        is_overpowered: false,
        bridge_layers: 6,
    };
    let effective = apply_duration_modifiers(base, &mods);
    // Bridge capped at 50% => *0.50 => 28800 * 0.50 = 14400
    assert_eq!(effective, 14400);
}

#[test]
fn test_duration_mods_familiarity_mapped() {
    let base = 10000u64;
    let mods = DurationModifiers {
        has_outpost: false,
        familiarity: 30, // Mapped => *0.90
        has_saboteur: false,
        saboteur_is_veteran: false,
        is_overpowered: false,
        bridge_layers: 0,
    };
    let effective = apply_duration_modifiers(base, &mods);
    assert_eq!(effective, 9000); // 10000 * 0.90
}

#[test]
fn test_duration_mods_familiarity_familiar() {
    let base = 10000u64;
    let mods = DurationModifiers {
        has_outpost: false,
        familiarity: 55, // Familiar => *0.80
        has_saboteur: false,
        saboteur_is_veteran: false,
        is_overpowered: false,
        bridge_layers: 0,
    };
    let effective = apply_duration_modifiers(base, &mods);
    assert_eq!(effective, 8000); // 10000 * 0.80
}

#[test]
fn test_duration_mods_overpowered_alone() {
    let base = 10000u64;
    let mods = DurationModifiers {
        has_outpost: false,
        familiarity: 0,
        has_saboteur: false,
        saboteur_is_veteran: false,
        is_overpowered: true,
        bridge_layers: 0,
    };
    let effective = apply_duration_modifiers(base, &mods);
    assert_eq!(effective, 9000); // 10000 * 0.90
}

// =========================================================================
// DeepUiState lifecycle tests
// =========================================================================

#[test]
fn test_ui_state_defaults() {
    let ui = quest::deep::DeepUiState::new();
    assert!(!ui.open);
    assert_eq!(ui.view, quest::deep::DeepView::Hub);
    assert_eq!(ui.selected_index, 0);
    assert!(ui.event_mission_id.is_none());
    assert_eq!(ui.event_choice_index, 0);
    assert!(ui.staging_mission_index.is_none());
    assert!(ui.staged_squad.is_empty());
    assert!(ui.flash_message.is_none());
    assert_eq!(ui.hub_visit_count, 0);
    assert_eq!(ui.mission_visit_count, 0);
    assert_eq!(ui.roster_visit_count, 0);
    assert_eq!(ui.layer_visit_count, 0);
    assert_eq!(ui.event_visit_count, 0);
    assert_eq!(ui.recruit_visit_count, 0);
    assert!(!ui.show_help);
    assert!(ui.farewell_mercs.is_empty());
    assert!(ui.pending_story_stage.is_none());
}

#[test]
fn test_ui_state_open_sets_hub_and_increments_visit() {
    let mut ui = quest::deep::DeepUiState::new();
    ui.view = quest::deep::DeepView::Roster;
    ui.selected_index = 5;
    ui.flash_message = Some("old message".to_string());

    ui.open();

    assert!(ui.open);
    assert_eq!(ui.view, quest::deep::DeepView::Hub);
    assert_eq!(ui.selected_index, 0);
    assert!(ui.flash_message.is_none());
    assert_eq!(ui.hub_visit_count, 1);
}

#[test]
fn test_ui_state_open_visit_count_accumulates() {
    let mut ui = quest::deep::DeepUiState::new();
    ui.open();
    ui.open();
    ui.open();
    assert_eq!(ui.hub_visit_count, 3);
}

#[test]
fn test_ui_state_close_resets_state() {
    let mut ui = quest::deep::DeepUiState::new();
    ui.open();
    ui.view = quest::deep::DeepView::NewMission;
    ui.selected_index = 3;
    ui.staged_squad.push(42);
    ui.flash_message = Some("test".to_string());
    ui.show_help = true;

    ui.close();

    assert!(!ui.open);
    assert_eq!(ui.view, quest::deep::DeepView::Hub);
    assert_eq!(ui.selected_index, 0);
    assert!(ui.staged_squad.is_empty());
    assert!(ui.flash_message.is_none());
    assert!(!ui.show_help);
}

#[test]
fn test_ui_state_open_clears_staging() {
    let mut ui = quest::deep::DeepUiState::new();
    ui.staging_mission_index = Some(2);
    ui.staged_squad.push(1);
    ui.staged_squad.push(2);
    ui.event_mission_id = Some(99);
    ui.event_choice_index = 3;

    ui.open();

    assert!(ui.staging_mission_index.is_none());
    assert!(ui.staged_squad.is_empty());
    assert!(ui.event_mission_id.is_none());
    assert_eq!(ui.event_choice_index, 0);
}

// =========================================================================
// DeepView tab navigation
// =========================================================================

#[test]
fn test_view_tab_cycle_next() {
    use quest::deep::DeepView;
    let mut view = DeepView::Hub;
    view = view.next_tab();
    assert_eq!(view, DeepView::NewMission);
    view = view.next_tab();
    assert_eq!(view, DeepView::Roster);
    view = view.next_tab();
    assert_eq!(view, DeepView::Infrastructure);
    view = view.next_tab();
    assert_eq!(view, DeepView::Recruit);
    // Wraps around
    view = view.next_tab();
    assert_eq!(view, DeepView::Hub);
}

#[test]
fn test_view_tab_cycle_prev() {
    use quest::deep::DeepView;
    let mut view = DeepView::Hub;
    view = view.prev_tab();
    assert_eq!(view, DeepView::Recruit);
    view = view.prev_tab();
    assert_eq!(view, DeepView::Infrastructure);
    view = view.prev_tab();
    assert_eq!(view, DeepView::Roster);
    view = view.prev_tab();
    assert_eq!(view, DeepView::NewMission);
    view = view.prev_tab();
    assert_eq!(view, DeepView::Hub);
}

#[test]
fn test_view_event_response_not_in_tab_cycle() {
    use quest::deep::DeepView;
    // EventResponse is contextual, not in the tab cycle.
    let view = DeepView::EventResponse;
    assert_eq!(view.next_tab(), DeepView::EventResponse);
    assert_eq!(view.prev_tab(), DeepView::EventResponse);
}

#[test]
fn test_view_tab_labels() {
    use quest::deep::DeepView;
    assert_eq!(DeepView::Hub.tab_label(), "Hub");
    assert_eq!(DeepView::NewMission.tab_label(), "Missions");
    assert_eq!(DeepView::Roster.tab_label(), "Roster");
    assert_eq!(DeepView::Infrastructure.tab_label(), "Layers");
    assert_eq!(DeepView::EventResponse.tab_label(), "Event");
    assert_eq!(DeepView::Recruit.tab_label(), "Recruit");
}

#[test]
fn test_view_tabs_constant_excludes_event_response() {
    use quest::deep::DeepView;
    assert!(!DeepView::TABS.contains(&DeepView::EventResponse));
    assert!(DeepView::TABS.contains(&DeepView::Hub));
    assert!(DeepView::TABS.contains(&DeepView::Recruit));
    assert_eq!(DeepView::TABS.len(), 5);
}

#[test]
fn test_view_next_prev_are_inverses() {
    use quest::deep::DeepView;
    // For each tab, next_tab followed by prev_tab should return to the same tab.
    for &tab in DeepView::TABS {
        assert_eq!(
            tab.next_tab().prev_tab(),
            tab,
            "next_tab().prev_tab() should be identity for {:?}",
            tab
        );
        assert_eq!(
            tab.prev_tab().next_tab(),
            tab,
            "prev_tab().next_tab() should be identity for {:?}",
            tab
        );
    }
}

#[test]
fn test_farewell_mercs_stores_tuples() {
    let mut ui = quest::deep::DeepUiState::new();
    ui.farewell_mercs
        .push(("Gareth the Unyielding".to_string(), 5, 12));
    ui.farewell_mercs.push(("Lyra the Swift".to_string(), 3, 7));

    assert_eq!(ui.farewell_mercs.len(), 2);
    assert_eq!(ui.farewell_mercs[0].0, "Gareth the Unyielding");
    assert_eq!(ui.farewell_mercs[0].1, 5);
    assert_eq!(ui.farewell_mercs[0].2, 12);
    assert_eq!(ui.farewell_mercs[1].0, "Lyra the Swift");
    assert_eq!(ui.farewell_mercs[1].1, 3);
    assert_eq!(ui.farewell_mercs[1].2, 7);
}
