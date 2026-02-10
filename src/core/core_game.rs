//! Core game engine implementing GameLoop trait.
//!
//! This module provides two ways to use the game logic:
//!
//! 1. **CoreGame struct** - A self-contained game engine for simulation/testing.
//!    Owns its GameState and manages all combat internally. Best for:
//!    - Running thousands of ticks for balance testing
//!    - Offline progression simulation
//!    - Unit tests
//!
//! 2. **resolve_combat_tick() function** - Executes one combat round on an existing
//!    GameState. Best for the interactive game where:
//!    - External timing controls when combat happens
//!    - Visual effects and combat logs need to be generated
//!    - Dungeons/fishing/minigames pause normal combat

use super::balance::KILLS_PER_BOSS;
use super::game_loop::{GameLoop, TickResult};
use super::game_state::GameState;
use super::progression::{can_access_zone, max_zone_for_prestige};
use crate::character::derived_stats::DerivedStats;
use crate::character::prestige::{can_prestige as check_can_prestige, perform_prestige};
use crate::combat::types::{generate_enemy_for_current_zone, generate_subzone_boss, Enemy};
use crate::core::combat_math::*;
use crate::core::game_logic::{apply_tick_xp, spawn_enemy_if_needed, xp_gain_per_tick};
use crate::items::drops::{try_drop_from_boss, try_drop_from_mob};
use crate::items::scoring::auto_equip_if_better;
use crate::zones::get_zone;
use rand::Rng;

/// Core game engine that implements the GameLoop trait.
///
/// This struct owns its GameState and is designed for simulation use cases
/// where we need to run many ticks quickly without UI updates.
///
/// For the interactive game with timing-based combat, use `resolve_combat_tick()`
/// instead, which operates on an external GameState.
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

// =============================================================================
// Standalone combat resolution for interactive game
// =============================================================================

/// Configuration for combat resolution bonuses (from Haven, etc.)
#[derive(Debug, Clone, Default)]
pub struct CombatBonuses {
    /// Bonus damage percentage (e.g., 10.0 = +10%)
    pub damage_percent: f64,
    /// Bonus crit chance (flat, e.g., 5 = +5% crit)
    pub crit_chance: u32,
    /// Bonus drop rate percentage
    pub drop_rate_percent: f64,
    /// Bonus item rarity percentage
    pub item_rarity_percent: f64,
    /// Bonus XP gain percentage
    pub xp_gain_percent: f64,
}

