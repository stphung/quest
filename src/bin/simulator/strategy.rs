//! Strategy profiles and outcome injection for balance simulation.

use quest::achievements::milestones::{MinigameDifficulty, MinigameType};
use quest::achievements::Achievements;
use quest::ascension::logic::{ascend, can_ascend, AscendResult};
use quest::character::prestige_actions::{can_prestige, perform_prestige};
use quest::core::game_state::GameState;
use quest::deep::DeepState;
use quest::enhancement::EnhancementProgress;
use quest::haven::HavenRoomId;
use quest::loom::{
    build_shuttle, eligible_sources_for_tier, try_upgrade_node, upgrade_shuttle, LoomState, NodeId,
    Resource,
};
use quest::stormglass::sigils::{roll_sigil, SigilEffectType, SigilGrade};
use quest::zones::{sync_account_zone_unlocks, FractureRegion};

#[derive(Debug, Clone, Copy)]
pub enum StrategyProfile {
    Casual,
    Optimal,
    Speedrun,
}

impl StrategyProfile {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "casual" => Some(Self::Casual),
            "optimal" => Some(Self::Optimal),
            "speedrun" => Some(Self::Speedrun),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Casual => "casual",
            Self::Optimal => "optimal",
            Self::Speedrun => "speedrun",
        }
    }

    pub fn haven_priority(&self) -> &'static [HavenRoomId] {
        match self {
            Self::Casual => &[
                HavenRoomId::Hearthstone,
                HavenRoomId::Bedroom,
                HavenRoomId::Garden,
                HavenRoomId::Library,
                HavenRoomId::FishingDock,
                HavenRoomId::Workshop,
                HavenRoomId::Vault,
            ],
            Self::Optimal => &[
                HavenRoomId::Hearthstone,
                HavenRoomId::Armory,
                HavenRoomId::Bedroom,
                HavenRoomId::TrainingYard,
                HavenRoomId::Garden,
                HavenRoomId::TrophyHall,
                HavenRoomId::Library,
                HavenRoomId::Watchtower,
                HavenRoomId::FishingDock,
                HavenRoomId::AlchemyLab,
                HavenRoomId::Workshop,
                HavenRoomId::WarRoom,
                HavenRoomId::Vault,
            ],
            Self::Speedrun => &[
                HavenRoomId::Hearthstone,
                HavenRoomId::Armory,
                HavenRoomId::Bedroom,
                HavenRoomId::TrainingYard,
                HavenRoomId::Garden,
                HavenRoomId::TrophyHall,
                HavenRoomId::Library,
                HavenRoomId::Watchtower,
                HavenRoomId::FishingDock,
                HavenRoomId::AlchemyLab,
                HavenRoomId::Workshop,
                HavenRoomId::WarRoom,
                HavenRoomId::Vault,
                HavenRoomId::StormForge,
            ],
        }
    }

    pub fn challenge_interval_ticks(&self) -> u64 {
        match self {
            Self::Casual => 36_000,  // 1 per hour
            Self::Optimal => 12_000, // 3 per hour
            Self::Speedrun => 6_000, // 6 per hour
        }
    }

    pub fn enhancement_pr_threshold(&self) -> u32 {
        match self {
            Self::Casual => 20,
            Self::Optimal => 15,
            Self::Speedrun => 10,
        }
    }

    pub fn enhancement_target_level(&self) -> u8 {
        match self {
            Self::Casual => 4,
            Self::Optimal => 7,
            Self::Speedrun => 10,
        }
    }

    pub fn stormglass_threshold(&self) -> u64 {
        match self {
            Self::Casual => 5000,
            Self::Optimal => 3000,
            Self::Speedrun => 1500,
        }
    }

    pub fn stormglass_cost(&self) -> u64 {
        match self {
            Self::Casual => 2000,
            Self::Optimal => 3000,
            Self::Speedrun => 4000,
        }
    }

    pub fn prestige_stuck_ticks(&self) -> u64 {
        match self {
            Self::Casual => 18_000,
            Self::Optimal => 9_000,
            Self::Speedrun => 3_600,
        }
    }

    /// Tick interval between Deep layer breakthroughs.
    /// Deep missions take hours in real-time; we compress to tick intervals.
    pub fn deep_layer_interval_ticks(&self) -> u64 {
        match self {
            Self::Casual => 72_000,   // 2 hours per layer
            Self::Optimal => 36_000,  // 1 hour per layer
            Self::Speedrun => 18_000, // 30 min per layer
        }
    }

    /// Maximum Deep layer this strategy profile will reach.
    pub fn deep_max_layer(&self) -> u32 {
        match self {
            Self::Casual => 12,
            Self::Optimal => 25,
            Self::Speedrun => 30,
        }
    }

    /// PR threshold before Deep injection begins (simulates P15+ discovery gate).
    pub fn deep_pr_threshold(&self) -> u32 {
        match self {
            Self::Casual => 20,
            Self::Optimal => 15,
            Self::Speedrun => 15,
        }
    }

    /// PR threshold before Loom auto-build begins.
    pub fn loom_pr_threshold(&self) -> u32 {
        match self {
            Self::Casual => 25,
            Self::Optimal => 20,
            Self::Speedrun => 15,
        }
    }
}

