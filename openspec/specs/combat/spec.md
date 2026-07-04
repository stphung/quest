# Combat Specification

## Purpose

Combat is the auto-battler core of the game: the hero fights the current enemy on a fixed timing cadence, dealing and taking damage through ordered, deterministic pipelines. This capability defines attack timing, the player-to-enemy and enemy-to-player damage/defense pipelines, enemy stat sourcing, boss encounters, post-kill regeneration, death handling, and the Zone 10 weapon gate. Enemy stats, item bonuses, prestige bonuses, and ascension multipliers enter combat only as inputs; their derivation belongs to other capabilities.

## Requirements

### Requirement: Combat Turn Cadence and State Progression

The system SHALL advance combat only when an enemy is present and the player is not regenerating. Each 100ms tick the player and enemy attack timers accumulate elapsed time; the player attacks when its timer reaches the effective player interval (base 1.5 seconds, shortened by attack-speed bonuses) and the enemy attacks when its timer reaches the effective enemy interval. Within a single tick the player's attack SHALL resolve before the enemy's attack, and a timer SHALL reset to zero when its owner attacks.

#### Scenario: No enemy present

- **WHEN** there is no current enemy and the player is not regenerating
- **THEN** no attacks occur and combat produces no damage events that tick

#### Scenario: Player timer reaches the interval

- **WHEN** the player attack timer reaches or exceeds the effective player interval (1.5 seconds at the base attack speed)
- **THEN** the player resolves an attack and the player attack timer resets to zero

#### Scenario: Player kill preempts the enemy attack

- **WHEN** the player's attack this tick reduces the enemy to zero HP and the enemy's timer was also ready
- **THEN** the enemy does not attack this tick and combat transitions to regeneration

### Requirement: Enemy Attack Interval by Enemy Tier

The system SHALL select the enemy's attack interval from the encounter tier: 2.0 seconds for a normal mob, 1.8 seconds for a subzone boss, 1.5 seconds for a zone (final-subzone) boss, 1.6 seconds for a dungeon elite, and 1.4 seconds for a dungeon boss. Dungeon room type SHALL take precedence, then overworld boss status, then the normal-mob default.

#### Scenario: Normal overworld mob

- **WHEN** the current enemy is an ordinary mob and no boss fight is active
- **THEN** the enemy attacks every 2.0 seconds

#### Scenario: Zone boss versus subzone boss

- **WHEN** a boss fight is active in the zone's final subzone
- **THEN** the enemy attacks every 1.5 seconds
- **AND** a boss fight in a non-final subzone attacks every 1.8 seconds

#### Scenario: Dungeon elite and boss rooms

- **WHEN** the active dungeon's current room is an Elite room
- **THEN** the enemy attacks every 1.6 seconds
- **AND** a Boss room enemy attacks every 1.4 seconds

### Requirement: Player-to-Enemy Damage Pipeline

The system SHALL compute player attack damage by applying these operations strictly in order: (1) start from the player's base damage; (2) multiply by (1 + Giant's Might percent / 100); (3) multiply by (1 + Haven damage percent / 100); (4) add prestige flat damage; (5) multiply by the ascension multiplier (default 1.0); (6) subtract the enemy's defense and floor the result at a minimum of 1; (7) if the attack is a critical hit, multiply by the critical multiplier (base 2.0). Intermediate results SHALL be truncated to whole numbers. The percent bonuses in steps 2 and 3 are two distinct multipliers applied at their own stages and SHALL NOT be summed.

#### Scenario: Ordered application of all stages

- **WHEN** a player attack resolves against an enemy with nonzero defense
- **THEN** the two percentage multipliers apply before the prestige flat addition, the ascension multiplier applies after the flat addition and before defense subtraction, and defense is subtracted last (before any crit)

#### Scenario: Defense floor guarantees minimum damage

