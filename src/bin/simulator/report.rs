use super::stats::{SimStats, TickProfile};
use super::SimConfig;
use quest::core::game_state::GameState;
use quest::core::tick::{TickEvent, TickResult};
use std::collections::HashMap;

pub fn ticks_to_time(ticks: u64) -> String {
    let total_secs = ticks / 10;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    if hours > 0 {
        format!("{hours}h {minutes:02}m {seconds:02}s")
    } else if minutes > 0 {
        format!("{minutes}m {seconds:02}s")
    } else {
        format!("{seconds}s")
    }
}

pub fn print_tick_events(tick: u64, result: &TickResult) {
    for event in &result.events {
        let label = match event {
            TickEvent::PlayerAttack {
                damage, was_crit, ..
            } => {
                if *was_crit {
                    format!("CRIT {damage} damage")
                } else {
                    format!("Hit {damage} damage")
                }
            }
            TickEvent::EnemyAttack {
                damage, enemy_name, ..
            } => {
                format!("{enemy_name} hit for {damage}")
            }
            TickEvent::EnemyDefeated {
                xp_gained,
                enemy_name,
                ..
            } => {
                format!("Killed {enemy_name} (+{xp_gained} XP)")
            }
            TickEvent::PlayerDied => "DIED".to_string(),
            TickEvent::PlayerDiedInDungeon => "DIED (dungeon)".to_string(),
            TickEvent::SubzoneBossDefeated { xp_gained, .. } => {
                format!("Boss defeated (+{xp_gained} XP)")
            }
            TickEvent::ItemDropped {
                item_name,
                rarity,
                equipped,
                ..
            } => {
                let eq = if *equipped { " [EQUIPPED]" } else { "" };
                format!("Item: {} ({:?}){}", item_name, rarity, eq)
            }
            TickEvent::LeveledUp { new_level } => format!("Level up! -> {new_level}"),
            TickEvent::DungeonDiscovered { .. } => "Dungeon discovered!".to_string(),
            TickEvent::DungeonCompleted { xp_earned, .. } => {
                format!("Dungeon completed (+{xp_earned} XP)")
            }
            TickEvent::DungeonFailed => "Dungeon failed".to_string(),
            TickEvent::FishCaught {
                fish_name, rarity, ..
            } => {
                format!("Caught {fish_name} ({rarity:?})")
            }
            TickEvent::FishingRankUp { .. } => "Fishing rank up!".to_string(),
            TickEvent::AchievementUnlocked { name, .. } => format!("Achievement: {name}"),
            TickEvent::HavenDiscovered => "Haven discovered!".to_string(),
            TickEvent::LoomDiscovered => "Loom of Worlds discovered!".to_string(),
            TickEvent::PatternMilestoneReached { message, .. } => {
                format!("Loom: {message}")
            }
            _ => return,
        };
        println!("[t={tick:>6}] {label}");
    }
}

