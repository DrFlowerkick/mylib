// base implementation of MCTSEdge

use super::{Heuristic, MCTSAlgo, MCTSEdge, MCTSGame, UCTCache};

#[derive(Clone)]
pub struct BaseEdge<G, H, A, UC>
where
    G: MCTSGame,
    H: Heuristic<G, A::Score>,
    A: MCTSAlgo<G, H>,
    UC: UCTCache<G, A::UCT, A::Config>,
{
    mv: G::Move,
    uct_cache: UC,
    phantom: std::marker::PhantomData<(H, A)>,
}

impl<G, H, A, UC> MCTSEdge<G, H, A> for BaseEdge<G, H, A, UC>
where
    G: MCTSGame,
    H: Heuristic<G, A::Score>,
    A: MCTSAlgo<G, H>,
    UC: UCTCache<G, A::UCT, A::Config>,
{
    fn new(mv: G::Move) -> Self {
        Self {
            mv,
            uct_cache: UC::default(),
            phantom: std::marker::PhantomData,
        }
    }

    fn get_move(&self) -> &G::Move {
        &self.mv
    }

    fn calc_exploitation_score(&self) -> f32 {
        self.uct_cache.calc_exploitation_score()
    }

    fn calc_exploration_score(
        &mut self,
        parent_visits: usize,
        mcts_config: &A::Config,
        acting_player: G::Player,
    ) -> f32 {
        self.uct_cache
            .calc_exploration_score(parent_visits, mcts_config, acting_player)
    }

    fn update_stats(&mut self, result: f32) {
        self.uct_cache.update_stats(result);
    }

    fn get_visits(&self) -> Option<usize> {
        self.uct_cache.get_visits()
    }

    fn get_accumulated_value(&self) -> Option<f32> {
        self.uct_cache.get_accumulated_value()
    }
}
