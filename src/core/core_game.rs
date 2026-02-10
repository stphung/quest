//! Core game engine implementing GameLoop trait.
//!
//! This is the shared game logic that both the interactive game and
//! simulator can use for consistent behavior.

use super::balance::KILLS_PER_BOSS;
use super::game_loop::{GameLoop, TickResult};
use super::game_state::GameState;
use super::progression::{can_access_zone, max_zone_for_prestige};
use crate::character::derived_stats::DerivedStats;
use crate::character::prestige::{can_prestige as check_can_prestige, perform_prestige};
use crate::combat::types::{generate_enemy_for_current_zone, generate_subzone_boss, Enemy};
use crate::core::combat_math::*;
use crate::core::game_logic::{apply_tick_xp, xp_gain_per_tick};
use crate::items::drops::{try_drop_from_boss, try_drop_from_mob};
use crate::items::scoring::auto_equip_if_better;
use crate::zones::get_zone;
use rand::Rng;

/// Core game engine that implements the GameLoop trait.
///
/// Encapsulates all game state and combat tracking needed for
/// executing game ticks deterministically with a provided RNG.
pub struct CoreGame {
    state: GameState,
    current_enemy: Option<Enemy>,
    kills_in_subzone: u32,
}

impl CoreGame {
    /// Create a new game with the given player name.
    pub fn new(player_name: String) -> Self {
        Self {
            state: GameState::new(player_name, chrono::Utc::now().timestamp()),
            current_enemy: None,
            kills_in_subzone: 0,
        }
    }

    /// Create a game from an existing state (for save/load).
    pub fn from_state(state: GameState) -> Self {
        Self {
            state,
            current_enemy: None,
            kills_in_subzone: 0,
        }
    }

    /// Get the current zone ID.
    fn current_zone(&self) -> u32 {
        self.state.zone_progression.current_zone_id
    }

    /// Get the current subzone ID.
    fn current_subzone(&self) -> u32 {
        self.state.zone_progression.current_subzone_id
    }

    /// Check if we should be fighting a boss.
    fn should_fight_boss(&self) -> bool {
        self.kills_in_subzone >= KILLS_PER_BOSS
    }

    /// Spawn an enemy appropriate for the current zone/subzone.
    fn spawn_enemy(&mut self, _rng: &mut impl Rng) {
        let derived = DerivedStats::calculate_derived_stats(&self.state.attributes, &self.state.equipment);
        let player_hp = derived.max_hp;
        let player_damage = derived.total_damage();

        let enemy = if self.should_fight_boss() {
            // Spawn boss
            if let Some(zone) = get_zone(self.current_zone()) {
                if let Some(subzone) = zone.subzones.iter().find(|s| s.id == self.current_subzone()) {
                    generate_subzone_boss(&zone, subzone, player_hp, player_damage)
                } else {
                    generate_enemy_for_current_zone(
                        self.current_zone(),
                        self.current_subzone(),
                        player_hp,
                        player_damage,
                    )
                }
            } else {
                generate_enemy_for_current_zone(
                    self.current_zone(),
                    self.current_subzone(),
                    player_hp,
                    player_damage,
                )
            }
        } else {
            // Spawn regular enemy
            generate_enemy_for_current_zone(
                self.current_zone(),
                self.current_subzone(),
                player_hp,
                player_damage,
            )
        };

        self.current_enemy = Some(enemy);
    }

    /// Advance to the next subzone or zone after defeating a boss.
    fn advance_zone(&mut self) -> bool {
        self.kills_in_subzone = 0;

        let zone_id = self.current_zone();
        let subzone_id = self.current_subzone();

        // Get max subzones for current zone
        let max_subzones = get_zone(zone_id)
            .map(|z| z.subzones.len() as u32)
            .unwrap_or(3);

        if subzone_id >= max_subzones {
            // Try to advance to next zone
            let next_zone = zone_id + 1;
            if next_zone <= 10 && can_access_zone(self.state.prestige_rank, next_zone) {
                self.state.zone_progression.current_zone_id = next_zone;
                self.state.zone_progression.current_subzone_id = 1;
                return true;
            }
            // At prestige wall, stay in current zone
            false
        } else {
            // Advance to next subzone
            self.state.zone_progression.current_subzone_id += 1;
            false
        }
    }

    /// Calculate XP for killing the current enemy.
    fn calculate_kill_xp(&self, is_boss: bool) -> u64 {
        use crate::character::attributes::AttributeType;

        let wis_mod = self.state.attributes.modifier(AttributeType::Wisdom);
        let cha_mod = self.state.attributes.modifier(AttributeType::Charisma);
        let base_xp = xp_gain_per_tick(self.state.prestige_rank, wis_mod, cha_mod);

        // Base XP ticks for combat kill (200-400 range from constants)
        let ticks = if is_boss { 400.0 } else { 300.0 };
        (base_xp * ticks) as u64
    }
}

