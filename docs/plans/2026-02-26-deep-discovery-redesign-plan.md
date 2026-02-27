# Deep Discovery Redesign — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the 30-prestige Rift Resonance story chain with a single trigger: killing The Endless (Zone 11 boss) at P15+.

**Architecture:** Remove all story chain machinery (rift resonance, 10 stages, story modals). Add a discovery check in `tick_stages.rs` when `BossDefeatResult::ExpanseCycle` fires. Rewrite `discovery.rs` to a simple `complete_discovery()`. Remove `rift_hint`/`rift_resonance` parameter threading from all UI functions.

**Tech Stack:** Rust, Ratatui

---

### Task 1: Rewrite `discovery.rs` — remove story chain, simplify to boss-trigger discovery

**Files:**
- Modify: `src/deep/discovery.rs`

**Step 1: Rewrite discovery.rs**

Remove `advance_deep_story()`. Rewrite `complete_story_discovery()` to `complete_discovery()` — remove the `deep_story_stage` guard (just check `discovered`). Keep `queue_first_orders()` unchanged.

```rust
use super::mercenaries::generate_starter_roster;
use super::types::{DeepState, MercStatus, Mission, MissionStatus, MissionType};
use chrono::Utc;
use rand::Rng;

/// Complete The Deep discovery. Called when the player kills The Endless
/// (Zone 11 boss) for the first time at P15+.
pub fn complete_discovery<R: Rng>(deep: &mut DeepState, rng: &mut R) {
    if deep.persistent.discovered {
        return;
    }
    deep.persistent.discovered = true;
    let starters = generate_starter_roster(
        deep.persistent.guild_rank,
        || deep.persistent.next_merc_id(),
        rng,
    );
    deep.prestige.roster.extend(starters);
    deep.prestige.available_missions =
        super::missions::generate_mission_pool(&deep.persistent, rng);
    deep.prestige.pool_refreshed_at = Some(Utc::now());
    deep.prestige.warband_marks = match deep.persistent.guild_rank.0 {
        1 => 50,
        2 => 100,
        3 => 200,
        4 => 350,
        5 => 500,
        _ => 50,
    };
    queue_first_orders(deep);
}

// queue_first_orders stays exactly the same
```

**Step 2: Verify it compiles**

Run: `cargo check 2>&1 | head -20`
Expected: Compile errors in files that reference old functions (mod.rs, prestige_input.rs, etc.) — that's expected, we'll fix those next.

**Step 3: Commit**

```
git add src/deep/discovery.rs
git commit -m "refactor(deep): rewrite discovery.rs — remove story chain, simplify to complete_discovery()"
```

---

### Task 2: Remove story chain constants and fields from `types.rs`

**Files:**
- Modify: `src/deep/types.rs`

**Step 1: Remove story chain constants**

Delete lines 15-26 (the `STORY_RESONANCE_THRESHOLDS`, `STORY_STAGE_ENTRANCE`, `STORY_STAGE_DISCOVERED` constants).

**Step 2: Remove `rift_resonance` and `deep_story_stage` fields from `DeepPersistent`**

In the struct definition (~line 696-701), remove both fields. In `DeepPersistent::new()` (~line 730-731), remove the initializers.

**Step 3: Remove `maybe_increment_rift_resonance()` and `check_story_progression()` methods**

Delete the `maybe_increment_rift_resonance()` method (~lines 917-924) and `check_story_progression()` method (~lines 926-948) from `impl DeepState`.

**Step 4: Remove `pending_story_stage` from `DeepUiState`**

Remove the field (~line 1077) and its initializer (~line 1099).

**Step 5: Verify it compiles**

Run: `cargo check 2>&1 | head -30`
Expected: More compile errors from consumers — expected.

**Step 6: Commit**

```
git add src/deep/types.rs
git commit -m "refactor(deep): remove rift resonance, story stage fields, and story chain methods"
```

---

### Task 3: Update `mod.rs` re-exports

**Files:**
- Modify: `src/deep/mod.rs`

**Step 1: Update re-exports**

Remove from the `pub use types` block: `STORY_RESONANCE_THRESHOLDS`, `STORY_STAGE_DISCOVERED`, `STORY_STAGE_ENTRANCE`.

Replace the discovery re-export line:
```rust
// Old:
pub use discovery::{advance_deep_story, complete_story_discovery};
// New:
pub use discovery::complete_discovery;
```

**Step 2: Commit**

```
git add src/deep/mod.rs
git commit -m "refactor(deep): update mod.rs re-exports for simplified discovery"
```

---

### Task 4: Remove story chain code from prestige input

**Files:**
- Modify: `src/input/prestige_input.rs`

**Step 1: Remove rift resonance and story chain calls**

In `handle_vault_selection()` (~lines 71-80): Delete the block that calls `maybe_increment_rift_resonance()` and `advance_deep_story()` / sets `pending_story_stage`.

In `handle_prestige_confirm()` (~lines 153-162): Same removal.

