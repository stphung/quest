# Stormglass — Currency & Storm Sigils Specification

## Purpose

Define the Stormglass system: a character-level soft currency earned passively from gameplay and spent at the Stormglass Exchange. This capability owns how Stormglass is discovered, earned, and spent (Invoke Challenge, Chrono Surge, Storm Lure, and Storm Sigils), the permanent percentage bonuses granted by etched Storm Sigils, and the deterministic daily rotation that governs which sigil effects can be etched each calendar day. Other systems (drops, dungeons, Soulforge, challenges, fishing, prestige) are referenced only as sources or consumers of the currency.

## Requirements

### Requirement: Stormglass Currency Discovery And Persistence

Stormglass SHALL be a per-character currency balance (not account-level) that persists across prestige resets. The currency SHALL be discovered the first time a non-equipped item is salvaged while the character is at prestige rank 15 or higher; a character below prestige rank 15 who has not yet discovered Stormglass SHALL neither salvage nor accrue it. Once discovered, salvage and all other earning SHALL always apply regardless of the character's current prestige rank.

#### Scenario: Discovery on first salvage at prestige 15+

- **WHEN** a character at prestige rank 15 or higher salvages a non-equipped item drop for the first time
- **THEN** Stormglass becomes discovered
- **AND** the salvage value is added to the balance and a discovery/salvage notification is surfaced

#### Scenario: No accrual before discovery below prestige 15

- **WHEN** a non-equipped item drops for a character below prestige rank 15 that has not discovered Stormglass
- **THEN** no Stormglass is awarded and Stormglass remains undiscovered

#### Scenario: Balance survives prestige

- **WHEN** a character with a Stormglass balance performs a prestige reset
- **THEN** the Stormglass balance and discovered state are retained

### Requirement: Currency Earning Sources

The system SHALL award Stormglass from four sources with fixed values. Item salvage of non-equipped drops SHALL award by rarity: Common 1, Magic 1, Rare 3, Epic 8, Legendary 25. Dungeon treasure caches SHALL award by dungeon size: Small 5, Medium 15, Large 30, Epic 50, Legendary 75. Failed Soulforge enhancements SHALL award a consolation by target enhancement level: levels 1–4 award 0 (they cannot fail), and levels 5–10 award 5, 10, 15, 25, 40, 75 respectively. Completed challenge minigames SHALL award Stormglass scaled by difficulty; if Stormglass is not yet discovered, the intended Stormglass reward SHALL instead be converted to XP at one tenth its value expressed as a percentage of the next level.

#### Scenario: Salvage value by rarity

- **WHEN** a non-equipped Legendary item is salvaged
- **THEN** 25 Stormglass is awarded

#### Scenario: Dungeon treasure cache by size

- **WHEN** a treasure room is entered in a Large dungeon with Stormglass active
- **THEN** 30 Stormglass is awarded

#### Scenario: Soulforge failure consolation

- **WHEN** an enhancement attempt targeting level 8 fails
- **THEN** 25 Stormglass is awarded, and a level 1–4 failure (which cannot happen) would award 0

#### Scenario: Challenge reward before discovery

- **WHEN** a challenge that grants Stormglass is completed but Stormglass has not been discovered
- **THEN** the reward is applied as XP equal to one tenth of the Stormglass amount as a percentage toward the next level instead of currency

### Requirement: Stormglass Exchange Spending Options And Storm Lure

The Stormglass Exchange SHALL offer exactly four spending options: Invoke Challenge, Chrono Surge, Storm Sigils, and Storm Lure. Selecting an option that costs currency SHALL be blocked when the balance is below the required amount, and any successful purchase SHALL deduct its cost from the balance. Storm Lure SHALL cost 50,000 Stormglass and, on purchase, activate a consumable that guarantees Storm Leviathan encounters; it SHALL only be purchasable when the balance is at least 50,000, no Storm Lure is already active, the Storm Leviathan has not yet been caught, and the character's fishing rank is at least 40.