pub fn print_summary(stats: &SimStats, seed: u64, config: &SimConfig) {
    if config.quiet {
        let total_items: u64 = stats.items_by_rarity.iter().sum();
        println!(
            "seed={seed} ticks={} level={} zone={}-{} kills={} deaths={} items={} achievements={} haven_built={} haven_spent={} pr_earned={} pr_spent={} challenges_won={}",
            stats.total_ticks,
            stats.final_level,
            stats.final_zone.0,
            stats.final_zone.1,
            stats.total_kills,
            stats.total_deaths,
            total_items,
            stats.achievements_unlocked,
            stats.haven_rooms_built,
            stats.haven_prestige_spent,
            stats.pr_earned,
            stats.pr_spent,
            stats.challenges_won,
        );
        return;
    }

    let game_time = ticks_to_time(stats.total_ticks);
    println!("============================================================");
    println!("  Quest Simulation Report  (seed={seed})");
    println!("============================================================");
    println!();

    // Duration
    println!("Duration: {} ticks ({game_time})", stats.total_ticks);
    println!("Starting prestige: P{}", config.prestige);
    println!();

    // Final state
    println!("--- Final State ---");
    println!(
        "Level: {}  |  XP: {}  |  Prestige: P{}",
        stats.final_level, stats.final_xp, stats.final_prestige
    );
    println!(
        "Zone: {}-{}  |  Fishing Rank: {}",
        stats.final_zone.0, stats.final_zone.1, stats.final_fishing_rank
    );

    let attr_names = ["STR", "DEX", "CON", "INT", "WIS", "CHA"];
    let attr_str: Vec<String> = attr_names
        .iter()
        .zip(stats.final_attributes.iter())
        .map(|(name, val)| format!("{name}:{val}"))
        .collect();
    println!("Attributes: {}", attr_str.join("  "));
    println!();

    // Combat stats
    println!("--- Combat ---");
    println!(
        "Kills: {}  |  Deaths: {}  |  K/D: {:.1}",
        stats.total_kills,
        stats.total_deaths,
        if stats.total_deaths > 0 {
            stats.total_kills as f64 / stats.total_deaths as f64
        } else {
            stats.total_kills as f64
        }
    );
    println!(
        "Boss kills: {}  |  Crits: {}",
        stats.total_boss_kills, stats.total_crits,
    );
    println!(
        "Retreats: {}  |  Boss enrages: {}  |  Frontier backoffs: {}",
        stats.combat_retreats, stats.boss_enrages, stats.frontier_backoffs,
    );
    println!(
        "Total XP: {}  |  Avg XP/kill: {:.0}",
        stats.total_xp_gained,
        if stats.total_kills > 0 {
            stats.total_xp_gained as f64 / stats.total_kills as f64
        } else {
            0.0
        }
    );
    println!();

    // Items
    println!("--- Items ---");
    let rarity_names = ["Common", "Magic", "Rare", "Epic", "Legendary"];
    let total_items: u64 = stats.items_by_rarity.iter().sum();
    println!(
        "Total drops: {total_items}  |  Equipped: {}",
        stats.items_equipped
    );
    for (i, name) in rarity_names.iter().enumerate() {
        if stats.items_by_rarity[i] > 0 {
            println!("  {name}: {}", stats.items_by_rarity[i]);
        }
    }
    if stats.boss_items_dropped > 0 {
        println!("  From bosses: {}", stats.boss_items_dropped);
    }
    println!();

    // Dungeons
    if stats.dungeons_discovered > 0 || stats.dungeons_completed > 0 || stats.dungeons_failed > 0 {
        println!("--- Dungeons ---");
        println!(
            "Discovered: {}  |  Completed: {}  |  Failed: {}",
            stats.dungeons_discovered, stats.dungeons_completed, stats.dungeons_failed
        );
        println!();
    }

    // Fishing
    if stats.fish_caught > 0 || stats.fishing_rank_ups > 0 {
        println!("--- Fishing ---");
        println!(
            "Fish caught: {}  |  Rank ups: {}  |  Final rank: {}",
            stats.fish_caught, stats.fishing_rank_ups, stats.final_fishing_rank
        );
        println!();
    }

    // Achievements
    if stats.achievements_unlocked > 0 || stats.haven_discovered {
        println!("--- Discoveries ---");
        println!("Achievements: {}", stats.achievements_unlocked);
        if stats.haven_discovered {
            println!("Haven: discovered");
        }
        println!();
    }

    // Haven build progress
    if let Some(ref strat) = config.strategy {
        println!("--- Haven ---");
        println!("Strategy: {}", strat.name());
        println!(
            "Rooms built: {}  |  Prestige spent: {}",
            stats.haven_rooms_built, stats.haven_prestige_spent
        );
        if !stats.haven_final_tiers.is_empty() {
            let tier_strs: Vec<String> = stats
                .haven_final_tiers
                .iter()
                .map(|(name, tier)| format!("{name} T{tier}"))
                .collect();
            println!("Final state: {}", tier_strs.join(", "));
        }
        println!();
    }

    // Deep
    if stats.deep_layers_reached > 0 {
        println!("--- The Deep ---");
        println!(
            "Deepest layer: {}  |  Fracture zone cap: Z{}",
            stats.deep_layers_reached, stats.fracture_zone_cap
        );
        println!();
    }

    // Loom
    if stats.loom_patterns_completed > 0 {
        println!("--- Loom of Worlds ---");
        println!(
            "Patterns completed: {}/28  |  Loom zone cap: Z{}",
            stats.loom_patterns_completed, stats.loom_zone_cap
        );
        println!();
    }

    // Economy Flow
    if stats.pr_earned > 0 || stats.pr_spent > 0 {
        println!("--- Economy Flow ---");
        println!(
            "PR earned: {}  |  PR spent: {}  |  Net: {}",
            stats.pr_earned,
            stats.pr_spent,
            stats.pr_earned as i64 - stats.pr_spent as i64
        );
        if stats.ascension_level > 0 {
            println!("Ascension level: {}", stats.ascension_level);
        }
        if stats.challenges_won > 0 {
            println!("Challenges won: {}", stats.challenges_won);
        }
        if stats.stormglass_balance > 0 {
            println!("Stormglass balance: {}", stats.stormglass_balance);
        }
        println!();
    }

    // Level milestones
    let milestones = [5, 10, 15, 20, 25, 50, 75, 100];
    let reached: Vec<String> = milestones
        .iter()
        .filter_map(|&lvl| {
            stats
                .level_at_tick
                .get(&lvl)
                .map(|t| format!("L{lvl} @ {}", ticks_to_time(*t)))
        })
        .collect();
    if !reached.is_empty() {
        println!("--- Level Milestones ---");
        for m in &reached {
            println!("  {m}");
        }
        println!();
    }

    // Zone progression
    let mut zones: Vec<(&(u32, u32), &u64)> = stats.zone_entry_tick.iter().collect();
    zones.sort_by_key(|&(k, v)| (*v, k.0, k.1));
    if zones.len() > 1 {
        println!("--- Zone Progression ---");
        for ((z, s), t) in &zones {
            println!("  Zone {z}-{s} entered @ {}", ticks_to_time(**t));
        }
        println!();
    }

    // Deaths by zone
    if !stats.deaths_per_zone.is_empty() {
        let mut death_zones: Vec<_> = stats.deaths_per_zone.iter().collect();
        death_zones.sort_by_key(|&(k, _)| (k.0, k.1));
        println!("--- Deaths by Zone ---");
        for ((z, s), count) in &death_zones {
            println!("  Zone {z}-{s}: {count} deaths");
        }
        println!();
    }

    // Combat retreats by destination zone
    if !stats.retreats_per_zone.is_empty() {
        let mut retreat_zones: Vec<_> = stats.retreats_per_zone.iter().collect();
        retreat_zones.sort_by_key(|&(k, _)| (k.0, k.1));
        println!("--- Retreats (by destination zone) ---");
        for ((z, s), count) in &retreat_zones {
            println!("  Zone {z}-{s}: {count} retreats");
        }
        println!();
    }

    // Boss enrages by zone
    if !stats.enrages_per_zone.is_empty() {
        let mut enrage_zones: Vec<_> = stats.enrages_per_zone.iter().collect();
        enrage_zones.sort_by_key(|&(k, _)| (k.0, k.1));
        println!("--- Boss Enrages by Zone ---");
        for ((z, s), count) in &enrage_zones {
            println!("  Zone {z}-{s}: {count} enrages");
        }
        println!();
    }
}

