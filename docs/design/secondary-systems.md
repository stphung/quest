# Secondary Systems Design

This document describes the fishing, dungeon, haven, and achievement systems as implemented.

## Fishing System

### Overview

A random fishing minigame with persistent ranking that survives prestige. Provides long-term progression (~1000+ hours to max rank). Fishing triggers randomly during overworld combat — fully automatic with no player input.

### Discovery and Sessions

- **Discovery chance**: 5% per tick during overworld combat (not in dungeon/minigame)
- **Mutual exclusivity**: Dungeon checked first, then fishing
- **Session length**: 3-8 fish (random)
- **Automation**: Fully automatic, no player input required
- **Session end**: "Fishing spot depleted" — returns to combat

### Fishing Phases (Per Fish)

Three-phase state machine per fish catch:
1. **Casting** — 0.5-1.5 seconds
2. **Waiting** — 1.0-8.0 seconds (random)
3. **Reeling** — 0.5-3.0 seconds

Average time per fish: ~5 seconds.

### Fish Rarities

| Rarity | Base Chance | XP Reward | Item Drop % |
|--------|-------------|-----------|-------------|
| Common | 60% | 50-100 | 5% |
| Uncommon | 25% | 150-250 | 5% |
| Rare | 10% | 400-600 | 15% |
| Epic | 4% | 1,000-1,500 | 35% |
| Legendary | 1% | 3,000-5,000 | 75% |

**Rank bonus per 5 ranks** (bonus_tiers = rank / 5):
- Common: -2% per tier (floor: 10%)
- Uncommon: +1% per tier
- Rare: +0.5% per tier
- Epic: +0.3% per tier
- Legendary: +0.2% per tier

### Fishing Spot Names (8)

Crystal Lake, Misty Pond, Rushing Creek, Coral Shallows, Abyssal Rift, Moonlit Bay, Serpent Cove, Whispering Falls.

### Fish Names (5 per rarity)

- **Common**: Minnow, Carp, Perch, Bluegill, Sunfish
- **Uncommon**: Trout, Bass, Catfish, Walleye, Crappie
- **Rare**: Salmon, Pike, Sturgeon, Muskie, Steelhead
- **Epic**: Marlin, Swordfish, Barracuda, Tuna, Mahi-mahi
- **Legendary**: Kraken Spawn, Sea Serpent, Leviathan Fry, Abyssal Eel, Phantom Whale

### Rank Progression — 40 Ranks, 8 Tiers

Base cap is Rank 30. Ranks 31-40 require the FishingDock Haven room at T4 (+10 max rank).

| Tier | Ranks | Fish/Rank | Cumulative Fish |
|------|-------|-----------|-----------------|
| Novice | 1-5 | 100 | 500 |
| Apprentice | 6-10 | 200 | 1,500 |
| Journeyman | 11-15 | 400 | 3,500 |
| Expert | 16-20 | 800 | 7,500 |
| Master | 21-25 | 1,500 | 15,000 |
| Grandmaster | 26-30 | 2,000 | 25,000 |
| Mythic | 31-35 | 4k-25k (escalating) | 86,000 |
| Transcendent | 36-40 | 40k-250k (escalating) | 701,000 |

### Rank Names (All 40)

| Rank | Name | Tier |
|------|------|------|
| 1-5 | Bait Handler, Line Tangler, Nibble Watcher, Hook Setter, Line Caster | Novice |
| 6-10 | Pond Fisher, River Wader, Lake Lounger, Stream Reader, Net Weaver | Apprentice |
| 11-15 | Tide Reader, Reef Walker, Shell Seeker, Wave Rider, Current Master | Journeyman |
| 16-20 | Deep Diver, Trench Explorer, Abyssal Angler, Pressure Breaker, Storm Fisher | Expert |
| 21-25 | Legend Hunter, Myth Seeker, Leviathan Lurer, Serpent Tamer, Kraken Caller | Master |
| 26-30 | Ocean Sage, Tidebinder, Depthless One, Sea Eternal, Poseidon's Chosen | Grandmaster |
| 31-35 | Titan's Bane, World Fisher, Primordial Angler, Abyss Walker, Fate Weaver | Mythic |
| 36-40 | Void Fisher, Eternal Caster, Reality Bender, Cosmos Reeler, The Unchained | Transcendent |

