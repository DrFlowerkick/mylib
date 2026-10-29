// MCTSNode defines the node structure of a tree node of the MCTS tree

use super::{Heuristic, MCTSAlgo, MCTSGame};

pub trait MCTSNode<G, H, A>: Clone + Sync + Send
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
{
    fn new(state: G::State) -> Self;
    fn get_state(&self) -> &G::State;
    // uct
    fn calc_exploitation_score(&self) -> f32;
    fn calc_exploration_score(&mut self, parent_visits: usize, mcts_config: &A::Config) -> f32;
    fn update_stats(&mut self, result: f32);
    fn get_visits(&self) -> Option<usize>;
    fn get_accumulated_value(&self) -> Option<f32>;
    // expansion
    fn get_expansion_policy(&self) -> &A::Expansion;
    fn pop_expandable_move(
        &mut self,
        visits: usize,
        num_children: usize,
        mcts_config: &A::Config,
        heuristic_config: &H::Config,
    ) -> Option<G::Move>;
    // solver
    fn get_solver(&self) -> &A::Solver;
    fn set_outcome(&mut self, outcome: G::Outcome);
    fn set_solver(&mut self, solver: A::Solver);
}
