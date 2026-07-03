//! Keyboard input for the Act 2 Crossing screen.
//!
//! Routed from `main.rs` when the voyage owns the loop (after launch, behind
//! the kill-switch). All choice lives on the map: the only verbs are playing
//! the arrival scene, choosing a road, setting trim, and reading rumors.

use crate::vessel::route;
use crate::vessel::voyage::{SceneState, Trim, VoyagePhase, VoyageState};
use crate::vessel::{SceneModal, VoyageUiState, VoyageView};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

/// What the main loop should do after a key was seen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoyageInputResult {
    Handled,
    /// Departure, trim change, scene played: worth saving now.
    HandledNeedsSave,
    /// Save and leave the game.
    Quit,
    Ignored,
}

pub fn handle_voyage_input(
    key: KeyEvent,
    voyage: &mut VoyageState,
    ui: &mut VoyageUiState,
) -> VoyageInputResult {
    // The intro plays once, then never again.
    if voyage.intro_pending {
        if matches!(key.code, KeyCode::Enter | KeyCode::Char(' ')) {
            voyage.intro_pending = false;
            return VoyageInputResult::HandledNeedsSave;
        }
        return VoyageInputResult::Handled;
    }

    // A scene being read swallows everything except its close keys.
    // Closing it surfaces the next queued log moment, if any.
    if ui.scene_modal.is_some() {
        if matches!(key.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ')) {
            ui.scene_modal = ui.moments.pop_front();
        }
        return VoyageInputResult::Handled;
    }
    if let Some(moment) = ui.moments.pop_front() {
        ui.scene_modal = Some(moment);
        return VoyageInputResult::Handled;
    }

    // A boarding ask is a door: it must be answered before anything else
    // (except choosing who steps ashore when the berths are full).
    if voyage.pending_ask.is_some() && !matches!(ui.view, VoyageView::Farewell { .. }) {
        return handle_ask_keys(key, voyage, ui);
    }

    match ui.view {
        VoyageView::Chart => handle_chart_keys(key, voyage, ui),
        VoyageView::Junction { selected } => handle_junction_keys(key, voyage, ui, selected),
        VoyageView::Trim { selected } => handle_trim_keys(key, voyage, ui, selected),
        VoyageView::Souls { selected } => handle_souls_keys(key, voyage, ui, selected),
        VoyageView::Farewell { selected } => handle_farewell_keys(key, voyage, ui, selected),
        VoyageView::Rumors => {
            if matches!(
                key.code,
                KeyCode::Esc | KeyCode::Char('r') | KeyCode::Char('R')
            ) {
                ui.view = VoyageView::Chart;
            }
            VoyageInputResult::Handled
        }
    }
}

fn handle_ask_keys(
    key: KeyEvent,
    voyage: &mut VoyageState,
    ui: &mut VoyageUiState,
) -> VoyageInputResult {
    let Some(asking) = voyage.pending_ask else {
        return VoyageInputResult::Handled;
    };
    match key.code {
        KeyCode::Char('a') | KeyCode::Char('A') | KeyCode::Enter => {
            if voyage.accept_ask() {
                let def = crate::vessel::souls::soul(asking);
                ui.scene_modal = Some(SceneModal {
                    title: format!("{} comes aboard", def.name),
                    body: format!("\u{201c}{}\u{201d}", def.voice),
                });
                VoyageInputResult::HandledNeedsSave
            } else {
                // Berths full: someone must step ashore first.
                ui.view = VoyageView::Farewell { selected: 0 };
                VoyageInputResult::Handled
            }
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if let Some(declined) = voyage.decline_ask() {
                let def = crate::vessel::souls::soul(declined);
                ui.scene_modal = Some(SceneModal {
                    title: format!("{} stays behind", def.name),
                    body: "The door closes. The name stays on the chart.".to_string(),
                });
            }
            VoyageInputResult::HandledNeedsSave
        }
        _ => VoyageInputResult::Handled,
    }
}

fn handle_souls_keys(
    key: KeyEvent,
    voyage: &mut VoyageState,
    ui: &mut VoyageUiState,
    selected: usize,
) -> VoyageInputResult {
    let count = voyage.souls.len();
    if count == 0 {
        ui.view = VoyageView::Chart;
        return VoyageInputResult::Handled;
    }
    let selected = selected.min(count - 1);
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            ui.view = VoyageView::Souls {
                selected: selected.saturating_sub(1),
            };
            VoyageInputResult::Handled
        }
        KeyCode::Down | KeyCode::Char('j') => {
            ui.view = VoyageView::Souls {
                selected: (selected + 1).min(count - 1),
            };
            VoyageInputResult::Handled
        }
        // Enter cycles the selected soul's post: rest -> Helm -> Tender ->
        // Watch -> rest. Only aboard souls take a post.
        KeyCode::Enter => {
            use crate::vessel::souls::Station;
            let s = voyage.souls[selected];
            let next = match s.station {
                None => Some(Station::Helm),
                Some(Station::Helm) => Some(Station::Tender),
                Some(Station::Tender) => Some(Station::Watch),
                Some(Station::Watch) => None,
            };
            if voyage.set_station(s.soul, next) {
                VoyageInputResult::HandledNeedsSave
            } else {
                VoyageInputResult::Handled
            }
        }
        KeyCode::Esc | KeyCode::Char('s') | KeyCode::Char('S') => {
            ui.view = VoyageView::Chart;
            VoyageInputResult::Handled
        }
        _ => VoyageInputResult::Ignored,
    }
}

