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
    fn calc_exploitation_score(&self, previous_player: G::Player) -> f32 {
        UCT::exploitation_score(self.accumulated_value, self.visits, previous_player)
    }

    fn update_stats(&mut self, result: f32, _previous_player: G::Player) {
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
    fn calc_exploration_score(
        &mut self,
        parent_visits: usize,
        mcts_config: &Config,
        previous_player: G::Player,
    ) -> f32 {
        UCT::exploration_score(self.visits, parent_visits, mcts_config, previous_player)
    }

    fn update_stats(&mut self, _result: f32, _previous_player: G::Player) {
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
    fn calc_exploitation_score(&self, previous_player: G::Player) -> f32 {
        UCT::exploitation_score(self.accumulated_value, self.visits, previous_player)
    }

    fn calc_exploration_score(
        &mut self,
        parent_visits: usize,
        mcts_config: &Config,
        previous_player: G::Player,
    ) -> f32 {
        UCT::exploration_score(self.visits, parent_visits, mcts_config, previous_player)
    }

    fn update_stats(&mut self, result: f32, _previous_player: G::Player) {
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
    fn calc_exploitation_score(&self, _previous_player: G::Player) -> f32 {
        self.exploitation_score
    }

    fn update_stats(&mut self, result: f32, previous_player: G::Player) {
        self.visits += 1;
        self.accumulated_value += result;
        self.exploitation_score =
            UCT::exploitation_score(self.accumulated_value, self.visits, previous_player);
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
    fn calc_exploration_score(
        &mut self,
        parent_visits: usize,
        mcts_config: &Config,
        previous_player: <G as MCTSGame>::Player,
    ) -> f32 {
        // update exploration score, if parent_visits changed
        if self.parent_visits != parent_visits {
            self.exploration_score =
                UCT::exploration_score(self.visits, parent_visits, mcts_config, previous_player);
            self.parent_visits = parent_visits;
        }
        self.exploration_score
    }

    fn update_stats(&mut self, _result: f32, _previous_player: G::Player) {
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
    fn calc_exploitation_score(&self, _previous_player: G::Player) -> f32 {
        self.exploitation_score
    }

    fn calc_exploration_score(
        &mut self,
        parent_visits: usize,
        mcts_config: &Config,
        previous_player: <G as MCTSGame>::Player,
    ) -> f32 {
        // update exploration score, if parent_visits changed
        if self.parent_visits != parent_visits {
            self.exploration_score =
                UCT::exploration_score(self.visits, parent_visits, mcts_config, previous_player);
            self.parent_visits = parent_visits;
        }
        self.exploration_score
    }

    fn update_stats(&mut self, result: f32, previous_player: G::Player) {
        self.visits += 1;
        self.accumulated_value += result;
        self.exploitation_score =
            UCT::exploitation_score(self.accumulated_value, self.visits, previous_player);
    }

    fn get_visits(&self) -> Option<usize> {
        Some(self.visits)
    }

    fn get_accumulated_value(&self) -> Option<f32> {
        Some(self.accumulated_value)
    }
}
