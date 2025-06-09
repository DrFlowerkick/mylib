// MCTSNode defines the node structure of a tree node of the MCTS tree

use super::{ExpansionPolicy, Heuristic, MCTSConfig, MCTSGame, UCTPolicy, UTCCache};

pub trait MCTSNode<G, H, MC, UP, EP>
where
    G: MCTSGame,
    H: Heuristic<G>,
    MC: MCTSConfig,
    UP: UCTPolicy<G, MC>,
    EP: ExpansionPolicy<G, H, MC>,
{
    type Cache: UTCCache<G, UP, MC>;

    fn new(state: G::State, expansion_policy: EP) -> Self;
    fn get_state(&self) -> &G::State;
    fn get_visits(&self) -> usize;
    fn get_accumulated_value(&self) -> f32;
    fn update_stats(&mut self, result: f32);
    fn calc_utc(
        &mut self,
        parent_visits: usize,
        perspective_player: G::Player,
        mcts_config: &MC,
    ) -> f32;
    fn should_expand(
        &self,
        visits: usize,
        num_parent_children: usize,
        mcts_config: &MC,
        heuristic_config: &H::Config,
    ) -> bool;
    fn expandable_moves(
        &mut self,
        num_parent_children: usize,
        mcts_config: &MC,
        heuristic_config: &H::Config,
    ) -> Vec<G::Move>;
}
