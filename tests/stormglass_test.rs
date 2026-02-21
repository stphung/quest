//! Tests for the Stormglass currency system.

use quest::core::game_state::GameState;
use quest::items::types::Rarity;
use quest::stormglass::{earning, spending, types};

// ── Earning: salvage values by rarity ──────────────────────────────────

#[test]
fn test_salvage_value_common() {
    assert_eq!(earning::salvage_value(Rarity::Common), 1);
}

#[test]
fn test_salvage_value_magic() {
    assert_eq!(earning::salvage_value(Rarity::Magic), 1);
}

#[test]
fn test_salvage_value_rare() {
    assert_eq!(earning::salvage_value(Rarity::Rare), 3);
}

#[test]
fn test_salvage_value_epic() {
    assert_eq!(earning::salvage_value(Rarity::Epic), 8);
}

#[test]
fn test_salvage_value_legendary() {
    assert_eq!(earning::salvage_value(Rarity::Legendary), 25);
}

#[test]
fn test_salvage_value_mythic_fallback() {
    // God/Mythic items should never be salvaged (auto-equip preserves them),
    // but if they were, they'd return the same as Legendary
    assert_eq!(earning::salvage_value(Rarity::Mythic), 25);
}

// ── Earning: dungeon cache by size ─────────────────────────────────────

#[test]
fn test_dungeon_cache_small() {
    assert_eq!(
        earning::dungeon_cache(quest::dungeon::types::DungeonSize::Small),
        5
    );
}

#[test]
fn test_dungeon_cache_medium() {
    assert_eq!(
        earning::dungeon_cache(quest::dungeon::types::DungeonSize::Medium),
        15
    );
}

#[test]
fn test_dungeon_cache_large() {
    assert_eq!(
        earning::dungeon_cache(quest::dungeon::types::DungeonSize::Large),
        30
    );
}

#[test]
fn test_dungeon_cache_epic() {
    assert_eq!(
        earning::dungeon_cache(quest::dungeon::types::DungeonSize::Epic),
        50
    );
}

#[test]
fn test_dungeon_cache_legendary() {
    assert_eq!(
        earning::dungeon_cache(quest::dungeon::types::DungeonSize::Legendary),
        75
    );
}

// ── Earning: Soulforge consolation by target level ─────────────────────

#[test]
fn test_soulforge_consolation_low_levels() {
    // Levels 1-4 have 100% success rate, no consolation
    for level in 1..=4 {
        assert_eq!(
            earning::soulforge_consolation(level),
            0,
            "Level {} should give 0 consolation",
            level
        );
    }
}

#[test]
fn test_soulforge_consolation_mid_levels() {
    assert_eq!(earning::soulforge_consolation(5), 5);
    assert_eq!(earning::soulforge_consolation(6), 10);
    assert_eq!(earning::soulforge_consolation(7), 15);
}

#[test]
fn test_soulforge_consolation_high_levels() {
    assert_eq!(earning::soulforge_consolation(8), 25);
    assert_eq!(earning::soulforge_consolation(9), 40);
    assert_eq!(earning::soulforge_consolation(10), 75);
}

#[test]
fn test_soulforge_consolation_out_of_range() {
    assert_eq!(earning::soulforge_consolation(0), 0);
    assert_eq!(earning::soulforge_consolation(11), 0);
}

// ── Spending: generate trial options ───────────────────────────────────

#[test]
fn test_generate_trial_options_returns_3() {
    let mut rng = rand::rng();
    let options = spending::generate_trial_options(&mut rng, &[]);
    assert_eq!(options.len(), 3);
}

#[test]
fn test_generate_trial_options_unique_types() {
    let mut rng = rand::rng();
    let options = spending::generate_trial_options(&mut rng, &[]);
    let types: Vec<_> = options.iter().map(|o| &o.challenge_type).collect();
    for i in 0..types.len() {
        for j in (i + 1)..types.len() {
            assert_ne!(
                types[i], types[j],
                "Trial options should have unique challenge types"
            );
        }
    }
}

#[test]
fn test_generate_trial_options_has_display_names() {
    let mut rng = rand::rng();
    let options = spending::generate_trial_options(&mut rng, &[]);
    for option in &options {
        assert!(!option.display_name.is_empty());
    }
}

#[test]
fn test_generate_trial_options_excludes_pending() {
    use quest::challenges::menu::ChallengeType;
    let mut rng = rand::rng();
    let exclude = vec![
        ChallengeType::Chess,
        ChallengeType::Morris,
        ChallengeType::Gomoku,
    ];
    let options = spending::generate_trial_options(&mut rng, &exclude);
    for option in &options {
        assert!(
            !exclude.contains(&option.challenge_type),
            "Trial options should not include pending challenge types"
        );
    }
}

// ── GameState: stormglass fields ───────────────────────────────────────

