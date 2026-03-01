//! The Deep — Standalone Balance Tuning Simulator
//!
//! An event-driven simulator for The Deep's mercenary expedition system.
//! Instead of stepping through 100ms game ticks, it jumps forward in time
//! to the next mission completion, with an AI agent making strategic decisions.
//!
//! Usage:
//!   cargo run --release --bin deep_simulator -- [OPTIONS]
//!
//! Options:
//!   --hours N           Simulated wall-clock hours (default: 168 = 1 week)
//!   --seed N            RNG seed (default: 42)
//!   --guild-rank N      Starting guild rank 1-5 (default: 1)
//!   --pre-cleared N     Pre-clear layers 1..N (default: 0)
//!   --starting-marks N  Starting Warband Marks (default: 50)
//!   --strategy STR      rush | farm | balanced | infrastructure (default: balanced)
//!   --runs N            Multi-run with seeds seed..seed+N-1 (default: 1)
//!   --verbose           Per-mission event logging
//!   --csv FILE          Hourly time-series CSV export
//!   --quiet             Single summary line per run

use chrono::{DateTime, Duration, TimeZone, Utc};
use quest::deep::{
    build_infrastructure, effective_concurrent_missions, effective_duration_secs,
    generate_mercenary, generate_mission_pool, generate_recruit_pool, generate_starter_roster,
    guild_upgrade_cost, infrastructure_build_cost, is_daily_supply_run_available,
    mark_layer_cleared, mission_launch_cost, mission_power_threshold, purge_lost_mercs,
    roster_has_capacity, start_mission, tick_all_missions, tick_merc_injury,
    try_upgrade_guild_rank, validate_squad_assignment, AvailableMission, DeepPersistent,
    DeepPrestige, GuildRank, Infrastructure, LayerTier, MercArchetype, MercQuality, MercStatus,
    MissionOutcome, MissionStatus, MissionType, RecruitPool,
};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::io::Write;

// ── Strategy ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeepStrategy {
    Rush,
    Farm,
    Balanced,
    Infrastructure,
}

impl DeepStrategy {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "rush" => Some(Self::Rush),
            "farm" => Some(Self::Farm),
            "balanced" => Some(Self::Balanced),
            "infrastructure" => Some(Self::Infrastructure),
            _ => None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Rush => "rush",
            Self::Farm => "farm",
            Self::Balanced => "balanced",
            Self::Infrastructure => "infrastructure",
        }
    }

    /// Mission type priority order for this strategy.
    fn mission_priority(self) -> &'static [&'static str] {
        match self {
            Self::Rush => &["Breakthrough", "Expedition", "Recon", "Supply Run"],
            Self::Farm => &["Supply Run", "Construction", "Recon"],
            Self::Balanced => &[
                "Breakthrough",
                "Recon",
                "Supply Run",
                "Expedition",
                "Construction",
            ],
            Self::Infrastructure => &["Construction", "Supply Run", "Recon", "Breakthrough"],
        }
    }

    /// Whether to attempt guild upgrade at current marks balance.
    fn should_upgrade_guild(self, marks: u32, cost: u32) -> bool {
        match self {
            Self::Rush => marks >= cost,
            Self::Farm => marks >= cost * 2,
            Self::Balanced => marks >= cost + cost / 2,
            Self::Infrastructure => marks >= cost + cost / 2,
        }
    }

    /// Whether to recruit more mercs given current roster state.
    /// Marks thresholds calibrated for post-rebalance recruit costs
    /// (Common 50-80, Uncommon 80-130, Rare 130-200, Elite 200-300).
    fn should_recruit(
        self,
        available_count: usize,
        roster_size: usize,
        concurrent: u32,
        marks: u32,
    ) -> bool {
        match self {
            Self::Rush => roster_size < (concurrent as usize) * 3 && marks > 60,
            Self::Farm => marks > 400 && available_count < (concurrent as usize) * 2,
            Self::Balanced => available_count < (concurrent as usize) * 2 && marks > 100,
            Self::Infrastructure => marks > 160 && available_count < (concurrent as usize) * 3,
        }
    }
}

// ── Configuration ──────────────────────────────────────────────────────────────

struct DeepSimConfig {
    hours: u64,
    seed: u64,
    guild_rank: u8,
    pre_cleared_layers: u32,
    starting_marks: u32,
    strategy: DeepStrategy,
    runs: u32,
    verbose: bool,
    csv_path: Option<String>,
    quiet: bool,
}

impl Default for DeepSimConfig {
    fn default() -> Self {
        Self {
            hours: 168,
            seed: 42,
            guild_rank: 1,
            pre_cleared_layers: 0,
            starting_marks: 50,
            strategy: DeepStrategy::Balanced,
            runs: 1,
            verbose: false,
            csv_path: None,
            quiet: false,
        }
    }
}

