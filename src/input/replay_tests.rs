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
use crate::achievements::{AchievementCategory, AchievementId};
use crate::challenges::{create_challenge, ChallengeType};
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
fn leviathan_catch_miss_modal_dismisses_only_on_enter() {
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.overlay = GameOverlay::LeviathanCatchMiss {
        lure_consumed: false,
    };

    let result = h.char('x');
    assert_eq!(result, InputResult::Continue);
    assert!(
        matches!(h.overlay, GameOverlay::LeviathanCatchMiss { .. }),
        "non-Enter key should not dismiss the catch-miss modal"
    );

    h.press(KeyCode::Enter);
    assert!(
        h.overlay_is_none(),
        "Enter should dismiss the catch-miss modal"
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

// -- Achievement browser overlay -------------------------------------------

fn achievements_ready() -> InputHarness {
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.char('a');
    h
}

#[test]
fn achievements_hotkey_opens_the_browser() {
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.char('a');
    assert!(
        matches!(h.overlay, GameOverlay::Achievements { .. }),
        "[A] should open the achievement browser"
    );
}

#[test]
fn achievement_browser_cycles_categories_and_closes_on_a_or_esc() {
    let mut h = achievements_ready();
    let GameOverlay::Achievements { ref browser, .. } = h.overlay else {
        panic!("expected the achievement browser overlay to be open");
    };
    assert_eq!(browser.selected_category, AchievementCategory::Combat);

    h.press(KeyCode::Right);
    let GameOverlay::Achievements { ref browser, .. } = h.overlay else {
        panic!("expected the achievement browser overlay to be open");
    };
    assert_eq!(
        browser.selected_category,
        AchievementCategory::Level,
        "[Right] should advance to the next category"
    );

    h.press(KeyCode::Left);
    let GameOverlay::Achievements { ref browser, .. } = h.overlay else {
        panic!("expected the achievement browser overlay to be open");
    };
    assert_eq!(
        browser.selected_category,
        AchievementCategory::Combat,
        "[Left] should go back to the previous category"
    );

    h.char('a'); // 'a' also closes the browser, matching the open hotkey
    assert!(h.overlay_is_none(), "[A] should close the browser again");

    let mut h = achievements_ready();
    h.press(KeyCode::Esc);
    assert!(h.overlay_is_none(), "Esc should also close the browser");
}

#[test]
fn achievement_browser_t_opens_the_nested_title_browser() {
    let mut h = achievements_ready();
    h.char('t');
    let GameOverlay::Achievements {
        ref title_browser, ..
    } = h.overlay
    else {
        panic!("expected the achievement browser overlay to be open");
    };
    assert!(
        title_browser.showing,
        "[T] should open the nested title browser"
    );
}

// -- Bug report / browser-link overlays ------------------------------------

#[test]
fn bug_report_hotkey_opens_overlay_with_a_summary() {
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.char('!');
    match &h.overlay {
        GameOverlay::BugReport { summary, .. } => {
            assert!(
                summary.contains("Hero"),
                "the bug report summary should mention the hero's name; got:\n{summary}"
            );
        }
        _ => panic!("['!'] should open the BugReport overlay"),
    }
}

#[test]
fn bug_report_overlay_closes_on_esc() {
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.char('!');
    assert!(matches!(h.overlay, GameOverlay::BugReport { .. }));
    h.press(KeyCode::Esc);
    assert!(
        h.overlay_is_none(),
        "Esc should close the bug report overlay"
    );
}

#[test]
fn browser_link_fallback_modal_closes_on_esc_only() {
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.overlay = GameOverlay::BrowserLink {
        url: "https://example.com".to_string(),
    };
    h.char('x');
    assert!(
        !h.overlay_is_none(),
        "a stray key should not dismiss the browser-link modal"
    );

    let result = h.press(KeyCode::Esc);
    assert!(
        h.overlay_is_none(),
        "Esc should dismiss the browser-link modal"
    );
    assert_eq!(result, InputResult::Continue);
}

// -- Time Vault hotkey -------------------------------------------------------

#[test]
fn time_vault_hotkey_signals_open_without_touching_the_overlay() {
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    let result = h.char('t');
    assert_eq!(
        result,
        InputResult::OpenTimeVault,
        "[T] should signal main.rs to populate and open the Time Vault"
    );
    assert!(
        h.overlay_is_none(),
        "the dispatcher itself must not set the TimeVault overlay -- main.rs does, from HistoryRepo"
    );
}

// -- Discovery / celebration modal family -----------------------------------
//
// HavenDiscovery, SoulforgeDiscovery, StormglassDiscovery, DeepDiscovery,
// LoomDiscovery, VesselDiscovery, FractureRegionUnlock, and
// PatternMilestoneUnlock all share `handle_dismiss_overlay`: Enter or Esc
// dismisses, anything else is swallowed and leaves the modal up.

type OverlayFactory = (&'static str, fn() -> GameOverlay);

#[test]
fn discovery_and_celebration_modals_dismiss_on_enter_or_esc_only() {
    let overlays: Vec<OverlayFactory> = vec![
        ("HavenDiscovery", || GameOverlay::HavenDiscovery),
        ("SoulforgeDiscovery", || GameOverlay::SoulforgeDiscovery),
        ("StormglassDiscovery", || GameOverlay::StormglassDiscovery),
        ("DeepDiscovery", || GameOverlay::DeepDiscovery),
        ("LoomDiscovery", || GameOverlay::LoomDiscovery),
        ("VesselDiscovery", || GameOverlay::VesselDiscovery),
        ("FractureRegionUnlock", || {
            GameOverlay::FractureRegionUnlock {
                region: crate::zones::FractureRegion::RedFault,
            }
        }),
        ("PatternMilestoneUnlock", || {
            GameOverlay::PatternMilestoneUnlock {
                milestone: crate::loom::PatternMilestone::ThreadWilds,
            }
        }),
    ];

    for (name, make) in overlays {
        let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
        h.overlay = make();
        h.char('x');
        assert!(
            !h.overlay_is_none(),
            "{name}: a stray key should not dismiss the modal"
        );

        let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
        h.overlay = make();
        h.press(KeyCode::Enter);
        assert!(
            h.overlay_is_none(),
            "{name}: Enter should dismiss the modal"
        );

        let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
        h.overlay = make();
        h.press(KeyCode::Esc);
        assert!(h.overlay_is_none(), "{name}: Esc should dismiss the modal");
    }
}

#[test]
fn achievement_unlocked_modal_dismisses_on_enter_esc_or_space_only() {
    for code in [KeyCode::Enter, KeyCode::Esc, KeyCode::Char(' ')] {
        let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
        h.overlay = GameOverlay::AchievementUnlocked {
            achievements: vec![AchievementId::Level100],
        };
        h.press(code);
        assert!(
            h.overlay_is_none(),
            "{code:?} should dismiss the achievement-unlocked modal"
        );
    }

    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.overlay = GameOverlay::AchievementUnlocked {
        achievements: vec![AchievementId::Level100],
    };
    h.char('x');
    assert!(
        !h.overlay_is_none(),
        "a stray key should not dismiss the achievement-unlocked modal"
    );
}

// -- Quit confirmation -------------------------------------------------------

#[test]
fn quit_confirm_enter_quits_other_keys_cancel() {
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.overlay = GameOverlay::QuitConfirm;
    let result = h.press(KeyCode::Enter);
    assert_eq!(result, InputResult::QuitToSelect);

    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.overlay = GameOverlay::QuitConfirm;
    let result = h.press(KeyCode::Esc);
    assert_eq!(result, InputResult::Continue);
    assert!(
        h.overlay_is_none(),
        "canceling quit should close the confirm dialog"
    );
}

#[test]
fn esc_quits_directly_when_no_challenges_are_pending() {
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    assert!(h.state.challenge_menu.challenges.is_empty());
    let result = h.press(KeyCode::Esc);
    assert_eq!(result, InputResult::QuitToSelect);
}

#[test]
fn esc_warns_before_quitting_when_challenges_are_pending() {
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.state
        .challenge_menu
        .add_challenge(create_challenge(&ChallengeType::Rune));
    let result = h.press(KeyCode::Esc);
    assert_eq!(
        result,
        InputResult::Continue,
        "Esc with pending challenges should not quit immediately"
    );
    assert!(matches!(h.overlay, GameOverlay::QuitConfirm));
}

// -- Deep / Loom / Ascension hotkeys ----------------------------------------

#[test]
fn deep_hotkey_is_gated_on_discovery() {
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.char('d');
    assert!(!h.deep_ui.open, "The Deep opened without being discovered");

    h.deep_state.persistent.discovered = true;
    h.char('d');
    assert!(h.deep_ui.open, "[D] did not open The Deep");
}

#[test]
fn loom_hotkey_is_gated_on_discovery() {
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.char('l');
    assert!(!h.loom_ui.open, "Loom opened without being discovered");

    h.loom_state.persistent.discovered = true;
    h.char('l');
    assert!(h.loom_ui.open, "[L] did not open the Loom of Worlds");
}

/// A harness eligible for Ascension I: the Deep is discovered, layer 3 is
/// reached (Ascension I's gate), and enough PR is banked to afford it.
fn ascension_ready() -> InputHarness {
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.deep_state.persistent.discovered = true;
    h.deep_state.persistent.deepest_layer_reached = 3;
    h.state.prestige_rank = 35;
    h
}

#[test]
fn ascension_hotkey_is_gated_on_deep_discovery_and_layer_gate() {
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.char('u');
    assert!(!matches!(h.overlay, GameOverlay::AscensionConfirm));

    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.deep_state.persistent.discovered = true; // layer 0 < 3: gate not met
    h.char('u');
    assert!(
        !matches!(h.overlay, GameOverlay::AscensionConfirm),
        "Ascension should stay gated until the Deep-layer requirement is met"
    );

    let mut h = ascension_ready();
    h.char('u');
    assert!(matches!(h.overlay, GameOverlay::AscensionConfirm));
}

#[test]
fn ascension_confirm_ascends_and_signals_a_save_with_event() {
    let mut h = ascension_ready();
    h.overlay = GameOverlay::AscensionConfirm;

    let result = h.char('y');

    assert_eq!(
        h.state.ascension_level, 1,
        "ascension level should increment"
    );
    assert_eq!(
        h.state.prestige_rank, 0,
        "Ascension I's 35 PR cost should be deducted"
    );
    assert!(h.overlay_is_none());
    assert!(
        matches!(
            result,
            InputResult::NeedsSaveWithEvent(SaveEvent::AchievementUnlocked(_))
        ),
        "ascending must signal a save-with-event; got {result:?}"
    );
}

#[test]
fn ascension_confirm_cancel_keys_leave_state_untouched() {
    for cancel in [KeyCode::Esc, KeyCode::Char('n'), KeyCode::Char('N')] {
        let mut h = ascension_ready();
        h.overlay = GameOverlay::AscensionConfirm;

        let result = h.press(cancel);

        assert_eq!(result, InputResult::Continue);
        assert_eq!(h.state.ascension_level, 0, "{cancel:?} must not ascend");
        assert!(h.overlay_is_none(), "{cancel:?} should close the dialog");
    }
}

// -- Debug menu ---------------------------------------------------------

#[test]
fn debug_menu_backtick_toggles_and_navigates_categories() {
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.debug_mode = true;

    h.char('`');
    assert!(h.debug_menu.is_open, "backtick should open the debug menu");

    h.press(KeyCode::Tab);
    assert_eq!(
        h.debug_menu.selected_category, 1,
        "Tab should advance to the next category"
    );

    h.char('`');
    assert!(
        !h.debug_menu.is_open,
        "backtick should close the debug menu again"
    );
}

#[test]
fn debug_menu_is_inert_without_debug_mode() {
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    assert!(!h.debug_mode);
    h.char('`');
    assert!(
        !h.debug_menu.is_open,
        "backtick should do nothing outside debug mode"
    );
}

#[test]
fn debug_menu_haven_discovery_action_opens_the_discovery_modal() {
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.debug_mode = true;
    h.debug_menu.open();
    h.debug_menu.selected_category = 1; // World
    h.debug_menu.selected_index = 2; // Trigger Haven Discovery

    h.press(KeyCode::Enter);

    assert!(
        matches!(h.overlay, GameOverlay::HavenDiscovery),
        "triggering the Haven Discovery debug action should open the discovery modal"
    );
    assert!(
        h.haven.discovered,
        "the debug action should also flip the discovery flag"
    );
}

// -- Challenge menu -----------------------------------------------------

#[test]
fn tab_opens_the_challenge_menu_once_a_challenge_is_pending() {
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.press(KeyCode::Tab);
    assert!(
        !h.state.challenge_menu.is_open,
        "Tab should do nothing with no challenges pending"
    );

    h.state
        .challenge_menu
        .add_challenge(create_challenge(&ChallengeType::Rune));
    h.press(KeyCode::Tab);
    assert!(
        h.state.challenge_menu.is_open,
        "[Tab] should open the challenge menu once a challenge is pending"
    );
}

#[test]
fn challenge_menu_navigates_opens_detail_and_declines() {
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.state
        .challenge_menu
        .add_challenge(create_challenge(&ChallengeType::Rune));
    h.state
        .challenge_menu
        .add_challenge(create_challenge(&ChallengeType::Snake));
    h.state.challenge_menu.open();

    assert_eq!(h.state.challenge_menu.selected_index, 0);
    h.press(KeyCode::Down);
    assert_eq!(h.state.challenge_menu.selected_index, 1);
    h.press(KeyCode::Up);
    assert_eq!(h.state.challenge_menu.selected_index, 0);

    h.press(KeyCode::Enter); // opens the difficulty-select detail view
    assert!(h.state.challenge_menu.viewing_detail);

    h.char('d'); // decline from the detail view
    assert_eq!(
        h.state.challenge_menu.challenges.len(),
        1,
        "declining should remove the selected challenge"
    );
    assert!(
        h.state.challenge_menu.is_open,
        "one challenge remains, so the menu should stay open"
    );

    h.press(KeyCode::Esc);
    assert!(!h.state.challenge_menu.is_open);
}

#[test]
fn challenge_menu_accepting_a_challenge_starts_the_minigame() {
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.state
        .challenge_menu
        .add_challenge(create_challenge(&ChallengeType::Rune));
    h.state.challenge_menu.open();

    h.press(KeyCode::Enter); // open detail
    h.press(KeyCode::Enter); // accept at the default difficulty

    assert!(
        h.state.active_minigame.is_some(),
        "accepting a challenge should start its minigame"
    );
    assert!(
        !h.state.challenge_menu.is_open,
        "the menu should close once a challenge is accepted"
    );
}

#[test]
fn active_minigame_intercepts_background_hotkeys() {
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.state
        .challenge_menu
        .add_challenge(create_challenge(&ChallengeType::Rune));
    h.state.challenge_menu.open();
    h.press(KeyCode::Enter); // open detail
    h.press(KeyCode::Enter); // accept -> starts the Rune minigame
    assert!(h.state.active_minigame.is_some());

    // Haven IS discovered, so [H] would normally open it, but an active
    // minigame (step 6) must swallow the key before base hotkeys (step 9) see it.
    h.haven.discovered = true;
    h.char('h');
    assert!(
        !h.haven_ui.showing,
        "an active minigame must swallow background hotkeys"
    );
}

// -- Remaining base-game hotkeys ---------------------------------------

#[test]
fn wiki_hotkey_opens_browser_or_falls_back_to_a_link_modal() {
    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.char('w');
    assert!(
        h.overlay_is_none() || matches!(h.overlay, GameOverlay::BrowserLink { .. }),
        "[W] should either launch the browser (no overlay) or show the BrowserLink fallback"
    );
}

#[test]
fn vessel_hotkey_stays_inert_while_act2_is_dark_shipped() {
    // Act 2 ships dark (`vessel::ACT2_ENABLED == false`); even a qualified
    // signal must not surface the `[V]` overlay unless a session opts in via
    // `QUEST_ACT2=1`. This test asserts the kill-switch itself, so it only
    // makes sense in the default (Act 2 disabled) configuration.
    assert!(
        !crate::vessel::act2_enabled(),
        "this test assumes Act 2 is dark-shipped in this run"
    );

    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.state.vessel_signal_discovered = true;
    h.char('v');
    assert!(
        h.overlay_is_none(),
        "[V] must stay inert while Act 2 is dark-shipped"
    );
}

/// Fully qualify a harness state for the launch burn: signal, Ascension X,
/// the PR, and all 28 Woven Patterns.
#[cfg(test)]
fn qualify_for_launch(h: &mut InputHarness, pr_headroom: u32) {
    h.state.vessel_signal_discovered = true;
    h.state.ascension_level = 10;
    h.state.prestige_rank = crate::vessel::LAUNCH_PR_COST + pr_headroom;
    crate::loom::initialize_loom(&mut h.loom_state);
    crate::loom::complete_discovery(&mut h.loom_state);
    for p in h.loom_state.persistent.patterns.iter_mut().take(28) {
        p.completed = true;
    }
}

#[test]
fn flag_on_vessel_hotkey_opens_overlay_and_enter_burns_the_launch() {
    // Self-skipping flag-ON smoke test: a green no-op in ordinary (dark)
    // runs; actually exercised by the dedicated `QUEST_ACT2=1 cargo test
    // flag_on` step in CI / scripts/ci-checks.sh, which reruns the built
    // test binaries in a fresh process where the OnceLock caches the flag ON.
    if !crate::vessel::act2_enabled() {
        return;
    }

    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    h.state.vessel_signal_discovered = true;
    h.char('v');
    assert!(
        matches!(
            h.overlay,
            GameOverlay::Vessel {
                confirm_pending: false
            }
        ),
        "[V] opens the Vessel overlay once the signal is discovered"
    );

    // Unqualified Enter must not arm the confirm; Esc closes.
    h.press(KeyCode::Enter);
    assert!(
        matches!(
            h.overlay,
            GameOverlay::Vessel {
                confirm_pending: false
            }
        ),
        "Enter without the prerequisites must not arm the confirm"
    );
    h.press(KeyCode::Esc);
    assert!(h.overlay_is_none());

    // Fully qualified: the two-step confirm reaches perform_launch, and the
    // consequential press reports NeedsSave (a wrong variant would silently
    // skip the save).
    qualify_for_launch(&mut h, 5);
    h.char('v');
    h.press(KeyCode::Enter);
    assert!(
        matches!(
            h.overlay,
            GameOverlay::Vessel {
                confirm_pending: true
            }
        ),
        "a qualified Enter arms the confirm"
    );
    let result = h.press(KeyCode::Enter);
    assert!(
        h.state.vessel_launched,
        "the confirmed Enter burns the launch"
    );
    assert_eq!(
        h.state.prestige_rank, 5,
        "exactly LAUNCH_PR_COST is subtracted"
    );
    assert_eq!(result, InputResult::NeedsSave);
}

#[test]
fn flag_on_esc_disarms_the_launch_confirm_without_closing() {
    // Self-skipping flag-ON smoke test — see the note on the test above.
    if !crate::vessel::act2_enabled() {
        return;
    }

    let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
    qualify_for_launch(&mut h, 0);
    h.char('v');
    h.press(KeyCode::Enter);
    h.press(KeyCode::Esc);
    assert!(
        matches!(
            h.overlay,
            GameOverlay::Vessel {
                confirm_pending: false
            }
        ),
        "Esc disarms the confirm but keeps the overlay open"
    );
    assert!(!h.state.vessel_launched);
    h.press(KeyCode::Esc);
    assert!(h.overlay_is_none(), "a second Esc closes the overlay");
}
