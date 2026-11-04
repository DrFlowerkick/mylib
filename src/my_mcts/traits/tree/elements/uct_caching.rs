// caching traits used inside tree elements

use super::{MCTSConfig, MCTSGame, UCTPolicy};

pub trait UCTCache<G, UCT, Config>: Clone + Sync + Send + Default
where
    G: MCTSGame,
    UCT: UCTPolicy<G, Config>,
    Config: MCTSConfig<G::Player>,
{
    fn update_stats(&mut self, _result: f32, _previous_player: G::Player) {}
    fn calc_exploitation_score(&self, _previous_player: G::Player) -> f32 {
        0.0
    }
    fn calc_exploration_score(
        &mut self,
        _parent_visits: usize,
        _mcts_config: &Config,
        _previous_player: G::Player,
    ) -> f32 {
        0.0
    }
    fn get_visits(&self) -> Option<usize> {
        None
    }
    fn get_accumulated_value(&self) -> Option<f32> {
        None
    }
}