### Storm Leviathan (Endgame Fishing Quest)

A special legendary fish required to forge the Stormbreaker weapon:

- **Requirement**: Fishing Rank 40+ (requires FishingDock T4)
- **Appearance**: Only when catching legendary fish at Rank 40+
- **10 encounters required** before it can be caught, with decreasing encounter rates: 8%, 6%, 5%, 4%, 3%, 2%, 1.5%, 1%, 0.5%, 0.25%
- **Catch chance**: 25% per legendary fish after 10 encounters
- **XP reward**: 10,000-15,000
- **Unlocks**: `StormLeviathan` achievement (required for Stormbreaker forging)

### Persistence

Fishing state (rank, total fish caught, legendary catches) persists across prestige resets. Only the active fishing session is cleared on prestige.

## Dungeon System

### Overview

Procedurally generated dungeon exploration triggered randomly after kills (2% chance per kill). Dungeons feature connected rooms with various types, a key system for locked doors, and safe death (no prestige loss).

### Dungeon Sizes

| Size | Grid | Based On |
|------|------|----------|
| Small | 5x5 | Low prestige |
| Medium | 7x7 | Mid prestige |
| Large | 9x9 | High prestige |
| Epic | 11x11 | Very high prestige |

### Room Types

- **Entrance**: Starting room, always revealed
- **Combat**: Enemy encounters
- **Treasure**: Item drops and XP rewards
- **Elite**: Stronger enemies, better rewards
- **Boss**: Dungeon boss, must defeat to complete

### Room States

- **Hidden**: Not yet discovered (fog of war)
- **Revealed**: Visible on map but not visited
- **Current**: Player's current location
- **Cleared**: Already completed

### Navigation and Keys

Rooms are connected via procedural generation ensuring all rooms are reachable. Movement uses arrow keys on the dungeon map. Some doors are locked and require keys found in treasure rooms.

### Death

Death in a dungeon exits the dungeon with no prestige loss — a safe environment for exploration.

## Haven System

### Overview

Account-level base building presented as a skill tree. Players spend prestige ranks to construct and upgrade rooms that provide permanent passive bonuses. The Haven persists across all prestige resets and benefits every character on the account. Stored in `~/.quest/haven.json`.

### Unlock Conditions

- **Prestige gate**: Character must be P10+ (Celestial tier)
- **Discovery**: Independent RNG roll per tick, chance scales with prestige rank:
  - `chance = 0.000014 + (prestige_rank - 10) * 0.000007`
  - P10: ~2 hours average, P15: ~34 min, P20: ~20 min
- **One-time**: Once discovered, accessible account-wide permanently via `[H]` key

### Currency

Players spend actual prestige ranks from the contributing character. Ranks decrease when spent.

Costs vary by room depth in the tree:

| Depth | T1 Cost | T2 Cost | T3 Cost |
|-------|---------|---------|---------|
| 0 (Hearthstone) | 1 | 2 | 3 |
| 1 (Armory, Bedroom) | 1 | 3 | 5 |
| 2-3 (mid-tree) | 2 | 4 | 6 |
| 4 (capstones) | 3 | 5 | 7 |
| FishingDock (special) | 2 | 4 | 6 | T4: 10 |
| StormForge | 25 (T1 only) | — | — |

### Skill Tree Structure (14 Rooms)

```
                     [Hearthstone]
                    /              \
            Combat Branch        QoL Branch
                 |                    |
              Armory              Bedroom
              /    \              /    \
      Training    Trophy     Garden   Library
        Yard       Hall        |        |
          |          |     Fish Dock  Workshop
      Watchtower  Alchemy     \       /
          \       Lab          Vault
           \     /           (capstone)
           War Room
          (capstone)
               \               /
                \             /
                 [StormForge]
              (ultimate capstone)
```

Progression: Building a room at T1 unlocks its children. Capstones (War Room, Vault) require T1 of both parent rooms. StormForge requires both capstones (War Room + Vault).

### Room Bonuses (Verified from Implementation)

