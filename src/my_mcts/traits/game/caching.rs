// GameCache provides the ability to cache
// 1. current state and a possible with the resulting game state
// 2. current state together with the evaluated terminal value, which may be
//      - None: state is not terminal
//      - Some(score): terminal score of game

pub trait GameCache<State, Move> {
    fn new() -> Self;
    fn get_applied_state(&self, _state: &State, _mv: &Move) -> Option<&State> {
        None
    }
    fn insert_applied_state(&mut self, _state: &State, _mv: &Move, _result: State) {}
    fn get_terminal_value(&self, _state: &State) -> Option<&Option<f32>> {
        None
    }
    fn insert_terminal_value(&mut self, _state: &State, _value: Option<f32>) {}
}
