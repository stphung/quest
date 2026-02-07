//! Enhanced Monte Carlo Tree Search AI for Go.
//!
//! Uses heuristics for move ordering and simulation guidance.

use super::heuristics::{get_top_moves, score_move};
use super::logic::{get_legal_moves, is_legal_move, make_move};
use super::types::{GoDifficulty, GoGame, GoMove, GoResult, Stone, BOARD_SIZE};
use rand::Rng;

/// UCT exploration constant
const UCT_C: f64 = 1.4;

/// Maximum moves in a simulation before scoring
const MAX_SIMULATION_MOVES: u32 = 120;

/// Number of top moves to consider for expansion (progressive widening)
const TOP_MOVES_LIMIT: usize = 15;

/// MCTS tree node
struct MctsNode {
    /// Move that led to this node
    move_taken: Option<GoMove>,
    /// Parent node index
    parent: Option<usize>,
    /// Child node indices
    children: Vec<usize>,
    /// Number of visits
    visits: u32,
    /// Number of wins (from perspective of player who made the move)
    wins: f32,
    /// Moves not yet expanded
    untried_moves: Vec<GoMove>,
    /// Player who just moved (to reach this state)
    player_just_moved: Stone,
    /// Prior score from heuristics (for move ordering)
    prior_score: f64,
}

impl MctsNode {
    fn new(
        parent: Option<usize>,
        move_taken: Option<GoMove>,
        player_just_moved: Stone,
        legal_moves: Vec<GoMove>,
        prior_score: f64,
    ) -> Self {
        Self {
            move_taken,
            parent,
            children: Vec::new(),
            visits: 0,
            wins: 0.0,
            untried_moves: legal_moves,
            player_just_moved,
            prior_score,
        }
    }

    fn uct_value(&self, parent_log_visits: f64) -> f64 {
        if self.visits == 0 {
            // Unvisited nodes get prior bonus
            return f64::INFINITY + self.prior_score;
        }
        let exploitation = self.wins as f64 / self.visits as f64;
        let exploration = UCT_C * (parent_log_visits / self.visits as f64).sqrt();

        // Add small prior bonus that diminishes with visits
        let prior_bonus = self.prior_score / (1.0 + self.visits as f64);

        exploitation + exploration + prior_bonus * 0.01
    }
}

/// Run MCTS and return the best move.
pub fn mcts_best_move<R: Rng>(game: &GoGame, rng: &mut R) -> GoMove {
    let simulations = game.difficulty.simulation_count();
    let mut nodes: Vec<MctsNode> = Vec::with_capacity(simulations as usize);

    // Get legal moves and order by heuristics
    let all_moves = get_legal_moves(game);
    let top_moves = get_top_moves(game, &all_moves, TOP_MOVES_LIMIT);

    // Create root node with ordered moves
    let root = MctsNode::new(None, None, game.current_player.opponent(), top_moves, 0.0);
    nodes.push(root);

    for _ in 0..simulations {
        let mut game_clone = game.clone();

        // Selection: traverse tree using UCT
        let mut node_idx = 0;
        while nodes[node_idx].untried_moves.is_empty() && !nodes[node_idx].children.is_empty() {
            node_idx = select_child(&nodes, node_idx);
            if let Some(mv) = nodes[node_idx].move_taken {
                make_move(&mut game_clone, mv);
            }
        }

        // Expansion: add a new child if possible
        if !nodes[node_idx].untried_moves.is_empty() && game_clone.game_result.is_none() {
            // Pick move with best heuristic score from untried
            let best_idx = select_best_untried(&nodes[node_idx].untried_moves, &game_clone);
            let mv = nodes[node_idx].untried_moves.swap_remove(best_idx);

            let prior = match mv {
                GoMove::Place(r, c) => score_move(&game_clone, r, c),
                GoMove::Pass => -50.0,
            };

            let current_player = game_clone.current_player;
            make_move(&mut game_clone, mv);

            // Get top moves for child (progressive widening)
            let child_all_moves = get_legal_moves(&game_clone);
            let child_moves = get_top_moves(&game_clone, &child_all_moves, TOP_MOVES_LIMIT);

            let child = MctsNode::new(Some(node_idx), Some(mv), current_player, child_moves, prior);
            let child_idx = nodes.len();
            nodes.push(child);
            nodes[node_idx].children.push(child_idx);
            node_idx = child_idx;
        }

        // Simulation: guided random playout
        let winner = simulate_guided_game(&mut game_clone, rng);

        // Backpropagation: update statistics
        backpropagate(&mut nodes, node_idx, winner);
    }

    // Select best move (most visits, with win rate tiebreaker)
    select_best_move(&nodes)
}

