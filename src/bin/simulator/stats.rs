use quest::character::attributes::AttributeType;
use quest::core::game_state::GameState;
use quest::core::tick::{TickEvent, TickResult};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SimStats {
    pub total_ticks: u64,
    pub total_kills: u64,
    pub total_deaths: u64,
    pub total_boss_kills: u64,
    pub total_crits: u64,
    pub total_xp_gained: u64,
    pub level_at_tick: HashMap<u32, u64>,
    pub zone_entry_tick: HashMap<(u32, u32), u64>,
    pub zone_boss_defeated_tick: HashMap<(u32, u32), u64>,
    pub deaths_per_zone: HashMap<(u32, u32), u64>,
    pub items_by_rarity: [u64; 5],
    pub items_equipped: u64,
    pub boss_items_dropped: u64,
    pub fish_caught: u64,
    pub fishing_rank_ups: u64,
    pub dungeons_completed: u64,
    pub dungeons_failed: u64,
    pub dungeons_discovered: u64,
    pub achievements_unlocked: u64,
    pub haven_discovered: bool,
    pub haven_rooms_built: u32,
    pub haven_prestige_spent: u32,
    pub haven_final_tiers: Vec<(String, u8)>,
    pub pr_earned: u64,
    pub pr_spent: u64,
    pub ascension_level: u32,
    pub challenges_won: u64,
    pub stormglass_balance: u64,
    // Final state snapshot
    pub final_level: u32,
    pub final_xp: u64,
    pub final_prestige: u32,
    pub final_zone: (u32, u32),
    pub final_fishing_rank: u32,
    pub final_attributes: [u32; 6],
}

impl Default for SimStats {
    fn default() -> Self {
        Self {
            total_ticks: 0,
            total_kills: 0,
            total_deaths: 0,
            total_boss_kills: 0,
            total_crits: 0,
            total_xp_gained: 0,
            level_at_tick: HashMap::new(),
            zone_entry_tick: HashMap::new(),
            zone_boss_defeated_tick: HashMap::new(),
            deaths_per_zone: HashMap::new(),
            items_by_rarity: [0; 5],
            items_equipped: 0,
            boss_items_dropped: 0,
            fish_caught: 0,
            fishing_rank_ups: 0,
            dungeons_completed: 0,
            dungeons_failed: 0,
            dungeons_discovered: 0,
            achievements_unlocked: 0,
            haven_discovered: false,
            haven_rooms_built: 0,
            haven_prestige_spent: 0,
            haven_final_tiers: Vec::new(),
            pr_earned: 0,
            pr_spent: 0,
            ascension_level: 0,
            challenges_won: 0,
            stormglass_balance: 0,
            final_level: 1,
            final_xp: 0,
            final_prestige: 0,
            final_zone: (1, 1),
            final_fishing_rank: 1,
            final_attributes: [10; 6],
        }
    }
}

#[derive(Default)]
pub struct TickProfile {
    pub tick_count: u64,
    pub total_ns: u128,
    pub min_ns: u128,
    pub max_ns: u128,
}

impl TickProfile {
    pub fn record(&mut self, elapsed_ns: u128) {
        self.tick_count += 1;
        self.total_ns += elapsed_ns;
        if self.tick_count == 1 || elapsed_ns < self.min_ns {
            self.min_ns = elapsed_ns;
        }
        if elapsed_ns > self.max_ns {
            self.max_ns = elapsed_ns;
        }
    }

    pub fn avg_us(&self) -> f64 {
        if self.tick_count == 0 {
            return 0.0;
        }
        (self.total_ns as f64 / self.tick_count as f64) / 1000.0
    }

    pub fn min_us(&self) -> f64 {
        self.min_ns as f64 / 1000.0
    }

    pub fn max_us(&self) -> f64 {
        self.max_ns as f64 / 1000.0
    }
}

impl SimStats {
    pub fn record_zone_entry(&mut self, tick: u64, zone_id: u32, subzone_id: u32) {
        self.zone_entry_tick
            .entry((zone_id, subzone_id))
            .or_insert(tick);
    }

    pub fn process_tick(
        &mut self,
        tick: u64,
        result: &TickResult,
        _state: &GameState,
        current_zone: (u32, u32),
    ) {
        self.total_ticks = tick + 1;

        for event in &result.events {
            match event {
                TickEvent::EnemyDefeated { xp_gained, .. } => {
                    self.total_kills += 1;
                    self.total_xp_gained += xp_gained;
                }
                TickEvent::PlayerDied => {
                    self.total_deaths += 1;
                    *self.deaths_per_zone.entry(current_zone).or_insert(0) += 1;
                }
                TickEvent::PlayerDiedInDungeon => {
                    self.total_deaths += 1;
                }
                TickEvent::SubzoneBossDefeated { xp_gained, .. } => {
                    self.total_boss_kills += 1;
                    self.total_xp_gained += xp_gained;
                    self.zone_boss_defeated_tick
                        .entry(current_zone)
                        .or_insert(tick);
                }
                TickEvent::PlayerAttack { was_crit, .. } => {
                    if *was_crit {
                        self.total_crits += 1;
                    }
                }
                TickEvent::ItemDropped {
                    rarity,
                    equipped,
                    from_boss,
                    ..
                } => {
                    let idx = *rarity as usize;
                    if idx < 5 {
                        self.items_by_rarity[idx] += 1;
                    }
                    if *equipped {
                        self.items_equipped += 1;
                    }
                    if *from_boss {
                        self.boss_items_dropped += 1;
                    }
                }
                TickEvent::LeveledUp { new_level } => {
                    self.level_at_tick.entry(*new_level).or_insert(tick);
                }
                TickEvent::DungeonCompleted { .. } | TickEvent::DungeonBossDefeated { .. } => {
                    self.dungeons_completed += 1;
                }
                TickEvent::DungeonFailed => {
                    self.dungeons_failed += 1;
                }
                TickEvent::DungeonDiscovered { .. } => {
                    self.dungeons_discovered += 1;
                }
                TickEvent::FishCaught { .. } => {
                    self.fish_caught += 1;
                }
                TickEvent::FishingRankUp { .. } => {
                    self.fishing_rank_ups += 1;
                }
                TickEvent::AchievementUnlocked { .. } => {
                    self.achievements_unlocked += 1;
                }
                TickEvent::HavenDiscovered => {
                    self.haven_discovered = true;
                }
                _ => {}
            }
        }

        // Track XP from dungeon boss completions separately
        for event in &result.events {
            if let TickEvent::DungeonBossDefeated { total_xp, .. } = event {
                self.total_xp_gained += total_xp;
            }
        }
    }

    pub fn finalize(&mut self, state: &GameState) {
        self.final_level = state.character_level;
        self.final_xp = state.character_xp;
        self.final_prestige = state.prestige_rank;
        self.final_zone = (
            state.zone_progression.current_zone_id,
            state.zone_progression.current_subzone_id,
        );
        self.final_fishing_rank = state.fishing.rank;
        self.ascension_level = state.ascension_level;
        self.stormglass_balance = state.stormglass;

        for attr in AttributeType::all() {
            self.final_attributes[attr.index()] = state.attributes.get(attr);
        }
    }
}