/// Mutable state used by inject_outcomes() across ticks.
pub struct InjectionState {
    pub last_challenge_tick: u64,
    pub enhancement_applied: bool,
    pub sigils_etched: bool,
    pub last_new_zone_tick: u64,
    /// Highest zone reached since the last prestige (frontier death loops
    /// bounce between zones, so any-zone-change is not a progress signal).
    pub max_zone_id: u32,
    pub deep_layers_injected: u32,
    pub last_deep_layer_tick: u64,
    pub deep_started: bool,
    pub loom_started: bool,
    /// Tick at which the last Loom pattern was completed (for pacing injection).
    pub last_loom_pattern_tick: u64,
    /// Starting prestige rank (doesn't reset on auto-prestige).
    pub starting_prestige: u32,
    /// Number of auto-prestiges performed (PR balance can be spent back to 0,
    /// so this is the only reliable record that the prestige loop cycled).
    pub prestige_count: u32,
}

impl InjectionState {
    pub fn new(starting_prestige: u32) -> Self {
        Self {
            last_challenge_tick: 0,
            enhancement_applied: false,
            sigils_etched: false,
            last_new_zone_tick: 0,
            max_zone_id: 1,
            deep_layers_injected: 0,
            last_deep_layer_tick: 0,
            deep_started: false,
            loom_started: false,
            last_loom_pattern_tick: 0,
            starting_prestige,
            prestige_count: 0,
        }
    }
}

