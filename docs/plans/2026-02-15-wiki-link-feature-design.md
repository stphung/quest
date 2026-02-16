# Wiki Link Feature Design

**Date:** 2026-02-15
**Status:** Approved

## Goal

Surface the wiki URL (`github.com/stphung/quest/wiki`) to players in contextual locations so they know where to find guides, strategies, and game mechanics documentation.

## Placements

### 1. Help Overlay (`[?]` keybinding)

A new `GameOverlay::Help` variant renders a centered modal overlay accessible from both the character select screen and the main game screen via the `?` key.

**Main game mockup (L/XL):**
```
+-- Help -----------------------------------------------+
|                                                        |
|  Controls                                              |
|  [P] Prestige    [H] Haven    [S] Soulforge            |
|  [A] Achievements    [Tab] Challenges                  |
|  [U] Toggle Updates    [Esc] Quit                      |
|                                                        |
|  Quest Wiki                                            |
|  github.com/stphung/quest/wiki                         |
|  Guides for combat, zones, prestige, and more.         |
|                                                        |
|  [Esc] Close                                           |
+--------------------------------------------------------+
```

Dismissed with `Esc` or `?` (toggle).

### 2. Character Select Controls

Add `[?] Help` to the controls bar on the character select screen:

**Large:**
```
[Enter] Play    [R] Rename    [D] Delete    [N] New    [Esc] Quit
[A] Achievements    [?] Help
```

**Compact (S):**
```
[Enter] Play  [R] Rename  [D] Del  [N] New  [Esc] Quit
[A] Achv  [?] Help
```

### 3. Update Drawer

Append the wiki URL after the existing "Run 'quest update' to install" line in the update drawer:

```
  Run 'quest update' to install
  Wiki: github.com/stphung/quest/wiki
```

## Implementation

### Constant

```rust
// src/core/constants.rs
pub const WIKI_URL: &str = "github.com/stphung/quest/wiki";
```

### GameOverlay

```rust
// src/input.rs
pub enum GameOverlay {
    None,
    Help,  // new
    HavenDiscovery,
    ...
}
```

### Help Overlay UI

New file `src/ui/help_overlay.rs` following the `prestige_confirm.rs` pattern:
- Centered Clear + bordered Paragraph overlay
- Controls summary section
- Wiki URL section
- `[Esc] Close` footer

### Input Routing

- Main game: `?` key sets `*overlay = GameOverlay::Help`
- Help overlay: `Esc` or `?` sets `*overlay = GameOverlay::None`
- Character select: `?` key triggers the same help overlay

### Files Touched

| File | Change |
|------|--------|
| `src/core/constants.rs` | Add `WIKI_URL` constant |
| `src/input.rs` | Add `Help` to `GameOverlay`, input routing for `?` |
| `src/ui/help_overlay.rs` | New file: help overlay rendering |
| `src/ui/mod.rs` | Register module, dispatch in draw |
| `src/ui/stats_panel.rs` | Add wiki line to update drawer |
| `src/ui/character_select.rs` | Add `[?] Help` to controls bar |
| `src/main.rs` or `src/character/input.rs` | Handle `?` in character select state |