/// Select the best untried move based on heuristics.
fn select_best_untried(moves: &[GoMove], game: &GoGame) -> usize {
    moves
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| {
            let score_a = match a {
                GoMove::Place(r, c) => score_move(game, *r, *c),
                GoMove::Pass => -50.0,
            };
            let score_b = match b {
                GoMove::Place(r, c) => score_move(game, *r, *c),
                GoMove::Pass => -50.0,
            };
            score_a.partial_cmp(&score_b).unwrap()
        })
        .map(|(i, _)| i)
        .unwrap_or(0)
}

/// Select child with highest UCT value.
fn select_child(nodes: &[MctsNode], parent_idx: usize) -> usize {
    let parent_log_visits = (nodes[parent_idx].visits as f64 + 1.0).ln();
    nodes[parent_idx]
        .children
        .iter()
        .max_by(|&&a, &&b| {
            nodes[a]
                .uct_value(parent_log_visits)
                .partial_cmp(&nodes[b].uct_value(parent_log_visits))
                .unwrap()
        })
        .copied()
        .unwrap_or(parent_idx)
}

/// Simulate a game with heuristic guidance.
/// Uses weighted random selection favoring better moves.
fn simulate_guided_game<R: Rng>(game: &mut GoGame, rng: &mut R) -> Option<Stone> {
    let mut moves_made = 0;
    let mut consecutive_passes = 0;

    while game.game_result.is_none() && moves_made < MAX_SIMULATION_MOVES {
        if let Some(mv) = guided_random_move(game, rng) {
            make_move(game, mv);
            consecutive_passes = 0;
        } else {
            make_move(game, GoMove::Pass);
            consecutive_passes += 1;
            if consecutive_passes >= 2 {
                break;
            }
        }
        moves_made += 1;
    }

    // Determine winner from final position
    if let Some(result) = game.game_result {
        match result {
            GoResult::Win => Some(Stone::Black),
            GoResult::Loss => Some(Stone::White),
            GoResult::Draw => None,
        }
    } else {
        let (black, white) = super::logic::calculate_score(&game.board);
        if black > white {
            Some(Stone::Black)
        } else if white > black {
            Some(Stone::White)
        } else {
            None
        }
    }
}

/// Select a move with probability proportional to heuristic score.
fn guided_random_move<R: Rng>(game: &GoGame, rng: &mut R) -> Option<GoMove> {
    // Collect candidate moves (sample empty positions)
    let mut candidates: Vec<(GoMove, f64)> = Vec::new();

    // Sample up to 20 random positions
    let mut attempts = 0;
    while candidates.len() < 10 && attempts < 30 {
        let row = rng.gen_range(0..BOARD_SIZE);
        let col = rng.gen_range(0..BOARD_SIZE);

        if game.board[row][col].is_none() && is_legal_move(game, row, col) {
            let mv = GoMove::Place(row, col);
            let score = score_move(game, row, col);
            // Ensure positive weights
            candidates.push((mv, (score + 100.0).max(1.0)));
        }
        attempts += 1;
    }

    if candidates.is_empty() {
        // Fallback: linear scan
        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                if game.board[row][col].is_none() && is_legal_move(game, row, col) {
                    return Some(GoMove::Place(row, col));
                }
            }
        }
        return None;
    }

    // Weighted random selection
    let total_weight: f64 = candidates.iter().map(|(_, w)| w).sum();
    let mut pick = rng.gen_range(0.0..total_weight);

    for (mv, weight) in candidates {
        pick -= weight;
        if pick <= 0.0 {
            return Some(mv);
        }
    }

    // Shouldn't reach here, but return first candidate
    candidates.first().map(|(mv, _)| *mv)
}