Remove the `deep_ui` parameter from both functions if it's only used for `pending_story_stage` — check first whether `farewell_mercs` still needs it (it does). Keep `deep_ui` param, just remove the story chain lines.

**Step 2: Verify it compiles**

Run: `cargo check 2>&1 | head -20`

**Step 3: Commit**

```
git add src/input/prestige_input.rs
git commit -m "refactor(deep): remove rift resonance and story chain from prestige input"
```

---

### Task 5: Remove story modal UI and pending_story_stage input handling

**Files:**
- Modify: `src/ui/deep_scene.rs`
- Modify: `src/main_helpers/overlay.rs`
- Modify: `src/input/mod.rs`

**Step 1: Remove story modal functions from `deep_scene.rs`**

Delete `story_modal_content()` (~lines 565-706) and `render_story_modal()` (~lines 708-756).

**Step 2: Remove story modal rendering from `overlay.rs`**

Delete the block at ~lines 263-266:
```rust
if let Some(stage) = deep_ui.pending_story_stage {
    ui::deep_scene::render_story_modal(frame, area, stage);
}
```

**Step 3: Remove story modal input handling from `input/mod.rs`**

Delete the block at ~lines 77-83:
```rust
// 0.4. Deep story event modal (Enter or Esc dismisses)
if deep_ui.pending_story_stage.is_some() {
    ...
}
```

**Step 4: Verify it compiles**

Run: `cargo check 2>&1 | head -20`

**Step 5: Commit**

```
git add src/ui/deep_scene.rs src/main_helpers/overlay.rs src/input/mod.rs
git commit -m "refactor(deep): remove story modal UI and input handling"
```

---

### Task 6: Remove `rift_hint` and `rift_resonance` parameter threading from UI

**Files:**
- Modify: `src/ui/prestige_confirm.rs`
- Modify: `src/ui/stats_prestige.rs`
- Modify: `src/ui/stats_panel.rs`
- Modify: `src/ui/mod.rs`
- Modify: `src/main_helpers/overlay.rs`

**Step 1: Simplify `draw_prestige_confirm()`**

Remove `rift_hint` parameter. Remove the `if rift_hint { ... }` block (~lines 101-115). Set `base_height` to always `18`.

**Step 2: Simplify `draw_prestige_panel()` in `stats_prestige.rs`**

Remove `rift_hint` and `rift_resonance` parameters. Remove the `if rift_hint { ... }` conditional (~lines 214-232) — always use the `unlock_hint` path.

**Step 3: Remove `rift_hint`/`rift_resonance` from `draw_stats_panel()` in `stats_panel.rs`**

Remove both parameters from the function signature (~lines 69-70) and the call to `draw_prestige_panel()` (~lines 101-102).

**Step 4: Remove from `draw_xl_l_layout()` and `draw_game_layout()` in `ui/mod.rs`**

Remove both parameters from `draw_xl_l_layout()` signature (~lines 456-457) and its call site (~lines 416-417). Remove the `rift_hint`/`rift_resonance` computation (~lines 396-399) from `draw_game_layout()`.

**Step 5: Remove from `draw_prestige_confirm` call in `overlay.rs`**

In `draw_game_overlays()` (~lines 108-111): Remove the `rift_hint` local and pass only `(frame, state, ctx)`.

**Step 6: Verify it compiles**

Run: `cargo check 2>&1 | head -20`

**Step 7: Commit**

```
git add src/ui/prestige_confirm.rs src/ui/stats_prestige.rs src/ui/stats_panel.rs src/ui/mod.rs src/main_helpers/overlay.rs
git commit -m "refactor(deep): remove rift_hint/rift_resonance UI parameter threading"
```

---

### Task 7: Add discovery trigger on ExpanseCycle in tick_stages.rs

**Files:**
- Modify: `src/core/tick_stages.rs`
- Modify: `src/core/tick_types.rs`

**Step 1: Add `TickEvent::DeepDiscovered` variant**

In `tick_types.rs`, add to the Discovery section (after `StormglassDiscovered`):
```rust
/// The Deep was discovered (first Endless kill at P15+).
DeepDiscovered,
```

**Step 2: Add `deep_discovered` to `TickResult`**

Not needed — `deep_changed` already exists and covers this.

**Step 3: Trigger discovery on `ExpanseCycle`**

In `tick_stages.rs`, in the `process_combat_events()` function, find the `BossDefeatResult::ExpanseCycle` match arm (~line 558). After the existing message formatting, add the discovery check:

```rust
BossDefeatResult::ExpanseCycle => {
    // ... existing message code ...

    // Check for Deep discovery on first Endless kill
    if !deep.persistent.discovered
        && state.prestige_rank >= crate::deep::DEEP_MIN_PRESTIGE_RANK
    {
        crate::deep::complete_discovery(deep, rng);
        result.events.push(TickEvent::DeepDiscovered);
        result.deep_changed = true;
        achievements.on_deep_discovered(Some(&state.character_name));
        if !debug_mode {
            result.achievements_changed = true;
        }
    }
}
```

