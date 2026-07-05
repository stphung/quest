# Haven — Account Base Building Specification

## Purpose

Define the Haven: an account-level base that persists across every prestige reset and is shared by all of the account's characters. The Haven is discovered by chance once the player reaches a prestige-rank threshold, then grown as a two-branch skill tree of rooms bought and upgraded with prestige ranks. Each room grants a passive bonus (combat damage, XP, drop rate, fishing, offline progress, and more) that is supplied to the consuming systems as an explicit input rather than read from global state. This capability owns discovery, the tree structure, room costs, the bonus magnitudes, and the decoupling rule; combat, XP, items, and fishing are referenced only as consumers of Haven bonuses.

## Requirements

### Requirement: Account-Level Persistence

The Haven SHALL be account-level state that is not tied to any single character: it SHALL persist unchanged across prestige resets, SHALL be shared across all characters on the account, and SHALL be stored separately from per-character save data. Discovery status and every room's tier SHALL survive prestige and character switches.

#### Scenario: Haven survives prestige

- **WHEN** a character prestiges (resetting level, zone, and equipment)
- **THEN** the Haven's discovered status and all built room tiers are retained unchanged

#### Scenario: Haven shared across characters

- **WHEN** the player switches to a different character on the same account
- **THEN** the same Haven (discovery status and room tiers) applies to that character

### Requirement: Prestige-Gated Random Discovery

The Haven SHALL be discovered by an independent per-tick random roll that is only possible at Prestige Rank 10 or higher. The per-tick discovery chance SHALL be exactly `0.000014 + (prestige_rank − 10) × 0.000007`, and SHALL be exactly 0 for any prestige rank below 10. The roll SHALL only be attempted while the Haven is undiscovered and no dungeon, fishing session, or challenge minigame is active. Once discovered, the Haven SHALL remain discovered permanently, and discovery SHALL be persisted.

#### Scenario: Below the prestige threshold

- **WHEN** the player is below Prestige Rank 10
- **THEN** the discovery chance is 0 and the Haven can never be discovered

#### Scenario: Discovery chance scales with rank

- **WHEN** the player is at Prestige Rank 10
- **THEN** the per-tick discovery chance is 0.000014
- **AND** at Prestige Rank 12 it is 0.000028, higher than at Rank 10

#### Scenario: Discovery is suppressed during active content

- **WHEN** a dungeon, fishing session, or challenge minigame is active
- **THEN** no Haven discovery roll is attempted that tick

#### Scenario: Discovery persists

- **WHEN** the discovery roll succeeds on a given tick
- **THEN** the Haven is marked discovered and stays discovered across future ticks and sessions

### Requirement: Two-Branch Skill Tree With Prerequisites

The Haven SHALL consist of exactly 14 rooms arranged as a skill tree rooted at Hearthstone. From Hearthstone the tree SHALL split into a combat branch (Armory → Training Yard / Trophy Hall → Watchtower / Alchemy Lab → War Room) and a quality-of-life branch (Bedroom → Garden / Library → Fishing Dock / Workshop → Vault). A room SHALL be unlocked only when every one of its parent rooms is built to at least Tier 1. The two capstone rooms, War Room and Vault, SHALL each require both of their parents. Storm Forge SHALL require both capstones (War Room and Vault).

#### Scenario: Root is always available

- **WHEN** a freshly discovered Haven has no rooms built
- **THEN** Hearthstone is unlocked and buildable, but its children (Armory, Bedroom) are locked

#### Scenario: Child requires parent at Tier 1

- **WHEN** Hearthstone is built to Tier 1
- **THEN** Armory and Bedroom become unlocked, while deeper rooms remain locked

#### Scenario: Capstone requires both parents

- **WHEN** only one of War Room's parents (Watchtower or Alchemy Lab) is built
- **THEN** War Room remains locked until both parents are built to at least Tier 1

#### Scenario: Storm Forge requires both capstones

- **WHEN** the player attempts to build Storm Forge
- **THEN** it is unlocked only after both War Room and Vault are built to at least Tier 1

### Requirement: Building And Upgrading With Prestige Ranks

