//! Nine Men's Morris minigame data structures and state management.
//!
//! Board layout (positions 0-23):
//! ```text
//! 0-----------1-----------2
//! |           |           |
//! |   3-------4-------5   |
//! |   |       |       |   |
//! |   |   6---7---8   |   |
//! |   |   |       |   |   |
//! 9---10--11      12--13--14
//! |   |   |       |   |   |
//! |   |   15--16--17  |   |
//! |   |       |       |   |
//! |   18------19------20  |
//! |           |           |
//! 21----------22----------23
//! ```

use serde::{Deserialize, Serialize};

/// The 16 valid mill lines (three-in-a-row formations)
pub const MILLS: [[usize; 3]; 16] = [
    // Horizontal mills - outer ring
    [0, 1, 2],
    [9, 10, 11],
    [12, 13, 14],
    [21, 22, 23],
    // Horizontal mills - middle ring
    [3, 4, 5],
    [18, 19, 20],
    // Horizontal mills - inner ring
    [6, 7, 8],
    [15, 16, 17],
    // Vertical mills - left side
    [0, 9, 21],
    [3, 10, 18],
    [6, 11, 15],
    // Vertical mills - right side
    [2, 14, 23],
    [5, 13, 20],
    [8, 12, 17],
    // Vertical mills - top and bottom
    [1, 4, 7],
    [16, 19, 22],
];

/// Adjacency list for each of 24 positions on the board
pub const ADJACENCIES: [&[usize]; 24] = [
    // Position 0: connects to 1, 9
    &[1, 9],
    // Position 1: connects to 0, 2, 4
    &[0, 2, 4],
    // Position 2: connects to 1, 14
    &[1, 14],
    // Position 3: connects to 4, 10
    &[4, 10],
    // Position 4: connects to 1, 3, 5, 7
    &[1, 3, 5, 7],
    // Position 5: connects to 4, 13
    &[4, 13],
    // Position 6: connects to 7, 11
    &[7, 11],
    // Position 7: connects to 4, 6, 8
    &[4, 6, 8],
    // Position 8: connects to 7, 12
    &[7, 12],
    // Position 9: connects to 0, 10, 21
    &[0, 10, 21],
    // Position 10: connects to 3, 9, 11, 18
    &[3, 9, 11, 18],
    // Position 11: connects to 6, 10, 15
    &[6, 10, 15],
    // Position 12: connects to 8, 13, 17
    &[8, 13, 17],
    // Position 13: connects to 5, 12, 14, 20
    &[5, 12, 14, 20],
    // Position 14: connects to 2, 13, 23
    &[2, 13, 23],
    // Position 15: connects to 11, 16
    &[11, 16],
    // Position 16: connects to 15, 17, 19
    &[15, 17, 19],
    // Position 17: connects to 12, 16
    &[12, 16],
    // Position 18: connects to 10, 19
    &[10, 19],
    // Position 19: connects to 16, 18, 20, 22
    &[16, 18, 20, 22],
    // Position 20: connects to 13, 19
    &[13, 19],
    // Position 21: connects to 9, 22
    &[9, 22],
    // Position 22: connects to 19, 21, 23
    &[19, 21, 23],
    // Position 23: connects to 14, 22
    &[14, 22],
];

/// Player in Nine Men's Morris
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Player {
    Human,
    Ai,
}

/// AI difficulty levels for Nine Men's Morris
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MorrisDifficulty {
    Novice,     // 50% random moves, depth 1
    Apprentice, // depth 1
    Journeyman, // depth 2
    Master,     // depth 3
}

difficulty_enum_impl!(MorrisDifficulty);

impl MorrisDifficulty {
    pub fn search_depth(&self) -> i32 {
        match self {
            Self::Novice => 2,
            Self::Apprentice => 3,
            Self::Journeyman => 4,
            Self::Master => 5,
        }
    }

    pub fn random_move_chance(&self) -> f64 {
        match self {
            Self::Novice => 0.5,
            _ => 0.0,
        }
    }

