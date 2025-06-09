// NoHeuristic can be used, if no Heuristic is feasible for your game.

use super::{Heuristic, HeuristicConfig, MCTSGame, NoHeuristicCache};

pub struct NoHeuristic {}

impl HeuristicConfig for NoHeuristic {}

impl<G: MCTSGame> Heuristic<G> for NoHeuristic {
    type Cache = NoHeuristicCache<G::State, G::Move>;
    type Config = Self;

    fn evaluate_state(
        state: &G::State,
        game_cache: &mut G::Cache,
        _heuristic_cache: &mut Self::Cache,
        _perspective_player: Option<G::Player>,
        _heuristic_config: &Self::Config,
    ) -> f32 {
        G::evaluate(state, game_cache).unwrap_or(0.5)
    }
    fn evaluate_move(
        _state: &G::State,
        _mv: &G::Move,
        _game_cache: &mut G::Cache,
        _heuristic_cache: &mut Self::Cache,
        _heuristic_config: &Self::Config,
    ) -> f32 {
        0.0
    }
}