/// Backpropagate result through the tree.
fn backpropagate(nodes: &mut [MctsNode], start_idx: usize, winner: Option<Stone>) {
    let mut node_idx = Some(start_idx);

    while let Some(idx) = node_idx {
        nodes[idx].visits += 1;

        if let Some(w) = winner {
            if nodes[idx].player_just_moved == w {
                nodes[idx].wins += 1.0;
            }
        } else {
            nodes[idx].wins += 0.5;
        }

        node_idx = nodes[idx].parent;
    }
}

/// Select the best move (most visited, win rate as tiebreaker).
fn select_best_move(nodes: &[MctsNode]) -> GoMove {
    nodes[0]
        .children
        .iter()
        .max_by(|&&a, &&b| {
            let visits_cmp = nodes[a].visits.cmp(&nodes[b].visits);
            if visits_cmp == std::cmp::Ordering::Equal {
                // Tiebreak by win rate
                let rate_a = if nodes[a].visits > 0 {
                    nodes[a].wins / nodes[a].visits as f32
                } else {
                    0.0
                };
                let rate_b = if nodes[b].visits > 0 {
                    nodes[b].wins / nodes[b].visits as f32
                } else {
                    0.0
                };
                rate_a.partial_cmp(&rate_b).unwrap()
            } else {
                visits_cmp
            }
        })
        .and_then(|&idx| nodes[idx].move_taken)
        .unwrap_or(GoMove::Pass)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcts_returns_move() {
        let game = GoGame::new(GoDifficulty::Novice);
        let mut rng = rand::thread_rng();
        let mv = mcts_best_move(&game, &mut rng);
        match mv {
            GoMove::Place(r, c) => {
                assert!(r < BOARD_SIZE);
                assert!(c < BOARD_SIZE);
            }
            GoMove::Pass => {}
        }
    }

    #[test]
    fn test_mcts_prefers_capture() {
        let mut game = GoGame::new(GoDifficulty::Apprentice);
        // Set up: Black stone at (4,4) can be captured at (4,5)
        game.board[4][4] = Some(Stone::Black);
        game.board[3][4] = Some(Stone::White);
        game.board[5][4] = Some(Stone::White);
        game.board[4][3] = Some(Stone::White);
        game.current_player = Stone::White;

        let mut rng = rand::thread_rng();
        let mv = mcts_best_move(&game, &mut rng);

        // Should capture at (4,5)
        assert_eq!(mv, GoMove::Place(4, 5), "MCTS should find the capture");
    }

    #[test]
    fn test_mcts_avoids_obvious_suicide() {
        let mut game = GoGame::new(GoDifficulty::Novice);
        game.board[3][4] = Some(Stone::Black);
        game.board[5][4] = Some(Stone::Black);
        game.board[4][3] = Some(Stone::Black);
        game.board[4][5] = Some(Stone::Black);
        game.current_player = Stone::White;

        let mut rng = rand::thread_rng();
        let mv = mcts_best_move(&game, &mut rng);

        assert_ne!(mv, GoMove::Place(4, 4), "Should not suicide");
    }

    #[test]
    fn test_mcts_defends_atari() {
        let mut game = GoGame::new(GoDifficulty::Apprentice);
        // White stone at (4,4) in atari, can escape at (4,5)
        game.board[4][4] = Some(Stone::White);
        game.board[3][4] = Some(Stone::Black);
        game.board[5][4] = Some(Stone::Black);
        game.board[4][3] = Some(Stone::Black);
        game.current_player = Stone::White;

        let mut rng = rand::thread_rng();
        let mv = mcts_best_move(&game, &mut rng);

        // Should save at (4,5)
        assert_eq!(mv, GoMove::Place(4, 5), "MCTS should save the stone");
    }
}
