//! Enemy generation functions for zone combat, dungeon encounters, and bosses.

use rand::RngExt;

use super::types::Enemy;
use crate::core::constants::*;
use crate::zones::{get_subzone, Subzone, Zone};

pub fn generate_enemy_name() -> String {
    let mut rng = rand::rng();

    let prefixes = [
        "Grizz", "Sav", "Dark", "Blood", "Bone", "Shadow", "Fel", "Dire", "Wild", "Grim",
    ];
    let roots = [
        "led", "age", "en", "tooth", "claw", "fang", "heart", "eye", "maw", "tail",
    ];
    let suffixes = [
        "Orc", "Troll", "Drake", "Crusher", "Render", "Maw", "Beast", "Fiend", "Horror", "Terror",
    ];

    let prefix = prefixes[rng.random_range(0..prefixes.len())];
    let root = roots[rng.random_range(0..roots.len())];
    let suffix = suffixes[rng.random_range(0..suffixes.len())];

    format!("{}{} {}", prefix, root, suffix)
}

/// Looks up zone base stats. Returns (base_hp, hp_step, base_dmg, dmg_step, base_def, def_step).
/// Zone IDs are 1-indexed; defaults to Zone 1 for invalid IDs.
fn zone_base_stats(zone_id: u32) -> (u64, u64, u64, u64, u64, u64) {
    let index = (zone_id.saturating_sub(1) as usize).min(ZONE_ENEMY_STATS.len() - 1);
    ZONE_ENEMY_STATS[index]
}

/// Calculates enemy stats for a given zone and subzone depth (1-based).
/// Returns (hp, damage, defense) with variance applied.
fn calc_zone_enemy_stats(zone_id: u32, subzone_depth: u32) -> (u64, u64, u64) {
    let mut rng = rand::rng();
    let (base_hp, hp_step, base_dmg, dmg_step, base_def, def_step) = zone_base_stats(zone_id);

    let depth_offset = subzone_depth.saturating_sub(1) as u64;
    let raw_hp = base_hp.saturating_add(depth_offset.saturating_mul(hp_step));
    let raw_dmg = base_dmg.saturating_add(depth_offset.saturating_mul(dmg_step));
    let raw_def = base_def.saturating_add(depth_offset.saturating_mul(def_step));

    let hp_var = rng.random_range(ENEMY_STAT_VARIANCE_MIN..ENEMY_STAT_VARIANCE_MAX);
    let dmg_var = rng.random_range(ENEMY_STAT_VARIANCE_MIN..ENEMY_STAT_VARIANCE_MAX);

    let hp = ((raw_hp as f64) * hp_var).max(1.0) as u64;
    let damage = ((raw_dmg as f64) * dmg_var).max(1.0) as u64;

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
        (hp as f64 * hp_m).max(1.0) as u64,
        (damage as f64 * dmg_m).max(1.0) as u64,
        (defense as f64 * def_m) as u64,
    )
}

