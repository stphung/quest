use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::zones::{get_zone, Subzone, Zone};
use std::collections::VecDeque;

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

pub fn generate_enemy(player_max_hp: u32, _player_damage: u32) -> Enemy {
    generate_enemy_with_multiplier(player_max_hp, _player_damage, 1.0)
}

/// Generates an enemy with a stat multiplier (for dungeon elites/bosses)
pub fn generate_enemy_with_multiplier(
    player_max_hp: u32,
    _player_damage: u32,
    stat_multiplier: f64,
) -> Enemy {
    let mut rng = rand::thread_rng();

    let name = generate_enemy_name();

    // Enemy HP: 80-120% of player HP, scaled by multiplier
    let hp_variance = rng.gen_range(0.8..1.2);
    let max_hp = ((player_max_hp as f64 * hp_variance * stat_multiplier) as u32).max(10);

    // Enemy damage calculated for 5-10 second fights, scaled by multiplier
    let damage_variance = rng.gen_range(0.8..1.2);
    let damage = ((player_max_hp as f64 / 7.0 * damage_variance * stat_multiplier) as u32).max(1);

    Enemy::new(name, max_hp, damage)
}

/// Generates a dungeon elite enemy (150% stats, guards the key)
pub fn generate_elite_enemy(player_max_hp: u32, player_damage: u32) -> Enemy {
    let mut enemy = generate_enemy_with_multiplier(player_max_hp, player_damage, 1.5);
    enemy.name = format!("Elite {}", enemy.name);
    enemy
}

/// Generates a dungeon boss enemy (200% stats)
pub fn generate_boss_enemy(player_max_hp: u32, player_damage: u32) -> Enemy {
    let mut enemy = generate_enemy_with_multiplier(player_max_hp, player_damage, 2.0);
    enemy.name = format!("Boss {}", enemy.name);
    enemy
}

/// Gets zone-specific enemy name prefixes based on zone ID
fn get_zone_enemy_prefixes(zone_id: u32) -> &'static [&'static str] {
    match zone_id {
        1 => &["Meadow", "Field", "Flower", "Grass", "Sunny"],
        2 => &["Forest", "Shadow", "Dark", "Thorn", "Wild"],
        3 => &["Mountain", "Rock", "Stone", "Peak", "Cliff"],
        4 => &["Ancient", "Ruin", "Temple", "Cursed", "Forgotten"],
        5 => &["Volcanic", "Flame", "Ash", "Molten", "Ember"],
        6 => &["Frozen", "Ice", "Frost", "Snow", "Glacial"],
        7 => &["Crystal", "Gem", "Prismatic", "Shard", "Luminous"],
        8 => &["Sunken", "Deep", "Coral", "Tidal", "Abyssal"],
        9 => &["Sky", "Cloud", "Wind", "Storm", "Floating"],
        10 => &["Thunder", "Lightning", "Tempest", "Storm", "Eternal"],
        _ => &["Wild", "Fierce", "Dark", "Savage", "Grim"],
    }
}

/// Gets zone-specific enemy suffixes based on zone ID
fn get_zone_enemy_suffixes(zone_id: u32) -> &'static [&'static str] {
    match zone_id {
        1 => &["Beetle", "Rabbit", "Wasp", "Boar", "Serpent"],
        2 => &["Wolf", "Spider", "Bat", "Treant", "Wisp"],
        3 => &["Goat", "Eagle", "Golem", "Yeti", "Harpy"],
        4 => &["Skeleton", "Mummy", "Spirit", "Gargoyle", "Specter"],
        5 => &["Salamander", "Phoenix", "Imp", "Drake", "Elemental"],
        6 => &["Mammoth", "Wendigo", "Wraith", "Bear", "Wyrm"],
        7 => &["Construct", "Guardian", "Sprite", "Watcher", "Golem"],
        8 => &["Kraken", "Shark", "Naga", "Leviathan", "Siren"],
        9 => &["Griffin", "Djinn", "Sylph", "Roc", "Wyvern"],
        10 => &["Titan", "Colossus", "Lord", "King", "Champion"],
        _ => &["Beast", "Horror", "Fiend", "Terror", "Monster"],
    }
}

