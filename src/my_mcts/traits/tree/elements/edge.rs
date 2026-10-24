// trait for edges of a MCTSTree

use super::{ExpansionPolicy, Heuristic, MCTSConfig, MCTSGame, UCTPolicy, UTCCache};

pub trait MCTSEdge<G, H, MC, UP, EP>: Clone + Sync + Send
where
    G: MCTSGame,
    H: Heuristic<G>,
    MC: MCTSConfig<G::Player>,
    UP: UCTPolicy<G, MC>,
    EP: ExpansionPolicy<G, H, MC>,
{
    type Cache: UTCCache<G, UP, MC>;

    fn new(mv: G::Move) -> Self;
    fn get_move(&self) -> &G::Move;

    // ToDo: handling utc related values and functions will be encapsulated in UTCCache
    // Edge will only provide access to UTCCache
    // Or perhaps not. I will think about it
}
