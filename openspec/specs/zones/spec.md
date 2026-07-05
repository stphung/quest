# Zones & World Progression Specification

## Purpose

Define how the hero advances through the game world: a fixed 50-zone structure divided into subzones, boss-gated forward movement, the Zone 10 Stormbreaker weapon gate and the infinitely cycling Expanse, and the two endgame zone bands — Deep-unlocked Fracture zones (12-30) and Woven-Pattern-unlocked Loom zones (31-50). It also fixes the numeric unlock gates, per-zone prestige requirements, and the item-level derived from a zone. Enemy stat pipelines and the internals of the Deep and Loom are owned by other capabilities and referenced here only as unlock sources.

## Requirements

### Requirement: Fifty-Zone World Structure

The system SHALL provide exactly 50 zones, each identified by a zone ID from 1 to 50 and containing an ordered list of subzones, where the last subzone of every zone holds that zone's zone boss. A newly created character SHALL start in Zone 1, Subzone 1 with Zones 1 and 2 unlocked and no bosses defeated.

#### Scenario: Fresh character starting location

- **WHEN** a new character is created
- **THEN** the current location is Zone 1, Subzone 1
- **AND** Zones 1 and 2 are unlocked while Zone 3 and beyond are locked
- **AND** the defeated-boss set is empty

#### Scenario: Final subzone carries the zone boss

- **WHEN** the player reaches the last subzone of a zone
- **THEN** that subzone's boss is the zone boss (zone-completing), and all earlier subzone bosses are non-zone bosses

### Requirement: Subzone Boss Spawning And Kill Tracking

The system SHALL count enemy kills within the current subzone and, upon reaching 10 kills (KILLS_FOR_BOSS), flag the subzone boss to spawn. While the boss fight is active, further kills SHALL NOT increment the counter. The kill counter SHALL reset to 0 both when the subzone boss is defeated and when the player dies (or otherwise retreats by death) during that subzone.

#### Scenario: Boss spawns on the tenth kill

- **WHEN** the player accumulates 10 kills in the current subzone
- **THEN** the boss-fight flag is set and the subzone's named boss is spawned
- **AND** the kills-until-boss count reads 0

#### Scenario: Kills do not accumulate during a boss fight

- **WHEN** an additional enemy is killed while the boss-fight flag is already set
- **THEN** the kill counter stays at 10 and no second boss is queued

#### Scenario: Boss defeat resets the counter

- **WHEN** the subzone boss is defeated
- **THEN** the kill counter is reset to 0 and the boss-fight flag is cleared, so a full 10 kills are needed before the next boss

#### Scenario: Death during a boss fight resets progress

- **WHEN** the player dies (death-loop retreat) while fighting in a subzone
- **THEN** the kill counter is reset to 0 and the boss-fight flag is cleared, requiring the full 10 kills to retry

### Requirement: Subzone Access And Advancement

The system SHALL allow entry to a subzone only when its zone is unlocked and either it is the first subzone or the immediately preceding subzone's boss has been defeated. Defeating a non-final subzone boss SHALL advance the player to the next subzone within the same zone.

#### Scenario: First subzone is always reachable in an unlocked zone

- **WHEN** the player attempts to travel to Subzone 1 of an unlocked zone
- **THEN** travel succeeds

#### Scenario: Later subzone requires the previous boss

- **WHEN** the player attempts to enter Subzone N (N greater than 1) of a zone whose Subzone N-1 boss is not defeated
- **THEN** entry is denied
- **AND** once the Subzone N-1 boss is defeated, entry to Subzone N is allowed

#### Scenario: Non-final subzone boss advances the subzone

- **WHEN** the player defeats a non-final subzone boss
- **THEN** the current subzone advances by one within the same zone

### Requirement: Zone Advancement Prestige And Boss Gate

The system SHALL advance the player to the next zone only when the current zone's final (zone) boss is defeated AND the next zone's prestige requirement is met by the player's prestige rank AND the previous zone's final boss is recorded as defeated. If the zone boss falls but the next zone's prestige requirement is not met, the player SHALL remain in the current zone and be informed of the required prestige rank.

#### Scenario: Zone completes and advances when prestige allows

- **WHEN** the player defeats a zone boss and holds at least the next zone's prestige requirement
- **THEN** the next zone is unlocked and becomes the current zone at Subzone 1

#### Scenario: Zone completes but is prestige-gated

- **WHEN** the player defeats a zone boss but lacks the next zone's prestige requirement
- **THEN** the player stays in the current zone and the outcome reports the required prestige rank for the next zone

### Requirement: Per-Zone Prestige Requirements

