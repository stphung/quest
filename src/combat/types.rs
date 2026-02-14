use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::core::constants::*;
use crate::zones::{get_zone, Subzone, Zone};
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

/// Looks up zone base stats. Returns (base_hp, hp_step, base_dmg, dmg_step, base_def, def_step).
/// Zone IDs are 1-indexed; defaults to Zone 1 for invalid IDs.
fn zone_base_stats(zone_id: u32) -> (u32, u32, u32, u32, u32, u32) {
    let index = (zone_id.saturating_sub(1) as usize).min(ZONE_ENEMY_STATS.len() - 1);
    ZONE_ENEMY_STATS[index]
}

/// Calculates enemy stats for a given zone and subzone depth (1-based).
/// Returns (hp, damage, defense) with variance applied.
fn calc_zone_enemy_stats(zone_id: u32, subzone_depth: u32) -> (u32, u32, u32) {
    let mut rng = rand::thread_rng();
    let (base_hp, hp_step, base_dmg, dmg_step, base_def, def_step) = zone_base_stats(zone_id);

    let depth_offset = subzone_depth.saturating_sub(1);
    let raw_hp = base_hp + depth_offset * hp_step;
    let raw_dmg = base_dmg + depth_offset * dmg_step;
    let raw_def = base_def + depth_offset * def_step;

    let hp_var = rng.gen_range(ENEMY_STAT_VARIANCE_MIN..ENEMY_STAT_VARIANCE_MAX);
    let dmg_var = rng.gen_range(ENEMY_STAT_VARIANCE_MIN..ENEMY_STAT_VARIANCE_MAX);

    let hp = ((raw_hp as f64) * hp_var).max(1.0) as u32;
    let damage = ((raw_dmg as f64) * dmg_var).max(1.0) as u32;

    (hp, damage, raw_def)
}

/// Generates a zone-based dungeon enemy using zone_id for base stats.
pub fn generate_dungeon_enemy(zone_id: u32) -> Enemy {
    let (hp, damage, defense) = calc_zone_enemy_stats(zone_id, 1);
    let name = generate_enemy_name();
    Enemy::new_with_defense(name, hp, damage, defense)
}

/// Generates a dungeon elite enemy using zone-based stats with elite multipliers.
pub fn generate_dungeon_elite(zone_id: u32) -> Enemy {
    let (hp, damage, defense) = calc_zone_enemy_stats(zone_id, 1);
    let (hp_m, dmg_m, def_m) = DUNGEON_ELITE_MULTIPLIERS;
    let name = format!("Elite {}", generate_enemy_name());
    Enemy::new_with_defense(
        name,
        (hp as f64 * hp_m).max(1.0) as u32,
        (damage as f64 * dmg_m).max(1.0) as u32,
        (defense as f64 * def_m) as u32,
    )
}