fn parse_args() -> DeepSimConfig {
    let args: Vec<String> = std::env::args().collect();
    let mut config = DeepSimConfig::default();
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "--hours" => {
                i += 1;
                config.hours = args[i].parse().expect("--hours requires a number");
            }
            "--seed" => {
                i += 1;
                config.seed = args[i].parse().expect("--seed requires a number");
            }
            "--guild-rank" => {
                i += 1;
                let rank: u8 = args[i].parse().expect("--guild-rank requires 1-5");
                assert!((1..=5).contains(&rank), "--guild-rank must be 1-5");
                config.guild_rank = rank;
            }
            "--pre-cleared" => {
                i += 1;
                config.pre_cleared_layers =
                    args[i].parse().expect("--pre-cleared requires a number");
            }
            "--starting-marks" => {
                i += 1;
                config.starting_marks =
                    args[i].parse().expect("--starting-marks requires a number");
            }
            "--strategy" => {
                i += 1;
                config.strategy = DeepStrategy::from_str(&args[i])
                    .unwrap_or_else(|| panic!("Unknown strategy: {}", args[i]));
            }
            "--runs" => {
                i += 1;
                config.runs = args[i].parse().expect("--runs requires a number");
            }
            "--verbose" => config.verbose = true,
            "--csv" => {
                i += 1;
                config.csv_path = Some(args[i].clone());
            }
            "--quiet" => config.quiet = true,
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            other => {
                eprintln!("Unknown option: {other}");
                print_usage();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    config
}

fn print_usage() {
    eprintln!(
        "The Deep — Balance Simulator\n\
         \n\
         Usage: cargo run --release --bin deep_simulator -- [OPTIONS]\n\
         \n\
         Options:\n\
         \x20 --hours N           Simulated wall-clock hours (default: 168 = 1 week)\n\
         \x20 --seed N            RNG seed (default: 42)\n\
         \x20 --guild-rank N      Starting guild rank 1-5 (default: 1)\n\
         \x20 --pre-cleared N     Pre-clear layers 1..N (default: 0)\n\
         \x20 --starting-marks N  Starting Warband Marks (default: 50)\n\
         \x20 --strategy STR      rush | farm | balanced | infrastructure (default: balanced)\n\
         \x20 --runs N            Multi-run with seeds seed..seed+N-1 (default: 1)\n\
         \x20 --verbose           Per-mission event logging\n\
         \x20 --csv FILE          Hourly time-series CSV export\n\
         \x20 --quiet             Single summary line per run"
    );
}

// ── Statistics ──────────────────────────────────────────────────────────────────

#[derive(Default)]
struct DeepSimStats {
    // Mission counts by type
    supply_runs: u32,
    recon_missions: u32,
    expedition_missions: u32,
    breakthrough_missions: u32,
    construction_missions: u32,

    // Outcomes
    successes: u32,
    partial_successes: u32,
    failures: u32,

    // Economy
    total_marks_earned: u64,
    marks_spent_missions: u64,
    marks_spent_recruitment: u64,
    marks_spent_guild: u64,
    marks_spent_infrastructure: u64,
    peak_marks: u32,

    // Progression
    deepest_layer: u32,
    layers_cleared: u32,
    final_guild_rank: u8,
    layer_clear_times: Vec<(u32, f64)>, // (layer, hours)
    guild_rank_times: Vec<(u8, f64)>,   // (rank, hours)

    // Roster
    mercs_recruited: u32,
    mercs_lost: u32,
    injury_count: u32,
    level_ups: u32,
    max_merc_level: u32,

    // Infrastructure
    infrastructure_built: u32,

    // Timing
    total_missions_launched: u32,

    // CSV data: hourly snapshots
    hourly_snapshots: Vec<HourlySnapshot>,
}

#[derive(Clone)]
struct HourlySnapshot {
    sim_hour: f64,
    marks: u32,
    deepest_layer: u32,
    layers_cleared: u32,
    guild_rank: u8,
    roster_size: usize,
    available_mercs: usize,
    active_missions: usize,
    total_missions: u32,
    marks_earned: u64,
    marks_spent: u64,
    mercs_lost: u32,
    infrastructure: u32,
}

impl DeepSimStats {
    fn total_launched(&self) -> u32 {
        self.total_missions_launched
    }

    fn success_rate(&self) -> f64 {
        let total = self.successes + self.partial_successes + self.failures;
        if total == 0 {
            return 0.0;
        }
        self.successes as f64 / total as f64 * 100.0
    }

    fn total_marks_spent(&self) -> u64 {
        self.marks_spent_missions
            + self.marks_spent_recruitment
            + self.marks_spent_guild
            + self.marks_spent_infrastructure
    }

    fn record_mission_type(&mut self, mt: MissionType) {
        match mt {
            MissionType::SupplyRun => self.supply_runs += 1,
            MissionType::Recon => self.recon_missions += 1,
            MissionType::Expedition => self.expedition_missions += 1,
            MissionType::Breakthrough | MissionType::GatewayExpedition => {
                self.breakthrough_missions += 1
            }
            MissionType::Construction(_) => self.construction_missions += 1,
        }
    }

    fn record_outcome(&mut self, outcome: &MissionOutcome) {
        match outcome {
            MissionOutcome::Success => self.successes += 1,
            MissionOutcome::PartialSuccess => self.partial_successes += 1,
            MissionOutcome::Failure => self.failures += 1,
        }
    }
}

// ── State Initialization ───────────────────────────────────────────────────────

fn initialize_state(
    config: &DeepSimConfig,
    seed: u64,
    sim_start: DateTime<Utc>,
) -> (DeepPersistent, DeepPrestige, ChaCha8Rng) {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    let mut persistent = DeepPersistent::new();
    persistent.discovered = true;
    persistent.guild_rank = GuildRank(config.guild_rank);

    // Pre-clear layers.
    for layer in 1..=config.pre_cleared_layers {
        mark_layer_cleared(&mut persistent, layer);
    }

    // Generate starter roster.
    let starter_roster = generate_starter_roster(
        persistent.guild_rank,
        || persistent.next_merc_id(),
        &mut rng,
    );

    let mut prestige = DeepPrestige {
        warband_marks: config.starting_marks,
        roster: starter_roster,
        active_missions: Vec::new(),
        available_missions: Vec::new(),
        recruit_pool: RecruitPool {
            candidates: Vec::new(),
            refreshed_at: sim_start,
            recruit_costs: Vec::new(),
        },
        pending_results: Vec::new(),
        generation_number: 0,
        warband_log: Vec::new(),
        total_marks_earned: 0,
        total_missions_completed: 0,
        total_mercs_lost: 0,
        pool_refreshed_at: None,
    };

    // Generate initial mission pool.
    prestige.available_missions = generate_mission_pool(&persistent, &mut rng);

    // Generate initial recruit pool.
    prestige.recruit_pool = generate_recruit_pool(
        persistent.guild_rank,
        || persistent.next_merc_id(),
        &mut rng,
    );
    prestige.recruit_pool.refreshed_at = sim_start;

    (persistent, prestige, rng)
}

// ── AI Decision Functions ──────────────────────────────────────────────────────

/// Try to upgrade the guild rank if strategy says to.
fn ai_upgrade_guild(
    persistent: &mut DeepPersistent,
    prestige: &mut DeepPrestige,
    strategy: DeepStrategy,
    stats: &mut DeepSimStats,
    elapsed_hours: f64,
) {
    if let Some(cost) = guild_upgrade_cost(persistent.guild_rank) {
        if strategy.should_upgrade_guild(prestige.warband_marks, cost) {
            let old_rank = persistent.guild_rank.0;
            if try_upgrade_guild_rank(persistent, prestige).is_ok() {
                let spent = cost;
                stats.marks_spent_guild += spent as u64;
                stats
                    .guild_rank_times
                    .push((persistent.guild_rank.0, elapsed_hours));
            }
            let _ = old_rank; // suppress unused warning
        }
    }
}

/// Try to recruit mercs if strategy says to.
fn ai_recruit(
    persistent: &mut DeepPersistent,
    prestige: &mut DeepPrestige,
    strategy: DeepStrategy,
    stats: &mut DeepSimStats,
    now: DateTime<Utc>,
    rng: &mut ChaCha8Rng,
) {
    // Refresh recruit pool if needed.
    if prestige.recruit_pool.needs_refresh(now) {
        prestige.recruit_pool =
            generate_recruit_pool(persistent.guild_rank, || persistent.next_merc_id(), rng);
        prestige.recruit_pool.refreshed_at = now;
    }

    let concurrent =
        effective_concurrent_missions(persistent.guild_rank, persistent.deepest_layer_reached);
    let avail = prestige.available_merc_count();
    let roster_size = prestige.roster.len();

    if !strategy.should_recruit(avail, roster_size, concurrent, prestige.warband_marks) {
        return;
    }

    if !roster_has_capacity(&prestige.roster, persistent.guild_rank) {
        return;
    }

    // Try to recruit the cheapest candidate we can afford.
    if let Some(best_idx) = find_best_recruit(prestige) {
        let cost = prestige.recruit_pool.recruit_costs[best_idx];
        if prestige.spend_marks(cost) {
            let merc = prestige.recruit_pool.candidates.remove(best_idx);
            prestige.recruit_pool.recruit_costs.remove(best_idx);
            prestige.roster.push(merc);
            stats.mercs_recruited += 1;
            stats.marks_spent_recruitment += cost as u64;
        }
    }
}

/// Find the best recruit candidate (cheapest that we can afford).
fn find_best_recruit(prestige: &DeepPrestige) -> Option<usize> {
    let marks = prestige.warband_marks;
    prestige
        .recruit_pool
        .recruit_costs
        .iter()
        .enumerate()
        .filter(|(_, &cost)| cost <= marks)
        .min_by_key(|(_, &cost)| cost)
        .map(|(idx, _)| idx)
}

/// Pick a squad for a given available mission.
fn pick_squad(available: &AvailableMission, prestige: &DeepPrestige) -> Option<Vec<u64>> {
    let mut candidates: Vec<_> = prestige
        .roster
        .iter()
        .filter(|m| m.is_available())
        .collect();

    if candidates.is_empty() {
        return None;
    }

    // Sort by effective_power descending.
    candidates.sort_by_key(|m| std::cmp::Reverse(m.effective_power()));

    let mut squad: Vec<u64> = Vec::new();
    let mut total_power: u32 = 0;

    // If required_archetype set, include one matching merc first.
    if let Some(req_arch) = available.required_archetype {
        if let Some(pos) = candidates.iter().position(|m| m.archetype == req_arch) {
            let merc = candidates.remove(pos);
            total_power += merc.effective_power();
            squad.push(merc.id);
        }
    }

    // Add highest-power mercs until min_squad_power is met.
    for merc in &candidates {
        if total_power >= available.min_squad_power && !squad.is_empty() {
            break;
        }
        squad.push(merc.id);
        total_power += merc.effective_power();
    }

    if squad.is_empty() {
        return None;
    }

    Some(squad)
}

/// Ensure the mission pool includes a Breakthrough for the frontier layer.
///
/// At low guild ranks (1-2), the pool only has 3 slots (SupplyRun, Recon,
/// Expedition) and Breakthrough doesn't fit. This injects one by replacing
/// the lowest-priority mission for the current strategy.
fn ensure_breakthrough_in_pool(
    persistent: &DeepPersistent,
    prestige: &mut DeepPrestige,
    strategy: DeepStrategy,
) {
    let frontier = persistent.frontier_layer();

    // Don't inject if frontier is already cleared.
    let frontier_cleared = persistent
        .layer_record(frontier)
        .map(|r| r.cleared)
        .unwrap_or(false);
    if frontier_cleared {
        return;
    }

    // Don't inject if pool already has a Breakthrough.
    if prestige
        .available_missions
        .iter()
        .any(|m| matches!(m.mission_type, MissionType::Breakthrough))
    {
        return;
    }

    // Create a Breakthrough mission manually.
    let tier = LayerTier::from_layer(frontier);
    let duration_secs =
        effective_duration_secs(tier, MissionType::Breakthrough, frontier, persistent);
    let min_power = mission_power_threshold(frontier, MissionType::Breakthrough);
    let cost = mission_launch_cost(MissionType::Breakthrough, frontier);

    let breakthrough = AvailableMission {
        mission_type: MissionType::Breakthrough,
        layer: frontier,
        duration_secs,
        min_squad_power: min_power,
        required_archetype: None,
        recommended_archetype: None,
        marks_cost: cost,
        description: "Push deeper into the unknown.".to_string(),
    };

    // Replace the lowest-priority mission in the pool, but never replace a SupplyRun
    // (it's the economic foundation — without it, post-prestige and low-mark scenarios deadlock).
    if !prestige.available_missions.is_empty() {
        let worst_idx = prestige
            .available_missions
            .iter()
            .enumerate()
            .filter(|(_, m)| !matches!(m.mission_type, MissionType::SupplyRun))
            .max_by_key(|(_, m)| mission_priority_score(m.mission_type, strategy))
            .map(|(i, _)| i);
        if let Some(idx) = worst_idx {
            prestige.available_missions[idx] = breakthrough;
        } else {
            // All missions are SupplyRuns — just append.
            prestige.available_missions.push(breakthrough);
        }
    } else {
        prestige.available_missions.push(breakthrough);
    }
}

/// Ensure the pool includes a Construction mission when there are cleared layers
/// with available infrastructure slots.
fn ensure_construction_in_pool(
    persistent: &DeepPersistent,
    prestige: &mut DeepPrestige,
    strategy: DeepStrategy,
) {
    // Only inject if strategy cares about construction.
    if !strategy.mission_priority().contains(&"Construction") {
        return;
    }

    // Don't inject if already present.
    if prestige
        .available_missions
        .iter()
        .any(|m| matches!(m.mission_type, MissionType::Construction(_)))
    {
        return;
    }

    // Find a cleared layer with buildable infrastructure.
    let target = persistent
        .layers
        .iter()
        .filter(|l| l.cleared && l.infrastructure.len() < Infrastructure::ALL.len())
        .map(|l| l.index)
        .next();

    let Some(target_layer) = target else {
        return;
    };

    // Pick an infrastructure type not yet built.
    let built = &persistent
        .layers
        .iter()
        .find(|l| l.index == target_layer)
        .map(|l| l.infrastructure.clone())
        .unwrap_or_default();
    let available_infra: Vec<Infrastructure> = Infrastructure::ALL
        .iter()
        .filter(|&&i| !built.contains(&i))
        .copied()
        .collect();

    if available_infra.is_empty() {
        return;
    }

    let infra = available_infra[0]; // Pick deterministically.
    let tier = LayerTier::from_layer(target_layer);
    let duration_secs = effective_duration_secs(
        tier,
        MissionType::Construction(infra),
        target_layer,
        persistent,
    );
    let min_power = mission_power_threshold(target_layer, MissionType::Construction(infra));
    let cost = mission_launch_cost(MissionType::Construction(infra), target_layer);

    let construction = AvailableMission {
        mission_type: MissionType::Construction(infra),
        layer: target_layer,
        duration_secs,
        min_squad_power: min_power,
        required_archetype: None,
        recommended_archetype: None,
        marks_cost: cost,
        description: format!("Build {} on layer {}.", infra.display_name(), target_layer),
    };

    // Replace the lowest-priority mission, but never replace a Breakthrough or SupplyRun.
    if !prestige.available_missions.is_empty() {
        let worst_idx = prestige
            .available_missions
            .iter()
            .enumerate()
            .filter(|(_, m)| {
                !matches!(
                    m.mission_type,
                    MissionType::Breakthrough | MissionType::SupplyRun
                )
            })
            .max_by_key(|(_, m)| mission_priority_score(m.mission_type, strategy))
            .map(|(i, _)| i);
        if let Some(idx) = worst_idx {
            prestige.available_missions[idx] = construction;
        } else {
            // All missions are Breakthroughs or SupplyRuns — just append.
            prestige.available_missions.push(construction);
        }
    } else {
        prestige.available_missions.push(construction);
    }
}

/// Mission type name for priority matching.
fn mission_type_name(mt: MissionType) -> &'static str {
    match mt {
        MissionType::SupplyRun => "Supply Run",
        MissionType::Recon => "Recon",
        MissionType::Expedition => "Expedition",
        MissionType::Breakthrough => "Breakthrough",
        MissionType::GatewayExpedition => "Gateway Expedition",
        MissionType::Construction(_) => "Construction",
    }
}

