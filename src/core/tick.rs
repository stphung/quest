//! Extracted game tick logic — the central per-tick orchestration function.
//!
//! This module contains the `game_tick()` function that processes a single
//! 100ms game tick, updating combat, fishing, dungeon, challenges, achievements,
//! and play time. It returns a [`TickResult`] describing what happened so the
//! presentation layer (main.rs) can update the UI without game logic depending
//! on any UI types.

use crate::achievements::Achievements;
use crate::challenges::menu::ChallengeType;
use crate::challenges::ActiveMinigame;
use crate::character::derived_stats::DerivedStats;
use crate::character::prestige::PrestigeCombatBonuses;
use crate::combat::logic::{update_combat, CombatEvent, HavenCombatBonuses};
use crate::core::constants::{
    FINAL_ZONE_ID, HAVEN_MIN_PRESTIGE_RANK, TICKS_PER_SECOND, TICK_INTERVAL_MS,
};
use crate::core::game_logic::{apply_tick_xp, spawn_enemy_if_needed, try_discover_dungeon};
use crate::core::game_state::GameState;
use crate::dungeon::logic::{
    add_dungeon_xp, calculate_boss_xp_reward, on_boss_defeated, on_elite_defeated,
    on_room_enemy_defeated, on_treasure_room_entered, update_dungeon,
};
use crate::dungeon::types::RoomType;
use crate::fishing::logic::{
    check_rank_up_with_max, get_max_fishing_rank, tick_fishing_with_haven_result,
    HavenFishingBonuses,
};
use crate::haven::Haven;
use crate::haven::HavenBonusType;
use crate::items::drops::{try_drop_from_boss, try_drop_from_mob};
use crate::items::scoring::auto_equip_if_better;
use crate::items::types::Rarity;
use crate::zones::BossDefeatResult;
use rand::{Rng, RngExt};

/// A single event produced by a game tick.
///
/// The presentation layer (main.rs) maps these to combat log entries,
/// visual effects, and UI state changes. The game logic layer never
/// touches UI types directly.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields are part of the public API contract; main.rs matches with `..`
pub enum TickEvent {
    // ── Combat ──────────────────────────────────────────────────
    /// Player attacked an enemy.
    PlayerAttack {
        damage: u32,
        was_crit: bool,
        message: String,
    },

    /// Player's attack was blocked because the boss requires a specific weapon.
    PlayerAttackBlocked {
        weapon_needed: String,
        message: String,
    },

    /// Enemy attacked the player.
    EnemyAttack {
        damage: u32,
        enemy_name: String,
        message: String,
    },

    /// Normal enemy or dungeon combat-room enemy was defeated.
    EnemyDefeated {
        xp_gained: u64,
        enemy_name: String,
        message: String,
    },

    /// Player died in overworld combat (boss encounter reset).
    PlayerDied { message: String },

    /// Player died in a dungeon (safe exit, no prestige loss).
    PlayerDiedInDungeon { message: String },

    // ── Item Drops ──────────────────────────────────────────────
    /// An item was dropped and auto-equip was evaluated.
    ItemDropped {
        item_name: String,
        rarity: Rarity,
        equipped: bool,
        slot: String,
        stats: String,
        from_boss: bool,
    },

    // ── Zone Progression ────────────────────────────────────────
    /// A subzone boss was defeated and zone progression updated.
    SubzoneBossDefeated {
        xp_gained: u64,
        result: BossDefeatResult,
        message: String,
    },

    // ── Dungeon ─────────────────────────────────────────────────
    /// Player entered a dungeon room during auto-exploration.
    DungeonRoomEntered {
        room_type: RoomType,
        message: String,
    },

    /// Treasure found in a dungeon treasure room.
    DungeonTreasureFound {
        item_name: String,
        equipped: bool,
        message: String,
    },

    /// Dungeon key found (from defeating the elite guardian).
    DungeonKeyFound { message: String },