pub fn print_multi_run_summary(all_stats: &[SimStats]) {
    let n = all_stats.len() as f64;
    println!("============================================================");
    println!("  Aggregate Results ({} runs)", all_stats.len());
    println!("============================================================");
    println!();

    // Helper closures
    let avg = |vals: &[u64]| -> f64 { vals.iter().sum::<u64>() as f64 / n };
    let min_max = |vals: &[u64]| -> (u64, u64) {
        (
            *vals.iter().min().unwrap_or(&0),
            *vals.iter().max().unwrap_or(&0),
        )
    };

    let levels: Vec<u64> = all_stats.iter().map(|s| s.final_level as u64).collect();
    let kills: Vec<u64> = all_stats.iter().map(|s| s.total_kills).collect();
    let deaths: Vec<u64> = all_stats.iter().map(|s| s.total_deaths).collect();
    let boss_kills: Vec<u64> = all_stats.iter().map(|s| s.total_boss_kills).collect();
    let items: Vec<u64> = all_stats
        .iter()
        .map(|s| s.items_by_rarity.iter().sum())
        .collect();
    let xp: Vec<u64> = all_stats.iter().map(|s| s.total_xp_gained).collect();
    let achievements: Vec<u64> = all_stats.iter().map(|s| s.achievements_unlocked).collect();

    let (lmin, lmax) = min_max(&levels);
    let (kmin, kmax) = min_max(&kills);
    let (dmin, dmax) = min_max(&deaths);
    let (bmin, bmax) = min_max(&boss_kills);
    let (imin, imax) = min_max(&items);
    let (xmin, xmax) = min_max(&xp);
    let (amin, amax) = min_max(&achievements);

    println!("{:<20} {:>10} {:>10} {:>10}", "Metric", "Min", "Avg", "Max");
    println!("{}", "-".repeat(52));
    println!(
        "{:<20} {:>10} {:>10.1} {:>10}",
        "Final Level",
        lmin,
        avg(&levels),
        lmax
    );
    println!(
        "{:<20} {:>10} {:>10.1} {:>10}",
        "Kills",
        kmin,
        avg(&kills),
        kmax
    );
    println!(
        "{:<20} {:>10} {:>10.1} {:>10}",
        "Deaths",
        dmin,
        avg(&deaths),
        dmax
    );
    println!(
        "{:<20} {:>10} {:>10.1} {:>10}",
        "Boss Kills",
        bmin,
        avg(&boss_kills),
        bmax
    );
    println!(
        "{:<20} {:>10} {:>10.1} {:>10}",
        "Items Found",
        imin,
        avg(&items),
        imax
    );
    println!(
        "{:<20} {:>10} {:>10.1} {:>10}",
        "Total XP",
        xmin,
        avg(&xp),
        xmax
    );
    println!(
        "{:<20} {:>10} {:>10.1} {:>10}",
        "Achievements",
        amin,
        avg(&achievements),
        amax
    );

    let haven_built: Vec<u64> = all_stats
        .iter()
        .map(|s| s.haven_rooms_built as u64)
        .collect();
    let haven_spent: Vec<u64> = all_stats
        .iter()
        .map(|s| s.haven_prestige_spent as u64)
        .collect();
    if haven_built.iter().any(|&v| v > 0) {
        let (hbmin, hbmax) = min_max(&haven_built);
        let (hsmin, hsmax) = min_max(&haven_spent);
        println!(
            "{:<20} {:>10} {:>10.1} {:>10}",
            "Haven Builds",
            hbmin,
            avg(&haven_built),
            hbmax
        );
        println!(
            "{:<20} {:>10} {:>10.1} {:>10}",
            "Haven PR Spent",
            hsmin,
            avg(&haven_spent),
            hsmax
        );
    }

    let pr_earned: Vec<u64> = all_stats.iter().map(|s| s.pr_earned).collect();
    let pr_spent: Vec<u64> = all_stats.iter().map(|s| s.pr_spent).collect();
    let challenges_won: Vec<u64> = all_stats.iter().map(|s| s.challenges_won).collect();
    if pr_earned.iter().any(|&v| v > 0) || pr_spent.iter().any(|&v| v > 0) {
        let (pemin, pemax) = min_max(&pr_earned);
        let (psmin, psmax) = min_max(&pr_spent);
        let (cwmin, cwmax) = min_max(&challenges_won);
        println!(
            "{:<20} {:>10} {:>10.1} {:>10}",
            "PR Earned",
            pemin,
            avg(&pr_earned),
            pemax
        );
        println!(
            "{:<20} {:>10} {:>10.1} {:>10}",
            "PR Spent",
            psmin,
            avg(&pr_spent),
            psmax
        );
        println!(
            "{:<20} {:>10} {:>10.1} {:>10}",
            "Challenges Won",
            cwmin,
            avg(&challenges_won),
            cwmax
        );
    }
    println!();

    // Final zone distribution
    let mut zone_counts: HashMap<(u32, u32), u32> = HashMap::new();
    for s in all_stats {
        *zone_counts.entry(s.final_zone).or_insert(0) += 1;
    }
    let mut zone_dist: Vec<_> = zone_counts.iter().collect();
    zone_dist.sort_by_key(|&(k, _)| (k.0, k.1));
    println!("Final zone distribution:");
    for ((z, s), count) in &zone_dist {
        println!("  Zone {z}-{s}: {count} runs");
    }
    println!();
}

