// implementation and configuration of Heuristic

use super::super::{BaseHeuristicConfig, Heuristic, MCTSGame, NoHeuristicCache};

pub struct NoHeuristic {}

impl<G: MCTSGame> Heuristic<G> for NoHeuristic {
    type Cache = NoHeuristicCache<G::State, G::Move>;
    type Config = BaseHeuristicConfig;

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
