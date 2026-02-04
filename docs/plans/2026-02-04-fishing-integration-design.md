# Fishing Integration Design

## Problem

The fishing system is mostly standalone. Items found while fishing are silently lost when sessions end, fishing rank only affects fish rarity distribution, and there's no reason for players to care about fishing beyond the XP it provides.

## Design

Three independent changes that tie fishing back into the core game loop.

### 1. Fix Lost Fishing Items

When a fishing session completes, items in `items_found` are discarded. Fix: capture `items_found` before the session is dropped and run each item through the existing auto-equip scoring system (same as combat item drops). No special treatment for fishing items — identical behavior to monster loot.

**Files:** `main.rs`, `fishing_logic.rs`

### 2. Fishing Rank Milestone Combat Bonuses

Permanent passive combat bonuses at each tier boundary, calculated in `derived_stats.rs` from `state.fishing.rank`:

| Rank | Tier | Bonus |
|------|------|-------|
| 5 | Novice | +2% crit chance |
| 10 | Apprentice | +5% HP regen speed |
| 15 | Journeyman | +3% damage |
| 20 | Expert | +5% defense |
| 25 | Master | +5% XP multiplier |
| 30 | Grandmaster | +10% all damage |

Bonuses are cumulative. A rank 30 fisher has all six. Bonuses are baked directly into `derived_stats.rs` calculations (like equipment bonuses) so the Derived Stats numbers already reflect them. The Prestige section displays which mastery bonuses are active as a compact line:

```
🎣 Fishing: Kraken Caller (25)
🐟 +2%Crit +5%Regen +3%DMG +5%Def +5%XP
████████░░░░░ 1200/1500
```

The mastery line only appears at rank 5+. If bonuses fit on one line, use one line; wrap to a second if needed. The Prestige section height adjusts dynamically.

**Files:** `derived_stats.rs`, `fishing.rs`, `ui/stats_panel.rs`

### 3. Well-Fed Buff After Fishing Sessions

When a fishing session completes, the player receives a "Well-Fed" buff granting bonus XP% on combat kills.

**Strength (from fishing rank):**
- +10% XP at rank 1, scaling linearly to +50% XP at rank 30
- Formula: `10 + (rank - 1) * 40 / 29`

**Duration (from fish caught in session):**
- 30 seconds per fish caught in the completed session
- 3-fish session = 1.5 min, 8-fish session = 4 min
- Stored as tick countdown (300 ticks per fish at 100ms/tick)

**Behavior:**
- New sessions overwrite any existing buff (no stacking)
- Buff is fully transient: `#[serde(skip)]` fields on `GameState`, default to 0
- Not serialized, not applied during offline progression — reward for active play only
- Timer ticks down continuously (during combat, dungeons, idle)

**UI:**
- When Well-Fed is active, the XP Mult line in Derived Stats is replaced:
  `📈 XP Mult: 1.50x (+25% Well-Fed 2m30s)`
  When inactive, shows normally: `📈 XP Mult: 1.50x`
- Combat kill XP messages include bonus: "+300 XP (+25% Well-Fed)"

**Files:** `game_state.rs`, `main.rs`, `combat_logic.rs`, `ui/stats_panel.rs`

## Pacing Analysis

- Fishing spots discovered ~every 8-9 minutes (5% chance per kill, ~5s per kill)
- Average session: 5.5 fish, ~40s duration
- Average buff: ~2.75 min duration
- Buff active ~30% of play time — noticeable but not permanent

## Not Included

- No fishing-specific items or affixes
- No zone gating behind fishing rank
- No fishing during dungeons
- No buff stacking across sessions
- No serialization of the buff
- No offline interaction with the buff
