use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enemy {
    pub name: String,
    pub max_hp: u32,
    pub current_hp: u32,
    pub damage: u32,
}

impl Enemy {
    pub fn new(name: String, max_hp: u32, damage: u32) -> Self {
        Self {
            name,
            current_hp: max_hp,
            max_hp,
            damage,
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

pub fn generate_enemy_name() -> String {
    let mut rng = rand::thread_rng();

    let prefixes = [
        "Grizz", "Sav", "Dark", "Blood", "Bone", "Shadow", "Fel", "Dire", "Wild", "Grim",
    ];
    let roots = [
        "led", "age", "en", "tooth", "claw", "fang", "heart", "eye", "maw", "tail",
    ];
    let suffixes = [
        "Orc", "Troll", "Drake", "Crusher", "Render", "Maw", "Beast", "Fiend", "Horror", "Terror",
    ];

    let prefix = prefixes[rng.gen_range(0..prefixes.len())];
    let root = roots[rng.gen_range(0..roots.len())];
    let suffix = suffixes[rng.gen_range(0..suffixes.len())];

    format!("{}{} {}", prefix, root, suffix)
}

pub fn generate_enemy(player_max_hp: u32, player_damage: u32) -> Enemy {
    let mut rng = rand::thread_rng();

    let name = generate_enemy_name();

    // Enemy HP: 80-120% of player HP
    let hp_variance = rng.gen_range(0.8..1.2);
    let max_hp = ((player_max_hp as f64 * hp_variance) as u32).max(10);

    // Enemy damage calculated for 5-10 second fights
    // Assuming 1.5s attack interval = ~3-7 attacks
    // Want player to take max_hp / 6-8 hits to die
    let damage_variance = rng.gen_range(0.8..1.2);
    let damage = ((player_max_hp as f64 / 7.0 * damage_variance) as u32).max(1);

    Enemy::new(name, max_hp, damage)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatState {
    pub current_enemy: Option<Enemy>,
    pub player_current_hp: u32,
    pub player_max_hp: u32,
    pub attack_timer: f64,
    pub regen_timer: f64,
    pub is_regenerating: bool,
}

impl CombatState {
    pub fn new(player_max_hp: u32) -> Self {
        Self {
            current_enemy: None,
            player_current_hp: player_max_hp,
            player_max_hp,
            attack_timer: 0.0,
            regen_timer: 0.0,
            is_regenerating: false,
        }
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
    fn test_generate_enemy_name() {
        let name = generate_enemy_name();
        assert!(!name.is_empty());
        assert!(name.contains(' ')); // Should have space between parts
    }

    #[test]
    fn test_generate_enemy() {
        let enemy = generate_enemy(50, 10);
        assert!(!enemy.name.is_empty());
        assert!(enemy.max_hp >= 10);
        assert!(enemy.damage >= 1);
        assert_eq!(enemy.current_hp, enemy.max_hp);
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
}
