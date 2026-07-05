# Items, Drops & Equipment Specification

## Purpose

Defines how the game generates, drops, scores, and equips loot. Items are produced through a single seeded generation path that stamps each one with an equipment slot, a rarity, an item level derived from the zone, and a quality tier rolled on an exponential curve. Item level and tier together scale the item's attributes and affixes, and an intrinsic, character-independent power score drives automatic equipping of upgrades. This capability covers mob and boss drop rates and rarity tables, the tier curve and multiplier table, item scoring, auto-equip, and the seven equipment slots. Enhancement and god items are separate capabilities that extend items rather than replace these rules.

## Requirements

### Requirement: Generate Items Through a Single Seeded Entry Point

The system SHALL generate every item through one generation routine driven by a caller-provided random number generator, so that the same seed produces the same item. Each generated item SHALL carry an equipment slot, a rarity, an item level (ilvl), a quality tier (T0–T9), attribute bonuses, a (possibly empty) list of affixes, and a procedurally generated display name.

#### Scenario: Reproducible generation from a seed

- **WHEN** the generation routine is invoked twice with the same slot, rarity, item level, and an equivalent seeded RNG
- **THEN** it produces two items with identical tier, attributes, affixes, and display name

#### Scenario: Every item is fully populated

- **WHEN** an item is generated
- **THEN** it has the requested slot, the requested rarity, the requested item level, a tier in the range T0–T9, and a non-empty display name

### Requirement: Scale Item Level From Zone

The system SHALL set an item's level to the zone identifier multiplied by 10 (ilvl = zone_id × 10), so Zone 1 yields ilvl 10 and Zone 10 yields ilvl 100. Item level SHALL contribute an ilvl multiplier of `1.0 + (ilvl − 10) / 30.0`, giving 1.0× at ilvl 10, ~2.33× at ilvl 50, and 4.0× at ilvl 100.

#### Scenario: Zone determines item level

- **WHEN** an item drops in Zone 1
- **THEN** the item's ilvl is 10

- **WHEN** an item drops in Zone 10
- **THEN** the item's ilvl is 100

#### Scenario: Higher item level yields stronger items on average

- **WHEN** many items of the same rarity are generated at ilvl 100 versus ilvl 10
- **THEN** the ilvl 100 items have a higher average attribute total because of the larger ilvl multiplier

### Requirement: Roll Item Tier On An Exponential Quality Curve

The system SHALL roll a quality tier from T0 through T9 using a fixed exponential distribution where lower tiers are common and T9 is exceedingly rare. The per-tier probabilities SHALL be:

| Tier | Probability |
|------|-------------|
| T0 | 38.0% |
| T1 | 24.0% |
| T2 | 15.0% |
| T3 | 10.0% |
| T4 | 6.0% |
| T5 | 3.5% |
| T6 | 2.0% |
| T7 | 1.0% |
| T8 | 0.4% |
| T9 | 0.1% |

An item whose recorded tier is outside the range T0–T9 SHALL be treated as T5. Items loaded from older saves that predate the tier system SHALL default to tier T1.

#### Scenario: Tier distribution over many rolls

- **WHEN** the tier roll is sampled a large number of times
- **THEN** roughly 38% land on T0, ~24% on T1, and only ~0.1% on T9, matching the table above

#### Scenario: Out-of-range tier falls back to T5

- **WHEN** an item's tier value is greater than 9
- **THEN** its quality multiplier is resolved as if it were T5

### Requirement: Apply The Tier Quality Multiplier

The system SHALL map each tier to a stat multiplier ranging from 0.40× at T0 to 1.00× at T9, and SHALL scale both attribute values and affix values by the effective multiplier `ilvl_multiplier × tier_multiplier`. The tier multipliers SHALL be:

| Tier | Multiplier |
|------|-----------|
| T0 | 0.40× |
| T1 | 0.47× |
| T2 | 0.54× |
| T3 | 0.61× |
| T4 | 0.68× |
| T5 | 0.74× |
| T6 | 0.80× |
| T7 | 0.86× |
| T8 | 0.93× |
| T9 | 1.00× |

