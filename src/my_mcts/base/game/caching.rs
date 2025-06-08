// Use NoGameCache, if caching of game date is not feasible.

use super::GameCache;

pub struct NoGameCache<State, Move> {
    phantom: std::marker::PhantomData<(State, Move)>,
}

impl<State, Move> GameCache<State, Move> for NoGameCache<State, Move> {
    fn new() -> Self {
        NoGameCache {
            phantom: std::marker::PhantomData,
        }
    }
}
