//! End-to-end test for the Vessel launch gate (Act 2 sub-project 1):
//! a real Zone 50 final-boss kill through the full tick pipeline must
//! discover the Vessel signal exactly once.

use quest::core::tick::game_tick_with_context;
use quest::core::tick_context::TickContext;
use rand::SeedableRng;

#[test]
fn z50_final_boss_kill_discovers_vessel_signal_through_full_tick() {
    let mut state = quest::GameState::new("Probe".to_string(), 0);
    state.prestige_rank = 103_218;
    state.ascension_level = 10;
    // Level + synced HP matter: a level-1 hero survives the Z50 zone boss
    // only on lucky rolls, which makes the fight outcome flaky.
    state.character_level = 900;
    quest::fixtures::set_attributes(&mut state, 516_000);
    quest::fixtures::sync_hp(&mut state);
    let mut gear_rng = rand_chacha::ChaCha8Rng::seed_from_u64(3);
    quest::fixtures::equip_all(
        &mut state,
        quest::Rarity::Mythic,
        quest::Rarity::Mythic,
        500,
        &mut gear_rng,
    );
    quest::fixtures::advance_to_zone(&mut state, 50, 5);
    state.zone_progression.kills_in_subzone = 10;

    let mut loom = quest::loom::LoomState::new();
    quest::loom::initialize_loom(&mut loom);
    quest::loom::complete_discovery(&mut loom);
    for p in loom.persistent.patterns.iter_mut().take(28) {
        p.completed = true;
    }

    let mut tick_counter = 0u32;
    let mut haven = quest::Haven::new();
    let mut enhancement = quest::EnhancementProgress::default();
    let mut deep = quest::deep::DeepState::new();
    let mut ach = quest::Achievements::default();
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(7);

    let mut signal_events = 0usize;
    for _ in 0..20_000 {
        let mut ctx = TickContext {
            state: &mut state,
            tick_counter: &mut tick_counter,
            haven: &mut haven,
            enhancement: &mut enhancement,
            deep: &mut deep,
            achievements: &mut ach,
            loom: &mut loom,
            debug_mode: true,
        };
        let result = game_tick_with_context(&mut ctx, &mut rng);
        signal_events += result
            .events
            .iter()
            .filter(|e| matches!(e, quest::TickEvent::VesselSignalDiscovered))
            .count();
    }

    assert!(
        state.vessel_signal_discovered,
        "Z50 final boss kill should discover the Vessel signal"
    );
    assert_eq!(
        signal_events, 1,
        "signal discovery should fire exactly once even as the zone keeps cycling"
    );
}
