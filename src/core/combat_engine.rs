//! Core game engine implementing GameLoop trait.
//!
//! This module provides CombatEngine as the central game engine:
//!
//! **CombatEngine struct** - The main game engine that owns GameState and manages
//! all game logic. It provides two modes of operation:
//!
//! 1. **Simulation mode** (`tick()`) - Self-contained combat for balance testing
//!    and offline progression. Manages its own enemy/kills tracking.
//!
//! 2. **Interactive mode** (`combat_tick()`) - Works with GameState's timing-based
//!    combat system. Used by main.rs for the actual game where:
//!    - External timing controls when combat happens
//!    - Visual effects and combat logs need to be generated
//!    - Haven bonuses apply
//!
//! The `resolve_combat_tick()` function is also exported for backward compatibility.

use super::balance::KILLS_PER_BOSS;
use super::game_loop::{GameLoop, TickResult};
use super::game_state::GameState;
use super::progression::{can_access_zone, max_zone_for_prestige};
use crate::achievements::Achievements;
use crate::character::derived_stats::DerivedStats;
use crate::character::prestige::{can_prestige as check_can_prestige, perform_prestige};
use crate::combat::types::{generate_enemy_for_current_zone, generate_subzone_boss, Enemy};
use crate::core::combat_math::*;
use crate::core::game_logic::{apply_tick_xp, spawn_enemy_if_needed, xp_gain_per_tick};
use crate::items::drops::{try_drop_from_boss, try_drop_from_mob};
use crate::items::scoring::auto_equip_if_better;
use crate::zones;
use crate::zones::BossDefeatResult;
use crate::zones::get_zone;
use rand::Rng;

/// Core game engine that implements the GameLoop trait.
///
/// This struct owns its GameState and is the central game engine.
/// Use `tick()` for simulation mode or `combat_tick()` for interactive mode
/// with Haven bonuses.
#[allow(dead_code)]
pub struct CombatEngine {
    state: GameState,
    /// Internal enemy for simulation mode (tick())
    current_enemy: Option<Enemy>,
    /// Internal kill counter for simulation mode (tick())
    kills_in_subzone: u32,
    /// Combat bonuses from Haven (used by combat_tick())
    bonuses: CombatBonuses,
}

#[allow(dead_code)]
impl CombatEngine {
    /// Create a new game with the given player name.
    pub fn new(player_name: String) -> Self {
        Self {
            state: GameState::new(player_name, chrono::Utc::now().timestamp()),
            current_enemy: None,
            kills_in_subzone: 0,
            bonuses: CombatBonuses::default(),
        }
    }

    /// Create a game from an existing state (for save/load).
    pub fn from_state(state: GameState) -> Self {
        Self {
            state,
            current_enemy: None,
            kills_in_subzone: 0,
            bonuses: CombatBonuses::default(),
        }
    }

    /// Set combat bonuses from Haven for interactive mode.
    ///
    /// These bonuses are applied when using `combat_tick()`.
    pub fn set_bonuses(&mut self, bonuses: CombatBonuses) {
        self.bonuses = bonuses;
    }

    /// Get the current combat bonuses.
    pub fn bonuses(&self) -> &CombatBonuses {
        &self.bonuses
    }