/// Run outcome injection after each game tick.
/// Checks milestone triggers per profile and mutates state directly.
#[allow(clippy::too_many_arguments)]
pub fn inject_outcomes(
    profile: &StrategyProfile,
    state: &mut GameState,
    enhancement: &mut EnhancementProgress,
    deep: &mut DeepState,
    achievements: &mut Achievements,
    loom: &mut LoomState,
    injection: &mut InjectionState,
    tick: u64,
    verbose: bool,
) {
    // -- Challenge wins --
    if tick > 0 && tick - injection.last_challenge_tick >= profile.challenge_interval_ticks() {
        let game_types = [
            MinigameType::Chess,
            MinigameType::Minesweeper,
            MinigameType::Rune,
            MinigameType::Go,
        ];
        let idx = ((tick / profile.challenge_interval_ticks()) as usize) % game_types.len();
        let game_type = game_types[idx];
        let difficulty = match profile {
            StrategyProfile::Casual => MinigameDifficulty::Novice,
            StrategyProfile::Optimal => MinigameDifficulty::Journeyman,
            StrategyProfile::Speedrun => MinigameDifficulty::Master,
        };

        achievements.on_minigame_won(game_type, difficulty, Some("Simulator"));

        let reward = challenge_reward_for_difficulty(&difficulty);
        state.prestige_rank += reward.0;
        state.stormglass += reward.1;

        injection.last_challenge_tick = tick;

        if verbose {
            println!(
                "[t={tick:>6}] INJECT: Challenge win ({game_type:?}/{difficulty:?}) +{} PR +{} SG",
                reward.0, reward.1
            );
        }
    }

    // -- Soulforge enhancement --
    if !injection.enhancement_applied && state.prestige_rank >= profile.enhancement_pr_threshold() {
        let target = profile.enhancement_target_level();
        for slot in 0..7 {
            enhancement.levels[slot] = target;
        }
        injection.enhancement_applied = true;
        state.invalidate_bonuses();

        if verbose {
            println!("[t={tick:>6}] INJECT: Enhancement all slots -> +{target}");
        }
    }

    // -- Stormglass sigil spending --
    if !injection.sigils_etched && state.stormglass >= profile.stormglass_threshold() {
        inject_sigils(profile, state);
        injection.sigils_etched = true;

        if verbose {
            println!(
                "[t={tick:>6}] INJECT: Sigils etched ({} profile), {} SG deducted",
                profile.name(),
                profile.stormglass_cost()
            );
        }
    }

    // -- Deep layer breakthroughs --
    // Use starting prestige (not current, which resets on auto-prestige)
    let effective_pr = injection.starting_prestige.max(state.prestige_rank);
    if effective_pr >= profile.deep_pr_threshold() {
        if !injection.deep_started {
            injection.deep_started = true;
            injection.last_deep_layer_tick = tick;
            deep.persistent.discovered = true;
        }

        if injection.deep_layers_injected < profile.deep_max_layer()
            && tick - injection.last_deep_layer_tick >= profile.deep_layer_interval_ticks()
        {
            injection.deep_layers_injected += 1;
            let layer = injection.deep_layers_injected;
            injection.last_deep_layer_tick = tick;

            deep.persistent.deepest_layer_reached = layer;

            // Check if this layer unlocks a fracture region
            if let Some(region) = FractureRegion::from_layer(layer) {
                let new_cap = region.end_zone_id();
                if new_cap > deep.persistent.fracture_zone_cap {
                    deep.persistent.fracture_zone_cap = new_cap;
                }
            }

            // Sync zone unlocks with new fracture cap
            let storms_end_unlocked =
                achievements.is_unlocked(quest::achievements::AchievementId::TheStormbreaker);
            let loom_zone_cap =
                quest::loom::loom_zone_cap_for_patterns(loom.persistent.completed_pattern_count());
            sync_account_zone_unlocks(
                &mut state.zone_progression,
                storms_end_unlocked,
                deep.persistent.fracture_zone_cap,
                state.prestige_rank,
                loom_zone_cap,
                state.ascension_level,
            );
            state.invalidate_bonuses();

            if verbose {
                println!(
                    "[t={tick:>6}] INJECT: Deep L{layer} cleared (fracture cap -> Z{})",
                    deep.persistent.fracture_zone_cap
                );
            }
        }
    }

    // -- Loom auto-build --
    if effective_pr >= profile.loom_pr_threshold() {
        if !injection.loom_started {
            injection.loom_started = true;
            quest::loom::complete_discovery(loom);
            // Force-unlock all 6 extractors immediately (skip neighbor unlock cascade)
            for node in &mut loom.persistent.nodes {
                node.unlocked = true;
            }
            // Seed extractor buffers for shuttle construction costs
            for node in &mut loom.persistent.nodes {
                node.buffer = node.buffer_capacity;
            }
            if verbose {
                println!("[t={tick:>6}] INJECT: Loom discovered, all extractors unlocked");
            }
        }

        auto_build_loom(loom, state.ascension_level, verbose, tick);

        // -- Loom pattern completion injection --
        // The Loom uses wall-clock time for pattern sustain, which doesn't advance
        // meaningfully in the headless simulator. Inject pattern completions at a
        // pace matching the pattern's sustain duration compressed to tick intervals.
        inject_pattern_completions(profile, loom, state, achievements, injection, tick, verbose);
    }

    // -- Ascension --
    let deepest_layer = deep.persistent.deepest_layer_reached;
    let completed_patterns = loom.persistent.completed_pattern_count();
    if can_ascend(
        state.ascension_level,
        state.prestige_rank,
        deepest_layer,
        completed_patterns,
    ) {
        let result = ascend(state, deepest_layer, completed_patterns);
        if let AscendResult::Success {
            new_level,
            multiplier,
        } = &result
        {
            let storms_end_unlocked =
                achievements.is_unlocked(quest::achievements::AchievementId::TheStormbreaker);
            let loom_zone_cap =
                quest::loom::loom_zone_cap_for_patterns(loom.persistent.completed_pattern_count());
            sync_account_zone_unlocks(
                &mut state.zone_progression,
                storms_end_unlocked,
                deep.persistent.fracture_zone_cap,
                state.prestige_rank,
                loom_zone_cap,
                state.ascension_level,
            );
            state.invalidate_bonuses();

            if verbose {
                println!("[t={tick:>6}] INJECT: Ascension -> level {new_level} ({multiplier:.0}x)");
            }
        }
    }

    // -- Auto-prestige (stuck detection) --
    // Progress means reaching a NEW highest zone. A frontier death loop bounces
    // between the frontier and its safe zone; treating any zone change as
    // progress would starve the stuck timer forever (#576).
    let current_zone = state.zone_progression.current_zone_id;
    if current_zone > injection.max_zone_id {
        injection.last_new_zone_tick = tick;
        injection.max_zone_id = current_zone;
    }

    if tick - injection.last_new_zone_tick >= profile.prestige_stuck_ticks() && can_prestige(state)
    {
        perform_prestige(state);
        injection.prestige_count += 1;
        injection.last_new_zone_tick = tick;
        injection.max_zone_id = state.zone_progression.current_zone_id;
        state.invalidate_bonuses();

        if verbose {
            println!(
                "[t={tick:>6}] INJECT: Auto-prestige -> P{}",
                state.prestige_rank
            );
        }
    }
}

