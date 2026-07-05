# The Loom — Resource Production Specification

## Purpose

Define the Loom of Worlds, a late-game resource-production and crafting system in which six fixed Extractor nodes feed buildable Shuttle nodes through a direct-pull network to combine base resources into higher-tier materials. Sustained production rates satisfy Woven Patterns, whose completion gates Ascension VII-X, unlocks the Loom zone bands, expands Shuttle capacity, and — once all 28 patterns are complete — converts Weave Rate into Prestige Ranks. This capability owns production, Shuttles, and patterns; it references Ascension, Prestige Rank, and zone unlocks only as consumers or gates.

## Requirements

### Requirement: Loom Discovery And Activation

The system SHALL keep the Loom undiscovered until The Deep has been discovered AND its Gateway (Deep Layer 30) has been opened, at which point the Loom SHALL become discovered exactly once. On discovery the system SHALL seed the full sequence of 29 Woven Patterns (28 completable plus 1 eternal), set the active pattern to the first pattern, and unlock only the Ember Spindle Extractor, leaving the other five Extractors locked until the neighbor-unlock process reaches them.

#### Scenario: Discovery fires when the Gateway opens

- **WHEN** The Deep is discovered and its Gateway at Layer 30 is opened while the Loom is not yet discovered
- **THEN** the Loom becomes discovered, the 29-pattern sequence is created with the first pattern active, and the Ember Spindle is the only unlocked Extractor

#### Scenario: Discovery does not repeat

- **WHEN** the discovery trigger is evaluated again after the Loom is already discovered
- **THEN** the Loom state is left unchanged and existing pattern progress is preserved

### Requirement: Extractor Base Production And Upgrades

The system SHALL provide six fixed Extractors, each producing one native base resource. An unlocked, non-upgrading Extractor SHALL produce at 25 units/hr at level 1, multiplied by `1 + (level - 1) × 0.5`, capped at level 20. Production SHALL accumulate into the node's buffer up to a capacity of ten hours of production at the current rate. Starting an upgrade SHALL consume 50% of the buffer's capacity (and SHALL be refused if the current buffer holds less than that amount), begin a lockout of `level × 2 hours` during which the node produces nothing, and on completion raise the node's level by one. Locked Extractors SHALL unlock through the neighbor process: any unlocked node whose buffer is at least 50% full accumulates unlock progress for up to two of its locked cycle-neighbors, which unlock after two hours of such progress.

#### Scenario: Level scales the production rate

- **WHEN** an unlocked Extractor at level 1 produces its native resource
- **THEN** it produces at 25 units/hr, and at level 3 it produces at 50 units/hr (25 × 2.0)

#### Scenario: Upgrade drains buffer and halts production

- **WHEN** a level-2 Extractor with at least half its buffer capacity begins an upgrade
- **THEN** 50% of its buffer capacity is consumed, a 4-hour lockout begins, and the node produces nothing until the lockout ends and its level becomes 3

#### Scenario: Upgrade refused without enough buffer

- **WHEN** an upgrade is attempted but the current buffer is below 50% of capacity
- **THEN** the upgrade does not start and the node keeps producing at its current level

#### Scenario: Neighbor unlocks a locked Extractor

- **WHEN** an unlocked Extractor stays at least 50% buffer-full for two hours of accumulated progress toward a locked cycle-neighbor
- **THEN** that neighbor Extractor unlocks and begins producing its own native resource

### Requirement: Recipe Registry And Production Tiers

The system SHALL define exactly 7 exclusive recipes, each identified by an unordered pair of input resources plus a node nature acting as a hidden catalyst, and each producing exactly one output resource belonging to exactly one tier. Tier 1 SHALL contain 3 recipes combining base resources into confluences, Tier 2 SHALL contain 3 recipes combining a confluence with a base resource into a reaction product, and Tier 3 SHALL contain the single recipe that combines two Tier 2 products into Woven Reality, the terminal resource. Recipe lookup SHALL be commutative in the two inputs, and a combination with no matching recipe SHALL yield no output.

#### Scenario: Seven recipes across three tiers

- **WHEN** the recipe registry is queried by tier
- **THEN** Tier 1 has 3 recipes, Tier 2 has 3 recipes, Tier 3 has 1 recipe, for 7 total, and each output resource is produced by exactly one recipe

#### Scenario: Commutative lookup

- **WHEN** a recipe is looked up with its two inputs in either order under the correct node nature
- **THEN** the same output resource and amount are returned

#### Scenario: Woven Reality is terminal

- **WHEN** the Tier 3 recipe runs
- **THEN** it produces Woven Reality, which is never consumed as an input by any recipe

### Requirement: Direct-Pull Shuttle Processing