#### Scenario: Four exchange options

- **WHEN** the Stormglass Exchange is opened
- **THEN** the menu presents Invoke Challenge, Chrono Surge, Storm Sigils, and Storm Lure

#### Scenario: Insufficient balance blocks purchase

- **WHEN** the player selects an option whose cost exceeds the current Stormglass balance
- **THEN** the purchase does not proceed and no currency is deducted

#### Scenario: Storm Lure purchase gated

- **WHEN** the player has 50,000+ Stormglass, fishing rank 40+, no active lure, and has not caught the Storm Leviathan
- **THEN** the Storm Lure can be purchased, deducting 50,000 Stormglass and activating the lure

#### Scenario: Storm Lure blocked by unmet gate

- **WHEN** any of the requirements is unmet (rank below 40, a lure already active, the Leviathan already caught, or insufficient balance)
- **THEN** Storm Lure cannot be purchased

### Requirement: Invoke Challenge Spending

Invoke Challenge SHALL cost 3,000 Stormglass and, on purchase, present three distinct randomly selected challenge types drawn from the pool of thirteen invokable challenge types, excluding any challenge already pending in the player's challenge menu. The player SHALL pick one of the three to add and start immediately, bypassing normal discovery. The option SHALL be unavailable unless at least three challenge types are available (not already pending). Forfeiting the pick after purchase SHALL forfeit the spent 3,000 Stormglass.

#### Scenario: Invoke presents three choices

- **WHEN** the player pays 3,000 Stormglass to Invoke Challenge and at least three non-pending challenge types exist
- **THEN** three distinct challenge types are offered and choosing one adds and starts that challenge

#### Scenario: Blocked when fewer than three available

- **WHEN** fewer than three invokable challenge types are available (the rest are already pending)
- **THEN** Invoke Challenge cannot be started

### Requirement: Chrono Surge Spending

Chrono Surge SHALL offer four fixed tiers that fast-forward simulated game ticks in exchange for Stormglass, at exactly: 9,000 ticks for 500 Stormglass ("15 minutes"), 36,000 ticks for 2,000 Stormglass ("1 hour"), 144,000 ticks for 8,000 Stormglass ("4 hours"), and 288,000 ticks for 16,000 Stormglass ("8 hours"), where 10 ticks equal one simulated second. At the start of each surge the system SHALL roll for an overcharge proc whose chance (as a percentage) equals the summed value of all etched Sigil of Overcharge bonuses; on a successful proc the surge's tick count SHALL be multiplied by 1.5.

#### Scenario: Purchasing a surge tier

- **WHEN** the player selects the 4-hour Chrono Surge tier with at least 8,000 Stormglass
- **THEN** 8,000 Stormglass is deducted and a surge of 144,000 ticks begins

#### Scenario: Overcharge extends the surge

- **WHEN** a Chrono Surge starts and the overcharge proc succeeds (its chance driven by the total Sigil of Overcharge bonus)
- **THEN** the surge runs 1.5 times its base tick count and is flagged as overcharged

### Requirement: Storm Sigil Slots

Storm Sigils SHALL provide up to five sigil slots that persist through prestige and are character-level. All slots SHALL start locked, and slots SHALL be unlocked in order at fixed escalating Stormglass costs of 25,000, 50,000, 100,000, 200,000, and 400,000 (a 2× exponential curve). Unlocking a slot SHALL deduct its cost and increase the unlocked-slot count by one; once all five are unlocked no further unlock SHALL be offered.

#### Scenario: Sequential slot unlock costs

- **WHEN** a character with three slots unlocked unlocks the next slot
- **THEN** 200,000 Stormglass is deducted and four slots become unlocked

#### Scenario: All slots unlocked

- **WHEN** all five slots are unlocked
- **THEN** no further slot-unlock action is available

### Requirement: Sigil Etch And Reroll

