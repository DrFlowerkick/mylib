// node trait of mcts

use super::*;

pub trait MCTSNode<G: MCTSGame, EP: ExpansionPolicy<G, H>, H: Heuristic<G>>
where
    G: MCTSGame,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
{
    fn new(state: G::State, expansion_policy: EP) -> Self;
    fn get_state(&self) -> &G::State;
    fn get_visits(&self) -> usize;
    fn get_accumulated_value(&self) -> f32;
    fn update_stats(&mut self, result: f32);
    fn calc_utc(
        &mut self,
        parent_visits: usize,
        perspective_player: G::Player,
        mcts_config: &G::Config,
    ) -> f32;
    fn should_expand(
        &self,
        visits: usize,
        num_parent_children: usize,
        mcts_config: &G::Config,
        heuristic_config: &H::Config,
    ) -> bool;
    fn expandable_moves(
        &mut self,
        num_parent_children: usize,
        mcts_config: &G::Config,
        heuristic_config: &H::Config,
    ) -> Vec<G::Move>;
}
