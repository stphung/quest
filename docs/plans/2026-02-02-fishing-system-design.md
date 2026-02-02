# Fishing System Design

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a random fishing minigame with persistent ranking that survives prestige, providing long-term progression (~1000 hours to max rank).

**Architecture:** Fishing triggers randomly like dungeons (5% chance in overworld), replaces combat temporarily with automatic fishing sessions. A separate `FishingState` persists across prestige resets.

**Tech Stack:** Rust, Ratatui TUI, Serde serialization

---

## Core Mechanics

| Aspect | Value |
|--------|-------|
| Discovery chance | 5% per tick in overworld |
| Mutual exclusivity | Check dungeon first, then fishing |
| Session length | 3-8 fish (random) |
| Automation | Fully automatic, no player input |
| Combat interaction | Replaces combat temporarily |
| Session end | "Fishing spot depleted" → return to combat |

## Fish Species & Rewards

| Rarity | Base Chance | XP Reward | Item Drop | Example Fish |
|--------|-------------|-----------|-----------|--------------|
| Common | 60% | 50-100 | - | Minnow, Carp, Perch |
| Uncommon | 25% | 150-250 | 5% | Trout, Bass, Catfish |
| Rare | 10% | 400-600 | 15% | Salmon, Pike, Sturgeon |
| Epic | 4% | 1000-1500 | 35% | Marlin, Swordfish, Barracuda |
| Legendary | 1% | 3000-5000 | 75% | Kraken Spawn, Sea Serpent, Leviathan Fry |

**Rank bonuses to rarity (every 5 ranks):**
- -2% Common, +1% Uncommon, +0.5% Rare, +0.3% Epic, +0.2% Legendary
- At Rank 30: Common 48%, Uncommon 31%, Rare 13%, Epic 5.8%, Legendary 2.2%

**Item drops:** Use existing item generation system, scaled to player level, with fishing-specific flavor text.

**XP distribution:**
- Character XP (affected by prestige multipliers)
- Fishing rank XP (separate track, never resets)

## Rank Progression (~1000 hours to max)

With 5% discovery rate and ~3 minute sessions: ~1 fishing session per hour, ~5 fish per session = **5 fish/hour**.

**1000 hours = ~5000 sessions = ~25,000 fish**

### Tier Structure

| Tier | Ranks | Fish to Complete | Cumulative | Hours |
|------|-------|------------------|------------|-------|
| Novice | 1-5 | 500 | 500 | ~100h |
| Apprentice | 6-10 | 1,000 | 1,500 | ~300h |
| Journeyman | 11-15 | 2,000 | 3,500 | ~500h |
| Expert | 16-20 | 4,000 | 7,500 | ~650h |
| Master | 21-25 | 7,500 | 15,000 | ~850h |
| Grandmaster | 26-30 | 10,000 | 25,000 | ~1000h |

### All 30 Rank Names

| Rank | Name | Tier |
|------|------|------|
| 1 | Bait Handler | Novice |
| 2 | Line Tangler | Novice |
| 3 | Nibble Watcher | Novice |
| 4 | Hook Setter | Novice |
| 5 | Line Caster | Novice |
| 6 | Pond Fisher | Apprentice |
| 7 | River Wader | Apprentice |
| 8 | Lake Lounger | Apprentice |
| 9 | Stream Reader | Apprentice |
| 10 | Net Weaver | Apprentice |
| 11 | Tide Reader | Journeyman |
| 12 | Reef Walker | Journeyman |
| 13 | Shell Seeker | Journeyman |
| 14 | Wave Rider | Journeyman |
| 15 | Current Master | Journeyman |
| 16 | Deep Diver | Expert |
| 17 | Trench Explorer | Expert |
| 18 | Abyssal Angler | Expert |
| 19 | Pressure Breaker | Expert |
| 20 | Storm Fisher | Expert |
| 21 | Legend Hunter | Master |
| 22 | Myth Seeker | Master |
| 23 | Leviathan Lurer | Master |
| 24 | Serpent Tamer | Master |
| 25 | Kraken Caller | Master |
| 26 | Ocean Sage | Grandmaster |
| 27 | Tidebinder | Grandmaster |
| 28 | Depthless One | Grandmaster |
| 29 | Sea Eternal | Grandmaster |
| 30 | Poseidon's Chosen | Grandmaster |

### Milestone Bonuses (every 5 ranks)

| Rank | Bonus |
|------|-------|
| 5 | +10% catch quality |
| 10 | Unlock rare fish species |
| 15 | +1 fish per session |
| 20 | Unlock legendary fish |
| 25 | +25% fishing XP |
| 30 | Mythic fish species + title "Poseidon's Chosen" |

