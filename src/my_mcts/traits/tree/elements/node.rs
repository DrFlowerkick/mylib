// MCTSNode defines the node structure of a tree node of the MCTS tree

use super::{ExpansionPolicy, Heuristic, MCTSConfig, MCTSGame, UCTPolicy, UTCCache};

pub trait MCTSNode<G, H, MC, UP, EP>: Clone + Sync + Send
where
    G: MCTSGame,
    H: Heuristic<G>,
    MC: MCTSConfig<G::Player>,
    UP: UCTPolicy<G, MC>,
    EP: ExpansionPolicy<G, H, MC>,
{
    type Cache: UTCCache<G, UP, MC>;

    fn new(state: G::State) -> Self;
    fn get_state(&self) -> &G::State;


    // ToDo: handling utc related values and functions will be encapsulated in UTCCache
    // Node will only provide access to UTCCache
    fn get_visits(&self) -> usize;
    fn get_accumulated_value(&self) -> f32;
    fn update_stats(&mut self, result: f32);
    fn calc_utc(&mut self, parent_visits: usize, mcts_config: &MC) -> f32;
}
