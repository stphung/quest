//! Monte Carlo Tree Search AI for Go.

use super::logic::{get_legal_moves, is_legal_move, make_move};
use super::types::{GoDifficulty, GoGame, GoMove, GoResult, Stone, BOARD_SIZE};
use rand::seq::SliceRandom;
use rand::Rng;

/// UCT exploration constant
const UCT_C: f64 = 1.4;

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
}

impl MctsNode {
    fn new(
        parent: Option<usize>,
        move_taken: Option<GoMove>,
        player_just_moved: Stone,
        legal_moves: Vec<GoMove>,
    ) -> Self {
        Self {
            move_taken,
            parent,
            children: Vec::new(),
            visits: 0,
            wins: 0.0,
            untried_moves: legal_moves,
            player_just_moved,
        }
    }

    fn uct_value(&self, parent_visits: u32) -> f64 {
        if self.visits == 0 {
            return f64::INFINITY;
        }
        let exploitation = self.wins as f64 / self.visits as f64;
        let exploration = UCT_C * ((parent_visits as f64).ln() / self.visits as f64).sqrt();
        exploitation + exploration
    }
}

/// Run MCTS and return the best move.
pub fn mcts_best_move<R: Rng>(game: &GoGame, rng: &mut R) -> GoMove {
    let simulations = game.difficulty.simulation_count();
    let mut nodes: Vec<MctsNode> = Vec::with_capacity(simulations as usize);

    // Create root node
    let legal_moves = get_legal_moves(game);
    let root = MctsNode::new(
        None,
        None,
        game.current_player.opponent(), // Opponent "just moved" to create current state
        legal_moves,
    );
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
            let mv_idx = rng.gen_range(0..nodes[node_idx].untried_moves.len());
            let mv = nodes[node_idx].untried_moves.swap_remove(mv_idx);

            let current_player = game_clone.current_player;
            make_move(&mut game_clone, mv);

            let child_legal_moves = get_legal_moves(&game_clone);
            let child = MctsNode::new(Some(node_idx), Some(mv), current_player, child_legal_moves);
            let child_idx = nodes.len();
            nodes.push(child);
            nodes[node_idx].children.push(child_idx);
            node_idx = child_idx;
        }

        // Simulation: random playout
        let winner = simulate_random_game(&mut game_clone, rng);

        // Backpropagation: update statistics
        backpropagate(&mut nodes, node_idx, winner);
    }

    // Select best move (most visits)
    select_best_move(&nodes)
}

/// Select child with highest UCT value.
fn select_child(nodes: &[MctsNode], parent_idx: usize) -> usize {
    let parent_visits = nodes[parent_idx].visits;
    nodes[parent_idx]
        .children
        .iter()
        .max_by(|&&a, &&b| {
            nodes[a]
                .uct_value(parent_visits)
                .partial_cmp(&nodes[b].uct_value(parent_visits))
                .unwrap()
        })
        .copied()
        .unwrap_or(parent_idx)
}

/// Simulate a random game to completion.
fn simulate_random_game<R: Rng>(game: &mut GoGame, rng: &mut R) -> Option<Stone> {
    let mut moves_made = 0;
    const MAX_MOVES: u32 = 200; // Prevent infinite games

    while game.game_result.is_none() && moves_made < MAX_MOVES {
        let legal_moves = get_legal_moves(game);
        if legal_moves.is_empty() {
            break;
        }

        // Prefer non-pass moves during simulation
        let non_pass: Vec<_> = legal_moves
            .iter()
            .filter(|m| **m != GoMove::Pass)
            .copied()
            .collect();
        let mv = if !non_pass.is_empty() && rng.gen::<f32>() > 0.1 {
            *non_pass.choose(rng).unwrap()
        } else {
            *legal_moves.choose(rng).unwrap()
        };

        make_move(game, mv);
        moves_made += 1;
    }

    // Determine winner
    match game.game_result {
        Some(GoResult::Win) => Some(Stone::Black), // Human is Black
        Some(GoResult::Loss) => Some(Stone::White), // AI is White
        Some(GoResult::Draw) => None,
        None => {
            // Game didn't end naturally, use current score
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
}

/// Backpropagate result through the tree.
fn backpropagate(nodes: &mut [MctsNode], start_idx: usize, winner: Option<Stone>) {
    let mut node_idx = Some(start_idx);

    while let Some(idx) = node_idx {
        nodes[idx].visits += 1;

        // Add win if this node's player matches the winner
        if let Some(w) = winner {
            if nodes[idx].player_just_moved == w {
                nodes[idx].wins += 1.0;
            }
        } else {
            // Draw - half point
            nodes[idx].wins += 0.5;
        }

        node_idx = nodes[idx].parent;
    }
}

/// Select the best move (most visited child of root).
fn select_best_move(nodes: &[MctsNode]) -> GoMove {
    nodes[0]
        .children
        .iter()
        .max_by_key(|&&idx| nodes[idx].visits)
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
        // Should return some move (likely a placement, not pass on empty board)
        match mv {
            GoMove::Place(r, c) => {
                assert!(r < BOARD_SIZE);
                assert!(c < BOARD_SIZE);
            }
            GoMove::Pass => {} // Also valid
        }
    }

    #[test]
    fn test_mcts_avoids_obvious_suicide() {
        let mut game = GoGame::new(GoDifficulty::Novice);
        // Create a situation where (4,4) would be suicide for White
        game.board[3][4] = Some(Stone::Black);
        game.board[5][4] = Some(Stone::Black);
        game.board[4][3] = Some(Stone::Black);
        game.board[4][5] = Some(Stone::Black);
        game.current_player = Stone::White;

        let mut rng = rand::thread_rng();
        let mv = mcts_best_move(&game, &mut rng);

        // Should not play at (4,4) - it's suicide
        assert_ne!(mv, GoMove::Place(4, 4));
    }

    #[test]
    fn test_uct_value() {
        let node = MctsNode {
            move_taken: Some(GoMove::Place(4, 4)),
            parent: Some(0),
            children: vec![],
            visits: 10,
            wins: 5.0,
            untried_moves: vec![],
            player_just_moved: Stone::Black,
        };

        let uct = node.uct_value(100);
        // Should be roughly 0.5 (exploitation) + exploration bonus
        assert!(uct > 0.5);
        assert!(uct < 2.0);
    }

    #[test]
    fn test_unvisited_node_has_infinite_uct() {
        let node = MctsNode {
            move_taken: Some(GoMove::Place(4, 4)),
            parent: Some(0),
            children: vec![],
            visits: 0,
            wins: 0.0,
            untried_moves: vec![],
            player_just_moved: Stone::Black,
        };

        assert!(node.uct_value(100).is_infinite());
    }
}
