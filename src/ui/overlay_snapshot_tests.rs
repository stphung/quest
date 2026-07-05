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
            scene_play: None,
            scene_modal: None,
            moments: Default::default(),
        };
        render_overlay(|f| {
            let area = f.area();
            let ctx = LayoutContext::from_frame(f);
            super::voyage_scene::render_voyage(f, area, &voyage, &ui, &ctx, None);
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
            super::voyage_scene::render_voyage(f, area, &voyage, &ui, &ctx, None);
        })
    });
}

#[test]
fn snapshot_voyage_chart_dimming_mid_era() {
    // A ferry run well into the era: the old world has gone dark behind the
    // Vessel, ⊘ ports scattered across the chart, a lit path still ahead.
    assert_overlay_snapshot("voyage_chart_dimming_xl_160x45", || {
        let voyage = fixtures::voyage_mid_leg(frozen_utc());
        let colony = fixtures::colony_midera();
        let ui = crate::vessel::VoyageUiState::default();
        render_overlay(|f| {
            let area = f.area();
            let ctx = LayoutContext::from_frame(f);
            super::voyage_scene::render_voyage(f, area, &voyage, &ui, &ctx, Some(&colony));
        })
    });
}

#[test]
fn snapshot_voyage_trim_panel_mid_leg() {
    assert_overlay_snapshot("voyage_trim_xl_160x45", || {
        let voyage = fixtures::voyage_mid_leg(frozen_utc());
        let ui = crate::vessel::VoyageUiState {
            view: crate::vessel::VoyageView::Trim { selected: 2 },
            scene_play: None,
            scene_modal: None,
            moments: Default::default(),
        };
        render_overlay(|f| {
            let area = f.area();
            let ctx = LayoutContext::from_frame(f);
            super::voyage_scene::render_voyage(f, area, &voyage, &ui, &ctx, None);
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
            super::voyage_scene::render_voyage(f, area, &voyage, &ui, &ctx, None);
        })
    });
}

#[test]
fn snapshot_voyage_strip_ferry_run() {
    // A ferry run (crossing 2+) embarks a counted hold, so the gauge strip
    // gains a "Carrying N souls — bound for the Tree" line the maiden voyage
    // does not show.
    assert_overlay_snapshot("voyage_strip_ferry_m_60x24", || {
        let mut voyage = fixtures::voyage_mid_leg(frozen_utc());
        voyage.crossing_number = 2;
        voyage.passengers = 1238;
        let colony = fixtures::colony_midera();
        let ui = crate::vessel::VoyageUiState::default();
        render_overlay_sized(60, 24, |f| {
            let area = f.area();
            let ctx = LayoutContext::from_frame(f);
            super::voyage_scene::render_voyage(f, area, &voyage, &ui, &ctx, Some(&colony));
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
            super::voyage_scene::render_voyage(f, area, &voyage, &ui, &ctx, None);
        })
    });
}

#[test]
fn snapshot_voyage_souls_panel() {
    assert_overlay_snapshot("voyage_souls_xl_160x45", || {
        let mut voyage = fixtures::voyage_at_first_junction(frozen_utc());
        // Torvald at the helm so the panel shows a post, a paused arc, and
        // resting souls side by side.
        voyage.set_station(
            crate::vessel::souls::SoulId(0),
            Some(crate::vessel::souls::Station::Helm),
        );
        let ui = crate::vessel::VoyageUiState {
            view: crate::vessel::VoyageView::Souls { selected: 1 },
            scene_play: None,
            scene_modal: None,
            moments: Default::default(),
        };
        render_overlay(|f| {
            let area = f.area();
            let ctx = LayoutContext::from_frame(f);
            super::voyage_scene::render_voyage(f, area, &voyage, &ui, &ctx, None);
        })
    });
}

#[test]
fn snapshot_voyage_boarding_ask() {
    assert_overlay_snapshot("voyage_ask_xl_160x45", || {
        let mut voyage = fixtures::voyage_at_first_junction(frozen_utc());
        // Stage Sefa's ask so the modal renders over the chart.
        voyage.pending_ask = Some(crate::vessel::souls::SoulId(4));
        let ui = crate::vessel::VoyageUiState::default();
        render_overlay(|f| {
            let area = f.area();
            let ctx = LayoutContext::from_frame(f);
            super::voyage_scene::render_voyage(f, area, &voyage, &ui, &ctx, None);
        })
    });
}

#[test]
fn snapshot_voyage_scene_playback() {
    assert_overlay_snapshot("voyage_scene_playback_xl_160x45", || {
        let mut voyage = fixtures::voyage_at_first_junction(frozen_utc());
        // Re-stage the Markets arrival unplayed and read its scene.
        voyage.phase = crate::vessel::voyage::VoyagePhase::HoldingStation {
            waypoint: crate::vessel::route::WaypointId(2),
            arrived_at_min: voyage.processed_minutes,
            scene_state: crate::vessel::voyage::SceneState::Waiting,
            arrived_by: Some(crate::vessel::route::RoadId(1)),
        };
        let playback = voyage.play_arrival_scene().expect("scene plays");
        let ui = crate::vessel::VoyageUiState {
            view: crate::vessel::VoyageView::Chart,
            scene_play: Some(crate::vessel::ScenePlay { playback, index: 0 }),
            scene_modal: None,
            moments: Default::default(),
        };
        render_overlay(|f| {
            let area = f.area();
            let ctx = LayoutContext::from_frame(f);
            super::voyage_scene::render_voyage(f, area, &voyage, &ui, &ctx, None);
        })
    });
}

