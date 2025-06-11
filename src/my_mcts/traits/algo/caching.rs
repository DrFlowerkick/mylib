// Use TranspositionTable to cache all states with their corresponding Node IDs

pub trait TranspositionTable<State, ID> {
    fn new() -> Self;
    fn get(&self, _state: &State) -> Option<&ID> {
        None
    }
    fn insert(&mut self, _state: State, _value: ID) {}
    fn clear(&mut self) {}
}
