//! Tick stage helper functions extracted from `game_tick()`.
//!
//! Each function implements one processing stage of the game tick loop,
//! keeping `game_tick()` in `tick.rs` readable as a high-level orchestrator.

use super::tick_types::{TickEvent, TickResult};
use crate::achievements::Achievements;
use crate::challenges::ActiveMinigame;
use crate::combat::events::CombatBonuses;
use crate::combat::logic::update_combat;
use crate::combat::CombatEvent;
use crate::core::constants::{
    FINAL_ZONE_ID, HAVEN_MIN_PRESTIGE_RANK, STORMGLASS_MIN_PRESTIGE_RANK, TICKS_PER_SECOND,
};
use crate::core::game_logic::{
    apply_tick_xp, process_level_ups_from_current_xp, try_discover_dungeon,
};
use crate::core::game_state::GameState;
use crate::dungeon::logic::{
    on_boss_defeated, on_elite_defeated, on_room_enemy_defeated, update_dungeon,
};
use crate::dungeon::rewards::{add_dungeon_xp, calculate_boss_xp_reward, on_treasure_room_entered};
use crate::dungeon::types::RoomType;
use crate::fishing::{
    check_rank_up_with_max, get_max_fishing_rank, tick_fishing_with_haven_result,
    HavenFishingBonuses,
};
use crate::haven::{Haven, HavenBonuses};
use crate::items::drops::{try_drop_from_boss, try_drop_from_mob};
use crate::items::scoring::auto_equip_if_better;
use crate::items::types::Rarity;
use crate::stormglass::sigils::SigilBonuses;
use crate::zones::BossDefeatResult;
use rand::{Rng, RngExt};

/// Apply XP to the player and emit a LeveledUp event if the character leveled up.
/// Also notifies the achievement system about any new levels reached.
fn apply_xp_and_check_levelup<R: Rng>(
    rng: &mut R,
    state: &mut GameState,
    xp: f64,
    achievements: &mut Achievements,
    result: &mut TickResult,
) {
    let level_before = state.character_level;
    apply_tick_xp(rng, state, xp);
    if state.character_level > level_before {
        for lvl in (level_before + 1)..=state.character_level {
            achievements.on_level_up(lvl, Some(&state.character_name));
        }
        result.events.push(TickEvent::LeveledUp {
            new_level: state.character_level,
        });
    }
}

/// Stage 4: Process dungeon exploration events.
///
/// Calls `update_dungeon()` and maps `DungeonEvent` variants to `TickEvent`s.
/// Handles room entry narration, treasure rooms, keys, boss unlock, completion,
/// and failure.
pub fn process_dungeon_events<R: Rng>(
    state: &mut GameState,
    delta_time: f64,
    haven_bonuses: &HavenBonuses,
    result: &mut TickResult,
    rng: &mut R,
) {
    if state.active_dungeon.is_none() {
        return;
    }

    let god_item_dungeon_speed = state.cached_god_item_bonuses.dungeon_speed_percent;
    let dungeon_events = update_dungeon(state, delta_time, god_item_dungeon_speed);
    for event in dungeon_events {
        match event {
            crate::dungeon::logic::DungeonEvent::EnteredRoom { room_type, .. } => {
                let narration = room_type.narration();
                let line = narration[rng.random_range(0..narration.len())];
                let message = format!("\u{1f6aa} {}", line);
                result
                    .events
                    .push(TickEvent::DungeonRoomEntered { room_type, message });

                // Handle treasure room
                if room_type == RoomType::Treasure {
                    if let Some((item, equipped)) =
                        on_treasure_room_entered(rng, state, haven_bonuses.item_rarity_percent)
                    {
                        let power = item.power();
                        let treasure_rarity = item.rarity;
                        let treasure_name = item.display_name.clone();
                        result.events.push(TickEvent::DungeonTreasureFound {
                            item_name: item.display_name,
                            rarity: item.rarity,
                            tier: item.tier,
                            ilvl: item.ilvl,
                            power,
                            equipped,
                        });

                        // Stormglass: salvage non-equipped treasure items
                        // Discovery requires P15+; once discovered, salvage always works
                        if !equipped
                            && (state.stormglass_discovered
                                || state.prestige_rank >= STORMGLASS_MIN_PRESTIGE_RANK)
                        {
                            let sg_amount =
                                crate::stormglass::earning::salvage_value(treasure_rarity);
                            state.stormglass += sg_amount;

                            if !state.stormglass_discovered {
                                state.stormglass_discovered = true;
                                result.events.push(TickEvent::StormglassDiscovered);
                            }

                            result.events.push(TickEvent::StormglassSalvaged {
                                item_name: treasure_name,
                                rarity: treasure_rarity,
                                amount: sg_amount,
                            });
                        }
                    }

                    // Stormglass: dungeon cache from treasure room
                    if state.stormglass_discovered
                        || state.prestige_rank >= STORMGLASS_MIN_PRESTIGE_RANK
                    {
                        if let Some(dungeon) = &state.active_dungeon {
                            let cache_amount =
                                crate::stormglass::earning::dungeon_cache(dungeon.size);
                            state.stormglass += cache_amount;
                            result.events.push(TickEvent::StormglassDungeonCache {
                                amount: cache_amount,
                            });
                        }
                    }
                }
            }
            crate::dungeon::logic::DungeonEvent::FoundKey => {
                result.events.push(TickEvent::DungeonKeyFound);
            }
            crate::dungeon::logic::DungeonEvent::BossUnlocked => {
                result.events.push(TickEvent::DungeonBossUnlocked);
            }
            crate::dungeon::logic::DungeonEvent::DungeonComplete {
                xp_earned,
                items_collected,
            } => {
                result.events.push(TickEvent::DungeonCompleted {
                    xp_earned,
                    items_collected,
                });
            }
            crate::dungeon::logic::DungeonEvent::DungeonFailed => {
                result.events.push(TickEvent::DungeonFailed);
            }
            _ => {}
        }
    }
}

/// Stage 5: Process fishing tick.
///
/// Ticks the fishing session, handles catches/items/rank-ups/Leviathan,
/// updates play time, and collects achievements. Returns `true` if fishing
/// was active (caller should skip combat stages).
#[allow(clippy::too_many_arguments)]
pub fn process_fishing_tick<R: Rng>(
    state: &mut GameState,
    tick_counter: &mut u32,
    delta_time: f64,
    haven_bonuses: &HavenBonuses,
    achievements: &mut Achievements,
    debug_mode: bool,
    result: &mut TickResult,
    rng: &mut R,
) -> bool {
    if state.active_fishing.is_none() {
        return false;
    }

    let haven_fishing = HavenFishingBonuses {
        timer_reduction_percent: haven_bonuses.fishing_timer_reduction,
        double_fish_chance_percent: haven_bonuses.double_fish_chance,
        max_fishing_rank_bonus: haven_bonuses.max_fishing_rank_bonus,
    };
    let god_item_fishing_reduction = state.cached_god_item_bonuses.fishing_reduction_percent;
    let fishing_result =
        tick_fishing_with_haven_result(state, rng, &haven_fishing, god_item_fishing_reduction);
    let level_before = state.character_level;

    // Storm Leviathan caught -> achievement
    if fishing_result.caught_storm_leviathan {
        state.fishing.leviathan_caught = true;
        achievements.on_storm_leviathan_caught(Some(&state.character_name));
        result.events.push(TickEvent::StormLeviathanCaught);
        if !debug_mode {
            result.achievements_changed = true;
        }
    }

    // Process fishing messages
    for message in &fishing_result.messages {
        let prefixed = format!("\u{1f3a3} {}", message);

        if message.contains("Caught") {
            let rarity = if message.contains("[Legendary]") {
                Rarity::Legendary
            } else if message.contains("[Epic]") {
                Rarity::Epic
            } else if message.contains("[Rare]") {
                Rarity::Rare
            } else if message.contains("[Uncommon]") {
                Rarity::Magic
            } else {
                Rarity::Common
            };
            let fish_name = message
                .split("Caught ")
                .nth(1)
                .and_then(|s| s.split(" [").next())
                .unwrap_or("Fish")
                .to_string();
            state.add_recent_drop(
                fish_name.clone(),
                rarity,
                false,
                "\u{1f41f}",
                String::new(),
                String::new(),
            );
            achievements.on_fish_caught(Some(&state.character_name));
            result.events.push(TickEvent::FishCaught {
                fish_name,
                rarity,
                message: prefixed,
            });
        } else if message.contains("Found item:") {
            let item_name = message
                .split("Found item: ")
                .nth(1)
                .map(|s| s.trim_end_matches('!'))
                .unwrap_or("Item")
                .to_string();
            state.add_recent_drop(
                item_name.clone(),
                Rarity::Rare,
                false,
                "\u{1f4e6}",
                String::new(),
                String::new(),
            );
            result.events.push(TickEvent::FishingItemFound {
                item_name,
                message: prefixed,
            });
        } else {
            result
                .events
                .push(TickEvent::FishingMessage { message: prefixed });
        }
    }

    // Fishing awards XP directly; resolve any pending level-ups from that XP.
    let (level_ups, _) = process_level_ups_from_current_xp(rng, state);
    if level_ups > 0 {
        for lvl in (level_before + 1)..=state.character_level {
            achievements.on_level_up(lvl, Some(&state.character_name));
        }
        result.events.push(TickEvent::LeveledUp {
            new_level: state.character_level,
        });
    }

    // Check fishing rank up
    let max_rank = get_max_fishing_rank(haven_fishing.max_fishing_rank_bonus);
    let rank_before = state.fishing.rank;
    if let Some(rank_msg) = check_rank_up_with_max(&mut state.fishing, max_rank) {
        if state.fishing.rank > rank_before {
            achievements.on_fishing_rank_up(state.fishing.rank, Some(&state.character_name));
        }
        let prefixed = format!("\u{1f3a3} {}", rank_msg);
        result
            .events
            .push(TickEvent::FishingRankUp { message: prefixed });
    }

    // Leviathan encounter
    result.leviathan_encounter = fishing_result.leviathan_encounter;
    result.leviathan_lure_consumed = fishing_result.lure_consumed;
    result.leviathan_catch_miss = fishing_result.leviathan_catch_miss;

    // Update play time while fishing
    *tick_counter += 1;
    if *tick_counter >= TICKS_PER_SECOND {
        state.play_time_seconds += 1;
        if state.combat_seconds_this_tick {
            state.xp_rate_samples.push_back(state.xp_this_second);
            if state.xp_rate_samples.len() > crate::core::constants::XP_RATE_WINDOW_SECONDS {
                state.xp_rate_samples.pop_front();
            }
        }
        state.xp_this_second = 0;
        state.combat_seconds_this_tick = false;
        *tick_counter = 0;
    }

    // Decay HUD flashes even while fishing
    state.combat_state.tick_hud(delta_time);

    true
}

