// policy traits of mcts

use super::{Heuristic, MCTSConfig, MCTSGame, SimulationScore};

pub trait UCTPolicy<G: MCTSGame, Config: MCTSConfig<G::Player>>: Clone + Sync + Send {
    fn exploitation_score(accumulated_value: f32, visits: usize) -> f32 {
        if visits == 0 {
            return 0.0;
        }
        accumulated_value / visits as f32
    }

    fn exploration_score(
        visits: usize,
        parent_visits: usize,
        mcts_config: &Config,
        _acting_player: G::Player,
    ) -> f32 {
        if visits == 0 {
            return f32::INFINITY;
        }
        mcts_config.exploration_constant() * ((parent_visits as f32).ln() / visits as f32).sqrt()
    }
}

pub trait ExpansionPolicy<
    G: MCTSGame,
    S: SimulationScore<G>,
    H: Heuristic<G, S>,
    Config: MCTSConfig<G::Player>,
>: Clone + Sync + Send + Default
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
    #[allow(clippy::too_many_arguments)]
    fn pop_expandable_move(
        &mut self,
        current_turn: usize,
        state: &G::State,
        visits: usize,
        num_children: usize,
        force: bool,
        mcts_config: &Config,
        heuristic_config: &H::Config,
    ) -> Option<G::Move>;
    fn is_fully_expanded(&self) -> bool;
}

// return value ist heuristic value at cutoff
pub trait SimulationPolicy<
    G: MCTSGame,
    S: SimulationScore<G>,
    H: Heuristic<G, S>,
    Config: MCTSConfig<G::Player>,
>: Clone + Sync + Send
{
    fn should_cutoff(
        _current_turn: usize,
        _state: &G::State,
        _depth: usize,
        _mcts_config: &Config,
        _heuristic_config: &H::Config,
    ) -> Option<S> {
        None
    }
}

#[derive(Clone, Copy)]
pub struct MoveStats {
    pub visits_from_parent: usize,
    pub accumulated_value: f32,
    pub acc_visits: usize,
}

impl MoveStats {
    pub fn get_win_probability(&self) -> f32 {
        self.accumulated_value / self.acc_visits as f32
    }
}

pub trait SelectMovePolicy<G: MCTSGame>: Clone + Sync + Send {
    fn select<'a, I>(candidates: I) -> Option<(&'a G::Move, f32)>
    where
        I: Iterator<Item = (&'a G::Move, MoveStats)>;
}

pub trait SolverPolicy<G: MCTSGame>: Clone + Sync + Send + Default {
    fn set_outcome(&mut self, outcome: G::Outcome, current_player: G::Player);
    fn get_outcome(&self, _current_player: G::Player) -> Option<G::Outcome> {
        None
    }
    fn filter_selection(&self, _child: &Self) -> bool {
        true
    }
    fn update<'a>(
        &'a mut self,
        _children: impl Iterator<Item = &'a Self>,
        _is_fully_expanded: bool,
    ) -> bool {
        false
    }
    fn get_guaranteed_outcome(&self, _current_player: G::Player) -> Option<G::Outcome> {
        None
    }
    fn filter_select_move(&self, _child: &Self) -> bool {
        true
    }
}
