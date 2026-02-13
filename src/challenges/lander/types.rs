//! Lunar Lander ("Lunar Descent") data structures.
//!
//! A real-time action minigame where the player lands a spacecraft on a pad
//! by controlling rotation and thrust against gravity, with limited fuel.

use rand::Rng;
use serde::{Deserialize, Serialize};

/// Difficulty levels for Lunar Lander.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LanderDifficulty {
    Novice,
    Apprentice,
    Journeyman,
    Master,
}

difficulty_enum_impl!(LanderDifficulty);

/// Game area dimensions.
pub const GAME_WIDTH: u16 = 60;
pub const GAME_HEIGHT: u16 = 30;

/// Number of terrain points across the width.
pub const TERRAIN_POINTS: usize = 61; // 0..=GAME_WIDTH inclusive

/// Physics tick interval in milliseconds (~60 FPS).
pub const PHYSICS_TICK_MS: u64 = 16;

/// Maximum safe landing vertical velocity (positive = downward).
pub const MAX_LANDING_VY: f64 = 0.08;

/// Maximum safe landing horizontal velocity (absolute value).
pub const MAX_LANDING_VX: f64 = 0.04;

/// Maximum safe landing angle deviation from vertical (in radians).
/// ~15 degrees.
pub const MAX_LANDING_ANGLE: f64 = 0.26;

/// Thrust acceleration magnitude per physics tick.
pub const THRUST_POWER: f64 = 0.02;

/// Rotation speed in radians per physics tick.
pub const ROTATION_SPEED: f64 = 0.04;

/// Fuel consumption per physics tick while thrusting.
pub const FUEL_BURN_RATE: f64 = 0.15;

/// Physics ticks to hold an input flag after a key press (~200ms).
/// Bridges the gap between terminal key-repeat events so holding a key
/// feels continuous rather than stuttery.
pub const INPUT_HOLD_TICKS: u32 = 12;

/// Thrust flame animation duration in physics ticks.
pub const FLAME_ANIM_TICKS: u32 = 4;

impl LanderDifficulty {
    /// Gravity acceleration (downward velocity increase per 16ms tick).
    pub fn gravity(&self) -> f64 {
        match self {
            Self::Novice => 0.002,
            Self::Apprentice => 0.003,
            Self::Journeyman => 0.004,
            Self::Master => 0.005,
        }
    }

    /// Starting fuel amount.
    pub fn starting_fuel(&self) -> f64 {
        match self {
            Self::Novice => 100.0,
            Self::Apprentice => 80.0,
            Self::Journeyman => 60.0,
            Self::Master => 40.0,
        }
    }

    /// Landing pad width in terrain points.
    pub fn pad_width(&self) -> usize {
        match self {
            Self::Novice => 12,
            Self::Apprentice => 8,
            Self::Journeyman => 5,
            Self::Master => 3,
        }
    }

    /// Terrain roughness factor (higher = more jagged).
    pub fn terrain_roughness(&self) -> f64 {
        match self {
            Self::Novice => 1.0,
            Self::Apprentice => 2.0,
            Self::Journeyman => 3.5,
            Self::Master => 5.0,
        }
    }

    /// Terminal velocity (max downward velocity per tick).
    pub fn terminal_velocity(&self) -> f64 {
        match self {
            Self::Novice => 0.3,
            Self::Apprentice => 0.3,
            Self::Journeyman => 0.3,
            Self::Master => 0.3,
        }
    }
}

/// Game outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanderResult {
    Win,
    Loss,
}

/// Terrain data: heights at each x coordinate, and landing pad location.
#[derive(Debug, Clone)]
pub struct Terrain {
    /// Height values for each x position (0..=GAME_WIDTH).
    /// Values represent the terrain height from the bottom (higher = taller mountain).
    pub heights: Vec<f64>,
    /// Left x index of the landing pad (inclusive).
    pub pad_left: usize,
    /// Right x index of the landing pad (inclusive).
    pub pad_right: usize,
    /// Height of the landing pad surface.
    pub pad_height: f64,
}

