// heuristic traits of mcts

use super::*;
use rand::seq::SliceRandom;

pub trait Heuristic<G: MCTSGame> {
    type Cache: HeuristicCache<G::State, G::Move>;
    type Config: HeuristicConfig;

    fn evaluate_state(
        state: &G::State,
        game_cache: &mut G::Cache,
        heuristic_cache: &mut Self::Cache,
        perspective_player: Option<G::Player>,
        heuristic_config: &Self::Config,
    ) -> f32;
    fn evaluate_move(
        state: &G::State,
        mv: &G::Move,
        game_cache: &mut G::Cache,
        heuristic_cache: &mut Self::Cache,
        heuristic_config: &Self::Config,
    ) -> f32;
    fn sort_moves(
        state: &G::State,
        moves: Vec<G::Move>,
        game_cache: &mut G::Cache,
        heuristic_cache: &mut Self::Cache,
        heuristic_config: &Self::Config,
    ) -> Vec<(f32, G::Move)> {
        let mut heuristic_moves = moves
            .into_iter()
            .map(|mv| {
                (
                    Self::evaluate_move(state, &mv, game_cache, heuristic_cache, heuristic_config),
                    mv,
                )
            })
            .collect::<Vec<_>>();
        heuristic_moves.shuffle(&mut rand::thread_rng());
        heuristic_moves
            .sort_unstable_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        heuristic_moves
    }
}

// RecursiveHeuristic is probably way to expensive for mcts
pub trait RecursiveHeuristic<G: MCTSGame>: Heuristic<G>
where
    <Self as Heuristic<G>>::Config: RecursiveHeuristicConfig,
{
    fn evaluate_state_recursive(
        state: &G::State,
        game_cache: &mut G::Cache,
        heuristic_cache: &mut Self::Cache,
        heuristic_config: &Self::Config,
        depth: usize,
        alpha: f32,
    ) -> f32 {
        let base_heuristic =
            Self::evaluate_state(state, game_cache, heuristic_cache, None, heuristic_config);

        if depth == 0 || G::evaluate(state, game_cache).is_some() {
            return base_heuristic;
        }

        let mut worst_response = f32::NEG_INFINITY;
        let next_player_alpha = alpha
            - (alpha - heuristic_config.target_alpha()) * heuristic_config.alpha_reduction_factor();
        // If no constraint on next move, this will be many moves to consider.
        // Therefore we use early exit to reduce calculation time.
        for next_player_move in G::available_moves(state) {
            let next_player_state = G::apply_move(state, &next_player_move, game_cache);

            let response_value = Self::evaluate_state_recursive(
                &next_player_state,
                game_cache,
                heuristic_cache,
                heuristic_config,
                depth - 1,
                next_player_alpha,
            );

            if response_value > worst_response {
                worst_response = response_value;
                // early exit, because next player does have guaranteed win
                if worst_response >= heuristic_config.early_exit_threshold() {
                    break;
                }
            }
        }

        // combine base heuristic with worst case response
        alpha * base_heuristic + (1.0 - alpha) * (1.0 - worst_response)
    }
}