fn handle_farewell_keys(
    key: KeyEvent,
    voyage: &mut VoyageState,
    ui: &mut VoyageUiState,
    selected: usize,
) -> VoyageInputResult {
    let aboard: Vec<_> = voyage.aboard().map(|s| s.soul).collect();
    if aboard.is_empty() {
        ui.view = VoyageView::Chart;
        return VoyageInputResult::Handled;
    }
    let selected = selected.min(aboard.len() - 1);
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            ui.view = VoyageView::Farewell {
                selected: selected.saturating_sub(1),
            };
            VoyageInputResult::Handled
        }
        KeyCode::Down | KeyCode::Char('j') => {
            ui.view = VoyageView::Farewell {
                selected: (selected + 1).min(aboard.len() - 1),
            };
            VoyageInputResult::Handled
        }
        KeyCode::Enter => {
            let ashore = aboard[selected];
            if voyage.farewell(ashore) && voyage.accept_ask() {
                let ashore_def = crate::vessel::souls::soul(ashore);
                ui.scene_modal = Some(SceneModal {
                    title: format!("{} steps ashore", ashore_def.name),
                    body: "A full ship is a chosen ship. The manifest remembers.".to_string(),
                });
            }
            ui.view = VoyageView::Chart;
            VoyageInputResult::HandledNeedsSave
        }
        KeyCode::Esc => {
            // The ask is still pending; back to answering it.
            ui.view = VoyageView::Chart;
            VoyageInputResult::Handled
        }
        _ => VoyageInputResult::Ignored,
    }
}

fn handle_chart_keys(
    key: KeyEvent,
    voyage: &mut VoyageState,
    ui: &mut VoyageUiState,
) -> VoyageInputResult {
    match key.code {
        KeyCode::Enter => match voyage.phase {
            VoyagePhase::HoldingStation {
                waypoint,
                scene_state: SceneState::Waiting,
                ..
            } => {
                if let Some(scene) = voyage.play_arrival_scene() {
                    ui.scene_modal = Some(SceneModal {
                        title: route::waypoint(waypoint).name.to_string(),
                        body: scene.placeholder.to_string(),
                    });
                }
                VoyageInputResult::HandledNeedsSave
            }
            VoyagePhase::HoldingStation {
                scene_state: SceneState::Played,
                ..
            } => {
                ui.view = VoyageView::Junction { selected: 0 };
                VoyageInputResult::Handled
            }
            _ => VoyageInputResult::Handled,
        },
        KeyCode::Char('t') | KeyCode::Char('T') => {
            let current = Trim::ALL
                .iter()
                .position(|t| *t == voyage.trim)
                .unwrap_or(1);
            ui.view = VoyageView::Trim { selected: current };
            VoyageInputResult::Handled
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            ui.view = VoyageView::Rumors;
            VoyageInputResult::Handled
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            ui.view = VoyageView::Souls { selected: 0 };
            VoyageInputResult::Handled
        }
        KeyCode::Char('b') | KeyCode::Char('B') => {
            if let Some(bought) = voyage.buy_rumor() {
                ui.scene_modal = Some(SceneModal {
                    title: "A rumor, bought".to_string(),
                    body: route::rumor(bought).text.to_string(),
                });
                VoyageInputResult::HandledNeedsSave
            } else {
                VoyageInputResult::Handled
            }
        }
        KeyCode::Char('q') | KeyCode::Char('Q') => VoyageInputResult::Quit,
        _ => VoyageInputResult::Ignored,
    }
}

fn handle_junction_keys(
    key: KeyEvent,
    voyage: &mut VoyageState,
    ui: &mut VoyageUiState,
    selected: usize,
) -> VoyageInputResult {
    let cards = crate::vessel::junction::current_junction_cards(voyage);
    if cards.is_empty() {
        ui.view = VoyageView::Chart;
        return VoyageInputResult::Handled;
    }
    let selected = selected.min(cards.len() - 1);
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            ui.view = VoyageView::Junction {
                selected: selected.saturating_sub(1),
            };
            VoyageInputResult::Handled
        }
        KeyCode::Down | KeyCode::Char('j') => {
            ui.view = VoyageView::Junction {
                selected: (selected + 1).min(cards.len() - 1),
            };
            VoyageInputResult::Handled
        }
        KeyCode::Enter => {
            let road_id = cards[selected].road.id;
            if voyage.depart(road_id).is_ok() {
                ui.view = VoyageView::Chart;
                VoyageInputResult::HandledNeedsSave
            } else {
                VoyageInputResult::Handled
            }
        }
        KeyCode::Esc => {
            ui.view = VoyageView::Chart;
            VoyageInputResult::Handled
        }
        _ => VoyageInputResult::Ignored,
    }
}