    /// Boss room is now unlocked.
    DungeonBossUnlocked { message: String },

    /// Dungeon boss defeated — dungeon completed with rewards.
    DungeonBossDefeated {
        xp_gained: u64,
        bonus_xp: u64,
        total_xp: u64,
        items_collected: usize,
        enemy_name: String,
        message: String,
    },

    /// Dungeon elite enemy defeated.
    DungeonEliteDefeated {
        xp_gained: u64,
        enemy_name: String,
        message: String,
    },

    /// Player died or was removed from dungeon.
    DungeonFailed { message: String },

    /// Dungeon completed event from auto-exploration (update_dungeon).
    DungeonCompleted {
        xp_earned: u64,
        items_collected: usize,
        message: String,
    },

    // ── Fishing ─────────────────────────────────────────────────
    /// A generic fishing phase/event message.
    FishingMessage { message: String },

    /// A fish was caught (for recent-drops tracking in the UI).
    FishCaught {
        fish_name: String,
        rarity: Rarity,
        message: String,
    },

    /// An item was found while fishing.
    FishingItemFound { item_name: String, message: String },

    /// Fishing rank increased.
    FishingRankUp { message: String },

    /// The Storm Leviathan was caught (triggers achievement).
    StormLeviathanCaught,

    // ── Discovery ───────────────────────────────────────────────
    /// A challenge minigame was discovered.
    ChallengeDiscovered {
        challenge_type: ChallengeType,
        message: String,
        follow_up: String,
    },

    /// A dungeon entrance was discovered after killing an enemy.
    DungeonDiscovered { message: String },

    /// A fishing spot was discovered after killing an enemy.
    FishingSpotDiscovered { message: String },

    /// The Haven was discovered (P10+ idle roll).
    HavenDiscovered,

    // ── Achievements ────────────────────────────────────────────
    /// An achievement was unlocked during this tick.
    AchievementUnlocked { name: String, message: String },

    // ── Level Up ────────────────────────────────────────────────
    /// Player leveled up (may occur multiple times per tick from large XP gains).
    LeveledUp { new_level: u32 },
}

/// Result of processing a single game tick.
#[derive(Debug, Clone, Default)]
pub struct TickResult {
    /// Events produced during this tick, in chronological order.
    pub events: Vec<TickEvent>,

    /// If set, a Storm Leviathan encounter occurred during fishing.
    /// The value is the encounter number (1-10). The presentation layer
    /// uses this to show the Leviathan modal overlay.
    pub leviathan_encounter: Option<u8>,

    /// True if achievements were modified and should be persisted to disk.
    /// The presentation layer is responsible for the actual IO.
    pub achievements_changed: bool,

    /// True if Haven state was modified (discovery) and should be persisted.
    pub haven_changed: bool,

    /// Achievement IDs ready to be shown in a modal overlay.
    /// Populated when the 500ms accumulation window has elapsed.
    /// Empty if no modal is ready or another overlay is already active.
    pub achievement_modal_ready: Vec<crate::achievements::AchievementId>,
}