/// Resolves one combat tick for an existing GameState.
///
/// This function is designed for the interactive game where:
/// - External timing controls when combat happens (attack_timer)
/// - The caller handles visual effects, combat logs, etc.
/// - Dungeons/fishing/minigames are handled separately
///
/// Call this when the attack timer has elapsed and the player should attack.
///
/// # Arguments
/// * `state` - The game state to modify
/// * `bonuses` - Combat bonuses from Haven, etc.
/// * `rng` - Random number generator
///
/// # Returns
/// A TickResult describing what happened (damage dealt, enemy killed, etc.)
pub fn resolve_combat_tick(
    state: &mut GameState,
    bonuses: &CombatBonuses,
    rng: &mut impl Rng,
) -> TickResult {
    let mut result = TickResult::default();

    // Don't do combat if regenerating
    if state.combat_state.is_regenerating {
        return result;
    }

    // Spawn enemy if needed (uses GameState's combat_state.current_enemy)
    spawn_enemy_if_needed(state);

    // No combat if no enemy
    if state.combat_state.current_enemy.is_none() {
        return result;
    }

    result.had_combat = true;
    result.was_boss = state.zone_progression.fighting_boss;

    // Calculate player attack
    let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
    let damage_multiplier = 1.0 + bonuses.damage_percent / 100.0;
    let attack = calculate_player_attack(&derived, bonuses.crit_chance, damage_multiplier, rng);

    result.damage_dealt = attack.damage;
    result.was_crit = attack.is_crit;

    // Player attacks enemy
    let enemy = state.combat_state.current_enemy.as_mut().unwrap();
    let enemy_name = enemy.name.clone();
    enemy.current_hp = enemy.current_hp.saturating_sub(attack.damage);

    if enemy.current_hp == 0 {
        // Enemy died - player won!
        result.player_won = true;
        result.enemy_name = Some(enemy_name);

        // Calculate XP with bonuses
        let base_xp = calculate_kill_xp(state.prestige_rank, &state.attributes, result.was_boss);
        let xp_with_bonus = (base_xp as f64 * (1.0 + bonuses.xp_gain_percent / 100.0)) as u64;
        result.xp_gained = xp_with_bonus;

        // Apply XP and check for level up
        let _level_before = state.character_level;
        let (levelups, _) = apply_tick_xp(state, xp_with_bonus as f64);
        if levelups > 0 {
            result.leveled_up = true;
            result.new_level = state.character_level;
        }

        // Handle loot drops
        let zone_id = state.zone_progression.current_zone_id as usize;
        let is_final_zone = zone_id == 10;

        if result.was_boss {
            // Boss always drops an item
            let item = try_drop_from_boss(zone_id, is_final_zone);
            result.loot_dropped = Some(item.clone());
            if auto_equip_if_better(item, state) {
                result.loot_equipped = true;
            }
        } else {
            // Regular mob has chance to drop
            if let Some(item) = try_drop_from_mob(
                state,
                zone_id,
                bonuses.drop_rate_percent,
                bonuses.item_rarity_percent,
            ) {
                result.loot_dropped = Some(item.clone());
                if auto_equip_if_better(item, state) {
                    result.loot_equipped = true;
                }
            }
        }

        // Handle zone/subzone advancement for boss kills
        if result.was_boss {
            let old_zone = state.zone_progression.current_zone_id;
            advance_after_boss_kill(state);
            if state.zone_progression.current_zone_id > old_zone {
                result.zone_advanced = true;
                result.new_zone = state.zone_progression.current_zone_id;
            }
        }

        // Clear enemy
        state.combat_state.current_enemy = None;
        state.zone_progression.fighting_boss = false;
    } else {
        // Enemy survives - attacks back
        let damage_taken = calculate_damage_taken(enemy.damage, derived.defense);
        result.damage_taken = damage_taken;

        state.combat_state.player_current_hp =
            apply_damage(state.combat_state.player_current_hp, damage_taken);

        if state.combat_state.player_current_hp == 0 {
            // Player died
            result.player_died = true;

            // Reset boss progress on death
            if result.was_boss {
                state.zone_progression.fighting_boss = false;
                state.zone_progression.kills_in_subzone = 0;
            }

            // Start regeneration
            state.combat_state.is_regenerating = true;
            state.combat_state.regen_timer = 0.0;

            // Clear enemy
            state.combat_state.current_enemy = None;
        }
    }

    // Update prestige status
    result.can_prestige = check_can_prestige(state);
    result.at_prestige_wall =
        state.zone_progression.current_zone_id >= max_zone_for_prestige(state.prestige_rank);

    result
}

/// Calculate XP for killing an enemy.
fn calculate_kill_xp(
    prestige_rank: u32,
    attributes: &crate::character::attributes::Attributes,
    is_boss: bool,
) -> u64 {
    use crate::character::attributes::AttributeType;

    let wis_mod = attributes.modifier(AttributeType::Wisdom);
    let cha_mod = attributes.modifier(AttributeType::Charisma);
    let base_xp = xp_gain_per_tick(prestige_rank, wis_mod, cha_mod);

    // Boss gives more XP
    let ticks = if is_boss { 400.0 } else { 300.0 };
    (base_xp * ticks) as u64
}

