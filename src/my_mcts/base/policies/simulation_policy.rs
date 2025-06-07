// Implementations of SimulationPolicy

use super::super::super::{Heuristic, HeuristicConfig, MCTSConfig, MCTSGame, SimulationPolicy};

pub struct DefaultSimulationPolicy {}

impl<G: MCTSGame, H: Heuristic<G>> SimulationPolicy<G, H> for DefaultSimulationPolicy {}

pub struct HeuristicCutoff {}

impl<G: MCTSGame, H: Heuristic<G>> SimulationPolicy<G, H> for HeuristicCutoff {
    fn should_cutoff(
        state: &G::State,
        depth: usize,
        game_cache: &mut G::Cache,
        heuristic_cache: &mut H::Cache,
        perspective_player: Option<G::Player>,
        mcts_config: &G::Config,
        heuristic_config: &H::Config,
    ) -> Option<f32> {
        let heuristic = H::evaluate_state(
            state,
            game_cache,
            heuristic_cache,
            perspective_player,
            heuristic_config,
        );
        if depth >= mcts_config.early_cut_off_depth()
            || heuristic <= heuristic_config.early_cut_off_lower_bound()
            || heuristic >= heuristic_config.early_cut_off_upper_bound()
        {
            Some(heuristic)
        } else {
            None
        }
    }
}
