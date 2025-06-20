// base implementation of UTCPolicy

use super::{MCTSConfig, MCTSGame, UCTPolicy};

#[derive(Clone)]
pub struct StaticC {}

impl<G, Config> UCTPolicy<G, Config> for StaticC
where
    G: MCTSGame,
    Config: MCTSConfig<G::Player>,
{
}

#[derive(Clone)]
pub struct StaticCWithExplorationBoost {}

impl<G, Config> UCTPolicy<G, Config> for StaticCWithExplorationBoost
where
    G: MCTSGame,
    Config: MCTSConfig<G::Player>,
{
    fn exploration_score(
        visits: usize,
        parent_visits: usize,
        mcts_config: &Config,
        last_player: G::Player,
    ) -> f32 {
        let factor = mcts_config.exploration_boost(last_player);
        mcts_config.exploration_constant()
            * factor
            * ((parent_visits as f32).ln() / visits as f32).sqrt()
    }
}

#[derive(Clone)]
pub struct DynamicC {}

impl<G, Config> UCTPolicy<G, Config> for DynamicC
where
    G: MCTSGame,
    Config: MCTSConfig<G::Player>,
{
    fn exploration_score(
        visits: usize,
        parent_visits: usize,
        mcts_config: &Config,
        _last_player: G::Player,
    ) -> f32 {
        let dynamic_c = mcts_config.exploration_constant() / (1.0 + (visits as f32).sqrt());
        dynamic_c * ((parent_visits as f32).ln() / visits as f32).sqrt()
    }
}

#[derive(Clone)]
pub struct DynamicCWithExplorationBoost {}

impl<G, Config> UCTPolicy<G, Config> for DynamicCWithExplorationBoost
where
    G: MCTSGame,
    Config: MCTSConfig<G::Player>,
{
    fn exploration_score(
        visits: usize,
        parent_visits: usize,
        mcts_config: &Config,
        last_player: G::Player,
    ) -> f32 {
        let factor = mcts_config.exploration_boost(last_player);
        let dynamic_c =
            mcts_config.exploration_constant() * factor / (1.0 + (visits as f32).sqrt());
        dynamic_c * ((parent_visits as f32).ln() / visits as f32).sqrt()
    }
}