The system SHALL, each tick, drive Shuttle production by having every operational Shuttle pull directly from its declared source nodes rather than through discrete pipe objects. When multiple Shuttles share a source, the source's available buffer SHALL be split equally among its consumers (`share = source_buffer / consumer_count`). A Shuttle's output rate SHALL equal `min(total_pull_a, total_pull_b) × recipe_amount`, and there SHALL be no per-tier intake cap gating throughput. Shuttles SHALL be processed in tier order (Tier 1, then Tier 2, then Tier 3) so lower-tier output is available to higher-tier consumers within the same tick. Extractors SHALL be valid sources for any Shuttle tier, while a Shuttle SHALL be a valid source only for consuming Shuttles of strictly higher tier.

#### Scenario: Output limited by the scarcer input

- **WHEN** a Shuttle pulls input A at 10/hr and input B at 6/hr for a recipe whose amount is 1.0
- **THEN** its output rate is 6/hr (the minimum of the two pulls times the recipe amount)

#### Scenario: Shared source split among consumers

- **WHEN** two Shuttles both pull from the same source node
- **THEN** each receives an equal half of that source's available buffer for the tick

#### Scenario: Tier ordering of source eligibility

- **WHEN** a Tier 2 Shuttle declares its sources
- **THEN** Extractors and Tier 1 Shuttles are valid sources, but Tier 2 or Tier 3 Shuttles are rejected

### Requirement: Shuttle Building And Tier Gates

The system SHALL allow a Shuttle to be built only when it is locked to a valid recipe, the recipe's tier is unlocked by completed pattern count (Tier 1 at 1 pattern, Tier 2 at 8 patterns, Tier 3 at 15 patterns), all declared sources are valid for the recipe's tier, the player is below the Shuttle capacity limit, and the input-A build cost can be paid (Tier 1 = 250, Tier 2 = 150, Tier 3 = 100 units of the input-A resource). A newly built Shuttle SHALL enter construction for a tier-based duration (Tier 1 = 2 hours, Tier 2 = 4 hours, Tier 3 = 6 hours) during which it does not produce, and SHALL become operational when construction completes.

#### Scenario: Tier locked by pattern count

- **WHEN** a build is attempted for a Tier 2 recipe with fewer than 8 completed patterns
- **THEN** the build is rejected as tier-locked

#### Scenario: Build cost paid and construction begins

- **WHEN** a Tier 1 Shuttle is built with at least 250 units of its input-A resource available
- **THEN** 250 units are deducted and the Shuttle enters a 2-hour construction period before producing

#### Scenario: Insufficient resources

- **WHEN** a build is attempted without enough input-A resource to cover the tier's cost
- **THEN** the build is rejected and no Shuttle is created

### Requirement: Shuttle Capacity Limit

The system SHALL cap the number of Shuttles the player may have according to completed pattern count: 0 patterns allow 0 Shuttles, 1-3 allow 1, 4-7 allow 2, 8-11 allow 3, 12-14 allow 4, and 15 or more allow the maximum of 5. A build SHALL be rejected when the current Shuttle count already meets or exceeds this limit.

#### Scenario: Capacity grows with patterns

- **WHEN** the player has completed 8 patterns
- **THEN** the Shuttle capacity is 3

#### Scenario: Capacity refuses builds at the limit

- **WHEN** the player is at the Shuttle capacity for their pattern count and attempts another build
- **THEN** the build is rejected as at-capacity

### Requirement: Shuttle Upgrades Gated By Ascension

The system SHALL allow a Shuttle to be upgraded, raising its level and increasing its effective throughput multiplier by `0.5` per level, with the maximum Shuttle level determined by Ascension tier: Ascension 0 through VI cap Shuttle level at 1, VII at 3, VIII at 5, IX at 7, and X at 10. An upgrade SHALL be refused when the Ascension cap is 1 (no upgrades possible), when the Shuttle is at its cap, when the Shuttle is under construction, or when the Shuttle's own buffer cannot pay the upgrade cost of `100 × level^1.2`.

#### Scenario: Ascension caps the shuttle level

- **WHEN** a Shuttle at level 5 is upgraded while the player is at Ascension VIII (cap 5)
- **THEN** the upgrade is rejected because the Shuttle is at its Ascension-gated cap

#### Scenario: No upgrades before Ascension VII

- **WHEN** a Shuttle upgrade is attempted while the player is at Ascension VI or lower (cap 1)
- **THEN** the upgrade is rejected as Ascension-too-low

### Requirement: Woven Pattern Structure

The system SHALL define 29 Woven Patterns: 28 completable patterns plus 1 eternal pattern. Each pattern SHALL carry one or more requirements, each demanding a specific resource sustained at or above a required rate (units/hr) for a required duration. The completable-pattern total reported to the player SHALL be 28 (excluding the eternal pattern), and the completed-pattern count SHALL likewise exclude the eternal pattern.

#### Scenario: Total and completable counts

- **WHEN** the pattern sequence is created
- **THEN** there are 29 patterns in total, the displayed Woven Pattern total is 28, and all 28 completable patterns fully completed yields a completed count of 28

#### Scenario: Multi-requirement pattern

- **WHEN** a pattern has multiple resource requirements
- **THEN** each requirement tracks its own sustained progress and completes independently, and the pattern completes only when every requirement is complete

### Requirement: Simultaneous Sustain Progression

