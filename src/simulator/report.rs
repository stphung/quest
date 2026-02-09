//! Simulation report generation.

use super::progression_sim::RunStats;
use std::collections::HashMap;

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
        }
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
            "  Avg Total Deaths:    {:.1}\n",
            self.avg_total_deaths
        ));
        report.push_str(&format!(
            "  Avg Ticks to Clear:  {:.0}\n\n",
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
        report.push_str("  Zone   Deaths    Kills    Ticks    Deaths/Kill\n");
        report.push_str("  ────   ──────    ─────    ─────    ───────────\n");
        for zone in 1..=10 {
            let deaths = self.avg_deaths_per_zone[zone];
            let kills = self.avg_kills_per_zone[zone];
            let ticks = self.avg_ticks_per_zone[zone];
            let deaths_per_kill = if kills > 0.0 { deaths / kills } else { 0.0 };

            if kills > 0.0 {
                report.push_str(&format!(
                    "  {:4}   {:6.1}   {:6.0}   {:6.0}   {:.3}\n",
                    zone, deaths, kills, ticks, deaths_per_kill
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
            },
        ];

        let report = SimReport::from_runs(runs, 5, 100000);
        assert_eq!(report.num_runs, 2);
        assert_eq!(report.runs_completed, 2);
        assert!((report.avg_final_level - 47.5).abs() < 0.1);
    }
}