    /// XP reward as a percentage of XP needed for current level.
    /// e.g. 25 means 25% of `xp_for_next_level(current_level)`.
    pub fn reward_xp_percent(&self) -> u32 {
        match self {
            Self::Novice => 50,
            Self::Apprentice => 100,
            Self::Journeyman => 150,
            Self::Master => 200,
        }
    }
}

/// Result of a completed Morris game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MorrisResult {
    Win,
    Loss,
    Forfeit,
}

/// Game phase in Nine Men's Morris
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MorrisPhase {
    /// Players are placing their 9 pieces on the board
    Placing,
    /// Players move pieces to adjacent positions
    Moving,
    /// A player with 3 pieces can "fly" (move to any empty position)
    Flying,
}

/// A move in Nine Men's Morris
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MorrisMove {
    /// Place a piece at the given position (during Placing phase)
    Place(usize),
    /// Move a piece from one position to another (during Moving/Flying phase)
    Move { from: usize, to: usize },
    /// Capture an opponent's piece at the given position (after forming a mill)
    Capture(usize),
}

/// Direction for cursor movement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Active Nine Men's Morris game session (transient, not saved)
#[derive(Debug, Clone)]
pub struct MorrisGame {
    /// The 24-position board; None = empty, Some(Player) = occupied
    pub board: [Option<Player>; 24],
    /// Current game phase
    pub phase: MorrisPhase,
    /// Pieces left to place: (human, ai)
    pub pieces_to_place: (u8, u8),
    /// Pieces currently on the board: (human, ai)
    pub pieces_on_board: (u8, u8),
    /// AI difficulty level
    pub difficulty: MorrisDifficulty,
    /// Current cursor position (0-23)
    pub cursor: usize,
    /// Selected position for moving a piece
    pub selected_position: Option<usize>,
    /// Whether the current player must capture a piece (after forming a mill)
    pub must_capture: bool,
    /// Whose turn it is
    pub current_player: Player,
    /// Game result (None if game is ongoing)
    pub game_result: Option<MorrisResult>,
    /// Whether forfeit confirmation is pending
    pub forfeit_pending: bool,
    /// Whether AI is currently thinking
    pub ai_thinking: bool,
    /// Ticks spent thinking
    pub ai_think_ticks: u32,
    /// Target ticks for AI "thinking" delay
    pub ai_think_target: u32,
    /// The move the AI has decided on (waiting to execute)
    pub ai_pending_move: Option<MorrisMove>,
    /// Last move made (for highlighting on game over)
    pub last_move: Option<MorrisMove>,
}

/// Cursor position mapping to board positions for navigation
/// This maps each position to its logical neighbors for cursor movement
const CURSOR_MAP: [(i8, i8); 24] = [
    // Row 0 (top): positions 0, 1, 2
    (0, 0),
    (3, 0),
    (6, 0),
    // Row 1: positions 3, 4, 5
    (1, 1),
    (3, 1),
    (5, 1),
    // Row 2: positions 6, 7, 8
    (2, 2),
    (3, 2),
    (4, 2),
    // Row 3 (middle): positions 9, 10, 11, 12, 13, 14
    (0, 3),
    (1, 3),
    (2, 3),
    (4, 3),
    (5, 3),
    (6, 3),
    // Row 4: positions 15, 16, 17
    (2, 4),
    (3, 4),
    (4, 4),
    // Row 5: positions 18, 19, 20
    (1, 5),
    (3, 5),
    (5, 5),
    // Row 6 (bottom): positions 21, 22, 23
    (0, 6),
    (3, 6),
    (6, 6),
];

impl MorrisGame {
    /// Create a new Morris game with the given difficulty
    pub fn new(difficulty: MorrisDifficulty) -> Self {
        Self {
            board: [None; 24],
            phase: MorrisPhase::Placing,
            pieces_to_place: (9, 9),
            pieces_on_board: (0, 0),
            difficulty,
            cursor: 0,
            selected_position: None,
            must_capture: false,
            current_player: Player::Human,
            game_result: None,
            forfeit_pending: false,
            ai_thinking: false,
            ai_think_ticks: 0,
            ai_think_target: 0,
            ai_pending_move: None,
            last_move: None,
        }
    }

