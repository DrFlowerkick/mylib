// policy traits of mcts

use super::{Heuristic, MCTSConfig, MCTSGame};

pub trait UCTPolicy<G: MCTSGame, Config: MCTSConfig<G::Player>> {
    // calculates the exploitation score from the view of the perspective player
    fn exploitation_score(
        accumulated_value: f32,
        visits: usize,
        last_player: G::Player,
        perspective_player: G::Player,
    ) -> f32 {
        let raw = accumulated_value / visits as f32;
        // this works only for 2 player games
        if last_player == perspective_player {
            raw
        } else {
            1.0 - raw
        }
    }

    fn exploration_score(
        visits: usize,
        parent_visits: usize,
        mcts_config: &Config,
        _last_player: G::Player,
    ) -> f32 {
        mcts_config.exploration_constant() * ((parent_visits as f32).ln() / visits as f32).sqrt()
    }
}

pub trait ExpansionPolicy<G: MCTSGame, H: Heuristic<G>, Config: MCTSConfig<G::Player>> {
    fn new(
        state: &G::State,
        game_cache: &mut G::Cache,
        heuristic_cache: &mut H::Cache,
        heuristic_config: &H::Config,
    ) -> Self;
    fn should_expand(
        &self,
        _visits: usize,
        _num_parent_children: usize,
        _mcts_config: &Config,
        _heuristic_config: &H::Config,
    ) -> bool {
        false
    }
    fn expandable_moves(
        &mut self,
        _visits: usize,
        _num_parent_children: usize,
        state: &G::State,
        _mcts_config: &Config,
        _heuristic_config: &H::Config,
    ) -> Vec<G::Move>;
}

// return value ist heuristic value at cutoff
pub trait SimulationPolicy<G: MCTSGame, H: Heuristic<G>, Config: MCTSConfig<G::Player>> {
    fn should_cutoff(
        _state: &G::State,
        _depth: usize,
        _game_cache: &mut G::Cache,
        _heuristic_cache: &mut H::Cache,
        _perspective_player: Option<G::Player>,
        _mcts_config: &Config,
        _heuristic_config: &H::Config,
    ) -> Option<f32> {
        None
    }
}
