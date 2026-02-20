use serde::{Deserialize, Serialize};

use crate::core::constants::*;
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enemy {
    pub name: String,
    pub max_hp: u32,
    pub current_hp: u32,
    pub damage: u32,
    #[serde(default)]
    pub defense: u32,
}

impl Enemy {
    #[allow(dead_code)]
    pub fn new(name: String, max_hp: u32, damage: u32) -> Self {
        Self {
            name,
            current_hp: max_hp,
            max_hp,
            damage,
            defense: 0,
        }
    }

    pub fn new_with_defense(name: String, max_hp: u32, damage: u32, defense: u32) -> Self {
        Self {
            name,
            current_hp: max_hp,
            max_hp,
            damage,
            defense,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.current_hp > 0
    }

    pub fn take_damage(&mut self, amount: u32) {
        self.current_hp = self.current_hp.saturating_sub(amount);
    }

    pub fn reset_hp(&mut self) {
        self.current_hp = self.max_hp;
    }
}

/// A brief damage number shown next to an HP bar.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DamageFlash {
    pub text: String,
    pub color: ratatui::style::Color,
    pub bold: bool,
    /// Remaining display time in seconds
    pub remaining: f64,
}

/// How long damage flashes display (seconds)
pub const DAMAGE_FLASH_DURATION: f64 = 0.8;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatLogEntry {
    pub message: String,
    pub is_crit: bool,
    pub is_player_action: bool,
}

/// Combat state for the player.
///
/// IMPORTANT: When adding new fields, use `#[serde(default)]` to maintain
/// backward compatibility with old save files. See test_minimal_v2_save_still_loads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatState {
    pub current_enemy: Option<Enemy>,
    pub player_current_hp: u32,
    pub player_max_hp: u32,
    /// Player's independent attack timer. Accumulates delta_time each tick.
    /// Player attacks when this reaches the effective player attack interval.
    #[serde(alias = "attack_timer")]
    pub player_attack_timer: f64,
    /// Enemy's independent attack timer. Accumulates delta_time each tick.
    /// Enemy attacks when this reaches the effective enemy attack interval.
    #[serde(default)]
    pub enemy_attack_timer: f64,
    pub regen_timer: f64,
    pub is_regenerating: bool,
    #[serde(skip)]
    pub visual_effects: Vec<crate::ui::combat_effects::VisualEffect>,
    #[serde(skip)]
    pub combat_log: VecDeque<CombatLogEntry>,
    #[serde(skip)]
    pub regen_start_hp: u32,
    #[serde(skip)]
    pub player_damage_floats: Vec<DamageFlash>,
    #[serde(skip)]
    pub enemy_damage_floats: Vec<DamageFlash>,
}

impl Default for CombatState {
    fn default() -> Self {
        Self::new(BASE_HP as u32)
    }
}

impl CombatState {
    pub fn new(player_max_hp: u32) -> Self {
        Self {
            current_enemy: None,
            player_current_hp: player_max_hp,
            player_max_hp,
            player_attack_timer: 0.0,
            enemy_attack_timer: 0.0,
            regen_timer: 0.0,
            is_regenerating: false,
            visual_effects: Vec::new(),
            combat_log: VecDeque::with_capacity(COMBAT_LOG_CAPACITY),
            regen_start_hp: 0,
            player_damage_floats: Vec::new(),
            enemy_damage_floats: Vec::new(),
        }
    }

    pub fn add_log_entry(&mut self, message: String, is_crit: bool, is_player_action: bool) {
        // Keep only the last 10 entries
        if self.combat_log.len() >= COMBAT_LOG_CAPACITY {
            self.combat_log.pop_front();
        }
        self.combat_log.push_back(CombatLogEntry {
            message,
            is_crit,
            is_player_action,
        });
    }

    pub fn update_max_hp(&mut self, new_max_hp: u32) {
        self.player_max_hp = new_max_hp;
        // If HP exceeds new max, cap it
        if self.player_current_hp > new_max_hp {
            self.player_current_hp = new_max_hp;
        }
    }

    pub fn is_player_alive(&self) -> bool {
        self.player_current_hp > 0
    }

    /// Decay HUD flash timers by delta_time. Removes expired floats.
    pub fn tick_hud(&mut self, delta_time: f64) {
        self.player_damage_floats.retain_mut(|f| {
            f.remaining -= delta_time;
            f.remaining > 0.0
        });
        self.enemy_damage_floats.retain_mut(|f| {
            f.remaining -= delta_time;
            f.remaining > 0.0
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enemy_creation() {
        let enemy = Enemy::new("Test Orc".to_string(), 50, 10);
        assert_eq!(enemy.name, "Test Orc");
        assert_eq!(enemy.max_hp, 50);
        assert_eq!(enemy.current_hp, 50);
        assert_eq!(enemy.damage, 10);
        assert!(enemy.is_alive());
    }

    #[test]
    fn test_enemy_take_damage() {
        let mut enemy = Enemy::new("Test Orc".to_string(), 50, 10);
        enemy.take_damage(20);
        assert_eq!(enemy.current_hp, 30);
        assert!(enemy.is_alive());

        enemy.take_damage(30);
        assert_eq!(enemy.current_hp, 0);
        assert!(!enemy.is_alive());
    }

    #[test]
    fn test_enemy_take_damage_no_underflow() {
        let mut enemy = Enemy::new("Test Orc".to_string(), 50, 10);
        enemy.take_damage(100);
        assert_eq!(enemy.current_hp, 0);
    }

    #[test]
    fn test_enemy_reset_hp() {
        let mut enemy = Enemy::new("Test Orc".to_string(), 50, 10);
        enemy.take_damage(40);
        assert_eq!(enemy.current_hp, 10);
        enemy.reset_hp();
        assert_eq!(enemy.current_hp, 50);
    }

    #[test]
    fn test_combat_state_creation() {
        let combat = CombatState::new(50);
        assert_eq!(combat.player_max_hp, 50);
        assert_eq!(combat.player_current_hp, 50);
        assert!(combat.is_player_alive());
        assert!(combat.current_enemy.is_none());
        assert!(!combat.is_regenerating);
    }

    #[test]
    fn test_combat_state_update_max_hp() {
        let mut combat = CombatState::new(50);
        combat.update_max_hp(70);
        assert_eq!(combat.player_max_hp, 70);
        assert_eq!(combat.player_current_hp, 50); // Current HP unchanged

        // Test capping when current > new max
        combat.player_current_hp = 80;
        combat.update_max_hp(60);
        assert_eq!(combat.player_current_hp, 60);
    }

    #[test]
    fn test_enemy_defense_field() {
        let enemy = Enemy::new_with_defense("Armored".to_string(), 100, 10, 5);
        assert_eq!(enemy.defense, 5);
        assert_eq!(enemy.max_hp, 100);
        assert_eq!(enemy.damage, 10);

        // Default constructor should set defense to 0
        let basic = Enemy::new("Basic".to_string(), 50, 5);
        assert_eq!(basic.defense, 0);
    }
}
