// miscellaneous mcts type definitions

use super::{
    ExpansionPolicy, Heuristic, MCTSCache, MCTSGame, MCTSPlayer, SimulationPolicy, UCTPolicy,
};
use rand::prelude::SliceRandom;

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

// new MCTS types
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

pub struct ExpandAll<G: MCTSGame> {
    moves: Vec<G::Move>,
}

impl<G: MCTSGame> ExpansionPolicy<G> for ExpandAll<G> {
    fn new(state: &<G as MCTSGame>::State) -> Self {
        let moves = if G::is_terminal(state) {
            vec![]
        } else {
            G::available_moves(state).collect::<Vec<_>>()
        };
        ExpandAll { moves }
    }
    fn should_expand(&self, _v: usize, _n: usize) -> bool {
        !self.moves.is_empty()
    }
    fn pop_expandable_move(&mut self, _v: usize, _n: usize) -> Option<<G as MCTSGame>::Move> {
        self.moves.pop()
    }
}

pub struct ProgressiveWidening<const C: usize, const AN: usize, const AD: usize, G: MCTSGame> {
    unexpanded_moves: Vec<G::Move>,
}

// default progressive widening with C = 2, alpha = 1/2
pub type PWDefault<G> = ProgressiveWidening<2, 1, 2, G>;

// fast progressive widening with C = 4, alpha = 1/3
pub type PWFast<G> = ProgressiveWidening<4, 1, 3, G>;

// slow progressive widening with C = 1, alpha = 2/3
pub type PWSlow<G> = ProgressiveWidening<1, 2, 3, G>;

impl<const C: usize, const AN: usize, const AD: usize, G: MCTSGame>
    ProgressiveWidening<C, AN, AD, G>
{
    fn allowed_children(visits: usize) -> usize {
        if visits == 0 {
            1
        } else {
            (C as f32 * (visits as f32).powf(AN as f32 / AD as f32)).floor() as usize
        }
    }
}

impl<const C: usize, const AN: usize, const AD: usize, G: MCTSGame> ExpansionPolicy<G>
    for ProgressiveWidening<C, AN, AD, G>
{
    fn new(state: &<G as MCTSGame>::State) -> Self {
        let unexpanded_moves = if G::is_terminal(state) {
            vec![]
        } else {
            let mut unexpanded_moves = G::available_moves(state).collect::<Vec<_>>();
            unexpanded_moves.shuffle(&mut rand::thread_rng());
            unexpanded_moves
        };
        ProgressiveWidening { unexpanded_moves }
    }
    fn should_expand(&self, visits: usize, num_parent_children: usize) -> bool {
        num_parent_children < Self::allowed_children(visits) && !self.unexpanded_moves.is_empty()
    }
    fn pop_expandable_move(
        &mut self,
        visits: usize,
        num_parent_children: usize,
    ) -> Option<<G as MCTSGame>::Move> {
        if !self.should_expand(visits, num_parent_children) {
            return None;
        }
        self.unexpanded_moves.pop()
    }
}

pub struct DefaultSimulationPolicy {}

impl SimulationPolicy for DefaultSimulationPolicy {}

pub struct HeuristicCutoff<const MXD: usize, const MNM: usize> {}

impl<const MXD: usize, const MNM: usize> SimulationPolicy for HeuristicCutoff<MXD, MNM> {
    fn should_cutoff(depth: usize, heuristic: f32, available_moves: usize) -> bool {
        depth >= MXD || available_moves <= MNM || heuristic <= 0.05 || heuristic >= 0.95
    }
}

pub struct DefaultHeuristic {}

impl<G: MCTSGame> Heuristic<G> for DefaultHeuristic {}