The system SHALL gate zone unlocking by prestige rank using these exact minimums: Zones 1-2 require P0; Zones 3-4 require P5; Zones 5-6 require P10; Zones 7-8 require P15; Zones 9-10 require P20; Zone 11 (The Expanse) requires P25; Zones 12-14 require P50; Zones 15-17 require P75; Zones 18-20 require P100; Zones 21-23 require P150; Zones 24-26 require P200; Zones 27-30 require P300; Zones 31-34 require P2,000; Zones 35-38 require P5,000; Zones 39-42 require P15,000; Zones 43-46 require P30,000; Zones 47-50 require P50,000.

#### Scenario: Zone 3 prestige boundary

- **WHEN** the player has defeated Zone 2's final boss
- **THEN** Zone 3 cannot be unlocked at P4 but can be unlocked at exactly P5

#### Scenario: Endgame band prestige minimums

- **WHEN** evaluating the highest bands
- **THEN** a Fracture zone in Zones 27-30 requires P300 and a Loom zone in Zones 47-50 requires P50,000, enforced alongside their other gates

### Requirement: Item Level Derivation From Zone

The system SHALL derive the item level (ilvl) of drops from the source zone as ilvl = zone_id × 10.

#### Scenario: Item level scales with zone ID

- **WHEN** an item drops in Zone 1
- **THEN** its ilvl is 10
- **AND** an item dropping in Zone 5 has ilvl 50 and one dropping in Zone 10 has ilvl 100

### Requirement: Zone 10 Stormbreaker Weapon Gate

The system SHALL block defeat of the Zone 10 (Storm Citadel) final boss unless the player has forged the Stormbreaker (tracked by the Stormbreaker achievement). Attempting to defeat this gated boss without Stormbreaker SHALL leave the boss undefeated and reset the encounter so it can be retried. The gate SHALL apply only to Zone 10's final subzone boss, not to Zone 10's earlier subzone bosses nor to any other zone's boss.

#### Scenario: Blocked without Stormbreaker

- **WHEN** the player fights the Zone 10 final boss without the Stormbreaker achievement
- **THEN** the outcome reports that the Stormbreaker weapon is required
- **AND** the boss is not recorded as defeated and the encounter is reset for a retry

#### Scenario: Allowed with Stormbreaker

- **WHEN** the player fights the Zone 10 final boss and holds the Stormbreaker achievement
- **THEN** the weapon gate does not block the fight

#### Scenario: Gate does not affect other bosses

- **WHEN** the player fights a non-final Zone 10 subzone boss, or any boss in a zone other than Zone 10, without Stormbreaker
- **THEN** the weapon gate does not apply

### Requirement: Zone 10 Completion And The Expanse Cycle

The system SHALL, upon defeating the Zone 10 final boss (with Stormbreaker), unlock the Storm's End achievement, unlock Zone 11 (The Expanse), and move the player into Zone 11 at Subzone 1. Zone 11 SHALL be dual-gated by the Storm's End achievement and prestige rank P25. Defeating Zone 11's zone boss SHALL cycle back to Subzone 1 indefinitely while no Fracture zones are unlocked; once Fracture zones are unlocked (Fracture cap above 11), defeating Zone 11's boss SHALL instead advance to Zone 12.

#### Scenario: Storm's End on Zone 10 completion

- **WHEN** the player defeats the Zone 10 final boss with Stormbreaker
- **THEN** the Storm's End achievement unlocks, Zone 11 unlocks, and the player is moved to Zone 11, Subzone 1

#### Scenario: Expanse cycles when no Fracture is open

- **WHEN** the player defeats the Zone 11 boss and the Fracture zone cap is 11 or lower
- **THEN** the player returns to Zone 11, Subzone 1 (Expanse cycle)

#### Scenario: Expanse advances to Zone 12 when Fracture is open

- **WHEN** the player defeats the Zone 11 boss, the Fracture zone cap is above 11, and Zone 12 is unlocked
- **THEN** the player advances into Zone 12 at Subzone 1

### Requirement: Fracture Zones Unlocked By Deep Layers

The system SHALL unlock Fracture zones 12-30 in chapters as Deep layer breakthroughs raise the Fracture zone cap, with each zone also honoring its prestige requirement: Deep Layer 3 opens Zones 12-14 (P50); Layer 7 opens Zones 15-17 (P75); Layer 12 opens Zones 18-20 (P100); Layer 18 opens Zones 21-23 (P150); Layer 25 opens Zones 24-26 (P200); Layer 30 opens Zones 27-30 (P300). Fracture enemy stats SHALL scale by 1.6x per zone from the Zone 11 base. Defeating the boss of the current Fracture cap zone SHALL cycle it back to Subzone 1, while non-cap Fracture zones advance forward, and Zone 30 is the permanent Fracture loop cap.

