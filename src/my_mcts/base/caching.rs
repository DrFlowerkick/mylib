// base implementations for all caching traits including TranspositionTable

use super::super::{
    ExpansionPolicy, GameCache, Heuristic, HeuristicCache, MCTSGame, MCTSNode, MCTSTree,
    TranspositionTable, UCTPolicy, UTCCache,
};

use std::collections::HashMap;

// base implementation of UTCCache
pub struct NoUTCCache;

impl<G: MCTSGame, UP: UCTPolicy<G>> UTCCache<G, UP> for NoUTCCache {
    fn new() -> Self {
        NoUTCCache
    }

    fn update_exploitation(&mut self, _v: usize, _a: f32, _l: G::Player, _p: G::Player) {}
    fn get_exploitation(
        &self,
        visits: usize,
        acc_value: f32,
        last_player: G::Player,
        perspective_player: G::Player,
    ) -> f32 {
        UP::exploitation_score(acc_value, visits, last_player, perspective_player)
    }

    fn update_exploration(&mut self, _v: usize, _p: usize, _mc: &G::Config) {}
    fn get_exploration(&self, visits: usize, parent_visits: usize, mcts_config: &G::Config) -> f32 {
        UP::exploration_score(visits, parent_visits, mcts_config)
    }
}

pub struct CachedUTC {
    exploitation: f32,
    exploration: f32,
    last_parent_visits: usize,
}

impl<G: MCTSGame, UP: UCTPolicy<G>> UTCCache<G, UP> for CachedUTC {
    fn new() -> Self {
        CachedUTC {
            exploitation: 0.0,
            exploration: 0.0,
            last_parent_visits: 0,
        }
    }

    fn update_exploitation(
        &mut self,
        visits: usize,
        acc_value: f32,
        last_player: G::Player,
        perspective_player: G::Player,
    ) {
        self.exploitation =
            UP::exploitation_score(acc_value, visits, last_player, perspective_player);
    }

    fn get_exploitation(&self, _v: usize, _a: f32, _c: G::Player, _p: G::Player) -> f32 {
        self.exploitation
    }

    fn update_exploration(&mut self, visits: usize, parent_visits: usize, mcts_config: &G::Config) {
        if self.last_parent_visits != parent_visits {
            self.exploration = UP::exploration_score(visits, parent_visits, mcts_config);
            self.last_parent_visits = parent_visits;
        }
    }

    fn get_exploration(&self, _v: usize, _p: usize, _mc: &G::Config) -> f32 {
        self.exploration
    }
}

// Use NoGameCache, if caching of game date is not feasible.
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

// Use NoHeuristicCache, if caching of heuristic data is not feasible.
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

// base implementation of TranspositionTable

pub struct NoTranspositionTable {}

impl<G, N, T, EP, H> TranspositionTable<G, N, T, EP, H> for NoTranspositionTable
where
    G: MCTSGame,
    G::State: Eq + std::hash::Hash,
    N: MCTSNode<G, EP, H>,
    T: MCTSTree<G, N, EP, H>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
{
    fn new() -> Self {
        NoTranspositionTable {}
    }
}

pub struct TranspositionHashMap<G, N, T, EP, H>
where
    G: MCTSGame,
    G::State: Eq + std::hash::Hash,
    N: MCTSNode<G, EP, H>,
    T: MCTSTree<G, N, EP, H>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
{
    pub table: HashMap<G::State, T::ID>,
}

impl<G, N, T, EP, H> TranspositionTable<G, N, T, EP, H> for TranspositionHashMap<G, N, T, EP, H>
where
    G: MCTSGame,
    G::State: Eq + std::hash::Hash,
    N: MCTSNode<G, EP, H>,
    T: MCTSTree<G, N, EP, H>,
    EP: ExpansionPolicy<G, H>,
    H: Heuristic<G>,
{
    fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    fn get(&self, state: &G::State) -> Option<&T::ID> {
        self.table.get(state)
    }

    fn insert(&mut self, state: G::State, value: T::ID) {
        self.table.insert(state, value);
    }
}