/// Stage 6: Map combat events to tick events.
///
/// Processes the `Vec<CombatEvent>` from `update_combat()`, maps each to
/// the appropriate `TickEvent`, applies XP, handles kills/deaths, processes
/// item drops and discoveries.
#[allow(clippy::too_many_arguments)]
pub fn process_combat_events<R: Rng>(
    state: &mut GameState,
    combat_events: Vec<CombatEvent>,
    haven_bonuses: &HavenBonuses,
    achievements: &mut Achievements,
    deep: &mut crate::deep::DeepState,
    debug_mode: bool,
    result: &mut TickResult,
    rng: &mut R,
) {
    for event in combat_events {
        match event {
            CombatEvent::PlayerAttackBlocked { weapon_needed } => {
                result
                    .events
                    .push(TickEvent::PlayerAttackBlocked { weapon_needed });
            }
            CombatEvent::PlayerAttack { damage, was_crit } => {
                result
                    .events
                    .push(TickEvent::PlayerAttack { damage, was_crit });
            }
            CombatEvent::EnemyAttack { damage } => {
                let enemy_name = state
                    .combat_state
                    .current_enemy
                    .as_ref()
                    .map(|e| e.name.clone())
                    .unwrap_or_default();
                result
                    .events
                    .push(TickEvent::EnemyAttack { damage, enemy_name });
            }
            CombatEvent::DamageReflected { damage } => {
                result.events.push(TickEvent::DamageReflected { damage });
            }
            CombatEvent::RegenComplete { healed } => {
                result.events.push(TickEvent::RegenComplete { healed });
            }
            CombatEvent::EnemyDied { xp_gained } => {
                let enemy_name = state
                    .combat_state
                    .current_enemy
                    .as_ref()
                    .map(|e| e.name.clone())
                    .unwrap_or_default();
                result.events.push(TickEvent::EnemyDefeated {
                    xp_gained,
                    enemy_name,
                });

                // Apply XP and check level up
                apply_xp_and_check_levelup(rng, state, xp_gained as f64, achievements, result);
                state.session_kills += 1;

                // Track XP in dungeon and mark room cleared
                add_dungeon_xp(state, xp_gained);
                if let Some(dungeon) = &mut state.active_dungeon {
                    on_room_enemy_defeated(dungeon);
                }

                // Item drops
                process_item_drop(state, haven_bonuses, result);

                // Discovery: dungeon, then fishing
                process_discoveries(state, rng, result);
            }
            CombatEvent::EliteDefeated { xp_gained } => {
                let enemy_name = state
                    .combat_state
                    .current_enemy
                    .as_ref()
                    .map(|e| e.name.clone())
                    .unwrap_or_default();
                result.events.push(TickEvent::DungeonEliteDefeated {
                    xp_gained,
                    enemy_name,
                });

                apply_xp_and_check_levelup(rng, state, xp_gained as f64, achievements, result);
                add_dungeon_xp(state, xp_gained);

                // Give key
                if let Some(dungeon) = &mut state.active_dungeon {
                    let events = on_elite_defeated(dungeon);
                    for de in events {
                        if matches!(de, crate::dungeon::logic::DungeonEvent::FoundKey) {
                            result.events.push(TickEvent::DungeonKeyFound);
                        }
                    }
                }
            }
            CombatEvent::BossDefeated { xp_gained } => {
                let enemy_name = state
                    .combat_state
                    .current_enemy
                    .as_ref()
                    .map(|e| e.name.clone())
                    .unwrap_or_default();

                // Calculate boss bonus XP
                let (bonus_xp, total_xp, items) = if let Some(dungeon) = &state.active_dungeon {
                    let bonus = calculate_boss_xp_reward(rng, dungeon.size);
                    let total = dungeon.xp_earned + xp_gained + bonus;
                    let item_count = dungeon.collected_items.len();
                    (bonus, total, item_count)
                } else {
                    (0, xp_gained, 0)
                };

                apply_xp_and_check_levelup(
                    rng,
                    state,
                    (xp_gained + bonus_xp) as f64,
                    achievements,
                    result,
                );

                achievements.on_dungeon_completed(Some(&state.character_name));

                result.events.push(TickEvent::DungeonBossDefeated {
                    xp_gained,
                    bonus_xp,
                    total_xp,
                    items_collected: items,
                    enemy_name,
                });

                // Clear dungeon
                let _events = on_boss_defeated(state);
            }
            CombatEvent::BossEnrage {
                weapon_blocked,
                enemy_name,
            } => {
                let message = if weapon_blocked {
                    format!(
                        "\u{1f525} {} enrages! You lack the weapon to challenge this foe. Retreating to safety...",
                        enemy_name
                    )
                } else {
                    format!(
                        "\u{1f525} {} enrages, striking you down! Boss encounter reset.",
                        enemy_name
                    )
                };
                result.events.push(TickEvent::BossEnrage { message });
            }
            CombatEvent::PlayerDiedInDungeon => {
                result.events.push(TickEvent::PlayerDiedInDungeon);
            }
            CombatEvent::PlayerDied => {
                result.events.push(TickEvent::PlayerDied);
            }
            CombatEvent::SubzoneBossDefeated {
                xp_gained,
                result: defeat_result,
            } => {
                apply_xp_and_check_levelup(rng, state, xp_gained as f64, achievements, result);
                state.session_kills += 1;

                // Track zone achievements
                process_zone_achievements(&defeat_result, achievements, &state.character_name);

                // WeaponRequired is already handled by PlayerAttackBlocked
                if matches!(defeat_result, BossDefeatResult::WeaponRequired { .. }) {
                    continue;
                }

                // Deep discovery: first Endless kill at P15+
                // Check before moving defeat_result into the event
                let is_expanse_cycle = matches!(defeat_result, BossDefeatResult::ExpanseCycle);
                // Vessel signal: first Zone 50 final boss kill (cap-zone cycle)
                let is_z50_cycle = matches!(
                    defeat_result,
                    BossDefeatResult::LoomZoneCycle { zone_id: 50 }
                );

                result.events.push(TickEvent::SubzoneBossDefeated {
                    xp_gained,
                    result: defeat_result,
                });

                if is_expanse_cycle
                    && !deep.persistent.discovered
                    && state.prestige_rank >= crate::deep::DEEP_MIN_PRESTIGE_RANK
                {
                    crate::deep::complete_discovery(deep, rng);
                    result.events.push(TickEvent::DeepDiscovered);
                    result.deep_changed = true;
                    achievements.on_deep_discovered(Some(&state.character_name));
                    if !debug_mode {
                        result.achievements_changed = true;
                    }
                }

                if is_z50_cycle && !state.vessel_signal_discovered {
                    state.vessel_signal_discovered = true;
                    result.events.push(TickEvent::VesselSignalDiscovered);
                }
            }
            CombatEvent::CombatRetreat { zone_name } => {
                result.events.push(TickEvent::CombatRetreat { zone_name });
            }
            CombatEvent::DungeonRetreat => {
                result.events.push(TickEvent::DungeonFailed);
            }
        }
    }
}

/// Process item drops after killing a mob/boss in overworld combat.
pub(super) fn process_item_drop(
    state: &mut GameState,
    haven_bonuses: &HavenBonuses,
    result: &mut TickResult,
) {
    let zone_id = state.zone_progression.current_zone_id as usize;
    let was_boss = state.zone_progression.fighting_boss;
    let is_final_zone = zone_id == FINAL_ZONE_ID as usize;

    let dropped_item = if was_boss {
        Some(try_drop_from_boss(zone_id, is_final_zone))
    } else {
        try_drop_from_mob(
            state,
            zone_id,
            haven_bonuses.drop_rate_percent,
            haven_bonuses.item_rarity_percent,
        )
    };

    if let Some(item) = dropped_item {
        let item_name = item.display_name.clone();
        let rarity = item.rarity;
        let tier = item.tier;
        let ilvl = item.ilvl;
        let power = item.power();
        let slot = item.slot_name().to_string();
        let stats = item.stat_summary();
        let icon = if was_boss { "\u{1f451}" } else { "\u{1f381}" };
        let equipped = auto_equip_if_better(item, state);
        if equipped {
            state.invalidate_derived_stats();
        }
        // Check if stormglass salvage will occur (needs item_name later)
        let will_salvage = !equipped
            && (state.stormglass_discovered || state.prestige_rank >= STORMGLASS_MIN_PRESTIGE_RANK);

        // Clone for add_recent_drop; move originals into events
        state.add_recent_drop(
            item_name.clone(),
            rarity,
            equipped,
            icon,
            slot.clone(),
            stats.clone(),
        );

        // If stormglass salvage needs item_name too, clone for ItemDropped and move into salvage
        if will_salvage {
            result.events.push(TickEvent::ItemDropped {
                item_name: item_name.clone(),
                rarity,
                tier,
                ilvl,
                power,
                equipped,
                slot,
                stats,
                from_boss: was_boss,
            });

            let sg_amount = crate::stormglass::earning::salvage_value(rarity);
            state.stormglass += sg_amount;

            // First salvage triggers discovery
            if !state.stormglass_discovered {
                state.stormglass_discovered = true;
                result.events.push(TickEvent::StormglassDiscovered);
            }

            result.events.push(TickEvent::StormglassSalvaged {
                item_name,
                rarity,
                amount: sg_amount,
            });
        } else {
            result.events.push(TickEvent::ItemDropped {
                item_name,
                rarity,
                tier,
                ilvl,
                power,
                equipped,
                slot,
                stats,
                from_boss: was_boss,
            });
        }
    }
}

/// Try to discover dungeon or fishing spot after killing an enemy.
pub(super) fn process_discoveries<R: Rng>(
    state: &mut GameState,
    rng: &mut R,
    result: &mut TickResult,
) {
    // Try dungeon discovery (only outside dungeons)
    let discovered_dungeon = state.active_dungeon.is_none() && try_discover_dungeon(rng, state);
    if discovered_dungeon {
        result.events.push(TickEvent::DungeonDiscovered {
            message: "\u{1f300} You notice a dark passage leading underground...".to_string(),
        });
    }

    // Try fishing spot discovery (only if no dungeon or fishing active)
    if !discovered_dungeon && state.active_dungeon.is_none() && state.active_fishing.is_none() {
        if let Some(message) = crate::fishing::discovery::try_discover_fishing(state, rng) {
            result.events.push(TickEvent::FishingSpotDiscovered {
                message: format!("\u{1f3a3} {}", message),
            });
        }
    }
}

/// Track zone completion achievements based on boss defeat result.
pub(super) fn process_zone_achievements(
    defeat_result: &BossDefeatResult,
    achievements: &mut Achievements,
    character_name: &str,
) {
    match defeat_result {
        BossDefeatResult::ZoneComplete {
            old_zone_id,
            old_zone: _,
            ..
        }
        | BossDefeatResult::ZoneCompleteButGated {
            old_zone_id,
            zone_name: _,
            ..
        } => {
            achievements.on_zone_fully_cleared(*old_zone_id, Some(character_name));
        }
        BossDefeatResult::StormsEnd => {
            achievements.on_zone_fully_cleared(10, Some(character_name));
            achievements.on_storms_end(Some(character_name));
        }
        BossDefeatResult::ExpanseCycle => {
            achievements.on_zone_fully_cleared(11, Some(character_name));
        }
        BossDefeatResult::FractureCycle { zone_id }
        | BossDefeatResult::LoomZoneCycle { zone_id }
        | BossDefeatResult::FrontierBackoff { zone_id, .. } => {
            achievements.on_zone_fully_cleared(*zone_id, Some(character_name));
        }
        _ => {}
    }
}

/// Track prestige rank gained passively during this tick (Power Cores,
/// WR→PR conversion). Manual prestige and challenge rewards are input-driven
/// and tracked by `main_helpers::achievements::track_input_achievements`.
pub(super) fn track_passive_prestige_gain(
    state: &GameState,
    achievements: &mut Achievements,
    prestige_before: u32,
) {
    if state.prestige_rank > prestige_before {
        achievements.on_prestige(state.prestige_rank, Some(&state.character_name));
    }
}

/// Emit an atmospheric Vessel whisper roughly every 60 seconds of play time
/// once the signal is discovered, until the Vessel launches. Rotates through
/// the whisper list deterministically (no RNG).
pub fn tick_vessel_whispers(state: &mut GameState, result: &mut TickResult) {
    if !state.vessel_signal_discovered || state.vessel_launched {
        return;
    }
    let elapsed = state
        .play_time_seconds
        .saturating_sub(state.vessel_last_whisper_at);
    if elapsed >= crate::vessel::WHISPER_INTERVAL_SECONDS {
        state.vessel_last_whisper_at = state.play_time_seconds;
        let index = state.play_time_seconds / crate::vessel::WHISPER_INTERVAL_SECONDS;
        result.events.push(TickEvent::VesselWhisper {
            message: crate::vessel::whisper_message(index),
        });
    }
}

/// Collect newly unlocked achievements into TickResult events.
pub(super) fn collect_achievement_events(achievements: &mut Achievements, result: &mut TickResult) {
    for id in achievements.take_newly_unlocked() {
        if let Some(def) = crate::achievements::get_achievement_def(id) {
            result.events.push(TickEvent::AchievementUnlocked {
                name: def.name.to_string(),
            });
            result.achievements_changed = true;
        }
    }
}

/// Stage 1: Tick AI thinking for any active challenge minigame.
pub(super) fn tick_challenge_ai<R: Rng>(state: &mut GameState, rng: &mut R) {
    match &mut state.active_minigame {
        Some(ActiveMinigame::Chess(game)) => {
            crate::challenges::chess::logic::process_ai_thinking(game, rng);
        }
        Some(ActiveMinigame::Morris(game)) => {
            crate::challenges::morris::logic::process_ai_thinking(game, rng);
        }
        Some(ActiveMinigame::Gomoku(game)) => {
            crate::challenges::gomoku::logic::process_ai_thinking(game, rng);
        }
        Some(ActiveMinigame::Go(game)) => {
            crate::challenges::go::process_ai_thinking(game, rng);
        }
        Some(ActiveMinigame::ShardFusion(game)) => {
            crate::challenges::shard_fusion::tick_shard_fusion(game, rng);
        }
        _ => {}
    }
}

