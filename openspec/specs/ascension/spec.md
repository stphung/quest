# Ascension Specification

## Purpose

Ascension is a per-character prestige sink that trades Prestige Rank for a permanent combat power multiplier across ten tiers (I–X). Each tier multiplies the player's outgoing damage, defensive value, and maximum HP by a single shared factor. Tiers I–VI are gated by progress into the Deep, while the endgame tiers VII–X are additionally gated by completed Woven Patterns. This capability defines the tier ceiling, the cost table, the multiplier curve, the eligibility gates, how the multiplier enters combat, and how the earned level persists.

## Requirements

### Requirement: Ascension Level Range and Sequential Progression

The system SHALL track a per-character Ascension level that starts at 0 (no ascension) and never exceeds a maximum of 10. Ascension SHALL be purchased one tier at a time — a purchase always targets the level immediately above the current one — and once level 10 is reached no further ascension is possible. At level 0 the combat multiplier SHALL be exactly 1.0x (no effect).

#### Scenario: New character starts unascended

- **WHEN** a character is first created
- **THEN** its Ascension level is 0 and its Ascension combat multiplier is 1.0x

#### Scenario: Purchase advances exactly one tier

- **WHEN** a character at Ascension level N (N < 10) successfully ascends
- **THEN** the Ascension level becomes N + 1, not any higher tier in a single action

#### Scenario: Maximum tier blocks further ascension

- **WHEN** a character already at Ascension level 10 attempts to ascend
- **THEN** the attempt is rejected as already at the maximum level and the level is unchanged

### Requirement: Prestige Rank Cost Table

The system SHALL charge Prestige Rank to ascend, deducting the full cost of the target tier on success. The cost to reach each tier SHALL be exactly: I = 35 PR, II = 65 PR, III = 120 PR, IV = 200 PR, V = 325 PR, VI = 500 PR, VII = 1,500 PR, VIII = 4,000 PR, IX = 8,000 PR, X = 15,000 PR. Each cost is charged independently per tier (not cumulatively), so reaching Ascension VI from scratch costs 1,245 PR in total across the six purchases.

#### Scenario: Cost is deducted on a successful ascension

- **WHEN** a character with sufficient Prestige Rank ascends from level 2 to level 3
- **THEN** exactly 120 Prestige Rank is deducted and the level becomes 3

#### Scenario: Endgame tier costs

- **WHEN** a character ascends into the Loom-gated tiers
- **THEN** the costs are 1,500 PR for VII, 4,000 PR for VIII, 8,000 PR for IX, and 15,000 PR for X

#### Scenario: Insufficient Prestige Rank blocks the purchase

- **WHEN** a character's current Prestige Rank is less than the target tier's cost
- **THEN** the ascension is rejected for insufficient Prestige Rank, no Prestige Rank is spent, and the level is unchanged

### Requirement: Ascension Combat Multiplier Curve

The system SHALL derive a single combat multiplier from the Ascension level. For levels 1 through 6 the multiplier SHALL be 2 raised to the level (2x, 4x, 8x, 16x, 32x, 64x). For levels 7 and above the multiplier SHALL be 64 times 1.5 raised to (level − 6), yielding 96x at VII, 144x at VIII, 216x at IX, and 324x at X. Level 0 SHALL yield 1.0x.

#### Scenario: Doubling tiers I through VI

- **WHEN** the Ascension level is 1, 2, 3, 4, 5, or 6
- **THEN** the multiplier is 2x, 4x, 8x, 16x, 32x, or 64x respectively

#### Scenario: Diminishing endgame tiers VII through X

- **WHEN** the Ascension level is 7, 8, 9, or 10
- **THEN** the multiplier is 96x, 144x, 216x, or 324x respectively

### Requirement: Deep Layer Gates for Tiers I–VI

The system SHALL require a minimum reached Deep layer to ascend into tiers I through VI. The required deepest layer SHALL be exactly: I → layer 3, II → layer 7, III → layer 12, IV → layer 18, V → layer 25, VI → layer 30. Tiers VII and above SHALL have no Deep layer gate.

#### Scenario: Deep gate not yet met

- **WHEN** a character with enough Prestige Rank for Ascension IV has only reached Deep layer 15
- **THEN** the ascension is rejected because the required layer 18 has not been reached, and no Prestige Rank is spent

#### Scenario: Deep gate satisfied

- **WHEN** a character has reached at least the required Deep layer for the target tier and meets its Prestige Rank cost
- **THEN** the Deep gate does not block the ascension

#### Scenario: Endgame tiers ignore the Deep gate

- **WHEN** a character ascends into tier VII or higher
- **THEN** no Deep layer requirement is checked

### Requirement: Woven Pattern Gates for Tiers VII–X

The system SHALL require a minimum count of completed Woven Patterns to ascend into the endgame tiers VII through X. The required completed pattern count SHALL be exactly: VII → 8 patterns, VIII → 16 patterns, IX → 22 patterns, X → 28 patterns. Tiers I through VI SHALL have no Woven Pattern gate.

