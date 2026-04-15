use criterion::{criterion_group, criterion_main, Criterion};
use quest::achievements::Achievements;
use quest::character::derived_stats::DerivedStats;
use quest::core::game_state::GameState;
use quest::core::tick::game_tick_with_context;
use quest::core::tick_context::TickContext;
use quest::enhancement::EnhancementProgress;
use quest::haven::Haven;
use quest::loom::LoomState;
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
    LoomState,
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
    let loom = LoomState::new();

    (state, haven, enhancement, deep, achievements, loom)
}

fn bench_game_tick(c: &mut Criterion) {
    let mut group = c.benchmark_group("game_tick");

    group.bench_function("early_L1_P0", |b| {
        let (mut state, mut haven, mut enhancement, mut deep, mut ach, mut loom) = make_state(1, 0);
        let mut tick_counter = 0u32;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        b.iter(|| {
            let mut ctx = TickContext {
                state: &mut state,
                tick_counter: &mut tick_counter,
                haven: &mut haven,
                enhancement: &mut enhancement,
                deep: &mut deep,
                achievements: &mut ach,
                loom: &mut loom,
                debug_mode: false,
            };
            game_tick_with_context(&mut ctx, &mut rng)
        });
    });

    group.bench_function("mid_L50_P10", |b| {
        let (mut state, mut haven, mut enhancement, mut deep, mut ach, mut loom) =
            make_state(50, 10);
        let mut tick_counter = 0u32;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        b.iter(|| {
            let mut ctx = TickContext {
                state: &mut state,
                tick_counter: &mut tick_counter,
                haven: &mut haven,
                enhancement: &mut enhancement,
                deep: &mut deep,
                achievements: &mut ach,
                loom: &mut loom,
                debug_mode: false,
            };
            game_tick_with_context(&mut ctx, &mut rng)
        });
    });

    group.bench_function("endgame_L100_P50", |b| {
        let (mut state, mut haven, mut enhancement, mut deep, mut ach, mut loom) =
            make_state(100, 50);
        let mut tick_counter = 0u32;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        b.iter(|| {
            let mut ctx = TickContext {
                state: &mut state,
                tick_counter: &mut tick_counter,
                haven: &mut haven,
                enhancement: &mut enhancement,
                deep: &mut deep,
                achievements: &mut ach,
                loom: &mut loom,
                debug_mode: false,
            };
            game_tick_with_context(&mut ctx, &mut rng)
        });
    });

    group.finish();
}

/// Benchmark the achievement milestone check path — called every kill/level-up.
/// Uses the public `on_enemy_killed` handler which calls `check_milestones`
/// internally, exercising the early-exit fast path for already-unlocked entries.
fn bench_achievement_milestones(c: &mut Criterion) {
    let mut group = c.benchmark_group("achievement_milestones");

    // Mid-game steady state: 1500 kills, first three Slayer milestones already
    // unlocked (100, 500, 1000). Each call does 15 is_unlocked HashMap lookups
    // (fast early-exit) and one counter increment.
    group.bench_function("on_enemy_killed_mid_game_1500_kills", |b| {
        let mut ach = Achievements::default();
        // Advance to 1500 kills with the first milestones already unlocked
        for _ in 0..1500 {
            ach.on_enemy_killed(false, Some("Hero"));
        }
        b.iter(|| {
            ach.on_enemy_killed(false, Some("Hero"));
        });
    });

    // Boss kill path additionally traverses BOSS_HUNTER_MILESTONES.
    group.bench_function("on_enemy_killed_boss_1000_bosses", |b| {
        let mut ach = Achievements::default();
        for _ in 0..1000 {
            ach.on_enemy_killed(true, Some("Hero"));
        }
        b.iter(|| {
            ach.on_enemy_killed(true, Some("Hero"));
        });
    });

    group.finish();
}

/// Benchmark the combat event processing path — the hot path that fires
/// on every player attack, enemy attack, and kill event.
/// Runs 20 consecutive ticks to ensure at least one full attack cycle (1.5s = 15 ticks).
fn bench_combat_event_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("combat_event_processing");

    group.bench_function("20_ticks_mid_game_combat", |b| {
        let (mut state, mut haven, mut enhancement, mut deep, mut ach, mut loom) =
            make_state(50, 10);
        let mut tick_counter = 0u32;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        b.iter(|| {
            for _ in 0..20 {
                let mut ctx = TickContext {
                    state: &mut state,
                    tick_counter: &mut tick_counter,
                    haven: &mut haven,
                    enhancement: &mut enhancement,
                    deep: &mut deep,
                    achievements: &mut ach,
                    loom: &mut loom,
                    debug_mode: false,
                };
                let _ = game_tick_with_context(&mut ctx, &mut rng);
            }
        });
    });

    group.bench_function("20_ticks_endgame_combat", |b| {
        let (mut state, mut haven, mut enhancement, mut deep, mut ach, mut loom) =
            make_state(100, 50);
        let mut tick_counter = 0u32;
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        b.iter(|| {
            for _ in 0..20 {
                let mut ctx = TickContext {
                    state: &mut state,
                    tick_counter: &mut tick_counter,
                    haven: &mut haven,
                    enhancement: &mut enhancement,
                    deep: &mut deep,
                    achievements: &mut ach,
                    loom: &mut loom,
                    debug_mode: false,
                };
                let _ = game_tick_with_context(&mut ctx, &mut rng);
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_game_tick,
    bench_combat_event_processing,
    bench_achievement_milestones
);
criterion_main!(benches);
