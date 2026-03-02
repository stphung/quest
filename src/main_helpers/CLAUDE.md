# Main Helpers Module

Thin orchestration wrappers extracted from `main.rs` to keep the game loop readable. Each file encapsulates one concern.

## Files

| File | Purpose |
|------|---------|
| `mod.rs` | Module declarations and public re-exports |
| `achievements.rs` | `log_synced_achievements()` -- logs newly synced achievements to combat log; `track_input_achievements()` -- tracks prestige, fishing rank, and minigame win achievements triggered by player input (not by game_tick) |
| `character_screens.rs` | `ScreenTransition` enum and frame handlers for character creation, deletion, and rename screens; each draws UI, polls input, returns transition |
| `cloud_ops.rs` | `reload_account_state()` -- reloads Haven, Enhancement, and Achievements from disk after cloud pull/resolve operations |
| `input_routing.rs` | `InputAction` enum and `route_game_input()` -- maps `InputResult` variants to side effects (save, quit, etc.); bridges input handling and persistence |
| `offline.rs` | `resolve_deep_offline()` -- resolves Deep missions completed while game was closed; `apply_offline_xp()` -- processes offline XP progression with combat log entries |
| `overlay.rs` | `draw_game_overlays()` -- renders all active game overlays on top of the main UI (modals, Haven, Soulforge, Stormglass, Deep, Chrono Surge, debug menu, save indicator) |
| `persistence.rs` | `save_all()` -- saves character, achievements, Haven, enhancement, and Deep state to disk; optionally creates a git history commit |
| `scene.rs` | `is_realtime_minigame()`, `SceneKind` enum, `current_scene_kind()`, `is_wide_scene()` -- scene classification helpers for terminal redraw management |
| `update.rs` | Update check helpers, jittered interval, startup splash screen with character select, cloud sync polling, and the full character select loop |

## Pattern

Each file wraps a single concern that would otherwise clutter `main.rs`:

- **achievements.rs**: Two-phase design -- `game_tick()` handles autonomous events; `track_input_achievements()` handles the two input-driven cases (prestige, minigame wins). This split avoids expanding `TickResult`'s interface.
- **character_screens.rs**: Each `handle_*_frame()` function is called once per frame iteration in the main loop. Returns `ScreenTransition` (Stay/GoToSelect/Quit) to control flow.
- **input_routing.rs**: Acts as a bridge between `handle_game_input()` (which returns `InputResult`) and the main loop. Translates results into save calls and loop control actions. Some `InputResult` variants (StartChronoSurge, Time Vault actions, cloud actions) are handled directly in `main.rs` before reaching `route_game_input()`.
- **persistence.rs**: `save_all()` is the single save entry point. Saves all JSON files, then optionally creates a git commit. Called from input routing on `NeedsSave*` results and on quit.
- **overlay.rs**: `draw_game_overlays()` is a large match over `GameOverlay` variants plus checks for open UI states (haven_ui.showing, soulforge_ui.open, etc.). Draws in layered order so modals appear on top of overlays.

## Integration Points

- **Bridges**: `main.rs` <-> domain modules (`character/`, `achievements/`, `haven/`, `enhancement/`, `deep/`, `history/`, `stormglass/`)
- **Input flow**: `main.rs` -> `input/mod.rs` (handle_game_input) -> `InputResult` -> `input_routing.rs` (route_game_input) -> `persistence.rs` (save_all)
- **Rendering flow**: `main.rs` -> `overlay.rs` (draw_game_overlays) -> `ui/` scene modules
- **Offline flow**: `main.rs` -> `offline.rs` (resolve_deep_offline, apply_offline_xp) -> domain modules
- **Cloud flow**: `main.rs` -> `cloud_ops.rs` (reload_account_state) -> persistence modules
