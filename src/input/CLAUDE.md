# Input Module

Keyboard input routing for the Game screen, dispatching to overlay handlers, minigame input processors, and base game controls.

## Files

| File | Purpose |
|------|---------|
| `mod.rs` | Top-level dispatcher (`handle_game_input`) with numbered priority chain; modal dismiss handlers; debug menu routing; base game hotkeys |
| `types.rs` | `GameOverlay` enum (18 variants), `InputResult` enum (22 variants), `HavenUiState` struct, `HavenConfirmation` enum |
| `loom_input.rs` | Loom of Worlds overlay input: FlowView navigation, shuttle build/demolish, Codex browsing |
| `minigame_input.rs` | Dispatches keyboard events to all 10 challenge minigame input handlers; game-over cooldown (2s) logic |
| `haven_input.rs` | Haven overlay input: room selection, build confirmation, Storm Forge confirmation |
| `prestige_input.rs` | Prestige confirmation dialog and Vault item selection (equipment preservation across prestige) |
| `soulforge_input.rs` | Soulforge enhancement overlay: slot selection, confirm/cancel, Soul Tithe toggle, hammering animation phase |
| `deep_input.rs` | The Deep overlay: Hub (roster/recruit), NewMission (pool/active/squad staging), Infrastructure, event response modal |
| `stormglass_input.rs` | Stormglass Exchange overlay: menu navigation, Invoke Trial (rolling animation + pick), Chrono Surge, Storm Sigils (etch/reroll/pick), Storm Lure |
| `time_vault_input.rs` | Time Vault overlay: branch/commit browsing, restore, fork, delete; GitHub cloud sync (token entry, repo selection, push/pull, divergence resolution) |

## Key Types

- **`GameOverlay`**: Enum with 18 variants representing the active modal/overlay. At most one is active at a time. Includes discovery modals, achievement browser, Time Vault, bug report, quit confirm, and more.
- **`InputResult`**: Enum with 22 variants describing what happened after processing input. Variants range from `Continue` (no-op) to `NeedsSaveWithEvent` (save + git commit) to cloud sync actions like `PushCloud`, `ResolveKeepLocal`, etc.
- **`HavenUiState`**: Tracks Haven overlay visibility, selected room index, confirmation state, and open timestamp for animations.
- **`HavenConfirmation`**: Three-state enum (`None`, `Build`, `Forge`) for Haven build/forge confirmation dialogs.

## Input Dispatch Flow

`handle_game_input()` processes input through a strict numbered priority chain:

1. **Step 0**: Dismiss-on-any-key modals (offline welcome, Leviathan encounter/miss)
2. **Step 0.5**: Achievement browser overlay (with nested title browser)
3. **Step 0.8**: Bug report overlay and browser link fallback modal
4. **Step 0.85**: Time Vault overlay (delegates to `time_vault_input`)
5. **Step 1**: Discovery/celebration modals (Haven, Soulforge, Stormglass, Deep, achievement unlock, fracture region, ascension confirm) -- Enter/Esc dismisses
6. **Step 2**: Full-screen overlays (Haven, Soulforge, Stormglass Exchange, The Deep) -- each delegates to its own handler
7. **Step 3**: Vault item selection (prestige equipment preservation)
8. **Step 4**: Prestige confirmation dialog
9. **Step 4.5**: Quit confirmation (pending challenges warning)
10. **Step 5**: Debug menu (backtick toggles, Tab/arrows navigate, Enter triggers)
11. **Step 6**: Active minigame (delegates to `minigame_input`)
12. **Step 7**: Challenge menu (open/navigate/select)
13. **Step 8**: Tab opens challenge menu
14. **Step 9**: Base game hotkeys (P=prestige, H=haven, S=soulforge, G=stormglass, D=deep, A=achievements, T=time vault, U=ascension, W=wiki, !=bug report)

**Key pattern**: Overlays intercept before game input. Higher-numbered steps only execute if all earlier overlays/modals are inactive. This ensures modals always block background input.

## Minigame Input Pattern

Each of the 10 minigames follows the same structure in `minigame_input.rs`:
1. Check if game result exists -- if so, apply result and dismiss after 2s cooldown
2. Map `KeyCode` to minigame-specific input enum (e.g., `ChessInput::Up`)
3. Call the minigame's `process_input()` function

The **forfeit pattern** (first Esc sets `forfeit_pending`, second Esc confirms) is implemented inside each minigame's logic, not in the input module.

## Integration Points

- **Imports from**: `challenges/` (menu processing, all 10 minigame input handlers), `character/prestige` (can_prestige, perform_prestige), `haven/` (try_build_room, can_forge_stormbreaker), `enhancement/` (roll_enhancement, costs), `deep/` (mission management, guild rank), `stormglass/` (sigils, spending), `achievements/` (browser, titles, sync), `ascension/` (ascend), `zones/` (sync_account_zone_unlocks), `utils/debug_menu` (DebugMenu), `history/` (SaveEvent)
- **Consumed by**: `main.rs` and `main_helpers/input_routing.rs` call `handle_game_input()` and route the `InputResult`
- **UI coupling**: References `ui::achievement_browser_scene`, `ui::title_browser_scene`, `ui::time_vault_scene`, `ui::stats_prestige` for type imports only (no rendering)
