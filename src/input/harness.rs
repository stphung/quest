//! Headless input-replay harness for driving [`handle_game_input`] in tests.
//!
//! `src/input/` is otherwise unverified by automated tests: the only way to
//! exercise the keyboard dispatch chain was to launch the real game in a tmux
//! session (the `drive-game` skill) and press keys by hand. This harness is the
//! programmable, assertable counterpart — the input equivalent of the UI
//! snapshot tests.
//!
//! It owns every piece of state that [`GameContext`] borrows, feeds
//! [`KeyEvent`]s through the real dispatcher, records each [`InputResult`], and
//! can render the resulting frame so tests can assert on both *state* and the
//! *rendered screen*.
//!
//! ```ignore
//! use crate::input::harness::InputHarness;
//! use crate::fixtures;
//! use ratatui::crossterm::event::KeyCode;
//!
//! let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
//! h.haven.discovered = true;        // the [H] hotkey is gated on discovery
//! h.press(KeyCode::Char('h'));      // open the Haven overlay
//! assert!(h.haven_ui.showing);
//! h.press(KeyCode::Esc);            // close it
//! assert!(!h.haven_ui.showing);
//! ```

// A reusable test toolkit: not every method is exercised by the tests that ship
// with it today (that's the point — it's built to write *more* input tests
// against), so don't warn on the currently-unused surface.
#![allow(dead_code)]

use super::{handle_game_input, GameOverlay, HavenUiState, InputResult, SoulforgeUiState};
use crate::achievements::Achievements;
use crate::core::game_state::GameState;
use crate::deep::types::{DeepState, DeepUiState};
use crate::enhancement::EnhancementProgress;
use crate::haven::Haven;
use crate::loom::{LoomState, LoomUiState};
use crate::main_helpers::game_context::GameContext;
use crate::stormglass::types::ExchangeUiState;
use crate::utils::debug_menu::DebugMenu;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Owns the full mutable game context and replays key events through
/// [`handle_game_input`]. All state fields are public so a test can arrange a
/// scenario (open an overlay, discover a system) before pressing keys and
/// assert on the outcome afterwards.
pub struct InputHarness {
    pub state: GameState,
    pub haven: Haven,
    pub haven_ui: HavenUiState,
    pub soulforge_ui: SoulforgeUiState,
    pub exchange_ui: ExchangeUiState,
    pub deep_state: DeepState,
    pub deep_ui: DeepUiState,
    pub enhancement: EnhancementProgress,
    pub overlay: GameOverlay,
    pub debug_menu: DebugMenu,
    pub debug_mode: bool,
    pub achievements: Achievements,
    pub loom_state: LoomState,
    pub loom_ui: LoomUiState,

    // Discovery flags read only by `render` (they gate which HUD affordances
    // draw). `handle_game_input` does not consult them, so leaving them `false`
    // never affects input dispatch.
    pub haven_discovered: bool,
    pub soulforge_discovered: bool,
    pub loom_discovered: bool,
    pub enhancement_levels: [u8; 7],

    results: Vec<InputResult>,
}

impl InputHarness {
    /// Build a harness around `state` with every overlay/UI substate in its
    /// fresh, nothing-open default. Mutate the public fields directly to set up
    /// a scenario before pressing keys.
    pub fn new(state: GameState) -> Self {
        Self {
            state,
            haven: Haven::new(),
            haven_ui: HavenUiState::new(),
            soulforge_ui: SoulforgeUiState::new(),
            exchange_ui: ExchangeUiState::new(),
            deep_state: DeepState::new(),
            deep_ui: DeepUiState::new(),
            enhancement: EnhancementProgress::default(),
            overlay: GameOverlay::None,
            debug_menu: DebugMenu::new(),
            debug_mode: false,
            achievements: Achievements::default(),
            loom_state: LoomState::new(),
            loom_ui: LoomUiState::new(),
            haven_discovered: false,
            soulforge_discovered: false,
            loom_discovered: false,
            enhancement_levels: [0; 7],
            results: Vec::new(),
        }
    }

