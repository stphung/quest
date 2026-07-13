//! Randomized input fuzzing: feed [`InputHarness`] long sequences of
//! unscripted key presses across the key scenarios of Act 1 — fresh start,
//! mid-progression, prestige-ready, boss encounter, active dungeon, active
//! fishing session, every major system discovered at once, and all 14
//! minigames — and assert only that `handle_game_input` (and the
//! base-screen renderer) never panics.
//!
//! [`replay_tests`](super::replay_tests) proves specific scripted paths
//! behave correctly; this file complements it by hammering paths nobody
//! thought to script — the same value a `drive-game` session gets from a
//! human mashing keys, except exhaustive and reproducible from a seed. A
//! failure prints the scenario and seed needed to reproduce it deterministically.
//!
//! `w`/`W` and `!` are deliberately excluded from the key pool: they shell
//! out to a real browser/clipboard process
//! (`bug_report::open_browser`/`copy_to_clipboard`) and have no place in a
//! hermetic fuzz loop.

use super::harness::InputHarness;
use crate::fixtures;
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;
use ratatui::crossterm::event::KeyCode;
use std::panic::{self, AssertUnwindSafe};

const FROZEN_CREATED_AT: i64 = 1_749_000_000;
const KEYS_PER_RUN: usize = 200;
const SEEDS_PER_SCENARIO: u64 = 12;
const SEEDS_PER_MINIGAME: u64 = 3;

/// Key universe fed to the dispatcher. Navigation keys are repeated so they
/// dominate the distribution — that's what actually drives menus, overlays,
/// and minigames deep enough to matter, rather than bouncing off the base
/// game's hotkey gate every time. Excludes `w`/`W`/`!` (see module docs).
fn key_pool() -> Vec<KeyCode> {
    let mut pool = Vec::new();
    for _ in 0..6 {
        pool.extend([
            KeyCode::Up,
            KeyCode::Down,
            KeyCode::Left,
            KeyCode::Right,
            KeyCode::Enter,
            KeyCode::Esc,
            KeyCode::Tab,
            KeyCode::Char(' '),
        ]);
    }
    for c in "hHsSgGdDaAtTuUvVlLpP".chars() {
        pool.push(KeyCode::Char(c));
    }
    for c in "0123456789".chars() {
        pool.push(KeyCode::Char(c));
    }
    for c in ('a'..='z').chain('A'..='Z') {
        if c != 'w' && c != 'W' {
            pool.push(KeyCode::Char(c));
        }
    }
    pool.extend([
        KeyCode::Backspace,
        KeyCode::Home,
        KeyCode::End,
        KeyCode::Delete,
    ]);
    pool
}

/// Feed `harness` `KEYS_PER_RUN` random presses from `pool` (seeded by
/// `seed`), then render the base screen once. Returns `Err` with a short
/// repro description on the first panic instead of propagating it, so the
/// caller can run every seed and report every failure at once rather than
/// stopping at the first.
fn fuzz_run(mut harness: InputHarness, pool: &[KeyCode], seed: u64) -> Result<(), String> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        for _ in 0..KEYS_PER_RUN {
            let key = pool[rng.random_range(0..pool.len())];
            harness.press(key);
        }
        // A key can corrupt state into something that only panics when
        // drawn, not when dispatched — exercise the renderer too.
        harness.render(160, 45);
    }));
    result.map_err(|e| {
        let msg = e
            .downcast_ref::<&str>()
            .map(|s| s.to_string())
            .or_else(|| e.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "<non-string panic payload>".to_string());
        format!("seed {seed}: {msg}")
    })
}

