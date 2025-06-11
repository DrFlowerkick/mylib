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
    ExpansionPolicy, Heuristic, MCTSConfig, MCTSGame, MCTSTree, SimulationPolicy,
    TranspositionTable, UCTPolicy,
};

pub trait MCTSAlgo<G: MCTSGame, H: Heuristic<G>>: Sized {
    type Tree: MCTSTree<G, H, Self>;
    type NodeID: Copy + Eq + std::fmt::Debug;
    type Config: MCTSConfig;
    type TranspositionTable: TranspositionTable<G::State, Self::NodeID>;
    type UTC: UCTPolicy<G, Self::Config>;
    type Expansion: ExpansionPolicy<G, H, Self::Config>;
    type Simulation: SimulationPolicy<G, H, Self::Config>;

    fn set_root(&mut self, state: &G::State) -> bool;
    fn reset_root(&mut self, state: &G::State);
    fn iterate(&mut self);
    fn select_move(&self) -> &G::Move;
}
