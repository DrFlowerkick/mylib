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
    fn should_expand(
        &self,
        visits: usize,
        num_children: usize,
        mcts_config: &A::Config,
        heuristic_config: &H::Config,
    ) -> bool;
    fn pop_expandable_move(
        &mut self,
        visits: usize,
        num_children: usize,
        force: bool,
        mcts_config: &A::Config,
        heuristic_config: &H::Config,
    ) -> Option<G::Move>;
    fn is_fully_expanded(&self) -> bool;
    // solver
    fn set_outcome(&mut self, outcome: G::Outcome);
    fn get_outcome(&self) -> Option<G::Outcome>;
    fn filter_selection(&self, child: &Self) -> bool;
    fn update_solver(&mut self, child: &Self) -> bool;

    fn get_solver(&self) -> &A::Solver;
    fn update_solver_old(&mut self, update: A::Solver) -> bool;
}