## UI Design

### Fishing Session Screen

```
╔═══════════════════════════════════════╗
║  🎣 FISHING - Crystal Lake            ║
╠═══════════════════════════════════════╣
║                                       ║
║     ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~        ║
║       ~~~~~~ O ~~~~~~                 ║
║     ~ ~ ~ ~ ~|~ ~ ~ ~ ~ ~ ~          ║
║              |                        ║
║                                       ║
║  Caught: 3/6 fish                     ║
║                                       ║
║  [Uncommon] Trout - 180 XP           ║
║  [Common] Carp - 65 XP               ║
║  [Rare] Salmon - 520 XP  📦          ║
║                                       ║
║  Rank: Reef Walker (12)               ║
║  Progress: ████████░░ 847/1000        ║
╚═══════════════════════════════════════╝
```

### Main Game Display (Stats Panel)

```
╔═══════════════════════════════════════╗
║  Hero - Level 15                      ║
║  Prestige: Silver IV (2.0x XP)        ║
║  Fishing: Reef Walker (12)            ║
╠═══════════════════════════════════════╣
║  STR: 14 (+2)    HP: 89/120           ║
...
```

### Fishing Spot Names (random selection)

- Crystal Lake
- Misty Pond
- Rushing Creek
- Coral Shallows
- Abyssal Rift
- Moonlit Bay

## Data Structures

### FishingState

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FishingState {
    pub rank: u32,                    // 1-30, default 1
    pub total_fish_caught: u32,       // Lifetime counter
    pub fish_toward_next_rank: u32,   // Progress within current rank
    pub legendary_catches: u32,       // Lifetime legendaries (for stats)
}
```

### FishingSession (active session)

```rust
pub struct FishingSession {
    pub spot_name: String,
    pub total_fish: u32,              // 3-8, determined at start
    pub fish_caught: Vec<CaughtFish>,
    pub items_found: Vec<Item>,
}

pub struct CaughtFish {
    pub name: String,
    pub rarity: FishRarity,
    pub xp_reward: u32,
}

pub enum FishRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}
```

### GameState Integration

```rust
pub struct GameState {
    // ... existing fields ...
    pub fishing: FishingState,              // Persists across prestige
    pub active_fishing: Option<FishingSession>, // Current session if any
}
```

## Persistence

**Prestige behavior:**
```rust
fn perform_prestige(state: &mut GameState) {
    // Reset: level, XP, attributes, combat_state
    // Preserve: equipment, fishing (unchanged)
}
```

**Save compatibility:**
- Add `fishing` field with `#[serde(default)]`
- Old saves load with `FishingState::default()` (Rank 1, 0 fish)

## Implementation Notes

### Discovery Logic (in game_logic.rs)

```rust
// In tick update, after dungeon check:
if state.active_dungeon.is_none() && state.active_fishing.is_none() {
    if rng.gen::<f64>() < 0.05 {
        // 5% chance, start fishing session
        state.active_fishing = Some(FishingSession::new(&mut rng));
    }
}
```

### Session Tick (new fishing_logic.rs)

```rust
pub fn tick_fishing(state: &mut GameState, rng: &mut impl Rng) {
    if let Some(session) = &mut state.active_fishing {
        // Catch fish on interval (similar to combat attack interval)
        // Roll rarity based on rank bonuses
        // Award XP, check for item drop
        // If session.fish_caught.len() >= session.total_fish, end session
    }
}
```

### Rank Calculation

```rust
impl FishingState {
    pub fn rank_name(&self) -> &'static str {
        RANK_NAMES[self.rank as usize - 1]
    }

    pub fn fish_required_for_rank(rank: u32) -> u32 {
        // Exponential curve matching tier structure
    }

    pub fn rarity_bonus(&self) -> f64 {
        // +0.2% legendary per 5 ranks, etc.
    }
}
```

---

## Files to Create/Modify

| File | Action |
|------|--------|
| `src/fishing.rs` | Create - FishingState, FishRarity, rank data |
| `src/fishing_logic.rs` | Create - Session management, tick logic |
| `src/fishing_generation.rs` | Create - Fish spawning, rarity rolls |
| `src/game_state.rs` | Modify - Add fishing field |
| `src/game_logic.rs` | Modify - Discovery trigger |
| `src/prestige.rs` | Modify - Preserve fishing on reset |
| `src/ui/mod.rs` | Modify - Route to fishing UI |
| `src/ui/fishing_scene.rs` | Create - Fishing session display |
| `src/ui/stats_panel.rs` | Modify - Show fishing rank |
| `src/main.rs` | Modify - Handle fishing state |