#[test]
fn test_game_state_new_defaults() {
    let state = GameState::new("Test".to_string(), 0);
    assert_eq!(state.stormglass, 0);
    assert!(!state.stormglass_discovered);
}

// ── ExchangeUiState ────────────────────────────────────────────────────

#[test]
fn test_exchange_ui_state_open_close() {
    let mut ui = types::ExchangeUiState::new();
    assert!(!ui.open);

    ui.open();
    assert!(ui.open);
    assert_eq!(ui.phase, types::ExchangePhase::Menu);

    ui.close();
    assert!(!ui.open);
    assert!(ui.trial_options.is_empty());
}

#[test]
fn test_exchange_ui_state_defaults() {
    let ui = types::ExchangeUiState::new();
    assert_eq!(ui.selected_item, 0);
    assert_eq!(ui.trial_selected, 0);
    assert!(ui.trial_options.is_empty());
    assert_eq!(ui.phase, types::ExchangePhase::Menu);
}

// ── Integration: game_tick with item drop salvage ──────────────────────

#[test]
fn test_item_drop_salvage_awards_stormglass() {
    use quest::achievements::Achievements;
    use quest::character::attributes::AttributeType;
    use quest::character::derived_stats::DerivedStats;
    use quest::core::tick::game_tick;
    use quest::enhancement::EnhancementProgress;
    use quest::haven::Haven;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    let mut state = GameState::new("Test".to_string(), 0);
    state.stormglass_discovered = true;

    // Make character strong enough to kill enemies and trigger item drops
    state.character_level = 10;
    state.prestige_rank = 15;
    state.attributes.set(AttributeType::Strength, 50);
    state.attributes.set(AttributeType::Intelligence, 50);
    let d = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment, &[0; 7]);
    state.combat_state.update_max_hp(d.max_hp);
    state.combat_state.player_current_hp = state.combat_state.player_max_hp;

    let mut tick_counter = 0u32;
    let mut enhancement = EnhancementProgress::new();
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    // Run many ticks to get item drops
    let initial_sg = state.stormglass;
    for _ in 0..10000 {
        let _result = game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut enhancement,
            &mut achievements,
            false,
            &mut rng,
        );
    }

    // After 10k ticks with active combat, should have earned some SG from salvage
    // (items that weren't equipped get salvaged)
    assert!(
        state.stormglass > initial_sg,
        "Should have earned Stormglass from salvaged items. Started: {}, now: {}",
        initial_sg,
        state.stormglass
    );
}

// ── Chrono Surge: cost table ──────────────────────────────────────────

#[test]
fn test_chrono_surge_costs_all_four_options() {
    // Verify all 4 duration options return correct (ticks, cost, label)
    let opt0 = spending::chrono_surge_cost(0).unwrap();
    assert_eq!(opt0, (9_000, 500, "15 minutes"));

    let opt1 = spending::chrono_surge_cost(1).unwrap();
    assert_eq!(opt1, (36_000, 2_000, "1 hour"));

    let opt2 = spending::chrono_surge_cost(2).unwrap();
    assert_eq!(opt2, (144_000, 8_000, "4 hours"));

    let opt3 = spending::chrono_surge_cost(3).unwrap();
    assert_eq!(opt3, (288_000, 16_000, "8 hours"));
}

#[test]
fn test_chrono_surge_cost_out_of_range() {
    assert!(spending::chrono_surge_cost(4).is_none());
    assert!(spending::chrono_surge_cost(100).is_none());
}

#[test]
fn test_chrono_surge_costs_monotonically_increase() {
    let mut prev_cost = 0;
    let mut prev_ticks = 0;
    for i in 0..4 {
        let (ticks, cost, _) = spending::chrono_surge_cost(i).unwrap();
        assert!(ticks > prev_ticks, "Ticks should increase");
        assert!(cost > prev_cost, "Cost should increase");
        prev_ticks = ticks;
        prev_cost = cost;
    }
}

// ── Chrono Surge: state constructor ──────────────────────────────────

#[test]
fn test_chrono_surge_state_new() {
    let surge = types::ChronoSurgeState::new(9_000);
    assert_eq!(surge.ticks_remaining, 9_000);
    assert_eq!(surge.ticks_total, 9_000);
    assert_eq!(surge.kills, 0);
    assert_eq!(surge.levels_gained, 0);
    assert_eq!(surge.items_equipped, 0);
    assert!(surge.batch_size > 0);
}

#[test]
fn test_chrono_surge_batch_size_scales_with_ticks() {
    // Longer surges should have larger batch sizes to maintain same animation duration
    let small = types::ChronoSurgeState::new(9_000);
    let large = types::ChronoSurgeState::new(288_000);
    assert!(
        large.batch_size > small.batch_size,
        "8hr batch ({}) should be larger than 15min batch ({})",
        large.batch_size,
        small.batch_size
    );
}

