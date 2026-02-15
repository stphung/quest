# Enhancement System (Blacksmith) Design

> **Issue:** #94 — Upgrade System

## Goal

Add an equipment enhancement system (+1 to +10) accessed via a one-time Blacksmith discovery event. Uses prestige ranks as currency with escalating risk/reward at higher levels. Features dramatic anvil hammering animations for upgrades, golden sparkle bursts for success, and red shake/crack effects for failure.

## Architecture

Standalone `src/enhancement/` module with account-wide persistence (`~/.quest/enhancement.json`). Follows the Haven pattern: one-time tick-based discovery, permanent hotkey access, separate persistence file.

## Data Model

```rust
pub struct EnhancementProgress {
    pub discovered: bool,
    pub levels: [u8; 7],           // Per-slot, 0-10, indexed by EquipmentSlot
    pub total_attempts: u32,
    pub total_successes: u32,
    pub total_failures: u32,
    pub highest_level_reached: u8,
}
```

Persisted to `~/.quest/enhancement.json`, loaded at startup alongside achievements and haven.

## Enhancement Table

| Level | Success Rate | Fail Penalty | Cost (PR) | Cumulative Cost |
|-------|-------------|--------------|-----------|-----------------|
| +1    | 100%        | —            | 1         | 1               |
| +2    | 100%        | —            | 1         | 2               |
| +3    | 100%        | —            | 1         | 3               |
| +4    | 100%        | —            | 1         | 4               |
| +5    | 70%         | -1           | 3         | 7               |
| +6    | 60%         | -1           | 3         | 10              |
| +7    | 50%         | -1           | 3         | 13              |
| +8    | 30%         | -2           | 5         | 18              |
| +9    | 15%         | -2           | 5         | 23              |
| +10   | 5%          | -2           | 10        | 33              |

**All 7 slots to +10 (no fails): 231 PR. Realistically with failures: 400-800+ PR.**

## Stat Bonuses (Cumulative)

| Level | Bonus | Cumulative |
|-------|-------|------------|
| +1    | +1%   | +1%        |
| +2    | +1%   | +2%        |
| +3    | +2%   | +4%        |
| +4    | +2%   | +6%        |
| +5    | +3%   | +9%        |
| +6    | +4%   | +13%       |
| +7    | +5%   | +18%       |
| +8    | +7%   | +25%       |
| +9    | +10%  | +35%       |
| +10   | +15%  | +50%       |

Bonus is a % multiplier on the item's attribute and affix contributions for that slot.

## Discovery & Access

- **Requirement:** P15+
- **Chance per tick:** `0.000014` base + `0.000007` per rank above 15 (~2hr avg at P15)
- **Conditions:** No active dungeon, fishing, or minigame
- **One-time:** Once discovered, permanently accessible via `[B]` hotkey
- **Discovery modal:** Centered overlay announcing the Blacksmith
- **Debug menu:** "Trigger Blacksmith Discovery" option

## Blacksmith UI

Full-screen overlay accessed via `[B]`:

```
┌─ Blacksmith ─────────────────────────────────────────────┐
│                                                          │
│  ⚒ THE BLACKSMITH                     Prestige: 47 PR   │
│  ─────────────────────────────────────────────────────   │
│                                                          │
│  Select equipment to enhance:                            │
│                                                          │
│  ▶ ⚔ Weapon    +4 → +5   (70%)    3 PR                  │
│    🛡 Armor     +3 → +4   (100%)   1 PR                  │
│    🪖 Helmet    +0 → +1   (100%)   1 PR                  │
│    🧤 Gloves    +2 → +3   (100%)   1 PR                  │
│    👢 Boots     +0 → +1   (100%)   1 PR                  │
│    📿 Amulet    +8 → +9   (15%)    5 PR                  │
│    💍 Ring      +1 → +2   (100%)   1 PR                  │
│                                                          │
│  Lifetime: 42 attempts, 31 successes, 11 failures        │
│                                                          │
│  [↑/↓] Navigate  [Enter] Enhance  [Esc] Close           │
└──────────────────────────────────────────────────────────┘
```

- +10 slots show `MAX`
- Insufficient PR: cost in red, Enter blocked
- Item name shown in rarity color if space allows

