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
use crate::achievements::AchievementId;
use crate::core::offline::OfflineReport;
use crate::fixtures;
use crate::haven::HavenRoomId;
use crate::history::SaveEvent;
use crate::items::{EquipmentSlot, Rarity};
use crate::ui::achievement_browser_scene::AchievementBrowserState;
use crate::ui::title_browser_scene::TitleBrowserState;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
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

// -- InputResult → persistence contract -----------------------------------
//
// Every consequential key press returns a specific `InputResult` variant that
// `main_helpers::input_routing` turns into a save (and, for `*WithEvent`, a git
// history commit). A wrong variant means the action *appears* to work but never
// persists — silent data loss with no other tripwire. These tests pin that
// contract: they assert both the returned variant and the state mutation.

#[test]
fn prestige_confirm_prestiges_and_signals_a_save_with_event() {
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.state.character_level = 999; // eligible for the next prestige tier
    h.overlay = GameOverlay::PrestigeConfirm;

    // The confirm key is 'y' (NOT Enter), and with no Vault built this takes the
    // direct-prestige path.
    let result = h.char('y');

    assert_eq!(
        result,
        InputResult::NeedsSaveWithEvent(SaveEvent::PrestigeRank(1)),
        "prestige must signal a save-with-git-event carrying the new rank"
    );
    assert_eq!(h.state.prestige_rank, 1, "prestige rank should increment");
    assert_eq!(h.state.character_level, 1, "prestige resets the character");
    assert!(h.overlay_is_none(), "the confirm dialog should close");
}

#[test]
fn prestige_confirm_cancel_keys_leave_state_untouched() {
    for cancel in [KeyCode::Esc, KeyCode::Char('n'), KeyCode::Char('N')] {
        let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
        h.state.character_level = 999;
        h.overlay = GameOverlay::PrestigeConfirm;

        let result = h.press(cancel);

        assert_eq!(result, InputResult::Continue, "cancel is a no-op result");
        assert_eq!(
            h.state.prestige_rank, 0,
            "cancel must not prestige ({cancel:?})"
        );
        assert!(h.overlay_is_none(), "cancel closes the dialog ({cancel:?})");
    }
}

#[test]
fn prestige_confirm_ignores_non_action_keys() {
    // Enter reads as "confirm" in many dialogs, but this one acts only on
    // y / n / Esc — a stray Enter must not prestige and must leave the dialog up.
    for stray in [KeyCode::Enter, KeyCode::Char('x'), KeyCode::Char(' ')] {
        let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
        h.state.character_level = 999;
        h.overlay = GameOverlay::PrestigeConfirm;

        let result = h.press(stray);

        assert_eq!(result, InputResult::Continue);
        assert_eq!(h.state.prestige_rank, 0, "{stray:?} must not prestige");
        assert!(
            matches!(h.overlay, GameOverlay::PrestigeConfirm),
            "{stray:?} should leave the confirm dialog open"
        );
    }
}

#[test]
fn vault_prestige_preserves_selected_item_and_signals_save() {
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.state.character_level = 999;
    // A Vault at tier 3 preserves up to 5 items (VaultSlots values [1, 3, 5, 0]
    // by tier). `haven_built` puts the Vault at an invalid tier 4 → 0 slots, so
    // build it explicitly here.
    h.haven.discovered = true;
    h.haven.rooms.insert(HavenRoomId::Vault, 3);

    // Equip a full set so there is something to preserve — and something to wipe.
    let mut rng = ChaCha8Rng::seed_from_u64(1);
    fixtures::equip_all(&mut h.state, Rarity::Rare, Rarity::Rare, 50, &mut rng);
    assert!(h.state.equipment.get(EquipmentSlot::Weapon).is_some());

    h.overlay = GameOverlay::VaultSelection {
        selected_index: 0, // Weapon
        selected_slots: Vec::new(),
        confirm_pending: false,
    };

    // Select the weapon, then confirm. The confirm is one or two Enters
    // depending on Vault capacity (immediate at max, else a second Enter
    // confirms), so drive both and assert on the outcome + recorded save signal.
    h.press(KeyCode::Char(' ')); // toggle-select the weapon slot
    h.press(KeyCode::Enter);
    h.press(KeyCode::Enter);

    assert_eq!(
        h.state.prestige_rank, 1,
        "vault prestige should increment rank"
    );
    assert!(
        h.overlay_is_none(),
        "the Vault overlay should close after prestige"
    );
    assert!(
        h.state.equipment.get(EquipmentSlot::Weapon).is_some(),
        "the Vault-selected weapon must survive prestige"
    );
    assert!(
        h.state.equipment.get(EquipmentSlot::Armor).is_none(),
        "unselected gear must be wiped by prestige"
    );
    assert!(
        h.results()
            .contains(&InputResult::NeedsSaveWithEvent(SaveEvent::PrestigeRank(1))),
        "the vault prestige must signal a save-with-event; got {:?}",
        h.results()
    );
}

#[test]
fn selecting_a_title_signals_a_save() {
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.achievements.unlock(AchievementId::Level100, None); // grants a selectable title

    let mut title_browser = TitleBrowserState::new();
    title_browser.showing = true; // the title sub-browser has priority
    h.overlay = GameOverlay::Achievements {
        browser: AchievementBrowserState::new(),
        title_browser,
    };

    let result = h.press(KeyCode::Enter);

    assert_eq!(
        result,
        InputResult::NeedsSave,
        "choosing a title must signal a save"
    );
    assert_eq!(
        h.achievements.selected_title,
        Some(AchievementId::Level100),
        "the chosen title should be recorded on the account"
    );
}

#[test]
fn clearing_a_title_signals_a_save() {
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.achievements.unlock(AchievementId::Level100, None);
    h.achievements.selected_title = Some(AchievementId::Level100);

    let mut title_browser = TitleBrowserState::new();
    title_browser.showing = true;
    h.overlay = GameOverlay::Achievements {
        browser: AchievementBrowserState::new(),
        title_browser,
    };

    let result = h.press(KeyCode::Backspace);

    assert_eq!(
        result,
        InputResult::NeedsSave,
        "clearing a title must signal a save"
    );
    assert_eq!(
        h.achievements.selected_title, None,
        "the title should be cleared"
    );
}
