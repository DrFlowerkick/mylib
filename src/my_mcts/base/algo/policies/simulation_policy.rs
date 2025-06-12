// Implementations of SimulationPolicy

use super::{Heuristic, HeuristicConfig, MCTSConfig, MCTSGame, SimulationPolicy};

pub struct DefaultSimulationPolicy {}

impl<G, H, Config> SimulationPolicy<G, H, Config> for DefaultSimulationPolicy
where
    G: MCTSGame,
    H: Heuristic<G>,
    Config: MCTSConfig<G::Player>,
{
}

pub struct EarlyCutoff {}

impl<G, H, Config> SimulationPolicy<G, H, Config> for EarlyCutoff
where
    G: MCTSGame,
    H: Heuristic<G>,
    Config: MCTSConfig<G::Player>,
{
    fn should_cutoff(
        state: &G::State,
        depth: usize,
        game_cache: &mut G::Cache,
        heuristic_cache: &mut H::Cache,
        perspective_player: Option<G::Player>,
        mcts_config: &Config,
        heuristic_config: &H::Config,
    ) -> Option<f32> {
        let heuristic = H::evaluate_state(
            state,
            game_cache,
            heuristic_cache,
            perspective_player,
            heuristic_config,
        );
        if depth >= mcts_config.early_cut_off_depth() {
            Some(heuristic)
        } else {
            None
        }
    }
}

pub struct HeuristicCutoff {}

impl<G, H, Config> SimulationPolicy<G, H, Config> for HeuristicCutoff
where
    G: MCTSGame,
    H: Heuristic<G>,
    Config: MCTSConfig<G::Player>,
{
    fn should_cutoff(
        state: &G::State,
        depth: usize,
        game_cache: &mut G::Cache,
        heuristic_cache: &mut H::Cache,
        perspective_player: Option<G::Player>,
        mcts_config: &Config,
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