/// Rotation angle indices for the lander sprite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanderAngle {
    HardLeft,  // ~-30 degrees
    Left,      // ~-15 degrees
    Straight,  // 0 degrees (upright)
    Right,     // ~+15 degrees
    HardRight, // ~+30 degrees
}

impl LanderAngle {
    /// Convert a continuous angle (radians) to the nearest sprite angle.
    /// 0 = upright, negative = left tilt, positive = right tilt.
    pub fn from_radians(angle: f64) -> Self {
        if angle < -0.39 {
            Self::HardLeft
        } else if angle < -0.13 {
            Self::Left
        } else if angle < 0.13 {
            Self::Straight
        } else if angle < 0.39 {
            Self::Right
        } else {
            Self::HardRight
        }
    }
}

/// Main game state.
#[derive(Debug, Clone)]
pub struct LanderGame {
    pub difficulty: LanderDifficulty,
    pub game_result: Option<LanderResult>,
    pub forfeit_pending: bool,
    /// True until the player presses Space to begin. Physics paused while waiting.
    pub waiting_to_start: bool,

    // Lander state
    /// Horizontal position (float for smooth physics). 0 = left edge, GAME_WIDTH = right edge.
    pub x: f64,
    /// Vertical position (float for smooth physics). 0 = top, GAME_HEIGHT = bottom.
    pub y: f64,
    /// Horizontal velocity (positive = rightward).
    pub vx: f64,
    /// Vertical velocity (positive = downward).
    pub vy: f64,
    /// Rotation angle in radians. 0 = upright, negative = left tilt, positive = right tilt.
    pub angle: f64,

    // Fuel
    /// Remaining fuel.
    pub fuel: f64,
    /// Maximum fuel (for display).
    pub max_fuel: f64,

    // Input state
    /// True while the player is holding thrust.
    pub thrusting: bool,
    /// True while the player is holding left rotation.
    pub rotating_left: bool,
    /// True while the player is holding right rotation.
    pub rotating_right: bool,
    /// Remaining physics ticks before clearing thrust flag.
    pub thrust_hold_ticks: u32,
    /// Remaining physics ticks before clearing rotate-left flag.
    pub rotate_left_hold_ticks: u32,
    /// Remaining physics ticks before clearing rotate-right flag.
    pub rotate_right_hold_ticks: u32,
    /// Ticks remaining to show flame animation.
    pub flame_timer: u32,

    // Terrain
    pub terrain: Terrain,

    // Timing
    /// Sub-tick time accumulator (milliseconds).
    pub accumulated_time_ms: u64,
    /// Total physics ticks elapsed.
    pub tick_count: u64,

    // Cached difficulty parameters
    pub gravity: f64,
    pub terminal_velocity: f64,
}

impl LanderGame {
    /// Create a new game with the given difficulty using the provided RNG.
    pub fn new<R: Rng>(difficulty: LanderDifficulty, rng: &mut R) -> Self {
        let terrain = generate_terrain(difficulty, rng);
        let starting_fuel = difficulty.starting_fuel();

        Self {
            difficulty,
            game_result: None,
            forfeit_pending: false,
            waiting_to_start: true,

            // Start centered horizontally, near the top
            x: GAME_WIDTH as f64 / 2.0,
            y: 2.0,
            vx: 0.0,
            vy: 0.0,
            angle: 0.0,

            fuel: starting_fuel,
            max_fuel: starting_fuel,

            thrusting: false,
            rotating_left: false,
            rotating_right: false,
            thrust_hold_ticks: 0,
            rotate_left_hold_ticks: 0,
            rotate_right_hold_ticks: 0,
            flame_timer: 0,

            terrain,

            accumulated_time_ms: 0,
            tick_count: 0,

            gravity: difficulty.gravity(),
            terminal_velocity: difficulty.terminal_velocity(),
        }
    }