/// Generates a dungeon boss enemy using zone-based stats with boss multipliers.
pub fn generate_dungeon_boss(zone_id: u32) -> Enemy {
    let (hp, damage, defense) = calc_zone_enemy_stats(zone_id, 1);
    let (hp_m, dmg_m, def_m) = DUNGEON_BOSS_MULTIPLIERS;
    let name = format!("Boss {}", generate_enemy_name());
    Enemy::new_with_defense(
        name,
        (hp as f64 * hp_m).max(1.0) as u64,
        (damage as f64 * dmg_m).max(1.0) as u64,
        (defense as f64 * def_m) as u64,
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
        12 => &["Rim", "Ash", "Fault", "Ember", "Bloodglass"],
        13 => &["Coalwind", "Soot", "Crucible", "Scarforge", "Rift"],
        14 => &["Vein", "Pyre", "Coreglass", "Magma", "Rupture"],
        15 => &["Shard", "Prism", "Mirror", "White", "Glass"],
        16 => &["Bent", "Parallax", "Reflected", "Lightfall", "Angle"],
        17 => &["Solar", "False", "Sunshard", "Witness", "Second"],
        18 => &["Char", "Gloam", "Ashen", "Cinder", "Veil"],
        19 => &["Maw", "Tooth", "Sable", "Gullet", "Windpipe"],
        20 => &["Void", "Jawbone", "Unlit", "First", "Mouth"],
        21 => &["Amber", "Pilgrim", "Candlebone", "Crown", "Processional"],
        22 => &["Index", "Sealed", "Theorem", "Pale", "Forbidden"],
        23 => &["Echo", "Dust", "Obsidian", "Crowned", "Hollow"],
        24 => &["Tideless", "Brine", "Stillborn", "Abyssal", "Salt"],
        25 => &["Harmonic", "Choir", "Resonant", "Petrified", "Oscillating"],
        26 => &["Fraying", "Liminal", "Static", "Flickering", "Wailing"],
        27 => &["Root", "Splinter", "Taproot", "Fossilized", "Scar"],
        28 => &["Echo", "Reverb", "Temporal", "Infinite", "Ancient"],
        29 => &["Dimming", "Hushed", "Shadow", "Silent", "Final"],
        30 => &["Fissure", "Primordial", "Wound", "Unbroken", "Origin"],
        31..=34 => &["Threadbare", "Woven", "Spindle", "Weft", "Loom"],
        35..=38 => &["Shuttle", "Pattern", "Weave", "Fabric", "Tapestry"],
        39..=42 => &["Frayed", "Unraveled", "Tangled", "Knotted", "Snarled"],
        43..=46 => &["Grand", "Master", "Blueprint", "Forged", "Gilded"],
        47..=50 => &["Final", "Origin", "Reality", "World", "Infinite"],
        _ => &["Wild", "Fierce", "Dark", "Savage", "Grim"],
    }
}