Scaled attribute values SHALL be rounded and clamped to a minimum of 1.

#### Scenario: Tier scales item strength

- **WHEN** two items of the same rarity and item level are generated, one at T0 and one at T9
- **THEN** the T9 item's attribute total is about 2.5× the T0 item's (1.00 ÷ 0.40) on average

#### Scenario: Combined scaling at max item level and tier

- **WHEN** an item is generated at ilvl 100 (4.0× ilvl multiplier) and T9 (1.00× tier multiplier)
- **THEN** its base attribute and affix values are multiplied by 4.0×

### Requirement: Assign Attributes And Affixes By Rarity

The system SHALL grant each item 1 to 3 randomly chosen attributes (from STR, DEX, CON, INT, WIS, CHA) with per-rarity base value ranges, and a per-rarity number of affixes drawn from a pool of affix types. Common items SHALL never receive affixes.

| Rarity | Base attribute range (at ilvl 10) | Affix count |
|--------|-----------------------------------|-------------|
| Common | 1 | 0 |
| Magic | 1–2 | 1 |
| Rare | 2–3 | 2–3 |
| Epic | 3–4 | 3–4 |
| Legendary / Mythic | 4–6 | 4–5 |

Affix values SHALL be drawn from per-rarity, per-affix-type base ranges and scaled by the effective ilvl × tier multiplier.

#### Scenario: Common items carry no affixes

- **WHEN** a Common item is generated
- **THEN** it has zero affixes and at least one attribute bonus

#### Scenario: Rarity governs affix count

- **WHEN** a Magic item is generated
- **THEN** it has exactly 1 affix

- **WHEN** a Legendary item is generated
- **THEN** it has between 4 and 5 affixes

### Requirement: Rank Rarity Tiers In A Fixed Order

The system SHALL define six rarity tiers ordered Common < Magic < Rare < Epic < Legendary < Mythic. The Mythic tier SHALL display as "God" and SHALL be reserved exclusively for god items; ordinary generation and drops SHALL never produce Mythic items.

#### Scenario: Rarity ordering

- **WHEN** two rarities are compared
- **THEN** Common is lowest, Legendary outranks Epic, and Mythic (displayed as "God") outranks Legendary

#### Scenario: Mythic reserved for god items

- **WHEN** an item is produced by ordinary mob or boss drops
- **THEN** its rarity is at most Legendary and is never Mythic

### Requirement: Drop Items From Mobs With Prestige-Scaled Chance

The system SHALL, on a normal (non-boss) enemy kill, drop an item with probability `0.15 + prestige_rank × 0.01`, capped at 0.25 (15% base, +1% per prestige rank, hard ceiling 25%). A Trophy Hall (Haven) drop-rate bonus SHALL be applied multiplicatively to the base chance, but the final drop chance SHALL never exceed 0.25. Mob drops SHALL be capped at Epic rarity and SHALL never be Legendary. When an item drops, its slot SHALL be chosen at random and its item level SHALL be the current zone's ilvl.

#### Scenario: Base drop chance at prestige 0

- **WHEN** a mob is killed at prestige rank 0 with no Haven bonus
- **THEN** the item drop probability is 15%

#### Scenario: Drop chance caps at 25%

- **WHEN** a mob is killed at a high prestige rank (e.g. rank 100), with or without a Trophy Hall bonus
- **THEN** the drop probability is capped at 25%

#### Scenario: Mobs never drop Legendary

- **WHEN** a mob drops an item, even at maximum prestige and Haven rarity bonus
- **THEN** the item's rarity is Common, Magic, Rare, or Epic — never Legendary

### Requirement: Roll Mob Drop Rarity With Prestige And Haven Modifiers

The system SHALL roll mob drop rarity from a base distribution of 60% Common, 28% Magic, 10% Rare, 2% Epic. A prestige bonus of 0.5% per rank (capped at 10%, reached at rank 20) SHALL shift weight out of Common: it lowers the Common weight (floored at 20%) and raises the Rare weight (by 60% of the bonus) with the remainder going to Epic. A Haven Workshop rarity bonus SHALL apply multiplicatively to the non-Common rates, capped at +25%, with Common absorbing the remainder (floored at 20%).