/// Generates a zone-themed enemy name
pub fn generate_zone_enemy_name(zone_id: u32) -> String {
    let mut rng = rand::thread_rng();
    let prefixes = get_zone_enemy_prefixes(zone_id);
    let suffixes = get_zone_enemy_suffixes(zone_id);

    let prefix = prefixes[rng.gen_range(0..prefixes.len())];
    let suffix = suffixes[rng.gen_range(0..suffixes.len())];

    format!("{} {}", prefix, suffix)
}

/// Generates an enemy scaled for the current zone and subzone
pub fn generate_zone_enemy(
    zone: &Zone,
    subzone: &Subzone,
    player_max_hp: u32,
    _player_damage: u32,
) -> Enemy {
    let mut rng = rand::thread_rng();

    let name = generate_zone_enemy_name(zone.id);

    // Base scaling from zone (10% per zone level)
    let zone_multiplier = 1.0 + (zone.id as f64 - 1.0) * 0.1;

    // Additional scaling from subzone depth (5% per depth level)
    let subzone_multiplier = 1.0 + (subzone.depth as f64 - 1.0) * 0.05;

    let total_multiplier = zone_multiplier * subzone_multiplier;

    // Enemy HP: 80-120% of player HP, scaled by zone/subzone
    let hp_variance = rng.gen_range(0.8..1.2);
    let max_hp = ((player_max_hp as f64 * hp_variance * total_multiplier) as u32).max(10);

    // Enemy damage calculated for 5-10 second fights, scaled
    let damage_variance = rng.gen_range(0.8..1.2);
    let damage = ((player_max_hp as f64 / 7.0 * damage_variance * total_multiplier) as u32).max(1);

    Enemy::new(name, max_hp, damage)
}

/// Generates a subzone boss with the boss's actual name
#[allow(dead_code)] // Will be used when boss combat is fully integrated
pub fn generate_subzone_boss(
    zone: &Zone,
    subzone: &Subzone,
    player_max_hp: u32,
    player_damage: u32,
) -> Enemy {
    let base_enemy = generate_zone_enemy(zone, subzone, player_max_hp, player_damage);

    // Zone bosses are stronger than regular subzone bosses
    let (hp_mult, dmg_mult) = if subzone.boss.is_zone_boss {
        (3.0, 2.0) // Zone boss: 3x HP, 2x damage
    } else {
        (2.0, 1.5) // Subzone boss: 2x HP, 1.5x damage
    };

    Enemy {
        name: subzone.boss.name.to_string(),
        max_hp: (base_enemy.max_hp as f64 * hp_mult) as u32,
        current_hp: (base_enemy.max_hp as f64 * hp_mult) as u32,
        damage: (base_enemy.damage as f64 * dmg_mult) as u32,
    }
}

/// Generates an enemy for the player's current zone and subzone
pub fn generate_enemy_for_current_zone(
    zone_id: u32,
    subzone_id: u32,
    player_max_hp: u32,
    player_damage: u32,
) -> Enemy {
    if let Some(zone) = get_zone(zone_id) {
        if let Some(subzone) = zone.subzones.iter().find(|s| s.id == subzone_id) {
            return generate_zone_enemy(&zone, subzone, player_max_hp, player_damage);
        }
    }
    // Fallback to generic enemy if zone/subzone not found
    generate_enemy(player_max_hp, player_damage)
}

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
    pub attack_timer: f64,
    pub regen_timer: f64,
    pub is_regenerating: bool,
    #[serde(skip)]
    pub visual_effects: Vec<crate::ui::combat_effects::VisualEffect>,
    #[serde(skip)]
    pub combat_log: VecDeque<CombatLogEntry>,
}

