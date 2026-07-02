# Vessel Launch Gate & Construction Overlay

**Parent spec:** `docs/superpowers/specs/2026-03-27-the-vessel-design.md`
**Sub-project:** 1 of 7

## Overview

After conquering Zone 50, the player begins seeing hints about the dying branch of Yggdrasil. A new `[V]` hotkey opens the Vessel overlay showing progress toward launch. Prestige rank keeps ticking up as normal; the fuel gate is simply **holding 250,000 PR at once**. Launch is a single all-or-nothing burn — confirming deducts the full 250,000 in one action. There is no partial banking and no fuel accumulator. This sub-project covers everything up to the launch confirmation — no Act 2 gameplay yet.

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

**Construction screen** showing the four requirements with check/cross indicators, a rank-progress bar toward the 250,000 burn, and a generation rate estimate.

The overlay renders into the scene buffer (same pattern as Deep overlay — `render_vessel_scene()` paints to a buffer, then the buffer is flushed to frame).

### Phase 4: The 250,000 PR Gate

**Prestige rank never freezes and PR grants are never diverted.** PR keeps ticking up from all sources exactly as before (WR→PR, Power Cores, challenges, prestige actions — none of that code is touched). The fuel gate is simply: **the player must hold 250,000 PR at the moment of launch.** There is no partial banking, no transfer controls, no `vessel_fuel` accumulator — the burn happens once, in full, when launch is confirmed.

The overlay shows current rank against the threshold:

```
Prestige: P152,840 / 250,000       ██████░░░░  61%
Income: ~7,320 PR/day — ready in ~13 days
```

**Consequences of this model:**
- The hero fights at full strength for the entire wait — rank (and its bonuses) stays intact until the single burn at launch.
- A veteran already holding 250k+ can launch the moment the signal appears. They earned it.
- A player arriving at the gate near P50,000 climbs to 250,000: ~108 days (~3.6 months) at typical pattern-28 rates (75 PR/hr), ~27 days at a maxed Loom (303 PR/hr). Maxing the extractors is effectively part of the launch grind at this price.
- The burn leaves `rank - 250,000` behind (usually a small remainder). Post-launch rank only matters to the background supply line (sub-project 7), and zone unlocks cannot re-lock (`sync_account_zone_unlocks` in `src/zones/access.rs` never removes an unlock), so there is no reason to over-save beyond the threshold.
- One dramatic moment instead of many small ones: the player watches everything they accumulated vanish in a single confirmed action. That IS the launch.

The overlay shows:
- Current prestige rank against the 250,000 threshold, with a progress bar (live — rank keeps ticking up during the watch)
- Estimated PR/day generation rate and days until the threshold is reached
- The other three requirements with ✓/✗

### Phase 5: Launch Ready

When `prestige_rank >= 250,000` and the other gates are met (28 patterns, Ascension X, signal discovered), the overlay changes to show a "Launch" option. The requirements section shows all green checkmarks. A `[Enter] Launch into the Void` prompt appears.

### Phase 6: Launch Confirmation

Pressing Enter on the ready screen shows a confirmation modal:
- The burn, stated plainly: `P253,218  →  P3,218` (before → after)
- Lists what will happen (250,000 PR consumed in one burn, the Loom transforms, the voyage begins)
- "There is no return."
- `[Enter] Confirm  [Esc] Cancel`

On confirm, in one action:

```
state.prestige_rank -= 250_000;
state.recalculate_prestige_bonuses();
state.derived_stats_dirty = true;
state.vessel_launched = true;
```

The actual mode transition to Act 2 is handled by a future sub-project — for now, launch performs the burn, sets the flag, and shows a "Coming soon" message or similar placeholder.

## State Model

New fields on `GameState` (with `#[serde(default)]`):

```rust
/// True after Z50 final boss first kill — enables ticker hints and [V] hotkey
pub vessel_signal_discovered: bool,
/// True after player confirms launch — triggers Act 2 mode shift (future sub-project)
pub vessel_launched: bool,
```

No separate module and no fuel accumulator needed. These two fields on `GameState` are sufficient for sub-project 1 — the fuel gate reads `prestige_rank` directly.

