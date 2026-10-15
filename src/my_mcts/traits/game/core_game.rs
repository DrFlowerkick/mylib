// MCTSGame: core game trait and core trait of MCTS, since it defines State and Move

use super::GamePlayer;

pub trait MCTSGame: Sized + Clone + Sync + Send {
    type State: Clone + PartialEq + Sync + Send;
    type Move: Clone + Sync + Send;
    type Player: GamePlayer;

    fn available_moves<'a>(state: &'a Self::State) -> Box<dyn Iterator<Item = Self::Move> + 'a>;
    fn apply_move(state: &Self::State, mv: &Self::Move) -> Self::State;
    fn evaluate(state: &Self::State) -> Option<f32>;
    fn current_player(state: &Self::State) -> Self::Player;
    fn last_player(state: &Self::State) -> Self::Player;
    fn perspective_player() -> Self::Player;
}
