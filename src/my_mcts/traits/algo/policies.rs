// policy traits of mcts

use super::{Heuristic, MCTSConfig, MCTSGame};

pub trait UCTPolicy<G: MCTSGame, Config: MCTSConfig<G::Player>>: Clone + Sync + Send {
    // calculates the exploitation score from the view of the perspective player
    fn exploitation_score(accumulated_value: f32, visits: usize, last_player: G::Player) -> f32 {
        if visits == 0 {
            return 0.0;
        }
        let raw = accumulated_value / visits as f32;
        // this works only for 2 player games
        if last_player == G::perspective_player() {
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
        if visits == 0 {
            return f32::INFINITY;
        }
        mcts_config.exploration_constant() * ((parent_visits as f32).ln() / visits as f32).sqrt()
    }
}

pub trait ExpansionPolicy<G: MCTSGame, H: Heuristic<G>, Config: MCTSConfig<G::Player>>:
    Clone + Sync + Send + Default
{
    fn should_expand(
        &self,
        _visits: usize,
        _num_children: usize,
        _mcts_config: &Config,
        _heuristic_config: &H::Config,
    ) -> bool {
        false
    }
    fn pop_expandable_move(
        &mut self,
        state: &G::State,
        visits: usize,
        num_children: usize,
        mcts_config: &Config,
        heuristic_config: &H::Config,
    ) -> Option<G::Move>;
}

// return value ist heuristic value at cutoff
pub trait SimulationPolicy<G: MCTSGame, H: Heuristic<G>, Config: MCTSConfig<G::Player>>:
    Clone + Sync + Send
{
    fn should_cutoff(
        _state: &G::State,
        _depth: usize,
        _perspective_player: Option<G::Player>,
        _mcts_config: &Config,
        _heuristic_config: &H::Config,
    ) -> Option<f32> {
        None
    }
}

// ToDo: add a SelectMovePolicy trait
// inputs are tree and path_node of corresponding move, returns Ordering,
// which can be used by max_by_key()