## Launch Burn Logic

A single gate check and a single deduction in the launch confirmation handler — no changes to any PR grant site:

```
fn can_launch(state: &GameState, loom: &LoomState) -> bool {
    state.vessel_signal_discovered
        && !state.vessel_launched
        && state.ascension_level >= 10
        && crate::loom::all_patterns_complete(&loom.persistent)
        && state.prestige_rank >= 250_000
}
```

(`vessel_signal_discovered` already implies Z50 was cleared — no separate `z50_cleared` flag is needed.)

The deduction happens exactly once, at confirmation (see Phase 6). This is the simplest possible model: PR grants remain untouched at all five sites (prestige action, WR→PR, Power Cores online/offline, challenge rewards), achievement passive-PR tracking (#612) keeps working unmodified, there is no partial state to persist, and the hero fights at full prestige bonuses for the entire wait.

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
Row 10-13: Four requirement lines with ✓/✗ (PR line shows rank / 250,000)
Row 14-15: Rank progress bar toward 250,000
Row 16:    Generation rate + days-until-ready estimate
Row 17:    Footer ([Enter] Launch / [Esc] Close)
```

## Files Changed

| File | Change |
|------|--------|
| `src/core/game_state.rs` | Add `vessel_signal_discovered`, `vessel_launched` fields |
| `src/core/tick_types.rs` | Add `VesselSignalDiscovered` tick event variant |
| `src/core/tick_stages.rs` | Detect Z50 boss kill, emit event |
| `src/tick_events.rs` | Handle `VesselSignalDiscovered` flag, add ticker messages |
| `src/input/mod.rs` | Add `[V]` hotkey, vessel overlay dispatch |
| `src/input/types.rs` | Add `GameOverlay::VesselDiscovery` variant + `VesselUiState` |
| `src/ui/vessel_scene.rs` | New file: render construction overlay |
| `src/ui/mod.rs` | Register vessel_scene module |
| `src/ui/stats_panel.rs` | Show vessel indicator line |
| `src/main.rs` | Wire vessel overlay into render loop |

## Release Staging (kill-switch)

Sub-project 1 can merge to main **dark**: `vessel::ACT2_ENABLED = false` keeps the entire feature invisible until Act 2 is deliberately launched.

| Layer | Behavior while disabled |
|-------|------------------------|
| Z50 detection | **Still records** `vessel_signal_discovered` in saves (silently) — qualified players light up the instant Act 2 is enabled, no re-kill needed |
| Discovery modal, log line, ticker entry | Suppressed (`src/tick_events.rs`) |
| Ticker whispers | Suppressed (stage 12c gate in `src/core/tick.rs`) |
| Stats panel row | Hidden (`src/ui/stats_panel.rs`) |
| `[V]` hotkey | Inert (`src/input/mod.rs`) — with no overlay, the launch burn is unreachable |

**To launch Act 2:** flip `ACT2_ENABLED` to `true` in `src/vessel/mod.rs` and update the `act2_kill_switch_is_off_for_release` release-guard test in the same file (deliberately a two-line change so the switch can't flip by accident). The push-to-main release pipeline ships it like any other change.

**To preview on any build** (dev, beta testers, drive-game screenshots): run with `QUEST_ACT2=1` in the environment — the runtime check `vessel::act2_enabled()` honors the override without recompiling.

## Testing

- Unit test: `can_launch` requires all four gates (signal, Ascension X, 28 patterns, 250,000 PR)
- Unit test: launch deducts exactly 250,000, recalculates prestige bonuses, sets `vessel_launched`
- Unit test: launch refused below 250,000 PR and after already launched
- Unit test: PR grants (WR→PR, Power Cores, challenges) are untouched during the wait — rank keeps rising
- Unit test: `vessel_signal_discovered` set on Z50 boss kill
- Unit test: serde round-trip for new GameState fields (backwards compat)
- Add save-format compatibility fixtures for the new fields (fixture corpus from #626)
- Snapshot test: construction overlay scene (full-frame TUI snapshot infra from #623/#624)
