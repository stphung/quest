//! Tick stage helper functions extracted from `game_tick()`.
//!
//! Each function implements one processing stage of the game tick loop,
//! keeping `game_tick()` in `tick.rs` readable as a high-level orchestrator.

use super::tick_types::{TickEvent, TickResult};
use crate::achievements::Achievements;
use crate::combat::CombatEvent;
use crate::core::constants::{FINAL_ZONE_ID, STORMGLASS_MIN_PRESTIGE_RANK, TICKS_PER_SECOND};
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
use crate::haven::HavenBonuses;
use crate::items::drops::{try_drop_from_boss, try_drop_from_mob};
use crate::items::scoring::auto_equip_if_better;
use crate::items::types::Rarity;
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

    let god_item_dungeon_speed =
        crate::god_items::equipped_god_item_dungeon_speed_percent(&state.equipment);
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
                        let status = if equipped {
                            "Equipped!"
                        } else {
                            "Kept current gear"
                        };
                        let power = item.power();
                        let treasure_rarity = item.rarity;
                        let treasure_name = item.display_name.clone();
                        let msg = format!("\u{1f48e} Found: {} [{}]", item.display_name, status);
                        result.events.push(TickEvent::DungeonTreasureFound {
                            item_name: item.display_name,
                            rarity: item.rarity,
                            tier: item.tier,
                            ilvl: item.ilvl,
                            power,
                            equipped,
                            message: msg,
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
                result.events.push(TickEvent::DungeonKeyFound {
                    message: "\u{1f5dd}\u{fe0f} A heavy key clatters to the ground. The way forward is open.".to_string(),
                });
            }
            crate::dungeon::logic::DungeonEvent::BossUnlocked => {
                result.events.push(TickEvent::DungeonBossUnlocked {
                    message: "\u{1f479} Somewhere deep in the dungeon, a sealed door grinds open."
                        .to_string(),
                });
            }
            crate::dungeon::logic::DungeonEvent::DungeonComplete {
                xp_earned,
                items_collected,
            } => {
                let message = format!(
                    "\u{1f3c6} Dungeon Complete! +{} XP, {} items found",
                    xp_earned, items_collected
                );
                result.events.push(TickEvent::DungeonCompleted {
                    xp_earned,
                    items_collected,
                    message,
                });
            }
            crate::dungeon::logic::DungeonEvent::DungeonFailed => {
                result.events.push(TickEvent::DungeonFailed {
                    message:
                        "\u{1f480} The dungeon spits you out, broken but alive. No prestige lost."
                            .to_string(),
                });
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
    let god_item_fishing_reduction =
        crate::god_items::equipped_god_item_fishing_reduction_percent(&state.equipment);
    let fishing_result =
        tick_fishing_with_haven_result(state, rng, &haven_fishing, god_item_fishing_reduction);
    let level_before = state.character_level;

    // Storm Leviathan caught -> achievement
    if fishing_result.caught_storm_leviathan {
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
    let current_enemy_name = state
        .combat_state
        .current_enemy
        .as_ref()
        .map(|e| e.name.clone())
        .unwrap_or_default();

    for event in combat_events {
        match event {
            CombatEvent::PlayerAttackBlocked { weapon_needed } => {
                let message = format!("\u{1f6ab} {} required to damage this foe!", weapon_needed);
                result.events.push(TickEvent::PlayerAttackBlocked {
                    weapon_needed,
                    message,
                });
            }
            CombatEvent::PlayerAttack { damage, was_crit } => {
                let message = if was_crit {
                    format!("\u{1f4a5} CRITICAL HIT for {} damage!", damage)
                } else {
                    format!("\u{2694} You hit for {} damage", damage)
                };
                result.events.push(TickEvent::PlayerAttack {
                    damage,
                    was_crit,
                    message,
                });
            }
            CombatEvent::EnemyAttack { damage } => {
                let enemy_name = current_enemy_name.clone();
                let message = format!("\u{1f6e1} {} hits you for {} damage", enemy_name, damage);
                result.events.push(TickEvent::EnemyAttack {
                    damage,
                    enemy_name,
                    message,
                });
            }
            CombatEvent::DamageReflected { damage } => {
                let message = format!("\u{1f4a5} {} reflected!", damage);
                result
                    .events
                    .push(TickEvent::DamageReflected { damage, message });
            }
            CombatEvent::RegenComplete { healed } => {
                result.events.push(TickEvent::RegenComplete { healed });
            }
            CombatEvent::EnemyDied { xp_gained } => {
                let enemy_name = current_enemy_name.clone();
                let message = format!("\u{2728} {} defeated! +{} XP", enemy_name, xp_gained);
                result.events.push(TickEvent::EnemyDefeated {
                    xp_gained,
                    enemy_name,
                    message,
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
                let enemy_name = current_enemy_name.clone();
                let message = format!(
                    "\u{2694}\u{fe0f} {} defeated! +{} XP",
                    enemy_name, xp_gained
                );
                result.events.push(TickEvent::DungeonEliteDefeated {
                    xp_gained,
                    enemy_name,
                    message,
                });

                apply_xp_and_check_levelup(rng, state, xp_gained as f64, achievements, result);
                add_dungeon_xp(state, xp_gained);

                // Give key
                if let Some(dungeon) = &mut state.active_dungeon {
                    let events = on_elite_defeated(dungeon);
                    for de in events {
                        if matches!(de, crate::dungeon::logic::DungeonEvent::FoundKey) {
                            result.events.push(TickEvent::DungeonKeyFound {
                                message: "\u{1f5dd}\u{fe0f} A heavy key clatters to the ground. The way forward is open.".to_string(),
                            });
                        }
                    }
                }
            }
            CombatEvent::BossDefeated { xp_gained } => {
                let enemy_name = current_enemy_name.clone();

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

                let message = format!(
                    "\u{1f3c6} Dungeon Complete! +{} bonus XP ({} total, {} items)",
                    bonus_xp, total_xp, items
                );
                result.events.push(TickEvent::DungeonBossDefeated {
                    xp_gained,
                    bonus_xp,
                    total_xp,
                    items_collected: items,
                    enemy_name,
                    message,
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
                result.events.push(TickEvent::PlayerDiedInDungeon {
                    message: "\u{1f480} You fell in the dungeon... (escaped without prestige loss)"
                        .to_string(),
                });
            }
            CombatEvent::PlayerDied => {
                result.events.push(TickEvent::PlayerDied {
                    message: "\u{1f480} You died! Boss encounter reset.".to_string(),
                });
            }
            CombatEvent::SubzoneBossDefeated {
                xp_gained,
                result: defeat_result,
            } => {
                apply_xp_and_check_levelup(rng, state, xp_gained as f64, achievements, result);
                state.session_kills += 1;

                // Track zone achievements
                process_zone_achievements(&defeat_result, achievements, &state.character_name);

                // Build message
                let message = match &defeat_result {
                    BossDefeatResult::SubzoneComplete { .. } => {
                        format!(
                            "\u{1f451} Boss defeated! +{} XP \u{2014} Moving to next area.",
                            xp_gained
                        )
                    }
                    BossDefeatResult::ZoneComplete {
                        old_zone,
                        new_zone_id,
                    } => {
                        let new_zone = crate::zones::get_zone(*new_zone_id)
                            .map(|z| z.name)
                            .unwrap_or("???");
                        format!(
                            "\u{1f451} {} conquered! +{} XP \u{2014} Advancing to {}!",
                            old_zone, xp_gained, new_zone
                        )
                    }
                    BossDefeatResult::ZoneCompleteButGated {
                        zone_name,
                        required_prestige,
                    } => {
                        format!(
                            "\u{1f451} {} conquered! +{} XP \u{2014} Next zone requires Prestige {}.",
                            zone_name, xp_gained, required_prestige
                        )
                    }
                    BossDefeatResult::StormsEnd => {
                        format!(
                            "\u{1f451} All zones conquered! +{} XP \u{2014} You have completed the game!",
                            xp_gained
                        )
                    }
                    BossDefeatResult::WeaponRequired { .. } => {
                        // Already handled by PlayerAttackBlocked
                        continue;
                    }
                    BossDefeatResult::ExpanseCycle => {
                        format!(
                            "\u{1f451} The Endless defeated! +{} XP \u{2014} The Expanse cycles anew...",
                            xp_gained
                        )
                    }
                };
                result.events.push(TickEvent::SubzoneBossDefeated {
                    xp_gained,
                    result: defeat_result.clone(),
                    message,
                });

                // Deep discovery: first Endless kill at P15+
                if matches!(defeat_result, BossDefeatResult::ExpanseCycle)
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
            }
            CombatEvent::CombatRetreat { zone_name } => {
                let message = format!("\u{1f3c3} Overwhelmed! You retreat to {}...", zone_name);
                result
                    .events
                    .push(TickEvent::CombatRetreat { zone_name, message });
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
        state.add_recent_drop(
            item_name.clone(),
            rarity,
            equipped,
            icon,
            slot.clone(),
            stats.clone(),
        );
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

        // Stormglass: salvage non-equipped items
        // Discovery requires P15+; once discovered, salvage always works
        if !equipped
            && (state.stormglass_discovered || state.prestige_rank >= STORMGLASS_MIN_PRESTIGE_RANK)
        {
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
        BossDefeatResult::ZoneComplete { old_zone, .. }
        | BossDefeatResult::ZoneCompleteButGated {
            zone_name: old_zone,
            ..
        } => {
            if let Some(zone) = crate::zones::get_all_zones()
                .iter()
                .find(|z| z.name == *old_zone)
            {
                achievements.on_zone_fully_cleared(zone.id, Some(character_name));
            }
        }
        BossDefeatResult::StormsEnd => {
            achievements.on_zone_fully_cleared(10, Some(character_name));
            achievements.on_storms_end(Some(character_name));
        }
        BossDefeatResult::ExpanseCycle => {
            achievements.on_zone_fully_cleared(11, Some(character_name));
        }
        _ => {}
    }
}

/// Collect newly unlocked achievements into TickResult events.
pub(super) fn collect_achievement_events(achievements: &mut Achievements, result: &mut TickResult) {
    for id in achievements.take_newly_unlocked() {
        if let Some(def) = crate::achievements::get_achievement_def(id) {
            let message = format!("\u{1f3c6} Achievement Unlocked: {}", def.name);
            result.events.push(TickEvent::AchievementUnlocked {
                name: def.name.to_string(),
                message,
            });
            result.achievements_changed = true;
        }
    }
}
