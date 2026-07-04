# Input Module

Keyboard input routing for the Game screen, dispatching to overlay handlers, minigame input processors, and base game controls.

## Files

| File | Purpose |
|------|---------|
| `mod.rs` | Top-level dispatcher (`handle_game_input`) with numbered priority chain; modal dismiss handlers; debug menu routing; base game hotkeys |
| `types.rs` | `GameOverlay` enum (22 variants), `InputResult` enum (23 variants), `HavenUiState` struct, `HavenConfirmation` enum |
| `loom_input.rs` | Loom of Worlds overlay input: GraphView navigation (topology-based arrow key movement), shuttle build/demolish |
| `minigame_input.rs` | Dispatches keyboard events to all 14 challenge minigame input handlers; game-over cooldown (2s) logic |
| `haven_input.rs` | Haven overlay input: room selection, build confirmation, Storm Forge confirmation |
| `prestige_input.rs` | Prestige confirmation dialog and Vault item selection (equipment preservation across prestige) |
| `soulforge_input.rs` | Soulforge enhancement overlay: slot selection, confirm/cancel, Soul Tithe toggle, hammering animation phase |
| `deep_input.rs` | The Deep overlay: Hub (roster/recruit), NewMission (pool/active/squad staging), Infrastructure, event response modal |
| `stormglass_input.rs` | Stormglass Exchange overlay: menu navigation, Invoke Trial (rolling animation + pick), Chrono Surge, Storm Sigils (etch/reroll/pick), Storm Lure |
| `time_vault_input.rs` | Time Vault overlay: branch/commit browsing, restore, fork, delete; GitHub cloud sync (token entry, repo selection, push/pull, divergence resolution) |
| `voyage_input.rs` | Act 2 "Crossing" screen input (`handle_voyage_input()` / `VoyageInputResult`): intro/scene/moment playback, boarding-ask and refit prompts, chart/junction/trim/souls/watch/farewell/rumors/manifest/keepsake view navigation |

## Key Types

- **`GameOverlay`**: Enum with 22 variants representing the active modal/overlay. At most one is active at a time. Includes discovery modals, achievement browser, Time Vault, bug report, quit confirm, `VesselDiscovery` (signal discovery celebration), `Vessel { confirm_pending }` (construction overlay and launch confirmation), `PatternMilestoneUnlock { milestone }` (Loom pattern milestone celebration), and more.
- **`InputResult`**: Enum with 23 variants describing what happened after processing input. Variants range from `Continue` (no-op) to `NeedsSaveWithEvent` (save + git commit) to cloud sync actions like `PushCloud`, `ResolveKeepLocal`, etc.
- **`HavenUiState`**: Tracks Haven overlay visibility, selected room index, confirmation state, and open timestamp for animations.
- **`HavenConfirmation`**: Three-state enum (`None`, `Build`, `Forge`) for Haven build/forge confirmation dialogs.

## Input Dispatch Flow

`handle_game_input()` processes input through a strict numbered priority chain:

1. **Step 0**: Dismiss-on-any-key modals (offline welcome); Enter-dismiss modals (Leviathan encounter at step 0.25, Leviathan catch-miss at step 0.26)
2. **Step 0.5**: Achievement browser overlay (with nested title browser)
3. **Step 0.8**: Bug report overlay and browser link fallback modal
4. **Step 0.85**: Time Vault overlay (delegates to `time_vault_input`)
5. **Step 1**: Discovery/celebration modals (Haven, Soulforge, Stormglass, Deep, Loom, Vessel signal, achievement unlock, fracture region, pattern milestone, ascension confirm) -- Enter/Esc dismisses
6. **Step 2**: Full-screen overlays (Haven, Soulforge, Stormglass Exchange, The Deep) -- each delegates to its own handler
7. **Step 2.9**: Loom of Worlds overlay (delegates to `loom_input::handle_loom` when `loom_ui.open`)
8. **Step 2.95**: The Vessel overlay -- `if matches!(overlay, GameOverlay::Vessel { .. }) { return handle_vessel_overlay(...) }`
9. **Step 3**: Vault item selection (prestige equipment preservation)
10. **Step 4**: Prestige confirmation dialog
11. **Step 4.5**: Quit confirmation (pending challenges warning)
12. **Step 5**: Debug menu (backtick toggles, Tab/arrows navigate, Enter triggers)
13. **Step 6**: Active minigame (delegates to `minigame_input`)
14. **Step 7**: Challenge menu (open/navigate/select)
15. **Step 8**: Tab opens challenge menu
16. **Step 8.5**: Loom of Worlds toggle (`l`/`L` opens the Loom overlay, gated on `loom_state.persistent.discovered`)
17. **Step 9**: Base game hotkeys (P=prestige, H=haven, S=soulforge, G=stormglass, D=deep, A=achievements, T=time vault, U=ascension, W=wiki, V=vessel, !=bug report)

