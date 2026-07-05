# Character Progression & Prestige Specification

## Purpose

Defines how a hero grows over a single run and across prestige cycles: the six RPG attributes and their per-modifier effects, the XP-to-level curve fed only by combat kills, the random attribute growth on each level-up, and the prestige loop that trades a fully reset run for a permanent, monotonically increasing Prestige Rank. Prestige Rank is the master account-level currency: it raises attribute caps, grants flat combat bonuses, multiplies XP, and gates nearly every endgame system. This capability owns the numbers behind that growth; it references combat, Ascension, Haven, and the Deep only as systems fed or gated by Prestige Rank.

## Requirements

### Requirement: Character Attributes And Modifiers

The system SHALL model every character with exactly six attributes — Strength (STR), Dexterity (DEX), Constitution (CON), Intelligence (INT), Wisdom (WIS), and Charisma (CHA) — each stored as a non-negative integer that starts at a base value of 10 on a fresh character. Each attribute contributes to gameplay through its modifier, computed as `(value − 10) / 2` using integer division that truncates toward zero (so values 10–11 give +0, 12–13 give +1, 8–9 give −1). Modifiers feed derived stats: STR adds +2 physical damage per point, INT adds +2 magic damage per point, CON adds +10 maximum HP per point, DEX adds +1 defense and +1% critical-hit chance per point, WIS adds +5% XP gain per point, and CHA adds +10% (0.1) to the prestige XP multiplier per point.

#### Scenario: Fresh character attribute baseline

- **WHEN** a new character is created
- **THEN** all six attributes are 10 and every attribute modifier is +0

#### Scenario: Modifier truncates toward zero

- **WHEN** an attribute value is 7
- **THEN** its modifier is −1 (because `(7 − 10) / 2` truncates to −1), and a value of 9 yields a modifier of +0

### Requirement: Level-Up Grants Random Attribute Points Within Caps

The system SHALL grant exactly 3 attribute points on each level-up, distributing them one at a time to randomly chosen attributes among the six, and SHALL skip any attribute already at its cap and retry another attribute. The attribute cap SHALL equal `20 + 5 × prestige_rank`, so a Prestige Rank 0 character caps every attribute at 20 and each additional Prestige Rank raises the cap by 5. If all six attributes are already at the cap, fewer than 3 points (down to zero) are distributed. Derived stats are recalculated after level-ups so the new attributes take effect.

#### Scenario: Standard level-up distribution

- **WHEN** a character with attributes below the cap gains a level
- **THEN** exactly 3 points are added across the six attributes, none exceeding the cap of `20 + 5 × prestige_rank`

#### Scenario: Cap scales with prestige rank

- **WHEN** the character's Prestige Rank is 10
- **THEN** each attribute's cap is 70 (`20 + 5 × 10`)

#### Scenario: All attributes already capped

- **WHEN** a level-up occurs while all six attributes sit at the cap
- **THEN** no attribute points are distributed and all attributes remain at the cap

### Requirement: XP-To-Level Curve

The system SHALL require `floor(100 × level^1.5)` XP to advance from a given level to the next (base 100, exponent 1.5), yielding 100 XP for level 1→2, 282 for level 2→3, 3162 for level 10→11, and 100,000 for level 100→101. The curve SHALL be strictly increasing with level. Accumulated XP SHALL be applied in a loop: while current XP meets or exceeds the requirement for the next level, the requirement is subtracted, the level increments, and attribute points are distributed — so a single large XP award can produce multiple level-ups and any surplus XP carries over toward the following level.

#### Scenario: XP requirement at a given level

- **WHEN** a character is level 1
- **THEN** 100 XP is required to reach level 2, and reaching level 3 additionally requires 282 XP

#### Scenario: Multiple level-ups from one award

- **WHEN** a level-1 character with 0 XP receives 400 XP at once
- **THEN** it advances to level 3 (spending 100 then 282), gains 3 attribute points per level gained, and retains the leftover 18 XP toward level 4

### Requirement: XP Is Earned Only From Kills And Scaled By Prestige, WIS, And CHA

The system SHALL award character XP only from defeating enemies; no other activity grants character XP. Each kill SHALL award XP equal to the passive per-tick XP rate multiplied by a random 200–400 (inclusive) tick count. The passive per-tick XP rate SHALL be `1.0 × prestige_multiplier × (1 + WIS_modifier × 0.05)`, where `prestige_multiplier = (1 + 0.5 × prestige_rank^0.7) + CHA_modifier × 0.1`. At Prestige Rank 0 with neutral WIS and CHA the per-tick rate is 1.0, so a kill yields 200–400 XP.

