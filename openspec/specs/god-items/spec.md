# God Items Specification

## Purpose

God Items are three Norse-mythology-themed endgame artifacts (Asprika, Sleipnir, Megingjord) that sit at the very top of the item rarity ladder, above Epic and Legendary. Each is a fixed, hand-authored piece — not a procedural drop — that grants a powerful combat passive plus attribute bonuses and a shared experience affix, and one of them also grants a suite of non-combat quality-of-life bonuses. Because they occupy the unique God (Mythic) rarity, they are shielded from the normal auto-equip replacement logic so a stronger ordinary item can never displace them.

## Requirements

### Requirement: God Rarity Tier

The system SHALL define a God (Mythic) rarity that ranks strictly above every ordinary rarity, forming the ordering Common < Magic < Rare < Epic < Legendary < God, and SHALL reserve this rarity exclusively for God Items. The God rarity SHALL be displayed to the player with the label "God".

#### Scenario: God outranks Legendary

- **WHEN** the rarity of a God Item is compared against a Legendary item
- **THEN** the God Item ranks higher (God is the maximum rarity, one step above Legendary)

#### Scenario: God rarity display label

- **WHEN** a God Item's rarity is rendered
- **THEN** it is labeled "God" (all ordinary items use their own labels: Common, Magic, Rare, Epic, Legendary)

### Requirement: God Item Roster and Identity

The system SHALL provide exactly three God Items, each occupying a distinct equipment slot, each created at God (Mythic) rarity with item level 100 and quality tier 9 (the maximum), and each carrying a single Experience Gain affix of +40%:

- **Asprika** ("Armor of the Æsir") — Armor slot; grants +40 Constitution and +20 Wisdom.
- **Sleipnir** ("Boots of the Eight-Legged") — Boots slot; grants +40 Dexterity and +20 Wisdom.
- **Megingjord** ("Belt of Giant Strength") — Ring slot; grants +40 Strength and +20 Constitution.

#### Scenario: Asprika identity

- **WHEN** Asprika is created
- **THEN** it is a God-rarity Armor with item level 100, tier 9, +40 Constitution, +20 Wisdom, and a +40% Experience Gain affix

#### Scenario: Sleipnir identity

- **WHEN** Sleipnir is created
- **THEN** it is a God-rarity Boots item with item level 100, tier 9, +40 Dexterity, +20 Wisdom, and a +40% Experience Gain affix

#### Scenario: Megingjord identity

- **WHEN** Megingjord is created
- **THEN** it is a God-rarity Ring with item level 100, tier 9, +40 Strength, +20 Constitution, and a +40% Experience Gain affix

### Requirement: Combat Passives

Each God Item SHALL grant exactly one unique combat passive while equipped:

- **Asprika — Divine Bulwark**: reduces incoming enemy damage by 30%, applied after enemy defense subtraction, with the resulting hit never falling below 1 damage.
- **Sleipnir — Windborne**: increases the player's attack speed by 100%, shortening the effective player attack interval.
- **Megingjord — Giant's Might**: increases the player's damage by 150%, applied as an early multiplier to base damage (before other percentage damage bonuses such as Haven Armory).

#### Scenario: Divine Bulwark reduces damage post-defense

- **WHEN** the player takes an enemy hit while Asprika is equipped
- **THEN** the post-defense damage is multiplied by (1 − 0.30) and floored at a minimum of 1

#### Scenario: Windborne speeds up attacks

- **WHEN** the player's attack interval is computed while Sleipnir is equipped
- **THEN** the +100% attack speed is added to the attack-speed term so the effective interval is shorter

#### Scenario: Giant's Might amplifies damage early

- **WHEN** the player's outgoing damage is computed while Megingjord is equipped
- **THEN** base damage is multiplied by (1 + 1.50) at the early-damage stage, before subsequent percentage damage bonuses

### Requirement: Non-Combat Bonuses

Sleipnir SHALL additionally grant three non-combat quality-of-life bonuses while equipped — Swiftstrider (50% reduction to post-encounter HP regen delay), Swiftfoot (50% reduction to dungeon room movement timers), and NimbleHands (50% reduction to fishing phase timers). Asprika and Megingjord SHALL grant no non-combat bonuses.

#### Scenario: Sleipnir grants all three quality-of-life bonuses

- **WHEN** Sleipnir is equipped
- **THEN** HP regen delay, dungeon movement timers, and fishing phase timers are each reduced by 50%

#### Scenario: Asprika and Megingjord have no non-combat bonuses

- **WHEN** Asprika or Megingjord is equipped
- **THEN** no regen, dungeon-speed, or fishing bonus is contributed by that item

### Requirement: Simultaneous Equipping and Concurrent Effects

Because the three God Items occupy three different equipment slots (Armor, Boots, Ring), the system SHALL allow all three to be equipped at the same time, and SHALL apply every equipped God Item's attribute bonuses, experience affix, combat passive, and non-combat bonuses concurrently.

#### Scenario: All three equipped at once

- **WHEN** Asprika, Sleipnir, and Megingjord are all equipped
- **THEN** Divine Bulwark, Windborne, Giant's Might, all three Sleipnir quality-of-life bonuses, all attribute bonuses, and the +40% experience affixes are all active together

#### Scenario: Bonus queried when the source item is not equipped

- **WHEN** a God Item's bonus value is queried but that item is not currently equipped
- **THEN** the reported value for that bonus is 0

### Requirement: Auto-Equip Protection

The system SHALL never allow the automatic loot-equip logic to replace an equipped God (Mythic) item with an item of any lower rarity, regardless of the incoming item's power score.

#### Scenario: Higher-power Legendary cannot displace a God Item

- **WHEN** a lower-rarity item (for example a Legendary) with a higher intrinsic power score is auto-equip evaluated against a slot holding a God Item
- **THEN** the God Item is kept and the incoming item is not equipped

### Requirement: God Item Acquisition

The system SHALL grant a God Item only through its dedicated forge action, which creates the item and places it directly into its equipment slot; no procedural drop, discovery roll, or player-facing unlock gate produces a God Item. The forge action SHALL refuse to re-create a God Item that is already equipped.

#### Scenario: Forging equips the God Item directly

- **WHEN** a God Item's forge action is invoked and that item is not already equipped
- **THEN** the God Item is created and placed into its slot (Asprika → Armor, Sleipnir → Boots, Megingjord → Ring)

#### Scenario: Forging an already-equipped God Item is refused

- **WHEN** a God Item's forge action is invoked while that same God Item is already equipped in its slot
- **THEN** the action makes no change and reports that the item is already equipped