Rooms SHALL be built and upgraded by spending prestige ranks. A build/upgrade SHALL only succeed when the room is unlocked, is not already at its maximum tier, and the player has at least the required cost; on success the tier SHALL increase by exactly 1 and the cost SHALL be deducted. A failed attempt (locked, maxed, or insufficient prestige ranks) SHALL change neither the room tier nor the prestige-rank balance. Most rooms SHALL have a maximum tier of 3; Fishing Dock SHALL have a maximum tier of 4; Storm Forge SHALL have a single tier (maximum tier 1).

#### Scenario: Successful build deducts cost and raises tier

- **WHEN** the player builds an unlocked, non-maxed room they can afford
- **THEN** the room's tier increases by 1 and its cost is subtracted from the prestige-rank balance

#### Scenario: Insufficient prestige ranks

- **WHEN** the player attempts to build a room whose cost exceeds their prestige-rank balance
- **THEN** the build fails, the tier is unchanged, and no prestige ranks are spent

#### Scenario: Cannot build a locked or maxed room

- **WHEN** the player attempts to build a room that is locked or already at its maximum tier
- **THEN** the build fails and no prestige ranks are spent

### Requirement: Tier Costs Scale With Tree Depth

The prestige-rank cost of each tier SHALL follow the room's depth in the tree, with special-cased costs for Fishing Dock's fourth tier and for Storm Forge. The costs SHALL be exactly:

| Room group (depth) | Rooms | T1 | T2 | T3 | T4 |
|--------------------|-------|----|----|----|----|
| Root (0) | Hearthstone | 1 | 2 | 3 | — |
| Branch (1) | Armory, Bedroom | 1 | 3 | 5 | — |
| Mid-tree (2–3) | Training Yard, Trophy Hall, Watchtower, Alchemy Lab, Garden, Library, Workshop | 2 | 4 | 6 | — |
| Capstone (4) | War Room, Vault | 3 | 5 | 7 | — |
| Fishing Dock | Fishing Dock | 2 | 4 | 6 | 10 |
| Storm Forge | Storm Forge | 25 | — | — | — |

#### Scenario: Root and capstone costs differ

- **WHEN** building Hearthstone Tier 1 versus War Room Tier 1
- **THEN** Hearthstone Tier 1 costs 1 prestige rank and War Room Tier 1 costs 3 prestige ranks

#### Scenario: Fishing Dock fourth tier is special

- **WHEN** upgrading Fishing Dock to Tier 4
- **THEN** the cost is exactly 10 prestige ranks

#### Scenario: Storm Forge single-tier cost

- **WHEN** building Storm Forge
- **THEN** it costs exactly 25 prestige ranks and cannot be upgraded further

### Requirement: Room Bonus Magnitudes

Each buildable room SHALL grant a passive bonus whose magnitude is fixed per tier. An unbuilt room (Tier 0) SHALL grant no bonus. The bonuses SHALL be exactly:

| Room | Bonus | T1 | T2 | T3 | T4 |
|------|-------|----|----|----|----|
| Hearthstone | Offline XP | +25% | +50% | +100% | — |
| Armory | Combat damage | +5% | +10% | +25% | — |
| Training Yard | XP gain | +5% | +10% | +30% | — |
| Trophy Hall | Mob drop rate | +5% | +10% | +15% | — |
| Watchtower | Crit chance | +5% | +10% | +20% | — |
| Alchemy Lab | HP regen | +25% | +50% | +100% | — |
| War Room | Double-strike chance | +10% | +20% | +35% | — |
| Bedroom | Regen-delay reduction | −15% | −30% | −50% | — |
| Garden | Fishing-timer reduction | −10% | −20% | −40% | — |
| Library | Challenge discovery rate | +20% | +30% | +50% | — |
| Fishing Dock | Double-fish chance (T1–T3); +10 max fishing rank (T4) | +25% | +50% | +100% | +10 rank |
| Workshop | Item rarity | +10% | +15% | +25% | — |
| Vault | Equipped items preserved on prestige | 1 | 3 | 5 | — |
| Storm Forge | Stormbreaker forging enabled | enabled | — | — | — |

#### Scenario: Bonus scales with tier

- **WHEN** Armory is at Tier 1, then Tier 2, then Tier 3
- **THEN** its combat damage bonus is +5%, then +10%, then +25%

