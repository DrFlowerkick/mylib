// Use NoHeuristicCache, if caching of heuristic data is not feasible.

use super::HeuristicCache;

pub struct NoHeuristicCache<State, Move> {
    phantom: std::marker::PhantomData<(State, Move)>,
}

impl<State, Move> HeuristicCache<State, Move> for NoHeuristicCache<State, Move> {
    fn new() -> Self {
        NoHeuristicCache {
            phantom: std::marker::PhantomData,
        }
    }
}