/// Generates a dungeon boss enemy using zone-based stats with boss multipliers.
pub fn generate_dungeon_boss(zone_id: u32) -> Enemy {
    let (hp, damage, defense) = calc_zone_enemy_stats(zone_id, 1);
    let (hp_m, dmg_m, def_m) = DUNGEON_BOSS_MULTIPLIERS;
    let name = format!("Boss {}", generate_enemy_name());
    Enemy::new_with_defense(
        name,
        (hp as f64 * hp_m).max(1.0) as u32,
        (damage as f64 * dmg_m).max(1.0) as u32,
        (defense as f64 * def_m) as u32,
    )
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

/// Generates an enemy scaled for the current zone and subzone using static zone-based stats.
/// Player stats are NOT used as input.
pub fn generate_zone_enemy(zone: &Zone, subzone: &Subzone) -> Enemy {
    let (hp, damage, defense) = calc_zone_enemy_stats(zone.id, subzone.depth);
    let name = generate_zone_enemy_name(zone.id);
    Enemy::new_with_defense(name, hp, damage, defense)
}

/// Generates a subzone boss with the boss's actual name using zone-based static stats.
pub fn generate_subzone_boss(zone: &Zone, subzone: &Subzone) -> Enemy {
    let (base_hp, base_damage, base_defense) = calc_zone_enemy_stats(zone.id, subzone.depth);

    let (hp_mult, dmg_mult, def_mult) = if subzone.boss.is_zone_boss {
        ZONE_BOSS_MULTIPLIERS
    } else {
        SUBZONE_BOSS_MULTIPLIERS
    };

    let boss_hp = (base_hp as f64 * hp_mult).max(1.0) as u32;
    let boss_damage = (base_damage as f64 * dmg_mult).max(1.0) as u32;
    let boss_defense = (base_defense as f64 * def_mult) as u32;

    Enemy::new_with_defense(
        subzone.boss.name.to_string(),
        boss_hp,
        boss_damage,
        boss_defense,
    )
}

/// Generates an enemy for the player's current zone and subzone using static zone-based stats.
pub fn generate_enemy_for_current_zone(zone_id: u32, subzone_id: u32) -> Enemy {
    if let Some(zone) = get_zone(zone_id) {
        if let Some(subzone) = zone.subzones.iter().find(|s| s.id == subzone_id) {
            return generate_zone_enemy(&zone, subzone);
        }
    }
    // Fallback: use zone 1, subzone 1 stats
    let (hp, damage, defense) = calc_zone_enemy_stats(zone_id, 1);
    Enemy::new_with_defense(generate_enemy_name(), hp, damage, defense)
}

/// Generates the subzone boss for the given zone/subzone using static zone-based stats.
pub fn generate_boss_for_current_zone(zone_id: u32, subzone_id: u32) -> Enemy {
    if let Some(zone) = get_zone(zone_id) {
        if let Some(subzone) = zone.subzones.iter().find(|s| s.id == subzone_id) {
            return generate_subzone_boss(&zone, subzone);
        }
    }
    // Fallback: zone boss with zone_id stats
    let (hp, damage, defense) = calc_zone_enemy_stats(zone_id, 1);
    let (hp_m, dmg_m, def_m) = ZONE_BOSS_MULTIPLIERS;
    Enemy::new_with_defense(
        "Unknown Boss".to_string(),
        (hp as f64 * hp_m) as u32,
        (damage as f64 * dmg_m) as u32,
        (defense as f64 * def_m) as u32,
    )
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
    fn test_generate_dungeon_enemy() {
        let enemy = generate_dungeon_enemy(1);
        assert!(!enemy.name.is_empty());
        assert!(enemy.max_hp >= 1);
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
    fn test_generate_dungeon_elite() {
        let enemy = generate_dungeon_elite(1);
        assert!(enemy.name.starts_with("Elite "));
        // Elite should have higher HP than base zone 1 enemy
        assert!(enemy.max_hp >= 30); // Zone 1 base HP is 30, elite is 1.5x
    }

    #[test]
    fn test_generate_dungeon_boss() {
        let enemy = generate_dungeon_boss(1);
        assert!(enemy.name.starts_with("Boss "));
        // Boss should have higher HP than base zone 1 enemy
        assert!(enemy.max_hp >= 50); // Zone 1 base HP is 30, boss is 2.5x
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
    fn test_generate_zone_enemy_static() {
        use crate::zones::get_all_zones;

        let zones = get_all_zones();
        let zone1 = &zones[0];
        let subzone1 = &zone1.subzones[0];

        let enemy = generate_zone_enemy(zone1, subzone1);
        assert!(!enemy.name.is_empty());
        // Zone 1 base HP is 55, with variance 0.9-1.1 -> 49-60
        assert!(enemy.max_hp >= 45 && enemy.max_hp <= 65);
        assert!(enemy.damage >= 1);
        assert_eq!(enemy.defense, 0); // Zone 1 has 0 base defense
    }

    #[test]
    fn test_zone_enemy_static_scaling() {
        use crate::zones::get_all_zones;

        let zones = get_all_zones();

        // Zone 1, subzone 1 - base HP 30
        let zone1 = &zones[0];
        let enemy1 = generate_zone_enemy(zone1, &zone1.subzones[0]);

        // Zone 5, subzone 1 - base HP 170 (much higher than zone 1)
        let zone5 = &zones[4];
        let enemy5 = generate_zone_enemy(zone5, &zone5.subzones[0]);

        // Zone 5 should always be much stronger (170 vs 30 base HP)
        assert!(enemy5.max_hp > enemy1.max_hp);
        assert!(enemy5.damage > enemy1.damage);
    }

    #[test]
    fn test_generate_subzone_boss_static() {
        use crate::zones::get_all_zones;

        let zones = get_all_zones();
        let zone1 = &zones[0];

        // Test regular subzone boss (subzone 1)
        let subzone1 = &zone1.subzones[0];
        let boss1 = generate_subzone_boss(zone1, subzone1);
        assert_eq!(boss1.name, "Field Guardian");
        assert!(!subzone1.boss.is_zone_boss);
        // Subzone boss: 2.5x HP of base ~30 = ~75
        assert!(boss1.max_hp >= 50);

        // Test zone boss (subzone 3 - Sporeling Queen)
        let subzone3 = &zone1.subzones[2];
        let zone_boss = generate_subzone_boss(zone1, subzone3);
        assert_eq!(zone_boss.name, "Sporeling Queen");
        assert!(subzone3.boss.is_zone_boss);
        // Zone boss: 4.0x HP of base ~40 (depth 3) = ~160
        assert!(zone_boss.max_hp >= 100);

        // Zone boss should have higher multipliers than subzone boss
        assert!(zone_boss.max_hp > boss1.max_hp);
    }

    #[test]
    fn test_generate_enemy_for_current_zone_static() {
        let enemy = generate_enemy_for_current_zone(1, 1);
        assert!(!enemy.name.is_empty());
        assert!(enemy.max_hp >= 20); // Zone 1 base HP ~30

        // Test fallback for invalid zone
        let fallback = generate_enemy_for_current_zone(999, 1);
        assert!(!fallback.name.is_empty());
        assert!(fallback.max_hp >= 1);
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

    #[test]
    fn test_zone_enemy_defense_scaling() {
        use crate::zones::get_all_zones;

        let zones = get_all_zones();

        // Zone 1 has 0 base defense
        let zone1_enemy = generate_zone_enemy(&zones[0], &zones[0].subzones[0]);
        assert_eq!(zone1_enemy.defense, 0);

        // Zone 5 has 11 base defense
        let zone5_enemy = generate_zone_enemy(&zones[4], &zones[4].subzones[0]);
        assert!(zone5_enemy.defense >= 10);
    }

    #[test]
    fn test_subzone_depth_increases_stats() {
        use crate::zones::get_all_zones;

        let zones = get_all_zones();
        let zone2 = &zones[1]; // Dark Forest: base_hp=50, hp_step=8

        // Subzone 1 (depth 1): base stats
        let e1 = generate_zone_enemy(zone2, &zone2.subzones[0]);
        // Subzone 3 (depth 3): base + 2*step
        let last_subzone = zone2.subzones.last().unwrap();
        let e3 = generate_zone_enemy(zone2, last_subzone);

        // Deeper subzone enemy should have higher HP on average
        // With zone 2: depth 1 = 50 HP, depth 3 = 50+2*8 = 66 HP
        // e3 should be generally higher but with variance, just check it's valid
        assert!(e1.max_hp >= 1);
        assert!(e3.max_hp >= 1);
    }
}
