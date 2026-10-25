// base implementation of UCTCache

use super::{MCTSConfig, MCTSGame, UCTCache, UCTPolicy};

#[derive(Clone, Default)]
pub struct NoUCTCache;

impl<G, UCT, Config> UCTCache<G, UCT, Config> for NoUCTCache
where
    G: MCTSGame,
    UCT: UCTPolicy<G, Config>,
    Config: MCTSConfig<G::Player>,
{
    fn update_stats(&mut self, _result: f32, _last_player: G::Player) {}
    fn calc_uct(
        &mut self,
        _parent_visits: usize,
        _mcts_config: &Config,
        _last_player: G::Player,
    ) -> f32 {
        0.0
    }
}

#[derive(Clone, Default)]
pub struct ExploitationOnly {
    visits: usize,
    accumulated_value: f32,
}

impl<G, UCT, Config> UCTCache<G, UCT, Config> for ExploitationOnly
where
    G: MCTSGame,
    UCT: UCTPolicy<G, Config>,
    Config: MCTSConfig<G::Player>,
{
    fn calc_uct(
        &mut self,
        _parent_visits: usize,
        _mcts_config: &Config,
        last_player: G::Player,
    ) -> f32 {
        // calc only exploitation score
        UCT::exploitation_score(
            self.accumulated_value,
            self.visits,
            last_player,
            G::perspective_player(),
        )
    }

    fn update_stats(&mut self, result: f32, _last_player: G::Player) {
        self.visits += 1;
        self.accumulated_value += result;
    }

    fn get_visits(&self) -> Option<usize> {
        Some(self.visits)
    }

    fn get_accumulated_value(&self) -> Option<f32> {
        Some(self.accumulated_value)
    }
}

#[derive(Clone, Default)]
pub struct ExplorationOnly {
    visits: usize,
}

impl<G, UCT, Config> UCTCache<G, UCT, Config> for ExplorationOnly
where
    G: MCTSGame,
    UCT: UCTPolicy<G, Config>,
    Config: MCTSConfig<G::Player>,
{
    fn calc_uct(
        &mut self,
        parent_visits: usize,
        mcts_config: &Config,
        last_player: G::Player,
    ) -> f32 {
        // calc only exploration score
        UCT::exploration_score(self.visits, parent_visits, mcts_config, last_player)
    }

    fn update_stats(&mut self, _result: f32, _last_player: G::Player) {
        self.visits += 1;
    }

    fn get_visits(&self) -> Option<usize> {
        Some(self.visits)
    }
}

#[derive(Clone, Default)]
pub struct ExploitationAndExploration {
    visits: usize,
    accumulated_value: f32,
}

impl<G, UCT, Config> UCTCache<G, UCT, Config> for ExploitationAndExploration
where
    G: MCTSGame,
    UCT: UCTPolicy<G, Config>,
    Config: MCTSConfig<G::Player>,
{
    fn calc_uct(
        &mut self,
        parent_visits: usize,
        mcts_config: &Config,
        last_player: G::Player,
    ) -> f32 {
        // sum of exploitation and exploration score
        UCT::exploitation_score(
            self.accumulated_value,
            self.visits,
            last_player,
            G::perspective_player(),
        ) + UCT::exploration_score(self.visits, parent_visits, mcts_config, last_player)
    }

    fn update_stats(&mut self, result: f32, _last_player: G::Player) {
        self.visits += 1;
        self.accumulated_value += result;
    }

    fn get_visits(&self) -> Option<usize> {
        Some(self.visits)
    }

    fn get_accumulated_value(&self) -> Option<f32> {
        Some(self.accumulated_value)
    }
}

#[derive(Clone, Default)]
pub struct ExploitationOnlyWithScoreCache {
    visits: usize,
    accumulated_value: f32,
    exploitation_score: f32,
}

impl<G, UCT, Config> UCTCache<G, UCT, Config> for ExploitationOnlyWithScoreCache
where
    G: MCTSGame,
    UCT: UCTPolicy<G, Config>,
    Config: MCTSConfig<G::Player>,
{
    fn calc_uct(
        &mut self,
        _parent_visits: usize,
        _mcts_config: &Config,
        _last_player: G::Player,
    ) -> f32 {
        // return only exploitation score
        self.exploitation_score
    }

    fn update_stats(&mut self, result: f32, last_player: G::Player) {
        self.visits += 1;
        self.accumulated_value += result;
        self.exploitation_score = UCT::exploitation_score(
            self.accumulated_value,
            self.visits,
            last_player,
            G::perspective_player(),
        );
    }

    fn get_visits(&self) -> Option<usize> {
        Some(self.visits)
    }

    fn get_accumulated_value(&self) -> Option<f32> {
        Some(self.accumulated_value)
    }
}

#[derive(Clone, Default)]
pub struct ExplorationOnlyWithScoreCache {
    visits: usize,
    exploration_score: f32,
    parent_visits: usize,
}

impl<G, UCT, Config> UCTCache<G, UCT, Config> for ExplorationOnlyWithScoreCache
where
    G: MCTSGame,
    UCT: UCTPolicy<G, Config>,
    Config: MCTSConfig<G::Player>,
{
    fn calc_uct(
        &mut self,
        parent_visits: usize,
        mcts_config: &Config,
        last_player: G::Player,
    ) -> f32 {
        // update exploration score, if parent_visits changed
        if self.parent_visits != parent_visits {
            self.exploration_score =
                UCT::exploration_score(self.visits, parent_visits, mcts_config, last_player);
            self.parent_visits = parent_visits;
        }
        // return only exploration score
        self.exploration_score
    }

    fn update_stats(&mut self, _result: f32, _last_player: G::Player) {
        self.visits += 1;
    }

    fn get_visits(&self) -> Option<usize> {
        Some(self.visits)
    }
}

#[derive(Clone, Default)]
pub struct ExploitationAndExplorationWithScoreCache {
    visits: usize,
    accumulated_value: f32,
    exploitation_score: f32,
    exploration_score: f32,
    parent_visits: usize,
}

impl<G, UCT, Config> UCTCache<G, UCT, Config> for ExploitationAndExplorationWithScoreCache
where
    G: MCTSGame,
    UCT: UCTPolicy<G, Config>,
    Config: MCTSConfig<G::Player>,
{
    fn calc_uct(
        &mut self,
        parent_visits: usize,
        mcts_config: &Config,
        last_player: G::Player,
    ) -> f32 {
        // update exploration score, if parent_visits changed
        if self.parent_visits != parent_visits {
            self.exploration_score =
                UCT::exploration_score(self.visits, parent_visits, mcts_config, last_player);
            self.parent_visits = parent_visits;
        }
        // sum of exploitation and exploration score
        self.exploitation_score + self.exploration_score
    }

    fn update_stats(&mut self, result: f32, last_player: G::Player) {
        self.visits += 1;
        self.accumulated_value += result;
        self.exploitation_score = UCT::exploitation_score(
            self.accumulated_value,
            self.visits,
            last_player,
            G::perspective_player(),
        );
    }

    fn get_visits(&self) -> Option<usize> {
        Some(self.visits)
    }

    fn get_accumulated_value(&self) -> Option<f32> {
        Some(self.accumulated_value)
    }
}
