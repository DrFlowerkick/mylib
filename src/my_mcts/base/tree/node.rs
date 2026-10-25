// plain implementation of MCTSNode

use super::{Heuristic, MCTSAlgo, MCTSGame, MCTSNode, UCTCache};

#[derive(Clone)]
pub struct BaseNode<G, H, A, UC>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    UC: UCTCache<G, A::UCT, A::Config>,
{
    pub state: G::State,
    pub uct_cache: UC,
    pub expansion_policy: A::Expansion,
}

impl<G, H, A, UC> MCTSNode<G, H, A> for BaseNode<G, H, A, UC>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    UC: UCTCache<G, A::UCT, A::Config>,
{
    fn new(state: G::State) -> Self {
        BaseNode {
            state,
            uct_cache: UC::default(),
            expansion_policy: A::Expansion::default(),
        }
    }

    fn get_state(&self) -> &G::State {
        &self.state
    }

    fn calc_uct(&mut self, parent_visits: usize, mcts_config: &A::Config) -> f32 {
        self.uct_cache
            .calc_uct(parent_visits, mcts_config, G::last_player(&self.state))
    }

    fn update_stats(&mut self, result: f32) {
        self.uct_cache
            .update_stats(result, G::last_player(&self.state));
    }

    fn get_visits(&self) -> Option<usize> {
        self.uct_cache.get_visits()
    }

    fn get_accumulated_value(&self) -> Option<f32> {
        self.uct_cache.get_accumulated_value()
    }
}
