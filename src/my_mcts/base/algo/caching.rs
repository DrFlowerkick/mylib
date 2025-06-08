// base implementations for all caching traits including TranspositionTable

use super::TranspositionTable;

use std::collections::HashMap;

// base implementation of TranspositionTable

pub struct NoTranspositionTable {}

impl<State, ID> TranspositionTable<State, ID> for NoTranspositionTable {
    fn new() -> Self {
        NoTranspositionTable {}
    }
}

pub struct TranspositionHashMap<State, ID>
where
    State: Eq + std::hash::Hash,
{
    pub table: HashMap<State, ID>,
}

impl<State, ID> TranspositionTable<State, ID> for TranspositionHashMap<State, ID>
where
    State: Eq + std::hash::Hash,
{
    fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    fn get(&self, state: &State) -> Option<&ID> {
        self.table.get(state)
    }

    fn insert(&mut self, state: State, value: ID) {
        self.table.insert(state, value);
    }
}