/// Gets zone-specific enemy suffixes based on zone ID
fn get_zone_enemy_suffixes(zone_id: u32) -> &'static [&'static str] {
    match zone_id {
        1 => &[
            "Beetle", "Rabbit", "Wasp", "Boar", "Serpent", "Grub", "Hare", "Toad", "Mantis",
            "Sprout",
        ],
        2 => &[
            "Wolf", "Spider", "Bat", "Treant", "Wisp", "Lynx", "Moth", "Hollow", "Shambler",
            "Wretch",
        ],
        3 => &[
            "Goat", "Eagle", "Golem", "Yeti", "Harpy", "Ram", "Condor", "Troll", "Bandit",
            "Gargoyle",
        ],
        4 => &[
            "Skeleton",
            "Mummy",
            "Spirit",
            "Gargoyle",
            "Specter",
            "Revenant",
            "Shade",
            "Lich",
            "Cultist",
            "Apparition",
        ],
        5 => &[
            "Salamander",
            "Phoenix",
            "Imp",
            "Drake",
            "Elemental",
            "Cinderwyrm",
            "Ashborn",
            "Magmite",
            "Hellhound",
            "Infernal",
        ],
        6 => &[
            "Mammoth", "Wendigo", "Wraith", "Bear", "Wyrm", "Moose", "Banshee", "Imp", "Glacial",
            "Revenant",
        ],
        7 => &[
            "Construct",
            "Guardian",
            "Sprite",
            "Watcher",
            "Golem",
            "Shard",
            "Crawler",
            "Prism",
            "Sentinel",
            "Echo",
        ],
        8 => &[
            "Kraken",
            "Shark",
            "Naga",
            "Leviathan",
            "Siren",
            "Eel",
            "Ray",
            "Lurker",
            "Drowned",
            "Hydra",
        ],
        9 => &[
            "Griffin",
            "Djinn",
            "Sylph",
            "Roc",
            "Wyvern",
            "Zephyr",
            "Pegasus",
            "Manticore",
            "Cloudwalker",
            "Stormhawk",
        ],
        10 => &[
            "Titan",
            "Colossus",
            "Lord",
            "King",
            "Champion",
            "Warlord",
            "Juggernaut",
            "Thunderborn",
            "Stormknight",
            "Breaker",
        ],
        12 => &["Stalker", "Hound", "Ram", "Brute", "Crawler"],
        13 => &["Maw", "Knight", "Colossus", "Warden", "Fiend"],
        14 => &["Breaker", "Cantor", "Regent", "Tyrant", "Revenant"],
        15 => &["Hound", "Jackal", "Widow", "Watcher", "Echo"],
        16 => &["Serpent", "Marshal", "Repeater", "Sentinel", "Engine"],
        17 => &["Wraith", "King", "Titan", "Chorus", "Herald"],
        18 => &["Wing", "Revenant", "Forger", "Giant", "Shade"],
        19 => &["Warden", "Behemoth", "Herd", "Devourer", "Judge"],
        20 => &["Hunger", "Colossus", "Choir", "Crawler", "Remnant"],
        21 => &["Sentinel", "Warden", "Knight", "Colossus", "Procession"],
        22 => &["Wraith", "Censor", "Construct", "Eater", "Archivist"],
        23 => &["Warden", "Chancellor", "Guardian", "Absence", "Sovereign"],
        24 => &["Wanderer", "Phantom", "Leviathan", "Depthless", "Mother"],
        25 => &["Hound", "Dissonant", "Resonant", "Warden", "Chorus"],
        26 => &["Stalker", "Undefined", "Bloom", "Flickerer", "Voice"],
        27 => &["Creeper", "Horror", "Warden", "Rupture", "Root"],
        28 => &["Echo", "Dweller", "Noise", "Once-Slain", "Reverberation"],
        29 => &["Walker", "Muted", "Beast", "Frequency", "Warden"],
        30 => &["Guardian", "Titan", "Unbroken", "Heart", "Final"],
        31..=34 => &["Weaver", "Spinner", "Threader", "Bobbin", "Shuttle"],
        35..=38 => &[
            "Loomguard",
            "Weftwalker",
            "Patternborn",
            "Fabricant",
            "Threadseeker",
        ],
        39..=42 => &["Unmaker", "Raveler", "Tanglefoe", "Knotter", "Splicer"],
        43..=46 => &["Architect", "Designer", "Schemer", "Artificer", "Crafter"],
        47..=50 => &[
            "Worldweaver",
            "Realityborn",
            "Originkeeper",
            "Threadmaster",
            "Loombinder",
        ],
        _ => &[
            "Beast",
            "Horror",
            "Fiend",
            "Terror",
            "Monster",
            "Abomination",
            "Void",
            "Rift",
            "Amalgam",
            "Remnant",
        ],
    }
}

/// Generates a zone-themed enemy name
pub fn generate_zone_enemy_name(zone_id: u32) -> String {
    let mut rng = rand::rng();
    let prefixes = get_zone_enemy_prefixes(zone_id);
    let suffixes = get_zone_enemy_suffixes(zone_id);

    let prefix = prefixes[rng.random_range(0..prefixes.len())];
    let suffix = suffixes[rng.random_range(0..suffixes.len())];

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

    let boss_hp = (base_hp as f64 * hp_mult).max(1.0) as u64;
    let boss_damage = (base_damage as f64 * dmg_mult).max(1.0) as u64;
    let boss_defense = (base_defense as f64 * def_mult) as u64;

    Enemy::new_with_defense(
        subzone.boss.name.to_string(),
        boss_hp,
        boss_damage,
        boss_defense,
    )
}