**Key pattern**: Overlays intercept before game input. Higher-numbered steps only execute if all earlier overlays/modals are inactive. This ensures modals always block background input.

## Minigame Input Pattern

Each of the 14 minigames follows the same structure in `minigame_input.rs`:
1. Check if game result exists -- if so, apply result and dismiss after 2s cooldown
2. Map `KeyCode` to minigame-specific input enum (e.g., `ChessInput::Up`)
3. Call the minigame's `process_input()` function

The **forfeit pattern** (first Esc sets `forfeit_pending`, second Esc confirms) is implemented inside each minigame's logic, not in the input module.

## Testing: the Input-Replay Harness

`handle_game_input` is exercised by [`harness.rs`](harness.rs) + [`replay_tests.rs`](replay_tests.rs) (both `#[cfg(test)]`, so they compile and run only under `cargo test`, in the binary crate where `input` lives). This is the programmable, assertable counterpart to the manual `drive-game` skill — the input analog of the UI snapshot tests.

`InputHarness` owns every piece of state a [`GameContext`](../main_helpers/game_context.rs) borrows, feeds `KeyEvent`s through the real dispatcher, records each `InputResult`, and can render the base frame for content assertions:

```rust
let mut h = InputHarness::new(fixtures::fresh("Hero", 0));
h.haven.discovered = true;         // [H] is gated on discovery
h.char('h');                       // press a key, get the InputResult back
assert!(h.haven_ui.showing);
h.replay("Esc");                   // compact whitespace key script
assert!(!h.haven_ui.showing);
```

- All state fields are `pub` — arrange a scenario (open an overlay, flip a discovery flag) before pressing keys.
- `press` / `char` / `press_event` drive one key; `type_str` types a word; `replay` runs a whitespace-separated script (`"Tab Down Down Enter Esc"`; named keys `Enter Esc Tab Up Down Left Right Space Backspace Home End Delete`, else single chars).
- `results()` / `last()` expose the recorded `InputResult` history; `render(w, h)` returns the base-screen buffer debug string for `contains`-style assertions (byte-exact frames belong in `ui::snapshot_tests`, which can freeze the clock).

**Add coverage** for any input change by extending `replay_tests.rs` — cover the discovery gate, the modal-interception order (higher-priority overlays must swallow keys first), and Enter-only vs any-key dismissal.

## Integration Points

- **Imports from**: `challenges/` (menu processing, all 14 minigame input handlers), `character/prestige` (can_prestige, perform_prestige), `haven/` (try_build_room, can_forge_stormbreaker), `enhancement/` (roll_enhancement, costs), `deep/` (mission management, guild rank), `stormglass/` (sigils, spending), `achievements/` (browser, titles, sync), `ascension/` (ascend), `zones/` (sync_account_zone_unlocks), `loom/` (Loom of Worlds state), `vessel/` (functions: `can_launch`, `perform_launch`; types: `route`, `voyage::*`, `SceneModal`, `VoyageUiState`, `VoyageView`), `utils/debug_menu` (DebugMenu), `history/` (SaveEvent)
- **Consumed by**: `main.rs` and `main_helpers/input_routing.rs` call `handle_game_input()` and route the `InputResult`
- **UI coupling**: References `ui::achievement_browser_scene`, `ui::title_browser_scene`, `ui::time_vault_scene`, `ui::stats_prestige` for type imports only (no rendering)