    /// Check if a position is part of a mill for the given player
    pub fn is_in_mill(&self, pos: usize, player: Player) -> bool {
        for mill in MILLS.iter() {
            if mill.contains(&pos) && mill.iter().all(|&p| self.board[p] == Some(player)) {
                return true;
            }
        }
        false
    }

    /// Check if placing/moving a piece to a position would form a new mill
    pub fn forms_mill(&self, pos: usize, player: Player) -> bool {
        for mill in MILLS.iter() {
            if mill.contains(&pos) {
                // Check if all three positions in the mill would be occupied by player
                let all_occupied = mill.iter().all(|&p| {
                    if p == pos {
                        true // The position we're checking
                    } else {
                        self.board[p] == Some(player)
                    }
                });
                if all_occupied {
                    return true;
                }
            }
        }
        false
    }

    /// Get pieces left to place for a player
    pub fn pieces_to_place_for(&self, player: Player) -> u8 {
        match player {
            Player::Human => self.pieces_to_place.0,
            Player::Ai => self.pieces_to_place.1,
        }
    }

    /// Check if a player can fly (has exactly 3 pieces and no pieces to place)
    pub fn can_fly(&self, player: Player) -> bool {
        let on_board = match player {
            Player::Human => self.pieces_on_board.0,
            Player::Ai => self.pieces_on_board.1,
        };
        let to_place = self.pieces_to_place_for(player);
        on_board == 3 && to_place == 0
    }

    /// Move the cursor in the given direction
    pub fn move_cursor(&mut self, direction: CursorDirection) {
        let (cx, cy) = CURSOR_MAP[self.cursor];

        // Find the nearest position in the given direction
        let target = match direction {
            CursorDirection::Up => self.find_nearest_position(cx, cy, 0, -1),
            CursorDirection::Down => self.find_nearest_position(cx, cy, 0, 1),
            CursorDirection::Left => self.find_nearest_position(cx, cy, -1, 0),
            CursorDirection::Right => self.find_nearest_position(cx, cy, 1, 0),
        };

        if let Some(pos) = target {
            self.cursor = pos;
        }
    }

    /// Find the nearest position in the given direction
    fn find_nearest_position(&self, cx: i8, cy: i8, dx: i8, dy: i8) -> Option<usize> {
        let mut best_pos = None;
        let mut best_dist = i32::MAX;

        for (pos, &(px, py)) in CURSOR_MAP.iter().enumerate() {
            if pos == self.cursor {
                continue;
            }

            // Check if position is in the correct direction
            let in_direction = match (dx, dy) {
                (0, -1) => py < cy, // Up
                (0, 1) => py > cy,  // Down
                (-1, 0) => px < cx, // Left
                (1, 0) => px > cx,  // Right
                _ => false,
            };

            if !in_direction {
                continue;
            }

            // Calculate distance (Manhattan distance weighted by direction)
            let dist_x = (px as i32 - cx as i32).abs();
            let dist_y = (py as i32 - cy as i32).abs();

            // Prefer positions that are more aligned with the direction
            let dist = if dx != 0 {
                dist_x + dist_y * 10 // Moving horizontally, penalize vertical deviation
            } else {
                dist_y + dist_x * 10 // Moving vertically, penalize horizontal deviation
            };

            if dist < best_dist {
                best_dist = dist;
                best_pos = Some(pos);
            }
        }

        best_pos
    }