Etching an empty unlocked slot and rerolling an already-etched slot SHALL each cost 25,000 Stormglass and present a pick-one-of-three screen of freshly rolled sigils drawn from the current day's rotation pool. Rerolling SHALL destroy the slot's existing sigil at the moment the cost is paid. Selecting a choice SHALL place that sigil in the target slot; forfeiting the pick SHALL leave the slot empty while the 25,000 Stormglass remains spent.

#### Scenario: Etch an empty slot

- **WHEN** the player pays 25,000 Stormglass to etch an empty unlocked slot and selects one of the three offered sigils
- **THEN** the chosen sigil is placed in that slot

#### Scenario: Reroll destroys before replacing

- **WHEN** the player pays 25,000 Stormglass to reroll an etched slot
- **THEN** the existing sigil is immediately removed and three new choices are offered

#### Scenario: Forfeiting a pick forfeits the currency

- **WHEN** the player forfeits the pick-one-of-three screen after paying
- **THEN** the slot remains empty and the 25,000 Stormglass is not refunded

### Requirement: Sigil Effect Types And Bonus Aggregation

The system SHALL define exactly twelve sigil effect types, each with a fixed value range: XP 5–25%, Damage 3–15%, Damage Reduction 1–5%, Crit Chance 2–8%, Drop Rate 2–10%, Max HP 3–15%, Fishing Speed 5–25%, Offline XP 5–20%, Attack Speed 2–10%, Double Strike 1–5%, Regen Delay 2–10%, and Chrono Overcharge 5–20%. The aggregate bonus for each effect SHALL be the sum of the values of every etched sigil of that effect (multiple sigils of the same effect stack additively), and these aggregated bonuses SHALL be injected into combat and other systems as permanent percentage bonuses.

#### Scenario: Effect value ranges enforced

- **WHEN** a Damage Reduction sigil is rolled
- **THEN** its value falls within 1–5%

#### Scenario: Same-effect sigils stack additively

- **WHEN** two etched sigils grant XP bonuses of 10% and 15%
- **THEN** the aggregated XP bonus is 25%

### Requirement: Sigil Value Roll And Grading

Each sigil SHALL be rolled from a single uniform value in [0,1) mapped through the exponential curve (e^(3p) − 1) / (e^3 − 1) into the effect's [min,max] range, rounded to one decimal place and clamped to the range, so low rolls compress near the minimum and high rolls stretch toward the maximum. The same uniform value SHALL determine one of twenty-one grades across seven letter tiers (F, E, D, C, B, A, S, each with minus/plain/plus variants) by percentile, where higher percentiles yield higher grades (for example, ≥ 0.985 → S+, ≥ 0.96 → S, ≥ 0.95 → S−, and below 0.03 → F−).

#### Scenario: High roll yields high value and grade

- **WHEN** a sigil is rolled with a uniform value near 1.0
- **THEN** its value is near the effect's maximum and its grade is S or S+

#### Scenario: Low roll yields low value and grade

- **WHEN** a sigil is rolled with a uniform value near 0.0
- **THEN** its value is near the effect's minimum and its grade is F−

### Requirement: Daily Sigil Rotation

Each calendar day SHALL make exactly five of the twelve sigil effect types available for etching and rerolling, chosen deterministically so the same day always yields the same five distinct types with no duplicates. The active day SHALL be the current UTC calendar date (the pool changes at UTC midnight), and etch/reroll rolls SHALL only draw from that day's five available effect types.

#### Scenario: Five distinct effects per day

- **WHEN** the daily sigil pool is computed for a given day
- **THEN** it contains exactly five distinct sigil effect types

#### Scenario: Deterministic and calendar-driven

- **WHEN** the pool is computed twice for the same UTC calendar day
- **THEN** both computations yield the identical set of five effects, and an adjacent day yields a different set

#### Scenario: Etch draws only from the daily pool

- **WHEN** the player etches or rerolls a sigil
- **THEN** every offered choice is one of the five effect types in the current day's rotation
