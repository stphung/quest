//! Maps [`TickEvent`]s to combat log entries and visual effects.
//!
//! This is a binary-only module (not part of `lib.rs`) because it bridges
//! pure game-logic events from [`core::tick`] to UI types like
//! [`VisualEffect`] and [`EffectType`].

use crate::core::game_state::GameState;
use crate::core::tick::TickEvent;
use crate::ui::combat_effects::{EffectType, VisualEffect};

/// Maps tick events to combat log entries and visual effects.
/// Returns true if the HavenDiscovered event was present.
pub fn apply_tick_events(game_state: &mut GameState, events: &[TickEvent]) -> bool {
    let mut haven_discovered = false;
    for event in events {
        match event {
            TickEvent::PlayerAttack {
                damage,
                was_crit,
                message,
            } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), *was_crit, true);

                // Spawn damage number effect
                let damage_effect = VisualEffect::new(
                    EffectType::DamageNumber {
                        value: *damage,
                        is_crit: *was_crit,
                    },
                    0.8,
                );
                game_state.combat_state.visual_effects.push(damage_effect);

                // Spawn attack flash
                let flash_effect = VisualEffect::new(EffectType::AttackFlash, 0.2);
                game_state.combat_state.visual_effects.push(flash_effect);

                // Spawn impact effect
                let impact_effect = VisualEffect::new(EffectType::HitImpact, 0.3);
                game_state.combat_state.visual_effects.push(impact_effect);
            }
            TickEvent::PlayerAttackBlocked { message, .. } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
            }
            TickEvent::EnemyAttack { message, .. } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, false);
            }
            TickEvent::EnemyDefeated { message, .. } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
            }
            TickEvent::PlayerDied { message } | TickEvent::PlayerDiedInDungeon { message } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, false);
            }
            TickEvent::ItemDropped { .. } => {
                // Item drops and recent_drops tracking are handled inside game_tick
            }
            TickEvent::SubzoneBossDefeated { message, .. } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
            }
            TickEvent::DungeonRoomEntered { message, .. }
            | TickEvent::DungeonTreasureFound { message, .. }
            | TickEvent::DungeonKeyFound { message }
            | TickEvent::DungeonBossUnlocked { message }
            | TickEvent::DungeonBossDefeated { message, .. }
            | TickEvent::DungeonEliteDefeated { message, .. }
            | TickEvent::DungeonCompleted { message, .. } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
            }
            TickEvent::DungeonFailed { message } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, false);
            }
            TickEvent::FishingMessage { message }
            | TickEvent::FishCaught { message, .. }
            | TickEvent::FishingItemFound { message, .. }
            | TickEvent::FishingRankUp { message } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
            }
            TickEvent::StormLeviathanCaught => {
                // Achievement persistence handled by achievements_changed flag at call site
            }
            TickEvent::ChallengeDiscovered {
                message, follow_up, ..
            } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
                game_state
                    .combat_state
                    .add_log_entry(follow_up.clone(), false, true);
            }
            TickEvent::DungeonDiscovered { message }
            | TickEvent::FishingSpotDiscovered { message } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
            }
            TickEvent::AchievementUnlocked { message, .. } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
            }
            TickEvent::HavenDiscovered => {
                haven_discovered = true;
            }
            TickEvent::LeveledUp { .. } => {
                // Level-up state changes are handled inside game_tick
            }
        }
    }
    haven_discovered
}
