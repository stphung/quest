# Deep Discovery Redesign

## Problem

The Deep requires 30 Rift Resonance (earned 1 per prestige from Zone 11) across a 10-stage story chain. This is an excessive time gate for an endgame system already behind P15+. The escalating cost curve is invisible to players, making progress feel like a flat 30-prestige grind.

## Design

Replace the Rift Resonance story chain with a single trigger: killing The Endless (Zone 11 subzone 4 boss) for the first time at P15+.

### Trigger

First `BossDefeatResult::ExpanseCycle` where `prestige_rank >= 15` and `!deep.persistent.discovered`.

### Player Experience

```
Kill The Endless (first time, P15+)
  → Single story modal: narrative moment about the earth cracking open
  → [Enter] to dismiss
  → Discovery modal: "The Deep Discovered!" (existing, unchanged)
  → Starter mercs created, First Orders mission queued
  → [D] keybind available
```

### Removals

- `rift_resonance` field on `DeepPersistent` (with `#[serde(default)]` for migration)
- `deep_story_stage` field and `STORY_STAGE_ENTRANCE`, `STORY_STAGE_DISCOVERED` constants
- `STORY_RESONANCE_THRESHOLDS` array
- `advance_deep_story()` in `discovery.rs`
- `check_story_progression()` method on `DeepState`
- `pending_story_stage` on `DeepUiState`
- `render_story_modal()` and `story_modal_content()` (10 story stages) in `deep_scene.rs`
- Rift Resonance display in stats panel (`Rift: X/30 · The Expanse responds to prestige`)
- Rift hint in prestige confirm dialog (`The Rift will remember this.`)
- `maybe_increment_rift_resonance()` and its call sites in `prestige_input.rs`
- `rift_hint` parameter threading through `draw_prestige_confirm`, `draw_stats_panel`, `draw_game_layout`

### Additions

- In `tick_stages.rs`, handle `BossDefeatResult::ExpanseCycle`: check discovery conditions, call `complete_story_discovery()`, emit `TickEvent::DeepDiscovered`
- Single story modal text for the boss-kill moment (replaces 10 stages)
- New field or reuse existing mechanism to show the story modal before the discovery modal

### Unchanged

- `complete_story_discovery()` internals (starter mercs, First Orders mission)
- Discovery modal UI (`render_deep_discovery_modal`)
- Debug menu "Discover The Deep" shortcut
- All post-discovery gameplay (missions, roster, layers, infrastructure)
- `DeepPersistent.discovered` field and save/load

### Migration

Existing saves with `rift_resonance > 0` but `discovered = false` will simply discover The Deep on their next Endless kill. Players who already discovered The Deep are unaffected — `discovered = true` is already set.

Fields removed from the struct use `#[serde(default)]` so old saves deserialize without error.
