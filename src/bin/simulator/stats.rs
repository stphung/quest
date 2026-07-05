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
    pub combat_retreats: u64,
    pub retreats_per_zone: HashMap<(u32, u32), u64>,
    pub boss_enrages: u64,
    pub enrages_per_zone: HashMap<(u32, u32), u64>,
    pub frontier_backoffs: u64,
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
    pub prestiges_performed: u32,
    pub ascension_level: u32,
    pub challenges_won: u64,
    pub stormglass_balance: u64,
    pub deep_layers_reached: u32,
    pub fracture_zone_cap: u32,
    pub loom_patterns_completed: usize,
    pub loom_zone_cap: u32,
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
            combat_retreats: 0,
            retreats_per_zone: HashMap::new(),
            boss_enrages: 0,
            enrages_per_zone: HashMap::new(),
            frontier_backoffs: 0,
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
            prestiges_performed: 0,
            ascension_level: 0,
            challenges_won: 0,
            stormglass_balance: 0,
            deep_layers_reached: 0,
            fracture_zone_cap: 11,
            loom_patterns_completed: 0,
            loom_zone_cap: 0,
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
                TickEvent::SubzoneBossDefeated { xp_gained, result } => {
                    self.total_boss_kills += 1;
                    self.total_xp_gained += xp_gained;
                    self.zone_boss_defeated_tick
                        .entry(current_zone)
                        .or_insert(tick);
                    if matches!(
                        result,
                        quest::zones::BossDefeatResult::FrontierBackoff { .. }
                    ) {
                        self.frontier_backoffs += 1;
                    }
                }
                TickEvent::CombatRetreat { .. } => {
                    self.combat_retreats += 1;
                    *self.retreats_per_zone.entry(current_zone).or_insert(0) += 1;
                }
                TickEvent::BossEnrage { .. } => {
                    self.boss_enrages += 1;
                    *self.enrages_per_zone.entry(current_zone).or_insert(0) += 1;
                }
                TickEvent::PlayerAttack { was_crit, .. } if *was_crit => {
                    self.total_crits += 1;
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

    pub fn finalize(
        &mut self,
        state: &GameState,
        deep: &quest::deep::DeepState,
        loom: &quest::loom::LoomState,
    ) {
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
        self.deep_layers_reached = deep.persistent.deepest_layer_reached;
        self.fracture_zone_cap = deep.persistent.fracture_zone_cap;
        self.loom_patterns_completed = loom.persistent.completed_pattern_count();
        self.loom_zone_cap = quest::loom::loom_zone_cap_for_patterns(self.loom_patterns_completed);

        for attr in AttributeType::all() {
            self.final_attributes[attr.index()] = state.attributes.get(attr);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quest::items::types::Rarity;
    use quest::zones::BossDefeatResult;

    fn tick_result(events: Vec<TickEvent>) -> TickResult {
        TickResult {
            events,
            ..Default::default()
        }
    }

    #[test]
    fn record_zone_entry_keeps_earliest_tick() {
        let mut stats = SimStats::default();
        stats.record_zone_entry(50, 2, 1);
        stats.record_zone_entry(10, 2, 1);
        assert_eq!(stats.zone_entry_tick[&(2, 1)], 50);
    }

    #[test]
    fn process_tick_counts_kills_and_xp() {
        let mut stats = SimStats::default();
        let state = GameState::new("Sim".to_string(), 0);
        let result = tick_result(vec![TickEvent::EnemyDefeated {
            xp_gained: 300,
            enemy_name: "Rat".to_string(),
        }]);
        stats.process_tick(5, &result, &state, (1, 1));
        assert_eq!(stats.total_ticks, 6);
        assert_eq!(stats.total_kills, 1);
        assert_eq!(stats.total_xp_gained, 300);
    }

    #[test]
    fn process_tick_tracks_deaths_per_zone() {
        let mut stats = SimStats::default();
        let state = GameState::new("Sim".to_string(), 0);
        let result = tick_result(vec![TickEvent::PlayerDied]);
        stats.process_tick(1, &result, &state, (3, 2));
        assert_eq!(stats.total_deaths, 1);
        assert_eq!(stats.deaths_per_zone[&(3, 2)], 1);

        let result = tick_result(vec![TickEvent::PlayerDiedInDungeon]);
        stats.process_tick(2, &result, &state, (3, 2));
        assert_eq!(stats.total_deaths, 2);
        // Dungeon deaths are not attributed to an overworld zone.
        assert_eq!(stats.deaths_per_zone[&(3, 2)], 1);
    }

    #[test]
    fn process_tick_records_boss_kills_and_frontier_backoffs() {
        let mut stats = SimStats::default();
        let state = GameState::new("Sim".to_string(), 0);
        let result = tick_result(vec![TickEvent::SubzoneBossDefeated {
            xp_gained: 1000,
            result: BossDefeatResult::FrontierBackoff {
                zone_id: 1,
                blocked_zone_id: 2,
            },
        }]);
        stats.process_tick(3, &result, &state, (1, 1));
        assert_eq!(stats.total_boss_kills, 1);
        assert_eq!(stats.total_xp_gained, 1000);
        assert_eq!(stats.frontier_backoffs, 1);
        assert_eq!(stats.zone_boss_defeated_tick[&(1, 1)], 3);
    }

    #[test]
    fn process_tick_counts_retreats_enrages_and_crits() {
        let mut stats = SimStats::default();
        let state = GameState::new("Sim".to_string(), 0);
        let result = tick_result(vec![
            TickEvent::CombatRetreat {
                zone_name: "Z".to_string(),
            },
            TickEvent::BossEnrage {
                message: "enraged".to_string(),
            },
            TickEvent::PlayerAttack {
                damage: 10,
                was_crit: true,
            },
            TickEvent::PlayerAttack {
                damage: 5,
                was_crit: false,
            },
        ]);
        stats.process_tick(1, &result, &state, (2, 1));
        assert_eq!(stats.combat_retreats, 1);
        assert_eq!(stats.retreats_per_zone[&(2, 1)], 1);
        assert_eq!(stats.boss_enrages, 1);
        assert_eq!(stats.enrages_per_zone[&(2, 1)], 1);
        assert_eq!(stats.total_crits, 1);
    }

    #[test]
    fn process_tick_tracks_item_drops_by_rarity_and_flags() {
        let mut stats = SimStats::default();
        let state = GameState::new("Sim".to_string(), 0);
        let result = tick_result(vec![TickEvent::ItemDropped {
            item_name: "Sword".to_string(),
            rarity: Rarity::Epic,
            tier: 5,
            ilvl: 50,
            power: 100,
            equipped: true,
            slot: "Weapon".to_string(),
            stats: "+STR".to_string(),
            from_boss: true,
        }]);
        stats.process_tick(1, &result, &state, (1, 1));
        assert_eq!(stats.items_by_rarity[Rarity::Epic as usize], 1);
        assert_eq!(stats.items_equipped, 1);
        assert_eq!(stats.boss_items_dropped, 1);
    }

    #[test]
    fn process_tick_tracks_level_ups_dungeons_fishing_and_achievements() {
        let mut stats = SimStats::default();
        let state = GameState::new("Sim".to_string(), 0);
        let result = tick_result(vec![
            TickEvent::LeveledUp { new_level: 5 },
            TickEvent::DungeonCompleted {
                xp_earned: 10,
                items_collected: 1,
            },
            TickEvent::DungeonFailed,
            TickEvent::DungeonDiscovered {
                message: "found".to_string(),
            },
            TickEvent::FishCaught {
                fish_name: "Trout".to_string(),
                rarity: Rarity::Common,
                message: "caught".to_string(),
            },
            TickEvent::FishingRankUp {
                message: "rank up".to_string(),
            },
            TickEvent::AchievementUnlocked {
                name: "Stormbreaker".to_string(),
            },
            TickEvent::HavenDiscovered,
        ]);
        stats.process_tick(1, &result, &state, (1, 1));
        assert_eq!(stats.level_at_tick[&5], 1);
        assert_eq!(stats.dungeons_completed, 1);
        assert_eq!(stats.dungeons_failed, 1);
        assert_eq!(stats.dungeons_discovered, 1);
        assert_eq!(stats.fish_caught, 1);
        assert_eq!(stats.fishing_rank_ups, 1);
        assert_eq!(stats.achievements_unlocked, 1);
        assert!(stats.haven_discovered);
    }

    #[test]
    fn process_tick_sums_dungeon_boss_xp_separately() {
        let mut stats = SimStats::default();
        let state = GameState::new("Sim".to_string(), 0);
        let result = tick_result(vec![TickEvent::DungeonBossDefeated {
            xp_gained: 100,
            bonus_xp: 50,
            total_xp: 150,
            items_collected: 2,
            enemy_name: "Elite".to_string(),
        }]);
        stats.process_tick(1, &result, &state, (1, 1));
        assert_eq!(stats.dungeons_completed, 1);
        assert_eq!(stats.total_xp_gained, 150);
    }

    #[test]
    fn finalize_snapshots_final_state() {
        let mut stats = SimStats::default();
        let mut state = GameState::new("Sim".to_string(), 0);
        state.character_level = 42;
        state.prestige_rank = 7;
        state.ascension_level = 2;
        state.stormglass = 999;
        let deep = quest::deep::DeepState::new();
        let loom = quest::loom::LoomState::new();
        stats.finalize(&state, &deep, &loom);
        assert_eq!(stats.final_level, 42);
        assert_eq!(stats.final_prestige, 7);
        assert_eq!(stats.ascension_level, 2);
        assert_eq!(stats.stormglass_balance, 999);
        assert_eq!(
            stats.deep_layers_reached,
            deep.persistent.deepest_layer_reached
        );
        assert_eq!(
            stats.loom_zone_cap,
            quest::loom::loom_zone_cap_for_patterns(0)
        );
    }

    #[test]
    fn tick_profile_tracks_min_max_and_average() {
        let mut profile = TickProfile::default();
        assert_eq!(profile.avg_us(), 0.0);
        profile.record(1000);
        profile.record(3000);
        profile.record(2000);
        assert_eq!(profile.tick_count, 3);
        assert_eq!(profile.min_us(), 1.0);
        assert_eq!(profile.max_us(), 3.0);
        assert!((profile.avg_us() - 2.0).abs() < f64::EPSILON);
    }
}
