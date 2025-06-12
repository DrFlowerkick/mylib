// base implementation of UTCCache

use super::{MCTSConfig, MCTSGame, UCTPolicy, UTCCache};

pub struct NoUTCCache;

impl<G, UTC, Config> UTCCache<G, UTC, Config> for NoUTCCache
where
    G: MCTSGame,
    UTC: UCTPolicy<G, Config>,
    Config: MCTSConfig<G::Player>,
{
    fn new() -> Self {
        NoUTCCache
    }

    fn update_exploitation(&mut self, _v: usize, _a: f32, _l: G::Player, _p: G::Player) {}
    fn get_exploitation(
        &self,
        visits: usize,
        accumulated_value: f32,
        last_player: G::Player,
        perspective_player: G::Player,
    ) -> f32 {
        UTC::exploitation_score(accumulated_value, visits, last_player, perspective_player)
    }

    fn update_exploration(
        &mut self,
        _visits: usize,
        _parent_visits: usize,
        _mcts_config: &Config,
        _last_player: G::Player,
    ) {
    }
    fn get_exploration(
        &self,
        visits: usize,
        parent_visits: usize,
        mcts_config: &Config,
        last_player: G::Player,
    ) -> f32 {
        UTC::exploration_score(
            visits,
            parent_visits,
            mcts_config,
            last_player,
        )
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
    Config: MCTSConfig<G::Player>,
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
        accumulated_value: f32,
        last_player: G::Player,
        perspective_player: G::Player,
    ) {
        self.exploitation =
            UTC::exploitation_score(accumulated_value, visits, last_player, perspective_player);
    }

    fn get_exploitation(
        &self,
        _visits: usize,
        _accumulated_value: f32,
        _last_player: G::Player,
        _perspective_player: G::Player,
    ) -> f32 {
        self.exploitation
    }

    fn update_exploration(
        &mut self,
        visits: usize,
        parent_visits: usize,
        mcts_config: &Config,
        last_player: G::Player,
    ) {
        if self.last_parent_visits != parent_visits {
            self.exploration = UTC::exploration_score(
                visits,
                parent_visits,
                mcts_config,
                last_player,
            );
            self.last_parent_visits = parent_visits;
        }
    }

    fn get_exploration(
        &self,
        _visits: usize,
        _parent_visits: usize,
        _mcts_config: &Config,
        _last_player: G::Player,
    ) -> f32 {
        self.exploration
    }
}