/// Processes a single 100ms game tick.
///
/// Updates game state (combat, fishing, dungeon, challenges, achievements,
/// play time) and returns a [`TickResult`] describing what happened.
///
/// # Arguments
/// - `state` — Mutable game state (character, combat, zones, equipment, etc.)
/// - `tick_counter` — Counts ticks for play-time tracking (10 ticks = 1 second).
///   Caller owns this counter across ticks.
/// - `haven` — Mutable Haven state for bonus calculations and discovery.
/// - `achievements` — Mutable achievement state for unlock tracking.
/// - `debug_mode` — When true, suppresses achievement/haven-save signals.
/// - `rng` — Random number generator (any `impl Rng`). Pass
///   `&mut rand::rng()` in production, or a seeded
///   `rand_chacha::ChaCha8Rng` in tests for deterministic behavior.
///
/// # Returns
/// A [`TickResult`] containing all events and flags. The caller (main.rs)
/// is responsible for:
/// - Mapping events to combat log entries via `add_log_entry()`
/// - Creating `VisualEffect` objects for [`TickEvent::PlayerAttack`] events
/// - Updating `visual_effects` lifetimes
/// - Persisting achievements to disk when `achievements_changed` is true
/// - Persisting Haven to disk when `haven_changed` is true
/// - Showing the Leviathan encounter modal when `leviathan_encounter` is `Some`
/// - Showing achievement modal overlay when `achievement_modal_ready` is non-empty
pub fn game_tick<R: Rng>(
    state: &mut GameState,
    tick_counter: &mut u32,
    haven: &mut Haven,
    achievements: &mut Achievements,
    debug_mode: bool,
    rng: &mut R,
) -> TickResult {
    let mut result = TickResult::default();
    let delta_time = TICK_INTERVAL_MS as f64 / 1000.0;

    // ── 1. Process challenge AI thinking ────────────────────────
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
        _ => {}
    }

    // ── 2. Try challenge discovery ──────────────────────────────
    {
        let haven_discovery = haven.get_bonus(HavenBonusType::ChallengeDiscoveryPercent);
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

    // ── 3. Sync player max HP with derived stats ────────────────
    let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
    state.combat_state.update_max_hp(derived.max_hp);

    // ── 4. Update dungeon exploration ───────────────────────────
    if state.active_dungeon.is_some() {
        let dungeon_events = update_dungeon(state, delta_time);
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
                        if let Some((item, equipped)) = on_treasure_room_entered(state) {
                            let status = if equipped {
                                "Equipped!"
                            } else {
                                "Kept current gear"
                            };
                            let msg =
                                format!("\u{1f48e} Found: {} [{}]", item.display_name, status);
                            result.events.push(TickEvent::DungeonTreasureFound {
                                item_name: item.display_name,
                                equipped,
                                message: msg,
                            });
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
                        message: "\u{1f479} Somewhere deep in the dungeon, a sealed door grinds open.".to_string(),
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
                        message: "\u{1f480} The dungeon spits you out, broken but alive. No prestige lost.".to_string(),
                    });
                }
                _ => {}
            }
        }
    }

    // ── 5. Update fishing (mutually exclusive with combat) ──────
    if state.active_fishing.is_some() {
        let haven_fishing = HavenFishingBonuses {
            timer_reduction_percent: haven.get_bonus(HavenBonusType::FishingTimerReduction),
            double_fish_chance_percent: haven.get_bonus(HavenBonusType::DoubleFishChance),
            max_fishing_rank_bonus: haven.fishing_rank_bonus(),
        };
        let fishing_result = tick_fishing_with_haven_result(state, rng, &haven_fishing);

        // Storm Leviathan caught → achievement
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

        // Check fishing rank up
        let max_rank = get_max_fishing_rank(haven_fishing.max_fishing_rank_bonus);
        if let Some(rank_msg) = check_rank_up_with_max(&mut state.fishing, max_rank) {
            let prefixed = format!("\u{1f3a3} {}", rank_msg);
            result
                .events
                .push(TickEvent::FishingRankUp { message: prefixed });
        }

        // Leviathan encounter
        result.leviathan_encounter = fishing_result.leviathan_encounter;

        // Update play time while fishing
        *tick_counter += 1;
        if *tick_counter >= TICKS_PER_SECOND {
            state.play_time_seconds += 1;
            *tick_counter = 0;
        }

        // Skip combat processing while fishing — collect achievements and return
        collect_achievement_events(achievements, &mut result);
        return result;
    }

    // ── 6. Combat ───────────────────────────────────────────────
    let haven_combat = HavenCombatBonuses {
        hp_regen_percent: haven.get_bonus(HavenBonusType::HpRegenPercent),
        hp_regen_delay_reduction: haven.get_bonus(HavenBonusType::HpRegenDelayReduction),
        damage_percent: haven.get_bonus(HavenBonusType::DamagePercent),
        crit_chance_percent: haven.get_bonus(HavenBonusType::CritChancePercent),
        double_strike_chance: haven.get_bonus(HavenBonusType::DoubleStrikeChance),
        xp_gain_percent: haven.get_bonus(HavenBonusType::XpGainPercent),
    };
    let prestige_combat = PrestigeCombatBonuses::from_rank(state.prestige_rank);
    // Apply prestige flat HP bonus to combat max HP (not in DerivedStats to avoid enemy scaling)
    if prestige_combat.flat_hp > 0 {
        let boosted_max = derived.max_hp + prestige_combat.flat_hp;
        state.combat_state.update_max_hp(boosted_max);
    }
    let combat_events = update_combat(
        state,
        delta_time,
        &haven_combat,
        &prestige_combat,
        achievements,
        &derived,
    );

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
                let enemy_name = state
                    .combat_state
                    .current_enemy
                    .as_ref()
                    .map(|e| e.name.clone())
                    .unwrap_or_default();
                let message = format!("\u{1f6e1} {} hits you for {} damage", enemy_name, damage);
                result.events.push(TickEvent::EnemyAttack {
                    damage,
                    enemy_name,
                    message,
                });
            }
            CombatEvent::EnemyDied { xp_gained } => {
                let enemy_name = state
                    .combat_state
                    .current_enemy
                    .as_ref()
                    .map(|e| e.name.clone())
                    .unwrap_or_default();
                let message = format!("\u{2728} {} defeated! +{} XP", enemy_name, xp_gained);
                result.events.push(TickEvent::EnemyDefeated {
                    xp_gained,
                    enemy_name,
                    message,
                });

                // Apply XP and check level up
                let level_before = state.character_level;
                apply_tick_xp(state, xp_gained as f64);
                if state.character_level > level_before {
                    achievements.on_level_up(state.character_level, Some(&state.character_name));
                    result.events.push(TickEvent::LeveledUp {
                        new_level: state.character_level,
                    });
                }
                state.session_kills += 1;

                // Track XP in dungeon and mark room cleared
                add_dungeon_xp(state, xp_gained);
                if let Some(dungeon) = &mut state.active_dungeon {
                    on_room_enemy_defeated(dungeon);
                }

                // Item drops
                process_item_drop(state, haven, &mut result);

                // Discovery: dungeon, then fishing
                process_discoveries(state, rng, &mut result);
            }
            CombatEvent::EliteDefeated { xp_gained } => {
                let enemy_name = state
                    .combat_state
                    .current_enemy
                    .as_ref()
                    .map(|e| e.name.clone())
                    .unwrap_or_default();
                let message = format!(
                    "\u{2694}\u{fe0f} {} defeated! +{} XP",
                    enemy_name, xp_gained
                );
                result.events.push(TickEvent::DungeonEliteDefeated {
                    xp_gained,
                    enemy_name,
                    message,
                });

                let level_before = state.character_level;
                apply_tick_xp(state, xp_gained as f64);
                if state.character_level > level_before {
                    achievements.on_level_up(state.character_level, Some(&state.character_name));
                    result.events.push(TickEvent::LeveledUp {
                        new_level: state.character_level,
                    });
                }
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
                let enemy_name = state
                    .combat_state
                    .current_enemy
                    .as_ref()
                    .map(|e| e.name.clone())
                    .unwrap_or_default();

                let level_before = state.character_level;
                apply_tick_xp(state, xp_gained as f64);

                // Calculate boss bonus XP
                let (bonus_xp, total_xp, items) = if let Some(dungeon) = &state.active_dungeon {
                    let bonus = calculate_boss_xp_reward(dungeon.size);
                    let total = dungeon.xp_earned + xp_gained + bonus;
                    let item_count = dungeon.collected_items.len();
                    (bonus, total, item_count)
                } else {
                    (0, xp_gained, 0)
                };

                apply_tick_xp(state, bonus_xp as f64);
                if state.character_level > level_before {
                    achievements.on_level_up(state.character_level, Some(&state.character_name));
                    result.events.push(TickEvent::LeveledUp {
                        new_level: state.character_level,
                    });
                }

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
                let level_before = state.character_level;
                apply_tick_xp(state, xp_gained as f64);
                if state.character_level > level_before {
                    achievements.on_level_up(state.character_level, Some(&state.character_name));
                    result.events.push(TickEvent::LeveledUp {
                        new_level: state.character_level,
                    });
                }
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
                    result: defeat_result,
                    message,
                });
            }
        }
    }

    // ── 7. Spawn enemy if needed ────────────────────────────────
    spawn_enemy_if_needed(state);

    // ── 8. Update play time ─────────────────────────────────────
    *tick_counter += 1;
    if *tick_counter >= TICKS_PER_SECOND {
        state.play_time_seconds += 1;
        *tick_counter = 0;
    }

    // ── 9. Collect achievement notifications ────────────────────
    collect_achievement_events(achievements, &mut result);

    // ── 10. Haven discovery check ────────────────────────────────
    // Independent roll per tick, only when eligible (P10+, no active content)
    if !haven.discovered
        && state.prestige_rank >= HAVEN_MIN_PRESTIGE_RANK
        && state.active_dungeon.is_none()
        && state.active_fishing.is_none()
        && state.active_minigame.is_none()
        && crate::haven::try_discover_haven(haven, state.prestige_rank, rng)
    {
        // Track Haven discovery achievement
        achievements.on_haven_discovered(Some(&state.character_name));
        result.events.push(TickEvent::HavenDiscovered);
        result.haven_changed = true;
        if !debug_mode {
            result.achievements_changed = true;
        }
    }

    // ── 11. Achievement modal accumulation ────────────────────────
    if achievements.is_modal_ready() {
        result.achievement_modal_ready = achievements.take_modal_queue();
    }

    result
}

