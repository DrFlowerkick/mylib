// Use TranspositionTable to cache all states with their corresponding Node IDs

pub trait TranspositionTable<State, ID>: Clone + Sync + Send {
    fn new(expected_num_nodes: usize) -> Self;
    fn get(&self, _state: &State) -> Option<&ID> {
        None
    }
    fn insert(&mut self, _state: State, _value: ID) {}
    fn clear(&mut self) {}
}