#### Scenario: Pattern gate not yet met

- **WHEN** a character with enough Prestige Rank for Ascension X has completed only 27 Woven Patterns
- **THEN** the ascension is rejected because 28 completed patterns are required, and no Prestige Rank is spent

#### Scenario: Pattern gate satisfied

- **WHEN** a character has completed at least the required number of Woven Patterns for the target endgame tier and meets its Prestige Rank cost
- **THEN** the Woven Pattern gate does not block the ascension

#### Scenario: Early tiers ignore the pattern gate

- **WHEN** a character ascends into any tier from I through VI
- **THEN** no completed Woven Pattern count is required

### Requirement: Eligibility Evaluation and Failure Reporting

The system SHALL determine ascension eligibility by checking, in order: whether the maximum level is already reached, whether current Prestige Rank covers the target tier's cost, whether the Deep layer gate (tiers I–VI) is satisfied, and whether the Woven Pattern gate (tiers VII–X) is satisfied. A rejected attempt SHALL leave both the Ascension level and the Prestige Rank balance unchanged and SHALL report the specific reason for rejection (maximum reached, insufficient Prestige Rank, Deep gate unmet, or Woven Pattern gate unmet). Prestige Rank is only deducted when all applicable checks pass.

#### Scenario: All conditions satisfied yields success

- **WHEN** the target tier is at or below level 10, the Prestige Rank cost is affordable, and every applicable gate is met
- **THEN** the Prestige Rank cost is deducted, the Ascension level advances by one, and the resulting multiplier is reported

#### Scenario: Prestige Rank is checked before the gates

- **WHEN** a character lacks the Prestige Rank cost and also fails a gate
- **THEN** the rejection reported is insufficient Prestige Rank

#### Scenario: No partial state change on rejection

- **WHEN** any eligibility check fails
- **THEN** neither the Ascension level nor the Prestige Rank balance is modified

### Requirement: Multiplier Application to Damage, Defense, and HP

The system SHALL apply the Ascension combat multiplier to three combat quantities using the same factor. In the player-to-enemy damage pipeline it SHALL multiply the damage after the prestige flat-damage addition and before the enemy's defense is subtracted (and before any critical multiplier). In the enemy-to-player defense pipeline it SHALL multiply the player's combined base-plus-flat defense before that total defense is subtracted from the enemy's damage. It SHALL also multiply the player's maximum HP. A multiplier of 1.0 (Ascension level 0) SHALL leave all three quantities unchanged.

#### Scenario: Damage is scaled before enemy defense

- **WHEN** a player attack resolves at an Ascension multiplier greater than 1.0
- **THEN** the multiplier is applied to the post-flat-damage value before the enemy's defense is subtracted and before any critical hit multiplier

#### Scenario: Defense is scaled before subtraction

- **WHEN** an enemy attack resolves at an Ascension multiplier greater than 1.0
- **THEN** the player's combined base and flat defense is multiplied by the Ascension multiplier before being subtracted from the enemy's damage

#### Scenario: Maximum HP is scaled

- **WHEN** the Ascension multiplier is greater than 1.0
- **THEN** the player's maximum HP is multiplied by that same factor

#### Scenario: Unascended character is unaffected

- **WHEN** the Ascension level is 0 (multiplier 1.0x)
- **THEN** damage, defense, and maximum HP are unchanged by the Ascension system

### Requirement: Ascension Persists Across Prestige

The system SHALL store the Ascension level as per-character save data that survives prestige. A prestige reset SHALL NOT reduce the earned Ascension level, even though the Prestige Rank previously spent on ascension is not refunded. Save data that predates the Ascension system SHALL load with an Ascension level of 0.

#### Scenario: Level survives a prestige reset

- **WHEN** a character at Ascension level 5 performs a prestige
- **THEN** the character remains at Ascension level 5 after the reset while its spent Prestige Rank is not returned

#### Scenario: Legacy save defaults to unascended

- **WHEN** a save file that omits the Ascension level is loaded
- **THEN** the character's Ascension level defaults to 0

### Requirement: Ascension Tier Raises the Loom Shuttle Level Cap

The system SHALL expose a maximum Loom shuttle upgrade level determined by the Ascension tier. For Ascension levels 0 through VI the cap SHALL be 1; for VII it SHALL be 3, for VIII it SHALL be 5, for IX it SHALL be 7, and for X it SHALL be 10.

#### Scenario: Pre-VII tiers cap shuttle level at one

- **WHEN** a character is at Ascension level 0 through 6
- **THEN** the maximum allowed Loom shuttle upgrade level is 1

#### Scenario: Endgame tiers raise the shuttle cap

- **WHEN** a character reaches Ascension VII, VIII, IX, or X
- **THEN** the maximum allowed Loom shuttle upgrade level is 3, 5, 7, or 10 respectively
