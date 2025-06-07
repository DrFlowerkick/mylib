// caching traits for mcts including TranspositionTable

use super::*;

pub trait UTCCache<G: MCTSGame, UP: UCTPolicy<G>> {
    fn new() -> Self;

    fn update_exploitation(
        &mut self,
        visits: usize,
        acc_value: f32,
        last_player: G::Player,
        perspective_player: G::Player,
    );

    fn get_exploitation(
        &self,
        visits: usize,
        acc_value: f32,
        last_player: G::Player,
        perspective_player: G::Player,
    ) -> f32;

    fn update_exploration(&mut self, visits: usize, parent_visits: usize, mcts_config: &G::Config);

    fn get_exploration(&self, visits: usize, parent_visits: usize, mcts_config: &G::Config) -> f32;
}

pub trait GameCache<State, Move> {
    fn new() -> Self;
    fn get_applied_state(&self, _state: &State, _mv: &Move) -> Option<&State> {
        None
    }
    fn insert_applied_state(&mut self, _state: &State, _mv: &Move, _result: State) {}
    fn get_terminal_value(&self, _state: &State) -> Option<&Option<f32>> {
        None
    }
    fn insert_terminal_value(&mut self, _state: &State, _value: Option<f32>) {}
}

pub trait HeuristicCache<State, Move> {
    fn new() -> Self;
    fn get_intermediate_score(&self, _state: &State) -> Option<f32> {
        None
    }
    fn insert_intermediate_score(&mut self, _state: &State, _score: f32) {}
    fn get_move_score(&self, _state: &State, _mv: &Move) -> Option<f32> {
        None
    }
    fn insert_move_score(&mut self, _state: &State, _mv: &Move, _score: f32) {}
}

pub trait TranspositionTable<G, N, T, EP, H>
where
    G: MCTSGame,
    G::State: Eq + std::hash::Hash,
    N: MCTSNode<G, EP, H>,
    T: MCTSTree<G, N, EP, H>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
{
    fn new() -> Self;
    fn get(&self, _state: &G::State) -> Option<&T::ID> {
        None
    }
    fn insert(&mut self, _state: G::State, _value: T::ID) {}
}
