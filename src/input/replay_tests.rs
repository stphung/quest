//! Input-replay tests: drive `handle_game_input` headlessly through
//! [`InputHarness`](super::harness::InputHarness) and assert on the resulting
//! state, the returned [`InputResult`], and (for a couple) the rendered frame.
//!
//! Before this harness `src/input/` had *zero* automated coverage — the only
//! way to exercise the keyboard dispatch chain was to launch the real game in
//! tmux and press keys by hand (the `drive-game` skill). These tests pin the
//! parts of the priority chain that are easy to get wrong in a refactor:
//! discovery-gated hotkeys, modal interception order, and Enter-only dismissal.

use super::harness::InputHarness;
use super::{GameOverlay, InputResult};
use crate::core::offline::OfflineReport;
use crate::fixtures;
use ratatui::crossterm::event::KeyCode;

/// A harness on a fresh hero with the Haven discovered, so the `[H]` hotkey is
/// live. Nothing else is unlocked.
fn haven_ready() -> InputHarness {
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.haven.discovered = true;
    h
}

#[test]
fn haven_hotkey_is_gated_on_discovery() {
    // Undiscovered: pressing [H] does nothing.
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    assert!(!h.haven.discovered);
    let result = h.char('h');
    assert!(!h.haven_ui.showing, "Haven opened without being discovered");
    assert_eq!(result, InputResult::Continue);

    // Discovered: pressing [H] opens the overlay; Esc closes it again.
    let mut h = haven_ready();
    h.char('h');
    assert!(h.haven_ui.showing, "[H] did not open the Haven overlay");
    h.press(KeyCode::Esc);
    assert!(!h.haven_ui.showing, "Esc did not close the Haven overlay");
}

#[test]
fn uppercase_hotkey_variant_also_opens_overlay() {
    let mut h = haven_ready();
    h.char('H');
    assert!(h.haven_ui.showing, "[Shift+H] should open the Haven too");
}

#[test]
fn soulforge_and_stormglass_hotkeys_are_gated_on_discovery() {
    // Soulforge: gated on `enhancement.discovered`.
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.char('s');
    assert!(!h.soulforge_ui.open, "Soulforge opened while undiscovered");
    h.enhancement.discovered = true;
    h.char('s');
    assert!(h.soulforge_ui.open, "[S] did not open the Soulforge");

    // Stormglass: gated on `state.stormglass_discovered`.
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.char('g');
    assert!(!h.exchange_ui.open, "Stormglass opened while undiscovered");
    h.state.stormglass_discovered = true;
    h.char('g');
    assert!(
        h.exchange_ui.open,
        "[G] did not open the Stormglass Exchange"
    );
}

#[test]
fn leviathan_modal_dismisses_only_on_enter() {
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.overlay = GameOverlay::LeviathanEncounter {
        encounter_number: 1,
        lure_consumed: false,
    };

    // A non-Enter key is swallowed but leaves the modal up.
    let result = h.char('x');
    assert_eq!(result, InputResult::Continue);
    assert!(
        matches!(h.overlay, GameOverlay::LeviathanEncounter { .. }),
        "non-Enter key should not dismiss the Leviathan modal"
    );

    // Enter dismisses it.
    h.press(KeyCode::Enter);
    assert!(
        h.overlay_is_none(),
        "Enter should dismiss the Leviathan modal"
    );
}

#[test]
fn active_modal_intercepts_background_hotkeys() {
    // Haven IS discovered, so [H] would normally open it...
    let mut h = haven_ready();
    // ...but a modal is up, and the priority chain must swallow the key first.
    h.overlay = GameOverlay::LeviathanEncounter {
        encounter_number: 1,
        lure_consumed: false,
    };

    h.char('h');
    assert!(
        !h.haven_ui.showing,
        "background [H] leaked through an active modal — priority chain broken"
    );
    assert!(matches!(h.overlay, GameOverlay::LeviathanEncounter { .. }));
}

#[test]
fn offline_welcome_dismisses_on_any_key() {
    // The offline-welcome modal (step 0) is dismissed by *any* key — unlike the
    // Enter-only Leviathan modals. Prove it across a few different keys.
    for code in [KeyCode::Char('q'), KeyCode::Esc, KeyCode::Char(' ')] {
        let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
        h.overlay = GameOverlay::OfflineWelcome {
            report: OfflineReport::default(),
        };
        let result = h.press(code);
        assert!(
            h.overlay_is_none(),
            "OfflineWelcome should dismiss on {code:?}"
        );
        assert_eq!(result, InputResult::Continue);
    }
}

#[test]
fn replay_dsl_drives_a_sequence() {
    let mut h = haven_ready();
    // Open the Haven then back out, all in one compact script.
    h.replay("h");
    assert!(h.haven_ui.showing);
    h.replay("Esc");
    assert!(!h.haven_ui.showing);
}

#[test]
fn results_history_records_every_press() {
    let mut h = haven_ready();
    h.char('h');
    h.press(KeyCode::Esc);
    h.char('x'); // no-op on the base screen

    assert_eq!(h.results().len(), 3);
    assert_eq!(h.last(), Some(&InputResult::Continue));
    // Every base-screen navigation here is a no-op result.
    assert!(h.results().iter().all(|r| *r == InputResult::Continue));
}

#[test]
#[should_panic(expected = "not a known key name")]
fn replay_rejects_multichar_non_key_tokens() {
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    // "hello" is neither a single char nor a named key — this is a test-author
    // mistake, so the harness panics loudly rather than silently misfiring.
    h.replay("hello");
}

#[test]
fn render_shows_the_base_game_screen() {
    let h = InputHarness::new(fixtures::fresh("Hero", 0));
    let frame = h.render(120, 40);
    assert!(
        frame.contains("Hero"),
        "rendered frame should show the hero's name; got:\n{frame}"
    );
}
