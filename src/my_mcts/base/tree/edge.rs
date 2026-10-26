// base implementation of MCTSEdge

use super::{Heuristic, MCTSAlgo, MCTSEdge, MCTSGame, UCTCache};

#[derive(Clone)]
pub struct BaseEdge<G, H, A, UC>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    UC: UCTCache<G, A::UCT, A::Config>,
{
    pub mv: G::Move,
    pub uct_cache: UC,
    phantom: std::marker::PhantomData<(H, A)>,
}

impl<G, H, A, UC> MCTSEdge<G, H, A> for BaseEdge<G, H, A, UC>
where
    G: MCTSGame,
    H: Heuristic<G>,
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

    fn calc_exploitation_score(&self, last_player: G::Player) -> f32 {
        self.uct_cache.calc_exploitation_score(last_player)
    }

    fn calc_exploration_score(
        &mut self,
        last_player: G::Player,
        parent_visits: usize,
        mcts_config: &A::Config,
    ) -> f32 {
        self.uct_cache
            .calc_exploration_score(parent_visits, mcts_config, last_player)
    }

    fn update_stats(&mut self, state: &G::State, result: f32) {
        self.uct_cache.update_stats(result, G::last_player(state));
    }

    fn get_visits(&self) -> Option<usize> {
        self.uct_cache.get_visits()
    }

    fn get_accumulated_value(&self) -> Option<f32> {
        self.uct_cache.get_accumulated_value()
    }
}