- **WHEN** the enemy's defense is greater than or equal to the post-ascension damage
- **THEN** the dealt damage is floored to 1 before any critical multiplier

#### Scenario: Double strike lands a bonus hit

- **WHEN** the double-strike roll succeeds and the enemy survives the first hit
- **THEN** the same computed damage is applied a second time this attack, and only the first strike may carry the critical flag

### Requirement: Critical Hits

The system SHALL roll for a critical hit on each player attack using the player's base critical chance plus the bonus critical chance (which includes the prestige critical bonus, capped at 15 percentage points). A successful roll SHALL multiply the post-defense damage by the critical multiplier (base 2.0).

#### Scenario: Critical roll succeeds

- **WHEN** the rolled value falls under the total critical chance
- **THEN** the post-defense damage is multiplied by the critical multiplier (2x at base) and the hit is flagged as a critical

#### Scenario: Prestige critical bonus is capped

- **WHEN** prestige rank would grant more than 15 percentage points of critical chance
- **THEN** the prestige contribution to critical chance is limited to 15 percentage points

### Requirement: Enemy-to-Player Defense Pipeline

The system SHALL compute the damage the player takes by applying these operations strictly in order: (1) sum the player's defense and prestige flat defense; (2) multiply that total defense by the ascension multiplier (default 1.0); (3) subtract the total defense from the enemy's damage and floor the result at a minimum of 1; (4) if a Bulwark damage-reduction percent is active, multiply by (1 − reduction percent / 100) and floor again at a minimum of 1. The final value SHALL be subtracted from the player's current HP.

#### Scenario: Defense reduces incoming damage with a floor

- **WHEN** an enemy attack resolves and the scaled total defense is less than the enemy's damage
- **THEN** the player loses (enemy damage − total defense) HP, never less than 1

#### Scenario: Bulwark damage reduction applies after defense

- **WHEN** a Bulwark damage-reduction percentage is active
- **THEN** it is applied after defense subtraction and the result is floored at a minimum of 1 HP lost

#### Scenario: Overwhelming defense still lets one damage through

- **WHEN** the scaled total defense is greater than or equal to the enemy's damage
- **THEN** the player still loses at least 1 HP from the hit

### Requirement: Enemy Stat Sourcing from Static Zone Tables

The system SHALL source enemy HP, damage, and defense from a fixed 50-entry per-zone stat table rather than from the player's stats. Each entry provides a base value plus a per-subzone-depth step for HP, damage, and defense; the step is added once per subzone depth beyond the first. HP and damage SHALL receive a random variance between 0.9x and 1.1x; defense SHALL NOT receive variance. Fracture zones 12–30 SHALL be 1.6x stronger per zone above The Expanse (Zone 11) baseline, and Loom zones 31–50 SHALL be 1.25x stronger per zone above the Zone 30 baseline, with this scaling baked into the static table.

#### Scenario: Deeper subzone raises base stats

- **WHEN** an enemy spawns in a later subzone of a zone
- **THEN** its pre-variance HP, damage, and defense equal the zone base plus (subzone depth − 1) times the corresponding per-depth step

#### Scenario: Variance affects HP and damage only

- **WHEN** a zone enemy is generated
- **THEN** its HP and damage are scaled by an independent factor in the range 0.9x–1.1x
- **AND** its defense equals the computed base with no variance applied

#### Scenario: Fracture and Loom scaling is fixed per zone

- **WHEN** comparing consecutive fracture zones (12 through 30)
- **THEN** each zone's tabled base stats are 1.6x the previous zone's, and each Loom zone (31 through 50) is 1.25x the previous zone's

### Requirement: Boss Encounter Spawning and Stat Multipliers

The system SHALL spawn the subzone's named boss after 10 mob kills in the current subzone. Boss stats SHALL be the zone's base enemy stats scaled by tier multipliers applied as (HP, damage, defense): 3.0/1.5/1.8 for a subzone boss, 5.0/1.8/2.5 for a zone (final-subzone) boss, 2.2/1.5/1.6 for a dungeon elite, and 3.5/1.8/2.0 for a dungeon boss.