/// Priority score for a mission type given the strategy.
fn mission_priority_score(mt: MissionType, strategy: DeepStrategy) -> usize {
    let name = mission_type_name(mt);
    let prio = strategy.mission_priority();
    prio.iter().position(|&n| n == name).unwrap_or(prio.len())
}

/// Try to launch missions according to strategy priorities.
#[allow(clippy::too_many_arguments)]
fn ai_launch_missions(
    persistent: &mut DeepPersistent,
    prestige: &mut DeepPrestige,
    strategy: DeepStrategy,
    stats: &mut DeepSimStats,
    now: DateTime<Utc>,
    rng: &mut ChaCha8Rng,
    verbose: bool,
    sim_start: DateTime<Utc>,
) {
    let concurrent_limit =
        effective_concurrent_missions(persistent.guild_rank, persistent.deepest_layer_reached);

    loop {
        let active_count = prestige.active_mission_count() as u32;
        if active_count >= concurrent_limit {
            break;
        }

        // Sort available missions by strategy priority.
        let mut indexed: Vec<(usize, usize)> = prestige
            .available_missions
            .iter()
            .enumerate()
            .map(|(i, m)| (i, mission_priority_score(m.mission_type, strategy)))
            .collect();
        indexed.sort_by_key(|&(_, score)| score);

        let mut launched = false;

        for &(idx, _) in &indexed {
            let avail = &prestige.available_missions[idx];

            // Check if we can afford it.
            let is_free = matches!(avail.mission_type, MissionType::SupplyRun)
                && is_daily_supply_run_available(Some(prestige.recruit_pool.refreshed_at), now);
            if !is_free && avail.marks_cost > prestige.warband_marks {
                continue;
            }

            // Pick a squad.
            let Some(squad) = pick_squad(avail, prestige) else {
                continue;
            };

            // Validate.
            if validate_squad_assignment(avail, &squad, prestige, persistent, is_free).is_err() {
                continue;
            }

            let avail_clone = prestige.available_missions.remove(idx);
            let cost = if is_free { 0 } else { avail_clone.marks_cost };

            let mission = start_mission(
                &avail_clone,
                &squad,
                prestige,
                persistent,
                is_free,
                now,
                rng,
            );

            stats.total_missions_launched += 1;
            stats.record_mission_type(mission.mission_type);
            if cost > 0 {
                stats.marks_spent_missions += cost as u64;
            }

            if verbose {
                let elapsed = (now - sim_start).num_minutes() as f64 / 60.0;
                let dur_h = (mission.ends_at - mission.started_at).num_seconds() as f64 / 3600.0;
                println!(
                    "  [{:6.1}h] Launched {} on L{} (squad power: {}, cost: {}, dur: {:.1}h)",
                    elapsed,
                    mission.mission_type.display_name(),
                    mission.layer,
                    squad
                        .iter()
                        .filter_map(|id| prestige.find_merc(*id))
                        .map(|m| m.effective_power())
                        .sum::<u32>(),
                    cost,
                    dur_h,
                );
            }

            // Add mission to active missions list so tick_all_missions can track it.
            prestige.active_missions.push(mission);

            launched = true;
            break;
        }

        if !launched {
            break;
        }
    }
}

