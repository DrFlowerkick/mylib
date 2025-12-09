// Heuristic is the core trait to the define heuristic of the game, which is represented by MCTSGame.
// It defines types for
//  - configuration of heuristic
//  - caching
//
// Heuristic depends upon MCTSGame for State and Move type amongst others.

use super::{HeuristicCache, HeuristicConfig, MCTSGame};
use rand::{rng, seq::SliceRandom};

pub trait Heuristic<G: MCTSGame>: Clone + Sync + Send {
    type Cache: HeuristicCache<G::State, G::Move> + Clone + Sync + Send;
    type Config: HeuristicConfig + Clone + Sync + Send;

    fn evaluate_state(
        state: &G::State,
        game_cache: &mut G::Cache,
        heuristic_cache: &mut Self::Cache,
        perspective_player: Option<G::Player>,
        heuristic_config: &Self::Config,
    ) -> f32;
    fn evaluate_move(
        state: &G::State,
        mv: &G::Move,
        game_cache: &mut G::Cache,
        heuristic_cache: &mut Self::Cache,
        heuristic_config: &Self::Config,
    ) -> f32;
    fn sort_moves(
        state: &G::State,
        moves: Vec<G::Move>,
        game_cache: &mut G::Cache,
        heuristic_cache: &mut Self::Cache,
        heuristic_config: &Self::Config,
    ) -> Vec<(f32, G::Move)> {
        let mut heuristic_moves = moves
            .into_iter()
            .map(|mv| {
                (
                    Self::evaluate_move(state, &mv, game_cache, heuristic_cache, heuristic_config),
                    mv,
                )
            })
            .collect::<Vec<_>>();
        heuristic_moves.shuffle(&mut rng());
        heuristic_moves
            .sort_unstable_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        heuristic_moves
    }
}
