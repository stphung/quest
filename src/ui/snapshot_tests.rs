//! Full-frame TUI snapshot tests.
//!
//! Renders `draw_ui_with_update()` into a ratatui `TestBackend` for every
//! size tier and a spread of fixture scenarios, then snapshots the buffer's
//! debug view (characters + style runs, so color regressions fail too) with
//! `insta`. The UI clock is frozen and item generation is seeded, so frames
//! are byte-identical across runs.
//!
//! When a test fails after an intentional UI change, review the diff and
//! re-bless with `INSTA_UPDATE=always cargo test -- snapshot` (or
//! `cargo insta review`), then eyeball the new frame with the `drive-game`
//! skill if the change is visual.

use super::{clock, draw_ui_with_update};
use crate::achievements::Achievements;
use crate::core::GameState;
use crate::deep::DeepState;
use crate::fixtures;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use ratatui::{backend::TestBackend, Terminal};

/// 2025-06-15T15:06:40Z — arbitrary but fixed; every spinner frame, pulse
/// phase, and countdown in a snapshot derives from this instant.
const FROZEN_MILLIS: u64 = 1_750_000_000_123;
/// Character creation time, comfortably before `FROZEN_MILLIS`.
const CREATED_AT: i64 = 1_749_000_000;
/// Fixed seed for generated gear (tiers, affixes, names).
const GEAR_SEED: u64 = 42;

fn gear_rng() -> ChaCha8Rng {
    ChaCha8Rng::seed_from_u64(GEAR_SEED)
}

/// The frozen instant as a chrono timestamp, for deriving fixture
/// timestamps (mission ETAs, core fill times) that must be time-relative.
fn frozen_utc() -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::from_timestamp_millis(FROZEN_MILLIS as i64).unwrap()
}

/// Everything `draw_ui_with_update()` reads besides the character state.
/// `Scene::new()` gives the bare setup (nothing discovered); tests flip
/// fields to exercise discovered-system rendering.
struct Scene<'a> {
    state: &'a GameState,
    deep: DeepState,
    achievements: Achievements,
    enhancement_levels: [u8; 7],
    haven_discovered: bool,
    soulforge_discovered: bool,
    loom_discovered: bool,
}

impl<'a> Scene<'a> {
    fn new(state: &'a GameState) -> Self {
        Self {
            state,
            deep: DeepState::new(),
            achievements: Achievements::default(),
            enhancement_levels: [0; 7],
            haven_discovered: false,
            soulforge_discovered: false,
            loom_discovered: false,
        }
    }
}

/// Renders one frame of the main game UI at the given terminal size and
/// returns the buffer's debug view.
fn render_scene(scene: &Scene, width: u16, height: u16) -> String {
    let _clock = clock::freeze_at_millis(FROZEN_MILLIS);
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal
        .draw(|frame| {
            draw_ui_with_update(
                frame,
                scene.state,
                None,  // update_info
                false, // update_check_completed
                false, // update_check_failed
                scene.haven_discovered,
                scene.soulforge_discovered,
                scene.state.stormglass_discovered,
                &scene.deep,
                scene.loom_discovered,
                &scene.achievements,
                &scene.enhancement_levels,
                &scene.deep,
            );
        })
        .unwrap();
    format!("{:?}", terminal.backend().buffer())
}

fn render_main(state: &GameState, width: u16, height: u16) -> String {
    render_scene(&Scene::new(state), width, height)
}

fn assert_frame_snapshot(name: &str, state: &GameState, width: u16, height: u16) {
    assert_scene_snapshot(name, &Scene::new(state), width, height);
}

fn assert_scene_snapshot(name: &str, scene: &Scene, width: u16, height: u16) {
    let frame = render_scene(scene, width, height);
    let mut settings = insta::Settings::clone_current();
    // Same snapshot files are asserted from both the lib and bin test
    // targets; a bare name keeps them shared and the directory readable.
    settings.set_prepend_module_to_snapshot(false);
    // The footer title shows BUILD_COMMIT (7 hex chars, or "unknown" in
    // clean checkouts). Mask it so snapshots don't churn on every commit;
    // both forms are 7 chars, so alignment is unaffected.
    settings.add_filter(r"┌ ([0-9a-f]{7}|unknown) ─", "┌ 0000000 ─");
    settings.bind(|| insta::assert_snapshot!(name, frame));
}

