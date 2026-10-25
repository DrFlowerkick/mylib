// trait for edges of a MCTSTree

use super::{Heuristic, MCTSAlgo, MCTSGame, UCTCache};

pub trait MCTSEdge<G, H, A>: Clone + Sync + Send
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
{
    type Cache: UCTCache<G, A::UCT, A::Config>;

    fn new(mv: G::Move) -> Self;
    fn get_move(&self) -> &G::Move;

    // ToDo: handling uct related values and functions will be encapsulated in UCTCache
    // Edge will only provide access to UCTCache
    // Or perhaps not. I will think about it
}