The system SHALL advance a pattern's requirement timers only when ALL of that pattern's incomplete requirements simultaneously meet or exceed their rate thresholds, measured over a 20-second rolling window. If any required rate drops below its threshold, no timers SHALL advance, but sustained progress SHALL only pause and never decay. A requirement SHALL complete when its sustained time reaches its required duration; when all requirements complete, the pattern SHALL complete and the active pattern SHALL advance to the next uncompleted pattern.

#### Scenario: All rates required at once

- **WHEN** a two-requirement pattern has only one of its two required resources at threshold
- **THEN** neither requirement's timer advances

#### Scenario: Progress pauses without decay

- **WHEN** a requirement has accumulated sustained time and its rate then falls below threshold
- **THEN** the accumulated time is retained unchanged until the rate recovers

#### Scenario: Pattern completion advances the active pattern

- **WHEN** the final incomplete requirement of the active pattern reaches its duration
- **THEN** the pattern is marked complete and the active pattern becomes the next uncompleted pattern

### Requirement: Eternal Pattern Never Completes

The system SHALL treat the 29th (eternal) pattern as a perpetual endgame sink whose requirement timers advance while its rate is met but whose requirements and pattern are never marked complete, and which is excluded from all pattern counts and from the all-patterns-complete condition.

#### Scenario: Eternal pattern excluded from counts

- **WHEN** every pattern including the eternal one is force-marked complete
- **THEN** the completed-pattern count is 28, not 29

#### Scenario: All-patterns-complete ignores the eternal pattern

- **WHEN** all 28 completable patterns are complete but the eternal pattern is not
- **THEN** the all-patterns-complete condition is satisfied

### Requirement: Pattern Milestones Gate Ascension And Zones

The system SHALL recognize pattern-completion milestones at 4, 8, 16, 22, and 28 completed patterns, and SHALL use completed pattern count as the gate for Ascension VII-X (requiring 8, 16, 22, and 28 patterns respectively) and as the source of Loom zone unlocks (4 patterns raise the Loom zone cap to 34, 8 to 38, 16 to 42, 22 to 46, and 28 to 50; fewer than 4 unlock no Loom zones). Reaching the 28-pattern milestone SHALL additionally enable Weave-Rate-to-Prestige-Rank conversion. Zone-unlock detail is owned by the zones capability; this capability only supplies the pattern count that feeds it.

#### Scenario: Ascension VII pattern gate

- **WHEN** Ascension VII eligibility is checked
- **THEN** it requires 8 completed patterns, and VIII/IX/X require 16/22/28 respectively

#### Scenario: Loom zone cap follows pattern count

- **WHEN** the player has completed 16 patterns
- **THEN** the Loom zone cap is zone 42, and reaching 28 patterns raises it to zone 50

#### Scenario: 28-pattern milestone enables conversion

- **WHEN** the 28th completable pattern completes
- **THEN** the Final Weave milestone is reached and Weave-Rate-to-Prestige-Rank conversion becomes active

### Requirement: Weave Rate To Prestige Rank Conversion

The system SHALL, only after all 28 completable patterns are complete, convert the current Weave Rate (the measured Woven Reality production rate in units/hr) into Prestige Ranks per hour using the self-multiplying formula `PR/hr = round(WR × (1 + WR/100))`, yielding roughly 1:1 at low rates and about 2.3× near the maximum. A Weave Rate that is zero, negative, or non-finite SHALL yield 0 PR/hr. Conversion SHALL grant whole Prestige Ranks over real elapsed wall-clock time based on that rate, with elapsed time capped at 7 days to bound offline accrual.

#### Scenario: Conversion inactive before all patterns complete

- **WHEN** fewer than 28 completable patterns are complete
- **THEN** no Prestige Ranks are granted from Weave Rate regardless of Woven Reality production

#### Scenario: Self-multiplying formula

- **WHEN** the Weave Rate is 10 units/hr
- **THEN** the conversion yields 11 PR/hr; at 50 it yields 75 PR/hr; at 131 it yields 303 PR/hr

#### Scenario: Non-positive or invalid rate yields nothing

- **WHEN** the Weave Rate is 0, negative, or non-finite
- **THEN** the conversion yields 0 PR/hr

### Requirement: Wall-Clock Timing And Surge Exclusion

The system SHALL run all Loom timers — Shuttle construction, Extractor upgrade lockouts, neighbor unlocking, pattern sustain durations, and Weave-Rate conversion accrual — on real wall-clock time rather than tick-accelerated time, and SHALL skip Loom processing entirely during a Chrono Surge so that surge acceleration does not advance Loom timers.

#### Scenario: Chrono Surge does not accelerate the Loom

- **WHEN** a Chrono Surge is active
- **THEN** the Loom is not ticked and its construction, upgrade, and sustain timers do not advance from the surge

#### Scenario: Timers measured in real time

- **WHEN** wall-clock time elapses between ticks
- **THEN** Loom timers advance by the real elapsed seconds (bounded per tick to avoid large jumps on resume)