    /// Clear the current selection
    pub fn clear_selection(&mut self) {
        self.selected_position = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_from_index() {
        assert_eq!(MorrisDifficulty::from_index(0), MorrisDifficulty::Novice);
        assert_eq!(
            MorrisDifficulty::from_index(1),
            MorrisDifficulty::Apprentice
        );
        assert_eq!(
            MorrisDifficulty::from_index(2),
            MorrisDifficulty::Journeyman
        );
        assert_eq!(MorrisDifficulty::from_index(3), MorrisDifficulty::Master);
        // Out of bounds should return Novice
        assert_eq!(MorrisDifficulty::from_index(99), MorrisDifficulty::Novice);
    }

    #[test]
    fn test_difficulty_properties() {
        // Search depth (optimized with make/unmake pattern)
        assert_eq!(MorrisDifficulty::Novice.search_depth(), 2);
        assert_eq!(MorrisDifficulty::Apprentice.search_depth(), 3);
        assert_eq!(MorrisDifficulty::Journeyman.search_depth(), 4);
        assert_eq!(MorrisDifficulty::Master.search_depth(), 5);

        // Random move chance
        assert_eq!(MorrisDifficulty::Novice.random_move_chance(), 0.5);
        assert_eq!(MorrisDifficulty::Apprentice.random_move_chance(), 0.0);
        assert_eq!(MorrisDifficulty::Journeyman.random_move_chance(), 0.0);
        assert_eq!(MorrisDifficulty::Master.random_move_chance(), 0.0);

        // XP reward percentages
        assert_eq!(MorrisDifficulty::Novice.reward_xp_percent(), 50);
        assert_eq!(MorrisDifficulty::Apprentice.reward_xp_percent(), 100);
        assert_eq!(MorrisDifficulty::Journeyman.reward_xp_percent(), 150);
        assert_eq!(MorrisDifficulty::Master.reward_xp_percent(), 200);

        // Names
        assert_eq!(MorrisDifficulty::Novice.name(), "Novice");
        assert_eq!(MorrisDifficulty::Apprentice.name(), "Apprentice");
        assert_eq!(MorrisDifficulty::Journeyman.name(), "Journeyman");
        assert_eq!(MorrisDifficulty::Master.name(), "Master");
    }

    #[test]
    fn test_difficulty_rewards_via_trait() {
        use crate::challenges::menu::DifficultyInfo;

        // Morris rewards XP, with fishing rank at Master
        let novice = MorrisDifficulty::Novice.reward();
        assert_eq!(novice.xp_percent, 50);
        assert_eq!(novice.prestige_ranks, 0);
        assert_eq!(novice.fishing_ranks, 0);

        let apprentice = MorrisDifficulty::Apprentice.reward();
        assert_eq!(apprentice.xp_percent, 100);
        assert_eq!(apprentice.fishing_ranks, 0);

        let journeyman = MorrisDifficulty::Journeyman.reward();
        assert_eq!(journeyman.xp_percent, 150);
        assert_eq!(journeyman.fishing_ranks, 0);

        let master = MorrisDifficulty::Master.reward();
        assert_eq!(master.xp_percent, 200);
        assert_eq!(master.fishing_ranks, 1);
        assert_eq!(master.prestige_ranks, 0);
    }

    #[test]
    fn test_new_game() {
        let game = MorrisGame::new(MorrisDifficulty::Journeyman);

        // Check initial state
        assert_eq!(game.difficulty, MorrisDifficulty::Journeyman);
        assert_eq!(game.phase, MorrisPhase::Placing);
        assert_eq!(game.pieces_to_place, (9, 9));
        assert_eq!(game.pieces_on_board, (0, 0));
        assert_eq!(game.cursor, 0);
        assert!(game.selected_position.is_none());
        assert!(!game.must_capture);
        assert_eq!(game.current_player, Player::Human);
        assert!(game.game_result.is_none());
        assert!(!game.forfeit_pending);
        assert!(!game.ai_thinking);

        // Check board is empty
        for pos in game.board.iter() {
            assert!(pos.is_none());
        }
    }

    #[test]
    fn test_mills_count() {
        // There should be exactly 16 mills in Nine Men's Morris
        assert_eq!(MILLS.len(), 16);

        // Each mill should have exactly 3 positions
        for mill in MILLS.iter() {
            assert_eq!(mill.len(), 3);
        }

        // All positions in mills should be valid (0-23)
        for mill in MILLS.iter() {
            for &pos in mill.iter() {
                assert!(pos < 24, "Invalid position {} in mill", pos);
            }
        }
    }

    #[test]
    fn test_adjacencies() {
        // There should be exactly 24 adjacency lists
        assert_eq!(ADJACENCIES.len(), 24);

        // Check that adjacencies are symmetric
        for (pos, neighbors) in ADJACENCIES.iter().enumerate() {
            for &neighbor in neighbors.iter() {
                assert!(
                    ADJACENCIES[neighbor].contains(&pos),
                    "Adjacency not symmetric: {} -> {} but {} does not -> {}",
                    pos,
                    neighbor,
                    neighbor,
                    pos
                );
            }
        }

        // Check some specific adjacencies from the board layout
        // Position 0 connects to 1 and 9
        assert!(ADJACENCIES[0].contains(&1));
        assert!(ADJACENCIES[0].contains(&9));
        assert_eq!(ADJACENCIES[0].len(), 2);

        // Position 4 (center of middle ring) connects to 1, 3, 5, 7
        assert!(ADJACENCIES[4].contains(&1));
        assert!(ADJACENCIES[4].contains(&3));
        assert!(ADJACENCIES[4].contains(&5));
        assert!(ADJACENCIES[4].contains(&7));
        assert_eq!(ADJACENCIES[4].len(), 4);

        // Position 10 connects to 3, 9, 11, 18
        assert!(ADJACENCIES[10].contains(&3));
        assert!(ADJACENCIES[10].contains(&9));
        assert!(ADJACENCIES[10].contains(&11));
        assert!(ADJACENCIES[10].contains(&18));
        assert_eq!(ADJACENCIES[10].len(), 4);
    }

    #[test]
    fn test_is_in_mill() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);