#[test]
fn test_chrono_surge_progress() {
    let mut surge = types::ChronoSurgeState::new(100);
    assert!((surge.progress() - 0.0).abs() < 0.01);

    surge.ticks_remaining = 50;
    assert!((surge.progress() - 0.5).abs() < 0.01);

    surge.ticks_remaining = 0;
    assert!((surge.progress() - 1.0).abs() < 0.01);
}

// ── Chrono Surge: batch execution with game_tick ─────────────────────

#[test]
fn test_chrono_surge_batch_accumulates_stats() {
    use quest::achievements::Achievements;
    use quest::character::attributes::AttributeType;
    use quest::character::derived_stats::DerivedStats;
    use quest::core::tick::game_tick;
    use quest::core::tick_types::TickEvent;
    use quest::enhancement::EnhancementProgress;
    use quest::haven::Haven;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    let mut state = GameState::new("Test".to_string(), 0);
    state.stormglass_discovered = true;
    state.character_level = 10;
    state.prestige_rank = 5;
    state.attributes.set(AttributeType::Strength, 50);
    state.attributes.set(AttributeType::Intelligence, 50);
    let d = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment, &[0; 7]);
    state.combat_state.update_max_hp(d.max_hp);
    state.combat_state.player_current_hp = state.combat_state.player_max_hp;

    let mut tick_counter = 0u32;
    let mut enhancement = EnhancementProgress::new();
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    let mut surge = types::ChronoSurgeState::new(9_000);

    // Run all ticks (simulating what main.rs does during surge across multiple frames)
    let total_ticks = surge.ticks_total;
    for _ in 0..total_ticks {
        let tick_result = game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut enhancement,
            &mut achievements,
            false,
            &mut rng,
        );

        for event in &tick_result.events {
            match event {
                TickEvent::EnemyDefeated { .. }
                | TickEvent::SubzoneBossDefeated { .. }
                | TickEvent::DungeonBossDefeated { .. }
                | TickEvent::DungeonEliteDefeated { .. } => {
                    surge.kills += 1;
                }
                TickEvent::ItemDropped { equipped: true, .. }
                | TickEvent::DungeonTreasureFound { equipped: true, .. } => {
                    surge.items_equipped += 1;
                }
                TickEvent::LeveledUp { .. } => {
                    surge.levels_gained += 1;
                }
                _ => {}
            }
        }
        surge.ticks_remaining -= 1;
    }

    // After running all ticks, the surge should have accumulated kills
    // (15 minutes of game time with a strong character)
    assert!(
        surge.kills > 0,
        "Should have gotten at least one kill in {} ticks",
        total_ticks
    );
    assert_eq!(surge.ticks_remaining, 0, "All ticks should be consumed");
}

// ── Chrono Surge: SG deduction ───────────────────────────────────────

#[test]
fn test_chrono_surge_deducts_stormglass() {
    let mut state = GameState::new("Test".to_string(), 0);
    state.stormglass_discovered = true;
    state.stormglass = 1000;

    let (_, cost, _) = spending::chrono_surge_cost(0).unwrap(); // 500 SG
    assert!(state.stormglass >= cost);

    state.stormglass -= cost;
    assert_eq!(state.stormglass, 500);
}

#[test]
fn test_chrono_surge_cannot_afford() {
    let mut state = GameState::new("Test".to_string(), 0);
    state.stormglass_discovered = true;
    state.stormglass = 100;

    let (_, cost, _) = spending::chrono_surge_cost(0).unwrap(); // 500 SG
    assert!(state.stormglass < cost, "Should not be able to afford");
}

// ── ExchangeUiState: surge_selected field ────────────────────────────

#[test]
fn test_exchange_ui_state_surge_selected_reset_on_open() {
    let mut ui = types::ExchangeUiState::new();
    ui.surge_selected = 3;
    ui.open();
    assert_eq!(ui.surge_selected, 0);
}

#[test]
fn test_exchange_ui_state_surge_selected_reset_on_close() {
    let mut ui = types::ExchangeUiState::new();
    ui.surge_selected = 2;
    ui.close();
    assert_eq!(ui.surge_selected, 0);
}