// ── Simulation Loop ────────────────────────────────────────────────────────────

fn run_simulation(config: &DeepSimConfig, seed: u64) -> DeepSimStats {
    let sim_start = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    let sim_end = sim_start + Duration::hours(config.hours as i64);

    let (mut persistent, mut prestige, mut rng) = initialize_state(config, seed, sim_start);

    let mut stats = DeepSimStats::default();
    let mut now = sim_start;
    let mut last_snapshot_hour: i64 = -1;
    let mut stall_count: u32 = 0;

    while now < sim_end {
        // ── Hourly snapshot for CSV ──
        let current_hour = (now - sim_start).num_hours();
        if current_hour > last_snapshot_hour {
            last_snapshot_hour = current_hour;
            stats.hourly_snapshots.push(HourlySnapshot {
                sim_hour: current_hour as f64,
                marks: prestige.warband_marks,
                deepest_layer: persistent.deepest_layer_reached,
                layers_cleared: persistent.layers.iter().filter(|l| l.cleared).count() as u32,
                guild_rank: persistent.guild_rank.0,
                roster_size: prestige.roster.len(),
                available_mercs: prestige.available_merc_count(),
                active_missions: prestige.active_mission_count(),
                total_missions: stats.total_missions_launched,
                marks_earned: stats.total_marks_earned,
                marks_spent: stats.total_marks_spent(),
                mercs_lost: stats.mercs_lost,
                infrastructure: stats.infrastructure_built,
            });
        }

        // Track peak marks.
        if prestige.warband_marks > stats.peak_marks {
            stats.peak_marks = prestige.warband_marks;
        }

        // ── Phase A: Tick all missions ──
        let _tick_summary = tick_all_missions(&mut prestige, &mut persistent, now, &mut rng);

        // ── Phase B: Collect completed missions from pending_results ──
        let completed: Vec<_> = prestige.pending_results.drain(..).collect();
        for mission in &completed {
            if let Some(ref result) = mission.result {
                stats.record_outcome(&result.outcome);
                stats.total_marks_earned += result.marks_earned as u64;

                // Track injuries and losses.
                stats.injury_count += result.injured_mercs.len() as u32;
                stats.mercs_lost += result.lost_mercs.len() as u32;
                stats.level_ups += result.merc_level_ups.len() as u32;

                // Track max merc level.
                for &(merc_id, _) in &result.merc_level_ups {
                    if let Some(merc) = prestige.find_merc(merc_id) {
                        if merc.level > stats.max_merc_level {
                            stats.max_merc_level = merc.level;
                        }
                    }
                }

                // Handle Construction infrastructure building.
                if let MissionType::Construction(infra) = mission.mission_type {
                    if matches!(
                        result.outcome,
                        MissionOutcome::Success | MissionOutcome::PartialSuccess
                    ) {
                        let record = persistent.layer_record_mut(mission.layer);
                        if build_infrastructure(record, infra).is_ok() {
                            stats.infrastructure_built += 1;
                            let cost = infrastructure_build_cost(infra, mission.layer);
                            // Cost was already paid via mission marks_cost, so don't
                            // double-deduct; just track for stats if needed.
                            let _ = cost;
                        }
                    }
                }

                if config.verbose {
                    let elapsed_h = (now - sim_start).num_minutes() as f64 / 60.0;
                    println!(
                        "  [{:6.1}h] Completed {} on L{}: {:?} (+{} marks)",
                        elapsed_h,
                        mission.mission_type.display_name(),
                        mission.layer,
                        result.outcome,
                        result.marks_earned,
                    );
                }
            }
        }

        // Tick injuries for mercs who just completed missions.
        for merc in &mut prestige.roster {
            if matches!(merc.status, MercStatus::Injured { .. }) {
                tick_merc_injury(merc);
            }
        }

        // Purge lost mercs.
        purge_lost_mercs(&mut prestige.roster);

        // ── Phase C: AI decisions ──
        let elapsed_hours = (now - sim_start).num_minutes() as f64 / 60.0;

        // 1. Try guild rank upgrade.
        ai_upgrade_guild(
            &mut persistent,
            &mut prestige,
            config.strategy,
            &mut stats,
            elapsed_hours,
        );

        // 2. Try recruit mercs.
        ai_recruit(
            &mut persistent,
            &mut prestige,
            config.strategy,
            &mut stats,
            now,
            &mut rng,
        );

        // 2b. Emergency recruit: if roster is critically low (<=1 available merc)
        // and we can't afford normal recruitment, grant a free Common recruit so
        // the run doesn't permanently stall from early losses.
        let available_count = prestige.available_merc_count();
        if available_count <= 1
            && prestige.active_mission_count() == 0
            && roster_has_capacity(&prestige.roster, persistent.guild_rank)
        {
            let cheapest_recruit = 50; // Common minimum cost
            if prestige.warband_marks < cheapest_recruit {
                let id = persistent.next_merc_id();
                let archetype = MercArchetype::Vanguard; // Reliable frontliner
                let emergency = generate_mercenary(id, archetype, MercQuality::Common, &mut rng);
                prestige.roster.push(emergency);
                if config.verbose {
                    let elapsed_h = (now - sim_start).num_minutes() as f64 / 60.0;
                    println!(
                        "  [{:6.1}h] Emergency recruit: free Vanguard (roster critically low)",
                        elapsed_h,
                    );
                }
            }
        }

        // 3. Regenerate mission pool if empty or stale.
        if prestige.available_missions.is_empty() {
            prestige.available_missions = generate_mission_pool(&persistent, &mut rng);
        }
        ensure_breakthrough_in_pool(&persistent, &mut prestige, config.strategy);
        ensure_construction_in_pool(&persistent, &mut prestige, config.strategy);

        // 4. Try launch missions.
        let launched_before = stats.total_missions_launched;
        ai_launch_missions(
            &mut persistent,
            &mut prestige,
            config.strategy,
            &mut stats,
            now,
            &mut rng,
            config.verbose,
            sim_start,
        );
        let launched_this_tick = stats.total_missions_launched - launched_before;

        // If we couldn't launch anything and have no active missions,
        // refresh the pool — the current pool may be unlaunchable.
        if launched_this_tick == 0
            && prestige.active_mission_count() == 0
            && !prestige.available_missions.is_empty()
        {
            prestige.available_missions = generate_mission_pool(&persistent, &mut rng);
            ensure_breakthrough_in_pool(&persistent, &mut prestige, config.strategy);
            ensure_construction_in_pool(&persistent, &mut prestige, config.strategy);

            // Desperation fallback: if no mission in the pool is affordable, inject
            // a guaranteed cheap Layer 1 SupplyRun so the economy can always restart.
            let has_affordable = prestige.available_missions.iter().any(|m| {
                m.marks_cost <= prestige.warband_marks
                    && m.min_squad_power
                        <= prestige
                            .roster
                            .iter()
                            .map(|r| r.effective_power())
                            .sum::<u32>()
            });
            if !has_affordable {
                let tier = LayerTier::from_layer(1);
                let duration_secs =
                    effective_duration_secs(tier, MissionType::SupplyRun, 1, &persistent);
                let fallback = AvailableMission {
                    mission_type: MissionType::SupplyRun,
                    layer: 1,
                    duration_secs,
                    min_squad_power: 0, // Always launchable
                    required_archetype: None,
                    recommended_archetype: None,
                    marks_cost: 0, // Free desperation supply run
                    description: "Scavenge supplies from the shallowest tunnels.".to_string(),
                };
                prestige.available_missions.push(fallback);
            }
        }

        // Track layer progression.
        let cleared_count = persistent.layers.iter().filter(|l| l.cleared).count() as u32;
        if cleared_count > stats.layers_cleared {
            let newly_cleared = cleared_count - stats.layers_cleared;
            for _ in 0..newly_cleared {
                let layer = stats.layers_cleared + 1;
                stats.layer_clear_times.push((layer, elapsed_hours));
                stats.layers_cleared += 1;
            }
        }
        stats.deepest_layer = persistent.deepest_layer_reached;

        // ── Phase D: Advance time to next event ──
        let next = next_event_time(&prestige, now, sim_end);

        if next <= now {
            // Deadlock prevention: if all mercs injured and no missions active,
            // advance by 6 hours and tick injuries.
            stall_count += 1;
            if stall_count > 10 {
                now += Duration::hours(6);
                for merc in &mut prestige.roster {
                    if matches!(merc.status, MercStatus::Injured { .. }) {
                        tick_merc_injury(merc);
                    }
                }
                stall_count = 0;
                continue;
            }
            now += Duration::minutes(1);
        } else {
            stall_count = 0;
            now = next;
        }
    }

    // Final stats.
    stats.final_guild_rank = persistent.guild_rank.0;
    stats.deepest_layer = persistent.deepest_layer_reached;
    stats.layers_cleared = persistent.layers.iter().filter(|l| l.cleared).count() as u32;

    // Track max merc level from final roster.
    for merc in &prestige.roster {
        if merc.level > stats.max_merc_level {
            stats.max_merc_level = merc.level;
        }
    }

    stats
}