    /// Borrow all owned state as a [`GameContext`] for a single dispatch call.
    fn ctx(&mut self) -> GameContext<'_> {
        GameContext {
            state: &mut self.state,
            haven: &mut self.haven,
            haven_ui: &mut self.haven_ui,
            soulforge_ui: &mut self.soulforge_ui,
            exchange_ui: &mut self.exchange_ui,
            deep_state: &mut self.deep_state,
            deep_ui: &mut self.deep_ui,
            enhancement: &mut self.enhancement,
            overlay: &mut self.overlay,
            debug_menu: &mut self.debug_menu,
            debug_mode: self.debug_mode,
            achievements: &mut self.achievements,
            loom_state: &mut self.loom_state,
            loom_ui: &mut self.loom_ui,
        }
    }

    /// Feed one fully-specified [`KeyEvent`] and return its [`InputResult`].
    /// Use this when you need modifiers or a non-`Press` kind; most tests want
    /// [`press`](Self::press) or [`char`](Self::char).
    pub fn press_event(&mut self, key: KeyEvent) -> InputResult {
        let mut ctx = self.ctx();
        let result = handle_game_input(key, &mut ctx);
        self.results.push(result.clone());
        result
    }

    /// Press a key with no modifiers.
    pub fn press(&mut self, code: KeyCode) -> InputResult {
        self.press_event(KeyEvent::new(code, KeyModifiers::NONE))
    }

    /// Press a printable character with no modifiers.
    pub fn char(&mut self, c: char) -> InputResult {
        self.press(KeyCode::Char(c))
    }

    /// Type each character of `s` in order; returns the last [`InputResult`].
    /// Handy for text entry (e.g. a Time Vault sync token).
    pub fn type_str(&mut self, s: &str) -> InputResult {
        let mut last = InputResult::Continue;
        for c in s.chars() {
            last = self.char(c);
        }
        last
    }

    /// Replay a compact whitespace-separated key script and return the last
    /// [`InputResult`]. A single character token is a [`KeyCode::Char`]; the
    /// named tokens below map to their key. Panics on a multi-character token
    /// that is not a known key name — use [`type_str`](Self::type_str) for
    /// words.
    ///
    /// Recognized names: `Enter Esc Tab Up Down Left Right Space Backspace
    /// Home End Delete`.
    ///
    /// ```ignore
    /// h.replay("Tab Down Down Enter Esc"); // open menu, move down twice, select, back out
    /// ```
    pub fn replay(&mut self, script: &str) -> InputResult {
        let mut last = InputResult::Continue;
        for token in script.split_whitespace() {
            last = self.press(token_to_key(token));
        }
        last
    }

    /// The most recent [`InputResult`], or `None` if no key has been pressed.
    pub fn last(&self) -> Option<&InputResult> {
        self.results.last()
    }

    /// Every [`InputResult`] returned so far, oldest first.
    pub fn results(&self) -> &[InputResult] {
        &self.results
    }

    /// True when no overlay/modal is active (`GameOverlay::None`).
    pub fn overlay_is_none(&self) -> bool {
        matches!(self.overlay, GameOverlay::None)
    }

    /// Render the base game screen into an off-screen `width`×`height` buffer
    /// and return the buffer's debug view (characters plus style runs). Lets a
    /// test assert that a key press actually changed what's on screen — e.g.
    /// that a HUD affordance appeared — with `contains`-style checks.
    ///
    /// This uses the live UI clock, so spinner/pulse *glyphs* vary between
    /// runs; assert on stable content (zone names, numbers, labels), not on
    /// byte-exact frames. Byte-exact snapshots belong in the `cfg(test)`
    /// snapshot suite (`ui::snapshot_tests`), which can freeze the clock.
    ///
    /// Note: overlays/modals are drawn by `main_helpers::overlay`, not by
    /// `draw_ui_with_update`, so this renders the base game screen that sits
    /// *underneath* any open overlay.
    pub fn render(&self, width: u16, height: u16) -> String {
        use ratatui::{backend::TestBackend, Terminal};

        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal
            .draw(|frame| {
                crate::ui::draw_ui_with_update(
                    frame,
                    &self.state,
                    None,
                    false,
                    false,
                    self.haven_discovered,
                    self.soulforge_discovered,
                    self.state.stormglass_discovered,
                    &self.deep_state,
                    self.loom_discovered,
                    &self.achievements,
                    &self.enhancement_levels,
                    &self.deep_state,
                );
            })
            .unwrap();
        format!("{:?}", terminal.backend().buffer())
    }
}

/// Map one `replay` token to a [`KeyCode`].
fn token_to_key(token: &str) -> KeyCode {
    match token {
        "Enter" => KeyCode::Enter,
        "Esc" => KeyCode::Esc,
        "Tab" => KeyCode::Tab,
        "Up" => KeyCode::Up,
        "Down" => KeyCode::Down,
        "Left" => KeyCode::Left,
        "Right" => KeyCode::Right,
        "Space" => KeyCode::Char(' '),
        "Backspace" => KeyCode::Backspace,
        "Home" => KeyCode::Home,
        "End" => KeyCode::End,
        "Delete" => KeyCode::Delete,
        other => {
            let mut chars = other.chars();
            let c = chars
                .next()
                .expect("split_whitespace never yields an empty token");
            assert!(
                chars.next().is_none(),
                "replay token `{other}` is multi-character but not a known key name; \
                 use `type_str` for words or a named key like `Enter`"
            );
            KeyCode::Char(c)
        }
    }
}
