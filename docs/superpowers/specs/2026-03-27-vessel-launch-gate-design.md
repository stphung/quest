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

These fire roughly every 60 seconds. They use a dim color (e.g. `Color::Rgb(120, 90, 160)`) with a `✦` icon.

**Implementation note:** the scrolling ticker (`src/core/ticker.rs`) is purely event-driven — nothing currently pushes to it on a timer. This needs a small new mechanism: a wall-clock check in the tick system (elapsed >= 60s since last hint → push a random hint entry via `Ticker::push`). The closest existing wall-clock rotation precedent is the Deep hub's atmosphere rotation (`millis / 8000 % len` in `src/ui/deep_missions.rs`), but that rotates panel text rather than pushing ticker entries.

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

**Vessel fuel auto-accumulates from all PR sources once all non-PR gates are met** (28 patterns, Asc X, Z50 cleared). PR earned from any source flows into `vessel_fuel` instead of `prestige_rank`. This makes the fuel gauge fill automatically without player action — fits the idle game design.

**Implementation note:** PR grants are decentralized — there is no central "grant PR" helper. The diversion must be applied at every grant site:

| Grant site | Location |
|-----------|----------|
| Prestige action | `src/character/prestige_actions.rs` (`perform_prestige`) |
| WR→PR conversion | `src/core/tick_stages.rs` (`tick_loom` stage) |
| Power Cores (online) | `src/power_cores/tick.rs` (`tick_power_cores`) |
| Power Cores (offline) | `src/power_cores/tick.rs` (`apply_offline_power_cores`) |
| Challenge rewards | `src/challenges/mod.rs` (`apply_challenge_rewards`) |

The cleanest approach is to introduce a `grant_pr(state, amount)` helper that routes to `vessel_fuel` when the gates are met, and refactor all five sites to use it. Two interactions to preserve: (1) `recalculate_prestige_bonuses()` is called after most grants — diverted PR should skip it (rank didn't change); (2) achievements track passive PR gains for prestige milestones (#612) — decide whether fuel-diverted PR still counts toward those milestones (recommended: no, fuel is not rank).

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

At each PR grant site (via the `grant_pr` helper):

```
if vessel_signal_discovered      // set by Z50 final boss kill — see Z50 Detection
   && ascension_level >= 10
   && completed_patterns >= 28
   && !vessel_launched
   && vessel_fuel < 100_000
{
    // Divert new PR to vessel fuel instead of prestige_rank
    vessel_fuel += pr_earned;
    // Don't add to prestige_rank
}
```

(`vessel_signal_discovered` already implies Z50 was cleared — no separate `z50_cleared` flag is needed.)

This means once all gates are met, the player stops gaining prestige rank and starts filling the fuel gauge instead. This is intentional — PR beyond P50,000 has no gameplay value, so diverting it is painless.

## Z50 Detection

Zone 50 is a Loom cap zone — killing its subzone 5 boss produces `BossDefeatResult::LoomZoneCycle { zone_id: 50 }` (see `src/zones/boss_defeat.rs`). There is no distinct "final boss of the game" result; Z50 cycles like any cap zone.

Detection follows the exact precedent of the Deep discovery in `src/core/tick_stages.rs` (`process_combat_events`, ~line 490): match the defeat result, set the flag, emit a tick event:

```rust
if matches!(defeat_result, BossDefeatResult::LoomZoneCycle { zone_id: 50 })
    && !state.vessel_signal_discovered
{
    state.vessel_signal_discovered = true;
    result.events.push(TickEvent::VesselSignalDiscovered);
}
```

The presentation layer handles the event the same way `TickEvent::DeepDiscovered` is handled (`src/tick_events.rs` → flag → `src/main.rs` pushes a discovery overlay onto `pending_overlays`).

## Input Integration

The codebase has two overlay mechanisms: the `GameOverlay` enum (`src/input/types.rs`) for one-shot modals/celebrations, and standalone `*UiState` structs (like `DeepUiState`, `LoomUiState`) held in `main.rs` for large interactive scenes. The Vessel uses both:

- **Discovery modal:** `GameOverlay::VesselDiscovery` unit variant — the one-time reveal celebration when the signal is discovered (same as `DeepDiscovery`)
- **Construction overlay:** a new `VesselUiState` struct with a `showing: bool` (the Deep/Loom pattern), dispatched from Step 2 of the input priority chain via a new `src/input/vessel_input.rs`
- **Hotkey:** `[V]` in the base-hotkey block (Step 9 of the priority chain in `src/input/mod.rs`), gated by `vessel_signal_discovered`
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
| `src/core/tick_stages.rs` | Detect Z50 boss kill, emit event; divert WR→PR grants to fuel |
| `src/character/prestige_actions.rs` | Route prestige PR through `grant_pr` helper |
| `src/power_cores/tick.rs` | Route Power Core PR (online + offline) through `grant_pr` helper |
| `src/challenges/mod.rs` | Route challenge reward PR through `grant_pr` helper |
| `src/tick_events.rs` | Handle `VesselSignalDiscovered` flag, add ticker messages |
| `src/input/mod.rs` | Add `[V]` hotkey, vessel overlay dispatch |
| `src/input/types.rs` | Add `GameOverlay::VesselDiscovery` variant + `VesselUiState` |
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
- Add save-format compatibility fixtures for the new fields (fixture corpus from #626)
- Snapshot test: construction overlay scene (full-frame TUI snapshot infra from #623/#624)