## Enhancement Flow

1. Select slot, press Enter
2. Confirmation: "Enhance Weapon +4 → +5? (70% success, 3 PR)" — Enter/Esc
3. PR deducted immediately
4. Anvil hammering animation plays (~2.5s)
5. Result screen (success or failure)
6. Any key returns to menu

## Animations

### Anvil Hammering (~2.5s, 25 ticks)

ASCII anvil with item displayed. Three hammer strikes with escalating sparks:

- Ticks 0-6: Hammer rises
- Ticks 7-8: STRIKE 1 — sparks fly (✦ ✧ * in Yellow/Orange)
- Ticks 9-14: Hammer rises
- Ticks 15-16: STRIKE 2 — more sparks, item glows brighter
- Ticks 17-22: Hammer rises slower (tension)
- Ticks 23-24: STRIKE 3 — big spark burst, brief screen flash
- Tick 25: Pause, then result

### Success Result (~2s)

- Golden sparkle burst: particles radiate outward over ~10 ticks
- "SUCCESS!" pulses between Yellow and gold using sin(tick)
- Item name in Green + BOLD
- Sparkle border (✦ ✧ *) shimmers around result
- Brief full-border flash to Yellow on first frame

### Failure Result (~1.5s)

- Red flash: border turns Red on first frame
- "FAILED!" in Red + BOLD, shakes ±1 char for ~5 ticks
- Crack effect: ╳ characters around item
- Level change: old level in DarkGray, new in Red
- Settles to static after shake

### Implementation

```rust
pub enum BlacksmithPhase {
    Menu,
    Confirming { slot: usize },
    Hammering { tick: u8, slot: usize },
    ResultSuccess { tick: u8, slot: usize, old_level: u8, new_level: u8 },
    ResultFailure { tick: u8, slot: usize, old_level: u8, new_level: u8 },
}
```

Tick counter advances each game tick (100ms). UI reads phase + tick to render.

## Combat Integration

Enhancement bonus is a % multiplier on item attribute and affix contributions per slot. Applied in `calculate_derived_stats()`:

```
(base_item_attrs × enhancement_multiplier) + haven_% + prestige_flat
```

- Scales with item quality (better gear benefits more)
- No new stat types
- Clean pipeline order, no double-dipping

## Display

- Item names gain enhancement prefix: `+5 Fine Sword` (rarity colored)
- +5 and above: prefix in Yellow
- +10: prefix in gold (Rgb(255,215,0)) + BOLD
- Stats panel shows enhanced values directly

## Achievements

| Achievement          | Condition                    | Category    |
|---------------------|------------------------------|-------------|
| Apprentice Smith    | Reach +1 on any slot         | Progression |
| Journeyman Smith    | Reach +5 on any slot         | Progression |
| Master Smith        | Reach +10 on any slot        | Progression |
| Fully Enhanced      | Reach +10 on all 7 slots     | Progression |
| Persistent Hammering| Attempt 100 enhancements     | Progression |

## Stats Tab Integration

Left column — new ENHANCEMENT section:
```
── ENHANCEMENT ──────────────
Attempts ................ 247
Successes ............... 158
Failures ................ 89
Highest Level ........... +8
```

Right column — per-slot level grid:
```
── ENHANCEMENT ──────────────
Weapon  +7  Armor   +5
Helmet  +3  Gloves  +4
Boots   +2  Amulet  +8
Ring    +1
```

Level colors: +0 DarkGray, +1-4 White, +5-7 Yellow, +8-9 Magenta, +10 gold.

## Module Structure

```
src/enhancement/
├── mod.rs           # Public API re-exports
├── types.rs         # EnhancementProgress, constants, success rates
├── logic.rs         # Discovery, enhancement roll, bonus calculation
└── persistence.rs   # JSON save/load from ~/.quest/enhancement.json

src/ui/
└── blacksmith_scene.rs  # Blacksmith menu, animations, result screens
```

## Prestige Economy Context

| System      | Prestige Cost        |
|-------------|---------------------|
| Haven rooms | 1-25 PR (~165 total) |
| Enhancement | 1-10 PR per attempt (~231-800+ total) |
| Storm Forge | 25 PR               |