impl GameLoop for CoreGame {
    fn tick(&mut self, rng: &mut impl Rng) -> TickResult {
        let mut result = TickResult::default();

        // Spawn enemy if needed
        if self.current_enemy.is_none() {
            self.spawn_enemy(rng);
        }

        if self.current_enemy.is_none() {
            return result;
        }

        // Combat tick
        result.had_combat = true;
        result.was_boss = self.should_fight_boss();

        let derived = DerivedStats::calculate_derived_stats(&self.state.attributes, &self.state.equipment);
        let attack = calculate_attack_simple(&derived, rng);

        // Player attacks enemy
        let enemy = self.current_enemy.as_mut().unwrap();
        enemy.take_damage(attack.damage);

        if !enemy.is_alive() {
            // Player won
            result.player_won = true;
            self.kills_in_subzone += 1;
            self.state.session_kills += 1;

            // XP reward
            let xp = self.calculate_kill_xp(result.was_boss);
            let (levelups, _) = apply_tick_xp(&mut self.state, xp as f64);

            result.xp_gained = xp;
            if levelups > 0 {
                result.leveled_up = true;
                result.new_level = self.state.character_level;
            }

            // Handle loot
            let zone_id = self.current_zone() as usize;
            let is_final_zone = zone_id == 10;

            if result.was_boss {
                // Boss always drops an item
                let item = try_drop_from_boss(zone_id, is_final_zone);
                result.loot_dropped = Some(item.clone());
                if auto_equip_if_better(item, &mut self.state) {
                    result.loot_equipped = true;
                }
            } else {
                // Mob has a chance to drop
                if let Some(item) = try_drop_from_mob(&self.state, zone_id, 0.0, 0.0) {
                    result.loot_dropped = Some(item.clone());
                    if auto_equip_if_better(item, &mut self.state) {
                        result.loot_equipped = true;
                    }
                }
            }

            // Advance zone if boss
            if result.was_boss {
                let old_zone = self.current_zone();
                if self.advance_zone() && self.current_zone() > old_zone {
                    result.zone_advanced = true;
                    result.new_zone = self.current_zone();
                }
            }

            // Clear enemy for next spawn
            self.current_enemy = None;
        } else {
            // Enemy attacks back
            let damage_taken = calculate_damage_taken(enemy.damage, derived.defense);
            self.state.combat_state.player_current_hp =
                apply_damage(self.state.combat_state.player_current_hp, damage_taken);

            if self.state.combat_state.player_current_hp == 0 {
                // Player died - respawn
                result.player_died = true;
                self.state.combat_state.player_current_hp = derived.max_hp;

                // Reset progress on boss death
                if result.was_boss {
                    self.kills_in_subzone = 0;
                }

                // Clear enemy
                self.current_enemy = None;
            }
        }

        result.can_prestige = self.can_prestige();
        result.at_prestige_wall = self.at_prestige_wall();

        result
    }

    fn prestige(&mut self) {
        perform_prestige(&mut self.state);
        self.kills_in_subzone = 0;
        self.current_enemy = None;
    }

    fn state(&self) -> &GameState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut GameState {
        &mut self.state
    }

    fn can_prestige(&self) -> bool {
        check_can_prestige(&self.state)
    }

    fn at_prestige_wall(&self) -> bool {
        self.current_zone() >= max_zone_for_prestige(self.state.prestige_rank)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game() {
        let game = CoreGame::new("Test Hero".to_string());
        assert_eq!(game.state().character_level, 1);
        assert_eq!(game.state().character_xp, 0);
        assert_eq!(game.current_zone(), 1);
        assert_eq!(game.current_subzone(), 1);
    }

    #[test]
    fn test_tick_spawns_enemy() {
        let mut game = CoreGame::new("Test Hero".to_string());
        let mut rng = rand::thread_rng();

        let result = game.tick(&mut rng);

        assert!(result.had_combat);
        // After first tick, either we won or enemy is still alive
        assert!(result.player_won || game.current_enemy.is_some());
    }

    #[test]
    fn test_kill_grants_xp() {
        let mut game = CoreGame::new("Test Hero".to_string());
        let mut rng = rand::thread_rng();

        // Run ticks until we get a kill
        let mut total_xp = 0u64;
        for _ in 0..100 {
            let result = game.tick(&mut rng);
            total_xp += result.xp_gained;
            if result.xp_gained > 0 {
                break;
            }
        }

        assert!(total_xp > 0, "Should have gained XP from a kill");
    }

    #[test]
    fn test_boss_spawns_after_kills() {
        let mut game = CoreGame::new("Test Hero".to_string());
        let mut rng = rand::thread_rng();

        // Kill enough enemies to reach boss
        let mut saw_boss = false;
        for _ in 0..5000 {
            let result = game.tick(&mut rng);
            if result.was_boss && result.player_won {
                saw_boss = true;
                break;
            }
        }

        assert!(saw_boss, "Should have encountered and defeated a boss within 5000 ticks");
    }

    #[test]
    fn test_can_prestige_initially_false() {
        let game = CoreGame::new("Test Hero".to_string());
        assert!(!game.can_prestige());
    }

    #[test]
    fn test_at_prestige_wall_initially_false() {
        let game = CoreGame::new("Test Hero".to_string());
        // At P0, max zone is 2, starting at zone 1
        assert!(!game.at_prestige_wall());
    }

    #[test]
    fn test_from_state() {
        let state = GameState::new("Loaded Hero".to_string(), 0);
        let game = CoreGame::from_state(state);

        assert_eq!(game.state().character_name, "Loaded Hero");
    }
}
