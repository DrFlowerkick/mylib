// miscellaneous mcts type definitions

use super::{MCTSCache, MCTSGame, MCTSPlayer, UCTPolicy};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub enum MonteCarloPlayer {
    #[default]
    Me,
    Opp,
}

impl MonteCarloPlayer {
    pub fn next_player(&self) -> Self {
        match self {
            MonteCarloPlayer::Me => MonteCarloPlayer::Opp,
            MonteCarloPlayer::Opp => MonteCarloPlayer::Me,
        }
    }
}

impl MCTSPlayer for MonteCarloPlayer {
    fn next(&self) -> Self {
        match self {
            MonteCarloPlayer::Me => MonteCarloPlayer::Opp,
            MonteCarloPlayer::Opp => MonteCarloPlayer::Me,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum MonteCarloNodeType {
    GameDataUpdate,
    ActionResult,
}

#[derive(Copy, Clone, PartialEq)]
// each game mode describes a different handling of player actions, see below
// normally each player has one action
// if multiple actions per player are possible, than starting_player does his actions, afterward the other player. this is true for every mode
pub enum MonteCarloGameMode {
    SameTurnParallel, // both players act parallel on same turn. Actions change game data at the same time
    ByTurns,          // each turn only one player acts, players switch at turn end
}

#[derive(Copy, Clone, PartialEq)]
pub enum MonteCarloNodeConsistency {
    Inconsistent,
    Consistent,
    ConsistentNeedsUpdate,
    PossibleFutureGameState,
}

pub struct StaticC {}

impl<G: MCTSGame> UCTPolicy<G> for StaticC {}

pub struct DynamicC {}

impl<G: MCTSGame> UCTPolicy<G> for DynamicC {
    fn exploration_score(visits: usize, parent_visits: usize, c: f32) -> f32 {
        let dynamic_c = c / (1.0 + (visits as f32).sqrt());
        dynamic_c * ((parent_visits as f32).ln() / visits as f32).sqrt()
    }
}

pub struct NoCache;

impl<G: MCTSGame, P: UCTPolicy<G>> MCTSCache<G, P> for NoCache {
    fn new() -> Self {
        NoCache
    }

    fn update_exploitation(&mut self, _v: usize, _a: f32, _c: G::Player, _p: G::Player) {}
    fn get_exploitation(
        &self,
        visits: usize,
        acc_value: f32,
        current_player: G::Player,
        perspective_player: G::Player,
    ) -> f32 {
        P::exploitation_score(acc_value, visits, current_player, perspective_player)
    }

    fn update_exploration(&mut self, _v: usize, _p: usize, _b: f32) {}
    fn get_exploration(&self, visits: usize, parent_visits: usize, base_c: f32) -> f32 {
        P::exploration_score(visits, parent_visits, base_c)
    }
}

pub struct WithCache {
    exploitation: f32,
    exploration: f32,
    last_parent_visits: usize,
}

impl<G: MCTSGame, P: UCTPolicy<G>> MCTSCache<G, P> for WithCache {
    fn new() -> Self {
        WithCache {
            exploitation: 0.0,
            exploration: 0.0,
            last_parent_visits: 0,
        }
    }

    fn update_exploitation(
        &mut self,
        visits: usize,
        acc_value: f32,
        current_player: G::Player,
        perspective_player: G::Player,
    ) {
        self.exploitation =
            P::exploitation_score(acc_value, visits, current_player, perspective_player);
    }

    fn get_exploitation(&self, _v: usize, _a: f32, _c: G::Player, _p: G::Player) -> f32 {
        self.exploitation
    }

    fn update_exploration(&mut self, visits: usize, parent_visits: usize, base_c: f32) {
        if self.last_parent_visits != parent_visits {
            self.exploration = P::exploration_score(visits, parent_visits, base_c);
            self.last_parent_visits = parent_visits;
        }
    }

    fn get_exploration(&self, _v: usize, _p: usize, _b: f32) -> f32 {
        self.exploration
    }
}
