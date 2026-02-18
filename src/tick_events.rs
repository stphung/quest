//! Maps [`TickEvent`]s to combat log entries and visual effects.
//!
//! This is a binary-only module (not part of `lib.rs`) because it bridges
//! pure game-logic events from [`core::tick`] to UI types like
//! [`VisualEffect`] and [`EffectType`].

use crate::combat::types::{DamageFlash, DAMAGE_FLASH_DURATION};
use crate::core::game_state::{GameState, TickerEntry, TickerSegment};
use crate::core::tick::TickEvent;
use crate::items::types::Rarity;
use crate::ui::combat_effects::{EffectType, VisualEffect};
use crate::ui::rarity_color;
use crate::zones::BossDefeatResult;
use ratatui::style::Color;

/// Flags returned from apply_tick_events indicating which discovery overlays to show.
pub struct TickEventFlags {
    pub haven_discovered: bool,
    pub soulforge_discovered: bool,
}

/// Maps tick events to combat log entries and visual effects.
/// Returns flags indicating which discovery events were present.
pub fn apply_tick_events(game_state: &mut GameState, events: &[TickEvent]) -> TickEventFlags {
    let mut haven_discovered = false;
    let mut soulforge_discovered = false;
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

                let (text, color, bold) = if *was_crit {
                    (format!("\u{2605}{}\u{2605}", damage), Color::Yellow, true)
                } else {
                    (format!("-{}", damage), Color::Green, false)
                };
                game_state
                    .combat_state
                    .enemy_damage_floats
                    .push(DamageFlash {
                        text,
                        color,
                        bold,
                        remaining: DAMAGE_FLASH_DURATION,
                    });

                // Keep attack flash and hit impact effects
                let flash_effect = VisualEffect::new(EffectType::AttackFlash, 0.2);
                game_state.combat_state.visual_effects.push(flash_effect);
                let impact_effect = VisualEffect::new(EffectType::HitImpact, 0.3);
                game_state.combat_state.visual_effects.push(impact_effect);
            }
            TickEvent::PlayerAttackBlocked { message, .. } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
                game_state
                    .combat_state
                    .enemy_damage_floats
                    .push(DamageFlash {
                        text: "BLOCK".to_string(),
                        color: Color::DarkGray,
                        bold: false,
                        remaining: DAMAGE_FLASH_DURATION,
                    });
            }
            TickEvent::EnemyAttack {
                damage, message, ..
            } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, false);
                game_state
                    .combat_state
                    .player_damage_floats
                    .push(DamageFlash {
                        text: format!("-{}", damage),
                        color: Color::Red,
                        bold: false,
                        remaining: DAMAGE_FLASH_DURATION,
                    });
            }
            TickEvent::DamageReflected { damage, message } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
                game_state
                    .combat_state
                    .enemy_damage_floats
                    .push(DamageFlash {
                        text: format!("\u{1f4a5}{}", damage),
                        color: Color::Magenta,
                        bold: false,
                        remaining: DAMAGE_FLASH_DURATION,
                    });
            }
            TickEvent::RegenComplete { healed } => {
                game_state
                    .combat_state
                    .player_damage_floats
                    .push(DamageFlash {
                        text: format!("+{}", healed),
                        color: Color::Green,
                        bold: false,
                        remaining: DAMAGE_FLASH_DURATION,
                    });
            }
            TickEvent::EnemyDefeated {
                xp_gained, message, ..
            } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{2728}",
                    text: format!("+{} XP", xp_gained),
                    color: Color::Green,
                    bold: false,
                    segments: None,
                });
            }
            TickEvent::PlayerDied { message } | TickEvent::PlayerDiedInDungeon { message } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, false);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{2620}",
                    text: "Slain!".to_string(),
                    color: Color::Red,
                    bold: true,
                    segments: None,
                });
            }
            TickEvent::ItemDropped {
                item_name,
                rarity,
                tier,
                ilvl,
                equipped,
                slot: _,
                stats: _,
                from_boss: _,
            } => {
                let rc = rarity_color(*rarity);
                let tc = crate::ui::tier_color(*tier);
                let mut segs = vec![
                    TickerSegment {
                        text: format!("{} ", rarity.name()),
                        color: rc,
                    },
                    TickerSegment {
                        text: format!("T{}", tier),
                        color: tc,
                    },
                    TickerSegment {
                        text: format!(" {}", item_name),
                        color: rc,
                    },
                    TickerSegment {
                        text: format!(" i{}", ilvl),
                        color: Color::DarkGray,
                    },
                ];
                if *equipped {
                    segs.push(TickerSegment {
                        text: " \u{1F528}".to_string(),
                        color: rc,
                    });
                }
                game_state.ticker.push(TickerEntry {
                    icon: "\u{2694}",
                    text: String::new(),
                    color: rc,
                    bold: *tier >= 7
                        || matches!(rarity, Rarity::Epic | Rarity::Legendary | Rarity::Mythic),
                    segments: Some(segs),
                });
            }
            TickEvent::SubzoneBossDefeated {
                xp_gained,
                message,
                result,
            } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{1F451}",
                    text: format!("Boss +{} XP", xp_gained),
                    color: Color::Yellow,
                    bold: true,
                    segments: None,
                });
                // Push zone advancement to ticker
                match result {
                    BossDefeatResult::SubzoneComplete { .. } => {
                        game_state.ticker.push(TickerEntry {
                            icon: "\u{1F5FA}",
                            text: "New Area!".to_string(),
                            color: Color::Cyan,
                            bold: false,
                            segments: None,
                        });
                    }
                    BossDefeatResult::ZoneComplete {
                        old_zone: _,
                        new_zone_id,
                    } => {
                        let zone_name = crate::zones::get_zone(*new_zone_id)
                            .map(|z| z.name)
                            .unwrap_or("???");
                        game_state.ticker.push(TickerEntry {
                            icon: "\u{1F5FA}",
                            text: format!("Zone: {}!", zone_name),
                            color: Color::Cyan,
                            bold: true,
                            segments: None,
                        });
                    }
                    BossDefeatResult::ZoneCompleteButGated { zone_name, .. } => {
                        game_state.ticker.push(TickerEntry {
                            icon: "\u{1F5FA}",
                            text: format!("{} Conquered!", zone_name),
                            color: Color::Cyan,
                            bold: true,
                            segments: None,
                        });
                    }
                    BossDefeatResult::StormsEnd => {
                        game_state.ticker.push(TickerEntry {
                            icon: "\u{1F5FA}",
                            text: "All Zones Conquered!".to_string(),
                            color: Color::Yellow,
                            bold: true,
                            segments: None,
                        });
                    }
                    BossDefeatResult::ExpanseCycle => {
                        game_state.ticker.push(TickerEntry {
                            icon: "\u{1F5FA}",
                            text: "Expanse Cycles!".to_string(),
                            color: Color::Cyan,
                            bold: false,
                            segments: None,
                        });
                    }
                    _ => {} // WeaponRequired doesn't go to ticker
                }
            }
            TickEvent::DungeonRoomEntered { message, .. }
            | TickEvent::DungeonBossUnlocked { message } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
            }
            TickEvent::DungeonTreasureFound {
                item_name,
                rarity,
                tier,
                ilvl,
                equipped,
                message,
            } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
                let rc = rarity_color(*rarity);
                let tc = crate::ui::tier_color(*tier);
                let mut segs = vec![
                    TickerSegment {
                        text: format!("{} ", rarity.name()),
                        color: rc,
                    },
                    TickerSegment {
                        text: format!("T{}", tier),
                        color: tc,
                    },
                    TickerSegment {
                        text: format!(" {}", item_name),
                        color: rc,
                    },
                    TickerSegment {
                        text: format!(" i{}", ilvl),
                        color: Color::DarkGray,
                    },
                ];
                if *equipped {
                    segs.push(TickerSegment {
                        text: " \u{1F528}".to_string(),
                        color: rc,
                    });
                }
                game_state.ticker.push(TickerEntry {
                    icon: "\u{1F48E}",
                    text: String::new(),
                    color: rc,
                    bold: *tier >= 7
                        || matches!(rarity, Rarity::Epic | Rarity::Legendary | Rarity::Mythic),
                    segments: Some(segs),
                });
            }
            TickEvent::DungeonKeyFound { message } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{1F5DD}",
                    text: "Key found!".to_string(),
                    color: Color::Yellow,
                    bold: false,
                    segments: None,
                });
            }
            TickEvent::DungeonBossDefeated { message, .. } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{1F451}",
                    text: "Dungeon Boss!".to_string(),
                    color: Color::Magenta,
                    bold: true,
                    segments: None,
                });
            }
            TickEvent::DungeonEliteDefeated { message, .. } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{2694}",
                    text: "Elite!".to_string(),
                    color: Color::Magenta,
                    bold: false,
                    segments: None,
                });
            }
            TickEvent::DungeonCompleted { message, .. } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{1F3F0}",
                    text: "Dungeon Complete!".to_string(),
                    color: Color::Magenta,
                    bold: true,
                    segments: None,
                });
            }
            TickEvent::DungeonFailed { message } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, false);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{1F480}",
                    text: "Dungeon failed".to_string(),
                    color: Color::Red,
                    bold: false,
                    segments: None,
                });
            }
            TickEvent::FishingMessage { message } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
            }
            TickEvent::FishingItemFound { item_name, message } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{1F41F}",
                    text: item_name.clone(),
                    color: Color::Cyan,
                    bold: false,
                    segments: None,
                });
            }
            TickEvent::FishingRankUp { message } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{1F3A3}",
                    text: "Rank Up!".to_string(),
                    color: Color::Cyan,
                    bold: true,
                    segments: None,
                });
            }
            TickEvent::FishCaught {
                fish_name,
                rarity,
                message,
            } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
                let text = format!("{} {}", fish_name, rarity.name());
                let color = rarity_color(*rarity);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{1F41F}",
                    text,
                    color,
                    bold: false,
                    segments: None,
                });
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
                game_state.ticker.push(TickerEntry {
                    icon: "\u{1F3B2}",
                    text: "New Challenge!".to_string(),
                    color: Color::Yellow,
                    bold: true,
                    segments: None,
                });
            }
            TickEvent::DungeonDiscovered { message } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{1F3F0}",
                    text: "Dungeon Found!".to_string(),
                    color: Color::Magenta,
                    bold: false,
                    segments: None,
                });
            }
            TickEvent::FishingSpotDiscovered { message } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{1F41F}",
                    text: "Fishing Spot Found!".to_string(),
                    color: Color::Cyan,
                    bold: false,
                    segments: None,
                });
            }
            TickEvent::AchievementUnlocked { name, message } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{1F3C6}",
                    text: name.clone(),
                    color: Color::Yellow,
                    bold: true,
                    segments: None,
                });
            }
            TickEvent::HavenDiscovered => {
                haven_discovered = true;
            }
            TickEvent::SoulforgeDiscovered => {
                soulforge_discovered = true;
            }
            TickEvent::LeveledUp { new_level } => {
                game_state.ticker.push(TickerEntry {
                    icon: "\u{2B06}",
                    text: format!("Level {}!", new_level),
                    color: Color::Green,
                    bold: true,
                    segments: None,
                });
            }
        }
    }
    TickEventFlags {
        haven_discovered,
        soulforge_discovered,
    }
}
