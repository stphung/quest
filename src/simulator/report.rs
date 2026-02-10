//! Simulation report generation.

use super::progression_sim::{PrestigeCycle, RunStats};
use std::collections::HashMap;

/// Format a number with thousand separators.
fn format_with_commas(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Convert ticks to human-readable time string.
/// 10 ticks = 1 second.
pub fn ticks_to_time_str(ticks: u64) -> String {
    let total_seconds = ticks / 10;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("~{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("~{}m {}s", minutes, seconds)
    } else {
        format!("~{}s", seconds)
    }
}

/// Convert ticks to seconds (as float for precision).
fn ticks_to_seconds(ticks: u64) -> f64 {
    ticks as f64 / 10.0
}

/// Calculate theoretical prestige speedup multiplier.
fn theoretical_prestige_mult(rank: u32) -> f64 {
    1.0 + 0.5 * (rank as f64).powf(0.7)
}

/// Aggregated results from multiple simulation runs.
#[derive(Debug, Clone)]
pub struct SimReport {
    pub num_runs: u32,
    pub runs_completed: u32,
    pub runs_timed_out: u32,

    // Aggregated stats
    pub avg_final_level: f64,
    pub avg_final_zone: f64,
    pub avg_total_kills: f64,
    pub avg_total_deaths: f64,
    pub avg_ticks_to_complete: f64,

    // Distribution data
    pub level_distribution: HashMap<u32, u32>,
    pub zone_distribution: HashMap<u32, u32>,
    pub death_distribution: Vec<u64>,

    // Loot analysis
    pub avg_legendary_drops: f64,
    pub avg_upgrades_equipped: f64,
    pub avg_final_ilvl: f64,
    pub actual_drop_rate: f64,

    // Per-zone analysis
    pub avg_deaths_per_zone: Vec<f64>,
    pub avg_kills_per_zone: Vec<f64>,
    pub avg_ticks_per_zone: Vec<f64>,

    // Individual run stats for detailed analysis
    pub run_stats: Vec<RunStats>,

    // Level pacing analysis
    pub level_pacing: LevelPacingAnalysis,

    // Difficulty wall detection
    pub difficulty_walls: Vec<DifficultyWall>,

    // Prestige progression summary
    pub prestige_summary: Vec<PrestigeCycleSummary>,

    // Flag to show detailed level curve
    pub show_level_curve: bool,

    // Combat timing analysis
    pub avg_fight_duration_ticks: f64,
    pub avg_combat_percent: f64,

    // Prestige wall analysis
    pub avg_wall_percent: f64,

    // XP source breakdown
    pub xp_from_kills_percent: f64,
    pub xp_from_passive_percent: f64,

    // Death breakdown
    pub avg_boss_deaths: f64,
    pub avg_regular_deaths: f64,
}

/// Analysis of level-up pacing.
#[derive(Debug, Clone, Default)]
pub struct LevelPacingAnalysis {
    /// Average ticks per level for ranges: 1-10, 11-25, 26-50, 51-100
    pub avg_ticks_per_range: Vec<(String, f64)>,
    /// Level where significant slowdown occurs (>2x slower than previous range)
    pub slowdown_level: Option<u32>,
    /// Slowdown multiplier
    pub slowdown_multiplier: f64,
}

/// Represents a difficulty wall between zones.
#[derive(Debug, Clone)]
pub struct DifficultyWall {
    pub from_zone: u32,
    pub to_zone: u32,
    pub from_death_rate: f64,
    pub to_death_rate: f64,
}

/// Summary of a prestige cycle across all runs.
#[derive(Debug, Clone, Default)]
pub struct PrestigeCycleSummary {
    pub rank: u32,
    pub avg_ticks: f64,
    pub avg_deaths: f64,
    pub improvement_pct: f64, // Improvement over previous cycle
}

impl SimReport {
    /// Create a new report from completed run stats.
    pub fn from_runs(runs: Vec<RunStats>, _target_zone: usize, max_ticks: u64) -> Self {
        let num_runs = runs.len() as u32;
        let runs_completed = runs.iter().filter(|r| r.reached_target).count() as u32;
        let runs_timed_out = runs.iter().filter(|r| r.total_ticks >= max_ticks).count() as u32;

        // Calculate averages
        let avg_final_level =
            runs.iter().map(|r| r.final_level as f64).sum::<f64>() / num_runs as f64;
        let avg_final_zone =
            runs.iter().map(|r| r.final_zone as f64).sum::<f64>() / num_runs as f64;
        let avg_total_kills =
            runs.iter().map(|r| r.total_kills as f64).sum::<f64>() / num_runs as f64;
        let avg_total_deaths =
            runs.iter().map(|r| r.total_deaths as f64).sum::<f64>() / num_runs as f64;
        let avg_ticks_to_complete = runs
            .iter()
            .filter(|r| r.reached_target)
            .map(|r| r.total_ticks as f64)
            .sum::<f64>()
            / runs_completed.max(1) as f64;

        // Level distribution
        let mut level_distribution = HashMap::new();
        for run in &runs {
            *level_distribution.entry(run.final_level).or_insert(0) += 1;
        }

        // Zone distribution
        let mut zone_distribution = HashMap::new();
        for run in &runs {
            *zone_distribution.entry(run.final_zone).or_insert(0) += 1;
        }

        // Death distribution
        let death_distribution: Vec<u64> = runs.iter().map(|r| r.total_deaths).collect();

        // Loot stats
        let avg_legendary_drops = runs
            .iter()
            .map(|r| r.loot_stats.legendary_drops as f64)
            .sum::<f64>()
            / num_runs as f64;
        let avg_upgrades_equipped = runs
            .iter()
            .map(|r| r.loot_stats.upgrades_equipped as f64)
            .sum::<f64>()
            / num_runs as f64;
        let avg_final_ilvl = runs.iter().map(|r| r.final_avg_ilvl).sum::<f64>() / num_runs as f64;
        let actual_drop_rate =
            runs.iter().map(|r| r.loot_stats.drop_rate()).sum::<f64>() / num_runs as f64;

        // Per-zone stats
        let mut avg_deaths_per_zone = vec![0.0; 11];
        let mut avg_kills_per_zone = vec![0.0; 11];
        let mut avg_ticks_per_zone = vec![0.0; 11];

        for i in 1..=10 {
            avg_deaths_per_zone[i] = runs
                .iter()
                .map(|r| r.zone_deaths.get(i).copied().unwrap_or(0) as f64)
                .sum::<f64>()
                / num_runs as f64;
            avg_kills_per_zone[i] = runs
                .iter()
                .map(|r| r.zone_kills.get(i).copied().unwrap_or(0) as f64)
                .sum::<f64>()
                / num_runs as f64;
            avg_ticks_per_zone[i] = runs
                .iter()
                .map(|r| r.ticks_per_zone.get(i).copied().unwrap_or(0) as f64)
                .sum::<f64>()
                / num_runs as f64;
        }

        // Calculate level pacing analysis
        let level_pacing = Self::analyze_level_pacing(&runs);

        // Detect difficulty walls
        let difficulty_walls =
            Self::detect_difficulty_walls(&avg_deaths_per_zone, &avg_kills_per_zone);

        // Summarize prestige progression
        let prestige_summary = Self::summarize_prestige(&runs);

        // Combat timing analysis
        let total_combat_ticks: u64 = runs.iter().map(|r| r.total_combat_ticks).sum();
        let total_fights: u64 = runs.iter().map(|r| r.fight_count).sum();
        let avg_fight_duration_ticks = if total_fights > 0 {
            total_combat_ticks as f64 / total_fights as f64
        } else {
            0.0
        };

        let total_ticks_all_runs: u64 = runs.iter().map(|r| r.total_ticks).sum();
        let avg_combat_percent = if total_ticks_all_runs > 0 {
            (total_combat_ticks as f64 / total_ticks_all_runs as f64) * 100.0
        } else {
            0.0
        };

        // Prestige wall analysis
        let total_wall_ticks: u64 = runs.iter().map(|r| r.ticks_at_zone_cap).sum();
        let avg_wall_percent = if total_ticks_all_runs > 0 {
            (total_wall_ticks as f64 / total_ticks_all_runs as f64) * 100.0
        } else {
            0.0
        };

        // XP source breakdown
        let total_xp_from_kills: u64 = runs.iter().map(|r| r.xp_from_kills).sum();
        let total_xp_from_passive: u64 = runs.iter().map(|r| r.xp_from_passive).sum();
        let total_xp = total_xp_from_kills + total_xp_from_passive;
        let xp_from_kills_percent = if total_xp > 0 {
            (total_xp_from_kills as f64 / total_xp as f64) * 100.0
        } else {
            0.0
        };
        let xp_from_passive_percent = if total_xp > 0 {
            (total_xp_from_passive as f64 / total_xp as f64) * 100.0
        } else {
            0.0
        };

        // Death breakdown
        let avg_boss_deaths =
            runs.iter().map(|r| r.boss_deaths as f64).sum::<f64>() / num_runs as f64;
        let avg_regular_deaths =
            runs.iter().map(|r| r.regular_deaths as f64).sum::<f64>() / num_runs as f64;

        Self {
            num_runs,
            runs_completed,
            runs_timed_out,
            avg_final_level,
            avg_final_zone,
            avg_total_kills,
            avg_total_deaths,
            avg_ticks_to_complete,
            level_distribution,
            zone_distribution,
            death_distribution,
            avg_legendary_drops,
            avg_upgrades_equipped,
            avg_final_ilvl,
            actual_drop_rate,
            avg_deaths_per_zone,
            avg_kills_per_zone,
            avg_ticks_per_zone,
            run_stats: runs,
            level_pacing,
            difficulty_walls,
            prestige_summary,
            show_level_curve: false,
            avg_fight_duration_ticks,
            avg_combat_percent,
            avg_wall_percent,
            xp_from_kills_percent,
            xp_from_passive_percent,
            avg_boss_deaths,
            avg_regular_deaths,
        }
    }

    /// Analyze level-up pacing from run data.
    fn analyze_level_pacing(runs: &[RunStats]) -> LevelPacingAnalysis {
        if runs.is_empty() {
            return LevelPacingAnalysis::default();
        }

        // Aggregate level-up ticks across all runs
        let mut level_ticks: Vec<Vec<u64>> = vec![Vec::new(); 101];

        for run in runs {
            for level in 2..=run.final_level.min(100) {
                let level_idx = level as usize;
                if level_idx < run.level_up_ticks.len() {
                    let current_tick = run.level_up_ticks[level_idx];
                    let prev_tick = if level_idx > 1 {
                        run.level_up_ticks[level_idx - 1]
                    } else {
                        0
                    };
                    if current_tick > prev_tick {
                        level_ticks[level_idx].push(current_tick - prev_tick);
                    }
                }
            }
        }

        // Calculate averages for ranges
        let ranges = [
            ("Levels  1-10", 2, 10),
            ("Levels 11-25", 11, 25),
            ("Levels 26-50", 26, 50),
            ("Levels 51-100", 51, 100),
        ];

        let mut avg_ticks_per_range = Vec::new();
        let mut prev_avg = 0.0;
        let mut slowdown_level = None;
        let mut slowdown_multiplier = 1.0;

        for (label, start, end) in ranges {
            let mut total_ticks = 0u64;
            let mut count = 0u64;

            for level in start..=end {
                if level < level_ticks.len() {
                    for &ticks in &level_ticks[level] {
                        total_ticks += ticks;
                        count += 1;
                    }
                }
            }

            let avg = if count > 0 {
                total_ticks as f64 / count as f64
            } else {
                0.0
            };

            if avg > 0.0 {
                avg_ticks_per_range.push((label.to_string(), avg));

                // Check for slowdown (>2x slower than previous range)
                if prev_avg > 0.0 && avg / prev_avg > 2.0 && slowdown_level.is_none() {
                    slowdown_level = Some(start as u32);
                    slowdown_multiplier = avg / prev_avg;
                }
                prev_avg = avg;
            }
        }

        LevelPacingAnalysis {
            avg_ticks_per_range,
            slowdown_level,
            slowdown_multiplier,
        }
    }

    /// Detect difficulty walls between zones.
    fn detect_difficulty_walls(
        avg_deaths_per_zone: &[f64],
        avg_kills_per_zone: &[f64],
    ) -> Vec<DifficultyWall> {
        let mut walls = Vec::new();

        // Calculate death rate per zone (deaths / kills as percentage)
        let mut death_rates: Vec<f64> = vec![0.0; 11];
        #[allow(clippy::needless_range_loop)]
        for zone in 1..=10 {
            let deaths = avg_deaths_per_zone.get(zone).copied().unwrap_or(0.0);
            let kills = avg_kills_per_zone.get(zone).copied().unwrap_or(0.0);
            death_rates[zone] = if kills > 0.0 {
                (deaths / kills) * 100.0
            } else {
                0.0
            };
        }

        // Check for walls (>20% jump between adjacent zones)
        for zone in 1..10 {
            let from_rate = death_rates[zone];
            let to_rate = death_rates[zone + 1];

            // Only flag if there's actual data and a significant jump
            if from_rate > 0.0 && to_rate > 0.0 && (to_rate - from_rate) > 20.0 {
                walls.push(DifficultyWall {
                    from_zone: zone as u32,
                    to_zone: (zone + 1) as u32,
                    from_death_rate: from_rate,
                    to_death_rate: to_rate,
                });
            }
        }

        walls
    }

    /// Summarize prestige progression across all runs.
    fn summarize_prestige(runs: &[RunStats]) -> Vec<PrestigeCycleSummary> {
        if runs.is_empty() {
            return Vec::new();
        }

        // Group cycles by prestige rank
        let mut cycles_by_rank: HashMap<u32, Vec<&PrestigeCycle>> = HashMap::new();

        for run in runs {
            for cycle in &run.prestige_cycles {
                cycles_by_rank.entry(cycle.rank).or_default().push(cycle);
            }
        }

        // Calculate averages per rank
        let mut summaries: Vec<PrestigeCycleSummary> = Vec::new();
        let max_rank = cycles_by_rank.keys().max().copied().unwrap_or(0);

        let mut prev_avg_ticks = 0.0;

        for rank in 0..=max_rank {
            if let Some(cycles) = cycles_by_rank.get(&rank) {
                let avg_ticks = cycles
                    .iter()
                    .map(|c| c.ticks_to_complete as f64)
                    .sum::<f64>()
                    / cycles.len() as f64;
                let avg_deaths =
                    cycles.iter().map(|c| c.total_deaths as f64).sum::<f64>() / cycles.len() as f64;

                let improvement_pct = if prev_avg_ticks > 0.0 {
                    ((prev_avg_ticks - avg_ticks) / prev_avg_ticks) * 100.0
                } else {
                    0.0
                };

                summaries.push(PrestigeCycleSummary {
                    rank,
                    avg_ticks,
                    avg_deaths,
                    improvement_pct,
                });

                prev_avg_ticks = avg_ticks;
            }
        }

        summaries
    }

    /// Generate a text report.
    pub fn to_text(&self) -> String {
        let mut report = String::new();

        report.push_str("═══════════════════════════════════════════════════════════════\n");
        report.push_str("                    SIMULATION REPORT\n");
        report.push_str("               (Using Real Game Mechanics)\n");
        report.push_str("═══════════════════════════════════════════════════════════════\n\n");

        report.push_str(&format!(
            "Runs: {} total, {} completed, {} timed out\n\n",
            self.num_runs, self.runs_completed, self.runs_timed_out
        ));

        report.push_str("── PROGRESSION ──────────────────────────────────────────────────\n");
        report.push_str(&format!(
            "  Avg Final Level:     {:.1}\n",
            self.avg_final_level
        ));
        report.push_str(&format!(
            "  Avg Final Zone:      {:.1}\n",
            self.avg_final_zone
        ));
        report.push_str(&format!(
            "  Avg Total Kills:     {:.0}\n",
            self.avg_total_kills
        ));
        report.push_str(&format!(
            "  Avg Total Deaths:    {:.1} (Boss: {:.1}, Regular: {:.1})\n",
            self.avg_total_deaths, self.avg_boss_deaths, self.avg_regular_deaths
        ));
        report.push_str(&format!(
            "  Avg Time to Clear:   {} ({:.0} ticks)\n\n",
            ticks_to_time_str(self.avg_ticks_to_complete as u64),
            self.avg_ticks_to_complete
        ));

        report.push_str("── LOOT ─────────────────────────────────────────────────────────\n");
        report.push_str(&format!(
            "  Actual Drop Rate:    {:.1}%\n",
            self.actual_drop_rate * 100.0
        ));
        report.push_str(&format!(
            "  Avg Legendary Drops: {:.2}\n",
            self.avg_legendary_drops
        ));
        report.push_str(&format!(
            "  Avg Upgrades Equipped: {:.1}\n",
            self.avg_upgrades_equipped
        ));
        report.push_str(&format!(
            "  Avg Final ilvl:      {:.1}\n\n",
            self.avg_final_ilvl
        ));

        report.push_str("── COMBAT TIMING ────────────────────────────────────────────────\n");
        let avg_fight_seconds = ticks_to_seconds(self.avg_fight_duration_ticks as u64);
        let fight_rating = if avg_fight_seconds < 5.0 {
            "FAST (under 5s)"
        } else if avg_fight_seconds <= 10.0 {
            "TARGET (5-10s)"
        } else {
            "SLOW (over 10s)"
        };
        report.push_str(&format!(
            "  Avg Fight Duration:  {:.1}s ({} ticks) — {}\n",
            avg_fight_seconds, self.avg_fight_duration_ticks as u64, fight_rating
        ));
        report.push_str(&format!(
            "  Time in Combat:      {:.1}%\n\n",
            self.avg_combat_percent
        ));

        report.push_str("── XP SOURCES ───────────────────────────────────────────────────\n");
        report.push_str(&format!(
            "  XP from Kills:       {:.1}%\n",
            self.xp_from_kills_percent
        ));
        report.push_str(&format!(
            "  XP from Passive:     {:.1}%\n\n",
            self.xp_from_passive_percent
        ));

        if self.avg_wall_percent > 0.1 {
            report.push_str("── PRESTIGE WALL ANALYSIS ───────────────────────────────────────\n");
            report.push_str(&format!(
                "  Time at Zone Cap:    {:.1}% of run\n",
                self.avg_wall_percent
            ));
            if self.avg_wall_percent > 20.0 {
                report.push_str("  ⚠️  Significant grind at prestige gate (>20% at cap)\n");
            }
            report.push('\n');
        }

        report.push_str("── ZONE COMPLETION ──────────────────────────────────────────────\n");
        for zone in 1..=10 {
            let reached = self.zone_distribution.get(&zone).copied().unwrap_or(0);
            let pct = (reached as f64 / self.num_runs as f64) * 100.0;
            let bar_len = (pct / 5.0) as usize;
            let bar: String = "█".repeat(bar_len);
            report.push_str(&format!("  Zone {:2}: {:>5.1}% {}\n", zone, pct, bar));
        }
        report.push('\n');

        report.push_str("── PER-ZONE BREAKDOWN ───────────────────────────────────────────\n");
        report.push_str("  Zone   Deaths    Kills    Time         Deaths/Kill\n");
        report.push_str("  ────   ──────    ─────    ────         ───────────\n");
        for zone in 1..=10 {
            let deaths = self.avg_deaths_per_zone[zone];
            let kills = self.avg_kills_per_zone[zone];
            let ticks = self.avg_ticks_per_zone[zone];
            let deaths_per_kill = if kills > 0.0 { deaths / kills } else { 0.0 };

            if kills > 0.0 {
                report.push_str(&format!(
                    "  {:4}   {:6.1}   {:6.0}   {:10}   {:.3}\n",
                    zone,
                    deaths,
                    kills,
                    ticks_to_time_str(ticks as u64),
                    deaths_per_kill
                ));
            }
        }
        report.push('\n');

        report.push_str("── DEATH ANALYSIS ───────────────────────────────────────────────\n");
        let min_deaths = self.death_distribution.iter().min().unwrap_or(&0);
        let max_deaths = self.death_distribution.iter().max().unwrap_or(&0);
        let median_deaths = {
            let mut sorted = self.death_distribution.clone();
            sorted.sort();
            sorted.get(sorted.len() / 2).copied().unwrap_or(0)
        };
        report.push_str(&format!("  Min Deaths:    {}\n", min_deaths));
        report.push_str(&format!("  Median Deaths: {}\n", median_deaths));
        report.push_str(&format!("  Max Deaths:    {}\n\n", max_deaths));

        // Level Pacing section
        if !self.level_pacing.avg_ticks_per_range.is_empty() {
            report.push_str("── LEVEL PACING ─────────────────────────────────────────────────\n");
            for (label, avg_ticks) in &self.level_pacing.avg_ticks_per_range {
                let time_str = ticks_to_time_str(*avg_ticks as u64);
                report.push_str(&format!(
                    "  {}:   {} ({:.0} ticks) per level\n",
                    label, time_str, avg_ticks
                ));
            }
            if let Some(slowdown_level) = self.level_pacing.slowdown_level {
                report.push_str(&format!(
                    "  ⚠️ Leveling slowdown at level {}+ ({:.1}x slower)\n",
                    slowdown_level, self.level_pacing.slowdown_multiplier
                ));
            }
            report.push('\n');
        }

        // Difficulty Walls section
        if !self.difficulty_walls.is_empty() {
            report.push_str("── DIFFICULTY WALLS ─────────────────────────────────────────────\n");
            for wall in &self.difficulty_walls {
                report.push_str(&format!(
                    "  ⚠️ Wall at Zone {}→{}: death rate {:.0}% → {:.0}%\n",
                    wall.from_zone, wall.to_zone, wall.from_death_rate, wall.to_death_rate
                ));
            }
            report.push('\n');
        }

        // Prestige Progression section
        if !self.prestige_summary.is_empty() {
            report.push_str("── PRESTIGE PROGRESSION ─────────────────────────────────────────\n");

            // Get P0 time for speedup comparison
            let p0_ticks = self
                .prestige_summary
                .iter()
                .find(|s| s.rank == 0)
                .map(|s| s.avg_ticks)
                .unwrap_or(1.0);

            for summary in &self.prestige_summary {
                let time_str = ticks_to_time_str(summary.avg_ticks as u64);

                // Calculate actual vs theoretical speedup
                let actual_speedup = if summary.rank > 0 && p0_ticks > 0.0 {
                    p0_ticks / summary.avg_ticks
                } else {
                    1.0
                };
                let theoretical = theoretical_prestige_mult(summary.rank);

                if summary.rank == 0 {
                    report.push_str(&format!(
                        "  P{}: {} to Z10, {:.0} deaths (baseline)\n",
                        summary.rank, time_str, summary.avg_deaths
                    ));
                } else {
                    let speedup_diff = actual_speedup - theoretical;
                    let speedup_indicator = if speedup_diff.abs() < 0.1 {
                        "✓"
                    } else if speedup_diff < 0.0 {
                        "⚠️"
                    } else {
                        "+"
                    };
                    report.push_str(&format!(
                        "  P{}: {} to Z10, {:.0} deaths — {:.2}x actual vs {:.2}x theory {}\n",
                        summary.rank,
                        time_str,
                        summary.avg_deaths,
                        actual_speedup,
                        theoretical,
                        speedup_indicator
                    ));
                }
            }
            report.push('\n');
        }

        report.push_str("── BALANCE ASSESSMENT ───────────────────────────────────────────\n");
        let completion_rate = (self.runs_completed as f64 / self.num_runs as f64) * 100.0;
        let death_rating = if self.avg_total_deaths < 5.0 {
            "TOO EASY - Players rarely die"
        } else if self.avg_total_deaths < 20.0 {
            "GOOD - Challenging but fair"
        } else if self.avg_total_deaths < 50.0 {
            "HARD - Many deaths but completable"
        } else {
            "TOO HARD - Excessive deaths"
        };

        report.push_str(&format!("  Completion Rate: {:.1}%\n", completion_rate));
        report.push_str(&format!("  Death Rating:    {}\n", death_rating));

        // Identify problem zones (high death rate)
        for zone in 1..=10 {
            let deaths = self.avg_deaths_per_zone[zone];
            let kills = self.avg_kills_per_zone[zone];
            if kills > 0.0 {
                let deaths_per_kill = deaths / kills;
                if deaths_per_kill > 0.5 {
                    report.push_str(&format!(
                        "  ⚠️  Zone {} has high death rate ({:.1}% per fight)\n",
                        zone,
                        deaths_per_kill * 100.0
                    ));
                }
            }
        }

        if self.avg_final_zone < 5.0 {
            report.push_str("  ⚠️  Most runs stuck early - early game too hard?\n");
        }
        if self.avg_legendary_drops < 0.5 && self.runs_completed > 0 {
            report.push_str("  ⚠️  Very few legendaries - boss rates too low?\n");
        }
        if self.avg_total_deaths > 100.0 {
            report.push_str("  ⚠️  Death count very high - damage/HP imbalance?\n");
        }

        report.push_str("\n═══════════════════════════════════════════════════════════════\n");

        report
    }

    /// Generate detailed level curve output.
    pub fn level_curve_text(&self) -> String {
        let mut output = String::new();
        output.push_str("── DETAILED LEVEL CURVE ─────────────────────────────────────────\n");

        if self.run_stats.is_empty() {
            output.push_str("  No run data available.\n");
            return output;
        }

        // Calculate per-level average ticks
        let mut level_ticks: Vec<Vec<u64>> = vec![Vec::new(); 101];

        for run in &self.run_stats {
            for level in 2..=run.final_level.min(100) {
                let level_idx = level as usize;
                if level_idx < run.level_up_ticks.len() {
                    let current_tick = run.level_up_ticks[level_idx];
                    let prev_tick = if level_idx > 1 {
                        run.level_up_ticks[level_idx - 1]
                    } else {
                        0
                    };
                    if current_tick > prev_tick {
                        level_ticks[level_idx].push(current_tick - prev_tick);
                    }
                }
            }
        }

        output.push_str("  Level   Time       Ticks     Samples\n");
        output.push_str("  ─────   ────       ─────     ───────\n");

        #[allow(clippy::needless_range_loop)]
        for level in 2..=100 {
            if !level_ticks[level].is_empty() {
                let avg =
                    level_ticks[level].iter().sum::<u64>() as f64 / level_ticks[level].len() as f64;
                let samples = level_ticks[level].len();
                output.push_str(&format!(
                    "  {:5}   {:10} {:9.0}   {:7}\n",
                    level,
                    ticks_to_time_str(avg as u64),
                    avg,
                    samples
                ));
            }
        }

        output
    }

    /// Generate a JSON report for further analysis.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}

// Implement Serialize for JSON output
impl serde::Serialize for SimReport {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("SimReport", 16)?;
        state.serialize_field("num_runs", &self.num_runs)?;
        state.serialize_field("runs_completed", &self.runs_completed)?;
        state.serialize_field("runs_timed_out", &self.runs_timed_out)?;
        state.serialize_field("avg_final_level", &self.avg_final_level)?;
        state.serialize_field("avg_final_zone", &self.avg_final_zone)?;
        state.serialize_field("avg_total_kills", &self.avg_total_kills)?;
        state.serialize_field("avg_total_deaths", &self.avg_total_deaths)?;
        state.serialize_field("avg_ticks_to_complete", &self.avg_ticks_to_complete)?;
        state.serialize_field("avg_legendary_drops", &self.avg_legendary_drops)?;
        state.serialize_field("avg_upgrades_equipped", &self.avg_upgrades_equipped)?;
        state.serialize_field("avg_final_ilvl", &self.avg_final_ilvl)?;
        state.serialize_field("actual_drop_rate", &self.actual_drop_rate)?;
        state.serialize_field("avg_deaths_per_zone", &self.avg_deaths_per_zone)?;
        state.serialize_field("avg_kills_per_zone", &self.avg_kills_per_zone)?;
        state.serialize_field(
            "completion_rate",
            &((self.runs_completed as f64 / self.num_runs as f64) * 100.0),
        )?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::super::loot_sim::LootStats;
    use super::*;

    #[test]
    fn test_report_generation() {
        let runs = vec![
            RunStats {
                final_level: 50,
                final_zone: 5,
                final_subzone: 1,
                final_prestige: 0,
                total_kills: 500,
                total_boss_kills: 10,
                total_deaths: 15,
                total_ticks: 10000,
                loot_stats: LootStats::default(),
                final_avg_ilvl: 45.0,
                reached_target: true,
                zone_deaths: vec![0; 11],
                zone_kills: vec![0; 11],
                ticks_per_zone: vec![0; 11],
                level_up_ticks: vec![0; 101],
                prestige_cycles: Vec::new(),
                total_combat_ticks: 5000,
                total_regen_ticks: 2000,
                fight_count: 500,
                ticks_at_zone_cap: 0,
                xp_from_kills: 10000,
                xp_from_passive: 5000,
                boss_deaths: 2,
                regular_deaths: 13,
            },
            RunStats {
                final_level: 45,
                final_zone: 5,
                final_subzone: 1,
                final_prestige: 0,
                total_kills: 450,
                total_boss_kills: 8,
                total_deaths: 20,
                total_ticks: 9000,
                loot_stats: LootStats::default(),
                final_avg_ilvl: 40.0,
                reached_target: true,
                zone_deaths: vec![0; 11],
                zone_kills: vec![0; 11],
                ticks_per_zone: vec![0; 11],
                level_up_ticks: vec![0; 101],
                prestige_cycles: Vec::new(),
                total_combat_ticks: 4500,
                total_regen_ticks: 1800,
                fight_count: 450,
                ticks_at_zone_cap: 0,
                xp_from_kills: 9000,
                xp_from_passive: 4500,
                boss_deaths: 3,
                regular_deaths: 17,
            },
        ];

        let report = SimReport::from_runs(runs, 5, 100000);
        assert_eq!(report.num_runs, 2);
        assert_eq!(report.runs_completed, 2);
        assert!((report.avg_final_level - 47.5).abs() < 0.1);
    }

    #[test]
    fn test_ticks_to_time_str() {
        assert_eq!(ticks_to_time_str(0), "~0s");
        assert_eq!(ticks_to_time_str(10), "~1s");
        assert_eq!(ticks_to_time_str(100), "~10s");
        assert_eq!(ticks_to_time_str(600), "~1m 0s");
        assert_eq!(ticks_to_time_str(2000), "~3m 20s");
        assert_eq!(ticks_to_time_str(36000), "~1h 0m");
    }

    #[test]
    fn test_theoretical_prestige_mult() {
        // P0 = 1.0
        assert!((theoretical_prestige_mult(0) - 1.0).abs() < 0.01);
        // P1 = 1.5
        assert!((theoretical_prestige_mult(1) - 1.5).abs() < 0.01);
        // P5 = 1.0 + 0.5 * 5^0.7 ≈ 2.54
        assert!((theoretical_prestige_mult(5) - 2.54).abs() < 0.1);
    }
}