/// Stage 2: Try to discover a new challenge minigame (skipped during Chrono Surge).
pub(super) fn tick_challenge_discovery<R: Rng>(
    state: &mut GameState,
    haven_bonuses: &HavenBonuses,
    rng: &mut R,
    result: &mut TickResult,
) {
    if state.chrono_surge_active {
        return;
    }
    let haven_discovery = haven_bonuses.challenge_discovery_percent;
    if let Some(challenge_type) =
        crate::challenges::menu::try_discover_challenge_with_haven(state, rng, haven_discovery)
    {
        let icon = challenge_type.icon();
        let flavor = challenge_type.discovery_flavor();
        result.events.push(TickEvent::ChallengeDiscovered {
            challenge_type,
            message: format!("{} {}", icon, flavor),
            follow_up: format!("{} Press [Tab] to view pending challenges", icon),
        });
    }
}

/// Stage 3: Recalculate derived stats if dirty, then sync max HP.
///
/// Applies ALL HP bonuses (flat prestige HP, ascension multiplier, sigil max HP%)
/// so that `player_max_hp` is always the fully-boosted value. This prevents
/// a snap-back where regen fills HP to boosted max, then the next tick briefly
/// resets max HP to base before Stage 6 re-applies bonuses.
pub(super) fn sync_derived_stats(
    state: &mut GameState,
    enhancement: &crate::enhancement::EnhancementProgress,
    sigil_bonuses: &SigilBonuses,
) {
    if state.derived_stats_dirty {
        state.recalculate_derived_stats(&enhancement.levels);
        state.recalculate_prestige_bonuses();
        state.cached_god_item_bonuses =
            crate::god_items::CachedGodItemBonuses::compute(&state.equipment);
    }
    let derived = state.cached_derived_stats;
    let mut max_hp = derived.max_hp;

    // Apply flat HP bonus from prestige
    let prestige_combat = state.cached_prestige_bonuses;
    if prestige_combat.flat_hp > 0 {
        max_hp += prestige_combat.flat_hp;
    }

    // Apply Ascension multiplier
    let ascension_mult = crate::ascension::ascension_combat_multiplier(state.ascension_level);
    if ascension_mult > 1.0 {
        max_hp = (max_hp as f64 * ascension_mult) as u64;
    }

    // Apply sigil max HP% bonus
    if sigil_bonuses.max_hp_percent > 0.0 {
        max_hp = (max_hp as f64 * (1.0 + sigil_bonuses.max_hp_percent / 100.0)) as u64;
    }

    state.combat_state.update_max_hp(max_hp);
}

/// Compute merged Haven + Sigil bonuses for the current tick.
/// Returns cached values if bonuses_dirty is false; recomputes and caches when dirty.
pub(super) fn compute_merged_bonuses(
    haven: &Haven,
    state: &mut GameState,
) -> (HavenBonuses, SigilBonuses) {
    if !state.bonuses_dirty {
        return (state.cached_haven_bonuses, state.cached_sigil_bonuses);
    }

    let mut haven_bonuses = haven.compute_bonuses();
    let sigil_bonuses = SigilBonuses::compute(&state.storm_sigils);

    // Inject sigil bonuses into haven_bonuses for fields that flow through
    // process_item_drop (drop_rate) and process_fishing_tick (fishing speed)
    haven_bonuses.drop_rate_percent += sigil_bonuses.drop_rate_percent;
    haven_bonuses.fishing_timer_reduction += sigil_bonuses.fishing_speed_percent;

    state.cached_haven_bonuses = haven_bonuses;
    state.cached_sigil_bonuses = sigil_bonuses;
    state.bonuses_dirty = false;

    (haven_bonuses, sigil_bonuses)
}

/// Stage 6: Build CombatBonuses, run combat, and process events.
#[allow(clippy::too_many_arguments)]
pub(super) fn run_combat<R: Rng>(
    state: &mut GameState,
    delta_time: f64,
    haven_bonuses: &HavenBonuses,
    sigil_bonuses: &SigilBonuses,
    achievements: &mut Achievements,
    deep: &mut crate::deep::DeepState,
    _loom: &crate::loom::LoomState,
    debug_mode: bool,
    result: &mut TickResult,
    rng: &mut R,
) {
    let prestige_combat = state.cached_prestige_bonuses;
    let derived = state.cached_derived_stats;
    let combat_bonuses = CombatBonuses {
        // Haven bonuses
        hp_regen_percent: haven_bonuses.hp_regen_percent,
        hp_regen_delay_reduction: haven_bonuses.hp_regen_delay_reduction,
        damage_percent: haven_bonuses.damage_percent + sigil_bonuses.damage_percent,
        crit_chance_percent: haven_bonuses.crit_chance_percent
            + sigil_bonuses.crit_chance_percent
            + prestige_combat.crit_chance,
        double_strike_chance: haven_bonuses.double_strike_chance
            + sigil_bonuses.double_strike_percent,
        xp_gain_percent: haven_bonuses.xp_gain_percent + sigil_bonuses.xp_percent,
        // God item bonuses (cached, recomputed only when equipment changes)
        early_damage_percent: state.cached_god_item_bonuses.damage_percent,
        damage_reduction_percent: state.cached_god_item_bonuses.damage_reduction_percent
            + sigil_bonuses.damage_reduction_percent,
        attack_speed_percent: state.cached_god_item_bonuses.attack_speed_percent
            + sigil_bonuses.attack_speed_percent,
        regen_reduction_percent: state.cached_god_item_bonuses.regen_reduction_percent
            + sigil_bonuses.regen_delay_percent,
        // Prestige flat bonuses
        flat_damage: prestige_combat.flat_damage,
        flat_defense: prestige_combat.flat_defense,
        // Ascension multiplier from per-character Ascension level
        ascension_multiplier: crate::ascension::ascension_combat_multiplier(state.ascension_level),
    };
    // NOTE: HP bonuses (flat_hp, ascension multiplier, sigil max HP%) are now
    // applied in sync_derived_stats (Stage 3) to prevent regen snap-back where
    // max HP is briefly reset to base between ticks.

    // Update cached power rating
    state.cached_power_rating = crate::core::power_rating::compute_power_rating(
        &derived,
        &combat_bonuses,
        state.combat_state.player_max_hp,
    );

    let loom_zone_cap = state.cached_loom_zone_cap;
    let combat_events = update_combat(
        rng,
        state,
        delta_time,
        &combat_bonuses,
        achievements,
        &derived,
        deep.persistent.fracture_zone_cap,
        loom_zone_cap,
    );

    process_combat_events(
        state,
        combat_events,
        haven_bonuses,
        achievements,
        deep,
        debug_mode,
        result,
        rng,
    );
}

/// Stage 8: Increment tick counter and update play time every second.
pub(super) fn update_play_time(state: &mut GameState, tick_counter: &mut u32) {
    *tick_counter += 1;
    if *tick_counter >= TICKS_PER_SECOND {
        state.play_time_seconds += 1;
        if state.combat_seconds_this_tick {
            state.xp_rate_samples.push_back(state.xp_this_second);
            if state.xp_rate_samples.len() > crate::core::constants::XP_RATE_WINDOW_SECONDS {
                state.xp_rate_samples.pop_front();
            }
        }
        state.xp_this_second = 0;
        state.combat_seconds_this_tick = false;
        *tick_counter = 0;
    }
}

/// Stage 10: Roll for Haven discovery (P10+, no active content).
pub(super) fn tick_haven_discovery(
    state: &GameState,
    haven: &mut Haven,
    achievements: &mut Achievements,
    debug_mode: bool,
    result: &mut TickResult,
    rng: &mut impl Rng,
) {
    if !haven.discovered
        && state.prestige_rank >= HAVEN_MIN_PRESTIGE_RANK
        && state.active_dungeon.is_none()
        && state.active_fishing.is_none()
        && state.active_minigame.is_none()
        && crate::haven::try_discover_haven(haven, state.prestige_rank, rng)
    {
        achievements.on_haven_discovered(Some(&state.character_name));
        result.events.push(TickEvent::HavenDiscovered);
        result.haven_changed = true;
        if !debug_mode {
            result.achievements_changed = true;
        }
    }
}

/// Stage 11: Roll for Soulforge discovery (P15+, no active content).
pub(super) fn tick_soulforge_discovery(
    state: &GameState,
    enhancement: &mut crate::enhancement::EnhancementProgress,
    achievements: &mut Achievements,
    debug_mode: bool,
    result: &mut TickResult,
    rng: &mut impl Rng,
) {
    if !enhancement.discovered
        && state.prestige_rank >= crate::enhancement::SOULFORGE_MIN_PRESTIGE_RANK
        && state.active_dungeon.is_none()
        && state.active_fishing.is_none()
        && state.active_minigame.is_none()
        && crate::enhancement::try_discover_soulforge(enhancement, state.prestige_rank, rng)
    {
        achievements.on_soulforge_discovered(Some(&state.character_name));
        result.events.push(TickEvent::SoulforgeDiscovered);
        result.enhancement_changed = true;
        if !debug_mode {
            result.achievements_changed = true;
        }
    }
}

/// Stage 11c: Tick active Deep missions, resolve completions, fire achievements.
pub(super) fn tick_deep_missions(
    state: &GameState,
    deep: &mut crate::deep::DeepState,
    achievements: &mut Achievements,
    debug_mode: bool,
    result: &mut TickResult,
    rng: &mut impl Rng,
) {
    if !deep.persistent.discovered {
        return;
    }

    let now = chrono::Utc::now();
    let pending_before = deep.prestige.pending_results.len();
    let summary = crate::deep::missions::tick_all_missions(
        &mut deep.prestige,
        &mut deep.persistent,
        now,
        rng,
    );

    if summary.missions_completed > 0 || summary.events_fired > 0 || summary.mercs_recovered > 0 {
        result.deep_changed = true;
    }

    // Fire achievement handlers for completed missions
    for _ in 0..summary.missions_completed {
        achievements.on_deep_mission_complete(Some(&state.character_name));
    }
    for layer in &summary.breakthroughs {
        achievements.on_deep_breakthrough(*layer, Some(&state.character_name));
        // Check if this breakthrough unlocks a fracture region
        if let Some(region) = crate::zones::FractureRegion::from_layer(*layer) {
            let new_cap = region.end_zone_id();
            if new_cap > deep.persistent.fracture_zone_cap {
                deep.persistent.fracture_zone_cap = new_cap;
                deep.persistent.pending_fracture_region_unlock = Some(region);
                result.deep_changed = true;
            }
        }
    }
    for _ in 0..summary.mercs_lost {
        achievements.on_deep_merc_lost(Some(&state.character_name));
    }
    if summary.gateway_opened {
        achievements.on_deep_gateway_opened(Some(&state.character_name));
    }

    // Loom of Worlds discovery: triggers when Gateway at Layer 30 completes.
    // Handled in tick_loom() which runs after this stage.

    if (summary.missions_completed > 0 || summary.mercs_lost > 0) && !debug_mode {
        result.achievements_changed = true;
    }

    // Emit tick events for newly completed missions
    for pending in deep.prestige.pending_results.iter().skip(pending_before) {
        if let Some(ref res) = pending.result {
            let outcome_str = match res.outcome {
                crate::deep::MissionOutcome::Success => "Success",
                crate::deep::MissionOutcome::PartialSuccess => "Partial Success",
                crate::deep::MissionOutcome::Failure => "Failure",
            };
            result.events.push(TickEvent::DeepMissionComplete {
                message: format!(
                    "\u{1F4DC} Mission complete: {} ({})",
                    pending.mission_type.display_name(),
                    outcome_str
                ),
            });
        }
    }

    // Check whether the mission pool needs a 6-hour refresh.
    if crate::deep::missions::maybe_refresh_mission_pool(
        &mut deep.prestige,
        &deep.persistent,
        now,
        rng,
    ) {
        result.deep_changed = true;
    }

    if crate::deep::missions::run_softlock_safeguards(
        &mut deep.prestige,
        &mut deep.persistent,
        now,
        rng,
    ) {
        result.deep_changed = true;
    }
}

