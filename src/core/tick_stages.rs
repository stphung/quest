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
    let current_enemy_name = state
        .combat_state
        .current_enemy
        .as_ref()
        .map(|e| e.name.clone())
        .unwrap_or_default();

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
                let enemy_name = current_enemy_name.clone();
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
                let enemy_name = current_enemy_name.clone();
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
                let enemy_name = current_enemy_name.clone();
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
            }
            CombatEvent::CombatRetreat { zone_name } => {
                result.events.push(TickEvent::CombatRetreat { zone_name });
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
        | BossDefeatResult::LoomZoneCycle { zone_id } => {
            achievements.on_zone_fully_cleared(*zone_id, Some(character_name));
        }
        _ => {}
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
        max_hp = (max_hp as f64 * ascension_mult) as u32;
    }

    // Apply sigil max HP% bonus
    if sigil_bonuses.max_hp_percent > 0.0 {
        max_hp = (max_hp as f64 * (1.0 + sigil_bonuses.max_hp_percent / 100.0)) as u32;
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

    if summary.missions_completed > 0 || summary.events_fired > 0 {
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

    // Loom runs on wall-clock time only — skip during Chrono Surge bursts.
    if state.chrono_surge_active {
        return;
    }

    let warp = if loom.time_warp > 0.0 {
        loom.time_warp
    } else {
        1.0
    };
    let tick_seconds: f64 = 0.1 * warp; // 100ms tick interval × debug time warp

    // Tick staggered second-node unlock.
    if loom.persistent.second_node_unlock_elapsed.is_some() {
        let unlocked = crate::loom::tick_loom_staggered_unlock(loom, tick_seconds);
        if unlocked {
            result.loom_changed = true;
        }
    }

    // Tick shuttle construction (decrement timers, complete when done).
    let completed_shuttles = crate::loom::tick_shuttle_construction(loom);
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
            let pr_per_day = crate::loom::wr_to_pr_per_day(wr_rate);
            if pr_per_day > 0 {
                let fill_secs = 86400i64 / pr_per_day as i64;
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
            ],
            &haven_bonuses,
            &mut achievements,
            &mut deep,
            false,
            &mut result,
            &mut rng,
        );

        assert_eq!(result.events.len(), 8);
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
}
