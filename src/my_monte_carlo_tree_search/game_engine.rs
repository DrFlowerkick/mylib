use super::*;

// trait for functions to create an game engine, implement trait on same game_data datatype as MonteCarloGameData
pub trait MonteCarloGameEngine: Copy + Clone + PartialEq {
    fn setup_game_data(&mut self, starting_player: MonteCarloPlayer, random: bool);
    fn prepare_second_player(&self) -> Self;
    fn update_random_game_data(&mut self, played_turns: usize);
    fn copy_random_game_data_second_player(
        &mut self,
        game_data_first_player: &impl MonteCarloGameData,
        played_turns: usize,
    );
}
