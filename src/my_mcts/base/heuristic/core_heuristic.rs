// NoHeuristic can be used, if no Heuristic is feasible for your game.

use super::{Heuristic, HeuristicConfig, MCTSGame, SimulationScore};

#[derive(Clone)]
pub struct NoHeuristic {}

impl HeuristicConfig for NoHeuristic {}

impl<G: MCTSGame, S: SimulationScore<G>> Heuristic<G, S> for NoHeuristic {
    type Config = Self;

    fn evaluate_state(state: &G::State, _heuristic_config: &Self::Config) -> S {
        G::evaluate(state).map_or_else(|| S::default(), |o| o.into())
    }
    fn evaluate_move(_state: &G::State, _mv: &G::Move, _heuristic_config: &Self::Config) -> S {
        S::default()
    }
}
