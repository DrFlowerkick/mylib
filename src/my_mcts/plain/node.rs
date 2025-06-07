// plain implementation of MCTSNode

use super::super::{ExpansionPolicy, Heuristic, MCTSGame, MCTSNode, UCTPolicy, UTCCache};

pub struct PlainNode<G, UP, UC, EP, H>
where
    G: MCTSGame,
    UP: UCTPolicy<G>,
    UC: UTCCache<G, UP>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
{
    pub state: G::State,
    pub visits: usize,
    pub accumulated_value: f32,
    pub utc_cache: UC,
    pub expansion_policy: EP,

    phantom: std::marker::PhantomData<(UP, H)>,
}

impl<G, UP, UC, EP, H> MCTSNode<G, EP, H> for PlainNode<G, UP, UC, EP, H>
where
    G: MCTSGame,
    UP: UCTPolicy<G>,
    UC: UTCCache<G, UP>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
{
    fn new(state: G::State, expansion_policy: EP) -> Self {
        PlainNode {
            expansion_policy,
            state,
            visits: 0,
            accumulated_value: 0.0,
            utc_cache: UC::new(),
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
    fn calc_utc(
        &mut self,
        parent_visits: usize,
        perspective_player: G::Player,
        mcts_config: &G::Config,
    ) -> f32 {
        if self.visits == 0 {
            return f32::INFINITY;
        }
        let exploitation = self.utc_cache.get_exploitation(
            self.visits,
            self.accumulated_value,
            G::last_player(&self.state),
            perspective_player,
        );
        self.utc_cache
            .update_exploration(self.visits, parent_visits, mcts_config);
        let exploration = self
            .utc_cache
            .get_exploration(self.visits, parent_visits, mcts_config);
        exploitation + exploration
    }
    fn should_expand(
        &self,
        visits: usize,
        num_parent_children: usize,
        mcts_config: &G::Config,
        heuristic_config: &H::Config,
    ) -> bool {
        self.expansion_policy.should_expand(
            visits,
            num_parent_children,
            mcts_config,
            heuristic_config,
        )
    }
    fn expandable_moves(
        &mut self,
        num_parent_children: usize,
        mcts_config: &G::Config,
        heuristic_config: &H::Config,
    ) -> Vec<G::Move> {
        self.expansion_policy.expandable_moves(
            self.visits,
            num_parent_children,
            &self.state,
            mcts_config,
            heuristic_config,
        )
    }
}
