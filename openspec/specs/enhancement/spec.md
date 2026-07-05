# Soulforge Equipment Enhancement Specification

## Purpose

Defines the Soulforge, an account-wide system that lets players spend Prestige Rank to enhance each equipment slot from +0 to +10, multiplying that slot's equipped item stats. Low levels are cheap and guaranteed, while high levels become an escalating gamble that can destroy progress on failure. Players may instead pay a much larger Soul Tithe for a guaranteed success. The Soulforge is a hidden endgame system discovered only after reaching high Prestige Rank. Prestige Rank (the currency) and equipment items (the enhancement targets) are inputs owned by other capabilities.

## Requirements

### Requirement: Enhance Each Equipment Slot Independently

The system SHALL maintain an account-wide enhancement level, an integer from 0 to a hard maximum of 10, for each of the seven equipment slots. Enhancement levels SHALL be shared across all of the account's characters, SHALL be tracked independently per slot, and SHALL persist for a slot even when no item is currently equipped in it. An enhancement level SHALL never be set above 10.

#### Scenario: Slots advance independently

- **WHEN** one equipment slot is enhanced
- **THEN** only that slot's level changes and the other six slots keep their existing levels

#### Scenario: Level is capped at the maximum

- **WHEN** a slot is already at level 10 and an enhancement is attempted on it
- **THEN** the level stays at 10 and no further increase occurs

### Requirement: Charge Prestige Rank For Every Enhancement Attempt

The system SHALL require the player to be able to afford an attempt before it begins, and SHALL deduct the attempt's Prestige Rank cost once the attempt resolves, whether the attempt succeeds or fails. A standard (non-tithe) attempt to reach a target level SHALL cost the following Prestige Rank amounts:

| Target level | Standard cost (PR) |
|--------------|--------------------|
| +1 | 1 |
| +2 | 1 |
| +3 | 1 |
| +4 | 1 |
| +5 | 2 |
| +6 | 3 |
| +7 | 3 |
| +8 | 4 |
| +9 | 4 |
| +10 | 5 |

#### Scenario: Cost is spent on success and on failure

- **WHEN** a standard attempt to reach a target level resolves, regardless of whether it succeeded or failed
- **THEN** the standard Prestige Rank cost for that target level is subtracted from the player's Prestige Rank

#### Scenario: Attempt blocked when unaffordable

- **WHEN** the player's Prestige Rank is less than the cost of enhancing a slot to its next level
- **THEN** the enhancement for that slot cannot be started

### Requirement: Roll Standard Enhancement Success By Target Level

The system SHALL determine the outcome of a standard enhancement attempt by rolling against a fixed success probability that depends only on the target level. Target levels +1 through +4 SHALL always succeed. The success probabilities SHALL be:

| Target level | Success probability |
|--------------|---------------------|
| +1 | 100% |
| +2 | 100% |
| +3 | 100% |
| +4 | 100% |
| +5 | 70% |
| +6 | 55% |
| +7 | 40% |
| +8 | 30% |
| +9 | 20% |
| +10 | 10% |

#### Scenario: Low levels are guaranteed

- **WHEN** a standard attempt targets any level from +1 through +4
- **THEN** the attempt always succeeds

#### Scenario: High levels roll against the published rate

- **WHEN** a standard attempt targets +10
- **THEN** it succeeds with 10% probability and otherwise fails

### Requirement: Apply Success And Failure Outcomes

On a successful attempt the system SHALL raise the slot's enhancement level by exactly one. On a failed standard attempt the system SHALL lower the slot's enhancement level by that target level's failure penalty, never going below 0. Target levels +1 through +4 SHALL have a failure penalty of 0, target levels +5 through +9 SHALL have a failure penalty of 1, and target level +10 SHALL have a failure penalty of 2.

| Target level | Failure penalty (levels lost) |
|--------------|-------------------------------|
| +1 to +4 | 0 |
| +5 to +9 | 1 |
| +10 | 2 |

#### Scenario: Success raises the level by one

- **WHEN** a standard attempt to reach +6 succeeds
- **THEN** the slot's enhancement level becomes +6

#### Scenario: Failure at a mid tier drops one level

- **WHEN** a standard attempt to reach +5 (from +4) fails
- **THEN** the slot's enhancement level drops by one to +3 and the attempt's Prestige Rank cost is still consumed

#### Scenario: Failure at the top tier drops two levels