    /// Get the discrete sprite angle for rendering.
    pub fn sprite_angle(&self) -> LanderAngle {
        LanderAngle::from_radians(self.angle)
    }

    /// Get the altitude (distance from lander to terrain below).
    pub fn altitude(&self) -> f64 {
        let x_idx = (self.x.round() as usize).min(GAME_WIDTH as usize);
        let terrain_y = GAME_HEIGHT as f64 - self.terrain.heights[x_idx];
        (terrain_y - self.y).max(0.0)
    }

    /// Check if the lander is over the landing pad.
    pub fn over_pad(&self) -> bool {
        let x_idx = self.x.round() as usize;
        x_idx >= self.terrain.pad_left && x_idx <= self.terrain.pad_right
    }
}

/// Generate procedural terrain with a landing pad.
pub fn generate_terrain<R: Rng>(difficulty: LanderDifficulty, rng: &mut R) -> Terrain {
    let pad_width = difficulty.pad_width();
    let roughness = difficulty.terrain_roughness();

    // Pick a random position for the pad (ensuring it fits and has margin from edges)
    let margin = 5;
    let pad_left = rng.gen_range(margin..(TERRAIN_POINTS - pad_width - margin));
    let pad_right = pad_left + pad_width - 1;

    // Base terrain height (distance from bottom)
    let base_height = 6.0;

    // Generate terrain heights using midpoint displacement
    let mut heights = vec![base_height; TERRAIN_POINTS];

    // Random walk for terrain
    let mut current_height = base_height + rng.gen_range(-1.0..1.0) * roughness;
    for (i, h) in heights.iter_mut().enumerate() {
        if i >= pad_left && i <= pad_right {
            // Flat pad area
            continue; // Will be set below
        }

        // Random walk with mean reversion
        let delta = rng.gen_range(-1.0..1.0) * roughness * 0.5;
        let mean_revert = (base_height - current_height) * 0.1;
        current_height += delta + mean_revert;
        // Clamp between 3 and 12 to keep terrain within visible area
        current_height = current_height.clamp(3.0, 12.0);
        *h = current_height;
    }

    // Set pad height: average of adjacent terrain points, clamped
    let left_h = if pad_left > 0 {
        heights[pad_left - 1]
    } else {
        base_height
    };
    let right_h = if pad_right + 1 < TERRAIN_POINTS {
        heights[pad_right + 1]
    } else {
        base_height
    };
    let pad_height = ((left_h + right_h) / 2.0).clamp(4.0, 10.0);

    // Flatten pad area
    for h in heights.iter_mut().take(pad_right + 1).skip(pad_left) {
        *h = pad_height;
    }

    // Smooth transition near pad edges (2 points on each side)
    for offset in 1..=2 {
        if pad_left >= offset {
            let idx = pad_left - offset;
            let blend = offset as f64 / 3.0;
            heights[idx] = heights[idx] * blend + pad_height * (1.0 - blend);
        }
        if pad_right + offset < TERRAIN_POINTS {
            let idx = pad_right + offset;
            let blend = offset as f64 / 3.0;
            heights[idx] = heights[idx] * blend + pad_height * (1.0 - blend);
        }
    }

    Terrain {
        heights,
        pad_left,
        pad_right,
        pad_height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game_defaults() {
        let mut rng = rand::thread_rng();
        let game = LanderGame::new(LanderDifficulty::Novice, &mut rng);
        assert_eq!(game.difficulty, LanderDifficulty::Novice);
        assert!(game.game_result.is_none());
        assert!(!game.forfeit_pending);
        assert!(game.waiting_to_start);
        assert!(!game.thrusting);
        assert!(!game.rotating_left);
        assert!(!game.rotating_right);
        assert_eq!(game.tick_count, 0);
        assert!((game.angle - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_starting_position() {
        let mut rng = rand::thread_rng();
        let game = LanderGame::new(LanderDifficulty::Novice, &mut rng);
        assert!((game.x - 30.0).abs() < f64::EPSILON); // GAME_WIDTH / 2
        assert!((game.y - 2.0).abs() < f64::EPSILON);
        assert!((game.vx - 0.0).abs() < f64::EPSILON);
        assert!((game.vy - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_difficulty_fuel() {
        assert!((LanderDifficulty::Novice.starting_fuel() - 100.0).abs() < f64::EPSILON);
        assert!((LanderDifficulty::Apprentice.starting_fuel() - 80.0).abs() < f64::EPSILON);
        assert!((LanderDifficulty::Journeyman.starting_fuel() - 60.0).abs() < f64::EPSILON);
        assert!((LanderDifficulty::Master.starting_fuel() - 40.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_difficulty_gravity() {
        // Gravity should increase with difficulty
        let novice = LanderDifficulty::Novice.gravity();
        let apprentice = LanderDifficulty::Apprentice.gravity();
        let journeyman = LanderDifficulty::Journeyman.gravity();
        let master = LanderDifficulty::Master.gravity();
        assert!(novice < apprentice);
        assert!(apprentice < journeyman);
        assert!(journeyman < master);
    }

    #[test]
    fn test_difficulty_pad_width() {
        // Pad should get smaller with difficulty
        let novice = LanderDifficulty::Novice.pad_width();
        let apprentice = LanderDifficulty::Apprentice.pad_width();
        let journeyman = LanderDifficulty::Journeyman.pad_width();
        let master = LanderDifficulty::Master.pad_width();
        assert!(novice > apprentice);
        assert!(apprentice > journeyman);
        assert!(journeyman > master);
    }

    #[test]
    fn test_difficulty_roughness() {
        // Roughness should increase with difficulty
        let novice = LanderDifficulty::Novice.terrain_roughness();
        let master = LanderDifficulty::Master.terrain_roughness();
        assert!(novice < master);
    }

    #[test]
    fn test_difficulty_from_index() {
        assert_eq!(LanderDifficulty::from_index(0), LanderDifficulty::Novice);
        assert_eq!(
            LanderDifficulty::from_index(1),
            LanderDifficulty::Apprentice
        );
        assert_eq!(
            LanderDifficulty::from_index(2),
            LanderDifficulty::Journeyman
        );
        assert_eq!(LanderDifficulty::from_index(3), LanderDifficulty::Master);
        assert_eq!(LanderDifficulty::from_index(99), LanderDifficulty::Novice);
    }

    #[test]
    fn test_difficulty_names() {
        assert_eq!(LanderDifficulty::Novice.name(), "Novice");
        assert_eq!(LanderDifficulty::Apprentice.name(), "Apprentice");
        assert_eq!(LanderDifficulty::Journeyman.name(), "Journeyman");
        assert_eq!(LanderDifficulty::Master.name(), "Master");
    }

    #[test]
    fn test_all_difficulties() {
        assert_eq!(LanderDifficulty::ALL.len(), 4);
    }

    #[test]
    fn test_terrain_generation_has_correct_size() {
        let mut rng = rand::thread_rng();
        let terrain = generate_terrain(LanderDifficulty::Novice, &mut rng);
        assert_eq!(terrain.heights.len(), TERRAIN_POINTS);
    }

    #[test]
    fn test_terrain_pad_is_flat() {
        let mut rng = rand::thread_rng();
        for _ in 0..10 {
            let terrain = generate_terrain(LanderDifficulty::Novice, &mut rng);
            let pad_height = terrain.pad_height;
            for i in terrain.pad_left..=terrain.pad_right {
                assert!(
                    (terrain.heights[i] - pad_height).abs() < f64::EPSILON,
                    "Pad should be flat at x={}",
                    i
                );
            }
        }
    }

    #[test]
    fn test_terrain_pad_within_bounds() {
        let mut rng = rand::thread_rng();
        for diff in &LanderDifficulty::ALL {
            for _ in 0..10 {
                let terrain = generate_terrain(*diff, &mut rng);
                assert!(terrain.pad_left >= 5);
                assert!(terrain.pad_right < TERRAIN_POINTS - 5);
                assert!(terrain.pad_right >= terrain.pad_left);
                assert_eq!(terrain.pad_right - terrain.pad_left + 1, diff.pad_width());
            }
        }
    }

    #[test]
    fn test_terrain_heights_in_range() {
        let mut rng = rand::thread_rng();
        for diff in &LanderDifficulty::ALL {
            let terrain = generate_terrain(*diff, &mut rng);
            for (i, &h) in terrain.heights.iter().enumerate() {
                assert!(
                    (2.0..=14.0).contains(&h),
                    "Height at x={} is {} for {:?}, expected 2..14",
                    i,
                    h,
                    diff
                );
            }
        }
    }

    #[test]
    fn test_lander_angle_from_radians() {
        assert_eq!(LanderAngle::from_radians(0.0), LanderAngle::Straight);
        assert_eq!(LanderAngle::from_radians(-0.5), LanderAngle::HardLeft);
        assert_eq!(LanderAngle::from_radians(-0.2), LanderAngle::Left);
        assert_eq!(LanderAngle::from_radians(0.2), LanderAngle::Right);
        assert_eq!(LanderAngle::from_radians(0.5), LanderAngle::HardRight);
    }

    #[test]
    fn test_sprite_angle() {
        let mut rng = rand::thread_rng();
        let mut game = LanderGame::new(LanderDifficulty::Novice, &mut rng);
        assert_eq!(game.sprite_angle(), LanderAngle::Straight);

        game.angle = 0.3;
        assert_eq!(game.sprite_angle(), LanderAngle::Right);

        game.angle = -0.3;
        assert_eq!(game.sprite_angle(), LanderAngle::Left);
    }

    #[test]
    fn test_altitude() {
        let mut rng = rand::thread_rng();
        let mut game = LanderGame::new(LanderDifficulty::Novice, &mut rng);
        // Lander starts at y=2.0, terrain heights are roughly 4-12
        // altitude = (GAME_HEIGHT - terrain_height) - y
        let alt = game.altitude();
        assert!(alt > 0.0, "Starting altitude should be positive");

        // Move lander near the ground
        let x_idx = game.x.round() as usize;
        let terrain_y = GAME_HEIGHT as f64 - game.terrain.heights[x_idx];
        game.y = terrain_y - 1.0;
        let alt = game.altitude();
        assert!((alt - 1.0).abs() < 0.5, "Altitude should be ~1.0");
    }

    #[test]
    fn test_over_pad() {
        let mut rng = rand::thread_rng();
        let mut game = LanderGame::new(LanderDifficulty::Novice, &mut rng);
        let pad_center = (game.terrain.pad_left + game.terrain.pad_right) / 2;
        game.x = pad_center as f64;
        assert!(game.over_pad());

        // Move far from pad
        game.x = 0.0;
        if game.terrain.pad_left > 0 {
            assert!(!game.over_pad());
        }
    }

    #[test]
    fn test_fuel_initialized_correctly() {
        let mut rng = rand::thread_rng();
        for diff in &LanderDifficulty::ALL {
            let game = LanderGame::new(*diff, &mut rng);
            assert!((game.fuel - diff.starting_fuel()).abs() < f64::EPSILON);
            assert!((game.max_fuel - diff.starting_fuel()).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_all_difficulties_have_valid_parameters() {
        for diff in &LanderDifficulty::ALL {
            assert!(diff.gravity() > 0.0, "{:?} gravity must be positive", diff);
            assert!(
                diff.starting_fuel() > 0.0,
                "{:?} starting fuel must be positive",
                diff
            );
            assert!(
                diff.pad_width() > 0,
                "{:?} pad width must be positive",
                diff
            );
            assert!(
                diff.terrain_roughness() > 0.0,
                "{:?} roughness must be positive",
                diff
            );
            assert!(
                diff.terminal_velocity() > 0.0,
                "{:?} terminal velocity must be positive",
                diff
            );
        }
    }
}