#### Scenario: Base rarity distribution

- **WHEN** mob rarity is rolled many times at prestige rank 0 with no Haven bonus
- **THEN** results approximate 60% Common, 28% Magic, 10% Rare, 2% Epic

#### Scenario: Haven Workshop reduces Common share

- **WHEN** mob rarity is rolled with a +25% Workshop rarity bonus
- **THEN** the proportion of Common results drops noticeably compared to no bonus

### Requirement: Guarantee Boss Drops With Fixed Rarity Tables

The system SHALL always drop exactly one item when a boss is defeated, independent of drop-chance rolls. Boss drops SHALL ignore Haven and prestige bonuses and SHALL use fixed rarity tables. A normal boss SHALL roll 40% Magic, 35% Rare, 23% Epic, 2% Legendary. The final-zone boss (Zone 10) SHALL roll 20% Magic, 40% Rare, 35% Epic, 5% Legendary. Bosses SHALL never drop Common items.

#### Scenario: Boss always drops

- **WHEN** any boss is defeated
- **THEN** exactly one item is produced, with item level equal to the zone's ilvl

#### Scenario: Final boss has higher Legendary rate

- **WHEN** many Zone 10 final-boss kills and many normal-boss kills are compared
- **THEN** the final boss yields Legendary at ~5% versus ~2% for a normal boss

#### Scenario: Bosses never drop Common

- **WHEN** a boss drop rarity is rolled
- **THEN** the result is Magic, Rare, Epic, or Legendary — never Common

### Requirement: Provide Seven Equipment Slots

The system SHALL define exactly seven equipment slots — Weapon, Armor, Helmet, Gloves, Boots, Amulet, Ring — each holding at most one item. Dropped items SHALL be assigned a slot chosen uniformly at random from these seven, and each slot SHALL be independently equippable.

#### Scenario: All slots reachable on drops

- **WHEN** many items are dropped
- **THEN** every one of the seven slots appears as a drop target

#### Scenario: Slots are independent

- **WHEN** a Weapon and an Armor item are equipped
- **THEN** both occupy their respective slots simultaneously without displacing each other

### Requirement: Score Item Power Intrinsically

The system SHALL compute an intrinsic, character-independent power score for each item as the sum of all attribute values plus the sum of each affix value multiplied by its affix power weight, rounded to an integer. The same item SHALL always produce the same power score regardless of who holds it. The affix power weights SHALL be:

| Affix | Weight |
|-------|--------|
| Damage Percent | 2.0 |
| Crit Chance | 1.5 |
| Crit Multiplier | 1.5 |
| Damage Reduction | 1.3 |
| Attack Speed | 1.2 |
| HP Regen | 1.0 |
| XP Gain | 1.0 |
| Damage Reflection | 0.8 |
| HP Bonus | 0.5 |

#### Scenario: Power sums attributes and weighted affixes

- **WHEN** an item has 1 total attribute point and a Damage Percent affix of value 20
- **THEN** its power score is 41 (1 + 20 × 2.0)

#### Scenario: Power is build-independent

- **WHEN** the same item is scored for two different characters
- **THEN** both compute the identical power number

### Requirement: Auto-Equip Higher-Power Items

The system SHALL automatically equip a newly acquired item into its slot when the item's intrinsic power is strictly greater than the currently equipped item's power (treating an empty slot as power 0); on equal power it SHALL keep the incumbent. A Mythic ("God") item SHALL never be auto-replaced by an item of any lower rarity.

#### Scenario: Upgrade replaces incumbent

- **WHEN** a new item has strictly higher power than the item in its slot
- **THEN** the new item is equipped, replacing the old one

#### Scenario: Equal or lower power is rejected

- **WHEN** a new item has power equal to or lower than the equipped item
- **THEN** the equipped item is kept and the new item is not auto-equipped

#### Scenario: God items are protected

- **WHEN** a Mythic (God) item is equipped and a higher-power Legendary item is offered for the same slot
- **THEN** the Mythic item is retained and the Legendary is not auto-equipped