#[test]
fn snapshot_voyage_refit_door() {
    assert_overlay_snapshot("voyage_refit_xl_160x45", || {
        let mut voyage = fixtures::voyage_at_first_junction(frozen_utc());
        voyage.pending_refit = Some(0);
        let ui = crate::vessel::VoyageUiState::default();
        render_overlay(|f| {
            let area = f.area();
            let ctx = LayoutContext::from_frame(f);
            super::voyage_scene::render_voyage(f, area, &voyage, &ui, &ctx, None);
        })
    });
}

// ── The harbor (spec 7) ─────────────────────────────────────────────────────

fn arrived_voyage() -> crate::vessel::voyage::VoyageState {
    fixtures::voyage_arrived("fixture-voyager".to_string(), frozen_utc())
}

#[test]
fn snapshot_voyage_harbor() {
    assert_overlay_snapshot("voyage_harbor_xl_160x45", || {
        let voyage = arrived_voyage();
        let ui = crate::vessel::VoyageUiState::default();
        render_overlay(|f| {
            let area = f.area();
            let ctx = LayoutContext::from_frame(f);
            super::voyage_scene::render_voyage(f, area, &voyage, &ui, &ctx, None);
        })
    });
}

#[test]
fn snapshot_voyage_harbor_strip() {
    assert_overlay_snapshot("voyage_harbor_strip_m_60x24", || {
        let voyage = arrived_voyage();
        let ui = crate::vessel::VoyageUiState::default();
        render_overlay_sized(60, 24, |f| {
            let area = f.area();
            let ctx = LayoutContext::from_frame(f);
            super::voyage_scene::render_voyage(f, area, &voyage, &ui, &ctx, None);
        })
    });
}

#[test]
fn snapshot_voyage_reckoning_early() {
    // A few crossings in, all three yards still affordable — the state where
    // the yard comparison is a live decision.
    assert_overlay_snapshot("voyage_reckoning_early_xl_160x45", || {
        let voyage = fixtures::voyage_mid_leg(frozen_utc());
        let colony = fixtures::colony_early();
        let ui = crate::vessel::VoyageUiState {
            view: crate::vessel::VoyageView::Reckoning,
            scene_play: None,
            scene_modal: None,
            moments: Default::default(),
        };
        render_overlay(|f| {
            let area = f.area();
            let ctx = LayoutContext::from_frame(f);
            super::voyage_scene::render_voyage(f, area, &voyage, &ui, &ctx, Some(&colony));
        })
    });
}

#[test]
fn snapshot_voyage_reckoning() {
    assert_overlay_snapshot("voyage_reckoning_xl_160x45", || {
        let voyage = fixtures::voyage_mid_leg(frozen_utc());
        let colony = fixtures::colony_midera();
        let ui = crate::vessel::VoyageUiState {
            view: crate::vessel::VoyageView::Reckoning,
            scene_play: None,
            scene_modal: None,
            moments: Default::default(),
        };
        render_overlay(|f| {
            let area = f.area();
            let ctx = LayoutContext::from_frame(f);
            super::voyage_scene::render_voyage(f, area, &voyage, &ui, &ctx, Some(&colony));
        })
    });
}

#[test]
fn snapshot_voyage_dock_just_docked() {
    // A crossing just delivered; Riftglass hasn't charged at all yet.
    assert_overlay_snapshot("voyage_dock_just_docked_xl_160x45", || {
        let voyage = fixtures::voyage_mid_leg(frozen_utc());
        let mut colony = fixtures::colony_midera();
        colony.dock(frozen_utc());
        let ui = crate::vessel::VoyageUiState {
            view: crate::vessel::VoyageView::Dock {
                confirm_pending: false,
            },
            scene_play: None,
            scene_modal: None,
            moments: Default::default(),
        };
        render_overlay(|f| {
            let area = f.area();
            let ctx = LayoutContext::from_frame(f);
            super::voyage_scene::render_voyage(f, area, &voyage, &ui, &ctx, Some(&colony));
        })
    });
}

#[test]
fn snapshot_voyage_dock_jump_confirmation() {
    // The one-way, no-undo jump commitment's second key press.
    assert_overlay_snapshot("voyage_dock_confirm_xl_160x45", || {
        let voyage = fixtures::voyage_mid_leg(frozen_utc());
        let mut colony = fixtures::colony_midera();
        colony.dock(frozen_utc());
        let ui = crate::vessel::VoyageUiState {
            view: crate::vessel::VoyageView::Dock {
                confirm_pending: true,
            },
            scene_play: None,
            scene_modal: None,
            moments: Default::default(),
        };
        render_overlay(|f| {
            let area = f.area();
            let ctx = LayoutContext::from_frame(f);
            super::voyage_scene::render_voyage(f, area, &voyage, &ui, &ctx, Some(&colony));
        })
    });
}