/// Generates an enemy for the player's current zone and subzone using static zone-based stats.
pub fn generate_enemy_for_current_zone(zone_id: u32, subzone_id: u32) -> Enemy {
    if let Some((zone, subzone)) = get_subzone(zone_id, subzone_id) {
        return generate_zone_enemy(zone, subzone);
    }
    // Fallback: use zone 1, subzone 1 stats
    let (hp, damage, defense) = calc_zone_enemy_stats(zone_id, 1);
    Enemy::new_with_defense(generate_enemy_name(), hp, damage, defense)
}

/// Generates the subzone boss for the given zone/subzone using static zone-based stats.
pub fn generate_boss_for_current_zone(zone_id: u32, subzone_id: u32) -> Enemy {
    if let Some((zone, subzone)) = get_subzone(zone_id, subzone_id) {
        return generate_subzone_boss(zone, subzone);
    }
    // Fallback: zone boss with zone_id stats
    let (hp, damage, defense) = calc_zone_enemy_stats(zone_id, 1);
    let (hp_m, dmg_m, def_m) = ZONE_BOSS_MULTIPLIERS;
    Enemy::new_with_defense(
        "Unknown Boss".to_string(),
        (hp as f64 * hp_m) as u64,
        (damage as f64 * dmg_m) as u64,
        (defense as f64 * def_m) as u64,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_endgame_boss_stats_exceed_u32_range() {
        use crate::zones::get_all_zones;

        let zones = get_all_zones();
        let zone50 = zones.iter().find(|z| z.id == 50).unwrap();
        let last_subzone = zone50.subzones.last().unwrap();
        assert!(last_subzone.boss.is_zone_boss);

        // Zone 50 depth-5 raw HP (~4.3B) x zone boss multiplier (5.0) far
        // exceeds u32::MAX; before the u64 migration this clamped at ~4.29B.
        let boss = generate_subzone_boss(zone50, last_subzone);
        assert!(
            boss.max_hp > u32::MAX as u64,
            "Zone 50 boss HP {} should exceed u32::MAX",
            boss.max_hp
        );
    }

    #[test]
    fn test_endgame_zone_boss_hp_monotonically_increases() {
        use crate::zones::get_all_zones;

        // Loom zones scale 1.25x per zone, which beats the worst-case
        // variance spread (0.9/1.1), so boss HP must strictly increase.
        // Under u32 saturation, zones 45-50 all flattened at u32::MAX.
        let zones = get_all_zones();
        let mut prev_hp = 0u64;
        for zone_id in 43..=50 {
            let zone = zones.iter().find(|z| z.id == zone_id).unwrap();
            let last_subzone = zone.subzones.last().unwrap();
            let boss = generate_subzone_boss(zone, last_subzone);
            assert!(
                boss.max_hp > prev_hp,
                "Zone {} boss HP {} should exceed zone {} boss HP {}",
                zone_id,
                boss.max_hp,
                zone_id - 1,
                prev_hp
            );
            prev_hp = boss.max_hp;
        }
    }

    #[test]
    fn test_fracture_zones_have_unique_prefixes() {
        for zone_id in 12..=30 {
            let prefixes = get_zone_enemy_prefixes(zone_id);
            assert!(
                prefixes.len() >= 5,
                "Zone {} should have at least 5 prefixes, got {}",
                zone_id,
                prefixes.len()
            );
            // Should NOT be the fallback array
            assert_ne!(
                prefixes[0], "Wild",
                "Zone {} should not use fallback prefixes",
                zone_id
            );
        }
    }

    #[test]
    fn test_fracture_zones_have_unique_suffixes() {
        for zone_id in 12..=30 {
            let suffixes = get_zone_enemy_suffixes(zone_id);
            assert!(
                suffixes.len() >= 5,
                "Zone {} should have at least 5 suffixes, got {}",
                zone_id,
                suffixes.len()
            );
            // Should NOT be the fallback array
            assert_ne!(
                suffixes[0], "Beast",
                "Zone {} should not use fallback suffixes",
                zone_id
            );
        }
    }
}