// ── Scenario × size-tier matrix ────────────────────────────────────────────
// Sizes pick one representative per tier (responsive.rs):
// XL 160x45, L 80x30, M 60x24, S 40x16, TooSmall 30x10.

#[test]
fn snapshot_fresh_all_tiers() {
    let state = fixtures::fresh("Fresh", CREATED_AT);
    assert_frame_snapshot("fresh_xl_160x45", &state, 160, 45);
    assert_frame_snapshot("fresh_l_80x30", &state, 80, 30);
    assert_frame_snapshot("fresh_m_60x24", &state, 60, 24);
    assert_frame_snapshot("fresh_s_40x16", &state, 40, 16);
    assert_frame_snapshot("fresh_too_small_30x10", &state, 30, 10);
}

#[test]
fn snapshot_midgame_all_tiers() {
    let state = fixtures::midgame("Midgame", CREATED_AT, &mut gear_rng());
    assert_frame_snapshot("midgame_xl_160x45", &state, 160, 45);
    assert_frame_snapshot("midgame_l_80x30", &state, 80, 30);
    assert_frame_snapshot("midgame_m_60x24", &state, 60, 24);
    assert_frame_snapshot("midgame_s_40x16", &state, 40, 16);
}

#[test]
fn snapshot_endgame() {
    let state = fixtures::endgame("Endgame", CREATED_AT, &mut gear_rng());
    assert_frame_snapshot("endgame_xl_160x45", &state, 160, 45);
    assert_frame_snapshot("endgame_l_80x30", &state, 80, 30);
}

#[test]
fn snapshot_boss_ready() {
    let state = fixtures::boss("Boss", CREATED_AT, &mut gear_rng());
    assert_frame_snapshot("boss_ready_xl_160x45", &state, 160, 45);
}

/// Mid-fight combat: enemy sprite and both HP bars partially filled —
/// the state the combat panel spends most of its life in.
#[test]
fn snapshot_combat_active() {
    let mut state = fixtures::midgame("Combat", CREATED_AT, &mut gear_rng());
    fixtures::engage_enemy(&mut state);
    assert_frame_snapshot("combat_xl_160x45", &state, 160, 45);
    assert_frame_snapshot("combat_m_60x24", &state, 60, 24);
}

/// All account-level systems visible at once: discovered Deep with a
/// running expedition, two Power Cores (one ready, one filling), Haven /
/// Soulforge / Loom discovery indicators, and enhanced gear.
#[test]
fn snapshot_systems_discovered() {
    let state = fixtures::endgame("Systems", CREATED_AT, &mut gear_rng());
    let mut scene = Scene::new(&state);
    scene.deep = fixtures::deep_state_active(frozen_utc());
    fixtures::unlock_power_cores(&mut scene.achievements, 2, frozen_utc().timestamp());
    scene.enhancement_levels = [7, 5, 4, 3, 3, 2, 1];
    scene.haven_discovered = true;
    scene.soulforge_discovered = true;
    scene.loom_discovered = true;
    assert_scene_snapshot("systems_discovered_xl_160x45", &scene, 160, 45);
    assert_scene_snapshot("systems_discovered_l_80x30", &scene, 80, 30);
}

/// Minigame overlay: an active chess challenge (deterministic initial
/// board) replaces the combat panel.
#[test]
fn snapshot_minigame_chess() {
    let mut state = fixtures::midgame("Chess", CREATED_AT, &mut gear_rng());
    fixtures::start_chess_challenge(&mut state);
    assert_frame_snapshot("minigame_chess_xl_160x45", &state, 160, 45);
}

/// Two renders of the same state must be byte-identical. If this fails, a
/// nondeterministic input (wall clock, thread RNG, iteration order) leaked
/// into the render path — fix that before touching any snapshot.
#[test]
fn snapshot_rendering_is_deterministic() {
    let a = {
        let state = fixtures::midgame("Midgame", CREATED_AT, &mut gear_rng());
        render_main(&state, 160, 45)
    };
    let b = {
        let state = fixtures::midgame("Midgame", CREATED_AT, &mut gear_rng());
        render_main(&state, 160, 45)
    };
    assert_eq!(a, b);
}