/// Challenge rewards by difficulty. PR values reflect that most game
/// types award 0 PR at lower difficulties; we use conservative averages.
fn challenge_reward_for_difficulty(diff: &MinigameDifficulty) -> (u32, u64) {
    match diff {
        MinigameDifficulty::Novice => (0, 500),
        MinigameDifficulty::Apprentice => (0, 1500),
        MinigameDifficulty::Journeyman => (1, 3000),
        MinigameDifficulty::Master => (1, 6000),
    }
}

// ── Loom Auto-Build ──────────────────────────────────────────────────────

/// Best recipe to produce each non-base resource, preferring highest yield at lowest tier.
fn best_recipe_for(
    resource: Resource,
    completed_patterns: usize,
) -> Option<quest::loom::recipes::Recipe> {
    let mut candidates = quest::loom::recipes::recipes_producing(resource);
    // Filter to unlocked tiers (T1=1 pattern, T2=8, T3=15)
    candidates.retain(|r| completed_patterns >= tier_unlock_threshold(r.tier));
    // Prefer lowest tier first (simpler chains), then highest amount
    candidates.sort_by(|a, b| {
        a.tier.cmp(&b.tier).then(
            b.amount
                .partial_cmp(&a.amount)
                .unwrap_or(std::cmp::Ordering::Equal),
        )
    });
    candidates.into_iter().next()
}

fn tier_unlock_threshold(tier: u8) -> usize {
    match tier {
        1 => 1,
        2 => 8,
        _ => 15,
    }
}

fn is_base_resource(r: Resource) -> bool {
    matches!(
        r,
        Resource::Ember
            | Resource::Reflection
            | Resource::VoidEssence
            | Resource::Memory
            | Resource::Silence
            | Resource::Resonance
    )
}

/// Returns true if there's already a shuttle producing this resource (not under construction).
fn has_shuttle_producing(loom: &LoomState, resource: Resource) -> bool {
    loom.persistent
        .shuttles
        .iter()
        .any(|s| s.output == resource && !s.under_construction)
}

/// Count shuttles producing a specific resource.
fn shuttle_count_for(loom: &LoomState, resource: Resource) -> usize {
    loom.persistent
        .shuttles
        .iter()
        .filter(|s| s.output == resource)
        .count()
}

