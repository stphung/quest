//! Maps [`TickEvent`]s to combat log entries and visual effects.
//!
//! This is a binary-only module (not part of `lib.rs`) because it bridges
//! pure game-logic events from [`core::tick`] to UI types like
//! [`VisualEffect`] and [`EffectType`].

use crate::ascension::ascension_combat_multiplier;
use crate::combat::types::{DamageFlash, DAMAGE_FLASH_DURATION};
use crate::core::game_state::{GameState, TickerEntry, TickerSegment};
use crate::core::tick::TickEvent;
use crate::items::types::Rarity;
use crate::ui::combat_effects::{EffectType, VisualEffect};
use crate::ui::rarity_color;
use crate::ui::stats_prestige::to_roman;
use crate::zones::BossDefeatResult;
use ratatui::style::Color;

/// Flags returned from apply_tick_events indicating which discovery overlays to show.
pub struct TickEventFlags {
    pub haven_discovered: bool,
    pub soulforge_discovered: bool,
    pub stormglass_discovered: bool,
    pub deep_discovered: bool,
    pub loom_discovered: bool,
    pub fracture_region_unlocked: Option<crate::zones::FractureRegion>,
}

/// Maps tick events to combat log entries and visual effects.
/// Returns flags indicating which discovery events were present.
pub fn apply_tick_events(game_state: &mut GameState, events: &[TickEvent]) -> TickEventFlags {
    let mut haven_discovered = false;
    let mut soulforge_discovered = false;
    let mut stormglass_discovered = false;
    let mut deep_discovered = false;
    let mut loom_discovered = false;
    let mut fracture_region_unlocked = None;
    for event in events {
        match event {
            TickEvent::PlayerAttack { damage, was_crit } => {
                let message = if *was_crit {
                    format!("\u{1f4a5} CRITICAL HIT for {} damage!", damage)
                } else {
                    format!("\u{2694} You hit for {} damage", damage)
                };
                game_state
                    .combat_state
                    .add_log_entry(message, *was_crit, true);

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
            TickEvent::PlayerAttackBlocked { weapon_needed } => {
                let message = format!("\u{1f6ab} {} required to damage this foe!", weapon_needed);
                game_state.combat_state.add_log_entry(message, false, true);
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
            TickEvent::EnemyAttack { damage, enemy_name } => {
                let message = format!("\u{1f6e1} {} hits you for {} damage", enemy_name, damage);
                game_state.combat_state.add_log_entry(message, false, false);
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
            TickEvent::DamageReflected { damage } => {
                let message = format!("\u{1f4a5} {} reflected!", damage);
                game_state.combat_state.add_log_entry(message, false, true);
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
                xp_gained,
                enemy_name,
            } => {
                let message = format!("\u{2728} {} defeated! +{} XP", enemy_name, xp_gained);
                game_state.combat_state.add_log_entry(message, false, true);
            }
            TickEvent::BossEnrage { message } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, false);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{1f525}",
                    text: "Boss Enraged!".to_string(),
                    color: Color::Red,
                    bold: true,
                    segments: None,
                });
            }
            TickEvent::PlayerDied => {
                let message = "\u{1f480} You died! Boss encounter reset.".to_string();
                game_state.combat_state.add_log_entry(message, false, false);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{2620}",
                    text: "Slain!".to_string(),
                    color: Color::Red,
                    bold: true,
                    segments: None,
                });
            }
            TickEvent::PlayerDiedInDungeon => {
                let message =
                    "\u{1f480} You fell in the dungeon... (escaped without prestige loss)"
                        .to_string();
                game_state.combat_state.add_log_entry(message, false, false);
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
                power,
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
                        text: format!(" Z{}", ilvl / 10),
                        color: Color::DarkGray,
                    },
                    TickerSegment {
                        text: format!(" \u{26A1}{}", power),
                        color: Color::Cyan,
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
            TickEvent::SubzoneBossDefeated { xp_gained, result } => {
                // Build message from BossDefeatResult
                let message = match result {
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
                        // Should not reach here — filtered at event creation
                        String::new()
                    }
                    BossDefeatResult::ExpanseCycle => {
                        format!(
                            "\u{1f451} The Endless defeated! +{} XP \u{2014} The Expanse cycles anew...",
                            xp_gained
                        )
                    }
                    BossDefeatResult::FractureCycle { zone_id } => {
                        let zone_name = crate::zones::get_zone(*zone_id)
                            .map(|z| z.name)
                            .unwrap_or("Unknown");
                        format!(
                            "\u{1f451} {} conquered! +{} XP \u{2014} Zone cycles anew...",
                            zone_name, xp_gained
                        )
                    }
                };
                game_state.combat_state.add_log_entry(message, false, true);
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
                    BossDefeatResult::ZoneCompleteButGated {
                        zone_name,
                        required_prestige,
                    } => {
                        game_state.ticker.push(TickerEntry {
                            icon: "\u{1F512}",
                            text: format!("{} cleared! P{} needed", zone_name, required_prestige),
                            color: Color::DarkGray,
                            bold: false,
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
            TickEvent::DungeonRoomEntered { message, .. } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
            }
            TickEvent::DungeonBossUnlocked => {
                let message = "\u{1f479} Somewhere deep in the dungeon, a sealed door grinds open."
                    .to_string();
                game_state.combat_state.add_log_entry(message, false, true);
            }
            TickEvent::DungeonTreasureFound {
                item_name,
                rarity,
                tier,
                ilvl,
                power,
                equipped,
            } => {
                let status = if *equipped {
                    "Equipped!"
                } else {
                    "Kept current gear"
                };
                let message = format!("\u{1f48e} Found: {} [{}]", item_name, status);
                game_state.combat_state.add_log_entry(message, false, true);
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
                        text: format!(" Z{}", ilvl / 10),
                        color: Color::DarkGray,
                    },
                    TickerSegment {
                        text: format!(" \u{26A1}{}", power),
                        color: Color::Cyan,
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
            TickEvent::DungeonKeyFound => {
                let message = "\u{1f5dd}\u{fe0f} A heavy key clatters to the ground. The way forward is open.".to_string();
                game_state.combat_state.add_log_entry(message, false, true);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{1F5DD}",
                    text: "Key found!".to_string(),
                    color: Color::Yellow,
                    bold: false,
                    segments: None,
                });
            }
            TickEvent::DungeonBossDefeated {
                bonus_xp,
                total_xp,
                items_collected,
                ..
            } => {
                let message = format!(
                    "\u{1f3c6} Dungeon Complete! +{} bonus XP ({} total, {} items)",
                    bonus_xp, total_xp, items_collected
                );
                game_state.combat_state.add_log_entry(message, false, true);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{1F451}",
                    text: "Dungeon Boss!".to_string(),
                    color: Color::Magenta,
                    bold: true,
                    segments: None,
                });
            }
            TickEvent::DungeonEliteDefeated {
                xp_gained,
                enemy_name,
            } => {
                let message = format!(
                    "\u{2694}\u{fe0f} {} defeated! +{} XP",
                    enemy_name, xp_gained
                );
                game_state.combat_state.add_log_entry(message, false, true);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{2694}",
                    text: "Elite!".to_string(),
                    color: Color::Magenta,
                    bold: false,
                    segments: None,
                });
            }
            TickEvent::DungeonCompleted {
                xp_earned,
                items_collected,
            } => {
                let message = format!(
                    "\u{1f3c6} Dungeon Complete! +{} XP, {} items found",
                    xp_earned, items_collected
                );
                game_state.combat_state.add_log_entry(message, false, true);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{1F3F0}",
                    text: "Dungeon Complete!".to_string(),
                    color: Color::Magenta,
                    bold: true,
                    segments: None,
                });
            }
            TickEvent::DungeonFailed => {
                let message =
                    "\u{1f480} The dungeon spits you out, broken but alive. No prestige lost."
                        .to_string();
                game_state.combat_state.add_log_entry(message, false, false);
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
            TickEvent::AchievementUnlocked { name } => {
                let message = format!("\u{1f3c6} Achievement Unlocked: {}", name);
                game_state.combat_state.add_log_entry(message, false, true);
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
            TickEvent::StormglassDiscovered => {
                stormglass_discovered = true;
            }
            TickEvent::DeepDiscovered => {
                deep_discovered = true;
                game_state.combat_state.add_log_entry(
                    "\u{2693} A mercenary captain has found you. The Deep awaits...".to_string(),
                    false,
                    true,
                );
            }
            TickEvent::DeepMissionComplete { message } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{1F4DC}",
                    text: "Mission Complete!".to_string(),
                    color: Color::Cyan,
                    bold: true,
                    segments: None,
                });
            }
            TickEvent::DeepEventPending { message } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{26A1}",
                    text: "Deep Event!".to_string(),
                    color: Color::Yellow,
                    bold: true,
                    segments: None,
                });
            }
            TickEvent::DeepMercInjured { message } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, false);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{1FA79}",
                    text: "Merc Injured".to_string(),
                    color: Color::Yellow,
                    bold: false,
                    segments: None,
                });
            }
            TickEvent::DeepMercLost { message } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, false);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{1FAA6}",
                    text: "Merc Lost!".to_string(),
                    color: Color::Red,
                    bold: true,
                    segments: None,
                });
            }
            TickEvent::DeepBreakthrough { message } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{2B50}",
                    text: "Breakthrough!".to_string(),
                    color: Color::Magenta,
                    bold: true,
                    segments: None,
                });
            }
            TickEvent::DeepGuildRankUp { message } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{2694}",
                    text: "Guild Rank Up!".to_string(),
                    color: Color::Cyan,
                    bold: true,
                    segments: None,
                });
            }
            TickEvent::StormglassSalvaged {
                item_name, amount, ..
            } => {
                let sg_color = Color::Rgb(100, 180, 255);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{1F48E}",
                    text: format!("Salvaged {} \u{2192} +{} SG", item_name, amount),
                    color: sg_color,
                    bold: false,
                    segments: None,
                });
            }
            TickEvent::StormglassDungeonCache { amount } => {
                let sg_color = Color::Rgb(100, 180, 255);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{1F48E}",
                    text: format!("Found {} Stormglass!", amount),
                    color: sg_color,
                    bold: true,
                    segments: None,
                });
            }
            TickEvent::FractureRegionUnlocked { region, message } => {
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, true);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{1F30B}",
                    text: region.unlock_ticker_text().to_string(),
                    color: Color::Magenta,
                    bold: true,
                    segments: None,
                });
                fracture_region_unlocked = Some(*region);
            }
            TickEvent::Ascended { level } => {
                let roman = to_roman(*level);
                let multiplier = ascension_combat_multiplier(*level);
                let message = format!(
                    "\u{2728} Ascended to Ascension {}! ({:.1}x multiplier)",
                    roman, multiplier
                );
                game_state.combat_state.add_log_entry(message, false, true);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{2728}",
                    text: format!("Ascension {}!", level),
                    color: Color::Yellow,
                    bold: true,
                    segments: None,
                });
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
            TickEvent::CombatRetreat { zone_name } => {
                let message = format!("\u{1f3c3} Overwhelmed! You retreat to {}...", zone_name);
                game_state
                    .combat_state
                    .add_log_entry(message.clone(), false, false);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{1f3c3}",
                    text: message,
                    color: Color::Yellow,
                    bold: true,
                    segments: None,
                });
            }
            TickEvent::PowerCoreGranted { core_name } => {
                let message = format!("\u{26A1} Power Core [{}] granted +1 PR", core_name);
                game_state.combat_state.add_log_entry(message, false, true);
                game_state.ticker.push(TickerEntry {
                    icon: "\u{26A1}",
                    text: format!("{} Core!", core_name),
                    color: Color::Rgb(255, 165, 0),
                    bold: true,
                    segments: None,
                });
            }
            TickEvent::LoomDiscovered => {
                loom_discovered = true;
                game_state.combat_state.add_log_entry(
                    "\u{29BF} The Gateway opens. Beyond it, threads of reality stretch into the void. The Loom of Worlds awaits...".to_string(),
                    false,
                    true,
                );
            }
            TickEvent::WovenRealityPRGranted { pr_amount, wr_rate } => {
                game_state.combat_state.add_log_entry(
                    format!(
                        "\u{29BF} Woven Reality grants +{} Prestige Rank{} ({:.1} WR/hr)",
                        pr_amount,
                        if *pr_amount == 1 { "" } else { "s" },
                        wr_rate,
                    ),
                    false,
                    true,
                );
            }
        }
    }
    TickEventFlags {
        haven_discovered,
        soulforge_discovered,
        stormglass_discovered,
        deep_discovered,
        loom_discovered,
        fracture_region_unlocked,
    }
}
