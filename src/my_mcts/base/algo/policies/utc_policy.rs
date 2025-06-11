// base implementation of UTCPolicy

use super::{MCTSConfig, MCTSGame, UCTPolicy};

pub struct StaticC {}

impl<G: MCTSGame, Config: MCTSConfig> UCTPolicy<G, Config> for StaticC {}

pub struct DynamicC {}

impl<G: MCTSGame, Config: MCTSConfig> UCTPolicy<G, Config> for DynamicC {
    fn exploration_score(
        visits: usize,
        parent_visits: usize,
        mcts_config: &Config,
        last_player: G::Player,
        perspective_player: G::Player,
    ) -> f32 {
        let factor = if last_player == perspective_player {
            1.0
        } else {
            mcts_config.non_perspective_player_exploration_boost()
        };
        let dynamic_c =
            mcts_config.exploration_constant() * factor / (1.0 + (visits as f32).sqrt());
        dynamic_c * ((parent_visits as f32).ln() / visits as f32).sqrt()
    }
}