    /// Execute one combat tick in interactive mode.
    ///
    /// This uses the state's timing-based combat system (GameState.combat_state)
    /// and applies Haven bonuses. Use this for the interactive game.
    ///
    /// Unlike `tick()`, this:
    /// - Respects `is_regenerating` flag
    /// - Uses `state.combat_state.current_enemy` for enemies
    /// - Applies combat bonuses from Haven
    /// - Checks boss weapon requirements
    /// - Tracks kills for achievements
    ///
    /// # Arguments
    /// * `achievements` - Global achievements (for boss weapon check and kill tracking)
    /// * `rng` - Random number generator
    ///
    /// # Returns
    /// A TickResult describing what happened
    pub fn combat_tick(&mut self, achievements: &mut Achievements, rng: &mut impl Rng) -> TickResult {
        resolve_combat_tick(&mut self.state, &self.bonuses, achievements, rng)
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
        let derived =
            DerivedStats::calculate_derived_stats(&self.state.attributes, &self.state.equipment);
        let player_hp = derived.max_hp;
        let player_damage = derived.total_damage();

        let enemy = if self.should_fight_boss() {
            // Spawn boss
            if let Some(zone) = get_zone(self.current_zone()) {
                if let Some(subzone) = zone
                    .subzones
                    .iter()
                    .find(|s| s.id == self.current_subzone())
                {
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

impl GameLoop for CombatEngine {
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

        let derived =
            DerivedStats::calculate_derived_stats(&self.state.attributes, &self.state.equipment);
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

            // Apply damage reflection: reflect percentage of damage taken back to attacker
            if derived.damage_reflection_percent > 0.0 && damage_taken > 0 {
                let reflected =
                    calculate_damage_reflection(damage_taken, derived.damage_reflection_percent);
                if reflected > 0 {
                    let enemy = self.current_enemy.as_mut().unwrap();
                    enemy.current_hp = enemy.current_hp.saturating_sub(reflected);

                    // Check if enemy died from reflection (player wins even if both die)
                    if enemy.current_hp == 0 {
                        result.player_won = true;
                        self.kills_in_subzone += 1;
                        self.state.session_kills += 1;

                        let xp = self.calculate_kill_xp(result.was_boss);
                        let (levelups, _) = apply_tick_xp(&mut self.state, xp as f64);
                        result.xp_gained = xp;
                        if levelups > 0 {
                            result.leveled_up = true;
                            result.new_level = self.state.character_level;
                        }

                        if result.was_boss {
                            let old_zone = self.current_zone();
                            if self.advance_zone() && self.current_zone() > old_zone {
                                result.zone_advanced = true;
                                result.new_zone = self.current_zone();
                            }
                        }

                        self.current_enemy = None;
                        result.can_prestige = self.can_prestige();
                        result.at_prestige_wall = self.at_prestige_wall();
                        return result;
                    }
                }
            }

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
    /// Chance for double strike (War Room bonus)
    pub double_strike_chance: f64,
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
/// * `achievements` - Global achievements (for boss weapon check and kill tracking)
/// * `rng` - Random number generator
///
/// # Returns
/// A TickResult describing what happened (damage dealt, enemy killed, etc.)
pub fn resolve_combat_tick(
    state: &mut GameState,
    bonuses: &CombatBonuses,
    achievements: &mut Achievements,
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

    // Check if boss requires a weapon we don't have
    if let Some(weapon_name) = state.zone_progression.boss_weapon_blocked(achievements) {
        result.attack_blocked = true;
        result.weapon_needed = Some(weapon_name.to_string());
        // Enemy still attacks back (handled below after attack logic)
    }

    // Calculate player stats
    let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
    let damage_multiplier = 1.0 + bonuses.damage_percent / 100.0;

    // Roll for double strike (War Room bonus)
    let double_strike_roll = rng.gen::<f64>() * 100.0;
    let num_strikes = if double_strike_roll < bonuses.double_strike_chance {
        result.was_double_strike = true;
        2
    } else {
        1
    };

    // Get enemy info before combat
    let enemy = state.combat_state.current_enemy.as_ref().unwrap();
    let enemy_name = enemy.name.clone();
    let enemy_damage = enemy.damage;

    // Player attacks (unless blocked by weapon requirement)
    let mut enemy_killed = false;
    if !result.attack_blocked {
        let enemy = state.combat_state.current_enemy.as_mut().unwrap();

        // Apply damage for each strike
        for _ in 0..num_strikes {
            let attack =
                calculate_player_attack(&derived, bonuses.crit_chance, damage_multiplier, rng);
            result.damage_dealt += attack.damage;
            if attack.is_crit {
                result.was_crit = true;
            }
            enemy.current_hp = enemy.current_hp.saturating_sub(attack.damage);
        }

        enemy_killed = enemy.current_hp == 0;
    }

    if enemy_killed {
        // Enemy died - player won!
        result.player_won = true;
        result.enemy_name = Some(enemy_name);

        // Calculate XP with bonuses
        let base_xp = calculate_kill_xp(state.prestige_rank, &state.attributes, result.was_boss);
        let xp_with_bonus = (base_xp as f64 * (1.0 + bonuses.xp_gain_percent / 100.0)) as u64;
        result.xp_gained = xp_with_bonus;

        // Apply XP and check for level up
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

            // Record kill for boss spawn tracking (non-boss kills only)
            state.zone_progression.record_kill();
        }

        // Handle zone/subzone advancement for boss kills
        if result.was_boss {
            let old_zone = state.zone_progression.current_zone_id;
            let boss_result = advance_after_boss_kill(state, achievements);
            result.boss_defeat_result = Some(boss_result);
            if state.zone_progression.current_zone_id > old_zone {
                result.zone_advanced = true;
                result.new_zone = state.zone_progression.current_zone_id;
            }
        }

        // Track kill for achievements
        achievements.on_enemy_killed(result.was_boss, Some(&state.character_name));

        // Clear enemy and start regeneration
        state.combat_state.current_enemy = None;
        // Only clear fighting_boss flag if we killed a boss (not for regular mob kills)
        // For regular mobs, record_kill() may have just SET this flag to trigger boss spawn
        if result.was_boss {
            state.zone_progression.fighting_boss = false;
        }
        state.combat_state.is_regenerating = true;
        state.combat_state.regen_timer = 0.0;
    } else {
        // Enemy survives (or attack was blocked) - enemy attacks back
        let damage_taken = calculate_damage_taken(enemy_damage, derived.defense);
        result.damage_taken = damage_taken;

        state.combat_state.player_current_hp =
            apply_damage(state.combat_state.player_current_hp, damage_taken);

        // Damage reflection: reflect percentage of damage taken back to attacker
        if derived.damage_reflection_percent > 0.0 && damage_taken > 0 {
            let reflected =
                calculate_damage_reflection(damage_taken, derived.damage_reflection_percent);
            if reflected > 0 {
                if let Some(enemy) = state.combat_state.current_enemy.as_mut() {
                    enemy.current_hp = enemy.current_hp.saturating_sub(reflected);

                    // Check if enemy died from reflection (player wins even if both die)
                    if enemy.current_hp == 0 {
                        result.player_won = true;
                        result.enemy_name = Some(enemy_name.clone());

                        // Calculate XP with bonuses
                        let base_xp = calculate_kill_xp(
                            state.prestige_rank,
                            &state.attributes,
                            result.was_boss,
                        );
                        let xp_with_bonus =
                            (base_xp as f64 * (1.0 + bonuses.xp_gain_percent / 100.0)) as u64;
                        result.xp_gained = xp_with_bonus;

                        // Apply XP and check for level up
                        let (levelups, _) = apply_tick_xp(state, xp_with_bonus as f64);
                        if levelups > 0 {
                            result.leveled_up = true;
                            result.new_level = state.character_level;
                        }

                        // Record kill for progression (non-boss only)
                        if !result.was_boss {
                            state.zone_progression.record_kill();
                        }

                        // Handle zone/subzone advancement for boss kills
                        if result.was_boss {
                            let old_zone = state.zone_progression.current_zone_id;
                            let boss_result = advance_after_boss_kill(state, achievements);
                            result.boss_defeat_result = Some(boss_result);
                            if state.zone_progression.current_zone_id > old_zone {
                                result.zone_advanced = true;
                                result.new_zone = state.zone_progression.current_zone_id;
                            }
                        }

                        achievements.on_enemy_killed(result.was_boss, Some(&state.character_name));

                        state.combat_state.current_enemy = None;
                        if result.was_boss {
                            state.zone_progression.fighting_boss = false;
                        }
                        state.combat_state.is_regenerating = true;
                        state.combat_state.regen_timer = 0.0;

                        result.can_prestige = check_can_prestige(state);
                        result.at_prestige_wall = state.zone_progression.current_zone_id
                            >= max_zone_for_prestige(state.prestige_rank);
                        return result;
                    }
                }
            }
        }

        if state.combat_state.player_current_hp == 0 {
            // Player died
            result.player_died = true;

            if result.was_boss {
                // Reset boss progress on death
                state.zone_progression.fighting_boss = false;
                state.zone_progression.kills_in_subzone = 0;
                state.combat_state.current_enemy = None;
            } else {
                // Non-boss death: reset enemy HP so fight continues with same mob
                if let Some(enemy) = state.combat_state.current_enemy.as_mut() {
                    enemy.reset_hp();
                }
            }

            // Reset player HP and start regeneration period
            // (consistent with post-kill behavior - player needs recovery time)
            state.combat_state.player_current_hp = state.combat_state.player_max_hp;
            state.combat_state.is_regenerating = true;
            state.combat_state.regen_timer = 0.0;
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
///
/// Handles special cases:
/// - Zone 10 completion: unlocks Zone 11 (The Expanse) and triggers StormsEnd achievement
/// - Zone 11 completion: cycles back to Zone 11 Subzone 1 (infinite endgame zone)
fn advance_after_boss_kill(
    state: &mut GameState,
    achievements: &mut Achievements,
) -> BossDefeatResult {
    use crate::achievements::AchievementId;

    let zone_id = state.zone_progression.current_zone_id;
    let subzone_id = state.zone_progression.current_subzone_id;

    // Record boss defeat (tracks in defeated_bosses list)
    state.zone_progression.defeat_boss(zone_id, subzone_id);

    // Get zone info for result
    let zone_name = get_zone(zone_id)
        .map(|z| z.name.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    // Get max subzones for current zone
    let max_subzones = get_zone(zone_id)
        .map(|z| z.subzones.len() as u32)
        .unwrap_or(3);

    let is_zone_boss = subzone_id >= max_subzones;

    if is_zone_boss {
        // Zone 11 (The Expanse) - infinite cycling back to subzone 1
        if zone_id == 11 {
            state.zone_progression.current_subzone_id = 1;
            state.zone_progression.kills_in_subzone = 0;
            return BossDefeatResult::ExpanseCycle;
        }

        // Zone 10 completion - unlock Zone 11 and StormsEnd achievement
        if zone_id == 10 {
            achievements.unlock(AchievementId::StormsEnd, None);
            state.zone_progression.unlock_zone(11);
            state.zone_progression.current_zone_id = 11;
            state.zone_progression.current_subzone_id = 1;
            return BossDefeatResult::StormsEnd;
        }

        // Normal zone advancement (zones 1-9)
        let next_zone = zone_id + 1;
        if next_zone <= 10 && can_access_zone(state.prestige_rank, next_zone) {
            state.zone_progression.current_zone_id = next_zone;
            state.zone_progression.current_subzone_id = 1;
            return BossDefeatResult::ZoneComplete {
                old_zone: zone_name,
                new_zone_id: next_zone,
            };
        }

        // At prestige wall - stay in current zone
        let next_prestige = get_zone(next_zone)
            .map(|z| z.prestige_requirement)
            .unwrap_or(0);
        return BossDefeatResult::ZoneCompleteButGated {
            zone_name,
            required_prestige: next_prestige,
        };
    }

    // Advance to next subzone within the same zone
    state.zone_progression.current_subzone_id += 1;
    BossDefeatResult::SubzoneComplete {
        new_subzone_id: state.zone_progression.current_subzone_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game() {
        let game = CombatEngine::new("Test Hero".to_string());
        assert_eq!(game.state().character_level, 1);
        assert_eq!(game.state().character_xp, 0);
        assert_eq!(game.current_zone(), 1);
        assert_eq!(game.current_subzone(), 1);
    }

    #[test]
    fn test_tick_spawns_enemy() {
        let mut game = CombatEngine::new("Test Hero".to_string());
        let mut rng = rand::thread_rng();

        let result = game.tick(&mut rng);

        assert!(result.had_combat);
        // After first tick, either we won or enemy is still alive
        assert!(result.player_won || game.current_enemy.is_some());
    }

    #[test]
    fn test_kill_grants_xp() {
        let mut game = CombatEngine::new("Test Hero".to_string());
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
        let mut game = CombatEngine::new("Test Hero".to_string());
        let mut rng = rand::thread_rng();

        // Run enough ticks to encounter a boss (10 kills triggers boss)
        let mut saw_boss = false;
        for _ in 0..500 {
            let result = game.tick(&mut rng);
            if result.was_boss {
                saw_boss = true;
                break;
            }
        }

        assert!(saw_boss, "Should have encountered a boss within 500 ticks");
    }

    #[test]
    fn test_can_prestige_initially_false() {
        let game = CombatEngine::new("Test Hero".to_string());
        assert!(!game.can_prestige());
    }

    #[test]
    fn test_at_prestige_wall_initially_false() {
        let game = CombatEngine::new("Test Hero".to_string());
        // At P0, max zone is 2, starting at zone 1
        assert!(!game.at_prestige_wall());
    }

    #[test]
    fn test_from_state() {
        let state = GameState::new("Loaded Hero".to_string(), 0);
        let game = CombatEngine::from_state(state);

        assert_eq!(game.state().character_name, "Loaded Hero");
    }

    // ==========================================================================
    // Tests for resolve_combat_tick (used by interactive game)
    // ==========================================================================

    #[test]
    fn test_resolve_combat_tick_spawns_enemy() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let bonuses = CombatBonuses::default();
        let mut achievements = Achievements::default();
        let mut rng = rand::thread_rng();

        // First call should spawn an enemy and do combat
        let result = resolve_combat_tick(&mut state, &bonuses, &mut achievements, &mut rng);

        assert!(result.had_combat);
        // Enemy should be spawned (either killed or still alive)
        assert!(result.player_won || state.combat_state.current_enemy.is_some());
    }

    #[test]
    fn test_resolve_combat_tick_skips_when_regenerating() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        state.combat_state.is_regenerating = true;
        let bonuses = CombatBonuses::default();
        let mut achievements = Achievements::default();
        let mut rng = rand::thread_rng();

        let result = resolve_combat_tick(&mut state, &bonuses, &mut achievements, &mut rng);

        assert!(
            !result.had_combat,
            "Should not have combat while regenerating"
        );
        assert!(state.combat_state.current_enemy.is_none());
    }

    #[test]
    fn test_resolve_combat_tick_grants_xp_on_kill() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let bonuses = CombatBonuses::default();
        let mut achievements = Achievements::default();
        let mut rng = rand::thread_rng();

        // Run until we get a kill
        let mut total_xp = 0u64;
        for _ in 0..100 {
            let result = resolve_combat_tick(&mut state, &bonuses, &mut achievements, &mut rng);
            total_xp += result.xp_gained;
            if result.xp_gained > 0 {
                break;
            }
        }

        assert!(total_xp > 0, "Should have gained XP from a kill");
    }

    #[test]
    fn test_resolve_combat_tick_respects_damage_bonus() {
        // Run many trials with and without bonus to compare average damage
        let trials = 100;
        let mut damage_no_bonus = 0u32;
        let mut damage_with_bonus = 0u32;

        let no_bonus = CombatBonuses::default();
        let with_bonus = CombatBonuses {
            damage_percent: 100.0, // +100% damage
            ..CombatBonuses::default()
        };
        let mut achievements = Achievements::default();

        for _ in 0..trials {
            let mut state_a = GameState::new("A".to_string(), 0);
            let mut state_b = GameState::new("B".to_string(), 0);
            let mut rng = rand::thread_rng();

            let result_a = resolve_combat_tick(&mut state_a, &no_bonus, &mut achievements, &mut rng);
            let result_b = resolve_combat_tick(&mut state_b, &with_bonus, &mut achievements, &mut rng);

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
        let mut achievements = Achievements::default();

        for _ in 0..trials {
            let mut state_a = GameState::new("A".to_string(), 0);
            let mut state_b = GameState::new("B".to_string(), 0);
            let mut rng = rand::thread_rng();

            // Get a kill for each
            for _ in 0..100 {
                let result_a =
                    resolve_combat_tick(&mut state_a, &no_bonus, &mut achievements, &mut rng);
                if result_a.xp_gained > 0 {
                    xp_no_bonus += result_a.xp_gained;
                    break;
                }
            }
            for _ in 0..100 {
                let result_b =
                    resolve_combat_tick(&mut state_b, &with_bonus, &mut achievements, &mut rng);
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
        let mut achievements = Achievements::default();
        let mut rng = rand::thread_rng();

        // Get a combat result
        let result = resolve_combat_tick(&mut state, &bonuses, &mut achievements, &mut rng);

        assert!(result.had_combat);
        assert!(result.damage_dealt > 0, "Should have dealt some damage");
    }

    #[test]
    fn test_resolve_combat_tick_player_death_starts_regen() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let bonuses = CombatBonuses::default();
        let mut achievements = Achievements::default();
        let mut rng = rand::thread_rng();

        // Run until player dies
        let mut saw_death = false;
        for _ in 0..1000 {
            // Reset regen to keep combat active
            state.combat_state.is_regenerating = false;
            let result = resolve_combat_tick(&mut state, &bonuses, &mut achievements, &mut rng);
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
            // Note: Enemy is cleared for boss deaths, but reset HP for non-boss deaths
            // Both cases should trigger regeneration
        }
        // If no death in 1000 ticks, that's OK - player is just winning
    }

    #[test]
    fn test_resolve_combat_tick_player_death_to_boss_clears_enemy_and_regens() {
        let mut state = GameState::new("Test Hero".to_string(), 0);

        // Set up a boss fight
        state.zone_progression.fighting_boss = true;
        state.zone_progression.kills_in_subzone = 10;

        // Spawn a strong boss that will kill the player
        state.combat_state.current_enemy = Some(crate::combat::types::Enemy::new(
            "Deadly Boss".to_string(),
            1000,
            5000, // Very high damage
        ));
        state.combat_state.player_current_hp = 1; // Player is weak

        let bonuses = CombatBonuses::default();
        let mut achievements = Achievements::default();
        let mut rng = rand::thread_rng();

        let result = resolve_combat_tick(&mut state, &bonuses, &mut achievements, &mut rng);

        assert!(result.player_died, "Player should have died");
        assert!(result.was_boss, "Should have been a boss fight");
        assert!(
            state.combat_state.is_regenerating,
            "Player death to boss should trigger regeneration"
        );
        assert!(
            state.combat_state.current_enemy.is_none(),
            "Boss should be cleared on player death"
        );
        assert!(
            !state.zone_progression.fighting_boss,
            "Boss encounter flag should be reset"
        );
        assert_eq!(
            state.zone_progression.kills_in_subzone, 0,
            "Kill counter should be reset"
        );
    }

    #[test]
    fn test_resolve_combat_tick_double_strike() {
        // Run many trials with double strike to verify it works
        let trials = 500;
        let mut double_strikes = 0;

        let bonuses = CombatBonuses {
            double_strike_chance: 50.0, // 50% double strike chance
            ..CombatBonuses::default()
        };
        let mut achievements = Achievements::default();

        for _ in 0..trials {
            let mut state = GameState::new("Test".to_string(), 0);
            let mut rng = rand::thread_rng();

            let result = resolve_combat_tick(&mut state, &bonuses, &mut achievements, &mut rng);
            if result.was_double_strike {
                double_strikes += 1;
            }
        }

        // With 50% double strike chance, expect ~250 double strikes in 500 trials
        // Allow significant variance due to RNG
        assert!(
            (200..=300).contains(&double_strikes),
            "Expected ~250 double strikes (50%), got {}",
            double_strikes
        );
    }

    #[test]
    fn test_resolve_combat_tick_record_kill_triggers_boss() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let bonuses = CombatBonuses::default();
        let mut achievements = Achievements::default();
        let mut rng = rand::thread_rng();

        // Verify we start with no kills and no boss
        assert_eq!(state.zone_progression.kills_in_subzone, 0);
        assert!(!state.zone_progression.fighting_boss);

        // Kill enough enemies to trigger boss spawn (KILLS_FOR_BOSS = 10)
        let mut kills = 0;
        for _ in 0..1000 {
            // Reset regeneration to allow combat
            state.combat_state.is_regenerating = false;

            let result = resolve_combat_tick(&mut state, &bonuses, &mut achievements, &mut rng);

            if result.player_won && !result.was_boss {
                kills += 1;
            }

            // Check if boss flag got set (happens when kills_in_subzone >= 10)
            if state.zone_progression.fighting_boss {
                break;
            }
        }

        assert!(
            state.zone_progression.fighting_boss,
            "Boss should spawn after {} kills (need 10), kills_in_subzone={}",
            kills,
            state.zone_progression.kills_in_subzone
        );
        assert!(
            kills >= 10,
            "Should need at least 10 kills, got {}",
            kills
        );
    }

    #[test]
    fn test_resolve_combat_tick_regen_after_kill() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let bonuses = CombatBonuses::default();
        let mut achievements = Achievements::default();
        let mut rng = rand::thread_rng();

        // Run until we get a kill
        for _ in 0..100 {
            let result = resolve_combat_tick(&mut state, &bonuses, &mut achievements, &mut rng);
            if result.player_won {
                // After a kill, should be regenerating
                assert!(
                    state.combat_state.is_regenerating,
                    "Should start regenerating after kill"
                );
                assert!(
                    state.combat_state.current_enemy.is_none(),
                    "Enemy should be cleared after kill"
                );
                return;
            }
        }
        panic!("Failed to get a kill in 100 ticks");
    }

    #[test]
    fn test_resolve_combat_tick_achievement_tracking() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let bonuses = CombatBonuses::default();
        let mut achievements = Achievements::default();
        let mut rng = rand::thread_rng();

        let initial_kills = achievements.total_kills;

        // Run until we get some kills
        let mut kills = 0;
        for _ in 0..200 {
            // Reset regen to keep fighting
            state.combat_state.is_regenerating = false;
            let result = resolve_combat_tick(&mut state, &bonuses, &mut achievements, &mut rng);
            if result.player_won {
                kills += 1;
                if kills >= 5 {
                    break;
                }
            }
        }

        assert!(
            achievements.total_kills > initial_kills,
            "Achievements should track kills (had {}, now {})",
            initial_kills,
            achievements.total_kills
        );
    }

    // Note: Damage reflection is tested via combat_math::calculate_damage_reflection tests
    // and combat::logic tests. The integration in resolve_combat_tick uses the same
    // formula, so we verify the code path exists rather than duplicating those tests.

    #[test]
    fn test_resolve_combat_tick_non_boss_death_resets_enemy_and_regens() {
        let mut state = GameState::new("Test Hero".to_string(), 0);

        // Spawn a strong enemy that will kill the player
        state.combat_state.current_enemy = Some(crate::combat::types::Enemy::new(
            "Strong Enemy".to_string(),
            100,  // Some HP
            5000, // Very high damage to kill player
        ));
        state.combat_state.player_current_hp = 1; // Player is weak

        let bonuses = CombatBonuses::default();
        let mut achievements = Achievements::default();
        let mut rng = rand::thread_rng();

        // This should result in player death
        let result = resolve_combat_tick(&mut state, &bonuses, &mut achievements, &mut rng);

        assert!(result.player_died, "Player should have died");
        assert!(!result.was_boss, "Should not have been a boss fight");

        // Player should be regenerating
        assert!(
            state.combat_state.is_regenerating,
            "Player death to non-boss should trigger regeneration"
        );

        // Enemy should still exist with reset HP
        assert!(
            state.combat_state.current_enemy.is_some(),
            "Non-boss enemy should remain after player death"
        );
        let enemy = state.combat_state.current_enemy.as_ref().unwrap();
        assert_eq!(
            enemy.current_hp, enemy.max_hp,
            "Enemy HP should be reset after player death"
        );
    }

    #[test]
    fn test_resolve_combat_tick_boss_defeat_tracked() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let bonuses = CombatBonuses::default();
        let mut achievements = Achievements::default();
        let mut rng = rand::thread_rng();

        // Manually set up a boss fight
        state.zone_progression.fighting_boss = true;
        state.zone_progression.kills_in_subzone = 10;

        // Spawn a very weak "boss" that player can one-shot
        state.combat_state.current_enemy = Some(crate::combat::types::Enemy::new(
            "Weak Boss".to_string(),
            1, // 1 HP = instant kill
            1,
        ));

        let initial_defeated = state.zone_progression.defeated_bosses.len();

        let result = resolve_combat_tick(&mut state, &bonuses, &mut achievements, &mut rng);

        if result.player_won && result.was_boss {
            assert!(
                state.zone_progression.defeated_bosses.len() > initial_defeated,
                "Boss defeat should be tracked in defeated_bosses"
            );
        }
    }

    #[test]
    fn test_resolve_combat_tick_boss_weapon_blocked() {
        let mut state = GameState::new("Test Hero".to_string(), 0);

        // Set up Zone 10 final boss fight (requires Stormbreaker)
        // Zone 10 has 4 subzones, the final one (id=4) is Apex Spire with The Undying Storm
        state.zone_progression.current_zone_id = 10;
        state.zone_progression.current_subzone_id = 4; // Final subzone (Apex Spire)
        state.zone_progression.fighting_boss = true;
        state.prestige_rank = 20; // High enough to access zone 10

        // Spawn a boss
        state.combat_state.current_enemy = Some(crate::combat::types::Enemy::new(
            "The Undying Storm".to_string(),
            1000,
            50,
        ));

        let bonuses = CombatBonuses::default();
        let mut achievements = Achievements::default();
        // Don't unlock Stormbreaker achievement - attack should be blocked
        let mut rng = rand::thread_rng();

        let result = resolve_combat_tick(&mut state, &bonuses, &mut achievements, &mut rng);

        // Attack should be blocked
        assert!(
            result.attack_blocked,
            "Attack should be blocked without Stormbreaker"
        );
        assert!(
            result.weapon_needed.is_some(),
            "Should indicate weapon is needed"
        );
        assert_eq!(
            result.damage_dealt, 0,
            "Should deal no damage when blocked"
        );
    }

    // ==========================================================================
    // Tests for boss defeat result tracking
    // ==========================================================================

    #[test]
    fn test_resolve_combat_tick_returns_boss_defeat_result() {
        // This test verifies that boss defeats return a BossDefeatResult
        // which is needed for proper achievement tracking (fixes zone completion bug)
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let bonuses = CombatBonuses::default();
        let mut achievements = Achievements::default();
        let mut rng = rand::thread_rng();

        // Set up a boss fight at subzone 3 of zone 1 (the final subzone)
        state.zone_progression.current_zone_id = 1;
        state.zone_progression.current_subzone_id = 3; // Final subzone of zone 1
        state.zone_progression.fighting_boss = true;
        state.zone_progression.kills_in_subzone = 10;

        // Spawn a very weak boss
        state.combat_state.current_enemy = Some(crate::combat::types::Enemy::new(
            "Zone 1 Boss".to_string(),
            1, // 1 HP = instant kill
            1,
        ));

        let result = resolve_combat_tick(&mut state, &bonuses, &mut achievements, &mut rng);

        // Should have won and got a boss defeat result
        assert!(result.player_won, "Should defeat the boss");
        assert!(result.was_boss, "Should be a boss kill");
        assert!(
            result.boss_defeat_result.is_some(),
            "Should have a boss defeat result for achievement tracking"
        );

        // The result should indicate zone completion
        let boss_result = result.boss_defeat_result.unwrap();
        match boss_result {
            BossDefeatResult::ZoneComplete { old_zone, new_zone_id } => {
                assert_eq!(old_zone, "Meadow", "Should complete Meadow zone");
                assert_eq!(new_zone_id, 2, "Should advance to zone 2");
            }
            other => {
                panic!("Expected ZoneComplete, got {:?}", other);
            }
        }

        // State should have advanced
        assert_eq!(
            state.zone_progression.current_zone_id, 2,
            "Should have advanced to zone 2"
        );
        assert!(result.zone_advanced, "zone_advanced flag should be set");
    }

    #[test]
    fn test_resolve_combat_tick_subzone_complete_result() {
        // Test that defeating a non-final subzone boss returns SubzoneComplete
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let bonuses = CombatBonuses::default();
        let mut achievements = Achievements::default();
        let mut rng = rand::thread_rng();

        // Set up a boss fight at subzone 1 of zone 1 (not the final subzone)
        state.zone_progression.current_zone_id = 1;
        state.zone_progression.current_subzone_id = 1;
        state.zone_progression.fighting_boss = true;
        state.zone_progression.kills_in_subzone = 10;

        // Spawn a very weak boss
        state.combat_state.current_enemy = Some(crate::combat::types::Enemy::new(
            "Subzone Boss".to_string(),
            1, // 1 HP = instant kill
            1,
        ));

        let result = resolve_combat_tick(&mut state, &bonuses, &mut achievements, &mut rng);

        assert!(result.player_won, "Should defeat the boss");
        assert!(
            result.boss_defeat_result.is_some(),
            "Should have a boss defeat result"
        );

        let boss_result = result.boss_defeat_result.unwrap();
        match boss_result {
            BossDefeatResult::SubzoneComplete { new_subzone_id } => {
                assert_eq!(new_subzone_id, 2, "Should advance to subzone 2");
            }
            other => {
                panic!("Expected SubzoneComplete, got {:?}", other);
            }
        }

        // Should have advanced to subzone 2, not zone 2
        assert_eq!(
            state.zone_progression.current_zone_id, 1,
            "Should still be in zone 1"
        );
        assert_eq!(
            state.zone_progression.current_subzone_id, 2,
            "Should be in subzone 2"
        );
        assert!(
            !result.zone_advanced,
            "zone_advanced should be false for subzone advancement"
        );
    }

    #[test]
    fn test_resolve_combat_tick_zone10_storms_end() {
        // Test that defeating Zone 10 final boss returns StormsEnd
        use crate::achievements::AchievementId;

        let mut state = GameState::new("Test Hero".to_string(), 0);
        let bonuses = CombatBonuses::default();
        let mut achievements = Achievements::default();
        let mut rng = rand::thread_rng();

        // Set up Zone 10 final boss fight
        state.zone_progression.current_zone_id = 10;
        state.zone_progression.current_subzone_id = 4; // Final subzone
        state.zone_progression.fighting_boss = true;
        state.zone_progression.unlock_zone(10);
        state.prestige_rank = 20;

        // Player needs Stormbreaker to damage the boss
        achievements.unlock(AchievementId::TheStormbreaker, None);

        // Spawn a very weak boss
        state.combat_state.current_enemy = Some(crate::combat::types::Enemy::new(
            "The Undying Storm".to_string(),
            1, // 1 HP = instant kill
            1,
        ));

        let result = resolve_combat_tick(&mut state, &bonuses, &mut achievements, &mut rng);

        assert!(result.player_won, "Should defeat the boss");
        assert!(
            result.boss_defeat_result.is_some(),
            "Should have a boss defeat result"
        );

        let boss_result = result.boss_defeat_result.unwrap();
        assert!(
            matches!(boss_result, BossDefeatResult::StormsEnd),
            "Expected StormsEnd result, got {:?}",
            boss_result
        );

        // Zone 11 should be unlocked
        assert!(
            state.zone_progression.is_zone_unlocked(11),
            "Zone 11 should be unlocked"
        );
        assert_eq!(
            state.zone_progression.current_zone_id, 11,
            "Should advance to Zone 11"
        );
    }

    // ==========================================================================
    // Tests for damage reflection in tick() mode (simulation)
    // ==========================================================================

    #[test]
    fn test_tick_applies_damage_reflection() {
        use crate::items::types::{Affix, AffixType, AttributeBonuses, EquipmentSlot, Item, Rarity};

        let mut game = CombatEngine::new("Test Hero".to_string());

        // Equip armor with 100% damage reflection
        let armor = Item {
            slot: EquipmentSlot::Armor,
            rarity: Rarity::Legendary,
            ilvl: 10,
            base_name: "Thorned Armor".to_string(),
            display_name: "Thorned Armor".to_string(),
            attributes: AttributeBonuses::new(),
            affixes: vec![Affix {
                affix_type: AffixType::DamageReflection,
                value: 100.0, // 100% reflection
            }],
        };
        game.state_mut()
            .equipment
            .set(EquipmentSlot::Armor, Some(armor));

        // Manually set up an enemy with known damage
        let enemy_damage = 20;
        let enemy_hp = 1000;
        game.current_enemy = Some(crate::combat::types::Enemy::new(
            "Test Enemy".to_string(),
            enemy_hp,
            enemy_damage,
        ));
        game.kills_in_subzone = 0;

        // Give player high HP so they survive
        game.state_mut().combat_state.player_current_hp = 10000;
        game.state_mut().combat_state.player_max_hp = 10000;

        let mut rng = rand::thread_rng();
        let result = game.tick(&mut rng);

        // If player didn't win (enemy survived), check that reflection was applied
        if !result.player_won && game.current_enemy.is_some() {
            let enemy = game.current_enemy.as_ref().unwrap();
            // Enemy should have taken: player's attack + reflected damage
            // With 100% reflection and 20 damage, reflected = 20
            // (assuming player has some defense, actual damage taken may be less)
            let derived = DerivedStats::calculate_derived_stats(
                &game.state().attributes,
                &game.state().equipment,
            );
            let damage_taken_by_player =
                calculate_damage_taken(enemy_damage, derived.defense);
            let reflected = calculate_damage_reflection(
                damage_taken_by_player,
                derived.damage_reflection_percent,
            );

            // Enemy HP should have decreased by player attack + reflection
            // We can't know exact player damage due to crit variance, but we can verify
            // the enemy took at least the reflection damage
            assert!(
                enemy.current_hp < enemy_hp,
                "Enemy should have taken damage from player attack"
            );

            // The enemy should have taken more damage than just the player's base attack
            // because of reflection - verify reflection was applied
            let player_damage_only = derived.total_damage();
            let expected_min_hp_loss = player_damage_only + reflected;

            // Allow for crits doubling damage
            let actual_hp_loss = enemy_hp - enemy.current_hp;
            assert!(
                actual_hp_loss >= expected_min_hp_loss || actual_hp_loss >= player_damage_only * 2,
                "Enemy should take at least {} damage (attack + reflection), took {}",
                expected_min_hp_loss,
                actual_hp_loss
            );
        }
    }

    #[test]
    fn test_tick_damage_reflection_can_kill_enemy() {
        use crate::items::types::{Affix, AffixType, AttributeBonuses, EquipmentSlot, Item, Rarity};

        let mut game = CombatEngine::new("Test Hero".to_string());

        // Equip armor with very high damage reflection
        let armor = Item {
            slot: EquipmentSlot::Armor,
            rarity: Rarity::Legendary,
            ilvl: 10,
            base_name: "Mega Thorned Armor".to_string(),
            display_name: "Mega Thorned Armor".to_string(),
            attributes: AttributeBonuses::new(),
            affixes: vec![Affix {
                affix_type: AffixType::DamageReflection,
                value: 5000.0, // 5000% reflection - massive
            }],
        };
        game.state_mut()
            .equipment
            .set(EquipmentSlot::Armor, Some(armor));

        // Low HP enemy with high damage (kills itself via reflection)
        game.current_enemy = Some(crate::combat::types::Enemy::new(
            "Suicidal Enemy".to_string(),
            10,  // Low HP
            100, // High damage
        ));
        game.kills_in_subzone = 0;

        // Give player high HP so they survive
        game.state_mut().combat_state.player_current_hp = 10000;
        game.state_mut().combat_state.player_max_hp = 10000;

        let mut rng = rand::thread_rng();
        let result = game.tick(&mut rng);

        // Enemy should be dead from combined player attack + reflection
        assert!(
            result.player_won,
            "Enemy should die from player attack + damage reflection"
        );
    }

    // ==========================================================================
    // Tests for Zone 10/11 progression (advance_after_boss_kill)
    // ==========================================================================

    #[test]
    fn test_zone_10_completion_unlocks_zone_11() {
        use crate::achievements::AchievementId;

        let mut state = GameState::new("Test Hero".to_string(), 0);
        let bonuses = CombatBonuses::default();
        let mut achievements = Achievements::default();
        let mut rng = rand::thread_rng();

        // Set up Zone 10 final boss fight with Stormbreaker
        state.zone_progression.current_zone_id = 10;
        state.zone_progression.current_subzone_id = 4; // Final subzone
        state.zone_progression.fighting_boss = true;
        state.zone_progression.unlock_zone(10);
        state.prestige_rank = 20;

        // Unlock Stormbreaker so we can actually defeat the boss
        achievements.unlock(AchievementId::TheStormbreaker, None);

        // Spawn a very weak "boss" that player can one-shot
        state.combat_state.current_enemy = Some(crate::combat::types::Enemy::new(
            "The Undying Storm".to_string(),
            1, // 1 HP = instant kill
            1,
        ));

        let result = resolve_combat_tick(&mut state, &bonuses, &mut achievements, &mut rng);

        assert!(result.player_won, "Should defeat the Zone 10 boss");
        assert!(result.was_boss, "Should recognize as boss fight");

        // Zone 11 should be unlocked
        assert!(
            state.zone_progression.is_zone_unlocked(11),
            "Zone 11 (The Expanse) should be unlocked after Zone 10 completion"
        );

        // Should have advanced to Zone 11, Subzone 1
        assert_eq!(
            state.zone_progression.current_zone_id, 11,
            "Should advance to Zone 11"
        );
        assert_eq!(
            state.zone_progression.current_subzone_id, 1,
            "Should start at subzone 1 of Zone 11"
        );

        // StormsEnd achievement should be unlocked
        assert!(
            achievements.is_unlocked(AchievementId::StormsEnd),
            "StormsEnd achievement should be unlocked"
        );
    }

    #[test]
    fn test_zone_11_cycles_back_to_subzone_1() {
        use crate::achievements::AchievementId;

        let mut state = GameState::new("Test Hero".to_string(), 0);
        let bonuses = CombatBonuses::default();
        let mut achievements = Achievements::default();
        let mut rng = rand::thread_rng();

        // Set up Zone 11 final boss fight
        state.zone_progression.current_zone_id = 11;
        state.zone_progression.current_subzone_id = 4; // Final subzone of The Expanse
        state.zone_progression.fighting_boss = true;
        state.zone_progression.unlock_zone(11);
        state.prestige_rank = 20;

        // Stormbreaker needed to have reached Zone 11
        achievements.unlock(AchievementId::TheStormbreaker, None);
        achievements.unlock(AchievementId::StormsEnd, None);

        // Spawn a very weak "boss" that player can one-shot
        state.combat_state.current_enemy = Some(crate::combat::types::Enemy::new(
            "Avatar of Infinity".to_string(),
            1, // 1 HP = instant kill
            1,
        ));

        let result = resolve_combat_tick(&mut state, &bonuses, &mut achievements, &mut rng);

        assert!(result.player_won, "Should defeat the Zone 11 boss");
        assert!(result.was_boss, "Should recognize as boss fight");

        // Should stay in Zone 11 but cycle back to Subzone 1
        assert_eq!(
            state.zone_progression.current_zone_id, 11,
            "Should remain in Zone 11 (The Expanse)"
        );
        assert_eq!(
            state.zone_progression.current_subzone_id, 1,
            "Should cycle back to subzone 1"
        );

        // Kills should be reset
        assert_eq!(
            state.zone_progression.kills_in_subzone, 0,
            "Kills should be reset after Zone 11 boss"
        );

        // Boss defeat should be recorded
        assert!(
            state.zone_progression.is_boss_defeated(11, 4),
            "Zone 11 boss defeat should be recorded"
        );
    }

    #[test]
    fn test_zone_advancement_updates_result_new_zone() {
        let mut state = GameState::new("Test Hero".to_string(), 0);
        let bonuses = CombatBonuses::default();
        let mut achievements = Achievements::default();
        let mut rng = rand::thread_rng();

        // Set up Zone 1 final boss fight
        state.zone_progression.current_zone_id = 1;
        state.zone_progression.current_subzone_id = 3; // Final subzone of Meadow
        state.zone_progression.fighting_boss = true;

        // Spawn a very weak "boss" that player can one-shot
        state.combat_state.current_enemy = Some(crate::combat::types::Enemy::new(
            "Sporeling Queen".to_string(),
            1, // 1 HP = instant kill
            1,
        ));

        let result = resolve_combat_tick(&mut state, &bonuses, &mut achievements, &mut rng);

        assert!(result.player_won, "Should defeat the Zone 1 boss");
        assert!(result.zone_advanced, "Should advance to next zone");
        assert_eq!(result.new_zone, 2, "Should advance to Zone 2 (Dark Forest)");
    }
}