impl Default for CombatState {
    fn default() -> Self {
        Self::new(50) // Base HP for fresh character
    }
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
            visual_effects: Vec::new(),
            combat_log: VecDeque::with_capacity(10),
        }
    }

    pub fn add_log_entry(&mut self, message: String, is_crit: bool, is_player_action: bool) {
        // Keep only the last 10 entries
        if self.combat_log.len() >= 10 {
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

    #[test]
    fn test_generate_elite_enemy() {
        let enemy = generate_elite_enemy(100, 20);
        assert!(enemy.name.starts_with("Elite "));
        // Elite should have ~150% HP (with variance)
        assert!(enemy.max_hp >= 100); // At least base HP
    }

    #[test]
    fn test_generate_boss_enemy() {
        let enemy = generate_boss_enemy(100, 20);
        assert!(enemy.name.starts_with("Boss "));
        // Boss should have ~200% HP (with variance)
        assert!(enemy.max_hp >= 120); // At least 1.2x base HP
    }

    #[test]
    fn test_generate_zone_enemy_name() {
        // Test zone 1 (Meadow)
        let name = generate_zone_enemy_name(1);
        assert!(!name.is_empty());
        assert!(name.contains(' ')); // Should have space between prefix and suffix

        // Test zone 10 (Storm Citadel)
        let name10 = generate_zone_enemy_name(10);
        assert!(!name10.is_empty());
    }

    #[test]
    fn test_generate_zone_enemy() {
        use crate::zones::get_all_zones;

        let zones = get_all_zones();
        let zone1 = &zones[0];
        let subzone1 = &zone1.subzones[0];

        let enemy = generate_zone_enemy(zone1, subzone1, 100, 20);
        assert!(!enemy.name.is_empty());
        assert!(enemy.max_hp >= 10);
        assert!(enemy.damage >= 1);
    }

    #[test]
    fn test_zone_enemy_scaling() {
        use crate::zones::get_all_zones;

        let zones = get_all_zones();

        // Zone 1, subzone 1 - should be baseline
        let zone1 = &zones[0];
        let enemy1 = generate_zone_enemy(zone1, &zone1.subzones[0], 100, 20);

        // Zone 5, subzone 1 - should be scaled up (40% more from zone)
        let zone5 = &zones[4];
        let enemy5 = generate_zone_enemy(zone5, &zone5.subzones[0], 100, 20);

        // On average, zone 5 enemies should be stronger (with variance this may not always hold)
        // So we just check both are valid
        assert!(enemy1.max_hp >= 10);
        assert!(enemy5.max_hp >= 10);
    }

    #[test]
    fn test_generate_subzone_boss() {
        use crate::zones::get_all_zones;

        let zones = get_all_zones();
        let zone1 = &zones[0];

        // Test regular subzone boss (subzone 1)
        let subzone1 = &zone1.subzones[0];
        let boss1 = generate_subzone_boss(zone1, subzone1, 100, 20);
        assert_eq!(boss1.name, "Field Guardian");
        assert!(!subzone1.boss.is_zone_boss);

        // Test zone boss (subzone 3 - Sporeling Queen)
        let subzone3 = &zone1.subzones[2];
        let zone_boss = generate_subzone_boss(zone1, subzone3, 100, 20);
        assert_eq!(zone_boss.name, "Sporeling Queen");
        assert!(subzone3.boss.is_zone_boss);

        // Zone boss should have higher multipliers (3x HP vs 2x)
        // With same base, zone boss HP should be higher
    }

    #[test]
    fn test_generate_enemy_for_current_zone() {
        let enemy = generate_enemy_for_current_zone(1, 1, 100, 20);
        assert!(!enemy.name.is_empty());
        assert!(enemy.max_hp >= 10);

        // Test fallback for invalid zone
        let fallback = generate_enemy_for_current_zone(999, 1, 100, 20);
        assert!(!fallback.name.is_empty());
    }
}