/// Ensure a shuttle chain exists to produce the given resource.
/// Recursively builds prerequisite shuttles for confluence inputs.
/// Returns true if a shuttle was built this call.
#[allow(clippy::only_used_in_recursion)]
fn ensure_resource_production(
    loom: &mut LoomState,
    resource: Resource,
    ascension_level: u32,
    verbose: bool,
    tick: u64,
) -> bool {
    if is_base_resource(resource) {
        return false;
    }
    if has_shuttle_producing(loom, resource) {
        return false;
    }

    let completed = loom.persistent.completed_pattern_count();
    let max_shuttles = completed.max(1); // at least 1 after discovery
    if loom.persistent.shuttles.len() >= max_shuttles {
        return false;
    }

    let recipe = match best_recipe_for(resource, completed) {
        Some(r) => r,
        None => return false,
    };

    // Recursively ensure inputs are available (for T2/T3 recipes needing confluences)
    if !is_base_resource(recipe.input_a) {
        ensure_resource_production(loom, recipe.input_a, ascension_level, verbose, tick);
    }
    if !is_base_resource(recipe.input_b) {
        ensure_resource_production(loom, recipe.input_b, ascension_level, verbose, tick);
    }

    // Check shuttle capacity again (recursive calls may have built some)
    let completed = loom.persistent.completed_pattern_count();
    let max_shuttles = completed.max(1);
    if loom.persistent.shuttles.len() >= max_shuttles {
        return false;
    }

    // Ensure buffer has build cost
    let cost = match recipe.tier {
        1 => 250.0,
        2 => 150.0,
        _ => 100.0,
    };
    inject_resource_to_buffer(loom, recipe.input_a, cost);

    let sources_a = eligible_sources_for_tier(loom, recipe.tier, recipe.input_a);
    let sources_b = eligible_sources_for_tier(loom, recipe.tier, recipe.input_b);

    if sources_a.is_empty() || sources_b.is_empty() {
        return false;
    }

    match build_shuttle(
        loom,
        recipe.input_a,
        recipe.input_b,
        recipe.node_nature,
        sources_a,
        sources_b,
    ) {
        Ok(idx) => {
            // Skip construction delay — make shuttle immediately operational
            let shuttle = &mut loom.persistent.shuttles[idx];
            shuttle.under_construction = false;
            shuttle.construction_secs_remaining = 0.0;

            if verbose {
                println!(
                    "[t={tick:>6}] LOOM: Built T{} shuttle #{idx} ({:?}+{:?}->{:?}, amount={:.1})",
                    recipe.tier, recipe.input_a, recipe.input_b, recipe.output, recipe.amount
                );
            }
            true
        }
        Err(_) => false,
    }
}

