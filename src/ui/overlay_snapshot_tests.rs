//! Snapshot tests for the full-screen overlay scenes.
//!
//! These scenes are dispatched from `main_helpers/overlay.rs`, not
//! `draw_ui_with_update()`, so each test calls the scene's public render
//! entry point directly with deterministic state under a frozen UI clock.
//!
//! Every assertion renders its scene TWICE from independently built state
//! and requires byte-identical frames before snapshotting — this catches
//! hidden mutable animation state (the Loom scene advances
//! `throbber_frame`/`particle_phases` during render, so a shared-state
//! double render would diverge; fresh state per render must not).
//!
//! Known exclusions:
//! - Deep `Roster`/`Recruit` sub-views: `deep_missions::render_roster`
//!   iterates the mercenary `HashMap` via `.values()`, so display order is
//!   nondeterministic. Snapshot them only after that iteration is sorted.
//! - Character-select splash: rendered by `main_helpers::update` (bin-only
//!   module) and reads `Utc::now()` directly; needs a clock-routing
//!   refactor first.
//! - Stormglass rolling phases (`InvokeTrialRolling`/`SigilRolling`):
//!   animate from `ExchangeUiState` wall-clock fields
//!   (`stormglass/types.rs` reads `SystemTime` directly), which the frozen
//!   UI clock does not cover. The Menu phase renders with those fields
//!   `None`.

use super::responsive::LayoutContext;
use super::time_vault_scene::TimeVaultState;
use super::{
    clock, deep_scene, haven_scene, loom_scene, soulforge_scene, stormglass_scene, time_vault_scene,
};
use crate::achievements::Achievements;
use crate::deep::{DeepUiState, DeepView};
use crate::enhancement::{
    EnhancementProgress, EnhancementResult, SoulforgePhase, SoulforgeUiState,
};
use crate::fixtures;
use crate::history::types::{CommitInfo, TimelineInfo};
use crate::loom::types::LoomUiState;
use crate::stormglass::types::ExchangeUiState;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use ratatui::{backend::TestBackend, Frame, Terminal};

/// Same frozen instant as `snapshot_tests.rs`.
const FROZEN_MILLIS: u64 = 1_750_000_000_123;
const CREATED_AT: i64 = 1_749_000_000;

fn frozen_utc() -> chrono::DateTime<chrono::Utc> {
    chrono::DateTime::from_timestamp_millis(FROZEN_MILLIS as i64).unwrap()
}

/// Renders one overlay frame at XL (160x45) under a frozen clock.
fn render_overlay(draw: impl FnOnce(&mut Frame)) -> String {
    let _clock = clock::freeze_at_millis(FROZEN_MILLIS);
    let mut terminal = Terminal::new(TestBackend::new(160, 45)).unwrap();
    terminal.draw(draw).unwrap();
    format!("{:?}", terminal.backend().buffer())
}

/// Builds the frame twice (fresh state each time via `make_frame`), asserts
/// the renders are byte-identical, then snapshots.
fn assert_overlay_snapshot(name: &str, mut make_frame: impl FnMut() -> String) {
    let frame = make_frame();
    assert_eq!(
        frame,
        make_frame(),
        "overlay '{name}' rendered differently from identical state — \
         nondeterministic input in the render path"
    );
    let mut settings = insta::Settings::clone_current();
    settings.set_prepend_module_to_snapshot(false);
    // Commit timestamps in the Time Vault are formatted in the host's local
    // timezone ("%b %d, %Y  %l:%M %p" — fixed width). Mask them so the same
    // snapshot passes in any timezone. The mask matches itself, so
    // re-filtering is idempotent.
    settings.add_filter(
        r"[A-Z][a-z]{2} \d{2}, \d{4}  [ \d]\d:\d{2} [AP]M",
        "Jan 01, 1970  12:00 AM",
    );
    settings.bind(|| insta::assert_snapshot!(name, frame));
}

#[test]
fn snapshot_haven_overlay() {
    assert_overlay_snapshot("overlay_haven_xl_160x45", || {
        let haven = fixtures::haven_built();
        let achievements = Achievements::default();
        render_overlay(|f| {
            let area = f.area();
            let ctx = LayoutContext::from_frame(f);
            haven_scene::render_haven_tree(f, area, &haven, 2, None, 25, &achievements, &ctx);
        })
    });
}

