> Backported design record. Sources: docs/plans/2026-03-03-unified-status-strip-design.md.

## 2026-03-03-unified-status-strip-design.md

# Unified Status Strip Design

## Problem

The floating combat text system was designed when HP bars sat adjacent to the enemy sprite. Now that HP bars live in the 2-row status strip at the bottom of the right panel, the floating text over the sprite area is spatially disconnected from the health changes it represents.

Additionally, the combat and dungeon status strips use completely different rendering approaches:
- **Combat strip** (`draw_status_strip_combat`): text spans + `text_hp_bar()`, shows timers/DPS, no damage flash
- **Dungeon strip** (`draw_status_strip_dungeon`): `Gauge` widget via `render_hp_bar_with_flash()`, shows room/key info, has damage flash but no timers

## Solution

Replace both with a single reusable `StatusRow` renderer. Remove the floating damage text from the XL/L sprite area. All combat feedback appears inline in the status strip as right-aligned damage flash numbers.

### StatusRow Component

Each row renders: `Label:current/max [████░░░░] | segment | segment ... flash`

```
HP:340/500 ████░░░░ | You:0.8s | DPS:42           -45
Goblin:120/200 ███░░░ | Foe:1.2s                  -82
```

The component accepts:
- **label**: "HP" or enemy name (truncated to fit)
- **current / max**: HP values (smart-abbreviated when space is tight)
- **bar_color**: Color for the text bar fill
- **segments**: Variable list of right-side info spans (timer, DPS, room info)
- **flash**: Optional `DamageFlash` — rendered right-aligned at row end

### Smart Number Abbreviation

Numbers are abbreviated only when the row width is tight:
- Under 1,000: always full (`340`, `500`)
- 1,000+: use `K` when space is tight (`12.4K/25K`)
- 1,000,000+: use `M` (`1.2M`)

When the row is wide enough, full numbers with commas are shown (`12,450/25,000`).

### Context Variants

**Combat — fighting:**
```
HP:340/500 ████░░░░ | You:0.8s | DPS:42           -45
Goblin:120/200 ███░░░ | Foe:1.2s                  -82
```

**Combat — boss with enrage:**
```
HP:12.4K/25K ████░░░░ | You:0.8s | DPS:42       -1.2K
Fire Drake:8.2K/15K ███░░ | ⚡Enrage:25s         -3.4K
```

**Combat — regenerating (no enemy):**
```
HP:340/500 ████████ | DPS:42                       +50
⏳ Searching for enemies...
```

**Dungeon — fighting:**
```
HP:340/500 ████░░░░ | You:0.8s | Rm 3/8 🔑        -45
Boss Lich:800/1.2K ███░░ | Foe:1.4s              -120
```

**Dungeon — exploring (no enemy):**
```
HP:340/500 ████████ | Rm 3/8 🔑
⏳ Exploring the dungeon...
```

### Changes

1. **New**: `draw_status_row()` function — reusable renderer for one HP row with segments and flash
2. **New**: `format_hp_value()` helper — smart abbreviation (K/M/B) based on available width
3. **Rewrite**: `draw_status_strip_combat()` — calls `draw_status_row()` twice
4. **Rewrite**: `draw_status_strip_dungeon()` — calls `draw_status_row()` twice
5. **Remove**: `draw_floating_damage()` call from `draw_combat_full()` (XL/L sprite area)
6. **Remove**: `render_hp_bar_with_flash()` — replaced by `draw_status_row()`
7. **Keep**: S-tier `draw_s_player_hp()` / `draw_s_enemy_hp()` (they use `render_hp_bar_with_flash` which becomes `draw_status_row`)
8. **Keep**: M-tier compact combat scene (has its own HP bars above/below sprite)

### Files Affected

- `src/ui/mod.rs` — new `draw_status_row()`, `format_hp_value()`, rewrite combat/dungeon strips, remove `render_hp_bar_with_flash()`
- `src/ui/combat_scene.rs` — remove `draw_floating_damage()` call from `draw_combat_full()`
