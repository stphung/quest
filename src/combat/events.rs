use crate::zones::BossDefeatResult;

/// Haven bonuses that affect combat
#[derive(Debug, Clone, Default)]
pub struct HavenCombatBonuses {
    /// Alchemy Lab: +% HP regen speed
    pub hp_regen_percent: f64,
    /// Bedroom: -% HP regen delay (reduces wait time before regen starts)
    pub hp_regen_delay_reduction: f64,
    /// Armory: +% damage
    pub damage_percent: f64,
    /// Watchtower: +% crit chance
    pub crit_chance_percent: f64,
    /// War Room: +% chance to strike twice
    pub double_strike_chance: f64,
    /// Training Yard: +% XP from kills
    pub xp_gain_percent: f64,
}

/// God item bonuses that affect combat
pub struct GodItemCombatBonuses {
    /// Asprika: Divine Bulwark damage reduction percent
    pub damage_reduction_percent: f64,
    /// Sleipnir: Windborne attack speed percent bonus
    pub attack_speed_percent: f64,
    /// Sleipnir: Swiftstrider regen duration reduction percent
    pub regen_reduction_percent: f64,
    /// Megingjord: Giant's Might damage percent bonus
    pub damage_percent: f64,
}

impl Default for GodItemCombatBonuses {
    fn default() -> Self {
        Self {
            damage_reduction_percent: 0.0,
            attack_speed_percent: 0.0,
            regen_reduction_percent: 0.0,
            damage_percent: 0.0,
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
}
