//! Combat simulation using real game mechanics.

use crate::character::attributes::Attributes;
use crate::character::derived_stats::DerivedStats;
use crate::combat::types::{generate_enemy_for_current_zone, generate_subzone_boss, Enemy};
use crate::items::Equipment;
use crate::zones::get_zone;
use rand::Rng;

/// Simulated player state for combat.
/// Uses real game DerivedStats calculation.
#[derive(Debug, Clone)]
pub struct SimPlayer {
    pub level: u32,
    pub attributes: Attributes,
    pub equipment: Equipment,
    pub current_hp: u32,
    // Cached derived stats
    derived: DerivedStats,
}

impl SimPlayer {
    /// Create a new player at level 1 with base stats.
    pub fn new() -> Self {
        let attributes = Attributes::default();
        let equipment = Equipment::new();
        let derived = DerivedStats::calculate_derived_stats(&attributes, &equipment);

        Self {
            level: 1,
            attributes,
            equipment,
            current_hp: derived.max_hp,
            derived,
        }
    }

    /// Create a player at a specific level with scaled attributes.
    pub fn at_level(level: u32) -> Self {
        let mut player = Self::new();

        // Scale base attributes with level (simplified)
        // In real game this comes from leveling up
        let bonus = level.saturating_sub(1);
        player.attributes.set(
            crate::character::attributes::AttributeType::Strength,
            10 + bonus / 2,
        );
        player.attributes.set(
            crate::character::attributes::AttributeType::Constitution,
            10 + bonus / 2,
        );
        player.attributes.set(
            crate::character::attributes::AttributeType::Dexterity,
            10 + bonus / 3,
        );

        player.level = level;
        player.recalculate_stats();
        player
    }

    /// Recalculate derived stats after equipment/attribute changes.
    pub fn recalculate_stats(&mut self) {
        self.derived = DerivedStats::calculate_derived_stats(&self.attributes, &self.equipment);
        // Don't reset HP - keep current percentage
        let hp_percent = self.current_hp as f64 / self.derived.max_hp.max(1) as f64;
        self.current_hp = (self.derived.max_hp as f64 * hp_percent.min(1.0)) as u32;
    }

    /// Equip an item if it's an upgrade.
    pub fn equip(&mut self, item: crate::items::Item) {
        self.equipment.set(item.slot, Some(item));
        self.recalculate_stats();
    }

    /// Get derived stats.
    pub fn stats(&self) -> &DerivedStats {
        &self.derived
    }

    /// Heal to full HP.
    pub fn heal_full(&mut self) {
        self.current_hp = self.derived.max_hp;
    }

    /// Take damage, returns true if still alive.
    pub fn take_damage(&mut self, amount: u32) -> bool {
        let actual = amount.saturating_sub(self.derived.defense);
        self.current_hp = self.current_hp.saturating_sub(actual);
        self.current_hp > 0
    }

    /// Calculate damage for one attack (using real crit mechanics).
    pub fn calc_attack_damage(&self, rng: &mut impl Rng) -> (u32, bool) {
        let base = self.derived.total_damage();
        let crit_roll = rng.gen_range(0..100);
        let is_crit = crit_roll < self.derived.crit_chance_percent;

        let damage = if is_crit {
            (base as f64 * self.derived.crit_multiplier) as u32
        } else {
            base
        };

        (damage, is_crit)
    }
}

impl Default for SimPlayer {
    fn default() -> Self {
        Self::new()
    }
}

/// Wrapper around real Enemy for simulation.
#[derive(Debug, Clone)]
pub struct SimEnemy {
    pub inner: Enemy,
    pub is_boss: bool,
    pub xp_reward: u32,
}

impl SimEnemy {
    /// Generate a normal enemy for the given zone using real game logic.
    pub fn for_zone(zone_id: u32, subzone_id: u32, player: &SimPlayer) -> Self {
        let enemy = generate_enemy_for_current_zone(
            zone_id,
            subzone_id,
            player.stats().max_hp,
            player.stats().total_damage(),
        );

        // XP scales with zone
        let xp_reward = 10 + zone_id * 5 + subzone_id * 2;

        Self {
            inner: enemy,
            is_boss: false,
            xp_reward,
        }
    }

    /// Generate a boss enemy using real game logic.
    pub fn boss_for_zone(zone_id: u32, subzone_id: u32, player: &SimPlayer) -> Self {
        if let Some(zone) = get_zone(zone_id) {
            if let Some(subzone) = zone.subzones.iter().find(|s| s.id == subzone_id) {
                let enemy = generate_subzone_boss(
                    &zone,
                    subzone,
                    player.stats().max_hp,
                    player.stats().total_damage(),
                );

                // Boss XP is much higher
                let xp_reward = (10 + zone_id * 5) * 10;

                return Self {
                    inner: enemy,
                    is_boss: true,
                    xp_reward,
                };
            }
        }

        // Fallback
        let mut normal = Self::for_zone(zone_id, subzone_id, player);
        normal.inner.max_hp *= 3;
        normal.inner.current_hp *= 3;
        normal.inner.damage *= 2;
        normal.is_boss = true;
        normal.xp_reward *= 10;
        normal
    }