#### Scenario: Baseline kill XP

- **WHEN** a Prestige Rank 0 character with +0 WIS and +0 CHA defeats an enemy
- **THEN** the kill awards between 200 and 400 XP

#### Scenario: Multipliers stack multiplicatively

- **WHEN** a character has Prestige Rank 1 (base multiplier 1.5), +5 WIS modifier, and +3 CHA modifier
- **THEN** the prestige multiplier is 1.8 (`1.5 + 3 × 0.1`) and the per-tick XP rate is 2.25 (`1.0 × 1.8 × 1.25`), scaling every kill accordingly

### Requirement: Offline Progression Awards Reduced Kill-Based XP

The system SHALL grant offline XP on return by simulating kills over the elapsed time at a reduced rate. Elapsed time SHALL be capped at 7 days (604,800 seconds). The system SHALL estimate one kill per 5 seconds, apply a 25% offline multiplier, and value each kill at the average of the 200–400 range (300 ticks) times the same prestige/WIS/CHA-scaled per-tick rate used online, then multiply by any offline-XP account bonus. Offline XP feeds the same level-up loop as online XP.

#### Scenario: Offline XP is capped at seven days

- **WHEN** a character has been away for 14 days
- **THEN** offline XP is computed as if only 7 days elapsed

#### Scenario: Offline rate is one quarter of online

- **WHEN** offline XP is computed for a duration with no account bonus
- **THEN** the effective rate is 25% of the equivalent online kill rate

### Requirement: Prestige Requires Meeting The Next Tier's Level Threshold

The system SHALL allow a prestige only when the character's level is at least the required level of the next Prestige Rank's tier. Required levels SHALL be: rank 1 = 10, 2 = 25, 3 = 50, 4 = 65, 5 = 80, 6 = 90, 7 = 100, 8 = 110, 9 = 120, 10 = 130, 11 = 140, 12 = 150, 13 = 160, 14 = 170, 15 = 180, 16 = 190, 17 = 200, 18 = 210, 19 = 220, and for ranks 20 and above `220 + (rank − 19) × 15`. Each Prestige Rank SHALL carry a display name — 1 Bronze, 2 Silver, 3 Gold, 4 Platinum, 5 Diamond, 6 Emerald, 7 Sapphire, 8 Ruby, 9 Obsidian, 10 Celestial, 11 Astral, 12 Cosmic, 13 Stellar, 14 Galactic, 15 Transcendent, 16 Divine, 17 Exalted, 18 Mythic, 19 Legendary, and 20+ Eternal. Attempting to prestige while below the threshold SHALL have no effect.

#### Scenario: Eligible to prestige into rank 1

- **WHEN** a Prestige Rank 0 character reaches level 10
- **THEN** it becomes eligible to prestige into rank 1 (Bronze)

#### Scenario: Below threshold is a no-op

- **WHEN** a prestige is attempted while the character's level is below the next tier's required level
- **THEN** the character state is unchanged and no rank is gained

### Requirement: Prestige Resets Run Progress

The system SHALL, on a successful prestige, reset the current run: character level to 1, XP to 0, all attributes to their base value of 10, equipment to a complete wipe, any active dungeon cleared, any active fishing session and active minigame cleared, combat state reset to a fresh 50 base HP, and zone progression reset to Zone 1 / subzone 1 with kill counters and defeated-boss records cleared. After reset the set of unlocked zones (up to Zone 30) SHALL be recomputed from the new, higher Prestige Rank so newly-qualified zones become available. A vault-preserving prestige variant MAY instead retain a limited set of equipment slots (gated elsewhere by vault tier) while performing the same run reset.

#### Scenario: Run state wiped on prestige

- **WHEN** a level-130 character at Prestige Rank 9 prestiges
- **THEN** its level returns to 1, XP to 0, all attributes to 10, equipment is emptied, and zone progression restarts at Zone 1 subzone 1

#### Scenario: Zone unlocks recomputed for new rank

- **WHEN** the prestige raises the character to a rank meeting a higher zone's prestige requirement
- **THEN** that zone (id ≤ 30) is present in the unlocked set after the reset

### Requirement: Prestige Preserves And Advances Account Progression

