//! Combat simulation logic.

use rand::Rng;

/// Simulated player state for combat.
#[derive(Debug, Clone)]
pub struct SimPlayer {
    pub level: u32,
    pub max_hp: u32,
    pub current_hp: u32,
    pub damage: u32,
    pub defense: u32,
    pub crit_chance: f64,
    pub crit_multiplier: f64,
    pub attack_speed: f64, // attacks per second
}

impl SimPlayer {
    /// Create a new player at the given level with base stats.
    pub fn new(level: u32) -> Self {
        // Base stats scale with level
        let max_hp = level * 10 + 50;
        let damage = level + 5;

        Self {
            level,
            max_hp,
            current_hp: max_hp,
            damage,
            defense: 0,
            crit_chance: 0.05,
            crit_multiplier: 2.0,
            attack_speed: 1.0,
        }
    }

    /// Apply equipment bonuses (simplified).
    pub fn apply_gear_bonus(&mut self, damage_mult: f64, hp_mult: f64, crit_bonus: f64) {
        self.damage = ((self.damage as f64) * damage_mult) as u32;
        self.max_hp = ((self.max_hp as f64) * hp_mult) as u32;
        self.current_hp = self.max_hp;
        self.crit_chance = (self.crit_chance + crit_bonus).min(0.75);
    }

    /// Reset HP to full.
    pub fn heal_full(&mut self) {
        self.current_hp = self.max_hp;
    }

    /// Take damage, returns true if still alive.
    pub fn take_damage(&mut self, amount: u32) -> bool {
        let actual = amount.saturating_sub(self.defense);
        self.current_hp = self.current_hp.saturating_sub(actual);
        self.current_hp > 0
    }

    /// Calculate damage for one attack.
    pub fn calc_attack_damage(&self, rng: &mut impl Rng) -> u32 {
        let base = self.damage;
        if rng.gen::<f64>() < self.crit_chance {
            (base as f64 * self.crit_multiplier) as u32
        } else {
            base
        }
    }
}

/// Simulated monster.
#[derive(Debug, Clone)]
pub struct SimMonster {
    pub level: u32,
    pub max_hp: u32,
    pub current_hp: u32,
    pub damage: u32,
    pub is_boss: bool,
    pub xp_reward: u32,
}

impl SimMonster {
    /// Create a normal monster for the given zone level.
    pub fn normal(level: u32) -> Self {
        let max_hp = level * 10;
        let damage = level * 2;
        let xp_reward = level * 5;

        Self {
            level,
            max_hp,
            current_hp: max_hp,
            damage,
            is_boss: false,
            xp_reward,
        }
    }

    /// Create a boss monster (10x HP, 1.5x damage).
    pub fn boss(level: u32) -> Self {
        let max_hp = level * 10 * 10; // 10x HP
        let damage = (level * 2 * 3) / 2; // 1.5x damage
        let xp_reward = level * 50; // 10x XP

        Self {
            level,
            max_hp,
            current_hp: max_hp,
            damage,
            is_boss: true,
            xp_reward,
        }
    }

    /// Take damage, returns true if still alive.
    pub fn take_damage(&mut self, amount: u32) -> bool {
        self.current_hp = self.current_hp.saturating_sub(amount);
        self.current_hp > 0
    }
}

/// Result of a single combat encounter.
#[derive(Debug, Clone)]
pub struct CombatResult {
    pub player_won: bool,
    pub ticks_elapsed: u32,
    pub damage_dealt: u32,
    pub damage_taken: u32,
    pub xp_gained: u32,
    pub was_boss: bool,
}

/// Simulate a single combat encounter.
pub fn simulate_combat(
    player: &mut SimPlayer,
    monster: &mut SimMonster,
    rng: &mut impl Rng,
) -> CombatResult {
    let mut ticks = 0u32;
    let mut damage_dealt = 0u32;
    let mut damage_taken = 0u32;
    let initial_player_hp = player.current_hp;

    // Simple turn-based: player attacks, then monster attacks
    // Attack speed affects how many attacks per "tick"
    let player_attacks_per_tick = player.attack_speed;
    let _monster_attacks_per_tick = 1.0;

    loop {
        ticks += 1;

        // Player attacks (potentially multiple times based on speed)
        let num_attacks = if player_attacks_per_tick >= 1.0 {
            player_attacks_per_tick as u32
        } else if rng.gen::<f64>() < player_attacks_per_tick {
            1
        } else {
            0
        };

        for _ in 0..num_attacks {
            let dmg = player.calc_attack_damage(rng);
            damage_dealt += dmg;
            if !monster.take_damage(dmg) {
                // Monster died
                return CombatResult {
                    player_won: true,
                    ticks_elapsed: ticks,
                    damage_dealt,
                    damage_taken: initial_player_hp - player.current_hp,
                    xp_gained: monster.xp_reward,
                    was_boss: monster.is_boss,
                };
            }
        }

        // Monster attacks
        damage_taken += monster.damage;
        if !player.take_damage(monster.damage) {
            // Player died
            return CombatResult {
                player_won: false,
                ticks_elapsed: ticks,
                damage_dealt,
                damage_taken: initial_player_hp, // Lost all HP
                xp_gained: 0,
                was_boss: monster.is_boss,
            };
        }

        // Safety: prevent infinite loops
        if ticks > 10000 {
            return CombatResult {
                player_won: false,
                ticks_elapsed: ticks,
                damage_dealt,
                damage_taken,
                xp_gained: 0,
                was_boss: monster.is_boss,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_creation() {
        let player = SimPlayer::new(50);
        assert_eq!(player.level, 50);
        assert_eq!(player.max_hp, 550); // 50*10 + 50
        assert_eq!(player.damage, 55); // 50 + 5
    }

    #[test]
    fn test_monster_creation() {
        let monster = SimMonster::normal(50);
        assert_eq!(monster.max_hp, 500);
        assert_eq!(monster.damage, 100);
        assert!(!monster.is_boss);

        let boss = SimMonster::boss(50);
        assert_eq!(boss.max_hp, 5000);
        assert_eq!(boss.damage, 150);
        assert!(boss.is_boss);
    }

    #[test]
    fn test_combat_simulation() {
        let mut rng = rand::thread_rng();
        let mut player = SimPlayer::new(50);
        player.apply_gear_bonus(2.0, 1.5, 0.1); // Good gear

        let mut monster = SimMonster::normal(50);

        let result = simulate_combat(&mut player, &mut monster, &mut rng);
        assert!(
            result.player_won,
            "Geared player should beat same-level monster"
        );
    }
}
