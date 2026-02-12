# UX Design: Decoupled Attack Timers

## Context

Currently, combat uses a single shared `attack_timer`. When it fires, the player attacks first, then the enemy attacks back in the same tick. This creates a rigid, turn-based feel where attacks are always paired.

The proposed change introduces independent attack timers for the player and the enemy, allowing them to attack at different rates. This document provides UX recommendations for how to communicate this change to the player through the terminal UI.

## Current Combat UI Layout

```
+-- Combat ----------------------------------------+
| Player HP: 45/50       [==============--------]  |  <- 1 line
|                                                   |
|                    /\_/\                          |
|                   ( o.o )                         |  <- sprite area
|                    > ^ <                          |
|                   Meadow Beetle                   |
|                                                   |
| Meadow Beetle: 30/40  [===========---------]     |  <- 1 line
| * In Combat | Next: 0.8s | DPS: 12                |  <- 1 line status
+---------------------------------------------------+
```

The bottom info panel shows a combat log (right half) with color-coded entries:
- Green: player attacks
- Yellow+bold: player crits
- Red: enemy attacks

## Recommendation 1: Dual Timer Display in the Status Bar

**Change**: Replace the single "Next: 0.8s" countdown with two compact countdowns showing both timers.

**Current status line (in combat)**:
```
* In Combat | Next: 0.8s | DPS: 12
```

**Proposed status line (in combat)**:
```
* In Combat | You: 0.4s  Foe: 1.1s | DPS: 12
```

**Rationale**: The status bar is already 1 line and already shows a single timer. Splitting it into two labeled countdowns is minimal visual change, easy to scan, and directly communicates the core mechanical difference. The player immediately sees that the two timers count down independently.

**Color coding**:
- "You: 0.4s" in green (matches player attack log color)
- "Foe: 1.1s" in red (matches enemy attack log color)
- The color reinforces which timer belongs to whom without needing extra labels

**When an attack is imminent** (under 0.3s remaining), the number could flash with the BOLD modifier to create a subtle anticipation cue.

## Recommendation 2: Do NOT Add Timer Progress Bars

**Recommendation**: Do not add gauge/progress bars for attack timers next to the HP bars.

**Rationale**: Adding progress bars would consume vertical space in an already constrained layout (the combat area has `Min(5)` for the sprite, plus 1 line each for player HP, enemy HP, and status). Progress bars would either shrink the sprite area or require expanding the combat panel. The status bar countdown is sufficient -- this is an idle game where precise timer tracking is informational, not interactive. Players do not need to time actions against the timer.

## Recommendation 3: Visual Effects Need No Timing Changes

**Current behavior**: `DamageNumber`, `AttackFlash`, and `HitImpact` effects are lifetime-based (created on the event, fade over their `max_lifetime`). They are independent of the attack timer.

**Recommendation**: Keep the visual effect system as-is. Since effects are already event-driven (spawned when a `CombatEvent` fires), they will naturally work with decoupled timers. When the player attacks, player effects fire. When the enemy attacks on its own timer, enemy effects fire. No overlapping issue arises because effects are rendered per-frame based on their remaining lifetime.

**One refinement**: Consider differentiating the `AttackFlash` effect by source. Currently it uses yellow swords (`"*".repeat(20)`). To help the player visually distinguish simultaneous attacks (which become possible with decoupled timers):
- Player attack flash: keep yellow swords
- Enemy attack flash: use red exclamation marks or a red-tinted variant

This is a minor enhancement. If both attacks happen to land on the same tick, the player will see both flash effects and both log entries, which is acceptable.

## Recommendation 4: Combat Log Entries -- Add Timestamp Context

**Current behavior**: Log entries are color-coded (green = player, red = enemy) with a message string. When attacks were paired, the log naturally read as alternating exchanges.

**With decoupled timers**: Attacks may arrive at different rates, so the log will no longer alternate predictably. For example, a fast-attacking player might show three green entries before a single red entry.

**Recommendation**: No structural change needed. The existing color coding (green vs. red) already differentiates the source clearly. The log scrolls newest-first, so the player can see the natural rhythm of attacks.

**Optional enhancement**: When consecutive entries are from the same source (e.g., three player attacks in a row), a subtle visual separator or grouping is NOT recommended -- it would add visual noise. The color alone is sufficient.

## Recommendation 5: HP Bars Remain Unchanged

**Recommendation**: Keep HP bars exactly as they are. HP bars show current state, not attack timing. They will naturally reflect the new attack cadence as HP changes occur at different rates.

## Recommendation 6: 3D Dungeon View -- No Changes Needed

**Current behavior**: The 3D view (`combat_3d.rs`) renders the enemy sprite centered in the area, or a waiting message when no enemy is present. It does not display timer information.

**Recommendation**: No changes needed. The 3D view is purely a sprite renderer. Timer information is handled by the status bar below it, which applies identically in both dungeon and overworld combat.

## Recommendation 7: DPS Display Adjustment

**Current behavior**: DPS is calculated as `total_damage / ATTACK_INTERVAL_SECONDS`, adjusted for crit. It displays as a single number.

**With decoupled timers**: The player's attack interval may differ from the constant `ATTACK_INTERVAL_SECONDS` (equipment with attack speed affixes already modify this via `attack_speed_multiplier`). The DPS calculation already accounts for this.

**Recommendation**: Keep the single DPS number. It already reflects the player's effective attack rate. If desired, an "Enemy DPS" could be shown, but this adds clutter and is not actionable information in an idle game. The player's HP bar decreasing rate already communicates incoming damage visually.

## Recommendation 8: Regeneration Phase -- No Changes

**Current behavior**: After killing an enemy, the player enters a regen phase (2.5s base) where HP gradually restores. Both timers are irrelevant during this phase.

**Recommendation**: No changes. The status bar already shows "Regenerating..." during this phase. Neither attack timer is relevant, so the dual timer display simply does not render during regen.

## Summary of Changes

| UI Element | Change Required | Details |
|---|---|---|
| Status bar timer | Yes | Split "Next: 0.8s" into "You: 0.4s  Foe: 1.1s" with color coding |
| Attack flash effects | Optional | Differentiate player vs enemy flash color |
| HP bars | No | Already reflect state changes naturally |
| Combat log | No | Color coding already distinguishes source |
| 3D dungeon view | No | Does not display timer info |
| DPS display | No | Already accounts for player attack speed |
| Regen phase | No | Timers not shown during regen |
| Timer progress bars | No (explicitly rejected) | Would consume too much vertical space |

## Implementation Notes for Developers

1. **Status bar**: Modify `draw_combat_status()` in `combat_scene.rs` to read two timer values (e.g., `player_attack_timer` and `enemy_attack_timer`) instead of the single `attack_timer`. Format them side-by-side with colored spans.

2. **Bold flash on imminent attack**: When a timer value is under 0.3s, add `Modifier::BOLD` to its span style to create a subtle urgency cue.

3. **Attack flash differentiation**: In `combat_effects.rs`, the `AttackFlash` variant could carry a `is_player: bool` field. The render method would choose yellow for player, red for enemy.

4. **CombatState changes**: The single `attack_timer: f64` field will be replaced by `player_attack_timer: f64` and `enemy_attack_timer: f64`. The status bar reads these directly.

5. **Backward compatibility**: Use `#[serde(default)]` on new timer fields (as noted in the existing `CombatState` doc comment) so old save files load correctly with timers initialized to 0.0.
