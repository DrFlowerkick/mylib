// MCTSNode defines the node structure of a tree node of the MCTS tree

use super::{Heuristic, MCTSAlgo, MCTSGame, UTCCache};

pub trait MCTSNode<G, H, A>: Clone + Sync + Send
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
{
    type Cache: UTCCache<G, A::UTC, A::Config>;

    fn new(state: G::State) -> Self;
    fn get_state(&self) -> &G::State;

    // ToDo: handling utc related values and functions will be encapsulated in UTCCache
    // Node will only provide access to UTCCache
    fn get_visits(&self) -> usize;
    fn get_accumulated_value(&self) -> f32;
    fn update_stats(&mut self, result: f32);
    fn calc_utc(&mut self, parent_visits: usize, mcts_config: &A::Config) -> f32;
}
