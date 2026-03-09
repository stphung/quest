#![allow(deprecated)]
use criterion::{criterion_group, criterion_main, Criterion};
use quest::achievements::Achievements;
use quest::character::derived_stats::DerivedStats;
use quest::core::game_state::GameState;
use quest::core::tick::game_tick;
use quest::enhancement::EnhancementProgress;
use quest::haven::Haven;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Create a game state at a given level and prestige rank.
fn make_state(
    level: u32,
    prestige: u32,
) -> (
    GameState,
    Haven,
    EnhancementProgress,
    quest::deep::DeepState,
    Achievements,
) {
    let mut state = GameState::new("Bench".to_string(), 0);
    state.character_level = level;
    state.prestige_rank = prestige;

    let derived =
        DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment, &[0; 7]);
    state.combat_state.player_max_hp = derived.max_hp;
    state.combat_state.player_current_hp = derived.max_hp;

    let haven = Haven::default();
    let enhancement = EnhancementProgress::new();
    let deep = quest::deep::DeepState::new();
    let achievements = Achievements::default();

    (state, haven, enhancement, deep, achievements)
}

fn bench_game_tick(c: &mut Criterion) {
    let mut group = c.benchmark_group("game_tick");

    group.bench_function("early_L1_P0", |b| {
        let (mut state, mut haven, mut enhancement, mut deep, mut ach) = make_state(1, 0);
        let mut tick_counter = 0u32;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        b.iter(|| {
            game_tick(
                &mut state,
                &mut tick_counter,
                &mut haven,
                &mut enhancement,
                &mut deep,
                &mut ach,
                false,
                &mut rng,
            )
        });
    });

    group.bench_function("mid_L50_P10", |b| {
        let (mut state, mut haven, mut enhancement, mut deep, mut ach) = make_state(50, 10);
        let mut tick_counter = 0u32;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        b.iter(|| {
            game_tick(
                &mut state,
                &mut tick_counter,
                &mut haven,
                &mut enhancement,
                &mut deep,
                &mut ach,
                false,
                &mut rng,
            )
        });
    });

    group.bench_function("endgame_L100_P50", |b| {
        let (mut state, mut haven, mut enhancement, mut deep, mut ach) = make_state(100, 50);
        let mut tick_counter = 0u32;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        b.iter(|| {
            game_tick(
                &mut state,
                &mut tick_counter,
                &mut haven,
                &mut enhancement,
                &mut deep,
                &mut ach,
                false,
                &mut rng,
            )
        });
    });

    group.finish();
}

criterion_group!(benches, bench_game_tick);
criterion_main!(benches);