fn handle_trim_keys(
    key: KeyEvent,
    voyage: &mut VoyageState,
    ui: &mut VoyageUiState,
    selected: usize,
) -> VoyageInputResult {
    let selected = selected.min(Trim::ALL.len() - 1);
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            ui.view = VoyageView::Trim {
                selected: selected.saturating_sub(1),
            };
            VoyageInputResult::Handled
        }
        KeyCode::Down | KeyCode::Char('j') => {
            ui.view = VoyageView::Trim {
                selected: (selected + 1).min(Trim::ALL.len() - 1),
            };
            VoyageInputResult::Handled
        }
        KeyCode::Enter => {
            voyage.set_trim(Trim::ALL[selected]);
            ui.view = VoyageView::Chart;
            VoyageInputResult::HandledNeedsSave
        }
        KeyCode::Esc => {
            ui.view = VoyageView::Chart;
            VoyageInputResult::Handled
        }
        _ => VoyageInputResult::Ignored,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vessel::route::ROUTE_START;
    use ratatui::crossterm::event::{KeyEventKind, KeyEventState, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn fresh() -> (VoyageState, VoyageUiState) {
        let now = "2026-07-03T12:00:00Z".parse().unwrap();
        let mut v = VoyageState::begin("c".to_string(), 1, now);
        v.intro_pending = false;
        (v, VoyageUiState::default())
    }

    #[test]
    fn enter_plays_the_arrival_scene_then_opens_the_junction() {
        let (mut v, mut ui) = fresh();
        // First Enter: go ashore — scene modal opens, scene marked played.
        let r = handle_voyage_input(key(KeyCode::Enter), &mut v, &mut ui);
        assert_eq!(r, VoyageInputResult::HandledNeedsSave);
        assert!(ui.scene_modal.is_some());
        // Close the modal.
        handle_voyage_input(key(KeyCode::Esc), &mut v, &mut ui);
        assert!(ui.scene_modal.is_none());
        // Second Enter: chart a course.
        handle_voyage_input(key(KeyCode::Enter), &mut v, &mut ui);
        assert_eq!(ui.view, VoyageView::Junction { selected: 0 });
        // Commit: the Last Harbor's one road out.
        let r = handle_voyage_input(key(KeyCode::Enter), &mut v, &mut ui);
        assert_eq!(r, VoyageInputResult::HandledNeedsSave);
        assert!(matches!(v.phase, VoyagePhase::Traveling { .. }));
        assert_eq!(ui.view, VoyageView::Chart);
    }

    #[test]
    fn intro_consumes_keys_until_cast_off() {
        let now = "2026-07-03T12:00:00Z".parse().unwrap();
        let mut v = VoyageState::begin("c".to_string(), 1, now);
        let mut ui = VoyageUiState::default();
        assert!(v.intro_pending);
        handle_voyage_input(key(KeyCode::Char('t')), &mut v, &mut ui);
        assert!(v.intro_pending, "only Enter casts off");
        assert_eq!(ui.view, VoyageView::Chart);
        let r = handle_voyage_input(key(KeyCode::Enter), &mut v, &mut ui);
        assert_eq!(r, VoyageInputResult::HandledNeedsSave);
        assert!(!v.intro_pending);
    }

    #[test]
    fn trim_panel_sets_the_posture() {
        let (mut v, mut ui) = fresh();
        handle_voyage_input(key(KeyCode::Char('t')), &mut v, &mut ui);
        assert_eq!(
            ui.view,
            VoyageView::Trim { selected: 1 },
            "opens on current"
        );
        handle_voyage_input(key(KeyCode::Down), &mut v, &mut ui);
        let r = handle_voyage_input(key(KeyCode::Enter), &mut v, &mut ui);
        assert_eq!(r, VoyageInputResult::HandledNeedsSave);
        assert_eq!(v.trim, Trim::Quiet);
        assert_eq!(ui.view, VoyageView::Chart);
    }

    #[test]
    fn quit_only_from_the_chart() {
        let (mut v, mut ui) = fresh();
        assert_eq!(
            handle_voyage_input(key(KeyCode::Char('q')), &mut v, &mut ui),
            VoyageInputResult::Quit
        );
        ui.view = VoyageView::Rumors;
        assert_ne!(
            handle_voyage_input(key(KeyCode::Char('q')), &mut v, &mut ui),
            VoyageInputResult::Quit
        );
        assert_eq!(v.current_waypoint(), Some(ROUTE_START));
    }
}