/// Auto-build Loom infrastructure: build shuttles for active pattern requirements,
/// upgrade extractors and shuttles for throughput.
fn auto_build_loom(loom: &mut LoomState, ascension_level: u32, verbose: bool, tick: u64) {
    if !loom.persistent.discovered {
        return;
    }

    // Only run auto-build every 100 ticks (10 seconds) to avoid per-tick overhead
    if !tick.is_multiple_of(100) {
        return;
    }

    let active_idx = loom.persistent.active_pattern;
    if active_idx >= loom.persistent.patterns.len() {
        return;
    }
    if loom.persistent.patterns[active_idx].completed {
        return;
    }

    // Collect required resources from active pattern
    let needed: Vec<Resource> = loom.persistent.patterns[active_idx]
        .requirements
        .iter()
        .filter(|r| !r.completed)
        .map(|r| r.resource)
        .collect();

    // Ensure production chains exist for each needed resource
    for resource in &needed {
        ensure_resource_production(loom, *resource, ascension_level, verbose, tick);
    }

    // For high-rate requirements, build additional shuttles for the same resource.
    // Collect requirement data first to avoid borrow conflict.
    let high_rate_needs: Vec<(Resource, f64)> = loom.persistent.patterns[active_idx]
        .requirements
        .iter()
        .filter(|r| !r.completed && !is_base_resource(r.resource) && r.required_rate > 20.0)
        .map(|r| (r.resource, r.required_rate))
        .collect();

    for (resource, required_rate) in high_rate_needs {
        let completed = loom.persistent.completed_pattern_count();
        let max_shuttles = completed.max(1);
        let current_count = shuttle_count_for(loom, resource);
        if current_count < 3 && loom.persistent.shuttles.len() < max_shuttles {
            if let Some(recipe) = best_recipe_for(resource, completed) {
                let cost = match recipe.tier {
                    1 => 250.0,
                    2 => 150.0,
                    _ => 100.0,
                };
                inject_resource_to_buffer(loom, recipe.input_a, cost);
                let sources_a = eligible_sources_for_tier(loom, recipe.tier, recipe.input_a);
                let sources_b = eligible_sources_for_tier(loom, recipe.tier, recipe.input_b);
                if !sources_a.is_empty() && !sources_b.is_empty() {
                    if let Ok(idx) = build_shuttle(
                        loom,
                        recipe.input_a,
                        recipe.input_b,
                        recipe.node_nature,
                        sources_a,
                        sources_b,
                    ) {
                        let shuttle = &mut loom.persistent.shuttles[idx];
                        shuttle.under_construction = false;
                        shuttle.construction_secs_remaining = 0.0;
                        if verbose {
                            println!(
                                "[t={tick:>6}] LOOM: Built extra T{} shuttle #{idx} for {:?} (rate target: {:.0}/hr)",
                                recipe.tier, recipe.output, required_rate
                            );
                        }
                    }
                }
            }
        }
    }

    // Upgrade extractors — top up buffer to cover cost when buffer_capacity is too small.
    // Cap at level 20 (525/hr) — sufficient for all pattern requirements.
    for node_id in [
        NodeId::EmberSpindle,
        NodeId::ReflectionLens,
        NodeId::VoidCondenser,
        NodeId::MemoryArchive,
        NodeId::SilenceWell,
        NodeId::ResonanceForge,
    ] {
        let node_level = loom.persistent.nodes[node_id.index()].level;
        if node_level >= 20 {
            continue;
        }
        let cost = quest::loom::node_upgrade_cost(loom, node_id);
        let node = &mut loom.persistent.nodes[node_id.index()];
        if node.unlocked && node.buffer < cost {
            // Inject resources to cover upgrade cost (simulates patient accumulation)
            node.buffer = cost;
        }
        if try_upgrade_node(loom, node_id) && verbose {
            let node = &loom.persistent.nodes[node_id.index()];
            println!(
                "[t={tick:>6}] LOOM: Upgraded {:?} to level {} ({:.0}/hr)",
                node_id,
                node.level,
                node.base_rate * quest::loom::node_level_multiplier(node.level)
            );
        }
    }

    // Upgrade shuttles — top up buffer to cover cost
    let max_level = quest::ascension::types::max_shuttle_level(ascension_level);
    for i in 0..loom.persistent.shuttles.len() {
        if max_level > 1 {
            let shuttle = &mut loom.persistent.shuttles[i];
            if !shuttle.under_construction && shuttle.level < max_level {
                let cost = 100.0 * (shuttle.level as f64).powf(1.2);
                if shuttle.buffer < cost {
                    shuttle.buffer = cost;
                }
            }
        }
        if upgrade_shuttle(loom, i, ascension_level).is_ok() && verbose {
            let shuttle = &loom.persistent.shuttles[i];
            println!(
                "[t={tick:>6}] LOOM: Upgraded shuttle #{i} ({:?}) to level {}",
                shuttle.output, shuttle.level
            );
        }
    }
}

