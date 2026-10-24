// trait for edges of a MCTSTree

use super::{Heuristic, MCTSAlgo, MCTSGame, UTCCache};

pub trait MCTSEdge<G, H, A>: Clone + Sync + Send
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
{
    type Cache: UTCCache<G, A::UTC, A::Config>;

    fn new(mv: G::Move) -> Self;
    fn get_move(&self) -> &G::Move;

    // ToDo: handling utc related values and functions will be encapsulated in UTCCache
    // Edge will only provide access to UTCCache
    // Or perhaps not. I will think about it
}
