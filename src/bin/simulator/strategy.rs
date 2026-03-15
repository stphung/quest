//! Strategy profiles and outcome injection for balance simulation.

use quest::achievements::milestones::{MinigameDifficulty, MinigameType};
use quest::achievements::Achievements;
use quest::ascension::logic::{ascend, can_ascend, AscendResult};
use quest::character::prestige_actions::{can_prestige, perform_prestige};
use quest::core::game_state::GameState;
use quest::deep::DeepState;
use quest::enhancement::EnhancementProgress;
use quest::haven::HavenRoomId;
use quest::loom::LoomState;
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

    /// Tick interval between Loom pattern completions.
    pub fn loom_pattern_interval_ticks(&self) -> u64 {
        match self {
            Self::Casual => 108_000,  // 3 hours per pattern
            Self::Optimal => 54_000,  // 1.5 hours per pattern
            Self::Speedrun => 27_000, // 45 min per pattern
        }
    }

    /// Maximum Loom patterns this strategy will complete.
    pub fn loom_max_patterns(&self) -> usize {
        match self {
            Self::Casual => 8,
            Self::Optimal => 22,
            Self::Speedrun => 28,
        }
    }

    /// PR threshold before Loom injection begins.
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
    pub last_zone_id: u32,
    pub deep_layers_injected: u32,
    pub last_deep_layer_tick: u64,
    pub deep_started: bool,
    pub loom_patterns_injected: usize,
    pub last_loom_pattern_tick: u64,
    pub loom_started: bool,
    /// Starting prestige rank (doesn't reset on auto-prestige).
    pub starting_prestige: u32,
}

impl InjectionState {
    pub fn new(starting_prestige: u32) -> Self {
        Self {
            last_challenge_tick: 0,
            enhancement_applied: false,
            sigils_etched: false,
            last_new_zone_tick: 0,
            last_zone_id: 1,
            deep_layers_injected: 0,
            last_deep_layer_tick: 0,
            deep_started: false,
            loom_patterns_injected: 0,
            last_loom_pattern_tick: 0,
            loom_started: false,
            starting_prestige,
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

    // -- Loom pattern completions --
    if effective_pr >= profile.loom_pr_threshold() {
        if !injection.loom_started {
            injection.loom_started = true;
            injection.last_loom_pattern_tick = tick;
            quest::loom::complete_discovery(loom);
        }

        if injection.loom_patterns_injected < profile.loom_max_patterns()
            && tick - injection.last_loom_pattern_tick >= profile.loom_pattern_interval_ticks()
        {
            let idx = injection.loom_patterns_injected;
            injection.loom_patterns_injected += 1;
            injection.last_loom_pattern_tick = tick;

            // Mark pattern and all its requirements as completed
            if idx < loom.persistent.patterns.len() {
                for req in &mut loom.persistent.patterns[idx].requirements {
                    req.completed = true;
                    req.sustained_secs = req.sustain_duration_secs;
                }
                loom.persistent.patterns[idx].completed = true;
            }

            let completed = loom.persistent.completed_pattern_count();

            // Sync zone unlocks with new pattern count
            let storms_end_unlocked =
                achievements.is_unlocked(quest::achievements::AchievementId::TheStormbreaker);
            let loom_zone_cap = quest::loom::loom_zone_cap_for_patterns(completed);
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
                    "[t={tick:>6}] INJECT: Loom pattern {} completed ({completed}/{} total, zone cap -> Z{})",
                    idx + 1,
                    loom.persistent.patterns.len(),
                    loom_zone_cap
                );
            }
        }
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
    let current_zone = state.zone_progression.current_zone_id;
    if current_zone != injection.last_zone_id {
        injection.last_new_zone_tick = tick;
        injection.last_zone_id = current_zone;
    }

    if tick - injection.last_new_zone_tick >= profile.prestige_stuck_ticks() && can_prestige(state)
    {
        perform_prestige(state);
        injection.last_new_zone_tick = tick;
        injection.last_zone_id = state.zone_progression.current_zone_id;
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

/// Etch sigils per profile spec.
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
