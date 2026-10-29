// plain implementation of MCTSNode

use crate::my_mcts::{ExpansionPolicy, SolverPolicy};

use super::{Heuristic, MCTSAlgo, MCTSGame, MCTSNode, UCTCache};

#[derive(Clone)]
pub struct BaseNode<G, H, A, UC>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    UC: UCTCache<G, A::UCT, A::Config>,
{
    state: G::State,
    uct_cache: UC,
    expansion_policy: A::Expansion,
    solver: A::Solver,
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
            solver: A::Solver::default(),
        }
    }

    fn get_state(&self) -> &G::State {
        &self.state
    }

    // -------------- UCT --------------
    fn calc_exploitation_score(&self) -> f32 {
        self.uct_cache
            .calc_exploitation_score(G::last_player(&self.state))
    }

    fn calc_exploration_score(&mut self, parent_visits: usize, mcts_config: &A::Config) -> f32 {
        self.uct_cache.calc_exploration_score(
            parent_visits,
            mcts_config,
            G::last_player(&self.state),
        )
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

    // -------------- Expansion --------------
    fn should_expand(
        &self,
        visits: usize,
        num_children: usize,
        mcts_config: &A::Config,
        heuristic_config: &H::Config,
    ) -> bool {
        self.expansion_policy
            .should_expand(visits, num_children, mcts_config, heuristic_config)
    }

    fn pop_expandable_move(
        &mut self,
        visits: usize,
        num_children: usize,
        mcts_config: &A::Config,
        heuristic_config: &H::Config,
    ) -> Option<G::Move> {
        let (state, ep) = (&self.state, &mut self.expansion_policy);
        ep.pop_expandable_move(state, visits, num_children, mcts_config, heuristic_config)
    }

    fn fully_expanded(&self) -> bool {
        self.expansion_policy.fully_expanded()
    }

    // -------------- Solver --------------
    fn set_outcome(&mut self, outcome: G::Outcome) {
        self.solver.set_solved(outcome);
    }
}