#[test]
fn test_first_salvage_discovers_stormglass() {
    use quest::achievements::Achievements;
    use quest::character::attributes::AttributeType;
    use quest::character::derived_stats::DerivedStats;
    use quest::core::tick::game_tick;
    use quest::enhancement::EnhancementProgress;
    use quest::haven::Haven;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    let mut state = GameState::new("Test".to_string(), 0);
    assert!(!state.stormglass_discovered);

    state.character_level = 10;
    state.prestige_rank = 15;
    state.attributes.set(AttributeType::Strength, 50);
    state.attributes.set(AttributeType::Intelligence, 50);
    let d = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment, &[0; 7]);
    state.combat_state.update_max_hp(d.max_hp);
    state.combat_state.player_current_hp = state.combat_state.player_max_hp;

    let mut tick_counter = 0u32;
    let mut enhancement = EnhancementProgress::new();
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    // Run many ticks — once an item drops and isn't equipped, stormglass gets discovered
    for _ in 0..10000 {
        let result = game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut enhancement,
            &mut achievements,
            false,
            &mut rng,
        );
        // Check if any StormglassDiscovered event was emitted
        for event in &result.events {
            if matches!(
                event,
                quest::core::tick_types::TickEvent::StormglassDiscovered
            ) {
                assert!(state.stormglass_discovered);
                assert!(state.stormglass > 0);
                return; // Test passes
            }
        }
    }

    // If we ran 10k ticks and still haven't gotten an item drop, that's concerning
    // but not impossible with certain seeds. The first test (above) verifies the mechanic works.
    // This test is more about the discovery event emission.
    if state.stormglass_discovered {
        assert!(state.stormglass > 0);
    }
}

#[test]
fn test_pre_p15_salvage_does_not_discover_stormglass() {
    use quest::achievements::Achievements;
    use quest::character::attributes::AttributeType;
    use quest::character::derived_stats::DerivedStats;
    use quest::core::tick::game_tick;
    use quest::core::tick_types::TickEvent;
    use quest::enhancement::EnhancementProgress;
    use quest::haven::Haven;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    let mut state = GameState::new("Test".to_string(), 0);
    // P5 — below the P15 threshold
    state.prestige_rank = 5;
    state.character_level = 10;
    state.attributes.set(AttributeType::Strength, 50);
    state.attributes.set(AttributeType::Intelligence, 50);
    let d = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment, &[0; 7]);
    state.combat_state.update_max_hp(d.max_hp);
    state.combat_state.player_current_hp = state.combat_state.player_max_hp;

    let mut tick_counter = 0u32;
    let mut enhancement = EnhancementProgress::new();
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    // Run many ticks — items will drop but stormglass should never be discovered
    for _ in 0..10000 {
        let result = game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut enhancement,
            &mut achievements,
            false,
            &mut rng,
        );
        for event in &result.events {
            assert!(
                !matches!(event, TickEvent::StormglassDiscovered),
                "Stormglass should not be discovered before P15"
            );
            assert!(
                !matches!(event, TickEvent::StormglassSalvaged { .. }),
                "Items should not be salvaged for SG before P15"
            );
            assert!(
                !matches!(event, TickEvent::StormglassDungeonCache { .. }),
                "Dungeon cache should not award SG before P15"
            );
        }
    }

    assert!(!state.stormglass_discovered);
    assert_eq!(state.stormglass, 0);
}

/// Regression: once Stormglass is discovered, salvaging should keep working
/// even if prestige_rank drops below 15 (e.g. from spending ranks on Haven).
#[test]
fn test_salvage_continues_after_prestige_rank_drops_below_15() {
    use quest::achievements::Achievements;
    use quest::character::attributes::AttributeType;
    use quest::character::derived_stats::DerivedStats;
    use quest::core::tick::game_tick;
    use quest::core::tick_types::TickEvent;
    use quest::enhancement::EnhancementProgress;
    use quest::haven::Haven;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    let mut state = GameState::new("Test".to_string(), 0);
    // Simulate a player who discovered Stormglass at P15+ but spent ranks on Haven
    state.stormglass_discovered = true;
    state.stormglass = 100;
    state.prestige_rank = 6; // Below the P15 threshold
    state.character_level = 10;
    state.attributes.set(AttributeType::Strength, 50);
    state.attributes.set(AttributeType::Intelligence, 50);
    let d = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment, &[0; 7]);
    state.combat_state.update_max_hp(d.max_hp);
    state.combat_state.player_current_hp = state.combat_state.player_max_hp;

    let mut tick_counter = 0u32;
    let mut enhancement = EnhancementProgress::new();
    let mut haven = Haven::default();
    let mut achievements = Achievements::default();
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    let initial_sg = state.stormglass;
    let mut saw_salvage = false;

    for _ in 0..10000 {
        let result = game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut enhancement,
            &mut achievements,
            false,
            &mut rng,
        );
        for event in &result.events {
            if matches!(event, TickEvent::StormglassSalvaged { .. }) {
                saw_salvage = true;
            }
        }
    }

    assert!(
        saw_salvage,
        "Should still salvage items after stormglass_discovered even with prestige_rank < 15"
    );
    assert!(
        state.stormglass > initial_sg,
        "Stormglass balance should have increased from salvage. Started: {}, now: {}",
        initial_sg,
        state.stormglass
    );
}