#[test]
fn snapshot_deep_overlay_missions() {
    assert_overlay_snapshot("overlay_deep_missions_xl_160x45", || {
        let deep = fixtures::deep_state_active(frozen_utc());
        let mut ui = DeepUiState::new();
        ui.view = DeepView::Active;
        render_overlay(|f| {
            let area = f.area();
            let ctx = LayoutContext::from_frame(f);
            deep_scene::render_deep_overlay(f, area, &deep, &ui, None, &ctx);
        })
    });
}

#[test]
fn snapshot_deep_overlay_infrastructure() {
    assert_overlay_snapshot("overlay_deep_infrastructure_xl_160x45", || {
        let deep = fixtures::deep_state_active(frozen_utc());
        let mut ui = DeepUiState::new();
        ui.view = DeepView::Infrastructure;
        render_overlay(|f| {
            let area = f.area();
            let ctx = LayoutContext::from_frame(f);
            deep_scene::render_deep_overlay(f, area, &deep, &ui, None, &ctx);
        })
    });
}

#[test]
fn snapshot_loom_overlay() {
    assert_overlay_snapshot("overlay_loom_xl_160x45", || {
        let mut loom = fixtures::loom_state_with_shuttle();
        let mut ui = LoomUiState::new();
        render_overlay(|f| {
            let area = f.area();
            loom_scene::render_loom_overlay(f, area, &mut loom, &mut ui, 2500);
        })
    });
}

#[test]
fn snapshot_soulforge_overlay_menu() {
    assert_overlay_snapshot("overlay_soulforge_menu_xl_160x45", || {
        let mut enhancement = EnhancementProgress::new();
        enhancement.discovered = true;
        enhancement.levels = [7, 5, 4, 3, 3, 2, 1];
        enhancement.highest_level_reached = 7;
        let mut ui = SoulforgeUiState::new();
        ui.open = true;
        ui.selected_slot = 2;
        render_overlay(|f| {
            let area = f.area();
            let ctx = LayoutContext::from_frame(f);
            soulforge_scene::render_soulforge(f, area, &ui, &enhancement, 25, &ctx);
        })
    });
}

#[test]
fn snapshot_soulforge_overlay_success() {
    assert_overlay_snapshot("overlay_soulforge_success_xl_160x45", || {
        let mut enhancement = EnhancementProgress::new();
        enhancement.discovered = true;
        enhancement.levels = [8, 5, 4, 3, 3, 2, 1];
        enhancement.highest_level_reached = 8;
        let mut ui = SoulforgeUiState::new();
        ui.open = true;
        ui.selected_slot = 0;
        ui.phase = SoulforgePhase::ResultSuccess;
        ui.animation_tick = 3;
        ui.last_result = Some(EnhancementResult {
            slot_index: 0,
            success: true,
            old_level: 7,
            new_level: 8,
            cost: 3,
        });
        render_overlay(|f| {
            let area = f.area();
            let ctx = LayoutContext::from_frame(f);
            soulforge_scene::render_soulforge(f, area, &ui, &enhancement, 25, &ctx);
        })
    });
}

#[test]
fn snapshot_stormglass_overlay_menu() {
    assert_overlay_snapshot("overlay_stormglass_menu_xl_160x45", || {
        let state = fixtures::midgame("Stormglass", CREATED_AT, &mut ChaCha8Rng::seed_from_u64(42));
        let mut ui = ExchangeUiState::new();
        ui.open = true;
        render_overlay(|f| {
            let area = f.area();
            let ctx = LayoutContext::from_frame(f);
            stormglass_scene::render_stormglass_exchange(f, area, &ui, &state, &ctx);
        })
    });
}

