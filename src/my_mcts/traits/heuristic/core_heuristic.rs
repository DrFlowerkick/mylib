// Heuristic is the core trait to the define heuristic of the game, which is represented by MCTSGame.
// It defines types for
//  - configuration of heuristic
//  - caching
//
// Heuristic depends upon MCTSGame for State and Move type amongst others.

use super::{HeuristicConfig, MCTSGame, SimulationScore};

pub trait Heuristic<G: MCTSGame, S: SimulationScore<G>>: Clone + Sync + Send {
    type Config: HeuristicConfig + Clone + Sync + Send;

    fn evaluate_state(state: &G::State, heuristic_config: &Self::Config) -> S;
    fn evaluate_move(state: &G::State, mv: &G::Move, heuristic_config: &Self::Config) -> S;
}