| Room | Bonus Type | T1 | T2 | T3 | T4 |
|------|-----------|-----|-----|-----|-----|
| **Hearthstone** | Offline XP % | +25% | +50% | +100% | — |
| **Armory** | Damage % | +5% | +10% | +25% | — |
| **Training Yard** | XP Gain % | +5% | +10% | +30% | — |
| **Trophy Hall** | Drop Rate % | +5% | +10% | +15% | — |
| **Watchtower** | Crit Chance % | +5% | +10% | +20% | — |
| **Alchemy Lab** | HP Regen % | +25% | +50% | +100% | — |
| **War Room** | Double Strike % | +10% | +20% | +35% | — |
| **Bedroom** | Regen Delay Reduction | -15% | -30% | -50% | — |
| **Garden** | Fishing Timer Reduction | -10% | -20% | -40% | — |
| **Library** | Challenge Discovery % | +20% | +30% | +50% | — |
| **Fishing Dock** | Double Fish Chance % | +25% | +50% | +100% | +10 Max Rank |
| **Workshop** | Item Rarity % | +10% | +15% | +25% | — |
| **Vault** | Items Survive Prestige | 1 | 3 | 5 | — |
| **StormForge** | Stormbreaker Access | Yes | — | — | — |

### StormForge (Endgame Capstone)

The StormForge is the ultimate Haven room that enables forging the Stormbreaker weapon:

**Build requirements:**
- War Room (combat capstone) at T1+
- Vault (QoL capstone) at T1+
- 25 prestige ranks (build cost)

**Forge requirements** (`can_forge_stormbreaker()`):
1. StormLeviathan achievement unlocked (from fishing quest)
2. 25+ prestige ranks available to spend

**Forging process:**
- Player confirms forge in Haven UI
- 25 prestige ranks deducted
- Unlocks `TheStormbreaker` achievement
- One-time only — cannot forge again

**Complete Stormbreaker progression chain:**
```
1. Reach Fishing Rank 40 (requires FishingDock T4 to extend cap)
2. Encounter Storm Leviathan 10 times while catching legendary fish
3. Catch Storm Leviathan (25% per attempt after 10 encounters)
4. Build entire Haven tree including both capstones
5. Build StormForge (25 prestige ranks)
6. Forge Stormbreaker (25 prestige ranks + StormLeviathan achievement)
7. Defeat The Undying Storm (Zone 10 final boss)
8. Unlock The Expanse (Zone 11, infinite post-game)
```

### Bonus Injection

Haven bonuses are passed as explicit parameters to game systems rather than accessed globally. This keeps modules decoupled. Bonuses are computed when a character is loaded, not recalculated every tick.

## Achievement System

### Overview

Account-level achievement system that persists across all characters. Stored in `~/.quest/achievements.json`.

### Categories (5)

**Combat:**
- Slayer I-IX: 100, 500, 1K, 5K, 10K, 50K, 100K, 500K, 1M kills
- Boss Hunter I-VIII: 1, 10, 50, 100, 500, 1K, 5K, 10K boss kills

**Level:**
- Milestones: L10, L25, L50, L100, L150, L200, L250, L500, L750, L1000, L1500

**Progression:**
- FirstPrestige, then P5, P10, P15, P20, P25, P30, P40, P50, P70, P90, Eternal (P100)

**Challenges:**
- Per-game per-difficulty wins: ChessNovice through ChessMaster, MorrisNovice through MorrisMaster, etc. for all 6 challenge types
- GrandChampion: 100 total minigame wins

**Exploration:**
- Zone1Complete through Zone10Complete
- ExpanseCycleI-IV: 1, 100, 1K, 10K cycles of Zone 11
- StormLeviathan: Caught the Storm Leviathan
- TheStormbreaker: Forged the Stormbreaker
- StormsEnd: Defeated The Undying Storm with Stormbreaker

**Haven:**
- HavenDiscovered, HavenBuilderI (all T1), HavenBuilderII (all T2), HavenArchitect (all T3)

**Fishing:**
- FishermanI-IV: Ranks 10, 20, 30, 40
- FishCatcherI-IV: 100, 1K, 10K, 100K fish caught

**Dungeons:**
- DungeonDiver through DungeonMasterVI: 1, 10, 50, 100, 1K, 5K, 10K dungeons cleared