#### Scenario: Unbuilt room grants nothing

- **WHEN** a room is at Tier 0 (unbuilt)
- **THEN** it contributes a bonus value of 0

#### Scenario: Fishing Dock fourth tier changes the bonus

- **WHEN** Fishing Dock reaches Tier 4
- **THEN** it grants +10 maximum fishing rank in addition to its Tier 3 double-fish chance, raising the fishing-rank cap from 30 to 40

### Requirement: Bonuses Supplied As Explicit Inputs

Haven bonuses SHALL be exposed as a computed set of values, recomputed from current room tiers, and SHALL be supplied to every consuming system as explicit inputs (parameters) rather than read from global state. No consuming system SHALL reach into Haven state directly to obtain its bonus. Each bonus type SHALL come from exactly one room, so bonuses of a given type do not stack across rooms.

#### Scenario: Consumer receives the bonus as a parameter

- **WHEN** a consuming system (such as item drops, combat, XP, offline progression, or fishing) needs a Haven bonus
- **THEN** the relevant bonus value is passed to it as an explicit input and the consumer does not query Haven state itself

#### Scenario: Bonuses reflect current tiers

- **WHEN** a room is upgraded to a higher tier
- **THEN** the recomputed bonus set reflects the new tier's magnitude for that bonus type

### Requirement: Bonus Application To Consuming Systems

The Haven bonuses SHALL be applied by their consuming systems as follows: Armory damage, Watchtower crit chance, and War Room double-strike chance SHALL affect combat; Training Yard SHALL increase combat kill XP; Hearthstone SHALL increase offline XP; Alchemy Lab and Bedroom SHALL affect HP regen amount and regen delay respectively; Garden and Fishing Dock SHALL affect fishing timers, double-fish chance, and (Tier 4) the fishing-rank cap; Library SHALL increase challenge discovery rate. Trophy Hall drop-rate and Workshop item-rarity bonuses SHALL apply only to mob drops and SHALL NOT affect boss drops.

#### Scenario: Combat consumes damage/crit/double-strike

- **WHEN** Armory, Watchtower, and War Room are built
- **THEN** the player's combat damage, crit chance, and double-strike chance are increased by those rooms' bonuses

#### Scenario: Drop bonuses apply to mobs only

- **WHEN** Trophy Hall and Workshop are built
- **THEN** mob drops receive the drop-rate and item-rarity bonuses
- **AND** boss drops use their fixed rates, unaffected by these Haven bonuses

### Requirement: Vault Preserves Equipment Across Prestige

The Vault SHALL let the player carry a limited number of equipped items through a prestige reset, with the number of preservable items equal to the Vault's tier bonus: 1 at Tier 1, 3 at Tier 2, and 5 at Tier 3. Items not selected for preservation SHALL be cleared by prestige as normal, and the count of preservable items SHALL never exceed the Vault's current tier bonus.

#### Scenario: Preserve up to the Vault limit

- **WHEN** the player prestiges with a Tier 1 Vault and selects one equipped item to keep
- **THEN** that item remains equipped after prestige while all other equipment is cleared

#### Scenario: Higher tiers preserve more

- **WHEN** the Vault is at Tier 3
- **THEN** up to 5 equipped items may be preserved through a prestige reset

### Requirement: Storm Forge Enables Stormbreaker Forging

Storm Forge SHALL be the ultimate Haven room: building it SHALL require both capstones (War Room and Vault) and cost 25 prestige ranks. Storm Forge SHALL enable forging the Stormbreaker weapon, but the forge itself SHALL only be usable once the player has caught the Storm Leviathan and reached Prestige Rank 25 or higher. Stormbreaker SHALL be the weapon required to defeat the Zone 10 final boss.

#### Scenario: Building Storm Forge

- **WHEN** both War Room and Vault are built and the player has at least 25 prestige ranks
- **THEN** Storm Forge can be built for 25 prestige ranks, enabling Stormbreaker forging

#### Scenario: Forging gated by Leviathan and prestige

- **WHEN** the player attempts to forge Stormbreaker
- **THEN** forging is allowed only if the Storm Leviathan has been caught and the player is at Prestige Rank 25 or higher
