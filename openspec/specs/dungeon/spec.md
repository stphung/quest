# Dungeons Specification

## Purpose

Define the optional side-content dungeon: how it is discovered after a kill and immediately entered, how its grid of connected rooms is procedurally generated, the room types and the auto-exploration that walks them, the boss-key gating, the dungeon-specific combat cadence, the rewards for clearing rooms and defeating the dungeon boss, and the safe-exit rules on death or stalemate. Enemy stat pipelines and item generation belong to the combat and items capabilities and appear here only as inputs and outputs.

## Requirements

### Requirement: Dungeon Discovery And Entry

The system SHALL roll for dungeon discovery after every enemy kill while the player is not already inside a dungeon, using a fixed 1% (0.01) chance per kill independent of prestige rank. On a successful roll the system SHALL immediately generate and enter a dungeon shaped by the player's current character level, prestige rank, and current zone, storing that zone as the dungeon's scaling zone. A dungeon discovery in a given kill SHALL suppress the fishing-spot discovery roll for that same kill.

#### Scenario: Discovery roll succeeds

- **WHEN** the player kills an enemy while not in a dungeon and the 1% roll succeeds
- **THEN** a new dungeon is generated and becomes the active dungeon with the player placed at its entrance
- **AND** the zone the player was in is recorded on the dungeon for enemy scaling

#### Scenario: No discovery while already in a dungeon

- **WHEN** an enemy is killed while a dungeon is already active
- **THEN** no dungeon discovery roll occurs and no new dungeon is created

#### Scenario: Prestige does not change discovery rate

- **WHEN** the discovery roll is made at any prestige rank
- **THEN** the discovery chance remains 1% per kill, and prestige rank instead influences the generated dungeon's size and rewards

### Requirement: Dungeon Size Determination

The system SHALL determine dungeon size from a base tier equal to a level component plus a prestige component: the level component is 0 below character level 25, 1 from level 25 to 74, and 2 at level 75 and above; the prestige component is the prestige rank divided by 2 (integer division). The system SHALL then apply a variance roll of -1 tier at 20% probability, no change at 60%, and +1 tier at 20%, clamped to a minimum tier of 0. Tiers map to Small (5x5 grid), Medium (7x7), Large (9x9), Epic (11x11), and Legendary (13x13) at tier 4 and above.

#### Scenario: Low-level fresh character yields a small dungeon band

- **WHEN** a dungeon is rolled for a level-10, prestige-0 character
- **THEN** the base tier is 0 (Small) and the rolled size is Small or, on the +1 variance, Medium

#### Scenario: Prestige raises the size band

- **WHEN** a dungeon is rolled for a level-10 character at prestige rank 6
- **THEN** the base tier is 3 (Epic) before variance is applied

#### Scenario: Size fixes grid and room-count range

- **WHEN** a dungeon of a given size is generated
- **THEN** its grid dimensions and target room-count range are fixed by size: Small 8-12 rooms, Medium 15-20, Large 25-30, Epic 35-45, Legendary 50-65

### Requirement: Procedural Room Generation

The system SHALL carve the dungeon as a connected maze using a randomized depth-first (recursive-backtracker) walk that begins at the grid center and continues until a target room count, rolled within the size's room-count range, is reached. Adjacent carved rooms SHALL be linked by matching directional connections, and the generator SHALL additionally add extra connections between adjacent room pairs at a 15% (0.15) per-pair chance to create loops, while never adding an extra connection to the boss room. Every room in the finished dungeon SHALL be reachable from the entrance.

#### Scenario: Room count stays within the size band

- **WHEN** a dungeon of a given size finishes generation
- **THEN** its total room count lies within that size's target room-count range

#### Scenario: Full connectivity

- **WHEN** any generated dungeon is inspected from the entrance
- **THEN** a path of connected rooms exists from the entrance to every other room, including the boss and elite rooms

### Requirement: Special Room Placement

The system SHALL place exactly one Entrance room, exactly one Elite room, and exactly one Boss room, plus a size-determined number of Treasure rooms (Small 1, Medium 2, Large 3, Epic 5, Legendary 8); all remaining rooms SHALL be Combat rooms. The Boss room SHALL always be a dead end (exactly one connection) positioned at the farthest dead end from the entrance so it can never block the path to the key. The Elite room (the key guardian) SHALL be placed at a dead end far enough from the entrance that it is not adjacent to it, requiring exploration to reach.

#### Scenario: Boss room is a dead end

- **WHEN** any dungeon is generated
- **THEN** the boss room has exactly one connection and is not adjacent to the entrance

#### Scenario: Treasure room count matches size

- **WHEN** a Legendary dungeon is generated
- **THEN** it contains exactly 8 Treasure rooms, one Entrance, one Elite, one Boss, and the rest Combat rooms

#### Scenario: Elite requires exploration

- **WHEN** any dungeon is generated
- **THEN** the elite (key) room is not orthogonally adjacent to the entrance

### Requirement: Room Types And Entry Behavior

The system SHALL treat each room type distinctly on first entry: an Entrance room has no enemy and starts cleared; a Treasure room has no combat, auto-clears on entry, and yields an item; a Combat room, an Elite room, and a Boss room each start combat that must be won before the player may leave. Entering a room SHALL mark the previously occupied room as cleared exactly once and reveal any hidden rooms adjacent to the newly entered room; re-entering an already-cleared room SHALL NOT start combat again nor re-count it as cleared.

#### Scenario: Treasure room auto-clears

- **WHEN** the player enters a Treasure room
- **THEN** the room is immediately cleared, no enemy is fought, and a treasure item is produced