/// Advance zone/subzone after killing a boss.
fn advance_after_boss_kill(state: &mut GameState) {
    // Reset kill counter
    state.zone_progression.kills_in_subzone = 0;

    let zone_id = state.zone_progression.current_zone_id;
    let subzone_id = state.zone_progression.current_subzone_id;

    // Get max subzones for current zone
    let max_subzones = get_zone(zone_id)
        .map(|z| z.subzones.len() as u32)
        .unwrap_or(3);

    if subzone_id >= max_subzones {
        // Try to advance to next zone
        let next_zone = zone_id + 1;
        if next_zone <= 10 && can_access_zone(state.prestige_rank, next_zone) {
            state.zone_progression.current_zone_id = next_zone;
            state.zone_progression.current_subzone_id = 1;
        }
        // At prestige wall - stay in current zone
    } else {
        // Advance to next subzone
        state.zone_progression.current_subzone_id += 1;
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

    // ==========================================================================
    // Tests for resolve_combat_tick (used by interactive game)
    // ==========================================================================

    #[test]
    fn test_resolve_combat_tick_spawns_enemy() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let bonuses = CombatBonuses::default();
        let mut rng = rand::thread_rng();

        // First call should spawn an enemy and do combat
        let result = resolve_combat_tick(&mut state, &bonuses, &mut rng);

        assert!(result.had_combat);
        // Enemy should be spawned (either killed or still alive)
        assert!(result.player_won || state.combat_state.current_enemy.is_some());
    }

    #[test]
    fn test_resolve_combat_tick_skips_when_regenerating() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        state.combat_state.is_regenerating = true;
        let bonuses = CombatBonuses::default();
        let mut rng = rand::thread_rng();

        let result = resolve_combat_tick(&mut state, &bonuses, &mut rng);

        assert!(!result.had_combat, "Should not have combat while regenerating");
        assert!(state.combat_state.current_enemy.is_none());
    }

    #[test]
    fn test_resolve_combat_tick_grants_xp_on_kill() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let bonuses = CombatBonuses::default();
        let mut rng = rand::thread_rng();

        // Run until we get a kill
        let mut total_xp = 0u64;
        for _ in 0..100 {
            let result = resolve_combat_tick(&mut state, &bonuses, &mut rng);
            total_xp += result.xp_gained;
            if result.xp_gained > 0 {
                break;
            }
        }

        assert!(total_xp > 0, "Should have gained XP from a kill");
    }

    #[test]
    fn test_resolve_combat_tick_respects_damage_bonus() {
        let mut state = GameState::new("Test Hero".to_string(), 0);

        // Run many trials with and without bonus to compare average damage
        let trials = 100;
        let mut damage_no_bonus = 0u32;
        let mut damage_with_bonus = 0u32;

        let no_bonus = CombatBonuses::default();
        let with_bonus = CombatBonuses {
            damage_percent: 100.0, // +100% damage
            ..CombatBonuses::default()
        };

        for _ in 0..trials {
            let mut state_a = GameState::new("A".to_string(), 0);
            let mut state_b = GameState::new("B".to_string(), 0);
            let mut rng = rand::thread_rng();

            let result_a = resolve_combat_tick(&mut state_a, &no_bonus, &mut rng);
            let result_b = resolve_combat_tick(&mut state_b, &with_bonus, &mut rng);

            damage_no_bonus += result_a.damage_dealt;
            damage_with_bonus += result_b.damage_dealt;
        }

        // With +100% damage, average should be roughly 2x
        // Allow some variance due to random damage rolls
        let ratio = damage_with_bonus as f64 / damage_no_bonus as f64;
        assert!(
            ratio > 1.5 && ratio < 2.5,
            "Damage bonus should roughly double damage, got ratio {:.2}",
            ratio
        );
    }

    #[test]
    fn test_resolve_combat_tick_respects_xp_bonus() {
        // Run until we get kills and compare XP with/without bonus
        let trials = 50;
        let mut xp_no_bonus = 0u64;
        let mut xp_with_bonus = 0u64;

        let no_bonus = CombatBonuses::default();
        let with_bonus = CombatBonuses {
            xp_gain_percent: 50.0, // +50% XP
            ..CombatBonuses::default()
        };

        for _ in 0..trials {
            let mut state_a = GameState::new("A".to_string(), 0);
            let mut state_b = GameState::new("B".to_string(), 0);
            let mut rng = rand::thread_rng();

            // Get a kill for each
            for _ in 0..100 {
                let result_a = resolve_combat_tick(&mut state_a, &no_bonus, &mut rng);
                if result_a.xp_gained > 0 {
                    xp_no_bonus += result_a.xp_gained;
                    break;
                }
            }
            for _ in 0..100 {
                let result_b = resolve_combat_tick(&mut state_b, &with_bonus, &mut rng);
                if result_b.xp_gained > 0 {
                    xp_with_bonus += result_b.xp_gained;
                    break;
                }
            }
        }

        // With +50% XP, average should be ~1.5x
        let ratio = xp_with_bonus as f64 / xp_no_bonus as f64;
        assert!(
            ratio > 1.3 && ratio < 1.7,
            "XP bonus should increase XP by ~50%, got ratio {:.2}",
            ratio
        );
    }

    #[test]
    fn test_resolve_combat_tick_returns_damage_info() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let bonuses = CombatBonuses::default();
        let mut rng = rand::thread_rng();

        // Get a combat result
        let result = resolve_combat_tick(&mut state, &bonuses, &mut rng);

        assert!(result.had_combat);
        assert!(result.damage_dealt > 0, "Should have dealt some damage");
    }

    #[test]
    fn test_resolve_combat_tick_player_death_starts_regen() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let bonuses = CombatBonuses::default();
        let mut rng = rand::thread_rng();

        // Run until player dies
        let mut saw_death = false;
        for _ in 0..1000 {
            let result = resolve_combat_tick(&mut state, &bonuses, &mut rng);
            if result.player_died {
                saw_death = true;
                break;
            }
        }

        if saw_death {
            assert!(
                state.combat_state.is_regenerating,
                "Player death should trigger regeneration"
            );
            assert!(
                state.combat_state.current_enemy.is_none(),
                "Enemy should be cleared on player death"
            );
        }
        // If no death in 1000 ticks, that's OK - player is just winning
    }
}
