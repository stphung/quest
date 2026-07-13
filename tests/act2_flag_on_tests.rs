//! Flag-ON smoke tests for the Act 2 kill-switch integration surface.
//!
//! EVERY test in this binary runs with Act 2 ENABLED: `enable_act2()` sets
//! `QUEST_ACT2=1` and primes `vessel::act2_enabled()`'s `OnceLock` before
//! anything else in this process can cache it dark. Do not add tests here
//! that assume the kill-switch is off — they belong in the ordinary test
//! binaries, which all run dark. (This process-wide split exists because
//! the `OnceLock` cache makes the flag untoggleable within one process.)
//!
//! What this binary covers vs. what stays elsewhere is mapped in
//! `src/vessel/CLAUDE.md` ("Flag-ON test map").

use quest::core::tick::game_tick_with_context;
use quest::core::tick_context::TickContext;
use rand::SeedableRng;
use std::sync::Once;

/// Enable Act 2 for this whole process, before the `OnceLock` can cache
/// the dark default. Call first in every test.
fn enable_act2() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        std::env::set_var("QUEST_ACT2", "1");
        assert!(
            quest::vessel::act2_enabled(),
            "QUEST_ACT2=1 must enable Act 2 (was act2_enabled() called before set_var?)"
        );
    });
}

/// Run one full game tick against a minimal context.
fn tick(state: &mut quest::GameState, rng: &mut rand_chacha::ChaCha8Rng) -> quest::TickResult {
    let mut tick_counter = 0u32;
    let mut haven = quest::Haven::new();
    let mut enhancement = quest::EnhancementProgress::default();
    let mut deep = quest::deep::DeepState::new();
    let mut ach = quest::Achievements::default();
    let mut loom = quest::loom::LoomState::new();
    let mut ctx = TickContext {
        state,
        tick_counter: &mut tick_counter,
        haven: &mut haven,
        enhancement: &mut enhancement,
        deep: &mut deep,
        achievements: &mut ach,
        loom: &mut loom,
        debug_mode: true,
    };
    game_tick_with_context(&mut ctx, rng)
}

/// The whisper stage (tick Stage 12c) is gated on `act2_enabled()` at the
/// call site in `core/tick.rs` — with the flag on, a discovered signal must
/// produce `VesselWhisper` events through the REAL tick loop, and they must
/// stop once the Vessel launches.
#[test]
fn whispers_flow_through_the_full_tick_loop() {
    enable_act2();
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(11);
    let mut state = quest::GameState::new("FlagOn".to_string(), 0);
    state.vessel_signal_discovered = true;
    state.vessel_last_whisper_at = 0;
    state.play_time_seconds = quest::vessel::WHISPER_INTERVAL_SECONDS;

    let result = tick(&mut state, &mut rng);
    assert!(
        result
            .events
            .iter()
            .any(|e| matches!(e, quest::TickEvent::VesselWhisper { .. })),
        "with the kill-switch on, the whisper stage runs inside the full tick"
    );

    // After launch the whispers go quiet, still through the full loop.
    state.vessel_launched = true;
    state.play_time_seconds += 10 * quest::vessel::WHISPER_INTERVAL_SECONDS;
    let result = tick(&mut state, &mut rng);
    assert!(
        !result
            .events
            .iter()
            .any(|e| matches!(e, quest::TickEvent::VesselWhisper { .. })),
        "a launched Vessel whispers no more"
    );
}

/// The launch gate end-to-end with the flag on: all four prerequisites,
/// then the all-or-nothing burn.
#[test]
fn launch_gate_and_burn_with_flag_on() {
    enable_act2();
    let mut state = quest::GameState::new("FlagOn".to_string(), 0);
    state.vessel_signal_discovered = true;
    state.ascension_level = 10;
    state.prestige_rank = quest::vessel::LAUNCH_PR_COST + 123;

    assert!(
        !quest::vessel::can_launch(&state, 27),
        "27 patterns is not 28"
    );
    assert!(quest::vessel::can_launch(&state, 28));

    assert!(quest::vessel::perform_launch(&mut state, 28));
    assert!(state.vessel_launched);
    assert_eq!(
        state.prestige_rank, 123,
        "the burn subtracts exactly LAUNCH_PR_COST"
    );
    assert!(
        !quest::vessel::perform_launch(&mut state, 28),
        "no double launch"
    );
}

/// Discovery stays exactly-once with the flag on: the unconditional Zone 50
/// detection and the gated presentation compose — a discovered state ticked
/// further emits no second `VesselSignalDiscovered`.
#[test]
fn discovery_does_not_refire_with_flag_on() {
    enable_act2();
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(5);
    let mut state = quest::GameState::new("FlagOn".to_string(), 0);
    state.vessel_signal_discovered = true;

    for _ in 0..100 {
        let result = tick(&mut state, &mut rng);
        assert!(
            !result
                .events
                .iter()
                .any(|e| matches!(e, quest::TickEvent::VesselSignalDiscovered)),
            "an already-discovered signal never refires"
        );
    }
}

/// With the flag on, the Vessel achievements become visible everywhere
/// the dark-side test (`vessel_visibility_tests` in the lib) asserts
/// they are hidden.
#[test]
fn vessel_achievements_are_visible_with_flag_on() {
    enable_act2();
    use quest::achievements::{
        get_achievements_by_category, AchievementCategory, AchievementId, Achievements,
    };
    assert!(
        get_achievements_by_category(AchievementCategory::Progression)
            .iter()
            .any(|a| a.id == AchievementId::TheBurn),
        "the browser lists The Burn once Act 2 is enabled"
    );
    let a = Achievements::default();
    assert_eq!(a.total_count(), 247, "all seven join the total");
}