#### Scenario: Combat room blocks departure until cleared

- **WHEN** the player enters a Combat, Elite, or Boss room for the first time
- **THEN** combat starts and the player cannot move to another room until the room's enemy is defeated

#### Scenario: Backtracking through cleared rooms

- **WHEN** the player re-enters a room already marked cleared
- **THEN** no new combat begins and the cleared-room count is not incremented again

### Requirement: Auto-Exploration Movement

The system SHALL auto-explore the dungeon using shortest-path navigation over cleared, current, and target rooms, advancing one room each time an accumulating move timer reaches its interval: 2.5 seconds when entering a new (unexplored) room and 0.8 seconds when traveling through already-cleared rooms. Movement SHALL be blocked while the current room is not yet cleared. A god-item dungeon-speed bonus of P percent SHALL reduce the effective interval by multiplying it by (1 - P/100). The explorer SHALL prefer the nearest unexplored revealed room, and once the key is held SHALL route toward the not-yet-cleared boss room.

#### Scenario: New-room versus travel cadence

- **WHEN** the current room is cleared and a next room is chosen
- **THEN** the player moves after 2.5 seconds if that room is unexplored, or after 0.8 seconds if it is an already-cleared room being traveled through

#### Scenario: Movement blocked during combat

- **WHEN** the current room's combat is not yet resolved
- **THEN** the move timer does not advance the player and no room transition occurs

#### Scenario: Speed bonus shortens the interval

- **WHEN** a 50% dungeon-speed bonus is active
- **THEN** the effective new-room interval is 1.25 seconds and the travel interval is 0.4 seconds

### Requirement: Boss Key Gating

The system SHALL require the boss key before the auto-explorer routes to the boss room. Defeating the Elite room's guardian SHALL grant the key exactly once and unlock the boss. Until the key is held, the explorer SHALL skip the boss room when choosing where to go next.

#### Scenario: Elite grants the key

- **WHEN** the Elite room's guardian is defeated for the first time
- **THEN** the key is acquired and the boss room becomes an unlocked target
- **AND** defeating another elite-type enemy afterward does not grant a second key

#### Scenario: Boss skipped without key

- **WHEN** the boss room is revealed but the key has not been found
- **THEN** the auto-explorer does not select the boss room as its next destination

### Requirement: Dungeon Combat Cadence And Enemy Scaling

The system SHALL apply dungeon-specific enemy attack intervals that take precedence over overworld boss timing: a Boss room enemy attacks every 1.4 seconds and an Elite room enemy attacks every 1.6 seconds, while an ordinary dungeon Combat room enemy uses the normal 2.0-second interval. Dungeon enemies SHALL be scaled from the dungeon's stored discovery zone, with elite enemies amplified (2.2x HP, 1.5x damage, 1.6x defense) and boss enemies amplified further (3.5x HP, 1.8x damage, 2.0x defense) relative to that zone's base combat enemy.

#### Scenario: Elite and boss room attack timing

- **WHEN** the active dungeon's current room is an Elite room
- **THEN** the enemy attacks every 1.6 seconds
- **AND** a Boss room enemy attacks every 1.4 seconds

#### Scenario: Combat room uses the normal interval

- **WHEN** the current dungeon room is an ordinary Combat room
- **THEN** the enemy attacks on the standard 2.0-second interval

#### Scenario: Enemies scale to the discovery zone

- **WHEN** a dungeon discovered in a higher zone spawns any enemy
- **THEN** that enemy's base stats derive from the dungeon's stored discovery zone, not the player's current overworld zone

### Requirement: Dungeon Rewards

The system SHALL award experience for every enemy defeated inside the dungeon and accumulate it into the dungeon run's tally. Entering a Treasure room SHALL generate an item whose rarity is boosted by the dungeon size (+1 tier for Small, Medium, and Large; +2 for Epic; +3 for Legendary, capped at Legendary rarity) and whose item level derives from the player's current zone, and that item SHALL be auto-equipped when it is an upgrade. Defeating the dungeon boss SHALL grant the boss kill experience plus a size-scaled bonus reward (Small 1000-1500, Medium 2000-3000, Large 4000-6000, Epic 8000-12000, Legendary 15000-25000), report the run total and items collected as a completion event, and end the dungeon.

#### Scenario: Treasure rarity boost by size

- **WHEN** a Treasure room item is generated in an Epic dungeon
- **THEN** the item's rolled rarity is raised by 2 tiers, not exceeding Legendary

#### Scenario: Boss completion bonus

- **WHEN** the dungeon boss is defeated in a Large dungeon
- **THEN** a bonus experience reward in the range 4000-6000 is granted on top of the boss kill experience, and the dungeon is cleared and completed

### Requirement: Safe Exit On Death And Stalemate

The system SHALL treat death inside a dungeon as a safe exit: the player leaves the dungeon, the active dungeon is discarded, player HP is restored, and no prestige rank is lost, in contrast to an overworld boss death which resets subzone progress. A dungeon fight that stalls for 60 seconds SHALL trigger a retreat that likewise abandons the dungeon with no prestige loss, preventing the uncleared room's enemy from respawning in an endless loop.

#### Scenario: Death exits with no prestige loss

- **WHEN** the player's HP reaches zero while inside a dungeon
- **THEN** the dungeon is abandoned, the player survives with HP restored, and prestige rank is unchanged

#### Scenario: Stalemate timeout abandons the dungeon

- **WHEN** a single dungeon fight lasts 60 seconds without resolving
- **THEN** the player retreats out of the dungeon safely with no prestige loss and the dungeon is cleared from the active state
