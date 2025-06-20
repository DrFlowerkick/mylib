// Use NoHeuristicCache, if caching of heuristic data is not feasible.

use super::HeuristicCache;

#[derive(Clone)]
pub struct NoHeuristicCache<State, Move>
where
    State: Clone,
    Move: Clone,
{
    phantom: std::marker::PhantomData<(State, Move)>,
}

impl<State, Move> HeuristicCache<State, Move> for NoHeuristicCache<State, Move>
where
    State: Clone,
    Move: Clone,
{
    fn new() -> Self {
        NoHeuristicCache {
            phantom: std::marker::PhantomData,
        }
    }
}
