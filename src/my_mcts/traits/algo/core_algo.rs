// MCTSAlgo is the core trait of the MCTS algorithm. It defines types for
//  - tree
//  - policies
//  - configuration of policies
//  - caching
//
// MCTSAlgo depends upon MCTSGame for State and Move type amongst others
// While MCTSAlgo policies ExpansionPolicy and SimulationsPolicy depend upon Heuristic,
// NoHeuristic can be used if ho heuristic is required or feasible.

use super::{
    ExpansionPolicy, Heuristic, MCTSConfig, MCTSGame, MCTSTree, SimulationPolicy, UCTPolicy,
};

pub trait MCTSAlgo<G: MCTSGame, H: Heuristic<G>>: Sized + Clone + Sync + Send {
    type Config: MCTSConfig<G::Player>;
    type UTC: UCTPolicy<G, Self::Config>;
    type Expansion: ExpansionPolicy<G, H, Self::Config>;
    type Simulation: SimulationPolicy<G, H, Self::Config>;

    fn set_root(
        &self,
        state: &G::State,
        tree: &mut impl MCTSTree<G, H, Self>,
        max_depth: usize,
    ) -> bool;
    // ToDo: do we really need reset_root() in API of MCTSAlgo?
    fn reset_root(&self, state: &G::State, tree: &mut impl MCTSTree<G, H, Self>);
    fn iterate(&self, tree: &mut impl MCTSTree<G, H, Self>);
    fn select_move<'a>(&'a self, tree: &'a impl MCTSTree<G, H, Self>) -> &'a G::Move;
}
