use crate::zones::BossDefeatResult;

/// Unified combat bonuses from all sources (Haven, god items, prestige).
///
/// The damage pipeline applies two separate damage% multipliers at different stages:
/// 1. `early_damage_percent` — applied to base damage first (e.g. Giant's Might)
/// 2. `damage_percent` — applied after early_damage_percent (e.g. Haven Armory)
///
/// Numeric fields default to 0 (no bonus).
#[derive(Debug, Clone, Copy)]
pub struct CombatBonuses {
    // --- Damage pipeline (player_attack.rs) ---
    /// +% base damage, applied first (e.g. Giant's Might 150%)
    pub early_damage_percent: f64,
    /// +% damage, applied after early_damage_percent (e.g. Haven Armory)
    pub damage_percent: f64,
    /// Flat damage added after % multipliers, before enemy defense (prestige)
    pub flat_damage: u32,
    /// +% crit chance (e.g. Haven Watchtower + prestige crit)
    pub crit_chance_percent: f64,
    /// +% chance to strike twice (e.g. Haven War Room)
    pub double_strike_chance: f64,
    /// +% XP from kills (e.g. Haven Training Yard)
    pub xp_gain_percent: f64,

    // --- Defense pipeline (enemy_attack.rs) ---
    /// Flat defense added to derived defense (prestige)
    pub flat_defense: u32,
    /// Damage reduction % applied after defense subtraction (e.g. Divine Bulwark 30%)
    pub damage_reduction_percent: f64,

    // --- Attack speed (orchestration.rs) ---
    /// +% attack speed (e.g. Windborne 100%)
    pub attack_speed_percent: f64,

    // --- Regeneration (regen.rs) ---
    /// +% HP regen speed (e.g. Haven Alchemy Lab)
    pub hp_regen_percent: f64,
    /// -% HP regen delay (e.g. Haven Bedroom)
    pub hp_regen_delay_reduction: f64,
    /// -% regen duration (e.g. Sleipnir Swiftstrider)
    pub regen_reduction_percent: f64,

    // --- Ascension multiplier (player_attack.rs, enemy_attack.rs) ---
    /// Ascension combat multiplier applied to damage, defense, and HP.
    /// Defaults to 1.0 (no ascension).
    pub ascension_multiplier: f64,
}

impl Default for CombatBonuses {
    fn default() -> Self {
        Self {
            early_damage_percent: 0.0,
            damage_percent: 0.0,
            flat_damage: 0,
            crit_chance_percent: 0.0,
            double_strike_chance: 0.0,
            xp_gain_percent: 0.0,
            flat_defense: 0,
            damage_reduction_percent: 0.0,
            attack_speed_percent: 0.0,
            hp_regen_percent: 0.0,
            hp_regen_delay_reduction: 0.0,
            regen_reduction_percent: 0.0,
            ascension_multiplier: 1.0,
        }
    }
}

pub enum CombatEvent {
    PlayerAttack {
        damage: u32,
        was_crit: bool,
    },
    /// Player's attack was blocked because boss requires a weapon
    PlayerAttackBlocked {
        weapon_needed: String,
    },
    EnemyAttack {
        damage: u32,
    },
    /// Damage reflected back to the enemy when they hit the player
    DamageReflected {
        damage: u32,
    },
    PlayerDied,
    /// Player died while in a dungeon (no prestige loss)
    PlayerDiedInDungeon,
    EnemyDied {
        xp_gained: u64,
    },
    /// Elite enemy defeated in dungeon (player gets key)
    EliteDefeated {
        xp_gained: u64,
    },
    /// Boss enemy defeated in dungeon (dungeon complete)
    BossDefeated {
        xp_gained: u64,
    },
    /// HP regen completed after a kill
    RegenComplete {
        healed: u32,
    },
    /// Boss enraged after fight timer expired — instant kill.
    /// If weapon_blocked, player retreats to subzone 1 of the current zone.
    BossEnrage {
        weapon_blocked: bool,
        enemy_name: String,
    },
    /// Subzone boss defeated (zone progression)
    SubzoneBossDefeated {
        xp_gained: u64,
        result: BossDefeatResult,
    },
    /// Player was overwhelmed and auto-retreated to a safe zone.
    CombatRetreat {
        zone_name: String,
    },
}