#### Scenario: Boss spawns after ten kills

- **WHEN** the player reaches 10 kills in the current subzone
- **THEN** the next encounter is the subzone's boss rather than a normal mob

#### Scenario: Zone boss uses the higher multipliers

- **WHEN** the boss is the zone's final-subzone (zone) boss
- **THEN** its stats are scaled by 5.0x HP, 1.8x damage, and 2.5x defense over the zone base

### Requirement: HP Regeneration After a Kill

The system SHALL enter a regeneration state immediately after any enemy is killed, restoring the player toward full HP over a base duration of 2.5 seconds. Regeneration SHALL take priority over new combat, and no new enemy attacks occur while regenerating. Regeneration bonuses MAY shorten the effective duration.

#### Scenario: Regeneration starts on kill

- **WHEN** an enemy dies
- **THEN** the current enemy is cleared and the player begins regenerating HP over 2.5 seconds (at the base rate)

#### Scenario: Regeneration completes to full HP

- **WHEN** the regeneration timer reaches the effective regeneration duration
- **THEN** the player's HP is set to maximum and the regeneration state ends

### Requirement: Player Death Handling

The system SHALL handle player death without any prestige loss and by restoring the player's HP to maximum. On death inside a dungeon the player SHALL exit the dungeon and the current enemy SHALL be cleared. On death to an overworld boss the player SHALL immediately retreat to the last safe zone (the highest zone with a defeated boss, defaulting to Zone 1) at subzone 1 with the subzone kill count reset to zero. On death to an overworld mob the player SHALL retry the same encounter at full HP, and only after 3 consecutive deaths SHALL retreat to the last safe zone.

#### Scenario: Death inside a dungeon

- **WHEN** the player's HP reaches zero while a dungeon is active
- **THEN** the dungeon is exited, the current enemy is cleared, HP is restored to maximum, and no prestige is lost

#### Scenario: Death to an overworld boss

- **WHEN** the player dies while fighting an overworld boss
- **THEN** the player retreats to the highest zone with a defeated boss at subzone 1, the subzone kill count resets to zero, and HP is restored to maximum with no prestige loss

#### Scenario: Death to an overworld mob retries then retreats

- **WHEN** the player dies to an ordinary overworld mob
- **THEN** the same enemy is reset to full HP and the fight continues, unless this is the third consecutive death, in which case the player retreats to the last safe zone

### Requirement: Boss Enrage Timeout

The system SHALL enrage a boss after 60 seconds of continuous boss combat, instantly defeating the player. On enrage the player SHALL be restored to full HP and retreat to subzone 1 of the current zone with the subzone kill count reset, and the encounter SHALL indicate whether the boss was weapon-gated.

#### Scenario: Boss fight exceeds the enrage timer

- **WHEN** a boss fight has been ongoing for at least 60 seconds
- **THEN** the boss enrages, the player is reset to full HP and returned to subzone 1 of the current zone, and combat ends without prestige loss

### Requirement: Zone 10 Final-Boss Weapon Gate

The system SHALL block the player's attacks against the Zone 10 (Storm Citadel) final-subzone boss until the player has forged Stormbreaker. While blocked, player attacks SHALL deal no damage and SHALL report that Stormbreaker is required. Non-final-subzone bosses and other zones SHALL NOT be weapon-gated.

#### Scenario: Attacking the gated Zone 10 boss without Stormbreaker

- **WHEN** the player attacks the Zone 10 final boss without having forged Stormbreaker
- **THEN** the attack is blocked, deals no damage, and reports that Stormbreaker is needed

#### Scenario: Stormbreaker unlocks the boss

- **WHEN** the player has forged Stormbreaker
- **THEN** attacks against the Zone 10 final boss proceed through the normal damage pipeline
