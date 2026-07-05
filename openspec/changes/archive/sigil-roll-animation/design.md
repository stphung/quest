> Backported design record. Sources: docs/plans/2026-02-21-sigil-roll-animation-design.md.

## 2026-02-21-sigil-roll-animation-design.md

# Sigil Roll Animation Design

## Overview

Add a rune inscription ritual animation when rolling for three sigils in the Stormglass exchange. The animation plays after the player confirms an inscribe or reroll, showing energy gathering, character-by-character sigil etching, and grade reveals before transitioning to the existing pick screen.

## Design Decisions

- **Feel**: Rune inscription ritual — mystical energy converges, sigils etch into existence
- **Duration**: ~4 seconds full animation, skippable with any key after 200ms minimum floor
- **Grade reveal**: Grades shown during the pick phase (revealed at end of animation)
- **Architecture**: Cosmetic-only animation; sigil choices pre-generated before animation starts

## Animation Phases

### Phase 1: Energy Gathering (0–1400ms)

Storm particles intensify across the panel. A rune circle builds in the center using progressive unicode characters (`·` → `◦` → `○` → `◎`). Orbiting glyphs (`⚡` `✦` `◈`) converge toward the center. Storm gradient background intensifies over the phase.

### Phase 2: Sigil Etching (1400–2800ms)

Three sigil names etch character-by-character from left to right, staggered ~150ms apart. Each name has a brief inscription flash (warm white highlight) on completion. The storm backdrop holds steady during etching.

### Phase 3: Grade Reveal (2800–3500ms)

Grade labels fade in below each sigil name. Grade-specific color effects:
- S+/S grades: gold glow
- A/B grades: white
- C and below: dim/muted

### Phase 4: Transition to Pick (3500–4000ms)

Sigils solidify (colors stabilize). Selection cursor `►` appears on the first choice. Animation exits into the existing `PickSigil` phase.

## Skip Behavior

- Any keypress after 200ms minimum display floor jumps directly to `PickSigil` phase
- The 200ms floor prevents accidental instant-skip from key repeat
- All keys (Enter, Space, arrows, letters) trigger skip after the floor

## State Machine Changes

```
ExchangePhase flow:
  ConfirmInscribe ──[Enter]──► SigilRolling ──[timeout/skip]──► PickSigil
  ConfirmReroll   ──[Enter]──► SigilRolling ──[timeout/skip]──► PickSigil
```

### New ExchangePhase Variant

Add `SigilRolling` to the `ExchangePhase` enum in `src/stormglass/types.rs`.

### New Fields on ExchangeUiState

- `sigil_animation_start_ms: Option<u128>` — set to `current_millis()` when entering `SigilRolling`
- `sigil_animation_skipped: bool` — set `true` on any keypress after 200ms floor

## Rendering Approach

Uses existing `scene_fx.rs` infrastructure (`SceneCell` buffer, `current_millis()`, `lerp_rgb()`). Renders within the Stormglass panel area (right half of screen).

| Phase | Background | Center Content | Effects |
|-------|-----------|---------------|---------|
| Energy Gathering | Storm gradient intensifies | Rune circle builds | Orbiting particles |
| Sigil Etching | Storm holds | Sigil names etch L→R | Flash on completion |
| Grade Reveal | Storm holds | Names + grade labels | Grade-specific colors |
| Transition | Storm calms | All three solidified | Cursor appears |

### Color Constants

- Storm backdrop: reuse existing zone_bg storm colors
- Rune circle: `(180, 160, 255)` purple
- Inscription flash: `(255, 255, 200)` warm white
- Grade colors: mapped from existing `grade_color()` function

## Edge Cases

1. **SG deduction timing**: SG deducted on confirm, before animation. Quitting during animation = SG spent, no sigil gained. Consistent with forfeit pattern.

2. **Reroll safety**: Old sigil destroyed before animation. More punishing than inscribe forfeit (loses SG + old sigil vs SG only). Acceptable since player already confirmed.

3. **Terminal resize**: Scene buffer recreated at new size on next frame. No crash risk.

4. **`sigil_choices` protection**: Choices generated and stored before animation starts. Animation reads but never overwrites them.

5. **No persistence**: Animation state is ephemeral. Crash during animation = SG spent, no sigil (same as forfeit).

## Files Changed

| File | Changes | ~LOC |
|------|---------|------|
| `src/stormglass/types.rs` | Add `SigilRolling` variant, 2 new fields | ~10 |
| `src/input/stormglass_input.rs` | Skip handling, phase transitions | ~30 |
| `src/ui/stormglass_scene.rs` | Animation rendering (4 sub-phases) | ~240 |
| `src/main.rs` | Animation timeout check | ~20 |

**Total**: ~300 LOC across 4 files. No new files. No new dependencies.
