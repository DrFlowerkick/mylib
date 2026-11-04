// trait for edges of a MCTSTree

use super::{Heuristic, MCTSAlgo, MCTSGame};

pub trait MCTSEdge<G, H, A>: Clone + Sync + Send
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
{
    fn new(mv: G::Move) -> Self;
    fn get_move(&self) -> &G::Move;
    fn calc_exploitation_score(&self, previous_player: G::Player) -> f32;
    fn calc_exploration_score(
        &mut self,
        previous_player: G::Player,
        parent_visits: usize,
        mcts_config: &A::Config,
    ) -> f32;
    fn update_stats(&mut self, state: &G::State, result: f32);
    fn get_visits(&self) -> Option<usize>;
    fn get_accumulated_value(&self) -> Option<f32>;
}
