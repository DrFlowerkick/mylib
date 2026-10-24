// plain implementation of MCTSNode

use super::{Heuristic, MCTSAlgo, MCTSGame, MCTSNode, UTCCache};

#[derive(Clone)]
pub struct BaseNode<G, H, A, UC>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    UC: UTCCache<G, A::UTC, A::Config>,
{
    pub state: G::State,
    pub visits: usize,
    pub accumulated_value: f32,
    pub utc_cache: UC,
    pub expansion_policy: A::Expansion,
    phantom: std::marker::PhantomData<H>,
}

impl<G, H, A, UC> MCTSNode<G, H, A> for BaseNode<G, H, A, UC>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    UC: UTCCache<G, A::UTC, A::Config>,
{
    type Cache = UC;

    fn new(state: G::State) -> Self {
        BaseNode {
            state,
            visits: 0,
            accumulated_value: 0.0,
            utc_cache: UC::default(),
            expansion_policy: A::Expansion::default(),
            phantom: std::marker::PhantomData,
        }
    }
    fn get_state(&self) -> &G::State {
        &self.state
    }
    fn get_visits(&self) -> usize {
        self.visits
    }
    fn get_accumulated_value(&self) -> f32 {
        self.accumulated_value
    }
    fn update_stats(&mut self, result: f32) {
        self.visits += 1;
        self.accumulated_value += result;
        self.utc_cache.update_exploitation(
            self.visits,
            self.accumulated_value,
            G::last_player(&self.state),
            G::perspective_player(),
        );
    }
    // ToDo: this must be placed into the UTC Policy, which than although allows variations
    // like progressive bias.
    fn calc_utc(&mut self, parent_visits: usize, mcts_config: &A::Config) -> f32 {
        if self.visits == 0 {
            return f32::INFINITY;
        }
        let exploitation = self.utc_cache.get_exploitation(
            self.visits,
            self.accumulated_value,
            G::last_player(&self.state),
            G::perspective_player(),
        );
        self.utc_cache.update_exploration(
            self.visits,
            parent_visits,
            mcts_config,
            G::last_player(&self.state),
        );
        let exploration = self.utc_cache.get_exploration(
            self.visits,
            parent_visits,
            mcts_config,
            G::last_player(&self.state),
        );
        exploitation + exploration
    }
}
