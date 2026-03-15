//! Vault Warden data types.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultWardenDifficulty {
    Novice,
    Apprentice,
    Journeyman,
    Master,
}

difficulty_enum_impl!(VaultWardenDifficulty);

impl VaultWardenDifficulty {
    /// Number of undos available at this difficulty.
    pub fn max_undos(&self) -> u8 {
        match self {
            Self::Novice => 5,
            Self::Apprentice => 3,
            Self::Journeyman => 2,
            Self::Master => 1,
        }
    }

    /// Number of restart attempts (same for all difficulties).
    pub fn max_attempts(&self) -> u8 {
        5
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultWardenResult {
    Win,
    Loss,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VaultWardenInput {
    Up,
    Down,
    Left,
    Right,
    Undo,
    Restart,
    Forfeit,
    Other,
}

/// Static terrain cell (walls and floors).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Wall,
    Floor,
}

/// Record of a single move for undo support.
#[derive(Debug, Clone)]
pub struct MoveRecord {
    pub player_from: (usize, usize),
    pub pushed_crate: Option<CratePush>,
}

#[derive(Debug, Clone)]
pub struct CratePush {
    pub from: (usize, usize),
    pub to: (usize, usize),
}

#[derive(Debug, Clone)]
pub struct VaultWardenGame {
    pub difficulty: VaultWardenDifficulty,
    pub game_result: Option<VaultWardenResult>,
    pub forfeit_pending: bool,
    pub grid: Vec<Vec<Cell>>,
    pub width: usize,
    pub height: usize,
    pub player_pos: (usize, usize),
    pub crate_positions: Vec<(usize, usize)>,
    pub goal_positions: Vec<(usize, usize)>,
    pub moves: u16,
    pub undos_remaining: u8,
    pub undos_max: u8,
    pub attempts_remaining: u8,
    pub attempts_max: u8,
    pub move_history: Vec<MoveRecord>,
    // Snapshot of initial state for restart
    pub initial_player_pos: (usize, usize),
    pub initial_crate_positions: Vec<(usize, usize)>,
}

impl VaultWardenGame {
    /// Count of crates currently on goal squares.
    pub fn crates_on_goals(&self) -> usize {
        self.crate_positions
            .iter()
            .filter(|c| self.goal_positions.contains(c))
            .count()
    }

    /// Total number of crates (= total goals).
    pub fn total_crates(&self) -> usize {
        self.crate_positions.len()
    }

    /// Check if a position contains a crate.
    pub fn has_crate_at(&self, pos: (usize, usize)) -> bool {
        self.crate_positions.contains(&pos)
    }

    /// Check if a position is a goal.
    pub fn is_goal(&self, pos: (usize, usize)) -> bool {
        self.goal_positions.contains(&pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_enum() {
        assert_eq!(VaultWardenDifficulty::ALL.len(), 4);
        assert_eq!(
            VaultWardenDifficulty::from_index(0),
            VaultWardenDifficulty::Novice
        );
        assert_eq!(
            VaultWardenDifficulty::from_index(3),
            VaultWardenDifficulty::Master
        );
        assert_eq!(
            VaultWardenDifficulty::from_index(99),
            VaultWardenDifficulty::Novice
        );
    }

    #[test]
    fn test_max_undos() {
        assert_eq!(VaultWardenDifficulty::Novice.max_undos(), 5);
        assert_eq!(VaultWardenDifficulty::Apprentice.max_undos(), 3);
        assert_eq!(VaultWardenDifficulty::Journeyman.max_undos(), 2);
        assert_eq!(VaultWardenDifficulty::Master.max_undos(), 1);
    }

    #[test]
    fn test_max_attempts() {
        assert_eq!(VaultWardenDifficulty::Novice.max_attempts(), 5);
        assert_eq!(VaultWardenDifficulty::Master.max_attempts(), 5);
    }

    fn make_test_game() -> VaultWardenGame {
        VaultWardenGame {
            difficulty: VaultWardenDifficulty::Novice,
            game_result: None,
            forfeit_pending: false,
            grid: vec![vec![Cell::Floor; 5]; 5],
            width: 5,
            height: 5,
            player_pos: (0, 0),
            crate_positions: vec![(1, 1), (2, 2)],
            goal_positions: vec![(1, 1), (3, 3)],
            moves: 0,
            undos_remaining: 5,
            undos_max: 5,
            attempts_remaining: 5,
            attempts_max: 5,
            move_history: vec![],
            initial_player_pos: (0, 0),
            initial_crate_positions: vec![(1, 1), (2, 2)],
        }
    }

    #[test]
    fn test_crates_on_goals() {
        let game = make_test_game();
        assert_eq!(game.crates_on_goals(), 1); // (1,1) is on goal
        assert_eq!(game.total_crates(), 2);
    }

    #[test]
    fn test_has_crate_at() {
        let game = make_test_game();
        assert!(game.has_crate_at((1, 1)));
        assert!(game.has_crate_at((2, 2)));
        assert!(!game.has_crate_at((0, 0)));
    }

    #[test]
    fn test_is_goal() {
        let game = make_test_game();
        assert!(game.is_goal((1, 1)));
        assert!(game.is_goal((3, 3)));
        assert!(!game.is_goal((0, 0)));
    }
}