/// Stage 11e: Tick the Loom of Worlds.
///
/// Discovery: fires when the Gateway Expedition at Deep Layer 30 completes.
/// After discovery: ticks staggered unlock, base production, neighbor unlocking,
/// and pattern sustain tracking every game tick.
pub(super) fn tick_loom(
    deep: &crate::deep::DeepState,
    loom: &mut crate::loom::LoomState,
    state: &mut crate::core::game_state::GameState,
    achievements: &mut crate::achievements::Achievements,
    result: &mut TickResult,
) {
    // Discovery trigger: requires Deep discovered + Gateway opened.
    if !loom.persistent.discovered {
        if deep.persistent.discovered && deep.persistent.gateway_opened {
            crate::loom::complete_discovery(loom);
            achievements.on_loom_discovered(Some(&state.character_name));
            result.events.push(TickEvent::LoomDiscovered);
            result.loom_changed = true;
        }
        return;
    }

    // Loom runs on wall-clock time — skip during Chrono Surge bursts.
    if state.chrono_surge_active {
        return;
    }

    // Loom uses wall-clock time. Compute real elapsed seconds since last tick.
    let now = chrono::Utc::now();
    let wall_delta = match loom.last_tick_at {
        Some(prev) => {
            let elapsed = (now - prev).num_milliseconds().max(0) as f64 / 1000.0;
            // Cap at 1 second to avoid huge jumps on resume/load.
            elapsed.min(1.0)
        }
        None => 0.1, // First tick after load — use nominal 100ms.
    };
    loom.last_tick_at = Some(now);

    // Apply debug time warp to wall delta (for testing only).
    let warp = if loom.time_warp > 1.0 {
        loom.time_warp
    } else {
        1.0
    };
    let tick_seconds: f64 = wall_delta * warp;

    // Tick shuttle construction (decrement timers, complete when done).
    let completed_shuttles = crate::loom::tick_shuttle_construction(loom, tick_seconds);
    if !completed_shuttles.is_empty() {
        result.loom_changed = true;
    }

    // Tick direct-pull shuttle processing.
    let shuttle_produced = crate::loom::tick_shuttle_pull(loom, tick_seconds);

    // Update stall flags for UI display.
    crate::loom::tick_stall_detection(loom);

    // Update shuttle stall flags.
    crate::loom::tick_shuttle_stall_detection(loom);

    // Tick extractor upgrade timers (before base production so completed upgrades apply immediately).
    crate::loom::tick_node_upgrades(loom, tick_seconds);

    // Tick base production for all unlocked nodes.
    let mut produced = crate::loom::tick_base_production(loom, tick_seconds);

    // Merge shuttle production into base production map for pattern sustain.
    for (resource, amount) in shuttle_produced {
        *produced.entry(resource).or_insert(0.0) += amount;
    }

    // Tick neighbor unlocking.
    let newly_unlocked = crate::loom::tick_neighbor_unlocking(loom, tick_seconds);
    if !newly_unlocked.is_empty() {
        result.loom_changed = true;
        loom.graph_dirty = true;
    }

    // Push per-tick production amounts into rate trackers.
    // Divide by time_warp so the rolling-window rate reflects the logical (un-warped)
    // production rate. The actual buffers already received the full warped amount.
    for (resource, &amount) in &produced {
        loom.rate_trackers
            .entry(*resource)
            .or_default()
            .push(amount / warp);
    }
    // Push 0.0 for resources not produced this tick (so their rate decays naturally).
    let all_resources = [
        crate::loom::Resource::Ember,
        crate::loom::Resource::Reflection,
        crate::loom::Resource::VoidEssence,
        crate::loom::Resource::Memory,
        crate::loom::Resource::Silence,
        crate::loom::Resource::Resonance,
        crate::loom::Resource::ForgedLight,
        crate::loom::Resource::EchoGlass,
        crate::loom::Resource::StillbornSong,
        crate::loom::Resource::CondensedEmber,
        crate::loom::Resource::EmberEcho,
        crate::loom::Resource::PurifiedVoid,
        crate::loom::Resource::WovenReality,
    ];
    for resource in &all_resources {
        if !produced.contains_key(resource) {
            loom.rate_trackers.entry(*resource).or_default().push(0.0);
        }
    }

    // Read measured rates from trackers for pattern sustain.
    let rates: std::collections::HashMap<crate::loom::Resource, f64> = loom
        .rate_trackers
        .iter()
        .map(|(resource, tracker)| (*resource, tracker.rate_per_hour()))
        .collect();

    let pattern_completed =
        crate::loom::tick_pattern_sustain(&mut loom.persistent, &rates, tick_seconds);
    if pattern_completed {
        result.loom_changed = true;
        loom.graph_dirty = true;
        let completed_count = loom.persistent.completed_pattern_count();
        achievements.on_loom_pattern_completed(completed_count, Some(&state.character_name));
        // Sync Loom zone unlocks on pattern completion
        let loom_cap = crate::loom::loom_zone_cap_for_patterns(completed_count);
        state.cached_loom_zone_cap = loom_cap;
        let fracture_cap = state.cached_fracture_zone_cap;
        let storms_end = state
            .zone_progression
            .is_zone_unlocked(crate::core::constants::EXPANSE_ZONE_ID);
        crate::zones::sync_account_zone_unlocks(
            &mut state.zone_progression,
            storms_end,
            fracture_cap,
            state.prestige_rank,
            loom_cap,
            state.ascension_level,
        );
        // Set pending milestone for tick.rs consumption (mirrors Deep's pending_fracture_region_unlock)
        if let Some(milestone) = crate::loom::PatternMilestone::from_count(completed_count) {
            loom.persistent.pending_pattern_milestones.push(milestone);
        }
    }

    // Tick WR→PR conversion (active after all 28 patterns complete).
    if crate::loom::all_patterns_complete(&loom.persistent) {
        let now = chrono::Utc::now().timestamp();
        if loom.persistent.wr_pr_last_granted_at == 0 || loom.persistent.wr_pr_last_granted_at > now
        {
            loom.persistent.wr_pr_last_granted_at = now;
            result.loom_changed = true;
        } else {
            let wr_rate = loom
                .rate_trackers
                .get(&crate::loom::Resource::WovenReality)
                .map(|t| t.rate_per_hour())
                .unwrap_or(0.0);
            let pr_per_hour = crate::loom::wr_to_pr_per_hour(wr_rate);
            if pr_per_hour > 0 {
                let fill_secs = (3600i64 / pr_per_hour as i64).max(1);
                let last = loom.persistent.wr_pr_last_granted_at;
                // Cap elapsed to 7 days to prevent exploits from bogus timestamps
                let elapsed = (now - last).min(604800);
                if elapsed >= fill_secs {
                    let completed_cycles = (elapsed / fill_secs) as u32;
                    state.prestige_rank = state.prestige_rank.saturating_add(completed_cycles);
                    state.recalculate_prestige_bonuses();
                    state.derived_stats_dirty = true;
                    loom.persistent.wr_pr_last_granted_at =
                        last + fill_secs * completed_cycles as i64;
                    // Coalesce into a single event with total PR
                    result.events.push(TickEvent::WovenRealityPRGranted {
                        pr_amount: completed_cycles,
                        wr_rate,
                    });
                    result.loom_changed = true;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::achievements::AchievementId;
    use crate::combat::Enemy;
    use crate::haven::Haven;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn apply_xp_and_check_levelup_emits_event_and_unlocks_level_milestone() {
        let mut rng = ChaCha8Rng::seed_from_u64(17);
        let mut state = GameState::new("Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        let mut result = TickResult::default();

        state.character_level = 9;
        state.character_xp = 0;

        apply_xp_and_check_levelup(
            &mut rng,
            &mut state,
            crate::core::xp::xp_for_next_level(9) as f64,
            &mut achievements,
            &mut result,
        );

        assert_eq!(state.character_level, 10);
        assert!(matches!(
            result.events.as_slice(),
            [TickEvent::LeveledUp { new_level: 10 }]
        ));
        assert!(achievements
            .take_newly_unlocked()
            .contains(&AchievementId::Level10));
    }

    #[test]
    fn apply_xp_and_check_levelup_no_level_emits_no_event() {
        let mut rng = ChaCha8Rng::seed_from_u64(18);
        let mut state = GameState::new("Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        let mut result = TickResult::default();

        apply_xp_and_check_levelup(
            &mut rng,
            &mut state,
            (crate::core::xp::xp_for_next_level(1) - 1) as f64,
            &mut achievements,
            &mut result,
        );

        assert_eq!(state.character_level, 1);
        assert!(result.events.is_empty());
        assert!(achievements.take_newly_unlocked().is_empty());
    }

    #[test]
    fn process_combat_events_maps_non_random_combat_events() {
        let mut rng = ChaCha8Rng::seed_from_u64(23);
        let mut state = GameState::new("Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        let mut deep = crate::deep::DeepState::new();
        let mut result = TickResult::default();
        let haven_bonuses = Haven::new().compute_bonuses();

        state.combat_state.current_enemy = Some(Enemy::new("Shade".to_string(), 40, 8));

        process_combat_events(
            &mut state,
            vec![
                CombatEvent::PlayerAttackBlocked {
                    weapon_needed: "Relic Blade".to_string(),
                },
                CombatEvent::PlayerAttack {
                    damage: 12,
                    was_crit: false,
                },
                CombatEvent::EnemyAttack { damage: 7 },
                CombatEvent::DamageReflected { damage: 3 },
                CombatEvent::RegenComplete { healed: 9 },
                CombatEvent::PlayerDiedInDungeon,
                CombatEvent::PlayerDied,
                CombatEvent::CombatRetreat {
                    zone_name: "Meadow".to_string(),
                },
                CombatEvent::DungeonRetreat,
            ],
            &haven_bonuses,
            &mut achievements,
            &mut deep,
            false,
            &mut result,
            &mut rng,
        );

        assert_eq!(result.events.len(), 9);
        assert!(matches!(
            result.events[0],
            TickEvent::PlayerAttackBlocked { .. }
        ));
        assert!(matches!(
            result.events[1],
            TickEvent::PlayerAttack {
                damage: 12,
                was_crit: false,
                ..
            }
        ));
        assert!(matches!(
            result.events[2],
            TickEvent::EnemyAttack {
                damage: 7,
                ref enemy_name,
                ..
            } if enemy_name == "Shade"
        ));
        assert!(matches!(
            result.events[3],
            TickEvent::DamageReflected { damage: 3, .. }
        ));
        assert!(matches!(
            result.events[4],
            TickEvent::RegenComplete { healed: 9 }
        ));
        assert!(matches!(result.events[5], TickEvent::PlayerDiedInDungeon));
        assert!(matches!(result.events[6], TickEvent::PlayerDied));
        assert!(matches!(
            result.events[7],
            TickEvent::CombatRetreat {
                ref zone_name,
                ..
            } if zone_name == "Meadow"
        ));
        assert!(matches!(result.events[8], TickEvent::DungeonFailed));
    }

    #[test]
    fn collect_achievement_events_drains_queue_and_sets_dirty_flag() {
        let mut achievements = Achievements::default();
        let mut result = TickResult::default();
        let expected_name = crate::achievements::get_achievement_def(AchievementId::Level10)
            .unwrap()
            .name;

        achievements.unlock(AchievementId::Level10, Some("Hero".to_string()));

        collect_achievement_events(&mut achievements, &mut result);

        assert!(result.achievements_changed);
        assert!(matches!(
            result.events.as_slice(),
            [TickEvent::AchievementUnlocked { name }]
                if name == expected_name
        ));
        assert!(achievements.take_newly_unlocked().is_empty());
    }

    // ── process_dungeon_events ──────────────────────────────────────────────

    #[test]
    fn process_dungeon_events_noop_without_active_dungeon() {
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        let mut state = GameState::new("Hero".to_string(), 0);
        let mut result = TickResult::default();
        let haven_bonuses = Haven::new().compute_bonuses();

        process_dungeon_events(&mut state, 0.1, &haven_bonuses, &mut result, &mut rng);

        assert!(result.events.is_empty());
    }

    #[test]
    fn process_dungeon_events_treasure_room_entry_drops_item_and_salvages_stormglass() {
        use crate::dungeon::types::{Dungeon, DungeonSize, Room, RoomState, RoomType, DIR_RIGHT};
        use crate::items::generation::generate_item_with_rng;
        use crate::items::types::EquipmentSlot;

        let mut rng = ChaCha8Rng::seed_from_u64(99);
        let mut state = GameState::new("Hero".to_string(), 0);
        state.prestige_rank = 20; // >= STORMGLASS_MIN_PRESTIGE_RANK
        state.zone_progression.current_zone_id = 5;

        // Pre-equip Mythic gear in every slot so the treasure item can never be
        // auto-equipped, forcing the Stormglass salvage branch deterministically
        // (auto_equip_if_better always refuses to replace a Mythic item).
        for slot in [
            EquipmentSlot::Weapon,
            EquipmentSlot::Armor,
            EquipmentSlot::Helmet,
            EquipmentSlot::Gloves,
            EquipmentSlot::Boots,
            EquipmentSlot::Amulet,
            EquipmentSlot::Ring,
        ] {
            let guard = generate_item_with_rng(slot, Rarity::Mythic, 999, &mut rng);
            state.equipment.set(slot, Some(guard));
        }

        let mut dungeon = Dungeon::new(DungeonSize::Small);
        dungeon.entrance_position = (0, 0);
        dungeon.boss_position = (4, 4);
        dungeon.player_position = (0, 0);
        dungeon.current_room_cleared = true;
        dungeon.move_timer = 10.0; // already well past ROOM_MOVE_INTERVAL

        let mut entrance = Room::new(RoomType::Entrance, (0, 0));
        entrance.state = RoomState::Current;
        entrance.connections[DIR_RIGHT] = true;
        dungeon.grid[0][0] = Some(entrance);

        let mut treasure = Room::new(RoomType::Treasure, (1, 0));
        treasure.state = RoomState::Revealed;
        dungeon.grid[0][1] = Some(treasure);

        state.active_dungeon = Some(dungeon);

        let haven_bonuses = Haven::new().compute_bonuses();
        let mut result = TickResult::default();

        process_dungeon_events(&mut state, 0.1, &haven_bonuses, &mut result, &mut rng);

        assert!(result.events.iter().any(|e| matches!(
            e,
            TickEvent::DungeonRoomEntered {
                room_type: RoomType::Treasure,
                ..
            }
        )));
        assert!(result.events.iter().any(|e| matches!(
            e,
            TickEvent::DungeonTreasureFound {
                equipped: false,
                ..
            }
        )));
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, TickEvent::StormglassDiscovered)));
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, TickEvent::StormglassSalvaged { .. })));
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, TickEvent::StormglassDungeonCache { .. })));
        assert!(state.stormglass_discovered);
        assert!(state.stormglass > 0);
    }

    // ── process_fishing_tick ─────────────────────────────────────────────────

    #[test]
    fn process_fishing_tick_returns_false_without_active_fishing() {
        let mut rng = ChaCha8Rng::seed_from_u64(2);
        let mut state = GameState::new("Hero".to_string(), 0);
        let mut tick_counter = 0u32;
        let mut achievements = Achievements::default();
        let mut result = TickResult::default();
        let haven_bonuses = Haven::new().compute_bonuses();

        let handled = process_fishing_tick(
            &mut state,
            &mut tick_counter,
            0.1,
            &haven_bonuses,
            &mut achievements,
            false,
            &mut result,
            &mut rng,
        );

        assert!(!handled);
        assert_eq!(tick_counter, 0);
    }

    #[test]
    fn process_fishing_tick_casting_phase_emits_message_and_advances_to_waiting() {
        let mut rng = ChaCha8Rng::seed_from_u64(3);
        let mut state = GameState::new("Hero".to_string(), 0);
        state.active_fishing = Some(crate::fishing::FishingSession {
            spot_name: "Test Cove".to_string(),
            total_fish: 5,
            fish_caught: Vec::new(),
            items_found: Vec::new(),
            ticks_remaining: 1,
            phase: crate::fishing::FishingPhase::Casting,
        });
        let mut tick_counter = 0u32;
        let mut achievements = Achievements::default();
        let mut result = TickResult::default();
        let haven_bonuses = Haven::new().compute_bonuses();

        let handled = process_fishing_tick(
            &mut state,
            &mut tick_counter,
            0.1,
            &haven_bonuses,
            &mut achievements,
            false,
            &mut result,
            &mut rng,
        );

        assert!(handled);
        assert_eq!(tick_counter, 1);
        assert!(matches!(
            state.active_fishing.as_ref().unwrap().phase,
            crate::fishing::FishingPhase::Waiting
        ));
        assert!(result.events.iter().any(
            |e| matches!(e, TickEvent::FishingMessage { message } if message.contains("waiting for a bite"))
        ));
    }

    #[test]
    fn process_fishing_tick_waiting_phase_emits_message_and_advances_to_reeling() {
        let mut rng = ChaCha8Rng::seed_from_u64(4);
        let mut state = GameState::new("Hero".to_string(), 0);
        state.active_fishing = Some(crate::fishing::FishingSession {
            spot_name: "Test Cove".to_string(),
            total_fish: 5,
            fish_caught: Vec::new(),
            items_found: Vec::new(),
            ticks_remaining: 1,
            phase: crate::fishing::FishingPhase::Waiting,
        });
        let mut tick_counter = 0u32;
        let mut achievements = Achievements::default();
        let mut result = TickResult::default();
        let haven_bonuses = Haven::new().compute_bonuses();

        process_fishing_tick(
            &mut state,
            &mut tick_counter,
            0.1,
            &haven_bonuses,
            &mut achievements,
            false,
            &mut result,
            &mut rng,
        );

        assert!(matches!(
            state.active_fishing.as_ref().unwrap().phase,
            crate::fishing::FishingPhase::Reeling
        ));
        assert!(result.events.iter().any(
            |e| matches!(e, TickEvent::FishingMessage { message } if message.contains("Got a bite"))
        ));
    }

    #[test]
    fn process_fishing_tick_reeling_catches_fish_and_updates_play_time() {
        let mut rng = ChaCha8Rng::seed_from_u64(5);
        let mut state = GameState::new("Hero".to_string(), 0);
        state.active_fishing = Some(crate::fishing::FishingSession {
            spot_name: "Test Cove".to_string(),
            total_fish: 100,
            fish_caught: Vec::new(),
            items_found: Vec::new(),
            ticks_remaining: 1,
            phase: crate::fishing::FishingPhase::Reeling,
        });
        let mut tick_counter = TICKS_PER_SECOND - 1;
        let mut achievements = Achievements::default();
        let mut result = TickResult::default();
        let haven_bonuses = Haven::new().compute_bonuses();
        let play_time_before = state.play_time_seconds;

        let handled = process_fishing_tick(
            &mut state,
            &mut tick_counter,
            0.1,
            &haven_bonuses,
            &mut achievements,
            false,
            &mut result,
            &mut rng,
        );

        assert!(handled);
        assert_eq!(tick_counter, 0);
        assert_eq!(state.play_time_seconds, play_time_before + 1);
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, TickEvent::FishCaught { .. })));
    }

    #[test]
    fn process_fishing_tick_rank_up_fires_achievement_and_event() {
        let mut rng = ChaCha8Rng::seed_from_u64(11);
        let mut state = GameState::new("Hero".to_string(), 0);
        state.fishing.rank = 1;
        state.fishing.fish_toward_next_rank =
            crate::fishing::types::FishingState::fish_required_for_rank(1) - 1;
        state.active_fishing = Some(crate::fishing::FishingSession {
            spot_name: "Rank Cove".to_string(),
            total_fish: 10,
            fish_caught: Vec::new(),
            items_found: Vec::new(),
            ticks_remaining: 1,
            phase: crate::fishing::FishingPhase::Reeling,
        });
        let mut tick_counter = 0u32;
        let mut achievements = Achievements::default();
        let mut result = TickResult::default();
        let haven_bonuses = Haven::new().compute_bonuses();

        process_fishing_tick(
            &mut state,
            &mut tick_counter,
            0.1,
            &haven_bonuses,
            &mut achievements,
            false,
            &mut result,
            &mut rng,
        );

        assert_eq!(state.fishing.rank, 2);
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, TickEvent::FishingRankUp { .. })));
    }

    #[test]
    fn process_fishing_tick_storm_leviathan_caught_sets_flag_and_achievement() {
        let mut rng = ChaCha8Rng::seed_from_u64(2024);
        let mut state = GameState::new("Hero".to_string(), 0);
        state.fishing.rank = 40;
        state.fishing.leviathan_encounters = 10;
        let mut tick_counter = 0u32;
        let mut achievements = Achievements::default();
        let haven_bonuses = Haven::new().compute_bonuses();

        let mut caught = false;
        for _ in 0..5000 {
            state.active_fishing = Some(crate::fishing::FishingSession {
                spot_name: "Storm Cove".to_string(),
                total_fish: 1_000_000,
                fish_caught: Vec::new(),
                items_found: Vec::new(),
                ticks_remaining: 1,
                phase: crate::fishing::FishingPhase::Reeling,
            });
            let mut result = TickResult::default();
            process_fishing_tick(
                &mut state,
                &mut tick_counter,
                0.1,
                &haven_bonuses,
                &mut achievements,
                false,
                &mut result,
                &mut rng,
            );
            if result
                .events
                .iter()
                .any(|e| matches!(e, TickEvent::StormLeviathanCaught))
            {
                caught = true;
                break;
            }
        }

        assert!(
            caught,
            "Storm Leviathan should be caught within 5000 attempts"
        );
        assert!(state.fishing.leviathan_caught);
        assert!(achievements
            .take_newly_unlocked()
            .contains(&AchievementId::StormLeviathan));
    }

    #[test]
    fn process_fishing_tick_can_find_item_while_catching() {
        let mut rng = ChaCha8Rng::seed_from_u64(55);
        let mut state = GameState::new("Hero".to_string(), 0);
        let mut tick_counter = 0u32;
        let mut achievements = Achievements::default();
        let haven_bonuses = Haven::new().compute_bonuses();

        let mut found_item = false;
        for _ in 0..3000 {
            state.active_fishing = Some(crate::fishing::FishingSession {
                spot_name: "Item Cove".to_string(),
                total_fish: 1_000_000,
                fish_caught: Vec::new(),
                items_found: Vec::new(),
                ticks_remaining: 1,
                phase: crate::fishing::FishingPhase::Reeling,
            });
            let mut result = TickResult::default();
            process_fishing_tick(
                &mut state,
                &mut tick_counter,
                0.1,
                &haven_bonuses,
                &mut achievements,
                false,
                &mut result,
                &mut rng,
            );
            if result
                .events
                .iter()
                .any(|e| matches!(e, TickEvent::FishingItemFound { .. }))
            {
                found_item = true;
                break;
            }
        }

        assert!(
            found_item,
            "Should find a fishing item within 3000 attempts"
        );
    }

    // ── process_combat_events ────────────────────────────────────────────────

    #[test]
    fn process_combat_events_enemy_died_awards_xp_and_processes_drops() {
        let mut rng = ChaCha8Rng::seed_from_u64(31);
        let mut state = GameState::new("Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        let mut deep = crate::deep::DeepState::new();
        let mut result = TickResult::default();
        let haven_bonuses = Haven::new().compute_bonuses();

        state.combat_state.current_enemy = Some(Enemy::new("Rat".to_string(), 10, 2));
        let kills_before = state.session_kills;

        process_combat_events(
            &mut state,
            vec![CombatEvent::EnemyDied { xp_gained: 250 }],
            &haven_bonuses,
            &mut achievements,
            &mut deep,
            false,
            &mut result,
            &mut rng,
        );

        assert_eq!(state.session_kills, kills_before + 1);
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, TickEvent::EnemyDefeated { xp_gained: 250, .. })));
    }

    #[test]
    fn process_combat_events_enemy_died_in_dungeon_tracks_xp_and_clears_room() {
        let mut rng = ChaCha8Rng::seed_from_u64(32);
        let mut state = GameState::new("Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        let mut deep = crate::deep::DeepState::new();
        let mut result = TickResult::default();
        let haven_bonuses = Haven::new().compute_bonuses();

        state.combat_state.current_enemy = Some(Enemy::new("Rat".to_string(), 10, 2));
        let mut dungeon = crate::dungeon::generation::generate_dungeon(10, 0, 1);
        dungeon.xp_earned = 0;
        dungeon.current_room_cleared = false;
        state.active_dungeon = Some(dungeon);

        process_combat_events(
            &mut state,
            vec![CombatEvent::EnemyDied { xp_gained: 300 }],
            &haven_bonuses,
            &mut achievements,
            &mut deep,
            false,
            &mut result,
            &mut rng,
        );

        let dungeon = state.active_dungeon.as_ref().unwrap();
        assert_eq!(dungeon.xp_earned, 300);
        assert!(dungeon.current_room_cleared);
    }

    #[test]
    fn process_combat_events_elite_defeated_grants_key_in_dungeon() {
        let mut rng = ChaCha8Rng::seed_from_u64(33);
        let mut state = GameState::new("Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        let mut deep = crate::deep::DeepState::new();
        let mut result = TickResult::default();
        let haven_bonuses = Haven::new().compute_bonuses();

        state.combat_state.current_enemy = Some(Enemy::new("Guardian".to_string(), 100, 20));
        state.active_dungeon = Some(crate::dungeon::generation::generate_dungeon(10, 0, 1));

        process_combat_events(
            &mut state,
            vec![CombatEvent::EliteDefeated { xp_gained: 400 }],
            &haven_bonuses,
            &mut achievements,
            &mut deep,
            false,
            &mut result,
            &mut rng,
        );

        assert!(state.active_dungeon.as_ref().unwrap().has_key);
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, TickEvent::DungeonKeyFound)));
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, TickEvent::DungeonEliteDefeated { xp_gained: 400, .. })));
    }

    #[test]
    fn process_combat_events_boss_defeated_in_dungeon_awards_bonus_xp_and_clears_dungeon() {
        let mut rng = ChaCha8Rng::seed_from_u64(41);
        let mut state = GameState::new("Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        let mut deep = crate::deep::DeepState::new();
        let mut result = TickResult::default();
        let haven_bonuses = Haven::new().compute_bonuses();

        state.combat_state.current_enemy = Some(Enemy::new("Dungeon Boss".to_string(), 500, 50));
        let mut dungeon = crate::dungeon::generation::generate_dungeon(10, 0, 1);
        dungeon.xp_earned = 1000;
        state.active_dungeon = Some(dungeon);

        process_combat_events(
            &mut state,
            vec![CombatEvent::BossDefeated { xp_gained: 500 }],
            &haven_bonuses,
            &mut achievements,
            &mut deep,
            false,
            &mut result,
            &mut rng,
        );

        assert!(state.active_dungeon.is_none());
        assert!(result.events.iter().any(|e| matches!(
            e,
            TickEvent::DungeonBossDefeated {
                xp_gained: 500,
                total_xp,
                ..
            } if *total_xp > 2000
        )));
    }

    #[test]
    fn process_combat_events_boss_defeated_outside_dungeon_has_no_bonus_xp() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let mut state = GameState::new("Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        let mut deep = crate::deep::DeepState::new();
        let mut result = TickResult::default();
        let haven_bonuses = Haven::new().compute_bonuses();

        state.combat_state.current_enemy = Some(Enemy::new("Field Boss".to_string(), 500, 50));

        process_combat_events(
            &mut state,
            vec![CombatEvent::BossDefeated { xp_gained: 777 }],
            &haven_bonuses,
            &mut achievements,
            &mut deep,
            false,
            &mut result,
            &mut rng,
        );

        assert!(result.events.iter().any(|e| matches!(
            e,
            TickEvent::DungeonBossDefeated {
                xp_gained: 777,
                bonus_xp: 0,
                total_xp: 777,
                items_collected: 0,
                ..
            }
        )));
    }

    #[test]
    fn process_combat_events_subzone_boss_weapon_required_skips_event_but_still_awards_xp() {
        let mut rng = ChaCha8Rng::seed_from_u64(51);
        let mut state = GameState::new("Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        let mut deep = crate::deep::DeepState::new();
        let mut result = TickResult::default();
        let haven_bonuses = Haven::new().compute_bonuses();

        process_combat_events(
            &mut state,
            vec![CombatEvent::SubzoneBossDefeated {
                xp_gained: 300,
                result: BossDefeatResult::WeaponRequired {
                    weapon_name: "Stormbreaker",
                },
            }],
            &haven_bonuses,
            &mut achievements,
            &mut deep,
            false,
            &mut result,
            &mut rng,
        );

        assert!(!result
            .events
            .iter()
            .any(|e| matches!(e, TickEvent::SubzoneBossDefeated { .. })));
        assert_eq!(state.session_kills, 1);
    }

    #[test]
    fn process_combat_events_expanse_cycle_triggers_deep_discovery() {
        let mut rng = ChaCha8Rng::seed_from_u64(61);
        let mut state = GameState::new("Hero".to_string(), 0);
        state.prestige_rank = 20; // >= DEEP_MIN_PRESTIGE_RANK
        let mut achievements = Achievements::default();
        let mut deep = crate::deep::DeepState::new();
        let mut result = TickResult::default();
        let haven_bonuses = Haven::new().compute_bonuses();

        assert!(!deep.persistent.discovered);

        process_combat_events(
            &mut state,
            vec![CombatEvent::SubzoneBossDefeated {
                xp_gained: 300,
                result: BossDefeatResult::ExpanseCycle,
            }],
            &haven_bonuses,
            &mut achievements,
            &mut deep,
            false,
            &mut result,
            &mut rng,
        );

        assert!(deep.persistent.discovered);
        assert!(result.deep_changed);
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, TickEvent::DeepDiscovered)));
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, TickEvent::SubzoneBossDefeated { .. })));
        assert!(achievements
            .take_newly_unlocked()
            .contains(&AchievementId::TheDeepDiscovered));
    }

    #[test]
    fn process_combat_events_z50_loom_zone_cycle_triggers_vessel_signal_discovery() {
        let mut rng = ChaCha8Rng::seed_from_u64(71);
        let mut state = GameState::new("Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        let mut deep = crate::deep::DeepState::new();
        let mut result = TickResult::default();
        let haven_bonuses = Haven::new().compute_bonuses();

        assert!(!state.vessel_signal_discovered);

        process_combat_events(
            &mut state,
            vec![CombatEvent::SubzoneBossDefeated {
                xp_gained: 300,
                result: BossDefeatResult::LoomZoneCycle { zone_id: 50 },
            }],
            &haven_bonuses,
            &mut achievements,
            &mut deep,
            false,
            &mut result,
            &mut rng,
        );

        assert!(state.vessel_signal_discovered);
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, TickEvent::VesselSignalDiscovered)));
    }

    // ── process_item_drop ────────────────────────────────────────────────────

    #[test]
    fn process_item_drop_boss_drop_salvages_when_not_equipped() {
        use crate::items::generation::generate_item_with_rng;
        use crate::items::types::EquipmentSlot;

        let mut rng = ChaCha8Rng::seed_from_u64(81);
        let mut state = GameState::new("Hero".to_string(), 0);
        state.prestige_rank = 20;
        state.zone_progression.current_zone_id = 3;
        state.zone_progression.fighting_boss = true;

        for slot in [
            EquipmentSlot::Weapon,
            EquipmentSlot::Armor,
            EquipmentSlot::Helmet,
            EquipmentSlot::Gloves,
            EquipmentSlot::Boots,
            EquipmentSlot::Amulet,
            EquipmentSlot::Ring,
        ] {
            let guard = generate_item_with_rng(slot, Rarity::Mythic, 999, &mut rng);
            state.equipment.set(slot, Some(guard));
        }

        let haven_bonuses = Haven::new().compute_bonuses();
        let mut result = TickResult::default();

        process_item_drop(&mut state, &haven_bonuses, &mut result);

        assert!(result.events.iter().any(|e| matches!(
            e,
            TickEvent::ItemDropped {
                equipped: false,
                from_boss: true,
                ..
            }
        )));
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, TickEvent::StormglassDiscovered)));
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, TickEvent::StormglassSalvaged { .. })));
        assert!(state.stormglass_discovered);
        assert!(state.stormglass > 0);
    }

    #[test]
    fn process_item_drop_boss_drop_equipped_skips_salvage_even_when_stormglass_active() {
        let mut state = GameState::new("Hero".to_string(), 0);
        state.prestige_rank = 20;
        state.zone_progression.current_zone_id = 10;
        state.zone_progression.fighting_boss = true;

        let haven_bonuses = Haven::new().compute_bonuses();
        let mut result = TickResult::default();

        process_item_drop(&mut state, &haven_bonuses, &mut result);

        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, TickEvent::ItemDropped { equipped: true, .. })));
        assert!(!result
            .events
            .iter()
            .any(|e| matches!(e, TickEvent::StormglassSalvaged { .. })));
    }

    #[test]
    fn process_item_drop_boss_drop_without_stormglass_access_never_salvages() {
        use crate::items::generation::generate_item_with_rng;
        use crate::items::types::EquipmentSlot;

        let mut rng = ChaCha8Rng::seed_from_u64(82);
        let mut state = GameState::new("Hero".to_string(), 0);
        state.zone_progression.current_zone_id = 10;
        state.zone_progression.fighting_boss = true;
        // prestige_rank stays 0 (< STORMGLASS_MIN_PRESTIGE_RANK) and stormglass undiscovered.

        for slot in [
            EquipmentSlot::Weapon,
            EquipmentSlot::Armor,
            EquipmentSlot::Helmet,
            EquipmentSlot::Gloves,
            EquipmentSlot::Boots,
            EquipmentSlot::Amulet,
            EquipmentSlot::Ring,
        ] {
            let guard = generate_item_with_rng(slot, Rarity::Mythic, 999, &mut rng);
            state.equipment.set(slot, Some(guard));
        }

        let haven_bonuses = Haven::new().compute_bonuses();
        let mut result = TickResult::default();

        process_item_drop(&mut state, &haven_bonuses, &mut result);

        assert!(result.events.iter().any(|e| matches!(
            e,
            TickEvent::ItemDropped {
                equipped: false,
                ..
            }
        )));
        assert!(!result
            .events
            .iter()
            .any(|e| matches!(e, TickEvent::StormglassSalvaged { .. })));
        assert!(!state.stormglass_discovered);
    }

    #[test]
    fn process_item_drop_mob_drop_eventually_fires_item_dropped_event() {
        let mut state = GameState::new("Hero".to_string(), 0);
        state.prestige_rank = 50; // maxes the item drop chance
        state.zone_progression.current_zone_id = 3;
        state.zone_progression.fighting_boss = false;

        let haven_bonuses = Haven::new().compute_bonuses();

        let mut dropped = false;
        for _ in 0..500 {
            let mut result = TickResult::default();
            process_item_drop(&mut state, &haven_bonuses, &mut result);
            if result.events.iter().any(|e| {
                matches!(
                    e,
                    TickEvent::ItemDropped {
                        from_boss: false,
                        ..
                    }
                )
            }) {
                dropped = true;
                break;
            }
        }

        assert!(
            dropped,
            "Mob should eventually drop an item at max drop chance"
        );
    }

    // ── process_discoveries ──────────────────────────────────────────────────

    #[test]
    fn process_discoveries_dungeon_discovery_populates_active_dungeon() {
        let mut rng = ChaCha8Rng::seed_from_u64(101);
        let mut state = GameState::new("Hero".to_string(), 0);
        let mut discovered = false;

        for _ in 0..5000 {
            state.active_dungeon = None;
            let mut result = TickResult::default();
            process_discoveries(&mut state, &mut rng, &mut result);
            if result
                .events
                .iter()
                .any(|e| matches!(e, TickEvent::DungeonDiscovered { .. }))
            {
                discovered = true;
                break;
            }
        }

        assert!(
            discovered,
            "Dungeon should be discovered within 5000 attempts"
        );
        assert!(state.active_dungeon.is_some());
    }

    #[test]
    fn process_discoveries_fishing_discovery_populates_active_fishing() {
        let mut rng = ChaCha8Rng::seed_from_u64(111);
        let mut state = GameState::new("Hero".to_string(), 0);
        let mut discovered = false;

        for _ in 0..2000 {
            state.active_dungeon = None;
            state.active_fishing = None;
            let mut result = TickResult::default();
            process_discoveries(&mut state, &mut rng, &mut result);
            if result
                .events
                .iter()
                .any(|e| matches!(e, TickEvent::FishingSpotDiscovered { .. }))
            {
                discovered = true;
                break;
            }
        }

        assert!(
            discovered,
            "Fishing spot should be discovered within 2000 attempts"
        );
    }

    // ── process_zone_achievements ────────────────────────────────────────────

    #[test]
    fn process_zone_achievements_handles_all_boss_defeat_variants() {
        let mut achievements = Achievements::default();

        process_zone_achievements(
            &BossDefeatResult::ZoneComplete {
                old_zone: "Meadow",
                old_zone_id: 2,
                new_zone_id: 3,
            },
            &mut achievements,
            "Hero",
        );
        assert!(achievements
            .take_newly_unlocked()
            .contains(&AchievementId::Zone2Complete));

        process_zone_achievements(
            &BossDefeatResult::ZoneCompleteButGated {
                zone_name: "Wilds",
                old_zone_id: 4,
                required_prestige: 2,
            },
            &mut achievements,
            "Hero",
        );
        assert!(achievements
            .take_newly_unlocked()
            .contains(&AchievementId::Zone4Complete));

        process_zone_achievements(&BossDefeatResult::StormsEnd, &mut achievements, "Hero");
        assert!(achievements
            .take_newly_unlocked()
            .contains(&AchievementId::Zone10Complete));

        process_zone_achievements(&BossDefeatResult::ExpanseCycle, &mut achievements, "Hero");
        assert!(achievements
            .take_newly_unlocked()
            .contains(&AchievementId::BeyondInfinity));

        process_zone_achievements(
            &BossDefeatResult::FractureCycle { zone_id: 13 },
            &mut achievements,
            "Hero",
        );
        assert!(achievements
            .take_newly_unlocked()
            .contains(&AchievementId::FractureZone13));

        process_zone_achievements(
            &BossDefeatResult::LoomZoneCycle { zone_id: 35 },
            &mut achievements,
            "Hero",
        );
        // Zone 35 has no individual achievement mapping (table tops out at 30) —
        // should not panic, and should not unlock anything.
        assert!(achievements.take_newly_unlocked().is_empty());

        process_zone_achievements(
            &BossDefeatResult::FrontierBackoff {
                zone_id: 20,
                blocked_zone_id: 21,
            },
            &mut achievements,
            "Hero",
        );
        assert!(achievements
            .take_newly_unlocked()
            .contains(&AchievementId::FractureZone20));

        process_zone_achievements(
            &BossDefeatResult::SubzoneComplete { new_subzone_id: 2 },
            &mut achievements,
            "Hero",
        );
        assert!(achievements.take_newly_unlocked().is_empty());

        process_zone_achievements(
            &BossDefeatResult::WeaponRequired {
                weapon_name: "Stormbreaker",
            },
            &mut achievements,
            "Hero",
        );
        assert!(achievements.take_newly_unlocked().is_empty());
    }

    // ── track_passive_prestige_gain ──────────────────────────────────────────

    #[test]
    fn track_passive_prestige_gain_fires_achievement_when_rank_increases() {
        let mut state = GameState::new("Hero".to_string(), 0);
        state.prestige_rank = 5;
        let mut achievements = Achievements::default();

        track_passive_prestige_gain(&state, &mut achievements, 5);
        assert!(achievements.take_newly_unlocked().is_empty());

        track_passive_prestige_gain(&state, &mut achievements, 3);
        assert!(!achievements.take_newly_unlocked().is_empty());
    }

    // ── tick_vessel_whispers ─────────────────────────────────────────────────

    #[test]
    fn tick_vessel_whispers_noop_when_signal_not_discovered() {
        let mut state = GameState::new("Hero".to_string(), 0);
        let mut result = TickResult::default();

        tick_vessel_whispers(&mut state, &mut result);

        assert!(result.events.is_empty());
    }

    #[test]
    fn tick_vessel_whispers_noop_when_vessel_already_launched() {
        let mut state = GameState::new("Hero".to_string(), 0);
        state.vessel_signal_discovered = true;
        state.vessel_launched = true;
        state.play_time_seconds = 10_000;
        let mut result = TickResult::default();

        tick_vessel_whispers(&mut state, &mut result);

        assert!(result.events.is_empty());
    }

    #[test]
    fn tick_vessel_whispers_emits_after_interval_elapses() {
        let mut state = GameState::new("Hero".to_string(), 0);
        state.vessel_signal_discovered = true;
        state.vessel_last_whisper_at = 0;
        state.play_time_seconds = crate::vessel::WHISPER_INTERVAL_SECONDS;
        let mut result = TickResult::default();

        tick_vessel_whispers(&mut state, &mut result);

        assert_eq!(state.vessel_last_whisper_at, state.play_time_seconds);
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, TickEvent::VesselWhisper { .. })));
    }

    #[test]
    fn tick_vessel_whispers_does_not_fire_before_interval() {
        let mut state = GameState::new("Hero".to_string(), 0);
        state.vessel_signal_discovered = true;
        state.vessel_last_whisper_at = 0;
        state.play_time_seconds = crate::vessel::WHISPER_INTERVAL_SECONDS - 1;
        let mut result = TickResult::default();

        tick_vessel_whispers(&mut state, &mut result);

        assert!(result.events.is_empty());
    }

    // ── tick_challenge_ai ────────────────────────────────────────────────────

    #[test]
    fn tick_challenge_ai_noop_without_active_minigame() {
        let mut rng = ChaCha8Rng::seed_from_u64(121);
        let mut state = GameState::new("Hero".to_string(), 0);

        tick_challenge_ai(&mut state, &mut rng);

        assert!(state.active_minigame.is_none());
    }

    #[test]
    fn tick_challenge_ai_dispatches_to_each_minigame_variant() {
        use crate::challenges::{
            ChessDifficulty, ChessGame, GoDifficulty, GoGame, GomokuDifficulty, GomokuGame,
            MorrisDifficulty, MorrisGame, ShardFusionDifficulty, ShardFusionGame,
        };

        let mut rng = ChaCha8Rng::seed_from_u64(122);
        let mut state = GameState::new("Hero".to_string(), 0);

        state.active_minigame = Some(ActiveMinigame::Chess(Box::new(ChessGame::new(
            ChessDifficulty::Novice,
        ))));
        tick_challenge_ai(&mut state, &mut rng);

        state.active_minigame = Some(ActiveMinigame::Morris(MorrisGame::new(
            MorrisDifficulty::Novice,
        )));
        tick_challenge_ai(&mut state, &mut rng);

        state.active_minigame = Some(ActiveMinigame::Gomoku(GomokuGame::new(
            GomokuDifficulty::Novice,
        )));
        tick_challenge_ai(&mut state, &mut rng);

        state.active_minigame = Some(ActiveMinigame::Go(GoGame::new(GoDifficulty::Novice)));
        tick_challenge_ai(&mut state, &mut rng);

        state.active_minigame = Some(ActiveMinigame::ShardFusion(ShardFusionGame::new(
            ShardFusionDifficulty::Novice,
        )));
        tick_challenge_ai(&mut state, &mut rng);

        // Reaching here without panicking exercises every dispatch arm.
    }

    // ── tick_challenge_discovery ─────────────────────────────────────────────

    #[test]
    fn tick_challenge_discovery_skips_during_chrono_surge() {
        let mut rng = ChaCha8Rng::seed_from_u64(131);
        let mut state = GameState::new("Hero".to_string(), 0);
        state.chrono_surge_active = true;
        state.prestige_rank = 5;
        let haven_bonuses = Haven::new().compute_bonuses();
        let mut result = TickResult::default();

        tick_challenge_discovery(&mut state, &haven_bonuses, &mut rng, &mut result);

        assert!(result.events.is_empty());
    }

    #[test]
    fn tick_challenge_discovery_eventually_discovers_a_challenge() {
        let mut rng = ChaCha8Rng::seed_from_u64(132);
        let mut state = GameState::new("Hero".to_string(), 0);
        state.prestige_rank = 5;
        let haven_bonuses = Haven::new().compute_bonuses();

        let mut discovered = false;
        for _ in 0..3_000_000 {
            let mut result = TickResult::default();
            tick_challenge_discovery(&mut state, &haven_bonuses, &mut rng, &mut result);
            if result
                .events
                .iter()
                .any(|e| matches!(e, TickEvent::ChallengeDiscovered { .. }))
            {
                discovered = true;
                break;
            }
        }

        assert!(
            discovered,
            "A challenge should be discovered within 3,000,000 ticks"
        );
    }

    // ── sync_derived_stats ───────────────────────────────────────────────────

    #[test]
    fn sync_derived_stats_applies_prestige_ascension_and_sigil_hp_bonuses() {
        let mut state = GameState::new("Hero".to_string(), 0);
        state.derived_stats_dirty = true;
        state.prestige_rank = 20;
        state.ascension_level = 3;
        let enhancement = crate::enhancement::EnhancementProgress::new();
        let sigil_bonuses = SigilBonuses {
            max_hp_percent: 50.0,
            ..Default::default()
        };

        sync_derived_stats(&mut state, &enhancement, &sigil_bonuses);

        assert!(!state.derived_stats_dirty);
        assert!(state.combat_state.player_max_hp > state.cached_derived_stats.max_hp);
    }

    #[test]
    fn sync_derived_stats_skips_recalculation_and_bonus_branches_when_clean() {
        let mut state = GameState::new("Hero".to_string(), 0);
        state.derived_stats_dirty = false;
        let expected = state.cached_derived_stats.max_hp;
        let enhancement = crate::enhancement::EnhancementProgress::new();
        let sigil_bonuses = SigilBonuses::default();

        sync_derived_stats(&mut state, &enhancement, &sigil_bonuses);

        assert_eq!(state.combat_state.player_max_hp, expected);
    }

    // ── compute_merged_bonuses ───────────────────────────────────────────────

    #[test]
    fn compute_merged_bonuses_recomputes_and_caches_when_dirty() {
        let mut state = GameState::new("Hero".to_string(), 0);
        state.bonuses_dirty = true;
        let haven = Haven::new();

        let (haven_bonuses, sigil_bonuses) = compute_merged_bonuses(&haven, &mut state);

        assert!(!state.bonuses_dirty);
        assert_eq!(
            state.cached_haven_bonuses.drop_rate_percent,
            haven_bonuses.drop_rate_percent
        );
        assert_eq!(
            state.cached_sigil_bonuses.xp_percent,
            sigil_bonuses.xp_percent
        );
    }

    #[test]
    fn compute_merged_bonuses_returns_cached_values_when_not_dirty() {
        let mut state = GameState::new("Hero".to_string(), 0);
        state.bonuses_dirty = false;
        state.cached_haven_bonuses.drop_rate_percent = 42.0;
        let haven = Haven::new();

        let (haven_bonuses, _sigil_bonuses) = compute_merged_bonuses(&haven, &mut state);

        assert_eq!(haven_bonuses.drop_rate_percent, 42.0);
    }

    // ── run_combat ───────────────────────────────────────────────────────────

    #[test]
    fn run_combat_computes_power_rating_and_processes_events() {
        let mut rng = ChaCha8Rng::seed_from_u64(141);
        let mut state = GameState::new("Hero".to_string(), 0);
        state.derived_stats_dirty = true;
        let enhancement = crate::enhancement::EnhancementProgress::new();
        sync_derived_stats(&mut state, &enhancement, &SigilBonuses::default());
        crate::core::game_logic::spawn_enemy_if_needed(&mut state);
        state.combat_state.player_current_hp = state.combat_state.player_max_hp;

        let mut achievements = Achievements::default();
        let mut deep = crate::deep::DeepState::new();
        let loom = crate::loom::LoomState::new();
        let mut result = TickResult::default();
        let haven_bonuses = Haven::new().compute_bonuses();
        let sigil_bonuses = SigilBonuses::default();

        run_combat(
            &mut state,
            1.6, // exceeds ATTACK_INTERVAL_SECONDS, guaranteeing at least one swing
            &haven_bonuses,
            &sigil_bonuses,
            &mut achievements,
            &mut deep,
            &loom,
            false,
            &mut result,
            &mut rng,
        );

        assert!(state.cached_power_rating > 0.0);
    }

    // ── update_play_time ─────────────────────────────────────────────────────

    #[test]
    fn update_play_time_increments_seconds_and_records_xp_sample() {
        let mut state = GameState::new("Hero".to_string(), 0);
        state.combat_seconds_this_tick = true;
        state.xp_this_second = 500;
        let mut tick_counter = TICKS_PER_SECOND - 1;
        let play_before = state.play_time_seconds;

        update_play_time(&mut state, &mut tick_counter);

        assert_eq!(tick_counter, 0);
        assert_eq!(state.play_time_seconds, play_before + 1);
        assert_eq!(state.xp_rate_samples.back(), Some(&500));
        assert!(!state.combat_seconds_this_tick);
        assert_eq!(state.xp_this_second, 0);
    }

    #[test]
    fn update_play_time_does_not_increment_before_full_second() {
        let mut state = GameState::new("Hero".to_string(), 0);
        let mut tick_counter = 0u32;
        let play_before = state.play_time_seconds;

        update_play_time(&mut state, &mut tick_counter);

        assert_eq!(tick_counter, 1);
        assert_eq!(state.play_time_seconds, play_before);
    }

    #[test]
    fn update_play_time_evicts_oldest_xp_sample_when_window_full() {
        let mut state = GameState::new("Hero".to_string(), 0);
        for _ in 0..crate::core::constants::XP_RATE_WINDOW_SECONDS {
            state.xp_rate_samples.push_back(0);
        }
        state.combat_seconds_this_tick = true;
        state.xp_this_second = 10;
        let mut tick_counter = TICKS_PER_SECOND - 1;

        update_play_time(&mut state, &mut tick_counter);

        assert_eq!(
            state.xp_rate_samples.len(),
            crate::core::constants::XP_RATE_WINDOW_SECONDS
        );
        assert_eq!(state.xp_rate_samples.back(), Some(&10));
    }

    // ── tick_haven_discovery / tick_soulforge_discovery ─────────────────────

    #[test]
    fn tick_haven_discovery_discovers_at_extreme_prestige() {
        let mut rng = ChaCha8Rng::seed_from_u64(151);
        let state = {
            let mut s = GameState::new("Hero".to_string(), 0);
            s.prestige_rank = 500_000; // pushes the (uncapped) discovery chance past 1.0
            s
        };
        let mut haven = Haven::new();
        let mut achievements = Achievements::default();
        let mut result = TickResult::default();

        tick_haven_discovery(
            &state,
            &mut haven,
            &mut achievements,
            false,
            &mut result,
            &mut rng,
        );

        assert!(haven.discovered);
        assert!(result.haven_changed);
        assert!(result.achievements_changed);
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, TickEvent::HavenDiscovered)));
    }

    #[test]
    fn tick_haven_discovery_debug_mode_suppresses_achievement_flag() {
        let mut rng = ChaCha8Rng::seed_from_u64(152);
        let mut state = GameState::new("Hero".to_string(), 0);
        state.prestige_rank = 500_000;
        let mut haven = Haven::new();
        let mut achievements = Achievements::default();
        let mut result = TickResult::default();

        tick_haven_discovery(
            &state,
            &mut haven,
            &mut achievements,
            true,
            &mut result,
            &mut rng,
        );

        assert!(haven.discovered);
        assert!(!result.achievements_changed);
    }

    #[test]
    fn tick_haven_discovery_skips_when_dungeon_active() {
        let mut rng = ChaCha8Rng::seed_from_u64(153);
        let mut state = GameState::new("Hero".to_string(), 0);
        state.prestige_rank = 500_000;
        state.active_dungeon = Some(crate::dungeon::generation::generate_dungeon(1, 0, 1));
        let mut haven = Haven::new();
        let mut achievements = Achievements::default();
        let mut result = TickResult::default();

        tick_haven_discovery(
            &state,
            &mut haven,
            &mut achievements,
            false,
            &mut result,
            &mut rng,
        );

        assert!(!haven.discovered);
    }

    #[test]
    fn tick_soulforge_discovery_discovers_at_extreme_prestige() {
        let mut rng = ChaCha8Rng::seed_from_u64(161);
        let mut state = GameState::new("Hero".to_string(), 0);
        state.prestige_rank = 500_000;
        let mut enhancement = crate::enhancement::EnhancementProgress::new();
        let mut achievements = Achievements::default();
        let mut result = TickResult::default();

        tick_soulforge_discovery(
            &state,
            &mut enhancement,
            &mut achievements,
            false,
            &mut result,
            &mut rng,
        );

        assert!(enhancement.discovered);
        assert!(result.enhancement_changed);
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, TickEvent::SoulforgeDiscovered)));
    }

    #[test]
    fn tick_soulforge_discovery_skips_when_fishing_active() {
        let mut rng = ChaCha8Rng::seed_from_u64(162);
        let mut state = GameState::new("Hero".to_string(), 0);
        state.prestige_rank = 500_000;
        state.active_fishing = Some(crate::fishing::FishingSession {
            spot_name: "Cove".to_string(),
            total_fish: 5,
            fish_caught: Vec::new(),
            items_found: Vec::new(),
            ticks_remaining: 1,
            phase: crate::fishing::FishingPhase::Casting,
        });
        let mut enhancement = crate::enhancement::EnhancementProgress::new();
        let mut achievements = Achievements::default();
        let mut result = TickResult::default();

        tick_soulforge_discovery(
            &state,
            &mut enhancement,
            &mut achievements,
            false,
            &mut result,
            &mut rng,
        );

        assert!(!enhancement.discovered);
    }

    // ── tick_deep_missions ───────────────────────────────────────────────────

    #[test]
    fn tick_deep_missions_noop_before_discovery() {
        let mut rng = ChaCha8Rng::seed_from_u64(171);
        let state = GameState::new("Hero".to_string(), 0);
        let mut deep = crate::deep::DeepState::new();
        let mut achievements = Achievements::default();
        let mut result = TickResult::default();

        tick_deep_missions(
            &state,
            &mut deep,
            &mut achievements,
            false,
            &mut result,
            &mut rng,
        );

        assert!(!result.deep_changed);
        assert!(result.events.is_empty());
    }

    #[test]
    fn tick_deep_missions_refreshes_pool_once_discovered() {
        let mut rng = ChaCha8Rng::seed_from_u64(172);
        let state = GameState::new("Hero".to_string(), 0);
        let mut deep = crate::deep::DeepState::new();
        deep.persistent.discovered = true;
        let mut achievements = Achievements::default();
        let mut result = TickResult::default();

        tick_deep_missions(
            &state,
            &mut deep,
            &mut achievements,
            false,
            &mut result,
            &mut rng,
        );

        assert!(result.deep_changed);
        assert!(!deep.prestige.available_missions.is_empty());
    }

    #[test]
    fn tick_deep_missions_breakthrough_success_unlocks_fracture_region() {
        use crate::deep::{
            MercArchetype, MercQuality, MercStatus, Mercenary, Mission, MissionStatus, MissionType,
        };
        use chrono::{Duration, Utc};

        let mut succeeded = false;
        for seed in 0u64..20 {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let state = GameState::new("Hero".to_string(), 0);
            let mut deep = crate::deep::DeepState::new();
            deep.persistent.discovered = true;

            let merc = Mercenary {
                id: 1,
                name: "Bruiser".to_string(),
                archetype: MercArchetype::Vanguard,
                power: 999_999,
                resilience: 999_999,
                expertise: 0,
                level: 1,
                missions_completed: 0,
                quality: MercQuality::Common,
                status: MercStatus::OnMission(1),
            };
            deep.prestige.roster.insert(merc.id, merc);

            let now = Utc::now();
            let mission = Mission {
                id: 1,
                mission_type: MissionType::Breakthrough,
                layer: 3,
                squad: vec![1],
                started_at: now - Duration::hours(4),
                ends_at: now - Duration::seconds(5),
                events: Vec::new(),
                pending_event_index: 0,
                status: MissionStatus::Active,
                result: None,
                is_first_orders: false,
            };
            deep.prestige.active_missions.push(mission);

            let mut achievements = Achievements::default();
            let mut result = TickResult::default();

            tick_deep_missions(
                &state,
                &mut deep,
                &mut achievements,
                false,
                &mut result,
                &mut rng,
            );

            if deep.persistent.pending_fracture_region_unlock.is_some() {
                assert!(result.deep_changed);
                assert_eq!(deep.persistent.fracture_zone_cap, 14);
                assert!(result
                    .events
                    .iter()
                    .any(|e| matches!(e, TickEvent::DeepMissionComplete { .. })));
                assert!(achievements
                    .take_newly_unlocked()
                    .contains(&AchievementId::FirstBreakthrough));
                succeeded = true;
                break;
            }
        }

        assert!(
            succeeded,
            "An overpowered Breakthrough mission should succeed and unlock a fracture region within 20 seeds"
        );
    }

    // ── tick_loom ────────────────────────────────────────────────────────────

    #[test]
    fn tick_loom_noop_before_gateway_opens() {
        let deep = crate::deep::DeepState::new();
        let mut loom = crate::loom::LoomState::new();
        let mut state = GameState::new("Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        let mut result = TickResult::default();

        tick_loom(&deep, &mut loom, &mut state, &mut achievements, &mut result);

        assert!(!loom.persistent.discovered);
        assert!(result.events.is_empty());
    }

    #[test]
    fn tick_loom_discovers_when_gateway_opens() {
        let mut deep = crate::deep::DeepState::new();
        deep.persistent.discovered = true;
        deep.persistent.gateway_opened = true;
        let mut loom = crate::loom::LoomState::new();
        let mut state = GameState::new("Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        let mut result = TickResult::default();

        tick_loom(&deep, &mut loom, &mut state, &mut achievements, &mut result);

        assert!(loom.persistent.discovered);
        assert!(result.loom_changed);
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, TickEvent::LoomDiscovered)));
    }

    #[test]
    fn tick_loom_skips_ticking_during_chrono_surge() {
        let mut deep = crate::deep::DeepState::new();
        deep.persistent.discovered = true;
        deep.persistent.gateway_opened = true;
        let mut loom = crate::loom::LoomState::new();
        crate::loom::complete_discovery(&mut loom);
        let mut state = GameState::new("Hero".to_string(), 0);
        state.chrono_surge_active = true;
        let mut achievements = Achievements::default();
        let mut result = TickResult::default();

        tick_loom(&deep, &mut loom, &mut state, &mut achievements, &mut result);

        assert!(loom.last_tick_at.is_none());
    }

    #[test]
    fn tick_loom_ticks_base_production_after_discovery() {
        let mut deep = crate::deep::DeepState::new();
        deep.persistent.discovered = true;
        deep.persistent.gateway_opened = true;
        let mut loom = crate::loom::LoomState::new();
        crate::loom::complete_discovery(&mut loom);
        let mut state = GameState::new("Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        let mut result = TickResult::default();

        tick_loom(&deep, &mut loom, &mut state, &mut achievements, &mut result);
        assert!(loom.last_tick_at.is_some());

        // A second tick exercises the "previous tick exists" wall-clock delta branch.
        tick_loom(&deep, &mut loom, &mut state, &mut achievements, &mut result);
        assert!(loom.last_tick_at.is_some());
    }

    #[test]
    fn tick_loom_completes_pattern_and_initializes_wr_pr_conversion() {
        let mut deep = crate::deep::DeepState::new();
        deep.persistent.discovered = true;
        deep.persistent.gateway_opened = true;

        let mut loom = crate::loom::LoomState::new();
        crate::loom::complete_discovery(&mut loom);
        loom.persistent.active_pattern = 0;
        loom.persistent.patterns = vec![crate::loom::WovenPattern {
            index: 0,
            name: "Test Pattern".to_string(),
            requirements: vec![crate::loom::PatternRequirement {
                resource: crate::loom::Resource::Ember,
                required_rate: 0.0,
                sustain_duration_secs: 0.01,
                sustained_secs: 0.0,
                completed: false,
                amount: 0.0,
                accumulated: 0.0,
            }],
            completed: false,
            flavor: String::new(),
            eternal: false,
        }];
        // Force a real wall-clock delta so the sustain timer clearly exceeds the
        // tiny requirement duration on the very next tick.
        loom.last_tick_at = Some(chrono::Utc::now() - chrono::Duration::seconds(1));

        let mut state = GameState::new("Hero".to_string(), 0);
        let mut achievements = Achievements::default();
        let mut result = TickResult::default();

        tick_loom(&deep, &mut loom, &mut state, &mut achievements, &mut result);

        assert!(loom.persistent.patterns[0].completed);
        assert!(result.loom_changed);
        assert_ne!(loom.persistent.wr_pr_last_granted_at, 0);
    }
}