/// Compute the next event time: the earliest mission.ends_at among active missions.
fn next_event_time(
    prestige: &DeepPrestige,
    now: DateTime<Utc>,
    sim_end: DateTime<Utc>,
) -> DateTime<Utc> {
    let next_mission = prestige
        .active_missions
        .iter()
        .filter(|m| {
            matches!(
                m.status,
                MissionStatus::Active | MissionStatus::EventPending
            )
        })
        .map(|m| m.ends_at)
        .min();

    match next_mission {
        Some(t) if t > now => t,
        _ => {
            // No active missions: advance 1 minute to retry AI decisions.
            let advance = now + Duration::minutes(1);
            advance.min(sim_end)
        }
    }
}

// ── Output ─────────────────────────────────────────────────────────────────────

fn print_report(config: &DeepSimConfig, seed: u64, stats: &DeepSimStats) {
    let rank_name = match stats.final_guild_rank {
        1 => "Freelancers",
        2 => "Sellswords",
        3 => "Company",
        4 => "Battalion",
        5 => "Legion",
        _ => "Unknown",
    };

    println!(
        "\n\
         ══════════════════════════════════════════════════════\n\
         \x20 Deep Simulator  (seed={}, strategy={})\n\
         ══════════════════════════════════════════════════════\n",
        seed,
        config.strategy.name()
    );

    println!(
        "Duration: {:.1}h ({})  |  Starting: Rank {}, {} marks\n",
        config.hours as f64,
        format_duration_label(config.hours),
        config.guild_rank,
        config.starting_marks,
    );

    // Final State
    println!("── Final State ──");
    println!(
        "Guild Rank: {} ({})     Deepest Layer: {}",
        stats.final_guild_rank, rank_name, stats.deepest_layer
    );
    println!(
        "Layers Cleared: {}           Warband Marks: (see economy)",
        stats.layers_cleared,
    );
    println!();

    // Missions
    let total = stats.total_launched();
    let total_resolved = stats.successes + stats.partial_successes + stats.failures;
    println!("── Missions ──");
    println!(
        "Launched: {}  |  Success: {} ({:.0}%)  |  Partial: {} ({:.0}%)  |  Failed: {} ({:.0}%)",
        total,
        stats.successes,
        if total_resolved > 0 {
            stats.successes as f64 / total_resolved as f64 * 100.0
        } else {
            0.0
        },
        stats.partial_successes,
        if total_resolved > 0 {
            stats.partial_successes as f64 / total_resolved as f64 * 100.0
        } else {
            0.0
        },
        stats.failures,
        if total_resolved > 0 {
            stats.failures as f64 / total_resolved as f64 * 100.0
        } else {
            0.0
        },
    );
    println!(
        "  Supply: {}  Recon: {}  Expedition: {}  Breakthrough: {}  Construction: {}",
        stats.supply_runs,
        stats.recon_missions,
        stats.expedition_missions,
        stats.breakthrough_missions,
        stats.construction_missions,
    );
    println!();

    // Economy
    println!("── Economy ──");
    println!(
        "Earned: {} marks  |  Spent: {} marks",
        format_number(stats.total_marks_earned),
        format_number(stats.total_marks_spent()),
    );
    println!(
        "  Mission costs: {}  |  Recruitment: {}  |  Guild: {}  |  Peak: {}",
        format_number(stats.marks_spent_missions),
        format_number(stats.marks_spent_recruitment),
        format_number(stats.marks_spent_guild),
        format_number(stats.peak_marks as u64),
    );
    println!();

    // Roster
    println!("── Roster ──");
    println!(
        "Recruited: {}  |  Lost: {}  |  Injured: {}x  |  Level-ups: {}  |  Max level: {}",
        stats.mercs_recruited,
        stats.mercs_lost,
        stats.injury_count,
        stats.level_ups,
        stats.max_merc_level,
    );
    println!();

    // Infrastructure
    println!(
        "── Infrastructure ──\n  Built: {}\n",
        stats.infrastructure_built
    );

    // Layer Progression
    if !stats.layer_clear_times.is_empty() {
        println!("── Layer Progression ──");
        for chunk in stats.layer_clear_times.chunks(3) {
            let parts: Vec<String> = chunk
                .iter()
                .map(|(layer, hours)| format!("L{} @ {:6.1}h", layer, hours))
                .collect();
            println!("  {}", parts.join("  |  "));
        }
        println!();
    }

    // Guild Milestones
    if !stats.guild_rank_times.is_empty() {
        println!("── Guild Milestones ──");
        for (rank, hours) in &stats.guild_rank_times {
            let name = match rank {
                2 => "Sellswords",
                3 => "Company",
                4 => "Battalion",
                5 => "Legion",
                _ => "Unknown",
            };
            println!("  Rank {} ({}) @ {:.1}h", rank, name, hours);
        }
        println!();
    }
}