pub fn print_profile(profile: &TickProfile, config: &SimConfig) {
    println!();
    println!(
        "=== Tick Profile ({} ticks, P{}) ===",
        config.ticks, config.prestige
    );
    println!("{:<25} {:>10}", "Metric", "Value");
    println!("{}", "\u{2500}".repeat(37));
    println!(
        "{:<25} {:>10.1} \u{00b5}s",
        "Avg per tick",
        profile.avg_us()
    );
    println!(
        "{:<25} {:>10.1} \u{00b5}s",
        "Min per tick",
        profile.min_us()
    );
    println!(
        "{:<25} {:>10.1} \u{00b5}s",
        "Max per tick",
        profile.max_us()
    );
    let total_s = profile.total_ns as f64 / 1_000_000_000.0;
    println!("{:<25} {:>10.3} s", "Total wall time", total_s);
    if total_s > 0.0 {
        println!(
            "{:<25} {:>10.0}",
            "Ticks/second",
            profile.tick_count as f64 / total_s
        );
    }
}

pub fn print_final_equipment(state: &GameState) {
    println!("--- Final Equipment ---");
    let slots = [
        ("Weapon", &state.equipment.weapon),
        ("Armor", &state.equipment.armor),
        ("Helmet", &state.equipment.helmet),
        ("Gloves", &state.equipment.gloves),
        ("Boots", &state.equipment.boots),
        ("Amulet", &state.equipment.amulet),
        ("Ring", &state.equipment.ring),
    ];
    for (name, item) in &slots {
        match item {
            Some(i) => println!("  {name}: {} ({:?})", i.display_name, i.rarity),
            None => println!("  {name}: (empty)"),
        }
    }
    println!();
}