#[test]
fn snapshot_voyage_manifest() {
    assert_overlay_snapshot("voyage_manifest_xl_160x45", || {
        let voyage = arrived_voyage();
        let ui = crate::vessel::VoyageUiState {
            view: crate::vessel::VoyageView::Manifest { scroll: 0 },
            scene_play: None,
            scene_modal: None,
            moments: Default::default(),
        };
        render_overlay(|f| {
            let area = f.area();
            let ctx = LayoutContext::from_frame(f);
            super::voyage_scene::render_voyage(f, area, &voyage, &ui, &ctx, None);
        })
    });
}

#[test]
fn snapshot_voyage_keepsake_chart() {
    assert_overlay_snapshot("voyage_keepsake_xl_160x45", || {
        let voyage = arrived_voyage();
        let (x, y) = crate::vessel::route::waypoint(crate::vessel::route::ROUTE_SINK).chart_pos;
        let ui = crate::vessel::VoyageUiState {
            view: crate::vessel::VoyageView::Keepsake { x, y },
            scene_play: None,
            scene_modal: None,
            moments: Default::default(),
        };
        render_overlay(|f| {
            let area = f.area();
            let ctx = LayoutContext::from_frame(f);
            super::voyage_scene::render_voyage(f, area, &voyage, &ui, &ctx, None);
        })
    });
}

#[test]
fn snapshot_voyage_record() {
    assert_overlay_snapshot("voyage_record_xl_160x45", || {
        let voyage = arrived_voyage();
        let ui = crate::vessel::VoyageUiState {
            view: crate::vessel::VoyageView::Record { scroll: 0 },
            scene_play: None,
            scene_modal: None,
            moments: Default::default(),
        };
        render_overlay(|f| {
            let area = f.area();
            let ctx = LayoutContext::from_frame(f);
            super::voyage_scene::render_voyage(f, area, &voyage, &ui, &ctx, None);
        })
    });
}

/// No Right Path, rule 3, as a test: pan the keepsake chart across the
/// whole canvas and assert no unvisited, un-crossed-out waypoint's name
/// ever reaches the frame. The fog outlives the crossing.
#[test]
fn keepsake_chart_never_reveals_the_fog() {
    use crate::vessel::route;
    let _clock = clock::freeze_at_millis(FROZEN_MILLIS);
    let voyage = arrived_voyage();
    let visited: std::collections::HashSet<_> = voyage.visited.iter().copied().collect();
    let untaken_dests: std::collections::HashSet<_> =
        voyage.untaken.iter().map(|r| route::road(*r).to).collect();

    for (cx, cy) in [(0u16, 0u16), (30, 22), (60, 45), (90, 67), (119, 89)] {
        let lines = super::voyage_scene::chart_lines_centered(
            &voyage,
            100,
            44,
            cx,
            cy,
            &std::collections::HashSet::new(),
        );
        let text: String = lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.clone()))
            .collect();
        for wp in &route::WAYPOINTS {
            if !visited.contains(&wp.id) && !untaken_dests.contains(&wp.id) {
                assert!(
                    !text.contains(wp.name),
                    "the fog broke at ({cx},{cy}): {} was never seen and \
                     must never be named",
                    wp.name
                );
            }
        }
    }
}

#[test]
fn snapshot_voyage_watch_panel() {
    assert_overlay_snapshot("voyage_watch_xl_160x45", || {
        let mut voyage = fixtures::voyage_mid_leg(frozen_utc());
        voyage.set_station(
            crate::vessel::souls::SoulId(2),
            Some(crate::vessel::souls::Station::Watch),
        );
        let ui = crate::vessel::VoyageUiState {
            view: crate::vessel::VoyageView::Watch { selected: 1 },
            scene_play: None,
            scene_modal: None,
            moments: Default::default(),
        };
        render_overlay(|f| {
            let area = f.area();
            let ctx = LayoutContext::from_frame(f);
            super::voyage_scene::render_voyage(f, area, &voyage, &ui, &ctx, None);
        })
    });
}

#[test]
fn snapshot_launch_transition_first_beat() {
    assert_overlay_snapshot("launch_transition_beat1_xl_160x45", || {
        render_overlay(|f| {
            let area = f.area();
            super::vessel_scene::render_launch_transition(f, area, 1);
        })
    });
}

#[test]
fn snapshot_launch_transition_final_beat() {
    assert_overlay_snapshot("launch_transition_beat5_xl_160x45", || {
        render_overlay(|f| {
            let area = f.area();
            super::vessel_scene::render_launch_transition(
                f,
                area,
                crate::vessel::transition::BEAT_COUNT,
            );
        })
    });
}