The system SHALL, on a successful prestige, increment Prestige Rank by 1 and increment the lifetime prestige count by 1, while preserving account-level progression that lives outside the run: achievements, Haven, fishing rank and lifetime fishing progression, Ascension level, Stormglass currency and Storm Sigils, the Deep, the Loom, Soulforge enhancement, character identity, and accumulated play time. The lifetime prestige count SHALL track only manual prestige actions.

#### Scenario: Rank and count advance while account state persists

- **WHEN** a character prestiges from Prestige Rank 4
- **THEN** its Prestige Rank becomes 5, its lifetime prestige count increases by 1, and its Ascension level, fishing rank, Haven, and achievements are unchanged

### Requirement: Prestige Rank Grants Flat Combat Bonuses

The system SHALL derive four flat combat bonuses from Prestige Rank that are independent of attributes and equipment: flat damage `floor(5.0 × rank^0.7)`, flat defense `floor(3.0 × rank^0.6)`, bonus critical-hit chance `min(rank × 0.5, 15.0)` percent, and flat HP `floor(15.0 × rank^0.6)`. At Prestige Rank 0 all four bonuses SHALL be zero. These bonuses feed the combat pipeline — flat damage added after percentage multipliers and before enemy defense, flat defense and bonus crit added to the DEX-derived values, and flat HP added to combat maximum HP.

#### Scenario: Bonuses at rank 10

- **WHEN** Prestige Rank is 10
- **THEN** flat damage is 25, flat defense is 11, bonus crit chance is 5%, and flat HP is 59

#### Scenario: Crit bonus caps at 15 percent

- **WHEN** Prestige Rank is 30 or higher
- **THEN** the bonus critical-hit chance is capped at 15%

#### Scenario: No bonuses before first prestige

- **WHEN** Prestige Rank is 0
- **THEN** all four flat combat bonuses are zero

### Requirement: Prestige Rank Increases Monotonically From Multiple Sources

The system SHALL only ever increase Prestige Rank during gameplay; no gameplay action SHALL decrease it. Prestige Rank SHALL be earned from multiple sources: the manual prestige action (+1 per prestige), winning challenge minigames (awarding 1–5 ranks by difficulty), passive Power Core generation, and the Loom's Woven-Reality-to-Prestige-Rank conversion. Passive grants SHALL accrue over both online ticks and offline time and add to the same Prestige Rank total.

#### Scenario: Challenge win grants ranks

- **WHEN** a character wins a Master-difficulty challenge
- **THEN** its Prestige Rank increases by the challenge's rank reward without any run reset

#### Scenario: Rank never regresses

- **WHEN** any combination of prestige, challenge, Power Core, or Loom rewards is applied
- **THEN** the resulting Prestige Rank is greater than or equal to its previous value

### Requirement: Prestige Rank Gates Endgame Systems And Multiplies XP

The system SHALL use Prestige Rank as the primary gate for progression and endgame systems: challenge discovery requires Prestige Rank 1+, Haven discovery requires 10+, Soulforge, the Deep, and Stormglass require 15+, the Stormbreaker weapon requires 25, and individual zones unlock at their own prestige requirements. Prestige Rank SHALL also raise the attribute cap (`20 + 5 × rank`), improve mob item-drop chance (+1% per rank over a 15% base, capped at 25%), and multiply XP gain via the prestige multiplier `1 + 0.5 × rank^0.7` (P1 = 1.5×, P5 ≈ 2.585×, P10 ≈ 3.507×, P20 ≈ 5.075×).

#### Scenario: System gated below its rank requirement

- **WHEN** a character is Prestige Rank 9
- **THEN** Haven discovery (requires 10+) cannot occur yet

#### Scenario: XP multiplier grows with rank

- **WHEN** Prestige Rank rises from 0 to 1
- **THEN** the base XP multiplier rises from 1.0× to 1.5×

### Requirement: Character Identity And Naming

The system SHALL assign each character a stable unique identifier and a player-chosen display name, persisting both across every prestige. A valid name SHALL be 1–16 characters using letters, digits, spaces, hyphens, or underscores, with no leading or trailing spaces and excluding reserved names. The system SHALL also surface derived titles from progression state — a prestige tier name from the current Prestige Rank and an adventurer rank derived from average level.

#### Scenario: Name validation rejects an over-length name

- **WHEN** a proposed character name exceeds 16 characters
- **THEN** the name is rejected as invalid

#### Scenario: Identity survives prestige

- **WHEN** a character prestiges
- **THEN** its unique identifier and display name are unchanged
