# Main Helpers Module

Thin orchestration wrappers extracted from `main.rs` to keep the game loop readable. Each file encapsulates one concern.

## Files

| File | Purpose |
|------|---------|
| `mod.rs` | Module declarations and public re-exports |
| `achievements.rs` | `log_synced_achievements()` -- logs newly synced achievements to combat log; `track_input_achievements()` -- tracks manual prestige, fishing rank, and minigame win achievements triggered directly by player input |
| `chrono_surge.rs` | Chrono Surge batched tick execution for accelerated gameplay |
| `cloud_sync.rs` | Cloud sync state and operations (polling results, dispatching cloud input actions) |
| `game_context.rs` | Shared game context structs to reduce argument counts on hot-path functions |
| `character_screens.rs` | `ScreenTransition` enum and frame handlers for character creation, deletion, and rename screens; each draws UI, polls input, returns transition |
| `cloud_ops.rs` | `reload_account_state()` -- reloads Haven, Enhancement, and Achievements from disk after cloud pull/resolve operations |
| `input_routing.rs` | `InputAction` enum and `route_game_input()` -- maps `InputResult` variants to side effects (save, quit, etc.); bridges input handling and persistence |
| `offline.rs` | `resolve_deep_offline()` -- resolves Deep missions completed while game was closed; `apply_offline_xp()` -- processes offline XP progression with combat log entries; `resolve_loom_offline()` -- simulates Loom production while game was closed |
| `overlay.rs` | `draw_game_overlays()` -- renders all active game overlays on top of the main UI (modals, Haven, Soulforge, Stormglass, Deep, Loom, Chrono Surge, debug menu, save indicator) |
| `persistence.rs` | `save_all()` -- saves character, achievements, Haven, enhancement, and Deep state to disk; optionally creates a git history commit |
| `scene.rs` | `is_realtime_minigame()`, `SceneKind` enum, `current_scene_kind()`, `is_wide_scene()` -- scene classification helpers for terminal redraw management |
| `update.rs` | Update check helpers, jittered interval, startup splash screen with character select, cloud sync polling, and the full character select loop |

## Pattern

Each file wraps a single concern that would otherwise clutter `main.rs`:

- **achievements.rs**: Prestige achievement progress is tracked via three paths: `track_input_achievements()` reports manual prestige (Enter on the prestige dialog) and minigame wins driven directly by player input; `core::tick_stages::track_passive_prestige_gain()` (called from `game_tick()`, stage 12b) reports passive PR gains from Power Cores and WR->PR conversion; and `main_helpers/update.rs::load_character_for_game()` reports `on_prestige()` after offline Power Core catchup. This keeps input-driven tracking out of `TickResult` while still catching prestige gains that happen autonomously inside the tick or during offline resolution.
- **character_screens.rs**: Each `handle_*_frame()` function is called once per frame iteration in the main loop. Returns `ScreenTransition` (Stay/GoToSelect/Quit) to control flow.
- **input_routing.rs**: Acts as a bridge between `handle_game_input()` (which returns `InputResult`) and the main loop. Translates results into save calls and loop control actions. Some `InputResult` variants (StartChronoSurge, Time Vault actions, cloud actions) are handled directly in `main.rs` before reaching `route_game_input()`.
- **persistence.rs**: `save_all()` is the single save entry point. Saves all JSON files, then optionally creates a git commit. Called from input routing on `NeedsSave*` results and on quit.
- **overlay.rs**: `draw_game_overlays()` is a large match over `GameOverlay` variants plus checks for open UI states (haven_ui.showing, soulforge_ui.open, etc.). Draws in layered order so modals appear on top of overlays.

## Integration Points

- **Bridges**: `main.rs` <-> domain modules (`character/`, `achievements/`, `haven/`, `enhancement/`, `deep/`, `history/`, `stormglass/`, `loom/`)
- **Input flow**: `main.rs` -> `input/mod.rs` (handle_game_input) -> `InputResult` -> `input_routing.rs` (route_game_input) -> `persistence.rs` (save_all)
- **Rendering flow**: `main.rs` -> `overlay.rs` (draw_game_overlays) -> `ui/` scene modules
- **Offline flow**: `main.rs` -> `offline.rs` (resolve_deep_offline, apply_offline_xp, resolve_loom_offline) -> domain modules
- **Cloud flow**: `main.rs` -> `cloud_ops.rs` (reload_account_state) -> persistence modules
