# Vessel Launch Gate & Construction Overlay

**Parent spec:** `docs/superpowers/specs/2026-03-27-the-vessel-design.md`
**Sub-project:** 1 of 7

## Overview

After conquering Zone 50, the player begins seeing hints about the dying branch of Yggdrasil. A new `[V]` hotkey opens the Vessel overlay showing construction progress toward launch. Prestige rank keeps ticking up as normal, and the player **spends** rank into "Vessel Fuel" (1:1, irreversible) until 100,000 is banked, then they can launch. This sub-project covers everything up to the launch confirmation — no Act 2 gameplay yet.

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

### Phase 4: PR Fuel Transfer

`vessel_fuel: u32` is a new persistent field on `GameState`.

**Prestige rank never freezes and PR grants are never diverted.** PR keeps ticking up from all sources exactly as before (WR→PR, Power Cores, challenges, prestige actions — none of that code is touched). Instead, the player **spends prestige rank as fuel**, 1 PR : 1 fuel, from within the Vessel overlay. Burning your accumulated rank into the ship is the mechanic — reality itself is the fuel, which is the parent spec's fiction made literal.

Transfer controls in the overlay:

```
Prestige: P61,420          Fuel: 38,500 / 100,000
[1] Transfer 1,000   [2] Transfer 10,000   [3] Transfer Max
```

- Transfers clamp to available rank and to the 100,000 cap. Irreversible — fuel cannot be refunded to rank.
- Each transfer calls `recalculate_prestige_bonuses()` and sets `derived_stats_dirty` — spending rank genuinely weakens the hero's prestige bonuses until income regrows it. A real sacrifice, but a temporary one at endgame generation rates.
- Spending below P50,000 shows a warning line ("The Loom's furthest threads will slacken") but is allowed. Zone unlocks are safe: `sync_account_zone_unlocks` (`src/zones/access.rs`) never removes an unlock once granted, so dropping below a zone's prestige requirement cannot re-lock it.
- Transfers are available as soon as `vessel_signal_discovered` is set — the other launch gates (patterns, Ascension X) are independent checkmarks, not prerequisites for banking fuel.

**Consequences of this model:**
- A veteran sitting on a large PR bank (say P150k+) can fund the entire 100,000 immediately and launch the moment the signal appears. They earned it.
- A player arriving at the gate near P50,000 who wants to keep their rank generates fresh PR: ~14 days at a maxed Loom (303 PR/hr), ~54 days at typical pattern-28 rates (75 PR/hr).
- The choice of *when* and *how deep* to burn is the construction-watch gameplay: dip below P50k and launch sooner at weaker bonuses, or cruise while the Loom refills the bank.

The overlay shows:
- Current prestige rank (live — it keeps ticking up during the watch)
- Current fuel / 100,000 with a progress bar
- Transfer controls (`[1]` 1,000 / `[2]` 10,000 / `[3]` Max)
- Estimated PR/day generation rate and days-to-full if the player banked everything as it arrives

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

## Fuel Transfer Logic

A single action handler in the Vessel overlay input path — no changes to any PR grant site:

```
fn transfer_fuel(state: &mut GameState, requested: u32) {
    if !state.vessel_signal_discovered || state.vessel_launched {
        return;
    }
    let room = 100_000u32.saturating_sub(state.vessel_fuel);
    let amount = requested.min(state.prestige_rank).min(room);
    if amount == 0 {
        return;
    }
    state.prestige_rank -= amount;
    state.vessel_fuel += amount;
    state.recalculate_prestige_bonuses();
    state.derived_stats_dirty = true;
}
```

(`vessel_signal_discovered` already implies Z50 was cleared — no separate `z50_cleared` flag is needed. The other launch gates are checked at launch time, not at transfer time.)

Note this is far simpler than intercepting PR income: grants remain untouched at all five sites (prestige action, WR→PR, Power Cores online/offline, challenge rewards), and achievement passive-PR tracking (#612) keeps working unmodified because rank genuinely goes up before the player chooses to spend it.

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
- **Within overlay:** `[Esc]` closes. `[1]`/`[2]`/`[3]` transfer 1,000 / 10,000 / Max PR to fuel. `[Enter]` triggers launch when ready. Arrow keys unused for now.

## UI Rendering

Single scene rendered into a buffer (like Deep). No sub-views for sub-project 1 — just the construction screen.

Layout:
```
Row 0-8:   Ship ASCII art + narrative text
Row 9:     Separator
Row 10-13: Four requirement lines with ✓/✗
Row 14:    Prestige balance + fuel gauge with progress bar
Row 15:    Transfer controls ([1] 1,000  [2] 10,000  [3] Max)
Row 16:    Generation rate + estimate (+ P50k warning line when relevant)
Row 17:    Footer ([Enter] Launch / [Esc] Close)
```

## Files Changed

| File | Change |
|------|--------|
| `src/core/game_state.rs` | Add `vessel_signal_discovered`, `vessel_fuel`, `vessel_launched` fields |
| `src/core/tick_types.rs` | Add `VesselSignalDiscovered` tick event variant |
| `src/core/tick_stages.rs` | Detect Z50 boss kill, emit event |
| `src/tick_events.rs` | Handle `VesselSignalDiscovered` flag, add ticker messages |
| `src/input/mod.rs` | Add `[V]` hotkey, vessel overlay dispatch |
| `src/input/types.rs` | Add `GameOverlay::VesselDiscovery` variant + `VesselUiState` |
| `src/ui/vessel_scene.rs` | New file: render construction overlay |
| `src/ui/mod.rs` | Register vessel_scene module |
| `src/ui/stats_panel.rs` | Show vessel indicator line |
| `src/main.rs` | Wire vessel overlay into render loop |

## Testing

- Unit test: transfer moves PR to fuel 1:1 and recalculates prestige bonuses
- Unit test: transfer clamps to available rank and to the 100,000 cap
- Unit test: transfer refused before signal discovery and after launch
- Unit test: PR grants (WR→PR, Power Cores, challenges) are untouched during the fuel phase — rank keeps rising
- Unit test: `vessel_signal_discovered` set on Z50 boss kill
- Unit test: serde round-trip for new GameState fields (backwards compat)
- Add save-format compatibility fixtures for the new fields (fixture corpus from #626)
- Snapshot test: construction overlay scene (full-frame TUI snapshot infra from #623/#624)