- **WHEN** a standard attempt to reach +10 (from +9) fails
- **THEN** the slot's enhancement level drops by two to +7 and the attempt's Prestige Rank cost is still consumed

### Requirement: Offer Soul Tithe For Guaranteed Success

For target levels +5 through +10 the system SHALL offer a Soul Tithe alternative that pays a higher fixed Prestige Rank cost in exchange for guaranteed success with no risk of failure or level loss. The Soul Tithe SHALL NOT be available for target levels +1 through +4, which are already guaranteed. The Soul Tithe costs SHALL be:

| Target level | Soul Tithe cost (PR) |
|--------------|----------------------|
| +5 | 4 |
| +6 | 6 |
| +7 | 8 |
| +8 | 25 |
| +9 | 85 |
| +10 | 750 |

#### Scenario: Soul Tithe guarantees the upgrade

- **WHEN** the player pays the Soul Tithe cost to reach +8
- **THEN** 25 Prestige Rank is spent and the slot reaches +8 with certainty, with no chance of failure or downgrade

#### Scenario: Soul Tithe is unavailable at guaranteed tiers

- **WHEN** the target level is +1 through +4
- **THEN** no Soul Tithe option is offered

### Requirement: Scale Equipped Item Stats By Enhancement Multiplier

The system SHALL scale the attribute bonuses and affix values of the item equipped in a slot by a multiplier derived from that slot's enhancement level. The multiplier SHALL equal `1 + cumulative_bonus/100`, using the cumulative bonus schedule below, so level 0 applies no change (1.00x) and level 10 applies 2.50x. The multiplier SHALL apply only to the item currently equipped in the enhanced slot.

| Level | Cumulative bonus | Multiplier |
|-------|------------------|------------|
| +0 | 0% | 1.00x |
| +1 | 5% | 1.05x |
| +2 | 10% | 1.10x |
| +3 | 15% | 1.15x |
| +4 | 20% | 1.20x |
| +5 | 30% | 1.30x |
| +6 | 40% | 1.40x |
| +7 | 55% | 1.55x |
| +8 | 75% | 1.75x |
| +9 | 100% | 2.00x |
| +10 | 150% | 2.50x |

#### Scenario: Maxed slot multiplies item stats by 2.5

- **WHEN** a slot is at +10 and holds an equipped item
- **THEN** that item's attribute bonuses and affix values contribute 2.5x their base amount

#### Scenario: Unenhanced slot leaves stats unchanged

- **WHEN** a slot is at +0
- **THEN** the equipped item's attribute bonuses and affix values contribute their base amount unchanged (1.0x)

### Requirement: Discover The Soulforge At High Prestige Rank

The system SHALL keep the Soulforge hidden until it is discovered, and SHALL only roll for discovery once the player reaches Prestige Rank 15 or higher (the probability is zero below that rank). On each eligible tick the discovery probability SHALL equal `0.000014 + (prestige_rank − 15) × 0.000007`. The discovery roll SHALL be skipped while a dungeon, fishing session, or challenge minigame is active. Once discovered, the Soulforge SHALL remain discovered permanently for the account.

#### Scenario: No discovery below the prestige threshold

- **WHEN** the player's Prestige Rank is below 15
- **THEN** the Soulforge discovery probability is zero and it cannot be discovered

#### Scenario: Discovery chance rises with prestige rank

- **WHEN** the player is at Prestige Rank 15 with no active dungeon, fishing session, or minigame
- **THEN** the per-tick discovery probability is 0.000014, rising by 0.000007 for each rank above 15

#### Scenario: Discovery is not rolled during active content

- **WHEN** a dungeon, fishing session, or challenge minigame is active
- **THEN** no Soulforge discovery roll is made that tick

#### Scenario: Discovery persists

- **WHEN** the Soulforge has already been discovered
- **THEN** it remains discovered and is never re-hidden

### Requirement: Track Enhancement Attempt Statistics

The system SHALL record cumulative enhancement statistics across the account: the total number of attempts, the total successes, the total failures, and the highest enhancement level ever reached on any slot. Each resolved attempt SHALL increment the total attempts, and SHALL increment either the successes or the failures according to its outcome.

#### Scenario: Counters update on each attempt

- **WHEN** an enhancement attempt resolves as a success
- **THEN** total attempts and total successes each increase by one and total failures is unchanged

#### Scenario: Highest level reached is retained

- **WHEN** a slot reaches a new highest enhancement level
- **THEN** the recorded highest level reached is updated and never decreases afterward, even if the slot later loses levels to failures
