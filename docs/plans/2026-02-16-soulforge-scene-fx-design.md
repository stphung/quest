# Soulforge Scene FX Effects Design

## Overview

Add scene_fx-powered visual effects to the Soulforge enhancement overlay, converting it from pure Ratatui widgets to a full SceneCell buffer with animated ember/furnace backdrop, spark showers on hammer strikes, and dramatic success/failure result screens.

## Approach

**Full Scene Buffer Replacement (Approach A):** Rewrite `render_soulforge()` to use a `SceneCell` buffer, consistent with `fishing_scene.rs` and `combat_3d.rs`. The entire overlay renders into a buffer, then flushes with `render_buffer()`.

## Effects by Phase

### All Phases: Ember/Furnace Backdrop

- **Background gradient:** Bottom rows warm orange/red `(180, 60, 20)` fading to near-black `(15, 8, 5)` at top. Creates the feel of standing near a forge pit.
- **Drifting embers:** 8-12 particles drift upward using `current_millis()`. Characters: `·`, `•`, `*`, `✦`. Warm colors (orange, yellow, red). Positions seeded with `hash2d()`. Embers dim as they rise (bright orange at bottom, dark red/gone at top).
- **Heat shimmer:** Subtle brightness oscillation on a few backdrop cells using sine-based shifting on the red channel.

### Menu Phase

- Standard warm forge backdrop behind the overlay.
- Equipment slot list, detail panel, stats, help text rendered via `put_cell()` with high-contrast fg colors.
- Selected slot row keeps warm highlight bg `(40, 40, 20)`.
- Flavor text gets subtle warm color pulse (very slow).

### Confirming Phase

- Backdrop intensifies slightly — more embers, brighter base gradient.
- Centered confirmation prompt rendered over backdrop.
- Subtle pulsing glow on the slot name being enhanced.

### Hammering Phase

- **Spark shower on strikes:** When `is_strike` is true (ticks 14-16, 30-32, 46-48), 8-12 spark particles spray outward from anvil contact point.
  - Fan angles: 30-150 degrees upward.
  - Characters: `✦`, `·`, `*`, `✧`.
  - Color lifecycle: bright white/yellow at spawn, orange mid-life, dark red at expiry (~10 tick lifetime).
  - Gravity: sparks drift downward after initial upward burst.
  - Rendered via `put_cell()` on top of backdrop.
- **Anvil glow:** Anvil ASCII characters pulse from DarkGray to warm orange/yellow on strikes (~4 tick fade). Subtle warm glow remains between strikes.
- **Progress bar:** Filled portion pulses with warm gradient (dark orange to bright yellow) synced to forge rhythm.
- **Hammer afterimage:** On first tick of a strike, render both raised and striking positions — raised copy in dimmer color for motion trail effect.

### ResultSuccess — Golden Radiant Burst

- Backdrop shifts instantly from forge orange to brilliant gold.
- ~20 golden embers/sparks explode upward from center, fanning across overlay.
- Continuous upward golden rain effect.
- "SUCCESS!" text in bright gold with warm glow halo on surrounding cells.
- Sparkle characters (`✦`, `✧`, `*`) appear at random positions and twinkle (3-4 tick fade in/out cycle).
- Over ~30 ticks, settles into warm golden ambient glow.

### ResultFailure — Ash and Decay

- Backdrop rapidly cools over ~8 ticks using `lerp_rgb()` from warm to ash-gray `(40, 40, 45)`.
- Drifting embers slow and darken: orange, dark red, then extinguish.
- Crack characters (`╳`) appear at random backdrop positions.
- "FAILED!" text in red with existing shake_offset effect preserved.
- A few dying embers (`·`) drift downward instead of up.
- Backdrop stays cold/dark until user presses key and returns to Menu (which restores warm glow).

## Technical Details

### scene_fx utilities used

- `SceneCell` buffer for the full 62x24 overlay area
- `render_buffer()` to flush to frame
- `put_cell()` for all character placement (text and effects)
- `hash2d()` for deterministic ember placement
- `lerp_rgb()` / `lerp_channel()` for color transitions
- `current_millis()` for animation timing

### State changes

- `SoulforgeUiState` may need additional fields for spark particle state (positions, velocities, lifetimes) if we want sparks to persist across frames. Alternative: derive all particle positions purely from `animation_tick` and `current_millis()` to keep state minimal.

### Readability principle

All text rendered with explicit high-contrast fg colors. Backdrop is dark enough that white/yellow/green/red/cyan text remains clearly readable. Key information (rates, costs, penalties) preserves existing color scheme.

## Files Modified

- `src/ui/soulforge_scene.rs` — Major rewrite to scene_fx buffer rendering
- `src/enhancement/types.rs` — Possible minor additions to `SoulforgeUiState` for particle state

## Files Not Modified

- `src/ui/scene_fx.rs` — Existing utilities are sufficient
- `src/enhancement/logic.rs` — No logic changes
- Game tick, input handling — No changes needed
