// base implementations for all caching traits including TranspositionTable

use super::TranspositionTable;

use std::collections::HashMap;

// base implementation of TranspositionTable

pub struct NoTranspositionTable {}

impl<State, ID> TranspositionTable<State, ID> for NoTranspositionTable {
    fn new(_expected_num_nodes: usize) -> Self {
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
    fn new(expected_num_nodes: usize) -> Self {
        Self {
            table: HashMap::with_capacity(expected_num_nodes),
        }
    }

    fn get(&self, state: &State) -> Option<&ID> {
        self.table.get(state)
    }

    fn insert(&mut self, state: State, value: ID) {
        // make sure that always a state is given to insert.
        assert!(self.table.insert(state, value).is_none());
    }

    fn clear(&mut self) {
        self.table.clear();
    }
}