#### Scenario: Red Fault opens at Deep Layer 3

- **WHEN** the Deep reaches a Layer 3 breakthrough and the player holds P50
- **THEN** Zones 12, 13, and 14 become unlocked and Zone 15 stays locked

#### Scenario: Full Fracture band at cap 30

- **WHEN** the Fracture cap is 30 and prestige is at least P300
- **THEN** every zone from 12 through 30 is unlocked and Zone 31 remains locked by this gate

#### Scenario: Prestige still gates within an opened cap

- **WHEN** the Fracture cap is 20 but the player holds only P50
- **THEN** Zones 12-14 unlock while Zones 15 and above stay locked until P75 and beyond are reached

#### Scenario: Cap zone cycles instead of advancing

- **WHEN** the player defeats the zone boss of the current Fracture cap zone and no Loom zone beyond it is unlocked
- **THEN** the zone returns to Subzone 1 (Fracture cycle) rather than advancing

### Requirement: Loom Zones Triple-Gated By Patterns, Ascension, And Prestige

The system SHALL unlock Loom zones 31-50 only when all three gates are satisfied — completed Woven Patterns, Ascension tier, and prestige rank — per chapter: Zones 31-34 require 4 patterns, no Ascension requirement, and P2,000; Zones 35-38 require 8 patterns, Ascension VII, and P5,000; Zones 39-42 require 16 patterns, Ascension VIII, and P15,000; Zones 43-46 require 22 patterns, Ascension IX, and P30,000; Zones 47-50 require 28 patterns, Ascension X, and P50,000. Loom enemy stats SHALL scale by 1.25x per zone from the Zone 30 base. Defeating the boss of the current Loom cap zone SHALL cycle it back to Subzone 1.

#### Scenario: First Loom chapter unlocks

- **WHEN** the player has completed 4 Woven Patterns and holds P2,000 (no Ascension requirement)
- **THEN** Zones 31-34 unlock and Zone 35 stays locked

#### Scenario: Ascension gate blocks the second chapter

- **WHEN** the player has 28 patterns and P50,000 but only Ascension VI
- **THEN** Zones 31-34 unlock but Zone 35 remains locked until Ascension VII is reached

#### Scenario: Prestige gate within the pattern cap

- **WHEN** the pattern count would allow up to Zone 50 but prestige is only P2,000
- **THEN** only Zones 31-34 unlock and Zone 35 and beyond stay locked

#### Scenario: Loom cap zone cycles

- **WHEN** the player defeats the zone boss of the current Loom cap zone
- **THEN** the zone returns to Subzone 1 (Loom-zone cycle)

### Requirement: Prestige Reset Of Progression

The system SHALL, on prestige reset, return the player to Zone 1, Subzone 1, clear all defeated bosses and kill tracking, clear death-retreat memory, and recompute the unlocked set to include every zone with ID at most 30 whose prestige requirement is met by the new rank. Loom zones (31+) SHALL NOT be unlocked by the reset itself and are re-synced afterward against their Ascension and prestige gates.

#### Scenario: Reset returns to the start and keeps prestige-eligible zones

- **WHEN** the player prestiges to a rank of P5
- **THEN** the current location is Zone 1, Subzone 1 with an empty defeated-boss set
- **AND** Zones 1-4 are unlocked (P0 and P5 zones) while Zone 5 (P10) is not

#### Scenario: Zone 1 always survives reset

- **WHEN** a prestige reset occurs at any rank
- **THEN** Zone 1 is unlocked and is the current zone

### Requirement: Frontier Death-Loop Backoff

The system SHALL, when a zone-boss defeat would otherwise advance the player into a zone that recently caused a death-loop retreat, instead cycle the current zone back to Subzone 1 and consume one cooldown cycle, up to a capped number of cycles (maximum 8) that grows with repeated retreats. The backoff SHALL be cleared when the player defeats any boss in the recorded retreat zone or on prestige reset, and SHALL only affect the specific zone that was retreated from.

#### Scenario: Backoff cycles instead of advancing into a killer zone

- **WHEN** the player has recorded death-loop retreats from the next zone and then defeats the current zone's boss
- **THEN** the current zone cycles to Subzone 1 and one cooldown cycle is consumed rather than advancing forward

#### Scenario: Backoff clears once the zone is survived

- **WHEN** the player defeats a boss in the zone previously retreated from
- **THEN** the death-retreat memory is cleared and normal advancement resumes

#### Scenario: Backoff is scoped to the retreated zone

- **WHEN** a retreat was recorded for a distant zone unrelated to the current frontier
- **THEN** advancing from the current zone into its normal next zone is unaffected
