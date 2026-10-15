// NoHeuristic can be used, if no Heuristic is feasible for your game.

use super::{Heuristic, HeuristicConfig, MCTSGame};

#[derive(Clone)]
pub struct NoHeuristic {}

impl HeuristicConfig for NoHeuristic {}

impl<G: MCTSGame> Heuristic<G> for NoHeuristic {
    type Config = Self;

    fn evaluate_state(
        state: &G::State,
        _perspective_player: Option<G::Player>,
        _heuristic_config: &Self::Config,
    ) -> f32 {
        G::evaluate(state).unwrap_or(0.5)
    }
    fn evaluate_move(_state: &G::State, _mv: &G::Move, _heuristic_config: &Self::Config) -> f32 {
        0.0
    }
}
