// caching traits for mcts including TranspositionTable

// ToDo: total refactoring. We will add Edge and will separate UTCCache
// into two traits, one for exploitation and one for exploration. They will not be
// called Cache anymore. Instead the implementing struct will decide, if it will cache or not. 

use super::{MCTSConfig, MCTSGame, UCTPolicy};

pub trait UTCCache<G, UTC, Config>: Clone + Sync + Send
where
    G: MCTSGame,
    UTC: UCTPolicy<G, Config>,
    Config: MCTSConfig<G::Player>,
{
    fn new() -> Self;

    fn update_exploitation(
        &mut self,
        visits: usize,
        accumulated_value: f32,
        last_player: G::Player,
        perspective_player: G::Player,
    );

    fn get_exploitation(
        &self,
        visits: usize,
        accumulated_value: f32,
        last_player: G::Player,
        perspective_player: G::Player,
    ) -> f32;

    fn update_exploration(
        &mut self,
        visits: usize,
        parent_visits: usize,
        mcts_config: &Config,
        last_player: G::Player,
    );

    fn get_exploration(
        &self,
        visits: usize,
        parent_visits: usize,
        mcts_config: &Config,
        last_player: G::Player,
    ) -> f32;
}
