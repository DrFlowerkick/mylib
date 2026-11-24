// Implementations of SimulationPolicy

use super::{Heuristic, HeuristicConfig, MCTSConfig, MCTSGame, SimulationPolicy, SimulationScore};

#[derive(Clone)]
pub struct DefaultSimulationPolicy {}

impl<G, S, H, Config> SimulationPolicy<G, S, H, Config> for DefaultSimulationPolicy
where
    G: MCTSGame,
    S: SimulationScore<G>,
    H: Heuristic<G, S>,
    Config: MCTSConfig<G::Player>,
{
}

#[derive(Clone)]
pub struct EarlyCutoff {}

impl<G, S, H, Config> SimulationPolicy<G, S, H, Config> for EarlyCutoff
where
    G: MCTSGame,
    S: SimulationScore<G>,
    H: Heuristic<G, S>,
    Config: MCTSConfig<G::Player>,
{
    fn should_cutoff(
        _current_turn: usize,
        state: &G::State,
        depth: usize,
        mcts_config: &Config,
        heuristic_config: &H::Config,
    ) -> Option<S> {
        let heuristic = H::evaluate_state(state, heuristic_config);
        if depth >= mcts_config.early_cut_off_depth() {
            // previous player, because the current state results from the move of the previous player
            // and we want to check, if the move of the previous player results in a good enough score
            // to cutoff the simulation
            Some(heuristic)
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct HeuristicCutoff {}

impl<G, S, H, Config> SimulationPolicy<G, S, H, Config> for HeuristicCutoff
where
    G: MCTSGame,
    S: SimulationScore<G>,
    H: Heuristic<G, S>,
    Config: MCTSConfig<G::Player>,
{
    fn should_cutoff(
        current_turn: usize,
        state: &G::State,
        depth: usize,
        mcts_config: &Config,
        heuristic_config: &H::Config,
    ) -> Option<S> {
        if let Some(max_turn) = heuristic_config.max_turn()
            && current_turn > max_turn
        {
            // Fallback to "no cut off", if current_turn exceeds max_turn
            None
        } else {
            // previous player, because the current state results from the move of the previous player
            // and we want to check, if the move of the previous player results in a good enough score
            // to cutoff the simulation
            let heuristic = H::evaluate_state(state, heuristic_config);
            let previous_player_score = heuristic.get_player_score(G::previous_player(state));
            if depth >= mcts_config.early_cut_off_depth()
                || previous_player_score <= heuristic_config.early_cut_off_lower_bound()
                || previous_player_score >= heuristic_config.early_cut_off_upper_bound()
            {
                Some(heuristic)
            } else {
                None
            }
        }
    }
}
