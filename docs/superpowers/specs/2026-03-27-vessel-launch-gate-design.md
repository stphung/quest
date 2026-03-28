# Vessel Launch Gate & Construction Overlay

**Parent spec:** `docs/superpowers/specs/2026-03-27-the-vessel-design.md`
**Sub-project:** 1 of 7

## Overview

After conquering Zone 50, the player begins seeing hints about the dying branch of Yggdrasil. A new `[V]` hotkey opens the Vessel overlay showing construction progress toward launch. PR accumulates as "Vessel Fuel" until 100,000 is reached, then the player can launch. This sub-project covers everything up to the launch confirmation — no Act 2 gameplay yet.

## Phases

### Phase 1: Ticker Hints (after Z50 first clear)

When the player first clears Zone 50's final boss (subzone 5 boss kill), a `VesselSignalDiscovered` tick event fires. This sets a persistent flag `vessel_signal_discovered: bool` on `GameState`.

Once the flag is set, atmospheric ticker messages appear periodically (mixed in with normal loot):
- "The Loom trembles. Something distant answers."
- "A signal pulses from beyond the branches."
- "The Origin Thread frays. The roots grow cold."
- "The weave resonates with something far away."
- "Yggdrasil shudders. A beacon calls."

These fire roughly every 60 seconds via the tick system (similar to existing atmospheric ticker messages). They use a dim color (e.g. `Color::Rgb(120, 90, 160)`) with a `✦` icon.

### Phase 2: Stats Panel Indicator

Once `vessel_signal_discovered` is true, the stats panel shows a new line:

```
Prestige: P52,340 (Eternal)
Ascension: X (324x)
✦ The branch withers...        [V] Vessel
```

The `[V] Vessel` hint pulses between dim and bright (using the existing tick millis for animation). Pressing `[V]` opens the Vessel overlay.

### Phase 3: Vessel Overlay

A full-screen overlay (like Haven, Deep, Soulforge) opened via `[V]`. Contains:

**Construction screen** showing the four requirements with check/cross indicators, a fuel gauge, and generation rate estimate.

The overlay renders into the scene buffer (same pattern as Deep overlay — `render_vessel_scene()` paints to a buffer, then the buffer is flushed to frame).

### Phase 4: PR Fuel Accumulation

`vessel_fuel: u32` is a new persistent field on `GameState`. It accumulates separately from `prestige_rank` — the player's rank is not reduced until launch.

**Fuel does NOT auto-accumulate from PR generation.** Instead, the player manually "transfers" PR to vessel fuel from within the Vessel overlay. This gives the player control over when to invest PR vs keep it for other uses (if any remain).

Actually, simpler: **vessel fuel auto-accumulates from all PR sources once all non-PR gates are met** (28 patterns, Asc X, Z50 cleared). PR earned from WR→PR, Power Cores, and prestige actions flows into `vessel_fuel` instead of `prestige_rank`. This makes the fuel gauge fill automatically without player action — fits the idle game design.

The overlay shows:
- Current fuel / 100,000
- A progress bar
- Estimated PR/day generation rate
- Estimated days remaining

### Phase 5: Launch Ready

When `vessel_fuel >= 100,000`, the overlay changes to show a "Launch" option. The requirements section shows all green checkmarks. A `[Enter] Launch into the Void` prompt appears.

### Phase 6: Launch Confirmation

Pressing Enter on the ready screen shows a confirmation modal:
- Lists what will happen (consume 100k PR worth of fuel, transform the Loom, begin voyage)
- "There is no return."
- `[Enter] Confirm  [Esc] Cancel`

On confirm: sets `vessel_launched: bool = true` on game state. The actual mode transition to Act 2 is handled by a future sub-project — for now, launch just sets the flag and shows a "Coming soon" message or similar placeholder.

## State Model

New fields on `GameState` (with `#[serde(default)]`):

```rust
/// True after Z50 final boss first kill — enables ticker hints and [V] hotkey
pub vessel_signal_discovered: bool,
/// Accumulated PR fuel for Vessel construction (0-100,000)
pub vessel_fuel: u32,
/// True after player confirms launch — triggers Act 2 mode shift (future sub-project)
pub vessel_launched: bool,
```

No separate module needed yet. These three fields on `GameState` are sufficient for sub-project 1.

## Fuel Accumulation Logic

In the tick system, after PR is normally granted (from WR→PR, Power Cores, or prestige):

```
if vessel_signal_discovered
   && ascension_level >= 10
   && completed_patterns >= 28
   && z50_cleared
   && !vessel_launched
   && vessel_fuel < 100_000
{
    // Divert new PR to vessel fuel instead of prestige_rank
    vessel_fuel += pr_earned_this_tick;
    // Don't add to prestige_rank
}
```

This means once all gates are met, the player stops gaining prestige rank and starts filling the fuel gauge instead. This is intentional — PR beyond P50,000 has no gameplay value, so diverting it is painless.

## Z50 Detection

Zone 50 cleared = the player has killed the subzone 5 boss in zone 50. This can be detected by checking `zone_progression.current_zone_id >= 50` after a boss kill, or by adding a `z50_cleared: bool` flag. The simplest approach: set `vessel_signal_discovered = true` when a `BossDefeatResult` fires while in zone 50, subzone 5 (the final boss rotation).

## Input Integration

- **Hotkey:** `[V]` in `handle_base_game()`, gated by `vessel_signal_discovered`
- **Overlay:** `VesselOverlay` variant in `GameOverlay` or a standalone `vessel_ui_open: bool` (follow the Deep/Loom pattern of a separate UI state bool)
- **Within overlay:** `[Esc]` closes. `[Enter]` triggers launch when ready. Arrow keys unused for now.

## UI Rendering

Single scene rendered into a buffer (like Deep). No sub-views for sub-project 1 — just the construction screen.

Layout:
```
Row 0-8:   Ship ASCII art + narrative text
Row 9:     Separator
Row 10-13: Four requirement lines with ✓/✗
Row 14-15: Fuel gauge with progress bar
Row 16:    Generation rate + estimate
Row 17:    Footer ([Enter] Launch / [Esc] Close)
```

## Files Changed

| File | Change |
|------|--------|
| `src/core/game_state.rs` | Add `vessel_signal_discovered`, `vessel_fuel`, `vessel_launched` fields |
| `src/core/tick_types.rs` | Add `VesselSignalDiscovered` tick event variant |
| `src/core/tick_stages.rs` | Detect Z50 boss kill, emit event; divert PR to vessel fuel |
| `src/tick_events.rs` | Handle `VesselSignalDiscovered` flag, add ticker messages |
| `src/input/mod.rs` | Add `[V]` hotkey, vessel overlay dispatch |
| `src/input/types.rs` | Add vessel UI state (or use existing pattern) |
| `src/ui/vessel_scene.rs` | New file: render construction overlay |
| `src/ui/mod.rs` | Register vessel_scene module |
| `src/ui/stats_panel.rs` | Show vessel indicator line |
| `src/main.rs` | Wire vessel overlay into render loop |

## Testing

- Unit test: fuel accumulation diverts PR when all gates met
- Unit test: fuel stops at 100,000 cap
- Unit test: fuel doesn't accumulate when gates not met
- Unit test: `vessel_signal_discovered` set on Z50 boss kill
- Unit test: serde round-trip for new GameState fields (backwards compat)