#[test]
fn snapshot_time_vault_overlay() {
    fn commit(id: &str, message: &str, age_secs: i64, level: u32, prestige: u32) -> CommitInfo {
        CommitInfo {
            id: id.to_string(),
            message: message.to_string(),
            timestamp: CREATED_AT + age_secs,
            level,
            prestige,
            zone: 8,
            playtime: 60 * 60 * 30,
        }
    }

    assert_overlay_snapshot("overlay_time_vault_xl_160x45", || {
        let commits = vec![
            commit("a1b2c3d", "Reached Zone 8 boss", 500_000, 45, 5),
            commit("e4f5a6b", "Prestige 4 -> 5", 300_000, 1, 5),
            commit("c7d8e9f", "Epic weapon drop", 100_000, 38, 4),
        ];
        let branches = vec![
            TimelineInfo {
                name: "main".to_string(),
                is_active: true,
                head_commit: Some(commits[0].clone()),
            },
            TimelineInfo {
                name: "pre-prestige".to_string(),
                is_active: false,
                head_commit: Some(commits[2].clone()),
            },
        ];
        let vault = TimeVaultState::new(branches, commits);
        render_overlay(|f| {
            let area = f.area();
            let ctx = LayoutContext::from_frame(f);
            time_vault_scene::draw_time_vault(f, area, &vault, &ctx);
        })
    });
}

/// Renders one overlay frame at an arbitrary size under a frozen clock
/// (the Act 2 strip layout needs a small tier).
fn render_overlay_sized(width: u16, height: u16, draw: impl FnOnce(&mut Frame)) -> String {
    let _clock = clock::freeze_at_millis(FROZEN_MILLIS);
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(draw).unwrap();
    format!("{:?}", terminal.backend().buffer())
}

#[test]
fn snapshot_voyage_junction_cards() {
    assert_overlay_snapshot("voyage_junction_xl_160x45", || {
        let voyage = fixtures::voyage_at_first_junction(frozen_utc());
        let ui = crate::vessel::VoyageUiState {
            view: crate::vessel::VoyageView::Junction { selected: 0 },
            scene_modal: None,
        };
        render_overlay(|f| {
            let area = f.area();
            let ctx = LayoutContext::from_frame(f);
            super::voyage_scene::render_voyage(f, area, &voyage, &ui, &ctx);
        })
    });
}

#[test]
fn snapshot_voyage_chart_mid_leg() {
    assert_overlay_snapshot("voyage_chart_mid_leg_xl_160x45", || {
        let voyage = fixtures::voyage_mid_leg(frozen_utc());
        let ui = crate::vessel::VoyageUiState::default();
        render_overlay(|f| {
            let area = f.area();
            let ctx = LayoutContext::from_frame(f);
            super::voyage_scene::render_voyage(f, area, &voyage, &ui, &ctx);
        })
    });
}

#[test]
fn snapshot_voyage_trim_panel_mid_leg() {
    assert_overlay_snapshot("voyage_trim_xl_160x45", || {
        let voyage = fixtures::voyage_mid_leg(frozen_utc());
        let ui = crate::vessel::VoyageUiState {
            view: crate::vessel::VoyageView::Trim { selected: 2 },
            scene_modal: None,
        };
        render_overlay(|f| {
            let area = f.area();
            let ctx = LayoutContext::from_frame(f);
            super::voyage_scene::render_voyage(f, area, &voyage, &ui, &ctx);
        })
    });
}

#[test]
fn snapshot_voyage_strip_small_tier() {
    assert_overlay_snapshot("voyage_strip_m_60x24", || {
        let voyage = fixtures::voyage_mid_leg(frozen_utc());
        let ui = crate::vessel::VoyageUiState::default();
        render_overlay_sized(60, 24, |f| {
            let area = f.area();
            let ctx = LayoutContext::from_frame(f);
            super::voyage_scene::render_voyage(f, area, &voyage, &ui, &ctx);
        })
    });
}

#[test]
fn snapshot_voyage_intro() {
    assert_overlay_snapshot("voyage_intro_xl_160x45", || {
        let mut voyage = fixtures::voyage_at_first_junction(frozen_utc());
        voyage.intro_pending = true;
        let ui = crate::vessel::VoyageUiState::default();
        render_overlay(|f| {
            let area = f.area();
            let ctx = LayoutContext::from_frame(f);
            super::voyage_scene::render_voyage(f, area, &voyage, &ui, &ctx);
        })
    });
}