/// Collect newly unlocked achievements into TickResult events.
fn collect_achievement_events(achievements: &mut Achievements, result: &mut TickResult) {
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

/// Process item drops after killing a mob/boss in overworld combat.
fn process_item_drop(state: &mut GameState, haven: &Haven, result: &mut TickResult) {
    let zone_id = state.zone_progression.current_zone_id as usize;
    let was_boss = state.zone_progression.fighting_boss;
    let is_final_zone = zone_id == FINAL_ZONE_ID as usize;

    let dropped_item = if was_boss {
        Some(try_drop_from_boss(zone_id, is_final_zone))
    } else {
        let haven_drop_rate = haven.get_bonus(HavenBonusType::DropRatePercent);
        let haven_rarity = haven.get_bonus(HavenBonusType::ItemRarityPercent);
        try_drop_from_mob(state, zone_id, haven_drop_rate, haven_rarity)
    };

    if let Some(item) = dropped_item {
        let item_name = item.display_name.clone();
        let rarity = item.rarity;
        let slot = item.slot_name().to_string();
        let stats = item.stat_summary();
        let icon = if was_boss { "\u{1f451}" } else { "\u{1f381}" };
        let equipped = auto_equip_if_better(item, state);
        state.add_recent_drop(
            item_name.clone(),
            rarity,
            equipped,
            icon,
            slot.clone(),
            stats.clone(),
        );
        result.events.push(TickEvent::ItemDropped {
            item_name,
            rarity,
            equipped,
            slot,
            stats,
            from_boss: was_boss,
        });
    }
}

/// Try to discover dungeon or fishing spot after killing an enemy.
fn process_discoveries<R: Rng>(state: &mut GameState, rng: &mut R, result: &mut TickResult) {
    // Try dungeon discovery (only outside dungeons)
    let discovered_dungeon = state.active_dungeon.is_none() && try_discover_dungeon(state);
    if discovered_dungeon {
        result.events.push(TickEvent::DungeonDiscovered {
            message: "\u{1f300} You notice a dark passage leading underground...".to_string(),
        });
    }

    // Try fishing spot discovery (only if no dungeon or fishing active)
    if !discovered_dungeon && state.active_dungeon.is_none() && state.active_fishing.is_none() {
        if let Some(message) = crate::fishing::logic::try_discover_fishing(state, rng) {
            result.events.push(TickEvent::FishingSpotDiscovered {
                message: format!("\u{1f3a3} {}", message),
            });
        }
    }
}

/// Track zone completion achievements based on boss defeat result.
fn process_zone_achievements(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::haven::Haven;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn test_rng() -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(42)
    }

    #[test]
    fn test_game_tick_returns_empty_result_for_idle_state() {
        let mut state = GameState::new("Test".to_string(), 0);
        let mut tick_counter = 0u32;
        let mut haven = Haven::default();
        let mut achievements = Achievements::default();
        let mut rng = test_rng();

        let result = game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut achievements,
            false,
            &mut rng,
        );

        // A fresh state with no enemy should just spawn an enemy
        assert!(state.combat_state.current_enemy.is_some());
        // tick_counter should have incremented
        assert_eq!(tick_counter, 1);
        // No leviathan encounter
        assert!(result.leviathan_encounter.is_none());
    }

    #[test]
    fn test_game_tick_increments_play_time() {
        let mut state = GameState::new("Time Test".to_string(), 0);
        let mut tick_counter = 0u32;
        let mut haven = Haven::default();
        let mut achievements = Achievements::default();
        let mut rng = test_rng();

        let initial_time = state.play_time_seconds;

        for _ in 0..10 {
            game_tick(
                &mut state,
                &mut tick_counter,
                &mut haven,
                &mut achievements,
                false,
                &mut rng,
            );
        }

        assert_eq!(state.play_time_seconds, initial_time + 1);
        assert_eq!(tick_counter, 0);
    }

    #[test]
    fn test_game_tick_spawns_enemy() {
        let mut state = GameState::new("Spawn Test".to_string(), 0);
        let mut tick_counter = 0u32;
        let mut haven = Haven::default();
        let mut achievements = Achievements::default();
        let mut rng = test_rng();

        assert!(state.combat_state.current_enemy.is_none());

        game_tick(
            &mut state,
            &mut tick_counter,
            &mut haven,
            &mut achievements,
            false,
            &mut rng,
        );

        assert!(state.combat_state.current_enemy.is_some());
    }

    #[test]
    fn test_game_tick_combat_produces_events() {
        use crate::character::attributes::AttributeType;

        let mut state = GameState::new("Combat Test".to_string(), 0);
        state.attributes.set(AttributeType::Strength, 50);
        state.attributes.set(AttributeType::Intelligence, 50);
        let derived = DerivedStats::calculate_derived_stats(&state.attributes, &state.equipment);
        state.combat_state.update_max_hp(derived.max_hp);
        state.combat_state.player_current_hp = state.combat_state.player_max_hp;

        let mut tick_counter = 0u32;
        let mut haven = Haven::default();
        let mut achievements = Achievements::default();
        let mut rng = test_rng();

        let mut all_events = Vec::new();
        for _ in 0..5000 {
            let result = game_tick(
                &mut state,
                &mut tick_counter,
                &mut haven,
                &mut achievements,
                false,
                &mut rng,
            );
            all_events.extend(result.events);

            // Stop after first enemy defeated
            if all_events
                .iter()
                .any(|e| matches!(e, TickEvent::EnemyDefeated { .. }))
            {
                break;
            }
        }

        assert!(
            all_events
                .iter()
                .any(|e| matches!(e, TickEvent::EnemyDefeated { .. })),
            "Should have an EnemyDefeated event"
        );
    }
}
