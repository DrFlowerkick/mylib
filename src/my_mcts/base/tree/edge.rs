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
    phantom: std::marker::PhantomData<(A, H)>,
}

impl<G, H, A, UC> MCTSEdge<G, H, A> for BaseEdge<G, H, A, UC>
where
    G: MCTSGame,
    H: Heuristic<G>,
    A: MCTSAlgo<G, H>,
    UC: UCTCache<G, A::UCT, A::Config>,
{
    type Cache = UC;

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
}
