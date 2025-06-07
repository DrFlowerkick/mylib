// base implementation of UTCPolicy

use super::super::super::{MCTSConfig, MCTSGame, UCTPolicy};

pub struct StaticC {}

impl<G: MCTSGame> UCTPolicy<G> for StaticC {}

pub struct DynamicC {}

impl<G: MCTSGame> UCTPolicy<G> for DynamicC {
    fn exploration_score(visits: usize, parent_visits: usize, mcts_config: &G::Config) -> f32 {
        let dynamic_c = mcts_config.exploration_constant() / (1.0 + (visits as f32).sqrt());
        dynamic_c * ((parent_visits as f32).ln() / visits as f32).sqrt()
    }
}