This requires adding `deep: &mut DeepState` and `debug_mode: bool` parameters to `process_combat_events()`. Update the function signature and its call site in `tick.rs`.

**Step 4: Handle `TickEvent::DeepDiscovered` in `tick_events.rs`**

Add a match arm in the tick event processing that sets `deep_discovered` flag (same pattern as `haven_discovered`/`soulforge_discovered`). In `main.rs`, map this flag to `GameOverlay::DeepDiscovery`.

Check how `HavenDiscovered` and `SoulforgeDiscovered` are handled in `tick_events.rs` and `main.rs` first — follow the exact same pattern.

**Step 5: Verify it compiles**

Run: `cargo check 2>&1 | head -20`

**Step 6: Commit**

```
git add src/core/tick_stages.rs src/core/tick_types.rs src/tick_events.rs src/core/tick.rs
git commit -m "feat(deep): trigger Deep discovery on first Endless kill at P15+"
```

---

### Task 8: Update debug menu

**Files:**
- Modify: `src/utils/debug_menu.rs`

**Step 1: Update debug menu discovery shortcut**

In `trigger_deep_discovery()` (~line 626-631): Remove the `deep_story_stage` assignment. Change `complete_story_discovery` to `complete_discovery`:

```rust
fn trigger_deep_discovery(deep: &mut crate::deep::DeepState, _prestige_rank: u32) -> &'static str {
    let mut rng = rand::rng();
    crate::deep::complete_discovery(deep, &mut rng);
    "The Deep discovered!"
}
```

**Step 2: Commit**

```
git add src/utils/debug_menu.rs
git commit -m "refactor(deep): update debug menu to use simplified discovery"
```

---

### Task 9: Update tests

**Files:**
- Modify: `tests/deep_tutorial_test.rs`
- Modify: `tests/deep_types_coverage_test.rs`
- Modify: `tests/deep_prestige_persistence_test.rs`
- Modify: `tests/deep_integration_test.rs`

**Step 1: Rewrite story chain tests in `deep_tutorial_test.rs`**

The tests starting around line 880 (`test_deep_story_chain_full_progression`, `test_deep_story_final_stage_requires_resonance_and_p15`, `test_deep_discovery_only_once_via_game_tick`, `test_rift_resonance_only_increments_in_expanse`) test the removed story chain. Replace them with a single test for the new trigger:

```rust
#[test]
fn test_deep_discovery_on_endless_kill() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut deep = DeepState::new();

    // Discovery requires !discovered
    assert!(!deep.persistent.discovered);

    quest::deep::complete_discovery(&mut deep, &mut rng);
    assert!(deep.persistent.discovered);
    assert_eq!(deep.prestige.roster.len(), 3);
    assert_eq!(deep.prestige.warband_marks, 50);
    assert!(!deep.prestige.active_missions.is_empty()); // First Orders queued
}

#[test]
fn test_deep_discovery_is_idempotent() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut deep = DeepState::new();

    quest::deep::complete_discovery(&mut deep, &mut rng);
    let roster_count = deep.prestige.roster.len();

    // Calling again should not double-init
    quest::deep::complete_discovery(&mut deep, &mut rng);
    assert_eq!(deep.prestige.roster.len(), roster_count);
}
```

**Step 2: Fix any tests referencing removed fields**

Search test files for `rift_resonance`, `deep_story_stage`, `advance_deep_story`, `complete_story_discovery`, `STORY_STAGE_ENTRANCE`, `STORY_STAGE_DISCOVERED`, `STORY_RESONANCE_THRESHOLDS`, `pending_story_stage`, `maybe_increment_rift_resonance`, `check_story_progression`. Remove or update each reference.

**Step 3: Run the full test suite**

Run: `cargo test 2>&1 | tail -20`
Expected: All tests pass.

**Step 4: Commit**

```
git add tests/
git commit -m "test(deep): update tests for boss-trigger discovery, remove story chain tests"
```

---

### Task 10: Update documentation

**Files:**
- Modify: `src/deep/CLAUDE.md`
- Modify: `CLAUDE.md`

**Step 1: Update `src/deep/CLAUDE.md`**

Update the Discovery section to reflect boss-trigger instead of story chain. Remove references to Rift Resonance, story stages, and per-tick random roll. Update the constants table (remove story chain constants). Update integration points (discovery now happens in tick_stages.rs on ExpanseCycle, not in prestige_input.rs).

**Step 2: Update root `CLAUDE.md`**

Update The Deep Module description if it mentions story chain or Rift Resonance.

**Step 3: Commit**

```
git add src/deep/CLAUDE.md CLAUDE.md
git commit -m "docs: update CLAUDE.md files for boss-trigger Deep discovery"
```

---

### Task 11: Final verification

**Step 1: Run full CI checks**

Run: `make check`
Expected: All checks pass (format, clippy, test, build, audit).

**Step 2: Commit any formatting fixes**

Run: `make fmt` if needed, then commit.