/// Inject pattern completions to compensate for the Loom's wall-clock time model.
///
/// The Loom uses `chrono::Utc::now()` for production rates and pattern sustain timers,
/// which means it barely progresses in a headless simulator where ticks run at CPU speed.
/// This function directly completes patterns at a pace derived from each pattern's
/// sustain duration, compressed into tick intervals.
fn inject_pattern_completions(
    profile: &StrategyProfile,
    loom: &mut LoomState,
    state: &mut GameState,
    achievements: &mut Achievements,
    injection: &mut InjectionState,
    tick: u64,
    verbose: bool,
) {
    if !loom.persistent.discovered {
        return;
    }

    let active_idx = loom.persistent.active_pattern;
    if active_idx >= loom.persistent.patterns.len() {
        return;
    }
    if loom.persistent.patterns[active_idx].completed {
        return;
    }

    // Determine the tick interval for completing this pattern.
    // Each pattern's sustain duration (in hours) is compressed to a tick interval.
    // The compression rate varies by strategy profile.
    let max_sustain_hours = loom.persistent.patterns[active_idx]
        .requirements
        .iter()
        .filter(|r| !r.completed)
        .map(|r| r.sustain_duration_secs / 3600.0)
        .fold(0.0_f64, f64::max);

    // Ticks per game-hour: 36,000 ticks = 1 hour of game time.
    // We compress the pattern sustain time by a factor that varies by strategy.
    let compression_factor = match profile {
        StrategyProfile::Casual => 3.0,   // 3x slower than real time
        StrategyProfile::Optimal => 1.5,  // 1.5x slower than real time
        StrategyProfile::Speedrun => 1.0, // Real-time pace
    };
    let ticks_for_pattern = (max_sustain_hours * 36_000.0 * compression_factor) as u64;
    let min_ticks = 3_600; // At least 6 minutes of game time per pattern

    let interval = ticks_for_pattern.max(min_ticks);

    if injection.last_loom_pattern_tick == 0 {
        // First pattern — start the clock from Loom discovery.
        injection.last_loom_pattern_tick = tick;
        return;
    }

    if tick - injection.last_loom_pattern_tick < interval {
        return;
    }

    // Complete the active pattern.
    let pattern = &mut loom.persistent.patterns[active_idx];
    for req in &mut pattern.requirements {
        req.sustained_secs = req.sustain_duration_secs;
        req.completed = true;
    }
    pattern.completed = true;

    // Advance to next pattern.
    let next = active_idx + 1;
    if next < loom.persistent.patterns.len() {
        loom.persistent.active_pattern = next;
    }

    injection.last_loom_pattern_tick = tick;

    let completed_count = loom.persistent.completed_pattern_count();
    achievements.on_loom_pattern_completed(completed_count, Some("Simulator"));

    // Sync zone unlocks.
    let loom_zone_cap = quest::loom::loom_zone_cap_for_patterns(completed_count);
    state.cached_loom_zone_cap = loom_zone_cap;
    let storms_end_unlocked =
        achievements.is_unlocked(quest::achievements::AchievementId::TheStormbreaker);
    sync_account_zone_unlocks(
        &mut state.zone_progression,
        storms_end_unlocked,
        state.cached_fracture_zone_cap,
        state.prestige_rank,
        loom_zone_cap,
        state.ascension_level,
    );
    state.invalidate_bonuses();

    if verbose {
        println!(
            "[t={tick:>6}] INJECT: Loom pattern #{active_idx} '{}' completed ({completed_count}/28, zone cap Z{loom_zone_cap})",
            loom.persistent.patterns[active_idx].name
        );
    }
}

/// Etch sigils per profile spec.
/// Ensure a resource buffer has at least `amount` available (for simulator injection).
fn inject_resource_to_buffer(loom: &mut LoomState, resource: Resource, amount: f64) {
    // Try extractor first.
    for node in &mut loom.persistent.nodes {
        if node.unlocked && quest::loom::logic::node_native_resource(node.id) == resource {
            if node.buffer < amount {
                node.buffer = amount.min(node.buffer_capacity);
            }
            return;
        }
    }
    // Try shuttle buffer.
    for shuttle in &mut loom.persistent.shuttles {
        if !shuttle.under_construction && shuttle.output == resource {
            if shuttle.buffer < amount {
                shuttle.buffer = amount.min(shuttle.buffer_capacity);
            }
            return;
        }
    }
}

fn inject_sigils(profile: &StrategyProfile, state: &mut GameState) {
    use SigilEffectType::*;
    use SigilGrade::*;

    let (effects, grade): (&[SigilEffectType], SigilGrade) = match profile {
        StrategyProfile::Casual => (&[DamagePercent, DamageReductionPercent], C),
        StrategyProfile::Optimal => (
            &[DamagePercent, DamageReductionPercent, CritChancePercent],
            A,
        ),
        StrategyProfile::Speedrun => (&[DamagePercent, CritChancePercent, MaxHpPercent], SPlus),
    };

    let needed = effects.len() as u8;
    if state.storm_sigils.slots_unlocked < needed {
        state.storm_sigils.slots_unlocked = needed;
    }

    while state.storm_sigils.sigils.len() < needed as usize {
        state.storm_sigils.sigils.push(None);
    }

    let uniform_roll = match grade {
        C => 0.50,
        A => 0.85,
        SPlus => 0.99,
        _ => 0.50,
    };

    for (i, &effect) in effects.iter().enumerate() {
        let mut sigil = roll_sigil(effect, uniform_roll);
        sigil.grade = grade;
        state.storm_sigils.sigils[i] = Some(sigil);
    }

    state.stormglass = state.stormglass.saturating_sub(profile.stormglass_cost());
}