/// Run `build` over `SEEDS_PER_SCENARIO` seeds and fail with every
/// reproducing seed if any run panicked.
fn run_fuzz_suite(label: &str, build: impl Fn(u64) -> InputHarness) {
    let pool = key_pool();
    let failures: Vec<String> = (0..SEEDS_PER_SCENARIO)
        .filter_map(|seed| fuzz_run(build(seed), &pool, seed).err())
        .collect();
    assert!(
        failures.is_empty(),
        "input fuzzing found {} panic(s) in the '{label}' scenario:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

fn scenario_fresh(_seed: u64) -> InputHarness {
    InputHarness::new(fixtures::fresh("Fuzz", FROZEN_CREATED_AT))
}

fn scenario_boss(seed: u64) -> InputHarness {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    InputHarness::new(fixtures::boss("Fuzz", FROZEN_CREATED_AT, &mut rng))
}

/// Mid Act 1 progression (level 45, P5, Zone 8, rare/epic gear) — the bulk
/// of a real playthrough sits here, between the fresh start and the fully
/// discovered late game.
fn scenario_midgame(seed: u64) -> InputHarness {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    InputHarness::new(fixtures::midgame("Fuzz", FROZEN_CREATED_AT, &mut rng))
}

/// Level high enough that `[P]` reliably passes `can_prestige` and opens
/// `PrestigeConfirm`, so random Enter/Esc/y/n presses actually exercise the
/// confirm-or-cancel dialog instead of bouncing off the eligibility gate.
fn scenario_prestige_ready(seed: u64) -> InputHarness {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut h = InputHarness::new(fixtures::midgame("Fuzz", FROZEN_CREATED_AT, &mut rng));
    h.state.character_level = 999;
    h
}

/// An active dungeon underneath the base game — hotkeys (Haven, Soulforge,
/// etc.) can still be pressed mid-dungeon in the real game, since dungeon
/// progress is tick-driven rather than input-driven.
fn scenario_dungeon(seed: u64) -> InputHarness {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut h = InputHarness::new(fixtures::midgame("Fuzz", FROZEN_CREATED_AT, &mut rng));
    h.state.active_dungeon = Some(crate::dungeon::generation::generate_dungeon(
        h.state.character_level,
        h.state.prestige_rank,
        h.state.zone_progression.current_zone_id,
    ));
    h
}

/// An active fishing session underneath the base game — same rationale as
/// [`scenario_dungeon`]: fishing ticks in the background, so overlay hotkeys
/// must keep working while it's running.
fn scenario_fishing(seed: u64) -> InputHarness {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut h = InputHarness::new(fixtures::midgame("Fuzz", FROZEN_CREATED_AT, &mut rng));
    h.state.active_fishing = Some(crate::fishing::generation::generate_fishing_session(
        &mut rng,
    ));
    h
}

/// Every major system discovered and populated with real content, so
/// hotkeys open live overlays (Haven, Soulforge, Stormglass, Deep, Loom,
/// Achievements) instead of bouncing off a discovery gate — the scenario
/// most likely to reach deep, rarely-scripted overlay navigation code.
fn scenario_discovered(seed: u64) -> InputHarness {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let now = chrono::DateTime::from_timestamp(1_750_000_000, 0).unwrap();
    let mut h = InputHarness::new(fixtures::endgame("Fuzz", FROZEN_CREATED_AT, &mut rng));

    h.haven = fixtures::haven_built();
    h.haven_discovered = true;

    h.enhancement.discovered = true;
    h.enhancement.levels = [5; 7];
    h.enhancement_levels = [5; 7];
    h.soulforge_discovered = true;

    h.state.stormglass_discovered = true;

    h.deep_state = fixtures::deep_state_active(now);

    h.loom_state = fixtures::loom_state_with_shuttle();
    h.loom_discovered = true;

    h.achievements
        .unlock(crate::achievements::AchievementId::Level100, None);

    h
}

#[test]
fn fuzz_fresh_state_never_panics() {
    run_fuzz_suite("fresh", scenario_fresh);
}

#[test]
fn fuzz_boss_encounter_never_panics() {
    run_fuzz_suite("boss", scenario_boss);
}

#[test]
fn fuzz_midgame_never_panics() {
    run_fuzz_suite("midgame", scenario_midgame);
}

#[test]
fn fuzz_prestige_ready_never_panics() {
    run_fuzz_suite("prestige_ready", scenario_prestige_ready);
}

#[test]
fn fuzz_dungeon_active_never_panics() {
    run_fuzz_suite("dungeon", scenario_dungeon);
}

#[test]
fn fuzz_fishing_active_never_panics() {
    run_fuzz_suite("fishing", scenario_fishing);
}

#[test]
fn fuzz_discovered_systems_never_panic() {
    run_fuzz_suite("discovered", scenario_discovered);
}

#[test]
fn fuzz_active_minigames_never_panic() {
    let pool = key_pool();
    let mut failures = Vec::new();

    // `all_challenges` builds one instance of every minigame per call, so
    // build it once per seed and fuzz all 14 from that batch — calling it
    // once per (game, seed) pair would rebuild the other 13 games each time
    // for nothing.
    for seed in 0..SEEDS_PER_MINIGAME {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let games = fixtures::all_challenges(&mut rng);
        for (name, game) in games {
            let mut h = InputHarness::new(fixtures::midgame("Fuzz", FROZEN_CREATED_AT, &mut rng));
            h.state.active_minigame = Some(game);
            if let Err(msg) = fuzz_run(h, &pool, seed) {
                failures.push(format!("[{name}] {msg}"));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "input fuzzing found {} panic(s) across active minigames:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// ── Act 2: the Voyage (release-polish net) ────────────────────────────────
//
// The Crossing screen has its own dispatcher (`voyage_input`) outside
// `handle_game_input`, so the Act 1 fuzz above never reaches it. Same
// contract: hammer random keys across the voyage's key states and assert
// dispatch + rendering never panic. `Quit` results are ignored (the loop
// keeps mashing); the frame is rendered after every key like main.rs would.
#[test]
fn fuzz_voyage_input_never_panics() {
    use crate::input::voyage_input::handle_voyage_input;
    use crate::ui::responsive::LayoutContext;
    use crate::vessel::{VoyageUiState, VoyageView};
    use chrono::TimeZone;
    use ratatui::{backend::TestBackend, Terminal};

    type VoyageBuilder = Box<dyn Fn() -> crate::vessel::voyage::VoyageState>;
    let now = chrono::Utc.timestamp_millis_opt(1_750_000_000_123).unwrap();
    let scenarios: Vec<(&str, VoyageBuilder)> = vec![
        ("mid-leg", Box::new(move || fixtures::voyage_mid_leg(now))),
        (
            "junction",
            Box::new(move || fixtures::voyage_at_first_junction(now)),
        ),
        (
            "arrived",
            Box::new(move || fixtures::voyage_arrived("fuzz".to_string(), now)),
        ),
    ];
    for (label, build) in &scenarios {
        for seed in 0..SEEDS_PER_SCENARIO {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let pool = key_pool();
            let mut voyage = build();
            // Era-over colony on the arrived scenario stresses the
            // dock/reckoning/record post-era branches too.
            let colony = if *label == "arrived" && seed % 2 == 1 {
                let mut c = fixtures::colony_era_complete();
                c.character_id = "fuzz".to_string();
                Some(c)
            } else {
                let mut c = fixtures::colony_midera();
                c.character_id = "fuzz".to_string();
                Some(c)
            };
            let mut ui = VoyageUiState {
                view: VoyageView::Chart,
                scene_play: None,
                scene_modal: None,
                moments: Default::default(),
            };
            let result = panic::catch_unwind(AssertUnwindSafe(|| {
                let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
                for _ in 0..KEYS_PER_RUN {
                    let key = pool[rng.random_range(0..pool.len())];
                    let _ = handle_voyage_input(
                        ratatui::crossterm::event::KeyEvent::from(key),
                        &mut voyage,
                        &mut ui,
                    );
                    terminal
                        .draw(|f| {
                            let area = f.area();
                            let ctx = LayoutContext::from_frame(f);
                            crate::ui::voyage_scene::render_voyage(
                                f,
                                area,
                                &voyage,
                                &ui,
                                &ctx,
                                colony.as_ref(),
                            );
                        })
                        .unwrap();
                }
            }));
            assert!(
                result.is_ok(),
                "voyage fuzz panicked: scenario '{label}', seed {seed} — rerun with this seed"
            );
        }
    }
}
