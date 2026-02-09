# Game Systems Architecture

This document visualizes the game flow, tick processing order, and how all systems relate to each other.

## High-Level System Map

```
                          ┌──────────────────────┐
                          │      main.rs         │
                          │    (Game Loop)        │
                          │  100ms tick cycle     │
                          └──────┬───────────────┘
                                 │
                    ┌────────────┼────────────────┐
                    ▼            ▼                 ▼
              ┌──────────┐ ┌──────────┐    ┌────────────┐
              │  Input   │ │ Game     │    │    UI      │
              │ Routing  │ │ Tick     │    │ Rendering  │
              │(input.rs)│ │(game_    │    │ (ui/)      │
              │          │ │ logic.rs)│    │            │
              └──────────┘ └────┬─────┘    └────────────┘
                                │
           ┌────────────────────┼──────────────────────┐
           │                    │                       │
     ┌─────┴──────┐     ┌──────┴──────┐     ┌─────────┴────────┐
     │  Combat    │     │  Dungeon    │     │    Fishing        │
     │  System    │     │  System     │     │    System         │
     │(combat/)   │     │(dungeon/)   │     │  (fishing/)       │
     └─────┬──────┘     └──────┬──────┘     └─────────┬────────┘
           │                   │                       │
     ┌─────┴──────┐     ┌─────┴──────┐         ┌──────┴──────┐
     │   Zone     │     │   Item     │         │  Character  │
     │Progression │     │   Drops    │         │  Attributes │
     │ (zones/)   │     │ (items/)   │         │(character/) │
     └────────────┘     └────────────┘         └──────┬──────┘
                                                      │
           ┌──────────────────────────────────────────┤
           │                    │                      │
     ┌─────┴──────┐     ┌──────┴──────┐     ┌────────┴───────┐
     │  Prestige  │     │   Haven     │     │  Achievements  │
     │  System    │     │  (Account)  │     │   (Account)    │
     │(prestige.rs│     │ (haven/)    │     │(achievements/) │
     └────────────┘     └─────────────┘     └────────────────┘
                               │
                        ┌──────┴──────┐
                        │ Challenges  │
                        │ (Minigames) │
                        │(challenges/)│
                        └─────────────┘
```

## Game Loop: One Tick (100ms)

Every 100 milliseconds the game processes one tick. Here is the exact order of operations:

```
┌─────────────────────────────────────────────────────────────────┐
│                     GAME TICK (100ms)                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  1. AI THINKING                                                  │
│     Chess AI ─────┐                                              │
│     Morris AI ────┤  Process background AI computation           │
│     Gomoku AI ────┤  (non-blocking, accumulates over ticks)      │
│     Go AI ────────┘                                              │
│                                                                  │
│  2. CHALLENGE DISCOVERY                                          │
│     Roll once per tick (requires P1+)                            │
│     ~0.014% base chance, boosted by Haven Library                │
│     Weighted table: Minesweeper > Rune > Gomoku > Morris > ...  │
│                                                                  │
│  3. SYNC DERIVED STATS                                           │
│     Recalculate max_hp, damage, defense, crit from:             │
│       attributes + equipment + prestige                          │
│                                                                  │
│  4. DUNGEON UPDATE ──── (if active_dungeon is Some)              │
│     Navigate rooms, process room combat, award treasure          │
│     └─► Events: RoomEntered, FoundKey, DungeonComplete           │
│                                                                  │
│  5. FISHING ─────────── (if active_fishing, SKIPS combat)        │
│     Casting → Waiting → Reeling → Catch                          │
│     └─► Awards XP + fishing rank progress                        │
│                                                                  │
│  6. COMBAT ──────────── (if NOT fishing & NOT in dungeon)        │
│     Every 1.5s: player attacks → enemy counters                  │
│     Haven bonuses injected: damage%, crit%, regen, etc.          │
│     └─► Events: PlayerAttack, EnemyAttack, Death                 │
│                                                                  │
│  7. PROCESS COMBAT EVENTS                                        │
│     EnemyDied ─────► apply XP ──► check level-up ──► try drop   │
│     BossDefeated ──► complete dungeon                            │
│     SubzoneBoss ───► advance zone/subzone                        │
│                                                                  │
│  8. VISUAL EFFECTS                                               │
│     Decay damage numbers, attack flashes, hit impacts            │
│                                                                  │
│  9. SPAWN ENEMY                                                  │
│     If no enemy & not regenerating & not in treasure room        │
│     Zone enemy or Dungeon room enemy                             │
│                                                                  │
│ 10. PLAY TIME + ACHIEVEMENT LOG                                  │
│     Increment playtime, log newly unlocked achievements          │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Input Routing Priority Chain

Input is checked top-to-bottom; the first matching handler consumes the event.

```
┌─────────────────────────────────────────────┐
│              KEYBOARD INPUT                  │
└──────────────────┬──────────────────────────┘
                   ▼
         ┌─── Overlays ───┐
         │                 │
         │  1. Offline welcome (any key dismisses)
         │  2. Storm Leviathan modal (Enter)
         │  3. Achievement browser (Esc/A)
         │  4. Haven discovery modal (Enter/Esc)
         │  5. Achievement unlock modal (any key)
         │                 │
         └────────┬────────┘
                  ▼
         ┌─── Screens ────┐
         │                 │
         │  6. Haven overlay (blocks all input)
         │  7. Vault selection (prestige items)
         │  8. Prestige confirmation (Y/N/Esc)
         │  9. Debug menu (` to toggle)
         │                 │
         └────────┬────────┘
                  ▼
         ┌── Minigames ───┐
         │                 │
         │ 10. Active minigame (game-specific keys)
         │ 11. Challenge menu (Tab opens)
         │                 │
         └────────┬────────┘
                  ▼
         ┌── Base Game ───┐
         │                 │
         │ 12. P = Prestige, H = Haven
         │     A = Achievements, U = Update
         │     Q = Quit to character select
         │                 │
         └─────────────────┘
```

## Activity Exclusion: What Can Run Simultaneously

Only one primary activity runs per tick. The priority is:

```
  Active Minigame?  ───yes──►  Minigame processes (blocks all else)
         │ no
         ▼
  Active Dungeon?   ───yes──►  Dungeon update (has its own combat)
         │ no
         ▼
  Active Fishing?   ───yes──►  Fishing tick (SKIPS zone combat)
         │ no
         ▼
  Zone Combat ──────────────►  Normal enemy combat + zone progression
```

Challenge discovery and AI thinking run *every* tick regardless of activity.

## Enemy Defeat: XP and Item Flow

```
                 ┌──────────────┐
                 │ Enemy Killed │
                 └──────┬───────┘
                        │
                        ▼
              ┌─────────────────────┐
              │   Calculate XP      │
              │                     │
              │ base × prestige_mul │
              │ × CHA bonus (10%)   │
              │ × WIS bonus (5%)    │
              │ × Haven XP bonus    │
              └─────────┬───────────┘
                        │
            ┌───────────┴───────────┐
            ▼                       ▼
   ┌────────────────┐    ┌──────────────────┐
   │  Apply XP      │    │  Try Item Drop   │
   │                │    │                  │
   │ xp += amount   │    │ 15% base         │
   │ check level-up │    │ +1% per prestige │
   └───────┬────────┘    │ cap at 25%       │
           │             │ × Haven Trophy   │
           ▼             └────────┬─────────┘
   ┌────────────────┐             │
   │  Level Up?     │      ┌─────┴─────┐
   │                │      │  yes  no  │
   │ xp >= curve?   │      │     └─────┼──► nothing
   │ 100 × lvl^1.5  │      ▼           │
   │                │ ┌──────────────┐  │
   │ +3 random attr │ │Generate Item │  │
   │ points (capped)│ │              │  │
   └────────────────┘ │ rarity roll  │  │
                      │ attr gen     │  │
                      │ affix gen    │  │
                      │ name gen     │  │
                      └──────┬───────┘  │
                             │          │
                             ▼          │
                      ┌──────────────┐  │
                      │  Auto-Equip  │  │
                      │              │  │
                      │ score new vs │  │
                      │ current item │  │
                      │ weighted by  │  │
                      │ specializ.   │  │
                      │              │  │
                      │ better? swap │  │
                      └──────┬───────┘  │
                             │          │
                             ▼          │
                      ┌──────────────┐  │
                      │ Loot Panel   │◄─┘
                      │ (UI display) │
                      └──────────────┘
```

## Zone Progression Flow

```
   ┌──────────────────────────────────────────────────┐
   │              ZONE MAP (10 zones)                  │
   │                                                   │
   │  Zone 1-2  (P0)   ── 3 subzones each            │
   │  Zone 3-4  (P5)   ── 3 subzones each            │
   │  Zone 5-6  (P10)  ── 4 subzones each            │
   │  Zone 7-8  (P15)  ── 4 subzones each            │
   │  Zone 9-10 (P20)  ── 4 subzones each            │
   │  The Expanse       ── infinite cycles            │
   └──────────────────────────────────────────────────┘

   Progression within a subzone:

   ┌────────────┐    ┌────────────┐    ┌──────────────┐
   │ Kill enemy │──►│ kills += 1 │──►│ kills == 10? │
   └────────────┘    └────────────┘    └──────┬───────┘
                                              │
                                    ┌─────────┴─────────┐
                                    │ yes               │ no
                                    ▼                   ▼
                           ┌────────────────┐    (keep killing)
                           │ Spawn Subzone  │
                           │    Boss        │
                           └───────┬────────┘
                                   │
                                   ▼
                           ┌────────────────┐
                           │ Boss defeated  │
                           └───────┬────────┘
                                   │
                          ┌────────┴────────┐
                          │ Zone boss?      │
                          ▼                 ▼
                  ┌──────────────┐  ┌──────────────┐
                  │ Advance to   │  │ Advance to   │
                  │ next zone    │  │ next subzone  │
                  │ (if prestige │  │ kills = 0     │
                  │  allows)     │  │               │
                  └──────────────┘  └──────────────┘

   Death to boss: kills_in_subzone resets to 0, must re-earn 10 kills
   Zone 10 final boss: requires Stormbreaker weapon (Haven Storm Forge)
```

## Dungeon Lifecycle

```
   ┌────────────────────────────────────────┐
   │ 2% chance per combat kill (not in      │
   │ dungeon) to discover a dungeon         │
   └──────────────────┬─────────────────────┘
                      ▼
   ┌────────────────────────────────────────┐
   │ Generate dungeon                       │
   │ Size by prestige: P0=5×5 ... P20=13×13│
   │ Rooms: Combat 60%, Treasure 20%,      │
   │        Elite 15%, Boss 5%             │
   └──────────────────┬─────────────────────┘
                      ▼
   ┌──────────────────────────────────────────────────────────┐
   │                    DUNGEON LOOP                           │
   │                                                           │
   │  ┌─────────┐   move    ┌───────────────┐                │
   │  │ Current ├──────────►│ Adjacent Room │                │
   │  │  Room   │           │ (revealed)    │                │
   │  └─────────┘           └───────┬───────┘                │
   │                                │                         │
   │                    ┌───────────┴────────────┐            │
   │                    ▼           ▼            ▼            │
   │              ┌──────────┐ ┌─────────┐ ┌──────────┐      │
   │              │ Combat   │ │Treasure │ │  Elite   │      │
   │              │ Room     │ │ Room    │ │  Room    │      │
   │              │          │ │         │ │          │      │
   │              │ fight    │ │ auto-   │ │ fight    │      │
   │              │ enemy    │ │ collect │ │ elite    │      │
   │              │ (1.0x)   │ │ item    │ │ (1.5x)  │      │
   │              └──────────┘ └─────────┘ │ +key    │      │
   │                                       └──────────┘      │
   │                                                          │
   │              ┌──────────────────────────┐                │
   │              │      Boss Room           │                │
   │              │  (requires key from      │                │
   │              │   elite room)            │                │
   │              │                          │                │
   │              │  fight boss (2.0x)       │                │
   │              └───────────┬──────────────┘                │
   │                          │                               │
   └──────────────────────────┼───────────────────────────────┘
                              │
                   ┌──────────┴──────────┐
                   ▼                     ▼
          ┌──────────────┐      ┌──────────────┐
          │ Boss defeated│      │ Player dies  │
          │              │      │              │
          │ Dungeon      │      │ Exit dungeon │
          │ Complete!    │      │ No prestige  │
          │ XP + items   │      │ loss         │
          └──────────────┘      └──────────────┘
```

## Prestige Flow

```
   ┌────────────────────────────────────────────┐
   │ Player presses P                           │
   │ Level >= requirement (10 for P1, scales)   │
   └──────────────────┬─────────────────────────┘
                      ▼
           ┌──────────────────────┐
           │ Prestige Confirm     │
           │ dialog (Y/N/Esc)     │
           └──────────┬───────────┘
                      │ Y
                      ▼
           ┌──────────────────────┐     ┌──────────────────────┐
           │ Has Haven Vault?     │─yes─│ Vault Selection      │
           │                      │     │ Pick items to keep   │
           └──────────┬───────────┘     │ (1-3 slots by tier)  │
                      │ no              └──────────┬───────────┘
                      ▼                            │
           ┌───────────────────────────────────────┘
           │
           ▼
   ┌──────────────────────────────────────────────┐
   │             perform_prestige()                │
   │                                               │
   │  RESET:                                       │
   │    level → 1                                  │
   │    xp → 0                                     │
   │    all attributes → 10                        │
   │    zone → 1, subzone → 1                      │
   │    active dungeon → None                      │
   │    active fishing → None                      │
   │    challenge menu → cleared                   │
   │                                               │
   │  KEEP:                                        │
   │    prestige_rank += 1                         │
   │    equipment (non-vaulted cleared)            │
   │    fishing state (ranks, catches)             │
   │    chess stats                                │
   │    achievements (account-level)               │
   │    Haven (account-level)                      │
   │    vaulted items preserved                    │
   │                                               │
   │  GAIN:                                        │
   │    higher XP multiplier (1 + 0.5 × rank^0.7) │
   │    higher attribute cap (20 + rank × 5)       │
   │    higher item drop rate (+1% per rank)       │
   │    access to harder zones                     │
   └──────────────────────────────────────────────┘
```

## Haven Bonus Injection Pattern

Haven bonuses flow into other systems through explicit parameters, keeping modules decoupled.

```
   ┌──────────────────────────────────────────────────────────────┐
   │                       HAVEN                                   │
   │                                                               │
   │                    Hearthstone                                │
   │                   /           \                               │
   │              Armory          Bedroom                          │
   │             /      \        /      \                          │
   │      TrainingYard TrophyHall Garden Library                  │
   │         /           |        |      |                        │
   │    Watchtower  AlchemyLab FishDock Workshop                  │
   │         \           /        |      |                        │
   │          War Room            |      |                        │
   │              \               |      |                        │
   │               Vault ─────── | ──── |                         │
   │                  \          |      |                         │
   │                Storm Forge ─┘──────┘                         │
   └────────────────────────┬─────────────────────────────────────┘
                            │
          ┌─────────────────┼──────────────────────┐
          │                 │                       │
          ▼                 ▼                       ▼
   ┌──────────────┐  ┌──────────────┐  ┌────────────────────┐
   │ Combat       │  │ Items        │  │ Fishing            │
   │ Bonuses      │  │ Bonuses      │  │ Bonuses            │
   │              │  │              │  │                    │
   │ Armory:      │  │ Trophy Hall: │  │ Garden: timer      │
   │  +damage%    │  │  +drop rate% │  │  reduction%        │
   │ Watchtower:  │  │ Workshop:    │  │ Fishing Dock:      │
   │  +crit%      │  │  +rarity     │  │  double fish%      │
   │ AlchemyLab:  │  │  shift%      │  │  max rank bonus    │
   │  +regen%     │  └──────────────┘  └────────────────────┘
   │ Bedroom:     │
   │  -regen delay│  ┌──────────────┐  ┌────────────────────┐
   │ War Room:    │  │ Challenges   │  │ Offline / Prestige │
   │  double      │  │              │  │                    │
   │  strike%     │  │ Library:     │  │ Hearthstone:       │
   │ TrainingYard:│  │  +discovery  │  │  offline XP mult   │
   │  +xp%        │  │  rate%       │  │ Vault: preserve    │
   └──────────────┘  └──────────────┘  │  items on prestige │
                                       │ Storm Forge:       │
                                       │  craft Stormbreaker│
                                       └────────────────────┘
```

## Fishing Session Flow

```
   ┌────────────────────────────────────────────┐
   │ 5% chance per non-combat tick when:        │
   │   - not in dungeon                         │
   │   - not in combat                          │
   │   - not in minigame                        │
   └──────────────────┬─────────────────────────┘
                      ▼
   ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
   │ CASTING  │───►│ WAITING  │───►│ REELING  │───►│  CATCH   │
   │  ~1s     │    │  2-4s    │    │  1-2s    │    │          │
   │          │    │ (Haven   │    │          │    │ rarity   │
   │          │    │  Garden  │    │          │    │ roll     │
   │          │    │  reduces)│    │          │    │          │
   └──────────┘    └──────────┘    └──────────┘    └─────┬────┘
                                                         │
                                                         ▼
                                                  ┌──────────────┐
                                                  │ Award XP     │
                                                  │ +rank points  │
                                                  │ +catches      │
                                                  └──────┬───────┘
                                                         │
                                                  ┌──────┴───────┐
                                                  │ Rank 40 +    │
                                                  │ 10 legendary?│
                                                  └──────┬───────┘
                                                         │ yes
                                                  ┌──────▼───────┐
                                                  │ Storm        │
                                                  │ Leviathan    │
                                                  │ encounter    │
                                                  │ (10 escapes  │
                                                  │  then catch) │
                                                  └──────────────┘
```

## Challenge Minigame Lifecycle

```
   ┌──────────────────────────────────────────────┐
   │ Every tick: roll for discovery (P1+ required) │
   │ Base ~0.014% + Haven Library bonus            │
   │                                               │
   │ Weighted table:                               │
   │   Minesweeper 30 │ Rune 25 │ Gomoku 20       │
   │   Morris 15 │ Chess 10 │ Go 10               │
   └──────────────────────┬────────────────────────┘
                          ▼
                  ┌───────────────┐
                  │ Challenge     │
                  │ added to menu │
                  │ (banner shown)│
                  └───────┬───────┘
                          │ Tab key
                          ▼
                  ┌───────────────┐
                  │ Challenge     │
                  │ Menu          │
                  │               │
                  │ Select game   │
                  │ Pick difficulty│
                  │ (Novice →     │
                  │  Master)      │
                  └───────┬───────┘
                          │ Enter
                          ▼
    ┌─────────────────────────────────────────────┐
    │            ACTIVE MINIGAME                   │
    │  (blocks combat, fishing, zone progression)  │
    │                                              │
    │  ┌────────┐ ┌──────┐ ┌───────┐ ┌─────────┐ │
    │  │ Chess  │ │  Go  │ │Morris │ │ Gomoku  │ │
    │  │ 8×8   │ │ 9×9  │ │ board │ │ 15×15   │ │
    │  └────────┘ └──────┘ └───────┘ └─────────┘ │
    │  ┌────────────┐ ┌──────────┐                │
    │  │Minesweeper │ │  Rune    │                │
    │  │ 9×9-20×16  │ │ deduction│                │
    │  └────────────┘ └──────────┘                │
    │                                              │
    │  Forfeit: Esc → Esc (two-press confirm)     │
    └─────────────────────┬───────────────────────┘
                          │
               ┌──────────┴──────────┐
               ▼                     ▼
       ┌──────────────┐     ┌──────────────┐
       │     WIN      │     │ LOSS/FORFEIT │
       │              │     │              │
       │ +prestige    │     │ no penalty   │
       │ +XP (% of    │     │              │
       │  next level) │     │              │
       │ +fish ranks  │     │              │
       └──────────────┘     └──────────────┘
```

## Achievement Tracking Integration

Achievements are tracked at specific events across all systems:

```
   ┌──────────────────────────────────────────────────────────────┐
   │                    EVENT SOURCES                              │
   │                                                               │
   │  Combat ────► on_enemy_killed()  ──► Slayer I-IX             │
   │              on_boss_killed()   ──► Boss Hunter I-VIII        │
   │                                                               │
   │  Leveling ──► on_level_up()     ──► Level 10..1500           │
   │                                                               │
   │  Prestige ──► on_prestige()     ──► Prestige P1..P∞          │
   │                                                               │
   │  Zones ─────► on_zone_cleared() ──► Zone 1-10 Complete       │
   │                                                               │
   │  Dungeons ──► on_dungeon_done() ──► Dungeon achievements     │
   │                                                               │
   │  Fishing ───► on_leviathan()    ──► Storm Leviathan          │
   │                                                               │
   │  Minigames ─► on_minigame_won() ──► Challenge achievements   │
   │                                                               │
   │  Haven ─────► on_haven_found()  ──► Haven Discovered         │
   │              on_stormbreaker()  ──► Stormbreaker Forged       │
   │                                                               │
   └─────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │ achievements/   │
                    │                 │
                    │ Account-level   │
                    │ Persists across │
                    │ all characters  │
                    │                 │
                    │ ~/.quest/       │
                    │ achievements.   │
                    │ json            │
                    └─────────────────┘
```

## Persistence: What Saves Where

```
   ┌─────────────────────────────────────────────────────────────┐
   │                   ~/.quest/ directory                        │
   │                                                              │
   │  {character_name}.json          Per-Character Save           │
   │  ├── level, xp, attributes                                  │
   │  ├── prestige_rank                                          │
   │  ├── combat_state (current enemy)                           │
   │  ├── equipment (7 slots)                                    │
   │  ├── zone_progression (zone, subzone, kills)                │
   │  ├── active_dungeon (rooms, position, keys)                 │
   │  ├── fishing (rank, catches, leviathan state)               │
   │  └── chess_stats                                            │
   │                                                              │
   │  haven.json                     Account-Level Haven          │
   │  ├── rooms built + tiers                                    │
   │  ├── discovery state                                        │
   │  └── storm_forge state                                      │
   │                                                              │
   │  achievements.json              Account-Level Achievements   │
   │  └── unlocked achievement IDs + timestamps                  │
   │                                                              │
   └─────────────────────────────────────────────────────────────┘

   Auto-save: every 30 seconds
   Manual save: on quit, prestige, character switch
```

## UI Layout

```
   ┌─ Challenge Banner (if pending challenges) ──────────────────┐
   ├──────────────────────────┬──────────────────────────────────┤
   │                          │                                  │
   │   STATS PANEL (50%)      │   MAIN VIEW (50%)               │
   │                          │                                  │
   │   Character name/level   │   Zone combat (default)          │
   │   XP bar                 │    ─or─                          │
   │   Prestige rank          │   Dungeon 3D view + minimap      │
   │   Attributes (6)         │    ─or─                          │
   │   Derived stats          │   Fishing scene                  │
   │   Equipment (7 slots)    │    ─or─                          │
   │   Zone / subzone         │   Minigame board                 │
   │                          │                                  │
   ├──────────────────────────┴──────────────────────────────────┤
   │ LOOT PANEL (left half)     │   COMBAT LOG (right half)      │
   │ Recent item drops          │   Damage, XP, events           │
   │ (last 8 entries)           │   (last 8 entries)             │
   ├────────────────────────────┴────────────────────────────────┤
   │ FOOTER: controls hint, version, play time                   │
   └─────────────────────────────────────────────────────────────┘

   Overlays (drawn on top):
   ├── Haven building screen
   ├── Prestige confirmation dialog
   ├── Achievement browser
   ├── Debug menu (--debug flag)
   └── Modal popups (achievements, discoveries)
```

## System Dependency Summary

| System | Depends On | Feeds Into |
|--------|-----------|------------|
| **Game Loop** | All systems | UI rendering |
| **Combat** | Zone data, Derived stats, Haven bonuses | XP, Items, Zone progression, Achievements |
| **Zones** | Prestige rank (gates), Combat (boss defeats) | Enemy generation, Progression |
| **Dungeons** | Combat (kills trigger discovery), Prestige (size) | Items, XP, Achievements |
| **Fishing** | Tick timer, Haven bonuses | XP, Fishing rank, Achievements |
| **Items** | Combat (drops), Dungeon (treasure), Haven (rates) | Equipment, Derived stats, Auto-equip |
| **Prestige** | Level (requirement), Haven (vault) | XP mult, Attr cap, Drop rate, Zone access |
| **Haven** | Prestige ranks (costs), Fishing ranks (costs) | Bonuses to all systems (parameter injection) |
| **Challenges** | Prestige P1+ (gate), Haven Library (rate) | Prestige ranks, XP, Fish ranks |
| **Achievements** | All systems (event tracking) | UI notifications |
| **Character** | Attributes, Equipment | Derived stats (HP, damage, defense, crit) |
| **Derived Stats** | Attributes + Equipment + Prestige | Combat scaling, Enemy generation |
