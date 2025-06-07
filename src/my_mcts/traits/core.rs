// core traits of mcts

use super::*;

pub trait MCTSPlayer: PartialEq {
    fn next(&self) -> Self;
}

pub trait MCTSGame: Sized {
    type State: Clone + PartialEq;
    type Move;
    type Player: MCTSPlayer;
    type Cache: GameCache<Self::State, Self::Move>;
    type Config: MCTSConfig;

    fn available_moves<'a>(state: &'a Self::State) -> Box<dyn Iterator<Item = Self::Move> + 'a>;
    fn apply_move(
        state: &Self::State,
        mv: &Self::Move,
        game_cache: &mut Self::Cache,
    ) -> Self::State;
    fn evaluate(state: &Self::State, game_cache: &mut Self::Cache) -> Option<f32>;
    fn current_player(state: &Self::State) -> Self::Player;
    fn last_player(state: &Self::State) -> Self::Player;
    fn perspective_player() -> Self::Player;
}

pub trait MCTSAlgo<G: MCTSGame> {
    fn set_root(&mut self, state: &G::State) -> bool;
    fn iterate(&mut self);
    fn select_move(&self) -> &G::Move;
}