    /// Take damage, returns true if still alive.
    pub fn take_damage(&mut self, amount: u32) -> bool {
        self.inner.take_damage(amount);
        self.inner.current_hp > 0
    }
}

/// Result of a single combat encounter.
#[derive(Debug, Clone)]
pub struct CombatResult {
    pub player_won: bool,
    pub ticks_elapsed: u32,
    pub damage_dealt: u32,
    pub damage_taken: u32,
    pub crits_landed: u32,
    pub xp_gained: u32,
    pub was_boss: bool,
}

/// Simulate a single combat encounter using real game mechanics.
pub fn simulate_combat(
    player: &mut SimPlayer,
    enemy: &mut SimEnemy,
    rng: &mut impl Rng,
) -> CombatResult {
    let mut ticks = 0u32;
    let mut damage_dealt = 0u32;
    let mut damage_taken = 0u32;
    let mut crits_landed = 0u32;
    let initial_player_hp = player.current_hp;

    // Attack speed determines ticks between attacks
    // Base: 1 attack per tick, speed multiplier affects this
    let player_speed = player.stats().attack_speed_multiplier;
    let mut player_attack_accumulator = 0.0;

    loop {
        ticks += 1;

        // Player attacks based on attack speed
        player_attack_accumulator += player_speed;
        while player_attack_accumulator >= 1.0 {
            player_attack_accumulator -= 1.0;

            let (dmg, was_crit) = player.calc_attack_damage(rng);
            damage_dealt += dmg;
            if was_crit {
                crits_landed += 1;
            }

            if !enemy.take_damage(dmg) {
                // Enemy died
                return CombatResult {
                    player_won: true,
                    ticks_elapsed: ticks,
                    damage_dealt,
                    damage_taken: initial_player_hp.saturating_sub(player.current_hp),
                    crits_landed,
                    xp_gained: enemy.xp_reward,
                    was_boss: enemy.is_boss,
                };
            }
        }

        // Enemy attacks (once per tick)
        let enemy_dmg = enemy.inner.damage;
        damage_taken += enemy_dmg;
        if !player.take_damage(enemy_dmg) {
            // Player died
            return CombatResult {
                player_won: false,
                ticks_elapsed: ticks,
                damage_dealt,
                damage_taken: initial_player_hp,
                crits_landed,
                xp_gained: 0,
                was_boss: enemy.is_boss,
            };
        }

        // Safety: prevent infinite loops
        if ticks > 10000 {
            return CombatResult {
                player_won: false,
                ticks_elapsed: ticks,
                damage_dealt,
                damage_taken,
                crits_landed,
                xp_gained: 0,
                was_boss: enemy.is_boss,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_creation() {
        let player = SimPlayer::new();
        assert_eq!(player.level, 1);
        assert!(player.stats().max_hp > 0);
        assert!(player.stats().total_damage() > 0);
    }

    #[test]
    fn test_player_at_level() {
        let player = SimPlayer::at_level(50);
        assert_eq!(player.level, 50);
        // Higher level should have higher stats
        let lvl1 = SimPlayer::new();
        assert!(player.stats().max_hp > lvl1.stats().max_hp);
    }

    #[test]
    fn test_enemy_generation() {
        let player = SimPlayer::at_level(20);
        let enemy = SimEnemy::for_zone(1, 1, &player);

        // Enemy should have reasonable HP relative to player
        assert!(enemy.inner.max_hp > 0);
        assert!(enemy.inner.damage > 0);
    }

    #[test]
    fn test_boss_stronger_than_normal() {
        let player = SimPlayer::at_level(20);
        let normal = SimEnemy::for_zone(1, 1, &player);
        let boss = SimEnemy::boss_for_zone(1, 1, &player);

        assert!(boss.inner.max_hp > normal.inner.max_hp);
        assert!(boss.is_boss);
    }

    #[test]
    fn test_combat_simulation() {
        let mut rng = rand::thread_rng();
        let mut player = SimPlayer::at_level(30);
        let mut enemy = SimEnemy::for_zone(1, 1, &player);

        let result = simulate_combat(&mut player, &mut enemy, &mut rng);

        // Combat should complete
        assert!(result.ticks_elapsed > 0);
        assert!(result.damage_dealt > 0);
    }
}
