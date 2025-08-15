// Use NoGameCache, if caching of game date is not feasible.

use super::GameCache;

#[derive(Clone)]
pub struct NoGameCache<State, Move>
where
    State: Clone,
    Move: Clone,
{
    phantom: std::marker::PhantomData<(State, Move)>,
}

impl<State, Move> GameCache<State, Move> for NoGameCache<State, Move>
where
    State: Clone,
    Move: Clone,
{
    fn new() -> Self {
        NoGameCache {
            phantom: std::marker::PhantomData,
        }
    }
}
