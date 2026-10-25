// caching traits used inside tree elements

use super::{MCTSConfig, MCTSGame, UCTPolicy};

pub trait UCTCache<G, UCT, Config>: Clone + Sync + Send + Default
where
    G: MCTSGame,
    UCT: UCTPolicy<G, Config>,
    Config: MCTSConfig<G::Player>,
{
    fn update_stats(&mut self, result: f32, last_player: G::Player);
    fn calc_uct(
        &mut self,
        parent_visits: usize,
        mcts_config: &Config,
        last_player: G::Player,
    ) -> f32;
    fn get_visits(&self) -> Option<usize> {
        None
    }
    fn get_accumulated_value(&self) -> Option<f32> {
        None
    }
}