fn print_quiet_line(seed: u64, config: &DeepSimConfig, stats: &DeepSimStats) {
    println!(
        "seed={:<5} strategy={:<14} layers={:<3} rank={} missions={:<4} success={:.0}% marks_earned={:<6} lost={} infra={}",
        seed,
        config.strategy.name(),
        stats.layers_cleared,
        stats.final_guild_rank,
        stats.total_launched(),
        stats.success_rate(),
        stats.total_marks_earned,
        stats.mercs_lost,
        stats.infrastructure_built,
    );
}

fn print_multi_run_summary(all_stats: &[DeepSimStats]) {
    println!(
        "\n\
         ══════════════════════════════════════════════════════\n\
         \x20 Multi-Run Aggregate  ({} runs)\n\
         ══════════════════════════════════════════════════════\n",
        all_stats.len()
    );

    println!("{:<25} {:>8} {:>8} {:>8}", "Metric", "Min", "Avg", "Max");
    println!("{}", "─".repeat(53));

    macro_rules! stat_row {
        ($label:expr, $field:expr) => {{
            let values: Vec<f64> = all_stats.iter().map(|s| $field(s) as f64).collect();
            let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let avg = values.iter().sum::<f64>() / values.len() as f64;
            println!("{:<25} {:>8.1} {:>8.1} {:>8.1}", $label, min, avg, max);
        }};
    }

    stat_row!("Deepest Layer", |s: &DeepSimStats| s.deepest_layer);
    stat_row!("Layers Cleared", |s: &DeepSimStats| s.layers_cleared);
    stat_row!("Final Guild Rank", |s: &DeepSimStats| s.final_guild_rank);
    stat_row!("Missions Launched", |s: &DeepSimStats| s.total_launched());
    stat_row!("Success Rate (%)", |s: &DeepSimStats| s.success_rate()
        as u32);
    stat_row!("Marks Earned", |s: &DeepSimStats| s.total_marks_earned);
    stat_row!("Mercs Lost", |s: &DeepSimStats| s.mercs_lost);
    stat_row!("Infrastructure", |s: &DeepSimStats| s.infrastructure_built);
    println!();
}