        // Place pieces to form a mill at positions 0, 1, 2
        game.board[0] = Some(Player::Human);
        game.board[1] = Some(Player::Human);
        game.board[2] = Some(Player::Human);

        // All three positions should be in a mill
        assert!(game.is_in_mill(0, Player::Human));
        assert!(game.is_in_mill(1, Player::Human));
        assert!(game.is_in_mill(2, Player::Human));

        // Position 3 is not in a mill
        assert!(!game.is_in_mill(3, Player::Human));

        // AI pieces are not in a mill
        assert!(!game.is_in_mill(0, Player::Ai));
    }

    #[test]
    fn test_forms_mill() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);

        // Place two pieces of a potential mill
        game.board[0] = Some(Player::Human);
        game.board[1] = Some(Player::Human);

        // Placing at position 2 would form a mill
        assert!(game.forms_mill(2, Player::Human));

        // Placing at position 3 would not form a mill
        assert!(!game.forms_mill(3, Player::Human));

        // AI placing at position 2 would not form a mill (different player)
        assert!(!game.forms_mill(2, Player::Ai));
    }

    #[test]
    fn test_can_fly() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);

        // Initially cannot fly (no pieces on board, 9 pieces to place)
        assert!(!game.can_fly(Player::Human));
        assert!(!game.can_fly(Player::Ai));

        // Set up a flying scenario: 3 pieces on board, 0 to place
        game.pieces_on_board = (3, 4);
        game.pieces_to_place = (0, 0);

        // Human can fly (3 pieces), AI cannot (4 pieces)
        assert!(game.can_fly(Player::Human));
        assert!(!game.can_fly(Player::Ai));
    }

    #[test]
    fn test_pieces_to_place_for() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);

        assert_eq!(game.pieces_to_place_for(Player::Human), 9);
        assert_eq!(game.pieces_to_place_for(Player::Ai), 9);

        game.pieces_to_place = (5, 3);

        assert_eq!(game.pieces_to_place_for(Player::Human), 5);
        assert_eq!(game.pieces_to_place_for(Player::Ai), 3);
    }

    #[test]
    fn test_clear_selection() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);
        game.selected_position = Some(5);

        game.clear_selection();

        assert!(game.selected_position.is_none());
    }

    #[test]
    fn test_move_cursor() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);

        // Start at position 0 (top-left corner)
        assert_eq!(game.cursor, 0);

        // Move right should go to position 1
        game.move_cursor(CursorDirection::Right);
        assert_eq!(game.cursor, 1);

        // Move right again should go to position 2
        game.move_cursor(CursorDirection::Right);
        assert_eq!(game.cursor, 2);

        // Move down from position 2 should go toward position 14
        game.move_cursor(CursorDirection::Down);
        assert!(game.cursor > 2); // Should move to a lower position

        // Reset and test left movement
        game.cursor = 2;
        game.move_cursor(CursorDirection::Left);
        assert_eq!(game.cursor, 1);
    }

    #[test]
    fn test_cursor_bounds() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);

        // At position 0 (top-left), moving up or left should stay at 0
        game.cursor = 0;
        game.move_cursor(CursorDirection::Up);
        assert_eq!(game.cursor, 0);

        game.cursor = 0;
        game.move_cursor(CursorDirection::Left);
        assert_eq!(game.cursor, 0);

        // At position 23 (bottom-right), moving down or right should stay at 23
        game.cursor = 23;
        game.move_cursor(CursorDirection::Down);
        assert_eq!(game.cursor, 23);

        game.cursor = 23;
        game.move_cursor(CursorDirection::Right);
        assert_eq!(game.cursor, 23);
    }

    #[test]
    fn test_player_equality() {
        assert_eq!(Player::Human, Player::Human);
        assert_eq!(Player::Ai, Player::Ai);
        assert_ne!(Player::Human, Player::Ai);
    }

    #[test]
    fn test_morris_move_variants() {
        let place = MorrisMove::Place(5);
        let move_piece = MorrisMove::Move { from: 0, to: 1 };
        let capture = MorrisMove::Capture(10);

        // Just verify they can be created and compared
        assert_eq!(place, MorrisMove::Place(5));
        assert_eq!(move_piece, MorrisMove::Move { from: 0, to: 1 });
        assert_eq!(capture, MorrisMove::Capture(10));
        assert_ne!(place, capture);
    }

    #[test]
    fn test_morris_phase_transitions() {
        // Just verify the phases can be compared
        assert_eq!(MorrisPhase::Placing, MorrisPhase::Placing);
        assert_eq!(MorrisPhase::Moving, MorrisPhase::Moving);
        assert_eq!(MorrisPhase::Flying, MorrisPhase::Flying);
        assert_ne!(MorrisPhase::Placing, MorrisPhase::Moving);
    }

    #[test]
    fn test_morris_result_variants() {
        assert_eq!(MorrisResult::Win, MorrisResult::Win);
        assert_eq!(MorrisResult::Loss, MorrisResult::Loss);
        assert_eq!(MorrisResult::Forfeit, MorrisResult::Forfeit);
        assert_ne!(MorrisResult::Win, MorrisResult::Loss);
    }

    #[test]
    fn test_multiple_mills_detection() {
        let mut game = MorrisGame::new(MorrisDifficulty::Novice);

        // Create two mills sharing position 1:
        // Mill 1: 0-1-2 (horizontal top)
        // Mill 2: 1-4-7 (vertical)
        game.board[0] = Some(Player::Human);
        game.board[1] = Some(Player::Human);
        game.board[2] = Some(Player::Human);
        game.board[4] = Some(Player::Human);
        game.board[7] = Some(Player::Human);

        // Position 1 is in both mills
        assert!(game.is_in_mill(1, Player::Human));

        // Position 0 and 2 are in the horizontal mill
        assert!(game.is_in_mill(0, Player::Human));
        assert!(game.is_in_mill(2, Player::Human));

        // Position 4 and 7 are in the vertical mill
        assert!(game.is_in_mill(4, Player::Human));
        assert!(game.is_in_mill(7, Player::Human));
    }
}
