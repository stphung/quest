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
use quest::zones::sync_account_zone_unlocks;

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
}

/// Mutable state used by inject_outcomes() across ticks.
pub struct InjectionState {
    pub last_challenge_tick: u64,
    pub enhancement_applied: bool,
    pub sigils_etched: bool,
    pub last_new_zone_tick: u64,
    pub last_zone_id: u32,
}

impl InjectionState {
    pub fn new() -> Self {
        Self {
            last_challenge_tick: 0,
            enhancement_applied: false,
            sigils_etched: false,
            last_new_zone_tick: 0,
            last_zone_id: 1,
        }
    }
}

/// Run outcome injection after each game tick.
/// Checks milestone triggers per profile and mutates state directly.
pub fn inject_outcomes(
    profile: &StrategyProfile,
    state: &mut GameState,
    enhancement: &mut EnhancementProgress,
    deep: &mut DeepState,
    achievements: &mut Achievements,
    loom: &LoomState,
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