fn write_csv(path: &str, stats: &DeepSimStats) {
    let mut file =
        std::fs::File::create(path).unwrap_or_else(|e| panic!("Cannot create {path}: {e}"));
    writeln!(
        file,
        "sim_hour,marks,deepest_layer,layers_cleared,guild_rank,roster_size,available_mercs,active_missions,total_missions,marks_earned,marks_spent,mercs_lost,infrastructure"
    )
    .unwrap();

    for snap in &stats.hourly_snapshots {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{},{},{},{},{}",
            snap.sim_hour,
            snap.marks,
            snap.deepest_layer,
            snap.layers_cleared,
            snap.guild_rank,
            snap.roster_size,
            snap.available_mercs,
            snap.active_missions,
            snap.total_missions,
            snap.marks_earned,
            snap.marks_spent,
            snap.mercs_lost,
            snap.infrastructure,
        )
        .unwrap();
    }
}

fn format_number(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{},{:03}", n / 1_000, n % 1_000)
    } else {
        format!("{}", n)
    }
}

fn format_duration_label(hours: u64) -> String {
    if hours.is_multiple_of(168) && hours >= 168 {
        let weeks = hours / 168;
        if weeks == 1 {
            "1 week".to_string()
        } else {
            format!("{weeks} weeks")
        }
    } else if hours.is_multiple_of(24) && hours >= 24 {
        let days = hours / 24;
        if days == 1 {
            "1 day".to_string()
        } else {
            format!("{days} days")
        }
    } else {
        format!("{hours} hours")
    }
}

// ── Main ───────────────────────────────────────────────────────────────────────

fn main() {
    let config = parse_args();

    if config.runs == 1 {
        let stats = run_simulation(&config, config.seed);
        if config.quiet {
            print_quiet_line(config.seed, &config, &stats);
        } else {
            print_report(&config, config.seed, &stats);
        }
        if let Some(ref path) = config.csv_path {
            write_csv(path, &stats);
            if !config.quiet {
                println!("CSV written to {path}");
            }
        }
    } else {
        let mut all_stats = Vec::with_capacity(config.runs as usize);

        for run_idx in 0..config.runs {
            let seed = config.seed + run_idx as u64;
            let stats = run_simulation(&config, seed);
            if config.quiet {
                print_quiet_line(seed, &config, &stats);
            } else if !config.quiet {
                print_report(&config, seed, &stats);
            }
            // CSV only for first run in multi-run.
            if run_idx == 0 {
                if let Some(ref path) = config.csv_path {
                    write_csv(path, &stats);
                }
            }
            all_stats.push(stats);
        }

        print_multi_run_summary(&all_stats);
    }
}
