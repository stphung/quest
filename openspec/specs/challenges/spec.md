# Challenge Minigames Specification

## Purpose

Challenge minigames are optional, player-controlled skill games that appear randomly during idle play once the hero has prestiged at least once. This capability defines the shared framework every minigame plugs into: how challenges are discovered and offered, the four common difficulty tiers, how a challenge is accepted or declined, how a victory pays out rewards, and the uniform forfeit-confirmation flow. Per-game rules (board sizes, win conditions, AI) are out of scope; this spec fixes only the contract common to all minigames.

## Requirements

### Requirement: Challenge Roster and Shared Framework

The system SHALL offer a fixed roster of exactly 14 challenge minigames — Chess, Morris, Gomoku, Minesweeper, Rune Deciphering, Go, Snake (Serpent's Path), Flappy Bird (Skyward Gauntlet), JezzBall (Containment Breach), Sigil Surge (Runic Shift), Sudoku (Sigil Matrix), Shard Fusion, Runic Lights, and Vault Warden. Every minigame in the roster SHALL conform to the same shared framework: it is discoverable through the same discovery gate, offers the same four difficulty tiers, is accepted or declined through the same challenge menu, pays out through the same reward structure, and uses the same forfeit-confirmation flow. Only one minigame SHALL be active at any given time.

#### Scenario: Roster size

- **WHEN** the set of selectable challenge types is enumerated
- **THEN** exactly 14 distinct minigames are available, each conforming to the shared framework

#### Scenario: Only one active minigame

- **WHEN** a minigame is already active
- **THEN** no second minigame can be started and no new challenge can be discovered until the active one ends

### Requirement: Challenge Discovery Gate

The system SHALL make challenges discoverable only when the character has reached Prestige Rank 1 or higher and is not currently in a dungeon, not fishing, and not already in an active minigame. When those conditions hold, the system SHALL roll once per game tick against a base per-tick discovery chance of 0.000014 (approximately one challenge every two hours of active play). While any gate condition fails, the discovery roll SHALL NOT occur and no challenge SHALL be offered.

#### Scenario: Below the prestige gate

- **WHEN** a character at Prestige Rank 0 plays for any length of time
- **THEN** no challenge is ever discovered

#### Scenario: Blocked by a competing activity

- **WHEN** the character is in a dungeon, fishing, or already in an active minigame
- **THEN** the per-tick discovery roll is skipped and no challenge is offered

#### Scenario: Eligible character discovers over time

- **WHEN** a character at Prestige Rank 1 or higher is idle-battling and not otherwise occupied
- **THEN** each tick rolls against the base 0.000014 chance and a challenge may be added to the challenge menu

### Requirement: Haven Discovery Boost

The system SHALL allow an account-level Haven discovery bonus to increase the effective challenge discovery chance. The effective per-tick chance SHALL equal the base chance multiplied by (1 + bonus_percent / 100). A bonus of 0 SHALL leave the base chance unchanged, and a positive bonus SHALL raise the discovery chance proportionally.

#### Scenario: No Haven bonus

- **WHEN** the Haven discovery bonus is 0 percent
- **THEN** the effective discovery chance equals the base 0.000014

#### Scenario: Positive Haven bonus raises the rate

- **WHEN** the Haven discovery bonus is greater than 0 percent
- **THEN** the effective discovery chance is the base chance scaled up by (1 + bonus_percent / 100), producing more discoveries over time than with no bonus

### Requirement: Weighted Selection and Deduplication

When a discovery roll succeeds, the system SHALL choose which minigame to offer using a weighted distribution across the roster, so that faster puzzles appear more often and longer strategy games appear more rarely. The system SHALL exclude any challenge type already pending in the challenge menu from the selection pool, and SHALL add exactly one pending challenge of the selected type. If every roster type is already pending, no new challenge SHALL be added.

#### Scenario: Weighted pick added to the menu

- **WHEN** a discovery roll succeeds and at least one roster type is not already pending
- **THEN** exactly one challenge of a type chosen by relative weight is added to the challenge menu

#### Scenario: No duplicate pending challenge

- **WHEN** a challenge type is already pending in the menu
- **THEN** that type is excluded from selection so the menu never holds two pending challenges of the same type

#### Scenario: Full menu adds nothing

- **WHEN** every roster type is already pending
- **THEN** the successful roll adds no new challenge

### Requirement: Four Difficulty Tiers

The system SHALL offer every minigame at exactly four difficulty tiers, in ascending order: Novice, Apprentice, Journeyman, and Master. When a pending challenge is accepted, the player SHALL select one of these four tiers, and the started game SHALL use the selected tier. Higher tiers SHALL yield larger victory rewards than lower tiers of the same minigame.

#### Scenario: Tier selection on accept

- **WHEN** a player opens a pending challenge's detail view and chooses a difficulty
- **THEN** the selectable options are exactly Novice, Apprentice, Journeyman, and Master, and the game starts at the chosen tier

#### Scenario: Reward increases with tier

- **WHEN** the same minigame is won at a higher tier versus a lower tier
- **THEN** the higher tier awards a greater reward

### Requirement: Accept and Decline Pending Challenges

The system SHALL hold discovered challenges as pending entries in a challenge menu until the player acts on each. Accepting a pending challenge SHALL remove it from the menu and start the corresponding minigame at the selected difficulty as the single active minigame. Declining a pending challenge SHALL remove it from the menu without starting a game and without granting any reward.

#### Scenario: Accepting starts the game

- **WHEN** the player accepts a pending challenge at a chosen difficulty
- **THEN** that challenge is removed from the menu and the corresponding minigame becomes the active minigame at that difficulty

#### Scenario: Declining discards the challenge

- **WHEN** the player declines a pending challenge
- **THEN** it is removed from the menu, no minigame starts, and no reward is granted

### Requirement: Victory Reward Payout

The system SHALL grant a reward only when the player wins the active minigame, applying the reward defined for that minigame's chosen difficulty. A reward SHALL be composed of Prestige Ranks, a Stormglass currency amount, and Fishing Ranks, any of which may be zero. Stormglass SHALL be paid as Stormglass currency when Stormglass has been discovered; when it has not, the Stormglass amount SHALL instead be converted to experience equal to (Stormglass / 10) percent of the experience required for the character's next level. Fishing Rank rewards SHALL be capped at the maximum fishing rank and SHALL grant nothing when the character is already at that cap. A loss, draw, or forfeit SHALL grant no reward of any kind.

#### Scenario: Win grants the difficulty's reward

- **WHEN** the player wins a minigame at a given difficulty
- **THEN** the Prestige Ranks, Stormglass, and Fishing Ranks defined for that difficulty are added to the character

#### Scenario: Stormglass falls back to experience before discovery

- **WHEN** a winning reward includes Stormglass but Stormglass has not yet been discovered
- **THEN** no Stormglass is granted and the character instead gains experience equal to (Stormglass / 10) percent of the next level's requirement

#### Scenario: Fishing rank reward respects the cap

- **WHEN** a winning reward includes Fishing Ranks and the character is already at the maximum fishing rank
- **THEN** the fishing rank is not increased beyond the maximum

#### Scenario: Non-win grants nothing

- **WHEN** the active minigame ends in a loss, draw, or forfeit
- **THEN** Prestige Ranks, Stormglass, experience, and Fishing Ranks are all unchanged

### Requirement: Forfeit Confirmation Flow

The system SHALL use a two-step forfeit confirmation shared by all minigames. The first forfeit input (Esc) SHALL set a pending-forfeit state without ending the game. A second consecutive forfeit input SHALL confirm the forfeit and resolve the game as a loss. Any other input while forfeit is pending SHALL cancel the pending-forfeit state and return to normal play. A confirmed forfeit SHALL be treated as a loss and therefore SHALL grant no reward.

#### Scenario: First forfeit input arms confirmation

- **WHEN** the player presses the forfeit key once during a game
- **THEN** the game enters a pending-forfeit state and continues without a result

#### Scenario: Second forfeit input confirms the loss

- **WHEN** the player presses the forfeit key again while forfeit is pending
- **THEN** the game resolves as a loss and no reward is granted

#### Scenario: Any other input cancels the pending forfeit

- **WHEN** the player presses any non-forfeit input while forfeit is pending
- **THEN** the pending-forfeit state is cleared and the game continues normally

### Requirement: Achievement Tracking on Win

The system SHALL, upon a minigame victory, emit a win record identifying the minigame type and the difficulty tier that was won, for consumption by the achievement system. A loss, draw, or forfeit SHALL emit no such win record.

#### Scenario: Win emits a tracking record

- **WHEN** the player wins a minigame at a specific difficulty
- **THEN** a win record carrying that minigame's type and difficulty is emitted for achievement tracking

#### Scenario: Non-win emits no record

- **WHEN** the minigame ends in a loss, draw, or forfeit
- **THEN** no win record is emitted
