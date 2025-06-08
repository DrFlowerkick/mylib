// base implementation of UTCCache

use super::{MCTSConfig, MCTSGame, UCTPolicy, UTCCache};

pub struct NoUTCCache;

impl<G, UTC, Config> UTCCache<G, UTC, Config> for NoUTCCache
where
    G: MCTSGame,
    UTC: UCTPolicy<G, Config>,
    Config: MCTSConfig,
{
    fn new() -> Self {
        NoUTCCache
    }

    fn update_exploitation(&mut self, _v: usize, _a: f32, _l: G::Player, _p: G::Player) {}
    fn get_exploitation(
        &self,
        visits: usize,
        acc_value: f32,
        last_player: G::Player,
        perspective_player: G::Player,
    ) -> f32 {
        UTC::exploitation_score(acc_value, visits, last_player, perspective_player)
    }

    fn update_exploration(&mut self, _v: usize, _p: usize, _mc: &Config) {}
    fn get_exploration(&self, visits: usize, parent_visits: usize, mcts_config: &Config) -> f32 {
        UTC::exploration_score(visits, parent_visits, mcts_config)
    }
}

pub struct CachedUTC {
    exploitation: f32,
    exploration: f32,
    last_parent_visits: usize,
}

impl<G, UTC, Config> UTCCache<G, UTC, Config> for CachedUTC
where
    G: MCTSGame,
    UTC: UCTPolicy<G, Config>,
    Config: MCTSConfig,
{
    fn new() -> Self {
        CachedUTC {
            exploitation: 0.0,
            exploration: 0.0,
            last_parent_visits: 0,
        }
    }

    fn update_exploitation(
        &mut self,
        visits: usize,
        acc_value: f32,
        last_player: G::Player,
        perspective_player: G::Player,
    ) {
        self.exploitation =
            UTC::exploitation_score(acc_value, visits, last_player, perspective_player);
    }

    fn get_exploitation(&self, _v: usize, _a: f32, _c: G::Player, _p: G::Player) -> f32 {
        self.exploitation
    }

    fn update_exploration(&mut self, visits: usize, parent_visits: usize, mcts_config: &Config) {
        if self.last_parent_visits != parent_visits {
            self.exploration = UTC::exploration_score(visits, parent_visits, mcts_config);
            self.last_parent_visits = parent_visits;
        }
    }

    fn get_exploration(&self, _v: usize, _p: usize, _mc: &Config) -> f32 {
        self.exploration
    }
}
