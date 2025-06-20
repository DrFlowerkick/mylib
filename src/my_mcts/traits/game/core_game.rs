// MCTSGame: core game trait and core trait of MCTS, since it defines State and Move

use super::{GameCache, GamePlayer};

pub trait MCTSGame: Sized + Clone + Sync + Send {
    type State: Clone + PartialEq + Sync + Send;
    type Move: Clone + Sync + Send;
    type Player: GamePlayer;
    type Cache: GameCache<Self::State, Self::Move> + Sync + Send;

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
